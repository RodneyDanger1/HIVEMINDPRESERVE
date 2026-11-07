# RUN 3+ FINISHER — approval replay wiring (client store)

**Finisher:** strip-watchdog · **Date:** 2026-11-06 · **Mode:** only the write scope below; no builds, no tests, no commits, no gate run.
**Current HEAD:** `889b779` (`docs: record run-2 independent approval primitives bugcheck`, 19:12:17 CT) · **tree:** dirty, **36 entries** at this snapshot.
**Write scope used:** `web/src/state/store.ts` (approval replay wiring only), `plans/foresight/RUN-3.md` (this file). **`web/tests/approval.state.test.ts` was not edited** (not needed: the pure replay helpers it covers already exist; no store-level test harness was added because no test may be run in this task).

## 1. Current dirty tree at this snapshot (recorded, not mine except store.ts)

```
HEAD 889b779  (36 entries)
 M backend/crates/hiverouter-core/src/execution.rs
 M backend/crates/hiverouter-core/src/http.rs
 M backend/crates/hiverouter-core/src/tool_loop.rs
 M backend/crates/hiverouter-core/src/tools_bridge.rs
 M backend/crates/hiverouter-core/src/ws.rs
 M backend/crates/hiverouter-core/tests/approval_ledger.rs
 M backend/crates/hiverouter-core/tests/run_lifecycle.rs
 M backend/crates/hiverouter-core/tests/tool_loop_engine.rs
 M backend/crates/hiverouter-tools/src/approval.rs
 M backend/crates/hiverouter-tools/src/executor.rs
 M backend/crates/hiverouter-tools/src/jail.rs
 M backend/crates/hiverouter-tools/src/lib.rs
 M e2e/helpers/live-server.ts
 M tools/build-release.ps1
 M tools/qa/apply-live-trace.mjs
 M tools/qa/run-gates.ps1
 M tools/qa/verify-harness.mjs
 M web/src/panels/ApprovalOverlay.tsx
 M web/src/state/store.ts          <-- this finisher's edit
 M web/src/state/strings.ts
 M web/tests/agentsToolActivityMount.test.ts
?? backend/crates/hiverouter-core/tests/suspension.rs
?? backend/crates/hiverouter-core/tests/tool_loop_matrix.rs
?? backend/crates/hiverouter-tools/src/side_effects.rs
?? e2e/specs/live-resync.spec.ts
?? e2e/specs/live-security.spec.ts
?? e2e/specs/live-tool-turn.spec.ts
?? e2e/specs/live-trace-capture.spec.ts
?? plans/foresight/RUN-3.md       <-- this file (prior trace preserved in Appendix)
?? tools/qa/apply_run3_overlay.py
?? tools/qa/apply_run3_store.py
?? tools/qa/check-seal-topology.mjs
?? tools/qa/fix_run3_web.py
?? tools/qa/tests/
?? web/src/state/approval.ts
?? web/tests/approval.state.test.ts
```

Baseline files not changed by this finisher: `web/src/state/approval.ts`, `web/tests/approval.state.test.ts`, `e2e/*`, all engine/tools files, `ws.rs`, and the prior RUN-3 trace content below.

## 2. Exact additions in `web/src/state/store.ts` (current line refs)

- **Imports** (`:36-55`) from `./approval`: `APPROVAL_REPLAY_SETTLE_MS`, `approvalDisplayStatus`, `approvalStatusAfterRequest`, `beginReplayWindow`, `isReplayWindowOpen`, `markApprovalsStale`, `settleReplay`, `settleReplayStatus`, `stampReplayEpoch` (plus the pre-existing approval imports; all are used).
- **State field** (`:190`): `readonly approvalSeenEpoch: Readonly<Record<string, number>>` — per-approval replay epoch confirmation; initial value `{}` (`:791`).
- **Replay timer/helpers** (`:362-413`):
  - `approvalReplayTimer` (`ReturnType<typeof setTimeout> | null`);
  - `clearApprovalReplayTimer()` — cancel on stale/hello/dispose;
  - `settleApprovalReplayWindow()` — closes the 400 ms window, calls `settleReplayStatus(...)` for every approval, marks unseen actionable records (`pending`/`retryable`) as `unconfirmed`, clears `approvalsStale`/window/seen map;
  - `armApprovalReplayTimer()` — arms exactly one `APPROVAL_REPLAY_SETTLE_MS` timer;
  - `markApprovalReplayStale()` — cancels the window timer and advances/closes replay state.
- **`case 'SERVER_HELLO'`** (`:426-443`): keeps records, marks them stale (unless already stale), opens `beginReplayWindow(..., 'hello', Date.now())`, sets `approvalsStale: true`, arms the settle timer. No command is sent.
- **`case 'RESYNC_REQUIRED'`** (`:444-448`): `markApprovalReplayStale()` then `connection: 'reconnecting'`.
- **`case 'APPROVAL_REQUEST'`** (`:615-655`): replace-by-id dedupe (one record per `approval_id`); during an open replay window stamps `approvalSeenEpoch[id] = replay.epoch`; ensures an initial resolve state when absent; derives status with `approvalStatusAfterRequest(...)` (preserves in-flight `resolving`/`awaiting_server`, preserves/upgrades terminal states per helper); no command, no auto-approve, no nonce change.
- **`case 'APPROVAL_RESOLVED'`** (`:657-675`): removes the approval, its resolve state and its seen-epoch entry (plus existing timer cleanup).
- **`dispose()`** (`:827-838`): clears the replay timer and all ACK timers before unsubscribing/closing.
- **`dismissApproval()`** (`:1039-1061`; seen-epoch cleanup `:1054-1059`): deletes the seen-epoch entry with the record/resolve state.
- **Unchanged:** `resolveApproval`/`retryApproval` logic, nonce handling, command payloads, contract, every other web file; no new command/event/refusal.

## 3. Semantics wired

1. **SERVER_HELLO** (every hello, including reconnect/reload) keeps existing approvals, marks the list stale, opens a bounded 400 ms replay window and arms one settle timer.
2. **APPROVAL_REQUEST within the window** is confirmed against the current replay epoch (`approvalSeenEpoch`) and upserted by id (dedupe); a missing resolve state gets the neutral initial state. It never sends anything.
3. **RESYNC_REQUIRED** cancels the window timer and marks the list stale; the next hello reopens the window.
4. **Settle timer** closes the window and turns only *unreconfirmed, still-actionable* records into read-only `unconfirmed` (with `settleReplayStatus`); confirmed records stay `pending`, in-flight records stay `resolving`/`awaiting_server`, terminal records are preserved.
5. **No auto-approve, no retry, no command, no nonce handling change** from any replay path.

**Server source of truth read:** `backend/crates/hiverouter-core/src/ws.rs:299-336` — `MAX_APPROVAL_REPLAY_EVENTS = 20`, `publish_pending_approval_replays`, `send_hello_replay_and_approvals`; it publishes pending `APPROVAL_REQUEST` frames after `SERVER_HELLO` on both hello paths (`ws.rs:147`, `:359`). Replay is **server-push-only**.

## 4. Tests / gates run by others (not by this finisher)

| When | Source (recorded) | Result |
|---|---|---|
| 18:56:23 | `docs/evidence/1.2-backend-tests-6.log` | 396 passed / 0 failed / 2 ignored, 39 suites, `EXIT=0` (pre-final-WP-01 baseline) |
| 19:04:29 | `docs/evidence/1.2-backend-tests-7.log` | 401 passed / 0 failed / 2 ignored, 39 suites, `EXIT=0` |
| 19:11:39 | `docs/evidence/1.2-backend-tests-8.log` | 404 passed / 0 failed / 2 ignored, 40 suites, `EXIT=0` |
| 19:52 (recorded) | prior RUN-3 trace below | tools lib 111/0/0; workspace **418 passed / 0 failed / 2 ignored** claimed from a `%TEMP%` log; **not re-run or re-verified by this finisher** |
| 20:02:21 | `docs/evidence/1.2-web-tests-2.log` (another lane) | **FAIL: 4 files failed, 5 tests failed / 174 passed (179 total)** — failures in `toolActivityPanels.test.ts`, `approval.integration.test.ts`, `agentsToolActivityMount.test.ts`, `toolActivity.integration.test.ts`; this log predates this finisher's store edit (store mtime 20:10:32) |
| — | e2e / live / paid / `run-gates` / docs-checker / harness | **not run by this finisher; no current log cited** |

## 5. Residual (explicit)

- **Live hydration command absent:** replay is push-only (`SERVER_HELLO` → bounded `APPROVAL_REQUEST` batch from `ws.rs`); there is **no client command or server command** to request hydration separately (contract has `APPROVAL_RESOLVE` only; no `HYDRATE*` command). This finisher did not add one and did not run a live/e2e replay.
- **No TypeScript build/typecheck or web test run by this finisher.** Types were kept consistent by reading the existing `approval.ts` contract and current store types; this is **not** a tsc receipt. The last recorded web run (20:02, by another lane) is **red** (5 failures) and predates this edit.
- **No store-level replay-wiring test added.** Pure helpers (`markApprovalsStale`/`beginReplayWindow`/`settleReplayStatus`/`stampReplayEpoch`) are already covered in `web/tests/approval.state.test.ts`; the store cases above remain unexecuted.
- **No live hydration/e2e proof; no commit.** Other lanes' dirty files (backend engine/tools, e2e, panel/strings) are untouched by this finisher.

## 6. What must not be claimed

- Do **not** claim tsc/build/web-test/live/e2e pass from this work; no such command was run.
- Do **not** claim live replay/hydration works; only server-push replay exists and no live run was executed.
- Do **not** claim auto-approve or nonce behavior changed — neither changed.
- Do **not** claim the current web suite is green: the last recorded run (20:02, other lane) had 5 failures, and it predates this edit.

---

## Appendix — prior Run-3 trace (WP-02 side effects, recorded 19:52; historical, not this finisher)

> The prior trace below was written by an earlier Run-3 lane when the tree was clean at `889b779`. Its "clean tree" snapshot is historical; the current tree is dirty (36 entries) and contains this finisher's store edit. Its test claims are recorded as that lane's claims, not re-run here. Retained verbatim so its evidence is not lost.
# Deep Foresight — Run 3 trace

**Date:** 2026-11-06 · **Starting HEAD:** `889b779` (Run-2 committed; clean tree) · **Version:** `1.2.0-dev`
**Objective (from RUN-2 next-run handoff):** decide **C1a** (grant-check interface) and **C2** (side-effect rollout gate), then implement **R2-01 WP-02 side-effect `file_write`**, then R2-02 engine suspension and R2-04 deterministic matrix. Deterministic gates before anything paid/live.

## Lead decisions (recorded before coding)

| ID | Decision | Rationale (evidence) |
|---|---|---|
| **C1a** | **Split API adopted**: `side_effects::preflight_side_effect` (read-only) → `ToolsBridge::verify_tool_grant` (single-use CAS, live WP-01) → `side_effects::commit_side_effect` (effect). **No** monolithic `execute_approved` in this wave. | The live bridge owns the DB/grant seam (`core/src/tools_bridge.rs:166` `verify_tool_grant`); a monolithic tools-side `execute_approved` would need an `ApprovalGate` + `Connection` borrow in the tools crate that WP-02 deliberately lacks. The engine calls the three steps in order (S §4). |
| **C2** | Engine wiring stays identical for both variants: suspension only when `RunEngine::tools` is `Some` (`with_tools`). Default **off** behind the engine configuration until the deterministic matrix is green; WP-02 itself introduces **no flag and no caller**. | Matches E's engine design; avoids a half-wired production path in a wave with no engine changes. Do not claim runtime reachability. |
| U-A | `expected_sha256` mismatch/malformed ⇒ `E_SCHEMA_INVALID` (ledger choice; no dedicated stale-write code). | Blueprint §2.4/§5 + compiled ledger C-SE-03; `E_TOOL_SCHEMA` reserved for target-shape refusals. |
| U-D | WP-01 is committed at `c219f51`; no `lib.rs` collision remains. | `git log --oneline`, clean tree at run start. |

## Lead code (actual, this run)

1. **New** `backend/crates/hiverouter-tools/src/side_effects.rs` — the only granted write path:
   - `MAX_WRITE_BYTES = 1 MiB`, `MAX_SIDE_EFFECT_TEXT_BYTES = 64 KiB`;
   - opaque `PreparedWrite` (private fields, redacting `Debug`: path/len/sha only) + `PreparedSideEffect`;
   - `preflight_side_effect(jail, registry, tier, tool, args, cancellation)` — cancellation → registry/tier/schema → permission class → file_write checks (empty path, size, jail containment, root/dir/non-regular refusal, stale `expected_sha256`); **never** reads approvals or consumes a grant;
   - `commit_side_effect(jail, prepared).await` — re-resolves containment (TOCTOU defence using `PreparedWrite.target`), calls `jail.write_atomic`, returns bounded `ExecutedTool::Ok` text `wrote N bytes to <path>; sha256=<hash>` (never content);
   - `shell_exec`/read/unknown tools refused by name before any effect; shell arm documented as additive next wave.
2. **`jail.rs`**: `WorkspaceJail::write_atomic` — same-directory `create_new` temp → `write_all` → `sync_all` → rename-over-target; parent containment + creation; root/dir/non-regular refusal; temp cleanup on every error path. `use std::io::Write as _`.
3. **`lib.rs`**: `pub mod side_effects;` + re-exports (`commit_side_effect`, `preflight_side_effect`, `PreparedSideEffect`, `PreparedWrite`, caps).
4. **Item #16 closure**: the constant-time `nonce_matches` helper + threat-model note were already committed (`approval.rs:446-505`); added the missing unit tests `nonce_compare_accepts_only_an_exact_match` and `nonce_mismatch_refusal_never_echoes_either_nonce` (refusal never echoes presented/stored nonce).

## Trace (commands and quoted results)

| Step | Command / artifact | Result |
|---|---|---|
| Tree frozen | `git rev-parse --short HEAD`; `git status --porcelain` | `889b779`; clean |
| WP-02 build/test | `cd backend` (PATH `tools/w64devkit/bin` + `.cargo/bin`) `cargo test -p hiverouter-tools --lib --locked *> log; $LASTEXITCODE` | `LIB EXIT=0`; `test result: ok. 111 passed; 0 failed; 0 ignored` (109 + 2 new nonce tests); 8 `side_effects::tests` + 4 `jail::tests::write_atomic_*` green |
| Format | `rustfmt --edition 2021 side_effects.rs jail.rs`; `--check` both | format/check exit 0 for the new/edited WP-02 files; `approval.rs` retains pre-existing fmt drift from Run 2 (not reformatted to avoid unrelated churn) |
| Compile gate | `cargo test --workspace --locked *> log; $LASTEXITCODE` | **`WORKSPACE EXIT=0`; 418 passed / 0 failed / 2 ignored** (Run-2 baseline 404 + 12 WP-02 + 2 nonce) |
| Evidence | `$env:TEMP\run3-final-workspace-test.log` (quoted above); changed files list below | read |
| Not run | paid/live, e2e, web suite | not affected by this wave; no web/backend runtime path changed |

**Changed files:** `M backend/crates/hiverouter-tools/src/{approval.rs,jail.rs,lib.rs}`; `?? backend/crates/hiverouter-tools/src/side_effects.rs`. No commit (owner action). No web/core/contract edits.

## Verification table

| Claim | Evidence | Status |
|---|---|---|
| `file_write` executes only after preflight+grant and reports path/size/sha without content | `side_effects::tests::commit_writes_and_reports_path_size_sha_without_content` green | VERIFIED by run |
| Atomic replace leaves no temp residue and replaces existing content | `commit_replaces_existing_file_atomically_without_temp_residue`, `jail::tests::write_atomic_creates_and_replaces_file` green | VERIFIED by run |
| Doomed preflight never consumes the AllowOnce grant | `doomed_write_does_not_consume_grant` (executed_at stays NULL across escape/root/dir/oversize/malformed/mismatch/missing vectors) | VERIFIED by run |
| Grant consumed exactly once; replay refused | `grant_is_consumed_exactly_once` → `E_APPROVAL_REPLAY` on second verify | VERIFIED by run |
| `shell_exec`/read-only/unknown refused before grant | `preflight_refuses_read_only_unknown_and_shell_tools` | VERIFIED by run |
| Cancelled token refuses before grant | `cancelled_token_refuses_before_grant` | VERIFIED by run |
| Result text bounded on a char boundary | `result_text_is_bounded_on_char_boundary` | VERIFIED by run |
| Runtime reachability (engine executes approved writes) | no caller; WP-03 absent | **UNVERIFIED / not claimed** |
| Windows `fs::rename` replaces existing files | overwrite test passed on this host | VERIFIED by run |
| Clippy / full gate `run-gates` / docs / harness | not run this wave | **UNVERIFIED** |

## Not done / blockers carried to Run 4

- **R2-02 WP-03 engine suspension/resume/wait/event wiring** — not started; C1a/C2 are now decided, so Run 4 can implement H1–H12 from `RUN2-ENGINE-CODE.md`. `execution.rs` non-read tools still refuse `E_TOOL_NOT_ALLOWED`.
- **R2-04 deterministic matrix** — depends on WP-03; new `core/tests/tool_loop_matrix.rs` not written.
- **R2-03 backend replay (WP-05)**, **R2-05 web replay/retry (WP-06)**, **paid live tool-turn proof**, **1.2 seal** — unchanged residual.
- **Item #17 (`runtime-test/**` tracked SQLite/WAL artifacts)**: confirmed 82 tracked files; untracking/purging is an owner git action (no index/history mutation performed).
- WP-02 has **no caller**: nothing is user-visible until WP-03; do not claim 1.2 side effects work.

## Lessons / traps (also in skill `references/RUN-3-LESSONS.md`)

1. PowerShell `cargo test ... 2>&1 | Select-Object` reports a non-zero process exit for a *passing* run (stderr NativeCommandError); use `*> log; $LASTEXITCODE` and quote both.
2. Match workspace edition for rustfmt (`2021`); formatting only your changed files avoids churn in another lane's pre-existing `approval.rs` drift.
3. The opaque `PreparedWrite.target` field becomes dead code unless the commit path uses it; use it for the commit-time containment recheck (defence in depth) instead of `#[allow(dead_code)]`.
4. RED-first test trap: proving "doomed call does not consume grant" requires both `preflight(...).is_err()` **and** the DB `executed_at IS NULL` read; error-only assertions would pass after a burn.
5. Verify repository claims before coding a residual item: item #16 was already implemented; the missing part was tests/evidence, so the run added tests instead of duplicating code.

---

## Run 3 addendum — WP-03 engine suspension landed (same run, after the WP-02 slice)

**Decisions:** C1a split protocol (preflight -> erify_tool_grant -> commit) is now consumed by ToolsBridge::execute_approved; C2 keeps the pre-P0 read-only behavior when .with_tools is absent (test seams use with_services + no jail wiring; production paths inject the bridge).

**Engine files (ownership: Lead):**
- core/src/execution.rs (+630): Clock/default_now_unix_secs; ArmedApproval RAII; RunEngine.{tools,waiters,clock,shell_policy} + with_tools/with_clock/with_shell_policy/notify_approval_resolved; async execute_tool_turn with approval branch + precounted waits; persist_approval_transitions, esume_transitions, suspend_for_approval, pply_committed_decision, esume_and_execute, esume_denied, inish_timeout_wait, inish_cancelled_wait; dispatch_inner Planning->Running Start frame; dispatch terminal-owner guard (inal_status); execute_task await + double-failure guard; cancel_run token-only (engine owns the single terminal frame and TOOL_RESULT ordering); kill_worker awaiting branch.
- core/src/tools_bridge.rs (+101): pending_approval_events_all(limit) read-only replay projection; execute_approved(...) (preflight -> grant CAS -> one effect).
- core/src/ws.rs (+70): publish_pending_approval_replays/send_hello_replay_and_approvals wired into both hello paths; wake 
otify_approval_resolved after a committed APPROVAL_RESOLVE (late/duplicate resolve = warn only).
- core/src/http.rs (+16): tools built before the engine; bridge injected in 
ew/with_graph/with_services.
- core/src/tool_loop.rs: MAX_APPROVAL_WAITS_PER_RUN = 8.
- 	ools/src/{side_effects.rs,jail.rs,lib.rs,approval.rs} (ownership: Lead): shell_exec arm completed (allowlisted bare program, argv/NUL/argv-size/cwd bounds, Job Object execution, no argv text in events); write_atomic; replay projections; nonce-compare tests.

**Tests:** core/tests/suspension.rs 14/14 green (agent-authored scenarios: suspend, approve-once, deny, timeout, cancel, kill, CAS-loss resume, concurrent resolve, second prompt, restart, wait cap, wrong args, shell); existing loops reconciled for the new RUN_STATUS running frame. Test corrections made with evidence: agent 	ool_loop_matrix.rs gained Debug on Collected (compile), shell scenario now asserts the frozen TOOL_RESULT payload has size+hash and never output/argv text, and the 1 s-TTL CAS race test became deterministic (second-granularity clock documented).

**Gate (actual):** cargo test --workspace --locked -> **441 passed / 0 failed / 2 ignored, exit 0**; cargo test -p hiverouter-tools --lib -> 114/0; cargo check --workspace --tests --locked -> 0. No release build, no paid call.

**Not in this ownership lane:** web WP-06 (store.ts, ApprovalOverlay.tsx, strings.ts, state/approval.ts) is the Lead's concurrent lane; live/trace/seal files are agent lanes.
