# 09 — Live visual QA (HiveMind)

**Snapshot:** 2026-11-07 16:04–16:07 CST (America/Chicago)
**Verdict:** no P0; three P1 layout/hierarchy defects (minimap, control bar, chat composition). No pixel-parity claim is made.

## 0. Environment and evidence

| Item | Value |
|---|---|
| Live URL | `http://127.0.0.1:8099` |
| Server | `hiverouter.exe` PID **6244**, started **16:04:24**; `GET /` returned 200 at 16:06:47 |
| Build | bundle **`/assets/index-Dygo4wOW.js`**, 739,030 B, mtime **2026-11-07T16:01:43-06:00**, SHA-256 `16B12DD7E3AA85C0C497FFBD55A7CA056BF4C142DB3BCA6C3739B99D30D4ACB6` |
| Styles | **`/assets/index-CA5F52F8.css`**, 58,319 B, mtime 16:01:43, SHA-256 `B7ADB17D30DC4B665A9CBEFC05C89D3C7D3C4FBFAE1F83B9B839ED72DD020C94` |
| Source/build alignment | At capture 16:04–16:07 the latest source edit was 16:01:22 (`HiveField.tsx`) and the build was 16:01:43; no source edit after the build was observed through 16:06:47. **After capture**, `Canvas.tsx` was edited at **16:10:06** and `graph.ts` at **16:10:48**; those edits are NOT in the Dygo4wOW bundle and are untested here. |
| Browser | Chrome 156.0.8078.75, headless, CDP `:9222`, DPR 1 |
| Method | `Page.navigate`, wait for header text **CONNECTED** (observed 595 ms), `document.fonts.ready`, 1.8 s settle, then viewport overrides. **No `--virtual-time-budget`** was used; no false `E_WS_DISCONNECTED` toast appeared. |
| State under test | No run, no history, no agents, HUD values `—`; chat shows its starter empty state. This is the clean empty project state, not a seeded graph. |
| Environment note | The originally named PID 16072 disappeared during the operator rebuild at ~15:56; QA never signalled or restarted it. The screenshots below are against the replacement process (6244) and the Dygo4wOW build. No product source, FigCraft asset, or git state was modified by QA. Any source edits after 16:10 require a rebuild and a fresh run; this report must not be read as validating them. |

### Screenshot evidence (exact dimensions verified from PNG IHDR)

| File | Viewport | Bytes | SHA-256 | Purpose |
|---|---|---|---|---|
| [1440x900.png](shots-live/1440x900.png) | 1440×900 | 409,053 | `5FFE5B96BBBCB745ABB389643DA3319DCF8EDDC629869E3183C6C451315F073F` | required 1440 capture, CONNECTED |
| [1280x900.png](shots-live/1280x900.png) | 1280×900 | 386,510 | `C3FB149286189B41CC02549059536D2E52CFC74C9B669ADEDD0EDE1620684E0D` | required 1280 capture, CONNECTED |
| [1024x800.png](shots-live/1024x800.png) | 1024×800 | 313,905 | `382FC510FD1C686F227056B3E1ED0EAD3CE2FD380E0466156E61329182C2818B` | required 1024 capture, CONNECTED |
| [1440x900-reduced-motion.png](shots-live/1440x900-reduced-motion.png) | 1440×900 | 377,142 | `A452F96D1CF2FF4191AEFFC791183AF9624E7800952C946D84B55E44C3645C8F` | reduced-motion, CONNECTED |
| [1440x900-empty-state.png](shots-live/1440x900-empty-state.png) | 1440×900 | 582,246 | `90EC45CF67DC3C108BCF255D6E0EFE6821ADFD7E13F09CBC49B3E466AE03BEEC` | extra: chat hidden, canvas empty-state visible |
| [1024x800-empty-state.png](shots-live/1024x800-empty-state.png) | 1024×800 | 438,203 | `CA55F9DD9A5DF30BFE571702A52DC662B4ACD43561F52261D62E052540834138` | extra: chat hidden, 1024 empty-state collision |

All three required captures show the header chip `CONNECTED`. The measured header innerText at capture was `HiveMind\nCONNECTED\nHM\nYOU`; the composer/search/tabs were interactive.

## 1. DOM geometry (task 2)

All values are `getBoundingClientRect()` at the stated viewport, after CONNECTED and font settle.

### 1440×900

| Element | Selector | x | y | w | h | Notes |
|---|---|---:|---:|---:|---:|---|
| Topbar | `header` | 0 | 0 | 1440 | 52 | `position: static`; no clipping |
| Nav rail | `nav[aria-label="Primary navigation"]` | 0 | 52 | 72 | 848 | 6 × 64 px items; labels 8 px |
| Library | `aside[aria-label="Library panel"]` | 72 | 52 | 240 | 848 | internal scroll 848/848 |
| HUD | `[aria-label="Swarm throughput"]` | 646 | 64 | 460 | 44 | matches the board's 460×44; all metrics `—` |
| Chat core | `section[aria-label="HyperAgent hive chat core"]` | 676 | 236 | 400 | 480 | centered in the full 848 px content area |
| Chat list / empty block | chat `div[aria-live="polite"]` / first child | 688 / 696 | 296 / 304 | 376 / 360 | 324 / 150 | paragraph 360×36; 3 starter buttons 360×30 |
| Minimap | `.react-flow__minimap` | 1120 | 630 | 220 | 150 | right inset 100, bottom inset 120 |
| Zoom stack | `[aria-label="Zoom controls"]` | 324 | 612 | 32 | 160 | four 32×32 buttons; label 100% |
| Control bar | spawn button's parent | 312 | 810 | 1128 | 47 | 30 px action buttons |
| Scrubber | `footer` | 312 | 857 | 1128 | 43 | “NO RUN HISTORY YET”, LIVE disabled |
| Controlbar status | span “NOTHING HERE YET” | 1317.14 | 826 | 110.86 | 16 | right aligned |
| Scrubber note | span “NO RUN HISTORY YET” | 1241.58 | 871 | 124.70 | 16 | right aligned |
| Search | `button[aria-label="Search agents"]` | 1217.84 | 9.5 | 32 | 32 | opens a 302×42 popover input at (1117, 57) |
| Submit | `form button[type=submit]` | 1019 | 640 | 32 | 32 | icon only; disabled on empty draft |
| Project row | library first `div` | 84 | 64 | 215 | 32 | static `DIV`, `cursor:auto`, no role/tabindex |

**Empty-state:** with the chat visible the canvas empty-state is intentionally not rendered (`web/src/flow/Canvas.tsx:481` requires `!chatVisible`). With the chat hidden it measures `682.02,724,387.97×16` at 1440. The chat's own empty block is the visible empty state in the default view (696,304,360×150). The canvas note only appears when the chat is hidden; both extra screenshots capture that condition.

### 1280×900 / 1024×800 deltas

| Element | 1280×900 | 1024×800 |
|---|---|---|
| HUD | x=566, y=64, 460×44 | x=438, y=64, 460×44 |
| Chat | x=596, y=236, 400×480 | x=468, y=186, 400×480 |
| Minimap | x=960, y=630, 220×150 | x=704, y=530, 220×150 |
| Control bar | x=312, y=810, 968×47 | x=312, y=686, 712×**71** (wraps) |
| Scrubber | x=312, y=857, 968×43 | x=312, y=757, 712×43 |
| Zoom stack | x=324, y=612, 32×160 | x=324, y=512, 32×160 |
| Empty state (chat hidden) | 602.02,724,387.97×16 | 474.02,624,387.97×16 |
| Topbar / rail / library | unchanged x/w; library h=848 | unchanged x/w; library h=748 |

### Overlap and clipping

| Pair / check | 1440 | 1280 | 1024 | Severity |
|---|---|---|---:|---|
| Chat ∩ minimap | none | **36×86** (left 36 px of minimap under chat) | **164×136** (≈67.6% of the 220×150 minimap under chat) | **P1** |
| Empty-state ∩ minimap (chat hidden) | none (empty right 1070 < minimap left 1120) | **30×16** (right end of copy) | **158×16** (right 41% of copy hidden) | **P2** (part of P1 minimap) |
| Chat ∩ control bar | none (94 px gap) | none (94 px) | none, but only **20 px** gap (chat bottom 666, bar top 686) | P1 composition |
| Zoom ∩ control bar | none (38 px gap) | none (38 px) | none (14 px gap) | P2 |
| Minimap ∩ control bar | none (30 px gap) | none (30 px) | none (6 px gap) | P2 |
| HUD ∩ topbar/chat | none | none | none | pass |
| Viewport clipping of measured boxes | none; `document.scrollWidth/Height` = 1440/900 | none; 1280/900 | none; 1024/800 | pass |
| Library internal overflow | none (848/848) | none | none (748/748) | pass |
| Controlbar wrap | one row | one row | **height 47→71**; “NOTHING HERE YET” moves to a second row under KILL WORKER | P2 |

No measured box extends outside the viewport and no document scrollbar is introduced. The 1024 defect is occlusion, not clipping: the chat paints over the minimap and the minimap paints over the empty-state copy.

## 2. Motion (task 3)

`document.getAnimations()` after 2 s, default media at 1440. Seven CSS animations are running; six are visible, one is on an off-screen element.

| Animation | Target | Duration | Iterations / timing | Visible | Reduced motion |
|---|---|---:|---|---|---|
| `motion-honeycomb-travel` | `.motion-honeycomb-glow` (1128×848 field) | 16 s | infinite, alternate, ease-in-out | yes | disabled; `animation-name:none`, static `opacity:.55` |
| `motion-hive-sheen-sweep` | `.motion-hive-sheen` (620×660) | 12 s | infinite, linear | yes | disabled (`none`) |
| `hive-scan` | `.hive-scan` (1128×848) | 16 s | infinite, linear | yes | disabled (`none`) |
| `ambient-flight` | `.ambient-bee-a` | 13 s | infinite, ease-in-out, delay −2 s | yes (small bee) | disabled (`none`) |
| `ambient-flight` | `.ambient-bee-b` | 16 s | infinite, ease-in-out, delay −6 s | yes (small bee) | disabled (`none`) |
| `ambient-flight` | `.ambient-bee-c` | 18 s | infinite, ease-in-out, delay −9 s | yes (small bee) | disabled (`none`) |
| `state-pulse` | off-screen SVG state dot inside a closed `<details>` | 1.6 s | infinite, ease-in-out | **no** (y=920 at 1440; `offsetParent` null) | disabled (`none`) |

Not currently present: chat message/toast/card entrance animations (no messages/toasts), work-bee spinner (no streaming), typing dots, progress sheen. Zoom transitions were not sampled at reduced motion.

**Reduced-motion verification:** a fresh load with `prefers-reduced-motion: reduce` applied before navigation reported **0** animations at 2.5 s and again at 4.0 s; computed `animation-name` was `none` for the glow, sheen, scan, all three bees, and the hidden state dot. The glow rests at its declared `opacity: .55`. A dynamic media toggle sampled at 700/1200 ms retained one hidden `state-pulse` entry in `getAnimations()` but its computed style was `none`; it is not visible. Treat reduced motion as effective, with that measurement caveat. Screenshot: [1440x900-reduced-motion.png](shots-live/1440x900-reduced-motion.png).

**Motion-density finding:** six decorative loops run concurrently on the same canvas (glow, sheen, scan, three bees), despite `web/src/styles/motion.css:9-10` stating “one calm movement at a time”. The hidden state dot also runs a 1.6 s loop. Both are P2 craft/perf issues, not functional defects.

## 3. AI-slop audit (task 4)

| Signal | Evidence | Verdict | Severity |
|---|---|---|---|
| Generic gradient blobs | 4 CSS gradients: honeycomb glow, hive sheen, hive scan, chat header. All use honey/ink token colours; the field glow is 1128×848 but is contained by the canvas (`scrollWidth=clientWidth=1128`) and is brand-derived, not a generic purple/blue blob. | No classic blob slop; on-brand | — |
| Random neon | Saturated computed colours are only token colours: `rgb(78,58,21)` ink-700, `rgb(232,184,80)` honey-400, `rgb(178,58,46)` signal, `rgb(200,128,48)` honey-600, `rgb(240,192,80)` honey-300. No cyan/magenta/electric green. | none | — |
| Excessive glow | Only `--shadow-e1`/`e2` (max 6 px 20 px) on HUD/surface/chat zoom; no `text-shadow`, `filter`, `backdrop-filter`; the only blend mode is `soft-light` on the sheen. | acceptable; no neon bloom | — |
| Emoji icons | 0 `Extended_Pictographic` characters in body text. Close/zoom use text glyphs `×`, `+`, `−`, `0`, `⤢`, not emoji. | no emoji; glyph-vs-line-icon consistency is P2 | P2 |
| Fake metrics | HUD reads `HYPERAGENT · — SUPERAGENTS · — WORKERS · — GREEN`. Only percentage is `100%` (real zoom); `0/0` is the real task count. No demo numbers, no invented green %. | pass | — |
| Duplicated controls | Two adjacent “Re-route” affordances: target `<select aria-label="Re-route">` and action `<button>RE-ROUTE</button>`. Reference shows one Re-route button. | confusing duplicate label | P2 |
| Dead buttons | Exercised: search (opens/closes), 3 starter buttons (one clicked, draft set), zoom in/recenter (100→120%), tabs Workspace↔Swarm, chat hide/reopen, details disclosure. All responded. `Hide finished`, `Focus branch`, all controlbar actions, and `LIVE` are disabled by state (not dead). Template insert, Submit and other nav destinations were not exercised. | no dead enabled control found; 3 mutation paths unverified | — |
| Placeholder copy | No `lorem/TODO/FIXME/dummy/sample/fake/mock` in body text. `Type your prompt...` is a legitimate placeholder; `NOTHING HERE YET`, `NO RUN HISTORY YET`, `NO AGENTS IN THIS RUN` are honest empty states. | pass | — |
| Fake image assets | 0 `<img>` raster images; 24 inline SVGs. No stock-photo/gradient-blob placeholders. | pass | — |
| Decorative overload | Six visible infinite loops plus three free-floating bees; reference board has no equivalent field animation. | craft/perf note | P2 |
| Static “control” that looks interactive | Project row is a 215×32 `DIV` with `cursor:auto`, no role, no tabindex, no chevron; reference shows a dropdown chevron. | dead affordance | P2 |
| Hidden animation | `state-pulse` runs 1.6 s infinite on an off-screen SVG in closed `<details>` at y=920. | wasted loop | P2 |

**Overall:** no classic AI-slop (no generic blobs, neon, emoji, fake metrics, or fabricated rows). The defects are layout/spec drift and decoration density.

## 4. Six highest visual mismatches vs the reference boards (task 5)

Reference values are from the supplied boards and are approximate; **no pixel parity is claimed**. Boards: [hivemind-spatial-canvas.png](../../ui/figcraft/ASSETS/hivemind-spatial-canvas.png), [hivemind-controls-and-hud.png](../../ui/figcraft/ASSETS/hivemind-controls-and-hud.png), [hivemind-agent-cards.png](../../ui/figcraft/ASSETS/hivemind-agent-cards.png), [hivemind-states-and-refusals.png](../../ui/figcraft/ASSETS/hivemind-states-and-refusals.png). The boards are internally inconsistent about two sizes: the spatial board draws library 240 px and minimap 220×150 px, while the controls board captions say 260 px and 260×170 px. Live matches the spatial-board drawings; the caption values are marked **UNVERIFIED**.

| # | Severity | Area | Expected (board) | Actual (measured) | Exact code seam | Evidence |
|---|---|---|---|---|---|---|
| 1 | **P1** | Mini-map | Textured 220×150 mini-map (spatial board draws 220×150 at x≈1121–1338, y≈701–848; controls caption says 260×170), bottom-right, clear of the chat, with visible node/mask content. | 220×150 at (1120,630), right 100/bottom **120**; **blank white card**: computed `background-image:none`; chat covers 36×86 at 1280 and 164×136 at 1024; with chat hidden it covers 30×16/158×16 of the empty-state copy. | `web/src/flow/Canvas.tsx:398-424` (inline `background:` at 421 resets the CSS grid); `web/src/styles/index.css:132-135` (grid + `margin:0 100px 120px 0`) | 1440/1280/1024 shots; empty-state shots |
| 2 | **P1** | Control bar | 40 px minimum targets; filled honey primary “Spawn sub-hive”; paper “Re-route”; text “Cancel”; crimson outline “Kill worker”; one re-route action. | Bar 47 px (1440/1280), 71 px at 1024; all four action buttons are 30 px high and white-outlined; spawn has no primary fill; two “Re-route” affordances (select + button); status note wraps at 1024. 21 of 27 visible buttons are under 40 px high. | `web/src/panels/ControlBar.tsx:41-116` (outline classes at 60/85/99); `web/src/flow/Canvas.tsx:436-479` (32 px zoom stack) | 1440/1024 shots |
| 3 | **P1** | Chat core | ≈420×488, top ≈188, bottom ≈676, clear of the bottom hint strip; composer has send + microphone + attachment affordances. | 400×480 at (676,236), bottom 716; centered in the full 848 px content area while the 90 px controlbar+scrubber overlays the bottom. At 1024 it ends 666, only 20 px above the controlbar. Composer has only the 32 px send icon; no microphone/attachment. | `web/src/chat/ChatCore.tsx:176-180` (absolute center), `274-305` (composer), `web/src/App.tsx:97-101` (bottom overlay), `web/src/tokens/tokens.ts:170` | 1440/1280/1024 shots |
| 4 | **P2** | Topbar cluster | Search, notification bell + red dot, “AC” avatar, “A. Chen” name and chevron. | Search, `CONNECTED` chip, `HM` avatar, `YOU`; no bell, no operator menu/chevron. Search works (popover at 1117,57, 302×42 input). The status chip is an app addition. | `web/src/panels/Header.tsx:36-68` (search 37-48, status 49-61, owner 62-67) | 1440/1024 shots; interaction probe |
| 5 | **P2** | Library typography / project selector | Nav labels ≈10 px; “ACTIVE AGENTS” / “TEMPLATES” ≈14 px semibold; project selector with dropdown chevron. | Nav labels 8 px/10 line; headings 11.5 px; template labels 11 px in 30 px rows; project row 215×32 static `DIV`, `cursor:auto`, no role/tabindex/chevron. Width 240 matches the spatial board (controls caption 260 conflicts). | `web/src/panels/NavRail.tsx:61`; `web/src/panels/LibraryPanel.tsx:38-42, 65, 96, 102`; `web/src/tokens/tokens.ts:169` | 1440/1024 shots |
| 6 | **P2** | Canvas chrome / decoration | Clean honeycomb + 5-lobe field, dashed bee routes, and the bottom hint `0.1x - 3.0x · Cmd+0 recentres…`; no filter chips or free-floating bees. | Extra `HIDE FINISHED / FOCUS BRANCH / 0/0` chips at 324–580, y 116–142; three free-floating ambient bees; full-field scan/sheen/glow; no bottom hint. Lobe core reads paler/brighter than the board's deeper ochre. | `web/src/flow/Canvas.tsx:294-303` (HiveField + scan + bees), `315-365` (filter chips); `web/src/flow/HiveField.tsx:48-78`; `web/src/styles/motion.css` | 1440 empty-state / normal shots |

Reference notes: the chat estimate is from the board image (header width 420 px; the body runs ≈188–676). The controls caption's 40 px target rule is explicit. The library/minimap size captions conflict with the spatial board pixels, so those sizes are not scored as defects.

## 5. Console and network at 1440 (task 6)

Instrumented 1440 load (same session that produced [1440x900.png](shots-live/1440x900.png)):

| Check | Result |
|---|---|
| `Runtime.consoleAPICalled` | **0** messages |
| `Runtime.exceptionThrown` | **0** exceptions |
| `Log.entryAdded` | **0** entries |
| `Network.loadingFailed` | **0** failed requests |
| HTTP response status ≥ 400 | **0** |
| WebSocket | CONNECTED at 595 ms; no disconnect banner or `E_WS_DISCONNECTED` toast |
| Document scroll | 1440×900; no horizontal/vertical overflow |

Only the 1440 load was instrumented. The 1280 and 1024 screenshots are viewport resizes of the same connected page, not separate console sessions.

## 6. Coverage and UNVERIFIED

| Status | Items |
|---|---|
| **Exercised** | CONNECTED boot at three viewports; screenshots; DOM boxes, overlaps, clipping; all running animations and reduced-motion startup; search open/Escape; starter draft; zoom in/recenter; Workspace/Swarm tabs; chat hide/reopen; details disclosure; console/network at 1440. |
| **Deliberately not exercised** | Submit prompt, template insert, and other nav destinations (server mutations / require provider key). Controlbar actions are disabled with no active run. |
| **UNVERIFIED — populated states** | Node cards (SuperAgent/Worker/Tool/Error), active-agent chips, real HUD counts/green %, graph edges, toast/refusal, approval overlay, streaming/replay/history, populated minimap nodes and mask. Live has no run, so the agent-card and states boards were not rendered. |
| **UNVERIFIED — interaction states** | Hover, focus-visible, active/pressed, keyboard traversal, and enabled Spawn/Re-route/Cancel/Kill states. |
| **UNVERIFIED — reference sizes** | Authoritative library/minimap sizes: 240/220×150 drawn vs 260/260×170 captioned. Need a single spec source. |
| **UNVERIFIED — environment** | OS-level reduced-motion (tested via CDP emulation, not a real OS setting); browsers/OS other than Chrome 156 on Windows; 1023 px (just below the `lg` breakpoint); viewport heights below 800. |
| **UNVERIFIED — long run** | No soak test, no repeated reconnect cycle; console clean only for the single 1440 load. |

## 7. P0 / P1 / P2

**P0 — none.** The page loads, connects, and all exercised controls respond; no crash, no console error, no failed request.

**P1**
1. **Minimap is effectively broken at 1024 and partly covered at 1280:** blank white surface (CSS dot grid overridden by inline `background`), 36×86 occlusion at 1280 and 164×136 at 1024; 158×16 of the empty-state copy hidden at 1024 when the chat is hidden.
2. **Control bar has no primary hierarchy and misses the 40 px target rule:** 30 px buttons, all outline, two Re-route affordances, 71 px wrapped bar at 1024; 21 of 27 visible buttons are under 40 px high.
3. **Chat core is composed too low and too close to the bottom chrome:** 400×480 at y=236–716 vs board ≈188–676; only 20 px above the control bar at 1024; composer lacks the reference microphone/attachment affordances.

**P2**
1. Topbar lacks the reference notification bell and operator menu/chevron; adds a `CONNECTED` chip.
2. Library type scale is undersized (8 px nav, 11.5 px headings, 11 px templates) and the project selector is a static 215×32 `DIV` with no chevron/role/tabindex.
3. Canvas adds filter chips, three floating ambient bees, scan/sheen/glow, and no bottom hint; lobe core is brighter than the board.
4. Minimap dot grid is missing due to the inline `background` reset; empty mini-map is a blank white card.
5. Hidden `state-pulse` loop runs 1.6 s infinite on a closed `<details>` element; six visible decorative loops run concurrently despite the “one calm movement” note.
6. Text glyphs `× + − 0 ⤢` stand in for line icons; minor iconography inconsistency.
7. Empty-state copy collides with the minimap at 1280 (30×16) and 1024 (158×16) when the chat is hidden; the default chat-visible state suppresses the note cleanly (`Canvas.tsx:481`).
