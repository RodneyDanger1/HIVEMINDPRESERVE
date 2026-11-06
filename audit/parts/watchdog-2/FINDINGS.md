# FINDINGS — watchdog-2 partition (e2e/**, tools/**, runtime-test/**)

Severity scale: HIGH = can hide product failure; MEDIUM = weakens evidence/CI integrity; LOW = debt/robustness; INFO = accepted limitation. All findings are read-only observations with `path:line` evidence; no fixes were made.

## S1 — Ad-hoc probe scripts retained in e2e (MEDIUM)

`e2e/data-probe.mjs`, `e2e/live-check2.mjs`, `e2e/live-inspect.mjs`, `e2e/live-probe.mjs`, `e2e/live-shot.mjs`, `e2e/live-smoke.mjs` are exploratory scripts not wired into `npx playwright test`; they duplicate live checks, have no assertions/exit discipline, and `live-inspect.mjs` writes `docs/trace/live/inspect.json`. Evidence: `e2e/package.json` scripts (playwright test + tools/qa only); file scan flags ad-hoc-probe; `e2e/live-inspect.mjs:1-16`.

**Recommended action:** Move to a tools/dev probes folder, wire into scripts with explicit exit codes, or delete; keep the single evidence generator.

## S2 — Skip-as-green risk when env absent (MEDIUM)

All browser tests call `test.skip(appMissing(), skipReason())` (e.g. `e2e/specs/live-app.spec.ts:16`, `trace-hops.spec.ts:91`, `ws-contract.spec.ts:12`); with `E2E_APP_READY!=1` the suite reports success with 0 executed tests (observed earlier: `.last-run.json` "passed" from an all-skip run). A CI/Press-runner invocation without env can therefore look green. Evidence: `e2e/helpers/matrix.ts` appMissing gate + previous `.last-run.json`.

**Recommended action:** Fail closed when CI env requests execution: e.g. assert `E2E_APP_READY=1` in a setup test if `CI` is set, or make the gate a `test.fail`/config error.

## S3 — test-results artifacts are ephemeral (MEDIUM)

Playwright `outputDir: ./test-results` (`e2e/playwright.config.ts:21`) is wiped at the start of every run; manually stored run logs under `e2e/test-results/run-*.log` were deleted by a later trace-only run (observed during closeout: log existed at 09:21, then disappeared after a subsequent run and had to be regenerated). Evidence: config line + observed file lifecycle.

**Recommended action:** Write durable run receipts outside `outputDir` (e.g. `docs/trace/live/runs/<timestamp>.log`) or copy logs post-run via a wrapper script.

## S4 — Trace capture completeness not enforced (MEDIUM)

`trace-hops.spec.ts` writes whatever was accumulated in the module-level `records` array in `test.afterAll` (`trace-hops.spec.ts:76-87`) without asserting `records.length===28`, unique IDs, or the 11-hop schema; an early test failure/crash yields a partial capture file that downstream `apply-live-trace.mjs` accepts. Also the matrix has 29 controls (incl. `S-APPROVAL`) while `SURFACES` has 28 (`trace-hops.spec.ts:28-57`) — no S-APPROVAL trace record exists. Evidence: file lines; `docs/trace/controls.json` has 29 controls.

**Recommended action:** Add a capture-integrity check (28 guide IDs exactly, one record each, all 11 hops) in afterAll or a validator check; add explicit S-APPROVAL record or document exclusion.

## S5 — S21 trace under-claims render-back (LOW)

Settings branch sets hops 2–5 and 9 from captured ACK/result (`trace-hops.spec.ts:115-142`) but leaves `10_render_back` UNVERIFIED even though Settings rendered provider/model/key values; in the final capture S21 has 9=WIRED-AND-WORKS, 10=UNVERIFIED. Evidence: `docs/trace/live/live-app/12-settings-status.json` and capture S21 record.

**Recommended action:** Assert and record the rendered provider/model/key text after the result frame, then set 10_render_back from that evidence.

## S6 — Tests hardcode runtime fixture names (LOW)

`live-app.spec.ts:282-283, 307-312` and `trace-hops.spec.ts:171-172,204-206` assert literal `note.txt`/`README.md`/empty-state copy from the server workspace fixture. Changing `runtime-test/workspace` breaks tests independent of product behaviour; repo also contains the fixture files as tracked artifacts.

**Recommended action:** Derive expected entries from the FILE_TREE_RESULT payload instead of literals, or provision fixtures in a test setup step.

## S7 — Base URL default drifts from shipped server (LOW)

`e2e/playwright.config.ts:17` defaults to `http://127.0.0.1:8080`, while the shipped binary evidence runs use `8799` (`live-app.spec.ts:6`, `trace-hops.spec.ts:14`); running without `E2E_BASE_URL` targets a different port/possibly stale app, and the suite does not verify the server binary hash/asset identity. Evidence: config line + final run commands.

**Recommended action:** Default to the shipped port or read it from a single env manifest; add a preflight assertion comparing `/health` version/build to the expected artifact.

## S8 — Static validator uses lower-bound thresholds (LOW)

`verify-harness.mjs:56-59` accepts `frozenCommands>=10`, `events>=12`, `refusals>=30` rather than exact expected sets, so contract vocabulary could shrink/drift while checks still pass; `section()` relies on exact markdown headings (`verify-harness.mjs:30-36`). Evidence: file lines.

**Recommended action:** Pin expected exact counts/sets from the contract revision (e.g. 14/21/46) or compare parsed sets to a checked-in expected JSON.

## S9 — Tracked binary/runtime artifacts in git (LOW)

`tools/bin/hiverouter.exe` (9,329,152 B) and `runtime-test/data/hiverouter.db*` are tracked under this partition (confirmed by `git ls-files` and manifest); binary provenance is build-time only (sha recorded in QA docs) and the DB contains live runtime state that will churn on each run. Evidence: manifest rows + tracked status in FILES metadata.

**Recommended action:** Store binaries as release artifacts (not git) with checksum, and gitignore/regenerate runtime DBs; keep only deterministic fixtures under version control.

## S10 — Evidence writes into product tree during tests (LOW)

Specs write evidence JSON/screenshots into `docs/trace/live/` (`live-app.spec.ts:26,35,47,...`; `trace-hops.spec.ts:76-87`), so running the suite dirties the repo with untracked artifacts (observed `15-data-size.json`, `14-file-tree.json` untracked at closeout). Evidence: file lines + git status at closeout.

**Recommended action:** Write evidence to `test-results/`/a configurable evidence dir or ensure `docs/trace/live/**` generated files are gitignored with a manifest.

## S11 — No unit tests for QA tooling (INFO)

`tools/qa/verify-harness.mjs`, `build-trace-seeds.mjs`, `apply-live-trace.mjs` have no unit tests; their correctness is only evidenced by running against the current tree (27/27 pass). Evidence: file list and absence of test files in partition.

**Recommended action:** Add fixture-based tests for parser/validator edge cases (missing headings, partial capture, evidence-less verdicts).

## S12 — Host-pinned browser channels (INFO)

Browser policy uses host-installed Chrome/Edge (`playwright.config.ts:2-8`) with versions recorded only in comments; a different host can silently change rendering/behaviour while the suite still passes. Evidence: config comments; closeout Chrome 156.0.8078.75/Edge 156.0.4314.58.

**Recommended action:** Capture browser version in the evidence JSON and assert the expected major/runtime policy at run start.

## S13 — noWebServer/one-worker trade-off (INFO)

`e2e/playwright.config.ts:10-14,25` intentionally starts no server and forces 1 worker; this is honest and deterministic but makes the suite dependent on an externally started artifact and serial (3.6 min for 93 tests). Evidence: config lines + final run log.

**Recommended action:** Keep external start; consider sharding per file only if WS/event ordering requirements allow.

## Provenance / donor evidence

- The shipped HTTP+WS app under test is `tools/bin/hiverouter.exe` (PE, 9,329,152 B, sha256 `184D51A7…C9A9` at the final run); served UI asset at closeout was `index-Cbfm1IuM.js`. This is a compiled artifact, not source; its provenance is the repo's release build, verified only by hash + live behaviour.
- `runtime-test/workspace/{note.txt,README.md}` are fixture inputs for the Data panel (20 B / 44 B); `runtime-test/data/hiverouter.db` is generated state (schema_version 3 observed in /health). No provenance beyond the runtime session is recorded in the tree.
- No third-party source vendoring is present inside this partition's tracked 122 files; the ignored `tools/w64devkit/**` tree (9,452 files, 717 MB) is vendored toolchain intentionally excluded from the 507-file audit manifest per wave sheet §4.

## Migration-roadmap candidates (read-only suggestions)

1. Quarantine/delete ad-hoc e2e probe scripts and keep one evidence generator (S1).
2. Add fail-closed env gate + durable run receipts (S2/S3).
3. Add capture-integrity + S-APPROVAL coverage and fix S21 under-claim (S4/S5).
4. Decouple tests from fixture literals and product-tree evidence writes (S6/S10).
5. Pin exact contract vocabulary and artifact identity in preflight (S7/S8/S12).
6. Move generated binaries/DBs out of git (S9).


## Runtime-artifact census addendum (measured, `runtime-test/**` + `tools/**` binaries)

82 of the 122 assigned files are tracked runtime artifacts (category `runtime-artifact`), ~34 MiB in total; they are generated by the server/QA sessions, not authored source. Measured breakdown:

| Class | Count | Notes / hashes |
|---|---:|---|
| PE executables (`.exe`) | 8 tracked | `runtime-test/pkg-cli-final/hiverouter.exe`, `runtime-test/pkg-final/hiverouter.exe`, `tools/bin/hiverouter.exe` are **byte-identical** (sha256 `184D51A7C3CCA7B9D50954B677C5AD8A14472700A651FDF0D270441BE3EAC9A9`, 9,329,152 B); `runtime-test/pkg-smoke/hiverouter.exe` is an older build (sha256 `71B229B5…C38FC`, 9,099,776 B); `hiverouter-cli.exe` appears twice identical (sha256 `51E1ABA6…B9AD`, 5,364,224 B); `tools/cc-test/{r,t}.exe` add two more. |
| SQLite DB sets (`.db` + `-shm` + `-wal`) | 9 dirs / 27 files | `runtime-test/data{,2..9}`, `pkg-*/data`, `pkg-final/smoke-data`; WALs 94–185 KB carry live run/event state; DB provenance is runtime session only. |
| Logs/outputs (`.out/.err/.txt`) | ~20 | `s5..s9`, `server{,2,3,4}`, `pkg-*/out|err|smoke.*`; 12 are zero-byte `.err` files — leftover capture scaffolding with no recorded failure text. |
| JSON snapshots | 3 | 271 B snapshot payloads under `data4/data6/data9/snapshots/<uuid>/0000000000.json`. |
| Docs fixtures/mirrors | 8 | `pkg-*/README.md`, `RUNBOOK.md`, `CHECKSUMS.txt` (release-package mirrors) + `workspace/{README.md,note.txt}` (44 B/20 B fixtures used by live tests). |

**Provenance finding (extends S9):** the repository ships the same production server binary three times plus a stale build and duplicate CLI, and keeps 34 MiB of session DB/WAL/snapshot/log state under version control. `CHECKSUMS.txt` inside the pkg dirs only covers the package copies, so there is no single authoritative artifact identity; QA had to assert the shipped hash externally (`184D51A7…`). Runtime DB/WAL bytes are non-deterministic per session and will churn if the server is re-run from the tree even though the source did not change.

**Evidence commands:** `git ls-files` (tracked set), manifest bytes/sha256 re-hash (122/122 match), per-`.exe` sha256 grouping, `runtime-test/**` size/count aggregation.

## Accepted limitations (as required by wave sheet §4)

`tools/w64devkit/**` (9,452 files / 717 MB), `web/node_modules` (3,976 / 110 MB), `e2e/node_modules` (194 / 18 MB), `target/` (13,525 / 8,087 MB), `dist/` and `.git/` are ignored/generated classes and were **not** prose-analysed; only the 122 tracked assignment files above are in scope. Binary contents (`.exe`, `.db`, `.db-wal`) were hashed and classified, not decompiled; behaviour claims about them are UNVERIFIED at the byte level.
