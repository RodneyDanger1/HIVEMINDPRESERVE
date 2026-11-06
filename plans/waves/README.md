# HiveMind 1.2 wave index (W1–W12)

Status snapshot: **2026-11-06**, HEAD `57d5157` (`hivemind-1.1.0-3-g57d5157`) + uncommitted W11/W12 work. The working
tree now declares **`1.2.0-dev`** (pre-release; see “W12 Wave 2 — version truth” below) while the sealed 1.1.0 tag
(`87079ce`) and its artifacts remain the authoritative release. This index is documentation only: it records what is
**tracked/verified**, what is **landed but uncommitted**, what is **blocked**, and what is still **design**. No build,
test, Playwright or release script was executed for this wave (W12 rule 2); statuses come from tracked files, committed
evidence logs and the W11/W12 plans.

Legend: **DONE** = tracked deliverable + committed evidence; **LANDED-UNCOMMITTED** = file exists in the working tree
but is not yet committed; **PARTIAL** = mixed tracked/untracked; **DESIGN** = plan only, no implementation claimed;
**BLOCKED** = evidence gate cannot be satisfied; **GAP** = wave identifier with no artifact.

| Wave | Lane / deliverable | Status | Evidence | Dependencies |
|---|---|---|---|---|
| W1 | Core recovery gates — `backend/crates/hiverouter-core/tests/recovery_gates.rs` (tracked) | **DONE** — suite ran green in the 1.1 backend gate | `docs/evidence/1.1-backend-final.log` (“Running tests\recovery_gates.rs”) | W1 depends on the W3 streaming/executor baseline it pins |
| W2 | Provider error matrix — `backend/crates/hiverouter-provider/tests/error_matrix.rs` (tracked) | **DONE** — suite ran green | `docs/evidence/1.1-backend-final.log` (“Running tests\error_matrix.rs”) | W1 |
| W3 | True SSE streaming — `provider/src/provider_executor.rs` `stream_reply`, `tests/provider_streaming.rs` | **DONE** — shipped in 1.1.0; truncation→`E_PROVIDER_INCOMPLETE` test green | `docs/evidence/1.1-backend-final.log` (`length_finish_reason_is_incomplete_not_success ... ok`); `docs/RELEASE-EVIDENCE.md` | W1, W2 |
| W4 | Live e2e harness — `e2e/helpers/live-server.ts`, `e2e/specs/live-streaming.spec.ts`, `live-replay.spec.ts` (tracked; W11 adds capture writes) | **LANDED** / live proof **BLOCKED** — no operator key; observed run `{"expected":0,"skipped":5}`; claim demoted by audit | `docs/evidence/live-trace-1.1.md` (`E_LIVE_TRACE_BLOCKED_NO_OPERATOR_KEY`); `plans/verification/RELEASE-1.1-AUDIT.md` R1 | W3 |
| W4 | Replay/state browser proof — `web/tests/graph.replay.test.ts`, `e2e/specs/replay-state.spec.ts` (tracked) | **DONE** — 7 replay tests inside the 93/93 web suite; replay spec tracked | `docs/evidence/1.1-release-build-4.log` (`graph.replay.test.ts (7 tests)`; `Tests 93 passed (93)`) | W3 |
| W4 | Trace capture truth — `docs/trace/CAPTURE-REGISTRY.md` + trace docs (tracked) | **PARTIAL** — registry/docs done; raw `docs/trace/live/trace-capture.json` has `surfaces: []` (empty capture, recorded in the registry) | `docs/trace/CAPTURE-REGISTRY.md`; `docs/trace/live/README.md` | W9 trace-truth |
| W5 | Prompt-asset wiring — `core/src/prompts.rs`, `tests/prompt_assets*.rs` (tracked) | **DONE** — prompt-asset suites green on the production path | `docs/evidence/1.1-backend-final.log` (`Running tests\prompt_assets.rs`, `health_reports_prompt_assets_loaded_on_the_production_path ... ok`) | W1 |
| W5B | UI lifecycle/streaming tests — `web/tests/apiLifecycle.test.ts`, `web/tests/streamingRender.test.ts` (tracked) | **DONE** — both files green inside 93/93 | `docs/evidence/1.1-release-build-4.log` (`apiLifecycle.test.ts (13 tests)`, `streamingRender.test.ts (5 tests)`) | W4 replay |
| W6 | — | **GAP** — no W6 file exists in the wave corpus; release engineering was serialized as W7. Recorded here so the gap is visible, not silently skipped. | `git ls-files plans/waves` (no `W6-*`) | — |
| W7 | Release engineering — `tools/build-release.ps1`, `tools/release-manifest.json`, sealed 1.1.0 artifacts | **DONE / VERIFIED** — manifest `version=1.1.0`, `source_commit=87079ce`, `tag=hivemind-1.1.0`; build log ends `DONE - release sealed` | `docs/RELEASE-EVIDENCE.md`; `docs/evidence/1.1-release-build-4.log`; `dist/HiveMind-1.1.0-win64.zip` (8,613,122 B) | W1–W5B |
| W8 | Docs truth checker — `tools/qa/check-docs-truth.mjs` (tracked) | **DONE** — 11/11 checks in the 1.1 audit; **extended by W12 Seat 8** to parse `UI-COMMANDS-v1` event/state counts | `tools/qa/check-docs-truth.mjs`; `plans/verification/RELEASE-1.1-AUDIT.md` | W7 |
| W8 | Docs truth content — `docs/RELEASE-EVIDENCE.md`, current-facing docs corrections (tracked) | **DONE** — version/streaming/history claims and 1.0 historical masking verified | `node tools/qa/check-docs-truth.mjs` output in the 1.1 audit | W8 checker |
| W9 | GUI e2e gates — `e2e/specs/gui-gates.spec.ts` (tracked), `e2e/GATE-EXCLUSIONS.md` (modified), `e2e/specs/gui-residual.spec.ts` (**untracked**) | **PARTIAL** — tracked gates + exclusion registry landed; W11 residual-spec and exclusions update are uncommitted | `e2e/GATE-EXCLUSIONS.md`; `plans/waves/W11-GUI-RESIDUAL.md` | W8 |
| W9 | Trace truth — `docs/trace/controls.json` + trace docs (tracked) | **DONE (docs)** — current counts: **1 WIRED-AND-WORKS / 264 PENDING / 5 MISSING / 49 UNVERIFIED of 319 hops**; raw capture still empty | `docs/trace/controls.json`; `docs/trace/QA-HARNESS.md`; `docs/trace/TRACE-SCHEMA.md` | W4 trace |
| W10 | Tool-loop / Tauri decision — `plans/waves/W10-TOOL-LOOP-DECISION.md` | **DONE (decision)** — decision **NO for 1.1**; blockers B1–B7; 1.2 target design recorded | `plans/waves/W10-TOOL-LOOP-DECISION.md` | W3 |
| W11 | Contract patch — `contract/src/event.rs`, `command.rs`, `schema.rs`, `schemas/contract-v1.json`, `web/src/api/contract.ts` (modified, uncommitted) | **LANDED-UNCOMMITTED** — 23 events / 16 states + optional approval fields; **no runtime producer/consumer yet** | `plans/waves/W11-CONTRACT.md`; contract tests (`schema_roundtrip` 23, `web/tests/contract.test.ts` 23) | W10 |
| W11 | Tool-call assembler — `provider/src/tool_assembler.rs` + `provider/tests/tool_assembler.rs` (**untracked**) | **LANDED-UNCOMMITTED** — pure provider validation/limits; **zero production callers** (no tool loop) | `plans/waves/W11-TOOL-ASSEMBLER.md` | W11 contract |
| W11 | Gate harness — `tools/qa/run-gates.ps1` (**untracked**) | **LANDED-UNCOMMITTED** — G0–G7; W12 Seat 8 adds `-RequireE2E`, `-Release` (`source_commit==HEAD`), skip≠PASS | this file; `plans/waves/W11-GATES.md` | W7, W8 |
| W11 | Suspension/UI/trace/GUI designs — `W11-SUSPENSION-DESIGN.md`, `W11-UI-TOOL-ACTIVITY.md`, `W11-TRACE.md`, `W11-GUI-RESIDUAL.md` | **DESIGN** — run suspension, UI tool activity and live capture are designs only; no runtime/UI implementation claimed | the four plan files | W10, W11 contract |
| W12 | 1.2 interface freeze + version line — `plans/waves/W12-INTERFACES.md`; versions now `1.2.0-dev` | **FROZEN interface / VERSION BUMPED (Wave 2) + POST-VERSION TRUTH (Wave 3)** — shared 1.2 interface contract; W11 contract/assembler/gates lanes are landed but uncommitted; version strings bumped to `1.2.0-dev` in 4 crate manifests + 4 `Cargo.lock` entries + web package/lock root. Wave 3: docs checker derives the sealed release from `tools/release-manifest.json` (asserts current source line + preserved sealed 1.1 hashes); `run-gates.ps1 -Release` accepts a documented pre-release tree only with `GATES: PRE-RELEASE` **exit 3**, never PASS. **No runtime claim yet**: the tool loop, approval suspension/resume, tool activity UI and 1.2 artifacts are **not implemented/run**; the checkout is not a 1.2 release and `hivemind-1.1.0` remains the build source. Evidence: `plans/waves/W12-INTERFACES.md`; `docs/RELEASE-EVIDENCE.md` current-line note; version files; W12 Wave 2/3 sections below. | W11 lanes |

## Evidence rule

A wave is **DONE** only with a tracked deliverable plus committed evidence (log/hash/doc). Untracked deliverables are
**LANDED-UNCOMMITTED** regardless of content quality, and design documents never imply implementation. The 1.1 release
tag `hivemind-1.1.0` (`87079ce`) and its sealed artifacts are unchanged by any W11/W12 work.

## W12 Wave 2 — version truth (Seat 8, 2026-11-06)

**Outcome:** the working tree now declares **`1.2.0-dev`** consistently (pre-release). The sealed **1.1.0** tag/source
(`87079ce`), its artifacts and `tools/release-manifest.json` are unchanged and remain the authoritative release record.
**No runtime claim:** the W12 tool loop / approval suspension / tool-activity UI are not implemented or run; no 1.2
artifact was built. This wave changed version strings and documentation only (no code/tests/gates edits).

| Location | OLD | NEW | Status |
|---|---|---|---|
| `backend/crates/hiverouter-{contract,core,provider,tools}/Cargo.toml` (4) | `1.1.0` | `1.2.0-dev` | **changed 4/4** |
| `backend/Cargo.lock` — the four `hiverouter-*` package entries | `1.1.0` | `1.2.0-dev` | **changed 4/4**; 5 unrelated third-party crates that are also `1.1.0` (`http-body`, `idna`, `unicode-general-category`, `utf8_iter`, `web-time`) **untouched** |
| `web/package.json` | `1.1.0` | `1.2.0-dev` | **changed** |
| `web/package-lock.json` — root (`version`) + `packages[""]` (`version`) only | `1.1.0` | `1.2.0-dev` | **changed 2/2**; dependency versions untouched |
| `tools/release-manifest.json` | `1.1.0` | `1.1.0` | **unchanged — authoritative sealed 1.1.0 release** |
| git tag `hivemind-1.1.0` / source `87079ce` | — | — | **unchanged**; HEAD `57d5157` is ahead (post-release evidence + uncommitted W11/W12 work) |
| contract schema version (`hiverouter.v1`) | — | — | **unchanged** (version bump is package metadata only) |

**Wave-3 resolution (the debt documented here in Wave 2 is now closed in this tree):**
- `tools/qa/check-docs-truth.mjs` now derives the sealed release from `tools/release-manifest.json`
  (version/source/tag), asserts the **current source line** (`1.2.0-dev`) in `RELEASE-EVIDENCE` plus the preserved
  sealed `1.1.0` artifact hashes, and fails if a pre-release tree is presented as the released version — it no longer
  pins the literal `1.1.0`.
- `tools/qa/run-gates.ps1 -Release` now accepts this **documented pre-release tree** (manifest still `1.1.0`, tag target
  == manifest `source_commit` `87079ce`, sealed source an ancestor of HEAD, docs state both lines) only with
  `GATES: PRE-RELEASE` and **exit 3** — it never prints a release PASS. Sealed trees keep the strict
  `manifest.source_commit == HEAD` requirement. Full checks: §“W12 Wave 3 — post-version truth” below.

**Validation performed (static only; NO build/run):** 4/4 crate manifests `1.2.0-dev`; 4/4 `Cargo.lock` package entries
`1.2.0-dev` with the 5 third-party `1.1.0` crates untouched; `web/package.json` and both `web/package-lock.json` root
entries `1.2.0-dev`; both JSON files parse (`node -e JSON.parse`, exit 0); `release-manifest.json` still `1.1.0`;
`hivemind-1.1.0` tag source still `87079ce`; HEAD `57d5157`; scope diff = only the 9 files of this wave.

### W12 Wave 2 — Phase 1 JSON (logic-gatekeeper, recorded before edits)

```json
{
  "phase1": {
    "architecture": "Version truth is duplicated across four crate manifests (backend/crates/*/Cargo.toml), backend/Cargo.lock package entries, web/package.json and web/package-lock.json root entries; release evidence (docs/RELEASE-EVIDENCE.md) and plans/waves/README.md describe release line and wave status. The sealed 1.1.0 tag/source (87079ce) and tools/release-manifest.json remain the authoritative release record and are out of scope.",
    "dataFlow": "Cargo.toml version -> Cargo.lock hiverouter package version -> cargo metadata/next build; web/package.json version -> package-lock root version -> npm tooling; version strings -> docs/RELEASE-EVIDENCE.md current line and plans/waves/README.md status snapshot; release-manifest.json/tag stay at 1.1.0 until a new sealed 1.2 build.",
    "edgeCases": ["partial bump (some crates/lock/web left at 1.1.0) -> version divergence; must update all 4 crates + 4 lock entries + web package/lock root 2 entries", "third-party Cargo.lock packages that happen to be version 1.1.0 (stdlib/infra crates) -> must NOT be touched", "web/package-lock only root version entries change; dependency versions untouched", "docs check-docs-truth.mjs still pins REQUIRED_VERSION_RE 1.1.0 and release-manifest.json is still 1.1.0 -> release gate/checker expectations change until a later wave; must be documented, not silently fixed (gates out of scope)", "semver pre-release 1.2.0-dev must be valid in Cargo and npm package metadata (it is)"],
    "rootCause": "The working tree still advertises 1.1.0 everywhere after 1.1.0 was sealed, so W11/W12 changes are indistinguishable from the released source; a consistent 1.2.0-dev pre-release bump plus explicit release-evidence/wave-index truth is required before any 1.2 development or gate claim."
  }
}
```

## W12 Wave 3 — post-version truth (Seat 8, 2026-11-06)

**Outcome:** after the Wave-2 `1.2.0-dev` bump, the docs checker and the release gate now treat the checkout as a
**documented pre-release**: the checker expects the current source line `1.2.0-dev` *and* preserves the sealed `1.1.0`
artifact record; `run-gates.ps1 -Release` accepts a documented pre-release tree only with an explicit **PRE-RELEASE**
result (**exit 3**) and never prints a release PASS. No runtime claim; no build/run executed.

| File | Change | Exact check / behaviour |
|---|---|---|
| `tools/qa/check-docs-truth.mjs` | Now reads `tools/release-manifest.json` → `sealedVersion`, `sealedSource`, `sealedTag`; derives `sourceVersionRe`/`sealedVersionRe`; `preRelease = currentVersion !== sealedVersion` | `VERSION: web and all backend crates agree` compares against the **current source** version (`1.2.0-dev`); release-facing docs (`README.md`, `docs/RELEASE.md`) must state the **sealed release** `1.1.0` and — when pre-release — must **not present `1.2.0-dev` as the released version**; `docs/RELEASE-EVIDENCE.md` must state the current source line **and** the sealed release, must label the tree **pre-release**, must cite the **sealed tag + sealed source**, and must preserve **every sealed artifact/package sha256** from the manifest (2 artifacts + package zip; `SEALED: RELEASE-EVIDENCE preserves sealed artifact hashes`, `SEALED: RELEASE-EVIDENCE records the sealed tag and source`); claims JSON/CSV checks assert the **sealed release** record. Hardcoded `REQUIRED_VERSION_RE` removed (0 references); summary prints `current source version` + `sealed release version (pre-release working tree)`. |
| `docs/RELEASE-EVIDENCE.md` | Documented the Wave-3 policy in the OLD→NEW block | States current source `1.2.0-dev` + sealed `1.1.0`, tag `hivemind-1.1.0` @ `87079ce`, “pre-release”, and the `-Release` **PRE-RELEASE exit 3 / never PASS** rule; sealed 1.1.0 artifact section preserved unchanged. |
| `tools/qa/run-gates.ps1` | `-Release` preflight now distinguishes sealed vs documented pre-release trees; `G7` relaxes only the manifest/version equality under accepted pre-release; summary/dry-run add PRE-RELEASE handling; exit code 3 documented | **Sealed tree:** unchanged strict `manifest.source_commit == HEAD`. **Pre-release tree accepted only when all hold:** `version` contains `-` and != `manifest.version`; manifest tag exists and **tag target == `manifest.source_commit`**; `HEAD != manifest.source_commit`; sealed source is an **ancestor of HEAD**; `docs/RELEASE-EVIDENCE.md` contains current version, sealed version, sealed tag, sealed source prefix (8 chars), and `pre[- ]release`; `plans/waves/README.md` contains current version + `pre[- ]release`. Accepted → `PRE-RELEASE TREE ACCEPTED` warning; `G7` does not fail manifest-version equality and, in accepted pre-release mode, reports workspace artifact/web-dist drift from the sealed manifest as **pre-release drift warnings** (dev builds are expected to differ) while keeping **tag/source and the sealed package zip/sidecar/CHECKSUMS strict**; summary prints `GATES: PRE-RELEASE (not a release)` and exits **3**; `-DryRun` likewise exits 3 for an accepted pre-release. Any missing condition → `E_GATE_PRERELEASE` preflight failure (no acceptance), and release mode still fails on skipped gates/`-Rehearsal`/dirty tree. Never prints release `GATES: PASS` on a pre-release tree. |

### W12 Wave 3 — Phase 1 JSON (logic-gatekeeper, recorded before edits)

```json
{
  "phase1": {
    "architecture": "docs truth is enforced by tools/qa/check-docs-truth.mjs (node) which derives the working version from web/package.json + crate manifests and must now also derive the sealed release version/source/tag from tools/release-manifest.json; docs/RELEASE-EVIDENCE.md is the human record (current source line vs sealed 1.1 artifacts); tools/qa/run-gates.ps1 G0 preflight owns release-mode acceptance; plans/waves/README.md records wave status.",
    "dataFlow": "web/package.json version -> checker currentVersion; release-manifest.json {version, source_commit, tag} -> checker sealedVersion/sealed tag+sources and run-gates release preflight; RELEASE-EVIDENCE + wave README strings -> checker/run-gates acceptance; run-gates -Release: detect pre-release tree (currentVersion != manifest.version and contains '-'), require manifest tag target == manifest source_commit == documented sealed source, source ancestor of HEAD, RELEASE-EVIDENCE documents both versions + tag/source + preserved sealed artifact hashes, wave README documents the dev line; then report PRE-RELEASE (exit 3), never green PASS.",
    "edgeCases": ["current 1.2.0-dev != manifest 1.1.0 with tag target 87079ce and source 87079ce, HEAD 57d5157 ahead -> documented pre-release tree accepted only after all string/ancestry checks", "manifest still 1.1.0 but RELEASE-EVIDENCE loses sealed artifact hashes -> checker SEALED check fails", "pre-release tree silently claiming release version in README/RELEASE -> checker fails", "sealed tree (currentVersion == manifest.version) keeps strict source_commit == HEAD requirement", "pre-release with no documented tag/source in RELEASE-EVIDENCE -> run-gates fails E_GATE_PRERELEASE (no false acceptance)", "checker must still pass on claims register which documents sealed 1.1.0, not the dev version"],
    "rootCause": "The Wave-2 1.2.0-dev bump left the docs checker pinned to the literal 1.1.0 and the release preflight requiring manifest.source_commit == HEAD; both misclassify the documented pre-release working tree (manifest 1.1.0/tag 87079ce, HEAD ahead) and a naive relaxation could produce a false green release claim."
  }
}
```

**Validation performed (static only; NO build/run, checker/gate not executed):** `node --check tools/qa/check-docs-truth.mjs`
→ exit **0**; PowerShell parser on `tools/qa/run-gates.ps1` → **0 parse errors**; `REQUIRED_VERSION_RE` references = **0**;
Wave-4 re-validation of the checker markers: `releaseManifest`=6, `sealedVersion`=13, `preRelease`=4,
`sealedHashEntries`=7, `sealedPackage`=2, `sealedTagSourceDocumented`=3, sealed-hash check=1, sealed tag/source check=1;
run-gates markers: `$preReleaseCandidate`=4, `E_GATE_PRERELEASE`=15, `$script:PreReleaseAccepted`=6, `PRE-RELEASE`=17,
`exit 3`=3, “gates not necessarily executed”=1; current-truth conditions verified statically:
`web_version=1.2.0-dev`, `manifest_version=1.1.0` source `87079cec` tag `hivemind-1.1.0`, tag target == source,
source ancestor of HEAD = **True**, RELEASE-EVIDENCE has current+sealed+tag+source+pre-release, wave index has
current+pre-release, sealed artifact/package hashes present **3/3** (exe, cli, zip).

## W12 Wave 4 — static logic audit (Seat 8, 2026-11-06)

Read-only static audit (`/logic-debugger` + `/forgewright-lead`) of: version/manifest/tag/pre-release logic, event/state
doc counts, gate skip semantics, release-mode conditions, and evidence claims vs actual files. No build/test/commit.

**Proven errors fixed (gate/doc only):**
1. `tools/qa/check-docs-truth.mjs` sealed-preservation check covered only `releaseManifest.artifacts[]` (exe, cli); the
   sealed **package zip** hash presented in `RELEASE-EVIDENCE` was not enforced. Fixed: sealed hash entries are now
   artifacts + `package.zip_sha256` (**3/3** present) and the check fails if any is removed.
2. The checker did not verify that `RELEASE-EVIDENCE` cites the sealed **tag/source**; added
   `SEALED: RELEASE-EVIDENCE records the sealed tag and source` (`hivemind-1.1.0` @ `87079ce`).
3. `run-gates.ps1` exit-code 3 help said “gates executed”, but `-DryRun -Release` can exit 3 without executing gates;
   wording corrected to “gates not necessarily executed”.
4. **Late fix — pre-release acceptance was unreachable:** `G7` enforced the sealed workspace artifact/web-dist hashes
   against dev-built outputs (observed: expected `web/dist/assets/index-DRvC4Qb-.js` missing; `web/dist/index.html`
   `9B468C98…` vs sealed `01f31347…`), so an accepted pre-release tree would still fail `G7` and never reach
   `GATES: PRE-RELEASE`. Fixed: in accepted pre-release mode `G7` keeps **tag/source + sealed package
   zip/sidecar/CHECKSUMS strict** but reports workspace artifact/web-dist drift as `pre-release drift (not a failure)`
   warnings; docs updated. No false green: result remains exit 3, never release PASS.

**Audited and found correct (no change):** version `1.2.0-dev` (web/crates) vs sealed `manifest.version=1.1.0` /
`tag=hivemind-1.1.0` / `tag target == source_commit 87079ce` / source ancestor of HEAD; checker derives sealed fields
from the manifest (no hardcoded version regex); UI-COMMANDS **23 events / 16 states** match `event.rs`/`agentState.ts`;
`CommandName::ALL` = 17 and **65 mapped refusal codes** (the note’s “65 refusal codes” is correct; §5 is the Phase-1
subset); gate skips are never counted as PASS (normal mode records `SKIPPED`; `-RequireE2E` makes a skip FAIL;
`-Release` fails on any skip); release mode rejects a pre-release tree unless every documented tag/source/ancestry/docs
condition holds, and accepted pre-release runs exit **3** with `GATES: PRE-RELEASE`, never release PASS; `G7` in accepted
pre-release mode is drift-tolerant for workspace artifacts/web-dist but strict for tag/source + sealed package;
evidence claims match files: `live-1.1-paid.log` 5 passed / EXIT=0, capture frames 10 and 12 (replay chosen
revision 0 / 2 nodes), backend log 286/0/2 EXIT=0, web 93/93, zip 6 members + hash match.

**Residual risks (not fixed):**
- R1: Neither the checker nor the gates were executed in this audit (NO build/test); behaviour is static/syntax-verified
  only and must be confirmed by the Lead’s next G1/`-Release` run.
- R2: The working tree is dirty from other lanes, so an actual `-Release` run is blocked by `E_GATE_DIRTY_TREE` until
  the Lead commits; the documented pre-release acceptance path is currently unexercised.
- R3: `RELEASE-EVIDENCE`’s baseline line “Docs truth checker: 11/11 checks” is **seal-time history**; the checker has
  since been extended (Wave 1/3/4 — static 17 call sites, 19 runtime checks). A reader could mistake it for the current
  check count.
- R4: In **non-release** mode a run with named skips exits 0 and prints `GATES: PASS (with N named SKIP(s); skips are
  not PASS)`; strict consumers keying only on the `GATES: PASS` substring could misread it (release mode and
  `-RequireE2E` fail skips).
- R5: `UI-COMMANDS-v1.md` still carries the “FROZEN CONTRACT (Phase 1 freeze)” title and §5 lists only the Phase-1
  refusal subset (46 of 65 mapped codes); the 1.2 additive note clarifies counts but the title/section scope may confuse.
- R6: Pre-release acceptance in `run-gates` checks **text presence** for docs/tag/source; G7 keeps tag/source + sealed
  package strict but **tolerates workspace artifact/web-dist drift** (warnings) in accepted pre-release mode, and G1
  enforces the sealed doc hashes. A misconfigured gate list (skipping G1/G7) would weaken the proof.
- R7: The checker treats any version mismatch as pre-release; an accidental non-`-dev` bump (e.g. `2.0.0` before seal)
  therefore requires pre-release labelling — correct fail-closed behaviour, but possibly surprising.
- R8: Sidecar/CHECKSUMS sha256 are not part of the checker's sealed-preservation check (2 artifact + zip hashes are);
  G7 still verifies the sealed sidecar/CHECKSUMS files strictly, and the manifest remains the authority.
- R9: Because G7 tolerates workspace drift in pre-release mode, the dev tree's binaries/`web/dist` are not
  cryptographically tied to the sealed record by the gate run itself; the sealed zip/sidecar/CHECKSUMS + tag/source are
  strict, and the run can never print a release PASS (exit 3).

## W12 Seat 8 — Phase 1 JSON (Wave 1 record, logic-gatekeeper)

```json
{
  "phase1": {
    "architecture": "Release/docs truth surfaces: docs/contracts/UI-COMMANDS-v1.md is the frozen wire vocabulary; backend/crates/hiverouter-contract/src/event.rs owns EventName::ALL (23) and state.rs owns AgentState (16); web/src/state/agentState.ts mirrors 16 states; tools/qa/check-docs-truth.mjs enforces docs truth; tools/qa/run-gates.ps1 is the one-command gate (G0-G7) including G4 guarded e2e; plans/waves/ is the wave record corpus (W1-W12 plus W12 interface freeze).",
    "dataFlow": "Contract source -> UI-COMMANDS-v1 doc; web/agentState.ts -> UI-COMMANDS-v1 states; check-docs-truth.mjs reads both sources and the doc -> PASS/FAIL (exit 1); run-gates G0 reads git HEAD/manifest/tag/dirty state, G4 runs guarded e2e or skips by name, summary counts PASS/FAIL/SKIPPED; plans/waves/README.md indexes wave evidence paths.",
    "edgeCases": ["UI-COMMANDS doc stale (21 events/15 states) while contract has 23/16 -> docs checker must fail", "G4 skipped because E2E_APP_READY!=1 or E2E_BASE_URL unset -> must be SKIPPED, never PASS; -RequireE2E must make it fail", "release mode with manifest source_commit != HEAD (HEAD 9651722 vs 87079ce) -> must fail closed", "-SkipE2E combined with -RequireE2E -> preflight failure", "wave index lists uncommitted/untracked lanes honestly (no DONE claim without tracked evidence)"],
    "rootCause": "The frozen contract doc and gate harness lag the working contract: UI-COMMANDS-v1 still advertises 21 events/15 states, the docs checker never parses that file, and run-gates can record a G4 skip while still printing a green PASS with no release-mode source_commit==HEAD guard."
  }
}
```

## W12 Seat 8 — validation performed (static only; no build/run)

- `node --check tools/qa/check-docs-truth.mjs` → exit **0**.
- PowerShell parser (`[System.Management.Automation.Language.Parser]::ParseFile`) on `tools/qa/run-gates.ps1` →
  **0 parse errors**.
- `UI-COMMANDS-v1.md` §3 lists **23** distinct contract events (incl. `TOOL_CALL`, `TOOL_RESULT`); §4 lists **16**
  states (incl. `interrupted`) — counted by static grep on the doc, matching `event.rs` (23) and `agentState.ts` (16).
- Not executed: docs checker run, gate harness, cargo, npm, Playwright, release script (W12 rule 2).
