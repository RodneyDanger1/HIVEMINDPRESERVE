# Live app evidence — HiveMind v1 browser QA (2026-11-06)

> **Dated correction — W9 trace-truth (2026-11-06T14:10:40-0600, HEAD `8a5b583`).** Recomputed from `docs/trace/controls.json`:
> **49 WIRED-AND-WORKS / 264 PENDING / 5 MISSING / 1 UNVERIFIED of 319 hops**; 55 hops carry evidence
> (non-PENDING), 54 evidence-backed excluding the UNVERIFIED marker. This supersedes the earlier
> `43 WIRED / 270 PENDING / 6 MISSING` figures (which counted a stale pre-fix `S22 6_service=MISSING` and did not
> include the 1.1 wave-2 evidence). Applied evidence: W3 streaming tests (S27 6_service/7_store/8_provider/9_response),
> W4 replay browser proof + source (S10 2_handler WIRED, 10_render_back UNVERIFIED), S22 6_service fixed from the
> tracked live probe. No hop was upgraded without a repo pointer; `delivered[]` stays empty (no fabricated frames).
> Recompute: `python -c "import json,collections;d=json.load(open('docs/trace/controls.json'));c=collections.Counter(h['verdict'] for x in d['controls'] for h in x['hops'].values());print(dict(c))"`




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
- Trace capture: `docs/trace/live/trace-capture.json` (refreshed 2026-11-06T15:24:24.056Z; 28 surfaces, 43 WIRED-AND-WORKS / 260 UNVERIFIED / 5 MISSING hops). **W9 recompute of `controls.json`: 49 WIRED-AND-WORKS / 264 PENDING / 5 MISSING / 1 UNVERIFIED of 319 hops (54 evidence-backed excluding the UNVERIFIED marker)**
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
`5 × MISSING` (S15-S19 node cards need a live run), `0 × WIRED-BUT-DEAD` — capture-level counts.
`controls.json` after W9 wave-2 evidence: **49 WIRED-AND-WORKS / 264 PENDING / 5 MISSING / 1 UNVERIFIED of 319 hops (54 evidence-backed excluding the UNVERIFIED marker)**; 23 controls `PARTIALLY_VERIFIED`, 6 `PENDING_APP_NOT_BUILT`.

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
