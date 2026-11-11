# L5 — UI/UX Lane Report (every React/TypeScript surface)

**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **Lane:** all of `web/src/**` + `web/tests/**`
**Author:** Codebase Analyst Engineer 5/7 · **Date:** 2026-11-10 · **Mode:** read-only; this file is the only write.

**Evidence convention.** Every claim carries a repo-relative citation (`web/src/...:line[-line]`).
Tags: **VERIFIED** = read directly in the cited file (or a test that pins it); **UNVERIFIED** = not observable in this lane / not executed (no browser, no server, no provider, no test run); **PROPOSED** = additive recommendation, not present in code.
**What was not done:** no `npm test`/`vite build`, no browser render, no server start. Visual/material statements are code-derived, not screenshot-verified. Test claims are file-content claims unless quoted as a run.

---

## 1. Executive verdict

**The shell is real, not a mock.** Production entry wires one Zustand store to a WebSocket API, and the dev mock is compile-time gated and dynamically imported (`web/src/api/index.ts:32`, `web/src/api/index.ts:62-80`; boundary pinned by `web/tests/apiBoundary.test.ts`). Every mutation goes through a named command with ACK/refusal handling (`web/src/state/store.ts:1843-1876`), and no shipped branch fabricates run data. **VERIFIED.**

**It feels alive at the graph layer, and thin at the work layer.**
- Alive: spatial canvas with hive field, bee edges, node cards, HUD, minimap, pinned chat, live status dot, refusal toasts, approval modal (`web/src/flow/Canvas.tsx:443-748`, `web/src/chat/ChatCore.tsx:206-438`).
- Thin: the UI cannot show *what* any agent did with a tool. `TOOL_CALL.task_id` and `TOOL_RESULT.bytes/sha256` are on the wire (`web/src/api/contract.ts:295-313`) but the store drops all three (`web/src/state/store.ts:1659-1674`, `web/src/state/store.ts:1709-1715`), so tool rows are `tool + args_sha256 + outcome` with no owning agent and no result size (`web/src/panels/ToolActivityPanel.tsx:157-177`). **VERIFIED** (protocol + projection).
- Thin: **no surface renders the snapshot/rehydration state at all** — `snapshot` is written by the store and read only by the store (`web/src/state/store.ts:190-201`, `:1079-1307`; zero consumers in `web/src`). A degraded graph looks identical to a confirmed one. **VERIFIED.**

**Coherence gaps that read as "unfinished" to a user:**
1. Clicking the Workspace tabs (Swarm / Routing / History) while on any other nav section changes hidden state and does nothing visible (`web/src/panels/LibraryPanel.tsx:142` sets only `tab`; `web/src/App.tsx:148-163` chooses content by `nav`). **VERIFIED** dead end.
2. The Header chat toggle is enabled on every surface but the chat only exists in Workspace → Workspace (`web/src/panels/Header.tsx:50-67`; `web/src/App.tsx:141-146`). **VERIFIED** dead control.
3. On the live Agents surface every SPAWN button is disabled with the reason **"Replay is read-only. Exit replay to spawn a child."** because `AgentsPanel` mounts `AgentHierarchy` without `spawnTask` (`web/src/panels/AgentsPanel.tsx:43`; reason at `web/src/panels/AgentHierarchy.tsx:79-82`). **VERIFIED** — actively misleading copy on a live run.

**Design authority is respected.** HoneyGlass/FigCraft values live in one token module and its CSS mirror (`web/src/tokens/tokens.ts:22-43`, `web/src/styles/tokens.css:6-53`), with the forced WCAG corrections (meta text ink-500, focus ink-700) (`web/src/tokens/tokens.ts:72-79`). The splash is the fullest expression of the material (`web/src/splash/HiveMindSplash.tsx:67-142`, `web/src/styles/splash.css:13-114`). Additive improvements only — see §10.

**Overall:** a credible installed-app frame with strong honesty discipline (every failure is named and actionable) that currently under-reports the Hive's actual work and ships three navigation/capability lies. Fixing the top 10 (§9) plus four small projections (§10) would raise perceived completeness by roughly the target 10–20% without a redesign.

---

## 2. Architecture in one page

| Layer | File | What it does | Status |
|---|---|---|---|
| Entry | `web/src/main.tsx:1-50` | React 19 `StrictMode`, fonts, xyflow CSS, launch-token fetch shim for `/api/*` | **VERIFIED** |
| Root | `web/src/App.tsx:35-221` | one store per mount, hash router, nav/tab frames, overlay chrome, splash | **VERIFIED** |
| Store | `web/src/state/store.ts:796-2584` | single Zustand vanilla store; 30 actions; 24-event reducer | **VERIFIED** |
| Context | `web/src/state/context.ts:9-25` | provider + `useUiStore(selector)` | **VERIFIED** |
| Transport | `web/src/api/client.ts:196-666` | `/api/session` → WS, hello frame, ACK/ERROR correlation, reconnect | **VERIFIED** |
| Contract | `web/src/api/contract.ts:100-1160` | 14 frozen commands (12 over WS), 24 events, refusals, decoders, validators | **VERIFIED** |
| UI API seam | `web/src/api/index.ts:42-174` | dev-mock gate + lazy delegate that queues listeners | **VERIFIED** |
| Tokens | `web/src/tokens/tokens.ts`, `web/src/styles/tokens.css` | 20 swatches + chrome-only values, type/space/radius/elevation | **VERIFIED** |
| Styles | `web/src/styles/index.css:1-435`, `motion.css:1-501`, `splash.css:1-340` | utilities, ambient/legacy motion, splash material | **VERIFIED** |
| Tests | `web/vite.config.ts:26-29` (`environment: 'node'`, `tests/**/*.test.ts`) + 31 specs | pure logic, store, static `renderToStaticMarkup` | **VERIFIED** |

**Navigation map (as built).**

```
#/<nav>/<tab>   nav ∈ workspace agents data models routing remote settings
                tab ∈ workspace swarm routing history        (web/src/App.tsx:32-33,47-81)

Header (always) ─ chat toggle (no-op off Workspace) ─ search ─ connection ─ operator
NavRail (always, 7 buttons) ─ workspace | agents | data | models | routing | remote | settings
LibraryPanel (≥lg) ─ tabs [Workspace Swarm Routing History] + agent chips + templates
main ─ nav==workspace ? workspaceView(tab) : Agents|Data|Models|Routing|Remote|Settings
       workspaceView: swarm→SwarmPanel  routing→RoutingPanel  history→HistoryPanel+Canvas
                      else→Canvas+ChatCore
overlays ─ ControlBar+Scrubber (workspace/workspace only), Scrubber (history only),
           ToastRegion, ApprovalOverlay, Splash, RemoteLane(inside Canvas)
```
**VERIFIED** (`web/src/App.tsx:109-196`, `web/src/panels/NavRail.tsx:9-17`, `web/src/panels/LibraryPanel.tsx:19-24`).

Note the **duplicate Routing destination**: nav `routing` and tab `routing` both render `RoutingPanel` (`web/src/App.tsx:112-113`, `:155-156`), so the same table is reachable two ways while the tab route has no workspace chrome. **VERIFIED.**

---

## 3. File census — every file under `web/src/**`

### 3.1 Top level, a11y, assets, chat, figcraft, splash

| File | Role | Tests |
|---|---|---|
| `web/src/App.tsx` | shell, hash nav, 28-surface frame, splash mount | none direct (static pins in `designParity`, `layout_hive`) |
| `web/src/main.tsx` | bootstrap, launch token shim, StrictMode | `client.strictmode`, `apiLifecycle` (indirect) |
| `web/src/fontsource.d.ts` | font module declaration | — |
| `web/src/a11y/keyboard.ts` | shortcuts, zoom clamp, roving index, bee angle | `keyboard.test.ts` |
| `web/src/a11y/reducedMotion.ts` | OS reduced-motion hook | none direct |
| `web/src/assets/manifest.ts` | FigCraft gate + accepted-export hashes | `noHardcoded` (consumer limits) |
| `web/src/chat/ChatCore.tsx` | pinned chat, composer, root intents, tool strip | `chatMotion` (parser only) |
| `web/src/chat/ChatMotion.tsx` | fenced-block renderer, bee, dots, code block | `chatMotion.test.ts` |
| `web/src/figcraft/types.ts` | catalog types | none |
| `web/src/figcraft/catalog.ts` | generated 21-board/114-surface catalog | `noHardcoded`, `designParity` (presence) |
| `web/src/figcraft/FigCraftGallery.tsx` | accessible board gallery + preview | none |
| `web/src/splash/HiveMindSplash.tsx` | boot veil with live connection copy | none |
| `web/src/styles/index.css` | utilities, focus, reduced motion, ambient/legacy motion | `designParity` (source pins) |
| `web/src/styles/motion.css` | `motion-*` decorative layer | none |
| `web/src/styles/splash.css` | HoneyGlass material | none |
| `web/src/styles/tokens.css` | CSS token mirror | `tokens.test.ts` (TS side) |
| `web/src/tokens/tokens.ts` | THE token module | `tokens.test.ts`, `designParity`, `noHardcoded` |
| `web/src/tokens/contrast.ts` | WCAG math | `tokens.test.ts` |

### 3.2 API layer

| File | Role | Tests |
|---|---|---|
| `web/src/api/index.ts` | API factory + dev-mock boundary | `apiBoundary`, `apiLifecycle` |
| `web/src/api/client.ts` | WS client/transport, session bootstrap, ACK correlation | `apiClient`, `client.strictmode`, `apiLifecycle` |
| `web/src/api/contract.ts` | frozen vocabulary, decoders, validators | `contract.test.ts` |
| `web/src/api/http.ts` | bounded fetch (Promise.race + abort) | `apiLifecycle`, `health` |
| `web/src/api/health.ts` | `/health` probe | `health.test.ts` |
| `web/src/api/failure.ts` | named HTTP failure / schema failure | `failure.test.ts` |
| `web/src/api/refusals.ts` | client refusal construction | `refusalParity` |
| `web/src/api/ids.ts` | uuidv7 id factory | `contract.test.ts` (uuid bits) |
| `web/src/api/mock.ts` | dev-only test double (must never ship) | `apiBoundary` |

### 3.3 State layer

| File | Role | Tests |
|---|---|---|
| `web/src/state/store.ts` | whole UI state + reducer + actions | `store`, `store.strictmode`, `store.spawn-replay`, `streamingRender`, `toolActivity.*`, `approval.*`, `agents_spawn_matrix` |
| `web/src/state/context.ts` | store context/hooks | all store tests |
| `web/src/state/agentState.ts` | 16-state vocabulary, transitions, node kinds | `agentState`, `state.parity` |
| `web/src/state/strings.ts` | single string table incl. refusal copy | `refusalParity`, `agentsToolActivityMount` |
| `web/src/state/approval.ts` | approval resolve/replay machines + selectors | `approval.state`, `approval.integration` |
| `web/src/state/spawnMatrix.ts` | class matrix + spawn gate | `agents_spawn_matrix` |
| `web/src/state/metrics.ts` | HUD derivation (server vs derived) | `metrics.test.ts` |
| `web/src/state/chatIntents.ts` | prompt-only root intents | `agents_spawn_matrix` |
| `web/src/state/useAppReducedMotion.ts` | OS ∨ in-app reduced motion | none direct |

### 3.4 Flow layer

| File | Role | Tests |
|---|---|---|
| `web/src/flow/Canvas.tsx` | React Flow canvas, filters, drag-rewire, zoom, minimap, replay | `layout_hive`, `graph.replay` (static pins) |
| `web/src/flow/graph.ts` | pure layout/edges/filters/replay projection | `layout_hive`, `graph.replay` |
| `web/src/flow/nodes.tsx` | 6 card renderers + anchor + state glyphs | `layout_hive` (nodrag pins) |
| `web/src/flow/BeeEdge.tsx` | bee edge, direction, ride animation, broken routing | none |
| `web/src/flow/HiveField.tsx` | honeycomb/beehive vector backdrop, ripple, swarm | none |
| `web/src/flow/RemoteLane.tsx` | floating remote/daemon lane | none |

### 3.5 Panels (36)

| File | Role | Tests |
|---|---|---|
| `AgentsPanel.tsx` | agents registry + hierarchy + 9 embedded panels | `agentsToolActivityMount` |
| `AgentHierarchy.tsx` | parent-link tree, per-row SPAWN | `agents_spawn_matrix` (gate), `layout_hive` |
| `AgentProfilesPanel.tsx` | profile list + JSON import/delete | none |
| `AgentRosterPanel.tsx` | server roster projection | none |
| `AgentMessagesPanel.tsx` | durable mailbox + operator answer | none |
| `AgentMemoryPanel.tsx` | role memory + branch-scoped memory | none |
| `MemoryBanksPanel.tsx` | v14 bank inspector, journal, clear/prune/trust | none |
| `A2AInboxPanel.tsx` | open-question inbox + answer + history | none |
| `ProposalsPanel.tsx` | self-evolution proposals + authorize/apply/rollback | `proposals.test.ts` (decode only) |
| `LessonsPanel.tsx` | authorized lessons (read-only) | `lessons.test.ts` (decode only) |
| `ProjectsPanel.tsx` | project registry (read-only) | none |
| `ApprovalOverlay.tsx` | nonce-bound approval dialog | `approval.*`, `toolActivityPanels`, `toolActivity.integration` |
| `ConfirmDialog.tsx` | generic destructive confirm | none |
| `ControlBar.tsx` | spawn/re-route/cancel/kill + pending status | none |
| `DaemonPanel.tsx` | daemons, supervisor, reports, switch/retire | none |
| `DataPanel.tsx` | jailed file tree + preview | none |
| `DiagnosticsPanel.tsx` | `/diagnose` summary + raw JSON | none |
| `DevGallery.tsx` | DEV-only 16-state gallery | none |
| `FigCraftPanel.tsx` | in-app FigCraft representation | `noHardcoded`, `designParity` |
| `Header.tsx` | brand, chat toggle, search, connection, operator | none |
| `HistoryPanel.tsx` | committed runs/revisions list | none |
| `HudBadge.tsx` | throughput HUD | `metrics.test.ts` (derivation) |
| `LibraryPanel.tsx` | project/library rail, tabs, chips, templates | none |
| `ModelsPanel.tsx` | provider/model/usage/balance | none |
| `NavIcon.tsx` | 7 inline icons | none |
| `NavRail.tsx` | 72px nav with roving arrows | none |
| `RemotePanel.tsx` | remotes, devices, grants, actions, audit + DaemonPanel | none |
| `RoutingPanel.tsx` | derived routing table | none |
| `RunDetailPanel.tsx` | run detail + state bar + memory snapshot | none |
| `RunTranscriptPanel.tsx` | persisted transcript | none |
| `Scrubber.tsx` | revision time-travel range | none |
| `SettingsPanel.tsx` | 5-tab settings (overview/provider/appearance/diagnostics/figcraft) | none |
| `StatePack.tsx` | PanelTitle/Empty/Loading/Error/Replay/Derived | none |
| `SwarmPanel.tsx` | swarm census + hierarchy + capacity + projects | none |
| `ToastRegion.tsx` | refusal toasts | none |
| `ToolActivityPanel.tsx` | tool-call rows + summary | `toolActivityPanels`, `toolActivity.integration` |

### 3.6 Tests (31 specs + 1 helper)

`agentState`, `agentsToolActivityMount`, `agents_spawn_matrix`, `apiBoundary`, `apiClient`, `apiLifecycle`, `approval.integration`, `approval.state`, `chatMotion`, `client.strictmode`, `contract`, `designParity`, `failure`, `graph.replay`, `health`, `keyboard`, `layout_hive`, `lessons`, `metrics`, `noHardcoded`, `proposals`, `refusalParity`, `state.parity`, `store.spawn-replay`, `store.strictmode`, `store`, `streamingRender`, `tokens`, `toolActivity.integration`, `toolActivity.store`, `toolActivityPanels`, `tests/helpers/fakeUiApi.ts` (controllable double). **VERIFIED** by directory listing; all are `.ts` and run in a **node** environment with no jsdom (`web/vite.config.ts:26-30`), i.e. component coverage is static-markup only.

---

## 4. Surface records

Each record: purpose · data source · actions · states · accessibility · material · wiring gaps · bugs · tests.

### 4.1 App shell — `web/src/App.tsx`

- **Purpose:** single mount frame; one store; hash-synced nav/tab; offline/mocks banners; skip link; overlays.
- **Data:** `view.nav`, `view.tab`, `connection`, `connectionRefusal`, `apiKind`, `run`, reduced motion (`web/src/App.tsx:36-44`). **VERIFIED**
- **Actions:** `setNav`, `setTab` via hash (`:47-81`); store `init`/`dispose` (`:208-213`).
- **States:** mock banner only in dev (`:175-179`); `role="alert"` offline/reconnect banner (`:180-184`); history empty panel with CTA (`:118-138`); splash (`:218`).
- **A11y:** skip link to `#hive-canvas` on workspace/workspace else `#main-content` (`:107`, `:167-169`); `main tabIndex={-1}` (`:174`).
- **Material:** `bg-canvas text-ink-900`, panel chrome, no layout animation beyond CSS.
- **Gaps:** hash supports only 7 nav + 4 tab segments (settings sub-tabs and panel scroll are not linkable); no error boundary anywhere in `web/src`; `#/workspace/history` shows Scrubber over an empty canvas area with no ControlBar (intended, `:192-196`).
- **Tests:** no App/route test; nav hash behaviour **UNVERIFIED** by execution.

### 4.2 Header — `web/src/panels/Header.tsx`
- **Purpose:** brand, chat visibility, agent search, connection pill, operator chip.
- **Data:** `connection`, `project`, `tasks`, `chat.visible`, `history.replayGraph`, `connectionRefusal` not read.
- **Actions:** toggle chat (`:55`), open search, select task (`:130`), exit replay (`:45`).
- **States:** replay "Return live" button (`:41-49`); connection pill green/amber/signal by status (`:80-92`); search empty ("No agents in this run.", `:121-123`).
- **A11y:** `aria-pressed` chat (`:53`), `aria-expanded` search (`:71`), label + Escape close (`:104-116`); search is a plain `ul` of buttons (no `listbox`/`option`, no arrow keys, no outside-click close).
- **Material:** 52px panel bar, honey avatar, hairline separators.
- **Gaps:** chat button is a no-op outside Workspace (`App.tsx:141-146`); search "No agents in this run" also appears when there is no run; no keyboard navigation in results.
- **Tests:** none.

### 4.3 NavRail — `web/src/panels/NavRail.tsx`
- **Purpose:** primary navigation for 7 destinations.
- **Data/actions:** `view.nav`, `setNav` (`:20-21`).
- **States:** active = ink pill + honey icon (`:50-66`); `aria-current="page"` (`:45`).
- **A11y:** roving tabindex (only active is tabbable, `:46`), ArrowUp/Down moves DOM focus but does **not** activate (`:28-36`) — users must press Enter; `nextRovingIndex` wraps (`web/src/a11y/keyboard.ts:100-106`).
- **Material:** 72px, honeycomb icon set, no text on hover beyond label.
- **Gaps:** comment says "six destinations", code ships seven (`:1`, `:9-17`); no tooltip/keyboard hint.
- **Tests:** `keyboard.test.ts` covers the helper, not the rail.

### 4.4 LibraryPanel — `web/src/panels/LibraryPanel.tsx`
- **Purpose:** project identity, workspace tabs, agent chips, template catalog, legend/settings disclosure.
- **Data:** `tasks` or replay projection (`:45-49`), `project`, `settings`, fetched `/api/templates` (`:54-97`).
- **Actions:** `setTab` (`:142`), `selectTask` (`:167`), `insertTemplate` (`:216`).
- **States:** templates loading/error/empty (`:198-208`); agent empty (`:157-158`); `+N more` overflow (`:187-191`); hidden below `lg` (`:102`).
- **A11y:** `role="tablist"` + roving + Arrow/Home/End with focus move (`:111-132`), chip `aria-label` includes state (`:168`), disclosure for legend.
- **Material:** 240px paper-alt rail, honeycomb agent chips (clip-path hexagon), honey tip card.
- **Gaps (high):** tab click does not set `nav='workspace'`, so tabs are inert on every non-workspace nav (`:142`; `App.tsx:148-163`); chips exclude the HyperAgent (`:49`) so a root-only run shows "No active agents"; "+N more" has no destination; template fetch has no timeout/abort/token (global shim covers `/api/` only when launch token exists, `main.tsx:30`); hardcoded "HiveMind v1.2-dev" (`:261`) duplicates `package.json` version.
- **Tests:** none.

### 4.5 ControlBar — `web/src/panels/ControlBar.tsx`
- **Purpose:** the four mutating run controls with visible preconditions.
- **Data:** `run`, `tasks`, `selectedTaskId`, `pendingCommands`, `connection`, `replayGraph` (`:13-25`).
- **Actions:** `spawnSubHive(seedParent?.id)`, `rewireSelected(target)`, `cancelRun`, `killWorker` after confirm (`:82-178`); target select (`:58-71`).
- **States:** disabled + reason line for spawn (`:76-96`), disabled reroute/cancel/kill (`:111-144`), inline kill modal (`:145-181`), pending/run/empty footer (`:183-189`).
- **A11y:** titles explain disabled state; `role="note"` reason (`:91`); kill dialog `role="dialog" aria-modal` (`:148-152`) but **no focus move, no trap, no Escape** and background stays reachable — inconsistent with `ConfirmDialog` (`web/src/panels/ConfirmDialog.tsx:30-50`).
- **Material:** 52px panel footer, honey primary, signal kill.
- **Bugs:** pending commands render the label "Loading" (`:184-186`); `canRewire`/`canKill` require `kind === 'worker'` while the store gate only requires a selection (`:50-53` vs `state/store.ts:742-759`) — Subagents and SuperAgents can never be re-routed from the bar; target `<select>` lists every task including hyperagent (`:66-70`) and its `aria-label` misuses the worker label (`:60`).
- **Tests:** none.

### 4.6 Scrubber — `web/src/panels/Scrubber.tsx`
- **Purpose:** committed-revision time travel + "live" exit.
- **Data:** `history.revisions/selected/count/replayGraph`, `run` (`:11-20`).
- **Actions:** `setHistoryRevision(revision)` on pointer-up/key-up (`:34-39`, `:57-59`); `exitHistoryReplay` (`:69`).
- **States:** disabled when no revision (`:30`, `:53`); position text `n of m`, replay prefix; `aria-valuetext` (`:55-65`).
- **Material:** hslider `accent-honey-500`, monospace counter.
- **Bugs:** fires a command on **every** key-up (Arrow/Tab/Shift), so holding an arrow can spam `HISTORY_REVISION_SET` (`:58-59`); index fallback treats a revision number as an index (`:18-20`); `formatSelectedRevision` in the store is never used (`state/store.ts:2572-2581`).
- **Tests:** none.

### 4.7 ToastRegion — `web/src/panels/ToastRegion.tsx`
- **Purpose:** named refusals with `code / message / how_to_fix`.
- **Data/actions:** `refusals`, `dismissRefusal` (`:8-9`); motion/react enter/exit (`:18-27`).
- **States:** max 4 entries, dedupe ≤500 ms by code (`state/store.ts:841-852`); no auto-dismiss.
- **A11y:** container `aria-live="assertive"`, each card `role="alert"` (`:13-29`), dismiss label.
- **Material:** signal border on paper, elevation e2, 440px max.
- **Gaps:** z-50 top-right sits above the approval modal (z-40) and can cover its header on narrow windows; persistent toasts can stack over the canvas controls; no "copy code" affordance.
- **Tests:** none.

### 4.8 ApprovalOverlay — `web/src/panels/ApprovalOverlay.tsx`
- **Purpose:** blocking, nonce-bound tool-call approval with evidence and TTL.
- **Data:** `approvals`, `approvalsStale`, `resolvingApprovals`, `approvalResolve`, `connection` (`:181-187`).
- **Actions:** approve/deny → `resolveApproval` (reason "operator denied" for deny, `:241-245`); retry/dismiss only when the store's actions machine allows (`:392-411`, `:416-426`).
- **States:** one dialog at a time (`:259`), "N more approvals pending" (`:321-325`), stale banner (`:250-258`), resolving (`:379-383`), failure block with retry (`:385-414`), nonce missing (`:369-377`), network approval without preview fails closed (`:151-159`, `:428-432`), expired → disabled.
- **A11y:** `role="dialog" aria-modal`, focus first button and restore (`:205-217`), Tab trap + Escape dismiss (`:291-318`), per-action `aria-label` includes tool (`:162-170`), TTL `aria-live`.
- **Material:** full-bleed scrim, `surface-panel` card r14, e3 shadow, honey approve / signal deny.
- **Bugs/gaps:** **only `approvals[0]` is ever rendered** — there is no queue navigation; if the head approval is `unconfirmed`/`stale` and dismiss is unavailable, the operator cannot reach the rest (`:204`, `:259`); stale banner `fixed top-0 z-50` overlaps Header; retry cap is 3 (`state/approval.ts:27`) but the dialog never shows attempts left.
- **Tests:** strong: `approval.state`, `approval.integration`, `toolActivityPanels`, `toolActivity.integration`, `agentsToolActivityMount` (static render).

### 4.9 ChatCore + ChatMotion — `web/src/chat/*`
- **Purpose:** pinned hive chat: prompt composer, streamed replies, root-control intents, tool strip, run refusal.
- **Data:** `messages`, `pendingCommands`, `connection`, `chat`, `streamingRunId`, `run.status/errorCode`, `settings.keyPresent`, `toolActivity`, `tasks` (`ChatCore.tsx:80-95`).
- **Actions:** `submitPrompt` with validate → send → ACK/refusal settlement (`:148-199`); retract optimistic user row on refusal (`:191-198`); draft restored unless replaced (`:63-71`); cancel run (`:416-418`); hide/show chat (`:249`, `:128-140`); starters and CONTROL_INTENTS fill the composer only (`:284-287`, `:376-379`).
- **States:** empty (starters + "READY · NO RUN YET", `:275-301`), run refused banner (`:257-267`), streaming caret + dots (`:317`, `:356-363`), key-missing pre-check (`:153-165`), tool strip last 4 (`:321-355`).
- **A11y:** section/list labels (`:208-209`, `:272-273`), live status (`:231-244`), labelled textarea (`:392-399`), buttons labelled; chat hidden state leaves a reopen pill (`:128-140`).
- **Material:** 400×560 (max 92%), gradient ink→honey header, honeycomb dot texture, r18, e2; `ChatMotion` fenced code blocks with traffic dots (`ChatMotion.tsx:134-154`).
- **Gaps:** chat exists only in workspace/workspace; `CHAT_PIN`/`chat.pinned` are never used by any surface (`state/store.ts:2302-2307`, no consumer); `entry.text` is never populated, so the strip shows only tool name + outcome; edit is not supported; no message timestamps or run linkage UI; "READY · NO RUN YET" also shows for a selected history run with no transcript (`:299`).
- **Tests:** `chatMotion` (parser); no composer/submit UI test (store logic covered by `store.test.ts`).

### 4.10 Canvas + flow layer — `web/src/flow/*`
- **Purpose:** spatial hive: layout, cards, bee edges, filters, drag-to-reparent, zoom/minimap, read-only replay.
- **Data:** `tasks`, `selectedTaskId`, `run`, `connection`, `history`, `chat.visible`, `viewport` (`Canvas.tsx:108-135`).
- **Actions:** `selectTask`, `spawnSubHive`, `rewireSelected` via drop confirm (`:141-153`, `:273-305`, `:493-521`); filters (`:523-564`); zoom/fit/recenter (`:675-732`); shortcuts Ctrl+0/±/1/F (`web/src/a11y/keyboard.ts:52-81`); `setViewport` on move end (`:603-606`).
- **States:** empty hint (`:734-740`), replay badge (`:652-658`), replay fit (`:352-360`), drag line (`:475-492`), position persistence per run in localStorage (`:61-102`), hidden minimap <1280 (`styles/index.css:137-142`).
- **Graph:** deterministic column hive with fixed origin root (`graph.ts:91-203`), dagre fallback (`:205-310`), visibility filters that retain ancestors (`:319-366`), edge types (`:422-442`), replay projection (`:450-499`).
- **Cards:** Super 420×150, Worker 320×96, Tool 420×240 dark result well, Broken 320×110 signal, Subagent, HyperAgent pill, invisible hive anchor (`nodes.tsx:147-340`; sizes `tokens.ts:166-179`, `graph.ts:39-46`).
- **A11y:** `role="application"` + label (`:448-450`), node `ariaLabel`s (`graph.ts:411`), minimap labelled via prop and DOM patch (`:395-410`, `:620-649`), sr-only live selection/reconnect (`:744-747`), focus-visible ring on nodes (`styles/index.css:159-162`).
- **Material:** honeycomb vector field + beehive SVG + bees (`HiveField.tsx:73-196`), bee edges with arrow markers (`BeeEdge.tsx:14-91`), ambient scan/bee layer (`Canvas.tsx:462-473`), legend + zoom stack.
- **Gaps/bugs:** empty hint says "No agents in this run" whenever only the HyperAgent exists because `buildNodes` filters it out (`graph.ts:395-396`; `Canvas.tsx:734-739`); `role="application"` is a strong screen-reader mode that can trap virtual cursor; two ambient motion systems run together (`hive-scan`/`ambient-bee` in `styles/index.css:236-305` plus `motion-honeycomb-glow`/`motion-hive-sheen`/`honeycomb-ripple` in `HiveField.tsx:87`, `:160-190`); Ctrl+F also fits the view but Settings documents only Ctrl+1 (`keyboard.ts:76-79`; `SettingsPanel.tsx:26-35`); drag-target highlight hardcodes an off-token rgba glow (`Canvas.tsx:212`); no per-node tool/child counts on cards.
- **Tests:** `layout_hive` (origin, no overlap, branch sides, filters, nodrag static pins), `graph.replay`; no interaction test for drag/rewire/fit.

### 4.11 Agents section — `web/src/panels/AgentsPanel.tsx` + 9 embedded panels
- **Purpose:** one page for hierarchy, cards, tool activity, profiles, roster, A2A inbox, proposals, lessons, memory, banks, messages.
- **Data:** `tasks`, `selectedTaskId`, `toolActivity` (`AgentsPanel.tsx:27-33`).
- **Actions:** select, spawn (per hierarchy row), all mutations inside the embedded panels.
- **States:** empty agents (`:41`), tool-activity region (`:70-77`).
- **Wiring gaps:** 13+ HTTP reads fire on mount (profiles 1, roster 1, A2A 2, proposals 1, lessons 1, memory 2, banks 4, messages 1) — see §6.3; no sub-navigation, so the page is a long diagnostic dump; SPAWN rows are disabled with the replay reason (`AgentHierarchy.tsx:79-82`).
- **Tests:** `agentsToolActivityMount` static render pins the tool region; nothing else.

Sub-panel records (compact):

| Surface | Data / actions | States | Key gaps | Tests |
|---|---|---|---|---|
| `AgentHierarchy` | `tasks`; `selectTask`, optional `spawnTask`/`spawnReason` | indented rows, disabled SPAWN + reason | mounted without `spawnTask` in Agents → false replay reason (`:79-82`) | `agents_spawn_matrix` (gate only) |
| `AgentProfilesPanel` | GET/POST/DELETE `/api/agents/profiles*` (`:55,112,136`) | loading/ok/error/import notice | delete has no confirm; no busy guard → double import; textarea prefilled with a real example (`:34-42`) | none |
| `AgentRosterPanel` | GET roster by run (`:38`) | idle/loading/ok/error + empty | fetch error text collapsed to generic line (`:78-83`) | none |
| `AgentMessagesPanel` | GET messages, POST answer, confirm via re-read (`:52,119,143`) | loading/error/empty/kind filter/sending/notice | **one global answer draft**: input shows text only for `answeringId`, all Answer buttons disabled while sending (`:229-243`); load error message discarded | none |
| `A2AInboxPanel` | GET pending, GET history, POST answer, confirm by re-projection (`:77,142,201,221`) | loading/error/empty/notice/recent answers | notice used for both success and refusal (role=status); failure string parsed from `message.split(':')` (`:52-59`) | none |
| `AgentMemoryPanel` | GET `/api/agents/memory`, GET run memory, DELETE (`:80,141,189`) | loading/error/empty, scope filter, replay-hidden | clear is unconfirmed destructive; scope fetch race guarded by request counter | none |
| `MemoryBanksPanel` | 4 GETs + clear/prune/trust POSTs (`:74-253`, `:316-438`) | idle/loading/ok/error, journal chain, confirm dialog | entries effect deps `[selected]` omit `runId` → stale after run switch (`:245-304`, URL `:253`); clear leaves `status='ok'` with emptied lists; trust row locally patched after OK (`:456-458`) | none |
| `ProposalsPanel` | GET proposals, authorize/apply/rollback (`:142,186,219`) | idle/loading/error/empty/action error | uses `Authorization: Bearer` while every other write uses `x-hivemind-token` (`:194`, `:227`); `canAuthorizeProposal` requires `can_apply === false` (`:104-112`) — semantics unverified | `proposals` decode only |
| `LessonsPanel` | GET `/api/evolution/lessons`, reload on `hivemind:evolution-changed` (`:56-96`) | loading/error/empty | none beyond read-only | `lessons` decode only |
| `ProjectsPanel` | GET `/api/projects` (`:28`) | loading/error/empty | no Accept/credentials header; no retry | none |

### 4.12 Swarm, Routing, History, Run detail/transcript
| Surface | Purpose / data / actions | States | Gaps | Tests |
|---|---|---|---|---|
| `SwarmPanel` | census of the live/replay task map, capacity from `/api/runtime` polled 15 s, hierarchy with working spawn reasons, projects (`:25-99`, `:194-247`) | CAPACITY UNKNOWN / cap line / error; empty hero with 3 steps | capacity poll keeps running when tab hidden; hero CTA goes to Workspace only | none |
| `RoutingPanel` | table of parent→child edges from tasks (`:7-37`) | empty copy; no sorting | same panel reachable as nav and tab; no link back to a card | none |
| `HistoryPanel` | committed runs/revisions, replay entry, retry on refresh error, run detail + transcript (`:12-119`) | no run / no revisions / revision list / error+retry | run selector only when >1 run; revision list duplicates the Scrubber control | none |
| `RunDetailPanel` | `/api/runs/{id}/detail`, state distribution bar, task rows, memory snapshot inspect (`:86-226`) | idle/loading/ok/error + snapshot loading/ok/error | detail payload is cast, not decoded field-by-field (`:147`) | none |
| `RunTranscriptPanel` | renders store `messages` as a `<pre>` per row (`:9-38`) | no run / no transcript / list | duplicates ChatCore transcript with different typography | none |

### 4.13 Data, Models, Settings, Diagnostics
| Surface | Details | Gaps | Tests |
|---|---|---|---|
| `DataPanel` | `fileTree` + `requestFileTree`/`openFile`; dirs expand, files preview with bytes/sha (`:12-122`) | no breadcrumb/up control, no current-path label after entering a dir; refresh resets to root; preview has no close | none |
| `ModelsPanel` | `SETTINGS_STATUS` probe, `/api/provider/models`, `/api/provider/balance`, `/api/usage/runs/{id}` (`:45-223`) | provider/balance handling is exemplary (honest "not reported"); usage refetches only on run id/status (`:222`); `probeSettings` guard is 'pending' only, so each visit can re-probe | none |
| `SettingsPanel` | 5 tabs (overview/provider/appearance/diagnostics/figcraft), key+health probes, provider profile editor, motion slider, 20-swatch palette, shortcut card (`:16-35`, `:272-551`, `:129-270`) | `aria-controls` points to non-mounted tabpanels (`:421`, `:439`); reduced-motion override is not persisted (store initial false, `state/store.ts:1926`, `:2494`); provider POST renames tokenlabs→tokenlab (`:183`) — server expectation **UNVERIFIED**; FigCraft tab nests a 70vh scroll inside a scroll container (`:540-547`) | none |
| `DiagnosticsPanel` | `/diagnose`, `/api/workspace`, `/api/runs/active`; summary + raw JSON (`:34-208`) | `/diagnose` is **not** under `/api/` so the launch-token shim never adds the header (`main.tsx:30` vs `:50`) — 401/403 risk **VERIFIED** in code, impact UNVERIFIED server-side | none |

### 4.14 Remote and Daemon
| Surface | Details | Gaps | Tests |
|---|---|---|---|
| `RemotePanel` | remotes CRUD-ish (create/revoke/heartbeat), devices, grants, actions, nonce approval, audit (`:91-555`) | copy says "Fixture devices only; no network adapter" but performs real POSTs (`:330`); default names "Fixture remote"/"Study light" read as demo data (`:98-99`); capability mapping is hardcoded by class (`:85-89`) — server capability ids **UNVERIFIED**; no list refresh after action unless detail reload | none |
| `DaemonPanel` | daemons + supervisor + reports, enroll/switch/retire (`:34-287`) | interval input strips non-digits so empty → `Number('') === 0` silently (`:216`, `:111`); merge of "switch refused" notice with success notice (`:42`, `:192`) | none |

### 4.15 FigCraft and dev gallery
- `FigCraftPanel` renders gate counts (21 boards / 114 asset rows / 3 owners), gallery, source manifests, and a "pixel fidelity UNVERIFIED" footer (`web/src/panels/FigCraftPanel.tsx:16-79`; counts `web/src/figcraft/catalog.ts:757-765`). **VERIFIED.**
- `FigCraftGallery` is an accessible tablist grid with `aria-pressed` board cards, alt text incl. size/surfaces, lazy `img`, and an explicit "Preview unavailable" fallback (`web/src/figcraft/FigCraftGallery.tsx:51-134`). **VERIFIED** (read to line 140; remainder not inspected — **UNVERIFIED**).
- `DevGallery` returns `null` outside `import.meta.env.DEV` (`web/src/panels/DevGallery.tsx:7-10`). **VERIFIED.**
- **Tests:** none exercise the gallery; `noHardcoded.test.ts` only constrains consumers.

### 4.16 Splash — `web/src/splash/HiveMindSplash.tsx`
- **Purpose:** real boot veil that reports socket state and never blocks forever.
- **Data:** `connection` only (`:36`); phases visible→leaving→gone.
- **Behaviour:** ≥1.6 s minimum, leaves immediately when settled, else after 4.4 s more; 420 ms fade (`:41-60`); returning offline still opens the workspace.
- **A11y:** `role="status" aria-live="polite"` (`:71-73`), progressbar with min/max/now (`:117-126`), decorative layers `aria-hidden`.
- **Material:** frosted glass + specular + neumorphic well + liquid-glass hex field + Playfair wordmark + hand-drawn drip (`styles/splash.css:41-114`). Strongest HoneyGlass moment in the product.
- **Bugs/risks:** the progressbar value is **invented** (100/46/18, `:65`) — an honest indeterminate bar would match the "no fake data" discipline; the veil has no `inert`/focus trap, so Tab can reach the app behind it; splash.css hardcodes `rgb()` values instead of `var(--color-*)` and uses `backdrop-filter` (`splash.css:36-56`) contrary to the motion cost contract in `styles/motion.css:17-24`; no test.
- **Tests:** none.

### 4.17 Shared state surfaces
- `StatePack` gives every panel the same loading/empty/error vocabulary (`web/src/panels/StatePack.tsx:5-52`) — **VERIFIED**, but several panels bypass it with bespoke `<p>` lines (Remote, Daemon, MemoryBanks), so error styling is inconsistent.
- `ConfirmDialog` is the good modal: focus on confirm, Escape, restore focus, scrim cancel, `aria-labelledby/describedby` (`ConfirmDialog.tsx:30-103`) — used by MemoryBanks/Remote, **not** by ControlBar kill.
- `HudBadge` derives throughput with server-first, derived-second labeling and REPLAY hides live metrics (`HudBadge.tsx:8-63`; `metrics.ts:24-47`). **VERIFIED.**

---

## 5. Cross-cutting matrices

### 5.1 Store action inventory (30) — `web/src/state/store.ts:292-335`

| Action | Kind | Emits command / HTTP | Notes |
|---|---|---|---|
| `init` | lifecycle | boot snapshot + `/api/session` + WS | hydrate before socket (`:1952-1955`) |
| `dispose` | lifecycle | abort reads, close socket | generation guards |
| `submitPrompt` | mutation | `SUBMIT_PROMPT` | optimistic user row (`:2016-2021`) |
| `spawnSubHive` | mutation | `SPAWN_SUB_HIVE` | class matrix mirrored (`:2040-2064`) |
| `rewireSelected` | mutation | `REWIRE_TASK` + on-demand rev read | never guesses `expected_rev` (`:2085-2123`) |
| `cancelRun` / `killWorker` | mutation | `CANCEL_RUN` / `KILL_WORKER` | guard terminal/none |
| `resolveApproval` / `retryApproval` / `dismissApproval` | mutation | `APPROVAL_RESOLVE` | nonce required, double-submit guard, transitions in `approval.ts` |
| `pinChat` | mutation | `CHAT_PIN` | **no UI caller** |
| `insertTemplate` | mutation | `TEMPLATE_INSERT` | used by Library |
| `setHistoryRevision` / `exitHistoryReplay` / `selectHistoryRun` / `refreshRunSnapshot` | history | `HISTORY_REVISION_SET`, reads | revisions are authority |
| `setNav` / `setTab` / `selectTask` / `setChatVisible` / `setViewport` | view | none | `setTab` does not change nav (**bug**) |
| `openDiffModal` / `closeDiffModal` | dead | none | no diff surface; `diffOpen` unread |
| `setReducedMotionOverride` | pref | none | not persisted |
| `dismissRefusal` | toast | none | — |
| `probeSettings` / `probeHealth` | read | `SETTINGS_STATUS` / `/health` | pending guard |
| `requestFileTree` / `openFile` | read | `FILE_TREE_LIST` / `FILE_READ` | client jail check |
| `formatSelectedRevision` | pure | none | **no caller** |

Dead/unwired state found by exhaustive search: `diffOpen`, `chat.pinned`, `session.schema`, `viewport`, `formatSelectedRevision`, `isSnapshotDegraded`, `snapshot.*` (read nowhere outside the store). **VERIFIED** via grep over `web/src`.

### 5.2 WebSocket event consumer matrix (24 events) — single consumer `applyEvent` (`state/store.ts:1382-1841`)

| Event | Store effect | Surface |
|---|---|---|
| ACK / ERROR | toast; `E_KEY_MISSING`→settings; `E_RESYNC_REQUIRED`→gap | Toast, Header, Chat run banner |
| SERVER_HELLO | open/reconnecting, replay window, quiet snapshot | Header, Splash, approval stale banner |
| RESYNC_REQUIRED | gap + reconnecting + refusal + rehydrate | Toast, Header |
| RUN_CREATED | new run, clears graph/tools/history | Canvas, History, Chat |
| RUN_STATUS | status/error, seals stream, refreshes revisions | Canvas, ControlBar, Chat |
| REPLY_DELTA / REPLY_DONE | append/finalize assistant text (usage ignored) | ChatCore, RunTranscript |
| NODE_SPAWN / NODE_DRAG / NODE_REWIRE / NODE_STATE | task upsert with rev/kind/parent guards; rewire replay refresh | Canvas, Swarm, Agents, Routing |
| RUN_METRICS | HUD server source | HudBadge |
| APPROVAL_REQUEST / APPROVAL_RESOLVED | approval records + replay epochs | ApprovalOverlay |
| SETTINGS_STATUS_RESULT | provider/model/key/thinking/effort | Settings, Models, Library |
| FILE_TREE_RESULT / FILE_CONTENT | tree/preview | DataPanel |
| HISTORY_SNAPSHOT | replay graph + revision cursor | Canvas replay, Scrubber, History |
| PING / PONG | ignored | — |
| TOOL_CALL / TOOL_RESULT | tool rows, dedupe, bounded orphan buffer | ToolActivityPanel, ChatCore strip |
| AGENT_MESSAGE | bump `agentMessageVersion` only | AgentMessagesPanel refetch |

**No panel subscribes to the socket directly** — all WS consumption is centralized in the store. **VERIFIED.**

### 5.3 HTTP endpoint matrix (all fetches in `web/src`)
`/api/session` (`client.ts:294`) · `/health` (`health.ts:31`) · `/diagnose` (`DiagnosticsPanel.tsx:50`) · `/api/workspace` (`:51`) · `/api/runs/active` (`:52`) · `/api/templates` (`LibraryPanel.tsx:57`) · `/api/runtime` (`SwarmPanel.tsx:49`) · `/api/remotes` (`RemoteLane.tsx:35`, `RemotePanel.tsx:112`), `/api/remotes/{id}` (`:143`), create/devices/grants/revoke/heartbeat via `write()` (`:167`) · `/api/daemons` (`RemoteLane.tsx:75`, `DaemonPanel.tsx:51`), `/api/supervisor` (`:52`), reports (`:160`), switch/retire (`write`, `:90`) · `/api/devices/{id}/actions` + `/api/device-actions/{id}/resolve` (`RemotePanel.tsx:243`, `:276`) · `/api/projects` (`ProjectsPanel.tsx:28`) · `/api/provider/models` (`ModelsPanel.tsx:75`), `/api/provider/balance` (`:106`), `/api/usage/runs/{id}` (`:172`), `/api/provider/profile` (`SettingsPanel.tsx:140`, `:173`) · `/api/agents/profiles(+import/delete)` (`AgentProfilesPanel.tsx:55,112,136`) · `/api/agents/runs/{id}/roster` (`AgentRosterPanel.tsx:38`) · `/api/agents/runs/{id}/messages(+answer)` (`AgentMessagesPanel.tsx:52,119`) · `/api/agents/memory`, run memory, DELETE (`AgentMemoryPanel.tsx:80,141,189`) · `/api/a2a/pending`, `/api/a2a/messages` (`A2AInboxPanel.tsx:77,142`) · `/api/memory/journal/verify`, `/journal`, `/summary`, `/banks`, `/banks/{id}/entries`, `/clear`, `/prune`, `/trust` (`MemoryBanksPanel.tsx:74-438`) · `/api/runs/{id}/detail` (`RunDetailPanel.tsx:140`), `/api/memory/context/{task}` (`:103`) · `/api/agents/runs/{id}/proposals(+authorize/apply/rollback)` (`ProposalsPanel.tsx:142,186,219`) · `/api/evolution/lessons` (`LessonsPanel.tsx:66`) · history reads `/api/history/runs` etc. (`store.ts:480-492`, `:985-1011`). **VERIFIED.**

**Auth-header split (risk):** writes use `x-hivemind-token` in Remote/Daemon/Memory/Profiles/Messages/A2A/Settings (`RemotePanel.tsx:172`; `DaemonPanel.tsx:95`; `MemoryBanksPanel.tsx:321`; `AgentProfilesPanel.tsx:117`; `AgentMessagesPanel.tsx:126`; `A2AInboxPanel.tsx:208`; `SettingsPanel.tsx:180`), but Proposals uses `Authorization: Bearer` (`ProposalsPanel.tsx:194`, `:227`). **VERIFIED** divergence; which one the server accepts is **UNVERIFIED**.

### 5.4 Loading / empty / error / refusal inventory
- Loading: `LoadingState`, `settings.status==='pending'`, per-panel `status==='loading'`, `pendingCommands` (mislabeled "Loading" in ControlBar), splash phases.
- Empty: every panel has one; notable honest empties: history "Nothing to replay yet" (`App.tsx:118-138`), memory "no rows invented" (`AgentMemoryPanel.tsx:212-219`), FigCraft "no boards… unavailable rather than fabricated" (`FigCraftPanel.tsx:45-49`).
- Error: `ErrorState`, inline `role="alert"` lines, settings refusal block, history retry, Data error+retry.
- Refusal: single toast pipeline; server ERROR preferred; local `E_*` only for pre-send/transport (`state/store.ts:1858-1875`; `api/refusals.ts:20-45`).
- **Gap:** no surface for `snapshot.status` / `snapshot.incomplete` (`no-committed-revision`, `messages-unavailable`, `run-changed`, `no-run`) — these are computed and thrown away visually.

### 5.5 Accessibility attribute inventory (representative)
`aria-current` nav (`NavRail.tsx:45`); `aria-pressed` chips/cards/toggles (`Header.tsx:53`, `LibraryPanel.tsx:140`, `Canvas.tsx:526`, `FigCraftGallery.tsx:67`); `aria-label` regions/sections on nearly every panel; `aria-modal` dialogs (`ApprovalOverlay.tsx:285`, `ConfirmDialog.tsx:67`, `ControlBar.tsx:149`); `aria-live` (toast assertive, HUD, chat status, scrubber, approval TTL, canvas sr-only); `role="status"` splash/panel states; `role="progressbar"` splash; focus restore in approval/confirm; Tab traps in approval only; skip link; focus-visible tokens (`styles/index.css:27-30`).
**Gaps:** kill dialog lacks the trap/escape/focus behaviour; Header search lacks combobox semantics; Settings `aria-controls` broken for inactive tabs; splash not `inert`; `role="application"` on canvas; no reduced-transparency/forced-colors handling; no focus management on nav arrow keys beyond focus (activation requires Enter).

### 5.6 Motion and reduced-motion paths
- OS `prefers-reduced-motion` globally clamps durations (`styles/index.css:110-119`) and disables named ambient classes (`:336-352`), repeated per class in `motion.css:400-446`.
- In-app flag sets `data-reduced-motion` from OS ∨ override (`App.tsx:83-85`; `useAppReducedMotion.ts:7-10`) and CSS honours it (`motion.css:425-446`).
- Ambient slider sets `data-ambient-motion` + `--ambient-strength` (`App.tsx:87-99`; `SettingsPanel.tsx:291-300`; `motion.css:66`).
- **Bug:** `[data-ambient-motion="off"]` also kills `.state-pulse` (`styles/index.css:381-388`), i.e. a *state* indicator is suppressed by a *decoration* switch — contradicts `motion.css:11-15` ("Nothing here encodes state").
- **Duplication:** legacy `hive-*`/`bee-*` motion in `index.css:169-305` coexists with the `motion-*` layer; the canvas uses both (`Canvas.tsx:462-473` + `HiveField.tsx:87,160-194`).
- Reduced-motion fallbacks are thoughtful: BeeEdge swaps a traveling bee for a static midpoint glyph (`BeeEdge.tsx:51-66`); HiveField drops ripple/bees (`HiveField.tsx:160-190`); toast/`motion/react` duration 0 (`ToastRegion.tsx:22-25`).

### 5.7 Design tokens and material
- 20 printed swatches + 2 chrome-only values, exactly mirrored in CSS (`tokens.ts:22-43`, `:185-188`; `tokens.css:6-33`), pinned by `tokens.test.ts` and `designParity.test.ts`.
- Type scale utilities for the 8 printed steps (`index.css:38-91`) plus off-scale operator sizes (`text-[10.5px]`, `[11.5px]`, `[12.5px]`, `[26px]`) used widely (e.g. `LibraryPanel.tsx:180`, `Swatch` grid) — pragmatic but outside the printed ramp; `designParity` only enforces a minimum.
- One warm-ink elevation family; tier semantics explicitly UNVERIFIED (`tokens.ts:113-124`, `tokens.css:49-52`).
- Layout constants for cards/chat/hud/minimap/rail/library (`tokens.ts:166-179`) are used by canvas/chat/rail (`graph.ts:39-46`, `ChatCore.tsx:210`, `LibraryPanel.tsx:103`, `HudBadge.tsx:19-20`).
- **Contrast exception breached:** amber is declared large-text-only (`tokens.ts:77-78`) but used as 12 px text in `ToolActivityPanel.tsx:58` (`text-amber` on panel). **VERIFIED** code; ratio ≈3.3–3.6 → fails AA for normal text.
- Splash introduces raw `rgb()` material values and `backdrop-filter` outside the token mirror (`splash.css:36-56`), while the rest of the app is token-pure.

### 5.8 Splash (recorded in §4.16; summary)
Real boot surface, best material expression, honest connection copy, invented progress percentage, no focus isolation, no tests.

---

## 6. Product-feel assessment

### 6.1 Is it alive?
**Yes, at three levels, with caveats.**
1. Connection liveness is visible and honest: Header pill, splash copy, offline/reconnect banner, approval stale banner (`Header.tsx:80-92`; `App.tsx:101-106`; `ApprovalOverlay.tsx:250-258`).
2. Graph liveness reads well: node state glyphs, pulsing running dots, bee edges with direction, HUD derived/live distinction, replay hides live metrics (`nodes.tsx:41-75`; `HudBadge.tsx:39-58`).
3. Chat liveness: streaming caret, typing dots, working bee, live "N active" label (`ChatCore.tsx:95-102`, `:229-245`, `:317`, `:356-363`).
Caveat: simultaneous ambient systems plus `.state-pulse` make the canvas visually busy at rest; nothing throttles when idle.

### 6.2 Does it show what the Hive is doing?
**Structural work: yes. Executed work: only partially.**
- Yes: hierarchy, kinds, states, progress, parent edges, run status, revisions, approvals, agent messages, memory banks, proposals, lessons, daemon reports.
- No/weak: which agent ran which tool (wire `task_id` discarded), what a tool returned (`bytes`/`sha256` discarded), per-branch transcript (only one global chat + one run transcript), per-node timing/cost, no run timeline, no diff surface (decision-gated by design, `state/strings.ts:243-247`).
- The `ToolActivityPanel` is honest but abstract: `tool`, 12-char `args_sha256`, `turn`, outcome (`ToolActivityPanel.tsx:153-177`), no args preview exists on the wire at all (`contract.ts:295-302`).

### 6.3 Do agents have visible tools/capabilities?
**Profiles list grants, the canvas does not.**
- `AgentProfilesPanel` shows `allowed_tool_ids.length` and refusal policy (`AgentProfilesPanel.tsx:173-175`) — the only place a capability grant is visible.
- Approval dialog names the tool + args hash + optional preview (`ApprovalOverlay.tsx:326-367`) — good per-call evidence.
- Node cards show kind/label/progress only (`nodes.tsx:193-224`); no tool count, no last tool, no permission badge.
- Spawn capability *is* visible and explained (matrix reasons, `spawnMatrix.ts:93-123`), except the Agents-surface bug that mislabels it as replay (§9 #1).
**Net:** a user can learn that a worker exists but not what it is allowed to do or what it just did, without leaving the canvas.

### 6.4 Is the installed-app experience coherent?
**Frame: yes. Path: no.**
- Coherent: 52 px bar + 72 px rail + 240 px library + canvas + pinned chat is a designed spatial workstation; splash→app transition is branded; empty states teach the first action (Swarm hero `SwarmPanel.tsx:137-191`; chat starters `ChatCore.tsx:275-301`; history CTA `App.tsx:118-138`).
- Incoherent: tab/nav dead ends; chat toggle dead outside Workspace; Agents page = 11 stacked panels; Data has no path memory; version string drift (`LibraryPanel.tsx:261` vs `package.json:4`); Remote copy says "fixture… no adapter" while performing real device writes (`RemotePanel.tsx:330` vs `:164-181`); two motion systems.

---

## 7. Store selector/state reference (for the next lanes)

`UiState` fields (`state/store.ts:248-290`): `initialized, apiKind, connection, connectionRefusal, session, lastSeq, run, tasks, graphRev, selectedTaskId, messages, streamingRunId, agentMessageVersion, metrics, history, snapshot, resyncGap, approvals, resolvingApprovals, approvalResolve, approvalReplay, approvalsStale, approvalSeenEpoch, toolActivity, refusals, pendingCommands, view, viewport, chat, settings, fileTree, project, reducedMotionOverride, diffOpen`.

Notable derived/exported helpers: `mergeSnapshotTasks` (`:353-371`), `rewireBlocker` (`:742-759`), `isSnapshotDegraded` (`:788-794`), `MAX_BUFFERED_TOOL_RESULTS=16` (`:128`), `approval` selectors (`state/approval.ts:446-460`), `spawnGate` (`state/spawnMatrix.ts:93-123`), `deriveHud` (`state/metrics.ts:24-47`).

**Authorities that must not be contradicted by any UX change:** ACK-before-mutation; revisions/`expected_rev` from the server, never the local counter (`store.ts:78-87`); approvals nonce-bound and resolution confirmed only by `APPROVAL_RESOLVED`; replay is read-only; no invented metrics ("—" instead of 0).

---

## 8. Test coverage of UI surfaces

**Well covered (logic/static):** contract & refusals; store reducer incl. approvals/tool buffering/replay guards; spawn matrix; HUD derivation; keyboard; graph layout/filters/replay; tokens & design parity; string-table discipline; approval overlay static render; tool activity panel logic.

**Uncovered surfaces (zero tests):** App/hash routing, Header, NavRail, LibraryPanel, ControlBar, Scrubber, ChatCore composer/submit flow, Canvas interactions (drag, filter toggles, zoom, minimap), all 9 agents sub-panels except tool activity, Data, Models, Settings, Diagnostics, Remote, Daemon, History, Run detail/transcript, Swarm, Projects, FigCraft gallery, splash, all polling/loading/error paths.

**Structural limit:** `environment: 'node'` and `include: ['tests/**/*.test.ts']` (`web/vite.config.ts:26-29`) means no DOM events, no user-event testing, no jsdom assertions on focus/ARIA state. Focus traps, Escape handling, toast layering and hover/disabled behaviour are therefore asserted only by string/source pins (`layout_hive.test.ts` "static pins", `agentsToolActivityMount.test.ts` source glob) — **VERIFIED** by config and test contents; those behaviours are **UNVERIFIED** at runtime.

---

## 9. Top-10 UX / wiring bugs

Severity scale: **S1** = user-facing lie or dead end on the primary path; **S2** = broken/absent capability a user will hit; **S3** = polish/correctness defect.

| # | Sev | Bug (what a user experiences) | Evidence | Fix location (exact) |
|---|---|---|---|---|
| 1 | **S1** | On the live Agents page every **SPAWN** button is disabled with *"Replay is read-only. Exit replay to spawn a child."* even during an active run, because the hierarchy is mounted without a spawn handler. | `web/src/panels/AgentsPanel.tsx:43` mounts `<AgentHierarchy …>` with no `spawnTask`/`spawnReason`; fallback reason at `web/src/panels/AgentHierarchy.tsx:79-82` | `web/src/panels/AgentsPanel.tsx:43` — pass `spawnTask={(id)=>void spawnSubHive(id)}` and a `spawnReason` built from `spawnGate` exactly as `web/src/panels/SwarmPanel.tsx:200-211` does (AgentHierarchy already supports both props, `:21-23`). |
| 2 | **S1** | The Library **Workspace tabs** (Swarm / Routing / History) do nothing on Agents/Data/Models/Routing/Remote/Settings: clicking changes hidden state, screen stays put. | `web/src/panels/LibraryPanel.tsx:142` calls only `setTab`; `web/src/App.tsx:148-163` selects content by `nav`; `web/src/state/store.ts:2461-2463` `setTab` never touches `nav` | `web/src/state/store.ts:2461-2463` — add `setWorkspaceTab(tab){ set({view:{nav:'workspace',tab}}) }` (or make `setTab` also set `nav:'workspace'`) and call it from `web/src/panels/LibraryPanel.tsx:142`. |
| 3 | **S1** | The Header **chat toggle** is enabled everywhere but the chat only exists in Workspace → Workspace; pressing it on Settings/Agents/etc. changes nothing visible (and can leave the chat hidden when the user later returns). | `web/src/panels/Header.tsx:50-67` always renders/enables; `web/src/App.tsx:141-146` renders `<ChatCore/>` only in the workspace/workspace branch | `web/src/panels/Header.tsx:52-55` — when `view.nav!=='workspace' \|\| view.tab!=='workspace'`, either disable with `aria-disabled`/title "Chat lives in Workspace" or make the click call `setNav('workspace')` + `setTab('workspace')` + `setChatVisible(true)`. |
| 4 | **S2** | A degraded or failed rehydration is invisible: the canvas silently shows an unconfirmed/empty graph while the UI still looks "Connected". `snapshot.status`, `incomplete`, `source`, `attempts` are computed and never rendered. | `web/src/state/store.ts:190-201` (state), `:1079-1307` (writes incl. `degraded`/`incomplete`), `:788-794` (`isSnapshotDegraded` exported, zero consumers) | `web/src/App.tsx:184-185` — insert a `snapshot.status` banner (amber for `degraded`, signal for `failed`) listing `snapshot.incomplete` with a **Re-sync** button calling `refreshRunSnapshot('manual')`; reuse the existing store action (`web/src/state/store.ts:2450-2452`). |
| 5 | **S2** | Multiple pending approvals are unreachable: only `approvals[0]` is rendered, no queue navigation; toast region (z-50) sits above the modal (z-40) and can cover it. An unconfirmed head approval can block the rest. | `web/src/panels/ApprovalOverlay.tsx:204`, `:259` (`slice(0,1)`), `:321-325` ("N more"); `web/src/panels/ToastRegion.tsx:16` (`z-50`) vs `ApprovalOverlay.tsx:282` (`z-40`) | `web/src/panels/ApprovalOverlay.tsx:204,259` — add local `activeIndex` + "1 of N · Previous/Next" (data already in `approvals`), and lower toast container or move it to bottom-right while `approvals.length>0` in `ToastRegion.tsx:16`. |
| 6 | **S2** | Proposals authorization/apply/rollback send `Authorization: Bearer <token>` while the rest of the app sends `x-hivemind-token`; one of the two is wrong against the server, so these buttons can be the only writes that 401. | `web/src/panels/ProposalsPanel.tsx:194`, `:227`; contrast with `web/src/panels/MemoryBanksPanel.tsx:321`, `RemotePanel.tsx:172`, `DaemonPanel.tsx:95` | `web/src/panels/ProposalsPanel.tsx:192-197` and `:225-230` — send `'x-hivemind-token': sessionToken` (and `Accept`) to match the proven pattern, or introduce one `writeHeaders(token)` helper used by all panels. |
| 7 | **S2** | The launcher's one-time secret is attached only to `/api/*`; **`/health`** and **`/diagnose`** bypass the shim, so Diagnostics and Server-health can fail with 401/403 in a launch-token deployment while the rest of the UI works. | `web/src/main.tsx:30` (`pathname.startsWith('/api/')`); `web/src/api/health.ts:31`; `web/src/panels/DiagnosticsPanel.tsx:50` | `web/src/main.tsx:30` — extend the predicate to `p.startsWith('/api/') \|\| p === '/health' \|\| p === '/diagnose'` (the token must never be logged). |
| 8 | **S2** | Tool activity cannot be attributed to an agent and shows no result size/digest, even though the wire carries `task_id`, `bytes`, `sha256`; the chat tool strip therefore can never say "worker X ran tool Y". | `web/src/api/contract.ts:295-313`; dropped at `web/src/state/store.ts:1659-1674` and `:1709-1715`; record type `:114-122`; rendering `web/src/panels/ToolActivityPanel.tsx:157-177` | `web/src/state/store.ts:114-122` add `taskId/bytes/sha256`; populate at `:1659-1674` (task_id) and `:1709-1715` (bytes/sha256); render + group by task in `ToolActivityPanel.tsx:146-179` and use it for a per-card tool chip in `web/src/flow/nodes.tsx:193-224`. |
| 9 | **S2** | Run-memory bank entries stay stale after switching to another run with the same selected bank id: the effect depends on `selected` only while the URL embeds `runId`. | `web/src/panels/MemoryBanksPanel.tsx:245-304` (deps at `:304`), URL at `:253` | `web/src/panels/MemoryBanksPanel.tsx:304` — deps `[selected, runId]` and add an `AbortController` like `web/src/panels/ProposalsPanel.tsx:140-175`. |
| 10 | **S2** | The kill confirmation is the only modal without focus management: no initial focus, no Tab trap, **Escape does nothing**, background controls stay reachable (the shared ConfirmDialog already solves all of this). | `web/src/panels/ControlBar.tsx:145-181`; good pattern at `web/src/panels/ConfirmDialog.tsx:30-50` | `web/src/panels/ControlBar.tsx:145-181` — replace the inline block with `<ConfirmDialog danger confirmLabel="Kill worker" …/>` (`web/src/panels/ConfirmDialog.tsx:18-103`). |

### 9b. Next-tier findings (S3, still worth fixing)

| Finding | Evidence | Fix |
|---|---|---|
| Scrubber sends `HISTORY_REVISION_SET` on every key-up (Arrow/Tab/Shift) and can spam the server | `web/src/panels/Scrubber.tsx:58-59` | commit only on `onPointerUp`, `onKeyUp` for Arrow/Home/End, and `onBlur`; debounce 150 ms |
| ControlBar renders pending commands as **"Loading"** | `web/src/panels/ControlBar.tsx:183-189` | use `STRINGS.controls.pendingAck` new string, e.g. "Awaiting ACK" |
| AgentMessages answer box is one global draft; all Answer buttons disable while any row sends | `web/src/panels/AgentMessagesPanel.tsx:229-243` | per-row `drafts: Record<id,string>` + `sendingId` scoping (mirror `A2AInboxPanel.tsx:66,293-307`) |
| Empty-run copy contradicts the HyperAgent's existence: "No agents in this run", "No active agents", "READY · NO RUN YET" | `web/src/flow/Canvas.tsx:734-739`; `web/src/flow/graph.ts:395-396`; `web/src/panels/LibraryPanel.tsx:49,157-158`; `web/src/chat/ChatCore.tsx:299` | include the root as an agent in copy, or say "No child agents yet — the HyperAgent is the hive centre" |
| Amber used as 11–12 px text although declared large-text-only | `web/src/panels/ToolActivityPanel.tsx:58`; `web/src/tokens/tokens.ts:77-78` | use `text-ink-700` for the timeout label and keep amber for the border/icon |
| Reduced-motion override is not persisted (ambient slider is) | `web/src/state/store.ts:1926`, `:2494-2496`; ambient persistence `web/src/panels/SettingsPanel.tsx:291-300` | persist `hivemind.reduced-motion` and seed the store from it |
| Two reconnect strategies can fight: partysocket auto-reconnect (12 retries) plus the custom 1.5 s session refresh that closes the transport | `web/src/api/client.ts:111-118`, `:378-396`, `:424-464` | disable partysocket retries (`maxRetries: 0`) and let one recovery path own reconnect, or remove the custom refresh |
| Turning ambient motion **off** also stops the running-node state pulse, though state is not decoration | `web/src/styles/index.css:381-388` vs `web/src/styles/motion.css:11-15` | remove `.state-pulse` from the `data-ambient-motion="off"` list; keep it under reduced-motion only |
| No error boundary: one render throw blanks the whole app | `web/src/App.tsx:205-221` (no boundary anywhere) | wrap `<AppFrame/>` in a boundary that renders a named refusal + reload |
| Splash progress is fabricated (100/46/18) inside a real `progressbar` | `web/src/splash/HiveMindSplash.tsx:65`, `:117-126` | render an indeterminate bar (`aria-valuetext="Connecting"`, no `aria-valuenow`) or report only discrete named phases |
| Header agent search has no listbox semantics, no arrow keys, no outside-click close | `web/src/panels/Header.tsx:102-142` | `role="listbox"/"option"`, Arrow/Enter handling, `onBlur`/outside pointer close |
| Settings `aria-controls` points at panel ids that only exist for the active tab | `web/src/panels/SettingsPanel.tsx:421`, `:438-441` | keep one mounted panel per tab (hidden) or drop `aria-controls` for inactive tabs |
| Version string drift risk: hardcoded "HiveMind v1.2-dev" in Library | `web/src/panels/LibraryPanel.tsx:261` vs `web/package.json:4` ("1.2.0-dev") | read `__APP_VERSION__` from Vite `define` or remove the line |
| Data navigation has no path memory/breadcrumb after entering a directory | `web/src/panels/DataPanel.tsx:62-76` | show `fileTree.root`/current path + Up button; `FILE_TREE_LIST` already takes a path (`web/src/api/contract.ts:91-94`) |
| `Ctrl+F` silently also fits the swarm but is undocumented; Settings lists only `Ctrl+1` | `web/src/a11y/keyboard.ts:76-79`; `web/src/panels/SettingsPanel.tsx:26-35` | document it or drop the `f` branch |

---

## 10. Prioritized screen/component plan (additive 10–20%, no redesign)

Design authority: **FigCraft / HoneyGlass** — warm paper, honey/ink tokens, 1 px hairlines, one calm movement at a time, no neon, no new palette (`web/src/tokens/tokens.ts:22-43`; `web/src/styles/motion.css:1-34`). Every item below reuses existing tokens, components and store data. Nothing proposes pixel-fidelity claims; all are behaviour/composition changes.

### P0 — Restore trust and remove dead ends (est. 1–1.5 days)
1. **Fix the three lies:** Agents SPAWN reason (#1), workspace tabs (#2), Header chat toggle (#3). *Acceptance:* on a live run, every enabled/disabled control's reason matches the store gate; clicking Swarm from Settings lands on the Swarm tab; the chat button is either absent/disabled with a reason or navigates to the chat.
2. **Snapshot truth banner** (#4): a one-line strip under the mock/offline banners showing `Live · rev N` or `Unconfirmed · …`, `attempts`, `updatedAt`, with Re-sync. Uses existing `snapshot` + `isSnapshotDegraded`; no new state. *Acceptance:* forcing a failed `/api/history/runs` read shows the banner; recovery clears it.
3. **Approval queue** (#5): "1 of N" with Previous/Next and the existing evidence table; keep the focus trap. *Acceptance:* three seeded approvals are all reachable without resolving the first; Escape closes only when dismissal is legal.
4. **Replace kill modal with ConfirmDialog** (#10) and unify write-auth headers (#6) + extend the launch-token shim (#7). *Acceptance:* Escape cancels kill; one header helper used by every POST panel.

### P1 — Show the Hive's work (est. 2–3 days)
5. **Tool attribution + result evidence** (#8): add `taskId`, `bytes`, `sha256` to `ToolActivityRecord`; group the tool panel by agent; show a small honey `n tools` chip on worker/subagent cards; keep `args_sha256` (the wire has no args body — do not invent one, `contract.ts:295-302`). *Acceptance:* a run with two agents and three tool calls shows each call under its owner and a per-card count; unknown fields stay "—".
6. **Agents surface sub-navigation** (roster · tools · memory · messages · evolution) using the existing `role="tablist"` pattern from `SettingsPanel.tsx:406-436`; mount only the active sub-panel. Today mounting all nine fires ≥13 reads (`AgentsPanel.tsx:78-85` + each child's effect); sub-tabs cut that to ≤4 and remove the "diagnostic dump" feel. *Acceptance:* opening Agents issues no more than 4 requests; keyboard arrows switch sections.
7. **Run timeline strip** (new, additive, no new commands): horizontal honey timeline above the Scrubber using `history.revisions`, `run.status`, `toolActivity` turn numbers and approvals. Reuse `motion-progress-sheen` for the live marker. *Acceptance:* each committed revision is a tick; clicking a tick calls the existing `setHistoryRevision`; replay is visually distinct from live (existing `ReplayBanner`).
8. **Empty/root copy pass** (#S3): make every empty state distinguish "no run", "root only", "no children", "filtered out". Touches only `strings.ts` + the three call sites.

### P2 — Material and accessibility polish (est. 1–2 days)
9. **One motion system:** delete the legacy `hive-*`/`bee-*` duplicates in `web/src/styles/index.css:169-305` that `motion.css` supersedes, keep `motion-*` for the canvas, and confirm the canvas still breathes with `.motion-honeycomb-glow` + `.motion-hive-sheen` (`HiveField.tsx:87,192-194`). This reduces simultaneous infinite animations from ~6 to ~3 and makes "calm" meaningful.
10. **Splash honesty + focus isolation:** indeterminate progress (no invented %), `inert` on the app behind the veil or a focus sentinel, and token-var colours instead of raw `rgb()` in `splash.css:36-56`. Keep the glass/neumorphic material — it is the strongest brand moment.
11. **Contrast fix** for amber small text; add `forced-colors`/`prefers-reduced-transparency` fallbacks for `backdrop-filter`.
12. **Persist reduced-motion** and show the active source ("OS" vs "App") next to the toggle — no new state beyond one localStorage key.
13. **Perf hygiene:** pause `RemoteLane`/`SwarmPanel` 15 s polls when the document is hidden, and abort in-flight panel fetches on unmount (pattern already in `ProposalsPanel.tsx:140-175`).

**Explicitly out of scope (design authority / decision gates):** no code-diff surface (DIFF_* frozen), no new palette, no card geometry changes away from `tokens.ts:166-179`, no claim of pixel-fidelity, no replacement of the pinned chat overlay with a docked sidebar.

---

## 11. Explicit gaps, limits and open questions

1. **No runtime verification.** Nothing in this report was observed in a browser or against a running server. Rendering, focus order, toast stacking, animation cost and hover behaviour are **UNVERIFIED**.
2. **No test execution.** `npm test` was not run (lane is read-only and time-boxed). Coverage statements are from test file contents, not results.
3. **Server-side semantics unverified:** whether `/health`, `/diagnose`, `/api/templates`, `/api/projects`, history reads or Proposals accept the headers used; whether the provider profile expects `api_type: 'tokenlabs'` vs `tokenlab`; whether the server accepts `Authorization` or `x-hivemind-token`; whether revision numbers are sparse. Each is flagged at its call site.
4. **Partially inspected files:** `web/src/api/mock.ts` was read only to line 70 (gate/session/tree); its 368-line scenario body is not audited here — it is dev-only and boundary-pinned by `web/tests/apiBoundary.test.ts`. `web/src/figcraft/FigCraftGallery.tsx` was read to line 140 of 344; its preview/lightbox region is **UNVERIFIED**. `web/src/splash/HiveMindSplash.tsx` was read in full; `splash.css` only to line 120 of 340.
5. **No screenshot or design-board comparison** was performed; "material" statements cite token/CSS values, not rendered output.
6. **No diffs, no edits:** no source/config/test file was modified; no commit; no server; no provider call; no key printed.
7. **Open question for the orchestrator:** whether the launch-token shim scope (#7) and the Proposals header split (#6) are real server failures or harmless — resolving them needs the backend lane to state its auth policy per route.
8. **Open question:** `canAuthorizeProposal` requires `governance.can_apply === false` (`web/src/panels/ProposalsPanel.tsx:104-112`); if the server sets `can_apply` true when the four SuperAgent votes land, the operator button disappears exactly when it becomes legal. Needs backend confirmation.

---

## 12. Claims register

| Claim | Source | Status |
|---|---|---|
| Every mutation is ACK-gated and refusal-named | `web/src/state/store.ts:1843-1876`; `web/src/api/client.ts:466-514` | **VERIFIED** |
| Dev mock cannot ship in production | `web/src/api/index.ts:32,62-80`; `web/tests/apiBoundary.test.ts` | **VERIFIED** |
| 16-state vocabulary and 4-dot mapping are single-sourced | `web/src/state/agentState.ts:9-108`; `state.parity.test.ts` | **VERIFIED** |
| 20 printed swatches + only chrome-only exceptions | `web/src/tokens/tokens.ts:22-43,185-188`; `tokens.test.ts` | **VERIFIED** |
| Replay is read-only and never mutates live state | `web/src/flow/Canvas.tsx:250-305,566-607`; `graph.replay.test.ts` | **VERIFIED** |
| `snapshot` is never surfaced in any panel | grep over `web/src` (only store references) | **VERIFIED** (absence) |
| Agents-surface SPAWN always reports "Replay is read-only" | `web/src/panels/AgentsPanel.tsx:43`; `AgentHierarchy.tsx:79-82` | **VERIFIED** |
| Workspace tabs do nothing outside Workspace nav | `LibraryPanel.tsx:142`; `App.tsx:148-163`; `store.ts:2461-2463` | **VERIFIED** |
| Chat toggle is a no-op outside Workspace/workspace | `Header.tsx:50-67`; `App.tsx:141-146` | **VERIFIED** |
| Tool wire fields dropped by the store | `contract.ts:295-313`; `store.ts:1659-1674,1709-1715` | **VERIFIED** |
| Launch-token shim excludes `/health` and `/diagnose` | `main.tsx:30`; `health.ts:31`; `DiagnosticsPanel.tsx:50` | **VERIFIED** (code path); server impact **UNVERIFIED** |
| Amber small text violates its own large-only correction | `ToolActivityPanel.tsx:58`; `tokens.ts:77-78` | **VERIFIED**; measured ratio not re-computed here |
| Splash progress percentages are invented | `HiveMindSplash.tsx:65` | **VERIFIED** |
| No DOM/component interaction tests exist | `web/vite.config.ts:26-30`; 31 `.ts` specs | **VERIFIED** |
| The product "feels alive" | judgement from §6 | **PROPOSED/OPINION** (grounded in citations) |
| Any recommendation in §10 improves UX by 10–20% | judgement | **PROPOSED**, not measured |

---

*End of L5 report. Single write target: `docs/analysis/lanes/L5-ui-ux.md`.*

