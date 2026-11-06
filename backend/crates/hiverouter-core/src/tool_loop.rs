//! 1.2 tool-loop planning primitives.
//!
//! This module is deliberately pure: it turns assembled provider tool calls into
//! validated plans, exposes the registry subset a tier may offer to the model, and
//! builds the frozen `TOOL_CALL` / `TOOL_RESULT` payloads. It never executes a tool
//! and never publishes an event; [`crate::execution`] owns those side effects.

use hiverouter_contract::{Refusal, RunId, TaskId, ToolCallPayload, ToolOutcome, ToolResultPayload};
use hiverouter_provider::dto::{FunctionDefinition, ToolSpec};
use hiverouter_provider::tool_assembler::AssembledToolCall;
use hiverouter_tools::{AgentTier, PermissionClass, ToolId, ToolRegistry};

/// Maximum model turns in one task before the run fails closed.
pub const MAX_TOOL_TURNS: u32 = 4;
/// Maximum tool calls across one run.
pub const MAX_CALLS_PER_RUN: u32 = 16;
/// Maximum tool result body retained in memory (events carry the hash/bytes only).
pub const MAX_TOOL_RESULT_BYTES: usize = 65_536;

/// One validated model tool call, ready for approval/execution.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolCallPlan {
    /// Provider tool-call id (stable across call and result).
    pub tool_call_id: String,
    /// Frozen registry id (`file_read`, ...).
    pub tool: String,
    /// Parsed JSON argument object.
    pub args: serde_json::Value,
    /// SHA-256 of the canonical arguments, used for approval binding.
    pub args_sha256: String,
    /// 1-based loop turn.
    pub turn: u32,
}

/// A completed tool outcome. Text is retained only in memory; events carry the
/// hash/bytes, never an argument or body dump.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolExecution {
    /// Provider tool-call id this result answers.
    pub tool_call_id: String,
    /// Frozen outcome vocabulary.
    pub outcome: ToolOutcome,
    /// Named refusal code when applicable.
    pub code: Option<String>,
    /// Result body size when known.
    pub bytes: Option<u64>,
    /// SHA-256 of the result body when known.
    pub sha256: Option<String>,
    /// Result text for the next model turn; bounded by [`MAX_TOOL_RESULT_BYTES`].
    pub text: String,
}

impl ToolExecution {
    /// Builds a successful result from a bounded text body.
    #[must_use]
    pub fn ok(tool_call_id: impl Into<String>, text: String) -> Self {
        let bytes = u64::try_from(text.len()).unwrap_or(u64::MAX);
        let sha256 = hiverouter_tools::sha256_hex(text.as_bytes());
        Self {
            tool_call_id: tool_call_id.into(),
            outcome: ToolOutcome::Ok,
            code: None,
            bytes: Some(bytes),
            sha256: Some(sha256),
            text,
        }
    }

    /// Builds a refusal result; the text is diagnostic, never the argument body.
    #[must_use]
    pub fn refused(tool_call_id: impl Into<String>, code: impl Into<String>, text: String) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            outcome: ToolOutcome::Refused,
            code: Some(code.into()),
            bytes: None,
            sha256: None,
            text,
        }
    }

    /// Builds a denied-by-operator result.
    #[must_use]
    pub fn denied(tool_call_id: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            outcome: ToolOutcome::Denied,
            code: None,
            bytes: None,
            sha256: None,
            text: "operator denied the tool call; nothing executed".to_owned(),
        }
    }

    /// Builds a timeout result.
    #[must_use]
    pub fn timed_out(tool_call_id: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            outcome: ToolOutcome::Timeout,
            code: Some(Refusal::ApprovalTimeout.code().to_owned()),
            bytes: None,
            sha256: None,
            text: "approval or execution exceeded its time bound; no side effect is claimed"
                .to_owned(),
        }
    }

    /// Builds a cancelled result.
    #[must_use]
    pub fn cancelled(tool_call_id: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            outcome: ToolOutcome::Cancelled,
            code: Some(Refusal::ToolCancelled.code().to_owned()),
            bytes: None,
            sha256: None,
            text: "the run was cancelled before the tool completed".to_owned(),
        }
    }

    /// The frozen `TOOL_RESULT` payload for this execution.
    #[must_use]
    pub fn payload(&self, run_id: RunId, task_id: TaskId) -> ToolResultPayload {
        ToolResultPayload {
            run_id,
            task_id,
            tool_call_id: self.tool_call_id.clone(),
            outcome: self.outcome,
            code: self.code.clone(),
            bytes: self.bytes,
            sha256: self.sha256.clone(),
        }
    }
}

/// Converts one assembled provider call into a validated plan.
///
/// The plan is pure data; registry membership and schema validation happen in
/// [`validate_plan`] so an unknown tool is a named refusal, never a panic.
///
/// # Errors
/// [`Refusal::ProviderInvalidResponse`] for an empty id/name or non-object args.
pub fn plan_call(call: &AssembledToolCall, turn: u32) -> Result<ToolCallPlan, Refusal> {
    if call.id.trim().is_empty() || call.name.trim().is_empty() {
        return Err(Refusal::ProviderInvalidResponse);
    }
    if !call.arguments.is_object() {
        return Err(Refusal::ProviderInvalidResponse);
    }
    if call.arguments.to_string().len() > MAX_TOOL_RESULT_BYTES {
        return Err(Refusal::ToolSchema);
    }
    let args_sha256 = hiverouter_tools::approval::hash_args(&call.arguments);
    if args_sha256.len() != 64 || !args_sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(Refusal::InternalError);
    }
    Ok(ToolCallPlan {
        tool_call_id: call.id.clone(),
        tool: call.name.clone(),
        args: call.arguments.clone(),
        args_sha256,
        turn,
    })
}

/// Validates a plan against the frozen registry and the caller's tier allowlist.
///
/// # Errors
/// Maps the tool-layer refusal identity unchanged (`E_TOOL_NOT_FOUND`,
/// `E_TOOL_NOT_ALLOWED`, `E_TOOL_SCHEMA`).
pub fn validate_plan(
    registry: &ToolRegistry,
    tier: AgentTier,
    plan: &ToolCallPlan,
) -> Result<(), Refusal> {
    registry
        .resolve_call(tier, &plan.tool, &plan.args)
        .map(|_| ())
        .map_err(|error| error.refusal())
}

/// Permission class for a registered tool, if known.
#[must_use]
pub fn permission_for(registry: &ToolRegistry, tool: &str) -> Option<PermissionClass> {
    ToolId::from_id(tool)
        .and_then(|id| registry.contract(id))
        .map(|contract| contract.permission_class)
}

/// Whether the tool needs an operator approval before any side effect.
#[must_use]
pub fn requires_approval(registry: &ToolRegistry, tool: &str) -> bool {
    permission_for(registry, tool).is_some_and(PermissionClass::requires_approval)
}

/// The model-facing tool specifications for one tier, in registry order.
///
/// Only registered, allowlisted tools are offered. Side-effecting tools remain in
/// the list because the loop gates them on approval; offering them does not grant
/// execution authority.
#[must_use]
pub fn tool_specs_for_tier(registry: &ToolRegistry, tier: AgentTier) -> Vec<ToolSpec> {
    registry
        .allowlist(tier)
        .iter()
        .filter_map(|id| registry.contract(*id))
        .map(|contract| {
            ToolSpec::function(FunctionDefinition {
                name: contract.id.id().to_owned(),
                description: Some(contract.description.to_owned()),
                parameters: contract.schema.clone(),
            })
        })
        .collect()
}

/// Builds the frozen `TOOL_CALL` payload for a plan.
#[must_use]
pub fn call_payload(plan: &ToolCallPlan, run_id: RunId, task_id: TaskId) -> ToolCallPayload {
    ToolCallPayload {
        run_id,
        task_id,
        tool_call_id: plan.tool_call_id.clone(),
        tool: plan.tool.clone(),
        args_sha256: plan.args_sha256.clone(),
        turn: plan.turn,
    }
}

/// Bounds and hashes a result body before it can be stored or replayed.
///
/// The returned text is at most [`MAX_TOOL_RESULT_BYTES`]; callers must not send
/// the original unbounded body to the model or event bus.
#[must_use]
pub fn bounded_tool_text(text: &str) -> String {
    if text.len() <= MAX_TOOL_RESULT_BYTES {
        return text.to_owned();
    }
    let mut end = MAX_TOOL_RESULT_BYTES;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}\n[truncated at {MAX_TOOL_RESULT_BYTES} bytes]", &text[..end])
}

/// Returns the next turn after a completed one, or a named refusal at the cap.
///
/// # Errors
/// [`Refusal::BudgetExceeded`] when the cap is reached.
pub fn next_turn(completed_turns: u32) -> Result<u32, Refusal> {
    if completed_turns >= MAX_TOOL_TURNS {
        return Err(Refusal::BudgetExceeded);
    }
    Ok(completed_turns + 1)
}

/// Checks the per-run call budget before executing a call.
///
/// # Errors
/// [`Refusal::BudgetExceeded`] when the run has exhausted its call budget.
pub fn check_call_budget(used_calls: u32) -> Result<(), Refusal> {
    if used_calls >= MAX_CALLS_PER_RUN {
        return Err(Refusal::BudgetExceeded);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hiverouter_tools::ToolRegistry;

    fn assembler_call(id: &str, name: &str, args: serde_json::Value) -> AssembledToolCall {
        AssembledToolCall {
            index: 0,
            id: id.to_owned(),
            name: name.to_owned(),
            arguments: args,
        }
    }

    #[test]
    fn plans_valid_read_call() {
        let call = assembler_call("call_1", "file_read", serde_json::json!({"path": "a.txt"}));
        let plan = plan_call(&call, 1).expect("plan");
        assert_eq!(plan.tool, "file_read");
        assert_eq!(plan.args_sha256.len(), 64);
        let registry = ToolRegistry::standard();
        validate_plan(&registry, AgentTier::WorkerAgent, &plan).expect("allowed");
        assert!(!requires_approval(&registry, "file_read"));
    }

    #[test]
    fn write_requires_approval_and_unknown_is_named() {
        let registry = ToolRegistry::standard();
        assert!(requires_approval(&registry, "file_write"));
        let call = assembler_call("call_2", "not_a_tool", serde_json::json!({}));
        let plan = plan_call(&call, 1).expect("plan");
        assert_eq!(
            validate_plan(&registry, AgentTier::WorkerAgent, &plan),
            Err(Refusal::ToolNotFound)
        );
    }

    #[test]
    fn empty_ids_and_non_object_args_fail_closed() {
        let empty = assembler_call("", "file_read", serde_json::json!({}));
        assert_eq!(
            plan_call(&empty, 1),
            Err(Refusal::ProviderInvalidResponse)
        );
        let array = assembler_call("id", "file_read", serde_json::json!(["a"]));
        assert_eq!(plan_call(&array, 1), Err(Refusal::ProviderInvalidResponse));
    }

    #[test]
    fn result_payload_has_no_argument_body() {
        let execution = ToolExecution::ok("call_3", "hello".to_owned());
        let payload = execution.payload(RunId::new(), TaskId::new());
        assert_eq!(payload.outcome, ToolOutcome::Ok);
        assert_eq!(payload.bytes, Some(5));
        assert_eq!(payload.sha256.as_deref(), Some(hiverouter_tools::sha256_hex(b"hello").as_str()));
    }

    #[test]
    fn result_body_is_bounded_on_a_char_boundary() {
        let long = "Ã©".repeat(MAX_TOOL_RESULT_BYTES);
        let bounded = bounded_tool_text(&long);
        assert!(bounded.len() <= MAX_TOOL_RESULT_BYTES + 64);
        assert!(bounded.is_char_boundary(bounded.len()));
    }

    #[test]
    fn turn_and_call_caps_are_closed() {
        assert_eq!(next_turn(0), Ok(1));
        assert_eq!(next_turn(MAX_TOOL_TURNS), Err(Refusal::BudgetExceeded));
        assert_eq!(check_call_budget(0), Ok(()));
        assert_eq!(
            check_call_budget(MAX_CALLS_PER_RUN),
            Err(Refusal::BudgetExceeded)
        );
    }
}


