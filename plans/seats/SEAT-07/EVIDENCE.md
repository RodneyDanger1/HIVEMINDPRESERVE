# EVIDENCE.md — S7 (Verification/Quality)

**Time:** 2026-11-06T11:24:00-06:00 · **Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **HEAD:** `5c69b66` 'feat: wire DeepSeek provider into core; live smoke, replay arm, 16-state parity, settings' (parent `f4c780d`).
**Read-only discipline:** no product/git/config writes; no key access; no paid calls; writes only `plans/seats/SEAT-07/**`.

## E1. Repository state (VERIFIED by command)

| command | observed output |
|---|---|
| `git rev-parse --short HEAD` | `5c69b66` |
| `git log --oneline -2` | `5c69b66 feat: wire DeepSeek provider into core; live smoke, replay arm, 16-state parity, settings` / `f4c780d audit: close verifier doc-sync notes` |
| `git status --porcelain` | 8 paths changed: M `backend/Cargo.lock`, M `backend/crates/hiverouter-{contract,core,provider,tools}/Cargo.toml`, M `web/package.json`; `?? docs/RELEASE.md`, `?? plans/seats/` |

> **Quiescence failure:** the tree is dirty and HEAD has advanced beyond every landed gate log (`3988622`-era). Per `auditor` quiescence rule, no gate can be certified on a moving tree; the current tree is therefore evidence-only, not gate-certified.

## E2. Test corpus inventory (VERIFIED by full file reads)

| class | files | declared tests | notes |
|---|---:|---:|---|
| backend Rust `tests/**` | 18 files (incl. 2 support mods + common) | ~110 `fn`-declared test fns | 233-pass claim cannot be re-derived from files alone (parameterized/matrix tests expand); landed log exists (E4) |
| web `tests/**` | 8 files | 46 `it()` | matches shipped claim 46 exactly (5+5+8+7+3+4+8+6) |
| e2e specs | 5 spec files (+6 helper/probe/config) | 93 executed in landed log (79 pass / 14 skip) | 14 skips are explicit `test.fixme`/skip lines, not passes |
| release scripts | `tools/build-release.ps1`, `tools/test.ps1`, `tools/qa/check-env.ps1` | — | build/test wrappers; no CI config found in slice |

## E3. Quality matrix recomputation (VERIFIED)

`audit/QUALITY-MATRIX.csv` parses to **507 rows / 507 unique paths**, 9 columns; per-seat counts 122+103+91+55+54+49+33 = 507.
All-file means recomputed: **{"correctness": 4.24, "maintainability": 3.72, "modularity": 3.52, "error_handling": 3.63, "security": 4.06, "docs": 3.34}**.
Tool chain verified: `merge_quality.py` writes matrix from 7 seat CSVs; `quality_code_stats.py` filters to code extensions and recomputes means. Neither tool enforces manifest membership or duplicate paths (validated here by hand: 0 duplicates, 507 rows).

## E4. Landed gate logs (READ; provenance gaps recorded)

| log | size | observed | gap |
|---|---:|---|---|
| `docs/audit/evidence/final-gate-tests.log` | 41,644 B | Rust `test result: ok.` lines present; tail shows contract/core/provider/tools doc-tests 0; compiled 3m02s | no exit code, no HEAD/commit id, no JUnit XML; predates HEAD `5c69b66` |
| `e2e/test-results/run-live-FINAL-artifact.log` | 14,217 B | `Running 93 tests using 1 worker`; `14 skipped`; `79 passed (3.6m)`; tail names 12 explicit skips | binary/HEAD not recorded; skips not re-run; log untracked per paper A-F08 |

## E5. Audit tool inspection (VERIFIED)

- `finalize_audit.py` (5,657 B, sha256 `A39F6778…BC59`): **mutates** WAVE-SHEET, FINDINGS-REGISTER, paper, and rebuilds `audit/VERIFICATION.md` from the verifier part + Lead addendum. Self-attestation path: verifier output is re-authored by the finalizer.
- `merge_quality.py` (2,516 B, `6BC19E13…D0BE`): matrix construction; no manifest-membership gate.
- `quality_code_stats.py` (2,381 B, `62C60F5E…EDE8`): code-only means; score 0 cells excluded from averages (silent denominator shrink).

## E6. Official DeepSeek docs consulted (fetched 2026-11-06)

| URL | facts used |
|---|---|
| https://api-docs.deepseek.com/ | base_url, model ids, `stream`, `DEEPSEEK_API_KEY`, POST /chat/completions |
| https://api-docs.deepseek.com/quick_start/pricing.md | per-1M pricing, 1M context, 384K max output, concurrency limits, peak/off-peak windows |
| https://api-docs.deepseek.com/guides/thinking_mode.md | thinking toggle default enabled/high; `reasoning_content` pass-back rule (HTTP 400 if omitted with tools); ignored temperature/penalties |
| https://api-docs.deepseek.com/api/create-chat-completion.md | request/response schemas; max_tokens 1..384K; usage fields; finish_reason enum; SSE `[DONE]` + last-chunk usage |

## E7. Commands to reproduce every number in REPORT (read-only)

```powershell
git -C C:\Users\Riley\Desktop\hiverouter-v1 rev-parse HEAD            # 5c69b66
git -C C:\Users\Riley\Desktop\hiverouter-v1 status --porcelain          # 8 paths (dirty)
node e2e\tools 2>$null; cd e2e; npx playwright test --project=chrome --reporter=list   # 93 tests (app must be up)
cd backend; cargo test -p hiverouter-contract -p hiverouter-core -p hiverouter-provider -p hiverouter-tools  # gate, see REPORT E5
cd web; npm.cmd test -- --run; npm.cmd run typecheck; npm.cmd run build
node tools\qa\verify-harness.mjs                                          # static harness 27/27 claim
```

## E8. UNVERIFIED (explicit)

- No test/gate run was executed by this seat (read-only mandate + dirty tree): all pass counts are **READ from landed logs**, not reproduced.
- No paid-provider call (no key access) → live model path, budget enforcement under real 402/429, and streamed terminal usage remain UNVERIFIED.
- HEAD `5c69b66` (provider wiring, replay arm, 16-state parity) has **no landed gate log**: its changes are not covered by any captured evidence.
- `docs/evidence/live-provider-smoke.md` (referenced by seat brief) exists in the tree? **not read by this seat** — outside assigned slice; live-smoke claims UNVERIFIED.
