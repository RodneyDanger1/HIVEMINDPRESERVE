# DEPENDENCIES — b-core partition (contract + core)

Checkpoint `3988622`. Local graph derived from `use`/`mod` lines in the 54 assigned files (`Select-String` evidence in
EVIDENCE.md) plus the two manifests read in full.

## 1. Intra-partition module graph (compile-time)

```
hiverouter-contract
  lib.rs -> command, envelope, event, ids, refusal, schema, state, timestamp, usage
  command.rs -> envelope, ids, refusal, schema
  envelope.rs -> command, event, ids, refusal, schema, timestamp
  event.rs -> command, envelope, ids, refusal, state, usage
  ids.rs -> schemars(uuid1), serde, uuid
  refusal.rs -> serde, thiserror
  schema.rs -> command, envelope, event, refusal (schemars, jsonschema, OnceLock)
  state.rs / timestamp.rs / usage.rs -> serde, schemars (timestamp: std only)
  tests/common -> all public modules; golden_schemas -> schema+command+event; schema_roundtrip/ui_command_matrix -> schema+command+event
hiverouter-core
  lib.rs -> 22 modules (bus, caps, checks, config, context, db, entities, execution, graph, handshake, history,
            http, planner, prompts, recovery, snapshot, static_ui, store, tools_bridge, ws)
  bus.rs        -> contract::{Envelope,Event,Refusal}, tokio::sync::broadcast, uuid
  caps.rs       -> contract::Refusal
  checks.rs     -> std::process, serde
  config.rs     -> std, thiserror
  context.rs    -> contract::{TaskId,Refusal}
  db.rs         -> rusqlite, thiserror, contract::{Refusal,Timestamp}
  entities.rs   -> contract::{AgentState,RunId,TaskId,Timestamp}, serde, uuid
  execution.rs  -> async_trait, tokio, tokio_util, contract events/ids/refusals,
                   store, bus, graph, caps, planner, prompts, snapshot, context
  graph.rs      -> petgraph, serde, contract
  handshake.rs  -> contract::{RunId,TaskId,Refusal}, uuid, serde
  history.rs    -> db, entities, snapshot, contract
  http.rs       -> axum (+ConnectInfo/Path/Query), tower stack, all core services
  main.rs       -> config, db, recovery, snapshot, graph, http, checks
  planner.rs    -> contract, serde
  prompts.rs    -> prompty 2.1.0, serde, contract
  recovery.rs   -> db/store, graph, snapshot
  snapshot.rs   -> db, graph, contract, serde
  static_ui.rs  -> rust-embed, axum
  store.rs      -> rusqlite, uuid, contract, db, entities
  tools_bridge.rs -> hiverouter-tools (jail/approval/commands/health/settings), contract, db
  ws.rs         -> axum::extract::ws, tokio::sync::broadcast, contract schema/events, bus, graph, engine, tools bridge
```

## 2. External crates in use (from the two manifests, READ)

| Crate | Where | Feature notes |
|---|---|---|
| serde/serde_json | both | derive; `preserve_order` workspace pin |
| uuid 1.29 (`v7,serde`) | both | uuid-v7 ids; `schemars` feature `uuid1` for JsonSchema |
| thiserror 2.0.21 | both | refusal/display and config/DB error enums |
| schemars 1.2.3 / jsonschema 0.60.2 | contract | schema generation + payload validation |
| tokio 1.54.1, axum 0.8.9, tower-http 0.7.1 | core | transport/HTTP; axum `ws` feature (no direct tokio-tungstenite) |
| rusqlite 0.41.1 (`bundled`) | core | SQLite; C toolchain needed only at build |
| petgraph 0.8.3 | core | graph authority + toposort |
| prompty 2.1.0 | core | `.prompty` loader (render implemented locally) |
| async-trait 0.1.89, tokio-util 0.7.20, uuid, tracing 0.1.44 | core | engine traits, CancellationToken, seq ids, logs |
| rust-embed 8.13.0 | core | embedded `web/dist` UI |
| hiverouter-tools (path) | core | jailed files, approvals, settings, `/diagnose` |
| tempfile 3.23 (dev) | both | test fixtures only |

## 3. Cross-partition links

- **contract → providers**: `hiverouter-provider` and `hiverouter-tools` consume `hiverouter-contract` (events, refusals,
  command payloads); the reverse edge does not exist (single-source vocabulary).
- **core → tools**: `Cargo.toml:21` path dep; wired in `src/tools_bridge.rs` and used by `src/ws.rs:398-431` and
  `src/http.rs:255` `/diagnose`. This is the D-2/D-3 integration seam.
- **core ⇸ provider (MISSING)**: core does **not** depend on `hiverouter-provider`; `src/http.rs:77/117/154` construct
  `RunEngine` with `UnconfiguredExecutor`, so no live model calls can originate from the shipped server (FINDINGS F-01).
- **tools → provider**: tools' settings/health import provider model/id types (cross-partition, outside this seat).

## 4. Runtime call chains (with anchors)

1. **Boot** `main.rs:54` Database::open → `:64` migrate → `:79` SnapshotStore → `:81` recover_on_boot →
   `:117` AppState::with_graph → `:126` http::serve. Failure paths exit non-zero with a refusal code.
2. **HTTP** `http.rs:224` build_router → `/health` (checks+db+transport), `/diagnose`
   (→`tools_bridge.diagnostics`→tools health census), `/api/session` (token+Origin), `/api/history/*`
   (→`history.rs`→snapshot files), `/ws` (`:212`→`ws.rs` handler), fallback `static_ui`.
3. **WS session** pre-upgrade Origin/token (`ws.rs`) → `session_loop` subscribe bus + hello replay/resync →
   `handle_client_text`: rewire→`graph.rs`; SUBMIT/SPAWN/CANCEL/KILL→`execution.rs`; settings/file/approval→
   `tools_bridge.rs`; results published via `bus.rs` fan-out with `correlation_id`.
4. **Run** `execution.rs submit_prompt` → run row+receipt (`store.rs`) → spawn dispatch → one `executor.plan` → plan
   validate (`planner.rs`) → caps (`caps.rs`) → per-node `store.insert_task` + `graph.add_task` → leaf `executor.execute`
   → `REPLY_DELTA`/`REPLY_DONE` (`bus.rs`) → assistant message → terminal status → optional snapshot
   (`snapshot.rs` → db index).
5. **Recovery** `recovery.rs`: non-terminal runs → `Interrupted` (`store.rs`/`db.rs`); snapshots on disk →
   `graph.restore_from_snapshot`; corrupt → report + leave file.
6. **Test callers** integration tests import public modules (`tests/support/mod.rs:13-18`), drive real loopback sockets
   (`ws_transport.rs`), real SQLite (`chat_store.rs`), real subprocess kill (`kill_restore.rs`).
