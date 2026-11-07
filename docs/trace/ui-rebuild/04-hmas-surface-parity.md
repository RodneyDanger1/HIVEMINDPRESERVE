# 04 — HMAS surface parity: design → current UI → real API → rebuild seams

> **Status:** read-only analysis. No product source was edited. This file is the only artifact
> written by this seat.
>
> **Snapshot:** 2026-11-07 ~15:16 America/Chicago, repo HEAD `e39cf3b`, working tree with
> concurrent UI edits (see §0.3). All `path:line` references are anchors in that snapshot;
> the working tree was still moving while this report was written. Re-read the cited
> function before editing, and treat the hash table in §0.3 as the evidence of record.
>
> **Scope:** map every requested HiveMind/HMAS capability to (a) the current UI render,
> (b) the real API/command path, (c) the authoritative FigCraft surface, (d) the missing
> representation, and (e) the exact fix seam — then flag dead/lying controls, hidden
> capabilities, and surfaces that need a new FigCraft board.

---

## 0. Sources, snapshot, and method

### 0.1 Authoritative design inputs

The task names `docs/ui/figcraft/ASSETS/hivemind-hmas.png`. **That exact file does not
exist in this checkout.** The HMAS design is split across two boards, both present:

| Board | File | Size | Role |
|---|---|---|---|
| 08 · HMAS Hierarchy | `docs/ui/figcraft/ASSETS/hivemind-hmas-hierarchy.png` | 1440×900 | tree + live topology + state legend (`README.md:33`) |
| 09 · HMAS Governance | `docs/ui/figcraft/ASSETS/hivemind-hmas-governance.png` | 1440×900 | profiles/import, memory, mailbox, projects, usage, approval (`README.md:34`) |
| 10 · States & Refusals | `docs/ui/figcraft/ASSETS/hivemind-states-and-refusals.png` | 1440×900 | named refusal, reconnect, empty/loading, kill confirm, replay, diff gate (`README.md:35`) |
| 07 · Spatial Canvas | `docs/ui/figcraft/ASSETS/hivemind-spatial-canvas.png` | 1440×900 | canvas/chat/rail/library/HUD/minimap composition (`README.md:32`) |
| 06 · Controls & HUD | `docs/ui/figcraft/ASSETS/hivemind-controls-and-hud.png` | 1440×1040 | composer, controls, tabs, HUD, scrubber, toast (`README.md:31`) |

The design README also records the authoritative board-07 geometry: 52px topbar, 72px rail,
240px library, canvas x=312, chat 420×500 at 620,196, HUD 460×44, minimap 220×150,
SuperAgent 420×150, WorkerAgent 320×96, ToolAgent 420×240, crimson error card
(`README.md:44-48`). Boards 08/09/10 are explicitly **PROPOSED-ADDITION** surfaces layered on
the authoritative style system (`README.md:44-45`); they are the parity target for HMAS, not
the original PDF.

Board 10 is included because the requested capabilities include failures/refusals and
replay/history, and the README lists it as part of the same regenerated set. Its SHA-256
changed during this analysis (another agent was regenerating it); the content described
below is the 15:03 version and is cited for representation, not pixel fidelity.

### 0.2 What “parity” means here

This report distinguishes four different failures that look alike in a screenshot:

1. **Missing representation** — the backend has the fact, the UI never renders it.
2. **Hidden capability** — a real endpoint/event/ledger exists but no UI path reaches it.
3. **Dead/lying control** — a control that does nothing, or renders a value that is not the
   thing its label claims, or promises an interaction that has no handler.
4. **Contract/producer gap** — the design requires a fact the server does not publish at all.
   This is not fixable by a UI change without a backend/contract addition; it must be shown
   as “not reported” or raised as an additive field, never fabricated.

The rebuild must not collapse (4) into (1). That is the “no fake controller” boundary in §14.

### 0.3 Evidence snapshot

HEAD is `e39cf3b` (`docs: record approval-gated cross-branch messaging`). The following files
were modified in the working tree during the analysis window, which is why some line anchors
may shift:

`web/src/App.tsx`, `web/src/chat/ChatCore.tsx`, `web/src/flow/BeeEdge.tsx`,
`web/src/flow/Canvas.tsx`, `web/src/flow/graph.ts`, `web/src/flow/nodes.tsx`,
`web/src/panels/Header.tsx`, `web/src/panels/LibraryPanel.tsx`, `web/src/panels/NavRail.tsx`,
`web/src/state/strings.ts`, `web/src/tokens/tokens.ts`, `docs/ui/figcraft/ASSETS-INDEX.json`.

Hashes at the analysis snapshot (re-checked at 15:16 for the two files that moved):

| File | Modified | Hash |
|---|---|---|
| `web/src/state/store.ts` | 10:21 | `27A2E6D4CA06` |
| `web/src/panels/AgentsPanel.tsx` | 12:07 | `FFC44145B443` |
| `web/src/panels/AgentHierarchy.tsx` | 12:07 | `F671771EFEE7` |
| `web/src/panels/AgentProfilesPanel.tsx` | 12:02 | `5382A10BF9CC` |
| `web/src/panels/AgentMemoryPanel.tsx` | 11:29 | `CA87A6110508` |
| `web/src/panels/AgentMessagesPanel.tsx` | 10:52 | `A6F9CE413A22` |
| `web/src/panels/ModelsPanel.tsx` | 11:10 | `29B16BB061C8` |
| `web/src/panels/DataPanel.tsx` | 10:21 | `1F84FC30DCB1` |
| `web/src/panels/RoutingPanel.tsx` | 04:53 | `F0269705A18A` |
| `web/src/panels/SettingsPanel.tsx` | 10:46 | `EB270A7C3DF6` |
| `web/src/panels/ApprovalOverlay.tsx` | 20:17 (prev day) | `1595C7BECDFF` |
| `web/src/panels/ControlBar.tsx` | 04:52 | `A22B87B6D259` |
| `web/src/flow/Canvas.tsx` | 15:04 | `1B98B49BC30B` |
| `web/src/flow/nodes.tsx` | 15:16 | `96DA9E29AEC4` |
| `web/src/flow/graph.ts` | 15:15 | `D386901290A4` |
| `web/src/api/contract.ts` | 11:18 | `4E66EB111F8B` |
| `web/src/api/client.ts` | 21:51 (prev day) | `745FC57A06EB` |

**Live capture caveat.** `docs/trace/ui-rebuild/shots/` contains 1440×900 PNGs produced by a
concurrent probe. Their rendered project header (“PROPOSED · V1 · 01A187D6”) does not match
the current `LibraryPanel.tsx`, so they are **not** used as evidence for “current render” in
this report. Source-read is the evidence. No live server or browser was run by this seat;
pixel fidelity and runtime behaviour remain **UNVERIFIED**.

### 0.4 Design read — the five boards in one table

| Board | Required representation (what must be visible) | Facts the UI needs |
|---|---|---|
| 06 Controls & HUD | Composer with send/attach/link, Spawn sub-hive, Re-route, Cancel, Kill worker, tabs Workspace/Swarm/Routing/History, HUD `4 SUPERAGENTS · 12 WORKERS · 98.2% GREEN +3.4%`, time-travel scrubber with timestamps and `routing state 12 of 38`, nav rail, library, minimap, dark refusal toast with `REFUSED BY NAME · NOT SILENT` | 40px targets, 10px radius, toast code + message + how_to_fix |
| 07 Spatial Canvas | 52px topbar with brand/search/bell/avatar, 72px nav rail, 240px library with Project selector, ACTIVE AGENTS (Nova/Vector/Spark), TEMPLATES (Content Writer/Data Analyst/Code Reviewer/Chatbot), honeycomb canvas with file cards (`router.ts`, `schema.prisma`, `bean.rs`, `agent.spec.ts`, `main.tsx`), central beehive chat (420×500), HUD, 220×150 minimap, bottom hint `0.1x–3.0x · Cmd+0 recentres the core hive · drag a worker to re-route` | project identity, active agents, template ids, file names/associations, HUD counts |
| 08 HMAS Hierarchy | Root `HyperAgent` (`root · 4 superagents · healthy`), SuperAgent rows with worker counts, Worker rows with template/state/score/retry, nesting `HyperAgent > SuperAgent > WorkerAgent > ToolAgent`, `nesting` rule, re-parent only while **not streaming**, kill/rewire confirmation-first, live topology with bee direction, state legend (healthy/running/queued/failed/awaiting_approval), card sizes | parent/child edges, per-node kind, label, state, progress, error code, worker count, ctx/score (where available) |
| 09 HMAS Governance | Profiles/import (`Default profile · active · 2 gates`, `Strict review · paused`, import folder or `.figcraft` manifest), Memory (run summary 12.4k ctx, decision log 3 rows, refusal ledger 5 rows, “Clear is confirmation-first; the ledger is append-only”), Mailbox (2 unread, Backend→Frontend, read-only, delivery is a branch event), Projects (HiveMind AI active, sandbox idle, project_id localhost only), Usage (4 superagents, 12 workers, 98.2% green, 38 calls, 12.4k ctx, derived counters), Message approval (Backend Architecture → Frontend Architecture, args_sha256, Deny/Approve, nonce/single-use/TTL 120s, “grants the engine one consume”) | profile status/gates, memory counts/ledger, message unread/branch identity, project list/status, usage totals/calls/ctx, approval branch evidence |
| 10 States & Refusals | Named refusal with code/message/how_to_fix and `ERROR FRAME → TOAST + LOG + LEDGER`, reconnect banner, empty/loading skeleton, kill confirmation dialog, replay read-only banner + no mutations, decision-gated code-diff review | refusal record persistence, run/task error detail, kill precondition, replay mode, `DIFF_*` gate status |

---

## 1. Capability matrix at a glance

Priority definitions: **P0** = false affordance, unsafe mutation, or a core HMAS capability
that is effectively unusable; **P1** = required design representation absent/broken while the
real data or endpoint exists; **P2** = polish, additive contract work, or future-surface gap.

| # | Capability | Backend reality | Current render | Verdict | Priority |
|---|---|---|---|---|---|
| 1 | Hierarchy / topology | `NODE_*` events, `SPAWN_SUB_HIVE`, `REWIRE_TASK`, history graph; only id/state/rev/kind/parent published | tree list + React Flow canvas + routing table | partial; design metrics/name fields missing | **P0/P1** |
| 2 | Profiles / import / delete | real HTTP GET/POST/DELETE, persisted Persona rows | `AgentProfilesPanel` (nested in Agents) | works, but no status/gates/confirm and dev base URL bypass | **P1** |
| 3 | Memory / list / clear | real HTTP GET/DELETE, max 8 rows returned | `AgentMemoryPanel` (nested) | works; destructive clear without confirm; no counts/ledger | **P1** |
| 4 | Mailbox / messages | real HTTP GET, real `AGENT_MESSAGE` event, no send endpoint (read-only by design) | `AgentMessagesPanel` (nested) | stale during run; unread/branch absent; event dropped | **P0/P1** |
| 5 | Projects | `GET /api/projects`, project materialized by `create_run`; no create/rename route | local random UUID + constant name + decorative chevron | **lying controller**; template insert can fail after reload | **P0** |
| 6 | Usage / tokens | `GET /api/usage/runs/{id}` (6 fields), `REPLY_DONE.usage`, `RUN_METRICS` | HUD + `ModelsPanel` (3 fields) | no in-place refresh after run; hidden fields | **P0/P1** |
| 7 | Cross-branch approval | `APPROVAL_REQUEST` (with args_preview on live, omitted on replay), `APPROVAL_RESOLVE`, server ledger; cross-parent send suspends for approval | generic modal per approval | works mechanically; branch/message/queue/ledger absent; expired card can trap | **P0/P1** |
| 8 | Run failures / refusals | `ERROR` frames with code/message/how_to_fix; `RUN_STATUS.error_code`; ledger primitive + approval rows only; no read endpoint | toast (max 4, ephemeral), chat error card, tool outcome list | no log/ledger surface; run error detail lost | **P1** |
| 9 | Replay / history | history HTTP, `HISTORY_REVISION_SET` → `HISTORY_SNAPSHOT`; snapshot has id/parent/state/rev only | scrubber + replay canvas | mutations still enabled; no run picker/timestamps/playback; replay all-worker | **P0/P1** |
| 10 | Files | `FILE_TREE_LIST`/`FILE_READ` over WS; real jail + 1MiB cap | `DataPanel` tree + preview | works; no breadcrumb/up, no board, no diff (decision-gated) | **P1** |
| 11 | Settings / model status | `SETTINGS_STATUS`, `GET /health` (database/transport/prompt assets), `usage` | `SettingsPanel` + `ModelsPanel` + library details | no board; model status not auto-loaded; health detail dropped; dead diff button | **P1** |

---

## 2. Hierarchy / topology

### Current render

- `AgentsPanel` is a single scrolling surface: title, empty state, `AgentHierarchy`, a flat
  grid of task cards, then tool activity, profiles, memory, and mailbox
  (`web/src/panels/AgentsPanel.tsx:31-75`). There is no separate HMAS console, no detail
  pane, and no topology view inside the Agents destination.
- `AgentHierarchy` groups tasks by `parentId`, sorts by label/id, indents by real depth,
  marks roots with `◆`, and selects the existing card. It renders only kind, state glyph,
  label, and kind label (`web/src/panels/AgentHierarchy.tsx:21-80`). It has no worker count,
  no branch summary, no root summary, no drag/re-parent affordance, and no confirmation.
- `Canvas` is the richest topology: React Flow with `buildNodes`/`buildEdges`, bee edges,
  minimap, HUD, selection, drag-reparent on node intersection, hide-finished/focus-branch
  filters (`web/src/flow/Canvas.tsx:146-350`, `web/src/flow/graph.ts:42-128`,
  `web/src/flow/BeeEdge.tsx:13-57`). The drag path selects the worker and immediately calls
  `rewireSelected(target.id)` (`Canvas.tsx:320-348`).
- `RoutingPanel` is a plain parent-edge table (`web/src/panels/RoutingPanel.tsx:14-36`). Its
  fourth column is headed **role** but renders `visualFor(task.state).label` — i.e. the
  state, not the role (`RoutingPanel.tsx:20,31`).
- `LibraryPanel` lists up to 8 active agents as hex tiles (`web/src/panels/LibraryPanel.tsx:41-60`).

### Real API

- WS events `NODE_SPAWN` / `NODE_DRAG` / `NODE_REWIRE` / `NODE_STATE`
  (`web/src/api/contract.ts:356-381`; decoded at `701-735`). The producer publishes
  `{task_id, state, rev, kind, parent_task_id}` only — no label, progress, error code, role,
  template, worker count, ctx, or score (`backend/crates/hiverouter-core/src/ws.rs:952-1009`;
  `backend/crates/hiverouter-core/src/execution.rs:743-779`).
- Commands `SPAWN_SUB_HIVE {parent_task_id, role}` and
  `REWIRE_TASK {task_id, new_parent_id, expected_rev}`
  (`web/src/api/contract.ts:49-57,100-115`; sent by `store.spawnSubHive` /
  `store.rewireSelected`, `web/src/state/store.ts:1805-1875`).
- Boot/replay graph from `GET /api/history/runs/{run_id}/state?revision=N`
  (`backend/crates/hiverouter-core/src/http.rs:570-598`) and `HISTORY_SNAPSHOT`
  (`contract.ts:323-327,873-880`). `GraphSnapshot` nodes carry only
  `{task_id, parent_task_id, state, rev}` (`backend/crates/hiverouter-core/src/graph.rs:20-40`).

### Design surface

Board 08 (`hivemind-hmas-hierarchy.png`): root **HyperAgent** summary row, indented
SuperAgent rows with `3 workers · running`, Worker rows with `react/vite · 55%` and
`t0421 · retry 2/3`, nesting legend, “re-parent a worker only while it is not streaming”,
confirmation-first kill/rewire buttons, a **live topology** mini-graph with bee direction,
and a shape+text state legend. Board 07 supplies the spatial canvas + HUD + minimap.

### Missing representation

1. **No semantic labels.** The server does not publish `label`; the store falls back to
   `Task ${shortId}` (`store.ts:1424`). The design’s `Backend Architecture`,
   `tailwind-builder`, `Prisma Generator` cannot be rendered from the real producer. This is
   a producer/contract gap, not a UI styling gap.
2. **No per-node metrics.** `progress`, `error_code`, `role`, `template_id`, `score`, `ctx`
   are absent from `NODE_*`; cards render `—` or fixed copy (`nodes.tsx:136-187`). A client
   may derive a SuperAgent’s child count from `tasks`, but not `12.4k ctx` or `retry 2/3`.
3. **No root summary.** Design root row (`root · 4 superagents · healthy`) is not built.
   Derivable from `tasks` (root + children), so this is UI work, not a contract gap.
4. **No topology inside the HMAS destination.** The live-edge graph only exists on the
   Workspace canvas; `Routing` is a table, `Agents` is a tree + cards. Board 08’s
   “HIERARCHY / WORKSPACE-JAILED + TOPOLOGY / LIVE EDGES” pair is not a single surface.
5. **Re-parent precondition absent.** Contract says `REWIRE_TASK` precondition “worker not
   streaming” (`docs/contracts/UI-COMMANDS-v1.md:34`); `rewireBlocker` checks run/selection/
   target/rev but not `state` (`store.ts:651-668`), and `ControlBar.canRewire` checks only
   `kind === 'worker'` (`ControlBar.tsx:33-34`). The canvas drop path has the same gap.
6. **No confirmation.** Drag-drop rewire fires immediately (`Canvas.tsx:345-348`); kill fires
   immediately (`ControlBar.tsx:105-113`). Board 08 says “kill/rewire confirmation-first”;
   board 10 shows the kill dialog.
7. **No radius-gated snapping / displacement preview / path priority.** The drop handler uses
   `getIntersectingNodes` (overlap) and picks the first candidate, not a 150px radius, closest
   path-aligned node, or collision preview (`Canvas.tsx:336-348`).
8. **Replay topology loses kind.** `buildReplayTasks` forces `kind: 'worker'` for every
   snapshot node (`graph.ts:158-163`), so historical topologies render as identical worker
   cards even when the snapshot was a SuperAgent/ToolAgent graph. The snapshot payload has no
   kind (see above).

### Exact fix seam

| Seam | Change | Dependency |
|---|---|---|
| `backend/.../execution.rs:743-779`, `ws.rs:952-1009` | publish additive `label`, `progress`, `error_code`, `role`/`template_id` on `NodeEvent` (contract already accepts label/progress/error at `contract.ts:237-246`; add role/template if the design needs them) | backend + contract additive fields; no new event |
| `web/src/state/store.ts:1401-1438` | map the new fields into `TaskRecord`; keep `null` for absent values | above |
| `web/src/flow/graph.ts:136-174` | preserve `kind`/`label` when the snapshot carries them; keep the worker fallback only for old snapshots | snapshot field addition |
| `web/src/panels/AgentHierarchy.tsx` | root summary, child counts, branch grouping, worker/tool tags, selection; no metric that is not in `TaskRecord` | store fields |
| `web/src/panels/AgentsPanel.tsx` | give HMAS its own section/tab (tree + topology + detail) instead of appending every operator surface to one scroll | IA decision (see §13.4) |
| `web/src/panels/RoutingPanel.tsx:20,31` | rename `role` header to `state` (or render the real role once published) | copy-only fix |
| `web/src/panels/ControlBar.tsx:32-37`, `store.ts:651-668` | add `state !== 'streaming'` (and running/awaiting-approval) preconditions; disable Re-route/Kill with explanatory title | none |
| `web/src/flow/Canvas.tsx:320-348` | require a confirmation step for drag rewire; implement radius/nearest-path selection only after design sign-off | UX decision |
| `web/src/flow/nodes.tsx:202` | remove `errorCode ?? 'Compile failed'`; render `errorCode` or the honest `—` / “error state” | none (see §2 flags) |

### Flags

- **P0** `SPAWN_SUB_HIVE` is hardcoded to `role: 'SuperAgent'` (`store.ts:1819`). The server
  only allows a SuperAgent parent to spawn worker/subagent, and a Worker parent to spawn
  subagent only (`execution.rs:108-142`). Selecting a SuperAgent and pressing Spawn is
  refused. With no selection, the store prefers the first SuperAgent
  (`store.ts:1811-1814`), so the default Spawn path is also refused once a SuperAgent exists.
  The button is therefore **broken for the normal HMAS hierarchy** until the role is chosen
  by parent tier (or a role selector is added and validated).
- **P1** `SuperAgentNode` labels a percentage as `workers`
  (`nodes.tsx:136-141`); `WorkerAgentNode` hardcodes `formatCtx(null)` (`:160-162`); the Tool card
  shows `score —` (`:182-187`). These are label/value mismatches even before the data gap is
  fixed.
- **P1** The empty state on the canvas says “No agents yet — submit a prompt” even when a
  snapshot rehydrate failed or is degraded; the store has `snapshot` and
  `isSnapshotDegraded` but no component reads them (see §9).
- **P2** `LibraryPanel` slices active agents to 8 without a `+N more` indicator
  (`LibraryPanel.tsx:42`), and the `State & settings` block shows only four dot states, not
  the 16-state vocabulary (`LibraryPanel.tsx:94-99`).

---

## 3. Profiles: list / import / delete

### Current render

`AgentProfilesPanel` is mounted at the bottom of the Agents scroll (`AgentsPanel.tsx:71`).
It fetches `GET /api/agents/profiles` on mount, decodes rows, marks a hardcoded set of ids as
built-ins, renders `display_name`, `tier`, `template_id`, tool count, refusal policy and
optional limits, shows a delete button for non-built-ins, and provides a prefilled JSON
textarea + “Import profile” button (`web/src/panels/AgentProfilesPanel.tsx:22-190`).
Delete has no confirmation; import has no in-flight disable; error text is the raw code/HTTP
status (`:93-111,114-125`); there is no refresh control and no detail view.

### Real API

| Method/path | Payload | Response | Evidence |
|---|---|---|---|
| `GET /api/agents/profiles` | none | `{profiles:[{id,display_name,tier,template_id,allowed_tool_ids,output_schema_id,refusal_policy,max_children,max_depth}]}` | `http.rs:748-768` |
| `POST /api/agents/profiles/import` | `{profile: Persona}` (body without `profile` is also accepted) | `200 {profile:{...},source:"api"}`; `400` schema/validation refusal | `http.rs:670-710` |
| `DELETE /api/agents/profiles/{profile_id}` | none | `{id,removed:true}`, or `404 {code:"E_PROFILE_NOT_FOUND"}`; deleting an override restores the embedded profile | `http.rs:712-746` |

The server validates `Persona` against the loaded templates and persists the row
(`http.rs:674-691`). It accepts **native Persona JSON only**; there is no folder or
`.figcraft` manifest import path.

### Design surface

Board 09 `PROFILES · IMPORT`: a profile row with status (`Default profile · active · 2 gates`),
a second row (`Strict review · paused`), copy “Import a folder or a .figcraft manifest;
nothing executes at import time”, and an `Import project` button.

### Missing representation

- **Status/gates** (`active`, `paused`, `2 gates`) are not fields on `Persona` or in the
  response. The UI shows `refusal_policy` and optional limits instead. Do not map “strict”
  to “paused”; they are different facts.
- **Folder / `.figcraft` import** is not supported by the endpoint. The current textarea is a
  real native-profile import, but it is not the design’s import shape.
- **No confirmation** on delete; no indication that deleting an override restores the
  built-in (the button says “Delete imported profile”, which is true but incomplete).
- **No detail/editor**: `allowed_tool_ids` names, `output_schema_id`, and `max_*` are decoded
  but only count/limits are shown (`AgentProfilesPanel.tsx:61-72,147-153`).
- **No retry** on load failure; no refresh; no success announcement beyond `Imported.`.
- **No template/licence detail** on import failure; the server refusal’s `message` and
  `how_to_fix` are discarded (`:102-110`).

### Exact fix seam

- `web/src/panels/AgentProfilesPanel.tsx`:
  - confirmation dialog for delete (reuse the board-10 kill-confirmation pattern);
  - in-flight state for import/delete, disable duplicate submits;
  - render `allowed_tool_ids` and `output_schema_id` in an expandable row;
  - render `code`, `message`, `how_to_fix`, `retryable` from the refusal body, not just
    `body.code`;
  - add a refresh control and a load error retry.
- Add a shared HTTP helper that honors `apiBaseUrl()` (see §3). The panel currently calls
  relative `fetch('/api/...')`, which is same-origin in production but fails in the Vite
  5173 dev server (no proxy in `web/vite.config.ts:10-14`; `apiBaseUrl()` returns `:8080`
  for port 5173 at `web/src/api/client.ts:146-160`).
- Backend (only if the board’s import shape is required): a manifest/folder importer and
  additive `status`/`gates` fields. Until then, the UI must not display those as if they
  existed.

### Flags

- **P1** no delete confirmation; destructive.
- **P1** relative fetch bypasses `VITE_API_BASE` / dev server target.
- **P2** hardcoded `BUILTIN_IDS` (`AgentProfilesPanel.tsx:22`) can drift from the server’s
  embedded profile set; derive “is built-in” from a server flag or omit the distinction.
- **P2** `status === 'loading'` copy is a paragraph, not a skeleton; the board-10 loading
  language expects skeleton bars.

---

## 4. Memory: list / clear

### Current render

`AgentMemoryPanel` fetches `GET /api/agents/memory` on mount, keeps only `{key,value}` rows,
renders profile cards with a `Clear` button per profile, and refreshes after a successful
delete (`web/src/panels/AgentMemoryPanel.tsx:19-124`). Clear is immediate — no confirmation,
no “what will be deleted” summary. The empty state is shown only when every profile has zero
rows (`:94-96`); a load failure collapses to a generic “Memory unavailable” (`:91-93`).

### Real API

`GET /api/agents/memory` → `{profiles:[{id,display_name,memories:[{key,value}]}]}`, with at
most **8** rows per persona (`backend/crates/hiverouter-core/src/http.rs:770-788`; the
`agent_memory(&persona.id, 8)` call). `DELETE /api/agents/memory/{agent_key}` →
`{agent_key, removed}` (`http.rs:790-806`). There is no count/ledger endpoint and no
“confirmation token”.

### Design surface

Board 09 `MEMORY`: `Run summary · 12.4k ctx`, `Decision log · 3 rows`,
`Refusal ledger · 5 rows`, copy “Clear is confirmation-first; the ledger is append-only”,
and a `Clear memory` action.

### Missing representation

- **Counts/summary**: the API returns a capped list, not category counts. 12.4k ctx, decision
  log rows, and refusal ledger rows are not available. The UI cannot compute them honestly
  from 8 key/value rows.
- **Ledger**: the run ledger primitive writes approval decisions
  (`backend/crates/hiverouter-tools/src/ledger.rs:1-105`;
  `backend/crates/hiverouter-core/src/tools_bridge.rs:330-409`), but there is no read route
  and no UI. No refusal ledger is produced at all.
- **Truncation**: the 8-row cap is invisible; a profile with more than 8 memories looks
  complete.
- **Confirmation**: clear is immediate; board 09 says confirmation-first.
- **Refresh**: no manual refresh; a memory written during the run won’t appear until remount.

### Exact fix seam

- `AgentMemoryPanel`: confirmation dialog with profile id + row count; refresh control;
  “showing latest 8” disclosure; render `removed`/failure with code/message.
- Backend (if board 09 is the target): add counts and a ledger read endpoint; append
  refused/denied tool outcomes to `run-ledger.jsonl` (the ledger type already supports
  `Refused`/`Denied`/`TimedOut`/`Cancelled`/`Failed`,
  `ledger.rs:31-77`). Do not synthesize a ledger in the browser.
- If no backend addition is approved: keep the card but label it “latest 8 keys; count and
  ledger not reported” — never render fabricated counts.

### Flags

- **P1** destructive clear with no confirmation.
- **P1** hidden 8-row cap; no count.
- **P2** error state hides the last-known rows instead of showing stale data with an error.

---

## 5. Mailbox / messages

### Current render

`AgentMessagesPanel` is the last block in the Agents scroll (`AgentsPanel.tsx:73`). It reads
`state.run?.id`, fetches `GET /api/agents/runs/{runId}/messages` once when the run id changes,
decodes `{id, from_task_id, to_task_id, kind, text, created_at}`, and renders a flat list of
`kind · from→to` plus the text (`web/src/panels/AgentMessagesPanel.tsx:20-108`). There is no
unread count, no subject, no branch label, no read-only banner, no manual refresh, and no
retry after an error. When no run exists the status stays `idle`, so the section renders blank
rather than an empty state (`:25-31,84-95`).

### Real API

`GET /api/agents/runs/{run_id}/messages` →
`{run_id,messages:[{id,from_task_id,to_task_id,kind,text,created_at}]}` (`http.rs:640-668`;
route at `:487`). The server also publishes a live `AGENT_MESSAGE` event with the same fields
(`contract.ts:314-322,947-971`; producer `execution.rs:721-741`), emitted when
`ask_parent`/`send_to_sibling` writes a row (`execution.rs:2410-2603`). Messages are written
by tools, not by the UI; there is **no send endpoint** and no read/unread endpoint. The
per-run message budget is 32 (`execution.rs:2563-2571`; `store.agent_message_count`,
`store.rs:556-572`).

**Critical wiring gap:** `AGENT_MESSAGE` is decoded but the store’s `applyEvent` switch has
no case for it (`store.ts:1265-1645`; grep finds `AGENT_MESSAGE` only in `contract.ts`). A
live message therefore never reaches any component. Because the panel only fetches when
`runId` changes, messages created during a run are invisible while the panel stays mounted —
they appear only after the panel remounts (nav away/back), a new run, or a reload.

### Design surface

Board 09 `MAILBOX · 2 UNREAD`: `Backend → Frontend · schema.prisma v3 ready`,
`Frontend → Backend · API drift report`, copy “Mailbox is read-only; delivery is a branch
event.”

### Missing representation

- **Live delivery**: `AGENT_MESSAGE` is dropped; the mailbox is stale by construction.
- **Unread state**: no read/unread field exists on the row or endpoint. “2 unread” cannot be
  rendered without a new server field; total message count is derivable/available
  (`agent_message_count`), but it is not unread.
- **Branch identity**: sender/recipient are task ids; task labels are absent (§2), and there
  is no branch name. The board’s Backend/Frontend names are not representable.
- **Dedicated surface**: mailbox is the fifth block in a long scroll, below tool activity,
  profiles, and memory; there is no unread badge in the nav.
- **Read-only semantics**: the panel looks like an inbox but does not say delivery is a
  branch event or that the surface is read-only.
- **Budget/limits**: the 32-message per-run budget is not shown.
- **Empty/loading**: blank idle; no skeleton/error/retry.

### Exact fix seam

- `store.ts:1265` — add `case 'AGENT_MESSAGE'` that appends/replaces by message id and filters
  to the current run; keep the event as the live source. This is a real event, not a fake
  controller.
- `AgentMessagesPanel` — read the store’s live message list (with the HTTP fetch as
  hydration), add a refresh, show a read-only banner, and group by `from→to`; use task ids
  until labels exist. If unread is required, add a backend read-state field; otherwise show
  “N messages in this run” and do not claim unread.
- IA: give mailbox its own tab/section or a nav badge fed by the store count; do not bury it.
- Backend (if design names are required): publish `from_label`/`to_label` or add them to the
  message row; do not reconstruct names from mock/seed data.

### Flags

- **P0** real `AGENT_MESSAGE` event dropped; mailbox never live-updates.
- **P1** “2 unread” has no backing field (contract gap).
- **P1** branch identity unavailable; only task ids.
- **P1** mailbox buried in Agents scroll; no badge.
- **P2** idle state renders blank instead of an honest empty state.

---

## 6. Projects

### Current render

The store creates a **client-local project** at store construction:

```ts
function baseProject() {
  return { id: uuidv7(), name: STRINGS.library.projectFallback, owner: STRINGS.chat.userLabel };
}
```

(`web/src/state/store.ts:323-329`; initial state at `:1723`; `projectFallback` is
`"HiveMind AI"`, `strings.ts:118`.) Every page load therefore gets a **new random project
id**, while the visible name stays `HiveMind AI`. `LibraryPanel` renders the name and a
decorative `⌄` glyph in a non-interactive `span` — there is no dropdown
(`web/src/panels/LibraryPanel.tsx:29-34`). `Header` shows the owner initials/name, not the
project (`web/src/panels/Header.tsx:69-74`). The id is sent with `SUBMIT_PROMPT`
(`store.ts:1802`) and `TEMPLATE_INSERT` (`store.ts:2058-2063`).

### Real API

`GET /api/projects` → `{projects:[{id,name,created_at}]}` (`http.rs:843-858`; route at
`:494`). There is **no** POST/PATCH/DELETE project route. A project is materialized
implicitly by `RunStore::create_run` via
`INSERT OR IGNORE INTO projects(id, name, created_at) VALUES (?1, ?1, ?2)` — i.e. the
server-side **name is the id** (`backend/crates/hiverouter-core/src/store.rs:86-110`).
`GET /api/history/runs` returns `project_id` for each run (`http.rs:525-542`), but the web
decoder keeps only `runId/status/errorCode` (`store.ts:432-453`).

### Design surface

Board 09 `PROJECTS`: `HiveMind AI · active`, `sandbox · idle`, `project_id · localhost only ·
PDF:256,275-282`. Board 07 library: `Project: HiveMind AI` selector at the top of the rail.

### Missing representation

- No real project list, no active/idle state, no selection, no creation, no rename.
- The displayed name is a client constant, not the server row. The server has no name
  separate from the id, so “HiveMind AI” is currently a **fabricated label**.
- The client-generated id changes on reload; it is not persisted. Consequence: after a reload
  with an active run, `TEMPLATE_INSERT` sends a project id that no longer equals
  `run.project_id` and is refused `E_SCHEMA_INVALID` (`ws.rs:236-238`). The same random-id
  churn also means each new run can create a new project row.
- The `⌄` affordance promises a picker that does not exist.

### Exact fix seam

- `store.ts:323-329` / `:1723` — replace `uuidv7()` with a stable project identity:
  - prefer `project_id` from the chosen history run (add it to `HistoryRunSummary` by
    keeping the field already returned by `GET /api/history/runs`);
  - otherwise `GET /api/projects` and adopt an existing project, or create a stable local
    id persisted under a namespaced key (only if the product accepts a local-only project).
- `store.ts:432-453` — keep `project_id` and `created_at` in `HistoryRunSummary`.
- `LibraryPanel.tsx:29-34` — replace the span with a real `<button>`/listbox bound to
  `project`; show the real id/name and an explicit “local-only” note.
- `Header.tsx` — show the active project name/id next to the owner, not just the owner.
- Backend only if creation/rename is required: add `POST/PATCH /api/projects`; do not
  implement a client-side project editor that only changes local state.

### Flags

- **P0** random project id per reload: lying project identity + `TEMPLATE_INSERT` breakage
  after reload.
- **P0** decorative dropdown (dead affordance).
- **P1** no `/api/projects` consumer despite the endpoint existing (hidden capability).
- **P2** project name == id on the server; the board’s human names need a naming decision
  (store the name or display a deterministic short id; do not invent “sandbox”).

---

## 7. Usage / tokens

### Current render

- `HudBadge` shows `HyperAgent · N SUPERAGENTS · N WORKERS · N% GREEN` from
  `RUN_METRICS` when present, otherwise `deriveHud(tasks)` (`web/src/panels/HudBadge.tsx:8-33`;
  `web/src/state/metrics.ts:24-47`). It renders a `live`/`derived` badge.
- `ModelsPanel` shows provider/model from `settings`, and — if a run exists — fetches
  `GET /api/usage/runs/{runId}` and renders **prompt/completion/total tokens** only
  (`web/src/panels/ModelsPanel.tsx:17-98`). The effect depends only on `runId`
  (`:67`), so it does not refetch when the run transitions to a terminal state.
- `REPLY_DONE.usage` is decoded with six nullable fields (`contract.ts:168-175,684-700`) but
  the store’s `REPLY_DONE` handler only clears `streaming` — it **drops the usage**
  (`store.ts:1390-1400`).
- `ModelsPanel` shows the “server has not reported provider/model” empty state unless
  Settings was opened, because `probeSettings()` is only invoked from `SettingsPanel`
  (`SettingsPanel.tsx:17-24`; `ModelsPanel.tsx:72-74`).

### Real API

| Source | Payload | Evidence |
|---|---|---|
| `GET /api/usage/runs/{run_id}` | `{run_id,usage:{prompt_tokens,completion_tokens,total_tokens,prompt_cache_hit_tokens,prompt_cache_miss_tokens,reasoning_tokens}\|null}` | `http.rs:808-841` |
| `REPLY_DONE.usage` | the same six fields | `contract.ts:168-175,684-700` |
| `RUN_METRICS` | `{superagents,workers,green_pct,delta}` | `contract.ts:247-252`; producer `execution.rs:653-696` |
| `TOOL_CALL` / `TOOL_RESULT` | tool-call identity/outcome; count derivable | `contract.ts:294-313`; store `:1458-1528` |

The endpoint returns `usage:null` until the run’s usage is persisted at run end
(`execution.rs:705-718`).

### Design surface

Board 09 `USAGE`: `4 SUPERAGENTS`, `12 WORKERS`, `98.2% GREEN`, `38 calls · 12.4k ctx ·
derived counters`, plus a progress bar. Boards 06/07 show the HUD badge with `+3.4%`.

### Missing representation

- **Live refresh**: after a run completes, `ModelsPanel` does not refetch while it stays
  mounted; the usage appears only after a remount or a new run id. The design’s usage card
  must update when the run ends.
- **Hidden fields**: cache-hit, cache-miss and reasoning tokens are offered by both the HTTP
  endpoint and `REPLY_DONE`, but never rendered.
- **Calls**: `toolActivity.length` can provide the design’s `38 calls` honestly; it is not
  used in the usage card.
- **Ctx**: there is no context-token field in `RUN_METRICS`, `REPLY_DONE`, or
  `/api/usage`; `12.4k ctx` cannot be rendered without a new producer field. The current
  `formatCtx(null)` is always `—`.
- **Per-agent usage**: no endpoint/field; only run totals.
- **Cost**: not settled anywhere; do not display a cost.
- **Green semantics**: the server computes `green_pct` as **completed / all tasks**
  (`execution.rs:669-677`), while the client fallback counts `healthy + completed`
  (`metrics.ts:22,45`). Board 08 shows a high green percentage while agents are running; the
  server value would be low/0 until completion. This is a semantic mismatch, not a styling
  issue.
- **Model status**: no auto-probe; no provider health/latency in the model surface.

### Exact fix seam

- `store.ts:1390-1400` — either store `event.payload.usage` (last reply) or, preferably, set a
  run-usage refresh trigger on terminal `RUN_STATUS` (the run totals are the authoritative
  read).
- `ModelsPanel.tsx:23-67` — depend on `run?.status` (or `isTerminal`) and refetch
  `/api/usage/runs/{runId}` after terminal; render all six fields; keep “Not reported” for
  nulls. Show a loading state instead of an empty gap.
- `ModelsPanel` / store init — call `probeSettings()` when the Models surface mounts (or once
  at boot) so provider/model is not dependent on visiting Settings.
- `HudBadge` — add the derived tool-call count (`toolActivity`) if the usage card needs it;
  keep `DerivedBadge`.
- `backend/.../execution.rs:669-677` — decide whether `green_pct` means completed or
  healthy+completed; if the design means green status, change the producer (and tests), not
  the label.
- Add context-token usage only if the backend tracks it; otherwise label the slot
  “not measured”.

### Flags

- **P0** usage does not refresh in place after the run finishes (only remount/new run).
- **P1** three of six usage fields hidden; calls not shown.
- **P1** HUD green percentage disagrees between server and derived fallback.
- **P2** `ModelsPanel` is empty if Settings was never opened (model status probe coupling).

---

## 8. Cross-branch approval

### Current render

`ApprovalOverlay` renders one full-screen `role="dialog"` per pending approval, with tool,
`args_sha256`, run/task/tool-call/approval ids, TTL countdown, nonce-missing banner,
stale banner, retry/dismiss on a resolve refusal, and Approve/Deny buttons gated by
`approvalActionsEnabled` (`web/src/panels/ApprovalOverlay.tsx:164-364`). The layout is a
global modal, not a board-09 card; multiple approvals are stacked full-screen dialogs
(`:220-361`). Evidence fields exist; **branch context does not**.

### Real API

`APPROVAL_REQUEST` payload (`contract.ts:253-280`): `approval_id`, `tool`, optional
`decision`, `nonce`, `run_id`, `task_id`, `tool_call_id`, `args_sha256`, `args_preview`,
`expires_at`. The live producer attaches a bounded, redacted `args_preview`
(`tools_bridge.rs:481-518`); a post-hello replay of a persisted row **omits** it
(`tools_bridge.rs:502-518`).

`APPROVAL_RESOLVE {approval_id, nonce, decision, reason?}` → `ACK` + `APPROVAL_RESOLVED`
(`contract.ts:84-89,100-115`; `ws.rs:874-899`). The server consumes the nonce once and
appends exactly one approval-decision row to `data/ledger/run-ledger.jsonl`
(`tools_bridge.rs:330-409`). Cross-parent `send_to_sibling` is a Mutate-class call: it
suspends for operator approval and writes the message once after approval
(`execution.rs:2483-2603`; `docs/CODEMAP/18-HMAS-RESEARCH.md:154-155`). The approval payload
carries the issuer `task_id` and the redacted args (which can contain the target task id),
but **not** a sender/receiver branch name or a route.

### Design surface

Board 09 `MESSAGE APPROVAL`: `Backend Architecture → Frontend Architecture`,
`schema.prisma v3 ready to merge`, `args_sha256 · 9f86d081884c…`, `Deny`/`Approve`,
`APPROVAL_RESOLVE · nonce · single-use grant · TTL 120 s`, “Approve executes no effect by
itself; it grants the engine one consume.” Board 10 `S14 NAMED REFUSAL` shows
`E_APPROVAL_TIMEOUT` with `how_to_fix` and `ERROR FRAME → TOAST + LOG + LEDGER`.

### Missing representation

- **No branch route.** The UI shows `run_id`/`task_id`/`tool_call_id`, not
  “Backend Architecture → Frontend Architecture”. Task labels are already absent from the
  producer (see §2), and there is no target-branch field. A client can at best render task
  ids or derive a parent chain from `tasks`; it cannot invent names.
- **No `args_preview`.** The contract decodes it, the live server sends it, the store drops
  it (`store.ts:1534-1545`), and the overlay has no field for it (`ApprovalOverlay.tsx:24-34`).
  The design’s message/argument evidence is unavailable even though it is on the wire.
- **No approval queue.** Every approval is a full-screen dialog; there is no count, inbox, or
  grouped-by-branch list. With multiple pending approvals the last rendered dialog wins the
  screen; earlier ones are still in the DOM.
- **No ledger link.** Approval decisions are written to `run-ledger.jsonl`, but no read route
  exposes them; the UI shows no “decided/allowed/denied” history.
- **Resolve failure detail is partial**: code + message are shown; `how_to_fix` is not
  (`ApprovalOverlay.tsx:303-308`).
- **Expired/missing-nonce dead end**: `dismissApproval` returns early when
  `approvalResolve[id]` is undefined (`store.ts:2027-2038`), and the only retry/dismiss block
  is rendered inside `resolveState?.refusal` (`ApprovalOverlay.tsx:303-331`). On server TTL
  expiry, `finalize_approval_timeout` fails the run but does **not** publish
  `APPROVAL_RESOLVED` (`execution.rs:2313-2368`), so the client record can remain with no
  close path. The nonce-missing card has the same shape: explanatory banner, no dismiss.

### Exact fix seam

- `store.ts:94-106,1530-1570` — add `argsPreview` to `ApprovalRecord` and copy it from the
  decoded payload (it is already validated and bounded by `contract.ts:762-785`).
- `ApprovalOverlay.tsx`:
  - render a redacted `args_preview` block, labelled display-only; keep `args_sha256` as the
    binding;
  - derive the issuer’s parent chain from `tasks` to show at least `Task A → Task B` ids;
  - add an approval queue/inbox frame (count, ordered list) instead of stacking dialogs;
  - render terminal states (`expired`, `stale`, `unconfirmed`) with a working Dismiss even
    when no resolve record exists; allow closing without resolving;
  - show `how_to_fix` on resolve failure.
- Backend if the design’s branch names are required: add `from_task_id`/`to_task_id` (or
  labels) to the approval payload, or publish labels on `NodeEvent` and derive the branch
  path; add a read-only ledger endpoint. Do not parse `args_preview` as authority.
- On approval timeout, either publish an `APPROVAL_RESOLVED`/terminal removal for the
  approval or have the client clear approvals whose run/task is terminal; the latter must be
  explicit and honest (e.g. show “approval expired; run failed E_APPROVAL_TIMEOUT”).

### Flags

- **P0** expired/missing-nonce approval can trap the UI with no close path.
- **P1** `args_preview` dropped — real cross-branch evidence hidden.
- **P1** no branch route/queue/ledger.
- **P1** `how_to_fix` not rendered for resolve failures.
- **P2** full-screen modal per approval; board 09 presents an inline card.

---

## 9. Run failures / refusals

### Current render

- `ToastRegion` is the only refusal surface: it maps up to **4** entries from
  `state.refusals`, each with `code`, `message`, `how_to_fix`, and a dismiss button
  (`web/src/panels/ToastRegion.tsx:7-49`). `pushRefusal` prepends, dedupes codes within
  500ms, and slices to 4 (`store.ts:750-760`). Entries are in-memory only; a reload loses
  them.
- `ChatCore` shows a run-level error only while the chat is visible and the nav is
  Workspace: `run.errorCode` → `clientRefusal(code)` card with `code`, `message`,
  `how_to_fix` (`web/src/chat/ChatCore.tsx:79-80,206-216`). `RUN_STATUS` stores only
  `error_code`, not the originating `ERROR` message (`store.ts:1344-1369`).
- `BrokenRouteNode` renders `task.errorCode ?? STRINGS.node.failed`; the fallback string is
  **“Compile failed”** (`web/src/flow/nodes.tsx:202`; `strings.ts:289`). For any error-kind
  node whose code is absent (which is currently every real node, because `NODE_*` does not
  publish `error_code`), the card states a specific compile failure that was never reported.
- `ToolActivityPanel` renders per-call outcomes and codes (`ToolActivityPanel.tsx:137-181`).
- Settings renders its own refusal block with code/message/how_to_fix
  (`SettingsPanel.tsx:51-64`); approval resolution renders code/message without
  `how_to_fix` (`ApprovalOverlay.tsx:303-308`).

### Real API

- `ERROR` WS event: `{code, message, how_to_fix, retryable}` (`contract.ts:181-186`).
- `RUN_STATUS`: `{run_id, status, error_code?}` (`contract.ts:199-203`).
- `TOOL_RESULT`: `{outcome, code?, bytes?, sha256?}` (`contract.ts:304-313`).
- `run-ledger.jsonl` supports refusal/denial/timeout/cancel/fail rows with `refusal_code`
  (`backend/crates/hiverouter-tools/src/ledger.rs:29-105`), but only approval decisions are
  appended by core (`tools_bridge.rs:330-409`). No read route exists in `http.rs:478-498`.
- The approval ledger row is the only persisted decision evidence; there is **no refusal
  log**, no run-failure list, and no endpoint for either.

### Design surface

Board 10: `S14 NAMED REFUSAL` (`E_APPROVAL_TIMEOUT` with code, message, `how_to_fix`,
`ERROR FRAME → TOAST + LOG + LEDGER`), reconnect state, empty/loading, kill confirmation,
replay read-only, decision-gated diff. Board 09 memory card: `Refusal ledger · 5 rows`,
“the ledger is append-only”.

### Missing representation

- **No persistent refusal log/ledger.** The toast is ephemeral and capped at 4; the design
  explicitly asks for `LOG + LEDGER`. A user who misses a toast cannot recover the failure.
- **No run-failure detail.** The run error card is hidden when the chat is hidden or another
  nav is active; it shows only the code-derived copy, not the server `ERROR` message or
  `how_to_fix`.
- **No task error detail.** `NODE_*` does not publish `error_code`, so the hierarchy cannot
  show a named task failure; the canvas fallback fabricates “Compile failed”.
- **No empty-vs-failed distinction.** `AgentsPanel` shows “No agents yet — submit a prompt”
  when `snapshot.status` is `failed`/`degraded`; `Scrubber` shows “No run history yet” when
  `history.refreshStatus === 'error'`. The store records both (`snapshot`,
  `history.refreshStatus/refusal`) but no component reads them (grep: `isSnapshotDegraded`
  is exported but unused; `history.refreshStatus` is only written).
- **No refusal count/severity/time.** `ToastRefusal` has `at` but the toast does not show it;
  the 4-entry list is not a ledger.
- **Approval refusal lacks `how_to_fix`.**

### Exact fix seam

- `store.ts:750-760` — retain a bounded but visible session log (do not silently slice to 4)
  with `at`, code, message, how_to_fix, retryable, source (command/event/transport). If the
  design requires persistence across reloads, add a backend endpoint; a browser-only list
  must be labelled “this session only”.
- Add a `RefusalLedger` surface (new board 13; see §13.4) fed by that log. Render code,
  message, how_to_fix, timestamp, and the originating run/task when known. Do not fabricate
  rows to fill it.
- `ChatCore` / `AgentsPanel` / `Header` — surface `run.errorCode` outside the chat; if the
  `ERROR` frame arrived for the same run, show its message/how_to_fix rather than the generic
  client copy.
- `backend/.../execution.rs` / `ws.rs` — publish `error_code` on `NODE_*` (additive; contract
  already accepts it) so the tree/canvas can show the real failure.
- `nodes.tsx:202` — replace `“Compile failed”` with `errorCode` or the neutral “error state”;
  a specific cause must come from the server.
- `Scrubber` / `AgentsPanel` / `Canvas` — render `snapshot.status`, `snapshot.incomplete`, and
  `history.refreshStatus/refusal`: a failed snapshot is not an empty run; a failed revision
  load is not “no history”.

### Flags

- **P1** no log/ledger; toast cap 4 and reload loss.
- **P1** run error detail lost outside the chat.
- **P1** fabricated “Compile failed” fallback.
- **P1** empty states lie on snapshot/history failure.
- **P2** approval refusal lacks `how_to_fix`.

---

## 10. Replay / history

### Current render

- `Scrubber` is a bottom footer when `nav === 'workspace'` (`App.tsx:96-98`). It renders a
  range input over `history.revisions`, commits on pointer-up/key-up, shows
  `routing state N of M`, and a `live` button that calls `exitHistoryReplay`
  (`web/src/panels/Scrubber.tsx:10-76`). It has no run picker, timestamps, revision-status
  markers, play/pause/step, or error/retry state.
- `Canvas` switches to `replayNodes`/`replayEdges` when `history.replayGraph !== null`, fits
  the view, disables node click and drag, and shows a replay banner
  (`web/src/flow/Canvas.tsx:146-155,303-355,388-394`). Node types are forced to `worker`
  by `buildReplayTasks` (`graph.ts:158-163`).
- `store.setHistoryRevision` sends `HISTORY_REVISION_SET`; `HISTORY_SNAPSHOT` sets
  `replayGraph` (`store.ts:1626-1641,2065-2102`). `exitHistoryReplay` is explicit and sends
  no command (`:2088-2102`).
- **Mutations are not blocked in replay.** `ControlBar` computes `canSpawn/canCancel/canKill/
  canRewire` from connection + active run only (`ControlBar.tsx:24-37`); `ChatCore` still
  submits; `Canvas` blocks only its own drag path. A user can send `SPAWN_SUB_HIVE`,
  `REWIRE_TASK`, `KILL_WORKER`, `CANCEL_RUN`, or `SUBMIT_PROMPT` while inspecting a historical
  revision.
- `history.refreshStatus/refreshRefusal` are recorded (`store.ts:947-975`) but no component
  reads them; `Scrubber` maps both loading and error to “No run history yet”
  (`Scrubber.tsx:61-65`). `history.latest`/`revisions` hold numbers only (`store.ts:145-158`);
  the timestamps and `run_status` returned by the API are discarded in
  `decodeHistoryRevisions` (`:486-509`).
- `diffOpen` is set by the Settings button but no component consumes it (see §12/§13.1).

### Real API

| Method/path or command | Payload | Response | Evidence |
|---|---|---|---|
| `GET /api/history/runs` | none | `{runs:[{run_id,project_id,status,error_code,created_at}]}` | `http.rs:525-542` |
| `GET /api/history/runs/{run_id}/revisions` | none | `{run_id,latest_revision,revision_count,revisions:[{revision,created_at,run_status}]}` | `http.rs:544-568` |
| `GET /api/history/runs/{run_id}/state?revision=N` | query `revision` | `{run_id,revision,schema_version,run_status,graph}` | `http.rs:570-598` |
| `GET /api/history/runs/{run_id}/messages` | none | `{run_id,messages:[...],truncated}` | `http.rs:600-638` |
| `HISTORY_REVISION_SET` | `{run_id,revision}` | `ACK` + `HISTORY_SNAPSHOT {run_id,revision,graph}` | `contract.ts:71-74,100-115`; `ws.rs:832-852` |

`GraphSnapshot` nodes carry `{task_id,parent_task_id,state,rev}` only (`graph.rs:20-40`), so
replay cannot recover node kind/label without a snapshot addition.

### Design surface

Board 10 `REPLAY / READ-ONLY`: `Replay (read-only) · revision held`, dashed bee graph,
`HISTORY_REVISION_SET → HISTORY_SNAPSHOT · no mutations while in replay`. Board 06
time-travel scrubber: `09:41:07 · routing state 12 of 38` with timestamp markers. Prompt 10
adds play/pause/step-back. Board 10 `S25 DECISION-GATED` is the only diff surface.

### Missing representation

- **Run picker**: the store silently chooses the current/active/first run
  (`chooseHistoryRun`, `store.ts:609-621`); the UI never lists runs. `GET /api/history/runs`
  is called internally but its `project_id`/`created_at` are dropped.
- **Timestamps/status markers**: real API has `created_at` and `run_status`; the scrubber
  has no markers and no timestamp.
- **Playback**: no play/pause/step; only a range input.
- **Mutation guard**: replay is view-only in the design; the UI is not.
- **Replay topology fidelity**: all nodes render as `worker`, so board 08’s topology cannot
  be inspected historically.
- **Error/empty distinction**: a failed revision read looks like an empty history.
- **Mutation confirmation**: even outside replay, rewire/kill lack the board-10 confirmation.
- **Diff**: the Settings button opens no modal; the contract says `DIFF_*` are decision-gated
  and not frozen (`docs/contracts/UI-COMMANDS-v1.md:45`; `contract.ts:153-157`).

### Exact fix seam

- `store.ts:432-453,486-509` — keep `project_id`, `created_at`, `run_status` in the decoded
  history rows; add `runs`/`revisionMeta` to `HistoryState` (`:145-158`).
- `Scrubber.tsx` — render timestamp/status markers from `revisionMeta`; show a loading/error
  state from `history.refreshStatus`; add a run picker (or a `HistoryPanel` using the same
  state); keep revision numbers exact (no max-index clamp).
- `ControlBar.tsx` / `ChatCore.tsx` / `store.ts` — add a single replay guard
  (`history.replayGraph !== null`) that disables every mutating control and refuses in the
  store if called programmatically; the canvas already blocks drag, but the command surface
  must too. This needs no server change.
- `backend/.../graph.rs` / snapshot types — add optional `kind`/`label` to snapshot nodes so
  `buildReplayTasks` can preserve topology; keep the worker fallback for old snapshots.
- Diff: remove the dead button or render the board-10 decision-gated explainer with
  `STRINGS.diff.decisionRequired` (`strings.ts:239-243`), “Propose/Apply” disabled. Never
  implement a fake diff.
- Play/pause/step: iterate the exact `history.revisions` list and call
  `HISTORY_REVISION_SET` per step; no new API needed.

### Flags

- **P0** mutations are allowed while in replay (contract/design violation; live run can be
  changed while inspecting history).
- **P0** code-diff button is dead; `diffOpen` has no reader.
- **P1** replay renders every node as `worker`; historical topology/type is lost.
- **P1** no run picker, timestamps, or playback.
- **P1** history load failure is rendered as “No run history yet”.
- **P1** playback controls absent despite prompt 10 and board 06’s scrubber.

---

## 11. Files / data

### Current render

`DataPanel` is a two-column surface: a workspace tree from `FILE_TREE_LIST` (directories
expand in place) and a read-only text preview from `FILE_READ`, with size and sha256
(`web/src/panels/DataPanel.tsx:12-122`). It shows only the root refresh and no breadcrumb,
parent/up control, search, filter, or file-type handling. The preview is plain `<pre>`, no
syntax highlighting. There is no file-write, no diff, and no association between a file and a
task/card.

### Real API

| Command | Payload | Result | Evidence |
|---|---|---|---|
| `FILE_TREE_LIST` | `{path?}` (optional workspace-relative dir) | `FILE_TREE_RESULT {root,entries:[{path,name,kind,size_bytes}]}` | `contract.ts:91-94,337-347`; `tools_bridge.rs:285-291` |
| `FILE_READ` | `{path}` (workspace-relative) | `FILE_CONTENT {path,sha256,bytes,text}` or a named refusal (`E_PATH_OUTSIDE_JAIL`, `E_PATH_NOT_FOUND`, `E_FILE_TOO_LARGE`) | `contract.ts:95-98,348-353`; `tools_bridge.rs` / `hiverouter-tools/src/commands.rs` |

The server canonicalises and jails paths and caps reads at 1 MiB (`contract.ts:1070-1094`);
the client pre-validates `FILE_READ` (`client.ts:463-469`; `contract.ts:1077-1089`).

### Design surface

There is **no dedicated Data/Files FigCraft board**. The design corpus covers files only as
(a) the left-sidebar “file/code hierarchy viewer” in the target spec (`HIVEMIND-TARGET-SPEC.md`
Prompt 3) and (b) the split-screen code-diff modal in Prompt 8 / board 10’s `S25`, which is
decision-gated. Board 07 shows floating file cards (`router.ts`, `schema.prisma`, `bean.rs`,
`agent.spec.ts`, `main.tsx`) around agents, but no file-browser board exists.

### Missing representation

- **Board**: no canonical Data/Files board; current `DataPanel` has no design target beyond
  generic surface primitives.
- **Navigation**: no breadcrumb, no parent/up, no “current directory” indicator; selecting a
  directory replaces the root list and the only way back is `Refresh root`.
- **Preview metadata**: bytes/sha shown; no line numbers, language, encoding, truncation
  notice, or copy/download.
- **Task association**: the canvas file cards are not connected to `FILE_TREE_*`; no task →
  file mapping is exposed.
- **Diff**: the only design reference is decision-gated; do not build it.
- **Write/upload**: not in the frozen command set; do not add a fake upload.

### Exact fix seam

- `DataPanel` — breadcrumb/up navigation; retain the current path; validate directory paths
  through the same client jail helper; refresh the current directory (not only root); render
  a loading skeleton and a named refusal with retry; distinguish empty directory from
  failure.
- New board 11 (see §13.4) for the tree/preview/diff-entry states.
- File cards on canvas only if a real task→file association exists (new backend field or a
  `FILE_TREE_*` read per task); do not infer from seed/mock names.
- `SettingsPanel` diff button: either remove or show the decision-gated explanation; do not
  implement `DIFF_*`.

### Flags

- **P1** no board for Data/Files.
- **P1** navigation loses the root and has no breadcrumb/up.
- **P1** the diff entry point is dead/decision-gated.
- **P2** preview has no line numbers/syntax/truncation/bytes formatting beyond raw bytes.

---

## 12. Settings / model status

### Current render

`SettingsPanel` probes `SETTINGS_STATUS` and `GET /health` once on mount, then renders:
provider, model, thinking mode, reasoning effort, key presence, server health + checks,
reduced-motion override, and a “Code-diff review” button (`web/src/panels/SettingsPanel.tsx:8-128`).
The health block shows `status` and `check.name:ok|failed` strings; `fetchHealth` discards
the database, transport and prompt-asset sections, and consumes the version without any
component rendering it (`web/src/api/health.ts:15-75`; `settings.healthVersion` is stored but
unused). The reduced-motion label is
`{STRINGS.states.replay} · reduced motion override`, i.e. “Replay (read-only) · reduced motion
override” (`SettingsPanel.tsx:108-115`). The diff button calls `openDiffModal`; nothing
renders `diffOpen` (`store.ts:282,1725,2131-2137`; grep finds no reader).

`ModelsPanel` duplicates provider/model and adds usage (see §7). `LibraryPanel`’s
`State & settings` disclosure duplicates provider/model/key and is the only other model
status outside Settings (behind a collapsed `<details>`).

### Real API

- `SETTINGS_STATUS {}` → `ACK` + `SETTINGS_STATUS_RESULT
  {provider,model,key_present,thinking?,reasoning_effort?}` (`contract.ts:90,100-115,330-336`;
  producer `ws.rs:777-796`). Key presence is a boolean; the key itself is never sent.
- `GET /health` → `{status,version,schema,uptime_secs,checks[],database{path,schema_version,
  error_code},transport{last_seq,graph_nodes,allowed_origins},prompt_assets{loaded,
  error_code}}` (`http.rs:860-911`). `fetchHealth` keeps `status`, `version` and flattened
  check names; Settings renders only status + checks (`health.ts:56-75`).
- `GET /api/usage/runs/{id}` for usage (see §7).

### Design surface

There is **no dedicated Settings/Model FigCraft board**. Board 06 shows the Settings nav
icon; board 07 shows provider/model/key in the library disclosure; board 09’s Usage card is
the only model/usage board. The design set cannot answer “what does model status look like”
without a new board.

### Missing representation

- **Model status**: no board, no provider health/latency/readiness, no “model loaded /
  configured / missing” summary beyond `key_present`.
- **Health detail**: database path/schema version, transport counters, prompt-asset load
  status and error codes are real but hidden.
- **Auto-probe**: provider/model is only known after visiting Settings; Models/Library can
  show `—` while the server has values.
- **Cost**: not settled; do not show.
- **Key input**: correctly absent; never add a field that writes a key into the UI.
- **Copy collision**: the reduced-motion label uses the replay string.
- **Dead diff**: see above.

### Exact fix seam

- `web/src/api/health.ts` — extend `HealthSnapshot` with `version` (already), `database`,
  `transport`, `prompt_assets`, `uptime`; keep the flattened checks but do not drop fields.
- `SettingsPanel` — render health detail, not just names; fix the reduced-motion label; remove
  or gate the diff button; keep key presence only (never a key input).
- `store.ts:2147-2169` / `ModelsPanel` — probe settings at boot or on Models mount (one
  `SETTINGS_STATUS`), so model status does not depend on visiting Settings.
- New board 12 (see §13.4) for provider/model/health; reuse board 09 for usage.
- Do not add cost or model-switching controls until the API exists.

### Flags

- **P0** dead diff button.
- **P1** no Settings/Model board.
- **P1** model status not auto-loaded; health details hidden.
- **P2** `Replay (read-only) · reduced motion override` copy collision.
- **P2** duplicated provider/model in three places (Settings, Models, Library) with different
  freshness.

---

## 13. Cross-cutting findings

### 13.1 Dead / lying controls

| Control | Where | Observed behaviour | Why it is wrong | Fix | Pri |
|---|---|---|---|---|---|
| `Code-diff review` | `SettingsPanel.tsx:116-122` | sets `diffOpen`, no component reads it; nothing opens | dead control; contract says `DIFF_*` decision-gated and not frozen (`UI-COMMANDS-v1.md:45`) | remove, or render board-10 decision-gated explainer with disabled Propose/Apply | **P0** |
| Project `⌄` | `LibraryPanel.tsx:29-34` | static `span`, no click/keyboard handler | promises a selector that does not exist | real button/listbox bound to project state; or remove the glyph | **P0** |
| `Swarm` tab | `Header.tsx:37-53`; `App.tsx:46-58` | sets `tab='swarm'`; workspaceView falls through to the same Canvas+Chat | no distinct Swarm view; tab highlight is the only change | map Swarm to board 08 hierarchy/topology or remove | **P1** |
| `History` tab | same | on Workspace it hides the chat; the Scrubber is always present; no history view | label promises history; behaviour is “hide chat” | make it a real history/replay view (board 10) or remove | **P1** |
| Header tabs under non-Workspace nav | `Header` always mounted; `App.tsx:60-73` | clicking Routing/History/Swarm changes `tab` but `content` is driven by `nav`, so nothing changes | controls appear global but only affect Workspace | scope tabs to Workspace, or make them drive `nav` | **P1** |
| `Spawn sub-hive` | `store.ts:1805-1820` | always sends `role: 'SuperAgent'`; server refuses from Super/Worker parents (`execution.rs:108-142`) | core hierarchy action fails for the normal Super→Worker branch | choose role by parent tier, or add a validated role selector; never send a fixed SuperAgent role | **P0** |
| `Kill worker` | `ControlBar.tsx:36-37,105-113`; `store.ts:1886-1898` | enabled for any selected worker, including queued/completed; sends immediately | server only kills Running/AwaitingApproval (`execution.rs:1142-1151`); no confirmation; design says confirmation-first | add `state ∈ {running, awaiting_approval}` + confirm dialog | **P0/P1** |
| Re-route | `ControlBar.tsx:33-34`; `store.ts:651-668`; `Canvas.tsx:320-348` | enabled for a streaming worker; drag drop sends immediately | contract precondition “worker not streaming”; design says re-parent only while not streaming; no confirmation | add streaming guard + confirm; radius-gated drop only after sign-off | **P0/P1** |
| Replay mutations | `ControlBar.tsx:24-37`; `ChatCore.tsx` submit | all mutating commands remain enabled while `history.replayGraph !== null` | board 10 says “no mutations while in replay”; live run can be changed | single replay guard in store + disabled controls | **P0** |
| `SuperAgent` card “workers” | `nodes.tsx:136-141` | label `workers`, value `progress%` | mislabelled metric | render child count under “workers”, progress under “progress” | **P1** |
| `Worker` card “ctx” | `nodes.tsx:160-162` | always `formatCtx(null)` → `—` | label suggests a value that is never available | remove until a ctx field exists, or label “not measured” | **P2** |
| Error-state card “Compile failed” | `nodes.tsx:202` | any error-kind node without `errorCode` renders “Compile failed” | fabricates a specific cause; `NODE_*` never sends `error_code` | render `errorCode` or neutral “error state” | **P1** |
| Routing “role” column | `RoutingPanel.tsx:20,31` | header “role”, value is the state label | mislabelled data | rename to “state” or publish/render real role | **P2** |
| Reduced-motion checkbox | `SettingsPanel.tsx:108-115` | label starts “Replay (read-only)” | copy collision with the replay state | “Reduced motion override” | **P2** |
| Chat empty copy | `ChatCore.tsx:224-226` | “Drop a repository into the core hive…” | no drop handler, no attach command; only text `SUBMIT_PROMPT` | change copy to what is supported, or implement a real ingest command | **P1** |
| Models usage | `ModelsPanel.tsx:23-67` | no refetch after run terminal | card looks like “no usage” when the server has it | refetch on terminal / store usage | **P0** |
| Mailbox | `AgentMessagesPanel.tsx:25-82` | fetch only on `runId`; `AGENT_MESSAGE` dropped | inbox is stale during the run | store case + live list | **P0** |
| Delete profile / Clear memory | profiles `:114-125`; memory `:74-83` | immediate destructive POST/DELETE | no confirmation; design says confirmation-first | confirm dialogs | **P1** |

### 13.2 Hidden capabilities (real backend, no UI path)

| Capability | Backend evidence | What is hidden | Seam |
|---|---|---|---|
| Projects list | `GET /api/projects` (`http.rs:843-858`) | real project rows and ids are never fetched; UI shows a local random id/constant name | `store.init` + `LibraryPanel`/`Header` |
| Live agent messages | `AGENT_MESSAGE` event (`contract.ts:380,947-971`; producer `execution.rs:721-741`) | store has no case; mailbox never live-updates | `store.applyEvent` |
| Approval argument preview | `args_preview` live producer (`tools_bridge.rs:481-518`) | store drops it; overlay cannot show the message/args evidence | `ApprovalRecord` + overlay |
| Usage detail | six fields in `/api/usage` and `REPLY_DONE` | UI shows 3 and ignores `REPLY_DONE`; cache/reasoning hidden | `ModelsPanel` + store |
| History metadata | `project_id`, `created_at`, `run_status` in history HTTP | decoders keep only ids/numbers; no timestamps/run picker/status markers | `decodeHistoryRuns/Revisions` |
| Health detail | `database`, `transport`, `prompt_assets`, `version`, `uptime` in `/health` | only status + flattened check names rendered; `healthVersion` unused | `health.ts` + `SettingsPanel` |
| Approval ledger | `data/ledger/run-ledger.jsonl` (`tools_bridge.rs:330-409`) | written but no read route; UI has no decision history | new read endpoint + board |
| Profile detail | `allowed_tool_ids`, `output_schema_id` in profiles response | count/limits only; names/schema hidden | profile detail row |
| Message budget | `agent_message_count`, 32/run (`execution.rs:2563-2571`) | not shown; mailbox cannot explain a refusal | panel copy |
| Tool task/run identity | `TOOL_CALL` carries `run_id`/`task_id` (`contract.ts:295-302`) | store’s `ToolActivityRecord` drops both; activity cannot group by branch | store/type + panel |

### 13.3 Contract / producer gaps (do not fake in the UI)

| Design fact | Current producer | Gap | Required action |
|---|---|---|---|
| Node labels (`Backend Architecture`, `tailwind-builder`, `Prisma Generator`) | `NodeEvent` only id/state/rev/kind/parent | no `label` | additive `label` on `NODE_*`; store already accepts it |
| Node progress/score/retry/ctx | none | `progress`/`error_code` fields exist in the contract but producer omits them; no score/ctx fields at all | additive producer fields; show `—` until then |
| Node role/template (`react/vite`, `WORKER`) | task row has `role`; event has only kind | no role/template on event | additive `role`/`template_id` or label |
| Cross-branch route (`Backend → Frontend`) | approval has issuer `task_id` + redacted args; no target/branch labels | no route | additive `from/to` or labels; derive only task ids for now |
| Mailbox unread count | no read/unread state | no field | additive read state or show total, not unread |
| Refusal ledger | ledger type supports refusal rows; core only appends approval decisions; no read route | no persisted refusal log | append refusals + read endpoint, or label session-only log |
| Profile active/paused/2 gates | `Persona` has `refusal_policy`/limits only | no status/gates | additive fields or omit the design elements |
| Folder/`.figcraft` import | import accepts Persona JSON only | no manifest importer | backend importer or keep native JSON and label it |
| Project names/active/idle | server names == ids, no status | no name/status/create route | add name/status/CRUD or display real ids |
| Context tokens (`12.4k ctx`) | not tracked in run metrics/usage | no field | add producer field or “not measured” |
| `DIFF_*` | decision-gated, not frozen (`contract.ts:153-157`) | no commands | do not implement; decision required |

### 13.4 Surfaces that need a new FigCraft board

| Proposed board | Why the existing set is insufficient | Required frames / states | Data dependency | Priority |
|---|---|---|---|---|
| **11 · Data & Files** | boards 06/07/08 cover canvas/cards, not a file browser; prompt 3/8 are prose only | tree (dir/file/empty/error), preview (text/bytes/sha/truncated), breadcrumb/up, read-only banner, diff entry marked decision-gated | existing `FILE_TREE_*`; task→file association is a contract gap | **P1** |
| **12 · Settings & Model Status** | no board shows provider/model/key/health/reduced-motion; board 09 only shows usage | provider/model/thinking/effort, key present/missing (never the key), health status + checks + database/transport/prompt assets, reduced motion, no diff (or gated) | existing `SETTINGS_STATUS` + `/health` | **P1** |
| **13 · Run Detail: Tool Activity, Failures & Refusal Ledger** | board 08 shows one ToolAgent card; board 10 shows one named refusal; neither shows a list/ledger or run-level failure summary | tool-call rows (tool, args_sha256, turn, outcome, code), run error card with message/how_to_fix, refusal ledger rows (session/persisted), empty/loading | existing `TOOL_*`/`ERROR`/`RUN_STATUS`; persisted ledger needs backend | **P1** |
| Extend **09** with an approval inbox frame | board 09 shows one approval card; current UI stacks full-screen dialogs | queue count, sender→receiver ids (names when available), args_preview, TTL, deny/approve, decided/expired, ledger link | `args_preview` + task labels/route | **P0/P1** |
| Extend **10** with a run picker + revision markers | board 10 shows one scrubber; current UI has no run list, timestamps or playback | run list (project/status/time), revision markers (time/status), play/pause/step, replay guard | history HTTP metadata | **P1** |

No new board is needed for the Workspace canvas, hierarchy tree, or HUD — boards 07/08/06
cover them; the work there is implementation, not design invention. The `Swarm` tab can reuse
board 08 rather than earning a new board.

### 13.5 Prioritized rebuild matrix

| Pri | Item | Type | Exact seam | Dependency |
|---|---|---|---|---|
| **P0** | Spawn role fixed to `SuperAgent` | broken control | `store.ts:1805-1820` + `execution.rs:108-142` | none |
| **P0** | Replay allows live mutations | unsafe/lying | `ControlBar.tsx`, `ChatCore.tsx`, `store.ts` guard | none |
| **P0** | Random project id / fake project name / dead chevron | lying controller | `store.ts:323-329,432-453,1723`; `LibraryPanel.tsx:29-34` | `/api/projects` (exists) |
| **P0** | Expired/missing-nonce approval traps UI | dead end | `ApprovalOverlay.tsx:287-331`; `store.ts:2027-2038` | none (clear terminal records) |
| **P0** | Code-diff button dead | dead control | `SettingsPanel.tsx:116-122`; `store.ts:2131-2137` | decision on diff |
| **P0** | Mailbox never live-updates | hidden event | `store.ts:1265`; `AgentMessagesPanel.tsx` | none |
| **P0** | Usage does not refresh in place after run | stale representation | `ModelsPanel.tsx:23-67`; `store.ts:1390-1400` | none |
| **P1** | No node label/progress/error from server | contract/producer gap | `execution.rs:743-779`; `ws.rs:952-1009` | additive fields |
| **P1** | Hierarchy lacks root summary/worker counts/topology | missing representation | `AgentHierarchy.tsx`; `AgentsPanel.tsx` | store fields |
| **P1** | Replay all-worker / no run picker / no timestamps | missing representation | `graph.ts:136-174`; `store.ts:432-509`; `Scrubber.tsx` | snapshot/history fields |
| **P1** | Profiles no status/gates/confirm/error detail | partial | `AgentProfilesPanel.tsx` | additive profile fields for status/gates |
| **P1** | Memory no confirmation/counts/ledger | partial/destructive | `AgentMemoryPanel.tsx` | counts/ledger endpoint |
| **P1** | Approval no branch/args_preview/queue/ledger | missing representation | `store.ts:94-106,1534-1545`; `ApprovalOverlay.tsx` | `args_preview` (exists) + route/labels |
| **P1** | No refusal log/ledger; run error outside chat | missing representation | `store.ts:750-760`; new surface | ledger read endpoint or session-only label |
| **P1** | Empty vs failed snapshot/history | lying empty state | `AgentsPanel.tsx:36`; `Scrubber.tsx:61-65` | none (state exists) |
| **P1** | Data no breadcrumb/board; diff dead | missing representation | `DataPanel.tsx`; new board | board 11 |
| **P1** | Settings/Models no board; model status not auto-loaded; health detail dropped | partial | `SettingsPanel.tsx`; `health.ts`; new board | board 12 |
| **P1** | SuperAgent “workers” label; Routing “role” header; “Compile failed” | lying representation | `nodes.tsx:136-141,202`; `RoutingPanel.tsx:20` | none |
| **P1** | Header Swarm/History/tab scoping | dead/confusing controls | `Header.tsx`; `App.tsx:46-73` | IA decision |
| **P1** | Profiles/memory/messages relative fetch bypass dev base URL | integration | `AgentProfilesPanel.tsx:42`; `AgentMemoryPanel.tsx:25`; `AgentMessagesPanel.tsx:33`; `ModelsPanel.tsx:30` | shared HTTP helper |
| **P2** | Snapping radius / collision preview / bee path alignment | interaction parity | `Canvas.tsx:320-348`; `BeeEdge.tsx` | design sign-off |
| **P2** | Library overflow count; 16-state disclosure | polish | `LibraryPanel.tsx:42,94-99` | none |
| **P2** | Reduced-motion copy collision; chat drop copy | copy | `SettingsPanel.tsx:114`; `ChatCore.tsx:224-226` | none |
| **P2** | Approval inline vs modal; queue styling | board extension | `ApprovalOverlay.tsx` | board 09 extension |

---

## 14. The “no fake controller” boundary (binding)

The rebuild may add **view state and command dispatch**, not a second HMAS engine. The UI is
allowed to be smart about presentation; it is not allowed to become the source of truth for
hierarchy, profiles, memory, projects, approvals, messages, usage, history, or failures.

### 14.1 Allowed sources for every rendered fact

1. A value decoded from a real server event (`NODE_*`, `RUN_*`, `REPLY_*`, `TOOL_*`,
   `APPROVAL_*`, `AGENT_MESSAGE`, `FILE_*`, `SETTINGS_*`, `HISTORY_*`, `ERROR`).
2. A value fetched from a real HTTP endpoint documented in §16.
3. A value **derived** from (1)/(2) by a pure function, and visibly labelled `derived`
   (the HUD already has `DerivedBadge`, `StatePack.tsx:42-52`).
4. Static design copy (`STRINGS`), not static data that looks live.

### 14.2 Prohibited

- No client-side creation of tasks, edges, agent kinds, profiles, memories, projects,
  messages, approvals, revisions, usage, or ledger rows. “Optimistic” is allowed only for
  the existing prompt row, which is retracted on a named refusal (`ChatCore.tsx:121-159`).
- No synthesized node metrics, labels, retry counts, ctx, costs, or “green” values. If the
  producer does not send it, show `—` / “not reported” / “not measured”.
- No seed/mock data to fill a production surface. `api/mock.ts` is a dev-only test double
  gated by `import.meta.env.DEV && VITE_UI_MOCK === '1'` (`web/src/api/index.ts:31-70`;
  `mock.ts:1-10`); it must never be used as design evidence or as a fallback when the WS is
  down. The `DEV MOCK · simulated data` banner (`App.tsx:85-89`) is the only legitimate use.
- No new command/event names. The frozen command set is 14 (+3 decision-gated `DIFF_*`),
  the event set is 24 (`contract.ts:119-157,385-410`; `UI-COMMANDS-v1.md:7-15,28-61`).
- No `DIFF_*` implementation, no fake diff viewer/apply, no `PROJECT_CREATE`, no
  `REFUSAL_LIST` invented as a client-only “ledger” unless it is explicitly labelled
  session-only and does not claim persistence.
- No hidden write path: profile import/delete and memory clear use the real HTTP routes;
  approvals use `APPROVAL_RESOLVE` with the server nonce; project id is the one sent in
  `SUBMIT_PROMPT`/`TEMPLATE_INSERT`; no direct SQLite/localStorage as authority. Canvas
  position overrides are the one intentional local view state
  (`Canvas.tsx:39-80`), and they are keyed by run/task and never sent as graph truth.
- No “success” UI before `ACK`; no removal of an approval that implies a decision before
  `APPROVAL_RESOLVED`. Dismissing an expired/stale/unconfirmed read-only card is a local UI
  dismissal (not a decision) and must be labelled as such; a terminal-run cleanup must not
  claim the request was approved or denied. No optimistic hierarchy mutation. The store
  already documents ACK-before-mutation (`store.ts:1-5`) — preserve it.

### 14.3 Required labels for unknowns

| Situation | Required render | Never |
|---|---|---|
| `settings.provider/model === null` | “Not reported” / “Server has not reported provider/model yet” | a guessed provider/model |
| `RUN_METRICS` absent | derived values + `derived` badge; `—` when no tasks | 0 or a plausible count |
| Node label/progress absent | `Task <short-id>` + `—` | “Backend Architecture”, “55%”, “12.4k ctx” |
| Node error code absent | neutral “error state” | “Compile failed” |
| Usage `null` | “Usage not reported yet” + loading/refresh | 0 tokens |
| History not loaded/error | “History unavailable” + retry | “No run history yet” |
| Snapshot failed/degraded | banner naming the incomplete reason | “No agents yet” |
| Unread not available | total count / “unread not reported” | a fabricated badge |
| Approval no nonce | banner + close/dismiss (no resolve) | a generated nonce |
| Ledger not read | “Not exposed” / session-only label | an invented ledger |

### 14.4 Review gate

A PR that adds any HMAS surface must answer “yes” to all of these:

1. Every write calls an existing command/HTTP route and waits for `ACK`/2xx.
2. Every rendered metric has a source (event/HTTP/derived+label) named in the PR.
3. Every absent field renders an honest unknown, not a default.
4. Replay blocks all mutations (control disabled **and** store guard).
5. No `diffOpen`-style state without a renderer; no button without a handler.
6. No mock/seed values in a production path; `import.meta.env.DEV` guard still holds.
7. The surface is reachable from the nav/tab IA, and its empty/loading/error states exist.

---

## 15. Acceptance checks for the rebuild

These are the minimum checks implied by the findings above. They are proposed, not run by
this seat.

### 15.1 Source-level checks

- `rg "diffOpen" web/src` must return only a rendered component path or nothing; no
  setter-only state.
- `rg "AGENT_MESSAGE" web/src/state/store.ts` must hit a `case` arm.
- `rg "args_preview" web/src/panels/ApprovalOverlay.tsx web/src/state/store.ts` must hit both.
- `rg "prompt_cache_hit_tokens|reasoning_tokens" web/src/panels/ModelsPanel.tsx` must hit.
- `rg "apiBaseUrl|fetch\\('/api" web/src/panels` — every panel HTTP call must go through the
  shared base URL or a shared client.
- `rg "replayGraph" web/src/panels/ControlBar.tsx web/src/chat/ChatCore.tsx web/src/state/store.ts`
  must show a guard in all three.
- `rg "Compile failed" web/src` must not hit a production fallback.
- `rg "role: STRINGS.node.superAgent" web/src/state/store.ts` must not exist; the role must be
  chosen from the parent tier or a validated control.
- Design geometry test (`web/tests/designParity.test.ts`) must still pass; the current canvas
  uses `LAYOUT` for minimap and library width.

### 15.2 Runtime scenarios (real server, not the mock)

1. **Spawn hierarchy**: create a run → spawn a SuperAgent from root (succeeds) → select the
   SuperAgent → spawn a Worker (succeeds after role fix); select a Worker → spawn a subagent
   (succeeds only with the correct role); verify each refusal names a code and remains
   visible.
2. **Replay**: complete a run with ≥2 revisions → open History/Scrubber → select an old
   revision → verify all mutating controls are disabled and no command is sent; press Live →
   verify the live graph returns.
3. **Project reload**: create a run → reload the page → verify the project id is the run’s
   `project_id` (not a new uuid) and `TEMPLATE_INSERT` succeeds.
4. **Mailbox**: run a plan that emits `ask_parent`/`send_to_sibling`; verify the message row
   appears live (no reload) and the unread/total label is honest.
5. **Usage**: complete a run → verify the Models usage card refreshes without a reload and
   shows the six fields the API reports (or “not reported” for nulls).
6. **Approval timeout**: let an approval TTL expire → verify the run fails with
   `E_APPROVAL_TIMEOUT`, the approval card is closable, and the toast/ledger shows the code +
   how_to_fix. (Server currently does not publish `APPROVAL_RESOLVED` on timeout; the client
   must handle the terminal run explicitly.)
7. **Approval cross-branch**: trigger a cross-parent `send_to_sibling` → verify the card shows
   at least the issuer/target ids and the redacted args preview (no fabricated names).
8. **Empty vs failed**: kill the server, open the UI → verify the snapshot failure is a named
   banner, not “No agents yet”; restore, reload → verify recovery.
9. **Files**: open root → open a directory → go up/root → read a file → trigger a jail/size
   refusal and verify code/how_to_fix + retry.
10. **Diff**: click the diff control → verify it is either absent or an explicit
    decision-gated explainer; no fake editor.

### 15.3 Evidence to attach to each fix

- Command + observed output for the runtime scenario (per the repo’s definition of done).
- The exact server event/HTTP sample (redacted) that the UI rendered.
- A screenshot of the empty/loading/error state, not only the happy path.
- A refusal test for every mutating control (at least one named code).

---

## 16. Evidence index

### 16.1 Design

- `docs/ui/figcraft/HiveMind/README.md:7-48` — build, board table, composition geometry,
  PROPOSED-ADDITION note.
- `docs/ui/figcraft/ASSETS/hivemind-hmas-hierarchy.png` — board 08.
- `docs/ui/figcraft/ASSETS/hivemind-hmas-governance.png` — board 09.
- `docs/ui/figcraft/ASSETS/hivemind-spatial-canvas.png` — board 07.
- `docs/ui/figcraft/ASSETS/hivemind-controls-and-hud.png` — board 06.
- `docs/ui/figcraft/ASSETS/hivemind-states-and-refusals.png` — board 10.
- `docs/ui/figcraft/HiveMind/SPEC/HIVEMIND-TARGET-SPEC.md:43-186` — canvas, cards, sidebar,
  proximity, HUD, errors, diff, macro, time travel.
- `docs/ui/figcraft/HiveMind/SPEC/HIVEMIND-TARGET-SPEC.md:188-209` — three transition rules
  (pure graph, context isolation, async event streams).

### 16.2 Current UI

| File | Relevant evidence |
|---|---|
| `web/src/App.tsx:46-98` | nav/tab composition; tabs only affect Workspace; ControlBar/Scrubber only on workspace |
| `web/src/state/store.ts` | graph/task state, commands, history, approvals, settings, files, refusals; key lines cited in §2-§13 |
| `web/src/api/contract.ts` | frozen commands/events/refusals; optional node fields; approval args_preview; usage |
| `web/src/api/client.ts:146-160,455-501` | `apiBaseUrl`, ACK/error transport |
| `web/src/flow/Canvas.tsx` | React Flow, minimap, HUD, drag rewire, replay nodes, filters |
| `web/src/flow/graph.ts` | dagre layout, edges, replay projection |
| `web/src/flow/nodes.tsx` | card renderers and metric fallbacks |
| `web/src/flow/BeeEdge.tsx` | bee direction/broken edge |
| `web/src/panels/AgentsPanel.tsx` | HMAS surface composition |
| `web/src/panels/AgentHierarchy.tsx` | tree projection |
| `web/src/panels/AgentProfilesPanel.tsx` | profile HTTP/import/delete |
| `web/src/panels/AgentMemoryPanel.tsx` | memory HTTP/clear |
| `web/src/panels/AgentMessagesPanel.tsx` | mailbox HTTP |
| `web/src/panels/ApprovalOverlay.tsx` | approval evidence/actions/TTL |
| `web/src/panels/ModelsPanel.tsx` | usage render/refetch |
| `web/src/panels/DataPanel.tsx` | file tree/preview |
| `web/src/panels/RoutingPanel.tsx` | routing table |
| `web/src/panels/SettingsPanel.tsx` | settings/health/diff |
| `web/src/panels/ControlBar.tsx` | spawn/reroute/cancel/kill preconditions |
| `web/src/panels/Header.tsx` | tabs/connection/owner |
| `web/src/panels/LibraryPanel.tsx` | project/active agents/templates/settings details |
| `web/src/panels/Scrubber.tsx` | history slider/live exit |
| `web/src/chat/ChatCore.tsx` | composer/run error/empty copy |
| `web/src/state/metrics.ts` | HUD derivation |
| `web/src/state/approval.ts` | approval classifier/replay state |
| `web/src/state/strings.ts` | copy/refusal table; `diff` decision-gated copy |

### 16.3 Backend

| File | Relevant evidence |
|---|---|
| `backend/crates/hiverouter-core/src/http.rs:478-498` | router: history, agent messages/profiles/memory, usage, projects, health |
| `http.rs:525-598` | history runs/revisions/state |
| `http.rs:600-668` | chat history and agent messages |
| `http.rs:670-806` | profile import/delete/list, memory list/clear |
| `http.rs:808-858` | usage and projects |
| `http.rs:860-911` | health payload |
| `backend/crates/hiverouter-core/src/ws.rs:649-899` | command dispatch; approval ledger; node events |
| `ws.rs:952-1009` | `NODE_*` producer fields; spawn publisher |
| `backend/crates/hiverouter-core/src/execution.rs:108-163` | spawn role/tier policy |
| `execution.rs:653-779` | run metrics, usage persistence, node publish |
| `execution.rs:1016-1112` | submit/spawn |
| `execution.rs:1133-1168` | kill rules |
| `execution.rs:2255-2368` | approval timeout terminal handling |
| `execution.rs:2410-2603` | ask_parent/send_to_sibling; cross-parent approval |
| `backend/crates/hiverouter-core/src/graph.rs:20-40` | `TaskNode`/`GraphSnapshot` fields |
| `backend/crates/hiverouter-core/src/store.rs:86-128` | project materialization/list |
| `store.rs:526-599` | agent message persistence/count/list |
| `backend/crates/hiverouter-core/src/tools_bridge.rs:330-518` | approval ledger + request projection |
| `backend/crates/hiverouter-tools/src/ledger.rs:1-105` | ledger outcomes/row fields |
| `docs/contracts/UI-COMMANDS-v1.md:28-99` | frozen command/event/refusal/surface matrix |

---

## 17. Open decisions (owner input needed)

1. **Project semantics** — is a project a local-only identity (stable client id) or a
   server-owned row with a human name/status? The current server names rows by id and has no
   create/rename route. The board’s `HiveMind AI`/`sandbox` names need a decision.
2. **Node metadata** — which additive fields will the backend publish (`label`, `progress`,
   `error_code`, `role`, `template_id`, `ctx`)? The design cannot be met without at least
   `label` and `error_code`; `ctx`/score needs a tracking decision.
3. **Cross-branch evidence** — add `from`/`to` branch ids/labels to `APPROVAL_REQUEST`, or
   accept task ids + `args_preview` as the v1 representation?
4. **Ledger** — add a read-only ledger endpoint, or expose only a labelled session log? The
   ledger file exists but has no read route and only approval rows are appended.
5. **Unread** — add a read-state field, or drop “unread” from the mailbox board and show a
   total?
6. **Profiles** — implement folder/`.figcraft` import and status/gates, or keep native
   Persona JSON and amend the board?
7. **History IA** — is the run picker part of History, Projects, or the Scrubber? Does the
   `History` tab become the replay surface or does the existing bottom scrubber remain the
   only entry?
8. **Swarm tab** — map it to board 08 (hierarchy/topology) or remove it; it currently renders
   Workspace.
9. **Diff** — keep the board-10 decision-gated explainer in Settings, or remove the control
   until `DIFF_*` is frozen?
10. **Green metric** — is `green_pct` completed-only or healthy+completed? Server and client
    fallback currently disagree.
11. **Context tokens** — is `12.4k ctx` a required v1 fact? If yes, the producer must track
    it; otherwise the board needs a “not measured” variant.

---

## 18. Bottom line

- **Capability exists, surface does not.** Most HMAS primitives are real: profile
  import/delete, memory list/clear, mailbox rows + live event, project rows, run usage,
  approvals with nonce/TTL, a ledger file, cross-branch message approval, hierarchy events
  and replay. The UI exposes only part of each, and several real events/fields are dropped in
  the store.
- **The most dangerous defects are not visual.** A hardcoded spawn role breaks the normal
  Super→Worker path; replay leaves live mutations enabled; the project id is regenerated on
  every load; an expired approval can trap the UI; the diff button is dead. These are P0.
- **The design boards ask for facts the producer does not publish.** Node labels, progress,
  error codes, ctx/score, branch names, unread, and profile status/gates are contract gaps.
  The rebuild must either add them additively or render honest unknowns; it must not fill
  them with seed/mock values.
- **Three new boards and two board extensions are justified** (§13.4): Data & Files,
  Settings & Model Status, Run Detail (tool/failure/ledger); extend 09 with an approval inbox
  and 10 with a run picker/revision markers. Boards 06/07/08 already cover controls, canvas,
  hierarchy/topology, and HUD.
- **The no-fake-controller boundary is the acceptance gate** (§14): every fact has a real
  source; every write is ACKed; every unknown is labelled; replay is read-only; no `DIFF_*`
  fiction; no mock data in production.

<!-- END -->

