# CONSOLIDATED FINDINGS REGISTER — HiveMind v1 audit

HEAD `3988622` · 507 tracked files · **30 consolidated findings (3 P0 / 10 P1 / 16 P2 / 1 informational)** · seats: b-core (BC), b-provider (BP), watchdog-1 (W1), watchdog-2 (W2), figcraft-a (FA), figcraft-b (FB), ui-watchdog (UW), strip-watchdog (SW). IDs below are audit IDs; per-seat originals are in `audit/parts/<seat>/FINDINGS.md`.

## P0 — correctness / data-integrity blockers (must fix before any live-use claim)

| ID | Finding | Evidence | Origin |
|---|---|---|---|
| A-F01 | **Shipped server cannot perform a live model run.** `hiverouter-core` has no dependency on `hiverouter-provider`; `http.rs` constructs an `UnconfiguredExecutor` and a default template registry, so `SUBMIT_PROMPT` can never reach the provider even when `DEEPSEEK_API_KEY` is set. All prior "provider wired" claims are crate-level only. | `backend/crates/hiverouter-core/Cargo.toml` (deps = contract + tools only); `core/src/http.rs:77,117,154`; `core/src/execution.rs`; `provider` is reached only through `tools` CLI, not the binary run path | BC-F01 |
| A-F02 | **History/replay rendering is dead.** `Canvas` passes an always-empty `replayNodes` array; `HISTORY_SNAPSHOT` is stored but never rendered, so the time-travel scrubber cannot show a graph. | `web/src/flow/Canvas.tsx:113-127` (`useMemo<Node[]>(() => [], [])`); `web/src/state/store.ts:379-385` | W1-F01 |
| A-F03 | **State vocabulary parity defect.** Backend contract declares **16** `AgentState` values (incl. `interrupted`); frontend declares **15** and omits `interrupted`, while its header calls the contract a "14-state list". A backend `interrupted` event is not representable by the UI decoder. | `backend/.../state.rs:18-51,55-72`; `web/src/state/agentState.ts:1-24` | Lead + W1 (pending C-seat confirmation of decode severity) |

## P1 — contract / verification / reproducibility defects

| ID | Finding | Evidence | Origin |
|---|---|---|---|
| A-F04 | Prompt/persona assets never loaded by the production server; handshake/context-switch code has no live rewire/engine caller. | `core/src/http.rs:82` (`TemplateRegistry::default()`); `core/src/ws.rs:338-353` (no `src` caller) | BC-F02/F03 |
| A-F05 | Settings panel renders a refusal from the wrong source (`refusals[0]`) instead of the `SETTINGS_STATUS` error. | `web/src/state/store.ts:666` | W1-F02 |
| A-F06 | Money overflow silently yields `Decimal::ZERO` in `price()` (violates the no-silent-zero cost rule). | `backend/.../provider/src/accounting.rs:330` | BP-H1 |
| A-F07 | Duplicate major versions resolved in lockfile (`thiserror` 1.0.69 + 2.0.21; `tower-http` 0.6.11 + 0.7.1) — the `cargo tree -d` gate is not clean. | `backend/Cargo.lock`; `cargo tree -d` not re-run in audit (`UNVERIFIED` verdict) | BP-H6 |
| A-F08 | Headline verification evidence is **untracked** (`docs/audit/evidence/final-gate-tests.log`, `e2e/test-results/run-live-FINAL-artifact.log`, etc.), so the 507-file manifest does not contain the receipts used by FINAL-REPORT/FINAL-DELTA. | `git ls-files` vs `FINAL-REPORT.md:36,38`; `FINAL-DELTA-VERIFICATION.md:8` | FB-F05 |
| A-F09 | 82 tracked `runtime-test/` artifacts (SQLite DB/WAL/SHM, snapshots, `.out/.err`) including the shipped binary stored **three times** (identical hash), one stale build, duplicate CLI, and nine DB/WAL sets. Generated artifacts committed to source control. | `git ls-files runtime-test`; hashes in W2-FINDINGS | W2-S9 |
| A-F10 | Documentation/claims drift: FINAL-REPORT trace counts (260 UNVERIFIED/5 MISSING) disagree with `docs/trace/controls.json` (270 PENDING/6 MISSING); FINAL-DELTA claim-register FAIL is stale after later repair; committed `evidence/final/pw-out/.last-run.json` says `failed` while closeout says passed. | `FINAL-REPORT.md:45`; `docs/trace/controls.json`; `FINAL-DELTA-VERIFICATION.md:12,23`; `docs/audit/evidence/final/pw-out/.last-run.json:2` | FB-F01..F04 |
| A-F11 | `.cargo/config.toml` hard-codes absolute host paths (`C:/Users/Riley/...`) for linker/CC/AR, so builds are machine-specific. | `.cargo/config.toml` | UW-F01 |
| A-F12 | **18 primary design deliverables are gitignored and outside the tracked manifest.** `.gitignore:7,9` excludes `*.figcraft` and `ASSETS/*.svg|png`; the 507-file manifest therefore omits the actual pixel products (18 files, 22,009,234 B). Hashes are preserved in A/B manifests, `ASSETS-INDEX.json` and the watchdog ledger and were recomputed in Appendix D. | `.gitignore:7,9`; `git check-ignore -v`; `audit/APPENDIX-IGNORED-DELIVERABLES.md` | FA-F1 / UW-F13 |

## P2 — hygiene, UX, maintainability

| ID | Finding | Evidence | Origin |
|---|---|---|---|
| A-F13 | 6 ad-hoc e2e probe scripts (`data-probe`, `live-check2/inspect/probe/shot/smoke`) are not wired to the Playwright runner. | `e2e/*.mjs`; `e2e/package.json` scripts | W2-S1 |
| A-F14 | Skip-as-green: app-required specs skip when `E2E_APP_READY` is unset, and a skipped run exits 0. | `e2e/helpers/matrix.ts`; `.last-run.json` | W2-S2 |
| A-F15 | Test output is wiped each run (`test-results/`), so run logs are not durable. | `e2e/playwright.config.ts` outputDir | W2-S3 |
| A-F16 | Trace capture is not completeness-enforced; `S-APPROVAL` has no capture; validator uses lower-bound thresholds (≥10/≥12/≥30) instead of exact sets. | `tools/qa/verify-harness.mjs`; `docs/trace/controls.json` | W2-S4/S8 |
| A-F17 | Tests write evidence into `docs/` during runs (generated artifacts mixed into documentation tree). | `e2e` trace/evidence writers; `docs/audit/evidence/**` | W2-S10 |
| A-F18 | Playwright default baseURL is `:8080` while the shipped workflow used `:8799`, with no artifact-identity preflight. | `e2e/playwright.config.ts`; runbook | W2-S7 |
| A-F19 | Declared Playfair/Inter faces are not bundled as font packages; UI relies on system/fallback fonts (R-U12 unmet). | `web/src/styles/tokens.css:30-32`; `web/package.json` (no font deps) | W1-F04 |
| A-F20 | Dev mock is statically imported in the production entry; runtime-gated but may be bundled. | `web/src/api/index.ts:8`; dev-only guard | W1-F05 |
| A-F21 | No HTTP timeout/abort on `/api/session` or `/health` fetch paths. | `web/src/api/client.ts:235`; `web/src/api/health.ts:15-22` | W1-F06 |
| A-F22 | Token-isolation test scans `src/` only; shipped `index.html:9` contains a hex favicon colour. | `web/index.html:9`; token test | W1-F07 |
| A-F23 | Approval nonce is treated as an optional extension; frozen `APPROVAL_REQUEST` row lacks it. | contract `event.rs` approval payload; UI-COMMANDS-v1 | W1-F08 |
| A-F24 | Stale docs: several pages still say the app does not exist / trace is empty after the release; claim-register JSON body retains stale decision-critical entries. | `docs/trace/live/README.md` pending sections; `CLAIM-REGISTER.json` C-026/decision list | UW-F02 / FB-F01 |

## Quality aggregation (code-only, 159 files scored before FA partition lands)

| Dimension | Mean /5 |
|---|---:|
| correctness | 4.50 |
| maintainability | 4.29 |
| modularity | 4.37 |
| error handling | 4.33 |
| security | 4.29 |
| documentation | 3.97 |

Full matrix: `audit/QUALITY-MATRIX.csv`; code-only stats: `audit/QUALITY-CODE-STATS.md`. Scores are evidence-weighted judgments, not coverage metrics; FA docs/asset partition (103 files, 30 build scripts) is pending and will move the code-file count to 188.

## Status discipline

- `VERIFIED`: run/read by the audit (e.g., manifest hashes, test counts cited from captured logs, source reads).
- `READ`: source inspected by a seat but not executed (most runtime claims).
- `UNVERIFIED`: no proving evidence yet (provider live path, cargo tree -d, bench/clippy, pixel fidelity, transitive licences).
- No finding becomes "fixed" in this audit; fixes belong to the roadmap waves.

## FigCraft-A partition findings (docs/ui + ledgers; full text in `audit/parts/figcraft-a/FINDINGS.md`)

| ID | Finding | Severity |
|---|---|---|
| FA-F1 | 18 primary design deliverables gitignored/outside the 507 manifest (registered in Appendix D; hashes verified) | HIGH → P1 |
| FA-F2 | `ASSETS-INDEX.json` carries A hashes (28/28) but **0/27** B export hashes | MED → P2 |
| FA-F3 | `ASSETS-INDEX` watchdog block stale ("spinner #D9D9D9 fix in progress") vs the ledger's fixed state | MED → P2 |
| FA-F4 | B episodes use non-standard `outcome="stage-ok"` (7/8) and omit command ids in `plan.steps` | MED → P2 |
| FA-F5 | 6 tracked CPython-3.14 `.pyc` files + scratch `B/build/_run.json`; no `*.pyc` ignore rule | MED → P2 |
| FA-F6 | 8 tracked patch scripts + intermediate `.figcraft`/QA PNGs in delivered build folders | LOW → informational |
