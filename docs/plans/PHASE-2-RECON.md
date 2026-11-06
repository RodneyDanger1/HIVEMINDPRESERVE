# PHASE 2 RECON — Transport & Graph Authority

ForgeWright recon ledger (skill: `forgewright-recon`). Written BEFORE any Phase 2 edit.

## Recon

- **Target behavior today.** After Phase 1 the binary boots, serves `GET /health` and a 404 fallback, and owns a SQLite
  bootstrap (`backend/crates/hiverouter-core/src/{http.rs,db.rs}`). There is **no** WebSocket transport, no Origin/token
  gate, no session sequencing and no task graph. Phase 2 adds exactly one axum `/ws` endpoint, one server-scoped event
  sequence + fan-out, and one authoritative petgraph task graph that refuses cycles.
- **Entry point.** `backend/crates/hiverouter-core/src/http.rs` — `build_router()` is the single router assembly point
  (called by `http::serve` from `src/main.rs`). New modules `src/ws.rs` (transport) and `src/graph.rs` (graph authority)
  are mounted from there; `AppState` (in `http.rs`) grows the transport + graph handles.
- **Callers.** `src/main.rs` boots the router; later FigCraft/frontend consumes the wire protocol
  (`docs/contracts/UI-COMMANDS-v1.md` §1-§5); Phase 3 rides the same socket for the `HandshakeFrame`; Phase 4 dispatches
  runs through the same bus and graph. No caller exists outside this crate yet.
- **Tests + command.** From `backend/`: `cargo check` (phase preflight; the blueprint also lists `npm run typecheck`, but
  the web scaffold is owned by the frontend teammate and is not present yet — recorded as owned elsewhere, not skipped
  silently). Acceptance: `cargo test -p hiverouter-core ws_fanout_two_clients`, `cargo test -p hiverouter-core
  cycle_rejected` (edge count unchanged), `cargo test -p hiverouter-core origin_rejected`, `cargo test -p
  hiverouter-core hello_resync`, plus an error-path test `invalid_json_frame_keeps_connection_usable`.
- **Data flow (one path, one failure path).** Browser opens `ws://127.0.0.1:<port>/ws?token=<uuid-v7>&last_seq=<n>` →
  Origin header must be on the allowlist (`http://127.0.0.1:<port>`, `http://localhost:<port>`, plus
  `HIVEROUTER_ALLOWED_ORIGINS`) → token must equal the boot-time session token → session task starts → client sends
  `{"type":"hello","last_seq":n}` → server either replays buffered events after `n` (`SERVER_HELLO{last_seq}` first) or,
  when `n` is older than the retained ring buffer, sends `RESYNC_REQUIRED{expected_seq,last_seq}` then a
  `HISTORY_SNAPSHOT` of the current in-memory graph → live events fan out to every subscribed session through a
  `tokio::sync::broadcast` channel with a server-global monotonic `seq`.
  Failure path: a non-allowlisted Origin is refused with HTTP 403 + `E_ORIGIN_REJECTED` before the upgrade and the socket
  closes; invalid JSON after upgrade gets an `ERROR` frame `E_BAD_FRAME` and the connection stays usable (never a silent
  drop, never `_ => {}`).
- **Graph authority design (single owner).** `GraphAuthority` owns one `petgraph::DiGraph<TaskId, ()>` behind an
  `Arc<Mutex<..>>` in `AppState`; the transport/router never touches petgraph directly. Mutations: `add_task(parent, id)`,
  `rewire(task, new_parent, expected_rev)` (optimistic revision check), `set_state`. A rewire that would create a cycle is
  detected **before** insertion (`petgraph::algo::has_path_connecting(new_parent, task)` plus
  `is_cyclic_directed` as a defensive re-check), returns `Refusal::Cycle` (`E_CYCLE`) and leaves node/edge counts
  unchanged. Stored `toposort` order is recomputed after a successful rewire.
- **Unknowns / evidence still missing.**
  - axum 0.8.9 WS API details (`WebSocketUpgrade`, `Message::Text(Utf8Bytes)`) — UNVERIFIED until compile
    (FACT-REGISTER F2 API-diff risk).
  - No direct `tokio-tungstenite` dependency is allowed (FACT-REGISTER F2: use axum's `ws` feature and let it resolve),
    so the test client is a hand-rolled RFC6455 client under `tests/` only (masking + text frames); if that proves
    infeasible the fallback is to test the bus/session logic directly and record the transport gap honestly.
  - Token distribution: the web build does not exist yet, so `GET /api/session` returns the boot token only to a
    loopback peer whose Origin is absent or allowlisted (Phase 9/history read API may extend this). No second port, no
    network exposure.
  - Blueprint Phase 2 preflight includes `npm run typecheck`; web ownership is frontend's, so this phase quotes
    `cargo check` and records the npm half as an external dependency rather than faking it.
- **Files expected to touch (exclusive ownership).** `backend/crates/hiverouter-core/src/{http.rs,lib.rs,main.rs}` (edits),
  new `backend/crates/hiverouter-core/src/{ws.rs,graph.rs,bus.rs}`, new
  `backend/crates/hiverouter-core/tests/{ws_transport.rs,graph_authority.rs}`, `backend/Cargo.toml`/`backend/Cargo.lock`
  if a pinned dependency is deliberately added (petgraph 0.8.3 / tokio-util 0.7.20 are already pinned workspace-wide),
  `docs/plans/PHASE-2-{RECON,EVIDENCE}.md`, `docs/claims/*`. **Never:** provider/tools crates, `web/`,
  `docs/ui/figcraft`, the blueprint.
- **Rollback.** Additive modules + route mount; removing the `/ws` route and the two modules restores Phase 1 behaviour.
  The graph authority holds no persistent state in this phase, so rollback has no data migration.

**Recon completeness:** complete — entry point (`http.rs`), boot caller (`main.rs`) and every existing module this phase
edits have been read; no caller or test exists yet for the transport, so both are created by this phase.
