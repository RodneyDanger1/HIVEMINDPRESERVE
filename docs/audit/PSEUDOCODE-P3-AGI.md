# PSEUDOCODE-P3-AGI - HiveMind, AGI, and the metalearning/curiosity lens

Status: audit deliverable, read-only pass. No product source was edited; no test was run.
Date: 2026-11-10. Workspace: `C:\Users\Riley\Desktop\hiverouter-v1`.
Sources: the four URLs in the task brief, plus local code and audit documents cited inline.
This document does not claim AGI and does not claim self-improvement. Section 7 states the
strongest claim that survives the evidence, and section 5.3 lists claims that are refused.

---

## 0. Scope, grade vocabulary, and what this pass did not do

### 0.1 What was done

- Fetched and read the four briefed sources: the Wikipedia AGI article (tertiary), and
  Schmidhuber's `metalearning.html`, `metalearner.html`, and
  `artificial-curiosity-since-1990.html` (primary, author-hosted).
- Read local evidence: `README.md`, `backend/crates/hiverouter-core/src/caps.rs`,
  `.../src/planner.rs` (public API only), `.../src/db.rs` (migrations),
  `docs/CODEMAP/18-HMAS-RESEARCH.md`, `docs/CODEMAP/09-RISKS-GAPS.md`,
  `docs/claims/CLAIM-REGISTER.json`, `audit/FINDINGS-REGISTER.md`.
- Produced one file, this one.

### 0.2 What was not done (and therefore cannot be claimed)

- No command was executed against the product. There is no runtime evidence in this
  document that any HiveMind feature works now. Statements about the product are
  **read** from source or from committed documents, not observed.
- No live model call, no test suite, no replay, no archive exists to inspect.
- The Wikipedia article is a tertiary source; it is used for framing only.
- Line numbers are given where the read tool returned them. Where only a section is
  known, the section is named instead of inventing a line.

### 0.3 Grade vocabulary

Adapted from the house convention already used in `docs/CODEMAP/18-HMAS-RESEARCH.md:4-5`,
extended with `INFERRED` as the brief requires.

| Grade | Means exactly |
|---|---|
| **VERIFIED** | Read in the named primary source at that URL, or observed in command output quoted in this workspace. This pass produced no execution-VERIFIED product claim. |
| **READ** | Read in this repository at the named `path:line` or named section. |
| **INFERRED** | Follows from a READ/VERIFIED item by reasoning stated in the row. Not directly observed; may be wrong if the citation is stale. |
| **UNVERIFIED** | Asserted somewhere, not proven here, or contradicted by another artifact. Do not build on it. |
| **REJECTED** | Asserted or implied, and refuted by the evidence in this document. |

Housekeeping note: the repo's own convention marks a primary-source read as `VERIFIED`
and a repo read as `READ`. This document follows that, so `VERIFIED` here never means
"we ran it". The only execution-grade claim class in this document is empty.

### 0.4 Source grades

| Source | Grade | Use in this document |
|---|---|---|
| `en.wikipedia.org/wiki/Artificial_general_intelligence` | READ (tertiary) | Framing, the contested-test list, and the "hypothetical" qualifier. Not treated as authority. |
| `people.idsia.ch/~juergen/metalearning.html` | VERIFIED | Metalearning vs transfer, SMP/SSA, OOPS, Godel machine, curiosity+Meta-RL. |
| `people.idsia.ch/~juergen/metalearner.html` | VERIFIED | Learning-to-learn definition; LSTM metalearning result (Hochreiter 2001). |
| `people.idsia.ch/~juergen/artificial-curiosity-since-1990.html` | VERIFIED | Curiosity as learning progress, information gain, compression progress, PowerPlay. |

---

## 1. AGI: a definition that can fail

### 1.1 Operational definition

A system `S` is **AGI** only if all six conditions hold. Each is necessary; none alone is
sufficient. The definition is deliberately written so that a system can fail it.

**D1 - Breadth under held-out evaluation.** For a task distribution `D` that `S` was not
trained on and cannot enumerate in advance, `S` reaches human-baseline or better on a
randomly drawn, externally scored sample of `D`. Scoring is by an evaluator that does not
share `S`'s model of success. Self-report is not evidence.

**D2 - Transfer not explained by surface similarity.** The performance in D1 survives
renaming, remapping, and re-embedding of the task interface. If gains vanish when the
surface form changes, the system memorised a surface, not a competence.

**D3 - Continual learning without catastrophic forgetting.** `S` improves from its own
experience, in place, without a full retrain and without a human curating the curriculum.
After learning task family B, performance on family A is retained within a stated bound.

**D4 - Calibrated self-model.** `S` predicts its own success and failure better than a
base-rate predictor, per task class, with intervals, and uses that prediction to accept,
defer, or decline work. A confidence number with no calibration evidence is not a
self-model.

**D5 - Open-ended goal generation.** `S` generates and pursues its own subgoals and
produces non-degenerate, non-repetitive progress that an external evaluator recognises as
progress. Novelty alone does not satisfy this: an agent that generates endless random
novel tasks has satisfied nothing.

**D6 - Safety envelope.** D1-D5 hold under bounded resources, auditable actions, and no
drift outside the operator's grant. A system that becomes general by escaping its caps has
failed the definition, not passed it.

### 1.2 Non-definition: what is not AGI

- **N1 - Breadth of training data.** A model that talks about many topics is not thereby
  general; topic coverage is a property of a corpus.
- **N2 - Tool count.** A large tool registry is a capability inventory, not transfer. Tool
  selection can be memorised per tool.
- **N3 - Multi-agent topology.** "Hierarchical", "swarm", "mesh", "crew" name execution
  shapes. A hierarchy with fixed roles is an orchestration pattern.
- **N4 - Editing prompts or configuration.** Changing a prompt, a temperature, or a JSON
  profile is hyperparameter adjustment. The metalearning source is explicit that
  "metalearning is sometimes confused with simple transfer learning" and that true
  metalearning is also "much more than just learning to adjust hyper-parameters"
  (`metalearning.html`, opening section). Config editing is neither.
- **N5 - Using an LLM.** An LLM is a component. Generality is a measured property of the
  assembled system, not a property inherited from a component.
- **N6 - Apparent self-improvement from a better model.** Swapping in a stronger model is
  an upgrade performed by an operator, not self-improvement.
- **N7 - Consciousness, understanding, or "true" reasoning.** Untestable with the
  instruments available here and orthogonal to D1-D6; asserting either direction is noise.
- **N8 - Passing one named test.** The Wikipedia article catalogues human-level AGI tests
  (Turing, IKEA, coffee, Suleyman, video games) as separate, contestable sections and
  separately catalogues "AI-complete" problems (article contents sections 2.3 and 2.4).
  A single benchmark pass is a data point, not a verdict.

### 1.3 Why the definition is written this way

The Wikipedia lead frames AGI as a **hypothetical** type of intelligence, and its own
structure treats the tests as a menu of contested proxies rather than a standard (READ:
article lead and contents). A definition that cannot fail is useless for an audit, so D1-D6
each name a measurement and each can return "no". The metalearning source supplies the
sharpest available distinction for D3/N4: true metalearning encodes the initial learning
algorithm in a universal language with self-modifying code, surrounded by a recursive
framework that keeps only useful self-modifications (VERIFIED: `metalearning.html`,
paragraphs 2-3). That is a much higher bar than "the system changed a file".

### 1.4 Could any current HiveMind artifact pass?

| Condition | Verdict | Why |
|---|---|---|
| D1 breadth, held out | No | Caps close the task space; no evaluated held-out distribution exists. |
| D2 transfer | No | Fixed tool allowlists and role taxonomy; nothing measures remapped interfaces. |
| D3 continual learning | No | No learning loop; see section 2 and the claim ledger rows 10-11. |
| D4 self-model | No | No calibrated predictor; UI state is descriptive, not predictive. |
| D5 open-ended goals | No | Goals arrive as operator prompts and a plan DAG. |
| D6 safety envelope | Partially, in the narrow sense | Caps, named refusals, and approval gates exist (section 2.1). This is the only row with a defensible positive, and it is about bounds, not generality. |

---

## 2. What HiveMind is, precisely

### 2.1 Architecture facts (READ)

| Fact | Evidence |
|---|---|
| Rust workspace, four crates (`hiverouter-core`, `-contract`, `-provider`, `-tools`), single localhost binary | `README.md` sections Layout and Status |
| Spawn depth max 3, fan-out max 8, agents per run 64, provider calls per run 64 | `backend/crates/hiverouter-core/src/caps.rs:9,11,13,18` |
| Caps refuse with named contract codes, never a silent clamp | `caps.rs` module doc ("a refusal is always the named contract code, never a silent clamp") |
| Plan is a validated DAG with `topo_order`, `leaf_order`, `aggregation_order` | `backend/crates/hiverouter-core/src/planner.rs` (public `Plan` API) |
| Persona record with tier `Hyper/Super/Worker/Tool` and tool allowlists | INFERRED: `18-HMAS-RESEARCH.md:38-43` cites `prompts.rs:340-369`; `prompts.rs` was not opened in this pass |
| Run shape: plan DAG -> tasks -> spawn -> terminal snapshot in SQLite, replay via `HISTORY_SNAPSHOT` revisions | `docs/CODEMAP/18-HMAS-RESEARCH.md:47-49` |
| Scoped key/value memory table `agent_memory` with run/scope/agent indexes (migration v7) | `backend/crates/hiverouter-core/src/db.rs:181-272` |
| Role-scoped memory recall, latest 8, injected into task context | INFERRED: `18-HMAS-RESEARCH.md:145-146`; not exercised here |
| Peer messaging `ask_parent` / `send_to_sibling`, `agent_messages`, 32-message per-run budget | INFERRED: `18-HMAS-RESEARCH.md:106-108,142-145` |
| No semantic retrieval, no HMAS controller, no per-run cap profiles | `18-HMAS-RESEARCH.md:158-159` |

### 2.2 Why "narrow hierarchical HMAS" is the correct label

HiveMind is a **hierarchical multi-agent system**: agents are tasks in a capped tree, a
parent spawns children, children return results, and a terminal snapshot closes the run.
It is **narrow** because five closure properties hold, and each is a limit, not an
implementation detail:

1. **Closed role set.** `Hyper/Super/Worker/Tool` with per-tier spawn and tool rights
   (INFERRED: `18-HMAS-RESEARCH.md:148-149`).
2. **Closed tool set.** A registry plus per-persona allowlists; a tool must resolve or the
   profile is refused (`18-HMAS-RESEARCH.md:72`).
3. **Closed budget.** Depth, fan-out, agents, and calls are compile-time constants
   (`caps.rs:9-18`); no adaptive component may raise them.
4. **Closed goal source.** A run begins with an operator prompt and a planner-produced DAG;
   the system does not invent runs.
5. **No cross-run learning.** Runs are recorded, not learned from; memory is an
   operator-gated key/value store (`18-HMAS-RESEARCH.md:145-146,152`).

A system with all five closures is an **orchestrator**. "Narrow HMAS" is not a criticism;
it is the accurate category, and it is the category in which the caps and refusals are
strengths.

### 2.3 Criterion-by-criterion gap

| AGI requirement | HiveMind today | What would have to exist |
|---|---|---|
| Transfer (D2) | Personas and tools are fixed at boot; a task class is routed to a role. | A measured remap test, plus a mechanism that changes behaviour when the surface changes. |
| Continual learning (D3) | Runs persist; memory is manual and role-scoped; no parameter or policy update. | An in-place update path with forgetting bounds, and a curriculum not curated by a human. |
| Self-model (D4) | Descriptive state machine; no prediction of own success. | A calibrated predictor with held-out calibration evidence (proposal P4). |
| Open-ended goals (D5) | Operator prompt + plan DAG. | Goal generation **and** a skill-acquisition path, plus a verifier. |
| Safety (D6) | Caps, named refusals, approval for Mutate-class calls (`18-HMAS-RESEARCH.md:154-156`). | The same envelope extended to any adaptive component, with a kill switch (P8). |

### 2.4 The two claims most likely to be overclaimed

**"HiveMind is self-improving."** False as stated. Nothing in the read material updates
the planner, the caps, the personas, or the policy from experience. A prompt edit is N4.
The metalearning source's own bar (self-modifying code plus a filter that retains only
useful modifications) is not met, and the source below shows why the nearest analogue,
OOPS, is a *search over programs*, not a config change (`metalearning.html` section 4).

**"HiveMind has memory, therefore it learns."** False as stated. It has a scoped key/value
table and an operator-facing clear path. Storage is a precondition for learning, not
learning. The repo itself lists "semantic retrieval" as not done (`18-HMAS-RESEARCH.md:158`).

---

## 3. Metalearning and curiosity: what the sources actually license

### 3.1 Five distinctions that decide what is honest here

1. **Metalearning vs transfer learning.** The source states plainly that metalearning "is
   sometimes confused with simple transfer learning" and that even a feedforward net can
   transfer-learn; true metalearning encodes the learning algorithm itself in a universal
   language (VERIFIED: `metalearning.html` opening).
2. **Progress, not surprise.** In stochastic environments, rewarding raw prediction error
   lets a controller get stuck on an unpredictable "noisy TV"; the fix is to reward the
   *first derivative* of error, i.e. learning progress (VERIFIED:
   `artificial-curiosity-since-1990.html` section 2). This is the single most useful
   warning for any "novelty routing" proposal: route to learnable novelty, not to noise.
3. **Information gain and compression progress.** Curiosity can be formalised as KL
   divergence between the world model before and after an observation, or as compression
   progress including model description length (VERIFIED: same page, sections 3 and 5).
4. **OOPS reuse is guarded.** OOPS spends part of its search budget testing programs that
   reuse earlier solutions; if reuse does not help, "at least the previous solutions will
   not cause much harm" (VERIFIED: `metalearning.html` section 4).
5. **PowerPlay verifies.** It searches for the simplest still-unsolvable problem and takes
   verification time into account (VERIFIED: same curiosity page, abstract/section 8).
   Goal generation without verification is not PowerPlay; it is noise generation.

### 3.2 Three mechanisms HiveMind could add now (concrete, safe)

Each is a **selection or bookkeeping** change. None changes the policy, the caps, or the
prompt path without an operator, and none is a learning algorithm.

**M1 - Episode archive (append-only projection).** One derived record per terminal run:
plan hash, per-task role/tier/state/refusal codes/attempts/usage, cost, terminal outcome,
provenance (binary version, model id, schema version). Read-only with respect to
execution; write-once; deletable by `run_id`. Safety property: it observes, it cannot
steer. This is the substrate every other mechanism needs.

**M2 - Lesson candidates (human-gated).** Cluster archived episodes by
`(refusal_code, role, plan_shape)`; at or above a support threshold, emit a **candidate**
with evidence pointers and a templated one-line claim. Candidates live in their own table
and are never read by prompt assembly. An operator may approve one into `agent_memory`.
Safety property: the only path from observation to behaviour runs through a human, and
the claim text is templated, not free-form model output.

**M3 - Progress-weighted novelty routing (order only).** Reorder the nodes of an
already-validated plan by predicted learning value: unpracticed, checkable, and stable
classes first; classes dominated by transient failures last (the noisy-TV guard). It may
not add, remove, or re-parent nodes, and it may not touch caps. Safety property: a bad
router degrades to the planner's topological order, and the ordering is logged with the
archive revision that produced it.

These three correspond to the honest parts of the literature: M1 is the record a
single-lifelong-trial system would need; M2 is OOPS-style reuse with a human in the loop
where the proof obligation would be; M3 is progress-weighted selection without pretending
to have a world model.

### 3.3 Three mechanisms that are not honestly implementable here

**N1 - True metalearning / recursive self-improvement.** Requires the learning algorithm
to be part of the modifiable system, a universal representation to modify it in, and a
retention filter (statistical, as in the success-story algorithm, or proof-based, as in the
Godel machine) that only keeps useful self-modifications (VERIFIED:
`metalearning.html` sections 2 and 5). HiveMind's policy is compiled Rust; caps are
`const`s; there is no self-modification channel. A prompt or config edit is
hyperparameter tuning, which the source excludes. Claiming otherwise would be the exact
category error the source warns about.

**N2 - A learned world model with a curiosity signal.** Requires a predictive model
trained on the system's own interaction stream, and an intrinsic reward computed as
learning progress, information gain, or compression progress (VERIFIED: curiosity page
sections 1-5). HiveMind has no environment model, no observation stream, and no training
loop; its "environment" is a provider API and a local database. Any number produced here
would be a hand-tuned heuristic using the vocabulary of a world model. M3 is the honest
subset: a heuristic that says so.

**N3 - Open-ended goal generation.** Requires generating well-defined problems, verifying
their solutions, and adding acquired skills to a repertoire (PowerPlay; VERIFIED: curiosity
page section 8). HiveMind's skills are fixed tool contracts; there is no acquisition
mechanism, so a goal generator would produce work the system cannot learn from. It also
collides with D6: goal generation without an approval path contradicts the operator-grant
model already enforced for cross-branch sends (`18-HMAS-RESEARCH.md:154-156`).

---

## 4. Proposals

Ranking (value x safety x implementability now):

| Rank | ID | Proposal | Depends on |
|---|---|---|---|
| 1 | P1 | Episode archive projection | - |
| 2 | P2 | Lesson candidates, human-gated | P1 |
| 3 | P6 | Counterfactual replay gate | P1 |
| 4 | P3 | Progress-weighted ordering | P1, P6 |
| 5 | P8 | Adaptive envelope + kill switch | - |
| - | P4 | Calibrated capability self-report | P1 |
| - | P5 | Refusal-signature plan reuse | P1, P6 |
| - | P7 | Claim-gate linter | - |

P7 is the cheapest and can ship independently; it is not ranked because it changes
documentation practice, not product behaviour.

Each proposal below uses the same shape: **Before** (read from the repo), **After**
(pseudocode), **Evidence gate** (what must be observed before the change may be claimed),
**Anti-overclaim** (the sentence that may be written and the sentence that may not).

### P1 - Episode archive

**Before.** Run facts are spread across `runs`, `tasks`, `snapshots`, `run_usage`,
`agent_messages`, and `agent_memory` (`db.rs` migrations, `18-HMAS-RESEARCH.md:47-49`).
Replay is a revision projection, not a per-run record. There is no single artifact that
answers "what ran, in what shape, at what cost, with what outcome".

**After.**

```text
ARCHIVE_EPISODE(run_id) -> Episode | Refusal
  pre: run.state in {TERMINAL, INTERRUPTED}          # never archive a live run
  pre: run.archived_at is null                        # write-once

  ep := {
    run_id,
    schema: 1,
    provenance: { binary_version, model_id, caps: {depth, fanout, agents, calls} },
    plan_hash: sha256(canonical(plan.nodes)),
    tasks: [ { task_id, parent_id, depth, role, tier, state,
               refusal_codes: [...], attempts, tokens_in, tokens_out,
               started_at, ended_at } ],
    outcome: { terminal_state, aggregation_ok, verifier: null | PASS | FAIL },
    prompt_refs: [ { template_id, prompt_sha256 } ]    # hashes only, not prompt text
  }

  # redaction is a refusal, not a silent scrub
  if SCAN_FOR_SECRETS(serialize(ep)) hits any token/key pattern:
      return Refusal(E_ARCHIVE_REDACTION)

  append(ep)                       # append-only; no update path
  set run.archived_at = now()

DELETE_EPISODE(run_id, operator):
  pre: operator present
  delete exactly the rows keyed by run_id; never a range delete
```

Invariants: derived, never authoritative; no prompt bodies (hashes and template ids only);
no mutation after write; deletion is scoped and audited.

**Evidence gate.**

1. Recompute `plan_hash` for ten archived runs from the source tables; all ten match.
2. Planted-fake-token test: archive refuses with `E_ARCHIVE_REDACTION`; the row does not
   exist afterwards.
3. Deletion test: delete a run; the episode and its task rows are gone; no other run is
   touched.
4. Reconciliation test: `count(episodes) == count(runs where state in terminal)` after a
   batch; mismatches are named, not rounded.

**Anti-overclaim.** May say: "we can reconstruct what ran and what it cost."
May not say: "HiveMind learns from experience", "memory of past runs improves it", or
"the system reflects". An archive is evidence of work done.

### P2 - Lesson candidates, human-gated

**Before.** `agent_memory` is a role-scoped key/value store; recall takes the latest 8;
there is a clear/delete path and a memory panel (`18-HMAS-RESEARCH.md:145-146,152`). A
repeated refusal teaches nothing: no distillation, no evidence link, no promotion path.

**After.**

```text
PROPOSE_LESSONS(window, min_support = 3) -> [Candidate]
  eps := ARCHIVE where ended_at in window
  groups := group_by(eps, key = (refusal_code, role, plan_shape_hash))

  for g in groups where count(g) >= min_support:
      if success_rate(g) >= threshold: continue
      cand := {
        id: hash(g.key, window),
        claim: TEMPLATE(g.refusal_code, g.role),      # templated, NOT model-authored
        support: [run_id ...],
        counterexamples: [run_id ...],
        proposed_action: one of
          { RETRY_WITH_BACKOFF, ESCALATE_TIER, SPLIT_TASK, NO_ACTION },
        status: CANDIDATE
      }
      store(cand)                                    # table: lesson_candidates
  return candidates

APPROVE(cand_id, operator):
  pre: operator present and cand.status == CANDIDATE
  write agent_memory { key: "lesson:<id>", value: cand.claim,
                       evidence: cand.support, scope: <operator-chosen> }
  cand.status := APPROVED

# HARD INVARIANT
PROMPT_ASSEMBLY reads agent_memory only.
PROMPT_ASSEMBLY never reads lesson_candidates.
```

The templated `claim` is the safety-critical choice: free-form model text is not allowed to
become a candidate claim, because it cannot be linted. `proposed_action` is a closed enum
with no "widen tool allowlist" member, so a candidate cannot propose its own privilege.

**Evidence gate.**

1. Static proof: the prompt-assembly module has zero references to `lesson_candidates`
   (grep plus a test that fails if the reference appears).
2. Support and counterexample counts recompute from the archive and match the candidate row.
3. Behavioural non-interference: 20 runs with candidates present but unapproved produce
   plans identical to the 20-run baseline (compare `plan_hash`).
4. Precision: hand-label 20 candidates; record precision and false-positive categories
   before any claim of usefulness.

**Anti-overclaim.** May say: "candidate lessons with evidence links and operator approval."
May not say: "the system learned", "self-improving memory", or "experience made it
better". No auto-approval path may be added under this proposal.

### P3 - Progress-weighted ordering (novelty routing)

**Before.** Node execution follows the planner order (`topo_order`, `leaf_order`,
`aggregation_order`; `planner.rs`). Selection has no signal beyond structure. Caps are
constants (`caps.rs:9-18`).

**After.**

```text
ORDER_TASKS(plan, archive_snapshot) -> [TaskId]
  pre: plan.validate() == Ok
  pre: archive_snapshot.revision is pinned for this run

  scored := []
  for t in plan.nodes:
      s := archive_snapshot.stats(role = t.role, class = t.class)
      p := laplace_success(s.success, s.attempts)        # smoothed, not raw
      v := verifiability(t)     # HIGH if outcome is machine-checkable, LOW if prose-only
      r := transient_failure_rate(s)                     # the noisy-TV guard
      value := (1 - p) * weight(v) * (1 - r)
      scored.append((t.id, value))

  # stable: ties keep planner order
  order := stable_sort(plan.topo_order(), by = -value, tie = planner_index)

  assert length(order) == length(plan.nodes)             # no node invented or dropped
  assert caps(plan) unchanged
  log { archive_revision, order, scores }                # auditable
  return order

# fallback
if archive_snapshot.coverage(plan.nodes) < 0.5:
    return plan.topo_order()                             # no evidence, no reordering
```

Why this is the honest curiosity analogue: the `(1 - r)` term is the noisy-TV guard from
the 1991 source (reward progress, not error), reduced to a selection weight. It is a
heuristic over recorded outcomes and is named as such.

**Evidence gate.**

1. Counterfactual replay (P6) over >= 30 archived runs: same plans, old vs new order;
   compare terminal success and calls used, paired per run.
2. Noisy-TV test: a synthetic class with a 100% transient failure rate is ranked last in
   100% of generated plans.
3. Determinism: identical `(plan, archive_revision)` yields identical order across 100
   evaluations.
4. Cap-delta test: depth, fan-out, agent count, and call estimate are byte-identical
   before and after ordering.

**Anti-overclaim.** May say: "task ordering is a heuristic over recorded outcomes."
May not say: "intrinsic motivation", "curiosity", "the system explores", "world model", or
"the planner learns". The planner's code is unchanged; only its input order differs.

### P4 - Calibrated capability self-report (narrow self-model)

**Before.** State is descriptive (`16-state` vocabulary per `README.md` Status and
`CLAIM-REGISTER.json:105` context). No component predicts its own success, and no UI
number carries a sample size.

**After.**

```text
SELF_REPORT(role, class, archive_snapshot) -> Report
  n := archive_snapshot.count(role, class)
  if n < MIN_N:
      return { n, p_hat: null, ci: null, basis: "insufficient evidence" }
  k := archive_snapshot.successes(role, class)
  return { n, p_hat: wilson_center(k, n), ci: wilson_interval(k, n, 0.95),
           basis: "archive:" + archive_snapshot.revision }

RENDER(report):
  if report.p_hat is null: show "no estimate (n=" + report.n + ")"
  else: show p_hat with ci and n; never a bare percentage
```

`null` rather than `0.0` when evidence is thin: the repo already treats NULL-not-zero as a
correctness rule for usage (`18-HMAS-RESEARCH.md:147`), and the same rule applies here.

**Evidence gate.**

1. Reliability diagram and Brier score on held-out archived runs, reported with `n`.
2. Baseline comparison: if the estimate is not better than the base rate for that class,
   the feature is shown as uninformative and disabled.
3. Null-propagation test: `n < MIN_N` renders "no estimate", never `0%` or `100%`.

**Anti-overclaim.** May say: "empirical success rate with a confidence interval for this
role and class." May not say: "the system knows what it can do", "self-aware",
"metacognition", or "it understands its limits". A frequency table is not introspection.

### P5 - Refusal-signature plan reuse (OOPS-lite)

**Before.** The planner emits a plan once; a refusal is terminal for that task; a
previously successful subplan shape is not reused across runs.

**After.**

```text
PLAN_WITH_REUSE(intent, archive_snapshot):
  base := existing_planner(intent)
  if base.validate() != Ok: return base

  cands := archive_snapshot.subplans_matching(
              signature(intent), min_success = 0.8, min_n = 3)
  for c in sort(cands, by = (-support, hash)):        # deterministic
      merged := graft(base, c)                        # same-shape subtree replacement
      if merged.validate() != Ok:            record(REUSE_REJECTED, "invalid"); continue
      if not within_caps(merged, base.caps): record(REUSE_REJECTED, "caps");   continue
      if not tools_subset(merged, persona.allowlist):
                                             record(REUSE_REJECTED, "tools");  continue
      return merged
  return base
```

Mirrors the OOPS property that unhelpful reuse must not harm: every rejection is recorded
with a reason, and the fallback is the base plan.

**Evidence gate.**

1. `validate()` passes on 100% of grafted plans, over a corpus of >= 100 generated plans.
2. Cap delta zero and tool-set delta zero, asserted per graft.
3. Replay A/B on >= 20 intents via P6; paired success and call-count deltas reported.
4. Rejection accounting: every `REUSE_REJECTED` carries one of the closed reason codes.

**Anti-overclaim.** May say: "plans may reuse a previously successful subplan shape."
May not say: "curriculum learning", "bias-optimal", "metalearning", or "the planner
improved itself". The planner binary is unchanged; only its inputs differ.

### P6 - Counterfactual replay gate

**Before.** `HISTORY_SNAPSHOT` replay is a read-only projection of a stored revision
(`18-HMAS-RESEARCH.md:47-49`). There is no harness that compares two policies on the same
recorded episode without new provider calls.

**After.**

```text
REPLAY_GATE(proposal, episodes, metric):
  for ep in episodes:
      for variant in [BASELINE, PROPOSAL]:
          obs := ep.recorded_observations
          if MISSING(obs, variant.needs):
              result[variant][ep] := SKIP(reason)       # never 0, never default
          else:
              result[variant][ep] := replay_pure(ep.plan, variant.policy, obs)
  report := paired_diff(result[BASELINE], result[PROPOSAL], metric)
  report.skipped := count(SKIP)
  if report.skipped > 0:
      report.verdict := PARTIAL                       # may not be presented as a result
  return report
```

The hard rule is skip accounting. A replay that silently treats a missing observation as a
zero is worse than no replay: it manufactures a comparison.

**Evidence gate.**

1. Baseline reproduction: five hand-checked episodes reproduce identical outcomes.
2. Injected regression: a deliberately worse proposal is caught by the gate.
3. Skip visibility: a run with one missing observation reports `PARTIAL`, and the verdict
   field refuses to say "improved".

**Anti-overclaim.** May say: "in replay, ordering X used N fewer calls on K episodes."
May not say: "this improves real runs" without a live A/B, and may not report a `PARTIAL`
report as a verdict.

### P7 - Claim-gate linter

**Before.** Claims live in `docs/claims/CLAIM-REGISTER.csv` (25 KB) and
`CLAIM-REGISTER.json` (80 claims; 72 VERIFIED, 8 READ, 0 UNVERIFIED per
`CLAIM-REGISTER.json:2-9`). Drift is found by later audits: stale trace counts and stale
register entries are recorded findings (`FINDINGS-REGISTER.md` A-F10, A-F24). Nothing
enforces form at write time.

**After.**

```text
LINT_CLAIMS(files):
  for claim in extract_claims(files):
      if claim.grade not in {VERIFIED, READ, INFERRED, UNVERIFIED}: fail("no grade")
      if claim.grade == VERIFIED and not has_evidence(claim):
          fail("VERIFIED without command+output or path:line")
      if claim.grade == UNVERIFIED and claim.in_summary_surface():
          fail("unverified claim promoted to a summary")
      if claim.text matches {AGI, self-improving, autonomous, curiosity, learns}:
          require claim.grade != INFERRED and has_evidence(claim)
  exit non-zero on any failure
```

**Evidence gate.**

1. Run against the current corpus. Expected: it fails on the known drift (A-F10/A-F24).
   A linter that finds nothing on a corpus with known drift is itself broken; record the
   baseline violation count.
2. After repair, violation count must be monotonically non-increasing per commit.
3. Negative test: a synthetic doc with an ungraded capability sentence must fail.

**Anti-overclaim.** May say: "claims carry a grade and an evidence pointer, and the linter
enforces the form." May not say: "the documentation is accurate". A linter checks form,
not truth; the ledger is only as good as the evidence behind each row.

### P8 - Adaptive-component envelope and kill switch

**Before.** Caps are compile-time constants (`caps.rs:9-18`). No adaptive component exists,
so no envelope exists either. Approval is already required for Mutate-class calls
(`18-HMAS-RESEARCH.md:154-156`).

**After.**

```text
ADAPTIVE(component, input, budget) -> Output
  pre: component.enabled == operator_flag
  pre: budget.calls_used + estimate <= caps.max_calls
  out := component.propose(input)          # a proposal, never a commit
  if not valid(out) or not within_caps(out):
      log(REFUSED, component.id, reason)
      return baseline(input)
  return out

KILL(component):
  operator sets enabled = false
  # every ADAPTIVE call now returns baseline(input); no migration, no data loss
```

Invariants: every adaptive component is a pure function of `(input, archive_snapshot)`,
has a named baseline it degrades to, and may write only to its own log table. No component
may write to prompt assembly, caps, or tool allowlists.

**Evidence gate.**

1. Kill-switch test: with the component disabled, output equals baseline on 100% of a
   fixed 100-case corpus.
2. Cap-invariant test: no component call changes the four cap values.
3. Write-scope test: component writes are confined to its log table (schema assert).

**Anti-overclaim.** May say: "an optional ordering component with a documented baseline
and a kill switch." May not say: "autonomous", "self-directed", or "adaptive system".

---

## 5. Claim ledger

Grades per section 0.3. Rows are the claims this document is willing to stand behind, plus
the ones it explicitly refuses.

### 5.1 Product claims

| # | Claim | Grade | Evidence | Note |
|---|---|---|---|---|
| 1 | HiveMind is a Rust workspace of four crates built as one localhost binary | READ | `README.md` Layout, Status | - |
| 2 | Spawn depth <= 3, fan-out <= 8, agents <= 64, calls <= 64 | READ | `caps.rs:9,11,13,18` | Constants, not config |
| 3 | Caps refuse with named contract codes, never a silent clamp | READ | `caps.rs` module doc | - |
| 4 | Plan is a validated DAG with topo/leaf/aggregation orders | READ | `planner.rs` public API | - |
| 5 | Persona tiers `Hyper/Super/Worker/Tool` with allowlists | INFERRED | `18-HMAS-RESEARCH.md:38-43` cites `prompts.rs:340-369` | `prompts.rs` not opened here |
| 6 | `agent_memory` table is scoped by run/scope/agent | READ | `db.rs:181-272` | Migration v7 |
| 7 | Memory recall is role-scoped, latest 8, into task context | INFERRED | `18-HMAS-RESEARCH.md:145-146` | Not exercised |
| 8 | Peer messaging exists with a 32-message per-run budget | INFERRED | `18-HMAS-RESEARCH.md:106-108,142-145` | Not exercised |
| 9 | Semantic retrieval is absent | READ | `18-HMAS-RESEARCH.md:158` | - |
| 10 | No HMAS controller, no per-run cap profiles | READ | `18-HMAS-RESEARCH.md:158-159` | "Do not claim HMAS complete" |
| 11 | "Memory/research/learning loops absent; no product code or tables" | UNVERIFIED | `09-RISKS-GAPS.md`, Product/runtime gaps item 8 | Contradicted by row 6; treat as stale doc |
| 12 | Claim register counts: 80 total, 72 VERIFIED, 8 READ, 0 UNVERIFIED | READ | `CLAIM-REGISTER.json:2-9` | Machine-generated counts |
| 13 | Trace totals: 1 WIRED-AND-WORKS, 49 UNVERIFIED, 264 PENDING, 5 MISSING of 319 | READ | `CLAIM-REGISTER.json:105` | - |
| 14 | At audit HEAD, the shipped server could not perform a live model run | READ | `FINDINGS-REGISTER.md` A-F01 | Historical P0 at that HEAD; see conflict 5.4 |
| 15 | True SSE streaming is real in the sealed 1.1.0, with a paid smoke log | UNVERIFIED | `README.md` Status claims; `docs/evidence/live-1.1-paid.log` not opened | Claim is plausible but unread here |
| 16 | HiveMind is AGI, or on an AGI path | REJECTED | Section 1.4: D1-D5 all fail | No measurement exists |
| 17 | HiveMind is self-improving | REJECTED | Section 2.4; no update path from experience | Prompt/config edits are N4 |
| 18 | HiveMind has curiosity or intrinsic motivation | REJECTED | No world model, no progress signal (section 3.3 N2) | - |
| 19 | HiveMind performs metalearning | REJECTED | No self-modifiable learning algorithm (section 3.3 N1) | - |
| 20 | The M3 ordering is curiosity | UNVERIFIED by construction | Proposal P3 | It is a heuristic; name it as one |
| 21 | An episode archive exists | NOT IMPLEMENTED | Proposal P1 | No table, no code |
| 22 | Lesson candidates exist | NOT IMPLEMENTED | Proposal P2 | No table, no code |

### 5.2 Literature claims

| # | Claim | Grade | Evidence |
|---|---|---|---|
| 23 | AGI is characterised as hypothetical; human-level tests are catalogued as separate contested proxies; "AI-complete" problems are catalogued separately | READ (tertiary) | Wikipedia article lead, contents 2.3 and 2.4 |
| 24 | True metalearning encodes the initial learning algorithm in a universal language with self-modifying code, inside a framework that retains only useful modifications | VERIFIED | `metalearning.html` opening paragraphs |
| 25 | Transfer learning and hyperparameter tuning are explicitly not true metalearning | VERIFIED | `metalearning.html` opening paragraphs |
| 26 | SMP/SSA: a single lifelong trial, no resets, credit assignment across self-modifications | VERIFIED | `metalearning.html` section 2 |
| 27 | OOPS reuses prior solution programs, and unhelpful reuse must not cause harm | VERIFIED | `metalearning.html` section 4 |
| 28 | The Godel machine rewrites itself only after a proof of usefulness | VERIFIED | `metalearning.html` section 5 |
| 29 | Curiosity as adversarial controller/world-model (1990); progress rather than error in stochastic environments (1991); information gain (1995); compression progress (2006) | VERIFIED | `artificial-curiosity-since-1990.html` sections 1, 2, 3, 5 |
| 30 | PowerPlay searches for the simplest still-unsolvable problem and accounts for verification time | VERIFIED | same page, abstract and section 8 |
| 31 | LSTM-based metalearning produced fast online learners on frozen weights (Hochreiter 2001) | VERIFIED | `metalearner.html` closing section |

### 5.3 Refused claims (explicit)

The following sentences are refused by this document and must not be derived from it:

- "HiveMind is AGI", "HiveMind is proto-AGI", "HiveMind is AGI-adjacent", "HiveMind is on
  the path to AGI".
- "HiveMind is self-improving", "HiveMind improves itself", "HiveMind evolves".
- "HiveMind learns from experience", "HiveMind has continual learning".
- "HiveMind has curiosity / intrinsic motivation / a world model".
- "HiveMind performs metalearning", "HiveMind is a metalearner".
- "The archive makes HiveMind intelligent", "memory makes it learn".
- "The ordering is emergent", "the system explores autonomously".

### 5.4 Internal conflicts found (do not resolve by assertion)

1. **Memory existence.** `09-RISKS-GAPS.md` item 8 says memory loops are absent with "no
   product code or tables"; `db.rs:181-272` contains the `agent_memory` migrations. The
   likely reading is that the doc predates the memory work, but this pass did not date the
   two artifacts. Grade both rows, do not average them.
2. **Live model run.** `FINDINGS-REGISTER.md` A-F01 records a P0 at audit HEAD: the server
   could not reach the provider. `README.md` Status claims a sealed 1.1.0 with real
   streaming and a paid smoke. Both can be true at different commits; neither is verified
   here. This is exactly the drift that proposal P7 is meant to stop.
3. **Stale claims.** `FINDINGS-REGISTER.md` A-F10 and A-F24 already record trace-count and
   register drift. Rows 12-13 of this ledger are only as current as those files.

---

## 6. What would change the verdict

Cheap, falsifiable checks, in order. None is performed here.

1. **Archive determinism (P1):** recompute plan hashes for existing runs. If they do not
   recompute, P1's premise is wrong and P2/P3/P4/P5 lose their substrate.
2. **Counterfactual lift (P6, P3):** paired replay over >= 30 runs. If the ordering shows no
   paired improvement, M3 is decoration and should be dropped, not defended.
3. **Calibration (P4):** Brier score versus base rate. If it does not beat base rate, there
   is no self-model and the claim stays at "frequency table".
4. **Non-interference (P2):** candidates present but unapproved must not change a single
   `plan_hash`. If they do, the gate is broken and the mechanism is unsafe.
5. **Live capability (row 14/15):** one recorded live run with the provider, quoted command
   and output, at a named commit. Until then, no product capability claim in this area
   moves above UNVERIFIED.

---

## 7. The strongest non-claim

> **HiveMind does not learn. Not in the metalearning sense, not in the continual-learning
> sense, and not in the operational AGI sense. It executes a capped, single-provider
> hierarchical plan graph and records it. The mechanisms proposed here are observation,
> ordering, and bookkeeping around that execution; the archive they create is evidence of
> work done, not a substrate for self-improvement.**

This is the strongest statement the evidence supports, and it is a statement about what is
absent. The strongest *positive* statement is narrower: HiveMind is a local-first,
capped hierarchical multi-agent orchestrator whose hierarchy, refusals, and approval gates
are real engineering assets, and whose distance from AGI is currently measurable in every
one of D1-D5, not merely unproven.

---

## Appendix A - Review checklist for this document class

- [ ] Every capability sentence carries a grade and an evidence pointer.
- [ ] No sentence uses AGI, self-improvement, curiosity, or metalearning as an achieved
      property; at most as a defined target or an explicitly refused claim.
- [ ] Proposals state a baseline they degrade to and a kill switch where they change order.
- [ ] Every proposed mechanism that reads history says whether that history can influence
      behaviour without an operator. If yes, it must not be "safe now".
- [ ] Skip/refusal counts are reported, never rounded to zero.
- [ ] Conflicts between documents are listed, not silently resolved.
- [ ] The document states what it did not do. This one did not run the product.
