# PHASE B VERIFICATION — independent verifier

- **Seat:** independent verifier (read-only on product code; writes only `docs/audit/**`, `docs/claims/VERIFIER-*.md`, `docs/diagrams/**`).
- **Date:** 2026-11-06, 07:28–08:20 America/Chicago.
- **Tree:** `C:\Users\Riley\Desktop\hiverouter-v1`, `git HEAD ab3a33b` ("W4: live server serves embedded UI, release binary green, e2e harness unblocked").
- **Live artifact:** `tools\bin\hiverouter.exe`, 9,099,776 B, sha256 `8C777A786FACE7C586910E8BD854B8C5AADF1C3765FE4DD7D1A11503C8EC3827`, process PID 29736 started 07:27:20, listening `127.0.0.1:8799` (loopback only).
- **Method:** every gate below was re-run by this seat on the running tree; no teammate receipt was accepted. No product edits, no installs, no paid API call (no `DEEPSEEK_API_KEY` present — a live `SUBMIT_PROMPT` correctly failed with `E_KEY_MISSING`, no provider request).
- **Evidence directory:** `docs/audit/evidence/phase-b/` (`cargo-test.log`, `cli/*.out|*.err`, `live-probe-verifier.json`, `e2e-chrome.log`).

## Gate verdicts

| # | Gate | Verdict | Evidence |
|---|---|---|---|
| G1 | `cargo test -p hiverouter-contract -p hiverouter-core -p hiverouter-provider -p hiverouter-tools` | **VERIFIED** | 25 `test result` suites (21 test binaries + 4 empty doc-test runs), **228 passed / 0 failed / 1 ignored**, exit 0 (`cargo-test.log`): contract **32** (18 lib + 3 golden + 6 roundtrip + 5 matrix), core **78** (34 lib + 5 chat_store + 2 context_switch + 7 graph_authority + 2 handshake + 5 http_boot + 2 kill_restore(+1 ignored helper) + 2 prompt_assets + 6 run_lifecycle + 4 snapshot_history + 9 ws_transport), provider **40** (29 lib + 11 wiremock refusal matrix), tools **78** (77 lib + 1 JSONL audit e2e). These match the claimed 32/78/40/78 exactly. |
| G2 | Live server HTTP surface | **VERIFIED** | `GET /health` → 200, JSON `status:ok`, `schema_version:3`, checks rustc 1.99.1 / node v26.7.0; `GET /api/session` → 200 `{token, schema:hiverouter.v1, ws:/ws}`; `GET /` → 200 `text/html`, `<div id="root">`; `/assets/index-_C6E8rly.js` → 200 669,206 B; `/assets/index-BWOl__uM.css` → 200 35,753 B; SPA fallback `/workspace/agents` → 200 HTML. |
| G3 | Path traversal / static safety | **VERIFIED** | `--path-as-is` probes `/assets/../Cargo.toml`, `/assets/%2e%2e/Cargo.toml`, `/assets/..%2fCargo.toml`, `/../Cargo.toml`, `/assets/../../backend/Cargo.toml` → all **404** JSON, no source leak. |
| G4 | WS transport security + protocol | **VERIFIED** | curl upgrade: foreign `Origin: http://evil.example` → **403 `E_ORIGIN_REJECTED`** JSON; allowed Origin + wrong token → **401 `E_TOKEN_INVALID`** JSON; allowed Origin + session token → **101 Switching Protocols**. Browser live probe: `hello` → `SERVER_HELLO`; `SUBMIT_PROMPT ""` → `E_PROMPT_EMPTY`; valid `SUBMIT_PROMPT` → `RUN_CREATED` + `ACK` + `RUN_STATUS failed (E_KEY_MISSING)` (honest refusal, no fake success); `SPAWN_SUB_HIVE` unknown parent → `E_TASK_NOT_FOUND`. |
| G5 | Capability census (registry==prompt==callable) | **VERIFIED** | `hiverouter-cli capabilities census --format=json` exit 0: `tool_count 7`, `refusal_count 64`, `tier_count 4`, `permission_class_count 4`; `registry_tool_ids == allowlist_tool_ids == prompt_claim_tool_ids` (**all three sets equal**, 7 ids: spawn_agent, report_result, finish, file_read, file_list, file_write, shell_exec); census derives counts from `strum::EnumIter` (no typed constants). |
| G6 | Agent-native CLI review (version/schema/exit codes/dry-run/json) | **VERIFIED with 2 defects** | `version --format=json` exit 0, stdout pure JSON (stderr 0 B); `schema` exit 0 (progressive command table with tier/mutating); `health --url http://127.0.0.1:8799` exit 0 live; `health --dry-run` exit 0 request-shape only; `preflight` (cargo PATH) exit 0 `ready:true`; `preflight` PATH-cleared exit **3** `E_TOOLCHAIN_MSRV`; unknown command exit **2** `E_COMMAND_UNKNOWN`; missing positional exit **2** `E_SCHEMA_INVALID`; `tools describe rm_rf` exit **3** `E_TOOL_NOT_FOUND`; provider cost peak `1.506` / off-peak `0.753` (Decimal strings); human mode: stdout 0 B, human line on stderr. **Defects D-3, D-7 below.** |
| G7 | Claim-register audit (C-016, C-026, C-301..C-329 and register integrity) | **FAIL (register drift) + open claim** | CSV 73 rows, 73 unique ids: **64 VERIFIED / 8 READ / 1 UNVERIFIED**; JSON summary says **63 VERIFIED** → sums to 72 ≠ 73 (**D-4**). All 25 focus rows inspected; C-016/C-301..C-329 re-checked against my own runs (contract 32, core 78, provider 40, tools 78, release binary hash `8C777A78…` = my observed hash, non-loopback bind refusal `E_ORIGIN_REJECTED` exit 1, history gap `404 E_HISTORY_GAP` on `/state?revision=99`, UI watchdog PASS). **C-026 remains UNVERIFIED** (see G8). |
| G8 | Browser live e2e (C-026) | **UNVERIFIED (harness defect)** | Independent Playwright/Chrome probe (`live-probe-verifier.json`): app renders (`status 200`, title, `#root`, 26 buttons, 6 nav items, 4 tabs, canvas aria, chat core, minimap `rf__minimap`, HUD `— … NO METRICS`, scrubber disabled) and live WS command round-trips work (G4). **But the project e2e suite cannot execute app-required tests:** `e2e/helpers/matrix.ts:47` `skipReason()` returns a constant string unconditionally (never undefined); run with `E2E_BASE_URL=http://127.0.0.1:8799` → **370 tests: 7 passed (static only), 363 skipped**, exit 0 (`e2e-chrome.log`). A skipped run is not evidence, and `e2e/test-results/.last-run.json` `status: passed` is therefore a vacuous pass (**D-1**). |
| G9 | Trace ladder (28 surfaces × 11 hops) | **UNVERIFIED (honest)** | `docs/trace/controls.json`: 29 controls (28 + S-APPROVAL), **all `PENDING_APP_NOT_BUILT`**, all hops `PENDING`; `traces.json` status `NO_RUNS`, `traces: []`; `events.json` status `NOT_COLLECTED`. No fabricated verdicts (integrity PASS), but the trace gate has not been executed → cannot be claimed. |
| G10 | UI/PDF conformance vs ui-watchdog ledger | **PARTIAL** | `ledgers/ui-watchdog.json` (05:23) `overall: PASS`, `allow_frontend: true`, 20 artifacts, `blueprint_hashes_ok: true`; canonical `docs/ui/figcraft/ASSETS-INDEX.json` present (35,998 B). Asset-level conformance VERIFIED; **DOM-level G-UI-1..12 runtime gates UNVERIFIED** (blocked by D-1); pixel fidelity remains `needs_human` by design. |
| G11 | Stub sweep on product sources | **VERIFIED** | 105 shipped-source files scanned (`backend/crates/*/src`, `web/src`, `tools/qa`, `tools/*.ps1`): 47 hits, all classified **permitted**: 33 `mock`/10 `fake` are (a) the DEV-only `web/src/api/mock.ts` test double gated by `import.meta.env.DEV && VITE_UI_MOCK === '1'` whose constructor throws in production, (b) `wiremock` dev-only provider tests, (c) comments/refusal copy that explicitly say "never faked"; (d) `refusal.rs` "decision-gated and is not implemented in v1" (correct `DIFF_*` refusal). **0 TODO/FIXME/XXX/placeholder implementations.** Production bundle contains **0** mock strings. |
| G12 | Artifact truth (release binary + embedded UI) | **VERIFIED with staleness note** | Running binary sha256 `8C777A78…` matches claim C-329 and the on-disk artifact; embedded index/assets served 200. **Note:** the binary embeds an older frontend build (`index-_C6E8rly.js`) than current `web/dist` (`index-DzTucYf3.js`) → rebuild before the final release claim (**D-5**). |
| G13 | Environment / no-paid-API | **VERIFIED** | `DEEPSEEK_API_KEY` absent (preflight `present:false`); live `SUBMIT_PROMPT` → `RUN_STATUS failed E_KEY_MISSING`, no provider call; disk free 17.99 GB; no installs by verifier. |

## Defects (remaining, with fix seams)

| ID | Sev | Defect | Evidence | Fix seam |
|---|---|---|---|---|
| D-1 | **P1** | Project e2e suite cannot execute any app-required test: `skipReason()` is a constant string, so all 363 app tests skip even with the live app on :8799; the `.last-run.json "passed"` is vacuous. C-026 and DOM G-UI gates cannot close. | `e2e/helpers/matrix.ts:42-52` (`appMissing()` unused; `skipReason` unconditional); `e2e-chrome.log` "363 skipped / 7 passed". | `e2e/helpers/matrix.ts`: return `undefined` when `E2E_APP_READY=1` and a real probe of `/health` passes; implement trace-hop evidence capture instead of `void hop`; then re-run `npm run e2e:chrome` against :8799. |
| D-2 | **P1** | Frozen-contract commands are not implemented in the server WS dispatch: `SETTINGS_STATUS`, `FILE_TREE_LIST`, `FILE_READ`, `APPROVAL_RESOLVE` hit the fallback and return **`E_INTERNAL_ERROR` (`retryable:true`)** — a misleading code, not a truthful "not available" refusal. UI surfaces S21 (Settings), S22 (file tree) and the approval overlay therefore cannot be wired; Phase 6 is crate-level green only, not end-to-end. | `backend/crates/hiverouter-core/src/ws.rs:397-405` ("refuse by name until then" but publishes `Refusal::InternalError`); live probe frames: `SETTINGS_STATUS → ERROR E_INTERNAL_ERROR`, `FILE_TREE_LIST → ERROR E_INTERNAL_ERROR`. | Wire the Phase-6 tools-crate handlers into `ws.rs` dispatch, or return a distinct named refusal (`E_COMMAND_UNAVAILABLE`) and mark S21/S22/S-APPROVAL `BLOCKED` in `docs/trace/controls.json`; re-run browser e2e. |
| D-3 | **P2** | CLI `diagnose` is unusable against the shipped server: core router has no `/diagnose` route (only `/health`, `/api/session`, `/api/history/*`, `/ws`), so `GET /diagnose` falls through to the SPA (200 HTML); the CLI maps the non-JSON body to `E_WS_DISCONNECTED` (retryable), which is untrue. Runbook §3 step 4 tells operators to run it. | `hiverouter-cli diagnose --url http://127.0.0.1:8799 --format=json` exit 3 with `E_WS_DISCONNECTED … local server returned non-JSON`; `http.rs:207-212` route list; `/diagnose` → 200 `text/html` SPA body. | Either mount the tools-crate `diagnose` handler on the core router (it exists and is tested in `hiverouter-tools/src/health.rs`), or drop the command from CLI/runbook and return a truthful refusal; add a live `diagnose` e2e assertion. |
| D-4 | **P2** | Claim register summary drift: CSV has 73 rows = 64 VERIFIED / 8 READ / 1 UNVERIFIED, but `CLAIM-REGISTER.json` `counts` says **63 VERIFIED** (sum 72 with total 73). | `docs/claims/CLAIM-REGISTER.csv` vs `.json` re-parse this session. | Regenerate the JSON `counts` from the CSV (single source); re-parse must sum to `total`. |
| D-5 | **P2** | Release binary embeds a stale web build: served `index-_C6E8rly.js` vs current `web/dist` `index-DzTucYf3.js`. C-329 is valid for its 07:19 build time but the shipped artifact is behind the current frontend. | served `/` asset refs vs `web/dist/index.html`; binary built 07:19:06, dist rebuilt after. | Rebuild `cargo build --release --locked` after frontend freeze, re-probe root/assets/health, re-record hash; keep both hashes in PHASE-12 evidence. |
| D-6 | **P3** | `GET /favicon.ico` → 404 JSON, producing a browser console 404 (only console error on load; page has inline data-URI icon). Cosmetic. | CDP capture: `{url: http://127.0.0.1:8799/favicon.ico, status: 404}`; console error quoted in `live-probe-verifier.json`. | Serve a favicon (or return 204) from the static layer. |
| D-7 | **P3** | `preflight` with rustc absent reports `E_TOOLCHAIN_MSRV` "older than the required MSRV" even though rustc is missing (`observed: null`) — misleading diagnosis. | `cli/preflight.err` PATH-cleared run: exit 3 `E_TOOLCHAIN_MSRV` message "older than the required MSRV 1.89.0", `observed: null`. | Split `E_TOOLCHAIN_MISSING` vs too-old message, or word the MSRV refusal to cover absent/older. |
| D-8 | **P3** | `CHANGE-LEDGER-IMPL.md` CL-I-001 still says `applied-pending-evidence (Rust enum + matrix test pending)` although the enum and 5 matrix tests now exist and pass. | `CHANGE-LEDGER-IMPL.md:8` vs my contract test run (32 passed, incl 5 matrix). | Update the row to `applied (evidence: 32 contract tests)` . |

## Open claim / gate that must not be claimed

- **C-026 (browser live UI e2e)** stays **UNVERIFIED**. My independent probe proves the shell renders, the session token + WS handshake work, and command round-trips are honest (`E_PROMPT_EMPTY`, `E_KEY_MISSING`, `E_TASK_NOT_FOUND`) — but a click-driven UI e2e did not execute (D-1) and the S21/S22/approval flows cannot work until D-2 is fixed.
- **Trace ladder** (28 surfaces × 11 hops) is **not executed**: 29 PENDING controls, `NO_RUNS`, `NOT_COLLECTED`. Honest absence, not a pass.
- **DOM-level G-UI-1..12** gates unexecuted (D-1); asset-level UI/PDF watchdog conformance is PASS.

## V2 re-verification — artifact changed during this audit (read with the tables above)

The tree was **not quiescent**: while this seat verified the 07:27 artifact, the Lead rebuilt and restarted the server. Both snapshots are recorded (times America/Chicago):

| Snapshot | Binary sha256 | PID / start | Served UI | Notes |
|---|---|---|---|---|
| V1 | `8C777A78…3827` (9,099,776 B, built 07:19:06) | 29736 @ 07:27:20 | embedded `index-_C6E8rly.js` (stale vs dist at the time) | all gates in the tables above |
| V2 | `71B229B5…C38FC` (9,099,776 B, built 07:44:57) | 7292 @ 07:45:18 | embedded `index-DzTucYf3.js` = current `web/dist` (dist mtime 07:42:45) | re-probed below; DB `runtime-test/data6`, schema_version 3 |

**V2 live probes (my runs):** `/health` 200 (rustc 1.99.1 / node v26.7.0, schema_version 3, last_seq 5), `/api/session` 200, `/` 200 774 B with `<div id="root">`, assets `/assets/index-DzTucYf3.js` + `index-BWOl__uM.css` 200; **0 console errors / 0 4xx / 0 failed requests** in Chrome (favicon fixed vs V1); WS `hello` → `SERVER_HELLO`, `SUBMIT_PROMPT ""` → `E_PROMPT_EMPTY`, valid `SUBMIT_PROMPT` → `RUN_CREATED` + `ACK` + `RUN_STATUS failed E_KEY_MISSING`, `SPAWN_SUB_HIVE` unknown parent → `E_TASK_NOT_FOUND`. `/diagnose` still returns 200 `text/html` SPA (D-3 persists). `SETTINGS_STATUS` / `FILE_TREE_LIST` still → **`E_INTERNAL_ERROR`** (D-2 persists). Evidence: `live-probe-verifier-v2.json`, `live-probe-verifier-v2.out.txt`. Probe side effects: two runs appended (status failed, `E_KEY_MISSING`, cost NULL) to the runtime DBs; no paid provider call.

**V2 project e2e (QA implemented real Playwright live specs during this audit; D-1 partially resolved):** with `E2E_BASE_URL=http://127.0.0.1:8799 E2E_APP_READY=1`, `playwright test --project=chrome` → **91 tests: 87 passed, 4 failed (4.0 m), exit 1** (`e2e-chrome-v2.log`). The "363 skipped" result in G8/D-1 was the pre-fix state; the new `e2e/live-app.spec.ts` executes and captures evidence (screenshots + `docs/trace/live/live-app/*.json`). The 4 failures are real gate items:

| # | Failed test | Observed | Classification |
|---|---|---|---|
| F1 | `live-app: chrome surface census` | HUD rendered text `"NO METRICS"` (source `strings.ts:45 sourceNone: 'no metrics'`; CSS uppercases it) vs test literal `'no metrics'` | test brittleness (case) — P3; align assertion or copy |
| F2 | `live-app: zoom controls and Ctrl+0 change viewport transform` | after Fit swarm then Ctrl+0 the transform is unchanged (`matrix(1.2,0,0,1.2,474,269.5)`) | product/test reconciliation — Ctrl+0 may be a no-op when already centred; confirm intent (P3 / UNVERIFIED) |
| F3 | `live-app: SETTINGS_STATUS … ACK + SETTINGS_STATUS_RESULT` | **two** `SETTINGS_STATUS` frames sent per Settings open; server also answers `E_INTERNAL_ERROR` | product defect: duplicate command emission (P2), blocked further by D-2 |
| F4 | `live-app: Data panel and Approval copy reflect frozen contract` | Data panel still says "no workspace-file API"; approval copy stale vs frozen `FILE_TREE_LIST`/`FILE_READ`/`APPROVAL_RESOLVE` | product copy/contract drift; symptom of unimplemented commands (P2, fix with D-2) |

**V2 trace capture:** the live trace tests passed and appended `docs/trace/live/trace-capture.json` (rule: only hops with captured DOM/interaction evidence are `WIRED-AND-WORKS`, everything else `UNVERIFIED`) plus `docs/trace/live/live-app/*.json`. Canonical `docs/trace/{controls,traces,events}.json` are still the honest placeholders (29 × `PENDING_APP_NOT_BUILT`, `NO_RUNS`, `NOT_COLLECTED`) — the live capture is **not merged** into the canonical trace ledger, so the trace gate stays OPEN (partial evidence, not a pass).

**Defect status after V2:** D-1 **partially resolved** (live suite exists and executes; 4 red tests remain); D-5 **resolved** (V2 embeds current dist); D-6 **resolved** (0 console/4xx); D-2 and D-3 **confirmed on V2**; D-4, D-7, D-8 unchanged; F1/F2 (P3) and F3/F4 (P2) added.

## Verdict (V2 snapshot)

**PASS-WITH-DEFECTS (conditional). The backend/serve/CLI/census/stub/asset gates are independently VERIFIED; the live UI/e2e gate is RED and the release gate is NOT clean.**

- The Lead's Phase 1/2/3/4/9 (contract 32, core 78) and Phase 5/6/11/12 (provider 40, tools 78) claims are **confirmed by my own re-run** — 228 passed / 1 ignored / 0 failed, exit 0. Backend sources were unchanged between that run and the V2 rebuild (git status showed only web/e2e/binary/evidence changes); the 07:44 rebuild re-embedded updated web assets.
- **VERIFIED:** baseline hash/toolchain gate, live HTTP surface, WS Origin/token/hello protocol, traversal refusal, CLI contract + capability census equality (registry==allowlist==prompt), UI-watchdog asset conformance, stub sweep, non-loopback bind refusal, history API + `E_HISTORY_GAP`.
- **OPEN / BLOCKING for a release claim:** **D-2** (frozen `SETTINGS_STATUS`/`FILE_TREE_LIST`/`FILE_READ`/`APPROVAL_RESOLVE` unimplemented → `E_INTERNAL_ERROR`; S21/S22/approval unwired) and **F3** duplicate `SETTINGS_STATUS` emission, **F4** stale contract copy; **C-026 remains UNVERIFIED** (4 red live e2e tests); canonical trace ledger unmerged; **D-4** register counts; **D-3** `/diagnose`; P3 polish (D-7, D-8, F1/F2).
- **Fixed during the audit:** D-1 partially (QA live suite), D-5, D-6.
- The audit observed **two artifact snapshots and ongoing web/e2e edits**; a green re-run of these gates on a frozen tree is required before any release claim.

## Report shape

| Done | Evidence |
|---|---|
| Re-ran all four crate test suites | 228 passed / 0 failed / 1 ignored, 25 binaries, exit 0; per-crate counts as claimed (`docs/audit/evidence/phase-b/cargo-test.log`) |
| Probed live server HTTP/WS/security/traversal | root/assets/SPA/health/session 200; traversal 404; Origin 403 / token 401 / WS 101; history API + `E_HISTORY_GAP`; non-loopback bind refused `E_ORIGIN_REJECTED` |
| Capability census + agent-native CLI review | `census` equality 7/64/4/4; version/schema/preflight/tools/cost/health/dry-run/exit-code matrix; JSON-stdout/human-stderr split |
| Independent browser probe | `live-probe-verifier.json` — shell, session, WS hello, 5 command round-trips |
| Claim-register audit | 73 rows, focus C-016/C-026/C-301..C-329 re-checked; D-4 drift found |
| Stub sweep | 105 files, 47 classified-permitted hits, 0 TODO/FIXME, production bundle 0 mock strings |
| UI/PDF conformance vs watchdog ledger | watchdog PASS / 20 artifacts / ASSETS-INDEX present; DOM gates UNVERIFIED |
| This report | `docs/audit/PHASE-B-VERIFICATION.md` |

| Not done | Reason |
|---|---|
| Click-driven UI e2e / trace-hop evidence capture | D-1: suite skips all app tests unconditionally |
| S21/S22/approval end-to-end | D-2: commands unimplemented server-side |
| DOM G-UI-1..12 gates | blocked by D-1 |
| Release-claim closure | D-1/D-2 + C-026/trace open |

| Unverified | What would prove it |
|---|---|
| C-026 live UI e2e | fix D-1 + D-2, run `npm run e2e:chrome` against :8799 with real assertions, append `traces.json` |
| Trace ladder 28×11 | same run with evidence capture per hop; verdicts may leave PENDING only with evidence |
| Pixel-level UI/PDF fidelity | human visual acceptance (no board raster; `needs_human`) |
| Live provider behavior / cost vs real usage | user-approved key + budget (per policy, not run) |
