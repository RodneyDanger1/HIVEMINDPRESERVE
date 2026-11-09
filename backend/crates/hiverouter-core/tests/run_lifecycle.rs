//! Phase 4 acceptance: run lifecycle (plan -> dispatch -> streamed reply),
//! hard caps and the provider-error path (blueprint card commands
//! `run_lifecycle_plans_and_dispatches`, `caps_refused_by_name`, error path).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use hiverouter_contract::event::EventName;
use hiverouter_contract::{AgentState, Refusal, RunId, TaskId, Usage};
use hiverouter_core::bus::{EventBus, SequencedEvent};
use hiverouter_core::daemon::DaemonStore;
use hiverouter_core::db::Database;
use hiverouter_core::entities::TaskRecord;
use hiverouter_core::execution::{
    DeltaSink, ProviderExecutor, RunEngine, StreamedReply, UnconfiguredExecutor,
};
use hiverouter_core::graph::GraphAuthority;
use hiverouter_core::planner::{Plan, PlanNode};
use hiverouter_core::prompts::{Persona, PersonaTier, TemplateRegistry};
use hiverouter_core::remote::RemoteStore;
use hiverouter_core::store::RunStore;
use hiverouter_provider::tool_assembler::AssembledToolCall;
use serde_json::json;
use tokio::sync::broadcast::error::RecvError;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

fn leaf_plan(count: usize, depth: u8) -> Plan {
    let nodes = (0..count)
        .map(|_| PlanNode {
            node_id: TaskId::new(),
            // Strict hierarchy: a HyperAgent may only spawn SuperAgent branch
            // leads; a direct worker plan would be a named refusal.
            role: "super".to_owned(),
            deps: Vec::new(),
            depth,
            parent: None,
        })
        .collect();
    Plan::new(nodes)
}

fn tree_plan() -> Plan {
    let super_agent = PlanNode {
        node_id: TaskId::new(),
        role: "super".to_owned(),
        deps: Vec::new(),
        depth: 1,
        parent: None,
    };
    let agent = PlanNode {
        node_id: TaskId::new(),
        role: "worker".to_owned(),
        deps: Vec::new(),
        depth: 2,
        parent: Some(super_agent.node_id),
    };
    let subagent = PlanNode {
        node_id: TaskId::new(),
        role: "subagent".to_owned(),
        deps: Vec::new(),
        depth: 3,
        parent: Some(agent.node_id),
    };
    Plan::new(vec![super_agent, agent, subagent])
}

struct MockExecutor {
    plan: Plan,
    fail_on_execute: Option<usize>,
    slow: bool,
    executed: AtomicUsize,
    prompts: Mutex<Vec<String>>,
}

impl MockExecutor {
    fn new(plan: Plan) -> Arc<Self> {
        Arc::new(Self {
            plan,
            fail_on_execute: None,
            slow: false,
            executed: AtomicUsize::new(0),
            prompts: Mutex::new(Vec::new()),
        })
    }

    fn failing(plan: Plan, fail_on_execute: usize) -> Arc<Self> {
        Arc::new(Self {
            plan,
            fail_on_execute: Some(fail_on_execute),
            slow: false,
            executed: AtomicUsize::new(0),
            prompts: Mutex::new(Vec::new()),
        })
    }

    fn slow(plan: Plan) -> Arc<Self> {
        Arc::new(Self {
            plan,
            fail_on_execute: None,
            slow: true,
            executed: AtomicUsize::new(0),
            prompts: Mutex::new(Vec::new()),
        })
    }
}

#[async_trait]
impl ProviderExecutor for MockExecutor {
    async fn plan(
        &self,
        _run_id: hiverouter_contract::RunId,
        _prompt: &str,
        _cancel: &CancellationToken,
    ) -> Result<Plan, Refusal> {
        Ok(self.plan.clone())
    }

    async fn execute(
        &self,
        _run_id: hiverouter_contract::RunId,
        task: &hiverouter_core::entities::TaskRecord,
        prompt: &str,
        cancel: &CancellationToken,
        on_delta: &DeltaSink,
    ) -> Result<StreamedReply, Refusal> {
        self.prompts
            .lock()
            .expect("prompt capture")
            .push(prompt.to_owned());
        let index = self.executed.fetch_add(1, Ordering::SeqCst);
        if self.slow {
            tokio::select! {
                _ = cancel.cancelled() => return Err(Refusal::ToolCancelled),
                _ = tokio::time::sleep(Duration::from_secs(5)) => {}
            }
        }
        if self.fail_on_execute == Some(index) {
            return Err(Refusal::LlmRateLimit);
        }
        // Emulate a streaming provider: fragments reach the engine through the
        // sink before execute returns, and their concatenation is also returned
        // for persistence.
        let deltas = vec![format!("task {} ", task.id), "done".to_owned()];
        for delta in &deltas {
            on_delta(delta);
        }
        Ok(StreamedReply {
            deltas,
            usage: Usage {
                prompt_tokens: Some(1),
                completion_tokens: Some(2),
                total_tokens: Some(3),
                ..Usage::default()
            },
            tool_calls: Vec::new(),
            reasoning: String::new(),
        })
    }
}

struct RootRemoteExecutor {
    orchestrations: AtomicUsize,
    finalizes: AtomicUsize,
}

impl RootRemoteExecutor {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            orchestrations: AtomicUsize::new(0),
            finalizes: AtomicUsize::new(0),
        })
    }
}

#[async_trait]
impl ProviderExecutor for RootRemoteExecutor {
    async fn plan(
        &self,
        _run_id: RunId,
        _prompt: &str,
        _cancel: &CancellationToken,
    ) -> Result<Plan, Refusal> {
        Ok(leaf_plan(2, 1))
    }

    async fn execute(
        &self,
        _run_id: RunId,
        task: &TaskRecord,
        _prompt: &str,
        _cancel: &CancellationToken,
        on_delta: &DeltaSink,
    ) -> Result<StreamedReply, Refusal> {
        let text = format!("branch {}", task.id);
        on_delta(&text);
        Ok(StreamedReply {
            deltas: vec![text],
            usage: Usage::default(),
            tool_calls: Vec::new(),
            reasoning: String::new(),
        })
    }

    async fn orchestrate_root(
        &self,
        _run_id: RunId,
        _task: &TaskRecord,
        _prompt: &str,
        _cancel: &CancellationToken,
        _on_delta: &DeltaSink,
    ) -> Result<StreamedReply, Refusal> {
        if self.orchestrations.fetch_add(1, Ordering::SeqCst) == 0 {
            return Ok(StreamedReply {
                deltas: Vec::new(),
                usage: Usage::default(),
                tool_calls: vec![AssembledToolCall {
                    index: 0,
                    id: "call-remote".to_owned(),
                    name: "spawn_remote_agent".to_owned(),
                    arguments: json!({
                        "display_name": "Satellite relay",
                        "kind": "daemon_relay"
                    }),
                }],
                reasoning: String::new(),
            });
        }
        Ok(StreamedReply {
            deltas: Vec::new(),
            usage: Usage::default(),
            tool_calls: vec![AssembledToolCall {
                index: 0,
                id: "call-finish".to_owned(),
                name: "finish".to_owned(),
                arguments: json!({}),
            }],
            reasoning: String::new(),
        })
    }

    async fn finalize_root(
        &self,
        _run_id: RunId,
        _task: &TaskRecord,
        _prompt: &str,
        _cancel: &CancellationToken,
        on_delta: &DeltaSink,
    ) -> Result<StreamedReply, Refusal> {
        self.finalizes.fetch_add(1, Ordering::SeqCst);
        let text = "root answer with detached relay";
        on_delta(text);
        Ok(StreamedReply {
            deltas: vec![text.to_owned()],
            usage: Usage::default(),
            tool_calls: Vec::new(),
            reasoning: String::new(),
        })
    }
}

struct RootSuperExecutor {
    orchestrations: AtomicUsize,
}

impl RootSuperExecutor {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            orchestrations: AtomicUsize::new(0),
        })
    }
}

#[async_trait]
impl ProviderExecutor for RootSuperExecutor {
    async fn plan(
        &self,
        _run_id: RunId,
        _prompt: &str,
        _cancel: &CancellationToken,
    ) -> Result<Plan, Refusal> {
        Ok(leaf_plan(1, 1))
    }

    async fn execute(
        &self,
        _run_id: RunId,
        task: &TaskRecord,
        _prompt: &str,
        _cancel: &CancellationToken,
        on_delta: &DeltaSink,
    ) -> Result<StreamedReply, Refusal> {
        let text = format!("branch {}", task.id);
        on_delta(&text);
        Ok(StreamedReply {
            deltas: vec![text],
            usage: Usage::default(),
            tool_calls: Vec::new(),
            reasoning: String::new(),
        })
    }

    async fn orchestrate_root(
        &self,
        _run_id: RunId,
        _task: &TaskRecord,
        _prompt: &str,
        _cancel: &CancellationToken,
        _on_delta: &DeltaSink,
    ) -> Result<StreamedReply, Refusal> {
        let name = if self.orchestrations.fetch_add(1, Ordering::SeqCst) == 0 {
            "spawn_superagent"
        } else {
            "finish"
        };
        Ok(StreamedReply {
            deltas: Vec::new(),
            usage: Usage::default(),
            tool_calls: vec![AssembledToolCall {
                index: 0,
                id: format!("call-{name}"),
                name: name.to_owned(),
                arguments: json!({}),
            }],
            reasoning: String::new(),
        })
    }

    async fn finalize_root(
        &self,
        _run_id: RunId,
        _task: &TaskRecord,
        _prompt: &str,
        _cancel: &CancellationToken,
        on_delta: &DeltaSink,
    ) -> Result<StreamedReply, Refusal> {
        let text = "root answer";
        on_delta(text);
        Ok(StreamedReply {
            deltas: vec![text.to_owned()],
            usage: Usage::default(),
            tool_calls: Vec::new(),
            reasoning: String::new(),
        })
    }
}

struct RootToolExecutor {
    orchestrations: AtomicUsize,
    tool: &'static str,
    arguments: serde_json::Value,
}

impl RootToolExecutor {
    fn new(tool: &'static str, arguments: serde_json::Value) -> Arc<Self> {
        Arc::new(Self {
            orchestrations: AtomicUsize::new(0),
            tool,
            arguments,
        })
    }
}

#[async_trait]
impl ProviderExecutor for RootToolExecutor {
    async fn plan(
        &self,
        _run_id: RunId,
        _prompt: &str,
        _cancel: &CancellationToken,
    ) -> Result<Plan, Refusal> {
        Ok(leaf_plan(1, 1))
    }

    async fn execute(
        &self,
        _run_id: RunId,
        task: &TaskRecord,
        _prompt: &str,
        _cancel: &CancellationToken,
        on_delta: &DeltaSink,
    ) -> Result<StreamedReply, Refusal> {
        let text = format!("branch {}", task.id);
        on_delta(&text);
        Ok(StreamedReply {
            deltas: vec![text],
            usage: Usage::default(),
            tool_calls: Vec::new(),
            reasoning: String::new(),
        })
    }

    async fn orchestrate_root(
        &self,
        _run_id: RunId,
        _task: &TaskRecord,
        _prompt: &str,
        _cancel: &CancellationToken,
        _on_delta: &DeltaSink,
    ) -> Result<StreamedReply, Refusal> {
        let name = if self.orchestrations.fetch_add(1, Ordering::SeqCst) == 0 {
            self.tool
        } else {
            "finish"
        };
        Ok(StreamedReply {
            deltas: Vec::new(),
            usage: Usage::default(),
            tool_calls: vec![AssembledToolCall {
                index: 0,
                id: format!("call-{name}"),
                name: name.to_owned(),
                arguments: if name == "finish" {
                    json!({})
                } else {
                    self.arguments.clone()
                },
            }],
            reasoning: String::new(),
        })
    }

    async fn finalize_root(
        &self,
        _run_id: RunId,
        _task: &TaskRecord,
        _prompt: &str,
        _cancel: &CancellationToken,
        on_delta: &DeltaSink,
    ) -> Result<StreamedReply, Refusal> {
        let text = "root answer";
        on_delta(text);
        Ok(StreamedReply {
            deltas: vec![text.to_owned()],
            usage: Usage::default(),
            tool_calls: Vec::new(),
            reasoning: String::new(),
        })
    }
}

struct TestEngine {
    engine: Arc<RunEngine>,
    bus: Arc<EventBus>,
    store: RunStore,
    remote: Arc<RemoteStore>,
    daemons: Arc<DaemonStore>,
    _temp: tempfile::TempDir,
}

fn test_engine(executor: Arc<dyn ProviderExecutor>) -> TestEngine {
    test_engine_with_personas(executor, Vec::new())
}

fn test_engine_with_personas(
    executor: Arc<dyn ProviderExecutor>,
    personas: Vec<Persona>,
) -> TestEngine {
    let temp = tempfile::tempdir().expect("temp dir");
    let db = Arc::new(Database::open(&temp.path().join("hiverouter.db")).expect("db"));
    db.migrate().expect("migrate");
    let bus = Arc::new(EventBus::new());
    let remote = Arc::new(RemoteStore::new(db.clone()));
    let daemons = Arc::new(DaemonStore::new(db.clone()));
    let engine = Arc::new(
        RunEngine::new(
            db.clone(),
            bus.clone(),
            Arc::new(GraphAuthority::new()),
            executor,
            Arc::new(TemplateRegistry::default()),
        )
        .with_personas(personas)
        .with_remote_store(remote.clone())
        .with_daemon_store(daemons.clone())
        .with_root_orchestrator(),
    );
    TestEngine {
        engine,
        bus,
        store: RunStore::new(db),
        remote,
        daemons,
        _temp: temp,
    }
}

async fn wait_for(
    receiver: &mut tokio::sync::broadcast::Receiver<SequencedEvent>,
    predicate: impl Fn(&SequencedEvent) -> bool,
) -> SequencedEvent {
    for _ in 0..200 {
        let received = tokio::time::timeout(Duration::from_secs(10), receiver.recv()).await;
        match received {
            Ok(Ok(event)) => {
                if predicate(&event) {
                    return event;
                }
            }
            Ok(Err(RecvError::Lagged(_))) => continue,
            Ok(Err(RecvError::Closed)) => panic!("event bus closed"),
            Err(_) => panic!("timed out waiting for the expected event"),
        }
    }
    panic!("expected event did not arrive within 200 frames");
}

fn event_name(event: &SequencedEvent) -> &str {
    event.envelope.name.as_str()
}

#[tokio::test]
async fn unknown_spawn_role_is_refused_by_name() {
    let harness = test_engine(MockExecutor::new(leaf_plan(2, 1)));
    let run = harness
        .engine
        .store
        .create_run("p", "unknown role", AgentState::Running)
        .expect("run");
    let root = TaskRecord {
        id: TaskId::new(),
        run_id: run.id,
        parent_task_id: None,
        role: "planner".to_owned(),
        depth: 0,
        spawn_seq: 0,
        state: AgentState::Running,
        rev: 0,
    };
    harness.store.insert_task(&root).expect("root");
    let refused = harness.engine.spawn_sub_hive(root.id, "researcher");
    assert_eq!(refused, Err(Refusal::PersonaUnknown));
}

#[tokio::test]
async fn strict_hierarchy_refuses_direct_hyper_agent_and_super_subagent_edges() {
    let direct_agent = Plan::new(vec![PlanNode {
        node_id: TaskId::new(),
        role: "worker".to_owned(),
        deps: Vec::new(),
        depth: 1,
        parent: None,
    }]);
    let harness = test_engine(MockExecutor::new(direct_agent));
    let mut receiver = harness.bus.subscribe();
    harness
        .engine
        .submit_prompt("direct agent", "p", Uuid::now_v7())
        .expect("submit");
    let failed = wait_for(&mut receiver, |event| {
        event_name(event) == EventName::RunStatus.as_str()
            && event.envelope.payload["status"] != "running"
    })
    .await;
    assert_eq!(
        failed.envelope.payload["error_code"],
        Refusal::ToolNotAllowed.code(),
        "HyperAgent must not plan/spawn a direct Agent"
    );
}

#[tokio::test]
async fn hyperagent_can_spawn_a_detached_remote_without_a_task_row() {
    let harness = test_engine(RootRemoteExecutor::new());
    let mut receiver = harness.bus.subscribe();
    let run = harness
        .engine
        .submit_prompt("spawn a relay", "p", Uuid::now_v7())
        .expect("submit");
    let completed = wait_for(&mut receiver, |event| {
        event_name(event) == EventName::RunStatus.as_str()
            && event.envelope.payload["status"] == "completed"
    })
    .await;
    assert_eq!(completed.envelope.payload["status"], "completed");

    let remotes = harness.remote.list_remotes().expect("remotes");
    assert_eq!(remotes.len(), 1, "root tool must create exactly one remote");
    assert_eq!(remotes[0].owner_kind, "hive");
    assert!(
        remotes[0].created_by.starts_with("hyper:"),
        "root attribution must name the HyperAgent task: {}",
        remotes[0].created_by
    );
    assert_eq!(remotes[0].adapter_kind, "daemon_relay");
    let devices = harness
        .remote
        .list_devices(&remotes[0].remote_id)
        .expect("devices");
    assert_eq!(devices.len(), 1, "a spawned relay must be actuable");
    assert_eq!(devices[0].class, "switch");
    let tasks = harness.store.tasks_for_run(run.id).expect("tasks");
    assert_eq!(tasks.len(), 3, "root plus two branch tasks");
    assert!(tasks.iter().all(|task| task.id.to_string() != remotes[0].remote_id));
}

#[tokio::test]
async fn hyperagent_checkpoint_can_add_a_superagent_branch() {
    let harness = test_engine(RootSuperExecutor::new());
    let mut receiver = harness.bus.subscribe();
    let run = harness
        .engine
        .submit_prompt("add a branch", "p", Uuid::now_v7())
        .expect("submit");
    let completed = wait_for(&mut receiver, |event| {
        event_name(event) == EventName::RunStatus.as_str()
            && event.envelope.payload["status"] == "completed"
    })
    .await;
    assert_eq!(completed.envelope.payload["status"], "completed");
    let tasks = harness.store.tasks_for_run(run.id).expect("tasks");
    assert_eq!(tasks.len(), 3, "root + planned super + checkpoint super");
    let root = tasks
        .iter()
        .find(|task| task.parent_task_id.is_none())
        .expect("root task");
    assert_eq!(root.role, "planner");
    let supers: Vec<_> = tasks
        .iter()
        .filter(|task| task.role == "super")
        .collect();
    assert_eq!(supers.len(), 2, "both branch leads must be persisted");
    assert!(
        supers
            .iter()
            .all(|task| task.parent_task_id == Some(root.id)),
        "both branch leads must be strict HyperAgent children"
    );
}

#[tokio::test]
async fn superagent_hive_bank_and_branch_read_set_are_real() {
    let harness = test_engine(MockExecutor::new(tree_plan()));
    let mut receiver = harness.bus.subscribe();
    let run = harness
        .engine
        .submit_prompt("hive memory", "p", Uuid::now_v7())
        .expect("submit");
    wait_for(&mut receiver, |event| {
        event_name(event) == EventName::RunStatus.as_str()
            && event.envelope.payload["status"] == "completed"
    })
    .await;

    let tasks = harness.store.tasks_for_run(run.id).expect("tasks");
    let super_task = tasks
        .iter()
        .find(|task| task.role == "super")
        .expect("super");
    let worker = tasks
        .iter()
        .find(|task| task.role == "worker")
        .expect("worker");
    let subagent = tasks
        .iter()
        .find(|task| task.role == "subagent")
        .expect("subagent");
    let banks = harness.store.memory_banks_for_run(run.id).expect("banks");
    assert!(
        banks
            .iter()
            .any(|bank| bank.kind == "hive" && bank.owner_task_id.as_deref() == Some(&super_task.id.to_string())),
        "the SuperAgent must own a hive bank: {banks:?}"
    );
    assert!(
        banks
            .iter()
            .any(|bank| bank.kind == "branch" && bank.owner_task_id.as_deref() == Some(&worker.id.to_string())),
        "the Agent must own a branch bank: {banks:?}"
    );

    harness
        .store
        .record_agent_memory_scoped("super", "hive-key", "hive-value", run.id, super_task.id, "run")
        .expect("hive memory");
    harness
        .store
        .record_agent_memory_scoped("worker", "worker-key", "worker-value", run.id, worker.id, "run")
        .expect("branch memory");
    let read = harness
        .store
        .memory_read_set(run.id, subagent.id, Some(worker.id), 16)
        .expect("read set");
    let keys: Vec<&str> = read.iter().map(|entry| entry.key.as_str()).collect();
    assert!(keys.contains(&"hive-key"), "child must read its hive: {keys:?}");
    assert!(keys.contains(&"worker-key"), "child must read its parent branch: {keys:?}");
}

#[tokio::test]
async fn hyperagent_checkpoint_can_enroll_a_daemon_without_a_task_row() {
    let executor = RootToolExecutor::new(
        "spawn_daemon",
        json!({
            "display_name": "Hive relay",
            "kind": "relay",
            "interval_secs": 60
        }),
    );
    let harness = test_engine(executor);
    let mut receiver = harness.bus.subscribe();
    let run = harness
        .engine
        .submit_prompt("enroll a relay", "p", Uuid::now_v7())
        .expect("submit");
    wait_for(&mut receiver, |event| {
        event_name(event) == EventName::RunStatus.as_str()
            && event.envelope.payload["status"] == "completed"
    })
    .await;
    let daemons = harness.daemons.list().expect("daemons");
    assert_eq!(daemons.len(), 1);
    assert_eq!(daemons[0].kind, "relay");
    assert_eq!(daemons[0].owner_kind, "hive");
    assert!(daemons[0].created_by.starts_with("hyper:"));
    let tasks = harness.store.tasks_for_run(run.id).expect("tasks");
    assert!(
        tasks.iter().all(|task| !task.id.to_string().contains("daemon")),
        "daemons must never become task rows"
    );
}

#[tokio::test]
async fn keyless_submit_is_refused_before_any_run_row() {
    let harness = test_engine(Arc::new(UnconfiguredExecutor));
    let refused = harness
        .engine
        .submit_prompt("must not create a run", "project", Uuid::now_v7());
    assert_eq!(refused, Err(Refusal::KeyMissing));
    let runs = harness.store.list_runs(10).expect("runs");
    assert!(runs.is_empty(), "a keyless refusal must not create a run row");
    assert!(harness.store.latest_active_run().expect("active").is_none());
}

#[tokio::test]
async fn run_lifecycle_plans_and_dispatches() {
    let executor = MockExecutor::new(leaf_plan(2, 1));
    let harness = test_engine(executor);
    let mut receiver = harness.bus.subscribe();

    let run = harness
        .engine
        .submit_prompt("build the hive", "project-1", Uuid::now_v7())
        .expect("submit prompt");
    let created = wait_for(&mut receiver, |event| {
        event_name(event) == EventName::RunCreated.as_str()
    })
    .await;
    assert_eq!(created.envelope.payload["run_id"], run.id.to_string());
    assert_eq!(created.envelope.payload["status"], "queued");

    let mut reply_deltas = 0;
    let mut reply_done = 0;
    let mut node_spawns = 0;
    let mut run_metrics = 0;
    let final_status = loop {
        let event = wait_for(&mut receiver, |_| true).await;
        match event_name(&event) {
            "REPLY_DELTA" => reply_deltas += 1,
            "REPLY_DONE" => reply_done += 1,
            "NODE_SPAWN" => node_spawns += 1,
            "RUN_METRICS" => run_metrics += 1,
            "RUN_STATUS" if event.envelope.payload["status"] != "running" => break event,
            _ => {}
        }
    };
    assert_eq!(final_status.envelope.payload["status"], "completed");
    assert_eq!(reply_done, 3, "two leaves + the multi-node root synthesis");
    assert!(reply_deltas >= 2, "streamed reply chunks missing");
    assert_eq!(node_spawns, 3, "root + two planned nodes");
    assert!(run_metrics >= 1, "RUN_METRICS must have a producer");

    let stored_run = harness.store.get_run(run.id).expect("run row");
    assert_eq!(stored_run.status, AgentState::Completed);
    let mut usage = None;
    for _ in 0..50 {
        usage = harness.store.run_usage(run.id).expect("usage read");
        if usage.is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let usage = usage.expect("terminal usage row");
    assert_eq!(usage.prompt_tokens, Some(3));
    assert_eq!(usage.completion_tokens, Some(6));
    assert_eq!(usage.total_tokens, Some(9));
    assert_eq!(harness.store.task_count(run.id).expect("tasks"), 3);
    let messages = harness.store.messages(run.id).expect("messages");
    assert_eq!(
        messages.len(),
        4,
        "user prompt + two leaf replies + root synthesis"
    );
    assert_eq!(messages[0].role, hiverouter_core::entities::ChatRole::User);
    assert_eq!(
        messages[1].role,
        hiverouter_core::entities::ChatRole::Assistant
    );
    assert!(messages[1].text.contains("done"));
}

#[tokio::test]
async fn caps_refused_by_name() {
    // Depth 4 is over the depth-3 cap.
    let deep = test_engine(MockExecutor::new(leaf_plan(1, 4)));
    let mut receiver = deep.bus.subscribe();
    let _run = deep
        .engine
        .submit_prompt("too deep", "p", Uuid::now_v7())
        .expect("submit");
    let failed = wait_for(&mut receiver, |event| {
        event_name(event) == EventName::RunStatus.as_str()
            && event.envelope.payload["status"] != "running"
    })
    .await;
    assert_eq!(failed.envelope.payload["status"], "failed");
    assert_eq!(
        failed.envelope.payload["error_code"],
        Refusal::SpawnDepth.code()
    );

    // 9 siblings exceed the fan-out cap of 8.
    let wide = test_engine(MockExecutor::new(leaf_plan(9, 1)));
    let mut receiver = wide.bus.subscribe();
    let _run = wide
        .engine
        .submit_prompt("too wide", "p", Uuid::now_v7())
        .expect("submit");
    let failed = wait_for(&mut receiver, |event| {
        event_name(event) == EventName::RunStatus.as_str()
            && event.envelope.payload["status"] != "running"
    })
    .await;
    assert_eq!(
        failed.envelope.payload["error_code"],
        Refusal::FanoutLimit.code()
    );
}

#[tokio::test]
async fn provider_calls_are_counted_against_the_real_run_cap() {
    let mut harness = test_engine(MockExecutor::new(leaf_plan(1, 1)));
    // The planning call claims one; the first task call must then refuse.
    Arc::get_mut(&mut harness.engine)
        .expect("single owner before submit")
        .caps
        .max_calls = 1;
    let mut receiver = harness.bus.subscribe();
    let _run = harness
        .engine
        .submit_prompt("cap probe", "p", Uuid::now_v7())
        .expect("submit");
    let failed = wait_for(&mut receiver, |event| {
        event_name(event) == EventName::RunStatus.as_str()
            && event.envelope.payload["status"] != "running"
    })
    .await;
    assert_eq!(failed.envelope.payload["status"], "failed");
    assert_eq!(
        failed.envelope.payload["error_code"],
        Refusal::CallLimit.code(),
        "the cap must count planned + actual provider sends, not a literal 1"
    );
}

#[tokio::test]
async fn provider_error_preserves_partial_state() {
    let executor = MockExecutor::failing(leaf_plan(2, 1), 1);
    let harness = test_engine(executor);
    let mut receiver = harness.bus.subscribe();
    let run = harness
        .engine
        .submit_prompt("two tasks", "p", Uuid::now_v7())
        .expect("submit");
    let failed = wait_for(&mut receiver, |event| {
        event_name(event) == EventName::RunStatus.as_str()
            && event.envelope.payload["status"] != "running"
    })
    .await;
    assert_eq!(failed.envelope.payload["status"], "failed");
    assert_eq!(
        failed.envelope.payload["error_code"],
        Refusal::LlmRateLimit.code()
    );

    let stored = harness.store.get_run(run.id).expect("run");
    assert_eq!(stored.status, AgentState::Failed);
    assert_eq!(
        stored.error_code.as_deref(),
        Some(Refusal::LlmRateLimit.code())
    );
    // Partial state is preserved: root + two tasks, first leaf completed.
    let tasks = harness.store.tasks_for_run(run.id).expect("tasks");
    assert_eq!(tasks.len(), 3);
    assert!(tasks
        .iter()
        .any(|task| task.state == AgentState::Completed));
    assert!(tasks.iter().any(|task| task.state == AgentState::Failed));
    // The completed leaf's reply stays in the chat store; nothing was faked.
    let messages = harness.store.messages(run.id).expect("messages");
    assert_eq!(messages.len(), 2, "user + one completed assistant reply");
}

#[tokio::test]
async fn cancel_run_stops_the_run() {
    let executor = MockExecutor::slow(leaf_plan(1, 1));
    let harness = test_engine(executor);
    let mut receiver = harness.bus.subscribe();
    let run = harness
        .engine
        .submit_prompt("slow run", "p", Uuid::now_v7())
        .expect("submit");
    tokio::time::sleep(Duration::from_millis(200)).await;
    harness.engine.cancel_run(run.id).expect("cancel");
    let status = wait_for(&mut receiver, |event| {
        event_name(event) == EventName::RunStatus.as_str()
            && event.envelope.payload["run_id"] == run.id.to_string()
            && event.envelope.payload["status"] == "cancelled"
    })
    .await;
    assert_eq!(status.envelope.payload["status"], "cancelled");
    let stored = harness.store.get_run(run.id).expect("run");
    assert_eq!(stored.status, AgentState::Cancelled);
    // Cancelling an already-terminal run is refused by name.
    assert_eq!(harness.engine.cancel_run(run.id), Err(Refusal::RunTerminal));
}

#[tokio::test]
async fn second_prompt_while_active_is_refused_by_name() {
    let executor = MockExecutor::slow(leaf_plan(1, 1));
    let harness = test_engine(executor);
    let run = harness
        .engine
        .submit_prompt("first", "p", Uuid::now_v7())
        .expect("first submit");
    let second = harness
        .engine
        .submit_prompt("second", "p", Uuid::now_v7());
    assert_eq!(second, Err(Refusal::RunActive));
    let run_id: RunId = run.id;
    harness.engine.cancel_run(run_id).expect("cleanup cancel");
}

#[tokio::test]
async fn spawn_sub_hive_unknown_parent_is_refused() {
    let harness = test_engine(MockExecutor::new(leaf_plan(1, 1)));
    assert_eq!(
        harness.engine.spawn_sub_hive(TaskId::new(), "worker"),
        Err(Refusal::TaskNotFound)
    );
}

#[tokio::test]
async fn planned_parent_chain_is_persisted_not_flattened_to_root() {
    let executor = MockExecutor::new(tree_plan());
    let harness = test_engine(executor.clone());
    let mut receiver = harness.bus.subscribe();
    let run = harness
        .engine
        .submit_prompt("build a hierarchy", "p", Uuid::now_v7())
        .expect("submit");
    let mut reply_done = 0;
    let final_status = loop {
        let event = wait_for(&mut receiver, |_| true).await;
        if event_name(&event) == "REPLY_DONE" {
            reply_done += 1;
        }
        if event_name(&event) == "RUN_STATUS" {
            let status = event.envelope.payload["status"].as_str().unwrap_or_default();
            if status != "queued" && status != "planning" && status != "running" {
                break status.to_owned();
            }
        }
    };
    assert_eq!(final_status, "completed");
    assert_eq!(reply_done, 4, "leaf + two aggregators + HyperAgent root");
    let tasks = harness.store.tasks_for_run(run.id).expect("tasks");
    let root = tasks.iter().find(|task| task.depth == 0).expect("root");
    let super_agent = tasks.iter().find(|task| task.depth == 1).expect("super");
    let agent = tasks.iter().find(|task| task.depth == 2).expect("agent");
    let subagent = tasks.iter().find(|task| task.depth == 3).expect("subagent");
    assert_eq!(super_agent.parent_task_id, Some(root.id));
    assert_eq!(agent.parent_task_id, Some(super_agent.id));
    assert_eq!(subagent.parent_task_id, Some(agent.id));
    for task in &tasks {
        let context = harness
            .engine
            .contexts
            .get(task.id)
            .expect("context lookup")
            .expect("context seeded at task creation");
        assert_eq!(context.parent_task_id, task.parent_task_id);
        assert!(
            context.facts.contains_key("memory:run:prompt"),
            "every branch must recall the engine-owned run bank"
        );
    }
    for task in [root, super_agent, agent, subagent] {
        assert!(
            harness.store.task_result(task.id).expect("result").is_some(),
            "every executed task must persist its result"
        );
    }
    let prompts = executor.prompts.lock().expect("prompts");
    assert_eq!(prompts.len(), 4, "leaf + two aggregators + HyperAgent root");
    assert!(
        prompts.iter().any(|prompt| prompt.contains("Aggregate the completed child results")),
        "an aggregator must receive child results, not just the original prompt: {prompts:?}"
    );
    assert!(
        prompts.iter().any(|prompt| prompt.contains("You are the HyperAgent orchestrator")),
        "the root must receive the synthesis prompt: {prompts:?}"
    );
    let messages = harness.store.messages(run.id).expect("messages");
    assert_eq!(
        messages.len(),
        5,
        "user + leaf + two aggregators + HyperAgent root"
    );
    let final_message = messages.last().expect("root answer");
    assert_eq!(
        final_message.role,
        hiverouter_core::entities::ChatRole::Assistant
    );
    assert!(
        final_message.text.contains(&root.id.to_string()),
        "the final assistant message must be the root's answer"
    );
}

#[tokio::test]
async fn boot_persona_allowlist_is_seeded_on_worker_contexts() {
    let worker = Persona {
        id: "worker".to_owned(),
        display_name: "Worker".to_owned(),
        tier: PersonaTier::Worker,
        template_id: "planner".to_owned(),
        allowed_tool_ids: vec!["file_read".to_owned()],
        output_schema_id: None,
        refusal_policy: "strict".to_owned(),
        max_children: None,
        max_depth: None,
    };
    let harness = test_engine_with_personas(MockExecutor::new(tree_plan()), vec![worker]);
    let mut receiver = harness.bus.subscribe();
    let run = harness
        .engine
        .submit_prompt("persona check", "p", Uuid::now_v7())
        .expect("submit");
    loop {
        let event = wait_for(&mut receiver, |_| true).await;
        if event_name(&event) == "RUN_STATUS" {
            let status = event.envelope.payload["status"].as_str().unwrap_or_default();
            if status != "queued" && status != "planning" && status != "running" {
                break;
            }
        }
    }
    let tasks = harness.store.tasks_for_run(run.id).expect("tasks");
    let worker_tasks: Vec<_> = tasks.iter().filter(|task| task.role == "worker").collect();
    assert!(!worker_tasks.is_empty());
    for task in worker_tasks {
        let context = harness
            .engine
            .contexts
            .get(task.id)
            .expect("context lookup")
            .expect("context");
        assert_eq!(context.allowed_tool_ids, vec!["file_read".to_owned()]);
    }
}

#[tokio::test]
async fn durable_role_memory_is_not_recalled_across_runs() {
    let harness = test_engine(MockExecutor::new(tree_plan()));
    harness
        .store
        .record_agent_memory(
            "worker",
            "task:old",
            "remembered fact",
            RunId::new(),
            TaskId::new(),
        )
        .expect("memory");
    let mut receiver = harness.bus.subscribe();
    let run = harness
        .engine
        .submit_prompt("recall", "p", Uuid::now_v7())
        .expect("submit");
    loop {
        let event = wait_for(&mut receiver, |_| true).await;
        if event_name(&event) == "RUN_STATUS" {
            let status = event.envelope.payload["status"].as_str().unwrap_or_default();
            if status != "queued" && status != "planning" && status != "running" {
                break;
            }
        }
    }
    let tasks = harness.store.tasks_for_run(run.id).expect("tasks");
    let worker = tasks.iter().find(|task| task.role == "worker").expect("worker");
    let context = harness
        .engine
        .contexts
        .get(worker.id)
        .expect("context lookup")
        .expect("context");
    assert_eq!(
        context.facts.get("memory:task:old"),
        None,
        "a foreign run's memory must never be injected into this hive"
    );
    assert!(
        context.facts.keys().any(|key| key.starts_with("roster:")),
        "a running task must receive the bounded peer roster"
    );
}

#[tokio::test]
async fn worker_spawn_is_restricted_to_subagents_not_superagents() {
    let harness = test_engine(MockExecutor::new(leaf_plan(1, 1)));
    let run = harness
        .store
        .create_run("p", "spawn policy", AgentState::Running)
        .expect("run");
    let root = hiverouter_core::entities::TaskRecord {
        id: TaskId::new(),
        run_id: run.id,
        parent_task_id: None,
        role: "planner".to_owned(),
        depth: 0,
        spawn_seq: 1,
        state: AgentState::Running,
        rev: 0,
    };
    let worker = hiverouter_core::entities::TaskRecord {
        id: TaskId::new(),
        run_id: run.id,
        parent_task_id: Some(root.id),
        role: "worker".to_owned(),
        depth: 1,
        spawn_seq: 2,
        state: AgentState::Running,
        rev: 0,
    };
    harness.store.insert_task(&root).expect("root");
    harness.store.insert_task(&worker).expect("worker");
    harness.engine.graph.add_task(root.id, None).expect("root graph");
    harness
        .engine
        .graph
        .add_task(worker.id, Some(root.id))
        .expect("worker graph");

    assert_eq!(
        harness.engine.spawn_sub_hive(worker.id, "super"),
        Err(Refusal::ToolNotAllowed),
        "a Worker must not escalate itself to SuperAgent"
    );
    let child = harness
        .engine
        .spawn_sub_hive(worker.id, "subagent")
        .expect("worker may spawn a subagent");
    assert_eq!(child.depth, 2);
    assert_eq!(child.parent_task_id, Some(worker.id));
}

#[tokio::test]
async fn profile_max_children_tightens_the_global_fanout_cap() {
    let worker = Persona {
        id: "worker".to_owned(),
        display_name: "Worker".to_owned(),
        tier: PersonaTier::Worker,
        template_id: "planner".to_owned(),
        allowed_tool_ids: vec!["spawn_agent".to_owned()],
        output_schema_id: None,
        refusal_policy: "strict".to_owned(),
        max_children: Some(1),
        max_depth: Some(2),
    };
    let harness = test_engine_with_personas(MockExecutor::new(leaf_plan(1, 1)), vec![worker]);
    let run = harness
        .store
        .create_run("p", "profile caps", AgentState::Running)
        .expect("run");
    let root = hiverouter_core::entities::TaskRecord {
        id: TaskId::new(),
        run_id: run.id,
        parent_task_id: None,
        role: "planner".to_owned(),
        depth: 0,
        spawn_seq: 1,
        state: AgentState::Running,
        rev: 0,
    };
    let worker_task = hiverouter_core::entities::TaskRecord {
        id: TaskId::new(),
        run_id: run.id,
        parent_task_id: Some(root.id),
        role: "worker".to_owned(),
        depth: 1,
        spawn_seq: 2,
        state: AgentState::Running,
        rev: 0,
    };
    harness.store.insert_task(&root).expect("root");
    harness.store.insert_task(&worker_task).expect("worker");
    harness.engine.graph.add_task(root.id, None).expect("root graph");
    harness
        .engine
        .graph
        .add_task(worker_task.id, Some(root.id))
        .expect("worker graph");
    harness
        .engine
        .spawn_sub_hive(worker_task.id, "subagent")
        .expect("first child within profile cap");
    assert_eq!(
        harness.engine.spawn_sub_hive(worker_task.id, "subagent"),
        Err(Refusal::FanoutLimit),
        "the profile's max_children=1 must refuse the second child"
    );
}

#[tokio::test]
async fn streamed_deltas_are_published_before_done_and_not_duplicated() {
    let harness = test_engine(MockExecutor::new(leaf_plan(1, 1)));
    let mut receiver = harness.bus.subscribe();
    let run = harness
        .engine
        .submit_prompt("stream one task", "p", Uuid::now_v7())
        .expect("submit");

    let mut fragments: Vec<String> = Vec::new();
    loop {
        let event = wait_for(&mut receiver, |_| true).await;
        match event_name(&event) {
            "REPLY_DELTA" => fragments.push(
                event.envelope.payload["text_delta"]
                    .as_str()
                    .expect("text_delta string")
                    .to_owned(),
            ),
            "REPLY_DONE" => break,
            _ => {}
        }
    }
    assert_eq!(
        fragments.len(),
        2,
        "one REPLY_DELTA per sink fragment; the returned StreamedReply deltas are not replayed"
    );
    assert!(fragments[0].starts_with("task "), "first fragment: {}", fragments[0]);
    assert_eq!(fragments[1], "done");

    let messages = harness.store.messages(run.id).expect("messages");
    let assistant = messages
        .iter()
        .find(|message| message.role == hiverouter_core::entities::ChatRole::Assistant)
        .expect("assistant message");
    assert_eq!(
        assistant.text,
        fragments.concat(),
        "persisted text is the concatenation of the streamed fragments"
    );
}
