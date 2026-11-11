# HiveMind release evidence

## Current working state — 2026-11-10 (supersedes the stale 2026-11-06 snapshot below)

- **Audit HEAD:** `1129349`; **audit docs commit:** `44e0535`; **current branch HEAD may be later** (README truth follow-up `6f91d2b`). **Installed product build:** `806251b` (`dirty:false`).
- **Version/schema:** `1.2.0-dev`, schema `hiverouter.v1` version **24**.
- **Offline battery:** `docs/evidence/live/full-battery-20261110-234515.json` — 8/8 executed, 0 failed, `working_tree_dirty:true` at `1129349` (audit docs in the tree; the ledger endpoint candidate was discarded). This is **not** a clean-HEAD or release seal.
- **Install receipt:** `docs/evidence/live/install-1.2.0-dev.json` — installed server `1.2.0-dev`, schema 24, prompt assets loaded, health `degraded` with `E_KEY_MISSING`, 1440/375 screenshots, package SHA-256 `B4BAC18D29560D5C63404649C2953E359FFC6170C1214A3AA1B54DE127811061`.
- **Crash drill:** `docs/evidence/live/crash-drill-20261110-230629.json` — process killed, restarted, boot_id changed, schema 24, data preserved.
- **Scale drill:** `docs/evidence/live/scale-drill-20261110-230653.json` — 41-task serial and bounded-parallel probes; the receipt explicitly says **not a live 40-node provider run**.
- **No live paid provider proof** for 1.2; no provider key is configured. All current live prompts refuse `E_KEY_MISSING`.
- **No 1.2 tag, release manifest, or strict release PASS.** `-Release` remains PRE-RELEASE exit 3.
- **Truth audit:** `docs/audit/HIVEMIND-TRUTH-AND-ROADMAP-2026-11-10.md`; plan status with strikethrough: `docs/plans/HIVEMIND-FINISH-PLAN-v3-STATUS.md`.

The older 2026-11-06 observations below are retained as history and are superseded by this block where they conflict.

**Working tree source line: `1.2.0-dev` (pre-release; unsealed; no 1.2 artifacts or tag).** The authoritative
exact-hash registry for released artifacts remains `tools/release-manifest.json` (sealed 1.1.0 build); this file never
invents hashes. The `1.2.0-dev` version bump itself changes version strings only (four crate manifests + `backend/Cargo.lock`
workspace package entries + `web/package.json` / `web/package-lock.json` root); the same 1.2 WIP tree also contains
substantive product code (approval suspension, side effects, web replay, live specs) — **no runtime/tool-loop claim is
made for 1.2**, no 1.2 release build was run, and `hivemind-1.1.0` (`87079ce`) remains the build source that produced the
released artifacts. HEAD `889b779` is 7 commits ahead of that tag (post-release evidence commits + uncommitted
W11/W12 work); the tree is dirty.

> **OLD → NEW (2026-11-06, W12 Wave 2):** `1.1.0` → `1.2.0-dev` in the four `backend/crates/*/Cargo.toml` manifests,
> the four matching `backend/Cargo.lock` package entries, and the `web/package.json` + `web/package-lock.json` root
> versions. No dependency version, contract schema version (`hiverouter.v1`), git tag, or release-manifest hash changed.
> **Post-version truth (Wave 3):** `tools/qa/check-docs-truth.mjs` derives the sealed release from
> `tools/release-manifest.json` (version/source/tag), asserts this file states the current source line **and** preserves
> the sealed artifact hashes below, and rejects a pre-release tree presented as the released version. `run-gates.ps1
> -Release` accepts this documented pre-release tree (manifest still `1.1.0`, tag target `87079ce` == source, source is
> an ancestor of HEAD) only with an explicit **PRE-RELEASE** result (exit 3), never a release PASS. In accepted
> pre-release mode `G7` keeps the sealed tag/source and package zip/sidecar/CHECKSUMS strict while reporting workspace
> artifact/web-dist drift from the sealed manifest as pre-release drift warnings (dev builds are expected to differ; no
> release claim). `README.md` and `docs/RELEASE.md` remain sealed-release docs; both now carry a working-tree
> pre-release notice, and neither may claim 1.2 as released or verified.

## Working tree status: `1.2.0-dev` (UNRELEASED, UNVERIFIED)

The working tree declares `1.2.0-dev` in `web/package.json`, all four `backend/crates/*/Cargo.toml` manifests, and the
four matching `backend/Cargo.lock` package entries. It is a **pre-release source line**: no 1.2 tag, release manifest,
artifact, live paid proof, or release-gate PASS exists. The sealed 1.1.0 record in the next section remains the only
release authority. **No end-to-end verification of the current 1.2 tree exists.** Evidence below is labelled by
source: the 1.2 logs are gitignored local files, `plans/foresight/RUN-3.md` is an untracked working-tree trace at
audit time, and the E2E result files are gitignored; none of it is a committed or sealed release receipt.

| Item | Recorded state | Evidence (labelled source) |
|---|---|---|
| Source commit | HEAD `889b779` (`hivemind-1.1.0-7-g889b779`); tree dirty; `hivemind-1.1.0` remains `87079ce` | `git rev-parse HEAD`, `git describe --tags`, `git status --porcelain` (read 2026-11-06); `plans/foresight/RUN-3.md:4-5` records the same HEAD and, at the RUN-3 snapshot, a 36-entry dirty tree |
| Tag | **No 1.2 tag.** Existing tags: `hivemind-1.0.0`, `hivemind-1.1.0`, `v1.0.0` | `git tag --list` (read 2026-11-06) |
| Artifacts | **No 1.2 artifact.** `dist/` contains the 1.0.0/1.1.0 packages plus older/rehearsal files (`HiveMind-v1-win64.zip`, `.release-manifest-*.json`) and the 1.1.0 staging directory; there is no 1.2 zip or 1.2 staging directory | `dist/` listing (read 2026-11-06) |
| Manifest | `tools/release-manifest.json` is still the sealed 1.1.0 record (`version=1.1.0`, source/tag target `87079ce`); no 1.2 manifest was created | `tools/release-manifest.json` |
| Backend tests (current integrated tree) | **475 passed / 0 failed / 2 ignored**, `EXIT=0` (`cargo test --workspace --locked`, PATH prepended with `tools/w64devkit/bin`) | `docs/evidence/1.2-backend-tests-14.log` — local, gitignored (`*.log`) |
| Backend tests (superseded claim) | 441 passed was an intermediate claim; the current 475-run supersedes it | `plans/foresight/RUN-3.md:197`; `docs/evidence/1.2-backend-tests-14.log` |
| Compile gate | `cargo check --workspace --tests --locked` exit 0 on the current integrated tree (22:46 CT) | command output; `docs/evidence/1.2-compile-gate-2.log` (historical) |
| Web tests | **179 passed / 0 failed**, `EXIT=0`; `tsc --noEmit` exit 0 | `docs/evidence/1.2-web-tests-3.log`, `docs/evidence/1.2-web-typecheck.log` (local, gitignored) |
| E2E (guarded, mocked + live skipped) | **106 passed / 0 failed / 166 skipped**, exit 0; skips are the live/paid and capture lanes | `node run-guarded.mjs` output in the session; `e2e/test-results/` (local, gitignored) |
| Static docs/harness checks | 19/19 docs-truth checks pass, 0 violations; 28/28 harness checks pass, 0 failures | commands run by lane F on 2026-11-06: `node tools/qa/check-docs-truth.mjs`, `node tools/qa/verify-harness.mjs` (console output; no log file written). These check docs/harness structure, not 1.2 product code |
| Live paid proof | **None for 1.2.** No `docs/trace/live/captures/w12/` directory and no 1.2 live log exist. The last recorded live-trace capture attempt (1.1) was blocked (`E_LIVE_TRACE_BLOCKED_NO_OPERATOR_KEY`) with no fabricated hop; no 1.2 live attempt is recorded | `docs/evidence/live-trace-1.1.md` (1.1 attempt); `docs/trace/capture-registry.json` (`CAPTURE_PLAN_ONLY`); `docs/trace/live/trace-capture.json` (`surfaces: []`, 286 B) |
| Trace / controls | 29 controls / 319 hops: **1 WIRED / 264 PENDING / 5 MISSING / 49 UNVERIFIED**. The one WIRED hop is `S22/6_service`, backed by a pre-HEAD raw probe — not a 1.2 proof | `docs/trace/controls.json`; `docs/trace/capture-registry.json` |
| Release gate | **No 1.2 `run-gates -Release` PASS or PRE-RELEASE log exists.** A 1.2 release build is blocked until the fixes and gates below pass | `plans/foresight/COMPILED-LEDGER.md:39-43,154`; `plans/foresight/PROOF-RELEASE-PLAN.md:228-243` |

**What is not claimed for 1.2:** no feature is released, no artifact is built, no tag exists, no live paid call was made,
no release gate PASS, no 1.2 live paid proof, and no sealed 1.2 artifact exists, and no trace hop was promoted. The uncommitted 1.2 work
(approval suspension, side effects, web replay, live specs) is WIP. The 441 backend claim and the sealed 1.1 live
results are not 1.2 evidence.

**Release-build blockers (current):** the dirty tree/uncommitted WIP, the missing 1.2 live
tool-turn/approval/security/resync proof, the absent axe dependency/approval, the empty raw trace capture, and the
unresolved seal topology (C10). See `plans/foresight/COMPILED-LEDGER.md:39-43,154` and
`plans/foresight/PROOF-RELEASE-PLAN.md:228-243`.

## Sealed released artifacts: HiveMind 1.1.0 (authoritative)

- Released 1.1.0 was declared in `web/package.json` and all four `backend/crates/*/Cargo.toml` at seal time; the
  working tree now declares `1.2.0-dev` (pre-release, see above) — the 1.1.0 values in this section remain the
  authoritative release record.
- **Streaming:** true SSE streaming is implemented — the provider adapter forwards ordered text fragments to the
  delta sink (`backend/crates/hiverouter-core/src/provider_executor.rs` `stream_reply`), usage settles from the
  documented final chunk, malformed/incomplete streams refuse by name. Offline matrix:
  `backend/crates/hiverouter-core/tests/provider_streaming.rs`; run log `docs/evidence/1.1-backend-tests.log`.
- **Replay:** `HISTORY_REVISION_SET` → read-only `HISTORY_SNAPSHOT`; canvas projects snapshot nodes/edges; browser
  proof recorded in the 1.1 wave.
- **State parity:** 16 states including `interrupted` (backend and web).
- **Artifact hashes (sealed 1.1.0 build):** `hiverouter.exe` `9c3174fb7c118143139ff283c08a3190c8e146a831717fb3ff8e25d6468d7207` (13,624,320 B); `hiverouter-cli.exe` `f757f55833f52f626cff6f962d31cb18d740dab53e314c5ba6048599f09214df` (5,372,416 B); `dist/HiveMind-1.1.0-win64.zip` `af2345f663643f1c0d1dbc18a298511817bf38090351d5f5b4842418f4d63e56` (8,613,122 B). Authoritative registry: `tools/release-manifest.json` (`version=1.1.0`, source `87079cecf6a4cf2212b9d41e56f4c7643a97c7fe`, tag `hivemind-1.1.0`). Zip extraction verified 6 members + CHECKSUMS.
- **Sealed artifacts re-hashed on disk (2026-11-06):** the two `target/release/*.exe` files and `dist/HiveMind-1.1.0-win64.zip` match the three hashes above.
- **Live paid-provider streaming smoke on the sealed release binary:** `target/release/hiverouter.exe` via the live e2e harness with `LIVE_ONE_PROMPT=1` and `E2E_SERVER_BIN=target/release/hiverouter.exe`; `live-streaming.spec.ts` **3 passed** (boot, no-key fail-closed, paid ordered `REPLY_DELTA -> REPLY_DONE -> RUN_STATUS completed`), and `live-replay.spec.ts` **2 passed** (unknown-run refusal, paid `HISTORY_SNAPSHOT` >=2 nodes with read-only UI). Raw evidence: `docs/evidence/live-1.1-paid.log` (5 passed, exit 0), `docs/trace/live/captures/w11/live-streaming.json` (10 frames), `docs/trace/live/captures/w11/live-replay.json` (12 frames, revision 0, 2 nodes). No key material was written to evidence.

## Test evidence at the sealed 1.1.0 baseline

- Backend 1.1 gate: **286 passed / 0 failed / 2 ignored** (`docs/evidence/1.1-backend-final.log`; includes the
  truncation, provider-error, prompt-asset and streaming suites).
- Web: **93/93** unit tests (15 files), typecheck 0 errors, production build green.
- Release build log: `docs/evidence/1.1-release-build-4.log` (web build, cargo build, deterministic zip, extraction
  verification, manifest write, absolute-path scan, DONE).

<!-- docs-truth:historical:start -->
## Historical: HiveMind 1.0.0 (SUPERSEDED)

The 1.0.0 artifacts below were sealed before the 1.1 streaming work and are retained for provenance only. They are
**not current** and must not be presented as the current release; the current authority is
`tools/release-manifest.json` after the sealed 1.1.0 build.

### 1.0.0 artifacts (historical)

| Artifact | SHA-256 | Bytes |
|---|---|---:|
| `hiverouter.exe` | `b17eb3d6469c347f623a4dedc94537b15bb633a76215828742624fa6fe062a9a` | 13,253,120 |
| `hiverouter-cli.exe` | `49f46e2be488dd675280f3dae7e8d455ac5eeec2aa46d7fd44f1c2557c386623` | 5,373,952 |
| `dist/HiveMind-1.0.0-win64.zip` | `46305fcb1fe1bee23d3645a94a82635a68ee47fbcd3ce0537df747de3c6105ef` | 8,444,021 |

The 1.0.0 zip contains both binaries, `README.md`, `RUNBOOK.md`, `RELEASE.md`, and `CHECKSUMS.txt`; an extraction
smoke listed all six files and verified the archive opened.

### 1.0.0 release-binary live smoke (historical)

Started `target/release/hiverouter.exe` on `127.0.0.1:8800` with a fresh data directory and the key only in the
process environment.

- `GET /health` → `ok`, version `1.0.0`.
- `GET /` → 200, embedded UI HTML present (`id="root"`), 774 bytes.
- `SETTINGS_STATUS` → `{"provider":"deepseek","model":"deepseek-flash","key_present":true,"thinking":"enabled","reasoning_effort":"high"}`.
- `SUBMIT_PROMPT` `Reply with exactly HIVE-ARTIFACT-OK` → `RUN_CREATED` → nodes/reply events → `REPLY_DONE` usage
  `{prompt:91, completion:8, total:99}` → `RUN_STATUS completed`; reply `HIVE-ARTIFACT-OK`.
- `HISTORY_REVISION_SET` revision 0 → `HISTORY_SNAPSHOT` with 2 graph nodes (real snapshot; not fabricated).
- The key never appeared in the server log, settings payload, or evidence.
- Child-process isolation: shell/tool spawns remove `DEEPSEEK_API_KEY` before execution, pinned by the
  `child_process_does_not_inherit_the_provider_key` test.
- Source commit for that build: `6a2b1c6d461a81e7ee66951d0d0eb6b1063bf540`; manifest: `tools/release-manifest.json`
  (records the 1.0.0 build until the sealed 1.1.0 build replaces it); web-dist hashes were recorded per file.
- Retired limitation: the 1.0.0 leaf reply was emitted as one delta after a non-streaming completion. This behavior
  is superseded by the 1.1.0 SSE streaming; it is historical and must not be stated as current.

### 1.0.0 test evidence (historical)

- Rust workspace on sealed commit `6a2b1c6`: 237 passed / 0 failed / 2 ignored (`docs/evidence/final-gate-tests.log`).
- Web: 46/46 unit tests, typecheck 0 errors, Vite production build green (`docs/evidence/final-web-tests.log`,
  `final-web-build.log`).
- Live provider adapter: planner node + `HIVE-PROVIDER-OK`, usage 99 tokens.
- Key scan: strict credential pattern (`sk-[A-Za-z0-9]{32,}`) = 0 occurrences across tracked non-vendor files.

### 1.0.0 final tag gate (historical)

`v1.0.0` remains immutable. Moving it requires owner confirmation and records the superseded commit.
<!-- docs-truth:historical:end -->
