# FIGCRAFT-B BRIEF — spatial canvas, interaction states, composition

## Mission
Produce the spatial-canvas design truth for HiveMind v1 (the proposed product) with the headless FigCraft CLI: full
composition artboards plus interaction/state variants. Not app code; assets only.

## Tools & protocol (craft-agent-operator)
Same as BRIEF-A: `C:\Program Files\FigCraft\figcraft-cli.exe`; `run` transactional scripts; `export`; `commands --json`
(597) + atlas references; episode per run in `docs/ui/figcraft/EPISODES/`; no invented command ids; new files only.

## Inputs
- `evidence/pdf/pages.txt` lines 147-273 (brand, components, controls, canvas) and lines 2-14/274-305 (cover/hero).
- `blueprint/UI-WIRING-GUIDE.md` §0-§2 (S01 canvas, S13 minimap, S20 zoom, S10 scrubber, S19 state dots, S25 gated) and
  `docs/contracts/UI-COMMANDS-v1.md`.
- A's accepted outputs (do not modify; if A is still running, use the PDF values directly and note dependency).

## Required artboards
1. **"Canvas / default" 1440×900**: warm canvas `#F8F4EB` with a soft honeycomb lattice (one vector path with many
   subpaths; honey line tone from tokens); core hive/chat at centre (beehive containment); sub-hive left (Backend
   Architecture, RUNNING, 3 workers · 12.4k ctx); worker card below-right (tailwind-builder, WORKER · react/vite, 55%);
   tool card (schema.prisma · 0.78) with dark syntax box; broken-route card with dashed crimson wire near right; bee-line
   edges with direction arrow (bee) from parent→child; minimap 260×170 bottom-right with viewport rect; time-travel
   scrubber bottom-centre ("09:41:07 · routing state 12 of 38"); HUD top-centre ("4 SUPERAGENTS · 12 WORKERS · 98.2%
   GREEN +3.4%"); nav rail 72px left; library panel 260px left; Ctrl+0 recentres hint text.
2. **"Canvas / zoom states"**: same composition at 0.1× (overview, LOD labels only), 1.0×, 3.0× (detail tokens/meta,
   node cards readable) on one 1440×900 board with 3 frames labelled.
3. **"Canvas / interaction states"**: selected node (ink-700 focus ring ≥3:1), dragging ghost, edge re-route preview,
   reconnect banner, awaiting-approval card overlay, refusal toast ("Re-route refused / context-prune blocked: worker
   still streaming / REFUSED BY NAME · NOT SILENT"), kill-worker confirm, time-travel replay.
4. **"Brand hero" 1440×900** (optional): cover-like statement using the brand lockups + one hero canvas fragment.

## Outputs
- `docs/ui/figcraft/B/scripts/*.json`; `docs/ui/figcraft/B/B-canvas.figcraft`.
- Exports to `docs/ui/figcraft/ASSETS/`: `b-canvas-default.svg/png`, `b-zoom-states.svg/png`, `b-interaction.svg/png`,
  `b-hero.svg/png` (SVG + PNG @2×).
- `docs/ui/figcraft/B/B-ASSETS.json`: `{surface_id, name, page, node_id, exports, sha256s, dimensions, pdf_lines}` for
  S01/S10/S13/S19/S20/S24/S25.
- Episodes in `docs/ui/figcraft/EPISODES/`.

## Acceptance (self-check)
- `info --json` parses; pages/exports exist and are non-zero; explicit zoom values 0.1/1.0/3.0 shown; bee direction
  source→target; no invented tokens; decision-gated code-diff not drawn as implemented (label DECISION-GATED if shown).
- Return <=20 lines: files, pages, exports, command count, rejected guesses, open issues.
