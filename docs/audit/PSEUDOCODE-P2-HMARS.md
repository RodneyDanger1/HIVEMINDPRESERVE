# PSEUDOCODE-P2-HMARS — Intended HMARS organism vs. the code that exists

- **Pass:** P2 pseudocode audit. **Mode:** read-only. No source file was modified.
- **Written:** `docs/audit/PSEUDOCODE-P2-HMARS.md` (this file is the only write).
- **Sources read in this pass:**
  - `C:\Users\Riley\Desktop\Hyperagent psuedocode helpful info.txt` lines 231–530 (HMARS core model, `ContextBank`, `EventBus`, `TokenGovernor`, `HyperAgent`, gated admission, audit), 521–530 (mitigations), 1042–1260 (async engine / supervisor), 1276–1450 (asymmetric tool registry). Headers located by index; the PDF `2409.16299v3.pdf` defines the Planner / Navigator / Code Editor / Executor decomposition and is mapped in §2.5 from the four-tier material already in the file plus the repo's own four-stage executor names — **the PDF body itself was not extracted in this pass** (see §9).
  - `backend/crates/hiverouter-core/src/execution.rs` — `root_checkpoint` 3869–3960+, `record_root_gate_decision` 4347–4369, call sites 3935/3944/3980/3992/4023/4066, `MAX_ROOT_TOOL_CALLS` 3902, `has_planned_branch` 3897.
  - `backend/crates/hiverouter-core/src/remote_supervisor.rs` — `tick_once` 96–101, backoff loop 71–92, `record_state` 108+.
  - `docs/audit/REMOTE-PERMISSION-GAPS.md` §1, §3.2, §3.3, §7 G1–G9 (the current gap register; two of its line citations are now stale — see §6.0).
  - `docs/audit/CAPABILITY-MATRIX.md` §1 roles, §3 tier matrix, §6 root-only tools, §7 A2A.

## 0. Evidence labels used in this document

| Label | Meaning |
|---|---|
| `READ` | Line-verified in this pass, cited to file:line. |
| `AUDIT-DOC` | Asserted by `REMOTE-PERMISSION-GAPS.md` / `CAPABILITY-MATRIX.md`, not re-verified line-by-line here. |
| `DESIGN` | From the pseudocode file. Aspirational, not current behaviour. |
| `INFER` | My conclusion from two pieces of evidence; not directly observed. |

---

## 1. The intended organism (from the pseudocode file)

### 1.1 Topology `DESIGN` — file lines 234–258

```text
[ HYPERAGENT ] <============ Event Bus: REQUEST_REMOTE / APPROVAL_DECISION / TELEMETRY
      |
      +--> [ SUPERAGENT ] --(hive bank)--> [ AGENT ] --> [ SUBAGENT ]   (synchronous tree)
      |
      +==> [ DETACHED REMOTE ]        (satellite / lightswitch, tree-free, grant-bounded)
      +==> [ ACTIVE DAEMON ]          (continuous loop, ticks, emits telemetry)
```

Six tiers with two *non-tree* branches hanging off the root over an event bus. The tree is the
synchronous spine; remotes and daemons are asynchronously attached to the same governor.

### 1.2 The intended pseudocode, condensed

```python
class Tier(Enum): HYPERAGENT, SUPERAGENT, AGENT, SUBAGENT, REMOTE, DAEMON
class EventType(Enum): REQUEST_REMOTE, APPROVAL_DECISION, TELEMETRY, TASK_COMPLETE

class ContextBank:                      # scope-isolated memory vault
    def __init__(self, scope): self.scope = scope; self.storage = {}
    def write(self, k, v): self.storage[f"{self.scope}:{k}"] = v
    def read(self, k):     return self.storage.get(f"{self.scope}:{k}")
    # scope is "bank:run:<id>" or "bank:branch:<id>" — cross-run rewire is impossible
    # because the scope prefix is structural, not a caller-chosen string.

class EventBus:                         # asynchronous connective tissue
    def publish(self, t, payload):
        for cb in self.listeners[t]: cb(payload)      # root handlers run here

class TokenGovernor:                    # spend gate BEFORE the provider call
    def enforce_and_consume(self, cost):
        if self.consumed + cost > self.limit: raise E_LLM_RATE_LIMIT
        self.consumed += cost

class HyperAgent:                       # the sovereign
    root_bank = ContextBank("bank:run:<self>")        # root memory
    def __init__: self.bus.subscribe(REQUEST_REMOTE, self.handle_remote_admission_request)
    def generate_macro_plan(goal) -> [ {step, tier, branch_required, task} ]
    def handle_remote_admission_request(payload):
        # THE GATE. Depth-2 agents cannot allocate; they ask.
        if duplicate_name(payload.name): deny(E_DUPLICATE_NAME); return
        log_gate_decision(payload.agent_id, payload.name, allowed=True, "APPROVED_BY_ROOT")
        create DetachedRemote(name, device_type, grant_scope="toggle_state",
                              attribution="hyper:<requesting_agent_id>")
        publish(APPROVAL_DECISION, {agent_id, status: APPROVED, remote_id})
    def log_gate_decision(agent_id, target, allowed, reason):
        row = {timestamp, origin_node, target_resource, decision: ALLOWED|REFUSED, reason}
        self.admission_gate_logs.append(row)                 # in-memory
        self.root_bank.write(f"audit_gate:{uuid4()}", row)   # durable, root-scoped
    def execute_system_sync(goal):
        plan = generate_macro_plan(goal)
        for step in plan:
            if step.branch_required: execute_superagent_branch(new super node)
            # else: the step runs at this tier / directly against a primitive

class DetachedRemote:                   # tree-free, grant-bounded
    def execute_gated_action(cmd):
        if cmd not in self.grant_scope: return E_ACCESS_DENIED
        self.state.status = ACTIVE

class DaemonWorker:                     # uncoupled ticker
    def tick(self):
        if self.is_alive: self.bus.publish(TELEMETRY, {source: self.id, metric: payload})
```

### 1.3 The four properties the intended design actually claims `DESIGN` — file 521–530

1. **Dead spawn path resolved** — `generate_macro_plan` inspects the goal and emits
   `branch_required: False` steps, so a valid plan can exist with no branch lead.
2. **Inter-agent authorization protocol** — an `Agent` "cannot natively declare system entities";
   it publishes `REQUEST_REMOTE` and the root adjudicates at runtime.
3. **Immutable audit trail** — every allocation records timestamp, caller attribution,
   decision, and reason into durable storage, not just success.
4. **Structural isolation** — memory is namespaced per run and per branch; a parallel branch
   cannot falsify another branch's state.

### 1.4 HyperAgent→SuperAgent→Agent→Subagent, plus detached (file 247–258)

```text
HyperAgent   = persistent orchestrator; owns run bank; cross-run dependencies; sole allocator
               of external resources; signs detachable grants.
SuperAgent   = branch lead; owns one hive bank ("bank:branch:<id>"); decomposes a macro-slice;
               spawns Agents; aggregates; monitors child spend.
Agent        = execution surface (sandbox FS, gated shell, tool suites); may spawn Subagents;
               may *request* (never perform) remote allocation.
Subagent     = stateless single-prompt leaf; restricted tools; self-terminates on resolution.
Remote       = tree-free fixture holding a Grant; actions bounded by grant_scope; attribution
               must map back to hyper:<task>.
Daemon       = background ticker off the tree; publishes telemetry; never blocks planning.
```

---

## 2. Side-by-side: intended vs. current

### 2.1 Component map, one row per pseudocode construct

| Intended construct (file) | Current counterpart | Status | Evidence |
|---|---|---|---|
| `Tier` enum incl. `REMOTE`, `DAEMON` | `AgentTier` (hyper/super/worker/subagent/tool) + separate `remote`/`daemon` **stores**, no tier | **Split-brain** | `execution.rs:131–235` `READ`; `remote.rs`, `daemon.rs` `READ` (file list) |
| `ContextBank(scope)` with mandatory prefix | `context.rs` + `store.record_agent_memory_scoped(bank, key, value, run_id, task_id, scope)` | **Partial** — banks exist; no `remote:`/`daemon:` bank | `execution.rs:4361` `READ`; G8 `AUDIT-DOC` |
| `EventBus.publish` fan-out to root handler | `bus.rs` (in-process broadcast) **plus** `RunEngine::publish` to WS | **Partial** — bus exists but admission does not travel on it | `execution.rs:950` `publish` `READ`; §4.2 |
| `handle_remote_admission_request` (async, gated) | `root_checkpoint` synchronous tool loop | **Absent as a gate** — see §4.2 | `execution.rs:3869–3960` `READ` |
| `log_gate_decision` → `root_bank` durable | `record_root_gate_decision` → `agent_memory` row, bank `"hyper"` | **Present but mis-typed** — memory row, not an audit row | `execution.rs:4347–4369` `READ` |
| `DetachedRemote.execute_gated_action` | `remote.rs` principal + device + grant; `in_process` fixture | **Fixture only** | CAPABILITY-MATRIX §1 "Label / fixture" `AUDIT-DOC` |
| `DaemonWorker.tick()` on a live loop | `LocalRemoteSupervisor::run` loop → `tick_once()` | **Partial** — real loop, no live daemon body | `remote_supervisor.rs:71–101` `READ` |
| `TokenGovernor.enforce_and_consume` before spend | `claim_provider_call` + `add_run_usage` + caps | **Partial** — post-hoc accounting, not pre-spend gate | `execution.rs:713, 3908, 3913` `READ` |
| `generate_macro_plan` can emit `branch_required: False` | `Plan::validate` **forbids** a branchless plan | **Contradicted** — the "fix" is unreachable | `planner.rs:50–53, 96–101` `AUDIT-DOC`; §4.1 |

### 2.2 The current code, as pseudocode

```rust
// execution.rs:3869  — the whole "HyperAgent governor" is this function
async fn root_checkpoint(run, root, plan, token) -> Result<(), Refusal> {
    if !self.root_orchestrator { return Ok(()) }             // :3876  feature flag
    let prompt = "...You are the HyperAgent orchestrator..."  // :3885-3892
    let has_planned_branch = plan.nodes.iter().any(|n| n.depth == 1);  // :3897
    const MAX_ROOT_TOOL_CALLS: usize = 4;                    // :3902
    for _turn in 0..1 {                                      // :3903  exactly one turn
        self.claim_provider_call(run.id)?;                   // :3908  spend claim
        let reply = executor.orchestrate_root(...).await?;   // :3911
        self.add_run_usage(run.id, reply.usage);             // :3913
        if reply.tool_calls.is_empty() { return Ok(()) }     // :3915
        if reply.tool_calls.len() > 4 { warn!(...ignoring excess...) }   // :3917-3923
        let mut _result_text = String::new();                // :3925  ← DISCARDED
        let mut root_remote_names: HashSet<String> = …;      // :3926  per-turn only
        let mut root_daemon_names: HashSet<String> = …;      // :3927
        for call in reply.tool_calls.iter().take(4) {        // :3928
            match call.name.as_str() {
                "spawn_superagent" => {                      // :3930
                    if has_planned_branch {                  // :3931  ALWAYS TRUE
                        _result_text += "- refused: planner already supplied branch leads\n";
                        self.record_root_gate_decision(run, root, "spawn_superagent",
                            "refused", "planner already supplied depth-1 branch leads")?;  // :3935
                    } else {                                 // :3942  DEAD
                        let child = self.spawn_sub_hive(root.id, "super")?;
                        self.record_root_gate_decision(..., "allowed", ...)?;             // :3944
                    }
                }
                "spawn_remote_agent" => {                    // :3957
                    // validate name (1..120); dedupe against root_remote_names (this turn only)
                    // create_remote + create_device + grant(permanent) + attribution
                    // record_root_gate_decision(..., "allowed"|"refused", ...)  :3980/:3992
                }
                "spawn_daemon" => { /* store.create(enrolled); gate row :4023/:4066 */ }
                "read_memory_summary" => { memory_hyper_summary(run, 8) }  // :3973ff
                "finish" => return,
            }
        }
        // _result_text dropped here. No event, no row, no feedback.
    }
}

// execution.rs:4347 — where a gate decision goes today
fn record_root_gate_decision(self, run, root, action, decision, detail) -> Result<(), Refusal> {
    let payload = json!({ action, decision, root_task_id, detail });
    self.store.record_agent_memory_scoped(
        "hyper",                                  // bank
        &format!("root_gate:{}", Uuid::now_v7()), // key  ← unique per row, never read by key
        &payload.to_string(),                     // value
        run.id, root.id, "run",                   // scope
    )
}

// remote_supervisor.rs:96 — the "daemon tick"
fn tick_once(&self) -> Result<(), Refusal> {
    self.remote.expire_leases()?;     // lease sweep
    self.remote.expire_approvals()?;  // approval TTL sweep
    self.daemons.tick_due(64)?;       // ≤64 due daemons per pass, no per-daemon claim
}
```

---

## 3. Three structural facts that decide everything else

**F1 — The governor is a single bounded turn, not a subscriber.** `for _turn in 0..1` (`READ`
:3903). The intended `HyperAgent` "persistent orchestrator" that *reacts* to `REQUEST_REMOTE`
does not exist. There is no waiter, no inbox, no resumed turn. A lower-tier request cannot be
answered by this design at all, because the root has already returned before any Agent runs —
`root_checkpoint` executes before `Planning -> Running` (`AUDIT-DOC` §3.2, `execution.rs:1899-1914`).

**F2 — Admission is a `match` arm inside a tool loop.** Every authority-changing act is a
synchronous side effect inside `for call in reply.tool_calls`. Idempotency is a `HashSet` that
lives for one turn (`READ` :3926–3927). Durability of the decision is a memory row
(`READ` :4361). There is no request object, no request id, no decision id, no nonce, no TTL.

**F3 — The one gate that looks conditional is unconditionally refusing.** `has_planned_branch`
is `any(depth == 1)` (`READ` :3897); `Plan::validate` requires a non-empty node set and
requires every parentless node to be depth 1 (`AUDIT-DOC` `planner.rs:50–53, 96–101`). Therefore
`has_planned_branch == true` for every valid plan and `:3942–3955` never executes.

---

## 4. Exact gaps

### 4.0 Stale citations to correct before anyone acts on the old doc

| Old claim | Reality in this pass |
|---|---|
| `has_planned_branch` at `:3885` | now `:3897` `READ` |
| "no gate audit row at all" (G4) | a row **is** written at `:3935/3944/3980/3992/4023/4066` → `record_root_gate_decision` `READ`. It is written to `agent_memory`, bank `hyper`, key `root_gate:<uuid7>` — durable **but not an audit row and never read back by key** |
| "spawn_remote_agent/spawn_daemon are not gated at all" | they now record a gate decision; they are still **not approved** (no approval row/nonce/decision object) `READ` |

### 4.1 GAP-RG — Root gate dead path

```text
INTENDED   "root may add one branch lead only when the planner produced a flat plan"
           → implies flat plans are producible                          (execution.rs:3893-3896 comment)
REALITY    Plan::validate ⇒ every valid plan has ≥1 depth-1 node
           ⇒ has_planned_branch ≡ true ⇒ allow branch ≡ dead            (:3897, :3942-3955)
CONSEQUENCE  (a) spawn_superagent is a capability the model is told it has and cannot use;
             (b) the test at run_lifecycle.rs:599-630 proves the refusal only;
             (c) the allow path carries an audit row that can never be written;
             (d) any future reader of the comment believes a conditional decision exists.
```

### 4.2 GAP-EB — Event bus exists; admission is synchronous

```text
INTENDED   Agent --publish(REQUEST_REMOTE)--> HyperAgent.handle_remote_admission_request
           (asynchronous, non-blocking, decided later, answer published back)

CURRENT    bus.rs exists and carries WS events; RunEngine::publish fans out to UI
           BUT: no EventType::REQUEST_REMOTE, no subscriber, no waiter, no resume.
           The only creation path is the root's own tool call, in the root's own turn,
           before any Agent exists. A depth-2 Agent literally cannot emit this event.

GAP        G1 (AUDIT-DOC, High) is unfixed AND unfixable by adding a bus topic alone:
           a root turn that has already returned cannot adjudicate.
           The fix needs either (i) a durable request row decided by the *next* root turn or
           by the operator, or (ii) a second root turn resumed on request arrival.
```

### 4.3 GAP-AU — Audit is memory, not audit

```text
INTENDED   log_gate_decision → immutable row {ts, origin_node, target_resource,
           decision ALLOWED|REFUSED, reason}, queryable for compliance

CURRENT    record_root_gate_decision(:4347) → record_agent_memory_scoped(
             bank="hyper", key="root_gate:<uuid>", value=<json>, run, task, "run")
           • typed as agent memory, so every bank/scope query returns it as memory
           • key is a fresh UUID ⇒ not addressable, not dedupable, not idempotent
           • read_memory_summary formats "bank_id:key" only (CAPABILITY-MATRIX §6 M-13)
             ⇒ the decision text (`detail`) is unreachable through the model's own reader
           • no audit table row, so `remote_audit`/run audit exports omit it
           • actor attribution is still operator for device rows; `granted_by` still
             hardcoded on some paths (GAP register G2, AUDIT-DOC)
```

### 4.4 GAP-MEM — Memory scoping

```text
INTENDED   bank:run:<id> / bank:branch:<id>, structural, cross-run injection impossible
CURRENT    run-scoped and branch-scoped banks exist (context.rs; record_agent_memory_scoped
           takes run_id/task_id/scope) — but:
             • no bank for detached principals (remote:/daemon:) → G8
             • root summary ignores `scope`/`limit`, always (run, 8) → M-13
             • root's final answer is written to the wrong bank (CAPABILITY-MATRIX §1)
             • gate rows land in the root's memory bank, polluting it
NEITHER    (a) a read barrier for a bank a task should not see, nor
           (b) a per-branch write proof, both of which the pseudocode's prefix implies.
```

### 4.5 GAP-DT — Daemon tick

```text
INTENDED   DaemonWorker.tick() on a live loop, publishes TELEMETRY, never blocks planning
CURRENT    LocalRemoteSupervisor::run → tick_once(): expire_leases, expire_approvals,
           daemons.tick_due(64)                                        (remote_supervisor.rs:96-101)
             • 64 = throughput bound, NOT an admission cap (G9)
             • tick_due has no per-daemon claim/lease ⇒ two supervisors (or a retry)
               can run the same due daemon in the same window
             • backoff + record_state exist (:82-90, :108+) but record only
               last_tick/backoff/error_code — no per-daemon outcome counters
             • no live body for non-watchdog kinds ({} payload ⇒ degraded) (M-20)
             • root-created daemons are omitted from the root control prompt (M-20)
```

---

## 5. Improvements

Each: **before → after**, safety, effort **S/M/L**, and the test that would prove it.
"Now?" = implementable without inventing a capability the tree does not already have.

### I1 — Make the gate decision retrievable and ordered *(the row exists; it cannot be read)*

A UUID-keyed memory row is already durable. The gap is **retrievability**, not persistence.

```text
BEFORE  key = "root_gate:<uuid v7>"           → unique, unaddressable, re-listed as memory
AFTER   key = "root_gate:<run_id>:<seq>"      → stable, ordered, countable
        + store decision in a dedicated `gate_decisions` view/table projection when present
        + read_memory_summary(scope, limit) finally honours its args
```
**Safety:** additive; no path change; no new authority. **Effort:** S. **Now?** yes.
**Test:** unit — call `root_checkpoint` with a refused `spawn_superagent`; assert
`memory.list(run, bank="hyper")` yields exactly one `root_gate:` key whose value parses to
`{action, decision:"refused"}`; assert a second run does not see it.

### I2 — Stop discarding the root's answer; publish a `ROOT_CONTROL` event with the gate outcome

```text
BEFORE  let mut _result_text = String::new();  …  // dropped at end of turn (:3925, :3913)
AFTER   let mut result_text = String::new();
        … same match arms …
        self.publish(Event::RootControl { run_id, decisions: result_text }, …)  // WS-visible
        self.store.record_tool_turn(run, root, "root_control", &result_text)?   // durable
```
**Safety:** read-only observer additions; no new authority; WS consumer must tolerate a new
event kind (version the event enum). **Effort:** S. **Now?** yes.
**Test:** e2e — submit a prompt whose root turn requests `spawn_superagent`; assert the WS
stream contains one `ROOT_CONTROL` frame containing `"refused"`, and the run audit export
contains the same string. Today both assertions fail.

### I3 — Resolve the dead branch: decide topology, do not patch around it

```text
OPTION A (delete the lie — recommended)
BEFORE  root prompt lists "spawn_superagent (extra branch lead)"; match arm :3930; dead else :3942
AFTER   prompt lists spawn_remote_agent / spawn_daemon / read_memory_summary / finish only;
        match arm returns refusal "E_ROOT_TOOL_RETIRED" for any legacy call; gate row retained
OPTION B (make it real — expensive, changes the plan contract)
BEFORE  Plan::validate requires ≥1 depth-1 node
AFTER   validate accepts `nodes.is_empty() || leading_node.depth == 1`; a validated flat plan
        sets has_planned_branch=false and the allow path becomes reachable; add a counter and
        a per-run cap of 1 before spawn_sub_hive
```
**Safety:** A removes capability claims and cannot break a passing path (the else is
unreachable). B widens the plan contract and every consumer of `depth==1` must be re-audited —
**do not do B in the same change as anything else**. **Effort:** A = S, B = L. **Now?** A yes;
B needs a topology decision from the owner first (the doc's §2.3 conflict is unresolved).
**Test:** A — assert the root tool schema advertised to the provider no longer contains
`spawn_superagent`, and a forged call yields `E_ROOT_TOOL_RETIRED` plus one gate row.
B — a plan fixture with zero depth-1 nodes validates and produces exactly one `sub-hive` child.

### I4 — Durable `detached_request` + one `admit_detached()` admission function

```text
BEFORE  root match arms each do:  validate name → create → grant → gate row
        (three stores, three attributions, zero request identity)
AFTER   struct DetachedRequest { id, run_id, requesting_task_id, kind: Remote|Daemon,
                                 target, desired_scope, reason, idempotency_key, state }
        fn admit_detached(req) -> Decision {
            if let Some(existing) = store.find_by_idempotency(req.run_id, req.idempotency_key) {
                return Decision::Replay(existing)               // no second side effect
            }
            if store.count_run(run, kind) >= cap(kind) { return Decision::Refused(E_…_LIMIT) }
            let decision = decide(req);                          // policy, single place
            store.insert_decision(req, &decision)?;              // BEFORE the effect
            if decision.allowed { create principal + grant, attributing actor=hyper }
            return decision;
        }
```
**Safety:** this is the only change that touches the create path; land it behind one flag, port
remotes first, daemons second. **Effort:** M (schema + one new module) / L if daemons move too.
**Now?** yes for the request table + remote path; **no** for the async event routing (I5).
**Test:** call `admit_detached` twice with the same `idempotency_key`; assert one principal row,
one grant row, two decision rows (one `replay`), and zero duplicate side effects. Then assert
`E_REMOTE_LIMIT` at cap.

### I5 — The real event-driven path: request row → durable waiter → next root turn decides

```text
BEFORE  nothing can enqueue a request; the root turn is the only creator
AFTER   Agent tool `request_remote_allocation` (Worker tier only) →
          store.insert(detached_request, state=pending) → publish(DetachedRequested)
        root turn N+1 (or operator endpoint) calls admit_detached on pending rows
        pending request older than TTL → state=expired + audit row + notify requester
```
**Safety:** high blast radius (new tool on a tier, new resume trigger). Requires F1 to be
relaxed: the root must be able to run a second control turn. **Effort:** L. **Now?** only the
*request table + operator-decided* half; the model-decided half needs a topology/resume change.
**Test:** with the operator endpoint, a pending request can be approved and produces exactly
one principal; denied produces none and one audit row; expiry produces one audit row and no
principal.

### I6 — Audit the authority act, not the memory bank

```text
BEFORE  actor_kind ∈ {operator, system}; granted_by='operator' on some paths; refusal_code never written
AFTER   actor_kind ∈ {operator, hyper, agent, daemon}; granted_by = created_by;
        every refusal path writes {code, target, reason} to the control-plane audit;
        remote/daemon create, device action, and grant resolve each emit one row
        with (request_id, decision_id) FK when present
```
**Safety:** attribution-only writes; no behaviour change; must backfill nothing (audit is
append-only). **Effort:** M. **Now?** yes.
**Test:** create a remote as the root; assert audit rows carry `actor_kind='hyper'` and
`granted_by = <root task>`; then issue a denied device action and assert a row exists with
`refusal_code='E_ACCESS_DENIED'`. Today both fail (success-biased audit).

### I7 — Daemon tick: claim-then-run with per-daemon outcome counters

```text
BEFORE  tick_due(64): select due → run → no claim; two ticks can overlap on one daemon
AFTER   for d in select_due(64):
            if !store.claim_due(d.id, now, lease = tick_interval*2) { continue }   // CAS
            let r = run_body(d)
            store.record_tick(d.id, r.ok|err, next_due)      // counters: ran/ok/failed/degraded
        record_state(last_tick, backoff, error_code, lane_started)  // plus lane counters
```
**Safety:** one UPDATE…WHERE state='due' guard; observable, reversible. **Effort:** M.
**Now?** yes.
**Test:** force two supervisors in one test DB; assert `record_tick` count for one daemon in one
window is exactly 1, and that a second supervisor's pass ran 0 bodies. Then assert a failing
body increments `failed` and sets a backoff without starving the other daemons.

### I8 — Detached memory banks + scoped root summary

```text
BEFORE  banks: run, branch, hyper. remotes/daemons have none. summary = (run, 8), values hidden
AFTER   bank "remote:<principal_id>" and "daemon:<principal_id>", scope="principal"
        read_memory_summary(scope, limit) → honours both; returns redacted values (≤200 chars)
        agent_memory write requires a bank prefix the caller owns
```
**Safety:** new prefixes only; existing rows keep working; summary is read-only. **Effort:** M.
**Now?** yes.
**Test:** write a remote-scoped memory row; assert a run-scoped reader cannot see it; assert
`read_memory_summary("remote", 3)` returns ≤3 rows with values, and that `("run", 8)` still
returns 8.

### I9 — A2A answer must check the deadline inside the CAS

```text
BEFORE  answer_agent_message: CAS status='open' only            (G5)
AFTER   CAS status='open' AND deadline_at > now AND run/task non-terminal
        late attempt → E_DEADLINE_EXPIRED + a2a_audit row (attempts+1)
```
**Safety:** one WHERE-clause; existing expiry sweep untouched. **Effort:** S. **Now?** yes.
**Test:** insert an open message with `deadline_at` in the past; call answer; assert refusal
code, zero state change, one audit row. Then the same with a terminal run.

### I10 — Per-run detached admission cap at the same function

```text
BEFORE  root: 4 calls/turn, no scope cap; supervisor: 64/pass (throughput, not admission)
AFTER   admit_detached refuses with E_REMOTE_LIMIT / E_DAEMON_LIMIT at cap(run) and cap(global)
```
**Safety:** pure refusal; land after I4 so there is one place to put it. **Effort:** S.
**Now?** yes (as part of I4).
**Test:** request cap+1 remotes in one run; assert the last returns `E_REMOTE_LIMIT`, exactly
`cap` principals exist, and the refusal is audited.

---

## 6. Ranking by user impact

| # | Improvement | User-visible impact | Risk | Effort | Now? |
|---|---|---|---|---|---|
| 1 | **I6** honest authority audit (actor kinds, refusal rows, `granted_by`) | An operator can finally answer "who created this remote and why was that denial issued?" — the entire trust story for detached agents rests on it | Low (writes only) | M | **yes** |
| 2 | **I2** surface the root's decisions instead of dropping them | The UI stops being silent about refused/approved allocations; today `_result_text` is destroyed | Low (additive event) | S | **yes** |
| 3 | **I4** request identity + idempotency + one `admit_detached` | Duplicate double-spawns become impossible; every later gate change has one home | Medium (create path) | M | **yes** (remote first) |
| 4 | **I1** retrievable, ordered gate rows | Turns a write-only log into evidence a reviewer/export can read; unblocks the "durable conditional decision" acceptance item | Low | S | **yes** |
| 5 | **I3-A** retire the dead `spawn_superagent` from the root surface | Removes a capability the model is told it has and cannot use; kills a false comment | Low | S | **yes** |
| 6 | **I8** detached memory banks + honest summary | Remotes/daemons stop being context-blind; summary stops lying about `scope`/`limit` | Low | M | **yes** |
| 7 | **I7** claim-then-run daemon ticks + counters | "Is my daemon actually running?" becomes answerable and double-runs stop | Low | M | **yes** |
| 8 | **I9** deadline-checked A2A answer | A late answer can no longer resurrect a dead question | Low | S | **yes** |
| 9 | **I10** admission cap | Bounds runaway allocation | Low | S | yes (inside I4) |
| 10 | **I5** true event-bus admission | The HMARS promise itself: a Subagent/Agent can *ask*, and the root *answers later* | High (needs a resumed root turn) | L | **partly** — operator-decided half only |
| — | **I3-B** make the allow path real | Only matters if the owner wants root branch expansion | High (plan contract) | L | **no** — needs a decision first |

### 6.1 Top 5

1. **I6** — honest authority audit.
2. **I2** — stop dropping root decisions; publish them.
3. **I4** — request identity, idempotency, one admission function.
4. **I1** — retrievable/ordered gate rows.
5. **I3-A** — retire the dead root tool.

### 6.2 Safest first fix

> **I2 + I1 together, as one S-effort change:** keep `record_root_gate_decision` where it is,
> re-key it `root_gate:<run_id>:<seq>`, and publish the accumulated decision text as a
> `ROOT_CONTROL` event instead of dropping `_result_text`.

Why this one first: it is **purely additive** (two lines moved, one `publish`, one key format),
touches no create path, no schema, no authority, cannot turn a passing test red, and it converts
the single weakest claim in the whole audit — "the root gate fired as a durable conditional
decision" — from `INFER/WEAK` into something a test can assert. It also has to land before I4
or I6, because both of those need a decision record that can be read back.

If the owner wants *behavioural* risk reduced before *evidence* risk, the alternate first fix is
**I3-A** (delete the dead tool). Also S, also no new capability, but it changes the model-facing
tool schema and therefore deserves a prompt regression run.

---

## 7. What is implementable now without inventing a capability

**Yes:** I1, I2, I3-A, I4 (remote half), I6, I7, I8, I9, I10 — each is expressed entirely in
constructs the tree already has: `store.record_agent_memory_scoped`, `RunEngine::publish`,
`remote.rs`/`daemon.rs` stores, `db.rs` migrations, `tools_bridge` approval gate, the existing
supervisor loop.

**Not yet:** I5's model-decided half (no mechanism resumes a returned root turn — F1) and
I3-B (requires relaxing `Plan::validate`, which the owner has not decided).

**Do not do:** wire `spawn_remote_agent`/`spawn_daemon` into `request_tool_approval`.
`AUDIT-DOC` §3.2 records that `request_tool_approval` refuses an unknown tool id; using it
would need a registry entry, i.e. a new capability. Use `admit_detached` (I4) instead.

---

## 8. One-paragraph verdict

The repository has a genuine HyperAgent root turn, genuine detached *records*, and a genuine
supervisor loop — but it is not yet an HMARS organism. It is a **linear tree with a bounded
root preamble**. The event bus carries UI telemetry, not authority; admission is a `match` arm,
not a gate; the gate's allow branch is unreachable; the audit is a memory row; and the daemon
tick is a sweep rather than a claim. Every one of those five deltas is fixable without evading
the tree's real constraints, and the cheapest of them (I1+I2) buys the most trust.

---

## 9. Limitations of this pass

1. **The PDF was not text-extracted.** `2409.16299v3.pdf` is binary; no extractor was run inside
   the time box. The Planner/Navigator/Code Editor/Executor mapping in §2.1 is built from the
   repo's own stage names (`plan`, `execute`, `continue_with_tools`, `repair_plan`) and the
   four-tier material in the pseudocode file — treat the PDF-side correspondence as `INFER`.
2. **`planner.rs`, `remote.rs`, `daemon.rs`, `store.rs` were not read line-by-line.** Claims
   about `Plan::validate`, the grant path, and daemon kinds are `AUDIT-DOC`, not `READ`.
3. **No test was executed.** Every "test that would prove it" is a specification, not a result.
4. **No impact/change analysis was run**, because no symbol was modified; the file is new and
   additive. If any improvement here is implemented, GitNexus `impact` must run first on
   `record_root_gate_decision`, `root_checkpoint`, and `record_agent_memory_scoped`.
5. **Two citations in `REMOTE-PERMISSION-GAPS.md` are stale** (§4.0). That document should be
   corrected before it is used as the change brief; this file does not edit it.
