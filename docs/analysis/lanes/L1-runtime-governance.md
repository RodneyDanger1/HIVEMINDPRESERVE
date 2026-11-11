# L1 — Runtime Execution, Planning, Graph, Root/HyperAgent Governance, Recovery, Lifecycle

**Lane:** 1 of 7 (Codebase Analyst). **Mode:** read-only analysis; the only file written is this report.
**Repo root:** `C:\Users\Riley\Desktop\hiverouter-v1` (all citations are repo-relative).
**Model/context:** single pass; no source, config, test, doc, or database was modified; no server started; no provider called; no key printed.
**Evidence markers used below:**
- **VERIFIED** — read in this session (exact lines named).
- **PARTIAL** — some of the file was read and the unread region is named.
- **CITED-FROM-TEST** — the behaviour is asserted by a test I read the assertions of, or by a test whose name/line I read but whose body I did not read in full.
- **UNVERIFIED** — not read, or inferred beyond what the code states. Inferences are labelled as such.

---

## 0. Executive summary (the three questions I was asked)

### 0.1 Is the HyperAgent truly governing, or only a one-shot controller?

**It is a bounded one-shot controller plus a final synthesis turn. It does not govern.** VERIFIED from:

- The root control turn is invoked exactly once, before dispatch starts, and is limited to **2 provider turns** and **4 tool calls per turn**: `backend/crates/hiverouter-core/src/execution.rs:4328` (`for turn in 0..2`), `execution.rs:4319` (`const MAX_ROOT_TOOL_CALLS: usize = 4`), called from `execution.rs:2056` (`self.root_checkpoint(run, &root, &plan, token).await?`) **before** the run transitions `planning -> running` (`execution.rs:2057-2068`).
- Its entire vocabulary is: `spawn_superagent` (at most one, and only when the plan has no depth-1 branch), `spawn_remote_agent`, `spawn_daemon`, `read_memory_summary`, `propose_improvement`, `convene_evolution_vote`, `finish` (`execution.rs:4300-4307`, dispatch arms at `execution.rs:4381-4811`). Turn 2 may not create anything (`execution.rs:4362-4380`).
- It cannot inspect task state, cannot re-plan, cannot reassign, cannot cancel/kill, cannot retry, cannot gate a child's output, cannot adjust caps, cannot add a dependency edge, and is not consulted again while tasks execute. The only post-dispatch root action is one answer-only synthesis call at the end (`execution.rs:2201-2280`; provider side: `backend/crates/hiverouter-core/src/provider_executor.rs:672-697`, which advertises **no tools**).
- The actual scheduler is the engine, not the model: readiness by deps+children (`execution.rs:2101-2112`), bounded parallel pool (`execution.rs:2294-2369`), spawn queue (`execution.rs:2463-2513`), cap checks (`execution.rs:1545-1623`, `backend/crates/hiverouter-core/src/caps.rs:44-74`).

So hierarchy authority is **enforced by hard-coded engine rules around the model**, not exercised by the model as a governor. The root is a one-shot topology tweaker with an auditable decision log (`execution.rs:5229-5263`).

### 0.2 Is task settlement / caps honest?

**Settlement is honest about "terminal", but not about "completed". Caps are honest at insertion time; two caps are not honest under concurrency or under arithmetic.**

- Terminal-state guard is real and enforced in three layers: pure state machine (`backend/crates/hiverouter-core/src/run_state.rs:432-455`, `562-633`), engine read-check-write under a lock (`execution.rs:1402-1420`), and SQL guard (`backend/crates/hiverouter-core/src/store.rs:576` with terminal set from `store.rs:308`). A killed/failed task is never resurrected. VERIFIED.
- **But** structural rows (planner root, aggregator/queued rows that never execute) are settled to `Completed` purely because the run completed (`execution.rs:1784-1819`, called at `execution.rs:1842-1844`). The root row for a single-node plan is never executed at all (the root-final block is gated on `has_aggregators || plan.nodes.len() > 1`, `execution.rs:2206`), yet it ends `Completed` with no `task_results` row. Terminal-honest, not outcome-honest. VERIFIED by code, and consistent with the comment at `execution.rs:1786-1791`.
- Cap enforcement at spawn is under one lock (`execution.rs:1553-1616`) and pre-insert; that is honest. VERIFIED.
- Two cap defects: (a) the per-run **tool-call** budget is check-then-act across parallel tasks (`execution.rs:2554-2562` reads counters, `2607-2619` checks a stale value, `2720-2727` increments after the turn), so `parallel_dispatch = 32` (`backend/crates/hiverouter-core/src/http.rs:365-379`) can overshoot `MAX_CALLS_PER_RUN = 40` (`backend/crates/hiverouter-core/src/tool_loop.rs:16`); (b) the **provider-call** cap of 64 (`caps.rs:14-18`) is arithmetically inconsistent with a 40-node plan (`backend/crates/hiverouter-core/src/planner.rs:10`) plus planner + repair + 2 root-control turns + root final + up to 12 tool-loop continuations per task (`tool_loop.rs:14`) + parent escalation answers (`execution.rs:3736`). A large honest plan can die with `E_CALL_LIMIT`.
- Approval waits are **not** concurrency-safe: the run has one status slot and `suspend` is legal only from `Running` (`run_state.rs:510-512`), so a second task suspending while another waits produces `E_SCHEMA_INVALID` and a failed run (`execution.rs:3106-3113`).

### 0.3 Where can hierarchy or task-state violations occur?

| Surface | Can it violate hierarchy/state? | Evidence |
|---|---|---|
| Spawn from a task tool (`spawn_agent`) | No: caps + class matrix + depth checks run before the child row exists | `execution.rs:4027-4039`, `1550-1623` |
| Planned nodes | No for parent/depth/cycle; **run-fatal** on an unknown role (`E_PERSONA_UNKNOWN`) | `planner.rs:50-129`; `execution.rs:2002-2003` |
| `REWIRE_TASK` | Guarded (same-run, `parent.depth+1 == task.depth`, spawn matrix) **only when a task row exists**; a graph-only node rewires with CAS only | `backend/crates/hiverouter-core/src/ws.rs:802-859`, `ws.rs:860-863` |
| Root control | Cannot skip a level (`spawn_sub_hive` path), but **model/shape errors abort the run** instead of being bounded refusals | `execution.rs:4462`, `4565`, `4572-4579`, `4598`, `4730`, `4776` |
| Settlement | `settle_unfinished_tasks` writes an **illegal** `running -> queued` edge directly through the store | `execution.rs:2453-2457` |
| Graph API | `GraphAuthority::set_state` accepts **any** `AgentState` with no transition validation | `backend/crates/hiverouter-core/src/graph.rs:236-247` |
| Recovery of non-restored runs | Tasks are rebuilt **without parent edges** when the parent image was superseded | `backend/crates/hiverouter-core/src/recovery.rs:417-430`, `446-469` |
| Task identity (class/rank) | Only the `role` string; a rewired/renamed role keeps the old identity columns until re-insert | `backend/crates/hiverouter-core/src/db.rs:341-365`; `store.rs:1189` |

---

## 1. File coverage table (every file in the lane)

| # | File | Status | What I covered | Gaps / why |
|---|---|---|---|---|
| 1 | `backend/crates/hiverouter-core/src/execution.rs` (5768 lines) | **VERIFIED** | Read 1-1388, 1389-2590, 2591-3810, 3811-4926, 4927-5626, 5627-5768: all phases, dispatch serial/parallel, spawn queue, root control, convene voters, monitor/final, approvals, caps, cancel/kill, prune, tests | none |
| 2 | `backend/crates/hiverouter-core/src/run_state.rs` (1273) | **VERIFIED** (1-879) / PARTIAL (880-1273) | Whole state machine, guards, effects, `apply_run`/`apply_task`, tests 667-879 | 880-1273 is the remainder of the unit-test module (not behaviour) |
| 3 | `backend/crates/hiverouter-core/src/planner.rs` (340) | **VERIFIED** | `Plan`, `PlanNode`, `validate`, `topo_order`, `leaf_order`, `aggregation_order`, tests | none |
| 4 | `backend/crates/hiverouter-core/src/graph.rs` (444) | **VERIFIED** | `TaskNode`, `GraphSnapshot`, `GraphAuthority` add/metadata/rewire/set_state/restore/snapshot, tests | none |
| 5 | `backend/crates/hiverouter-core/src/caps.rs` (99) | **VERIFIED** | Constants, `Caps`, `check_spawn`, `check_calls`, tests | none |
| 6 | `backend/crates/hiverouter-core/src/context.rs` (445) | **VERIFIED** | `ScopedContext`, `prompt_block`, redaction, `switch_context`, bounded `ContextStore`, tests | none |
| 7 | `backend/crates/hiverouter-core/src/entities.rs` (99) | **VERIFIED** | `Run`, `TaskRecord`, `ChatRole`, `ChatMessage` | none |
| 8 | `backend/crates/hiverouter-core/src/snapshot.rs` (355) | **VERIFIED** | Envelope, atomic write, read/migrate, revisions, prune, `capture`, `capture_scoped` | none |
| 9 | `backend/crates/hiverouter-core/src/recovery.rs` (974) | **VERIFIED** (1-930) / PARTIAL (931-974) | Boot recovery steps 1-5, settlement predicates, graph repair, snapshot revision, tests 540-930 | 931-974 tail of the test module |
| 10 | `backend/crates/hiverouter-core/src/prompts.rs` (475) | **VERIFIED** (1-460) / PARTIAL (461-475) | Registry, licence gate, prompty parse, role rendering, `Persona`, limits | 461-475 tail (`parse_personas`/tests) |
| 11 | `backend/crates/hiverouter-core/src/provider_executor.rs` (1143) | **PARTIAL** | 30-149 (system prompts, pending usage), 150-259 (persona lookups, budget), 428-507 (`plan`, `repair_plan`, `execute`, `answer_escalation` head), 490-819 (`orchestrate_root`, `finalize_root`, continuations, `finish_run`), function index | 260-427 (`stream_reply` internals) and 820-1143 (`WirePlan`, helpers, unit tests) not read |
| 12 | `backend/crates/hiverouter-core/src/tool_loop.rs` (349) | **PARTIAL** | 1-120 (caps/constants/`ToolExecution`), 150-318 (`validate_plan`, tier specs, budgets, `next_turn`, tests head) | 121-149 (`plan_call` head), 319-349 (tests tail) not read |
| 13 | `backend/crates/hiverouter-core/src/store.rs` (3525) | **PARTIAL** | `set_run_status` (479), `insert_task` (519), `update_task_state` (576), `update_task_parent` (644), `task_result` (820), `task_identity` (1189), `memory_hyper_summary` (2237), `terminal_task_state_sql` (308) | Full file not read; not my lane owner (only settlement/guard touchpoints) |
| 14 | `backend/crates/hiverouter-core/src/db.rs` (1735) | **PARTIAL** | Migration 13 `task_identity` trigger + backfill (296-365) | Rest of schema/migrations |
| 15 | `backend/crates/hiverouter-core/src/ws.rs` (1619) | **PARTIAL** | `REWIRE_TASK` handler body 780-942, command index; `TEMPLATE_INSERT` enqueue/seed calls 1019-1045 | Full transport owner is another lane |
| 16 | `backend/crates/hiverouter-core/src/http.rs` (3715) | **PARTIAL** | Engine boot wiring 330-389; proposals read projection 2360-2517; operator authorize gate 2519-2629 | Rest of HTTP surface (other lane) |
| 17 | `backend/crates/hiverouter-core/assets/prompts/planner.prompty` | **VERIFIED** | Full 24 lines | — |
| 18 | `backend/crates/hiverouter-core/assets/prompts/manifest.json`, `personas.json` | **UNVERIFIED** | Referenced only via `prompts.rs:38-77` embedding | Not read; asset content is a different lane |
| 19 | `backend/crates/hiverouter-contract/src/state.rs` | **PARTIAL** | `is_terminal` region 99-106 via grep | Contract owner is another lane |
| 20 | `backend/crates/hiverouter-contract/schemas/contract-v1.json` | **UNVERIFIED** | Not read | Wire schema owner is another lane |
| 21 | `backend/crates/hiverouter-core/tests/hyperagent_governance.rs` (522) | **VERIFIED** | Whole file: detached admission, duplicate conflict, root refusal durability, foreign-run rewire refusal | — |
| 22 | `backend/crates/hiverouter-core/tests/run_lifecycle.rs` (1961) | **PARTIAL** | Full test-name index (30-1921); bodies 1229-1348 (root proposal, second-turn refusal, hive bank) | Other bodies not read line-by-line |
| 23 | `backend/crates/hiverouter-core/tests/recovery_multi_run.rs` (149) | **CITED-FROM-TEST** | Test name/line only (`two_interrupted_runs_are_both_represented_and_degradation_is_named`, :57) | Body not read |
| 24 | `backend/crates/hiverouter-core/tests/snapshot_history.rs` (265) | **CITED-FROM-TEST** | Names only (`:44`, `:73`, `:107`, `:131`, `:180`, `:264`) | Body not read |
| 25 | `backend/crates/hiverouter-core/tests/scale_s0.rs` (114) / `scale_s1.rs` (181) | **CITED-FROM-TEST** | Names only (`s0_probe_measures_small_fanout` :113; `s1_parallel_baseline_measures_41_tasks_at_bound_32` :101; `s1_serial_baseline_measures_41_tasks` :149) | Bodies not read |
| 26 | `backend/crates/hiverouter-core/tests/graph_authority.rs` (109) | **CITED-FROM-TEST** | Names only (:19, :39, :48, :65, :80, :93, :114) | Body not read |
| 27 | `backend/crates/hiverouter-core/tests/kill_restore.rs` (109) | **CITED-FROM-TEST** | Names only (:22, :48, :92) | Body not read |
| 28 | `backend/crates/hiverouter-core/tests/recovery_gates.rs` (104) | **CITED-FROM-TEST** | Names only (:64, :70, :98) — note: despite the name, these are provider-key health checks, not recovery tests | Body not read |
| 29 | `backend/crates/hiverouter-core/tests/prompt_assets.rs` (121) / `prompt_assets_wiring.rs` (252) | **CITED-FROM-TEST** | Names only | Bodies not read |

Budgets for this lane: 7 subagent slots total are root-managed (I am one child; I started none — depth limit 0). Background jobs started: 0. Provider calls: 0. Files written: 1.

---

## 2. `execution.rs` — the run engine

### 2.1 Module contract and constants

`execution.rs:1-11` claims four guarantees: hard caps enforced before spawn/reply, one cancellation token per run, "no fake success" (executor error fails the run with a named code and preserves partial state), and idempotency by `correlation_id` for 24 h. I checked each against code:

- Caps: **VERIFIED at spawn** (`execution.rs:1596`, `2029`, `1640`) but **provider-call claims are counted after increment** (`execution.rs:796-804`), and the call cap is checked *after* the counter moves — the call is refused, not made, so no unaccounted send occurs. VERIFIED.
- One token per run: `RunRegistry` (`execution.rs:583-651`), created in `submit_prompt` (`execution.rs:1528`), removed at terminal (`execution.rs:1889`). Per-task child tokens exist too (`execution.rs:716`, `1221-1257`). VERIFIED.
- No fake success: every executor error path calls `fail_task` before returning (`execution.rs:5346-5357`, `5376-5390`, `5491-5499`), and `complete_task` refuses to complete a non-`Running` task (`execution.rs:5536-5552`). VERIFIED.
- Idempotency: `receipt_run` short-circuits `SUBMIT_PROMPT` (`execution.rs:1514-1516`). VERIFIED for submit only; other mutating commands are **not** idempotent by correlation id in this file (e.g. `spawn_sub_hive`, `kill_worker`, `cancel_run`) — an **UNVERIFIED/absent** claim relative to the module doc. The doc's blanket statement over-reaches.

Bound values that govern behaviour: agent-message 2 000 chars, 32 messages/run, 120 s escalation wait, 2 persisted answer attempts (`execution.rs:140-146`); root prompt 24 000 chars, per-task root result 2 000 chars, digest 6 000 chars (`execution.rs:2217-2218`, `2241`).

### 2.2 Prompt assembly (and where the shipped planner asset is lost)

```rust
// execution.rs:2515-2529
fn planning_prompt(&self, run: &Run) -> String {
    if self.templates.get(PLANNER_TEMPLATE_ID).is_some() {
        let mut inputs = BTreeMap::new();
        inputs.insert("task".to_owned(), run.prompt.clone());
        if let Ok(messages) = self.templates.render(PLANNER_TEMPLATE_ID, &inputs) {
            let rendered = messages
                .iter()
                .map(|message| format!("{:?}: {}", message.role, message.content))
                .collect::<Vec<_>>()
                .join("\n");
            return prompt_with_authorized_lessons(&rendered, &self.authorized_lessons_snapshot());
        }
    }
    prompt_with_authorized_lessons(&run.prompt, &self.authorized_lessons_snapshot())
}
```

Derived pseudocode:

```
planning_prompt(run):
  if registry has "planner":
     msgs = render(planner template, {task: run.prompt})
     text = join("{role:?}: {content}" for each message)   # markers flattened into one string
     return text + authorized_lessons_block
  return run.prompt + authorized_lessons_block
```

Consequence: the built-in `assets/prompts/planner.prompty:16-24` declares `system:` / `user:` sections, but the engine serialises them into a **single user string**, and the provider's `plan()` does not call this renderer at all — it always sends its own `PLANNER_SYSTEM` constant (`provider_executor.rs:446` with the constant at `provider_executor.rs:26-33`). The asset's system text therefore never reaches the provider as a system message; the shipped template is decorative for planning. **VERIFIED** by the two cited sites. This is a real prompt-wiring defect, not a style issue: two instruction sources can disagree, and the model sees `System: ...` as literal text inside the user turn.

Also note `prompt_with_authorized_lessons` (`execution.rs:63-76`) appends a fenced `<<<AUTHORIZED_LESSONS_DATA` block capped at 8 lines × 400 chars — good, but it is appended to the **planner and every task prompt** (`execution.rs:4244-4247`, `4269-4272`, `4327`), so lessons are re-injected per call and can crowd small prompts.

### 2.3 Run submission, single-run policy, dispatch spawn

```rust
// execution.rs:1501-1543 (condensed, verbatim order)
let chars = prompt.chars().count();
if chars == 0 { return Err(Refusal::PromptEmpty); }
if chars > hiverouter_contract::command::MAX_PROMPT_CHARS { return Err(Refusal::PromptTooLong); }
if let Some(existing) = self.store.receipt_run(correlation)? { return self.store.get_run(existing); }
if self.executor.key_available() == Some(false) { return Err(Refusal::KeyMissing); }
if self.store.latest_active_run()?.is_some() || self.active_run().is_some() { return Err(Refusal::RunActive); }
let run = self.store.create_run(project_id, prompt, AgentState::Queued)?;
self.store.record_receipt(correlation, run.id)?;
self.store.append_message(run.id, ChatRole::User, prompt)?;
self.set_active(Some(run.id))?;
let token = self.tokens.create(run.id);
self.publish(Event::RunCreated(...));
tokio::spawn(async move { engine.dispatch(run_for_task, token).await; });
```

Pseudocode: validate → replay receipt → key gate → **single active run** → create row → receipt → user message → set active → token → publish `RUN_CREATED` → spawn `dispatch`. Invariants: no run row is created unless the key is available (honest refusal at admission, `execution.rs:1517-1520`) and unless no run is active. Failure mode: if `tokio::spawn` never gets scheduled (runtime shutdown), the run stays `queued` with an active token; there is no timeout. **Risk (LOW):** leaked active run on runtime teardown; boot recovery covers process death (`recovery.rs:119-137`), not a stuck task.

**Single-run policy is process-wide, not per-project** (`active_run: Mutex<Option<RunId>>`, `execution.rs:722`). The graph, spawn queue, A2A delivery set, metrics map, and usage map are also engine-wide (`execution.rs:707-734`), and the terminal cleanup clears the whole queue and the whole delivered set (`execution.rs:1896-1901`). Under the current single-run invariant that is correct; if that invariant is ever relaxed, cross-run task loss follows. **Risk (LOW now, HIGH if relaxed).**

### 2.4 Planning phase

```rust
// execution.rs:1904-1934 (condensed)
self.store.set_run_status(run.id, AgentState::Planning, None)?;
self.claim_provider_call(run.id)?;
let planning_prompt = self.planning_prompt(run);
let first_plan = tokio::select! {
    _ = token.cancelled() => return Err(Refusal::ToolCancelled),
    result = self.executor.plan(run.id, &planning_prompt, token) => result,
};
self.add_run_usage(run.id, self.executor.take_pending_usage());
let plan = match first_plan {
    Ok(plan) => plan,
    Err(refusal) if matches!(refusal, Refusal::SchemaInvalid | Refusal::ProviderInvalidResponse) => {
        self.claim_provider_call(run.id)?;                       // one repair, counted
        let retried = tokio::select! { ... executor.repair_plan(...) ... };
        self.add_run_usage(...); retried?
    }
    Err(refusal) => return Err(refusal),
};
plan.validate()?;
```

Pseudocode: `planning` → claim 1 call → plan (cancellable) → account usage → on schema/invalid-response only, claim a second call and repair → `Plan::validate`.

Invariants: exactly one provider call per plan attempt; the repair is itself counted; a cancellation before the reply returns `E_TOOL_CANCELLED`. Failure modes: any other provider refusal (e.g. `E_PROVIDER_RATE_LIMIT`) fails the run immediately without repair — correct, since repair is only for schema errors. Note `add_run_usage` is called **before** inspecting the plan result, so failed-plan token spend is recorded (honest accounting). VERIFIED.

`Plan::validate` (`planner.rs:50-129`) is strict: non-empty, ≤ 40 nodes (`planner.rs:54-56`), unique ids, known deps, `role` non-empty and `depth != 0`, `depth <= 3`, parent must exist at `depth-1`, parent may not also be a dep, no parent-less depth ≥ 2 node, dependency on a descendant refused, parent-chain loop refused, topological order must consume all nodes. This is the strongest part of the planning lane; `backend/crates/hiverouter-core/tests/run_lifecycle.rs:906` (`strict_hierarchy_refuses_direct_hyper_agent_and_super_subagent_edges`) and `planner.rs` unit tests (245-339) cover it. **VERIFIED.**

One residual `topo_order` defect: duplicate deps inflate indegree twice but are decremented once (`planner.rs:136-165`), so a duplicate dep is reported as `E_SCHEMA_INVALID` (a false cycle) rather than as "duplicate dependency". Behaviour is fail-closed; message is misleading. **LOW.**

### 2.5 Root row creation and planned-node materialisation

```rust
// execution.rs:1937-1951
let root = TaskRecord { id: TaskId::new(), run_id: run.id, parent_task_id: None,
    role: "planner".to_owned(), depth: 0, spawn_seq: 1, state: AgentState::Running, rev: 0 };
self.store.insert_task(&root)?;
self.graph.add_task(root.id, None)?;
self.graph.set_metadata(root.id, "hyperagent", &root.role, Some("HyperAgent"))?;
self.graph.set_state(root.id, AgentState::Running)?;
```

Then the run-bank writes (`execution.rs:1955-1977`), context seeding (`1978`), `NODE_SPAWN` (`1979`), and planned-node insertion sorted by `(depth, index)` (`1987-2052`). Pseudocode:

```
root = Task("planner", depth 0, Running); persist row + graph node + metadata(hyperagent)
record run bank: prompt (2000 chars), plan digest (role:depth list, <=32)
seed root context; publish NODE_SPAWN(root)
planned = sort(plan.nodes by (depth, index))
for node in planned:
    agents = count(tasks in run)
    parent_id = node.parent or root.id
    if parent unknown in this plan -> E_SCHEMA_INVALID
    parent_tier = tier(tasks[parent])
    child_class = role_class(node.role) or E_PERSONA_UNKNOWN
    validate_spawn_role(parent_tier, node.role)
    parent_class = task_identity(parent).class or role_class(parent.role)
    if parent_class and not class_may_spawn(parent_class, child_class) -> E_TOOL_NOT_ALLOWED
    if subagent role and node.depth < 2 -> E_SPAWN_DEPTH
    siblings = count(tasks with parent == parent_id)          # per-parent, not plan index
    caps.check_spawn(node.depth, siblings, agents)
    insert row; graph.add_task; set_metadata(kind by role); seed context; publish NODE_SPAWN
```

Invariants: depth-first insertion guarantees the parent row/graph node exists; fan-out is per parent (`execution.rs:2023-2028`); planned roles are validated **before** the child row exists (`1996-2003`); the class matrix is applied on top of the tier matrix (`2004-2019`).

Failure modes: (a) **unknown role aborts the run** with `E_PERSONA_UNKNOWN` (`execution.rs:2002`) — a planner that writes `"researcher"` or `"analyst"` kills a whole run even though `task_agent_tier` would have treated it as a Worker (`execution.rs:284-299`); test `run_lifecycle.rs:883` asserts refusal for `spawn_sub_hive` but the run-fatal effect on the planner path is **CITED-FROM-TEST-by-name only** and otherwise **VERIFIED by code**. (b) `agents` is re-read per node (`1991`), so the agent cap is exact; the 41st node cannot be inserted. (c) A model-supplied `node_id` that collides with an existing task id fails `insert_task` and the run fails — no collision check is performed before insertion (row PK is the guard). **LOW.**

### 2.6 Root control checkpoint (`root_checkpoint`)

```rust
// execution.rs:4283-4330 (condensed)
if !self.root_orchestrator { return Ok(()); }
let prompt = format!("{run.prompt}\n\nYou are the HyperAgent orchestrator. The planner proposed this DAG: {plan_digest}. \
    You may call one or more of: spawn_superagent ... convene_evolution_vote ... finish. \
    Never spawn a direct Agent or Subagent. Return no prose in this control turn.\n");
let has_planned_branch = plan.nodes.iter().any(|node| node.depth == 1);
const MAX_ROOT_TOOL_CALLS: usize = 4;
let mut control_results = String::new();
...
for turn in 0..2 {
    if token.is_cancelled() { return Err(Refusal::ToolCancelled); }
    let sink: DeltaSink = Arc::new(|_| {});
    self.claim_provider_call(run.id)?;
    let reply = self.executor.orchestrate_root(run.id, root, &turn_prompt, token, &sink).await?;
    self.add_run_usage(run.id, reply.usage.clone());
    if reply.tool_calls.is_empty() { return Ok(()); }
    ...
    for call in reply.tool_calls.iter().take(MAX_ROOT_TOOL_CALLS) { ... }
}
Ok(())
```

Pseudocode for the gate table: turn 1 may create; turn 2 may only read/propose/finish; `spawn_superagent` refused when any depth-1 plan node exists **or** one was already spawned (`execution.rs:4383-4393`); `spawn_remote_agent` refused on duplicate display name in the hive or in this run (`4417-4442`); `spawn_daemon` refused on duplicate (`4464-4487`); `propose_improvement` validates 80/800/2000-char bounds and dedupes by content (`4558-4623`), records a `governance` block that states five approvals are required and `can_apply: false` (`4635-4659`), and on turn 0 auto-convenes four voters (`4676-4712`); `convene_evolution_vote` requires a recorded proposal id and four seats (`4717-4806`); `finish` returns (`4807`); any other tool → `Err(ToolNotAllowed)` (`4808-4811`).

Invariants: root tools are **local to this method** and never enter the global registry (`execution.rs:4280-4282`); each decision is durably recorded (`record_root_gate_decision`, `5229-5263`); the root cannot skip a level because `spawn_sub_hive(root.id, "super")` is the only creation path (`4395`, `4776`). All **VERIFIED**, and covered by `backend/crates/hiverouter-core/tests/run_lifecycle.rs:1178` (refuses an extra branch), `:1301` (second turn cannot create a boundary), `:1230` (proposal never auto-applies); `hyperagent_governance.rs:392-430` (refusal durability).

Defects:
1. **Run-fatal root model errors.** `ok_or(Refusal::SchemaInvalid)?` and `spawn_sub_hive(...)?` propagate out of `root_checkpoint` (`execution.rs:4462`, `4598`, `4730`, `4776`), and `orchestrate_root(...).await?` propagates any provider refusal (`4341-4344`). A malformed `spawn_daemon` call (missing `display_name`) kills the whole run **after** planning and **before** any task executes. Comparable leaf-tool errors are converted to named tool-result refusals (`execution.rs:2629-2649`). Severity **MEDIUM** (availability + asymmetry; honesty is preserved because the run fails loudly).
2. **Cap exhaustion during the optional control turn fails the run.** `claim_provider_call` at `4333` returns the error via `?`; there is no "skip control and continue" path. Severity **LOW-MEDIUM**.
3. **Wire/tool drift.** The engine accepts `convene_evolution_vote` and its prompt advertises it (`execution.rs:4304`, `4717`), but the provider's `ROOT_CONTROL_SYSTEM` (`provider_executor.rs:39-45`) omits it and `orchestrate_root` does not advertise it as a tool (`provider_executor.rs:583-666` advertises spawn_superagent, spawn_remote_agent, spawn_daemon, read_memory_summary, propose_improvement, finish). A conforming provider can never call it; a non-conforming one can. Severity **LOW** (the same capability is reachable through `propose_improvement`'s auto-convene at turn 0).
4. **Final-root prompt lies about tools.** `ROOT_SYSTEM` says the root "may call spawn_remote_agent at most once" (`provider_executor.rs:47-53`), but `finalize_root` advertises no tools at all (`provider_executor.rs:682-689`, comment at `672-673`). The model may emit a textual tool call that nothing executes. Severity **LOW** (no fake success, but misleading asset).

### 2.7 Dispatch: readiness, serial and parallel

Serial scheduler (`execution.rs:2096-2193`):

```
completed = {}
while |completed| < |plan.nodes|:
    fail if token cancelled
    next = first node where deps ⊆ completed AND children(node) ⊆ completed
        (none -> E_SCHEMA_INVALID)
    prompt = run.prompt + for each dep/child: "DIRECT CHILD RESULT"/"Completed dependency" + id + role + bounded text
             + "Aggregate the completed child results..." if it has children
    result = execute_task_with_prompt(...)
    commit snapshot with current run status
    on Ok: mark completed
    on Err(ToolCancelled|TaskTerminal) with non-cancelled token and fresh task terminal: mark completed and continue  # per-task kill
    else: return Err
```

Parallel scheduler (`execution.rs:2294-2369`): identical readiness predicate, `take(parallel_dispatch)`, `JoinSet`, one snapshot commit per joined task, `set.abort_all()` and drain on first non-per-task-kill error. Invariant: provider, DB and graph stay the authorities; concurrency is only a scheduling hint (comment `2291-2293`). **VERIFIED** (test name `scale_s1.rs:101` measures 41 tasks at bound 32; `run_lifecycle.rs` 1498/1536/1921 cover caps, call counting, streaming).

Defect (design): readiness requires **all direct children** to complete before the parent runs, even if the parent's own `deps` are satisfied. That is correct for aggregation, but it means a parent never gets partial child results, and a parent cannot start early. Not a bug; a capability gap (no streaming/pipelined aggregation).

Defect (state): `execute_task_with_prompt` applies `Start` only when `start_task=true` (`execution.rs:5317-5323`); the root final turn passes `false` (`execution.rs:4274`) and instead forces the graph node to Running (`execution.rs:2214`). If the DB root row were not `Running` (e.g. killed), the `is_terminal_task_state` check at `2211-2213` catches it and the run fails `E_TASK_TERMINAL`; killing the root therefore turns the run into `failed`, not `cancelled`. **CITED-FROM-TEST-by-name:** `kill_restore.rs:48`; not the same scenario. Marked as **observed by code, not test-verified**.

### 2.8 Spawn queue (`enqueue_spawned_task` / `next_queued_task` / `dispatch_spawned_tasks`)

```rust
// execution.rs:1260-1269, 1386-1395, 2463-2513 (condensed)
fn enqueue_spawned_task(&self, task_id: TaskId) -> Result<(), Refusal> { ... if !queue.contains(&task_id) { queue.push(task_id) } }
fn next_queued_task(&self) -> Option<TaskId> { let mut queue = self.spawn_queue.lock().ok()?; if queue.is_empty() { None } else { Some(queue.remove(0)) } }
async fn dispatch_spawned_tasks(&self, run: &Run, token: &CancellationToken) -> Result<(), Refusal> {
    while let Some(task_id) = self.next_queued_task() {
        if token.is_cancelled() { return Err(Refusal::ToolCancelled); }
        let task = match self.store.get_task(task_id) { Ok(t) => t, Err(Refusal::TaskNotFound) => continue, Err(e) => return Err(e) };
        if task.run_id != run.id || is_terminal_task_state(task.state) { continue; }
        if task.state != AgentState::Queued && task.state != AgentState::Running { continue; }
        let result = self.execute_task(run, &task, token).await;
        ...
        match result { Ok(_) => {}, Err(refusal) => { /* per-task kill tolerated; else fail_task + return Err */ } }
    }
    Ok(())
}
```

Pseudocode: pop → cancel check → re-read row (killed/removed tasks skipped) → foreign-run and terminal rows skipped → execute to terminal → snapshot. Invariants: caps were checked before the row existed (`execution.rs:2286-2289` comment); queue is drained before the run can become terminal (`execution.rs:2198`). Failure mode: if a task is left `AwaitingApproval` (not possible without a waiter in this path) the loop skips it and `settle_unfinished_tasks` later cannot settle it either (it only handles Queued/Running) — then the root final still runs and the run completes with an `AwaitingApproval` task, which `reconcile_terminal_tasks` then settles to `Completed`. **Reachability is UNVERIFIED** (requires a suspended task whose waiter vanished in-process, which the approval design tries to prevent). I flag it as a **latent settlement hole (MEDIUM)** rather than a confirmed bug.

Test coverage: `run_lifecycle.rs:1649` (planned parent chain persisted, not flattened), `:1178` (extra branch refused), `:883` (unknown role), scale tests. `TEMPLATE_INSERT` enqueues through the same queue (`ws.rs:1019-1045`).

### 2.9 Task execution and the tool loop

```rust
// execution.rs:5304-5316
if let Err(refusal) = self.claim_provider_call(run.id) {
    self.fail_task(run, task, AgentState::Failed)?;
    return Err(refusal);
}
if start_task { let Some(start) = self.apply_task_state(task.id, StateEvent::Start)? else { return Ok(None) }; ... }
```

```
execute_task(prompt):
  claim provider call            (fail task + Err on cap)
  if start: apply Start (None => task already terminal => Ok(None), no provider call)
  publish NODE running; commit snapshot
  reply = select(cancel -> Cancel task + Err | execute/finalize_root)
  on Err: fail_task(Cancelled|Failed); return Err
  publish returned deltas if sink unused
  loop:
    if no tool calls: break
    if cancelled: fail(Cancelled)
    turn = execute_tool_turn(...)   # may persist TOOL_RESULT rows
      Err -> fail task unless already terminal; return Err
      cap_reached -> fail(Cancelled? no: Failed) + E_CALL_LIMIT
      executed empty & budget_refused -> Failed + E_CALL_LIMIT
      executed empty -> Failed + E_PROVIDER_INVALID_RESPONSE
    if a successful `finish` call -> complete_task(verdict) and return
    if a successful `report_result` call -> complete_task(summary) and return
    transcript.push(turn)
    claim provider call
    continuation = select(cancel | continue_with_tools_with_updates)
    on Err: fail task; return Err
  complete_task(final reply)
```

Citations: `execution.rs:5313-5330`, `5336-5359`, `5366-5410` (loop + honest failure mapping), `5416-5431` (`finish`), `5437-5463` (`report_result`), `5464-5503` (transcript + continuation), `5505-5516` (final completion). **VERIFIED.**

Invariants: only the final reply is persisted once (`complete_task` appends one assistant message, `execution.rs:5553-5557`); a `finish`/`report_result` verdict is persisted as the final message without another round trip; a refused `finish` falls through to the transcript (comment `5411-5415`) rather than fabricating completion. Streaming: a fresh sink per provider call so returned deltas are never duplicated (`execution.rs:2736-2792`; test `run_lifecycle.rs:1921`).

`execute_tool_turn` (`execution.rs:2540-2734`) order: persisted turn/call budgets → `plan_call` → tier+persona authorization → publish `TOOL_CALL` → tool-specific path (spawn/ask/sibling/detached/vote/approval/read-only) → publish `TOOL_RESULT` + durable row. Notably:

- `authorize_tool_call` uses the **real** task tier from the role, plus the persona allowlist when the context was persona-scoped (`execution.rs:1470-1494`). Empty persona allowlist means "no tools" (`execution.rs:1485-1486`). VERIFIED; test `run_lifecycle.rs:1727` (persona allowlist seeded).
- Unknown-role tasks get `WorkerAgent` capability (`execution.rs:296-298`) — never wider. VERIFIED.
- Read-only execution requires a `WorkspaceJail`; without it, `E_FILE_TREE_UNAVAILABLE` and nothing runs (`execution.rs:3009-3015`). VERIFIED.
- `web_fetch`: https-only, default port only, no credentials, no localhost, DNS + per-address SSRF denial including NAT64/IPv4-mapped, no redirects, 15 s timeout, 1 MiB cap, cancellation-aware (`execution.rs:2819-2993`). Unit tests at `execution.rs:5606-5628`. **VERIFIED, strongest security code in the lane.**
- `ask_parent`: 120 s, duplicate-question refusal, per-run 32-message budget, attempt audit, parent-answer first, operator fallback (`execution.rs:3602-3859`). `send_to_sibling`: same-parent only unless approved (`3861-3981`). `spawn_agent`: parent-claim verification, empty-role refusal, caps before insert, inline execution, result returned as tool result (`3990-4101`). `vote_evolution`: SuperAgent-only, proposal must exist, duplicate voter refused (`4824-4924`). `request_detached`: SuperAgent/Agent only, 12-char reason, 4-principal run cap, idempotent by `request_id` fingerprint, audit rows (`4931-5158`). All **VERIFIED**.

**Tool-budget race (MEDIUM-HIGH).** `used_calls` is read once per turn (`execution.rs:2554-2562`), each call checks `check_call_budget(used_calls + index)` (`2607-2619`), and the increment happens after the turn (`2720-2727`). With `parallel_dispatch` up to 32 (`http.rs:365-379`), N concurrent tasks can all read `39` and all execute, so the true count can exceed `MAX_CALLS_PER_RUN = 40` by up to `(parallel_dispatch - 1) × calls_in_turn`. The increment itself is atomic (`store.rs` `increment_run_tool_counters`, cited by call site only), so the *counter* is honest; the *admission decision* is not. No test covers concurrent budget exhaustion (test names in `run_lifecycle.rs` are serial). **VERIFIED by code; correctness impact verified by reasoning, not by test.**

### 2.10 Approvals (suspend / resume / deny / timeout / cancel)

`suspend_for_approval` (`execution.rs:3151-3280`): budget check (8 waits/run, `3174-3185`) → create approval row → **arm waiter before persisting** (`3192-3200`, drop-disarm guard `152-161`) → persist run+task `awaiting_approval` (`3106-3125`) → snapshot → publish `APPROVAL_REQUEST` → check for an already-committed decision (`3214-3231`) → race `token.cancelled()` vs TTL wait (`3238-3244`) → dispatch on wake kind: cancel/closed → `finish_cancelled_wait`; timeout → CAS expire then `finalize_approval_timeout` or apply committed decision; resolved → allow (resume+execute exactly once) or deny (`3272-3277`).

Single-owner guarantees I could verify: timeout CAS winner publishes one terminal `RUN_STATUS failed` with `E_APPROVAL_TIMEOUT` under the lifecycle lock (`execution.rs:3543-3573`); a late expired label cannot recurse (`execution.rs:3484-3503`, comment `3305-3308`); cancel consumes the row and resumes the run only if the **run** was not cancelled (`execution.rs:3576-3600`); approve re-authorizes before `execute_approved` so consent is not consumed by a refused call (`execution.rs:3328-3355`); `web_fetch` honors the global side-effect kill switch (`execution.rs:3356-3390`). **VERIFIED.**

**Concurrency defect (HIGH).** The run has a single status and `Suspend` is legal only from `Running` (`run_state.rs:510-512`); `persist_approval_transitions` calls `apply_run(current_run.status, Suspend)` (`execution.rs:3106-3112`). Two tasks that suspend concurrently in the parallel pool: the second reads `awaiting_approval` and gets `E_SCHEMA_INVALID`, its approval row is expired (`execution.rs:3203-3205`), the error propagates and the run fails. Production default is 32 concurrent tasks (`http.rs:365-379`), so any plan whose siblings both use an approval-gated tool can hit this. No test covers two simultaneous waits (`suspension.rs` is not in my lane; names in `run_lifecycle.rs` show single-wait tests). **VERIFIED by code path; reachability in production VERIFIED by the parallel default; test coverage UNVERIFIED.**

### 2.11 Cancellation, kill, terminalisation, reconcile, prune

- `cancel_run` (`execution.rs:1672-1689`): terminal check → `tokens.cancel` → returns; the engine owns the single terminal frame. If the token is missing (run finished between the read and the cancel) it reports `E_RUN_TERMINAL`. **VERIFIED.**
- `kill_worker` (`execution.rs:1691-1726`): only `Running`/`AwaitingApproval`; applies `Kill` through the state machine (terminal-guarded), cancels the **task** token, publishes, and if the task was awaiting approval, resumes the run. **VERIFIED.** Tests: `hyperagent_governance.rs` does not cover kill; `kill_restore.rs:48` is crash recovery.
- Terminalisation (`execution.rs:1821-1902`): reads persisted status first (a wait-timeout or `CANCEL_RUN` already wrote it), maps outcome (`Ok → completed`, `ToolCancelled → cancelled`, other → `failed` + code), **reconciles every task**, writes+publishes the terminal frame once under the lifecycle lock, commits a post-terminal snapshot, prunes to 16 revisions, releases `active_run`, removes the token, persists usage, `finish_run`, clears counters/metrics/A2A/queue. **VERIFIED.** Test names: `run_lifecycle.rs:1600` (cancel stops the run), `:1562` (provider error preserves partial state).
- `reconcile_terminal_tasks` (`execution.rs:1784-1819`): maps run outcome to a task terminal state (`completed|cancelled|interrupted|failed`), sorts by `spawn_seq`, skips already-terminal rows but repairs graph disagreement, otherwise raw-writes the terminal state and publishes. **This is where structural rows become `Completed` without ever executing.** Cite `execution.rs:1786-1791` (comment admitting it) and `2206` (root final gating).
- `snapshot_revision` (`execution.rs:1732-1738`) counts **indexed rows** rather than `max+1`; `prune_run(run, 16)` deletes rows and files (`execution.rs:1880`, `snapshot.rs:206-231`). After pruning, the count is smaller than the highest revision, so a later commit can allocate an existing revision number and `SnapshotStore::write` uses `INSERT OR REPLACE` + the same filename (`snapshot.rs:83-113`), silently overwriting. Recovery instead uses `max(indexed, on_disk) + 1` (`recovery.rs:480-494`). Two incompatible allocators. Reachability: only a post-prune commit for the same run; none is currently issued after terminalisation. **Risk MEDIUM, latent.**

---

## 3. `run_state.rs` — the pure lifecycle authority

Purpose: deterministic `(from, event) -> {to, effect}` for run and task graphs, no I/O (`run_state.rs:1-46`). Sets: `RUN_STATES` (`64-73`), `RUN_TERMINAL_STATES` (`76-81`), `TASK_STATES` (`87-96`, adds `Killed`, excludes `Planning`), `TASK_TERMINAL_STATES` (`99-105`), events (`131-156`), effects (`205-250`), errors mapped to frozen refusals (`337-354`).

```rust
// run_state.rs:492-538 (condensed edge table)
match (from, event) {
    (Queued, Plan) => (Planning, MarkPlanning),
    (Queued | Planning, Start) => (Running, StartRun),
    (Running, Complete) => (Completed, MarkCompleted),
    (Queued | Planning | Running, Fail) => (Failed, MarkFailed),
    (Queued | Planning | Running | AwaitingApproval, Cancel) => (Cancelled, CancelRun),
    (Running, Suspend) => (AwaitingApproval, RequestApproval),
    (AwaitingApproval, Resume) => (Running, ResumeFromApproval),
    (AwaitingApproval, Deny) => (Running, FeedDeniedToolResult),
    (AwaitingApproval, Timeout) => (Failed, FailOnApprovalTimeout),
    (Queued | Planning | Running | AwaitingApproval, Restart) => (Interrupted, MarkInterrupted),
    _ => if event in {Resume, Deny, Timeout} { E_APPROVAL_REPLAY } else { E_SCHEMA_INVALID }
}
```

Task table (`run_state.rs:581-622`) additionally maps `(Running, Kill) -> Killed` and `(AwaitingApproval, Kill) -> Cancelled`, and has no `Planning` edges.

Invariants: every transition changes state (`545-547`, `629-631`); one effect per edge by construction; terminal states have no outgoing edge (`574-576`); approval decisions with no wait are `E_APPROVAL_REPLAY` not a silent no-op. Failure modes: `fail` from `awaiting_approval` is deliberately illegal (`816-843` tests) — the engine must use Timeout/Cancel; any code calling `Fail` during a wait gets `E_SCHEMA_INVALID` (this is exactly the parallel-approval bug's blast radius).

Coverage: unit tests 667-1273 (state sets, submit-once, planning/start, complete-only-running, fail edges, suspend-only-from-running, resume/deny distinct effects, terminal predicate). Plus `suspension.rs` (not in my required list; **UNVERIFIED**).

Gaps: no `Run`-level `Killed`; no per-task `Planning`; no event for "retry"; no event for "reassign"; no effect for re-planning. So the state machine itself cannot express the decisions a governor would need to make.

---

## 4. `planner.rs` — plan DAG

Covered above (§2.4). Additional facts: `PlanNode.parent` is `#[serde(default)]` (`planner.rs:26-27`) so legacy flat plans parse; `leaf_order` (`176-191`) is unused by the engine's dispatch (the engine uses its own children map at `execution.rs:2078-2091`) — dead-ish API kept for callers/tests. `aggregation_order` (`199-222`) is only used to decide whether a root-final turn is needed (`execution.rs:2194`). Missing capability: no per-node tool/budget hints, no expected-output schema, no acceptance criteria, no cost estimate, no per-node priority — a plan cannot express governance intent.

---

## 5. `graph.rs` — authoritative task graph

Purpose and ownership: `petgraph::DiGraph` is the authority; all mutations go through `GraphAuthority` (`graph.rs:1-7`). Operations: `add_task` (parent must exist, duplicate id refused, `82-119`), `set_metadata` (`126-140`), `begin_rewire`/`finish_rewire` (`147-164`), `rewire` (CAS on `expected_rev`, in-flight guard, self/cycle refusal, rollback on defensive cycle, `174-234`), `set_state` (`236-247`), `snapshot` (`253-256`), `restore_from_snapshot` (validates dangling edges and cycles, replaces the whole image, `337-360`).

Invariants: edges are parent→child; cycle insertion is impossible; `rev` bumps on every mutation; snapshots sort nodes/edges and include a topological order (`363-386`). Failure modes: lock poisoning → `E_INTERNAL_ERROR`; corrupted snapshot edges → `E_SNAPSHOT_CORRUPT` (never a silent reset).

**Gap (MEDIUM):** `set_state` performs **no transition validation** — any `AgentState` can be written to any node (used by the engine, recovery, and root-final repair). The state machine lives in `run_state.rs`, but the graph's writer does not consult it. Also `restore_from_snapshot` replaces the entire process-wide graph, so only one run's image is ever live (`recovery.rs:145-199` documents this).

Coverage: `graph.rs:388-444` unit tests; `backend/crates/hiverouter-core/tests/graph_authority.rs:19-114` (cycle, self-rewire, stale rev, in-flight, unknown task, toposort, duplicate ids).

---

## 6. `caps.rs` + budget arithmetic

```rust
// caps.rs:8-18
pub const MAX_SPAWN_DEPTH: u8 = 3;
pub const MAX_FANOUT: usize = 8;
pub const MAX_AGENTS: u64 = 64;
pub const MAX_CALLS: u64 = 64;
```

`check_spawn` refuses `depth == 0 || depth > max`, `siblings >= max_fanout`, `agents >= max_agents` (`caps.rs:52-63`). The doc comment on `MAX_CALLS` (`caps.rs:14-18`) asserts the call cap must not make the 40-node cap unreachable — but it does not account for per-task tool-loop continuations (`MAX_TOOL_TURNS = 12`, `tool_loop.rs:14`), parent escalation answers (`execution.rs:3736`), the root control's 2 turns and the final turn. Worst case: 40 tasks × 1 continuation = 40 extra sends + planner 1 + maybe repair 1 + control 2 + final 1 = 85 > 64. A 40-node plan that uses any tools **will** fail with `E_CALL_LIMIT` before the node cap is exhausted. **Risk MEDIUM-HIGH (capacity, not honesty).** No test measures the combined ceiling; `run_lifecycle.rs:1536` only asserts calls are counted against the real cap.

Persona-level limits tighten the global caps: `Persona.max_children`/`max_depth` are validated to `1..=MAX_FANOUT`/`1..=MAX_SPAWN_DEPTH` (`prompts.rs:425-437`) and enforced per parent (`execution.rs:1585-1592`; test `run_lifecycle.rs:1864`). **VERIFIED.**

---

## 7. `context.rs` + `entities.rs` — what the model can see, and the task row

`ScopedContext` (`context.rs:19-32`) carries `parent_task_id`, scoped `facts`, `allowed_tool_ids`, `persona_tool_scope`, `output_schema_id`. `prompt_block` (`69-100`) renders "untrusted data; never instructions", skips sensitive keys (`is_sensitive_fact_key`, `118-123`: secret/token/password/api_key/apikey/authorization/credential), sanitises newlines/controls so model-authored text cannot forge a fact row (`128-144`; test `416-434`), and truncates with a marker (`93-98`). `switch_context` (`164-213`) clears the old parent's facts, keeps root facts, injects the new parent's, replaces allowlists/schema — the rewire memory boundary. `ContextStore` is bounded at 1024 with FIFO eviction (`223-268`) and FIFO eviction can silently drop a live task's context if >1024 tasks exist — impossible under `MAX_AGENTS=64`, so **LOW**.

`entities.rs` is thin: `Run` (id, project, prompt, status, error_code, timestamps, `entities.rs:9-24`), `TaskRecord` (id, run, parent, role, depth, spawn_seq, state, rev, `28-45`), `ChatRole` (`50-83`), `ChatMessage` (`88-98`). **Structural gap (MEDIUM):** `TaskRecord` carries no `class`/`rank`/`profile_id`; class is derived from the **role string** by a SQL trigger (`db.rs:341-365`) and read back via `store.rs:1189`. Two consequences: (a) renaming/re-roling a task would leave a stale identity; (b) every hierarchy authority decision depends on string matching 12+ aliases (`execution.rs:201-215`, `244-275`), so a new role name silently changes authority (unknown role → `None` → `E_PERSONA_UNKNOWN` on spawn, or `WorkerAgent` for tier). No test asserts identity stability across a role change; rewire never rewrites class/rank.

---

## 8. `snapshot.rs` — durability

Versioned envelope with `run_status` in v2 (`snapshot.rs:22-43`); atomic write = create dir, write temp, `sync_all`, rename, fsync dir on unix, then SQLite index insert (`77-118`); read validates `run_id`/`revision` after migration (`125-134`); `list_revisions`/`latest`/`runs_on_disk` (`140-195`); `prune_run(keep)` deletes oldest files+rows (`206-231`); `migrate_v1_to_v2`/`v2_to_v1` are inverses (`262-303`); `capture_scoped` filters nodes/edges/toposort to one run's task ids (`314-335`) — this is what prevents cross-run leakage into snapshots and is used at every commit (`execution.rs:1769`). Invariant: a corrupt snapshot is refused, never a workspace reset. **VERIFIED.** Coverage: `run_lifecycle.rs`-adjacent `snapshot_history.rs:73` (migration roundtrip), `:107` (corrupt), `:131`/`:180` (history API), `:264` (prune keeps newest) — **CITED-FROM-TEST**; plus `kill_restore.rs:92` (corrupt snapshot at recovery refused).

Honesty gap (LOW): `SnapshotEnvelope.created_at` uses `Timestamp::now()` at capture (`snapshot.rs:351`), and pruning keeps the highest **numeric** revision, not the newest `created_at`; the two can disagree only if a revision number is reused (see §2.11) — same root cause.

---

## 9. `recovery.rs` — boot recovery

```rust
// recovery.rs:119-137 (condensed)
let runs = store.list_runs(500)?;
let mut interrupted: BTreeSet<RunId> = runs.iter().filter(|r| r.status == Interrupted).map(|r| r.id).collect();
for run in &runs {
    if run.status.is_terminal() { continue; }
    let error_code = if run.status == AwaitingApproval { Some(Refusal::ApprovalTimeout.code()) } else { None };
    store.set_run_status(run.id, AgentState::Interrupted, error_code)?;
    interrupted.insert(run.id); report.interrupted_runs.push(run.id);
}
for run_id in &report.interrupted_runs { let _ = store.expire_open_agent_messages(*run_id); }
```

Then: restore snapshots oldest-first so the newest image is live (`151-193`), name the degradation for superseded runs (`196-199`), settle every non-terminal task through `apply_task(Restart)` (`205-251`, `settlement_target` at `385-397`), repair graph nodes from rows or rebuild them (`406-477`), settle graph-only nodes of the restored image (`257-283`), and commit one new revision per changed run via `max+1` (`288-333`, `480-494`). `publish_recovered_node_states` catches the bus up after boot (`350-376`).

Invariants: a terminal task is never resurrected (`207-229`); an interrupted run with a crashed mid-recovery is still settled (`740-763` test); a corrupt snapshot is reported and settlement still happens (`814-848`); recovery is idempotent (`916`). Approval opens are expired rather than left answerable (`139-143`) — the in-process waiter cannot survive a boot (`execution.rs:705-707`). **VERIFIED** and covered by `recovery.rs` internal tests 616-930 plus `recovery_multi_run.rs:57` (**CITED-FROM-TEST**).

**Hierarchy-integrity defect (MEDIUM):** when a task's parent node is missing (its run's image was superseded by the single graph image), recovery deliberately inserts the task **at top level without a parent edge** (`recovery.rs:417-430`, `446-469`), logging a warning. The DB row keeps `parent_task_id`, so DB and graph now disagree about the hierarchy — the very invariant `assert_db_and_graph_agree` checks only for *state*, not parent. A subsequent `spawn_sub_hive` reads the **row** (`execution.rs:1558`, `1577-1582`), so authority is intact; but the graph/UI topology is flattened. **VERIFIED by code.** No test asserts parent equality after recovery (tests compare state only, e.g. `recovery.rs:605-614`).

**Honesty gap (LOW):** an `awaiting_approval` run is recovered as `interrupted` **with `E_APPROVAL_TIMEOUT`** (`recovery.rs:129-134`). A crash is not a timeout; the error code misattributes the cause. The report and DB keep the distinction (`interrupted` status), but the `error_code` column will read `E_APPROVAL_TIMEOUT`.

---

## 10. `prompts.rs`, personas, and the root assets

`prompts.rs` embeds the manifest and 8 `.prompty` files (`prompts.rs:38-77`), enforces a licence/source/fetch gate (`219-244`, `ALLOWED_LICENCES` 23-30), forbids `${env:}`/`${file:}` (`284-286`), isolates the pinned `prompty` parser behind `catch_unwind` (`292-296`), and renders role sections (`320-360`). `Persona` carries tier, template, tool allowlist, output schema, refusal policy and optional child/depth caps (`387-408`), validated against the registry and global caps (`415-438`). Failure modes are named (`E_TEMPLATE_LICENSE_MISSING`, `E_TEMPLATE_REF_FORBIDDEN`, `E_TEMPLATE_UNKNOWN`, `E_TEMPLATE_INVALID`, `E_PERSONA_UNKNOWN`). **VERIFIED**; `prompt_assets.rs:129` and `prompt_assets_wiring.rs:43-249` (**CITED-FROM-TEST**) cover the embedded/refusal paths.

Root-governance assets: `provider_executor.rs:39-45` (`ROOT_CONTROL_SYSTEM`) and `47-53` (`ROOT_SYSTEM`) define the controller persona; the *effective* control prompt is the engine-built string at `execution.rs:4299-4309`, which is more specific than the provider constant and includes the evolution tools. So the root's behaviour is governed by a **concatenation of two independently maintained prompts** with known drift (§2.6 items 3-4). `planner.prompty:17-21` states the strict 3-level rule, matching `execution.rs:239-275`. **VERIFIED.**

Missing capability: no persona/tier is consulted for root **authority** (the root is whatever role string `"planner"` maps to); personas govern tools and prompts only. There is no per-persona budget, no per-persona model choice, and no output-schema enforcement anywhere in this lane — `output_schema_id` is stored (`context.rs:31`, `execution.rs:1321`) and **never read** for validation. That is a **dead governance field (MEDIUM)**.

---

## 11. Tests in the lane — what is actually proven

| Test | Proves | Status |
|---|---|---|
| `hyperagent_governance.rs:219` lower-level `request_detached` creates one attributed remote + device + grant + audit | admission attribution and provisioning | VERIFIED (body read) |
| `hyperagent_governance.rs:281` duplicate `request_detached` refused, no second row | idempotency/conflict honesty | VERIFIED |
| `hyperagent_governance.rs:392` root `spawn_superagent` refusal durable in run bank | gate auditability | VERIFIED |
| `hyperagent_governance.rs:453` foreign-run rewire refused before mutation | hierarchy boundary | VERIFIED |
| `run_lifecycle.rs:883` unknown spawn role refused by name | role gate | CITED-FROM-TEST (name+line) |
| `run_lifecycle.rs:906` strict hierarchy refuses direct hyper→agent / super→subagent | spawn matrix | CITED-FROM-TEST |
| `run_lifecycle.rs:933` hyperagent creates detached remote without a task row | root remote path | CITED-FROM-TEST |
| `run_lifecycle.rs:1178` root checkpoint refuses extra branch when planner supplied one | one-shot topology guard | CITED-FROM-TEST |
| `run_lifecycle.rs:1230` proposal recorded, never auto-applied (4 voters, 4 lenses) | governance honesty | VERIFIED (body read 1229-1298) |
| `run_lifecycle.rs:1301` second control turn cannot create boundaries | turn authority | VERIFIED (body read 1300-1325) |
| `run_lifecycle.rs:1431` plans and dispatches | happy path | CITED-FROM-TEST |
| `run_lifecycle.rs:1498` caps refused by name | cap refusal | CITED-FROM-TEST |
| `run_lifecycle.rs:1536` provider calls counted against the real cap | call accounting | CITED-FROM-TEST |
| `run_lifecycle.rs:1562` provider error preserves partial state | no fake success | CITED-FROM-TEST |
| `run_lifecycle.rs:1600` cancel stops the run | cancellation | CITED-FROM-TEST |
| `run_lifecycle.rs:1649` planned parent chain persisted, not flattened | hierarchy persistence | CITED-FROM-TEST |
| `run_lifecycle.rs:1727` persona allowlist seeded on worker contexts | tool scoping | CITED-FROM-TEST |
| `run_lifecycle.rs:1815` worker spawn restricted to subagents | matrix | CITED-FROM-TEST |
| `run_lifecycle.rs:1864` profile `max_children` tightens fan-out | persona caps | CITED-FROM-TEST |
| `run_lifecycle.rs:1921` streamed deltas before done, not duplicated | streaming honesty | CITED-FROM-TEST |
| `scale_s1.rs:101` 41 tasks at parallel bound 32 | parallel pool | CITED-FROM-TEST (name only) |
| `scale_s0.rs:113` small fan-out probe | scale probe | CITED-FROM-TEST (name only) |
| `snapshot_history.rs:44/73/107/131/180/264` kind/role, migration, corruption, history API, prune | snapshot durability | CITED-FROM-TEST (names only) |
| `kill_restore.rs:48/92` crash restore, corrupt image refused | recovery | CITED-FROM-TEST (names only) |
| `recovery_multi_run.rs:57` two interrupted runs represented, degradation named | multi-run recovery | CITED-FROM-TEST (name only) |
| `graph_authority.rs:19-114` cycle/self-rewire/CAS/in-flight/unknown/toposort/dupes | graph invariants | CITED-FROM-TEST (names only) |
| `recovery_gates.rs:64/70/98` provider-key health checks expose no secret | health checks, **not** recovery | CITED-FROM-TEST (names only) |

**Coverage conclusions.** The lane is well tested for: happy-path planning/dispatch, cap refusals, cancellation, streaming, approval timeout/cancel single-owner paths (in `suspension.rs`, unread here), snapshot/recovery, graph invariants, hierarchy refusals at spawn and rewire. It is **not** tested for: two simultaneous approval waits, parallel tool-budget exhaustion, provider-call exhaustion mid-plan with tool loops, `settle_unfinished_tasks` on a stale Running row, root-final failure semantics, snapshot revision reuse after prune, recovery parent-edge preservation, or any vote-tally path that requires the four voters to actually call `vote_evolution` (the test at `run_lifecycle.rs:1230` asserts *requests*, not votes).

---

## 12. Top-10 bugs / risks (severity ordered)

| # | Severity | Finding | Evidence |
|---|---|---|---|
| 1 | **HIGH** | Concurrent approval waits break the single-slot run machine: second `suspend` from `awaiting_approval` is illegal → approval row expired → run fails `E_SCHEMA_INVALID`. Production runs 32 tasks concurrently. | `run_state.rs:510-512`; `execution.rs:3106-3113`, `3203-3205`; parallel default `http.rs:365-379` |
| 2 | **HIGH** | The HyperAgent is a one-shot controller: one pre-dispatch control turn (2 turns, 4 calls), then an answer-only final call; no mid-run oversight, no re-plan, no reassignment, no kill/retry/veto. | `execution.rs:2056`, `4319-4328`, `2201-2280`; `provider_executor.rs:672-697` |
| 3 | **HIGH** | "Detached" principals are in-process fixtures: `transport_kind = "in_process"`, `endpoint_ref = "fixture"`, fixed device/grant profiles; nothing executes real remote work even though the root/agents are told they spawned a remote. | `execution.rs:5176-5205`; prompt `4299-4307`, `4948-4953` |
| 4 | **MED-HIGH** | Per-run tool-call budget is check-then-act; concurrent tasks can exceed `MAX_CALLS_PER_RUN`. | `execution.rs:2554-2562`, `2607-2619`, `2720-2727`; `tool_loop.rs:16` |
| 5 | **MED-HIGH** | Cap arithmetic: 64 provider calls cannot cover 40 tasks + planner + repair + 2 root-control + final + up to 12 continuations each + escalations; large plans die `E_CALL_LIMIT`. | `caps.rs:14-18`; `planner.rs:10`; `tool_loop.rs:14`; `execution.rs:1908/1924/4333/5469/3736` |
| 6 | **MED** | Root-control model/shape errors abort the run instead of producing a bounded tool-result refusal (asymmetry with leaf tools). | `execution.rs:4341-4344`, `4462`, `4598`, `4730`, `4776` vs `2629-2649` |
| 7 | **MED** | Snapshot revision allocated by row count after `prune_run(16)` can reuse an existing revision and overwrite a file via `INSERT OR REPLACE`; recovery uses `max+1`. Two allocators. | `execution.rs:1732-1738`, `1880`; `snapshot.rs:83-113`, `206-231`; `recovery.rs:480-494` |
| 8 | **MED** | `settle_unfinished_tasks` writes an illegal `running -> queued` edge directly through the store, bypassing the state machine that is documented as the single authority. | `execution.rs:2453-2457` vs `run_state.rs:1-9`, `562-622` |
| 9 | **MED** | Structural rows are settled `Completed` without ever executing or producing a result (single-node plans never run the root at all). Terminal-honest, outcome-dishonest; `task_results` has no row. | `execution.rs:1784-1819`, `1838-1844`, `2206`, `5566-5568` |
| 10 | **MED** | Planner brittleness + prompt-asset bypass: any role outside the fixed alias list fails the run `E_PERSONA_UNKNOWN`; the shipped `planner.prompty` system text is flattened into a user string and never used by the provider, which sends its own `PLANNER_SYSTEM`. | `execution.rs:2002-2003`, `1563`; `execution.rs:2519-2525`; `provider_executor.rs:446` + `26-33`; `planner.prompty:16-24` |

**Also found, below the top 10 (all VERIFIED unless marked):**

- **MED** — `output_schema_id` is written into context and never enforced; a persona's declared output contract is dead governance (`context.rs:31`, `execution.rs:1321`, no reader).
- **MED** — Vote convening has no engine tally or repair: four SuperAgent tasks are created and messaged (`execution.rs:1652-1669`), the request is injected as *untrusted data* (`execution.rs:1365-1369`; `context.rs:74-75`), and nothing verifies that any vote exists before the run terminalises. The 5-seat gate only exists in HTTP reads/writes (`http.rs:2478-2482`, `2598-2600`).
- **MED** — Recovery flattens parents for non-restored runs (`recovery.rs:417-430`, `446-469`); DB and graph hierarchy can disagree while tests only compare states (`recovery.rs:605-614`).
- **LOW-MED** — `awaiting_approval` runs recovered as `interrupted` carry `E_APPROVAL_TIMEOUT`, misattributing cause (`recovery.rs:129-134`).
- **LOW-MED** — `GraphAuthority::set_state` accepts any state; the graph is not a state machine (`graph.rs:236-247`).
- **LOW** — `ROOT_SYSTEM` promises a `spawn_remote_agent` tool the final turn does not advertise (`provider_executor.rs:47-53` vs `682-689`).
- **LOW** — Engine-wide spawn queue / A2A delivery set / metrics / usage maps; terminal cleanup clears them globally (`execution.rs:707-734`, `1896-1901`).
- **LOW** — Module doc claims every mutating command is idempotent by `correlation_id`; only `SUBMIT_PROMPT` is (`execution.rs:11` vs `1514-1516`).
- **LOW** — Duplicate plan deps are misreported as a cycle (`planner.rs:136-165`).
- **LOW** — Cancellation is cooperative only; `cancel_run` returns before any state write (`execution.rs:1676-1689`), so a run blocked in a non-select section (snapshot write, DB) stays `running` until it reaches the next `select!`.
- **LOW** — `write_run_digest` is persisted (`execution.rs:2246`) but I found no reader in this lane; potential dead artifact (**reader UNVERIFIED**).

---

## 13. Explicit assessments

### 13.1 "Truly governing" scorecard

| Governing capability | Present? | Evidence |
|---|---|---|
| Decide the initial topology | Partially — one extra branch lead only when the plan is flat | `execution.rs:4314-4319`, `4383-4393` |
| Amend the plan after dispatch | **No** | no code path; dispatch reads `plan` immutably (`execution.rs:2096-2112`) |
| Observe task progress/state during the run | **No** — root never reads task rows | root tools `execution.rs:4381-4811` |
| Retry / reassign / reorder failed work | **No** | failure fails the run (`execution.rs:2174-2190`) |
| Cancel or kill a branch | **No** — only operator commands (`execution.rs:1672-1726`) |
| Veto or validate a child's output | **No** — child results are concatenated into the final prompt (`execution.rs:2225-2271`) |
| Gate side effects | **No** — approvals are operator-only (`execution.rs:3149-3280`) |
| Manage its own budget | **No** — caps are constants (`caps.rs:8-18`) |
| Durable decision trail | **Yes** — `root_gate:*` rows + remote control audit | `execution.rs:5229-5263` |
| Self-improvement without self-modification | **Yes, proposal-only** | `execution.rs:4558-4716`; `http.rs:2519-2629` |
| Multi-seat evolution governance | **Partly** — operator+4-seat gate enforced in HTTP, not by the engine | `http.rs:2598-2600` |

### 13.2 Settlement honesty

- **Honest:** terminal guards (state machine + lock + SQL), refusal codes on every failure, usage checkpointed after each reply (`execution.rs:1111-1135`) and persisted at terminal (`1137-1151`), snapshot at every committed transition (`1747-1782`), no fabricated replies (`UnconfiguredExecutor` refuses, `execution.rs:551-581`).
- **Not fully honest:** `Completed` is used for structural rows that never ran; the root row's completion is a reconciliation artifact (`execution.rs:1784-1819`); vote-based governance state can read `awaiting_votes` forever without any engine-side consequence; recovered `awaiting_approval` runs carry a timeout error code.

### 13.3 Where hierarchy/task-state violations can occur (ranked)

1. **Approval concurrency** — run-status single slot (§12 #1). Highest practical impact.
2. **Recovery flattening** — graph loses parent edges for superseded runs (§12, recovery).
3. **Bypassed state machine** — `settle_unfinished_tasks` raw `Running→Queued` write (§12 #8).
4. **Unvalidated graph writes** — `GraphAuthority::set_state` (§12).
5. **Fatal root-control shape errors** — not a hierarchy widening, but a governance liveness failure (§12 #6).
6. **Class identity from role strings** — no `TaskRecord.class`; a re-role would silently change authority (`entities.rs:28-45`; `db.rs:341-365`).
7. **Graph-only node rewire** — CAS only, all hierarchy checks skipped when no task row exists (`ws.rs:794-859`).

No path I read lets a task **widen** its own class or depth: `spawn_sub_hive` validates both the tier matrix and the persisted/derived class matrix before insertion (`execution.rs:1563-1576`), and the planner path repeats it (`2002-2019`). The subagent floor is additionally enforced (`execution.rs:1593-1595`, `2020-2022`). **That part of the hierarchy is sound.**

---

## 14. What a true governing HyperAgent still needs

All items are **PROPOSED** (design), not present in the code.

1. **A control loop, not a control turn.** A root that is re-entered on durable events (task started/completed/failed, approval requested, budget thresholds, child result) with an explicit event queue and a bounded actions-per-round contract. Today: one call before dispatch (`execution.rs:2056`).
2. **Authority verbs that exist in the state machine.** `Retry(task)`, `Reassign(task,parent)`, `CancelBranch(task)`, `AmendPlan(add_node, add_edge)` with `run_state` edges, caps, and audit. Today: no such events (`run_state.rs:131-156`).
3. **Read access to execution state.** A bounded `inspect_tasks` / `task_result(task_id)` root tool. Today: root tools cannot read task rows.
4. **Output gating.** A per-child acceptance step (schema and/or explicit verdict) before a result is accepted into the parent's aggregation. Today: `output_schema_id` is never enforced (`context.rs:31`).
5. **Budget leases, not constants.** Per-branch token/call leases derived from caps and returned/unused at branch completion, so a governor can reallocate. Today: constants (`caps.rs:8-18`, `tool_loop.rs:14-21`).
6. **Approval-aware scheduling.** Serialize or multiplex approval waits per run (a wait registry keyed by task), so N branches can await consent concurrently. Today: one run-status slot (`run_state.rs:510-512`).
7. **In-engine vote collection.** After convening, verify seat count and reject/approve outcomes before terminalisation, and expose a single decision object. Today: votes only if voters volunteer; tally only in HTTP reads.
8. **True remote execution plane.** A transport that is not `in_process`/`fixture` (`execution.rs:5180-5182`), with its own lifecycle and result ingestion.
9. **Governor recovery semantics.** On boot, a root that can be resumed with the reconciled task set (and a durable root transcript) instead of only a final synthesis. Today: recovery settles everything `interrupted` and never resumes root control (`recovery.rs:119-137`).
10. **A decision log that the model can read back.** `root_gate:*` rows are written for audit but are not part of the next root turn's prompt except through bounded memory summaries. A governor must see its own prior decisions.

---

## 15. Done / Not done / Unverified (report self-check)

| Done | Evidence |
|---|---|
| Read `execution.rs` end-to-end (5768 lines) and documented every phase with line citations | reads at 1-1388, 1389-2590, 2591-3810, 3811-4926, 4927-5626, 5627-5768 |
| Read planner, graph, caps, context, entities, snapshot, run_state (1-879), recovery (1-930), prompts (1-460) | citations throughout §3-§10 |
| Read root assets (`planner.prompty`, root system prompts) and the boot wiring | `planner.prompty`; `provider_executor.rs:39-53`; `http.rs:337-380` |
| Read governance/wire-relevant code: rewire handler, task-identity trigger, proposals API | `ws.rs:786-942`; `db.rs:296-365`; `http.rs:2360-2629` |
| Verified every material claim by path:line; marked test-only claims CITED-FROM-TEST | tables §1 and §11 |
| Produced top-10 risk list and governing-capability gap list | §12, §13, §14 |

| Not done | Reason |
|---|---|
| Full read of `store.rs` (3525), `db.rs` (1735), `ws.rs` (1619), `http.rs` (3715), `provider_executor.rs` 260-427 & 820-1143, `tool_loop.rs` 121-149 & 319-349 | Out of lane ownership; only governance touchpoints read. Gaps are named in §1 |
| Bodies of `scale_s0/s1`, `snapshot_history`, `recovery_multi_run`, `graph_authority`, `kill_restore`, `recovery_gates`, `prompt_assets*`, and most of `run_lifecycle` | Time budget; coverage recorded honestly as CITED-FROM-TEST (names/lines) |
| `suspension.rs`, `tool_loop_engine.rs`, `tool_loop_matrix.rs`, `proposal_authorization.rs`, `mem_isolation.rs` | Not in the assigned lane list; approval/timeout behaviour is therefore partly evidenced from `execution.rs` only |
| `contract-v1.json`, `manifest.json`, `personas.json` | Wire-schema and asset ownership belongs to other lanes |
| Running any test or build | Task is read-only; no server, no provider, no execution |

| Unverified | What would prove it |
|---|---|
| That two concurrent approval waits actually occur in production plans | A two-branch plan where both branches call an approval-gated tool, with `HIVEROUTER_PARALLEL_TASKS >= 2`; or a new test driving `suspension` twice concurrently |
| That the per-run tool budget is exceeded under parallel dispatch | A test with `parallel_dispatch > 1` and many tool-calling tasks asserting the persisted counter ≤ 40 |
| That `E_CALL_LIMIT` is reachable with a legal 40-node plan | A scale test with 40 tool-calling nodes |
| That a post-prune snapshot commit can collide | A test calling `commit_run_snapshot` after `prune_run`, then reading the overwritten revision |
| Whether `write_run_digest` has a reader | A repo-wide search of `run_digest` consumers (out of lane) |
| Whether `suspension.rs` already covers the concurrent-wait case | Read `backend/crates/hiverouter-core/tests/suspension.rs` (1065 lines) |
| Whether the operator authorize endpoint is auth-gated | Read the router/auth middleware (other lane) |

---

*End of L1 report. Every code claim above resolves to a repo-relative path and line range in this checkout; claims I could not ground are labelled UNVERIFIED or marked as inference in place.*
