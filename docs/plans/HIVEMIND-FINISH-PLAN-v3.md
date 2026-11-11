# HiveMind Finish Plan v3 — evidence-derived, code-level, 40 rounds

**Status:** v3 fact-checked by two independent agents; corrections below are adopted.
Not a completion claim.
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1`
**Audit baseline:** `6a41250`; **execution baseline:** `abfff55` (analysis artifacts +
QA gate/docs checker committed), version `1.2.0-dev`, package `dist/HiveMind.zip`
`15BF4E61…6CA5ADA`, server `F369D7AC…A7CF871`. Current tracked count is **1,593**
(1,573 at the census pin + 20 analysis artifacts); the census remains pinned to
`6a41250` and needs a small addendum, not a silent rewrite.
**Inputs:** `docs/analysis/lanes/L1..L7`, `docs/analysis/WATCHDOG-LEDGER.md`,
`PLAN-FACTCHECK-CORE.md`, `PLAN-FACTCHECK-UX.md`, `GAP-AUDIT-1/2.md`,
`GAP-PATCH-1/2.md`, `docs/analysis/CODEBASE-CENSUS.md`,
`docs/analysis/LEAD-RESEARCH-PSEUDOCODE-LANGUAGES.md`.
**Census scope correction:** 1,573/1,573 paths indexed; the census feature table rows
sum to 249, while its verdict counts 216 in the eight named families; 23 migrations and
10 prompt assets sit outside that total, and 78 refusals + 4 caps are enumerated but not
counted. Do not repeat an unqualified "216/216 features".
**Authority:** operator-gated evolution only; no autonomous source mutation, no
AGI claim, no unmeasured scale claim.

## 0. Freeze ledger

- HEAD `6a41250`; tree dirty by `docs/analysis/**` and this plan. R1 commits both, then
  `git fetch --unshallow` or explicitly records that ancestry is unavailable (shallow
  clone), before any rebase/ancestry claim.
- Paid-call policy: no provider call without an explicit receipt; TokenLab is optional
  and rate-limited; keys never printed.
- Gate commands (environment-correct):
  `$env:PATH="$env:USERPROFILE\.cargo\bin;"+$env:PATH; Set-Location backend; cargo test --workspace --locked`;
  `cmd.exe /c npm.cmd run typecheck` and `cmd.exe /c npm.cmd test -- --run` in `web/`;
  `node --test tools/qa/tests/*.test.mjs`; install smoke; full battery.
- The current tree cannot pass `run-gates.ps1` (L6 §9.2) because the manifest is
  sealed `1.1.0` while the tree is `1.2.0-dev`. Acceptance is: strict gate is expected
  RED at G7; `-Rehearsal` has no non-G7 failure; `-Release` exits 3 and is recorded as
  PRE-RELEASE; do not claim a green seal.

## 1. Truth constraints

0. Corrections adopted from the watchdog and gap audits:
   - `finish`/`report_result` **do** complete tasks; L2 R6 is refuted. Test them, do
     not "fix" them.
   - Proposals auth header is a style inconsistency, not a 401 path; server accepts
     `Authorization: Bearer` or `x-hivemind-token`.
   - Counts: **63** method+path routes including `/ws` (62 excluding); **78** refusal
     codes (the 79th is a test fixture); **639** declared Rust test functions
     (373 unit + 266 integration); **231** web `it/test` sites; **81** Playwright
     `test()` sites in 16 specs; **2** QA guard suites; `hivemind-cli.mjs` 80,007 B.
   - Current dirty tree must be made clean or explicitly excluded before a gate PASS;
     do not call `-Rehearsal` exit 3 green.
   - Keep `build.rs` unowned/audited explicitly; census's L4 owner label is wrong.
1. The HyperAgent is currently a one-shot controller plus answer-only final, not a
   governor (L1 #2; `execution.rs:2056`, `provider_executor.rs:672-697`).
2. The tool bridge's jail does not deny the data directory even though the engine jail
   does (L2 #2; `http.rs:259-262` vs `:347-351`).
3. `cmd.exe` in the shell allowlist makes approved shell arbitrary user-level code
   execution (L2 #1; `shell.rs:61-72`).
4. Approval grants ignore TTL and bind only `(run, tool, args_sha256)` (L2 #4;
   `approval.rs:935,942-951`).
5. Root control runs a second, unregistered tool vocabulary (L1/L2;
   `provider_executor.rs:39-53,583-666`).
6. Structural rows can settle `Completed` without executing; single-node plans never
   run the root (L1 #9; `execution.rs:1784-1819,2206`).
7. `finish`/`report_result` **do** complete tasks (`execution.rs:5411-5462`); L2's R6
   was refuted by the watchdog — do not "fix" a working path.
8. 12 tools / 5 classes / 35 grants; 7 grants are unreachable (L2/L7).
9. The installed build is unidentifiable; launcher redirects pipes with no readers and
   hardcodes a dev path/port (L6 #1/#3/#6).
10. The UI has real S1 wiring defects: Agents SPAWN dead, workspace tabs inert off
    Workspace, header chat toggle no-op (L5 #1-#3).
11. New gap findings to include: daemon script env leak (`daemon.rs:428-445` has no
    `env_clear`/allowlist and inherits `DEEPSEEK_API_KEY`); e2e guard fails open when
    `results.json` is missing/unparsable (`e2e/run-guarded.mjs:16-35`); planner
    validation omits fan-out/agent caps (`planner.rs:50-129`) then fails after the
    planning call; history maps corrupt timestamps to 1970 (`history.rs:79`); exact
    `/api` falls through to the SPA (`static_ui.rs:48`); `handshake.rs` is dead public
    API; `build.rs` rejects prerelease/nightly and embeds a non-reproducible
    `rustc -Vv`; 25 tracked paths are absent from all lane reports, including
    `web/index.html`, `web/package-lock.json`, `SOURCES.json`, and 21 served FigCraft
    SVG boards.

## 2. Work packages

Each work package names files, required behavior, tests, and rollback. The Lead owns
the final patch integration; helper agents get disjoint file scopes.

### WP-SEC-1 — containment

Files: `backend/crates/hiverouter-tools/src/{shell.rs,side_effects.rs,jail.rs}`,
`backend/crates/hiverouter-core/src/{http.rs,tools_bridge.rs,daemon.rs}`, containment tests.

Required:
- remove **`cmd.exe` only** from `conservative_default`; do not claim argv is a shell
  (`shell.rs` passes args directly, so `&|>` are literal chars); document that
  `git.exe`/`cargo.exe` remain arbitrary code execution and that the approval gate is
  the only boundary; deny interpreter hosts even when explicitly allowlisted; update
  `shell.rs:368-465`, `approval.rs:1512`, `tools_bridge.rs:921`,
  `execution.rs:783/1036`, `suspension.rs:287/1073` fixtures to an explicit safe
  `ShellPolicy::new(["git.exe"])` test policy, never `cmd.exe`;
- pass `data_dir` **and `db_path`/+wal/+shm** into `ToolsBridge` and deny them; fail
  closed if a deny path cannot be canonicalized; update all four constructor sites
  (`http.rs:260`, `tools_bridge.rs:625`, `lead_bugcheck_boot.rs:48`, `suspension.rs:276`);
- daemon `.exe`/`.ps1` jobs get `.env_clear()` + `env_safety` allowlist; test that
  `DEEPSEEK_API_KEY` is absent (do not print it), including a `.ps1` case;
- add tests: `file_read("data/...")`/custom `--db` path and `file_write` refuse;
  interpreter hosts refused even by an explicit test policy; no regression to
  legitimate workspace reads/writes; `send_to_sibling` must not bypass grant
  verification (add expiry test).

Acceptance: focused tool tests + full backend suite. Rollback: revert the files; no
schema change.

### WP-SEC-2 — approval integrity

Files: `backend/crates/hiverouter-tools/src/approval.rs`,
`backend/crates/hiverouter-core/src/{tools_bridge.rs,execution.rs,run_state.rs,approval_wait.rs}`,
approval tests.

Required:
- `allow_once` verification requires `expires_at > now`; `allow_session` remains
  intentionally non-expiring and scoped by `(run, tool, args)`;
- for `allow_once`, bind `approval_id`/`tool_call_id` through `execute_approved` and
  the `verify_grant` call sites (14, not 15); do not break repeated session grants;
  select the grant **by approval_id**, not oldest-by-args (two identical-arg grants
  can shadow each other);
- order ledger append before the decision becomes irrevocable, or add a real
  post-decision `cancelled`/`superseded` state honored by the timeout path (the
  existing `cancel_pending` CAS only matches `decision IS NULL` and is a no-op after a
  decision);
- concurrent suspension: implement a bounded waiter queue/serialization (one task
  suspends, others wait in `approval_wait` and proceed in order) — the plan cannot
  selectively refuse pre-dispatch because `PlanNode` has no permission hints;
- no args-free replay approval; preserve named refusals.

Acceptance: allow_once-before-expiry then execute-after-expiry refuses; two identical-
arg grants cannot shadow; two-concurrent-suspension test; simulated ledger failure;
allow_session beyond 120 s still works; `send_to_sibling` expiry test; replay refusal;
full suite. Files include `run_state.rs` and `approval_wait.rs` (assign
`approval_wait.rs` explicitly to this WP). Rollback: revert; no migration.

### WP-SEC-3 — root tool truth

Files: `backend/crates/hiverouter-core/src/{provider_executor.rs,execution.rs}`,
`backend/crates/hiverouter-tools/src/{tool.rs,census.rs}`, tool-loop tests.

Decision: **isolate** the root control vocabulary as a bounded validated control
surface, not a registry tier. Add schemas/validation/audit for its six tools, remove
the false tier claim, and leave `ToolId` at 12. Remove the unreachable ToolAgent tier
grants and the dead HyperAgent registry grants; update `census.rs` 35 → 28 and its
tests. Reconcile `finish`/`report_result`: add task-level tests proving `finish`
completes, `report_result success=false` labels failure, and no extra provider call.

Acceptance: root-control schema validation/refusal tests; task-level terminal tests;
census expected-count update; no dead-grant count. Rollback: revert; no schema change.

### WP-SCHEMA-1 — freeze and ship v24 once

Files: `backend/crates/hiverouter-core/src/{db.rs,store.rs,main.rs}`,
`backend/crates/hiverouter-core/src/tools_bridge.rs`, migration tests.

Required: freeze **v24 = `tasks.instruction TEXT` only** (implemented in this pass).
Memory role-key/approvals/state-integrity columns are **v25 or later**; do not add them
to v24 after it ships. The conditional one-pending approval unique index remains a
post-migration step in `ToolsBridge::migrate` with the existing named refusal; do not
delete that logic. Do not gate all required health checks — only an approval migration
failure (or let fatal `Database::migrate` own it). Add a pre-v19 duplicate-hyper precheck
with `E_SCHEMA_CONFLICT` before v19 applies. v24 down restores the pre-up backup; do not
drop a pre-existing approvals table destructively.

Acceptance: upgrade/downgrade roundtrip, pre-migration backup, legacy two-hyper and
approval-row fixture, create_all parity. Rollback: v24 down + backup restore.

### WP-GOV-1 — task instruction and root objective

Files: `backend/crates/hiverouter-core/src/{db.rs,entities.rs,store.rs,execution.rs,http.rs,provider_executor.rs}`.

Required: use the frozen v24 `tasks.instruction TEXT`; nullable, backfilled null;
insert/select/update; update `provider_executor.rs` root schemas so the model is
actually advertised `spawn_superagent {objective, persona?}` and
`assign_task {task_id, objective}` (today `spawn_superagent` has `properties: {}` and
`assign_task` is absent); update `ROOT_CONTROL_SYSTEM`; child prompt includes the
instruction; API detail/roster exposes a bounded instruction; auto-voters get
proposal/lens instructions. A provider-conformance test must prove the advertised
schema accepts the real root args.

Acceptance: upgrade/downgrade uses WP-SCHEMA-1, instruction propagation and
cap/terminal refusals, provider-conformance test, full suite. Rollback: feature revert;
v24 already shipped under WP-SCHEMA-1 (restore backup for schema changes).

### WP-GOV-2 — monitor/review

Files: `backend/crates/hiverouter-core/src/{execution.rs,provider_executor.rs}`.

Required: after dispatch, one bounded root monitor turn with explicitly defined
`send_instruction {task_id,text}` and `cancel_task {task_id,reason}` tools (no such
tools exist today; `kill_worker` is a WS command), bounded repair semantics (one
replacement branch only if the failed task is terminal and caps remain), no mutation
after final; final remains tool-free and must return a truthful
status/changes/non-changes/evidence report. Root creation stays under caps; no
duplicate branches.

Acceptance: scripted-provider test proving assign → child uses instruction → monitor
reacts → final; late-mutation refusal; cap refusal before row creation; no zombie task.

### WP-GOV-3 — caps/settlement

Files: `backend/crates/hiverouter-core/src/{execution.rs,run_state.rs,caps.rs}`,
`backend/crates/hiverouter-core/tests/*`.

Required: atomic tool-call reservation (not read/check/increment across tasks);
provider-call accounting that covers legal plans/tool loops; ensure no structural
`Completed` without execution/result or mark it truthfully; make single-node plans
execute the root; do not bypass `run_state` for running→queued.

Acceptance: parallel tool budget test with ≥2 concurrent tasks ≤40; a 40-node plan with
at least one 10-turn tool task stays under the defined provider-call formula
(`1 planner + root-control turns + task turns + final + continuations`); single-node
execution test; settlement-state matrix. Rollback: revert; no schema.

### WP-HARDEN-1 — boot/edge truth

Files: `backend/crates/hiverouter-core/src/{planner.rs,history.rs,static_ui.rs,handshake.rs,remote_supervisor.rs,build.rs,checks.rs}`,
`backend/crates/hiverouter-core/src/lib.rs`, `e2e/run-guarded.mjs`, tests.

Required:
- daemon env is handled in WP-SEC-1 (not deferred here);
- `Plan::validate` enforces fan-out ≤8 before the planning call returns (agents ≤64 is
  unreachable under `MAX_PLAN_NODES = 40`; state that), with named refusals;
- `history.rs` refuses corrupt timestamps instead of 1970; bounded revision reads;
- exact `/api` does not fall through to the SPA; non-GET/HEAD returns JSON 405;
- **delete** `handshake.rs`, its `lib.rs` re-exports, `tests/handshake.rs`, and any docs
  claim (decision: dead public API, not wire it in 1.2-dev);
- `build.rs` really parses minor from `1.93.0-nightly` correctly; the actual defects are
  patch-level ignored and the full version line embedded. Extract parsing to a testable
  `src` module (build scripts are not cargo-test targets); add `checks.rs` (the `env!`
  consumer) if changing the variable;
- supervisor due time honors interval; state-write failures are surfaced.

Acceptance: one test per item, focused then full suite; no production behavior
weakened. Rollback: revert; no schema.

### WP-STATE-1 — migration/state integrity

Files: `backend/crates/hiverouter-core/src/{db.rs,store.rs,http.rs,snapshot.rs,recovery.rs,execution.rs,tools_bridge.rs,main.rs}`,
`backend/crates/hiverouter-tools/src/approval.rs`.

Required: use the schema frozen by WP-SCHEMA-1 (do not edit v24 here); keep the
conditional approval unique index as a post-migration step; memory-clear guard in the
same transaction as delete; role clear uses `source_task_id → tasks.role` (v24 has no
`agent_key`), with a v25 column only if the query proves insufficient; handle legacy
NULL rows; live memory writer uses the strict writer via
`execution.rs`; governance prefix guard complete; snapshot revision shared max+1 (change
`execution.rs` owner); recovery scans all active runs. Gate only approval-migration
failure, not every required health check.

Acceptance: orphaning test, both-flat-and-bank role-clear test, strict-writer valid/
invalid tests, snapshot collision/regression test, recovery test; full suite.
Rollback: v24 down + backup; feature reverts only.

### WP-API-1 — CLI/API truth

Files: `tools/hivemind-cli.mjs`, `backend/crates/hiverouter-core/src/{http.rs,ws.rs}`,
`docs/contracts/HTTP-API-v1.md` (new).

Decision: **document WS-only run/task control**; do not add new HTTP mutation routes in
this plan. Required: CLI sends the session token and launch header from an explicit
source (`HIVEROUTER_LAUNCH_TOKEN` or `%LOCALAPPDATA%\HiveMind\launch.token`), uses WS
for mutations, and never logs the token; bounded provider proxy timeouts; consistent
refusal shape; `docs/contracts/HTTP-API-v1.md` records the read-only HTTP + WS command
surface and route inventory.

Acceptance: CLI mutation test against a real local server started with a launch token
(WS path), route contract test, refusal-shape test. Rollback: revert; no schema.

### WP-UI-1 — repair existing shell

Files: `web/src/panels/{AgentsPanel,AgentHierarchy,LibraryPanel,Header,ApprovalOverlay,ToastRegion,ToolActivityPanel,MemoryBanksPanel,ControlBar}.tsx`,
`web/src/state/store.ts`, `web/src/App.tsx`, `web/tests/*`.

Required: pass spawn handlers to Agents tree; workspace tabs set nav; header chat
switch works everywhere; snapshot degraded banner + re-sync; approval queue navigation
and focus; tool attribution (`task_id`, bytes, sha256) rendered; change both Proposals
writes to the proven `x-hivemind-token` helper (the server accepts Bearer too; this is a
style/consistency fix, not a 401 fix); memory deps runId; kill dialog uses ConfirmDialog;
keep `npm test`/typecheck green (they already pass: tsc 0, 231/231) and add new tests.
Also: cover the 25 unowned UI assets (`web/index.html`, package-lock, `SOURCES.json`,
21 served FigCraft SVGs); remove the synthetic FigCraft gallery fallback and NaN count;
clear mock timers on `close()`; correct splash token wording without inventing alpha
tokens; read `__APP_VERSION__` instead of hard-coded `HiveMind v1.2-dev`; keep the mock
dev boundary intact.

Acceptance: web tests, typecheck/build, UI e2e at 1440; add `--width/--height` support
and a 375px app-shell assertion (do not claim a 375 spec that does not exist). Add a
Playwright route-abort test for the snapshot banner and a seeded approval fixture;
state failure-injection mechanics explicitly. Rollback: revert.

### WP-UI-2 — existing-surface product polish (10–20% ceiling)

Files: new `HivePulse.tsx`, `ToolsMatrixPanel.tsx`, `MissionGallery.tsx`,
`FirstRunNotice.tsx`; existing `App.tsx`, `ChatCore.tsx`, `AgentsPanel.tsx`,
`ProposalsPanel.tsx`, `SettingsPanel.tsx`, `tokens.css`, `splash/*`, styles, tests.

Required: a HivePulse strip inside the existing Workspace (not new navigation); an
8-template mission gallery sourced from the existing `GET /api/templates` route (the
repo has 8 templates, not six; no mission entity or mission API is invented); a
tier×tool matrix sub-tab inside the existing Agents surface; per-task detail presented
with the existing task selection/RunDetail; first-run provider notice in Settings;
governance timeline inside the existing ProposalsPanel. Add only alpha/chrome material
tokens (no new palette swatch), preserve IA/palette, accessible focus/contrast/
reduced-motion, and make the splash progress indeterminate (no fake numeric percent).

Explicitly cut from this plan: `MissionControlPanel`, `MissionsPanel` as new nav,
`CommandPalette`, new mission API/entities. They require operator ceiling approval and
new backend routes and are a follow-up, not this 40-round execution.

Acceptance: real API-bound e2e for HivePulse/8 cards/tools matrix; screenshots at
1440 and 375; deterministic token/contrast test plus manual new-pair contrast (axe is
not installed; do not claim an axe run); no hardcoded mock; no new nav section.
Rollback: revert UI files; no backend schema.

### WP-INST-1 — installable desktop

Files: `tools/build-desktop.ps1`, `tools/desktop-launcher/HiveMindLauncher.cs`,
new `tools/install-desktop.ps1`, `tools/uninstall-desktop.ps1`,
`tools/set-provider-key.ps1` (non-interactive safe mode), package README.

Required: keep the existing beside-launcher precedence; workspace resolution is
`HIVEMIND_WORKSPACE_ROOT` → `%LOCALAPPDATA%\HiveMind\workspace` (created with notice) →
explicit first-run choice; do **not** default agents to all of Documents. Drain
redirected pipes; scan 8099–8199 for a free port; remove the hardcoded dev path; write
`BUILD.json` with `commit`, `version`, `server_sha256`, `launcher_sha256`, `built_at`,
`dirty` (not the zip's own hash); fix `build-desktop.ps1`'s `--release`/`$Configuration`
mismatch. Installer copies to `%LOCALAPPDATA%\Programs\HiveMind\app`, supports
`-InstallRoot`/`-DataRoot` for a clean-profile test, creates Desktop/Start Menu
shortcuts, writes only the non-secret provider profile (key setup is a hidden first-run
prompt or `set-provider-key.ps1`), verifies `/health` and `/health.version`, and
uninstall preserves `%LOCALAPPDATA%\HiveMind` unless `-PurgeData`. Package ships
`set-provider-key.ps1`, `CHECKSUMS.txt`, version/commit/build date, workspace/data
semantics, first-run/key rotation, and uninstall docs.

Acceptance: clean profile = fresh `-InstallRoot` + fresh `-DataRoot` + no pre-existing
`provider.key`; install → shortcut launch → `/health.version == 1.2.0-dev` → screenshot
at a named evidence path; uninstall/reinstall preserves data; no **plaintext** key in
package/source/command line/logs/output (the DPAPI `provider.key` is the only key file);
fresh zip extract can perform every documented step without the repo. Rollback:
uninstall script; data preserved.

### WP-TEST-1 — gate/battery

Files: `tools/qa/run-gates.ps1`, `tools/qa/tests/*`, backend tests,
`tools/qa/run-full-battery.ps1` (new), `e2e/*`.

Required: `node --test tools/qa/tests/*.test.mjs` step; clean-tree gate on analysis
artifacts (commit them or explicitly exclude); e2e guard must fail when results.json is
missing/unparsable or stale; build identity check; install smoke; one long HMARS battery
(hierarchy, instructions, tools, memory, A2A, remotes/daemons, votes/apply, monitor,
settlement, no zombies, API cap); crash/restart; 8/16/40 scale; UI e2e.

Acceptance: `node --test tools/qa/tests/*.test.mjs` is wired into `run-gates.ps1` and
runs the 8 already-passing cases (5 seal + 3 trace); e2e zero-collected fails; focused
suites GREEN; gate truth is recorded as: strict `run-gates.ps1` RED at G7 against the
sealed 1.1.0 manifest, `-Rehearsal` has no non-G7 failure, `-Release` exits 3
PRE-RELEASE. Do not claim "all commands exit 0" and do not re-seal 1.2. Create
`tools/qa/run-full-battery.ps1` with named steps/expected outputs before R34. Rollback:
tests are additive; gate changes reversible.

### WP-DOCS-1 — truth/release

Files: `README.md`, `docs/audit/HYPERAGENT-TRUTH-REPORT.md`,
`docs/audit/HYPERAGENT-PSEUDOCODE-REPORT.md`, `docs/RELEASE-EVIDENCE.md`,
`docs/RELEASE.md`, `docs/reports/FINAL-REPORT.md`, `docs/DOWNLOADS.md` (new),
`docs/evidence/live/INDEX.md` (new), Downloads copy, `dist/*`.

Required: current HEAD/version/counts; 12/5/35 tool truth; implemented votes/UI vs
code-apply absent; installed-build/Downloads section; correct zip/package claims; keep
sealed 1.1.0 separate from 1.2.0-dev; no unverified "live" statement. Also:
- fix `check-docs-truth.mjs` false positive on "Working tree: 1.2.0-dev (unreleased)"
  and false negative on "response is streamed"; update the false 19/19 claim in
  `RELEASE-EVIDENCE.md` to the measured 17/19 before the fix;
- add `e2e/README.md` corrections (81 declarations/16 specs, 0 fixme, 1/264/5/49 of
  319 trace, existing ASSETS-INDEX, no nonexistent bundle);
- split census feature totals 216 / +33 / 82 and correct 21 SVG + SOURCES.json, line
  totals, DIFF_* decode-refusal pins;
- add `docs/evidence/live/INDEX.md` with observed_at/head_commit/what-proves, mark
  aggregate receipts READ not VERIFIED, add bytes/body_not_captured, hard_kill fields,
  and both exact-two receipts;
- README web_fetch claim: add receipt fields or rewrite to what the receipt proves;
- release manifest: record stale web_dist hashes or re-seal after a real build; cite
  or drop the unverified 393,216 token ceiling; keep 1.1.0 sealed;
- add claims register/provider truth/CODEMAP banners; state the clone is shallow and
  do not claim tag ancestry until `fetch --unshallow`.

Acceptance: after the checker fix, `node tools/qa/check-docs-truth.mjs` exits 0 with
19/19; `verify-harness.mjs`; named link/path scan; no stale claims remain.
Rollback: docs only.

### WP-MISSION-1 — 8-template gallery/use cases

Files: `web/src/panels/MissionGallery.tsx`, `web/src/api/**` (templates fetch),
`web/tests/*`, `e2e/specs/*` (one gallery/launch spec); no new backend route.

Required: render the existing `GET /api/templates` catalogue (8 template ids from
`assets/prompts/manifest.json`) as mission cards with title/purpose/expected tools;
launching a card submits a normal run prompt through the existing WS submit path; no
mock data, no invented mission entity/API, no claim of six missions. Acceptance: a
scripted-provider end-to-end run launched from the gallery; card/empty/error states;
8/8 template count quote.

### WP-OPT-1 — optional local verifier / playground

Core decision: do **not** add external playgrounds as core. Add a local,
approval-gated structured verification tool (fixed `cargo check/test/clippy/fmt`,
`tsc`, `vitest`, `node --test` in a temp/jailed copy, bounded, no network) only after
WP-SEC-1/2. Optional external Rust Playground connector only behind `Network` +
per-call approval, disabled by default, never for HiveMind source. Official Rust
Playground facts and limits: `LEAD-RESEARCH-PSEUDOCODE-LANGUAGES.md §11`.

## 3. 40-round execution

Rounds are checkpoints, not promises; each ends with a command/evidence and a push.

| Rounds | Work | Exit |
|---|---|---|
| R1 | freeze HEAD; commit `docs/analysis/**` + plan + fact-check/gap reports and the two QA checker edits; `git fetch --unshallow` or record ancestry unavailable; push; baseline commands | clean tree; evidence record |
| R2 | WP-TEST-1: wire all `tools/qa/tests/*.test.mjs` into `run-gates.ps1` dynamically; docs checker 19/19; run baseline tests | 8/8; checker measured; no green seal claim |
| R3–R5 | WP-SEC-1 + WP-SEC-2 (incl. daemon env, shell/jail, approval allow_once/concurrency) | containment/approval tests red→green; full focused suite |
| R6 | WP-SCHEMA-1: freeze v24 up/down, pre-v19 duplicate preflight, approval base table; no later edits | migration roundtrip + legacy fixture |
| R7–R9 | WP-GOV-1 task instruction + provider root schemas (`spawn_superagent`/`assign_task`) | provider-conformance + propagation tests |
| R10–R11 | WP-GOV-2 monitor/review (`send_instruction`/`cancel_task`/repair) | scripted root monitor test |
| R12–R13 | WP-GOV-3 caps/settlement | parallel budget/40-node tool task/single-node tests |
| R14–R16 | WP-STATE-1 memory/snapshot/recovery/role-clear | state tests, full suite |
| R17–R19 | WP-HARDEN-1 planner/history/static/handshake/build/supervisor | hardening tests, full suite |
| R20 | WP-API-1 CLI/WS-only docs | CLI/API contract tests |
| R21–R23 | WP-UI-1 repair | web tests/typecheck/e2e at 1440, 375 support |
| R24–R27 | WP-INST-1 installer/launcher | clean-profile installed smoke + screenshot |
| R28–R30 | WP-UI-2 existing-surface polish + WP-MISSION-1 8-template gallery | UI e2e + screenshots + contrast |
| R31–R34 | WP-TEST-1 full battery/crash/scale/install smoke | all focused gates; no zombie/API burn |
| R35 | optional live TokenLab smoke with named receipt/paid opt-in | live evidence or explicit UNVERIFIED |
| R36–R38 | WP-DOCS-1 truth/release/docs checker | docs truth 19/19; package hashes/INDEX |
| R39 | independent verifier pass + defect fixes | no failing citation/test |
| R40 | final installed screenshot, push, Done/Not done/Unverified; create follow-up 25–50-round goal only if residual | final receipt |

## 4. Agent structure after plan approval

1. **Gap auditors (2, read-only except reports)** — complete: `GAP-AUDIT-1/2`,
   `GAP-PATCH-1/2`; findings merged into this plan.
2. **Plan fact-checkers (2, read-only)** — complete: `PLAN-FACTCHECK-CORE.md` and
   `PLAN-FACTCHECK-UX.md`; their exact corrections are applied above. No source edits.
3. **Code workers (3)** — after plan fact-check; file-ownership matrix is frozen
   before R1 and repeated in `plans/foresight/RUN-FINISH.md`:
   - `impl-security`: `backend/crates/hiverouter-tools/**`,
     `backend/crates/hiverouter-core/src/{http.rs,tools_bridge.rs,approval_wait.rs}`,
     and their tests. It does not edit `execution.rs`, `provider_executor.rs`, `db.rs`,
     or `store.rs`; daemon env flips in `daemon.rs` are Lead-owned.
   - `impl-ui-install`: `web/src/**`, `web/tests/**`,
     `tools/{install-desktop.ps1,uninstall-desktop.ps1,build-desktop.ps1,set-provider-key.ps1}`,
     `tools/desktop-launcher/**`, `dist/HiveMind/README.txt`.
   - `trailing-fixer`: no writes in the first pass; after Lead/H1/H2 diffs it may patch
     only files assigned by the Lead, and must reproduce each bug before fixing. It
     never edits a file currently owned by another writer.
   Lead owns `backend/crates/hiverouter-core/src/{execution.rs,provider_executor.rs,db.rs,store.rs,snapshot.rs,recovery.rs,planner.rs,history.rs,daemon.rs,handshake.rs,build.rs,checks.rs,run_state.rs,caps.rs,static_ui.rs}`,
   `tools/hivemind-cli.mjs`, `tools/qa/**`, `e2e/**`, `docs/**`, the plan, gates, and
   final integration; no two writers share a file.
4. All workers are read-only until the Lead assigns a task ID and file scope; no
   commits by workers; Lead runs final gates and commits. If a required file crosses
   scopes, the Lead serializes it by round rather than allowing concurrent edits.

## 5. Goal and handoff

After the two plan fact-checkers return and the Lead applies their exact corrections,
create a fresh **40-round goal** (within the requested 25–50 range) with this plan as
the execution objective. If R40 still has a named residual, create a follow-up
25–50-round goal only for that residual; do not silently reset or mark complete. The
blocked 80-round goal from the prior session is replaced, not resumed.

## 6. Acceptance / refusal rules

- No "fixed" claim without a RED test or a reproduced failure and a GREEN re-run.
- No "installed" claim without the installed shortcut opening the current build and a
  matching `/health.version` + screenshot.
- No "gate green" claim on a dirty/mixed HEAD; no skipped test counted as pass.
- No source mutation by an external playground; no key in output.
- No AGI/autonomy claim: the operator remains the fifth seat; code self-modification
  remains unimplemented.
- A blocked WP is reported as BLOCKED with the exact missing capability; it is not
  replaced by a mock.

## 7. Residual items that stay out of core

- real remote hardware/network satellite transport (current remotes are fixtures);
- real-time multi-user/web deployment;
- arbitrary 300–600-agent scale claim;
- autonomous code self-evolution/evaluation;
- external playgrounds as product dependencies (optional connector only);
- any unmeasured "learning gain" claim; lesson retrieval remains L1 mechanism only.
