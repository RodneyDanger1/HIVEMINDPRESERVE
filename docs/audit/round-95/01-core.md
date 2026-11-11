# Round 95 — Core-Truth Audit (HiveMind)

**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **HEAD:** `1129349` ("evidence(qa): add real kill/restart and 41-task scale receipts") · **Working tree:** one uncommitted change, `backend/crates/hiverouter-core/src/http.rs` (ledger-summary endpoint). It is **excluded** from every claim below.

**Method / labels.** Every material claim carries an evidence label:
`VERIFIED` = I ran the command and observed the output; `READ` = read the cited code/doc at HEAD; `UNVERIFIED` = plausible but not proven; `NOT DOCUMENTED` = no code/doc/term found.
I ran **no** test suite. The committed full-battery receipt (`docs/evidence/live/full-battery-20261110-215831.json`, `VERIFIED` to exist) records `source_commit: be03094…` and `working_tree_dirty: true` — **not** HEAD `1129349`. It therefore **cannot** be used to claim a green battery for this HEAD.

---

## 1. What each tier can actually do

Runtime tiers are derived from the task `role` string; unknown roles fall back to `WorkerAgent` and are never promoted (`execution.rs:295-310`). Spawn edges are enforced before a child row exists: Hyper→Super only, Super→Agent, Agent→Subagent, Subagent/Tool→none (`execution.rs:250-286`). The planner prompt states the same strict hierarchy (`provider_executor.rs:26-33`).

| Tier | Actual runtime | Spawn authority | Registry allowlist (tier ∩ persona is the effective surface) | Hard limits |
|---|---|---|---|---|
| **HyperAgent** | Root `planner` task depth 0; a separate **two-turn pre-dispatch control loop**, then a tool-less **one-turn post-dispatch monitor**, then tool-less final synthesis (`execution.rs:2266-2399`, `4435-4503`; `provider_executor.rs:705-761`) | `spawn_superagent` once, and only if the planner supplied **no** depth-1 branch; `spawn_remote_agent` / `spawn_daemon`; `assign_task`; `propose_improvement`; `convene_evolution_vote`; `read_memory_summary`; `finish` (`execution.rs:4454-4464`) | `spawn_agent`, `report_result`, `finish` (`tool.rs:463-466`). The root-control vocabulary is **local to `root_checkpoint`, not in the registry** (`execution.rs:4435-4437`) | ≤4 tool calls/turn, exactly 2 turns; turn 2 may not create agents/principals (`execution.rs:4475-4536`) |
| **SuperAgent** | Depth-1 branch lead, class `super`, owns `bank:hive:<task>` (`store.rs:2794-2822`) | Agent only; cannot spawn Subagent directly (`execution.rs:263-276`) | `spawn_agent`, `ask_parent`, `send_to_sibling`, `report_result`, `finish`, `file_read`, `file_list`, `file_write`, `shell_exec`, `request_detached`, `web_fetch`, `vote_evolution` (`tool.rs:467-483`) | fan-out 8; depth ≤3; 64 agents/run (`caps.rs:7-16`) |
| **Agent (Worker)** | Depth-2 worker, class `agent`, owns `bank:branch:<task>` | Subagent only (`execution.rs:277`) | same as SuperAgent **minus** `vote_evolution` (`tool.rs:484-499`) | same caps |
| **Subagent** | Depth-3 temporary leaf, class `subagent` | none (`execution.rs:278`) | `ask_parent`, `report_result`, `finish`, `file_read`, `file_list` (`tool.rs:500-509`) | same caps |
| **ToolAgent** | Census tier only; **no shipped producer** (`README.md:28`) | none | `file_read`, `file_list`, `file_write`, `shell_exec` (`tool.rs:510-518`) | n/a |

**Caps (code):** depth `3`, fan-out `8`, agents/run `64`, provider calls/run `64` (`caps.rs:7-16`). Refusals are named `E_SPAWN_DEPTH` / `E_FANOUT_LIMIT` / `E_AGENT_LIMIT` / `E_CALL_LIMIT`, never silent clamps (`caps.rs:55-70`). README additionally claims 40 tool calls/run, 12 tool turns, 32 messages, 2 A2A attempts (`README.md:44-46`) — **READ (doc only)**, the enforcement sites were not re-read here.

**Is HyperAgent a governor?** No, not in the continuous sense. It is a **bounded, two-phase root controller**: (a) up to two pre-dispatch LLM turns with a small creation vocabulary and a one-extra-branch rule, (b) dispatch/settlement, (c) a single **read-only** post-dispatch monitor turn whose text is appended as labelled data, and (d) a tool-less final synthesis. It cannot observe or intervene mid-flight, cannot re-dispatch after the monitor, and the monitor has no tools and no authority (`provider_executor.rs:709-733`; `execution.rs:2342-2398`). Lower-level detached requests are decided by a **deterministic admission policy**, not by the HyperAgent's reasoning (`README.md:72-88`, `execution.rs:5182+`).

---

## 2. Learning loop and EVO-GATE

**There is a lesson-artifact loop, not a behavioral learning loop.**

1. Root may call `propose_improvement`; it records `improvement:*` in the run bank with `applied:false` / `requires_human_review:true` and cannot edit code (`README.md:131-134`; prompt at `execution.rs:4459`).
2. SuperAgents record `approve`/`reject` via `vote_evolution`; only a `SuperAgent` task may vote, one vote per task, and only designated voters (`execution.rs:5070-5160`). Four distinct approvals + the operator's fifth seat are required (`http.rs:2522+`; `README.md:338-348`).
3. On apply, the engine appends one JSONL line to `<data_dir>/evolution/LESSONS.jsonl` after verifying the authorization record, proposal hash, replay, and safe text (`http.rs:2827-2937`). Rollback removes the line (`http.rs:2943-2965`).
4. **EVO-GATE now** = the module `evolution_gate.rs`. Before the lesson line is appended, the operator must have placed `data_dir/evolution/eval/<proposal>/receipt.json`; the gate requires schema `hivemind.evolution-eval.v1`, matching `proposal_id`, held-out set ≥ 8 tasks, a 64-hex task-set hash, evaluator id/hash, artifacts, and `candidate.ci_lower > baseline.ci_upper` (`evolution_gate.rs:1-15, 76-166`). The call site is real (`http.rs:2856-2871`).
5. Applied lessons are loaded at boot and hot-reloaded into planner, root-control, and task prompts as bounded labelled **data** (8 lines × 400 chars per `README.md:377-382`; injection sites `execution.rs:4424-4427,4483`).

**Persistent?** Yes, as a file artifact plus a memory-bank entry (`http.rs:2939-2949`). **Evaluated/gated?** Yes, but the gate *validates a receipt it does not run* (`evolution_gate.rs:13-15`). **Do agents/superagents improve the hive/HyperAgent?** No. Lessons change prompt text only; there is no code/schema/prompt mutation, no weight update, no automatic promotion, no skill library, and `can_apply` remains false for code (`README.md:363-376`). SuperAgent votes are data, never authority (`tool.rs:454`).

**EVO-GATE gaps (READ):** `head_commit` is present in the receipt fixture (`evolution_gate.rs:178`) but is **never read or compared to HEAD**; `artifacts[].sha256` is only format-checked, not bound to actual files (`evolution_gate.rs:144-157`); the evaluator is not executed or attested, so a well-formed operator-authored receipt passes. This is an admission filter, not measurement.

---

## 3. Memory: no vector search, no embeddings, no semantic recall

A repo-wide `rg` for `embedding|vector|semantic|cosine|similarity` over `backend/**/*.rs` returned **zero memory-related hits** (only "escape vector", "semantic validation" comments). There is **no vector DB, no embedding call, no ANN index, no semantic query path** (`NOT DOCUMENTED`).

What exists is **SQLite-backed, bank-partitioned, recency-ordered text memory**:

- Tables: `memory_banks` (kind ∈ global/project/run/hive/branch/agent_role), `memory_entries` (kind, value, source_kind, trust ∈ quarantined/unverified/verified/authoritative, status), `memory_journal`, `memory_context_snapshots`, plus legacy `agent_memory` / `agent_memory_scoped` (`db.rs:183-200, 253-269, 397-436, 519-560`).
- Banks: HyperAgent → `bank:run:<run>`; SuperAgent → `bank:hive:<task>`; Agent/Subagent → `bank:branch:<task>` (`README.md:99-102`; `store.rs:2794-2822`).
- Retrieval is **recency + fixed bank set, not similarity**: a task reads run bank, its own branch, its parent branch, and the nearest ancestor hive bank, `ORDER BY created_at DESC LIMIT ?` (`store.rs:2217-2270`). Sibling/foreign-run banks are not addressable. The HyperAgent control turn reads **run+hive summaries only**, `memory_hyper_summary` → `hyper_summary_rows` (`store.rs:2272-2286, 2824-2857`).
- Prompt injection: `prompt_with_context(...)` plus `prompt_with_authorized_lessons(...)` at every task execution and at root control (`execution.rs:4421-4427,4483`); root final synthesis receives a bounded task digest and per-task results capped at 2,000 chars with a 24,000-char budget (`execution.rs:2282-2341`).
- Write path: agent results/engine summaries are written into the caller's bank; `write_run_digest` stores the digest (`execution.rs:2311`).

**Who sees what:** HyperAgent → run/hive summaries; SuperAgent → its hive bank + own branch + run bank + parent; Agent/Subagent → own branch + parent branch + run + nearest hive. Trust filter admits `unverified` entries (`store.rs:2241-2242`) — unverified memory *is* injected.

---

## 4. HiveWire

**`HiveWire` does not exist in this repository.** A case-insensitive repo-wide search for `HiveWire|Hivewire|hivewire|hive_wire|HIVEWIRE` returned **0 matches** across code, docs, web, e2e and plans; a filename search under `C:\Users\Riley\Desktop` (depth 3) also returned nothing. Status: **NOT DOCUMENTED / NOT IMPLEMENTED**. It is not usable as an agent harness, and I will not invent a design for it.

Do not confuse it with what does exist:

| Thing | What it is | Evidence |
|---|---|---|
| **HiveMind** | The local single-provider hierarchical multi-agent framework (HMARS): planner → root HyperAgent → SuperAgents → Agents → Subagents, with SQLite store, approvals, remotes/daemons. | `README.md:1-14` |
| **`ws.rs`** | The WebSocket/event transport between engine and UI; not an agent harness or a wire protocol for third-party agents. | `backend/crates/hiverouter-core/src/ws.rs` (role mapping `:273-297`; rewire `:1321`) |
| **`remote.rs`** | Tree-free **in-process fixture** principal + device + narrow grant; explicitly not a network satellite. | `README.md:26,187`; `remote.rs:1371-1404` |
| **`daemon.rs`** | Enrolled tree-free record; watchdog has a real DB-check job, but no continuously scheduled OS process / live tick proof. | `README.md:27,188`; `db.rs:795-825` |

If "HiveWire" is a separate product/spec, it lives outside this repo and must be supplied before it can be audited.

---

## 5. Is HiveMind AGI? A Gödel machine or meta-HyperAgent?

**No.** It is a capped, single-run, single-provider hierarchical orchestrator. The README's own claim register says so (`README.md:179-193`). Code supports that: lessons are prompt data, proposals never mutate code, no evaluation loop, no self-model, no open-ended goals, no cross-run weight/skill update, no held-out benchmark, no variant archive or evolution sandbox. It is **not** a Gödel machine (no self-referential proof search or self-rewriting program) and **not** a meta-HyperAgent (the HyperAgent does not improve the procedure that produced it; EVO-GATE admits operator-authored lesson text only).

| Capability | Boundary (evidence) |
|---|---|
| Planning | One planner LLM call → DAG, depth ≤3, ≤8 nodes (`provider_executor.rs:26-33`; `caps.rs:7-16`) |
| Control | 2 root-control turns, ≤4 calls/turn; no continuous loop (`execution.rs:4475-4484`) |
| Self-modification | None; proposal-only, `applied:false` (`README.md:131-134`) |
| Learning | Lesson text injection only; no weights/skills/tools mutated (`README.md:377-382`) |
| Evaluation | Receipt *validation* only; evaluator not run (`evolution_gate.rs:13-15`) |
| Memory | SQLite banks, recency retrieval; no embeddings/vector recall (`§3`) |
| Action | Jail + approval gate; shell is approval-gated arbitrary code, not an OS sandbox (`shell.rs:169-181`) |
| Network | HTTPS-only `web_fetch` with SSRF filtering, 1 MiB cap (`README.md:316-324`); remotes are in-process |
| Scale | 64 agents/64 calls; 40-node live scale **not run** (`README.md:191`) |
| Persistence | One run at a time; no daemon scheduler, no durable cross-run autonomy (`README.md:188`) |

---

## 6. Biggest capability gaps (code evidence)

1. **Approval/containment is human-gated but not a sandbox.** `Mutate`/`Execute`/`Network` require approval; `Read`/`Control` do not (`tool.rs:57-61`). Approval binds `(run, tool, args_sha256)`, 120 s TTL, single-use nonce (`approval.rs:1-16,31`), but `AllowSession` is **non-expiring** (`approval.rs:12-13`). `shell_exec` allows `git.exe`/`cargo.exe`, which can execute repo-controlled code with user privileges; interpreters are denied, but this is explicitly **not** an OS sandbox (`shell.rs:169-181`).
2. **Root monitor review is read-only and post-hoc.** It can only emit text; no tools are advertised; if it returns tool calls the run **fails** (`execution.rs:2384-2387`; `provider_executor.rs:709-733`). It cannot re-dispatch, cancel, or repair. It also costs one provider call (`execution.rs:2347`). There is no mid-flight root supervision.
3. **Caps are per-run and small.** depth 3 / fan-out 8 / 64 agents / 64 provider calls (`caps.rs:7-16`); root control 2×4 (`execution.rs:4475-4484`). There is no per-tenant, per-project, or cost budget here. README's 40-node claim is explicitly not live-tested (`README.md:191`).
4. **Settlement is a sequential repair loop, not a scheduler.** `settle_unfinished_tasks` re-queues `Running` tasks and executes pending tasks one at a time, max 64 iterations, then `InternalError` (`execution.rs:2557-2583`). `dispatch_spawned_tasks` is a single-consumer `while let` loop (`execution.rs:2585-2606`). Concurrency is provider-call-bound, not worker-parallel.
5. **Tool grants are static, tier-based, with a dead ToolAgent lane.** 12 tools, 5 tiers, 35 grants (`tool.rs:462-518`; `README.md:270-276`). ToolAgent has grants but no producer (`README.md:28`). HyperAgent's registry allowlist includes `spawn_agent`, but strict hierarchy means the only valid child is a SuperAgent (`execution.rs:259-262`). Effective surface is tier ∩ persona, so a persona can hide tools a tier allows (`README.md:40-42`).
6. **EVO-GATE validates shape, not truth.** It does not run the evaluator, does not bind `head_commit` to HEAD, and does not hash-check artifacts against files (`evolution_gate.rs:76-166`). An operator can hand-author a passing receipt.
7. **Memory has no semantic recall and admits unverified entries.** Retrieval is recency-only (`store.rs:2238-2244`); trust filter includes `unverified` (`store.rs:2241-2242`). There is no dedup/conflict resolution beyond `status` and `UNIQUE(bank_id, entry_key) WHERE active` (`db.rs:432-433`).
8. **Remote/daemon are not network-capable.** Remote is an `in_process` fixture; daemon has no live tick proof (`README.md:26-27,187-188`).

---

## 7. Pseudocode — the three mechanisms that matter most

### 7.1 Root control + post-dispatch monitor (bounded two-phase governor)

```text
ROOT_CHECKPOINT(run, root, plan):
  if not ROOT_ORCHESTRATOR: return
  has_branch = any(node.depth == 1 for node in plan.nodes)
  results = ""; super_spawned = false
  for turn in 0..2:
    claim_provider_call(run)                       # counted, may refuse E_CALL_LIMIT
    reply = ORCHESTRATE_ROOT(prompt + results)     # tools: spawn_superagent|assign_task|
                                                   # spawn_remote_agent|spawn_daemon|
                                                   # read_memory_summary|propose_improvement|
                                                   # convene_evolution_vote|finish
    if reply.tool_calls empty: return
    for call in reply.tool_calls.take(4):
      if turn == 2 and call in {spawn_superagent, spawn_remote_agent, spawn_daemon,
                                convene_evolution_vote}:
        record_root_gate_decision("turn_authority","refused"); continue
      if call == spawn_superagent and (has_branch or super_spawned):
        record_root_gate_decision("spawn_superagent","refused"); continue
      execute(call); results += durable_result_text

DISPATCH_AND_SETTLE:
  for while queued: execute_task(one task)          # single consumer
  settle_unfinished_tasks(root):                    # requeue Running, execute, max 64
  if every child terminal:
    if ROOT_ORCHESTRATOR and executor.supports_root_review():
      claim_provider_call(run)
      review = REVIEW_ROOT(task states + bounded results)   # no tools advertised
      if review.tool_calls non-empty: FAIL run
      root_prompt += "POST-DISPATCH MONITOR REVIEW (data)" + review.text[..2000]
    FINALIZE_ROOT(root_prompt)                      # no tools advertised
```

### 7.2 Spawn admission and structural settlement

```text
SPAWN(parent_task, requested_role):
  tier = TIER(parent.role)                          # unknown role -> WorkerAgent
  if not SPAWN_ALLOWED[tier](requested_role): refuse E_TOOL_NOT_ALLOWED
  depth = parent.depth + 1
  CAPS.check_spawn(depth, child_count(parent), run.task_count)
     # depth 1..3, siblings < 8, agents < 64
  insert child row (state Queued); graph CAS; context parent switch
  # a task is ready only when deps AND direct children are terminal
```

### 7.3 Lesson admission (EVO-GATE) and prompt injection

```text
APPLY_LESSON(run, proposal_uuid):
  auth = run_bank["evolution_authorization:"+id]        # 4 SuperAgent + operator seats
  require auth exists; proposal exists; proposal.sha256 == auth.proposal_sha256
  require no prior "evolution_apply:"+id                 # replay -> E_APPROVAL_REPLAY
  receipt = VERIFY_EVALUATION_RECEIPT(data_dir, id)
     # file: <data_dir>/evolution/eval/<id>/receipt.json, <=64 KiB, regular file
     # schema == hivemind.evolution-eval.v1, proposal_id == id
     # task_set.n >= 8, task_set.sha256 64-hex, evaluator id+sha256
     # candidate.ci_lower > baseline.ci_upper; artifacts sha256 64-hex
     # NOTE: head_commit NOT compared; evaluator NOT executed
  append lesson JSONL atomically (receipt_sha256, baseline/candidate rates)
  write run_bank entry "evolution_apply:"+id; hot-reload lessons

TASK_PROMPT(task):
  prompt = prompt_with_context(prompt, task.context)
  prompt = prompt_with_authorized_lessons(prompt, snapshot)   # 8 x 400 chars, data only
```

---

## 8. Task status

`task-24` could not be claimed or completed: no shared-task tool is mounted in this subagent session, and a repo-wide search for `task-24` returned **0 matches** (no board file, no ledger row). The audit itself is complete and written here. This is a tooling limitation, not a finding about HiveMind.

---

## 9. Report shape

### Done
| Item | Evidence |
|---|---|
| Repo/HEAD verified | `git log --oneline -3` → `1129349`; `git status --short` → only `M …/http.rs` (`VERIFIED`) |
| Tier/spawn/allowlist audit | `execution.rs:250-310`, `tool.rs:462-518` (`READ`) |
| Caps audit | `caps.rs:7-70` (`READ`) |
| Root control + monitor audit | `execution.rs:2266-2399, 4435-4536`; `provider_executor.rs:705-761` (`READ`) |
| Learning loop / EVO-GATE audit | `evolution_gate.rs:76-166`; `http.rs:2827-2949`; `execution.rs:5070-5160` (`READ`) |
| Memory audit | `db.rs:397-436`; `store.rs:2217-2286, 2794-2857` (`READ`) |
| HiveWire determination | repo-wide search 0 matches; Desktop filename search 0 (`VERIFIED`) |
| Gaps + pseudocode | §6, §7 above |

### Not done
| Item | Reason |
|---|---|
| Claim/complete `task-24` | No shared-task tool mounted; no `task-24` artifact in repo or Desktop (depth 3). |
| Run backend/web tests at HEAD | 8-minute budget; would not finish. No HEAD-matching receipt exists. |
| Audit every one of the named files end-to-end | Budget; `store.rs` (3,564 lines) and `execution.rs` (5,838) were sampled at the cited mechanisms, not fully read. `remote.rs`, `daemon.rs`, `tools_bridge.rs`, `web/src` were searched/spot-read only. |

### Unverified
| Claim | What would prove it |
|---|---|
| Root monitor review is exercised in a live run | A committed receipt/log for commit `1129349` showing the monitor turn and its prompt label. Code path is `READ`, not live-proven. |
| 40-node live scale | A committed 41-task live receipt at this HEAD (commit message claims one; I did not open it). |
| README-only caps (40 tool calls, 12 turns, 32 messages, 2 A2A) | Read the enforcing modules (`tool_loop.rs`, A2A path) and run the corresponding tests. |
| "HiveWire" intended meaning | A spec or repo defining it; none exists here. |
