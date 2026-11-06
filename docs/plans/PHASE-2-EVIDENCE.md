# PHASE 2 EVIDENCE — Transport & Graph Authority

Date: 2026-11-06 (America/Chicago). Executor: B-Core. Recon: `PHASE-2-RECON.md`.
Commands run from `C:\Users\Riley\Desktop\hiverouter-v1\backend` with the Phase 1 PATH and env described in
`PHASE-1-EVIDENCE.md` §preamble.

## 1. Preflight

Blueprint preflight: `cargo check && npm run typecheck`.
- `cargo check -p hiverouter-core` → `Finished dev profile ... in 10.33s`, `CORE_CHECK_EXIT=0` (quoted in
  PHASE-1-EVIDENCE §5).
- Workspace-wide `cargo check` still fails on the external `hiverouter-tools` crate (see PHASE-1-EVIDENCE §8); this is
  recorded, not faked.
- `npm run typecheck`: the `web/` scaffold is owned by the frontend teammate and was not present/typed at this gate;
  recorded as an external dependency of this phase (not skipped as "passing").

## 2. Acceptance tests (blueprint Phase 2 card)

Full transport + graph suite, command `cargo test -p hiverouter-core`:

```
running 9 tests        (tests/ws_transport.rs)
test hello_replay_and_gap_resync ... ok
test invalid_json_frame_keeps_connection_usable ... ok
test rewire_ack_and_node_event_fan_out_to_both_clients ... ok
test rewire_cycle_is_refused_over_ws_and_graph_is_unchanged ... ok
test thousand_rewires_without_a_dropped_ack ... ok
test origin_rejected_when_not_allowlisted ... ok
test token_invalid_is_refused ... ok
test unimplemented_run_command_is_an_error_not_a_silent_drop ... ok
test ws_fanout_two_clients ... ok
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.43s

running 7 tests        (tests/graph_authority.rs)
test cycle_rejected ... ok
test rewire_in_flight_is_refused_then_cleared ... ok
test rewire_to_self_is_a_cycle ... ok
test single_owner_rejects_duplicate_task_ids ... ok
test snapshot_exposes_toposort_and_states ... ok
test stale_expected_revision_requires_resync ... ok
test unknown_task_is_refused_by_name ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.21s
```

Named-command runs (acceptance command names from the card):

```
$ cargo test -p hiverouter-core ws_fanout_two_clients
test ws_fanout_two_clients ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 2.58s

$ cargo test -p hiverouter-core cycle_rejected
test cycle_rejected ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out

$ cargo test -p hiverouter-core origin_rejected
test origin_rejected_when_not_allowlisted ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 7.36s
```

Full core suite result (lib 19 + graph 7 + http_boot 5 + ws 9 = 40): `CORE_TEST_EXIT=0` (PHASE-1-EVIDENCE §5).

## 3. Exit-gate mapping (blueprint Phase 2)

| Exit-gate item | Evidence |
|---|---|
| one client, 1,000 drags/rewires without a dropped ACK | `thousand_rewires_without_a_dropped_ack`: 1,000 `REWIRE_TASK` commands, each matched to its ACK by `correlation_id`; passed (tested in the same suite run) |
| a second client receives server-initiated diff frames in the same test | `ws_fanout_two_clients` (same event + same `seq` on both clients) and `rewire_ack_and_node_event_fan_out_to_both_clients` |
| cycle insertion returns `E_CYCLE` and leaves graph unchanged | `cycle_rejected` (unit graph API) + `rewire_cycle_is_refused_over_ws_and_graph_is_unchanged` (edge count + parent unchanged) |
| unknown frame type returns a typed error, never `_ => {}` | `invalid_json_frame_keeps_connection_usable` (`E_BAD_FRAME`, connection stays usable and ACKs the next command); `unimplemented_run_command_is_an_error_not_a_silent_drop` |
| non-allowlisted Origin returns `E_ORIGIN_REJECTED` and closes | `origin_rejected_when_not_allowlisted` (HTTP 403 body carries the code, both wrong Origin and missing Origin); `token_invalid_is_refused` (HTTP 401 `E_TOKEN_INVALID`) |
| reconnect resynchronises via `hello{last_seq}` | `hello_replay_and_gap_resync`: replay of buffered events 2,3 after `last_seq=1`; `RESYNC_REQUIRED` when the gap exceeds the 256-event replay buffer |

## 4. Implementation facts (read back from the code)

- `src/ws.rs`: one `/ws` route; Origin allowlist + token checked **before** upgrade (403/401 with named refusal JSON);
  `hello` timeout 10 s; server-global monotonic `seq`; replay buffer 256; broadcast fan-out capacity 1,024; PING→PONG;
  invalid JSON → `ERROR E_BAD_FRAME` without closing.
- `src/bus.rs`: `EventBus::publish/publish_with_correlation` assign seq starting at 1; `replay_after()` returns
  `Fresh | Current | Events | Gap`.
- `src/graph.rs`: single owner `GraphAuthority` (Mutex-protected petgraph `DiGraph`); rewire checks
  `has_path_connecting(task -> new_parent)` + defensive `is_cyclic_directed` re-check; cycle refusal is atomic
  (edges/parents/revision unchanged); `toposort` stored in the snapshot; in-flight rewire marker gives
  `E_REWIRE_IN_FLIGHT`; stale `expected_rev` gives `E_RESYNC_REQUIRED`.
- `src/http.rs`: `GET /health` now also reports transport counters (`last_seq`, `graph_nodes`, `allowed_origins`);
  `GET /api/session` returns the token to loopback/no-Origin or allowlisted-Origin callers only; server is
  `into_make_service_with_connect_info::<SocketAddr>` so the peer address is checked loopback.
- Boot token: generated with uuid-v7 at boot, held in memory, never logged and never written to disk.

## 5. Deviations recorded (not silently resolved)

1. **Node drag** has no frozen command in `UI-COMMANDS-v1.md` §2 (client-side canvas interaction); the two-client drag
   fan-out test therefore publishes the `NODE_DRAG` server event through the bus — the transport path under test is the
   same server→client fan-out, and the frozen vocabulary is not extended by this phase.
2. **Run-lifecycle commands** (`SUBMIT_PROMPT`, `SPAWN_SUB_HIVE`, `CANCEL_RUN`, `KILL_WORKER`, `TEMPLATE_INSERT`,
   `HISTORY_REVISION_SET`, `APPROVAL_RESOLVE`, `SETTINGS_STATUS`, `FILE_*`) are decoded by the transport but have no
   Phase-2 handler; they are refused with `E_INTERNAL_ERROR` (retryable) and logged, never silently dropped
   (`unimplemented_run_command_is_an_error_not_a_silent_drop`). Phase 4 replaces this interim refusal with real handlers.
3. `E_UNKNOWN_COMMAND` in the blueprint Phase 2 acceptance text is emitted as the frozen `E_COMMAND_UNKNOWN`
   (contract §5 is authoritative; recorded in PHASE-1-RECON unknowns).
4. Full `RESYNC` history snapshot delivery depends on the Phase 9 versioned snapshot store; Phase 2 sends
   `RESYNC_REQUIRED` + `SERVER_HELLO` and resumes the live stream (no fake `run_id` was invented to fabricate a
   `HISTORY_SNAPSHOT`).

## 6. Done / Not done / Unverified

| Done | Evidence |
|---|---|
| axum localhost WS transport, Origin allowlist + per-session token, hello replay/resync, typed ERROR, fan-out | §2, 9 tests |
| petgraph graph authority: single owner, cycle refusal atomic, optimistic rev, in-flight refusal, toposort snapshot | §2, 7 tests |
| two-client diffusion + 1,000 ACKed rewires | §2 |
| `/api/session` token distribution restricted to loopback/allowlisted Origin | §4 |

| Not done (reason) | |
|---|---|
| `npm run typecheck` | web scaffold owned by frontend teammate |
| workspace-wide `cargo check` | blocked by external `hiverouter-tools` compile error (B-Provider in progress) |

| Unverified | What would prove it |
|---|---|
| browser-origin behaviour against the real Vite dev server | frontend integration run once `web/` exists |
| TLS provider path | B-Provider Phase 5 gate |
