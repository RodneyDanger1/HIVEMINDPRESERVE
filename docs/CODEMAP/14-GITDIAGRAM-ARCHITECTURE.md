# 14 — GitDiagram Architecture (External, Sampled)

## Provenance

- File: `C:\Users\Riley\Downloads\rodneydanger1_hivemind architecture diagram_ how it works.html`
- SHA-256: `0B857D60E8457FE06D15C783F4985728014BEF918CD181CC3CF9E5437C3FDC4B`
- Source URL embedded in page: `https://gitdiagram.com/rodneydanger1/hivemind`
- Generated: `2026-11-07T05:48:53.259Z` = 2026-11-06 23:48:53 -0600, **7m36s after local HEAD `aa16e13` was committed**. Likely the same tree, but the exact commit is not exposed; treat as a near-current external snapshot.
- Self-reported: **40 source files read · 25 components · 30 connections · 22/30 connections with evidence**.
- Quoted disclaimer:

> “Several backend implementation modules and UI panels are present in the tree but unsampled; their subsystems are represented only where README or source-index evidence supports them, and unverified links are omitted.”

The saved HTML contains no Mermaid source (0 `subgraph`/`flowchart`); the graph was recovered from rendered SVG nodes, anchors, `data-edge` paths, and edge labels. Recovery fidelity: **25/25 nodes, 30/30 edges, 30/30 labels, 4/4 clusters, 0 unparsed**.

## Components

| Group | Component | Diagram file hint / anchor | Current mapping |
|---|---|---|---|
| Workspace UI | App Shell | `App.tsx` | `web/src/App.tsx` |
| Workspace UI | UI State | `store.ts` | `web/src/state/store.ts` |
| Workspace UI | API Client | `client.ts` | `web/src/api/client.ts` |
| Workspace UI | Wire Contract | `contract.ts` | `web/src/api/contract.ts` + Rust contract |
| Workspace UI | Run Canvas | `Canvas.tsx` | `web/src/flow/Canvas.tsx` |
| Workspace UI | Chat Surface | `ChatCore.tsx` | `web/src/chat/ChatCore.tsx` |
| Workspace UI | Approval Review | `panels/ApprovalOverlay.tsx` (via anchor) | `web/src/panels/ApprovalOverlay.tsx` + `state/approval.ts` |
| Workspace UI | Routing View | `RoutingPanel.tsx` | `web/src/panels/RoutingPanel.tsx` |
| Workspace UI | History Replay | `Canvas.tsx` (duplicate anchor) | `Canvas.tsx` + `Scrubber.tsx` + `store.ts` |
| Local Runtime | Local Server | `main.rs` | boot only (`main.rs`); server is `http.rs` + `ws.rs` |
| Local Runtime | Command Handling | `execution.rs` | **mismatch:** wire dispatch is `ws.rs`; `execution.rs` is run engine |
| Local Runtime | Run Planning | `planner.rs` | `planner.rs` only plan DAG; planning call is `execution.rs` + `provider_executor.rs` |
| Local Runtime | Run Execution | no hint; anchor `provider_executor.rs` | actual engine `execution.rs`; diagram’s anchor is provider executor |
| Local Runtime | Event Contract | `event.rs` | `contract/src/event.rs` |
| Local Runtime | History Service | `history.rs` | `core/src/history.rs` |
| Local Runtime | Runtime Store | `db.rs` | `db.rs` bootstrap; queries `store.rs`, snapshots `snapshot.rs` |
| Model Provider | Provider Client | `client.rs` | `provider/src/client.rs` |
| Model Provider | Usage Budgets | `budget.rs` | `provider/src/budget.rs` |
| Model Provider | Usage Accounting | `accounting.rs` | `provider/src/accounting.rs` |
| Model Provider | Provider DTOs | `dto.rs` | `provider/src/dto.rs` |
| Actors | DeepSeek API | external | remote API |
| Tool Safety | Tool Engine | `tool.rs` | catalogue only; `tool_loop.rs`, `executor.rs`, `side_effects.rs` execute |
| Tool Safety | Tool Approval | `approval.rs` | `tools/approval.rs` + `core/tools_bridge.rs` |
| Actors | Operator | — | human |
| Actors | Tool Operator | — | human |

All **21 distinct hinted paths** (22 anchors; `Canvas.tsx` linked twice) exist verbatim at current HEAD. The problem is role labelling, not path rot.

## Connections

30 edges with these labels (some repeated):

`uses`, `creates`, `initializes`, `renders` (4×), `reads and updates`, `projects state`, `reads tasks`, `renders replay`, `sends commands`, `validates frames`, `dispatches`, `plans runs`, `serves replay`, `reads history`, `emits frames`, `requests completions`, `streams responses`, `records attempts`, `accounts usage`, `uses payloads`, `runs tools`, `requests approval`, `reviews requests`, `resolves decisions`, `requests snapshots`, `projects snapshots`.

9 edges are dotted, but line style is **not** an evidence flag: 22/30 have evidence, 8 do not, and 9 dotted edges do not map 1:1 to that count.

## Structural findings

| Finding | Evidence |
|---|---|
| `Run Execution` has **in-degree 0** | Nothing in the diagram points to it; the plan→execute edge is absent. |
| `Run Planning` has **out-degree 0** | No plan→execute connection; the central handoff is missing. |
| 8 dead-end nodes | Several subsystems have no outgoing path. |
| `Approval Review` has **no inbound server edge** | Approval review is drawn but not connected from the server path. |
| `Event Contract` is a dead end | It feeds the UI but has no outgoing edge in the sampled graph. |
| `Tool Safety` contains **no safety code** | `jail.rs`, `shell.rs`, `side_effects.rs` are unsampled. |
| Only provider subgraph is strongly corroborated | `client.rs` imports `budget`/`dto`; `budget.rs` imports `accounting`; 22/23/24 are real. |

## Omissions

The diagram references **21 of 111 tracked source files** (90 omitted): contract 1/10, core 6/26, provider 4/9, tools 2/21, web 8/45. Missing include `ws.rs`, `http.rs`, `bus.rs`, `run_state.rs`, `tools_bridge.rs`, `command.rs`, `refusal.rs`, `jail.rs`, `shell.rs`, `side_effects.rs`, `recovery.rs`, `snapshot.rs`, `graph.rs`, `caps.rs`, `context.rs`, and 14 of 16 web panels. No tests are represented.

## How to cite it

Use it as a near-current external topology cross-check and as a source of component/edge vocabulary. Do not use it as proof that a module exists at a path, proof of a connection, or a current count. Its own page says unverified links are omitted.

## Corrected current architecture in one line

`React UI → WsApiClient → WS/HTTP (ws.rs/http.rs) → RunEngine (execution.rs) → planner/provider (execution.rs/provider_executor.rs/client.rs) → tool loop/approval (tool_loop.rs/tools_bridge.rs/approval.rs/approval_wait.rs) → side effects (side_effects.rs/jail.rs/shell.rs) → SQLite store/snapshots (db.rs/store.rs/snapshot.rs) → bus (bus.rs) → event frames → store projection → UI panels.`
