# Foresight Plan 1 — Engine approval suspension / resume (1.2.0-dev)

**Planner:** Foresight Planner 1 · **Mode:** READ-ONLY (no code, no builds, no commits; this file is the only write)
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **Target HEAD:** `37f0e7c68b835f267e64db3cd5c0612114328e46`
(“wip(1.2): compile-green read-only tool loop, approval command/UI, 23-event contract, e2e tool/approval specs”)
**Tree state at recon (READ, 2026-11-06 18:13–18:30 CT):** **DIRTY** — 8 files modified by concurrent lanes:
`core/src/{execution.rs, http.rs, provider_executor.rs, recovery.rs, tools_bridge.rs}`, `core/tests/run_lifecycle.rs`,
`core/tests/tool_loop_engine.rs`, `tools/src/approval.rs`. Current-behavior anchors below are taken from **HEAD
`37f0e7c`** via `git show HEAD:<path>` so they are stable; the uncommitted WIP is described separately (§2.3) and is a
**merge dependency**, not a claim of current behavior.
**Skills applied:** `/research`, `/forgewright-lead`, `/forgewright-plan`, `/deep-naming`, `/truthseeker`,
`/scientific-thinking`, `/forge`, `/evidence-check` (all loaded; evidence in §17). No external documentation was needed:
every fact below is read from this repository; unknowns are labelled **UNKNOWN**.

---

## 0. Scope and the “items 1, 2, 5, 9” reference — UNKNOWN

The assignment says “engine approval suspension/resume (items 1,2,5,9 of the list)”. **The numbered list itself is not
present in the repository.** Searched: `plans/**`, `docs/**`, `git log --all --grep='items|suspension|engine'`,
`git show 37f0e7c` message. No 9-item engine checklist exists; the closest enumerated corpora are
`W11-SUSPENSION-DESIGN.md` §5 (5-step protocol) and §13 (8 blockers W11-B1..B8), and `run_state.rs`’s twelve
`StateEvent`s — none is a 1/2/5/9 list. Therefore:

- **UNKNOWN:** the exact content of “the list”.
- **Assumption A0 (explicit, falsifiable):** in this plan, items 1/2/5/9 are interpreted as the four engine-side
  suspension/resume work areas: **(1) suspend a run when a model-originated side-effect call needs consent (commit +
  `awaiting_approval` + `APPROVAL_REQUEST`)**; **(2) wake/notify the engine when `APPROVAL_RESOLVE` commits**;
  **(5) resume and execute the approved side effect exactly once**; **(9) terminal/edge paths — deny, timeout,
  cancel/kill, restart, duplicate resolve, unknown run**.
- If the Lead’s list differs, only §3/§9 mappings change; the API/state/data-flow design (§4–§8) remains the complete
  engine suspension surface and should be reconciled before coding. **Do not invent content for missing list items.**

**Deliverable:** `plans/foresight/APPROVAL-SUSPENSION-PLAN.md` (this file). No other file is written.

---

## 1. Research note (`/research` — question → confirmed facts → unknowns)

| # | Question | Confirmed fact (evidence, HEAD `37f0e7c`) | Status |
|---|---|---|---|
| Q1 | Does the engine suspend today? | No. `dispatch` is linear and maps its single outcome to Completed/Cancelled/Failed (`execution.rs:535-579`); `dispatch_inner` plans then executes leaves once (`:581-654`); `execute_task` calls the executor then loops read-only tool turns (`:964-1096`). No wait, no `awaiting_approval` producer. | VERIFIED (read) |
| Q2 | Is an approval producer available in core? | Yes, but **no caller**. `ToolsBridge::request_model_tool_approval` (`tools_bridge.rs:271-311`) commits the row through the tools gate and returns the wire `APPROVAL_REQUEST`; `pending_approvals` (`:231`), `approval_request_event` (`:320`), `pending_approval_events` (`:342`) exist. `grep request_model_tool_approval` = definition + bridge tests only. | VERIFIED (read, grep) |
| Q3 | Does resolving wake the engine? | No. The WS arm resolves/ledgers/publishes then returns (`ws.rs:465-483`); no `notify_approval_resolved`, no waiter registry anywhere (`grep notify_approval\|ApprovalWaiters` = 0 production hits). | VERIFIED (read, grep) |
| Q4 | Are approvals expired at boot / on restart? | **HEAD: no.** `approval.rs` (HEAD) has no `expire_pending`; `recovery.rs:50-64` marks active runs `interrupted` with `error_code = None` and does not touch approvals; `http.rs:107-114` only runs the approvals DDL. **Uncommitted WIP adds exactly this** (see §2.3), so the plan depends on that lane. | VERIFIED (HEAD read); WIP READ |
| Q5 | Does anything drive `run_state` transitions? | No. `run_state::apply_run/apply_task` define the legal edges + effects (§2.4), but `execution.rs` has **zero** references to `run_state` (grep). | VERIFIED |
| Q6 | Is there waiter infrastructure / async wake in the engine? | None. `RunEngine` fields are `store, db, bus, graph, contexts, tokens, executor, templates, caps, snapshots, tool_workspace, tool_registry, active_run` (`execution.rs:256-285`); no gate/bridge/waiter/clock. `tokio` workspace features include `sync` (oneshot available) — `Cargo.toml:15-16`. | VERIFIED (read) |
| Q7 | Can the engine reach the approval gate/bridge today? | No. `RunEngine` has no `ToolsBridge`; `AppState` builds engine before tools (`http.rs:183-185` HEAD) and never passes tools into it. `RunEngine::new`/`with_services` take no bridge (`execution.rs:302-377`). | VERIFIED |
| Q8 | Is there a side-effect executor for the resume path? | No. `tools/executor.rs` (HEAD) exports only `ExecutedTool` + `execute_read_only` (read-only; every non-read tool → `E_TOOL_NOT_ALLOWED`). Raw primitives exist: `WorkspaceJail::write` (`jail.rs:223`), `shell::execute` + `ShellPolicy`/`ShellRequest` (`shell.rs:34,114`). | VERIFIED (read) |
| Q9 | Is there exactly-once protection for execution after approval? | Gate has `verify_grant` with single-use `executed_at` marker for `allow_once` (`approval.rs:653-...`; UPDATE `executed_at WHERE executed_at IS NULL`). It is **never called by production code** (grep = 0). | VERIFIED |
| Q10 | Are caps/waits bounded for a suspension loop? | Partly. `tool_loop::MAX_TOOL_TURNS=4`, `MAX_CALLS_PER_RUN=16` (`tool_loop.rs:14,16`); engine call cap is still the literal `check_calls(1)` at `execution.rs:585,970` (pure comparison, no counter — a known defect); no approval-wait cap exists. | VERIFIED |
| Q11 | Is a duplicate pending approval for one tool call possible? | Yes. `approvals` DDL has `UNIQUE(nonce)` only; no unique `(run_id, tool_call_id) WHERE decision IS NULL` (HEAD `approval.rs` DDL). Bridge producer dedupes in Rust (same `(run,tool_call_id)`+same args returns the existing row; different args → `E_APPROVAL_REPLAY`), but two *concurrent* producer calls can race (documented by the bridge seat). | VERIFIED (read) |
| Q12 | Is the UI able to resolve? | Wire + UI receipts were reported in W11/W12 plans (nonce/args/TTL fields, approve/deny sending `APPROVAL_RESOLVE`), but this planner did not re-read `web/` in this pass → **UNKNOWN for the current tree**; not required for the engine design because the engine only needs the WS command → wake path. | UNKNOWN |
| Q13 | Does the boot order allow expiry before serving? | Boot: `recover_on_boot` (`main.rs:81`) → `AppState::with_graph` (`main.rs:123`) → `build_engine` then `build_tools` (`http.rs:183-185` HEAD). With the WIP, `build_tools` runs the expiry reaper while constructing state, i.e. before the listener binds — correct; but tools are built **after** the engine, which is the deliberate wiring change in §5/§9. | VERIFIED (HEAD) / WIP |

**Design implication:** the engine is the only missing owner. The tools layer (producer, gate, TTL, single-use grant,
ledger), the WS resolve command and the frozen 23-event contract already exist; what is absent is (a) an engine wait
primitive and waiter registry, (b) a producer call + `awaiting_approval` transition + `APPROVAL_REQUEST` publish,
(c) a resume path that verifies the grant and executes the side effect exactly once, (d) terminal paths and boot
recovery for waits, (e) the engine’s handle on `ToolsBridge` and a clock/TTL seam. No new contract vocabulary is needed.

---

## 2. Current behavior (HEAD `37f0e7c`; every claim has an anchor)

### 2.1 Run lifecycle (linear, single-run)
- `submit_prompt` validates, refuses a second active run (`store.latest_active_run()` includes `awaiting_approval` —
  `store.rs:95-114`), creates the run `queued`, records the receipt, creates one `CancellationToken`, publishes
  `RUN_CREATED`, spawns `dispatch` (`execution.rs:410-448`).
- `dispatch` awaits `dispatch_inner`, then maps exactly one outcome: `Ok` → `completed`; `Err(ToolCancelled)` →
  `cancelled` (no code); any other `Err` → `failed` + error code; persists `RUN_STATUS`, writes a terminal snapshot,
  clears `active_run`, removes the token, calls `executor.finish_run` (`execution.rs:535-583`).
- `dispatch_inner` sets `planning`, makes one planning call (`caps.check_calls(1)`, `:585`), validates the plan,
  creates root + plan tasks, and executes leaves in order (`:581-654`). There is no persisted run phase beyond `runs.status`.

### 2.2 Tool turn + read-only executor (1.2 WIP that this plan extends)
- `execute_task` (`:964-1096`) checks the call cap literal (`:970`), marks the task `running`, streams one executor
  reply, then loops over `reply.tool_calls` (`:1013-1088`): `execute_tool_turn` → transcript → `continue_with_tools` →
  repeat; finally `complete_task` persists the final assistant text, publishes `REPLY_DONE`, marks task `completed`
  (`:1099-1127`).
- `execute_tool_turn` (`:680-806`) reads persisted counters, plans/validates each call at `WorkerAgent` tier, publishes
  `TOOL_CALL` (`:919-931`), routes to `execute_tool_plan`, then `finish_tool_call` publishes `TOOL_RESULT` and inserts a
  `tool_turns` row (`:930-962`); counters are incremented once per turn (`:796-800`).
- `execute_tool_plan` (`:875-920`) is **synchronous and read-only**: any tool whose permission class is not `Read` is
  refused `E_TOOL_NOT_ALLOWED` with “side-effecting calls stay approval/loop-owned and did not execute”
  (`:880-891`); read calls go through `execute_read_only` + `WorkspaceJail`.
- `tool_loop` (HEAD) offers the WorkerAgent allowlist specs including `file_write`/`shell_exec` (`tool_loop.rs:203-226`
  + `tool.rs` allowlists); `requires_approval` exists (`tool_loop.rs:193`) and is **not called anywhere in core**
  (grep: definition + tests only). `ToolExecution::denied`/`timed_out`/`cancelled` constructors already exist
  (`tool_loop.rs:84-121`) — the deny/timeout result vocabulary is ready.

### 2.3 Uncommitted WIP observed (dependency, not current behavior) — READ
`git diff` at recon shows another lane already adding:
`approval.rs::expire_pending(conn, now) -> Result<u64, ToolError>` (marks TTL-elapsed pending rows `decision='expired'`
by CAS); `ToolsBridge::expire_pending_approvals(db, now)`; `http.rs::build_tools` calling it at boot;
`recovery.rs` setting `error_code=E_APPROVAL_TIMEOUT` for runs that were `awaiting_approval`; plus reasoning-content
plumbing in `execution.rs`/`provider_executor.rs`/tests (not related to suspension).
**Implication:** the boot/restart half of the plan must **reuse** this WIP and not re-implement it; the engine half is
unstarted. Merge order requirement: WIP lands first (or the implementing wave rebases onto it) because the plan’s
`expire/cancel` additions touch the same `approval.rs`/`tools_bridge.rs` regions.

### 2.4 State vocabulary and the unused transition table
`run_state.rs` already encodes the legal approval edges (committed file):
- `StateEvent::{Suspend, Resume, Deny, Timeout, Restart}` (`run_state.rs:131-156`).
- `Effect::{RequestApproval, ResumeFromApproval, FeedDeniedToolResult, FailOnApprovalTimeout, CancelRun, …}`
  (`run_state.rs:205-250`); `Effect::refusal()` maps only `FailOnApprovalTimeout` → `E_APPROVAL_TIMEOUT` (`:258-265`).
- Run edges: `running + Suspend → awaiting_approval` (`:497-499`), `awaiting_approval + Resume → running` (`:500-502`),
  `+ Deny → running` (`:503-505`), `+ Timeout → failed` (`:506-508`), `Cancel` legal from awaiting (`:490-496`),
  `Restart` from awaiting → `interrupted` (`:509-515`); illegal replay of Resume/Deny/Timeout yields
  `RunStateError::ApprovalReplay` → `E_APPROVAL_REPLAY` (`:516-524,302-354`).
- Task edges mirror them (`run_state.rs:568-619`), including `awaiting_approval + Kill → cancelled` (`:581-583`).
**The engine never calls this table.** This plan makes the engine call it for the suspension path only (decision D6).

### 2.5 Approval storage/gate (tools, HEAD)
`APPROVAL_TTL_SECS=120` (`approval.rs:26`); `Approvals` DDL (`migrate`, `:141-...`) with `UNIQUE(nonce)`;
`ApprovalGate::request` fills `approval_id`, run/task/tool_call ids, tool, permission class, `args_sha256`, created/
expires timestamps, nonce, and inserts (`:621-652`); `resolve_with_nonce`/`resolve_with_reason` CAS
`UPDATE … WHERE decision IS NULL` (single winner) and return `ApprovalTimeout` after marking expired (`:415-514`);
`verify_grant` consumes one `allow_once` by setting `executed_at` (`:694-...`); `pending_for_run` (`:561-579`);
`request_tool_approval` producer (`:730-...`). **HEAD has no `expire_pending`, no `cancel_pending`, no per-approval
decision reader** — WIP adds the first.
Bridge (HEAD): producer/dedupe/wire-event/pending (`tools_bridge.rs:231-351`), resolve+single ledger row
(`:201-223`), resolve without ledger (`:154-171`). Ledger path `<data_dir>/ledger/run-ledger.jsonl`
(`:178-242`). `db.rs` schema v4 adds `runs.tool_turns`, `runs.tool_calls`, `tool_turns` table + counters
(`db.rs:115-145,424-...`, counters `:550-...`); `chat_messages` has no tool-turn/meta columns
(v1-v4 only). Approvals table is created by `ToolsBridge::migrate` at boot (`http.rs:107-114`), outside
`db::MIGRATIONS`.

### 2.6 WS resolve path (HEAD)
`Command::ApprovalResolve` → `state.tools.resolve_approval_ledgered(payload, <data_dir>/ledger)` → on success
`publish_ack` + `APPROVAL_RESOLVED` (truthful approve/deny from the resolution); on error `ERROR` with the gate code
(`ws.rs:465-483`). **No engine wake, and a ledger failure after a committed decision returns ERROR while the row is
already decided** (bridge docs, `tools_bridge.rs:182-223`).

---

## 3. Exact missing declarations / functions (nothing here exists at HEAD)

**Engine (`core/src/execution.rs`)**
1. `RunEngine` fields: `tools: Arc<ToolsBridge>` (or `Option` for tests), `ledger_dir: PathBuf`,
   `waiters: Arc<ApprovalWaiters>`, `approval_ttl_secs: u64`, `now_unix_secs: Arc<dyn Fn() -> u64 + Send + Sync>`.
2. `struct ApprovalWaiters` + `enum ApprovalWake` and methods `arm/take/wake_resolved/wake_cancelled/len`.
3. `RunEngine::notify_approval_resolved(&self, &ApprovalResolution) -> bool`.
4. Builders: `with_tools`, `with_approval_ttl`, `with_clock` (test seam).
5. `async fn suspend_for_approval(&self, run, task, plan, turn) -> Result<ApprovalWake, Refusal>` — the
   commit→arm→persist→publish→select sequence.
6. `async fn wait_for_approval(...)` helper (select receiver / TTL / cancellation) + timeout CAS + race re-read.
7. `async fn execute_approved_side_effect(&self, plan) -> ToolExecution` (verify grant → execute once → TOOL_RESULT).
8. `execute_tool_turn` must become **async** and route non-read plans into `suspend_for_approval` instead of
   `execute_tool_plan`’s refusal (`execution.rs:875-920` refusal branch is the seam).
9. `execute_tool_plan` splits: read branch unchanged; non-read branch calls the suspension path (keep the function
   name for the read router or rename per §14).
10. `kill_worker` (`execution.rs:521-533` HEAD) currently refuses a task unless `state == Running`; must accept
    `AwaitingApproval` (and consume/cancel the approval) per `run_state` task edge (`run_state.rs:581-583`).
11. `cancel_run` (`:495-519` HEAD) must consume pending approvals for the run after token cancel.
12. Engine must call `run_state::apply_run/apply_task` for the suspension edges (§2.4) instead of hand-writing
    statuses; effect→action table in §6.
13. Approval-wait cap constant + check (propose `MAX_APPROVAL_WAITS_PER_RUN = 8` in `tool_loop.rs`; use existing
    `E_CALL_LIMIT` because no new code is allowed in this wave; see D11).

**Tools (`tools/src/approval.rs`, `tools/src/executor.rs`, `tools/src/lib.rs`)**
14. `pub fn expire(conn, approval_id: &str, now_unix_secs: u64) -> Result<bool, ToolError>` — CAS
    `decision='expired' WHERE decision IS NULL` (per-approval TTL winner; the WIP `expire_pending` is the bulk boot
    version, not enough for a per-wait timer).
15. `pub fn cancel_pending(conn, approval_id: &str, now_unix_secs: u64) -> Result<bool, ToolError>` — CAS
    `decision='cancelled'` (internal label, never a wire decision; see W11 D7).
16. `pub fn decision_label(conn, approval_id: &str) -> Result<Option<String>, ToolError>` — read-only re-read used by
    timeout/cancel races (see §7).
17. `executor::execute_side_effect(jail, shell_policy, registry, tier, tool, args) -> Result<ExecutedTool, ToolError>`
    — exactly `file_write` (`jail.write`) and `shell_exec` (`shell::execute`); everything else
    `ExecutedTool::Refused { code: E_TOOL_NOT_ALLOWED }`. Must be reachable **only** after `verify_grant`.
18. `lib.rs` re-exports for 14–17.

**Bridge (`core/src/tools_bridge.rs`)**
19. `pub fn expire_approval(&self, approval_id: ApprovalId, now: u64) -> Result<bool, Refusal>`,
    `pub fn cancel_approval(...)`, `pub fn approval_decision(&self, approval_id: ApprovalId) -> Result<Option<String>, Refusal>`,
    `pub fn verify_tool_grant(&self, run_id: RunId, tool: &str, args: &serde_json::Value, now: u64) -> Result<(), Refusal>`
    (maps tool name → `ToolId`, delegates to `ApprovalGate::verify_grant` on the bridge DB connection).
    These keep the gate/DB inside the tools bridge (single owner) and avoid the engine opening SQLite directly.
20. Reuse the WIP `expire_pending_approvals` at boot (no duplicate).

**WS (`core/src/ws.rs`)**
21. In the `ApprovalResolve` Ok branch, after publishing ACK/`APPROVAL_RESOLVED`:
    `let woke = state.engine.notify_approval_resolved(&resolution);` and `tracing::warn!` when `!woke`
    (decided row with no waiter is legal — recovery/terminal cases).

**Wiring (`core/src/http.rs`, main unchanged)**
22. `build_tools` before `build_engine`; pass `Arc<ToolsBridge>` + ledger dir into `build_engine` →
    `RunEngine::new(...).with_tools(tools.clone(), config.data_dir.join("ledger"))`; same for
    `AppState::with_services` (tests use `with_services`, so an engine builder seam is required:
    `RunEngine::with_tools`/`with_clock` on the existing public constructors).
23. `main.rs` needs no change (recovery before AppState already correct).

**Recovery/boot (already WIP, listed for completeness)**
24. Pending approvals expired at boot, and runs that were `awaiting_approval` marked `interrupted` with
    `error_code=E_APPROVAL_TIMEOUT` (WIP; must land before suspension is safe). `RecoveryReport` should expose an
    `expired_approvals` count if the WIP does not already (UNKNOWN — WIP not final).

---

## 4. Proposed API signatures (Rust; PROPOSED, exact for the implementing wave)

```rust
// core/src/execution.rs
/// One in-process wake for one approval id. Armed before the request is published.
#[derive(Debug)]
pub struct ApprovalWaiters {
    inner: std::sync::Mutex<std::collections::HashMap<hiverouter_contract::ApprovalId,
        tokio::sync::oneshot::Sender<ApprovalWake>>>,
}
impl ApprovalWaiters {
    pub fn new() -> Self;
    /// Register the receiver for `approval_id`; returns it to select on.
    /// Inserting over an existing key replaces the stale sender (a retried
    /// request for the same id can never be lost).
    pub fn arm(&self, approval_id: hiverouter_contract::ApprovalId)
        -> tokio::sync::oneshot::Receiver<ApprovalWake>;
    pub fn wake_resolved(&self, resolution: &hiverouter_tools::ApprovalResolution) -> bool;
    pub fn wake_cancelled(&self, approval_id: hiverouter_contract::ApprovalId) -> bool;
    pub fn disarm(&self, approval_id: hiverouter_contract::ApprovalId);
    #[must_use] pub fn len(&self) -> usize;
}

/// What a waiter can learn. `Resolved` is the gate's committed decision; a
/// cancelled wait never produces a decision (W11 D7).
#[derive(Debug, Clone, PartialEq)]
pub enum ApprovalWake {
    Resolved(hiverouter_tools::ApprovalResolution),
    Cancelled,
}

impl RunEngine {
    // Builders (test seam; production uses with_tools from http.rs):
    #[must_use] pub fn with_tools(mut self, tools: std::sync::Arc<ToolsBridge>,
                                  ledger_dir: std::path::PathBuf) -> Self;
    #[must_use] pub fn with_approval_ttl(mut self, ttl_secs: u64) -> Self;
    #[must_use] pub fn with_clock(mut self, now: std::sync::Arc<dyn Fn() -> u64 + Send + Sync>) -> Self;

    /// Wakes one suspended run. Returns false when no waiter is armed (the row
    /// is already decided; recovery/terminal run — caller logs, never fails the
    /// resolve command).
    pub fn notify_approval_resolved(&self, resolution: &hiverouter_tools::ApprovalResolution) -> bool;

    /// Commits the approval row, arms the waiter, persists `awaiting_approval`,
    /// publishes RUN_STATUS/NODE_STATE/TOOL_CALL/APPROVAL_REQUEST, then waits on
    /// resolution / TTL / run cancellation. Never executes here.
    async fn suspend_for_approval(&self, run: &Run, task: &TaskRecord,
                                  plan: &ToolCallPlan, turn: u32)
        -> Result<ApprovalWake, Refusal>;

    /// After a grant: verify the single-use grant, execute exactly once, and
    /// return the bounded ToolExecution (publish/persist happen in the caller so
    /// events and tool_turns keep their single owner).
    async fn execute_approved_side_effect(&self, run: &Run, plan: &ToolCallPlan)
        -> ToolExecution;
}

// core/src/tools_bridge.rs (all delegate to ApprovalGate / tools executor)
impl ToolsBridge {
    pub fn expire_approval(&self, approval_id: ApprovalId, now_unix_secs: u64) -> Result<bool, Refusal>;
    pub fn cancel_approval(&self, approval_id: ApprovalId, now_unix_secs: u64) -> Result<bool, Refusal>;
    pub fn approval_decision(&self, approval_id: ApprovalId) -> Result<Option<String>, Refusal>;
    pub fn verify_tool_grant(&self, run_id: RunId, tool: &str,
                             args: &serde_json::Value, now_unix_secs: u64) -> Result<(), Refusal>;
}

// tools/src/approval.rs
pub fn expire(conn: &rusqlite::Connection, approval_id: &str, now_unix_secs: u64)
    -> Result<bool, crate::error::ToolError>;
pub fn cancel_pending(conn: &rusqlite::Connection, approval_id: &str, now_unix_secs: u64)
    -> Result<bool, crate::error::ToolError>;
pub fn decision_label(conn: &rusqlite::Connection, approval_id: &str)
    -> Result<Option<String>, crate::error::ToolError>;

// tools/src/executor.rs (new function beside execute_read_only)
pub fn execute_side_effect(
    jail: &crate::jail::WorkspaceJail,
    shell_policy: &crate::shell::ShellPolicy,
    registry: &crate::tool::ToolRegistry,
    tier: crate::tool::AgentTier,
    tool: &str,
    args: &serde_json::Value,
) -> Result<ExecutedTool, crate::error::ToolError>;
// file_write -> jail.write(path, content bytes); shell_exec -> shell::execute(policy, request);
// registry.validate_call(tier, tool, args) first; everything else ExecutedTool::Refused(E_TOOL_NOT_ALLOWED).
```

**Callers that stay stable:** `submit_prompt`, `cancel_run` (signature), `kill_worker` (signature), `Add`/recovery,
`ws_handler`, the frozen commands/events. Only `RunEngine` construction gains builders; `RunEngine::new` /
`with_services` keep their signatures and produce engines without approval wiring (tests that do not need it keep
passing; a no-tools engine must refuse suspension with `E_INTERNAL_ERROR`, not fake a grant).

---

## 5. State transitions (the engine must drive these; `run_state` is the table)

| From | Trigger | To | Persist / events (single effect) | Owner |
|---|---|---|---|---|
| running | model returns >=1 side-effect call (`requires_approval`) | awaiting_approval | approval row committed → arm waiter → `RUN_STATUS awaiting_approval` → `NODE_STATE awaiting_approval` → `TOOL_CALL` → `APPROVAL_REQUEST` | engine |
| awaiting_approval | `APPROVAL_RESOLVE approve` (gate committed) and engine woken | running | `verify_grant` → execute once → `TOOL_RESULT ok` → `RUN_STATUS running` → `NODE_STATE running` → continue loop | engine (execution); WS publishes ACK/APPROVAL_RESOLVED + ledger |
| awaiting_approval | `APPROVAL_RESOLVE deny` | running | `TOOL_RESULT denied` (`E_APPROVAL_REQUIRED` code? see D5) → `RUN_STATUS running` → continue loop (failure of the run is not required) | engine |
| awaiting_approval | TTL elapsed, expiry CAS wins | failed | `decision='expired'` → task failed → dispatch terminal `RUN_STATUS failed error_code=E_APPROVAL_TIMEOUT`; **no** `APPROVAL_RESOLVED` | engine + dispatch |
| awaiting_approval | TTL elapsed but resolve/deny won the CAS first | per decision | re-read via `approval_decision` → apply the winner; never fail on a race loss | engine |
| awaiting_approval | `CANCEL_RUN` | cancelled | consume approval (`decision='cancelled'`) → token cancelled → dispatch maps `ToolCancelled` → terminal `cancelled` | engine |
| awaiting_approval | `KILL_WORKER` on the waiting task | cancelled | consume approval (cancelled) + run token cancelled → `NODE_STATE cancelled`, run `cancelled` | engine (`kill_worker` change) |
| awaiting_approval | process crash/restart | interrupted (existing recovery) | recovery: run `interrupted` + `error_code=E_APPROVAL_TIMEOUT` when it was awaiting; pending approvals expired; no waiters survive; no auto-approve | boot lane (WIP) |
| awaiting_approval | duplicate/late `APPROVAL_RESOLVE` | unchanged | gate returns `E_APPROVAL_REPLAY` / `E_APPROVAL_NOT_FOUND`; engine waiter already consumed; no second execution | gate/WS |
| awaiting_approval | wrong args at grant | failed or denied result (D4) | `verify_grant` → `E_APPROVAL_REQUIRED`; execute nothing | engine |
| any active | crash/restart | interrupted | existing recovery; approvals expired at boot | recovery (WIP) |

Illegal (must fail closed): executing a side effect while the run is not `running`; two resolvable approvals for one
`(run, tool_call_id)`; completing/failing directly from `awaiting_approval` (except timeout/restart/cancel/kill);
auto-approval on boot; a second `SUBMIT_PROMPT` while awaiting (`E_RUN_ACTIVE` — already enforced because
`store.latest_active_run` includes `awaiting_approval`, `store.rs:95-114`, and `execution.rs:426-428`).

---

## 6. Data flow (end-to-end; no paid call; deterministic)

```
SUBMIT_PROMPT (ws.rs:369) -> engine.submit_prompt -> run queued (execution.rs:410)
  -> dispatch_inner -> plan -> execute_task -> executor.execute/continue_with_tools
     -> reply.tool_calls non-empty
        -> execute_tool_turn (async in the target): counters -> next_turn -> per call
           -> plan_call -> validate_plan (WorkerAgent) -> publish TOOL_CALL
           -> permission == Read ? execute_read_only : suspend_for_approval
              a. bridge.request_model_tool_approval(run, task, call.id, call.name, call.args)
                 -> approvals row committed (nonce, args_sha256, expires_at=now+TTL) -> wire event
              b. waiters.arm(event.approval_id)                       // before publishing: no lost wake
              c. run_state::apply_run(status, Suspend) -> persist runs.status=awaiting_approval
                 task apply_task(task.state, Suspend) -> persist task awaiting_approval
              d. publish RUN_STATUS awaiting_approval, NODE_STATE awaiting_approval,
                 APPROVAL_REQUEST(event), (TOOL_CALL was published before suspend)
              e. tokio::select! { wake = rx => wake, _ = sleep_until(event.expires_at) => expire-CAS,
                                  _ = token.cancelled() => cancel-CAS }
        -> APPROVE wake: bridge.verify_tool_grant(run, tool, args, now)   // single-use executed_at marker
           -> tools::execute_side_effect(jail, shell_policy, ...)          // file_write | shell_exec only
           -> TOOL_RESULT ok + tool_turns row -> RUN_STATUS running + NODE_STATE running
           -> transcript push -> continue_with_tools -> final reply -> REPLY_DONE/task completed -> dispatch Completed
        -> DENY wake: TOOL_RESULT denied (no execution) -> task/run running -> transcript -> continue
        -> TIMEOUT: expire CAS; if lost -> approval_decision re-read and apply winner; else fail ApprovalTimeout
        -> CANCEL wake: consume approval (cancelled) -> Err(ToolCancelled) -> dispatch Cancelled
APPROVAL_RESOLVE (ws.rs:465) -> bridge.resolve_approval_ledgered (gate CAS + ledger row)
  -> publish ACK + APPROVAL_RESOLVED -> engine.notify_approval_resolved(&resolution) -> waiter wake
RECOVERY (main.rs:81) -> interrupted runs (+ E_APPROVAL_TIMEOUT when awaiting) -> AppState tools build ->
  expire_pending_approvals (WIP) -> no resolvable stale rows; no auto-approve
```

**No new database tables/columns are required for the engine wait:** `runs.status` (store), `tool_turns` (v4),
`approvals` (tools DDL) and the event bus carry everything; `chat_messages` stays as-is (assistant text persisted by
`complete_task` today; see D7 for the suspended-turn transcript).

---

## 7. Edge cases (each with required behavior and the failure it prevents)

| ID | Case | Required behavior | Prevents |
|---|---|---|---|
| E1 | **Deny** | Wake with `Deny`; synthesize `ToolExecution::denied(tool_call_id)` via `tool_loop::denied`; publish one `TOOL_RESULT denied`; persist one `tool_turns` row (UNIQUE); run/task return to `running`; loop continues within caps. No run failure, no side effect. | Operator denial mistaken for product failure; double/absent tool result |
| E2 | **Timeout** (TTL elapsed) | On the wait timer, CAS `decision='expired'` (`approval::expire`). If CAS wins → `Err(Refusal::ApprovalTimeout)` → task failed → dispatch terminal `RUN_STATUS failed E_APPROVAL_TIMEOUT`; no `APPROVAL_RESOLVED` (wire vocabulary is approve/deny); no side effect. If CAS loses → `approval_decision` re-read: allow → resume exactly as E6; deny → E1. | Losing a race to a valid resolve and failing a run that was approved; hanging waiters; executor unbounded wait |
| E3 | **Cancel while waiting** | `CANCEL_RUN` cancels the token; waiter select's cancel arm CASes `decision='cancelled'` (`approval::cancel_pending`); if the CAS loses to a resolve, re-read and do **not** execute (run is cancelled) — row is decided, grant unconsumed, never executed; return `ToolCancelled`; dispatch maps `cancelled` (no code). `KILL_WORKER` on an awaiting task must be allowed (`kill_worker` change), consume the approval, cancel the token, and reach `cancelled` via the same mapping (`run_state.rs:581-583`). | Resolvable approval after the run is gone; executing during/after cancel; `E_KILL_DENIED` dead end |
| E4 | **Crash/restart** | No in-process wait survives; boot: runs non-terminal → `interrupted` (+ `error_code=E_APPROVAL_TIMEOUT` when awaiting, WIP), pending approvals expired (WIP), never auto-approved; late resolve gets `E_APPROVAL_TIMEOUT`/`E_APPROVAL_NOT_FOUND`; no tool side effect can replay. | Auto-approval on restart; decision accepted for a dead run; duplicate execution |
| E5 | **Duplicate resolve** | Gate CAS makes exactly one winner; loser `E_APPROVAL_REPLAY`; waiter registry `wake_resolved` takes the sender once (`remove`), so a second wake is a no-op; no second execution. A resolve for an id with no waiter (run gone/crashed) returns `false` from notify and is only logged. | Two grants/executions; engine panic on missing waiter |
| E6 | **Unknown run / unknown approval** | `resolve` on an unknown id → gate `E_APPROVAL_NOT_FOUND` before any waiter/store write. Notify for an unknown approval → `false` (no panic). A wait for a run that no longer exists (waiter armed, then cancellation) is ended by the cancel arm, not by a fabricated resolution. | Silent success; orphaned waiters |
| E7 | **Lost-wake race** | Arm the waiter **before** publishing `APPROVAL_REQUEST`; `wake_resolved` uses `HashMap::remove`; if the row is already decided when the engine arms (should not happen for a freshly committed request but can after a retry/idempotent producer), the wait path re-reads `approval_decision` and applies it instead of waiting for a wake that already passed. | Run suspended forever after a fast operator decision |
| E8 | **Ledger failure after a committed decision** | WS arm returns `ERROR` although the gate decision committed and no ledger row exists. The engine was not notified; the wait timer later loses the expiry CAS (decision not NULL), re-reads `approval_decision`, and resumes/denies on the real decision. No fake ACK; no second decision path; the missing ledger row is surfaced by the WS error, not hidden. | A run stuck forever on a decided non-expired row; fake success |
| E9 | **Wrong/tampered arguments at execution** | Engine calls `verify_tool_grant(run, tool, args)`; mismatch → `E_APPROVAL_REQUIRED`, execute nothing. Default product behavior: fail the run by name (alternative: feed a refused tool result and continue — recorded D4). | Approving args A and executing args B; model tampering |
| E10 | **AllowSession** | `verify_grant` returns Ok without a single-use marker; every approved invocation may execute while the session grant matches `(run, tool, args_sha256)`; bounded by `MAX_CALLS_PER_RUN`/wait cap. No wire value (`allow_session` is programmatic only). | Multi-use grant being mistaken for single-use / unbounded loop |
| E11 | **Caps during suspension** | A wait counts as an attempted tool call: increment the persisted `runs.tool_calls` counter **before** suspending (not after execution) so deny/expiry cannot be retried for free; add `MAX_APPROVAL_WAITS_PER_RUN=8` and refuse `E_CALL_LIMIT` before a 9th request; `MAX_TOOL_TURNS=4` still bounds turns. | Unbounded model loop through denied/expired approvals; counters that under-count attempts |
| E12 | **Second prompt while awaiting** | Keeps single-run policy: `submit_prompt` refuses `E_RUN_ACTIVE` (store includes `awaiting_approval` in the active query, `store.rs:95-114`; engine check `execution.rs:426-428`). | Two concurrent runs sharing one gate/waiter registry |
| E13 | **Approve but run cancelled before execution** | Waiter may observe Resolved while the token is already cancelled (select fairness). Do **not** execute when the token is cancelled; return `ToolCancelled`. The `allow_once` grant stays unconsumed (no side effect) — safe, audited as decided-but-not-executed. | Side effect after cancellation |
| E14 | **Two pending approvals for one call** | Bridge producer dedupes same `(run,tool_call_id)`+same args and refuses different args (`tools_bridge.rs:271-311`); still add the W11-D11 unique partial index `approvals(run_id, tool_call_id) WHERE decision IS NULL` in `tools::approval::migrate` so a concurrent producer race cannot create two resolvable rows. | Two cards/decisions for one call; ambiguous which grant executes |

---

## 8. Likely bugs to prevent (predicted, with the guard that prevents each)

1. **Lost wake** (publish before arm) → arm first; E7 test: publish then resolve immediately from the same task.
2. **Double execution** (both `AllowOnce` executions) → `verify_grant` single-use `executed_at` CAS **before** the side
   effect; test replays the wake after execution → `E_APPROVAL_REPLAY`, no second file write.
3. **Lose-to-resolve timeout fails an approved run** → on expire-CAS loss, re-read decision and apply winner (E2).
4. **Deny treated as failure** → deny must yield `ToolExecution::denied`, continue the loop (E1); run fails only if
   caps/other refusals say so.
5. **Timeout fabricated as `APPROVAL_RESOLVED deny`** → never publish `APPROVAL_RESOLVED` for expiry/cancel
   (wire decision vocabulary is approve/deny; W11 D6/D7).
6. **Executing a tool while the run is not `running`** (e.g. after cancel) → gate the execution on
   `run_state::apply_run(run.status, Resume)` success and on `!token.is_cancelled()` (E13).
7. **Resolvable approval after terminal** → consume the row on cancel/kill/timeout (E2/E3) and expire at boot (E4).
8. **Waiter leak/poison** → `disarm` on every exit path; mutex poisoning maps to `E_INTERNAL_ERROR`, never panic; add
   a `waiters.len()==0` assertion in tests after each terminal path.
9. **Ledger double-write** → the engine must **not** call `append_approval_decision`; WS owns ACK/`APPROVAL_RESOLVED`,
   bridge owns the single ledger row (Wave-2/4 receipts). Engine-side test asserts exactly one row after approve.
10. **`TOOL_RESULT` duplication** → exactly one `finish_tool_call` per planned call on every branch (read, approve,
    deny, execute error, timeout result) — parameterize one helper; count events in tests.
11. **Counter under-count → free retry loop** → increment the call counter before suspension (E11).
12. **Engine calls gate while holding state locks/turns** → DB calls are short; never hold `active_run`/`waiters`
    mutex across `.await` (Rust will reject most of it; keep the arm/publish/select sequence lock-free).
13. **Boot order regression** → tools/expiry must be constructed before serving; `build_tools` before `build_engine`
    and before `serve`; the boot expiry must run after `recover_on_boot` (both WIP/hidden tests).
14. **Cancel path publishes duplicate terminal `RUN_STATUS`** → `cancel_run` currently publishes terminal `cancelled`
    while `dispatch` will also publish on `ToolCancelled`; either let `dispatch` own the terminal frame (and keep
    `cancel_run`'s write idempotent) or assert identical payloads; test counts frames.
15. **`kill_worker` on awaiting** → today refused (`KillDenied`); allowing it must also consume the approval, else a
    late approve could be accepted by the gate (no waiter, row expired only by TTL). Test the window.
16. **Engine with no tools wiring** (tests/older call sites) → suspension path must refuse `E_INTERNAL_ERROR` /
    `E_TOOL_NOT_ALLOWED` naming the missing wiring, never treat the call as allowed or silently complete.
17. **`approvals` table missing/migrations not run** → bridge producer returns a DB refusal; the engine must fail the
    task by name (not hang); tests seed via `ToolsBridge::migrate`.

---

## 9. File-by-file change list (implementation wave; no file is edited by this planner)

| # | File | Change (exact functions) | Notes/dependency |
|---|---|---|---|
| 1 | `core/src/execution.rs` | `RunEngine` fields (tools, ledger_dir, waiters, ttl, clock); `ApprovalWaiters`/`ApprovalWake`; builders `with_tools/with_approval_ttl/with_clock`; `notify_approval_resolved`; async `suspend_for_approval` + `wait_for_approval` + `execute_approved_side_effect`; `execute_tool_turn` async; `execute_task` await path; `kill_worker` awaiting arm; `cancel_run` approval consumption; `run_state::apply_run/apply_task` on suspension edges; counters before suspend | Depends on WIP reasoning field only by line drift; the biggest change (≈350–450 lines incl. tests’ seam) |
| 2 | `core/src/tools_bridge.rs` | `expire_approval`, `cancel_approval`, `approval_decision`, `verify_tool_grant` (thin delegates) | WIP already adds `expire_pending_approvals`; merge that first |
| 3 | `tools/src/approval.rs` | `expire`, `cancel_pending`, `decision_label`; unique partial index in `migrate`; extend tests | WIP already adds bulk `expire_pending` |
| 4 | `tools/src/executor.rs` | `execute_side_effect` (+ tests); re-export | file_write/shell_exec only; no shell spawn outside allowlist |
| 5 | `tools/src/lib.rs` | re-exports | trivial |
| 6 | `core/src/ws.rs` | Ok arm: `state.engine.notify_approval_resolved(&resolution)` after ACK/APPROVAL_RESOLVED; log when `false` | ~6 lines |
| 7 | `core/src/http.rs` | build tools first; pass tools+ledger into `build_engine`; same for `with_services` | WIP already edits `build_tools`; merge order |
| 8 | `core/src/tool_loop.rs` | `MAX_APPROVAL_WAITS_PER_RUN = 8` (+ doc) | new const only |
| 9 | `core/src/db.rs` | **No change** (no new columns; `tool_turns` v4 exists) — approvals live in the tools DDL (D3 keeps it) | if W11 D9 promotion is chosen instead: v5 migration + boot order; rejected in D3 |
| 10 | `core/src/recovery.rs` | WIP already: `interrupted` + `E_APPROVAL_TIMEOUT` for awaiting; add `expired_approvals` count to report only if absent | merge dependency |
| 11 | `core/tests/suspension.rs` (new) | the deterministic suite (§10) | new test file |
| 12 | `core/tests/tool_loop_engine.rs` | add one suspension harness test or keep new file for suspension only | existing harness has ScriptedExecutor; extend rather than duplicate |
| 13 | Web/e2e | **out of scope for this plan** (UI already receives extended payload; verify only) | — |

**Merge order (critical):** (a) WIP boot-expiry/reasoning lanes commit; (b) tools `expire/cancel/decision` +
`execute_side_effect`; (c) bridge delegates; (d) engine + `run_state` wiring; (e) ws notify; (f) http wiring;
(g) tests. Never land (d) before (b)/(c) or the engine will call missing methods.

---

## 10. Tests and commands (deterministic, offline; no paid provider)

New `backend/crates/hiverouter-core/tests/suspension.rs` (MockExecutor scripted: one side-effect `file_write` call then
a final answer; `TempDir` workspace; `ApprovalGate`-seeded rows via the bridge; injected clock; short TTL):

1. `suspend_publishes_awaiting_approval_and_approval_request` — run status/task state become `awaiting_approval`; one
   `APPROVAL_REQUEST` with nonce/run/task/tool_call/args_sha256/expires_at; one `TOOL_CALL`; no file written.
2. `approve_resumes_and_executes_the_file_write_exactly_once` — resolve via WS command/engine notify; exactly one
   `TOOL_RESULT ok`, one `tool_turns` row, file content matches; second wake/replay → `E_APPROVAL_REPLAY`, still one row.
3. `deny_returns_denied_result_and_run_continues` — one `TOOL_RESULT denied`, run reaches a final model reply and
   `completed`, workspace unchanged.
4. `timeout_fails_run_with_e_approval_timeout_and_no_side_effect` — injected clock advances past TTL; run `failed`
   code `E_APPROVAL_TIMEOUT`; no `APPROVAL_RESOLVED`; row `decision='expired'`; no file.
5. `cancel_while_awaiting_cancels_run_and_consumes_approval` — `CANCEL_RUN`; run `cancelled`; row consumed
   (`cancelled`); later resolve → `E_APPROVAL_REPLAY`/`E_APPROVAL_NOT_FOUND`; no file.
6. `kill_worker_while_awaiting_cancels_without_side_effect` — new `kill_worker` arm.
7. `timeout_cas_loss_to_resolve_still_resumes` — resolve just before the timer, waiter delayed; engine re-reads the
   decided row and executes once (E2/E7).
8. `concurrent_resolve_has_one_winner_and_no_second_execution` — two `APPROVAL_RESOLVE` frames; one ACK/event, one
   `E_APPROVAL_REPLAY`; one execution.
9. `second_prompt_while_awaiting_is_refused_e_run_active`.
10. `restart_marks_awaiting_run_interrupted_and_expires_pending` — build state on a DB whose run is awaiting; recovery
    marks interrupted + code; boot expiry consumes pending; no auto-approve (WIP dependency).
11. `wait_cap_refuses_the_ninth_request` — `E_CALL_LIMIT` before a 9th wait.
12. `wrong_args_grant_refused_before_side_effect` — engine plan args differ from approved; `E_APPROVAL_REQUIRED`, no write.

Commands (Lead runs; this planner does not):
```powershell
cd backend
$env:PATH = "$env:USERPROFILE\.cargo\bin;$PWD\..\tools\w64devkit\bin;$env:PATH"
cargo test -p hiverouter-core --test suspension
cargo test -p hiverouter-core --test tool_loop_engine
cargo test -p hiverouter-tools --lib approval::tests
cargo test -p hiverouter-core --test ws_transport
cargo test --workspace --locked
```
Failure-mode test required for each of E1–E14 where listed; every test must fail if the corresponding guard is removed
(plant the defect once per `/forge` protocol). Log HEAD + command + exit code for every claim (`run-gates` S7 rule).

---

## 11. Rollback

- **Feature-flag by absence:** engines built without `with_tools` (all current test call sites) keep the read-only
  behaviour and refuse suspension with a named `E_INTERNAL_ERROR`; the production `http.rs` wiring is the only switch.
  Rolling back = revert http wiring + engine changes; approvals table/rows remain harmless audit data.
- **Data:** no new columns/tables are added by the engine; the only schema edit proposed is the unique partial index in
  the tools DDL (`CREATE UNIQUE INDEX IF NOT EXISTS`), reversible with `DROP INDEX IF EXISTS approvals_one_pending_per_call`.
  Existing approval rows are retained (audit truth); decisions are already durable.
- **Code inverse per file:** revert `execution.rs` (single file), `ws.rs` notify (6 lines), http builder change,
  tools expire/cancel/decision functions + index, executor `execute_side_effect` (dead code without callers), test file
  deleted. No commit is made by the planner; the implementing wave commits as one reviewable change.
- **Boot safety:** if recovery/expiry WIP must be reverted independently, suspension must not ship: without boot
  expiry a restarted process could accept a late decision (no engine waiter, but row would be decided for a terminal
  run) — so boot expiry is a **precondition**, not an option (D8).

---

## 12. What will work / what will not work after this plan is implemented

**Will work (design target, to be verified by the tests):** a single-run localhost engine that, when the model asks for
`file_write`/`shell_exec`, commits an approval row with nonce/args/TTL, publishes one `APPROVAL_REQUEST`, pauses the run
in `awaiting_approval`, and resumes on approve to execute the exact approved side effect once; deny feeds a denied tool
result and the model continues; TTL expires to `failed E_APPROVAL_TIMEOUT`; cancel/kill consume the approval;
restart interrupts the run and expires pending approvals; duplicate resolves cannot double-execute; the read-only loop
and all pre-1.2 behaviour remain unchanged when no approval tool is used.

**Will not work / out of scope:** crash-resume of a suspended turn (D1 — the run becomes interrupted; no executor
cursor is persisted); spawn_agent/report_result/finish are not executable side effects (loop-owned; only
file_write/shell_exec are reachable at WorkerAgent tier); no batching of approvals per turn (per-call binding); no TTL
extension/renewal; no websocket “pending approvals on connect” replay beyond the existing read-only
`pending_approval_events` (UI/command work not in this plan); a second concurrent run remains refused; no new refusal
codes are added (uses `E_APPROVAL_*`, `E_TOOL_NOT_ALLOWED`, `E_CALL_LIMIT`, `E_RUN_ACTIVE`, `E_RUN_TERMINAL`).

---

## 13. Decisions (with rejected alternative) — `/forgewright-plan` + `/forgewright-lead`

- **D1 — In-process suspension only; no crash-resume of a turn.** Persisted truth = run/task status + approvals row +
  tool_turns. Crash → recovery interrupts + expires. *Rejected:* persist executor cursor (stream offset, partial
  tool-call assembly, reasoning buffer) — large, risky, no 1.2 need.
- **D2 — Waiter registry keyed by `ApprovalId` in the engine; arm before publish.** *Rejected:* broadcast per run
  (lifetime/identity ambiguity); resolving in the engine (would duplicate the gate owner).
- **D3 — Approval table stays tools-owned (bridge DDL + boot migrate); no core v5 migration.** Add the W11-D11 unique
  partial index in `tools::approval::migrate`. *Rejected:* promoting the table into `db::MIGRATIONS` (larger schema
  surface, cross-owner migration, risk to the sealed 1.1 line) — revisit only if/when history queries need SQL joins.
- **D4 — Approve executes once, deny continues, wrong-args at execution fails the run by name** (default). *Rejected:*
  deny = run failure (destroys recoverable-refusal semantics); wrong-args = silent retry (would mask model change).
- **D5 — Deny tool result code:** publish `TOOL_RESULT outcome=denied` with `code=null` and message “denied by
  operator” (W11 D5) — the contract outcome `denied` carries the semantic; no synthetic refusal code. *Rejected:*
  `E_APPROVAL_REQUIRED` on deny (conflates “needs approval” with “was denied”).
- **D6 — Engine drives `run_state` edges for the approval path only.** Other status writes stay as-is in this wave
  (avoids a risky refactor); the table remains the authority for suspend/resume/deny/timeout/cancel/restart.
  *Rejected:* full engine refactor onto `run_state` in the same change.
- **D7 — The suspended assistant turn is persisted as plain assistant text via `store.append_message` before the wait**
  (no `chat_messages.meta_json` column in this wave; tool-call evidence lives in TOOL_CALL/tool_turns). *Rejected:*
  adding `meta_json`/`turn` now (schema change + history/UI ripple; W11 D9 deferred).
- **D8 — Boot expiry + interrupted-code are preconditions, owned by the WIP lane.** Suspension does not land without
  them. *Rejected:* relying on TTL only after restart (a decided non-expired row could be accepted for a terminal run).
- **D9 — Timeout, cancel, kill never publish `APPROVAL_RESOLVED`.** Wire decisions stay approve/deny; terminal events
  express the outcome. *Rejected:* fabricate a deny frame for timeout/cancel.
- **D10 — Side effects execute only through `tools::executor::execute_side_effect` after `verify_grant`.** The engine
  never calls `jail.write`/`shell::execute` directly. *Rejected:* engine-level filesystem/process calls (bypasses the
  tools owner and the jail/shell policy).
- **D11 — No new refusal code; approval-wait cap reuses `E_CALL_LIMIT`** (existing frozen code) rather than inventing
  `E_TOOL_LOOP_LIMIT` in this wave. *Rejected:* adding a code (contract patch cost/policy not in scope).
- **D12 — Deterministic seams:** `with_approval_ttl` + `with_clock` on `RunEngine`; tests never sleep for 120 s.
  *Rejected:* `tokio::time::pause` only (needs test-util feature; explicit seam is simpler and already used for caps).

---

## 14. Naming audit (`/deep-naming`; plan only — no rename applied)

Vocabulary observed: **approval request** (row/wire event), **approval resolution/decision**, **grant**, **wait**,
**suspend/resume**, **waiters**, **wake**, **ledger**, **tool call/result**, **run/task state**. The project uses
`request`/`resolve`/`grant` consistently in `approval.rs` and `pending_*` for open rows.

| Current | Proposed | Files/call sites | Risk / verdict |
|---|---|---|---|
| `hiverouter_tools::approval::ApprovalRequest` **and** `hiverouter_contract::event::ApprovalRequest` (two different things under one name) | tools row → `ApprovalRecord`; wire event stays `ApprovalRequest` | `tools/src/approval.rs` (all bridge/producer code), `tools/src/lib.rs`; `grep -c 'ApprovalRequest'` = 20+ | **Misleading (highest).** Public API break (E_PUBLIC_API) → propose only in this plan, hand to batch refactor after 1.2; bridge can alias `pub use ... as ApprovalRecord` later |
| `ToolsBridge::resolve_approval_ledgered` | `resolve_approval_with_evidence` | `tools_bridge.rs:201`, `ws.rs:473`, tests | Long/opaque (“ledgered” hides that it writes the run ledger once) → confirm before rename; not blocking |
| `RunEngine::execute_tool_plan` (will become the router that may suspend) | `route_tool_plan` or split `execute_read_only_plan` + `suspend_side_effect_plan` | `execution.rs:875` | A verb that hides `.await`/side effect — split names (proposed) |
| `Effect::RequestApproval` vs bridge `request_model_tool_approval` vs engine `notify_approval_resolved` | keep all three; the row/commit vs wake vs effect are distinct concepts | `run_state.rs:219`, `tools_bridge.rs:271`, new engine method | Consistent once documented: **request = commit row**, **notify = wake**, **effect = persist+publish** |
| `ApprovalWake` (proposed) | keep (explicit event-shaped name) | new | Good: no generic `Outcome`/`Event` |
| `ApprovalWaiters` (proposed) | keep | new | Good: plural owner of `ApprovalId → sender` |
| `pending_approvals` vs `pending_approval_events` | keep pair (row vs wire projection) | `tools_bridge.rs:231,342` | Clear distinction, keep |
| `ApprovalGate::verify_grant` | keep | `approval.rs:653` | Accurate; the docs already say it does not execute |
| `E_TOOL_NOT_ALLOWED` reuse for non-executable tools | keep | tool contract | Fine for this wave; D11 covers caps |
| `decision_label` (proposed reader) | `approval_decision` | new | Align with bridge method name (`approval_decision`) |

No rename is applied by this planner (read-only); public-API renames require approval (`E_PUBLIC_API`).

---

## 15. Evidence-check table (`/evidence-check`: claim → evidence → status)

| Claim | Evidence | Status |
|---|---|---|
| Engine has no suspension/wait path | `execution.rs:535-583,581-654,964-1096`; no `awaiting_approval` producer; grep `notify_approval`/`ApprovalWaiter` = 0 | **VERIFIED (read)** |
| Approval producer exists but is uncalled | `tools_bridge.rs:271-311`; grep call sites = bridge tests only | **VERIFIED (grep/read)** |
| WS resolve does not wake the engine | `ws.rs:465-483`; no engine call in that branch | **VERIFIED (read)** |
| `run_state` approval edges exist and are unused | `run_state.rs:131-156,205-250,465-538,568-619`; `grep run_state execution.rs` = 0 | **VERIFIED** |
| TTL/single-use grant semantics exist | `approval.rs:26,415-514,621-652,653-…` | **VERIFIED (read)** |
| No side-effect executor exists | `tools/executor.rs` HEAD exports only read-only; `shell::execute`/`jail::write` exist as primitives | **VERIFIED (read)** |
| HEAD does not expire approvals at boot | HEAD `approval.rs` no `expire_pending`; `recovery.rs:50-64` no approvals; `http.rs:107-114` migrate only | **VERIFIED (HEAD read)** |
| WIP lane adds boot expiry/recovery code | `git diff` shows `expire_pending`, `expire_pending_approvals`, `http.rs` call, `recovery.rs` error code | **READ (uncommitted WIP; may change)** |
| Engine has no bridge/gate handle | `execution.rs:256-285` fields; `http.rs:183-185` build order | **VERIFIED** |
| Engine call cap is inert | `execution.rs:585,970` literal `check_calls(1)`; `caps.rs:66-71` pure comparison | **VERIFIED** |
| `store.latest_active_run` counts awaiting as active | `store.rs:95-114` status list includes `awaiting_approval` | **VERIFIED** |
| Exact list items 1/2/5/9 | not present in repo (searched) | **UNKNOWN (A0 assumption recorded)** |
| Any runtime behaviour of the target | not implemented, no build/test run by this planner | **UNVERIFIED** |
| UI can resolve on the current tree | not re-read this pass | **UNKNOWN** |

---

## 16. UNKNOWN register (what would settle each)

| ID | Unknown | Settle by |
|---|---|---|
| U1 | Exact content of “items 1,2,5,9” | Lead provides the list; reconcile §3/§9 mapping |
| U2 | Whether WIP boot-expiry/recovery/reasoning lanes land unchanged | Lead commit/integration; re-diff before implementation |
| U3 | Current UI decode of the extended `APPROVAL_REQUEST` + resolve flow | read `web/src/api/contract.ts`, `web/src/state/store.ts`, `ApprovalOverlay.tsx` at implementation time |
| U4 | Whether `tokio::sync::oneshot` is available under the pinned tokio features in the core crate | `Cargo.toml` workspace features include `sync` (verified) — confirm crate-level `tokio` features at build |
| U5 | Event ordering guarantees of `EventBus`/WS replay for the APPROVAL_RESOLVED→TOOL_RESULT sequence | read `bus.rs` publish/sequence semantics + a listener test |
| U6 | Whether `approvals` unique partial index is acceptable to the WIP/other lanes (DDL ordering) | Lead decision; it is additive/idempotent |
| U7 | Whether the model tool specs actually offer `shell_exec`/`file_write` at WorkerAgent tier today | `tool_loop.rs:203-226` + `tool.rs` allowlist (reads suggest yes); confirm in tests |
| U8 | RecoveryReport extension for expired approvals (if WIP does not add it) | read WIP final `recovery.rs`/`tools_bridge.rs` |

---

## 17. Skill application evidence

- **/research**: §1 question→evidence note with path:line; local code read first; no web source needed; unknowns listed.
- **/forgewright-plan**: all required sections present (outcome §0/§12, current behavior §2, decisions §13, file-by-file
  §9, failure modes §7/§8, tests §10, rollback §11, out-of-scope §12).
- **/forgewright-lead**: current code read before hypothesis; each claim labelled VERIFIED/READ/UNVERIFIED; no fake
  implementation; no commit; no build.
- **/deep-naming**: §14 table with blast radius and E_PUBLIC_API marks; no rename applied.
- **/truthseeker**: single-owner invariants stated as falsifiable sentences (one decision → one ledger row; one grant →
  at most one side effect; one wait → one wake); crash window and residual risks surfaced (§7 E8, §13 D1).
- **/scientific-thinking**: competing hypotheses for “why no suspension” narrowed to the missing engine owner using
  decisive evidence (grep/read), alternatives (WIP already implements it) checked and rejected with evidence.
- **/forge**: predicted defects each paired with a RED test that fails if the guard is removed (§8, §10); minimal-change
  boundaries; no symptom patches.
- **/evidence-check**: §15 claim table; unverified/unknown items kept instead of promoted.

---

## 18. References
`plans/waves/W11-SUSPENSION-DESIGN.md` (D1–D12, §4 transitions, §5 protocol, §9 semantics, §11 tests);
`plans/waves/W12-INTERFACES.md` (frozen APIs); `plans/waves/W10-TOOL-LOOP-DECISION.md` (B1–B8);
`plans/waves/README.md` (wave status/evidence rules); contract `event.rs:443-590` (23 events), `command.rs`
(`ApprovalResolve`), `refusal.rs:179-193`; tools `approval.rs`; core `execution.rs`, `run_state.rs`, `tools_bridge.rs`,
`ws.rs`, `db.rs`, `http.rs`, `recovery.rs`, `store.rs`, `caps.rs`, `tool_loop.rs`; tests
`core/tests/tool_loop_engine.rs`, `core/tests/run_lifecycle.rs`, `core/tests/ws_transport.rs`,
`core/tests/approval_ledger.rs`.
