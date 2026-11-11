# HiveMind Capability Expansion Plan — same HMAS, full coding capability

**Status:** **Annex A** to [HIVEMIND-FINISH-PLAN-v3.1](HIVEMIND-FINISH-PLAN-v3.md). The v3.1 plan
is authoritative; this annex provides the detailed capability reference. It supersedes the earlier
dual-mode/Code-mode framing, but it is not a competing plan.
**Goal:** keep HiveMind exactly what it is — HMAS planner/root/SuperAgent/Agent/Subagent, canvas,
approvals, ledger, evidence, SQLite data, memory banks, proposals/lessons, remotes/daemons,
8-template library, installer, FigCraft/HoneyGlass identity — and **add full coding capability to
the existing agents and UI**. No second product, no mode switch, no visual replacement.

**Truth baseline:** product commit `806251b`; docs/audit commit `44e0535`; current branch HEAD
`014d0d7`; installed `1.2.0-dev`, schema 24, health `degraded`/`E_KEY_MISSING`; docs truth 19/19;
latest offline battery 8/8 PASS on a docs-dirty tree. No live provider claim.

---

## 1. The contract

1. **One HiveMind.** One engine, one run model, one canvas, one brand, one installer.
2. **The HMAS remains the orchestrator.** The planner and root still plan and delegate; coding tasks
   are executed by the existing SuperAgent/Agent/Subagent roles with new tools. A simple task may
   use a one-branch plan; multi-file work may use several agents for implementation/review/verify.
3. **Coding is a capability of the existing product**, not a separate harness mode.
4. **Additive only.** No removal or rename of current panels, commands, events, tools, grants,
   refusal codes, data tables, routes, or installer paths. New schema is additive.
5. **Current behavior remains available.** Existing Team/HMAS runs, approvals, history, canvas and
   old data keep working. New tools and UI panes are added alongside them.
6. **Full capability coding** is supported through one explicit policy toggle; current approval
   behavior remains a selectable policy.
7. **Learning stays operator-gated.** No autonomous code mutation; lessons remain prompt data after
   the evidence gate.
8. **No false claims.** No AGI, no HiveWire, no OS-sandbox claim, no “better than X” without measured
   receipts, no live/40-node claim without a keyed receipt.

---

## 2. Product shape

```text
Same HiveMind run
  planner → root control → HMAS tasks → monitor → final
        │
        ├─ existing tools + canvas + approvals + history + memory + brand
        └─ new capability tools:
             repo read/search/context
             apply patch / write / git / tests / command / terminal
             plan/todo/verify/retry
             sessions/checkpoints/background jobs
             reasoning + per-agent timeline
             skills/CLI/MCP
```

No `Code` mode is required. The workspace UI gains panes/surfaces; the current panels remain.
Coding runs are ordinary HiveMind runs, planned by the same roles.

---

## 3. Capability packs

### A. Coding tools for existing agents

Add, without removing the current 12 tools:

- **Repo read:** `fs_glob`, `fs_grep`, `file_read_range`, `repo_map`, `code_search`, `@file`.
- **Edit:** `apply_patch` (atomic unified diff), retained `file_write`, `file_delete` (explicit
  destructive policy), `file_create`.
- **Execution:** `run_command` (allowlisted programs, cwd continuity, output cap, timeout),
  `run_tests`, `job_output`, `job_kill`.
- **Git:** `git_status`, `git_diff`, `git_log`, `git_stage`, `git_commit`.
- **State:** `todo_write`, `todo_read`, `update_plan`, memory search.
- **Terminal:** PTY/ConPTY exposed through native UI (model uses `run_command`/terminal tools under
  policy, not an unchecked raw pipe).

Tools are granted through the current tier/persona system. SuperAgents/Agents receive the coding
tools appropriate to their role; Subagents get read-only context/review tools. Existing tools and
grants remain.

### B. Repo context

Local repo index: ignore `.git`, `target`, `node_modules`, binaries, >1 MiB; chunk files; SQLite FTS5;
glob/grep/read-range; repo map; bounded context; explicit truncation. No network. No vector memory
required for this pack; it is added later.

### C. Workbench panes (inside existing UI)

Add panes under the existing Workspace/Data/Agents surfaces:

- file tree with git status;
- editor with syntax highlighting (CodeMirror);
- diff review with accept/reject;
- terminal panel (xterm.js + PTY);
- tool-call cards and a per-agent timeline;
- plan/todo;
- reasoning stream/summary;
- session list, checkpoint list, job list;
- cost/token bar.

The canvas, navigation, splash, templates, panels and HoneyGlass identity stay. New panes use the
same brand tokens; no generic IDE skin.

### D. Reasoning and attribution

- Display reasoning when the provider returns it; label it as model reasoning, not hidden
  chain-of-thought.
- Per task/agent: what the HyperAgent decided, what a SuperAgent delegated, what an Agent executed,
  what a tool returned, what evidence verified it.
- Store bounded reasoning per turn and show it in the timeline; never fabricate a thought the
  provider did not return.

### E. Sessions, checkpoints, jobs

- Named sessions over workspace roots; resume, switch, archive; old runs stay readable.
- File checkpoint before every mutation; content-addressed blobs; revert by run/sequence; hash drift
  refuses restore.
- Background job table for long commands; output tail/kill; cleanup on session end.
- These are additions, not a new run scheduler. One active run at a time initially; queue multiple
  sessions.

### F. Full coding policy

Policies are additive and explicit:

| Policy | Behavior |
|---|---|
| `classic` | Current approvals: every Mutate/Execute/Network asks; existing semantics preserved. |
| `plan` | Read-only; no writes/commands. |
| `standard` | Workspace edits auto-approved; shell/network ask. |
| `trusted` | Full local coding: workspace edit/command/test/terminal without per-action prompts; outside-workspace/external side effects still refused or asked. |

**Default for new coding runs:** `trusted` is selectable and remembered; `classic` remains available
and is the behavior of existing runs. No policy bypasses the jail, caps, ledger, or the hard rule that
nothing outside the workspace is silently touched. The app never claims an OS sandbox.

### G. Skills, CLI, MCP

- Local skill files loaded as labelled prompt data; no remote marketplace.
- CLI `hivemind run --json`, status/resume/revert/approve/tail; token from environment/DPAPI token
  file; no secret logging.
- MCP stdio server exposing tools/resources with the same policy classes; local only by default.
- Extension manifest for local tools/panels; same jail and policy.

---

## 4. HMAS usage for coding

The same hierarchy handles coding:

```text
prompt: "fix failing test X"
planner → one branch lead (simple) or multiple branches (multi-file)
root control → assign objectives
SuperAgent → spawn one Agent (or reviewers)
Agent → repo search → read → patch → tests → repair → report
root/monitor → final answer with diff/test evidence
```

No separate coding orchestrator. The existing root/monitor still runs; coding tools are available in
the task tool loop. Higher caps are applied to coding-capable task types, not to the whole Team mode
unless configured.

---

## 5. Additive schema

| Version | Addition |
|---|---|
| v25 | `sessions`, `sessions_events`, `todos`; nullable `runs.session_id`, `mode` |
| v26 | `repo_files`, `repo_chunks`, `repo_fts` |
| v27 | `file_ops`, `checkpoint_blobs`, `checkpoints` |
| v28 | `jobs`, `terminal_sessions`, `terminal_chunks` |
| v29 | `lessons`, `lesson_evidence`, `acceptance_runs` |
| v30 | eval tables (future pack) |
| v31 | optional memory vectors (future pack) |
| v32 | `skills`, `mcp_servers`, provider/session config |

No table/column is dropped; v24 remains readable. Every migration has up/down and a v24→latest
roundtrip with backup/restore.

---

## 6. UI plan

- Keep current shell/nav/canvas/templates/splash.
- Add **Files**, **Diff**, **Terminal**, **Timeline**, **Plan/Todo**, **Sessions**, **Jobs** and
  **Cost** panes inside the existing workspace/data areas.
- Add a **Reasoning** disclosure and per-agent thought/action timeline.
- Add a **policy** selector (`classic`/`plan`/`standard`/`trusted`); persist per session.
- Add diff approval card with file/path/hunk and accept/reject.
- Fix current truth defects: socket-only CONNECTED, 375px hidden library, inaccessible disabled
  template reasons, splash skip, header popover focus.
- Preserve the 10–20% polish rule for existing surfaces; new panes use brand tokens and 1440/375
  layouts.

---

## 7. Work phases

| Phase | Work | Coding outcome |
|---|---|---|
| P0 | Baseline tag/bundle/snapshot; additive contracts; session/tool schema skeleton | no behavior change |
| P1 | Repo index + read/search/glob/grep/read-range | agents can inspect any repo file |
| P2 | Patch/write/delete/create + diff preview + journal | agents can edit code safely/atomically |
| P3 | Git + run_command + run_tests + jobs | agents can build/test/commit |
| P4 | Sessions, checkpoints, revert, terminal, policy modes | full coding loop, resumable |
| P5 | Planner/todo/verify/retry/compaction + monitor actions | multi-step coding quality |
| P6 | Reasoning/timeline/tool cards/per-agent attribution | visible thinking/decisions |
| P7 | Workbench UI panes + 1440/375 + command palette | usable coding UI |
| P8 | Skills/CLI/MCP; costs; docs | harness parity surfaces |
| P9 | Learning/EVO v2/optional vector memory | measured improvement loop |
| P10 | Full integration gate, build, bug-fix, keyed live, benchmark | release candidate |

**Round rule:** no installer build, full battery, or release gate until all coding phases (P0–P9)
are complete. Per-round compile/typecheck/unit tests are allowed to keep coding unblocked. Then one
gate/build/bug-fix cycle; no new features during bug-fix.

**Estimate:** full capability expansion is ~70–100 engineer-days; the minimum usable coding
capability (P1–P8 without learning/MCP) is ~45–60 days. The plan is large because “full access
coding” is large.

---

## 8. Preservation gates (must pass before merge/build)

1. Existing backend/web/e2e tests pass unchanged.
2. Old v24 database fixture: all old runs/tasks/messages/approvals/history/memory readable after
   migration.
3. All current commands/tools/panels/routes remain; new additions are additive.
4. Team/HMAS behavior differential checks: same prompts produce equivalent event/approval/snapshot
   semantics, aside from additive tools/reasoning.
5. Policy test: `classic` remains current approval behavior; `trusted` only affects new coding runs.
6. Old package remains installable; data backup before migration; down migration tested.
7. Independent audit and no test weakening.

**If any gate fails: do not build/replace; fix or revert. The existing HiveMind remains.**

---

## 9. Acceptance: full coding capability

A user with a DeepSeek key can, using the current HiveMind UI and HMAS:

1. Open a workspace and start a normal HiveMind run.
2. Have the planner/root/agents inspect the repo with glob/grep/read-range/repo map.
3. See a plan/todo and per-agent reasoning/actions.
4. Receive a patch/diff proposal; accept or reject it.
5. With `trusted`, edit code, run commands/tests, iterate repair, and see evidence.
6. Use checkpoints/undo, resume after restart, and inspect cost.
7. Get a final answer citing files, tests and tool evidence.
8. Use the same canvas/hierarchy/brand as before.

No separate mode, no clone UI, no removal of the current workflow.

---

## 10. Non-goals / honesty boundaries

- No AGI/Gödel/autonomy claim.
- No HiveWire; it is not in this repo.
- No OS sandbox claim. `trusted` is full local capability, not containment.
- No live 40-node or provider-proof claim without a keyed receipt.
- No “better than Hermes/DeepSeek Harness/Claude Code” claim without a measured benchmark.
- No autonomous code mutation or unattended self-improvement.
