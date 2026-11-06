# SEAT H — INDEPENDENT 1.1 RELEASE / 1.2 READINESS AUDIT

- **Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **HEAD `9651722`** (2 docs/release commits after tag target) · `git status` **clean** · branch `master`.
- **Scope:** read-only audit; only this file written. No product edits, no commits/tags, no key access, no paid calls.
- **Skills used (all five):** `forgewright-recon` (§Recon ledger), `forgewright-lead` (read callers/tests before claims; VERIFIED/READ/INFERRED/UNVERIFIED separation; no mutation), `logic-gatekeeper` (Phase-1 JSON — used as refusal gate for the release-evidence claim), `integration-logic` (call chain: provider SSE → core adapter → WS → UI; release script → zip → manifest → docs), `logic-debugger` (evidence-binding and skip-as-pass invariants; fault: claimed passed live run vs skipped results).

## Recon (forgewright-recon ledger)

- **Target behaviour:** a sealed, truthfully documented 1.1.0 release whose tag, artifacts, manifest, docs, tests, trace counts and live streaming/replay evidence all agree.
- **Entry points:** tag `hivemind-1.1.0` (`87079ce`); `tools/release-manifest.json`; `dist/HiveMind-1.1.0-win64.zip`; `docs/evidence/1.1-*`; `tools/qa/check-docs-truth.mjs`; `e2e/specs/live-streaming.spec.ts` + `live-replay.spec.ts` + `e2e/helpers/live-server.ts`; `docs/evidence/1.1-backend-final.log`.
- **Callers/consumers:** `docs/RELEASE-EVIDENCE.md` (claims), `README.md`/`docs/RELEASE.md` (version/streaming claims), `docs/claims/CLAIM-REGISTER.*`, `docs/trace/*` (trace truth), `e2e/run-guarded.mjs`, `tools/build-release.ps1` (sealed build).
- **Tests + commands:** `cargo test --workspace` (log); `npm run typecheck|test|build` (release build log); `node tools/qa/check-docs-truth.mjs`; guarded Playwright (`npm run e2e`) / live specs (`E2E_SERVER_BIN`, `LIVE_ONE_PROMPT=1`).
- **Data flow:** `tag 87079ce` → `tools/build-release.ps1` (HEAD check, tag behind=0, web build, cargo build, zip+CHECKSUMS, manifest, absolute-path scan) → artifacts/manifest → docs claims. Failure path: live specs must **skip by name**, never fake-pass (`serverSkipReason`/`paidGate`).
- **Unknowns:** live paid-proof durability (no committed log found), trace-capture emptiness, 1.2 requirements.
- **Files inspected:** manifest, zip/CHECKSUMS/sidecar, docs/RELEASE-EVIDENCE.md, README, docs/RELEASE.md, claims CSV/JSON, controls.json + trace docs, 1.1 logs, e2e specs/helpers, build-release script/logs, tags/artifacts.

## Gate results

| # | Item | Result | Exact evidence |
|---|---|---|---|
| 1 | HEAD / tag identity | **PASS** | `HEAD=9651722a8e4b709fef09e58ae0b7804277cfdcc9`; `git rev-parse hivemind-1.1.0^{commit}` = **`87079cecf6a4cf2212b9d41e56f4c7643a97c7fe`** (annotated tag → source commit); `git status --porcelain` = **0 lines**; old tags unchanged: `hivemind-1.0.0=23ba74a`, `v1.0.0=3988622`. `git describe` = `hivemind-1.1.0-2-g9651722` (HEAD = tag + 2 release/doc commits). |
| 2 | Release artifacts | **PASS (exact)** | `target/release/hiverouter.exe` 13,624,320 B **`9c3174fb7c118143139ff283c08a3190c8e146a831717fb3ff8e25d6468d7207`**; CLI 5,372,416 B **`f757f55833f52f626cff6f962d31cb18d740dab53e314c5ba6048599f09214df`**; `dist/HiveMind-1.1.0-win64.zip` 8,613,122 B **`af2345f663643f1c0d1dbc18a298511817bf38090351d5f5b4842418f4d63e56`** — all three match the expected values and the manifest. Zip opens with **6 members** (`CHECKSUMS.txt`, 2 exes, README, RELEASE, RUNBOOK); **all member hashes recomputed = CHECKSUMS.txt**; sidecar `af2345f6…` = zip hash; `dist/HiveMind-1.1.0-win64/CHECKSUMS.txt` sha256 **`0770cfd8…6ea84b`** = manifest `package.checksums_sha256`. |
| 3 | Manifest source/tag/version | **PASS** | `tools/release-manifest.json`: `schema=hivemind.release-manifest.v1`, `version="1.1.0"`, `tag="hivemind-1.1.0"`, `tag_target=source_commit=87079ce…`, `tag_behind_commits=0`, `source_dirty=false`, `source_describe=hivemind-1.1.0`, toolchain (rustc 1.99.1 / node v26.7.0 / gcc 16.2.0), provider block (endpoint/models), `supersedes`/`old_to_new` recording 1.0.0 → 1.1.0, `note` "Paths are repo-relative… No tag was moved". **Real Windows drive paths in manifest = 0** (only `s:/` matched inside `https://` URL — false positive). Web files' per-file hashes match disk; **web tree hash reproduced exactly**: sorted repo-relative `path<TAB>sha256<TAB>bytes` lines, LF-joined, trailing newline → **`7911b91c71870ec7b4f961a6b0e2d9b064f318cadbc64f8ac5a2a3494a103f10`** = manifest `web.tree_sha256`. |
| 4 | Built version claim | **PASS** | `target/release/hiverouter.exe --version` → `hiverouter 1.1.0 (contract schema hiverouter.v1)`; `hiverouter-cli.exe version --format=json` → `"version":"1.1.0"`, `provider_version":"1.1.0"`. |
| 5 | Docs truth checker | **PASS** | `node tools/qa/check-docs-truth.mjs` → 11/11 PASS, **`checks: 11/11 pass; violations: 0`**, `DOCS TRUTH: PASS`, exit **0**. Covers version agreement, README/RELEASE streaming claims, RELEASE-EVIDENCE version + manifest pointer + masked 1.0 history, claims JSON/CSV current-version truth. |
| 6 | Test claims 286 / 93 | **PASS (log-recomputed; binding note)** | `docs/evidence/1.1-backend-final.log` → 32 `test result` suites, **286 passed / 0 failed / 2 ignored**, `EXIT=0` (sums recomputed). `docs/evidence/1.1-release-build-4.log` → `Tests 93 passed (93)`, `Test Files 15 passed (15)`, typecheck + web build green, `HEAD: 87079ce… (hivemind-1.1.0)`, `EXIT=0`, `=== DONE - release sealed ===`. Note: the backend-final log carries **no HEAD/COMMIT line** (trace bound to build log instead); no re-run performed in this read-only audit. |
| 7 | Key scan | **PASS** | Strict credential regex `sk-[A-Za-z0-9]{32,}` over tracked non-vendor files = **0 hits**; no key material in manifest/evidence/logs/zip; live specs assert `redactSecrets(...)` never matches `sk-…` and fail closed with `E_KEY_MISSING` when keyless. |
| 8 | Trace counts / stale trace & claims | **PASS (current docs) / residual** | `docs/trace/controls.json` = **1 WIRED-AND-WORKS / 264 PENDING / 5 MISSING / 49 UNVERIFIED of 319 hops**; trace docs (`live/README.md:4`, `QA-HARNESS.md:4`, `TRACE-SCHEMA.md:4`, `CAPTURE-REGISTRY.md:27`), claims JSON notes, and FINAL-REPORT supersession block all state these same current numbers. Grep for `270 PENDING|43 WIRED|260 UNVERIFIED|6 MISSING` in README/RELEASE/RELEASE-EVIDENCE/claims JSON = **0** (only historical sections/superseded references remain, correctly labelled). **Residual:** `docs/trace/live/trace-capture.json` has `surfaces: []` (0) — registered as a W4 capture defect in `CAPTURE-REGISTRY.md`, so trace truth rests on `controls.json` only. |
| 9 | **Live streaming / replay evidence** | **BLOCKED** | `docs/RELEASE-EVIDENCE.md:19` claims: "**Live paid-provider streaming smoke on the sealed release binary** … `live-streaming.spec.ts` **3 passed** … and `live-replay.spec.ts` **2 passed**". No committed log/JSON backs this claim (no live-streaming/live-replay result file in `docs/evidence/`, `docs/trace/`, `e2e/test-results/` is gitignored/untracked). The only observed on-disk run artifact was transient `e2e/test-results/results.json` (seen 2026-11-06T16:46:16, 144 s) with `stats: {"expected":0,"skipped":5,"unexpected":0}` — i.e. the recorded live run **skipped all 5 tests** (gates `LIVE_ONE_PROMPT`/`E2E_SERVER_BIN` not set), and that file has since disappeared (uncommitted). So the 3+2 passed claim is **not verifiable and is contradicted by the only observed run output**; by the gatekeeper rule (claims without reproducible evidence cannot pass) this release-evidence item is **BLOCKED**. |
| 10 | 1.2 readiness surface | **NOTE** | No 1.2 tag/artifact/manifest; `plans/waves/` contains 1.2 planning (`W10-TOOL-LOOP-DECISION.md`, `W11-CONTRACT.md`, `W11-GUI-RESIDUAL.md`, `W11-TOOL-ASSEMBLER.md`, `W11-TRACE.md`). 1.2 gate/acceptance wording not yet defined in a released record. No 1.2 claim is made in current-facing docs. |

### Logic-gatekeeper Phase-1 JSON (refusal gate — no mutation performed)

```json
{
  "phase1": {
    "architecture": "Sealed git tag hivemind-1.1.0 -> commit 87079ce; artifacts + release-manifest.json; docs claims (RELEASE-EVIDENCE/README/claims/trace); e2e harness with named-skip gates; HEAD 9651722 clean.",
    "dataFlow": "build-release.ps1 from tag -> web/cargo build -> staging + zip + CHECKSUMS -> manifest (hashes/tree/version/tag) -> docs claims; live specs gated by E2E_SERVER_BIN + LIVE_ONE_PROMPT -> results.json (uncommitted) -> RELEASE-EVIDENCE prose.",
    "edgeCases": ["live run without LIVE_ONE_PROMPT -> all 5 specs skipped, 0 passed (observed)", "tag vs HEAD drift (tag behind=0, 2 docs commits ahead)", "0-byte/empty trace capture (surfaces=[] observed)", "manifest fields with URL containing 's:/' vs real drive paths"],
    "rootCause": "The live paid streaming/replay proof is asserted in release evidence but not bound to any committed, reproducible run artifact; the only observed live-run output skipped all five tests, so the evidence claim cannot be distinguished from a skip-as-pass and blocks the release-evidence gate."
  }
}
```

No mutating edit was performed (write scope = this file only), so Phase 2 does not apply.

## Integration-logic (call chain and seams)

`SUBMIT_PROMPT` → run engine → `ProviderExecutorAdapter::stream_reply` (SSE fragments) → `REPLY_DELTA` frames → usage from final chunk → `REPLY_DONE` → `RUN_STATUS completed`; `HISTORY_REVISION_SET` → `HISTORY_SNAPSHOT` → canvas read-only projection. Seam checks that hold: offline matrix `provider_streaming.rs` pins ordering/cancellation/retry without network; keyless path refuses `E_KEY_MISSING` with zero deltas (spec assertion); live specs call `test.skip(<named reason>)`, never fabricate. Seam that fails the gate: the live proof's **result artifact is not conserved** — claim lives only in prose (§9). Release chain (script→zip→manifest→docs) verified end-to-end except the staged-doc host-path warning (§Residual R2).

## Logic-debugger (invariants attacked)

- **Invariant 1 — every release claim is bound to a reproducible artifact:** violated for live streaming/replay evidence (BLOCKER). Evidence: no committed log; transient `results.json` showed `expected:0, skipped:5`.
- **Invariant 2 — skipped ≠ passed:** code correctly implements named skips (verified), but the *claim text* treats a 3+2 passed run as fact without the captured output → same defect class at the evidence layer.
- **Invariant 3 — artifact/tag/manifest/version agree:** holds (hashes, tag target, manifest version/source/tag, tree hash, zip CHECKSUMS, binary `--version` all verified).
- **Invariant 4 — docs agree with product truth:** holds at HEAD (docs checker 11/11; no stale trace tokens in current-facing docs).
- **Invariant 5 — old release immutable:** holds (`hivemind-1.0.0`, `v1.0.0` unchanged).

## Ranked residual list

| Rank | Residual | Severity | Evidence | Fix |
|---|---|---|---|---|
| R1 | **Live paid-provider streaming/replay proof is unverifiable**: claimed 3+2 passed tests have no committed log; only observed `e2e/test-results/results.json` shows `expected:0, skipped:5`; file transient/uncommitted. **Corroborated during this audit window:** a new untracked `docs/evidence/live-trace-1.1.md` (seat D, capture attempt `2026-11-06T22:46:32.102Z`, checkpoint HEAD `9651722`) records **`E_LIVE_TRACE_BLOCKED_NO_OPERATOR_KEY`** — `LIVE_ONE_PROMPT` unset, `DEEPSEEK_API_KEY` absent, "No paid call was made… no trace hop was fabricated"; raw capture required by registry rule. This independently confirms no paid streaming/replay capture was possible in the audited environment. | **BLOCKER** | RELEASE-EVIDENCE:19; captured results.json stats; `docs/evidence/live-trace-1.1.md` (untracked); file now absent | Re-run `live-streaming.spec.ts` + `live-replay.spec.ts` with `E2E_SERVER_BIN=target/release/hiverouter.exe LIVE_ONE_PROMPT=1` on sealed 1.1.0 and commit raw log/JSON + timestamp/hashes to `docs/evidence/`; or demote the claim to READ/UNVERIFIED in RELEASE-EVIDENCE |
| R2 | Trace truth degraded: `docs/trace/live/trace-capture.json` is empty (`surfaces: []`) after W4 capture incident; controls rest on registry, not raw capture | Medium | `CAPTURE-REGISTRY.md`; parse `surfaces=0` | Re-capture on sealed 1.1.0 binary; keep `controls.json` counts (1/264/5/49) labelled registry-only meanwhile |
| R3 | Release absolute-path scan WARN: 1 staged doc contains a host path (manifest/CHECKSUMS clean = 0 hit) | Low | `1.1-release-build-4.log:100`: "WARN: 1 staged doc(s) contain a host path; docs are read-only for this seat" | Sanitize the staged doc before next seal |
| R4 | Backend 286-test log lacks HEAD/COMMIT binding (build log binds HEAD 87079ce; backend log does not) | Low (evidence binding) | `1.1-backend-final.log` first lines (no HEAD=) | Add HEAD/version header to test-log generator |
| R5 | 1.2 readiness not defined: no 1.2 tag/artifact/manifest or acceptance wording; only planning docs (W10 tool-loop decision, W11 contract/UI/tool/trace) | Note | `plans/waves/*`; tag list | Publish 1.2 scope/gates before starting 1.2 work |

## Verdict

**BLOCKED** — the 1.1.0 **artifacts, tag, manifest, version, docs and 286/93 test claims verify**, but the release's **live paid-provider streaming/replay evidence** (the headline 1.1 claim) is not backed by a committed, reproducible artifact and the only observed live-run output skipped all 5 tests. Everything else is PASS or a documented residual; no product/git/tag edits, no key access, no paid calls were made by this audit.

## Evidence appendix (quoted commands)

```
git rev-parse HEAD                          -> 9651722a8e4b709fef09e58ae0b7804277cfdcc9
git rev-parse 'hivemind-1.1.0^{commit}'     -> 87079cecf6a4cf2212b9d41e56f4c7643a97c7fe
git status --porcelain                      -> (0 lines)
git tag -l                                  -> hivemind-1.0.0, hivemind-1.1.0, v1.0.0
Get-FileHash target/release/hiverouter.exe  -> 9C3174FB…8D7207 (13,624,320 B)
Get-FileHash target/release/hiverouter-cli.exe -> F757F558…9214DF (5,372,416 B)
Get-FileHash dist/HiveMind-1.1.0-win64.zip   -> AF2345F6…4D63E56 (8,613,122 B)
zip members=6; CHECKSUMS member hashes all match; checksums_sha256=0770cfd8…6ea84b = manifest
node tools/qa/check-docs-truth.mjs          -> 11/11 pass; violations:0; DOCS TRUTH: PASS (exit 0)
1.1-backend-final.log                       -> 32 suites, 286 passed / 0 failed / 2 ignored, EXIT=0
1.1-release-build-4.log                     -> HEAD 87079ce… (hivemind-1.1.0); Tests 93 passed (93); Test Files 15 passed (15); DONE sealed; EXIT=0
manifest                                    -> version 1.1.0; tag hivemind-1.1.0; source 87079ce…; tag_behind=0; source_dirty=false; real drive paths=0
web tree hash                               -> 7911b91c…103f10 (reproduced from sorted relpath<TAB>sha256<TAB>bytes LF lines + trailing newline)
controls.json                               -> 1 WIRED-AND-WORKS / 264 PENDING / 5 MISSING / 49 UNVERIFIED (319 hops)
transient e2e/test-results/results.json     -> stats {"expected":0,"skipped":5,"unexpected":0} (16:46:16); file now absent
```

## What was NOT verified

- Live paid-provider streaming/replay claims were not re-run (no key access/paid calls permitted); §9 documents why they fail the evidence gate without a re-run.
- No fresh `cargo test` / web test build was executed (read-only audit; logs were parsed and sums recomputed).
- 1.2 acceptance was not defined anywhere to verify; only planning docs exist.
