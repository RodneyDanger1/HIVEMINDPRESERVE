# FORESIGHT PLAN 2 — Side-effect execution + approval grant enforcement

**Items:** 3 (side-effect tool execution: `file_write`, `shell_exec`) and 16 (approval **grant enforcement** in the
run engine) — the two residual gaps of the 1.2 tool loop after the read-only wave.
**Seat:** Foresight Planner 2 · **READ-ONLY** (this document is the only write) · Repo `C:\Users\Riley\Desktop\hiverouter-v1`
· HEAD observed `37f0e7c` · 2026-11-06.
**Skills applied:** `/forgewright-recon`, `/forgewright-plan`, `/forgewright-lead`, `/logic-gatekeeper`
(Phase-1 contract at §16), `/integration-logic`, `/logic-debugger`, `/truthseeker`, `/capability-auditor`.
**Status:** decision-complete implementation plan; **no code written**, no build/test, no key access.

## 0. Decision (short)

Today's loop executes **read-only tools only**. The primitives for the side-effect lane are already present and tested
(`WorkspaceJail::write`, `shell::execute` with allowlist/timeout/Job-Object/env sanitizer, `ApprovalGate` with
single-use nonce/args binding/TTL, `ToolsBridge` request/resolve/ledger, and an **unwired** `run_state` approval
transition table). The gap is not capability but **wiring and enforcement**: the engine refuses every Mutate/Execute
tool (`execution.rs:874-892`), never suspends a run for consent, never verifies a grant, and never calls the
side-effect executors.

**Recommendation: implement both items as one wave behind a default-off feature flag**, adding
(a) a tools-layer *granted* executor (`execute_approved`) that verifies the approval grant **before** any side
effect, and (b) an engine suspension/resume seam that commits the approval row, publishes `TOOL_CALL` +
`APPROVAL_REQUEST`, persists `awaiting_approval` through the existing `run_state` table, waits for the decision,
then re-verifies and executes once. Keep read-only behavior unchanged; fail closed on every ambiguity.

## 1. Evidence table (current tree, verified reads)

| # | Path:line | Fact |
|---|---|---|
| E1 | `tools/src/executor.rs:7-15,52-80` | Read-only executor deliberately refuses every tool except `file_read`/`file_list` (`E_TOOL_NOT_ALLOWED`); no write/shell path. |
| E2 | `tools/src/executor.rs:209-252` | Policy refusal is a **result**, not an error; unknown/read-only-refused tools never touch disk. |
| E3 | `tools/src/jail.rs:216-243` | `WorkspaceJail::write` exists (parents created after containment check) but its doc says "caller is responsible for the approval gate". |
| E4 | `tools/src/jail.rs:302-437` | Escape-matrix tests (parent/absolute/UNC/drive/short-name/symlink) and `MAX_READ_BYTES = 1 MiB` are proven. |
| E5 | `tools/src/shell.rs:164-283` | `shell::execute`: bare-name allowlist, no shell interpolation, jailed cwd, env `sanitize_tokio`, 60 s timeout, 1 MiB/256 KiB caps, kill-on-close Job Object, cancellation → `E_TOOL_CANCELLED`. |
| E6 | `tools/src/shell.rs:61-72` + `src/executor.rs:201-208` | `ShellPolicy::conservative_default()` = `cmd.exe`, `git.exe`, `cargo.exe`; **no core config wires a policy yet**. |
| E7 | `tools/src/approval.rs:85-134` | `ApprovalRequest` carries run/task/tool_call id, tool, permission class, `args_sha256`, TTL, single-use `nonce`. |
| E8 | `approval.rs:141-202` | SQLite `approvals` table with `UNIQUE(nonce)`, `decision`, `decided_at`, `executed_at`; insert = replay refusal on collision. |
| E9 | `approval.rs:440-555` | `resolve_with_reason`/`expire_pending`: one decision per request, TTL expiry is terminal, replay uses existing codes. |
| E10 | `approval.rs:643-711` | `verify_grant`: re-hashes args, AllowSession scoped to (run, tool, args); AllowOnce consumed exactly once via `UPDATE … executed_at IS NULL`; missing/expired/denied → `E_APPROVAL_REQUIRED`, already executed → `E_APPROVAL_REPLAY`. |
| E11 | `approval.rs:714-812` | `request_tool_approval` validates id/tool/permission class and commits the row **before any side effect**. |
| E12 | `approval.rs:814-1214` (tests) | Once-only roundtrip, args binding, replay, TTL zero-side-effects, deny, session scope, wire nonce, no-arg-body ledger, read-only cannot request. |
| E13 | `core/src/tools_bridge.rs:217-239` | `resolve_approval_ledgered`: gate decides first, ledger append after; a decision without evidence returns `E_INTERNAL_ERROR` (fail closed). |
| E14 | `tools_bridge.rs:260-348` | `request_model_tool_approval` commits/returns the **frozen `APPROVAL_REQUEST` payload** with nonce/run/task/tool_call/args_sha256/expires_at; pending-event replay read-only. |
| E15 | `tools_bridge.rs:90-131` + `core/http.rs:116-127` | Boot reaper expires stale pending approvals; tools migration runs at boot. |
| E16 | `core/src/run_state.rs:205-267,465-660` | Approval transition table + effects exist: `RequestApproval`, `ResumeFromApproval`, `DenyApproval`, `FailOnApprovalTimeout` (+ task variants); timeout carries `E_APPROVAL_TIMEOUT`; illegal/replayed edges fail closed. |
| E17 | `core/src/recovery.rs:50-64` | Boot marks active runs interrupted; a run left in `awaiting_approval` gets `E_APPROVAL_TIMEOUT` and never auto-executes. |
| E18 | `core/src/execution.rs:680-806,874-920` | Tool turn loop plans/validates/publishes/tool-results, but `execute_tool_plan` refuses every non-Read tool and only calls `execute_read_only`. |
| E19 | `execution.rs:260-292,345-346` | `RunEngine` holds `tool_workspace: Option<WorkspaceJail>`, `tool_registry: ToolRegistry`; **no shell policy, no approval waiter**. |
| E20 | `core/src/ws.rs` `ApprovalResolve` arm | Resolves the nonce, publishes ACK + `APPROVAL_RESOLVED`; **does not notify the engine** (no waiter exists). |
| E21 | `contract/src/event.rs` `ApprovalRequest` | Payload carries `approval_id, tool, decision?, nonce?, run_id?, task_id?, tool_call_id?, args_sha256?, expires_at?`; `ToolOutcome` = `ok/refused/denied/timeout/cancelled`. |
| E22 | refusal table (frozen 65) | Existing codes suffice: `E_APPROVAL_*`, `E_TOOL_*`, `E_JAIL_ESCAPE`, `E_PATH_*`, `E_FILE_TOO_LARGE`, `E_SCHEMA_INVALID`, `E_INTERNAL_ERROR`, `E_DB_BUSY` — **no new code needed**. |
| E23 | `plans/waves/W12-INTERFACES.md:94-115` | Interfaces: read-only executor; `request_tool_approval`; ledger append; approval request producer must populate all six optional fields. |
| E24 | `core/src/tool_loop.rs` (pure module) | `MAX_TOOL_TURNS = 4`, `MAX_CALLS_PER_RUN = 16`, `MAX_TOOL_RESULT_BYTES = 65_536`; planning/validation only — no execution. |

## 2. Current behavior (what happens today to a model Mutate/Execute call)

1. Model streams a tool call → assembler → `StreamedReply.tool_calls` → `execute_tool_turn` plans and validates it
   against the frozen registry/tier allowlist (`WorkerAgent` in `execution.rs:770-783`).
2. `execute_tool_plan` sees `permission_class != Read` and returns `ToolExecution::refused(E_TOOL_NOT_ALLOWED)`
   (`execution.rs:879-892`) — **no approval row, no suspension, no side effect**.
3. `TOOL_RESULT` is published, the tool turn is persisted, the model continues. So `file_write`/`shell_exec` are
   honestly refused today; the approval subsystem is reachable only by user-driven surfaces, not by the loop.
4. `run_state` approval transitions exist but are **not called by the engine** (grep: only `lib.rs` re-export and
   `recovery.rs` reference), and `ws.rs` cannot wake the loop because no waiter exists.

## 3. Exact functions to add (decisions, not code)

### 3.1 Tools layer (`backend/crates/hiverouter-tools/src/`)

New module `side_effects.rs` (registered in `lib.rs`; exports below), keeping `executor.rs` read-only and truthful:

```rust
/// Why a granted execution is refused before any side effect.
pub enum GrantedRefusal { ApprovalRequired, ApprovalTimeout, ApprovalReplay, ToolNotAllowed, Schema, Bounds, Io }
impl GrantedRefusal { pub fn refusal(self) -> hiverouter_contract::Refusal; }

/// Everything the granted executor needs; assembled by the engine, borrowed only.
pub struct GrantedExecution<'a> {
    pub jail: &'a WorkspaceJail,
    pub registry: &'a ToolRegistry,
    pub shell: &'a ShellPolicy,
    pub tier: AgentTier,
    pub conn: &'a rusqlite::Connection,
    pub gate: &'a ApprovalGate,
    pub run_id: RunId,
    pub now_unix_secs: u64,
}

/// One statically bounded write snapshot; content is UTF-8 by schema.
pub struct WriteSpec { pub path: String, pub content: String, pub expected_sha256: Option<String> }

/// The one granted entry point: grant is verified, then exactly one tool runs.
pub async fn execute_approved(
    ctx: GrantedExecution<'_>,
    tool: &str,
    args: &serde_json::Value,
    cancellation: Option<&CancellationToken>,
) -> Result<ExecutedTool, ToolError>;

/// file_write: jail resolve → stale guard → bounds → atomic replace → bounded summary.
pub fn execute_file_write(jail: &WorkspaceJail, args: &serde_json::Value) -> Result<ExecutedTool, ToolError>;

/// shell_exec: registry/schema → ShellPolicy → shell::execute → bounded text result.
pub async fn execute_shell_exec(
    jail: &WorkspaceJail,
    policy: &ShellPolicy,
    args: &serde_json::Value,
    cancellation: Option<&CancellationToken>,
) -> Result<ExecutedTool, ToolError>;
```

Rules for `execute_approved` (order is load-bearing):
1. Registry-first: `registry.resolve_call(tier, tool, args)?` — unknown tool/tier/tier-not-allowed/args schema refuse
   **before** any grant/database/disk/process work (`E_TOOL_NOT_FOUND`/`E_TOOL_NOT_ALLOWED`/`E_TOOL_SCHEMA`).
2. Permission class must be `Mutate`/`Execute` (Read tools go through `execute_read_only`; a read tool here is
   `E_TOOL_NOT_ALLOWED`).
3. **Grant verification:** `ctx.gate.verify_grant(conn, run_id, ToolId, args, now)?` — missing/denied/expired/wrong
   args ⇒ `E_APPROVAL_REQUIRED`; already-consumed once-only ⇒ `E_APPROVAL_REPLAY`; the accepted error identity must be
   propagated unchanged so the loop can map it to the tool result.
4. Execute exactly one tool: `FileWrite` → `execute_file_write`; `ShellExec` → `execute_shell_exec`;
   anything else ⇒ `E_TOOL_NOT_ALLOWED`.
5. Never log/emit argument bodies; on success return a bounded `ExecutedTool::Ok { text }` summary (path + bytes +
   sha256 for writes; program/exit/caps for shell), never the raw file content.

New small `WorkspaceJail` helper (jail.rs): `pub fn write_atomic(&self, requested: &str, bytes: &[u8]) ->
Result<PathBuf, ToolError>` — resolve + containment, refuse directory/NUL/ADS/short-name targets, write a temp file
in the **same** directory (`.<name>.hive-<uuid>.tmp`), flush/sync, then `fs::rename` over the target; on any error
remove the temp; no partial-file success. Existing `write()` stays for tests/internal calls.

### 3.2 Core engine (`backend/crates/hiverouter-core/src/`)

New module `approval_wait.rs` (or extend `run_state.rs` + `execution.rs`; names below matter for review):

```rust
/// One wait seat per (run, tool_call) so a decision wakes exactly one execution.
pub struct ApprovalBroker { /* Mutex<HashMap<(RunId,String), Vec<oneshot::Sender<()>>>> */ }
impl ApprovalBroker {
    pub fn register(&self, run_id: RunId, tool_call_id: &str) -> oneshot::Receiver<()>;
    pub fn notify(&self, run_id: RunId, tool_call_id: &str) -> bool;  // true = a waiter existed
    pub fn drop_run(&self, run_id: RunId);                            // cancel/timeout cleanup
}

pub enum ApprovalWaitOutcome { Approved, Denied, TimedOut, Cancelled }
```

`RunEngine` additions:
- fields: `approval_broker: Arc<ApprovalBroker>`, `shell_policy: ShellPolicy`, `side_effects_enabled: bool`.
- builders: `pub fn with_approval_broker(...)`, `pub fn with_shell_policy(ShellPolicy)`, `pub fn with_side_effects_enabled(bool)`.
- `async fn execute_tool_plan(&self, run, task, plan, token) -> ToolExecution` (currently sync and read-only-only):
  branch `Read` → existing `execute_read_only`; `Mutate|Execute` → `execute_side_effect_plan`; else refused.
- `async fn execute_side_effect_plan(&self, run, task, plan, token) -> ToolExecution`:
  1. flag-off ⇒ `ToolExecution::refused(E_TOOL_NOT_ALLOWED, "side-effect lane disabled")` (default).
  2. `publish_tool_call` (existing) **before** the approval request.
  3. `run_state::apply_run(run.status, StateEvent::Suspend)` → commit approval row via
     `ToolsBridge::request_model_tool_approval` → publish the returned `APPROVAL_REQUEST` (payload already carries the
     six fields) → persist `awaiting_approval` (run + task) — one effect chain, exactly once.
  4. `register` waiter; wait `tokio::select!` on decision, `expires_at` deadline, `token.cancelled()`.
  5. outcomes: Approved ⇒ `run_state` Resume → `tools::side_effects::execute_approved(...)` with the DB connection from
     `self.db.with_connection` → `ToolExecution::ok`/refused; Denied ⇒ `ToolExecution::denied`; TimedOut ⇒
     `ToolExecution::timed_out` + `StateEvent::Timeout` (persist `E_APPROVAL_TIMEOUT`); Cancelled ⇒
     `ToolExecution::cancelled` + token path. **No second execution, no retry after a consumed grant.**
  6. `finish_tool_call` (existing) publishes `TOOL_RESULT` and persists `tool_turns` (with `args_sha256`, outcome,
     code, bytes/sha256 — never bodies).
- `pub fn notify_approval_resolved(&self, resolution: &ApprovalResolution)` — called by `ws.rs` after
  `resolve_approval_ledgered`; publishes `APPROVAL_RESOLVED` (existing) then `broker.notify(run_id, tool_call_id)`.
- `RunEngine::cancel_run`/`kill` paths call `broker.drop_run(run_id)` after cancelling the token.

`ws.rs` `ApprovalResolve` arm (existing lines) gains: after the ledgered resolution succeeds,
`state.engine.notify_approval_resolved(&resolution)` **before**/with the ACK+`APPROVAL_RESOLVED` publish (order:
decision committed → waiter woken; event publication is the UI echo).

### 3.3 Config + boot (`core/src/config.rs`, `http.rs`)

- New `ShellPolicy`-bearing config (single owner: engine): `HIVEROUTER_SHELL_ALLOWLIST` (comma-separated bare names;
  default `conservative_default()`), `HIVEROUTER_SHELL_TIMEOUT_SECS` (default 60, hard cap 300),
  and `HIVEROUTER_SIDE_EFFECTS=0|1` (default **0**). Reject separator/empty entries at boot (`E_TOOL_NOT_ALLOWED`
  class config error, boot fails loud).
- `http.rs`: `RunEngine::new(...).with_shell_policy(policy).with_side_effects_enabled(flag)`; `ToolsBridge` continues to
  own the gate + approval table; boot reaper unchanged. No new DB migration: `approvals` and `tool_turns` already exist.

## 4. `file_write` requirements (all mandatory)

| # | Requirement | Enforcement point / refusal |
|---|---|---|
| F1 | Tier allowlist + schema validated first (`FileWriteArgs { path, content, expected_sha256? }`, `deny_unknown_fields`) | `registry.resolve_call` → `E_TOOL_NOT_ALLOWED` / `E_TOOL_SCHEMA` |
| F2 | Approval grant bound to exact args (path + full content + expected hash) | `verify_grant` → `E_APPROVAL_REQUIRED`; grant before any resolve/write |
| F3 | Path inside jail: same six escape vectors, ADS `:`, `~`, absolute/UNC/device, symlink/junction containment | `WorkspaceJail::resolve`/`write_atomic` → `E_PATH_OUTSIDE_JAIL` |
| F4 | Target must be a regular file or not exist; directory target refused; root write refused | `E_TOOL_SCHEMA` / `E_PATH_OUTSIDE_JAIL` (root) — no clamping |
| F5 | Content bound: ≤ 1 MiB UTF-8 (`String` already UTF-8); larger ⇒ named refusal before grant consumption/dispatch | `E_FILE_TOO_LARGE` (reuse frozen code; detail says write cap) |
| F6 | Stale-write guard: if `expected_sha256` present, require 64 hex chars, compute current file sha256; missing file or mismatch ⇒ refuse with **no write and no grant consumption**; mismatch code `E_TOOL_SCHEMA` (caller fixable, no new code) | `execute_file_write` before grant? grant is verified in `execute_approved` first — **order decision:** validate shape/path/bounds/stale **before** consuming the once-only grant (read-only checks), then `verify_grant`, then atomic write; verification must be last pre-effect step. |
| F7 | Atomic replace: temp in same directory → rename; cleanup temp on error; never partial success | `write_atomic`; failures `E_INTERNAL_ERROR` (no success claim) |
| F8 | Result summary bounded (≤ `MAX_TOOL_RESULT_BYTES`), contains relative path, bytes, sha256; **content never echoed** and never persisted/logged | `execute_approved` + core `bounded_tool_text` + `tool_turns` stores hash/bytes only |
| F9 | No binary write (schema is text), no executable-bit/symlink creation, no directory creation beyond parents inside jail | `jail.write_atomic` + schema |
| F10 | Cancel before effect = no write; cancel during rename window is not claimable (at-most-once, see §5) | token checks before verify/write |

## 5. `shell_exec` requirements (all mandatory)

| # | Requirement | Enforcement point / refusal |
|---|---|---|
| S1 | Tier allowlist + schema validated first (`ShellExecArgs { program, args, cwd? }`, `deny_unknown_fields`) | `resolve_call` → `E_TOOL_NOT_ALLOWED` / `E_TOOL_SCHEMA` |
| S2 | Approval grant bound to exact args (program string + full argv + cwd) | `verify_grant` → `E_APPROVAL_REQUIRED`; consume once |
| S3 | Bare allowlisted program only (no separators/absolute path; no shell interpolation); policy from config | `ShellPolicy::allows` → `E_TOOL_NOT_ALLOWED` |
| S4 | cwd through the jail (`None` = root); missing/non-dir ⇒ refuse | `jail.resolve` → `E_PATH_OUTSIDE_JAIL` / `E_TOOL_SCHEMA` |
| S5 | Child env is the explicit allowlist (`env_safety::sanitize_tokio`), stdin null, stdout/stderr capped 1 MiB/256 KiB, kill-on-close Job Object; timeout default 60 s, hard cap 300 s | `shell::execute` → `E_TOOL_TIMEOUT` / `E_TOOL_CANCELLED` / `E_INTERNAL_ERROR` |
| S6 | Run cancellation/cancel token terminates child, returns `E_TOOL_CANCELLED`; no further spawn | token select in `shell::execute` |
| S7 | Result: bounded text summary (program, exit code, success, duration, stdout/stderr **truncated to the core result bound**, truncation flag); non-zero exit is a **result** (`ToolOutcome::Ok`) not a fake success and not a refusal | `execute_shell_exec` + `bounded_tool_text` |
| S8 | Never log full argv/child output beyond bounded result; no key/env exposure (env allowlist excludes `DEEPSEEK_API_KEY` — existing test) | shell + ledger rules |
| S9 | Side-effect lane default **off**; enabling is explicit operator config. `cmd.exe` in the default allowlist is a **documented risk**: `cmd /C` is a general shell; see §12. | config flag; approval UI review |

## 6. Grant verification path (item 16, end-to-end)

```
model tool_call (Mutate/Execute)
  → engine plan/validate (tier+schema, MAX_TURNS/MAX_CALLS)
  → if side_effects disabled: TOOL_RESULT refused E_TOOL_NOT_ALLOWED (today's behavior)
  → publish TOOL_CALL
  → run_state::apply_run(Running, Suspend) = Effect::RequestApproval
      → ToolsBridge::request_model_tool_approval(run, task, call_id, tool, args)
          → approval::request_tool_approval → ApprovalGate::request → INSERT row (nonce, args_sha256, expires_at)
      → persist run/task awaiting_approval
      → publish APPROVAL_REQUEST (six fields populated)
  → wait: decision | expires_at deadline | cancel token
      APPROVE: run_state Resume → tools::side_effects::execute_approved
          → registry resolve_call (fresh) → read-only pre-checks (path/schema/bounds/stale for write)
          → ApprovalGate::verify_grant  (args re-hash; AllowOnce CAS executed_at → exactly one execution)
          → execute exactly one tool (file_write atomic | shell_exec policy)
          → bounded ExecutedTool::Ok/Refused
      DENY: ToolExecution::denied ("operator denied; nothing executed")
      TIMEOUT: run_state Timeout → ToolExecution::timed_out (E_APPROVAL_TIMEOUT, no side effect)
      CANCEL/KILL: ToolExecution::cancelled; broker.drop_run; pending row stays expired/undecided (never resurrected)
  → publish TOOL_RESULT + persist tool_turns row
```

Rules:
- The **decision commit and ledger append happen in `ws.rs` via `resolve_approval_ledgered` before the waiter is
  notified**; the engine never resolves its own request.
- The engine alerts only after the waiter wakes; if the notify finds no waiter (duplicate resolution/crash), the
  decision stays committed and the run fails closed at recovery — no execution without a live waiter.
- `verify_grant` is called exactly once per execution, **after** all read-only pre-checks, so a doomed call (bad path,
  stale hash, oversize content) never consumes the single-use grant. If execution itself fails after the grant is
  consumed (I/O error/crash), the grant is spent and a retry returns `E_APPROVAL_REPLAY` — at-most-once.
- Approval row + `awaiting_approval` persistence happen **before** `APPROVAL_REQUEST` is published; the UI can always
  replay pending events (`pending_approval_events`).
- Boot: `ToolsBridge::expire_pending_approvals` + `recovery::recover_on_boot` mark `awaiting_approval` runs
  interrupted with `E_APPROVAL_TIMEOUT`; no auto-resume, no auto-execute.

**Once-only semantics:** nonce `UNIQUE`, decision `UPDATE … WHERE decision IS NULL` (single winner), AllowOnce
`executed_at` CAS (single consumer), replay refused with `E_APPROVAL_REPLAY`; AllowSession remains scoped to
(run, tool, args) and is not exposed by the wire (`approve|deny` only) — the 1.2 model loop consumes AllowOnce grants.

## 7. Bounds and timeouts (single table)

| Bound | Value | Owner | On violation |
|---|---|---|---|
| Approval TTL | 120 s (`APPROVAL_TTL_SECS`) | tools approval | `E_APPROVAL_TIMEOUT`; row marked expired; no side effect |
| Approval wait deadline | = `expires_at` from the committed row | core engine | Timeout outcome, run/task failed by name |
| Shell wall clock | 60 s default, 300 s hard cap | `ShellPolicy` (config) | `E_TOOL_TIMEOUT`, child+grandchildren killed (Job Object) |
| Shell stdout / stderr caps | 1 MiB / 256 KiB | `ShellPolicy` | truncated flag in result; no unbounded memory |
| File write content | ≤ 1 MiB UTF-8 | tools side-effects | `E_FILE_TOO_LARGE` before grant consumption |
| File read (existing) | 1 MiB | jail | `E_FILE_TOO_LARGE` |
| Tool result text | `MAX_TOOL_RESULT_BYTES` = 65,536 | core tool loop | truncated summary marker, hash retained |
| Tool turns / calls/run | 4 turns / 16 calls | core tool loop | `E_BUDGET_EXCEEDED` (turn) / `E_BUDGET_EXCEEDED` (call) refusals, nothing executes |
| Approval rows pending/run | one live request per (run, tool_call_id) — idempotent re-request | ToolsBridge | same request returned; different args ⇒ `E_APPROVAL_REPLAY` |
| Concurrency | tool calls executed sequentially per turn; concurrent runs may touch the same file — last writer wins under `expected_sha256` guard | engine + file_write | stale guard refuses, never silently merges |
| Cancel | checked before request, before verify, and inside `shell::execute` | engine/shell | `E_TOOL_CANCELLED`, no side effect claimed |

## 8. Denial / refusal codes (existing frozen 65 only; no new codes)

| Situation | Code | Outcome/wire |
|---|---|---|
| No approval grant / denied / expired / args mismatch at execution | `E_APPROVAL_REQUIRED` | `ToolExecution::refused` (or RunStateError refusal) |
| Request TTL elapsed before decision / wait deadline | `E_APPROVAL_TIMEOUT` | `ToolOutcome::Timeout` + run/task failed by name |
| Once-only grant already consumed / duplicate decision | `E_APPROVAL_REPLAY` | `ToolExecution::refused`; run continues or fails per run_state |
| Approval decision invalid (`approve|deny` only) | `E_APPROVAL_DECISION_INVALID` | existing ws error path |
| Unknown approval id / nonce mismatch | `E_APPROVAL_NOT_FOUND` / `E_APPROVAL_REPLAY` | existing resolve path |
| Tool unknown / tier not allowed / bad args | `E_TOOL_NOT_FOUND` / `E_TOOL_NOT_ALLOWED` / `E_TOOL_SCHEMA` | refused result |
| Program/chdir outside policy or jail | `E_TOOL_NOT_ALLOWED` / `E_PATH_OUTSIDE_JAIL` | refused result |
| Shell timeout / cancelled | `E_TOOL_TIMEOUT` / `E_TOOL_CANCELLED` | Timeout/Cancelled outcome |
| Write too large / stale hash / directory target | `E_FILE_TOO_LARGE` / `E_TOOL_SCHEMA` | refused, no side effect |
| DB/ledger failure during approval or result persistence | `E_DB_BUSY` / `E_INTERNAL_ERROR` | fail closed (no success claim; consumed nonce stays spent) |
| Side-effect lane disabled (default) | `E_TOOL_NOT_ALLOWED` | refused result (today's behavior preserved) |
| Operator deny | — (no code; `ToolOutcome::Denied` + text) | `TOOL_RESULT` denied, nothing executed |

## 9. Security / secret rules

1. **Grant is the only execution authority** for Mutate/Execute; `execute_approved` is not reachable without a
   `verify_grant` success; no "trusted caller" bypass, no env-flag default-on.
2. **Args are hashed, never stored/logged**: ledger and `tool_turns` keep `args_sha256`/bytes/sha256; approval table
   keeps hash only; `TOOL_CALL`/`APPROVAL_REQUEST` events carry hash (no argv/content body). File content and shell
   argv may contain secrets → never echo, never persist, never log beyond non-secret classes.
3. **Child environment**: explicit allowlist (`env_safety::ALLOWED_ENV`, key excluded — existing regression test);
   stdin closed; no shell interpolation; job-object containment.
4. **Path containment**: every file_write path goes through the six-vector jail; write target must stay inside after
   canonicalisation; temp file in same directory → rename; no partial-file claim on I/O failure.
5. **Approval integrity**: nonce single-use + TTL + args binding + decision single-winner + boot reaper; engine waits,
   never polls approvals and never auto-approves; UI resolves through existing `APPROVAL_RESOLVE` command with the
   additive nonce (contract already carries six fields — W12 rule "producer MUST populate all six").
6. **Crash/replay**: at-most-once; consumed grant + missing result = fail closed on boot, run interrupted with
   `E_APPROVAL_TIMEOUT`; never re-execute.
7. **Truthfulness**: no fake success; a side-effect tool returns `Ok` only after the executor's own success; shell
   non-zero exit is data in the result, not `Ok` success semantics of the tool (the summary says `success: false`).
8. **Key custody**: no provider key in child env, no key in approval rows/ledger/events; shell_exec cannot read the
   server env (allowlist) — existing `child_process_does_not_inherit_the_provider_key` test must stay green.

## 10. Tests (implementation wave; deterministic, offline where possible)

Tools layer (tempdir; no paid/key access):
1. `execute_approved_refuses_without_grant` (file_write/shell_exec) ⇒ `E_APPROVAL_REQUIRED`, zero FS/process change.
2. `execute_approved_refuses_wrong_args_after_grant` ⇒ `E_APPROVAL_REQUIRED`; grant NOT consumed for a pre-check
   failure (path escape/stale/oversize) — assert `executed_at IS NULL` afterwards, then approve and execute once.
3. `file_write_happy_path_atomic_and_bounded`: content written inside jail, result summary path/bytes/sha256, no content
   echo, temp file gone; parent dirs created inside; escape vectors refused; directory target refused; >1 MiB refused.
4. `file_write_stale_sha256_guard`: matching `expected_sha256` accepted; missing file / mismatch ⇒ `E_TOOL_SCHEMA`,
   file unchanged, grant unconsumed; malformed hex refused.
5. `file_write_once_only`: second execution of the same AllowOnce grant ⇒ `E_APPROVAL_REPLAY`, file unchanged.
6. `shell_exec_happy_path`: allowlisted program (`cmd /C echo` or existing ping helper) under approval; result bounded,
   exit code/success truthful; non-zero exit reported as result; disallowed program refused pre-grant.
7. `shell_exec_timeout_cancel_env`: timeout ⇒ `E_TOOL_TIMEOUT`; cancel ⇒ `E_TOOL_CANCELLED`; child cannot see
   `DEEPSEEK_API_KEY` (extend existing canary); stdout cap truncation.
8. `denied_and_expired_do_not_execute`: deny ⇒ no FS/process change, `ToolOutcome::Denied`; expiry ⇒
   `E_APPROVAL_TIMEOUT`, zero effects; replay of decision ⇒ `E_APPROVAL_REPLAY`.

Core layer:
9. `approval_wait_roundtrip`: engine suspends (persist `awaiting_approval`, publish `TOOL_CALL` then
   `APPROVAL_REQUEST` with six fields), ws resolve (nonce) wakes the waiter, engine re-verifies, executes exactly once,
   publishes `APPROVAL_RESOLVED` then `TOOL_RESULT`, persists `tool_turns`.
10. `approval_deny_timeout_cancel_replay` engine-level: outcomes map to `Denied/Timeout/Cancelled`, no side effect;
    duplicate resolution/replay ⇒ `E_APPROVAL_REPLAY` and no second execution.
11. `side_effect_lane_flag_off_preserves_1_2_behavior`: refusal `E_TOOL_NOT_ALLOWED`, no approval row created.
12. `boot_recovery_fails_pending_approval_run`: run persisted `awaiting_approval` + restart ⇒ interrupted with
    `E_APPROVAL_TIMEOUT`; pending row expired; nothing executed.
13. `ledger_event_truth`: TOOL_CALL/TOOL_RESULT/APPROVAL_REQUEST/APPROVAL_RESOLVED ordering and payload fields;
    approval-decision ledger row contains `args_sha256` only, no argv/content.

UI/e2e (future wave, noted not implemented here): approval card renders tool + args_sha256 + TTL/nonce, approve/deny
disabled while resolving, cannot approve twice, cannot render optimistic success; fake-provider e2e with
`HIVEROUTER_SIDE_EFFECTS=1`.

## 11. Rollback

- One feature flag (`HIVEROUTER_SIDE_EFFECTS`, default `0`) gates the entire lane; code is additive (`side_effects.rs`,
  `approval_wait.rs`, config fields, engine branches). Revert the wave commit ⇒ read-only behavior unchanged.
- No schema migration: `approvals` and `tool_turns` tables already exist; new columns none. Reverting leaves unused
  rows (harmless) — no destructive down-migration needed.
- If the lane is enabled and misbehaves at runtime: flip the env flag off and restart; pending `awaiting_approval` runs
  fail closed by boot recovery (`E_APPROVAL_TIMEOUT`) rather than resume.
- Approval gate/table changes are none; ledger format unchanged; no release tag/tag movement.

## 12. What will work / will not work (capability honesty)

**Will work**
- Approved `file_write` of UTF-8 content ≤1 MiB inside the workspace jail, atomic replacement, stale-write guard,
  bounded summary, no content echo; once-only grant, deny/timeout/replay/cancel semantics; ledger + tool_turns evidence.
- Approved `shell_exec` of bare allowlisted programs with exact argv, jailed cwd, explicit env allowlist, no shell
  interpolation, 60 s (300 s hard cap) timeout, kill-on-close Job Object reaping grandchildren, capped output,
  cancel → `E_TOOL_CANCELLED`.
- Run suspension for consent using the existing `run_state` transition table; UI can resolve with nonce; boot reaper
  and recovery fail closed; events truthful and ordered.
**Will not work / explicitly not claimed**
- **No OS-level sandbox**: the shell child runs as the server user. The jail constrains *cwd* and *file_write paths*,
  not the child's ability to open other files or reach the network. Approval is the only gate; document this.
- **`cmd.exe` on the default allowlist is a general shell** (`cmd /C …` can run arbitrary commands). For a
  model-driven loop, recommend a narrower default (e.g. exclude `cmd.exe`, or require argv-prefix policy) or accept
  the risk explicitly in the approval display; the plan does not silently widen this.
- `git.exe`/`cargo.exe` can also execute repository-controlled code (hooks, `cargo run`); approval review must show
  program+argv hash, and the UI cannot show argv content (contract payload carries only `tool` + `args_sha256`).
  **This is a real UX/security gap**: showing an argument preview needs a contract addition (new optional field), which
  this plan deliberately keeps out of scope (W12 froze the payload).
- No elevation/admin, no PTY/interactive stdin, no long-running daemons, no binary writes, no directory deletion,
  no undo of a completed side effect (rollback is pre-execution only), no network isolation, no concurrent-write merge.
- Crash exactly-once is impossible across a process boundary: the grant is consumed **before** the side effect to make
  it at-most-once; a crash between consume and effect leaves an unknown state (recorded as failed/interrupted, never
  retried).

## 13. Naming audit (frozen names vs new names)

| Concept | Existing name (keep) | New name decision | Rationale |
|---|---|---|---|
| Approval request/decision | `APPROVAL_REQUEST`, `APPROVAL_RESOLVED`, `APPROVAL_RESOLVE`, `ApprovalRequest`, `ApprovalGate`, `ApprovalDecision` | unchanged | frozen contract/wire names |
| Grant check | `ApprovalGate::verify_grant` | reuse; no alias | "grant" already means the checked execution right |
| Executor split | `execute_read_only` | `execute_approved` (granted), `execute_file_write`, `execute_shell_exec` | "approved" = grant verified; "read_only" = no grant needed; no "perm"/"consent"/"allow" synonyms |
| Permission model | `PermissionClass::{Read,Mutate,Execute,Network}` | unchanged | tier/registry authority, distinct from approval |
| Effects | `run_state::Effect::{RequestApproval, ResumeFromApproval, DenyApproval, FailOnApprovalTimeout}` | reuse exactly | one state-machine vocabulary |
| Events | `TOOL_CALL`, `TOOL_RESULT`, `ToolOutcome` | unchanged | contract already frozen at 23 events |
| Wait seam | — | `ApprovalBroker`, `register/notify/drop_run`, `ApprovalWaitOutcome` | "waiter" not "channel"/"bus" (single decision seat) |
| Side-effect lane flag | — | `HIVEROUTER_SIDE_EFFECTS` (bool) | "side effects" matches the item's language |
| Files | `executor.rs` (read-only) | new `side_effects.rs`, new `approval_wait.rs` | no read-only file gains write authority; review boundary stays obvious |

Naming rules enforced: refusal codes only from the frozen table; no new event/command names; fields keep
`args_sha256`, `tool_call_id`, `nonce`, `expires_at`; docs/tests use `awaiting_approval` (state) and `verify_grant`
(verb) consistently.

## 14. Open questions / approval gates for the implementation wave

1. **Q1 (blocks enabling by default):** Should the model be offered `cmd.exe` at all? Recommend excluding it from the
   model-lane default allowlist; operator may opt in explicitly. This is a Lead/product security decision.
2. **Q2:** Approval preview: UI currently sees only `tool` + `args_sha256` + ids/TTL. Is that sufficient for operator
   consent for shell/files, or must the contract add an optional redacted `args_preview` (contract change, W12+)?
3. **Q3:** `expected_sha256` mismatch code: keep `E_TOOL_SCHEMA` (decision above) or accept `E_APPROVAL_REQUIRED`?
   Recommendation: `E_TOOL_SCHEMA` (caller-fixable, no grant consumed) — confirm with contract owner.
4. **Q4:** Shell timeout config cap: 60 s default / 300 s hard cap acceptable? (Per-run wall-clock budget not yet a
   first-class cap; the existing 4-turn/16-call budget is the loop bound.)
5. **Q5:** Recovery semantics for an in-flight shell child at process death: kill-on-close Job Object kills it with the
   server; boot recovery marks the run interrupted — acceptable as the only honest answer (no orphan reattach).

## 15. Acceptance criteria for items 3 and 16

- Item 3: a model-originated `file_write` and a `shell_exec` can execute **only** after an approved, single-use grant;
  every requirement in §4/§5 has a named negative test; no read-only path executes side effects; lane default-off keeps
  today's refusal behavior. Evidence: `cargo test -p hiverouter-tools -p hiverouter-core --locked` green, plus
  `docs/contracts/ADAPTERS.md`/`RUNBOOK` note for the two new env vars.
- Item 16: the engine now uses `run_state` approval transitions; `APPROVAL_REQUEST` is published with all six fields;
  `verify_grant` is the only execution authority and is called exactly once per execution; deny/timeout/cancel/replay
  paths produce the mapped outcomes with zero side effects; boot recovery fails pending runs closed; events and
  `tool_turns`/ledger rows are complete and contain no argument bodies or secrets.
- Explicit non-claims: OS sandboxing, argv preview in UI, exactly-once across crashes, interactive shell (see §12).

## 16. Logic-gatekeeper Phase 1 (for the implementing wave's edits)

```json
{"phase1": {
  "architecture": "approval.rs owns the single-use grant (SQLite UNIQUE(nonce), decision CAS, executed_at CAS); tools_bridge owns the producer/resolver/ledger seam; run_state.rs owns legal approval edges/effects; execution.rs owns the loop and will own the wait; side_effects.rs (new) owns the only granted execution path; jail.rs/shell.rs own containment/execution primitives; config/http wire the shell policy and default-off flag.",
  "dataFlow": "model tool_call -> engine plan/validate -> Suspend effect -> request row (args_sha256,nonce,TTL) -> publish TOOL_CALL+APPROVAL_REQUEST -> waiter -> ws APPROVAL_RESOLVE(nonce) -> gate decision + ledger -> notify -> Resume effect -> read-only pre-checks -> verify_grant (args re-hash, executed_at CAS) -> execute one tool (file_write atomic | shell_exec policy) -> bounded result -> TOOL_RESULT + tool_turns; deny/timeout/cancel produce named outcomes with zero side effects; boot reaper/recovery fail closed.",
  "edgeCases": [
    "execution attempted with no grant / wrong args / expired row => E_APPROVAL_REQUIRED, nothing touches disk or spawns",
    "same AllowOnce grant executed twice => first wins executed_at, second E_APPROVAL_REPLAY, no second side effect",
    "grant consumed but executor fails (I/O error) => at-most-once, run fails by name, no retry",
    "bad path / oversize content / stale expected_sha256 checked before verify_grant => grant NOT consumed",
    "deny / TTL timeout / cancel / kill while awaiting => Denied/Timeout/Cancelled outcomes, pending row expired, boot never auto-executes",
    "duplicate APPROVAL_RESOLVE (nonce replay) => gate refuses before ledger, waiter not notified, no execution",
    "shell timeout or cancel => child+grandchildren killed (Job Object), E_TOOL_TIMEOUT/E_TOOL_CANCELLED, no success claim",
    "crash between grant consumption and effect => run interrupted with E_APPROVAL_TIMEOUT at boot, never re-executed"
  ],
  "rootCause": "The 1.2 loop has two halves of one safety property but no bridge: the read-only executor refuses side effects while the approval gate/run-state table are complete but unreachable from the loop, so a model's Mutate/Execute call is refused by name. Wiring execution without the grant check would let unreviewed side effects run; wiring the grant check without the wait/resume seam would deadlock or auto-execute. The fix is one ordered chain: commit row -> publish request -> suspend -> decision/ledger -> notify -> re-verify grant -> execute exactly once."
}}
```

### Evidence-index note

All claims above are READ from the cited files/lines this session; no code was written, no cargo build/test or
Playwright was run (foresight planning seat), no key was accessed. The two items are the only unimplemented pieces of
the W12 tool-loop contract; the rest of the 1.2 contract surface (read-only loop, events, persistence, approval
primitives, state machine, recovery, ledger) exists and is tested.
