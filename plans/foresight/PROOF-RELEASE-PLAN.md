# FORESIGHT PLANNER 4 — PROOF / RELEASE CLOSURE PLAN (items 12, 13, 18–25)

**Mode:** read-only planning. This file is the only write. No product code, tests, configs, gate scripts, git, keys, paid calls or builds were touched.
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **Observed at plan time:** HEAD `37f0e7c` (`hivemind-1.1.0-4-g37f0e7c`), in-flight uncommitted edits in 8 tracked product files (runtime loop/suspension lanes: `core/src/{execution,http,provider_executor,recovery,tools_bridge}.rs`, `core/tests/{run_lifecycle,tool_loop_engine}.rs`, `tools/src/approval.rs`) plus this new plan file; working tree declares **`1.2.0-dev`** (web + 4 crates, Wave-2 bump); sealed release record remains **1.1.0** (`tools/release-manifest.json`, tag target/source `87079ce`).
**Skills:** `/forgewright-recon` → `/forgewright-lead` → `/logic-gatekeeper` → `/integration-logic` → `/logic-debugger`. Phase-1 gating: this is a planning document only; no mutation of product/release state is performed, so the logic-gatekeeper Phase 2 mutation rule does not apply.

**Item-number assumption (recorded explicitly):** the Lead assigned closure items **12, 13, 18, 19, 20, 21, 22, 23, 24, 25** without a register in the repo. They are interpreted as the areas the assignment names: **12** = suspension proof, **13** = live tool-turn proof, **18** = trace promotion procedure/thresholds, **19** = G-UI closure, **20** = axe subcheck closure, **21** = strict gate closure, **22** = 1.2 seal/release execution, **23** = blockers/risks, **24** = non-claims, **25** = naming/evidence table. If the register's labels differ, the deliverables below are the work; only the labels change.

---

## 0. Verified current state (read-only recon)

| Fact | Evidence / command |
|---|---|
| Sealed release = 1.1.0; tag `hivemind-1.1.0` → `87079ce`; old tags `hivemind-1.0.0`/`v1.0.0` unchanged | `git rev-parse hivemind-1.1.0^{commit}`; `tools/release-manifest.json` (`version=1.1.0`, `tag_target=source_commit=87079ce`, zip `dist/HiveMind-1.1.0-win64.zip`) |
| Working tree declares `1.2.0-dev`; **no 1.2 artifacts/tag exist** | `web/package.json`, `backend/crates/*/Cargo.toml`; `git tag -l` |
| 1.1 live paid proof committed (READ-level evidence) | `docs/evidence/live-1.1-paid.log` (5 passed / `EXIT=0`); `docs/trace/live/captures/w11/live-streaming.json` (10 frames), `live-replay.json` (12 frames, observed/chosen revision 0, 2 nodes); all tracked |
| Trace truth: **1 WIRED-AND-WORKS / 264 PENDING / 5 MISSING / 49 UNVERIFIED of 319**; raw capture **EMPTY** (`surfaces: []`, 286 B at `2026-11-06T20:25:04.044Z`); `raw_capture_backed_wired=1` | `docs/trace/controls.json` `_meta.counts`/`_meta.live_capture`; `docs/trace/live/trace-capture.json`; `docs/trace/CAPTURE-REGISTRY.md` rules 1–4 |
| Trace promotion inputs: 5 registry entries — `CAP-S10-REPLAY` (UNVERIFIED/NOT_CAPTURED), `CAP-S15-19-CARDS` (MISSING), `CAP-S27-STREAMING` (UNVERIFIED), `CAP-SAPPROVAL` (PENDING), `CAP-S22-RECHECK` (partial pre-HEAD raw) | `docs/trace/capture-registry.json`; `CAPTURE-REGISTRY.md` rows |
| Guarded e2e + paid gates exist: `LIVE_ONE_PROMPT=1` + key required; `E2E_SERVER_BIN`/`target/{debug,release}/hiverouter.exe`; skip reasons named | `e2e/helpers/live-server.ts:17,24-28,101-137,322-425`; `e2e/specs/live-streaming.spec.ts:112-191`, `live-replay.spec.ts:72-150` |
| Mock UI specs already exist for `TOOL_CALL`/`TOOL_RESULT` and approval overlay (no live server/key) | `e2e/specs/tool-loop.spec.ts` (7 tests), `e2e/specs/approval-flow.spec.ts` (8 tests); helpers `replay-harness.ts` exporters |
| **No committed runtime tool loop / suspension exists yet:** stream pump discarded `ToolCallDelta`; W10/W11 blockers B1–B8. In-flight uncommitted edits now touch `core/src/{execution,provider_executor,recovery,tools_bridge}.rs`, `tools/src/approval.rs` and new `tests/tool_loop_engine.rs` (unverified; no build/test/live evidence yet) | `core/src/provider_executor.rs:171-173` (pre-edit behavior `tool_calls_seen = true` only); `git status`; `plans/waves/W10-TOOL-LOOP-DECISION.md:92-106`; `plans/waves/W11-SUSPENSION-DESIGN.md:392-412` |
| Gate harness: G0–G7; `-RequireE2E` makes a G4 skip a FAIL; `-Release` requires zero skips and, on a pre-release tree, emits `GATES: PRE-RELEASE` and exits **3**; sealed tree requires `manifest.source_commit == HEAD` | `tools/qa/run-gates.ps1:23-45,125-133,248-315` (observed flags/exit text); `tools/qa/README.md` (component/step description) |
| Release sealer: deterministic zip/CHECKSUMS/sidecar/manifest; requires tag at HEAD unless `-AllowTagAncestor` (rehearsal only); records `source_commit=HEAD`, `tag_target`, `tag_behind_commits` | `tools/build-release.ps1:23-31,174-250,530-606`; `-DryRun` writes nothing |
| G-UI exclusions: `G-UI-3`+`W1` foreign Origin, `G-UI-10`+`W2` resync, `G-UI-8` axe subcheck | `e2e/GATE-EXCLUSIONS.md:10-14`; registry test `e2e/specs/gui-gates.spec.ts:335-355` |
| `apply-live-trace.mjs` merges a capture into controls but has **no empty-capture guard**; `verify-harness.mjs` does not verify evidence-pointer existence/content | `tools/qa/apply-live-trace.mjs:18-58`; `plans/waves/W4-TRACE-CAPTURE.md:56,51-59` |

**Material release-topology finding (affects item 22):** the sealer requires the tag/target at HEAD (`build-release.ps1:239-244`) and writes `source_commit = HEAD` (`:545`), but the seal artifacts/manifest/evidence can only be committed *after* they are built. The 1.1 precedent has tag/source `87079ce` while the artifacts/evidence landed in later commits (`c0b09a6`, `57d5157`). Under the current gate, a strict green `-Release` cannot exist on that topology (`manifest.source_commit == HEAD`). Closure therefore requires an explicit **two-commit seal model** (source commit `C`, seal commit `D`) plus a gate/manifest change, described in item 22.

---

## 1. Closure plan by item

### Item 12 — Suspension proof (suspension/approval/resume/at-most-once)

**Prerequisite:** W11-B1/B2/B3/B5/B7 and W10-B4 runtime work lands (engine suspension seam, waiter, DB v4, nonce payload producer). Until then these tests are BLOCKED (they must fail RED, not skip).

**A. Deterministic backend tests (no provider, no key):** new `backend/crates/hiverouter-core/tests/suspension.rs` (W11 §11.1 names) using a `MockExecutor` returning one side-effect tool call then a final answer, real `ApprovalGate` seeded rows (as `tests/ws_transport.rs:306-327` does), TTL/clock seam, temp DB. Required named tests:
`approval_suspends_run_and_approve_resumes_executes_once` · `deny_returns_denied_tool_result_and_run_continues` · `timeout_fails_run_e_approval_timeout_without_side_effect` · `cancel_while_awaiting_consumes_approval_and_cancels_run` · `kill_worker_while_awaiting_cancels_run_without_side_effect` · `concurrent_resolve_has_exactly_one_winner_and_no_second_execution` · `replayed_nonce_is_refused_before_any_side_effect` · `second_prompt_while_awaiting_is_refused_e_run_active` · `suspension_keeps_budget_and_increments_calls_counter` · `approval_request_event_carries_nonce_and_args_sha256` · `restart_marks_awaiting_run_interrupted_and_expires_pending`.

**B. Extend existing backend suites:** `tests/ws_transport.rs` add `approval_request_frame_is_published_when_run_suspends` (+ engine waiter wake, replay after terminal = `E_APPROVAL_REPLAY`); `tests/recovery_gates.rs` add awaiting→`interrupted` + expired approval; `tests/chat_store.rs` add `meta_json`/`turn` round-trip; `tests/snapshot_history.rs` migration v4 up/down `migration_v4_up_down_roundtrip`; `hiverouter-tools` lib `approval::tests` add `expire_pending_is_a_cas`, `cancelled_decision_is_not_resolvable`, `unique_pending_per_tool_call`; `hiverouter-contract` golden/matrix/roundtrip for extended `APPROVAL_REQUEST` + `TOOL_CALL`/`TOOL_RESULT` (event count 23; refusal count per B1 decision).

**C. Deterministic UI order test (mock WS, no key):** extend `e2e/specs/approval-flow.spec.ts` with `suspension event order is APPROVAL_RESOLVED -> TOOL_RESULT -> next REPLY_DELTA/RUN_STATUS` and `no TOOL_RESULT before APPROVAL_RESOLVED`; the existing tool-loop/approval specs already pin overlay/TTL/disabled-until-resolved/duplicate-row semantics.

**D. E2E live suspension (paid, operator-gated, one short prompt):** new `e2e/specs/live-tool-approval.spec.ts` using `startLiveServer`/`openWsProbe` (real binary, real WS) and new env gate `LIVE_TOOL_TURN=1` (+ `LIVE_ONE_PROMPT=1`, key present). Tests (names final):
`live approval: TOOL_CALL -> APPROVAL_REQUEST(nonce,args_sha256,expires_at) -> approve -> TOOL_RESULT ok -> RUN_STATUS completed` (assert exactly one side effect: temp workspace file written once; `tool_turns`/`approvals.executed_at` row; no duplicate `APPROVAL_RESOLVED`/`TOOL_RESULT`);
`live approval: deny -> TOOL_RESULT denied, no side effect, run continues/completes`;
`live approval: TTL expiry -> RUN_STATUS failed error_code E_APPROVAL_TIMEOUT, no side effect` (short test TTL via server env);
`live approval: cancel while awaiting -> RUN_STATUS cancelled, overlay gone, no side effect`;
`live approval: reload during wait -> interrupted/expired, never auto-approved`.
Capture each run to `docs/trace/live/captures/w12/<run-id>/approval-frames.jsonl` + DOM/PNG + `manifest.json` (UTC, command, sha256).

**Commands (backend; PATH as documented):**
```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$PWD\tools\w64devkit\bin;" + $env:PATH
cd backend
cargo test -p hiverouter-core --test suspension --locked
cargo test -p hiverouter-core --test ws_transport --locked
cargo test -p hiverouter-core --test recovery_gates --locked
cargo test -p hiverouter-core --test chat_store --locked
cargo test -p hiverouter-core --test snapshot_history --locked
cargo test -p hiverouter-tools --lib approval::tests --locked
cargo test -p hiverouter-contract --locked
```
**Commands (e2e; live sections need a running app and explicit operator key):**
```powershell
cd e2e
node run-guarded.mjs --project=chrome specs/tool-loop.spec.ts specs/approval-flow.spec.ts   # mock, no key
$env:E2E_APP_READY='1'; $env:E2E_BASE_URL='http://127.0.0.1:8799'
$env:E2E_SERVER_BIN='target/release/hiverouter.exe'
$env:LIVE_ONE_PROMPT='1'; $env:LIVE_TOOL_TURN='1'   # paid opt-in; key only in process env
node run-guarded.mjs --project=chrome specs/live-tool-approval.spec.ts
```
**Threshold:** all test names above PASS with 0 skips; approval resolves within TTL; exactly one side effect per approve; every refusal named; evidence JSON committed with sha256; no paid run without `LIVE_TOOL_TURN=1` + key (default = named skip, never fake pass).

---

### Item 13 — Live tool-turn proof

**A. Deterministic multi-turn loop test (no provider key):** new `backend/crates/hiverouter-core/tests/tool_loop_matrix.rs` using `wiremock` SSE: turn 1 returns a `tool_calls` fragment + finish, turn 2 returns final text; assert assembler → registry validation → read-only tool execution → `TOOL_CALL`/`TOOL_RESULT` ordering → final `REPLY_DONE`/`RUN_STATUS completed`; cap violation (turn > MAX_TOOL_TURNS / calls > MAX_CALLS_PER_RUN) yields the named cap refusal; malformed/unknown tool/argument-schema refusal yields `TOOL_RESULT refused` + named code, never execution. This closes W10-B7 without a paid call.

**B. Live paid tool turn (operator only):** extend `backend/crates/hiverouter-core/tests/provider_live.rs` with `#[ignore = "live tool turn; LIVE_TOOL_TURN=1 + DEEPSEEK_API_KEY; one short prompt"]` `live_tool_turn_is_assembled_executed_and_reported` (model is asked to read one file; real model must emit a tool call; assert `tool_calls` assembled, tool executed, final reply completed; usage truthful). Env gate must be **distinct** from `LIVE_ONE_PROMPT` so a plain paid streaming run never triggers a loop. Budget cap `$0.50`, one prompt, no retries beyond documented policy; key never logged.
**C. Live UI proof:** `e2e/specs/live-tool-turn.spec.ts` (same env gate): `SUBMIT_PROMPT` with tool-enabled request → `TOOL_CALL` panel row pending → `TOOL_RESULT` settled → final `REPLY_DONE`/completed; assert ordered frames and no duplicate tool rows.
**Commands:**
```powershell
cd backend
cargo test -p hiverouter-core --test tool_loop_matrix --locked
cargo test -p hiverouter-core --test provider_live --locked                     # ignored test compiles/skips by name
cargo test -p hiverouter-core --test provider_live --locked -- --ignored --exact live_tool_turn_is_assembled_executed_and_reported
cd ..\e2e
$env:E2E_APP_READY='1'; $env:E2E_BASE_URL='http://127.0.0.1:8799'
$env:E2E_SERVER_BIN='target/release/hiverouter.exe'
$env:LIVE_ONE_PROMPT='1'; $env:LIVE_TOOL_TURN='1'
node run-guarded.mjs --project=chrome specs/live-tool-turn.spec.ts
```
**Threshold:** deterministic matrix green before any live claim; live test recorded run id/usage/frames in `docs/evidence/1.2-live-tool-<runid>.json` (redacted); otherwise the live tool turn remains READ/UNVERIFIED. **No claim of autonomous multi-turn tooling** until both A and B/C evidence exist.

---

### Item 18 — Trace promotion procedure & evidence thresholds

**Invariant:** no hop may leave PENDING/MISSING/UNVERIFIED unless a **committed raw artifact** under `docs/trace/live/captures/<wave>/` supports that specific hop (CAPTURE-REGISTRY rules 1–4). Prose, source pointers, test-summary lines and `target/` logs do not qualify. Current raw capture is empty, so the current truth (1/264/5/49 of 319) must not be improved by any document edit.

**Capture requirements per registry row** (all must be tracked with UTC timestamps + producing command):
1. `CAP-S10-REPLAY`: WS frame JSONL with `HISTORY_REVISION_SET` → `ACK` + `HISTORY_SNAPSHOT`; post-replay DOM snapshot/banner + revision; PNG; test-result JSON; assert 0 draggable/selectable nodes.
2. `CAP-S15-19-CARDS`: `RUN_CREATED`/`NODE_SPAWN`/`NODE_STATE` frames; AgentsPanel + canvas card DOM; one card per kind; enum-derived labels; PNG; run id.
3. `CAP-S27-STREAMING`: ordered `REPLY_DELTA` frames (contiguous seq, UTC ts) → `REPLY_DONE` usage; rendered reply DOM/PNG; run/usage row snapshot; provider metadata without key; no fake completion.
4. `CAP-SAPPROVAL`: `APPROVAL_REQUEST` with nonce/args_sha256/expires_at; overlay DOM/PNG; `APPROVAL_RESOLVE` command + `ACK`/`APPROVAL_RESOLVED` (or named ERROR) frames; audit ledger row; no silent effect; approve/deny only.
5. `CAP-S22-RECHECK`: fresh `FILE_TREE_LIST` roundtrip at current HEAD + Data panel DOM/PNG; replaces pre-HEAD probe pointer.

**Promotion procedure (exact order, one commit):**
```powershell
# 0. capture is produced by e2e/QA lane; this is the trace-seat validation step
node tools/qa/verify-harness.mjs                       # must stay 27/27 (seeds still valid)
# 1. validate capture content before merge: surfaces>0; every promoted hop has evidence kind+quote; artifact sha256 manifest matches files
# 2. merge capture -> controls (must not run on an empty/ambiguous capture)
node tools/qa/apply-live-trace.mjs
# 3. recompute counts exactly as documented
node -e "const d=require('./docs/trace/controls.json');const c={};for(const x of d.controls)for(const h of Object.values(x.hops))c[h.verdict]=(c[h.verdict]||0)+1;console.log(c);"
# 4. re-verify
node tools/qa/verify-harness.mjs
# 5. commit capture + manifest + controls + capture-registry updates together; no separate "prose" commit
```
**Thresholds:** `surfaces > 0`; `raw_capture_backed_wired == promoted WIRED hops`; `_meta.counts` recomputed from a single command and printed; `verify-harness.mjs` PASS; every evidence pointer resolves to a committed file with sha256 in the capture manifest. If **all** registered entries are captured and accepted, expected totals become **24 WIRED / 253 PENDING / 0 MISSING / 42 UNVERIFIED of 319** (1+23 promoted; 11 PENDING, 5 MISSING, 7 UNVERIFIED converted; `raw_capture_backed_wired=24`); any subset is reported by the recompute, never hand-edited.
**Required tool hardening (separate gate change):** `apply-live-trace.mjs` must fail when `surfaces.length === 0`; `verify-harness.mjs` must resolve `evidence[].pointer` files and (where listed) sha256; capture writers must refuse to overwrite a non-empty capture with an empty one. Until landed, an empty `trace-capture.json` is a blocker, and `docs/trace/CAPTURE-REGISTRY.md` row status must remain the only authority.

---

### Item 19 — G-UI closure (G-UI-3/10, W1/W2)

Every G-UI id must be **implemented or explicitly excluded**; the registry test enforces this (`e2e/specs/gui-gates.spec.ts:335-355`). Closure requires deleting the exclusion row **only after** the live evidence is committed.

**G-UI-3 / W1 (foreign Origin):** new `e2e/specs/live-security.spec.ts` using a real server (`startLiveServer({withProviderKey:false})`):
- serve a tiny page on a **non-allowlisted origin** and open `ws://127.0.0.1:<port>/ws`; assert close code **1008**, typed `E_ORIGIN_REJECTED` captured from server/WS evidence, no state mutation (pre/post `/health`/`/api/session`; zero new DB rows);
- also assert bad token → 401 and allowlisted page connects; capture JSON + screenshot to `docs/trace/live/captures/w12/security/`.
- command: `node run-guarded.mjs --project=chrome specs/live-security.spec.ts` (app ready env), then delete G-UI-3/W1 rows and update the acceptance note with artifact paths.

**G-UI-10 / W2 (resync):** new `e2e/specs/live-resync.spec.ts`:
- kill the WS mid-run (close socket); reconnect; assert `SERVER_HELLO` with `last_seq`; run state/snapshot adopted; **no duplicated reply text** (concatenated deltas before/after reconnect);
- stale mutation: send `REWIRE_TASK`/`HISTORY_REVISION_SET` with stale revision/seq → typed `E_RESYNC_REQUIRED`; no UI mutation before refusal.
- command: `node run-guarded.mjs --project=chrome specs/live-resync.spec.ts`; capture frames/DOM to `docs/trace/live/captures/w12/resync/`; then delete G-UI-10/W2 rows.
**Threshold:** real-server test PASS (not mock), evidence committed with sha256; registry test green after row deletion; `GATE-EXCLUSIONS.md` must state the closing evidence path and date.

---

### Item 20 — Axe subcheck closure

G-UI-8 focus-ring/contrast is implemented (`gui-gates.spec.ts:226-264`); only the axe serious-violation subcheck is excluded. To close:
1. Lead/tooling owner approves a **pinned** axe dependency and records exact version + integrity from official npm (the plan must not invent a version; current exclusion documents `MODULE_NOT_FOUND` and no install).
2. `cd e2e; npm.cmd install --save-dev --save-exact @axe-core/playwright@<Lead-approved-version>`; commit `package.json` + `package-lock.json` (e2e lock).
3. Add to G-UI-8 (or new `axe` test in `gui-gates.spec.ts`): load `web/dist`, run axe with tags `wcag2a/wcag2aa`; assert `violations.filter(v => v.impact === 'serious' || v.impact === 'critical')` is empty; record `docs/trace/live/a11y-axe.json` (rule ids, impact, target selectors) + command/exit.
4. Remove the G-UI-8 axe row from `GATE-EXCLUSIONS.md`; rerun registry test.
**Command:** `npx playwright test specs/gui-gates.spec.ts --project=chrome -g "G-UI-8"` (inside e2e), then `node run-guarded.mjs --project=chrome specs/gui-gates.spec.ts specs/a11y-gates.spec.ts`.
**Threshold:** axe serious+critical = 0 on `web/dist` with the pinned dependency; focus ring test still PASS; evidence JSON committed. If dependency approval is refused, the row **stays** and no "axe clean" claim is permitted.

---

### Item 21 — Strict gate closure (pre-release → sealed)

Current gate semantics are already implemented and must be exercised, not trusted: G4 skip is never PASS; `-RequireE2E` turns a skip into FAIL; `-Release` requires zero skips and blocks (`exit 1`) unless a documented pre-release tree is accepted (`GATES: PRE-RELEASE`, exit **3**, never release PASS); sealed trees require `manifest.source_commit == HEAD`.

**Required sequence:**
```powershell
# 1. plan only
powershell -NoProfile -ExecutionPolicy Bypass -File tools/qa/run-gates.ps1 -DryRun
# 2. mid-wave rehearsal (dirty tree allowed; e2e must be real, not skipped)
$env:E2E_APP_READY='1'; $env:E2E_BASE_URL='http://127.0.0.1:8799'
powershell -NoProfile -ExecutionPolicy Bypass -File tools/qa/run-gates.ps1 -Rehearsal -RequireE2E
# 3. documented pre-release acceptance (1.2.0-dev tree; manifest still 1.1.0) -> expect GATES: PRE-RELEASE, exit 3
powershell -NoProfile -ExecutionPolicy Bypass -File tools/qa/run-gates.ps1 -Release -RequireE2E
# 4. paid live proof only after 1-3: run-gates -Live -RequireE2E (app running, key present, no skips)
#    + LIVE_TOOL_TURN=1 for the live tool-turn specs
```
**Acceptance:** no `SKIPPED` step in a `-Release`/`-RequireE2E` run; `GATES: PRE-RELEASE` (exit 3) only on the pre-release tree is expected; a **sealed** tree must print `GATES: PASS` and exit 0. Gate outputs and HEAD start/end must be quoted in the receipt. `E2E_ALLOW_ALL_SKIPS` must never be set.
**Known blockers to resolve before the final run:** dirty tree (currently 9 paths) → commit; pre-release acceptance requires clean tree; full e2e needs a running app; paid steps need operator key/budget.

---

### Item 22 — HiveMind 1.2 seal / release execution

**Topology decision required first (item 21 dependency):** because `build-release.ps1` requires the tag at HEAD and records `source_commit=HEAD`, but seal artifacts/manifest/evidence are committed after the build, a green strict `-Release` requires an explicit **source commit C + seal commit D** model:
- Add `seal_commit` to `tools/release-manifest.json` (written by `build-release.ps1` when run in seal mode, or by a post-build seal step).
- Add gate release-mode acceptance: sealed if `HEAD == manifest.seal_commit` and `git diff --name-only source_commit..HEAD` ⊆ seal allowlist (`dist/**`, `tools/release-manifest.json`, `docs/RELEASE-EVIDENCE.md`, `docs/RELEASE.md`, `README.md`, `docs/claims/**`, `plans/waves/README.md`, `docs/trace/**` only when evidence is part of the seal); any code/test/config file in that diff ⇒ FAIL. If `seal_commit` absent, keep today's strict `source_commit == HEAD`.
- Alternative (not recommended): retag at the seal commit, which moves the tag and loses the source/seal distinction; if used it requires owner confirmation and recorded supersession.

**Seal sequence (owner commands; no paid calls except step 6):**
1. **Freeze source commit C:** all 1.2 waves merged; `git status --porcelain` empty; versions set to **`1.2.0`** (from `1.2.0-dev`) in 4 crates + `Cargo.lock` + web package/lock; docs updated for source/seal roles; commit C.
2. **Pre-seal verification at C:** `node tools/qa/check-docs-truth.mjs`; `node tools/qa/verify-harness.mjs`; `cargo test --workspace --locked`; web typecheck/test/build; mock e2e (tool-loop/approval/replay/gui); trace/claims updates committed.
3. **Annotated tag at C (owner action):** `git tag -a hivemind-1.2.0 -m "HiveMind 1.2.0 source: suspended tool loop, trace promotion, G-UI/axe closure" C` — do **not** move `v1.0.0`/`hivemind-1.1.0`.
4. **Build from tag:** `powershell -NoProfile -ExecutionPolicy Bypass -File tools/build-release.ps1 -Version 1.2.0 -Tag hivemind-1.2.0` (HEAD must equal C). Writes `dist/HiveMind-1.2.0-win64.zip` + sidecar + staging CHECKSUMS + `tools/release-manifest.json` (source C). Run twice to prove determinism; compare artifact/zip sha256 (must be identical).
5. **Seal commit D:** update `docs/RELEASE-EVIDENCE.md` (artifact hashes, source C/seal D, test/live/trace/axe evidence), `docs/RELEASE.md`/`README.md` version+limits; update claims register; record `seal_commit=D` in manifest; commit D (`release: seal HiveMind 1.2.0 artifacts, manifest, evidence`).
6. **Final gate at D:** `powershell -NoProfile -ExecutionPolicy Bypass -File tools/qa/run-gates.ps1 -Release -RequireE2E` with app running (or `-Live` run separately with operator key) → expected `GATES: PASS`, exit **0**, zero skips, G7 artifact/zip/sidecar/CHECKSUMS/web-dist hashes verified against the manifest; `node tools/qa/check-docs-truth.mjs` PASS. Re-run after any evidence-only commit and expect a fresh seal update (evidence commits after D require a new seal commit).
7. **Publish:** push commit C/D and `hivemind-1.2.0` tag only after the PASS receipt; keep 1.1 record immutable; release notes point at `tools/release-manifest.json`.

**Acceptance wording (do not weaken):** "1.2.0 is released only when the tag points at source commit C, the seal commit D is a documentation/artifact-only descendant of C, `tools/release-manifest.json` records both, `run-gates -Release -RequireE2E` exits 0 with zero skips on D, and live tool-turn + suspension + trace-promotion + axe evidence are committed and linked." Any missing item ⇒ **blocked**, never "PASS with notes".

---

### Item 23 — Blockers / risks

| ID | Blocker/risk | Evidence | Resolution / owner |
|---|---|---|---|
| FB1 | Runtime tool loop + suspension not **committed/verified**: in-flight uncommitted edits exist, no build/test/live evidence (BLOCKER for items 12/13) | `provider_executor.rs:171-173` pre-edit behavior; `git status` (8 product paths modified); W10 B1–B8; W11 blockers W11-B1…B8 | W12 S1/S2/S3 runtime waves; tests first (RED), then GREEN |
| FB2 | No 1.2 artifacts/tag; source is `1.2.0-dev` (BLOCKER for item 22) | manifest 1.1.0; `git tag -l`; version files | Version freeze + seal sequence above |
| FB3 | Seal topology cannot be strict-green under current gate (BLOCKER) | `build-release.ps1:239-245,545`; 1.1 tag/source vs later evidence commits (`57d5157`) | Two-commit model + `seal_commit` gate/manifest change |
| FB4 | Trace raw capture empty; all registry captures absent (BLOCKER for item 18) | `controls.json` `_meta.live_capture` (surfaces 0); registry statuses | E2E/QA capture lane + apply/verify hardening |
| FB5 | G-UI-3/10/W1/W2 need live-server security/resync tests (BLOCKER for item 19) | GATE-EXCLUSIONS rows; only mocked WS today | New live specs + committed evidence |
| FB6 | Axe dependency not approved/pinned (BLOCKER for item 20) | GATE-EXCLUSIONS: `MODULE_NOT_FOUND`, no install | Lead approval + exact version/integrity; else row stays |
| FB7 | Live paid steps need operator key + budget; no key in harness environment | `live-server.ts` paidGate; `capture-live-1.1.json` gate false | Operator run: `LIVE_ONE_PROMPT=1`/`LIVE_TOOL_TURN=1`, $0.50 cap, one short prompt |
| FB8 | Dirty tree (8 in-flight product paths + this plan at final read) + no `-SealOutputs` allowlist | `git status --porcelain`; run-gates strict clean-tree gate | Commit waves; implement seal-commit allowlist |
| FB9 | Trace promotion tooling can silently accept empty capture; `verify-harness` doesn't resolve evidence pointers | `apply-live-trace.mjs:18-58`; W4 §5 | Harden tools before any promotion; tests for empty capture/pointer existence |
| FB10 | Paid live evidence in repo is READ-level, not independently reproduced by this plan | `docs/evidence/live-1.1-paid.log`; captures w11 | Treat as READ until operator re-runs on 1.2; never upgrade label |

### Item 24 — What must NOT be claimed

- No **1.2 release**, tag, artifacts, manifest or "released" wording before item-22 steps complete with `GATES: PASS` exit 0.
- No **runtime tool loop / auto tool-calling / suspension / resume / at-most-once** claim before backend suspension + loop tests pass and live proof is captured (currently only contract/assembler/mocked UI exists).
- No **trace WIRED** promotion from prose, source pointers, unit-test summaries or `target/` logs; no exact promoted counts except the recomputed command output; current truth remains 1/264/5/49 until captures are committed.
- No **G-UI-3/10/W1/W2 closed** claim from mock WS tests; no **axe clean** claim without the pinned dependency and committed axe result.
- No **strict gate green** claim while any gate is SKIPPED, the tree is dirty/pre-release, or `manifest.source_commit != HEAD`/`seal_commit` rules are unmet; `PRE-RELEASE` (exit 3) is not a release pass.
- No claim that 1.1 live paid evidence proves the 1.2 tool loop; no "no skips" claim until guarded e2e executes; no performance/P95 claim (G-UI-12 remains unmeasured).
- No claim that a re-run of the release script moved a tag (it never does); tag moves require owner confirmation + recorded supersession.

### Item 25 — Naming and evidence table

**Naming conventions (proposed, freeze before coding):**
- Backend tests: `backend/crates/hiverouter-core/tests/suspension.rs`, `.../tests/tool_loop_matrix.rs`; names in `snake_case` verb-first exactly as listed in items 12/13.
- E2E tests: `e2e/specs/live-tool-approval.spec.ts`, `live-tool-turn.spec.ts`, `live-security.spec.ts`, `live-resync.spec.ts`; test titles `"live <area>: <observable> (<gate>)"`; env gates `LIVE_TOOL_TURN=1` (loop/suspension), `LIVE_ONE_PROMPT=1` (single streaming/replay prompt); skip reasons must start with `LIVE_`/`E2E_` prefix.
- Evidence: logs `docs/evidence/1.2-<area>-<HEAD8>.log`; JSON captures `docs/evidence/1.2-<area>-<runid>.json`; trace captures `docs/trace/live/captures/w12/<registry-id>/` with `frames.jsonl`, `dom.json`, `shot.png`, `manifest.json` (UTC, command, sha256 per file); release evidence `docs/RELEASE-EVIDENCE.md`; seal manifest `tools/release-manifest.json`.
- Gate receipts: commit message prefix `release:`/`test:`/`trace:`; every receipt quotes command + exit code + HEAD start/end.

**Evidence table (deliverable → artifact → exact command → threshold → current status):**

| Item | Artifact(s) | Command | Threshold | Current status |
|---|---|---|---|---|
| 12 suspension | `backend/.../tests/suspension.rs` results in `docs/evidence/1.2-suspension-<HEAD8>.log`; live `docs/evidence/1.2-live-approval-<runid>.json`; capture dir `docs/trace/live/captures/w12/CAP-SAPPROVAL/` | `cargo test -p hiverouter-core --test suspension --locked`; e2e command item 12 | 0 failed/skipped; one side effect per approve; CAS single winner; TTL/cancel/deny named codes | **BLOCKED** (runtime seam absent) |
| 13 live tool turn | `docs/evidence/1.2-live-tool-<runid>.json` (frames/usage/run id, redacted); `tool_loop_matrix` log | `cargo test -p hiverouter-core --test tool_loop_matrix --locked`; `cargo test -p hiverouter-core --test provider_live --locked -- --ignored --exact live_tool_turn_is_assembled_executed_and_reported`; e2e live-tool-turn | deterministic loop green; live run completed with tool result; no key material | **BLOCKED** (loop absent; 1.1 live streaming READ only) |
| 18 trace promotion | `docs/trace/live/captures/w12/**` + `capture-registry.json` + updated `controls.json` counts | `node tools/qa/verify-harness.mjs`; `node tools/qa/apply-live-trace.mjs`; recompute `node -e "…"`; `node tools/qa/verify-harness.mjs` | surfaces>0; per-hop raw evidence; counts recomputed; `raw_capture_backed_wired == promoted`; expected full-capture totals 24/253/0/42 | **BLOCKED** (capture empty: 1/264/5/49; surfaces 0) |
| 19 G-UI closure | `docs/trace/live/captures/w12/security/`, `…/resync/`; updated GATE-EXCLUSIONS | `node run-guarded.mjs --project=chrome specs/live-security.spec.ts specs/live-resync.spec.ts` | real-server PASS; 1008+`E_ORIGIN_REJECTED`; resync `SERVER_HELLO{last_seq}` + no duplicate text; `E_RESYNC_REQUIRED` | **BLOCKED** (live specs absent; mock only) |
| 20 axe | `e2e/package.json`/lock with pinned dep; `docs/trace/live/a11y-axe.json` | `npx playwright test specs/gui-gates.spec.ts --project=chrome -g "G-UI-8"` | serious+critical violations = 0 on `web/dist`; focus test green | **BLOCKED** (no approved dep) |
| 21 strict gate | `docs/evidence/1.2-gates-<HEAD8>.log` (stdout) | `-DryRun`; `-Rehearsal -RequireE2E`; `-Release -RequireE2E`; `-Live` (paid) | exit 0 sealed; exit 3 only pre-release; zero skips; HEAD start=end | **PARTIAL** (harness exists; dirty/pre-release; no seal topology) |
| 22 seal/release | `tools/release-manifest.json` (source C + seal D), `dist/HiveMind-1.2.0-win64.zip`+sidecar/CHECKSUMS, `docs/RELEASE-EVIDENCE.md`, tag `hivemind-1.2.0` | tag; `build-release.ps1 -Version 1.2.0 -Tag hivemind-1.2.0` (twice); `run-gates -Release -RequireE2E`; `node tools/qa/check-docs-truth.mjs` | deterministic hashes; gate exit 0; docs checker PASS; old tags unchanged | **BLOCKED** (1.2.0-dev/no artifacts/no seal model) |
| 23 blockers | this section/items | reviewed per wave | each closed or explicitly blocked | ACTIVE |
| 24 non-claims | release notes/docs review | grep for prohibited claims before seal | no "released/works/proven" without evidence path | ACTIVE |
| 25 naming/evidence | naming table above + commit receipts | review checklist | every claim cites artifact+command+threshold | READY |

**Critical path:** W12 runtime loop/suspension → deterministic tests (items 12/13) → live capture lane (trace/G-UI/axe/live proofs) → seal topology change (item 22 gate/manifest) → version freeze 1.2.0 → seal + strict gate → release receipt. Trace promotion and G-UI/axe can proceed in parallel once runtime is green; no item may be reported DONE before its evidence row is committed and quoted.

---

**Plan status:** planning only; no builds/tests/paid calls executed for this document. The plan deliberately keeps 1.1 as the authoritative sealed record and makes no 1.2/runtime/trace-promotion claim.
