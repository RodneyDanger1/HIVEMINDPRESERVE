# 07 — CODE BACKEND: agent tiers, hierarchy, memory, A2A, tools, and what actually blocks 50–150 / 300–600

**Analyst:** Codebase Analyst #1 (read-only) · **Date:** 2026-11-07 (America/Chicago)
**Repository:** `C:\Users\Riley\Desktop\hiverouter-v1` · **HEAD read:** `bd575ba` `feat: real SuperAgent/Subagent personas and persona prompt execution`
**Tree at read:** dirty but only non-runtime paths — `M docs/provider/AGENT-ARCHITECTURE-TRUTH.md`, `M docs/ui/figcraft/.../__pycache__/*.pyc`, `M tools/desktop-launcher/HiveMind.exe`, `?? .claude/`, `?? AGENTS.md`, `?? CLAUDE.md`. **No product source edit, no build, no test run, no server restart, no credentials read** (only environment-variable *names* appear in the source).
**HMARS corpus:** `docs/research/hmars/` now exists with peer reports `01-deepseek-api.md` … `06-security-cost.md`. Those are cited as `[READ-DOC]` for goal/limits context; every product claim below is grounded in product source `path:line`.
**Label legend:** `VERIFIED` = independently cross-confirmed in two reads/derivations; `READ` = directly read in the cited source once; `INFERRED` = reasoned from cited code, not directly stated; `UNVERIFIED` = runtime behaviour not executed in this pass; `READ-DOC` = peer research document read (not product proof).

---

## 0. Top 10 findings (executive)

1. **The "HyperAgent" is a planner *row* and a UI anchor, not a running model agent.** The engine creates one root task `role="planner"`, `depth=0` (`execution.rs:1321-1335`) and publishes it as `kind=hyperagent` (`execution.rs:97-106`, `:764-779`); no dispatch loop ever executes the root (`planner.rs:143-193`; `execution.rs:1395-1490` dispatches only plan nodes and the spawn queue). The single planning LLM call is a separate adapter call with `PLANNER_SYSTEM` (`provider_executor.rs:25-29,286-313`).
2. **ChatCore is an operator UI, not a SuperAgent.** It submits `SUBMIT_PROMPT` and renders streamed replies (`web/src/chat/ChatCore.tsx:1-5,77-86,140-152`; store `submitPrompt` at `state/store.ts:1835-1847`); the root is rendered as the fixed chat-core overlay and filtered out of the canvas node list (`web/src/flow/graph.ts:26-31,99,151,172`; aria label `strings.ts:77`).
3. **Subagent is a role/persona, not an agent tier.** `AgentTier` has only Hyper/Super/Worker/Tool (`hiverouter-tools/src/tool.rs:66-100`); `task_agent_tier` maps every unrecognized role — including `subagent` — to `WorkerAgent` (`execution.rs:151-163`); the contract node-kind vocabulary contains no `subagent` (`contract/src/event.rs:240`). The persona/prompt/allowlist exist (`personas.json:32-37`; `subagent.prompty:16-19`).
4. **Persona prompts reach exactly one model call.** Leaf calls resolve `task.role` against the rendered persona-system map (`provider_executor.rs:82-87,315-348`), but tool-turn continuations hard-code `TASK_SYSTEM` (`provider_executor.rs:369-372`), and the planner call uses `PLANNER_SYSTEM` (`:295`), so a persona identity/shape is lost after the first tool round and the planner persona never runs.
5. **Of seven shipped personas, four worker personas are not reachable through the spawn policy.** `validate_spawn_role` accepts only hyper/super/worker/agent/subagent names (`execution.rs:113-142`); `content-writer`, `data-analyst`, `code-reviewer`, `chatbot` are refused by `SPAWN_SUB_HIVE`/`spawn_agent`. They are reachable via `TEMPLATE_INSERT` because `insert_template_dag` sets `role = template_id` and does not call the spawn-role validator (`ws.rs:104-173,200-293,797-831`; `personas.json:12-19,41-65`).
6. **A2A messages are write-only in practice.** `ask_parent`/`send_to_sibling` insert `agent_messages` and publish `AGENT_MESSAGE` (`execution.rs:2417-2487,2509-2609,721-742`), but no code path injects those rows into any model prompt/context; the only reader is the HTTP mailbox endpoint consumed by the UI (`store.rs:574-605`; `http.rs:519,674-702`; `AgentMessagesPanel.tsx:35-84`). `store.agent_messages_for_run` has exactly one consumer (`http.rs:684`) — verified by grep.
7. **Agents have no roster/discovery surface.** There is no roster endpoint or tool; routes expose messages/profiles/memory/projects only (`http.rs:518-526`). The task prompt contains only run prompt + role + depth (`provider_executor.rs:439-445`); a `send_to_sibling` needs an exact `TaskId` the model has no way to learn (`tool.rs:231-239`; `execution.rs:2526-2559`), so the verb is practically unusable except for children known from a prior `spawn_agent` result (`execution.rs:2684-2691`).
8. **Memory is a role-keyed global channel, not a hive/branch bank.** `agent_memory(agent_key, key, value, run_id, task_id, created_at, PK(agent_key,key))` (`db.rs:180-197`); reads are newest-8 by role (`store.rs:645-671`), writes are the completed task text (2,000 chars, marker filter) under `task:<id>` (`execution.rs:3017-3026,61-65`). There is no hive/branch column, no trust/provenance/expiry, and `HyperAgent` has no aggregate read; the root's context is seeded with `planner` memory (`execution.rs:880-884,1334`) but the root never executes.
9. **All execution is sequential: one active run, one task at a time, one provider call in flight.** `submit_prompt` refuses a second active run (`execution.rs:1032-1034`); leaves run in a sequential loop (`:1395-1426`), aggregators sequentially (`:1431-1485`), the spawn queue is drained sequentially (`:1500-1550`), and a model-spawned child is awaited inline inside the parent tool call (`:2668-2681`). The provider limiter allows 8 in flight (`hiverouter-provider/src/budget.rs:42-43`, acquired at `client.rs:380,569`), but the engine never offers more than one call at a time.
10. **The published scale blockers are all real and stack.** Hard caps 64 agents/8 fan-out/depth 3/40 calls (`caps.rs:8-15`; provider duplicates at `budget.rs:28-43`), planner prompt 1..8 nodes (`provider_executor.rs:25-29`) vs plan cap 40 (`planner.rs:9-10,44-56`), 400K-token/$0.50-per-run budgets (`budget.rs:24-31,62-81,235-284`), one SQLite connection behind a mutex (`db.rs:295-350`), 1,024 fan-out/256 replay events (`bus.rs:18-20`), per-fragment `REPLY_DELTA` publishing (`execution.rs:1764-1795`), per-node snapshots (`:1391-1408,1470,1522-1527`), and no scheduler/worker pool anywhere in `backend/crates/**/src` (grep `scheduler|join_all|FuturesUnordered|Semaphore` shows only the provider limiter). **50–150 concurrently running agents is impossible today; 300–600 tasks is short by 5–10× on multiple independent caps.** This corroborates the peer scale analysis `[READ-DOC docs/research/hmars/03-scale-concurrency.md:88-106,11-23]`.

---

## 1. What was read and how claims are labelled

Product source read at HEAD `bd575ba` (all paths below relative to repo root):

- `backend/crates/hiverouter-core/src/{execution.rs,context.rs,prompts.rs,planner.rs,caps.rs,config.rs,db.rs,store.rs,tools_bridge.rs,http.rs,ws.rs,provider_executor.rs,tool_loop.rs,bus.rs,graph.rs}`
- `backend/crates/hiverouter-tools/src/{tool.rs,executor.rs,approval.rs,side_effects.rs}`
- `backend/crates/hiverouter-provider/src/{budget.rs,client.rs}`
- `backend/crates/hiverouter-contract/src/{event.rs,command.rs}`
- `backend/crates/hiverouter-core/assets/prompts/{manifest.json,personas.json,*.prompty}`
- `web/src/{chat/ChatCore.tsx,flow/graph.ts,panels/AgentHierarchy.tsx,panels/AgentMessagesPanel.tsx,panels/AgentMemoryPanel.tsx,state/store.ts,state/strings.ts,api/contract.ts}`
- Peer HMARS docs `docs/research/hmars/01..06` (headings + the sections cited; `[READ-DOC]`) and `docs/provider/AGENT-ARCHITECTURE-TRUTH.md` (`[READ-DOC]`, itself dirty).
- `docs/CODEMAP/18-HMAS-RESEARCH.md` for the recorded implementation delta (`[READ-DOC]`).

No build/test/server/paid call was executed. Therefore every *runtime* statement is `UNVERIFIED`; every *code path* statement is `READ` (or `VERIFIED` when two independent sources agree). The HMARS target numbers (50–150 concurrently active agents, 300–600 tasks) are defined in `[READ-DOC 03-scale-concurrency.md:1,11-23,40-42]`, not in product source.

---

## 2. Actual agent tiers and role mapping

### 2.1 The enums

| Concept | Code | Facts |
|---|---|---|
| Runtime tier | `hiverouter-tools/src/tool.rs:66-100` | `AgentTier::{HyperAgent, SuperAgent, WorkerAgent, ToolAgent}`; labels `hyper_agent`, `super_agent`, `worker_agent`, `tool_agent`. |
| Persona tier | `core/src/prompts.rs:367-379` | `PersonaTier::{Hyper, Super, Worker, Tool}` — a **second, independent** tier enum stored on each `Persona` (`:381-404`). |
| Node kind (wire/UI) | `core/src/execution.rs:97-106`; `contract/src/event.rs:240` | `HyperAgent→"hyperagent"`, `SuperAgent→"superagent"`, `WorkerAgent→"worker"`, `ToolAgent→"tool"`; vocabulary is exactly those four. |
| UI kinds | `web/src/api/contract.ts:213`; `web/src/state/agentState.ts:126` | `hyperagent, superagent, worker, tool, error` — no `subagent`. |

### 2.2 The role→tier function is the real authority

`task_agent_tier(task)` (`execution.rs:151-163`):

| Role string (trimmed, lowercase) | Tier |
|---|---|
| `planner`, `hyper`, `hyperagent`, `hyper_agent`, `orchestrator` | `HyperAgent` |
| `super`, `superagent`, `super_agent`, `sub_hive`, `subhive` | `SuperAgent` |
| `tool`, `toolagent`, `tool_agent` | `ToolAgent` |
| `worker`, `workeragent`, `worker_agent` | `WorkerAgent` |
| `""` **and** `depth == 0` | `HyperAgent` |
| **anything else** (including `subagent`, `content-writer`, `data-analyst`, `code-reviewer`, `chatbot`) | `WorkerAgent` (`:161`) |

Consequences (`READ`+`INFERRED`):
- The mapping is fail-closed against capability escalation: an unrecognized role cannot become Super/Hyper/Tool.
- But it also **silently flattens every persona not named by the legacy vocabulary to Worker**, including `subagent` and all worker personas. The persona/template/allowlist may still apply (lookup by exact role string, §5), but the tier/kind/card classification is `worker`.
- `PersonaTier` is not what the engine consults. Grep for persona `tier` use finds only serialization/HTTP exposure (`http.rs:732,791`; `prompts.rs:389`) — the engine derives tier from the *role string*, not from `persona.tier`. That is a metadata/authority split (`INFERRED` risk).

### 2.3 Spawn authorization (capability cannot escalate)

`validate_spawn_role(parent_tier, requested_role)` (`execution.rs:108-142`):

| Parent tier | May spawn (names) | Never |
|---|---|---|
| HyperAgent | super/superagent/super_agent/sub_hive/subhive, worker/workeragent/worker_agent, agent, subagent/sub_agent | tool, arbitrary persona ids |
| SuperAgent | worker/workeragent/worker_agent, agent, subagent/sub_agent | super, hyper, tool, arbitrary persona ids |
| WorkerAgent | subagent, sub_agent **only** | super, hyper, tool, worker, persona ids |
| ToolAgent | none (`E_TOOL_NOT_ALLOWED`) | everything |

This is a real no-escalation rule (`READ`, and corroborated by peer analysis `[READ-DOC 02-hmars-patterns.md:248]`). **But** planned tasks are not passed through this function: `dispatch_inner` inserts plan nodes after `plan.validate()` and `caps.check_spawn()` only (`execution.rs:1337-1375`; `planner.rs:44-100` accepts any non-empty role at `:63-71`). A planner model (or a prompt-injected planner response) can therefore produce `role:"tool"` (ToolAgent permission set) or `role:"hyper"` (HyperAgent allowlist) plan nodes. Side-effecting tools still require approval, and caps still apply, but the *authorization model* is unenforced for planner-produced roles (`INFERRED` risk R1).

---

## 3. HyperAgent root vs the ChatCore UI

### 3.1 The root task

- `dispatch_inner` sets run `planning`, performs exactly one `executor.plan(...)` call (`execution.rs:1309-1319`), validates the plan, then inserts the root (`:1321-1335`): `role="planner"`, `parent_task_id=None`, `depth=0`, `state=Running`, `spawn_seq=1`.
- Root row, graph node, scoped context and `NODE_SPAWN` are created (`:1332-1335`); `publish_node_spawn` computes `kind = node_kind(task_agent_tier(task)) = "hyperagent"` (`:764-779`).
- The root is seeded with the `planner` persona's `allowed_tool_ids` (empty, `personas.json:3-10`) and the latest 8 `agent_memory` rows whose `agent_key == "planner"` (`execution.rs:856-886`), because `seed_task_context(&root, None)` is called at `:1334`.

### 3.2 The root never executes a model turn

Execution order over a plan is:
1. `plan.leaf_order()` — planned nodes that nobody depends on (`planner.rs:143-162`), looped sequentially (`execution.rs:1395-1426`);
2. `plan.aggregation_order()` — planned nodes that have children/dependents (`planner.rs:164-193`), looped sequentially (`execution.rs:1431-1485`);
3. the post-plan spawn queue, drained sequentially (`execution.rs:1500-1550`).

The root is not a `PlanNode` (it is inserted separately at `:1321-1335`), so `leaf_order`/`aggregation_order` never return it, and no `execute_task(root)` call exists. Therefore:
- The `HyperAgent` tier allowlist (`spawn_agent`, `report_result`, `finish` — `tool.rs:392-395`) is **never exercised by a model**.
- The planner-role memory loaded into the root context is **never rendered into a prompt** (`prompt_with_context` is only called by `execute_task_with_prompt`, `execution.rs:53-58,2747-2762`).
- The root is settled only at terminal reconciliation (`execution.rs:1216-1243`, called at `:1264-1268`) and contributes to the metrics counters (`:657-698`).

`VERIFIED` (two independent reads: root creation vs executor order + absence of root in planner orders).

### 3.3 The planning call is not a persona call

- `planning_prompt` renders the `planner` template (`planner.prompty`) if present and flattens its messages into text (`execution.rs:1552-1566`).
- `ProviderExecutorAdapter::plan` then builds `ChatMessage::system(PLANNER_SYSTEM)` + a user message wrapping that text (`provider_executor.rs:286-313`). `PLANNER_SYSTEM` itself defines the role/schema (`:25-29`).
- The adapter's `persona_prompts` map is consulted only by `system_prompt_for(task)` inside `execute` (`:82-87,328`), never by `plan`. The planner persona's `.prompty` system body (`planner.prompty:16-18`) is therefore **not** the planner system prompt; only its user/plan text is flattened into the user message (`INFERRED` from code paths; `READ` lines above).
- The root context (facts/memory) is not passed to `plan()` either.

### 3.4 ChatCore is the operator UI, not an agent

- `ChatCore` owns the fixed chat core + composer, submits only `SUBMIT_PROMPT`, renders `REPLY_DELTA` streaming (`web/src/chat/ChatCore.tsx:1-5,77-86,130-152,320-364`; `state/store.ts:1835-1847`).
- The aria label is literally `"HyperAgent hive chat core"` (`web/src/state/strings.ts:77`; section uses `STRINGS.chat.aria` at `ChatCore.tsx:179-180,275-276`).
- The canvas renders the root as a fixed anchor, not as a second card: `graph.ts:26-28` documents the invisible `hive-core-anchor`; `graph.ts:99` skips `kind==="hyperagent"` in layout reservations; `graph.ts:151` filters it out of `buildNodes`; `graph.ts:172` uses the anchor as the edge source for root children.
- The UI hierarchy view is built only from task parent links (`web/src/panels/AgentHierarchy.tsx:1-53`) — a derived visualization, not an agent directory or controller.
- `HyperAgentNode` exists as a React component (`nodes.tsx:245-261`), but the hyperagent task is filtered from the canvas node list (`graph.ts:151`), so in practice it represents the overlay/anchor rather than an executing card.

`READ`; no code path makes ChatCore send model calls other than `SUBMIT_PROMPT`/approval/file commands. **ChatCore is not a SuperAgent** (contradicts a plausible reading of the product's branding; matches the peer assessment `[READ-DOC docs/provider/AGENT-ARCHITECTURE-TRUTH.md:24]`).

---

## 4. SuperAgent and Subagent: capability, spawn policy, personas

### 4.1 SuperAgent

| Fact | Evidence |
|---|---|
| Persona id `superagent`, tier `super`, template `superagent`, allowed tools `spawn_agent, ask_parent, send_to_sibling, file_read, file_list, report_result, finish`, caps `max_children=8`, `max_depth=3` | `assets/prompts/personas.json:21-30` |
| System body: "sub-orchestrator of one branch under the HyperAgent… spawn the minimum useful worker agents… respect fan-out/depth… use ask_parent… report a concrete result" | `assets/prompts/superagent.prompty:16-18` |
| Tier allowlist: spawn/ask/sibling/report/finish/file_read/file_list (no file_write/shell) | `tool.rs:396-407` |
| Role `superagent`/`super` maps to `AgentTier::SuperAgent` | `execution.rs:151-158` |
| Spawn policy: Super may spawn only worker/agent/subagent | `execution.rs:130-133` |
| Per-persona caps are enforced only when the **parent's role string** equals the persona id: `persona_spawn_limits(&parent.role)` | `execution.rs:549-558,1083-1090` |
| Planned superagent nodes are inserted as `parent=node.parent or root`, depth from plan | `execution.rs:1337-1375` |
| Model-driven spawn from a running Super/Worker task is inline and blocking | `execution.rs:1682-1692,2618-2729` |
| No `sub_hive` card distinction on the canvas beyond `kind=superagent` | `contract/src/event.rs:240`; `contract.ts:213` |

What a SuperAgent can actually do today (`READ`+`INFERRED`):
- If the planner emits a `superagent` node, it gets the SuperAgent persona system prompt on its first model call (role lookup, §5), the `superagent` tool allowlist on first **and** continuation calls (tier specs, §8), and can call `spawn_agent` while executing. The child runs inline to a terminal state and its bounded result is returned as the tool result (`execution.rs:2655-2691`).
- Its `max_children`/`max_depth` are effective only when the parent role is exactly `superagent`; alias roles like `super`/`sub_hive` skip the persona-level check, though the global caps are identical today (8/3) so the practical effect is nil (`VERIFIED` by comparing values at `caps.rs:8-15` and `personas.json:28-29`).
- Planned SuperAgent hierarchy is **not produced by the planner prompt**: `PLANNER_SYSTEM`'s schema lists only `id/role/deps/depth`, never `parent` (`provider_executor.rs:25-29`), while `WireNode.parent` is optional (`:508-519`). Nodes without a parent attach to the root (`execution.rs:1346-1351`). So the planner's tree is flat by default; SuperAgent sub-trees only appear if the model invents the undocumented `parent` field or via Library/Template insertion/model spawning (`INFERRED` from the schema/parse code).

### 4.2 Subagent

| Fact | Evidence |
|---|---|
| Persona id `subagent`, tier `worker`, template `subagent`, allowed tools `ask_parent, send_to_sibling, file_read, file_list, report_result, finish` (no spawn, no write/shell) | `assets/prompts/personas.json:32-37` |
| System body: narrow worker for a parent; do only the assigned sub-task; ask_parent when blocked; report precise result/uncertainty | `assets/prompts/subagent.prompty:16-18` |
| No `Subagent` variant in `AgentTier`; `subagent` role is not in `task_agent_tier`, so it falls into the `_ => WorkerAgent` arm | `tool.rs:66-100`; `execution.rs:151-162` |
| Spawn policy: Worker may spawn subagent only; Super/Hyper may too | `execution.rs:113-142` |
| Spawn creates a real child row with `parent_task_id=caller`, `depth=parent+1`, role from args; caps enforced before insert | `execution.rs:1061-1111,2618-2729` |
| Child prompt = run prompt + `Child agent instruction (role "<role>"): <instruction>` | `execution.rs:2671-2679`; `provider_executor.rs:439-445` |
| Child result returns `{child_task_id, role, depth, state, result}` (bounded) | `execution.rs:2684-2691` |
| No distinct node kind/card/memory class | `contract/src/event.rs:240`; `web/src/api/contract.ts:213`; memory keyed by role only `db.rs:180-197` |

`READ`+`INFERRED`: a Subagent is a genuine capability-narrow worker with its own persona prompt and allowlist, but at the engine/UI/wire level it is a WorkerAgent. It cannot spawn (blocked both by its persona allowlist and by the Worker spawn rule), and its persona prompt is used only on its first model call (§5).

### 4.3 Spawn policy summary (who may create whom)

```
Hyper --spawn--> Super | Worker/Agent | Subagent
Super --spawn--> Worker/Agent | Subagent
Worker --spawn--> Subagent only
Tool  --spawn--> nobody
```
Enforced by `validate_spawn_role` for command/model spawning (`execution.rs:113-142,1061-1111,2618-2729`); **planned roles are not run through it** (`execution.rs:1337-1375`). Per-persona caps are additional tightening only when the parent role matches a persona id (`execution.rs:549-558,1083-1090`; validation requires non-zero and ≤ global caps at `prompts.rs:421-432`).

### 4.4 Persona inventory shipped (seven)

| id | persona tier | template | allowed tools | special caps |
|---|---|---|---|---|
| `planner` | hyper | planner | none | — |
| `superagent` | super | superagent | spawn_agent, ask_parent, send_to_sibling, file_read, file_list, report_result, finish | max_children 8, max_depth 3 |
| `subagent` | worker | subagent | ask_parent, send_to_sibling, file_read, file_list, report_result, finish | — |
| `content-writer` | worker | content-writer | file_read, file_list, ask_parent, send_to_sibling, report_result, finish | — |
| `data-analyst` | worker | data-analyst | same as content-writer | — |
| `code-reviewer` | worker | code-reviewer | same as content-writer | — |
| `chatbot` | worker | chatbot | ask_parent, send_to_sibling, report_result, finish | — |

Evidence: `assets/prompts/personas.json:3-66`; templates registered in `assets/prompts/manifest.json:3-45`; each `.prompty` has a `system:` block at line 16 and a `user:` block at line 19 (e.g. `superagent.prompty:16-19`).

**Reachability matrix (`READ`+`INFERRED`):**
- `superagent`: reachable if planner emits `role:"superagent"` (prompt explicitly permits it), or via `REWIRE`? no — via plan role; model spawn role `superagent` is refused by Worker/Super parents and only Hyper may spawn super (but no executing Hyper task exists); so effective path is **planner role** (and `SPAWN_SUB_HIVE` from a Hyper task, which never runs).
- `subagent`: reachable via `spawn_agent` from Worker/Super tasks; also planner could emit role.
- `planner`: root only; never executed; planner call does not use its persona prompt.
- `content-writer`/`data-analyst`/`code-reviewer`/`chatbot`: **not reachable via `SPAWN_SUB_HIVE`/`spawn_agent`** (spawn names rejected at `execution.rs:113-142`); reachable via `TEMPLATE_INSERT` because that path sets `role = template_id` and skips `validate_spawn_role` (`ws.rs:104-120,200-293,797-831`), or if the planner emits the exact persona id as a free-form role (`planner.rs:63-71` accepts any non-empty role; `PLANNER_SYSTEM` only *asks* for worker/superagent at `provider_executor.rs:28-29`).

---

## 5. Persona/template loading and whether prompts reach model calls

### 5.1 Loading

| Step | Code | Fact |
|---|---|---|
| Embedded manifest + personas + template files | `prompts.rs:38-44` | `EMBEDDED_MANIFEST`, `EMBEDDED_PERSONAS`, `EMBEDDED_TEMPLATE_FILES` are `include_str!` assets. |
| Template registry load | `prompts.rs:183` (`load_embedded`), `:251-263` (`get`/`template_ids`), `:265-314` (render); `http.rs:70-107` boot check | Missing/invalid assets become a required failed health check + `prompt_assets_error`, never silent. |
| Persona load + validation | `prompts.rs:454-465` (`load_personas_embedded` → `parse_personas`), `:406-434` (`Persona::validate`: non-empty id/display, refusal policy, template exists, caps bounds) | A persona pointing at a missing template refuses boot personas. |
| Persisted profile merge | `http.rs:137-184` (`merge_persisted_personas`); persistence table migration v9 `db.rs:218-232`; store CRUD `store.rs:749-783`; HTTP GET/import/DELETE `http.rs:519-525,704-801` | Imported profiles override embedded by id after the same validation; invalid rows are logged and ignored, embedded retained. |
| Persona system-prompt map | `http.rs:230-252` (`build_executor` → `adapter.with_persona_prompts(prompts)`), `:254-278` (`persona_system_prompts`: render template, keep the `System`-role message, key by persona id) | Only the system block is kept; template inputs are empty (`BTreeMap::new()`), so `{{ task }}`/sample values are irrelevant here. |

### 5.2 Where the persona prompt actually reaches a model call

| Call | System prompt used | Evidence |
|---|---|---|
| Planning call | `PLANNER_SYSTEM` constant; planner template flattened into the user message by `planning_prompt` | `provider_executor.rs:286-313`; `execution.rs:1552-1566` |
| Leaf/completion call | Adapter `system_prompt_for(task)` = `persona_prompts.get(task.role).unwrap_or(TASK_SYSTEM)` | `provider_executor.rs:82-87,315-348` |
| Tool-turn continuation | **`TASK_SYSTEM`**, not the persona prompt; replay transcript + tool results | `provider_executor.rs:357-424` (messages start at `:369-372`) |
| Child spawn | Same leaf path; child task role drives `system_prompt_for` | `execution.rs:2671-2691` |
| Aggregate turn | Same leaf path with an aggregate prompt built from child `task_result`s | `execution.rs:1427-1485,2747-2762` |

**Consequences (`READ`+`VERIFIED` cross-check):**
- Persona identity is applied **only for the first provider call of a task**. As soon as the model calls a tool and the engine continues the loop, the continuation request uses the generic `TASK_SYSTEM`, so role-specific instructions are lost exactly when the agent is acting with tools (the highest-stakes step). This is a concrete integration gap (`R2`).
- The planner persona is never used (`plan` ignores `persona_prompts`), and the root/planner memory is never in the prompt (§3.2/§6.4).
- Persona lookup is by exact `task.role` string. A planned `worker` node has no `worker` persona in `personas.json`, so it falls back to `TASK_SYSTEM`; a `superagent` plan node matches; a Library template node with role `content-writer` matches only if created by `TEMPLATE_INSERT` (or planner emits that string).
- Template `.prompty` `user:` blocks are not sent as user messages; the task prompt is independently built as `"Original user prompt: … Assigned role … Spawn depth …"` (`provider_executor.rs:439-445`). So persona templates contribute only their system text, not their user template.
- `TASK_SYSTEM` says "You are a HiveMind agent" (`provider_executor.rs:31-33`) — the generic fallback.

---

## 6. Per-task context, agent memory, inheritance, per-hive memory, HyperAgent access

### 6.1 ScopedContext — a per-task snapshot (not durable memory)

`ScopedContext` (`context.rs:19-29`): `parent_task_id: Option<TaskId>`, `facts: BTreeMap<String,String>`, `allowed_tool_ids: Vec<String>`, `output_schema_id: Option<String>`.
- Fact keys: `parent:<parent id>:<key>` for scoped facts (`FACT_PREFIX` `context.rs:16`; `fact_key :109`), plain keys for root facts.
- `ContextStore` is an in-memory `Mutex<HashMap<TaskId, ScopedContext>>` (`context.rs:190-257`); it is **not persisted**. A crash/restart loses all per-task contexts; the durable records are messages, task results, and agent_memory.
- `prompt_block(max_chars)` renders a bounded block (`context.rs:66-95`): skips keys that look secret-shaped (`is_sensitive_fact_key` `:113-118`), truncates with a marker, and prefixes `"Scoped facts for this branch (authoritative):"` (`:71`).
- The prompt integration is `prompt_with_context(prompt, context)` (`execution.rs:53-58`), called by `execute_task_with_prompt` (`:2747-2762`) and hence by leaf, aggregate, and inline-spawn paths. The block is capped at 4,000 chars (`:54`).

### 6.2 Creation and inheritance

`seed_task_context(task, parent)` (`execution.rs:856-886`):
1. `ScopedContext::new(parent)`.
2. If a parent exists, copy **the parent's entire `facts` map** into the child (`:865-870`) — a snapshot, not a live view.
3. If the task's `role` matches a loaded persona, copy persona `allowed_tool_ids` and `output_schema_id` (`:871-879`).
4. Load the newest **8** `agent_memory` rows for `agent_key = task.role` and insert each as a fact `memory:<key>` (`:880-884`). Read errors are swallowed by `if let Ok(...)` — a DB failure is indistinguishable from "no memory" (`R3`).

Call sites: planned nodes `execution.rs:1372`, root `:1334`, command spans `:1106`, template insert via `seed_template_contexts` (`execution.rs:849-854`, `ws.rs:814`), model-spawn `:1106`.
Rewire (`ws.rs:650-736`) computes the previous parent before mutating the graph, then calls `apply_rewire_context` → `ContextStore::switch_on_rewire` (`context.rs:225-244`), which uses `switch_context` (`context.rs:138-188`): clears only the old parent's prefixed facts, keeps root facts and `memory:*`, injects the new parent's facts, and keeps the task's own allowed tool ids/output schema. Graph mutation and context switch are two writes; if the context switch fails after the graph write, the command publishes an error instead of ACKing (ws `:707-719`) — a loud partial-failure path.

### 6.3 agent_memory — durable role-keyed channel

Schema migration v7 (`db.rs:180-197`):
```
agent_memory(agent_key TEXT NOT NULL, key TEXT NOT NULL, value TEXT NOT NULL,
             run_id TEXT NOT NULL, task_id TEXT NOT NULL, created_at TEXT NOT NULL,
             PRIMARY KEY(agent_key, key))
+ index on agent_key
```
- `record_agent_memory(agent_key, key, value, run_id, task_id)` upserts on `(agent_key,key)` (`store.rs:608-643`).
- `agent_memory(agent_key, limit)` returns newest rows (`created_at DESC, rowid DESC LIMIT`) (`store.rs:645-671`).
- Write path: on task completion, if the final text is non-empty and passes `memory_text_is_safe` (only four substring markers: `DEEPSEEK_API_KEY`, `sk-`, `Bearer `, `Authorization:` — `execution.rs:61-65`), the first 2,000 chars are stored with `agent_key = task.role`, key `task:<task_id>` (`execution.rs:3000-3026`).
- Read path: only at task creation, newest 8 for the role (`execution.rs:880-884`); the HTTP UI lists ≤8 per persona (`http.rs:804-822`) and can clear one role globally (`http.rs:824-840`; UI `AgentMemoryPanel.tsx:26-84`).
- There is **no run scope, no hive/branch scope, no trust/provenance/expiry, no promotion, no semantic ranking, no audit of what was read**. `run_id`/`task_id` are provenance columns on an upsert row, not part of the key: two runs of the same role share the same eight-row channel (`READ`, corroborated by peer `[READ-DOC 04-memory-architecture.md:133-145,160-191]`).

### 6.4 Per-hive memory and HyperAgent access

- **No per-hive/branch memory exists.** No table/column/type for hive or branch scope; grep of migrations v1–v10 shows `agent_messages`, `agent_memory`, `task_results`, `agent_profiles`, `projects`, none hive-scoped (`db.rs:32-249`). `ScopedContext.facts` is the only branch-scoped container and is in-memory only.
- **HyperAgent access:** the root context is seeded with the `planner` role's newest 8 memory rows (`execution.rs:1334,880-884`) and the `planner` persona allowlist (empty), but the root never executes and the planner call does not read context (§3.2/§3.3). Therefore:
  - there is **no HyperAgent read policy**, no aggregate memory, no digest; the only memory access at the Hyper layer is the unused root context;
  - the operator can read all role rows through the UI, but no agent can.
- **Task results are the actual upward data plane:** one row per task (`task_results`, migration v5 `db.rs:142-158`; write at `execution.rs:3015`; read/aggregate at `:1447-1466`; inline spawn returns the child result directly `:2681-2692`). Aggregation reads child results via parent/deps edges, never the run-wide chat transcript (`execution.rs:1427-1465`, comment at `:1428-1430`). This is a correct branch-isolation property (`READ`).
- Memory facts are presented as authoritative: `prompt_block` header says "authoritative" (`context.rs:71`) and `memory:*` entries are inserted into the same fact map as parent facts (`execution.rs:880-884`). Model-generated text can therefore be rendered as authoritative context in later runs (`INFERRED` prompt-injection/poisoning risk; peer research flags the same at `[READ-DOC 04-memory-architecture.md:173-177]`).

---

## 7. A2A: ask_parent, send_to_sibling, agent_messages, roster/discovery, buses

### 7.1 Tools and permission classes

Tool ids/args: `spawn_agent`, `ask_parent`, `send_to_sibling`, `report_result`, `finish`, file tools (`tool.rs:108-123`; arg structs `:210-257`).
Contracts (`tool.rs:332-389`): `spawn_agent` = Execute; `ask_parent` = Read; `send_to_sibling` = Mutate ("cross-branch targets require operator approval"); `report_result` = Read; `finish` = Read; file read/list = Read; file_write = Mutate; shell_exec = Execute.
Tier allowlists (`tool.rs:391-430`): Hyper = spawn/report/finish; Super = spawn/ask/sibling/report/finish/read/list; Worker = all nine; Tool = file read/list/write/shell.

### 7.2 ask_parent

`execute_ask_parent` (`execution.rs:2417-2487`):
- Deserializes `AskParentArgs{text}`.
- Requires `task.parent_task_id`; refuses `E_TASK_NOT_FOUND` when the caller has no parent (`:2433-2438`).
- Text must be non-empty and ≤ `MAX_AGENT_MESSAGE_CHARS = 2_000` (`:67-68`; enforced `:2440-2447`).
- Per-run message budget `MAX_AGENT_MESSAGES_PER_RUN = 32` (`:69-70`; check `:2448-2464`), refusal `E_CALL_LIMIT`.
- Inserts `agent_messages(run, from, to=parent, kind="ask_parent", text)` and publishes `AGENT_MESSAGE`; returns `{message_id, to_task_id, kind}` (`:2465-2486`; publish `:721-742`).
- **No reply path, no waiting, no inbox, no prompt injection.** The parent may already be finished, paused in a tool call, or currently executing a different turn; nothing reads the message back into any model conversation (`READ`; see §7.4).

### 7.3 send_to_sibling

`execute_send_to_sibling` (`execution.rs:2509-2609`):
- Args `SendToSiblingArgs{target_task_id, text}`; target must parse as `TaskId`, not self, and exist.
- `target.run_id == task.run_id`; same parent **or** `allow_cross_parent=true`; otherwise `E_TOOL_NOT_ALLOWED` (`:2550-2559`).
- Same-parent calls are handled immediately (`execution.rs:1719-1720`) **without approval even though the contract class is Mutate**.
- Cross-branch calls are detected by `is_cross_branch_message` (`:2493-2507`) and go through `suspend_for_approval` when both `tools` and `tool_workspace` are wired (`:1695-1718`); without them the call is refused `E_TOOL_NOT_ALLOWED` and nothing is written. The text is bounded 2,000 and counted against the same per-run 32-message budget (`:2561-2586`); on success it inserts + publishes `AGENT_MESSAGE` (`:2587-2608`).
- **No delivery:** the target's next prompt does not include the message.

### 7.4 agent_messages storage/consumers

- Schema v6 (`db.rs:160-178`): `(id, run_id, from_task_id, to_task_id, kind, text, created_at)` + run index. No `in_reply_to`, no status/open/answered/expired, no read state.
- Store: `record_agent_message` (`store.rs:522-554`), `agent_message_count` (`:556-572`), `agent_messages_for_run` (`:574-605`).
- Consumers (grep-verified): only `http.rs:684` (`GET /api/agents/runs/{run_id}/messages`, route at `http.rs:519`) and the UI panel that fetches it (`web/src/panels/AgentMessagesPanel.tsx:35-84`, rendering `kind · from → to` + text). The event also bumps a store version (`state/store.ts:1494-1496`).
- **No consumer in `execution.rs`** — the engine never reads `agent_messages_for_run` or individual messages, so no model sees them. `ask_parent`/`send_to_sibling` are therefore durable/observable mailbox writes, not a communication mechanism between agents (`VERIFIED`: code path + grep single consumer).
- Event payload/name: `AgentMessagePayload` (`contract/src/event.rs:607-622`), `EventName::AgentMessage` (`:66-67,96,126`), event variant (`:753-754`), publish helper (`execution.rs:721-742`).

### 7.5 Roster/discovery

- No roster endpoint, event, command, or tool. Server routes for agents are only messages/profiles/memory (`http.rs:518-526`); there is no `/roster`, `/agents`, or equivalent (grep across `http.rs`/`ws.rs`/`tool.rs`).
- The model's only identity information is: `task.role`, `task.depth`, and the original prompt (`provider_executor.rs:439-445`); for `spawn_agent`, the child id is returned after the child completes (`execution.rs:2684-2691`); `ask_parent` needs no id; `send_to_sibling` needs an exact target id that is not exposed anywhere to the caller (`tool.rs:231-239`; `execution.rs:2526-2559`).
- The UI has a topology view, but it is derived from parent links, not served to agents (`web/src/panels/AgentHierarchy.tsx:1-53`; graph nodes/edges from store tasks).
- Therefore **agent-to-agent target discovery is absent**; "sibling messaging" requires out-of-band knowledge of a UUID. This matches the peer mapping: escalate/notify are "write-only"/"notify" (`[READ-DOC 02-hmars-patterns.md:256-258]`).

### 7.6 Buses

| Bus/plane | Code | Fact |
|---|---|---|
| Server event bus | `core/src/bus.rs:1-20,84-172` | Global monotonic `seq`, broadcast fan-out 1,024, replay 256; sessions subscribe in `ws.rs:401-402`; lag → `RESYNC_REQUIRED` (`ws.rs:444-455`). |
| Graph authority | `core/src/graph.rs:1-49,73-201` | `petgraph::DiGraph` parent→child; cycle-checked rewire; `TaskNode{parent_task_id,state,rev}`. Mutations only here (`add_task`/`rewire`/`set_state`), no per-agent bus. |
| A2A "bus" | `agent_messages` DB + `AGENT_MESSAGE` event | Write-only mailbox; no delivery/inbox/subscription for agents (§7.4). |
| Provider transport | `hiverouter-provider` client (SSE/HTTP) | Not an agent bus; one request per call. |

`READ`; the event bus is explicitly a UI/transport bus, not an agent communication plane (corroborated by `[READ-DOC 02-hmars-patterns.md:264,274-276]`).

### 7.7 A2A constraints & risks

Constraints: parent-only scope for `ask_parent`; same-run + same-parent (or approved cross-branch) for `send_to_sibling`; text 2,000 chars; 32 messages/run; no reply/status; messages survive restart; secret filtering absent on message text (only memory has a marker filter). Risks:
- **No delivery → claims of collaboration are false today** (`RISK A2A-1`).
- An agent can spam/poison the operator mailbox with 32 messages/run of model text, unredacted (`RISK A2A-2`; no secret-marker check in `execute_ask_parent`/`execute_send_to_sibling`).
- Cross-branch target approval exists, but the target/task id is unobtainable; approval UI can approve a message whose text was model-generated and never reviewed by the target (`RISK A2A-3`).
- `ask_parent` is permission-class **Read** yet writes DB/event (`tool.rs:339-344` vs `execution.rs:2465-2470`) — a permission-class semantic mismatch (`RISK A2A-4`; see §8.4).

---

## 8. Tool registry, allowlists, approvals, and first-turn tools

### 8.1 Registry and validation

- One `ToolRegistry::standard()` with a `BTreeMap<ToolId, ToolContract>` and per-tier allowlists (`tool.rs:304-430`).
- Contract fields: id, display name, description, permission class, JSON Schema (`tool.rs:153-187`); schemas are schemars-derived with `additionalProperties:false` (`:189-207`).
- `resolve_call(tier, tools_name, args)` resolves id → tier grant → schema validation (`tool.rs:466-505`).
- Tier allowlists (`tool.rs:391-430`) summarized in §7.1.
- Tool-loop budget is separate and persisted: max 4 turns, 16 calls/run, 64 KiB result, 8 approval waits (`tool_loop.rs:14-21`); plan validation also caps turn/call atomically (`:147-230`).
- Persona allowlists intersect with tier allowlists via `ScopedContext.allowed_tool_ids` in `authorize_tool_call` (`execution.rs:984-1009`); empty list = no narrowing (`:999-1009`). Note: persona ids are validated against templates/caps but **`allowed_tool_ids` values are not validated against the tool registry** (`Persona::validate` at `prompts.rs:411-434` checks id/display/policy/template/caps only; peer research flags the same at `[READ-DOC 02-hmars-patterns.md:404-407]`). A typo in a persona tool list simply makes that tool unusable (no error at load) (`INFERRED`).

### 8.2 First-turn tools (and the later continuation)

- `ProviderExecutorAdapter::execute` builds tool specs for `task_agent_tier(task)` and adds `.with_tools(tools).with_tool_choice(ToolChoice::Auto)` when non-empty (`provider_executor.rs:315-348`, comment at `:334-339` documents that the first request previously omitted tools).
- The continuation also offers tier specs (`provider_executor.rs:409-415`).
- The planning call offers no tools (`provider_executor.rs:286-313`), and the Hyper root never executes, so the Hyper allowlist is unreachable through a model.
- Tools are offered per **tier**, not per persona (`tool_specs_for_tier`, `tool_loop.rs:206-216`); the persona allowlist only gates calls after the model makes them (`authorize_tool_call`). So a persona with a narrow allowlist still *sees* every tool its tier allows (and can attempt them; they are then refused). `INFERRED` privacy/minimalism gap.

### 8.3 Engine routing and special-cased tools

Inside `execute_tool_turn` (`execution.rs:1576-1762`): after plan/tier/persona authorization (`:1668-1682`):
- `spawn_agent` → `execute_spawn_agent` (`:1684-1692`, `:2618-2729`). **No approval** even though the contract class is Execute; the comment says spawning is loop-owned, not a workspace side effect (`:1685-1690`).
- `ask_parent` → `execute_ask_parent` (`:1693-1694`).
- `send_to_sibling` cross-branch → approval suspension (`:1695-1718`); same-parent → immediate (`:1719-1720`).
- Any other tool with `requires_approval` and a wired bridge+jail → approval suspension (`:1721-1734`; counted before wait `:1727-1730`); otherwise → `execute_tool_plan` (`:1735-1736`), which permits only Read-class tools and executes file_read/file_list through the jail, refusing all side effects (`:1844-1885`).
- Successful `finish` terminates the task with the verdict text without another provider round-trip (`:2864-2884`); successful `report_result` terminates with the summary and labels `success=false` (`:2885-2915`); both are loop-owned semantics, not model tools that write external state.

### 8.4 Approvals

- Config: `HIVEROUTER_SIDE_EFFECTS` env, parsed with truthy semantics; **default enabled** (`config.rs:195,355-371`); carried into the engine's `ToolsBridge` (`http.rs:206-226`; `tools_bridge.rs:33-47,116-117`).
- Approval-gated execution: `ToolsBridge::execute_approved` refuses when the kill switch is off **without consuming the approval** (`tools_bridge.rs:196-247`, check `:217-224`), otherwise preflight (read-only) → `verify_tool_grant` (single-use CAS) → commit exactly one effect (`:226-245`). Grant verification is `verify_tool_grant` (`:169-188`); approval suspension is engine-side (§8.3). This is the 1.2 side-effect path.
- `file_write`/`shell_exec` are Mutate/Execute and require approval (`tool.rs:375-386`); their executor is `execute_approved` → `tools/side_effects` (only reachable when bridge+jail wired).
- `spawn_agent`/`send_to_sibling` cross-branch/report/finish use the loop-owned/approval paths above rather than the side-effect executor.
- Permission-class mismatches (`READ`+`INFERRED`, `RISK PC-1`): `ask_parent`/`report_result`/`finish` are **Read** but perform DB writes/terminal state transitions (`tool.rs:339-361`; `execution.rs:2465-2470,2864-2915`); `send_to_sibling` is **Mutate** but the same-parent path skips approval (`execution.rs:1719-1720`); `spawn_agent` is **Execute** but is exempt from approval (`execution.rs:1682-1692`). If the permission class is meant to describe *effect risk*, these are inconsistent; the code comments explain the intended policy, but the contract class alone is not a reliable gate. Peer research independently recommends making the delegate/notify semantics explicit (`[READ-DOC 02-hmars-patterns.md:291-292,349-380]`).
- `tool_loop::MAX_APPROVAL_WAITS_PER_RUN = 8` (`tool_loop.rs:21`) bounds approval waits per run (enforced in engine approval path).

---

## 9. Caps, scheduler, concurrency — and what blocks 50–150 or 300–600

### 9.1 Hard caps and where they are enforced

| Cap | Value | Where enforced |
|---|---|---|
| Spawn depth | 3 (`Hyper→Super→Worker`) | `caps.rs:8-9`; `Caps::check_spawn :49-60`; checked at plan insert `execution.rs:1358`, command spawn `:1091`, template insert `ws.rs:263-271`; planner depth sanity `provider_executor.rs:541-543`; provider duplicate `budget.rs:33-37,438-490` |
| Fan-out | 8 per parent | `caps.rs:10-11`; per-parent siblings at `execution.rs:1075-1080,1352-1358`; provider `budget.rs:36-37` |
| Agents/run | 64 | `caps.rs:12-13`; `check_spawn :56-58`; provider `budget.rs:39-40` |
| Provider calls/run | 40 attempts incl retries | `budget.rs:30-31`; enforced at `preflight :243-251`; engine's `caps.check_calls(1)` is a **stateless compare** (`caps.rs:62-71`) called with literal `1` at `execution.rs:1313,2771`, so it never accumulates a count (`READ`; peer scale doc flags the same bound as `CALL_CAP`) |
| Cost cap | $0.50/run | `budget.rs:24-25,252-261` |
| Combined tokens | 400,000/run | `budget.rs:27-28,262-280` |
| Planner nodes | plan validator 40 (`planner.rs:9-10,44-56`); **prompt asks 1..8** (`provider_executor.rs:25-29`); planner max output tokens default 16,384 (`config.rs:18,64`) | `Plan::validate` rejects >40 with `E_CALL_LIMIT`; the planner's own instruction is smaller than the validator; a single 16K-token plan cannot enumerate 300–600 nodes (`[READ-DOC 03-scale-concurrency.md:19,104]`) |
| Tool loop | 4 turns, 16 calls per task/run budget, 8 approval waits, 64 KiB result | `tool_loop.rs:14-21`; persisted counters checked in `execute_tool_turn` (`execution.rs:1590-1640,1748-1756`) |
| Per-persona caps | max_children/max_depth ≤ global | `prompts.rs:421-432`; `execution.rs:549-558,1083-1090` |

### 9.2 Scheduler/concurrency reality

- **One active run per server**: `submit_prompt` refuses when `latest_active_run()` or `active_run()` is set (`execution.rs:1032-1034`), records active (`:1038`) and spawns a single Tokio task for dispatch (`:1048-1052`). There is no server-level queue/admission component; a second run is refused with `E_RUN_ACTIVE`.
- **One task at a time inside a run**: leaves loop sequentially (`:1395-1426`); aggregators sequentially deepest-first (`:1431-1485`); post-plan spawn queue drained sequentially (`:1500-1550`); a model-spawned child is awaited inline inside the parent's tool call (`:2668-2681`). There is no `join_all`/`FuturesUnordered`/task-per-agent pool in `backend/crates/**/src` (grep this session; peer scale doc also records one concurrency slot `[READ-DOC 03-scale-concurrency.md:88-106]`).
- **One provider call in flight per run**: the provider client has a process-wide `InFlightLimiter` of 8 (`budget.rs:42-43,371-431`; `client.rs:54,65`), acquired per non-streaming and streaming call (`client.rs:380,569`). The engine's serialization means the limiter is almost never the active bottleneck today; at a future parallel scheduler it becomes the first local ceiling (8 vs 150 target, as the peer doc computes at `[READ-DOC 03-scale-concurrency.md:94]`).
- **DB serialization**: `Database { path, connection: Mutex<Connection> }` (`db.rs:295-297`), configured `busy_timeout=5000`, WAL, synchronous=NORMAL (`:338-345`), locked for every read/write (`:348-350`). One connection means no reader concurrency even though WAL would allow it; each call blocks the calling async task.
- **Event fan-out**: `FANOUT_CAPACITY=1024`, `REPLAY_CAPACITY=256` (`bus.rs:18-20`); one `REPLY_DELTA` event per non-empty fragment (`execution.rs:1764-1795`); sessions subscribe and lagging triggers `RESYNC_REQUIRED` (`ws.rs:401-455`).
- **Snapshots**: committed after every executed leaf (`execution.rs:1402-1408`), aggregator (`:1470`), and spawned task (`:1522-1527`), plus at start and terminal (`:1391-1393,1302`); each snapshot is graph + revision (`execution.rs:1189-1214`), so write amplification scales with nodes.
- **Metrics**: `publish_run_metrics` scans all tasks on every node spawn/state (`execution.rs:657-698`; called from `:761,778`), O(tasks) per event; at 600 tasks this is repeated global work.
- **Task state lock**: `task_state_lock` serializes state read/check/write (`execution.rs:429-431,916-940`); per-task cancellation tokens (`:427-428,781-820`); run lifecycle lock for terminal write (`:940-955,1272-1299`).
- **UI**: canvas builds nodes for every task (`web/src/flow/graph.ts:143-166`) and passes them to React Flow without visible-element culling in this read (peer scale doc records no `onlyRenderVisibleElements` at `[READ-DOC 03-scale-concurrency.md:103]`). Browser rendering is a scaling risk (`INFERRED`).

### 9.3 What blocks the target numbers

**50–150 concurrently active agents (one in-flight model call/tool each):**
1. **Single active run** (`execution.rs:1032-1034`) — only one HyperAgent run exists.
2. **Sequential execution** (`execution.rs:1395-1550`, `:2668-2681`) — exactly one task runs at a time; "concurrently active" is 1, not 150.
3. **Call/token/cost caps** — 40 attempts/run (`budget.rs:30-31`), 400K tokens, $0.50 (`:24-28`): even 50 one-call agents need 50 calls and >50×400K? no, 50 calls would pass call cap 40? no — 50 > 40, so a 50-agent run is refused at the 41st call. The cap is a hard blocker before provider concurrency.
4. **DB single mutex** (`db.rs:295-350`) — every spawn/state/tool/result write serializes.
5. **Event/delta fan-out** — 1,024 ring, per-fragment events; 150×40 tok/s ≈ 6,000 events/s → 171 ms live/43 ms replay (peer arithmetic `[READ-DOC 03-scale-concurrency.md:253-266]`), forcing resync storms.
6. **Provider limiter 8** (`budget.rs:42-43`) — would bind first at a parallel scheduler.
7. **No scheduler/queue/backpressure** — no admission component exists; a second active run is refused, no per-run fairness/queue depth.

**300–600 tasks under a HyperAgent:**
1. `MAX_AGENTS=64` refuses task 65 with `E_AGENT_LIMIT` (`caps.rs:12-13,56-58`; `execution.rs:1091,1358`).
2. `MAX_FANOUT=8` and depth 3 cap structural nodes (`caps.rs:8-15`); even a full tree is far below 300–600.
3. Planner prompt asks for 1..8 nodes (`provider_executor.rs:25-29`), while the validator allows ≤40 (`planner.rs:9-10`); a single JSON planner call cannot enumerate 300–600 nodes (`[READ-DOC 03-scale-concurrency.md:19,65-160]`).
4. `CALL_CAP=40` / token 400K / $0.50 (`budget.rs:24-31`) refuses hundreds of agent calls before spending matters.
5. Single run/sequential execution/DB/event limits as above; a 600-task sweep needs hierarchical planning plus a real scheduler, not bigger constants (`[READ-DOC 03-scale-concurrency.md:11-23,138-149]`).
6. Per-node snapshots + O(n) metrics + full graph render add write/CPU/UI pressure (`execution.rs:657-698,1391-1527`; `graph.ts:143-166`).

**What is possible today (INFERRED from caps+serial path):** a single run whose plan is typically ≤8 nodes (prompt) and whose total provider attempts stay under 40, executed strictly serially; a few spawned children/subagents inline if the call budget allows. The 64-agent cap is effectively unreachable under the 40-call cap and the sequential scheduler. Runtime confirmation is `UNVERIFIED` (no run in this pass).

---

## 10. Exact gaps, risks, and proposed seams

### 10.1 Gaps (code-level)

| # | Gap | Evidence | Consequence |
|---|---|---|---|
| G1 | Subagent is not a tier/card/wire kind; all unrecognized roles including `subagent` map to Worker | `tool.rs:66-100`; `execution.rs:151-163`; `event.rs:240`; `contract.ts:213` | No Subagent card, no dedicated memory scope, no distinct metrics; UI cannot distinguish a subagent from a worker. |
| G2 | Persona prompts only on the first model call; continuation uses `TASK_SYSTEM` | `provider_executor.rs:357-372` vs `:315-348` | Role identity/instructions are lost precisely when tools run. |
| G3 | Planner persona/memory not used; planner call is `PLANNER_SYSTEM` + flattened template text | `provider_executor.rs:286-313`; `execution.rs:1552-1566` | No "HyperAgent personality"/learned planning from `planner` memory; template system body dead for plan. |
| G4 | Planner output schema omits `parent`; planned nodes default to root | `provider_executor.rs:25-29,508-519`; `execution.rs:1346-1351` | Planning produces a flat tree in practice; nested SuperAgent branches require undocumented model behavior or another command. |
| G5 | Four worker personas unreachable via spawn policy (only Template Insert / free-form planner role) | `execution.rs:113-142`; `ws.rs:200-293,797-831`; `personas.json:12-19,41-65` | Persona catalog is partly decorative; the spawn policy and persona registry disagree. |
| G6 | A2A messages are stored/evented but never delivered to a recipient's context/turn | `store.rs:574-605`; only consumer `http.rs:684`; `execution.rs` reads none | `ask_parent` cannot get an answer; `send_to_sibling` cannot inform a peer; no reply/correlation/status. |
| G7 | No roster/discovery: no endpoint/tool/prompt field with task ids/parents/siblings | `http.rs:518-526`; `provider_executor.rs:439-445`; `tool.rs:231-239` | Agents cannot discover valid sibling targets; `send_to_sibling` is unusable without out-of-band UUIDs. |
| G8 | Memory is role-global, cross-run, no hive/branch/trust/expiry; HyperAgent aggregate absent | `db.rs:180-197`; `store.rs:645-671`; `execution.rs:880-884,3017-3026` | Role bleed across runs/hives; poisoning; no forgetting; no orchestrator digest. |
| G9 | Context is in-memory only; rewire doesn't refresh a running task's prompt; read errors swallowed | `context.rs:190-257`; `execution.rs:880`; `ws.rs:650-736`; `execution.rs:2747-2762` | Crash loses context; rewire semantics diverge from the model's current prompt; DB outages invisible. |
| G10 | Persona `tier` and `allowed_tool_ids` are not authority; tier comes from role string; tool ids unvalidated | `execution.rs:984-1009`; `prompts.rs:389,411-434`; `http.rs:732,791` | Profile metadata can disagree with runtime capability; typo'd tool ids silently unusable. |
| G11 | Planned roles are not validated against spawn policy/registry | `execution.rs:1337-1375`; `planner.rs:63-71` | Planner/prompt injection can create ToolAgent/HyperAgent-class tasks (approval still gates effects). |
| G12 | Permission classes are inconsistent with actual effects | `tool.rs:339-361` vs `execution.rs:1682-1692,1719-1720,2465-2470,2864-2915` | Contract class is not a reliable authorization predicate; audit/summary tools mislabel risk. |
| G13 | `caps.check_calls(1)` is stateless; engine does not track calls | `caps.rs:62-71`; `execution.rs:1313,2771` | The engine-side call cap is inert; only provider `RunBudget` enforces the real cap. |
| G14 | No scheduler/admission/queue; single run; sequential dispatch | `execution.rs:1032-1052,1395-1550,2668-2681` | Cannot use available concurrency; one stuck call stalls the only run. |
| G15 | DB single connection mutex; per-node snapshots; O(n) metrics | `db.rs:295-350`; `execution.rs:657-698,1391-1527` | Serialized I/O and write amplification at scale. |
| G16 | Event bus is a single per-fragment UI stream with 1,024/256 bounds | `bus.rs:18-20`; `execution.rs:1764-1795`; `ws.rs:401-455` | Resync storms at high event rates; no agent plane (which is correct — do not turn it into one). |
| G17 | Template DAG catalog is single-node per template; comment claims Library ids lack prompt assets while manifest now includes them | `ws.rs:122-173` vs `manifest.json:3-45` | Multi-node library DAGs do not exist; comment is stale (`READ` doc drift). |
| G18 | No per-branch/per-hive memory or promotion API; no delivery/correlation fields on messages | §6/§7 | HMARS scope ladder unimplemented; A2A status protocol absent. |

### 10.2 Risks (ranked)

1. **R1 — Planner output is untrusted and only weakly authorized.** Role strings from the model control tier/allowlist; plan validation checks structure but not role vocabulary/persona existence (`planner.rs:63-71`; `execution.rs:151-163,1337-1375`). A malicious/garbled planner can create ToolAgent (write/shell with approval) or HyperAgent (spawn) tasks. Mitigation: validate planned roles against the persona registry and a frozen role vocabulary, and require parent/tier consistency.
2. **R2 — Tool-turn persona loss** (G2): the model is told "generic HiveMind agent" after its first tool round; role policies (e.g. Subagent "do not expand scope") stop applying in practice.
3. **R3 — Memory poisoning/role bleed** (G8): model output becomes durable memory automatically, role-global across runs; the prompt header calls facts "authoritative" (`context.rs:71`). A prompt-injected task can seed instructions read by later runs of the same role. Peer research documents the same failure modes at length (`[READ-DOC 04-memory-architecture.md:160-191]`).
4. **R4 — A2A claim inflation** (G6/G7): any feature claim that agents "coordinate via ask_parent/send_to_sibling" is false; they write an operator mailbox. Peer mapping calls escalate "write-only" (`[READ-DOC 02-hmars-patterns.md:256-258]`).
5. **R5 — Scaling claim inflation** (G13–G16): 50–150 running /300–600 tasks are impossible; language choice is irrelevant. Peer doc's verdict: provider is not the blocker; local ceilings are (`[READ-DOC 03-scale-concurrency.md:13-23,88-106]`).
6. **R6 — Permission-class mismatch** (G12/R1): approvals are correct for workspace side effects, but spawn/sibling/ask/report/finish rely on special-case routing, not the class; audits of "Mutate/Execute always require approval" would be wrong.
7. **R7 — Secret/prompt-injection exposure in memory/messages**: memory write filter has 4 substring markers (`execution.rs:61-65`); agent message text has **no** filter (`:2440-2467,2561-2589`); both surface in UI/events and memory lands in future prompts. Context skips only key-name-shaped secrets (`context.rs:113-118`).
8. **R8 — Silent degradation**: memory read errors swallowed (`execution.rs:880`); snapshot failures logged not fatal (`commit_run_snapshot` call sites); agent messages log via `tracing::error!` on publish failure (`:721-742` uses `self.publish` which logs). Operators may not see missing memory/persistence.
9. **R9 — Single-run liveness**: one stuck child/provider call blocks the only run; cancellation exists (run/task tokens, `kill_worker` `execution.rs:1142`) but no progress/stall detection or deadline besides provider retry bounds; peer research names "livelock by blocking" (`[READ-DOC 02-hmars-patterns.md:267-276]`).
10. **R10 — Doc/test drift**: `ws.rs:124-132` comment says Library templates lack prompt assets while `manifest.json` now includes them; `docs/provider/AGENT-ARCHITECTURE-TRUTH.md` is dirty/being edited. Trust code, not docs, at implementation start.

### 10.3 Proposed seams (minimal, code-anchored — recommendations, not implemented)

| # | Seam | Exact insertion points | Why it fixes which gap |
|---|---|---|---|
| S1 | **Role/persona authorization at plan validation** — reject planned roles not in a frozen `{hyper,superagent,worker,subagent,tool}` vocabulary, or map them to a persona before insert; enforce `persona.tier` ↔ mapped tier consistency | `planner.rs:63-71`; `execution.rs:1337-1375`; `prompts.rs:411-434`; `prompts.rs:367-404` | G10/G11/R1 |
| S2 | **Persona prompt on every provider call** — pass `system_prompt_for(task)` into `continue_with_tools` instead of `TASK_SYSTEM`; optionally include planner persona in `plan()` | `provider_executor.rs:357-372` (continuation), `:286-313` (plan) | G2/G3/R2 |
| S3 | **Hierarchical planner schema** — add `parent` to `PLANNER_SYSTEM` JSON example, describe it, and validate role/parent/depth; aggregate only from `task_result` | `provider_executor.rs:25-29,508-519`; `planner.rs:75-100`; `execution.rs:1427-1465` | G4 (nested SuperAgent trees) |
| S4 | **Roster/discovery (bounded)** — add a read-only `GET /api/agents/runs/{run}/roster` and a `list_agents` tool returning `{task_id, role, tier, parent_task_id, depth, state}` for the caller's run, plus expose parent/sibling ids in the child prompt; keep caps (e.g. 64 rows, 2k chars) | `http.rs:518-526`; `tool.rs:108-123,332-430`; `execution.rs:721-742`; `provider_executor.rs:439-445` | G7; enables S5 |
| S5 | **A2A inbox/delivery** — add `status/in_reply_to/answered_at` columns (v11); at turn start, inject the task's bounded **open** messages (newest-first, capped) as *data, not instructions*; answer path for parent/operator; expire on terminal/deadline; no mailbox reads by aggregation | `db.rs:160-178`; `store.rs:522-605`; `execution.rs:2747-2762` (turn start); `http.rs:674-702`; UI `AgentMessagesPanel` | G6/G18/R4 |
| S6 | **Per-hive/branch memory bank** — migration adding `scope/hive_id/branch_id/trust/provenance/expiry` (or a new bank table), read policy = own + ancestors + own role bank; promotion one level with approval; HyperAgent reads distilled hive summaries only; retention/purge | `db.rs:180-197` (v11); `store.rs:608-671`; `execution.rs:856-886,3017-3026`; `http.rs:804-840` | G8/R3; aligns with peer recommendation `[READ-DOC 04-memory-architecture.md:44-56]` |
| S7 | **Admission scheduler + bounded worker pool** — bounded queue, per-run fairness, per-model permits (configurable), task deadline/stall detection, parallel ready leaves; keep one terminal owner and per-task tokens | `execution.rs:1032-1052,1395-1550`; new `scheduler` module; `caps.rs` (configurable); `budget.rs:438-490` | G14/R5; peer blueprint `[READ-DOC 03-scale-concurrency.md:138-165]` |
| S8 | **DB writer actor + read pool + batched transactions** — one write thread behind bounded mpsc; 2–4 read-only connections; batch spawn/state/result writes; short busy timeout | `db.rs:295-350`; all `store.rs` call sites; `execution.rs` snapshot/metric sites | G15/R5; peer DB architecture `[READ-DOC 03-scale-concurrency.md:233-243]` |
| S9 | **Two-lane events + delta coalescing** — keep sequence numbers on control events; coalesce `REPLY_DELTA` per task at 16–33 ms; bounded per-session outbound queue; never let an agent subscribe to the UI bus | `bus.rs:18-20,92-172`; `execution.rs:1764-1795`; `ws.rs:401-455` | G16/R5; peer recommendation `[READ-DOC 03-scale-concurrency.md:268-279]` |
| S10 | **Context persistence + rewire refresh** — persist `ScopedContext` per task (new table or JSON column), reload on boot, re-inject at next turn after rewire; surface memory read failures instead of swallowing | `context.rs:190-257`; `execution.rs:856-886,880`; `ws.rs:650-736` | G9/R8 |
| S11 | **Permission/approval semantics cleanup** — make `ask_parent`/`report_result`/`finish` request-audit classes explicit in comments/contract docs; decide whether `spawn_agent` needs an approval class or an explicit `loop-owned` marker; ensure same-parent sibling approval policy is deliberate | `tool.rs:332-389`; `execution.rs:1682-1736`; `docs/contracts` | G12/R6 |
| S12 | **Observability truth** — per-task provider call/token/cost counters (today only run-level usage persisted at terminal `execution.rs:698-719,3029`; no per-task data), memory read audit, message delivery status, queue depth | `execution.rs:698-719`; `store.rs` / `db.rs`; `http.rs` diagnostics | R5/R8/R9, and the "what did the model see" gap the peer doc flags `[READ-DOC 04-memory-architecture.md:189-191]` |

---

## 11. Twenty-question Q&A about the backend

**Q1. What agent tiers exist in the code?**
Four runtime tiers: `AgentTier::{HyperAgent, SuperAgent, WorkerAgent, ToolAgent}` (`backend/crates/hiverouter-tools/src/tool.rs:66-100`). There is a second, independent `PersonaTier::{Hyper,Super,Worker,Tool}` stored on personas (`prompts.rs:367-379`), but the engine does not use it to derive capability. Wire/UI node kinds are `hyperagent/superagent/worker/tool` (`execution.rs:97-106`; `contract/src/event.rs:240`; `web/src/api/contract.ts:213`). `READ`/`VERIFIED`.

**Q2. How does a free-form role string become a tier?**
`task_agent_tier` maps known aliases to Hyper/Super/Tool/Worker and maps **every unknown role to `WorkerAgent`** (`execution.rs:151-163`). `subagent` and the four worker persona ids are unknown → Worker. `READ`.

**Q3. What is the HyperAgent at runtime — a model agent?**
It is a root task `role="planner"`, `depth=0`, `state=Running` (`execution.rs:1321-1335`), published with `kind="hyperagent"` (`:764-779`), filtered out of the canvas and anchored to the ChatCore overlay (`web/src/flow/graph.ts:26-31,99,151,172`). It is never dispatched: leaf/aggregation orders only include `PlanNode`s (`planner.rs:143-193`; `execution.rs:1395-1489`), so no model call ever uses the HyperAgent tier. `VERIFIED`.

**Q4. Is ChatCore a SuperAgent?**
No. ChatCore is the operator UI: it sends `SUBMIT_PROMPT` and renders `REPLY_DELTA`s (`web/src/chat/ChatCore.tsx:1-5,77-86,130-152`; `state/store.ts:1835-1847`). The aria label "HyperAgent hive chat core" (`strings.ts:77`) names the surface, not an agent. `READ`.

**Q5. Does the planner call use the planner persona?**
No. `planning_prompt` renders the planner template into text (`execution.rs:1552-1566`), then `ProviderExecutorAdapter::plan` prepends `PLANNER_SYSTEM` (`provider_executor.rs:25-29,286-313`). The persona-prompt map is only consulted in `execute` (`:82-87`). The planner template's `system:` block (`planner.prompty:16-18`) does not reach this call. `READ`/`INFERRED`.

**Q6. Is Subagent a real tier?**
No. `AgentTier` has no Subagent variant (`tool.rs:66-81`); `task_agent_tier` maps `subagent` to Worker (`execution.rs:151-162`); the contract has no `subagent` kind (`event.rs:240`). It is a role + persona + allowlist (`personas.json:32-37`; `subagent.prompty:16-19`). `VERIFIED`.

**Q7. Which personas ship and what do they allow?**
Seven: planner (hyper, no tools), superagent (super; spawn/ask/sibling/read/list/report/finish; max 8/3), subagent (worker; ask/sibling/read/list/report/finish), content-writer/data-analyst/code-reviewer (worker; read/list/ask/sibling/report/finish), chatbot (worker; ask/sibling/report/finish). Evidence: `assets/prompts/personas.json:3-66`. `READ`.

**Q8. Are all personas reachable as spawned agents?**
No. `validate_spawn_role` only accepts hyper/super/worker/agent/subagent names (`execution.rs:113-142`), so content-writer/data-analyst/code-reviewer/chatbot are refused by `SPAWN_SUB_HIVE`/`spawn_agent`. They are reachable through `TEMPLATE_INSERT`, which sets `role = template_id` and skips the spawn-role validator (`ws.rs:104-120,200-293,797-831`). The planner can also emit an arbitrary role string (validation accepts any non-empty role, `planner.rs:63-71`), but the prompt asks for worker/superagent only (`provider_executor.rs:28-29`). `READ`+`INFERRED`.

**Q9. Does the persona system prompt reach the model?**
Yes on the first provider call: `execute` uses `system_prompt_for(task)` = persona prompt by exact `task.role`, else `TASK_SYSTEM` (`provider_executor.rs:82-87,315-348`). No on tool-turn continuations: `continue_with_tools` starts with `TASK_SYSTEM` (`:369-372`). No for planning (`PLANNER_SYSTEM`, `:295`). No for persona `user:` blocks (not used; task prompt is built separately at `:439-445`). `VERIFIED` (two call sites).

**Q10. How is per-task context created and inherited?**
`seed_task_context` copies the parent's facts snapshot, adds the role persona's allowed tools/output schema, and injects the newest 8 role memories as `memory:*` facts (`execution.rs:856-886`). The prompt renders a bounded (4,000-char) scoped-facts block (`execution.rs:53-58`; `context.rs:66-95`). `ContextStore` is in-memory only (`context.rs:190-257`). `VERIFIED`.

**Q11. What does a rewire do to context?**
After the graph `rewire` succeeds (`graph.rs:141-201`), `ws.rs:650-736` calls `ContextStore::switch_on_rewire` (`context.rs:225-244`), which clears only the previous parent's prefixed facts, injects the new parent's facts, and keeps the task's own tool ids/output schema (`context.rs:138-188`). Failure after the graph write is loudly refused, not ACKed (`ws.rs:707-719`). `READ`.

**Q12. What is `agent_memory`'s schema and scope?**
`agent_memory(agent_key, key, value, run_id, task_id, created_at)`, `PK(agent_key,key)` (`db.rs:180-197`). It is role-keyed and global across runs/hives; `run_id/task_id` are just provenance; there is no hive/branch/trust/expiry (`READ`; peer confirms `[READ-DOC 04-memory-architecture.md:133-145]`).

**Q13. When is memory written and read?**
Written on task completion with the final text (2,000 chars) under key `task:<task_id>`, only if it passes four secret substring markers (`execution.rs:3000-3026,61-65`). Read at task creation: newest 8 for `task.role`, errors silently ignored (`:880-884`; `store.rs:645-671`). UI lists/deletes by role (`http.rs:804-840`). `READ`.

**Q14. Is there per-hive memory or HyperAgent aggregate access?**
No. Migrations v1–v10 contain no hive/branch scope (`db.rs:32-249`). The root context is seeded with planner-role memory (`execution.rs:1334,880-884`) but the root never executes and `plan()` reads no context (`provider_executor.rs:286-313`), so there is no HyperAgent read policy or digest. `VERIFIED` (absence shown at the only plausible sites).

**Q15. How do agents communicate?**
Via three model tools: `ask_parent` (write-only message to the parent), `send_to_sibling` (same-parent immediate; cross-branch requires approval), and `spawn_agent` (inline blocking child whose result returns to the caller) (`execution.rs:2417-2729`). Messages are stored in `agent_messages` and published as `AGENT_MESSAGE` (`db.rs:160-178`; `execution.rs:721-742`; UI `AgentMessagesPanel`), but **no code delivers them into any model prompt** (only reader: HTTP mailbox, `http.rs:674-702`). `VERIFIED` (grep single consumer).

**Q16. Can an agent discover its siblings/roster?**
No. There is no roster endpoint/tool; routes are messages/profiles/memory/projects only (`http.rs:518-526`). The task prompt has role/depth but no task ids (`provider_executor.rs:439-445`). `send_to_sibling` requires an exact target `TaskId` (`tool.rs:231-239`), which an agent can only know from a prior `spawn_agent` result (its child) — not for siblings. `READ`+`INFERRED`.

**Q17. What buses exist?**
`EventBus` (server→UI session fan-out, monotonic seq, 1,024 broadcast / 256 replay; `bus.rs:18-20,84-172`; sessions subscribe `ws.rs:401-402`), and `GraphAuthority` (single owner of parent/child topology + rewire; `graph.rs:1-49,73-201`). There is **no agent bus**; the event plane is for UI/transport, and `agent_messages` is the only A2A record (write-only). `READ`.

**Q18. Which tools does each tier see on the first turn, and how are approvals applied?**
First turn advertises `tool_specs_for_tier(task_agent_tier(task))` (`provider_executor.rs:315-348`; `tool_loop.rs:206`); continuation re-advertises the same tier specs (`:409-415`). Tier allowlists are at `tool.rs:391-430`. Approval-required calls (Mutate/Execute other than the loop-owned spawn/sibling paths) suspend through the approval bridge; the side-effect kill switch defaults on (`config.rs:195,355-371`; `tools_bridge.rs:196-247`); `spawn_agent` is exempt (`execution.rs:1682-1692`); same-parent `send_to_sibling` skips approval despite Mutate (`:1719-1720`). `VERIFIED`/`READ`.

**Q19. What are the hard caps, and can they be configured?**
Depth 3, fan-out 8, agents 64, provider attempts 40, tokens 400,000, cost $0.50, tool turns 4, tool calls 16, approval waits 8, plan nodes 40 (prompt asks 1..8), planner max output 16,384 (`caps.rs:8-15`; `budget.rs:24-43,62-81`; `tool_loop.rs:14-21`; `planner.rs:9-10`; `provider_executor.rs:25-29`). `Caps` is hard-coded `Caps::default()` in the engine (`execution.rs:473`; `ws.rs:1099+` tests) — no env/config override was found in this read (`INFERRED`: raising caps requires a code change). `READ` for values.

**Q20. Is there a scheduler/worker pool, and why can't 50–150 models run concurrently?**
No scheduler: one active run (`execution.rs:1032-1052`), sequential leaves/aggregators/spawn drain (`:1395-1550`), inline child await (`:2668-2681`), one SQLite connection behind a mutex (`db.rs:295-350`), provider limiter of 8 (`budget.rs:42-43`), per-fragment event fan-out with 1,024/256 bounds (`bus.rs:18-20`; `execution.rs:1764-1795`). The engine can have at most one task/provider call running; "50–150 concurrently active" would require a bounded scheduler, parallel ready tasks, a DB writer/read-pool split, coalesced events, and raised/reconfigured caps. `READ`+`INFERRED`; the peer arithmetic arrives at the same conclusion (`[READ-DOC 03-scale-concurrency.md:11-23,88-106,138-165]`).

---

## 12. Consolidated claim register (material claims → evidence → status)

| # | Claim | Evidence | Status |
|---|---|---|---|
| 1 | Agent tiers are exactly Hyper/Super/Worker/Tool; no Subagent tier | `hiverouter-tools/src/tool.rs:66-100`; `execution.rs:151-163` | VERIFIED |
| 2 | Role→tier mapping is fail-closed (unknown → Worker) | `execution.rs:151-163` | READ |
| 3 | HyperAgent root is a planner row/Ui anchor; never executed | `execution.rs:1321-1335,1395-1489`; `planner.rs:143-193` | VERIFIED |
| 4 | ChatCore is UI only; no agent role | `ChatCore.tsx:1-5,77-86,140-152`; `store.ts:1835-1847` | READ |
| 5 | Planner call does not use the planner persona system prompt | `provider_executor.rs:286-313` vs `:82-87`; `execution.rs:1552-1566` | READ |
| 6 | Persona system prompt is used on first leaf call, not on continuation | `provider_executor.rs:315-348` vs `:357-372` | VERIFIED |
| 7 | 7 personas ship; 4 worker personas rejected by spawn role validation | `personas.json:3-66`; `execution.rs:113-142` | VERIFIED |
| 8 | Template Insert bypasses spawn-role validation and sets role=template id | `ws.rs:200-293,797-831` | READ |
| 9 | Subagent maps to Worker tier; no subagent node kind | `tool.rs:66-100`; `execution.rs:151-162`; `event.rs:240` | VERIFIED |
| 10 | Per-persona spawn caps only apply when parent role matches persona id | `execution.rs:549-558,1083-1090`; `prompts.rs:421-432` | READ |
| 11 | ScopedContext is in-memory, facts copied from parent snapshot, rewire clears old parent facts | `context.rs:19-29,138-188,190-257`; `execution.rs:856-886`; `ws.rs:650-736` | VERIFIED |
| 12 | agent_memory is role-keyed global, newest 8, written from completed task text (2k chars) | `db.rs:180-197`; `store.rs:608-671`; `execution.rs:880-884,3017-3026` | VERIFIED |
| 13 | No per-hive/branch memory scope exists | `db.rs:32-249` (no such column/table) | VERIFIED (negative, v1–v10) |
| 14 | HyperAgent has no memory aggregate read; root memory seed never executes | `execution.rs:1334,880-884,1395-1489`; `provider_executor.rs:286-313` | READ |
| 15 | ask_parent writes message + event; no reply/delivery | `execution.rs:2417-2487`; `store.rs:574-605`; only consumer `http.rs:684` | VERIFIED |
| 16 | send_to_sibling same-parent immediate; cross-branch approval; no delivery | `execution.rs:1695-1720,2493-2609` | READ |
| 17 | No roster/discovery endpoint or tool; task prompt lacks ids | `http.rs:518-526`; `provider_executor.rs:439-445`; `tool.rs:231-239` | VERIFIED (negative grep) |
| 18 | EventBus is UI transport only; no agent bus | `bus.rs:1-20,84-172`; `ws.rs:401-402` | READ |
| 19 | Tool registry has 9 tools, 4 permission classes, 4 tier allowlists | `tool.rs:66-100,153-187,304-430` | READ |
| 20 | First provider call now advertises tier tool specs; continuation too | `provider_executor.rs:315-348,409-415`; comment `:334-339` | VERIFIED |
| 21 | spawn_agent is loop-owned and approval-free despite Execute class | `execution.rs:1682-1692`; `tool.rs:332-338` | READ |
| 22 | ask_parent/report_result/finish are Read-class but write state/DB | `tool.rs:339-361`; `execution.rs:2417-2487,2864-2915` | READ |
| 23 | Side effects default enabled; approval path preflight→grant→commit | `config.rs:195,355-371`; `tools_bridge.rs:196-247` | READ |
| 24 | Hard caps: depth3/fanout8/64 agents/40 calls/400k tokens/$0.50 | `caps.rs:8-15`; `budget.rs:24-43,62-81` | VERIFIED |
| 25 | Planner prompt asks 1..8 nodes; validator allows ≤40 | `provider_executor.rs:25-29`; `planner.rs:9-10,44-56` | VERIFIED |
| 26 | Engine `check_calls(1)` is stateless (no accumulated count) | `caps.rs:62-71`; `execution.rs:1313,2771` | VERIFIED |
| 27 | One active run; dispatch sequential leaves/aggregators/spawns; child awaited inline | `execution.rs:1032-1052,1395-1550,2668-2681` | VERIFIED |
| 28 | DB is one `Mutex<Connection>`; WAL/busy_timeout configured | `db.rs:295-350` | READ |
| 29 | Event bus 1,024 broadcast / 256 replay; one REPLY_DELTA per fragment | `bus.rs:18-20`; `execution.rs:1764-1795` | READ |
| 30 | Snapshot after every node; run metrics scan all tasks | `execution.rs:657-698,1391-1408,1470,1522-1527` | READ |
| 31 | UI filters hyperagent from canvas and anchors its edges; hierarchy from parent links | `graph.ts:26-31,99,151,172`; `AgentHierarchy.tsx:1-53` | READ |
| 32 | No scheduler/worker pool exists in backend source | grep `scheduler|Semaphore|join_all|FuturesUnordered` → only provider limiter | READ (negative) |
| 33 | 50–150 concurrent active agents impossible today | derived from 27 + 24 + 28 + 29 | INFERRED (code-constrained; `UNVERIFIED` runtime) |
| 34 | 300–600 tasks under one HyperAgent impossible today | derived from 24/25/26/27/28 | INFERRED (code-constrained) |
| 35 | Runtime behavior of every path above | not executed in this pass | **UNVERIFIED** |

---

## 13. What must not be claimed (non-claims) and open unknowns

**Do not claim:** that ChatCore is a SuperAgent; that the HyperAgent executes a model turn; that SuperAgents form nested sub-hives in production planning (the planner schema omits `parent`); that ask_parent/send_to_sibling deliver messages or get answers; that agents can discover a roster/siblings; that per-hive memory or HyperAgent memory aggregation exists; that subagent is a real tier/card; that persona prompts apply on tool continuations; that 50–150 concurrent active agents or 300–600 tasks are supported; that permission classes are a complete approval policy; that the current UI/aggregation scales to hundreds of nodes; that any runtime behavior was proven here (no build/test/run).

**Open unknowns (`UNVERIFIED`):** actual provider/token throughput and 429 behaviour on this account; live scheduler-free wall-clock for a 40-call run; whether the installed binary corresponds to HEAD `bd575ba`; whether UI renders 100+ cards acceptably; whether persisted agent profiles are present/valid in a real data dir; planner output distribution (does the model ever emit `parent` or persona ids without instruction); whether `max_args_preview`/newer HMAS event fields are exercised by the current UI.

---

## 14. Read-only statement

This analyst read the source and peer HMARS corpus listed in §1 and wrote **only** this file (`docs/research/hmars/07-code-backend.md`). No product code, test, configuration, gate, git state, key, credential, server process, provider call or paid call was touched; no build or test was run. Every runtime statement is therefore unverified.
