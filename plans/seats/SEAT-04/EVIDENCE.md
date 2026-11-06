# EVIDENCE — SEAT S4 (Live UX/E2E)

Read-only seat. Every command below was read-only; only writes are under `plans/seats/SEAT-04/`. No key access, no paid call, no install/rebuild, no git write.

## Commands and observed output

| # | Command | Observed |
|---|---|---|
| 1 | `git rev-parse HEAD` | `5c69b66e363e9f1d0801ea5c2e66a1fc12c8943c` (paper baseline 3988622; tree ahead) |
| 2 | `git status --short` | M backend/Cargo.lock, 4 crate Cargo.toml, web/package.json; ?? docs/RELEASE.md — provider/replay integration present but uncommitted |
| 3 | `Get-Process hiverouter`; `curl /health` | no process; `health=000` → live journey could not be re-executed |
| 4 | `e2e/test-results/run-live-FINAL-artifact.log` | 14,217 B, mtime 09:21:29; 79 passed / 0 failed / 14 skipped |
| 5 | `e2e/playwright-report/index.html` | exists, 580,422 B |
| 6 | `docs/trace/live/trace-capture.json` | generated 2026-11-06T15:24:24Z; 28 surfaces; 43 WIRED-AND-WORKS / 260 UNVERIFIED / 5 MISSING hops |
| 7 | `docs/trace/controls.json` | 29 controls; 23 PARTIALLY_VERIFIED / 6 PENDING_APP_NOT_BUILT |
| 8 | grep `web/src/flow/Canvas.tsx` | replay nodes built from HISTORY_SNAPSHOT at 114-118,135 (A-F02 addressed in working tree) |
| 9 | grep `backend/.../core/src/http.rs` | ProviderExecutorAdapter wired at 21,25,67-73 with UnconfiguredExecutor fallback (A-F01 addressed in tree) |
| 10 | grep `web/src/state/agentState.ts` | 16 states incl `interrupted` at 9-28 (A-F03 addressed in tree) |
| 11 | file hashing | every READLOG file re-hashed SHA-256 and line-counted from disk (see READLOG) |
| 12 | docs fetch | 4 DeepSeek docs pages fetched HTTP 200 at 2026-11-06T17:06:44.002Z; pricing table absent and 2 tails truncated → UNVERIFIED |
| 13 | `git ls-files docs/evidence/live-provider-smoke.md` | tracked (smoke evidence is part of the corpus, unlike the final e2e log) |

## Evidence inventory
- Paper: `audit/HIVEMIND-AUDIT-PAPER.md` (244 lines, sha256 in READLOG).
- Live smoke: `docs/evidence/live-provider-smoke.md` (tracked, 13 lines).
- E2E artifacts: `e2e/test-results/run-live-FINAL-artifact.log`, `e2e/playwright-report/index.html`, `docs/trace/live/trace-capture.json`, `docs/trace/live/live-app/*.json`.
- Assigned audit files: `audit/diagrams/05-dependency-lineage.mmd`, `audit/tools/build_roadmap.py`.

## Limits
- No live server and no paid provider call were available; journey evidence is the tracked smoke doc plus prior e2e artifacts, not a fresh run.
- Two fetched docs pages were truncated; content beyond truncation is UNVERIFIED.
- `web/src/hooks/**` does not exist; recorded as missing.
