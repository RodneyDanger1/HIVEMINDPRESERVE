# CODE-PROOF — exact proof plan for the next implementation wave

**Planner:** CODE RESEARCH PLANNER 3 · **Mode:** read-only planning; **this file is the only write** — no product code, tests, configs, gates, git, keys, paid calls or builds were touched.
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **Observed:** HEAD `37f0e7c` (`hivemind-1.1.0-4-g37f0e7c`), 8 dirty product paths (runtime loop/suspension lane: `core/src/{execution,http,provider_executor,recovery,tools_bridge}.rs`, `core/tests/{run_lifecycle,tool_loop_engine}.rs`, `tools/src/approval.rs`), tree declares **`1.2.0-dev`**, sealed record remains **1.1.0** (`tools/release-manifest.json`: `version=1.1.0`, `source_commit=tag_target=87079ce`, no `seal_commit` field).
**Skills applied:** `/forgewright-recon` → `/forgewright-lead` → `/logic-gatekeeper` → `/integration-logic` → `/logic-debugger` (Phase-1 gating in `plans/foresight/PROOF-RELEASE-PLAN.md` §1 still holds; this is planning only, no Phase-2 mutation).

**Inputs read:** `plans/foresight/PROOF-RELEASE-PLAN.md` (251 lines), `e2e/specs/**` (12 specs), `e2e/helpers/**`, `backend/crates/hiverouter-core/tests/**` (17 files incl. `tool_loop_engine.rs` 23,625 B and `approval_ledger.rs`), `docs/trace/**` (controls.json 115,149 B; capture-registry.json; CAPTURE-REGISTRY.md; trace-capture.json 286 B/empty; w11 captures), `tools/qa/run-gates.ps1` (36,275 B), `tools/build-release.ps1` (31,922 B).

---

## 0. Verified proof-state (what exists and what is claimed)

| Claim in the repo | Current evidence | Status |
|---|---|---|
| "No runtime claim yet: tool loop / approval suspension / tool-activity UI not implemented or run" | `plans/waves/README.md:35,47`; uncommitted engine edits only; `tool_loop_engine.rs` (9 mocked-executor tests) exists but **no suspension suite** | **Claim is accurate; must be falsified by proof, not asserted** |
| 1.1.0 sealed; 1.2.0-dev pre-release; `run-gates -Release` must return `GATES: PRE-RELEASE` exit 3 | `RELEASE-EVIDENCE.md:3-21`; `run-gates.ps1:23-33,248-316`; manifest 1.1.0 | Documented, **not yet gate-proven on the current tree** |
| Trace truth: `1 WIRED / 264 PENDING / 5 MISSING / 49 UNVERIFIED` of 319; `raw_capture_backed_wired=1`; raw capture EMPTY (`surfaces: []`, 286 B, `2026-11-06T20:25:04Z`) | `docs/trace/controls.json` `_meta`; `docs/trace/live/trace-capture.json` | Honest but **promotion lane untested** |
| 5 capture-registry entries: `CAP-S10-REPLAY` NOT_CAPTURED, `CAP-S15-19-CARDS` MISSING_NO_CAPTURE, `CAP-S27-STREAMING` UNVERIFIED_NO_CAPTURE, `CAP-SAPPROVAL` PENDING_NO_CAPTURE, `CAP-S22-RECHECK` PARTIAL_RAW_PRE_HEAD | `docs/trace/capture-registry.json` | Capture plan only |
| Live paid proof exists for 1.1 (5 passed/EXIT=0) but **no 1.2 live tool-turn proof** | `docs/evidence/live-1.1-paid.log`; no live tool spec | 1.2 proof gap |
| G-UI-3/W1 + G-UI-10/W2 + G-UI-8 axe subcheck are **named exclusions**, not passes | `e2e/GATE-EXCLUSIONS.md`; `gui-gates.spec.ts:226,335-361` | Exclusion truth; closure unproven |
| Gate harness strict modes exist: `-DryRun/-Rehearsal/-SkipE2E/-RequireE2E/-Release/-Live`; exit `0/1/2/3`; `-Release` requires zero skips and documented pre-release markers | `run-gates.ps1:23-45,248-316`; `RELEASE-EVIDENCE.md` + `plans/waves/README.md` both mark pre-release | Implemented, **not executed on this tree** |
| Build at HEAD ahead of tag is a rehearsal only (`-AllowTagAncestor`); no seal topology support | `build-release.ps1:228-250,543-546`; manifest has no `seal_commit`/sealed-topology fields | **1.2 seal blocker** |
| Trace tooling cannot refuse bad promotion inputs | `apply-live-trace.mjs` only checks file existence; `verify-harness.mjs:108` checks verdict support, not evidence-pointer resolution | **Hardening gap** |

---

## 1. PROOF SUITE A — deterministic suspension tests (RED-first; mocked executor, no paid path)

**New file:** `backend/crates/hiverouter-core/tests/suspension.rs` (or extend `run_lifecycle.rs` if the lane prefers; the names below are the contract).
**Fixture:** mirror `run_lifecycle.rs` `test_engine`/`TestEngine`: `tempfile::tempdir()` → `Database::open(...).migrate()` → `Arc<EventBus>` → `RunEngine::new(db, bus, GraphAuthority, Arc<dyn ProviderExecutor>, TemplateRegistry::default())`; compose a `MockExecutor` whose first `execute` returns `StreamedReply { deltas, usage, tool_calls: [AssembledToolCall{file_read...}] }` and whose follow-up returns final text (`run_lifecycle.rs:23-127` patterns; `tool_loop_engine.rs:35-99,199-292` `Harness`/`drive` patterns). Seed a pending approval with the existing helper pattern from `ws_transport.rs:306-327` (`ApprovalGate::request_tool_approval` via `ToolsBridge`, returning `(approval_id, nonce)`), then resolve it inside the test through the engine/command path. **No network, no key, no paid call.**

| # | Test name (`#[tokio::test] async fn`) | Fixture / setup | Command | Expected output | Falsifies claim |
|---|---|---|---|---|---|
| A1 | `suspension_approve_resumes_once_and_completes` | tool call needing approval → seed approval → `approve` | `cd backend; cargo test -p hiverouter-core --test suspension suspension_approve_resumes_once_and_completes --locked -- --exact` | `test result: ok. 1 passed; 0 failed; 0 ignored` | "suspension/resume works" |
| A2 | `approve_then_duplicate_resolve_cannot_execute_twice` | approve twice with same/next nonce | same file, `-g` all `suspension_` … | `1 passed; 0 failed` + executor call-count assertion = 1 | "exactly one side effect" |
| A3 | `deny_resumes_without_tool_execution` | seed approval → deny | same | `1 passed` + zero tool executions | "deny never executes" |
| A4 | `boot_reaper_expires_pending_approval_to_named_timeout` | pending row; restart/boot reaper (`http.rs:111-126`) | same | `1 passed` + run status `failed`/`E_APPROVAL_TIMEOUT` | "expired approvals never execute" |
| A5 | `kill_while_suspended_never_executes_the_tool` | approval pending → `CANCEL_RUN`/kill | same | `1 passed` + status `cancelled`/`killed`, no `TOOL_RESULT` | "cancel during wait is safe" |
| A6 | `resume_after_restart_surfaces_exactly_one_terminal_event` | approve, restart engine, drive to terminal | same | `1 passed` + exactly 1 `RUN_STATUS` terminal | "replay/restart single-effect" |
| A7 | `wrong_approval_hash_is_refused_before_execution` | seed approval then mutate `args_sha256` binding | same | `1 passed` + `E_APPROVAL_DECISION_INVALID` refusal, no execution | "args binding enforced" |
| A8 | `wrong_approval_session_is_refused_by_name` | request nonce from another session | same | `1 passed` + `E_APPROVAL_NOT_FOUND`/`E_APPROVAL_DECISION_INVALID` | "approval authority scoped" |
| A9 | `resume_appends_no_duplicate_reply_deltas` | follow-up streaming deltas after resume | same | `1 passed` + deltas concatenated once | "no duplicated stream after resume" |
| A10 | `resume_failure_sets_named_error_and_releases_the_waiter` | follow-up executor returns error | same | `1 passed` + named error code, waiter released (no hang; test timeout 10 s) | "no stuck waiter" |
| A11 | `run_terminal_drops_pending_waiter_and_approval_row` | run cancel/terminal while approval row exists | same | `1 passed` + table empty/terminal, no late execution | "no orphan approvals" |
| A12 | `approval_ledger_writes_one_row_on_resume_and_none_on_replay` | extension of `approval_ledger.rs` fixture (`approval_ledger.rs:1-130`) | `cargo test -p hiverouter-core --test approval_ledger --locked` | `test result: ok. 3 passed; 0 failed` (2 existing + 1 new) | "ledger exactly-once" |
| A13 | `provider_live_tool_turn` (`#[ignore]`) — future live gate, see B1 | `provider_live.rs` pattern | `cargo test -p hiverouter-core --test provider_live --locked -- --ignored` | **PAID** (see §7) | "live provider tool turn works" |

**Existing suites to quote as regression evidence (no new code):** `run_lifecycle` (7 tests), `tool_loop_engine` (9 tests: `read_only_tool_call_executes_publishes_and_persists`, `plain_reply_keeps_the_pre_tool_loop_behaviour`, `non_read_tool_is_refused_by_name_and_never_executes`, `read_call_without_a_jail_is_refused_unavailable`, `per_run_call_budget_refuses_calls_beyond_the_cap`, `follow_up_turn_finishes_after_read_only_result`, `second_tool_turn_executes_then_finishes`, `persisted_turn_cap_fails_honestly_instead_of_fake_completion`, `streaming_follow_up_does_not_duplicate_deltas`, `exhausted_call_budget_fails_with_call_limit`, `continuation_refusal_fails_the_task_by_name`), `recovery_gates` (3), `approval_ledger` (2), `ws_transport` approval seeding. Command: `cargo test --workspace --locked` → `test result: ok. N passed; 0 failed; 2 ignored` (baseline 286+; record exact).

**Mock e2e ordering extension — `e2e/specs/approval-flow.spec.ts`** (existing mocked-WS fixtures; no paid):
| Test name | Fixture | Expected | Falsifies |
|---|---|---|---|
| `'suspension order: APPROVAL_RESOLVED -> TOOL_RESULT -> REPLY_DELTA -> RUN_STATUS completed'` | `installMockWs` + `sendApprovalRequest` + `eventFrame` | row order in `ul[aria-label="Tool calls"]`, one terminal row, final status rendered | "no ordering regression" |
| `'no TOOL_RESULT may precede APPROVAL_RESOLVED for an approval-gated call'` | same, emit `TOOL_RESULT` first | no `li[data-outcome]` before resolution; buffered replay after resolution | "gated call cannot execute before approval" |

---

## 2. PROOF SUITE B — live tool-turn spec and gating (PAID-GATED)

**New helper gate:** extend `e2e/helpers/live-server.ts` `paidGate()` (115-137) with a second, independent gate `LIVE_TOOL_TURN=1` (keep `LIVE_ONE_PROMPT` for streaming/replay); extend `startLiveServer` options (312-425) with `allowedOrigins?: string[]` (for §4) and a `toolTurn` mode marker. Skip reason must name the exact missing env (`LIVE_TOOL_TURN is not 1`) — never a silent pass.

**New file:** `e2e/specs/live-tool-turn.spec.ts` · `describe('live tool turn (paid gate)')`
**Fixture:** `startLiveServer({ withProviderKey: true, env: { LIVE_TOOL_TURN: '1' } })`, `openWsProbe(page, live.wsUrl())` (`live-server.ts:443-459`), `captureFrames(page)` + `writeEvidence('live-tool-turn.json', …)` + `shot(page,'live-tool-turn')`.
**Selectors:** nav `nav[aria-label="Primary navigation"]` → `button[name="Agents"]`; `[data-testid="agents-tool-activity"]`; `ul[aria-label="Tool calls"] li[data-outcome]`; `#composer`; HUD `role=status`; refusal toast `role=alert`.

| # | Test name | Steps / expected output | Falsifies claim |
|---|---|---|---|
| B1 | `'tool turn: TOOL_CALL -> TOOL_RESULT ok -> RUN_STATUS completed (paid)'` | submit prompt that forces `file_read`; expect `li[data-outcome="pending"]` → `li[data-outcome="ok"]`, exactly one terminal row; WS probe shows `TOOL_CALL` before `TOOL_RESULT` before `RUN_STATUS completed`; evidence JSON written. Playwright: `X passed`; guarded runner exit 0. | "runtime tool loop works live" |
| B2 | `'tool turn: denied approval yields TOOL_RESULT denied, no side effect (paid)'` | prompt for `file_write` → approval card → click Deny → `li[data-outcome="denied"]`, workspace file absent, run terminal | "denied side effect never executes" |
| B3 | `'tool turn: approval approve executes exactly once (paid)'` | approve → `ok` row, file content exactly once, one `TOOL_RESULT` frame, run terminal | "at most one side effect" |
| B4 | `'tool turn: assistant text after tool result is not duplicated (paid)'` | final `REPLY_DELTA` sequence concatenated once; `#composer` enabled after `REPLY_DONE` | "no duplicated stream" |
| B5 | `'tool turn: gate missing => named skip, never fake pass'` | run without `LIVE_TOOL_TURN=1` | test is reported `skipped` with reason containing `LIVE_TOOL_TURN`; `run-guarded` exit 1 unless `E2E_ALLOW_ALL_SKIPS=1` (skip-as-green guard) | "green without live run" |
| B6 | `'tool turn: malformed tool args are refused by name (paid)'` | provider mock returns invalid args → run fails `E_TOOL_SCHEMA`; no `ok` row | "no unsupported tool executes" |

**Deterministic matrix (no paid) — new `backend/crates/hiverouter-core/tests/tool_loop_matrix.rs`** (wiremock SSE; pattern `provider_streaming.rs`):
`tool_loop_matrix_read_only_call_roundtrip`, `tool_loop_matrix_turn_cap_refusal`, `tool_loop_matrix_call_cap_refusal`, `tool_loop_matrix_invalid_arguments_refused`, `tool_loop_matrix_non_read_tool_refused_by_name` → `cargo test -p hiverouter-core --test tool_loop_matrix --locked` → `test result: ok. 5 passed; 0 failed`. Falsifies "loop mechanics unproven off-network".

**Commands (acceptance):**
```
cd backend && cargo test -p hiverouter-core --test suspension --locked
cd backend && cargo test -p hiverouter-core --test tool_loop_matrix --locked
cd e2e && $env:LIVE_TOOL_TURN='1'; $env:E2E_APP_READY='1'; $env:E2E_BASE_URL='http://127.0.0.1:8799'; npx playwright test specs/live-tool-turn.spec.ts --project=chrome
node e2e/run-guarded.mjs --project=chrome specs/live-tool-turn.spec.ts
```
**Gate hardening test:** `e2e/specs/live-tool-turn.spec.ts` test `'gate refuses paid call when LIVE_TOOL_TURN missing'` asserts an internal probe never contacts provider (no `SUBMIT_PROMPT` frames) — falsifies "gate leaks spend".

---

## 3. PROOF SUITE C — trace promotion commands, thresholds, hardening

**Capture producer (new):** `e2e/specs/live-trace-capture.spec.ts` · `describe('W12 live trace capture (registry lane)')`, one test per registry id, each writing `docs/trace/live/captures/w12/<CAP-ID>/{frames.jsonl,dom.json,shot.png,manifest.json}` (`{id,status,head,server_bin_sha256,captured_at,hop_evidence[]}`); use `startLiveServer`, `openWsProbe`, `captureFrames`, `shot`, `writeEvidence` and a `CAPTURE_REGISTRY` constant mirroring `docs/trace/capture-registry.json`. Tests: `capture CAP-S22-RECHECK file tree` (no paid), `capture CAP-S10-REPLAY history snapshot` (**paid**), `capture CAP-S27-STREAMING ordered deltas` (**paid**), `capture CAP-SAPPROVAL approve+deny` (**paid**), `capture CAP-S15-19-CARDS node cards` (**paid**). Each asserts non-empty `frames.jsonl` (>0 frames), `surfaces/evidence` pointer present, and HEAD/binary sha recorded; **empty capture fails the test**.
**Promotion commands (exact sequence, after commit of captures):**
```
node tools/qa/verify-harness.mjs
node tools/qa/apply-live-trace.mjs
node -e "const c=require('./docs/trace/controls.json');const k=c._meta.counts;if(!(k.wired_and_works>=12&&k.missing===0&&k.raw_capture_backed_wired===k.wired_and_works))process.exit(1);console.log(k)"
node tools/qa/verify-harness.mjs
```
**Thresholds:** `total_hops=319`; `wired_and_works >= 12` (minimum after 11 suspension/approval hops + 1 recheck); `missing = 0`; `raw_capture_backed_wired == wired_and_works`; every `WIRED-AND-WORKS` hop has ≥1 evidence pointer whose target file exists and sha matches; `evidence_backed_hops >= 66`. Expected final (PROOF plan target): `wired=24, pending=253, missing=0, unverified=42` (recompute string exactly from `_meta.counts`).
**Hardening (files + tests):**
- `tools/qa/apply-live-trace.mjs`: refuse empty capture (`surfaces.length === 0` → exit 1, message `E_CAPTURE_EMPTY`), refuse stale `generated_at` older than HEAD commit, and require per-hop evidence pointer array non-empty.
- `tools/qa/verify-harness.mjs:108`: resolve every evidence pointer (`pointer.split('#')` file exists; optional `:sha`) → new check `evidence pointers resolve`, fails with `N unresolved pointers`.
- New `tools/qa/tests/trace-tools.test.mjs` (`node:test`): `rejects empty capture`, `rejects unresolved evidence pointer`, `accepts full capture` → `node --test tools/qa/tests/trace-tools.test.mjs` → `# pass 3`, `# fail 0`.
Falsifies: "1 WIRED-AND-WORKS is truthful", "trace promotion cannot fabricate passes".

---

## 4. PROOF SUITE D — G-UI-3/W1 security, G-UI-10/W2 resync, axe subcheck (closes exclusions)

**D1 — new `e2e/specs/live-security.spec.ts`** · `describe('G-UI-3/W1 origin+token boundaries (live server)')`
Fixture: `startLiveServer({ allowedOrigins: ['http://127.0.0.1:8799'] })` (helper extension §2); serve attacker page from `startDistHarness()` (different loopback port) and connect with `page.evaluate(new WebSocket('ws://127.0.0.1:8799/ws?token=<valid>'))`; capture close code + server log/ledger via `openWsProbe`/`readFile`.
Tests: `'foreign Origin is refused with E_ORIGIN_REJECTED, never silent accept'` → close code **1008**, no session row; `'bad token is refused 401/E_TOKEN_INVALID'` → HTTP 401 + body code; `'loopback bind refuses non-loopback bind'` optional (start binary with `--bind 0.0.0.0` → named refusal). **No paid.** Evidence: `docs/trace/live/captures/w12/CAP-SECURITY.json`. Then close exclusion: remove G-UI-3/W1 row(s) from `e2e/GATE-EXCLUSIONS.md`; `npx playwright test specs/gui-gates.spec.ts -g "registry"` → `1 passed`; whole file `gui-gates` no-skip. Falsifies "G-UI-3 cannot be proven", "foreign origin accepted silently".

**D2 — new `e2e/specs/live-resync.spec.ts`** · `describe('G-UI-10/W2 reconnect mid-stream (live server, paid gate)')` — **PAID** (`LIVE_ONE_PROMPT=1` + key).
Fixture: `startLiveServer({ withProviderKey: true })`, `captureFrames`; tests: `'socket kill mid-stream: SERVER_HELLO{last_seq} then no duplicated reply text'` (kill via `live.url + page.route`/`ws.close()`; expect reconnect banner, `SERVER_HELLO.last_seq` > 0, assembled text equals pre-kill+post-reconnect without duplication); `'stale mutation returns E_RESYNC_REQUIRED and resnapshot restores'` (send `REWIND`/stale command; expect named refusal + `HISTORY_SNAPSHOT`). Expected `2 passed`; evidence `docs/trace/live/captures/w12/CAP-RESYNC.json`. Remove G-UI-10/W2 rows; rerun registry test. Falsifies "reconnect can duplicate text", "G-UI-10 unprovable".

**D3 — axe subcheck closure (G-UI-8)** — **NETWORK + LOCKFILE-CHANGE (see §7).**
Command (owner-approved): `cd e2e && npm install --save-dev --save-exact @axe-core/playwright@4.11.0` (pin exact; modifies `e2e/package.json` + lock; network). New test in `e2e/specs/gui-gates.spec.ts` named `'G-UI-8: axe serious/critical violations are zero on web/dist'` with fixture `startDistHarness` + `AxeBuilder`; expected `0` serious/critical, evidence `docs/trace/live/a11y-axe.json`; then remove `axe subcheck excluded` row and rerun registry test. If install is refused: keep exclusion row, record `E_AXE_DEP_UNOBSERVED` (do not claim closure). Falsifies "G-UI-8 axe cannot be closed" or "a11y clean".

---

## 5. PROOF SUITE E — strict gate sequence (exact order, expected outputs)

Preconditions: all wave edits committed; `run-gates.ps1`/`build-release.ps1`/manifests pushed to worktree; app launched at `127.0.0.1:8799` with `E2E_APP_READY=1`, `E2E_BASE_URL=http://127.0.0.1:8799`; `DEEPSEEK_API_KEY`/`LIVE_*` only for paid runs.

| # | Command | Expected output / exit |
|---|---|---|
| E0 | `git rev-parse HEAD; git status --porcelain` | HEAD recorded; empty status (quiescence) |
| E1 | `powershell -NoProfile -ExecutionPolicy Bypass -File tools/qa/run-gates.ps1 -DryRun` | plan printed; `GATES: PASS` (no execution) exit 0; no writes |
| E2 | `node tools/qa/check-docs-truth.mjs` | `11/11` (or current count) checks, exit 0; no `1.2.0-dev` claimed as released |
| E3 | `cargo test --workspace --locked` (from `backend/`, PATH = cargo bin + w64devkit) | `test result: ok. N passed; 0 failed; 2 ignored` (record N; baseline ≥286) |
| E4 | `cd web && npm.cmd run typecheck && npm.cmd test -- --run && npm.cmd run build` | exit 0; web tests count recorded; build green |
| E5 | `node tools/qa/verify-harness.mjs && node tools/qa/apply-live-trace.mjs` | `checks=N pass=N fail=0`; promotion idempotent |
| E6 | `powershell … run-gates.ps1 -Rehearsal -RequireE2E` | **mocked gate only**; real e2e pass count; `GATES: PASS` exit 0; `REHEARSAL` marker on dirty tree; **zero skips** |
| E7 | `powershell … run-gates.ps1 -Release -RequireE2E` on the current pre-release tree | exactly `GATES: PRE-RELEASE` **exit 3** (never PASS) |
| E8 | paid: `run-gates.ps1 -Live -RequireE2E` | **PAID**; live specs (`live-streaming`, `live-replay`, `live-tool-turn` if gate set) execute; `GATES: PASS` exit 0 **only** if all live pass; skip → FAIL |

**Falsifies:** "strict gate is green on this tree" (E7 must be exit 3), "zero-skip release mode" (E6/E8), "docs truth checked" (E2/E5).

---

## 6. PROOF SUITE F — 1.2 seal topology fix (source C + seal D)

**Problem:** `build-release.ps1:228-250` requires `tag_target == HEAD`; the documented release model needs **source commit C** (code) and **seal commit D** (C + evidence/manifest/docs only) with the release tag at D. The gate currently rejects D (`E_RELEASE_TAG_MISMATCH`) and the manifest cannot express the topology.

**Fix (exact):**
1. `tools/release-manifest.json` schema → `hivemind.release-manifest.v2`: keep `version, tag, tag_target, source_commit, artifacts, package, web`, add `sealed: bool`, `seal_allowlist: string[]` (exact paths: `tools/release-manifest.json`, `dist/**`, `docs/RELEASE-EVIDENCE.md`, `plans/waves/README.md`, `docs/RELEASE.md`, `README.md`), `seal_diff: {from,to,sha256}`. For 1.2: `source_commit=C`, `tag=hivemind-1.2.0`, `tag_target=null` (resolved by gate), `sealed=false` at build; after D, `sealed=true`.
2. `tools/build-release.ps1`: add `-SourceOnly` (build artifacts+manifest at C with `sealed=false`, `tag_target=null`; skip tag equality requirement) and `-RecordSeal -SealCommit <D>` (write `sealed=true`, `seal_diff.from=C`, `.to=D`, `sha256` of `git diff --name-only C..D`, re-hash artifacts and FAIL on drift; no rebuild). Manifest write block at `:530-560`.
3. New pure checker `tools/qa/check-seal-topology.mjs` (export `evaluateSealTopology(manifest, gitInfo)` + CLI) and `tools/qa/tests/seal-topology.test.mjs` (`node --test`):
   - `source-only manifest is not sealed` → `{ok:false, code:'E_SEAL_UNSEALED'}`
   - `sealed when HEAD == tag target D and diff[:] ⊆ allowlist` → `{ok:true}`
   - `fails when diff contains src file` → `E_SEAL_DIFF`
   - `fails when HEAD != tag target` → `E_SEAL_TAG`
   - `fails when artifact hash drifted` → `E_SEAL_ARTIFACT`
   Expected: `node --test tools/qa/tests/seal-topology.test.mjs` → `# pass 5`, `# fail 0`; `node tools/qa/check-seal-topology.mjs` → `PASS sealed source=<C> seal=<D>` exit 0.
4. `tools/qa/run-gates.ps1` release branch (`:248-316`): when `manifest.sealed == true` → resolve tag target; require `HEAD == tag_target`, `git diff --name-only source_commit..HEAD ⊆ seal_allowlist`, artifact re-hash matches manifest, else fail (`E_SEAL_*`); keep pre-release exit 3 path; `-RequireE2E` zero skips.
5. **Actual 1.2 seal sequence (owner actions; destructive/paid flags in §7):**
   1) commit source C (all code/tests/docs except release artifacts) — clean; 2) `build-release.ps1 -SourceOnly -Tag hivemind-1.2.0` at C (**overwrites `dist/` + manifest**; no paid); 3) commit D = C + `dist/`, manifest, evidence/README updates; 4) `build-release.ps1 -RecordSeal -SealCommit <D>` (writes sealed manifest; **overwrites manifest only**); 5) amend/commit D' (manifest inclusion) — the seal diff allowlist contains only those paths, so D' remains a seal commit; 6) owner creates annotated tag `hivemind-1.2.0` at D' (**destructive-ish owner git action**); 7) `run-gates.ps1 -Release -RequireE2E` → `GATES: PASS` exit 0; 8) paid `-Live` final gate (§E8) → `GATES: PASS`.
   Determinism proof: run step 2 twice from clean C; compare `dist/*.zip` sha256 and manifest artifact hashes (must be identical) → `E_SEAL_DETERMINISM` if drift.

**Falsifies:** "1.2 can be sealed at HEAD without topology change", "the tag/source mismatch is only a rehearsal flag", "1.2 released" (only gate PASS + tag D + manifest sealed allows the claim).

---

## 7. Paid / destructive command register (owner approval required)

| Class | Command | Why flagged |
|---|---|---|
| **PAID** | `LIVE_ONE_PROMPT=1` + key for `live-streaming`/`live-replay`/`live-tool-turn`; `cargo test -p hiverouter-core --test provider_live -- --ignored`; `run-gates.ps1 -Live`; trace capture tests CAP-S10/S27/SAPPROVAL/CARDS | real DeepSeek spend; requires key; budget cap/amount to be set by owner |
| **PAID** | `e2e/specs/live-resync.spec.ts` (needs an active streaming run) | runs a real prompt |
| DESTRUCTIVE (release) | `tools/build-release.ps1` (all modes) — overwrites `dist/` and `tools/release-manifest.json`; `-RecordSeal` rewrites manifest | release artifacts are overwritten; not reversible without git |
| DESTRUCTIVE (git) | `git tag hivemind-1.2.0 <D>`; any tag move is **forbidden** (immutability) | owner-only; moving tags rewrites release provenance |
| DESTRUCTIVE (trace) | `node tools/qa/apply-live-trace.mjs` mutates `docs/trace/controls.json`; capture specs overwrite `docs/trace/live/captures/**` | evidence truth data overwritten |
| DESTRUCTIVE/NETWORK | `cd e2e && npm install --save-dev --save-exact @axe-core/playwright@4.11.0` | network install + lockfile change |
| NOT paid | all mocked-WS e2e, `cargo test` workspace, `node --test` tool tests, `run-gates` without `-Live` | no provider call |

---

## 8. Falsification matrix — current claim → proof that would falsify it

| Current claim (source) | Proof | Verdict rule |
|---|---|---|
| "No runtime claim yet — tool loop/suspension/UI not implemented or run" (`plans/waves/README.md:35,47`) | A1–A12 + B1–B6 + tool_loop_matrix | any failing proof ⇒ claim **stays**; all green + live ⇒ **falsified** (claim becomes release evidence) |
| "1.1.0 sealed; 1.2.0-dev pre-release; `-Release` exit 3" (`RELEASE-EVIDENCE.md`, `run-gates.ps1:248-316`) | E1, E7 | E7 ≠ exit 3 ⇒ gate claim falsified |
| "trace truth 1/264/5/49, raw capture EMPTY" (`controls.json`) | C capture+promotion | any WIRED without pointer ⇒ truth claim falsified |
| "G-UI-3/W1, G-UI-10/W2, axe subcheck cannot be proven" (`GATE-EXCLUSIONS.md`) | D1–D3 | passing live/axe ⇒ exclusion falsified (must close) |
| "Live 1.1 proof READ-level only; no 1.2 live proof" (`PROOF-RELEASE-PLAN.md:17`) | B1–B4, E8 | passing 1.2 live ⇒ falsified for 1.2 |
| "Exactly one side effect / nonce replay protection" (`approval_ledger.rs` claim) | A2/A7/A12, B2/B3 | duplicate execution ⇒ falsified |
| "Strict gate zero-skip release mode" (`run-gates.ps1:23-33`) | E6, E8 | skip in `-Release/-RequireE2E` ⇒ falsified |
| "1.2 seal requires HEAD == tag target" (`build-release.ps1:228-250`) | F tests + E-seal | seal at D via allowlist ⇒ current topology assumption falsified/fixed |
| "1.2 released" — no current claim | F + E8 | only tag D + sealed manifest + live gate PASS may create it |

---

## 9. Execution order & critical path

1. **A (suspension, mocked)** — fastest, no paid; requires the in-flight engine lane to land/commit. 2. **tool_loop_matrix** (wiremock). 3. **B (live tool-turn)** — paid, needs key + budget + engine. 4. **C (trace captures → promotion hardening)** — replay/recheck no-paid first; streaming/approval/cards paid. 5. **D (G-UI-3 → D2 → axe)** — D1/D2 after B; axe needs owner install. 6. **E (strict gate)** — after commits; E7 must be exit 3 until seal. 7. **F (seal topology)** — after A–E green; build/tag owner actions; final `-Release -RequireE2E` + paid `-Live`. No item may be reported DONE before its evidence row is committed and quoted (artifact+command+threshold).

**Blockers:** suspension/tool-loop engine lane is **uncommitted** (8 dirty paths) — A/B are RED-first against a moving tree; paid key/budget approval needed for B/C(partial)/D2/E8; axe install needs network+lockfile approval; owner-only git tag/build actions for F. **No code/builds were executed for this plan.**

---

## 10. Receipt

Read-only planning: **only this file written**. Exact proof plan covers deterministic suspension tests (11 backend + 12th ledger + 2 mock e2e), live tool-turn spec with independent `LIVE_TOOL_TURN` gate (6 tests, paid flagged), trace capture/promotion with thresholds and tool-hardening tests, G-UI-3/W1 + G-UI-10/W2 + axe closures, strict gate sequence with expected exit codes (0/3), and the source-C/seal-D 1.2 topology with a pure checker + 5 node tests. Each test names file, describe/test, fixture, selectors/commands, expected output, and the claim it falsifies; every paid or destructive command is flagged in §7.
