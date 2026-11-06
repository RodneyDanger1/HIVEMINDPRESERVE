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
    NodeEvent, ReplyDelta, ReplyDone, RunCreated, RunStatusPayload,
};
use hiverouter_contract::{AgentState, Event, Refusal, RunId, TaskId, Timestamp, Usage};
use hiverouter_provider::tool_assembler::AssembledToolCall;
use hiverouter_tools::{
    execute_read_only, AgentTier, ExecutedTool, PermissionClass, ToolRegistry, WorkspaceJail,
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::bus::EventBus;
use crate::caps::Caps;
use crate::context::ContextStore;
use crate::db::{Database, ToolTurnRecord};
use crate::entities::{ChatRole, Run, TaskRecord};
use crate::graph::GraphAuthority;
use crate::planner::Plan;
use crate::prompts::TemplateRegistry;
use crate::store::RunStore;
use crate::tool_loop::{self, ToolCallPlan, ToolExecution};

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
    active_run: Mutex<Option<RunId>>,
}

impl std::fmt::Debug for RunEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunEngine")
            .field("active_run", &self.active_run())
            .field("caps", &self.caps)
            .field("tool_workspace", &self.tool_workspace.is_some())
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
            caps: Caps::default(),
            snapshots: None,
            tool_workspace: None,
            tool_registry: ToolRegistry::standard(),
            active_run: Mutex::new(None),
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
            caps,
            snapshots: None,
            tool_workspace: None,
            tool_registry: ToolRegistry::standard(),
            active_run: Mutex::new(None),
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

    fn publish_node(&self, run_id: RunId, task_id: TaskId, state: AgentState) {
        self.publish(
            Event::NodeState(NodeEvent { task_id, state }),
            Some(run_id),
            None,
        );
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
        let parent = self.store.get_task(parent_task_id)?;
        let run = self.store.get_run(parent.run_id)?;
        if run.status.is_terminal() {
            return Err(Refusal::RunTerminal);
        }
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
        self.graph
            .add_task(task.id, Some(parent_task_id))
            .map_err(|error| error)?;
        self.publish(
            Event::NodeSpawn(NodeEvent {
                task_id: task.id,
                state: AgentState::Queued,
            }),
            Some(task.run_id),
            None,
        );
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
        self.store
            .set_run_status(run_id, AgentState::Cancelled, None)?;
        self.publish(
            Event::RunStatus(RunStatusPayload {
                run_id,
                status: AgentState::Cancelled,
                error_code: None,
            }),
            Some(run_id),
            None,
        );
        Ok(())
    }

    /// Handles `KILL_WORKER`.
    ///
    /// # Errors
    /// `E_TASK_NOT_FOUND`, `E_TASK_TERMINAL`, `E_KILL_DENIED`.
    pub fn kill_worker(&self, task_id: TaskId) -> Result<(), Refusal> {
        let task = self.store.get_task(task_id)?;
        if task.state.is_terminal() {
            return Err(Refusal::TaskTerminal);
        }
        if task.state != AgentState::Running {
            return Err(Refusal::KillDenied);
        }
        self.store.update_task_state(task_id, AgentState::Killed)?;
        let _ = self.graph.set_state(task_id, AgentState::Killed);
        self.publish_node(task.run_id, task_id, AgentState::Killed);
        Ok(())
    }

    async fn dispatch(self: Arc<Self>, run: Run, token: CancellationToken) {
        let outcome = self.dispatch_inner(&run, &token).await;
        let (status, error_code) = match &outcome {
            Ok(()) => (AgentState::Completed, None),
            Err(refusal) if *refusal == Refusal::ToolCancelled => (AgentState::Cancelled, None),
            Err(refusal) => (AgentState::Failed, Some(refusal.code().to_owned())),
        };
        if let Err(error) = self
            .store
            .set_run_status(run.id, status, error_code.as_deref())
        {
            tracing::error!(code = error.code(), "failed to persist final run status");
        }
        self.publish(
            Event::RunStatus(RunStatusPayload {
                run_id: run.id,
                status,
                error_code,
            }),
            Some(run.id),
            None,
        );
        if let Some(store) = &self.snapshots {
            let revision = store
                .indexed_revisions(run.id)
                .map(|rows| u64::try_from(rows.len()).unwrap_or(0))
                .unwrap_or(0);
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
        let _ = self.set_active(None);
        self.tokens.remove(run.id);
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
        self.publish(
            Event::NodeSpawn(NodeEvent {
                task_id: root.id,
                state: AgentState::Queued,
            }),
            Some(run.id),
            None,
        );

        let mut tasks: HashMap<TaskId, TaskRecord> = HashMap::new();
        tasks.insert(root.id, root.clone());
        let mut spawn_seq: u64 = 1;
        for (index, node) in plan.nodes.iter().enumerate() {
            let agents = self.store.task_count(run.id)?;
            self.caps.check_spawn(node.depth, index, agents)?;
            spawn_seq = spawn_seq.saturating_add(1);
            let record = TaskRecord {
                id: node.node_id,
                run_id: run.id,
                parent_task_id: Some(root.id),
                role: node.role.clone(),
                depth: node.depth,
                spawn_seq,
                state: AgentState::Queued,
                rev: 0,
            };
            self.store.insert_task(&record)?;
            self.graph.add_task(record.id, Some(root.id))?;
            self.publish(
                Event::NodeSpawn(NodeEvent {
                    task_id: record.id,
                    state: AgentState::Queued,
                }),
                Some(run.id),
                None,
            );
            tasks.insert(record.id, record);
        }

        // Execute leaf tasks in topological order; reply chunks stream per task.
        for leaf_id in plan.leaf_order()? {
            if token.is_cancelled() {
                return Err(Refusal::ToolCancelled);
            }
            let task = tasks.get(&leaf_id).cloned().ok_or(Refusal::InternalError)?;
            self.execute_task(run, &task, token).await?;
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
    /// [`tool_loop::plan_call`] -> WorkerAgent tier/registry validation ->
    /// publish `TOOL_CALL` -> read-only execution through the configured
    /// [`WorkspaceJail`] -> publish `TOOL_RESULT` -> durable `tool_turns` row.
    /// Non-read-only tools, missing jail, invalid plans and exhausted budgets
    /// produce a named `TOOL_RESULT` refusal and execute nothing.
    fn execute_tool_turn(
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
                        let execution = match tool_loop::validate_plan(
                            &self.tool_registry,
                            AgentTier::WorkerAgent,
                            &plan,
                        ) {
                            Err(refusal) => ToolExecution::refused(
                                plan.tool_call_id.clone(),
                                refusal.code(),
                                format!(
                                    "tool {:?} refused by the WorkerAgent tier allowlist: {}",
                                    plan.tool,
                                    refusal.message()
                                ),
                            ),
                            Ok(()) => {
                                self.publish_tool_call(run, task, &plan);
                                self.execute_tool_plan(&plan)
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
                .increment_run_tool_counters(run.id, 1, calls_in_turn)
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
    fn fail_task(&self, run: &Run, task: &TaskRecord, state: AgentState) -> Result<(), Refusal> {
        self.store.update_task_state(task.id, state)?;
        let _ = self.graph.set_state(task.id, state);
        self.publish_node(run.id, task.id, state);
        Ok(())
    }

    /// Routes one validated plan to the single read-only executor.
    ///
    /// Read-only is a property of the frozen registry contract: a non-read tool
    /// is refused here and never reaches the executor or the jail. Without a
    /// configured jail a read call is refused as `E_FILE_TREE_UNAVAILABLE`.
    fn execute_tool_plan(&self, plan: &ToolCallPlan) -> ToolExecution {
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
            AgentTier::WorkerAgent,
            &plan.tool,
            &plan.args,
        ) {
            Ok(ExecutedTool::Ok { text }) => ToolExecution::ok(
                plan.tool_call_id.clone(),
                tool_loop::bounded_tool_text(&text),
            ),
            Ok(ExecutedTool::Refused { code, detail }) => {
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
        self.db
            .record_tool_turn(&record)
            .map_err(|error| error.refusal())
    }

    async fn execute_task(
        &self,
        run: &Run,
        task: &TaskRecord,
        token: &CancellationToken,
    ) -> Result<(), Refusal> {
        if let Err(refusal) = self.caps.check_calls(1) {
            self.fail_task(run, task, AgentState::Failed)?;
            return Err(refusal);
        }
        self.store
            .update_task_state(task.id, AgentState::Running)?;
        let _ = self.graph.set_state(task.id, AgentState::Running);
        self.publish_node(run.id, task.id, AgentState::Running);

        // Real-time delta publishing: a fresh sink per provider call keeps the
        // streaming/no-duplicate bookkeeping exact across follow-up tool turns.
        let (sink, streamed_deltas) = self.reply_delta_sink(run.id);

        let reply = match tokio::select! {
            _ = token.cancelled() => {
                self.fail_task(run, task, AgentState::Cancelled)?;
                return Err(Refusal::ToolCancelled);
            }
            result = self.executor.execute(run.id, task, &run.prompt, token, &sink) => result,
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
            let turn = match self.execute_tool_turn(run, task, &final_reply, token) {
                Ok(turn) => turn,
                Err(refusal) => {
                    // A persistence/budget error while the task is Running must
                    // not leave it non-terminal when the run fails; a cancelled
                    // turn keeps its cancelled state instead of becoming failed.
                    let state = if refusal == Refusal::ToolCancelled {
                        AgentState::Cancelled
                    } else {
                        AgentState::Failed
                    };
                    self.fail_task(run, task, state)?;
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
            transcript.push(ToolTurnTranscript {
                assistant_text: final_reply.text(),
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
                    &run.prompt,
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

        if let Err(refusal) = self.complete_task(run, task, &final_reply) {
            let state = if refusal == Refusal::ToolCancelled {
                AgentState::Cancelled
            } else {
                AgentState::Failed
            };
            self.fail_task(run, task, state)?;
            return Err(refusal);
        }
        Ok(())
    }

    /// Persists the final assistant reply once, publishes one `REPLY_DONE` and
    /// marks the task completed.
    fn complete_task(
        &self,
        run: &Run,
        task: &TaskRecord,
        reply: &StreamedReply,
    ) -> Result<(), Refusal> {
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
        let _ = self.graph.set_state(task.id, AgentState::Completed);
        self.publish_node(run.id, task.id, AgentState::Completed);
        Ok(())
    }
}

