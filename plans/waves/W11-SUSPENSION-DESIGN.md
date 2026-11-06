# W11 — 1.2 RUN SUSPENSION / APPROVAL DESIGN (decision-complete; no code)

**Seat:** F (1.2 run suspension/approval design) · **Write scope:** this new file only (`plans/waves/W11-SUSPENSION-DESIGN.md`);
read-only everywhere else. **No product/test/config/git writes; no key access; no paid calls; no commit.**
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **HEAD at recon:** `9651722` ("docs: record sealed 1.1.0 hashes and
live streaming/replay proof"). Skills applied: `/forgewright-recon`, `/forgewright-lead`, `/logic-gatekeeper`,
`/integration-logic`, `/logic-debugger` (evidence of each in §12).
**Relationship to W10:** this document refines W10 blockers **B2 (run suspension/resume)**, **B3 (conversation
persistence)** and **B6 (approval UX/security)** into an implementable 1.2 design; it does not re-decide W10 B1/B4/B5
and it does not claim any product behavior exists today.

---

## 1. Recon ledger (files read this session; bytes / lines / sha256)

| # | File | bytes | lines | sha256 (lower) | Anchors used |
|---|---|---:|---:|---|---|
| 1 | `backend/crates/hiverouter-core/src/execution.rs` | 23,530 | 641 | `9b055295…f52a82d4` | `ProviderExecutor` 49-77; `StreamedReply` 84-98; `RunRegistry` 128-196; `RunEngine` 198-222; `submit_prompt` 326; `cancel_run` 411; `kill_worker` 437; `dispatch` 455-499; `dispatch_inner` 501-574; `execute_task` 591-…; `caps.check_calls(1)` 505,597 |
| 2 | `backend/crates/hiverouter-core/src/ws.rs` | 21,447 | 537 | `d418dda6…b483072a` | `ApprovalResolve` arm 465-476 (ACK + `APPROVAL_RESOLVED`, refusals mapped) |
| 3 | `backend/crates/hiverouter-core/src/db.rs` | 15,761 | 420 | `1822162a…e8a47c` | migrations v1 `bootstrap` 32-47, v2 `runs_tasks_chat` 48-95 (chat_messages 74-82), v3 `snapshot_index` 96-…; no v4; module doc claims cache/metrics/usage rows (line 3) but no such tables exist |
| 4 | `backend/crates/hiverouter-core/src/recovery.rs` | 2,787 | 75 | `4b3c7098…f9842df3` | active runs → `Interrupted` 60-68 (no error_code); snapshot restore 70+ |
| 5 | `backend/crates/hiverouter-core/src/caps.rs` | 2,918 | 87 | `d21061f2…eba704782` | `MAX_CALLS=40` 15; `check_calls` 66-71 (pure comparison, no counter) |
| 6 | `backend/crates/hiverouter-core/src/tools_bridge.rs` | 6,788 | 167 | `4767e3e0…0dc5f8e` | `migrate` 73-77; gate 62/98-101; `resolve_approval` 141-158; `approval_resolved_event` 173-184 |
| 7 | `backend/crates/hiverouter-core/src/store.rs` | 15,733 | 431 | `ca2bc6dd…5fdf9eb` | active status query incl. `awaiting_approval` 95-114; `set_run_status` 136; `append_message` 272; `messages` 308; state map 445 |
| 8 | `backend/crates/hiverouter-tools/src/approval.rs` | 34,944 | 948 | `e32411e6…5652c22` | TTL 26; decision enum 28-81; request payload 83-106; DDL 141-165; `insert_request` 171-202; `resolve_with_nonce` 415-438; `resolve_with_reason` 440-514; `pending_for_run` 520-538; `ApprovalGate` 540-712 (`request` 580-611, `verify_grant` 653-711); tests 714+ |
| 9 | `backend/crates/hiverouter-tools/src/tool.rs` | 19,612 | 558 | `353ab4e6…319765` | `PermissionClass` 30-64: Read/Mutate/Execute/Network, `requires_approval = !Read` (55-57) |
| 10 | `backend/crates/hiverouter-contract/src/state.rs` | 4,791 | 147 | `dea156dd…bea134b` | 16 `AgentState` values 18-51; `is_terminal` 99-108 (Failed/Completed/Cancelled/Killed/Interrupted) |
| 11 | `backend/crates/hiverouter-contract/src/event.rs` | 18,508 | 489 | `c5776ef6…409b2aa` | `APPROVAL_REQUEST` payload 242-252 = `{approval_id, tool, decision?}` **only**; `APPROVAL_RESOLVED` 254-263; `EventName::ALL` 65 |
| 12 | `backend/crates/hiverouter-contract/src/command.rs` | 21,334 | 533 | `2eeafcdd…73964f` | `ApprovalResolve` 187-199 (`approval_id + nonce + decision + reason?`); `CommandName::ALL` 65 |
| 13 | `backend/crates/hiverouter-contract/src/refusal.rs` | 17,229 | 314 | `9c99c447…9abe9f97` | `E_APPROVAL_REQUIRED/REPLAY/TIMEOUT/NOT_FOUND/DECISION_INVALID` 179-193; `E_TOOL_CANCELLED`; `E_CALL_LIMIT` |
| 14 | `web/src/panels/ApprovalOverlay.tsx` | 2,857 | 64 | `eb1595b3…030a37bf` | ready for wire nonce: buttons disabled when `nonce === null`, named gap copy `STRINGS.approval.nonceMissing` |
| 15 | `web/src/state/store.ts` | 23,799 | 701 | `8b3e95a9…4a03374` | `ApprovalRecord` 55-60 (nonce nullable); `APPROVAL_REQUEST` 325-336; `APPROVAL_RESOLVED` 338-344; `resolveApproval` 573-597 (refuses without nonce, named `E_SCHEMA_INVALID` gap) |
| 16 | `web/src/api/contract.ts` | 29,829 | 845 | `1dc79439…a1515bd` | `ApprovalRequestPayload` 223-229 ("nonce arrives with the pending approval (optional for back-compat)"), `ApprovalResolvePayload` 70-90, event union 282-309 |
| 17 | `plans/waves/W10-TOOL-LOOP-DECISION.md` | 29,603 | 245 | `9222f082…7ca99c` | §3 blockers B1-B8; §4.3 approval lifecycle sketch; §4.5 persistence sketch |

Supporting verified checks: `ApprovalGate::request` has no production caller (only `approval.rs` tests + core
`tests/ws_transport.rs:306-327` seeds a row directly through the gate); no code publishes `Event::ApprovalRequest`;
`rg 'awaiting_approval'` finds only the contract/state map/store-active query — no transition produces it;
`rg 'check_calls'` finds exactly `execution.rs:505,597` with the literal `1`; `rg 'CACHE|usage'` over the SQLite schema
finds no cache/metrics/usage table despite `db.rs:3`.

---

## 2. Phase 1 — logic-gatekeeper (written before any mutation; no mutation is performed by this seat)

```json
{
  "phase1": {
    "architecture": "Run execution is a linear state machine in core/src/execution.rs: submit_prompt creates a run + one CancellationToken, dispatch runs plan -> tasks -> execute_task -> terminal, and dispatch maps only Ok/ToolCancelled/Err to Completed/Cancelled/Failed. Approval state lives in a second owner, tools/src/approval.rs (ApprovalGate over a SQLite `approvals` table created by tools_bridge::migrate, not by core/db.rs MIGRATIONS). The frozen wire layer (contract) exposes only APPROVAL_REQUEST {approval_id, tool, decision?} and APPROVAL_RESOLVE {approval_id, nonce, decision, reason?}; the UI (ApprovalOverlay + store) is already built to consume a server nonce but disables approve/deny when it is absent. Persistence owns runs/tasks/chat_messages/snapshots only; there is no persisted run phase, provider-call counter, tool-turn, or suspension cursor.",
    "dataFlow": "A model-originated side-effect tool call would enter at the provider/executor seam (currently absent: adapter discards ToolCallDelta, provider_executor.rs:171-173) and must be gated by the approval row before any workspace/shell side effect. Intended flow: executor turn returns tool calls -> core asks ApprovalGate::request (commits approval row with nonce, args_sha256, TTL 120 s) -> run/task status awaits approval -> APPROVAL_REQUEST is published with the row's nonce -> client sends APPROVAL_RESOLVE {approval_id, nonce, decision} -> ws arm resolves through the gate (single-use CAS) -> waiter wakes -> approve: verify_grant(args) then execute exactly once; deny/timeout/cancel: no side effect. Restart loses the in-memory waiter, so persisted rows are the only truth.",
    "edgeCases": [
      "two APPROVAL_RESOLVE frames race (approve vs deny, or replay): exactly one may win the DB CAS; loser gets E_APPROVAL_REPLAY / E_APPROVAL_NOT_FOUND and no execution",
      "TTL expiry races with a valid resolve: DB CAS on decision IS NULL decides the single winner; expiry winner fails the run E_APPROVAL_TIMEOUT with zero side effects",
      "CANCEL_RUN / KILL_WORKER while the run is awaiting approval: waiter must consume/cancel the approval and never execute afterwards; pending row must not stay resolvable",
      "process restart while awaiting approval: recovery must mark the run interrupted and expire pending approvals; nothing is auto-approved and no tool side effect may replay",
      "duplicate/late APPROVAL_REQUEST for the same tool_call_id (e.g., retry after a dropped frame): the UI must be idempotent by approval_id and the engine must not create a second pending row for the same tool_call (unique run_id+tool_call_id pending index)",
      "second SUBMIT_PROMPT while awaiting approval must keep the single-run policy and refuse E_RUN_ACTIVE (awaiting is active)",
      "approval for different args than the committed args_sha256 (tampered model output or changed tool args) must be refused E_APPROVAL_REQUIRED before execution"
    ],
    "rootCause": "The run engine has no suspension state, no approval waiter and no resume path, and the frozen APPROVAL_REQUEST payload carries no nonce (nor run/task/tool-call/args binding), so even though the approval gate and the UI are implemented, a run can never pause for consent and an operator can never resolve a model-originated approval; additionally the compiled approval table sits outside the versioned migration ledger and the provider-call cap is a literal comparison without a counter, so a resumed tool loop would be neither durably recorded nor bounded."
  }
}
```

Phase 2 (only after Phase 1 acceptance): every resume/deny/timeout/cancel path is a single DB CAS on the approval row
before any side effect; the engine never executes on a wake without a winning grant; callers of `submit_prompt`,
`cancel_run`, `kill_worker` keep their signatures; the same contradiction (execution without a consumed approval) now
fails closed by construction (waiter wake ⇒ grant check ⇒ execute; no grant ⇒ named refusal, no side effect).

---

## 3. Decisions (decision-complete; each with rejected alternative)

**D1 — Suspension is in-process; no crash-resume of a suspended turn.** The suspended run keeps its tokio task alive
and waits on an approval waiter; the durable truth is the `approvals` row + `runs.status='awaiting_approval'` + the
persisted assistant turn. A crash loses the executor cursor; recovery marks the run `interrupted` (existing terminal
state) and expires the approval. *Rejected:* persisting a full executor continuation (provider stream offset, partial
tool-call assembly, reasoning_content buffer) — high risk, large state, no product need in 1.2; revisit only if
crash-resume of multi-turn loops becomes a requirement.

**D2 — `awaiting_approval` is a run status and a task state, produced by the engine (not the gate).** `RunEngine` sets
`runs.status=awaiting_approval` and the waiting task's state to `awaiting_approval` before publishing events; it does
not release the single-run `active_run` lock. New prompt during a wait → existing `E_RUN_ACTIVE`. *Rejected:* a new
non-enum phase column — the frozen enum already contains `awaiting_approval` and the store already treats it as active
(`store.rs:101,445`), so adding a parallel phase would create a second vocabulary.

**D3 — Waiter registry keyed by approval id in the engine; no global channel.** Add
`ApprovalWaiters { Mutex<HashMap<ApprovalId, oneshot::Sender<ApprovalResolution>>> }` to `RunEngine`
(test-injectable like `RunRegistry`). The engine arms the waiter **before** publishing `APPROVAL_REQUEST` (no lost-wake
race). The ws `ApprovalResolve` arm keeps authority in the DB (`resolve_command`), then calls
`engine.notify_approval_resolved(&resolution)` (best-effort wake; if the waiter is gone, the DB row is already decided
and nobody executes). *Rejected:* resolving in the engine and notifying the gate — would duplicate the nonce/replay
owner; *rejected:* broadcast channel per run — unbounded lifetime and ambiguous receiver identity.

**D4 — Approve resumes and executes exactly once.** Wake with `AllowOnce`/`AllowSession` ⇒ engine calls
`ApprovalGate::verify_grant(run, tool, args)` (the single-use executor marker) then executes the tool; a second wake or
a second execution attempt for the same grant is `E_APPROVAL_REPLAY` before side effects. A crash between
`verify_grant` and the side effect loses the grant (executed_at set) ⇒ the run is interrupted; the design accepts
at-most-once (never at-least-once) for side effects. *Rejected:* marking executed after the side effect — a crash
between side effect and marker would permit a double execution.

**D5 — Deny does not fail the run.** Deny ⇒ approval row `decision='deny'` (terminal for that row), publish
`APPROVAL_RESOLVED{decision:"deny"}`; the engine synthesizes the tool result `{outcome:"denied", code:null,
message:"denied by operator"}` for that `tool_call_id`, marks the task `running` again, and continues the loop (the model
finalizes normally within the turn cap). No side effect. *Rejected:* failing the run on deny — destroys the loop's
recoverable-refusal semantics and makes operator denial indistinguishable from a product failure.

**D6 — Timeout fails the run by name; it is not converted into a deny.** The wait is armed with
`ApprovalGate::ttl_secs()` (120 s, `APPROVAL_TTL_SECS`) using the DB clock (`Timestamp::now`), and the engine performs
the expiry as a DB CAS (decision `expired`). If the CAS wins: `RUN_STATUS{failed, error_code:"E_APPROVAL_TIMEOUT"}`
(retryable=true per contract) and `NODE_STATE failed`; no side effect; no `APPROVAL_RESOLVED` (the wire decision
vocabulary is `approve|deny` only). If a resolve won the CAS first, the resolution is used. *Rejected:* treating timeout
as an implicit deny and continuing the loop — it hides operator abandonment and can silently consume the model's turn
budget; this remains a product-call alternative recorded in §13 (D-ALT-TIMEOUT) and can be flipped without schema
change.

**D7 — Cancel/kill while waiting are named, side-effect-free terminations.** `CANCEL_RUN` cancels the run token; the
waiter select-arm performs CAS `decision='cancelled'` (internal label, not a wire decision), wakes/chains no execution,
and the existing dispatch mapping yields `cancelled` with no error code. `KILL_WORKER` on the waiting task does the same
for that task and returns `E_TOOL_CANCELLED` from the engine (existing dispatch mapping: `ToolCancelled` ⇒ `cancelled`),
consuming the approval. In both cases the UI clears the overlay because the run/task leaves the active set; no
`APPROVAL_RESOLVED` frame is fabricated for cancelled/expired decisions. *Rejected:* leaving a cancelled approval
pending — it would remain resolvable after the run is gone and could execute against a terminal run.

**D8 — One additive contract patch (v1.2-alpha) covers approval payload + tool visibility; no second bump.**
Extend `APPROVAL_REQUEST` to
`{approval_id, tool, decision:null, run_id, task_id, tool_call_id, permission_class, args_sha256, nonce, expires_at,
turn}` (all required at 1.2; UI decoders mark the new fields optional for back-compat as today) and add
`TOOL_CALL {run_id, task_id, tool_call_id, tool, permission_class, args_sha256, turn}` and
`TOOL_RESULT {run_id, task_id, tool_call_id, outcome: ok|refused|denied|timeout|cancelled, code?, bytes?, sha256?}`.
`APPROVAL_RESOLVE` is unchanged (nonce was already required). Optionally add `E_TOOL_LOOP_LIMIT` (one new refusal) for
turn/tool caps; approval-timeout keeps `E_APPROVAL_TIMEOUT`. Counts after patch: events **21 → 23**, commands **14**
(unchanged), refusals **65 → 66** if `E_TOOL_LOOP_LIMIT` is approved. *Rejected:* publishing the nonce out-of-band via
HTTP — the WS bus is the frozen event channel and the UI's nonce-null path exists precisely because the wire lacks it.

**D9 — Persistence is one additive migration v4 owned by core.** See §7 for exact DDL. It (a) adds run counters
(`calls`, `turn`, `tool_calls`) so caps survive suspension/restart, (b) adds `chat_messages.turn` + `meta_json` so tool
turns and reasoning content are persisted, and (c) promotes the `approvals` DDL into the versioned migration ledger
(same idempotent `CREATE TABLE IF NOT EXISTS`; `tools::approval::migrate` remains for in-memory tests and is documented
as a test seam). *Rejected:* keeping counter/budget state only in memory — a suspended run's caps would reset on
restart and an unbounded loop could continue after recovery; *rejected:* per-waiter rows in a new table — the approvals
row already carries run/task/tool_call/tool/args binding needed for resume and audit.

**D10 — Budget must survive suspension.** The adapter's per-run `RunBudget` map is evicted only by `finish_run`, and
`dispatch` calls `finish_run` only after the terminal status; suspension must not call `finish_run`. On restart the
budget is discarded together with the interrupted run (no spend continuation), and measured usage rows already written
remain truthful. *Rejected:* persisting the Decimal budget to SQLite — not needed for at-most-once + interrupted
restart policy; it becomes needed only if crash-resume is adopted (D1).

**D11 — Single approval per tool call; approval waits are capped.** Add a unique partial index on
`approvals(run_id, tool_call_id) WHERE decision IS NULL` so a retried/duplicated request cannot create two resolvable
rows for one call; the second request returns `E_APPROVAL_REPLAY`. Per-run caps: `MAX_APPROVAL_WAITS_PER_RUN = 8`
(≤ tool-call cap 12 by construction) and the run fails `E_TOOL_LOOP_LIMIT`/`E_CALL_LIMIT` before a 9th wait.
*Rejected:* unbounded waits — a looping model could keep a single-run server suspended indefinitely (each wait is
bounded by 120 s; unbounded count is the residual risk).

**D12 — Time is a dependency, not a global.** `RunEngine` gets `approval_ttl_secs: u64` (default 120,
`with_approval_ttl` for tests) and a `now_unix_secs: fn() -> u64`-shaped clock seam (default `Timestamp::now`), so the
timeout path is deterministic in tests. Gate calls stay `(conn, …, now_unix_secs)` as today.

---

## 4. Run/task state machine (legal transitions only)

`AgentState` already contains `AwaitingApproval` (`state.rs:38,55-72`); terminal = {failed, completed, cancelled,
killed, interrupted} (`state.rs:99-108`); active = all others (`store.rs:101`).

| From | Event/action | To | Side effects / events |
|---|---|---|---|
| `planning` | plan ok, leaf dispatch | `running` (detail leaves `queued`→`running` as today) | `NODE_STATE running` |
| `running` (leaf turn) | model returns side-effect tool call | `awaiting_approval` | approval row committed **before** the transition; `RUN_STATUS awaiting_approval`, `NODE_STATE awaiting_approval`, `APPROVAL_REQUEST` (with nonce), `TOOL_CALL` |
| `awaiting_approval` | `APPROVAL_RESOLVE approve` (fresh nonce) | `running` | `APPROVAL_RESOLVED approve`; `verify_grant` then tool executes; `TOOL_RESULT ok`; loop continues |
| `awaiting_approval` | `APPROVAL_RESOLVE deny` | `running` | `APPROVAL_RESOLVED deny`; `TOOL_RESULT denied`; loop continues |
| `awaiting_approval` | TTL expired (CAS wins) | `failed` + `E_APPROVAL_TIMEOUT` | `RUN_STATUS failed`; no side effect; row `decision='expired'` |
| `awaiting_approval` | `CANCEL_RUN` | `cancelled` | `RUN_STATUS cancelled`; row `decision='cancelled'`; no side effect |
| `awaiting_approval` | `KILL_WORKER` (waiting task) | `cancelled` via `ToolCancelled` mapping | row `decision='cancelled'`; no side effect |
| any active | crash/restart | `interrupted` (existing recovery) | pending approvals expired; `error_code=E_APPROVAL_TIMEOUT` for runs that were awaiting |
| `awaiting_approval` | replay of an already-decided nonce | unchanged | `ERROR E_APPROVAL_REPLAY`; no state change, no side effect |
| `awaiting_approval` | mismatched nonce / wrong args | unchanged | `ERROR E_APPROVAL_REPLAY` / `E_APPROVAL_REQUIRED`; no side effect |

Illegal and rejected: `awaiting_approval → completed` directly; executing a tool while the run is not `running`; two
resolvable approvals for one `(run, tool_call_id)`; auto-approval on restart; `approved` without a consumed single-use
grant.

---

## 5. Suspension protocol (engine seam; design signatures, no code)

1. **Executor turn outcome** (provider/executor side, W10 B4 dependency): `ProviderExecutor::execute` becomes
   `execute_turn(...) -> Result<TurnOutcome, Refusal>` where
   `TurnOutcome = Final(StreamedReply) | ToolCalls { text: String, usage: Usage, calls: Vec<AssembledToolCall> }`.
   The engine persists the assistant turn (text + `meta_json.tool_calls` + turn index + reasoning_content when thinking
   is enabled) **before** any approval request, so the transcript is truthful even if the process dies mid-wait.
2. **Gate step in the engine** (`execute_task`): for each assembled call, resolve tool+permission class; if
   `PermissionClass::requires_approval()` (Mutate/Execute/Network, `tool.rs:55-57`):
   `approval = gate.request(conn, run, task, tool_call_id, tool, class, args, now)` (DB commit is the suspend point),
   arm waiter, set run/task `awaiting_approval`, persist the turn, publish `RUN_STATUS awaiting_approval`,
   `NODE_STATE awaiting_approval`, `TOOL_CALL`, `APPROVAL_REQUEST` (payload per D8), then
   `tokio::select! { resolution = waiter => …, _ = timeout(ttl) => expire-CAS, _ = token.cancelled() => cancel-CAS }`.
3. **Wake**: `notify_approval_resolved(resolution)` maps `ApprovalId → sender` and removes the entry under one lock;
   the engine re-reads the row (authority) and applies the D4/D5 CAS outcomes before touching a tool.
4. **Resume**: set run/task back to `running`, publish `RUN_STATUS running` + `NODE_STATE running`, execute the granted
   tool once, append the tool result message (`role=tool`, `meta_json={tool_call_id,outcome,code}`), then continue the
   loop to the next provider turn (bounded by D9/D11 caps) or finish the run.
5. **Ownership**: `RunEngine` owns the wait, the CAS application and the status transitions; `ApprovalGate` remains the
   single nonce/replay/TTL owner; `ToolsBridge` remains the single core→tools wiring point (add
   `resolve_approval_with_wake(&self, payload) -> (ApprovalResolution, Option<Refusal>)` only if the ws arm needs the
   waiter notification; alternatively ws calls `state.engine.notify_approval_resolved` after a successful resolve — one
   place, no duplicated authority). UI never mints a nonce; operator intent is approve/deny only.

---

## 6. Wire contract (exact additive payloads; contract patch B1)

```jsonc
// APPROVAL_REQUEST (extended; existing approval_id/tool/decision stay)
{"approval_id":"<uuid-v7>","tool":"file_write","decision":null,
 "run_id":"<uuid-v7>","task_id":"<uuid-v7>","tool_call_id":"call_...",
 "permission_class":"mutate","args_sha256":"<64 hex>","nonce":"<uuid-v7>",
 "expires_at":"RFC3339","turn":2}

// TOOL_CALL (new)
{"run_id":"<uuid-v7>","task_id":"<uuid-v7>","tool_call_id":"call_...","tool":"file_write",
 "permission_class":"mutate","args_sha256":"<64 hex>","turn":2}

// TOOL_RESULT (new; exactly one per TOOL_CALL)
{"run_id":"<uuid-v7>","task_id":"<uuid-v7>","tool_call_id":"call_...",
 "outcome":"ok|refused|denied|timeout|cancelled","code":null,"bytes":1234,"sha256":"<64 hex>|null"}
```
`APPROVAL_RESOLVE` stays `{approval_id, nonce, decision:"approve"|"deny", reason?}`. Event ordering on approval is
strict: `APPROVAL_RESOLVED` → `TOOL_RESULT` → next `REPLY_DELTA`/`REPLY_DONE`/`RUN_STATUS`. The patch requires: golden
schema regeneration, `EventName::ALL` count 21→23, matrix rows, `CHANGE-LEDGER-IMPL.md` row, UI `contract.ts` decoder
update + parity test, and doc-count updates (`FINAL-REPORT` event count). No command is added.

---

## 7. Persistence — migration v4 (`backend/crates/hiverouter-core/src/db.rs`, append-only)

```sql
-- v4 "suspension_approvals" up
ALTER TABLE runs ADD COLUMN calls       INTEGER NOT NULL DEFAULT 0;
ALTER TABLE runs ADD COLUMN turn        INTEGER NOT NULL DEFAULT 0;
ALTER TABLE runs ADD COLUMN tool_calls  INTEGER NOT NULL DEFAULT 0;
ALTER TABLE chat_messages ADD COLUMN turn INTEGER NOT NULL DEFAULT 0;
ALTER TABLE chat_messages ADD COLUMN meta_json TEXT NULL;
CREATE TABLE IF NOT EXISTS approvals (            -- promoted from tools::approval::migrate (same DDL)
  approval_id TEXT PRIMARY KEY NOT NULL, run_id TEXT NOT NULL, task_id TEXT NOT NULL,
  tool_call_id TEXT NOT NULL, tool_name TEXT NOT NULL, args_sha256 TEXT NOT NULL,
  permission_class TEXT NOT NULL, created_at TEXT NOT NULL, expires_at TEXT NOT NULL,
  nonce TEXT NOT NULL UNIQUE, decision TEXT, decided_at TEXT, reason TEXT, executed_at TEXT);
CREATE INDEX IF NOT EXISTS approvals_run_pending ON approvals (run_id, tool_name, args_sha256, decision);
CREATE UNIQUE INDEX IF NOT EXISTS approvals_one_pending_per_call
  ON approvals (run_id, tool_call_id) WHERE decision IS NULL;
-- v4 down (documented exception: approvals table/rows are audit data and are NOT dropped on rollback)
ALTER TABLE chat_messages DROP COLUMN meta_json;   -- SQLite >=3.35 (bundled rusqlite qualifies)
ALTER TABLE chat_messages DROP COLUMN turn;
ALTER TABLE runs DROP COLUMN tool_calls;
ALTER TABLE runs DROP COLUMN turn;
ALTER TABLE runs DROP COLUMN calls;
```
- `chat_messages.meta_json` shape: assistant `{"tool_calls":[…],"reasoning_content"?,"response_id"?}`; tool
  `{"tool_call_id":"…","outcome":"ok|denied|refused|timeout|cancelled","code"?}`. `turn` is the loop turn index
  (0 = first planner/leaf turn).
- `runs.calls/turn/tool_calls` are the **caps counters**: increment in the same transaction that records the
  provider call / tool call, and check before the next send/execute; they survive suspension and are read by
  `/api/history/runs` diagnostics.
- Migration tests: new `migration_v4_up_down_roundtrip` (fresh DB → v4 → v3 → v4; columns/index/table present/absent as
  specified), plus `chat_store` round-trip for `meta_json`/`turn`, plus the existing `snapshot_history` migration test
  unchanged. `ToolsBridge::migrate` keeps calling `tools::approval::migrate` for in-memory test DBs (idempotent), and
  boot order becomes: core migrations (v4) → tools bridge migrate (no-op).
- Replay/history: `HISTORY_SNAPSHOT` stays graph+status; suspended turns are visible via chat messages (`meta_json`) and
  the approvals ledger. Snapshot capture remains terminal-only; a suspended run's snapshot is **not** written on
  suspension (graph is unchanged); on restart the run becomes interrupted and a terminal snapshot is written by the
  existing dispatch path only if a dispatch exists — recovery currently writes no snapshot, recorded as residual.
- No new DB for budget; usage remains in the run/ledger data already present (see §12 U2).

---

## 8. Nonce binding and approval security (B6)

- **Binding**: the approval row binds `(run_id, task_id, tool_call_id, tool, permission_class, args_sha256)`; the wire
  carries `nonce` + `args_sha256` so the operator sees what is approved. `verify_grant(run, tool, args)` re-hashes the
  exact argument JSON (`hash_args`, `approval.rs:132-134`) and refuses `E_APPROVAL_REQUIRED` on mismatch — a model
  cannot change arguments between request and execution.
- **Single use**: `UNIQUE(nonce)` at insert + `UPDATE … WHERE decision IS NULL` CAS at resolve + `executed_at` marker at
  grant consumption = one decision, one execution; replay paths return `E_APPROVAL_REPLAY` before side effects.
- **Operator surface**: approve/deny only (`ApprovalDecision::from_wire` accepts exactly those); `AllowSession` stays
  programmatic until a future contract adds a wire value (documented). UI: `ApprovalOverlay` shows tool + approval id and
  the TTL/args-hash fields after the D8 patch; buttons stay disabled without a nonce (current honest gap). Add
  `expires_at` countdown and `args_sha256` display in the same patch; no raw args body is put on the wire or in the
  ledger (args_sha256 only; previews, if added, are operator-local and redacted).
- **Threat notes**: a compromised UI can approve only a request that exists with that nonce; it cannot mint approvals or
  replay. Model-driven shell/file-write/spawn remain gated by `PermissionClass::requires_approval` and the existing
  jail/shell policy; phase A read-only tools need no approval. No auto-approve on restart, ever.
- **Caps**: approvals per run are bounded (D11); approval waits are bounded by TTL; a 9th wait is a named cap refusal.

---

## 9. Deny / timeout / cancel / kill / restart semantics (exact)

| Case | DB action (single authority) | Wire events (order) | Run status / error | Side effect | UI |
|---|---|---|---|---|---|
| approve | CAS decision `allow_once`/`allow_session`; grant `executed_at` set once | `APPROVAL_RESOLVED approve` → `TOOL_RESULT ok` → next turn | `running` then terminal | tool executes once | overlay removed |
| deny | CAS decision `deny`; row terminal | `APPROVAL_RESOLVED deny` → `TOOL_RESULT denied` | stays `running` | none | overlay removed; model continues |
| timeout (120 s) | CAS decision `expired` (clock = DB time) | `RUN_STATUS failed E_APPROVAL_TIMEOUT` + `NODE_STATE failed` | `failed` `E_APPROVAL_TIMEOUT` (retryable) | none | overlay removed by terminal run status |
| cancel | CAS decision `cancelled` | `RUN_STATUS cancelled` (+ task node cancelled) | `cancelled` no error | none | overlay removed |
| kill worker | CAS decision `cancelled`; task killed | `NODE_STATE killed` + `RUN_STATUS cancelled` (ToolCancelled mapping) | `cancelled` no error | none | overlay removed |
| replayed nonce | no write | `ERROR E_APPROVAL_REPLAY` | unchanged | none | toast |
| mismatched nonce | no write | `ERROR E_APPROVAL_REPLAY` | unchanged | none | toast |
| wrong args at grant | no write | `ERROR E_APPROVAL_REQUIRED` (run fails via loop guard) | `failed` `E_APPROVAL_REQUIRED` (product call: deny result instead) | none | refusal |
| restart while waiting | boot: runs non-terminal → `interrupted` (+`error_code=E_APPROVAL_TIMEOUT` when was awaiting); pending approvals → `expired` | none at boot | `interrupted` | none | next client sees terminal row |

Race rule for all cells: the approval row CAS is the winner; the in-memory waiter is only a wake-up hint. If two paths
race, the loser observes the row's decision and becomes a no-op (no second event, no second execution).

---

## 10. Caps and budgets (B5 dependency; fixed here for suspension correctness)

1. Replace the two `caps.check_calls(1)` literals (`execution.rs:505,597`) with a persisted per-run counter:
   `runs.calls += 1` before each provider send, then `caps.check_calls(run.calls)`; refusal `E_CALL_LIMIT` at 41.
   (Current behavior **never enforces MAX_CALLS** — verified: `check_calls` is a pure comparison and the only call sites
   pass the literal `1`; this is a defect, not a design choice.)
2. New caps in `Caps`: `max_tool_turns = 4`, `max_tool_calls_per_turn = 4`, `max_tool_calls_per_run = 12`,
   `max_approval_waits_per_run = 8`; persisted counters `runs.turn`, `runs.tool_calls`. Refusal for turn/tool caps:
   `E_TOOL_LOOP_LIMIT` (new code in the D8 patch) or `E_CALL_LIMIT` if the Lead reuses the existing code; decision:
   add `E_TOOL_LOOP_LIMIT` (one code, clear how_to_fix) — recorded as part of the contract patch.
3. Approval wait time counts against TTL only; suspension must not evict `RunBudget` (D10) and must not reset counters
   (they are persisted in `runs`). On restart counters die with the interrupted run (truthful: the run is terminal).
4. Wall clock: each wait ≤ 120 s; ≤ 8 waits ⇒ bounded by 16 min worst case; no separate run wall cap in 1.2 (product
   note D-ALT-WALLCLOCK).

---

## 11. Tests (named; commands; offline/deterministic unless stated)

Backend (`cd backend`; PATH = `%USERPROFILE%\.cargo\bin;<repo>\tools\w64devkit\bin;` + existing PATH):
1. **`cargo test -p hiverouter-core --test suspension`** (new):
   `approval_suspends_run_and_approve_resumes_executes_once`,
   `deny_returns_denied_tool_result_and_run_continues`,
   `timeout_fails_run_e_approval_timeout_without_side_effect`,
   `cancel_while_awaiting_consumes_approval_and_cancels_run`,
   `kill_worker_while_awaiting_cancels_run_without_side_effect`,
   `concurrent_resolve_has_exactly_one_winner_and_no_second_execution`,
   `replayed_nonce_is_refused_before_any_side_effect`,
   `second_prompt_while_awaiting_is_refused_e_run_active`,
   `suspension_keeps_budget_and_increments_calls_counter`,
   `approval_request_event_carries_nonce_and_args_sha256` (after D8 patch),
   `restart_marks_awaiting_run_interrupted_and_expires_pending`.
   Harness: seed approval rows through the real `ApprovalGate` (as `ws_transport.rs:306-327` does), a
   `MockExecutor` that returns one side-effect tool call then a final answer, and the `expire`/clock seam (D12).
2. **`cargo test -p hiverouter-tools --lib approval::tests`** — extend: `expire_pending_is_a_cas`,
   `cancelled_decision_is_not_resolvable`, `unique_pending_per_tool_call`; existing replay/TTL/args tests must stay green.
3. **`cargo test -p hiverouter-core --tests`** — migration v4 up/down roundtrip, chat_store meta_json/turn round-trip,
   http_boot/ws_transport/recovery regression (`recovery_gates` must add: awaiting run → interrupted + expired approval).
4. **`cargo test -p hiverouter-contract`** — golden/matrix/roundtrip for the extended `APPROVAL_REQUEST` + new
   `TOOL_CALL`/`TOOL_RESULT`; event count 23; refusal count 66 (with `E_TOOL_LOOP_LIMIT`); UI parity list.
5. **`cargo test -p hiverouter-core --test ws_transport`** — existing `approval_resolve_command_acks_and_emits_resolved_event`
   now also asserts the engine waiter wake and that a replay after a terminal run is `E_APPROVAL_REPLAY`; add
   `approval_request_frame_is_published_when_run_suspends` (producer exists).
UI (`cd web`; `npm.cmd run typecheck; npm.cmd test -- --run`):
6. `store.test.ts` — extended `APPROVAL_REQUEST` fills nonce/args_sha256/expires_at; approve/deny send one
   `APPROVAL_RESOLVE`; terminal `RUN_STATUS` clears a pending overlay (timeout/cancel); no optimistic clear before ACK.
7. `ApprovalOverlay.test.tsx` (new) — no-nonce gap still renders disabled buttons; with nonce: tool, args hash, TTL
   countdown, approve/deny enabled; replay refusal toasts by code.
E2E (`cd e2e`; fake provider per W10 B7):
8. `approval-flow.spec.ts` (new) — suspend card appears with nonce/hash; deny path → tool result denied and run
   completes; approve path → exactly one file write in temp workspace; TTL path (short test TTL) → failed
   `E_APPROVAL_TIMEOUT`; cancel path; reload during wait → interrupted/expired, never auto-approved.
9. Gate logs record HEAD + command + exit code for every claim (S7 finding).

All tests deterministic/offline; no paid provider call. A live provider smoke is optional, operator-key-gated, and not
a release gate.

---

## 12. Skill application (evidence)

- **forgewright-recon**: ledger §1 with bytes/lines/sha256 and anchors; callers traced (`ApprovalGate::request` has only
  test callers; ws resolve arm; recovery; caps call sites), tests located by name.
- **forgewright-lead**: read code/callers/tests before any hypothesis; root cause and evidence recorded before the
  design; every claim labelled VERIFIED/READ/INFERRED/UNVERIFIED; no fake implementation; no commit (Lead owns it).
- **logic-gatekeeper**: Phase 1 JSON in §2 written before the (document-only) mutation; Phase 2 entry-check statement
  included; no mutation performed.
- **integration-logic**: full chain provider turn → engine gate → approvals table → WS command → waiter → tool execution
  → persistence traced; refusal-boundary matrix in §9; every unconfigured/unavailable path named.
- **logic-debugger**: failure/edge paths enumerated in §2 edgeCases and §9 (races, TTL vs resolve, cancel/kill, restart,
  replay, tampered args); each maps to a named test in §11.

---

## 13. Exact blockers (what stops implementation today)

| ID | Blocker | Evidence | Required resolution (owner) | Acceptance |
|---|---|---|---|---|
| **W11-B1** | Frozen wire lacks nonce and approval context; `APPROVAL_REQUEST` is `{approval_id,tool,decision?}` only | `event.rs:242-252`; UI nonce gap `ApprovalOverlay.tsx:29-35`, `store.ts:583-590`, `contract.ts:227` | Contract patch (CONTRACT + Lead): payload extension + `TOOL_CALL`/`TOOL_RESULT` (+optional `E_TOOL_LOOP_LIMIT`), goldens/matrix/CHANGE-LEDGER/UI decoders | `cargo test -p hiverouter-contract` green with new rows; UI parity test |
| **W11-B2** | No suspension/resume seam, no `awaiting_approval` producer, no waiter | `execution.rs:455-574` is linear; `state.rs:38` has the state but grep finds no transition; `tools_bridge` has resolve but no request producer | Core (engine): waiter registry, suspending dispatch, resume path, terminal mapping; UI clears overlay on terminal | §11.1 tests green |
| **W11-B3** | Conversation persistence cannot store tool turns; run counters absent; approvals DDL outside the migration ledger | `db.rs` v1-v3 only; `chat_messages` 74-82; `approval.rs:141-165`; `tools_bridge.rs:73-77` | Core migration v4 (§7) + store read/write APIs + migration roundtrip test | §11.3 green; up/down roundtrip |
| **W11-B4** | No tool-call assembly/execution seam into the run path (W10 B4); approval gate cannot be reached by a model | `provider_executor.rs:171-173`; W10 §2.1-2.4 | Provider+core loop wave (W10 B4); suspension consumes its `TurnOutcome` only after that lands | wiremock multi-turn + §11.1 harness |
| **W11-B5** | Caps are inert: `check_calls(1)` literal, no counter; no turn/tool caps | `execution.rs:505,597`; `caps.rs:15,66-71` | Core: persist counters (§7) + check before send/execute; add `E_TOOL_LOOP_LIMIT` in B1 patch | cap refusal tests |
| **W11-B6** | Approval UX/security undefined for model-originated side effects: no producer → no card in production; args hash/TTL/countdown not on wire; session grant not exposed | `ApprovalOverlay.tsx`; `store.ts:573-597`; `event.rs:242-252`; `tool.rs:55-57` | UI+Lead: args_sha256/TTL display, deny/timeout/cancel copy, threat review (shell/file-write/spawn), read-only phase A gating policy | UI tests §11.6-7; e2e §11.8 |
| **W11-B7** | Recovery cannot distinguish an interrupted approval wait; no pending-approval expiry at boot | `recovery.rs:60-68` (Interrupted, no error_code; no approvals step) | Core+tools: recovery calls `expire_pending(now)`, sets `error_code=E_APPROVAL_TIMEOUT` for awaiting runs, report field | recovery test |
| **W11-B8** | No deterministic suspension/approval harness (fake provider + seeded approval + TTL clock) in core tests | `ws_transport.rs:306-327` seeds via gate (usable seam); W10 B7 says harness does not exist | Core test infra: mock executor/tool + clock seam (D12); wiremock provider after B4 | §11.1 executes offline |

**What can start before the contract patch (staged 1.2 implementation plan):**
- **S1 (no wire change):** migration v4 + counters/persistence + `ApprovalGate::expire_pending` + recovery expiry/error_code
  + waiter registry + suspension/timeout/cancel/kill/restart engine paths with gate-seeded approvals; tests §11.1-3.
- **S2 (needs W11-B1):** `APPROVAL_REQUEST` extension + `TOOL_CALL`/`TOOL_RESULT`; ws arm wakes the engine; producer
  publishes the request; contract goldens/matrix/UI decoders.
- **S3 (needs W11-B4):** real model tool-call turns; approval e2e with fake provider; phase A read-only tools.
- S1 and S2 are separable; S3 is W10's loop wave.

**Product calls surfaced (not decided by this seat):** D-ALT-TIMEOUT (fail vs implicit-deny on TTL — default fail,
§D6), D-ALT-WALLCLOCK (add a total run wall cap?), wire `allow_session` (currently internal), and whether approval
happens per call or batched per turn (default per call; batch would weaken the args binding).

---

## 14. Done / Not done / Unverified

**Done (design deliverable):** recon ledger with hashes (§1); Phase 1 gatekeeper JSON (§2); 12 explicit decisions with
rejected alternatives (§3); legal state-transition table (§4); suspension protocol and ownership (§5); exact additive
wire payloads (§6); exact migration v4 DDL + down policy + replay rules (§7); nonce/security model (§8); deny/timeout/
cancel/kill/restart matrix with races (§9); caps/budget corrections including the verified inert-cap defect (§10);
named tests + commands (§11); skill evidence (§12); exact blocker register and stageable S1/S2/S3 plan (§13).

**Not done (by design):** no product code, tests, configs, migrations, contract or docs edits; no commit; no builds or
test runs (this seat is design-only); no Tauri/loop implementation; no repo-wide reformat.

**Unverified (needs execution to become fact):** all §3-§11 behavior is a **design**, not implemented behavior — the
current shipped 1.1.0 has no suspension/approval flow; exact provider behavior for a resumed tool loop under thinking
mode (reasoning_content pass-back) is UNVERIFIED (official docs say 400 without pass-back; not exercised live);
double-execution safety relies on engine+gate being wired exactly as §5 (not yet implemented); `SQLite DROP COLUMN`
support on the bundled rusqlite version is inferred from table-rebuild precedent (`db.rs` migration style) and must be
proven by the v4 roundtrip test; restart-during-wait UX copy (`E_APPROVAL_TIMEOUT` on an `interrupted` run) is a product
decision (documented, not settled); no paid/live provider run was performed.

---

## 15. References for the implementing wave
W10 (tool-loop staging, B1-B8): `plans/waves/W10-TOOL-LOOP-DECISION.md`. Contract: `docs/contracts/UI-COMMANDS-v1.md`
(14 commands / 21 events / 65 refusals — counts change only via the D8 patch + `CHANGE-LEDGER-IMPL.md` row). Ledger
format: `backend/crates/hiverouter-tools/src/audit.rs`. UI readiness: `web/src/panels/ApprovalOverlay.tsx`,
`web/src/state/store.ts`, `web/src/api/contract.ts`. Gate: `backend/crates/hiverouter-tools/src/approval.rs`.
Recovery: `backend/crates/hiverouter-core/src/recovery.rs`. Caps: `backend/crates/hiverouter-core/src/caps.rs`.
