# RUN-2 — PROOF CODE: WP-04 suspension matrix + WP-07 live tool-turn

**Planner:** RUN 2 PLANNER 4 · **Mode:** READ-ONLY (only this file written) · **Date:** 2026-11-06
**Target:** `plans/foresight/COMPILED-LEDGER.md` **WP-04** (deterministic suspension/loop matrix) and the ledger-table **WP-07** (live tool-turn + live approval proof, item 13), after the **Run-1 tree**.
**Skills applied:** `/forgewright-recon` → `/forgewright-lead` → `/logic-gatekeeper` → `/integration-logic` → `/logic-debugger` (planning only; no Phase-2 mutation).
**No code, no builds, no tests, no key access, no paid calls were performed.**

> **Naming warning (ledger inconsistency):** `COMPILED-LEDGER.md:39` defines WP-07 as *“Live tool-turn + live approval proof (item 13)”*; `COMPILED-LEDGER.md:106` heads WP-07 as *“Args preview (item 11; optional, contract-gated)”*. This plan targets the **live tool-turn/approval proof** per the task and the WP table; args preview stays deferred (`COMPILED-LEDGER.md:114,245`, C3). The implementation wave must not renumber the ledger; record this reconciliation in its receipt.

---

## 0. Verified current state (Run-1 tree, read-only)

| # | Fact | Evidence |
|---:|---|---|
| S1 | HEAD `90cb7a7` (“feat(1.2): reasoning passback, approval boot expiry, recovery timeout; foresight run-1 plans/ledger”); tree dirty: `core/src/tools_bridge.rs`, `tools/src/{approval.rs,lib.rs}` + untracked `tools/qa/apply_wp01.py` (WP-01 lane in flight) | `git log --oneline -3`, `git status --short` (this read) |
| S2 | Run-1 baseline gate: **396 passed / 0 failed** workspace; web **147/147**; mocked browser **16 passed** | `plans/foresight/RUN-1.md:21-23` |
| S3 | Current core test files: `tool_loop_engine.rs` **11 tests**, `run_lifecycle.rs` **7**, `approval_ledger.rs` **2**, `ws_transport.rs` **13**, `recovery_gates.rs` **3**; **no `suspension.rs`, no `tool_loop_matrix.rs`** | file list + `#[tokio::test]/#[test]` counts (this read) |
| S4 | Run-1 read-only tool loop exists: `execute_tool_turn`/`execute_tool_plan` publish `TOOL_CALL`, execute read-only, publish `TOOL_RESULT`, persist `tool_turns`; non-read tools are refused `E_TOOL_NOT_ALLOWED` | `backend/crates/hiverouter-core/src/execution.rs:684-800,879-920` (read; C-ENG-01) |
| S5 | Run-1 tree has **no suspension engine**: `ApprovalWaiters`, `execute_side_effect`, `execute_approved`, `write_atomic`, `notify_approval_resolved` = **0 occurrences**; approval producer still has **no production caller** | symbol grep (this read); `COMPILED-LEDGER.md:23,202-203` |
| S6 | WS `APPROVAL_RESOLVE` currently resolves + ACKs/casts `APPROVAL_RESOLVED` but does **not** wake an engine; pending-approval replay is **not** wired into hello | `core/src/ws.rs:465-483`; `pending_approval_events` callers = tests only (C-UX-03) |
| S7 | Live harness gate today: `paidGate()` requires only `LIVE_ONE_PROMPT=1` + `DEEPSEEK_API_KEY` + binary; **no `LIVE_TOOL_TURN` gate**; `StartLiveServerOptions` = `{withProviderKey?, bootTimeoutMs?}`; no `extraEnv`/`seedWorkspace`; `openWsProbe` exists | `e2e/helpers/live-server.ts:115-137,312-316,322-425,459` (read) |
| S8 | Mocked specs exist and pass on Run-1 tree: `tool-loop.spec.ts` 8 tests, `approval-flow.spec.ts` 8 tests; selectors: `nav[aria-label="Primary navigation"]` → Agents, `[data-testid="agents-tool-activity"]`, `section[aria-label="Tool activity"]`, `ul[aria-label="Tool calls"] li[data-outcome]`, `#composer`, `role=alert`, HUD `role=status` | spec files (read); `AgentsPanel.tsx:59-61`, `ToolActivityPanel.tsx:138,145` |
| S9 | Contract is frozen at **14 commands / 23 events / 65 refusals**; no new event/command/refusal is allowed for this wave; approval fields are 1.2-additive only | `docs/contracts/UI-COMMANDS-v1.md:37,54-65`; `COMPILED-LEDGER.md:248` non-claim 11 |
| S10 | Run-1 ledger contradictions C4–C8 are **unresolved** and directly affect these tests (wrong-args code, timeout status, cancel result count, restart semantics, test counts 9/11/12/14) | `COMPILED-LEDGER.md:263-267` |
| S11 | The required runtime WPs are prerequisites: WP-01 (gate primitives), WP-02 (side-effect executor), WP-03 (engine suspension); WP-04 depends WP-02+WP-03; WP-07 depends WP-04 (+WP-06 for replay) | `COMPILED-LEDGER.md:33-39,76-105` |
| S12 | Sealed release record remains **1.1.0** (`tools/release-manifest.json` v1, `source_commit=tag_target=87079ce`); tree declares 1.2.0-dev; no 1.2 release binary is guaranteed current | `COMPILED-LEDGER.md:6`; `CODE-PROOF.md:4` |

**Baseline rule:** all counts below are **expected** and must be re-quoted from the implementing wave’s commands; no count in this file is a claim of execution.

---

## 1. WP-04 — deterministic suspension + loop matrix (no paid path)

### 1.1 New/extended files (exact)

| Action | Path |
|---|---|
| **NEW** | `backend/crates/hiverouter-core/tests/suspension.rs` |
| **NEW** | `backend/crates/hiverouter-core/tests/tool_loop_matrix.rs` |
| **EXTEND** | `backend/crates/hiverouter-core/tests/approval_ledger.rs` (one new test; 2 → 3) |
| **KEEP** | `backend/crates/hiverouter-core/tests/tool_loop_engine.rs` (11 tests — **freeze count, do not rename/extend**, C8) |
| **RECONCILE** (only if required by D2 frame additions) | `core/tests/run_lifecycle.rs` (7), `core/tests/ws_transport.rs` (13); never delete assertions; count stays unless additively justified |
| Evidence logs (implementation wave writes) | `docs/evidence/1.2-wp04-suspension.log`, `docs/evidence/1.2-wp04-matrix.log` |

### 1.2 Fixture for `suspension.rs` (exact, no network)

```rust
// per R1 §11 harness
let temp = tempfile::tempdir()?;
let db = Arc::new(Database::open(&temp.path().join("hiverouter.db"))?.migrate()?);
let bus = Arc::new(EventBus::new(...));                       // subscribe BEFORE submit
let bridge = ToolsBridge::new(...)                            // ToolsBridge::migrate(&db)
    .with_approval_ttl(1);                                    // D4 test seam (1 s TTL)
let jail = WorkspaceJail::open(temp.path().join("workspace"))?;
let engine = RunEngine::new(db, bus.clone(), GraphAuthority::new(), executor, TemplateRegistry::default())
    .with_tool_workspace(jail)
    .with_tools(bridge.clone())
    .with_shell_policy(ShellPolicy::new(["cmd.exe".to_owned()]))
    .with_clock(fixed_clock_or_default);
```
- `MockExecutor` (or `ScriptedExecutor`) returns one side-effect `AssembledToolCall` (`file_write` with `{"path":"approval-proof.txt","content":"HIVE-APPROVAL-OK"}`) then a final answer; follow-ups are scripted, never real provider.
- Approve helper: read `APPROVAL_REQUEST` from bus → `(approval_id, nonce)`; call `bridge.resolve_approval(&ApprovalResolve { approval_id, nonce, decision: "approve", reason: None })`; call `engine.notify_approval_resolved(&resolution)`.
- Assertions read: bus frames, `tool_turns` rows, filesystem, `waiters.len()/is_empty()`, DB `approvals.decision`.
- **No network, no key, no `LIVE_*` env, no paid call.**

### 1.3 `suspension.rs` test cases (14; names are the contract / claim IDs)

| # | `#[tokio::test] async fn` name | Steps / expected output | Falsifies |
|---:|---|---|---|
| S1 | `suspend_publishes_awaiting_approval_and_approval_request` | run `RUN_STATUS awaiting_approval`; task `NODE_STATE awaiting_approval`; exactly one `TOOL_CALL`; exactly one `APPROVAL_REQUEST` with nonce/run/task/tool_call/args_sha256/expires_at; no file; `waiters.len()==1` | suspension event seam |
| S2 | `approve_resumes_and_executes_the_file_write_exactly_once` | one `TOOL_RESULT ok`; one `tool_turns` row; file content exact; `RUN_STATUS running` then final `completed`; second `notify` replay → no second execution (`E_APPROVAL_REPLAY` at gate), `waiters.is_empty()` | one side effect |
| S3 | `deny_returns_denied_result_and_run_continues` | one `TOOL_RESULT denied` with `code=None`; no file; follow-up final reply persisted; run/task completed | deny safety |
| S4 | `timeout_fails_run_with_e_approval_timeout_and_no_side_effect` | wait past TTL (engine timeout path, **not boot reaper**); run status **failed** + `E_APPROVAL_TIMEOUT`; exactly one terminal `RUN_STATUS`; row decision `expired`; no `APPROVAL_RESOLVED`; no file; one `TOOL_RESULT timeout` | TTL fail-closed |
| S5 | `cancel_while_awaiting_cancels_run_and_consumes_approval` | `CANCEL_RUN`; run `cancelled`; row `decision='cancelled'`; **exactly one `TOOL_RESULT cancelled`** (C6/R1 D13 — *not* zero); later resolve → `E_APPROVAL_REPLAY`; no file; single terminal frame | cancel safety |
| S6 | `kill_worker_while_awaiting_cancels_without_side_effect` | awaiting-task kill ACKs; run `cancelled`; one cancelled result; no write; waiter gone | kill safety |
| S7 | `timeout_cas_loss_to_resolve_still_resumes` | resolve wins CAS just before timer; delayed wake; engine re-reads decided row → executes exactly once; no timeout failure | CAS race correctness |
| S8 | `concurrent_resolve_has_one_winner_and_no_second_execution` | two `resolve_approval`: one Ok, one `E_APPROVAL_REPLAY`; one execution; second notify false | one decision/one execution |
| S9 | `second_prompt_while_awaiting_is_refused_e_run_active` | `submit_prompt` → `E_RUN_ACTIVE` while one run suspended; no second run | active-run guard |
| S10 | `restart_marks_awaiting_run_interrupted_and_expires_pending` | seed awaiting run + future-TTL pending; `recover_on_boot` then build services; run **interrupted** with `error_code=E_APPROVAL_TIMEOUT` (C5/C7: **not** `failed`, **no crash-resume**); pending row `decision='expired'` even before TTL; later resolve → `E_APPROVAL_TIMEOUT`; no execution | restart truth |
| S11 | `wait_cap_refuses_the_ninth_request` | `MAX_APPROVAL_WAITS_PER_RUN=8`; 9th request → `TOOL_RESULT refused E_CALL_LIMIT`; no new approval row | bounded waits |
| S12 | `wrong_args_grant_refused_before_side_effect` | approve args A, resume plan with args B; refusal is **`E_APPROVAL_REQUIRED`** (C4 — **not** `E_APPROVAL_DECISION_INVALID`); no write; run failed by name; one refused result | args binding |
| S13 | `shell_exec_runs_only_the_allowlisted_program_after_approval` | allowlist `cmd.exe`; approved call captures stdout; non-allowlisted program refused `E_TOOL_NOT_ALLOWED`, never spawns | shell allowlist |
| S14 | `no_bridge_engine_keeps_the_read_only_refusal` | engine built without `.with_tools`; `file_write` call → `E_TOOL_NOT_ALLOWED`; Run-1 read-only behavior preserved | no accidental privilege |

### 1.4 `tool_loop_matrix.rs` (5 tests; wiremock SSE, no paid)

Fixture: wiremock server returning SSE tool-call frames (pattern `core/tests/provider_streaming.rs`); real `RunEngine` with test jail; no key.
`tool_loop_matrix_read_only_call_roundtrip`, `tool_loop_matrix_turn_cap_refusal`, `tool_loop_matrix_call_cap_refusal`, `tool_loop_matrix_invalid_arguments_refused`, `tool_loop_matrix_non_read_tool_refused_by_name` → expected **5 passed**, 0 failed.

### 1.5 Mocked browser ordering additions (existing `e2e/specs/approval-flow.spec.ts`)

| New test name | Fixture | Expected | Falsifies |
|---|---|---|---|
| `'suspension order: APPROVAL_RESOLVED -> TOOL_RESULT -> REPLY_DELTA -> RUN_STATUS completed'` | `installMockWs`, `sendApprovalRequest`, harness frame helpers | event order in captured received frames; one `li[data-outcome]` terminal row; final `RUN_STATUS completed` rendered | no ordering regression |
| `'no TOOL_RESULT may precede APPROVAL_RESOLVED for an approval-gated call'` | same; emit `TOOL_RESULT` first | zero `li[data-outcome]` before `APPROVAL_RESOLVED`; after resolution buffered replay/one row (Wave-3 policy) | gated call cannot render executed before approval |

Expected: `approval-flow.spec.ts` 8 → **10 tests**, all pass; `tool-loop.spec.ts` stays 8.

### 1.6 WP-04 commands + expected outputs (run before any live work)

```powershell
cd backend
$env:PATH = "$env:USERPROFILE\.cargo\bin;$PWD\..\tools\w64devkit\bin;$env:PATH"
cargo test -p hiverouter-core --test suspension --locked
cargo test -p hiverouter-core --test tool_loop_matrix --locked
cargo test -p hiverouter-core --test approval_ledger --locked
cargo test -p hiverouter-core --test tool_loop_engine --locked
cargo test -p hiverouter-core --test run_lifecycle --locked
cargo test -p hiverouter-core --test ws_transport --locked
cargo test -p hiverouter-core --test recovery_gates --locked
cargo test -p hiverouter-tools --lib --locked
cargo test --workspace --locked
```
| Command | Expected output (quote exact) |
|---|---|
| suspension | `test result: ok. 14 passed; 0 failed; 0 ignored` |
| tool_loop_matrix | `test result: ok. 5 passed; 0 failed` |
| approval_ledger | `test result: ok. 3 passed; 0 failed` (2 existing + ledger-on-resume) |
| tool_loop_engine | `test result: ok. 11 passed; 0 failed` (freeze; do not claim 9/12) |
| run_lifecycle | `test result: ok. 7 passed; 0 failed` (or +strict-frames additive only) |
| ws_transport | `test result: ok. 13 passed; 0 failed` |
| recovery_gates | `test result: ok. 3 passed; 0 failed` |
| tools lib | record count (`approval::tests` + `executor::tests` + new primitives) |
| workspace | `test result: ok. N passed; 0 failed; 2 ignored`; floor **≥416** (Run-1 396 + 14 + 5 + 1); expected ≥429 once WP-01/02 unit tests land; record exact N |

---

## 2. WP-07 — live tool-turn + live approval proof (PAID-GATED)

### 2.1 Capability truth after Run-1 tree (what this proof can and cannot reach)

| Capability | Status now | Requirement |
|---|---|---|
| Live **read-only** tool turn (`file_read` → `TOOL_RESULT ok` → final reply) | **available in source** (Run-1 loop; `execution.rs:684-800`) but needs a 1.2-dev release binary built from the Run-2 tree and a paid provider run | WP-04 deterministic green + Lead rebuild + gate §2.2 |
| Live **approval/side-effect** turn (`file_write` suspend → approve → execute once) | **not reachable on Run-1 tree**: no suspension engine, no side-effect executor, no production approval producer/notify | WP-01/02/03 land, C2 default-off/always-wired decision, binary rebuild |
| Live **deny** (no side effect) | same as approval | WP-01/02/03 |
| Live **TTL timeout** during suspension | not worth a live prompt; deterministic S4 covers it; live waiting 120 s per prompt is wasteful/flaky | out of live scope; deterministic only |
| Live **replay after reconnect** | WP-05/06; out of this proof scope | later wave |

### 2.2 Harness extension (exact file/symbols; new code needed before specs compile)

**File:** `e2e/helpers/live-server.ts` (extend; preserve all existing exports/behavior).

```ts
export type LiveGateMode = 'prompt' | 'tool-turn';
export function paidGate(mode: LiveGateMode = 'prompt'): LiveGate;
// backward-compatible default keeps LIVE_ONE_PROMPT semantics for streaming/replay.
// 'tool-turn' requires LIVE_TOOL_TURN=1 + DEEPSEEK_API_KEY + binary.
export interface StartLiveServerOptions {
  readonly withProviderKey?: boolean;
  readonly bootTimeoutMs?: number;
  readonly env?: Readonly<Record<string,string>>;        // NEW: explicit child extras (e.g. HIVEROUTER_SIDE_EFFECTS=1)
  readonly seedWorkspace?: Readonly<Record<string,string>>; // NEW: files written into temp workspace
}
export interface LiveServer {
  // existing fields stay; ADD:
  readonly workspaceRoot: string; // exact path for file-effect assertions
}
export function openWsProbe(page: Page, url: string): Promise<WsProbe>; // unchanged
```

**Gating env vars (exact):**

| Env | Required for | Behaviour when missing |
|---|---|---|
| `LIVE_TOOL_TURN=1` | any live tool-turn/approval paid test (**independent** of `LIVE_ONE_PROMPT`) | named `test.skip` reason `LIVE_TOOL_TURN is not 1: paid tool-turn proof disabled…`; **zero** server start / submit / provider spend if checked first |
| `DEEPSEEK_API_KEY` (process env only, never printed/written) | all paid tests | named skip reason `DEEPSEEK_API_KEY is not set: paid tool-turn proof refuses to fake a pass…` |
| `LIVE_ONE_PROMPT=1` | existing streaming/replay specs only | not sufficient for tool-turn; keep separate |
| `E2E_SERVER_BIN` | release/debug binary path; default discovery unchanged | binary-missing named skip |
| `HIVEROUTER_SIDE_EFFECTS=1` (only if C2 chooses default-off) | live approval side-effect tests | live approval tests skip with `SIDE_EFFECTS_NOT_ENABLED…` (never fake) |
| `E2E_APP_READY` / `E2E_BASE_URL` | legacy external-server specs only; self-host harness needs neither | unchanged |

**Paid-call warning (must be repeated in the spec header and receipt):** each paid test sends **one** real prompt to DeepSeek with an operator key; this is real spend with unknown usage; the operator must set a budget cap, be present, keep the key only in process env, and stop on first failure. Gate-negative tests must never start a server or call the provider.

### 2.3 New file `e2e/specs/live-tool-turn.spec.ts` (read-only tool turn; 2 tests, 1 paid)

Fixture: `startLiveServer({ withProviderKey: gate.enabled, seedWorkspace: { 'README.md': 'HIVE-TOOL-OK\n' } })`; `page.goto(live.baseURL)`; wait `Connected`; `openWsProbe(page, live.wsUrl())`; selectors from S8; `#composer` submit; capture WS frames; write `docs/evidence/1.2-wp07-live-tool-turn.json` (redacted: frames, run id, binary sha, gate env names, prompt count=1) + `.log`.

| # | Test name | Gate | Steps / expected output | Falsifies |
|---:|---|---|---|---|
| T1 | `'live gate: LIVE_TOOL_TURN missing => named skip, no server start, no paid call'` | none (runs now) | call `paidGate('tool-turn')` in-process; assert `enabled === false` and reason contains `LIVE_TOOL_TURN`; no `page.goto`, no `startLiveServer`; expected **passed** | gate leak/spend |
| T2 | `'live read-only tool turn: TOOL_CALL -> TOOL_RESULT ok -> RUN_STATUS completed (paid)'` | `LIVE_TOOL_TURN=1` + key | submit `"Read README.md with the file_read tool, then reply exactly HIVE-TOOL-OK"`; expect WS received order `TOOL_CALL` < `TOOL_RESULT ok` < terminal `RUN_STATUS completed`; UI `ul[aria-label="Tool calls"] li[data-outcome="ok"]` exactly 1; `README.md` body token echoed once (no duplicate deltas); composer enabled after `REPLY_DONE`; evidence JSON written | live tool loop + no duplicate stream |

### 2.4 New file `e2e/specs/live-tool-approval.spec.ts` (side-effect approval; 3 tests, 2 paid, blocked until WP-02/03 + C2 decision)

Fixture: same harness, `env` includes `HIVEROUTER_SIDE_EFFECTS=1` only when the C2 decision requires it; `seedWorkspace` empty except `README.md`; prompt asks for `file_write` of `approval-proof.txt` containing `HIVE-APPROVAL-OK`.
Selectors: approval `role=dialog`; buttons `Approve`/`Deny` (accessible names from `STRINGS.controls`); tool activity rows; refusal toast `role=alert`.

| # | Test name | Gate | Steps / expected output | Falsifies |
|---:|---|---|---|---|
| A1 | `'approval live gate: missing LIVE_TOOL_TURN / side-effects flag => named skip, no paid call'` | none (runs now) | assert `paidGate('tool-turn')` reason and (if C2 default-off) activation reason; expected passed, no server start | gate/flag leak |
| A2 | `'live approval: approve executes file_write exactly once (paid)'` | gate + key + side-effects enabled | submit prompt; expect exactly one approval card; **no** `TOOL_RESULT` before click; click Approve; expect one `APPROVAL_RESOLVE` command (same id/nonce), `APPROVAL_RESOLVED`, one `TOOL_RESULT ok`, file exists with exact content once, run terminal completed; one evidence JSON | approval authority + one effect |
| A3 | `'live approval: deny leaves no file (paid)'` | same | submit prompt; click Deny; expect `TOOL_RESULT denied`, file absent, run continues/terminal per product, one approval resolution | deny safety |

If `HIVEROUTER_SIDE_EFFECTS` is not enabled/wired, A2/A3 must be reported **skipped with the exact named reason** and **not** counted as pass; only WP-02/03 landing + C2 decision unblocks them.

### 2.5 Exact expected outputs for live runs

```powershell
cd e2e
$env:E2E_SERVER_BIN = '<repo>\target\release\hiverouter.exe'   # rebuilt from Run-2 source; record sha256
$env:LIVE_TOOL_TURN = '1'
$env:DEEPSEEK_API_KEY = '<process env only, never written>'
# optional if C2 requires: $env:HIVEROUTER_SIDE_EFFECTS = '1'
.\node_modules\.bin\playwright.cmd test specs/live-tool-turn.spec.ts specs/live-tool-approval.spec.ts --project=chrome --reporter=list
```
- All gates enabled + code landed: expected **5 passed** (T1/T2/A1/A2/A3; 3 paid prompts), 0 failed, 0 skipped, exit 0.
- Without gate/key: expected T1/A1 pass, T2/A2/A3 **skipped** with exact reasons; `node run-guarded.mjs --project=chrome specs/live-tool-turn.spec.ts specs/live-tool-approval.spec.ts` → **exit 1** (`[skip-guard] …`), proving no paid call/skip-as-green.
- Approval blocked by C2: expected **3 passed, 2 skipped** (A2/A3 named `SIDE_EFFECTS_NOT_ENABLED`); guard exit 0 because passes exist — receipt must say paid approval proof **not executed**.
- Each paid test writes redacted evidence; zero key material in files/console; verify `Select-String 'sk-[A-Za-z0-9]{16,}'` = 0 over new evidence before receipt.

---

## 3. Minimum deterministic suite before ANY live call (ordered; all no-paid)

| Step | Command | Expected output / gate |
|---|---|---|
| D0 | `git rev-parse HEAD; git status --porcelain` after WP-01/02/03 merge/commit | HEAD recorded; clean tree (proof quiescent); record report hashes |
| D1 | suspension/matrix/ledger/tool_loop_engine/run_lifecycle/ws_transport/recovery_gates tests (commands in §1.6) | all `0 failed`; suspension **14**, matrix **5**, ledger **3**, tool_loop_engine **11**, run_lifecycle **7**, ws_transport **13**, recovery_gates **3** |
| D2 | `cargo test -p hiverouter-tools --lib --locked; cargo test --workspace --locked` | 0 failed; record N ≥416; 2 ignored baseline |
| D3 | `cd web; npm.cmd run typecheck; npm.cmd test -- --run; npm.cmd run build` | exit 0; all tests pass (Run-1 baseline 147/147; record new count); build green |
| D4 | `cd e2e; node run-guarded.mjs --project=chrome specs/tool-loop.spec.ts specs/approval-flow.spec.ts` | expected **18 passed, 0 failed, 0 skipped**, exit 0 (8 tool-loop + 10 approval-flow after §1.5 additions) |
| D5 | `cd e2e; node run-guarded.mjs --project=chrome specs/gui-gates.spec.ts specs/gui-residual.spec.ts specs/a11y-gates.spec.ts` | 0 failed; record pass/fixme counts (exclusions stay named); exit 0 |
| D6 | `node tools/qa/verify-harness.mjs`; `node tools/qa/check-docs-truth.mjs`; `powershell -NoProfile -ExecutionPolicy Bypass -File tools/qa/run-gates.ps1 -DryRun` | harness **27/27**; docs checker **19/19** (Run-1 baseline); `GATES: PASS` exit 0 (no execution) |
| D7 | negative paid gate: run live specs with `LIVE_TOOL_TURN` **unset** | T1/A1 pass; paid tests skipped with exact gate reasons; `run-guarded` exit 1; no server start/prompt (evidence of spend protection) |
| D8 | Lead builds release/debug binary from the clean Run-2 tree; `Get-FileHash` + `--version` | record binary sha256 + version `1.2.0-dev`; embedded web dist current; **no paid** |
| D9 | only now set `LIVE_TOOL_TURN=1`, key, optional side-effects flag; run §2.5 | expected output per §2.5; quote redacted log; do not claim before D1–D8 all green |

**Stop rule:** any failing D1–D7 ⇒ **no live call**; report RED with the failing command. No retry loop inside the paid spec; on first paid failure stop and return the captured frames/refusal.

---

## 4. What can / cannot be claimed after each stage

| Stage | Can claim | Cannot claim |
|---|---|---|
| After D1–D3 | deterministic suspension/tool-loop matrix green (mocked executor) | runtime suspension/live provider behavior |
| After D4–D6 | mocked approval ordering + UI activity + static gates green | live tool turn, paid approval, exactly-once outside tests |
| After D7 | paid gate is fail-closed (no spend without flags) | tool-turn works |
| After D8 | binary hash/version/embedded UI recorded | binary works live |
| After D9 | the specific live cases T2/A2/A3 that passed, with redacted evidence | live timeout; crash-resume; cross-process exactly-once; args preview; trace promotion; strict release gate |

Blockers to record at implementation start: WP-01/02/03 not landed on Run-1 tree (S5/S11); C2 side-effects default-off vs always-wired; C4–C8 semantics (adopt R1 per §1.3); 1.2.0-dev binary rebuild; paid key/budget/operator; no 1.2 release record. If WP-02/03 are absent, live approval A2/A3 must stay `skip` — never fake.

---

## 5. Evidence table (all READ this session; no execution)

| Claim | Pointer |
|---|---|
| Run-1 tree/HEAD/dirty WIP | git commands (this read); `COMPILED-LEDGER.md:6` |
| Run-1 baselines 396/147/16 | `plans/foresight/RUN-1.md:21-23` |
| Current test counts (11/7/2/13/3) and absence of suspension/matrix | file list + `#[tokio::test]/#[test]` grep (this read); `COMPILED-LEDGER.md:77,231` |
| Read-only tool loop today; non-read refusal | `backend/crates/hiverouter-core/src/execution.rs:684-800,879-920`; `COMPILED-LEDGER.md:201` |
| No suspension/side-effect symbols; no approval producer caller | symbol grep = 0; `COMPILED-LEDGER.md:23,202-203` |
| WS resolve without engine notify; no hello replay for approvals | `ws.rs:465-483`; `COMPILED-LEDGER.md:203,218` |
| Live gate only `LIVE_ONE_PROMPT`; no `LIVE_TOOL_TURN`; helper shapes | `e2e/helpers/live-server.ts:115-137,312-316,322-425,459` |
| Selectors (nav, testid, list, composer, alert) | `e2e/specs/tool-loop.spec.ts:80-129`; `AgentsPanel.tsx:59-61`; `ToolActivityPanel.tsx:138,145` |
| WP-04 acceptance / 14 test names / contradictions | `CODE-APPROVAL-ENGINE.md:1237-1254`; `COMPILED-LEDGER.md:76-84,263-267` |
| WP-07 live proof requirements/paid register | `COMPILED-LEDGER.md:39,182,189`; `CODE-PROOF.md:58-85,165-175` |
| Contract frozen 14/23/65; no new events | `docs/contracts/UI-COMMANDS-v1.md:37,54-65`; `COMPILED-LEDGER.md:248` |

**Read-only guarantee:** this plan explicitly does **not** run builds/tests, touch code/config/tests/keys, or place any paid call; only this file was written.
