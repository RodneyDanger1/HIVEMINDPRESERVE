# RUN 2 — ENGINE CODE (WP-03): suspension / resume / wait / event wiring

**Planner:** Code Research Planner 2 (Run 2) · **Mode:** READ-ONLY for product code (this file is the only write; no source edit, no build, no test, no commit)
**Skill path used:** `/forgewright-lead` (read → root cause → ordered hunks → unverified split) and `/logic-gatekeeper` (Phase 1 below; no product mutation).
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1`
**Baseline at recon start:** HEAD `90cb7a7` ("feat(1.2): reasoning passback, approval boot expiry, recovery timeout; foresight run-1 plans/ledger"), clean tree (`git status --porcelain` empty).
**Target:** WP-03 from `plans/foresight/COMPILED-LEDGER.md:66-74`, implemented **after** the Run-1 tree and **after** the WP-01/WP-02 prerequisite commits.
**Predecessor blueprint:** `plans/foresight/CODE-APPROVAL-ENGINE.md` (1,351 lines, written against HEAD `37f0e7c` + uncommitted WIP). Run-1 committed that WIP as `90cb7a7`, so the predecessor's WIP line offsets are now the stable HEAD anchors verified below; this document supersedes its WIP-merge section (`:140-155`) and records the ledger resolutions.
**Source labels:** `[READ path:line]` read this session; `[READ-WIP]` uncommitted working-tree content read during recon; `[LEDGER §]` `COMPILED-LEDGER.md`; `[BLUEPRINT §]` `CODE-APPROVAL-ENGINE.md`; `[INFERRED]` derived; `[UNVERIFIED]` not established.

---

## 0. Inputs and baseline

### 0.1 Files read this session

`plans/foresight/{RUN-1.md, COMPILED-LEDGER.md, CODE-APPROVAL-ENGINE.md, CODE-SIDE-EFFECTS.md}`; `backend/crates/hiverouter-core/src/{execution.rs, run_state.rs, tools_bridge.rs, ws.rs, http.rs, db.rs, main.rs, tool_loop.rs}`; `backend/crates/hiverouter-tools/src/{approval.rs, error.rs, executor.rs, lib.rs}`; `backend/crates/hiverouter-core/tests/{tool_loop_engine.rs, run_lifecycle.rs, ws_transport.rs, http_boot.rs, lead_bugcheck_boot.rs}`; `backend/Cargo.toml`; `git show --stat HEAD`; `git status`; `git diff` of the working tree.

### 0.2 Baseline vs live workspace (VERIFIED at recon 2026-11-06 ~18:52-18:58 CT)

| Fact | Evidence |
|---|---|
| HEAD `90cb7a7` committed Run-1 Lead changes: `execution.rs +5` (reasoning), `http.rs +15`, `provider_executor.rs +20`, `recovery.rs +13`, `tools_bridge.rs +14`, `tests/run_lifecycle.rs +1`, `tests/tool_loop_engine.rs +4`, `approval.rs +211` | `git show --stat HEAD` |
| Predecessor blueprint anchors are now stable HEAD anchors (execution: `RunEngine` `:260`, `new` `:305`, `with_tool_workspace` `:345`, `with_services` `:352`, `dispatch` `:539`, `dispatch_inner` `:585`, `execute_tool_turn` `:684`, `execute_tool_plan` `:879`, `finish_tool_call` `:934`, `execute_task` `:968`, `complete_task` `:1104`; file 1,130 lines) | `[READ execution.rs:260-1130]` |
| **During this recon a different wave (WP-01/WAVE-SE) began uncommitted edits**: `tools_bridge.rs +76`, `approval.rs +73`, `tools/lib.rs +6`; new `tools/qa/apply_wp01.py`; untracked `plans/foresight/RUN2-WEB-CODE.md` (another planner in parallel) | `git status --porcelain`; `git diff --stat`; `git diff` |
| Those in-flight edits add exactly the engine prerequisites: `ToolsBridge::{with_approval_ttl, expire_approval(&str,u64)->bool, cancel_approval(&str,u64)->bool, approval_decision(&str)->Option<String>, approval_count_for_run(RunId)->u64, verify_tool_grant(RunId, ToolId, args, u64)}`; `tools/approval.rs::{expire, cancel_pending, decision_label, count_for_run}`, unique partial index `approvals_one_pending_per_call`; `tools/lib.rs` re-exports | `[READ-WIP]` diff |
| **WP-02 is NOT landed**: no `backend/crates/hiverouter-tools/src/side_effects.rs`; `executor.rs` has only `execute_read_only`; `tools/lib.rs` exports no `execute_approved` | file inventory `[READ]`; `tool.rs/executor.rs` grep |
| Run-1 commit contains **no** engine suspension: `execute_tool_plan` still refuses non-read tools; no `awaiting_approval` producer; no `notify_approval_resolved`; `run_state` referenced 0 times from `execution.rs` | `[READ execution.rs:879-920,499-537]`; predecessor `§13` lines re-verified |
| `run_state` approval edges/effects unchanged: `Effect` `:205-266`, `apply_run` `:465`, `apply_task` `:549`; run `Suspend` legal only from `Running`; task `AwaitingApproval+Kill→Cancelled` | `[READ run_state.rs]` |
| `tokio` features include `sync`, `time`, `rt-multi-thread` (oneshot/`select!`/timeout available) | `[READ backend/Cargo.toml:17]` |
| `Database` exposes only synchronous `with_connection/with_connection_refusal` closures; `ToolsBridge` keeps `gate`, `jail`, `db` private | `[READ db.rs:389,405; tools_bridge.rs:35-45]` |
| Boot order: `recover_on_boot` → `AppState::with_graph` (build engine, then build tools today) → `serve` | `[READ main.rs:81,123,160; http.rs:189-285]` |
| Existing test harnesses: `tool_loop_engine.rs` has **11** `#[tokio::test]` and builds engines with `with_tool_workspace` only (no `.with_tools`); its `drive` stops at the **first** `RUN_STATUS`; `run_lifecycle.rs` has 7 tests, two stop at the first `RUN_STATUS` (`run_lifecycle_plans_and_dispatches:198`, `provider_error_preserves_partial_state:269-272`) | `[READ tests]` |

### 0.3 What changes vs the Run-1 predecessor blueprint

1. `[BLUEPRINT §2]` merge notes are obsolete: the reasoning/recovery/boot-expiry WIP is committed at `90cb7a7`; the engine hunks below are written against current HEAD anchors.
2. Ledger contradictions **C1, C2, C4, C5, C6, C7, C9** are resolved in §2 and reflected in the hunks; **C1a** (WP-02 grant-check interface) is a new hard dependency found at recon.
3. The predecessor's `resume_and_execute` called `verify_tool_grant` **and** `execute_side_effect` separately (`[BLUEPRINT §5.3]`). Per `[LEDGER C1]`, the engine now calls **one** bridge delegate that owns preflight → grant CAS → effect; the exact required WP-02 interface is in §3.3.
4. WAIT-cap counting, terminal-frame ownership and the run `Start` transition are retained from the predecessor (`[BLUEPRINT D2, D7, D12]`).
5. Frame-counting test reconciliation is specified exactly (§7.2) because the new `RUN_STATUS running` frame breaks two `run_lifecycle` tests and the `tool_loop_engine::drive` helper today.

---

## 1. Phase 1 (logic-gatekeeper record)

```json
{
  "phase1": {
    "architecture": "RunEngine in core/src/execution.rs owns the model tool-turn loop and dispatch; run_state.rs is the pure transition authority; ToolsBridge owns the approvals gate/DB/ledger; ws.rs owns APPROVAL_RESOLVE dispatch and SERVER_HELLO; http.rs builds tools, engine and AppState in one construction order. WP-01 primitives (tools_bridge delegates + tools/approval.rs CAS/index) are landing in the working tree from another wave; WP-02 (side-effect executor) is not landed.",
    "dataFlow": "Untrusted model output (assistant tool calls) enters via StreamedReply.tool_calls -> execute_tool_turn -> tool_loop validation -> (only for Mutate/Execute when bridge+jail are wired) suspension: request_model_tool_approval commits a nonce-bound row, waiter arms, run/task transitions persist awaiting_approval, APPROVAL_REQUEST publishes, then wait on oneshot/token/TTL; operator decision arrives via ws.rs APPROVAL_RESOLVE -> gate CAS + ledger -> notify waiter. Approved -> resume Running -> granted executor verifies single-use grant then executes once; denied -> Running + denied result; timeout -> expire CAS + failed E_APPROVAL_TIMEOUT; cancel/kill -> row cancelled + task cancelled + one cancelled TOOL_RESULT; dispatch terminalizes only when no owner already persisted the terminal state.",
    "edgeCases": [
      "approve wake arrives before the waiter is armed (must arm before publishing; re-read the committed decision after publish to apply a winner)",
      "timer wins expiry CAS while the operator resolve committed concurrently (lost CAS must re-read the decision label and apply allow/deny/expired/cancelled, never fabricate APPROVAL_RESOLVED)",
      "CANCEL_RUN persists terminal cancelled and publishes it, then the waiter wakes on token cancel; dispatch must not publish a second terminal RUN_STATUS",
      "ACK-only/duplicate resolve: notify_approval_resolved returns false legitimately; the command still ACKs the committed decision and the engine timer path re-reads",
      "ninth approval request in one run must refuse E_CALL_LIMIT before committing a row",
      "wrong/tampered args at resume: verify fails E_APPROVAL_REQUIRED before any effect (no execution)",
      "process restart while awaiting: recovery marks the run interrupted with E_APPROVAL_TIMEOUT and the boot reaper expires the row unconditionally; no resume/auto-approve"
    ],
    "rootCause": "The engine still has no production suspension seam: execute_tool_plan refuses every non-read call, run_state approval edges are unused, the approval producer has no caller, APPROVAL_RESOLVE never wakes an in-process wait, and the extra RUN_STATUS running frame required by the run_state Start transition will break frame-counting tests; the ordered hunks must add the waiter/bridge/turn/wiring path, reconcile those tests, and must not start before the WP-01/WP-02 prerequisite commits and the WP-02 grant-check interface are fixed."
  }
}
```

**Why the contradiction fails closed after the hunks (Phase 2 statement):** every wait has exactly one wake source (oneshot/expiry CAS/token) and an RAII disarm; terminal wait paths persist the terminal state before returning `Err`, and `dispatch` only terminalizes when no owner did; grant verification remains the single execution authority inside the tools layer (preflight before grant); a no-bridge/no-jail engine keeps the frozen `E_TOOL_NOT_ALLOWED` refusal; no new refusal code, event, command, ledger write, or DB migration is introduced by the engine.

---

## 2. Ledger resolutions applied in this blueprint

| Ledger id | Resolution applied | Where |
|---|---|---|
| **C1** waiter/executor identity | Adopt **R1 `ApprovalWaiters`** in `execution.rs` (inline in-process wait; no dispatch outcome change) **and** one tools-owned granted executor. Engine keeps read-only pre-checks (`validate_plan`) and calls **one** bridge delegate for preflight→verify→effect; engine does not call `verify_grant` twice. | H2, H6, §3.3 |
| **C2** side-effect rollout gate | Engine code is identical for both variants: suspension requires `self.tools.is_some() && self.tool_workspace.is_some()`; a default-off env flag, if the Lead chooses it, gates only whether `http.rs` calls `.with_tools(...)` (H11). No engine flag field. Recommended: production always wires (R1 D3) because the bridge is already wired for approvals; if the Lead prefers default-off, only H11 condition changes. | H11, §8 |
| **C4** wrong-args refusal | Hash mismatch at grant verification is **`E_APPROVAL_REQUIRED`** (never `E_APPROVAL_DECISION_INVALID`). Test #12 named accordingly. | §7.1 |
| **C5** recovery status | Recovery asserts run status **`interrupted`** + `error_code = E_APPROVAL_TIMEOUT`, not `failed`. | test #10 |
| **C6** cancel result | Cancel/timeout produce **exactly one** `TOOL_RESULT` (`cancelled` / `timeout`) and one `tool_turns` row per planned call. | H6, test #5/#4 |
| **C7** restart resume | No crash-resume test: restart marks interrupted, boot reaper expires future-TTL pending rows, a late resolve returns `E_APPROVAL_TIMEOUT`, no execution. | test #10 |
| **C8** test names/counts | Freeze the 14 names in §7.1; existing counts: `tool_loop_engine` 11 tests, `run_lifecycle` 7, `approval_ledger` 2, `ws_transport` 13 — do not invent new counts. WP-04 owns the separate `tool_loop_matrix.rs`. | §7 |
| **C9** file collision | Strict order: WP-01 + WP-02 committed → **this engine wave** (single writer of `execution.rs`, `tool_loop.rs`, `tools_bridge.rs`, `ws.rs`, `http.rs`, `tests/suspension.rs`, and the two test files) → WP-05 replay. WP-05 only edits `send_hello_and_replay`/its call sites in `ws.rs` (disjoint from the `ApprovalResolve` arm) and appends to `tools_bridge.rs`; still, one writer at a time. Observed live collision: the in-flight WP-01 editor is touching **engine-owned** `tools_bridge.rs` — serialize/commit before H1. | §8 |
| **C1a** (new, from recon) | `CODE-SIDE-EFFECTS.md`'s `execute_approved(GrantedExecution<'_>, …)` takes `&Connection` + `&ApprovalGate`; the bridge owns `db`/`gate` and only exposes synchronous closure APIs, so the engine cannot construct `GrantedExecution` and a DB lock must not be held across the async shell spawn. **Required interface adjustment before H6 is coded** (§3.3, option A recommended). | §3.3, §11 blocker |

---

## 3. Prerequisite contract (H0 — external, must be committed before H1)

### 3.1 WP-01 symbols (observed in-flight; expected after the WP-01 commit)

```rust
// hiverouter_tools::approval (additive)
pub fn expire(conn: &Connection, approval_id: &str, now_unix_secs: u64) -> Result<bool, ToolError>;
pub fn cancel_pending(conn: &Connection, approval_id: &str, now_unix_secs: u64) -> Result<bool, ToolError>;
pub fn decision_label(conn: &Connection, approval_id: &str) -> Result<Option<String>, ToolError>;
pub fn count_for_run(conn: &Connection, run_id: RunId) -> Result<u64, ToolError>;
// migrate(): CREATE UNIQUE INDEX IF NOT EXISTS approvals_one_pending_per_call
//             ON approvals (run_id, tool_call_id) WHERE decision IS NULL;

// hiverouter_core::ToolsBridge (all on &self)
pub fn with_approval_ttl(self, ttl_secs: u64) -> Self;                 // test seam only
pub fn expire_approval(&self, approval_id: &str, now_unix_secs: u64) -> Result<bool, Refusal>;
pub fn cancel_approval(&self, approval_id: &str, now_unix_secs: u64) -> Result<bool, Refusal>;
pub fn approval_decision(&self, approval_id: &str) -> Result<Option<String>, Refusal>;
pub fn approval_count_for_run(&self, run_id: RunId) -> Result<u64, Refusal>;
pub fn verify_tool_grant(&self, run_id: RunId, tool: ToolId, args: &serde_json::Value,
                         now_unix_secs: u64) -> Result<(), Refusal>;
```
Preflight grep (must all match before H1; if any is missing → **BLOCKED**, do not code):
`grep -n "fn with_approval_ttl\|fn expire_approval\|fn cancel_approval\|fn approval_decision\|fn approval_count_for_run\|fn verify_tool_grant" core/src/tools_bridge.rs` and the four `approval.rs` functions + the index DDL. **Do not re-implement them in the engine wave; `tools_bridge.rs` ownership is engine but these hunks arrive from WP-01.**

### 3.2 WP-02 symbols (NOT landed at recon — hard blocker)

Required from WP-02/WAVE-SE before the engine wave:
```rust
// hiverouter_tools::side_effects (new module; WP-02 owns)
pub struct ApprovedOutcome { /* final bounded text or named refusal; WP-02 chooses shape */ }
// plus the interface in §3.3
```
`tools/lib.rs` must export the module and `side_effects::execute_approved`; `tools/executor.rs` stays read-only.

### 3.3 Required `execute_approved` grant-check interface (C1a — Lead/WP-02 decision)

The engine cannot pass `&Connection`/`&ApprovalGate` to the tools crate (`ToolsBridge` fields private; `Database` exposes only sync closures). Recommended adjustment to `CODE-SIDE-EFFECTS.md:122-127` keeps its load-bearing order (registry → read-only preflight → grant CAS → effect) with a synchronous verify callback:

```rust
// hiverouter_tools::side_effects — OPTION A (recommended; engine calls via one bridge delegate)
pub async fn execute_approved<F>(
    jail: &WorkspaceJail,
    shell_policy: &ShellPolicy,
    registry: &ToolRegistry,
    tier: AgentTier,
    run_id: RunId,
    tool: &str,
    args: &serde_json::Value,
    cancellation: Option<&CancellationToken>,
    now_unix_secs: u64,
    verify_grant: F,                     // called AFTER all preflights, immediately BEFORE the effect
) -> Result<ExecutedTool, ToolError>
where
    F: FnOnce(ToolId) -> Result<(), Refusal>;
```
Bridge delegate (engine wave, `tools_bridge.rs`, placed immediately after `verify_tool_grant`; do not edit WP-01 hunks above it):

```rust
/// Executes exactly the approved side effect through the tools-owned single
/// authority: registry -> read-only preflight -> grant CAS -> one effect.
/// The DB lock is taken only inside the synchronous grant closure.
pub async fn execute_approved(
    &self,
    shell_policy: &hiverouter_tools::ShellPolicy,
    registry: &hiverouter_tools::ToolRegistry,
    tier: hiverouter_tools::AgentTier,
    run_id: RunId,
    tool: &str,
    args: &serde_json::Value,
    cancellation: Option<&tokio_util::sync::CancellationToken>,
) -> Result<hiverouter_tools::ExecutedTool, Refusal> {
    let jail = self.jail.as_ref().ok_or(Refusal::FileTreeUnavailable)?;
    let now = Timestamp::now().unix_secs().unwrap_or(0);
    hiverouter_tools::side_effects::execute_approved(
        jail, shell_policy, registry, tier, run_id, tool, args, cancellation, now,
        |tool_id| {
            self.db.with_connection_refusal(|conn| {
                self.gate
                    .verify_grant(conn, run_id, tool_id, args, now)
                    .map_err(|error| error.refusal())
            })
        },
    )
    .await
    .map_err(|error| error.refusal())
}
```
**Option B (fallback, one known gap):** if WP-02 refuses the callback shape, the engine calls the existing sync `verify_tool_grant` and then `side_effects::execute_approved_prechecked(jail, shell, registry, tier, args, cancellation)` (WP-02 must expose the preflight+effect half); this burns the grant before the effect and any preflight refusal then needs post-grant handling — record it as a consent-burn gap (`C1a-B`) and do not claim preflight-before-grant. Do not hold the DB lock across the shell await (no async DB API exists; `[READ db.rs:389,405]`).

---

## 4. Ordered code hunks

Apply in this order; each hunk is a single-writer file edit. Run `cargo check -p hiverouter-core` after H3, H6 and H12 (Lead runs commands; this planner ran none).

### H1 — `core/src/tool_loop.rs`: wait cap constant

After `MAX_TOOL_RESULT_BYTES` (`[READ tool_loop.rs:18]`) insert:

```rust
/// Maximum committed approval waits per run; the next request is refused with
/// the frozen `E_CALL_LIMIT`, never a new code.
pub const MAX_APPROVAL_WAITS_PER_RUN: u32 = 8;
```

### H2 — `core/src/execution.rs`: imports, `Clock`, waiter types, RAII guard

Imports (extend the blocks at `:16-37`; keep existing imports):

```rust
use hiverouter_tools::approval::{ApprovalDecision, ApprovalResolution};
use hiverouter_tools::shell::ShellPolicy;
use hiverouter_tools::{ExecutedTool as ToolsExecutedTool, ToolId};
use tokio::sync::oneshot;

use crate::run_state::{self, StateEvent};
use crate::tools_bridge::ToolsBridge;
```

After `ToolTurnResult` (`[READ execution.rs:148-159]`) insert the complete waiter module:

```rust
/// Default approval clock: Unix seconds (the approval/TTL unit).
fn default_now_unix_secs() -> u64 {
    Timestamp::now().unix_secs().unwrap_or(0)
}

/// One time source for approval timestamps/CAS (tests inject a fixed clock).
pub type Clock = Arc<dyn Fn() -> u64 + Send + Sync>;

/// What an armed wait can learn. `Resolved` is the gate's committed decision;
/// a cancelled wait never produces a decision.
#[derive(Clone, Debug, PartialEq)]
pub enum ApprovalWake {
    /// The operator decision was committed and handed to the engine.
    Resolved(ApprovalResolution),
    /// The run/task was cancelled while waiting (row consumed as `cancelled`).
    Cancelled,
}

/// In-process waiter registry: one oneshot sender per pending approval id.
/// Armed before `APPROVAL_REQUEST` is published, so a fast resolve is never
/// lost. The mutex is held only for map operations, never across `.await`.
#[derive(Default)]
pub struct ApprovalWaiters {
    inner: std::sync::Mutex<HashMap<String, oneshot::Sender<ApprovalWake>>>,
}

impl std::fmt::Debug for ApprovalWaiters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApprovalWaiters")
            .field("armed", &self.len())
            .finish()
    }
}

impl ApprovalWaiters {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers the receiver for `approval_id`; inserting over a stale key
    /// drops the old sender (its receiver fails closed instead of hanging).
    ///
    /// # Errors
    /// `E_INTERNAL_ERROR` only if the mutex is poisoned.
    pub fn arm(
        &self,
        approval_id: impl Into<String>,
    ) -> Result<oneshot::Receiver<ApprovalWake>, Refusal> {
        let (sender, receiver) = oneshot::channel();
        let mut inner = self.inner.lock().map_err(|_| Refusal::InternalError)?;
        inner.insert(approval_id.into(), sender);
        Ok(receiver)
    }

    /// Delivers a resolved wake exactly once; false when no waiter is armed
    /// (late/duplicate/terminal resolve is legal).
    pub fn wake_resolved(&self, resolution: &ApprovalResolution) -> bool {
        let key = resolution.request.approval_id.to_string();
        let sender = self.inner.lock().ok().and_then(|mut inner| inner.remove(&key));
        sender.is_some_and(|sender| sender.send(ApprovalWake::Resolved(resolution.clone())).is_ok())
    }

    /// Delivers a cancellation wake exactly once.
    pub fn wake_cancelled(&self, approval_id: &str) -> bool {
        let sender = self
            .inner
            .lock()
            .ok()
            .and_then(|mut inner| inner.remove(approval_id));
        sender.is_some_and(|sender| sender.send(ApprovalWake::Cancelled).is_ok())
    }

    /// Removes a waiter without waking it (every exit path).
    pub fn disarm(&self, approval_id: &str) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.remove(approval_id);
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

/// Disarms a waiter on every exit path (wake, timeout, cancel, error, panic).
struct ArmedApproval {
    waiters: Arc<ApprovalWaiters>,
    approval_id: String,
}

impl Drop for ArmedApproval {
    fn drop(&mut self) {
        self.waiters.disarm(&self.approval_id);
    }
}
```

### H3 — `core/src/execution.rs`: `RunEngine` fields, constructors, builders, notify

Fields — insert after `tool_registry` (`[READ execution.rs:286-289]`) and before `active_run`:

```rust
    /// Approval/consent bridge. `None` keeps the pre-1.2 read-only behaviour:
    /// side-effect plans refuse `E_TOOL_NOT_ALLOWED`.
    tools: Option<Arc<ToolsBridge>>,
    /// In-process approval waiters (one server process; no cross-process wait).
    waiters: Arc<ApprovalWaiters>,
    /// One clock for approval timestamps/CAS (tests inject a fixed clock).
    clock: Clock,
    /// Shell allowlist used only for an approved `shell_exec`.
    shell_policy: ShellPolicy,
```

`Debug` (`:293-299`) add:

```rust
            .field("approval_wired", &self.tools.is_some())
            .field("waiters", &self.waiters.len())
```

Both constructors (`new` `:305-327`, `with_services` `:352-377`) initializer add (same values in each):

```rust
            tools: None,
            waiters: Arc::new(ApprovalWaiters::new()),
            clock: Arc::new(default_now_unix_secs),
            shell_policy: ShellPolicy::conservative_default(),
```

Builders after `with_tool_workspace` (`:345-348`):

```rust
    /// Attaches the approval bridge; production `http.rs` always does.
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
    /// Returns false when no waiter is armed (legal: late/duplicate/terminal
    /// resolve); the caller logs and never fails the command.
    #[must_use]
    pub fn notify_approval_resolved(&self, resolution: &ApprovalResolution) -> bool {
        self.waiters.wake_resolved(resolution)
    }
```

`RunEngine::new` and `with_services` signatures stay **unchanged** (D3: engines without `.with_tools` keep the pre-1.2 refusal).

### H4 — `core/src/execution.rs`: `dispatch_inner` run `Start` transition (D2)

Insert after the plan-node creation loop (`[READ execution.rs:622-647]`; immediately before the comment `// Execute leaf tasks…` at `:649`):

```rust
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
```
This is the **only** behavior addition to `dispatch_inner`; plan/cap failures occur before this insert (so `caps_refused_by_name` still sees failure as first `RUN_STATUS`).

### H5 — `core/src/execution.rs`: async `execute_tool_turn` + suspension branch

Signature: `fn execute_tool_turn(` at `[READ :684]` becomes `async fn execute_tool_turn(` (`&self` unchanged); body changes:

1. After `let (completed_turns, used_calls) = …` (`:698-701`) add `let mut precounted: u32 = 0;` next to `let mut executed` (`:736`); change the per-call budget line `let used_before = used_calls.saturating_add(index);` (`:746`) to `let used_before = used_calls.saturating_add(index).saturating_add(precounted);` so later calls in the same turn see the pretaken waits.
2. Replace the `Ok(())` branch body (`:783-789`, today `self.publish_tool_call(...); self.execute_tool_plan(&plan)`) with:

```rust
                            Ok(()) => {
                                self.publish_tool_call(run, task, &plan);
                                if tool_loop::requires_approval(&self.tool_registry, &plan.tool)
                                    && self.tools.is_some()
                                    && self.tool_workspace.is_some()
                                {
                                    // D12: count the attempt before the wait so a
                                    // deny/timeout cannot retry for free.
                                    self.db
                                        .increment_run_tool_counters(run.id, 0, 1)
                                        .map_err(|error| error.refusal())?;
                                    precounted = precounted.saturating_add(1);
                                    // Terminal waits finalize their own TOOL_RESULT
                                    // and return Err (D13); approve/deny return Ok.
                                    self.suspend_for_approval(run, task, call, &plan, turn, token)
                                        .await?
                                } else {
                                    self.execute_tool_plan(&plan)
                                }
                            }
```
3. End-of-turn increment (`:796-800`) becomes:

```rust
        if calls_in_turn > 0 {
            self.db
                .increment_run_tool_counters(
                    run.id,
                    1,
                    calls_in_turn.saturating_sub(precounted),
                )
                .map_err(|error| error.refusal())?;
        }
```
4. Call site in `execute_task` (`[READ :1021]`): `self.execute_tool_turn(run, task, &final_reply, token).await` (adds `.await` only).

All other branches keep the existing `executed.push((plan, execution.clone())); execution` pair at `:788-789`: it runs only for the `Ok(())` result of `validate_plan`; a terminal wait returns `Err` from `suspend_for_approval`, so `?` propagates before the push and the terminal helper's own `TOOL_RESULT` is the only result for that call.

### H6 — `core/src/execution.rs`: suspend / decision / timeout / cancel / resume helpers

Insert the full helper block after `finish_tool_call` (`[READ :934-966]`, i.e. before `execute_task` at `:968`). These are the load-bearing hunks; all code is complete.

```rust
    fn now_unix_secs(&self) -> u64 {
        (self.clock)()
    }

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
        self.publish(
            Event::RunStatus(RunStatusPayload {
                run_id: run.id,
                status: run_transition.to,
                error_code: None,
            }),
            Some(run.id),
            None,
        );
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
        self.publish(
            Event::RunStatus(RunStatusPayload {
                run_id: run.id,
                status: run_transition.to,
                error_code: None,
            }),
            Some(run.id),
            None,
        );
        self.store.update_task_state(task.id, task_transition.to)?;
        let _ = self.graph.set_state(task.id, task_transition.to);
        self.publish_node(run.id, task.id, task_transition.to);
        Ok(())
    }

    /// Commits one approval, arms the waiter, persists `awaiting_approval`,
    /// publishes `APPROVAL_REQUEST`, then waits for resolution / TTL / cancel.
    /// Approve/deny return the execution for the caller's `TOOL_RESULT`; terminal
    /// waits (timeout/cancel/closed/lost-CAS-internal) finalize their own
    /// `TOOL_RESULT` + `tool_turns` row and return `Err` (D13).
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
        let Some(_workspace) = self.tool_workspace.as_ref() else {
            return Ok(ToolExecution::refused(
                plan.tool_call_id.clone(),
                Refusal::FileTreeUnavailable.code(),
                "no workspace jail is configured; the side effect did not execute".to_owned(),
            ));
        };
        let waits = tools
            .approval_count_for_run(run.id)
            .map_err(|error| error.refusal())?;
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
        let approval_key = event.approval_id.to_string();

        // Arm BEFORE publishing: a fast resolve can never be lost.
        let mut receiver = self
            .waiters
            .arm(approval_key.clone())
            .map_err(|_| Refusal::InternalError)?;
        let _armed = ArmedApproval {
            waiters: Arc::clone(&self.waiters),
            approval_id: approval_key.clone(),
        };

        // Persist + publish only after the row is committed and the waiter armed.
        let current_run = self.store.get_run(run.id).map_err(|error| error.refusal())?;
        let current_task = self.store.get_task(task.id).map_err(|error| error.refusal())?;
        let run_transition = run_state::apply_run(current_run.status, StateEvent::Suspend)
            .map_err(|error| error.refusal())?;
        let task_transition = run_state::apply_task(current_task.state, StateEvent::Suspend)
            .map_err(|error| error.refusal())?;
        if let Err(refusal) =
            self.persist_approval_transitions(run, task, run_transition, task_transition)
        {
            let _ = tools.expire_approval(&approval_key, now);
            return Err(refusal);
        }
        self.publish(Event::ApprovalRequest(event.clone()), Some(run.id), None);

        // A request can already be decided by a concurrent driver/retry.
        if let Some(label) = tools
            .approval_decision(&approval_key)
            .map_err(|error| error.refusal())?
        {
            return self
                .apply_committed_decision(
                    run, task, call, plan, turn, token, tools, &approval_key, &label,
                )
                .await;
        }

        let expires = event
            .expires_at
            .as_ref()
            .and_then(Timestamp::unix_secs)
            .unwrap_or_else(|| now.saturating_add(tools.approval_gate().ttl_secs()));
        let remaining = expires.saturating_sub(self.now_unix_secs()).max(1);
        let wait = tokio::time::timeout(std::time::Duration::from_secs(remaining), receiver);
        let wake = tokio::select! {
            biased;
            _ = token.cancelled() => None,
            result = wait => Some(result),
        };

        match wake {
            None => {
                self.finish_cancelled_wait(run, task, call, plan, turn, tools, &approval_key)
                    .await
            }
            Some(Err(_elapsed)) => {
                self.finish_timeout_wait(run, task, call, plan, turn, tools, &approval_key, token)
                    .await
            }
            Some(Ok(Err(_closed))) => {
                // Sender dropped without a wake: fail closed, consume the row.
                let _ = tools.cancel_approval(&approval_key, self.now_unix_secs());
                self.finish_cancelled_wait(run, task, call, plan, turn, tools, &approval_key)
                    .await
            }
            Some(Ok(Ok(ApprovalWake::Cancelled))) => {
                self.finish_cancelled_wait(run, task, call, plan, turn, tools, &approval_key)
                    .await
            }
            Some(Ok(Ok(ApprovalWake::Resolved(resolution)))) => {
                if token.is_cancelled() {
                    // A decision that arrives after cancellation never executes.
                    let _ = tools.cancel_approval(&approval_key, self.now_unix_secs());
                    return self
                        .finish_cancelled_wait(run, task, call, plan, turn, tools, &approval_key)
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

    /// Applies an already-committed decision found right after arming.
    async fn apply_committed_decision(
        &self,
        run: &Run,
        task: &TaskRecord,
        call: &hiverouter_provider::tool_assembler::AssembledToolCall,
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
                self.finish_timeout_wait(run, task, call, plan, turn, tools, approval_id, token)
                    .await
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
        let executed = tools
            .execute_approved(
                &self.shell_policy,
                &self.tool_registry,
                AgentTier::WorkerAgent,
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
        let run_transition = run_state::apply_run(AgentState::AwaitingApproval, StateEvent::Deny)
            .map_err(|error| error.refusal())?;
        let task_transition = run_state::apply_task(AgentState::AwaitingApproval, StateEvent::Deny)
            .map_err(|error| error.refusal())?;
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
        self.store.update_task_state(task.id, task_transition.to)?;
        let _ = self.graph.set_state(task.id, task_transition.to);
        self.publish_node(run.id, task.id, task_transition.to);
        Ok(ToolExecution::denied(plan.tool_call_id.clone()))
    }

    /// Timeout: expire CAS first; if lost, apply the winner. A won CAS finalizes
    /// one timeout TOOL_RESULT + row, then persists the terminal failed run with
    /// `E_APPROVAL_TIMEOUT`; dispatch skips the duplicate frame (D7).
    async fn finish_timeout_wait(
        &self,
        run: &Run,
        task: &TaskRecord,
        call: &hiverouter_provider::tool_assembler::AssembledToolCall,
        plan: &ToolCallPlan,
        turn: u32,
        tools: &ToolsBridge,
        approval_id: &str,
        token: &CancellationToken,
    ) -> Result<ToolExecution, Refusal> {
        let now = self.now_unix_secs();
        if tools
            .expire_approval(approval_id, now)
            .map_err(|error| error.refusal())?
        {
            let execution = ToolExecution::timed_out(plan.tool_call_id.clone());
            self.finish_tool_call(run, task, call, &execution, turn)?;
            let run_transition =
                run_state::apply_run(AgentState::AwaitingApproval, StateEvent::Timeout)
                    .map_err(|error| error.refusal())?;
            let task_transition =
                run_state::apply_task(AgentState::AwaitingApproval, StateEvent::Timeout)
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
                    error_code,
                }),
                Some(run.id),
                None,
            );
            self.store.update_task_state(task.id, task_transition.to)?;
            let _ = self.graph.set_state(task.id, task_transition.to);
            self.publish_node(run.id, task.id, task_transition.to);
            return Err(Refusal::ApprovalTimeout);
        }
        match tools
            .approval_decision(approval_id)
            .map_err(|error| error.refusal())?
        {
            Some(label) => {
                self.apply_committed_decision(
                    run, task, call, plan, turn, token, tools, approval_id, &label,
                )
                .await
            }
            None => Err(Refusal::InternalError),
        }
    }

    /// Cancel: consume the row, mark the task cancelled once (if not already
    /// terminal), finish one cancelled TOOL_RESULT + row, return `E_TOOL_CANCELLED`.
    async fn finish_cancelled_wait(
        &self,
        run: &Run,
        task: &TaskRecord,
        call: &hiverouter_provider::tool_assembler::AssembledToolCall,
        plan: &ToolCallPlan,
        turn: u32,
        tools: &ToolsBridge,
        approval_id: &str,
    ) -> Result<ToolExecution, Refusal> {
        let _ = tools.cancel_approval(approval_id, self.now_unix_secs());
        let fresh = self.store.get_task(task.id).map_err(|error| error.refusal())?;
        if !fresh.state.is_terminal() {
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
```

Note: `finish_cancelled_wait` uses `apply_task(fresh.state, Cancel)`; the authority allows Cancel from Queued/Running/Awaiting (so `AwaitingApproval+Kill→Cancelled` per run_state is subsumed by the cancel edge; if the fresh state is `Running`, Cancel is also legal). `apply_committed_decision` does not fabricate `APPROVAL_RESOLVED` (the WS command owns it).

### H7 — `core/src/execution.rs`: `dispatch` terminal-ownership guard (D7)

Replace the head of `dispatch` (`[READ execution.rs:539-560]`), keeping the snapshot/cleanup block (`:561-582`) but using `final_status`:

```rust
    async fn dispatch(self: Arc<Self>, run: Run, token: CancellationToken) {
        let outcome = self.dispatch_inner(&run, &token).await;
        let persisted = self
            .store
            .get_run(run.id)
            .map(|current| current.status)
            .unwrap_or(run.status);
        let final_status = if persisted.is_terminal() {
            // A wait terminal path (timeout) or CANCEL_RUN already persisted and
            // published the terminal frame; never duplicate it.
            persisted
        } else {
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
            status
        };
        // …snapshot block unchanged except `status` becomes `final_status`…
        let _ = self.set_active(None);
        self.tokens.remove(run.id);
        self.executor.finish_run(run.id).await;
    }
```

### H8 — `core/src/execution.rs`: `execute_task` await + double-failure guard

1. `[READ :1021]` add `.await` to `execute_tool_turn`.
2. Replace the `Err(refusal)` arm (`:1023-1034`) with the terminal-state guard (prevents a duplicate `NODE_STATE failed`/cancelled after a wait helper already persisted the task terminal; covers `ApprovalTimeout` and wait-produced `ToolCancelled` without weakening the ordinary failure/cancel paths):

```rust
                Err(refusal) => {
                    let fresh = self.store.get_task(task.id).map_err(|error| error.refusal())?;
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
```

### H9 — `core/src/execution.rs`: `cancel_run` / `kill_worker`

`cancel_run` (`[READ :499-519]`) — use `run_state` as the authority, keep named refusals:

```rust
    pub fn cancel_run(&self, run_id: RunId) -> Result<(), Refusal> {
        let run = self.store.get_run(run_id)?;
        if run.status.is_terminal() {
            return Err(Refusal::RunTerminal);
        }
        if !self.tokens.cancel(run_id) {
            return Err(Refusal::RunTerminal);
        }
        let transition = run_state::apply_run(run.status, StateEvent::Cancel)
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
    }
```

`kill_worker` (`[READ :525-537]`) — awaiting task wakes the waiter via the run token; the waiter owns the single task/node update:

```rust
    pub fn kill_worker(&self, task_id: TaskId) -> Result<(), Refusal> {
        let task = self.store.get_task(task_id)?;
        if task.state.is_terminal() {
            return Err(Refusal::TaskTerminal);
        }
        if task.state == AgentState::AwaitingApproval {
            // The waiter consumes the approval and applies the task transition;
            // cancelling the run token wakes it. Exit is single-owned.
            self.tokens.cancel(task.run_id);
            return Ok(());
        }
        if task.state != AgentState::Running {
            return Err(Refusal::KillDenied);
        }
        let transition = run_state::apply_task(task.state, StateEvent::Kill)
            .map_err(|error| error.refusal())?;
        self.store.update_task_state(task_id, transition.to)?;
        let _ = self.graph.set_state(task_id, transition.to);
        self.publish_node(task.run_id, task_id, transition.to);
        Ok(())
    }
```

### H10 — `core/src/ws.rs`: wake the engine after a committed decision

In `Command::ApprovalResolve` (`[READ ws.rs:465-483]`), after the two publishes and before the arm's closing `}` (i.e. immediately after line 478 `publish_with_correlation(...)`), insert:

```rust
                    if !state.engine.notify_approval_resolved(&resolution) {
                        tracing::warn!(
                            approval_id = %resolution.request.approval_id,
                            "approval decided with no armed engine wait (late/duplicate/terminal run)"
                        );
                    }
```
No import change; the command still ACKs the committed decision even when no waiter is armed (truthful). Errors/refusals stay before the wake (never wake on a refused decision).

### H11 — `core/src/http.rs`: build tools before engine; inject bridge

1. `build_engine` (`[READ http.rs:155-179]`) gains a `tools: Arc<ToolsBridge>` parameter (after `snapshots`) and adds `.with_tools(tools)` to the constructor chain; body otherwise unchanged.
2. In all three constructors (`new` `:198-200`, `with_graph` `:235-237`, `with_services` `:269-271`) reorder to:

```rust
        let tools = build_tools(&db, &config);
        let engine = build_engine(db.clone(), bus.clone(), graph.clone(), &config, templates, snapshots.clone(), tools.clone(), <wire flag>);
```
and keep `tools` in the `AppState` literal. `wire_workspace_jail` stays `true` for `new`/`with_graph` and `false` for `with_services` (Run-1 test-seam lesson: do not expose the test jail).
3. If the Lead chooses C2 default-off: gate only the `.with_tools(tools)` call in `build_engine` on the config flag; engine code unchanged. Keep the WIP boot reaper inside `build_tools` untouched.
4. `main.rs`, `recovery.rs`, `db.rs`, contract, `web/*`: **no change**.

### H12 — tests

1. New `backend/crates/hiverouter-core/tests/suspension.rs` with the 14 scenarios in §7.1 (engine + real `ToolsBridge`/`ApprovalGate`, tempdir jail, `with_services`-style construction or direct `RunEngine::new(...).with_tools(...).with_tool_workspace(...).with_clock(...)`, `.with_shell_policy(...)`; `ToolsBridge::migrate(&db).expect(...)`; no paid provider).
2. Reconcile `tests/tool_loop_engine.rs` `drive` (`[READ :249-280]`): skip the new `running` frame instead of stopping:
```rust
            "RUN_STATUS" => {
                if event.envelope.payload["status"] == "running" {
                    continue; // D2: the run now starts before leaf execution
                }
                collected.final_status = event.envelope.payload.clone();
                break;
            }
```
3. Reconcile `tests/run_lifecycle.rs`:
   - `run_lifecycle_plans_and_dispatches` `[READ :192-201]`: break only on a terminal `RUN_STATUS`
     (`"RUN_STATUS" if event.envelope.payload["status"] != "running" => break event,` plus the existing `_ => {}` arm; the subsequent `assert_eq!(final_status…["status"], "completed")` stays).
   - `provider_error_preserves_partial_state` `[READ :269-272]`: predicate adds `&& event.envelope.payload["status"] != "running"` (it currently takes the first `RUN_STATUS`).
   - `caps_refused_by_name`/`cancel_run_stops_the_run`/`streamed_deltas…` need no change (cap failures happen before H4's insert; cancel predicate already filters status; stream test breaks on `REPLY_DONE`).
4. `tests/ws_transport.rs::approval_resolve_command_acks_and_emits_resolved_event` `[READ :453]` needs no edit: the extra `notify` call returns false (warn only) and the ACK/`APPROVAL_RESOLVED` order is unchanged. `tests/approval_ledger.rs`, `tests/http_boot.rs` unchanged; `lead_bugcheck_boot.rs` uses `ToolsBridge` directly (no engine) and stays.

---

## 5. Exact signature index

| Symbol | Signature | Hunk |
|---|---|---|
| `tool_loop::MAX_APPROVAL_WAITS_PER_RUN` | `pub const MAX_APPROVAL_WAITS_PER_RUN: u32 = 8;` | H1 |
| `execution::Clock` | `pub type Clock = Arc<dyn Fn() -> u64 + Send + Sync>;` | H2 |
| `execution::ApprovalWake` | `pub enum ApprovalWake { Resolved(ApprovalResolution), Cancelled }` (`Clone, Debug, PartialEq`) | H2 |
| `execution::ApprovalWaiters` | `pub fn new() -> Self`; `pub fn arm(&self, approval_id: impl Into<String>) -> Result<oneshot::Receiver<ApprovalWake>, Refusal>`; `pub fn wake_resolved(&self, &ApprovalResolution) -> bool`; `pub fn wake_cancelled(&self, &str) -> bool`; `pub fn disarm(&self, &str)`; `pub fn len(&self) -> usize`; `pub fn is_empty(&self) -> bool` | H2 |
| `RunEngine` fields | `tools: Option<Arc<ToolsBridge>>`, `waiters: Arc<ApprovalWaiters>`, `clock: Clock`, `shell_policy: ShellPolicy` (private) | H3 |
| `RunEngine` builders | `pub fn with_tools(self, Arc<ToolsBridge>) -> Self`; `pub fn with_clock(self, Clock) -> Self`; `pub fn with_shell_policy(self, ShellPolicy) -> Self`; `pub fn notify_approval_resolved(&self, &ApprovalResolution) -> bool` | H3 |
| `RunEngine::execute_tool_turn` | `async fn execute_tool_turn(&self, run: &Run, task: &TaskRecord, reply: &StreamedReply, token: &CancellationToken) -> Result<ToolTurnResult, Refusal>` (was sync) | H5 |
| `RunEngine::suspend_for_approval` | `async fn suspend_for_approval(&self, &Run, &TaskRecord, &AssembledToolCall, &ToolCallPlan, u32, &CancellationToken) -> Result<ToolExecution, Refusal>` | H6 |
| `RunEngine::apply_committed_decision` | `async fn apply_committed_decision(&self, &Run, &TaskRecord, &AssembledToolCall, &ToolCallPlan, u32, &CancellationToken, &ToolsBridge, &str, &str) -> Result<ToolExecution, Refusal>` | H6 |
| `RunEngine::resume_and_execute` | `async fn resume_and_execute(&self, &Run, &TaskRecord, &ToolCallPlan, &CancellationToken) -> Result<ToolExecution, Refusal>` | H6 |
| `RunEngine::resume_denied` | `fn resume_denied(&self, &Run, &TaskRecord, &ToolCallPlan) -> Result<ToolExecution, Refusal>` | H6 |
| `RunEngine::finish_timeout_wait` | `async fn finish_timeout_wait(&self, &Run, &TaskRecord, &AssembledToolCall, &ToolCallPlan, u32, &ToolsBridge, &str, &CancellationToken) -> Result<ToolExecution, Refusal>` | H6 |
| `RunEngine::finish_cancelled_wait` | `async fn finish_cancelled_wait(&self, &Run, &TaskRecord, &AssembledToolCall, &ToolCallPlan, u32, &ToolsBridge, &str) -> Result<ToolExecution, Refusal>` | H6 |
| `RunEngine::persist_approval_transitions` / `resume_transitions` | `fn(&self, &Run, &TaskRecord, run_state::RunTransition, run_state::TaskTransition) -> Result<(), Refusal>` / `fn(&self, &Run, &TaskRecord) -> Result<(), Refusal>` | H6 |
| `RunEngine::now_unix_secs` | `fn now_unix_secs(&self) -> u64` | H6 |
| `ToolsBridge::execute_approved` (engine-owned hunk, WP-02 delegate) | `pub async fn execute_approved(&self, &ShellPolicy, &ToolRegistry, AgentTier, RunId, &str, &serde_json::Value, Option<&CancellationToken>) -> Result<ExecutedTool, Refusal>` | H6/§3.3 |
| `tools::side_effects::execute_approved` | **external prerequisite**; callback form in §3.3 option A | H0 |
| `run_state` | **no edits**; uses `apply_run`, `apply_task`, `RunTransition`, `TaskTransition`, `StateEvent`, `Effect::refusal()` | H4, H6 |
| Contract/DB/web | **no edits** (no new refusal/event/command, no ledger write, no migration) | — |

---

## 6. State / effect / event order (single-owner table)

| # | Trigger | Run | Task | Events / results |
|---|---|---|---|---|
| S1 | approvable call, bridge+jail | `apply_run(Running,Suspend)` persist + `RUN_STATUS awaiting_approval` | `apply_task(Running,Suspend)` persist + `NODE_STATE awaiting_approval` | `TOOL_CALL` (H5 before request) then one `APPROVAL_REQUEST` (nonce/ids/args_sha256/expiry) |
| S2 | approve wake | `apply_run(Awaiting,Resume)` persist + `RUN_STATUS running` | `apply_task(Awaiting,Resume)` persist + node running | granted executor: preflight → grant CAS → one effect; caller `finish_tool_call` publishes `TOOL_RESULT ok` + row |
| S3 | deny wake | `apply_run(Awaiting,Deny)` + `RUN_STATUS running` | `apply_task(Awaiting,Deny)` + node running | `TOOL_RESULT denied` + row; loop continues |
| S4 | timeout CAS won | `apply_run(Awaiting,Timeout)` → failed + terminal `RUN_STATUS failed error_code=E_APPROVAL_TIMEOUT` | `apply_task(Awaiting,Timeout)` → failed + node failed | one `TOOL_RESULT timeout` + row; `Err(ApprovalTimeout)`; dispatch skips duplicate |
| S5 | timeout CAS lost | none in wait; apply winner (S2/S3/expired/cancelled) | winner | no fabricated `APPROVAL_RESOLVED` |
| S6 | `CANCEL_RUN` while waiting | `cancel_run` persists/publishes cancelled; waiter wakes on token | waiter applies task Cancel once (state may be Awaiting or Running) | one `TOOL_RESULT cancelled` + row; `Err(ToolCancelled)`; dispatch skips duplicate |
| S7 | `KILL_WORKER` on awaiting task | token cancel; waiter S6; dispatch maps `Err(ToolCancelled)`→cancelled terminal | waiter applies task Cancel once | one cancelled result; no double node update |
| S8 | restart while awaiting | recovery: interrupted + `E_APPROVAL_TIMEOUT`; boot reaper expires row unconditionally | task keeps persisted state | no resume, no auto-approve, no `APPROVAL_RESOLVED` |
| S9 | duplicate/late resolve | no waiter; gate refusal per WIP (`E_APPROVAL_REPLAY`/`E_APPROVAL_TIMEOUT`/`E_APPROVAL_NOT_FOUND`) | none | WS ERROR only; `notify` false logged |
| S10 | wrong args at resume | resume Running persisted; grant CAS fails `E_APPROVAL_REQUIRED` | existing failure path (task not terminal → failed) | no effect; one refused result; dispatch failed `E_APPROVAL_REQUIRED` |
| S11 | no bridge/jail | no suspension | none | frozen `E_TOOL_NOT_ALLOWED` result (D3) |

---

## 7. Test contract

### 7.1 New `tests/suspension.rs` (names are the claim IDs; no paid provider)

1. `suspend_publishes_awaiting_approval_and_approval_request`
2. `approve_resumes_and_executes_the_file_write_exactly_once`
3. `deny_returns_denied_result_and_run_continues`
4. `timeout_fails_run_with_e_approval_timeout_and_no_side_effect`
5. `cancel_while_awaiting_cancels_run_and_consumes_approval` (exactly one `TOOL_RESULT cancelled`; row `cancelled`; later resolve `E_APPROVAL_REPLAY`)
6. `kill_worker_while_awaiting_cancels_without_side_effect`
7. `timeout_cas_loss_to_resolve_still_resumes`
8. `concurrent_resolve_has_one_winner_and_no_second_execution`
9. `second_prompt_while_awaiting_is_refused_e_run_active`
10. `restart_marks_awaiting_run_interrupted_and_expires_pending` (C5/C7: status `interrupted` + `error_code=E_APPROVAL_TIMEOUT`; future-TTL row `expired` at boot; later resolve `E_APPROVAL_TIMEOUT`; no auto-approve/execution)
11. `wait_cap_refuses_the_ninth_request` (`E_CALL_LIMIT` result, no new row)
12. `wrong_args_grant_refused_before_side_effect` (C4: `E_APPROVAL_REQUIRED`, no write, one refused result)
13. `shell_exec_runs_only_the_allowlisted_program_after_approval`
14. `no_bridge_engine_keeps_the_read_only_refusal` (engine without `.with_tools` → `E_TOOL_NOT_ALLOWED`)

Engine-unit tests optional in `execution.rs` `#[cfg(test)]`: `approval_waiters_arm_wake_disarm_roundtrip`, `wake_resolved_after_disarm_returns_false`, `waiters_debug_reports_armed_count`. Do not duplicate WP-01/WP-02 unit tests (owned by WAVE-SE) or WP-04 `tool_loop_matrix.rs`.

### 7.2 Existing-test reconciliation (exact, from §4 H12)

`tool_loop_engine::drive` skip-running; `run_lifecycle_plans_and_dispatches` terminal-only break; `provider_error_preserves_partial_state` terminal-only predicate. No other test edits. Counts stay: tool_loop_engine 11, run_lifecycle 7, approval_ledger 2, ws_transport 13.

### 7.3 Commands (Lead runs; this planner ran none)

```powershell
cd backend
$env:PATH = "$env:USERPROFILE\.cargo\bin;$PWD\..\tools\w64devkit\bin;$env:PATH"
cargo check --workspace --tests --locked
cargo test -p hiverouter-core --test suspension --locked
cargo test -p hiverouter-core --test tool_loop_engine --locked
cargo test -p hiverouter-core --test run_lifecycle --locked
cargo test -p hiverouter-core --test approval_ledger --locked
cargo test -p hiverouter-core --test ws_transport --locked
cargo test -p hiverouter-tools --lib --locked
cargo test --workspace --locked
```

---

## 8. Collision / sequencing notes

1. **Live collision at recon:** WAVE-SE's WP-01 edits were uncommitted in `core/src/tools_bridge.rs` (+76) although the handoff (`[LEDGER §2 handoff]`) assigns that file to WAVE-ENGINE. Do **not** start H1 on a dirty tree; wait for the Lead to commit WP-01/WP-02, then record `git rev-parse HEAD`/`git status --porcelain`. Re-grep every Prerequisite symbol (§3.1) and confirm WP-02 exists (§3.2).
2. **Line anchors drift** after the WP-01/WP-02 merges (`[LEDGER C11/C12]`): all line numbers here are HEAD `90cb7a7`; re-grep by symbol before each edit, never by line number.
3. **Same-file writers:** engine wave is single writer of `execution.rs`, `tool_loop.rs`, `tools_bridge.rs`, `ws.rs`, `http.rs`, `tests/suspension.rs`, `tests/tool_loop_engine.rs`, `tests/run_lifecycle.rs`. WP-05 replay later edits `ws.rs::send_hello_and_replay` + call sites (disjoint from the `ApprovalResolve` arm at `:465-483`) and appends to `tools_bridge.rs`/`approval.rs`; sequence engine → replay, never parallel. WP-06 web is disjoint.
4. **`tools_bridge.rs` hunk placement:** `execute_approved` goes **after** the WP-01 `verify_tool_grant` block; do not rewrite or reorder WP-01 hunks (their diff is the merge base).
5. **`http.rs` is touched only by this wave**; `main.rs`/`recovery.rs` untouched; Run-1 `lead_bugcheck_boot.rs` uses `ToolsBridge` directly and is unaffected.
6. **`C1a` interface blocker:** if WP-02 keeps the `GrantedExecution<'_>`/`&Connection` shape from `CODE-SIDE-EFFECTS.md`, H6 cannot be compiled as written. The Lead/WP-02 owner must publish the callback form (§3.3 A) or the fallback (§3.3 B) before H6; if neither, the wave is **BLOCKED**, not partially green.
7. **`C2` Lead decision** affects only H11's `.with_tools` wiring; do not add an engine-local feature flag (keeps engine semantics identical under both choices).
8. **Timer flakiness (U7):** tests #4/#7 with a short TTL; prefer `ApprovalGate::with_ttl(1)` + real `tokio::time` like the blueprint; if flaky, inject the clock for timestamps but keep the real timer (no `tokio::test-util` dependency added by this wave).

---

## 9. Must NOT be claimed (implementing-wave report guardrails)

1. No build/test/compile/runtime claim from this planner; only a Lead-run quoted command may claim green.
2. No crash-resume of a suspended turn; no cross-process exactly-once. Exactly-once is per committed grant within one process; a crash between grant verification and the effect consumes the grant without an effect (honest audit gap).
3. No auto-approve/auto-resume after restart; boot reaper expires pendings unconditionally (Run-1 behavior).
4. No new refusal code, event, command, DB migration, contract change, or UI work. `APPROVAL_RESOLVED` is published only by the operator approve/deny command, never by timeout/cancel.
5. The engine never appends ledger rows (ledger remains WS/bridge-owned) and never writes SQLite directly (all through `ToolsBridge`).
6. No `run_state` claim beyond the approval edges + the single run `Start`; task start/complete/fail stay direct writes.
7. No claim that WP-01/WP-02 code was authored by this wave; they are prerequisite commits. No claim that the `C1a` interface is resolved until the Lead records the decision.
8. No shell-sandbox/arbitrary-execution safety claim; allowlist + jail + caps only, and `ShellPolicy::conservative_default()` is a default, not a boundary.
9. No `expected_sha256` dedicated-refusal claim (`E_SCHEMA_INVALID` mapping is a known semantic gap); no args-preview work (1.3/contract-gated).
10. No multi-run/multi-client waiter isolation claim; single-active-run policy unchanged. `notify_approval_resolved` returning false is legal (late/duplicate/terminal) and must be reported as such.
11. No claim that the frame-count test edits are behavior changes: they only stop counting `RUN_STATUS running` as terminal; product behavior is fixed by H4.
12. No claim that existing test suites pass until the Lead runs the commands in §7.3 and quotes output.

---

## 10. Rollback

- Absence of `.with_tools` keeps the pre-1.2 refusal; reverting H11's bridge injection restores the read-only engine without touching the other hunks.
- `MAX_APPROVAL_WAITS_PER_RUN` is additive; reverting H1 only removes the cap (not recommended).
- `ApprovalWaiters`/`ApprovalWake` are additive types; removing them leaves no persisted state.
- No DB schema change in this wave; WP-01's index is additive/`IF NOT EXISTS`.
- `run_lifecycle`/`tool_loop_engine` edits are test-only and independently revertible; product behavior is defined by H4/H7.

---

## 11. Open questions / UNVERIFIED register

| ID | Item | Settle by |
|---|---|---|
| U1 | Exact WP-01 commit content/signatures (observed uncommitted; may change before commit) | re-diff after Lead commits WP-01; re-grep §3.1 symbols |
| U2 | WP-02 `execute_approved` final interface (C1a) | Lead/WP-02 decision recorded before H6; both options in §3.3 |
| U3 | Existing tests tolerate the new `RUN_STATUS running` frame (only the 3 sites in H12 read it as terminal — verified by read; run to confirm) | `cargo test` in §7.3 |
| U4 | Tokio timer precision for 1 s TTL tests #4/#7 | Lead run; inject clock or lengthen TTL if flaky |
| U5 | Production shell allowlist policy / C2 rollout flag | Lead decision; only H11 conditional changes |
| U6 | Duplicate pending rows on a live DB blocking WP-01's unique index (from `[LEDGER U2]`) | WP-01 owner query before index; not an engine hunk |
| U7 | Whether a waiter can be armed while a stale row from a previous process exists (boot reaper should prevent) | test #10 + boot order `[READ main.rs:81-123]` |
| U8 | Actual runtime behavior of every hunk | **UNVERIFIED** — no build/test run by this planner |

---

## 12. Read references

`plans/foresight/{COMPILED-LEDGER.md (§1 WP-03, §2 handoff, §4 do-not-claim, §5 C1-C13/U1-U14), CODE-APPROVAL-ENGINE.md (§1-§14), CODE-SIDE-EFFECTS.md (§3.1 execute_approved, §7 engine seam), RUN-1.md, APPROVAL-SUSPENSION-PLAN.md}`; `backend/crates/hiverouter-core/src/{execution.rs (1-160,250-1130), run_state.rs (205-620), tools_bridge.rs (1-401), ws.rs (465-483), http.rs (34-292), db.rs (389,405), main.rs (81-160), tool_loop.rs}`; `backend/crates/hiverouter-tools/src/{approval.rs (83-130,415-760,843), error.rs, executor.rs, lib.rs}`; `backend/crates/hiverouter-contract/src/{lib.rs,ids.rs,event.rs,state.rs}`; `backend/crates/hiverouter-core/tests/{tool_loop_engine.rs,run_lifecycle.rs,ws_transport.rs,http_boot.rs,lead_bugcheck_boot.rs}`; `backend/Cargo.toml`; `git show --stat HEAD`, `git status`, `git diff` (WP-01 in-flight).
