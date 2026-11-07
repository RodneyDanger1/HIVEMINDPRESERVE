# HiveMind Shell Parity — Gap Matrix (01)

**Scope:** shell chrome only — 52px topbar, 72px rail, 240px left panel, HUD, 220×150 minimap, control bar + scrubber, typography, colour, spacing/radii, responsive, overlay layering.
**Status of this document:** analysis only. No product file was edited. The only file written is this one.
**Analysis snapshot:** 2026-11-07 15:11–15:12 America/Chicago. The working tree was being edited *by another process during* this analysis (see §0.4) — every row cites the file state at that snapshot, and hashes are recorded so drift is detectable.
**Verdict:** the five hard geometry facts the rebuild depends on are now pinned in code (`LAYOUT`) and match the reference. The remaining gaps are **anatomy, placement, colour token coverage, and chrome that does not exist yet** — not widths.

---

## 0. Method and claim discipline

### 0.1 Evidence legend

| Tag | Meaning |
|---|---|
| `B07:nnnn` | `docs/ui/figcraft/HiveMind/build/build_hive_style.py:nnnn` — the deterministic builder that **generated** the reference PNG (`board_hero`, B07:1147-1250). This is committed design source, not a screenshot. |
| `B06:nnnn` | Same file, `board_controls` (B06:1018-1141) — the component board. |
| `MEASURED` | Derived by scanning the canonical PNG with a script (token-colour masks + longest-run detection). Values quoted are the scan result. |
| `DERIVED` | Arithmetic on two declared/measured numbers (e.g. `540 − 312 = 228`). |
| `ESTIMATED` | My inference where neither artboard nor code states the answer. Flagged because it is **not** authoritative. |
| `UNDEFINED` | The reference set does not specify it. A decision is required; it is not a gap to "fix". |

> **Line-number note.** All `B07:`/`B06:` citations refer to the **in-repo** copy
> `docs/ui/figcraft/HiveMind/build/build_hive_style.py` (1685 lines). The external original
> (`C:\Users\Riley\Desktop\HiveRouter\design\hive-style\build_hive_style.py`, 1369 lines) is the
> same builder one line earlier for these functions — `board_hero` is L1146 there, L1147 here —
> because the repo copy additionally appends the two HMAS boards. Subtract 1 when cross-checking
> against the external file.

### 0.2 Why two evidence sources are used, and what that is *not*

The artboards and `build_hive_style.py` agree, and that agreement is the point: the PNG scan only **corroborates** the builder constants (they matched exactly for the HUD, minimap, rail tile, panel edges, and buttons). No token value in this document is derived from sampled pixels — `web/src/assets/manifest.ts:8-13` forbids exactly that ("Use the token module for all values; never sample pixels from a board export"), and `noHardcoded.test.ts:19-30` enforces hex isolation. Recommended values below therefore cite either `tokens.css` or the builder.

### 0.3 Reference resolution rule (boards disagree — this resolves it)

The reference set contains two scales for the same components. `docs/ui/figcraft/HiveMind/README.md:44-48` states board 07 **keeps the authoritative composition** and lists its numbers. Therefore:

> **Board 07 (1440×900 `hivemind-spatial-canvas.png`) is authoritative for shell geometry. Board 06 (`hivemind-controls-and-hud.png`) is authoritative for component internals that board 07 does not draw.**

Consequences: HUD is **460×44** (B07) not 420×52 (B06); minimap **220×150** not 260×170; library **240** not 260; rail pitch **92** not 88.

### 0.4 Snapshot caveat — the tree moved during analysis

`web/src/tokens/tokens.ts` gained a `LAYOUT` export, and `HudBadge.tsx` / `LibraryPanel.tsx` / `Canvas.tsx` were rewritten to consume it, **while this analysis was running** (an untracked `web/tests/designParity.test.ts` also appeared, which now pins the same numbers). Rows below describe the post-change state. Snapshot hashes (`sha256`, first 12):

| File | mtime | sha256 | Note |
|---|---|---|---|
| `web/src/panels/Header.tsx` | 11-07 14:45:42 | `D06070D3437A` | unchanged during analysis |
| `web/src/panels/NavRail.tsx` | 11-07 14:57:03 | `74164F90F6E7` | unchanged |
| `web/src/panels/NavIcon.tsx` | 11-07 14:56:57 | `A9586AF48D63` | unchanged |
| `web/src/panels/LibraryPanel.tsx` | 11-07 15:06:02 | `2B2EACC20E55` | **changed mid-analysis** |
| `web/src/panels/HudBadge.tsx` | 11-07 15:05:15 | `9180E704BC6F` | **changed mid-analysis** |
| `web/src/flow/Canvas.tsx` | 11-07 15:04:45 | `1B98B49BC30B` | **changed mid-analysis** |
| `web/src/tokens/tokens.ts` | 11-07 15:01:13 | `00976BBD4343` | **changed mid-analysis** |
| `web/src/panels/ControlBar.tsx` | 11-06 04:52:35 | `A22B87B6D259` | unchanged |
| `web/src/panels/Scrubber.tsx` | 11-06 21:54:07 | `3329167DD0A8` | unchanged |
| `web/src/panels/ToastRegion.tsx` | 11-06 04:52:38 | `403D24C553BF` | unchanged |
| `web/src/styles/tokens.css` | 11-06 04:19:37 | `43A0B8E4C07E` | unchanged |
| `web/src/styles/index.css` | 11-06 04:19:39 | `E3925C3E6BFE` | unchanged |
| `web/tests/designParity.test.ts` | 11-07 15:02:15 | `2E440E206350` | new, untracked |

### 0.5 The live server does not represent current source

`http://127.0.0.1:8099` answered HTTP 200 early in this session (`bytes=774`) and served `web/dist` built **14:52:11** — nine minutes *older* than the current `web/src` shell. That stale bundle renders a superseded left panel ("STATE VOCABULARY", "SETTINGS", project title block, no hex chips) which does **not** exist in current source. The listener stopped answering during analysis (no process on :8099; **I did not kill it**).

The single live capture is therefore used for exactly two things, both valid because the components are byte-identical across dist and src (mtimes 11-06):

1. rendered **heights** of `ControlBar` (47px) and `Scrubber` (43px);
2. the **toast-over-topbar overlap** (SH-O1).

It is *not* used as evidence for left-panel or HUD appearance.

---

## 1. Reference geometry atlas (the numbers the rebuild must hit)

All in the 1440×900 artboard coordinate space; `canvas-local` = minus (312, 52).

| Region | Reference geometry | Evidence | Corroboration |
|---|---|---|---|
| Topbar | 52px tall (0–51); 1px hairline at y=52; `PANEL` fill | `B07:1150-1151` | MEASURED: white to y=50, rule y=51, canvas from y=52 |
| Topbar brand | mark 28×28 at (16,12); wordmark 17px **Bold** `INK900` at x=52 | `B07:1152-1153` | MEASURED: mark ink 16–43, glyphs from x≈53 |
| Topbar right | search 18px (1214,17); bell 18px (1256,17); `ERR` dot 8px (1272,12); avatar 32px `H200` (1300,10); initials 11px `INK800` (1308,17); name 12.5px Medium (1344,16); chevron 8×5.6 `INK500` (1404,22) | `B07:1154-1160` | MEASURED: bell ink to x=1268; dot 1272–1279 × 12–19; avatar 1300–1331 × 10–41 |
| Topbar middle | **empty** — no tabs, no content between brand and right cluster | `B07:1150-1160` (no node emitted) | MEASURED: **zero** dark-ink pixels in x=400–1150, y=6–46 |
| Rail | 72px wide; 6 destinations, pitch **92px**; icon 22px at (25, y+7); label 8px centred on x=36 at y+50; active tile **48×48 r11** at (12,68) `INK900`; inactive icon `INK700`; active icon `H400` | `B07:1163-1175` | MEASURED: tile 12–59 × 68–115 (48×48); icon bands 263/359/447/543 (Δ92) |
| Left panel | 240px (x 72–312), full height, `PANEL_ALT`; hairline at **x=312 only** | `B07:1178-1179` | MEASURED: rail 0–71 white, panel-alt from 72; rule at 312; canvas from 313 |
| Panel selector | 216×32 r8 at (84,68), `PANEL` + `HAIR`; text 11.5 Medium `INK800` at (96,76) | `B07:1180-1181` | MEASURED: hairline bbox 84–299 × 68–99 |
| Panel heading | 11.5 **SemiBold** `INK900`, title case, at (84,116) | `B07:1183` | — |
| Hex chips | 40px **pointy-top hex** (≈34.6 wide), face `INK900`, stroke `BORDER_GOLD` 1.2, bee glyph `H400`, label 9.5 Medium `INK700` below | `B07:1184-1187`; `hex_chip` `build:906-911` | MEASURED: chip stroke x 87–120/145–178/203–236 (Δ58), y 140–176 then 213–247 |
| Templates | rows 216×**26** r7, **pitch 32**, text 11 `INK800`, layers icon 12 `INK600` | `B07:1188-1194` | MEASURED: row rules at y 306/331, 338/363, 370/395, 402/427 |
| Canvas | x 312–1440, y 52–900 (1128×848); honeycomb R=44 `#EFE6CE` | `B07:1197-1198` | MEASURED: first canvas px at x=313 |
| Canvas labels | "HIVEMIND / SPATIAL CANVAS" 9.5 mono `#A0916F` at (328,70); footer legend 9.5 mono at (328,866) | `B07:1247-1250` | MEASURED: footer ink 347–732 at y 868–872 |
| HUD | **460×44**, r999, `PANEL`, `BORDER_GOLD` 1.2, `SH_E2`; dot 10px at (564,90); label 10px mono SemiBold `INK800` ls 6% at (580,83) | `B07:1229-1234` | MEASURED: gold bbox 540–999 × 68–111 (=460×44); dot 559–568 × 85–94 |
| Minimap | **220×150** at (1120,700), r12, `PANEL` + `HAIR`, `SH_E2`; lattice inset 6; nodes `H100` + `H600` ~1px, r3; viewport rect 100×60 `INK900` 1.4 r6 | `B07:1236-1244` | MEASURED: white 1121–1338 × 701–848; borders 1120/1339, 700/849 |
| Buttons | 44px tall, r10, 13px SemiBold (primary/secondary/danger) / Medium (ghost); primary `HONEY_GRAD`; secondary `PANEL`+`BORDER_GOLD` 1.5; ghost `AMBER`; danger `#FFF7F5`+`ERR` 1.5 | `B06:1000-1015, 1037-1043` | MEASURED: primary fill 72–239 × 372–415 (=168×44) |
| Tabs | 34px tall, r8, gap **18**, active `PANEL` + `BORDER_GOLD` 1.2, 12px Medium (active) / Regular; width = 24 + 8.4·n | `B06:1046-1055` | MEASURED: gold tab 190–254 × 486–519 (65×34) |
| Scrubber | 600×**66** r12 `PANEL`+`HAIR`+`SH_E1`; label 9.5 mono `INK500` at (96,706); track **552×4** at (96,728); 7 marks 3px @92; thumb 15px ring `H500` 2px + white fill | `B06:1067-1078` | MEASURED: panel 72–671 × 696–761; thumb ring ≈408–423 |

Control-bar **placement** and scrubber **placement** are not drawn on board 07 — see §4.

---

## 2. Gap matrix

Each row is one actionable delta. `Reference` cites §1 (or the builder line directly); `Current` gives the snapshot `path:line`; `Fix seam` is the smallest edit that closes it; `Pri` rolls up in §3. Rows with `Pri = —` are already at parity and are listed to stop the rebuild churning them.

### 2.1 Topbar (52px)

| ID | Reference | Current (`path:line`) | Exact gap | Fix seam | Pri |
|---|---|---|---|---|---|
| SH-T1 | No content between brand and right cluster (MEASURED 0 ink px in x400–1150) | 4-tab `role="tablist"` centred in the bar: `Header.tsx:37-53`, `TABS` at `:6-11` | Structural: the reference topbar is brand + search/bell/avatar only. Tabs occupy the optical centre and are the single largest deviation. | Delete `Header.tsx:37-53`; relocate the tablist (see §4 D1). Preserve `role=tab`/`aria-selected`/roving focus at the new host. | **P0** |
| SH-T2 | Search 18px glyph at (1214,17), `INK900`, stroke 1.6 | Absent. No search/filter in `web/src` at all (grep) | Missing control. Also net-new behaviour, not restyle. | `Header.tsx` right cluster: insert before the connection pill. Needs a real target or an honest disabled state (D2). | P1 |
| SH-T3 | Bell 18px at (1256,17) + `ERR` dot 8px at (1272,12) | Absent. No notification concept in `web/src` | Missing control + missing unread affordance. | `Header.tsx`; the dot must be data-driven, never decorative. | P1 |
| SH-T4 | name 12.5 Medium `INK800` at 1344; chevron 8×5.6 `INK500` at 1404; ≈28px right inset (DERIVED) | avatar 32 ✓, initials 11 ✓ at `Header.tsx:70`; name `hidden … sm:block` `:73`; **no chevron**; container `px-3` = 12px | No disclosure chevron; right inset 12 vs ~28; name hidden below `sm` | `Header.tsx:55-74`: add chevron button (real menu or absent), right padding to 28px | P2 |
| SH-T5 | No connection-status chip anywhere in the topbar | Pill at `Header.tsx:56-68` (`type-mono-meta`, pill border, truthful state colours) | Reference has no counterpart. Removing it would violate the honesty contract (offline/refusing states must stay visible). | **Decision D3** — keep as documented deviation (recommended) or fold the state into the HUD/toast. Do not delete silently. | P1 |
| SH-T6 | mark = hexagon + **bee glyph** (`hexmark()` B07:1152) | nested hexagons, `Header.tsx:22-33`; left inset 12 (px-3) vs 16 | Icon art differs; 4px left inset | `Header.tsx:22-33` swap paths for a bee mark; container `pl-4` | P2 |
| SH-T7 | If tabs live anywhere: active = `PANEL` + `BORDER_GOLD` 1.2, 34px tall, r8, 12px **Medium**, gap 18 | active = `bg-honey-100` fill, no stroke, `py-1.5` (≈30px), r8, `type-label` 600 uppercase, `gap-1` | Fill vs outline; 30 vs 34px; weight 600 vs 500; uppercase; gap 4 vs 18 | Wherever tabs land: `border border-[var(--color-border-gold)] bg-panel` active; `h-[34px]`; `gap-[18px]` | P1 |

### 2.2 Nav rail (72px)

| ID | Reference | Current (`path:line`) | Exact gap | Fix seam | Pri |
|---|---|---|---|---|---|
| SH-R1 | pitch **92px**, 6 items | pitch **70px** (`h-[62px]` + `gap-2`): `NavRail.tsx:26,48` | 22px per item; the rail's rhythm reads much tighter than the reference | `NavRail.tsx:26` `gap-2` → `gap-0`; `:48` `h-[62px]` → tile 48 + label; pitch = tile + label block + gap ≈92 (**ESTIMATED** decomposition; the 92 total is declared) | **P0** |
| SH-R2 | active tile **48×48** r11; label sits **below** the tile on the panel | tile is the whole button **56×62**, label **inside** it: `NavRail.tsx:48-55` | Box is 8px wider and 14px taller than the tile; label is on `INK900` instead of on `PANEL` | `:48` restructure: outer `flex flex-col items-center` (no border/radius) → inner `flex h-12 w-12 items-center justify-center rounded-[11px]` + label `<span>` after it | **P0** |
| SH-R3 | inactive items are **bare** icon + label | inactive items are bordered objects (`border border-transparent rounded-[11px]`): `NavRail.tsx:48-52` | Inactive items read as buttons-in-a-box rather than bare glyphs | Same restructure as SH-R2; drop the border from the outer element | **P0** |
| SH-R4 | active label `INK900` (on `PANEL`); inactive `INK500` | active label `honey-300` (on `INK900`); inactive `INK500` ✓ | Active label colour/host surface inverted | `:50` → `text-ink-900` on the label; keep `text-honey-300` for the **icon only** | **P0** |
| SH-R5 | no rule at x=72; the boundary is `PANEL`→`PANEL_ALT` contrast (MEASURED white to x=71, panel-alt from 72) | `border-r border-hairline` at x=71: `NavRail.tsx:26` | Extra 1px line the reference does not draw | `:26` drop `border-r border-hairline` | P2 |
| SH-R6 | active icon `H400` `#E8B850` | `honey-300` `#F0C050` (`NavIcon.tsx:5`) | One step brighter than reference | `NavIcon.tsx:5` → `--color-honey-400` | P2 |
| SH-R7 | rail "Agents" uses the **bell** glyph (`icon_bell`, B06:1085 / B07:1165) | "Agents" uses a dome/robot glyph: `NavIcon.tsx:13` | Reference would put a bell in the rail *and* a bell in the topbar (SH-T3) — the same glyph twice in one shell | **Decision D5**: keep current distinct glyph (recommended) and give the topbar bell a different treatment, or accept duplication | P2 |
| SH-R8 | first tile top = 68 (16px below bar) | `py-3` → first button top = 64 | 4px | `NavRail.tsx:26` `py-3` → `pt-4` | P2 |

### 2.3 Left panel (240px)

| ID | Reference | Current (`path:line`) | Exact gap | Fix seam | Pri |
|---|---|---|---|---|---|
| SH-P1 | "Active Agents" / "Templates" — **title case**, 11.5 SemiBold `INK900` | `text-[11.5px] font-semibold text-ink-900` ✓ but the strings are `'ACTIVE AGENTS'` / `'TEMPLATES'`: `LibraryPanel.tsx:37,65`; `strings.ts:114-120` | Size/weight/colour match; **copy case does not** (renders as shouting; board 06's uppercase 9px treatment is the *specimen*, not board 07) | `strings.ts:114-120` → `'Active Agents'`, `'Templates'`. Keep `type-label` uppercase out of these headings. | P1 |
| SH-P2 | pointy-top hex: ≈34.6w × 40h, `BORDER_GOLD` 1.2 stroke, `INK900` face | 40×40 box, `clipPath` polygon at `LibraryPanel.tsx:52` — vertices (25%,6.7%)(75%,6.7%)(100%,50%)(75%,93.3%)(25%,93.3%)(0,50%) ⇒ a **flat-top** hex (rotated 90° vs reference) inside a 40×40 square, **no stroke**: `:50-53` | Rotated 90°; non-regular (40 wide where a 40-tall pointy-top is ≈34.6); missing gold stroke; 5.4px too wide | `:50-53` → `width: 34.6, height: 40` with a pointy-top path (or inline SVG `hex_path(20)`) + `stroke: var(--color-border-gold)` 1.2 | P1 |
| SH-P3 | bee glyph inside, **tinted by state**, no text inside the chip | `StateGlyph size={9}` at `LibraryPanel.tsx:54` — and `StateGlyph` renders shape **plus** an 11px mono uppercase label (`flow/nodes.tsx:21-35`) | Text is rendered inside a 34–40px hexagon → clipped/overflowing; reference encodes state by **colour of the bee**, not by glyph shape + word | `:54` replace with a bee path whose fill = `tokenColor(visual.toneToken)`; the state word moves to the button's `aria-label` (already present at `:48`) | P1 |
| SH-P4 | label 9.5px **UI Medium**, title case, `INK700`, ~6px below chip | `type-mono-meta text-[8px]` → 8px mono **UPPERCASE** `ink-700`: `LibraryPanel.tsx:56` | Font family, size, case | `:56` → `text-[9.5px] font-medium text-ink-700` (no `type-mono-meta`) | P1 |
| SH-P5 | chip pitch **58**; 3 chips in row 1, 1 in row 2 (declared positions) | `li.w-12` + `gap-2` → pitch 56; 4 fit per row (216px content) | 2px pitch; wrap count differs — but board 07's 3+1 split is demo content (`B07:1184-1187` draws "Spark" twice), so the wrap **rule is UNDEFINED** | `:41-43` `gap-2` → `gap-x-[23.4px]` to yield pitch 58; do **not** hard-code a 3-column grid without a decision | P2 |
| SH-P6 | rows **26px tall**, **pitch 32**, r7 | `h-[30px]` + `space-y-1.5` (6px) ⇒ pitch 36, r7 ✓: `LibraryPanel.tsx:71,66` | +4px height, +4px pitch per row (×4 rows ≈ 16px cumulative) | `:71` `h-[30px]` → `h-[26px]`; `:66` 6px gap already correct ⇒ pitch becomes 32 | P2 |
| SH-P7 | chevron 8×5.6 vector, `INK500`, stroke 1.5, at x≈274 | `⌄` text glyph at 11px `text-meta`: `LibraryPanel.tsx:33` | Glyph type, optical size, weight | `:33` → 8×5.6 SVG path (`M0 0 L4 4.8 L8 0`, `stroke=ink-500`, 1.5) | P2 |
| SH-P8 | TIP: 228×96 r**10** `H100`, insets 12/10, label 8.5 mono **SemiBold `H800`**, body 10.5 `H800` lh 145% | `rounded-[8px] bg-honey-100 p-2`, label `type-mono-meta text-meta` (11px **`ink-500`**), body 10.5 `ink-700` lh 15px, no fixed height: `LibraryPanel.tsx:86-88` | r8 vs 10; inset 8 vs 12/10; label 11 vs 8.5 and `ink-500` vs `H800`; body `ink-700` vs `H800`; 228→216 (DERIVED: −24 inset for the 240 panel) | `:86` `rounded-[10px] px-3 py-2.5`; `:87` `text-[8.5px] font-semibold text-honey-800`; `:88` `text-honey-800 leading-[1.45]`. Board 07 omits TIP ⇒ panel placement remains **UNDEFINED** | P2 |
| SH-P9 | no equivalent block | `<details>` State & settings + "HiveMind v1.2-dev" `LibraryPanel.tsx:91-119` | App-honesty addition (shows provider/model/key) with no reference counterpart | Keep (honesty). Ensure it stays collapsed and does not displace TIP; `mt-auto` already pushes it down | P2 |
| SH-P10 | panel present at all widths (it is a fixed structural element) | `hidden … xl:flex`: `LibraryPanel.tsx:26` | Panel vanishes below 1280 — see SH-Z2 | see SH-Z2 | P1 |

### 2.4 HUD

| ID | Reference | Current (`path:line`) | Exact gap | Fix seam | Pri |
|---|---|---|---|---|---|
| SH-H1 | 460×44 | **460×44 via `LAYOUT.hud`** (`HudBadge.tsx:17`) | **none — already at parity** (was content-driven before the concurrent edit) | none; keep pinned | — |
| SH-H2 | (540,68) abs = canvas-local (**228,16**) | `absolute left-1/2 top-3 -translate-x-1/2`: `HudBadge.tsx:14` → local x = (1128−460)/2 = **334**, y = 12 | **106px right** of the reference (DERIVED 334−228) and 4px high | `:14` → `left-[228px] top-4` at ≥1440, or keep centring and accept a **documented** 106px deviation (D4) | **P0** |
| SH-H3 | border `BORDER_GOLD #E5C27A` 1.2 | `border-honey-400` `#E8B850`: `HudBadge.tsx:16` | Wrong hue (token does not exist — SH-K1) | `:16` → `border-[var(--color-border-gold)]` + `border-[1.2px]` | **P0** |
| SH-H4 | dot 10px | `h-[6px] w-[6px]`: `HudBadge.tsx:21` | 4px too small | `:21` → `h-2.5 w-2.5` | P1 |
| SH-H5 | 10px mono, tracking 6% | `type-mono-meta` + `text-[9.5px]` → 9.5px, tracking 8% (`HudBadge.tsx:16`; `index.css:84-90`) | Size 9.5 vs 10; tracking 8% vs 6% | `:16` `text-[10px] tracking-[0.06em]`, drop `type-mono-meta` (or add a token step) | P1 |
| SH-H6 | pill content is one 10px run; **delta is below** the pill (B06:1065) | delta rendered **inside** the pill: `HudBadge.tsx:29-31` | Extra token widens the run and breaks the reference rhythm | `:29-31` → move below the pill (absolute, `top: calc(100% + 6px)`) | P1 |
| SH-H7 | `SH_E2` = `0 2px 6px #28201014` + `0 20px 48px #2820101F` (B builder 68-71) | `var(--shadow-e2)` = single `0 6px 20px rgb(40 32 16 / .12)` (`tokens.css:46`) | Different depth recipe (one short soft shadow vs wide ambient) | `tokens.css` add a chrome shadow pair; or accept as documented simplification | P2 |
| SH-H8 | dot at 24px from left, text at 40px | `px-4` = 16px, `gap-2` = 8px between every token | Inset 16 vs 24; uniform 8px gaps vs a single text run | `:16` `pl-6 pr-4`; wrap the text in one `<span>` instead of 7 gap-separated spans | P2 |

### 2.5 Minimap (220×150)

| ID | Reference | Current (`path:line`) | Exact gap | Fix seam | Pri |
|---|---|---|---|---|---|
| SH-M1 | inset **100px right / 50px bottom** (DERIVED: 1440−1340, 900−850) | React Flow `.react-flow__panel { margin: 15px }` + bottom-right ⇒ **15px** (`node_modules/@xyflow/react/dist/style.css:291-294`; MiniMap renders `<Panel>` at `esm/index.js:4808`) | 85px right / 35px bottom short | `Canvas.tsx:378-384` style → add `margin: 0, right: 100, bottom: 50` (inline beats the panel class) | **P0** |
| SH-M2 | no veil; viewport = `INK900` 1.4px outline (B07:1244) | default `maskColor` `rgba(240,240,240,.6)`, `maskStrokeColor` `transparent` (`style.css:16-17`) | Grey veil over everything outside the viewport — a large, immediately visible colour break; no viewport outline | `Canvas.tsx:359-385`: `maskColor="transparent"` `maskStrokeColor="var(--color-ink-900)"` `maskStrokeWidth={1.4}` | **P0** |
| SH-M3 | nodes `H100` fill + `H600` ~1px, r3 | per-kind fill (`honey-400/600`, `ink-500`, `signal`, `honey-300`), default stroke `transparent` 2px, no radius | Node colour language differs (state-honey vs silhouette); no stroke/rounding | `:363-377` → single `nodeColor="var(--color-honey-100)"`, `nodeStrokeColor="var(--color-honey-600)"`, `nodeStrokeWidth={1}`, `nodeBorderRadius={3}` | P1 |
| SH-M4 | `SH_E2` | no shadow (only border) | Missing elevation; the card reads flat on the canvas | `:378-384` add `boxShadow: 'var(--shadow-e2)'` | P1 |
| SH-M5 | honeycomb lattice inside the card, inset 6, R=12 (B07:1239) | none — flat `PANEL` fill | Missing texture | CSS on `.react-flow__minimap` (background-image) or an inset `<svg>` layer | P2 |
| SH-M6 | 220×150, r12, `PANEL`, `HAIR` | 220×150 ✓, r12 ✓, panel ✓, hairline ✓ (`Canvas.tsx:379-384`) | **none** | — | — |

### 2.6 Control bar and scrubber

| ID | Reference | Current (`path:line`) | Exact gap | Fix seam | Pri |
|---|---|---|---|---|---|
| SH-C1 | Board 07 draws **no** bottom dock; canvas runs to y=900. Spec §4 Prompt 10: timeline "floating at bottom center" (SPEC:180-181) | Two **in-flow** full-width bars: `App.tsx:96-97` after the content div | Structural: docked bars remove **90px** (MEASURED 47+43) from the canvas and break the "chrome floats" language | `App.tsx:84-98`: move `<ControlBar/>`/`<Scrubber/>` inside the `relative` content wrapper (`:95`) as absolutely-positioned overlays; canvas keeps full height | **P0** |
| SH-C2 | 44px tall (B06:1038-1041, 1042-1043) | `px-3 py-1.5` + 16px line + 2px border = **30px** (`ControlBar.tsx:58,83,97,107`); measured bar 47px = 1+8+30+8 confirms 30 | 14px short of the declared 44 and of the "40px min touch target" note (B06:1042) | all four buttons → `h-11 px-4` (44px) | **P0** |
| SH-C3 | 13px SemiBold/Medium, **sentence case** ("Spawn sub-hive") | `type-label` = 12px / 600 / uppercase / +0.08em (`ControlBar.tsx:58`; `index.css:70-77`) | 12 vs 13px; uppercase vs sentence case; 0.08em tracking vs 0 | `:58,83,97,107` → `text-[13px] font-semibold normal-case tracking-normal` | **P0** |
| SH-C4 | 4 variants: primary `HONEY_GRAD`; secondary `PANEL`+`BORDER_GOLD` 1.5; ghost `AMBER` text; danger `#FFF7F5`+`ERR` 1.5 (B06:1000-1015) | all are transparent ghosts with `border-hairline`; only Kill differs (`border-signal text-signal`, `ControlBar.tsx:107`) | No visual hierarchy: the primary action is indistinguishable from Cancel | spawn → gradient fill + `INK800` label; re-route → `border-border-gold`; cancel → `text-amber` no border; kill → `bg-[#FFF7F5]` | P1 |
| SH-C5 | 600×66 r12 panel; track 552×4; 15px ring thumb; 7 marks @92 (B06:1069-1078) | full-width **native** `<input type="range">` with `h-1` + `accent-color` only (`Scrubber.tsx:46-60`) | No panel, no fixed width, UA thumb/track rendering (browser-dependent), no marks | `Scrubber.tsx:41-74` rebuild as a fixed 600×66 panel; style `::-webkit-slider-thumb`/`::-moz-range-thumb` in `index.css` to a 15px white disc + 2px `H500` ring | **P0** |
| SH-C6 | label "09:41:07 · routing state 12 of 38", 9.5 mono `INK500` **above** the track (B06:1077) | `type-label` uppercase "Routing state" to the **left**; readout at the right (`Scrubber.tsx:43-45,61-65`) | Missing timestamp; label position/case; readout "1 of 1" is data-only | `:42-65` restructure to label-above-track | P1 |
| SH-C7 | Spec requires Play / Pause / Step-Back (SPEC:184-185); **board 06 draws none** | none | Reference conflict. Building them needs new store actions (scrub step backwards already exists via revisions) | **Decision D6**: build (spec wins) or document the omission | P1 |
| SH-C8 | no selects in the reference | role select + re-route select (`ControlBar.tsx:42-54,68-80`) | App-specific; no parity target | Restyle only: `type-mono-meta` + `border-hairline` + `bg-panel` (already close) | P2 |
| SH-C9 | — | status readout `ml-auto` (`ControlBar.tsx:116-122`) | none material | keep | — |

### 2.7 Typography

| ID | Reference | Current (`path:line`) | Exact gap | Fix seam | Pri |
|---|---|---|---|---|---|
| SH-Y1 | Sentence case for every control; uppercase only for mono meta and board-06's 9px panel labels (B06:1105,1109) | `.type-label` (uppercase + 0.08em) applied to tabs, all four buttons, scrubber label, replay badge (`index.css:70-77`) | Systematic: the chrome "shouts" where the reference speaks | Introduce `.type-control` (13px/600/normal case/0 tracking) and reserve `.type-label` for genuine labels | **P0** |
| SH-Y2 | Chrome uses 11 sizes: 8, 8.5, 9, 9.5, 10, 10.5, 11, 11.5, 12.5, 13, 17 (B07:1153-1250) | `TYPE_SCALE` has **8** steps: 11, 12, 13, 17, 24, 40, 56, 96 (`tokens.ts:91-100`; pinned by `tokens.test.ts:53`). 8 of the 11 chrome sizes are off-scale; components hardcode px (`LibraryPanel.tsx:30,37,56,65,88`; `Header.tsx:34,70,73`; `NavRail.tsx:48`; `HudBadge.tsx:16`) | No codified chrome ramp; the reference itself is off its own printed scale | Add a **separate** `CHROME_TYPE` export (do **not** touch `TYPE_SCALE`, test-pinned) + `--text-chrome-*` vars; migrate literals | P1 |
| SH-Y3 | tab label 12px **Medium** | `.type-label` weight 600 (`index.css:72`) | 100 weight | part of SH-Y1/SH-T7 | P1 |
| SH-Y4 | mono sizes differ per surface: HUD 10, chips 9.5, TIP 8.5, scrubber 9.5 | single `type-mono-meta` 11px used for all (`index.css:84-90`) | One size for five different jobs | resolved by SH-Y2 | P2 |

### 2.8 Colour

| ID | Reference | Current (`path:line`) | Exact gap | Fix seam | Pri |
|---|---|---|---|---|---|
| SH-K1 | `BORDER_GOLD #E5C27A` (B builder 41) used by HUD border, active tab, chip stroke, secondary button | **token does not exist**; substituted by `honey-400` / `honey-100` | 4 different surfaces use the wrong (nearest) hue | **Guard:** `SWATCHES` is pinned to exactly 20 (`tokens.test.ts:13-17`, `noHardcoded.test.ts:32-36`) — do **not** add a 21st swatch. Add `CHROME_TOKENS` (provenance = builder constant) + `--color-border-gold` in `tokens.css:8-27`. | **P0** |
| SH-K2 | `INK600 #6B5A33` (B builder 54) used for the template icon | `stroke="var(--color-ink-600)"` at `LibraryPanel.tsx:77` — **`--color-ink-600` is not defined in `tokens.css`** (grep-verified: referenced, never declared) | Broken token reference. Per CSS Variables semantics the declaration is invalid at computed-value time and `stroke` computes to `none`, so the 12×12 icon paints nothing while still reserving 12px + gap (**MECHANISM verified; rendered effect DERIVED — confirm in a browser**) | Define `--color-ink-600: #6b5a33` in `tokens.css` (as a chrome token), or switch to the existing `ink-500` | P1 |
| SH-K3 | honeycomb line `#EFE6CE` (B07:1198) / `HAIR_SOFT #EFE7D2` (builder 40) | `color: var(--color-honey-200)` = `#F6E7BE`, opacity .9 (`index.css:121-127`) | Noticeably more saturated/yellow than the reference lattice; no `HAIR_SOFT` token | Add `--color-hair-soft`; `index.css:125` → that token; drop the 0.9 opacity | P1 |
| SH-K4 | canvas label/footer `#A0916F` (B07:1248-1250); ghost labels `#A99B7C` (B07:1217) | not present (no canvas labels at all — SH-M…); values absent from the palette | Needs tokens or a documented snap to `ink-500` `#75663F` (which is darker) | add `--color-canvas-label` or document the substitution | P2 |
| SH-K5 | danger fill `#FFF7F5` (B06:1013) | absent; kill is transparent | Missing surface pair for the danger variant | add as chrome token when implementing SH-C4 | P1 |
| SH-K6 | `INK950 #14100A` (builder 50) | absent | unused in shell | defer | P2 |
| SH-K7 | — | `HUD`/`tab`/`chip` substitute honey-400/honey-100 | see SH-K1 | — | — |

### 2.9 Spacing and radii

| ID | Reference | Current (`path:line`) | Exact gap | Fix seam | Pri |
|---|---|---|---|---|---|
| SH-S1 | r7 (template rows), r11 (rail tile), r10 (buttons/TIP), r12 (minimap/scrubber/HUD-pill 999) | r7 ✓ r11 ✓ r10 ✓ r12 ✓ — but `RADII_STOPS` is `[4,8,12,16,24,48,999]` and `RADIUS_EXCEPTIONS` documents only `10` and `14` (`tokens.ts:107-111`; pinned by `tokens.test.ts:55`) | Two in-use radii (7, 11) are undeclared deviations from the printed scale | extend `RADIUS_EXCEPTIONS` (not `RADII_STOPS`) with `{px:7, source:'build_hive_style.py B07:1192'}`, `{px:11, source:'B07:1170'}` | P2 |
| SH-S2 | Off-grid values exist by design: tab gap **18**, chip pitch **58**, rail pitch **92** (B06:1055, B07:1184-1186, B07:1168) | current uses 4/8-multiples (`gap-1`, `gap-2`, `space-y-1.5`) | Chasing the 4px grid will *create* parity drift | Do not force these three to 4px; document them as intentional | P2 |
| SH-S3 | — | — | duplicate of SH-P5/SH-P6 | — | — |

### 2.10 Responsive

| ID | Reference | Current (`path:line`) | Exact gap | Fix seam | Pri |
|---|---|---|---|---|---|
| SH-Z1 | **no responsive spec exists** — one 1440×900 artboard; the style system ships no breakpoints | — | Not a parity gap; a decision the reference cannot answer | **Decision D7** | — |
| SH-Z2 | panel is a fixed structural element (never absent) | `hidden … xl:flex` (`LibraryPanel.tsx:26`) ⇒ below 1280 the project selector, hex chips, templates and TIP are **unreachable** (no drawer, no overflow) | Functional hole: template insertion is impossible <1280 | `:26` keep the panel and switch to an overlay drawer <1280, or collapse to icons; never remove without an alternative entry point | P1 |
| SH-Z3 | — | `ControlBar` is `flex-wrap` (`ControlBar.tsx:40`) ⇒ dock grows taller at narrow widths; `Scrubber` has no wrap strategy (`Scrubber.tsx:42`) | Bars reflow unpredictably; with SH-C1 fixed this becomes moot | resolve with SH-C1 (overlay dock is height-stable) | P2 |
| SH-Z4 | — | topbar = brand + 4 tabs + status pill + avatar in one non-wrapping flex row (`Header.tsx:20`) | **ESTIMATED**: below ≈900px these collide (no measured evidence) | resolve with SH-T1 (tabs leave the bar) | P2 |

### 2.11 Overlay layering (boundary row — internal toast design is out of scope)

| ID | Reference | Current (`path:line`) | Exact gap | Fix seam | Pri |
|---|---|---|---|---|---|
| SH-O1 | no overlay in board 07; board 06 toast is a 260×92 dark card | toast is `fixed right-4 top-4 z-50 w-[min(440px,92vw)]` (`ToastRegion.tsx:16`) ⇒ it floats **over** the topbar's right cluster | **MEASURED**: in the live capture the topbar's 1px bottom rule is visible only from x=0 to x≈975; the toast covers the rest of the bar's right side (avatar/name area) | `:16` `top-4` → `top-[64px]` (52 + 12 gutter) or render toasts inside the canvas region | P1 |
| SH-O2 | 260×92 dark `INK900`, white title, `H300` body, mono footer (B06:1136-1141) | light `bg-panel` + `border-signal`, up to 440px wide | Size/treatment differ | **Out of scope here** — owned by the toast/refusal doc; only the width cap and anchor are shell-level | P2 |

---

## 3. Prioritised work list

### P0 — composition-defining (the shell does not read as the reference without these)

1. **SH-T1** Remove the tablist from the 52px bar (the reference bar is empty between brand and right cluster — measured 0 ink pixels).
2. **SH-R1/R2/R3/R4** Rebuild rail anatomy: 92px pitch, 48×48 r11 active tile, label **outside** the tile, bare inactive items, `INK900` active label.
3. **SH-C1** Take the control bar and scrubber **out of layout flow** (recovers the measured 90px of canvas).
4. **SH-C2/C3** Buttons to 44px / 13px SemiBold sentence case.
5. **SH-C5** Scrubber to the 600×66 panel with a 552×4 track and a 15px ring thumb.
6. **SH-H2** Fix the HUD anchor (106px off centre) — or accept it explicitly as D4.
7. **SH-H3 + SH-K1** Add `BORDER_GOLD #E5C27A` as a **chrome** token (not a 21st swatch) and apply it to the HUD border.
8. **SH-M1/M2** Minimap inset to 100/50 and remove the grey mask veil; draw the `INK900` viewport outline.
9. **SH-Y1** Stop uppercasing interactive labels; add `.type-control`.

### P1 — high-visibility correctness

10. SH-T2/T3 Search + bell (or documented omission — D2).
11. SH-T5 Connection-status home (D3).
12. SH-T7 + SH-Y3 Tab treatment wherever tabs land.
13. SH-P1/Templates copy case ("Active Agents", "Templates").
14. SH-P2/P3/P4 Chip anatomy: pointy-top hex, gold stroke, state-tinted bee, 9.5px sentence-case label.
15. SH-P10/SH-Z2 Keep the panel reachable below 1280.
16. SH-H4/H5/H6 HUD dot 10px, 10px/6% type, delta below the pill.
17. SH-M3/M4 Minimap node language + shadow.
18. SH-C4 Button variant system; SH-C6 scrubber label/marks; SH-C7 playback controls (D6).
19. SH-K2 Fix undefined `--color-ink-600` (invisible template icon).
20. SH-K3 Honeycomb line token; SH-K5 danger fill.
21. SH-Y2 Codify the chrome type ramp.
22. SH-O1 Toast must not cover the topbar.

### P2 — polish and record-keeping

23. SH-T4 topbar name/chevron/inset.
24. SH-T6 bee brand mark.
25. SH-R5/R6/R7 rail rule, icon hue, bell-glyph collision.
26. SH-R8 rail inset.
27. SH-P5/P6/P7/P8/P9 chip pitch, template rows, chevron, TIP, details block.
28. SH-H7/H8 shadow recipe, HUD insets.
29. SH-M5 minimap lattice.
30. SH-C8/C9 select restyle.
31. SH-K4/K6 canvas-label and `INK950` tokens.
32. SH-S1/S2 radius exceptions + off-grid documentation.
33. SH-Z3/Z4 narrow-width reflow.

---

## 4. Decisions required (each blocks a row above)

| ID | Decision | Recommended default | Why it needs a human |
|---|---|---|---|
| **D1** | Where do the four workspace tabs live? | Left panel, per `SPEC:21-23` (the ASCII layout lists `[UI Tabs]` inside the left sidebar) and `SPEC:52-54` | Board 07's topbar has none, board 06 draws tabs only as a standalone specimen, and board 07's panel draws no tab strip ⇒ no authoritative placement exists. |
| **D2** | Search + bell: implement or omit? | Implement search over agents/templates if a target exists; otherwise **omit** and record the deviation | `src` has no search/notification domain. A decorative bell or a red dot with no data source would be a fake affordance, which the repo contract forbids. |
| **D3** | Connection-status pill has no reference home | Keep it, documented as a deliberate honesty deviation | Removing live connection state would hide `refusing`/`offline`, which the app must never do. |
| **D4** | HUD: literal 228px anchor or centred? | Literal `left: 228px` at ≥1440 | Centred is 106px right of the composition; literal breaks gracefully only if re-anchored below 1440. |
| **D5** | Rail "Agents" bell vs topbar notification bell | Keep the current distinct rail glyph | The reference uses the same bell for both roles. |
| **D6** | Scrubber playback controls (Play/Pause/Step-Back) | Build them (spec Prompt 10 wins) | Board 06 draws the scrubber without them ⇒ internal contradiction. |
| **D7** | Responsive strategy | Drawer below 1280; keep the rail; overlay the dock | The style system defines no breakpoints, so any choice is an addition, not parity. |

---

## 5. Non-goals (explicitly out of scope for this document)

- **Canvas nodes** — SuperAgent/WorkerAgent/ToolAgent/crimson error cards (`hivemind-agent-cards.png`, `LAYOUT.cards.*`). Consumed `hivemind-agent-cards.png` only to extract the **chip** anatomy (`chip = 40px hex, bee = state colour`) used in §2.3.
- **Chat core internals** — composer, bubbles, code block, pin/unpin. Board geometry (420×500 r18 at 620,196) is acknowledged but owned elsewhere.
- **Toast/refusal internal design** — dark-card treatment, copy, dismiss semantics. Only layering/width are shell-level (SH-O1/O2).
- **Approval overlay** and all non-Library panels' content.
- **State, API, contract, and command surface** — no new commands; nothing here implies backend work.
- **Making the live build current** — rebuilding `web/dist` was out of scope and would have written product artifacts.
- **A11y redesign** — a11y is an *invariant*, not a target: any relocation must preserve `role=tablist`/`tab`, `aria-current`, roving `tabIndex`, the skip link, and `aria-live` announcements.
- **Pixel-derived token values** — forbidden by `assets/manifest.ts:8-13`.

---

## 6. Verification plan (how each fix gets proven, not just claimed)

1. **Geometry** — after each P0, assert in `web/tests/designParity.test.ts` (already the enforcement seam): `LAYOUT` values plus source-usage assertions. Extend it with `HUD_ANCHOR = { left: 228, top: 16 }` and `MINIMAP_INSET = { right: 100, bottom: 50 }`.
2. **Rendered pixels** — capture the shell at 1440×900 (headless, `--window-size=1440,900`) and re-run the token-mask scan used here: topbar bottom edge, rail tile bbox, panel right edge, gold HUD bbox, minimap white bbox. Expected deltas: tile 48×48 at (12,68); HUD 460×44 at (540,68); minimap 220×150 with its right edge at 1340 and bottom at 850.
3. **No new hex** — `noHardcoded.test.ts` must stay green (all new colours go through `tokens.ts` / `tokens.css`).
4. **Token tests** — `tokens.test.ts` must stay green: `SWATCHES` stays 20, `TYPE_SCALE` stays 8, `RADII_STOPS` unchanged.
5. **Undefined-var sweep** — re-run the referenced-vs-declared scan; expect zero undefined `var(--*)` (currently one: `--color-ink-600`).
6. **Failure paths** — verify the offline/refusing banner still renders and still sits below the topbar (never under the toast), and that the HUD/minimap do not overlap the chat core at 1440×900.

---

## 7. What is already at parity (do not churn)

| Item | Evidence |
|---|---|
| Topbar height 52 + 1px hairline; `PANEL` fill | `Header.tsx:20`; MEASURED live 52 |
| Brand: 28px mark + 17px Bold wordmark | `Header.tsx:22-34` |
| Rail width 72, 6 destinations, 22px icons, 8px labels, `INK900` active tile, r11 | `NavRail.tsx:26,48-52`; `NavIcon.tsx:8` |
| Panel width 240 via `LAYOUT`, `PANEL_ALT`, 12px insets | `LibraryPanel.tsx:26-27`; `tokens.ts:169` |
| Project selector 216×32 r8, `PANEL` + `HAIR`, 11.5 Medium `INK800` | `LibraryPanel.tsx:29-31` |
| Panel headings 11.5 SemiBold `INK900` | `LibraryPanel.tsx:37,65` |
| Template rows r7, 11px text, `PANEL` + `HAIR`, 12px icon | `LibraryPanel.tsx:71,76-79` |
| TIP container `honey-100` + 10.5px body | `LibraryPanel.tsx:86,88` |
| HUD **460×44** pinned | `HudBadge.tsx:17`; `tokens.ts:171` |
| Minimap **220×150**, r12, `PANEL`, `HAIR` | `Canvas.tsx:379-384`; `tokens.ts:172` |
| Canvas `#F8F4EB` + vector honeycomb (never raster) | `index.css:17,121-127` |
| Token isolation (zero hex outside the token module) | `noHardcoded.test.ts:19-30` |
| A11y: tablist/tab, `aria-current`, roving tabindex, skip link, `aria-live` | `Header.tsx:37-52`; `NavRail.tsx:44-46`; `App.tsx:77-79` |

---

## 8. Reference conflicts recorded (not resolved here)

| # | Conflict | Resolution used |
|---|---|---|
| 1 | HUD 460×44 (B07:1230) vs 420×52 (B06:1060) | Board 07 wins (§0.3) |
| 2 | Minimap 220×150 (B07:1237) vs 260×170 (B06:1123) | Board 07 wins |
| 3 | Library 240 (B07:1178) vs 260 (B06:1100) | Board 07 wins |
| 4 | Rail pitch 92 (B07:1168) vs 88 (B06:1088) | Board 07 wins |
| 5 | Panel heading case: "Active Agents" 11.5 (B07:1183) vs "ACTIVE AGENTS" 9 uppercase (B06:1105) | Board 07 wins for the shell |
| 6 | Scrubber playback controls required by `SPEC:184-185` but absent from B06:1069-1078 | Escalated as D6 |
| 7 | Reference rail reuses the bell glyph for "Agents" and for notifications | Escalated as D5 |
| 8 | Reference chrome uses 8 sizes outside its own printed 8-step scale | Documented (SH-Y2), not "fixed" |
