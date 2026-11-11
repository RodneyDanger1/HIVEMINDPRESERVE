# HiveMind Finish Plan v3.1 — capability expansion first

**Revision 3.1 — 2026-11-11.** The capability expansion in §0 is now **mandatory first work**;
the remaining v3 hardening work follows it. The pre-revision plan is archived at
[archive/HIVEMIND-FINISH-PLAN-v3-2026-11-11.md](archive/HIVEMIND-FINISH-PLAN-v3-2026-11-11.md)
and recoverable from the pre-expansion tag and preservation repo. This is a plan edit, not an
implementation claim.

**Status:** v3 fact-checked by two independent agents; corrections below are adopted.
This revision adds the capability-expansion contract; it is not a completion claim.
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1`
**Audit baseline:** `6a41250` (historical); **product commit:** `806251b`; **pre-expansion tag:**
`pre-capability-expansion-2026-11-11` → `014d0d7`; **plan revision commit:** `b0e60c8` plus this
edit. Installed `1.2.0-dev`, schema 24; package SHA-256
`B4BAC18D29560D5C63404649C2953E359FFC6170C1214A3AA1B54DE127811061`; server SHA-256
`D9D7690287E5D9D84CF655525690983A1F109D625EFD7CFD7168CD2529732C78`. Tracked count at
`b0e60c8` is **1,645**; the census pin remains `6a41250` and its historical numbers must not be
presented as the current count. Working tree at revision time is docs-only dirty (plan files), not
the old `docs/analysis/**` state.
**Inputs:** `docs/analysis/lanes/L1..L7`, `docs/analysis/WATCHDOG-LEDGER.md`,
`PLAN-FACTCHECK-CORE.md`, `PLAN-FACTCHECK-UX.md`, `GAP-AUDIT-1/2.md`,
`GAP-PATCH-1/2.md`, `docs/analysis/CODEBASE-CENSUS.md`,
`docs/analysis/LEAD-RESEARCH-PSEUDOCODE-LANGUAGES.md`.
**Census scope correction (historical pin):** 1,573/1,573 paths indexed; the census feature table rows
sum to 249, while its verdict counts 216 in the eight named families; 23 migrations and
10 prompt assets sit outside that total, and 78 refusals + 4 caps are enumerated but not
counted. Do not repeat an unqualified "216/216 features".
**Authority:** operator-gated evolution only; no autonomous source mutation, no
AGI claim, no unmeasured scale claim.

**Baseline mapping (do not conflate):** `6a41250` = audit/census pin; `73e5fad` = historical v3
execution baseline; `806251b` = installed product commit; `014d0d7` = pre-expansion tag; `b0e60c8`
= Revision 3.1 plan commit. A receipt or claim must name which baseline it belongs to.

## 0. MANDATORY FIRST — capability expansion

This section is the first work order. No remaining v3 hardening WP may start until
CAP-0..CAP-11 are complete, unless a CAP depends on a specific v3 fix
(then that fix is embedded in the CAP, not deferred). CAP-12 is the **final gate/build/bug-fix**
phase after all capability and residual coding. The full detailed annex is
[`HIVEMIND-CAPABILITY-EXPANSION-PLAN.md`](HIVEMIND-CAPABILITY-EXPANSION-PLAN.md); this
section is binding and wins if they disagree.

### 0.1 Product contract (non-negotiable)

- **One HiveMind.** One engine, one run model, one canvas, one brand, one installer.
- **The HMAS planner/root/SuperAgent/Agent/Subagent flow remains the orchestrator for all work.**
- **Coding is an added capability of the existing agents**, not a separate coding product,
  new primary navigation mode, or replacement IDE shell.
- Existing HiveMind behavior remains the default; new panes/tools are additive.
- **Full coding capability is a remembered permission policy**, not a product-mode switch.
- No current command, tool, panel, route, table, approval semantic or installer path is removed
  or renamed. Schema changes are additive.
- No autonomous code mutation; no AGI/HiveWire/OS-sandbox claim; no “better than X” or live/40-node
  claim without a receipt.

### 0.2 Capabilities to add

**A. Coding tools for existing agents.** Add, without removing the current 12 tools:
`fs_glob`, `fs_grep`, `file_read_range`, `repo_map`, `code_search`, `@file`;
`apply_patch` (atomic unified diff), retained `file_write`, `file_delete`/`file_create`;
`run_command`, `run_tests`, `job_output`, `job_kill`;
`git_status`, `git_diff`, `git_log`, `git_stage`, `git_commit`;
`todo_write`, `todo_read`, `update_plan`, `memory_search`.
Terminal/PTY is exposed through the native UI; the model uses governed command/terminal tools.
SuperAgents/Agents receive coding tools; Subagents get read-only context/review tools.

**B. Repo context.** Local index; ignore `.git`, `target`, `node_modules`, binaries and files
>1 MiB; chunks; SQLite FTS5; glob/grep/read-range; repo map; bounded context; explicit truncation;
no network.

**C. Workbench panes inside the existing UI.** File tree/git status; editor; diff accept/reject;
terminal; tool cards; per-agent timeline; plan/todo; reasoning; sessions; checkpoints; jobs;
cost/token bar. No new primary nav mode and no replacement canvas/nav/splash/templates; keep the
FigCraft/HoneyGlass identity and fix current truth defects (socket-only CONNECTED, 375px hidden
library, inaccessible disabled-template reasons, splash skip, header focus).

**D. Reasoning and attribution.** Display provider-returned reasoning labelled as model reasoning,
not hidden chain-of-thought. Show what HyperAgent decided, what SuperAgent delegated, what Agent
executed, what tools returned and what evidence verified. Store bounded reasoning per turn; never
invent unreturned reasoning.

**E. Sessions, checkpoints, jobs.** Named sessions over workspace roots; resume/switch/archive;
old runs readable. Content-addressed file checkpoints before mutation; revert by run/sequence;
hash drift refuses restore. Background jobs with output/kill/cleanup; one active run at a time
initially, queued sessions.

**F. Full coding permission policy (not a mode).**

| Policy | Behavior |
|---|---|
| `classic` | Current approvals: every Mutate/Execute/Network asks; existing semantics preserved. |
| `plan` | Read-only; no writes/commands. |
| `standard` | Workspace edits auto-approved; shell/network ask. |
| `trusted` | Full local coding: workspace edit/command/test/terminal without per-action prompts; outside-workspace/external side effects refused or asked. |

Default for **all new runs**: `classic` — the current approval behavior is preserved. `trusted` is
opt-in, explicitly selected and remembered per session; it is not inferred from “this is a coding
run.” Existing runs keep their recorded policy. No policy bypasses the jail, caps, ledger or the
rule that nothing outside the workspace is silently touched. No OS-sandbox claim.

**G. Skills / CLI / MCP.** Local skill files as labelled prompt data; no remote marketplace.
`hivemind run --json`, status/resume/revert/approve/tail; token from env/DPAPI token file; no
secret logging. MCP stdio server with tools/resources under the same policies; local only by
default. Extension manifest for local tools/panes under the same jail/policy.

### 0.3 How coding uses the existing HMAS

```text
prompt → planner (one or several branches) → root control → SuperAgents → Agents
       → repo search/read → patch → tests → repair → report → root/monitor → final evidence
```

No separate coding orchestrator. Higher caps apply only to coding-capable task types.

### 0.4 What must not change

- Existing HMAS run path and roles.
- Canvas, nav, splash, 8-template library and existing panels.
- Approvals TTL/nonce/single-use/suspension, jail, ledger and evidence.
- SQLite v24 data, old runs, history snapshots, memory banks, proposals/lessons.
- Remotes/daemons, installer, DPAPI key path.
- Brand tokens, typography and HoneyGlass identity.
- No removal/rename of commands, tools, routes, panels or schema columns.

### 0.5 Full coding acceptance (first proof)

A keyed user can, from current HiveMind: open a workspace and start a normal HiveMind run;
inspect the repo with search/read/context; see plan/todo and per-agent reasoning/actions; receive
a patch/diff and accept/reject; under `trusted`, edit code, run commands/tests, repair and see
evidence; checkpoint/undo and resume after restart; get a final answer citing files/tests/tool
evidence; and keep using the same canvas/hierarchy/brand.

### 0.6 Mandatory order and preservation

Capability work is first, then residual v3 hardening. CAP-0..CAP-11 and all residual coding must be
complete before CAP-12's gate/build/bug-fix cycle. Preservation gates in §6 must pass before merge:
existing tests unchanged; real v24 database readback; all current commands/tools/routes/panels
remain; differential HMAS behavior; `classic` equals current approvals; old package installable;
independent audit. If a gate fails, do not merge or replace; the current HiveMind remains.

## 1. Freeze ledger

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

## 2. Truth constraints

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

## 3. Capability work packages (mandatory, first)

These WPs are the first coding work. They are additive; they must not remove or rename current
surfaces. Each WP ends with targeted tests and a commit. Integration suites may run during CAP-11;
no installer build, release battery or release gate until all capability and residual coding is
complete. The current 12 tools/5 classes/35 grants remain a baseline subset; capability tools are
new ToolIds/grants, and the final census is derived by a test, not hardcoded.

### CAP-0 — Preservation baseline
- **Behavior:** archive current v3; tag/branch; verify source snapshot/bundle/GitHub preservation;
  no current file deleted.
- **Files:** `docs/plans/**`, `docs/evidence/**`; no product code.
- **Tests:** docs-truth; checksum/restore; no deletion proof.

### CAP-1 — One-product contract, policies, schema
- **Behavior:** add permission policy (`classic|plan|standard|trusted`); no product-mode field;
  v25 sessions/todos; additive contract rows.
- **Files:** `hiverouter-contract/**`, `db.rs`, `store.rs`, `execution.rs`, `approval*.rs`, `tool_loop.rs`.
- **Schema/API:** v25 `sessions`, `sessions_events`, `todos`; `runs.session_id`,
  `runs.permission_policy`; policy on run. No `runs.mode`.
- **Tests:** v24 readback; policy matrix; migration roundtrip; no product mode.

### CAP-2 — Repo index and context
- **Files:** new `repo_index.rs`; `code_search`/`repo_map`; `provider_executor` context assembly.
- **Schema:** v26 `repo_files`, `repo_chunks`, `repo_fts`.
- **Behavior:** local FTS5 chunks; ignore `.git`/`target`/`node_modules`/binaries/>1 MiB; glob/grep/
  read-range; `@file`; bounded context; explicit truncation; no network.
- **Dependency gate:** prove the bundled SQLite supports FTS5 with a real
  `CREATE VIRTUAL TABLE … USING fts5` test before relying on it; if absent, add the required
  `rusqlite` feature and lockfile change. No silent lexical fallback in the capability itself.
- **Tests:** FTS5 probe, jail, binary/large refusal, injection-safe FTS, deterministic index.

### CAP-3 — Editing and execution tools
- **Files:** `tool.rs`, new `patch.rs`, `fs_search.rs`, `git.rs`, `tests_runner.rs`,
  `tools_bridge.rs`, `execution.rs`.
- **Behavior:** atomic multi-hunk patch; failed-hunk refusal; expected-sha; file journal;
  create/delete; git status/diff/log/stage/commit; run command/tests; output caps.
- **Tests:** apply/reject/conflict byte identity; jail/symlink; kill-tree/timeout; no unapproved
  writes in `plan`/`classic`.

### CAP-4 — Workbench panes inside existing UI
- **Files:** new `web/src/workbench/{FileTree,Editor,DiffView,Terminal,Timeline,TodoPanel,
  SessionRail,CostBar,CommandPalette}.tsx`; update `App.tsx`, state, strings/styles.
- **Dependency gate:** pin exact CodeMirror 6 and xterm.js versions in `package.json` and
  `package-lock.json`; add the backend PTY dependency and a Windows ConPTY/fallback spike. No
  unversioned import and no terminal/editor claim without the dependency/build proof.
- **Behavior:** panes inside existing Workspace/Data/Agents surfaces; no new primary nav; 1440/375;
  keyboard; diff cards; keep current panels/canvas/brand. `CommandPalette` is un-cut by this
  revision; the old v3 cut is historical and superseded. Add a palette behavior/test and
  accessibility check.
- **Tests:** Vitest/Playwright/contrast/screenshots; legacy nav/panels unchanged; editor and
  terminal dependency/build smoke. Axe is optional only if installed; do not claim an axe run
  without the dependency.

### CAP-5 — Sessions, checkpoints, jobs
- **Files:** new `session.rs`, `checkpoint.rs`, `jobs.rs`; `http.rs`/`ws.rs`.
- **Schema:** v27 checkpoints/file ops, v28 jobs/terminal; content-addressed blobs/file ops/jobs.
  No v25/v26 redefinition.
- **Behavior:** named sessions; resume/switch; exact file restore; drift refusal; background jobs;
  cleanup; one active run at a time initially.
- **Tests:** v24 data intact; restore exact hash; external drift refuses; restart/resume; job kill.

### CAP-6 — Reasoning, timeline, attribution
- **Files:** provider reasoning capture; timeline events; UI timeline/reasoning.
- **Behavior:** display provider-returned reasoning only; per-agent decisions/tools/evidence;
  bounded storage; no invented thoughts.
- **Tests:** reasoning event roundtrip; redaction; timeline ordering.

### CAP-7 — Loop quality
- **Files:** `planner.rs`, `execution.rs`, `provider_executor.rs`, new `compaction.rs`, `todo.rs`.
- **Behavior:** plan steps/acceptance; persistent todo; verify after mutation; retry ≤3;
  compaction preserving evidence; monitor `send_instruction`/`cancel_task`/one deduped repair.
- **Tests:** scripted failure→repair→pass; late mutation refusal; retry cap; compaction evidence.

### CAP-8 — Skills, CLI, MCP, model/cost
- **Files:** CLI, HTTP contract, `mcp.rs`, provider settings, cost meter.
- **Schema:** v29 `skills`, `mcp_servers`, provider/session config.
- **Behavior:** `hivemind run --json`, status/resume/revert/approve/tail; token source; MCP stdio;
  local skills; per-session model picker; no secret logging.
- **Tests:** token never logged; CLI receipt; MCP permission conformance.

### CAP-9 — Learning, eval, optional vector
- **Files:** learning/eval/memory modules.
- **Schema:** v30 lessons/acceptance, v31 eval suites/runs/trials/metrics, v32 optional memory
  vectors/embedding jobs.
- **Behavior:** trace manifest; candidate lessons; executed baseline/candidate eval; activation/
  rollback; FTS + optional local embeddings; no autonomous code mutation.
- **Tests:** shape-only receipt refused; executed eval; non-inferiority; rollback; scope/trust.

### CAP-10 — Containment for new tools
- **Files:** jail, shell, approval, policy, db-path, e2e guard, history.
- **Behavior:** `db_path`/`+wal`/`+shm` deny; trusted policy bounded to workspace/external rules;
  diff/preview hash; no OS-sandbox claim.
- **Tests:** negative/refusal tests; no unapproved writes; e2e fail-closed; old approval tests.

### CAP-11 — Integration/preservation
- **Behavior:** all capability + legacy tests; old data fixture; UI parity; additive contracts.
- **Tests:** full backend/web/e2e; v24 migration; differential old HMAS.

### CAP-12 — Final gate/build/bug-fix/live
- **Behavior:** only after CAP-0..CAP-11 and all residual v3 coding (R38–R43). Sequence: full
  integration battery/build/e2e → focused bug-fix loop → clean install/crash/scale → keyed live
  coding → benchmark → final docs. No new features during bug-fix.
- **Tests:** focused suites green; receipts; strict G7 expected RED as PRE-RELEASE; no green-seal
  claim without clean evidence.

**Dependency order:** CAP-0 → 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10 → 11 → 12.
CAP-4/CAP-6 may start after CAP-3 against fixture APIs.

## 4. Existing v3 work packages

> **Revision 3.1 precedence note:** these are the historical v3 work packages, retained with their
> original statuses. Where any line here conflicts with §0 or §3 (for example an old cut of a
> command palette or mission surface), the capability plan wins. Existing done items stay closed;
> only residual items move to R38–R43. No old work package is deleted.
>
> **Preservation exceptions, explicitly enumerated:** (a) keep all 35 baseline grants; do not
> adopt WP-SEC-3's dead-grant removal, its “leave ToolId at 12” rule, or its `census 35→28`
> assertion — capability tools add new ToolIds and the final census is derived by tests; the
> effective tier∩persona surface must be proven unchanged; (b) the only permitted test deletion is
> the dead `handshake.rs`/`tests/handshake.rs` path, replaced by a no-export/absence assertion; all
> other tests remain. No other exception is authorized.

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

Decision (historical; superseded by §0/§4 preservation exception): **isolate** the root
control vocabulary as a bounded validated control surface, not a registry tier. Add
schemas/validation/audit for its six tools and remove the false tier claim. **Do not** remove
the 35 baseline grants or change the `ToolId` count in this revision; capability tools are new
ToolIds, and the final census is derived by tests. Keep the seven unreachable grants documented
as unreachable. Reconcile `finish`/`report_result`: add task-level tests proving `finish`
completes, `report_result success=false` labels failure, and no extra provider call.

Acceptance: root-control schema validation/refusal tests; task-level terminal tests; baseline
12/5/35 unchanged; derived post-expansion census test. Rollback: revert; no schema change.

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
new mission API/entities. They require operator ceiling approval and new backend routes and are a
follow-up to the historical 40-round baseline; the 52-round execution in §5 supersedes that
schedule. The command palette is **not** cut by Revision 3.1 (CAP-4 owns it).

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

Required: current HEAD/version/counts; baseline 12/5/35 tool truth plus a **derived**
post-expansion census (no hardcoded final count); implemented votes/UI vs
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

## 5. Execution order and rounds — capability first, 52 rounds

Rounds are checkpoints, not promises. Capability work is mandatory first. No installer build,
release battery or release gate until all capability and residual coding (R2–R43) is complete; the
CAP-11 integration suite is not the release battery. Targeted
compile/typecheck/unit tests are allowed during coding; they are not gate claims.

| Rounds | Work | Exit |
|---|---|---|
| R1 | CAP-0 preservation baseline/tag/archive; no code changes | preserved baseline; docs-truth |
| R2–R3 | CAP-1 one-product policy/contracts/v25 schema | policy matrix; v24 readback; no `runs.mode` |
| R4–R5 | CAP-2 repo index/context | jail/index/FTS tests |
| R6–R9 | CAP-3 coding/edit/git/test tools | patch/jail/kill-tree tests |
| R10–R14 | CAP-4 workbench panes inside existing UI | Vitest/Playwright/contrast/1440/375; old UI intact; axe only if installed |
| R15–R17 | CAP-5 sessions/checkpoints/jobs | restore/drift/resume/job tests |
| R18–R19 | CAP-6 reasoning/timeline/attribution | reasoning/timeline tests |
| R20–R23 | CAP-7 plan/todo/verify/retry/compaction/monitor actions | scripted provider tests |
| R24–R26 | CAP-8 skills/CLI/MCP/model/cost | token/MCP/CLI receipts |
| R27–R30 | CAP-9 learning/eval/optional vector | executed eval/rollback/scope tests |
| R31–R33 | CAP-10 containment/hardening | negative/refusal/e2e tests |
| R34–R37 | CAP-11 integration/preservation | full legacy + capability tests; v24 migration |
| R38–R43 | residual existing v3 WPs only (done items skipped; no rework) | residual tests; no feature removal |
| R44 | **CAP-12 gate begins:** full backend/web/docs/QA battery, build, e2e, install smoke | focused suites green; strict G7 expected RED (PRE-RELEASE); bug-fix mode only |
| R45–R47 | bug-fix/gate reruns only; no new features | focused suites green; PRE-RELEASE evidence |
| R48–R49 | clean-profile install, keyed live coding, crash/scale | receipts |
| R50 | benchmark vs external harnesses; no superiority claim without receipt | benchmark receipt |
| R51–R52 | final docs/evidence/version push and handoff | final receipt |

**Round size:** each round is 2–3 engineer-days; 52 rounds ≈ 104–156 engineer-days. This is a
checkpoint schedule, not 52 calendar days. If residual work exceeds R38–R43, extend the schedule and
the goal; do not compress or silently drop it.

If R44 exposes a real feature gap, reopen coding and repeat the gate; do not hide it by weakening
tests. The strict release gate remains expected RED at G7 because the sealed manifest is 1.1.0;
this plan does **not** claim a green strict seal. A separate 1.3/2.0 seal is required before any
release claim.

## 6. Capability acceptance and preservation gates

### Acceptance
1. Existing HMAS run works end-to-end; canvas, history, approvals, memory and old data intact.
2. Keyed full coding loop: workspace → plan/todo → repo search/read → patch/diff → accept/trusted →
   test → repair → final evidence.
3. `trusted` allows full local coding without per-action prompts; `classic` preserves current
   approvals; no product-mode switch.
4. SuperAgents/Agents implement; Subagents read/review/verify; all through the existing hierarchy.
5. Sessions/checkpoints/revert/resume work; old runs readable.
6. UI additions are panes in existing surfaces; no clone shell; 1440/375.
7. Reasoning/attribution shows provider-returned reasoning and observable actions only.
8. Learning/eval gated; no autonomous code mutation.
9. CLI/MCP/skills work under the same permissions.
10. No AGI/HiveWire/OS-sandbox/40-node/superiority claim without evidence.

### Preservation gates
- All existing backend/web/e2e tests pass; the only permitted deletion is the dead
  `handshake.rs`/`tests/handshake.rs` path, replaced by an absence assertion. No test weakening,
  skip or silent removal.
- Real v24 database fixture migrates and reads old runs/approvals/history/memory.
- All current commands/tools/routes/panels remain; new additions are additive. All 35 grants remain
  (7 marked unreachable); no dead-grant removal in this plan.
- Differential HMAS behavior; `classic` equals current approvals.
- Old package remains installable; backup/down migration tested; sequential v24→v32 migration test
  after each CAP.
- Independent audit; no merge on failure.

## 7. Agent structure after plan approval

**Capability ownership (Revision 3.1):**
- `impl-core`: `contract/**`, `db.rs`, `store.rs`, `execution.rs`, `provider_executor.rs`,
  `tool_loop.rs`, `approval*.rs`, new session/checkpoint/jobs/learning modules; never edits
  `web/**` or `tools/**`.
- `impl-tools`: `hiverouter-tools/**` and new fs/patch/git/tests modules; never edits
  `hiverouter-core/**` or `web/**`.
- `impl-ui`: `web/src/**`, `web/tests/**`; never edits Rust.
- Lead owns integration, `http.rs`, `ws.rs`, `main.rs`, migrations ordering, docs/gates and
  commits; no concurrent writers on one file.
- CAP-1..CAP-12 follow this ownership; cross-scope changes are serialized by the Lead.

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

## 8. Goal and handoff

After the two plan fact-checkers return and the Lead applies their exact corrections,
create a fresh **52-round goal** with this plan as the execution objective. The capability
phases are first. If R52 still has a named residual, create a follow-up goal only for that
residual; do not silently reset or mark complete. The blocked 80-round goal from the prior
session is replaced, not resumed.

## 9. Acceptance / refusal rules

- No "fixed" claim without a RED test or a reproduced failure and a GREEN re-run.
- No "installed" claim without the installed shortcut opening the current build and a
  matching `/health.version` + screenshot.
- No "gate green" claim on a dirty/mixed HEAD; no skipped test counted as pass.
- No source mutation by an external playground; no key in output.
- No AGI/autonomy claim: the operator remains the fifth seat; code self-modification
  remains unimplemented.
- A blocked WP is reported as BLOCKED with the exact missing capability; it is not
  replaced by a mock.

## 10. Residual items that stay out of core

- real remote hardware/network satellite transport (current remotes are fixtures);
- real-time multi-user/web deployment;
- arbitrary 300–600-agent scale claim;
- autonomous code self-evolution/evaluation;
- external playgrounds as product dependencies (optional connector only);
- any unmeasured "learning gain" claim; lesson retrieval remains L1 mechanism only.
