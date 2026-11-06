# PLANNER-04 — Verification/Quality: the evidence & acceptance system for HiveMind 1.0

**Planner:** P4 (Verification/Quality/Roadmap/Docs) · **Time:** 2026-11-06T11:32:00-06:00
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **HEAD:** `5c69b66` = `v1.0.0-3-g5c69b66` · **Tag `v1.0.0`:** `3988622` (3 commits behind) · **Tree:** dirty at audit time (8–12 paths; all seats agree)
**Inputs read in full:** all 8 seat deliverables (48 files, 333,417 B; per-file bytes/sha256/lines in `READLOG.md`) → audit paper (244 lines) → 4 official DeepSeek docs (quick start, pricing, thinking mode, chat completions; fetched 2026-11-06). Cycles: `GATE-CY-1/2/3.md` (recon → plan → lead each; none failed).
**Skills used:** forgewright-recon/plan/lead, integration-logic, research, auditor, logic-debugger, trace, weaver, agent-experience, capability-auditor, truthseeker.

## 0. Planning answer in one paragraph

The program has a strong **source tree** and a weak **evidence system**. All P0 product defects from the audit baseline (`A-F01` provider-unwired, `A-F02` dead replay, `A-F03` 16-vs-15 parity) are fixed in **source at `5c69b66`** (S1/S2/S3/S8), but the **shipped artifacts predate that commit** (binary `184D51A7…` 09:10:59, zip `BC7A2BAD…` 09:33:17 vs commit 10:59 — S8 V4, S5 V3), the **tree is dirty**, and **no gate is HEAD-bound** (S7-F01/F04). Therefore HiveMind 1.0 as *shipped* is **BLOCKED**, and the correct planner output is not a prose narrative but a falsifiable **evidence pipeline** plus exact acceptance wording. This planner specifies that pipeline (G0–G6), the provider error-path matrix, live-smoke gating, replay/state browser proof, stale-paper/doc supersession, the final 1.0 acceptance text, and the remaining 1.1 roadmap.

## 1. What the 8 seats proved (consolidated, no summary-drift)

| Layer | Verified state | Source seats |
|---|---|---|
| Source: provider wiring | `hiverouter-core` depends on `hiverouter-provider`; `ProviderExecutorAdapter` built from env; honest `UnconfiguredExecutor`→`E_KEY_MISSING` fallback | S1 V4/V12, S2, S6, S8 V1 |
| Source: replay + state parity | 16 states incl. `interrupted`; `HISTORY_SNAPSHOT` → `buildReplayTasks` → nodes/edges wired; replay banner | S3 V1/V2, S8 V3 |
| Live provider evidence | `docs/evidence/key-validation.md` (HTTP 200 `HIVE-KEY-OK`, usage 14/6/20) and `live-provider-smoke.md` (completed run, `HIVE-LIVE-OK`, usage 89/6/95) — **self-attested prose, no commit/binary hash** | S1 §2, S6 F-02, S8 V2 |
| Shipped artifact | binary `184D51A7…C9A9` 09:10:59; zip `BC7A2BAD…FEE7` 09:33:17 — **predate provider commit** (10:59); dist untracked | S5 V1–V9, S8 V4 |
| Gates | Rust/e2e logs exist with counts but **no HEAD, command, exit code, or JUnit**; e2e 93 = 79 pass / **14 explicit skips** (12 G-UI DOM gates + 2 WS stubs) | S4, S7-F01/F04/F06, S8 S-8 |
| Trace | 43 WIRED-AND-WORKS / 270 PENDING / 6 MISSING of 319 hops; 29 controls (23 partial, 6 pending); S-APPROVAL uncaptured | S4 §3.5, S7, S8 V8 |
| Release engineering | tag behind HEAD; `tools/release-manifest.json` missing though promised; `build-release.ps1` does not build web or assemble dist; duplicate majors | S5 S5-01..05, S8 S-1/S-11 |
| Security | env-only key, redacted Debug, no key in logs/UI/DB; **P1 subprocess env inheritance** (no `env_clear`); live-doc claims conflict | S6 F-01/F-02, S5 V15 |
| Contract parity | backend 14 frozen / 21 events / 65 refusals / 16 states; UI 14/21 OK; **UI refusal map 45/65 (medium)**; `ERROR.code` free string | S2 §3, S3 V8 |
| Quality matrix | 507 rows / 507 unique paths; means 4.24/3.72/3.52/3.63/4.06/3.34 | S7 §1 |

## 2. Evidence-system defects → gates (incorporates S7, S8, S4 explicitly)

| ID | Defect (evidence) | Required gate | Acceptance |
|---|---|---|---|
| E-01 (S7-F01, S7-F04) | No gate log is HEAD-bound: `final-gate-tests.log` (41,644 B), `run-live-FINAL-artifact.log` (14,217 B) lack commit id/command/exit code/JUnit | **G0 Quiescence & identity** | A run record `evidence/gate-<HEAD>.json` contains: `head_sha`, `git status --porcelain` (empty), UTC start/end, toolchain versions, binary sha, every command, per-gate exit code |
| E-02 (S7 §5, S8 S-8) | Counts are prose; XML/exit not captured | **G1 Rust**, **G2 web**, **G3 e2e** | Rust: `cargo nextest run --profile ci` **or** `cargo test` wrapper emits JUnit XML; counts read from XML; acceptance = XML exists + 0 failed + passed ≥ baseline (233) + ignored enumerated. Web: exit 0 + coverage report. e2e: `--reporter=list,junit` → junit.xml + list; acceptance = 93 executed, 0 failed, **0 skipped OR each skip has an UNVERIFIED ledger row and a non-gate declaration** |
| E-03 (S4-1) | Skip-as-green: all browser tests skip unless `E2E_APP_READY=1` (`e2e/helpers/matrix.ts:43-52`) | G3 precondition | Bare run fails loudly (`E_APP_READY` missing → non-zero exit), never "success 0 tests"; assert `RUN executed>0` |
| E-04 (S4-4) | baseURL drift `:8080` default vs `:8799` live (`playwright.config.ts:17`) | G3 preflight | Spec fails unless `E2E_BASE_URL` set explicitly; preflight compares server `/diagnose` version+asset hash to expected |
| E-05 (S4-6) | Seed/apply ordering can silently drop trace verdicts (`build-trace-seeds.mjs:81-86`, `apply-live-trace.mjs`) | G3 integrity | `verify-harness.mjs` asserts monotonic verdict count, 29/29 controls, 0 PENDING after apply, and fails if seeds run after apply |
| E-06 (S7-F02, S6-F05) | Self-attestation: `finalize_audit.py` rebuilds `audit/VERIFICATION.md` while patching paper/register; tools edit other seats' evidence | Documentation gate | Split read-only `check_audit.py` from authoring; verifier file is never regenerated after verification; every generated doc names its generator + input hashes |
| E-07 (S8 S-1/S-8) | Doc claims exceed artifact reality: RELEASE.md "verified in the build"; 3 different zip hashes; 15 vs 16 states; trace 260/5 vs 270/6; "no paid run" vs live docs | Docs truth gate | Every numeric/hash claim resolves to `evidence/release-<HEAD>.json`; a link-checker/test fails on mismatch; supersession note required when source fixes postdate a release artifact |
| E-08 (S5 S5-02/S5-03) | Release cannot seal: manifest missing; script doesn't build web/dist; tag behind HEAD | **G6 Release seal** | `tools/release-manifest.json` exists with commit, clean status, toolchain, per-artifact sha256/size, zip hash, checksums; `build-release.ps1` builds web + rust + assembles dist + writes manifest; tag created **at the built commit** |
| E-09 (S7-F05) | Provider error paths proven only at unit/wiremock level | **G4 run-path error matrix** (§3) + **G5 live smoke** | Stub-provider run-path e2e asserts code→run row→ledger→event→UI; live smoke (operator-gated) asserts same against real provider |

## 3. Provider error-path test matrix (required for 1.0 acceptance)

| Path | Expected named code(s) | Required test level | Acceptance assertion |
|---|---|---|---|
| Missing key | `E_KEY_MISSING` (env var named, never value) | unit **+ run-path e2e (stub)** | `SUBMIT_PROMPT` → ERROR frame with code+`how_to_fix`; 0 provider calls; no run row success; UI toast; key string absent from logs/DB |
| Bad/absent WS token (401) | `E_TOKEN_INVALID`; provider 401 → `E_PROVIDER_AUTH` | unit + run-path | 401 at transport gate closes socket; provider 401 → run `failed` with `error_code`, ledger row, UI refusal identity preserved (not generic) |
| Billing (402) | `E_PROVIDER_AUTH`-class (mapping documented) | wiremock + run-path | 402 body reason preserved into run ledger + event + UI `how_to_fix`; no retry storm |
| Rate limit (429) | `E_PROVIDER_RATE_LIMIT` / `E_LLM_RATE_LIMIT` | wiremock + run-path | retry/backoff bounded; terminal run reflects rate limit; ledger row |
| Timeout | `E_PROVIDER_TIMEOUT` | wiremock + run-path | bounded timeout; run `failed`; no fake success; UI clears streaming |
| Budget exceeded | `E_BUDGET_EXCEEDED` (pre-send, preflight) | unit + run-path | provider call count 0 after refusal; `runs.cost_usd/prompt_tokens/completion_tokens` **NULL/INCOMPLETE, never 0** |
| Malformed plan/schema | `E_SCHEMA_INVALID` / `E_PROVIDER_INVALID_RESPONSE` | contract + wiremock + run-path | DAG not persisted on invalid plan; refusal names code; no hang |
| Usage absent | `E_USAGE_INCOMPLETE`; cost unresolved | wiremock + terminal truth | `usage` row marks incomplete; aggregate renders "—"/NULL, never 0; failed/budget terminal truth matches live events vs persisted ledger |
| Replay / approval nonce replay | `E_APPROVAL_REPLAY`, `E_RESYNC_REQUIRED` | unit + browser e2e | replay render read-only; duplicate nonce refused; stale `expected_rev` resync |

Commands: `cargo test -p hiverouter-provider --test refusal_matrix`; stub run-path: `cargo test -p hiverouter-core --test run_lifecycle -- --nocapture`; browser: `cd e2e; npx playwright test specs/live-app.spec.ts -g "provider"`. **New tests required:** `run_lifecycle::provider_refusal_matrix_maps_to_run_and_ledger`, `terminal_truth::failed_run_usage_is_null`, `replay::scrubber_renders_snapshot_read_only`, `parity::ui_enum_equals_contract_enum`.

## 4. Live-smoke gating (operator-gated, never default CI)

1. Script `tools/qa/live-smoke.ps1` requires **all** of: `--live --yes`, `DEEPSEEK_API_KEY` present, `--max-cost-usd` cap (>0), explicit base URL, and a clean checkout at a recorded HEAD.
2. It records: HEAD, binary sha, request `correlation_id`, provider response id, terminal `RUN_STATUS`, usage counters, wall time, and a redacted raw transcript to `evidence/live-<HEAD>-<ts>.json` (tracked-by-manifest or hash-registered).
3. Acceptance for 1.0: transcript exists, `completed` terminal, usage non-null, no key material in file (grep gate), and `cost_usd` within the declared pricing (official pricing page URL+fetch date recorded).
4. Key hygiene: rotate the transcript-adjacent key after release (S8 S-12); never commit key; subprocess env isolation fix (S6 F-01 `env_clear`) must land before repeating "key never leaves the process" claims.
5. Never run by default: absent `--live` the script exits non-zero with `E_LIVE_GATE` (not skip-as-green).

## 5. Replay / state browser proof (S3 + S4)

- **Replay render:** e2e — open History tab → select a revision → `HISTORY_SNAPSHOT` received → canvas nodes/edges count matches snapshot graph; replay nodes `draggable:false`; banner names revision; no write commands emitted in replay.
- **State parity pin:** contract enum (16) == web enum (16) == docs claim; `agentState` terminal set includes `failed`, `interrupted`, `completed`, `cancelled`, `killed` (S3 D1 fix) — a failing test must catch the 16-vs-15 regression.
- **Terminal truth:** failed/budget run → header shows terminal state, usage NULL rendered as "—", and persisted `/events` matches live event sequence (trace skill §terminal-state).
- Acceptance: `npx playwright test specs/live-app.spec.ts -g "replay|state parity|terminal"` → all pass; JUnit counts recorded; no skip.

## 6. Stale audit-paper claims — supersession policy

- Paper (`audit/HIVEMIND-AUDIT-PAPER.md`) is an artifact of HEAD `3988622`; it must **not** be silently patched to look current. Add a dated **supersession block** after the abstract: A-F01/A-F02/A-F03 fixed in source at `5c69b66`; shipped artifacts still predate; A-F06/A-F04 and P2 findings remain open; trace/quality numbers as of baseline.
- `audit/VERIFICATION.md` must state its generator (`finalize_audit.py`) and the exact input commits/hashes, or be replaced by a manually signed verifier statement.
- `docs/reports/FINAL-REPORT.md` corrections (S8 §3): 15→16 states (L26), trace 260/5→270/6 (L45), "no paid provider run" (L51) updated to distinguish **pre-release live evidence exists / release-artifact proof absent**, zip hash corrected to actual `BC7A2BAD…` (L12). `README.md` zip hash/size corrected. `docs/RELEASE.md` remains **draft** until G6 passes.
- Acceptance: a `docs-truth` test/script parses every hash/count claim and fails on mismatch; release docs may cite only `evidence/release-<HEAD>.json`.

## 7. Final HiveMind 1.0 acceptance wording (canonical)

Two-layer wording, because artifact ≠ source:

**A. Recommended now — `BLOCKED` for release certification:**
> "HiveMind 1.0 is **BLOCKED** for release certification. The source at `5c69b66` contains the P0 fixes (provider wiring, replay arm, 16-state parity), but (1) the shipped artifacts (`tools/bin/hiverouter.exe` sha256 `184D51A7…C9A9`, `dist/HiveMind-v1-win64.zip` sha256 `BC7A2BAD…FEE7`) predate the fix commit; (2) the working tree is dirty and no gate is HEAD-bound (no JUnit/exit-code/HEAD capture); (3) live provider evidence is self-attested and pre-release; (4) 12 of the 14 e2e skips are the declared G-UI DOM gates, so accessibility is unproven. Rebuild at HEAD on a clean tree, run gates G0–G6, seal the release manifest, then re-adjudicate."

**B. Allowed alternative once G0–G4/G6 pass — `PASS-WITH-NOTES` (source build only):**
> "HiveMind 1.0 **PASS-WITH-NOTES** for the **source integration build** at `<HEAD>`. Evidence: clean-tree HEAD-bound gate logs with JUnit XML and exit 0; Rust/Web/e2e counts from XML; stub-provider run-path matrix (missing key/401/402/429/timeout/budget/malformed/usage-absent) mapped to run ledger/event/UI; replay and 16-state parity browser tests pass; release manifest sealed at the built commit; remaining UNVERIFIED: live paid-provider path (operator-gated evidence only), 12 G-UI DOM gates (declared non-gates until implemented), pixel-level design fidelity."

**Not acceptable:** plain `PASS`; `PASS-WITH-NOTES` without a clean-tree HEAD-bound evidence bundle; any wording that says "verified in the build" while artifacts predate source.

## 8. Remaining 1.1 roadmap (ordered by risk)

| Wave | Task | Owner | Acceptance |
|---|---|---|---|
| W1 | Evidence system: G0–G3 (HEAD-bound logs, JUnit XML, exit codes, quiescence, artifact identity) | S7/Lead | one `evidence/gate-<HEAD>.json` per gate; docs-truth checker |
| W2 | Provider run-path proof: G4 stub matrix + `provider_refusal_matrix_maps_to_run_and_ledger`, terminal NULL-not-zero test | S1/S7 | 8/8 error paths; failed-run usage NULL; no paid call |
| W3 | Replay/state browser proof: replay render read-only, 16-state parity pin, `failed` terminal fix (S3 D1) | S3/S4 | e2e green; parity test fails on 15-state regression |
| W4 | Release engineering: G6 manifest, web build in release script, tag at built commit, checksums, `.gitattributes` | S5 | manifest+tag+clean rebuild; zip/binary hashes match docs |
| W5 | Docs truth: paper supersession block, FINAL-REPORT/README/PHASE-12/claim-register corrections; RELEASE.md final after G6 | S8/Lead | docs-truth checker passes; zero unresolved hash/count contradictions |
| W6 | Key hardening: `env_clear` on subprocess spawn (S6 F-01), key-file ignore patterns, rotation after transcript | S6 | test proves child env lacks key; docs claim matches |
| W7 | Live smoke: operator-gated script with budget cap + transcript + rotation note (G5) | S1/S4 | passing transcript for release commit; never in CI default |
| W8 | P2 debt: skip-as-green/baseURL/seed-order/validator thresholds/writes-into-docs; quality-instrument gates; trace completeness (43→≥80% or declared) | S4/S7 | harness self-tests; trace matrix 29/29; 0 skip-as-green |

## 9. Incorporation matrix — S7 / S8 / S4 findings used

| Source finding | Where incorporated |
|---|---|
| S7-F01/F04 (no HEAD-bound/JUnit logs) | E-01, E-02, G0–G3, acceptance wording A(2)/B |
| S7-F02 (finalizer self-attestation) | E-06, §6 |
| S7-F03 (paper baseline stale) | §6 supersession policy |
| S7-F05 (provider errors unit-only) | §3 matrix, W2 |
| S7-F06 (12 DOM-gate skips) | E-02 acceptance, wording A(4), W8 |
| S8 S-1 (RELEASE.md claims exceed build), S-3/S-4 (zip hash contradiction), S-5 (15 vs 16), S-6 (260/5 vs 270/6), S-7 (no paid run vs live docs), S-8 (logs predate commit), S-9 (paper A-F01 fixed in source), S-12 (key rotation) | E-07, §6, §7, W5 |
| S8 V4 (shipped artifacts predate commit) | E-08, §7 wording A(1), W4 |
| S4-1 skip-as-green, S4-4 baseURL drift, S4-6 seed/apply order, S4-3 no automated paid-safe journey, S4 §journey gaps (replay/plan/terminal) | E-03, E-04, E-05, §4, §5, W2/W3/W8 |

## 10. UNVERIFIED (planner limits)

- No gate was executed by P4 (read-only; tree dirty): all counts are READ from seat/landed evidence; the G0–G6 commands are **specified, not run**.
- Live provider behaviour (402/429/budget under real API), terminal usage parity, and streamed SSE deltas remain UNVERIFIED until G4/G5 run.
- `docs/evidence/live-provider-smoke.md`/`key-validation.md` claims are self-attested prose; no raw transcript was available to this planner.
- Shipped binary/zip content vs HEAD source not diffed (would require rebuild — not allowed here); S8's timestamp/hash comparison is the evidence basis.
- Whether the 233/46/79 counts reproduce at HEAD is unknown until G1–G3 run.

## 11. Deliverables

`plans/planners/PLANNER-04/{PLANNER-04.md, READLOG.md, GATE-CY-1.md, GATE-CY-2.md, GATE-CY-3.md}`. Read-only: no product/git/config writes, no key access, no paid calls; only this directory written.
