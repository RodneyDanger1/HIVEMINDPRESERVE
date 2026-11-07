# RUN-2 COMPILED LEDGER — approval / side-effect / replay / proof closure

**Compiler:** strip-watchdog (Run-2 Ledger Compiler) · `/deep-research` + `/forgewright-lead` + `/evidence-check` + `/truthseeker`.
**Mode:** READ-ONLY for product code; **this file is the only write**. No source edits, builds, test runs, git commits/tags, keys, or paid calls by the compiler.
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **HEAD `90cb7a7`** (`feat(1.2): reasoning passback, approval boot expiry, recovery timeout; foresight run-1 plans/ledger`, committed 18:50:20 CT) · version `1.2.0-dev`; sealed release remains **1.1.0**.
**Tree at compile snapshot:** `git status --porcelain` = **11 entries at final verification (12 including this ledger)**: 4 modified product files (`core/src/lib.rs`, `core/src/tools_bridge.rs`, `tools/src/approval.rs`, `tools/src/lib.rs`), 2 untracked product files (`core/src/approval_wait.rs` and `tools/tests/lead_bugcheck_wp01.rs`, which appeared while this ledger was compiling), the 4 untracked Run-2 reports, and untracked `tools/qa/apply_wp01.py`. **WP-01 is implemented in the working tree but uncommitted; a separate waiter-module lane started at ~19:00 CT while this ledger was compiling.**

## Inputs — frozen Run-2 corpus (read; no product mutation)

| # | Report | Bytes | Lines | SHA-256 (freeze) | Covers |
|---|---|---|---|---|---|
| E | `plans/foresight/RUN2-ENGINE-CODE.md` | 67,831 | 1,135 | `B2BC41CBD20EEE992E3616EF06FA7E77590955C56894976A5B6F40F8EEF4D501` | WP-03 engine suspension (H0–H12) |
| S | `plans/foresight/RUN2-SIDE-EFFECT-CODE.md` | 46,650 | 706 | `6F3927FCFD40C315D08242137F6DDEE5D589F83B0C4FE06EFDF11733235AA646` | WP-02 side-effect executor (`file_write` only) |
| W | `plans/foresight/RUN2-WEB-CODE.md` | 35,879 | 266 | `5BAA9AA84A9F9C4FD5C57F3F881BEA5E735CD372C805087BBA912DAE68B3365A` | WP-06 web replay/retry (H1–H13) |
| P | `plans/foresight/RUN2-PROOF-CODE.md` | 26,017 | 264 | `C1A96330EAC1334DBD7EA8F22C6A4D4E7273E61826FCAAA7398795773195B705` | WP-04 matrix + WP-07 live proof |

**Context read for verification:** `plans/foresight/RUN-1.md:15-34` (Run-1 baselines/defects), `plans/foresight/COMPILED-LEDGER.md` (Run-1 WPs/IDs), current product files and test counts, `docs/evidence/1.2-backend-tests-6.log`, `e2e/helpers/live-server.ts`, `e2e/specs/{tool-loop,approval-flow}.spec.ts`, `docs/trace/controls.json`.

**Frozen-state caveats (material):**
- The four reports are **untracked working-tree files**; hashes above are the compile-time freeze. Report writers observed different tree states (E/W say clean at recon; S/P observed WP-01 dirty) because WP-01 edits began ~18:51 CT; the tree is now dirty and changed again at ~19:00 CT. Re-hash and re-read before coding.
- **WP-01 code is present** in the working tree: `approval.rs::{expire:572,cancel_pending:588,decision_label:606,count_for_run:621}`, unique index `approvals_one_pending_per_call` (`approval.rs:163`); `ToolsBridge::{with_approval_ttl:106,expire_approval:115,cancel_approval:126,approval_decision:141,approval_count_for_run:152,verify_tool_grant:166}`; `tools/src/lib.rs` approval re-exports. `approval.rs` has **16 `#[test]` attributes** now (the 396 log lists 14 approval tests and predates this final test edit). It is **not committed**, so it is not a verified baseline.
- **New untracked waiter module** `core/src/approval_wait.rs` (174 lines, 6,243 B, mtime 19:00:37) defines `ApprovalWaiters`/`ApprovalWake` with 4 tests; `core/src/lib.rs` exports it (mtime 19:00:19). It is uncommitted, not in any Run-2 report's file map (E places these types inline in `execution.rs`, H2), and has no engine caller yet.
- **Test baseline:** `docs/evidence/1.2-backend-tests-6.log` = 39 suites, **396 passed / 0 failed / 2 ignored**, `EXIT=0`, mtime 18:56:23; `RUN-1.md:21` records the same 396 baseline. The log contains 14 `approval::tests` and **not** the WP-01 test `expire_cancel_and_decision_labels_are_cas_guarded` (`approval.rs:1069`), so it is a pre-final-WP-01 baseline; **396 is not the post-WP-01 count** and no run of the current WIP exists (UNVERIFIED).
- Verified current counts: `tool_loop_engine` 11, `run_lifecycle` 7, `approval_ledger` 2, `ws_transport` 13, `recovery_gates` 3; no `suspension.rs`, no `tool_loop_matrix.rs`, no `side_effects.rs`, no `write_atomic`, no `execute_approved`, no `notify_approval_resolved`, no live tool/approval specs, no `LIVE_TOOL_TURN`; e2e `tool-loop.spec.ts` = 8 tests, `approval-flow.spec.ts` = 8 tests; web baseline 147/147 (RUN-1.md:22); trace still 1/264/5/49 of 319 with empty raw capture.

## WP mapping (Run-1 ledger → Run-2 reports)

| Run-1 ID | Run-2 report | Run-2 work | Status at compile |
|---|---|---|---|
| WP-01 | prerequisite | approval primitives + bridge delegates + unique pending index | **implemented uncommitted; tests unrun** |
| WP-02 | S | side-effect executor (`file_write`) | not started |
| WP-03 | E | engine suspension/resume/wait/event wiring | not started; waiter module WIP exists |
| WP-04 | P | deterministic suspension + loop matrix | not started |
| WP-05 | **missing Run-2 report** (design in W §4) | backend pending-approval replay | not started; required by W |
| WP-06 | W | web replay + retry/refusal state machine | not started |
| WP-07 | P | live tool-turn + live approval proof (item 13) | not started; paid-gated |
| (WP-07 header vs table drift) | P/W flag it | ledger header calls WP-07 args preview, table calls it live proof | adopt **live proof**; args preview stays deferred 1.3 |

---

## 1. Ranked Run-2 work packages

Ranking = dependency order, then risk. `P0` blocks everything below; `P1` proof/UX; `P2` paid proof. One writer per file set; each wave commits before the next wave touches an overlapping file.

| Rank | WP | Scope | P | Depends on |
|---:|---|---|---|---|
| 1 | R2-P0 | Commit/reconcile WP-01 + waiter lane; freeze baseline, decisions | P0 | none |
| 2 | R2-01 | Side-effect executor (`file_write`) | P0 | R2-P0 |
| 3 | R2-02 | Engine suspension/resume/wait/events (14 tests) | P0 | R2-P0, R2-01 interface decision |
| 4 | R2-03 | Backend replay projection (WP-05, no Run-2 report) | P1 | R2-02 |
| 5 | R2-04 | Deterministic matrix + mocked ordering proof | P0 | R2-02, R2-01 |
| 6 | R2-05 | Web replay + retry/refusal state machine | P1 | R2-03 (real replay); mocked cases can run earlier |
| 7 | R2-06 | Live tool-turn + live approval proof (paid) | P2 | R2-01, R2-02, R2-04, R2-05 |
| 8 | R2-07 | Integration gate + evidence receipt (Lead) | P1 | all above |

### R2-P0 — Prerequisite commit, waiter reconciliation, decision freeze
- **Target files (commit only; no new code):** `backend/crates/hiverouter-tools/src/approval.rs`, `backend/crates/hiverouter-core/src/tools_bridge.rs`, `backend/crates/hiverouter-tools/src/lib.rs`, `backend/crates/hiverouter-core/src/lib.rs`, `backend/crates/hiverouter-core/src/approval_wait.rs` (untracked), plus deletion/decision on untracked `tools/qa/apply_wp01.py`. Reports stay untracked/read-only.
- **Language:** Rust.
- **Exact symbols to verify present after commit:** `approval::{expire, cancel_pending, decision_label, count_for_run}`; `CREATE UNIQUE INDEX IF NOT EXISTS approvals_one_pending_per_call ON approvals(run_id, tool_call_id) WHERE decision IS NULL`; `ToolsBridge::{with_approval_ttl, expire_approval, cancel_approval, approval_decision, approval_count_for_run, verify_tool_grant}`; `ApprovalWaiters`/`ApprovalWake` (either in `approval_wait.rs` as observed, or folded into `execution.rs` by Lead decision; one definition only).
- **Dependencies:** none; this wave is the merge base.
- **Verification commands:** `git rev-parse HEAD; git status --porcelain`; `git show --stat HEAD`; `cargo check --workspace --tests --locked`; `cargo test -p hiverouter-tools --lib --locked`; `cargo test --workspace --locked`; `git diff --check`; `Select-String` the symbols. Record exact HEAD, clean-tree proof, and the exact post-WP-01 test count (not 396).
- **Acceptance:** tree clean after commit(s); no untracked product residue; symbols present exactly once; full workspace test run passes and its exact N/0/ignored is quoted; no report file modified; WP-01 ownership recorded; 396 baseline superseded by a fresh number.
- **Rollback:** `git revert` the merge-base commit(s); WP-01-only revert restores Run-1 behavior (`E_TOOL_NOT_ALLOWED` side effects); delete `approval_wait.rs` if the module decision is to discard (do not leave an unowned module).
- **Blockers:** **X-R2-01** waiter placement conflict (observed `approval_wait.rs` vs E inline H2); **X-R2-07** baseline composition (396 predates final WP-01 tests); live WP-01 lane still writing files at compile; no clean-tree evidence yet. Also decide **X-R2-02/C1a** interface and **X-R2-03/C2** rollout flag in the same freeze.

### R2-01 — Side-effect executor, `file_write` wave (Run-1 WP-02; report S)
- **Target files:** new `backend/crates/hiverouter-tools/src/side_effects.rs`; `backend/crates/hiverouter-tools/src/jail.rs` (append `write_atomic` after `write:223`); `backend/crates/hiverouter-tools/src/lib.rs` (`pub mod side_effects; pub use side_effects::{...}`); tests inside `side_effects.rs`/`jail.rs`. **No edits** to `approval.rs`, `tools_bridge.rs`, `executor.rs`, `shell.rs`, `tool.rs`, core, contract, db, web.
- **Language:** Rust (std fs, `uuid::Uuid::now_v7`, existing `shell.rs` untouched).
- **Exact symbols:** `side_effects::MAX_WRITE_BYTES: usize = 1_048_576`; `MAX_SIDE_EFFECT_TEXT_BYTES: usize = 65_536`; `pub struct PreparedWrite { private target/display_path/bytes/sha256 }` + redacted `Debug`; `pub enum PreparedSideEffect { FileWrite(PreparedWrite) }`; `pub fn preflight_side_effect(jail: &WorkspaceJail, registry: &ToolRegistry, tier: AgentTier, tool: &str, args: &serde_json::Value, cancellation: Option<&CancellationToken>) -> Result<PreparedSideEffect, ToolError>`; `fn preflight_file_write(...) -> Result<PreparedWrite, ToolError>`; `pub async fn commit_side_effect(jail: &WorkspaceJail, prepared: PreparedSideEffect) -> Result<ExecutedTool, ToolError>`; `fn commit_file_write(...)`; `fn bounded_text(...)`; `WorkspaceJail::write_atomic(&self, requested: &str, bytes: &[u8]) -> Result<PathBuf, ToolError>` (same-dir `create_new` temp → `write_all` → `sync_all` → `rename`, cleanup on error, root/dir/non-regular/symlink refusals, parent containment).
- **Dependencies:** R2-P0 (`ToolsBridge::verify_tool_grant` live); no engine caller in this wave (dead code until R2-02 — do not claim reachability).
- **Verification commands:** `cd backend`; `$env:PATH = "$env:USERPROFILE\.cargo\bin;$PWD\..\tools\w64devkit\bin;$env:PATH"`; `cargo test -p hiverouter-tools --lib jail::tests --locked`; `cargo test -p hiverouter-tools --lib side_effects::tests --locked`; `cargo test -p hiverouter-tools --lib --locked`; `cargo test --workspace --locked`.
- **Acceptance:** the 8 side-effect tests in S §7 pass (preflight refuses read/unknown/shell; doomed write does not consume grant; commit writes/reports path/size/sha without content; atomic replace without temp residue; stale hash match/mismatch; grant consumed exactly once; cancelled token refuses before grant; bounded text on char boundary) + 5 `write_atomic` tests in `jail.rs`; preflight-before-grant ordering proven; frozen 65 refusal codes only; no content/argv/file-body in results/events/ledger/logs; no new dependency.
- **Rollback:** delete `side_effects.rs`, remove `lib.rs` exports and the `jail.rs` hunk; no caller, no persisted state, no schema change.
- **Blockers:** **X-R2-02/C1a** interface (engine's `execute_approved` callback vs this split API — S does **not** implement `execute_approved`); **X-R2-03/C2** default-off flag/`HIVEROUTER_SIDE_EFFECTS` decision; **X-R2-04** `expected_sha256` mismatch code (S chooses `E_SCHEMA_INVALID`; R2 proposed `E_TOOL_SCHEMA` — Lead must confirm); U-R2-11 Windows rename-over-existing proof; U-R2-06 shell policy deferred; `tools/src/lib.rs` collision with WP-01 commit must be resolved first.

### R2-02 — Engine suspension / resume / wait / event wiring (Run-1 WP-03; report E)
- **Target files:** `backend/crates/hiverouter-core/src/execution.rs`, `core/src/tool_loop.rs`, `core/src/tools_bridge.rs`, `core/src/ws.rs`, `core/src/http.rs`; `core/src/lib.rs`/`approval_wait.rs` only to reconcile the existing WIP waiter module; new `core/tests/suspension.rs`; reconcile `core/tests/tool_loop_engine.rs` and `core/tests/run_lifecycle.rs` (H12 three sites only). Do **not** edit `run_state.rs` (transition authority), `db.rs`, `main.rs`, `recovery.rs`, contract, web, tools crate.
- **Language:** Rust (tokio `oneshot`/`select!`, `run_state::apply_run/apply_task`).
- **Exact symbols (H1–H12):** `tool_loop::MAX_APPROVAL_WAITS_PER_RUN: u32 = 8`; `execution::Clock`; `ApprovalWake`/`ApprovalWaiters` (reuse `core/src/approval_wait.rs` if kept; do not define twice); `RunEngine` fields `tools: Option<Arc<ToolsBridge>>`, `waiters`, `clock`, `shell_policy`; builders `with_tools/with_clock/with_shell_policy`; `notify_approval_resolved(&ApprovalResolution) -> bool`; async `execute_tool_turn`; `suspend_for_approval`; `apply_committed_decision`; `finish_timeout_wait`; `finish_cancelled_wait`; `resume_and_execute`; `resume_denied`; `persist_approval_transitions`; `resume_transitions`; `ArmedApproval` RAII; `dispatch` terminal-owner guard (D7); `execute_task` await + double-failure guard; `dispatch_inner` `Planning→Running` once; `cancel_run`/`kill_worker` awaiting branches; `ws.rs` wake after committed `APPROVAL_RESOLVE` (both ACK/`APPROVAL_RESOLVED` path), log false, never fail command; `http.rs` build tools before engine and inject bridge into all three `AppState` constructors; `ToolsBridge::execute_approved` **only if** C1a Option A is chosen (else engine calls preflight→verify→commit from S §6).
- **Dependencies:** R2-P0; R2-01 (for one-effect resume); **C1a interface fixed before H6**; **C2 flag decision before H11**; waiter-module ownership fixed before H2/H3.
- **Verification commands:** `cargo check --workspace --tests --locked`; then E §7.3: `cargo test -p hiverouter-core --test suspension --locked`; `--test tool_loop_engine`; `--test run_lifecycle`; `--test approval_ledger`; `--test ws_transport`; `cargo test -p hiverouter-tools --lib --locked`; `cargo test --workspace --locked`.
- **Acceptance:** the 14 named tests in E §7.1 / P S1–S14 pass (suspend publishes one `APPROVAL_REQUEST` with all six fields; approve executes exactly once; deny continues; timeout `E_APPROVAL_TIMEOUT`; cancel/kill consume with exactly one `TOOL_RESULT`; CAS-loss resumes once; concurrent resolve one winner; second prompt `E_RUN_ACTIVE`; restart interrupted+expired, later resolve `E_APPROVAL_TIMEOUT`, no auto-approve; 9th wait `E_CALL_LIMIT`; wrong args `E_APPROVAL_REQUIRED` before effect; shell allowlist; no-bridge keeps `E_TOOL_NOT_ALLOWED`); exactly one terminal frame; waiter map empty; no new refusal/event/command; engine never writes SQLite/ledger directly; no crash-resume claim.
- **Rollback:** absence of `.with_tools` restores read-only refusal (E D3); waiter types additive; H1 cap additive; H11 injection revertible; no schema change (WP-01 index additive).
- **Blockers:** **X-R2-01** waiter placement (observed module vs E inline) and **X-R2-02** C1a; **X-R2-03** C2; **X-R2-09** suspension.rs ownership collision with WP-04; U-R2-4 timer flakiness; U-R2-3 frame-count tolerance; U-R2-5 `execute_approved` not in S blueprint; current untracked waiter lane must be adopted or removed, never duplicated.

### R2-03 — Backend pending-approval replay projection (Run-1 WP-05; **no Run-2 code report — design lifted from W §4**)
- **Target files:** `backend/crates/hiverouter-tools/src/approval.rs` (`pending_ids_for_active_runs`, make `load_request` public; read-only, no nonce consumption); `backend/crates/hiverouter-core/src/tools_bridge.rs` (`pending_approval_events_all`); `backend/crates/hiverouter-core/src/ws.rs` (replay after hello); new `backend/crates/hiverouter-core/tests/approval_replay.rs`.
- **Language:** Rust (rusqlite read-only, existing `approval_request_event`, EventBus publish).
- **Exact symbols:** `approval::pending_ids_for_active_runs(conn: &Connection, limit: usize) -> Result<Vec<String>, ToolError>`; `pub fn load_request(...)`; `ToolsBridge::pending_approval_events_all(&self, limit: usize) -> Result<Vec<ApprovalRequest>, Refusal>`; `MAX_APPROVAL_REPLAY_EVENTS: usize = 20`; `publish_pending_approval_replays(state)`; `send_hello_replay_and_approvals(...)` called from both hello paths (`ws.rs:147` initial, `:316` re-hello/resync); ordering: `SERVER_HELLO` before replayed `APPROVAL_REQUEST` on the same socket.
- **Dependencies:** R2-P0; R2-02 producer/notify; W §4 contract (six optional fields exactly as `approval_request_event`).
- **Verification commands:** `cargo test -p hiverouter-core --test approval_replay --locked` (new); `--test ws_transport`; `--test approval_ledger`; `cargo test --workspace --locked`; mocked e2e reconnect case from W H12-1.
- **Acceptance:** bounded 20; oldest-first; non-terminal/`decision IS NULL` only; no expired rows; no nonce consumption; idempotent by `approval_id`; no duplicate replay; `hello{last_seq}` ordering preserved; no `RESYNC_REQUIRED` loop; real client replay confirms once with zero commands; fail-closed when WP-05 absent.
- **Rollback:** remove projection functions/replay call; hello path reverts to `send_hello_and_replay`; additive, no schema change.
- **Blockers:** **X-R2-05 (missing Run-2 report)** — no detailed test names/edge cases beyond W §4; U-R2-6 replay ordering/seq policy and "all active runs vs latest-run" Lead decision; U-R2-5 (Playwright reconnect support) affects only e2e proof, not server logic; file collision with R2-02 (`ws.rs`/`tools_bridge.rs`) requires sequencing after R2-02 commit.

### R2-04 — Deterministic suspension matrix + mocked ordering proof (Run-1 WP-04; report P)
- **Target files:** `backend/crates/hiverouter-core/tests/tool_loop_matrix.rs` (new, 5 tests); `backend/crates/hiverouter-core/tests/approval_ledger.rs` (+1 test, 2 → 3); `e2e/specs/approval-flow.spec.ts` (+2 ordering cases); evidence logs `docs/evidence/1.2-wp04-{suspension,matrix}.log`. **`suspension.rs` is owned by R2-02 (E H12), not this wave** (collision C-R2-09); this wave runs/records it, never writes it.
- **Language:** Rust (wiremock SSE, mocked executor; no paid path), TypeScript/Playwright (mocked WS).
- **Exact symbols:** matrix test names `tool_loop_matrix_read_only_call_roundtrip`, `tool_loop_matrix_turn_cap_refusal`, `tool_loop_matrix_call_cap_refusal`, `tool_loop_matrix_invalid_arguments_refused`, `tool_loop_matrix_non_read_tool_refused_by_name`; ordering e2e names `'suspension order: APPROVAL_RESOLVED -> TOOL_RESULT -> REPLY_DELTA -> RUN_STATUS completed'`, `'no TOOL_RESULT may precede APPROVAL_RESOLVED for an approval-gated call'`.
- **Dependencies:** R2-01 + R2-02 committed; `suspension.rs` owner green.
- **Verification commands:** P §1.6 full command block (`suspension`, `tool_loop_matrix`, `approval_ledger`, `tool_loop_engine`, `run_lifecycle`, `ws_transport`, `recovery_gates`, tools lib, workspace); `cd e2e; node run-guarded.mjs --project=chrome specs/approval-flow.spec.ts specs/tool-loop.spec.ts`.
- **Acceptance:** matrix 5/5; suspension 14/14 (owner's file); ledger 3/3; tool_loop_engine 11/11; run_lifecycle 7/7; ws_transport 13/13; recovery_gates 3/3; workspace 0 failed with floor **≥416** (396 + 14 + 5 + 1) and record exact N; mocked e2e expected **18** per P D4 (8 tool-loop + 10 approval-flow) — but see C-R2-05: record exact observed count, never force it; zero skips/no paid.
- **Rollback:** tests are additive; revert matrix/ledger/e2e files; no production behavior change.
- **Blockers:** **X-R2-05** mocked count conflict (8+8 today; WEB adds 6 cases → approval-flow 14, total 22; P expects 10/18); **X-R2-09** suspension.rs ownership; U-R2-8 exact post-WP counts; U-R2-9 floor/expected counts; no execution by planner.

### R2-05 — Web replay + retry/refusal state machine (Run-1 WP-06; report W)
- **Target files:** new `web/src/state/approval.ts`; `web/src/state/store.ts`; `web/src/panels/ApprovalOverlay.tsx` (list/mount only; delete local lock); new `web/src/panels/ApprovalCard.tsx`; new `web/src/panels/ApprovalStatusStrip.tsx`; `web/src/state/strings.ts`; new `web/tests/approval.state.test.ts`; extend `web/tests/approval.integration.test.ts`; new `web/tests/approvalPanels.test.ts`; update `agentsToolActivityMount.test.ts` + `toolActivityPanels.test.ts`; `e2e/helpers/replay-harness.ts`; `e2e/specs/approval-flow.spec.ts` (W H12 cases; coordinate with R2-04 ordering owner). `web/src/App.tsx` unchanged.
- **Language:** TypeScript 7 / React 19 / zustand 5 / Vitest / Playwright.
- **Exact symbols:** constants `APPROVAL_REPLAY_SETTLE_MS=400`, `MAX_APPROVAL_REPLAY_EVENTS=20`, `MAX_APPROVAL_RESOLVE_ATTEMPTS=3`, `APPROVAL_ACK_SETTLE_MS=2000`, `APPROVAL_RESOLVE_TIMEOUT_MS=15000`; types `CommandOutcome = {ok:true}|{ok:false;refusal:Refusal}`, `ApprovalResolveOutcome`, `ApprovalResolveState`, `ApprovalDisplayStatus`, `ApprovalFailureKind`; functions `initialApprovalResolveState`, `classifyApprovalRefusal`, `approvalDisplayStatus`, `approvalActionsEnabled`; store state/actions `approvalResolve`, `approvalSeenEpoch`, `approvalsStale`, `approvalReplayEpoch`, `approvalReplayWindow`, `lastApprovalReplayAt`, `approvalReplaySource`, `hydrateApprovals`, `markApprovalsStale`, `beginApprovalReplay`, `settleApprovalReplay`, `expireApproval`, `dismissApproval`, `retryApproval`, typed `run()`/`resolveApproval()`; overlay exports preserved except `canResolveApproval`/`prunePendingResolutions` removed (grep all imports); helper exports `closeMockSocket`, `waitForReconnect`, `helloFrameCount`, `sendServerHello`, `sendErrorForCommand`; flag `VITE_UI_APPROVAL_REPLAY` (`'0'` restores legacy).
- **Dependencies:** R2-03 for real replay; mocked e2e can run before it; no backend/contract/event/command/refusal change; args preview out of scope (WP-07=live proof; preview 1.3).
- **Verification commands:** `cd web; npm.cmd run typecheck`; `npm.cmd test -- --run tests/approval.state.test.ts tests/approval.integration.test.ts tests/approvalPanels.test.ts`; `npm.cmd test -- --run` (147 baseline + new, 0 failed); `npm.cmd run build`; then `cd e2e; node run-guarded.mjs --project=chrome specs/approval-flow.spec.ts specs/tool-loop.spec.ts`.
- **Acceptance:** no auto-send on hydrate/replay/settle; ACK never terminal; only `APPROVAL_RESOLVED` removes; retry bounded 3, same id/nonce/new correlation; E_APPROVAL_REPLAY/NOT_FOUND → stale; E_APPROVAL_TIMEOUT/REQUIRED → expired; E_APPROVAL_DECISION_INVALID → retryable; overlay lock removed; `data-approval-status`/status strip/refusal verbatim; all existing selectors keep matching; timers cleared on dispose/resolved; all copy via `STRINGS`; flag rollback works.
- **Rollback:** `VITE_UI_APPROVAL_REPLAY=0` restores pre-WP-06 behavior; revert H1–H6 + tests as one change set; no DB/contract/event changes; additive component files.
- **Blockers:** U-R2-5 Playwright `WebSocketRoute.close`/reconnect support (spike first; reload fallback only); U-R2-6 server replay policy; U-R2-8 e2e count; `approval-flow.spec.ts` shared with R2-04; U-R2-3 `run()` call-site anchors may drift after WP-01 commit.

### R2-06 — Live tool-turn + live approval proof (Run-1 WP-07; report P; PAID-GATED)
- **Target files:** `e2e/helpers/live-server.ts` (extend only); new `e2e/specs/live-tool-turn.spec.ts`; new `e2e/specs/live-tool-approval.spec.ts`; evidence `docs/evidence/1.2-wp07-live-tool-turn.{json,log}` and `1.2-wp07-live-approval.{json,log}` (redacted).
- **Language:** TypeScript/Playwright + Node child-process harness.
- **Exact symbols:** `paidGate(mode: 'prompt'|'tool-turn' = 'prompt')` (backward-compatible; `'tool-turn'` requires `LIVE_TOOL_TURN=1` **and** `DEEPSEEK_API_KEY`); `StartLiveServerOptions` adds `env?`, `seedWorkspace?`; `LiveServer.workspaceRoot: string`; tests T1 gate-skip, T2 live read-only turn (`TOOL_CALL` → `TOOL_RESULT ok` → `RUN_STATUS completed`, exactly one `li[data-outcome="ok"]`), A1 gate skip, A2 approve `file_write` exactly once, A3 deny no file; `HIVEROUTER_SIDE_EFFECTS=1` only if C2 chooses default-off.
- **Dependencies:** R2-04 deterministic green + Lead rebuild of a 1.2-dev release binary; R2-02/03 for suspension/approval; R2-05 for UI; operator key/budget.
- **Verification commands:** P §2.5 (`$env:E2E_SERVER_BIN`, `LIVE_TOOL_TURN=1`, key only process env; `.node_modules\.bin\playwright.cmd test specs/live-tool-turn.spec.ts specs/live-tool-approval.spec.ts --project=chrome --reporter=list`); negative gate: `node run-guarded.mjs --project=chrome` with `LIVE_TOOL_TURN` unset → expected exit 1 and named skips, no server start.
- **Acceptance:** with all gates/code: **5 passed** (T1/T2/A1/A2/A3; 3 paid prompts), 0 failed/skipped; without gate/key: T1/A1 pass, T2/A2/A3 named skips + guard exit 1; without WP-02/03: A2/A3 skip `SIDE_EFFECTS_NOT_ENABLED` (do not fake); redacted evidence per test; zero key material (`sk-[A-Za-z0-9]{16,}` scan = 0); stop on first paid failure.
- **Rollback:** delete new specs/evidence; revert live-server extensions; no product change.
- **Blockers:** paid key/budget/operator; 1.2-dev binary rebuild from clean tree; **X-R2-02/03** interface/flag; if WP-02/03 absent live approval stays skipped; do not claim live timeout/crash-resume/cross-process exactly-once/args preview/trace/release.

### R2-07 — Integration gate + evidence receipt (Lead)
- **Target files:** no product files; evidence `docs/evidence/1.2-run2-*.log`; wave receipt; reports untouched.
- **Language:** PowerShell/commands + Markdown/JSON receipts.
- **Exact symbols:** command matrix below; receipt fields HEAD-before/after, `git status`, command, exit code, exact `test result` lines, counts.
- **Dependencies:** all waves.
- **Verification commands (ordered):** `git rev-parse HEAD; git status --porcelain`; D1 backend suite (P §1.6); D2 `cargo test -p hiverouter-tools --lib --locked; cargo test --workspace --locked`; D3 web `typecheck`, `test -- --run`, `build`; D4 mocked e2e (18 expected; record exact); D5 gui/a11y mocked (0 failed; exclusions stay named); D6 `node tools/qa/verify-harness.mjs`; `node tools/qa/check-docs-truth.mjs`; `run-gates.ps1 -DryRun`; D7 negative paid gate; D8 binary hash/version; D9 only then paid live (R2-06).
- **Acceptance:** all deterministic gates 0 failed; workspace count recorded and ≥416 (post-WP-04) with exact N; 147 web baseline preserved + new; mocked e2e exact observed; docs checker/harness counts re-quoted (RUN-1 says 19/19, 27/27 — re-run, do not trust); negative paid gate proves no spend; no release/seal/trace/G-UI claims; receipt distinguishes VERIFIED/READ/UNVERIFIED.
- **Rollback:** receipts/docs only; revert no product.
- **Blockers:** U-R2-8/9 counts; U-R2-13 checker counts not run; U-R2-10 binary/paid; any red gate stops live phase (P stop rule).

---

## 2. Paste-ready handoff prompt (Lead → next coding wave)

```text
You are the next implementation wave for HiveMind 1.2 approval closure. READ-ONLY planning is complete; you may edit ONLY the files assigned to your wave. Do not edit reports, other lanes' files, contract/DB/gates/tags, or unowned files.

REPO: C:\Users\Riley\Desktop\hiverouter-v1  (Windows, PowerShell 5.1)
PATH: cd backend; $env:PATH = "$env:USERPROFILE\.cargo\bin;$PWD\..\tools\w64devkit\bin;$env:PATH"
BASELINE: HEAD 90cb7a7 + uncommitted WP-01 work (tools/src/approval.rs, core/src/tools_bridge.rs, tools/src/lib.rs, core/src/lib.rs) + untracked core/src/approval_wait.rs + untracked reports + tools/qa/apply_wp01.py. Baseline evidence: docs/evidence/1.2-backend-tests-6.log = 396 passed / 0 failed / 2 ignored, 39 suites, EXIT=0 (pre-final-WP-01; RUN-1.md:21). Do NOT treat 396 as the post-WP-01 count.
READ FIRST (full): plans/foresight/RUN2-COMPILED-LEDGER.md, RUN2-ENGINE-CODE.md, RUN2-SIDE-EFFECT-CODE.md, RUN2-WEB-CODE.md, RUN2-PROOF-CODE.md. Re-hash reports; re-grep symbols before every edit.
LEAD PREREQUISITE (before any agent edit): commit WP-01; decide and record (a) waiter placement — adopt existing core/src/approval_wait.rs (ApprovalWaiters/ApprovalWake, 3 tests) or fold into execution.rs, never both; (b) C1a interface — prefer report S split API: preflight_side_effect -> ToolsBridge::verify_tool_grant -> commit_side_effect (report E Option A callback requires WP-02 to expose execute_approved, which report S does not build); (c) C2 rollout — bridge-presence vs default-off HIVEROUTER_SIDE_EFFECTS; (d) expected_sha256 mismatch code (S chooses E_SCHEMA_INVALID; R2 proposed E_TOOL_SCHEMA). Record HEAD, clean tree, and fresh test count before wave 1.

COLLISION-FREE WAVES (one writer per file set; commit between waves; never run overlapping waves concurrently):
  WAVE-T1 (tools): NEW tools/src/side_effects.rs, tools/src/jail.rs (write_atomic append), tools/src/lib.rs (module/exports), tests in those files. Do NOT touch approval.rs (WP-01), tools_bridge.rs, core, contract, db, web.
  WAVE-ENGINE (core): core/src/execution.rs, tool_loop.rs, tools_bridge.rs, ws.rs, http.rs; use/reconcile core/src/approval_wait.rs (single definition); NEW core/tests/suspension.rs; reconcile core/tests/{tool_loop_engine.rs,run_lifecycle.rs} only at the three H12 sites. Depends WAVE-T1 + Lead decisions. Do NOT touch tools/src/approval.rs, side_effects.rs, jail.rs, contract, db, web.
  WAVE-REPLAY (backend, after WAVE-ENGINE commit): tools/src/approval.rs (pending_ids_for_active_runs; make load_request pub), core/src/tools_bridge.rs (pending_approval_events_all), core/src/ws.rs (replay after hello), NEW core/tests/approval_replay.rs. Same files as ENGINE — sequential, single writer.
  WAVE-WEB (after WAVE-REPLAY for real replay; mocked cases may run against mock earlier): web/src/state/approval.ts NEW, store.ts, panels/{ApprovalOverlay.tsx + ApprovalCard.tsx NEW + ApprovalStatusStrip.tsx NEW}, strings.ts, web/tests/*, e2e/helpers/replay-harness.ts, e2e/specs/approval-flow.spec.ts (coordinate ordering cases with PROOF owner). No backend edits.
  WAVE-PROOF (after ENGINE/WEB): new core/tests/tool_loop_matrix.rs (5), approval_ledger.rs (+1), e2e ordering cases, e2e/helpers/live-server.ts + NEW live-tool-turn/live-tool-approval specs, evidence files. Paid only with operator flags; no paid call otherwise.
  WAVE-GATE (Lead): run the full ordered command matrix; quote exact HEAD/status/counts; stop before live if any deterministic gate fails.

EXACT COMMANDS (quote output; no “should work”):
  cargo check --workspace --tests --locked
  cargo test -p hiverouter-core --test suspension --locked
  cargo test -p hiverouter-core --test tool_loop_matrix --locked
  cargo test -p hiverouter-core --test approval_ledger --locked
  cargo test -p hiverouter-core --test tool_loop_engine --locked
  cargo test -p hiverouter-core --test run_lifecycle --locked
  cargo test -p hiverouter-core --test ws_transport --locked
  cargo test -p hiverouter-core --test recovery_gates --locked
  cargo test -p hiverouter-tools --lib --locked
  cargo test --workspace --locked
  cd web; npm.cmd run typecheck; npm.cmd test -- --run; npm.cmd run build
  cd e2e; node run-guarded.mjs --project=chrome specs/tool-loop.spec.ts specs/approval-flow.spec.ts specs/gui-gates.spec.ts specs/gui-residual.spec.ts specs/a11y-gates.spec.ts
  node tools/qa/verify-harness.mjs; node tools/qa/check-docs-truth.mjs; powershell -NoProfile -ExecutionPolicy Bypass -File tools/qa/run-gates.ps1 -DryRun
  # paid ONLY after all above green, operator present, key only in process env:
  $env:LIVE_TOOL_TURN='1'; $env:DEEPSEEK_API_KEY='<process env>'; $env:E2E_SERVER_BIN='<repo>\target\release\hiverouter.exe'
  cd e2e; .\node_modules\.bin\playwright.cmd test specs/live-tool-turn.spec.ts specs/live-tool-approval.spec.ts --project=chrome --reporter=list

EXPECTED OUTPUTS (record exact; never force): suspension 14/14; matrix 5/5; approval_ledger 3/3; tool_loop_engine 11/11; run_lifecycle 7/7; ws_transport 13/13; recovery_gates 3/3; workspace 0 failed, N ≥416 floor (expected ≥429 after WP-01/02 units; record N); web 147 baseline + new, 0 failed; mocked e2e: exact observed (PROOF says 18; WEB adds 6 replay cases making 22 — record actual, do not flatten); negative paid gate = named skips + guard exit 1 + no server start; all-gates live = 5 passed, 0 skipped, 3 paid prompts; without WP-02/03 = 3 passed/2 named skips (A2/A3 not executed).

DO NOT CLAIM: WP-01 is committed/tested (it is WIP and unrun); 396 is the current count; suspension/approval/side effects work; crash-resume or cross-process exactly-once; shell_exec works; web reconnect works; live tool turn/approval was proven without quoted redacted evidence; trace promotion (truth stays 1/264/5/49 of 319, raw capture empty); G-UI/axe closure; 1.2 release/tag/artifacts; args preview in 1.2; C1a resolved without a Lead record; any count you did not quote.
REPORT SHAPE: wave + HEAD before/after + clean-tree proof; files changed with exact symbols; commands with quoted pass/fail/skip; VERIFIED vs READ vs INFERRED vs UNVERIFIED; contradictions/blockers; rollback; explicit non-claims. Lead owns commits.
```

---

## 3. Claim / evidence table (evidence-check)

| ID | Claim | Evidence (path:line / command / log) | Status |
|---|---|---|---|
| C-R2-01 | Run-1 baseline is 396 passed/0 failed/2 ignored backend | `docs/evidence/1.2-backend-tests-6.log` (39 suites, EXIT=0, 18:56:23); `RUN-1.md:21` | VERIFIED (read) |
| C-R2-02 | 396 baseline is pre-final-WP-01; WP-01 adds tests | log has 14 `approval::tests`, no `expire_cancel_and_decision_labels_are_cas_guarded`; `approval.rs:1069`; current `#[test]` count 16 | VERIFIED (read/log); post-WP-01 count UNVERIFIED |
| C-R2-03 | WP-01 primitives are implemented in the working tree, uncommitted | `approval.rs:572,588,606,621,163`; `tools_bridge.rs:106,115,126,141,152,166`; `tools/src/lib.rs:50`; `git status`/`git diff --stat` | VERIFIED (read/WIP); commit/test status **UNVERIFIED** |
| C-R2-04 | WP-01 unique pending index exists | `approval.rs:163` (`approvals_one_pending_per_call`, partial `WHERE decision IS NULL`) | VERIFIED (read WIP) |
| C-R2-05 | A separate waiter module now exists but is unowned/unwired | `core/src/approval_wait.rs` 6,243 B/174 lines, mtime 19:00:37, `ApprovalWaiters`/`ApprovalWake` + 4 tests; `core/src/lib.rs:8,33`; no `notify_approval_resolved`/engine caller | VERIFIED (read WIP); ownership UNKNOWN |
| C-R2-06 | No side-effect executor exists yet | `tools/src/side_effects.rs` absent; `write_atomic`/`execute_approved` = 0 occurrences; `executor.rs:52` only `execute_read_only` | VERIFIED (read/grep) |
| C-R2-07 | `jail.write` is non-atomic; no `write_atomic` | `jail.rs:223-243`; grep 0 | VERIFIED (read) |
| C-R2-08 | `verify_grant` is the single-use authority; docs-vs-code timeout mismatch (R6) | HEAD `approval.rs:772`, WIP `:741`; `UPDATE … executed_at IS NULL`; S R6 records doc-comment claims timeout but code returns `E_APPROVAL_REQUIRED` | VERIFIED (read); risk R6 |
| C-R2-09 | Engine still has no suspension; non-read refusal remains | `execution.rs:684,879` (HEAD); no `execute_tool_turn` async; no `ApprovalWaiters` in execution (only new module); `run_state` grep in execution = 0 | VERIFIED (read) |
| C-R2-10 | WS resolve does not wake an engine; no approval replay in hello | `ws.rs:465-483`; no `notify_approval_resolved`; `pending_approval_events` callers tests only | VERIFIED (read/grep) |
| C-R2-11 | Approval producer has no production caller | grep `request_model_tool_approval`: definition/doc/tests in `tools_bridge.rs` only; no execution caller | VERIFIED (grep) |
| C-R2-12 | Run-2 E report anchors on HEAD 90cb7a7 | `execution.rs:260,684,879,968`; `approval.rs:772,849`; `ws.rs:465` | VERIFIED (HEAD read) |
| C-R2-13 | Current test-file counts are 11/7/2/13/3 | `#[tokio::test]/#[test]` grep on named files | VERIFIED (grep) |
| C-R2-14 | No suspension/matrix/live specs exist | file inventory: `suspension.rs`, `tool_loop_matrix.rs`, `live-tool-turn.spec.ts`, `live-tool-approval.spec.ts` absent | VERIFIED |
| C-R2-15 | Live harness has only `paidGate()`/`LIVE_ONE_PROMPT`; no `LIVE_TOOL_TURN`/seed/env | `live-server.ts:115`; grep `LIVE_TOOL_TURN` = 0; `StartLiveServerOptions` lacks env/seedWorkspace | VERIFIED (read/grep) |
| C-R2-16 | Mocked e2e today = 8 tool-loop + 8 approval-flow | `tool-loop.spec.ts` 8, `approval-flow.spec.ts` 8 | VERIFIED (grep) |
| C-R2-17 | Web store swallows typed refusals; overlay local lock can stick | `store.ts:565-596`; `ApprovalOverlay.tsx:167,183-215` (W §0 B3/B6) | VERIFIED (read report/code) |
| C-R2-18 | No backend pending-approval replay projection exists | grep 4 symbols = 0; only per-run `pending_approval_events` (W B14) | VERIFIED (read/grep) |
| C-R2-19 | Pending approval projection pieces exist (`pending_for_run`, private load, per-run events) | `approval.rs:710`, `:213`; `tools_bridge.rs:432,410`; hello sites `ws.rs:147,316` | VERIFIED (read) |
| C-R2-20 | Web test baseline is 147/147; browser baseline 16 passed | `RUN-1.md:22-23` | VERIFIED (read record; not rerun) |
| C-R2-21 | Docs checker 19/19 and harness 27/27 claimed by Run-1 | `RUN-1.md:24`; P D6 repeats as expected | READ/UNVERIFIED (not run by compiler) |
| C-R2-22 | Trace truth remains 1/264/5/49 of 319; raw capture empty | `docs/trace/controls.json:_meta`; `trace-capture.json` 286 B `surfaces: []` | VERIFIED (read) |
| C-R2-23 | Run-2 reports are untracked and were frozen at compile hashes | `git status` (4 `??` RUN2 files); hashes in §Inputs | VERIFIED (read) |
| C-R2-24 | WP-02/03/04/05/06/07 have no implementation on HEAD | inventories above; no `side_effects.rs`; no engine wiring; no web changes | VERIFIED (read) |
| C-R2-25 | `expected_sha256` has no dedicated refusal code | ledger C-SE-03/C13; `tool.rs:255-263`; S §5 mapping | VERIFIED (read); semantic gap |
| C-R2-26 | Live proof currently unreachable (no suspension/executor) | P §2.1; X-R2-09/10; A2/A3 would skip | VERIFIED (read) |
| C-R2-27 | 396/147/16 are Run-1 baselines, not Run-2 results | `RUN-1.md:21-23`; no Run-2 gate log | READ (record); Run-2 execution UNVERIFIED |
| C-R2-28 | Run-2 reports observed inconsistent clean/dirty baselines | E/W say clean at recon; S/P say dirty (WP-01 WIP); current tree dirty | VERIFIED (read reports + git status) |
| C-R2-29 | No build/test/paid run by the compiler | this task; only this file written | VERIFIED (scope) |

---

## 4. Contradictions preserved (Lead must resolve; do not silently merge)

| ID | Contradiction | Recommended resolution |
|---|---|---|
| X-R2-01 | **Waiter placement:** E H2 defines `ApprovalWaiters`/`ApprovalWake` inline in `execution.rs`; the live tree now has a separate untracked `core/src/approval_wait.rs` (+4 tests, exported from `lib.rs`). | Adopt the existing module and import it in `execution.rs` (no duplicate type); if E wants inline, delete the module before committing. Record owner/decision in R2-P0. |
| X-R2-02 | **C1a grant-check interface:** E §3.3 recommends a bridge `ToolsBridge::execute_approved` + tools `side_effects::execute_approved` callback; S explicitly builds a **split** API (`preflight_side_effect` → caller `verify_tool_grant` → `commit_side_effect`) and does **not** expose `execute_approved`. | Adopt S split API; engine H6 calls preflight→verify→commit (R1 D10 order) and does not require a tools `execute_approved`. If Option A is chosen instead, S must add the callback symbol (not in its blueprint) and E H6 must compile against it. Do not claim preflight-before-grant if the fallback burns consent (C1a-B). |
| X-R2-03 | **C2 rollout:** E keeps engine identical and gates only H11 `.with_tools`; S has no flag; P/W expect optional `HIVEROUTER_SIDE_EFFECTS=1` for live tests. | Lead decides default-off env flag vs always-wired. Engine semantics must not change; only `http.rs` wiring (and live-test env) differ. Record the decision before H11/live proof. |
| X-R2-04 | **`expected_sha256` mismatch code:** S maps stale/malformed to `E_SCHEMA_INVALID` (ledger choice); R2 proposed `E_TOOL_SCHEMA` (S U-A). | Confirm ledger mapping `E_SCHEMA_INVALID`; do not add a new refusal code. Record U-A as resolved or re-open explicitly. |
| X-R2-05 | **Mocked e2e count:** current 8 + 8; W H12 adds 6 replay/retry cases (approval-flow 14, total 22) while P D4 expects **18** (8 tool-loop + 10 approval-flow, i.e. W's 6 not counted or only 2 ordering cases). | Count the actual spec files after implementation; acceptance must quote exact passed count, never force 18 or 22. R2-04/R2-05 must agree on `approval-flow.spec.ts` ownership/count before merging. |
| X-R2-06 | **WP-07 name/id drift:** Run-1 ledger table calls WP-07 “live tool-turn/approval proof”; its section header calls WP-07 “args preview (optional)”; W §8 and P §0 flag this. | Use **WP-07 = live proof** per task; args preview remains deferred/1.3; compiler notes the reconciliation, no renumbering without Lead. |
| X-R2-07 | **396 baseline composition:** 396 log predates final WP-01 tests (14 approval tests vs 16 attrs now); task statement says “tests are 396”; P derives floor ≥416 from 396+14+5+1. | Treat 396 as pre-final-WP-01 baseline; after WP-01 commit re-run and quote N (>396); floor 416/expected 429 are arithmetic expectations, not results. |
| X-R2-08 | **WP-01 “implemented” vs uncommitted/untested:** task says implemented; git shows 4 modified + 2 untracked product files (`approval_wait.rs`, `tools/tests/lead_bugcheck_wp01.rs`), no commit, no run of final WIP. | State “implemented in working tree, uncommitted; runtime unverified” until R2-P0 commit + fresh test run. |
| X-R2-09 | **suspension.rs ownership collision:** E H12 creates `core/tests/suspension.rs`; P WP-04 lists the same new file as its deliverable. | One writer: R2-02 engine owns `suspension.rs` + 14 tests; R2-04 runs it and owns only `tool_loop_matrix.rs`, `approval_ledger.rs` +1, e2e ordering; no duplicate creation. |
| X-R2-10 | **`approval-flow.spec.ts` shared:** W H12 extends it; P §1.5 adds ordering tests; both plan edits. | W owns replay/retry cases; P adds ordering cases after W's commit, or Lead assigns one file owner; the final count is recorded (C-R2-05). |
| X-R2-11 | **Baseline clean/dirty:** E/W report clean tree; S/P report dirty (WP-01 lane); current tree also gained `approval_wait.rs` at 19:00. | Re-baseline at implementation start; the Run-2 reports describe different moments, not a stable tree. |
| X-R2-12 | **Docs checker/harness counts:** RUN-1 record 19/19 and 27/27; P D6 repeats them as expected; compiler did not run them. | Re-run and quote; do not inherit counts as Run-2 results. |
| X-R2-13 | **Live gate behavior:** P expects 5 passed with gates; without WP-02/03 only 3 passed/2 skipped; if C2 default-off, A2/A3 skip with `SIDE_EFFECTS_NOT_ENABLED`. | Record the exact gate/flag matrix at run time; skipped paid tests are never passes. |
| X-R2-14 | **`execute_approved` symbol absent from S blueprint** while E H6/H11 reference it (X-R2-02) — potential compile blocker. | Resolve before H6; either add symbol to WP-02 or change H6 to split API. |
| X-R2-15 | **Untracked `tools/qa/apply_wp01.py`** (mtime 18:51) is a lane helper, not part of WP-01; no report mentions it. | Lead decides: keep as tooling (commit under tools/qa) or delete; never leave untracked product-adjacent residue. |
| X-R2-16 | **`approval_wait.rs` tests count 4** but R2 proof expected floor arithmetic does not include them; workspace count targets (416/429) may already include/exclude them. | Recompute floor from actual post-WP-01 N and record; no fixed expected number in claims. |

## 5. UNKNOWN / UNVERIFIED register

| ID | Unknown | Settle by |
|---|---|---|
| U-R2-1 | Final WP-01 commit content, test count, and whether `approval_wait.rs` is part of it | `git log`/`git diff`, fresh `cargo test` after commit |
| U-R2-2 | Waiter module owner/decision (module vs inline) | Lead decision in R2-P0 |
| U-R2-3 | C1a interface (split vs callback) | Lead decision before engine H6 |
| U-R2-4 | C2 rollout (default-off vs always-wired) | Lead decision before H11/live |
| U-R2-5 | Playwright `WebSocketRoute.close`/reconnect in 1.65 | H11 spike; fallback reload only, record UNVERIFIED |
| U-R2-6 | Server replay ordering/seq policy, all-active vs latest-run, bounded 20 | Lead/backend decision; WP-05 implementation |
| U-R2-7 | ACK→resolved settle 2000 ms realism | product decision; live/recording evidence |
| U-R2-8 | Actual post-WP-04/WP-06 test counts (backend/web/e2e) | run commands, quote exact |
| U-R2-9 | Workspace floor after WP-01/02 tests (416 vs 429 vs actual) | fresh `cargo test --workspace` |
| U-R2-10 | Paid key/budget/operator, 1.2-dev binary rebuild, `HIVEROUTER_SIDE_EFFECTS` activation | operator/Lead; record binary sha/version |
| U-R2-11 | Windows `fs::rename` replace-existing semantics | `write_atomic_creates_and_replaces_file` Lead run |
| U-R2-12 | Whether 396 log was produced at HEAD 90cb7a7 exactly (log has no HEAD line) | re-run; record HEAD with log |
| U-R2-13 | Docs checker/harness actual counts (19/27 claimed) | rerun D6 |
| U-R2-14 | WP-05 backend replay report/test blueprint absent | Lead authorizes design from W §4 or a new planner report |
| U-R2-15 | Live gate exact pass/skip counts under C2/flag variants | D7/D9 run matrix |
| U-R2-16 | New `approval_wait.rs` lane origin/ownership (untracked at 19:00) | Lead identifies lane; adopt or delete |
| U-R2-17 | Whether `apply_wp01.py` is intended tooling or residue | Lead decision |

---

## 6. Explicit “do not claim” list

1. **Do not claim WP-01 is committed or tested** — it is implemented in a dirty working tree; no commit and no run of the final WIP exists. 396 is a pre-final-WP-01 baseline.
2. **Do not claim suspension/approval/resume works** until R2-02 lands and the 14 suspension tests are quoted green by the Lead.
3. **Do not claim side effects execute** — R2-01 has no caller until R2-02; no live file write has been run.
4. **Do not claim exactly-once across crashes/cross-process** — only per-process committed-grant at-most-once; crash between grant and effect consumes the grant with no effect (honest gap).
5. **Do not claim `shell_exec` works** — R2-01 is `file_write` only; shell is refused before grant by name.
6. **Do not claim web replay/reconnect works** — WP-06 is unimplemented; Playwright reconnect support is UNVERIFIED; WP-05 backend replay is unimplemented.
7. **Do not claim a live tool turn or live approval proof** without a redacted paid log and exact quoted pass/skip counts; skipped paid tests are never passes.
8. **Do not claim trace promotion / improved counts** — truth remains 1/264/5/49 of 319, raw capture empty; Run-2 reports do not close trace.
9. **Do not claim G-UI-3/10/W1/W2/axe closure, strict release gate, or 1.2 release/tag/artifacts** — out of Run-2 scope; sealed record is 1.1.0.
10. **Do not claim args preview in 1.2** — deferred/1.3-gated; hash-only copy only.
11. **Do not claim `expected_sha256` has a dedicated refusal code** — `E_SCHEMA_INVALID` mapping is a known semantic gap.
12. **Do not force test counts** (396 current, 416/429 floor, 18/22 mocked e2e, 5 live passed) — quote the actual command output; record HEAD, exit code, and skips.
13. **Do not claim C1a/C2/waiter decisions are resolved** without a Lead record; do not merge two waiter definitions or two grant interfaces.
14. **Do not claim the untracked `approval_wait.rs`/`apply_wp01.py` are committed/product** — they are unowned WIP at compile snapshot.
15. **Do not claim any build/test/paid run by the compiler** — this ledger is read-only.

---

## 7. Residual and exact compiler result

**Residual:**
- Four Run-2 reports are **untracked** and describe different tree moments; WP-01 WIP and `approval_wait.rs` appeared/changed during compile. Re-hash and re-read at implementation start.
- WP-01 is **implemented uncommitted**; the 396 baseline log is pre-final-WP-01 (14 approval tests; current file has 16 attrs). No current-WIP test result exists.
- **WP-05 has no Run-2 code report**; its design is lifted from W §4, so its tests/edge cases are less specified than the other WPs.
- The highest-risk unresolved interface is **C1a/X-R2-02** (split vs callback grant interface) followed by **X-R2-01** waiter placement; both block engine H6/H2 and must be Lead-decided before code.
- All count targets (416/429 workspace floor, 18/22 mocked e2e, 5/3+2 live) are **expectations**, not results.
- The compiler ran no build/test/paid step and edited no product file.

**Exact compiler result:**
- **4 reports frozen** (bytes/lines/sha256 above) + Run-1 ledger/`RUN-1.md` context; `git`/code/test-count verification performed with read-only commands.
- **8 ranked work packages** (R2-P0 prerequisite, R2-01 side effects, R2-02 engine, R2-03 backend replay, R2-04 deterministic proof, R2-05 web, R2-06 live proof, R2-07 gate/receipt), each with target files, language, exact symbols, dependencies, verification commands, acceptance, rollback, blockers.
- **1 paste-ready handoff prompt** with six collision-free waves and exact commands/expected outputs.
- **29 claim/evidence rows**; **16 preserved contradictions** (X-R2-01…X-R2-16) and **17 UNKNOWNs** (U-R2-1…U-R2-17); **15 explicit non-claims**.
- **Read-only guarantee:** no product/test/config/gate/git/tag writes, no builds, no tests, no key access, no paid calls; only `plans/foresight/RUN2-COMPILED-LEDGER.md` written.
