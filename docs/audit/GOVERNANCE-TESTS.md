# Governance integration tests — result record

**Task:** `task-23` — independent HyperAgent governance integration tests.
**Date:** 2026-11-10 (America/Chicago).
**Repository:** `C:\Users\Riley\Desktop\hiverouter-v1`.
**Scope written:** only
`backend/crates/hiverouter-core/tests/hyperagent_governance.rs` and this file.
No production source or existing test was edited.

## Command

```powershell
$root = 'C:\Users\Riley\Desktop\hiverouter-v1'
Set-Location "$root\backend"
$env:PATH = "$root\tools\w64devkit\bin;C:\Users\Riley\.cargo\bin;$env:PATH"
cargo test -p hiverouter-core --test hyperagent_governance
```

## Result

```text
running 4 tests
test lower_level_request_detached_creates_attributed_remote_and_audit ... ok
test duplicate_request_detached_is_refused_and_does_not_duplicate_rows ... ok
test root_spawn_superagent_refusal_is_durable_in_the_run_bank ... ok
test rewire_to_a_foreign_run_parent_is_refused_before_mutation ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
```

**Counts:** 4 tests, 4 passed, 0 failed, 0 ignored, 0 filtered.
**Exit code:** 0.
**Compile blockers:** none after final edit. The first compiled run emitted
one `unused import: Mutex` warning in the new test file; the import was removed
and the final run had no warnings from this test target.

## Coverage and assertions

1. `lower_level_request_detached_creates_attributed_remote_and_audit`
   - Plan: one `super` branch; leaf calls `request_detached` with `kind=remote`
     and a `reason`.
   - Asserts exactly one remote; `owner_id = run.id`; `created_by` starts with
     `agent:` (the requesting task id).
   - Asserts one device and one grant are provisioned.
   - Asserts audit rows `remote_created`, `device_created`, `grant_created`
     all carry `actor_kind="system"` and `actor_id == created_by`; and a
     `hyper_admission` row with `actor_kind="system"`,
     `actor_id == created_by`, and `detail_json` containing
     `"decision":"allow"`.

2. `duplicate_request_detached_is_refused_and_does_not_duplicate_rows`
   - The same task returns two identical-name `request_detached` calls in one
     turn.
   - Asserts exactly one remote exists afterward.
   - Asserts the persisted `tool_turns` rows include a
     `request_detached` row with `code = E_DETACHED_CONFLICT`.

3. `root_spawn_superagent_refusal_is_durable_in_the_run_bank`
   - Plan already contains one `super` branch; the root orchestrator asks to
     `spawn_superagent` once, then `finish`.
   - Asserts run completes; exactly two tasks exist (root + planned branch),
     root role is `planner`, and exactly one `super` exists.
   - Asserts `store.memory_hyper_summary(run.id, 8)` contains a
     `root_gate:` entry whose value has `"action":"spawn_superagent"` and
     `"decision":"refused"`.

4. `rewire_to_a_foreign_run_parent_is_refused_before_mutation`
   - Starts an in-process `TestServer`; creates two runs, two roots, and a task
     under run A's root; sends `REWIRE_TASK` targeting run B's root as the new
     parent.
   - Asserts the WS guard returns `E_TASK_NOT_FOUND`.
   - Asserts the graph parent and the persisted task row both remain run A's
     root (no graph/context mutation).

## Notes

- The test file is self-contained (`mod support;` for the existing in-process
  server/WS client) and uses public engine/store/remote seams. It does not
  depend on `run_lifecycle.rs`, so it remains independent evidence.
- No provider key, network, production source, existing test, server restart or
  paid call was used.
- The working tree already contained other lanes' uncommitted changes
  (production source and audit docs) at run time; they were not authored or
  modified by this task.
