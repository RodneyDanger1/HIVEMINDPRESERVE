# QA-ENV — QA/trace environment & artifact evidence audit

| Field | Value |
|---|---|
| Seat | QA / trace engineer (role changed from Watchdog 2; watchdog task closed) |
| Date | 2026-11-06, evidence captured 04:15–09:22 America/Chicago |
| Scope | read-only probes + harness files under `e2e/**`, `docs/trace/**`, `docs/audit/QA-*.md`, `tools/qa/**` |
| Verdict | **HARNESS LIVE ON SHIPPED ARTIFACT: 93 tests = 79 passed, 0 failed, 14 skipped (exit 0)** — no fabricated pass claims |
| Skills loaded | `trace`, `capability-auditor`, `evidence-check`, `mermaid-skill` (`docs/claims/SKILL-USE.md` is outside QA write scope) |

## Claim / evidence / status

| # | Claim | Evidence (command → observed) | Status |
|---|---|---|---|
| 1 | Node available (>=22.12.0) | `node --version` → `v26.7.0` | VERIFIED |
| 2 | npm available (pwsh `.ps1` blocked; use cmd) | `cmd /c npm --version` → `11.19.0` | VERIFIED |
| 3 | Playwright 1.65.0 installed in `e2e/` (package only) | `node -e require('./e2e/node_modules/@playwright/test/package.json').version` → `1.65.0` | VERIFIED |
| 4 | Installed Chrome/Edge channels used, no browser download | Chrome 156.0.8078.75 / Edge 156.0.4314.58; `channel: 'chrome'`; `PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1` at install | VERIFIED |
| 5 | Blueprint implementation pins match EXECUTION-PROMPT §0 | sha256: blueprint `4921ABE4…716B` ✅; UI-WIRING `3911C3E5…8104` ✅; EXTERNAL-DEPENDENCIES `0C933FBF…0308` ✅ | VERIFIED |
| 6 | Frozen source unchanged | `hivemind-blueprint/source/HIVEMIND POSSIBLITY.pruned.txt` sha256 `7B85760E…D713` | VERIFIED |
| 7 | Frozen contract extent (latest) | `docs/contracts/UI-COMMANDS-v1.md`: 14 frozen commands + 3 decision-gated DIFF, 21 events, 46 refusal codes (incl. `E_FILE_TREE_UNAVAILABLE`) | VERIFIED (parsed) |
| 8 | 28-surface matrix + supplemental approval control consistent | `node tools/qa/verify-harness.mjs` → **27 checks, 27 PASS, 0 FAIL**, exit 0 | VERIFIED |
| 9 | Shipped binary/artifact under test | `tools/bin/hiverouter.exe` 9,329,152 B, sha256 `184D51A7C3CCA7B9D50954B677C5AD8A14472700A651FDF0D270441BE3EAC9A9`, built 09:10:59; served UI asset `index-Cbfm1IuM.js`; server PID 33580 (start 09:11:05); `/health` 200, `/diagnose` 200 | VERIFIED |
| 10 | FINAL full Chrome suite on shipped artifact | **93 tests = 79 passed, 0 failed, 14 skipped, exit 0** (3.6m), log `e2e/test-results/run-live-FINAL-artifact.log`, HTML `e2e/playwright-report/index.html` (580,422 B, 09:21:29), evidence `docs/trace/live/` | VERIFIED |
| 11 | SETTINGS_STATUS contract path | live run: exactly 1 command → ACK + `SETTINGS_STATUS_RESULT` (provider deepseek, model deepseek-flash, key_present false); evidence `docs/trace/live/live-app/12-settings-status.json` | VERIFIED |
| 12 | FILE_TREE_LIST/FILE_READ path | live run: ACK + `FILE_TREE_RESULT` with `README.md`/`note.txt`; Data panel renders both; evidence `live-app/14-file-tree.json` + trace S22 chain WIRED-AND-WORKS | VERIFIED |
| 13 | Data panel unknown-size display | `README.md—`, `note.txt—` (server entries carry no `size_bytes`, so unknown path renders); **`0 B` absent** from DOM and from shipped bundle; fallback source `web/src/panels/DataPanel.tsx:44`; evidence `live-app/15-data-size.json` | VERIFIED |
| 14 | Contract-gap copy gone | shipped bundle has no "no workspace-file API yet" / "no approval-response command"; evidence `live-app/13-contract-drift.json` | VERIFIED |
| 15 | No console errors / uncaught exceptions on load | `live-app/03-console.json` | VERIFIED |
| 16 | WS handshake uses `/api/session` token | URL token == session token, `last_seq` match, client hello frame, server `SERVER_HELLO`; `live-app/10-ws-handshake.json` | VERIFIED |
| 17 | ACK-before-mutation (safe command) | CHAT_PIN command → matching ACK → `aria-pressed` flips; `live-app/08-chat-pin.json` | VERIFIED |
| 18 | Refusal without fake success | empty prompt → named `E_PROMPT_EMPTY` toast, **zero** SUBMIT_PROMPT frames; `live-app/09-empty-prompt.json` | VERIFIED |
| 19 | Token colours/contrast | body `rgb(40,32,16)` on canvas `rgb(248,244,235)` → 14.6677 ≥ 4.5; `live-app/11-colours.json` | VERIFIED |
| 20 | Trace capture refreshed from shipped-artifact run | `docs/trace/live/trace-capture.json` generated 2026-11-06T15:24:24.056Z, 28 surfaces, 23 present; hop verdicts 43 WIRED-AND-WORKS / 260 UNVERIFIED / 5 MISSING / 0 WIRED-BUT-DEAD | VERIFIED |
| 21 | Evidence-backed verdicts applied to controls | `docs/trace/controls.json`: 23 `PARTIALLY_VERIFIED`, 6 `PENDING_APP_NOT_BUILT`, 48 evidence-backed hop verdicts (`node tools/qa/apply-live-trace.mjs`) | VERIFIED |
| 22 | FigCraft assets present (A+B exports) | 16 exports (8 boards × PNG+SVG) + `A-design.figcraft` 126,928 B sha `93226DDF…CD74`; `figcraft-cli info` exit 0 (1368 nodes/31 styles/20 variables, 4 pages) | VERIFIED (point-in-time 04:55) |
| 23 | A manifest candidate present; canonical path absent | `A/A-ASSETS.json` (28/28 S01-S28 mapping, sha matching design); `ASSETS-INDEX.json` absent — Lead acceptance/alias pending; `A-SELF-CHECK.json ok:true` is self-report only | VERIFIED (presence/absence) |
| 24 | Disk headroom dynamic | measured 8.77 → 3.71 → 15.9 GB across the session | VERIFIED (snapshots) |
| 25 | QA writes stayed in scope | touched only `e2e/**`, `docs/trace/**`, `docs/audit/QA-ENV.md`, `tools/qa/**`; no web/backend/FigCraft/blueprint edits by QA | VERIFIED (git status review) |

## Discrepancies / notes (owned elsewhere)

1. **Playwright**: package-only install pinned 1.65.0 in `e2e/`; installed Chrome channel used; npx cache 1.62.1 is irrelevant.
2. **S03 envelope drift** (contract owner): UI-WIRING-GUIDE S03 writes `{type:"SUBMIT_PROMPT"}`; frozen envelope is `type:"COMMAND"` + `name:"SUBMIT_PROMPT"`. Harness asserts frozen envelope.
3. **FigCraft manifest path**: canonical `ASSETS-INDEX.json` missing; candidate `A/A-ASSETS.json` + self-check need Lead/UI-watchdog acceptance.
4. Font bundling / elevation tiering risks in `web/src/styles/tokens.css` — frontend owner (unchanged from earlier finding).
5. 14 skipped tests are `test.fixme`/UNDONE (12 a11y DOM gates, 2 WS stubs); they are **not passes**.

## Not verified / open

- Node cards S15-S19, streaming S27, terminal-run truth, provider/run lifecycle, usage NULL-vs-0 on failed runs — need one approved mock/wiremock run.
- G-UI-1..12 DOM gates (only static contrast + live colour/zoom/refusal/WS checks run); synthetic foreign-Origin refusal.
- FigCraft asset→surface mapping against PDF/UI requirements (candidate only).
- Performance P95 frame time (G-UI-12).
