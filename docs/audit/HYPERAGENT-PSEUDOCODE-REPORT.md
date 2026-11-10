# HyperAgent / HMARS — Pseudocode Report and 20 Improvements

**Date:** 2026-11-10
**Purpose:** describe what HiveMind is in executable pseudocode, then list twenty
concrete improvements with before→after logic. Status is explicit; no proposal
is described as shipped unless it has code **and** a test.

Sources: local code, `Hyperagent psuedocode helpful info.txt`, arXiv:2603.19461
(Meta HyperAgents), arXiv:2409.16299 (HyperAgent SE), the AGI/metalearning
pages linked from the goal, and the local audit set.

---

## 0. Current canonical pseudocode

```text
create_run(prompt):
  plan = provider.plan(prompt)                  # counted call; one repair on schema failure
  plan.validate()                               # DAG, depth, parent/child, caps
  root = persist_task(class=hyper, role=planner, depth=0)
  seed_run_bank(root); seed_context(root)

  control_results = ""
  for turn in [0, 1]:                           # two bounded turns, not a daemon
    reply = provider.root_control(prompt + control_results)
    if reply.tool_calls == []: break
    for call in reply.tool_calls[0:4]:
      decision = handle_root_call(root, call)   # spawn_superagent | remote | daemon |
                                                # memory | propose_improvement | finish
      control_results += decision.text
      persist_gate_decision(decision)           # run bank + root_control audit

  ready = nodes whose deps and direct children are terminal
  while not all_terminal:
    batch = first N ready nodes
    for node in batch:
      reply = provider.execute(node)
      while reply.tool_calls:
        result = execute_tool(node, call)       # spawn | A2A | file | shell | request_detached
        reply = provider.continue_with_tools(...)
    mark completed
  final = provider.finalize_root(all_completed_results)  # no tools
  persist_usage(); finish_run()
```

### Current per-level pseudocode

```text
HyperAgent:
  owns run bank; plans; two control turns; final synthesis
  can: spawn_superagent (normally refused), spawn_remote, spawn_daemon,
       read_memory_summary, propose_improvement, finish
  cannot: self-edit, run async, approve a lower-level request in an LLM turn

SuperAgent:
  owns hive bank; spawns Agent; aggregates direct child results; A2A; files
  request_detached -> HyperAgent admission gate
  effective tools = tier ∩ persona (no file_write/shell_exec in SuperAgent persona)

Agent:
  owns branch bank; widest tools; spawns Subagent; request_detached
  file_write/shell_exec require approval

Subagent:
  temporary depth-3 leaf; read/list + A2A + report/finish; no spawn
```

---

## 1–20. Improvements

Status key: **APPLIED** = code + test/live evidence; **PARTIAL** = code only;
**PROPOSED** = pseudocode only; **REJECTED** = unsafe/not implemented.

### 1. Lower-level detached request (APPLIED)
```text
before: only HyperAgent root control can create a remote/daemon
after:
  SuperAgent/Agent -> request_detached(kind, display_name, reason)
    -> deterministic HyperAgent admission gate
```
Evidence: live `Lantern` run; admission tests.

### 2. Admission policy checks (APPLIED)
```text
before: no lower-level path
after:
  reject unless tier in {SuperAgent, WorkerAgent}
  reject reason < 12 chars
  reject duplicate name across remotes AND daemons
  reject if lower-level detached count >= 4
```
Evidence: `execute_request_detached`; duplicate/daemon tests.

### 3. `hyper_admission` audit row (APPLIED)
```text
before: no lower-level approval record
after: allow -> audit(system, agent:<task>, hyper_admission, decision=allow)
```
Evidence: live Lantern audit row.

### 4. Durable root-gate decisions (APPLIED)
```text
before: root tool result discarded
after: run-bank root_gate:<id> + remote_audit root_control row
```
Evidence: root gate test.

### 5. Root-control feedback loop (APPLIED)
```text
before: for turn in 0..1; _result_text discarded
after: for turn in 0..2; turn 2 prompt includes "Prior root-control results..."
```
Evidence: second-prompt assertion.

### 6. Proposal-only self-improvement (APPLIED)
```text
before: no safe improvement surface
after:
  propose_improvement(area, problem, proposal, evidence)
    -> run-bank improvement:<id> {applied:false, requires_human_review:true}
  no code write, no execution, no promotion
```
Evidence: root proposal test. This is the deliberate alternative to
self-modifying code.

### 7. One extra root branch per run (APPLIED)
```text
before: up to 4 spawn_superagent calls on a flat plan
after: root_super_spawned=true after first allow; further calls refused
```
Evidence: root gate tests.

### 8. Cross-kind duplicate names (APPLIED)
```text
before: remote names and daemon names checked separately
after: one root_all_names set covers both kinds
```
Evidence: duplicate tests.

### 9. Daemon admission audit (APPLIED)
```text
before: lower-level daemon admission had no audit row
after: root_control audit with daemon_id + caller + allow
```
Evidence: daemon admission test.

### 10. Truthful device/grant attribution (APPLIED)
```text
before: device_created/grant_created actor operator
after: actor system + actor_id hyper:<root> / agent:<task>
```
Evidence: remote and lower-level audit tests.

### 11. Persona aliases (PARTIAL)
```text
before: role "super"/"worker" found no persona
after: super -> superagent, worker -> agent, hyper -> planner
```
Evidence: alias unit test; no live request capture.

### 12. Tier∩persona tool advertisement (PARTIAL)
```text
before: provider advertised full tier surface
after: tools = tool_specs_for_tier(tier) ∩ persona.allowed_tool_ids
```
Evidence: source; focused request-capture test still missing.

### 13. Same-run rewire guard (APPLIED)
```text
before: REWIRE_TASK could attach to a foreign-run parent
after: if task.run_id != parent.run_id -> E_TASK_NOT_FOUND before mutation
```
Evidence: WS integration test.

### 14. HyperAgent run bank (APPLIED)
```text
before: class hyper + non-"run" scope wrote branch bank
after: class hyper always -> bank:run:<run>
```
Evidence: run-bank test.

### 15. Direct-child result label (PARTIAL)
```text
before: "Completed child <id>"
after: "DIRECT CHILD RESULT (this task's own child) <id>"
```
Evidence: source; model-quality effect unverified.

### 16. Root memory scope/limit (PARTIAL)
```text
before: read_memory_summary ignored scope and returned fixed limit
after: requested_scope filter + clamp 1..8
```
Evidence: source; no focused test.

### 17. Request idempotency (PROPOSED)
```text
request(key, caller, kind, name, reason)
if ledger[key] exists: return prior result (idempotent)
else admit once; ledger[key]=result
```
Not implemented; duplicate same name currently refuses rather than replays.
Safe next step; requires a request ledger.

### 18. Dedicated control ledger (PROPOSED)
```text
before: root decisions split between run-bank memory and remote_audit
after: root_control_log(run, seq, actor, action, decision, detail, hash)
```
Not implemented; `root_control` rows currently live in `remote_audit` with
`remote_id=NULL`, which is durable but not a first-class export.

### 19. Network remote transport (PROPOSED / RESEARCH)
```text
remote = enroll(endpoint, certificate, capabilities)
outbound heartbeat + action grant; no inbound tree edge
```
Not implemented; current transport is `in_process` fixture only. Do not claim
network satellite.

### 20. Daemon runtime tick (PROPOSED)
```text
daemon.tick():
  if now >= next_tick_at: run job; publish telemetry; update next_tick_at
```
Not implemented as a live scheduler; daemon is enrolled and has a DB-check job.

### Also proposed but rejected
- **Self-modifying HyperAgent / hot-swapped source (REJECTED).** No editable
  task+meta program, no sandbox promotion, no auto-apply. Proposal-only is the
  safety boundary.
- **AGI claim (REJECTED).** Fails held-out breadth, transfer, continual learning,
  calibrated self-model, and open-ended goals.

---

## 2. Safety rules for the proposed work

1. A proposal is data, never an instruction to change code.
2. No agent may edit its own allowlist, caps, verifier, reward, or promotion gate.
3. Any future mutation sandbox must have compile/interface/invariant tests,
   human approval, a backup and an automatic rollback.
4. Network remotes require explicit operator enrollment, narrow grants, TTL and
   inbound audit; until then the honest word is "fixture".
5. Every claim in README/report carries a grade: VERIFIED / READ / INFERRED /
   UNVERIFIED. Unknown is not zero.

## 3. Bottom line

HiveMind is a **real, tested, capped hierarchical HMAS** with a governing root,
branch memory, A2A, admitted detached fixtures, audit rows, proposal-only
self-improvement, and an observable UI. It is **not** an AGI and not a
self-modifying HyperAgent. The next safe gains are request idempotency, a
first-class control ledger, network transport, daemon ticks, and live stability —
not a claim of autonomy.
