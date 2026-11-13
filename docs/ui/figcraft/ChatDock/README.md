# ChatDock — proposed FigCraft board for the new UI dock + Help

**Status: PROPOSED. Not watchdog-audited. Not in the served catalog.**

This directory contains the real FigCraft evidence for the new UI's full-width
chat dock and Help destination. It was generated with the installed
`figcraft-cli 0.17.0` after verifying its command schema. It is deliberately not
listed in `docs/ui/figcraft/ASSETS-INDEX.json` and not copied to
`web/public/figcraft/`: the served set is pinned to the 21 accepted boards by
`web/tests/figcraftAssets.test.ts`, and this board has no watchdog verdict.

## Artifacts

| file | role |
|---|---|
| `build/build_chatdock.py` | deterministic builder; writes `commands.json` |
| `build/commands.json` | 388 verified CLI commands (generated) |
| `build/run.log` / `build/run.err` | CLI run evidence |
| `build/publish_chatdock.py` | re-export + checks (12 artifact checks recorded in the manifest, then the manifest-write check) |
| `ChatDock-Help.figcraft` | the design file |
| `exports/chatdock-help.svg` | 1440×1800 SVG export |
| `exports/chatdock-help.png` | 1× PNG export |
| `exports/chatdock-help@2x.png` | 2× PNG export |
| `ChatDock-ASSETS.json` | hashes, evidence, status, source refs |
| `SPEC/CHATDOCK-TARGET-SPEC.md` | geometry + content spec + promotion checklist |

## Rebuild

```powershell
cd docs\ui\figcraft\ChatDock
python build\build_chatdock.py
& 'C:\Program Files\FigCraft\figcraft-cli.exe' run build\commands.json
python build\publish_chatdock.py
```

The publish step exits non-zero on any failed check. It writes
`ChatDock-ASSETS.json` only after the SVG/PNG exports, dimensions and font checks
pass.

## Board content

One 1440×1800 frame, two scenes:

- **13A — chat dock**: 52 header / 72 rail / 240 library / 1128 main column; the
  dock is 1128 × 560 at (312, 428), with a 64×12 grip, 420/560/760 height range,
  +240 tray, and the CONTROLS / STATE & SETTINGS / CLEAR / HIDE footer.
- **13B — Help**: Help is an active nav destination, with the three tutorials,
  ten feature explanations and the three current buttons.

Illustrative transcript/usage copy is labelled `ILLUSTRATIVE COPY · NOT RUN
DATA`. The board claims no runtime behaviour and no pixel fidelity.

## Honesty

- The board's face says `PROPOSED · NOT WATCHDOG-AUDITED · NOT SERVED`.
- `ChatDock-ASSETS.json` records `watchdog_audited: false`, `served: false` and
  the exact reason.
- The watchdog ledger (`ledgers/ui-watchdog.json`) has no entry for this SVG.
- Promotion requires a watchdog audit, an index entry, and an update to the
  served-set test; see the spec's checklist.
