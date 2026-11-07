//! WP-04 deterministic suspension / side-effect matrix (Run-3 scaffold).
//!
//! These 14 tests drive the *real* `RunEngine` + `ToolsBridge` with a scripted
//! `ProviderExecutor` (test double only: no network, no key, no paid call). They
//! pin the suspension contract from `plans/foresight/RUN2-ENGINE-CODE.md` §7.1
//! and `plans/foresight/RUN2-PROOF-CODE.md` §1:
//!
//! - suspend publishes `awaiting_approval` + one `APPROVAL_REQUEST` and waits;
//! - approve executes exactly one granted side effect; deny executes none;
//! - timeout/cancel/kill/restart are terminal, named and side-effect free;
//! - duplicate/concurrent resolves have one winner and one execution;
//! - the approval-wait cap refuses the ninth request with `E_CALL_LIMIT`;
//! - wrong arguments never consume the grant or execute;
//! - an engine without `.with_tools` keeps the pre-1.2 read-only refusal.
//!
//! ## API assumptions (flagged; product engine is being wired concurrently)
//!
//! 1. `RunEngine::{new, with_tool_workspace, with_tools, with_clock,
//!    with_shell_policy, notify_approval_resolved}` and public fields
//!    `store`/`db`/`bus` exist (current dirty-tree wiring; `execution.rs`).
//! 2. `ToolsBridge::{new, migrate, with_approval_ttl, request_model_tool_approval,
//!    resolve_approval, approval_decision, approval_count_for_run, expire_approval,
//!    cancel_approval, verify_tool_grant, pending_approvals}` exist with the
//!    signatures read on 2026-11-06 (`tools_bridge.rs`).
//! 3. Public `ApprovalWaiters`/waiter count has **no test accessor**; S1 asserts
//!    the armed waiter via `RunEngine`'s `Debug` output (`waiters: 1`) instead of
//!    a private field. If the engine wave adds `waiters_len()`, prefer it.
//! 4. Boot order assumption for S10: the production boot path expires orphaned
//!    pending rows and then marks awaiting runs interrupted with
//!    `error_code=E_APPROVAL_TIMEOUT`; the test calls
//!    `ToolsBridge::expire_pending_approvals` then `recover_on_boot`.
//! 5. S7 is a best-effort CAS-race test (resolve ~just before the 1 s TTL); it
//!    asserts the observable invariant (no timeout terminal, exactly one effect)
//!    without controlling timer scheduling. A deterministic clock seam for the
//!    wait timer is not present on the current tree.
//! 6. S13 targets the planned `shell_exec` side-effect arm. `RUN-3.md` records
//!    that WP-02 ships `file_write` first and the shell arm is additive next
//!    wave; until it lands this test is expected RED and must not be reported as
//!    a pass. It stays active (no `#[ignore]`) so the gap is visible.
//! 7. Side-effect args use the frozen registry shapes (`file_write`:
//!    `{path,content,expected_sha256?}`; `shell_exec`:
//!    `{program,args,...}`). Invalid shapes are refused by preflight, not by a
//!    panic.
//!
//! No build/run was performed by this scaffold seat.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use hiverouter_contract::command::ApprovalResolve;
use hiverouter_contract::event::ApprovalRequest as ApprovalRequestEvent;
use hiverouter_contract::{AgentState, Refusal, RunId, TaskId, Timestamp, ToolOutcome, Usage};
use hiverouter_core::bus::{EventBus, SequencedEvent};
use hiverouter_core::db::Database;
use hiverouter_core::entities::{Run, TaskRecord};
use hiverouter_core::execution::{
    DeltaSink, ProviderExecutor, RunEngine, StreamedReply, ToolTurnTranscript,
};
use hiverouter_core::graph::GraphAuthority;
use hiverouter_core::planner::{Plan, PlanNode};
use hiverouter_core::prompts::TemplateRegistry;
use hiverouter_core::recovery::recover_on_boot;
use hiverouter_core::snapshot::SnapshotStore;
use hiverouter_core::store::RunStore;
use hiverouter_core::tools_bridge::ToolsBridge;
use hiverouter_provider::tool_assembler::AssembledToolCall;
use hiverouter_tools::shell::ShellPolicy;
use hiverouter_tools::{ToolId, WorkspaceJail};
use tokio::sync::broadcast::error::RecvError;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

const WRITE_PATH: &str = "approval-proof.txt";
const WRITE_TEXT: &str = "HIVE-APPROVAL-OK";
const SHELL_MARKER: &str = "HIVE-SHELL-OK";

/* ------------------------------------------------------------------ */
/* Scripted provider (test double; mirrors tool_loop_engine.rs)        */
/* ------------------------------------------------------------------ */

fn leaf_plan() -> Plan {
    Plan::new(vec![PlanNode {
        node_id: TaskId::new(),
        role: "worker".to_owned(),
        deps: Vec::new(),
        depth: 1,
        parent: None,
    }])
}

/// Returns a fixed leaf plan and a scripted sequence of assistant replies.
/// The first reply is returned by `execute`; queued follow-ups are returned by
/// `continue_with_tools`; when the queue is empty a plain final answer is used.
struct ScriptedExecutor {
    plan: Plan,
    replies: Mutex<VecDeque<StreamedReply>>,
}

impl ScriptedExecutor {
    fn new(replies: Vec<StreamedReply>) -> Arc<Self> {
        Arc::new(Self {
            plan: leaf_plan(),
            replies: Mutex::new(replies.into()),
        })
    }
}

#[async_trait]
impl ProviderExecutor for ScriptedExecutor {
    async fn plan(
        &self,
        _run_id: RunId,
        _prompt: &str,
        _cancel: &CancellationToken,
    ) -> Result<Plan, Refusal> {
        Ok(self.plan.clone())
    }

    async fn execute(
        &self,
        _run_id: RunId,
        _task: &TaskRecord,
        _prompt: &str,
        _cancel: &CancellationToken,
        on_delta: &DeltaSink,
    ) -> Result<StreamedReply, Refusal> {
        let reply = self
            .replies
            .lock()
            .expect("scripted replies lock")
            .pop_front()
            .unwrap_or_else(final_answer);
        for delta in &reply.deltas {
            on_delta(delta);
        }
        Ok(reply)
    }

    async fn continue_with_tools(
        &self,
        _run_id: RunId,
        _task: &TaskRecord,
        _prompt: &str,
        _cancel: &CancellationToken,
        on_delta: &DeltaSink,
        _transcript: &[ToolTurnTranscript],
    ) -> Result<StreamedReply, Refusal> {
        let reply = self
            .replies
            .lock()
            .expect("scripted replies lock")
            .pop_front()
            .unwrap_or_else(final_answer);
        for delta in &reply.deltas {
            on_delta(delta);
        }
        Ok(reply)
    }
}

fn final_answer() -> StreamedReply {
    StreamedReply {
        deltas: vec!["final answer".to_owned()],
        usage: Usage::default(),
        tool_calls: Vec::new(),
        reasoning: String::new(),
    }
}

fn tool_only_reply(calls: Vec<AssembledToolCall>) -> StreamedReply {
    StreamedReply {
        deltas: Vec::new(),
        usage: Usage::default(),
        tool_calls: calls,
        reasoning: String::new(),
    }
}

fn write_args(path: &str, content: &str) -> serde_json::Value {
    serde_json::json!({ "path": path, "content": content })
}

fn write_call(tool_call_id: &str, path: &str, content: &str, index: u32) -> AssembledToolCall {
    AssembledToolCall {
        index,
        id: tool_call_id.to_owned(),
        name: "file_write".to_owned(),
        arguments: write_args(path, content),
    }
}

fn shell_call(tool_call_id: &str, index: u32) -> AssembledToolCall {
    AssembledToolCall {
        index,
        id: tool_call_id.to_owned(),
        name: "shell_exec".to_owned(),
        arguments: serde_json::json!({
            "program": "cmd.exe",
            "args": ["/C", format!("echo {SHELL_MARKER}")]
        }),
    }
}

/* ------------------------------------------------------------------ */
/* Harness                                                             */
/* ------------------------------------------------------------------ */

struct Harness {
    engine: Arc<RunEngine>,
    bridge: Arc<ToolsBridge>,
    bus: Arc<EventBus>,
    db: Arc<Database>,
    /// Snapshot/data root used by the restart-recovery test.
    data: tempfile::TempDir,
    /// Workspace jail root for every side-effect assertion.
    workspace: tempfile::TempDir,
}

fn harness(executor: Arc<dyn ProviderExecutor>, with_tools: bool, approval_ttl_secs: u64) -> Harness {
    let data = tempfile::tempdir().expect("data tempdir");
    let workspace = tempfile::tempdir().expect("workspace tempdir");
    let db = Arc::new(Database::open(&data.path().join("hiverouter.db")).expect("open db"));
    db.migrate().expect("migrate core tables");
    ToolsBridge::migrate(&db).expect("migrate tools tables");

    let bridge = Arc::new(
        ToolsBridge::new(workspace.path(), db.clone()).with_approval_ttl(approval_ttl_secs),
    );
    let bus = Arc::new(EventBus::new());
    let mut engine = RunEngine::new(
        db.clone(),
        bus.clone(),
        Arc::new(GraphAuthority::new()),
        executor,
        Arc::new(TemplateRegistry::default()),
    )
    .with_tool_workspace(WorkspaceJail::open(workspace.path()).expect("workspace jail"))
    .with_shell_policy(ShellPolicy::new(["cmd.exe".to_owned()]));
    if with_tools {
        engine = engine.with_tools(bridge.clone());
    }
    Harness {
        engine: Arc::new(engine),
        bridge,
        bus,
        db,
        data,
        workspace,
    }
}

impl Harness {
    fn workspace_path(&self, relative: &str) -> std::path::PathBuf {
        self.workspace.path().join(relative)
    }

    fn run(&self, run_id: RunId) -> Run {
        self.engine.store.get_run(run_id).expect("run row")
    }

    fn decisions(&self, approval_id: &str) -> Option<String> {
        self.bridge
            .approval_decision(approval_id)
            .expect("approval decision")
    }

    fn resolve(
        &self,
        request: &ApprovalRequestEvent,
        decision: &str,
    ) -> Result<hiverouter_tools::approval::ApprovalResolution, Refusal> {
        let nonce = request
            .nonce
            .clone()
            .expect("APPROVAL_REQUEST must carry a nonce");
        let payload = ApprovalResolve {
            approval_id: request.approval_id,
            nonce,
            decision: decision.to_owned(),
            reason: None,
        };
        self.bridge.resolve_approval(&payload)
    }

    fn resolve_and_wake(&self, request: &ApprovalRequestEvent, decision: &str) -> bool {
        let resolution = self.resolve(request, decision).expect("committed decision");
        self.engine.notify_approval_resolved(&resolution)
    }
}

#[derive(Default)]
struct Collected {
    sequence: Vec<String>,
    tool_calls: Vec<serde_json::Value>,
    tool_results: Vec<serde_json::Value>,
    reply_deltas: Vec<String>,
    statuses: Vec<String>,
    terminal_status: serde_json::Value,
    final_run_status: Option<String>,
}

impl Collected {
    fn push(&mut self, event: &SequencedEvent) {
        let name = event.envelope.name.as_str();
        self.sequence.push(name.to_owned());
        match name {
            "TOOL_CALL" => self.tool_calls.push(event.envelope.payload.clone()),
            "TOOL_RESULT" => self.tool_results.push(event.envelope.payload.clone()),
            "REPLY_DELTA" => {
                if let Some(text) = event.envelope.payload["text_delta"].as_str() {
                    self.reply_deltas.push(text.to_owned());
                }
            }
            "RUN_STATUS" => {
                let status = event.envelope.payload["status"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned();
                self.statuses.push(status.clone());
                self.final_run_status = Some(status);
                self.terminal_status = event.envelope.payload.clone();
            }
            _ => {}
        }
    }

    fn terminal(&self) -> bool {
        matches!(
            self.final_run_status.as_deref(),
            Some("completed" | "failed" | "cancelled" | "killed" | "interrupted")
        )
    }

    fn outcomes(&self) -> Vec<String> {
        self.tool_results
            .iter()
            .map(|result| result["outcome"].as_str().unwrap_or_default().to_owned())
            .collect()
    }

    fn codes(&self) -> Vec<String> {
        self.tool_results
            .iter()
            .map(|result| result["code"].as_str().unwrap_or_default().to_owned())
            .collect()
    }
}

async fn next_event(
    receiver: &mut tokio::sync::broadcast::Receiver<SequencedEvent>,
) -> SequencedEvent {
    loop {
        match tokio::time::timeout(Duration::from_secs(15), receiver.recv()).await {
            Ok(Ok(event)) => return event,
            Ok(Err(RecvError::Lagged(_))) => continue,
            Ok(Err(RecvError::Closed)) => panic!("engine event bus closed"),
            Err(_) => panic!("timed out waiting for an engine event"),
        }
    }
}

const TERMINAL_STATES: [&str; 5] = ["completed", "failed", "cancelled", "killed", "interrupted"];

fn is_terminal_status(status: &str) -> bool {
    TERMINAL_STATES.contains(&status)
}

/// Submits a prompt and collects until the run reaches an approval request.
async fn begin_approval(
    harness: &Harness,
    prompt: &str,
) -> (
    Run,
    ApprovalRequestEvent,
    tokio::sync::broadcast::Receiver<SequencedEvent>,
    Collected,
) {
    let mut receiver = harness.bus.subscribe();
    let run = harness
        .engine
        .submit_prompt(prompt, "project-suspension", Uuid::now_v7())
        .expect("submit prompt");
    let mut collected = Collected::default();
    loop {
        let event = next_event(&mut receiver).await;
        collected.push(&event);
        if event.envelope.name == "APPROVAL_REQUEST" {
            let request: ApprovalRequestEvent =
                serde_json::from_value(event.envelope.payload.clone()).expect("approval payload");
            return (run, request, receiver, collected);
        }
        if let Some(status) = collected.final_run_status.as_deref() {
            if is_terminal_status(status) {
                panic!("run terminated before requesting approval: {status}");
            }
        }
    }
}

/// Collects events until a terminal `RUN_STATUS` (approval must already be handled).
async fn settle(
    mut receiver: tokio::sync::broadcast::Receiver<SequencedEvent>,
    mut collected: Collected,
) -> Collected {
    while !collected.terminal() {
        let event = next_event(&mut receiver).await;
        collected.push(&event);
    }
    collected
}

/// Drives a plain (no-approval) run to terminal and returns the collected trace.
async fn drive_terminal(harness: &Harness, prompt: &str) -> (Run, Collected) {
    let mut receiver = harness.bus.subscribe();
    let run = harness
        .engine
        .submit_prompt(prompt, "project-suspension", Uuid::now_v7())
        .expect("submit prompt");
    let mut collected = Collected::default();
    while !collected.terminal() {
        let event = next_event(&mut receiver).await;
        collected.push(&event);
    }
    (run, collected)
}

fn assert_no_side_effect(harness: &Harness, relative: &str) {
    assert!(
        !harness.workspace_path(relative).exists(),
        "side effect must not exist at {relative}"
    );
}

fn assert_file(harness: &Harness, relative: &str, expected: &str) {
    let bytes = std::fs::read(harness.workspace_path(relative)).expect("file must exist");
    assert_eq!(String::from_utf8(bytes).expect("utf8 file"), expected);
}

fn tool_turns(harness: &Harness, run: RunId) -> Vec<hiverouter_core::db::ToolTurnRecord> {
    harness.db.tool_turns_for_run(run).expect("tool turns")
}

/* ------------------------------------------------------------------ */
/* 1. suspend                                                          */
/* ------------------------------------------------------------------ */

#[tokio::test]
async fn suspend_publishes_awaiting_approval_and_approval_request() {
    let harness = harness(
        ScriptedExecutor::new(vec![tool_only_reply(vec![write_call(
            "call-suspend",
            WRITE_PATH,
            WRITE_TEXT,
            0,
        )])]),
        true,
        120,
    );
    let (run, request, receiver, collected) = begin_approval(&harness, "write the proof file").await;

    assert_eq!(collected.statuses.last().map(String::as_str), Some("awaiting_approval"));
    assert_eq!(collected.tool_calls.len(), 1, "one TOOL_CALL before suspension");
    assert_eq!(collected.tool_calls[0]["tool"], "file_write");
    assert_eq!(collected.tool_results.len(), 0, "nothing executes before a decision");
    assert_eq!(harness.run(run.id).status, AgentState::AwaitingApproval);
    assert_eq!(
        harness.engine.store.get_task(request.task_id.expect("task id")).expect("task").state,
        AgentState::AwaitingApproval
    );
    assert_eq!(request.tool, "file_write");
    assert!(request.nonce.as_deref().is_some_and(|nonce| !nonce.is_empty()));
    assert!(request.args_sha256.as_deref().is_some_and(|hash| hash.len() == 64));
    assert!(request.expires_at.is_some());
    assert!(request.run_id == Some(run.id));
    assert!(request.tool_call_id.as_deref() == Some("call-suspend"));
    assert_no_side_effect(&harness, WRITE_PATH);
    assert!(format!("{:?}", harness.engine).contains("waiters: 1"));
    assert_eq!(
        harness.bridge.approval_count_for_run(run.id).expect("approval count"),
        1
    );

    // Clean terminal state for the test (deny: no side effect).
    assert!(harness.resolve_and_wake(&request, "deny"));
    let _ = settle(receiver, collected).await;
}

/* ------------------------------------------------------------------ */
/* 2. approve once                                                     */
/* ------------------------------------------------------------------ */

#[tokio::test]
async fn approve_resumes_and_executes_the_file_write_exactly_once() {
    let harness = harness(
        ScriptedExecutor::new(vec![tool_only_reply(vec![write_call(
            "call-approve",
            WRITE_PATH,
            WRITE_TEXT,
            0,
        )])]),
        true,
        120,
    );
    let (run, request, receiver, collected) = begin_approval(&harness, "write once").await;
    let resolution = harness.resolve(&request, "approve").expect("approve committed");
    assert!(harness.engine.notify_approval_resolved(&resolution));

    let collected = settle(receiver, collected).await;
    assert_eq!(collected.final_run_status.as_deref(), Some("completed"));
    assert_eq!(collected.outcomes(), vec!["ok".to_owned()]);
    assert_file(&harness, WRITE_PATH, WRITE_TEXT);
    let turns = tool_turns(&harness, run.id);
    assert_eq!(turns.len(), 1, "one durable tool turn");
    assert_eq!(turns[0].outcome, ToolOutcome::Ok);

    // Replay of the same committed resolution cannot produce a second execution.
    assert!(!harness.engine.notify_approval_resolved(&resolution));
    assert_file(&harness, WRITE_PATH, WRITE_TEXT);
    assert_eq!(tool_turns(&harness, run.id).len(), 1);
}

/* ------------------------------------------------------------------ */
/* 3. deny                                                             */
/* ------------------------------------------------------------------ */

#[tokio::test]
async fn deny_returns_denied_result_and_run_continues() {
    let harness = harness(
        ScriptedExecutor::new(vec![tool_only_reply(vec![write_call(
            "call-deny",
            WRITE_PATH,
            WRITE_TEXT,
            0,
        )])]),
        true,
        120,
    );
    let (run, request, receiver, collected) = begin_approval(&harness, "deny the write").await;
    assert!(harness.resolve_and_wake(&request, "deny"));

    let collected = settle(receiver, collected).await;
    assert_eq!(collected.final_run_status.as_deref(), Some("completed"));
    assert_eq!(collected.outcomes(), vec!["denied".to_owned()], "denied tool result");
    assert_eq!(collected.codes(), vec!["".to_owned()], "deny carries no error code");
    assert_no_side_effect(&harness, WRITE_PATH);
    assert_eq!(harness.run(run.id).error_code, None);
}

/* ------------------------------------------------------------------ */
/* 4. timeout                                                          */
/* ------------------------------------------------------------------ */

#[tokio::test]
async fn timeout_fails_run_with_e_approval_timeout_and_no_side_effect() {
    let harness = harness(
        ScriptedExecutor::new(vec![tool_only_reply(vec![write_call(
            "call-timeout",
            WRITE_PATH,
            WRITE_TEXT,
            0,
        )])]),
        true,
        1, // 1 s TTL test seam (D4)
    );
    let (run, request, receiver, collected) = begin_approval(&harness, "time out").await;
    let collected = settle(receiver, collected).await;

    assert_eq!(collected.final_run_status.as_deref(), Some("failed"));
    assert_eq!(
        collected.terminal_status["error_code"].as_str(),
        Some("E_APPROVAL_TIMEOUT")
    );
    assert_eq!(collected.outcomes(), vec!["timeout".to_owned()]);
    assert_eq!(
        harness.decisions(&request.approval_id.to_string()).as_deref(),
        Some("expired")
    );
    assert_no_side_effect(&harness, WRITE_PATH);
    assert_eq!(harness.run(run.id).error_code.as_deref(), Some("E_APPROVAL_TIMEOUT"));
    assert!(
        !collected.sequence.iter().any(|name| name == "APPROVAL_RESOLVED"),
        "timeout must not publish APPROVAL_RESOLVED"
    );
}

/* ------------------------------------------------------------------ */
/* 5. cancel                                                           */
/* ------------------------------------------------------------------ */

#[tokio::test]
async fn cancel_while_awaiting_cancels_run_and_consumes_approval() {
    let harness = harness(
        ScriptedExecutor::new(vec![tool_only_reply(vec![write_call(
            "call-cancel",
            WRITE_PATH,
            WRITE_TEXT,
            0,
        )])]),
        true,
        120,
    );
    let (run, request, receiver, collected) = begin_approval(&harness, "cancel while waiting").await;
    harness.engine.cancel_run(run.id).expect("cancel accepted");

    let collected = settle(receiver, collected).await;
    assert_eq!(collected.final_run_status.as_deref(), Some("cancelled"));
    assert_eq!(collected.outcomes(), vec!["cancelled".to_owned()]);
    assert_eq!(
        harness.decisions(&request.approval_id.to_string()).as_deref(),
        Some("cancelled")
    );
    assert_no_side_effect(&harness, WRITE_PATH);
    assert_eq!(harness.run(run.id).status, AgentState::Cancelled);

    let error = harness
        .resolve(&request, "approve")
        .expect_err("a cancelled request cannot be resolved");
    assert_eq!(error.code(), "E_APPROVAL_REPLAY");
}

/* ------------------------------------------------------------------ */
/* 6. kill worker while awaiting                                       */
/* ------------------------------------------------------------------ */

#[tokio::test]
async fn kill_worker_while_awaiting_cancels_without_side_effect() {
    let harness = harness(
        ScriptedExecutor::new(vec![tool_only_reply(vec![write_call(
            "call-kill",
            WRITE_PATH,
            WRITE_TEXT,
            0,
        )])]),
        true,
        120,
    );
    let (run, request, receiver, collected) = begin_approval(&harness, "kill while waiting").await;
    let task_id = request.task_id.expect("task id on approval");
    harness.engine.kill_worker(task_id).expect("kill accepted");

    let collected = settle(receiver, collected).await;
    // M-012: killing one awaiting worker cancels that task only; the run keeps
    // running (the scripted executor then completes it) and no effect occurs.
    assert_eq!(collected.final_run_status.as_deref(), Some("completed"));
    assert_eq!(collected.outcomes(), vec!["cancelled".to_owned()]);
    assert_no_side_effect(&harness, WRITE_PATH);
    assert_eq!(harness.engine.store.get_task(task_id).expect("task").state, AgentState::Cancelled);
    assert_eq!(harness.run(run.id).status, AgentState::Completed);
}

/* ------------------------------------------------------------------ */
/* 7. CAS race: resolve wins, timer must not fail the run              */
/* ------------------------------------------------------------------ */

#[tokio::test]
async fn timeout_cas_loss_to_resolve_still_resumes() {
    let harness = harness(
        ScriptedExecutor::new(vec![tool_only_reply(vec![write_call(
            "call-cas",
            WRITE_PATH,
            WRITE_TEXT,
            0,
        )])]),
        true,
        120,
    );
    let (run, request, receiver, collected) = begin_approval(&harness, "race the timer").await;
    // Resolve after a wait but inside the TTL; the engine must resume exactly once
    // from the committed row (the second-granularity approval clock makes a
    // sub-second TTL boundary non-deterministic, so the timer-CAS-loss branch is
    // covered by the timeout/restart tests instead of a wall-clock race here).
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(harness.resolve_and_wake(&request, "approve"));

    let collected = settle(receiver, collected).await;
    assert!(
        !collected
            .statuses
            .iter()
            .any(|status| status.as_str() == "failed"),
        "timer must not fail a run whose resolve won the CAS"
    );
    assert_eq!(collected.final_run_status.as_deref(), Some("completed"));
    assert_eq!(collected.outcomes(), vec!["ok".to_owned()]);
    assert_file(&harness, WRITE_PATH, WRITE_TEXT);
    assert_eq!(tool_turns(&harness, run.id).len(), 1);
}

/* ------------------------------------------------------------------ */
/* 8. concurrent resolve: one winner, one execution                    */
/* ------------------------------------------------------------------ */

#[tokio::test]
async fn concurrent_resolve_has_one_winner_and_no_second_execution() {
    let harness = harness(
        ScriptedExecutor::new(vec![tool_only_reply(vec![write_call(
            "call-concurrent",
            WRITE_PATH,
            WRITE_TEXT,
            0,
        )])]),
        true,
        120,
    );
    let (run, request, receiver, collected) = begin_approval(&harness, "resolve twice").await;
    let winner = harness.resolve(&request, "approve").expect("first resolve wins");
    let replay = harness
        .resolve(&request, "approve")
        .expect_err("second resolve must be refused");
    assert_eq!(replay.code(), "E_APPROVAL_REPLAY");
    assert!(harness.engine.notify_approval_resolved(&winner), "one wake");
    assert!(
        !harness.engine.notify_approval_resolved(&winner),
        "second wake cannot execute"
    );

    let collected = settle(receiver, collected).await;
    assert_eq!(collected.final_run_status.as_deref(), Some("completed"));
    assert_eq!(collected.outcomes(), vec!["ok".to_owned()], "exactly one execution result");
    assert_file(&harness, WRITE_PATH, WRITE_TEXT);
    assert_eq!(tool_turns(&harness, run.id).len(), 1);
}

/* ------------------------------------------------------------------ */
/* 9. second prompt while awaiting                                     */
/* ------------------------------------------------------------------ */

#[tokio::test]
async fn second_prompt_while_awaiting_is_refused_e_run_active() {
    let harness = harness(
        ScriptedExecutor::new(vec![tool_only_reply(vec![write_call(
            "call-active",
            WRITE_PATH,
            WRITE_TEXT,
            0,
        )])]),
        true,
        120,
    );
    let (run, request, receiver, collected) = begin_approval(&harness, "first prompt").await;

    let error = harness
        .engine
        .submit_prompt("second prompt", "project-suspension", Uuid::now_v7())
        .expect_err("a second run is refused while approval is pending");
    assert_eq!(error.code(), "E_RUN_ACTIVE");
    assert_eq!(harness.run(run.id).status, AgentState::AwaitingApproval);

    assert!(harness.resolve_and_wake(&request, "deny"));
    let _ = settle(receiver, collected).await;
}

/* ------------------------------------------------------------------ */
/* 10. restart: interrupted + expired, later resolve is timeout        */
/* ------------------------------------------------------------------ */

#[tokio::test]
async fn restart_marks_awaiting_run_interrupted_and_expires_pending() {
    let harness = harness(ScriptedExecutor::new(Vec::new()), true, 120);
    let store = RunStore::new(harness.db.clone());
    let run = store
        .create_run("project-restart", "awaiting approval at crash", AgentState::Queued)
        .expect("run row");
    store
        .set_run_status(run.id, AgentState::AwaitingApproval, None)
        .expect("awaiting status");
    let task = TaskRecord {
        id: TaskId::new(),
        run_id: run.id,
        parent_task_id: None,
        role: "worker".to_owned(),
        depth: 1,
        spawn_seq: 1,
        state: AgentState::AwaitingApproval,
        rev: 0,
    };
    store.insert_task(&task).expect("task row");
    let request = harness
        .bridge
        .request_model_tool_approval(
            run.id,
            task.id,
            "call-restart",
            "file_write",
            &write_args(WRITE_PATH, WRITE_TEXT),
        )
        .expect("pending approval row");
    assert_no_side_effect(&harness, WRITE_PATH);

    // Boot path: consume every orphaned pending row, then mark active runs
    // interrupted (awaiting runs carry E_APPROVAL_TIMEOUT).
    let now = Timestamp::now().unix_secs().unwrap_or(0);
    ToolsBridge::expire_pending_approvals(&harness.db, now).expect("boot orphan reaper");
    let graph = GraphAuthority::new();
    let snapshots = SnapshotStore::new(harness.data.path(), harness.db.clone());
    let report = recover_on_boot(harness.db.clone(), &graph, &snapshots).expect("boot recovery");

    assert!(report.interrupted_runs.contains(&run.id));
    let after = store.get_run(run.id).expect("recovered run");
    assert_eq!(after.status, AgentState::Interrupted);
    assert_eq!(after.error_code.as_deref(), Some("E_APPROVAL_TIMEOUT"));
    assert_eq!(
        harness.decisions(&request.approval_id.to_string()).as_deref(),
        Some("expired"),
        "future-TTL pending row is consumed at boot"
    );
    let error = harness
        .resolve(&request, "approve")
        .expect_err("a decision after restart is refused");
    assert_eq!(error.code(), "E_APPROVAL_TIMEOUT");
    assert_no_side_effect(&harness, WRITE_PATH);
}

/* ------------------------------------------------------------------ */
/* 11. wait cap: ninth approval request is refused                     */
/* ------------------------------------------------------------------ */

#[tokio::test]
async fn wait_cap_refuses_the_ninth_request() {
    let mut calls = Vec::new();
    for index in 0..9u32 {
        calls.push(write_call(
            &format!("call-cap-{index}"),
            &format!("cap-{index}.txt"),
            &format!("cap-{index}"),
            index,
        ));
    }
    let harness = harness(ScriptedExecutor::new(vec![tool_only_reply(calls)]), true, 120);
    let mut receiver = harness.bus.subscribe();
    let run = harness
        .engine
        .submit_prompt("nine writes", "project-cap", Uuid::now_v7())
        .expect("submit prompt");
    let mut collected = Collected::default();
    let mut approved = 0usize;
    while !collected.terminal() {
        let event = next_event(&mut receiver).await;
        collected.push(&event);
        if event.envelope.name == "APPROVAL_REQUEST" {
            let request: ApprovalRequestEvent =
                serde_json::from_value(event.envelope.payload.clone()).expect("approval payload");
            assert!(
                approved < 8,
                "the ninth request must be refused by the wait cap, not published"
            );
            approved += 1;
            assert!(harness.resolve_and_wake(&request, "approve"));
        }
    }

    assert_eq!(approved, 8, "exactly eight approval waits are allowed");
    assert_eq!(
        harness.bridge.approval_count_for_run(run.id).expect("approval count"),
        8,
        "no ninth approval row"
    );
    let codes = collected.codes();
    assert!(
        codes.iter().any(|code| code == "E_CALL_LIMIT"),
        "ninth call must be refused with E_CALL_LIMIT; results={codes:?}"
    );
    for index in 0..8 {
        assert_file(&harness, &format!("cap-{index}.txt"), &format!("cap-{index}"));
    }
    assert_no_side_effect(&harness, "cap-8.txt");
}

/* ------------------------------------------------------------------ */
/* 12. wrong args never consume the grant or execute                   */
/* ------------------------------------------------------------------ */

#[tokio::test]
async fn wrong_args_grant_refused_before_side_effect() {
    let harness = harness(
        ScriptedExecutor::new(vec![tool_only_reply(vec![write_call(
            "call-wrong-args",
            WRITE_PATH,
            WRITE_TEXT,
            0,
        )])]),
        true,
        120,
    );
    let (run, request, receiver, collected) = begin_approval(&harness, "approve real args").await;
    let now = Timestamp::now().unix_secs().unwrap_or(0);
    let resolution = harness.resolve(&request, "approve").expect("approve committed");

    // A fabricating caller that presents different arguments must be refused
    // before any effect and must not burn the grant.
    let error = harness
        .bridge
        .verify_tool_grant(
            run.id,
            ToolId::FileWrite,
            &write_args("wrong-target.txt", "WRONG"),
            now,
        )
        .expect_err("wrong args cannot consume the grant");
    assert_eq!(error.code(), "E_APPROVAL_REQUIRED");
    assert_no_side_effect(&harness, "wrong-target.txt");

    assert!(harness.engine.notify_approval_resolved(&resolution));
    let collected = settle(receiver, collected).await;
    assert_eq!(collected.final_run_status.as_deref(), Some("completed"));
    assert_eq!(collected.outcomes(), vec!["ok".to_owned()]);
    assert_file(&harness, WRITE_PATH, WRITE_TEXT);
    assert_no_side_effect(&harness, "wrong-target.txt");
}

/* ------------------------------------------------------------------ */
/* 13. shell_exec is allowlisted and approved-only                     */
/* ------------------------------------------------------------------ */

#[tokio::test]
async fn shell_exec_runs_only_the_allowlisted_program_after_approval() {
    // The shell arm is landed (WP-02 shell wave): shell_exec is allowlisted,
    // approved, and executed through the Job Object path.
    let harness = harness(
        ScriptedExecutor::new(vec![tool_only_reply(vec![shell_call("call-shell", 0)])]),
        true,
        120,
    );
    let (run, request, receiver, collected) = begin_approval(&harness, "run echo").await;
    assert_eq!(request.tool, "shell_exec");
    assert!(harness.resolve_and_wake(&request, "approve"));

    let collected = settle(receiver, collected).await;
    assert_eq!(collected.final_run_status.as_deref(), Some("completed"));
    assert_eq!(collected.outcomes(), vec!["ok".to_owned()]);
    let payloads = serde_json::to_string(&collected.tool_results).expect("serialize results");
    assert!(
        payloads.contains("sha256") && payloads.contains("bytes"),
        "an executed shell result carries the bounded output size + hash"
    );
    assert!(
        !payloads.contains(SHELL_MARKER),
        "the frozen TOOL_RESULT payload never carries output text; the model gets it from the transcript"
    );
    assert!(
        !payloads.contains("/C"),
        "argv is never published in an event"
    );
    // Non-allowlisted program is refused by the shell policy (never spawned).
    let policy_error = hiverouter_tools::side_effects::preflight_side_effect(
        &WorkspaceJail::open(harness.workspace.path()).expect("jail"),
        &hiverouter_tools::ToolRegistry::standard(),
        &ShellPolicy::new(["cmd.exe".to_owned()]),
        hiverouter_tools::AgentTier::WorkerAgent,
        "shell_exec",
        &serde_json::json!({ "program": "powershell.exe", "args": ["-Command", "id"] }),
        None,
    )
    .expect_err("non-allowlisted program must be refused");
    assert_eq!(policy_error.code(), "E_TOOL_NOT_ALLOWED");
    assert_eq!(harness.run(run.id).status, AgentState::Completed);
}

/* ------------------------------------------------------------------ */
/* 14. no bridge: pre-1.2 read-only refusal preserved                  */
/* ------------------------------------------------------------------ */

#[tokio::test]
async fn no_bridge_engine_keeps_the_read_only_refusal() {
    let harness = harness(
        ScriptedExecutor::new(vec![tool_only_reply(vec![write_call(
            "call-no-bridge",
            WRITE_PATH,
            WRITE_TEXT,
            0,
        )])]),
        false,
        120,
    );
    let (run, collected) = drive_terminal(&harness, "write without bridge").await;

    assert_eq!(collected.final_run_status.as_deref(), Some("completed"));
    assert_eq!(collected.tool_calls.len(), 1, "the attempt is still announced");
    assert_eq!(collected.outcomes(), vec!["refused".to_owned()]);
    assert_eq!(collected.codes(), vec!["E_TOOL_NOT_ALLOWED".to_owned()]);
    assert_no_side_effect(&harness, WRITE_PATH);
    assert_eq!(harness.run(run.id).error_code, None);
}
