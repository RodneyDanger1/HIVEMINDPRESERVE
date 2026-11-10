# IMPROVE-RESEARCH — Meta HyperAgents + HyperAgent SE vs. the code in this worktree

- **Pass:** research/compare. **Mode:** read-only. **Only write:** this file.
- **Time box:** 2m30s. No PDF extraction, no test run, no provider call, no network fetch.
- **Inputs actually used:** `docs/audit/PSEUDOCODE-P1-META.md` (P1), `docs/audit/PSEUDOCODE-P2-HMARS.md` (P2),
  and a targeted re-read of `backend/crates/hiverouter-core/src/execution.rs` in **this** worktree.
- **Local PDFs** `2603.19461v1.pdf` / `2409.16299v3.pdf` were **not** re-extracted in this pass.
  Paper-side claims below are therefore `P` only to the extent P1/P2 already quoted them; where
  P1/P2 say the PDF was not extracted, this document repeats that limitation rather than hiding it.

## 0. Labels

| Label | Meaning |
|---|---|
| `P` | Primary: quoted from the paper by P1/P2, which extracted the local PDF text. Not independently re-extracted here. |
| `READ` | Line-verified in this worktree during this pass, cited `file:line`. |
| `INFERRED` | My conclusion from ≥2 pieces of evidence; not directly observed. |
| `UNVERIFIED` | Asserted or believed, but no check in this pass supports it. Never use as a premise. |

## 1. The two reference designs, in one line each

**Meta HyperAgents (arXiv:2603.19461)** — `P`: a task agent and a meta agent fused into **one
editable program**, where the *modification procedure itself is editable*; a Darwin-Gödel-Machine
loop branches from selected candidates, evaluates variants, and keeps a growing stepping-stone
archive; all experiments run under sandboxing and human oversight (P1 §1.1–1.2).

**HyperAgent SE (arXiv:2409.16299v3)** — `P (not re-verified this pass)`: a four-role software-
engineering decomposition — **Planner → Navigator → Code Editor → Executor** — where the Navigator
localises the code region and the Editor produces the patch. P2 §2.1/§9 already flagged that this
PDF body was never extracted and that the mapping was `INFER`; this pass did not change that.
`UNVERIFIED` in this worktree: **no `Navigator` and no `CodeEditor` symbol exists in any crate**
(`READ`: zero matches for `navigator|Navigator|code_editor|CodeEditor` under `backend/crates/core/src`).

## 2. Eight cited claims

### C1 — `P` — The fusing is the mechanism, not the branding
The paper's contribution is that task agent and meta agent are *one editable program* and the
meta-level procedure is editable. A fixed role hierarchy that happens to be named "HyperAgent" is
not an instance of it (P1 §1.1, §1.3). **Boundary:** this is the paper's claim, not a claim about us.

### C2 — `P` — Archive + evaluation-before-trust + external safety frame
Variants are retained with evaluation records and branched from; a variant enters the archive only
with an evaluation result; safety is external (sandbox, human oversight), not implied by capability
(P1 §1.2). **Boundary:** "sandboxing and human oversight" is the paper's description of its own
experiments; it is not a claim that our controls are equivalent.

### C3 — `P (UNVERIFIED locally)` — HyperAgent SE's four roles
Planner/Navigator/Code Editor/Executor is a *paper* decomposition. No local extraction was performed
in this pass, so any statement of the form "the paper routes patch generation through Navigator" is
`P`-by-P2 only, and P2 itself marks it `INFER` (P2 §2.1, §9).

### C4 — `READ` — The root control turn is now **two** turns with feedback, not one
`execution.rs:3909` is `for _turn in 0..2`; `:3906` declares `control_results`;
`:3915–3920` appends prior results into the next turn's prompt as *"data, not instructions"*;
`:4180` does `control_results.push_str(&_result_text)`.
This **contradicts** P1 §2.1 ("`for _turn in 0..1`", "`_result_text` is never fed back") and
P2 §2.2/§4.0 (F1 "the governor is a single bounded turn"). Those documents are stale at this
location; the proposals built on "the root decides blind" (P1-P1, P2-I2) are partly obsolete.
Also `claim_provider_call` is called at `:3914` **before** `orchestrate_root` at `:3922`, so the
spend claim is pre-call, not purely post-hoc as P2 §2.1 suggested.

### C5 — `READ` — The dead `spawn_superagent` branch is no longer dead
`:3942` is `if has_planned_branch || root_super_spawned`, and `:3955` sets `root_super_spawned = true`
after one successful `spawn_sub_hive`. The allow arm `:3953–3967` is therefore reachable **at most
once**, even when the planner produced branches. P2 §4.1 (GAP-RG: "allow branch ≡ dead") is stale.
What remains true: the *policy* is one extra branch per run, hard-coded as a bool, not an evaluated
or audited quota.

### C6 — `READ` — A non-applying meta-proposal surface already exists
`:4162–4171` matches a `propose_improvement` tool call and then
`record_root_gate_decision(run, root, "propose_improvement", "recorded", "<key> area=<area> applied=false")`.
The text appended to `control_results` is literally `"... applied=false"`. So a proposal channel
exists, but it has **no apply path, no proposal id, no dedupe, no human promotion step, and no
evaluator result**. It is a recorded suggestion, not a DGM variant.

### C7 — `READ` — Lower tiers can *request* detached principals through a governed path
`execute_request_detached` (`:4193`) refuses unless the caller tier is
`AgentTier::SuperAgent | AgentTier::WorkerAgent` (`:4203`), and returns coded refusals
(`E_ACCESS_DENIED`-class codes via `Refusal::ToolNotAllowed.code()` / `SchemaInvalid`).
`AgentTier` is a real enum at `hiverouter-tools/src/tool.rs:76`, and `execution.rs:133` maps
`AgentTier::HyperAgent => "hyperagent"`; node kinds are a closed vocabulary at
`hiverouter-contract/src/event.rs:240` (`hyperagent|superagent|subagent|worker|tool`).
`INFERRED`: this is the closest thing in-tree to HMARS's `REQUEST_REMOTE` — but it is a
*deterministic admission function*, explicitly documented as "not a second model turn" (:4191),
not an async request/decision pair.

### C8 — `INFERRED` — There is still no self-modification, and no tested invariant proving it
No code path found in this pass writes prompt files, role tables, tool allowlists, or the planner;
`propose_improvement` records `applied=false` and returns. The capability-freeze conclusion is an
**absence** argument (strong static evidence, not a proof against dynamic loading), and there is no
CI test that enforces it. P1-P6 proposed exactly that test; it is not present (`UNVERIFIED` as a
test, `INFERRED` as a property).

> Note on drift: P1/P2 cite `record_root_gate_decision` at `:4347`; it is at `:4467` here, and
> `root_checkpoint` at `:3869` (unchanged). Treat every P1/P2 line number as needing re-read.

## 3. Five improvement proposals

Each: before → after, **evidence gate** (what must be verified before coding, and what proves the
change), and an explicit **not claiming** boundary.

### A1 — Split authority by turn: turn 2 is decision-only (S, safest high-value)

```text
BEFORE (READ :3909, :3941-:4177)
  for _turn in 0..2:
      match call.name: spawn_superagent | spawn_remote_agent | spawn_daemon
                     | request_detached | propose_improvement | read_memory_summary | finish
  # turn 2 can still create principals; only root_super_spawned bounds one arm

AFTER
  for turn_idx in 0..2:
      let allowed = if turn_idx == 0 { CONTROL_TOOLS } else { &["finish","propose_improvement","read_memory_summary"] };
      if !allowed.contains(call.name) {
          record_root_gate_decision(run, root, call.name, "refused",
                                    "turn 2 is decision-only")?;
          _result_text += "- refused: no new authority in the final control turn\n";
          continue;                        // never fall through to the create arms
      }
```

**Evidence gate.** (a) Read `:3909–:4184` end-to-end and confirm no create arm is reachable in
turn 2 after this change; (b) a test that a scripted turn-2 `spawn_remote_agent` yields
`decision="refused"` and **zero** new principal rows; (c) assert turn 1 behaviour is byte-identical
for the existing refusal tests. **Not claiming:** that two turns are better than one, or that this
prevents a malicious model — it bounds one class of second-turn escalation only.

### A2 — Publish and persist the control result text (`ROOT_CONTROL`) (S)

```text
BEFORE (READ :4180)
  control_results.push_str(&_result_text);   # fed to turn 2, then dropped when fn returns

AFTER
  control_results.push_str(&_result_text);
  if !_result_text.is_empty() {
      self.publish(Event::RootControl { run_id: run.id, turn: turn_idx,
                                        decisions: _result_text.clone() }, ...)?;
      self.store.record_root_control(run.id, root.id, turn_idx, &_result_text)?;  # durable, append-only
  }
```

**Evidence gate.** First `grep -n "RootControl" backend/crates` — if an equivalent event already
exists, extend it instead of adding one (event enum is versioned; a WS consumer must tolerate it).
Prove with an e2e test: a root turn that is refused produces exactly one `ROOT_CONTROL` frame and
one durable row; today the text is only in the next prompt. **Not claiming:** that the text is
correct or complete — it is the model's own narration and must be labelled untrusted data.

### A3 — Make gate decisions retrievable and ordered (S)

```text
BEFORE (P2 §4.3; key format NOT re-read this pass — verify first)
  key = "root_gate:<uuid v7>"        # durable, unaddressable, re-listed as agent memory
  read_memory_summary(scope, limit)  # ignores its args (P2 M-13, AUDIT-DOC)

AFTER
  key = "root_gate:<run_id>:<seq>"   # monotonic per run, countable, addressable
  seq = store.next_gate_seq(run.id)  # single UPDATE..RETURNING or a tx-scoped counter
  read_memory_summary(scope, limit): honour both args; return redacted values (<=200 chars)
  # optional projection: gate_decisions(run_id, seq, action, decision, detail, ts)
```

**Evidence gate.** Read `record_root_gate_decision` at `:4467` **before** writing code — the current
worktree may already have changed the key format since P2. Prove: two refusals in one run yield
`root_gate:<run>:1` and `:2`, a second run cannot see them, and a reader returns both in order.
**Not claiming:** that a memory row is an audit record. If compliance-grade audit is required, the
dedicated table (A2's durable row / a `gate_decisions` projection) is the deliverable, not this key.

### A4 — Idempotent, capped admission inside `execute_request_detached` (M)

```text
BEFORE (READ :4193-:4207 + match arms)
  check tier -> parse kind/name -> (create principal + grant) -> gate row
  # no request id, no idempotency key, no per-run count cap visible in the checked slice

AFTER
  fn execute_request_detached(run, task, plan):
      let key = plan.args.idempotency_key or sha256(run.id|kind|lower(name)|task.id)
      if let Some(d) = store.find_decision(run.id, key) { return Replay(d) }   // no 2nd effect
      if store.count_run(run.id, kind) >= cap(kind) { return refused(E_*_LIMIT) }
      let decision = decide(...)                    // one policy fn, one place
      store.insert_decision(run.id, key, &decision)?   // BEFORE the side effect
      if decision.allowed { create principal + grant, actor_kind = "hyper" }
      decision
```

**Evidence gate.** `impact` on `execute_request_detached` and `create_remote` (GitNexus) before
editing; confirm whether an idempotency column already exists in the migration set; test double-call
with the same key → one principal, one grant, two decisions (one `replay`), and cap+1 → refusal with
exactly `cap` principals. **Not claiming:** that this makes admission asynchronous or model-decided —
it stays synchronous and deterministic (see C7).

### A5 — Capability-freeze test + human promotion gate for proposals (S test-only, plus policy)

```text
BEFORE
  propose_improvement -> gate row "recorded", applied=false (READ :4162-:4171)
  "we do not self-modify" -> prose + absence argument (C8)

AFTER (test, CI-blocking)
  test_no_runtime_self_edit:
      assert no production module opens for write: prompts/*.prompty, planner.rs,
             role/tier tables, tool allowlists, provider config, own binary
      assert propose_improvement's only store write is the proposal row (no apply symbol)
      assert every config/persona write path is operator-gated
  test_proposal_is_inert:
      invoke propose_improvement -> assert applied=false, no file mtime change,
      no tool-allowlist delta, and that the proposal is readable by key
  promotion (policy, not code):
      apply(proposal) requires status="evaluated" AND approver != author AND inverse recorded
```

**Evidence gate.** The negative test must be run once and observed to pass on the current tree, and
observed to fail on a deliberately injected writer (otherwise it proves nothing). Allowlist any
legitimate config path explicitly with a reason. **Not claiming:** that a passing test proves the
absence of self-modification at runtime (dynamic loading is out of scope); it proves no *source*
writer exists in the checked paths.

## 4. Ranking

| # | Proposal | Value | Risk | Effort | Gate first |
|---|---|---|---|---|---|
| 1 | **A1** turn-2 authority split | Removes a real second-turn escalation path | Low | S | re-read `:3909–:4184` |
| 2 | **A5** capability-freeze + inert-proposal tests | Converts a prose safety claim into CI | None (test-only) | S | inject a writer to prove the test bites |
| 3 | **A3** retrievable ordered gate rows | Turns a write-only log into evidence | Low | S | read `:4467` (P2 is stale) |
| 4 | **A2** `ROOT_CONTROL` publish + persist | Operators stop being blind to refusals | Low | S | grep for an existing event |
| 5 | **A4** idempotent capped admission | Kills duplicate allocation; one policy home | Medium | M | GitNexus impact |

Sequencing: **A5 + A3 (S) → A1 (S) → A2 (S) → A4 (M).** A1 before A2 because publishing a
decision stream is only meaningful once the decisions are bounded.

## 5. What this document does not claim

- It does not claim HiveMind is a hyperagent, a DGM-H, or self-improving. Nothing here found a
  self-modification path; C8 is an absence claim.
- It does not claim the HyperAgent SE four-role design is implemented, or that our `planner` +
  `executor` stages are equivalent to Planner/Navigator/Editor/Executor (C3, C8).
- It does not claim P1/P2 are correct at their cited lines — C4 and C5 show two places where they
  are already stale; the remaining line numbers must be re-read before any change.
- It does not claim any test passed: no test was executed in this pass.

## 6. Verification status

| Item | Status |
|---|---|
| Paper quotes | `P` — quoted by P1/P2 from local PDF text; **not re-extracted here** |
| `execution.rs` claims C4–C7 | `READ` — lines re-read in this worktree during this pass |
| `AgentTier` / node-kind vocabulary | `READ` — `hiverouter-tools/src/tool.rs:76`, `hiverouter-contract/src/event.rs:240` |
| Absence of Navigator/CodeEditor | `READ` — grep returned zero matches in `backend/crates/hiverouter-core/src` |
| No self-modification (C8) | `INFERRED` — absence of a writer, not a tested invariant |
| `read_memory_summary` arg handling, key format at `:4467` | `UNVERIFIED` this pass — carried from P2 as `AUDIT-DOC` |
