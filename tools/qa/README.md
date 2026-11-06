# tools/qa — QA harness checks and the one-command gate

These scripts run **now**, before/immediately after the app exists, and never fabricate execution evidence.
`run-gates.ps1` is the single no-paid-by-default release gate; the others are its components and read-only probes.

| Script | What it proves | Run |
|---|---|---|
| `run-gates.ps1` | **One-command gate:** clean-tree/version/tag/manifest preflight → docs-truth → cargo workspace tests → web typecheck/tests/build → guarded e2e → static harness → strict key scan → manifest/artifact hashes. No paid call unless `-Live`. | `powershell -NoProfile -ExecutionPolicy Bypass -File tools/qa/run-gates.ps1` |
| `check-docs-truth.mjs` | Docs vs product truth: version parity, real-SSE streaming claim (no retired one-delta claim), no superseded 1.0 hashes as current, 16-state parity; historical sections allowed. | `node tools/qa/check-docs-truth.mjs` |
| `verify-harness.mjs` | 28-surface matrix integrity, frozen command/event/refusal vocabulary cross-check against `docs/contracts/UI-COMMANDS-v1.md`, 11-hop names, pre-evidence statuses only, trace seeds valid and PENDING. | `node tools/qa/verify-harness.mjs` |
| `build-trace-seeds.mjs` | Regenerates `docs/trace/controls.json` / `traces.json` / `events.json` from the matrix (all PENDING; empty runs). | `node tools/qa/build-trace-seeds.mjs` |
| `figcraft-assets.mjs` | Exact FigCraft-A export inventory (file/size/sha256) + manifest/A-design presence (read-only). | `node tools/qa/figcraft-assets.mjs` |
| `check-env.ps1` | Host/toolchain/pin/browser/disk/project-state evidence (read-only). | `powershell -NoProfile -ExecutionPolicy Bypass -File tools/qa/check-env.ps1` |

## run-gates.ps1

One command, ordered, fail-closed. Every step logs its name, exit code, duration and status
(`PASS` / `FAIL` / `SKIPPED` / `REHEARSAL`); HEAD is logged at start and re-checked at the end. Printed paths are
repo-relative; the script writes no tracked files (cargo/web/e2e write only ignored build/test dirs) and never
prints a key value.

**Steps:** G0 preflight (clean tree, web/crate version parity, tag + `tools/release-manifest.json` freshness,
toolchain) → G1 `check-docs-truth.mjs` → G2 `cargo test --workspace --locked` → G3a web typecheck → G3b web unit
tests → G3c web production build → G4 guarded Playwright e2e → G5 `verify-harness.mjs` → G6 strict key scan
(`sk-[A-Za-z0-9]{32,}` over `git ls-files` text files, near-misses only warned) → G7 manifest/artifact verification
(artifact sha/bytes, package zip/sidecar/CHECKSUMS hashes and members, web dist files and tree hash).

**Flags:**

| Flag | Meaning |
|---|---|
| `-DryRun` | Resolve and print the plan + preflight only; executes nothing, writes nothing. Exit reflects whether the preflight would allow a real run. |
| `-Rehearsal` | Allows a dirty tree and a version-mismatched manifest for pre-seal rehearsals; printed as `REHEARSAL`; real defects still fail. Without it those conditions are hard failures. |
| `-Live` | Explicit **paid** opt-in: requires `E2E_APP_READY=1`, `E2E_BASE_URL` and a present `DEEPSEEK_API_KEY` (never printed), then sets `LIVE_ONE_PROMPT=1`. Without `-Live`, `LIVE_ONE_PROMPT=0` is forced so paid specs skip by name. |
| `-SkipE2E` | Skip guarded e2e by name (recorded `SKIPPED`, never counted as a pass). |

**E2E guard:** with no `E2E_APP_READY=1` + `E2E_BASE_URL` the guarded Playwright step is `SKIPPED` with a named
reason; it never starts an app and never treats an all-skip run as a pass (`E2E_ALLOW_ALL_SKIPS` is never set by
this script). `-Live` additionally requires the app already running and a key present.

**G7 is a drift detector, not a rebuild step:** it runs after `G3c` regenerates `web/dist`, so if any uncommitted
source change (including another in-flight wave's edit) rebuilds a different bundle than the sealed manifest records,
G7 fails with the exact file/hash mismatch. That is the correct signal to re-seal the manifest; it is not a bug.

**Exit codes:** `0` = all gates pass (named skips allowed), `1` = one or more gates failed, `2` = setup/file error
(missing toolchain or required file; no gate claimed as passed).

Examples:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools/qa/run-gates.ps1 -DryRun    # rehearse the plan
powershell -NoProfile -ExecutionPolicy Bypass -File tools/qa/run-gates.ps1 -Rehearsal # full no-paid run mid-wave
powershell -NoProfile -ExecutionPolicy Bypass -File tools/qa/run-gates.ps1            # strict release gate
# paid live streaming/replay proof (operator only, app already running, key present):
$env:E2E_BASE_URL='http://127.0.0.1:8799'; $env:E2E_APP_READY='1'
powershell -NoProfile -ExecutionPolicy Bypass -File tools/qa/run-gates.ps1 -Live
```

Exit codes for the component scripts: `verify-harness.mjs` / `check-docs-truth.mjs` → 0 pass, 1 check failure,
2 parse/file error. No script writes outside `docs/trace/` (`build-trace-seeds.mjs` only) and none touches the
blueprint. `e2e/` runs only when explicitly included by `run-gates.ps1` guards.
