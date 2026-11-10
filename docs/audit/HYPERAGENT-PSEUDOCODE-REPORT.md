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

### 17. Request idempotency (APPLIED)
```text
request_detached(..., request_id)
  -> if remote_audit has this request_id:
       same display_name + adapter -> return existing remote (no second row)
       different payload -> E_DETACHED_CONFLICT
  -> otherwise run the normal admission gate and create once
```
Evidence: `lower_level_request_detached_is_idempotent_with_a_request_id`.

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

---

## 4. Current-wave pseudocode update (2026-11-10)

```text
tool_registry = 11 tools / 5 permission classes
web_fetch:
  caller tier in {SuperAgent, WorkerAgent} AND persona allows web_fetch
  -> request approval (Network)
  -> operator resolves nonce
  -> resume_and_execute
     if HIVEROUTER_SIDE_EFFECTS == 0: refuse E_TOOL_NOT_ALLOWED
     verify_and_consume_durable_grant(run, web_fetch, args)
     parse URL: https only, default port only, no userinfo
     resolve host under 10s timeout + cancellation
     reject loopback/private/link-local/CGNAT/240/4/IPv4-mapped/NAT64
     GET with redirects disabled, 15s timeout
     stream body until EOF or >1 MiB -> E_FILE_TOO_LARGE
     return bounded result

root propose_improvement:
  record in run bank
  state = awaiting_votes, required = 4 superagents + operator, unanimous
  applied = false; no code/schema/tool/prompt/gate changes
  no vote-submission/apply path exists yet
```

### Evidence additions

- `web_fetch` live success: run `01a1c689-e005-72cf-8f36-eefbedf973bb`,
  HTTP 200, one `web_fetch: ok`, 9,257 tokens.
- kill switch: run `01a1c676-20cb-74ca-8535-1540b1e942ce`, approved request
  refused `E_TOOL_NOT_ALLOWED`, no fetch.
- shell: run `01a1c678-fcbb-7182-9bfa-8f04218520ff`, approved
  `cmd.exe /c echo shell-ok`, stdout `shell-ok`, exit 0.
- Tests: backend 50 suites / 633 passed / 0 failed / 2 ignored; web 227 passed.
- Real bugs fixed by audit: kill-switch bypass, DNS hang, post-buffer body cap,
  unnecessary jail requirement, hyphenated persona lookup, missing durable
  grant verification for `web_fetch`.

### Honest boundary

RapidTool's phase pipeline, evidence-hashed DoD gate, Agent Packet, and unified
ledger remain **not implemented**. HiveMind now has a real approval-gated
network tool and a truthful five-seat proposal contract, but no implemented
SuperAgent vote submission or self-apply path.
---

## 5. Proposal surface verification (2026-11-10)

```text
root_control two turns:
  turn 1: propose_improvement A -> dedupe scan run bank -> record improvement:<uuid>
  turn 2: same A -> dedupe scan finds content-identical entry -> record root_control
          deduplicated; do not create a second proposal
proposal API:
  GET /api/agents/runs/{id}/proposals
  -> { count, applied:false, vote_path:not_implemented,
       proposals:[{proposal_id, area, problem, proposal,
                   governance:{required_approvals:5, operator:1, superagents:4,
                               unanimous:true, state:awaiting_votes, can_apply:false}}] }
ui ProposalsPanel:
  read-only; no approve/apply control; renders awaiting_votes and can_apply:false
```

Live: run `01a1c6a2-6c25-7333-a970-8ef8d6a61a0c` produced exactly one proposal
after two identical root-control attempts; `applied=false`, `vote_path=not_implemented`.
Backend 50 suites / 633 passed; web 229 passed. Package hashes are in README.

Still absent: four spawned SuperAgent voters, operator authorization, apply, and
rollback. The dedupe proves idempotent recording, not self-evolution.
---

## 6. SuperAgent vote recording (2026-11-10)

```text
tool vote_evolution (SuperAgent allowlist only):
  caller tier == SuperAgent
  proposal_id must exist as improvement:<id> in run bank
  vote in {approve, reject}; rationale 8..400
  one vote per (proposal_id, voter_task_id)
  write run-bank entry evolution_vote:<proposal_id>:<task_id>
  value {proposal_id, voter_task_id, role:superagent, vote, rationale, applied:false}
  no code, schema, tool grant, prompt, gate, or agent changes

GET /api/agents/runs/{id}/proposals:
  join proposal entries with evolution_vote:* entries
  approvals >= 4 -> vote_state=awaiting_operator
  any rejection -> rejected_by_superagent
  else awaiting_votes
  can_apply=false always
```

This is a real SuperAgent vote path, not a completed five-seat apply path. The
operator vote, a spawned four-SuperAgent evaluation orchestration, apply and
rollback remain absent. Tool surface is 12 tools / 35 grants; backend 50 suites /
633 passed; web 229 passed. Package hashes are in README.
---

## 7. Operator fifth-seat authorization (2026-11-10)

```text
POST /api/agents/runs/{run_id}/proposals/{proposal_id}/authorize
  proposal must exist as improvement:<proposal_id>
  collect evolution_vote:<proposal_id>:* entries
  require exactly 4 distinct SuperAgent voters, 4 approve, 0 reject
  require proposal applied=false
  append evolution_authorization:<proposal_id>
    {operator_approved:true, superagent_approvals:4, unanimous:true,
     state:"authorized_not_applied", applied:false}
  no code, schema, tool grant, prompt, gate, or agent changes
GET proposals now reports authorized=true and vote_state=authorized_not_applied
UI shows "Authorize with operator (5th seat)" only when all four approvals exist
```

This is the fifth seat, not self-application. Apply and rollback remain absent;
`can_apply` is false. Backend 633 passed; web 229 passed.
Live fifth-seat refusal: `POST .../authorize` on a proposal with zero
SuperAgent votes returned HTTP 409; no authorization row and no apply. Evidence:
`docs/evidence/live/live-tokenlab-authorize-refusal.json`.

The UI sends the per-boot `Authorization: Bearer <session token>` header; without
it the mutation correctly returns 401. This is the only fifth-seat path, and it
still cannot apply code.
Voter-identity hardening: the authorization endpoint now requires each of the
four voter task ids to belong to this run, map to `AgentTier::SuperAgent`, and be
`Completed`; unknown/worker/subagent/foreign-run voters are refused. A final-build
no-vote `POST .../authorize` returned **409**. This is still authorization only —
`applied:false` and no rollback/apply path.