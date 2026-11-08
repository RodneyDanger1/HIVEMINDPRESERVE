# 02 - WEAVER control/wiring audit (SEAT B)

Status: **AUDIT ONLY — no product file was edited by this seat, no process was restarted, no provider endpoint was called.**
Owner: SEAT B (`/weaver`).
Date: 2026-11-07T23:10–23:35 America/Chicago.
Workspace: `C:\Users\Riley\Desktop\hiverouter-v1`.
Report path: `docs/trace/orca-wave/02-weaver.md` (the only file written by this seat).

This is a control/wiring audit, not a root-cause audit. Shared root causes for the hive layout, replay
kind projection, terminal reconcile and HUD total are already proven in
[`03-forge.md`](03-forge.md) (D1–D10, P2-5); this report references those IDs instead of re-proving them.
The unique contribution here is the **full visible-control census** (pixel -> component -> store -> command/route -> rendered
state), the dead-control list, the lying copy/counts, and a locked minimal fix queue.

---

## 0. Snapshot, scope and method

### 0.1 Source snapshot (hash-pinned)

The working tree moved while this audit was reading it: another seat edited `web/src/flow/Canvas.tsx`
(23:11:51, then 23:25:20) and `web/src/flow/graph.ts` (23:11:44). The line numbers below are pinned to the
hashes actually read. Re-pin before applying a fix.

| File | SHA-256 (at analysis) | Lines |
|---|---|---|
| `web/src/flow/Canvas.tsx` | `ACEE54C8630E7A3995B52262AE7012C49311A15B73C82C4072ADF1CAB05B0242` | 805 |
| `web/src/flow/graph.ts` | `600CF896E1E969C41DD16587D71D8A34F7F77E1A613FE5BE9FEC4FC4DD39F5B8` | 330 |
| `web/src/flow/HiveField.tsx` | `28204359E6C48EEF30264BA117A905DE4569B811CE25D8C3ECB3389369071792` | 114 |
| `web/src/panels/Header.tsx` | `8B32C46DF2914138B1A45738F31405D3C293150FAA2C5371939CD4D817999363` | 145 |
| `web/src/panels/LibraryPanel.tsx` | `0B274428EADDC4547D2F8742A2A80633C522CB0AA3DBAF2E07DCF4DAA36A3B94` | 193 |
| `web/src/panels/ControlBar.tsx` | `39B4FF73F92BEDD312E9A7A9BF36D9DDE101451EBAA3BBFDCCD87ABC847E96F2` | 127 |
| `web/src/chat/ChatCore.tsx` | `0C4E2DD746EA48E1639A8FCB1396371DC9BBF13B327C5853A7A02083315F49CB` | 367 |
| `web/src/panels/HudBadge.tsx` | `BE7FD1C62FF60CAA5BDC451E43E48F96AC4563CAA1B3A67333501EA02E7263B9` | 55 |
| `web/src/panels/Scrubber.tsx` | `3329167DD0A8CF929E5991959161E6934338E16165E838481BA7269E9D9ACAEB` | 76 |
| `web/src/panels/ApprovalOverlay.tsx` | `2E360B16AE719FE16126FFD52E239A09F2E0A733E47DE54A046F3CD969F20231` | 373 |
| `web/src/panels/AgentProfilesPanel.tsx` | `2ADA693528B3634F36BBAE874FD356DA1EC6E3AE5219D3C89709A658078A5A1A` | 191 |
| `web/src/panels/RoutingPanel.tsx` | `F0269705A18A93B587EB5C9B887737E4DEFE3F5C7453DCBEB14F06103FFEB1B9` | 39 |
| `web/src/state/store.ts` | `E4F163176B6E44D44831A8C5426F1F7A6AE26BE4922F40F75E43C4450A634C2E` | 2314 |
| `web/src/state/metrics.ts` | `1B264CDD370F04C99DE8FDC9D83DD2300A2A896987C03693C597D7645DA8FCC9` | 46 |
| `web/src/styles/index.css` | `E50B5376416DDF996922A9B2DBC9D61DBE2528759F96FD00E66B656AA8E81A50` | 360 |

Base commit: `240fe0f` (`chore: checkpoint hierarchy fixture and hide-finished layout work`). Uncommitted at
snapshot: `web/src/flow/Canvas.tsx`, `web/src/flow/graph.ts`, `tools/desktop-launcher/HiveMind.exe`.
The served bundle is **older than the current source** (FORGE §0.2): e.g. live `Hide finished` booted
`aria-pressed=false`, while current source defaults `hideFinished` to `true` at `Canvas.tsx:112`.
Live evidence is therefore labelled **served-bundle** where Canvas behavior is involved; source claims are
labelled **source**. Do not conflate them.

### 0.2 Scope and non-actions

- Every visible control on: topbar, left rail and workspace tabs, canvas, chat/composer, control bar, scrubber,
  approval overlay, agents profiles/memory/messages, models, data, settings, routing, swarm, projects, toasts.
- Traced each through: component -> store action -> command or HTTP route -> store event -> rendered value.
- Live HTTP reads were made only against the already-running keyless server (`127.0.0.1:8099`), before it exited:
  `/health`, `/api/history/*`, `/api/agents/*`, `/api/projects`. No `/api/provider/*` request was made.
- A temporary headless Chrome was driven against the already-running UI for chat hide/show and DOM geometry.
  This is not a product edit and did not restart the server.
- **The server exited during the audit (this seat did not stop it)** (observed by both this seat and FORGE); it was not
  restarted. All live values below were captured before that exit.
- No product file was edited. No unit test was modified. No provider key was read or written.

### 0.3 Verdict vocabulary

| Verdict | Meaning |
|---|---|
| `WIRED-AND-WORKS` | Click/gesture reaches a real store action and a real command/route; the rendered result changes. |
| `WIRED-BUT-DEAD` | The handler runs, but the user-visible result is absent, unreachable, or a guaranteed refusal. |
| `MISSING-UI` | State/action exists but there is no control or no truthful empty/error state for it. |
| `LYING` | The rendered label/count/claim contradicts the source, the server, or the control's actual effect. |
| `(WEAK live)` | Code path is present but was not exercised against a running/keyed server in this audit. |

A control that renders is not a control that works. A green control is only green when every hop's true value
reaches the next hop and a live request/row proves it.

---

## 1. Evidence pack

### 1.1 Live server (captured before it exited)

```
GET /health -> 200
{
  "status": "degraded",
  "version": "1.2.0-dev",
  "schema": "hiverouter.v1",
  "checks": [
    {"name":"rustc","required":true,"ok":true},
    {"name":"node","required":false,"ok":true},
    {"name":"deepseek_api_key","required":false,"ok":false,
     "detail":"DEEPSEEK_API_KEY is not set; live prompts will refuse E_KEY_MISSING",
     "code":"E_KEY_MISSING"}
  ],
  "database": {"path":"C:\\Users\\Riley\\AppData\\Local\\HiveMind\\hiverouter.db","schema_version":10},
  "transport": {"last_seq":0,"graph_nodes":35,"allowed_origins":2},
  "prompt_assets": {"loaded":true,"error_code":null}
}
```

| Probe | Observed |
|---|---|
| `GET /api/history/runs` | 5 runs. Latest `01a1b931-941a-7171-a209-1c05777354f8`, `status=completed`, no error. |
| `GET .../runs/{latest}/revisions` | `latest_revision=7`, `revision_count=8`, revisions `0..7`; every row `running` except rev 7 `completed`. |
| `GET .../runs/{latest}/state?revision=7` | `revision=7`, `run_status=completed`, 4 nodes: **1 hyperagent (completed, role=planner)** + **3 workers (completed, role=worker)**. HTTP projection carries kind/role/label. |
| `GET .../runs/{latest}/messages` | 4 messages (1 user + 3 assistant), `truncated=false`. |
| `GET /api/agents/profiles` | **7 profiles**: `chatbot, code-reviewer, content-writer, data-analyst, planner, subagent, superagent`. |
| `GET /api/agents/memory` | 7 profiles, all `memories: []` (empty is real, not invented). |
| `GET /api/projects` | 1 project; `name` equals the UUID `01a1b85f-71f4-7e9f-9a79-22729491030f`. |

### 1.2 Browser DOM probes (served bundle, before the exit)

At 1440x900, with the app loaded and WebSocket `Connected`:

- Header: `Hide chat` (`aria-pressed=true`), search button, connection chip `Connected`, avatar `HM OPERATOR`.
- Left rail: `ACTIVE AGENTS` -> `NO ACTIVE AGENTS`.
- HUD: `HYPERAGENT · 0 SUPERAGENTS · 3 WORKERS · 100.0% GREEN`.
- Canvas chip: `4/4`.
- React Flow DOM: **3 `node-worker` buttons + 1 invisible `hiveanchor`**; `4/4` includes the hyperagent, which
  `buildNodes` never renders as a card.
- One worker card measured `[785,163,166,50]` and `document.elementFromPoint` at its center returned the
  **chat header** (`Hive AI Assistant ×`) - the card is painted under the chat.
- The minimap measured `[1164,535,220,150]`; it rendered a dotted rectangle with **0 visible node marks**.
- Chat panel measured `[668,156,400,480]`; chat hide removed it; the ChatCore reopen pill measured
  `[809,68,117,34]` but was **under the HUD** (`HudBadge` `top-3 z-20`, 44px tall).
- After two refusals, the header `Show chat` at `[1058,10,97,32]` returned a **toast** from
  `document.elementFromPoint` (toast region `fixed right-4 top-4 z-50`), so a real mouse click would hit the
  toast, not the header control.
- Programmatic `.click()` on the header `Show chat` **did restore** chat (`aria-pressed` flipped back to `true`),
  so the store state was not lost. The "forever" report is an affordance/occlusion/persistence defect, not a
  state-machine loss.

### 1.3 Screens (workspace artifacts)

- `docs/trace/orca-wave/shots/baseline-edge.png` - full page: HUD 3 workers/100%, left `NO ACTIVE AGENTS`,
  `4/4`, blank dotted minimap, a worker card behind the chat, disabled control bar on the terminal run,
  scrubber `8 OF 8` + `LIVE`.
- `docs/trace/orca-wave/shots/TD-01-baseline.png` - same state in a real Chrome window; the minimap is a blank
  dot grid and the left Active Agents contradicts the HUD.
- Temporary captures made by this seat (not persisted in the repo): header/chat hide/show, hidden-state DOM,
  node rects, minimap rect, chat reopen rect. Values are quoted above.

---

## 2. The five screenshot-reported issues

### F1 — "grid icon corruption"

There are **three different grids** in the screenshot; only two are defects. Naming them precisely prevents a
wrong fix.

| Surface | What it is | Evidence | Verdict |
|---|---|---|---|
| Bottom-right minimap | A 220x150 React Flow `MiniMap` filled with a CSS dot grid; no node marks | `Canvas.tsx:691-718`; `index.css:130-155`; DOM rect `[1164,535,220,150]`; FORGE measured minimap nodes = 0 with 4 tasks | **WIRED-BUT-DEAD / LYING** - it renders like a corrupted grid icon because it is an empty grid with a stray viewport outline. |
| Canvas background honeycomb | A single 48x42 hexagon path repeated on a rectangular tile | `HiveField.tsx:39-46`, painted at `HiveField.tsx:99` | **VISUAL DEFECT** - a single hexagon on a 48x42 tile is not a honeycomb tessellation; the screenshot shows open chevrons/double lines. |
| Left-rail Workspace glyph | Two nested hexagons inside the active black square | `NavIcon.tsx:9-11`; 8x crop shows clean nested hexagons | **NOT CORRUPT** - it is an intentional nested-hexagon mark. Do not "fix" it as an image; at most simplify for 22px legibility. |

Minimal fix for the real grid defects:

1. **Minimap:** remove the cosmetic dot grid and do not show an empty minimap. Gate `MiniMap` on a proven
   drawable-node count, or remove it (zoom/fit/recentre already cover navigation). If kept, add a test that
   asserts a non-zero minimap node mark for a 4-task graph. The current `.hive-minimap-empty` only handles
   `visibleTasks.length === 0`, not "4 tasks but 0 drawable marks".
2. **Honeycomb field:** replace the single-hexagon tile with a staggered two-hexagon tile (or a proven hex
   tessellation), then re-capture the background. This is a 10-line SVG/CSS change, not a canvas rewrite.
3. **Rail glyph:** leave as-is unless the screenshot specifically points at it; it is legible at 8x.

### F2 — "Show chat removes chat forever"

**Live result: the state toggle works.** Clicking the header `Hide chat` hid the panel; clicking the header
`Show chat` restored it (`aria-pressed` true -> false -> true, chat section present -> absent -> present).
The store writes `hivemind.chat.visible.v1` to `localStorage` (`store.ts:2200-2207`) and reads it at boot
(`store.ts:395-402`), so a hidden state also survives a reload.

The "forever" perception is produced by three occlusions, not by a lost state:

1. **In-canvas reopen pill is under the HUD.** `ChatCore.tsx:112-124` renders the pill at `top-4` with no
   z-index; `HudBadge.tsx:14` is `top-3 z-20` and is 44px tall. Measured pill `[809,68,117,34]` sits inside the
   HUD rectangle; the screenshot shows the HUD, not the pill.
2. **Header toggle is under refusal toasts.** `ToastRegion.tsx:16` is `fixed right-4 top-4 z-50`, width up to
   440px. The header `Show chat` is at `[1058,10,97,32]`; with two refusals the hit-test returns the toast.
   The keyless app routinely produces `E_KEY_MISSING`/`E_RUN_TERMINAL`/`E_RESYNC_REQUIRED` toasts, so the
   one obvious recovery control is covered exactly when the user needs it.
3. **The toggle is global but the chat is workspace-only.** `Header.tsx:50-67` is rendered on every nav;
   `App.tsx:89-102` renders `ChatCore` only inside the workspace view. On Agents/Data/Models/Routing/Settings
   (and Swarm/Routing/History tabs), clicking `Show chat` changes state with no visible chat. This is a
   contextual dead control.

Verdict: `WIRED-AND-WORKS` for the state toggle; `WIRED-BUT-DEAD (in-canvas reopen)` and
`LYING (persisted hidden + no visible recovery)` for the interaction. "Forever" is an affordance defect, not
an irreversible state.

Minimal fix: move the ChatCore reopen pill below the HUD (`top-20 z-30`), move the toast region below the
header (`top-[60px]`) or raise the header above it, and either scope the header toggle to workspace or make
"Show chat" switch to the workspace tab. Optionally store `chat.visible` in `sessionStorage` so a reload starts
visible; if persistence is desired, add a permanent header-level recovery affordance not covered by toasts.

### F3 — "left Active Agents lies"

The left column says `ACTIVE AGENTS / NO ACTIVE AGENTS` while the same screen's HUD says
`3 WORKERS · 100.0% GREEN` (`shots/baseline-edge.png`). Both read `state.tasks`, but with different predicates:

| Source | Predicate | Consequence |
|---|---|---|
| `LibraryPanel.tsx:40-42` | `task.kind !== 'hyperagent' && !isTerminal(task.state)` | Excludes the HyperAgent root (even while it is the only active task), includes `queued` workers (not active), includes `tool` nodes as "agents". |
| `HudBadge.tsx:42-47` + `metrics.ts:41-45` | all tasks; `workers = worker + tool`; `superagents = superagent` only | Displays 3 workers and 100% green; neither count is "active". |
| `LibraryPanel.tsx:106` | `active.slice(0, 8)` | Silently drops the 9th+ active agent; no count and no "+N more". |

So the panel is wrong in both directions: it can show "No active agents" while a run is active and only the
HyperAgent is running, and it can show a queued worker under "ACTIVE". In the captured run all tasks are
completed, which is why it shows none while the HUD still shows the completed workers. The resulting screen is
a lie from the operator's point of view.

In replay, the contradiction gets worse: `Canvas` renders `replayTasks` (`Canvas.tsx:429-432,650-651`) while
`LibraryPanel` continues to read live `tasks` (`LibraryPanel.tsx:33`). A revision with 3 queued workers can be
on the canvas while the left column describes the live completed run.

Minimal fix: add one exported census selector (e.g. `selectAgentCensus(tasks)`) that returns
`{ hyperagent, superagents, workers, tools, errors, running, queued, completed, total }`; use it in
`LibraryPanel`, `HudBadge`, `SwarmPanel` and the Canvas count. Label the left block `ACTIVE / TOTAL` (or
`RUNNING`), include the HyperAgent in the total, count queued separately, show `+N` instead of silently
truncating, and pass `replayTasks` when `history.replayGraph !== null`. Tests: one selector test with
hyperagent-only active, queued worker, tool, and 9 running agents.

### F4 — dead tabs / dead commands

No nav destination is a no-op: the six rail buttons and four workspace tabs all reach a real panel
(`App.tsx:50-102`, `NavRail.tsx:37-68`, `LibraryPanel.tsx:79-98`). The dead controls are contextual or
state-gated:

| Dead / blind control | Why | File:line | Verdict |
|---|---|---|---|
| Header `Hide/Show chat` on non-workspace nav | ChatCore is not rendered outside the workspace view | `Header.tsx:50-67`; `App.tsx:89-102` | `WIRED-BUT-DEAD (contextual)` |
| Canvas `Graph only` / `Show chat` on History tab | History renders Canvas but no ChatCore | `Canvas.tsx:640-647`; `App.tsx:56-79` | `WIRED-BUT-DEAD (contextual)` |
| Canvas `Hide finished` / `Focus branch` in replay | Filters feed `liveNodes`/`visibleTasks`, not `replayNodes`; the replay canvas ignores them | `Canvas.tsx:172-198,200-245,650-651` | `WIRED-BUT-DEAD (replay)` |
| Canvas `4/4` count in replay | Counts live `visibleTasks` while the canvas shows replay tasks | `Canvas.tsx:637-639` | `LYING` |
| `Target` select in ControlBar | Changes the **selected task**, not the re-route target; the actual target is the next select | `ControlBar.tsx:42-56` vs `70-82` | `LYING (label) / WIRED-AND-WORKS (as selection)` |
| `Stop` in ChatCore during `planning`/`thinking` | Disabled unless `streamingRunId !== null || runStatus === 'running'`; `cancelRun` supports any active run; ControlBar `Cancel` is enabled | `ChatCore.tsx:339-351` vs `ControlBar.tsx:33-38` | `WIRED-BUT-DEAD (state predicate)` |
| Search result for a finished/hidden agent | `hideFinished` defaults true; selection changes but the card stays filtered; no focus/scroll | `Header.tsx:18-20,129-132`; `Canvas.tsx:112,172-198` | `WIRED-BUT-DEAD (no visible result)` |
| Profile `Delete imported profile` on `subagent` / `superagent` | Live API returns 7 profiles; `BUILTIN_IDS` contains only 5, so the two embedded profiles get a Delete button; backend `delete_agent_profile` returns 404 for embedded rows | `AgentProfilesPanel.tsx:23,156-164`; `http.rs:879-913` | `LYING / WIRED-BUT-DEAD` |
| Approval `Dismiss` on a TTL-expired approval | The Dismiss button is rendered only inside `resolveState?.refusal`; client expiry produces `actions.dismiss=true` with `refusal=null` | `ApprovalOverlay.tsx:312-341` (button at 330); `approval.ts:254-266` | `MISSING-UI (soft-lock)` |
| `pinChat`, diff modal | Store actions exist but no component calls them | `store.ts:2124-2129,2209-2215`; no caller | `MISSING-UI (dead code)` |
| `DerivedBadge`, `ReplayBanner`, `healthVersion`, `fileTree.refusal`, `history.refreshRefusal` | Stored/exported but never rendered | `StatePack.tsx:34-51`; `store.ts:224,233,161` | `MISSING-UI (unused state)` |

The data/history refresh paths are not dead, but a failed history read is rendered as **empty** rather than
**error** (see F5/§3.6), which is the same operator-facing outcome as a dead tab.

### F5 — misleading counts

| Where | Rendered | Truth | File:line | Verdict |
|---|---|---|---|---|
| Left Active Agents | `NO ACTIVE AGENTS` | 3 workers exist (completed); HUD says 3 workers; panel excludes hyperagent, includes queued, caps at 8 | `LibraryPanel.tsx:40-42,101-132` | `LYING` |
| HUD | `0 SUPERAGENTS · 3 WORKERS · 100.0% GREEN` | Derived from graph, not server metrics; no "derived" label; `hyperagent` counted in neither display count; tools counted as workers | `HudBadge.tsx:26-48`; `metrics.ts:41-45` | `LYING (unlabeled derived) / misleading` |
| HUD source | no text | `deriveHud` returns `source:'derived'` when `metrics===null`; `DerivedBadge` exists but is unused | `metrics.ts:24-46`; `StatePack.tsx:42-51` | `MISSING-UI` |
| Canvas count | `4/4` | 3 cards render; hyperagent is represented by the chat overlay and is filtered out by `buildNodes` | `Canvas.tsx:637-639`; `graph.ts:219-233` | `LYING` |
| Swarm `SUB-HIVES` | `0` for latest run | The graph has 1 `hyperagent` root; count only `kind==='superagent'` | `SwarmPanel.tsx:21-27,38-51` | `LYING` |
| Swarm `ACTIVE` | includes tools and errors | "Active" is `!isTerminal`, not `isRunActive`; no running/queued split | `SwarmPanel.tsx:26` | `misleading` |
| Left project | `Project: HiveMind AI` | `baseProject` hardcodes `STRINGS.library.projectFallback`; server project name is the UUID | `LibraryPanel.tsx:52`; `store.ts:336-363`; live `/api/projects` | `LYING` |
| Topbar operator | `OPERATOR` / `HM` | `project.owner` is hardcoded `STRINGS.chat.userLabel` ('You'), then mapped to 'Operator'; no auth/identity | `Header.tsx:93-100`; `store.ts:362-363` | `LYING` |
| Projects `CURRENT` | may not mark the local project | `currentProjectId` is a local UUID; the server project in this browser profile was a different UUID | `ProjectsPanel.tsx:61-77`; `store.ts:336-360` | `misleading` |
| Routing `role` column | displays state label | Header says `role`, cell renders `visualFor(task.state).label`; the server `task.role` is never shown | `RoutingPanel.tsx:20,31` | `LYING` |
| Models copy | "the account data below is live" | Shown before/without a successful provider read; provider error can render below it | `ModelsPanel.tsx:160-164,210-213` | `LYING` |
| Library tip | "re-route its context stream live" | The connect confirmation itself says memory/tool/message scope changes are **not applied yet** | `strings.ts:122`; `LibraryPanel.tsx:158`; `Canvas.tsx:583-586` | `LYING` |
| History revisions | `8 revisions` | True: API returned `revision_count=8`, `latest_revision=7` | `HistoryPanel.tsx:26-29` | `WIRED-AND-WORKS` |

---

## 3. Control matrix (every visible control)

Each row is one visible control. Hop 1 is the rendered component; hop 2 is the store/state action; hop 3 is the
transport, route, or view state. Line numbers are pinned to the hashes in §0.1.

### 3.1 Topbar (`Header.tsx`)

| Control | Hop 1 -> Hop 2 -> Hop 3 | Verdict | Actual / defect |
|---|---|---|---|
| Return live | `Header.tsx:41-49` -> `exitHistoryReplay` `store.ts:2161-2175` -> local `history.replayGraph=null` | `WIRED-AND-WORKS` | Visible only while replaying; clears replay and returns to live graph. Not exercised live (server exited). |
| Hide / Show chat | `Header.tsx:50-67` -> `setChatVisible` `store.ts:2200-2207` -> `chat.visible` + `localStorage['hivemind.chat.visible.v1']` | `WIRED-AND-WORKS` (live) | Live hide/show restored correctly. But on non-workspace nav the chat is not rendered; and the button is covered by toasts when refusals exist. See F2. |
| Search open | `Header.tsx:68-79` -> local `searchOpen` | `WIRED-AND-WORKS` | Opens the agent popover; `aria-expanded` updates. No `aria-controls`; no outside-click close. |
| Search query/results | `Header.tsx:18-20,102-141` -> `selectTask` `store.ts:2196-2198` -> local `selectedTaskId` | `WIRED-AND-WORKS / LYING (hidden selection)` | Filters all `tasks` by label, max 6. Selecting a finished agent while `hideFinished=true` changes selection but leaves no visible card. |
| Connection chip | `Header.tsx:80-92` -> `state.connection` `store.ts:1799-1807,1825-1829` -> WS status | `WIRED-AND-WORKS` | Live `Connected`; boot `Connecting`; resync sets `Reconnecting`. Does not claim connected while `resyncGap` is set. |
| Operator / avatar | `Header.tsx:93-100` -> `state.project.owner` `store.ts:336-364` | `LYING` | `baseProject` hardcodes owner `'You'`; the header maps that to `OPERATOR` and `HM`. There is no auth or server identity; the project name is also a local fallback. |
| Brand mark | `Header.tsx:24-38` | not a control | Static hexagon wordmark; no handler expected. |

### 3.2 Left rail and workspace tabs (`NavRail.tsx`, `LibraryPanel.tsx`, `App.tsx`)

| Control | Hop 1 -> Hop 2 -> Hop 3 | Verdict | Actual / defect |
|---|---|---|---|
| Workspace | `NavRail.tsx:9-16,37-68` -> `setNav('workspace')` `store.ts:2181-2186` -> `App.tsx:89-102` | `WIRED-AND-WORKS` | Renders the current workspace tab. |
| Agents | same -> `setNav('agents')` -> `AgentsPanel` `App.tsx:90-91` | `WIRED-AND-WORKS` | Renders cards, hierarchy, profiles, memory, mailbox. |
| Data | same -> `setNav('data')` -> `DataPanel`; also fires `requestFileTree` on idle `store.ts:2183-2185` | `WIRED-AND-WORKS` | Loads FILE_TREE_LIST; error has a named toast. |
| Models | same -> `setNav('models')` -> `ModelsPanel` | `WIRED-AND-WORKS` (side effect) | Opening the tab fires `GET /api/provider/models` + `/balance` (`ModelsPanel.tsx:62-103`); with a key this is a real provider call on tab open. Not exercised (keyless). |
| Routing | same -> `setNav('routing')` -> `RoutingPanel` | `WIRED-AND-WORKS` | Table is derived from live `tasks`. The `role` column lies (see §3.9). |
| Settings | same -> `setNav('settings')` -> `SettingsPanel` | `WIRED-AND-WORKS` | Probes SETTINGS_STATUS + `/health` on mount. |
| Workspace tab | `LibraryPanel.tsx:15-20,79-98` -> `setTab('workspace')` `store.ts:2188-2190` -> `App.tsx:82-87` | `WIRED-AND-WORKS` | Canvas + ChatCore + ControlBar + Scrubber. |
| Swarm tab | same -> `setTab('swarm')` -> `App.tsx:50-53` -> `SwarmPanel` | `WIRED-AND-WORKS` | Census + hierarchy + Projects. Counts are misleading (F5). |
| Routing tab | same -> `setTab('routing')` -> `App.tsx:53-55` -> `RoutingPanel` | `WIRED-AND-WORKS` | Same component as the rail destination; no dead tab. |
| History tab | same -> `setTab('history')` -> `App.tsx:55-81` -> `HistoryPanel` + `Canvas` + `Scrubber` | `WIRED-AND-WORKS` | Revisions are real; replay/filters/HUD are not coherent (F4/F5). |
| Project row | `LibraryPanel.tsx:50-54` -> `state.project` `store.ts:336-364` | `LYING` | Always `Project: HiveMind AI` (local fallback). Server project name is the UUID; no project switch exists. |
| Active Agents | `LibraryPanel.tsx:33,40-42,100-132` -> live `state.tasks` only | `LYING` | Excludes hyperagent, includes queued/tools, silently caps at 8, and uses live tasks in replay. See F3. |
| Template buttons (4) | `LibraryPanel.tsx:135-154` -> `insertTemplate` `store.ts:2131-2136` -> `TEMPLATE_INSERT` WS | `WIRED-AND-WORKS (code) / WEAK live` | Real command; no pending/disabled state, so repeated clicks can duplicate; no ACK-visible feedback. |
| State & settings disclosure | `LibraryPanel.tsx:161-190` | `WIRED-AND-WORKS` | Read-only status; settings may be `unknown` until Settings is opened because the panel does not probe. |
| Library tip | `LibraryPanel.tsx:158` + `strings.ts:122` | `LYING` | Says it re-routes the "context stream live"; the connect confirmation says memory/tool/message scope is not applied. |

### 3.3 Canvas controls (`Canvas.tsx`, `graph.ts`, `HiveField.tsx`, `nodes.tsx`)

| Control | Hop 1 -> Hop 2 -> Hop 3 | Verdict | Actual / defect |
|---|---|---|---|
| Hide finished | `Canvas.tsx:608-621` -> local `hideFinished` -> `visibleTasks` `Canvas.tsx:172-198` -> `buildNodes`/`buildEdges` | `WIRED-AND-WORKS (live) / DEAD-IN-REPLAY` | Filters live tasks. In replay the rendered nodes come from `replayNodes` (`Canvas.tsx:650-651`), so the button changes nothing on the canvas. Default `true` + terminal run = zero cards with no message (FORGE D2). |
| Focus branch | `Canvas.tsx:622-636` -> local `focusBranch` -> same filter | `WIRED-AND-WORKS (live) / DEAD-IN-REPLAY` | Works when a live task is selected; disabled without selection. Replay ignores it. |
| `N/N` count | `Canvas.tsx:637-639` -> `Object.keys(visibleTasks).length` / `Object.keys(tasks).length` | `LYING` | Counts the hyperagent even though `buildNodes` never renders it (`graph.ts:219-233`); live `4/4` = 3 cards + 1 invisible root. In replay it counts live tasks. |
| Graph only / Show chat | `Canvas.tsx:640-647` -> `setChatVisible` `store.ts:2200-2207` | `WIRED-AND-WORKS` (live) | Toggles the chat; label flips. On History tab there is no ChatCore, so it toggles state with no visible chat. |
| Zoom in | `Canvas.tsx:729-742` -> React Flow `zoomIn` -> viewport | `WIRED-AND-WORKS` | Live label moved with the viewport (52% measured); min/max 0.1/3.0 on the flow. |
| Zoom out | `Canvas.tsx:743-754` -> `zoomOut` | `WIRED-AND-WORKS` | Same. |
| Recentre | `Canvas.tsx:755-768` -> `setCenter(0,0,{zoom:getZoom()})` | `WIRED-AND-WORKS` (code) | Keyboard Ctrl+0 has a document-capture handler (`Canvas.tsx:382-409`); not exercised live. |
| Fit swarm | `Canvas.tsx:769-780` -> `fitView({duration,padding:0.2})` | `WIRED-AND-WORKS` (code) | A fit that can fit: unlike the auto-fit, this call has no min/max clamp. |
| Minimap | `Canvas.tsx:691-718` + `index.css:130-155` | `WIRED-BUT-DEAD / LYING` | 220x150 dotted rectangle; measured 0 visible node marks with 4 tasks. The dot-grid cosmetic + viewport outline reads as corrupted "grid icon". See F1. |
| Node cards | `Canvas.tsx:661-665` -> `selectTask` `store.ts:2196-2198` | `WIRED-AND-WORKS` (live) | Card click selects. One measured card was painted under the 400x480 chat panel (FORGE D9); selection is reachable only via search or after hiding chat. |
| Pending reparent Cancel / Connect | `Canvas.tsx:577-606` -> local state + `rewireSelected` `store.ts:1895-1948` -> `REWIRE_TASK` | `WIRED-AND-WORKS (code) / WEAK live` | Dialog is honest that memory/tool/message scope is not applied. Drag offset is persisted before confirmation (FORGE D3). |
| Filtered-empty message | `Canvas.tsx:783-789` | `MISSING-UI` | "No agents match the current filters." is gated on `!chatVisible`. With chat visible (default) and all tasks filtered, the canvas is blank and silent. |
| Replay banner | `Canvas.tsx:721-727` | `WIRED-AND-WORKS` | Shows `Replay (read-only) · Routing state N`. The HUD above it still shows live counts (F5). |

### 3.4 Chat and composer (`ChatCore.tsx`)

| Control | Hop 1 -> Hop 2 -> Hop 3 | Verdict | Actual / defect |
|---|---|---|---|
| Close `×` | `ChatCore.tsx:200-207` -> `setChatVisible(false)` | `WIRED-AND-WORKS` (live) | Hides the chat; state persists. The recovery affordances are occluded (F2). |
| In-canvas Show chat pill | `ChatCore.tsx:112-124` -> `setChatVisible(true)` | `WIRED-BUT-DEAD (visually)` | Rendered at `top-4` under `HudBadge top-3 z-20`; the user sees the HUD, not the pill. Hit-testing still reaches the button because the HUD is pointer-events-none. |
| Composer textarea | `ChatCore.tsx:323-338` -> local draft -> `submitPrompt` | `WIRED-AND-WORKS` | Disabled when disconnected/replaying; validation lives in `submitPrompt` (`store.ts:1856-1868`). |
| Submit prompt | `ChatCore.tsx:339-361` -> `submitPrompt` `store.ts:1856-1868` -> `SUBMIT_PROMPT` | `WIRED-AND-WORKS (code) / WEAK provider` | Keyless live path produced the real `E_KEY_MISSING` refusal and kept/restored the draft; no provider run was made. |
| Stop | `ChatCore.tsx:339-351` -> `cancelRun` `store.ts:1950-1957` -> `CANCEL_RUN` | `WIRED-BUT-DEAD (planning/thinking)` | Disabled unless `streamingRunId !== null || runStatus === 'running'`; ControlBar Cancel uses `isRunActive`, so the two controls disagree for planning/thinking/queued. |
| Prompt starters (3) | `ChatCore.tsx:232-246` -> local `setDraft` + focus | `WIRED-AND-WORKS` | Never auto-submits; only rendered when `messages.length === 0`. |
| Transcript | `ChatCore.tsx:223-317` -> `state.messages` from history/replies | `WIRED-AND-WORKS` (live) | Live reload showed the persisted 4-message transcript. Assistant text is rendered verbatim, so markdown `**...**` remains literal (presentation, not wiring). |
| Run refusal banner | `ChatCore.tsx:211-221` -> `run.errorCode` -> `clientRefusal` | `WIRED-AND-WORKS` | Named code + how-to-fix; duplicates the toast by design. |
| Tool-activity strip | `ChatCore.tsx:274-308` -> `state.toolActivity` | `WIRED-AND-WORKS (code)` | Only server TOOL_CALL/TOOL_RESULT rows; no live tool data in the captured run. |

### 3.5 Control bar (`ControlBar.tsx`)

| Control | Hop 1 -> Hop 2 -> Hop 3 | Verdict | Actual / defect |
|---|---|---|---|
| `Target` select | `ControlBar.tsx:42-56` -> `selectTask` -> local `selectedTaskId` | `LYING` | It selects the task to act on, not the re-route target. The actual target is the next select. |
| Spawn sub-hive | `ControlBar.tsx:58-68` -> `spawnSubHive` `store.ts:1871-1893` -> `SPAWN_SUB_HIVE` | `WIRED-AND-WORKS (code) / WEAK live` | Disabled on terminal/no run; parent is selected task or first hyperagent. No visible parent preview. |
| Re-route target select | `ControlBar.tsx:70-82` -> local `target` | `WIRED-AND-WORKS / state bug` | Lists super/hyper parents but not the worker's current parent; `target` is not reset when the selected task changes, so a new selection can be rewired to a stale target. |
| Re-route | `ControlBar.tsx:83-95` -> `rewireSelected(target)` `store.ts:1895-1948` -> `REWIRE_TASK` with `expected_rev` | `WIRED-AND-WORKS (code) / WEAK live` | Disabled unless a worker + target is chosen. Store fetches a committed rev if unknown and refuses rather than guessing. |
| Cancel | `ControlBar.tsx:97-106` -> `cancelRun` `store.ts:1950-1957` -> `CANCEL_RUN` | `WIRED-AND-WORKS (code) / WEAK live` | Correctly disabled on terminal runs. No confirmation for a destructive run-ending action. |
| Kill worker | `ControlBar.tsx:107-116` -> `killWorker` `store.ts:1959-1971` -> `KILL_WORKER` | `WIRED-AND-WORKS (code) / WEAK live` | Only selects `kind==='worker'`; the store also permits `tool`, so a tool cannot be killed from the UI. No confirmation. |
| Status text | `ControlBar.tsx:118-124` | `WIRED-AND-WORKS` | Shows pending / run id + terminal `Unavailable` / `Empty`; live showed `01A1B931 · UNAVAILABLE`. |

### 3.6 Scrubber and history (`Scrubber.tsx`, `HistoryPanel.tsx`)

| Control | Hop 1 -> Hop 2 -> Hop 3 | Verdict | Actual / defect |
|---|---|---|---|
| Revision range | `Scrubber.tsx:46-60` -> local value on input, `commit` on pointer/key up -> `setHistoryRevision` `store.ts:2138-2159` -> `HISTORY_REVISION_SET` | `WIRED-AND-WORKS (code/live 8 of 8)` | Commits only on release; `aria-valuetext` matches. On error the range is disabled and presented as "No run history yet". |
| LIVE button | `Scrubber.tsx:66-73` -> `exitHistoryReplay` `store.ts:2161-2175` | `WIRED-AND-WORKS` | Disabled while already live; explicit exit exists. |
| Position text | `Scrubber.tsx:61-65` | `WIRED-AND-WORKS` | `8 OF 8`; adds `Replay ·` while replaying. |
| History revision buttons | `HistoryPanel.tsx:30-56` -> `setHistoryRevision` -> WS | `WIRED-AND-WORKS` (code) | Real revision list; latest badge; no live click captured. |
| History LIVE button | `HistoryPanel.tsx:57-65` -> `exitHistoryReplay` | `WIRED-AND-WORKS` | Duplicate exit control; disabled unless replaying. |
| History empty state | `HistoryPanel.tsx:20-24` + `store.ts:160-161,1023-1024` | `MISSING-UI / LYING` | `refreshStatus='error'` is rendered as "No committed revisions yet"; `refreshRefusal` is never read. A failed read is not "none". |
| Scrubber empty state | `Scrubber.tsx:30,61-64` | `MISSING-UI / LYING` | Same: error -> `available=false` -> "No run history yet"; no retry. |
| Toast dismiss | `ToastRegion.tsx:36-43` -> `dismissRefusal` `store.ts:2217-2219` | `WIRED-AND-WORKS` | Removes one refusal; bounded list max 4. Toast region covers the header (F2). |

### 3.7 Approvals (`ApprovalOverlay.tsx`, `approval.ts`)

| Control | Hop 1 -> Hop 2 -> Hop 3 | Verdict | Actual / defect |
|---|---|---|---|
| Approve | `ApprovalOverlay.tsx:343-366` -> `resolveApproval` `store.ts:1973-2070` -> `APPROVAL_RESOLVE` with nonce | `WIRED-AND-WORKS (code) / WEAK live` | Enabled only for pending, non-expired, connected, nonce-present; in-flight guard. No live approval in the captured run. |
| Deny | same with `decision:'deny'` + `reason:'operator denied'` | `WIRED-AND-WORKS (code) / WEAK live` | Deny and approve are distinct decisions on the wire. |
| Retry | `ApprovalOverlay.tsx:319-328` -> `retryApproval` `store.ts:2072-2098` -> `resolveApproval` | `WIRED-AND-WORKS (code)` | Retry gate (`canRetryApproval`) requires status retryable, nonce, open socket, live TTL, attempts < 3. |
| Dismiss | `ApprovalOverlay.tsx:312-341`, button only at `330-337` | `MISSING-UI (soft-lock)` | `approvalActionsEnabled` returns `dismiss:true` for `expired/stale/unconfirmed` (`approval.ts:254-266`), but the button is nested under `resolveState?.refusal`. A client-clock expiry with no refusal leaves a full-screen modal with no exit. |
| Stale banner | `ApprovalOverlay.tsx:212-220` -> `approvalsStale` | `WIRED-AND-WORKS` | Explicit "do not approve an unconfirmed card". |
| Dialog focus / Escape | `ApprovalOverlay.tsx:244-249` | `MISSING-UI (a11y)` | `role=dialog aria-modal` but no focus trap, no initial focus, no Escape handler. Background controls remain tabbable. |
| Multiple approvals | `ApprovalOverlay.tsx:221-370` maps one full-screen overlay per approval | `WIRED-BUT-STACKED` | Later overlay covers earlier; no queue/count; only the top card is actionable until resolved. |

### 3.8 Agents: profiles, memory, messages (`AgentsPanel.tsx` and panels)

| Control | Hop 1 -> Hop 2 -> Hop 3 | Verdict | Actual / defect |
|---|---|---|---|
| Agent cards | `AgentsPanel.tsx:40-64` -> `selectTask` | `WIRED-AND-WORKS` | One card per task; selected border. Count in title includes all tasks. |
| Hierarchy rows | `AgentHierarchy.tsx:55-80` -> `selectTask` | `WIRED-AND-WORKS` | Parent-chain depth; cycles/orphans re-rooted visibly. |
| Tool activity | `AgentsPanel.tsx:65-72` -> `ToolActivityPanel` -> `state.toolActivity` | `WIRED-AND-WORKS (code)` | Empty state is explicit; no fabricated rows. |
| Profiles list | `AgentProfilesPanel.tsx:40-82` -> `GET /api/agents/profiles` (`http.rs:522`) | `WIRED-AND-WORKS` (live 7) | Real server rows; loading/error/empty are explicit, though the HTTP status is not named. |
| Import profile | `AgentProfilesPanel.tsx:88-113,168-188` -> `POST /api/agents/profiles/import` (`http.rs:523`) | `WIRED-AND-WORKS (code)` | Server validates persona/template/tools; import status is text; no busy/disable. |
| Delete imported profile | `AgentProfilesPanel.tsx:115-127,156-164` -> `DELETE /api/agents/profiles/{id}` (`http.rs:524`) | `LYING / WIRED-BUT-DEAD` | `BUILTIN_IDS` (`:23`) has 5 ids; live has 7, so `subagent` and `superagent` get a Delete button. Backend deletes only imported rows and returns 404 for embedded; the button cannot succeed. |
| Memory list | `AgentMemoryPanel.tsx:24-73` -> `GET /api/agents/memory` (`http.rs:525`) | `WIRED-AND-WORKS` (live 7 empty) | Honest empty state; no rows invented. |
| Clear memory | `AgentMemoryPanel.tsx:75-84,105-111` -> `DELETE /api/agents/memory/{agent_key}` (`http.rs:526`) | `WIRED-AND-WORKS` | Immediate destructive action, no confirmation; failure collapses to generic `error` without the named refusal. |
| Messages list | `AgentMessagesPanel.tsx:21-84` -> `GET /api/agents/runs/{run}/messages` (`http.rs:521`) | `WIRED-AND-WORKS` (route/code) | Refreshes on `AGENT_MESSAGE` version (`store.ts:1514-1516`); real rows only. |
| Messages idle/empty | `AgentMessagesPanel.tsx:27-31,95-97` | `MISSING-UI` | When `runId===null`, status stays `idle` and the section renders **no** empty/loading/error text - just the heading. |

### 3.9 Models, data, settings, routing, swarm, projects

| Control | Hop 1 -> Hop 2 -> Hop 3 | Verdict | Actual / defect |
|---|---|---|---|
| Provider account Refresh | `ModelsPanel.tsx:62-103,199-206` -> `GET /api/provider/models` + `/balance` (`http.rs:529-530`) | `WIRED-AND-WORKS (code) / WEAK live` | Real provider proxy; keyless gives named `E_KEY_MISSING`; no provider call was made by this seat. |
| Usage panel | `ModelsPanel.tsx:105-155` -> `GET /api/usage/runs/{run}` (`http.rs:527`) | `WIRED-AND-WORKS (code)` | Null usage -> explicit "No token usage reported"; error -> "Usage unavailable; no token count invented." |
| Models copy | `ModelsPanel.tsx:160-164` | `LYING` | "the account data below is live" is claimed even while loading/error; the data is fetched, not continuously live. |
| Settings provider re-check | `SettingsPanel.tsx:39-47` -> `probeSettings` `store.ts:2225-2247` -> `SETTINGS_STATUS` | `WIRED-AND-WORKS` | Pending/error/refusal rendered; key present/missing from server. |
| Settings health re-check | `SettingsPanel.tsx:84-92` -> `probeHealth` `store.ts:2249-2269` -> `GET /health` | `WIRED-AND-WORKS` | Live keyless health `degraded` with the three check rows. |
| Reduced motion override | `SettingsPanel.tsx:106-114` -> `setReducedMotionOverride` `store.ts:2221-2223` -> `App.tsx:39-41` data attribute | `WIRED-AND-WORKS` | CSS honours the attribute; no separate persisted store. |
| Data Refresh root / error retry | `DataPanel.tsx:28-37,47-56` -> `requestFileTree` `store.ts:2271-2277` -> `FILE_TREE_LIST` | `WIRED-AND-WORKS` | Real retry path; error copy names Refresh and the button exists. |
| Data directory row | `DataPanel.tsx:61-78` -> `requestFileTree(entry.path)` | `WIRED-AND-WORKS` | Replaces the list with children; no breadcrumb/up control except Refresh root. |
| Data file row | `DataPanel.tsx:79-100` -> `openFile` `store.ts:2279-2296` -> `FILE_READ` | `WIRED-AND-WORKS / race` | Path jailed client-side; two rapid clicks can let an older `FILE_CONTENT` overwrite the newer selection (the event handler at `store.ts:1680-1689` has no request identity). |
| Routing table | `RoutingPanel.tsx:14-34` | `LYING` | `role` header at `:20` renders `visualFor(task.state).label` at `:31`; server `task.role` is not shown. No control here; the nav/tab is wired. |
| Swarm census | `SwarmPanel.tsx:21-27,38-51` | `LYING` | `SUB-HIVES` excludes the hyperagent root (latest run: 1 hyperagent, 0 superagents -> 0); `ACTIVE` includes tools/errors and has no running/queued split. |
| Swarm action buttons | `SwarmPanel.tsx:77-90,120-135` -> `setTab`/`setNav` | `WIRED-AND-WORKS` | Start Workspace, Open profiles & memory, Open agents, Open files all route. |
| Projects list | `ProjectsPanel.tsx:20-77` -> `GET /api/projects` (`http.rs:528`) | `WIRED-AND-WORKS (read-only)` | No create/rename/switch route exists, so the absence is honest. The `CURRENT` marker only matches after a run was submitted with this browser's local project id; in the captured profile the server project was a different UUID, so the row shows `committed`. The panel can go stale after a run because it fetches once. |
| Dev gallery | `SettingsPanel.tsx:117` -> `DevGallery.tsx:8-10` | not visible in prod | `import.meta.env.DEV` guard; production returns null. |

### 3.10 Store actions and state with no visible control

| Action/state | Location | Status |
|---|---|---|
| `pinChat` / `CHAT_PIN` | `store.ts:2124-2129`; `contract.ts:110` | No UI caller; dead action (the old unpin control was removed). |
| `openDiffModal` / `closeDiffModal` / `diffOpen` | `store.ts:2209-2215`; state `:287` | No modal and no caller; decision-gated, but the dead store surface should be removed or documented. |
| `DerivedBadge` | `StatePack.tsx:42-51` | Exported, never rendered; the HUD therefore has no source label. |
| `ReplayBanner` | `StatePack.tsx:34-39` | Exported, never rendered; Canvas has its own inline replay pill. |
| `healthVersion` | `store.ts:224`; set at `2256` | Never displayed; server version is lost to the operator. |
| `fileTree.refusal` | `store.ts:233` | Declared but never set or rendered; Data errors rely on toasts. |
| `history.refreshRefusal` | `store.ts:161,1023-1024` | Never rendered; failed history reads look like empty history. |
| `agentMessageVersion` | `store.ts:258,1514-1516` | Used as a refetch key; correct. |
| `avail/selected` history cursor fallback | `Scrubber.tsx:18-20` | If `history.selected` is a revision number not present in `revisions`, it is used as an index; clamp is not a semantic match. Minor. |

---

## 4. Locked fix queue (minimal changes)

Ordered by operator-facing severity. Each item is one seam; do not combine a backend fix into a UI PR or vice
versa. File:line is the pinned snapshot; re-grep by symbol if the line moved.

### 1. Restore an always-reachable chat recovery path (P0, UI)

- `ChatCore.tsx:112-124`: move the reopen pill out from under the HUD (`top-4` -> `top-20`, add `z-30`).
- `ToastRegion.tsx:16`: move the toast stack below the header (`top-[60px]`) or give the header a stacking
  context above `z-50`.
- `Header.tsx:50-67`: if `nav !== 'workspace'` and the user clicks Show, call `setNav('workspace')` (and keep the
  workspace tab) instead of changing invisible state. Optionally persist `chat.visible` in `sessionStorage`
  so a reload starts visible.
- Test: Playwright hides chat, injects two refusals, clicks the **visible** recovery control at 1440, asserts
  chat section exists; repeat at 1280 and 1024.

### 2. One agent census for every count (P0, UI selector)

- Add `selectAgentCensus(tasks)` (pure, unit-tested) returning `{ hyperagent, superagents, workers, tools,
  errors, running, queued, completed, total }`.
- Use it in `LibraryPanel.tsx:40-42,100-132` (label `ACTIVE / TOTAL`, show `+N` at the 8-chip cap, count queued
  separately, include hyperagent in the total), `HudBadge.tsx:42-47`, `SwarmPanel.tsx:21-51`, and the Canvas
  count `Canvas.tsx:637-639`.
- When `history.replayGraph !== null`, pass `replayTasks` to the selector (or disable the left blocks with a
  Replay label); do not mix live counts with replay pixels.
- Tests: hyperagent-only active -> "1 active"; 9 running -> `+1`; replay revision with 3 queued -> left and HUD
  both report 3 queued, not the live completed counts.

### 3. Label derived vs measured and make HUD counts add up (P0, UI)

- `HudBadge.tsx:26-48`: render `DerivedBadge` (or a text `derived`/`live` chip) from `hud.source`; do not show
  bare computed numbers.
- `metrics.ts:41-45`: count hyperagent in its own slot; keep `worker + tool` out of "WORKERS" or label the
  combination. For the live run `0 + 3` must not be presented as the whole 4-task swarm (FORGE P2-5).
- `Canvas.tsx:637-639`: count renderable cards (exclude hyperagent) and show the root separately, or display
  `4 agents / 3 cards + hive`.

### 4. Make replay surfaces agree (P0/P1, UI)

- `Canvas.tsx:172-198,200-245,650-651`: apply the Hide finished/Focus branch filters to `replayTasks` or disable
  both controls in replay. The current buttons are dead in replay.
- `Canvas.tsx:637-639,721-727`: count/format replay tasks and revision together; do not show live counts over a
  replayed graph.
- `LibraryPanel.tsx:33`: read `replayTasks`/a display-tasks selector while replaying.
- `HudBadge`: accept the display tasks as a prop from Canvas rather than reaching into `state.tasks`.
- Cross-reference FORGE D1/D6: replay `HISTORY_SNAPSHOT` lacks kind/role; the current replay projection
  flattens the hierarchy. Do not mark replay green until D1 is fixed.

### 5. Fix the two real "grid" defects (P1, visual)

- `HiveField.tsx:39-46,99`: replace the single 48x42 hexagon tile with a proven staggered two-hexagon tile
  (or CSS hex pattern); re-capture the background at 1x/2x. This is the broken open-chevron honeycomb.
- `Canvas.tsx:691-718` + `index.css:132-151`: do not render an empty dotted minimap. Either remove `MiniMap`
  until its node marks are proven, or gate it on a drawable-node count and remove the cosmetic dot grid. Add a
  test asserting node marks > 0 for the 4-task fixture.
- Leave `NavIcon.tsx:9-11` alone unless the screenshot specifically targets the rail glyph; the 8x crop is a
  clean nested hexagon, not a broken image.

### 6. Make history errors visible and retryable (P1, UI)

- `HistoryPanel.tsx:20-24` and `Scrubber.tsx:30,61-64`: branch on `history.refreshStatus === 'error'` and render
  `history.refreshRefusal.code/message` with a Retry that calls `refreshRunSnapshot('manual')`.
- Never present a failed read as "No committed revisions yet" / "No run history yet".
- Test: mock a 503 history read; assert the refusal code is visible and Retry re-issues the read.

### 7. Close the expired-approval soft-lock (P1, UI)

- `ApprovalOverlay.tsx:312-341`: render the `Dismiss` button in the footer whenever `actions.dismiss` is true,
  outside the `resolveState?.refusal` block. Keep the in-flight guard.
- Add an initial focus + Escape handler; Escape should only dismiss terminal statuses (`expired/stale/unconfirmed`),
  never a pending approval.
- Test: approval with `expires_at` in the past and `refusal=null`; assert the dialog has exactly one enabled
  Dismiss and Escape removes it.

### 8. Stop offering an impossible profile delete (P1, UI)

- `AgentProfilesPanel.tsx:23`: add `subagent` and `superagent` to `BUILTIN_IDS`, or better, have the API return
  `source`/`removable` and render Delete only for imported rows.
- Test: live/embedded profile list with 7 ids; assert 0 Delete buttons for the 5 embedded + 2 runtime built-ins
  and 1 for an imported row.

### 9. Align the control-bar and Stop state machine (P1, UI)

- `ControlBar.tsx:42-56`: rename `Target` to `Selected agent`; rename the second select's placeholder to
  `New parent`; reset `target` in a `useEffect` on `selectedTaskId`; exclude the selected task's current parent
  from `parents` (`ControlBar.tsx:29-31`).
- `ChatCore.tsx:343`: use `isRunActive(runStatus)` (or a shared `canCancel`) so Stop and Cancel agree during
  planning/thinking/queued.
- Add a confirmation step for Kill (and preferably Cancel) using the existing Canvas confirm pattern; do not use
  a browser `confirm` in the product.
- Tests: select worker A -> target P -> select worker B -> `target` is empty; planning run -> Stop enabled;
  terminal run -> both disabled.

### 10. Correct the remaining lying copy/identity (P2, copy)

- `Header.tsx:93-100` + `store.ts:336-364`: replace `OPERATOR`/`HM` with an honest local label (`Local operator`)
  and stop treating the hardcoded project fallback as a server identity.
- `LibraryPanel.tsx:52`: use the server project name (match `/api/projects` by id) or `Local project`; do not
  claim `HiveMind AI` when the server row is the UUID.
- `RoutingPanel.tsx:20,31`: add a real `role` cell (`task.role ?? '—'`) and a separate state cell, or rename the
  header to `state`.
- `SwarmPanel.tsx:38-51`: rename/count the hyperagent separately; do not show `SUB-HIVES 0` for a graph with a
  hyperagent root.
- `LibraryPanel.tsx:158` + `strings.ts:122`: change the tip to "re-route its parent edge; memory/tool/message
  scope is unchanged" (match the confirmation dialog).
- `ModelsPanel.tsx:160-164`: "account data is fetched from the provider on open/refresh" instead of "live".
- `Canvas.tsx:783-789`: drop `&& !chatVisible` (FORGE D2) so a filtered-empty canvas always explains itself;
  default `hideFinished` to `false` (`Canvas.tsx:112`) so a completed run is not born blank.

---

## 5. Contract/census parity

The HTTP route census is consistent: every direct `fetch` in the web client has a served route in
`backend/crates/hiverouter-core/src/http.rs:514-531`.

| Client call | Client file:line | Server route | Server line | Match |
|---|---|---|---|---|
| `/health` | `api/health.ts`; `store.ts:2251` | `GET /health` | `http.rs:514` | yes |
| `/api/history/runs` | `store.ts:442,1060` | `GET /api/history/runs` | `http.rs:517` | yes |
| `/api/history/runs/{id}/revisions` | `store.ts:444-446,1008-1011` | `GET .../revisions` | `http.rs:518` | yes |
| `/api/history/runs/{id}/state?revision=` | `store.ts:448-450,1088-1093` | `GET .../state` | `http.rs:519` | yes |
| `/api/history/runs/{id}/messages` | `store.ts:452-454,1076-1081` | `GET .../messages` | `http.rs:520` | yes |
| `/api/agents/runs/{id}/messages` | `AgentMessagesPanel.tsx:35` | `GET .../messages` | `http.rs:521` | yes |
| `/api/agents/profiles` | `AgentProfilesPanel.tsx:43` | `GET /api/agents/profiles` | `http.rs:522` | yes |
| `/api/agents/profiles/import` | `AgentProfilesPanel.tsx:97-101` | `POST .../import` | `http.rs:523` | yes |
| `/api/agents/profiles/{id}` | `AgentProfilesPanel.tsx:116` | `DELETE .../{id}` | `http.rs:524` | yes |
| `/api/agents/memory` | `AgentMemoryPanel.tsx:26` | `GET /api/agents/memory` | `http.rs:525` | yes |
| `/api/agents/memory/{key}` | `AgentMemoryPanel.tsx:76` | `DELETE .../{key}` | `http.rs:526` | yes |
| `/api/usage/runs/{id}` | `ModelsPanel.tsx:112` | `GET .../usage` | `http.rs:527` | yes |
| `/api/projects` | `ProjectsPanel.tsx:27` | `GET /api/projects` | `http.rs:528` | yes |
| `/api/provider/models` | `ModelsPanel.tsx:72` | `GET .../models` | `http.rs:529` | yes |
| `/api/provider/balance` | `ModelsPanel.tsx:73` | `GET .../balance` | `http.rs:530` | yes |

No frontend call targets a missing server route. The one guaranteed 404 outcome is the embedded-profile Delete
above: the route exists, but the row is not an imported profile. The defect is not missing routes; it is that
the direct-fetch panels bypass the central store's refusal/timeout state and report only a generic loading/error
string.

WS command parity (UI -> store -> contract): `SUBMIT_PROMPT`, `SPAWN_SUB_HIVE`, `REWIRE_TASK`, `CANCEL_RUN`,
`KILL_WORKER`, `APPROVAL_RESOLVE`, `SETTINGS_STATUS`, `FILE_TREE_LIST`, `FILE_READ`, `TEMPLATE_INSERT`,
`HISTORY_REVISION_SET` are all named in `contract.ts:106-131` and have UI call sites. `CHAT_PIN` is named but has
no UI caller (dead action, §3.10).

---

## 6. Verification plan (before any fix is called green)

1. Build the current working tree (`cd web; npm run build`) and load the **same** bundle that is being fixed.
   The served bundle was older than the source at audit time; do not re-use old screenshots as proof.
2. Playwright at 1440x900, 1280x900, 1024x800:
   - hide chat; inject 2 refusals; click the visible Show chat; assert chat exists and is focusable.
   - click Hide finished with a terminal run; assert cards remain by default and the empty message appears only
     when a filter actually removes all nodes.
   - open History, click a revision; assert HUD/left/count/Canvas all describe the same revision.
   - open Agents; assert 7 profiles and 0 Delete buttons on `subagent`/`superagent`.
   - open Models only with the key unset; assert the named `E_KEY_MISSING` row and no provider request in the
     network log; then (separately, keyed) confirm the provider call is explicit on open/refresh.
3. API tests (Rust): history revisions/state/messages, profiles import/delete built-in refusal, projects.
4. Unit tests: `selectAgentCensus`, replay filter, history error state, approval expired dismiss, control-bar
   target reset, `isRunActive` Stop predicate.
5. Re-run the stub sweep (contract §23) on every changed file; no product edits by this seat.

---

## 7. Not done / unverified (explicit)

| Item | Why it is unverified | What would prove it |
|---|---|---|
| Provider-keyed submit / streaming / cancel | Key absent; no provider call allowed; server exited during audit | One keyed run with Playwright request/response + DB rows; not claimed green. |
| Spawn / re-route / kill against a live run | Terminal run at capture; all three buttons disabled | A running fixture + WS frames `ACK`/`NODE_*` + graph row. |
| Approval approve/deny/retry | No pending approval in captured data | Inject an `APPROVAL_REQUEST` with a nonce and replay/approve/deny. |
| `Show chat` on every nav / after toast | Only header hide/show was live-tested; the occlusion geometry is measured | Playwright click-through at 1440/1280/1024 with refusals present. |
| Minimap root cause | Measured 0 marks; not traced into React Flow internals | DOM/SVG count test and a minimal reproduction; until then remove or gate it. |
| Honeycomb pattern | Screenshot/geometry proof; no browser pixel diff at multiple zooms | Before/after capture at 1x/2x; assert no open chevrons. |
| Replay HUD/filters mismatch | Source-only (server exited before a history click) | Click rev 0/7 and compare HUD/left/count to the replay nodes. |
| Agent profile Delete | Live list + backend route prove the 404 path; Delete not clicked | Click and capture the 404 toast on a disposable profile. |
| Expired approval dismiss | Source-only (no approval in capture) | Inject an expired approval; assert a Dismiss is present and Escape closes. |
| "Grid icon corruption" exact referent | The screenshot could mean minimap, field, or rail glyph; two are proven defects, one is not | Name the surface in the next screenshot; the fixes above cover all three without touching the intact glyph. |

---

## 8. Evidence appendix

### 8.1 Commands used (read-only, no secrets)

```
GET http://127.0.0.1:8099/health
GET http://127.0.0.1:8099/api/history/runs
GET http://127.0.0.1:8099/api/history/runs/{id}/revisions
GET http://127.0.0.1:8099/api/history/runs/{id}/state?revision=7
GET http://127.0.0.1:8099/api/history/runs/{id}/messages
GET http://127.0.0.1:8099/api/agents/profiles
GET http://127.0.0.1:8099/api/agents/memory
GET http://127.0.0.1:8099/api/projects
```

No `/api/provider/*`, no `/ws` mutation, no `cargo run`, no process restart, no file edit in `web/` or
`backend/`. The headless browser read the already-served page and clicked only Hide/Show chat.

### 8.2 Key file:line index

| Finding | File:line |
|---|---|
| Chat toggle | `Header.tsx:50-67`; `store.ts:2200-2207`; `ChatCore.tsx:112-124` |
| Toast/header/HUD occlusion | `ToastRegion.tsx:16`; `HudBadge.tsx:14`; `Header.tsx:50-67` |
| Active Agents | `LibraryPanel.tsx:40-42,100-132` |
| HUD derived counts | `HudBadge.tsx:26-48`; `metrics.ts:24-46` |
| Canvas count | `Canvas.tsx:637-639`; `graph.ts:219-233` |
| Replay filter gap | `Canvas.tsx:172-198,200-245,650-651` |
| Minimap | `Canvas.tsx:691-718`; `index.css:130-155` |
| Honeycomb pattern | `HiveField.tsx:39-46,99` |
| Hide-finished default/empty | `Canvas.tsx:112,172-198,783-789` (FORGE D2) |
| History error | `store.ts:160-161,1023-1024`; `HistoryPanel.tsx:20-24`; `Scrubber.tsx:30,61-64` |
| Approval dismiss | `ApprovalOverlay.tsx:312-341`; `approval.ts:254-266` |
| Profile delete lie | `AgentProfilesPanel.tsx:23,156-164`; `http.rs:879-913` |
| Control bar label/state | `ControlBar.tsx:29-31,42-56,70-82` |
| Stop predicate | `ChatCore.tsx:339-351`; `ControlBar.tsx:33-38` |
| Routing role lie | `RoutingPanel.tsx:20,31` |
| Swarm sub-hives | `SwarmPanel.tsx:21-27,38-51` |
| Project/operator | `store.ts:336-364`; `Header.tsx:93-100`; `LibraryPanel.tsx:52` |
| Models copy | `ModelsPanel.tsx:160-164` |
| Library tip | `strings.ts:122`; `LibraryPanel.tsx:158`; `Canvas.tsx:583-586` |

---

*End of SEAT B report. No product edits; no restart; no provider call.*


