# HiveMind — Architecture (truth-first, 2026-11-10)

**What this is:** a local, single-provider, single-run hierarchical multi-agent
system (HMARS). A root HyperAgent plans through one provider call, has two
bounded pre-dispatch control turns, dispatches a DAG of SuperAgents → Agents →
Subagents, settles tasks, makes one **read-only** post-dispatch monitor review,
then produces a tool-less final answer. It is **not** AGI, a Gödel machine, a
self-modifying system, a continuous daemon, or a network-capable HMAS.

For the full per-file map see [`docs/CODEMAP/02-ARCHITECTURE.md`](docs/CODEMAP/02-ARCHITECTURE.md);
for the audit see [`docs/audit/HIVEMIND-TRUTH-AND-ROADMAP-2026-11-10.md`](docs/audit/HIVEMIND-TRUTH-AND-ROADMAP-2026-11-10.md).

## 1. Runtime planes

```text
web/src (React 19 + Vite 8 + Zustand + @xyflow/react)
   │  WS /ws + HTTP /api/*  (x-hivemind-token / Bearer)
   ▼
backend/crates/hiverouter-core/src/main.rs
   │  Config → SQLite Database → EventBus → GraphAuthority → ToolsBridge
   │  → RunEngine → Axum router (embedded SPA + API + WS)
   ├─ planner + provider adapter (provider_executor.rs)
   ├─ execution/tool loop + caps + terminal reconciliation (execution.rs)
   ├─ approval gate / waiters / grant commit (approval.rs, approval_wait.rs, tools_bridge.rs)
   ├─ workspace jail + side effects + shell policy (jail.rs, side_effects.rs, shell.rs)
   ├─ SQLite store + snapshots + memory banks (db.rs, store.rs, snapshot.rs)
   └─ WS command dispatch / event fan-out (ws.rs, bus.rs)
```

## 2. Run sequence (bounded)

```text
submit_prompt
  → planner provider call; validate ≤40 nodes, depth ≤3, DAG, caps
  → seed root HyperAgent task
  → root control turn 1: spawn/assign/remote/daemon/memory/proposal/finish
  → root control turn 2: feedback; creation refused on turn 2
  → dispatch ready tasks (deps + direct children terminal)
  → settle unfinished tasks (sequential repair loop, max 64)
  → one read-only monitor review (no tools; text appended as data)
  → tool-less final synthesis
  → persist usage/status/memory/snapshot; one terminal RUN_STATUS
```

Hard caps: depth 3, fan-out 8, 64 agents/run, 64 provider calls/run, 40 plan
nodes, 12 tool turns, 40 tool calls/run, 8 approval waits/run, 64 KiB tool
result, $0.50 and 400k combined tokens per run (`tool_loop.rs`). Root control is
≤4 calls/turn and 2 turns.

## 3. What is real

- Single-use approval with TTL and nonce; one per-run suspension; both queued
  approvals execute one write each (tested).
- Workspace jail; `file_read`/`file_list`; approval-gated `file_write` and
  `shell_exec`; shell allowlist is **git.exe + cargo.exe only**, no interpreters,
  **not an OS sandbox**.
- Approval-gated `web_fetch` with HTTPS-only/SSRF/no-redirect/1 MiB limits.
- SQLite memory banks with recency retrieval; scoped facts; journal hash chain.
- Deterministic detached admission; in-process remotes; enrolled daemons.
- Proposal-only improvement; five-seat authorization (4 SuperAgents + operator);
  **EVO-GATE requires a held-out receipt** at apply, and boot rejects receipt-less
  lesson lines. The gate validates shape, not an executed evaluation.
- Offline full battery, docs truth, crash/restart drill, 41-task scale probes,
  1440/375 screenshots, per-user installer, schema v24.

## 4. What is not real (do not claim)

- No vector DB, embeddings, ANN, semantic memory, or cross-hive vector search.
- No `HiveWire` implementation in this repository.
- No monitor actions (`send_instruction`, `cancel_task`, repair), no mid-flight
  root supervision.
- No automatic learning, weight updates, skill library, or autonomous code
  mutation; lessons are prompt data only.
- No network remotes/daemons; no live 40-node provider run; no live paid proof
  without a configured key.
- No clean-profile install proof, no 1.2 tag/release seal, no AGI/Gödel machine.

## 5. Current evidence identity

- Product commit `806251b`; installed `1.2.0-dev`, schema 24, health `degraded`
  with `E_KEY_MISSING`; package SHA-256 `B4BAC18D…11061`.
- Offline battery `docs/evidence/live/full-battery-20261110-232942.json`
  (PASS, dirty tree at `1129349`); install receipt
  `docs/evidence/live/install-1.2.0-dev.json`.
