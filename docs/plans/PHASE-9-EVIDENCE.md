# PHASE 9 EVIDENCE — State, Persistence & Memory

Date: 2026-11-06 (America/Chicago). Executor: B-Core. Recon: `PHASE-9-RECON.md`.
Commands from `backend/` with the Phase 1 PATH/env (see PHASE-1-EVIDENCE preamble).

## 1. Preflight + full suite

`cargo test -p hiverouter-core` (post disk-wipe rebuild, Phase 9 modules compiled):

```
running 34 tests (lib)                                   test result: ok. 34 passed; 0 failed
Running tests\kill_restore.rs                            running 3 tests
running 1 test
test result: ok. 2 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 8.20s
Running tests\snapshot_history.rs                        running 4 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.96s
Running tests\chat_store.rs                              running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
CORE_TEST_EXIT=0
```

The single ignored test is the crash-writer helper process, which is spawned by `kill_restore` (it hard-kills itself and
therefore cannot run inside the normal harness).

## 2. Acceptance tests (blueprint Phase 9 card)

| Card item | Test | Behaviour asserted |
|---|---|---|
| `cargo test kill_restore` | `tests/kill_restore.rs::kill_restore` | child helper process writes a `running` run + committed snapshot then `std::process::abort()`; parent asserts the child was killed, re-opens the database, runs `recover_on_boot` -> run row `interrupted`, latest snapshot restored into the graph (2 nodes), report lists the run |
| `cargo test migration_roundtrip` | `tests/snapshot_history.rs::migration_roundtrip` | snapshot `v2 -> v1 -> v2`: graph payload byte-equal after the round-trip, revision/run id preserved, v1 carries no `run_status`, unknown schema version -> `E_SNAPSHOT_CORRUPT` |
| error-path corrupt snapshot | `corrupt_snapshot` + `kill_restore::corrupt_snapshot_at_recovery_is_refused_not_reset` | corrupt JSON -> `E_SNAPSHOT_CORRUPT`, file left on disk, run row still recovered, workspace never silently reset |
| `cargo test history_api` | `tests/snapshot_history.rs::history_api_returns_real_revisions` + `history_http_api` | time-travel view reports the real 2 revisions; `state_at(1)` returns `run_status=completed` + 2 graph nodes; missing revision -> `E_HISTORY_GAP`; HTTP `GET /api/history/runs`, `/revisions`, `/state?revision=1` return 200 with real data, missing revision -> 404 `E_HISTORY_GAP` |
| SQLite migration round-trip without data loss | `tests/chat_store.rs::migration_v2_down_and_up_roundtrip` (v3 -> v2 -> v0 -> v3) | down/up applies every inverse and re-applies the chain |

## 3. Implementation facts (read back from the code)

- Persistence split (blueprint WP 9.8): SQLite owns run/task/chat rows + the snapshot index (migration v3
  `snapshots(run_id, revision, path, created_at, schema_version)`); the graph is memory + one versioned JSON file per
  revision under `<data-dir>/snapshots/<run-id>/<revision>.json`.
- `src/snapshot.rs`: `SnapshotEnvelope` (schema_version, run_id, revision, created_at, GraphSnapshot, run_status);
  atomic write (temp file + rename) then SQLite index row; `read` migrates older schema versions; `latest`,
  `list_revisions`, `runs_on_disk`, `indexed_revisions`; `migrate_v1_to_v2` / `migrate_v2_to_v1` are the exercised inverse
  pair; `capture(graph, ...)` builds the envelope.
- `src/graph.rs::restore_from_snapshot`: rebuilds nodes+edges from the snapshot, refuses `E_SNAPSHOT_CORRUPT` for unknown
  edge endpoints or a cyclic restore, restores the revision counter.
- `src/recovery.rs::recover_on_boot`: marks every non-terminal run `interrupted`, restores each run's latest snapshot,
  leaves corrupt snapshots on disk and reports them (never resets the workspace).
- `src/history.rs`: `HistoryReadModel` (runs, revisions, state_at) and `TimeTravel` read model with a real
  `revision_count()` for the scrubber; `E_HISTORY_GAP` for a revision that is not readable.
- `src/http.rs`: `GET /api/history/runs`, `GET /api/history/runs/{run_id}/revisions`,
  `GET /api/history/runs/{run_id}/state?revision=N` (JSON, named refusals).
- `src/main.rs`: boots through `recover_on_boot` before serving; the recovered graph is handed to `AppState::with_graph`;
  a failed recovery refuses to start the process instead of serving a corrupt workspace.

## 4. Deviations recorded (not silently resolved)

1. ChromaDB/LanceDB/Python embedding memory is out of v1 per the blueprint Phase 9 scope rewrite; the v1 memory surface is
   the graph + run rows + versioned snapshots (no vector store was invented).
2. Cloud/bucket disaster recovery (source Phase 116) stays local-only per the D1 fence; no cloud path exists.
3. `run_status` exists only in v2 snapshots; v1 files load with `run_status: None` (recorded in the migration test).

## 5. Done / Not done / Unverified

| Done | Evidence |
|---|---|
| persistence split: SQLite rows + versioned JSON snapshots with index | §2, §3 |
| snapshot schema_version + v1->v2->v1 migration round-trip | §2 `migration_roundtrip` |
| kill -> restart: interrupted run row + snapshot restore | §2 `kill_restore` (real hard-killed process) |
| corrupt snapshot refused, never silently reset | §2 |
| history/replay read API + undo/time-travel read model | §2 `history_api` |

| Not done (reason) | |
|---|---|
| vector/embedding memory (FTS/JSON vectors) | removed from v1 by blueprint Phase 9 scope rewrite |

| Unverified | What would prove it |
|---|---|
| multi-run snapshot restore at scale | stress test with many runs (not part of the phase gate) |
