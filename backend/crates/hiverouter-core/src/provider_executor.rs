//! Production provider adapter: the one place where the tested DeepSeek client
//! meets the run engine. It never invents output, never logs the key and never
//! sends a call that failed the per-run budget preflight.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use hiverouter_contract::{Refusal, RunId, TaskId};
use hiverouter_provider::{
    BudgetLimits, CallEstimate, ChatMessage, ChatRequest, ClientConfig, Completion, DeepSeekClient,
    RunBudget, StreamEvent,
};
use hiverouter_provider::dto::ResponseFormat;
use tokio_util::sync::CancellationToken;

use crate::config::ProviderSettings;
use crate::entities::TaskRecord;
use crate::execution::{DeltaSink, ProviderExecutor, StreamedReply};
use crate::planner::{Plan, PlanNode};

const PLANNER_SYSTEM: &str = "You are the HiveMind planner. Return ONE JSON object only, \
no markdown fences, no prose. Schema: {\"nodes\":[{\"id\":\"n1\",\"role\":\"worker\",\
\"deps\":[],\"depth\":1}]}. Use short string ids; every dep must be the id of another node; \
depth must be >= 1 and <= 3. Produce the smallest useful DAG (1..=40 nodes).";

const TASK_SYSTEM: &str = "You are a HiveMind agent. Answer the assigned task directly and \
concisely. Do not claim tool calls or side effects you did not perform. If the task cannot be \
done from the provided context, say exactly what is missing.";

/// Production executor backed by the single `DeepSeekClient`.
pub struct ProviderExecutorAdapter {
    client: DeepSeekClient,
    settings: ProviderSettings,
    budgets: tokio::sync::Mutex<HashMap<RunId, RunBudget>>,
}

impl std::fmt::Debug for ProviderExecutorAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderExecutorAdapter")
            .field("model", &self.settings.model.id())
            .field("planner_thinking", &self.settings.planner_thinking)
            .field("leaf_thinking", &self.settings.leaf_thinking)
            .finish_non_exhaustive()
    }
}

impl ProviderExecutorAdapter {
    /// Builds the adapter with the documented client configuration.
    ///
    /// # Errors
    /// Returns the provider error when the key is absent, so the server can
    /// fall back to the honest unconfigured executor instead of panicking.
    pub fn from_env(settings: ProviderSettings) -> Result<Self, hiverouter_provider::ProviderError> {
        let client = DeepSeekClient::from_env(ClientConfig::default())?;
        Ok(Self::new(client, settings))
    }

    /// Builds the adapter over an explicit client (tests inject wiremock).
    #[must_use]
    pub fn new(client: DeepSeekClient, settings: ProviderSettings) -> Self {
        Self {
            client,
            settings,
            budgets: tokio::sync::Mutex::new(HashMap::new()),
        }
    }

    async fn complete(
        &self,
        run_id: RunId,
        request: &ChatRequest,
        max_output_tokens: u32,
    ) -> Result<Completion, Refusal> {
        let estimate = CallEstimate {
            prompt_tokens: None,
            max_output_tokens: Some(u64::from(max_output_tokens)),
        };
        let mut budgets = self.budgets.lock().await;
        let budget = budgets
            .entry(run_id)
            .or_insert_with(|| RunBudget::new(BudgetLimits::default()));
        self.client
            .complete_with_budget(budget, request, &estimate, now_unix_secs())
            .await
            .map_err(|error| {
                tracing::warn!(
                    code = error.code(),
                    detail = error.detail(),
                    run_id = %run_id,
                    "provider call refused"
                );
                error.refusal()
            })
    }
    /// Drives one budgeted leaf stream: forwards every text fragment to
    /// `on_delta` as it arrives, settles usage from the final documented chunk,
    /// and returns the concatenated text for persistence.
    ///
    /// The stream is never replayed: any failure after the first fragment (or
    /// at any point) returns the named refusal without a retry. Absent or
    /// partial usage settles as INCOMPLETE (`E_USAGE_INCOMPLETE`), never 0.
    async fn stream_reply(
        &self,
        run_id: RunId,
        request: &ChatRequest,
        max_output_tokens: u32,
        cancel: &CancellationToken,
        on_delta: &DeltaSink,
    ) -> Result<StreamedReply, Refusal> {
        let estimate = CallEstimate {
            prompt_tokens: None,
            max_output_tokens: Some(u64::from(max_output_tokens)),
        };
        let mut budgets = self.budgets.lock().await;
        let budget = budgets
            .entry(run_id)
            .or_insert_with(|| RunBudget::new(BudgetLimits::default()));
        let mut handle = self
            .client
            .stream_with_budget(budget, request, &estimate)
            .await
            .map_err(|error| {
                tracing::warn!(
                    code = error.code(),
                    detail = error.detail(),
                    run_id = %run_id,
                    "provider stream refused"
                );
                error.refusal()
            })?;

        let mut text = String::new();
        let mut tool_calls_seen = false;
        let (usage, finish_reason) = loop {
            let next = tokio::select! {
                _ = cancel.cancelled() => {
                    let _ = budget.record_call(self.settings.model, None, now_unix_secs());
                    return Err(Refusal::ToolCancelled);
                }
                event = handle.next() => event,
            };
            match next {
                None => {
                    // The client sends an explicit error before closing when
                    // the stream ends without [DONE]; reaching here means the
                    // pump dropped without a terminal event.
                    let _ = budget.record_call(self.settings.model, None, now_unix_secs());
                    return Err(Refusal::ProviderIncomplete);
                }
                Some(Err(error)) => {
                    tracing::warn!(
                        code = error.code(),
                        detail = error.detail(),
                        run_id = %run_id,
                        "provider stream failed after {} bytes of reply text",
                        text.len()
                    );
                    // Spend after a partial stream is unknown: record the call
                    // as INCOMPLETE so no report can show a fabricated cost.
                    let _ = budget.record_call(self.settings.model, None, now_unix_secs());
                    return Err(error.refusal());
                }
                Some(Ok(StreamEvent::Delta { content, .. })) => {
                    if content.is_empty() {
                        continue;
                    }
                    text.push_str(&content);
                    on_delta(&content);
                }
                Some(Ok(StreamEvent::ToolCallDelta { .. })) => {
                    tool_calls_seen = true;
                }
                Some(Ok(StreamEvent::Finished {
                    usage: reported,
                    finish_reason: reason,
                })) => {
                    break (reported, reason);
                }
            }
        };
        // The final documented content chunk carries the usage statistics;
        // absent or partial usage is INCOMPLETE (`E_USAGE_INCOMPLETE`), never 0.
        budget
            .record_call(self.settings.model, usage.as_ref(), now_unix_secs())
            .map_err(|error| {
                tracing::warn!(
                    code = error.code(),
                    detail = error.detail(),
                    run_id = %run_id,
                    "stream usage could not be settled"
                );
                error.refusal()
            })?;
        if finish_reason.as_deref() == Some("length") {
            tracing::warn!(
                finish_reason = ?finish_reason,
                run_id = %run_id,
                "provider truncated the reply at the output-token cap"
            );
            return Err(Refusal::ProviderIncomplete);
        }
        if text.is_empty() {
            tracing::warn!(
                finish_reason = ?finish_reason,
                tool_calls_seen,
                run_id = %run_id,
                "provider stream produced no visible text"
            );
            return Err(Refusal::ProviderInvalidResponse);
        }
        Ok(StreamedReply {
            deltas: vec![text],
            usage: usage.unwrap_or_default(),
        })
    }
}

#[async_trait]
impl ProviderExecutor for ProviderExecutorAdapter {
    async fn plan(
        &self,
        run_id: RunId,
        prompt: &str,
        cancel: &CancellationToken,
    ) -> Result<Plan, Refusal> {
        let request = ChatRequest::new(
            self.settings.model,
            vec![
                ChatMessage::system(PLANNER_SYSTEM),
                ChatMessage::user(format!(
                    "Create the execution-plan DAG for this user task. Return JSON only.\n\nTask:\n{prompt}"
                )),
            ],
        )
        .with_thinking(self.settings.planner_thinking)
        .with_reasoning_effort(self.settings.planner_reasoning_effort)
        .with_max_tokens(self.settings.planner_max_tokens)
        .with_response_format(ResponseFormat::json_object());
        let completion = tokio::select! {
            _ = cancel.cancelled() => return Err(Refusal::ToolCancelled),
            result = self.complete(run_id, &request, self.settings.planner_max_tokens) => result?,
        };
        let content = message_text(&completion)?;
        let wire: WirePlan = serde_json::from_str(strip_fences(&content))
            .map_err(|_| Refusal::SchemaInvalid)?;
        wire.into_plan()
    }

    async fn execute(
        &self,
        run_id: RunId,
        task: &TaskRecord,
        prompt: &str,
        cancel: &CancellationToken,
        on_delta: &DeltaSink,
    ) -> Result<StreamedReply, Refusal> {
        let task_prompt = format!(
            "Original user prompt:\n{prompt}\n\nAssigned role: {}\nSpawn depth: {}\n\
             Return the final answer for this task only.",
            task.role, task.depth
        );
        let request = ChatRequest::new(
            self.settings.model,
            vec![
                ChatMessage::system(TASK_SYSTEM),
                ChatMessage::user(task_prompt),
            ],
        )
        .with_thinking(self.settings.leaf_thinking)
        .with_max_tokens(self.settings.leaf_max_tokens);
        self.stream_reply(
            run_id,
            &request,
            self.settings.leaf_max_tokens,
            cancel,
            on_delta,
        )
        .await
    }

    async fn finish_run(&self, run_id: RunId) {
        self.budgets.lock().await.remove(&run_id);
    }
}

fn message_text(completion: &Completion) -> Result<String, Refusal> {
    if let Some(content) = completion
        .response
        .first_message()
        .and_then(|message| message.content.clone())
        .filter(|content| !content.trim().is_empty())
    {
        return Ok(content);
    }
    let finish = completion
        .response
        .choices
        .first()
        .and_then(|choice| choice.finish_reason.clone());
    let has_tool_calls = completion
        .response
        .first_message()
        .and_then(|message| message.tool_calls.as_ref())
        .is_some_and(|calls| !calls.is_empty());
    tracing::warn!(
        response_id = %completion.response.id,
        finish_reason = ?finish,
        has_tool_calls,
        "provider returned no visible content"
    );
    Err(Refusal::ProviderInvalidResponse)
}

fn strip_fences(raw: &str) -> &str {
    let trimmed = raw.trim();
    if let Some(rest) = trimmed.strip_prefix("```json") {
        return rest.trim_end_matches("```").trim();
    }
    if let Some(rest) = trimmed.strip_prefix("```") {
        return rest.trim_end_matches("```").trim();
    }
    trimmed
}

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[derive(Debug, serde::Deserialize)]
struct WirePlan {
    nodes: Vec<WireNode>,
}

#[derive(Debug, serde::Deserialize)]
struct WireNode {
    id: String,
    role: String,
    #[serde(default)]
    deps: Vec<String>,
    #[serde(default = "one")]
    depth: u8,
}

const fn one() -> u8 {
    1
}

impl WirePlan {
    fn into_plan(self) -> Result<Plan, Refusal> {
        let mut ids: HashMap<String, TaskId> = HashMap::new();
        for node in &self.nodes {
            if node.id.trim().is_empty() || ids.contains_key(&node.id) {
                return Err(Refusal::SchemaInvalid);
            }
            ids.insert(node.id.clone(), TaskId::new());
        }
        let mut nodes = Vec::with_capacity(self.nodes.len());
        for node in self.nodes {
            let node_id = *ids.get(&node.id).ok_or(Refusal::SchemaInvalid)?;
            let mut deps = Vec::with_capacity(node.deps.len());
            for dep in node.deps {
                deps.push(*ids.get(&dep).ok_or(Refusal::SchemaInvalid)?);
            }
            if node.role.trim().is_empty() || node.depth == 0 || node.depth > 3 {
                return Err(Refusal::SchemaInvalid);
            }
            nodes.push(PlanNode {
                node_id,
                role: node.role,
                deps,
                depth: node.depth,
            });
        }
        let plan = Plan::new(nodes);
        plan.validate()?;
        Ok(plan)
    }
}

/// Unused model/effort imports are kept as compile-time documentation of the
/// documented vocabulary; the adapter exposes both through `ProviderSettings`.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_plan_rejects_duplicate_ids() {
        let wire = WirePlan {
            nodes: vec![
                WireNode {
                    id: "a".to_owned(),
                    role: "worker".to_owned(),
                    deps: vec![],
                    depth: 1,
                },
                WireNode {
                    id: "a".to_owned(),
                    role: "worker".to_owned(),
                    deps: vec![],
                    depth: 1,
                },
            ],
        };
        assert_eq!(wire.into_plan(), Err(Refusal::SchemaInvalid));
    }

    #[test]
    fn wire_plan_maps_ids_and_deps() {
        let wire = WirePlan {
            nodes: vec![
                WireNode {
                    id: "a".to_owned(),
                    role: "super".to_owned(),
                    deps: vec![],
                    depth: 1,
                },
                WireNode {
                    id: "b".to_owned(),
                    role: "worker".to_owned(),
                    deps: vec!["a".to_owned()],
                    depth: 2,
                },
            ],
        };
        let plan = wire.into_plan().expect("valid plan");
        assert_eq!(plan.nodes.len(), 2);
        assert_eq!(plan.nodes[1].deps.len(), 1);
        assert_eq!(plan.nodes[1].deps[0], plan.nodes[0].node_id);
    }

    #[test]
    fn fences_are_stripped() {
        assert_eq!(strip_fences("```json\n{\"nodes\":[]}\n```"), "{\"nodes\":[]}");
    }
}

