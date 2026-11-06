# PHASE 4 EVIDENCE — Scheduling, Delegation & Run Lifecycle

Date: 2026-11-06 (America/Chicago). Executor: B-Core. Recon: `PHASE-4-RECON.md`.
Commands from `backend/` with the Phase 1 PATH/env (see PHASE-1-EVIDENCE preamble).

## 1. Preflight + full suite

`cargo test -p hiverouter-core` (post disk-wipe rebuild, Phase 4 modules compiled):

```
running 34 tests (lib)                                   test result: ok. 34 passed; 0 failed
Running tests\chat_store.rs                              running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.86s
Running tests\run_lifecycle.rs                           running 6 tests
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.89s
Running tests\ws_transport.rs                            running 9 tests
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.06s
CORE_TEST_EXIT=0
```

## 2. Acceptance tests (blueprint Phase 4 card)

`tests/run_lifecycle.rs` (6 tests, all ok):

| Card item | Test | Behaviour asserted |
|---|---|---|
| `cargo test run_lifecycle_plans_and_dispatches` | `run_lifecycle_plans_and_dispatches` | prompt -> run row -> one planning call -> root + 2 planned tasks -> `NODE_SPAWN` x3 -> streamed `REPLY_DELTA` chunks -> `REPLY_DONE` per leaf -> run row `completed`; chat store holds user + 2 assistant messages |
| `cargo test caps_refused_by_name` | `caps_refused_by_name` | a depth-4 plan fails with `E_SPAWN_DEPTH`; a 9-sibling plan fails with `E_FANOUT_LIMIT`; run row + `RUN_STATUS(failed, error_code)` carry the named code |
| error-path provider 429 mid-plan | `provider_error_preserves_partial_state` | second leaf returns `E_LLM_RATE_LIMIT`; run row `failed` with `error_code=E_LLM_RATE_LIMIT`; root + both task rows preserved, first leaf `Completed`, second `Failed`, completed assistant message kept — no fake success |
| cancellation | `cancel_run_stops_the_run` | `CANCEL_RUN` cancels the run token, run row + `RUN_STATUS(cancelled)`, second cancel is refused with `E_RUN_TERMINAL` |
| single active run | `second_prompt_while_active_is_refused_by_name` | second `SUBMIT_PROMPT` while a run is active is refused with `E_RUN_ACTIVE` |
| unknown parent | `spawn_sub_hive_unknown_parent_is_refused` | `SPAWN_SUB_HIVE` for a missing parent -> `E_TASK_NOT_FOUND` |

`tests/chat_store.rs` (5 tests, all ok): user/assistant message persistence + ordering, durable re-open, run status
transitions + active-run tracking, task rows with depth/fan-out fields, 24 h command-receipt idempotency, migration
v3 -> v2 -> v0 -> v3 down/up round-trip.

`tests/ws_transport.rs` (9 tests, all ok): the Phase 4 engine now handles `SUBMIT_PROMPT`/`SPAWN_SUB_HIVE`/`CANCEL_RUN`/
`KILL_WORKER` over the socket; `run_commands_are_handled_by_the_phase4_engine` asserts `CANCEL_RUN` for a missing run
returns `E_RUN_NOT_FOUND` and that Phase 6 commands still produce a typed `E_INTERNAL_ERROR` (never a silent drop).

## 3. Implementation facts (read back from the code)

- `src/db.rs` migration v2 adds `runs`, `tasks`, `chat_messages`, `command_receipts` (FK cascade, indexes; tested inverse).
- `src/entities.rs` + `src/store.rs`: `Run`, `TaskRecord`, `ChatMessage`; typed store with `create_run`, status updates,
  active-run query, task insert/state transitions, message append/list and 24 h idempotency receipts.
- `src/caps.rs`: `Caps { max_depth: 3, max_fanout: 8, max_agents: 64, max_calls: 40 }`; checks return
  `E_SPAWN_DEPTH` / `E_FANOUT_LIMIT` / `E_AGENT_LIMIT` / `E_CALL_LIMIT` before any row is written.
- `src/planner.rs`: plan DAG validation (non-empty, <= 40 nodes, unique ids, known deps, acyclic); cycle refused at plan
  validation with `E_SCHEMA_INVALID`; leaf-order execution.
- `src/execution.rs`: `ProviderExecutor` seam (`plan`/`execute`), `RunRegistry` (one `tokio_util::sync::CancellationToken`
  per run), `RunEngine::submit_prompt/spawn_sub_hive/cancel_run/kill_worker`, background dispatch that streams
  `REPLY_DELTA` then one `REPLY_DONE(usage)` per leaf, persists chat messages, preserves partial state on failure, and
  writes a versioned snapshot after every terminal run state (Phase 9 store).
- Production default executor is `UnconfiguredExecutor`: with no provider configured it refuses with `E_KEY_MISSING`
  instead of fabricating a reply (tests use a mock executor defined only inside the test files, which the blueprint
  explicitly permits: "Phase 4 spawn tests use a tool-less mock executor").

## 4. Deviations recorded (not silently resolved)

1. Token/cost budgets are Phase 5's; this phase consumes `Usage` but hardcodes no prices and does not settle cost.
2. `E_AGENT_LIMIT` comes from blueprint §2a (budget invariants), not UI-COMMANDS §5; it is already present in the frozen
   refusal table as a recorded superset and is used by the cap gate.
3. Phase 6 commands (`APPROVAL_RESOLVE`, `SETTINGS_STATUS`, `FILE_*`) are decoded but still refused with a typed
   `E_INTERNAL_ERROR` until the tools crate wiring lands (documented, retryable, never dropped).
4. Plan nodes are parented to the run root in the graph; dependency ordering from the plan is enforced by the engine's
   leaf execution order (graph edges model spawn topology, not dependency semantics).

## 5. Done / Not done / Unverified

| Done | Evidence |
|---|---|
| run/task entities + SQLite store (migration v2) + chat message store | §1, §2 chat_store |
| plan DAG validation + one provider call per plan | §2 |
| hard caps depth 3 / fan-out 8 / agents 64 / calls 40 refused by name | §2 caps test |
| cancellation token per run + `CANCEL_RUN` / `KILL_WORKER` handlers | §2 |
| streamed reply interfaces (`REPLY_DELTA`/`REPLY_DONE`) with real usage, no fake success | §2 |
| WS dispatch for the four run-lifecycle commands | §1 ws_transport |

| Not done (reason) | |
|---|---|
| live provider end-to-end | no key/provider in this phase; provider seam lands in Phase 5 (B-Provider) |
| approval/file/settings command execution | Phase 6 tools crate wiring |

| Unverified | What would prove it |
|---|---|
| real DeepSeek plan/reply against the provider | Phase 5 live-call integration test with a key (not available) |
