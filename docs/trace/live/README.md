# Live app evidence — HiveMind v1 browser QA (2026-11-06)

> **Dated correction 2026-11-06 (W8 docs-truth; HEAD `0dfe23b`).** This is the historical live-QA record from the pre-1.0.0 artifact (`184D51A7…` / asset `index-Cbfm1IuM.js`). Current 1.0.0 artifacts are server `b17eb3d6…062a9a` (13,253,120 B), CLI `49f46e2b…386623`, package `46305fcb…6105ef` (8,444,021 B); embedded assets are `index-6T84PJfk.js` / `index-BWOl__uM.css` (`web/dist/index.html` sha256 `8264ae92…8776d`). Current trace totals: **43 WIRED-AND-WORKS / 270 PENDING / 6 MISSING of 319 hops** (`docs/trace/controls.json`) — the `260 UNVERIFIED / 5 MISSING` counts below are historical. Live-provider evidence now exists (`docs/evidence/key-validation.md`, `docs/evidence/live-provider-smoke.md`, READ-level) and supersedes any "no paid run" phrasing. Historical run rows and asset names below are kept as dated evidence; current release record: `docs/RELEASE-EVIDENCE.md`.

Base URL: <http://127.0.0.1:8799> (embedded UI served by `tools/bin/hiverouter.exe`; final build
`index-Cbfm1IuM.js`, process PID 33580, binary sha256 `184D51A7C3CCA7B9D50954B677C5AD8A14472700A651FDF0D270441BE3EAC9A9`).
Harness: Playwright **1.65.0**, installed Chrome channel (no browser download), workers=1, no paid API.

## Final result — 0 failures

```powershell
cd e2e
$env:E2E_BASE_URL='http://127.0.0.1:8799'; $env:E2E_APP_READY='1'
npx playwright test --project=chrome --reporter=list,html
# => 93 tests: 79 passed, 0 failed, 14 skipped (exit 0, 3.6m)
```

- Run log: `e2e/test-results/run-live-FINAL-artifact.log`
- HTML report: `e2e/playwright-report/index.html`
- Screenshots/traces for failures: `e2e/test-results/` (no failures in the final run; earlier failure artifacts retained)
- Per-test evidence: `docs/trace/live/live-app/*.json`; boot/census screenshots `docs/trace/live/live-app/*.png`
- Trace capture: `docs/trace/live/trace-capture.json` (refreshed 2026-11-06T15:24:24.056Z; 28 surfaces, 43 WIRED-AND-WORKS / 260 UNVERIFIED / 5 MISSING hops)
- Controls: `docs/trace/controls.json` (evidence-backed verdicts applied by `tools/qa/apply-live-trace.mjs`)
- Static validator: `node tools/qa/verify-harness.mjs` → **27/27 PASS**

## What the final run proves (all with captured evidence)

| Area | Evidence file |
|---|---|
| Boot 200, exact title, `#root`, skip link `Skip to canvas` → `#hive-canvas` (target exists) | `live-app/01-boot.json`, `02-skip-link.json`, `boot.png` |
| No console errors / uncaught exceptions on load | `live-app/03-console.json` |
| Census: nav 6, tabs 4, chat core, composer, 4 control buttons, HUD (NULL shown as "no metrics"), scrubber disabled, library + 4 templates, minimap, 4 zoom controls | `live-app/04-census.json`, `surface-census.png` |
| Tabs + six nav views render-back; return to Workspace | `live-app/05-tabs.json`, `06-nav.json` |
| Zoom in / Ctrl+0 viewport transform changes; Fit safe on empty graph | `live-app/07-zoom.json` |
| CHAT_PIN → matching ACK → `aria-pressed` render-back (ACK-before-mutation) | `live-app/08-chat-pin.json` |
| Empty prompt → named `E_PROMPT_EMPTY`, **zero** SUBMIT_PROMPT frames (no fake success) | `live-app/09-empty-prompt.json` |
| WS: URL token = `/api/session` token, `last_seq` match, hello frame, server `SERVER_HELLO` | `live-app/10-ws-handshake.json` |
| Token colours: body `rgb(40,32,16)` on canvas `rgb(248,244,235)` → contrast **14.6677** ≥ 4.5 | `live-app/11-colours.json` |
| **SETTINGS_STATUS**: exactly 1 command → ACK + `SETTINGS_STATUS_RESULT` (provider/model/key_present) | `live-app/12-settings-status.json` |
| No stale Data/Approval contract-gap copy in the shipped bundle | `live-app/13-contract-drift.json` |
| **FILE_TREE_LIST/FILE_READ**: server entries `README.md`, `note.txt` render in the Data panel | `live-app/14-file-tree.json` |
| 28-surface trace capture: 23 surfaces present; S22 full client→route→response→render-back chain verified; S15-S19 node cards MISSING (no run started) | `trace-capture.json` |

### Trace verdict totals (from `trace-capture.json`)

`43 × WIRED-AND-WORKS`, `260 × UNVERIFIED` (server hops not provable from the browser / no paid run),
`5 × MISSING` (S15-S19 node cards need a live run), `0 × WIRED-BUT-DEAD`.
`controls.json`: 23 controls `PARTIALLY_VERIFIED`, 6 `PENDING_APP_NOT_BUILT`, 48 hop verdicts evidence-backed.

## History of this live pass (all defects found here were fixed and re-verified)

| Run | Server build | Result | Failures found (fixed later) |
|---|---|---|---|
| run-live-2 | binary 71B229B5… / asset DzTucYf3 (pre-fix) | 75 passed / 2 failed / 14 skipped | SETTINGS_STATUS `E_INTERNAL_ERROR` ×2; stale Data/Approval contract-gap copy |
| run-live-3 | binary DE676AFB… / asset DFBi-yZj | 76 passed / 1 failed / 14 skipped | nav test expected old Data copy; FILE_TREE_RESULT received but Data panel stayed LOADING (is_dir→kind decoder) |
| run-live-final2 | same | 76 passed / 2 failed / 14 skipped | transient trace-hops failure while server was being restarted |
| **run-live-FINAL-artifact** | **binary 184D51A7…C9A9 / asset Cbfm1IuM** | **79 passed / 0 failed / 14 skipped (exit 0, 3.6m)** | none — shipped artifact; Data unknown-size shows — (not 0 B) |

Earlier fixes verified by the final run: SETTINGS_STATUS ACK+result (and single command), Data/Approval copy updated,
FILE_TREE tree entries rendered, S22 trace chain complete.

## Shipped-artifact confirmation — Data unknown-size display
The shipped bundle (`index-Cbfm1IuM.js`) renders unknown entry sizes as the em dash, never `0 B`.
Captured from the final run (`live-app/15-data-size.json`): `README.md—` and `note.txt—` in the Data tree,
`"0 B"` absent from the DOM and from the shipped bundle; source fallback `web/src/panels/DataPanel.tsx:44`
(`entry.size_bytes === null ? "—"`). The live workspace entries carry no `size_bytes`, so this exercises the
unknown-size path directly.

## Skipped / UNVERIFIED (honest limits — not passes)

- `a11y-gates.spec.ts` 12 DOM gates: `test.fixme` (not implemented; static contrast math passes separately;
  live contrast/zoom/refusal/WS checks are in `live-app.spec.ts`).
- `ws-contract.spec.ts` 2 stubs (synthetic foreign-Origin refusal page; duplicate hello/ACK scenario): `test.fixme`.
- Node-card surfaces S15-S19 and streaming S27, terminal-run truth, provider/run lifecycle: UNVERIFIED because no
  paid run was started (no key/budget authorized). NULL metrics are shown honestly as "no metrics", never 0.
- G-UI-12 performance P95 frame time: not measured.

## Reproduce

```powershell
cd e2e
$env:E2E_BASE_URL='http://127.0.0.1:8799'; $env:E2E_APP_READY='1'
npx playwright test --project=chrome --reporter=list,html   # expects 79 passed / 0 failed / 14 skipped
node ../tools/qa/apply-live-trace.mjs                       # merge captured trace verdicts into controls.json
node ../tools/qa/verify-harness.mjs                          # 27/27 static contract checks
```
