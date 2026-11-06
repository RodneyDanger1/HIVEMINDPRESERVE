# FIGCRAFT-A BRIEF — foundations, brand, component library

## Mission
Produce the design-system truth for the **new** HiveMind v1 UI from the supplied FigCraft PDF artboards, using the headless
FigCraft CLI. You are a design producer, not an app coder. Your assets are consumed by the frontend worker only after the
UI Watchdog accepts them.

## Tools & protocol (craft-agent-operator)
- Binary: `C:\Program Files\FigCraft\figcraft-cli.exe` (do NOT launch the GUI).
- Drive mode: `figcraft-cli run <script.json> --out <file.figcraft>` (transactional; any bad param rolls back the whole
  script), then `figcraft-cli export <file> -o <out> --format svg|png`.
- Discover commands: `figcraft-cli commands --json` (597 commands) and the atlas skill files under
  `C:\Users\Riley\.dsh\skills\artcraft-command-atlas\references\` (vc-color-style.md, vc-file-document-export.md,
  vc-layers-artboards.md via command_lookup). Never invent a command id or parameter.
- Every run: write an episode JSON in `docs/ui/figcraft/EPISODES/` (brief, command ids used, script path, output file,
  sha256, dimensions, export paths, failures). Work on new files only; never overwrite an accepted asset.
- If a command is rejected, read `figcraft-cli commands --filter <name> --json` and fix the schema; do not guess twice.

## Inputs (read these)
- `C:\Users\Riley\Desktop\hivemind-blueprint\evidence\pdf\pages.txt` — lines 2-273 are your authority (cover, foundations,
  type, spacing/radius/elevation, brand, components p6, controls p7, canvas p8).
- `C:\Users\Riley\Desktop\hivemind-blueprint\blueprint\UI-WIRING-GUIDE.md` — §0 rules, §1 surfaces S01-S28, §5 conflicts.
- `docs/contracts/UI-COMMANDS-v1.md` — frozen labels for buttons/commands.
- `reports/ui/UI-REQUIREMENTS.md` corrections (contrast fixes, meta/focus replacements, radii exceptions).

## Required pages/artboards
1. **Page "00 Foundations"** (1440×1200): 20 printed swatches with hex + token names (Canvas, Paper, Panel Alt, Recess,
   Hairline, Honey 100-800, Ink 900/800/700/500, Amber, Signal, Verify); type scale rows (display XXL 96/100/-3%,
   display L 56/60/-2%, h1 40/48/-1.5%, h3 24/32/-0.5%, body L 17/28/0, label 12/16/+8%, mono code 13/22/0,
   mono meta 11/16/+8%); font families (Playfair Display SemiBold, Inter SemiBold, Consolas Regular; note
   Instrument Serif self-hosted in product); spacing 4,8,12,16,20,24,32,40,48,64,80,96; radii 4,8,12,16,24,48,999
   (+ exceptions 10 button / 14 super card recorded as exceptions); elevation e1/e2/e3 (same printed value — label as
   "printed value unresolved, do not differentiate without decision").
2. **Page "01 Brand"** (1440×900): one mark / four lockups (primary, stacked, inverse, icon), clear space 0.25× mark
   height, bee glyph anatomy (head/thorax/abdomen/wings/antennae/stinger) + min sizes 56/32/16 px, <24 px drop inner bee,
   wordmark min 20 px, one-colour mono fallback. The bee must point in the data-flow direction.
3. **Page "02 Components"** (1440×~2600): SuperAgent sub-hive card (296px wide, light honey header, 1.5px honey-gold
   border, radius 14 exception, run-state pill, "decomposes intent", "3 workers · 12.4k ctx", 8px grid); WorkerAgent card
   (no header fill, 5px progress track + 3px worker bar, mono 9.5px meta — use corrected meta colour `#75663F` not
   `#8A7A52`); ToolAgent card (dark `#282010` syntax box, gold `#8A5A1C` execution bar, `schema.prisma · 0.78`);
   broken-route error card (crimson border `#B23A2E`, dashed crimson wire, halted bee + warning glyph, "Compile failed /
   db-schema · E0421 · retry 2/3"); HyperAgent pinned hive chat core (beehive containment, off-white `#FBF3DC` input
   surface, composer row, `×` close); agent chip (40px hex, bee state colour) + four state dots + text/shape cue;
   buttons Spawn sub-hive / Re-route / Cancel / Kill worker (40px min target, radius 10, Inter SemiBold 13) each with
   default/hover/pressed/disabled/refused states; tabs (Workspace/Swarm/Routing/History); HUD throughput badge
   (4 SUPERAGENTS · 12 WORKERS · 98.2% GREEN +3.4%); time-travel scrubber (09:41:07 · routing state 12 of 38);
   nav rail 72px six items (Workspace/Agents/Data/Models/Routing/Settings); library panel 260px (Project: HiveMind AI,
   Active Agents, Templates ×4, TIP); minimap 260×170; refusal toast (Re-route refused / context-prune blocked: worker
   still streaming / REFUSED BY NAME · NOT SILENT); Settings key status; file/code tree (router.ts, schema.prisma,
   agent.spec.ts, main.tsx, beam.rs).
4. **Page "03 Specimens"**: selected/hover/focus(replacement ink-700 ring ≥3:1)/disabled/empty/loading/offline/error/
   awaiting-approval/streaming/reduced-motion variants.

## Outputs
- `docs/ui/figcraft/A/scripts/*.json`; `docs/ui/figcraft/A/A-design.figcraft`.
- Exports to `docs/ui/figcraft/ASSETS/`: `a-foundations.svg/png`, `a-brand.svg/png`, `a-components.svg/png`,
  `a-specimens.svg/png` (each named exactly; use `--scale 1` for SVG, `--scale 2` for PNG).
- `docs/ui/figcraft/A/A-ASSETS.json`: array of `{surface_id, name, page, node_id, export_svg, export_png, sha256_svg,
  sha256_png, dimensions, provenance_pdf_lines}` covering S01-S28 mapping where applicable.
- `docs/ui/figcraft/A/A-EPISODES.json` or individual episode JSONs.

## Acceptance (self-check before returning)
- `figcraft-cli info docs/ui/figcraft/A/A-design.figcraft --json` parses; every listed page/artboard exists.
- Exports are non-zero, dimensions recorded; token hexes match PDF exactly (or are corrected per UI-REQUIREMENTS and
  labelled as corrected); no invented token/colour/size.
- Manifest covers every component; watchdog audit is the authority, not your self-check.
- Return <=20 lines: files created, pages, export list, command count, any schema guesses rejected, open issues.
