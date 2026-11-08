# Round 9 — `/deep-frontend` seat: HiveMind frontend audit

**Seat:** AUDIT-ONLY frontend skill seat (no product code, test, or config edits).
**Date:** 2026-11-08. **Target:** `web/src` (React 19.3.1 + Vite 8.3.6 + Tailwind 4.3.4 + `@xyflow/react` 12.12.1 + zustand 5.0.16, versions read from `web/package.json`).
**Scope asked:** Canvas.tsx, graph.ts, ChatCore.tsx, Header.tsx, LibraryPanel.tsx, ApprovalOverlay.tsx, ControlBar, HistoryPanel/Scrubber, state/store.ts.
**Scope opened for context:** `App.tsx`, `a11y/keyboard.ts`, `chat/ChatMotion.tsx`, `flow/nodes.tsx`, `flow/HudBadge.tsx`, `state/agentState.ts`, `state/approval.ts`, `state/strings.ts`, `tokens/tokens.ts`, `backend/.../ws.rs` (only to establish when `HISTORY_SNAPSHOT` is emitted), `web/tests/*` (only to see what behaviour is already pinned).

## 0. Capability gates (skill procedure, checked before any judgement)

| Need | Check | Result |
|---|---|---|
| Read source | built in | **OK** — every claim below is a line-cited source read |
| Design tokens | `web/src/tokens/tokens.ts` + `styles/tokens.css` present | **OK** — no `E_NO_TOKENS`; no palette was invented |
| Data contract | typed props/contract in `web/src/api/contract.ts` | **OK** — no `E_NO_DATA` |
| Run dev server | **forbidden this round** | **`E_NOT_RUNNING`** — no render, no screenshot |
| Browser / screenshot | **forbidden this round** | **`E_NO_BROWSER`** — all layout, z-order and focus-order results below are **screenshot-unverified** |

**Verification vocabulary used in this document**
- `source` — the fact is read from the cited line; this is the only kind of evidence I have.
- `render` — **unverified**. The app is stopped, so no width, no tab-through, no pixel check was performed.
- Every row of the checklist is `source OK · render unverified`.

**Consequence to keep in mind:** "geometry overlap" claims (e.g. §FE-12) are computed from the Tailwind offsets and the `LAYOUT` constants, not observed. They are stated as *must be visually confirmed*, never as a screenshot fact.

---

## 1. Required component checklist

`element | semantic | keyboard | states | verified`

| element | semantic | keyboard | states | verified |
|---|---|---|---|---|
| `App` shell (`App.tsx:104-141`) | `<div>` + `<main>` + `<header>` + `<aside>` + `<nav>`; one `<main>`; landmarks correct | skip-link first (`App.tsx:106`), but `href="#hive-canvas"` targets an element that only exists in workspace/history-with-run (`Canvas.tsx:502`) | offline banner `role="alert"` (`App.tsx:120`); mock banner (`App.tsx:114-118`); no global loading state | source OK · render unverified |
| `Header` (`Header.tsx:23-144`) | `<header>` = banner landmark; search popover is a bare `<div class="surface-panel absolute">`, **no** `role="dialog"`, no listbox/combobox | toggle reachable; popover `autoFocus` input; **Escape closes but focus is not restored** to the toggle (`Header.tsx:112-116`) | connection badge `aria-live` (`Header.tsx:89`); search empty "No agents in this run." (`:122`); no loading; no result count | source OK · render unverified |
| `NavRail` (`NavRail.tsx:24-70`) | `<nav aria-label>` + `aria-current="page"`; roving tabindex | ArrowUp/Down move focus; **Enter/click required to activate**; Home/End missing | 6 destinations, always rendered | source OK · render unverified |
| `LibraryPanel` (`LibraryPanel.tsx:43-197`) | `<aside aria-label>`; `role="tablist"` with `role="tab"` + roving tabindex — but **no `aria-controls` and no `role="tabpanel"` anywhere in the app** | ArrowLeft/Right/Home/End implemented (`:58-75`); **Up/Down missing** on a 2×2 grid; tab chips are 12px targets | fixed width 240px; empty agents copy; `slice(0,8)` + "+N more" text with no action; templates always enabled (incl. offline/replay) | source OK · render unverified |
| `Canvas` root (`Canvas.tsx:500-508`) | `role="application"` + `aria-label`, `tabIndex={-1}` — **nested inside React Flow's own `role="application"` wrapper** (verified in `@xyflow/react` dist) | only a skip-link target; no arrow-key traversal of nodes implemented | empty overlay (`:787-793`) collapses "no run", "loading", "error", "no agents" into one sentence | source OK · render unverified |
| React Flow nodes (`nodes.tsx:141-292`) | `ariaLabel` per node (`graph.ts:244`), wrapper `role` from library; inner cards are real `<button>`s | one tab stop per node wrapper **plus** 1–2 per inner button (SuperAgent has two, `nodes.tsx:145,158`); `aria-pressed` used for single-select (`:150,204,284`) | selected/unselected border; progress `<div>` has **no `role="progressbar"`/`aria-valuenow`** (`nodes.tsx:112-139`); "—" for absent metrics | source OK · render unverified |
| React Flow edges (`Canvas.tsx:651-660`, `graph.ts:251-272`) | library supplies `role="group"`, `aria-roledescription="edge"` and a default name `"Edge from <uuid> to <uuid>"` — no label set by this app | `edgesFocusable` (`:660`) adds **one tab stop per edge named by two UUIDs** | dashed red "broken" variant; no label, no state text | source OK · render unverified |
| Minimap (`Canvas.tsx:693-722`, effect `:413-428`) | `ariaLabel` prop + a mount-time effect that writes `role="img"`/`aria-label` onto `.react-flow__minimap` | not focusable (fine); effect only runs at mount, so a re-mounted minimap loses the label | empty class `hive-minimap-empty`; fixed 220×150 | source OK · render unverified |
| Zoom cluster (`Canvas.tsx:733-785`) | four `<button aria-label>` with `aria-hidden` svg; container has `aria-label="Zoom controls"` on a **generic `div` (label not exposed — no role)** | 4 extra tab stops in tab order before the chat composer | zoom percent text is a plain `<span>`; no `aria-live` | source OK · render unverified |
| Canvas filter chips (`Canvas.tsx:609-650`) | 4 `<button aria-pressed>`; count is a `<span>` | reachable; `disabled` on Focus branch when nothing selected | **operate on live `tasks` while the canvas may be rendering replay** (`:174-200` vs `:651-653`): dead controls + wrong counts in replay | source OK · render unverified |
| `pendingReparent` dialog (`Canvas.tsx:579-608`) | `<div>` — **no `role="dialog"`, no `aria-modal`, no heading, no `aria-live`** | appears after a drag; focus is **not moved**, Escape not handled; buttons are tabbable but late in DOM | two buttons; labels fall back to "agent"/"target" when the ids no longer exist; no loading state while `rewireSelected` runs | source OK · render unverified |
| `ChatCore` panel (`ChatCore.tsx:192-378`) | `<section aria-label>` (region) + `<header>` + `<form>`; message list `aria-live="polite"` (`:240`) and `<article>` per message | composer reachable **after** every canvas node and edge in tab order (§FE-07); Enter submits, Shift+Enter newline (`:344-349`) | empty starter copy (`:243-269`); streaming dots + `role="status"` (`:323-330`); refusal block `role="alert"` (`:226-234`); long unbroken text overflows (§FE-34) | source OK · render unverified |
| Chat composer (`ChatCore.tsx:333-377`) | labelled `<textarea id="composer">` via `sr-only <label htmlFor>`; send/stop are `<button aria-label>` | textarea disabled when `connection !== 'open'` or replaying; no `aria-describedby` for why | draft 1-row textarea, no maxLength guard in UI (contract caps 8000, `contract.ts:16`); stop disabled for `thinking`/`planning` (§FE-32) | source OK · render unverified |
| `ApprovalOverlay` dialog (`ApprovalOverlay.tsx:221-381`) | `<section role="dialog" aria-modal="true" aria-label>` — **`aria-modal` without any focus management** | **no initial focus, no trap, no Escape, no focus restore** (§FE-01) | pending / in-flight / refusal+retry / expired / nonce-missing / unknown placeholders; expiry `aria-live` ticks every second (`:291`) | source OK · render unverified |
| Approval stale banner (`ApprovalOverlay.tsx:211-220`) | `role="status"`, `fixed top-0 z-50` | not focusable (fine) | only while `approvalsStale`; **overlays the 52px header** | source OK · render unverified |
| `ControlBar` (`ControlBar.tsx:41-126`) | `<div>` of buttons/selects, no group label | 6 tab stops; `title` carries disabled reasons (not exposed to keyboard/SR) | all five controls disabled together while any command is pending (`:27,33-38`); status text says **"Unavailable"** for any terminal run (`:122` + `strings.ts:193`) | source OK · render unverified |
| `Scrubber` (`Scrubber.tsx:42-75`) | `<footer>` inside `<main>` = footer landmark misused for a toolbar; `input type=range` with `<label htmlFor="scrubber">` | native range keys fire `onKeyUp` → commits; **commit is pointer-up/key-up only**, so a drag released off the input never commits (§FE-04) | disabled when no revision; `aria-valuetext` "n of m"; readout `aria-live="polite"` | source OK · render unverified |
| `HistoryPanel` (`HistoryPanel.tsx:18-68`) | `<aside aria-label>` + `<ol>` of `<button aria-pressed>` | tab through every revision button (no roving/arrow semantics for an exclusive list) | two honest empty states (`:20-24`); no loading state while a revision loads; `LATEST` derives from `history.latest` which can lag a newer commit | source OK · render unverified |
| `ToastRegion` (`ToastRegion.tsx:13-48`) | `aria-live="assertive"` container **and** `role="alert"` per toast = double announcement | dismiss button per toast | max 4 (store `pushRefusal` `store.ts:803-814`); `fixed right-4 top-16 z-50` collides with the header search popover | source OK · render unverified |

---

## 2. Findings

Severity is assigned against the skill's own bar ("a component with no keyboard path is not done") and against the repo's stated doctrine ("every mutating control is disabled unless its precondition is visibly true", `ControlBar.tsx:1-5`; "never a silent drop", `store.ts:1-6`).

### FE-01 — P0 — Approval modal: `aria-modal="true"` with no focus trap, no initial focus, no Escape, no restore
`web/src/panels/ApprovalOverlay.tsx:244-250` (dialog), `:177-185` (the only effect in the component), `:355-378` (decision buttons).

**Why it is a defect.** The component has exactly one `useEffect` and it only ticks a clock for the TTL. There is no ref, no `focus()`, no keydown handler, no `inert`/`aria-hidden` on the background, and the overlay is rendered as the last child of the app root (`App.tsx:139`). So the first Tab from a fresh page load lands on the skip link and walks **Header → NavRail → Library → every canvas node and edge → ControlBar → Scrubber** before it can reach Approve/Deny — and once focus is inside the dialog, Tab escapes into the still-focusable background while `aria-modal="true"` tells the screen reader that content does not exist. This is the single gated side-effect authorization surface in the product; a keyboard-only operator cannot reliably authorize or deny.

**Minimal safe fix candidate (not applied).** Add a `useEffect` on `approvals.length > 0` that (a) stores `document.activeElement`, (b) focuses the first enabled decision button (or the dialog container with `tabIndex={-1}`), (c) installs a keydown handler for Tab cycling inside the dialog and Escape → `dismissApproval` only when `actions.dismiss` is true (never when `pending`, since dismissing a live approval is not a decision), (d) restores focus on unmount. Background isolation can be a `data-approval-open` attribute on the app root plus `aria-hidden` on the three siblings, or a portal; do not change the store.

### FE-02 — P1 — The whole server-snapshot honesty surface is computed and never rendered; a failed *boot* refresh is completely silent
`web/src/state/store.ts:187-198` (state shape), `:1038-1255` (refresh), `:1211-1221` (write), `:1229-1248` (catch), `:1246-1248` (toast gate). Consumers: **none** — `grep` for `state.snapshot` across `web/src/**/*.tsx` returns zero hits (`snapshot`, `resyncGap`, `graphRev`, `viewport`, `diffOpen`, `chat.pinned` are all lifted and unread).

**Why it is a defect.** The store carries `status: 'idle'|'loading'|'ready'|'degraded'|'failed'`, `source`, `revision`, `incomplete: ['no-run'|'no-committed-revision'|'messages-unavailable'|'run-changed']` and `refusal` — a deliberately honest model — and no component reads a single field. The only places the user can learn that a reload failed to rehydrate are `Canvas.tsx:787-793` ("No agents yet — submit a prompt to create the first run") and `:790` ("No agents match the current filters"). And the catch at `:1246-1248` pushes a toast only when `source` is `resync|refusal|manual`; a `boot` failure writes `snapshot.status='failed'` and shows **nothing**. A reload while `GET /api/history/runs` is failing therefore renders an authoritative-looking empty canvas for a run that exists on the server, and the composer invites the operator to start a fresh run.

**Minimal safe fix candidate.** Add one read-only status strip in `App.tsx` under the offline banner that renders nothing when `status === 'ready'`, and otherwise names the state (`loading` / `degraded` + `incomplete` code / `failed` + `refusal.code`). Also include `boot` in the toast gate at `:1246`, or deliberately suppress the toast and rely on the strip.

### FE-03 — P1 — Any snapshot refresh silently drops the operator out of replay
`web/src/state/store.ts:1201-1210` (`replayGraph: null`), versus `:977-996` (`applyHistoryRevisionList` deliberately preserves `replayGraph` when the revision still exists).

**Why it is a defect.** Two different "refresh the history cursor" paths treat replay mode inconsistently. `performSnapshotRefresh` is called from boot, reconnect (`SERVER_HELLO`, `:1389`), resync (`:1282`), rewire fallback (`:1547`), and the rewire rev probe (`:1309,1318`). Any of those firing while the operator is scrubbing history replaces the historical projection with the live graph, resets `selected` to `latest`, and re-enables mutations — with no event and no toast. A `RESYNC_REQUIRED` arriving during replay can therefore make the operator believe they are still looking at revision 7 while they are looking at live state; the only cue is the replay pill at `Canvas.tsx:725-731` disappearing.

**Minimal safe fix candidate.** In the `history` block at `:1201-1210`, reuse the `applyHistoryRevisionList` rule: keep `replayGraph` and `selected` when `history.replayGraph !== null && list.revisions.includes(history.replayGraph.revision)`; otherwise fall back to `null`/`latest` and raise one named refusal so the forced exit is visible.

### FE-04 — P1 — Scrubber local mirror desynchronises from the store when a revision set is refused
`web/src/panels/Scrubber.tsx:21-25` (local `value`, reset effect keyed only on `selectedIndex`), `:57-59` (`onChange` moves the thumb only; `onPointerUp`/`onKeyUp` commit), `web/src/state/store.ts:2182-2203` (refusal paths return without changing `history.selected`).

**Why it is a defect.** The thumb position is local state; the graph position is `history.replayGraph`. When `setHistoryRevision` refuses (`E_HISTORY_GAP` at `:2195-2201`, `E_RUN_NOT_FOUND`, or a transport failure), `history.selected` does not change, so `selectedIndex` does not change, so the synchronising effect at `:23-25` does not re-run — the thumb stays at the refused position and the readout (`:61-65`) says "state 6 of 8" while the canvas still renders revision 2. Same desync when the pointer is released outside the input (mouse drag has no implicit capture), because nothing else commits. This is the primary time-travel control reporting a state that does not exist.

**Minimal safe fix candidate.** Derive the displayed value from `history.selected` (lifting the source of truth), keep a `dragging` flag that lets `onChange` move only while `onPointerDown` holds, commit in `onChange` (the store already guards duplicates by revision), and add `onBlur`/`onPointerLeave` commits for the released-outside case.

### FE-05 — P1 — The operator's typed-but-unsent prompt is destroyed by any view change
`web/src/chat/ChatCore.tsx:94` (`const [draft, setDraft] = useState('')`), `App.tsx:50-102` (tab/nav switch replaces `workspaceView`).

**Why it is a defect.** Messages, selection, chat visibility, settings and even `reducedMotionOverride` are lifted into the store, but the one piece of genuinely precious local state — the draft — is not. Switching nav (Workspace → Agents) or tab (Workspace → History) unmounts `ChatCore`; the draft is lost with no refusal, no toast and no recovery. The same component already demonstrates the right pattern for preserved state across hide/show (`chat.visible` early return at `:113` keeps the component mounted, so the draft survives *hiding* but not *navigating*).

**Minimal safe fix candidate.** Move `draft` into the store (`chat: { pinned, visible, draft }`) with `setChatDraft`, or hoist a `<ChatDraftProvider>` above the view switch. Keep `draftAfterAck`/`draftAfterRefusal` pure helpers exactly as they are.

### FE-06 — P2 — Reopening the chat does not scroll to the newest reply
`web/src/chat/ChatCore.tsx:106-111` (effect deps `[messages]`), `:113-125` (hidden branch renders a different subtree).

**Why it is a defect.** While `chat.visible` is false the message list is not mounted, so `listRef.current` is null and the effect no-ops. On reopen the list mounts fresh at `scrollTop = 0`; because the reply stream has not changed since the last scroll, the effect does not re-run, so a long transcript opens at the top and the live reply is off-screen. Recovery looks broken even though the state is intact.

**Minimal safe fix candidate.** Add `chat.visible` (and `toolActivity.length`) to the dependency list, or re-run the scroll in a `useLayoutEffect` that also fires when the list node identity changes.

### FE-07 — P1 — The chat composer is behind the entire graph in tab order
`web/src/flow/Canvas.tsx:659-660` (`nodesFocusable`, `edgesFocusable`), `web/src/flow/graph.ts:244` (per-node label) and `:251-272` (edges get no `ariaLabel`), `App.tsx:83-86` (`<Canvas />` then `<ChatCore />`).

**Why it is a defect.** DOM order puts the chat *after* the canvas, and React Flow makes every card focusable, every edge focusable, and every card exposes 1–2 nested buttons (`nodes.tsx:145,158`). With 30 nodes and 29 edges the composer sits behind ~90–120 tab stops; the skip link at `App.tsx:106` targets the canvas, not the composer, so there is no shortcut. The product's primary input is unreachable in practice for keyboard-only use. The edges make it worse: they contribute unnamed-but-for-UUIDs stops (`"Edge from <uuid> to <uuid>"`) that do nothing.

**Minimal safe fix candidate.** Two independent, low-risk moves: (a) render the chat container before the canvas in DOM order and give the canvas `order` via CSS if the visual stacking must stay; (b) set `edgesFocusable={false}` (edges are not actionable: `deleteKeyCode={null}`, no reconnect handlers) and/or give `buildEdges` a human `ariaLabel` and `focusable: false` for the anchor. Add a second skip link "Skip to composer" if DOM order cannot change.

### FE-08 — P1 — `pendingRewire` can outlive its command and then pin the wrong parent on someone else's rewire
`web/src/state/store.ts:1981-1991` (set before send, cleared only on a failed send), `:1291-1298` (`takePendingRewire` consumes by task id alone), `:1512-1519` (payload parent wins, otherwise the stale intent, otherwise the existing parent), `:1540-1548` (the authoritative snapshot fallback is **skipped** when an intent was consumed).

**Why it is a defect.** The intent is keyed only on `taskId` and has no origin, no timestamp and no expiry. If the server ACKs `REWIRE_TASK` but the matching `NODE_REWIRE` frame is lost, delayed past a snapshot refresh, or replaced by a second rewire (another tab, or a replay after reconnect), the stale intent is consumed by *that* frame and `parentId` is set to the operator's old target. The `intended !== null` branch also suppresses the refresh that would have learned the true edge, so the wrong edge becomes the rendered truth until the next snapshot. `exitHistoryReplay`, `RUN_CREATED` (`:1403-1425`) and `performSnapshotRefresh` do not clear it.

**Minimal safe fix candidate.** Store `{ taskId, newParentId, sentAt }`; consume only within a bounded window (e.g. the command timeout); clear on `RUN_CREATED`, on a settled snapshot refresh, and on any `NODE_REWIRE` that carries an explicit `parent_task_id` for a different task; keep the `performSnapshotRefresh('rewire')` fallback whenever the payload omits the parent (do not let a stale intent suppress it).

### FE-09 — P2 — The auto-fit effect is a "first graph" in its comment and a viewport hijack in its code
`web/src/flow/Canvas.tsx:355-376` (comment `:356-357`, call `:367`, deps `:376`).

**Why it is a defect.** The comment says "First graph: fit the whole swarm". The dependency list is `[runId, taskCount, hasTasks, history.replayGraph, fitView, chatVisible]`, so the effect re-fires on **every spawned task** (240 ms debounce) and on **every chat toggle**. During a live run the viewport re-centres and snaps to zoom 0.75 repeatedly while the operator is inspecting a card, and toggling the chat throws away pan/zoom. The `chatVisible` dependency exists only to change the padding constant.

**Minimal safe fix candidate.** Gate on a `hasFittedRef` (fit once per `runId`), and compute the chat padding at fit time only; do not include `chatVisible` in the deps.

### FE-10 — P1 — `nodes` is memoised without the `interactiveGraph` dependency, so cards stay draggable after the run ends
`web/src/flow/Canvas.tsx:202-221` (uses `interactiveGraph` at `:206`; deps at `:221` omit it), `:125` (`interactiveGraph` derives from `run.status`).

**Why it is a defect.** `RUN_STATUS` updates `run.status` and bumps `graphRev`, but it does not touch `tasks`. So when a run becomes terminal with no further `NODE_*` frame, `visibleTasks` is referentially unchanged, the memo does not recompute, and every node keeps `draggable: true` / `selectable: true` from the active-run render. The operator can keep dragging cards in a finished run; the drop then calls `rewireSelected`, which refuses with `E_RUN_TERMINAL` (`store.ts:708-710`) — the UI offers a mutation it knows is illegal, contradicting the `ControlBar` doctrine. There is no `eslint-plugin-react-hooks` in `web/package.json`, so nothing catches it.

**Minimal safe fix candidate.** Add `interactiveGraph` to the dependency array (and to `liveNodes` via `nodes`). Cheap and exactly correct.

### FE-11 — P1 — Below `lg` the only tab bar is `display:none`, so Swarm, Routing and History are unreachable
`web/src/panels/LibraryPanel.tsx:43-47` (`hidden … lg:flex`), `App.tsx:111-112` (Library is the only tab host), `NavRail.tsx:9-16` (nav sections only).

**Why it is a defect.** The workspace tabs (`workspace/swarm/routing/history`) live exclusively in `LibraryPanel`, which is hidden below 1024 px. `NavRail` switches `nav`, not `tab`; `Header` has no tab control. On a 768–1023 px window — a normal laptop with browser chrome, or any tablet — replay/time-travel and the swarm/routing views simply do not exist, and there is no alternative affordance. This is functional loss, not a layout squeeze, so it outranks the purely visual responsive risks.

**Minimal safe fix candidate.** Keep the tab list mounted at all widths and hide only its *body* (`hidden lg:block` on the project row, agents, templates and tip), or move the tab list into `Header`/`NavRail` for narrow widths. No store change.

### FE-12 — P2 — Three overlays share the same centred top anchor; the replay banner is under the HUD
`web/src/flow/Canvas.tsx:725-731` (replay pill, `left-1/2 top-4`, **no z-index**), `web/src/panels/HudBadge.tsx:15` (`left-1/2 top-3 z-20`, `width: 460, height: 44` from `tokens.ts:171`), `web/src/chat/ChatCore.tsx:115` (reopen button, `inset-x-0 top-4 z-30`).

**Why it is a defect.** Geometry: the HUD occupies y = 12→56 px, centred, 460 px wide; the replay pill occupies y ≈ 16→42 px, centred. The HUD is `z-20`, the pill has no z-index, so the pill paints **behind** the HUD in replay mode. The chat reopen button (`z-30`) then paints over both. Three independent "state" affordances collide in the same 44 px band, and the two lowest-priority ones (replay mode, chat hidden) are the ones that get hidden. The chat toggle is the newest overlord of a 44 px strip that also hosts two other controls. Visually unverified, but the offsets are exact.

**Minimal safe fix candidate.** One owner for the top-centre band: put the replay pill and the reopen affordance into a single flex column anchored at `top-3 z-30` above/below the HUD, or move the HUD to `top-16` when `history.replayGraph !== null`.

### FE-13 — P1 — N pending approvals render N identical full-screen modals at the same coordinates
`web/src/panels/ApprovalOverlay.tsx:221-244` (`approvals.map` → `div.fixed.inset-0.z-40.flex.items-center.justify-center` per approval).

**Why it is a defect.** Nothing serialises the queue. Each approval gets its own `fixed inset-0` scrim (`bg-ink-900/20`) and its own centred `min(560px,94%)` dialog, so with k > 1 pending approvals: the scrims composite to `k × 20%` darkening, k dialogs are stacked pixel-identically (only the last is visible), and the k−1 covered dialogs keep **focusable, invisible** Approve/Deny/Retry buttons in the tab order — a security-critical button you can press without seeing it. There is also no "1 of k" cue, so the operator cannot know a queue exists. This is a reachable state: the server replays up to 20 pending approvals after a hello (`backend/.../ws.rs:567-598`, `MAX_APPROVAL_REPLAY_EVENTS`).

**Minimal safe fix candidate.** Render one dialog for `approvals[0]` plus a "k−1 more waiting" line, or render a single dialog containing a list. Either way: one `fixed inset-0` scrim, and only the visible approval's controls in the tab order.

### FE-14 — P2 — `hideFinished` can delete a *visible* child's edge
`web/src/state/store.ts` → `web/src/flow/Canvas.tsx:378` (`buildEdges(visibleTasks)`), `web/src/flow/graph.ts:251-253` (drops any edge whose parent is not in the map), `Canvas.tsx:174-200` (the filter that removes terminal tasks).

**Why it is a defect.** The filter removes each terminal task individually, so "SuperAgent completed + its worker still running" leaves the worker visible and the parent gone; `buildEdges` then silently drops the edge because `tasks[parentId] === undefined`. The card floats with no connector and no explanation. `hideFinished` is a documented user-facing toggle (`:609-623`), so this is a main-path visual lie about topology. `ownerLinks` degrades the same way when the hive root is hidden (`:314-322`).

**Minimal safe fix candidate.** Keep structural parents when any non-terminal descendant is visible (mark them `hidden` rather than removing them), or re-source an orphaned visible child at `HIVE_ANCHOR_ID` when the drop is due to filtering. The anchor already exists for exactly this role (`graph.ts:29`).

### FE-15 — P2 — Canvas chrome keeps operating on live state while the canvas shows replay
`web/src/flow/Canvas.tsx:609-650` (chips always rendered), `:174-200` (`visibleTasks` reads live `tasks`), `:651-653` (nodes/edges switch to replay).

**Why it is a defect.** In replay, "Hide finished" and "Focus branch" still mutate `hideFinished`/`focusBranch`, still disable themselves from `Object.keys(tasks)`, and the `{visible}/{total}` counter (`:639-641`) shows the **live** counts next to a historical graph. Clicking does nothing visible; the numbers describe a different graph than the one drawn. "Show chat"/"Graph only" is fine. Same class of mismatch: `onPaneClick` (`:684`) clears the live selection while a replay graph is displayed.

**Minimal safe fix candidate.** Render the filter/count chips only when `history.replayGraph === null`, or make them operate on the replay projection and label the counts "live graph" explicitly.

### FE-16 — P1 — `REPLY_DELTA` is not run-filtered and can inject a foreign run's text and steal `streamingRunId`
`web/src/state/store.ts:1336-1346` (the run filter: `NODE_*` and `HISTORY_SNAPSHOT` only), `:1468-1485` (`REPLY_DELTA` appends or pushes a message for any `run_id` and sets `streamingRunId` to it).

**Why it is a defect.** The store explicitly assumes out-of-run frames can arrive ("A delayed replay from another run must never mutate this run's graph"), yet the chat arm has no equivalent guard: a delta whose `run_id` is not the active run and does not match the last streaming assistant message **pushes a new assistant message** with that run's text and sets `streamingRunId` to the foreign run. Symptoms: duplicated reply text after a replay, the working bee spinning for a finished run, the Stop button enabled for a run that is not there, and a transcript that no longer matches `run.id`. The existing test at `web/tests/streamingRender.test.ts:157` only covers *sequential* runs (new `RUN_CREATED` then new deltas), which is intended product behaviour (the conversation persists), so it does not contradict this finding.

**Minimal safe fix candidate.** Drop a delta when `get().run !== null && payload.run_id !== get().run.id`, or (narrower) when that `run_id` already has a non-streaming assistant message and no streaming one; never set `streamingRunId` to a run that is not `get().run.id`.

### FE-17 — P2 — Two rewire affordances accept different target sets; the drag path offers a worker parent
`web/src/flow/Canvas.tsx:152-172` (target kinds include `worker`), `web/src/panels/ControlBar.tsx:29-31` (parents are `superagent | hyperagent` only), `web/src/state/store.ts:704-721` (the shared blocker checks run/selection/target/rev — not depth or cycle).

**Why it is a defect.** The same `REWIRE_TASK` command is offered by two widgets with different preconditions: the ControlBar refuses to offer a worker as a parent, the canvas drop happily proposes "Connect worker-A under worker-B". The store's `rewireBlocker` has no cycle or depth guard, so those tries can only fail server-side (`E_CYCLE` / `E_SPAWN_DEPTH`) and produce a toast after the operator has confirmed a dialog. The repo's own rule is that a mutating control is disabled unless its precondition is visibly true.

**Minimal safe fix candidate.** Export the allowed-parent predicate from the store (one function, used by `ControlBar.parents` and `findDropTarget`), and extend `rewireBlocker` with `target-is-descendant` so the dialog never appears for a cycle.

### FE-18 — P2 — Header search popover: no click-outside close, Escape drops focus to `<body>`
`web/src/panels/Header.tsx:102-142` (popover), `:107-119` (`autoFocus` input), `:112-116` (Escape).

**Why it is a defect.** The popover is a bare absolutely-positioned `<div>` with no dialog/listbox role and no `aria-controls` from the toggle. Escape closes it, which unmounts the focused input, so focus falls back to `<body>`; the next Tab restarts at the top of the document instead of returning to the search button. Clicking anywhere else leaves it open (no backdrop, no outside-click handler), so it can sit over the canvas indefinitely. 320 px wide anchored `right-3` also overflows a 320 px viewport.

**Minimal safe fix candidate.** Keep a ref on the toggle; call `toggleRef.current?.focus()` on both close paths, add a `pointerdown` outside handler, and give the container `role="dialog"` + `aria-label` with the input's `aria-controls` pointing at the result list.

### FE-19 — P2 — Fixed pixel geometry overflows small viewports (four concrete cases)
`web/src/panels/HudBadge.tsx:15-22` (460 px, `whitespace-nowrap`, centred), `web/src/chat/ChatCore.tsx:195` (`h-[480px]` with `items-center`, no scroll), `web/src/panels/ApprovalOverlay.tsx:249` (`w-[min(560px,94%)]` but the `dl` has 8 rows and the overlay has no `overflow-y-auto`), `web/src/panels/ControlBar.tsx:41` (`flex-wrap` inside a bottom overlay), `web/src/tokens/tokens.ts:171-172` (`hud: 460×44`, `minimap: 220×150`).

**Why it is a defect.** At 360×640 the HUD is wider than the viewport; the chat panel is 480 px tall in a ~540 px content area with `items-center` and no `max-h`, so the composer is clipped; the approval dialog's content (8 definition rows + up to 4 buttons) exceeds a 600 px-tall window with no scroll container, making Deny/Approve unreachable; and the wrapping ControlBar can grow to three rows over the canvas and the zoom cluster (`Canvas.tsx:733`, `bottom-32 left-3`). Narrow widths also lose the tab bar (§FE-11), so this is the second-order part of the same responsive story.

**Minimal safe fix candidate.** `max-h-[calc(100dvh-1rem)]` + `overflow-y-auto` on the chat section and the approval dialog; `w-[min(460px,calc(100vw-1rem))]` on the HUD; `flex-wrap` + `max-w-[calc(100vw-1rem)]` on the Canvas chip row (`:609`); `bottom-[max(8rem,var(--controlbar-h))]` or a grid layout for the canvas overlays instead of `bottom-32`.

### FE-20 — P2 — Canvas root is a second `role="application"` nested inside React Flow's own
`web/src/flow/Canvas.tsx:505` (`role="application"`), and verified in `web/node_modules/@xyflow/react/dist/esm/index.js` (`rf__wrapper` already renders `role: "application"`).

**Why it is a defect.** Two nested application roles is an accessibility anti-pattern: the outer one puts *everything* inside it (zoom buttons, chips, HUD, the pending-reparent dialog, the minimap) into an application-mode region that the inner wrapper already declares, and the outer name ("HiveMind spatial canvas") competes with the inner one. Nothing in the app implements application-mode keyboard semantics (no arrow-key traversal, no roving node focus) to justify the role.

**Minimal safe fix candidate.** Outer container → `role="region"` + `aria-label` (or a labelled `role="group"`), leaving React Flow's wrapper to be the single application.

### FE-21 — P2 — Edge and card keyboard stops are mislabelled or duplicated
`web/src/flow/Canvas.tsx:659-660` (`nodesFocusable`/`edgesFocusable`), `web/src/flow/graph.ts:251-272` (edges: no `ariaLabel`, no `focusable`), `web/src/flow/nodes.tsx:145-170` (SuperAgent: two buttons for one action), `:112-139` (`Progress` is a plain `<div>`), `:150,204,284` (`aria-pressed` on single-select).

**Why it is a defect.** (a) Edges are focusable but never actionable, and their library-generated name is two UUIDs. (b) A SuperAgent card is two tab stops for the same action ("label" and "body"), while every other card is one — the tab sequence changes shape by kind. (c) Progress bars expose no value to assistive tech even though the value is the point of the card. (d) `aria-pressed` is toggle semantics on an exclusive selection; `aria-selected`/`listbox` or plain `aria-current` is the honest mapping. All four are cheap, mechanical accessibility debt in the most-used surface.

**Minimal safe fix candidate.** `edgesFocusable={false}` (or label them "Edge from A to B" with task labels); one button per card; `<div role="progressbar" aria-valuenow={v} aria-valuemin={0} aria-valuemax={100}>`; drop `aria-pressed` in favour of `aria-current="true"` on the selected card.

### FE-22 — P2 — ControlBar's first select is labelled as the rewire target but mutates selection
`web/src/panels/ControlBar.tsx:43-56` (`value={selectedTaskId}`, `onChange → selectTask`, `aria-label={`${STRINGS.node.workerLabel}: ${STRINGS.controls.reroute}`}` = "WORKER: Re-route"), `:70-82` (the actual target select).

**Why it is a defect.** Two adjacent `<select>`s: the first reads "Target" (visible `<span>` at `:42`) and its accessible name is "WORKER: Re-route", but it does **not** choose the rewire target — it changes the global selection. The real target select is unlabelled by the visible "Target" text and is separately named "Re-route". Blank option in the selection select is `STRINGS.node.noMetrics` (`"—"`), i.e. a metrics placeholder used as "nothing selected". A screen-reader user hears two differently-named comboboxes and cannot tell which one the Re-route button consumes (`:90` uses `target`, not the selection).

**Minimal safe fix candidate.** Rename the first select "Selected node" with its own visible label, remove the composed `aria-label`, move the visible "Target" text to the second select, and give the empty option an explicit "No target" string in `STRINGS`.

### FE-23 — P2 — Terminal run status renders as "Unavailable"
`web/src/panels/ControlBar.tsx:118-124` (`isTerminal(run.status) ? STRINGS.states.unavailable : run.status`), `web/src/state/strings.ts:193` (`unavailable: 'Unavailable'`).

**Why it is a defect.** A run that completed successfully, or failed by name, is reported in the control bar as "Unavailable" — the UI loses the single most important fact (how the run ended) exactly where the operator looks for it. The branch conflates "this value is not displayable" with "the run is over".

**Minimal safe fix candidate.** Render `STRINGS.agentStates[run.status]` directly (the vocabulary already has labels for all 16 states via `visualFor`), and use `unavailable` only for genuinely absent data.

### FE-24 — P2 — Approval dialog: 1 Hz `aria-live` countdown, and a silently-swallowed click
`web/src/panels/ApprovalOverlay.tsx:291` (`<dd aria-live="polite">` around the expiry text, refreshed every 1000 ms at `:181-185`), `:191-208` (`submit` re-evaluates the gate and `return`s with no feedback when `!enabled`).

**Why it is a defect.** A polite live region that changes once a second will queue an announcement per second in most screen readers — the dialog becomes unusable for the exact users the `aria-live` was meant to help; the countdown is already visible. Separately, the render path evaluates the gate with the 1 Hz `now` state while `submit` evaluates with `Date.now()`; in the window where the approval just expired, the button still looks enabled, the click is swallowed at `:200-202`, and nothing is named. That is the "silent no-op" the rest of the app bans.

**Minimal safe fix candidate.** Drop `aria-live` from the countdown (or switch to `aria-live="off"` plus a `role="timer"` on the wrapper) and disable the buttons from the same clock the render uses; when `submit` refuses, push the named refusal through the existing `clientRefusal` path instead of returning silently.

### FE-25 — P2 — Approval chrome overlaps the header; toasts overlap the search popover
`web/src/panels/ApprovalOverlay.tsx:211-220` (`fixed inset-x-0 top-0 z-50`), `web/src/panels/Header.tsx:23` (`h-[52px]` normal-flow header), `web/src/panels/ToastRegion.tsx:16` (`fixed right-4 top-16 z-50`), `Header.tsx:103` (`absolute right-3 top-[48px] z-50`).

**Why it is a defect.** The stale-approval banner is pinned to `top-0` at `z-50`, so it covers the top 32-ish px of the 52 px header (brand and, at narrow widths, the header buttons) whenever approvals are unconfirmed. The toast stack and the search popover are both `z-50` in the same top-right corner; the toast wins on DOM order, so a refusal toast covers the open search results. Both are recoverable, both look like a rendering bug to the operator.

**Minimal safe fix candidate.** Render the banner into normal flow above `<Header/>` (it is a global condition), and push the toast stack clear of the popover (`top-16` → `top-[calc(48px+320px)]` only while the popover is open is worse; simpler: give the popover `z-40` and keep toasts at `z-50`, accepting that toasts win).

### FE-26 — P2 — Replay has three different exits, two of them identically labelled
`web/src/panels/Header.tsx:41-49` ("Return live"), `web/src/panels/HistoryPanel.tsx:57-64` (`STRINGS.scrubber.live`), `web/src/panels/Scrubber.tsx:66-73` (same string), `App.tsx:125-135` (Scrubber shown on both the workspace and history tabs).

**Why it is a defect.** Three controls, one action, two labels, two visual placements, and in the history tab two of them are on screen simultaneously. The HistoryPanel one is `<button aria-pressed={selected}>` per revision — toggle semantics for an exclusive set — and its `LATEST` tag compares against `history.latest`, which only advances on terminal/refresh (`store.ts:1460-1465`), so a still-running run can label revision 3 as LATEST while the server has 4.

**Minimal safe fix candidate.** One exit (the Header pill), one disabled reason, and change the revision buttons to `aria-current`/`radio` semantics with `history.revisions` as the only tag source ("NEWEST" only when it equals the last committed revision).

### FE-27 — P2 — `insertTemplate` is not gated like every other mutation
`web/src/panels/LibraryPanel.tsx:144-156` (`onClick → void insertTemplate(...)`), `web/src/panels/ControlBar.tsx:33-38` (every other mutation gated on `connected && runActive && !anyPending && !replaying`), `web/src/state/store.ts:2175-2180` (no guard).

**Why it is a defect.** While replaying a historical revision, or with the socket closed, the four template buttons still fire `TEMPLATE_INSERT` against the live run. The store then either sends a command that the server will refuse (toast) or, worse, mutates the live graph behind the read-only replay view — the exact confusion replay mode is supposed to prevent. Every other mutating control in the app is disabled in those states.

**Minimal safe fix candidate.** Derive `canInsert = connected && runActive && !replaying && !anyPending` in the panel and `disabled` the four buttons; add the same guard at the top of `insertTemplate` so the store is safe regardless of caller.

### FE-28 — P2 — Side effect inside a React state updater; drag offsets survive a rewire
`web/src/flow/Canvas.tsx:669-683` (`setPositionOverrides(previous => { … savePositions(runId, next); return next; })`), `:265-282` (the same drop also arms the reparent prompt).

**Why it is a defect.** `savePositions` is a `localStorage` write performed inside an updater; updaters must be pure and are invoked twice under StrictMode/React 19 concurrent rendering, so the write is duplicated (harmless) but the pattern is a correctness hazard the codebase otherwise avoids. Worse semantically: the drop position is persisted **before** a reparent is confirmed, so a node that is then re-parented (and therefore re-laid-out by `columnHiveLayout`) keeps the operator's stale drag offset forever, floating away from its new sibling stack, and the offset is persisted per run id so it survives reloads. Offsets for deleted tasks are never pruned, so `localStorage` grows for the life of the run.

**Minimal safe fix candidate.** Compute `next` outside the updater and call `savePositions` in an effect keyed on `positionOverrides`; skip recording an override when `dropTargetId !== null` (a reparent, not a placement); prune overrides whose task id is no longer in `tasks` when loading/saving.

### FE-29 — P2 — Chat state copy and stop-gate disagree with the rest of the app
`web/src/chat/ChatCore.tsx:243-269` (empty state "Type a prompt to create the first run… READY · NO RUN YET" rendered regardless of connection), `:353-365` (Stop disabled unless `streamingRunId !== null || runStatus === 'running'`), `web/src/state/agentState.ts:121-123` (`isRunActive` also true for `thinking`, `planning`, `awaiting_approval`), `web/src/panels/ControlBar.tsx:36` (Cancel uses `runActive`).

**Why it is a defect.** With the socket closed the chat still presents "READY · NO RUN YET" and a disabled composer with no in-place reason (the reason is a separate banner at the app root). And a run that is `thinking`/`planning`/`awaiting_approval` is active for Cancel but **not cancellable from the chat** — the stop button is disabled at exactly the moments a run is most likely to need stopping, while the ControlBar offers Cancel for the same state. Two preconditions for one intent.

**Minimal safe fix candidate.** Use `isRunActive(runStatus)` for the stop button (import it — Canvas already does at `Canvas.tsx:29`); make the empty-state copy connection-aware (`connection !== 'open'` → "Waiting for the local server"), and keep "READY · NO RUN YET" only when open.

### FE-30 — P2 — ChatCore bypasses the store's refusal pipeline (and uses a Web-Crypto API directly)
`web/src/chat/ChatCore.tsx:141-150` (`store.setState({refusals: [...current.refusals, { id: crypto.randomUUID(), … }]})`), `web/src/state/store.ts:803-814` (`pushRefusal` dedupes within 500 ms and caps at 4), `web/src/api/ids.ts` (`uuidv7` is the store's id source).

**Why it is a defect.** The `E_KEY_MISSING` pre-check writes the refusal list directly, bypassing the 500 ms dedupe and the 4-entry cap — repeated Enter presses stack identical toasts — and it does not go through `inFlightRef`, so it is also outside the double-submit guard. It is the only non-store writer of store state in the app. `crypto.randomUUID()` is only defined in a secure context; the store's own `uuidv7()` exists and is used everywhere else, so this is an unnecessary portability cliff if the UI is ever served from a LAN address.

**Minimal safe fix candidate.** Export a `raiseRefusal(code, overrides?)` action from the store and call it; use `uuidv7()`. No behaviour change beyond dedupe/cap.

### FE-31 — P2 — Long unbroken content escapes the chat bubble
`web/src/chat/ChatMotion.tsx:166-172` (`whitespace-pre-wrap` text block, no `break-words`/`overflow-wrap`), `web/src/chat/ChatCore.tsx:239` (list has `overflow-y-auto` only).

**Why it is a defect.** A reply containing a long token (URL, base64, hash, minified line) is wider than the bubble; with no `break-words` it overflows horizontally out of the bubble and, because the scroll container only scrolls on Y, the content is simply clipped off-panel with no way to read it. Every other long-content surface in this codebase handles it (`ApprovalOverlay.tsx:264` uses `break-words whitespace-pre-wrap`, `CodeBlock` uses `overflow-x-auto` at `ChatMotion.tsx:149`).

**Minimal safe fix candidate.** Add `break-words [overflow-wrap:anywhere]` to the text block at `ChatMotion.tsx:166-168`; no structural change.

### FE-32 — P2 — Replay *projection* has no cycle guard; layout relies on the graph being a DAG
`web/src/flow/graph.ts:66-73` (`measure` recurses through `childrenByParent` with no visited set), `:75-84` (`placeColumn`), `:280-329` (`buildReplayTasks` re-roots *missing* parents but does not detect cycles), `web/src/state/store.ts:582-645` (`recordsFromSnapshotGraph`, same gap), `web/src/flow/Canvas.tsx:186-194` (`walk` is safe because of its `keep` set).

**Why it is a defect.** Both ingest paths are documented as fail-closed for malformed snapshots, but "malformed" is only checked per node. A snapshot whose parents form a cycle (`a.parent = b`, `b.parent = a`, or a self-parent) passes both decoders and then makes `measure`/`placeColumn` recurse until the stack dies — a crash of the whole canvas (React error boundary absent) from a server payload. The live path cannot easily produce a cycle today (the server refuses `REWIRE_TASK` with `E_CYCLE`), but the replay/ingest path can, and it is the path the UI trusts as "authoritative".

**Minimal safe fix candidate.** In `buildReplayTasks` and `recordsFromSnapshotGraph`, walk parent chains with a visited set and re-root (`parentId: null`) any node whose chain revisits a node; add a defensive depth cap in `measure`. Pure functions, unit-testable.

### FE-33 — P2 — Dead lifted state and dead commands
`web/src/state/store.ts:281` (`viewport`), `:287` (`diffOpen`), `:282` (`chat.pinned`), `:262-264` (`snapshot`, `resyncGap`), `:2236-2238` (`setViewport`), `:2168-2173` (`pinChat`), `:2253-2259` (`openDiffModal`/`closeDiffModal`).

**Why it is a defect (local vs lifted).** `Canvas.tsx:687` writes `setViewport` on every `onMoveEnd` into a slice **no component reads** — a store update on every pan/zoom-end for nothing. `diffOpen` has no consumer and no modal; `pinChat`/`chat.pinned` have no UI; `openDiffModal` is never called. This is the inverse of the draft bug (§FE-05): state that should be local (`zoomLabel`, done correctly) and state that should not exist are lifted and paid for. `snapshot` is the one lifted slice that *should* be rendered (§FE-02).

**Minimal safe fix candidate.** Delete `setViewport`/`viewport`, `diffOpen`/`openDiffModal`/`closeDiffModal`, and `pinChat`/`pinned`; or wire them. Do not leave half-wired store surface — it is what made §FE-02 invisible.

### FE-34 — P2 — No linter, so hook-dependency and unused-export drift is unenforced
`web/package.json` (scripts: `dev`, `typecheck`, `build`, `preview`, `test`; no ESLint, no `react-hooks` plugin), which is why §FE-10 and the `chatVisible` dependency in §FE-09 are invisible to CI.

**Why it is a defect (process, but with code consequences).** `tsc --noEmit` cannot see a missing effect dependency, a stale memo closure, or an unused export, and the project has zero of those checks. Both hook defects found here are of exactly that shape.

**Minimal safe fix candidate.** Add `eslint` + `eslint-plugin-react-hooks` with `react-hooks/exhaustive-deps: error` and run it in `typecheck`. No runtime change.

### FE-35 — P2 — Approval tests assert source text instead of behaviour, and one rule is implemented twice
`web/tests/approval.integration.test.ts:265-267` (`expect(overlaySource).toContain('isApprovalExpired' | 'canResolveApproval' | 'prunePendingResolutions')`), `web/src/panels/ApprovalOverlay.tsx:101-110` (local `isApprovalExpired`) vs `web/src/state/approval.ts:183-195` (the real one), `ApprovalOverlay.tsx:129-144` (`prunePendingResolutions` has no production caller — the panel keeps no pending lock, so the file's header comment at `:9-13` describing that lock is stale).

**Why it is a defect.** Those three assertions pass because the identifiers are *defined in the file*, not because the overlay *uses* them; the overlay's actual gate is `approvalActionsEnabled` (`:193-199`, `:224-229`). So the test suite gives false assurance about the fail-closed rules, and two copies of `isApprovalExpired` (with identical logic today) can drift silently. Not a live behaviour bug — a guardrail bug.

**Minimal safe fix candidate.** Replace the source-text assertions with a rendered-behaviour assertion (or at minimum assert the component references `approvalActionsEnabled`), delete the duplicate `isApprovalExpired`, and either use or delete `prunePendingResolutions` + fix the header comment.

---

## 3. Ranking

### P0 — blocks the security-critical action for keyboard users
| # | Finding | One-line |
|---|---|---|
| 1 | FE-01 | Approval dialog declares `aria-modal` but has no initial focus, no trap, no Escape, no restore; it is the last node in the DOM, so the decision buttons sit behind the whole app in tab order. |

### P1 — silently misrepresents authoritative state, loses operator input, or removes a control
| # | Finding | One-line |
|---|---|---|
| 2 | FE-02 | `snapshot`/`resyncGap` are computed and never rendered; a failed **boot** rehydrate is fully silent while the canvas asserts "No agents yet". |
| 3 | FE-03 | `performSnapshotRefresh` nulls `replayGraph`; any reconnect/resync/rewire during replay silently throws the operator back to live. |
| 4 | FE-04 | Scrubber mirror desyncs on a refused revision set (and on pointer-up outside the input): the thumb reports a revision the canvas is not showing. |
| 5 | FE-05 | The typed draft is local to `ChatCore`; any nav/tab change destroys it with no recovery. |
| 6 | FE-07 | Chat composer is behind every node and every (UUID-named) edge in tab order; the only skip link goes to the canvas. |
| 7 | FE-08 | `pendingRewire` is keyed only on `taskId` and never expires; a later rewire frame can inherit the wrong parent and suppress the authoritative refresh. |
| 8 | FE-10 | `nodes` memo omits `interactiveGraph`: cards stay draggable after the run ends. |
| 9 | FE-11 | Below `lg` the only tab bar is `display:none`; Swarm/Routing/History become unreachable. |
| 10 | FE-13 | N pending approvals render N pixel-identical full-screen modals; covered dialogs' Approve buttons remain focusable. |
| 11 | FE-16 | `REPLY_DELTA` lacks the run guard the `NODE_*` arm has: a foreign run's text is appended and steals `streamingRunId`. |

### P2 — real but lower blast radius, or conditional
| # | Finding | One-line |
|---|---|---|
| 12 | FE-06 | Reopened chat opens at the top; the newest reply is off-screen. |
| 13 | FE-09 | Auto-fit re-fires on every spawned task and every chat toggle (comment says "First graph"). |
| 14 | FE-12 | HUD (z-20, top-3, 460px) paints over the replay pill (top-4, no z); reopen button covers both. |
| 15 | FE-14 | `hideFinished` can delete a visible child's edge (parent filtered, edge dropped). |
| 16 | FE-15 | Canvas filter chips + counts operate on live state while the canvas renders replay. |
| 17 | FE-17 | Canvas drag accepts `worker` parents; ControlBar does not; no cycle guard in the shared blocker. |
| 18 | FE-18 | Header search: no outside-click close, Escape drops focus to `<body>`, 320px at `right-3` overflows narrow viewports. |
| 19 | FE-19 | Fixed 460/480/560px geometry + no scroll containers overflow small viewports. |
| 20 | FE-20 | Nested `role="application"` (app adds one inside React Flow's own). |
| 21 | FE-21 | Edge stops with UUID names; SuperAgent is two tab stops; progress bars have no `role="progressbar"`; `aria-pressed` on exclusive selection. |
| 22 | FE-22 | "Target" select changes selection, not the rewire target; two comboboxes with swapped labels. |
| 23 | FE-23 | Terminal runs display as "Unavailable". |
| 24 | FE-24 | 1 Hz `aria-live` countdown; clicks swallowed silently when the TTL gate flips. |
| 25 | FE-25 | Stale-approval banner covers the header; toasts cover the open search popover. |
| 26 | FE-26 | Three replay exits, two labels; `LATEST` can lag a new commit. |
| 27 | FE-27 | `insertTemplate` fires during replay/offline while every other mutation is gated. |
| 28 | FE-28 | `localStorage` write inside a state updater; drag offsets persist across a rewire. |
| 29 | FE-29 | Chat "READY · NO RUN YET" while offline; stop button disabled for `thinking`/`planning`/`awaiting_approval`. |
| 30 | FE-30 | ChatCore writes `refusals` directly (bypasses dedupe/cap) and uses `crypto.randomUUID`. |
| 31 | FE-31 | Long unbroken reply text overflows the bubble with no horizontal path. |
| 32 | FE-32 | Replay/snapshot ingest has no cycle guard; layout recurses without a visited set. |
| 33 | FE-33 | Dead lifted state (`viewport`, `diffOpen`, `pinned`) + a store write per pan-end that nothing reads. |
| 34 | FE-34 | No linter; hook-dependency defects are structurally invisible to CI. |
| 35 | FE-35 | Approval tests assert source text; `isApprovalExpired` implemented twice; `prunePendingResolutions` unused. |

---

## 4. Taste, not bugs (recorded, deliberately not ranked)

These are design decisions I would argue about, not defects. They are separated so the P-list stays honest.

1. **Ctrl+F and Ctrl+1 are hijacked to fit-view** (`a11y/keyboard.ts:68-79`, global capture at `Canvas.tsx:409`). It is documented (R-U7) and it does not fire inside inputs, but it takes away browser Find on the canvas — the exact place a user might want it. Product decision.
2. **`fitView` is pinned to zoom 0.75** (`Canvas.tsx:367,690`; comment "defaults 0.5/2 overridden, R-U7"). "Fit" that refuses to zoom past 0.75 means a 3-node swarm is shown at 75 %, not fitted. Spec'd, so taste.
3. **`readonly` cache-busting `graphRev` and `agentMessageVersion`** (`store.ts:254,258`) exist purely to force re-renders through zustand. Works; unusual; not a bug.
4. **Fixed 240px library, 72px rail, 460px HUD** (`tokens.ts:167-172`) rather than container queries. Against the skill's mobile-first preference, consistent with the 1440×900 parity board.
5. **`<footer>` for the scrubber** (`Scrubber.tsx:42`) — I would use `role="group" aria-label`, but inside `<main>` the footer element is only a generic footer, so nothing is broken for AT today.
6. **`aria-pressed` on the revision buttons and the chat toggle** — readable as toggles, and the chat toggle *is* a toggle. Fine.
7. **Chat panel is `position:absolute; inset:0` centred over the canvas** — the hive-core metaphor requires it. Nodes are laid out around a reserved box (`graph.ts:166-222`), so nothing important hides behind it. Design intent.
8. **Three replay exits** — the duplication is a bug (§FE-26) but having both a persistent scrubber and a history-panel list is legitimate.

---

## 5. Bugs to add to known areas

Register rows in the repo's `BUG-<AREA>-<n>` style (`plans/verification/LEAD-BUGCHECK.md` uses `BUG-BOOT-1/2`). "New" means I found no existing register or ledger row for it; "extends" means it sharpens something already noted in `audit/FINDINGS-REGISTER.md` / `ledgers/ui-watchdog.md`, which I did not find this class of defect in.

| id | area | path:line | defect | severity |
|---|---|---|---|---|
| `BUG-UI-01` | approval a11y | `web/src/panels/ApprovalOverlay.tsx:244-250` | modal with no focus entry/trap/Escape/restore; decision controls last in tab order | P0 |
| `BUG-UI-02` | snapshot honesty | `web/src/state/store.ts:187-198,1211-1221,1246-1248` | snapshot/resync state has no renderer; boot refresh failure silent | P1 |
| `BUG-UI-03` | replay vs live | `web/src/state/store.ts:1201-1210` | snapshot refresh nulls `replayGraph`; silent exit from replay | P1 |
| `BUG-UI-04` | time travel | `web/src/panels/Scrubber.tsx:21-25,57-59` | local thumb mirror desyncs when the revision set is refused or pointer-up is missed | P1 |
| `BUG-UI-05` | chat recovery | `web/src/chat/ChatCore.tsx:94` | unsent draft destroyed by nav/tab change (not lifted) | P1 |
| `BUG-UI-06` | chat recovery | `web/src/chat/ChatCore.tsx:106-111` | reopen does not scroll to newest reply | P2 |
| `BUG-UI-07` | focus order | `web/src/flow/Canvas.tsx:659-660` + `App.tsx:83-86` | composer behind every node/edge tab stop; skip link points at the canvas | P1 |
| `BUG-UI-08` | pending rewire | `web/src/state/store.ts:1291-1298,1512-1519,1981-1991` | `pendingRewire` never expires/clears; stale intent wins a later rewire and suppresses the snapshot refresh | P1 |
| `BUG-UI-09` | canvas interactivity | `web/src/flow/Canvas.tsx:202-221` | memo missing `interactiveGraph`; cards stay draggable after the run ends | P1 |
| `BUG-UI-10` | responsive | `web/src/panels/LibraryPanel.tsx:45` | tab bar hidden below `lg`; Swarm/Routing/History unreachable | P1 |
| `BUG-UI-11` | z-order | `web/src/flow/Canvas.tsx:725-731` + `HudBadge.tsx:15` + `ChatCore.tsx:115` | three top-centre overlays collide; replay pill painted under the HUD | P2 |
| `BUG-UI-12` | approval queue | `web/src/panels/ApprovalOverlay.tsx:221-244` | N identical full-screen modals; covered Approve buttons focusable | P1 |
| `BUG-UI-13` | graph truth | `web/src/flow/graph.ts:251-253` + `Canvas.tsx:378` | `hideFinished` drops a visible child's parent edge | P2 |
| `BUG-UI-14` | replay chrome | `web/src/flow/Canvas.tsx:609-650,684` | filter chips/counts mutate live state and show live counts in replay | P2 |
| `BUG-UI-15` | event-run filter | `web/src/state/store.ts:1468-1485` | `REPLY_DELTA` not guarded by active run; foreign text + `streamingRunId` theft | P1 |
| `BUG-UI-16` | rewire preconditions | `web/src/flow/Canvas.tsx:152-172` + `store.ts:704-721` | drag accepts worker parents; no cycle guard; diverges from ControlBar | P2 |
| `BUG-UI-17` | header search | `web/src/panels/Header.tsx:102-142` | no outside-click close; Escape loses focus; 320px overflow | P2 |
| `BUG-UI-18` | responsive | `HudBadge.tsx:15-22`, `ChatCore.tsx:195`, `ApprovalOverlay.tsx:249` | fixed geometry + no scroll containers on small viewports | P2 |
| `BUG-UI-19` | a11y semantics | `web/src/flow/Canvas.tsx:505` | nested `role="application"` inside React Flow's own | P2 |
| `BUG-UI-20` | a11y semantics | `nodes.tsx:112-139,145-170,150/204/284`; `Canvas.tsx:659-660` | progress has no progressbar role; double tab stop per SuperAgent; UUID-named edge stops; `aria-pressed` on exclusive selection | P2 |
| `BUG-UI-21` | control labelling | `web/src/panels/ControlBar.tsx:43-56` | first select labelled "WORKER: Re-route" but changes selection | P2 |
| `BUG-UI-22` | run status copy | `web/src/panels/ControlBar.tsx:118-124` | terminal run renders "Unavailable" | P2 |
| `BUG-UI-23` | approval a11y | `web/src/panels/ApprovalOverlay.tsx:291,200-202` | 1 Hz `aria-live`; silent click no-op when the TTL gate flips | P2 |
| `BUG-UI-24` | z-order | `ApprovalOverlay.tsx:211-220`; `ToastRegion.tsx:16`; `Header.tsx:103` | stale banner covers the header; toasts cover the search popover | P2 |
| `BUG-UI-25` | history | `HistoryPanel.tsx:31-64`, `Header.tsx:41-49`, `Scrubber.tsx:66-73` | three replay exits/two labels; `LATEST` lags a new commit | P2 |
| `BUG-UI-26` | preconditions | `web/src/panels/LibraryPanel.tsx:144-156` | `insertTemplate` fires during replay/offline | P2 |
| `BUG-UI-27` | canvas state | `web/src/flow/Canvas.tsx:669-683` | `localStorage` write inside an updater; drag offset persists across a rewire; offsets never pruned | P2 |
| `BUG-UI-28` | chat states | `web/src/chat/ChatCore.tsx:243-269,353-365` | offline "READY · NO RUN YET"; stop disabled for active non-`running` states | P2 |
| `BUG-UI-29` | chat store discipline | `web/src/chat/ChatCore.tsx:141-150` | direct `refusals` write bypasses dedupe/cap and the in-flight guard; `crypto.randomUUID` | P2 |
| `BUG-UI-30` | long content | `web/src/chat/ChatMotion.tsx:166-172` | unbroken text overflows the bubble | P2 |
| `BUG-UI-31` | graph robustness | `web/src/flow/graph.ts:66-73,280-329`; `store.ts:582-645` | no cycle guard on ingest; layout recursion has no visited set/depth cap | P2 |
| `BUG-UI-32` | dead state | `web/src/state/store.ts:281-287,2236-2238,2168-2173,2253-2259` | `viewport`/`diffOpen`/`pinned` lifted, never read; store write per pan-end | P2 |
| `BUG-UI-33` | process | `web/package.json` | no linter → hook-dependency defects invisible (FE-09/FE-10 shape) | P2 |
| `BUG-UI-34` | test guardrails | `web/tests/approval.integration.test.ts:265-267`; `ApprovalOverlay.tsx:101-110,129-144` | source-text assertions; duplicated `isApprovalExpired`; unused `prunePendingResolutions` + stale header comment | P2 |

---

## 6. Not verified (and what would verify it)

| Item | Why unverified | Cheapest proof |
|---|---|---|
| Every layout/visual claim in this document | app is stopped; no browser; `E_NO_BROWSER` | run the app, screenshot at 1440×900 and 768×1024, tab through each component |
| §FE-12 overlay collision (replay pill under HUD) | geometry derived from Tailwind + `LAYOUT`; not rendered | screenshot in replay mode with chat hidden — both pills in the same 44px band |
| §FE-13 stacked approval dialogs | needs ≥2 pending approvals; needs the server replay path | replay 2 pending approvals after a hello, then Tab — the covered Approve button receives focus |
| §FE-07 tab-stop count (~90–120) | estimated from 2 buttons on SuperAgents + 1 per node + 1 per edge; not counted in a live DOM | keyboard-only pass on a 30-node replay graph |
| §FE-01 behaviour under a real screen reader | no browser | VoiceOver/NVDA pass with an approval pending |
| §FE-19 viewport overflow | arithmetic from fixed px + no scroll containers | render at 360×640 and 768×1024 |
| §FE-32 cycle crash | no malformed snapshot was constructed (no writes allowed) | unit-level: `buildReplayTasks` with `a.parent=b, b.parent=a` then `layoutTasks`; expect a hang/stack overflow, then verify the guard |
| Any test-suite claim | **no test was executed** this round (audit-only, no app, no build) | `npx vitest run` in `web/` |
| Tailwind class truth (`hidden … lg:flex` really hides at 768) | source-read of the class string only | computed style at 768px |

**Method limits worth stating plainly:** this audit is a static, line-cited read of 9 target files plus 12 context files. It is strong on control flow, state ownership, event handling, ARIA attributes and literal geometry; it is worthless on actual painting, motion, contrast, and real AT behaviour. Nothing here was executed.
