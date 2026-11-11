# GAP-PATCH-1 — paste-ready finish-plan section (Gap Auditor 1/2)

**Repo/HEAD:** `C:\Users\Riley\Desktop\hiverouter-v1` @ `6a41250`. **Evidence:** `docs/analysis/GAP-AUDIT-1.md` (§/G-ids below). **Mode:** read-only; every item is a proposal, none applied or executed.

---

## A. Corrections to apply to the watchdog/lane record before planning

| # | Current record | Correction (paste this) | Evidence |
|---|---|---|---|
| A1 | Ledger `L4-009` says "78 → 79 refusal codes" | Revert to **78**. `refusal.rs` macro table has 78 code rows; the 79th literal `"E_NOT_A_REAL_CODE"` is a negative test fixture at `refusal.rs:366`. L4's 78 was right. | G-23; `refusal.rs:73-…,346,366` |
| A2 | Ledger `L1-008` compresses L1 to "64 provider calls cannot cover a legal 40-node plan" | Replace with: "the 64 provider-call cap is consistent with a no-tool 40-node plan (planner 1 + root-control ≤2 + 40 task turns + root final = 44-45), but inconsistent with the per-task 12-turn allowance; ~20 follow-up tool turns across the run exhaust it (`execution.rs:5469`; `tool_loop.rs:14`). Separately, `MAX_CALLS_PER_RUN = 40` (`tool_loop.rs:16`) is a *tool-call* cap, not a provider-call cap." | G-22, §4.1 |
| A3 | Ledger `L1-022` says only `db.rs`/`store.rs` use non-blank counts | Extend: L1 §1 rows 23-27 (`recovery_multi_run`, `snapshot_history`, `scale_s0/s1`, `graph_authority`, `kill_restore`) also use **non-blank** counts (265/181/109/149/109). | G-26 |
| A4 | L1 `L1-025` "bodies not read" | Resolve: bodies read; `snapshot_history`, `graph_authority`, `kill_restore`, `recovery_multi_run` **support** their citations. `scale_s1.rs:101` proves 41 tasks/overlap but **never asserts `max_active <= 32`**; its header still says "engine is serial today" (`:5-8`). | G-15/G-16 |
| A5 | L6 §5.2 D1 "dirty tree names `?? docs-analysis-tmp-index.txt`" | Current dirty tree = `?? docs/analysis/` + `?? docs/plans/HIVEMIND-FINISH-PLAN-v3.md`; `run-gates.ps1:224-227` refuses any dirty path. Deleting the old temp file changes nothing. Choose: commit swarm artifacts, gate exclusion, or `-Rehearsal` (never a green PASS). | G-27/§3.7 |
| A6 | L6 per-file line counts | Subtract 1: `snapshot_history` 290, `scale_s1` 189, `kill_restore` 117, `recovery_multi_run` 161, `graph_authority` 120 (L6 had 291/190/118/162/121). State the counting method. | G-25 |
| A7 | L7 §4 item 4 cites `dist/HiveMind/README.txt:18-19` | Citation is `:14-15` (the key instruction); `:18-20` is the launcher paragraph. Substance (tools/ absent) stands. | G-30 |
| A8 | Census ownership table | `CODEBASE-CENSUS.md:782` assigns `build.rs` to L4, but L4 never names it (0 hits). Keep it **unowned** in the plan; the watchdog §4 is right in practice. | G-03 |
| A9 | L5 §9 item 6 / §10 item 4 unhedged | Server accepts `Authorization: Bearer` **or** `x-hivemind-token` (`http.rs:767-779`); no 401 path from header choice. Keep only the consistency nit. | G-18 |
| A10 | L2 R6 "finish cannot finish" | Delete. Both tools execute as read-class terminal results (`tools/executor.rs:78-79`) and the engine completes the task after the turn (`execution.rs:5416-5430,5437-5462`). The real gap is zero test coverage. | G-17 |
| A11 | "Approval ledger failure is fail-closed" (code comments `tools_bridge.rs:387-391`) | False: after the decision commits, `cancel_approval` loses the `decision IS NULL` CAS (`approval.rs:640-643`), so the timeout path later executes the committed `allow_once` (`execution.rs:3447-3471`). Mark the comment a defect, not the design. | G-21 |
| A12 | L2 data-dir jail claim | Keep, and add the missing link: approved side effects go through `ToolsBridge::execute_approved` (`tools_bridge.rs:213-253`), which uses the **bridge** jail (`http.rs:259-262`, no `deny_path`), while only the engine read jail denies `data_dir` (`http.rs:347-351`). | G-19 |

---

## B. Gaps closed by this audit (add to the finish plan's "found" list)

1. **Daemon script env leak (HIGH, operator-reachable).** `daemon.rs:428-445`: `.ps1` runs `powershell -ExecutionPolicy Bypass -File`, `.exe` runs directly, with `.args(...)` but **no `.env_clear()`/env allowlist**; the child inherits `DEEPSEEK_API_KEY` that the launcher injects into the server (`dist/HiveMind/README.txt:11-13`). Model reachability is bounded: root `spawn_daemon` forces `{"check":"db"}`/`{}` (`execution.rs:4507-4511`), so only operator `POST /api/daemons` (`http.rs:2127`) creates script jobs. Fix: `command.env_clear().envs(allowlist)` (reuse `hiverouter-tools::env_safety`) and add a test that reads `DEEPSEEK_API_KEY` from a `.ps1` and asserts it is absent.
2. **Plan validation vs caps (MEDIUM).** `Plan::validate` (`planner.rs:50-129`) does not enforce fan-out ≤8 or agents ≤64; dispatch does (`execution.rs:2025-2029`). A validating plan can fail mid-run after the planning call. Add the checks to `validate()` and a test with 9 siblings under one parent.
3. **History silent epoch fallback (MEDIUM).** `history.rs:79`: corrupt `created_at` becomes 1970. Return a named refusal and test it. Also remove O(n) per-revision file reads (`:75-88`) or bound/cache them.
4. **SPA fallback hole (LOW).** `static_ui.rs:48` catches `api/…` but not exact `api`; `GET /api` returns `index.html` 200. Fix the predicate to `path == "api" || path.starts_with("api/")`; make non-GET/HEAD a JSON 405.
5. **Handshake is dead public API (MEDIUM).** `handshake.rs` is re-exported (`lib.rs:49`) but has no production caller; its doc claims a transport that does not carry it. Either wire it into `ws.rs` (then add idempotency-persistence and capability-set validation) or delete it plus `tests/handshake.rs`; do not ship a documented-but-unused protocol.
6. **Guarded e2e fails open (HIGH for test trust).** `e2e/run-guarded.mjs:16-35` skips the skip-guard when `test-results/results.json` is missing/unparsable, exactly its threat model. Delete the report before the run, fail on `report === null` (explicit override env if ever needed), and reject stale reports.
7. **No gate runs the script unit tests (HIGH).** `tools/qa/tests/seal-topology.test.mjs` (5 cases) and `trace-tools.test.mjs` (3) are never executed by `run-gates.ps1` (0 `node --test` hits). Add a G5b step `node --test tools/qa/tests/` and make G5b non-skippable in release mode.
8. **Gate/install identity (HIGH for release claim).** Manifest is `hivemind.release-manifest.v1`/1.1.0 (`release-manifest.json:2-4`) while the tree is 1.2.0-dev; `run-gates.ps1:246-248` requires v2 for sealed mode, and `:211-213,313-359` forces `-Release` to exit 3 (PRE-RELEASE) on this tree. Either migrate the manifest to v2 with a real seal record or state "no 1.2 release claim" explicitly; `dist/HiveMind` is the dev build (36,540,416 B = `target/release`), not the sealed 13,624,320 B artifact.
9. **build.rs toolchain parsing (LOW).** `build.rs:29-31,44-48` rejects nightly/beta (`1.93.0-nightly` → `"0-nightly"` parse fail) and ignores patch level; the full version line is embedded (`:38`), making binary reproduction toolchain-dependent. Pin/document a stable toolchain, or parse `-nightly` suffix and compare `(major, minor, patch)`.
10. **Supervisor observability (LOW).** `remote_supervisor.rs:116-126` sets `next_due_at` from backoff, ignoring the configured interval; `record_state` results are discarded (`:61,80,84`); no stopped state. Store the interval-derived due time and surface state-write failures.

---

## C. Residual unknowns (do not close without evidence)

- No code was executed. The approval race, the daemon env leak, the e2e guard gap and the `scale_s1` bound are static deductions; each needs one executed test to move from "code-derived" to "observed".
- `ws.rs` `Lagged`→`E_RESYNC_REQUIRED` mapping and `host_guard`/launch-token path were not read.
- Snapshot index ordering (`indexed_revisions`) not read; `history.latest_revision` assumes ascending.
- `census.rs` accepted from L2's full read (not independently reopened).
- `side_effects.rs`/`shell.rs` env allowlist accepted from citations; daemon contrast is import-level.
- `check-seal-topology.mjs` predicate body not fully read; its test set misses empty/duplicate allowlists, path normalization and missing-file cases.
- Release-manifest v2 template does not exist on disk; migration path unspecified.

---

## D. Work-package implications (file paths, acceptance)

| WP | Files | Change | Acceptance (must run) |
|---|---|---|---|
| WP-A jail parity | `backend/crates/hiverouter-core/src/http.rs:259-262`, `tools_bridge.rs:53-75` | `WorkspaceJail::open(workspace_root).deny_path(&config.data_dir)` for the bridge; add `jail_error`/deny test | `cargo test -p hiverouter-core` test: `FILE_READ data/hiverouter.db` → `E_PATH_OUTSIDE_JAIL`; `file_write data/…` → refused; `FILE_READ` inside workspace still ok |
| WP-B grant binding | `backend/crates/hiverouter-tools/src/approval.rs:924-982`, `tools_bridge.rs:364-397`, `execution.rs:3447-3471` | Lookup by `approval_id`/`tool_call_id` + `expires_at` check at verify; post-decision `cancelled`/`superseded` state that the timeout path honors; fix the false comment | New test in `tests/approval_ledger.rs`/`tests/suspension.rs`: expired-but-decided allow_once is **not** executed; different args/tool_call cannot consume the grant; simulated ledger failure → no execution |
| WP-C daemon env | `backend/crates/hiverouter-core/src/daemon.rs:428-448` | `.env_clear()` + `env_safety` allowlist; document operator-only reachability | Test script prints/checks `DEEPSEEK_API_KEY` → absent; `.exe` and `.ps1` both covered |
| WP-D plan caps | `backend/crates/hiverouter-core/src/planner.rs:50-129` | Enforce fan-out ≤8/parent and agents ≤64 in `validate()`; use `caps::MAX_*` (not literals) | `planner.rs` unit test: 9 siblings under one parent → `E_FANOUT_LIMIT` **before** dispatch; existing `node_cap_is_forty` still passes |
| WP-E history integrity | `backend/crates/hiverouter-core/src/history.rs:75-115` | Named refusal for corrupt `created_at`; bounded/cached revisions; sort or assert index order | Test: corrupt index row → refusal (not 1970); 100-revision run returns without 100 file reads (instrument or benchmark) |
| WP-F gate truth | `tools/qa/run-gates.ps1` (add G5b), `e2e/run-guarded.mjs`, `tools/release-manifest.json` | `node --test tools/qa/tests/`; e2e report freshness/fail-null; v2 manifest or explicit no-release | `pwsh tools/qa/run-gates.ps1 -Rehearsal` shows G5b PASS; delete `test-results/results.json` then run e2e → exit non-zero if nothing ran; `-Release` outcome documented as exit 3 |
| WP-G handshake decision | `backend/crates/hiverouter-core/src/handshake.rs`, `src/lib.rs:19,49`, `tests/handshake.rs` | Wire or delete; if wire, persist correlation ids and validate capability sets/depth against rows | If deleted: `cargo test -p hiverouter-core` compiles without `tests/handshake.rs`; if wired: new test replaying after restart and a second concurrent wait |
| WP-H build determinism | `backend/crates/hiverouter-core/build.rs:29-39` | Parse prerelease suffixes; enforce patch ≥0; record `rustc -Vv` hash separately only for `/health`; document pinned stable toolchain | `cargo build` on a stable 1.89+ toolchain; a unit test for `parse_major_minor("rustc 1.93.0-nightly (…)")` returns `(1,93)`; release binary hash stable across two builds on one toolchain |

---

## E. Minimum verification block for the Lead (commands are proposals, not executed here)

```powershell
# from repo root; read-only audit did NOT run these
cargo test -p hiverouter-core --locked
cargo test -p hiverouter-tools --locked
node --test tools/qa/tests/
pwsh -NoProfile -ExecutionPolicy Bypass -File tools/qa/run-gates.ps1 -Rehearsal
```

Expected after WP-F: `node --test` reports 8 cases (5 seal + 3 trace) and `run-gates` prints G5b **PASS** (not SKIPPED). Until then, the gate's seal predicate and trace guard are **untested by any gate**, and every "tests pass" statement in the seven lane reports remains unexecuted.

---

*Paste section A into the plan's corrections block, B into "found gaps", C into "residual unknowns", D into work packages. Do not merge before checking line anchors (all HEAD `6a41250`).*
