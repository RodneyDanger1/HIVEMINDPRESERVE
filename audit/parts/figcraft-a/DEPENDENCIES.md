# figcraft-a - DEPENDENCIES.md

Partition `docs/ui/**` + `ledgers/**` (103 tracked files) is a design-asset partition: the graph is build/registry lineage, not a call graph.

## 1. FigCraft-A lineage (hash-verified)
```
BRIEF-A.md
 -> A/build/*.py (fig.py driver, gen_common.py board/token primitives, pages_*.py layouts,
                 gen_build.py orchestrator, gen_manifest.py manifest/episode emitter)
 -> A/scripts/00_setup_pages ... 10_specimens_groups  (11 transactional scripts, 1444 commands, 19 distinct command ids; all valid in the 597-row catalogue)
 -> figcraft-cli run (transactional; any bad param rolls back)
 -> A/build/page00g..page03g.figcraft + styles.figcraft (intermediates)
 -> A/A-design.figcraft (4 pages, 28 surfaces, 20 colour variables, 31 styles, missingFonts=[])
      ├─ export --node -> ASSETS/a-{foundations,brand,components,specimens}.{svg(scale1),png(scale2)}
      ├─ A/A-ASSETS.json     28 rows: surface -> page/node/sha/dims/PDF lines (document_sha256 = design sha)
      ├─ A/A-EPISODES.json   12 episodes + command ids + generator source hashes
      └─ A/A-SELF-CHECK.json info dump + 20-token hex audit + style count + SVG hex audit + node checks
 -> docs/ui/figcraft/ASSETS-INDEX.json (Lead alias; keeps A sha256 per asset)
 -> ledgers/ui-watchdog.json|md (20 artifacts, PASS, allow_frontend=true)
```

## 2. FigCraft-B lineage
```
BRIEF-B.md -> B/build/{build_b,fig_b,boards_common,board_*}.py -> B/scripts/{00..06}.json (690 commands, 15 ids)
 -> B/B-canvas.figcraft (4 pages x 1440x900; 27 surfaces)
 -> ASSETS/b-{canvas-default,zoom-states,interaction,hero}.{svg,png} -> B/B-ASSETS.json (27 surfaces, pending=[])
 -> ASSETS-INDEX.json (B asset sha NOT carried - F2) -> ui-watchdog ledger (recomputed hashes PASS)
```
B manifest claims command_count=690, stages=7; measured from scripts: 690 commands across 7 scripts; invalid ids: none.

## 3. Cross-partition links
| dependency | owner | use |
|---|---|---|
| `docs/contracts/UI-COMMANDS-v1.md` | figcraft-b seat | frozen command names + E_* refusal codes printed on boards |
| `hivemind-blueprint/blueprint/UI-WIRING-GUIDE.md` | external blueprint | S01-S28 surfaces + PDF-line provenance |
| `hivemind-blueprint/reports/ui/UI-REQUIREMENTS.md` | external blueprint | corrections: meta #75663F, focus ink-700, amber large-only |
| `hivemind-blueprint/evidence/pdf/pages.txt` | external blueprint | printed hexes, type scale, component copy |
| `audit/WAVE-SHEET.md`, `audit/inventory/*` | Lead | assignment + manifest sha source |
| `ledgers/ui-watchdog.json|md` | this partition | PASS gate consumed by frontend |

No A<->B imports; both consume the same external blueprint/contract and register into one index. Root `.gitignore:7,9` excludes the 18 generated `.figcraft`/`.svg`/`.png` products from git (F1).

## 4. External tools
| tool | version/evidence | use |
|---|---|---|
| figcraft-cli.exe | 0.11.2; `commands --json`=597 | run/export/info (headless; GUI never launched) |
| FigCraft native format | parsed via `info --json` for all 18 tracked .figcraft | design docs + intermediates |
| Python | 3.12.14 audit env; B pyc tag cpython-314 | A generator drivers / B board builders |
| Pillow | 12.3.0 | PNG dimension evidence |
