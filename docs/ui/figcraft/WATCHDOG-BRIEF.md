# UI WATCHDOG BRIEF — read-only audit of FigCraft assets

## Mission
Independently audit every FigCraft asset from A and B against the PDF, the frozen UI contract, and the blueprint. You do
**not** edit assets, blueprint, source, or build code. You write a ledger and verdicts. You may BLOCK consumption by the
frontend worker.

## Inputs
- `C:\Users\Riley\Desktop\hivemind-blueprint\evidence\pdf\pages.txt` (authority; lines cited by designers).
- `blueprint/UI-WIRING-GUIDE.md` (28 surfaces, R-U1..R-U15, S01-S28) and `blueprint/UI-REQUIREMENTS.md` corrections.
- `docs/contracts/UI-COMMANDS-v1.md` (labels/refusals), `docs/ui/figcraft/BRIEF-A.md` / `BRIEF-B.md` (deliverable contract).
- `docs/ui/figcraft/A/A-ASSETS.json`, `B/B-ASSETS.json`, exports, `.figcraft` files, episode JSONs.
- Hashes: note that blueprint files must remain byte-identical to their recorded sha256 (try `Get-FileHash`).

## Checks (per asset)
1. Asset exists, non-zero, expected dimensions/format; `.figcraft` opens with `figcraft-cli info <file> --json` and lists
   the expected page(s).
2. **Token truth:** every colour hex on foundations matches the PDF 20 printed values (or a UI-REQUIREMENTS correction,
   labelled as corrected). Reject invented hexes, missing swatches, renamed tokens.
3. **Component presence:** every component in BRIEF-A §Required pages/artboards exists; every canvas element in BRIEF-B
   exists (chat core, 4 node types + error, 4 buttons, tabs, HUD, scrubber, nav rail, library, minimap, toast, settings,
   file tree).
4. **Value fidelity:** sizes (296/260/72/40/5/3 px), radii (4/8/12/16/24/48/999 + exceptions 10/14), fonts, type sizes,
   state dots/dot+shape cue, zoom 0.1/1.0/3.0, bee direction, minimap 260×170, scrubber text, HUD/counts labelled derived.
5. **Contract labels:** buttons carry the frozen command names (`SUBMIT_PROMPT`, `SPAWN_SUB_HIVE`, `REWIRE_TASK`,
   `CANCEL_RUN`, `KILL_WORKER`, `HISTORY_REVISION_SET`, `TEMPLATE_INSERT`); DIFF is labelled DECISION-GATED, not built;
   mutating controls show ACK/refusal states; refusal toast is "refused by name".
6. **Manifest truth:** each export is listed with surface id, page, sha256 and dimensions; no export missing from the
   manifest; no manifest row without a file; episodes exist and command ids were discovered (not fabricated).
7. **No blueprint/source modification:** check hashes; record.

## Verdicts
Per asset: `PASS` / `PASS-WITH-NOTES` / `FAIL` with evidence (`file:sha256`, PDF line, missing item). Overall gate:
`ALLOW-FRONTEND` only if A and B are PASS/PASS-WITH-NOTES and no FAIL. Record blocking items with exact fix request.

## Outputs
- `ledgers/ui-watchdog.md` (human table) and `ledgers/ui-watchdog.json` (`{generated_at, assets:[{file,sha256,verdict,
 issues}], overall, allow_frontend, blueprint_hashes_ok, watched_hashes}`).
- Read-only: do not write under `docs/ui/figcraft/{A,B,ASSETS}` or the blueprint/project code. Only the two ledger files.
- Return <=15 lines: overall verdict, per-artifact PASS/FAIL, blocking issues, whether frontend may consume assets.
