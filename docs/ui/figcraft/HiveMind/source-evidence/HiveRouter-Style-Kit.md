# HiveRouter — Spatial Style Kit v1.0

The applied rules for the HiveRouter spatial hive UI, produced in FigCraft (headless,
script-driven) from the target brief, the reference canvas image, and the shipped app tokens.
Every value here has a provenance; nothing was invented.

- Native master: [`HiveRouter-Style-System.figcraft`](HiveRouter-Style-System.figcraft)
- Boards: [`exports/`](exports/) — 8 artboards at 1x and @2x, plus one SVG
- Pipeline diagram: [`diagrams/HIVEROUTER-STYLE-SYSTEM.mmd`](diagrams/HIVEROUTER-STYLE-SYSTEM.mmd)
- Builder: [`build_hive_style.py`](build_hive_style.py)

**Verified token census** (`figcraft-cli info` + headless `style.list` / `variable.list`):
969 nodes · 8 frames · 36 styles (20 paint, 13 text, 3 effect) · 21 colour variables ·
1 collection · 0 missing fonts · 0 embedded rasters.

---

## 1. Colour

Provenance legend — **SAMPLED**: measured from the supplied reference image with Pillow;
**APP**: inherited from the shipped `web/src/styles.css` `:root` block; **DERIVED**: computed
from a sampled/APP anchor (stated in the note).

| Token | Hex | Name | Provenance |
|---|---|---|---|
| `surface/canvas` | `#F8F4EB` | Canvas | SAMPLED — dominant background cluster |
| `surface/panel` | `#FFFFFF` | Paper | SAMPLED |
| `surface/panel-alt` | `#FBF8F1` | Panel Alt | DERIVED — canvas/paper midpoint |
| `surface/recessed` | `#F1E9CF` | Recess | SAMPLED — pale honey inset |
| `border/hairline` | `#E5E1D8` | Hairline | APP — `--color-hairline` |
| `honey/100` | `#FBF3DC` | Honey 100 | DERIVED from honey/400 tint |
| `honey/200` | `#F6E7BE` | Honey 200 | DERIVED |
| `honey/300` | `#F0C050` | Honey 300 | SAMPLED — bright highlight |
| `honey/400` | `#E8B850` | Honey 400 | SAMPLED — dominant saturated gold (PRIMARY) |
| `honey/500` | `#D9A02F` | Honey 500 | DERIVED — pressed/gradient mid |
| `honey/600` | `#C88030` | Honey 600 | SAMPLED — amber shadow |
| `honey/700` | `#8A5A1C` | Honey 700 | DERIVED — deep outline |
| `honey/800` | `#604818` | Honey 800 | SAMPLED — bronze |
| `ink/900` | `#282010` | Ink 900 | SAMPLED — darkest warm ink (primary text, code) |
| `ink/800` | `#3F3115` | Ink 800 | SAMPLED — chat header base |
| `ink/700` | `#4E3A15` | Ink 700 | SAMPLED — header gradient end |
| `ink/500` | `#8A7A52` | Ink 500 | DERIVED — meta text |
| `accent/amber` | `#B7791F` | Amber | APP — `--color-amber` (links, eyebrows) |
| `state/error` | `#B23A2E` | Signal | DERIVED — "striking crimson" from Prompt 7 |
| `state/success` | `#3F7A55` | Verify | DERIVED — pass state |
| `canvas/honeycomb-line` | `#EFE7D2` | Honeycomb line | SAMPLED — grid line value |

**Contrast (WCAG 2.1, measured — printed on board 01):**

| Pair | Ratio | Verdict |
|---|---|---|
| ink-900 on canvas | 14.67:1 | AAA |
| ink-900 on honey-400 | 8.74:1 | AAA |
| white on ink-800 | 12.64:1 | AAA |
| amber on canvas | 3.32:1 | AA-LARGE only — **never use amber for body text** |
| error on canvas | 5.41:1 | AA |
| verify on canvas | 4.64:1 | AA |

Rules: honey-400 is a *surface*, never a text colour on light grounds. White only on ink-700
or darker. Amber is for eyebrows, links and one-word emphasis at ≥16px semibold.

## 2. Typography

| Role | Family | Weight | Fallback | Licence note |
|---|---|---|---|---|
| Brand / display | **Playfair Display** | SemiBold | Georgia, serif | SIL OFL — installed on this machine |
| UI / text | **Inter** | Regular / Medium / SemiBold / Bold | Segoe UI, Arial | SIL OFL — installed |
| Mono / code | **Consolas** | Regular | ui-monospace, Menlo | System font — installed |

**Font availability audit (verified by `figcraft-cli text.fonts`):** the shipped app CSS
declares `Instrument Serif` and `JetBrains Mono`; **neither is installed on this machine**.
Outlining either would bake a fallback. Ship the licensed files or accept the Playfair /
Consolas substitution before any outline step.

| Style token | Size / line / tracking |
|---|---|
| `Brand/Display XXL` | 96 / 104% / −3% |
| `Brand/Display L` | 56 / 107% / −2% |
| `UI/Heading H1` | 40 / 120% / −1.5% |
| `UI/Heading H3` | 24 / 133% / −0.5% |
| `UI/Heading H4` | 20 / 140% / 0 |
| `UI/Body L` | 17 / 165% / 0 |
| `UI/Body M` | 15 / 160% / 0 |
| `UI/Body S` | 13 / 154% / 0 |
| `UI/Label` | 12 / 133% / +8% (uppercase) |
| `UI/Label Micro` | 9.5 / 133% / +12% (uppercase) |
| `Mono/Code` | 13 / 169% / 0 |
| `Mono/Code S` | 10.5 / 150% / 0 |
| `Mono/Meta` | 11 / 145% / +8% (uppercase) |

## 3. Space, radius, depth

- **Spacing** — 4px base: 4, 8, 12, 16, 20, 24, 32, 40, 48, 64, 80, 96. Card padding 16,
  gaps only from {8, 16, 24}.
- **Radius** — 4, 8, 12, 16, 24, pill 999. SuperAgent 14, WorkerAgent 12, chat shell 18,
  composer 12, buttons 10, chips pill.
- **Depth** (warm ink, never neutral grey):
  - `e1 / node` — `0 1 3 rgba(40,32,16,.07)` + `0 8 24 rgba(40,32,16,.08)`
  - `e2 / float` — `0 2 6 rgba(40,32,16,.08)` + `0 20 48 rgba(40,32,16,.12)`
  - `e3 / hive` — `0 0 80 rgba(138,90,28,.24)` + `0 24 48 rgba(96,72,24,.20)`

## 4. Components

**HyperAgent core (hive chat).** 420×500 shell on an e2 float; header 44px, gradient
`#3F3115 → #604818` at 135°, bee glyph left, title Inter SemiBold 14, close `×`. Body paper.
User bubble `#F3F1EC` r10; assistant bubble `honey/100` r10; code block `ink/800` r8 with
three dots `#E05B4A · #E8B850 · #6FBF73` and mono 13 `#F1E9CF`. Composer: white, hairline,
r12, 56px, placeholder `ink/500`, three 15px icons right. Behind it: 5 stacked hive lobes with
radial gold gradients and `honey/700` outline — the origin (0,0) anchor.

**SuperAgent card.** 420×150, paper, r14, 1.5px `border/gold`, e1. Header strip `honey/100`
r14/14/0/0 with 5px state dot; name Inter SemiBold 14.5; `RUNNING` pill = honey-gradient r999,
mono 9.5 `ink/800`, +8% tracking. Body line Inter 11.5 `ink/600`. Progress: 5px track
`border/hairline`, fill honey-gradient. Meta mono 9.5 `ink/500`.

**WorkerAgent card.** 320×96, paper, r12, 1px hairline, e1. No header fill. 4.5px state dot,
name Inter Medium 13, role tag mono 9 +6%. Progress 3px.

**ToolAgent card.** 420×240, paper, r14, e1. Title + `TOOL` pill. Dark syntax box `ink/800`
r8 with dots, mono 10.5 code, 6px gold execution bar and mono 9 filename + ratio.

**Error state.** Paper `#FFF7F5`, 1.5px `state/error`, warning triangle, title Inter SemiBold
13 error, mono meta, body Inter 10.5 `ink/700`, and a **dashed crimson bee route** below with
the bee paused.

**Agent chip.** 40px hexagon, ink face, bee glyph in the state colour, label Inter Medium 9.5
underneath. State dots: healthy `honey/400`, running `honey/600`, failed `state/error`, queued
`border/hairline`.

**Controls.** Primary = honey gradient, ink label, r10. Secondary = paper + gold border.
Ghost = amber label. Danger = `#FFF7F5` + crimson border. Composer, tabs (active = paper + gold
border), HUD pill (top-centre), mini-map (bottom-right), timeline scrubber (bottom-centre).

## 5. Motion

- Bee routes: dashed `[10,10]` at 1.5px, bee 13–18px, **always facing the data direction**.
  The bee is a routing arrow, not decoration.
- Progress: width transition only, 200ms; never a spinner on a card.
- Pan/zoom: transform-driven; Cmd/Ctrl+0 eases the viewport back to the hive core.
- Reduced motion: bees static at the midpoint of the route; no dash animation.

## 6. Asset index

| File | What |
|---|---|
| `exports/hiverouter-cover.png` | Brand cover, 1440×900 |
| `exports/hiverouter-colour.png` | 20 swatches + ramps + measured contrast, 1440×1240 |
| `exports/hiverouter-typography.png` | 3 families + 8-step scale + font audit, 1440×1340 |
| `exports/hiverouter-space-and-depth.png` | spacing / radius / elevation, 1440×1000 |
| `exports/hiverouter-brand-lockups.png` | primary, icon, mono, inverse, clear space, min sizes |
| `exports/hiverouter-agent-cards.png` | 5 node types + anatomy + chips, 1440×1300 |
| `exports/hiverouter-controls-and-hud.png` | composer, buttons, tabs, rail, panel, HUD, timeline |
| `exports/hiverouter-spatial-canvas.png` | the full reference scene, 1440×900 |
| `exports/hiverouter-brand-lockups.svg` | vector lockup sheet (real paths, viewBox) |
| `exports/*@2x.png` | 2× raster for the same boards |

## 7. Regenerating

```powershell
$py = 'C:\Users\Riley\.dsh\dsh-runtimes\dsh-primary-runtime\dependencies\python\python.exe'
$cli = 'C:\Program Files\FigCraft\figcraft-cli.exe'
cd C:\Users\Riley\Desktop\HiveRouter\design\hive-style
& $py build_hive_style.py 1      # file + 8 frames + every node
& $cli run commands_phase1.json --out base.figcraft
& $py build_hive_style.py 2      # variables + styles + exports + saveAs
& $cli run commands_phase2.json --in base.figcraft
```

Node ids are deterministic (page = 1, +1 per created layer); the builder writes
`expected_ids.json` so a rebuild can be diffed against the CLI's returned ids.

## 8. Known deviations

- Dark mode is **out of scope**: the brief specifies a light spatial canvas; a dark hive would
  need a second token mode and was not invented.
- The colour board prints measured contrast; `amber/canvas` is deliberately shown as
  AA-LARGE to prevent it being used for body text.
- The cover's meta line states the design-system counts; it is kept in sync with the verified
  census above.
