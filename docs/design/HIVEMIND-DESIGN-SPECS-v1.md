# HiveMind Design Specs v1

**Status:** audit draft · **Tree:** `hiverouter-v1` HEAD `1129349` · **Screenshots:** `docs/evidence/live/install-ui-1440x900.png`, `install-ui-375x812.png` (1.2.0-dev install; record `install-1.2.0-dev.json`).
**Method:** static read of the shell, panels, styles, state and tests, plus the two supplied screenshots. No browser session, no test execution, no screenshot creation. `V` = verified; `P` = proposed; `U` = unverified.
**Binding constraint:** the cream / honey / forest / ink FigCraft–HoneyGlass direction is retained. This spec does not invent screens, data, or screenshots.

---

## 1. Layout shell

### 1.1 Verified structure
`App.tsx` mounts a three-band shell inside a full-height flex column (`App.tsx:168-211`):

```
┌──────────────────────────────────────────────────────────────┐
│ Header  52px  mark/wordmark · chat · search · status · avatar │
├──────┬───────────────┬───────────────────────────────────────┤
│ Nav  │ Library       │ main                                  │
│ 72px │ 240px (<lg:  │ banners (mock / offline / snapshot)    │
│      │  hidden)      │ content (nav × tab)                    │
│      │               │ workspace: Canvas + ChatCore           │
│      │               │            + ControlBar + Scrubber     │
└──────┴───────────────┴───────────────────────────────────────┘
        ToastRegion · ApprovalOverlay · HiveMindSplash (overlays)
```

- Shell: `flex h-full min-h-0 flex-col bg-canvas text-ink-900` (`App.tsx:169`).
- Header: `h-[52px] shrink-0` (`Header.tsx:50`); geometry pinned 52 (`tokens.ts:166-183`, `designParity.test.ts:17-33`).
- Nav rail: `w-[72px] shrink-0` (`NavRail.tsx:27`).
- Library: `hidden ... lg:flex`, inline `width: LAYOUT.library` = 240 (`LibraryPanel.tsx:82-86`).
- Main: `min-w-0 flex-1` (`App.tsx:177`), so it can shrink without pushing the rail/library.
- Settings content: scroll container `p-4`, centered `max-w-[920px]` (`SettingsPanel.tsx:401-404`).
- Overlays are siblings of the shell: `ToastRegion`, `ApprovalOverlay`, `HiveMindSplash` (`App.tsx:208-209,227`).

### 1.2 Shell rules (P)
1. Header and rail are persistent; the library is persistent only ≥1024px.
2. Exactly one vertical scroll owner per surface. Main content owns its scroll; the page body does not.
3. Never let a panel change the 52/72/240 shell constants; use `LAYOUT` (`tokens.ts:166-183`).
4. Banners stack above content in this order: mock strip, connection/offline, snapshot degraded. They must never cover the header or the control bar.
5. Modals render above toasts; toasts drop below a blocking approval (`uiWiring.test.ts:245-250`, `toastStackZClass`).

---

## 2. Header

### 2.1 Anatomy (V)
| Region | Content | Behavior |
|---|---|---|
| Brand | 28px hex mark + `HiveMind` 17px semibold (`Header.tsx:52-65`) | Static; not a link. |
| Chat | Pill button, chat glyph + `SHOW CHAT` / `CLOSE` | On Workspace toggles chat; elsewhere navigates to Workspace and shows it (`Header.tsx:12-17,34-43`; tested `uiWiring.test.ts:147-162`). |
| Search | 32px icon button, `aria-label="Search agents"`, `aria-expanded` | Opens a 320px popover anchored right under the header (`Header.tsx:102-113,136-176`). |
| Connection | Pill, `aria-live="polite"` | `open` → verify border/text; connecting/reconnecting → amber border + ink-800; else signal (`Header.tsx:114-126`). |
| Identity | 32px honey avatar `HM`; `OPERATOR` label hidden below `sm` | Static; no menu. |

### 2.2 Search popover contract
- Input is 12px, autofocus, Escape closes (`Header.tsx:141-153`).
- Results are the first 6 tasks whose label contains the query, case-insensitive (`:45-47`).
- Empty result: `No agents in this run.` (`:155-157`).
- Selecting closes the popover and selects the task (`:163-166`).
- **Defect:** no click-outside dismissal and no focus trap; tabbing out leaves the popover floating (`:136-176`). Required: close on outside pointerdown, `Escape` at document level while open, return focus to the trigger, and either trap or explicitly make it non-modal with `aria-expanded`.

### 2.3 375px behavior (V, screenshot)
- Wordmark truncates to `H`; mark + `SHOW CHAT` + search + `CONNECTED` + `HM` fit.
- `OPERATOR` is hidden by `hidden sm:block` (`Header.tsx:131`).
- No hamburger, no drawer, no compact brand rule. This is a compressed desktop header (D2/D3).

---

## 3. Navigation rail

- Seven destinations in fixed order: Workspace, Agents, Data, Models, Routing, Remote, Settings (`NavRail.tsx:9-17`; `App.tsx:21`). Note the doc comment says "six" (`NavRail.tsx:1`) — doc drift (D7).
- Each item is a full-width button: 48×48 icon tile, 11px label, active tile `ink-900` fill with inverse icon, inactive hover hairline/panel-alt (`NavRail.tsx:48-67`).
- Active item: `aria-current="page"`, `tabIndex=0`; others `-1`; ArrowUp/ArrowDown roving focus (`:28-36,45-46`).
- Settings is selected in both screenshots (dark tile, label `Settings`).
- **Rules (P):** keep one active destination; preserve arrow keys and Home/End; when a compact rail ships, keep accessible names on every icon-only control; active state must be more than color (fill + icon inverse + label weight).

---

## 4. Library rail

Visible ≥1024px only (`LibraryPanel.tsx:84`). Width 240.

### 4.1 Verified contents (1440 screenshot + source)
1. Project chip: `Project: HiveMind AI` (`LibraryPanel.tsx:87-91`).
2. Workspace tablist in a 2×2 grid: Workspace, Swarm, Routing, History (`:93-96`); click and Arrow/Home/End call `setWorkspaceTab` so the nav moves to Workspace with the tab (`uiWiring.test.ts:126-143`). Active tab is honey tint.
3. `AGENTS IN RUN` section: up to 8 agents, `+N more in this run` (`:170-175`); empty state from screenshot: `NO ACTIVE AGENTS`.
4. `TEMPLATES · N` gallery (§5).
5. Honey `TIP` card: "Drag a worker onto another super-agent to re-route its context stream live." (`:225-228`; 1440 screenshot).
6. `<details>` `STATE & SETTINGS` with the state legend (`:230-234`), collapsed in both screenshots.

### 4.2 Rules
- The library is a navigation/inspection rail, not a second page. Selecting a tab must always move `nav` to `workspace` (`uiWiring.test.ts:139-143`).
- Never hide the only route to a feature at a breakpoint. Today the library is the only route to the 8-template gallery and the state legend, so hiding it at <1024 removes those features (D3).
- Scroll the rail independently (`overflow-y-auto`), keep the project chip and tabs pinned if the rail grows.

---

## 5. The 8-template gallery (workspace/library)

### 5.1 Verified behavior
- `GET /api/templates` is decoded by `decodeTemplateCatalog`; it throws on a payload without `templates` rather than inventing a catalog (`templateCatalog.ts:15-45`).
- Backend returns every catalogued template with `insertable` true only for `content-writer`, `data-analyst`, `code-reviewer`, `chatbot` (`backend/crates/hiverouter-core/src/http.rs:1395-1419`).
- UI shows **all eight** rows and a count `TEMPLATES · 8` (`LibraryPanel.tsx:179-181`; 1440 screenshot).
- Insertable rows call `insertTemplate(id)` → store `run('TEMPLATE_INSERT', { template_id })` (`LibraryPanel.tsx:206-210`; `store.ts:2342-2345`).
- Non-insertable rows are `disabled` + `aria-disabled`; title says "`<name>` is catalog-only; it has no insertable plan"; a `CATALOG` chip is shown (`LibraryPanel.tsx:198-204,216-218`). The 1440 screenshot shows exactly four `CATALOG` chips: agent, planner, subagent, superagent; chat, code-reviewer, content-writer and data-analyst have none.
- Loading, error and zero states exist (`:182-191`); the zero copy is "No insertable templates." even when the catalog itself is empty (D9).
- Tests cover the decoder only: 8 rows / 4 insertable, malformed rows skipped, missing array throws (`templateGallery.test.ts:13-43`). **No DOM test asserts the rendered 8 rows, the disabled state, or a successful click-to-insert.** Click behavior is therefore `U` (verified statically, not exercised).

### 5.2 Required behavior (P)
| State | Requirement |
|---|---|
| Loading | Skeleton rows or `Loading templates…`; do not render `TEMPLATES · 0` as if it were data. |
| Loaded, N>0 | Show all N, ordered as the server returns; count equals rendered rows. |
| Catalog-only | Keep the row discoverable by keyboard/touch. Do not use a disabled `<button>` whose only explanation is `title`. Use a non-disabled button with `aria-disabled="true"` and visible helper text, or a non-interactive row with a `CATALOG` badge. |
| Empty | "No templates available." plus retry; distinguish transport failure from an empty catalog. |
| Error | `role="alert"`, named failure text, retry. |
| Insert | Disable the row while pending; on refusal show the named code; never optimistically insert a node. |
| Narrow viewport | Provide a drawer/route to the gallery; do not silently remove it at <1024. |

---

## 6. Workspace surface

### 6.1 Verified composition
- Default workspace tab: `<Canvas />` + `<ChatCore />` inside a relative, min-h-0 container (`App.tsx:144-148`).
- Bottom overlay only on `workspace/workspace`: ControlBar then Scrubber (`App.tsx:196-199`); history shows Scrubber only (`:202-204`).
- Workspace tabs: workspace, swarm, routing, history (`App.tsx:22`). Swarm mounts `SwarmPanel`; routing mounts `RoutingPanel`; history mounts `HistoryPanel` + Canvas when a run exists, otherwise an honest empty card "Nothing to replay yet" with a `Go to Workspace` action (`App.tsx:113-158`).
- Chat visibility is store-backed; the header never changes hidden state off-Workspace (`Header.tsx:8-17`).
- Canvas geometry: minimap 220×150 (`tokens.ts` LAYOUT); hidden <1280 (`index.css:137-142`). Canvas is the skip target (`App.tsx:127`).
- Control bar controls are disabled unless their precondition is visible; SPAWN shows the gate reason (`ControlBar.tsx:74-95`). Spawn wiring and the replay reason are tested (`uiWiring.test.ts:254-298`).

### 6.2 Rules (P)
- The canvas must remain the dominant surface at 1440 (no permanent chat overlay covering nodes); chat is a 400×480 panel, draggable/resizable if it overlaps.
- Bottom chrome must reserve a safe area so it does not cover the last row of nodes.
- Every control that can be disabled must expose a visible reason (title alone is insufficient).
- Empty canvas ≠ empty history; keep the two messages distinct.

---

## 7. Agents surface

- Mount: `AgentsPanel` (`App.tsx:153`).
- Verified supporting panels exist: roster, hierarchy, profiles, messages, tool activity, memory, A2A inbox, proposals, lessons (`web/src/panels/*`). Their exact visual composition was **not** inspected in this pass; treat layout below as `P`.
- Required anatomy: roster/list + hierarchy view + selected-agent detail + tool activity + approvals handoff. Empty state must say no active agents and offer spawn only when the spawn gate allows it.
- Spawn must carry the real gate reason; replay is read-only with the honest reason (`uiWiring.test.ts:254-298`).
- Do not show a tool-activity panel without data; use an explicit empty state, not a zeroed chart.

---

## 8. Data surface

- Mount: `DataPanel` (`App.tsx:155`). Related panels: `MemoryBanksPanel`, `LessonsPanel`, `A2AInboxPanel`, `ProjectsPanel`, `LibraryPanel` (`web/src/panels/`).
- Required (P): tabbed or sectioned layout for memory banks, lessons, A2A inbox, projects; each has empty/loading/error/degraded; no fabricated memory rows; counts are server-derived.
- Evidence of wiring is structural (mounted components and tests `lessons.test.ts`); live visual state `U`.

---

## 9. Models surface

- Mount: `ModelsPanel` (`App.tsx:157`).
- Must not relabel the provider. The provider boundary copy is explicit: "The server reports the configured provider; this panel never relabels it… never claims a balance or capability the endpoint did not report." (`SettingsPanel.tsx:460-466`).
- Provider key state is displayed in Settings, not here: `Key missing` with the `keyUnprobedHint` (`SettingsPanel.tsx:367-370`; 1440 screenshot).
- Required (P): model list shows server-reported ids only; no invented context windows or pricing; unavailable models are visibly unavailable with a reason.

---

## 10. Routing surface

- Two mounts: workspace tab `routing` (`App.tsx:116`) and nav `routing` (`:159`), both `RoutingPanel`.
- Routing semantics live on the canvas (bee = routing arrow) and in `flow/` (`BeeEdge`, `graph`, `RemoteLane`); the panel must not invent routes.
- Required (P): show current route, selected agent, and gate/refusal; empty state when no run; every reroute action gated and explained.

---

## 11. Remote surface

- Mount: `RemotePanel` (`App.tsx:161`); supporting `DaemonPanel`, `RemoteLane` (`web/src/panels/DaemonPanel.tsx`, `web/src/flow/RemoteLane.tsx`).
- Required (P): connection state, lane status, daemon health, and explicit offline/refusal copy; never imply a remote executor is healthy from a socket alone.
- Live visual state `U`.

---

## 12. Settings surface

### 12.1 Structure (V)
- Five tabs: Overview, Provider, Appearance, Diagnostics, FigCraft (`SettingsPanel.tsx:14-21`).
- Tablist: `role="tablist"`, `aria-label="Settings sections"`, buttons `role="tab"` with `aria-selected`, `aria-controls`, roving `tabIndex`, ArrowLeft/Right cycling and focus for keyboard selection (`SettingsPanel.tsx:309-330,406-436`). Click selection does not call `selectTab` (no focus move needed; browser focus is already on the button).
- Title: `Settings` (24px panel title); outer `p-4`, `max-w-[920px]` (`:401-405`).
- Provider tab in both screenshots: key status card (`Key missing`, provider deepseek, model deepseek-flash, thinking enabled, reasoning high), configured-provider boundary card, saved provider profile form (API TYPE, BASE URL, WIRE MODEL, ACCOUNTING MODEL, Save), and the restart notice.
- Save posts `{name, api_type, base_url, wire_model, model}` with `x-hivemind-token`; on success shows the restart-required message; errors surface as failure text (`SettingsPanel.tsx:165-203`).
- Key refresh calls `probeSettings` (one SETTINGS_STATUS per open), health refresh calls `probeHealth` (`:332-344,374-398`; store `:2535-2545`).
- Appearance tab includes ambient motion intensity (`:503`), diagnostics embeds `DiagnosticsPanel`, FigCraft embeds `FigCraftPanel` → `FigCraftGallery` (`:538-544`).

### 12.2 Rules
- Never place the API key in the UI or the profile JSON; the key is DPAPI/environment, and the restore command is `tools/set-provider-key.ps1` (verified file exists; referenced by the save notice `SettingsPanel.tsx:193-196`).
- `Key missing` is an error-adjacent state, not a footnote. Pair it with the exact recovery action and a link/copy command; today the user must know the script (P).
- After a save that requires restart, keep the notice sticky next to the affected status; do not show both "Saved" and an unchanged "Key missing" without explaining that they are different layers (profile file vs key presence). This is current screenshot behavior (V) and should be clarified (P).
- Tab labels are 12px uppercase; at 375 they wrap to three rows (V). See §14.

---

## 13. State contracts

| State | Component / carrier | Verified behavior | Required design |
|---|---|---|---|
| Empty | `EmptyState`; library; history; search | Dashed `panel-alt` card, 17px ink-500 (`StatePack.tsx:9-14`); `Nothing to replay yet` card (`App.tsx:150-158`); `NO ACTIVE AGENTS` screenshot | Name the empty thing and the next valid action; never show a zeroed metric. |
| Loading | `LoadingState`; `Loading templates…`; splash; settings pending; snapshot loading | `role="status"` (`StatePack.tsx:17-21`); templates text (`LibraryPanel.tsx:182-184`); splash 1.6s floor (`HiveMindSplash.tsx:41-54`); `settings.status === 'pending'` (`SettingsPanel.tsx:346`) | Skeleton or status text; no fake percentage unless measured; no layout shift on load. |
| Error | `ErrorState`; template alert; settings refusal; offline banner | `role="alert"`, signal border/text (`StatePack.tsx:25-31`); template error role alert (`LibraryPanel.tsx:185-188`); refusal code + message + how_to_fix (`SettingsPanel.tsx:347-356`); banner role alert (`App.tsx:181-187`) | Show named code, human sentence, recovery action; keep previous good data visible if safe. |
| Degraded | `SnapshotBanner`; health `unknown`; missing key | Banner only for loading/degraded/failed; ready/idle render nothing (`SnapshotBanner.tsx:15-27,47-59`); re-sync disabled while loading (`:81-88`); health card (`SettingsPanel.tsx:374-398`); install record health degraded + missing key | Degraded must be visible at shell level (see D1), not only inside Settings. |
| Approval | `ApprovalOverlay` | `role="dialog"`, `aria-modal="true"`, focus trap, focus restore, Escape dismiss when allowed, queue label and previous/next, stale banner (`ApprovalOverlay.tsx:207-380`); tested (`uiWiring.test.ts:211-251`) | One blocking dialog for the queue; no background mutation while open; toasts below it; approval actions name the tool/effect. |
| Offline / reconnecting | Header pill + `role="alert"` banner | Banner on error/closed with refusal message or offline copy; reconnect banner while reconnecting; mutations blocked by store (`App.tsx:126-137,181-187`) | Distinguish "server unreachable" from "server up, provider missing"; keep a retry control. |
| Replay | `ReplayBanner`, header `Return live` | "Replay is read-only. Exit replay to spawn a child."; header return action (`uiWiring.test.ts:278-298`) | Replay must be visually unmistakable on every surface; mutation controls disabled with reason. |

---

## 14. Responsive rules

### 14.1 1440×900 (V, screenshot)
- Header 52; rail 72; library 240; main ≈1128.
- Settings content 920 centered; five settings tabs in one row; provider cards comfortable; library shows 8 templates with 4 `CATALOG` chips.
- Bottom control bar/scrubber are not visible in the Settings screenshots because they only render on Workspace.

**Rules:** preserve the 240 library at ≥1280; allow the main column to breathe; do not exceed 920 for Settings text; keep five tabs in one row at ≥1024.

### 14.2 375×812 (V, screenshot + measured code)
- Header 52; rail 72; library hidden; main ≈303; Settings `p-4` → ≈271 content.
- Wordmark collapses to `H`; `OPERATOR` hidden; status pill and avatar still present.
- Settings tabs wrap into **three rows** (Overview+Provider / Appearance / Diagnostics+FigCraft).
- Provider key status label wraps; the 140px `dl` label column leaves ≈107px for values at this width (`SettingsPanel.tsx:358`).
- Body copy at 17/28 fills most of the viewport; the first card alone is taller than the remaining fold.
- Templates, project chip, state legend and TIP are **not reachable** (library hidden, no drawer).
- The rail still consumes 72/375 ≈19% of width; the remaining 303px is desktop content.

### 14.3 Required mobile behavior (P)
| Rule | Target |
|---|---|
| Breakpoints | `lg` 1024 (library drawer), `md` 768, `sm` 640; test 375 and 768 explicitly. |
| Navigation | Either 56px icon rail or a bottom bar; never keep a 72px desktop rail. All 7 destinations remain reachable; Settings may move into an overflow sheet. |
| Library | Off-canvas drawer opened from a labeled control; contains project, workspace tabs, agents, all 8 templates, TIP, state legend. Focus trap + Escape + backdrop close. |
| Brand | 28px mark alone with accessible name; no clipped wordmark. |
| Header | Keep chat, search, connection, avatar; move `SHOW CHAT` to icon+label only if it fits; status uses a dot + short label (`Online`/`Degraded`/`Offline`) rather than a long pill. |
| Settings tabs | Horizontally scrollable tab row with snap and edge fade, or a select; never a 3-row wrap. |
| Definition lists | One column below 480; labels above values; no fixed 140px column. |
| Type | Body 16/24; meta ≥12px; buttons ≥44px touch target. |
| Canvas/chat | Chat becomes a full-width sheet or bottom drawer; canvas remains reachable; minimap already hidden <1280. |
| Tables | Convert to stacked rows; no horizontal page scroll. |

---

## 15. Component anatomy

| Component | Anatomy | States | Rules |
|---|---|---|---|
| Top bar | 52px, 12px h-padding, brand left, actions right, 12px gaps | default; search open | One row; actions `shrink-0`; brand `min-w-0` with intentional compact variant. |
| Status pill | 11px mono uppercase, 1px border, pilled, 2/8 padding | connected / reconnecting / offline / degraded | Color + text; never color alone; `aria-live=polite`; dot optional. |
| Nav item | 48×48 tile (10px radius) + 11px label | active / hover / focus | Active fill ink-900 + inverse icon; focus ring 2px ink-700 offset 2. |
| Library tab | 2-col grid button, honey-100 selected, hairline border | selected / hover / focus | Roving tabIndex; arrow/Home/End; moving tab moves nav. |
| Template row | 32px min-height, 8px radius, hairline, 12px icon, truncating label, optional `CATALOG` meta | insertable / catalog-only / pending / error | All rows visible; explanation visible, not title-only; no disabled focus loss. |
| Settings tab | 36px height, 10px radius, icon+label, 12px uppercase | selected / hover / focus | One row ≥1024; scroll <640. |
| Surface card | panel, 1px hairline, 12px radius, 12px padding, optional label | default / selected / error | One shadow family; no nested cards deeper than 2. |
| Definition list | 140px label + 1fr value, 11px mono labels, 13px mono values | default / missing / loading | Stack <480; values wrap/break-all for ids/URLs. |
| Button | 32–40px height, 10px radius, 12px uppercase label | primary honey-300; secondary panel+hairline; disabled; busy | Min 44px touch; disabled reason visible; busy label `Saving…`; never remove focus ring. |
| Input | 8px radius, hairline, panel-alt, 12px text | default / focus / error / disabled | Label/placeholder distinction; `focus-visible:ring-2`; no key material. |
| Toast | panel, hairline, shadow e2, 12px text, z-50 (z-30 under approvals) | info / success / error | Max 3 visible; dismiss; not the only carrier of a refusal. |
| Modal | panel, 16px radius, e3, `role=dialog`, focus trap | approval / confirm | One modal at a time; restore focus; Escape only when dismiss allowed. |
| Splash | HoneyGlass pane, hex field, wordmark, status line, progress well | connecting / open / offline | 1.6s min; skip added; indeterminate or measured; reduced motion respected. |

---

## 16. Defect register (UI)

| # | Defect | Evidence | Fix owner |
|---|---|---|---|
| 1 | Header "CONNECTED" and splash "The hive is awake" mean socket-open only, while health is degraded and the key is missing. | `Header.tsx:114-126`; `HiveMindSplash.tsx:28-33`; `install-1.2.0-dev.json:19-25`; screenshots | UI + status model |
| 2 | 375 clips the wordmark to `H`; no compact brand variant. | 375 screenshot; `Header.tsx:51,64` | UI |
| 3 | <1024 removes the entire library, including the 8-template gallery; no alternate route. | `LibraryPanel.tsx:84`; 375 vs 1440 screenshots | UI |
| 4 | 375 is a compressed desktop: 72px rail, 3-row settings tabs, fixed 140px `dl` column, 17px body. | 375 screenshot; `NavRail.tsx:27`; `SettingsPanel.tsx:358,409` | UI |
| 5 | Catalog-only templates: disabled `<button>` + title-only explanation; not keyboard/touch discoverable. | `LibraryPanel.tsx:198-204,216-218`; 1440 screenshot | UI/a11y |
| 6 | Splash progress is a 3-bucket fake (18/46/100) exposed as `progressbar`; no skip. | `HiveMindSplash.tsx:41-65,117-126` | UI |
| 7 | Header search popover has no outside-dismiss and no focus management. | `Header.tsx:136-176` | UI/a11y |
| 8 | Template count reads 0 while loading; empty copy conflates "no catalog" with "no insertable". | `LibraryPanel.tsx:180-191` | UI |
| 9 | Nav rail comment says six destinations, renders seven. | `NavRail.tsx:1` vs `:9-17` | Docs |
| 10 | Click-to-insert for templates has no DOM test; only the decoder is tested. | `templateGallery.test.ts:13-43` | Tests |

Defects 1–8 are real product defects, not documentation nits. No web product code was changed by this audit.

---

## 17. Acceptance checklist

### Shell and navigation
- [ ] Header is exactly 52px at 1440 and 375; no wrap or horizontal overflow.
- [ ] Brand is an intentional lockup at both widths (no accidental `H`).
- [ ] All 7 nav destinations are reachable at 1440 and 375; active state has `aria-current` and a visible non-color cue.
- [ ] Skip link targets the canvas on workspace/workspace and `#main-content` elsewhere (`App.tsx:126-132`).
- [ ] Browser hash routing round-trips nav/tab and does not fight back/forward (`App.tsx:53-96`).

### Library and templates
- [ ] At 1440 the library shows project, 4 workspace tabs, agents, `TEMPLATES · 8`, TIP, state legend.
- [ ] Exactly 8 rows render; exactly the 4 backend-insertable ids are enabled; 4 show `CATALOG`.
- [ ] The catalog-only explanation is available to keyboard and touch users.
- [ ] Inserting a row calls `TEMPLATE_INSERT` once and never fakes a node.
- [ ] Loading, empty catalog, transport error and malformed payload each have distinct copy; malformed payload throws and is shown as an error.
- [ ] At 375 the gallery is reachable via drawer/route, not removed.

### Settings
- [ ] 5 tabs, `role=tablist`, arrow keys cycle, one tab in the tab order.
- [ ] Provider status shows key presence, provider, model, thinking, effort; missing key is prominent.
- [ ] Save sends the session token header, shows restart notice when required, and surfaces server refusal.
- [ ] No API key is present in DOM, storage, profile JSON, or logs.
- [ ] Appearance motion control persists; reduced motion applies globally.
- [ ] Diagnostics and FigCraft tabs mount real panels (no placeholder).
- [ ] At 375 tabs are scrollable/one row, `dl` stacks, touch targets ≥44px.

### States
- [ ] Empty states name the missing thing and the next action.
- [ ] Loading uses `role=status` and does not claim measured progress.
- [ ] Errors use `role=alert` with code + sentence + recovery.
- [ ] Degraded is visible at shell level, not only after opening Settings.
- [ ] Offline blocks mutations and offers retry; reconnecting is distinct from offline.
- [ ] Approval queue traps/restores focus, Escape behavior matches dismissability, toasts sit below the modal.
- [ ] Replay is visibly read-only; spawn shows the replay reason.

### Accessibility
- [ ] Global focus ring 2px ink-700 offset 2 remains visible on every interactive element.
- [ ] All interactive targets ≥24px (desktop) / ≥44px (touch).
- [ ] No text below 11px on operational controls; no amber small text.
- [ ] Every icon-only control has an accessible name.
- [ ] 375 and 1440 both pass a zoom-200 check without loss of function.
- [ ] `prefers-reduced-motion` and the in-app switch both stop ambient/splash motion.

### Truth and verification
- [ ] Header status incorporates provider/health state or is relabeled to "server connected".
- [ ] Splash percentage is removed or backed by real progress.
- [ ] The `install-ui-*.png` evidence matches the shipped build; screenshots are regenerated after fixes.
- [ ] Tests cover template click/disabled behavior, 375 shell, and header status semantics.

---

## 18. Verified vs proposed

**Verified (V):** shell composition and dimensions, header/nav/library/settings structure, template decoder and backend insertable set, screenshot contents at both widths, state component roles, approval focus behavior (source + tests read), existing token/geometry tests, install health record.

**Proposed (P):** mobile navigation/library drawer, compact brand lockup, shell-level degraded status, template row interaction pattern, status-color mapping, splash skip/indeterminate progress, settings tab scroll, definition-list stacking, type/touch floors.

**Unverified (U):** every panel's live visual state beyond the two Settings screenshots; template click behavior at runtime; actual splash contrast over translucency; device scale/browser chrome of the supplied screenshots; whether the library drawer is planned elsewhere. No browser or test command was run in this audit; no screenshot was fabricated.
