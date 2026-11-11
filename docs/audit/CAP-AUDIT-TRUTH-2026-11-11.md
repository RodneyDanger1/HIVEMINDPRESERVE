# CAP-1 truth audit — session policy, permission policy, checkpoints

**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1`
**Branch:** `capability-expansion`
**HEAD:** `94ba10a` (`feat(api): expose checkpoint list/detail/restore/revert routes`)
**Audit date:** 2026-11-11 (America/Chicago)
**Auditor:** delegated truthseeker subagent
**Scope:** independent runtime verification of the CAP-1/CAP-5 claims. No product
source, existing test, plan document or tools-crate file was edited.

**Owned files (only these are new from this audit):**

- `backend/crates/hiverouter-core/tests/session_policy_truth.rs`
- `docs/audit/CAP-AUDIT-TRUTH-2026-11-11.md` (this file)

## Method and harness

`tests/session_policy_truth.rs` owns its fixture. It starts the **real**
`http::build_router`/`AppState` on an ephemeral loopback listener, uses the real
`RunStore`/`Database`, the real ordered `db::MIGRATIONS`, and the real
`CheckpointJournal` over a temporary workspace. There are no route handlers, store
methods, migrations, checkpoint operations or JSON responses mocked or stubbed.
`tests/http_boot.rs` was read only as a reference for the TCP fixture pattern.

The suite hosts every audited route without source changes. It does not need a
provider: the session/policy routes and checkpoint routes are local-only. The one
claim that cannot be driven through the HTTP surface (`trusted` bypassing a tool
approval) is probed structurally at the single `decide_policy` call site; see D3.

### Result

| Run | Binary/source | Result |
|---|---|---|
| In-repo worktree | `cargo test -p hiverouter-core --test session_policy_truth` | **11 passed, 3 failed, 0 ignored** (14 tests), `finished in 6.42s` |
| Pristine `git archive` of `94ba10a` + the same test file | same command, separate `CARGO_TARGET_DIR` | **not completed** — the host ran out of disk (`There is not enough space on the disk`) while extracting/building; see U8 |

The in-repo tree was clean at audit start. At 11:24, 11:31 and 11:38 local time a
concurrent sibling wrote `src/checkpoint.rs`, `src/tools_bridge.rs` and
`tests/checkpoint_hook.rs` (including a `TEMPORARY RED INJECTION` comment in
`tools_bridge.rs`). None of those files are mine. The in-repo run compiled that
dirty tree, so its evidence is reported with this caveat:

- `git diff` on `checkpoint.rs` only **adds** `void_entry`,
  `record_after_or_discard`, `remove_entry` and a `NotRemovable` variant. It does
  not modify `entries_for_run`, `get`, `query_entries`, `restore_entry`,
  `restore_to`, or any route.
- `tools_bridge.rs` adds an optional journal and the `with_checkpoint_journal`
  setter; `http.rs` (clean) never calls it, and the audited routes open their own
  journal (`http.rs:985-988`).
- `http.rs`, `db.rs`, `store.rs`, `execution.rs`, `tool_loop.rs` and `policy.rs`
  were clean at the time of the run.

The defects are therefore in unchanged files and are HEAD-valid; a pristine build
is still unverified (U8). The temporary archive/target were deleted to restore
disk; no repo file was touched by that cleanup.

**Exact command (w64devkit on `PATH`):**

```powershell
cd C:\Users\Riley\Desktop\hiverouter-v1\backend
$env:PATH = "C:\Users\Riley\Desktop\hiverouter-v1\tools\w64devkit\bin;$env:USERPROFILE\.cargo\bin;$env:PATH"
cargo test -p hiverouter-core --test session_policy_truth
```

**Exact tail of the run:**

```text
test result: FAILED. 11 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.42s
```

The failing tests are the three `*_truth_defect_*` / wiring-probe tests; every
other assertion passed. The three failures are the evidence for D1, D2 and D3,
not harness errors.

## Claim register

Grade: `P` = primary evidence retrieved and quoted (executed test, raw DB read, or
direct source quote); `D` = plan/draft contract text only; `U` = not verified.
A refuted claim keeps grade `P` because the evidence is primary; the verdict is in
the claim text and the change column.

| ID | Claim | Grade | Evidence (quote / test) | Change from prior version |
|---|---|---|---|---|
| C1 | A run created without an explicit policy stores `classic`; the DB default, wire value and readback agree. | P | `create_run_defaults_to_classic_and_no_session` passes: `create_run` returns `Classic`, raw `SELECT permission_policy` is `"classic"`, `session_id` is `NULL`. `db.rs:902` `DEFAULT 'classic'`. | `D` (CAP-1 plan) → `P` (verified) |
| C2 | `trusted` is persisted only when explicitly requested — never inferred from a prompt, a trusted session, or the default. | P | `explicit_trusted_is_persisted_only_when_asked_for` passes: scoped trusted run is `trusted`; a plain `create_run` next to a trusted session is `classic`; only `set_run_policy` changes it. `permission_policy_default_is_classic_and_unknown_is_not_a_value` passes. | `D` → `P` (verified) |
| C3 | Session policy is stored and read back for all four values (`classic`, `plan`, `standard`, `trusted`) and the default is `classic`. | P | `session_create_stores_and_reads_back_each_policy` passes for all four: `POST /api/sessions` → 201, `GET /api/sessions/{id}` and `GET /api/sessions` agree, raw `sessions.permission_policy` agrees; a body without the field is `classic`. | `D` → `P` (verified) |
| C4 | `POST /api/sessions` refuses missing, blank or non-string required fields with `400 E_SCHEMA_INVALID` and creates nothing. | P | `session_create_rejects_missing_and_blank_required_fields` passes for 8 malformed bodies; `store.sessions()` stays empty. `http.rs:904-918`. | `U` → `P` (verified) |
| C5 | `POST /api/sessions` refuses an **explicit unknown/cased/numeric/null** policy instead of fabricating a policy. | P | **DISPROVEN.** `session_create_truth_defect_rejects_unknown_explicit_policy` fails: all four bodies return `201` with `"permission_policy":"classic"` and create rows (`before=0, after=4`). Source: `http.rs:919-923` `from_str_strict(...).unwrap_or(PermissionPolicy::Classic)`. | `U` → `P` (refuted; D1) |
| C6 | `POST /api/sessions/{id}/policy` updates a valid policy, returns the updated row, and readback/DB agree; it does not retroactively rewrite an existing run's explicit policy. | P | `session_policy_update_reads_back_and_does_not_retroactively_change_runs` passes: 200 with `trusted`, detail/list/DB `trusted`; a pre-existing run explicitly set to `trusted` stays `trusted` after the session is moved to `plan`. | `D` → `P` (verified) |
| C7 | The policy route refuses an invalid/missing policy with 400 and an unknown session with `404 E_SESSION_NOT_FOUND`; a refused update leaves the stored policy unchanged. | P | `session_policy_update_refuses_bad_input_and_unknown_sessions` passes for unknown string, wrong case, numeric, missing field, and unknown id. | `U` → `P` (verified) |
| C8 | A real v24 database upgrades to v27 preserving run/task/message/receipt/snapshot rows; old runs become `classic` with no session; 27→24→27 preserves the legacy rows. | P | `v24_database_migrates_to_v27_without_losing_legacy_rows` passes: applied names `sessions_policy_todos` (25), `repo_index` (26), `checkpoint_journal` (27); run tuple preserved, policy `classic`, `session_id NULL`; counts 1/1/1/1; down→up preserves the legacy run. | `D` → `P` (verified; in-memory only — see U1) |
| C9 | Checkpoint list/detail return the real journal row (path and before/after SHA-256), restore returns the real pre-image and records `restored_at`, and drift is refused without touching the file. | P | `checkpoint_routes_return_real_list_detail_and_restore_data` passes: `GET` list/detail expose `notes/a.txt` with the real `alpha`/`beta` digests; restore returns 200 and the file bytes are `alpha`; a second restore returns `409 E_INTERNAL_ERROR` and leaves `alpha`; `restored_at` is set. | `D` → `P` (verified) |
| C10 | Checkpoint routes refuse an absent run/entry instead of returning fabricated success. | P | **DISPROVEN for absent runs.** `checkpoint_truth_defect_refuses_absent_run` fails: `GET /api/runs/{absent}/checkpoints` → `200 {"checkpoints":[]}`; `POST .../revert` → `200 {"restored":[]}`; `POST .../1/restore` → `409 E_INTERNAL_ERROR`. Source: `http.rs:990-1064`; `checkpoint.rs:657-659`, `724-731` never require the run for reads. | `U` → `P` (refuted; D2) |
| C11 | The `standard`/`trusted` `Execute` decision is honored by the only execution call site, so those policies do not ask for every Mutate/Execute tool. | P | **DISPROVEN.** `trusted_execute_decision_is_wired_to_the_execution_path` fails: the call-site window contains only `PolicyDecision::Refuse => ...` and `_ => { ... suspend_for_approval(...) }`. `execution.rs:2894-2922`; `tool_loop.rs:17-25` defines `Execute` as "without an interactive approval". | `D` → `P` (refuted; D3) |
| C12 | A missing checkpoint **entry** is a refusal, not fake success. | P | `checkpoint_detail_and_restore_refuse_absent_entries` passes: detail 404 `E_RUN_NOT_FOUND`; restore 409 `E_INTERNAL_ERROR`; revert of a real run with no entries 200 `{"restored":[]}`. The refusal exists but the code is generic; see U4. | `U` → `P` (refusal confirmed; code fidelity not) |

## Defect register

### D1 — unknown explicit session policy is silently materialized as `classic`

**File:line:** `backend/crates/hiverouter-core/src/http.rs:919-923`

```rust
let policy = body
    .get("permission_policy")
    .and_then(serde_json::Value::as_str)
    .and_then(PermissionPolicy::from_str_strict)
    .unwrap_or(PermissionPolicy::Classic);
```

**Claim violated:** C5. The policy route (`http.rs:968-974`) correctly refuses the
same inputs with 400; the create route alone converts `None` into a real policy.
This is the "plausible default" failure mode: the caller asked for something the
server does not support and the server records a policy the caller did not choose.
It is not privilege escalation (a typo cannot produce `trusted`), but it is a
fabricated default and hides the error.

**Captured wrong output (from the failing test):**

```text
unknown string: expected 400 E_SCHEMA_INVALID, got 201 {"...","name":"invalid-policy-0","permission_policy":"classic",...}
wrong case:    expected 400 E_SCHEMA_INVALID, got 201 {"...","name":"invalid-policy-1","permission_policy":"classic",...}
numeric:       expected 400 E_SCHEMA_INVALID, got 201 {"...","name":"invalid-policy-2","permission_policy":"classic",...}
null:          expected 400 E_SCHEMA_INVALID, got 201 {"...","name":"invalid-policy-3","permission_policy":"classic",...}
4 invalid request(s) still created a session (before=0, after=4)
```

**Not fixed** (audit scope). The minimal fix is to require a string, parse it
strictly, and return `SchemaInvalid` on `None`; omit the field to get `classic`.

### D2 — checkpoint routes let an absent run masquerade as a real empty journal

**File:line:** `backend/crates/hiverouter-core/src/http.rs:990-1064`;
`backend/crates/hiverouter-core/src/checkpoint.rs:657-659` and `724-731`.

`run_checkpoints` calls `journal.entries_for_run(run_id)` and returns 200 for
whatever vector it gets; `entries_for_run` → `query_entries` filters by `run_id`
with no `require_run` check. `restore_to` returns `Ok(RestoreOutcome { restored: [] })`
when the entry vector is empty. The restore route maps a missing entry/run to
`409 E_INTERNAL_ERROR`. So a nonexistent run is indistinguishable from a real run
with no checkpoints on the list and revert routes.

**Captured wrong output (from the failing test):**

```text
GET list for absent run: expected 404 E_RUN_NOT_FOUND, got 200 {"run_id":"01a1cbb0-81e1-774f-8e40-a6cbe190d34b","checkpoints":[]}
POST revert for absent run: expected 404 E_RUN_NOT_FOUND, got 200 {"run_id":"01a1cbb0-81e1-774f-8e40-a6cbe190d34b","from_seq":1,"restored":[]}
POST restore for absent run: expected 404 E_RUN_NOT_FOUND, got 409 {"code":"E_INTERNAL_ERROR",...}
```

**Not fixed.** The minimal fix is to call `require_run` (or a public equivalent)
before listing/rewinding, and to map `CheckpointNotFound`/`RunNotFound` to 404.

### D3 — `standard`/`trusted` policy decisions are inert on the execution path

**File:line:** `backend/crates/hiverouter-core/src/execution.rs:2894-2922`

```rust
match tool_loop::decide_policy(
    run.permission_policy,
    class,
    inside_workspace,
) {
    tool_loop::PolicyDecision::Refuse => { /* refused */ }
    _ => {
        ...
        self.suspend_for_approval(run, task, call, &plan, turn, token).await?
    }
}
```

`decide_policy` returns `Execute` for `standard` (workspace Mutate) and `trusted`
(workspace Mutate/Execute) — `tool_loop.rs:31-59`. The only caller matches only
`Refuse`; `Execute` and `AskApproval` both enter `suspend_for_approval`, which
creates an approval request and waits (`execution.rs:3370+`). Therefore selecting
`standard` or `trusted` changes the stored/read-back policy but does **not** remove
per-action approval prompts; `plan` is the only policy with an observable effect
(it refuses). This contradicts CAP-1's `resolve_policy` pseudocode and the
`PolicyDecision::Execute` doc comment.

**Captured evidence:** the static wiring probe's failure message quotes the actual
call-site window (only `Refuse` + `_ => suspend_for_approval`).

**Not fixed.** The execution match needs an explicit `PolicyDecision::Execute`
arm that proceeds to the existing `execute_tool_plan` path (or an equivalent
approved path) without suspending; the `inside_workspace` gate must be preserved.

## Additional observations (not counted as the required defects)

- **`runs.session_id` has no foreign key.** CAP-PATCH-IMPACT line 22 says
  `ALTER runs ADD session_id TEXT NULL REFERENCES sessions(id)`. The shipped v25
  migration at `db.rs:901` is `ALTER TABLE runs ADD COLUMN session_id TEXT;` with
  no `REFERENCES`. The API path validates the session before creating a scoped run
  (`execution.rs:1591-1600`), so this is not reachable through the audited HTTP
  surface, but the store allows an orphan `session_id`. Grade `D → P` (source
  quote); runtime `PRAGMA foreign_key_list(runs)` not executed — see U5.
- **`session_detail` reports a truncated run list as complete.** `http.rs:940-955`
  calls `store.list_runs(200)` and filters by session; a session whose newest run
  is older than the global newest 200 returns `"runs":[]` with no truncation
  marker. Not exercised (requires >200 runs); grade `U`.

## Corrections log

| Before | After | Why |
|---|---|---|
| C5 assumed the create route would mirror the policy route's strict 400. | `P` REFUTED: 201 `classic` for all four invalid shapes. | Ran the real route; `unwrap_or(Classic)` is visible at `http.rs:922`. |
| C10 assumed a run-scoped checkpoint route would 404 an absent run. | `P` REFUTED: list/revert 200 empty, restore 409 generic. | Ran the real routes; no `require_run` on the read/rewind path. |
| C11/plan treated `standard`/`trusted` as functional auto-approval policies. | `P` REFUTED: only `Refuse` is matched; `Execute` still suspends. | Read the only `decide_policy` call site and probed it structurally. |
| C8 plan-only migration claim. | `P` verified for in-memory v24→v27→v24→v27. | Executed the real migration engine and seeded real v24 rows. |
| C1–C4, C6–C7, C9, C12 plan/assumption only. | `P` verified for the exercised paths. | Real HTTP/store/DB/journal roundtrips. |
| `runs.session_id` FK assumed from pseudocode. | `D → P` deviation: no FK in v25. | Source quote `db.rs:901`; PRAGMA not run. |

## Unverified stores (not "done")

| ID | Unverified | What would close it |
|---|---|---|
| U1 | v24→v27 on a **file-backed** database, including the pre-migration `VACUUM INTO` backup and WAL path. The audit used `:memory:`, which returns early from `backup_before_migration` (`db.rs:1121`). | Repeat `v24_database_migrates_to_v27_without_losing_legacy_rows` with `Database::open(temp.db)`, assert the `.pre-v24-*.bak` file exists and passes `PRAGMA integrity_check`. |
| U2 | End-to-end `trusted`/`standard` tool execution with a real provider turn (does the approval suspension actually disappear after an `Execute` arm is added?). | A provider-double integration test that drives one `file_write` tool call under `trusted`, asserts no `ApprovalRequest` event and one executed write; plus one denied/plan case. |
| U3 | Desktop/CLI/UI behavior of these routes (the Web GUI and CLI wrappers). | Browser/CLI e2e against the same binary; not run here. |
| U4 | Checkpoint refusal **codes**: missing entry/run returns `E_INTERNAL_ERROR` / `E_RUN_NOT_FOUND`, not a dedicated checkpoint code. The module docs say the contract has no checkpoint codes yet (`checkpoint.rs:38-47`). | Add/verify dedicated `E_CHECKPOINT_*` codes and assert them at the HTTP boundary. |
| U5 | `runs.session_id` FK enforcement at the SQLite layer. | `PRAGMA foreign_key_list(runs)` plus an insert of a run with a nonexistent session id, expecting a constraint failure. |
| U6 | Concurrent/crash restore atomicity under a mid-write process kill. | Kill the process between `validate_plan` and the rename; assert the journal and file agree after restart. |
| U7 | `POST /api/sessions` with a non-object JSON body (array/scalar) and with a `workspace_root` that does not exist. | Add those bodies to the validation test; the route currently returns 400 for non-object field access but does not validate workspace existence. |
| U8 | A pristine `94ba10a` rebuild. The archive/extract/build was attempted and aborted with `There is not enough space on the disk` (host free space ~3 MB at the time; repo target is 8.45 GB and shared with sibling agents). | Free several GB or build on another volume, then rerun the identical file in a clean `git archive` tree. The concurrent diffs that were present are additive only (see Result), but this is not a substitute for the clean run. |
| U9 | Session-policy inheritance by a **new run** (`submit_prompt_scoped` uses `requested_policy.unwrap_or(session.permission_policy)` at `execution.rs:1597`). The suite tests store-level explicit/scoped creation and the session readback, not the engine admission path: `submit_prompt_scoped` refuses at the `key_available`/provider gate before a run row is created. | A provider-double engine test that submits a prompt scoped to a `trusted` session with no explicit policy and asserts the created run row is `trusted`, plus a `classic` control. |
| U10 | `session_detail` completeness: `http.rs:940-955` filters the newest 200 global runs, so an older session can report `"runs":[]` with no truncation marker. | Seed >200 runs with controlled `created_at` values and assert the session detail either returns all its runs or an explicit truncation flag. |

## Stub / fake-code sweep

`Select-String` over the new test file for
`TODO|FIXME|XXX|HACK|placeholder|NotImplemented|not implemented|mock|stub|dummy|lorem|simplified for|for illustration`
returned three hits, all classified as permitted/benign:

- line 7 — the word "mock" in the doc comment saying the suite does **not** mock.
- `"sessions_policy_todos"` and `"todos"` — migration/table names, not placeholders.

No hit is a stub, placeholder, or fake success path.

## Command receipt (in-repo run)

```text
running 14 tests
...
test result: FAILED. 11 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.42s
```

Full output: `C:\Users\Riley\Desktop\session_policy_test_run2.log` (outside the
repo, so it is not part of the deliverable). The three failures are:

```text
failures:
    checkpoint_truth_defect_refuses_absent_run
    session_create_truth_defect_rejects_unknown_explicit_policy
    trusted_execute_decision_is_wired_to_the_execution_path
```
