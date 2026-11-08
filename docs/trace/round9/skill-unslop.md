# Round 9 — `/deepseek-unslop` redline audit (AUDIT ONLY)

**Seat:** skill seat, `/deepseek-unslop` applied to the HiveMind UI source.
**Mode:** **polish** (not heavy-slop rebuild, not redesign).
**Status:** report only. **Zero product, test, or config files were edited.** The only file written by this seat is this one.
**Model:** `deepseek-flash` / DeepSeek-V4.1-Flash (version-pinned in `references/deepseek-v4-1-flash-1017.md`).

---

## 0. Constraints honoured

| Constraint from the brief | How it was honoured | Evidence |
|---|---|---|
| Audit only, no fixes | No `edit`/`write` ever targeted a source file | `git status --porcelain` shows only `docs/trace/round9/` (new) and pre-existing unrelated modifications |
| Write only `docs/trace/round9/skill-unslop.md` | The one audit artifact I generated inside the repo (`audit/round9-src.json`) was moved to `%TEMP%\round9-unslop-audit-src.json` | `Test-Path audit/round9-src.json` → `False` |
| Do not start/stop HiveMind | No process was started, stopped, or signalled | Only read-only `Get-NetTCPConnection` / `Get-CimInstance` |
| No browser automation | `snapshot.mjs` was **not** run | §11 |
| No provider/key use | No API call, no key read | — |
| No screenshot fakery | Every visual claim is marked `UNVERIFIED` and left unclaimed | §5, §11 |

**Side observation (not a UI defect, flagged because it will bite the next runner):** `127.0.0.1:8080` *is* listening, but it belongs to an unrelated Vite dev server — `node ...\posterpal\node_modules\.bin\..\vite\bin\vite.js dev --host 127.0.0.1 --port 8080`. `web/vite.config.ts:5-6` states the HiveMind production bundle is served by the Rust core on that same port. So HiveMind is stopped, **and** its production port is currently occupied by another project. Do not read "8080 is listening" as "HiveMind is up."

---

## 1. Working set

| Item | Path | Note |
|---|---|---|
| Target (brief) | `web/src`, `web/index.html`, `web/src/index.css` | **`web/src/index.css` does not exist.** `Test-Path` → `False`. The real file is `web/src/styles/index.css`, and it is a thin shell that imports `styles/tokens.css` + `styles/motion.css` (`index.css:1-3`). Audited as the brief intended. |
| Shell | `web/index.html` | 15 lines, clean |
| Token layer | `web/src/styles/tokens.css`, `web/src/tokens/tokens.ts` | PDF-derived, test-pinned |
| Style layer | `web/src/styles/index.css` (368), `web/src/styles/motion.css` (445) | |
| Surfaces | `App.tsx`, `panels/Header|NavRail|NavIcon|HudBadge|ControlBar|Scrubber|LibraryPanel|ApprovalOverlay|ToastRegion|HistoryPanel|ToolActivityPanel|SwarmPanel`, `flow/Canvas|nodes|HiveField|BeeEdge`, `chat/ChatCore|ChatMotion` | 21 files read in full |
| Copy | `state/strings.ts`, `state/agentState.ts`, `api/refusals.ts` | |
| Contracts | `tokens/tokens.ts`, `tokens/contrast.ts`, `tests/designParity.test.ts`, `tests/noHardcoded.test.ts` | Used to decide "safe fix" vs "pinned brand" |
| Audit JSON | `%TEMP%\round9-unslop-audit-src.json` (78 KB, full machine output) | Deliberately outside the repo |

---

## 2. Five-line direction (verbatim, derived from the product's domain)

- **Product:** A local operator console where one person watches and steers a hive of LLM agents working on a repository, in real time.
- **Character:** measured, honey-lit, exacting.
- **World:** a paper drafting table under a warm desk lamp, with live routing markers tracked across it.
- **Moments:** (1) the hive field behind the pinned chat core at canvas centre; (2) the approval dialog's evidence list — ids, hashes, and an expires countdown treated as the object of the screen.
- **Signature:** the bee as a *direction marker on a routed edge* — one glyph, one size, one stroke, never ornament.

The codebase already claims this world explicitly (`tokens.css:1-5`, `motion.css:1-34`, `tokens.ts:126` "the bee is a routing arrow, never decoration (R-U6)"). The direction above is **not invented for this audit** — it is the direction the source declares, which makes several findings below *contract violations* rather than matters of taste.

---

## 3. Scope mode and why

**Polish.** Reasons, in order of weight:

1. The foundations are real: 20 PDF-derived swatches (`tokens.ts:22-43`), three declared text roles (`tokens.css:59-62`), one focus-ring token, AA-checked contrast rows (`tokens.ts:62-69`), keyboard roving tabindex (`NavRail.tsx:27-35`), reduced-motion honoured on two paths (`index.css:101-119`, `motion.css:399-445`), honest unknown states (`formatCount(null) → '—'`).
2. The information architecture is complete and truthful: named refusals, offline banner, no fabricated metrics, no mock data in production paths. Nothing here should be rebuilt.
3. Every failure I found is *execution drift inside a good system* — 19 hand-typed font sizes that bypass an 8-step scale, 8 stroke widths that bypass a 1.5px rule, 19 infinite loops that bypass a cost contract the file itself states. Polish mode is exactly the mode for that.

I did **not** invent a brand, did not re-order sections, and did not delete a feature. Where something looks redundant (three "return to live" affordances, the minimap's empty label), I report it and say what preserving it costs.

---

## 4. Deterministic audit — commands, exit codes, counts

```powershell
node C:\Users\Riley\.dsh\skills\deepseek-unslop\assets\scripts\audit.mjs `
  C:\Users\Riley\Desktop\hiverouter-v1\web\src --json      # exit 1
node C:\Users\Riley\.dsh\skills\deepseek-unslop\assets\scripts\audit.mjs `
  C:\Users\Riley\Desktop\hiverouter-v1\web\index.html --json # exit 0
```

| Run | Files scanned | error | warning | exit |
|---|---|---|---|---|
| `web/src` | 57 | **61** | **3** | **1** |
| `web/index.html` | 1 | 0 | 0 | **0** |

Findings by rule (`web/src`):

| Rule | Severity | Count | Real or false positive |
|---|---|---|---|
| U001 exclamation in copy | error | 25 | **25 false positives** — 24 are Tailwind `!` important modifiers (`nodes.tsx:105,107,273,289,297`), 1 is `!==` in `ChatMotion.tsx:135`; the `mock.ts:161,182,222` hits are dev-mock fixtures, not shipped copy |
| U003 weight ≥700 | error | 1 | real — `Header.tsx:37` |
| U005 infinite animation | error | 17 | **real, and undercounted**: 2 more exist outside the regex (`BeeEdge.tsx:38` inline JS animation, `ChatCore.tsx:284` `animate-pulse`) → **19 loops** |
| U006 spacing off 4px | error | 18 | mixed — see §6.1 for the permitted/forbidden split |
| U010 uppercase + tracking | warning | 3 | real by rule, but **pinned brand typesetting** (`index.css:77,90,150`) |
| U002/U004/U007/U008/U009/U011 | — | 0 | genuinely clean: no emoji, no gradient text, no negative tracking on small text, **zero** banned hype vocabulary, no filler data |

**Audit blind spots I had to cover by hand** (state this when citing exit codes; the script is necessary, not sufficient):

1. **Tailwind `.5` spacing is invisible to U006.** `py-1.5` = 6px, `px-2.5` = 10px, `gap-1.5` = 6px. The rule only matches bracket syntax (`h-[6px]`). 32 occurrences of `.5`-step spacing exist (`grep` in §6.1).
2. **Arbitrary font sizes are invisible to every rule.** 19 distinct `text-[Npx]` values pass clean.
3. **Stroke widths and icon boxes are not rules at all** — 8 strokes, 11 icon sizes pass clean.
4. **Inline `style={{ animation }}` is invisible to U005** — `BeeEdge.tsx:38`.
5. **`rgba()` is invisible to `noHardcoded.test.ts`**, which only greps `#[0-9a-f]{6}` (`noHardcoded.test.ts:25`) — see B7 (`Canvas.tsx:215`).
6. **U006 false-positives the permitted 2px micro-step.** 6px dots are legal under the skill's own text ("A `2px` step is permitted only for micro-adjustments inside compact controls"). I classify, I do not just count.

**No RED→GREEN cycle was run.** The skill requires one "prove the audit is real" cycle; it requires planting a defect and fixing it, which this seat is forbidden to do (no edits, no second writer). Recorded as UNVERIFIED in §5 rather than faked.

---

## 5. Claim register

| # | Claim | Evidence | Status |
|---|---|---|---|
| C1 | `web/src` audit = 61 error / 3 warning, exit 1; `web/index.html` = 0/0, exit 0 | two command runs above | **VERIFIED** |
| C2 | `web/src/index.css` does not exist; the real stylesheet is `web/src/styles/index.css` | `Test-Path … → False`; `styles/index.css:1-3` | **VERIFIED** |
| C3 | 19 infinite animations ship in the workspace view (17 audit + 2 regex-invisible) | §6.9 list, each with `path:line` | **VERIFIED** (source) |
| C4 | 19 distinct explicit font sizes vs an 8-step printed scale | `grep 'text-\[[0-9.]+px\]' web/src` | **VERIFIED** (source) |
| C5 | 8 distinct SVG stroke widths (1, 1.1, 1.2, 1.4, 1.5, 1.75, 1.8, 2) and 11 icon box sizes | `grep strokeWidth=` (20 hits) + `width=` on each `<svg>` | **VERIFIED** (source) |
| C6 | `text-amber` at 11px = **3.64:1**, contradicting `tokens.ts:78 amberLargeOnly: true` | computed `#B7791F` on `#FFFFFF`; formula from `contrast.ts:41-47` | **VERIFIED** (computed) |
| C7 | Minimap kind marks: honey-300 **1.70:1**, honey-400 **1.84:1** on its own `panel` background — below WCAG 1.4.11 (3:1) | computed from `tokens.ts` hexes + `Canvas.tsx:718 backgroundColor` | **VERIFIED** (computed) |
| C8 | HUD delta renders during replay while the same badge says "LIVE METRICS HIDDEN" | `HudBadge.tsx:39-40` vs `:57-59`; replay never clears `metrics` (`store.ts:1745`, `:2205-2219` do not touch `metrics`) | **VERIFIED** (source) |
| C9 | SuperAgent state chip overflows its fixed 68px pill for `Awaiting approval` | `nodes.tsx:154-156`; ~5.0–5.4px/char at 8.5px mono + 0.08em ⇒ 85–92px for 17 chars | **VERIFIED** (arithmetic) |
| C10 | Composer has no visible focus ring | `ChatCore.tsx:340 outline-none`; global `:focus-visible` lives in `@layer base` (`index.css:27-30`) and loses to the utilities layer | **VERIFIED** (cascade) / pixel appearance **UNVERIFIED** |
| C11 | `className="bee-bob"` (`BeeEdge.tsx:51`) has **no matching CSS rule** — the routing marker never bobs | `grep '^\.bee-bob' web/src/styles` → 0 hits; only `@keyframes bee-bob` (`index.css:182`) and a reduced-motion reference (`:339`) | **VERIFIED** (absence) |
| C12 | Decorative bees violate the project's own R-U6 rule | `Canvas.tsx:550-558` (3 × `ambient-bee`, `aria-hidden`) vs `tokens.ts:126` | **VERIFIED** (source) |
| C13 | LibraryPanel shows **live** agents during replay (no replay guard anywhere in the file) | full read of `LibraryPanel.tsx` (198 lines); no `replayGraph` / `history` read | **VERIFIED** (source) |
| C14 | Two empty messages render simultaneously with an empty graph | `Canvas.tsx:696` (`hive-minimap-empty` when live `visibleTasks` empty) + `:787-793` (empty pill) + `index.css:143-152` (`content: 'no nodes yet'`) | **VERIFIED** (source) |
| C15 | Header cannot fit its own chrome below ~520px viewport width | `Header.tsx:23` fixed `h-[52px]`, no wrap; `:40` `shrink-0` cluster ≈ 380–420px of buttons + 100px brand + 24px padding | **INFERRED** (arithmetic; no render) |
| C16 | Chat core (fixed 480px, centred in `inset-0`) clips on short viewports | `ChatCore.tsx:192-195`; `App.tsx:124 overflow-hidden` | **INFERRED** (no render) |
| C17 | Approval modal has no focus trap, no initial focus, no Escape, no focus return; one `aria-modal` dialog per pending approval | `ApprovalOverlay.tsx:221` (`.map`), `:243-249`; no `useRef`/`keydown` handling in file | **VERIFIED** (source) |
| C18 | The UI would look like this product if the name were swapped | §13 | **INFERRED** (reasoned from signature inventory) |
| C19 | Visual hierarchy, spacing rhythm, contrast *as rendered* on 1440×900 | app stopped; no render permitted | **UNVERIFIED** |
| C20 | The audit is sensitive enough to catch a planted defect | no RED→GREEN cycle run (forbidden) | **UNVERIFIED** |

---

## 6. The nine passes

Every row: `path:line` → rule → observed evidence → minimal safe fix. "Safe" = one-line class/value change, no token edit, no test edit, no behavioural change. Rows whose fix would touch a test-pinned token are marked **PINNED** and are *not* polish work.

### 6.1 Pass 1 — Spacing

Rule set: 4px grid; 2px only as a micro-nudge inside a compact control; 1px hairlines; one height per control group; internal padding < inter-section spacing.

| # | `path:line` | Rule | Observed evidence | Minimal safe fix |
|---|---|---|---|---|
| S1 | `panels/ControlBar.tsx:44` and `:71` vs `:60,85,99,109` | one control height per group | selects are `h-9` (36px), buttons are `h-10` (40px), same row | `h-9` → `h-10` on both selects (or both to `h-9`); one value |
| S2 | `panels/ControlBar.tsx:41` | no structural 2px step | `py-1.5` = **6px** on a 52px bar | `py-2` (8px) |
| S3 | `chat/ChatCore.tsx:251` | 4px grid | `px-2.5 py-1.5` = 10px / 6px on the starter buttons | `px-3 py-2` (12/8) |
| S4 | `flow/nodes.tsx:144` | 4px grid | `pl-[30px] pr-4` — 30px is a 2px step used as structural padding, and asymmetric with no optical reason (there is no leading avatar in this node) | `pl-8` (32px) + `pr-8`, or `px-4` if the extra inset was accidental |
| S5 | `flow/nodes.tsx:154` | 4px grid | `h-[18px]` chip height (2px step, structural) | `h-4` (16px) or `h-5` (20px) |
| S6 | `panels/NavRail.tsx:47` | 4px grid | `gap-1.5` = 6px between icon box and label | `gap-2` (8px) |
| S7 | `panels/LibraryPanel.tsx:146` | 4px grid | `h-[30px]` template rows (2px step, structural) | `h-8` (32px) |
| S8 | `panels/LibraryPanel.tsx:141` | 4px grid | `space-y-1.5` = 6px between rows | `space-y-2` |
| S9 | `styles/index.css:147` | 4px grid (micro) | pseudo-element `bottom: 6px` | **permitted** (micro-nudge); left as-is, but see D4 |
| S10 | `panels/HudBadge.tsx:28`, `flow/nodes.tsx:233-235`, `chat/ChatMotion.tsx:141-143` | 4px grid (micro) | `h-[6px] w-[6px]` status dots | **permitted** (2px micro-step inside a compact badge). These **14** audit hits are rule-compliant |
| S11 | `panels/ControlBar.tsx:41` + `panels/Scrubber.tsx:42` | internal padding < inter-section | bar padding 6px, section gap is the border — fine | no change |
| S12 | `App.tsx:126-135` | stacking order | two `absolute inset-x-0 bottom-0 z-30` bar wrappers over content | no change (correct), but see B15 for what they overlap |

Counts: of U006's 18 hits, **3 are structural** (`h-[30px]`, `h-[18px]`, `pl-[30px]`, all 2px-step values used as control heights/padding); **15 are permitted** 6px micro-steps (14 dots + 1 pseudo-element nudge). The tailwind `.5`-step set adds **8 more off-grid families** (S2, S3, S6, S8 and their siblings) that the rule never sees. No stray fractional offsets anywhere else — the shell geometry is genuinely disciplined (`Header.tsx:23 h-[52px]` = `ControlBar.tsx:41 min-h-[52px]` = `LAYOUT.topbar` `tokens.ts:167`; rail `w-[72px]` = `LAYOUT.navRail`; library `width: LAYOUT.library` `LibraryPanel.tsx:46`). That parity is the reason this is polish, not a rebuild.

### 6.2 Pass 2 — Color

Rule set: one tinted neutral ramp; **exactly three** text roles; one accent; richness only in designated moments; AA contrast; borders as 6–10% hairlines; no hue gradients.

| # | `path:line` | Rule | Observed evidence | Minimal safe fix |
|---|---|---|---|---|
| C-o1 | `chat/ChatCore.tsx:199` `text-white`, `:216` `text-white/90 border-white/25 hover:bg-white/10` | three text roles only | a **fourth** text colour exists on the chat header; the token layer declares exactly `--text-primary` / `--text-secondary` / `--meta-text` (`tokens.css:59-62`), and the same file already uses the correct on-dark role at `:228` (`text-canvas`) | replace the four `white` utilities with `text-canvas` / `border-canvas/25` / `hover:bg-canvas/10` — computed 11.51:1 on `ink-800` and 7.85:1 on `honey-800`, both above the 7.36:1 the `white/90` composite currently reaches at the gradient's light end |
| C-o2 | `chat/ChatCore.tsx:200` | gradients limited to tone shifts within **one** ramp | `linear-gradient(135deg, var(--color-ink-800), var(--color-honey-800))` — crosses from the neutral ramp into the accent ramp, on the focal chat header, behind 4 pieces of text | flat `bg-ink-800`, or `bg-honey-800` if the header should read as accent. Either keeps AA (12.64:1 / 8.62:1 for white; 11.51 / 7.85 for `text-canvas`) |
| C-o3 | `flow/Canvas.tsx:215` | no colour outside the token module (G-UI-11) + no glow | `boxShadow: '0 0 0 2px var(--color-verify), 0 0 28px rgba(24, 156, 112, 0.45)'` — a raw `rgba`, a hue (`rgb(24,156,112)`, teal-green) that is **not** any of the 20 swatches (`--color-verify` is `#3f7a55`), and a 28px coloured glow. `noHardcoded.test.ts:25` only greps hex, so this passes CI | keep the 2px ring, delete the glow: `boxShadow: '0 0 0 2px var(--color-verify)'` |
| C-o4 | `flow/Canvas.tsx:698-713` | one accent per chart; auxiliary series on the neutral ramp | the minimap encodes node **kind** with 4 honey steps + `verify` + `signal` + `ink-500`; `--color-verify` is declared in the `state` group (`tokens.ts:42`) and is already the colour of *health* (`HudBadge.tsx:30`, `ToolActivityPanel.tsx:55`). One token, two meanings | single honey step for all kinds + `signal` only for `error`; kind is already carried by shape/label on the canvas |
| C-o5 | `flow/Canvas.tsx:698-713` + `:718` | non-text contrast ≥ 3:1 (WCAG 1.4.11) | marks sit on `backgroundColor: 'var(--color-panel)'`; computed **honey-300 = 1.70:1**, **honey-400 = 1.84:1**, honey-500 = 2.33:1, honey-600 = 3.19:1, verify = 5.09:1. The *default* and *superagent* branches are the failing two — the most common kinds | `honey-300`→`honey-700` (5.90:1) in the default branch, `honey-400`→`honey-600` (3.19:1) for superagent |
| C-o6 | `panels/ToolActivityPanel.tsx:58` | AA 4.5:1 for small text; respect the documented correction | `timeout: 'border-amber bg-panel text-amber'` renders 11px `mono-meta` text; measured **3.64:1**. `tokens.ts:76-78` records `amberLargeOnly: true` and `index.css:97-99` ships `.text-amber-large` for the legal case | `text-amber` → `text-ink-800` (12.64:1); keep `border-amber` (3.64:1 is legal for a UI boundary) |
| C-o7 | `flow/nodes.tsx:132` | richness confined to moments; charts encode value, not decoration | progress fill is `bg-gradient-to-r from-honey-500 to-honey-300` — a decorative ramp on the one element whose job is to encode a number | flat `bg-honey-500` (or `bg-honey-600`, 3.19:1 against its own track) |
| C-o8 | `chat/ChatMotion.tsx:141-143`, `flow/nodes.tsx:233-235` | state colours are for state | three decorative "traffic-light" dots in `bg-signal` / `bg-honey-400` / `bg-verify` — the exact three semantic state hues, used as window-chrome ornament | delete the dot row (the language label already carries the meaning), or use `hairline` for all three |
| C-o9 | `styles/tokens.css:59-62` | three text roles | compliant: exactly 3 roles + focus ring. No sprawling tints found | none |

Not infractions, stated so the next runner does not "fix" them: `signal` on the Kill button (`ControlBar.tsx:109`) and on refusal toasts is *destructive/error semantics*, not a second accent; the honey ramp is a legitimate 8-step accent ramp; the paper neutrals (`#f8f4eb` / `#fff` / `#fbf8f1` / `#f1e9cf` / `#e5e1d8`) are one coherent warm ramp.

### 6.3 Pass 3 — Icons

Rule: one line-icon set; uniform 16 or 18px box; uniform **1.5px** stroke; icons inherit the text role; 8–10px gap before labels; no non-functional icons.

| # | `path:line` | Rule | Observed evidence | Minimal safe fix |
|---|---|---|---|---|
| I1 | 20 `strokeWidth` sites: `Header.tsx:30,62,76,77`; `Canvas.tsx:38,535,572,743,756,769,770,782`; `ChatCore.tsx:263,264,373`; `ChatMotion.tsx:108`; `BeeEdge.tsx:54`; `LibraryPanel.tsx:152`; `nodes.tsx:65`; `HiveField.tsx:44` | uniform 1.5px stroke | **8 distinct strokes**: 1, 1.1, 1.2, 1.4, 1.5, 1.75, 1.8, 2 | normalise UI glyphs to `1.5`; keep `1` for the 46px empty-state illustration and `2` for the drag guide only (deliberate weights) |
| I2 | `Header.tsx:25` (28), `:57` (13), `:75` (15); `NavIcon.tsx:8` (22); `Canvas.tsx:36` (14), `:742,755` (12), `:768,781` (13); `ChatCore.tsx:362` (11), `:372` (15), `:262` (46); `BeeEdge.tsx:52` (18); `ChatMotion.tsx:105` (16 via `h-4`); `LibraryPanel.tsx:151` (12); `nodes.tsx:63` (10) | uniform 16 or 18px box | **11 distinct box sizes**: 10, 11, 12, 13, 14, 15, 16, 18, 22, 28, 46 | two boxes only: `16` (inline/utility) and `18` (rail, bee). The 28px logo and 46px empty-state illustration are brand/illustration, exempt |
| I3 | `Canvas.tsx:36` (14), `ChatMotion.tsx:105` (16), `BeeEdge.tsx:52` (18) | signature consistency | **the signature glyph is drawn at three sizes** — same bee, three boxes, two different stroke stories (`1.1` in `Canvas.tsx:38`/`BeeEdge.tsx:54`/`ChatMotion.tsx:108`) | fix the bee at 18 inside edges, 16 inside text; extract one `BeeMark({size})` used everywhere |
| I4 | `chat/ChatCore.tsx:216-220` | line icons, not glyphs | the close control is the typographic character `×` (U+00D7) at `text-[16px] leading-none`, in a mono font — inconsistent with every other control, baseline-dependent, and its box is ≈25×20px | replace with the same 1.5px-stroke 16px "x" line icon used elsewhere; give the button `h-6 w-6` minimum |
| I5 | `flow/nodes.tsx:256` | line icons | the error state's marker is `▲` (U+25B2) — a text glyph from the same species as emoji, sitting inside a line-icon world | replace with a 12px stroked triangle/alert path |
| I6 | `Canvas.tsx:550-558` | no non-functional icons | three `ambient-bee` glyphs, `aria-hidden`, clipped to a 660×620 decorative layer | delete (see B7) |
| I7 | `Header.tsx:57` (13) vs `Canvas.tsx:742` (12) vs `ChatCore.tsx:362` (11) | one set | three different "small control" sizes for the same visual weight class | fold into I2 |

No emoji anywhere: U002 = 0. Credit where due.

### 6.4 Pass 4 — Copy

Rule: zero exclamation marks; no greeting banners; functional labels; short strings end without periods; no hype; plausible data; **and the project's own invariant** — `strings.ts:2-5`: "every user-facing string resolves from here so no copy is hardcoded in components".

| # | `path:line` | Rule | Observed evidence | Minimal safe fix |
|---|---|---|---|---|
| T1 | `chat/ChatCore.tsx:230` `'Run refused'`; `:267` `'READY · NO RUN YET'`; `:290` `'Tool activity in chat'`; `:294` `'ACTIONS'`; `:356` `'Stop the active run'` | declared invariant `strings.ts:2-5` | 5 hardcoded strings in the focal surface, next to a `STRINGS` import at `:9` | move to `STRINGS.chat.*`; no rendered text changes |
| T2 | `flow/Canvas.tsx:506` `'HiveMind spatial canvas'`; `:582-587` (two inline sentences, incl. "The card moves with this connection…"); `:595` `'Cancel'`; `:604` `'Connect'`; `:622` `'Hide finished'`; `:628` (2 titles); `:637` `'Focus branch'`; `:648` `'Graph only'`/`'Show chat'`; `:733` `'Zoom controls'`; `:790` `'No agents match the current filters.'` | same invariant | 12 hardcoded strings in the canvas | move to `STRINGS.canvas.*` / reuse `STRINGS.controls.cancel` |
| T3 | `panels/Header.tsx:52,66,70,122` | same invariant + label drift | `aria-label` says *'Hide Hive chat'* while the visible label is *'Hide chat'*; `STRINGS.chat.close` = `'Hide chat'` already exists (`strings.ts:82-83`) for the same action | use `STRINGS.chat.close` / `.reopen` / a new `STRINGS.header.searchAria` |
| T4 | `flow/nodes.tsx:203` `'Subagent'`, `:212` `'SUBAGENT · bounded worker'` | same invariant + duplicate | the kind label exists as `NODE_KIND_LABELS.subagent` (`agentState.ts:132`) but the node hardcodes it twice | `NODE_KIND_LABELS.subagent`; drop the epigram or promote it to a string |
| T5 | `panels/LibraryPanel.tsx:162` `'TIP'`, `:167` `'State & settings'`, `:178-186` `provider/model/key/—/present/missing`, `:193` `'HiveMind v1.2-dev'`; `panels/HistoryPanel.tsx:21,23,27,28,48,50`; `panels/ApprovalOverlay.tsx:262` `'args_preview'` | same invariant | ~12 more; `args_preview` is the only `dt` in the dialog not sourced from `STRINGS.approval.fields` (`:254-290`) | move each to `STRINGS`; the version string should come from `package.json` (`1.2.0-dev`), not be retyped as `v1.2-dev` |
| T6 | `panels/HistoryPanel.tsx:28` | plausible data | `{revisions.length} revisions` — renders **"1 revisions"** | `revisions.length === 1 ? '1 revision' : \`${revisions.length} revisions\`` |
| T7 | `panels/ControlBar.tsx:50` | functional labels | `<option value="">{STRINGS.node.noMetrics}</option>` — the "no selection" choice in a **target** select is labelled with the no-metrics placeholder (`'—'`, `strings.ts:294`) | a real label: "No target" |
| T8 | `panels/Scrubber.tsx:72`, `panels/HistoryPanel.tsx:63`, `panels/Header.tsx:47` | one label per action | one store action (`exitHistoryReplay`) is labelled **`live`** (lowercase, `strings.ts:136`), **`live`** again, and **`Return live`** | one key, sentence case: "Return to live" |
| T9 | `panels/Header.tsx:122`, `flow/Canvas.tsx:790`, `panels/ModelsPanel.tsx:191`, `:219` | short strings end without a period | four short labels/status lines carry terminal periods | strip the period on labels; keep it on multi-sentence copy |
| T10 | `style` of copy | no marketing greeting | none found. No "Welcome back"; empty states are factual ("No run history yet", "No agents yet — submit a prompt to create the first run.") | none — this is a genuine strength |
| T11 | `panels/HudBadge.tsx:58` | metrics state explicit comparisons | the delta renders as a bare `+2.4%` with no baseline, while `STRINGS.hud.deltaLabel` = `'since previous metrics frame'` (`strings.ts:43`) is **defined and never used anywhere** | render `{formatDelta(hud.delta)} {STRINGS.hud.deltaLabel}` or drop the unused key |
| T12 | whole tree | zero exclamation marks | U001's 25 hits are all false positives (Tailwind `!` modifiers, `!==`, dev mock fixtures) | none — but see I-note: the 5 `!` important modifiers (`nodes.tsx:105,107,273,289,297`) are themselves a specificity smell |

### 6.5 Pass 5 — Type

Rule: sentence case; weights 400–500 with 600 rare; no 700–900; no gradient text; no negative tracking on small text; tabular figures for numbers; a small number of sizes with clear steps.

| # | `path:line` | Rule | Observed evidence | Minimal safe fix |
|---|---|---|---|---|
| Y1 | 19 sizes, e.g. `HudBadge.tsx:46` 7.5 → `SwarmPanel.tsx:71` 28 | small number of sizes with clear steps | `TYPE_SCALE` declares **8** printed steps (`tokens.ts:91-100`); components ship **19**: 7.5, 8, 8.5, 9, 9.5, 10, 10.5, 11, 11.5, 12, 12.5, 13, 13.5, 14, 14.5, 16, 17, 26, 28. Ten of them are off-scale | map each off-scale size to its nearest declared step (11 / 12 / 13 / 17), one pass, no layout change beyond ±1px line box |
| Y2 | `panels/HudBadge.tsx:46` `text-[7.5px]`; `panels/NavRail.tsx:61` `text-[8px]`; `LibraryPanel.tsx:123,131`; `ChatCore.tsx:267,293`; `HistoryPanel.tsx:49`; `ModelsPanel.tsx:245` | legibility floor (craft, not WCAG) | 7.5px and 8px uppercase tracked text. At 1× DPR the x-height is ≈4px | raise the meta floor to 11px (`--text-mono-meta`) and let the label truncate; the 44px HUD has room (LAYOUT.hud.height) |
| Y3 | `panels/Header.tsx:37` | no weight ≥700 | `text-[17px] font-bold` — the wordmark is the **only** 700 in the tree (U003 = 1) | `font-semibold` (600) or `font-medium`; the hexagon mark carries the brand weight |
| Y4 | `styles/index.css:77`, `:90` | sentence case everywhere; eliminate uppercase tracked labels | `.type-label` and `.type-mono-meta` both hard-set `text-transform: uppercase` **plus** positive tracking, and they are the two most-used type utilities (nav-adjacent tabs `LibraryPanel.tsx:87`, outcome labels `ToolActivityPanel.tsx:155`, dialog buttons `ApprovalOverlay.tsx:358,369`, every meta line) | drop `text-transform` from both utilities, keep the tracking token; the strings are already sentence case in `STRINGS` (only 6 are literally uppercase in source). **PINNED**: `type-label` is PDF typesetting (PDF:122) — changing it is a design decision, not a polish fix |
| Y5 | `tokens.css:67,70,73` | no negative tracking on **small** text | `-0.03em` @96px, `-0.02em` @56px, `-0.015em` @40px, `-0.005em` @24px — all display sizes | **not an infraction**. U007 = 0. Stated so nobody "fixes" the display scale |
| Y6 | whole tree | tabular figures for numbers | **zero** `tabular-nums` in the codebase. `SwarmPanel.tsx:48` renders live counts at 26px in `font-display` (Playfair Display, proportional serif), so `12 → 8` shifts every digit | add `tabular-nums` to `font-display` numerals or use the mono face for stat values; one-line change per site |
| Y7 | `flow/nodes.tsx:159`, `SwarmPanel.tsx:47`, `ChatCore.tsx:267`, `App.tsx:64` | one tracking value per role | three tracking values where the token declares one: `0.08em` (token `tokens.css:88`), `0.12em`, `0.14em` | use `--tracking-mono-meta` (0.08em) everywhere |
| Y8 | `flow/nodes.tsx:147,155,159,160,226,287` | hierarchy: few sizes | **six** different sizes inside one 420×150 card (14.5, 8.5, 9.5, 11.5, 13.5, 14.5) | collapse to 12 / 11 / 14.5 |

Font families are coherent and preserved (Playfair for display, Inter for UI, Consolas for mono — `tokens.css:35-37`). No gradient text: U004 = 0.

### 6.6 Pass 6 — Layout

| # | `path:line` | Rule | Observed evidence | Minimal safe fix |
|---|---|---|---|---|
| L1 | `App.tsx:104-141` | app shell: fixed rail + fluid content | 52px header + 72px rail + 240px library + fluid `main`. No feature was moved: nav order Workspace→Agents→Data→Models→Routing→Settings (`NavRail.tsx:9-16`), tabs Workspace→Swarm→Routing→History (`LibraryPanel.tsx:15-20`) | none — shell is correct and pinned by `designParity.test.ts:16-33` |
| L2 | `App.tsx:125-135` | one primary action per view | two overlay stacks (`ControlBar`+`Scrubber` in workspace; `Scrubber` alone in history) — correct scoping | none |
| L3 | `panels/LibraryPanel.tsx:45` | responsive: no feature loss | `hidden … lg:flex` — below 1024px the library (project chip, workspace tabs, **active agents**, **templates**) vanishes, and the rail has no destination that replaces Templates | add a rail entry or a `lg:hidden` sheet trigger. This is a **feature loss**, not a refinement (B17) |
| L4 | `Header.tsx:23` + `:40` | responsive stacking in reading order | `flex h-[52px] … gap-3` with a `shrink-0` action cluster (~380–420px of buttons) and no wrap, no overflow handling; plus the brand block (`min-w-0 truncate`) is the only shrinkable child | allow the status badge to collapse to its dot below `sm`, or let the cluster scroll. **INFERRED** (needs a 375px render to confirm) |
| L5 | `ChatCore.tsx:192-195` | responsive | `h-[480px]` fixed, centred in `inset-0`, `-translate-y-8`, with the parent at `App.tsx:124 overflow-hidden`; no `max-h` | `h-[min(480px,calc(100%-96px))]`. **INFERRED** (no render) |
| L6 | `styles/index.css:137-142` | documented breakpoint behaviour | minimap hidden below 1280px, with the reason in a comment | none (correct, documented) |
| L7 | `App.tsx:106-108` | skip navigation | `.skip-link` → `#hive-canvas` present (`index.css:355-367`) | none — good |
| L8 | `styles/index.css:166-167` comment vs reality | documented contract | comment claims "no layout thrash"; contradicted by S6/D-motion findings | fix the code, not the comment |

Section order and features: **preserved**. No deletion proposed anywhere in this report.

### 6.7 Pass 7 — Details

| # | `path:line` | Rule | Observed evidence | Minimal safe fix |
|---|---|---|---|---|
| D1 | `ChatCore.tsx:195` + `:333` | inner radius = outer − padding (min 4) | outer 18, padding 12 (`p-3`) ⇒ inner should be **6**; the composer form is `rounded-[12px]` — rounder than its own container's inner curve | `rounded-[6px]` on the form, or padding `p-2` ⇒ inner ≤ 10 (use `--radius-button`). Pick one; do not leave 12 |
| D2 | `ApprovalOverlay.tsx:249` + `:297,:313,:322,333,347,358,369` | inner radius = outer − padding | outer `--radius-card` = **14**, padding `p-4` = **16** ⇒ inner should clamp to the 4px minimum; children are 8 and 10 | children → `rounded-[4px]` (`--radius-xs`), or the card → `p-3` (12). As shipped the dialog reads "squarer frame, rounder contents" |
| D3 | `flow/nodes.tsx:123,129` + `Frame` radius 12/14 (`:94`) | no pill inside a small card | the progress track and fill are `rounded-[var(--radius-pill)]` (999) inside a 12px card with 12px padding ⇒ inner should be 4 | `rounded-[4px]` on track and fill |
| D4 | 29 `rounded-[…pill]/full` sites; 4 in one cluster: `Canvas.tsx:615,630,639,645` (+`:727`,`:789`), `Header.tsx:44,54,82`, `HudBadge.tsx:17,46`, `nodes.tsx:154,227,281`, `ChatCore.tsx:118` | ≤ 3 pills per screen | **7 pills in the top third** of the workspace view (Header 3, HUD 2, Canvas filter bar 4 — overlapping region), plus 3 per node card | convert the 4 canvas filter pills to `rounded-[var(--radius-sm)]`; keep pills only for the HUD, the state chip, and the replay banner |
| D5 | `styles/index.css:143-152` | details rendered by the UI, not by CSS strings | `.hive-minimap-empty::after { content: 'no nodes yet' }` — copy injected from CSS: not in `STRINGS`, not testable, and it renders **at the same time** as the canvas empty pill (`Canvas.tsx:787-793`), producing two empty messages | delete the pseudo-element; the canvas already owns the empty state (B8) |
| D6 | `Canvas.tsx:274-278`, `LibraryPanel.tsx:114-119` | details must be achievable | both "hexagon" chips are a **bordered rect + `clipPath`**. `clip-path` clips the border too, and a border is only painted on the four rect edges — so the six slanted edges have **no stroke at all**. The result is a hexagonal *fill* with four disconnected straight strokes | replace with an inline 6-point `<polygon>` (or a `::before` outline). Two sites, same fix |
| D7 | `ChatCore.tsx:340` + `styles/index.css:27-30` | visible focus states | `.type-*`/utilities layer wins over `@layer base`, so `outline-none` on the composer leaves the app's **primary input** with no focus indicator (`:focus-visible` never paints) | add `focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[var(--focus-ring)]` to the form, or `has-[:focus-visible]:…` on the form so the whole field lights |
| D8 | `Scrubber.tsx:56` | interactive states / target size | `className="h-1 …"` = **4px** tall range input (WCAG 2.5.8 asks 24×24; the UA thumb overflows the box, so effective height is ~16px) | `h-6` with `appearance-none` thumb styling, or wrap in a 24px-high label |
| D9 | `SwarmPanel.tsx:46-48`, `HudBadge.tsx:58` | stat cards carry comparative baselines | stat cards are label + number only; the HUD's delta has no baseline text | see T11; add `since previous frame` |
| D10 | `flow/Canvas.tsx:698-713` | one chart dominates | the canvas dominates correctly; the minimap is the secondary series and uses 6 hues (see C-o4) | see C-o4 |
| D11 | focus inventory | visible focus everywhere else | `:focus-visible` global ring + `.react-flow__node:focus-visible` + `Header.tsx:118 focus-visible:ring-2` + `.motion-focus-glow` (`motion.css:310-316`, unused) | delete or adopt `.motion-focus-glow`; it is currently dead CSS |

### 6.8 Pass 8 — Visuals

| # | `path:line` | Rule | Observed evidence | Minimal safe fix |
|---|---|---|---|---|
| V1 | `Canvas.tsx:520` + `index.css:230-250` | remove decorative motion layers | `<div className="hive-scan">` — a full-canvas 105° honey band sweeping every 16s over the product's main work surface, `z-index: 0` | delete the element and the `.hive-scan`/`@keyframes hive-scan` pair |
| V2 | `HiveField.tsx:110` + `motion.css:90-107` | remove decorative motion layers | `.motion-hive-sheen` — 12s `linear` sweep with `mix-blend-mode: soft-light` across the hive | keep the hive, delete the sheen layer |
| V3 | `HiveField.tsx:36` + `motion.css:53-69` | remove decorative motion layers | `.motion-honeycomb-glow` — 16s `alternate` breathe | keep as a **static** radial pool (delete `animation`, keep the gradient) |
| V4 | `Canvas.tsx:549-559` + `index.css:269-299` | no decorative objects that imply nothing | three `ambient-bee` glyphs on 13/16/18s flights — see B7 (violates R-U6) | delete the layer |
| V5 | `styles/index.css:216-228` + `:174-177` | dead code is slop | `.hive-ambient-glow` + `@keyframes hive-breathe`: **no JSX references them** (`grep 'hive-ambient-glow'` → only CSS) | delete both (safe: unreferenced) |
| V6 | `styles/index.css:252-267` + `:174-177` | dead code | `.hive-sheen` + `@keyframes hive-sheen`: unreferenced | delete |
| V7 | `styles/index.css:315-318` + `:192-194` | dead code | `.bee-work-icon` + `@keyframes bee-work`: unreferenced (`ChatMotion.tsx:105` uses `.motion-work-bee`) | delete |
| V8 | `styles/index.css:320-324` | dead code / fragile override | `.motion-work-bee { animation-duration: 1.2s }` re-declares a class that `motion.css:330-339` already owns at 2.2s; the winner depends on `@import` order (`index.css:1-3`) | move the duration into `motion.css` as a modifier class |
| V9 | `motion.css:310-316` | dead/unadopted | `.motion-focus-glow` is never used in JSX | adopt or delete |
| V10 | `chat/ChatMotion.tsx:262-266` | imagery only for empty states/onboarding | a 46px hexagon illustration in the chat empty state — **legitimate** under the rule; but it is a third hexagon dialect (border-radius 0, stroke 1.2/1) | keep; align stroke to 1.5 (I1) |
| V11 | `HiveField.tsx:39-108` | product previews with real material | 5-lobe hive built from real radial gradients + specular + rim, no filters, no raster — this is the screen's **one legitimate bright element** | keep. Do not add more |

Real material is *good* here. The filler is: 3 ambient bees, 2 sweeping sheens, 1 scan band, 3 dead CSS animations, and a decorative gradient on a data bar. The hive itself is the product's world and should stay.

### 6.9 Pass 9 — Motion

Rule: remove infinite loops on static elements; keep functional micro-interactions; animate only transform/opacity/colour; one easing curve and duration profile.

| # | `path:line` | Rule | Observed evidence | Minimal safe fix |
|---|---|---|---|---|
| M1 | `index.css:212` (typing-dot), `:226` (breathe), `:243` (scan), `:265` (sheen), `:274` (ambient-flight), `:311` (state-pulse), `:316` (bee-work) — 7 | no infinite decorative loops | 7 in `index.css`; 4 of them (`breathe`, `scan`, `sheen`, `bee-work`) are on **static** elements | keep `typing-dot` (functional: "reply in flight") and `state-pulse` (functional: running state); delete the other 5 |
| M2 | `motion.css:67` (16s alternate), `:105` (12s sweep), `:128` (1.4s dash), `:145` (3.4s bob), `:168` (1.8s pulse), `:199` (2.4s ring), `:298` (1.9s progress sweep), `:337` (2.2s bee spin), `:363` (2.8s dot pulse) — 9 (+`index.css` 7 = **16**), plus `BeeEdge.tsx:38` (inline `bee-dash 2.4s infinite`) and `ChatCore.tsx:284` (`animate-pulse`) = **19** | no infinite decorative loops | see the count; U005 catches 17, misses 2 (regex cannot see inline JS animation or a Tailwind class) | delete: honeycomb breathe, hive sheen sweep, progress sheen, connection-live pulse (a status dot does not need to breathe at 2.8s forever). Keep: edge dash (direction), bee spin (work in flight), state pulse (state), typing dots (in flight), running ring (in flight) |
| M3 | `styles/index.css:364` | no layout-property transitions | `.skip-link { transition: top 120ms ease }` — `top` is a banned layout property | `transform: translateY(-48px)` + `transition: transform 120ms` |
| M4 | `index.css:302`; `motion.css:375-376`; `ChatCore.tsx:196` | the file's own cost contract: "no animated box-shadow" (`motion.css:20`) | **3 violations**: `.hive-node { transition: box-shadow 160ms, border-color 160ms }`; `.motion-button-press { transition: transform 110ms, box-shadow 110ms }`; inline `transition: 'box-shadow 160ms ease'` on the chat section | drop `box-shadow` from all three (keep `transform`/`border-color`); the shadow snap is imperceptible at 160ms |
| M5 | `index.css:213,227,244,266,275`; `motion.css:68,106,338` | `will-change` discipline | `will-change` declared **permanently on 8 animated elements** — including `will-change: background-position` (`:244,:266`, `motion.css:68,106`), a property browsers do not composite, so the hint buys nothing while the element keeps a promoted layer and a stacking context. `motion.css:21-22` claims "will-change is declared on exactly three … classes" | delete the four `background-position` hints; keep `transform`/`opacity` hints only on `.motion-work-bee`, and only while running |
| M6 | 6 distinct eases + 20+ durations | one easing curve, one duration profile | `ease`, `ease-out`, `ease-in-out`, `linear`, `cubic-bezier(0.22, 0.61, 0.36, 1)` (`motion.css:199`), `cubic-bezier(0.2, 0.8, 0.2, 1)` (`:224,239,258`), plus Tailwind's `cubic-bezier(0.4, 0, 0.6, 1)` from `animate-pulse`. Durations: 80, 110, 120, 160, 180, 200, 240, 250, 1.1s, 1.2s, 1.4s, 1.6s, 1.8s, 1.9s, 2.2s, 2.4s, 2.8s, 3.4s, 8s, 12s, 13s, 16s, 18s | two curves: `cubic-bezier(0.2, 0.8, 0.2, 1)` for entrances, `linear` for loops; three durations: 120 (press), 200 (enter), 1.8s/16s (state vs ambient) |
| M7 | `motion.css` vs `index.css` | one motion vocabulary | **two parallel vocabularies for the same five ideas**: `state-pulse` 1.6s vs `motion-state-pulse-soft` 1.8s; `bee-bob` (dead, B10) vs `motion-bee-bob-soft` 3.4s; `hive-sheen` 8s (dead) vs `motion-hive-sheen-sweep` 12s; `bee-work` 1.2s (dead) vs `motion-work-bee-spin` 2.2s (overridden back to 1.2s at `index.css:320`); edge dash `-40px/2.4s` (`BeeEdge.tsx:38` + `index.css:179-181`) vs `-28px/1.4s` (`motion.css:118-129`) | collapse to `motion.css`; delete the `hive-*`/`bee-*` vocabulary from `index.css` |
| M8 | `index.css:101-108` vs `motion.css:399-445` | reduced motion done properly | `motion.css` uses `animation: none !important` + `will-change: auto`; `index.css`'s `[data-reduced-motion="true"]` path only sets `animation-duration: 0.001ms` + `iteration-count: 1`, so loops freeze but keep `will-change` and 1 iteration | add the `hive-*`/`bee-*` classes to the `animation: none` list, and `will-change: auto` with them |
| M9 | all keyframes | animate transform/opacity/colour only | **compliant** — no keyframe animates width/height/top/left/margin (verified across both files). This is the strongest part of the motion layer | none |

---

## 7. Surface redlines

### 7.1 Hierarchy (what owns the screen)

The screen should have one bright element. Currently the honey accent is spent on: the chat header gradient (`ChatCore.tsx:200`), the send button (`:368`), the composer's active ring (n/a), the spawn button (`ControlBar.tsx:60`), the approval approve button (`ApprovalOverlay.tsx:369`), the HUD pill + dot (`HudBadge.tsx:17,28`), 7 canvas pills, the superagent node header + state chip (`nodes.tsx:144,154`), the tool node's recessed pill (`:227`), the hyperagent pill (`:281`), 3 progress fills, the minimap (6 hues), 3 ambient bees, and the 5-lobe hive itself. **Nothing dominates**, because everything does.

The one element that should own the screen — the hive field, which is the product's world — is the calmest thing on it. Minimal fix: spend the accent on the hive field + the single primary action for the current state, and de-accent the chrome (the 7 pills, the header gradient, the chat send button can all go neutral).

### 7.2 Canvas

- `Canvas.tsx:505-508` `role="application"` on the canvas wrapper turns off browse mode for the whole subtree. With `nodesFocusable` (`:659`) this is defensible, but there is no documented keyboard model for panning; `role="application"` without a complete keyboard story is a screen-reader trap. **INFERRED** (needs AT test) — record, do not change blindly.
- `Canvas.tsx:215` glow (C-o3), `:520` scan band (V1), `:550-558` bees (V4), `:698-713` minimap hues (C-o4/C-o5).
- `Canvas.tsx:609-649`: four pill-shaped filter/utility buttons + a raw `{visible}/{total}` counter with **no accessible label** (`:639-641`).
- `Canvas.tsx:787-793` duplicate empty state with `index.css:143-152` (C14).
- `Canvas.tsx:795-798`: `aria-live` region concatenates `"Alpha: running"` and `"Reconnecting…"` with **no separator** — screen readers announce `runningReconnecting…` (B11).

### 7.3 Chat core

- Fixed 480px height (L5/L6), gradient header (C-o2), `text-white` 4th role (C-o1), `×` glyph (I4), no focus ring on the composer (D7), `animate-pulse` caret (M2), decorative hexagon (V10), hardcoded strings (T1), `aria-live` on the whole transcript instead of `role="log"` (B11), and the **tool-activity sub-panel** inside chat duplicates `ToolActivityPanel` semantics with a different vocabulary (`ChatCore.tsx:288-321` renders raw `entry.outcome` strings, not `STRINGS.toolActivity.outcomes`, so it shows `pending`/`ok` instead of `Pending`/`OK` — a real inconsistency).
- One bright spot: the streaming/ACK/refusal settlement (`ChatCore.tsx:133-184`) is honest, retracts exactly the optimistic row, and never clears the draft before ACK. No finding.

### 7.4 Nav rail

`NavRail.tsx:26` 72px, 6 destinations, roving arrow-key focus (`:27-35`) — correct. Findings: `rounded-[11px]` (off-scale radius — `tokens.ts:107` stops are 4/8/12/16/24/48/999 + exceptions 10/14 → **11 is not a stop**), `text-[8px]` label (Y2), `gap-1.5` (S6), icons at 22px/1.6 stroke (I1/I2), and the **active state is `bg-ink-900`** (`:53`) — a large solid ink block in a 72px rail is the heaviest visual weight in the left column, heavier than the selected node card. Consider `bg-ink-800` (12.64:1 text still) or an ink-900 *border* + honey icon.

### 7.5 HUD

`HudBadge.tsx` is the most "product-specific" chrome on screen: throughput, source (server/derived/none), delta. Findings: 8.5px type (Y2), 460×44 hard width (`:19-20` via `LAYOUT.hud`) with `whitespace-nowrap` — the replay string (`:40`) plus the `DERIVED` chip (`:46`) plus three figures cannot fit 460px at 8.5px (≈78 chars × 5.2px = 405px + 4 gaps + 32px padding ≈ 470px), so the HUD is at its overflow ceiling in the derived+delta case. **INFERRED** (arithmetic). Plus C8 (delta during replay) and T11 (unlabelled delta).

### 7.6 Minimap

`index.css:132-157` places it with `margin: 0 40px 120px 0`, a radial-gradient dot grid (`background-size: 8px 8px`), a transparent mask with a 1.4px ink stroke, and a CSS-injected empty label. `Canvas.tsx:697` correctly sets `ariaLabel`, and `:414-428` force `role="img"` on the library element — the query runs once on mount plus one rAF, so if React Flow mounts the minimap later the label is lost (**INFERRED**, ordering-dependent). Findings: C-o4/C-o5 (hues + contrast), C14/D5 (empty label), `:696` uses **live** `visibleTasks` to decide the empty state while the canvas is replaying (B16).

### 7.7 Approval

The best-designed surface in the tree: evidence list of tool / args_sha256 / args_preview / run / task / tool_call / approval_id / expires, honest `—` for unknown, nonce-missing explanation, fail-closed gating, `prunePendingResolutions` semantics documented. Findings: no focus trap / initial focus / Escape / focus return (C17 — the lead already lists this at `lead-plan.md:42`), one `aria-modal` dialog per pending approval (`:221`), `args_preview` hardcoded (`:262`), radius inversion (D2), `py-1.5` buttons (S2-family), and `aria-live` on a countdown (`:291`) that will announce every second — a live region on a 1 Hz timer is noise; it should be `aria-live="off"` with the expiry surfaced on demand.

### 7.8 Control bar

`ControlBar.tsx` has the right idea (every mutating control disabled unless its precondition is visibly true, `:33-38`). Findings: mixed control heights (S1), `py-1.5` (S2), the target select's `aria-label` says **`WORKER: Re-route`** (`:45`) but the list contains *every* task including superagents and hyperagents (`:51`) — an a11y/label bug (B13), the empty option is the no-metrics placeholder (T7), and `disabled:opacity-50` is the only disabled treatment on the filled honey primary (`:60`) → 2.46:1 effective text contrast. Disabled controls are exempt from WCAG 1.4.3, so this is **craft, not a violation** — but the primary action becoming unreadable is a bad look; use `disabled:bg-panel-alt disabled:text-ink-500 disabled:border-hairline`.

### 7.9 Replay states

Coherence matrix at `history.replayGraph !== null`:

| Surface | Replay-scoped? | Evidence | Verdict |
|---|---|---|---|
| Canvas graph | yes | `Canvas.tsx:652-653, 689-690` renders `replayNodes`/`replayEdges` | correct |
| Canvas node selection | yes | `:663-667` no-op on click in replay; `noopSelect` `:435` | correct |
| Canvas replay banner | yes | `:725-731` "Replay (read-only) · Routing state N" | correct (copy see B18) |
| Canvas filter **count** | **no** | `:640` `{visibleTasks}/{tasks}` — live tasks | **bug** (B16) |
| Canvas minimap **empty label** | **no** | `:696` live `visibleTasks` | **bug** (B16) |
| HUD superagents/workers/green | yes | `HudBadge.tsx:39-42` | correct |
| HUD **delta** | **no** | `:57-59` renders `hud.delta` next to "LIVE METRICS HIDDEN" | **bug** (B1) |
| Library panel (agents in run, chips) | **no** | `LibraryPanel.tsx:40` live `tasks`; no replay reference in the file | **bug** (B2) |
| Composer | yes | `ChatCore.tsx:351` disabled when replaying | correct |
| Control bar + selects | yes | `ControlBar.tsx:33-38` all four `can*` include `!replaying` | correct |
| Scrubber | yes | `Scrubber.tsx:31-33,64` position + replay prefix | correct |
| Exiting replay | yes, ×3 | `Header.tsx:41-49`, `Scrubber.tsx:66-73`, `HistoryPanel.tsx:57-64` | redundant label inconsistency (T8) |
| Approvals during replay | **no** | `ApprovalOverlay.tsx:221` renders live approvals as a blocking `aria-modal` over the replayed graph | **bug** (B5) |

---

## 8. Bugs (not taste)

Each of these is a contract, arithmetic, or accessibility violation — something that is *wrong*, not merely unrefined.

| ID | Severity | Bug | `path:line` | Evidence | Smallest correct fix |
|---|---|---|---|---|---|
| B1 | **High** | HUD says "REPLAY · LIVE METRICS HIDDEN" while rendering a live delta | `HudBadge.tsx:39-40` vs `:57-59` | replay branch returns after the hidden text, but the delta `<span>` is rendered unconditionally after it; `metrics` is never cleared on replay (`store.ts:1745`, `:2205-2219`) | move the delta render inside the non-replay branch, or gate on `!replaying` |
| B2 | **High** | In replay, the left rail shows **live** agents (with live state dots) while the canvas shows the historical graph | `LibraryPanel.tsx:40`, `:105-128` | the file never reads `history`/`replayGraph`; `tasks` is live state | render the replay roster (same source `Canvas.tsx:431` uses) or hide the section and label the rail "live" |
| B3 | **High** | SuperAgent state chip overflows its fixed 68px pill | `nodes.tsx:154-156` | `w-[68px]` + `shrink-0` + 8.5px mono + 0.08em ⇒ "Awaiting approval" needs ≈85–92px; no `truncate`/`overflow-hidden` on the label span | `w-auto min-w-[68px] max-w-[104px] truncate px-2` on the pill |
| B4 | **High** | A `null` progress renders an empty track that reads as **0%**, directly above the text "—" | `nodes.tsx:122-137` (`value === null` → no fill), used at `:164,188,214,240` | the card claims "Metrics the wire contract does not deliver render '—', never a fabricated 0" (`nodes.tsx:4-5`), and then fabricates a zero-length bar | when `value === null`, render the track as a dashed 1px rule and `'—'`, or omit the bar |
| B5 | **High** | Approval modal ignores replay and has no keyboard contract (no trap, no initial focus, no Escape, no focus return); one `aria-modal` dialog per pending approval | `ApprovalOverlay.tsx:221`, `:243-249` | `.map` renders N `fixed inset-0` dialogs, all `aria-modal="true"`; no `useRef`, no `keydown`, no focus effect in the file | one dialog at a time (index the list), trap focus, focus the first enabled decision, Escape = dismiss when `actions.dismiss` |
| B6 | **High** | Composer — the app's primary input — has no visible focus indicator | `ChatCore.tsx:340` + `index.css:27-30` | `outline-none` sits in the utilities layer, which beats `@layer base :focus-visible` | add `focus-visible:outline-*` to the form/textarea |
| B7 | **Medium** | Raw off-palette `rgba` + a 28px coloured glow in canvas state styling | `Canvas.tsx:215` | `rgba(24, 156, 112, 0.45)` is not in the 20 swatches; G-UI-11 says zero colour outside the token module; `noHardcoded.test.ts:25` only greps hex so CI cannot see it | keep the 2px verify ring, delete the glow |
| B8 | **Medium** | Two "empty graph" messages render simultaneously | `Canvas.tsx:696` + `:787-793` + `index.css:143-152` | the minimap gets `hive-minimap-empty` from the same condition that shows the canvas pill | delete the CSS `::after` (D5) |
| B9 | **Medium** | `text-amber` used for small text, violating the project's own recorded correction | `ToolActivityPanel.tsx:58`; rule at `tokens.ts:76-78` | 11px amber on white = **3.64:1** < 4.5:1; `.text-amber-large` exists for the legal case | `text-ink-800` |
| B10 | **Medium** | The bee on every routed edge never bobs — the class has no rule | `BeeEdge.tsx:51` (`className="bee-bob"`) | `grep '^\.bee-bob'` → 0 hits; only `@keyframes bee-bob` (`index.css:182`) and a reduced-motion reference (`:339`). `motion.css` owns `.motion-bee-bob` (`:144`) | use `motion-bee-bob`, or delete the keyframe |
| B11 | **Medium** | `aria-live` misuse in three places: transcript `aria-live` instead of `role="log"`; `aria-label` on role-less `<div>`s; separator-less concatenation | `ChatCore.tsx:237-242` (`:241` aria-label on a bare div, duplicate of `:194`); `Canvas.tsx:733` (aria-label on a bare div); `Canvas.tsx:795-798` (no separator) | `aria-label` on a generic element computes no accessible name (ARIA 1.2), so both labels are inert; the sr-only region concatenates into `runningReconnecting…` | transcript → `role="log" aria-live="polite"`; move the labels to elements with roles; add `{' · '}` separators |
| B12 | **Medium** | Filter pills and the canvas counter expose no accessible meaning | `Canvas.tsx:639-641` | the counter is a bare `<span>{n}/{m}</span>` | `aria-label` on a `role="status"`: "Showing 12 of 40 agents" |
| B13 | **Medium** | Target select is labelled as a **worker** select but lists every kind | `ControlBar.tsx:45` vs `:51` | `aria-label={\`${STRINGS.node.workerLabel}: ${STRINGS.controls.reroute}\`}` = "WORKER: Re-route"; the options are all tasks | label it "Target agent"; the visible "Target" label at `:42` is already right |
| B14 | **Medium** | Range input is 4px tall | `Scrubber.tsx:56` | `h-1` on the time-travel scrubber | `h-6` + styled thumb |
| B15 | **Medium** | The approval dialog can block the UI while the operator is inspecting an old revision | `ApprovalOverlay.tsx:221` (no replay guard) | the overlay is not scoped to `history.replayGraph`; `run_id` is shown in the evidence but does not disabled the decisions | when replaying, render the dialog with `disabled` decisions + a "you are viewing revision N" note |
| B16 | **Medium** | Replay-invariant counts: the canvas counter and minimap-empty state read live tasks | `Canvas.tsx:640`, `:696` | `visibleTasks`/`tasks` are live; the rendered graph is `replayNodes` | compute both from the graph actually rendered |
| B17 | **Medium** | Below 1024px the Library panel (templates, active agents, workspace tabs) disappears with no alternative | `LibraryPanel.tsx:45` | `hidden … lg:flex`; `NavRail` has no Templates/agents-in-run destination (`NavRail.tsx:9-16`) | rail entry or a `lg:hidden` drawer trigger |
| B18 | **Low** | "1 revisions" | `HistoryPanel.tsx:28` | literal pluralization | singular/plural branch |
| B19 | **Low** | Ambiguous link between the canvas pill and its label | `Canvas.tsx:728` | `{STRINGS.states.replay} · {STRINGS.scrubber.label} {history.selected}` → "Replay (read-only) · Routing state 3"; `scrubber.label` is a **form control label**, reused as a noun phrase | a dedicated string: "Revision 3" |
| B20 | **Low** | `will-change` on non-compositable properties, permanently | `index.css:244,266`; `motion.css:68,106` | `background-position` is not composite-accelerated; the hint costs a stacking context and a promoted layer for nothing | delete those four hints |
| B21 | **Low** | Animated `box-shadow` contradicts the documented cost contract | `index.css:302`; `motion.css:375-376`; `ChatCore.tsx:196` | `motion.css:20` states "no filter chains, no backdrop-filter, no animated box-shadow" | drop `box-shadow` from the three transitions |
| B22 | **Low** | `.skip-link` animates `top` (layout property) | `index.css:364` | `transition: top 120ms ease` | `transform: translateY(-48px)` |
| B23 | **Low** | Hexagon chips lose their slanted edges | `nodes.tsx:274-278`; `LibraryPanel.tsx:114-119` | `clip-path` clips the border; a border paints only on the four rect edges | inline `<polygon>` |
| B24 | **Low** | Chat's tool-activity list shows raw enum values instead of the app's outcome labels | `ChatCore.tsx:302-314` | `{entry.outcome}` renders `pending` / `ok`; `STRINGS.toolActivity.outcomes` maps them to `Pending` / `OK` (`strings.ts:223-231`) | `toolActivityOutcomeLabel(entry.outcome)` |

---

## 9. Optional craft (not bugs — do not spend an edit round on these unless the above is done)

1. `Header.tsx:66` visible label "Hide chat" vs `:52` aria-label "Hide Hive chat" — pick one noun.
2. `NavRail.tsx:53` active state is a full `bg-ink-900` block; an ink hairline + honey icon would sit lower in the hierarchy and let the hive own the screen.
3. `ChatCore.tsx:216` close button `px-2 py-0.5` (~25×20) below the 24×24 comfort target even after I3.
4. `nodes.tsx:144` `pl-[30px]` with no leading glyph — likely a leftover from an avatar that no longer exists; remove the asymmetry rather than round it.
5. `index.css:184` `translateY(-1.5px)` sub-pixel bob (moot until B10 is fixed).
6. `LibraryPanel.tsx:193` version string retyped; derive from `package.json`.
7. `App.tsx:64`/`ChatCore.tsx:267` already-uppercase source strings + `text-transform: uppercase` = double uppercase; harmless, but it hard-codes a casing decision into copy.
8. `SwarmPanel.tsx:48` Playfair numerals for census counts (Y6) — looks editorial, reads imprecise.
9. `ControlBar.tsx:60` filled-honey primary at `disabled:opacity-50` → 2.46:1 (WCAG-exempt, visually weak).
10. The `!` important modifiers on node handles (`nodes.tsx:105,107,273,289,297`) — correct output, fragile input.

---

## 10. Bugs to add to known areas

Proposed additions to the standing known-areas list (this report does not edit any register). Ranked by whether they invalidate a claim the product makes about itself.

| New area | Why it belongs in known areas | Anchor |
|---|---|---|
| **Replay view is not replay-coherent** | Three surfaces claim replay while showing live data (HUD delta, LibraryPanel roster, canvas counter + minimap-empty), and approvals are not replay-scoped. The UI can show two runs at once. Extends `lead-plan.md:33` ("other replay counters still live-scoped") and `:40` (item 4) with exact lines | B1, B2, B15, B16 |
| **Honest-null rendering leaks a fake zero** | A `null` progress renders an empty bar inside cards whose own header promises never to fabricate a 0. This is a truthfulness defect, not styling — it belongs beside the "no fabricated metrics" guarantees | B4 |
| **Approval modal accessibility contract** | No focus trap / initial focus / Escape / focus return, and N stacked `aria-modal` dialogs. Already planned at `lead-plan.md:42`; this adds the stacking bug | B5 |
| **Primary-input focus is invisible** | The composer (and any future `outline-none` control) loses the app's focus ring to cascade order. A regression class, not a one-off | B6 |
| **Hardcoded copy drift vs `STRINGS`** | `strings.ts:2-5` declares "no copy hardcoded in components"; ~30 sites violate it, including 3 in the focal chat surface and 12 in the canvas. Any future copy/i18n pass hits all of them | T1-T5 |
| **Text/colour role leakage** | `text-white` introduces a 4th text role; `text-amber` violates the recorded large-only correction; one raw `rgba` bypasses G-UI-11 and the hex-only CI gate | C-o1, C-o3, B9 |
| **Motion cost contract vs motion code** | Two vocabularies, 19 infinite loops, 8 permanent `will-change` hints (4 on non-compositable properties), 3 animated box-shadows, 1 layout transition, 3 dead animations. The file comments assert the opposite | M1-M8, V5-V8, B20-B22 |
| **Dead CSS class ships as a missing animation** | `bee-bob` has no rule, so the signature routing marker silently lost its motion — the class of bug a class-rename produced | B10 |
| **Non-text contrast in the minimap** | The navigation widget's two most common marks are 1.70:1 and 1.84:1 | C-o5 |
| **Component-authoring drift** | 19 font sizes, 8 strokes, 11 icon boxes, 3 tracking values, 11 radius values of which **4 are off the declared stops** (6, 7, 11, 18 — `tokens.ts:107` allows 4/8/12/16/24/48/999 plus 10 and 14), 4 pill clusters. A shared primitives layer would prevent the next round | Y1, Y7, I1, I2, D4 |

---

## 11. Render evidence — **UNVERIFIED**

- **No baseline PNG, no final PNG, no `read_image` inspection.** This is deliberate and non-negotiable for this seat: `snapshot.mjs` drives headless Edge/Chrome, which the brief forbids ("no browser automation"), and `snapshot.mjs` would also need a served page.
- **Why a render was not possible even without that rule:** HiveMind is stopped (`lead-plan.md:51` records `hiverouter=0`; I independently found no HiveMind listener — only an unrelated `posterpal` Vite server on 8080, §0). There is no running origin to capture.
- Consequences, stated plainly:
  1. Every claim above is a **source** claim or an **arithmetic** claim. Nothing is a "looked at it" claim.
  2. Squint-level questions are unanswered: does the hive field actually read as one bright element at 1440×900? Does the 480px chat core crowd the control bar? Does the state chip overflow *visibly*? Does the mobile header clip? These are marked INFERRED (C15, C16, and §8 B3/B11 notes) or left open.
  3. Contrast numbers are computed (WCAG formula from `tokens/contrast.ts:41-47`), not eyedropped. They are correct for the token pairs; they do not account for the two gradients (`ChatCore.tsx:200`, `SwarmPanel.tsx:59`) at every point along their run.
- **What would close this gap:** `node assets/scripts/snapshot.mjs http://127.0.0.1:8080 audit/baseline.light.png --light --width 1440 --height 900` with the app running and a permitted browser, plus a 375×812 capture; then `read_image` on both. That is a different seat with different permissions.

---

## 12. What I did not audit

- `panels/AgentProfilesPanel.tsx`, `AgentMessagesPanel.tsx`, `AgentMemoryPanel.tsx`, `AgentHierarchy.tsx`, `DataPanel.tsx`, `ModelsPanel.tsx`, `SettingsPanel.tsx`, `ProjectsPanel.tsx`, `RoutingPanel.tsx`, `StatePack.tsx`, `DevGallery.tsx`, `flow/graph.ts`, `api/*`, `state/store.ts` beyond the replay/metrics paths — sampled via grep (type sizes, `aria-label`s, `text-signal`) but not read line-by-line. Findings there would be **additional**, not contradictory.
- Test suite (`web/tests/*`) was read only for the two pinning tests. I did not run `npm test` — that is not my seat and would touch the 187-test baseline.
- `web/dist/*` (built artifacts) — a committed `dist/` is visible at `web/dist/index.html` + hashed assets; whether it matches `src` is a **build-freshness** question, out of scope, flagged for whoever owns deploys.
- Backend rendering of errors (`backend/.../http.rs`) — only quoted where `lead-plan.md` already cites it.

---

## 13. Template-resilience answer

**Would this screen still feel tailored to this product if the name were swapped for a competitor's? Yes — the world carries it, the chrome does not.** The 5-lobe beehive with specular + rim gradients (`HiveField.tsx:39-108`), the honeycomb pattern at a real 48×42 cell rhythm (`:39`), the bee as an oriented routing marker (`BeeEdge.tsx:22-26, 51-56`), node cards sized by agent rank (420×150 / 320×96 / 420×96…, `LAYOUT.cards`), and a `ROUTING.snapRadiusPx = 150` that a re-parent actually uses — swap the wordmark and none of that survives translation to another product.

**Where it fails:** the middle band. The chat core (a rounded 400×480 card with a header bar and a composer), the control bar (four pill/flat buttons + two selects), and the HUD (pill + dot + three figures) are **generic agent-console furniture** — the same shapes would pass in any LLM dashboard. Three specific tells: (1) the chat header's ink→honey gradient is the single most "template" element on screen (C-o2); (2) the minimap recolours node kinds with a rainbow of honey steps for no semantic reason (C-o4); (3) the three traffic-light dots in the code block and tool node imply a window chrome that does not exist (C-o8). Removing those three and letting the hive + the approval evidence list carry the identity would tip the middle band toward this product. That is the polish direction; I did not apply it.

---

## 14. Report shape (AGENTS.md §24)

### Done

| Deliverable | Evidence |
|---|---|
| Deterministic audit of `web/src` and `web/index.html` with exit codes quoted | §4: exit **1** (61 error / 3 warning, 57 files) and exit **0** (1 file); full JSON preserved at `%TEMP%\round9-unslop-audit-src.json` |
| Nine-pass redline with `path:line` + rule + evidence + minimal fix for every infraction | §6, 87 rows across 9 passes (12/9/7/12/8/8/11/11/9) |
| Surface redlines for hierarchy / canvas / chat / rail / HUD / minimap / approval / control bar / replay | §7, including a replay-coherence matrix |
| Bugs separated from taste, with severity | §8 (24 bugs), §9 (10 optional craft items) |
| Claim register with VERIFIED / INFERRED / UNVERIFIED | §5, 20 rows |
| "Bugs to add to known areas" | §10, 10 areas |
| Scope-mode justification and five-line direction derived from the product's own declared world | §2, §3 |
| Honest render refusal instead of a fabricated screenshot | §11 |
| Wrote exactly one file | `docs/trace/round9/skill-unslop.md`; repo artifact moved to `%TEMP%` (§0) |

### Not done

| Asked | Why not |
|---|---|
| Rendered baseline + final PNG with `read_image` inspection | Forbidden (browser automation) and impossible (app stopped). See §11 |
| RED→GREEN proof that the audit catches a planted defect | Two edits would be required; this seat may not edit product code and may not create a second writer |
| Fixes for any infraction | Audit-only seat by instruction |
| Full read of the 12 remaining panels and `state/store.ts` beyond replay paths | Time-bounded seat; sampled by grep, disclosed in §12 |
| `npm test` / `tsc --noEmit` confirmation that the *baseline* is green | Belongs to the test seat; the 187-test baseline is quoted from `lead-plan.md:52`, not re-run here |

### Unverified

| Claim | What would prove it |
|---|---|
| Rendered hierarchy, spacing rhythm, and contrast at 1440×900 and 375×812 (C19) | One permitted 1440×900 light capture + one 375×812 capture, inspected with `read_image` |
| Mobile header overflow (C15) and chat-core clipping on short viewports (C16) | 375×812 and 1280×600 captures, or a CSS-width assertion harness |
| HUD fits 460px in the derived+delta+replay string combination (§7.5) | Render at the widest string state, or measure text width in a DOM test |
| Minimap `role`/`aria-label` survives React Flow mount timing (`Canvas.tsx:414-428`) | Mount-order test or a DOM snapshot after first paint |
| `role="application"` on the canvas is acceptable for this keyboard model (§7.2) | A screen-reader walkthrough (NVDA/JAWS) — not run |
| Hexagon chips visibly lose their slanted stroke (B23) | A 2× crop of one node card and one library chip |
| The audit rule set is complete enough for this codebase (§4 blind spots) | Extending `audit.mjs` with arbitrary-px-size, stroke, and Tailwind-`.5` rules, then re-running — a tooling change, not this seat |
| Whether `web/dist` matches `web/src` | A fresh build + hash comparison, by the deploy owner |

---

*Audit only. No file outside this report was created, modified, or deleted. HiveMind was not started, stopped, or contacted.*
