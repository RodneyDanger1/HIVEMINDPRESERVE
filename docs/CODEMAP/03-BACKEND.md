# 03 — Backend

## Workspace

`backend/Cargo.toml` is a workspace of four crates:

| Crate | Role |
|---|---|
| `hiverouter-contract` | Frozen wire types: 17 command names, 23 event names, refusal table, schemas, state/usage/timestamp. No I/O. |
| `hiverouter-provider` | DeepSeek HTTP/SSE client, DTOs, streaming parser, per-run budget, tool-call assembler. |
| `hiverouter-tools` | Tool registry/tiers/schemas, workspace jail, read executor, approval gate, side-effect executor, shell policy, census. |
| `hiverouter-core` | Boot/server/WS/engine/planner/task loop/recovery/history/store/bus. |

Dependency direction: `core → tools/provider/contract`; `tools → contract`; `provider → contract`; `contract` depends on no workspace crate. There are 112 tracked backend files: 102 Rust (14 contract, 16 provider, 24 tools, 48 core) plus 5 TOML, 3 JSON, 1 prompt, 1 lock. The earlier 13/16/23/46 table omitted four tracked `.rs` files (`contract/tests/common/mod.rs`, `core/tests/support/mod.rs`, `core/build.rs`, `tools/benches/engine_benches.rs`); they are in the file catalog.

## Contract crate (`backend/crates/hiverouter-contract`)

- `event.rs` — all event payloads, including `TOOL_CALL`/`TOOL_RESULT`, additive NODE `rev/kind/parent_task_id`, `APPROVAL_REQUEST.args_preview`, bounded redaction builder, 512 UTF-16 cap.
- `command.rs` — client commands, validation, `APPROVAL_RESOLVE`, rewire `expected_rev`, template insert, decision-gated diff commands, client pull note.
- `refusal.rs` — stable `E_*` codes, retryability, how_to_fix.
- `schema.rs` — generated payload schemas and envelope.
- `state.rs`, `usage.rs`, `timestamp.rs`, `run_id.rs`-style ids — shared primitives; unknown stays absent.
- Tests: golden schema, schema roundtrip, UI command matrix, refusal matrix, common fixture.

## Provider crate (`backend/crates/hiverouter-provider`)

- `client.rs` — DeepSeek HTTP/SSE client, auth, retries/budget preflight, cancellation.
- `dto.rs` — request/response/usage/tool-call DTOs.
- `stream.rs` / `budget.rs` — SSE parser and per-run `RunBudget` (`CALL_CAP=40`), `E_CALL_LIMIT`.
- `tool_assembler.rs` — streamed tool-call fragments assembled deterministically.
- Tests cover malformed SSE, budget refusal, usage completeness, streaming order.

## Tools crate (`backend/crates/hiverouter-tools`)

- `tool.rs` — 7 tool ids, 4 tiers, 4 permission classes, allowlists, JSON schemas.
- `executor.rs` — read-class execution only (`file_read`, `file_list`, `report_result`, `finish`).
- `jail.rs` — path normalization/escape prevention, 1 MiB read cap, atomic write.
- `approval.rs` — approval table/indexes, nonce + args digest, TTL, CAS resolve/expire/cancel/label/count, ledger.
- `side_effects.rs` — preflight → grant → one commit; file_write atomic/replace; shell_exec allowlist/argv/NUL/cwd bounds, Job Object.
- `shell.rs` — Windows process containment and allowlisted program policy.
- `census.rs` — derived tool/refusal/tier counts + executor-lane completeness check.
- `health.rs`, `commands.rs`, `watcher.rs` — boot checks, file tree/read commands, filesystem watcher.
- Tests: executor safety, jail escape, approval races, side-effect exactly-once, shell refusal.

## Core crate (`backend/crates/hiverouter-core`)

- `main.rs` / `http.rs` — boot, config, DB migrations, bus, graph, tools bridge, engine injection, Axum routes, static UI, recovery catch-up.
- `execution.rs` — planner dispatch, task DAG, provider streaming, tool loop, approval suspension/resume/denial/timeout/cancel, side-effect call, snapshots, spawn queue, terminal reconciliation, caps.
- `tools_bridge.rs` — approval request projection, `verify_tool_grant`, `execute_approved`, ledgered resolve, pending replay.
- `approval_wait.rs` — one-shot waiter map, duplicate-arm refusal.
- `provider_executor.rs` — planning/leaf/continuation requests, tool specs by task tier, reasoning passback.
- `ws.rs` — WS session, token/origin checks, command dispatch, event fan-out, hello replay, approval wake, template insert, rewire/context.
- `db.rs` — SQLite open/migrations v4, connection policy; `store.rs` — run/task/task-state guard/messages/tool-turns/receipts.
- `recovery.rs` — boot crash recovery: interrupted runs, task settlement, graph repair, snapshots.
- `history.rs`/`snapshot.rs` — revisions/graph snapshots; `bus.rs` — sequenced event bus/replay; `graph.rs` — DAG authority/rev; `run_state.rs` — state machine; `caps.rs` — depth/fan-out/agent/call limits; `prompts.rs` — planner prompt assets; `context.rs` — scoped persona context; `checks.rs`/`config.rs` — boot checks/config.
- Tests: run lifecycle, suspension, tool-loop matrix, approval ledger, WS transport, recovery gates, kill/restore, graph, snapshot, HTTP boot, context switch.

## Refusal and cap model

- Commands validate before dispatch; malformed payloads get typed schema errors.
- Tools fail closed: unknown tier/args never reach jail/side effects.
- Caps are hard, named refusals: `E_SPAWN_DEPTH`, `E_FANOUT_LIMIT`, `E_AGENT_LIMIT`, `E_CALL_LIMIT`, `E_BUDGET_EXCEEDED`, `E_TOOL_NOT_ALLOWED`.
- Crash recovery is forward-only; no fabricated historical success.

## Known backend gaps (verified or reported)

- Chat messages are persisted but no HTTP route exposes them; web hydration reports `messages-unavailable`.
- Only `planner` prompt/DAG asset exists; four Library templates still refuse after real lookup.
- No Persona registry loader; scoped `allowed_tool_ids` is the available carrier.
- `TaskRecord` does not carry the authoritative tier, so NODE `kind` is intentionally omitted.
- Tiny spawn-vs-terminal-drain race documented in engine comments.
