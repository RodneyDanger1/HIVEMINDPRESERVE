# Chat dock / Help — FigCraft proposal spec (board 13)

**Status: PROPOSED — not watchdog-audited, not served.**

This board documents the *new UI* chat dock and the Help destination. It is real
FigCraft evidence produced by the installed CLI, but it is not accepted design
truth: no watchdog ledger entry exists for it, and the served catalog stays at the
21 accepted boards until this board is audited and accepted.

| item | value |
|---|---|
| board id | `chatdock-help` |
| frame | `13 · Chat Dock & Help` |
| size | 1440 × 1800 (one frame, two scenes) |
| design file | `docs/ui/figcraft/ChatDock/ChatDock-Help.figcraft` |
| SVG export | `docs/ui/figcraft/ChatDock/exports/chatdock-help.svg` |
| PNG exports | `.../chatdock-help.png`, `.../chatdock-help@2x.png` |
| manifest / hashes | `docs/ui/figcraft/ChatDock/ChatDock-ASSETS.json` |
| audit status | `PROPOSED`, `watchdog_audited: false`, `served: false` |
| CLI | `figcraft-cli 0.17.0` at `C:\Program Files\FigCraft\figcraft-cli.exe` |

## Why the board exists

Commit `e83a444` replaced the old 420×500 pinned chat card with a full-width
bottom dock and added a Help destination. The accepted FigCraft sets do not show
that UI:

- HiveMind board 07 still pins the authoritative 420×500 chat card;
- board 11 shows the older 400×480 app variant;
- the Help destination is not on any accepted board.

This proposal shows the actual current geometry and content so the FigCraft
gallery can represent the new UI once the board is accepted. It does not modify
the accepted boards or claim they already show it.

## Verified CLI schema

Schema was read from the CLI itself, not guessed:

- `figcraft-cli --version` → `figcraft-cli 0.17.0`
- `figcraft-cli commands --json` → 643 command records.
- `node.create` params: `{type, x?, y?, width?, height?, parent?, index?, name?, text? | characters?, path? | points?, closed?, props?, select?}`; paint/geometry keys such as `fill`, `cornerRadius`, `stroke`, `effects`, `opacity` and `dashes` belong inside `props`.
- `text.create` params: `{text|characters, x, y, parent?, index?, name?, width?, props?}` with `fontSize`, `fontFamily`, `fontStyle`, `fill`, `lineHeight`, `letterSpacing`, `textAlign`, `textAutoResize` inside `props`.
- `frame.create` params: `{preset?, x?, y?, width?, height?, parent?, index?, name?, props?}`.
- `file.new` params: `{name?}`; `file.saveAs` params: `{path | data: true}`.
- The one-shot export is `figcraft-cli export <file> -o <out> --format svg|png --node <frame> --scale <n>`.

A smoke script (frame + rectangle + text + SVG/PNG export) was executed first;
the first rejected attempt put `fill`/`cornerRadius` at the top level of
`node.create`, which the CLI rejected transactionally. The working form uses
`props`. The shipped command script uses only the five verified command ids:
`file.new`, `frame.create`, `node.create`, `text.create`, `file.saveAs`.

## Geometry (read from the current UI source, not sampled pixels)

| value | value used | source |
|---|---|---|
| header | 52 px | `web/src/tokens/tokens.ts` `LAYOUT.topbar` |
| nav rail | 72 px, 8 destinations incl. Help | `LAYOUT.navRail`, `web/src/panels/NavRail.tsx` |
| library rail | 240 px | `LAYOUT.library` |
| main column at 1440 | 1128 px | 1440 − 72 − 240 |
| dock width | 1128 px (`w-full` of the main column) | `web/src/chat/ChatCore.tsx` dock section |
| dock default height | 560 px | `web/src/chat/chatDock.ts` `DEFAULT_CHAT_HEIGHT` |
| dock floor / ceiling | 420 / 760 px | `MIN_CHAT_HEIGHT` / `MAX_CHAT_HEIGHT` |
| tray delta | +240 px | `DOCK_TRAY_HEIGHT` |
| drag grip | 64 × 12 px | `h-3 w-16` on the dock grip |
| dock corner radius | 18 px top | `rounded-t-[18px]` dock class |
| Help content width | 980 px max | `web/src/panels/HelpPanel.tsx` |

Dock footer controls shown: `CONTROLS`, `STATE & SETTINGS`, `CLEAR`, `HIDE`;
the refusal example is labelled by name (`E_CALL_LIMIT`) and marked
`ILLUSTRATIVE COPY · NOT RUN DATA` on the board.

## Scene A — chat dock (y 96–996)

- 52 px header, 72 px rail, 240 px library, 1128 px main column.
- Honeycomb canvas, HUD badge, two illustrative agent cards above the dock.
- The dock spans the full 1128 px main column at x=312, y=428, h=560.
- Anatomy callouts: grip size, tray delta, and the bottom-fixed resize rule.
- The transcript, usage values and worker progress are illustrative design copy,
  not captured run data.

## Scene B — Help (y 1024–1800)

- Same shell; the Help nav-rail item is active (Help is a nav destination, not a
  dock tab).
- Header: `HELP & TUTORIALS`, `What do you want to do?`, the current explanatory
  paragraph.
- Three tutorial cards: First run, The dock, Safety.
- Ten feature explanations, matching the current `FEATURES` array, including the
  source's two features both labelled `Workspace`.
- Three buttons: Go to Workspace, Provider & key settings, Agent roster & jobs.
- A notes band records the source commit, the static-copy limit, the duplicate
  `Workspace` label, and the proposed status.

## Build and verify

```powershell
cd docs\ui\figcraft\ChatDock
python build\build_chatdock.py
& 'C:\Program Files\FigCraft\figcraft-cli.exe' run build\commands.json
python build\publish_chatdock.py
```

`publish_chatdock.py` re-exports SVG/PNG, checks the frame name/size, missing
fonts, PNG dimensions, the SVG canvas size, the command whitelist and the absence
of `HiveRouter`, then writes `ChatDock-ASSETS.json`. Exit 0 means all checks pass.
Hashes and byte counts live in that manifest, so this spec does not carry stale
copies.

## Promotion checklist (why it is not served today)

1. An independent watchdog audit must add this SVG to `ledgers/ui-watchdog.json`
   with a PASS/PASS-WITH-NOTES verdict and a matching sha256.
2. `tools/build_assets_index.py` must add the board with
   `audit.status: "watchdog-audited"`; the catalog generator then derives it.
3. `web/tests/figcraftAssets.test.ts` currently pins the served set to exactly 21
   SVGs and 22 public files; it must be updated to the new count.
4. Run `node tools/build-figcraft-catalog.mjs --check` and the web test suite.

Until 1–4 are done, the board remains a proposal. It must not be described as an
accepted, watchdog-audited, or served board.
