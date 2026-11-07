# 02 — Architecture

## Process and planes

HiveMind is a local-first single-binary Rust server plus a React client.

```
web/src (React 19 + Vite)
  └─ WsApiClient  ── WebSocket /ws + HTTP /api/* ──┐
                                                    ▼
backend/crates/hiverouter-core/src/main.rs
  └─ http.rs boot: Config → Database → EventBus → GraphAuthority → ToolsBridge
      → RunEngine → Axum router (static UI + API + WS)
          ├─ planner (provider_executor.rs → DeepSeek)
          ├─ task execution + tool loop (execution.rs)
          ├─ approval gate (tools/approval.rs + approval_wait.rs + tools_bridge.rs)
          ├─ side effects (tools/side_effects.rs, jail.rs, shell.rs)
          ├─ SQLite store (db.rs, store.rs) + snapshots
          └─ WS command dispatch (ws.rs) + event fan-out (bus.rs)
```

## The run path

1. Client sends `SUBMIT_PROMPT`; `ws.rs` dispatches to `RunEngine::submit_prompt`.
2. Engine creates a run row (`queued`), user message, active-run slot, and cancellation token.
3. `dispatch_inner` sets `planning`, calls the provider planner, validates the DAG, inserts root + plan tasks, publishes `NODE_SPAWN` frames, transitions `planning → running`.
4. Leaf tasks execute in topological order. Each task calls the provider; streamed deltas become `REPLY_DELTA`.
5. A tool call enters `tool_loop`: plan/validate → budget → tier/persona authorization → read-only executor or approval suspension.
6. Read-only tools execute through `WorkspaceJail`; results become `TOOL_RESULT` and are fed back via `continue_with_tools`.
7. Side-effecting tools request approval (`APPROVAL_REQUEST`), suspend the task as `awaiting_approval`, and wait on `ApprovalWaiters`.
8. `APPROVAL_RESOLVE` consumes the nonce, commits the decision, appends a ledger row, wakes the waiter. Approve → preflight → grant CAS → one effect; deny → denied result; timeout/cancel → terminal path with no effect.
9. Each tool turn persists a `tool_turns` row and increments run counters. Caps: turns/calls/waits, provider `RunBudget`, spawn depth/fan-out/agents.
10. Completion reconciles root/non-leaf tasks, writes a terminal snapshot, publishes one `RUN_STATUS`, then `REPLY_DONE`.

## Invariants

- No fake success: every failure is a typed `E_*` refusal with message/how_to_fix/retryable.
- Single-use approval: nonce + `args_sha256` + `decision IS NULL` CAS; unique pending index.
- At-most-once side effect: preflight before grant; grant consumed exactly once; no retry after commit failure.
- Terminal-state guard: a killed/failed/completed task cannot be overwritten by a late completion.
- One terminal `RUN_STATUS` owner; terminal owner guard in dispatch.
- Unknown is not zero: usage/bytes/limits stay absent when unmeasured.
- Replay is bounded and idempotent; approval replay is server-push after hello, no auto-approve.

## Key source anchors

- `backend/crates/hiverouter-core/src/main.rs` — binary entry.
- `http.rs` — boot/constructors, static UI, routes, recovery catch-up.
- `ws.rs` — command handler, event fan-out, hello replay, template insert, rewire.
- `execution.rs` — engine, suspension, side effects, spawn queue, snapshots, terminal reconciliation.
- `tools_bridge.rs` — approval event projection/grant/commit/ledger.
- `provider_executor.rs` — DeepSeek client, streaming, budget, tool specs.
- `db.rs` / `store.rs` — schema, migrations, run/task/message/tool-turn persistence.
- `web/src/state/store.ts` — client state machine and hydration/replay.
- `web/src/api/client.ts` / `contract.ts` — transport and typed contract.
- `e2e/playwright.config.ts` + `helpers/live-server.ts` — mocked/live test harness.
