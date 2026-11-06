# e2e — HiveMind v1 Playwright QA harness

**Status: LIVE — FINAL CHROME RUN 79 PASSED / 0 FAILED / 14 SKIPPED (exit 0).**
Playwright **1.65.0** installed in `e2e/` (package only, installed Chrome channel — no browser download).
Run against the **shipped binary** `184D51A7…C9A9` serving `index-Cbfm1IuM.js` at `http://127.0.0.1:8799`:
**93 tests = 79 passed, 0 failed, 14 skipped** (run log `test-results/run-live-FINAL-artifact.log`, HTML report
`playwright-report/index.html`, evidence `docs/trace/live/`). The 14 skips are explicit `test.fixme`/UNDONE
(12 a11y DOM gates, 2 WS stubs) — not passes. Node cards S15-S19, streaming S27 and terminal-run truth remain
UNVERIFIED because no paid run was started. Static harness (`tools/qa/verify-harness.mjs`) passes 27/27; trace
capture refreshed and applied to `docs/trace/controls.json`.

## Hard rules

1. **No browser download.** Use the installed Google Chrome / Microsoft Edge *channel*
   (`channel: 'chrome' | 'msedge'`). Never run `npx playwright install`; do not point
   `PLAYWRIGHT_BROWSERS_PATH` at a downloaded tree. NOTE: `%LOCALAPPDATA%\ms-playwright`
   already contains cached Chromium builds from unrelated tools — do **not** use them for e2e.
2. **No live paid API.** Never set `DEEPSEEK_API_KEY` for e2e. Provider paths are exercised with
   mock/wiremock only; a live call needs explicit user approval + budget.
3. **No fabricated evidence.** A skipped test is `SKIP`, not `PASS`. Trace data is filled only from
   captured requests, DOM state, DB rows or logs; UNVERIFIED stays UNVERIFIED.
4. **Frozen contract first.** Command/event/refusal names come from
   `docs/contracts/UI-COMMANDS-v1.md`; the matrix is `e2e/data/control-matrix.json`
   (28 surfaces from `blueprint/UI-WIRING-GUIDE.md` S01-S28 + supplemental `S-APPROVAL`).

## Prerequisites (verified on this host)

| Item | Value | Evidence |
|---|---|---|
| Node | v26.7.0 (>=22.12.0 floor) | `node --version` |
| npm | 11.19.0 | `cmd /c npm --version` |
| Playwright | **1.65.0** installed in `e2e/` (blueprint pin) | `node -e require('./node_modules/@playwright/test/package.json').version` |
| Browsers | installed Chrome 156.0.8078.75 / Edge 156.0.4314.58 via `channel` — no download | file versions |
| App target | shipped binary `tools/bin/hiverouter.exe` on `127.0.0.1:8799` (embedded UI) | `/health`, `/api/session`, `/diagnose` 200 |

Install (already done, package only):

```powershell
cd e2e
$env:PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = '1'
npm install
```

## Run

```powershell
cd e2e
$env:E2E_BASE_URL='http://127.0.0.1:8799'; $env:E2E_APP_READY='1'
npx playwright test --project=chrome --reporter=list,html
# expects: 93 tests = 79 passed, 0 failed, 14 skipped (exit 0)
```

No `webServer` is configured: the release binary is started by the build/Lead, not by Playwright.

## Specs

| File | Covers | Result on shipped binary |
|---|---|---|
| `specs/live-app.spec.ts` | boot/root/title/skip link, surface census (nav 6, tabs 4, chat, composer, 4 controls, HUD, scrubber, library, minimap, zoom), tabs + 6 nav views render-back, zoom/Ctrl+0, CHAT_PIN ACK-before-mutation, empty-prompt refusal (no SUBMIT_PROMPT), WS hello token, token colours/contrast, SETTINGS_STATUS ACK+result, Data/Approval copy, FILE_TREE tree render, unknown-size `—` (never `0 B`) | 15/15 pass |
| `specs/trace-hops.spec.ts` | real 28-surface 11-hop capture → `docs/trace/live/trace-capture.json` | 28/28 pass |
| `specs/control-matrix.spec.ts` | matrix integrity + 29 boot probes | pass |
| `specs/a11y-gates.spec.ts` | static contrast math (real) + 12 DOM gates | 2 pass, 12 fixme |
| `specs/ws-contract.spec.ts` | health/session (real) + 2 stubs | 1 pass, 2 fixme |

## Trace evidence output

- `docs/trace/live/trace-capture.json` — refreshed from the shipped-artifact run (28 surfaces; 43 WIRED-AND-WORKS /
  260 UNVERIFIED / 5 MISSING hops; 23 surfaces present).
- `docs/trace/controls.json` — evidence-backed verdicts applied (`node ../tools/qa/apply-live-trace.mjs`);
  23 `PARTIALLY_VERIFIED`, 6 `PENDING_APP_NOT_BUILT`, 48 evidence-backed hop verdicts.
- `docs/trace/live/live-app/*.json` — per-test evidence (incl. `15-data-size.json`: `README.md—`, `note.txt—`,
  `"0 B"` absent, shipped bundle has the `—` fallback).
- `docs/trace/live/README.md` — full evidence summary + run history.

## Known UNVERIFIED / open (honest)

- Node cards S15-S19, streaming S27, terminal-run truth, provider/run lifecycle: no paid run started.
- G-UI-1..12 DOM gates (12 `test.fixme`) and synthetic foreign-Origin refusal / duplicate hello-ACK stubs (2 fixme).
- FigCraft canonical manifest `ASSETS-INDEX.json` still absent; `A/A-ASSETS.json` candidate needs Lead acceptance.
