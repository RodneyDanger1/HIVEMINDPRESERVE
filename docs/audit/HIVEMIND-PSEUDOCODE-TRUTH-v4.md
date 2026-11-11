# HiveMind — current pseudocode truth and paper cross-audit (v4)

**Date:** 2026-11-10 (America/Chicago)
**Tree:** working tree after `848735a` plus the uncommitted security/approval/schema/UI/seat-binding changes.
**Purpose:** describe what HiveMind is and does today in executable-shaped pseudocode, cite the exact code
blocks/lines, and separate code facts from two local research-paper hypotheses. This is not a completion
claim and not a claim of AGI.

**Evidence labels used below**

| Label | Meaning |
|---|---|
| `CODE` | read in this session from the repo at the stated path:line |
| `PAPER` | asserted by one of the two local research texts; not independently verified |
| `UNCOMMITTED` | present in the working tree but not yet in a clean pushed commit |
| `UNVERIFIED` | not observed/tested in this report; the missing test is named |

The two paper inputs are analytical references, not accepted specifications:

- `C:\Users\Riley\Desktop\A Mechanistic Audit of a Capped Hierarchical MultiAgent Framework ...txt`
- `C:\Users\Riley\Downloads\Recursive Multi-Agent Evolutionary Topologies and Automated Synthetic Distillati.md`

## 1. One-paragraph truth

HiveMind is a **local, single-provider, hierarchical multi-agent runtime (HMARS)**: one HyperAgent plans
a DAG and performs a bounded root-control pass, SuperAgents lead branches, Worker/Agent tasks execute,
temporary Subagents do narrow leaves, and detached remotes/daemons live outside the task graph under
narrow grants. It has real refusal/approval/ledger/memory/crash machinery. It is **not** AGI, not an
autonomous self-modifying system, not a network satellite system, not a trained-model/distillation system,
and not a 300–600-agent system. Its strongest property is legible boundedness; its weakest property is
independent verification of its own outputs.

## 2. Runtime pseudocode

```text
SUBMIT(prompt):
  detect key/provider; refuse E_KEY_MISSING / E_RUN_ACTIVE before any row
  create run + root HyperAgent task
  planner = one counted provider call (one bounded schema repair)
  plan.validate: <=40 nodes, depth<=3, DAG, parent-chain, parent!=dep
  insert planned graph
  root_control(turn=0,1):
    turn 0: spawn_superagent(objective) | assign_task(branch,objective) |
            spawn_remote_agent | spawn_daemon | read_memory_summary |
            propose_improvement | finish
    turn 1: finish OR one final decision; creation tools are refused
  dispatch ready tasks:
    ready = deps complete AND direct children terminal
    super -> agent -> subagent; depth/fanout/agent/provider/tool caps pre-checked
  finalize_root: tool-free synthesis over completed results
  persist usage, terminal state, memory, snapshots; settle no running task may survive
```

Root pseudocode, with the important current limitation:

```text
ROOT:
  plan = planner_call(goal)                 # one provider call; optional one repair
  root.control = bounded 2-turn loop         # `for turn in 0..2`, execution.rs
  root.control may create at most one extra super branch, only when the planner
    supplied no depth-1 branch; creation on turn 2 is refused
  dispatch all planned tasks
  final = answer_only_call(no tools)        # finalize_root
  # There is NO post-dispatch monitor/repair turn in the current code.
```

The planner already produces at least one depth-1 branch for a valid hierarchy, so the root's
`spawn_superagent` path is usually refused by the `has_planned_branch` guard; the new `assign_task`
path is the way the root can give a planned branch a concrete objective. This is a real capability,
not an LLM-governance claim.

## 3. Agent-by-agent pseudocode

### 3.1 HyperAgent / root (`role=planner`, depth 0)

```text
HYPER:
  planner_call(goal) -> plan JSON
  create root task
  root_control(turn 0):
     may spawn one extra super, assign a queued branch, create fixture remote/daemon,
     read bounded memory, record proposal, finish
  root_control(turn 1):
     may finish/final decision; create tools refused
  finalize_root:
     no tools advertised; synthesize from completed child results
  CANNOT: self-edit code, approve its own proposal, run continuously,
          monitor/reassign after dispatch, or add a branch when planner supplied one
```

`CODE`: `execution.rs` root control `for turn in 0..2` and `MAX_ROOT_TOOL_CALLS = 4`;
`provider_executor.rs` root `orchestrate_root` schemas and `finalize_root` with no tools.
`UNCOMMITTED`: root schemas now advertise `spawn_superagent` with an objective and `assign_task`.

### 3.2 SuperAgent (`role=super`, depth 1)

```text
SUPER:
  receive branch objective/instruction; read its hive bank
  spawn Agent children (fanout/depth/class checks)
  aggregate direct child results; ask parent / message same-parent siblings
  read/list files; approval-gated write/shell/web; request detached admission
  vote_evolution only if this task is one of the four designated seats for the proposal
  CANNOT: create remotes/daemons directly, self-apply evolution, exceed caps
```

`CODE`: `execution.rs::spawn_sub_hive` class matrix and caps; `tool.rs` SuperAgent allowlist;
`provider_executor.rs` tier∩persona tool advertising. `UNCOMMITTED`: seat binding is now recorded when
the four election tasks are spawned and checked in `execute_vote_evolution`.

### 3.3 Worker / Agent (`role=agent`, depth 2)

```text
AGENT:
  execute assigned branch task with file_read/list/write, shell (approval),
  web_fetch (approval), A2A ask/sibling, request detached, report, finish
  may spawn a temporary Subagent at depth 3
  CANNOT: vote_evolution, create remote/daemon directly, act as root
```

`CODE`: `tool.rs` Worker allowlist; `execution.rs` spawn/depth policy; `TASK_SYSTEM`/tool-loop path.

### 3.4 Subagent (`role=subagent`, depth 3)

```text
SUBAGENT:
  read/list files; ask parent; report; finish
  no spawn, no write, no shell, no detached request, no evolution vote
```

`CODE`: `tool.rs` Subagent allowlist; `execution.rs` depth floor; persona intersection.

### 3.5 Detached remote

```text
REMOTE:
  admitted by root control or lower-level request_detached gate
  in-process fixture principal + device + narrow grant; no model, no task row
  audited create/device/grant/replay; operator can actuate through grant
  NOT a network satellite; no internet; no AI worker
```

`CODE`: `execution.rs` root remote branch and detached admission; `remote.rs`; UI remote lane.

### 3.6 Daemon

```text
DAEMON:
  enrolled record, not a task; supervisor ticks due rows
  watchdog {check:db} probes SQLite; optional .exe/.ps1 script under configured script root
  script child gets sanitized env (UNCOMMITTED fix: no DEEPSEEK_API_KEY inheritance)
  state/report row persisted; no model call, no task-graph membership
  NOT a continuously scheduled OS process with live-tick proof
```

`CODE`: `daemon.rs`; `remote_supervisor.rs`; `daemon_tick_degraded.rs`.
`UNCOMMITTED`: `sanitize_std` before spawn and a test asserting the provider key is absent.

### 3.7 ToolAgent

```text
TOOLAGENT:
  registry/census/UI tier exists; no verified production producer creates one
  its grants are not a callable production surface today
```

`CODE`: `tool.rs` allowlist, census; no planner/spawn arm. This is a label, not a working agent tier.

## 4. Capability matrix (current truth)

| Capability | Status | Evidence |
|---|---|---|
| DAG planning, hierarchy depth 3, fan-out 8, 64 agents | **CODE** | `planner.rs`, `caps.rs`, `execution.rs` |
| Root bounded control (2 turns, 4 calls) | **CODE** | `execution.rs`; `orchestrator_root` schemas |
| Root objective assignment to a planned branch | **UNCOMMITTED CODE** | `assign_task` + `set_task_instruction` |
| Task instruction persistence / prompt injection | **UNCOMMITTED CODE** | v24 `tasks.instruction`; `store.rs`; `execute_task` |
| Root post-dispatch monitor/reassign/cancel | **ABSENT** | no monitor/send_instruction/cancel_task |
| SuperAgent/Agent tools (file/shell/web/A2A) | **PARTIAL** | tier∩persona allowlists; approval gates |
| Approval TTL/identity binding | **UNCOMMITTED CODE** | `approval.rs` bound grant + `execution.rs` bound path |
| Data-dir denial for tools bridge | **UNCOMMITTED CODE** | `ToolsBridge::new_with_data_dir`; jail tests |
| Command interpreter denied | **UNCOMMITTED CODE** | `SHELL_INTERPRETER_HOSTS` |
| Four-voter + operator lesson apply/rollback | **CODE, lesson artifact only** | `http.rs` authorize/apply/rollback; `LESSONS.jsonl` |
| Designated-seat vote binding | **UNCOMMITTED CODE** | seat memory + vote check |
| Held-out independent evaluation of lessons | **ABSENT** | no receipt/evaluator; free-text evidence only |
| Independent critic outside the agent population | **ABSENT** | code-reviewer is still a plannable agent |
| Memory banks/A2A/durable recovery | **CODE** | `store.rs`, `snapshot.rs`, `recovery.rs`, tests |
| Real network remote / live daemon scheduler | **ABSENT** | fixtures/enrolled records only |
| Autonomous code mutation / weight learning | **ABSENT** | plan and code explicitly forbid/do not implement |
| AGI criteria | **NOT SUPPORTED** | no generality/continual-learning/self-modeling evidence |

## 5. Paper cross-audit (paper claim vs code)

| Paper mechanism/claim | Code reality | Verdict | Improvement implication |
|---|---|---|---|
| Two-turn root governor and deterministic detached admission | Root loop and admission gate exist (`execution.rs`) | **PAPER→CODE CONFIRMED** | Keep; add monitor only if capped/tested |
| Five-seat evolution / four designated SuperAgents | Payload says 5; authorizer originally accepted any four completed SuperAgents; seats now bound | **PARTIAL, improved uncommitted** | Test seat/vote refusal; no autonomy claim |
| Memory partitions / no cross-run learning | Banks exist; only lesson file crosses runs; no weights | **CODE CONFIRMED** | Label L1 only; do not claim learning |
| "Self-evolution" without weights | Lessons only; no code/schema/prompt mutation | **CODE CONFIRMED** | EVO-GATE would verify a receipt; not built |
| Held-out evaluator/observer outside agents | No independent evaluator; proposals are operator-gated but evidence is free text | **GAP** | EVO-GATE / sealed acceptance set candidate |
| Population/archive/fitness selection | No fitness function, no score archive, no population | **ABSENT** | Do not invent scores; require ground truth first |
| Distillation to sub-4GB student | No gradients/weights/dataset path | **ABSENT / out of scope** | Reject; do not simulate |
| Reward hacking/diff auditing | Tool ledger has no target class; file writes require approval, but test/CI paths are not specially classified | **PARTIAL GAP** | Evidence-ledger target_class candidate |
| AGI distance | No operational AGI evidence; local capped runtime | **PAPER/INFERRED** | Keep anti-AGI language |

## 6. Changes made in this work pass (and honest status)

The following are in the working tree; only the full backend suite *before* the final seat-binding edit
was green. Re-run the full suite after this report before claiming completion.

1. **Containment:** tool-bridge data-dir/db-path denial; interpreter hosts denied even if explicitly
   allowlisted; daemon child environment sanitized (no provider key inheritance). `UNCOMMITTED`.
2. **Approval integrity:** allow_once TTL/identity binding; ledger failure no longer executable; tests
   added. `UNCOMMITTED`.
3. **Root capability:** `spawn_superagent {objective}` + `assign_task` advertised and handled; task
   instruction stored in v24 and injected into child prompt. `UNCOMMITTED`.
4. **Five-seat truth:** election spawn records `evolution_seat:<proposal>:<voter>`; `vote_evolution`
   refuses a voter that is not one of those seats. `UNCOMMITTED`.
5. **UI truth/wiring:** Agents SPAWN handlers, workspace tab navigation, chat toggle, snapshot banner,
   approval queue, tool attribution, proposals header consistency, kill dialog focus, mock timer cleanup,
   FigCraft gallery fallback, version display. `COMMITTED` in `848735a`, tests 262/262 before later edits.
6. **Docs/QA:** docs-truth checker fixed to 19/19; QA unit tests wired into `run-gates.ps1` and 8/8 pass.
   `COMMITTED` in `abfff55` (+ later).
7. **Installer/launcher:** launcher now drains no unread pipes, scans a free port, uses
   `%LOCALAPPDATA%\HiveMind\workspace`, no hardcoded dev path; installer/uninstaller and BUILD/CHECKSUMS
   package script added. `UNCOMMITTED`, installer smoke passed on temp roots but final package not built.

## 7. Candidate improvements after this analysis

### 7.1 EVO-GATE (recommended, not yet implemented)
Add an operator-run held-out evaluation receipt (pre-registered task-set sha, evaluator sha, baseline/
candidate pass rates + CI, head commit) and require it when applying a lesson; skip receipt-less lesson
lines at boot. This directly tests the paper's held-out evaluator claim and closes the boot-trust gap.
New `evolution_gate.rs`, apply/loader changes, `tests/evolution_gate.rs`; existing loader fixture must go
RED first. No new refusal code; rollback reverts the gate; lesson rollback remains.

### 7.2 ROOT-MONITOR (not yet implemented)
One bounded post-dispatch root turn with `send_instruction`/`cancel_task` and at most one replacement
branch; final stays tool-free; structured rows, not chat. Requires exact schemas, provider budget, and
a scripted-provider test proving no late mutation.

### 7.3 EVIDENCE-LEDGER (not yet implemented)
Add a target class (`Tests|Ci|Oracle|Source|Other`) to tool-ledger rows (class only, never path) and a
read-only per-run integrity counter. Makes "this run did not touch tests/CI" falsifiable.

**Rejected:** autonomous scaffold evolution, self-modifying code, DGM-style population search, sub-4GB
distillation, fabricated fitness scores, and any AGI claim. The papers' DGM/distillation mechanisms
require infrastructure and verifiers that this repo does not have; implementing them would violate the
no-mock rule.

## 8. Claims ledger

| ID | Claim | Grade | Evidence |
|---|---|---|---|
| H1 | HiveMind is a capped local HMAS, not AGI/autonomous/self-modifying | CODE + PAPER | `execution.rs`, `tool.rs`, `README.md`; `PLAN-FACTCHECK-CORE` |
| H2 | Root is a two-turn pre-dispatch controller plus tool-free final | CODE | `execution.rs` root loop; `provider_executor.rs` |
| H3 | Four voter tasks + operator apply/rollback exist; code apply absent | CODE | `http.rs`; `LESSONS.jsonl`; report L7 |
| H4 | Votes previously accepted any four completed SuperAgents | CODE | `http.rs` voter validation; paper auditor 1 |
| H5 | Seat binding now recorded/checked | UNCOMMITTED CODE | `execution.rs` seat key + vote check |
| H6 | No held-out independent evaluator | CODE | no evaluator code; free-text evidence only |
| H7 | No score archive/population/fitness | CODE (absence) | grep/read; no fitness type |
| H8 | No distillation/weight training | CODE (absence) | provider is HTTP client; no optimizer |
| H9 | UI S1 repairs and tests | CODE + test | `web/tests` 262/262 after H2 commit; current full suite pending |
| H10 | Full backend suite green after latest seat edit | UNVERIFIED | rerun command below |

## 9. Test/evidence commands still required

```powershell
$env:PATH="C:\Users\Riley\Desktop\hiverouter-v1\tools\w64devkit\bin;$env:USERPROFILE\.cargo\bin;"+$env:PATH
Set-Location backend; cargo test --workspace --locked
Set-Location ..\web; cmd.exe /c npm.cmd run typecheck; cmd.exe /c npm.cmd test -- --run
node tools/qa/check-docs-truth.mjs
node --test tools/qa/tests/*.test.mjs
```

A claimed test pass must quote the command and the observed result. The full suite was green at
`848735a` before the final seat-binding edit; that edit has only targeted `run_lifecycle` evidence so far.
