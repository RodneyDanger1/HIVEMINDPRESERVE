# HiveMind FigCraft design set (authoritative spatial style system)

Source: `HiveRouter/design/hive-style` (external repo, read-only; copied evidence in
`source-evidence/`). User-facing HiveRouter/HIVEROUTER text and file names are renamed
to HiveMind; A/B under `docs/ui/figcraft/A|B` are unchanged.

## Build (deterministic FigCraft CLI, two transactional phases)
```
cd docs/ui/figcraft/HiveMind/build
python build_hive_style.py 1
figcraft-cli run commands_phase1.json --out base.figcraft
python build_hive_style.py 2
figcraft-cli run commands_phase2.json --in base.figcraft
python publish_hivemind.py
python verify_hivemind.py
```
`build_hive_style.py` is the source builder patched for HiveMind (rename
+ HMAS/state boards 08-10 + motion board 11 + exact-anatomy refinement of
board 05). `HiveMind-Style-System.figcraft` is the regenerated design file;
exports live in `exports/` (generated) and the canonical copies in
`docs/ui/figcraft/ASSETS/hivemind-*.{png,svg}`.

## Artboards
| # | frame | board size | export |
|---|---|---|---|
| 00 | 00 · Cover | 1440x900 | `hivemind-cover.png` |
| 01 | 01 · Colour | 1440x1240 | `hivemind-colour.png` |
| 02 | 02 · Typography | 1440x1340 | `hivemind-typography.png` |
| 03 | 03 · Space & Depth | 1440x1000 | `hivemind-space-and-depth.png` |
| 04 | 04 · Brand Lockups | 1440x1000 | `hivemind-brand-lockups.png` |
| 05 | 05 · Agent Cards | 1440x1300 | `hivemind-agent-cards.png` |
| 06 | 06 · Controls & HUD | 1440x1040 | `hivemind-controls-and-hud.png` |
| 07 | 07 · Spatial Canvas | 1440x900 | `hivemind-spatial-canvas.png` |
| 08 | 08 · HMAS Hierarchy | 1440x900 | `hivemind-hmas-hierarchy.png` |
| 09 | 09 · HMAS Governance | 1440x900 | `hivemind-hmas-governance.png` |
| 10 | 10 · States & Refusals | 1440x900 | `hivemind-states-and-refusals.png` |
| 11 | 11 · Motion & Live | 1440x900 | `hivemind-motion-and-live.png` |

## Provenance / claims
- Source style kit + target spec + original exports are preserved byte-identically
  under `source-evidence/` (see `HiveMind-SOURCE-EVIDENCE.json`).
- Episode command scripts and their hashes are in `HiveMind-EPISODES.json`
  and `episodes/` (phase 1 nodes, phase 2 styles/variables/exports).
- Token palette is the source's documented palette (no invented swatches); the
  regenerated artboards reuse it.
- HMAS and state/refusal boards are PROPOSED-ADDITION surfaces on top of the
  authoritative style system; board 07 keeps the authoritative composition
  (52px topbar, 72px rail, 240px library, canvas x=312, chat 420x500 at 620,196,
  HUD 460x44, minimap 220x150, SuperAgent 420x150, WorkerAgent 320x96,
  ToolAgent 420x240, crimson error card).
- Board 05 refines the exact card anatomy annotations (Super 420x150 header 40 +
  68x18 pill; Worker 320x96; Tool 420x240 with 132px dark result area; Error 320x110).
- Board 11 motion/live is a design proposal: durations/easing are PROPOSED, not
  measured; no component behaviour is frozen. It shows the current app chat variant
  400x480 while board 07 remains the authoritative 420x500.
