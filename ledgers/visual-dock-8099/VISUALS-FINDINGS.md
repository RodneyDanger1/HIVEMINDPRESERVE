# VISUALS — Chat dock audit (hiverouter-v1 @ 8099)

Read-only. Headless Chrome via Playwright (no visible window). App was **not** running on 8099 at start; launched `dist\HiveMind\hiverouter.exe --bind 127.0.0.1:8099 --data-dir ledgers\visual-dock-8099\data --workspace-root ledgers\visual-dock-8099` hidden (PID 26984, still running). No source edits.

Artifacts: `ledgers/visual-dock-8099/results.json`, `shots/01…10`, `dock-audit.mjs`.

Evidence baseline: console = 0 errors/warnings, pageerrors = 0, failed requests = 0 at 1440 and 375.

## Defects (8 max)

### D1 — First-run dock opens at 300px, not 390px (High)
- Shot: `shots/01-1440-initial.png`, `shots/08-375-initial.png`
- Actual: `chat h=300` with `localStorage['hivemind.chat-height']=null`.
- Cause: `readChatHeight()` does `Number(localStorage.getItem(...))`; `Number(null) === 0` → clamped to `MIN_CHAT_HEIGHT` (300). `web/src/chat/ChatCore.tsx:35-38`
- Expected: first-run height is the 390px default in the same function; only clamp when a real finite stored value exists.

### D2 — Resize handle is occluded by the header; dragging resizes nothing and selects header text (Critical)
- Shots: `02-1440-after-drag-usual.png`, `03-1440-hover-after-drag.png`, `04-1440-controls.png` (blue text selection on “Hive AI Assistant”), `05-1440-state.png`
- Actual: 120px upward drag on the separator centre `(876,603)` leaves height 300→300, stored height `null`; the drag paints a text selection in the header instead.
- Cause: separator renders at section content top (`y=600-606`), then the header applies `-mt-3` and overlaps it (`y=588-632`) and paints above it. Pointerdown lands on the header, so `startResize` never fires. `web/src/chat/ChatCore.tsx:590-603` (separator at :590-600, header `-mt-3` at :601-603)
- Expected: the handle must be the topmost hit target (raise z-index above the header, or remove the header's negative top margin); dragging up 120px gives `chatHeight ≈ 420`, keeps the bottom edge fixed, and persists `hivemind.chat-height=420`.

### D3 — Resize affordance is only 56×6px (High)
- Shot: `shots/01-1440-initial.png` (handle not visibly separable from header)
- Actual: measured `56×6` at `848,600`; recommended minimum 24×24.
- Cause: `h-1.5 w-14` `web/src/chat/ChatCore.tsx:595`
- Expected: ≥24px tall (ideally 44px on touch) grabbable strip with visible hover affordance; it is currently hidden under the header (D2).

### D4 — 375 layout: fixed 300px panel clips the empty-state copy mid-line and hides the starter prompts (High)
- Shots: `shots/08-375-initial.png`, `shots/09-375-controls.png`
- Actual: `section` stays `300px` tall; transcript shows “…before any worker” cut mid-sentence and the “MAP THIS REPOSITORY / REVIEW AN API CHANGE” buttons are below the fold in an unmarked internal scroll. `max-[640px]:h-auto` never applies.
- Cause: inline `style.height` at `web/src/chat/ChatCore.tsx:584` overrides the class `max-[640px]:h-auto` at `:582`.
- Expected: ≤640px uses content height (`height:auto`) bounded by `max-h-[calc(100dvh-84px)]`, so the empty-state sentence and both starter buttons render whole, composer docked below them.

### D5 — Hidden state: floating “Show chat” pill overlaps the HyperAgent HUD, and there are three reopen controls (Medium)
- Shot: `shots/06-1440-hidden.png`
- Actual: floating pill at `x817.5,y68,w117.1,h34` sits on top of the `HYPERAGENT · NO METRICS` HUD (`~645-1105 × 66-104`), obscuring its right portion; the header (“SHOW CHAT”) and canvas toolbar (“SHOW CHAT”) duplicate the same action.
- Cause: `if (!chat.visible)` returns `absolute inset-x-0 top-4 z-30` overlay `web/src/chat/ChatCore.tsx:480-491`; also `Canvas.tsx:561` and `Header.tsx:104`.
- Expected: reopen control does not overlap the HUD (e.g. bottom-centre pill where the chat was), and no redundant third control.

### D6 — Footer/toolbar hit targets below touch minimum (Medium)
- Shot: `shots/09-375-controls.png`
- Actual: `HIDE 43.7×24`, `CONTROLS 71.4×24`, `STATE & SETTINGS 126.9×24`; composer attach/stop/submit `32×32`.
- Cause: `px-2 py-1 text-[10px]` on tabs `web/src/chat/ChatCore.tsx:1002-1025`; icon buttons `h-8 w-8` at `:962-996`.
- Expected: ≥44×44 touch targets on ≤640px (at minimum 24×24 with spacing for AA); tabs should not be 24px-tall slivers on phone.

### D7 — HIDE's accessible name is not its visible text (Low)
- Shot: `shots/04-1440-controls.png`, `06-1440-hidden.png`
- Actual: visible “HIDE”, accessible name “Hide the chat dock” (aria-label overrides the text). Speech control “click HIDE” cannot match; the header toggle announces “Show Hive chat”/“Hide Hive chat”, a third naming scheme.
- Cause: `aria-label="Hide the chat dock"` `web/src/chat/ChatCore.tsx:1020-1024`.
- Expected: accessible name starts with the visible label (“Hide chat”), consistent across the dock footer, header toggle, and reopen pill.

### D8 — Focus is dropped to `<body>` on hide/show (Low)
- Shots: `shots/06-1440-hidden.png`, `07-1440-shown-again.png`
- Actual: after clicking the floating “Show chat”, `document.activeElement` is `<body>` (the clicked button unmounted). Same for HIDE.
- Cause: conditional mount/unmount in `ChatCore.tsx:480-492` and `:637` with no focus handoff.
- Expected: focus moves into the reopened chat (composer or panel) or back to the toggle that is still mounted; never silently to body.

## Verified passes
- Width at 1440: exactly 560px, centred, bottom 888 in a 900 viewport (12px gap).
- Controls tab: opens, `aria-expanded=true`, tray 149/149px (no internal clipping), panel grows 300→510 while top moves 588→378 and **bottom stays 888** — desktop expansion correctly pushes the chat up.
- State & settings tab: renders provider/model/key/reasoning/version plus four state dots, no clipping (`05-1440-state.png`); values honestly `—` on a fresh data dir.
- HIDE works; graph view remains usable; SHOW restores the panel with the previously open tab (510px).
- 375: panel is 329.9px wide (margins 22.5), fully within viewport with Controls open (510px tall, bottom 586.5 < 667); no horizontal clipping.
- No console errors, page errors, or failed requests at either breakpoint.

## Environment note
8099 was dead on arrival (`ERR Unable to connect`); only DSH's 54610 was listening. App was started hidden from the existing release binary; it remains running as PID 26984. If that process is not expected, stop it before any re-run.
