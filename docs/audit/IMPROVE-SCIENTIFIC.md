# IMPROVE-SCIENTIFIC — Claim Levels and Evidence Sufficiency for HiveMind

**Date:** 2026-11-10
**Mode:** read-only audit. No source edited, no test run, no paid provider call.
**Inputs read:** `docs/audit/HYPERAGENT-TRUTH-REPORT.md` (full), `docs/audit/AGENTIC-LIVE-TESTS.md`
(full), `docs/audit/PSEUDOCODE-P3-AGI.md` (§§0–1, 5.1–5.4, 6, 7), `backend/crates/hiverouter-core/src/caps.rs`
(lines 1–70), `docs/evidence/live/` (file listing).
**Answer to the brief's question:** the live tests support a *narrow engineering* claim and do
**not** support any stability, scale, autonomy, or learning claim. All AGI/self-improvement
claims are rejected, not merely unproven.

---

## 1. Claim-level vocabulary (the six levels used below)

The six levels are ordered by evidential strength. Each row of §2 carries exactly one.

| L | Level | Means exactly | Cannot be upgraded by |
|---|---|---|---|
| **L1** | **Observed fact** | Recorded in a run artifact (`docs/evidence/live/*.json`): the run happened, with these ids, counts, tokens, timestamps. | A second run does not strengthen it; it only widens it. |
| **L2** | **Direct test evidence** | A deterministic, re-runnable test asserts the property and passes (unit/integration). Proves *code path*, not *live model behaviour*. | Live runs at n=1. |
| **L3** | **Inference** | Follows from L1/L2 by stated reasoning, not directly instrumented. Example: "0 s and 0 tasks ⇒ no provider call". | Repetition of the same inference. |
| **L4** | **Hypothesis** | A falsifiable proposition with a named test that has **not** been run. Must carry its falsifier. | Assertion, plausibility, or design intent. |
| **L5** | **Speculation** | Narrative with no proposed discriminating test; may be interesting, is not evidence. | Any amount of prose. |
| **L6** | **Unknown** | Not measured, or measurement attempted and inconclusive. Explicitly not a low grade of "true". | Silence. |

**Rule adopted:** an `n = 1` live run is **L1**, never **L2** and never a stability claim. A
test that asserts a *guard* is **L2** for the guard, **L6** for the governed behaviour under a
real model.

---

## 2. Bounded claim table (10 rows)

`n` = independent live trials. `Code` = governing source. `Verdict` = whether the claim, **as
written**, survives.

| # | Claim (bounded wording) | L | Evidence | n | Verdict |
|---|---|---|---|---|---|
| C1 | A single exact-two prompt produced 7 tasks: 1 hyper d0 / 2 super d1 / 2 agent d2 / 2 subagent d3, 15,551 tokens | L1 | `live-tokenlab-hierarchy.json`; TRUTH T5 | 1 | **Holds** — as one observation of topology |
| C2 | Exact-two topology is **stable** at 3/3 | L6 | `live-tokenlab-exact-two-stability.json`; TRUTH T21 | 3 | **Fails** — 2/3. 2/3 at n=3 gives a 95% CI of ≈[0.09, 0.99]; no stability claim is licensed |
| C3 | `ask_parent` yields one durable message, `attempts=1`, `answer_actor=parent:<super>` | L1 | `live-tokenlab-a2a.json`; TRUTH T6 | 1 | **Holds** — for that one message |
| C4 | Parent-after-child ordering is never violated | L4 | AGENTIC-LIVE-TESTS §B "what was wrong": one SuperAgent said it had no children | 1 | **Not established** — the doc itself infers ordering from readiness code, not from an assertion |
| C5 | Root control turn created a detached remote with 1 device + 1 grant, actor `hyper:<root>`, audited | L1/L2 | `live-tokenlab-remote-daemon.json` + `root_control` audit test (TRUTH T10/T12) | 1 | **Holds** — as fixture record + audit row |
| C6 | The daemon **runs** (ticks, executes its `{"check":"db"}` job) | L6 | TRUTH T19: "enrolled record; no live tick proof" | 0 | **Unknown** — "enrolled" is a DB state, not execution |
| C7 | Lower-level `request_detached` passes a deterministic admission gate and creates Lantern (device+grant, `hyper_admission decision:allow`) | L1/L2 | `live-tokenlab-detached-request.json`; `lower_level_request_detached_*` tests | 1+2 | **Holds** — and is correctly *not* an async LLM approval |
| C8 | The rate governor refuses **before any provider spend** | L3 | `live-tokenlab-rate-limit.json` (391 B) — `E_LLM_RATE_LIMIT` at 0 s, 0 tasks | 1 | **Inference only** — elapsed-time and task-count are proxies; no wire-attempt counter was read |
| C9 | Hard kill retains measured usage (449 total: 319/130) and recovers 7 tasks as `interrupted` | L1 | `live-tokenlab-kill-usage.json`; TRUTH T3 | 1 | **Holds** — for one kill at one value; no second kill, no kill during dispatch |
| C10 | Caps are enforced at their **stated bounds** (depth 3, fanout 8, agents 64, calls 64) | L2 | `caps.rs:9-18,52-70`; `check_spawn`/`check_calls` refusals | unit | **Partially holds** — unit-level only. Live runs exercised 4 of 4 depth levels but only fanout 2, 7 of 64 agents, ~4 of 64 calls |

### 2.1 Claims explicitly rejected (not downgraded — rejected)

| Rejected claim | Why the evidence refutes it, not merely fails to support it |
|---|---|
| "HiveMind is AGI / proto-AGI / on the AGI path" | D1–D5 have **no measurement**, and D2/D3/D5 are structurally absent: no weight mutation, no episode archive, no goal generator. A system with no update path cannot be "on the path" by proximity. P3 §1.4, §5.3. |
| "HiveMind is self-improving / evolves" | `propose_improvement` writes `applied:false`, `requires_human_review:true` (TRUTH §2, §7). There is no code-write path from the tool to the running binary. Proposal ≠ improvement. |
| "HiveMind learns from experience / continual learning" | No training loop, no corpus, no weight mutation (TRUTH T22). Memory is *storage*; retrieval is not learning. P3 §7. |
| "HiveMind has curiosity / a world model / metalearning" | No progress signal, no self-modifiable learning algorithm. Prompt/config edits are N4 (hyperparameter adjustment), which the primary source explicitly excludes. P3 §1.2 N4, §5.3. |
| "Network satellite / remote agent" | Transport is `in_process` fixture only (TRUTH T18). The remote has a *narrow grant* and a *device row*, not a socket. |
| "40-node / HMARS scale" | Never run. Caps *allow* it; allowance is not evidence (TRUTH T20). |

**The strongest surviving positive claim:** HiveMind is a local, capped, single-provider
hierarchical orchestrator whose hierarchy, refusals, audit rows, and admission gate are real
and tested engineering. That is a *governance* claim, not a *cognition* claim.

---

## 3. Why the current live tests cannot carry the claims loaded onto them

1. **n is too small, and the one repeatable claim already failed once.** C2 is the only claim
   that was *tested for stability*, and it returned 2/3. A 2/3 result is not "mostly stable";
   at n=3 the confidence interval spans almost the whole unit interval. The honest status of
   exact-two is "reproduced, not yet established".
2. **Single-provider, single-window, single-machine.** Every run is Token Lab, one wire model,
   one process, one SQLite dir. Nothing here generalises to another provider, a rate-limited
   window, or concurrency > 1.
3. **The failures were classified, not explained.** `E_PROVIDER_TIMEOUT` at planning is
   recorded as "upstream", but the app also has a local governor and a call cap; without a
   wire-attempt counter, "upstream" is an **inference (L3)** wearing the clothes of a fact.
4. **Proxy metrics stand in for direct ones.** Rate-limit refusal is proved by elapsed time
   (C8); daemon liveness is proved by a DB state (C6). Both are the cheapest available
   measurement and both are one causal step away from the claim.
5. **Tests assert guards, not outcomes.** T9–T15 are genuine L2 evidence *that the guard
   refuses*. They are not evidence that the governed thing behaves well under a real model.
   Conflating these is the main systematic error in the current claim ledger.

---

## 4. Five improvements (pseudocode before → after)

Each improvement is written as a **pre-registration**: the pass/fail rule, the sample size,
and the falsifier are fixed *before* the run. A test whose threshold is chosen after seeing
the data is not a test.

---

### I1 — Pre-registered exact-two stability trial (replaces "2/3 is a stability claim")

**Before**
```text
run exact-two
if completed: report "topology reproduced"
else:          report "provider failure, not a topology violation"
# n=3, no threshold, no CI, failures excluded from the denominator by narrative
```
**After**
```text
PREREGISTER (written to docs/audit/PREREG-exact-two-<date>.json, committed BEFORE runs):
  hypothesis : P(exact 7-task topology) >= 0.90
  design     : n = 20 independent runs, fresh data dir each, same prompt hash
  primary    : k = count(run has 1/2/2/2 and status=completed)
  pass       : Wilson 95% lower bound on k/n >= 0.75
  fail       : lower bound < 0.75  -> claim is "unstable"; drop the word "stable"
  exposure   : every run counted, including timeouts and 0-task runs (intent-to-treat)
  secondary  : per-level class matrix; report every deviation by depth
for i in 1..20:
    d = fresh_dir(); run = exact_two(d, prompt_hash)
    record {i, status, task_counts_by_depth, tokens, wall_s, error_code}
k = count(primary)                      # timouts are failures, never excluded
report k, n, wilson_lower(k, n), every_deviating_depth
```
- **Sample size:** 20 (≈20 × 110 s ≈ 37 min; at 2/3 the observed rate, 20 runs distinguish
  "≥0.90" from "≈0.66" with useful power; 3 runs cannot).
- **Falsifier:** Wilson lower bound < 0.75. Also falsified if any run produces >2 depth-1
  leads, or a class matrix violation at any depth.
- **Safety boundary:** fresh data dir per run; no provider key in artifacts; hard stop after
  20 runs or 40 min, whichever first; budget cap enforced by the existing call cap, not by a
  human watching.
- **Cost of being wrong:** low — it only removes a word ("stable") from the report.

---

### I2 — Machine-checked parent-after-child invariant (replaces prose ordering inference)

**Before**
```text
# AGENTIC-LIVE-TESTS section B
observe: a SuperAgent emitted a result saying it had no children
reason:  dispatch readiness "does require" children complete first
conclude: probably not a scheduler violation  <- L4 presented as reassurance
```
**After**
```text
# invariant, checked at every parent result, in production AND in test
on parent_result(p):
    for c in direct_children(p):
        require c.state == completed or c.state == failed_terminal
        else: emit violation{run, parent, child, parent_state, child_state}
              refuse parent completion with E_CHILD_NOT_TERMINAL
    require p.result_emitted_at > max(c.completed_at for c in direct_children(p))

test parent_never_finalizes_before_child:
    seed run with child whose completion is delayed by a latch
    assert parent completion is refused with E_CHILD_NOT_TERMINAL
    release latch; assert parent completes after child.completed_at
```
- **Sample size:** 1 deterministic test × 3 seeds (latch, timeout-child, cancelled-child).
  Deterministic, so 1 passing run is L2; the seeds cover the three terminal states.
- **Falsifier:** any live or replayed run where `p.result_emitted_at <= child.completed_at`,
  or where a parent result is persisted with a non-terminal child.
- **Safety boundary:** refusal is a named contract code (`E_CHILD_NOT_TERMINAL`), never a
  silent wait — a scheduler that blocks forever on a lost child is worse than one that lies.
- **Why it matters:** this is the difference between "the hierarchy is correct" and "we never
  looked".

---

### I3 — Wire-level zero-spend proof for the rate governor (replaces the 0 s proxy)

**Before**
```text
# evidence: live-tokenlab-rate-limit.json
assert refusal_code == "E_LLM_RATE_LIMIT" and elapsed_s == 0 and tasks == 0
conclude "no provider request was made"      <- L3 inference
```
**After**
```text
# instrument the ONE chokepoint every send passes through
struct Wire { attempts: AtomicU64, refused_by_governor: AtomicU64 }

fn send(req):
    if !governor.acquire(req.estimated_tokens):     # BEFORE any socket work
        wire.refused_by_governor += 1
        return Err(E_LLM_RATE_LIMIT)
    wire.attempts += 1                               # only after the gate
    transport.post(req)

test governor_refuses_with_zero_wire_attempts:
    drain the window; snapshot (a0 = wire.attempts)
    submit; assert err == E_LLM_RATE_LIMIT
    assert wire.attempts == a0                  # direct, not elapsed time
    assert env_probe_never_called()             # key read is lazy, after the gate
wait one window; resubmit; assert wire.attempts == a0 + 1 and status == completed
```
- **Sample size:** 1 deterministic test at the chokepoint + 1 live confirmation (the existing
  run re-run once with the counter in the artifact).
- **Falsifier:** `wire.attempts` increments on a refused call, or the provider key/environment
  is read before the governor check.
- **Safety boundary:** the counter is local-only (no PII, no token values); the governor
  remains fail-closed; a governor that refuses everything must be distinguishable from one
  that refuses correctly — hence the post-window success assertion.
- **Converts:** C8 from L3 → L2.

---

### I4 — Decide the daemon by experiment: tick or retract the claim

**Before**
```text
spawn_daemon -> insert row {state: "enrolled", job: {"check":"db"}}
report: "Daemon is detached and has a real job"   <- true of the row, silent on execution
# TRUTH T19: "UNVERIFIED: enrolled record; no live tick proof"
```
**After**
```text
# Option A: make it runnable, then prove one tick
fn daemon_tick(d):
    require d.state == "enrolled"
    require d.next_tick_at <= now()
    record tick_started{daemon, tick_no}
    out = execute_job(d.job)                  # {"check":"db"}
    record tick_finished{daemon, tick_no, ok, latency_ms}
    d.next_tick_at = now() + d.interval

test daemon_ticks_and_writes_audit:
    spawn daemon(interval=1s); snapshot audit count
    wait <= 3 intervals
    assert audit contains >= 1 tick_started AND >= 1 tick_finished for that daemon
    assert daemon.state == "running" (not "enrolled")

# Option B (if ticking is out of scope this cycle): retract, do not soften
report: "Daemon is an enrolled record with a job definition; it does not execute."
```
- **Sample size:** 3 ticks (minimum to distinguish "ran once" from "ticks"), plus 1 negative
  test with the job deliberately failing to prove the failure is recorded, not swallowed.
- **Falsifier:** zero `tick_started` rows within 3 intervals, or a tick that reports success
  with no `ok` field / no latency. Option B falsifier: a docs sentence anywhere that says
  "runs".
- **Safety boundary:** tick loop must have a hard max-ticks-per-process and a kill switch;
  daemon writes only audit rows, never product tables; interval floor so a misconfigured
  1 ms interval cannot spin.
- **Rules out:** the current state, where the artifact supports "enrolled" and the prose
  implies "runs".

---

### I5 — Cap-boundary probe with a claim-gate linter (replaces "caps allow it")

**Before**
```text
caps.rs: MAX_SPAWN_DEPTH=3, MAX_FANOUT=8, MAX_AGENTS=64, MAX_CALLS=64
live evidence: depth 4/4 levels, fanout 2/8, agents 7/64, calls ~4/64
report: "Full 40-node live scale — UNVERIFIED / NOT RUN"    <- honest, but unbounded forever
```
**After**
```text
# 1. boundary probes, offline first (no provider spend), one per cap
test[fanout=8]:  build plan with 8 siblings; assert 8th accepted, 9th -> E_FANOUT_LIMIT
test[depth ]:    build chain to depth 3; assert depth 4 -> E_SPAWN_DEPTH
test[agents ]:   build 64 tasks; assert 65th -> E_AGENT_LIMIT
test[calls  ]:   count to 64; assert 65th -> E_CALL_LIMIT
# 2. one bounded live probe at the largest plan the caps permit, with a spend ceiling
live: 40-node plan, call cap 64, token ceiling enforced by the governor, abort on first refusal

# 3. claim-gate linter (P3 P7, wired into the audit pipeline)
lint(claim_row):
    if level in {L3,L4} and "stable|proven|guaranteed|learns|AGI" in text: error
    if level == L1 and n == 1: require the word "one run"
    if level in {L5,L6}: require a falsifier field or be removed
```
- **Sample size:** 4 deterministic boundary tests (1 each) + 1 live 40-node probe. The live
  probe is a single observation (L1) and is reported as such — it widens the envelope, it does
  not establish scale stability.
- **Falsifier:** any refusal that is not the named code, or a silent clamp; or a live 40-node
  plan that exceeds the call cap before completing (which would prove the caps are mutually
  inconsistent, a real design bug).
- **Safety boundary:** offline probes first — they cost nothing and catch the refusal path.
  The live probe runs with a hard token ceiling and aborts on first refusal; no retry storm.
- **Why a linter:** the audit corpus already contains three internal conflicts (P3 §5.4) and
  stale rows (P3 §5.1 rows 11, 15). Human review did not catch them; a lint on the claim
  register will.

---

## 5. What would change the verdict (cheapest first)

1. **I2** — highest value per hour. One invariant test converts the single most load-bearing
   behaviour claim (ordering) from inference to evidence.
2. **I1** — 37 minutes buys the right to use or lose the word "stable".
3. **I3** — small code change, converts a proxy into a direct measurement.
4. **I4/A** — only if a daemon runtime is in scope; otherwise **I4/B** costs one doc edit.
5. **I5** — the linter is cheap and stops the next overclaim before it is written.

## 6. Boundary of this document

- No test was run and no live model was called; every L1/L2 grade is quoted from a committed
  artifact or a read source file, not re-observed here.
- The claim table covers the six areas named in the brief. It is not a census of all product
  claims; `CLAIM-REGISTER.json` remains the wider (and partly stale) register.
- Sample-size reasoning for I1 is power-based on the observed 2/3 rate. If the true rate is
  ≥0.90, 20 runs will very likely pass; if it is 0.66, 20 runs will very likely fail. That is
  the discrimination the current n=3 cannot make.
- **Rejected without qualification:** AGI, proto-AGI, "on the path to AGI", self-improving,
  self-modifying, continual learning, curiosity, world model, metalearning, network satellite,
  and HMARS scale. These are refuted by absence of mechanism, not merely unsupported.
