//! Phase 4 acceptance: run lifecycle (plan -> dispatch -> streamed reply),
//! hard caps and the provider-error path (blueprint card commands
//! `run_lifecycle_plans_and_dispatches`, `caps_refused_by_name`, error path).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use hiverouter_contract::event::EventName;
use hiverouter_contract::{AgentState, Refusal, RunId, TaskId, Usage};
use hiverouter_core::bus::{EventBus, SequencedEvent};
use hiverouter_core::db::Database;
use hiverouter_core::execution::{ProviderExecutor, RunEngine, StreamedReply};
use hiverouter_core::graph::GraphAuthority;
use hiverouter_core::planner::{Plan, PlanNode};
use hiverouter_core::prompts::TemplateRegistry;
use hiverouter_core::store::RunStore;
use tokio::sync::broadcast::error::RecvError;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

fn leaf_plan(count: usize, depth: u8) -> Plan {
    let nodes = (0..count)
        .map(|_| PlanNode {
            node_id: TaskId::new(),
            role: "worker".to_owned(),
            deps: Vec::new(),
            depth,
        })
        .collect();
    Plan::new(nodes)
}

struct MockExecutor {
    plan: Plan,
    fail_on_execute: Option<usize>,
    slow: bool,
    executed: AtomicUsize,
}

impl MockExecutor {
    fn new(plan: Plan) -> Arc<Self> {
        Arc::new(Self {
            plan,
            fail_on_execute: None,
            slow: false,
            executed: AtomicUsize::new(0),
        })
    }

    fn failing(plan: Plan, fail_on_execute: usize) -> Arc<Self> {
        Arc::new(Self {
            plan,
            fail_on_execute: Some(fail_on_execute),
            slow: false,
            executed: AtomicUsize::new(0),
        })
    }

    fn slow(plan: Plan) -> Arc<Self> {
        Arc::new(Self {
            plan,
            fail_on_execute: None,
            slow: true,
            executed: AtomicUsize::new(0),
        })
    }
}

#[async_trait]
impl ProviderExecutor for MockExecutor {
    async fn plan(&self, _prompt: &str) -> Result<Plan, Refusal> {
        Ok(self.plan.clone())
    }

    async fn execute(
        &self,
        task: &hiverouter_core::entities::TaskRecord,
        cancel: &CancellationToken,
    ) -> Result<StreamedReply, Refusal> {
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
        Ok(StreamedReply {
            deltas: vec![format!("task {} ", task.id), "done".to_owned()],
            usage: Usage {
                prompt_tokens: Some(1),
                completion_tokens: Some(2),
                total_tokens: Some(3),
                ..Usage::default()
            },
        })
    }
}

struct TestEngine {
    engine: Arc<RunEngine>,
    bus: Arc<EventBus>,
    store: RunStore,
    _temp: tempfile::TempDir,
}

fn test_engine(executor: Arc<dyn ProviderExecutor>) -> TestEngine {
    let temp = tempfile::tempdir().expect("temp dir");
    let db = Arc::new(Database::open(&temp.path().join("hiverouter.db")).expect("db"));
    db.migrate().expect("migrate");
    let bus = Arc::new(EventBus::new());
    let engine = Arc::new(RunEngine::new(
        db.clone(),
        bus.clone(),
        Arc::new(GraphAuthority::new()),
        executor,
        Arc::new(TemplateRegistry::default()),
    ));
    TestEngine {
        engine,
        bus,
        store: RunStore::new(db),
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
    let final_status = loop {
        let event = wait_for(&mut receiver, |_| true).await;
        match event_name(&event) {
            "REPLY_DELTA" => reply_deltas += 1,
            "REPLY_DONE" => reply_done += 1,
            "NODE_SPAWN" => node_spawns += 1,
            "RUN_STATUS" => break event,
            _ => {}
        }
    };
    assert_eq!(final_status.envelope.payload["status"], "completed");
    assert_eq!(reply_done, 2, "one REPLY_DONE per executed leaf task");
    assert!(reply_deltas >= 2, "streamed reply chunks missing");
    assert_eq!(node_spawns, 3, "root + two planned nodes");

    let stored_run = harness.store.get_run(run.id).expect("run row");
    assert_eq!(stored_run.status, AgentState::Completed);
    assert_eq!(harness.store.task_count(run.id).expect("tasks"), 3);
    let messages = harness.store.messages(run.id).expect("messages");
    assert_eq!(messages.len(), 2, "user prompt + assistant reply");
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
    })
    .await;
    assert_eq!(
        failed.envelope.payload["error_code"],
        Refusal::FanoutLimit.code()
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
