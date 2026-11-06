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
use hiverouter_contract::{AgentState, Event, Refusal, RunId, TaskId, Usage};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::bus::EventBus;
use crate::caps::Caps;
use crate::context::ContextStore;
use crate::db::Database;
use crate::entities::{ChatRole, Run, TaskRecord};
use crate::graph::GraphAuthority;
use crate::planner::Plan;
use crate::prompts::TemplateRegistry;
use crate::store::RunStore;

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

    /// Releases any per-run provider state after the run reaches a terminal
    /// status. The default is a no-op for stateless/test executors.
    async fn finish_run(&self, _run_id: RunId) {}
}

/// One streamed assistant reply.
///
/// `deltas` holds the reply fragments. A streaming executor invokes the
/// [`DeltaSink`] with the same fragments and returns their concatenation here;
/// this field is the persistence source (`text()`), not a second event source.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StreamedReply {
    /// Reply fragments used to persist the assistant message (`text()`).
    pub deltas: Vec<String>,
    /// Provider usage for the reply; absent counters stay absent.
    pub usage: Usage,
}

impl StreamedReply {
    /// Full reply text.
    #[must_use]
    pub fn text(&self) -> String {
        self.deltas.concat()
    }
}

/// Production default when no provider is configured: refuses by name with
/// `E_KEY_MISSING` — never fabricates a reply.
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
    active_run: Mutex<Option<RunId>>,
}

impl std::fmt::Debug for RunEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunEngine")
            .field("active_run", &self.active_run())
            .field("caps", &self.caps)
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

    async fn execute_task(
        &self,
        run: &Run,
        task: &TaskRecord,
        token: &CancellationToken,
    ) -> Result<(), Refusal> {
        self.caps.check_calls(1)?;
        self.store
            .update_task_state(task.id, AgentState::Running)?;
        let _ = self.graph.set_state(task.id, AgentState::Running);
        self.publish_node(run.id, task.id, AgentState::Running);

        // Real-time delta publishing: the sink is invoked from the provider
        // read loop, so REPLY_DELTA events are emitted while chunks arrive.
        let streamed_deltas = Arc::new(AtomicUsize::new(0));
        let sink: DeltaSink = {
            let bus = Arc::clone(&self.bus);
            let run_id = run.id;
            let counter = Arc::clone(&streamed_deltas);
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

        let reply = match tokio::select! {
            _ = token.cancelled() => {
                self.store.update_task_state(task.id, AgentState::Cancelled)?;
                let _ = self.graph.set_state(task.id, AgentState::Cancelled);
                self.publish_node(run.id, task.id, AgentState::Cancelled);
                return Err(Refusal::ToolCancelled);
            }
            result = self.executor.execute(run.id, task, &run.prompt, token, &sink) => result,
        } {
            Ok(reply) => reply,
            Err(refusal) => {
                self.store.update_task_state(task.id, AgentState::Failed)?;
                let _ = self.graph.set_state(task.id, AgentState::Failed);
                self.publish_node(run.id, task.id, AgentState::Failed);
                return Err(refusal);
            }
        };

        // A streaming executor already published every fragment through the
        // sink; replaying `reply.deltas` would duplicate user-visible text.
        // Non-streaming executors (and tests) that never invoked the sink have
        // their returned fragments published here.
        if streamed_deltas.load(Ordering::SeqCst) == 0 {
            for delta in &reply.deltas {
                if delta.is_empty() {
                    continue;
                }
                self.publish(
                    Event::ReplyDelta(ReplyDelta {
                        run_id: run.id,
                        text_delta: delta.clone(),
                    }),
                    Some(run.id),
                    None,
                );
            }
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
        let _ = self.graph.set_state(task.id, AgentState::Completed);
        self.publish_node(run.id, task.id, AgentState::Completed);
        Ok(())
    }
}
