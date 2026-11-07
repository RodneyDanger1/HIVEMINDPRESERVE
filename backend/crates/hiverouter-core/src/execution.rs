//! Run lifecycle engine: prompt -> run row -> plan -> dispatch -> streamed reply
//! (blueprint Phase 4, WP 4.5/4.6/4.7/4.8).
//!
//! Guarantees:
//! - hard caps (depth 3, fan-out 8, 64 agents, 40 calls) are enforced before a
//!   spawn/reply and refuse by name;
//! - one `CancellationToken` per run, children select on cancellation;
//! - no fake success: an executor error fails the run with its named code and
//!   preserves the completed partial state;
//! - every mutating command is idempotent by `correlation_id` (24 h window).

use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use hiverouter_contract::event::{
    AgentMessagePayload, NodeEvent, ReplyDelta, ReplyDone, RunCreated, RunMetrics, RunStatusPayload,
};
use hiverouter_contract::{AgentState, ApprovalId, Event, Refusal, RunId, TaskId, Timestamp, Usage};
use hiverouter_provider::tool_assembler::AssembledToolCall;
use hiverouter_tools::approval::{ApprovalDecision, ApprovalResolution};
use hiverouter_tools::shell::ShellPolicy;
use hiverouter_tools::{
    execute_read_only, AgentTier, AskParentArgs, ExecutedTool as ToolsExecutedTool,
    PermissionClass, SendToSiblingArgs, SpawnAgentArgs, ToolId, ToolRegistry, WorkspaceJail,
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::approval_wait::{ApprovalWake, ApprovalWaiters};
use crate::bus::EventBus;
use crate::caps::Caps;
use crate::context::{ContextStore, ScopedContext};
use crate::db::{Database, ToolTurnRecord};
use crate::entities::{ChatRole, Run, TaskRecord};
use crate::graph::GraphAuthority;
use crate::planner::Plan;
use crate::prompts::{Persona, TemplateRegistry};
use crate::store::RunStore;
use crate::tool_loop::{self, ToolCallPlan, ToolExecution};
use crate::tools_bridge::ToolsBridge;
use crate::run_state::{self, StateEvent};

/// Default approval clock: Unix seconds (the approval/TTL unit).
fn default_now_unix_secs() -> u64 {
    Timestamp::now().unix_secs().unwrap_or(0)
}

/// Adds the task's model-visible scoped facts to a prompt, bounded by the
/// context renderer. The prompt is the only path that makes rewiring visible to
/// the model; storing facts without this step is a no-op.
fn prompt_with_context(prompt: &str, context: Option<&ScopedContext>) -> String {
    match context.and_then(|context| context.prompt_block(4_000)) {
        Some(block) => format!("{prompt}\n\n{block}"),
        None => prompt.to_owned(),
    }
}

/// Memory is model-visible later, so an obvious secret marker is never stored.
fn memory_text_is_safe(text: &str) -> bool {
    !["DEEPSEEK_API_KEY", "sk-", "Bearer ", "Authorization:"]
        .iter()
        .any(|marker| text.contains(marker))
}

/// Maximum visible characters in one inter-agent message.
const MAX_AGENT_MESSAGE_CHARS: usize = 2_000;
/// Per-run agent-message budget (prevents peer loops).
const MAX_AGENT_MESSAGES_PER_RUN: u64 = 32;

/// One time source for approval timestamps/CAS (tests inject a fixed clock).
pub type Clock = Arc<dyn Fn() -> u64 + Send + Sync>;

/// Disarms a waiter on every exit path (wake, timeout, cancel, error, panic).
struct ArmedApproval {
    waiters: Arc<ApprovalWaiters>,
    approval_id: ApprovalId,
}

impl Drop for ArmedApproval {
    fn drop(&mut self) {
        self.waiters.disarm(self.approval_id);
    }
}

/// True when a task state has no outgoing lifecycle edge (M-002 terminal guard).
///
/// A `KILL_WORKER` result (`Killed`) and every other terminal task state are
/// never overwritten by a late completion/failure write: the engine refuses any
/// transition out of these states instead of resurrecting the row.
#[must_use]
fn is_terminal_task_state(state: AgentState) -> bool {
    run_state::is_task_terminal(state)
}

/// Contract node-kind label for an agent tier (`NODE_KIND_VOCABULARY` values).
#[must_use]
pub(crate) fn node_kind(tier: AgentTier) -> &'static str {
    match tier {
        AgentTier::HyperAgent => "hyperagent",
        AgentTier::SuperAgent => "superagent",
        AgentTier::WorkerAgent => "worker",
        AgentTier::ToolAgent => "tool",
    }
}

/// Enforces the hierarchical spawn policy before a child row is created.
///
/// Hyper may spawn Super/Agent/Subagent; Super may spawn Agent/Subagent; Agent
/// (Worker) may spawn **Subagent only** — never a Super, so a child cannot
/// escalate its own capability tier. Subagents and tool actors cannot spawn.
fn validate_spawn_role(parent: AgentTier, requested_role: &str) -> Result<(), Refusal> {
    let role = requested_role.trim().to_ascii_lowercase();
    let allowed = match parent {
        AgentTier::HyperAgent => matches!(
            role.as_str(),
            "super"
                | "superagent"
                | "super_agent"
                | "sub_hive"
                | "subhive"
                | "worker"
                | "workeragent"
                | "worker_agent"
                | "agent"
                | "subagent"
                | "sub_agent"
        ),
        AgentTier::SuperAgent => matches!(
            role.as_str(),
            "worker" | "workeragent" | "worker_agent" | "agent" | "subagent" | "sub_agent"
        ),
        AgentTier::WorkerAgent => matches!(role.as_str(), "subagent" | "sub_agent"),
        AgentTier::ToolAgent => false,
    };
    if allowed {
        Ok(())
    } else {
        Err(Refusal::ToolNotAllowed)
    }
}


///
/// Known roles are mapped explicitly. An unrecognized role keeps the historic
/// `WorkerAgent` capability set and is never promoted to a broader allowlist;
/// the planner root is the run orchestrator, so `planner`/`hyper` map to
/// `HyperAgent` and `super` (sub-hive) maps to `SuperAgent`.
#[must_use]
pub(crate) fn task_agent_tier(task: &TaskRecord) -> AgentTier {
    let role = task.role.trim().to_ascii_lowercase();
    match role.as_str() {
        "planner" | "hyper" | "hyperagent" | "hyper_agent" | "orchestrator" => {
            AgentTier::HyperAgent
        }
        "super" | "superagent" | "super_agent" | "sub_hive" | "subhive" => AgentTier::SuperAgent,
        "tool" | "toolagent" | "tool_agent" => AgentTier::ToolAgent,
        "worker" | "workeragent" | "worker_agent" => AgentTier::WorkerAgent,
        "" if task.depth == 0 => AgentTier::HyperAgent,
        _ => AgentTier::WorkerAgent,
    }
}

/// Template used for the one planning call; the raw prompt is used when the
/// registry has no such template.
pub const PLANNER_TEMPLATE_ID: &str = "planner";

/// Receives streamed reply fragments from an executor in provider order.
///
/// The sink is invoked synchronously from the provider read loop while the SSE
/// chunks arrive, before the executor returns. `RunEngine` uses it to publish
/// `REPLY_DELTA` events in real time. Executors that cannot stream may ignore
/// the sink and return the full text in [`StreamedReply::deltas`] instead; the
/// engine publishes those only when the sink was never invoked, so a reply is
/// never emitted twice.
pub type DeltaSink = Arc<dyn Fn(&str) + Send + Sync>;

/// Provider seam: the Phase 5 client implements this; tests use a mock.
#[async_trait]
pub trait ProviderExecutor: Send + Sync {
    /// Produces the plan DAG for one prompt (exactly one call per plan).
    async fn plan(
        &self,
        run_id: RunId,
        prompt: &str,
        cancel: &CancellationToken,
    ) -> Result<Plan, Refusal>;

    /// Executes one leaf task and returns the reply text + truthful usage.
    ///
    /// Streaming executors call `on_delta` for every text fragment as it
    /// arrives and also return the concatenated text in the reply for
    /// persistence. Non-streaming executors may leave the sink unused; the
    /// engine then publishes the returned deltas after completion.
    async fn execute(
        &self,
        run_id: RunId,
        task: &TaskRecord,
        prompt: &str,
        cancel: &CancellationToken,
        on_delta: &DeltaSink,
    ) -> Result<StreamedReply, Refusal>;

    /// Continues the conversation after read-only tool results (1.2 follow-up
    /// tool turn).
    ///
    /// `transcript` is the cumulative per-turn history: each entry holds that
    /// turn's visible assistant text and its planned calls + bounded results.
    /// The implementor replays every entry as one assistant tool-call message
    /// followed by its `role=tool` messages and offers the WorkerAgent tool
    /// specs again. Streaming executors invoke `on_delta` exactly as in
    /// [`Self::execute`].
    ///
    /// The default refuses with `E_PROVIDER_INVALID_RESPONSE`: test doubles that
    /// never emit tool calls keep the pre-1.2 behaviour and stay unchanged.
    async fn continue_with_tools(
        &self,
        _run_id: RunId,
        _task: &TaskRecord,
        _prompt: &str,
        _cancel: &CancellationToken,
        _on_delta: &DeltaSink,
        _transcript: &[ToolTurnTranscript],
    ) -> Result<StreamedReply, Refusal> {
        Err(Refusal::ProviderInvalidResponse)
    }

    /// Releases any per-run provider state after the run reaches a terminal
    /// status. The default is a no-op for stateless/test executors.
    async fn finish_run(&self, _run_id: RunId) {}
}

/// One streamed assistant reply.
///
/// `deltas` holds the reply fragments. A streaming executor invokes the
/// [`DeltaSink`] with the same fragments and returns their concatenation here;
/// this field is the persistence source (`text()`), not a second event source.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StreamedReply {
    /// Reply fragments used to persist the assistant message (`text()`).
    pub deltas: Vec<String>,
    /// Provider usage for the reply; absent counters stay absent.
    pub usage: Usage,
    /// Assembled tool calls from this assistant turn (empty for plain replies).
    pub tool_calls: Vec<hiverouter_provider::tool_assembler::AssembledToolCall>,
    /// Provider reasoning content for the turn (empty when thinking is off).
    pub reasoning: String,
}

impl StreamedReply {
    /// Full reply text.
    #[must_use]
    pub fn text(&self) -> String {
        self.deltas.concat()
    }
}

/// One assistant tool turn replayed to the provider on continuation (1.2).
///
/// The engine accumulates one entry per follow-up turn so a multi-turn tool
/// conversation keeps every prior assistant tool call and its bounded result
/// instead of sending only the latest turn.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolTurnTranscript {
    /// Visible assistant text of the turn that requested the tools.
    pub assistant_text: String,
    /// Provider reasoning content to pass back for the next tool turn.
    pub reasoning: String,
    /// Planned calls and their bounded results/refusals, provider order.
    pub results: Vec<(ToolCallPlan, ToolExecution)>,
}

/// One executed tool turn of a task (engine-internal follow-up loop data).
struct ToolTurnResult {
    /// True when the persisted tool-turn cap refused the calls (nothing ran).
    cap_reached: bool,
    /// True when the per-run provider-call budget refused at least one call.
    /// Distinguishes an honest budget stop from malformed provider calls.
    budget_refused: bool,
    /// Planned results in provider order, refusals included; fed back to the
    /// model. Unplannable calls (malformed id/args) are not representable in a
    /// provider assistant tool-call message and are therefore absent here.
    executed: Vec<(ToolCallPlan, ToolExecution)>,
}

/// Production default when no provider is configured: refuses by name with
/// `E_KEY_MISSING` â€” never fabricates a reply.
#[derive(Debug, Default)]
pub struct UnconfiguredExecutor;

#[async_trait]
impl ProviderExecutor for UnconfiguredExecutor {
    async fn plan(
        &self,
        _run_id: RunId,
        _prompt: &str,
        _cancel: &CancellationToken,
    ) -> Result<Plan, Refusal> {
        Err(Refusal::KeyMissing)
    }

    async fn execute(
        &self,
        _run_id: RunId,
        _task: &TaskRecord,
        _prompt: &str,
        _cancel: &CancellationToken,
        _on_delta: &DeltaSink,
    ) -> Result<StreamedReply, Refusal> {
        Err(Refusal::KeyMissing)
    }
}

/// One cancellation token per run.
#[derive(Default)]
pub struct RunRegistry {
    tokens: Mutex<HashMap<RunId, CancellationToken>>,
}

impl std::fmt::Debug for RunRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunRegistry")
            .field("runs", &self.len())
            .finish()
    }
}

impl RunRegistry {
    /// Creates the token for a new run (and returns its child token).
    pub fn create(&self, run_id: RunId) -> CancellationToken {
        let token = CancellationToken::new();
        if let Ok(mut tokens) = self.tokens.lock() {
            tokens.insert(run_id, token.clone());
        }
        token
    }

    /// Looks the token up.
    #[must_use]
    pub fn token(&self, run_id: RunId) -> Option<CancellationToken> {
        self.tokens
            .lock()
            .ok()
            .and_then(|tokens| tokens.get(&run_id).cloned())
    }

    /// Cancels the run's token; returns false when the run has no active token.
    pub fn cancel(&self, run_id: RunId) -> bool {
        match self.token(run_id) {
            Some(token) => {
                token.cancel();
                true
            }
            None => false,
        }
    }

    /// Whether the run is cancelled.
    #[must_use]
    pub fn is_cancelled(&self, run_id: RunId) -> bool {
        self.token(run_id).map_or(false, |token| token.is_cancelled())
    }

    /// Drops the token for a finished run.
    pub fn remove(&self, run_id: RunId) {
        if let Ok(mut tokens) = self.tokens.lock() {
            tokens.remove(&run_id);
        }
    }

    /// Number of tracked runs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tokens.lock().map(|map| map.len()).unwrap_or(0)
    }

    /// Whether the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// The run engine (one per server process).
pub struct RunEngine {
    /// Typed store over the shared database.
    pub store: RunStore,
    /// Database handle (kept for later phases).
    pub db: Arc<Database>,
    /// Server event bus.
    pub bus: Arc<EventBus>,
    /// Authoritative graph owner.
    pub graph: Arc<GraphAuthority>,
    /// Scoped context owner.
    pub contexts: Arc<ContextStore>,
    /// Per-run cancellation tokens.
    pub tokens: Arc<RunRegistry>,
    /// Provider seam.
    pub executor: Arc<dyn ProviderExecutor>,
    /// Prompt template registry (planning prompt).
    pub templates: Arc<TemplateRegistry>,
    /// Boot-loaded personas keyed by role/template id.
    personas: std::sync::RwLock<HashMap<String, Persona>>,
    /// Hard caps.
    pub caps: Caps,
    /// Durable snapshot store; when present a snapshot is written after every
    /// terminal run state.
    pub snapshots: Option<Arc<crate::snapshot::SnapshotStore>>,
    /// Workspace jail used by read-only model tool execution. `None` refuses
    /// tool execution by name (`E_FILE_TREE_UNAVAILABLE`); the engine never
    /// reads the filesystem without it.
    tool_workspace: Option<WorkspaceJail>,
    /// Frozen tool registry used to validate model-originated calls
    /// (tier allowlists + argument schemas).
    tool_registry: ToolRegistry,
    /// Approval/consent bridge. `None` keeps the pre-1.2 read-only behaviour:
    /// side-effect plans refuse `E_TOOL_NOT_ALLOWED`.
    tools: Option<Arc<ToolsBridge>>,
    /// In-process approval waiters (one server process; no cross-process wait).
    waiters: Arc<ApprovalWaiters>,
    /// One clock for approval timestamps/CAS (tests inject a fixed clock).
    clock: Clock,
    /// Shell allowlist used only for an approved `shell_exec`.
    shell_policy: ShellPolicy,
    /// One cancellation token per executing task. The run token keeps the run
    /// alive; a task token lets `KILL_WORKER` stop exactly one task (M-012).
    task_tokens: Mutex<HashMap<TaskId, CancellationToken>>,
    /// Serializes the engine-side task-state read/check/write so a concurrent
    /// kill and completion cannot resurrect a terminal state (M-002).
    task_state_lock: Mutex<()>,
    /// Tasks created after planning (command or model spawn) awaiting dispatch.
    spawn_queue: Mutex<Vec<TaskId>>,
    active_run: Mutex<Option<RunId>>,
    /// Last published green percentage per run (delta provenance).
    metrics_last: Mutex<HashMap<RunId, f64>>,
    /// Accumulated provider usage per run, persisted at terminal.
    usage_by_run: Mutex<HashMap<RunId, Usage>>,
}

impl std::fmt::Debug for RunEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunEngine")
            .field("active_run", &self.active_run())
            .field("caps", &self.caps)
            .field("tool_workspace", &self.tool_workspace.is_some())
            .field("approval_wired", &self.tools.is_some())
            .field("waiters", &self.waiters.len())
            .finish()
    }
}

impl RunEngine {
    /// Builds an engine over the shared services.
    #[must_use]
    pub fn new(
        db: Arc<Database>,
        bus: Arc<EventBus>,
        graph: Arc<GraphAuthority>,
        executor: Arc<dyn ProviderExecutor>,
        templates: Arc<TemplateRegistry>,
    ) -> Self {
        Self {
            store: RunStore::new(db.clone()),
            db,
            bus,
            graph,
            contexts: Arc::new(ContextStore::new()),
            tokens: Arc::new(RunRegistry::default()),
            executor,
            templates,
            personas: std::sync::RwLock::new(HashMap::new()),
            caps: Caps::default(),
            snapshots: None,
            tool_workspace: None,
            tool_registry: ToolRegistry::standard(),
            tools: None,
            waiters: Arc::new(ApprovalWaiters::new()),
            clock: Arc::new(default_now_unix_secs),
            shell_policy: ShellPolicy::conservative_default(),
            task_tokens: Mutex::new(HashMap::new()),
            task_state_lock: Mutex::new(()),
            spawn_queue: Mutex::new(Vec::new()),
            active_run: Mutex::new(None),
            metrics_last: Mutex::new(HashMap::new()),
            usage_by_run: Mutex::new(HashMap::new()),
        }
    }

    /// Attaches the durable snapshot store (boot path).
    #[must_use]
    pub fn with_snapshot_store(
        mut self,
        snapshots: Arc<crate::snapshot::SnapshotStore>,
    ) -> Self {
        self.snapshots = Some(snapshots);
        self
    }

    /// Attaches the workspace jail used by read-only model tool execution.
    ///
    /// This is optional by design: without a jail, model tool calls are refused
    /// with `E_FILE_TREE_UNAVAILABLE` and nothing executes; plain replies are
    /// completely unchanged.
    #[must_use]
    pub fn with_tool_workspace(mut self, workspace: WorkspaceJail) -> Self {
        self.tool_workspace = Some(workspace);
        self
    }

    /// Attaches the approval bridge; production `http.rs` always does.
    #[must_use]
    pub fn with_tools(mut self, tools: Arc<ToolsBridge>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Attaches the boot-loaded persona registry.
    #[must_use]
    pub fn with_personas(self, personas: Vec<Persona>) -> Self {
        let installed: HashMap<String, Persona> = personas
            .into_iter()
            .map(|persona| (persona.id.clone(), persona))
            .collect();
        *self
            .personas
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = installed;
        self
    }

    /// Inserts or replaces one profile in the live registry.
    pub fn upsert_persona(&self, persona: Persona) {
        self.personas
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(persona.id.clone(), persona);
    }

    /// Removes one profile from the live registry; returns whether it existed.
    pub fn remove_persona(&self, id: &str) -> bool {
        self.personas
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(id)
            .is_some()
    }

    /// Returns the boot-loaded personas in stable id order.
    #[must_use]
    pub fn personas(&self) -> Vec<Persona> {
        let mut personas: Vec<Persona> = self
            .personas
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .values()
            .cloned()
            .collect();
        personas.sort_by(|left, right| left.id.cmp(&right.id));
        personas
    }

    /// Injects the approval clock (tests; production uses the system clock).
    #[must_use]
    pub fn with_clock(mut self, clock: Clock) -> Self {
        self.clock = clock;
        self
    }

    /// Overrides the shell allowlist for an approved `shell_exec`.
    #[must_use]
    pub fn with_shell_policy(mut self, shell_policy: ShellPolicy) -> Self {
        self.shell_policy = shell_policy;
        self
    }

    /// Wakes a suspended engine wait after the WS command committed a decision.
    /// Returns false when no waiter is armed (legal: late/duplicate/terminal
    /// resolve); the caller logs and never fails the command.
    #[must_use]
    pub fn notify_approval_resolved(&self, resolution: &ApprovalResolution) -> bool {
        self.waiters.wake_resolved(resolution)
    }

    /// Builds an engine with explicit contexts/registry (tests).
    #[must_use]
    pub fn with_services(
        db: Arc<Database>,
        bus: Arc<EventBus>,
        graph: Arc<GraphAuthority>,
        contexts: Arc<ContextStore>,
        tokens: Arc<RunRegistry>,
        executor: Arc<dyn ProviderExecutor>,
        templates: Arc<TemplateRegistry>,
        caps: Caps,
    ) -> Self {
        Self {
            store: RunStore::new(db.clone()),
            db,
            bus,
            graph,
            contexts,
            tokens,
            executor,
            templates,
            personas: std::sync::RwLock::new(HashMap::new()),
            caps,
            snapshots: None,
            tool_workspace: None,
            tool_registry: ToolRegistry::standard(),
            tools: None,
            waiters: Arc::new(ApprovalWaiters::new()),
            clock: Arc::new(default_now_unix_secs),
            shell_policy: ShellPolicy::conservative_default(),
            task_tokens: Mutex::new(HashMap::new()),
            task_state_lock: Mutex::new(()),
            spawn_queue: Mutex::new(Vec::new()),
            active_run: Mutex::new(None),
            metrics_last: Mutex::new(HashMap::new()),
            usage_by_run: Mutex::new(HashMap::new()),
        }
    }

    /// Currently active run (single-run policy).
    #[must_use]
    pub fn active_run(&self) -> Option<RunId> {
        self.active_run.lock().ok().and_then(|active| *active)
    }

    fn set_active(&self, run_id: Option<RunId>) -> Result<(), Refusal> {
        let mut active = self.active_run.lock().map_err(|_| Refusal::InternalError)?;
        *active = run_id;
        Ok(())
    }

    fn publish(&self, event: Event, run_id: Option<RunId>, correlation: Option<Uuid>) {
        if let Err(error) = self.bus.publish_for_run(event, run_id, correlation) {
            tracing::error!(code = error.code(), "failed to publish engine event");
        }
    }

    /// Publishes the authoritative run metric frame from persisted task rows.
    ///
    /// The delta is the difference from the last frame for this run; unknown
    /// counts are never fabricated (a task whose kind cannot be read is skipped).
    fn publish_run_metrics(&self, run_id: RunId) {
        let Ok(tasks) = self.store.tasks_for_run(run_id) else {
            return;
        };
        let superagents = tasks
            .iter()
            .filter(|task| node_kind(task_agent_tier(task)) == "superagent")
            .count();
        let workers = tasks
            .iter()
            .filter(|task| node_kind(task_agent_tier(task)) == "worker")
            .count();
        let completed = tasks
            .iter()
            .filter(|task| task.state == AgentState::Completed)
            .count();
        let green_pct = if tasks.is_empty() {
            0.0
        } else {
            (completed as f64 / tasks.len() as f64) * 100.0
        };
        let delta = {
            let mut last = match self.metrics_last.lock() {
                Ok(last) => last,
                Err(_) => return,
            };
            let previous = last.insert(run_id, green_pct).unwrap_or(green_pct);
            green_pct - previous
        };
        self.publish(
            Event::RunMetrics(RunMetrics {
                superagents: u32::try_from(superagents).unwrap_or(u32::MAX),
                workers: u32::try_from(workers).unwrap_or(u32::MAX),
                green_pct,
                delta,
            }),
            Some(run_id),
            None,
        );
    }

    fn add_run_usage(&self, run_id: RunId, usage: Usage) {
        if let Ok(mut totals) = self.usage_by_run.lock() {
            let entry = totals.entry(run_id).or_default();
            *entry = entry.checked_add(&usage);
        }
    }

    fn persist_run_usage(&self, run_id: RunId) {
        let usage = self
            .usage_by_run
            .lock()
            .ok()
            .and_then(|mut totals| totals.remove(&run_id));
        if let Some(usage) = usage {
            if let Err(error) = self.store.record_run_usage(run_id, &usage) {
                tracing::error!(
                    code = error.code(),
                    "failed to persist run usage totals"
                );
            }
        }
    }

    fn publish_agent_message(
        &self,
        run_id: RunId,
        from_task_id: TaskId,
        to_task_id: TaskId,
        kind: &str,
        text: &str,
    ) {
        self.publish(
            Event::AgentMessage(AgentMessagePayload {
                run_id,
                from_task_id,
                to_task_id,
                kind: kind.to_owned(),
                text: text.to_owned(),
                created_at: Timestamp::now(),
            }),
            Some(run_id),
            None,
        );
    }

    fn publish_node(&self, run_id: RunId, task_id: TaskId, state: AgentState) {
        let node = self.graph.node(task_id).ok().flatten();
        let kind = self
            .store
            .get_task(task_id)
            .ok()
            .map(|task| node_kind(task_agent_tier(&task)).to_owned());
        self.publish(
            Event::NodeState(NodeEvent {
                task_id,
                state,
                rev: node.as_ref().map(|node| node.rev),
                kind,
                parent_task_id: node.as_ref().and_then(|node| node.parent_task_id),
            }),
            Some(run_id),
            None,
        );
        self.publish_run_metrics(run_id);
    }

    /// Publishes the `NODE_SPAWN` frame with the node's authoritative topology.
    fn publish_node_spawn(&self, run_id: RunId, task: &TaskRecord) {
        let node = self.graph.node(task.id).ok().flatten();
        self.publish(
            Event::NodeSpawn(NodeEvent {
                task_id: task.id,
                state: AgentState::Queued,
                rev: node.as_ref().map(|node| node.rev),
                kind: Some(node_kind(task_agent_tier(task)).to_owned()),
                parent_task_id: task.parent_task_id,
            }),
            Some(run_id),
            None,
        );
        self.publish_run_metrics(run_id);
    }

    /// Registers a per-task child of `run_token` and returns it.
    fn register_task_token(
        &self,
        task_id: TaskId,
        run_token: &CancellationToken,
    ) -> Result<CancellationToken, Refusal> {
        let token = run_token.child_token();
        let mut tokens = self
            .task_tokens
            .lock()
            .map_err(|_| Refusal::InternalError)?;
        tokens.insert(task_id, token.clone());
        Ok(token)
    }

    /// Drops the per-task token once its execution turn ended.
    fn unregister_task_token(&self, task_id: TaskId) {
        if let Ok(mut tokens) = self.task_tokens.lock() {
            tokens.remove(&task_id);
        }
    }

    /// Cancels exactly one task's token; `false` when the task has no live turn.
    fn cancel_task_token(&self, task_id: TaskId) -> bool {
        let token = self
            .task_tokens
            .lock()
            .ok()
            .and_then(|tokens| tokens.get(&task_id).cloned());
        match token {
            Some(token) => {
                token.cancel();
                true
            }
            None => false,
        }
    }

    /// Enqueues a task created after planning for the run's dispatch loop.
    fn enqueue_spawned_task(&self, task_id: TaskId) -> Result<(), Refusal> {
        let mut queue = self
            .spawn_queue
            .lock()
            .map_err(|_| Refusal::InternalError)?;
        if !queue.contains(&task_id) {
            queue.push(task_id);
        }
        Ok(())
    }

    /// Enqueues template-inserted tasks after their rows and graph nodes exist.
    ///
    /// The run-scoped dispatcher drains this queue before the run can become
    /// terminal; without this call a template node stays `Queued` forever.
    ///
    /// # Errors
    /// `E_INTERNAL_ERROR` when the queue lock is poisoned.
    pub fn enqueue_spawned_tasks(&self, task_ids: &[TaskId]) -> Result<(), Refusal> {
        for task_id in task_ids {
            self.enqueue_spawned_task(*task_id)?;
        }
        Ok(())
    }

    /// Seeds scoped context for freshly inserted template tasks.
    ///
    /// # Errors
    /// Propagates the context-store refusal.
    pub fn seed_template_contexts(&self, tasks: &[TaskRecord]) -> Result<(), Refusal> {
        for task in tasks {
            self.seed_task_context(task, task.parent_task_id)?;
        }
        Ok(())
    }

    /// Seeds a task's scoped context from its authoritative parent at creation.
    ///
    /// Facts are inherited as a snapshot; the parent link is explicit so a later
    /// rewire can clear only that parent's facts. Empty facts are not invented.
    fn seed_task_context(
        &self,
        task: &TaskRecord,
        parent_task_id: Option<TaskId>,
    ) -> Result<(), Refusal> {
        let mut context = ScopedContext::new(parent_task_id);
        if let Some(parent_task_id) = parent_task_id {
            if let Some(parent) = self.contexts.get(parent_task_id)? {
                context.facts = parent.facts;
            }
        }
        if let Some(persona) = self
            .personas
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&task.role)
        {
            context.allowed_tool_ids = persona.allowed_tool_ids.clone();
            context.output_schema_id = persona.output_schema_id.clone();
        }
        if let Ok(memories) = self.store.agent_memory(&task.role, 8) {
            for (key, value) in memories {
                context.facts.insert(format!("memory:{key}"), value);
            }
        }
        self.contexts.set(task.id, context)
    }

    /// Removes one specific spawned task from the deferred queue.
    fn take_queued_task(&self, task_id: TaskId) -> bool {
        self.spawn_queue.lock().ok().is_some_and(|mut queue| {
            if let Some(position) = queue.iter().position(|queued| *queued == task_id) {
                queue.remove(position);
                true
            } else {
                false
            }
        })
    }

    /// Pops the next spawned task awaiting dispatch.
    fn next_queued_task(&self) -> Option<TaskId> {
        self.spawn_queue.lock().ok().and_then(|mut queue| {
            if queue.is_empty() {
                None
            } else {
                Some(queue.remove(0))
            }
        })
    }

    /// Applies one task lifecycle event to the freshest persisted state.
    ///
    /// A terminal task is never transitioned again (`Ok(None)`). The store and
    /// graph writes happen here under the engine's task-state lock; the caller
    /// owns publishing the matching `NODE_STATE`.
    fn apply_task_state(
        &self,
        task_id: TaskId,
        event: StateEvent,
    ) -> Result<Option<run_state::TaskTransition>, Refusal> {
        let _guard = self
            .task_state_lock
            .lock()
            .map_err(|_| Refusal::InternalError)?;
        let fresh = self.store.get_task(task_id)?;
        if is_terminal_task_state(fresh.state) {
            return Ok(None);
        }
        let transition =
            run_state::apply_task(fresh.state, event).map_err(|error| error.refusal())?;
        self.store.update_task_state(task_id, transition.to)?;
        let _ = self.graph.set_state(task_id, transition.to);
        Ok(Some(transition))
    }

    /// Runs one lifecycle read/check/write while holding the engine lifecycle
    /// lock, so a per-task kill and a terminal run write cannot interleave and
    /// resurrect a terminal run. The closure must not call another guarded
    /// lifecycle helper (the mutex is not reentrant).
    fn with_lifecycle_lock<T>(
        &self,
        apply: impl FnOnce() -> Result<T, Refusal>,
    ) -> Result<T, Refusal> {
        let _guard = self
            .task_state_lock
            .lock()
            .map_err(|_| Refusal::InternalError)?;
        apply()
    }

    /// Returns a run suspended on a per-task approval wait to `running` after a
    /// per-task kill, so sibling tasks keep executing (M-012).
    ///
    /// Serialized with the terminal run writers: a run that already reached a
    /// terminal status is never resurrected to `running`.
    fn resume_run_after_task_kill(&self, run_id: RunId) -> Result<(), Refusal> {
        self.with_lifecycle_lock(|| {
            let run = self.store.get_run(run_id)?;
            if run.status.is_terminal() || run.status != AgentState::AwaitingApproval {
                return Ok(());
            }
            let transition = run_state::apply_run(run.status, StateEvent::Resume)
                .map_err(|error| error.refusal())?;
            self.store.set_run_status(run_id, transition.to, None)?;
            self.publish(
                Event::RunStatus(RunStatusPayload {
                    run_id,
                    status: transition.to,
                    error_code: None,
                }),
                Some(run_id),
                None,
            );
            Ok(())
        })
    }

    /// Authorizes one model tool call against the task's real tier and the
    /// task's scoped persona allowlist (M-015).
    ///
    /// The tier comes from the task role ([`task_agent_tier`]) instead of the
    /// historic hardcoded `WorkerAgent`. When the task has a scoped context
    /// whose persona allowlist is non-empty, the tool id must be on it too.
    fn authorize_tool_call(
        &self,
        task: &TaskRecord,
        plan: &ToolCallPlan,
    ) -> Result<AgentTier, Refusal> {
        let tier = task_agent_tier(task);
        tool_loop::validate_plan(&self.tool_registry, tier, plan)?;
        if let Some(allowed) = self.scoped_persona_tools(task.id)? {
            if !allowed.is_empty() && !allowed.iter().any(|tool| tool == &plan.tool) {
                return Err(Refusal::ToolNotAllowed);
            }
        }
        Ok(tier)
    }

    /// The task's persona tool ids, when a scoped context carries them.
    ///
    /// The context owner copies the persona's `allowed_tool_ids` into the task's
    /// scoped context on creation/rewire; an empty list means the context has
    /// not narrowed the tier allowlist.
    fn scoped_persona_tools(&self, task_id: TaskId) -> Result<Option<Vec<String>>, Refusal> {
        Ok(self
            .contexts
            .get(task_id)?
            .map(|context| context.allowed_tool_ids))
    }

    /// Handles `SUBMIT_PROMPT`: validates, creates the run row, audit receipt,
    /// user chat message, `RUN_CREATED`, and starts the background dispatch.
    ///
    /// # Errors
    /// `E_PROMPT_EMPTY`, `E_PROMPT_TOO_LONG`, `E_RUN_ACTIVE`, or a store refusal.
    pub fn submit_prompt(
        self: &Arc<Self>,
        prompt: &str,
        project_id: &str,
        correlation: Uuid,
    ) -> Result<Run, Refusal> {
        let chars = prompt.chars().count();
        if chars == 0 {
            return Err(Refusal::PromptEmpty);
        }
        if chars > hiverouter_contract::command::MAX_PROMPT_CHARS {
            return Err(Refusal::PromptTooLong);
        }
        if let Some(existing) = self.store.receipt_run(correlation)? {
            return self.store.get_run(existing);
        }
        if self.store.latest_active_run()?.is_some() || self.active_run().is_some() {
            return Err(Refusal::RunActive);
        }
        let run = self.store.create_run(project_id, prompt, AgentState::Queued)?;
        self.store.record_receipt(correlation, run.id)?;
        self.store.append_message(run.id, ChatRole::User, prompt)?;
        self.set_active(Some(run.id))?;
        let token = self.tokens.create(run.id);
        self.publish(
            Event::RunCreated(RunCreated {
                run_id: run.id,
                status: AgentState::Queued,
            }),
            Some(run.id),
            Some(correlation),
        );
        let engine = Arc::clone(self);
        let run_for_task = run.clone();
        tokio::spawn(async move {
            engine.dispatch(run_for_task, token).await;
        });
        Ok(run)
    }

    /// Handles `SPAWN_SUB_HIVE` with the depth/fan-out/agent caps.
    ///
    /// # Errors
    /// `E_TASK_NOT_FOUND`, `E_RUN_TERMINAL`, `E_SPAWN_DEPTH`, `E_FANOUT_LIMIT`,
    /// `E_AGENT_LIMIT`.
    pub fn spawn_sub_hive(&self, parent_task_id: TaskId, role: &str) -> Result<TaskRecord, Refusal> {
        // Read + cap-check + insert under one lock so concurrent spawns cannot
        // both pass the fan-out/agent caps and exceed them (the caps are hard).
        let task = {
            let _guard = self
                .task_state_lock
                .lock()
                .map_err(|_| Refusal::InternalError)?;
            let parent = self.store.get_task(parent_task_id)?;
            let run = self.store.get_run(parent.run_id)?;
            if run.status.is_terminal() {
                return Err(Refusal::RunTerminal);
            }
            validate_spawn_role(task_agent_tier(&parent), role)?;
            let siblings = self
                .store
                .tasks_for_run(parent.run_id)?
                .into_iter()
                .filter(|task| task.parent_task_id == Some(parent_task_id))
                .count();
            let agents = self.store.task_count(parent.run_id)?;
            let depth = parent.depth.saturating_add(1);
            self.caps.check_spawn(depth, siblings, agents)?;
            let task = TaskRecord {
                id: TaskId::new(),
                run_id: parent.run_id,
                parent_task_id: Some(parent_task_id),
                role: role.to_owned(),
                depth,
                spawn_seq: agents.saturating_add(1),
                state: AgentState::Queued,
                rev: 0,
            };
            self.store.insert_task(&task)?;
            self.graph.add_task(task.id, Some(parent_task_id))?;
            task
        };
        self.seed_task_context(&task, Some(parent_task_id))?;
        self.publish_node_spawn(task.run_id, &task);
        // The run dispatch loop drains this queue and executes the child to a
        // terminal state before the run itself can become terminal (M-009).
        self.enqueue_spawned_task(task.id)?;
        Ok(task)
    }

    /// Handles `CANCEL_RUN`.
    ///
    /// # Errors
    /// `E_RUN_NOT_FOUND`, `E_RUN_TERMINAL`.
    pub fn cancel_run(&self, run_id: RunId) -> Result<(), Refusal> {
        let run = self.store.get_run(run_id)?;
        if run.status.is_terminal() {
            return Err(Refusal::RunTerminal);
        }
        if !self.tokens.cancel(run_id) {
            return Err(Refusal::RunTerminal);
        }
        // The engine owns the single terminal frame and TOOL_RESULT ordering:
        // cancelling the token wakes a suspended wait, whose cancel path finishes
        // one cancelled TOOL_RESULT, then dispatch persists/publishes the one
        // terminal RUN_STATUS cancelled frame.
        Ok(())
    }

    /// Handles `KILL_WORKER` with per-task scope.
    ///
    /// A running task is cancelled through its own execution token and settled
    /// `Killed`; an awaiting-approval task is settled `Cancelled`, its wait token
    /// is cancelled, and the run returns to `running` so sibling tasks keep
    /// executing. Only `CANCEL_RUN` cancels the run itself (M-002/M-012).
    ///
    /// # Errors
    /// `E_TASK_NOT_FOUND`, `E_TASK_TERMINAL`, `E_KILL_DENIED`.
    pub fn kill_worker(&self, task_id: TaskId) -> Result<(), Refusal> {
        let task = self.store.get_task(task_id)?;
        if is_terminal_task_state(task.state) {
            return Err(Refusal::TaskTerminal);
        }
        if !matches!(
            task.state,
            AgentState::Running | AgentState::AwaitingApproval
        ) {
            return Err(Refusal::KillDenied);
        }
        // The transition is single-owner and terminal-guarded: a task already
        // settled by a concurrent completion/failure path is never resurrected.
        let Some(transition) = self.apply_task_state(task_id, StateEvent::Kill)? else {
            return Err(Refusal::TaskTerminal);
        };
        // Cancel exactly this task's in-flight work. The run token is untouched:
        // only CANCEL_RUN may cancel the run and its sibling tasks (M-012).
        self.cancel_task_token(task_id);
        self.publish_node(task.run_id, task_id, transition.to);
        if transition.from == AgentState::AwaitingApproval {
            // The suspended waiter wakes on the task token and consumes the
            // approval; returning the run to running keeps siblings alive.
            self.resume_run_after_task_kill(task.run_id)?;
        }
        Ok(())
    }

    /// Next monotonically increasing snapshot revision for a run.
    ///
    /// A revision is consumed only when its index row is committed, so a failed
    /// write cannot create a gap or a duplicate revision.
    fn snapshot_revision(&self, run_id: RunId) -> u64 {
        self.snapshots
            .as_ref()
            .and_then(|store| store.indexed_revisions(run_id).ok())
            .map(|rows| u64::try_from(rows.len()).unwrap_or(0))
            .unwrap_or(0)
    }

    /// Captures and durably writes the committed graph for `run` (M-004).
    ///
    /// Called at the committed transitions (planning -> running, after each
    /// executed node, awaiting approval, terminal), so a crash mid-run restores
    /// the last committed graph instead of having no revision at all. A write
    /// failure is reported and leaves the previous revision intact; it never
    /// fabricates a snapshot.
    fn commit_run_snapshot(&self, run: &Run, revision: u64, status: AgentState) {
        let Some(store) = &self.snapshots else {
            return;
        };
        match crate::snapshot::capture(&self.graph, run.id, revision, status) {
            Ok(envelope) => {
                if let Err(error) = store.write(&envelope) {
                    tracing::error!(
                        code = error.code(),
                        "snapshot write failed; run state is persisted without a new revision"
                    );
                }
            }
            Err(error) => {
                tracing::error!(code = error.code(), "snapshot capture failed");
            }
        }
    }

    /// Settles every non-terminal task of a terminal run to the terminal state
    /// matching the run outcome (M-003).
    ///
    /// The planner root and every non-leaf/queued task are structural rows that
    /// never enter `execute_task`; without this reconciliation a `completed` run
    /// keeps the root `running` in SQLite and `queued` in the graph. Already
    /// terminal rows are never overwritten; the graph is aligned to the
    /// persisted state so DB and graph agree in the terminal snapshot.
    fn reconcile_terminal_tasks(&self, run: &Run, outcome: AgentState) -> Result<(), Refusal> {
        let terminal = match outcome {
            AgentState::Completed => AgentState::Completed,
            AgentState::Cancelled => AgentState::Cancelled,
            AgentState::Interrupted => AgentState::Interrupted,
            _ => AgentState::Failed,
        };
        let mut tasks = self.store.tasks_for_run(run.id)?;
        tasks.sort_by_key(|task| task.spawn_seq);
        for task in tasks {
            let _guard = self
                .task_state_lock
                .lock()
                .map_err(|_| Refusal::InternalError)?;
            let fresh = self.store.get_task(task.id)?;
            if is_terminal_task_state(fresh.state) {
                let graph_state = self.graph.state_of(task.id).ok();
                if graph_state.is_some_and(|state| state != fresh.state) {
                    let _ = self.graph.set_state(task.id, fresh.state);
                }
                continue;
            }
            self.store.update_task_state(task.id, terminal)?;
            let _ = self.graph.set_state(task.id, terminal);
            self.publish_node(run.id, task.id, terminal);
        }
        Ok(())
    }

    async fn dispatch(self: Arc<Self>, run: Run, token: CancellationToken) {
        let outcome = self.dispatch_inner(&run, &token).await;
        let persisted = self
            .store
            .get_run(run.id)
            .map(|current| current.status)
            .unwrap_or(run.status);
        let (final_status, final_error) = if persisted.is_terminal() {
            // A wait terminal path (timeout) or CANCEL_RUN already persisted and
            // published the terminal frame; never duplicate it.
            (persisted, None)
        } else {
            let (status, error_code) = match &outcome {
                Ok(()) => (AgentState::Completed, None),
                Err(refusal) if *refusal == Refusal::ToolCancelled => (AgentState::Cancelled, None),
                Err(refusal) => (AgentState::Failed, Some(refusal.code().to_owned())),
            };
            (status, error_code)
        };
        // Every task is settled before the terminal frame and terminal snapshot:
        // a terminal run must never keep a Running/Queued task row (M-003).
        if let Err(error) = self.reconcile_terminal_tasks(&run, final_status) {
            tracing::error!(code = error.code(), "failed to reconcile terminal task states");
        }
        if !persisted.is_terminal() {
            // Serialized with the per-task kill run resume: a terminal status is
            // written and published exactly once and never resurrected.
            let published = self.with_lifecycle_lock(|| {
                let current = self.store.get_run(run.id)?;
                if current.status.is_terminal() {
                    return Ok(false);
                }
                if let Err(error) = self
                    .store
                    .set_run_status(run.id, final_status, final_error.as_deref())
                {
                    tracing::error!(code = error.code(), "failed to persist final run status");
                }
                self.publish(
                    Event::RunStatus(RunStatusPayload {
                        run_id: run.id,
                        status: final_status,
                        error_code: final_error.clone(),
                    }),
                    Some(run.id),
                    None,
                );
                Ok(true)
            });
            if let Err(error) = published {
                tracing::error!(
                    code = error.code(),
                    "failed to write the terminal run status under the lifecycle lock"
                );
            }
        }
        // Keep the post-terminal snapshot in addition to the mid-run revisions.
        self.commit_run_snapshot(&run, self.snapshot_revision(run.id), final_status);
        let _ = self.set_active(None);
        self.tokens.remove(run.id);
        self.persist_run_usage(run.id);
        self.executor.finish_run(run.id).await;
    }

    async fn dispatch_inner(&self, run: &Run, token: &CancellationToken) -> Result<(), Refusal> {
        self.store.set_run_status(run.id, AgentState::Planning, None)?;

        // Exactly one planning call per plan.
        self.caps.check_calls(1)?;
        let planning_prompt = self.planning_prompt(run);
        let plan = tokio::select! {
            _ = token.cancelled() => return Err(Refusal::ToolCancelled),
            result = self.executor.plan(run.id, &planning_prompt, token) => result?,
        };
        plan.validate()?;

        // Root task.
        let root = TaskRecord {
            id: TaskId::new(),
            run_id: run.id,
            parent_task_id: None,
            role: "planner".to_owned(),
            depth: 0,
            spawn_seq: 1,
            state: AgentState::Running,
            rev: 0,
        };
        self.store.insert_task(&root)?;
        self.graph.add_task(root.id, None)?;
        self.seed_task_context(&root, None)?;
        self.publish_node_spawn(run.id, &root);

        let mut tasks: HashMap<TaskId, TaskRecord> = HashMap::new();
        tasks.insert(root.id, root.clone());
        let mut spawn_seq: u64 = 1;
        // Parents are always at a lower depth by plan validation, so inserting
        // depth-first guarantees the real parent row and graph node exist before
        // any child is attached. Legacy parent-less plans still attach to root.
        let mut planned: Vec<(usize, &crate::planner::PlanNode)> =
            plan.nodes.iter().enumerate().collect();
        planned.sort_by_key(|(index, node)| (node.depth, *index));
        for (index, node) in planned {
            let agents = self.store.task_count(run.id)?;
            self.caps.check_spawn(node.depth, index, agents)?;
            let parent_id = node.parent.unwrap_or(root.id);
            if !tasks.contains_key(&parent_id) {
                return Err(Refusal::SchemaInvalid);
            }
            spawn_seq = spawn_seq.saturating_add(1);
            let record = TaskRecord {
                id: node.node_id,
                run_id: run.id,
                parent_task_id: Some(parent_id),
                role: node.role.clone(),
                depth: node.depth,
                spawn_seq,
                state: AgentState::Queued,
                rev: 0,
            };
            self.store.insert_task(&record)?;
            self.graph.add_task(record.id, Some(parent_id))?;
            self.seed_task_context(&record, Some(parent_id))?;
            self.publish_node_spawn(run.id, &record);
            tasks.insert(record.id, record);
        }

        // D2: run_state accepts Suspend only from Running. The planning phase is
        // over; persist one Start transition and publish the one new frame.
        let start = run_state::apply_run(AgentState::Planning, StateEvent::Start)
            .map_err(|error| error.refusal())?;
        self.store.set_run_status(run.id, start.to, None)?;
        self.publish(
            Event::RunStatus(RunStatusPayload {
                run_id: run.id,
                status: start.to,
                error_code: None,
            }),
            Some(run.id),
            None,
        );
        // M-004: commit the planned graph before the first execution turn, so a
        // crash during execution restores this run instead of nothing.
        self.commit_run_snapshot(run, self.snapshot_revision(run.id), start.to);

        // Execute leaf tasks in topological order; reply chunks stream per task.
        for leaf_id in plan.leaf_order()? {
            if token.is_cancelled() {
                return Err(Refusal::ToolCancelled);
            }
            let task = tasks.get(&leaf_id).cloned().ok_or(Refusal::InternalError)?;
            let result = self.execute_task(run, &task, token).await;
            // Bounded mid-run commit after every executed node (M-004).
            let status = self
                .store
                .get_run(run.id)
                .map(|current| current.status)
                .unwrap_or(AgentState::Running);
            self.commit_run_snapshot(run, self.snapshot_revision(run.id), status);
            match result {
                Ok(_) => {}
                Err(refusal) => {
                    if !token.is_cancelled()
                        && matches!(refusal, Refusal::ToolCancelled | Refusal::TaskTerminal)
                    {
                        let fresh = self.store.get_task(task.id)?;
                        if is_terminal_task_state(fresh.state) {
                            // KILL_WORKER (or a kill/resolve race) stopped only
                            // this task; the run and its remaining tasks continue
                            // (M-012).
                            continue;
                        }
                    }
                    return Err(refusal);
                }
            }
        }
        // Phase 2: aggregators run after their children, deepest first. The child
        // result is read from the task-scoped result table, never from the
        // run-wide chat transcript, so one branch cannot consume another
        // branch's answer.
        for aggregator_id in plan.aggregation_order()? {
            if token.is_cancelled() {
                return Err(Refusal::ToolCancelled);
            }
            let task = tasks
                .get(&aggregator_id)
                .cloned()
                .ok_or(Refusal::InternalError)?;
            let mut aggregate_prompt = run.prompt.clone();
            aggregate_prompt.push_str(
                "\n\nAggregate the completed child results below into this task's final answer. \
                 Do not invent results for children that have none.\n",
            );
            let Some(aggregator) = plan.node(aggregator_id) else {
                return Err(Refusal::InternalError);
            };
            for child in tasks.values().filter(|child| {
                child.parent_task_id == Some(aggregator_id) || aggregator.deps.contains(&child.id)
            }) {
                match self.store.task_result(child.id)? {
                    Some(text) if !text.trim().is_empty() => {
                        aggregate_prompt.push_str("\nChild ");
                        aggregate_prompt.push_str(&child.id.to_string());
                        aggregate_prompt.push_str(" (role ");
                        aggregate_prompt.push_str(&child.role);
                        aggregate_prompt.push_str("):\n");
                        aggregate_prompt.push_str(&tool_loop::bounded_tool_text(&text));
                        aggregate_prompt.push('\n');
                    }
                    Some(_) | None => {
                        aggregate_prompt.push_str("\nChild ");
                        aggregate_prompt.push_str(&child.id.to_string());
                        aggregate_prompt.push_str(" has no recorded result.\n");
                    }
                }
            }
            let result = self
                .execute_task_with_prompt(run, &task, token, &aggregate_prompt)
                .await;
            self.commit_run_snapshot(run, self.snapshot_revision(run.id), AgentState::Running);
            match result {
                Ok(_) => {}
                Err(refusal) => {
                    if !token.is_cancelled()
                        && matches!(refusal, Refusal::ToolCancelled | Refusal::TaskTerminal)
                    {
                        let fresh = self.store.get_task(task.id)?;
                        if is_terminal_task_state(fresh.state) {
                            continue;
                        }
                    }
                    return Err(refusal);
                }
            }
        }
        // Tasks created after planning (operator SPAWN_SUB_HIVE or a model
        // spawn_agent that was not already joined inline) are executed to a
        // terminal state before the run can become terminal (M-009).
        self.dispatch_spawned_tasks(run, token).await?;
        Ok(())
    }

    /// Drains tasks created after planning and executes each one to a terminal
    /// state, sequentially, inside the run dispatch (M-009).
    ///
    /// Depth/fan-out/agent caps were enforced by [`Self::spawn_sub_hive`] before
    /// the child row existed; the queue consumer re-reads the row so a task
    /// killed/removed before dispatch is never run. A per-task kill settles that
    /// one child and the loop continues with the next queued task.
    async fn dispatch_spawned_tasks(
        &self,
        run: &Run,
        token: &CancellationToken,
    ) -> Result<(), Refusal> {
        while let Some(task_id) = self.next_queued_task() {
            if token.is_cancelled() {
                return Err(Refusal::ToolCancelled);
            }
            let task = match self.store.get_task(task_id) {
                Ok(task) => task,
                Err(Refusal::TaskNotFound) => continue,
                Err(error) => return Err(error),
            };
            if task.run_id != run.id || is_terminal_task_state(task.state) {
                continue;
            }
            if task.state != AgentState::Queued && task.state != AgentState::Running {
                // AwaitingApproval/terminal rows are owned by their own path.
                continue;
            }
            let result = self.execute_task(run, &task, token).await;
            let status = self
                .store
                .get_run(run.id)
                .map(|current| current.status)
                .unwrap_or(AgentState::Running);
            self.commit_run_snapshot(run, self.snapshot_revision(run.id), status);
            match result {
                Ok(_) => {}
                Err(refusal) => {
                    if !token.is_cancelled()
                        && matches!(refusal, Refusal::ToolCancelled | Refusal::TaskTerminal)
                    {
                        let fresh = self.store.get_task(task_id)?;
                        if is_terminal_task_state(fresh.state) {
                            // Per-task kill: this child is settled; siblings keep
                            // their own execution turns.
                            continue;
                        }
                    }
                    let fresh = self.store.get_task(task_id)?;
                    if !is_terminal_task_state(fresh.state) {
                        self.fail_task(run, &task, AgentState::Failed)?;
                    }
                    return Err(refusal);
                }
            }
        }
        Ok(())
    }

    fn planning_prompt(&self, run: &Run) -> String {
        if self.templates.get(PLANNER_TEMPLATE_ID).is_some() {
            let mut inputs = BTreeMap::new();
            inputs.insert("task".to_owned(), run.prompt.clone());
            if let Ok(messages) = self.templates.render(PLANNER_TEMPLATE_ID, &inputs) {
                return messages
                    .iter()
                    .map(|message| format!("{:?}: {}", message.role, message.content))
                    .collect::<Vec<_>>()
                    .join("\n");
            }
        }
        run.prompt.clone()
    }

    /// Executes the read-only tool calls returned in one assistant turn and
    /// returns the planned results for the follow-up model turn.
    ///
    /// Order per call (W12 interface): persisted turn/call budget check ->
    /// [`tool_loop::plan_call`] -> task-tier/persona registry validation ->
    /// publish `TOOL_CALL` -> read-only execution through the configured
    /// [`WorkspaceJail`] -> publish `TOOL_RESULT` -> durable `tool_turns` row.
    /// Non-read-only tools, missing jail, invalid plans and exhausted budgets
    /// produce a named `TOOL_RESULT` refusal and execute nothing.
    async fn execute_tool_turn(
        &self,
        run: &Run,
        task: &TaskRecord,
        reply: &StreamedReply,
        token: &CancellationToken,
    ) -> Result<ToolTurnResult, Refusal> {
        if reply.tool_calls.is_empty() {
            return Ok(ToolTurnResult {
                cap_reached: false,
                budget_refused: false,
                executed: Vec::new(),
            });
        }
        let (completed_turns, used_calls) = self
            .db
            .run_tool_counters(run.id)
            .map_err(|error| error.refusal())?;

        let turn = match tool_loop::next_turn(completed_turns) {
            Ok(turn) => turn,
            Err(refusal) => {
                // The turn cap is already reached: every call is refused by
                // name, nothing executes, and the counters do not advance.
                for call in &reply.tool_calls {
                    if token.is_cancelled() {
                        return Err(Refusal::ToolCancelled);
                    }
                    let execution = ToolExecution::refused(
                        call.id.clone(),
                        refusal.code(),
                        format!(
                            "tool turn budget exhausted ({completed_turns} completed of {}); nothing executed",
                            tool_loop::MAX_TOOL_TURNS
                        ),
                    );
                    self.finish_tool_call(
                        run,
                        task,
                        call,
                        &execution,
                        completed_turns.saturating_add(1),
                    )?;
                }
                return Ok(ToolTurnResult {
                    cap_reached: true,
                    budget_refused: false,
                    executed: Vec::new(),
                });
            }
        };

        let mut executed: Vec<(ToolCallPlan, ToolExecution)> = Vec::new();
        let mut precounted: u32 = 0;
        let mut calls_in_turn: u32 = 0;
        let mut budget_refused = false;
        for (index, call) in reply.tool_calls.iter().enumerate() {
            if token.is_cancelled() {
                return Err(Refusal::ToolCancelled);
            }
            let index = u32::try_from(index).map_err(|_| Refusal::InternalError)?;
            // Count every attempted call, including refusals, so a model cannot
            // loop for free on rejected calls.
            let used_before = used_calls.saturating_add(index);
            let execution = match tool_loop::check_call_budget(used_before) {
                Err(refusal) => {
                    budget_refused = true;
                    ToolExecution::refused(
                        call.id.clone(),
                        refusal.code(),
                        format!(
                            "per-run tool call budget exhausted ({used_before} used of {}); nothing executed",
                            tool_loop::MAX_CALLS_PER_RUN
                        ),
                    )
                }
                Ok(()) => match tool_loop::plan_call(call, turn) {
                    Err(refusal) => ToolExecution::refused(
                        call.id.clone(),
                        refusal.code(),
                        format!(
                            "tool call refused before planning: {}",
                            refusal.message()
                        ),
                    ),
                    Ok(plan) => {
                        let execution = match self.authorize_tool_call(task, &plan) {
                            Err(refusal) => {
                                hiverouter_tools::record_refusal(
                                    task.run_id,
                                    refusal,
                                    ToolId::from_id(&plan.tool),
                                    tool_loop::permission_for(&self.tool_registry, &plan.tool),
                                );
                                ToolExecution::refused(
                                    plan.tool_call_id.clone(),
                                    refusal.code(),
                                    format!(
                                        "tool {:?} refused for task role {:?} (tier {}): {}",
                                        plan.tool,
                                        task.role,
                                        task_agent_tier(task).label(),
                                        refusal.message()
                                    ),
                                )
                            }
                            Ok(tier) => {
                                self.publish_tool_call(run, task, &plan);
                                if plan.tool == "spawn_agent" {
                                    // Real model-spawned child path (M-010): caps are
                                    // enforced before the child row exists, the child
                                    // runs to a terminal state here, and its result is
                                    // returned as this tool's result. No approval is
                                    // consumed: spawning is loop-owned, never a
                                    // workspace side effect.
                                    self.execute_spawn_agent(run, task, &plan, token)
                                        .await
                                } else if plan.tool == "ask_parent" {
                                    self.execute_ask_parent(run, task, &plan)
                                } else if plan.tool == "send_to_sibling" {
                                    self.execute_send_to_sibling(run, task, &plan)
                                } else if tool_loop::requires_approval(&self.tool_registry, &plan.tool)
                                    && self.tools.is_some()
                                    && self.tool_workspace.is_some()
                                {
                                    // D12: count the attempt before the wait so a
                                    // deny/timeout cannot retry for free.
                                    self.db
                                        .increment_run_tool_counters(run.id, 0, 1)
                                        .map_err(|error| error.refusal())?;
                                    precounted = precounted.saturating_add(1);
                                    // Terminal waits finalize their own TOOL_RESULT and
                                    // return Err (D13); approve/deny return Ok.
                                    self.suspend_for_approval(run, task, call, &plan, turn, token)
                                        .await?
                                } else {
                                    self.execute_tool_plan(&plan, tier)
                                }
                            }
                        };
                        executed.push((plan, execution.clone()));
                        execution
                    }
                },
            };
            calls_in_turn = calls_in_turn.saturating_add(1);
            self.finish_tool_call(run, task, call, &execution, turn)?;
        }
        if calls_in_turn > 0 {
            self.db
                .increment_run_tool_counters(
                    run.id,
                    1,
                    calls_in_turn.saturating_sub(precounted),
                )
                .map_err(|error| error.refusal())?;
        }
        Ok(ToolTurnResult {
            cap_reached: false,
            budget_refused,
            executed,
        })
    }

    /// Builds the per-provider-call delta sink and its invocation counter.
    ///
    /// A fresh sink per turn keeps the "publish returned deltas only when the
    /// sink was never used" rule exact, so a streamed fragment is never emitted
    /// twice across follow-up turns.
    fn reply_delta_sink(&self, run_id: RunId) -> (DeltaSink, Arc<AtomicUsize>) {
        let streamed = Arc::new(AtomicUsize::new(0));
        let sink: DeltaSink = {
            let bus = Arc::clone(&self.bus);
            let counter = Arc::clone(&streamed);
            Arc::new(move |delta: &str| {
                if delta.is_empty() {
                    return;
                }
                counter.fetch_add(1, Ordering::SeqCst);
                if let Err(error) = bus.publish_for_run(
                    Event::ReplyDelta(ReplyDelta {
                        run_id,
                        text_delta: delta.to_owned(),
                    }),
                    Some(run_id),
                    None,
                ) {
                    tracing::error!(
                        code = error.code(),
                        "failed to publish a streamed REPLY_DELTA"
                    );
                }
            })
        };
        (sink, streamed)
    }

    /// Publishes an executor's returned fragments only when its sink was unused.
    fn publish_returned_deltas(
        &self,
        run_id: RunId,
        reply: &StreamedReply,
        streamed: &Arc<AtomicUsize>,
    ) {
        if streamed.load(Ordering::SeqCst) > 0 {
            return;
        }
        for delta in &reply.deltas {
            if delta.is_empty() {
                continue;
            }
            self.publish(
                Event::ReplyDelta(ReplyDelta {
                    run_id,
                    text_delta: delta.clone(),
                }),
                Some(run_id),
                None,
            );
        }
    }

    /// Marks the task terminal for a failed/cancelled turn (store+graph+event).
    ///
    /// Terminal-aware: a task already `Killed`/`Cancelled`/`Failed`/... is never
    /// overwritten by a late failure write (M-002).
    fn fail_task(&self, run: &Run, task: &TaskRecord, state: AgentState) -> Result<(), Refusal> {
        let event = match state {
            AgentState::Cancelled => StateEvent::Cancel,
            AgentState::Killed => StateEvent::Kill,
            AgentState::Interrupted => StateEvent::Restart,
            _ => StateEvent::Fail,
        };
        if let Some(transition) = self.apply_task_state(task.id, event)? {
            self.publish_node(run.id, task.id, transition.to);
        }
        Ok(())
    }

    /// Routes one validated plan to the single read-only executor.
    ///
    /// Read-only is a property of the frozen registry contract: a non-read tool
    /// is refused here and never reaches the executor or the jail. Without a
    /// configured jail a read call is refused as `E_FILE_TREE_UNAVAILABLE`.
    fn execute_tool_plan(&self, plan: &ToolCallPlan, tier: AgentTier) -> ToolExecution {
        if tool_loop::permission_for(&self.tool_registry, &plan.tool)
            != Some(PermissionClass::Read)
        {
            return ToolExecution::refused(
                plan.tool_call_id.clone(),
                Refusal::ToolNotAllowed.code(),
                format!(
                    "tool {:?} is not read-only; side-effecting calls stay \
                     approval/loop-owned and did not execute",
                    plan.tool
                ),
            );
        }
        let Some(workspace) = self.tool_workspace.as_ref() else {
            return ToolExecution::refused(
                plan.tool_call_id.clone(),
                Refusal::FileTreeUnavailable.code(),
                "no workspace jail is configured; the read-only tool did not execute".to_owned(),
            );
        };
        match execute_read_only(
            workspace,
            &self.tool_registry,
            tier,
            &plan.tool,
            &plan.args,
        ) {
            Ok(ToolsExecutedTool::Ok { text }) => ToolExecution::ok(
                plan.tool_call_id.clone(),
                tool_loop::bounded_tool_text(&text),
            ),
            Ok(ToolsExecutedTool::Refused { code, detail }) => {
                ToolExecution::refused(plan.tool_call_id.clone(), code, detail)
            }
            Err(error) => ToolExecution::refused(
                plan.tool_call_id.clone(),
                error.code(),
                error.detail().to_owned(),
            ),
        }
    }

    /// Publishes `TOOL_CALL` for a validated plan (event errors are logged).
    fn publish_tool_call(&self, run: &Run, task: &TaskRecord, plan: &ToolCallPlan) {
        let payload = tool_loop::call_payload(plan, run.id, task.id);
        if let Err(error) = self
            .bus
            .publish_for_run(Event::ToolCall(payload), Some(run.id), None)
        {
            tracing::error!(code = error.code(), "failed to publish TOOL_CALL");
        }
    }

    /// Publishes `TOOL_RESULT` and persists the durable `tool_turns` row.
    fn finish_tool_call(
        &self,
        run: &Run,
        task: &TaskRecord,
        call: &AssembledToolCall,
        execution: &ToolExecution,
        turn: u32,
    ) -> Result<(), Refusal> {
        if let Err(error) = self.bus.publish_for_run(
            Event::ToolResult(execution.payload(run.id, task.id)),
            Some(run.id),
            None,
        ) {
            tracing::error!(code = error.code(), "failed to publish TOOL_RESULT");
        }
        let record = ToolTurnRecord {
            id: Uuid::now_v7().to_string(),
            run_id: run.id,
            task_id: task.id,
            tool_call_id: execution.tool_call_id.clone(),
            tool: call.name.clone(),
            args_sha256: hiverouter_tools::hash_args(&call.arguments),
            outcome: execution.outcome,
            code: execution.code.clone(),
            bytes: execution.bytes,
            sha256: execution.sha256.clone(),
            turn,
            created_at: Timestamp::now(),
        };
        if let (Some(tool_id), Some(permission_class)) = (
            ToolId::from_id(&call.name),
            tool_loop::permission_for(&self.tool_registry, &call.name),
        ) {
            hiverouter_tools::record_tool_call(
                run.id,
                tool_id,
                permission_class,
                execution.outcome.as_str(),
                execution.code.as_deref(),
            );
        }
        self.db
            .record_tool_turn(&record)
            .map_err(|error| error.refusal())
    }

    fn now_unix_secs(&self) -> u64 {
        (self.clock)()
    }

    /// Applies the run+task Suspend transitions exactly once and publishes
    /// `RUN_STATUS awaiting_approval` + `NODE_STATE awaiting_approval`.
    ///
    /// The task transition is applied first from the freshest state so a task
    /// killed before the suspend can never leave the run suspended with a
    /// terminal task.
    fn persist_approval_transitions(&self, run: &Run, task: &TaskRecord) -> Result<(), Refusal> {
        let current_run = self.store.get_run(run.id)?;
        let run_transition = run_state::apply_run(current_run.status, StateEvent::Suspend)
            .map_err(|error| error.refusal())?;
        let Some(task_transition) = self.apply_task_state(task.id, StateEvent::Suspend)? else {
            return Err(Refusal::TaskTerminal);
        };
        self.store.set_run_status(run.id, run_transition.to, None)?;
        self.publish(
            Event::RunStatus(RunStatusPayload {
                run_id: run.id,
                status: run_transition.to,
                error_code: None,
            }),
            Some(run.id),
            None,
        );
        self.publish_node(run.id, task.id, task_transition.to);
        Ok(())
    }

    /// Applies run+task Resume and publishes `RUN_STATUS running` + NODE running.
    fn resume_transitions(&self, run: &Run, task: &TaskRecord) -> Result<(), Refusal> {
        let current_run = self.store.get_run(run.id)?;
        let run_transition = run_state::apply_run(current_run.status, StateEvent::Resume)
            .map_err(|error| error.refusal())?;
        let Some(task_transition) = self.apply_task_state(task.id, StateEvent::Resume)? else {
            return Err(Refusal::TaskTerminal);
        };
        self.store.set_run_status(run.id, run_transition.to, None)?;
        self.publish(
            Event::RunStatus(RunStatusPayload {
                run_id: run.id,
                status: run_transition.to,
                error_code: None,
            }),
            Some(run.id),
            None,
        );
        self.publish_node(run.id, task.id, task_transition.to);
        Ok(())
    }

    /// Commits one approval, arms the waiter, persists `awaiting_approval`,
    /// publishes `APPROVAL_REQUEST`, then waits for resolution/TTL/cancel.
    async fn suspend_for_approval(
        &self,
        run: &Run,
        task: &TaskRecord,
        call: &AssembledToolCall,
        plan: &ToolCallPlan,
        turn: u32,
        token: &CancellationToken,
    ) -> Result<ToolExecution, Refusal> {
        let Some(tools) = self.tools.as_ref() else {
            return Ok(ToolExecution::refused(
                plan.tool_call_id.clone(),
                Refusal::ToolNotAllowed.code(),
                "approval bridge is not wired; the side effect did not execute".to_owned(),
            ));
        };
        if self.tool_workspace.is_none() {
            return Ok(ToolExecution::refused(
                plan.tool_call_id.clone(),
                Refusal::FileTreeUnavailable.code(),
                "no workspace jail is configured; the side effect did not execute".to_owned(),
            ));
        }
        let waits = tools
            .approval_count_for_run(run.id)?;
        if waits >= u64::from(tool_loop::MAX_APPROVAL_WAITS_PER_RUN) {
            return Ok(ToolExecution::refused(
                plan.tool_call_id.clone(),
                Refusal::CallLimit.code(),
                format!(
                    "approval wait budget exhausted ({waits} of {}); nothing executed",
                    tool_loop::MAX_APPROVAL_WAITS_PER_RUN
                ),
            ));
        }

        let now = self.now_unix_secs();
        let event = tools
            .request_model_tool_approval(run.id, task.id, &plan.tool_call_id, &plan.tool, &plan.args)?;
        let approval_key = event.approval_id;

        // Arm BEFORE publishing: a fast resolve can never be lost.
        let receiver = self
            .waiters
            .arm(approval_key)
            .map_err(|_| Refusal::InternalError)?;
        let _armed = ArmedApproval {
            waiters: Arc::clone(&self.waiters),
            approval_id: approval_key,
        };

        // Persist + publish only after the row is committed and the waiter armed.
        if let Err(refusal) = self.persist_approval_transitions(run, task) {
            let _ = tools.expire_approval(&approval_key.to_string(), now);
            return Err(refusal);
        }
        // M-004: an awaiting-approval crash restores this committed graph.
        self.commit_run_snapshot(
            run,
            self.snapshot_revision(run.id),
            AgentState::AwaitingApproval,
        );
        self.publish(Event::ApprovalRequest(event.clone()), Some(run.id), None);

        // A request can already be decided by a concurrent driver/retry.
        if let Some(label) = tools
            .approval_decision(&approval_key.to_string())?
        {
            return self
                .apply_committed_decision(
                    run,
                    task,
                    call,
                    plan,
                    turn,
                    token,
                    tools,
                    &approval_key.to_string(),
                    &label,
                )
                .await;
        }

        let expires = event
            .expires_at
            .as_ref()
            .and_then(|timestamp| timestamp.unix_secs().ok())
            .unwrap_or_else(|| now.saturating_add(tools.approval_gate().ttl_secs()));
        let remaining = expires.saturating_sub(self.now_unix_secs()).max(1);
        let wait = tokio::time::timeout(std::time::Duration::from_secs(remaining), receiver);
        let wake = tokio::select! {
            biased;
            _ = token.cancelled() => None,
            result = wait => Some(result),
        };

        let approval_id = approval_key.to_string();
        match wake {
            None => {
                self.finish_cancelled_wait(run, task, call, plan, turn, tools, &approval_id)
                    .await
            }
            Some(Err(_elapsed)) => {
                self.finish_timeout_wait(run, task, call, plan, turn, tools, &approval_id, token)
                    .await
            }
            Some(Ok(Err(_closed))) => {
                let _ = tools.cancel_approval(&approval_id, self.now_unix_secs());
                self.finish_cancelled_wait(run, task, call, plan, turn, tools, &approval_id)
                    .await
            }
            Some(Ok(Ok(ApprovalWake::Cancelled))) => {
                self.finish_cancelled_wait(run, task, call, plan, turn, tools, &approval_id)
                    .await
            }
            Some(Ok(Ok(ApprovalWake::Resolved(resolution)))) => {
                if token.is_cancelled() {
                    let _ = tools.cancel_approval(&approval_id, self.now_unix_secs());
                    return self
                        .finish_cancelled_wait(run, task, call, plan, turn, tools, &approval_id)
                        .await;
                }
                match resolution.decision {
                    ApprovalDecision::AllowOnce | ApprovalDecision::AllowSession => {
                        self.resume_and_execute(run, task, plan, token).await
                    }
                    ApprovalDecision::Deny => self.resume_denied(run, task, plan),
                }
            }
        }
    }

    /// Applies an already-committed decision found right after arming or after
    /// the timeout CAS loses.
    ///
    /// The `expired` outcome is terminal and single-owner: it is routed to
    /// [`Self::terminal_approval_expired`], which never re-enters the timeout
    /// wait (M-001).
    async fn apply_committed_decision(
        &self,
        run: &Run,
        task: &TaskRecord,
        call: &AssembledToolCall,
        plan: &ToolCallPlan,
        turn: u32,
        token: &CancellationToken,
        tools: &ToolsBridge,
        approval_id: &str,
        label: &str,
    ) -> Result<ToolExecution, Refusal> {
        match ApprovalDecision::from_label(label) {
            Some(ApprovalDecision::AllowOnce | ApprovalDecision::AllowSession) => {
                self.resume_and_execute(run, task, plan, token).await
            }
            Some(ApprovalDecision::Deny) => self.resume_denied(run, task, plan),
            None if label == "expired" => {
                // Bounded terminal path: finalize the run and return. This arm
                // must never call finish_timeout_wait again (unbounded recursion).
                self.terminal_approval_expired(run, task, call, plan, turn, tools, approval_id)
            }
            None => {
                self.finish_cancelled_wait(run, task, call, plan, turn, tools, approval_id)
                    .await
            }
        }
    }

    /// Approve path: persist Running, then one granted-executor call
    /// (preflight -> single-use grant CAS -> exactly one effect).
    async fn resume_and_execute(
        &self,
        run: &Run,
        task: &TaskRecord,
        plan: &ToolCallPlan,
        token: &CancellationToken,
    ) -> Result<ToolExecution, Refusal> {
        let tools = self.tools.as_ref().ok_or(Refusal::InternalError)?;
        self.resume_transitions(run, task)?;
        // The granted call is authorized for the task's actual tier/persona, not
        // the historic hardcoded WorkerAgent (M-015). A refused re-authorization
        // never reaches execute_approved, so the consent is not consumed.
        let tier = task_agent_tier(task);
        if let Err(refusal) = self.authorize_tool_call(task, plan) {
            hiverouter_tools::record_refusal(
                task.run_id,
                refusal,
                ToolId::from_id(&plan.tool),
                tool_loop::permission_for(&self.tool_registry, &plan.tool),
            );
            return Ok(ToolExecution::refused(
                plan.tool_call_id.clone(),
                refusal.code(),
                format!(
                    "the granted tool {:?} is no longer authorized for task role {:?} \
                     (tier {}): {}",
                    plan.tool,
                    task.role,
                    tier.label(),
                    refusal.message()
                ),
            ));
        }
        let executed = tools
            .execute_approved(
                &self.shell_policy,
                &self.tool_registry,
                tier,
                run.id,
                &plan.tool,
                &plan.args,
                Some(token),
            )
            .await;
        Ok(match executed {
            Ok(ToolsExecutedTool::Ok { text }) => {
                ToolExecution::ok(plan.tool_call_id.clone(), tool_loop::bounded_tool_text(&text))
            }
            Ok(ToolsExecutedTool::Refused { code, detail }) => {
                ToolExecution::refused(plan.tool_call_id.clone(), code, detail)
            }
            Err(refusal) => ToolExecution::refused(
                plan.tool_call_id.clone(),
                refusal.code(),
                refusal.message().to_owned(),
            ),
        })
    }

    /// Deny path: persist Running and return the denied execution (no effect).
    fn resume_denied(
        &self,
        run: &Run,
        task: &TaskRecord,
        plan: &ToolCallPlan,
    ) -> Result<ToolExecution, Refusal> {
        let current_run = self.store.get_run(run.id)?;
        let run_transition = run_state::apply_run(current_run.status, StateEvent::Deny)
            .map_err(|error| error.refusal())?;
        let Some(task_transition) = self.apply_task_state(task.id, StateEvent::Deny)? else {
            return Err(Refusal::TaskTerminal);
        };
        self.store.set_run_status(run.id, run_transition.to, None)?;
        self.publish(
            Event::RunStatus(RunStatusPayload {
                run_id: run.id,
                status: run_transition.to,
                error_code: None,
            }),
            Some(run.id),
            None,
        );
        self.publish_node(run.id, task.id, task_transition.to);
        Ok(ToolExecution::denied(plan.tool_call_id.clone()))
    }

    /// Timeout: expire CAS first; a won CAS finalizes one timeout TOOL_RESULT and
    /// the terminal failed run with `E_APPROVAL_TIMEOUT`.
    async fn finish_timeout_wait(
        &self,
        run: &Run,
        task: &TaskRecord,
        call: &AssembledToolCall,
        plan: &ToolCallPlan,
        turn: u32,
        tools: &ToolsBridge,
        approval_id: &str,
        token: &CancellationToken,
    ) -> Result<ToolExecution, Refusal> {
        let now = self.now_unix_secs();
        if tools.expire_approval(approval_id, now)? {
            self.finalize_approval_timeout(run, task, call, plan, turn)?;
            return Err(Refusal::ApprovalTimeout);
        }
        match tools.approval_decision(approval_id)? {
            Some(label) => {
                self.apply_committed_decision(
                    run, task, call, plan, turn, token, tools, approval_id, &label,
                )
                .await
            }
            None => Err(Refusal::InternalError),
        }
    }

    /// Terminal single-owner outcome for an approval that is already expired
    /// (M-001).
    ///
    /// A late resolve at/after `expires_at` commits `decision = 'expired'`
    /// without publishing a run status. This helper claims the row once,
    /// finalizes the timeout task transition and exactly one terminal
    /// `RUN_STATUS failed` with `E_APPROVAL_TIMEOUT`, cancels the wait token, and
    /// returns. It never re-enters the timeout wait, so the expired label can
    /// never recurse and `active_run` is released by the normal dispatch
    /// cleanup that follows this error.
    fn terminal_approval_expired(
        &self,
        run: &Run,
        task: &TaskRecord,
        call: &AssembledToolCall,
        plan: &ToolCallPlan,
        turn: u32,
        tools: &ToolsBridge,
        approval_id: &str,
    ) -> Result<ToolExecution, Refusal> {
        // Claim/expire once. `false` is legal: a late resolve already marked the
        // row expired; this call still owns the one terminal run transition.
        let _ = tools.expire_approval(approval_id, self.now_unix_secs());
        self.finalize_approval_timeout(run, task, call, plan, turn)?;
        // Stop in-flight work: the run is terminal, so nothing else may execute.
        if let Some(token) = self.tokens.token(run.id) {
            token.cancel();
        }
        Err(Refusal::ApprovalTimeout)
    }

    /// Persists the single terminal approval-timeout outcome for one wait.
    ///
    /// Used by the timeout CAS winner and by the expired-approval terminal path,
    /// so both routes produce exactly one timeout `TOOL_RESULT`, one failed task
    /// and one terminal `RUN_STATUS failed` (`E_APPROVAL_TIMEOUT`). An already
    /// terminal run is never published twice (single owner).
    fn finalize_approval_timeout(
        &self,
        run: &Run,
        task: &TaskRecord,
        call: &AssembledToolCall,
        plan: &ToolCallPlan,
        turn: u32,
    ) -> Result<(), Refusal> {
        let execution = ToolExecution::timed_out(plan.tool_call_id.clone());
        let tool_result = self.finish_tool_call(run, task, call, &execution, turn);
        if let Err(error) = &tool_result {
            // The run must still be terminalized; report the persistence failure
            // after the terminal state is written instead of leaving the run
            // active forever.
            tracing::error!(code = error.code(), "failed to persist the timeout TOOL_RESULT");
        }
        let task_state = match self.apply_task_state(task.id, StateEvent::Timeout) {
            Ok(Some(transition)) => Some(transition.to),
            Ok(None) => None,
            Err(error) => {
                tracing::error!(
                    code = error.code(),
                    "approval-timeout task transition refused; run is still terminalized"
                );
                None
            }
        };
        if let Some(state) = task_state {
            self.publish_node(run.id, task.id, state);
        }
        // Serialized with the per-task kill run resume: a terminal run is never
        // resurrected, and the terminal frame is published exactly once.
        self.with_lifecycle_lock(|| {
            let current_run = self.store.get_run(run.id)?;
            if current_run.status.is_terminal() {
                return Ok(());
            }
            let run_transition = if current_run.status == AgentState::AwaitingApproval {
                run_state::apply_run(current_run.status, StateEvent::Timeout)
            } else {
                run_state::apply_run(current_run.status, StateEvent::Fail)
            }
            .map_err(|error| error.refusal())?;
            let error_code = run_transition
                .effect
                .refusal()
                .unwrap_or(Refusal::ApprovalTimeout)
                .code()
                .to_owned();
            self.store
                .set_run_status(run.id, run_transition.to, Some(&error_code))?;
            self.publish(
                Event::RunStatus(RunStatusPayload {
                    run_id: run.id,
                    status: run_transition.to,
                    error_code: Some(error_code),
                }),
                Some(run.id),
                None,
            );
            Ok(())
        })?;
        tool_result
    }

    /// Cancel: consume the row, mark the task cancelled once, finish one
    /// cancelled TOOL_RESULT, return `E_TOOL_CANCELLED`.
    async fn finish_cancelled_wait(
        &self,
        run: &Run,
        task: &TaskRecord,
        call: &AssembledToolCall,
        plan: &ToolCallPlan,
        turn: u32,
        tools: &ToolsBridge,
        approval_id: &str,
    ) -> Result<ToolExecution, Refusal> {
        let _ = tools.cancel_approval(approval_id, self.now_unix_secs());
        if let Some(transition) = self.apply_task_state(task.id, StateEvent::Cancel)? {
            self.publish_node(run.id, task.id, transition.to);
        }
        // A per-task cancellation (not a run cancel) must not leave the run
        // suspended on a wait that no longer exists (M-012).
        if !self.tokens.is_cancelled(run.id) {
            self.resume_run_after_task_kill(run.id)?;
        }
        let execution = ToolExecution::cancelled(plan.tool_call_id.clone());
        self.finish_tool_call(run, task, call, &execution, turn)?;
        Err(Refusal::ToolCancelled)
    }

    /// Executes one model-originated `ask_parent` call.
    fn execute_ask_parent(
        &self,
        run: &Run,
        task: &TaskRecord,
        plan: &ToolCallPlan,
    ) -> ToolExecution {
        let args: AskParentArgs = match serde_json::from_value(plan.args.clone()) {
            Ok(args) => args,
            Err(error) => {
                return ToolExecution::refused(
                    plan.tool_call_id.clone(),
                    Refusal::ToolSchema.code(),
                    format!("ask_parent arguments do not match the schema: {error}"),
                );
            }
        };
        let Some(parent) = task.parent_task_id else {
            return ToolExecution::refused(
                plan.tool_call_id.clone(),
                Refusal::TaskNotFound.code(),
                "ask_parent refused: the calling task has no parent".to_owned(),
            );
        };
        let text = args.text.trim();
        if text.is_empty() || text.chars().count() > MAX_AGENT_MESSAGE_CHARS {
            return ToolExecution::refused(
                plan.tool_call_id.clone(),
                Refusal::ToolSchema.code(),
                "ask_parent text must be non-empty and bounded".to_owned(),
            );
        }
        match self.store.agent_message_count(run.id) {
            Ok(count) if count >= MAX_AGENT_MESSAGES_PER_RUN => {
                return ToolExecution::refused(
                    plan.tool_call_id.clone(),
                    Refusal::CallLimit.code(),
                    "ask_parent refused: the per-run agent-message budget is exhausted".to_owned(),
                );
            }
            Ok(_) => {}
            Err(refusal) => {
                return ToolExecution::refused(
                    plan.tool_call_id.clone(),
                    refusal.code(),
                    refusal.message().to_owned(),
                );
            }
        }
        match self
            .store
            .record_agent_message(run.id, task.id, parent, "ask_parent", text)
        {
            Ok(message_id) => {
                self.publish_agent_message(run.id, task.id, parent, "ask_parent", text);
                ToolExecution::ok(
                    plan.tool_call_id.clone(),
                    serde_json::json!({
                        "message_id": message_id.to_string(),
                        "to_task_id": parent.to_string(),
                        "kind": "ask_parent",
                    })
                    .to_string(),
                )
            }
            Err(refusal) => ToolExecution::refused(
                plan.tool_call_id.clone(),
                refusal.code(),
                refusal.message().to_owned(),
            ),
        }
    }

    /// Executes one model-originated `send_to_sibling` call.
    ///
    /// Same-parent only: this is the narrow, non-leaking peer route until a
    /// profile-level cross-branch approval gate lands.
    fn execute_send_to_sibling(
        &self,
        run: &Run,
        task: &TaskRecord,
        plan: &ToolCallPlan,
    ) -> ToolExecution {
        let args: SendToSiblingArgs = match serde_json::from_value(plan.args.clone()) {
            Ok(args) => args,
            Err(error) => {
                return ToolExecution::refused(
                    plan.tool_call_id.clone(),
                    Refusal::ToolSchema.code(),
                    format!("send_to_sibling arguments do not match the schema: {error}"),
                );
            }
        };
        let Ok(target_id) = args.target_task_id.parse::<TaskId>() else {
            return ToolExecution::refused(
                plan.tool_call_id.clone(),
                Refusal::SchemaInvalid.code(),
                "send_to_sibling target_task_id is not a task id".to_owned(),
            );
        };
        if target_id == task.id {
            return ToolExecution::refused(
                plan.tool_call_id.clone(),
                Refusal::SchemaInvalid.code(),
                "send_to_sibling cannot target the calling task".to_owned(),
            );
        }
        let target = match self.store.get_task(target_id) {
            Ok(target) => target,
            Err(refusal) => {
                return ToolExecution::refused(
                    plan.tool_call_id.clone(),
                    refusal.code(),
                    refusal.message().to_owned(),
                );
            }
        };
        if target.run_id != task.run_id || target.parent_task_id != task.parent_task_id {
            return ToolExecution::refused(
                plan.tool_call_id.clone(),
                Refusal::ToolNotAllowed.code(),
                "send_to_sibling is limited to the same run and the same parent".to_owned(),
            );
        }
        let text = args.text.trim();
        if text.is_empty() || text.chars().count() > MAX_AGENT_MESSAGE_CHARS {
            return ToolExecution::refused(
                plan.tool_call_id.clone(),
                Refusal::ToolSchema.code(),
                "send_to_sibling text must be non-empty and bounded".to_owned(),
            );
        }
        match self.store.agent_message_count(run.id) {
            Ok(count) if count >= MAX_AGENT_MESSAGES_PER_RUN => {
                return ToolExecution::refused(
                    plan.tool_call_id.clone(),
                    Refusal::CallLimit.code(),
                    "send_to_sibling refused: the per-run agent-message budget is exhausted"
                        .to_owned(),
                );
            }
            Ok(_) => {}
            Err(refusal) => {
                return ToolExecution::refused(
                    plan.tool_call_id.clone(),
                    refusal.code(),
                    refusal.message().to_owned(),
                );
            }
        }
        match self
            .store
            .record_agent_message(run.id, task.id, target_id, "send_to_sibling", text)
        {
            Ok(message_id) => {
                self.publish_agent_message(run.id, task.id, target_id, "send_to_sibling", text);
                ToolExecution::ok(
                    plan.tool_call_id.clone(),
                    serde_json::json!({
                        "message_id": message_id.to_string(),
                        "to_task_id": target_id.to_string(),
                        "kind": "send_to_sibling",
                    })
                    .to_string(),
                )
            }
            Err(refusal) => ToolExecution::refused(
                plan.tool_call_id.clone(),
                refusal.code(),
                refusal.message().to_owned(),
            ),
        }
    }


    ///
    /// Resolves the calling task, enforces the parent/depth/fan-out/agent caps
    /// through [`Self::spawn_sub_hive`] before any child row is created, runs the
    /// child to a terminal state, and returns the child's result as this tool's
    /// result so the model can consume it. Every refusal is a named tool-result
    /// refusal, never a fabricated success.
    async fn execute_spawn_agent(
        &self,
        run: &Run,
        task: &TaskRecord,
        plan: &ToolCallPlan,
        token: &CancellationToken,
    ) -> ToolExecution {
        let args: SpawnAgentArgs = match serde_json::from_value(plan.args.clone()) {
            Ok(args) => args,
            Err(error) => {
                return ToolExecution::refused(
                    plan.tool_call_id.clone(),
                    Refusal::ToolSchema.code(),
                    format!("spawn_agent arguments do not match the frozen schema: {error}"),
                );
            }
        };
        if let Some(claim) = args.parent_task_id.as_deref() {
            if claim != task.id.to_string() {
                return ToolExecution::refused(
                    plan.tool_call_id.clone(),
                    Refusal::ToolNotAllowed.code(),
                    format!(
                        "spawn_agent parent_task_id claim {claim:?} does not match the calling \
                         task {}; no child was created",
                        task.id
                    ),
                );
            }
        }
        if args.role.trim().is_empty() {
            return ToolExecution::refused(
                plan.tool_call_id.clone(),
                Refusal::ToolSchema.code(),
                "spawn_agent role must not be empty; no child was created".to_owned(),
            );
        }
        let child = match self.spawn_sub_hive(task.id, args.role.trim()) {
            Ok(child) => child,
            Err(refusal) => {
                return ToolExecution::refused(
                    plan.tool_call_id.clone(),
                    refusal.code(),
                    format!(
                        "spawn_agent refused before any child was created or dispatched: {}",
                        refusal.message()
                    ),
                );
            }
        };
        // The model awaits this child: run it now instead of leaving it in the
        // deferred post-plan queue.
        let _ = self.take_queued_task(child.id);
        let instruction = args.instruction.trim();
        let child_prompt = if instruction.is_empty() {
            run.prompt.clone()
        } else {
            format!(
                "{}\n\nChild agent instruction (role {:?}):\n{}",
                run.prompt, child.role, instruction
            )
        };
        // Boxing breaks the async type cycle execute_task -> tool loop -> spawn.
        let result = Box::pin(self.execute_task_with_prompt(run, &child, token, &child_prompt)).await;
        match result {
            Ok(Some(text)) => {
                let payload = serde_json::json!({
                    "child_task_id": child.id.to_string(),
                    "role": child.role,
                    "depth": child.depth,
                    "state": AgentState::Completed.as_str(),
                    "result": tool_loop::bounded_tool_text(&text),
                });
                ToolExecution::ok(plan.tool_call_id.clone(), payload.to_string())
            }
            Ok(None) => {
                let state = self
                    .store
                    .get_task(child.id)
                    .map(|child| child.state)
                    .unwrap_or(AgentState::Failed);
                let code = if matches!(state, AgentState::Killed | AgentState::Cancelled) {
                    Refusal::ToolCancelled.code()
                } else {
                    Refusal::TaskTerminal.code()
                };
                ToolExecution::refused(
                    plan.tool_call_id.clone(),
                    code,
                    format!(
                        "child task {} ended {} before returning a result",
                        child.id, state
                    ),
                )
            }
            Err(refusal) => {
                let fresh = self.store.get_task(child.id).unwrap_or_else(|_| child.clone());
                if !is_terminal_task_state(fresh.state) {
                    let _ = self.fail_task(run, &child, AgentState::Failed);
                }
                ToolExecution::refused(
                    plan.tool_call_id.clone(),
                    refusal.code(),
                    format!(
                        "child task {} failed before reporting a result: {}",
                        child.id,
                        refusal.message()
                    ),
                )
            }
        }
    }

    /// Executes one planned/spawned task with the run prompt.
    async fn execute_task(
        &self,
        run: &Run,
        task: &TaskRecord,
        token: &CancellationToken,
    ) -> Result<Option<String>, Refusal> {
        self.execute_task_with_prompt(run, task, token, &run.prompt)
            .await
    }

    /// Executes one task with an explicit prompt.
    ///
    /// A plan/spawned task uses the run prompt; a model-spawned child adds its
    /// instruction. The returned text is the persisted final assistant reply
    /// (`None` when the task was already terminal and completion was suppressed).
    async fn execute_task_with_prompt(
        &self,
        run: &Run,
        task: &TaskRecord,
        token: &CancellationToken,
        prompt: &str,
    ) -> Result<Option<String>, Refusal> {
        let task_token = self.register_task_token(task.id, token)?;
        let context = self.contexts.get(task.id)?;
        let prompt = prompt_with_context(prompt, context.as_ref());
        let result = self
            .execute_task_inner(run, task, &task_token, &prompt)
            .await;
        self.unregister_task_token(task.id);
        result
    }

    async fn execute_task_inner(
        &self,
        run: &Run,
        task: &TaskRecord,
        token: &CancellationToken,
        prompt: &str,
    ) -> Result<Option<String>, Refusal> {
        if let Err(refusal) = self.caps.check_calls(1) {
            self.fail_task(run, task, AgentState::Failed)?;
            return Err(refusal);
        }
        let Some(start) = self.apply_task_state(task.id, StateEvent::Start)? else {
            // The task was killed/cancelled before its first turn: no provider
            // call, no reply, and no resurrection of its terminal state.
            return Ok(None);
        };
        self.publish_node(run.id, task.id, start.to);
        // M-004: commit the node-state change before the provider call so a
        // crash during execution restores the running node, not only the plan.
        self.commit_run_snapshot(
            run,
            self.snapshot_revision(run.id),
            AgentState::Running,
        );

        // Real-time delta publishing: a fresh sink per provider call keeps the
        // streaming/no-duplicate bookkeeping exact across follow-up tool turns.
        let (sink, streamed_deltas) = self.reply_delta_sink(run.id);

        let reply = match tokio::select! {
            _ = token.cancelled() => {
                self.fail_task(run, task, AgentState::Cancelled)?;
                return Err(Refusal::ToolCancelled);
            }
            result = self.executor.execute(run.id, task, prompt, token, &sink) => result,
        } {
            Ok(reply) => reply,
            Err(refusal) => {
                let state = if refusal == Refusal::ToolCancelled {
                    AgentState::Cancelled
                } else {
                    AgentState::Failed
                };
                self.fail_task(run, task, state)?;
                return Err(refusal);
            }
        };

        self.publish_returned_deltas(run.id, &reply, &streamed_deltas);
        // Follow-up tool turns (1.2): feed the bounded read-only results back to
        // the model and continue until it answers, the persisted caps stop the
        // loop, or a provider/tool refusal fails the run. Only the final reply
        // text is persisted, exactly once.
        let mut final_reply = reply;
        let mut transcript: Vec<ToolTurnTranscript> = Vec::new();
        loop {
            if final_reply.tool_calls.is_empty() {
                break;
            }
            if token.is_cancelled() {
                self.fail_task(run, task, AgentState::Cancelled)?;
                return Err(Refusal::ToolCancelled);
            }
            let turn = match self.execute_tool_turn(run, task, &final_reply, token).await {
                Ok(turn) => turn,
                Err(refusal) => {
                    // A wait terminal path (timeout/cancel) already persisted and
                    // published the task terminal state; never duplicate or
                    // overwrite it with a second failure frame.
                    let fresh = self.store.get_task(task.id)?;
                    if !fresh.state.is_terminal() {
                        let state = if refusal == Refusal::ToolCancelled {
                            AgentState::Cancelled
                        } else {
                            AgentState::Failed
                        };
                        self.fail_task(run, task, state)?;
                    }
                    return Err(refusal);
                }
            };
            if turn.cap_reached {
                // The persisted turn cap refused the calls; the loop cannot
                // continue honestly, so the run fails by name (no fake reply).
                self.fail_task(run, task, AgentState::Failed)?;
                return Err(Refusal::BudgetExceeded);
            }
            if turn.executed.is_empty() {
                if turn.budget_refused {
                    // The provider-call budget stopped the loop before any new
                    // plan existed; this is an honest E_CALL_LIMIT, not a
                    // malformed-provider response.
                    self.fail_task(run, task, AgentState::Failed)?;
                    return Err(Refusal::CallLimit);
                }
                // Nothing could be planned (malformed calls), so there is no
                // provider transcript to continue with; fail honestly.
                self.fail_task(run, task, AgentState::Failed)?;
                return Err(Refusal::ProviderInvalidResponse);
            }
            // A successful `finish` call is the model's explicit terminal
            // signal: its verdict becomes the final assistant message and the
            // task completes without another provider round-trip. A refused
            // finish falls through to the ordinary transcript so the model can
            // correct itself instead of the engine fabricating completion.
            if let Some((finish_plan, finish_result)) = turn.executed.iter().find(|(plan, result)| {
                plan.tool == "finish" && result.outcome == hiverouter_contract::ToolOutcome::Ok
            }) {
                let verdict = finish_plan
                    .args
                    .get("verdict")
                    .and_then(serde_json::Value::as_str)
                    .map_or_else(|| finish_result.text.clone(), str::to_owned);
                let terminal = StreamedReply {
                    deltas: if verdict.is_empty() { Vec::new() } else { vec![verdict] },
                    usage: final_reply.usage,
                    tool_calls: Vec::new(),
                    reasoning: final_reply.reasoning.clone(),
                };
                return self.complete_task(run, task, &terminal);
            }
            // `report_result` reports this task's result to its parent. The
            // engine persists it as the task's final assistant message and
            // completes the task here; parent runs consume the child result
            // from the queued-spawn path. A failed report is labelled, never
            // rewritten as a success.
            if let Some((report_plan, report_result)) = turn.executed.iter().find(|(plan, result)| {
                plan.tool == "report_result"
                    && result.outcome == hiverouter_contract::ToolOutcome::Ok
            }) {
                let summary = report_plan
                    .args
                    .get("summary")
                    .and_then(serde_json::Value::as_str)
                    .map_or_else(|| report_result.text.clone(), str::to_owned);
                let success = report_plan
                    .args
                    .get("success")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true);
                let report_text = if success {
                    summary
                } else {
                    format!("report_result success=false: {summary}")
                };
                let terminal = StreamedReply {
                    deltas: if report_text.is_empty() { Vec::new() } else { vec![report_text] },
                    usage: final_reply.usage,
                    tool_calls: Vec::new(),
                    reasoning: final_reply.reasoning.clone(),
                };
                return self.complete_task(run, task, &terminal);
            }
            transcript.push(ToolTurnTranscript {
                assistant_text: final_reply.text(),
                reasoning: final_reply.reasoning.clone(),
                results: turn.executed,
            });
            let (continuation_sink, continuation_streamed) = self.reply_delta_sink(run.id);
            let continuation = match tokio::select! {
                _ = token.cancelled() => {
                    self.fail_task(run, task, AgentState::Cancelled)?;
                    return Err(Refusal::ToolCancelled);
                }
                result = self.executor.continue_with_tools(
                    run.id,
                    task,
                    prompt,
                    token,
                    &continuation_sink,
                    &transcript,
                ) => result,
            } {
                Ok(reply) => reply,
                Err(refusal) => {
                    let state = if refusal == Refusal::ToolCancelled {
                        AgentState::Cancelled
                    } else {
                        AgentState::Failed
                    };
                    self.fail_task(run, task, state)?;
                    return Err(refusal);
                }
            };
            self.publish_returned_deltas(run.id, &continuation, &continuation_streamed);
            final_reply = continuation;
        }

        match self.complete_task(run, task, &final_reply) {
            Ok(text) => Ok(text),
            Err(refusal) => {
                let state = if refusal == Refusal::ToolCancelled {
                    AgentState::Cancelled
                } else {
                    AgentState::Failed
                };
                self.fail_task(run, task, state)?;
                Err(refusal)
            }
        }
    }

    /// Persists the final assistant reply once, publishes one `REPLY_DONE` and
    /// marks the task completed.
    ///
    /// Terminal-aware (M-002): a task that is no longer `Running` (killed,
    /// cancelled, failed, interrupted or already completed) is never overwritten
    /// and no `REPLY_DONE` is published for work the operator stopped. Returns
    /// the persisted text, or `None` when completion was suppressed.
    fn complete_task(
        &self,
        run: &Run,
        task: &TaskRecord,
        reply: &StreamedReply,
    ) -> Result<Option<String>, Refusal> {
        let _guard = self
            .task_state_lock
            .lock()
            .map_err(|_| Refusal::InternalError)?;
        let fresh = self.store.get_task(task.id)?;
        if is_terminal_task_state(fresh.state) {
            tracing::warn!(
                task = %task.id,
                state = %fresh.state,
                "refusing to complete a terminal task; no REPLY_DONE published"
            );
            return Ok(None);
        }
        if fresh.state != AgentState::Running {
            tracing::warn!(
                task = %task.id,
                state = %fresh.state,
                "task is not running; completion suppressed"
            );
            return Ok(None);
        }
        let text = reply.text();
        if !text.is_empty() {
            self.store
                .append_message(run.id, ChatRole::Assistant, &text)?;
        }
        self.publish(
            Event::ReplyDone(ReplyDone {
                run_id: run.id,
                usage: reply.usage,
            }),
            Some(run.id),
            None,
        );
        self.store
            .update_task_state(task.id, AgentState::Completed)?;
        self.store.record_task_result(run.id, task.id, &text)?;
        self.add_run_usage(run.id, reply.usage);
        if !text.trim().is_empty() && memory_text_is_safe(&text) {
            let bounded: String = text.chars().take(2_000).collect();
            self.store.record_agent_memory(
                &task.role,
                &format!("task:{}", task.id),
                &bounded,
                run.id,
                task.id,
            )?;
        }
        let _ = self.graph.set_state(task.id, AgentState::Completed);
        self.publish_node(run.id, task.id, AgentState::Completed);
        Ok(Some(text))
    }
}



#[cfg(test)]
mod prompt_context_tests {
    use super::*;

    #[test]
    fn prompt_includes_safe_scoped_facts_and_omits_secrets() {
        let parent = TaskId::new();
        let context = ScopedContext::new(Some(parent))
            .with_fact(parent, "style", "honeycomb")
            .with_fact(parent, "secret_scope", "do-not-send");
        let prompt = prompt_with_context("do the work", Some(&context));
        assert!(prompt.contains("do the work"));
        assert!(prompt.contains("honeycomb"));
        assert!(!prompt.contains("do-not-send"));
    }

    #[test]
    fn prompt_without_facts_is_unchanged() {
        assert_eq!(prompt_with_context("plain", None), "plain");
        assert_eq!(
            prompt_with_context("plain", Some(&ScopedContext::new(None))),
            "plain"
        );
    }
}
