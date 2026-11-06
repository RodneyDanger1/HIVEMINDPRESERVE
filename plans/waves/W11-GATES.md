# W11 — ONE-COMMAND GATE HARNESS (seat C)

Repo `C:\Users\Riley\Desktop\hiverouter-v1` · HEAD `9651722` (`hivemind-1.1.0` + 2 docs commits) · tree clean.
Write scope: new `tools/qa/run-gates.ps1`, `tools/qa/README.md` (+ this plan file). No commit.
Skills applied: `/forgewright-recon`, `/forgewright-lead`, `/logic-gatekeeper`, `/integration-logic`, `/logic-debugger`.

## Recon (forgewright-recon; path:line evidence)

- **Release state:** tag `hivemind-1.1.0` → `87079ce…`; `tools/release-manifest.json` schema
  `hivemind.release-manifest.v1`, version `1.1.0`, source_commit/tag_target `87079ce…`, artifacts
  `hiverouter.exe` `9c3174fb…` 13,624,320 B, `hiverouter-cli.exe` `f757f558…` 5,372,416 B, package
  `dist/HiveMind-1.1.0-win64.zip` `af2345f6…` 8,613,122 B, sidecar/checksums hashes + contents list, web
  `tree_sha256 7911b91c…`, per-file web hashes, `supersedes` 1.0.0 rows. `dist/HiveMind-1.1.0-win64.zip` and
  `target/release/*.exe` exist on disk; hashes match the manifest (VERIFIED by `Get-FileHash` in recon).
- **Version truth:** `web/package.json` version `1.1.0`; all four `backend/crates/*/Cargo.toml` `1.1.0`. HEAD is
  3 commits ahead of the sealed tag (post-release evidence/docs commits only) — not a stale manifest condition.
- **Existing verifiers (all green at recon):** `node tools/qa/check-docs-truth.mjs` → PASS (exit 0);
  `node tools/qa/verify-harness.mjs` → 27/27 checks, exit 0; `tools/qa/check-env.ps1` is a read-only env probe;
  `tools/test.ps1` is an older backend/web-only wrapper (per-crate cargo + web typecheck/test, no docs/hash/e2e/key
  gates and no dirty/manifest guard).
- **Test commands:** cargo workspace tests from `backend/` (`cargo test --workspace --locked`); web scripts in
  `web/package.json`: `typecheck` (`tsc --noEmit`), `test` (`vitest run`), `build` (`tsc --noEmit && vite build`);
  e2e guarded runner `e2e/run-guarded.mjs` (owns exit code, fails if all tests skip and none pass unless
  `E2E_ALLOW_ALL_SKIPS=1`) with `E2E_APP_READY=1` + `E2E_BASE_URL` required for real runs (`e2e/helpers/matrix.ts:43-50`);
  paid specs are gated by `paidGate()` = **`LIVE_ONE_PROMPT=1` AND `DEEPSEEK_API_KEY` non-empty**
  (`e2e/helpers/live-server.ts:112-132`), otherwise they skip by name.
- **Key scan:** no dedicated key-scan script exists (grep of `tools/**` for `sk-[` / credential patterns = 0 files);
  the documented strict pattern is `sk-[A-Za-z0-9]{32,}` over tracked non-vendor files (0 hits claimed in
  `docs/RELEASE-EVIDENCE.md`). Gate must implement it over `git ls-files`, skipping binary assets.
- **Node/npm/cargo:** node v26.7.0, npm 11.19.0, cargo/rustc 1.99.1 in `%USERPROFILE%\.cargo\bin` (not on PATH by
  default), gcc 16.2.0 in `tools/w64devkit/bin`; Playwright installed in `e2e/node_modules`; web/e2e `node_modules`
  present.
- **Unknowns:** whether an app server is running on `127.0.0.1:8799` during a run (guarded e2e must skip by name,
  never fake a pass); full cargo/web gate wall time (long-running → dry-run + real run with generous timeout).

## Phase 1 (logic-gatekeeper — written BEFORE any mutation)

```json
{
  "phase1": {
    "architecture": "One PowerShell orchestrator (tools/qa/run-gates.ps1) over existing verifiers: Node docs-truth checker, cargo workspace tests, web typecheck/test/build, guarded Playwright runner (e2e/run-guarded.mjs), static QA harness, tracked-tree key scan, and release-manifest/artifact hash verification. It holds no product logic; it resolves the repo root, toolchain, current version, manifest and guards, then aggregates step statuses into one exit code.",
    "dataFlow": "Repo state (HEAD/tag/dirty, web/package.json + crate versions, tools/release-manifest.json, dist artifacts, web/dist) and env guards (E2E_APP_READY, E2E_BASE_URL, LIVE_ONE_PROMPT, DEEPSEEK_API_KEY presence) -> run-gates preflight (dirty/stale gates) -> per-step exit codes + durations -> summary + exit 0/1/2 consumed by humans/CI; no writes to tracked files (web build writes ignored web/dist; e2e writes ignored test-results).",
    "edgeCases": [
      "Dirty working tree or version-mismatched manifest must fail closed unless -Rehearsal (then marked rehearsal, never silently passed).",
      "No app server: E2E_APP_READY missing means the guarded e2e step is SKIPPED by name (not PASS); all-skip output must not be counted as a pass.",
      "-Live must require E2E_APP_READY=1 + E2E_BASE_URL + a present DEEPSEEK_API_KEY and set LIVE_ONE_PROMPT=1; without -Live those variables must be cleared/forced off so no paid call can happen.",
      "One gate failure must not abort the remaining gates (report all, exit 1 at the end); missing toolchain/file is a setup error (exit 2), not a pass.",
      "Manifest whose tag/source_commit/artifact hashes disagree with disk must fail the artifact gate even when version strings match; a stale 1.0.0 manifest must fail non-rehearsal runs."
    ],
    "rootCause": "Release evidence is currently produced by several disconnected manual commands (backend/web wrapper, guarded e2e, docs/harness checks, manifest hashes, key scan) with inconsistent guards, so a release can be called green without a single reproducible pass that verifies version/doc truth, tests, guarded e2e skips, artifact hashes and key hygiene — and paid live specs can be triggered without an explicit opt-in; one no-paid gate command with fail-closed rehearsal flags and an explicit -Live switch resolves that."
  }
}
```

## Decisions

- **D1 Single command, no paid calls by default:** `pwsh -NoProfile -ExecutionPolicy Bypass -File tools/qa/run-gates.ps1`.
  Steps in order: (G0) preflight (clean tree, version, tag/manifest freshness, toolchain) → (G1) docs-truth checker →
  (G2) `cargo test --workspace --locked` → (G3) web typecheck/test/build → (G4) guarded e2e (skips by name when the app
  is not ready) → (G5) `verify-harness.mjs` → (G6) strict tracked-tree key scan → (G7) manifest/artifact hash
  verification → summary. Exit 0 all pass (skips named), 1 any gate failed, 2 setup/file error.
- **D2 Fail-closed rehearsals:** `-Rehearsal` permits a dirty tree and a version-mismatched manifest for pre-seal
  rehearsals; every such run prints `REHEARSAL` and still fails real defects. Without it, dirty tree / stale manifest
  are hard failures.
- **D3 Live opt-in:** `-Live` requires `E2E_APP_READY=1` + `E2E_BASE_URL` set + `DEEPSEEK_API_KEY` present (boolean
  only, never printed), sets `LIVE_ONE_PROMPT=1` for the guarded runner and states "paid provider proof authorized".
  Without `-Live`, the script sets `LIVE_ONE_PROMPT=0` (paid specs self-skip) and never reads the key value.
- **D4 Honest statuses:** every step logs `name`, command (repo-relative), duration, exit code, and status
  PASS/FAIL/SKIPPED(+reason)/REHEARSAL; HEAD is logged once at start and re-checked at the end (must be unchanged).
  A skipped e2e due to no app is a named SKIP, never a PASS; the guard's all-skip refusal is respected
  (`E2E_ALLOW_ALL_SKIPS` is never set by this script).
- **D5 Key scan strict:** over `git ls-files`, binary/asset extensions skipped, text decoded UTF-8, fail on
  `sk-[A-Za-z0-9]{32,}`; report near-misses (`sk-[A-Za-z0-9]{16,31}`) as WARN only; no key value is ever printed.
- **D6 Manifest verify:** schema/version/current-tag/tag-target/source_commit checks; each artifact path exists with
  matching sha256/bytes; package zip/sidecar/checksums exist and hash-match; zip members match `package.contents` and
  packaged binaries match artifact hashes; web files match `web.files` and `tree_sha256`. `HEAD` ahead of the sealed
  source is reported as INFO (post-release commits), not stale.
- **D7 No absolute paths / no writes:** all printed paths repo-relative; the script writes no files (console only;
  the underlying build/e2e steps write only ignored `web/dist`, `e2e/test-results`, `backend/target`).

## File-by-file

| File | Change |
|---|---|
| `tools/qa/run-gates.ps1` | new: parameterized orchestrator (`-DryRun`, `-Rehearsal`, `-Live`, `-SkipE2E`) with G0–G7, per-step logging, summary, exit 0/1/2 |
| `tools/qa/README.md` | add `run-gates.ps1` row, usage/flags, step list, exit codes; keep existing rows |

## Failure modes handled

Dirty tree → FAIL (`E_GATE_DIRTY_TREE`) unless `-Rehearsal`; stale/missing manifest → FAIL (`E_GATE_MANIFEST_STALE`) unless
`-Rehearsal`; missing toolchain → setup exit 2; docs checker exit 2 → setup error; cargo/web/e2e/harness failure → FAIL
with exit code; artifact hash mismatch → FAIL; key pattern hit → FAIL with `file:line`; `-Live` without app/key → setup
FAIL before any run; all other gates still run after a failure.

## Tests

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools/qa/run-gates.ps1 -DryRun        # plan only, no execution
powershell -NoProfile -ExecutionPolicy Bypass -File tools/qa/run-gates.ps1 -Rehearsal     # full no-paid run in this dirty-free tree
# failure-path exercises (temp copies / env manipulation, no repo writes):
#   -Rehearsal omitted on a deliberately dirty tree -> E_GATE_DIRTY_TREE
#   -Live with no E2E_APP_READY -> setup FAIL (no paid call)
#   key scan test against a temp fixture containing a fake 40-char sk- token -> FAIL
```

## Rollback

Remove `tools/qa/run-gates.ps1`, revert `tools/qa/README.md` (`git checkout -- tools/qa/README.md`), delete this plan
file. No product/runtime files changed; no commit; no tag movement.

## UNVERIFIED / out of scope

- Full no-live step outcomes depend on the host (cargo/web wall time, app presence): quoted in the execution receipt;
  the guarded e2e step is expected to SKIP without `E2E_APP_READY=1`.
- `-Live` paid path is implemented and setup-guarded but **not executed here** (no paid call in this seat; requires
  operator key + running app).
- Key-scan near-miss WARN threshold is a hygiene aid, not a cryptographic proof.

## 9. Execution evidence (2026-11-06; no commit; no paid call)

| file | bytes | sha256 |
|---|---:|---|
| `tools/qa/run-gates.ps1` | 25563 | `4768e3c91db53973fc224605377e47337b6edf4463f9b9b60a300ca5d8426ef8` |
| `tools/qa/README.md` | 5081 | `6a0a6db0a26a44b8c377927c47ea8fe5e62ba2c76aff492b202544b235283a09` |

**T1 dry-run, strict (executes nothing):** `powershell -NoProfile -ExecutionPolicy Bypass -File tools/qa/run-gates.ps1 -DryRun`
→ printed the full G1–G7 plan, then `E_GATE_DIRTY_TREE: working tree is not clean (20 path(s))` and exit **1**
(dirty paths were other in-flight W11 seats, not this script's writes). Fail-closed tree gate proven; no files written.

**T2 full no-live path (rehearsal, guarded e2e skips without an app):**
`powershell -NoProfile -ExecutionPolicy Bypass -File tools/qa/run-gates.ps1 -Rehearsal` → exit **1**
with: G1 docs-truth PASS (5.1 s) · G2 `cargo test --workspace --locked` PASS (558.8 s, includes a build-directory
lock wait; all workspace test binaries + doc-tests executed) · G3a web typecheck PASS (17.3 s) · G3b web unit
tests PASS (18.6 s) · G3c web production build PASS (13.0 s) · G4 guarded e2e **SKIPPED** with the named reason
`E2E_APP_READY!=1 (no app server)` (correct: no paid spec ran, `LIVE_ONE_PROMPT=0`) · G5 harness PASS (0.2 s) ·
G6 strict key scan PASS (3.3 s, 605 tracked text files scanned, 0 strict hits) · G7 manifest/artifact verify **FAIL**
(1.7 s): `web dist file missing: web/dist/assets/index-DRvC4Qb-.js; web dist sha mismatch: web/dist/index.html;
web dist tree_sha256 mismatch`. HEAD start = HEAD end (`9651722`).

**G7 failure is truthful drift detection, not a harness bug (verified root cause):** the sealed manifest records
`index-DRvC4Qb-.js` 675,204 B / `index.html` `01f31347…`; after the gate's own `G3c` rebuild the tree produced
`index--R3q_MQv.js` 676,300 B / `index.html` `9b468c98…`. During the run another in-flight wave edited
`web/src/api/contract.ts` + `web/tests/contract.test.ts` (visible in `git status`), so the rebuilt bundle legitimately
differs from the sealed build. Evidence that the sealed commit itself is unchanged: `git diff --stat hivemind-1.1.0..HEAD -- web/ backend/`
is empty; the difference came from the uncommitted concurrent edit. A strict run on a quiescent tree must PASS G7;
on this mid-wave tree it must fail — the gate is doing its job.

**T3 paid opt-in guard (no paid call):** `-Rehearsal -Live` with no app/key → `[E_GATE_LIVE_SETUP] -Live requires
E2E_APP_READY=1 and E2E_BASE_URL for a running app` → exit **2**, `GATES: SETUP ERROR - no gate was claimed as passed`.
No provider call was made; the key value was never read/printed.

**Scope/no-write proof:** this seat wrote only `tools/qa/run-gates.ps1`, `tools/qa/README.md` and this plan file; the
gate run wrote only ignored `web/dist`/`target`/`test-results` artifacts; `git status` shows the other W11 seats'
in-flight files unchanged by this seat; **no commit** was made (HEAD stayed `9651722`).

**Residual:** a strict (non-rehearsal) all-PASS quote requires a quiescent tree at the sealed commit with the sealed
manifest; that is the owner/Lead run. G7 confirms the released manifest's web hashes still need to be reproduced
after the in-flight web contract work lands (re-seal or confirm the sealed bundle was built from committed source).
