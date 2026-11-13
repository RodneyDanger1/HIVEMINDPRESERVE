# FigCraft evidence pipeline

The FigCraft boards in `docs/ui/figcraft/**` are the design evidence. The web
gallery consumes only the generated artifacts below; nothing under
`web/src/figcraft/catalog.ts` or `web/public/figcraft/` is hand-edited.

## Pipeline

```
A/A-ASSETS.json ─┐
B/B-ASSETS.json ─┼─> tools/build_assets_index.py ─> docs/ui/figcraft/ASSETS-INDEX.json
HiveMind/…json  ─┤        (+ ledgers/ui-watchdog.json, HiveMind verify report)
…                ┘                                      │
                                                        v
                         tools/build-figcraft-catalog.mjs
                                                        │
                    ┌───────────────────────────────────┼───────────────────────────┐
                    v                                   v                           v
   web/src/figcraft/catalog.ts       web/public/figcraft/<board>.svg       web/public/figcraft/SOURCES.json
```

Run from the repository root:

```powershell
python tools\build_assets_index.py
node tools\build-figcraft-catalog.mjs
node tools\build-figcraft-catalog.mjs --check   # CI / drift check; no writes
```

`build_assets_index.py` reads the accepted manifests and writes one board record
per served SVG with its accepted sha256 and audit status. It also appends the
HiveMind surface aliases and writes the source/design hashes. It never copies
pixels.

`build-figcraft-catalog.mjs` is the only writer of `catalog.ts`,
`web/public/figcraft/<board>.svg` and `SOURCES.json`. For every board it:

1. resolves the source SVG under `docs/ui/figcraft/` (path traversal rejected);
2. recomputes the sha256 and byte count and requires them to match the index;
3. cross-checks `watchdog-audited` boards against `ledgers/ui-watchdog.json`
   (verdict PASS/PASS-WITH-NOTES and the same sha256);
4. cross-checks `self-verified` boards against
   `HiveMind/build/verify-report.json` (status PASS and the same report sha256);
5. copies the source bytes only when the served copy differs, then re-hashes the
   copy;
6. removes stale `*.svg` files from `web/public/figcraft/` that are no longer in
   the served set.

## Audit status is data

Every board carries an `audit` object:

| status | meaning |
|---|---|
| `watchdog-audited` | A PASS/PASS-WITH-NOTES artifact in `ledgers/ui-watchdog.json`; sha256 matches the ledger. |
| `self-verified` | The owning set's own verify report passed. This is **not** a watchdog audit. |
| `proposed` | Marked PROPOSED-ADDITION or motion proposal by its own manifest/README. Not watchdog-audited. |
| `unverified` | No accepted audit evidence. |

Current served set: 21 boards = 8 watchdog-audited (A/B) + 8 self-verified
(HiveMind 00-07) + 5 proposed (HiveMind 08-12). The watchdog PASS verdict covers
A and B only; it is not a verdict on the HiveMind set.

## Proposed ChatDock board

`docs/ui/figcraft/ChatDock/` contains a real, CLI-generated proposal for the new
full-width chat dock + Help destination. It is deliberately **not** in
`ASSETS-INDEX.json` and **not** in `web/public/figcraft/`: it has no watchdog
verdict, and the served set is pinned to 21 boards by
`web/tests/figcraftAssets.test.ts`. See
`ChatDock/SPEC/CHATDOCK-TARGET-SPEC.md` for its geometry, evidence and promotion
checklist.

## Do not hand-edit generated files

- `web/src/figcraft/catalog.ts`
- `web/public/figcraft/SOURCES.json`
- `web/public/figcraft/*.svg`

Edit the source manifest or the generator/index builder instead, then rerun the
pipeline.
