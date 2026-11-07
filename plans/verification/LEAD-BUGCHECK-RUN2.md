# LEAD-BUGCHECK-RUN2 — WP-01 approval gate primitives + table integrity

**Agent:** watchdog-2 (BUG-CHECK; `/forge` + `/deep-investigate` + `/forgewright-lead` + `/evidence-check`)
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1`
**Baseline:** `HEAD 90cb7a736d36306a16b63f32633b9d43f16b5f8d` ("feat(1.2): reasoning passback, approval boot expiry, recovery timeout; foresight run-1 plans/ledger")
+ uncommitted WP-01 Lead changes in:
- `backend/crates/hiverouter-tools/src/approval.rs`
- `backend/crates/hiverouter-tools/src/lib.rs`
- `backend/crates/hiverouter-core/src/tools_bridge.rs`
(+ untracked `tools/qa/apply_wp01.py`, `plans/foresight/RUN2-*.md`).

**Constraints observed:** no release build, no paid/live provider call, no commit. Debug offline tests only; portable WinLibs GCC 16.2.0 in `%TEMP%` (not repo) used because the system GNU toolchain has no linker.

## 1. Reviewed change (diff scope)

- `approval.rs`: new partial unique index `approvals_one_pending_per_call ON approvals (run_id, tool_call_id) WHERE decision IS NULL`; new CAS primitives `expire`, `cancel_pending`; readers `decision_label`, `count_for_run`; new unit test `expire_cancel_and_decision_labels_are_cas_guarded`.
- `lib.rs`: re-exports `cancel_pending, count_for_run, decision_label, expire` (plus existing symbols).
- `tools_bridge.rs`: `with_approval_ttl` test seam; delegates `expire_approval`, `cancel_approval`, `approval_decision`, `approval_count_for_run`, `verify_tool_grant`.
- Existing WIP (Run-1) boot orphan reaper `expire_orphaned_pending` and TTL bulk `expire_pending` are expected to stay distinct per `plans/foresight/COMPILED-LEDGER.md` WP-01 and `CODE-APPROVAL-ENGINE.md` §4.5 merge note.

## 2. Suspicions — three hypotheses each, elimination

### S1: Migration safety on an existing DB with duplicate pending `(run_id, tool_call_id)` rows

Plan explicitly flags U2 ("query before index; do not delete audit rows to force the index") and requires unique-index errors to use the existing unique-violation refusal (`COMPILED-LEDGER` WP-01 blockers; `CODE-APPROVAL-ENGINE` §4.5 risk note).

| # | Hypothesis | Probe | Result |
|---|---|---|---|
| H1 | Duplicate pending rows make `CREATE UNIQUE INDEX` fail; `execute_batch` maps it through generic `From<rusqlite::Error>` to `E_INTERNAL_ERROR` and the index is absent | Fixture: migrate fresh DB, drop index, insert two pending rows for one `(run_id, tool_call_id)`, call `migrate` | **CONFIRMED RED**: `code=E_INTERNAL_ERROR`, `UNIQUE constraint failed: approvals.run_id, approvals.tool_call_id`, `approvals_one_pending_per_call` absent |
| H2 | Duplicates cannot exist because the producer dedupes first | Producer dedupe is in-Rust and only sees currently pending rows; a legacy/concurrent DB or direct gate insertion can still hold duplicates. Fixture inserted two rows directly through `ApprovalGate::request` | Eliminated — duplicate pending state is reachable |
| H3 | Migration failure is harmless because `build_tools` logs and continues and the boot orphan reaper clears pending rows, so the next boot creates the index | Read `http.rs` boot order (migrate → log/continue → reaper) and tested the recovery: after `expire_orphaned_pending`, `migrate` succeeds and the index appears | Partially true (no data loss, boot available, next boot repairs), but the WP-01 acceptance "unique index rejects a second pending row" is unmet on that boot and the failure was generic/unnamed — **defect** |

**Confirmed defect R2-1 (migration safety/naming).** Fixed: `migrate` now creates the base table + legacy indexes first, runs a duplicate-pending preflight query, and returns the existing `E_APPROVAL_REPLAY` refusal with a clear detail when duplicates block the unique index; no rows are deleted. A defensive `is_unique_violation` fallback maps a race-time index-creation violation to the same named refusal.

### S2: Unique partial index behavior / insert refusal semantics

| # | Hypothesis | Probe | Result |
|---|---|---|---|
| H1 | Index rejects a second pending row for the same call | Fixture: two `ApprovalGate::request` with same `(run_id, tool_call_id)` after normal migrate | **Verified**: second insert refused `E_APPROVAL_REPLAY` (existing `insert_request` unique mapping); count remains 1 |
| H2 | Index blocks re-requests/audit rows forever | Predicate is partial `WHERE decision IS NULL`; decided/expired rows do not participate | Verified by source + fixture (orphan expiry then re-migrate succeeds) |
| H3 | Migration deletes audit rows to force the index | Fixture asserts both duplicate rows remain present and are marked `expired` only by the boot orphan reaper; migration itself returns before any deletion | Verified — no deletion |

### S3: CAS behavior and the TTL/boot/per-wait distinction

| # | Hypothesis | Probe | Result |
|---|---|---|---|
| H1 | `expire`/`cancel_pending` return `true` only when this call wins `decision IS NULL` | Fixture: first CAS true, second CAS false, label `expired`/`cancelled`, count increments | **Verified** |
| H2 | `expire_pending` bulk TTL path duplicates the CAS SQL, contradicting WP-01's "one CAS predicate per primitive" merge | Read `approval.rs` before fix: direct `UPDATE approvals ... decision IS NULL` inside `expire_pending` in addition to `expire` | **CONFIRMED acceptance deviation**; fixed by having the TTL loop call `expire(conn, &id, now)` and count `true` |
| H3 | The boot path accidentally uses the live-wait `expire` primitive | Read `ToolsBridge::expire_pending_approvals` → `expire_orphaned_pending` (unconditional boot CAS); live timer is WP-03 (`expire_approval`) | Verified — distinction preserved; `expire_orphaned_pending` is not used by the live timer |

### S4: `decision_label` / refusal semantics

| # | Hypothesis | Probe | Result |
|---|---|---|---|
| H1 | Pending row → `None`; after CAS → stored label | Fixture: `None`, then `expired`/`cancelled` | **Verified** |
| H2 | Stored `expired` row resolves as a replay/forgery signal rather than timeout | Pre-existing WIP fix (Run-1) in `resolve`/`resolve_with_reason`; approval unit tests assert `E_APPROVAL_TIMEOUT` | Verified via unit suite (16 passed) |
| H3 | Unknown/decided id is misreported as a DB error | Source read: `query_row(...).optional().map(Option::flatten)`; unknown id → `None`, DB failure → `ToolError::from` | Verified by source; unknown-id runtime case not separately executed (unverified residual) |

### S5: `count_for_run` type / semantics

| # | Hypothesis | Probe | Result |
|---|---|---|---|
| H1 | Count type mismatched (`i64` returned as `u64`) | Fixture counts 1 then 2; function converts with checked `u64::try_from` | **Verified** |
| H2 | Count includes only pending rows instead of all decisions | SQL `COUNT(*) WHERE run_id = ?` (all decisions); plan acceptance says counts all decisions | Verified |
| H3 | Overflow path fabricates a count | Explicit `InternalError` on conversion failure; SQLite COUNT cannot be negative | Verified by source |

### S6: Bridge delegate error mapping / `with_approval_ttl` / dead code

| # | Hypothesis | Probe | Result |
|---|---|---|---|
| H1 | Bridge delegates flatten named refusals into generic DB errors | Read each delegate: `db.with_connection_refusal(... .map_err(|error| error.refusal()))`; `with_connection_refusal` explicitly preserves named refusals | No defect proven; compile verified |
| H2 | `with_approval_ttl` does not actually reach approval creation | Read: builder replaces `self.gate = ApprovalGate::with_ttl(ttl_secs)`; `request_model_tool_approval` uses `self.gate` | Source-verified; runtime TTL value not executed (residual) |
| H3 | New bridge primitives are dead code and should be deleted now | Plan marks WP-01/WP-02 as prerequisites compiled as dead code until WP-03 engine; `verify_tool_grant`/`expire_approval` are the intended engine seam; boot still uses `expire_orphaned_pending` | Expected per plan; `expire_pending` (TTL bulk) has no production caller yet and is kept per WP-01 merge note |

## 3. Reproduction and fix evidence

### Reproduction command

```
cd backend
cargo test -p hiverouter-tools --test lead_bugcheck_wp01 --offline -- --nocapture
```

**RED before fix (migration with legacy duplicates):**

```
migrate duplicate refusal: code=E_INTERNAL_ERROR display=E_INTERNAL_ERROR: An internal invariant failed; ... (SQLite failure while recording tool state: UNIQUE constraint failed: approvals.run_id, approvals.tool_call_id)
thread 'migration_on_legacy_duplicate_pending_rows_is_safe_and_named' panicked:
assertion `left != right` failed: the unique-index failure must use an existing named approval refusal, not a generic internal error
  left: "E_INTERNAL_ERROR"  right: "E_INTERNAL_ERROR"
test result: FAILED. 2 passed; 1 failed
```

**GREEN after fix (run twice):**

```
running 3 tests
test unique_pending_index_rejects_second_pending_row_for_the_same_call ... ok
test expire_and_cancel_are_cas_guarded_and_labels_are_readable ... ok
test migration_on_legacy_duplicate_pending_rows_is_safe_and_named ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
```
migrate duplicate refusal: code=E_APPROVAL_REPLAY display=E_APPROVAL_REPLAY: ... (1 duplicate pending approval group(s) ... audit/expire them and retry (no rows were deleted))
test result: ok. 3 passed; 0 failed
```

### Other commands / evidence

- `cargo test -p hiverouter-tools --lib approval:: --offline` → **16 passed; 0 failed** (Lead's new test + existing suite).
- `cargo test --workspace --offline` → **exit 0**, all observed suites `0 failed`; tools suites **97 passed**; reproduction 3 passed.
- `cargo check --workspace --tests --offline` → **exit 0**.

### Fixes (minimal, only the scoped files)

1. `backend/crates/hiverouter-tools/src/approval.rs`
   - `migrate` (`:149-205`): split DDL so the base table + two legacy indexes are created before the partial unique index; run a duplicate-pending preflight query (`GROUP BY run_id, tool_call_id HAVING COUNT(*) > 1`); return the existing `E_APPROVAL_REPLAY` refusal with a clear detail and **delete nothing**; create the unique index only when the preflight is clean; defensively map a race-time unique constraint via `is_unique_violation` to the same refusal.
   - `expire_pending` TTL loop (`:710+`, call at `:735`): now delegates to the single-row `expire` CAS instead of duplicating the `UPDATE ... decision IS NULL` predicate, satisfying WP-01's one-CAS-per-primitive merge requirement.
2. `backend/crates/hiverouter-tools/tests/lead_bugcheck_wp01.rs` (new): three regression tests — unique index rejects a second pending row; migration on legacy duplicate pending rows refuses with `E_APPROVAL_REPLAY`, preserves both audit rows, and succeeds after the boot orphan reaper; expire/cancel CAS + label + count semantics.
3. No changes to `lib.rs` or `tools_bridge.rs`: exports/delegates inspected, match the WP-01 contract, and compiled green; no proven defect.

### Files changed by this agent (sha256)

| File | sha256 |
|---|---|
| `backend/crates/hiverouter-tools/src/approval.rs` | `d66013857650c9ea4da594207cdda4d7a1ebbdb2b5ce7c022fc8d90b2c7455a4` |
| `backend/crates/hiverouter-tools/tests/lead_bugcheck_wp01.rs` | `ee31937131c35fed09465aed3c18f6074094b9140ad84d8b9115dba06051bd66` |
| `backend/crates/hiverouter-tools/src/lib.rs` (reviewed, unchanged by agent) | `e9f3b061df48ea327d5d2bb88dd30efdec5c0230f0778c2f8607938c80229dc6` |
| `backend/crates/hiverouter-core/src/tools_bridge.rs` (reviewed, unchanged by agent) | `63c2257f98e0651705d91d4557ab706ed7399af050bd49beaed5ca43a575dfd1` |

No commit; working tree still contains the Lead's uncommitted WP-01 changes.

## 4. Stub sweep

`Select-String` over all four reviewed/changed files for `TODO|FIXME|XXX|HACK|placeholder|NotImplemented|not implemented|lorem|simplified for|dummy` → **zero hits**. The new test file uses in-memory SQLite only; no network/paid path.

## 5. Residual / not fixed (explicit)

- **Dirty-DB first boot still lacks the unique index.** With duplicate pending rows, `migrate` now refuses by name (`E_APPROVAL_REPLAY`) and creates no index; `build_tools` logs and continues, then the boot orphan reaper consumes every pending row; the **next** boot creates the index. Audit rows are preserved (plan: "do not delete audit rows to force the index"). This is the plan's unresolved U2 risk; the alternative (expiring duplicate rows inside migrate) would mutate decisions outside the boot reaper and was not taken.
- **`build_tools` log wording** (`http.rs`) still says "tools approval-table migration failed; APPROVAL_RESOLVE will refuse by name" even though APPROVAL_RESOLVE keeps working and only the index is deferred; `http.rs` is outside this run's three-file review scope, so left unchanged.
- **Bridge delegates / `with_approval_ttl` are not runtime-exercised** (compiled green + source-read; their consumers are the WP-03 engine). `verify_tool_grant` error mapping is read, not executed through the bridge.
- **`expire_pending` has no production caller** after the boot path uses `expire_orphaned_pending`; kept per WP-01 merge note (live TTL bulk reaper). If the engine wave does not wire it, it should be removed rather than silently retained.
- **Unknown approval-id `decision_label` runtime case** is source-verified (`optional().flatten()` → `None`) but not separately tested.
- No release build; no paid/live provider call.

## 6. Evidence-check claim table

| Claim | Evidence | Status |
|---|---|---|
| Legacy duplicate pending rows block the unique index | RED test output + fixture | **verified (pre-fix)** |
| Pre-fix migrate returned generic `E_INTERNAL_ERROR` and left index absent | RED output + sqlite_master assertion | **verified (pre-fix)** |
| Fixed migrate refuses with existing `E_APPROVAL_REPLAY`, deletes no rows, and index succeeds after orphan expiry | GREEN test run twice; audit rows checked | **verified** |
| Unique index rejects a second pending `(run_id, tool_call_id)` | GREEN test | **verified** |
| `expire`/`cancel_pending` CAS win once; labels/count correct | GREEN test | **verified** |
| `expire_pending` now shares the per-row CAS | Source lines + 16 approval tests + workspace | **verified by source/tests** |
| Bridge delegates preserve named refusals | Source read + compile | **verified by source/compile; no runtime call test** |
| `with_approval_ttl` reaches request creation | Source read | **unverified runtime** |
| Whole backend still green | `cargo test --workspace --offline` exit 0; tools 97 passed | **verified** |
| No release build / paid call | Commands are debug offline only | **verified by command scope** |

## 7. Done / not done / unverified

**Done**
- Read-only review of the WP-01 diff and the new test; three hypotheses per suspicion recorded and eliminated/confirmed.
- R2-1 fixed: duplicate-pending migration preflight + existing named `E_APPROVAL_REPLAY` mapping; no audit rows deleted; unique index still created on a clean DB and accepts the retry after boot orphan expiry.
- R2-2 fixed: `expire_pending` now delegates to the single-row `expire` CAS (no duplicated predicate).
- Regression test file `lead_bugcheck_wp01.rs` added (3 tests; RED quoted pre-fix, GREEN twice).
- Full workspace debug suite green; compile check green.

**Not done**
- No fix for the dirty-DB one-boot index deferral beyond named refusal (plan U2: do not delete audit rows; boot reaper repairs next boot).
- No release build, no paid/live call, no commit.

**Unverified**
- `tools_bridge` delegates and `with_approval_ttl` at runtime (source + compile only; consumers land in WP-03).
- Unknown-approval-id `decision_label` runtime branch.
- `expire_pending` production wiring (currently no caller).
- The `build_tools` log wording mismatch in `http.rs` (outside this run's scope).
