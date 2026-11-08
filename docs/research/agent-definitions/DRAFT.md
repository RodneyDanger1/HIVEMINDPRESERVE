# DRAFT - Agent class taxonomy for HiveMind

## VERIFIER-PATCHES

Independent verification pass, 2026-11-08. Snapshot verified: **HEAD `66aff50` plus the worktree
dirty in `execution.rs` and `ws.rs`** (call this **S3**; the tree was still moving during the pass).
`execution.rs` is now **3,430 lines**. The draft's S2 facts were re-checked and mostly hold; the S3
worktree changes are not reflected in the body and are patched below. No runtime, build, test,
server, UI, or provider call was executed. Evidence: `VERIFIED.md`.

- **P1 - S3 snapshot added.** S3 = S2 + two uncommitted changes: `execution.rs` makes
  `validate_spawn_role` `pub(crate)` and adds a **compact persona-id match** (`execution.rs:600-609`,
  +7 lines after line 600); `ws.rs` calls `validate_spawn_role` for template nodes (`:247-250`) and
  writes graph metadata (`:294-299`). **Add +7 to every `execution.rs` line after 609 in the S2
  anchor table**; `persona_for_role` now ends at 609.
- **P2 - F4/C4 ("sub_agent alias escapes narrowing") is REFUTED in S3.** The compact match maps
  `"sub_agent"`, `"sub-agent"`, `"Subagent"` and any `-`/`_`/space variant to the `subagent` profile,
  so the alias no longer receives the 9-tool Worker grant. Patched §3.2, §5.4, §6, §7/F4, §14/C4.
  The remaining critical defect is the **unknown-role fallback** (F3), not the alias. This is
  uncommitted source-read, not a runtime test.
- **P3 - F8/C7 ("TEMPLATE_INSERT can create a depth-1 HyperAgent") is REFUTED in S3.** Template
  inserts now call `validate_spawn_role(root_tier, role)` (`ws.rs:247-250`) and set metadata
  (`:294-299`), so a depth-1 `"planner"` insert is refused. Patched §3.5, §5.1, §7/F8, §14/C7.
  **New defect found:** the template node role is still the **raw template id**, so the four shipped
  Library templates (`content-writer`, `data-analyst`, `code-reviewer`, `chatbot`) now fail
  `validate_spawn_role(HyperAgent, ...)` with `E_TOOL_NOT_ALLOWED`; the in-file tests
  `template_insert_spawns_manifest_backed_dag` and
  `template_insert_resolves_the_library_ids_after_a_real_lookup` would fail by construction. M7 now
  says to map template id -> spawn role/profile before admission, not to validate the raw id.
- **P4 - F6 self-contradiction.** F6 said "the plan loop calls only `caps.check_spawn`" and cited
  `execution.rs:1509`; that line is exactly the `validate_spawn_role(parent_tier, &node.role)?` call.
  The plan loop validates the role (since `aa15ec9`); the depth-floor gap remains. Patched F6.
- **P5 - Remaining anchor corrections.** `http.rs:1419` -> 1412 (`max_calls`); `:950`/`:1009` ->
  943/1002 (`persona.tier`); `:737-744`/`:1031-1038` -> 740-743/1034-1040 (history/roster kind);
  `:772-789` -> 765-781 (labels, which also include `ToolAgent`); `execution.rs:1428`/`:3075` ->
  1435/3129 (`check_calls(1)`); `event.rs:915-922` -> 915-924 (four-kind test).
- **P6 - §5.1 metric error.** "RUN_METRICS counts hyperagents from task kinds" is false:
  `RunMetrics` carries only `superagents`/`workers` (`event.rs:274-284`, `execution.rs:708-747`).
  Hyperagent counting is web-local (`SwarmPanel.tsx:84`). Patched.
- **P7 - Run bank, not run digest.** Since `aa15ec9` the engine writes `engine`/`prompt` and
  `engine`/`plan` rows into the `"run"` scope before seeding (`execution.rs:1462-1485`), and tasks
  re-read the run + parent banks at start (`:3005-3033`). The draft's "no run-digest writer" is true
  only for a *result* digest. Patched §3.6 and §5.1.
- **P8 - Doc-12 attribution.** `12:484-487` recorded the pre-`bb95a44` code ("empty means no
  narrowing") **and prescribed the reversal**; the current code matches the prescription. The doc's
  code description is stale; its model is not. Patched §3.3.
- **P9 - F14 misattribution.** `21:840-985` is a **design proposal** for ownership, not a claim that
  it exists; the same doc says ownership is absent at `:347`. Patched F14.
- **P10 - §7 claim framing.** F1 attacks an **operator requirement** as if it were a runtime claim.
  Patched to say the runtime rank is absent while the requirement is real. External C13-C16 were not
  re-fetched in this pass (A2A was re-checked and holds); they are not load-bearing for the verdict.
- **P11 - ToolActor is dormant, not engine-created.** §4 said the ToolActor is "engine-created for
  approved tool work"; no code path creates a task with tier `ToolAgent`. `task_agent_tier` maps
  `"tool"`, but the plan gate, `SPAWN_SUB_HIVE`, `spawn_agent`, and `TEMPLATE_INSERT` all reject it
  (`execution.rs:180`, `:157`; `ws.rs:247-250`), and approved tool work executes under the calling
  task's tier. Patched §4. It is a tier-with-no-producer, not an agent class.

**Status:** DRAFT research note. Not a specification, not an implementation plan approved by the
Lead, and not evidence that anything works.
**Repository:** `C:\Users\Riley\Desktop\hiverouter-v1`. The pass began at HEAD `bb95a44`; the tree advanced to HEAD `66aff50` ("fix: readiness DAG, run memory recall, CLI truth, kill confirm") while the pass ran, and the verifier read a third snapshot (S3) with the worktree dirty. All three snapshots are recorded below.
**Scope of this pass:** read-only inspection of source and docs, plus external reading. **No**
runtime, server, UI, provider call, build, migration, or test was executed. The only writes are this
file and `VERIFIED.md`.

## 0. Claim tags used everywhere below

| Tag | Meaning in this document |
|---|---|
| **[SR]** | **Source-read.** The cited line(s) were read in this pass. Line numbers are from the final read of that file. |
| **[GV]** | **Grep-verified.** An occurrence or absence observed in tool output over the stated path set. This is evidence about text, not about runtime behavior. |
| **[DR]** | **Doc-read.** The claim exists in a repository document. It may be stale, self-contradictory, or unsupported by code. |
| **[WR]** | **Web-read.** An official page was fetched in this pass; the fact is quoted/paraphrased from it. |
| **VERIFIED** | **Reserved and not used in this file for product behavior.** Nothing was executed. There is no runtime verification here, and the task forbids obtaining one. |
| **[SR S1] / [SR S2] / [SR S3]** | A source-read qualified by snapshot (section 0.1): S1 = HEAD `bb95a44` plus the initial dirty set; S2 = HEAD `66aff50`; S3 = S2 plus the uncommitted `execution.rs`/`ws.rs` changes. |

**Moving-tree caveat (important).** `execution.rs`, `http.rs`, `provider_executor.rs`, and web files
were being edited by another writer while this pass ran. Three snapshots matter:

| Snapshot | Meaning |
|---|---|
| **S1** | HEAD `bb95a44` plus the small initial dirty set. This is what most repo docs (2026-11-07/08) describe. |
| **S2** | HEAD `66aff50` (committed mid-pass): `execution.rs` 3,423 lines, a `persona_for_role` normalizer, a tier-based `node_kind` in `http.rs`, a **readiness-based dispatcher**, and a wider root-turn gate (`has_aggregators || plan.nodes.len() > 1`). **Unbuilt and untested in this pass.** |
| **S3** | S2 plus the **uncommitted worktree** in `execution.rs` and `ws.rs`: `validate_spawn_role` is `pub(crate)`, `persona_for_role` gains a compact id match, and `insert_template_dag` validates the spawn role and sets graph metadata. `execution.rs` is 3,430 lines. **Also unbuilt and untested.** |

In S3, add **+7** to every `execution.rs` line after 609 (the `persona_for_role` compact-match block);
`persona_for_role` itself starts at 588 and ends at 609.

Inline line numbers below are from the read that produced the adjacent sentence; they drift by up
to ~60 lines between S1 and S2. **Where an inline number and the table below disagree, the table
wins.** Re-grep the symbol before editing; do not trust a number in a moving tree.

**S2 anchor table (authoritative for `backend/crates/hiverouter-core/src/execution.rs`):**

| Symbol | Line (S2) | Symbol | Line (S2) |
|---|---:|---|---:|
| `node_kind` | 121 | `seed_task_context` | 911 |
| `validate_spawn_role` | 135 | `authorize_tool_call` | 1084 |
| `task_agent_tier` | 173 | `spawn_sub_hive` | 1160 |
| `persona_for_role` | 588 | `check_calls(1)` (dispatch) | 1428 |
| `publish_node` | 794 | root task row | 1437 |
| `publish_node_spawn` | 816 | `has_aggregators` | 1668 |
| root gate (S2) | 1679 | root memory read (32) | 1698 |
| `execute_tool_turn` | 1840 | `check_call_budget` | 1904 |
| `execute_ask_parent` | 2681 | `execute_send_to_sibling` | 2773 |
| `execute_spawn_agent` | 2882 | `refresh_memory_bank` | 2998 |
| `execute_task_with_prompt` | 3072 | `execute_task_inner` | 3113 |
| `complete_task` | 3336 | completion memory write | 3381-3392 |

The mid-pass commit is **[SR] only** - a commit is not a verified fix, and this pass did not build
or run it.

---

## 1. The one-page answer

Five words are used in this repo for things that are not the same kind of thing. The taxonomy below
sorts them by **accountability and lifetime**, not by model size or prompt wording.

| Name | One-line definition | Rank | Unit of work? | Wire kind today | Actually coded? |
|---|---|---|---|---|---|
| **HyperAgent** | The one run-scoped accountable orchestrator at depth 0 that owns the plan, the run answer, and the final verdict. | 1 | root task row (`role:"planner"`, depth 0) | `hyperagent` | Yes; root turn only for multi-node plans (S2) or aggregating plans (S1) |
| **SuperAgent** | A branch/hive-scoped sub-orchestrator that decomposes one assigned branch into Agents and Subagents and reports one branch result. | 2 | task row at depth 1 | `superagent` | Yes as tier+persona; ownership scope is not modeled |
| **Agent** (regular) | A leaf executor that does one assigned task with tools and returns one bounded result. | 3 | task row at depth 1-2 | `worker` | Yes, but has no built-in profile |
| **Subagent** | A narrowed leaf under an Agent/Super: no spawn, no write, no exec; returns a precise result or a precise uncertainty. | 4 | task row at depth 2-3 (design) | **none** (renders `worker`) | Persona yes; class/kind/depth floor no |
| **Remote tiny agent** | A tree-free capability principal outside any run that owns devices (e.g. one light switch) and reads universal/global memory read-only plus its own bank. | n/a (not a rank) | **no task row** | **none** | **No. Design only.** |

**The one scientific sentence.** In HiveMind, *rank* is accountability and orchestration scope;
*authority* is the intersection of tier allowlist and profile grant; *identity* is the pair
(task row, profile); and *lifetime* is the only reliable separator between a Remote principal (outlives
runs, no task) and every other class (born and dies inside a run). Today the code models only rank-1
to rank-3 by parsing a string, gives Subagent a profile but no class, and does not model Remote at
all.

**The one coding sentence.** Today one string (`tasks.role`) is doing four jobs at once - display
label, rank selector, capability selector, and memory key - and no persisted field records which
agent class a task actually is. That is the root defect; the individual bugs below are symptoms.

---

## 2. Six axes that must not be conflated

The task asks to distinguish **rank vs role vs profile vs principal vs process vs record**. They
differ in who writes them, what they are allowed to authorize, and how long they live.

| Axis | What it is | Written by | Lifetime | May it grant authority? | In HiveMind today |
|---|---|---|---|---|---|
| **Rank** | Ordinal accountability class in the hierarchy: Hyper (1), Super (2), Agent (3), Subagent (4). Comparable, sortable, and used for escalation rules. | The engine, from the resolved profile/class. | The instance. | Indirectly: rank constrains who may spawn whom; it must not be the whole grant. | **Not a field.** Derived by `task_agent_tier` from `tasks.role` **[SR]** `execution.rs:173-185`; no `rank` column. |
| **Role** | The declared job name a caller asks for (`"planner"`, `"superagent"`, `"subagent"`, `"worker"`, a persona alias, or arbitrary text). | Caller: planner model, WS command, template insert, test. | Spawn request. | **No.** It is untrusted input. | **The only identity source.** Same string is label, tier selector, persona key, memory key **[SR]** `execution.rs:173-185`, `:906-911`, `:3335-3343`. |
| **Profile** | Versioned declarative record: id, display name, tier, template, allowed tools, output schema, refusal policy, spawn limits. Data, not behavior. | Operator / shipped asset; human edit. | Across runs; versioned. | **Yes**, as policy, intersected with the tier. | `Persona` struct **[SR]** `prompts.rs:383-404`; 7 shipped profiles **[SR]** `assets/prompts/personas.json:1-67`; **no version field, no class field, and `tier` is never enforced**. |
| **Principal** | The accountable party an action is attributed to: the operator, a run owner, or (design) a remote agent. | The system that creates the action. | Longer than any one task. | **Yes** - it is the root of authority. | Only the operator (human) and task owners. No non-task principal exists **[GV]** zero `remote_*`/`device*`/`daemon` symbols under `backend/crates/*/src`. |
| **Process** | An OS/runtime execution instance: thread, async task, actor, child process. | OS/runtime. | Milliseconds to server lifetime. | No. | **Not modeled per agent.** One server process; one active run; serial dispatch inside a run **[SR]** `execution.rs:1132`, `:1550-1753`. "Agent" is a DB row, not a process. |
| **Record** | A persisted row or event: task, message, memory entry, approval, graph node, card projection. | Engine writers. | Until deleted / run retention. | **No.** Records are evidence. | `tasks`, `agent_memory`, `agent_messages`, `approvals`, graph nodes, snapshots **[SR]** `db.rs:181-192`, `:251-272`. A card is a projection of a record, never an authority object. |

### Rules that follow from the axes

- **A1 - Rank is not privilege.** The highest rank has the *narrowest* tool allowlist. HyperAgent:
  `spawn_agent`, `report_result`, `finish` (3). WorkerAgent: all 9 tools including `file_write` and
  `shell_exec` **[SR]** `tool.rs:392-421`. Rank buys accountability and delegation scope, not
  capability. Any UI or doc implying "HyperAgent can do everything" is false.
- **A2 - A role string must never authorize.** `task_agent_tier` maps unknown strings to
  `WorkerAgent` **[SR]** `execution.rs:183`, which is the broadest tool set. Today an unknown or
  mistyped role is therefore promoted to the most capable tier instead of being refused.
- **A3 - A profile is policy, not identity.** Two instances with the same profile can have different
  rank if rank is derived from ownership (a branch lead vs a department owner). The profile must not
  be used as the primary key of identity.
- **A4 - A principal can exist without a task; an agent instance cannot.** This is the whole reason
  a Remote tiny agent cannot be modeled as a task row: no `parent_task_id`, no depth, no run id, no
  `MAX_AGENTS` consumption. **[DR]** `docs/research/remote-agents/01-remote-agent-model.md:24-43`
  states this; the code has no such entity **[GV]**.
- **A5 - A process is an implementation detail.** Restarting a run creates new task rows, not a new
  "agent identity"; a killed process does not resurrect a terminal task **[SR]** `execution.rs:93-95`,
  `:3289-3298`.
- **A6 - A record is not a card, and a card is not authority.** The wire `kind` is a projection of
  the task tier **[SR]** `execution.rs:121-128`; the web card is a renderer of that kind
  **[SR]** `web/src/flow/nodes.tsx:302-310`. No card click may widen a grant **[DR]**
  `docs/research/hmars/12-role-capability-model.md:220-221`.

---

## 3. What the current tree actually does (mechanical truth)

This section is the shared factual base for the five definitions. All claims are tagged.

### 3.1 One enum, four tiers, no Subagent

- `AgentTier = {HyperAgent, SuperAgent, WorkerAgent, ToolAgent}` **[SR]**
  `backend/crates/hiverouter-tools/src/tool.rs:72-81`; labels are `hyper_agent`, `super_agent`,
  `worker_agent`, `tool_agent` **[SR]** `:88-92`.
- There is no `Subagent` variant, no `AgentClass` type, no `rank` field anywhere in `backend/`
  **[GV]** symbol search over `backend/crates/*/src`.
- Wire kinds are a pure function of tier: Hyper->`hyperagent`, Super->`superagent`,
  Worker->`worker`, Tool->`tool` **[SR]** `execution.rs:121-128`; contract vocabulary is exactly
  those four **[SR]** `backend/crates/hiverouter-contract/src/event.rs:240`, with a test that pins
  four values and asserts `error` is client-only **[SR]** `event.rs:915-924`.

### 3.2 Rank/class is inferred from a free-form role string

- `task_agent_tier` lowercases `task.role` and maps known aliases; **everything else becomes
  `WorkerAgent`** **[SR]** `execution.rs:173-185`. `"subagent"` has no tier arm of its own (it is
  accepted by the spawn matrix, not mapped to a tier), so it takes the `_` fallback to
  `WorkerAgent` **[SR]** `:183`.
- Empty role at depth 0 becomes HyperAgent **[SR]** `:182`; an empty role at depth > 0 becomes
  WorkerAgent.
- The persona lookup used to be exact-match on `task.role`. S2 normalized it to exact -> lowercase ->
  `-`/space-to-`_`; **S3 adds a compact id match** (lowercase, strip `-`/`_`/space) at
  `execution.rs:588-609`. Before that change, UI roles `"SuperAgent"` / `"Subagent"` missed their profiles
  **[DR]** `docs/research/hmars/21-roles-daemons-implementation.md:288-300`. **In S3 the
  `"sub_agent"` alias resolves to the `subagent` profile**; see the §5.4 correction.

### 3.3 Authority = tier allowlist intersected with the profile allowlist (when matched)

- Model-visible tool list: tier only **[SR]** `provider_executor.rs:335-351` via
  `tool_loop::tool_specs_for_tier` **[SR]** `tool_loop.rs:206-219`.
- Call-time gate: tier validation first, then, if the task's scoped context came from a matched
  persona, the tool must also be on that persona's list **[SR]** `execution.rs:1091-1105`;
  `persona_tool_scope=true` is set at context seed **[SR]** `:929-933`.
- An empty persona allowlist therefore means **no tools** for a matched profile (the planner
  profile is `[]`) **[SR]** `execution.rs:1091-1095`, `personas.json:7`. `12:484-487` recorded the
  pre-`bb95a44` code as "empty means no narrowing" **and prescribed the reversal** ("an empty grant
  means no tools"); the current code matches the prescription, so the doc's *code description* is
  stale, not its model. **[DR]** `docs/research/hmars/12-role-capability-model.md:484-487`.
- Consequence: a Subagent is *told* it has `spawn_agent`, `file_write`, and `shell_exec` (tier
  advertisement) but refused at call time (persona narrowing). Advertising is honest-ish for the
  tier but dishonest about the effective grant; the capability literature calls this a
  confused-deputy invitation, not a safety property.

### 3.4 Spawn matrix (the hierarchy policy actually enforced)

`validate_spawn_role(parent_tier, requested_role)` **[SR]** `execution.rs:135-164`:

| Parent tier | Accepted role strings (exact, lowercased) | Refusal |
|---|---|---|
| HyperAgent | `super`, `superagent`, `super_agent`, `sub_hive`, `subhive`, `worker`, `workeragent`, `worker_agent`, `agent`, `subagent`, `sub_agent` | `E_TOOL_NOT_ALLOWED` |
| SuperAgent | `worker`, `workeragent`, `worker_agent`, `agent`, `subagent`, `sub_agent` | `E_TOOL_NOT_ALLOWED` |
| WorkerAgent | `subagent`, `sub_agent` only | `E_TOOL_NOT_ALLOWED` |
| ToolAgent | nothing | `E_TOOL_NOT_ALLOWED` |

Read carefully:

- **There is no depth floor for Subagent.** HyperAgent accepts `"subagent"` at depth 1, so the root
  can create a Subagent directly at depth 1 **[SR]** `:138-151`. The design docs say Subagent is
  legal only at depth >= 2 **[DR]** `12-role-capability-model.md:170-172`,
  `21-roles-daemons-implementation.md:557-560`; the code does not enforce it.
- **No `E_SPAWN_PROFILE`, `E_SPAWN_PROFILE_DEPTH`, `E_PROFILE_VERSION`, or `E_CAPABILITY_EMPTY`
  exists.** The refusal enum has only `E_SPAWN_DEPTH`, `E_FANOUT_LIMIT`, `E_AGENT_LIMIT`,
  `E_PERSONA_UNKNOWN`, `E_TOOL_NOT_ALLOWED` **[SR]** `backend/crates/hiverouter-contract/src/refusal.rs:119-125`,
  `:212-214`, `:249`; the design docs' refusal table is aspirational. **[DR]**
  `12-role-capability-model.md:729-750`.
- A refusal here is `E_TOOL_NOT_ALLOWED`, which is semantically wrong (it is a *spawn policy*
  refusal, not a tool-allowlist refusal) and unhelpful to an operator.

### 3.5 Spawn paths and their admission

| Path | Admission | Gaps |
|---|---|---|
| Planning (`dispatch_inner`) | `plan.validate()` + parent tier resolution + `validate_spawn_role` + caps per node **[SR]** `execution.rs:1487-1529` | Roles are model-authored; no profile resolution; a planned `"subagent"` at depth 1 is accepted; no dedupe. |
| Operator `SPAWN_SUB_HIVE` | `validate_spawn_role` + per-profile `max_children`/`max_depth` + caps under one lock **[SR]** `execution.rs:1167-1224` | Depth floor absent; UI sends Title-Case roles **[SR]** `web/src/state/store.ts:1976-1982`; the S3 persona lookup now resolves them, but the row stores the role verbatim. |
| Model `spawn_agent` tool | Reuses `spawn_sub_hive`; parent claim checked; child executed inline and its bounded result returned **[SR]** `execution.rs:2889-3000` | Same depth/profile gaps; child runs inside the parent's turn (blocking). |
| `TEMPLATE_INSERT` | Template lookup + `validate_spawn_role(root_tier, node.role)` + caps; graph metadata written **[SR]** `ws.rs:240-301` (S3, uncommitted) | **S1/S2 bypassed `validate_spawn_role` and could create a depth-1 `"planner"` HyperAgent; S3 closes that** and sets metadata. Remaining: the template id is still used as the role, so the four shipped Library templates (`content-writer`, `data-analyst`, `code-reviewer`, `chatbot`) are now refused by the Hyper root's spawn allowlist (`E_TOOL_NOT_ALLOWED`), and a `"subagent"` template still inserts at depth 1. No profile resolution. |

### 3.6 Context and memory: scopes, not banks

- `ScopedContext` = parent id, facts map, allowed tool ids, persona scope flag, output schema
  **[SR]** `context.rs:19-32`; facts are keyed `parent:<task id>:<key>` **[SR]** `:114-116`.
- Inheritance at seed: the child copies the parent context's facts except `a2a:` and `roster:`
  keys **[SR]** `execution.rs:936-944`; then it reads up to 8 memory rows from the parent's scope
  and up to 8 from the `"run"` scope, across all roles **[SR]** `:934-952`; then a roster of up to 16
  peers and up to 8 inbox messages **[SR]** `:956-990`.
- The engine seeds the `"run"` scope itself: `engine`/`prompt` (the operator prompt, 2,000 chars) and
  `engine`/`plan` (a role:depth digest) are written before the root/child contexts are seeded
  **[SR]** `execution.rs:1462-1485`. So a run bank exists, but it holds only those engine rows plus
  each task's own completion row; there is no result digest.
- At task start, memory and inbox are re-read (so messages that arrived after seeding are visible)
  **[SR]** `execution.rs:3005-3062`.
- On completion, the task's final text (2,000 chars) is written with `scope_id = its own task id`,
  `agent_key = task.role` **[SR]** `execution.rs:3388-3399`. So each task owns one bank named after
  itself, and a child sees its immediate parent's bank plus the run bank.
- The root synthesis reads **up to 32 memory rows across the whole run** and labels them "Shared
  hive/branch memory" **[SR]** `execution.rs:1705-1725`. This is a real aggregate read, contrary to
  the older truth doc **[DR]** `docs/provider/AGENT-ARCHITECTURE-TRUTH.md:28-31`.
- There is **no global bank, no project bank, no hive bank, no role bank, no trust/retention, no
  vector store** **[SR]** `db.rs:251-272`; **[GV]** no `memory_banks` symbol. The operator's
  "universal memory" has no implementation **[DR]** `OPERATOR-DECISIONS.md:61-75`.

### 3.7 Communication is a write-only mailbox

- `ask_parent` validates a non-empty bounded text, checks a per-run message budget (32), inserts a
  row and publishes an event; it returns a message id **[SR]** `execution.rs:2688-2757`. **There is
  no answer path, no wake, no deadline, and no delivery status.**
- `send_to_sibling` is same-parent by default; cross-branch sends require `allow_cross_parent`
  **[SR]** `execution.rs:2780-2802+`.
- Recipient inbox facts are capped at 8 messages x 500 chars **[SR]** `execution.rs:988-997`;
  refreshed only at task start, not before continuation turns.

### 3.8 Lifecycle

- `AgentState` has 16 values including client-only `Reconnecting`, `Replay`, `Empty`, `Loading`
  **[SR]** `backend/crates/hiverouter-contract/src/state.rs:18-50`. Terminal task states are
  `Completed`, `Failed`, `Cancelled`, `Killed`, `Interrupted` **[SR]** `execution.rs:114-117`;
  `web/src/state/agentState.ts:111-119` mirrors the same five.
- Completion is terminal-aware: a task no longer `Running` is never overwritten **[SR]**
  `execution.rs:3343-3362`. The run reaches one terminal status exactly once **[SR]**
  `execution.rs:1379-1420`; leftover tasks are reconciled to the run's terminal status **[SR]**
  `:1362-1366`.
- **Failure is fail-fast for the whole run.** Any leaf/aggregator refusal that is not a
  kill/cancel of that task aborts `dispatch_inner` and fails the run **[SR]**
  `execution.rs:1656-1679`. There is no retry, no supervisor, no per-branch failure isolation, and
  no "HyperAgent decides retry" logic, despite the design **[DR]**
  `OPERATOR-DECISIONS.md:103-104`; `12-role-capability-model.md:1130-1190`.
- One active run per server **[SR]** `execution.rs:1132`; inside a run, S1 executes leaves in
  topological order, then aggregators deepest-first, then the post-plan spawn queue, then the root
  **[SR]** `execution.rs:1519-1705`. **S2 replaces that with a readiness loop**: a node runs only
  after every dependency and every direct child has a result, then the post-plan queue and root
  follow **[SR]** `execution.rs:1550-1753` (S2, committed `66aff50`, unbuilt here). Either way execution is
  **serial**: one model call at a time, no parallel branch execution.
- Caps: depth 3, fan-out 8, 64 agents/run, 40 calls/run **[SR]** `caps.rs:9-15`. **The 40-call cap
  is not a live counter**: both call sites pass the literal `1` **[SR]** `execution.rs:1435`,
  `:3129`, so `check_calls` can never fail; the real bounds are the tool-loop call budget (40)
  **[SR]** `tool_loop.rs:16`, `execution.rs:1911` and the provider `RunBudget`.

### 3.9 Observability / UI

- Server frames carry `kind` derived from tier **[SR]** `execution.rs:121-128`, `:757-808`;
  history/roster now also use tier (S2 fix, committed `66aff50`) **[SR]** `http.rs:740-743`, `:1034-1040`;
  labels are `HyperAgent <suffix>` / `SuperAgent <suffix>` / `ToolAgent <suffix>` /
  `<role> <suffix>` / `Agent <suffix>` **[SR]** `http.rs:765-781`.
- Web knows **six** kinds: `hyperagent`, `superagent`, `subagent`, `worker`, `tool`, `error`
  **[SR]** `web/src/api/contract.ts:213`, `web/src/state/agentState.ts:126-136`; six renderers
  **[SR]** `web/src/flow/nodes.tsx:141-310`.
- The backend can emit only four of those six. After the S2 `http.rs` fix, **no server path
  ever emits `subagent`**; `SubagentNode` is reachable only if a frame carries a kind the server
  cannot produce **[SR]** `execution.rs:121-128`; `http.rs:740-743`; `event.rs:240`. Either the card
  is dead, or the taxonomy is missing its wire representation.
- The root is filtered off the canvas and represented by a fixed chat anchor
  **[SR]** `web/src/flow/graph.ts:122`, `:235`; the tree/graph and the root card are different
  projections.
- Unknown or absent kinds are **coerced to `worker`** on hydration and replay
  **[SR]** `web/src/state/store.ts:652`; `web/src/flow/graph.ts:305`. This is a lie-by-default: a
  missing class is rendered as the most common class instead of "not reported".

---

## 4. Taxonomy diagram (text)

```
                         AUTHORITY (who is accountable)
                         --------------------------------
 operator (human, not an agent, no card; sole root of authority)
   |
   | owns runs, approves side effects, edits profiles, creates remotes/devices
   |
   +-- RUN (one active run per server, lifetime = run)
   |     |
   |     +-- [1] HyperAgent   role "planner", depth 0, exactly one
   |     |      rank 1 | class HyperAgent | tier HyperAgent | profile planner
   |     |      no parent | no workspace tools | plan gate + final answer
   |     |      turns: the final synthesis for any multi-node plan (S2/S3; S1: aggregating plans only)
   |     |
   |     +-- [2] SuperAgent  depth 1 (branch/hive owner in the design)
   |     |      rank 2 | tier SuperAgent | profile superagent
   |     |      spawns Agents + Subagents | read-only tools | no write/exec
   |     |      |
   |     |      +-- [3] Agent (regular)  depth 1-2
   |     |      |      rank 3 | tier WorkerAgent | profile <persona> (none shipped as "agent")
   |     |      |      spawns Subagents only | all 9 tools, write/exec approval-gated
   |     |      |      |
   |     |      |      +-- [4] Subagent  depth 2-3 (DESIGN; not enforced)
   |     |      |             rank 4 | tier WorkerAgent | profile subagent
   |     |      |             no spawn, no write, no exec; read/report/finish only
   |     |      |
   |     |      +-- [4] Subagent (also legal directly under a Super)
   |     |
   |     +-- [4] Subagent under a Super
   |     +-- ToolActor (tier ToolAgent; not a rank; no runtime producer — no public spawn
   |                  source creates a task with this tier, so it is a dormant allowlist)
   |
   +-- OUTSIDE ANY RUN (lifetime > run; no task row, no depth, no parent)
         |
         +-- [R] Remote tiny agent  (DESIGN ONLY, zero code)
         |      principal, tree-free | reads global/system bank RO + own bank
         |      owns devices: class light/switch/camera/cover/lock/sensor
         |      one basic function (e.g. light switch) + health heartbeat
         |      cannot spawn, cannot create runs, cannot read task banks
         |
         +-- Daemon / Watchdog  (DESIGN ONLY, zero code)
                record, not a principal: one immutable job + switch + cadence + reports
                no model, no tools, no memory bank, no tree depth

  Cross-cutting, not a class:
    Profile  = versioned policy record, grants capability, not identity
    Card     = UI projection of a record, no authority
    Kind     = wire projection of class/tier, currently only 4 values
    Process  = OS/async execution instance; not modeled per agent

  DANGEROUS SHORTCUTS THE DIAGRAM FORBIDS
    role string  -/-> authority
    card click   -/-> grant
    unknown role -/-> WorkerAgent with 9 tools   (today's behavior; must become a refusal)
    remote       -/-> task row / depth / NODE_*  (today: absent; must stay absent)
```

Legend: `[1]..[4]` are ranks. `[R]` is a principal class, not a rank. A Subagent is *not* a fifth
tier; it is rank 4 on the Worker tier with a narrowed profile. A Remote is not a rank, a tier, a
daemon, or a task.

---

## 5. The five classes, defined operationally

Format per class: purpose, authority, lifecycle/lifetime, memory (read/write), communication,
spawn rights, tools/allowlist, failure/supervision, observability/UI, invariants, code mapping,
and **what it must not be**. Each class has a **Current** and a **Target** statement; the target is
the smallest honest model that matches the operator's four ranks plus the remote principal.

### 5.1 HyperAgent

**Purpose.** Own one run. Convert one operator prompt into a validated plan DAG, decide the run's
final answer from child results, and be the single accountable party for the run's outcome. In the
target model it also decides replan/abort; today the engine decides failure.

**Authority.**
- *Current:* The engine, not the model, owns run termination. The root row is created `Running`
  before any planned node **[SR]** `execution.rs:1444-1453`; the root's only model turn is
  answer-only and tool calls there **fail the task** **[SR]** `provider_executor.rs:364-389`;
  `execution.rs:3177-3182`. Its tier allowlist (spawn/report/finish) is therefore never exercised by
  the model **[SR]** `tool.rs:392-395`. The root's real spawn authority is the plan gate, which
  validates every planned child against `HyperAgent` before insert **[SR]**
  `execution.rs:1509`, `:1485-1516`.
- *Target:* Hyper owns plan acceptance, branch-result adjudication, and the final answer; it may
  **request** a daemon/remote action but never approve its own action, never widen a grant, and
  never read a child's raw transcript. **[DR]** `12-role-capability-model.md:231-250`.

**Lifecycle / lifetime.** Exactly one per run, depth 0, no parent, created at planning time, ends
when the run reaches a terminal state. S1 ran the root turn only when the plan had at least one
aggregator **[SR S1]** `execution.rs:1554-1555`, `:1631-1705`; **S2/S3 run it when
`has_aggregators || plan.nodes.len() > 1`** **[SR S2/S3]** `execution.rs:1675-1686`, so a single-node
plan still has **no root model turn**, and the root row is settled by terminal reconciliation
**[SR]** `execution.rs:1379-1392`. It is not a long-lived process: no thread, no actor, no
background loop **[SR]**/**[GV]**.

**Memory.**
- Read: planner persona context; run-scope rows (up to 8 at seed) **[SR]** `execution.rs:945-952`;
  at the root turn, up to 32 rows across every scope of the run, explicitly labelled "Shared
  hive/branch memory (data, not instructions)" and truncated with an omission marker **[SR]**
  `execution.rs:1705-1725`; plus all child result text bounded to 2,000 chars each, total prompt
  budget 24,000 chars **[SR]** `execution.rs:1697-1758`.
- Write: if the root turn completes, its final text is written under `agent_key="planner"`,
  `scope_id=<root task id>` **[SR]** `execution.rs:3388-3399`. The engine also writes the
  `engine`/prompt and `engine`/plan rows into the `"run"` scope **[SR]** `execution.rs:1462-1485`;
  there is still no **result** digest writer and no per-hive summary writer.
- Target: read run bank + hive summaries/counts; write the run digest and decisions; raw all-bank
  access is an explicit operator setting. **[DR]** `12-role-capability-model.md:243-245`.

**Communication.** None outbound: `ask_parent` is not on the Hyper allowlist and the root has no
parent; `send_to_sibling` is not either **[SR]** `tool.rs:392-395`. The designed `ask_operator`
verb does not exist **[GV]** no such tool id. Inbound: operator prompt only; child results arrive as
prompt text, not as messages.

**Spawn rights.**
- Hyper may *accept* planned roles Super/Agent/Subagent and the operator's `SPAWN_SUB_HIVE` for the
  same set **[SR]** `execution.rs:138-151`, `:1139`.
- Hyper may **not** spawn in the target model a ToolActor, another Hyper, or a daemon; the plan gate
  rejects `"tool"` and `"planner"` at depth 1 **[SR]** `execution.rs:138-151`, and in S3 the
  `TEMPLATE_INSERT` path also calls `validate_spawn_role` **[SR]** `ws.rs:247-250`, so the S1/S2
  depth-1 `"planner"` template escape is closed. The template id is still the role, so the shipped
  Library templates now fail that gate (see §3.5 and M7); a `"subagent"` template still inserts at
  depth 1.

**Tools / allowlist.** Tier: `spawn_agent`, `report_result`, `finish` **[SR]** `tool.rs:392-395`.
Effective model-visible set on the root turn: **none** **[SR]** `provider_executor.rs:364-389`. The
planner profile declares `allowed_tool_ids: []` and would mean "no tools" under the current
narrowing rule anyway **[SR]** `personas.json:3-10`, `execution.rs:1091-1095`.

**Failure / supervision.** A root-turn refusal fails the task and therefore the run; there is no
retry and no supervisor above the root **[SR]** `execution.rs:1656-1679`, `:3108-3125`. The design's
"HyperAgent decides retry" is not implemented **[DR]** `OPERATOR-DECISIONS.md:103-104`.

**Observability / UI.** Wire kind `hyperagent` **[SR]** `execution.rs:101`; label
`HyperAgent <last 8 of id>` **[SR]** `http.rs:776`; the canvas filters it out and uses a fixed hive
anchor plus the chat core **[SR]** `web/src/flow/graph.ts:122`, `:235`; `HyperAgentNode` exists but
is not placed by `buildNodes` **[SR]** `nodes.tsx:270-292`, `graph.ts:235`. `RUN_METRICS` carries
only `superagents`/`workers` (`event.rs:274-284`, `execution.rs:708-747`); hyperagent counting is
web-local in the swarm census **[SR]** `SwarmPanel.tsx:84`.

**Invariants.**
1. Exactly one Hyper per run, depth 0, no parent.
2. Answer-only final turn; no tool call may change the graph after it starts.
3. No workspace read/write/exec.
4. Cannot be spawned by any agent, plan, or template.
5. Cannot approve its own action or widen its own grant.
6. Must not be the UI chat persona; chat is an operator surface **[DR]**
   `docs/provider/AGENT-ARCHITECTURE-TRUTH.md:24`.

**Code mapping.** `execution.rs:121-128` (kind), `:135-164` (spawn gate), `:173-185` (tier),
`:588-609` (persona resolution, S3), `:1444-1453` (root row), `:1497-1538` (planned children),
`:1675-1686` + `:1697-1760` (root gate and turn), `:3177-3182` (answer-only enforcement),
`:3388-3399` (memory write); `provider_executor.rs:35-38` (ROOT_SYSTEM), `:364-389` (finalize_root);
`tool.rs:392-395`; `http.rs:776`; `web/src/flow/graph.ts:235`.

**What it must NOT be.**
- Not a super-user: it has the **narrowest** tool list of any tier.
- Not a second chat agent and not a per-branch worker.
- Not creatable by `TEMPLATE_INSERT`, the planner, or `spawn_agent`.
- Not a process that keeps running after the run ends.
- Not the approver of its own or a child's side effect.

### 5.2 SuperAgent

**Purpose.** Own one branch/hive: decompose the branch objective into Agents (and, where allowed,
Subagents), keep the branch's context coherent, and return one branch result to the HyperAgent.
**[DR]** `12-role-capability-model.md:252-279`; `assets/prompts/superagent.prompty:17`.

**Authority.**
- *Current:* Tier SuperAgent with 7 tools; no write/exec **[SR]** `tool.rs:396-407`; profile
  `superagent` with `max_children=8`, `max_depth=3` **[SR]** `personas.json:21-30`, enforced in
  `spawn_sub_hive` **[SR]** `execution.rs:1189-1196`. No ownership field, no office, no branch id,
  no scope binding exist **[GV]** no `ownership`/`office`/`hive_id` in `backend/`; "branch" is only
  the memory `scope_id` convention.
- *Target:* four ownership modes (department, domain, branch, single task); exclusive scope; mode
  changes scope/lifetime/card chip, never tier; no Super->Super **[DR]**
  `12-role-capability-model.md:254-278`.

**Lifecycle / lifetime.** One task row at depth 1 under the root; lifetime = task. In S1 a plan
executes it as an **aggregator** after its children, with child results injected into its prompt
**[SR S1]** `execution.rs:1550-1610`; in S2 it runs when the readiness loop sees its deps and direct
children complete **[SR S2]** `execution.rs:1559-1680`. Spawned via `SPAWN_SUB_HIVE` it executes in
the deferred queue **[SR]** `execution.rs:1771-1821`. No office survives the task; "department
owner" does not exist.

**Memory.**
- Read: parent context facts (root facts), parent-scope memory + run memory at seed; re-read at
  start **[SR]** `execution.rs:943-969`, `:3005-3024`.
- Write: its own final text under its own task-id scope **[SR]** `execution.rs:3388-3399`.
- Target: branch/hive bank, project/domain bank in office modes; write branch/hive only; no task
  bank access for a remote. **[DR]** `12-role-capability-model.md:260-263, 292`.

**Communication.** `ask_parent` (to Hyper), `send_to_sibling` (same parent by default; cross-branch
needs approval) **[SR]** `tool.rs:396-407`, `execution.rs:2780-2802+`. Messages are write-only; no
answer path **[SR]** `execution.rs:2688-2757`.

**Spawn rights.** Agents and Subagents only **[SR]** `execution.rs:152-155`. No Super->Super, no
Hyper child, no ToolActor, no daemon **[SR]** `:130-133`; **[DR]** `12-role-capability-model.md:1069`.

**Tools / allowlist.** `spawn_agent`, `ask_parent`, `send_to_sibling`, `report_result`, `finish`,
`file_read`, `file_list` **[SR]** `tool.rs:396-407`; persona grant is the same set **[SR]**
`personas.json:25`. Deliberate ceiling: an orchestrator that writes files is doing an Agent's job.
**[DR]** `12-role-capability-model.md:457-459`.

**Failure / supervision.** A branch failure fails the run (fail-fast); the Hyper does not get to
retry or isolate the branch **[SR]** `execution.rs:1656-1679`. Approval-gated tools suspend and
resume the task, not the run **[SR]** `execution.rs:2596+`. No branch-level supervisor exists.

**Observability / UI.** Wire kind `superagent` **[SR]** `execution.rs:102`; `SuperAgentNode` 420x150
with a honey header, state chip, role caption, progress bar **[SR]** `web/src/flow/nodes.tsx:141-173`;
counted as `superagents` in metrics and in the swarm census **[SR]** `web/src/state/metrics.ts:41`,
`web/src/panels/SwarmPanel.tsx:85`. No ownership chip, no branch-bank summary, no child count beyond
the progress metric.

**Invariants.**
1. Depth 1; never spawns a Super or a Hyper.
2. No `workspace.write`, no `process.exec`.
3. Owns a scope; one active owner per scope id (target).
4. Reports one branch result; does not produce the run answer.
5. A mode change is a new instance, not an in-place promotion.

**Code mapping.** `execution.rs:102` (kind), `:152-156` (spawn), `:1189-1196` (profile caps),
`:1559-1619` (aggregator role), `:3388-3399` (memory); `tool.rs:396-407`; `personas.json:21-30`;
`assets/prompts/superagent.prompty`; `nodes.tsx:141-173`.

**What it must NOT be.**
- Not a writer or shell user.
- Not a second root and not a peer of another Super in the same scope.
- Not a rank above Agent in *capability*: it has fewer tools than an Agent.
- Not an "office" process that runs without a task; an inactive office has no card.
- Not the ChatCore persona.

### 5.3 Agent (regular Agent; wire `worker`)

**Purpose.** Execute one assigned task with tools and return one bounded result. The workhorse; the
only class expected to write files or run allowlisted programs (after approval).

**Authority.**
- *Current:* Tier WorkerAgent with all 9 tools **[SR]** `tool.rs:408-421`; write/exec require approval
  **[SR]** `tool.rs:55-57`, `:632-639`; may spawn **Subagent only** **[SR]**
  `execution.rs:156`.
- It is the **fallback tier for every unknown role** **[SR]** `execution.rs:183` and it has **no
  built-in profile** named `agent` **[SR]** `personas.json:1-67` (7 ids: planner, content-writer,
  superagent, subagent, data-analyst, code-reviewer, chatbot). So "regular Agent" is currently an
  unnamed, maximally-capable default.

**Lifecycle / lifetime.** Task row at depth 1-2 under the root or a Super; born Queued, runs once,
terminal one of the five terminal states; terminal is absorbing **[SR]** `execution.rs:3336-3355`.
No process outlives the task.

**Memory.**
- Read: parent facts + parent-scope memory + run memory (8+8) **[SR]** `execution.rs:936-962`;
  re-read at start **[SR]** `:2950-2977`; roster (16) and inbox (8) **[SR]** `:941-970`.
- Write: own final text, own task-id scope, 2,000 chars, secret-marker filtered **[SR]**
  `execution.rs:3381-3392`, `:60-65`.
- Target: branch bank read (own branch + ancestors + role bank); write own branch. **[DR]**
  `12-role-capability-model.md:292-293`.

**Communication.** `ask_parent` and `send_to_sibling` (same parent) **[SR]** `tool.rs:408-421`;
`report_result`/`finish` are the terminal verbs **[SR]** `execution.rs:3240-3286`. Messages are
write-only and budgeted at 32 per run **[SR]** `execution.rs:90-92`, `:2719-2735`.

**Spawn rights.** Subagent only **[SR]** `execution.rs:156`; cannot spawn an Agent (no
self-replication), a Super (no escalation), or a tool actor.

**Tools / allowlist.** All 9: spawn, ask, sibling, report, finish, file_read, file_list, file_write,
shell_exec **[SR]** `tool.rs:408-421`. A matched persona narrows this (e.g. `code-reviewer` loses
write/exec/spawn **[SR]** `personas.json:49-56`); an unmatched role keeps all 9.

**Failure / supervision.** Model/provider failure marks the task Failed and aborts the run
**[SR]** `execution.rs:3156-3173`, `:1656-1679`; a killed task is isolated and the run continues
**[SR]** `execution.rs:1656-1669`. No retry, no supervisor, no per-task restart.

**Observability / UI.** Wire kind `worker` **[SR]** `execution.rs:103`; `WorkerAgentNode` 320x96
**[SR]** `web/src/flow/nodes.tsx:175-194`; label is the role + id suffix, or `Agent <suffix>` when
the role is empty **[SR]** `http.rs:779-780`; counted as `workers` alongside subagents and tools
**[SR]** `web/src/state/metrics.ts:43`; the swarm census has a `workers` bucket **[SR]**
`SwarmPanel.tsx:87`.

**Invariants.**
1. One task, one result; owns no subtree beyond Subagents.
2. Writes/execs only through the approval gate and inside the workspace jail.
3. Cannot spawn an Agent/Super, cannot approve, cannot widen its grant.
4. Unknown role must **not** silently become this class (target: refuse or mark class unknown).

**Code mapping.** `execution.rs:103` (kind), `:156` (spawn), `:183` (fallback), `:1091-1105`
(authority), `:3388-3399` (memory); `tool.rs:408-421`; `nodes.tsx:175-194`; `metrics.ts:43`.

**What it must NOT be.**
- Not a sub-orchestrator (that is Super) and not a subagent (that is the narrowed rank).
- Not the default for arbitrary role strings.
- Not able to spawn an Agent, a Super, or a tool actor.
- Not a chat participant by default; chat is operator UI.
- Not a writer outside the workspace jail.

### 5.4 Subagent

**Purpose.** Do one narrow sub-task and return a precise result or a precise uncertainty; no
delegation, no side effects. **[DR]** `12-role-capability-model.md:299-313`;
`assets/prompts/subagent.prompty`.

**Authority.**
- *Current:* Profile `subagent` exists with 6 tools (ask, sibling, read, list, report, finish) and
  no spawn/write/exec **[SR]** `personas.json:31-39`; the call-time gate enforces it **only when the
  role string resolves to that profile** **[SR]** `execution.rs:1091-1105`, `:929-933`. Tier is
  `WorkerAgent` **[SR]** `execution.rs:183`; there is no Subagent tier and no class field.
- **The S1/S2 alias escalation is closed in S3.** The spawn matrix accepts both `"subagent"` and
  `"sub_agent"` **[SR]** `execution.rs:149-150`, `:156`, and the S3 persona lookup's compact match
  maps `"sub_agent"`, `"sub-agent"`, `"Subagent"` and other `-`/`_`/space variants to the
  `subagent` profile **[SR]** `execution.rs:588-609`, so those tasks get the 6-tool narrowed grant,
  not the 9-tool Worker grant. **The remaining escalation is an arbitrary unknown role** (F3), which
  still falls back to WorkerAgent with all 9 tools. This is a source-read finding, not a runtime test.
- The model is *advertised* the 9 Worker tools regardless **[SR]** `provider_executor.rs:335-351`.

**Lifecycle / lifetime.** Task row at depth 2-3 in the design **[DR]**
`12-role-capability-model.md:170-172`; **no depth floor exists in code**, and the root may create a
Subagent at depth 1 **[SR]** `execution.rs:138-151`, `:1497-1538`. The S3 template gate (§3.5) does
not add one either: a `"subagent"` template insert at depth 1 is still accepted. Terminal states as
any task. No children in the design; in code a canonical `subagent` is blocked at call time only by
its persona grant, not by rank.

**Memory.**
- Read: parent facts + parent-scope memory + run memory (8+8) **[SR]** `execution.rs:943-969`;
  inherits ancestor facts transitively via the parent snapshot **[SR]** `:921-929`.
- Write: own final text under its own task-id scope **[SR]** `execution.rs:3388-3399`.
- Target: own branch + ancestors + role bank; write only its own observation. **[DR]**
  `12-role-capability-model.md:309`.

**Communication.** `ask_parent`, `send_to_sibling`, `report_result`, `finish` **[SR]**
`personas.json:36`; no answer/deadline/wake today **[SR]** `execution.rs:2688-2757`.

**Spawn rights.** **None** in design **[DR]** `12-role-capability-model.md:305-306`. Current:
canonical `"subagent"` and its `-`/`_`/case variants are denied `spawn_agent` by the narrowed persona
grant (S3); an **arbitrary unmatched role string** still gets the full Worker grant, so the rank's
no-spawn invariant is **not enforced by rank or depth** — only by the persona lookup. A `"subagent"`
template insert at depth 1 is still accepted (§3.5).

**Tools / allowlist.** `ask_parent`, `send_to_sibling`, `file_read`, `file_list`, `report_result`,
`finish` **[SR]** `personas.json:36`; no write, no exec, no spawn. Advertised set is the 9 Worker
tools (mismatch) **[SR]** `provider_executor.rs:335-351`.

**Failure / supervision.** Same fail-fast behavior as any task. A Subagent cannot report a failure
upward except by returning text; `ask_parent` does not block or wait.

**Observability / UI.** **No server-emitted kind.** `node_kind(WorkerAgent)="worker"`
**[SR]** `execution.rs:121-128`; after the S2 `http.rs` change, roster/history also use the
tier-derived kind **[SR]** `http.rs:740-743`, `:1034-1040`; the contract vocabulary has no
`subagent` **[SR]** `event.rs:240`. The web nevertheless ships `SubagentNode` (compact card, label
"SUBAGENT - bounded worker") and a `subagent` census bucket **[SR]** `nodes.tsx:196-219`;
`agentState.ts:126-136`; `SwarmPanel.tsx:86`. Today the card has no honest producer; a Subagent
renders as an Agent.

**Invariants (target).**
1. Depth >= 2 (refused below, with a named refusal).
2. No spawn, no write, no exec - by grant and by rank, not by string.
3. Never renders as a plain Agent; its card says `SUB` and "narrowed".
4. Cannot be created by `TEMPLATE_INSERT` or a plan node at depth 1.
5. Its result is bounded text; it owns no children.

**Code mapping.** `execution.rs:121-128`, `:135-164`, `:173-185`, `:588-609`, `:929-933`,
`:1091-1105`, `:3388-3399`; `personas.json:31-39`; `tool.rs:408-421`; `nodes.tsx:196-219`;
`event.rs:240`; `http.rs:740-743`.

**What it must NOT be.**
- Not a fifth tier.
- Not a Worker with a different prompt.
- Not depth 1.
- Not a wire kind the server cannot emit while the UI pretends it exists.
- Not an alias (`sub_agent`) that escapes the narrowing.

### 5.5 Remote tiny agent (tree-free principal, device capability)

**Status: not implemented.** There is **zero** code for a remote agent, device, daemon, or
supervisor in `backend/` **[GV]** no `remote_agents`, `remote_id`, `devices`, `device_actions`,
`daemon` symbol; the only "roster" is `GET /api/agents/runs/{run_id}/roster` over tasks
**[SR]** `http.rs:536`, `:1019-1059`. Everything below is the design contract, tagged **[DR]**, with
the naming conflict flagged.

**Purpose.** Be the operator's tiny, tree-free delegate to the physical world: own a small fixed set
of devices/capabilities (canonical example: one light switch), expose a **basic function** (turn
on/off/toggle, plus a health/state read), read universal/global memory read-only, write only its
own private bank, and report health. It is deliberately simple: one job, one owner, one report
cadence. **[DR]** `docs/research/remote-agents/00-README.md:25-53`; `01-remote-agent-model.md:9-15`;
`03-device-capabilities.md:17-46`; `OPERATOR-DECISIONS.md:31-33`.

**Authority.**
- Enrolled by the operator; capabilities are a **grant**, not a class: effective =
  remote's enrolled set INTERSECT operator grant INTERSECT device-declared capability INTERSECT
  policy mode, minus revocations; empty/unknown = refuse. **[DR]**
  `01-remote-agent-model.md:64-82`.
- Per-device, per-action ACL; no wildcard; attenuation only; expiry; revocation immediate. **[DR]**
  `03-device-capabilities.md:56-73`.
- It cannot approve anything, self-grant, grant another remote, spawn tasks, create runs, read task
  banks, or use the browser session token. **[DR]** `00-README.md:69-78`;
  `01-remote-agent-model.md:64-82`.

**Lifecycle / lifetime.** Outlives runs. States: `enrolled -> online|offline|revoked`,
`online <-> degraded`, any -> `quarantined|revoked`, `revoked -> retired`; **no transition into any
task state** **[DR]** `01-remote-agent-model.md:85-100`. No task row, no `parent_task_id`, no graph
edge, no depth, no run/call/agent cap consumption **[DR]** `01-remote-agent-model.md:24-43`. It is
not a process per remote either: one supervisor lane probes/leases/reconnects **[DR]**
`01-remote-agent-model.md:137-146`.

**Memory.**
- Read set is exactly `{global:default, remote:<own_id>}`; global is operator-curated and read-only
  to every agent/remote; task-tree banks are invisible by construction (no ancestor walk, no "all
  banks" mode). **[DR]** `02-memory-isolation.md:39-53`.
- Write: only its own remote bank; entries are quarantined/unverified until operator promotion; no
  promotion in v1. **[DR]** `02-memory-isolation.md:54-58`.
- Budgets: <= 1,200 chars stable global + <= 2,800 chars own, max 12 entries; secrets refused; text
  is data, never instructions. **[DR]** `02-memory-isolation.md:59-73`.
- **Current tree:** no `global` bank and no `remote` bank exist; memory is `run` + task-id scopes
  only **[SR]** `db.rs:251-272`; **[GV]** no `memory_banks` symbol. "Universal memory" is a design
  policy, not a feature.

**Communication.** A2A-style external interop is optional and only over the remote transport;
internal verbs are HiveMind's own. Requests are bounded JSON (request id, remote id, device id,
capability, action, params, expected state, idempotency key, requested-by, approval id, deadline);
results are confirmed by a state read, and a transport 200 without confirmation is `unknown`.
**[DR]** `03-device-capabilities.md:75-116`; `01-remote-agent-model.md:119-133`.

**Spawn rights.** None. It cannot create a task, an office, a daemon, or another principal
**[DR]** `01-remote-agent-model.md:64-70`.

**Tools / allowlist.** No HiveMind model tools. Its entire action surface is the closed capability
inventory, e.g. `device.light.switch` with `turn_on|turn_off|toggle`, `power`/`brightness`/
`reachable` properties, low risk, state-read confirmation **[DR]**
`03-device-capabilities.md:30-46`. There is deliberately no generic `device.http`/`device.raw`
action: a remote must never become an open HTTP proxy. **[DR]** `03-device-capabilities.md:44-46`.

**Failure / supervision.** One supervisor lane owns leases/probes/backoff/quarantine and **cannot
approve or actuate**; daemons/watchdogs are records that observe and report, never model loops
**[DR]** `01-remote-agent-model.md:137-160`. Crash recovery marks `online -> unknown`, marks
in-flight actions `interrupted`, and **never replays actuation**; only safe reads retry **[DR]**
`01-remote-agent-model.md:171-186`.

**Observability / UI.** A tree-free **Remote lane** (roster/cards), not the canvas hierarchy; cards
carry switch, health dot, last-report age, next-report time, owner; `unknown|offline|stale|revoked|
quarantined` are first-class; no optimistic switch movement **[DR]**
`docs/research/remote-agents/05-ui-representation.md`; `00-README.md:46-52`. **Nothing of this
exists; there is no card and no event.**

**Invariants (design, R-1..R-12).** No task row/edge/cap; grant required per action; no cross-remote
read/control/spawn; audit every transition; device values carry provenance; at-most-once actuation;
daemons are records; one supervisor lane; credentials in the OS store only; memory isolated by bank;
no inbound listener in v1; every command/event/refusal has a matrix row and a named-refusal test.
**[DR]** `01-remote-agent-model.md:200-215`.

**Naming conflict that must be resolved (flag).** `docs/research/hmars/12-role-capability-model.md`
calls "Remote / TV-remote" a **daemon** with *no memory-bank writes*, read-only target scope, and an
"attached owner" edge on the canvas **[DR]** `12-role-capability-model.md:315-333`, `:451`. The
remote-agents folder defines a **tree-free capability principal** with a private bank, universal
memory read, and no canvas edge **[DR]** `01-remote-agent-model.md:9-43`. These cannot both be true.
The taxonomy in this draft adopts the remote-agents definition (it matches the task's "tree-free,
universal/global memory, device/light-switch capability, basic function") and reclassifies
Watchdog/Remote-as-daemon as **Watchdog daemon = record**, not a principal. 12's row is superseded.

**Code mapping.** None for the entity. Closest real surfaces: roster over tasks **[SR]**
`http.rs:1019-1059`; memory scopes **[SR]** `db.rs:251-272`, `execution.rs:3388-3399`; approval
bridge keyed to tasks **[SR]** `execution.rs:2596+`; event kinds **[SR]** `event.rs:240`. There is no
`remote_agents`, `devices`, `daemon_jobs`, `remote_supervisor_state`, or credential-store binding
**[GV]**.

**What it must NOT be.**
- Not a task, a tier, a rank, or a plan node.
- Not a daemon record with a model loop.
- Not a second scheduler; the supervisor is one lane, not one process per remote.
- Not a writer to global/system memory.
- Not a reader of run/hive/branch/role banks.
- Not an unattended actuator: garage/lock are never auto; light switch auto only under an explicit
  narrow grant and autonomy mode >= 2.
- Not a raw HTTP/shell proxy.

---

## 6. Decision table

Answers are one cell each. **All HiveMind cells are [SR]** from the anchors in section 5; design
columns are marked `D` where the answer is the target, not the code.

| Question | HyperAgent | SuperAgent | Agent (regular) | Subagent | Remote tiny agent |
|---|---|---|---|---|---|
| Rank | 1 | 2 | 3 | 4 | not a rank (principal) |
| Tier today | HyperAgent | SuperAgent | WorkerAgent | WorkerAgent | none |
| Is it a task row? | yes (root) | yes | yes | yes | **no** (D) |
| Parent | none | root (or owner in D) | root or Super | Agent or Super | **none, ever** |
| Depth | 0 | 1 | 1-2 | 2-3 (D); 1 possible today | n/a |
| Lifetime | the run | its task | its task | its task | across runs (D) |
| Exists outside a run | no | no | no | no | **yes** (D) |
| Accountable to | operator | Hyper | Super/root | its parent | operator (enrolled) |
| May spawn | Super, Agent, Subagent (via plan/command gate) | Agent, Subagent | Subagent | **nothing** | nothing |
| Spawnable by | nobody | Hyper | Hyper, Super | Hyper, Super, Agent | operator enrol only |
| Tool count today | 3 (never advertised) | 7 | 9 | 6 via profile, 9 advertised | 0 model tools; closed device verbs |
| Write / exec | no / no | no / no | approval / approval | no / no (by profile; unknown roles escape) | device actions only, approval per risk |
| Memory read | run + all-run aggregate (32) | parent + run + branch (D) | parent + run | parent + run | global RO + own bank (D) |
| Memory write | own task scope (if it runs) | own task scope | own task scope | own task scope | own remote bank, quarantined (D) |
| Outbound comms | none | ask_parent, sibling | ask_parent, sibling | ask_parent, sibling | request/ack/report on remote transport (D) |
| Approves side effects | never | never | never (operator does) | never | never (operator, or explicit auto grant) |
| Failure blast radius | the run | the run (fail-fast today) | the run (fail-fast today) | the run | one remote/device; never the tree (D) |
| Wire kind | `hyperagent` | `superagent` | `worker` | **none** | **none** |
| UI card | chat anchor (canvas root filtered) | SuperAgent card | Worker/Agent card | dead Subagent card | **none** (D: Remote lane) |
| Consumes MAX_AGENTS | yes | yes | yes | yes | **no** (D) |
| Implemented today | partial | partial | yes (unnamed) | profile only | **no** |

### Reading the table

- The **only** row that separates Remote from every other class structurally is "is it a task row?"
  and "exists outside a run". Everything else follows from those two.
- The **only** row that separates Subagent from Agent today is the profile grant. In S3 the alias
  variants (`sub_agent`, `sub-agent`, casing) canonicalize to that profile; an **arbitrary unknown
  role** still receives the full Worker grant, so the grant is only as strong as the persona lookup.
- The **only** row that separates Hyper from Super is rank/depth and the plan gate - the tool lists
  are already narrow for both, and Hyper's is narrower.
- No row shows a privilege hierarchy. The hierarchy is accountability; capability is the grant.

---

## 7. Where HiveMind's naming is false or overclaimed

Each item: the claim, where it is made, the code fact, and severity. Severities are this draft's
judgment, not the project's.

| # | Claim | Where claimed | Code fact | Severity |
|---|---|---|---|---|
| F1 | The operator's four-rank hierarchy is presented as a **runtime** model, although only the requirement exists. | Operator requirement **[DR]** `OPERATOR-DECISIONS.md:39`; UI labels **[SR]** `strings.ts:275-278` | There is **no rank field** and no Subagent tier; rank is inferred from `tasks.role` **[SR]** `execution.rs:173-185`. Subagent shares WorkerAgent's tier **[SR]** `tool.rs:72-81`. The requirement is real; the implementation is not. | High |
| F2 | "Subagent has a card and a wire kind." | Web kind union **[SR]** `contract.ts:213`; `agentState.ts:126`; `SubagentNode` **[SR]** `nodes.tsx:196-219`; swarm bucket **[SR]** `SwarmPanel.tsx:86` | No server path emits `subagent`: `node_kind` maps Worker->`worker` **[SR]** `execution.rs:121-128`; vocabulary is 4 values **[SR]** `event.rs:240`; the S2 `http.rs` fix removed the only producer **[SR]** `http.rs:737-744`, `:1031-1038`. Dead card. | High |
| F3 | "Unknown/foreign role is handled safely." | Implicit in "known roles are mapped explicitly" | Unknown role **falls back to the broadest tier** (9 tools) with no persona narrowing **[SR]** `execution.rs:183`; `tool.rs:408-421`. | **Critical** |
| F4 | "Subagent cannot spawn / is narrowed." | Docs **[DR]** `12-role-capability-model.md:305-306`; code comment **[SR]** `execution.rs:130-134` | S1/S2: `"sub_agent"` was accepted by the spawn matrix **[SR]** `execution.rs:149-150`, `:156` but did not match the persona id **[SR]** `personas.json:32`, so it got the 9-tool Worker grant. **S3 fixes it**: the compact persona match maps `"sub_agent"`/`"sub-agent"`/casing variants to `subagent` **[SR]** `execution.rs:588-609`. Remaining: an arbitrary unknown role still gets the 9-tool grant (F3). The alias finding is historical, not live. | Medium (was Critical) |
| F5 | "A profile has a tier." | `Persona.tier` **[SR]** `prompts.rs:389`; API field **[SR]** `http.rs:950`, `:1009` | `persona.tier` is **never read for authorization**; grep shows only serialization **[GV]**. Tier comes from the role string. A profile declaring `tier:"super"` and a task role `"worker"` silently gets the Worker tier. | High |
| F6 | "Subagent is legal only at depth >= 2." | **[DR]** `12-role-capability-model.md:170-172`; `21-roles-daemons-implementation.md:557-560` | No depth floor. Hyper's spawn list accepts `subagent` at depth 1 **[SR]** `execution.rs:138-151`. The draft's old evidence ("the plan loop calls only `caps.check_spawn`") was wrong: `execution.rs:1509` **is** `validate_spawn_role(parent_tier, &node.role)?`, and the plan loop has validated the role since `aa15ec9`. The gap is the missing **depth** floor, not a missing role check. | High |
| F7 | "HyperAgent orchestrates and produces the run answer." | **[DR]** `docs/provider/AGENT-ARCHITECTURE-TRUTH.md:17`; `provider_executor.rs:35-38` | S1: the root turn runs **only when the plan has an aggregator** **[SR S1]** `execution.rs:1554-1555`, `:1631`. S2 widens the gate to `has_aggregators or plan.nodes.len() > 1` **[SR S2]** `execution.rs:1668-1679`, so a 2+ node flat plan now gets a root turn, but a **single-node plan still does not**. On any plan it is answer-only and cannot use tools **[SR]** `:3171-3176`. | High |
| F8 | "HyperAgent is one per run and cannot be spawned." | **[DR]** `12-role-capability-model.md:233` (one per run); `21-roles-daemons-implementation.md:565-566` | S1/S2: `TEMPLATE_INSERT` bypassed `validate_spawn_role` and used the template id as the role **[SR]** `ws.rs:200-293`, so a depth-1 `"planner"` Hyper was one command away. **S3 closes it**: template inserts call `validate_spawn_role` (`ws.rs:247-250`) and set metadata (`:294-299`), so `"planner"` is refused. **New regression in the same fix:** the four shipped Library templates are now refused because their ids are not on the Hyper spawn allowlist; see M7. | Medium (was Critical; Hyper path resolved, template-library regression open) |
| F9 | "40 provider calls / run is a hard cap." | `/api/runtime` exposes `max_calls:40` **[SR]** `http.rs:1412`; caps doc comment **[SR]** `caps.rs:14-15` | Both `check_calls` call sites pass the literal `1` **[SR]** `execution.rs:1435`, `:3129`, so the check cannot fail. Real bounds are the tool-loop budget **[SR]** `tool_loop.rs:16`, `execution.rs:1911` and the provider `RunBudget`. The number is advertised but not enforced by this path. | Medium |
| F10 | "Remote agent exists / universal memory exists." | UI/roadmap language; **[DR]** remote-agents folder; operator decisions | Zero remote/device/daemon symbols **[GV]**; memory scopes are run + task only **[SR]** `db.rs:251-272`. There is no global bank and no remote bank. | High |
| F11 | "Remote is a daemon." | **[DR]** `12-role-capability-model.md:315-333`, `:451` | Contradicts the remote-agents design (tree-free principal, private bank) **[DR]** `01-remote-agent-model.md:9-43`. No code for either. The naming conflict itself is the defect. | High |
| F12 | "A2A messaging." | `ask_parent`/`send_to_sibling` are called A2A **[SR]** `execution.rs:2735-2760`; docs **[DR]** `23-a2a-implementation.md` | Messages are persisted rows with no answer, no wake, no deadline, no delivery state, no capability negotiation, no discovery **[SR]** `execution.rs:2681-2750`; inbox refreshed only at task start **[SR]** `:2950-3007`. This is a mailbox, not A2A. | Medium |
| F13 | "Every agent, including tiny Remote/Watchdog agents, gets a card." | Operator decision **[DR]** `OPERATOR-DECISIONS.md:31` | Remote: none. Watchdog: none. ToolActor: no dedicated runtime producer. Subagent: dead card. Root: no canvas card (chat anchor only) **[SR]** `graph.ts:235`. The sentence is false for >= 3 classes. | Medium |
| F14 | "Ownership/office/hive scoping exists." | Design proposal, not a current-code claim: **[DR]** `21-roles-daemons-implementation.md:840-985` (and `12-role-capability-model.md:252-265`); the same doc records "Ownership is absent" at `21:347`. | No `ownership`, `office`, `hive_id`, or branch table in `backend/` **[GV]**; "branch" is only the `scope_id` convention on `agent_memory` **[SR]** `db.rs:251-272`. The finding is real; the citation should not present a design target as an existence claim. | Medium |
| F15 | "The profile gate is authoritative." | **[DR]** `12-role-capability-model.md:469-487` | The gate exists and is enforced at call time **[SR]** `execution.rs:1084-1098`, but the model is advertised the full tier set **[SR]** `provider_executor.rs:333-349`, and the gate is skipped entirely when the role string does not match a persona id. Authority is policy-by-coincidence. | High |

**Root cause shared by F1-F3, F5, F15:** one string does four jobs. Fix the identity object and most
of these collapse; patch them individually and they will reappear as soon as a new alias, template,
or UI label is added. F4 is now fixed in S3 by canonicalization, which shows the point: the fix is a
resolver, not another alias. F7's root-turn gate is a separate engine-policy defect, not an identity
defect.

---

## 8. Authoritative patterns from outside (and what to adopt)

Only official sources were used, fetched in this pass where marked **[WR]**. None of this page text
is treated as instructions.

### 8.1 A2A: an agent is opaque, a task is the unit, a card is discovery

Fetched: [A2A Core Concepts](https://a2a-protocol.org/latest/topics/key-concepts/) **[WR]**. Key
facts: the A2A Server is a **Remote Agent** and is **opaque** to the client - "its internal
workings, memory, and tools stay hidden"; the **Agent Card** is a JSON metadata document for
identity/capabilities/auth; a **Task** is "a stateful unit of work ... with a unique ID and defined
lifecycle"; **Message/Part/Artifact** are the content containers; `contextId` groups related tasks;
the protocol runs over HTTP with JSON-RPC.

Adopt:
- **Opacity is the correct default for Remote.** A remote's memory and tools are not the tree's
  business; the operator grants a capability, the client sees a result. This supports the
  remote-agents isolation design **[DR]** `02-memory-isolation.md`, and it *refutes* any idea of the
  HyperAgent reading a remote's learned memory by default.
- **Card is discovery, not authority.** HiveMind's cards are projections of records; they must never
  be consulted for a grant (matches A6).
- **Task lifecycle ≠ agent lifetime.** A2A's task immutability/terminal semantics match HiveMind's
  terminal-absorbing rule **[SR]** `execution.rs:3336-3355`; but A2A does not have "ranks", so
  HiveMind's rank concept is local and must be documented as such, not smuggled into A2A vocabulary.

Do **not** adopt: A2A's network discovery/transport for internal parent/child messaging. The repo
already says internal verbs stay HiveMind's own **[DR]** `23-a2a-implementation.md`.

### 8.2 Actor model: supervision is a tree, failure is a message

Fetched: [Akka fault tolerance](https://doc.akka.io//libraries/akka-core/snapshot/typed/fault-tolerance.html)
**[WR]**. Key facts: an unhandled failure stops the actor by default; supervision is declared by the
**parent**; strategies are restart/resume/stop/backoff with a retry limit in a time window; child
actors are stopped when the parent restarts; a parent that does not handle a child's termination
fails itself (`DeathPactException`), so failures **bubble up the hierarchy**.

Adopt:
- **A supervisor is the parent, and its policy is explicit.** HiveMind has no supervision policy
  field: the parent's only option today is "the run fails". Add a per-class failure policy
  (fail-run / fail-branch / kill-subtree / quarantine) rather than an implicit global fail-fast.
- **"Let it crash" needs a fresh instance.** HiveMind's `retry` must create a new task row (design
  already says restart is a new instance **[DR]** `12-role-capability-model.md:1161-1166`), never
  resurrect a terminal row.
- **Bubbling is a signal, not a prompt.** A child failure should be an event the parent can observe
  and decide on; today it is an `Err` that unwinds `dispatch_inner` **[SR]**
  `execution.rs:1530-1557`.

Do **not** adopt: one actor process per agent. HiveMind's tasks are durable rows executed serially
by one engine; a process per agent would multiply failure surfaces without adding capability.

### 8.3 HMAS: five axes, and a hierarchy is a trade, not a virtue

Fetched: [A Taxonomy of Hierarchical Multi-Agent Systems (arXiv 2508.12683)](https://export.arxiv.org/abs/2508.12683)
**[WR]**. The abstract proposes five axes: **control hierarchy, information flow, role and task
delegation, temporal layering, and communication structure**, and notes hierarchies "simplify
coordination, but ... also introduce trade-offs". It connects structure to coordination mechanisms
(contract-net, hierarchical RL) and flags explainability to human operators and scaling as open
problems.

Adopt as an audit lens:

| Axis | HiveMind today | Consequence |
|---|---|---|
| Control hierarchy | 3 spawn levels, but one serial engine decides execution order **[SR]** `execution.rs:1519-1705` | Hierarchy buys structure, not parallelism; latency is additive. |
| Information flow | snapshot inheritance + write-only mailbox **[SR]** `execution.rs:936-992` | Upward flow is prompt text; downward flow is a snapshot; there is no feedback channel. |
| Role/task delegation | role string + spawn allowlist **[SR]** `execution.rs:135-185` | Delegation is string-matched; no capability negotiation, no contract-net. |
| Temporal layering | none: agents exist only for a task **[SR]** `execution.rs:3336-3355` | No long-lived owner; an "office" is a design idea only. |
| Communication structure | parent/sibling only **[SR]** `tool.rs:108-127` | No mailbox semantics, no delivery state; "A2A" is a misnomer. |

This is exactly why Remote needs a different axis: it has **no control-hierarchy position** and
**no temporal layer inside a run**. It is a principal with a lifetime, not a node in the tree.

### 8.4 Agents-as-tools vs handoffs: two different delegation semantics

Fetched: [OpenAI Agents SDK - Agent orchestration](https://openai.github.io/openai-agents-python/multi_agent/)
**[WR]**. "Agents as tools": a manager keeps control and calls a specialist as a bounded tool.
"Handoffs": the specialist becomes the active agent for the rest of the turn.

Map:
- HiveMind `spawn_agent` is **agents-as-tools**: parent retains control, child runs inline, parent
  receives a bounded result **[SR]** `execution.rs:2882-2993`. This is the right shape for
  Subagent and for a SuperAgent's workers.
- HiveMind's root final turn is closer to a **handoff**: control passes to the root for the final
  answer, and the root cannot call tools **[SR]** `execution.rs:1621-1701`. That is coherent only if
  the root is a single answer-only synthesizer; it must not also be the planner mid-run.
- Remote is neither: it is a **task delegation to an opaque principal** (A2A-style), with its own
  lifecycle and its own approvals.

The distinction yields the rank rule: **a rank is who retains control of the work, not who has more
tools.** Hyper retains control of the run and loses tool authority; Agent retains control of one task
and holds the tools; Remote retains control of nothing in the tree.

---

## 9. Design axioms (what "make the taxonomy real" means)

These are the testable statements. Each maps to a section-10 change or an explicit non-change.

- **I1. Identity is persisted, not parsed.** Every task stores its class, rank, and profile id at
  creation; no consumer re-derives identity from a display string.
- **I2. Rank is not capability.** Rank constrains spawn and reporting; the grant is the only thing an
  authorization check may consult.
- **I3. Admission is one function.** Every spawn source (plan, command, model tool, template,
  future remote call) passes the same admission with named refusals.
- **I4. Profiles are versioned and tier-consistent.** A profile's declared tier must equal the
  resolved tier, or the profile is invalid at load.
- **I5. Narrowing is total.** No alias, casing, separator, template id, or unknown string may widen a
  grant. Unknown is a refusal, not a fallback.
- **I6. One Hyper, one answer.** Exactly one Hyper per run; it always produces the run answer (even
  for a flat plan); nothing else can create one.
- **I7. A subagent is depth-bounded and tool-poor.** Depth >= 2, no spawn, no write, no exec, by rank
  and by grant.
- **I8. Memory scope follows class/ownership, not role text.** The writer and the reader agree on the
  same scope key; the root's aggregate read is explicit, bounded, and labelled.
- **I9. A message is a record with state.** No "A2A" claim without answer/expiry/delivery semantics.
- **I10. Failure policy is per class.** Fail-run, fail-branch, kill-subtree, quarantine - named, not
  implicit.
- **I11. Remote is a principal, not a task.** No `tasks` row, no depth, no NODE_*, no cap use; its
  memory is global-read-only + own bank only.
- **I12. Every wire kind has a producer and every class has exactly one card.** Unknown renders
  "class not reported", never `worker`.
- **I13. Every authority claim has a test.** A doc sentence that says "cannot" must map to a refusal
  code and a test that forges the call.

---

## 10. Minimal code changes to make the taxonomy real

Ordered. "Minimal" means: no new scheduler, no new UI surface, no second server, no provider change.
Each row names the exact seam and what it buys. Estimates are rough engineer-days for one person
already familiar with the code.

### M1 - One identity resolver (0.5-1 day) - prerequisite for everything

Add `AgentClass {Hyper, Super, Agent, Subagent, ToolActor}` and a pure
`resolve_identity(role, depth, profile) -> Identity {class, rank, tier, profile_id}`. Replace:
`task_agent_tier` **[SR]** `execution.rs:173-185`; `node_kind` **[SR]** `:121-128`;
`validate_spawn_role`'s four hand-written lists **[SR]** `:135-164`; the label mapper **[SR]**
`http.rs:765-781`; template role handling **[SR]** `ws.rs:240-301`.
**Rule:** the resolver accepts only canonical ids and known aliases; **anything else is
`E_PERSONA_UNKNOWN`**. This closes F3 and makes the S3 F4 canonicalization durable (S3 resolves
aliases in `persona_for_role`, not in one identity record).
**Risk:** every existing test that spawns `"worker"` still passes; a fixture using an invented role
must now expect a refusal.

### M2 - Persist identity on the task row (0.5-1 day, migration v12)

Add `class`, `rank`, `profile_id` (nullable for legacy rows) to `tasks`; write them at insert in
`spawn_sub_hive` **[SR]** `execution.rs:1174-1191` and the plan loop **[SR]** `:1504-1519`.
Read them for wire kind, roster, history, and card. Legacy rows with NULL class are reported as
`class_unreported`, never guessed.
This is the change that makes replay honest and removes the repeated string parsing (the same defect
class as the old `node_kind_for_role`).

### M3 - Make the Subagent kind real, once (0.25 day)

Add `"subagent"` to `NODE_KIND_VOCABULARY` **[SR]** `event.rs:240` and its mirror test, and emit the
kind from the M1 resolver in **both** live **[SR]** `execution.rs:121-128` and HTTP **[SR]**
`http.rs:740-743`. The web already renders it **[SR]** `nodes.tsx:196-219`; the contract test at
`event.rs:915-924` changes from four to five values. If the Lead refuses the contract change, the
alternative is to delete `subagent` from `NODE_KINDS` and `SubagentNode` - but then the operator's
fourth rank is invisible and the taxonomy claim must be withdrawn.

### M4 - Admission and withholding at the boundary (0.5 day)

- In `spawn_sub_hive` **[SR]** `execution.rs:1160-1217` and the plan loop **[SR]** `:1456-1498`:
  refuse `class=Subagent` at `depth < 2`; refuse `class=Hyper` at `depth >= 1`; keep one Hyper per
  run. Use one new named refusal (or reuse `E_SPAWN_DEPTH` with a precise message until the contract
  delta lands).
- In `tool_specs_for_tier` call sites **[SR]** `provider_executor.rs:333-349`: withhold
  `spawn_agent` from the **advertised** set when the class is Subagent or depth == max. Keep the
  call-time refusal as the complete mediation check.
**Why both:** removal changes what the model plans; refusal alone invites retries. This is the
Claude Code boundary pattern recorded in the repo docs **[DR]** `12-role-capability-model.md:476-483`.

### M5 - Make profiles authoritative and complete (0.5 day)

- Validate `PersonaTier` against the resolved tier at load **[SR]** `prompts.rs:411-434`, or delete
  `PersonaTier` and derive tier from the profile id. A profile whose tier does not match is invalid.
- Add the missing built-in `agent` profile (the regular Agent has no profile today
  **[SR]** `personas.json:1-67`), with the tier-consistent grant.
- Move alias canonicalization into the persisted identity resolver (M1): S3 already canonicalizes
  `"Subagent"`, `"sub_agent"`, `"sub-agent"` and casing in `persona_for_role`
  (`execution.rs:588-609`), but the identity is still not persisted and `"WorkerAgent"` still has no
  profile. The resolver must reject unknown ids, not normalize forever.
- Map template ids to spawn roles/profiles (M7); validating the raw template id against the Hyper
  spawn allowlist refuses the shipped Library templates.
**Do not** add a fourth `AgentTier`. Subagent stays rank 4 on WorkerAgent.

### M6 - One Hyper, one answer (0.5 day)

Remove the remaining single-node gate. S2 has already widened the condition to
`has_aggregators || plan.nodes.len() > 1` (committed `66aff50`) **[SR S2/S3]** `execution.rs:1686`;
the residual gap is the **single-node plan**, which still has no HyperAgent turn. Always run the root
synthesis when there is at least one completed child, or record "root not required" as an explicit
metric rather than leaving the class silent. S3 already refuses a `"planner"` template insert
(`ws.rs:247-250`), but keep the class-level guard when M1/M7 land: a Hyper instance must be
root-only, and template ids must map to non-Hyper spawn roles.
**Why minimal:** without this, the name "HyperAgent" promises a run owner that, on a one-node plan,
never speaks.

### M7 - Close the template side door (0.25 day) - **partially landed in S3, regression open**

S3 already calls `validate_spawn_role(root_tier, role)` in `insert_template_dag` (`ws.rs:247-250`)
and writes graph metadata (`ws.rs:294-299`), which closes the depth-1 `"planner"` Hyper path.
**But the role passed is still the raw template id**, so the shipped Library templates
(`content-writer`, `data-analyst`, `code-reviewer`, `chatbot`) are refused by the Hyper root's spawn
allowlist, and the in-file tests `template_insert_spawns_manifest_backed_dag` (`"planner"`) and
`template_insert_resolves_the_library_ids_after_a_real_lookup` would fail by construction. The
remaining change is to resolve the template id to a spawn role/persona (e.g. `agent`/worker for the
Library templates) and then run the same admission; refuse `planner`/Hyper and Subagent-at-depth-1.
Keep the metadata write. This is the smallest change that removes the invisible second-root class
without breaking the shipped template catalog.

### M8 - Memory identity and scope (1-1.5 days)

- Write with `agent_key = profile_id` (canonical), not `task.role` **[SR]**
  `execution.rs:3388-3399`, so the memory surface **[SR]** `http.rs:1218-1228` and the writer agree.
- Derive `scope_id` from class/ownership: root -> `"run"`; Super -> its hive/branch id; Agent ->
  its branch; Subagent -> its own task. Today **task completion** writes its own task id
  **[SR]** `execution.rs:3396`, while the **engine** writes the `"run"` scope for prompt/plan
  **[SR]** `:1462-1485`; two writers use two conventions. The root read is all-scope
  **[SR]** `:1705`.
- Make the root aggregate read a named "run digest" with an explicit count and omission reason
  (the title "Shared hive/branch memory" is currently broader than the data).
This is the last change needed for "universal memory" to mean a *policy*, not a phrase. The full
bank tree (global/project/run/hive/branch/role/remote) remains a separate lane **[DR]**
`docs/research/hmars/22-memory-bank-implementation.md`.

### M9 - Remote: do not fake it (design gate, not a patch)

There is nothing to "fix" because there is nothing built **[GV]**. The minimal honest move is:
1. State in UI/docs that Remote is not implemented (no card, no command, no route).
2. Add a **guard test** that asserts no `remote_*`/`device_*` task row or `NODE_*` kind can be
   produced until the R1 migration exists.
3. Only then start Delta R1: one migration + `remote_agents` + `devices` + `remote_audit` records, a
   tree-free roster projection, and a supervisor lane. Never a task row, never a `NODE_SPAWN`.
The remote-agents design is already detailed enough to build **[DR]**
`docs/research/remote-agents/04-contracts-api-ws-refusals.md`; do not re-derive it.

### Explicit non-changes (so the change set stays minimal)

- Do **not** add a fourth tier or a new `rank` enum for UI only.
- Do **not** implement ownership/offices/hives to make SuperAgent "real" in this change set; the
  class can be honest without offices by saying "branch = task subtree".
- Do **not** add A2A wire interop; fix the mailbox semantics first (answer/expiry), or stop calling
  it A2A.
- Do **not** add a scheduler or parallel dispatch; serial execution is a documented limit, not a
  taxonomy bug.
- Do **not** make the chat core an agent.

---

## 11. Acceptance evidence to demand (static/unit first; runtime later)

Names are proposed. Each must fail before the fix and pass after; none were run in this pass.
**S3 note:** `sub_agent_alias_is_canonical` and `template_insert_cannot_create_hyper` are now
satisfied by construction in the worktree (uncommitted, unbuilt), but the template fix needs a
companion regression test `template_insert_keeps_library_templates` - the current raw-id validation
would make the four shipped Library templates fail. All other rows still describe open gaps.

| Test | Asserts | Kills |
|---|---|---|
| `identity_resolver_is_total` | Every canonical id + alias resolves; an unknown id returns `E_PERSONA_UNKNOWN` | F1, F3, F4 |
| `sub_agent_alias_is_canonical` | `"sub_agent"` / `"Subagent"` / `"SUBAGENT"` resolve to the same profile and the narrowed grant | F4, F5 |
| `subagent_depth_floor` | depth 1 Subagent refused by plan, command, and model spawn | F6 |
| `template_insert_cannot_create_hyper` | `planner` template at depth 1 refused; no graph node without metadata | F8 |
| `root_turn_runs_for_flat_plan` | a 1-node plan still produces a HyperAgent answer (or an explicit "not required" metric) | F7 |
| `kind_is_emitted_once` | live, history, and roster all use the same resolver; every emitted kind is in the contract vocabulary | F2 |
| `unknown_kind_is_not_worker` | web renders "class not reported" for an absent kind (no coercion) | F2/F13 |
| `persona_tier_matches_runtime_tier` | a mismatched profile fails to load | F5 |
| `memory_writer_and_reader_agree` | completion write key/scope is listed by `/api/agents/memory` | M8 |
| `remote_has_no_task_row` | no remote/device symbol can create a `tasks` row or `NODE_*` event | F10 |
| `caps_check_calls_is_not_a_literal` | the advertised 40-call cap either counts or is removed from `/api/runtime` | F9 |

"Done" for this taxonomy means the first six pass **and** the doc sentences in section 7 are either
backed by those tests or deleted. A doc claim without a test is a wish.

---

## 12. Open decisions (explicit, not silently chosen)

1. **Subagent: rank 4 on the Worker tier, or a real fourth tier?** Recommendation: rank/profile on
   Worker (matches the docs and avoids a terminal-state surface). The UI must not imply a tier.
2. **Does HyperAgent ever get tools?** Recommendation: no. If the operator wants a conversable
   orchestrator, that is a new class ("chat agent") and must be decided, not smuggled into the root.
3. **Is the ChatCore ever an agent?** Operator decision is deferred **[DR]**
   `OPERATOR-DECISIONS.md:43-44`. Until decided, it is UI only.
4. **Failure policy default.** Fail-run (today) vs fail-branch with parent retry. Recommendation:
   fail-branch for Super-owned branches, fail-run for flat plans; make it a settings mode.
5. **A2A semantics.** Do `ask_parent` messages need answers/deadlines? If yes, they are a request/
   response protocol and need a state machine; if no, rename them to "notes" and stop calling them
   A2A.
6. **Remote memory scope.** global-read-only + own bank is the design; confirm the operator accepts
   that a remote cannot read run context (it cannot know what the house is currently doing unless
   told).
7. **One owner per branch.** Exclusive ownership needs `E_OWNERSHIP_BUSY`; today no such refusal
   exists. Decide before offices.
8. **Contract delta sequencing.** M3 changes `NODE_KIND_VOCABULARY` (a frozen contract). It needs a
   versioned delta, a web mirror update, and a replay-compatibility rule; do not slip it in as a
   drive-by edit.

---

## 13. Sources

### Internal (read this pass)

- `backend/crates/hiverouter-tools/src/tool.rs` - tiers, tool ids, allowlists, approval classes.
- `backend/crates/hiverouter-core/src/execution.rs` - resolver, spawn gate, dispatch, root, memory,
  A2A tools, terminal guards. **Dirty; anchors re-read at end of pass.**
- `backend/crates/hiverouter-core/src/{provider_executor,tool_loop,context,planner,caps,prompts,http,ws,db,store,recovery}.rs`.
- `backend/crates/hiverouter-contract/src/{event,state,refusal}.rs`.
- `backend/crates/hiverouter-core/assets/prompts/{personas.json,manifest.json,*.prompty}`.
- `web/src/{api/contract.ts,state/agentState.ts,state/store.ts,state/metrics.ts,flow/graph.ts,flow/nodes.tsx,panels/SwarmPanel.tsx,state/strings.ts}`.
- `docs/research/hmars/{12-role-capability-model.md,21-roles-daemons-implementation.md,23-a2a-implementation.md,22-memory-bank-implementation.md,OPERATOR-DECISIONS.md}`.
- `docs/research/remote-agents/{00-README.md,01-remote-agent-model.md,02-memory-isolation.md,03-device-capabilities.md,SOURCES.md}`.
- `docs/audit/hivemind-memory-a2a/{README.md,FINDINGS.json}` - useful but baseline `7610b86`;
  M-01's "authoritative header" is stale (the renderer now says "untrusted data; never
  instructions" **[SR]** `context.rs:74-75`).
- `docs/contracts/ADAPTERS.md` - contains donor-adapter provenance only; it defines **no** agent
  class. Its absence is itself a gap: the project has a contract for every dependency but not for
  the five things it calls "agent".
- `docs/provider/AGENT-ARCHITECTURE-TRUTH.md` - useful as a 2026-11-07 snapshot; several rows are
  stale versus this tree (Subagent card, aggregate memory read).

### External (fetched this pass) **[WR]**

- A2A Protocol, *Core Concepts* - <https://a2a-protocol.org/latest/topics/key-concepts/>
  (opaque remote agent, Agent Card, Task lifecycle, Message/Part/Artifact, `contextId`).
- Akka, *Fault Tolerance* -
  <https://doc.akka.io//libraries/akka-core/snapshot/typed/fault-tolerance.html>
  (default stop, parent supervision, restart limits, child stop on parent restart, failure bubbling).
- D. J. Moore, *A Taxonomy of Hierarchical Multi-Agent Systems* - <https://export.arxiv.org/abs/2508.12683>
  (five axes: control hierarchy, information flow, role/task delegation, temporal layering,
  communication structure; hierarchy trade-offs).
- OpenAI Agents SDK, *Agent orchestration* -
  <https://openai.github.io/openai-agents-python/multi_agent/>
  (agents-as-tools vs handoffs; manager retains control vs specialist becomes active).

Not used by rule: forums, social posts, SEO summaries, AI-generated pages. No paid API was called.

---

## 14. Claim register (top claims only)

| # | Claim | Tag | Basis |
|---|---|---|---|
| C1 | There is no `Subagent` tier or `AgentClass`; Subagent maps to WorkerAgent. | SR | `tool.rs:72-81`; `execution.rs:173-185` |
| C2 | `tasks.role` is the only identity source; unknown roles fall back to the broadest tier. | SR | `execution.rs:173-185`, `:183` |
| C3 | A matched persona narrows tools at call time; an empty persona grant means none. | SR | `execution.rs:1091-1105`; `personas.json:7` |
| C4 | **Historical (S1/S2):** the `sub_agent` alias bypassed the Subagent persona and got the full Worker grant. **S3 refutes this:** the compact persona match canonicalizes alias/case/separator variants. Unknown roles still escape (C2/F3). | SR | `execution.rs:588-609`; `:149-156`; `personas.json:32` |
| C5 | No depth floor exists for Subagent. | SR | `execution.rs:138-151`, `:1509` |
| C6 | S1: the root turn ran only for aggregating plans. S2/S3: for plans with 2+ nodes; a single-node plan still has no root turn. | SR | S1 `execution.rs:1554-1555`, `:1631`; S2/S3 `:1675-1686` |
| C7 | **Historical (S1/S2):** `TEMPLATE_INSERT` bypassed the spawn gate and could create a depth-1 Hyper. **S3 refuses it** via `validate_spawn_role` (`ws.rs:247-250`) but over-restricts the shipped Library templates; see M7. | SR | `ws.rs:247-250`, `:294-299` |
| C8 | No server path emits `subagent`; the web card has no producer. | SR | `execution.rs:121-128`; `event.rs:240`; `http.rs:740-743` |
| C9 | Remote/device/daemon code is absent. | GV | zero symbols under `backend/crates/*/src` |
| C10 | Memory scopes are run + task-id only; no global/remote bank. | SR | `db.rs:251-272`; `execution.rs:3388-3399` |
| C11 | `caps.check_calls` is called with a literal `1`; the 40-call cap is not enforced there. | SR | `execution.rs:1435`, `:3129`; `caps.rs:66-71` |
| C12 | A2A is a write-only mailbox (no answer/deadline/delivery state). | SR | `execution.rs:2681-2750` |
| C13 | A2A remote agents are opaque; a card is discovery, not authority. | WR | a2a-protocol.org key concepts |
| C14 | Actor supervision is parent-owned with explicit strategies and bubbling. | WR | Akka fault tolerance |
| C15 | HMAS is usefully audited on five axes; hierarchy is a trade-off. | WR | arXiv 2508.12683 |
| C16 | Agents-as-tools vs handoffs are distinct control-transfer semantics. | WR | OpenAI Agents SDK orchestration |

**No claim in this file is runtime-verified.** Anything that would require a run - whether a Subagent
actually gets 9 tools, whether the root turn fires, whether the alias spawns - is marked as a
source-read consequence and must be tested before it is treated as done.

---


---

## 15. The minimal change set, in one paragraph (the answer to "make it real")

One identity type and one resolver (M1), persisted on the task row (M2), used by every producer of
kind (M3); one admission function with a Subagent depth floor and spawn withholding (M4); profiles
that are tier-consistent and complete, with aliases canonicalized (M5); a root turn for every
multi-node plan and exactly one Hyper per run, including the template path (M6/M7); memory keys and
scopes derived from class/ownership (M8). Remote stays out of the task model until its own
schema/records exist (M9); until then the word "Remote" must appear in the UI and docs as
"not implemented", never as a card, a kind, or a tier. Everything else in this document exists to
justify those seven changes and to name, with `file:line`, what is false if they are not made.

That is the whole taxonomy: **rank is accountability, profile is policy, principal is lifetime,
record is evidence, process is an implementation detail - and only the task row binds them for the
duration of one run.**