# Live app evidence — first live run (2026-11-06)

Base URL: **http://127.0.0.1:8799** (embedded UI served by `tools/bin/hiverouter.exe`).
Harness: Playwright **1.65.0**, installed Chrome channel (no browser download), workers=1.
Command:

```powershell
$env:E2E_BASE_URL='http://127.0.0.1:8799'; $env:E2E_APP_READY='1'
npx playwright test --project=chrome --reporter=list,html
```

## Result (current server build)

**91 tests: 75 passed, 2 failed, 14 skipped** (run log `e2e/test-results/run-live-2.log`;
HTML report `e2e/playwright-report/index.html`; screenshots/traces under `e2e/test-results/`).

Served bundle: `assets/index-DzTucYf3.js` (built 07:42:45); binary sha256
`71B229B587A909A6D8C2BDB9B8978B6087CECBF0609B176A9D46CA69717C38FC` (built 07:44:57), process started 07:45:18.
Frontend source was still being fixed at 07:56–07:57, so this run is against the **pre-fix bundle**.

### What passed (evidence-backed)

| Area | Evidence |
|---|---|
| Boot 200, exact document title, `#root`, skip link `Skip to canvas` → `#hive-canvas` (target exists) | `live-app/01-boot.json`, `02-skip-link.json`, `boot.png` |
| No console errors / uncaught exceptions on load (favicon is a data URI in this bundle) | `live-app/03-console.json` |
| Chrome census: nav 6, tabs 4, chat core, composer, 4 control buttons, HUD (no-metrics = NULL not 0), scrubber disabled, library + 4 templates, minimap, 4 zoom controls | `live-app/04-census.json`, `surface-census.png` |
| Tab switching + render-back; nav rail six views + return | `live-app/05-tabs.json`, `06-nav.json` |
| Zoom in / Ctrl+0 viewport transform changes; Fit is a safe no-op on an empty graph | `live-app/07-zoom.json` |
| CHAT_PIN command → matching ACK (correlation id) → `aria-pressed` flips (ACK-before-mutation) | `live-app/08-chat-pin.json` |
| Empty prompt → named `E_PROMPT_EMPTY` toast, **zero** SUBMIT_PROMPT frames (no fake success) | `live-app/09-empty-prompt.json` |
| WS hello: URL token equals `/api/session` token, `last_seq` matches, client sends `{"type":"hello","last_seq":N}`, server `SERVER_HELLO` schema/name | `live-app/10-ws-handshake.json` |
| Token colours: body `rgb(40,32,16)` on canvas `rgb(248,244,235)`, contrast 14.6677 ≥ 4.5 | `live-app/11-colours.json` |
| 28-surface trace capture: 23 surfaces render-verified (S01-S14, S20-S28), node-card surfaces S15-S19 honestly UNVERIFIED (no run started, no paid API) | `trace-capture.json` |

### Remaining failures (app-side, reported to Lead)

1. **`SETTINGS_STATUS` safe read is broken**: two `SETTINGS_STATUS` commands are sent per Settings open and the
   server answers both with `ERROR E_INTERNAL_ERROR` instead of `ACK` + `SETTINGS_STATUS_RESULT`. The Settings panel
   shows a named refusal ("Something went wrong", no fake success) — contract step 9 fails. Evidence:
   `live-app/12-settings-status.json` + failure screenshot in `e2e/test-results/`.
2. **Stale contract-gap copy in the shipped bundle**: Data panel still says "no workspace-file API yet" although
   `FILE_TREE_LIST`/`FILE_READ` are frozen; Approval overlay copy still says the frozen set has "no approval-response
   command" although `APPROVAL_RESOLVE` is frozen. Evidence: `live-app/13-contract-drift.json`, failure screenshot.

### Skipped / fixme (honest, not passes)

- `a11y-gates.spec.ts` 12 DOM gates: not implemented yet → `test.fixme`; static contrast math passes separately.
- `ws-contract.spec.ts` 2 stubs (synthetic foreign-Origin page, duplicate hello/ACK scenario) → `test.fixme`;
  the live equivalents are covered by `live-app.spec.ts` WS handshake / CHAT_PIN / empty-prompt tests.
- Node-card traces S15-S19 and streaming S27: require a run; no paid API was used, so UNVERIFIED.

## Reproduce

```powershell
cd e2e
$env:E2E_BASE_URL='http://127.0.0.1:8799'; $env:E2E_APP_READY='1'
npx playwright test --project=chrome --reporter=list,html
node ../tools/qa/apply-live-trace.mjs   # merge captured verdicts into docs/trace/controls.json
```
