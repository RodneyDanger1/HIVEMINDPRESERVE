# LEAD-BUGCHECK — adversarial review of the Lead's 1.2 boot/reasoning changes

**Agent:** watchdog-2 (BUG-CHECK agent, `/forge` + `/deep-investigate` + `/forgewright-lead` + `/evidence-check`)
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1`
**Baseline:** `HEAD 37f0e7c68b835f267e64db3cd5c0612114328e46` ("wip(1.2): compile-green read-only tool loop, approval command/UI, 23-event contract, e2e tool/approval specs") + uncommitted Lead changes.
**Verified at start (`git status --short`):** Lead changes in
`backend/crates/hiverouter-core/src/{execution,http,provider_executor,recovery,tools_bridge}.rs`,
`backend/crates/hiverouter-tools/src/approval.rs`, `backend/crates/hiverouter-core/tests/run_lifecycle.rs`.
Concurrent seats later added `tool_loop_engine.rs` reasoning fields, `approval.rs` `nonce_matches` + TTL-reaper test, and untracked `plans/foresight/` — **not this agent's changes**.
**Constraints observed:** no release build, no paid/live provider call, no commit.
**Environment note:** system `cargo 1.99.1` uses `x86_64-pc-windows-gnu` (rust-toolchain.toml) and had no `gcc` on PATH, so `cargo check` worked but test binaries could not link. For **debug test execution only** this agent used a portable WinLibs GCC 16.2.0 unpacked to `%TEMP%\winlibs` (not in the repo, not committed) and prepended its `bin` to PATH. No release build was run.

## 1. Scope reviewed

Read-only first, then fixes only where a defect was proven, limited to the six Lead files (plus one new regression test):

- `core/src/provider_executor.rs` — reasoning accumulation + continuation reasoning passback.
- `core/src/execution.rs` — `StreamedReply.reasoning` / `ToolTurnTranscript.reasoning` propagation.
- `tools/src/approval.rs` — `expire_pending` + decision semantics.
- `core/src/tools_bridge.rs` — `expire_pending_approvals` boot reaper.
- `core/src/http.rs` — boot reaper call site.
- `core/src/recovery.rs` — `awaiting_approval` → `interrupted` / `E_APPROVAL_TIMEOUT`.

## 2. Suspicious behaviours — three hypotheses each, elimination

### S1: After a restart, approvals that were pending at crash time can still be resolved (design says they must be expired)

Design requirements read:
- `plans/waves/W11-SUSPENSION-DESIGN.md` §4/§9: "crash/restart ... **pending approvals expired**"; "boot: runs non-terminal → interrupted ...; **pending approvals → expired**"; "no resolvable stale rows".
- `plans/foresight/APPROVAL-SUSPENSION-PLAN.md` E4: restart → pending approvals expired; late resolve gets `E_APPROVAL_TIMEOUT`/`E_APPROVAL_NOT_FOUND`.
- `plans/foresight/SIDE-EFFECT-EXECUTOR-PLAN.md` test 12: run persisted `awaiting_approval` + restart ⇒ interrupted with `E_APPROVAL_TIMEOUT`; **pending row expired**; nothing executed.

| # | Hypothesis | Cheapest discriminating probe | Result |
|---|---|---|---|
| H1 | `expire_pending` only consumes TTL-elapsed rows; a future `expires_at` (normal crash window, TTL 120 s) survives | Boot-sequence integration test with future-TTL row: recovery → `ToolsBridge::expire_pending_approvals` | **CONFIRMED** — pre-fix test failed at `expired` count: `left: 0, right: 1`, pending row remains |
| H2 | Recovery runs after the reaper, so even a correct reaper sees live rows | Read `main.rs:81` (recovery) vs `http.rs:118` (reaper inside `build_tools` called by `AppState::with_graph` at `main.rs:123`) | Eliminated — recovery first, reaper second; test reproduced the production order |
| H3 | Reaper is not reached on the real boot path / UI replay resurrects rows | Read `http.rs:107-130`, `AppState::{new,with_graph,with_services}` all call `build_tools`; `pending_approval_events` is read-only and has no production ws caller (APPROVAL-UX-PLAN E11) | Eliminated — reaper is on the boot path; defect is its TTL gate |

**Confirmed defect BUG-BOOT-1.** `approval::expire_pending` filtered `now_unix_secs >= expires_at`; `ToolsBridge::expire_pending_approvals` called it at boot. A crash/restart within the 120 s TTL leaves the approval pending; the run is marked `interrupted`/`E_APPROVAL_TIMEOUT` but the row stays resolvable. `recover_on_boot` itself marks only runs, so nothing else expires it.

### S2: A late decision after boot/TTL expiry is reported as `E_APPROVAL_REPLAY` instead of the documented `E_APPROVAL_TIMEOUT`

| # | Hypothesis | Cheapest discriminating probe | Result |
|---|---|---|---|
| H1 | `resolve` / `resolve_with_reason` inspect any stored decision before the expiry branch, so `decision='expired'` goes to the replay branch | Read HEAD `approval.rs` `match existing { Some(Some(_)) => Replay, ... }` before `is_expired`; after fixing BUG-BOOT-1 the boot test hit this assertion and failed with `Replay` while the row was marked expired (source-proven while the test's first assertion already passed) | **CONFIRMED** (source + regression test; the boot test's earlier RED stopped at BUG-BOOT-1, so this leg is source-proven and now covered by the new unit test) |
| H2 | Reaper does not mark rows expired, so the expiry branch is never reached | Same test after BUG-BOOT-1 fix: row is `decision='expired'` yet resolve still returns Replay | Eliminated — H1 |
| H3 | ws layer maps the tool error to a different code | `ws.rs:465-486` calls `resolve_approval_ledgered` and publishes the returned refusal unchanged; `tools_bridge` maps `ToolError::refusal()` unchanged | Eliminated — mapping is identity |

**Confirmed defect BUG-BOOT-2.** Contradicts `APPROVAL-SUSPENSION-PLAN.md` E4 ("late resolve gets `E_APPROVAL_TIMEOUT`/`E_APPROVAL_NOT_FOUND`") and the existing doc comment "TTL expiry returns `E_APPROVAL_TIMEOUT`". A consumed-as-expired row is a timeout, not a replay/forgery signal.

### S3: Reasoning is accumulated but not passed back on the next tool turn

| # | Hypothesis | Cheapest discriminating probe | Result |
|---|---|---|---|
| H1 | `StreamEvent` reasoning fragments are discarded in the core adapter | Read `provider_executor.rs:136-181`: `let mut reasoning = String::new()`, `Delta { content, reasoning_content }` pushes fragments even when `content` is empty; client `client.rs:682-716` emits reasoning `Delta` events | Eliminated — accumulation present |
| H2 | The engine drops the reasoning before the continuation request | Read `execution.rs:1011-1087` and `:1055-1059`: each tool turn pushes `reasoning: final_reply.reasoning.clone()`; `provider_executor.rs:354-367` attaches `with_reasoning_content` to the replayed assistant tool-call message | Eliminated — propagation present |
| H3 | The no-visible-content assistant tool-call message loses reasoning | Provider wire tests `tool_continuation.rs` (6/6 pass): `reasoning_pass_back_is_attached_when_configured`, `a_second_continuation_appends_another_ordered_turn` assert exact assistant message with `tool_calls` + `reasoning_content` | Eliminated — serialization pinned; no defect proven |

**No reasoning/passback defect proven.** The provider dto layer is verified by 6 passing wire-shape tests; the core adapter's message-building was inspected but not executed end-to-end (no paid/live call; see Unverified). `StreamedReply`/`ToolTurnTranscript` constructors compile; `cargo check --workspace --tests` exits 0 (a concurrent seat added the missing `reasoning: String::new()` to `tool_loop_engine.rs`; that file is not one of the Lead's six).

### S4: Boot reaper failure handling / recovery completeness (investigated, not proven as defects)

| # | Hypothesis | Probe | Result |
|---|---|---|---|
| H1 | If the boot reaper fails, boot continues and restart-orphaned rows stay pending | Read `http.rs:118-128`: error is logged and `build_tools` returns normally | **Residual risk, not fixed** — plan says boot expiry is a "precondition"; however no waiter survives restart, so no side effect can execute without an executor. Needs an injectable DB-failure test and a boot-failure policy decision. |
| H2 | Recovery leaves task rows non-terminal even when run rows are interrupted | Read `recovery.rs:50-64`: only `runs` statuses are updated; task persistence is untouched by this wave | **Unverified residual** — not part of the Lead's diff; needs a task-state snapshot/boot test. |
| H3 | `list_runs(500)` truncation misses older active runs | Read `recovery.rs:51` | Pre-existing, outside Lead diff; residual only |

## 3. Reproduction and fix evidence

### Reproduction command (defined before the fix; run twice after)

```
cd backend
cargo test -p hiverouter-core --test lead_bugcheck_boot --offline -- --nocapture
```

**RED before fix (future-TTL restart orphan):**

```
thread '...' panicked at crates\hiverouter-core\tests\lead_bugcheck_boot.rs:89:5:
assertion `left == right` failed: the restart-orphaned pending row must be consumed
  left: 0
 right: 1
```

**GREEN after fix (quoted run 1 and run 2):**

```
running 1 test
test boot_restart_expires_orphaned_pending_approval_and_late_resolve_times_out ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
```
```
running 1 test
test boot_restart_expires_orphaned_pending_approval_and_late_resolve_times_out ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
```

### Other commands / evidence

- `cargo test -p hiverouter-tools --lib approval:: --offline` → **15 passed; 0 failed; 81 filtered out** (includes new `expire_orphaned_pending_consumes_live_requests_at_boot` and the updated late-decision expectations `E_APPROVAL_TIMEOUT`).
- `cargo test -p hiverouter-core --tests --offline` → exit 0; all core suites `0 failed`.
- `cargo test -p hiverouter-provider --tests --offline` → exit 0; `tool_continuation` **6 passed/0 failed**.
- `cargo test --workspace --offline` → **exit 0**, all observed suites `0 failed` (Lead's tally: 396 passed).
- `cargo check --workspace --tests --offline` → **exit 0** (compilation green after fixes).

### Fixes (minimal diff)

1. `backend/crates/hiverouter-tools/src/approval.rs`
   - Added `expire_orphaned_pending(conn, now)` (`:577-590`): one unconditional `UPDATE approvals SET decision='expired', decided_at=? WHERE decision IS NULL`; boot has no waiter, so every still-pending row is restart-orphaned.
   - Kept `expire_pending` (TTL, live process) unchanged; its doc now points to the boot orphan reaper (`:592-598`).
   - `resolve` and `resolve_with_reason` now map a stored `decision == 'expired'` to `E_APPROVAL_TIMEOUT` via `expired_decision_refusal()` (`:331-332`, `:497-498`, `:554-562`), instead of `E_APPROVAL_REPLAY`.
   - Added unit test `expire_orphaned_pending_consumes_live_requests_at_boot` (`:966+`); updated two existing late-decision expectations from `E_APPROVAL_REPLAY` to `E_APPROVAL_TIMEOUT`.
2. `backend/crates/hiverouter-core/src/tools_bridge.rs` (`:90-101`): boot reaper now calls `expire_orphaned_pending`; doc states the boot/no-waiter invariant.
3. `backend/crates/hiverouter-core/src/http.rs` (`:118-128`): boot log wording now describes restart-orphaned consumption and does not claim the TTL-only path.
4. New regression test `backend/crates/hiverouter-core/tests/lead_bugcheck_boot.rs`: production-order boot test (future-TTL pending approval → `recover_on_boot` → `expire_pending_approvals` → no pending rows → late resolve is `E_APPROVAL_TIMEOUT`; run error code checked).
5. No changes to `provider_executor.rs`, `execution.rs`, or `recovery.rs`: their Lead changes were inspected and either verified by existing tests or no defect was proven.

### Files changed by this agent (sha256)

| File | sha256 |
|---|---|
| `backend/crates/hiverouter-tools/src/approval.rs` | `38fe3ad9bc7ff4bfbe004e23f253e4fde05105a6a4a652dec4b58fa3dfc3a9b0` |
| `backend/crates/hiverouter-core/src/tools_bridge.rs` | `37c43799bafd9df664a34f168360094920fdd8ca3d0715f3dc4e19cadb3bcc5e` |
| `backend/crates/hiverouter-core/src/http.rs` | `5e1d62a73e519a492445536e35030b4edf3df6aeb711ed59d78b6525e155cf23` |
| `backend/crates/hiverouter-core/tests/lead_bugcheck_boot.rs` | `8679a8df4682fb20dccd5e205c656ff690abe548feb73ff74347fb45b33a19d0` |
| `plans/verification/LEAD-BUGCHECK.md` | `658a002a37afb52cda6566d572b4a44054fbb37cbdebc4043657d0da97b340a1` |

(No commit; working tree also contains the Lead's and concurrent seats' changes.)

## 4. Stub sweep

`Select-String` over the four bugcheck-changed files for `TODO|FIXME|XXX|HACK|placeholder|NotImplemented|not implemented|lorem|simplified for|dummy` returned **zero hits**. The new test file makes no network/live/provider call (`http|reqwest|DEEPSEEK|wiremock` sweep: zero hits). The test double (`Fake*`/tempdir SQLite) is not present in product code.

## 5. Residual / not fixed (explicit)

- **Boot reaper failure path (S4-H1):** DB failure at boot is logged and the server continues; orphan rows could remain pending. No side effect can execute without a live waiter, but a late `APPROVAL_RESOLVE` could still be ledgered/accepted for an interrupted run. Plan calls boot expiry a precondition; fixing means deciding whether boot should refuse to start. Not proven by an executable test; no change.
- **Task rows after restart (S4-H2):** recovery updates only `runs`; tasks persisted `awaiting_approval` are not adjusted by this diff. Unverified; outside the six-file Lead change.
- **`expire_pending` TTL reaper now has no production caller** (only its unit test); boot uses the orphan reaper. Kept as the live-process TTL primitive; flagged for the Lead to either wire a periodic sweeper or delete later.
- **Core adapter reasoning passback not executed end-to-end** (would need a wiremock request-capture test in core or a paid/live call; not allowed here). Provider dto wire-shape tests pass and the code path was read.
- **Repo-wide rustfmt is not clean before this diff** (pre-existing formatting churn in `approval.rs`, `tools_bridge.rs`, `http.rs`); this agent formatted only its new test file to avoid an unrelated mass reformat. Not a gate invoked by the Lead here.
- `recover_on_boot` `list_runs(500)` cap and task reaping are pre-existing/out-of-diff.

## 6. Evidence-check claim table

| Claim | Evidence | Status |
|---|---|---|
| Boot restart with future-TTL orphan left the row pending | RED command output above (`left: 0, right: 1`) + design docs | **verified (pre-fix)** |
| Boot now consumes restart-orphaned rows and late resolve is a timeout | GREEN test run twice; source lines listed; `recovery` error code asserted in test | **verified** |
| `expire_pending` TTL semantics preserved | `expire_pending_consumes_only_elapsed_requests` still passes; tools 15/15 | **verified** |
| Late decision on an expired row now returns `E_APPROVAL_TIMEOUT` | New unit test + boot test; updated expectations; source lines | **verified** |
| No reasoning defect proven | Provider `tool_continuation` 6/6 pass; `provider_executor.rs` / `execution.rs` lines; no failing repro | **verified for wire shape; core end-to-end unverified** |
| Compilation green | `cargo check --workspace --tests --offline` exit 0 | **verified** |
| Full backend green | `cargo test --workspace --offline` exit 0 (Lead: 396 passed) | **verified** |
| No release build / no paid call | Commands above are debug `cargo test`/`check`; no provider key/network path used | **verified by command scope** |
| Boot `AppState::with_graph` end-to-end exercised | New test calls recovery + bridge reaper directly, not full `AppState` | **unverified** (call site read) |
| Reaper-failure and task-state residuals | No executable test | **unverified** |

## 7. Done / not done / unverified

**Done**
- Read-only review of all six Lead files; three hypotheses per suspicious behaviour recorded and eliminated or confirmed.
- BUG-BOOT-1 fixed: boot restart now expires all restart-orphaned pending approvals regardless of remaining TTL.
- BUG-BOOT-2 fixed: a stored `expired` decision now resolves as `E_APPROVAL_TIMEOUT` (not replay/forgery).
- Regression test `lead_bugcheck_boot.rs` added; RED quoted, GREEN quoted twice.
- Existing approval tests updated to the documented timeout semantics; full workspace debug suite green.

**Not done**
- No release build, no paid/live provider call, no commit (per instructions).
- No fix for boot-reaper DB-failure policy or persisted task rows: not proven with an executable reproduction; outside the minimal proven-change boundary.

**Unverified**
- End-to-end `AppState::with_graph` boot call path (test reproduces the recovery + reaper calls directly; call-site wiring read in `main.rs`/`http.rs`).
- Core adapter reasoning passback end-to-end (provider dto wire tests pass; core request-message path read only).
- Reaper-failure and task-status residuals.
- Repo-wide rustfmt; only the new test file was formatted.
