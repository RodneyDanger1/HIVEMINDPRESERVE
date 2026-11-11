# CODEBASE-AGENT-TRUTH-v4 — agent-level truth map (current tree)

**Scope:** HiveMind at `C:\Users\Riley\Desktop\hiverouter-v1`, HEAD `848735a`, plus the
dirty worktree (22 modified files, 3 untracked; `git status`). Read-only audit; this file
is the only write. Labels: **VERIFIED** = code read at cited lines; **READ** = doc/comment
read, not executed; **UNVERIFIED** = plausible but not proven here; **PROPOSED** = design
suggestion, not implemented.

**Worktree state (VERIFIED by `git status` / `git diff --stat`):** core `execution.rs`
(+100), `provider_executor.rs` (+38), `daemon.rs` (+36), `tools_bridge.rs` (+564);
tools `approval.rs` (+469), `shell.rs` (+293), `side_effects.rs` (+109), `jail.rs` (+58),
`tool.rs` (+6). These are **UNCOMMITTED** and included below where relevant. HEAD is
`848735a`, not the `6a41250` baseline named in older plans.

---

## 1. L0 — HyperAgent (root task `role="planner"`, depth 0)

```text
# PLANNING  execution.rs:1904-1934, 1937-2052; provider_executor.rs:26-33, 99-129
claim_provider_call(run)                                  # counted against MAX_CALLS=64
plan = executor.plan(run, planning_prompt)                # PLANNER_SYSTEM, JSON-only
if plan invalid schema: plan = executor.repair_plan(...)  # exactly ONE repair, counted
plan.validate()                                           # 1..=40 nodes, acyclic, depth<=3
root = TaskRecord(role="planner", depth=0, parent=None)   # engine-owned
graph root kind="hyperagent"; seed run bank (prompt, plan digest)
for node sorted by depth:                                 # planner DAG inserted as rows
    validate_spawn_role(parent_tier, node.role)           # matrix below, before any row
    caps.check_spawn(depth, siblings, agents)             # refuse, never clamp
root_checkpoint(run, root, plan)                          # control turn, see below
run_state Planning->Start, snapshot                       # commit before execution
execute ready nodes by deps + children-complete readiness

# ROOT CONTROL  execution.rs:4296-4902; provider_executor.rs:568-694
if !root_orchestrator: skip                               # production boot enables it
turn in 0..2:                                             # TWO bounded turns
    claim_provider_call; executor.orchestrate_root(...)   # 1024 max tokens, tools advertised
    take first MAX_ROOT_TOOL_CALLS=4 calls                # excess only warn+ignore
    turn 2 may NOT create agents/remotes/votes            # creation is turn-1 only
    allow: spawn_superagent | assign_task | spawn_remote_agent
           spawn_daemon | read_memory_summary | propose_improvement
           convene_evolution_vote | finish                # convene not in provider tool list
    every allow/refuse -> durable run-bank gate record + remote audit

# FINAL  execution.rs:2194-2280, 4271-4291; provider_executor.rs:696-721
if has_aggregators || plan.nodes.len()>1:
    repair root to Running; digest all child results (<=400 chars each, 6KB digest)
    root_prompt budget 24,000 chars, per-result 2,000; omitted count disclosed
    executor.finalize_root(...)                           # answer-only: NO tools advertised
```

**Current limitations (VERIFIED unless marked):**
- `has_planned_branch` is true for every valid plan (all plans have a depth-1 node), so the
  advertised `spawn_superagent` allow-branch is unreachable; the tool is advertised but
  dead (`execution.rs:4328, 4397-4408`). This is an overstated capability in the UI/tool list.
- `convene_evolution_vote` appears in the control prompt (`execution.rs:4318`) and has a
  handler (`:4718-4902`), but is **not** in the 7 `ToolSpec`s sent to the provider
  (`provider_executor.rs:584-689`). A compliant provider cannot emit it; UNVERIFIED that a
  live provider can reach the vote-convening path.
- `ROOT_SYSTEM` tells the final root it "may call spawn_remote_agent at most once"
  (`provider_executor.rs:49-51`), but `finalize_root` advertises no tools (`:696-721`).
  The final turn is answer-only in code; the prompt overstates it.
- Root control runs before the run's Start transition/snapshot (`execution.rs:2054-2071`);
  a crash during control has no post-control snapshot until Start.
- Root is not a continuous governor: two turns, four calls, no async observation, no
  lower-level request adjudication (lower level is deterministic, `:5008-5240`).

## 2. L1 — SuperAgent (depth 1 branch lead)

```text
spawned only by planner or root control: role in {super, superagent, super_agent,
  sub_hive, subhive} (execution.rs:248-251, 290-292)
tier = SuperAgent; prompt = persona prompt or TASK_SYSTEM (provider_executor.rs:180-201)
tools = allowlist 12: spawn, ask_parent, send_to_sibling, report_result, finish,
  file_read, file_list, file_write, shell_exec, request_detached, web_fetch,
  vote_evolution (tool.rs:467-483)
may spawn only Agent/role aliases {worker, agent, content-writer, data-analyst,
  code-reviewer, chatbot}; never Subagent at depth 2 (`execution.rs:252-266`)
may request one detached principal (deterministic gate, not a model turn)
may vote_evolution only on an existing proposal in this run; duplicate voter refused;
  payload applied=false (`execution.rs:4906-5006`)
own hive bank + branch context; tools side effects still approval-gated
```

**Limitations:** allowlist grants authority, not availability — every Mutate/Execute/
Network call still needs a single-use operator approval (`approval.rs:918-1120`). Shell
is arbitrary user-level execution on approved programs, explicitly not a sandbox
(`tool.rs:433-439`; `shell.rs:7-16`). No independent scheduler; it is a task row.

## 3. L2 — Worker / Agent (depth 2)

```text
spawned by SuperAgent only; role aliases worker/workeragent/worker_agent/agent or a
  named persona (`execution.rs:252-265`)
tools = allowlist 11 (same as SuperAgent minus vote_evolution) (`tool.rs:484-499`)
may spawn Subagent only (`execution.rs:266`); task instruction is prefixed to the
  run prompt, never a capability grant (`execution.rs:4229-4236`)
may request detached; may web_fetch; file_write/shell_exec approval-gated
```

**Limitations:** a Worker cannot escalate itself to SuperAgent (spawn matrix + class
check, `execution.rs:1563-1575`); an unrecognized role maps to WorkerAgent, not a broader
tier (`:297`). Depth/fan-out/agents caps still apply (`caps.rs:52-62`).

## 4. L3 — Subagent (depth 3 leaf)

```text
role must be a subagent alias; refused at depth < 2 and by any non-Worker parent
  (`execution.rs:246, 266, 2020-2022, 1593-1595`)
tools = 5 read-only/coordination: ask_parent, report_result, finish, file_read,
  file_list (`tool.rs:500-509`); cannot spawn, write, shell, fetch, request detached
bounded A2A inbox refresh and parent escalation answer; terminal leaf
```

**Limitations:** no file write/shell/network by construction; it still consumes provider
turns and can ask the parent, but the parent answer is information, not approval authority
(`provider_executor.rs:517-535`).

## 5. Detached remotes and daemons (tree-free, not tasks)

```text
# lower-level gate  execution.rs:5013-5240
tier in {SuperAgent, WorkerAgent}; kind in {remote, daemon}; name unique per hive;
reason >=12 chars; run cap 4 detached principals (E_DETACHED_CAP); idempotent request_id
remote -> RemoteStore.create_remote(transport="in_process", endpoint_ref="fixture")
          + fixture device + one narrow grant; audit remote_admission
daemon -> DaemonStore.create(kind, job_json={"check":"db"} for watchdog else "{}")
          starts stopped; audit control_decision
# root-only  execution.rs:5349-5384; provider_executor.rs:617-648
spawn_remote_agent creates the same fixture remote with hyper:<root> attribution
spawn_daemon enrolls a record (no task row, no graph edge)

# supervisor  remote_supervisor.rs:1-101 (single lane, no model)
tick: expire remote leases; expire approvals; daemons.tick_due(64)
daemon job: {"check":"db"} -> real SELECT 1 (daemon.rs:304-318)
            {"kind":"script"} -> .exe/.ps1 under configured script root, sanitized env,
            else degraded E_DAEMON_JOB_UNIMPLEMENTED (daemon.rs:319-331, 333-399+)
```

**Limitations:** remotes are **fixtures**, not network satellites (`execution.rs:5258-5265`);
daemons are DB records ticked by an in-process supervisor, not OS services (`daemon.rs`
`tick_due`; supervisor doc `remote_supervisor.rs:1-6`). Script jobs execute local programs
with the server user's privileges under a configured root; this is a real execution path
whose safety depends on that root and env sanitization (UNCOMMITTED `env_safety` call).

## 6. ToolAgent (registry tier)

```text
AgentTier::ToolAgent exists with allowlist 4: file_read, file_list, file_write,
  shell_exec (`tool.rs:510-518`); label "tool_agent"; cannot spawn (`execution.rs:268`)
No planner role, spawn matrix entry, or root tool creates a ToolAgent task.
UI knows node kind "tool" (`web/src/state/agentState.ts:126-135`).
```

**Status:** a census/registry/UI tier. Whether any durable task row is ever created with
class `tool` is **UNVERIFIED**; the planner prompt allows only superagent/agent/subagent
(`provider_executor.rs:29-33`) and `validate_spawn_role` never accepts "tool". Treat
"ToolAgent is a runtime agent level" as false until a producer is shown.

---

## Capability table

| Level | Can (VERIFIED static) | Cannot | Unverified |
|---|---|---|---|
| HyperAgent | plan once (+1 repair), 2 control turns / 4 calls, spawn SuperAgent, assign objectives *, spawn fixture remote/daemon, read bounded memory, record proposal, final answer | edit code, run tools in final turn, continuously observe, approve lower-level requests live, spawn Agent/Subagent directly, add a branch when plan has one | live vote convening; crash safety before Start |
| SuperAgent | decompose one branch, spawn Agent, read/write/shell/fetch with approval, request detached, vote on proposal | spawn Subagent, self-promote, bypass approval, apply a vote | persona-specific limits beyond default |
| Worker/Agent | execute one task, spawn Subagent, read/write/shell/fetch with approval, request detached | spawn SuperAgent/peer, vote, bypass caps | — |
| Subagent | read/list, ask parent, report, finish | spawn, write, shell, network, detached | — |
| Remote | fixture principal + device + narrow grant, audited actions | be a model, task, or network satellite | real hardware transport |
| Daemon | recorded watchdog; real DB check; optional script job | run as OS service; act without an operator-created record | script-root hardening beyond env sanitization |
| ToolAgent | registry allowlist of 4 tools | spawn; be produced by any verified path | any live producer |

\* `assign_task` exists only in the UNCOMMITTED worktree (`execution.rs:4440-4491`;
`provider_executor.rs` diff). Committed HEAD lacks it.

## Caps, refusals, approvals, memory/A2A, votes, UI

- **Caps (VERIFIED):** depth 3, fan-out 8/parent, 64 agents/run, 64 provider calls/run
  (`caps.rs:9-18`); plan <=40 nodes (`planner.rs:10, 54`); tool loop 12 turns, 40 calls/run,
  8 approval waits (`tool_loop.rs:14-21`); root 2 turns/4 calls/24KB prompt/2KB per result
  (`execution.rs:2217-2218, 4333-4342`); 4 detached principals/run (`:5139`).
- **Refusals:** named codes, never silent clamps: `E_SPAWN_DEPTH`, `E_FANOUT_LIMIT`,
  `E_AGENT_LIMIT`, `E_CALL_LIMIT`, `E_TOOL_NOT_ALLOWED`, `E_DETACHED_CAP`,
  `E_DETACHED_CONFLICT`, `E_APPROVAL_REPLAY` (`caps.rs:52-74`, `refusal.rs`).
- **Approvals:** per-call, args-hash-bound, nonce single-use, TTL; replay refused
  (`approval.rs:6-8, 918-1120`). UNCOMMITTED: `verify_grant_bound` additionally binds
  `approval_id` + `tool_call_id` (`approval.rs` diff; `execution.rs` resume path).
- **Memory/A2A:** run bank seeded pre-dispatch; each task refreshes run + immediate-parent
  banks and recipient inbox at start (`execution.rs:4108-4219`); A2A text bounded
  500 chars / 2KB, marked data-not-instructions (`:4174-4186`).
- **Evolution votes:** only SuperAgent; proposal must exist in run bank; one vote per
  (proposal, voter); stored `applied:false`; five-seat gate = 4 SuperAgent + operator
  (`execution.rs:4906-5006`; `http.rs:2520`). No code application path.
- **UI projections (VERIFIED static):** REST reads at `http.rs:612-694` (`/api/runtime`,
  roster, memory, A2A, remotes, daemons, supervisor, approvals, run detail). Hierarchy is
  rebuilt only from `task.parentId` links; each row may request SPAWN with a visible
  disabled reason (`web/src/panels/AgentHierarchy.tsx:40-131`); 16-state vocabulary
  (`web/src/state/agentState.ts:9-26`). Live rendering is UNVERIFIED here.

## What it is / what it is not

**It is:** [VERIFIED] a deterministic, run-scoped, single-provider multi-agent task
orchestrator: one planner call, a bounded DAG (<=40 nodes), a four-rank task hierarchy
(hyper -> super -> worker -> subagent), a bounded two-turn root controller, approval-gated
tools, durable SQLite memory/A2A/votes, and a tree-free fixture remote/daemon lane.

**It is not:** [VERIFIED negative] an AGI; not autonomous (operator remains the fifth seat
and every side effect is approval-gated); not self-modifying (proposals and votes are data
with `applied:false`; no writer/loader path); not Meta's HyperAgent (no meta-agent or
editable improvement procedure); not a continuous daemon/scheduler; not a network
satellite (remotes are fixtures); not a 300–600-agent system (caps are 64 agents / 64
calls); not a system where the root executes tools in its final turn (no tools advertised).

## Top 5 stale or overstated claims still present in docs/reports

1. **`docs/plans/HIVEMIND-FINISH-PLAN-v3.md:6-10`** — audit baseline `6a41250`, execution
   baseline `abfff55`, tracked count `1,593`. Current HEAD is `848735a` (VERIFIED
   `git rev-parse`); the baseline sentence is stale and the next action ("R1 commits
   `docs/analysis/**`") has long since happened.
2. **`docs/plans/HIVEMIND-FINISH-PLAN-v3.md:24`** — "tree dirty by `docs/analysis/**` and
   this plan." Current tree is dirty in 22 tracked files including six core `.rs`, seven
   tools `.rs`, tests and `tools/*` (VERIFIED `git status`). Any gate claim keyed to this
   freeze line is unsound.
3. **`docs/plans/HIVEMIND-FINISH-PLAN-v3.md:56-57`** — "`cmd.exe` in the shell allowlist
   makes approved shell arbitrary user-level code execution." Current `shell.rs` refuses
   command-interpreter hosts even if explicitly allowlisted and the default allowlist omits
   `cmd.exe` (`shell.rs:48-99, 171-177, 205-215, 292-300`; UNCOMMITTED worktree). The
   residual true part — approved non-interpreter programs are still arbitrary code — is
   stated honestly at `tool.rs:433-439`.
4. **`docs/plans/HIVEMIND-FINISH-PLAN-v3.md:58-59`** — "Approval grants ignore TTL and bind
   only `(run, tool, args_sha256)`." Current `approval.rs` checks `expires_at` on consume
   and adds `approval_id`/`tool_call_id` binding (`approval.rs:933-1051, 1077-1120`;
   UNCOMMITTED `verify_grant_bound`). The claim was likely true at the old baseline; it is
   not true in this tree.
5. **`docs/audit/HYPERAGENT-TRUTH-REPORT.md:118`** — "**10** registered tools, **5**
   permission classes, **32** tier grants." Current registry has **12** tools (`tool.rs:
   115-140`) and **35** grants (3+12+11+5+4, `tool.rs:462-518`). Also stale in the same
   family: `docs/audit/AGENT-TAXONOMY-VERIFICATION.md:735` says "root tools are four fixed
   local actions" while the current root advertises seven local tool specs plus an
   execution-layer `convene_evolution_vote` (`provider_executor.rs:584-690`;
   `execution.rs:4313-4321`).

## Method / residual uncertainty

- Read at HEAD + worktree: `execution.rs` (dispatch, root control/final, spawn, votes,
  detached), `provider_executor.rs` (prompts, tool advertising, root/final), `tool.rs` +
  `tool_loop.rs` (tiers/grants/specs), `caps.rs`, `planner.rs`, `approval.rs`,
  `shell.rs`, `daemon.rs`, `remote_supervisor.rs`, `http.rs` routes, `AgentHierarchy.tsx`,
  `agentState.ts`, plus `git status`/`git diff`.
- **UNVERIFIED here:** live provider behavior, actual UI rendering, test execution, whether
  `convene_evolution_vote` or `spawn_superagent` can ever fire in practice, and any daemon
  script-root configuration present at boot. No server, provider, key, or test was run.
