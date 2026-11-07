# 13 — Historical Audit Register and Current Disposition

## What the dump actually is

The supplied dump is **not** a 1.0-era repository snapshot and **not** a second independent source.

- 635 delimited `FILE:` sections (the 636th `FILE:` hit is the Rust constant `DEFAULT_DB_FILE`, not a section).
- Captured from the `e021b12` **working tree**, including 15 untracked `docs/CODEMAP/` drafts.
- Current tree at publication: `aa16e13`, 823 tracked files.
- The dump is a **strict subset** of the current tree: 0 dump-only paths, 188 current tracked files without a section (89 text, 99 binary/UTF-16), 586 normalized-identical, 1 changed (`docs/CODEMAP/00-START-HERE.md`, edited after capture).
- The dump omits large/current files such as `backend/crates/hiverouter-core/src/execution.rs`, `ws.rs`, and `tools/src/approval.rs`; those are among the 188 extras. That is an export/selection limit, not evidence they were absent.
- The audit paper inside the dump describes the older `v1.0.0` audit at `3988622`/507 files. That is a **historical document carried forward inside the capture**, not the capture identity.

## Historical findings register (30 findings)

The dump contains `audit/FINDINGS-REGISTER.md`: 30 consolidated findings (3 P0 / 10 P1 / 16 P2 / 1 informational), derived from 58 seat-level findings. The table below uses the subagent’s re-check against current source.

### P0

| ID | Historical finding | Current disposition |
|---|---|---|
| A-F01 | Shipped server could not perform a live model run; core had no provider dependency. | **FIXED.** Current core depends on `hiverouter-provider`; `http.rs` constructs the provider executor. |
| A-F02 | Replay rendering dead; Canvas passed `replayNodes = []`. | **FIXED.** Current Canvas renders `history.replayGraph` through replayNodes. |
| A-F03 | 16 backend states vs 15 frontend states; `interrupted` missing. | **FIXED.** Current `agentState.ts` declares 16 states including `interrupted`. |

### P1

| ID | Historical finding | Current disposition |
|---|---|---|
| A-F04 | Prompt/persona assets never loaded by production; handshake/context-switch no caller. | **FIXED for embedded prompt loading** (`prompt_assets_for`). Persona loader is a separate current gap: `load_personas` still has no production caller and `personas.json` only defines `planner`. |
| A-F05 | Settings rendered `refusals[0]` instead of the settings error. | **FIXED** per current-source re-check. |
| A-F06 | Money overflow silently yielded `Decimal::ZERO` in `price()`. | **STILL OPEN.** Current `accounting.rs:330-335` still does `checked_mul(...).map(...).unwrap_or(Decimal::ZERO)`. Adjacent cost paths refuse `CostMathInvalid`, so the silent fallback is a real inconsistency. |
| A-F07 | Duplicate major versions in `Cargo.lock` (`thiserror` 1.0.69+2.0.21, `tower-http` 0.6.11+0.7.1). | **STILL OPEN** by static lockfile grep. `cargo tree -d` was not run. |
| A-F08 | Headline verification evidence untracked/absent from the 507 manifest. | **FIXED** per current-source re-check. |
| A-F09 | 82 tracked `runtime-test/` artifacts, duplicate binaries, DB/WAL/SHM. | **STILL OPEN.** `git ls-files runtime-test` is still 82; 36 of those paths are newer than the dump. |
| A-F10 | Documentation/claims drift (trace counts, stale final report, failed last-run). | **NOT RECHECKED** in this pass; treat as historical drift until re-read. |
| A-F11 | `.cargo/config.toml` hard-coded absolute host paths. | **FIXED.** Current config uses `gcc`/`ar` names. |
| A-F12 | 18 primary design deliverables gitignored and outside the tracked manifest. | **STILL OPEN.** `.gitignore:7-9` still excludes `*.figcraft` and `ASSETS/*.png|svg`. |

### P2 / FigCraft

| ID | Historical finding | Current disposition |
|---|---|---|
| A-F13 | 6 ad-hoc e2e probe scripts not wired to Playwright. | **PARTLY OPEN.** All six still present; `run-guarded.mjs` exists but the probes remain outside the spec set. |
| A-F14 | Skip-as-green: skipped app specs exit 0. | **STILL OPEN (guard present).** `matrix.ts:44` returns true when `E2E_APP_READY !== '1'`; a skip still can exit 0. |
| A-F15 | Test output wiped each run. | **STILL OPEN.** `playwright.config.ts` outputDir is `./test-results`. |
| A-F16 | Trace completeness not enforced; lower-bound thresholds. | **STILL OPEN.** `verify-harness.mjs` still uses `>= 10`, `>= 12`, `>= 30`. |
| A-F17 | Tests write evidence into `docs/`. | **STILL OPEN.** `e2e/helpers/evidence.ts` and `trace.ts` resolve into `docs/trace/`. |
| A-F18 | Playwright default baseURL `:8080` vs shipped `:8799`. | **STILL OPEN (documented).** Config default remains `:8080`. |
| A-F19 | Playfair/Inter fonts not bundled. | **STILL OPEN.** No font dependency in `web/package.json`. |
| A-F20 | Dev mock statically imported into production entry. | **FIXED.** Current `index.ts` dynamically imports `./mock` behind the dev guard. |
| A-F21 | No HTTP timeout/abort on session/health fetches. | **FIXED.** Current client uses `fetchWithBoundTimeout` and configured bounds. |
| A-F22 | Token-isolation test scans only `src/`; `index.html` hex favicon colour. | **STILL OPEN (moved).** `web/index.html:7` theme-color hex remains. |
| A-F23 | Approval nonce treated as optional. | **PARTIAL.** Contract now documents optional nonce; UI refuses nonce-less requests rather than inventing one; frozen compatibility remains optional. |
| A-F24 | Stale docs/claims after release. | **NOT RECHECKED.** Current public docs were corrected, older plans remain historical. |
| FA-F1 | 18 design deliverables gitignored/outside manifest. | **STILL OPEN** (same as A-F12). |
| FA-F2 | ASSETS-INDEX has A hashes but 0/27 B hashes. | **NOT RECHECKED.** |
| FA-F3 | ASSETS-INDEX watchdog block stale. | **NOT RECHECKED.** |
| FA-F4 | B episodes non-standard `stage-ok` / missing command ids. | **NOT RECHECKED.** |
| FA-F5 | 6 tracked CPython 3.14 `.pyc`; no ignore rule. | **PARTIAL.** `*.pyc` ignore exists, but the six old tracked files remain. |
| FA-F6 | 8 tracked patch scripts/intermediate assets. | **NOT RECHECKED.** |

## Reconciliation counts

- **9 fixed**: A-F01, A-F02, A-F03, A-F04, A-F05, A-F08, A-F11, A-F20, A-F21.
- **12 still open**: A-F06, A-F07, A-F09, A-F12, A-F13, A-F14, A-F15, A-F16, A-F17, A-F18, A-F19, A-F22.
- **2 partial**: A-F23, FA-F5.
- **6 not rechecked**: A-F10, A-F24, FA-F2, FA-F3, FA-F4, FA-F6.
- FA-F1 counts with A-F12.

## Historical metrics — do not use as current

507 tracked files; 63.98 MiB tracked; 188 code-classified; 36,802 source LOC; 405 Cargo packages; 147 npm packages; 233 backend tests; 79 live-browser tests. **Current gates:** 475 backend / 179 web / 106 mocked E2E / 0 failed; 823 tracked after CODEMAP publication.

## Dump integrity findings

- 49 of 635 bodies are synthetic placeholders (28 `[Empty file]`, 20 `[Binary file]`), including a false binary (`plans/waves/W1-CORE-RECOVERY.md`).
- 4 control characters corrupt identifiers inside the dump’s copy of `plans/foresight/RUN-3.md`.
- The audit’s own `FILE-MANIFEST*`, `QUALITY-MATRIX.csv`, `APPENDIX-FILE-CENSUS.md`, and `backend/Cargo.lock` are absent from the dump, so its 507-file hash claim is **read, not self-verified**.
- Two contradictory quality tables exist inside the audit corpus (159-file pre-merge vs 184/188 final); neither is current.
- The 18 ignored FigCraft deliverables are the only surviving hash record for assets absent from both git and the dump bytes.
