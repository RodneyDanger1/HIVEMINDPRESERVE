# PSEUDOCODE-P1-META — Meta HyperAgents (DGM-H) vs. HiveMind `HyperAgent`

Status: read-only research note. No source file was edited to produce it.
Date: 2026-11-10. Time-boxed pass (P1).
Paper: *HyperAgents*, arXiv:2603.19461 — v1 (60 pp.) and v2 (63 pp.) both present locally.
Both PDFs carry the same title, author list, and abstract opening; v2 is three pages longer.
Only the abstract/introduction and the taxonomy audit were read in full. A line-by-line v1↔v2
diff was **not** performed.

Code refs are to `919c339`-era paths cited by
[AGENT-TAXONOMY-VERIFICATION.md](AGENT-TAXONOMY-VERIFICATION.md); line numbers are the
current worktree. `docs/audit/AGENT-TAXONOMY-VERIFICATION.md` is the prior authority for the
"what HiveMind is" claims and is quoted where it already established a fact.

---

## 1. What the Meta HyperAgent actually is

### 1.1 Exact quoted facts (v1 p.1 abstract; v2 abstract repeats them)

> "We introduce hyperagents, self-referential agents that integrate a task agent (which
> solves the target task) and a meta agent (which modifies itself and the task agent) into a
> single editable program."

> "Crucially, the meta-level modification procedure is itself editable, enabling
> metacognitive self-modification, improving not only task-solving behavior, but also the
> mechanism that generates future improvements."

> "Starting from a single coding agent, the DGM repeatedly generates and evaluates
> self-modified variants, forming a growing archive of stepping stones for future
> improvement."

> "By allowing the improvement procedure to evolve, the DGM-H eliminates the assumption of
> domain-specific alignment between task performance and self-modification skill, and can
> potentially support self-accelerating progress on any computable task."

> "Across diverse domains (coding, paper review, robotics reward design, and Olympiad-level
> math-solution grading), the DGM-H improves performance over time and outperforms baselines
> without self-improvement or open-ended exploration, as well as prior self-improving systems
> like DGM."

> "All experiments were conducted with safety precautions (e.g., sandboxing, human
> oversight)."

From the introduction (v1 p.1–2):

> "Here, an agent is any computable program, optionally including calls to foundation models
> (FMs), external tools, or learned components. A task agent solves a given task. A meta agent
> modifies agents and generates new ones."

> "the DGM-H grows an archive of hyperagents by branching from selected candidates, allowing
> them to self-modify, evaluating the resulting hyperagents, and adding them back to the
> archive."

### 1.2 Mechanism in pseudocode (paper's design, not ours)

```text
DGM_H(seed_hyperagent):
  archive = { seed_hyperagent }                      # growing stepping-stone archive

  loop forever:
    parent   = select(archive)                       # exploration / selection pressure
    variant  = parent.self_modify()                  # edits task agent AND meta agent
    # both evaluation and self-modification are programs in the same editable file
    score    = evaluate(variant, task_suite)         # benchmark, not self-reported
    variant.lineage = parent.id; variant.score = score
    archive.add(variant)                             # keep failures too: stepping stones

  # every hyperagent is one editable program:
  #   task_agent(prompt) -> answer
  #   meta_agent(archive, history) -> edit proposal for task_agent + meta_agent
  # the meta agent is INSIDE the artifact it can rewrite (metacognitive self-modification)
```

Key properties, stated plainly:

1. **Fused artifact.** Task agent and meta agent are not two services; they are one program.
2. **Meta-level is editable.** The procedure that generates improvements is itself a target
   of improvement. This is the property that separates a hyperagent from a fixed scaffold.
3. **Archive.** Self-modified variants are retained (with evaluation records) and branched
   from. The archive is the population, not a log.
4. **Evaluation before trust.** A variant enters the archive only with an evaluation result.
5. **Sandbox + human oversight.** The paper explicitly names these as the safety frame for
   all experiments. It does **not** claim the sandbox/human review is unnecessary.

### 1.3 What the paper does *not* license

- It does not say a production orchestrator with a fixed role hierarchy is a hyperagent.
- It does not say prompt text calling an agent "meta" makes it metacognitive.
- It does not claim safety follows from capability; the safety precautions are external
  (sandbox, human oversight).

---

## 2. What OUR `HyperAgent` is — and is not

### 2.1 Current code path (verified static, per audit + re-read)

Root creation: `execution.rs:1780-1794` inserts a `TaskRecord { role: "planner", depth: 0,
parent_task_id: None }`, then labels the graph node `"hyperagent"` / `"HyperAgent"`.
There is exactly one planning provider call before this row exists
(`execution.rs:1747-1777`; one repair attempt). The root row therefore does not plan.

Control turn: `root_checkpoint` (`execution.rs:3866-4001`) runs once, **before** dispatch,
when `root_orchestrator` is enabled. It offers four locally-scoped tools
(`spawn_superagent`, `spawn_remote_agent`, `read_memory_summary`, `finish`; `spawn_daemon`
is also handled), caps calls at `MAX_ROOT_TOOL_CALLS = 4` (`execution.rs:3902`), loop is
`for _turn in 0..1` (`execution.rs:3903`), and the collected `_result_text` is never fed back
to the model. `read_memory_summary` runs before branch memories exist.

Final turn: `execution.rs:2043-2088` runs only when `has_aggregators || plan.nodes.len() > 1`;
it concatenates bounded child results into a prompt and calls `finalize_root`, which has no
tools. A single-node plan gets no root answer.

### 2.2 Our HyperAgent, in pseudocode

```text
OUR_HYPERAGENT(run):
  # --- what actually exists today ---
  plan = executor.plan(run.prompt)                  # detached call, no root row yet
  plan = executor.repair_plan(...) if invalid       # one counted retry
  plan.validate()                                   # <=40 nodes, depth<=3, tree parent chain

  root = insert_task(role="planner", depth=0, state=Running)
  graph.metadata(root, kind="hyperagent", label="HyperAgent")
  run_bank.write("engine/prompt", prompt[0..2000])
  run_bank.write("engine/plan", digest(plan))

  root_checkpoint(root, plan)                       # ONE provider turn, pre-dispatch
    reply = executor.orchestrate_root(prompt + plan_digest)
    for call in reply.tool_calls.take(4):
        spawn_superagent  -> spawn_sub_hive(root, "super")   # only if plan has no depth-1
        spawn_remote_agent-> remote principal + device + grant
        spawn_daemon      -> allowlisted script daemon
        read_memory_summary -> rows read, result discarded
        finish            -> stop
    # no second turn; no feedback of what happened; no TOOL_CALL audit event

  dispatch planned DAG                               # root does not execute a task
  drain_spawned_tasks()

  if plan has aggregators or >1 node:
      root_prompt = run.prompt + bounded(child results)
      answer = executor.finalize_root(root_prompt)   # no tools; last chat message wins
  run.status = terminal
```

### 2.3 Is / is not (the honest boundary)

| Meta paper property | HiveMind today | Evidence |
|---|---|---|
| task agent + meta agent fused in one editable program | **No.** Root is a task row; planning and execution are detached provider calls | `execution.rs:1747-1794` |
| meta-level modification procedure editable | **No.** No code path writes prompts, tools, planner, or role tables | audit G01; no writer found |
| archive of evaluated variants | **No variant archive.** There is an append-only hash-chained memory journal and run digests | `store.rs:40-102`, `execution.rs:2088` |
| evaluation before promotion | **No promotion concept.** Plan validation is schema validation, not fitness | `planner.rs:50-129` |
| open-ended loop / branching from selected candidates | **No.** One run = one plan; no cross-run selection | `execution.rs:1747+` |
| sandbox / human oversight | **Partial.** Operator approval for `file_write`/`shell_exec`; daemon allowlist root; remote grants | audit §4.3; `daemon.rs:125-139`; `remote.rs:61-71` |
| self-modification | **Absent.** No runtime self-edit, no weight update, no prompt rewrite | audit G01, final verdict |

One-line verdict: **HiveMind's `HyperAgent` is a run-scoped root task and final synthesizer
wearing Meta's name; the hierarchy is real, the metacognition is not.** HiveMind does not
self-modify, and nothing in this report should be read as claiming it does.

---

## 3. Improvement proposals (borrowed mechanism, not borrowed claims)

Legend for **auto-apply**:
- `YES` — documentation/test-only change; cannot alter runtime behavior.
- `NO` — changes runtime, schema, prompt, or budget; needs review + tests + human sign-off.

### P1. Post-dispatch root observation turn (bounded re-entry)

**Why.** The paper's meta agent acts on *evaluation results*. Our root acts once, before any
result exists, then never sees the run again until a tool-less final synthesis. That is a
blind meta level, not a metacognitive one.

**Before → after**

```text
BEFORE:
  root_checkpoint(plan)                  # once, pre-dispatch
  dispatch(...)
  finalize_root(results)                 # no tools, no decisions

AFTER:
  root_checkpoint(plan)                  # unchanged: pre-dispatch gate
  dispatch(...)
  if root_orchestrator and run not cancelled:
      obs = bounded_observation(results) # counts, failures, gate decisions; no raw text > N
      reply = executor.orchestrate_root(prompt + obs + "no new agents; choose finish or request_replan")
      for call in reply.tool_calls.take(1):
          "finish"          -> continue
          "request_replan"  -> record proposal only; NEVER auto-dispatch
          unknown           -> refuse E_TOOL_NOT_ALLOWED
      audit(root_action, "post_dispatch_observe", digest(obs))
```

**Safety constraint.** Exactly one extra provider call (claim it); max 1 tool call; the only
spawn-capable tool is absent; `request_replan` writes a proposal row and returns, it does not
mutate the DAG. Cancellation checked before and after.
**Effort:** M. **Auto-apply:** NO.

### P2. Hash-chained plan/decision archive (extend the journal we already have)

**Why.** The paper retains variants and their evaluations so later selection is evidence-based.
HiveMind already has the right primitive — `memory_journal` chains `prev_hash` → `event_hash`
over `(run, bank, entry, action, actor, detail, time)` (`store.rs:40-102`) — but it only covers
memory writes. Gate decisions, plan digests, and final answers are not in that chain.

**Before → after**

```text
BEFORE:
  memory writes -> memory_journal (hash-chained)
  root gate decisions -> row/event, not chained
  plan digest -> run bank value only

AFTER:
  append_audit(kind, payload):
      prev = last(event_hash for run)
      hash = sha256(prev | run | kind | actor | payload | ts)
      insert audit_journal(prev, hash, ...); update runs.audit_head = hash
  on plan_validated      -> append_audit("plan", digest(plan))
  on root_gate_decision  -> append_audit("root_gate", decision)
  on final_answer        -> append_audit("final", sha256(answer))
  verify_chain(run) -> bool   # exposed as a read-only check
```

**Safety constraint.** Append-only; no update/delete path; verification failure surfaces as a
refusal, never a silent pass; no payload may contain secrets (redact before hashing/insert).
**Effort:** M (schema + migration). **Auto-apply:** NO.

### P3. Variant archive of *run specifications* (stepping stones, not self-editing agents)

**Why.** The archive is the paper's memory of what worked. We can archive the *inputs* to a
run — plan shape, role assignment, prompt digest, evaluator score, outcome — without archiving
executable agents. This gives selection pressure with zero self-modification.

**Before → after**

```text
BEFORE:
  run ends -> run row + tasks + results (queryable, not curated)

AFTER:
  on terminal(run):
      variant = {
        plan_shape: [(role, depth, parent_index)],
        prompt_digest, budget_used, terminal_status,
        score: optional external evaluator result,
        lineage: parent_variant_id | null
      }
      archive.insert(sha256(canonical(variant)), variant)   # dedup by digest
  suggest(plan_shape): archive.nearest(plan_shape) -> read-only prior + its outcome
  # suggestion is injected as DATA, clearly labelled "prior run, not instruction"
```

**Safety constraint.** Archive stores no credentials, no raw prompts (digest only), no tool
outputs; suggestion text is untrusted data and cannot become a tool call; deletion of a run
must not break chain verification (tombstone record).
**Effort:** M. **Auto-apply:** NO.

### P4. Promotion gate for any behavior change (proposal → sandbox eval → human)

**Why.** The paper only admits an evaluated variant to the archive. If HiveMind ever tunes
personas/prompts/planner heuristics, it needs the same gate: no change reaches production on
self-reported success.

**Before → after**

```text
BEFORE:
  edit persona/prompt/planner -> deployed directly (no eval record)

AFTER:
  propose(change):
      record proposal(change_id, diff, author, rationale)
      run golden_suite(change) in isolated profile       # no network write, no real tools
      record metrics(change_id, pass/fail, cost)
      status = "evaluated"                               # NOT "applied"
  promote(change_id, human_approver):
      require status == "evaluated" and approver != author
      apply; record before_hash/after_hash; keep inverse
```

**Safety constraint.** The agent may create proposals and run evaluations; it may never call
`promote`. Promotion is an operator action with an explicit approver identity. Sandbox runs
use the existing daemon allowlist root, not the host filesystem.
**Effort:** L. **Auto-apply:** NO.

### P5. Separate `MetaProposer` capability from the root executor

**Why.** The paper fuses task and meta levels *inside an editable artifact*, which is exactly
what we must not do. The safe inverse is to separate them in the type system: a proposer that
can only emit diffs, and an executor that can only run the approved artifact. This borrows the
meta-level idea while refusing the self-edit.

**Before → after**

```text
BEFORE:
  root task can spawn, answer, and (conceptually) "orchestrate"
  no distinct meta role

AFTER:
  trait MetaProposer:
      propose(observation, archive_view) -> Vec<ChangeProposal>   # data only
      # no spawn, no tool execution, no store write except proposals

  root final synthesis uses executor.finalize_root (unchanged)
  meta pass (optional, post-run) uses MetaProposer -> proposals table
  apply path is P4's promote(), never the proposer
```

**Safety constraint.** `MetaProposer` implementation must not hold a `RemoteControl`,
`DaemonService`, or provider-with-tools handle; enforce with a constructor that takes only
read-only store views. A compile-time test asserts no spawn symbols in the module.
**Effort:** M. **Auto-apply:** NO.

### P6. Capability-freeze invariant + negative tests ("no self-modification path")

**Why.** The paper's central risk is an editable improvement procedure. Our safety claim
("HiveMind does not self-modify") is currently prose plus an audit inference. It should be a
tested invariant.

**Before → after**

```text
BEFORE:
  claim: no self-edit (audit G01, inferred from absence)

AFTER:
  test_no_runtime_self_edit:
      assert no production module writes: prompts/*.prompty, planner.rs, role tables,
             tool allowlists, provider config, or its own binary
      assert all writes to config/persona paths are operator-gated
      assert provider receives prompt text as data, never as a path it can rewrite
  CI gate: failure blocks merge
```

**Safety constraint.** Test and docs only; must not add a runtime "self-edit" feature to have
something to guard. If a legitimate config path exists, it is allowlisted explicitly with a
reason, not pattern-matched away.
**Effort:** S. **Auto-apply:** YES (test/doc only, no runtime behavior change).

### P7. Root action attribution: `actor = root`, not `operator`

**Why.** Human oversight is only real if the audit trail distinguishes human from agent
action. The taxonomy audit records that a root-spawned remote's audit actor is hardcoded
`operator` in the pinned commit (`remote.rs` audit note; apparent dirty-worktree fix). That is
the opposite of oversight: it makes an agent action look human.

**Before → after**

```text
BEFORE:
  create_remote(..., actor = "operator")        # agent-initiated looks human

AFTER:
  create_remote(..., actor_kind = "hyperagent", actor_id = root.id)
  every root tool call -> audit row (allowed|refused, reason, args_digest)
  operator actions keep actor_kind="operator"
```

**Safety constraint.** No behavior change to authorization; only attribution. Existing rows
are not rewritten (append-only correction row if needed). Actor id must be the task id, never
a display name.
**Effort:** S. **Auto-apply:** NO (touches audit semantics; docs portion may be YES).

### P8. Bounded self-critique artifact (reflection without tools)

**Why.** The paper improves the *process* that generates improvements. The safest borrowable
slice is a post-run critique that scores the plan/gates and writes an artifact — no tools, no
code changes, no automatic application. It creates the observation signal P3/P4 need.

**Before → after**

```text
BEFORE:
  run ends at final answer

AFTER:
  if critique_enabled and run terminal:
      prompt = run prompt + plan digest + outcome counts + gate decisions
      reply  = executor.execute_critique(prompt)   # no tools advertised, 1 call
      store.write_critique(run, {
        what_worked, what_failed, proposed_change (free text),
        confidence, evidence_refs
      })
      # never parsed into actions
```

**Safety constraint.** One provider call, no tools, max N chars, no raw child output beyond
the bounded digest; critique is stored as `unverified` and cannot trigger P4 automatically.
**Effort:** S. **Auto-apply:** NO (new provider call + schema; prompt change).

### P9. Cross-run memory promotion with trust tiers and TTL

**Why.** The paper's meta-level improvements "transfer across domains and accumulate across
runs". HiveMind has per-run/hive/branch banks (`store.rs:104-260`) but no explicit promotion
path from run-scoped to durable knowledge, and no expiry. Without one, "memory" is replay, not
learning; with a careless one, one bad run poisons every future run.

**Before → after**

```text
BEFORE:
  write -> run/hive/branch bank; read filter by trust label

AFTER:
  promote(entry, target_scope, evidence):
      require evidence.run_ids >= 2 and distinct outcomes
      require trust in {"verified","authoritative"} and human_approver for global
      write promotion row (source, target, evidence, expires_at)
  read: global entries past expires_at are ignored, never silently deleted
```

**Safety constraint.** No auto-promotion from a single run; global scope requires human
approval; promotions are reversible by tombstone; secrets/PII rejected at the write boundary.
**Effort:** M. **Auto-apply:** NO.

### P10. Golden-run regression harness for agent-surface changes

**Why.** The paper evaluates every variant against a suite before keeping it. Any change to
personas, tool allowlists, planner validation, or root gates should be measured against fixed
scenarios, including negative cases (spawn refused, budget exceeded, cancel mid-run).

**Before → after**

```text
BEFORE:
  unit tests per module; no end-to-end agent-surface suite

AFTER:
  golden_suite = [
    flat_plan_root_answers,
    hierarchical_plan_no_double_spawn,
    root_call_cap_enforced,
    depth_and_fanout_refusals,
    cancel_mid_dispatch,
    remote_grant_denied_without_grant,
  ]
  for scenario: run in isolated DB/profile; assert status, task count, refusal codes, audit rows
  record pass/fail + cost; block promotion (P4) on regression
```

**Safety constraint.** Test-only; fixtures use the existing test doubles permitted by the
contract (DB/HTTP/clock), never fake business logic; no production branch keyed on test mode.
**Effort:** M. **Auto-apply:** YES (test-only, no runtime path).

---

## 4. Borrowable safely now vs. research-only

### 4.1 Borrowable safely now (mechanism, not claim)

| Borrow | Why it is safe | Landing zone |
|---|---|---|
| Evaluation-before-promotion | Adds a gate; removes nothing | P4 |
| Append-only provenance / lineage | Uses existing hash-chain primitive | P2 |
| Archive as *data* (run specs, scores) | No executable artifact is stored | P3 |
| Bounded post-dispatch observation | One call, no new authority | P1 |
| Separation of proposer from executor | Type-level denial of self-edit | P5 |
| Attribution of agent vs. operator | Audit correctness only | P7 |
| Golden-run suite | Test-only | P10 |
| Trust-tiered memory promotion | Reversible, human-gated | P9 |

### 4.2 Research-only — do **not** fake

| Paper element | Why not now | What would be required |
|---|---|---|
| Editable meta-level procedure | Directly violates "no self-modification"; safety frame absent | Sandboxed code execution, eval harness, human review, rollback, threat model — none present |
| Self-modified agent variants | We have no fitness function, no isolation, no variant runtime | Full DGM-H loop + safety case |
| Open-ended archive selection across runs | No evaluator, no population, no selection policy | P3 + P4 + long-horizon eval |
| "Self-accelerating progress" | Not measurable in this system | External benchmark, multi-run study |
| Calling the root a "meta agent" | It does not modify anything | Rename or implement P5 + P4 honestly |

**Do not write** "HiveMind self-modifies", "HiveMind is a DGM-H", "the HyperAgent improves
itself", or "the archive evolves agents". The accurate sentence is: *HiveMind has a
run-scoped root orchestrator and an append-only memory journal; it borrows evaluation and
provenance ideas from HyperAgents, not self-modification.*

---

## 5. Top 5 proposals (ranked)

1. **P1 — Post-dispatch observation turn.** Biggest agency gain per unit risk: the root
   currently decides blind and its control results are discarded.
2. **P2 — Hash-chained plan/decision archive.** Reuses `store.rs:40-102`; makes every gate
   decision tamper-evident. Prerequisite for P3/P4.
3. **P7 — Root action attribution.** Cheapest governance fix; an agent action must never be
   logged as `operator`.
4. **P6 — Capability-freeze invariant + negative tests.** Converts "we don't self-modify"
   from prose into CI. Test-only, safe to land first.
5. **P4 — Promotion gate.** The paper's most transferable discipline: nothing becomes
   production behavior without an evaluation record and a human approver.

Sequencing: **P6 + P7 (S) → P2 (M) → P1 (M) → P3/P4 (M/L) → P5/P9/P10.**
P8 is optional and only useful once P3/P4 exist to consume its output.

---

## 6. Verification status of this note

- Quoted paper text: extracted with `pypdf` from the local v1 PDF, page 1; v2 page 1
  spot-checked for title/abstract. **Verified text; not independently diffed.**
- Code line refs: read in this session (`execution.rs`, `planner.rs`, `store.rs`, `daemon.rs`,
  `remote.rs`) plus the prior taxonomy audit. **Verified static; no runtime test run.**
- Claims about what HiveMind lacks (meta agent, self-edit, archive of variants) are
  **absence claims** from static reading; absence of a code path is strong but not a proof
  against dynamic loading. The taxonomy audit reaches the same conclusion independently.
- Nothing in this note was executed; no proposal was applied.
