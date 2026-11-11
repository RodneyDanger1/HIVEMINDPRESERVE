# HiveMind Brand System v1

**Status:** audit draft · **Audited tree:** `hiverouter-v1` HEAD `1129349` · **Scope:** the shipped operator UI
**Evidence class:** `V` = verified by reading code/screenshots in this tree; `P` = proposed rule, not yet implemented.
**Screenshots inspected:** `docs/evidence/live/install-ui-1440x900.png`, `docs/evidence/live/install-ui-375x812.png`
(both from the 1.2.0-dev install record `docs/evidence/live/install-1.2.0-dev.json`).

This document preserves the existing cream / honey / forest / ink direction. It does **not** invent a new
identity and does **not** claim a screenshot that does not exist. Where code contradicts the direction, the
contradiction is recorded as a defect in §12.

---

## 1. Brand attributes

| Attribute | Meaning in this product | Observable carrier (V) |
|---|---|---|
| **Warm precision** | Paper-like surfaces, warm ink, one honey accent; no neon, no generic slate SaaS. | `--color-canvas #F8F4EB`, honey ramp, Playfair + Inter (`web/src/styles/tokens.css:8-36`) |
| **Local-first sovereignty** | The product is a local hive, not a cloud dashboard. Copy says "local runtime", "one local hive". | Splash kicker `LOCAL HMARS · SPATIAL EXECUTION` (`web/src/splash/HiveMindSplash.tsx:97`); Settings overview copy (`SettingsPanel.tsx:447`) |
| **Governed hierarchy** | HyperAgent → SuperAgent → Agent → Subagent is the product's spine; hierarchy is shown, not implied. | `AgentsPanel`, hierarchy/spawn tests (`web/tests/agents_spawn_matrix.test.ts`, `uiWiring.test.ts:254-298`) |
| **Truth before polish** | Refusal codes, degraded snapshots, missing keys and offline states are named. The UI refuses to fabricate success. | `SnapshotBanner`, approval queue, "Key missing", named refusal blocks (`SettingsPanel.tsx:347-356`) |
| **Quiet craft** | Slow ambient motion, hairline borders, one elevated surface family, generous paper. | `motion.css` header ("DECORATIVE, NOT DATA"); `index.css` animation scope |

**Brand promise:** *"A local hive that plans, delegates, remembers, and reports what actually happened."*
(Verbatim splash copy, `HiveMindSplash.tsx:106-109`.)

---

## 2. Voice and copy

### 2.1 Voice
- **Calm operator, not mascot.** Short declaratives. No exclamation marks, no "magic", no "AI-powered".
- **Name the refusal.** Error copy names the code or the missing thing (`E_KEY_MISSING`, "Runtime is offline").
- **Never claim unverified success.** "Saved" may be followed by "restart required"; "Connected" must not imply
  provider health (see §12, D1).
- **Second person only for the operator's action.** "Submit a prompt…", "Re-sync", "Return live".
- **Hive metaphor is seasoning, not a replacement for product nouns.** "Hive" is allowed for the runtime;
  "cells", "nectar", "queen" are not used for UI controls.

### 2.2 Copy patterns (verified in source)
| Pattern | Example | Source |
|---|---|---|
| State label | `CONNECTED`, `RECONNECTING`, `OFFLINE` | `Header.tsx:114-126`, `STRINGS.connection` |
| Honest empty | `NO ACTIVE AGENTS`; "No agents in this run."; "Nothing to replay yet" | screenshot 1440; `Header.tsx:155-157`; `App.tsx:150-157` |
| Refusal | `Key missing`; "Template catalog unavailable."; named refusal code + `how_to_fix` | screenshot; `LibraryPanel.tsx:185-188`; `SettingsPanel.tsx:349-353` |
| Operator instruction | "Drag a worker onto another super-agent to re-route its context stream live." | 1440 screenshot TIP card |
| Restart truth | "Saved. Restart the server/desktop app to apply the provider profile." | `SettingsPanel.tsx:193-196` |

### 2.3 Proposed copy rules
- Use sentence case for body and buttons; reserve uppercase for 11-12px mono meta/labels.
- Prefer "local runtime" over "server" in user-facing copy; use "server" only where the process is the subject.
- Every progress indicator must state whether it is **indeterminate** or **measured**.
- Avoid "operator" as a decorative label if the identity is not actionable; current avatar is static (§12).

---

## 3. Logo and wordmark

### 3.1 Verified lockup
- **Mark:** a 28×28 hexagon, outer fill `ink-900`, 1px `honey-500` stroke, inner hexagon `honey-400`.
  Inline SVG, `Header.tsx:52-63`.
- **Wordmark:** `HiveMind`, Inter semibold, 17px, `leading-none`, `ink-900`, one space, capital H and M only.
  `Header.tsx:64`.
- **Lockup:** mark + 8px gap + wordmark, left-aligned in a 52px top bar (`Header.tsx:50-65`).
- **Splash lockup:** large display wordmark with a hand-drawn honey "drip" underline and a 9-cell hex field;
  one-time boot surface only (`HiveMindSplash.tsx:96-105`, CSS `splash.css`).
- **Favicon / app icon:** not verified in this pass; `web/public` was not exhaustively audited.

### 3.2 Rules (P until enforced)
1. **Clearspace:** at least the height of the inner hexagon (≈12px at 28px mark) on all sides.
2. **Minimums:** full lockup ≥120px wide; mark alone ≥20px; never redraw the hexagon with different angles.
3. **Do not** put the wordmark on honey-300/400 without ink-900; the mark's outer fill is always ink-900.
4. **Do not** stretch, outline, add glow, or place the splash drip in persistent chrome.
5. **Compact variant:** below 480px use the 28px mark alone with an accessible name `HiveMind` — do **not**
   clip the wordmark to `H` (current 375 screenshot does exactly that; see D2).
6. **Logo is not a control** unless a home route is implemented; if it remains static, do not add hover styling
   that implies navigation.

### 3.3 Defect: 375px wordmark collapse
At 375×812 the header shows the hexagon plus a single `H`; the rest of the wordmark is clipped by
`truncate` on the `min-w-0` brand container (`Header.tsx:51,64`). There is no monogram rule in code.
**Required:** either hide the wordmark intentionally and keep the mark, or define an `HM` monogram lockup.
Never ship an accidental one-letter truncation.

---

## 4. Color tokens

All colors are declared twice and must stay in sync: `web/src/tokens/tokens.ts:22-44` (TS, 20 printed
swatches) and `web/src/styles/tokens.css:8-31` (CSS mirror). A test pins the count and provenance
(`web/tests/tokens.test.ts:13-28`). **No component may carry a hex literal** (`designParity.test.ts:50-52`).

### 4.1 Core palette (verified hex)
| Token | Hex | Role | Typical use |
|---|---|---|---|
| `canvas` | `#F8F4EB` | App background | Body, behind panels |
| `panel` | `#FFFFFF` | Primary surface | Top bar, rail, cards |
| `panel-alt` | `#FBF8F1` | Secondary surface | Library rail, hover |
| `recessed` | `#F1E9CF` | Recess / mock banner | Mock-mode strip, wells |
| `hairline` | `#E5E1D8` | 1px borders | Cards, dividers, inputs |
| `honey-100` | `#FBF3DC` | Selected tint / tip | Selected tab, TIP card |
| `honey-200` | `#F6E7BE` | Soft accent / avatar | Operator avatar, glow |
| `honey-300` | `#F0C050` | Primary action fill | Enabled primary buttons |
| `honey-400` | `#E8B850` | Hover fill / mark inner | Primary hover, logo inner |
| `honey-500` | `#D9A02F` | Strong border / running | Selected borders, running node |
| `honey-600` | `#C88030` | Deep honey border | Rare; verify contrast before text |
| `honey-700` | `#8A5A1C` | Honey text on light | Links/large labels (not small body) |
| `honey-800` | `#604818` | Deep honey fill | Rare |
| `ink-900` | `#282010` | Primary text / active fill | Body, active nav tile |
| `ink-800` | `#3F3115` | Strong secondary text | Button labels, values |
| `ink-700` | `#4E3A15` | Focus ring / strong meta | Focus outline, hover text |
| `ink-500` | `#75663F` | Secondary/meta text | Labels, captions, `text-meta` |
| `amber` | `#B7791F` | Warning accent | Large text only; borders |
| `signal` | `#B23A2E` | Error/danger | Error text, failed nodes |
| `verify` | `#3F7A55` | Success/connected | Connected pill, verified state |
| `border-gold` | `#E5C27A` | Chrome-only border | Decorative gold border |
| `error-surface` | `#FFF7F5` | Error surface | Rare error panels |

`border-gold` and `error-surface` are chrome-only, not printed swatches (`tokens.css:27-31`).

### 4.2 Semantic roles (verified aliases)
| Role | Token | Source |
|---|---|---|
| Text primary | `ink-900` | `tokens.css:59` |
| Text secondary / meta | `ink-500` | `tokens.css:60-61` |
| Focus ring | `ink-700` (not honey-300) | `tokens.css:62`, correction in `tokens.ts:80-90` |
| Error | `signal` | `tokens.ts` `SEMANTIC` |
| Success / connected | `verify` | `Header.tsx:117-121` |
| Warning | `amber` (large text / non-text only) | `tokens.ts:86-88` |
| Running node | `honey-500` | `tokens.ts` `SEMANTIC.nodeRunning` |
| Healthy node | `honey-300` | `tokens.ts` `SEMANTIC.nodeHealthy` |
| Failed node | `signal` | `tokens.ts` `SEMANTIC.nodeFailed` |
| Bee / routing edge | `ink-800` | `tokens.ts` `SEMANTIC.bee` |

### 4.3 Status color mapping (P: make this the only mapping)
| Status | Foreground | Border | Fill |
|---|---|---|---|
| Connected / verified | `verify` | `verify` | `panel` |
| Connecting / reconnecting | `ink-800` + `amber` border | `amber` | `panel` |
| Offline / failed | `signal` | `signal` | `panel` |
| Degraded (server reachable, dependency missing) | `amber` / `ink-800` | `amber` | `honey-100` |
| Approval pending | `ink-900` | `honey-500` | `honey-100` |

**Gap:** the top bar has no "degraded" treatment even when health is degraded (D1). With the verified
installation showing `health.status: "degraded"` and `missing_key: true`
(`install-1.2.0-dev.json:19-25`), the header still renders `CONNECTED` from the socket alone
(`Header.tsx:114-126`).

---

## 5. Typography

### 5.1 Families (verified)
| Role | Family stack | Source |
|---|---|---|
| Display / brand editorial | `"Playfair Display Variable", "Playfair Display", Georgia, serif` | `tokens.css:34`; bundled via `@fontsource/playfair-display` |
| UI / body | `"Inter Variable", Inter, system-ui, -apple-system, "Segoe UI", sans-serif` | `tokens.css:35`; bundled via `@fontsource-variable/inter` |
| Code / meta | `Consolas, "JetBrains Mono", "Cascadia Mono", monospace` | `tokens.css:36` (JetBrains Mono is a fallback; not bundled) |

### 5.2 Scale (verified printed steps)
| Step | Size / line | Tracking | Use |
|---|---|---|---|
| `display-xxl` | 96 / 100 | -0.03em | Splash/hero only |
| `display-l` | 56 / 60 | -0.02em | Rare editorial |
| `heading-h1` | 40 / 48 | -0.015em | Page title (large) |
| `heading-h3` | 24 / 32 | -0.005em | Section heading |
| `body-l` | 17 / 28 | 0 | Default body (`body`, `index.css:19-22`) |
| `label` | 12 / 16 | 0.08em | Uppercase tab/button labels |
| `mono-code` | 13 / 22 | 0 | Values, model ids, evidence |
| `mono-meta` | 11 / 16 | 0.08em | Status, table meta, counts |

Derived display sizes `26px` (history empty) and `40px` page titles exist in components; they are not
separate printed tokens. **Current UI also uses 9.5, 10.5, 11, 11.5, 12, 12.5 and 14.5px ad hoc**, e.g.
template rows `11px` (`LibraryPanel.tsx:205`), TIP `10.5px` (`LibraryPanel.tsx:227`), card meta `9.5px`
(`tokens.css:83`). These pass contrast at ink-500 but are below a defensible operational floor.

### 5.3 Proposed type rules
- **Operational text floor:** 12px for labels/body-adjacent, 11px only for mono meta with ≥4.5:1.
  Never 9.5px for a clickable row or a status the operator must read.
- **Mobile floor:** 12px meta, 16px body; increase line-height rather than shrinking type.
- **Uppercase** only for labels ≤12px and status pills; never uppercase a sentence.
- **Numbers/IDs** use mono, tabular where available; do not letter-space code values.
- **Playfair** is a brand/editorial voice, not a UI workhorse; do not set controls or dense tables in it.

---

## 6. Spacing

- **Base:** 4px; printed stops `4, 8, 12, 16, 20, 24, 32, 40, 48, 64, 80, 96`
  (`tokens.ts:114-115`; `tokens.test.ts` pins the array).
- Tailwind's default spacing matches the 4px base; components use 1.5/2/3/4/6 gaps.
- **Fixed layout geometry** (verified and test-pinned, `tokens.ts:166-183`, `designParity.test.ts:17-33`):
  topbar 52, nav rail 72, library 240, chat 400×480 r18, HUD 460×44, minimap 220×150,
  superagent card 420×150 r14, worker 320×96 r12, tool 420×240 r14, error 320×110 r12.
- **Content:** Settings scroll container `p-4` (16px) with inner `max-w-[920px]`
  (`SettingsPanel.tsx:401-404`).
- Proposed: use the 4/8/12/16/24/32 rhythm for new work; keep 52/72/240 as shell constants, not magic
  numbers in components.

---

## 7. Radius and shape

| Token | Value | Use |
|---|---|---|
| `xs` | 4px | Tags, tiny wells |
| `sm` | 8px | Inputs, small buttons, template rows |
| `md` | 12px | Cards (standard), worker nodes |
| `lg` | 16px | Large panels, empty-state cards |
| `xl` | 24px | Rare large surfaces |
| `button` | 10px | Button exception (R-U14) |
| `card` | 14px | Superagent/tool cards (R-U14) |
| `pill` | 999px | Status pills, chat toggle |

Rules (P): one radius per component class; never mix 8 and 10 on adjacent controls; selected tab and
primary button both use 10px. Cards use 12/14/16 by size, never arbitrary `rounded-[13px]`
(`designParity.test.ts:51` already forbids arbitrary radii).

---

## 8. Elevation

- **One warm-ink shadow family** derived from `ink-900` at 7–13% alpha:
  `e1 0 2px 8px`, `e2 0 6px 20px`, `e3 0 12px 34px` (`tokens.css:49-51`).
- Tier **semantics are explicitly unverified** (`tokens.ts:130-139`); do not claim "e1 = card, e2 = modal".
- Borders carry most hierarchy; shadows are reserved for floating layers (search popover `e2`,
  approval overlay) and hover lift.
- Proposed: flat surfaces use hairline only; one floating layer uses e2; never stack e2 on e2.

---

## 9. Icons

- **Verified:** inline SVG, `currentColor` or token colors, stroke widths 1.2–1.5, sizes 12/13/14/15/28.
  Nav icons are custom (`NavIcon.tsx`, referenced `NavRail.tsx:7,58`); the mark is bespoke.
- **Rules (P):** 16/20/24 grid; 1.5px stroke at 20–24px; 1.2px only below 16px; round caps/joins;
  no filled icons except the active brand mark; icon-only buttons require `aria-label`; decorative icons
  require `aria-hidden`.
- **Current:** every nav item has a visible text label under the icon, so icon-only ambiguity is low.
  At 375 those labels are 11px and the rail consumes 72px; see the responsive spec.

---

## 10. Motion

### 10.1 Verified behavior
- Ambient "live feel" loops: honeycomb glow 16s, hive sheen 12s, scan 16s, bee bob 3.4s; transform/opacity/
  background-position only; no layout thrash (`motion.css` header contract; `index.css` keyframes).
- Motion is **decorative, not data**: "Delete this file and every number on screen stays identical"
  (`motion.css:12-15`). This is a brand asset and must remain so.
- Reduced motion: both `@media (prefers-reduced-motion: reduce)` and `[data-reduced-motion="true"]` zero out
  animation/transition (`index.css:105-117`); the in-app setting persists in localStorage
  (`App.tsx:107-124`), and ambient intensity is exposed in Settings (`SettingsPanel.tsx:503`).
- Splash: 1.6s minimum, then leave; up to 4.4s if the socket never settles; 420ms exit
  (`HiveMindSplash.tsx:41-60`).
- Proposed durations: 120–180ms for control transitions, 240–320ms for overlay entry, 12–18s ambient only.

### 10.2 Defect: splash progress is a state bucket, not progress
`progress` is 18 / 46 / 100 based only on connection state (`HiveMindSplash.tsx:64-65`), while the DOM
exposes `role="progressbar"` with that value (`:117-126`). A progressbar implies measured progress; this is
not measured. **Fix:** either make it indeterminate (no `aria-valuenow`) or label it "connection status"
and remove the percentage. Also add a skip affordance for the 1.6s floor on every launch.

---

## 11. Light and dark rules

- **Verified:** light only. `:root { color-scheme: light }` (`tokens.css:54`); no `prefers-color-scheme: dark`
  block and no dark token set. The 1440 and 375 screenshots are light.
- **Proposed dark mode (not implemented, do not claim):**
  - Keep honey as accent, not background; canvas becomes deep warm ink (e.g. ink-900 family), panels one
    step lighter, hairlines at 20–30% alpha, text on dark must be ≥4.5:1.
  - Preserve the same semantic roles; do not invert status hues (verify/signal/amber) without re-measuring.
  - Never introduce pure black or pure white as a dark surface pair.
  - The splash is light-branded; if dark mode ships, it needs its own dark HoneyGlass material, not a
    CSS filter.
- Any dark-mode deliverable must add a contrast table and a screenshot at both 1440 and 375 before it is
  called verified.

---

## 12. Accessibility contrast pairs (computed from the token module)

Math: WCAG 2.1 relative luminance in `web/src/tokens/contrast.ts:15-58`; values below recomputed from the
declared hexes. AA normal text ≥4.5:1; AA large ≥3:1; non-text UI ≥3:1.

| Foreground | Background | Ratio | Grade | Allowed use |
|---|---|---|---|---|
| `ink-900` | `canvas` | **14.67:1** | AAA | All text |
| `ink-900` | `panel` | **16.10:1** | AAA | All text |
| `ink-500` | `panel` | **5.63:1** | AA | Meta/secondary text |
| `ink-500` | `panel-alt` | **5.31:1** | AA | Meta on library |
| `ink-500` | `canvas` | **5.13:1** | AA | Meta on canvas |
| `ink-700` | `honey-100` | **9.77:1** | AAA | Selected tab text |
| `signal` | `panel` | **5.94:1** | AA | Error text |
| `verify` | `panel` | **5.09:1** | AA | Connected text |
| `ink-900` | `honey-300` | **9.48:1** | AAA | Primary button |
| `ink-800` | `honey-200` | **10.29:1** | AAA | Avatar text |
| `ink-500` | `recessed` | **4.64:1** | AA | Meta on mock strip (thin margin) |
| `amber` | `panel` | **3.64:1** | AA-large only | ≥24px or ≥18.66px bold only |
| `amber` | `canvas` | **3.32:1** | AA-large only | Large text / borders only |

**Rules (V/P):**
- `amber` must never carry small text. The code already restricts it via `CORRECTIONS.amberLargeOnly`
  (`tokens.ts:86-88`) and `.text-amber-large` (`index.css:99-101`).
- Focus ring: 2px `ink-700`, 2px offset (`index.css:24-27`), ≥3:1 against panel/canvas (verified by
  `tokens.test.ts:38-46`). Do not restore the printed honey-300 ring (1.55–1.70:1).
- Disabled rows: opacity 60% on ink-800 over white still computes >4.5:1 in theory, but the 11px size and
  reduced affordance make the "catalog" explanation hard to read; see D5.
- **Unverified:** contrast of splash copy over the animated HoneyGlass gradients, and of placeholder text in
  inputs. The splash uses layered translucent surfaces; measure pixels from a real render, not tokens.

---

## 13. Audit defects (brand-relevant)

| # | Defect | Evidence | Severity |
|---|---|---|---|
| D1 | "CONNECTED" and "The hive is awake" reflect only the socket, not provider health. The verified install is `degraded` with `missing_key: true`, yet the screenshot shows CONNECTED and Key missing simultaneously. | `Header.tsx:114-126`; `HiveMindSplash.tsx:28-33`; install JSON `:19-25`; 1440 screenshot | High (status truth) |
| D2 | At 375 the brand lockup clips to a single "H"; no compact monogram rule exists. | 375 screenshot; `Header.tsx:51,64` | High (brand) |
| D3 | At <1024 the entire library rail (project, tabs, agents, 8-template gallery, TIP, state legend) disappears; no drawer or alternate entry point. The 375 screenshot shows no templates. | `LibraryPanel.tsx:84`; 375 screenshot; 1440 screenshot | High (responsive/feature loss) |
| D4 | 375 layout keeps the 72px rail, 17px/28 body, 140px definition-label column and 5 settings tabs that wrap to 3 rows; it is a compressed desktop, not a mobile design. | 375 screenshot; `NavRail.tsx:27`; `SettingsPanel.tsx:358,409` | Medium |
| D5 | Four of eight templates are disabled buttons whose "catalog-only" reason lives in `title`; disabled controls are not focusable, so keyboard/touch users cannot discover the reason. | `LibraryPanel.tsx:198-204,216-218`; 1440 screenshot (4 CATALOG chips) | Medium (a11y) |
| D6 | Splash shows a percentage with `role="progressbar"` but progress is a connection-state bucket (18/46/100), and there is no skip. | `HiveMindSplash.tsx:41-65,117-126` | Medium (truth/UX) |
| D7 | Nav rail doc comment says "six destinations"; it renders seven. | `NavRail.tsx:1` vs `:9-17`; 1440/375 screenshots | Low (doc drift) |
| D8 | Header search popover has no click-outside dismissal or focus trap; Escape works only while the input has focus. | `Header.tsx:136-176` | Medium (focus) |
| D9 | Count reads `TEMPLATES · 0` while loading, with no skeleton or "loading" count; empty-state copy says "No insertable templates" even when the catalog itself is empty. | `LibraryPanel.tsx:180-191`; `templateCatalog.ts:15-18` | Low (truth/polish) |
| D10 | Many operational labels are 9.5–11.5px; contrast passes but legibility on 375 is poor. | `tokens.css:83`; `LibraryPanel.tsx:205,227`; 375 screenshot | Medium |

All ten are UI findings, not security findings. No web product code was edited.

---

## 14. Verified vs proposed — index

**Verified (V):** palette values, type families and printed steps, radii, elevation family, layout constants,
light-only scheme, focus ring, contrast ratios computed from the token module, 1440/375 screenshot contents,
header/logo/library/settings/splash wiring cited above, existing tests that pin tokens and layout.

**Proposed (P):** compact brand lockup, dark mode, copy rules, status-color mapping, type floors, icon grid,
motion durations for controls, template explanation pattern, mobile navigation and library drawer, splash
skip/indeterminate progress.

**Not verified in this pass:** actual rendered contrast on translucent splash/HoneyGlass layers; favicon and
app icon; all 28 panels' visual states; live interaction beyond the two screenshots; screenshot device scale
and browser chrome. No screenshot was created or altered for this document.
