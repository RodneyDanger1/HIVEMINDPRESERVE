# PHASE 9 RECON — State, Persistence & Memory

ForgeWright recon ledger (skill: `forgewright-recon`). Written BEFORE any Phase 9 edit.

## Recon

- **Target behavior today.** Phases 1-4 provide: SQLite migrations v1/v2 (app_meta, runs, tasks, chat_messages,
  command_receipts), a typed run/task store, the run engine, the event bus and the petgraph graph authority. There is
  **no** durable graph snapshot, no snapshot schema versioning/migration, no kill-restore recovery, no history read API
  and no undo/time-travel read model.
- **Entry points.** New core modules `src/snapshot.rs` (versioned JSON snapshot store + migrator), `src/recovery.rs`
  (boot recovery), `src/history.rs` (history read API + time-travel view); `GraphAuthority::restore_from_snapshot` added
  to `src/graph.rs`; migration v3 (`snapshots` index) added to `src/db.rs`; HTTP routes added to `src/http.rs`;
  `src/main.rs` boot calls recovery and records the newest snapshot after each run.
- **Callers.** `main.rs` boot (recovery), `http.rs` routes (history API), `RunEngine` dispatch (snapshot write per
  completed/failed run), Phase 10 UI scrubber (read API), tests.
- **Tests + commands.** `cargo test -p hiverouter-core kill_restore`, `cargo test -p hiverouter-core
  migration_roundtrip`, `cargo test -p hiverouter-core corrupt_snapshot`, `cargo test -p hiverouter-core history_api`.
- **Data flow (one path + one failure path).** Run dispatch completes/fails → `SnapshotStore::write(run_id, revision,
  run_status, graph)` serialises a versioned JSON envelope atomically (write temp file, fsync, rename) and records the
  revision in the SQLite `snapshots` index (v3 migration) → history API reads the index + file → `GET
  /api/history/runs/{id}/revisions` returns real revision numbers, `GET .../state?revision=N` returns the stored
  snapshot state.
  Failure path: a corrupt snapshot file (or a schema_version the migrator cannot upgrade) is refused with
  `E_SNAPSHOT_CORRUPT` and the workspace is never silently reset; recovery leaves the corrupt file on disk and reports
  the refusal.
  Kill path: a run is persisted as `running`, its last committed snapshot is on disk, the process is hard-killed (test
  helper aborts) → next boot `recover_on_boot` marks the run `interrupted`, restores the latest snapshot into the graph
  authority and reports what was recovered.
- **Snapshot schema versioning/migration.** `SnapshotEnvelope` carries `schema_version` (`hivemind.snapshot.v1`); the
  migrator upgrades v1 → v2 (adds `run_status`) and downgrades v2 → v1. `migration_roundtrip` asserts the graph payload
  is byte-equal after up→down and that the downgraded file loads.
- **Unknowns / evidence still missing.** No live provider in this phase (Phase 5 seam); snapshot writes are triggered by
  the run engine after a run reaches a terminal state, and the kill-restore test drives the store directly through the
  helper process so it does not depend on provider behaviour. Vector/embedding memory stays out of v1 (blueprint Phase 9
  scope rewrite): SQLite FTS/JSON vectors are not implemented; the store is graph + run rows + snapshots, which is the
  authoritative v1 memory surface.
- **Files expected to touch.** new `src/{snapshot.rs,recovery.rs,history.rs}`, new
  `tests/{kill_restore.rs,snapshot_history.rs}`; edits to `src/{db.rs,graph.rs,http.rs,main.rs,execution.rs,lib.rs}`;
  `docs/plans/PHASE-9-{RECON,EVIDENCE}.md`; `docs/claims/*`. **Never:** provider/tools, `web/`, FigCraft, blueprint.
- **Rollback.** Snapshot files live under the configured data directory; migration v3 has a tested down-migration;
  removing the history routes restores Phase 4 HTTP surface.

**Recon completeness:** complete — every module this phase edits (`db.rs`, `graph.rs`, `http.rs`, `main.rs`,
`execution.rs`) has been read; snapshot/recovery/history are new modules with no existing caller to preserve.
