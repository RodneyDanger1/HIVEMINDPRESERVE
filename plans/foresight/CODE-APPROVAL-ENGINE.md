# CODE-LEVEL BLUEPRINT — Engine approval suspension / resume (1.2.0-dev)

**Planner:** Code Research Planner 1 · **Mode:** READ-ONLY planning (no product edits, no build, no test, no commit; this file is the only write)
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1`
**Target:** HEAD `37f0e7c` ("wip(1.2): compile-green read-only tool loop, approval command/UI, 23-event contract, e2e tool/approval specs") **plus the uncommitted Lead WIP** listed in §2. Line numbers below were read in the working tree at 2026-11-06 18:20 CT; the Lead WIP was **re-diffed at 18:30 CT and had changed during this planning pass** (approval.rs +78→+166 lines: `expire_orphaned_pending`/`expired_decision_refusal` and `resolve` now maps a stored `expired` decision to `E_APPROVAL_TIMEOUT`; tools_bridge boot reaper now calls `expire_orphaned_pending`). §2 records the 18:30 state; the 18:20 line numbers will drift again. **Re-read `git diff HEAD` before implementing.**
**Predecessor:** `plans/foresight/APPROVAL-SUSPENSION-PLAN.md` (design plan; this file is the code-level instantiation; where this file corrects that plan the correction is labelled **CORRECTION**).
**Skills applied:** `/deep-research`, `/research`, `/forgewright-lead`, `/forgewright-plan`, `/deep-naming`, `/truthseeker`, `/scientific-thinking`, `/forge`, `/evidence-check` (evidence in §16).

**Source labels used below (every factual sentence carries one):**
`[READ <file>:<lines>]` = read from the current working tree; `[WIP]` = uncommitted Lead change read via `git diff`; `[HEAD]` = read from committed HEAD `37f0e7c`; `[PLAN §n]` = read from `APPROVAL-SUSPENSION-PLAN.md`; `[INFERRED]` = derived by this planner from the cited code; `[UNKNOWN]` / `[UNVERIFIED]` = not established.

---

## 0. Scope and deliverable

Implement the engine half of approval suspension/resume: when a model-originated tool call requires operator consent, commit the approval row, persist `awaiting_approval`, publish one `APPROVAL_REQUEST`, wait in-process, and on approve execute the exact approved side effect exactly once; on deny continue the model loop with a denied tool result; on TTL expiry fail the run with `E_APPROVAL_TIMEOUT`; on cancel/kill consume the wait; boot recovery/expiry is the already-existing Lead WIP.

**Deliverable scope (files this blueprint may lead an implementing wave to change):** `core/src/execution.rs`, `core/src/tools_bridge.rs`, `core/src/tool_loop.rs`, `core/src/ws.rs`, `core/src/http.rs`, `tools/src/approval.rs`, `tools/src/executor.rs`, `tools/src/lib.rs`, new `core/tests/suspension.rs` (+ minimal reconciliation of the tests named in §12.3). **No change:** `db.rs`, `contract/*`, `web/*`, `main.rs`, `recovery.rs` (merge WIP only), `tool_loop` contract/event vocabulary, refusal codes. `[PLAN §9]` row 9 explicitly says `db.rs` **no change**. `[READ db.rs:32-145]` migrations v1–v4 contain no approvals table; approvals are created by `tools::approval::migrate` through `ToolsBridge::migrate` `[READ http.rs:107-113; tools_bridge.rs:79-88]`.

---

## 1. Current code map (exact signatures and line regions; read in the WIP tree)

### 1.1 `core/src/execution.rs` (1,130 lines before the WIP `+5`)

| Symbol | Signature / shape | Lines |
|---|---|---|
| `ProviderExecutor` trait | `async fn plan(&self, run_id: RunId, prompt: &str, cancel: &CancellationToken) -> Result<Plan, Refusal>`; `async fn execute(&self, run_id: RunId, task: &TaskRecord, prompt: &str, cancel: &CancellationToken, on_delta: &DeltaSink) -> Result<StreamedReply, Refusal>`; `async fn continue_with_tools(..., transcript: &[ToolTurnTranscript]) -> Result<StreamedReply, Refusal>`; `async fn finish_run(&self, _run_id: RunId)` | 54–107 |
| `StreamedReply` | `{ deltas: Vec<String>, usage: Usage, tool_calls: Vec<AssembledToolCall> }`; **WIP adds `reasoning: String`** `[WIP]` | 113–122 |
| `ToolTurnTranscript` | `{ assistant_text: String, results: Vec<(ToolCallPlan, ToolExecution)> }`; **WIP adds `reasoning: String`** `[WIP]` | 137–145 |
| `ToolTurnResult` (private) | `{ cap_reached: bool, budget_refused: bool, executed: Vec<(ToolCallPlan, ToolExecution)> }` | 149–161 |
| `RunEngine` fields | `pub store`, `pub db: Arc<Database>`, `pub bus: Arc<EventBus>`, `pub graph: Arc<GraphAuthority>`, `pub contexts: Arc<ContextStore>`, `pub tokens: Arc<RunRegistry>`, `pub executor: Arc<dyn ProviderExecutor>`, `pub templates: Arc<TemplateRegistry>`, `pub caps: Caps`, `pub snapshots: Option<Arc<SnapshotStore>>`, `tool_workspace: Option<WorkspaceJail>`, `tool_registry: ToolRegistry`, `active_run: Mutex<Option<RunId>>` | 260–290 |
| `RunEngine::new` | `pub fn new(db: Arc<Database>, bus: Arc<EventBus>, graph: Arc<GraphAuthority>, executor: Arc<dyn ProviderExecutor>, templates: Arc<TemplateRegistry>) -> Self` | 305–327 |
| `with_snapshot_store` / `with_tool_workspace` | `pub fn with_snapshot_store(mut self, Arc<SnapshotStore>) -> Self`; `pub fn with_tool_workspace(mut self, WorkspaceJail) -> Self` | 331–348 |
| `with_services` | `pub fn with_services(db, bus, graph, contexts, tokens, executor, templates, caps) -> Self` | 352–377 |
| `active_run` / `set_active` | `pub fn active_run(&self) -> Option<RunId>`; `fn set_active(&self, Option<RunId>) -> Result<(), Refusal>` | 381–389 |
| `publish` / `publish_node` | `fn publish(&self, Event, Option<RunId>, Option<Uuid>)`; `fn publish_node(&self, RunId, TaskId, AgentState)` | 391–403 |
| `submit_prompt` | `pub fn submit_prompt(self: &Arc<Self>, prompt: &str, project_id: &str, correlation: Uuid) -> Result<Run, Refusal>` | 410–448 |
| `spawn_sub_hive` | `pub fn spawn_sub_hive(&self, parent_task_id: TaskId, role: &str) -> Result<TaskRecord, Refusal>` | 455–493 |
| `cancel_run` | `pub fn cancel_run(&self, run_id: RunId) -> Result<(), Refusal>` | 499–519 |
| `kill_worker` | `pub fn kill_worker(&self, task_id: TaskId) -> Result<(), Refusal>` | 525–537 |
| `dispatch` | `async fn dispatch(self: Arc<Self>, run: Run, token: CancellationToken)` — maps `Ok→completed`, `Err(ToolCancelled)→cancelled`, other `Err→failed+code`; persists+publishes terminal `RUN_STATUS`; snapshot; clears active/token; `finish_run` | 539–583 |
| `dispatch_inner` | `async fn dispatch_inner(&self, run: &Run, token: &CancellationToken) -> Result<(), Refusal>` — sets run `planning`, one planning call (`caps.check_calls(1)`), plan validate, root+plan tasks, executes leaves via `execute_task` | 585–658 |
| `execute_tool_turn` | **sync** `fn execute_tool_turn(&self, run: &Run, task: &TaskRecord, reply: &StreamedReply, token: &CancellationToken) -> Result<ToolTurnResult, Refusal>`; per call: counters → `next_turn` → `check_call_budget` → `plan_call` → `validate_plan` → `publish_tool_call` → `execute_tool_plan`; then `finish_tool_call`; one `increment_run_tool_counters(run.id, 1, calls_in_turn)` at the end | 684–806 |
| `execute_tool_plan` | **sync** `fn execute_tool_plan(&self, plan: &ToolCallPlan) -> ToolExecution`; non-`Read` → `ToolExecution::refused(E_TOOL_NOT_ALLOWED, "side-effecting calls stay approval/loop-owned and did not execute")`; no jail → `E_FILE_TREE_UNAVAILABLE`; else `execute_read_only` | 879–920 |
| `publish_tool_call` | `fn publish_tool_call(&self, run: &Run, task: &TaskRecord, plan: &ToolCallPlan)` | 923–931 |
| `finish_tool_call` | `fn finish_tool_call(&self, run, task, call: &AssembledToolCall, execution: &ToolExecution, turn: u32) -> Result<(), Refusal>`; publishes `TOOL_RESULT`, inserts `tool_turns` | 934–966 |
| `execute_task` | `async fn execute_task(&self, run: &Run, task: &TaskRecord, token: &CancellationToken) -> Result<(), Refusal>`; sets task `running`, one `execute`, delta publish, `loop { execute_tool_turn; transcript.push; continue_with_tools }`, `complete_task` | 968–1100 |
| `complete_task` | `fn complete_task(&self, run, task, reply: &StreamedReply) -> Result<(), Refusal>` | 1104–1128 |

Key derived facts: the run is set to `planning` and **never set to `running`** before leaf execution `[READ execution.rs:585-586,968-981; grep=no running set_run_status]`; task start is a direct `store.update_task_state(..., Running)` `[READ :978-981]`; the non-read refusal branch is exactly `execute_tool_plan` `:880-891` and is the suspension seam `[PLAN §3 item 9]`; `ToolTurnTranscript` push happens before `continue_with_tools` `[READ :1055-1073; WIP adds reasoning]`.

### 1.2 `core/src/run_state.rs` (1,237 lines; pure, no I/O)

| Symbol | Shape | Lines |
|---|---|---|
| `BEFORE_RUN` / state sets | `pub const BEFORE_RUN: AgentState = AgentState::Empty`; `RUN_STATES`, `RUN_TERMINAL_STATES`, `TASK_STATES`, `TASK_TERMINAL_STATES` | 56–106 |
| `StateEvent` | `Submit, Plan, Start, Complete, Fail, Cancel, Kill, Suspend, Resume, Deny, Timeout, Restart` + `as_str` | 131–178 |
| `Scope` / terminal refusal | `Run → E_RUN_TERMINAL`, `Task → E_TASK_TERMINAL` | 181–199 |
| `Effect` | `CreateRun, MarkPlanning, StartRun, MarkCompleted, MarkFailed, CancelRun, RequestApproval, ResumeFromApproval, FeedDeniedToolResult, FailOnApprovalTimeout, MarkInterrupted, CreateTask, StartTask, MarkTaskCompleted, MarkTaskFailed, MarkTaskCancelled, MarkTaskKilled, RequestTaskApproval, ResumeTaskFromApproval, FeedTaskDeniedToolResult, FailTaskOnApprovalTimeout, MarkTaskInterrupted`; `Effect::refusal()` returns `Some(Refusal::ApprovalTimeout)` **only** for the two timeout effects | 205–266 |
| `RunTransition` / `TaskTransition` | `{ from, event, to, effect }` (all `Copy`) | 270–292 |
| `RunStateError` | `Terminal{scope,state,event} → E_RUN_TERMINAL/E_TASK_TERMINAL`; `Illegal → E_SCHEMA_INVALID`; `ApprovalReplay → E_APPROVAL_REPLAY`; `Invariant → E_INTERNAL_ERROR`; `refusal()/code()` | 302–354 |
| `apply_run` | `pub fn apply_run(from: AgentState, event: StateEvent) -> Result<RunTransition, RunStateError>`; edges: `Running+Suspend→AwaitingApproval (RequestApproval)`, `AwaitingApproval+Resume→Running (ResumeFromApproval)`, `+Deny→Running (FeedDeniedToolResult)`, `+Timeout→Failed (FailOnApprovalTimeout)`, `Cancel` legal from Queued/Planning/Running/Awaiting→Cancelled, `Restart` from active→Interrupted; illegal Resume/Deny/Timeout replay → `ApprovalReplay` | 465–538 |
| `apply_task` | `pub fn apply_task(from: AgentState, event: StateEvent) -> Result<TaskTransition, RunStateError>`; `Queued+Start→Running (StartTask)`, `Running+Complete→Completed`, `Queued/Running+Fail→Failed`, `Cancel` legal from Queued/Running/Awaiting→Cancelled, `Running+Kill→Killed`, **`AwaitingApproval+Kill→Cancelled (MarkTaskCancelled)`**, `Running+Suspend→Awaiting (RequestTaskApproval)`, `Awaiting+Resume/Deny/Timeout` mirror run, `Restart`→Interrupted | 549–620 |
| helpers | `can_apply_run/task`, `legal_events_run/task`, `is_run_state/is_task_state/is_run_terminal/is_task_terminal` | 422–458, 622–656 |

`execution.rs` has **zero** `run_state` references today `[READ grep run_state execution.rs = 0; PLAN §1 Q5]`. The approval edges are exactly the ones the engine must apply `[PLAN §2.4, §5, D6]`.

### 1.3 `core/src/tools_bridge.rs` (781 lines)

| Symbol | Signature | Lines |
|---|---|---|
| `ToolsBridge` fields | `jail: Option<WorkspaceJail>`, `jail_error: Option<String>`, `gate: ApprovalGate`, `workspace_root: PathBuf`, `db: Arc<Database>`, `ledger: Mutex<Option<(PathBuf, RunLedger)>>` (all private; `Debug`) | 35–45 |
| `new` | `pub fn new(workspace_root: &Path, db: Arc<Database>) -> Self`; gate = `ApprovalGate::new()` | 51–77 |
| `migrate` | `pub fn migrate(db: &Database) -> Result<(), Refusal>` → `approval::migrate` on one connection | 84–88 |
| `expire_pending_approvals` **(WIP)** | `pub fn expire_pending_approvals(db: &Database, now_unix_secs: u64) -> Result<u64, Refusal>` → `approval::expire_pending` `[WIP; READ :90-104]` | 90–104 |
| `approval_gate` | `pub fn approval_gate(&self) -> &ApprovalGate` | 126–128 |
| `resolve_approval` | `pub fn resolve_approval(&self, payload: &command::ApprovalResolve) -> Result<ApprovalResolution, Refusal>`; uses `Timestamp::now()` | 170–187 |
| `approval_ledger_dir`/`_file` | `pub fn approval_ledger_dir(data_dir: &Path) -> PathBuf` (`data_dir/ledger`); `approval_ledger_file` | 194–196, 256–258 |
| `resolve_approval_ledgered` | `pub fn resolve_approval_ledgered(&self, payload: &ApprovalResolve, ledger_dir: &Path) -> Result<ApprovalResolution, Refusal>` — gate CAS first, then one ledger row; ledger failure returns `E_INTERNAL_ERROR` after the decision committed | 217–239 |
| `pending_approvals` | `pub fn pending_approvals(&self, run_id: RunId) -> Result<Vec<ApprovalRequest>, Refusal>` | 247–252 |
| `request_model_tool_approval` | `pub fn request_model_tool_approval(&self, run_id: RunId, task_id: TaskId, tool_call_id: &str, tool: &str, args: &serde_json::Value) -> Result<event::ApprovalRequest, Refusal>` — dedupes pending `(run, tool_call_id)` same args, refuses different args `E_APPROVAL_REPLAY`, commits through gate, returns wire payload; **no caller in production** `[grep: bridge tests only]` | 287–327 |
| `approval_request_event` | `pub fn approval_request_event(&ApprovalRequest) -> event::ApprovalRequest` — carries approval_id/tool/nonce/run_id/task_id/tool_call_id/args_sha256/expires_at, `decision: None` | 336–348 |
| `pending_approval_events` | `pub fn pending_approval_events(&self, run_id) -> Result<Vec<event::ApprovalRequest>, Refusal>` | 358–367 |
| `approval_resolved_event` | `pub fn approval_resolved_event(&ApprovalResolution) -> event::ApprovalResolved` (`approve`/`deny`) | 382–393 |

**Missing (this blueprint adds):** per-approval `expire_approval`, `cancel_approval`, `approval_decision`, `verify_tool_grant`, `approval_count_for_run`, and a test/CLI TTL seam on the gate (`with_approval_ttl`). `[READ: no such methods in the file]`

### 1.4 `core/src/ws.rs` (577 lines)

`Command::ApprovalResolve(payload)` arm `[READ ws.rs:465-483]`: computes `ledger_dir = ToolsBridge::approval_ledger_dir(&state.config.data_dir)`; `state.tools.resolve_approval_ledgered(&payload, &ledger_dir)`; on `Ok(resolution)` builds `approval_resolved_event(&resolution)`, `publish_ack(CommandName::ApprovalResolve, correlation)`, `publish_with_correlation(Event::ApprovalResolved(resolved), correlation)`; on `Err` `publish_error`. **No engine call.** `state.engine` is `Arc<RunEngine>` `[READ http.rs:49-50]`, so the wake call is a one-line addition after the two publishes.

### 1.5 `core/src/http.rs` (558 lines; WIP adds 15 lines in `build_tools`)

`AppState` fields `[READ :34-62]`; `build_tools` `[READ :107-130]` (WIP boot TTL reaper, `ToolsBridge::migrate`, then `Arc::new(ToolsBridge::new(...))`); `build_executor` `:134`; `build_engine` `[READ :155-179]` signature `fn build_engine(db: Arc<Database>, bus: Arc<EventBus>, graph: Arc<GraphAuthority>, config: &Config, templates: Arc<TemplateRegistry>, snapshots: Arc<SnapshotStore>, wire_workspace_jail: bool) -> Arc<RunEngine>` — currently `RunEngine::new(...).with_snapshot_store(...)` and optional `.with_tool_workspace(...)`; constructors `new` (:189–214), `with_graph` (:222–251), `with_services` (:260–285); **order in all three: engine first, `tools` second** `[READ :198-200, 235-237, 269-271]`; `build_tools` therefore cannot be passed into the engine yet. Routes `[READ :349-361]` have no approval HTTP route; resolve is WS-only.

### 1.6 `tools/src/approval.rs` (1,251 lines; WIP adds `expire_pending` + one test)

| Symbol | Signature | Lines |
|---|---|---|
| `APPROVAL_TTL_SECS` | `pub const APPROVAL_TTL_SECS: u64 = 120` | 26 |
| `ApprovalDecision` | `AllowOnce/AllowSession/Deny`; `label()` → `allow_once/allow_session/deny`; `from_label`; `from_wire("approve"→AllowOnce,"deny"→Deny)` | 31–81 |
| `ApprovalRequest` | `{ approval_id, run_id, task_id, tool_call_id, tool: ToolId, permission_class, args_sha256, created_at, expires_at, nonce }`; `is_expired(now)` | 85–117 |
| `ApprovalResolution` | `{ request: ApprovalRequest, decision: ApprovalDecision, decided_at: Timestamp }` | 121–128 |
| `hash_args` | `pub fn hash_args(&serde_json::Value) -> String` | 132 |
| `migrate` | DDL columns `approval_id TEXT PK, run_id, task_id, tool_call_id, tool_name, args_sha256, permission_class, created_at, expires_at, nonce UNIQUE, decision, decided_at, reason, executed_at`; two **non-unique** indexes `approvals_run_pending`, `approvals_session` | 141–165 |
| `insert_request` | inserts decision NULL; unique violation → `E_APPROVAL_REPLAY` | 171–202 |
| `resolve_with_nonce` | `(conn, approval_id, nonce, decision, reason, now) -> Result<ApprovalResolution, ToolError>`; nonce mismatch/used → `E_APPROVAL_REPLAY`; TTL → marks `expired` + `E_APPROVAL_TIMEOUT`; CAS `WHERE decision IS NULL` single winner | 415–514 |
| `expire_pending` **(WIP)** | `pub fn expire_pending(conn, now) -> Result<u64, ToolError>`; bulk CAS `decision='expired'` for elapsed pending rows `[WIP; READ :516-555]` | 516–555 |
| `pending_for_run` | `pub fn pending_for_run(conn, run_id) -> Result<Vec<ApprovalRequest>, ToolError>` | 561–579 |
| `ApprovalGate` | `{ ttl_secs: u64 }` (`Debug, Clone`); `new/with_ttl/ttl_secs` | 583–614 |
| `request` | `pub fn request(&self, conn, run_id, task_id, tool_call_id: impl Into<String>, tool: ToolId, permission_class, args, now) -> Result<ApprovalRequest, ToolError>`; read-only class → `E_INTERNAL_ERROR` | 621–652 |
| `decide` / `resolve_command` | `decide(conn, approval_id, decision, now)`; `resolve_command(conn, approval_id, nonce, decision, reason, now)` | 658–682 |
| `verify_grant` | `pub fn verify_grant(&self, conn, run_id: RunId, tool: ToolId, args, now) -> Result<(), ToolError>` — session match → Ok; else single-use allow_once row CAS `executed_at IS NULL`; absent/denied/expired → `E_APPROVAL_REQUIRED`; already executed → `E_APPROVAL_REPLAY` | 694–752 |
| `request_tool_approval` | `pub fn request_tool_approval(gate, conn, run_id, task_id, tool_call_id, tool: &str, args, now) -> Result<ApprovalRequest, ToolError>`; empty id `E_SCHEMA_INVALID`, unknown tool `E_TOOL_NOT_FOUND`, missing class `E_INTERNAL_ERROR` | 771–… |

**Missing:** `expire(conn, approval_id, now) -> Result<bool>`, `cancel_pending`, `decision_label`, `count_for_run`; unique partial pending index (E14/D11); gate TTL test seam through the bridge.

### 1.7 `tools/src/executor.rs` / `shell.rs` / `jail.rs` / `tool.rs`

- `ExecutedTool` `{ Ok{text}, Refused{code,detail} }` `[READ executor.rs:25-39]`; `execute_read_only(jail, registry, tier, tool, args) -> Result<ExecutedTool, ToolError>` `:52-80`; **no side-effect executor exists** `[PLAN §1 Q8]`.
- `ShellPolicy` (`new`, `conservative_default` = cmd.exe/git.exe/cargo.exe, `with_timeout`, `allows`), `ShellRequest::{new, with_cwd}`, `ShellOutput` (Serialize) `[READ shell.rs:34-162]`; **`pub async fn execute(jail, policy, request, cancellation: Option<&CancellationToken>) -> Result<ShellOutput, ToolError>`** `:173-178` → `execute_side_effect` **must be async** (**CORRECTION** to the sync signature in `[PLAN §4]`).
- `WorkspaceJail::write(&self, requested: &str, bytes: &[u8]) -> Result<PathBuf, ToolError>` `[READ jail.rs:223-243]`.
- `FileWriteArgs { path, content, expected_sha256: Option<String> }`, `ShellExecArgs { program, args, cwd: Option<String> }` `[READ tool.rs:252-276]`; `ToolRegistry::resolve_call(tier, tool_name, args) -> Result<&ToolContract, ToolError>` `:446-461`; WorkerAgent allowlist includes `file_write`/`shell_exec` `:370-380`; permission classes: FileWrite=`Mutate`, ShellExec=`Execute`, reads=`Read` `:315-350`.
- `expected_sha256` is **not used anywhere** `[READ grep expected_sha256 = 0 outside the struct]`; **no frozen stale-write refusal code exists** `[READ refusal.rs code list :75-279: no hash/conflict code]` → see decision D-2.

### 1.8 Contract surfaces that already exist (no change)

- `Command::ApprovalResolve(ApprovalResolve { approval_id, nonce, decision, reason })` `[READ command.rs:187-199]`.
- `Event::ApprovalRequest(event::ApprovalRequest { approval_id, tool, decision, nonce?, run_id?, task_id?, tool_call_id?, args_sha256?, expires_at? })` `[READ event.rs:250-281]`; `Event::ApprovalResolved(ApprovalResolved { approval_id, tool, decision })` `[READ event.rs:365-374]`; `RunStatusPayload { run_id, status, error_code: Option<String> }`, `NodeEvent { task_id, state }`, `ToolCallPayload { run_id, task_id, tool_call_id, tool, args_sha256, turn }`, `ToolResultPayload { run_id, task_id, tool_call_id, outcome, code?, bytes?, sha256? }` `[READ event.rs around 230-360]`.
- `AgentState::AwaitingApproval` is a frozen state `[READ run_state.rs:64-106]`; refusals `ApprovalRequired/Replay/Timeout/NotFound/DecisionInvalid` exist with codes `[READ refusal.rs:179-191]`; **no new refusal code is needed** `[PLAN D11; READ refusal list]`.
- `ApprovalId` newtype (`Display`, `new`, `from_uuid`) `[READ ids.rs:95-106]`; `RunId/TaskId` same.

### 1.9 Store ownership (effects become simple calls)

`RunStore::set_run_status(&self, run_id, state, error_code: Option<&str>) -> Result<(), Refusal>` `[READ store.rs:136]`; `update_task_state(&self, task_id, state) -> Result<u64, Refusal>` `:213`; `get_run` `:74`; `get_task` `:192`; `latest_active_run` `:95` includes `awaiting_approval` `[PLAN §1 Q13]`; `append_message` `:272`. `EventBus::publish_for_run(&self, Event, Option<RunId>, Option<Uuid>)` `[READ bus.rs:129]`.

---

## 2. Merge notes for the uncommitted Lead changes (`git diff HEAD`, read 18:20 CT, re-diffed 18:30 CT)

The WIP is a **merge precondition**, not current behavior. Exact hunks `[WIP]`:

| File | WIP change | Merge consequence for this blueprint |
|---|---|---|
| `core/src/execution.rs` (+5) | `StreamedReply.reasoning: String`; `ToolTurnTranscript.reasoning: String`; transcript push adds `reasoning: final_reply.reasoning.clone()` | Any construction of `StreamedReply`/`ToolTurnTranscript` in new code must fill `reasoning`; new suspension code does not touch these fields. The transcript push is at `:1055-1059`; the `execute_tool_turn` call site is `:1021`. |
| `core/src/provider_executor.rs` (+20) | captures `reasoning_content`, returns `reasoning`, replays it with `.with_reasoning_content(turn.reasoning.clone())` | No conflict with approval code; do not edit this file for approval. |
| `core/src/recovery.rs` (+13) | active set becomes `(RunId, AgentState)`; `AwaitingApproval` runs get `error_code=Some(E_APPROVAL_TIMEOUT)` when marked `interrupted` | **Suspension precondition** (D8). Keep as-is; do not re-implement. `RecoveryReport` still has only `interrupted_runs/restored_runs/corrupt_snapshots` `[READ recovery.rs:13-22]`; boot expiry count is only logged in `build_tools` `[WIP http.rs]`. |
| `core/src/tools_bridge.rs` (+17) | `pub fn expire_pending_approvals(db, now)` — doc says **boot orphan reaper**: calls `hiverouter_tools::approval::expire_orphaned_pending` (unconditional, TTL not consulted, because no in-process waiter survives restart); `http.rs` log text says "consumed restart-orphaned pending approvals at boot" | Merge first; the engine's per-approval TTL `expire_approval` is a **different** function (`expire`), not this boot reaper. Do not repurpose `expire_pending_approvals` for the live timer. |
| `core/src/http.rs` (+15) | `build_tools` boot reaper: `ToolsBridge::expire_pending_approvals(db, SystemTime now)` with orphan-expiry log wording | The blueprint's construction-order change (§7) rewrites the last line of `build_tools` and all three `AppState` constructors; rebase the WIP hunk closely. Keep the reaper inside `build_tools`; it is still called before the listener binds (`main.rs` recovery → `AppState::with_graph` → `serve`) `[READ main.rs:81-123]`. |
| `hiverouter-tools/src/approval.rs` (+166, changed during the pass) | `expire_orphaned_pending(conn, now)` (boot: consume **every** pending row unconditionally), `expire_pending(conn, now)` (TTL reaper), `expired_decision_refusal()` (resolve now reports a stored `expired` decision as `E_APPROVAL_TIMEOUT`, not replay), plus their tests | New per-approval `expire` and `cancel_pending` must be added **beside** these; recommend `expire_pending` reuse the new single-row `expire` (CAS per id). Conflict risk: both touch the region after `resolve_with_reason`. The engine's timeout path uses the per-row `expire`, never the boot orphan reaper. |
| `core/tests/tool_loop_engine.rs` (+4) | `reasoning: String::new()` in the three `StreamedReply` literals | No approval test lives here; keep the file as-is unless the suspension harness reuses `ScriptedExecutor` (do not duplicate). |
| `core/tests/run_lifecycle.rs` (+1) | `reasoning: String::new()` | No conflict. |

**Merge order (critical):** (1) WIP reasoning+recovery+expiry lands/commits; (2) `tools/approval.rs` new single-row primitives + unique index; (3) `tools/executor.rs::execute_side_effect` + `lib.rs` exports; (4) `tools_bridge` delegates (+ gate TTL seam); (5) engine suspension + `run_state` wiring + `tool_loop` wait cap; (6) `ws.rs` notify; (7) `http.rs` construction order; (8) tests. Do **not** merge (5) before (2)–(4): the engine calls methods that would otherwise not exist. `[PLAN §9 merge order]`

---

## 3. Design decisions (code-level; each names the rejected alternative)

- **D1 — In-process inline wait (no new dispatch outcome).** `execute_task` awaits the approval wake/hold inside the tool-turn loop; `dispatch_inner`/`execute_task` keep `Result<(), Refusal>`, so `dispatch`'s terminal mapping and cleanup remain the single owner. Rejected: returning a "Suspended" variant to `dispatch` (would require a new public outcome type and a resume entry point; the current run is single-active and dispatch stays alive anyway). `[INFERRED from READ execution.rs:539-658,968-1100]`
- **D2 — Run status must become `running` before any run-level `Suspend`.** The table only accepts `Running + Suspend` `[READ run_state.rs:497-499]`; today the run never leaves `planning` `[READ execution.rs:585-586,968-981]`. The blueprint applies `apply_run(Planning, Start)` exactly once in `dispatch_inner` before the leaf loop and publishes one `RUN_STATUS running`. This is a behavior addition (an extra status frame) and is called out for test reconciliation (§12.3). Rejected: applying `Suspend` from `Planning` (illegal by table; would bypass the authority).
- **D3 — Suspension only when both a `ToolsBridge` and a `WorkspaceJail` are wired.** Otherwise `execute_tool_turn` keeps today's `E_TOOL_NOT_ALLOWED` refusal (`execute_tool_plan` branch). This keeps `RunEngine::new`/`with_services` engines (all current tests) byte-compatible and refuses honestly instead of opening a wait that can never execute. **CORRECTION to `[PLAN §4]`** which proposed `E_INTERNAL_ERROR` for a no-tools engine: the existing Wave-2 test `non_read_tool_is_refused_by_name_and_never_executes` `[READ tests/tool_loop_engine.rs]` expects `E_TOOL_NOT_ALLOWED`; reusing the frozen code is more truthful than a generic internal error and avoids weakening an existing guard.
- **D4 — Approval TTL is owned by the request row (`expires_at`), not by a separate engine TTL.** The wait deadline is `request.expires_at - now()`; `with_approval_ttl` exists only as a fallback for malformed/absent `expires_at`. Tests need a short-TTL gate: add `ToolsBridge::with_approval_ttl(ttl_secs)` that rebuilds `self.gate = ApprovalGate::with_ttl(ttl_secs)` (test seam; production keeps 120 s). **CORRECTION to `[PLAN §4]` `RunEngine::with_approval_ttl`:** engine-side TTL alone cannot expire a row committed by a 120 s gate; the gate TTL and the timer must agree.
- **D5 — The engine does not own the ledger.** `[PLAN §4 item 1/§9]` proposed `RunEngine.ledger_dir`; the ledger is written exclusively by `ToolsBridge::resolve_approval_ledgered` from the WS command `[READ ws.rs:465-483; tools_bridge.rs:198-239]`. The engine never appends ledger rows (prevents double-write, `[PLAN §8 bug 9]`). No engine ledger field.
- **D6 — `run_state` is used only for the approval edges plus the run `Start` required by D2.** Start/complete/fail of tasks stay on the existing direct writes in this wave (`[PLAN D6]`), minimizing the diff; the approval path uses `apply_run`/`apply_task` and applies the returned effect exactly once. Rejected: full engine refactor onto `run_state` in the same change.
- **D7 — Terminal frame ownership moves to the path that persisted the terminal state; `dispatch` skips duplicates.** `cancel_run` already persists+publishes `cancelled` `[READ execution.rs:507-517]`, and the timeout path must persist+publish `failed E_APPROVAL_TIMEOUT` from `Effect::FailOnApprovalTimeout` `[READ run_state.rs:224-225,258-265]`. Therefore `dispatch` must not blindly re-persist/re-publish a terminal status; it reads the persisted status and only terminalizes when still non-terminal (then always does snapshot/cleanup). Rejected: duplicate terminal frames (the pre-existing bug listed in `[PLAN §8 bug 14]`).
- **D8 — Boot expiry and the WIP recovery error code are preconditions, not options.** No suspension ships without them; the WIP already marks `AwaitingApproval` runs `interrupted` with `E_APPROVAL_TIMEOUT` `[WIP recovery.rs]` and expires pending rows at boot `[WIP http/tools_bridge/approval]`. No engine-side auto-approve/auto-resume on boot (`[PLAN D8]`).
- **D9 — No new refusal code, no new contract event/command.** Reuse `E_TOOL_NOT_ALLOWED`, `E_APPROVAL_REQUIRED/REPLAY/TIMEOUT/NOT_FOUND`, `E_CALL_LIMIT`, `E_FILE_TREE_UNAVAILABLE`, `E_TOOL_SCHEMA`, `E_INTERNAL_ERROR` `[READ refusal.rs:75-279; PLAN D11]`.
- **D10 — Exactly-once execution = `verify_grant` before every side effect, in the tools layer.** `ApprovalGate::verify_grant` consumes an `allow_once` via `executed_at IS NULL` CAS `[READ approval.rs:694-752]`; the engine calls a bridge delegate and never calls `jail.write`/`shell::execute` directly (`[PLAN D10]`).
- **D11 — No crash-resume of a suspended turn.** In-process wait only; restart → recovery interrupts + boot expiry `[PLAN D1/D8]`.
- **D12 — Counters count attempts before the wait.** Increment the persisted call counter for the suspended call **before** waiting so deny/timeout cannot be retried for free (`[PLAN E11]`); the end-of-turn increment adds only the non-pre-counted calls and the turn. Add `MAX_APPROVAL_WAITS_PER_RUN=8` in `tool_loop.rs`; a 9th request is refused as a `ToolExecution::refused(E_CALL_LIMIT)` without committing a row.
- **D13 — Timeout/cancel must still produce one `TOOL_RESULT` + one `tool_turns` row.** `[PLAN §8 bug 10]`; `ToolExecution::timed_out/cancelled/denied` constructors already exist `[READ tool_loop.rs:82-120]`. The wait helper finalizes those paths itself so `execute_tool_turn` never double-finishes a planned call.

---

## 4. New declarations (exact Rust; additive)

### 4.1 `core/src/tool_loop.rs`

```rust
/// Maximum approval waits (committed requests) per run; a 9th request is
/// refused with the frozen `E_CALL_LIMIT` code, never a new code.
pub const MAX_APPROVAL_WAITS_PER_RUN: u32 = 8;
```

### 4.2 `core/src/execution.rs` — new imports, constants, waiter types

```rust
use hiverouter_contract::ApprovalId;
use hiverouter_tools::approval::{
    ApprovalDecision, ApprovalResolution,
};
use hiverouter_tools::shell::ShellPolicy;
use hiverouter_tools::executor::execute_side_effect;
use tokio::sync::oneshot;

use crate::run_state::{self, StateEvent};
use crate::tools_bridge::ToolsBridge;

/// One time source for approval timestamps (tests inject a fixed clock).
pub type Clock = Arc<dyn Fn() -> u64 + Send + Sync>;

/// What a suspended wait can learn. `Resolved` is the gate's committed
/// decision; a cancelled wait never produces a decision.
#[derive(Clone, Debug, PartialEq)]
pub enum ApprovalWake {
    /// The operator decision was committed and handed to the engine.
    Resolved(ApprovalResolution),
    /// The run/task was cancelled while waiting (row consumed as `cancelled`).
    Cancelled,
}

/// In-process waiter registry: one oneshot sender per pending `approval_id`.
/// Armed before the `APPROVAL_REQUEST` is published, so a fast resolve can
/// never be lost. The mutex is only held for map operations (never across
/// `.await`).
#[derive(Default)]
pub struct ApprovalWaiters {
    inner: std::sync::Mutex<HashMap<ApprovalId, oneshot::Sender<ApprovalWake>>>,
}

impl std::fmt::Debug for ApprovalWaiters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApprovalWaiters").field("armed", &self.len()).finish()
    }
}

impl ApprovalWaiters {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers the receiver for `approval_id`. Inserting over a stale key
    /// drops the old sender (its receiver observes a closed channel and fails
    /// closed instead of waiting forever).
    ///
    /// # Errors
    /// `E_INTERNAL_ERROR` only if the mutex is poisoned.
    pub fn arm(&self, approval_id: ApprovalId) -> Result<oneshot::Receiver<ApprovalWake>, Refusal> {
        let (sender, receiver) = oneshot::channel();
        let mut inner = self.inner.lock().map_err(|_| Refusal::InternalError)?;
        inner.insert(approval_id, sender);
        Ok(receiver)
    }

    /// Delivers a resolved wake exactly once. Returns false when no waiter is
    /// armed (decided row with no live wait: boot/recovery/late resolve).
    pub fn wake_resolved(&self, resolution: &ApprovalResolution) -> bool {
        let sender = self
            .inner
            .lock()
            .ok()
            .and_then(|mut inner| inner.remove(&resolution.request.approval_id));
        sender.is_some_and(|sender| sender.send(ApprovalWake::Resolved(resolution.clone())).is_ok())
    }

    /// Delivers a cancellation wake exactly once.
    pub fn wake_cancelled(&self, approval_id: ApprovalId) -> bool {
        let sender = self
            .inner
            .lock()
            .ok()
            .and_then(|mut inner| inner.remove(&approval_id));
        sender.is_some_and(|sender| sender.send(ApprovalWake::Cancelled).is_ok())
    }

    /// Removes a waiter without waking it (every exit path).
    pub fn disarm(&self, approval_id: ApprovalId) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.remove(&approval_id);
        }
    }

    /// Number of armed waiters (tests assert it returns to 0).
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.lock().map_or(0, |inner| inner.len())
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
```

### 4.3 `RunEngine` fields (+ updates to both constructors and `Debug`)

```rust
    /// Approval/consent gate and DB owner. `None` keeps the pre-1.2
    /// read-only behaviour: side-effect plans refuse `E_TOOL_NOT_ALLOWED`.
    tools: Option<Arc<ToolsBridge>>,
    /// In-process approval waiters (one server process).
    waiters: Arc<ApprovalWaiters>,
    /// One clock for approval timestamps/CAS (tests inject).
    clock: Clock,
    /// Shell allowlist used only for an approved `shell_exec`.
    shell_policy: ShellPolicy,
```

`RunEngine::new` and `with_services` initialize:
`tools: None, waiters: Arc::new(ApprovalWaiters::new()), clock: Arc::new(default_now_unix_secs), shell_policy: ShellPolicy::conservative_default()`.
`Debug` adds `.field("approval_wired", &self.tools.is_some()).field("waiters", &self.waiters.len())`.

```rust
/// Default clock: Unix seconds (the existing approval/TTL unit).
fn default_now_unix_secs() -> u64 {
    Timestamp::now().unix_secs().unwrap_or(0)
}
```

New builders (post-`with_tool_workspace`, signatures stable elsewhere):

```rust
    /// Attaches the approval bridge; production http.rs always does.
    #[must_use]
    pub fn with_tools(mut self, tools: Arc<ToolsBridge>) -> Self {
        self.tools = Some(tools);
        self
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
    /// Returns false when no waiter is armed (legal: recovery/terminal run or a
    /// duplicate/late resolve); the caller logs, never fails the resolve command.
    #[must_use]
    pub fn notify_approval_resolved(&self, resolution: &ApprovalResolution) -> bool {
        self.waiters.wake_resolved(resolution)
    }
```

### 4.4 `core/src/tools_bridge.rs` (new delegates; all keep the gate inside the tools owner)

```rust
    /// Consumes one pending approval whose TTL elapsed (per-wait timer CAS).
    pub fn expire_approval(&self, approval_id: ApprovalId, now_unix_secs: u64) -> Result<bool, Refusal> {
        self.db.with_connection_refusal(|conn| {
            hiverouter_tools::approval::expire(conn, &approval_id.to_string(), now_unix_secs)
                .map_err(|error| error.refusal())
        })
    }

    /// Consumes one pending approval as internally `cancelled` (cancel/kill).
    pub fn cancel_approval(&self, approval_id: ApprovalId, now_unix_secs: u64) -> Result<bool, Refusal> {
        self.db.with_connection_refusal(|conn| {
            hiverouter_tools::approval::cancel_pending(conn, &approval_id.to_string(), now_unix_secs)
                .map_err(|error| error.refusal())
        })
    }

    /// Read-only decision re-read used after a lost expiry/cancel CAS.
    pub fn approval_decision(&self, approval_id: ApprovalId) -> Result<Option<String>, Refusal> {
        self.db.with_connection_refusal(|conn| {
            hiverouter_tools::approval::decision_label(conn, &approval_id.to_string())
                .map_err(|error| error.refusal())
        })
    }

    /// Number of approval rows (any decision) for one run: the wait cap source.
    pub fn approval_count_for_run(&self, run_id: RunId) -> Result<u64, Refusal> {
        self.db.with_connection_refusal(|conn| {
            hiverouter_tools::approval::count_for_run(conn, run_id)
                .map_err(|error| error.refusal())
        })
    }

    /// Verifies (and for `allow_once` consumes) the grant for this exact
    /// `(run, tool, args)`; the engine never opens SQLite itself.
    pub fn verify_tool_grant(
        &self,
        run_id: RunId,
        tool: &str,
        args: &serde_json::Value,
        now_unix_secs: u64,
    ) -> Result<(), Refusal> {
        let tool_id = hiverouter_tools::tool::ToolId::from_id(tool)
            .ok_or(Refusal::ToolNotFound)?;
        self.db.with_connection_refusal(|conn| {
            self.gate
                .verify_grant(conn, run_id, tool_id, args, now_unix_secs)
                .map_err(|error| error.refusal())
        })
    }

    /// Test/CLI seam for a short approval TTL; production keeps
    /// `APPROVAL_TTL_SECS`. Rebuilds the gate with the requested TTL.
    #[must_use]
    pub fn with_approval_ttl(mut self, ttl_secs: u64) -> Self {
        self.gate = hiverouter_tools::approval::ApprovalGate::with_ttl(ttl_secs);
        self
    }
```

### 4.5 `tools/src/approval.rs` (new primitives; single CAS predicate)

```rust
/// Consumes exactly one pending approval whose TTL elapsed. Returns true when
/// this call won the CAS (the row is now `decision='expired'`). The expiry
/// predicate is parsed in Rust (same rule as the WIP `expire_pending`) rather
/// than compared as SQL text.
pub fn expire(conn: &Connection, approval_id: &str, now_unix_secs: u64) -> Result<bool, ToolError> {
    let raw: Option<Option<String>> = conn
        .query_row(
            "SELECT expires_at FROM approvals WHERE approval_id = ?1",
            params![approval_id],
            |row| row.get(0),
        )
        .optional()?;
    let Some(Some(raw_expires)) = raw else {
        return Ok(false);
    };
    let expires = Timestamp::parse(&raw_expires).map_err(|error| {
        ToolError::new(
            Refusal::InternalError,
            format!("stored approval has an invalid expires_at: {error}"),
        )
    })?;
    if !expires
        .unix_secs()
        .map(|at| now_unix_secs >= at)
        .unwrap_or(true)
    {
        return Ok(false);
    }
    let decided_at = Timestamp::from_unix_secs(now_unix_secs);
    let changed = conn.execute(
        "UPDATE approvals SET decision = 'expired', decided_at = ?1
         WHERE approval_id = ?2 AND decision IS NULL",
        params![decided_at.as_str(), approval_id],
    )?;
    Ok(changed == 1)
}

/// Consumes one pending approval as internally `cancelled` (never a wire
/// decision). Returns true when this call won the CAS.
pub fn cancel_pending(conn: &Connection, approval_id: &str, now_unix_secs: u64) -> Result<bool, ToolError> {
    let decided_at = Timestamp::from_unix_secs(now_unix_secs);
    let changed = conn.execute(
        "UPDATE approvals SET decision = 'cancelled', decided_at = ?1
         WHERE approval_id = ?2 AND decision IS NULL",
        params![decided_at.as_str(), approval_id],
    )?;
    Ok(changed == 1)
}

/// Raw stored decision label (`allow_once`, `allow_session`, `deny`,
/// `expired`, `cancelled`) or `None` while pending.
pub fn decision_label(conn: &Connection, approval_id: &str) -> Result<Option<String>, ToolError> {
    conn.query_row(
        "SELECT decision FROM approvals WHERE approval_id = ?1",
        params![approval_id],
        |row| row.get::<_, Option<String>>(0),
    )
    .optional()
    .map(Option::flatten)
}

/// Number of approval rows (all decisions) for one run.
pub fn count_for_run(conn: &Connection, run_id: RunId) -> Result<u64, ToolError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM approvals WHERE run_id = ?1",
        params![run_id.to_string()],
        |row| row.get(0),
    )?;
    u64::try_from(count).map_err(|_| ToolError::new(Refusal::InternalError, "approval count overflow"))
}
```

**Unique partial index (E14/D11), added inside `migrate` after the two existing indexes:**

```sql
CREATE UNIQUE INDEX IF NOT EXISTS approvals_one_pending_per_call
    ON approvals (run_id, tool_call_id) WHERE decision IS NULL;
```

*Risk:* an existing dirty DB with duplicate pending rows for one `(run_id, tool_call_id)` would make `migrate` fail; `build_tools` only logs migration failure and continues `[READ http.rs:108-113]`, so boot remains available but the producer's in-Rust dedupe `[READ tools_bridge.rs:300-313]` is the only guard. Flag in §15; do not delete audit rows to force the index.

**WIP merge:** rebase the WIP expiry functions onto this file. Keep `expire_orphaned_pending` (boot) untouched — it is deliberately unconditional; keep `expire_pending` but have its loop call the new single-row `expire(conn, &id, now)` instead of duplicating the CAS, so the TTL predicate exists once in the per-row `expire` (the boot orphan path keeps its own unconditional statement because it must ignore the TTL). The new per-row `expire`/`cancel_pending`/`decision_label`/`count_for_run` are additive. The WIP `resolve` now reports a stored `expired` decision as `E_APPROVAL_TIMEOUT`; the engine's lost-CAS re-read maps label `expired` to the timeout path (§5.4), so both sides agree.

### 4.6 `tools/src/executor.rs` — async side-effect executor (**CORRECTION:** async, because `shell::execute` is async)

```rust
/// Executes exactly the approved side-effect tools (`file_write`, `shell_exec`)
/// inside the jail, after the registry re-validated tier/schema. Everything
/// else is a named `E_TOOL_NOT_ALLOWED` refusal. Reachable only after
/// `ApprovalGate::verify_grant`; this function itself performs no approval.
pub async fn execute_side_effect(
    jail: &WorkspaceJail,
    shell_policy: &ShellPolicy,
    registry: &ToolRegistry,
    tier: AgentTier,
    tool: &str,
    args: &serde_json::Value,
    cancellation: Option<&CancellationToken>,
) -> Result<ExecutedTool, ToolError> {
    let contract = registry.resolve_call(tier, tool, args)?; // fail closed before I/O
    match contract.id {
        ToolId::FileWrite => execute_file_write(jail, args),
        ToolId::ShellExec => execute_shell_exec(jail, shell_policy, args, cancellation).await,
        _ => Ok(ExecutedTool::Refused {
            code: Refusal::ToolNotAllowed.code().to_owned(),
            detail: format!("tool {tool:?} has no approved side-effect executor"),
        }),
    }
}

fn execute_file_write(jail: &WorkspaceJail, args: &serde_json::Value) -> Result<ExecutedTool, ToolError> {
    let write: FileWriteArgs = serde_json::from_value(args.clone())
        .map_err(|error| ToolError::new(Refusal::InternalError,
            format!("file_write arguments do not deserialize after schema validation: {error}")))?;
    if let Some(expected) = write.expected_sha256.as_deref() {
        match jail.read(&write.path) {
            Ok(existing) => {
                let actual = crate::sha256::sha256_hex(&existing);
                if !actual.eq_ignore_ascii_case(expected) {
                    // D-2: no frozen stale-write code exists; the closest frozen
                    // precondition refusal is E_SCHEMA_INVALID (documented).
                    return Err(ToolError::new(Refusal::SchemaInvalid,
                        "expected_sha256 does not match the current file; stale write refused"));
                }
            }
            Err(error) if error.refusal() == Refusal::PathNotFound => {}
            Err(error) => return Err(error),
        }
    }
    let path = jail.write(&write.path, write.content.as_bytes())?;
    Ok(ExecutedTool::Ok {
        text: serde_json::json!({
            "path": path.display().to_string(),
            "bytes": write.content.len(),
            "sha256": crate::sha256::sha256_hex(write.content.as_bytes()),
        })
        .to_string(),
    })
}

async fn execute_shell_exec(
    jail: &WorkspaceJail,
    policy: &ShellPolicy,
    args: &serde_json::Value,
    cancellation: Option<&CancellationToken>,
) -> Result<ExecutedTool, ToolError> {
    let exec: ShellExecArgs = serde_json::from_value(args.clone())
        .map_err(|error| ToolError::new(Refusal::InternalError,
            format!("shell_exec arguments do not deserialize after schema validation: {error}")))?;
    let mut request = ShellRequest::new(exec.program, exec.args);
    if let Some(cwd) = exec.cwd {
        request = request.with_cwd(cwd);
    }
    let output = crate::shell::execute(jail, policy, &request, cancellation).await?;
    let text = serde_json::to_string(&output)
        .map_err(|error| ToolError::new(Refusal::InternalError,
            format!("shell output could not be serialized: {error}")))?;
    Ok(ExecutedTool::Ok { text })
}
```

`tools/src/lib.rs`: extend `pub use executor::{execute_read_only, execute_side_effect, ExecutedTool};` and keep `shell::execute` already exported `[READ lib.rs]`.

---

## 5. Proposed engine diff (NOT APPLIED) — `core/src/execution.rs`

### 5.1 `dispatch_inner`: make the run `running` before leaves (D2)

Insert after task creation and before `for leaf_id in plan.leaf_order()?` (current line 649):

```rust
        // D2: run_state only accepts Suspend from Running; the run is planned
        // and now starts executing leaves. One transition, one effect.
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
```

### 5.2 `execute_tool_turn` — async + suspension branch

Signature change: `async fn execute_tool_turn(...)`; call site in `execute_task` becomes
`let turn = match self.execute_tool_turn(run, task, &final_reply, token).await { ... }` (current `:1021`).
Per-call body change (inside the `Ok(plan)` arm, current `:768-790`):

```rust
                    Ok(plan) => {
                        let execution = match tool_loop::validate_plan(
                            &self.tool_registry,
                            AgentTier::WorkerAgent,
                            &plan,
                        ) {
                            Err(refusal) => ToolExecution::refused(/* unchanged */),
                            Ok(()) => {
                                self.publish_tool_call(run, task, &plan);
                                let needs_approval = tool_loop::requires_approval(
                                    &self.tool_registry,
                                    &plan.tool,
                                );
                                if needs_approval && self.tools.is_some() && self.tool_workspace.is_some() {
                                    // D12: count the attempt before the wait so deny/timeout
                                    // cannot retry for free.
                                    self.db
                                        .increment_run_tool_counters(run.id, 0, 1)
                                        .map_err(|error| error.refusal())?;
                                    precounted = precounted.saturating_add(1);
                                    let execution = self
                                        .suspend_for_approval(run, task, call, &plan, turn, token)
                                        .await?; // terminal waits finalize their own TOOL_RESULT
                                    executed.push((plan, execution.clone()));
                                    execution
                                } else {
                                    // No bridge/jail (tests, pre-1.2 engines) keeps the
                                    // frozen E_TOOL_NOT_ALLOWED refusal for side effects.
                                    self.execute_tool_plan(&plan)
                                }
                            }
                        };
                        // Every branch that returns an execution pushes exactly one
                        // `(plan, execution)` pair here; terminal wait paths returned Err
                        // after finalizing their own TOOL_RESULT and do not reach this push.
                        executed.push((plan, execution.clone()));
                        execution
                    }
```

Exact ordering inside `execute_tool_turn`:
1. counters read; `next_turn`; per index compute `attempted = used_calls + index + precounted`; `check_call_budget(attempted)`.
2. `plan_call` → `validate_plan` → `publish_tool_call`.
3. Read plan → `execute_tool_plan` (sync, unchanged).
4. Side-effect plan with bridge+jail → pre-increment calls by 1 → `suspend_for_approval(...).await`.
5. Push `(plan, execution.clone())` once for every planned call (the suspension branch returns the approve/deny execution; terminal branches return `Err` and have already finalized `TOOL_RESULT`).
6. End: `increment_run_tool_counters(run.id, 1, calls_in_turn.saturating_sub(precounted))`, where `calls_in_turn` still counts every attempt (including terminal waits).

### 5.3 New helper: the suspend/wait/apply sequence

```rust
    /// Commits one approval request, arms the waiter, persists `awaiting_approval`,
    /// publishes `RUN_STATUS`/`NODE_STATE`/`APPROVAL_REQUEST`, then waits for
    /// resolution / TTL / cancellation. Returns the approve/deny `ToolExecution`
    /// for the caller's transcript; **terminal** waits (timeout/cancel/verify
    /// failure) finalize their own `TOOL_RESULT` + `tool_turns` row and return
    /// `Err`, so a planned call always gets exactly one result (D13).
    async fn suspend_for_approval(
        &self,
        run: &Run,
        task: &TaskRecord,
        call: &hiverouter_provider::tool_assembler::AssembledToolCall,
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
        let Some(workspace) = self.tool_workspace.as_ref() else {
            return Ok(ToolExecution::refused(
                plan.tool_call_id.clone(),
                Refusal::FileTreeUnavailable.code(),
                "no workspace jail is configured; the side effect did not execute".to_owned(),
            ));
        };
        // Wait cap: a 9th request is a named refusal, not a new code (D9/D12).
        let waits = tools.approval_count_for_run(run.id).map_err(|error| error.refusal())?;
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
            .request_model_tool_approval(run.id, task.id, &plan.tool_call_id, &plan.tool, &plan.args)
            .map_err(|error| error.refusal())?;

        // Arm BEFORE publishing the request: a fast resolve must never be lost (E7).
        let mut receiver = self.waiters.arm(event.approval_id).map_err(|_| Refusal::InternalError)?;
        let _armed = ArmedApproval { waiters: Arc::clone(&self.waiters), approval_id: event.approval_id };

        // Persist + publish the suspension only after the row is committed and the
        // waiter is armed. If any write fails, consume the row and fail closed.
        let current_run = self.store.get_run(run.id).map_err(|error| error.refusal())?;
        let current_task = self.store.get_task(task.id).map_err(|error| error.refusal())?;
        let run_transition = run_state::apply_run(current_run.status, StateEvent::Suspend)
            .map_err(|error| error.refusal())?;
        let task_transition = run_state::apply_task(current_task.state, StateEvent::Suspend)
            .map_err(|error| error.refusal())?;
        if let Err(refusal) = self.persist_approval_transitions(run, task, run_transition, task_transition) {
            let _ = tools.expire_approval(event.approval_id, now);
            return Err(refusal);
        }
        self.publish(Event::ApprovalRequest(event.clone()), Some(run.id), None);

        // Re-read once: a request can already be decided only through a
        // concurrent driver/replay; apply the winner instead of waiting (E7).
        if let Some(label) = tools.approval_decision(event.approval_id).map_err(|e| e.refusal())? {
            return self
                .apply_committed_decision(run, task, call, plan, turn, token, workspace, tools, event.approval_id, &label)
                .await;
        }

        let expires = event
            .expires_at
            .as_ref()
            .and_then(Timestamp::unix_secs)
            .unwrap_or(now.saturating_add(tools.approval_gate().ttl_secs()));
        let remaining = expires.saturating_sub(self.now_unix_secs()).max(1);
        let wait = tokio::time::timeout(std::time::Duration::from_secs(remaining), receiver);
        let wake = tokio::select! {
            biased;
            _ = token.cancelled() => None,
            result = wait => Some(result),
        };

        match wake {
            None => self
                .finish_cancelled_wait(run, task, call, plan, turn, tools, event.approval_id)
                .await,
            Some(Err(_elapsed)) => {
                self.finish_timeout_wait(run, task, call, plan, turn, tools, event.approval_id, token)
                    .await
            }
            Some(Ok(Err(_closed))) => {
                // Sender dropped without a wake: fail closed, consume the row.
                let _ = tools.cancel_approval(event.approval_id, self.now_unix_secs());
                self.finish_cancelled_wait(run, task, call, plan, turn, tools, event.approval_id)
                    .await
            }
            Some(Ok(Ok(ApprovalWake::Cancelled))) => {
                self.finish_cancelled_wait(run, task, call, plan, turn, tools, event.approval_id)
                    .await
            }
            Some(Ok(Ok(ApprovalWake::Resolved(resolution)))) => {
                if token.is_cancelled() {
                    // E13: a decision that arrives after cancellation must not execute.
                    let _ = tools.cancel_approval(event.approval_id, self.now_unix_secs());
                    return self
                        .finish_cancelled_wait(run, task, call, plan, turn, tools, event.approval_id)
                        .await;
                }
                match resolution.decision {
                    ApprovalDecision::AllowOnce | ApprovalDecision::AllowSession => {
                        self.resume_and_execute(run, task, plan, workspace, token).await
                    }
                    ApprovalDecision::Deny => self.resume_denied(run, task, plan),
                }
            }
        }
    }

    /// Applies a decision that was already committed when the engine armed the
    /// waiter (retry/replay race), without fabricating an `APPROVAL_RESOLVED`.
    async fn apply_committed_decision(
        &self,
        run: &Run,
        task: &TaskRecord,
        call: &hiverouter_provider::tool_assembler::AssembledToolCall,
        plan: &ToolCallPlan,
        turn: u32,
        token: &CancellationToken,
        workspace: &WorkspaceJail,
        tools: &ToolsBridge,
        approval_id: ApprovalId,
        label: &str,
    ) -> Result<ToolExecution, Refusal> {
        match hiverouter_tools::approval::ApprovalDecision::from_label(label) {
            Some(ApprovalDecision::AllowOnce | ApprovalDecision::AllowSession) => {
                self.resume_and_execute(run, task, plan, workspace, token).await
            }
            Some(ApprovalDecision::Deny) => self.resume_denied(run, task, plan),
            None if label == "expired" => {
                self.finish_timeout_wait(run, task, call, plan, turn, tools, approval_id, token)
                    .await
            }
            None => {
                self.finish_cancelled_wait(run, task, call, plan, turn, tools, approval_id)
                    .await
            }
        }
    }

    /// Timer path: expire CAS first; if lost, re-read the committed decision and
    /// apply the winner; if won, one timeout TOOL_RESULT + failed run
    /// `E_APPROVAL_TIMEOUT`; dispatch skips the duplicate terminal frame.
    async fn finish_timeout_wait(
        &self,
        run: &Run,
        task: &TaskRecord,
        call: &hiverouter_provider::tool_assembler::AssembledToolCall,
        plan: &ToolCallPlan,
        turn: u32,
        tools: &ToolsBridge,
        approval_id: ApprovalId,
        token: &CancellationToken,
    ) -> Result<ToolExecution, Refusal> {
        let now = self.now_unix_secs();
        if tools.expire_approval(approval_id, now).map_err(|error| error.refusal())? {
            let execution = ToolExecution::timed_out(plan.tool_call_id.clone());
            self.finish_tool_call(run, task, call, &execution, turn)?;
            let run_transition = run_state::apply_run(AgentState::AwaitingApproval, StateEvent::Timeout)
                .map_err(|error| error.refusal())?;
            let task_transition = run_state::apply_task(AgentState::AwaitingApproval, StateEvent::Timeout)
                .map_err(|error| error.refusal())?;
            let error_code = run_transition
                .effect
                .refusal()
                .map(|refusal| refusal.code().to_owned());
            self.store.set_run_status(run.id, run_transition.to, error_code.as_deref())?;
            self.publish(
                Event::RunStatus(RunStatusPayload {
                    run_id: run.id,
                    status: run_transition.to,
                    error_code: error_code.clone(),
                }),
                Some(run.id),
                None,
            );
            self.store.update_task_state(task.id, task_transition.to)?;
            let _ = self.graph.set_state(task.id, task_transition.to);
            self.publish_node(run.id, task.id, task_transition.to);
            return Err(Refusal::ApprovalTimeout);
        }
        // Lost the CAS: a decision/cancel/expiry by another path won. Apply it.
        match tools.approval_decision(approval_id).map_err(|error| error.refusal())? {
            Some(label) => {
                let workspace = self.tool_workspace.as_ref().ok_or(Refusal::InternalError)?;
                self.apply_committed_decision(
                    run, task, call, plan, turn, token, workspace, tools, approval_id, &label,
                )
                .await
            }
            None => Err(Refusal::InternalError),
        }
    }

    /// Cancel path: consume the row, mark the task cancelled once, finish the
    /// cancelled TOOL_RESULT + row, return `E_TOOL_CANCELLED`.
    async fn finish_cancelled_wait(
        &self,
        run: &Run,
        task: &TaskRecord,
        call: &hiverouter_provider::tool_assembler::AssembledToolCall,
        plan: &ToolCallPlan,
        turn: u32,
        tools: &ToolsBridge,
        approval_id: ApprovalId,
    ) -> Result<ToolExecution, Refusal> {
        let _ = tools.cancel_approval(approval_id, self.now_unix_secs());
        let fresh = self.store.get_task(task.id).map_err(|error| error.refusal())?;
        if !fresh.state.is_terminal() && fresh.state == AgentState::AwaitingApproval {
            let transition = run_state::apply_task(fresh.state, StateEvent::Cancel)
                .map_err(|error| error.refusal())?;
            self.store.update_task_state(task.id, transition.to)?;
            let _ = self.graph.set_state(task.id, transition.to);
            self.publish_node(run.id, task.id, transition.to);
        }
        let execution = ToolExecution::cancelled(plan.tool_call_id.clone());
        self.finish_tool_call(run, task, call, &execution, turn)?;
        Err(Refusal::ToolCancelled)
    }

Companion helpers (all in `execution.rs`, additive):

```rust
    fn now_unix_secs(&self) -> u64 { (self.clock)() }

    /// Applies the run+task Suspend transitions exactly once and publishes
    /// `RUN_STATUS awaiting_approval` + `NODE_STATE awaiting_approval`.
    fn persist_approval_transitions(
        &self,
        run: &Run,
        task: &TaskRecord,
        run_transition: run_state::RunTransition,
        task_transition: run_state::TaskTransition,
    ) -> Result<(), Refusal> {
        self.store.set_run_status(run.id, run_transition.to, None)?;
        self.publish(Event::RunStatus(RunStatusPayload {
            run_id: run.id, status: run_transition.to, error_code: None,
        }), Some(run.id), None);
        self.store.update_task_state(task.id, task_transition.to)?;
        let _ = self.graph.set_state(task.id, task_transition.to);
        self.publish_node(run.id, task.id, task_transition.to);
        Ok(())
    }

    /// Applies run+task Resume and publishes `RUN_STATUS running` + NODE running.
    fn resume_transitions(&self, run: &Run, task: &TaskRecord) -> Result<(), Refusal> {
        let run_transition = run_state::apply_run(AgentState::AwaitingApproval, StateEvent::Resume)
            .map_err(|error| error.refusal())?;
        let task_transition = run_state::apply_task(AgentState::AwaitingApproval, StateEvent::Resume)
            .map_err(|error| error.refusal())?;
        self.store.set_run_status(run.id, run_transition.to, None)?;
        self.publish(Event::RunStatus(RunStatusPayload { run_id: run.id, status: run_transition.to, error_code: None }), Some(run.id), None);
        self.store.update_task_state(task.id, task_transition.to)?;
        let _ = self.graph.set_state(task.id, task_transition.to);
        self.publish_node(run.id, task.id, task_transition.to);
        Ok(())
    }

    /// Approve path: persist Running, verify the single-use grant, execute the
    /// approved side effect once, return its bounded `ToolExecution`.
    async fn resume_and_execute(
        &self,
        run: &Run,
        task: &TaskRecord,
        plan: &ToolCallPlan,
        workspace: &WorkspaceJail,
        token: &CancellationToken,
    ) -> Result<ToolExecution, Refusal> {
        self.resume_transitions(run, task)?;
        let now = self.now_unix_secs();
        let tools = self.tools.as_ref().ok_or(Refusal::InternalError)?;
        if let Err(refusal) = tools.verify_tool_grant(run.id, &plan.tool, &plan.args, now) {
            // Wrong/tampered args or a consumed grant: execute nothing, fail the
            // run by name (D4 in [PLAN §13]); task failure is applied by
            // execute_task's existing error path.
            return Err(refusal);
        }
        let executed = execute_side_effect(
            workspace,
            &self.shell_policy,
            &self.tool_registry,
            AgentTier::WorkerAgent,
            &plan.tool,
            &plan.args,
            Some(token),
        )
        .await;
        Ok(match executed {
            Ok(ExecutedTool::Ok { text }) => ToolExecution::ok(plan.tool_call_id.clone(), tool_loop::bounded_tool_text(&text)),
            Ok(ExecutedTool::Refused { code, detail }) => ToolExecution::refused(plan.tool_call_id.clone(), code, detail),
            Err(error) => ToolExecution::refused(plan.tool_call_id.clone(), error.code(), error.detail().to_owned()),
        })
    }
```

No further correction is needed: `resume_and_execute` receives the caller's `&CancellationToken` from `suspend_for_approval` (the approve wake branch passes `token`).

```rust
    /// Deny path: persist Running and return the denied execution (no side effect).
    fn resume_denied(&self, run: &Run, task: &TaskRecord, plan: &ToolCallPlan) -> Result<ToolExecution, Refusal> {
        let run_transition = run_state::apply_run(AgentState::AwaitingApproval, StateEvent::Deny)
            .map_err(|error| error.refusal())?;
        let task_transition = run_state::apply_task(AgentState::AwaitingApproval, StateEvent::Deny)
            .map_err(|error| error.refusal())?;
        self.store.set_run_status(run.id, run_transition.to, None)?;
        self.publish(Event::RunStatus(RunStatusPayload { run_id: run.id, status: run_transition.to, error_code: None }), Some(run.id), None);
        self.store.update_task_state(task.id, task_transition.to)?;
        let _ = self.graph.set_state(task.id, task_transition.to);
        self.publish_node(run.id, task.id, task_transition.to);
        Ok(ToolExecution::denied(plan.tool_call_id.clone()))
    }
```

The terminal wait helpers (`finish_timeout_wait` / `finish_cancelled_wait`) are defined above; they own exactly one `TOOL_RESULT` + row per planned call (D13).

### 5.4 Timeout / cancel / lost-CAS handling (exact algorithm)

Timer arm fires:
1. If `token.is_cancelled()` → treat as cancel (below).
2. `tools.expire_approval(approval_id, now)`:
   - `Ok(true)` (we won): build `ToolExecution::timed_out(call_id)`; `self.finish_tool_call(run, task, call, &execution, turn)?` (needs the `AssembledToolCall`; change `suspend_for_approval` to also take `call: &AssembledToolCall` from `execute_tool_turn`); apply task `Awaiting+Timeout→Failed` and run `Awaiting+Timeout→Failed` via `run_state`, persist both, publish `NODE_STATE failed` and the terminal `RUN_STATUS failed` with `error_code = Some(E_APPROVAL_TIMEOUT)` (from `Effect::refusal()`); return `Err(Refusal::ApprovalTimeout)`.
   - `Ok(false)` (lost CAS): `tools.approval_decision(approval_id)`:
     - `Some("allow_once" | "allow_session")` → resume/execute exactly as an approve wake (same code path; no `APPROVAL_RESOLVED` frame — the operator's resolve command owns it);
     - `Some("deny")` → denied path;
     - `Some("expired")` → timeout path (this process timed out or boot reaper won): finish `timed_out` + fail run `E_APPROVAL_TIMEOUT`;
     - `Some("cancelled")` → cancellation path;
     - `None` → `E_INTERNAL_ERROR` (CAS lost means a decision exists; fail closed).
   - `Err` → finalize a `timed_out` TOOL_RESULT + return the error (fail closed).
3. Cancel arm or `ApprovalWake::Cancelled`: `tools.cancel_approval(approval_id, now)` (ignore `Ok(false)`; decision may have won the race); if a decision won (`Ok(false)`) and it is `allow_*`, still do **not** execute (`token.is_cancelled()`), finish `cancelled`; apply task `Awaiting+Cancel→Cancelled` (run terminal frame comes from `cancel_run` or `dispatch`); finish `TOOL_RESULT cancelled` + row; return `Err(Refusal::ToolCancelled)`.

Cancellation of the wait must be reachable two ways: (a) `CANCEL_RUN` cancels the token in `RunRegistry` `[READ execution.rs:499-519]`; (b) `KILL_WORKER` on an awaiting task cancels the run token (new branch, below). No additional channel is needed.

### 5.5 `cancel_run` / `kill_worker` diffs

```rust
    pub fn cancel_run(&self, run_id: RunId) -> Result<(), Refusal> {
        let run = self.store.get_run(run_id)?;
        if run.status.is_terminal() {
            return Err(Refusal::RunTerminal);
        }
        if !self.tokens.cancel(run_id) {
            return Err(Refusal::RunTerminal);
        }
        // D6: validate the edge with the authority, then persist/publish it once.
        let transition = run_state::apply_run(run.status, StateEvent::Cancel)
            .map_err(|error| error.refusal())?;
        self.store.set_run_status(run_id, transition.to, None)?;
        self.publish(
            Event::RunStatus(RunStatusPayload { run_id, status: transition.to, error_code: None }),
            Some(run_id),
            None,
        );
        Ok(())
    }

    pub fn kill_worker(&self, task_id: TaskId) -> Result<(), Refusal> {
        let task = self.store.get_task(task_id)?;
        if task.state.is_terminal() {
            return Err(Refusal::TaskTerminal);
        }
        if task.state == AgentState::AwaitingApproval {
            // The waiter owns the task transition + approval consumption; the
            // token cancel wakes it. Exit is single-owned (no double NODE_STATE).
            self.tokens.cancel(task.run_id);
            return Ok(());
        }
        if task.state != AgentState::Running {
            return Err(Refusal::KillDenied);
        }
        let transition = run_state::apply_task(task.state, StateEvent::Kill)
            .map_err(|error| error.refusal())?; // Running+Kill -> Killed
        self.store.update_task_state(task_id, transition.to)?;
        let _ = self.graph.set_state(task_id, transition.to);
        self.publish_node(task.run_id, task_id, transition.to);
        Ok(())
    }
```

### 5.6 `dispatch` — terminal ownership guard (D7)

Replace the top of `dispatch` (current `:539-560`) with:

```rust
    async fn dispatch(self: Arc<Self>, run: Run, token: CancellationToken) {
        let outcome = self.dispatch_inner(&run, &token).await;
        let persisted = self.store.get_run(run.id).map(|current| current.status).unwrap_or(run.status);
        let final_status = if persisted.is_terminal() {
            // A suspension terminal path (timeout) or CANCEL_RUN already
            // persisted + published the terminal frame; do not duplicate it.
            persisted
        } else {
            let (status, error_code) = match &outcome {
                Ok(()) => (AgentState::Completed, None),
                Err(refusal) if *refusal == Refusal::ToolCancelled => (AgentState::Cancelled, None),
                Err(refusal) => (AgentState::Failed, Some(refusal.code().to_owned())),
            };
            if let Err(error) = self.store.set_run_status(run.id, status, error_code.as_deref()) {
                tracing::error!(code = error.code(), "failed to persist final run status");
            }
            self.publish(
                Event::RunStatus(RunStatusPayload { run_id: run.id, status, error_code }),
                Some(run.id),
                None,
            );
            status
        };
        // snapshot block unchanged but uses `final_status`;
        // set_active(None), tokens.remove(run.id), executor.finish_run(run.id).await unchanged.
    }
```

### 5.7 `execute_task` — await the async turn + treat wait refusals

- `let turn = match self.execute_tool_turn(run, task, &final_reply, token).await { ... }` (only ``.await` changes at `:1021`).
- `Err(Refusal::ApprovalTimeout)` from a wait must leave the task `failed` and the run terminal; the wait helper already persisted both, so the existing `Err(refusal)` arm calls `fail_task(Failed)` again (idempotent write + duplicate `NODE_STATE failed`). **Refine:** in that arm, if the failure is `ApprovalTimeout`, skip `fail_task` because the wait helper applied `FailTaskOnApprovalTimeout`; otherwise keep the existing behavior. Exact guard:
  ```rust
  Err(refusal) => {
      if refusal != Refusal::ApprovalTimeout {
          let state = if refusal == Refusal::ToolCancelled { AgentState::Cancelled } else { AgentState::Failed };
          self.fail_task(run, task, state)?;
      }
      return Err(refusal);
  }
  ```
- `Err(Refusal::ToolCancelled)` from a wait: the helper applied task `Cancelled`; skip `fail_task` too (otherwise `update_task_state` would overwrite Cancelled→Cancelled; graph/event duplicate). Same guard shape.

### 5.8 `ArmedApproval` RAII guard (module-private, after `ApprovalWaiters`)

```rust
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
```

---

## 6. Lock / channel design (explicit)

| Concern | Design | Evidence |
|---|---|---|
| Waiter registry | `std::sync::Mutex<HashMap<ApprovalId, oneshot::Sender<ApprovalWake>>>`; map operations only; never held across `.await` | `[READ execution.rs:289]` uses the same pattern for `active_run`; `[PLAN §4, D2]` |
| Arming order | commit row (`request_model_tool_approval`) → `waiters.arm` → persist/publish `APPROVAL_REQUEST` → re-read decision → select | E7 lost-wake guard `[PLAN E7]`; the re-read also covers a producer retry that returns a row decided by a concurrent driver |
| Wake path | WS handler (same tokio runtime, sync function) calls `state.engine.notify_approval_resolved(&resolution)` after publishing ACK/`APPROVAL_RESOLVED`; `wake_resolved` removes the sender under the mutex and sends; single consumer | `[READ ws.rs:465-483]`; `[PLAN §4, §6]` |
| Disarm | `ArmedApproval` Drop on every exit; `wake_*` already removed the map entry, disarm is a no-op then | `[PLAN §8 bug 8]` |
| Sender drop | Receiver resolves `Err(RecvError)`; the engine consumes the row as `cancelled` and returns `E_TOOL_CANCELLED`, never hangs | fail-closed rule |
| Timer | `tokio::time::timeout(remaining, &mut receiver)` inside `tokio::select!` with `token.cancelled()`; `remaining` from `request.expires_at` (D4); `with_clock` only supplies CAS timestamps | `[READ shell.rs:173-178]` for cancellation idiom; `[READ tool_loop.rs:14]` caps |
| DB vs lock | all bridge calls are short synchronous SQLite operations on `Database`'s own connection mutex `[READ db.rs:241-246]`; the waiters mutex is never taken while a DB call is in flight (arm before/after commit; wake is independent) | no lock inversion observed |
| RunEngine globals | `active_run` mutex stays untouched; no `suspended_runs` map is needed because the wait is inline (D1) | `[READ execution.rs:289,381-389]` |
| Cancellation | token cancel is the single cross-task signal; `RunRegistry::cancel` already used by `cancel_run` `[READ :504]`; `kill_worker` reuses it for awaiting tasks | no new channel |
| Multiple runs | single active run enforced by `store.latest_active_run` (includes awaiting) + engine check `[READ execution.rs:426-428; PLAN Q13]` | no multi-run waiter sharing |

**Deadlock audit:** the only await points while a waiter is armed are the select (receiver/timer/token) and, after wake, the bridge/verify/executor calls. Mutexes held across those: none of the engine's `std::sync::Mutex` (waiters removed before await by `wake_*`; `active_run` not touched). Tool bridge ledger mutex `[READ tools_bridge.rs:223]` is unrelated to the engine wait and is not taken by the engine.

---

## 7. Wiring / RunEngine field injection (`core/src/http.rs`)

**Order change (all three constructors):** `build_tools` must run before `build_engine`, because the engine now needs `Arc<ToolsBridge>`.

```rust
fn build_engine(
    db: Arc<Database>,
    bus: Arc<EventBus>,
    graph: Arc<GraphAuthority>,
    config: &Config,
    templates: Arc<TemplateRegistry>,
    snapshots: Arc<SnapshotStore>,
    tools: Arc<ToolsBridge>,
    wire_workspace_jail: bool,
) -> Arc<RunEngine> {
    let mut engine = RunEngine::new(db, bus, graph, build_executor(config), templates)
        .with_snapshot_store(snapshots)
        .with_tools(tools);
    if wire_workspace_jail {
        match WorkspaceJail::open(&config.workspace_root) { Ok(jail) => engine = engine.with_tool_workspace(jail), Err(error) => { /* existing warn */ } }
    }
    Arc::new(engine)
}
```

In `AppState::new`, `with_graph`, `with_services`:
1. `let tools = build_tools(&db, &config);`
2. `let engine = build_engine(..., tools.clone(), <wire flag>);`
3. keep `tools` in `AppState` as today.

`with_services` keeps `wire_workspace_jail = false` (test seam unchanged) but still wires `with_tools`, so AppState-level WS tests exercise the resolver/waiter path (deny/recovery) without touching disk; side-effect execution tests use the engine builder directly with a `TempDir` jail (§12.1). Production constructors keep `true` `[READ http.rs:198,235,269]`.

**Validation seam ownership:** the bridge's TTL/workspace is built by `build_tools`; the engine gets only `Arc<ToolsBridge>` (single tools owner). No `RunEngine` ledger field (D5). No `main.rs` change (recovery order stays `recover_on_boot` → `AppState` → `serve`) `[READ main.rs:81-123]`.

---

## 8. State transitions, effects and event order (authority: `run_state`)

`[READ run_state.rs:465-620; PLAN §5]` Applied by the engine exactly once; `persist_*` helpers in §5.3.

| # | Trigger | Run transition / persist / publish | Task transition / persist / publish | Tool/event |
|---|---|---|---|---|
| S1 | model returns an approvable call; bridge+jail present | `apply_run(Running,Suspend)` → `AwaitingApproval` (RequestApproval): `set_run_status` + `RUN_STATUS awaiting_approval` | `apply_task(Running,Suspend)` → `AwaitingApproval` (RequestTaskApproval): `update_task_state` + graph + `NODE_STATE awaiting_approval` | `TOOL_CALL` (already published) then `APPROVAL_REQUEST` with nonce/ids/args_sha256/expires_at |
| S2 | approve wake (AllowOnce/AllowSession) | `apply_run(Awaiting,Resume)` → `Running` (ResumeFromApproval): persist + `RUN_STATUS running` | `apply_task(Awaiting,Resume)` → `Running`: persist + `NODE_STATE running` | `verify_tool_grant` → execute once → caller `finish_tool_call` publishes `TOOL_RESULT ok` + `tool_turns` row → transcript → `continue_with_tools` |
| S3 | deny wake | `apply_run(Awaiting,Deny)` → `Running` (FeedDeniedToolResult) | `apply_task(Awaiting,Deny)` → `Running` | caller `finish_tool_call` publishes `TOOL_RESULT denied` (+ row); loop continues |
| S4 | timer wins expiry CAS | `apply_run(Awaiting,Timeout)` → `Failed` (FailOnApprovalTimeout, `Effect::refusal()=E_APPROVAL_TIMEOUT`): persist failed + **terminal** `RUN_STATUS failed error_code=E_APPROVAL_TIMEOUT` | `apply_task(Awaiting,Timeout)` → `Failed`: persist + `NODE_STATE failed` | one `TOOL_RESULT timeout` + row; `Err(ApprovalTimeout)`; `dispatch` skips duplicate frame (D7) |
| S5 | timer loses CAS to a decision | no suspend state change left; apply the winner (S2/S3/S4-expired/S6-cancelled) | same | no `APPROVAL_RESOLVED` fabricated by the engine |
| S6 | `CANCEL_RUN` while waiting | `cancel_run` applied `apply_run(status,Cancel)` → `Cancelled` + terminal frame; waiter applies task `awaiting+Cancel→Cancelled` | task persisted Cancelled + `NODE_STATE cancelled` | `TOOL_RESULT cancelled` + row; `Err(ToolCancelled)`; `dispatch` skips duplicate frame |
| S7 | `KILL_WORKER` on awaiting task | token cancel; waiter S6 task transition; run terminal via `dispatch` (non-terminal path) | single NODE update by waiter | `TOOL_RESULT cancelled`; `Err(ToolCancelled)` |
| S8 | boot after crash/exit | WIP recovery: non-terminal runs → `interrupted`, awaiting runs get `error_code=E_APPROVAL_TIMEOUT` `[WIP recovery.rs]` | tasks keep last persisted state (no resume) | boot reaper expires pending rows `[WIP http/tools_bridge/approval]`; no `APPROVAL_RESOLVED` |
| S9 | duplicate/late resolve | no state change (waiter consumed). Gate behavior (WIP re-read at 18:30): replay of a used nonce on a non-expired decision → `E_APPROVAL_REPLAY`; a decision `expired` by TTL/boot reaper → `E_APPROVAL_TIMEOUT`; unknown id → `E_APPROVAL_NOT_FOUND` | none | WS publishes ERROR; engine not woken (`notify` false, logged) |
| S10 | wrong/tampered args at resume | resume persisted Running, then `verify_tool_grant` fails → `Err(E_APPROVAL_REQUIRED)` | existing `fail_task(Failed)` | no side effect; `dispatch` terminalizes failed `E_APPROVAL_REQUIRED` |
| S11 | no bridge/jail (tests, pre-1.2) | no suspension | none | frozen `E_TOOL_NOT_ALLOWED` refusal result (D3) |

---

## 9. Error paths (fail-closed; no invented codes)

| Failure | Detection point | Result |
|---|---|---|
| Approval table missing / DB failure | `request_model_tool_approval`, `expire_approval`, `cancel_approval`, `approval_decision`, `verify_tool_grant`, `count_for_run` return `E_INTERNAL_ERROR`/`E_DB_BUSY` `[READ tools_bridge.rs map_tool_error]` | `suspend_for_approval` returns the refusal → `execute_task` fails task → `dispatch` failed with that code; no hang |
| Request refused: unknown tool / empty id / different args | gate/bridge returns `E_TOOL_NOT_FOUND` / `E_SCHEMA_INVALID` / `E_APPROVAL_REPLAY` `[READ approval.rs; tools_bridge.rs:295-313]` | task failed with the code; no row left pending (bridge returns before insert) |
| Wait cap exceeded | `approval_count_for_run >= MAX_APPROVAL_WAITS_PER_RUN` | `TOOL_RESULT refused E_CALL_LIMIT` (one row), no request committed; loop continues |
| Persist failure after commit | `persist_approval_transitions` error | `expire_approval` best-effort consumes the row; return the refusal; task/run failed by existing paths |
| No waiter armed when WS resolves | `notify_approval_resolved == false` | WS logs `warn!`, still ACKs the committed decision (truthful); engine (if any) re-reads on timer |
| Sender dropped / mutex poisoned | `arm`/receiver `Err` | `E_INTERNAL_ERROR`; row consumed as `cancelled`; task cancelled/failed by path |
| Timer expiry CAS lost | `expire_approval == Ok(false)` | re-read `approval_decision`; apply allow/deny/expired/cancelled; `None` → `E_INTERNAL_ERROR` |
| Verify grant failure (wrong args, consumed, denied-by-row) | `verify_tool_grant` | `E_APPROVAL_REQUIRED`/`E_APPROVAL_REPLAY`; **no execution**; run fails by name (S10) |
| Late resolve of a row already consumed as `expired` | WS resolve → WIP gate maps a stored `expired` decision to `E_APPROVAL_TIMEOUT` `[WIP approval.rs 18:30]` | ERROR event `E_APPROVAL_TIMEOUT`; no execution; the run is already terminal/interrupted |
| Side-effect executor refusal/error after grant | `execute_side_effect` returns `Ok(Refused)`/`Err(ToolError)` | one `TOOL_RESULT refused` (named code) + row; loop continues; grant stays consumed (audit honest) |
| Shell refused/timeout/cancel | `shell::execute` returns `E_TOOL_NOT_ALLOWED`/`E_TOOL_TIMEOUT`/`E_TOOL_CANCELLED` `[READ shell.rs]` | refused result with that code; no fake success |
| File write stale precondition | `execute_side_effect` `expected_sha256` mismatch (D-2) | `Err(E_SCHEMA_INVALID)` → refused result; no write |
| Cancel/approve race | token checked immediately after wake before `verify_grant`/execute (E13) | `TOOL_RESULT cancelled`; no execution; grant unspent or consumed by a decision |
| Duplicate terminal frame | `dispatch` guard (D7) | exactly one terminal `RUN_STATUS` from the owner that persisted it |
| Waiter leak | `ArmedApproval` Drop; tests assert `waiters.is_empty()` after every terminal | no orphan sender |

**Cancellation of the model loop while not waiting:** unchanged (`execute_task` select on token before/after each provider call) `[READ execution.rs:987-1004,1061-1085]`.

---

## 10. Caller changes and signature stability

| Caller | Change |
|---|---|
| `ws.rs::ApprovalResolve` arm `:465-483` | after `publish_ack` + `publish_with_correlation(Event::ApprovalResolved(...))` add:
  ```rust
  if !state.engine.notify_approval_resolved(&resolution) {
      tracing::warn!(approval_id = %resolution.request.approval_id,
          "approval decided with no armed engine wait (late/duplicate/terminal run)");
  }
  ```
  Import nothing new (`ApprovalResolution` is only borrowed through `resolution`). |
| `http.rs` `build_tools`/`build_engine`/3 constructors | §7 reorder + pass `tools`; keep WIP reaper; `build_engine` is private (current `fn` at :155; no external callers) `[READ grep build_engine: only http.rs]` |
| `tools_bridge.rs` | +6 methods (§4.4), all additive |
| `tools/approval.rs` | +4 functions + index + tests |
| `tools/executor.rs` | +`execute_side_effect` (+2 private helpers); `lib.rs` re-export |
| `tool_loop.rs` | +`MAX_APPROVAL_WAITS_PER_RUN` const; no other change |
| `execution.rs` | fields + builders + waiter types + `suspend_for_approval` + helpers + async `execute_tool_turn` + `dispatch_inner` Start + `dispatch` guard + `cancel_run`/`kill_worker` |
| `RunEngine::new` / `with_services` | **signatures unchanged**; engines without `with_tools` keep pre-1.2 side-effect refusal (D3); all existing tests keep compiling except the `StreamedReply` literals already fixed by WIP |
| `main.rs`, `recovery.rs`, `db.rs`, `contract/*`, `web/*` | no change (recovery is WIP-merged only) |

---

## 11. Test names and scenarios (new `core/tests/suspension.rs`; no paid provider)

Harness (`#[derive]` not exhaustive): build `RunEngine::new(db, bus, graph, MockExecutor, TemplateRegistry::default())` with `.with_tool_workspace(WorkspaceJail::open(tempdir))`, `.with_tools(bridge.clone())`, `.with_clock(fixed)` (or default clock for real short TTL), `.with_shell_policy(ShellPolicy::new(["cmd.exe".to_owned()]))`; seed `ToolsBridge::migrate(&db)`; bridge constructed `ToolsBridge::new(tempdir, db.clone()).with_approval_ttl(1)` (D4); `MockExecutor` returns one `file_write` (or `shell_exec`) call then a final answer; subscribe `EventBus` before submit. Approve helper: parse `APPROVAL_REQUEST` (approval_id + nonce), call `bridge.resolve_approval(&ApprovalResolve { approval_id, nonce, decision: "approve", reason: None })`, then `engine.notify_approval_resolved(&resolution)`.

1. `suspend_publishes_awaiting_approval_and_approval_request` — run `RUN_STATUS awaiting_approval`, task `NODE_STATE awaiting_approval`, exactly one `TOOL_CALL`, exactly one `APPROVAL_REQUEST` with nonce/run_id/task_id/tool_call_id/args_sha256/expires_at, no file written, `waiters.len()==1`.
2. `approve_resumes_and_executes_the_file_write_exactly_once` — one `TOOL_RESULT ok`, one `tool_turns` row, file content matches, `RUN_STATUS running` then final `completed`, `tool_turns` count unchanged after a second `notify` replay (`E_APPROVAL_REPLAY` from the gate), `waiters.is_empty()`.
3. `deny_returns_denied_result_and_run_continues` — one `TOOL_RESULT denied` with `code=None`, no write, follow-up final reply persisted, run `completed`, task `completed`.
4. `timeout_fails_run_with_e_approval_timeout_and_no_side_effect` — wait past TTL; run `failed` code `E_APPROVAL_TIMEOUT`, exactly one terminal `RUN_STATUS failed`, row `decision='expired'`, no `APPROVAL_RESOLVED`, no file, one `TOOL_RESULT timeout`.
5. `cancel_while_awaiting_cancels_run_and_consumes_approval` — `CANCEL_RUN`; run `cancelled`, row `decision='cancelled'`, one `TOOL_RESULT cancelled`, later resolve → `E_APPROVAL_REPLAY`, no file, single terminal frame.
6. `kill_worker_while_awaiting_cancels_without_side_effect` — awaiting task kill ACKs, run `cancelled`, no write, waiter gone.
7. `timeout_cas_loss_to_resolve_still_resumes` — resolve just before the timer; delay the wake processing so the timer arm executes; engine re-reads the decided row and executes exactly once (no timeout failure).
8. `concurrent_resolve_has_one_winner_and_no_second_execution` — two `resolve_approval` calls: one `Ok`, one `E_APPROVAL_REPLAY`; one execution; `notify` second returns false.
9. `second_prompt_while_awaiting_is_refused_e_run_active` — `submit_prompt` returns `E_RUN_ACTIVE` while suspended (store already includes awaiting `[READ store.rs:95-114]`).
10. `restart_marks_awaiting_run_interrupted_and_expires_pending` — seed DB with run `awaiting_approval` + a pending row whose TTL is still in the future; run `recover_on_boot` then build `AppState::with_services` (temp data dir); run `interrupted` with `error_code=E_APPROVAL_TIMEOUT`; pending row `decision='expired'` even though its TTL had not elapsed (WIP `expire_orphaned_pending` is unconditional; no waiter survives restart); later resolve → `E_APPROVAL_TIMEOUT`; no auto-approve/execution.
11. `wait_cap_refuses_the_ninth_request` — `MAX_APPROVAL_WAITS_PER_RUN=8`; 9th `TOOL_RESULT refused E_CALL_LIMIT`, no new approval row.
12. `wrong_args_grant_refused_before_side_effect` — approve args A, then a fabricated plan with args B reaches resume; `E_APPROVAL_REQUIRED`, no write, run failed by name, one `TOOL_RESULT refused`.
13. `shell_exec_runs_only_the_allowlisted_program_after_approval` — allowlist `cmd.exe`; approved call captures stdout; non-allowlisted program is refused `E_TOOL_NOT_ALLOWED` and never spawns (assert file/`shell` outcome).
14. `no_bridge_engine_keeps_the_read_only_refusal` — `RunEngine::new` (no `.with_tools`), `file_write` call → `E_TOOL_NOT_ALLOWED` result; existing behavior preserved (D3).

`#[cfg(test)]` unit tests to add beside the new tools code: `expire_consumes_only_elapsed_pending_row`, `cancel_pending_consumes_only_pending_row`, `decision_label_reads_each_state`, `count_for_run_counts_all_decisions`, `unique_pending_index_rejects_second_pending_row`, `verify_tool_grant_consumes_allow_once_exactly_once` (approval.rs); `execute_side_effect_writes_and_reports_hash`, `execute_side_effect_refuses_non_side_effect_tools`, `stale_expected_sha256_refuses_write`, `shell_exec_requires_allowlisted_program` (executor.rs); `approval_waiters_arm_wake_disarm_roundtrip`, `wake_resolved_after_disarm_returns_false`, `waiters_debug_reports_armed_count` (execution.rs unit module — optional if the integration file covers it).

Commands (Lead runs; this planner ran none):
```powershell
cd backend
$env:PATH = "$env:USERPROFILE\.cargo\bin;$PWD\..\tools\w64devkit\bin;$env:PATH"
cargo test -p hiverouter-core --test suspension
cargo test -p hiverouter-core --test tool_loop_engine
cargo test -p hiverouter-core --test run_lifecycle
cargo test -p hiverouter-core --test approval_ledger
cargo test -p hiverouter-core --test ws_transport
cargo test -p hiverouter-tools --lib approval::tests
cargo test -p hiverouter-tools --lib executor::tests
cargo test --workspace --locked
```

### 11.3 Existing tests to reconcile during implementation (read, not edited by this planner)

- `core/tests/tool_loop_engine.rs`: the default harness builds engines **without** `.with_tools`; D3 keeps all 12 existing scenarios unchanged. Only if the harness gains `.with_tools` do the side-effect refusal tests change meaning — do not wire it there.
- `core/tests/run_lifecycle.rs`: `dispatch_inner` now publishes one `RUN_STATUS running` before leaves (D2). Tests that count frames (`streamed_deltas_are_published_before_done_and_not_duplicated`, `run_lifecycle_plans_and_dispatches`) may need the extra frame tolerated; verify before claiming green.
- `core/tests/ws_transport.rs`: `approval_resolve_command_acks_and_emits_resolved_event` will now also call `notify` (no waiter → false, warn only); ACK/event behavior unchanged.
- `core/tests/approval_ledger.rs`: unchanged (WS owns the ledger; engine never appends).
- `core/tests/http_boot.rs`: construction order change only; `with_services` keeps `wire_workspace_jail=false`; health schema stays 4.

---

## 12. What must NOT be claimed (guardrails for the implementing wave's report)

1. **No build/test/compile claim by this planner.** This document is read-only analysis; every runtime statement is `UNVERIFIED`. Only a Lead-run `cargo test`/`cargo build` with quoted output may claim green.
2. **No crash-resume of a suspended turn.** A process restart interrupts the run (WIP recovery) and expires approvals; the model loop cannot continue from a persisted cursor (D11/D1).
3. **No exactly-once guarantee across processes** — exactly-once is per committed grant within one process: `verify_grant` single-use CAS + in-process wait. A crash between `verify_grant` and the side effect consumes the grant without an effect (honest audit gap, not a retry claim).
4. **No approval for `spawn_agent`/`report_result`/`finish`** — those are loop-owned/Read-class; only `file_write` (Mutate) and `shell_exec` (Execute) are reachable side effects at WorkerAgent tier `[READ tool.rs:315-380]`.
5. **No shell sandbox beyond the documented allowlist/jail/caps**: `ShellPolicy::conservative_default()` is a default, not a security boundary claim; do not claim arbitrary command execution safety.
6. **No new refusal codes, no new wire events/commands, no UI work.** The 23-event/17-command frozen contract is unchanged; `APPROVAL_RESOLVED` is published only for operator approve/deny, never for timeout/cancel (D9/S4-S6).
7. **No ledger row written by the engine**; ledger evidence remains WS/bridge-owned (D5).
8. **No `expected_sha256` contract claim**: no frozen stale-write refusal code exists; D-2 maps it to `E_SCHEMA_INVALID` and this mapping must be reported as a known semantic gap, not as a dedicated `E_FILE_CHANGED`.
9. **No `run_state` claim beyond the approval edges + run Start** (D6); start/complete/fail task writes remain direct in this wave.
10. **No claim that `recovery.rs`/boot expiry were implemented by this wave** — they are the Lead WIP merge dependency (D8); only integration/merge notes are in scope.
11. **No claim that `notify_approval_resolved` returns true in production paths** — false is legal (late/duplicate/terminal run) and only logged.
12. **No claim about a second concurrent run or multi-run waiter isolation**; single-active-run policy is unchanged.

---

## 13. Evidence-check table (`/evidence-check`: claim → evidence → status)

| Claim | Evidence | Status |
|---|---|---|
| Engine has no suspension today; non-read calls are refused in `execute_tool_plan` | `[READ execution.rs:879-920]`; no `awaiting_approval` producer in execution.rs | VERIFIED (read) |
| Approval producer + wire event exist but are uncalled | `[READ tools_bridge.rs:287-348]`; grep `request_model_tool_approval` call sites = bridge tests | VERIFIED (read/grep) |
| WS resolve does not wake the engine | `[READ ws.rs:465-483]` | VERIFIED (read) |
| `run_state` approval edges/effects exist and are unused by execution.rs | `[READ run_state.rs:465-620]`; grep `run_state` execution.rs = 0 | VERIFIED |
| `run_state` requires the run to be `running` before `Suspend`; engine never sets the run running | `[READ run_state.rs:497-499]`; `[READ execution.rs:585-586,968-981]` | VERIFIED |
| `shell::execute` is async → side-effect executor must be async | `[READ shell.rs:173-178]` | VERIFIED (plan §4 sync signature corrected) |
| No stale-write refusal code exists; `expected_sha256` unused | `[READ refusal.rs:75-279]`; grep `expected_sha256` = struct only | VERIFIED |
| Approvals table is tools-owned with no unique pending index | `[READ approval.rs:141-165]`; `[READ http.rs:107-113]`; `[READ db.rs migrations v1-v4]` | VERIFIED |
| WIP boot expiry / interrupted code / reasoning are uncommitted and **changed during this pass** | `git diff HEAD` at 18:20 and re-diff at 18:30 CT (§2); approval.rs +78→+166 | READ (WIP, may change again) |
| `dispatch` publishes a terminal frame that can duplicate `cancel_run`/timeout | `[READ execution.rs:539-583]` vs `:499-519`; `[PLAN §8 bug 14]` | VERIFIED (read) |
| `oneshot` is available (tokio `sync` feature) | `[READ backend/Cargo.toml:17]` features include `sync` | VERIFIED (manifest read) |
| Actual runtime behavior of the proposed diff | not implemented, no build/test run | **UNVERIFIED** |
| Exact "items 1,2,5,9" list | not in repo; `[PLAN §0]` records the assumption | UNKNOWN |

---

## 14. UNKNOWN register (what would settle each)

| ID | Unknown | Settle by |
|---|---|---|
| U1 | Whether the Lead WIP (reasoning/recovery/expiry) lands unchanged before this wave — **it changed once during this planning pass** (+78→+166 lines) | re-run `git diff HEAD` at implementation start; §2 pins the 18:30 hunks |
| U2 | Whether existing DBs contain duplicate pending `(run_id, tool_call_id)` rows that would block the unique partial index | query `SELECT run_id, tool_call_id, COUNT(*) FROM approvals WHERE decision IS NULL GROUP BY 1,2 HAVING COUNT(*)>1` on the live DB before adding the index |
| U3 | Whether `run_lifecycle`/`ws_transport` frame-count assertions tolerate the new `RUN_STATUS running` frame and the extra notify-false warning | read those tests at implementation time (§11.3); adjust only assertions, not behavior |
| U4 | Production shell allowlist policy for `shell_exec` (today only `conservative_default`) | product decision; engine builder seam exists, config wiring is a later change |
| U5 | Whether the UI presents/needs the extra fields (`decision` absent, expires_at countdown) — not re-read this pass | read `web/src/api/contract.ts`, `ApprovalOverlay.tsx` at implementation time |
| U6 | Behavior of `EventBus` replay ordering for `APPROVAL_REQUEST` vs `TOOL_CALL` on `last_seq` reconnect | read `bus.rs` replay semantics/tests (not needed for the engine wait itself) |
| U7 | Whether `tokio` timer precision/`timeout` under the test runtime makes scenario 4/7 flaky with a 1 s TTL | Lead test run; if flaky, add `tokio` `test-util` feature or inject a manual wake in tests |

---

## 15. Deep-naming audit (plan only; no rename applied) — `/deep-naming`

Vocabulary: **approval request** (row/wire), **approval decision/resolution**, **grant**, **wait/suspend/resume**, **waiter/wake**, **side effect**, **tool call/result**, **run/task state**. Names chosen above: `ApprovalWaiters` (plural owner of id→sender), `ApprovalWake` (event-shaped), `notify_approval_resolved` (sync wake), `suspend_for_approval` (explicit await), `resume_and_execute` / `resume_denied` (verb + effect), `finish_cancelled_wait` (terminal path), `execute_side_effect` (explicit side effect), `verify_tool_grant` (verification only), `approval_count_for_run` (count source). Rejected generic names: `handle_approval`, `ApprovalOutcome`, `resume_run` (ambiguous with boot recovery), `execute_tool` for side effects (would collide with read-only `execute_read_only`). `[PLAN §14]` already flagged the tools-vs-wire `ApprovalRequest` name collision; this wave does not rename it (public API, not in scope).

---

## 16. Skill application evidence

- **/deep-research**: question set (does the engine wait? producer? resolver? boot safety? caps?) answered from repository primary sources only; each answer carries `file:line`; no external sources needed; UNKNOWNs kept in §14.
- **/research**: current-behavior table §1 + HEAD/WIP distinction §2; no claim without a read source.
- **/forgewright-lead**: read code/callers/tests before hypothesis; root cause = missing engine owner (no wait/waiter/bridge fields) evidenced by grep/read; complete code snippets only (no placeholders; the two illustrative correction boxes explicitly say what to use instead).
- **/forgewright-plan**: outcome, current behavior, decisions with rejected alternatives, file-by-file changes, failure modes, tests, rollback (feature-flag by absence of `with_tools`), out-of-scope.
- **/deep-naming**: vocabulary list + proposed names + rejected generic names; no rename applied.
- **/truthseeker**: falsifiable invariants — one decision → one grant/execution; one wait → one wake; one planned call → one `TOOL_RESULT`/row; one terminal frame; disarm on every exit.
- **/scientific-thinking**: competing hypotheses for "why no suspension" (engine missing vs. WIP already implementing vs. not needed) evaluated; the decisive evidence is the grep/read absence of any production producer/wake path; alternatives rejected with evidence.
- **/forge**: each predicted defect has a RED test in §11; minimal-change boundaries (no db/contract/web changes).
- **/evidence-check**: §13 table; `UNVERIFIED` kept for runtime behavior; contradictions none found.

**Planner statement:** no product file was edited, no build/test/commit was run. The only write is this document. All line numbers are from the dirty tree at 2026-11-06 18:20–18:35 CT and will shift by the §2 WIP offsets after merge.
