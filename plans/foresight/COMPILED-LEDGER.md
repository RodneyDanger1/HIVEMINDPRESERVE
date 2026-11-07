# COMPILED LEDGER — 1.2 approval / side-effect / proof closure

**Compiler:** Ledger Compiler (read-only product; this file is the only write).
**Program:** HiveMind 1.2.0-dev → tree declares `1.2.0-dev`; sealed record remains **1.1.0**.
**Skills applied:** `/deep-research` (corpus freeze + source hierarchy), `/forgewright-lead` (read code/callers/tests before any claim; VERIFIED/READ/INFERRED/UNVERIFIED separation), `/evidence-check` (claim → evidence → status table), `/truthseeker` (falsifiable invariants: one decision → one grant/execution, one wait → one wake, one call → one `TOOL_RESULT`, one terminal frame).
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **HEAD `37f0e7c`** (`hivemind-1.1.0-4-g37f0e7c`) · working tree dirty: 8 modified product paths (`core/src/{execution,http,provider_executor,recovery,tools_bridge}.rs`, `core/tests/{run_lifecycle,tool_loop_engine}.rs`, `tools/src/approval.rs`) + untracked `core/tests/lead_bugcheck_boot.rs`; `plans/foresight/` untracked.

## Inputs — frozen corpus (read; no product edit/build/test/paid call by compiler)

| # | Report | Bytes | Lines | SHA-256 (frozen) | Role |
|---|---|---|---|---|---|
| R1 | `plans/foresight/CODE-APPROVAL-ENGINE.md` | 102,091 | 1,351 | `78513F63FBE597CB6605BFFEE605267A714C47233F2C2AB4E4D4FE3B3AE7129E` | code-level engine suspension blueprint (Planner 1; supersedes predecessor plan where marked CORRECTION) |
| R2 | `plans/foresight/CODE-SIDE-EFFECTS.md` | 40,929 | 729 | `0526AADFB193ED940FF33EFC746E79463E671FF3EBCEFF3BB44004DF46676D1B` | granted side-effect executor blueprint (Planner 4; items 3+16) |
| R3 | `plans/foresight/CODE-APPROVAL-UX.md` | 66,041 | 822 | `0DDE4379622BEA0736FC5DF1583B92790F37F3F19B41332CC679F913D6A8D588` | TS/React approval UX blueprint (Planner 2; items 10/11/15) |
| R4 | `plans/foresight/CODE-PROOF.md` | 28,341 | 205 | `2115FC8BD40104C7C6776C751A823159E16A63C6B162866E286F043A9A19C70B` | proof/release plan (Planner 3; suites A–F) |
| S1 | `plans/foresight/APPROVAL-SUSPENSION-PLAN.md` | 58,163 | 631 | `0FACFA27380D310D61DB7E3F260C3EE977B39BBA80E5A166F166EE23A8800D6B` | engine design plan (item assumptions 1/2/5/9; superseded by R1) |
| S2 | `plans/foresight/SIDE-EFFECT-EXECUTOR-PLAN.md` | 39,099 | 443 | `4821D642FA84B916D058CE0D0298F28F584D6986422931E12D3E85F997A85655` | side-effect plan (items 3+16; superseded by R2) |
| S3 | `plans/foresight/APPROVAL-UX-PLAN.md` | 24,091 | 228 | `539933B27E23BC797DF2386FBE88F185B414667B1F0AB4EA1FA6DE3F694F928E` | UX plan (items 10/11/15; superseded by R3) |
| S4 | `plans/foresight/PROOF-RELEASE-PLAN.md` | 33,272 | 198 | `97FF01511E55FBC117FB2FB364A17B718D18CA159BDFAA2C5F0E0CB104B3D6D6` | earlier proof plan; superseded by R4 where they differ |

**Corpus drift warning (material):** `CODE-APPROVAL-ENGINE.md` was rewritten while being read (99,433 B → 100,059 B → 102,091 B; mtime 18:30→18:31). All four reports are **uncommitted working-tree files**; hashes above are the compile-time freeze. Re-run `git status` + `Get-FileHash` before implementation; if a report hash differs, reconcile its §2/§14 before coding.

**Verified repo anchors (current tree; `git show HEAD:` used to separate HEAD from WIP):** `execution.rs` HEAD lines `RunEngine:256`, `execute_tool_turn:680`, `execute_tool_plan:875`, `execute_task:964` — WIP +4 each (`+5` diff); `tools_bridge.rs` `request_model_tool_approval` HEAD `271` → WIP `288` (+17); `ws.rs` `Command::ApprovalResolve:465` unchanged; `approval.rs` `verify_grant` HEAD `653` → WIP `741` (+88 before it; WIP diff +176 lines overall), `request_tool_approval` HEAD `730` → WIP `818`; `tool.rs` `expected_sha256:262`; `jail.rs write:223`; `executor.rs execute_read_only:52`; `shell.rs execute:173` (async). Absent in the tree: `ApprovalWaiters`, `execute_side_effect`, `write_atomic`, `ApprovalBroker`, `args_preview`, `verify_tool_grant`, `expire_approval`, `cancel_approval`, `notify_approval_resolved`, `side_effects.rs`, `approval_wait.rs`, `preview.rs`, `suspension.rs`, `tool_loop_matrix.rs`, live tool/security/resync/trace-capture specs, `approval.ts`/ApprovalCard/StatusStrip/ArgsPreview, `check-seal-topology.mjs` (all occurrence greps = 0; file checks `exists=False`).

---

## 1. Ranked work-package ledger

**Ranking rule:** dependency order first (compile blockers → proof → live/UI → closure/ops), then risk. `P0` = blocks everything below; `P1` = release-proof; `P2` = closure/optional. Each WP is a **single-writer** file set.

| Rank | WP | Scope | P | Depends on | Current viability |
|---:|---|---|---|---|---|
| 1 | WP-01 | Approval gate primitives + approvals-table integrity | P0 | WIP merge (boot expiry/recovery) | READY after WIP commit |
| 2 | WP-02 | Granted side-effect executor (items 3+16) | P0 | WP-01 | DESIGN READY (design conflict C1) |
| 3 | WP-03 | Engine suspension/resume/wait/event wiring (items 1,2,5,9) | P0 | WP-01, WP-02 | DESIGN READY |
| 4 | WP-04 | Deterministic suspension + loop test matrix | P0 | WP-02, WP-03 | TEST NAMES DISAGREE (C4–C7) |
| 5 | WP-05 | Backend replay projection for pending approvals | P1 | WP-03 | DESIGN READY |
| 6 | WP-06 | Web approval replay + retry/refusal state machine (items 10,15) | P1 | WP-03, WP-05 | DESIGN READY |
| 7 | WP-07 | Live tool-turn + live approval proof (item 13) | P1 | WP-04, WP-06 | BLOCKED: paid key/budget + runtime |
| 8 | WP-08 | Trace capture/promotion hardening (item 18) | P1 | capture lane | BLOCKED: raw capture empty |
| 9 | WP-09 | G-UI-3/W1 + G-UI-10/W2 closure (item 19) | P1 | WP-03 | BLOCKED: live specs absent |
| 10 | WP-10 | Axe subcheck closure (item 20) | P2 | Lead dependency approval | BLOCKED: dep not approved |
| 11 | WP-11 | Strict gate + 1.2 seal topology/release (items 21/22) | P1 | WP-01…WP-10 | BLOCKED: seal topology (C10) |
| 12 | WP-12 | Claims/evidence/non-claims register + wave index truth | P1 | continuous | THIS LEDGER |

### WP-01 — Approval gate primitives + approvals-table integrity
- **Target files:** `backend/crates/hiverouter-tools/src/approval.rs`; `backend/crates/hiverouter-core/src/tools_bridge.rs`.
- **Language:** Rust (rusqlite, tokio; no new crate deps).
- **Exact symbols:** `approval::expire(conn: &Connection, approval_id: &str, now_unix_secs: u64) -> Result<bool, ToolError>`; `approval::cancel_pending(...) -> Result<bool, ToolError>`; `approval::decision_label(...) -> Result<Option<String>, ToolError>`; `approval::count_for_run(conn, RunId) -> Result<u64, ToolError>`; unique partial index `CREATE UNIQUE INDEX IF NOT EXISTS approvals_one_pending_per_call ON approvals(run_id, tool_call_id) WHERE decision IS NULL`; `ToolsBridge::{expire_approval, cancel_approval, approval_decision, approval_count_for_run, verify_tool_grant, with_approval_ttl}`; keep WIP `expire_orphaned_pending` (boot, unconditional) **distinct** from per-wait `expire` (TTL CAS); refund `resolve` path WIP `expired_decision_refusal`.
- **Dependencies:** WIP merge first (`approval.rs` WIP +176 lines, `recovery.rs` +13, `http.rs` +15, `tools_bridge.rs` +17); `ApprovalGate` HEAD already has `verify_grant` (HEAD `:653`, WIP `:741`) and `request_tool_approval` (HEAD `:730`, WIP `:818`).
- **Verification command:** `cd backend` (PATH `%USERPROFILE%\.cargo\bin;..\tools\w64devkit\bin`), then `cargo test -p hiverouter-tools --lib approval::tests`, `cargo test -p hiverouter-core --test approval_ledger`, `cargo test -p hiverouter-core --test ws_transport`, `cargo test --workspace --locked`.
- **Acceptance:** one CAS predicate per primitive (bool true only when won); unique index rejects a second pending row for the same `(run_id, tool_call_id)`; `decision_label` reads each state; `count_for_run` counts all decisions; existing nonce/replay/TTL/ledger tests stay green; no production call of `expire_orphaned_pending` for the live timer.
- **Rollback:** revert the file hunks; index creation is `IF NOT EXISTS` and additive (drop index restores); no schema migration/backfill; UI/engine keep working with old refusal paths.
- **Blockers:** U2 (unknown duplicate pending rows on live DBs — query before index); WIP approval.rs still moving during planning; unique-index error mapping must use existing `is_unique_violation` (`approval.rs:204`) and refusal list, not a new code.

### WP-02 — Granted side-effect executor (items 3 + 16)
- **Target files:** new `backend/crates/hiverouter-tools/src/side_effects.rs`; `backend/crates/hiverouter-tools/src/jail.rs`; `backend/crates/hiverouter-tools/src/lib.rs`; optional `tool.rs` only if a typed mapping is added (`FileWriteArgs:255`, `ShellExecArgs:268` already exist).
- **Language:** Rust (std fs, tokio cancel token, existing `shell::execute`, `jail`, `sha256`).
- **Exact symbols:** `pub struct GrantedExecution { pub tool: ToolId, pub args: serde_json::Value, pub approval_id: ApprovalId }`; `pub fn execute_approved(jail: &WorkspaceJail, policy: &ShellPolicy, registry: &ToolRegistry, tier: AgentTier, tool: &str, args: &serde_json::Value, now_unix_secs: u64, cancellation: Option<&CancellationToken>) -> Result<ExecutedTool, ToolError>`; internal `preflight_file_write` → `commit_file_write`, `preflight_shell_exec` → `run_shell_exec`; `MAX_WRITE_BYTES`, `MAX_SHELL_OUTPUT_BYTES`, `MAX_SIDE_EFFECT_TEXT_BYTES`, `bounded_text`; `WorkspaceJail::write_atomic(&self, requested: &str, bytes: &[u8]) -> Result<PathBuf, ToolError>` (same-dir temp + `create_new` + `sync_all` + rename; refuse root/dirs/non-file/symlink).
- **Dependencies:** WP-01 (`ApprovalGate::verify_grant` is the only execution authority; `hash_args` `approval.rs:132`; frozen 65 refusals). **Ordering invariant:** all read-only pre-checks (schema, tier, path containment, size, stale `expected_sha256`, program allowlist, cwd/argv, cancellation) run **before** `verify_grant`, so a doomed call never burns consent; exactly one tool executes after grant.
- **Verification command:** `cargo test -p hiverouter-tools --lib executor::tests`, `cargo test -p hiverouter-tools --lib side_effects::tests`, `cargo test -p hiverouter-tools --test approval_ledger` (if used), `cargo test -p hiverouter-core --test suspension`, `cargo test --workspace --locked`.
- **Acceptance:** named tool tests (`execute_side_effect_writes_and_reports_hash`, `stale_expected_sha256_refuses_write`, `shell_exec_requires_allowlisted_program`, `execute_side_effect_refuses_non_side_effect_tools`); no read-only path executes side effects; `file_write` atomic and bounded; `shell_exec` allowlist/jail/env sanitize/timeout/cancel; `ToolOutcome::Timeout` after spawn keeps grant consumed; no argument bodies/argv/file bytes in events/ledger/`tool_turns`; grant verified exactly once.
- **Rollback:** new module + `jail.write_atomic` are additive; `lib.rs` export removal reverts; feature flag path: executor unreachable unless engine wires `execute_approved`; no data migration.
- **Blockers:** **C1** design divergence with R1 (R2's `execute_approved` single-authority vs R1's `verify_tool_grant` in engine + `executor::execute_side_effect`); **C2** R2's default-off `HIVEROUTER_SIDE_EFFECTS`/config lane vs R1's bridge-present wiring — Lead must pick one; U4 production shell allowlist policy (`conservative_default` only today, `tool.rs:315-380`); `expected_sha256` has no dedicated refusal code (map to `E_SCHEMA_INVALID`; semantic gap, §12.8 R1).

### WP-03 — Engine suspension/resume/wait/event wiring (items 1,2,5,9)
- **Target files:** `backend/crates/hiverouter-core/src/execution.rs`; `core/src/tool_loop.rs`; `core/src/tools_bridge.rs`; `core/src/ws.rs`; `core/src/http.rs`; `core/src/tool_loop.rs` (cap const); `core/tests/{suspension.rs, tool_loop_engine.rs, run_lifecycle.rs}`.
- **Language:** Rust (tokio `oneshot`/`select!`, `run_state` authority, existing `EventBus`).
- **Exact symbols:** `pub enum ApprovalWake { Resolved(ApprovalResolution), Cancelled }`; `pub struct ApprovalWaiters` with `new/arm(ApprovalId)->Result<Receiver<ApprovalWake>, Refusal>/wake_resolved(&ApprovalResolution)->bool/wake_cancelled(ApprovalId)->bool/disarm/len/is_empty`; `pub type Clock = Arc<dyn Fn() -> u64 + Send + Sync>`; `RunEngine` fields `tools: Option<Arc<ToolsBridge>>`, `waiters: Arc<ApprovalWaiters>`, `clock: Clock`, `shell_policy: ShellPolicy`; builders `with_tools/with_clock/with_shell_policy`; `notify_approval_resolved(&ApprovalResolution)->bool`; `MAX_APPROVAL_WAITS_PER_RUN: u32 = 8` in `tool_loop.rs`; async `execute_tool_turn`; `suspend_for_approval(...)`, `apply_committed_decision(...)`, `finish_timeout_wait(...)`, `finish_cancelled_wait(...)`, `resume_and_execute(...)`, `resume_denied(...)`, `persist_approval_transitions(...)`, `ArmedApproval` RAII; `dispatch` terminal-owner guard; `cancel_run`/`kill_worker` awaiting branches; `dispatch_inner` `Planning→Running` once; `ws.rs` calls `state.engine.notify_approval_resolved(&resolution)` after ACK/`APPROVAL_RESOLVED`, logs false, never fails the command; `http.rs` builds tools before engine and passes bridge to all `AppState` constructors.
- **Dependencies:** WP-01 + WP-02 committed; WIP recovery/interrupt/expiry merged; `run_state` approval effects (`:205-267`, `apply_run:465`, `apply_task:549`) are the only transition authority.
- **Verification command:** `cd backend` (PATH as above), `cargo test -p hiverouter-core --test suspension`, `--test tool_loop_engine`, `--test run_lifecycle`, `--test approval_ledger`, `--test ws_transport`, `--test recovery_gates`, `cargo test --workspace --locked`.
- **Acceptance:** the 14 named R1 §11 tests pass (suspend publishes one `APPROVAL_REQUEST` with nonce/run/task/tool-call/args/expiry; approve executes once; deny continues; timeout fails `E_APPROVAL_TIMEOUT`; cancel/kill consume; CAS-loss resumes; concurrent resolve one winner; second prompt `E_RUN_ACTIVE`; restart interrupted+expired; 9th wait `E_CALL_LIMIT`; wrong-args refused before effect; shell allowlist; no-bridge keeps `E_TOOL_NOT_ALLOWED`); exactly one terminal frame; waiter map empty after every path; no new refusal/event/command; engine never writes ledger; boot recovery expires.
- **Rollback:** absence of `.with_tools` keeps pre-1.2 read-only refusal (D3) → revert engine wiring only; `MAX_APPROVAL_WAITS_PER_RUN` additive; no DB/contract change; WIP recovery remains valid.
- **Blockers:** **C9** file collision with WP-06 backend replay (`ws.rs`/`tools_bridge.rs`) and WP-02 (`execution.rs` call seam) → strict wave sequencing required; U3 frame-count assertions in `run_lifecycle`/`ws_transport`; U7 tokio timer flakiness (1 s TTL); U6 bus replay ordering; D7 duplicate-terminal bug must be fixed exactly as R1 §5.6; `run_state` `Suspend` is legal only from `Running`, so D2 `RUN_STATUS running` is a behavior addition.

### WP-04 — Deterministic suspension + tool-loop test matrix
- **Target files:** new `backend/crates/hiverouter-core/tests/suspension.rs`; new `.../tests/tool_loop_matrix.rs`; reconcile `tests/tool_loop_engine.rs` (11 tests today; R4 §0 says 9 — actual `#[tokio::test]/#[test]` count = 11), `tests/run_lifecycle.rs` (7), `tests/approval_ledger.rs` (2), `tests/ws_transport.rs` (13).
- **Language:** Rust (`#[tokio::test]`, wiremock for matrix; no paid path).
- **Exact symbols:** test fns per R1 §11 (14) + R4 suites A (11 deterministic + ledger) + matrix `tool_loop_matrix.rs` multi-turn/caps; `MockExecutor`/`ScriptedExecutor` reuse, real `ApprovalGate` seed, fixed clock / short TTL, temp data dir; no new production symbols.
- **Dependencies:** WP-02 + WP-03.
- **Verification command:** `cargo test -p hiverouter-core --test suspension --locked`, `--test tool_loop_matrix --locked`, `cargo test --workspace --locked`; every run logged with HEAD+command+exit to `docs/evidence/1.2-*.log`.
- **Acceptance:** all named tests pass with 0 failures/skips; RED-first required for the invariants (one execution, one wake, one `TOOL_RESULT`, one terminal frame); test names are the claim IDs; no paid provider call.
- **Rollback:** tests are additive; revert files; runtime code untouched.
- **Blockers:** **C4** PROOF A7 expects `E_APPROVAL_DECISION_INVALID` for wrong args but `verify_grant` returns `E_APPROVAL_REQUIRED` for hash mismatch (decision-invalid is a wire-decision error) → adopt R1/WP-01 semantics; **C5** PROOF A4 expects run `failed` while current WIP recovery marks `Interrupted` + `error_code=E_APPROVAL_TIMEOUT` (`recovery.rs:57-62`, verified) → split into status-interrupted + code assertion; **C6** PROOF A5 says cancel produces no `TOOL_RESULT` but R1 D13 requires exactly one `cancelled` result; **C7** PROOF A6 restart-resume conflicts with D11 no crash-resume → rewrite as "replay after terminal/interrupt never auto-approves"; **C8** test-count/naming drift (9 vs 11 vs 12/14); R1 §11.3 says "12 existing scenarios" while actual count is 11.

### WP-05 — Backend replay projection for pending approvals
- **Target files:** `core/src/ws.rs` (`send_hello_and_replay` path), `core/src/tools_bridge.rs` (`pending_approval_events` consumer), `tools/src/approval.rs` (read-only projection `pending_for_run` already exists HEAD).
- **Language:** Rust.
- **Exact symbols:** replay call after `SERVER_HELLO` in `send_hello_and_replay` that emits pending `APPROVAL_REQUEST` events (bounded); existing `ToolsBridge::pending_approval_events(run_id)` (HEAD/WIP line 342/359) and `ApprovalGate::pending_for_run`; no new event/command; ordering policy decision required (all pending approvals bounded vs latest-active-run only — R3 open question 1).
- **Dependencies:** WP-03 (producer now reachable); WS hello path unchanged otherwise.
- **Verification command:** `cargo test -p hiverouter-core --test ws_transport`, `cargo test -p hiverouter-core --test approval_ledger`, e2e reconnect test (WP-06).
- **Acceptance:** reconnect emits exactly one bounded `APPROVAL_REQUEST` per still-pending approval; no nonce consumption; no duplicate when already resolved; `hello{last_seq}` ordering preserved; no new event/command/refusal.
- **Rollback:** remove replay call; existing behavior (no replay) returns; additive.
- **Blockers:** U6 EventBus replay ordering; synchronous WS publish while holding socket; bounded count policy must be Lead-approved (R3 §10 Q1).

### WP-06 — Web approval replay + retry/refusal state machine (items 10,15)
- **Target files:** new `web/src/state/approval.ts`; `web/src/state/store.ts`; `web/src/panels/ApprovalOverlay.tsx`; new `ApprovalCard.tsx`/`ApprovalStatusStrip.tsx`; `web/src/state/strings.ts`; `web/src/api/contract.ts` only if preview patch approved (WP-07); `web/tests/approval*.test.ts(x)`; `e2e/helpers/replay-harness.ts`; `e2e/specs/approval-flow.spec.ts`.
- **Language:** TypeScript 7 / React 19 / zustand 5 / Playwright.
- **Exact symbols:** `ApprovalRecord` extension (nullable six fields already exist `store.ts:61-73`); new typed state `approvalUiState: Record<ApprovalId, {status: 'idle'|'sending'|'awaiting'|'retryable'|'terminal', attempts:number, errorCode?:string}>`; actions `beginApprovalSend`, `settleApprovalAck`, `applyApprovalError`, `resetApprovalAttempts`, `applyApprovalList`; selectors `approvalStatus`, `canResolve`; `run()` result changes to a discriminated `CommandOutcome`/returns `{ok, code?}`; `resolveApproval` returns typed `CommandOutcome` (no swallowed code); remove `ApprovalOverlay` local `pendingResolution` lock (`ApprovalOverlay.tsx:167,198-225`); new e2e helpers `closeMockSocket`, `waitForReconnect`, `helloFrameCount`, `sendServerHello`, `sendErrorForCommand` (R3 §3.7); env/flag `VITE_UI_APPROVAL_REPLAY` default on only after tests.
- **Dependencies:** WP-05 backend replay; existing UI tool-loop/approval specs (`tool-loop.spec.ts`, `approval-flow.spec.ts`); no contract change for item 10.
- **Verification command:** `cd web`, `npm.cmd run typecheck`, `npm.cmd test -- --run` (incl. approval/store/overlay tests), `npm.cmd run build`; `cd e2e`, `node run-guarded.mjs --project=chrome specs/approval-flow.spec.ts specs/tool-loop.spec.ts`.
- **Acceptance:** reconnect replays a pending card exactly once, zero commands before click; resolved-while-away card read-only; no auto-send on reload; typed ERROR re-enables actions with code/how_to_fix and bounded retries (no stuck panel lock); ACK alone never closes/re-enables; `E_APPROVAL_REPLAY`/`E_APPROVAL_TIMEOUT` render named; no new strings hardcoded.
- **Rollback:** `VITE_UI_APPROVAL_REPLAY=0` restores no-replay behavior; `args_preview` flag stays off; typed state additions are additive; revert component split if needed.
- **Blockers:** **C9** engine and UX both edit `ws.rs`/`tools_bridge.rs`/`approval.rs` (backend replay) → assign backend replay to WP-05 owner before UI lane starts; Playwright `page.routeWebSocket` reconnect support is UNVERIFIED (R3 §8); `CHANGE-LEDGER-IMPL.md` not found by filename (U8) only blocks preview patch, not replay/retry; no build/test run in research.

### WP-07 — Args preview (item 11; optional, contract-gated)
- **Target files:** `backend/crates/hiverouter-contract/src/event.rs` (`ArgsPreview`, `ApprovalRequest.args_preview`), schema golden `schemas/contract-v1.json`, `backend/crates/hiverouter-core/src/preview.rs` (new), `tools/src/approval.rs` (approvals DDL `args_preview` column + producer), `web/src/api/contract.ts`, new `web/src/panels/ApprovalArgsPreview.tsx`, tests.
- **Language:** Rust + TypeScript/React.
- **Exact symbols:** `pub struct ArgsPreview { text: String, truncated: bool, redacted: bool }`; `ApprovalRequest.args_preview: Option<ArgsPreview>` (serde default, skip if none); `preview::redacted_args_preview(args) -> ArgsPreview` bounded ≤2048 chars, masks secret-shaped values; DB `ALTER TABLE approvals ADD COLUMN args_preview TEXT`; TS `ApprovalArgsPreview` + decoder; UI `ApprovalArgsPreview` with absent/hash-only fallback copy `STRINGS.approval.argsBoundByHash`.
- **Dependencies:** Lead approves additive contract + DB migration + redaction policy; WP-01/03.
- **Verification command:** `cargo test -p hiverouter-contract` (golden/matrix/roundtrip), `cargo test -p hiverouter-core --test suspension`, `npm.cmd test -- --run` (approval/preview), e2e approval card visibility.
- **Acceptance:** golden updated intentionally; preview never contains secrets/argument bodies beyond redacted text; absent means hash-only copy; no new event/command/refusal; bounded/truncated/redacted flags truthful; UI never reconstructs preview from hash.
- **Rollback:** contract field optional additive; DB column nullable additive; UI flag `VITE_UI_ARGS_PREVIEW=0`; revert migration in tools DDL only (no core migration).
- **Blockers:** **C3** R1 D9 says "no contract/UI change" for the engine wave; R3 calls preview "optional, same pattern as 1.2 additive fields" while its own comment says **1.3 additive**; item 11 is therefore deferred/conditional — 1.2 ships hash-only. U8 missing change-ledger process; U9 exact preview JSON shape/redaction rules need Lead decision; DB column ownership (tools vs core) must be settled with WP-01.

### WP-08 — Trace capture/promotion hardening (item 18; from R4 Suite C)
- **Target files:** new `e2e/specs/live-trace-capture.spec.ts`; `e2e/helpers/live-server.ts` (capture helper); `tools/qa/apply-live-trace.mjs`; `tools/qa/verify-harness.mjs`; new `tools/qa/tests/trace-tools.test.mjs`; `docs/trace/live/captures/w12/**` (evidence); `docs/trace/{controls.json,capture-registry.json,CAPTURE-REGISTRY.md}` updates.
- **Language:** TypeScript/Playwright + Node ESM + JSON evidence.
- **Exact symbols:** capture spec suites `capture replay`, `capture node cards`, `capture streaming terminal truth`, `capture approval`; helper `withTraceRecorder(page, dir)`/`stopTraceRecorder()` writing `frames.jsonl`/`dom.json`/`shot.png`/`manifest.json` (UTC, command, sha256); `apply-live-trace.mjs` guard `surfaces.length === 0 ⇒ exit 1`; `verify-harness.mjs` evidence-pointer existence/sha256 resolution; node tests for both guards.
- **Dependencies:** capture lane produces raw artifacts (CAPTURE-REGISTRY rows CAP-S10/S15-19/S27/S-APPROVAL/S22); trace seat owns docs validation, not e2e execution.
- **Verification command:** `node --test tools/qa/tests/trace-tools.test.mjs`; `node tools/qa/verify-harness.mjs`; promotion sequence `node tools/qa/apply-live-trace.mjs` → recompute counts → `node tools/qa/verify-harness.mjs`.
- **Acceptance:** empty/ambiguous capture refused; every promoted hop has committed raw artifact + evidence kind + sha256; counts recomputed (never hand-edited); current truth 1/264/5/49 of 319 preserved until captures exist; full-capture expected totals **24 WIRED / 253 PENDING / 0 MISSING / 42 UNVERIFIED** (per R4 C) with `raw_capture_backed_wired == wired`; no prose/test-summary promotion.
- **Rollback:** revert tools/qa hardening; delete w12 captures + controls edits together; restore capture-registry statuses; no product impact.
- **Blockers:** raw capture is EMPTY (`docs/trace/live/trace-capture.json` 286 B, `surfaces: []`, `2026-11-06T20:25:04.044Z`; `_meta.live_capture.raw_capture_backed_wired=1` only S22); no paid key for S27/approval live capture unless operator approves.

### WP-09 — G-UI-3/W1 + G-UI-10/W2 closure (item 19; R4 Suite D)
- **Target files:** new `e2e/specs/live-security.spec.ts`, new `e2e/specs/live-resync.spec.ts`; `e2e/helpers/live-server.ts` (foreign-origin helper, reconnect/kill); `e2e/GATE-EXCLUSIONS.md` rows removed only on evidence; `docs/evidence/1.2-gui-*.log`.
- **Language:** TypeScript/Playwright (real server harness).
- **Exact symbols:** `startLiveServer({withProviderKey:false})`, `openWsProbe`, foreign-origin page helper, `killSocketAndReconnect(page)`, `awaitWsClose1008(page)`; tests `foreign origin is refused with E_ORIGIN_REJECTED and no mutation`, `socket kill + hello{last_seq} replays without duplicate text`, `stale mutation is refused with E_RESYNC_REQUIRED`; evidence JSON/screenshots.
- **Dependencies:** WP-03 (server) + live-server helper; real server binary; no paid call for G-UI-3 (can use keyless); resync may use keyless or paid.
- **Verification command:** `cd e2e`, app ready env (`E2E_APP_READY=1`, `E2E_BASE_URL=http://127.0.0.1:8799`), `node run-guarded.mjs --project=chrome specs/live-security.spec.ts specs/live-resync.spec.ts`.
- **Acceptance:** close 1008 + typed `E_ORIGIN_REJECTED`, no state mutation/DB rows; reconnect hello last_seq, no duplicated reply text, snapshot/diff adoption; stale mutation `E_RESYNC_REQUIRED`; evidence committed; exclusion rows deleted; registry test green.
- **Rollback:** delete new specs; restore GATE-EXCLUSIONS rows; no product edits.
- **Blockers:** real server helper for synthetic foreign origin does not exist; no live spec infrastructure for socket kill; U12 paid requirement for resync if keyed path needed.

### WP-10 — Axe subcheck closure (item 20)
- **Target files:** `e2e/package.json` + `e2e/package-lock.json` (pinned dep); `e2e/specs/gui-gates.spec.ts` (G-UI-8 axe); `e2e/GATE-EXCLUSIONS.md` (remove row); `docs/evidence/1.2-axe.json`.
- **Language:** TypeScript/Playwright; axe-core.
- **Exact symbols:** `@axe-core/playwright` pinned exact version from official npm (Lead must name version + integrity; compiler must not invent); test `G-UI-8: axe serious/critical violations = 0 on web/dist`; result JSON of rule ids/impacts/selectors.
- **Dependencies:** Lead/tooling approval + network install; WP-06 UI DOM stable.
- **Verification command:** `cd e2e`, `npm.cmd install --save-dev --save-exact @axe-core/playwright@<approved>`, `npx playwright test specs/gui-gates.spec.ts --project=chrome -g "G-UI-8"`, `node run-guarded.mjs --project=chrome specs/gui-gates.spec.ts specs/a11y-gates.spec.ts`.
- **Acceptance:** serious+critical = 0; focus-ring portion still green; evidence JSON committed; exclusion row deleted only then.
- **Rollback:** remove dep/test + restore exclusion row; lockfile revert.
- **Blockers:** **no approved pinned version/integrity** (current exclusion records `MODULE_NOT_FOUND`, no install); network install; policy decision.

### WP-11 — Strict gate + 1.2 seal topology/release (items 21/22; R4 Suites E/F)
- **Target files:** `tools/build-release.ps1`; `tools/qa/run-gates.ps1`; new `tools/qa/check-seal-topology.mjs`; new `tools/qa/tests/seal-topology.test.mjs`; `tools/release-manifest.json` (generated v2 fields); `docs/RELEASE-EVIDENCE.md`; `docs/RELEASE.md`; `README.md`; git tag (owner).
- **Language:** PowerShell + Node ESM + JSON + git.
- **Exact symbols:** `build-release.ps1` flags `-SourceOnly`, `-RecordSeal`, `-SealCommit <sha>` (new); manifest `schema hivemind.release-manifest.v2` with `sealed: [ { tag, source_commit, seal_commit } ]`; `tag_target=null` at source build; `check-seal-topology.mjs` pure checker; `run-gates.ps1` release mode accepts sealed commit D when allowlisted diff from source C (only dist/docs/manifest/evidence paths) and tag target C; commands per R4 §5/§6.
- **Dependencies:** all WPs green; owner-approved git tag (build-release never moves tags); clean tree; paid `-Live` for live proof if claimed.
- **Verification command:** `powershell -NoProfile -ExecutionPolicy Bypass -File tools/build-release.ps1 -SourceOnly -Version 1.2.0 -Tag hivemind-1.2.0` (at C) → then `-RecordSeal -SealCommit D`; `node tools/qa/check-seal-topology.mjs`; `node --test tools/qa/tests/seal-topology.test.mjs`; `pwsh tools/qa/run-gates.ps1 -Release -RequireE2E` (exit 0 on D; exit 3 only pre-release); `node tools/qa/check-docs-truth.mjs`.
- **Acceptance:** source C tagged and immutable; manifest v2 records C/D; gate accepts only allowlisted seal diff (no code/test/config in C..D); `-Release` exit 0 with zero skips on D; 1.1 artifacts/tags unchanged; docs evidence updated; no tag move (or owner-confirmed supersession recorded).
- **Rollback:** revert tools/scripts (owner); manifest schema change additive (v1 consumers ignore new fields); restore prior release record; tag operations need owner confirmation.
- **Blockers:** **C10** three incompatible seal designs: R4 F (`-SourceOnly`/`-RecordSeal`, manifest v2, tag D, checker + 5 node tests), S4/PROOF-RELEASE (`seal_commit` + allowlist), current build-release (requires tag at HEAD, records `source_commit=HEAD` `:545`, no v2 fields) — adopt R4 F as target; owner-only tag/build action (U10); paid live proof and dirty tree block final run.

### WP-12 — Claims/evidence/non-claims register
- **Target files:** `plans/foresight/COMPILED-LEDGER.md` (this file) + the four reports/wave index; **no product files**.
- **Language:** Markdown/JSON evidence only.
- **Exact symbols:** claim IDs `C-ENG-01…`, `C-SE-01…`, `C-UX-01…`, `C-PRF-01…`, contradiction IDs `C1…C10`; evidence status labels.
- **Dependencies:** continuous; rerun at each wave receipt.
- **Verification command:** review against current `git status`/`Get-FileHash`; each wave receipt maps claims→evidence→status; no claim without path:line/command.
- **Acceptance:** every report claim appears once (deduplicated), statuses correct; contradictions recorded, not silently resolved; non-claims repeated verbatim in each implementation receipt.
- **Rollback:** documentation only.
- **Blockers:** reports are uncommitted/drifting; item-number register absent (U1).

---

## 2. Paste-ready DeepSeek/HiveMind handoff prompt (Lead → implementation agent)

```text
You are the implementation wave for HiveMind 1.2 approval/side-effect closure. READ-ONLY planning is over; you may edit ONLY the files listed for your assigned wave. Do not edit reports, other lanes' files, contract/DB migrations, or git tags unless your wave explicitly lists them.

REPO: C:\Users\Riley\Desktop\hiverouter-v1  (Windows, PowerShell 5.1; PATH setup for Rust: $env:PATH="$env:USERPROFILE\.cargo\bin;$PWD\tools\w64devkit\bin;"+$env:PATH; cargo/npm from the repo dirs)
BASELINE: HEAD 37f0e7c + uncommitted WIP (8 product paths + untracked test). The Lead MUST commit/merge the WIP first; start only on a clean tree with WIP merged, and record `git rev-parse HEAD`, `git status --porcelain` in your first receipt.
READ FIRST (full): plans/foresight/CODE-APPROVAL-ENGINE.md, CODE-SIDE-EFFECTS.md, CODE-APPROVAL-UX.md, CODE-PROOF.md, COMPILED-LEDGER.md (the four reports are uncommitted and may drift; re-hash them).
SOFTWARE RULES: complete code only (no TODO/stub/placeholder/mock in product paths); read code/callers/tests before edit; no new refusal code, event, or command; no `db.rs`/`contract/*`/`web/*` edits unless your wave says so; never invent symbols; run the listed commands and quote output; no paid provider call without LIVE_ONE_PROMPT=1/LIVE_TOOL_TURN=1 and an operator present key — otherwise the spec skips by name and you must not count it as pass.
COLLISION-FREE WAVES (one writer per file set; do not run two waves concurrently on overlapping files; commit between waves by Lead):
  WAVE-SE (tools, WP-01+WP-02): tools/src/approval.rs, NEW tools/src/side_effects.rs, jail.rs (write_atomic only), lib.rs, tools tests. Do NOT touch core/, contract/, web/, db.rs.
  WAVE-ENGINE (WP-03): core/src/execution.rs, tool_loop.rs, tools_bridge.rs, ws.rs, http.rs; core/tests/{suspension.rs NEW, tool_loop_engine.rs, run_lifecycle.rs}. Depends WAVE-SE committed. Do NOT touch web/contract/db.
  WAVE-REPLAY (WP-05): core/src/{ws.rs,tools_bridge.rs}, tools/src/approval.rs read-only projection — runs after WAVE-ENGINE; same owner lane only.
  WAVE-WEB (WP-06): web/src/state/{approval.ts NEW,store.ts}, web/src/panels/{ApprovalOverlay.tsx,ApprovalCard.tsx NEW,ApprovalStatusStrip.tsx NEW}, strings.ts, web tests, e2e/helpers/replay-harness.ts, e2e/specs/approval-flow.spec.ts. Depends WAVE-REPLAY; no backend edits.
  WAVE-PROOF (WP-04/07/08/09/10): e2e/specs/* NEW, tools/qa/{apply-live-trace.mjs,verify-harness.mjs,check-seal-topology.mjs NEW, tests NEW}, docs/evidence/docs trace files. Paid gates only with operator flags.
  WAVE-SEAL (WP-11, Lead/owner only): tools/build-release.ps1, tools/qa/run-gates.ps1, tools/release-manifest.json, docs/RELEASE-EVIDENCE.md, README, tags. Do not start before all waves green.
EXACT ACCEPTANCE COMMANDS (run and paste output; no “should work”):
  cargo test -p hiverouter-core --test suspension --locked; cargo test -p hiverouter-core --test tool_loop_engine --locked; cargo test -p hiverouter-core --test run_lifecycle --locked; cargo test -p hiverouter-core --test approval_ledger --locked; cargo test -p hiverouter-core --test ws_transport --locked; cargo test -p hiverouter-tools --lib --locked; cargo test --workspace --locked
  cd web; npm.cmd run typecheck; npm.cmd test -- --run; npm.cmd run build
  cd e2e; node run-guarded.mjs --project=chrome specs/tool-loop.spec.ts specs/approval-flow.spec.ts specs/gui-gates.spec.ts specs/gui-residual.spec.ts specs/a11y-gates.spec.ts
  # app already running for live: $env:E2E_APP_READY='1'; $env:E2E_BASE_URL='http://127.0.0.1:8799'; $env:E2E_SERVER_BIN='target/release/hiverouter.exe'
  # paid (operator only): $env:LIVE_ONE_PROMPT='1'; $env:LIVE_TOOL_TURN='1'; key only in process env; node run-guarded.mjs --project=chrome specs/live-tool-turn.spec.ts specs/live-tool-approval.spec.ts
  # strict gate (Lead): powershell -NoProfile -ExecutionPolicy Bypass -File tools/qa/run-gates.ps1 -DryRun | -Rehearsal -RequireE2E | -Release -RequireE2E (expect exit 3 PRE-RELEASE pre-seal; exit 0 sealed)
DO NOT CLAIM (repeat in receipt): tool loop/suspension works; approval executes once; crash-resume; cross-process exactly-once; args preview is 1.2; trace hops were promoted (current truth 1/264/5/49 of 319; raw capture empty); G-UI-3/10/axe closed; strict gate green; 1.2 released/tagged/artifacts; paid live proof without quoted redacted log; anything not in your quoted command output.
REPORT FORMAT (final answer): (1) wave + HEAD + clean-tree proof; (2) files changed with exact symbols; (3) tests/commands run with quoted results (pass/fail/skip); (4) VERIFIED vs READ vs INFERRED vs UNVERIFIED; (5) contradictions/blockers; (6) rollback notes; (7) what must not be claimed. Do not commit; Lead owns the commit.
```

---

## 3. Claim/evidence table (evidence-check style)

| ID | Claim | Evidence (path:line / command) | Status |
|---|---|---|---|
| C-ENG-01 | No engine suspension today; non-read calls refused in `execute_tool_plan` | `core/src/execution.rs:875-916` HEAD (`:879-920` WIP): `E_TOOL_NOT_ALLOWED` branch; grep `awaiting_approval` producer in execution = none | VERIFIED (read) |
| C-ENG-02 | Approval producer/wire event exist but have no production caller | `tools_bridge.rs:271-348` HEAD (`:287-364` WIP) `request_model_tool_approval`; grep call sites: definition/doc/tests inside `tools_bridge.rs` + one untracked test; `execution.rs` caller = 0 | VERIFIED (read/grep) |
| C-ENG-03 | WS resolve does not wake the engine | `ws.rs:465-483` HEAD: resolve → ACK/`APPROVAL_RESOLVED`; no `notify_approval_resolved` symbol anywhere | VERIFIED (read/grep) |
| C-ENG-04 | `run_state` approval edges exist and are unused by execution | `run_state.rs:205-267,465-620`; grep `run_state` in `execution.rs` = 0 | VERIFIED (grep) |
| C-ENG-05 | WIP boot expiry/recovery marks awaiting runs `interrupted` + `E_APPROVAL_TIMEOUT` | `recovery.rs:53-63` (WIP), `approval.rs:548` `expire_orphaned_pending`; `git diff` shows WIP | VERIFIED (read WIP) |
| C-ENG-06 | `dispatch` may publish duplicate terminal frames vs `cancel_run`/timeout | `execution.rs:539-583` vs `:499-519` (HEAD lines) | VERIFIED (read) |
| C-ENG-07 | `tokio` `sync` feature available for `oneshot` | `backend/Cargo.toml` tokio features include `sync` | VERIFIED (read) |
| C-ENG-08 | Engine runtime behavior of proposed diff | not implemented; no build/test run | **UNVERIFIED** |
| C-ENG-09 | Exact “items 1,2,5,9” list content | not in repo (S1 §0 documents assumption) | **UNKNOWN** |
| C-SE-01 | Read-only executor has no side-effect path | `tools/src/executor.rs:52` `execute_read_only`; no `execute_approved`/`execute_side_effect` symbol | VERIFIED (read/grep) |
| C-SE-02 | `ApprovalGate::verify_grant` single-use CAS exists | HEAD `approval.rs:653`, WIP `:741`; `executed_at IS NULL` update at WIP `:771-773` | VERIFIED (read; anchor drift +88) |
| C-SE-03 | `expected_sha256` has no production use | `tool.rs:262` field only; bench fixture + one unit-test JSON; no executor read | VERIFIED with nuance (no dedicated refusal code) |
| C-SE-04 | No unique pending index on approvals | `approval.rs:143-161`: `UNIQUE(nonce)` only + `approvals_run_pending`/`approvals_session` non-unique indexes | VERIFIED (read) |
| C-SE-05 | `shell::execute` is async and allowlisted | `shell.rs:173` async; `conservative_default:66`, `allows:96` | VERIFIED (read) |
| C-SE-06 | New side-effect module/preflight/grant ordering | not implemented; R2 blueprint only | **UNVERIFIED** |
| C-UX-01 | Store has six nullable approval fields + `resolvingApprovals`; `resolveApproval` sends once | `web/src/state/store.ts:61-73,160-183,498-511` | VERIFIED (read) |
| C-UX-02 | `run()` hides typed refusal code; overlay local lock can stick | `store.ts` `run()` boolean; `ApprovalOverlay.tsx:167,198-225` local `pendingResolution` pruned only on card removal | VERIFIED (read) |
| C-UX-03 | No production pending-approval replay on reconnect | `ws.rs:225` `send_hello_and_replay`; `pending_approval_events` call sites only `tools_bridge.rs:359,699,711` (def/tests) | VERIFIED (read/grep) |
| C-UX-04 | `args_preview` absent from contract and UI | `contract.ts` ApprovalRequest payload has no `args_preview`; `event.rs:252`; grep `args_preview` = 0 | VERIFIED (read/grep) |
| C-UX-05 | Reconnect/reload UI behavior | not implemented/tested in this research; Playwright `page.routeWebSocket` reconnect support unverified | **UNVERIFIED / UNKNOWN** |
| C-UX-06 | `CHANGE-LEDGER-IMPL.md` location/process | filename scan found none; frozen doc `UI-COMMANDS-v1.md:1-16` names it | **UNKNOWN** |
| C-PRF-01 | Trace truth 1 WIRED / 264 PENDING / 5 MISSING / 49 UNVERIFIED of 319 | `docs/trace/controls.json:_meta.counts`; raw capture 286 B `surfaces: []` | VERIFIED (read) |
| C-PRF-02 | 1.1 live paid proof exists; no 1.2 live tool-turn proof | `docs/evidence/live-1.1-paid.log` (5 passed/EXIT=0), `w11` captures; no live-tool spec | VERIFIED (read) |
| C-PRF-03 | G-UI-3/10/W1/W2/axe are exclusions, not passes | `e2e/GATE-EXCLUSIONS.md:10-14`; `gui-gates.spec.ts:226,335-361` | VERIFIED (read) |
| C-PRF-04 | Gate harness flags/exit codes implemented; not executed on this tree | `run-gates.ps1:23-45,248-316,351` | VERIFIED (read); execution **UNVERIFIED** |
| C-PRF-05 | Build at HEAD ahead of tag is rehearsal-only; no seal topology | `build-release.ps1:239-245,543-546`; no `seal_commit`/v2 fields | VERIFIED (read); 1.2 seal blocker |
| C-PRF-06 | Trace tooling cannot refuse bad promotion input | `apply-live-trace.mjs:18-58` only existence; `verify-harness.mjs` no pointer resolution | VERIFIED (read) |
| C-PRF-07 | Suspension/tool-loop runtime + live suites green | not run | **UNVERIFIED** |
| C-PRF-08 | Expected full-capture trace totals | arithmetic from controls + registry = 24/253/0/42 (R4 C); not measured | **UNVERIFIED (INFERRED arithmetic)** |
| C-LED-01 | Corpus frozen hashes as given in §Inputs | `Get-FileHash` at compile time; engine report changed twice during compile | VERIFIED at freeze; report drift noted |
| C-LED-02 | `tool_loop_engine.rs` has 11 tests, not 9/12 as reports say | `Select-String '#[tokio::test]|#[test]'` = 11 | VERIFIED (grep) |
| C-LED-03 | Reports were mid-flight during compilation | mtimes 18:23–18:31; engine 99,433→102,091 B | VERIFIED (mtime/bytes) |

---

## 4. Explicit “do not claim” list

1. **No runtime/suspension/tool-loop claim** until WP-02/03/04 land and a Lead-run `cargo test --workspace --locked` quotes green. The reports are blueprints; the current engine refuses non-read tools.
2. **No “approval executes exactly once”** claim beyond per-process single-use grant semantics; cross-process exactly-once and crash-resume are explicitly not claimed (R1 D11, R1 §12.3).
3. **No “live tool turn works”** claim without the paid, operator-gated `LIVE_TOOL_TURN=1` run + committed redacted evidence; skipped/absent key = named skip, never pass.
4. **No trace promotion / improved counts** from prose, source pointers, unit-test summaries, or `target/` logs. Current truth stays **1/264/5/49 of 319** until raw captures are committed; `raw_capture_backed_wired` must equal promoted WIRED hops.
5. **No “G-UI-3/G-UI-10/W1/W2 closed”** from mocks; only real-server live evidence closes them and deletes the exclusion rows.
6. **No “axe clean”** without a Lead-approved pinned dependency + committed axe result; the row stays otherwise.
7. **No “1.2 released/tagged/artifacts”** claim before WP-11 seal topology lands, the owner creates the tag, and `run-gates -Release -RequireE2E` exits 0 with zero skips on the seal commit; `GATES: PRE-RELEASE` (exit 3) is not a release.
8. **No args preview in 1.2** — R3 item (b) is optional/1.3-gated by a contract patch + DB migration + missing `CHANGE-LEDGER-IMPL.md`; ship hash-only copy.
9. **No `expected_sha256`/`E_FILE_CHANGED` dedicated refusal claim**; the field has no production read and maps to `E_SCHEMA_INVALID` as a known semantic gap.
10. **No shell sandbox / arbitrary execution safety** claim: allowlist + jail + caps only; `conservative_default` is a default, not a security boundary.
11. **No new refusal codes/events/commands, no ledger writes by the engine, no extra `run_state` claims** beyond the approval edges + one `Start`.
12. **No compile/test/build claim by the planners**; all report runtime statements are `UNVERIFIED` until a Lead-run command is quoted.
13. **No claim that the WIP recovery/boot expiry was authored by the implementation wave** — it is a merge precondition (R1 D8).
14. **No exact item-number mapping claim** — items 1/2/5/9, 3/16, 10/11/15 are assumptions recorded in source plans; the master list is absent (U1).

---

## 5. Contradictions / dedupe / UNKNOWN register

### Contradictions (must be resolved by the Lead before parallel coding)
| ID | Contradiction | Resolution recommendation |
|---|---|---|
| C1 | **Waiter/executor identity:** R1 `ApprovalWaiters` in `execution.rs` + `executor::execute_side_effect` + engine calls `verify_tool_grant`; R2 `ApprovalBroker` in new `core/src/approval_wait.rs` + `tools/src/side_effects.rs::execute_approved` (preflight-before-grant, verify inside tools). | Adopt R1's `ApprovalWaiters` (committed-row id authority, in-process only) **plus** R2's `execute_approved` module/ordering; engine calls one bridge delegate/executor function; do not implement both modules. Keep R2's preflight-before-grant invariant; remove engine-side duplicate verify (or make it a pre-grant read-only check). |
| C2 | **Rollout gate:** R2 adds `HIVEROUTER_SIDE_EFFECTS` default-off + approved shell policy; R1 uses bridge/jail presence (`.with_tools`) and production always wires it. | Lead decision: default-off feature flag for 1.2 (safer, matches R2) **or** always-wired (R1); if flag chosen, R1 never-wired tests still prove `E_TOOL_NOT_ALLOWED`; document env var in runbook before claim. |
| C3 | **Contract change:** R3 item (b) adds `args_preview` (contract + DB + UI); R1 D9 says no contract/UI change for this wave. | Defer item (b) to 1.3 unless Lead explicitly authorizes an additive contract + tools DDL migration; 1.2 ships hash-only copy. Record `CHANGE-LEDGER-IMPL.md` requirement (U8). |
| C4 | **Wrong-args refusal:** R4 A7 expects `E_APPROVAL_DECISION_INVALID`; R1 S12 expects `E_APPROVAL_REQUIRED`; `verify_grant` returns `E_APPROVAL_REQUIRED` for hash mismatch. | Adopt R1/WP-01: hash mismatch = `E_APPROVAL_REQUIRED` (decision-invalid is only for wire decision payloads). Update R4 A7 name/expectation. |
| C5 | **Recovery status:** R4 A4 expects run `failed`; current WIP recovery marks `Interrupted` + `error_code=E_APPROVAL_TIMEOUT` (`recovery.rs:57-62`). | Assert status `interrupted` + code `E_APPROVAL_TIMEOUT`; split A4 into status/code/expiry/never-auto-approve assertions. |
| C6 | **Cancel result:** R4 A5 expects no `TOOL_RESULT` on cancel; R1 D13 requires exactly one `TOOL_RESULT cancelled`. | Follow R1 D13 (one result per planned call); reconcile A5 accordingly. |
| C7 | **Restart resume:** R4 A6 “approve, restart engine, drive to terminal”; R1 D11 says no crash-resume. | Rewrite A6 as "restart marks interrupted, pending expired, later resolve gives `E_APPROVAL_TIMEOUT`, no auto-approve"; no crash-resume test. |
| C8 | **Counts/names:** R1 §0 says tool_loop_engine 9 tests, §11.3 says 12; actual 11; R4 §0 says 9; R1 lists 14 suspension tests, R4 lists 11+ledger. | Freeze test names in WP-04 from R1 §11 (14) + R2 tool matrix; fix count statements in receipt; do not duplicate tests. |
| C9 | **File collision:** engine (WP-03) and UX backend replay (WP-05) both edit `ws.rs`, `tools_bridge.rs`, `approval.rs`; side-effects (WP-02) needs `execution.rs` call seam if merged into engine. | Strict wave sequencing (handoff §2): backend replay after engine commit or same single owner; WP-02 exposes `execute_approved` symbol before engine edit; no concurrent writers. |
| C10 | **Seal topology design:** R4 F (`-SourceOnly`/`-RecordSeal`, manifest v2, tag D, pure checker + 5 node tests) vs S4 (`seal_commit` + allowlist) vs current build-release (tag at HEAD, `source_commit=HEAD` `:545`, no v2). | Adopt R4 F as target; mark S4 superseded; gate must verify tag target C + seal commit D + allowlisted C..D diff. Owner-only tag/build. |
| C11 | **Anchor drift:** R1 cites `approval.rs verify_grant:694`; HEAD is `:653`, WIP `:741`; `tools_bridge request_model_tool_approval:287` → WIP `:288`; execution +4 lines. | Treat report line numbers as report-time; use current anchors table in this ledger for implementation; re-grep before edits. |
| C12 | **Report drift:** engine report changed twice during compilation (99,433→100,059→102,091 B); reports uncommitted. | Re-freeze hashes at implementation start; diff reports; do not build against stale text. |
| C13 | `expected_sha256` “unused” vs 3 static references | Claim holds for production behavior; document as no production read + no dedicated refusal (C-SE-03). |

### Deduplication result
- **R1 vs S1:** same engine work; R1 is code-level and explicitly supersedes S1 on D3 (no-tools refusal stays `E_TOOL_NOT_ALLOWED`, not `E_INTERNAL_ERROR`), D4 (TTL owned by request row; `ToolsBridge::with_approval_ttl` test seam, not engine TTL), D5 (no engine ledger), D6 (run_state only approval edges + Start). Ledger uses R1.
- **R2 vs S2:** same side-effect work; R2 adds the file-write/shell preflight/commit chain, atomic write, bounds, and grant ordering. Ledger uses R2 (with C1/C2 reconciliation).
- **R3 vs S3:** same UX items; R3 adds exact components/states/tests and retry-state machine. Ledger uses R3 (item b deferred).
- **R4 vs S4:** same proof scope; R4 is newer with seal topology variants; ledger uses R4 F/E, marks S4 seal variant superseded (C10).
- Removed duplicates: 3 plan-level restatements of engine state table; 2 duplicate "read-only executor refuses" claims; 2 duplicate gate-flag lists; 2 duplicate trace-floor statements; 4 repeated setup commands consolidated in §2.

### UNKNOWN register (carry into receipts)
| ID | Unknown | Settle by |
|---|---|---|
| U1 | Master item list (1/2/5/9, 3/16, 10/11/15) and items 12,13,18–25 labels | Lead provides register; current mapping is assumption recorded in source plans |
| U2 | Duplicate pending `(run_id, tool_call_id)` rows on live DBs | `SELECT run_id,tool_call_id,COUNT(*) FROM approvals WHERE decision IS NULL GROUP BY 1,2 HAVING COUNT(*)>1` before unique index |
| U3 | Existing tests tolerate extra `RUN_STATUS running`/notify-false | Read/re-run `run_lifecycle`, `ws_transport` at implementation |
| U4 | Production shell allowlist policy | Product decision; `conservative_default` only today |
| U5 | UI needs/renders extra fields | Read contract/overlay at implementation |
| U6 | EventBus replay ordering for `APPROVAL_REQUEST` vs `TOOL_CALL` | Read `bus.rs` replay tests |
| U7 | 1 s TTL timer flakiness under tokio test runtime | Lead run; `tokio` `test-util`/manual wake if flaky |
| U8 | `CHANGE-LEDGER-IMPL.md` location/process | Lead/tooling owner; blocks preview contract change only |
| U9 | `args_preview` JSON shape/redaction rules/DB owner | Lead decision (item 11, 1.3) |
| U10 | Owner tag/build actions, key/budget, axe version | Lead/operator |
| U11 | WIP drift may change again | Re-diff at implementation start |
| U12 | Playwright `routeWebSocket` reconnect support | Run `node run-guarded.mjs` with a throwaway reconnect spec |
| U13 | Live server foreign-origin/socket-kill helper feasibility | Build helper; verify against real server |
| U14 | Trace raw capture availability/paid approval for S27/approval | e2e/QA lane + operator |

---

## 6. Residual and compiler result

**Residual:**
- All four reports and two plans are **uncommitted** and were still being written while this ledger compiled; the engine report changed twice. The ledger's hashes are a freeze, not a guarantee of final content.
- Reports contain **blueprint code**, not compiled code: all runtime behavior is UNVERIFIED. Every snippet must be adapted (types/imports/WIP drift) by the implementing wave.
- **Line anchors drift** between HEAD and WIP (table in §Inputs); report citations were preserved as reported and annotated with current offsets.
- **C1–C10 require Lead decisions/sequencing** before parallel coding; C9/C10 are release/architecture blockers, not coding nits.
- The 1.2 seal is blocked by the missing seal topology (C10) and the uncommitted runtime WIP; no 1.2 artifact/tag exists.
- Trace remains 1/264/5/49 of 319 with an empty raw capture; no promotion is justified yet.
- G-UI-3/10/axe closure and live tool-turn proof need a real server, operator key/budget, and approved axe dependency.

**Exact compiler result:**
- **8 inputs frozen** (4 reports + 4 source plans; bytes/lines/sha256 in §Inputs; engine report drift noted).
- **12 work packages** produced (WP-01…WP-12), **ranked**, each with target files, language, exact symbols, dependencies, verification command, acceptance, rollback, blockers.
- **13 contradictions** (C1–C13) and **14 UNKNOWNs** (U1–U14) recorded; **dedupe** collapsed each source plan into its newer report (R1/R2/R3/R4) and removed 4 plan-level duplicate claims + 3 duplicate command/flag sets.
- **1 paste-ready, wave-scoped, file-collision-free handoff prompt** in §2 (six waves: SE → ENGINE → REPLAY → WEB → PROOF → SEAL, each with exclusive file sets and exact commands).
- **32 claim/evidence rows** in §3 (VERIFIED / READ / INFERRED / UNVERIFIED / UNKNOWN; no claim without an evidence anchor) and **14 explicit non-claims** in §4.
- **Read-only guarantee:** no product/test/config/gate/git/tag writes, no builds, no tests, no key access, no paid calls; the only file written is this ledger.
