# SEAT 8 — INDEPENDENT RELEASE GATE AUDIT (HiveMind 1.1 dirty tree)

- **Scope:** read-only; only this file written. Repo `C:\Users\Riley\Desktop\hiverouter-v1`; **HEAD `c9390f5`**, branch `master`, dirty tree (27 modified + 12 untracked). No product edits, no commits/tags, no key access, no paid calls.
- **Verdict: BLOCKED** for a HiveMind 1.1 release gate. The 1.0.0 release remains intact (artifacts/tags verified), but the 1.1 tree cannot pass a gate today: unsealed dirty tree with untracked tests, two remaining absolute-path scripts, no 1.1 artifact/HEAD-bound evidence, and residual doc-count drift.
- **Skills used (all five, mandatory):** `forgewright-recon` (§Recon ledger), `forgewright-lead` (read code/callers/tests before claims; VERIFIED/READ/INFERRED/UNVERIFIED separation; no mutation), `logic-gatekeeper` (§Gate JSON — used as a **refusal gate**, no mutating edit performed), `integration-logic` (call chain provider→core→WS→UI→docs + refusal boundaries), `logic-debugger` (seal/hash/log-binding invariants and the dirty-tree violation).

## Recon (forgewright-recon ledger)

- **Target behaviour:** a releasable 1.1.0 build whose version, docs, paths, key handling, e2e guards, trace counts and artifacts all agree and are evidenced on a sealed tree.
- **Entry points:** `backend/crates/hiverouter-core/src/main.rs:99-112` (boot checks + key presence), `checks.rs:140-183` (`provider_key_check`), `provider_executor.rs:192-201` (finish_reason truncation → `Refusal::ProviderIncomplete`), `.cargo/config.toml` (toolchain paths), `e2e/package.json` + `e2e/run-guarded.mjs` (default e2e), `docs/trace/controls.json` (trace truth), `tools/release-manifest.json` (artifact truth).
- **Callers:** `main.rs` → `checks::provider_key_check`; `provider_executor.rs` → `Refusal::ProviderIncomplete` (`contract/refusal.rs:263` `E_PROVIDER_INCOMPLETE`); `main.rs:105`/`tools/cli.rs:480`/`settings.rs:22` → `ApiKey::present_in_env` (`provider/secret.rs:61`); Playwright default script → `run-guarded.mjs`; docs claims → `controls.json`/manifest.
- **Tests + commands:** `cargo test` (log at `docs/evidence/1.1-backend-tests.log`); `e2e/package.json` `e2e` script → `node run-guarded.mjs`; web vitest (`web/tests/*`); docs checker `tools/qa/check-docs-truth.mjs` (untracked).
- **Data flow (success/failure):** `DEEPSEEK_API_KEY` env → `ApiKey::present_in_env()` → **bool only** → `provider_key_check` → boot `Check` (never the value) → logs/`/health`; provider stream → `finish_reason=="length"` → `Refusal::ProviderIncomplete` (`E_PROVIDER_INCOMPLETE`, no fake success).
- **Unknowns:** no clean sealed 1.1 commit; backend log not HEAD-bound; no 1.1 artifact; no 1.1 web/e2e run; two scripts still absolute-path; no cargo build run in this audit.
- **Files inspected:** `.cargo/config.toml`, `checks.rs`, `main.rs`, `provider_executor.rs`, `provider_streaming.rs`, `refusal.rs`, `secret.rs`, `e2e/package.json`, `e2e/run-guarded.mjs`, `e2e/playwright.config.ts`, `README.md`, `docs/RELEASE*.md`, `docs/trace/controls.json`, `docs/claims/CLAIM-REGISTER.*`, `docs/reports/FINAL-REPORT.md`, `tools/build_assets_index.py`, `tools/qa/check-env.ps1`, untracked tests/logs (read-only).

## Gate results

| # | Gate item | Result | Exact evidence |
|---|---|---|---|
| 1 | Version consistency | **PASS** | `hiverouter-{contract,core,provider,tools}/Cargo.toml` + `web/package.json` + `web/package-lock.json:3,9` = `1.1.0`; `backend/Cargo.lock:982,995,1021,1040` = `1.1.0`; `cargo metadata --no-deps` exit **0** (workspace `1.1.0`). `tools/release-manifest.json` still records the sealed **1.0.0** artifacts (`source_commit 6a2b1c6`) — correct/expected, 1.1 artifacts do not exist yet (README states this). |
| 2 | No `C:/Users` absolute paths | **FAIL (BLOCKED item)** | `.cargo/config.toml` **fixed** (`CC/AR/linker = "gcc"/"ar"`, no absolute path; `cargo metadata` exit 0). But tracked scripts still hardcode the host: `tools/build_assets_index.py:6` `ROOT = Path(r"C:\Users\Riley\Desktop\hiverouter-v1")`; `tools/qa/check-env.ps1:34,43` `C:\Users\Riley\...` + `C:\Users\Riley\Desktop\hivemind-blueprint`. Tracked `tools/bin/*.exe` also embed the path string (generated artifacts). Claim "no C:/Users absolute paths" is **not true** as stated. |
| 3 | Key scan | **PASS** | Strict credential regex `sk-[A-Za-z0-9]{32,}` over 677 tracked non-vendor files = **0 hits**; no key material in manifest/evidence/logs (0 hits on those files). Key-check boundary is bool-only: `provider_key_check(enabled,present)` (`checks.rs`), `ApiKey::present_in_env()` returns `bool` (`secret.rs:61`), `main.rs:105` passes only the bool; no value read/logged. `1.1-backend-tests.log`: `provider_key_check_is_optional_and_never_reads_the_value ... ok`. Residual: canary test `tests/recovery_gates.rs` is **untracked**; broad literal `sk-` fixtures exist (non-secret test values). |
| 4 | Docs conflict (README non-streaming) | **RESOLVED at HEAD** | Root `README.md` is now **1.1.0**: `:24-25` version agreement, `:26-30` "Streaming is real in 1.1.0… ordered REPLY_DELTA frames… pushed to the UI as they arrive; retired 1.0.0 one-delta behavior is historical only", `:41-43` known limits (no sealed-artifact streaming smoke). `:47-54` historical 1.0.0 section covers the old single-delta behavior. `docs/RELEASE.md:21` / `docs/RELEASE-EVIDENCE.md:37` retain the one-delta limitation only as the **1.0** record. No current-facing non-streaming claim found. |
| 5 | e2e default scripts guarded | **PASS (read; not run)** | `e2e/package.json`: `"e2e": "node run-guarded.mjs"`, `e2e:chrome`/`e2e:msedge`→guarded; `"e2e:raw": "playwright test"` intentionally unguarded. `e2e/run-guarded.mjs`: runs Playwright then **exits 1 when `passed==0 && failed==0 && skipped>0`** unless `E2E_ALLOW_ALL_SKIPS=1` (named skip-guard). `playwright.config.ts`: installed Chrome/msedge channels only, no browser download, `forbidOnly: true`. Residual: guard catches only all-skip, not partial-skip; no run evidence for 1.1. |
| 6 | Release artifacts/tag still 1.0 | **PASS** | `target/release/hiverouter.exe` 13,253,120 B `B17EB3D6…062A9A`; CLI 5,373,952 B `49F46E2B…386623`; `dist/HiveMind-1.0.0-win64.zip` 8,444,021 B `46305FCB…6105EF` — all identical to the 1.0.0 release record; tags `hivemind-1.0.0`=`23ba74a`, `v1.0.0`=`3988622` unchanged; HEAD `c9390f5` (1.1 work) is **not** tagged. No 1.1 artifact/tag exists. |
| 7 | Trace counts | **PASS (current) / docs residual** | `docs/trace/controls.json`: **49 WIRED-AND-WORKS / 264 PENDING / 5 MISSING / 1 UNVERIFIED of 319 hops**; `docs/trace/live/README.md:4,32,58` + `QA-HARNESS.md:4` match and label 43/270/6 as superseded. Residual mismatch: `docs/reports/FINAL-REPORT.md:3` supersession block + `docs/claims/CLAIM-REGISTER.json:80` notes still say **270 PENDING / 6 MISSING (43 W&W)** — stale vs current controls. |
| 8 | Truncated-reply claim | **VERIFIED (logged)** | `provider_executor.rs:195-201` returns `Refusal::ProviderIncomplete` when `finish_reason=="length"`; refusal maps to `E_PROVIDER_INCOMPLETE` (`contract/refusal.rs:263`); unit test `provider_streaming.rs` `length_finish_reason_is_incomplete_not_success`; `1.1-backend-tests.log` shows `test length_finish_reason_is_incomplete_not_success ... ok`. |
| 9 | Key-check claim | **VERIFIED (logged; canary untracked)** | New `provider_key_check` optional boot check (`checks.rs:140-183`), wired in `main.rs:102-108`; tests in `checks.rs` + untracked `tests/recovery_gates.rs` (canary `sk-w1-canary-never-emit-9f3c1b7e` asserts no secret in serialized `Check`); log shows provider-key test **ok**. Untracked test file is a residual seal defect. |
| 10 | Config changes (.cargo) | **PARTIAL** | `.cargo/config.toml` now relative (`gcc`/`ar`/linker) — read + `cargo metadata` exit 0 proves the config parses (UTF-8 BOM accepted). Build not executed (no compile run in this audit); `gcc`/`ar` resolution depends on PATH (`tools/w64devkit/bin`); **UNVERIFIED** that `cargo build` succeeds from a clean PATH. |
| 11 | Sealed-tree requirement | **BLOCKED** | `git status`: **27 modified + 12 untracked**, including untracked test files `backend/crates/hiverouter-core/tests/recovery_gates.rs`, `backend/crates/hiverouter-provider/tests/error_matrix.rs`, `web/tests/apiLifecycle.test.ts`, `web/tests/streamingRender.test.ts`, plus `tools/qa/check-docs-truth.mjs`, trace capture files and wave records. No commit/tag for 1.1; 1.1 evidence cannot be bound to a sealed tree. |
| 12 | 1.1 evidence completeness | **PARTIAL / UNVERIFIED** | `docs/evidence/1.1-backend-tests.log` exists (47,508 B, 15:55:21) with `EXIT=0`, **263 passed / 0 failed / 2 ignored in 29 suites** (sums recomputed), including truncation + key-check + child-key-isolation tests. But the log carries **no HEAD=/COMMIT= binding line** (grep found none) → tree binding UNVERIFIED; **no 1.1 web test log, no 1.1 e2e run, no 1.1 artifact**. |

## Logic-gatekeeper Phase-1 JSON (refusal gate — no mutation performed)

```json
{
  "phase1": {
    "architecture": "Dirty working tree on master at c9390f5: backend crates (contract/core/provider/tools) at 1.1.0, web at 1.1.0, e2e harness, docs/claims/trace docs, release artifacts/manifest still 1.0.0, tags 1.0.0.",
    "dataFlow": "DEEPSEEK_API_KEY -> ApiKey::present_in_env (bool only) -> provider_key_check -> boot Check -> logs/health; provider SSE -> finish_reason==length -> Refusal::ProviderIncomplete -> E_PROVIDER_INCOMPLETE; docs claims -> controls.json/manifest/README.",
    "edgeCases": ["finish_reason=length with partial fragments (tested, logged)", "missing key + enabled provider -> E_KEY_MISSING, required=false (tested via provider_key_check test)", "all-skipped e2e run -> guarded exit 1 unless E2E_ALLOW_ALL_SKIPS=1 (read)", "build from clean PATH where gcc/ar are not present (unverified)"],
    "rootCause": "The 1.1 tree is pre-seal work-in-progress: uncommitted/untracked tests and docs, two tracked scripts still host-absolute, and no HEAD-bound 1.1 artifact or e2e/web evidence — so release-gate claims cannot be proven even where the code changes themselves look correct."
  }
}
```

No mutating edit was made (write scope is this audit file only), so Phase 2 does not apply.

## Integration-logic (call chain + refusal boundaries)

`SUBMIT_PROMPT` → run engine → `ProviderExecutorAdapter::execute` (stream_reply) → DeepSeek `/chat/completions` SSE → `REPLY_DELTA` fragments → usage from final chunk → `REPLY_DONE` → UI. Seam checks: malformed/incomplete/truncated stream → named `E_PROVIDER_INCOMPLETE` (no fake success); missing key at boot → optional `deepseek_api_key` check with `E_KEY_MISSING` (server boots degraded, `required=false`); child tool processes get an allowlisted env (no key inheritance, logged test ok); docs claims must trace to `controls.json`/manifest — two stale doc paths found (§7 residual). No untested seam was treated as working.

## Logic-debugger (invariants attacked)

- **Invariant 1 — sealed tree before release claims:** violated (27M+12??, no 1.1 commit/tag). Evidence: `git status --porcelain`.
- **Invariant 2 — evidence is bound to the tree it claims to test:** 1.1 backend log has no HEAD/COMMIT line → violated/UNVERIFIED; prior 1.0 logs were HEAD-bound (contrast `docs/evidence/final-gate-tests.log` has `HEAD=6a2b1c6…`).
- **Invariant 3 — no host-absolute paths in tracked code/scripts:** violated by 2 scripts (+ generated binaries), fixed in `.cargo/config.toml`.
- **Invariant 4 — docs agree with the product's current capability:** README now truthful (streaming 1.1, historical 1.0), trace docs current; two stale doc counts remain (FINAL-REPORT supersession, claims JSON notes).
- **Invariant 5 — 1.0 release unchanged:** holds (hashes + tags unchanged).
- No defect was "fixed" to make a gate pass; no test weakened.

## Ranked residual list (must-fix order)

| Rank | Residual | Severity | Evidence | Fix owner |
|---|---|---|---|---|
| R1 | Dirty tree: 27 modified + 12 untracked incl 4 untracked test files (`recovery_gates.rs`, `error_matrix.rs`, `apiLifecycle.test.ts`, `streamingRender.test.ts`) and `tools/qa/check-docs-truth.mjs` | **Blocker** | `git status --porcelain` | 1.1 owner: commit/label the wave, then re-run gate on clean tree |
| R2 | 1.1 evidence not HEAD-bound; no 1.1 artifact, no 1.1 web/e2e evidence; backend log unbound (`EXIT=0`, 263/0/2 recomputed) | **Blocker** | log grep `HEAD=`/`COMMIT=` absent; manifest still 1.0.0; `docs/evidence/` has only 1.0 web/build logs | Re-run `cargo test` on committed HEAD; add web vitest + guarded e2e logs; seal build |
| R3 | Absolute paths remain in `tools/build_assets_index.py:6`, `tools/qa/check-env.ps1:34,43` (+ embedded in `tools/bin/*.exe`) | **Blocker if claim is literal "no C:/Users"** | quoted lines above | Tools owner: derive repo root relatively (`$PSScriptRoot`/`__file__`/`git rev-parse --show-toplevel`) |
| R4 | Stale trace counts in `docs/reports/FINAL-REPORT.md:3` supersession block and `docs/claims/CLAIM-REGISTER.json:80` notes (43/270/6) vs current 49/264/5/1 | Medium (doc truth) | `controls.json` parse; quoted docs lines | Docs owner (W8/W9): correct to 49/264/5/1/319 |
| R5 | `.cargo/config.toml` relative `gcc`/`ar` unproven for a real build (metadata parses exit 0; no compile run) | Medium (verification) | `cargo metadata` exit 0; no build executed | Release owner: `cargo build --release --locked` from clean PATH and record log |
| R6 | Untracked canary test `recovery_gates.rs` not committed → key-check proof lives outside VCS | Medium | untracked file 4,445 B | 1.1 owner: include in wave commit |
| R7 | e2e guard only catches all-skip (not partial skips); `e2e:raw` bypasses guard; no 1.1 e2e run | Low (read-verified) | `run-guarded.mjs`, `e2e/package.json` | QA owner: run guarded e2e on sealed 1.1 artifact |
| R8 | Broad literal `sk-` strings remain only as non-secret test fixtures (no credential) | Info | `client.rs`/`secret.rs`/`refusal_matrix.rs`/`shell.rs` test values | None (document as non-secret fixtures) |

## Evidence appendix (commands and outputs)

```
git rev-parse HEAD                         -> c9390f5a6a57168ecd2613f4095b871d58d1a7c3
git status --porcelain                     -> 27 M + 12 ?? (listed in R1); tags: hivemind-1.0.0=23ba74a, v1.0.0=3988622
cargo metadata --no-deps --offline (backend) -> exit 0; hiverouter-contract 1.1.0 (workspace 1.1.0)
.cargo/config.toml                         -> CC/AR/linker = gcc/ar (no C:/Users)
tools/build_assets_index.py:6              -> ROOT = Path(r"C:\Users\Riley\Desktop\hiverouter-v1")
tools/qa/check-env.ps1:34,43               -> "cd /d C:\Users\Riley\Desktop\hiverouter-v1" / blueprint root
strict key scan (sk-[A-Za-z0-9]{32,}, 677 tracked non-vendor files) -> 0 hits
provider_executor.rs                       -> finish_reason=="length" => Err(Refusal::ProviderIncomplete) (E_PROVIDER_INCOMPLETE)
docs/evidence/1.1-backend-tests.log        -> EXIT=0; 263 passed / 0 failed / 2 ignored (29 suites, recomputed); includes
                                              length_finish_reason_is_incomplete_not_success ... ok;
                                              provider_key_check_is_optional_and_never_reads_the_value ... ok;
                                              child_process_does_not_inherit_the_provider_key ... ok
docs/trace/controls.json                   -> 49 WIRED-AND-WORKS / 264 PENDING / 5 MISSING / 1 UNVERIFIED (319 hops)
target/release/hiverouter.exe              -> b17eb3d6…062a9a (13,253,120 B)
target/release/hiverouter-cli.exe          -> 49f46e2b…386623 (5,373,952 B)
dist/HiveMind-1.0.0-win64.zip              -> 46305fcb…6105ef (8,444,021 B)
README.md:1,24-30,41-54                    -> v1.1.0; streaming real; 1.0 one-delta only under historical section
e2e/package.json + run-guarded.mjs         -> default `e2e` runs guarded wrapper; all-skip => exit 1 unless E2E_ALLOW_ALL_SKIPS=1
```

## What was NOT verified

- No build/compile and no fresh test execution was run during this audit (10-minute read-only gate; `cargo build` would mutate `target/` and require a longer window). All code claims above are source-read or quoted from `1.1-backend-tests.log`, whose tree binding is **unverified** (no HEAD/COMMIT line).
- No 1.1 artifact exists to hash; 1.0.0 hashes were re-verified and are unchanged.
- e2e/web 1.1 behaviour was not executed; guards were verified by reading only.
- Live provider/key calls were not attempted (no key access/paid calls permitted).
