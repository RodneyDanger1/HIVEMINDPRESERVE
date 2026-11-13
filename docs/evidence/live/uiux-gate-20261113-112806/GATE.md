# UIUX gate — bottom dock / Help / capability / named refusal

**Verdict: PASS (3/3 tests, 44 assertions) for the built artifact named below.**
Run at 2026-11-13T11:25:57-06:00 against `target/release/hiverouter.exe`
SHA-256 `DC9F68C7DAAF06D87442C902FAAEBEE2563DF981AB474DE91C38A2E9DEEE6B2C`
(embedded UI `/assets/index-CSbCvCgb.js`, git HEAD `e83a444`).
This is the built artifact, not a Vite dev server. Product files were not edited.

## Reproduce

```powershell
cd C:\Users\Riley\Desktop\hiverouter-v1
# Keyless, isolated server (refusal path must never reach a paid provider):
$env:DEEPSEEK_API_KEY = $null
Start-Process -FilePath "target\release\hiverouter.exe" `
  -ArgumentList '--bind','127.0.0.1:8100','--data-dir','docs\evidence\live\uiux-gate-<stamp>\data' `
  -WindowStyle Hidden
# (health must report deepseek_api_key.ok = false)
cd e2e
$env:E2E_BASE_URL='http://127.0.0.1:8100'
$env:E2E_APP_READY='1'
$env:UIUX_EVIDENCE_DIR='C:\Users\Riley\Desktop\hiverouter-v1\docs\evidence\live\uiux-gate-<stamp>'
node node_modules\playwright\cli.js test specs/uiux-gate.spec.ts --project=chrome --workers=1 --reporter=list
```

Installed Chrome channel only, `headless: true`, no browser download, no visible window,
no paid run. The refusal test asserts the STATE tray reports `key = missing` **before**
typing or submitting, so a key-present server can never start a run; zero `POST /api`
requests were observed across CLEAR and Dismiss.

## Observable results (from the JSON receipts)

| Requirement | Observed | Receipt |
|---|---|---|
| Dock x≈72 / right 1440 / bottom 900 at 1440x900 | `x=72, w=1368, right=1440, bottom=900` | `01-1440-dock-help.json` |
| Resize drag changes height | pointer drag +120px: `560 → 680`, bottom stays 900 | `02-1440-after-resize.png` |
| Resize persists | localStorage `hivemind.chat-height=680`; reload height 680, bottom 900 | `03-1440-after-reload.png` |
| CONTROLS expands, bottom fixed | `680 → 848` (clamped by `max-h-[calc(100vh-52px)]`=848), bottom 900, `aria-expanded=true` | `04-1440-controls.png` |
| STATE & SETTINGS expands, bottom fixed | dock 848, bottom 900, controls collapsed, provider row rendered | `05-1440-state.png` |
| Help page loads + tutorial text | h1 “What do you want to do?”; sections First run / The dock / Safety; dock resize tutorial copy; SCREEN ACCESS card | `06-help.png` |
| Refusal visible, named | toast `REFUSED BY NAME · NOT SILENT`, `E_KEY_MISSING`, keyless explanation | `07-refusal-visible.png` |
| CLEAR removes refusal | E_KEY_MISSING alert count 0; no POST | `08-after-clear.png` |
| Dismiss removes refusal | `Dismiss refusal` button → alert count 0 | `09`, `10` |
| 375 auto-height, no horizontal scroll, composer visible | computed height 557 (`max-h` cap; inline 560 overridden by `!h-auto`), doc/body scrollWidth 375, composer `right=230 < 375`; same with CONTROLS open; dock bottom 611 (nav 56) | `11-375-initial.png`, `12-375-controls.png` |
| Zero console/page errors | 0 / 0 in all 3 tests; 0 failed requests | `0*.json` |

## Artifacts

- `gate-summary.json` — verdict, command, artifact hash, per-test check counts.
- `01-1440-dock-help.json`, `02-refusal-clear-dismiss.json`, `03-375-mobile.json` — full
  steps + every check with measured geometry.
- `run.log` — final Playwright list output (3 passed).
- `01`–`12` PNGs as listed above.
- Attempt 1 (`../uiux-gate-20261113-111911/`) contains the calibration failures (tray
  clamp assertion, Dismiss accessible name, case-sensitive copy). These were test-side
  corrections, not product defects; the product checks (geometry, persistence, refusal
  removal) passed in that attempt too. That directory also holds the isolated server data.
- `FAIL-01-1440-dock-help.png` and `FAIL-02-refusal-clear-dismiss.png` in this directory
  are from the pre-fix runs of this same evidence dir (Help capability assertion used
  GuideOverlay wording); the final PASS artefacts are `01`–`12` and `0*.json`.

## Limits / UNVERIFIED

- **UNVERIFIED:** the currently dirty working tree (concurrent modifications to
  `ChatCore.tsx`, `HelpPanel.tsx`, etc. by other actors). This gate binds to the built
  binary/hash above; a rebuild invalidates it and the gate must be re-run.
- **UNVERIFIED:** WebKit/Firefox and a downloaded Chromium — only installed Chrome
  (channel) was used, per repo browser policy.
- Playwright's process exit was reported as `[exit code: 1]` by PowerShell because Node
  wrote a `NO_COLOR` warning to stderr; the test runner itself reported `3 passed (1.1m)`.
  `run.log` is the raw output. This is a shell-wrapper artifact, not a test failure.
- 375 height is `557px` = `max-h-[calc(100dvh-110px)]`, i.e. auto-height clamped by the
  mobile cap, not a fixed desktop height. That is the observed behaviour; no product edit
  was made.
