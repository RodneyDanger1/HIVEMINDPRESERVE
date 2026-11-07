# 05 — Visual QA & Accessibility Audit (HiveMind UI rebuild)

**Audit date:** 2026-11-07, 15:00–15:45 America/Chicago
**Target:** HiveMind v1.2-dev, live server `http://127.0.0.1:8099` and `web/dist`
**Mode:** read-only analysis. No product source was edited. The only writes are this file and the screenshots in [`shots/`](shots/).
**Browser:** installed Google Chrome 156.0.8078.75 (headless), driven by the repo's `e2e` Playwright 1.65.0 (`channel: chrome`, DPR 1).
**Auxiliary tools:** Pillow 12.3.0 (pixel sampling), WCAG 2.1 relative-luminance maths, Playwright `ariaSnapshot()` for the Chromium AX tree.

> **Build drift warning — read before using this report.**
> The UI was rebuilt **three times** during this audit. The live binary was restarted twice, and once served a stale bundle after `web/dist` had already changed.
>
> | Build | Bundle | Built | What it is |
> |---|---|---|---|
> | B0 | `index-mA4Aln4d.js` / `index-jzmQMMJW.css` | 14:52:11 | Live 8099 until ~15:14. Screenshots `1440x900-workspace-empty.png`, `1280x900-workspace-empty.png`, `1024x800-workspace-empty.png`; interaction evidence for template/E_RUN_NOT_FOUND, E_KEY_MISSING, code-diff, pin, replay. **Superseded.** |
> | B1 | `index-BNZI3ayt.js` / `index-EvfscYcz.css` | 15:07:11 | Live briefly; screenshots `*-live-boot.png`, `*-newbuild.png` (static harness). **Superseded.** |
> | **B2** | **`index-BAzYVmfv.js` / `index-3INdK3UX.css`** | **15:20:56** | **Current build.** `web/dist` has no source newer than the build. All `*-1520.png` screenshots, all current measurements, and the findings below are verified against B2 unless the row says otherwise. |
>
> The running binary embeds `web/dist` at compile time. At 15:07 the dist was rebuilt but the live server still served B0 until it was rebuilt/restarted; the same pattern recurred before the B2 restart at 15:23:47. Any screenshot or verdict taken against 8099 must record the served bundle name. Live B2 was captured after the 15:23:47 restart with `assets/index-BAzYVmfv.js`.

---

## 1. Method and coverage

- **Live pass (B2):** 1440×900, 1280×900, 1024×800. Load, render, screenshots, console/`pageerror`/failed-request capture, bounding boxes, overlap arithmetic, computed styles, `ariaSnapshot` of topbar/rail/library/chat/HUD/controlbar/scrubber/canvas/minimap, keyboard focus order (26 tabs), skip-link activation, tablist arrow/End keys, tab surface comparison, Settings probe, code-diff click, project-pill interactivity.
- **Seeded graph (B2):** the live origin's WebSocket was mocked at the transport boundary with Playwright `page.routeWebSocket`; the test pushed `RUN_CREATED`, `RUN_STATUS`, and six `NODE_SPAWN` frames (hyperagent, 2 superagents, 2 workers, 1 tool). This exercises the real client/reducers/renderers without a provider call and without writing to the server's store. The live DB itself already contained a failed run from another QA session (`01a1b754`, chat row "live banner proof"); that is real live state and is visible in the boot screenshots.
- **Interactions on B0/B1:** template insert refusal, empty-prompt/key-missing refusal, chat pin (removed in B2), replay enter/exit, zoom/recentre/fit, chat hide/show, code-diff click, Settings probes. Findings that depend on a B0/B1-only behaviour say so.
- **Pixel sampling:** WCAG ratios for text-over-SVG/background cases (empty-state line, placeholder, attribution) were computed from rendered PNG pixels (median background, glyph-colour sampling) or from the browser's own blended placeholder pixel.
- **Not done:** screen-reader software was not run; the AX tree is Chromium's, not NVDA/JAWS/VoiceOver.

### What "verified" means here

`VERIFIED (build)` = exercised in this audit and observed. `SOURCE` = read from the current source and not independently exercised. `UNVERIFIED` = could not be exercised; see §7.

---

## 2. Measured geometry (B2 live)

All values are CSS px, DPR 1.

| Element | 1440×900 | 1280×900 | 1024×800 |
|---|---|---|---|
| Topbar | 0,0 **1440×52** | 0,0 **1280×52** | 0,0 **1024×52** |
| Nav rail | 0,52 **72×848** | 0,52 **72×848** | 0,52 **72×748** |
| Library panel | 72,52 **240×848** | 72,52 **240×848** | 72,52 **240×748** |
| Canvas | 312,52 **1128×848** | 312,52 **968×848** | 312,52 **712×748** |
| Chat core | 676,236 **400×480** | 596,236 **400×480** | 468,186 **400×480** |
| HUD | 646,64 **460×44** | 566,64 **460×44** | 438,64 **460×44** |
| Canvas toolbar (`Hide finished`/`Focus branch`/count) | 324,64 **256×26** | 324,64 **256×26** | 324,64 **256×26** |
| Mini-map | 1120,700 **220×150** | 960,700 **220×150** | 704,600 **220×150** |
| Zoom stack | 324,728 **32×160** | 324,728 **32×160** | 324,628 **32×160** |
| ControlBar+Scrubber overlay | 312,810 **1128×90** | 312,810 **968×90** | 312,686 **712×114** |
| Scrubber | 312,857 **1128×43** | 312,857 **968×43** | 312,757 **712×43** |
| Empty-state line | 682,788 **388×16** | 602,788 **388×16** | 474,688 **388×16** |
| React Flow attribution | 1387,778 **53×32** | 1227,778 **53×32** | 971,678 **53×32** |

**Overlaps (B2):**

| Pair | 1440×900 | 1280×900 | 1024×800 |
|---|---|---|---|
| Mini-map ∩ ControlBar | **220×40** | **220×40** | **220×64** |
| Zoom stack ∩ ControlBar | **32×78** | **32×78** | **32×102** |
| Empty-state ∩ ControlBar | 388×0 | 388×0 | **388×16 (100 % covered)** |
| HUD ∩ canvas toolbar | 0×26 | **14×26** | **142×26** |
| Library visible? | yes | yes | **yes (`lg:flex`)** |
| Tablist location | library | library | library |

The `lg` breakpoint (1024 px) keeps the library — and therefore the only workspace tab strip — visible at exactly 1024×800, but it is hidden below 1024 (out of the tested range).

---

## 3. Keyboard focus order (B2 live, 1440×900, empty graph)

| # | Element | Accessible name / role |
|---|---|---|
| 1 | Skip link | `link "Skip to canvas"` (moves to y=8 on focus) |
| 2 | Rail — Workspace | `button "Workspace"` (roving tabindex: other rail items are skipped) |
| 3–6 | Workspace tabs (in library) | `tab "Workspace" [selected]`, `tab "Swarm"`, `tab "Routing"`, `tab "History"` |
| 7–10 | Templates | `button "Content Writer"`, `"Data Analyst"`, `"Code Reviewer"`, `"Chatbot"` |
| 11 | Library disclosure | `summary "State & settings"` |
| 12 | Canvas toolbar | `button "Hide finished"` |
| — | (`Focus branch` is disabled until a node is selected; not a tab stop) | |
| 13 | React Flow attribution | `link "React Flow attribution"` → `https://reactflow.dev?utm_source=attribution` |
| 14–17 | Zoom stack | `button "Zoom in"`, `"Zoom out"`, `"Recentre hive (Ctrl+0)"`, `"Fit swarm"` |
| 18 | Chat close | `button "Hide the chat core (the hive stays)"` |
| 19 | Composer | `textbox "Type your prompt..."` |
| 20 | Role select | `combobox "WORKER: Re-route"` (option `"—"`) |
| 21 | Re-route select | `combobox "Re-route"` (option `"Re-route"`) |
| 22 | Scrubber (enabled on this live DB, prior run) | `slider "Time-travel scrubber"` = 0 |
| 23 | Live/replay button | `button "live"`, disabled |
| 24 | — | focus leaves the document to `body` (wrap) |

The rail's ArrowUp/ArrowDown roving focus **works** (focus moved to Agents while `aria-current` stayed on Workspace, Enter then changed the view). The tab strip's ArrowRight/End **do not** (see VQA-11).

### Seeded canvas focus model (B2, 4-node probe)

The library agent chips become focusable before the templates (`button "Backend Architecture (Running)"`, etc.). Then:

- 3 edge `<g>` elements: `group "Edge from hive-core-anchor to sa-be"`, `group "Edge from sa-be to w-db"`, `group "Edge from sa-be to t-p"` — **no action**.
- Superagent: **3 stops** — `group "Backend Architecture (superagent, running)"`, `button "Backend Architecture — SuperAgent, Running"`, and an inner body button with **no accessible name**.
- Worker: **2 stops** — group + unnamed button.
- Tool: **2 stops** — group + unnamed button.
- Last node: **`group` with no accessible name and 22×22 px** (invisible hive anchor, `tabindex="0"`), see VQA-08.

---

## 4. Accessible names by region (B2, live)

| Region | AX snapshot (abridged) |
|---|---|
| Topbar | `banner: text "HiveMind"; text "Connected YO You"` — no tablist here in B2; no search/bell/avatar-menu controls (reference shows them; adding them without commands would be dead controls, so omission is acceptable, but the owner block is static). |
| Nav rail | `navigation "Primary navigation": button "Workspace", "Agents", "Data", "Models", "Routing", "Settings"` |
| Library | `complementary "Library panel": text "Project: HiveMind AI" + inert chevron; tablist "Workspace tabs" (4 tabs); text "ACTIVE AGENTS"; paragraph "No active agents"; text "TEMPLATES"; 4 template buttons; text "TIP"; paragraph tip; group "State & settings"` |
| Chat | `region "HyperAgent hive chat core": text "Hive AI Assistant"; button "Hide the chat core (the hive stays): ×"; alert "Run refused …"` (when a refused run exists); `article` rows; `textbox "Type your prompt..."`; `button "Submit prompt" [disabled]`; hint paragraph |
| HUD | `status "Swarm throughput": "HyperAgent — SUPERAGENTS — WORKERS — GREEN"` (empty) / `"HyperAgent 3 SUPERAGENTS 3 WORKERS 33.3% GREEN"` (seeded) |
| Canvas | `application "HiveMind spatial canvas"` → `status "Swarm throughput"`, `button "Hide finished"`, `button "Focus branch" [disabled]`, `text "0/0"`, nested `application` (React Flow), `img "Mini-map"` → nested `img "Mini-map"`, `link "React Flow attribution"`, zoom buttons, paragraph empty-state |
| Control bar | `text "role:"; combobox "WORKER: Re-route"; button "Spawn sub-hive" [disabled]; combobox "Re-route"; button "Re-route" [disabled]; button "Cancel" [disabled]; button "Kill worker" [disabled]; text run status` |
| Scrubber | `text "Routing state"; slider "Time-travel scrubber" [disabled]; text "No run history yet"; button "live" [disabled]` |
| Mini-map | `img "Mini-map"` → nested `img "Mini-map"` (double image role; VQA-31) |

---

## 5. Findings

Severity: **High** = core flow broken / unusable without a mouse / materially misleading; fix before sign-off. **Medium** = clear usability, a11y or parity defect; fix this cycle. **Low** = polish, wording, or evidence gaps.

### High

| ID | Surface | Expected | Actual (measured) | Evidence | Exact fix seam |
|---|---|---|---|---|---|
| **VQA-01** | Canvas spawn layout | Agent cards appear around the pinned hive/chat and are visible or the viewport fits them | With 6 tasks, the cards are laid out **downward** from the chat origin and land **outside the viewport**: superagents at y=806 (canvas ends 900; the 90 px control-bar overlay starts 810), workers at y=1118, tool at y=1046. At 1024: superagents y=756, workers y=1068, tool y=996 (canvas ends 800; overlay starts 672). The seeded screenshots show the chat and bee edges but **no agent cards at all**. | `1440x900-live-seeded-1520.png`, `1024x800-live-seeded-1520.png`; measured node rects (this row); `web/src/flow/graph.ts:44` (`rankdir: 'TB'`); `web/src/flow/Canvas.tsx:157-168` (centres world origin at 0,0) | `graph.ts` — lay children out in a fan/LR around the chat (or offset the TB root by `chat.height/2 + ranksep` so the first rank clears the overlay); `Canvas.tsx:157-168` — once the first tasks arrive, call `fitView({padding:0.25,maxZoom:1})` or centre on the graph's bounds, not `(0,0)`. Add a gate that asserts every spawned card intersects the canvas viewport. |
| **VQA-02** | Bottom chrome vs canvas chrome | Control bar and scrubber must not cover the mini-map, zoom stack, or empty-state | `ControlBar+Scrubber` is an absolute overlay (`z-30`) over the canvas: it hides **40 px of the mini-map** at 1440/1280 and **64 px** at 1024, and **78 px of the zoom stack** at 1440/1280 / **102 px** at 1024. At 1440 the `Recentre hive` and `Fit swarm` buttons are fully behind the bar (only `+`, `100%`, `−` are visible); at 1024 even `−` is covered. | `1440x900-live-1520.png`, `1024x800-live-1520.png`; overlap table §2; `web/src/App.tsx:96-101`; `web/src/flow/Canvas.tsx:396-461` (zoom `bottom-3`), `Canvas.tsx:380-408` + `styles/index.css:131-138` (mini-map margin 50 px) | `App.tsx:96-101` — either reserve space (`main { padding-bottom: <bars height> }` / flex rows as before) or raise canvas control insets: zoom `bottom: calc(90px + 12px)`, mini-map `margin-bottom: calc(90px + 50px)` and recompute at the 114 px/1024 bar. Add a visual gate asserting `mini-map`/zoom stack ∩ bar = 0. |
| **VQA-03** | Canvas node focus | A focused card shows a clearly visible focus indicator | Focused `.react-flow__node.selectable` computes `outline-style: none` (width 3 px, colour `rgb(40,32,16)`); the focus screenshot shows the chat core/card with **no ring**. React Flow's own rule `.react-flow__node.selectable:focus-visible{outline:none}` wins over the global `:focus-visible`. | `1440x900-live-node-focus-1520.png`, `1024x800-live-node-focus-1520.png`; computed style `outlineStyle:"none"`; `styles/index.css:26-29`; `web/node_modules/@xyflow/react/dist/style.css` | `styles/index.css` after the tokens import: `.react-flow__node.selectable:focus-visible{outline:2px solid var(--focus-ring); outline-offset:3px}` (higher specificity than the library rule). Test with keyboard Tab on a card. |
| **VQA-04** | Canvas "Focus branch" | Pressing it shows only the selected node's branch; tooltip says "Show only the selected branch" | No-op in every combination: with `db-schema` selected, pressing it leaves **6/6** nodes and `aria-pressed="true"`; pressing it again with `Hide finished` produces exactly the same 5 nodes/labels as `Hide finished` alone (Backend, Frontend, db-schema, tailwind-builder, anchor). The branch filter is unioned with every non-terminal task, so it can never isolate. | `1440x900-live-seeded-hide-finished-1520.png`; counts `focusOnly {6/6, 6}` / `focusHide` labels identical to `hideOnly`; `web/src/flow/Canvas.tsx:113-144` (fallback `if (!hideFinished || !isTerminal(state)) keep.add`) | `Canvas.tsx:113-144` — when `focusBranch && selectedTaskId`, return the branch set only (ancestors + descendants), applying `hideFinished` **within** that set; do not fall through to the all-non-terminal union. Add a test: 6-node graph, select worker, press Focus branch → exactly 3 nodes. |
| **VQA-05** | HUD vs canvas toolbar at narrow widths | HUD and the `Hide finished / Focus branch / count` toolbar remain separate | HUD (460 px) overlaps the toolbar by **14 px at 1280** and **142 px at 1024**; at 1024 the HUD covers the `6/6` counter and a large part of `Focus branch`. | `1024x800-live-1520.png` (HUD sits over `6/6` / `Focus branch`); overlap table §2; `web/src/flow/Canvas.tsx:293` (`absolute left-3 top-3`) vs `web/src/panels/HudBadge.tsx:14` | `Canvas.tsx:293` — place the toolbar below the HUD at `<xl` (`top-[64px]`) or make the HUD width fluid (`w-[min(460px,calc(100%-32px))]`) and give the toolbar `max-w`; assert no overlap at 1280/1024. |
| **VQA-06** | Empty-state guidance at 1024 | "No agents yet…" is visible when the canvas is empty | At 1024 the 388×16 line (y 688–704) is **100 % covered** by the control-bar overlay (starts y=672); the screenshot shows no guidance at all. At 1440 it is visible (6 px above the bar) but sits immediately above the chrome. | `1024x800-live-1520.png`; overlap table §2; `web/src/flow/Canvas.tsx:464-470` (`pb-24`) | `Canvas.tsx:464-470` — position the line above the chrome (`bottom: <overlay height + 24px>`) or render it inside the control bar when the graph is empty; verify at 1024×800. |

### Medium

| ID | Surface | Expected | Actual (measured) | Evidence | Exact fix seam |
|---|---|---|---|---|---|
| **VQA-07** | Mini-map content | A scaled silhouette of the swarm (reference: node rectangles + honeycomb + viewport outline) | The mini-map renders **zero node shapes** with 6 nodes in the graph: `rects: 0`, SVG children = 2 (`<title>`, one mask path). The mask now has a 1.4 px `ink-900` stroke (improvement over B1), but the swarm is absent and the card reads as blank white. | `1440x900-live-seeded-1520.png`, `1024x800-live-seeded-1520.png`; `minimapRects: 0`; `web/src/flow/Canvas.tsx:380-408` | `web/src/flow/graph.ts:83-105` — set explicit React Flow node `width`/`height` (v12 uses measured dimensions for `MiniMap`); or give `MiniMap` a `nodeComponent`/`nodeColor` path that draws from the node's `style` size. Add a gate: with N nodes, `.react-flow__minimap-node` count = N. |
| **VQA-08** | Invisible hive anchor node | Non-visual anchor nodes are not focusable or announced | A 1×1 `opacity:0` `hive-core-anchor` node renders as a **22×22 `.react-flow__node` with `tabindex="0"`, `role="group"`, and no accessible name**; it is the last canvas focus stop (Tab after the cards). Keyboard/screen-reader users get a blank stop at the canvas centre. | B2 probe: node list (`aria:null, tabindex:"0", 22×22, opacity 0`); focus probe step 27; `web/src/flow/Canvas.tsx:168-176` | `Canvas.tsx:168-176` — set `focusable: false`, `selectable: false`, and `aria-hidden`/`hidden` behaviour for the anchor (React Flow node `focusable` prop or `nodesFocusable={false}` on a dedicated hidden node); or replace it with a plain source handle/`sourcePosition` coordinate and no node. |
| **VQA-09** | Canvas focus model | One focus stop per agent, named; edges not focusable unless actionable | Superagent = **3 stops** (group, title button, unnamed body button); worker/tool = **2** (group + unnamed button); three edges = **3 focusable `group` stops with no action**. The same selection action is duplicated on the group and its inner buttons. | Seeded focus probe (B2); `ariaSnapshot` canvas; `web/src/flow/Canvas.tsx:336` (`nodesFocusable`, `edgesFocusable`); `web/src/flow/nodes.tsx:125-203` | `graph.ts:83-105`/`nodes.tsx` — keep the node wrapper as the only focusable target (`tabIndex={-1}` + `aria-hidden` on inner buttons, or remove the buttons and use the wrapper's click); set `edgesFocusable={false}` unless an edge action exists; give the body button a short explicit `aria-label` if it remains. |
| **VQA-10** | Skip link | Activating "Skip to canvas" moves keyboard focus to the canvas | URL becomes `#hive-canvas` but `document.activeElement` is `BODY`; the target has no `tabindex`. The skip link is visible on focus (y=8) but the bypass fails. | Live B2: `skip {hash:"#hive-canvas", active:"BODY", targetTabindex:null}`; `web/src/App.tsx:77-79`; `Canvas.tsx:235` | `Canvas.tsx:235` — add `tabIndex={-1}` to `#hive-canvas` and focus it on skip (onClick handler on the link) or make the canvas heading focusable; add a keyboard test. |
| **VQA-11** | Workspace tabs (moved into library) | WAI-ARIA tabs: one tab stop, ArrowLeft/Right/Home/End, `aria-controls` + tabpanel | ArrowRight and End do nothing (focus stays on Workspace; selection unchanged); all four tabs have `tabindex` unset (four tab stops); none has `aria-controls`; there are **0 `role="tabpanel"`** elements. | Live B2: `tabArrow {focused:"Workspace"}`, `tabWiring {panels:0, controls:null}`; `web/src/panels/LibraryPanel.tsx:45-63` | `LibraryPanel.tsx:45-63` — add `onKeyDown` with `nextRovingIndex` (ArrowLeft/Right/Home/End), roving `tabIndex` (`0` for selected, `-1` otherwise), `aria-controls` ids and a `role="tabpanel"` around the workspace content (or change the pattern to plain buttons with `aria-pressed` if no panel semantics are intended). |
| **VQA-12** | Library "Active Agents" tiles | Agent name fits/truncates inside its 48 px column | Once tasks exist, each chip's button is **97 / 139 / 146 px** wide inside a **48 px** `<li>`; labels are 97–146 px wide and overflow into neighbours. The seeded screenshot shows overlapping text ("HIVE CHAR… BACKEND ARCHITECTURE…"). | `1440x900-live-seeded-1520.png`; label table (`li:48, btn:97/139/146, label:97/139/146`); `web/src/panels/LibraryPanel.tsx:70-93` (`li.w-12`, button without `w-full`, `span.w-full.truncate`) | `LibraryPanel.tsx:70-93` — add `w-full min-w-0` to the button and `overflow-hidden` to the `<li>`; use `max-w-[48px] truncate` + `title={task.label}` for the label (or a 2-column grid with wider cells). |
| **VQA-13** | Project selector | A control that opens the project list (or a static label without a dropdown affordance) | Renders `DIV "Project: HiveMind AI⌄"` — no role, no tabindex, `cursor:auto`, no click handler. It looks like a dropdown and is not one. | Live B2: `projectPill {tag:"DIV", role:null, tabindex:null, cursor:"auto", hasChevron:true}`; `web/src/panels/LibraryPanel.tsx:38-43` | `LibraryPanel.tsx:38-43` — either implement a real `<button aria-haspopup="listbox">` backed by the durable project list API, or remove the `⌄` and style it as a static label. |
| **VQA-14** | Settings "Code-diff review" | The control either opens the decision-gated notice or is disabled/absent | Clicking it changes nothing: no dialog, no alert, no DOM change (byte-identical `body.innerHTML`). `openDiffModal` sets `diffOpen`, but no component reads `diffOpen`; only the store references it. | Live B2: `diff {exists:1, changed:false, dialogs:0}`; `1440x900-view-settings-live-1520.png`; `web/src/panels/SettingsPanel.tsx:116-122`; `web/src/state/store.ts:2131-2137` | `SettingsPanel.tsx:116-122` — render `STRINGS.diff.decisionRequired` in an inline panel/dialog keyed on `diffOpen`, or replace the button with disabled text explaining the decision gate. |
| **VQA-15** | Reduced-motion control label | A motion-specific label | The checkbox reads **"Replay (read-only) · reduced motion override"** — it reuses the replay state string and tells the operator they are enabling replay. | Live B2: `reducedMotionLabel`; `1440x900-view-settings-live-1520.png`; `web/src/panels/SettingsPanel.tsx:114` | `strings.ts` add `settings.reducedMotion = "Reduced motion override"`; use it at `SettingsPanel.tsx:114`. |
| **VQA-16** | Run status in the control bar | Show the real terminal status (failed/completed/cancelled) | Any terminal run displays the literal **"Unavailable"** instead of its status (`01a1b754 · Unavailable` for a failed E_KEY_MISSING run). | `1440x900-live-boot.png` (15:14 live), `1440x900-toast-submit-refusal.png` (B0); `web/src/panels/ControlBar.tsx:120` | `ControlBar.tsx:116-122` — render `run.status` (or `STRINGS.agentStates[run.status]`) and reserve "Unavailable" for an unknown/missing status. |
| **VQA-17** | "Swarm" tab | A distinct swarm/overview surface | `main` text is byte-identical to the Workspace tab (**571 chars equal** in B2; 522/620 in earlier builds): same Canvas + ChatCore, only the tab highlight changes. There is no Swarm-specific view. | B2: `swarm {equal:true, len:571}`; `1440x900-tab-swarm-live-1520.png`; `web/src/App.tsx:46-58` | `App.tsx:46-58` — implement a distinct Swarm surface, or remove/rename the tab (e.g. "Canvas" is already Workspace) so navigation cannot promise a non-existent view. |
| **VQA-18** | Composer placeholder contrast | Text contrast ≥ 4.5:1 at 13 px | Placeholder is `50 % currentColor` over `panel-alt`; rendered pixel **(145,140,128)** on **#FBF8F1** = **3.16:1** (AA fail). | B2 computed/blended pixel; `web/src/styles/index.css` (no `::placeholder` rule); `ChatCore.tsx:242-256` | `styles/index.css` — `#composer::placeholder{color:var(--color-ink-500)}` (5.13:1) or `ink-700`; re-measure. |
| **VQA-19** | Replay indicator | The read-only replay mode is visibly announced on the canvas | A `Replay (read-only) · Routing state 0` pill is rendered at `top:16px`, centred, with **no z-index**, under the HUD (`top:12px`, `height:44`, `z-20`). In the replay screenshot the pill is not visible; only the footer says replay. | `1440x900-live-replay-attempt.png`, `1440x900-live-replay-banner-occlusion.png` (B1 live, replay entered via scrubber); pill rect 733,68 286×26 vs HUD 646,64 460×44; `Canvas.tsx:412-418`; `HudBadge.tsx:14` | `Canvas.tsx:412-418` — place the replay pill below the HUD (`top-20`), give it `z-30`, or surface replay state in the HUD itself; verify with a non-empty revision. |
| **VQA-20** | HUD count semantics | "SUPERAGENTS" counts sub-superagents; the HyperAgent is not counted twice | `deriveHud` counts `hyperagent` **and** `superagent` into `superagents`; a 1-hyperagent + 2-superagent graph displays **"3 SUPERAGENTS"** next to a separate "HyperAgent" prefix. | B2 seeded: `HyperAgent·3 SUPERAGENTS·3 WORKERS·33.3% GREEN`; `web/src/state/metrics.ts:41-43` | `metrics.ts:41-44` — count only `superagent` (and expose the hive separately), or remove the `HYPERAGENT` prefix. |

### Low

| ID | Surface | Expected | Actual (measured) | Evidence | Exact fix seam |
|---|---|---|---|---|---|
| **VQA-21** | Chat transcript after a refusal | A refused prompt is retracted (the code says it retracts the optimistic row) | After `E_KEY_MISSING`, the chat keeps the user row ("visual QA probe" / "live banner proof") with the refusal alert above it; the article count is 1. The retraction only looks for a toast refusal, not the run-level refusal that actually occurs. | `1440x900-toast-submit-refusal.png` (B0 live); `1440x900-live-seeded-1520.png` (live row); `web/src/chat/ChatCore.tsx:146-158` | `ChatCore.tsx:146-158` — also treat a new run-level refusal (`run.errorCode` / new run id) as a refusal and retract the optimistic prompt, or update the comment/intent so the retained row is deliberate. |
| **VQA-22** | React Flow attribution | Attribution readable and unobstructed | `#999` on white at 10 px = **2.85:1** (AA fail). The mini-map no longer overlaps it after the B2 margin change. | B2 computed (attribution `rgb(153,153,153)`, 10 px); rect 1387,778 53×32; `web/node_modules/@xyflow/react/dist/style.css` | `styles/index.css` — `.react-flow__attribution a{color:var(--color-ink-500)}` (5.6:1); do not hide it (license). |
| **VQA-23** | "Hide finished" semantics | Label matches the filter | `isTerminal` includes `failed`, `killed`, `cancelled`, `interrupted`; pressing `Hide finished` on the seeded graph removes the **failed** Prisma tool (6→5, `5/6`) and would also hide killed/cancelled. | B2: `hideOnly {nodes:5}`; `web/src/state/agentState.ts:111-119`; `Canvas.tsx:135-142` | Rename to "Hide terminal"/"Hide done", or filter only `completed` + `cancelled`; update the tooltip. |
| **VQA-24** | Library diagnostics | Operator copy only | The `State & settings` disclosure ends with **"HiveMind v1.2-dev"** (build version in the product UI). | `1440x900-library-details-open-newbuild.png` (B1); source unchanged in B2 `LibraryPanel.tsx:149-151` | Remove it, or move the version to an About/diagnostics surface and label it "build". |
| **VQA-25** | Role select naming | A clear, unique name | `combobox "WORKER: Re-route"` selects **any** task (not just workers); the second combobox is also named `"Re-route"`, duplicating the button name. | B2 AX snapshot; `web/src/panels/ControlBar.tsx:44,70` | `ControlBar.tsx:44` — `"Selected node"` / `"Re-route target"` with `aria-describedby`; keep the button "Re-route". |
| **VQA-26** | Replay "live" button | A descriptive accessible name | `button "live"` (lowercase, ambiguous) when replay can be exited. | B2 scrubber snapshot; `web/src/panels/Scrubber.tsx:66-73` | Add `aria-label="Return to live view"` (or the equivalent STRINGS entry). |
| **VQA-27** | `E_RUN_NOT_FOUND` guidance | Operator-actionable copy | The toast's `how_to_fix` says **"Check the run ID returned by RUN_CREATED."** — contract jargon for an operator who just clicked a template with no run. | `1440x900-live-template-refusal.png` (B1 live); `web/src/state/strings.ts` `clientRefusals.E_RUN_NOT_FOUND` | Reword to e.g. "Submit a prompt to start a run, then insert the template." (and keep the run-id detail only for diagnostics). |
| **VQA-28** | Chat composition parity | Reference: single-line composer with send/attach/link icons; serif title, bee glyph, X close | B2 has a 2-row textarea + large `SUBMIT PROMPT` button; the header is sans-serif with a bee icon only while a run is active and an X close. The close now matches the reference; the composer does not. | `1440x900-live-1520.png` vs `hivemind-spatial-canvas.png`; `ChatCore.tsx:170, 238-264` | Design decision: if aligning, make the composer a single-line input with a send icon, but **do not add attach/link icons without commands** (dead controls). |
| **VQA-29** | HUD parity | Reference pill lists only the three metrics with values | B2 prepends `HYPERAGENT ·` and shows `—` placeholders when no metrics exist (`HYPERAGENT · — SUPERAGENTS · — WORKERS · — GREEN`); the reference shows `4 SUPERAGENTS · 12 WORKERS · 98.2% GREEN`. | `1440x900-live-1520.png`; `HudBadge.tsx:15-31`; reference board | Consider hiding the stats (or showing "No metrics yet") when `source === 'none'`; keep the em dash if shown. |
| **VQA-30** | Nested ARIA applications / double image | One application landmark, one image per mini-map | `#hive-canvas` is `role="application"` and contains React Flow's own `application`; the mini-map wrapper is `role="img"` with an inner SVG also `role="img"`, so "Mini-map" is announced twice. | B2 AX snapshots; `Canvas.tsx:235`, `Canvas.tsx:203-220` (aria-label patch) | Use `role="region"`/no role on the outer canvas (let React Flow own the application role), and either remove the wrapper's `role/aria-label` or the SVG's `<title>/role` — keep exactly one. |
| **VQA-31** | Toast announcements | One announcement per refusal | `aria-live="assertive"` on the region **and** `role="alert"` on each item; some screen readers will announce twice. | `web/src/panels/ToastRegion.tsx:13-29` | Keep one live mechanism: drop `aria-live` from the container (keep `role="alert"`), or set `role="status"` on items and `aria-live` on the region. |
| **VQA-32** | Settings duplication | One label per value | "PROVIDER KEY STATUS" is both the section heading and a row label inside it. | `1440x900-view-settings-live-1520.png`; `SettingsPanel.tsx:39,76` | Rename the row to "Status" or drop the row. |
| **VQA-33** | Disabled primary button legibility | Disabled text remains readable where practical | `SUBMIT PROMPT` at `opacity .6`: rendered text **(133,127,114)** on **(246,217,150)** = **2.9:1**. Disabled text is WCAG-exempt, but it is visibly weak on the honey button. | B0 pixel sample; `ChatCore.tsx:257-263` (`disabled:opacity-60`) | Set explicit disabled colours (`bg-honey-100 text-ink-500`) instead of opacity. |
| **VQA-34** | Empty-canvas "Fit swarm" | Enabled controls should do something visible | With an empty graph `fitView` leaves the transform unchanged (no nodes to fit). The control is enabled and appears actionable. This is expected per the existing e2e comment but is a dead-looking control in the empty state. | B1 transform test (`t2 === t1`); `Canvas.tsx:451-459` | Disable "Fit swarm" when `nodes.length === 0` (and explain with a title), or enable it once the graph exists. |
| **VQA-35** | `<1024` widths | Workspace tab strip remains reachable | Tabs now live in the library (`lg:flex`). Below 1024 the library is hidden, and with it the **only** tab switcher; no drawer exists. Out of the tested range but a regression risk for narrow windows. | Source `LibraryPanel.tsx:35,45-63`; `App.tsx:82-84` | Add a compact tab strip/drawer below `lg`, or keep the tabs in the topbar at narrow widths. |

### Process findings (not product defects)

| ID | Finding | Evidence | Action |
|---|---|---|---|
| **P1** | The running binary serves the dist embedded at build time. Rebuilding `web/dist` alone does **not** update 8099 (observed at 15:07–15:14: dist `index-BNZI3ayt.js`, server still served `index-mA4Aln4d.js`). | `GET /` asset names vs `web/dist` listing | Rebuild/restart the release binary, or serve dist from disk in QA builds; record the served bundle in every evidence file. |
| **P2** | The build moved three times mid-audit (14:52 → 15:07 → 15:20), including a layout rewrite (tabs into the library, chat 420×500 → 400×480, control bar to an overlay, dagre LR → TB). Findings from B0/B1 may already be fixed or replaced. | Build table above | Freeze a build before sign-off and re-run §5 against the frozen hash. |
| **P3** | The test harness's static server returns 404 for `/api/history/runs`, producing one console error in harness-only captures. The live server returns **200** with runs. This is **not** a product defect and is excluded from all live console counts. | Harness console `404 … /api/history/runs`; live `GET /api/history/runs` → 200 | None; keep using the live server for console gates. |

---

## 6. Reference parity vs the FigCraft boards

Measured reference geometry (`docs/ui/figcraft/ASSETS/hivemind-spatial-canvas.png`, 1440×900): topbar bottom border at y=51; mini-map white panel **218×148** at 1121–1338 × 701–848; chat core ~420×500 centred; rail 72 px.

| Item | Reference | B2 | Verdict |
|---|---|---|---|
| Topbar height | ~52 (border at 51) | 52 | Match |
| Rail width | 72 | 72 | Match |
| Chat core | ~420×500, serif title, bee glyph, X close | 400×480, sans title, bee only while running, X close | Close; 20/20 px smaller (token change to 400×480). Title face differs (Low). |
| Mini-map size | 218×148 | 220×150 | Match |
| Mini-map content | Honeycomb + node rectangles + viewport rectangle | Viewport mask only (stroke added); **no node shapes** | **Fail** (VQA-07) |
| Superagent card | 420×150 | 420×150 | Match |
| HUD | Green pill, metrics only, top-centre | 460×44, `HYPERAGENT ·` prefix, em dashes | Close (Low, VQA-29) |
| Library width | Board labels say 260 px; board pixels measure ~226 px | 240 px | Acceptable; do not change on the label alone |
| Library content | Project dropdown, Active Agents names under hexagons, Templates, TIP | Project **inert** pill, Active Agents labels **overflow**, Templates, TIP | **Fail** on the pill (VQA-13) and labels (VQA-12) |
| Hive lobes | Soft, feathered, glowing | Hard-edged ellipses + new ambient glow/sheen layers | Improved in B2 but still harder than the reference; low. |
| Composer | Single-line, icons | 2-row textarea + Submit | Deviation (VQA-28); do not add dead icons just for parity. |
| Left tabs | Tabs in the topbar in B1 | Tabs in the library in B2 (2×2 grid, 9 px) | Not in the reference topbar; visually acceptable, but the ARIA pattern is broken (VQA-11). |
| Top-right chrome | Search, bell, avatar with name | Connection pill, "YO", "You" | Search/bell have no backing commands; omission is preferable to dead controls. Note as intentional. |

---

## 7. Checks that passed (evidence of good behaviour)

- **Live console/network (B2):** 0 console errors, 0 `pageerror`, 0 failed requests / HTTP ≥ 400 at 1440, 1280, and 1024. (The live server returns 200 for `/api/history/runs`; the harness-only 404 is excluded, see P3.)
- **No document overflow:** `documentElement.scrollWidth === clientWidth` at all three sizes; no unintended element outside the viewport. (The only off-viewport element is the intentionally hidden `.skip-link` at `top:-48px`.)
- **Reference-matched chrome:** topbar 52, rail 72, chat ~400×480, mini-map 220×150, library 240 (visible at 1024 in B2 — fixed from B1's `xl` gate).
- **Honest refusals:** template insert with no run returns a named `E_RUN_NOT_FOUND` toast; submitting without a key returns `E_KEY_MISSING` in the chat with `how_to_fix` (no fake success). The refusal toast is dismissible and disappears on dismissal.
- **Replay works:** with one committed revision, ArrowRight on the scrubber enters replay (`REPLAY (READ-ONLY) · 1 OF 1`), the `LIVE` button enables, and clicking it exits replay. (Caveat: that revision has 0 nodes; see §8.)
- **Chat pin** (B0/B1) toggled `aria-pressed` only after the server ACK; the control was removed in B2.
- **Hide finished** actually filters: 6 → 5 nodes, count `5/6`, `aria-pressed="true"`.
- **Focus branch enables** after selecting a node (`disabled=false`) even though its action is a no-op (VQA-04).
- **Rail keyboard:** ArrowDown moved focus from Workspace to Agents while `aria-current` stayed; Enter activated Agents and the Agents panel rendered. ArrowUp returned to Workspace.
- **Settings probes:** provider/model/thinking/effort/key status rendered from `SETTINGS_STATUS`; health rendered `degraded` with `RUSTC:OK / NODE:OK / DEEPSEEK_API_KEY:FAILED`; both Refresh buttons re-probe.
- **Data view:** `FILE_TREE_LIST` loaded the workspace tree and directory rows expand; the preview hint is shown until a file is selected.
- **Tab surfaces:** Routing tab renders the routing table (empty-state row), tabs update `aria-selected`, and the Workspace tabs are reachable from the library.
- **Disabled gating:** Spawn/Cancel/Kill/Re-route are disabled unless the run/selection/pending preconditions hold; the toolbar and chat close only act when enabled.
- **Reduced motion:** the `data-reduced-motion` attribute is set on `<html>` and the CSS disables the new ambient animations under `prefers-reduced-motion`; the in-app override exists (label bug VQA-15).
- **No horizontal scroll in the library** and `State & settings` is focusable and expandable.

---

## 8. UNVERIFIED / not exercised

| Item | Why | What would prove it |
|---|---|---|
| Drag-to-rewire (150 px snap radius, drop onto a superagent) | Requires a live run and real graph mutation; the seeded WS is a transport mock, and no provider key is configured | Run one real prompt, drag `db-schema` onto `Frontend Architecture`, observe `NODE_REWIRE`/ACK and the edge change |
| Spawn sub-hive, Cancel, Kill worker on a running run | No live running run; buttons were only seen disabled | A live run with a key; exercise each and capture the ACK/refusal |
| Approval overlay, tool activity, tool results | No provider key → no tool calls/approvals | A keyed run that triggers `APPROVAL_REQUEST` and `TOOL_CALL` |
| Streaming reply (`REPLY_DELTA`) rendering | Not exercised; the live DB had only refused runs | A keyed prompt with streaming; check incremental rows and scroll behaviour |
| Offline/reconnecting banners, mock-mode banner | Server was up; mock requires a dev build | Stop the server mid-session; run a `VITE_UI_MOCK=1` dev build |
| Replay with a non-empty graph | The live revision had 0 nodes; the seeded graph had no committed revisions | A completed run with ≥2 nodes; enter replay and verify nodes, read-only interactions, banner visibility |
| History tab with multiple revisions | No multi-revision run in this DB | Same keyed run; scrub through revisions and inspect the History tab |
| Reduced-motion override effect | Checkbox label verified; behaviour not exercised | Toggle it and measure animation durations / the HTML attribute |
| Inside-node focus indicators | Node group focus is unringed; inner buttons may have the global ring | Tab through a node and capture each stop's computed outline (the group is the blocking case) |
| Focus order with approvals/dialogs open | No approval/dialog state | Trigger an approval and tab through the modal |
| Widths below 1024 (phone/tablet) and dark mode | Out of the requested matrix; no dark theme in v1 | Capture 768/390; confirm tab strip access and no clipping |
| Screen-reader announcements (NVDA/JAWS/VoiceOver) | Only the Chromium AX tree was inspected | A screen-reader pass on the fixed build |
| Contrast of every text pair | Only the fail-prone surfaces were sampled (placeholder, attribution, empty state, lobes, disabled submit) | Automated axe/pa11y run plus manual token audit |
| Mini-map pan/zoom interactions | Mini-map is blank (VQA-07); interactions not exercised | Fix node rendering, then pan/click the mini-map and compare the viewport |
| Pixel-perfect parity with the FigCraft boards | The boards are text-layer/contact-sheet exports; the manifest itself records pixel-layout fidelity as UNVERIFIED | A human design review with both boards at 1440×900 |

---

## 9. Reproduction notes

Raw measurement JSON was printed to the tool stdout rather than written to disk (the audit's write scope was this report plus `shots/`); every number quoted above is reproduced in §2 and §5 so the evidence stays auditable from this file.

**Live capture (B2):**
```
cd C:\Users\Riley\Desktop\hiverouter-v1\e2e
$script = @' … @'   # Playwright script executed from stdin (no repo file written)
$script | node --input-type=module -
```
Chrome: `chromium.launch({ channel: 'chrome', headless: true })`; `browser.newContext({ viewport: {width,height}, deviceScaleFactor: 1 })`; `page.goto('http://127.0.0.1:8099', { waitUntil: 'networkidle' })`.

**Seeded graph:** install `page.routeWebSocket(/\/ws(\?.*)?$/)`, auto-ACK every `COMMAND`, then `route.send(eventFrame(...))` for `RUN_CREATED`, `RUN_STATUS`, and six `NODE_SPAWN` frames. This is a transport-level test double only; no server write, no provider call. The live server was left untouched.

**Pixel sampling:** Python + Pillow; sample the text bbox for glyph colour (darkest/brightest percentile) and a local background band; WCAG 2.1 formula. The placeholder ratio uses the browser's own blended pixel (`getComputedStyle(el,'::placeholder')` rendered on a canvas).

---

## 10. Screenshot index

Primary evidence (B2, live 8099, bundle `index-BAzYVmfv.js`):
- [`shots/1440x900-live-1520.png`](shots/1440x900-live-1520.png) — workspace boot
- [`shots/1280x900-live-1520.png`](shots/1280x900-live-1520.png) — workspace boot
- [`shots/1024x800-live-1520.png`](shots/1024x800-live-1520.png) — workspace boot (HUD/toolbar collision; empty state hidden)
- [`shots/1440x900-live-seeded-1520.png`](shots/1440x900-live-seeded-1520.png) — 6-node graph: cards offscreen, blank mini-map, labels overflowing
- [`shots/1024x800-live-seeded-1520.png`](shots/1024x800-live-seeded-1520.png) — same at 1024
- [`shots/1440x900-live-seeded-hide-finished-1520.png`](shots/1440x900-live-seeded-hide-finished-1520.png) — `Hide finished` = 5/6; `Focus branch` no-op
- [`shots/1440x900-live-node-focus-1520.png`](shots/1440x900-live-node-focus-1520.png) — focused card, no focus ring
- [`shots/1024x800-live-node-focus-1520.png`](shots/1024x800-live-node-focus-1520.png) — same at 1024
- [`shots/1440x900-view-settings-live-1520.png`](shots/1440x900-view-settings-live-1520.png) — settings, dead diff button, reduced-motion label
- [agents](shots/1440x900-view-agents-live-1520.png), [data](shots/1440x900-view-data-live-1520.png), [models](shots/1440x900-view-models-live-1520.png), [routing](shots/1440x900-view-routing-live-1520.png)
- [`shots/1440x900-tab-swarm-live-1520.png`](shots/1440x900-tab-swarm-live-1520.png) — Swarm equals Workspace

Supporting evidence (B1, live 15:07 / 15:14, bundle `index-BNZI3ayt.js`):
- [`shots/1440x900-live-boot.png`](shots/1440x900-live-boot.png) — refused-run state, terminal status "Unavailable"
- [`shots/1440x900-live-template-refusal.png`](shots/1440x900-live-template-refusal.png) — `E_RUN_NOT_FOUND` + developer guidance
- [`shots/1440x900-live-replay-attempt.png`](shots/1440x900-live-replay-attempt.png) and [`shots/1440x900-live-replay-banner-occlusion.png`](shots/1440x900-live-replay-banner-occlusion.png) — replay footer; canvas pill hidden by HUD
- [`shots/1440x900-toast-submit-refusal.png`](shots/1440x900-toast-submit-refusal.png) — `E_KEY_MISSING`, retained prompt
- [`shots/1440x900-chat-hidden.png`](shots/1440x900-chat-hidden.png), [`shots/1440x900-chat-pin-toggled.png`](shots/1440x900-chat-pin-toggled.png) — chat hide/show and ACK-gated pin
- [`shots/1440x900-library-details-open-newbuild.png`](shots/1440x900-library-details-open-newbuild.png) — `State & settings`, version string

Historical / harness (B0/B1, superseded; kept for drift comparison):
- [`shots/1440x900-workspace-empty.png`](shots/1440x900-workspace-empty.png), [`shots/1280x900-workspace-empty.png`](shots/1280x900-workspace-empty.png), [`shots/1024x800-workspace-empty.png`](shots/1024x800-workspace-empty.png)
- [`shots/1440x900-workspace-empty-newbuild.png`](shots/1440x900-workspace-empty-newbuild.png), [`shots/1440x900-workspace-seeded-newbuild.png`](shots/1440x900-workspace-seeded-newbuild.png)
- [`shots/1440x900-workspace-empty-1520.png`](shots/1440x900-workspace-empty-1520.png), [`shots/1280x900-workspace-empty-1520.png`](shots/1280x900-workspace-empty-1520.png), [`shots/1024x800-workspace-empty-1520.png`](shots/1024x800-workspace-empty-1520.png) (static-harness rendering of the same B2 bundle)

---

## 11. Bottom line

B2 (15:20:56) is a **layout rewrite in progress**, not a shippable state:

1. A run's cards spawn **below the visible canvas**, so the primary spatial-canvas experience shows no agents (VQA-01).
2. The new absolute control-bar overlay **covers the mini-map, the zoom controls, and at 1024 the only empty-state guidance** (VQA-02, VQA-06); at 1024 the HUD also **covers the canvas toolbar** (VQA-05).
3. Two headline controls are **non-functional**: `Focus branch` (no-op; VQA-04) and `Code-diff review` (dead; VQA-14). The project pill looks interactive and is not (VQA-13).
4. Keyboard/screen-reader basics are **not met**: no visible focus on canvas nodes (VQA-03), skip link does not move focus (VQA-10), tabs have no arrow-key/tabpanel wiring (VQA-11), an invisible unlabeled node is a focus stop (VQA-08), and node/edge focus is duplicated (VQA-09).

The strongest parts are the honest refusal paths, the disabled-state gating, the live console (clean), the reference-matched chrome geometry, and the fix direction visible in the last two builds (library at 1024, tabs relocated, chat/pin cleanup, mini-map mask). The list above is actionable; every row carries a file/line seam. Re-run this document against a frozen build hash after the layout work lands.
