# 08 — HMARS UI truth: what the code actually renders, wires, and hides

**File:** `docs/research/hmars/08-code-ui.md`
**Author:** codebase analyst #2 (read-only UI/web seat)
**Date:** 2026-11-07 (America/Chicago; window ~20:20–20:55)
**Scope:** `web/src`, `web/tests`, and the exact HTTP/WS routes those surfaces call in
`backend/crates/hiverouter-core` and `backend/crates/hiverouter-tools`.
**Method:** source read + targeted line inspection + read-only `GET` probes of the already-running
server. No code edit, no app restart, no prompt submission, no provider/account call, no credential read.
**Working tree:** `bd575ba7adf3a5f2a9bdbc69f9bbf7084fe6fe65` (`bd575ba`), read at the hashes in §11.
The tree also contains uncommitted teammate edits (notably `docs/provider/AGENT-ARCHITECTURE-TRUTH.md`,
and the approval state machine files `web/src/state/approval.ts` / `store.ts`); line anchors below are
for the tree as read, not for the commit.

**Claim labels used in this report**

| Label | Meaning |
|---|---|
| **VERIFIED** | Exact `file:line` read in this pass (source as written), or an HTTP response observed in this pass. |
| **READ** | Read from a doc/prior audit or from source, but the claim depends on interpretation or on a different build. |
| **INFERRED** | Consequence deduced from two or more verified code paths; not exercised at runtime. |
| **UNVERIFIED** | Not observed in this pass; requires a live run, a build, or a browser capture. |

> **No pixel-parity claim is made here.** Board geometry is from the FigCraft README/ASSETS manifest;
> prior measured UI audit [`docs/trace/provider/05-figcraft-ux-gaps.md`](../../trace/provider/05-figcraft-ux-gaps.md)
> is READ (older build) and is not re-confirmed except where this pass re-reads the code.

---

## 0. Runtime snapshot observed in this pass (READ-ONLY, server not touched)

The portable server was already running on `127.0.0.1:8099`. Only `GET`s were sent.

| Probe | Response | Label |
|---|---|---|
| `GET /health` | `200` `{"status":"ok","version":"1.2.0-dev","schema":"hiverouter.v1","uptime_secs":3830,...}`; checks `rustc ok`, `node ok`, `deepseek_api_key` present (boolean only, no value read) | VERIFIED |
| `GET /api/projects` | `200 {"projects":[]}` — fresh/empty project registry | VERIFIED |
| `GET /api/history/runs` | `200 {"runs":[]}` — no committed runs in this data dir | VERIFIED |
| `GET /api/agents/profiles` | `200`, 7 profiles: `chatbot, code-reviewer, content-writer, data-analyst, planner, subagent, superagent` | VERIFIED |
| `GET /api/agents/memory` | `200`, 7 profiles, every `memories` array empty | VERIFIED |
| `GET /` | `200`, served asset `assets/index-a4oJkwuC.js`, CSS `assets/index-DMO_P0oK.css` | VERIFIED |
| `GET /api/provider/models` / `/api/provider/balance` | **not called in this pass** (would call the external provider account) | — |

Consequence for this pass: the live UI is in its **empty state** (no run, no projects, no history);
all "what renders during a run" claims below come from source, not from a browser capture.
`05`'s measured screenshots (`docs/trace/provider/shots-figcraft/`) and `docs/trace/figma-parity/*.png`
are from an older build and are READ, not re-measured.

---

## 1. Executive verdict (HMARS lens)

The UI is an honest, strict, *event-derived* shell around a real hierarchical engine. It is **not** a
view over the HMARS runtime the HMARS goal needs, because four identities are missing from the surface:

1. **Agent identity.** The wire carries a node *kind* (`hyperagent|superagent|worker|tool`), never a
   persona/profile/role label; a `subagent` has no kind and renders as a worker. The persona prompts the
   newest commit added are keyed by persona id, but the roles that actually reach the executor are
   `worker` (no such persona) and Title-Case `SuperAgent`/`WorkerAgent`/`Subagent` (case miss) — so most
   leaf calls fall back to the generic `TASK_SYSTEM`. **INFERRED** from verified paths (§3.4).
2. **Operator control of the runtime.** There is no model selector, thinking/reasoning selector, profile
   selector, spend gate, project switch, or A2A compose. Model/thinking/model-choice is environment-only
   (`HIVEROUTER_MODEL` etc.), surfaced read-only. **VERIFIED** (§5.2).
3. **Governance context.** Approvals are now genuinely wired and can execute side effects, but the
   approval card shows a tool name and a truncated `args_sha256`, not the arguments, target path, or
   message text. An expired/stale approval whose resolve never failed has **no Dismiss control**. **VERIFIED**
   (§5.4, §7.5).
4. **A real HMAS view.** "Swarm" is a read-only indented task list plus five counters; topology/A2A
   manipulation does not exist; time-travel replay flattens every node to a worker card. **VERIFIED**
   (§2.6, §7.4).

---

## 2. Q1 — What the UI actually renders for each HMARS entity

### 2.1 HyperAgent (root planner)

| Fact | Evidence | Label |
|---|---|---|
| Contract/render vocabulary has 5 kinds: `hyperagent, superagent, worker, tool, error` | `web/src/state/agentState.ts:126-135` (`NODE_KIND_KEYS`, `NODE_KIND_LABELS`) | VERIFIED |
| Backend emits the kind from the agent tier: `HyperAgent→"hyperagent"` etc. | `backend/crates/hiverouter-core/src/execution.rs:97-106` (`node_kind`) | VERIFIED |
| A HyperAgent card implementation exists | `web/src/flow/nodes.tsx:245-267` (`HyperAgentNode`), registered at `:269-275` | VERIFIED |
| **That card is never rendered for a live graph.** Canvas nodes filter out `kind === 'hyperagent'` | `web/src/flow/graph.ts:143-165` (`buildNodes`; filter at `:151`) | VERIFIED |
| HyperAgent is represented by the **fixed ChatCore overlay**, and only an invisible 1×1 anchor node + bee edges are added to the canvas | `web/src/flow/graph.ts:26-28` (`HIVE_ANCHOR_ID`), `web/src/flow/Canvas.tsx:167-189` (anchor node), `:213` edges | VERIFIED |
| ChatCore is labelled "Hive AI Assistant", not HyperAgent; aria = "HyperAgent hive chat core" | `web/src/state/strings.ts:76-89`; `web/src/chat/ChatCore.tsx:179-183` | VERIFIED |
| The root planner task *does* appear as the depth-0 row of the Agent/Swarm hierarchy (it is in `tasks`) | `web/src/panels/AgentHierarchy.tsx:21-53` (walk from `parentId=null` at `:35-45`) | INFERRED from root insert `execution.rs:1321-1330` |

**Verdict:** the HyperAgent exists in the UI only as the chat card + hierarchy root row. The dedicated
HyperAgent card is dead code (kept for the node-renderer map/tests). This matches FigCraft board 07's
"chat core anchored at the hive", but it means the operator cannot see HyperAgent state as a node: the
root task's `state` is only visible in the Agents list/hierarchy, never on the canvas.

### 2.2 SuperAgent

| Fact | Evidence | Label |
|---|---|---|
| Card is real: 420×150, radius 14, honey header 40 px, 68×18 state pill, progress bar, static role caption | `web/src/flow/nodes.tsx:141-173`; sizes `web/src/tokens/tokens.ts:173-178`; node size wiring `web/src/flow/graph.ts:30-40` | VERIFIED |
| Only produced when a plan node's role maps to `AgentTier::SuperAgent` (roles `super/superagent/super_agent/sub_hive/subhive`) | `execution.rs:144-163` (`task_agent_tier`), `:113-136` (spawn policy) | VERIFIED |
| The planner is explicitly told to use `superagent` only when a branch needs delegation | `backend/crates/hiverouter-core/src/provider_executor.rs:25-29` (`PLANNER_SYSTEM`) | VERIFIED |
| The server never sends a display label or progress, so the card shows `WorkerAgent`-style fallback labels and `progress=null` | producer `execution.rs:764-779` (no `label`, no `progress`); fallback `web/src/state/store.ts:325-331`, `:1460-1478`; card renders `—` at `web/src/flow/nodes.tsx:164-168` | VERIFIED for the absence; fallback label rendering INFERRED |
| Identity (persona/profile) is not displayed anywhere on the card: the only sub-label is the static `roleCaption` "decomposes intent, owns the sub-hive" | `nodes.tsx:158-162`; `strings.ts:281` | VERIFIED |

### 2.3 WorkerAgent

| Fact | Evidence | Label |
|---|---|---|
| Card is real: 320×96, radius 12, state dot + label + `WORKER` label + 3 px progress + context line | `web/src/flow/nodes.tsx:175-194` | VERIFIED |
| Context line is hard-coded `formatCtx(null)` — always the honest `—`, never a real context figure | `nodes.tsx:189`; `strings.ts` node block (`ctx: 'ctx'`) | VERIFIED |
| Progress is always `—` because no server event carries progress; `TaskRecord.progress` stays `null` | `execution.rs:764-779`; `store.ts:1466-1469` | VERIFIED |
| Planner workers get the role string `"worker"` | `provider_executor.rs:25-29`; plan validation only requires non-empty role (`planner.rs:64`) | VERIFIED |
| **No `worker` persona exists** in the shipped registry, so the leaf system prompt resolves to the generic `TASK_SYSTEM` for planner-created workers | `backend/crates/hiverouter-core/assets/prompts/personas.json` (ids: planner, content-writer, superagent, subagent, data-analyst, code-reviewer, chatbot); `provider_executor.rs:82-87` exact `HashMap::get(&task.role)` | VERIFIED path; effect INFERRED |

### 2.4 Subagent

| Fact | Evidence | Label |
|---|---|---|
| `subagent` is an accepted **spawn role**, and a persona/template for it now ships | `execution.rs:108-142` (spawn policy), `:151-162` (tier mapping); `personas.json` `subagent` entry | VERIFIED |
| It is **not a tier**: role `"subagent"` falls through `task_agent_tier` to the `_ => WorkerAgent` arm | `execution.rs:151-162` (no `subagent` match arm; default at `:161`) | VERIFIED |
| Therefore its node kind is `"worker"` and the UI renders a **WorkerAgent card** with a fallback label; there is no Subagent card, kind or label | `execution.rs:97-106`; `web/src/state/agentState.ts:126-135`; `nodes.tsx:175-194` | VERIFIED |
| The hierarchy shows it as a WorkerAgent row at its real depth; no role is shown | `AgentHierarchy.tsx:55-81`; `NODE_KIND_LABELS` at `agentState.ts:129-135` | VERIFIED |
| UI spawn sends the **Title-Case display string**: `'SuperAgent' | 'WorkerAgent' | 'Subagent'` | `web/src/state/store.ts:1865-1871`; labels `strings.ts:275-278` | VERIFIED |
| Persona lookup is exact-case by persona id, so Title-Case roles miss the lowercase persona map and fall back to `TASK_SYSTEM`; `system_prompt_for` is only used on the first leaf call anyway | `provider_executor.rs:82-87`, `:328`; continuation hard-codes `TASK_SYSTEM` at `:369-372` | VERIFIED path; effect INFERRED |

**Verdict:** a Subagent today is a WorkerAgent with a hidden persona string. It has no distinct visuals,
no memory tier, no kind, and (from the chat/UI path) no persona prompt.

### 2.5 ToolAgent / tool calls

| Fact | Evidence | Label |
|---|---|---|
| A ToolAgent card exists: 420×240, dark 132 px "result" box, score row that always shows `—` | `web/src/flow/nodes.tsx:196-224`; `strings.ts` (`noToolOutput: 'tool output not measured'`) | VERIFIED |
| A `tool`-tier node can only exist if the planner emits a role that maps to `AgentTier::ToolAgent` — but the planner prompt explicitly offers only `worker | superagent` | `provider_executor.rs:25-29`; `execution.rs:157-158` | VERIFIED |
| Real tool work is not a graph node: it is `TOOL_CALL`/`TOOL_RESULT` events rendered as activity rows | contract event map `web/src/api/contract.ts:385-411`; store handlers `store.ts:1502-1573` | VERIFIED |
| Tool activity is rendered in **two** places: a full panel under **Agents** (all runs' current run activity), and the chat's last-4 "ACTIONS" block | `web/src/panels/AgentsPanel.tsx:65-72`; `web/src/panels/ToolActivityPanel.tsx:131-184`; `ChatCore.tsx:274-308` | VERIFIED |
| Rows show tool, outcome, truncated `args_sha256` (12 chars), turn, tool-call id, optional refusal code, ≤160-char text | `ToolActivityPanel.tsx:145-179` | VERIFIED |
| No approval state is attached to an activity row, and the chat block does not distinguish pending vs awaiting approval | `ToolActivityPanel.tsx:23-33` (outcome union), `ChatCore.tsx:288-304` | VERIFIED |

### 2.6 Error / failed node

| Fact | Evidence | Label |
|---|---|---|
| `error` is a node kind, but the backend's `node_kind()` can never return it (only 4 tiers) | `execution.rs:97-106`; `agentState.ts:126-135` | VERIFIED |
| A broken-route card exists (320×110, crimson, shows `errorCode` if present) | `nodes.tsx:226-243` | VERIFIED |
| Store falls back to `kind='worker'` when a frame omits kind, so an `error` card cannot be produced by a normal run | `store.ts:1447-1450`, `:1460-1464` | VERIFIED |
| Errors do surface as **failed/killed state styling** on whatever card kind exists, plus broken bee edges | `nodes.tsx:21-38` (state tone), `graph.ts:167-188` (`data.broken` from kind/state at `:180-185`) | VERIFIED |
| Run-level failures surface in chat as a named "Run refused" banner (code/message/how_to_fix), fed from `RUN_STATUS.error_code` | `ChatCore.tsx:88-90`, `:211-221`; `store.ts:1377-1409` | VERIFIED |
| Per-node failure `error_code` reaches the store but is only rendered by the dead error card; card headers do not show it | `store.ts:1470`; `nodes.tsx:143-173` (no errorCode render) | VERIFIED |

### 2.7 State vocabulary

- 16 agent states are the one vocabulary: `agentState.ts:9-26` (`AGENT_STATES`), 4 dot states
  (`DOT_STATES` at `:30-31`), all others get shape+text cues (`:58-84`), terminal set `:111-119`,
  run-active predicate `:121-123`. **VERIFIED**
- The store's approval states are a separate display machine (`pending/resolving/awaiting_server/retryable/
  expired/stale/unconfirmed`) in `web/src/state/approval.ts` — two vocabularies the UI must reconcile
  (agent state `awaiting_approval` vs approval display status). **VERIFIED** (file read; not audited for
  contradictions here).

---

## 3. Q2 — Chat → `SUBMIT_PROMPT` → run → reply/tool event chain

### 3.1 Front-end half

| Step | Evidence | Label |
|---|---|---|
| Composer submit is the only prompt entry point; draft is kept until ACK settlement, and a named refusal restores it and retracts exactly the optimistic user row | `web/src/chat/ChatCore.tsx:126-175` (submit), `:319-363` (form) | VERIFIED |
| Store validates (`E_PROMPT_EMPTY`/length), appends an optimistic user message, sends `SUBMIT_PROMPT` with `project_id` | `web/src/state/store.ts:1835-1848`; `store.ts:1692-1745` (`run` helper) | VERIFIED |
| `project_id` is a **local** UUID persisted in `localStorage` (`hivemind.project-id`), not a server project selector; display name is a constant | `store.ts:333-362` (`baseProject`), `:1768` (`project: baseProject()`) | VERIFIED |
| Client WS API sends the command; ACK/refusal correlation is the store's `pendingCommands`/`refusals` | `web/src/api/client.ts` (client), `store.ts:1692-1760` | VERIFIED |
| Success is observed indirectly: `RUN_CREATED` clears the old run's tasks/activity/history and sets the run | `store.ts:1354-1375` | VERIFIED |
| `REPLY_DELTA` appends to the last streaming assistant message (or opens one) and marks `streamingRunId`; `REPLY_DONE` closes it | `store.ts:1411-1440` | VERIFIED |
| `NODE_SPAWN/NODE_DRAG/NODE_REWIRE/NODE_STATE` maintain `tasks`; kind defaults to the previous kind or `worker`; label falls back to `"<KindLabel> <last-8>"`; progress/rev/error follow payload or `null` | `store.ts:1441-1493`; label fallback `:325-331` | VERIFIED |
| `RUN_METRICS` sets HUD metrics; `TOOL_CALL/RESULT` maintain `toolActivity` with dedupe, buffering, immutable terminal outcomes; `APPROVAL_*` maintain `approvals`/`approvalResolve` | `store.ts:1498-1630` | VERIFIED |
| `AGENT_MESSAGE` only increments `agentMessageVersion`; no inbox, toast, selection or transcript row | `store.ts:1494-1497`; consumer `AgentMessagesPanel.tsx:22-23`, `:84` | VERIFIED |
| History is rehydrated from HTTP (`/api/history/runs`, `/revisions`, `/state?revision=`, `/messages`) on boot/reconnect and after terminal runs | `store.ts:418-560`, `:1020-1180`; routes `http.rs:512-529` | VERIFIED |

### 3.2 Server half

| Step | Evidence | Label |
|---|---|---|
| `ws.rs` dispatch arm calls `engine.submit_prompt(text, project_id, correlation)` and ACKs/errs | `backend/crates/hiverouter-core/src/ws.rs:744-753` (read context `:735-786`) | VERIFIED |
| Engine validates length/empty, dedupes by correlation receipt, rejects a second active run (`E_RUN_ACTIVE`), creates the run row, records the receipt, persists the user chat message, marks the run active, publishes `RUN_CREATED`, spawns dispatch | `execution.rs:1016-1054` | VERIFIED |
| Dispatch sets `planning`, makes **one** planning provider call, validates the plan DAG, inserts the root task (`role:"planner"`, depth 0), then spawns the plan nodes | `execution.rs:1309-1330`; root `:1321-1330`; planning prompt built at `:1553-1560` | VERIFIED (planning prompt source: `provider_executor.rs:286-313`) |
| Planned nodes become tasks with their plan role; each spawn publishes `NODE_SPAWN` with kind = tier mapping, parent, rev — **no label, no progress** | `execution.rs:1360-1400` (plan-node creation), `:764-779` (`publish_node_spawn`), `:1061-1112` (`spawn_sub_hive`) | VERIFIED |
| Leaf execution calls the provider with `system_prompt_for(task)` and role/turn user prompt, tier tool specs, and a per-call delta sink | `provider_executor.rs:315-348`; `task_prompt` at `:439-448`; tool specs `:323-339` | VERIFIED |
| Streaming: the sink publishes `REPLY_DELTA` frames while the provider streams; returned deltas are published only if the sink never fired | `execution.rs:2779-2812`; `reply_delta_sink`/publish anchors `:1780`, `:1812` | VERIFIED |
| Tool turns: model tool calls are assembled (provider `tool_assembler`), planned/validated, counted against caps, authorized, executed (read-only or approval-gated), and each call publishes `TOOL_CALL`/`TOOL_RESULT`; results are fed back for up to 4 turns / 16 calls/run / 65,536-byte result cap | `execution.rs:1595-1754` (turn/budget/authorize), `:1860-1920` (`TOOL_CALL`/`TOOL_RESULT` publish anchors), `:2812-3010` (continuation loop); caps in `web/src/api`-adjacent core `tool_loop` (reference) | VERIFIED at anchors; exact cap constants READ from prior session notes |
| Approval-gated side effects: engine `suspend_for_approval` persists `awaiting_approval`, arms a waiter, publishes `APPROVAL_REQUEST`, waits for resolve/TTL/cancel | `execution.rs:1998-2165`; builder of approval request `:2036-2054` | VERIFIED |
| Side-effect execution exists in the tools crate: read-only preflight, then one granted commit (`file_write` atomic under the workspace jail; `shell_exec` allowlisted Job-Object path) | `backend/crates/hiverouter-tools/src/side_effects.rs:1-13`, `:28-39`, `:103` (`preflight_side_effect`), `:163` (`commit_side_effect`); jail `jail.rs:262` (`write_atomic`) | VERIFIED |
| Ask/sibling tools persist `agent_messages` and publish `AGENT_MESSAGE`; same-parent sibling only; per-run message cap; cross-branch sibling requires the approval path | `execution.rs:2416-2487` (`ask_parent`), `:2489-2609` (`send_to_sibling`), `:1688-1721` (routing/approval gate); cap const `:70` | VERIFIED |
| Usage is accumulated per run and persisted at terminal dispatch; `run_usage` route reads it back | `execution.rs:698-719` (`add_run_usage`, `persist_run_usage`), `:1305` (terminal persist); `store.rs:678-700`, `:715-730`; `http.rs:842-875` | VERIFIED |
| Final assistant reply is persisted once; history route rehydrates transcript/revisions | `execution.rs:3003`; `http.rs:634-670` (history messages) | VERIFIED |

### 3.3 Sequence (one line, end to end)

`ChatCore.submit` (`ChatCore.tsx:132`) → `store.submitPrompt` (`store.ts:1835`) → `SUBMIT_PROMPT`
(`store.ts:1847`) → `ws.rs:744` → `RunEngine::submit_prompt` (`execution.rs:1016`) → `RUN_CREATED`
(`:1040`) → dispatch → `planning` (`:1310`) → plan (`provider_executor.rs:286`) → root+node tasks
(`execution.rs:1321-1400`) → `NODE_SPAWN` (`:765`) → per-task `provider.execute`
(`provider_executor.rs:315`) → `REPLY_DELTA` sink (`execution.rs:2791`) → store append
(`store.ts:1411`) → tool turns (`execution.rs:1595`) → `TOOL_CALL`/`TOOL_RESULT` (`:1892`, `:1908`)
→ approvals (`:1998`) → `REPLY_DONE`/`RUN_STATUS`/`RUN_METRICS` → `persist_run_usage`/snapshot
(`:1302`, `:1305`) → HTTP history rehydrate (`store.ts:1020+`).

### 3.4 Persona-chain defect (the most HMARS-significant chain finding)

The newest commit's claim is "real SuperAgent/Subagent personas and persona prompt execution"
(`bd575ba`). The code does render persona system prompts — `http.rs:254-275` builds
`persona_id → system` from each persona's template; `build_executor` wires it
(`http.rs:230-250`); `system_prompt_for` looks it up by **exact** `task.role`
(`provider_executor.rs:82-87`). But:

- Planner-created workers have role `"worker"`; the persona map has **no `worker` key**
  (`personas.json`; `provider_executor.rs:25-29`) → generic `TASK_SYSTEM`.
- UI-created children have Title-Case roles from `STRINGS.node.*` (`store.ts:1865-1871`;
  `strings.ts:275-278`), while persona ids are lowercase → exact `HashMap::get` misses →
  generic `TASK_SYSTEM`.
- Even when turn 1 gets a persona prompt, the **tool continuation rebuilds messages with
  `TASK_SYSTEM`** (`provider_executor.rs:369-372`) → persona is dropped from turn 2 onward.
- The planner call itself uses the hard-coded `PLANNER_SYSTEM` (`provider_executor.rs:25-29`,
  `:295-304`), not the `planner` persona prompt rendered in `http.rs:254-275`.

**Net effect (INFERRED):** SuperAgent/Subagent persona text is reachable only for tasks whose `role`
exactly equals a lowercase persona id — i.e. template-inserted DAG tasks (template id = role) and,
sometimes, planner `superagent` branches. Chat/planner workers and every UI-spawned SuperAgent /
WorkerAgent / Subagent fall back to the generic system prompt. The `AGENT-ARCHITECTURE-TRUTH.md`
sentence "the adapter uses it for leaf and tool-turn calls" (`docs/provider/AGENT-ARCHITECTURE-TRUTH.md`,
working-tree copy, line context around the table) is **not true for tool turns** in the code read here.

---

## 4. Q3 — Agent profiles / memory / messages panels: real or not?

### 4.1 Summary

| Panel | Data source | Real? | What it is not |
|---|---|---|---|
| `AgentProfilesPanel` | `GET /api/agents/profiles` → `state.engine.personas()`; import `POST /api/agents/profiles/import`; delete `DELETE /api/agents/profiles/{id}` | **Yes** — live probe returned 7 personas; server merges persisted profiles over embedded ones (`http.rs:137-180`, `:782-802`) | Not connected to run creation. No "use this profile" control; import only writes JSON text; no edit. |
| `AgentMemoryPanel` | `GET /api/agents/memory` → per-persona `agent_memory(id, 8)`; `DELETE /api/agents/memory/{agent_key}` | **Yes**, DB-backed (`http.rs:804-840`; read path `execution.rs:880`; write path `:3017-3020`) | Role-level, **max 8 rows per role**, no write/edit in UI, no per-hive/per-task scope, clear has no confirmation. |
| `AgentMessagesPanel` | `GET /api/agents/runs/{run_id}/messages`; refetch keyed to `run?.id` + `agentMessageVersion` | **Yes**, DB-backed (store.rs:526-590; `http.rs:674-701`), produced only by `ask_parent`/`send_to_sibling` | Read-only mailbox; no operator compose; no thread grouping; no notification; only exists for the current run; no filter by agent. |

### 4.2 Evidence details

**Profiles**
- Panel shape/decoder: `web/src/panels/AgentProfilesPanel.tsx:10-32` (row shape, example JSON),
  `:34-86` (fetch + decode, direct `fetch`, no AbortController), `:88-131` (import as JSON POST),
  `:133-190` (delete + textarea + render).
- Server truth: `http.rs:782-802` returns id/display_name/tier/template_id/allowed_tool_ids/
  output_schema_id/refusal_policy/max_children/max_depth; persisted-persona merge `:137-180`;
  import/delete `:704-780`. Live probe returned 7 rows. **VERIFIED**
- **Stale built-in classification:** UI hard-codes `BUILTIN_IDS = {planner, content-writer,
  data-analyst, code-reviewer, chatbot}` (`AgentProfilesPanel.tsx:23`), while the live server ships 7
  personas including `superagent` and `subagent` (`personas.json`; live probe). The panel therefore
  classifies the two HMARS-critical personas as non-builtin. **VERIFIED** (set read + live probe);
  visible consequence depends on the delete-branch logic in `:133-167` (not fully read) — **INFERRED**.
- Import/delete are real server mutations, but the UI treats them as a local text area; there is no
  schema preview beyond server refusal, and no run picks up a profile by id.

**Memory**
- Read path is role-keyed and capped: `store.agent_memory(&task.role, 8)` at `execution.rs:880-890`;
  facts are injected into the task's scoped context as `memory:<key>` (`:882`). **VERIFIED**
- Write path: after a successful task, non-empty safe text is appended via
  `store.record_agent_memory` (`execution.rs:3017-3020`); secrets are refused by
  `memory_text_is_safe` (`:61-68`). **VERIFIED at anchors**
- API: `GET /api/agents/memory` loops every persona and returns up to 8 rows each (`http.rs:804-822`);
  `DELETE` clears by agent key (`:824-840`). Panel: `AgentMemoryPanel.tsx:20-125`. **VERIFIED**
- There is no API or UI to write/update memory, choose a scope (hive/task/role), inspect provenance, or
  pin/evict. "Memory banks" in the HMARS sense do not exist.

**Messages**
- Persisted rows: `store.rs:526-560` (`record_agent_message`), `:560-570` (count), `:578-590`
  (list ordered by `created_at, rowid`); route `http.rs:674-701`. **VERIFIED**
- Producers are only the model tools: `ask_parent` (`execution.rs:2416-2487`) and
  `send_to_sibling` (`:2489-2609`); same-parent sibling only (`:2489-2493`); cross-branch sibling
  requires the approval lane (`:1688-1721`). Caps: `MAX_AGENT_MESSAGES_PER_RUN` (`:70`). **VERIFIED**
- UI: `AgentMessagesPanel.tsx:21-110`; it shows `kind · from8 → to8` plus text, no timestamps, no
  threading, no agent names, no operator input. Live `AGENT_MESSAGE` only bumps a version counter
  (`store.ts:1494-1497`) and causes a full refetch (`AgentMessagesPanel.tsx:27-84`). **VERIFIED**
- Nothing in the UI shows the *cause* (the tool call/approval) of a message, and no UI can send one.

---

## 5. Q4 — Provider model / balance / usage UI, and its loading/error/refusal states

### 5.1 Models panel, block by block (VERIFIED, `web/src/panels/ModelsPanel.tsx`)

| Block | Data | Behavior / states |
|---|---|---|
| Provider + model | store `settings` from `SETTINGS_STATUS_RESULT` (`store.ts:1631-1645`); probe on mount if `status==='unknown'` (`ModelsPanel.tsx:62-66`) | If both null: "Probing provider status…" or "has not been reported yet; the account data below is live" (`:160-165`). No key-present line here. |
| Usage | `GET /api/usage/runs/{run_id}` on `[runId, runStatus]` (`:105-155`) | 5 rows: prompt/completion/total/cache-hit-miss/reasoning; null → "Not reported" (`:44-46`, `:174-189`); usage error → "Usage unavailable; no token count invented." (`:190-192`); no run → "No token usage reported for this run yet." (`:193-195`). No cost, no per-agent, no live refresh. |
| Provider account | `Promise.all(GET /api/provider/models, GET /api/provider/balance)` on mount + manual Refresh (`:68-103`, `:199-207`) | Loading text "Reading provider account…" (`:208-209`); error → role=alert with `code: message` from `providerFailure` (`:33-42`, `:210-213`); balance rows or "No balance rows reported." (`:221-232`); `is_available===false` → "Account balance is insufficient." (`:218-220`); available models list (`:234-251`). |
| Contract note | static | `STRINGS.models.contractNote` (`:257`) |

### 5.2 The missing controls

- **No model selector and no thinking/reasoning selector anywhere.** Settings displays
  `provider/model/thinking/reasoning_effort/key_present` in a `<dl>` (`SettingsPanel.tsx:64-77`);
  the only editable setting is reduced-motion (`:106-115`). Values come from environment:
  `config.rs:25-35` defines `HIVEROUTER_MODEL`, `HIVEROUTER_THINKING`, `HIVEROUTER_LEAF_THINKING`,
  `HIVEROUTER_REASONING_EFFORT`, output-token envs; parsing at `config.rs:77-129`. **VERIFIED**
- There is **no route or WS command to mutate provider settings**: HTTP routes are exactly
  history/agents/usage/projects/provider/health/ws (`http.rs:512-529`); frozen command names are
  `SUBMIT_PROMPT, SPAWN_SUB_HIVE, REWIRE_TASK, CANCEL_RUN, KILL_WORKER, APPROVAL_RESOLVE,
  SETTINGS_STATUS, FILE_TREE_LIST, FILE_READ, CHAT_PIN, TEMPLATE_INSERT, HISTORY_REVISION_SET,
  VIEWPORT_SET, VIEWPORT_RECENTERE` (`contract.ts:119-135`; `SETTINGS_STATUS` is read-only).
  **VERIFIED**
- The catalogue marks support with a hard-coded two-id set `{deepseek-flash, deepseek-v4-pro}`
  (`ModelsPanel.tsx:31`, rendered `:242`); catalogue drift would label every model `NOT ACCEPTED`
  even if the wire supports it. **VERIFIED**

### 5.3 Provider routes — safety and behavior (VERIFIED, `http.rs`)

- `GET /api/provider/models` calls `DeepSeekClient::from_env` server-side and proxies only public
  metadata; `Cache-Control: no-store`; the key never reaches the browser (`http.rs:877-901`).
- `GET /api/provider/balance` same proxy (`:903-924`); `provider_origin_allowed` rejects non-allowlisted
  browser origins (`:926-935`); provider errors map to HTTP statuses (`:937-945`).
- These routes issue a **real external provider call on every view/refresh** — they are read-only
  account metadata, but they are not free of network/rate-limit coupling. No spend gate ties
  `balance.is_available` to the composer: submitting a prompt while balance is insufficient still
  sends `SUBMIT_PROMPT`; the only pre-send guard is the provider token budget in the executor
  (`provider_executor.rs:89-130` budget preflight). **VERIFIED path; spend-gate absence INFERRED.**
- `run_usage` returns `usage:null` honestly when no row exists (`http.rs:842-875`); the DB persists
  NULLs, not zeros (`store.rs:678-700`; test comment at `store.rs:1117-1130`). **VERIFIED**

### 5.4 Refusal/error rendering (UI)

- Run refusals: named toast with `code / message / how_to_fix` (`ToastRegion.tsx:7-49`).
- Chat run-level error: `RUN_STATUS.error_code` → `clientRefusal(...)` → "Run refused" block
  (`ChatCore.tsx:88-90`, `:211-221`; store `store.ts:1377-1409`).
- Settings refusal: inline alert with code/message/how_to_fix (`SettingsPanel.tsx:50-63`).
- Data/file refusal: `ErrorState` + Refresh (`DataPanel.tsx:39-57`).
- Approval refusal: classified and rendered with Retry/Dismiss **only** inside the resolve-failure
  block (`ApprovalOverlay.tsx:303-332`; see §7.5).
- Provider failure: string `code: message` in the account block (`ModelsPanel.tsx:33-42`, `:210-213`);
  usage failure is a generic sentence (`:190-192`).

---

## 6. Q5 — Dead / missing / misleading controls (actual state)

### 6.1 Command inventory reality

- 14 frozen command names (`contract.ts:119-135`), 12 of which go over WS (`contract.ts:137-150`);
  3 decision-gated `DIFF_*` are contract-only and **not implemented** (`contract.ts:152-157`).
- UI actually dispatches: `SUBMIT_PROMPT` (`store.ts:1847`), `SPAWN_SUB_HIVE` (`:1871`),
  `REWIRE_TASK` (`:1919`), `CANCEL_RUN` (`:1935`), `KILL_WORKER` (`:1949`), `APPROVAL_RESOLVE`
  (`:2000`), `SETTINGS_STATUS` (`:2208`), `FILE_TREE_LIST` (`:2247`), `FILE_READ` (`:2267`),
  `CHAT_PIN` (`:2104`), `TEMPLATE_INSERT` (`:2111`), `HISTORY_REVISION_SET` (`:2137`).
  `VIEWPORT_SET/VIEWPORT_RECENTER` are client-only by contract. **VERIFIED**
- **`CHAT_PIN` has no UI control.** Store has `pinChat` (`store.ts:2103-2106`) and default
  `pinned:true` (`:1765`), but ChatCore's header only renders a close button (`ChatCore.tsx:199-208`);
  no pin/unpin button exists in `web/src` (grep). Strings `pinned/pin/unpin` are unused. **VERIFIED**

### 6.2 Control matrix

| Control / capability | Reality | Evidence | Label |
|---|---|---|---|
| Submit prompt | Real WS command; front-end ACK/refusal settlement with draft rollback | `ChatCore.tsx:132-175`; `store.ts:1835-1848` | VERIFIED |
| Stop (chat square) | Real `cancelRun`, but **disabled unless `streamingRunId !== null || runStatus === 'running'`** — so it is disabled during `planning`, `thinking`, `queued`, `awaiting_approval`, and `streaming` before the first delta | `ChatCore.tsx:339-351`; run states `agentState.ts:9-26`; `isRunActive` `:121-123` | VERIFIED |
| Cancel (ControlBar) | Enabled for any active run (`isRunActive`), sends `CANCEL_RUN`; store pushes `E_RUN_TERMINAL` for dead runs | `ControlBar.tsx:25-38`, `:97-106`; `store.ts:1929-1936` | VERIFIED |
| Spawn sub-hive | Real `SPAWN_SUB_HIVE`; enabled only connected+active+no pending+not replaying; parent = selected, else hyperagent, else first task; role computed from parent kind and sent Title-Case | `ControlBar.tsx:33`, `:58-68`; `store.ts:1850-1872` | VERIFIED |
| Re-route | Real `REWIRE_TASK` with server rev + CAS refusal handling; requires selected worker + target; primary path is the ControlBar select, plus a server-rev fetch fallback | `ControlBar.tsx:34-38`, `:70-95`; `store.ts:1874-1927` | VERIFIED |
| Kill worker | Real `KILL_WORKER`; enabled only selected `kind==='worker'` on active run | `ControlBar.tsx:37-38`, `:107-116`; `store.ts:1938-1950` | VERIFIED |
| Approve / Deny | Real `APPROVAL_RESOLVE` with nonce/TTL fail-closed gate; disabled while resolving/expired/missing nonce | `ApprovalOverlay.tsx:164-364`; `approval.ts` (`approvalActionsEnabled`) | VERIFIED |
| Retry approval | Real store retry (bounded), rendered **only when the previous resolve produced a refusal** | `ApprovalOverlay.tsx:303-320`; `store.ts:2051-2077` | VERIFIED |
| Dismiss approval | Store action exists, but the only Dismiss button is inside the resolve-failure block; an expired/stale/unconfirmed approval with no refusal record has **no Dismiss** | `ApprovalOverlay.tsx:321-329`; `store.ts:2079-2085` | VERIFIED |
| Approval argument evidence | Missing: card shows tool + `args_sha256` (12 chars), ids, expiry; no path/command/text preview | `ApprovalOverlay.tsx:24-34`, `:288-301`; `contract.ts` approval payload has no preview field | VERIFIED |
| Approval keyboard/a11y | No focus trap, initial focus, or Escape handling in the overlay source; prior audit independently flagged the same | `ApprovalOverlay.tsx:164-364` (no keydown handler); READ `docs/trace/provider/03-ui-actions.md:386` | VERIFIED source; prior READ |
| Model selection | **Missing** (env-only; read-only display) | `SettingsPanel.tsx:64-77`; `config.rs:25-35`, `:77-129`; routes `http.rs:512-529` | VERIFIED |
| Thinking/reasoning selection | **Missing** (same) | same | VERIFIED |
| Spend gate / budget UI | **Missing**: balance is a passive card; composer never checks it; no per-run cost cap shown | `ModelsPanel.tsx:196-232`; `ChatCore.tsx:319-361`; `provider_executor.rs:89-130` (token budget only) | VERIFIED absence; no cost INFERRED |
| Agent profile selection for a run | **Missing**: profiles can be imported/deleted but not chosen; `SUBMIT_PROMPT` carries only text+project_id | `contract.ts:65-118` payloads; `AgentProfilesPanel.tsx`; `store.ts:1847` | VERIFIED |
| Memory write / scope / edit | **Missing**: read + clear only; role-level; server caps 8 rows | `AgentMemoryPanel.tsx`; `http.rs:804-840`; `execution.rs:880`, `:3017-3020` | VERIFIED |
| Feature/profile toggle (flags) | No UI; `side_effects_enabled` is server env/config (`config.rs:355`) | `diagnose` exposes it (`http.rs:535-557`) | VERIFIED |
| Topology editing (beyond spawn/rewire) | **Missing**: AgentHierarchy is read-only; no drag-to-rewire, no edge editor | `AgentHierarchy.tsx:21-81` | VERIFIED |
| A2A operator message ("message an agent") | **Missing**: mailbox read-only; no compose; no operator→agent route/command | `AgentMessagesPanel.tsx`; `contract.ts:119-157` | VERIFIED |
| Swarm action panel | **Missing**: Swarm tab has counts + read-only hierarchy only; ControlBar is mounted only on `nav==='workspace' && tab==='workspace'` | `SwarmPanel.tsx:13-151`; `App.tsx:125-130` | VERIFIED |
| Projects create/rename/switch | **Missing**: panel is deliberately read-only; current project is a localStorage UUID constant name | `ProjectsPanel.tsx:1-81`; `store.ts:333-362` | VERIFIED |
| Run retry after failure | **Missing**: no retry command/control; prior audit flagged it | `contract.ts:119-157`; READ `docs/trace/provider/03-ui-actions.md:308` | VERIFIED absence |
| Template insert | Real `TEMPLATE_INSERT` that inserts a template DAG into the **active run**; UI offers templates without a run-state precondition, so clicks at idle surface a named refusal (not silent) | `LibraryPanel.tsx:8-13`; `store.ts:2110-2117`; `ws.rs:797-831` | VERIFIED |
| Replay/time travel | Real committed-revision reads + scrubber; **but** replay tasks are rebuilt with `kind:'worker'` and synthetic `task xxxxxxxx` labels for every node | `Canvas.tsx:266-275`; `graph.ts:196-234` | VERIFIED |
| Dev gallery | Dev-only (`import.meta.env.DEV`), renders null in production | `DevGallery.tsx:1-25` | VERIFIED |
| Diff proposal/apply/reject | Decision-gated, contract-only; no UI or handler | `contract.ts:152-157` | VERIFIED |

### 6.3 Dead-code / unreachable visuals

1. `HyperAgentNode` — implemented (`nodes.tsx:245-267`) and registered, but filtered out of live nodes
   (`graph.ts:143-165`), so never rendered. **VERIFIED**
2. `BrokenRouteNode` / kind `error` — card implemented (`nodes.tsx:226-243`) but the backend cannot emit
   `kind:"error"` (`execution.rs:97-106`); store defaults omitted kinds to `worker`
   (`store.ts:1447-1450`). Broken-route visuals are reachable only through `data.broken` edge/state
   styling (`graph.ts:180-185`), not through the card. **VERIFIED**
3. `ToolAgentNode` — implemented (`nodes.tsx:196-224`) but the normal planner never emits tool-tier
   roles (`provider_executor.rs:25-29`; `task_agent_tier` `execution.rs:157-158`), so the tool card is
   effectively design-only; tool work is the activity list. **VERIFIED**
4. `CHAT_PIN` — store method with no control (`store.ts:2103-2106`; no `pinChat` caller in
   components/canvas — grep). **VERIFIED**
5. `canResolveApproval` / `prunePendingResolutions` — exported by `ApprovalOverlay.tsx:111-143` and
   pinned by tests, but **not imported or called by the component** (component uses
   `approvalActionsEnabled`, `ApprovalOverlay.tsx:18`, `:192-194`); they are test-only helpers, not
   dead code per the test double rule, but the overlay header comment still describes a panel-side
   pending lock that no longer exists (`ApprovalOverlay.tsx:9-13`). **VERIFIED**
6. `formatCount(null)` and `formatCtx(null)` paths keep Tool/Worker cards honest but permanently `—`
   (`nodes.tsx:189`, `:216-219`, `:237-238`). **VERIFIED**

---

## 7. Q6 — Design-source parity with FigCraft boards

### 7.1 Board inventory and what it covers (READ: `docs/ui/figcraft/HiveMind/README.md`)

| # | Board | App surface today | Parity status |
|---|---|---|---|
| 00–04 | Cover, Colour, Typography, Space/Depth, Brand lockups | design-system only; Header uses an inline SVG hex mark, not the FigCraft brand export (`Header.tsx:20-34`; `strings.ts` assets gate says exports are pending UI-watchdog allowlist) | tokens/colors only; brand exports unused (READ) |
| 05 | Agent Cards | SuperAgent/WorkerAgent/ToolAgent/BrokenRoute cards exist with the board's exact sizes; SuperAgent header 40 px + 68×18 pill matches; but labels are fallbacks and Tool/Error/Hyper are unreachable in normal runs | **geometry matches** (`tokens.ts:166-179`; `nodes.tsx:141-243`; `designParity.test.ts:15-26`), **anatomy/liveness partial** |
| 06 | Controls & HUD | ControlBar + HUD + minimap exist; HUD is 460×44 and shows SUPERAGENTS/WORKERS/GREEN with source label; controls are gated by run state | **present**, but controls only on Workspace tab (`App.tsx:125-130`), no running-run capture this pass (UNVERIFIED visually) |
| 07 | Spatial Canvas (authoritative composition) | 52 px topbar, 72 px rail, 240 px library, fixed chat, HUD, minimap 220×150, card sizes | **matches except chat**: token says 400×480 (`tokens.ts:170`, pinned by `designParity.test.ts:20`; ChatCore renders `h-[480px] w-[min(400px,92%)]` `ChatCore.tsx:181`), while README says board 07 remains **420×500** (`README.md:54-56`) and board 11 shows the 400×480 app variant (`HiveMind-ASSETS.json:12`). This is a **declared divergence**, not necessarily a defect — but it is not board-07 parity. |
| 08 | HMAS Hierarchy (PROPOSED-ADDITION) | "Swarm" = 5 counters + read-only indented task rows + projects card (`SwarmPanel.tsx:13-151`; `AgentHierarchy.tsx:55-81`); no edge view, no root summary ("root · N superagents · healthy"), no state legend/actions | **not built** as a board surface |
| 09 | HMAS Governance (PROPOSED) | Approval overlay exists and is wired (`ApprovalOverlay.tsx`), but consent shows tool+hash only; no governance feed/ledger, no risk classification, no "who asked whom" context | **partial, newly reachable**; board semantics missing |
| 10 | States & Refusals | 16-state vocabulary with dot/shape/text cues (`agentState.ts`), named refusals + toasts (`ToastRegion`), `ErrorState`/`EmptyState`/`LoadingState` (`StatePack.tsx`) | **strong** on vocabulary; micro-label floor still 8 px (prior measured audit P2, READ `05:148`) |
| 11 | Motion & Live (PROPOSED) | `motion.css`, reduced-motion handling, streaming pulse; chat variant 400×480 matches this board | **present, unmeasured**; durations/easing are design proposals (README:54-55) |
| 12 | Run Detail & Files | Data & Files (`DataPanel`), tool activity (Agents panel + chat), run detail (no dedicated surface), failure/refusal ledger (no), model status/usage (`ModelsPanel`), settings (`SettingsPanel`) | **partial**: no run-detail/ledger surface; tool activity is under Agents, not run detail; Models/Settings exist |

### 7.2 What the parity test actually pins (VERIFIED)

`web/tests/designParity.test.ts:15-26` pins `LAYOUT.topbar=52, navRail=72, library=240,
chat={400,480,radius:18}, hud={460,44}, minimap={220,150}, superagent={420,150,14},
worker={320,96,12}, tool={420,240,14}` and `:28-33` pins those constants' use in
`graph.ts`, `Canvas.tsx`, `LibraryPanel.tsx`, `Header.tsx`. The test **does not** pin the
error card (320×110) or the chat-vs-board-07 divergence, and it does not test feature presence.

### 7.3 Measured prior audit (READ, older build — not re-measured)

`docs/trace/provider/05-figcraft-ux-gaps.md` (239 lines) reports at 2026-11-07 18:12 build:
**P0 = 0, P1 = 5, P2 = 11**, no pixel-parity claim, measured at `127.0.0.1:8099`. Its P1 items
include the missing argument preview/governance context, Swarm gaps (no legend, no root row, no
actions), Models layout gap, and micro-label sizes. Since then (`bd575ba` and the uncommitted
approval work) some code changed — e.g. the approval overlay and agent profile/memory/message panels
now exist — so `05`'s severity register is **stale**; a fresh measured pass is required before
claiming a P0/P1 count. **UNVERIFIED** which P1s remain visually.

### 7.4 Parity verdict

- **Style/geometry parity:** strong where pinned (colors, radii, card sizes, chrome sizes); one
  declared chat divergence (400×480 vs 420×500) and a minimap caption conflict in the boards
  (README:48-50 says board 07 uses 220×150; older board captions differ). No pixel claim.
- **HMARS feature parity:** weak. Boards 08/09 describe governance/hierarchy surfaces that the app
  does not implement as surfaces; board 12's run-detail/ledger is absent; the app's real HMAS content
  (profiles, memory, mailbox) lives as four stacked read-only panels under **Agents**, not in the
  canvas/swarm composition.

---

## 8. Q7 — Exact UI changes needed for a true HMARS view

Ordered by dependency; **backend additions are required** and should be contract-reviewed before UI.

### P0 — identity and governance (HMARS cannot be truthful without these)

1. **Carry agent identity on the wire.** Extend the additive `NodeEventPayload` with
   `role` / `persona_id` (and `template_id`) and populate it in `publish_node_spawn` /
   `publish_node` (`backend/crates/hiverouter-core/src/execution.rs:743-779`). Add `personaId` to
   `TaskRecord` (`web/src/state/store.ts` type near `:290`; normalized at `:1441-1478`). Render
   persona name + role on SuperAgent/Worker cards (`web/src/flow/nodes.tsx:141-194`), hierarchy rows
   (`AgentHierarchy.tsx:55-81`), and the Agents list (`AgentsPanel.tsx:40-64`).
2. **Fix persona lookup + continuation.** Normalize role keys (`task.role.trim().to_ascii_lowercase()`)
   in `system_prompt_for` (`provider_executor.rs:82-87`), map `worker` to a real worker persona (or
   give the planner a configurable persona id), and use `self.system_prompt_for(task)` in
   `continue_with_tools` instead of `TASK_SYSTEM` (`provider_executor.rs:369-372`). Add a test that a
   planner worker and a UI-spawned `Subagent`/`SuperAgent` each receive their persona system prompt on
   **every** turn.
3. **Make Subagent a first-class kind (or carry the role and render it).** Contract-additive
   `NODE_KIND_KEYS += 'subagent'` (`agentState.ts:126-135`), backend `node_kind()` arm
   (`execution.rs:97-106`), a Subagent card (or role-badged Worker card), hierarchy/row labels, and an
   explicit Subagent spawn control with role `subagent`. Add its tier/memory semantics only after the
   contract review.
4. **Approval evidence + recovery.** Contract-additive approval `preview`/`args_summary`/target
   (redacted, bounded) so the overlay can show *what will happen*, plus from/to task labels; render
   it in `ApprovalOverlay.tsx` (currently tool + 12-char hash only). Provide Dismiss/Retry for
   expired/stale/unconfirmed states (move the controls outside the `resolveState.refusal` block,
   `ApprovalOverlay.tsx:303-332`), add focus trap/initial focus/Escape, and an approval history
   (governance ledger) for board 09.
5. **Operator-selectable model / thinking / budget (or honest boot-only labeling).** Add a runtime
   settings command/route (new frozen-additive WS command or HTTP `PUT /api/settings`) backed by a
   mutable provider config, then a model/thinking selector in Settings with the live catalogue and a
   visible "applies to next run" state. Until then, label the values "env-configured at boot" instead
   of presenting them as current controls. Files: `SettingsPanel.tsx:64-77`, `ModelsPanel.tsx`,
   `config.rs:77-129`; server `http.rs:512-529`/`ws.rs`.
6. **Stop must work for the whole active run.** Replace the ChatCore Stop predicate with
   `isRunActive(run.status)` (or store helper) so planning/thinking/queued/awaiting_approval are
   cancellable (`ChatCore.tsx:339-351`; store `cancelRun` `:1929-1936`). Keep the ControlBar Cancel as
   the second path. Add a "cancelling…" status until terminal.

### P1 — the HMAS surfaces themselves

7. **Swarm/topology panel.** Add the action row to the Swarm tab (spawn/rewire/kill/stop) or a
   dedicated topology toolbar, plus board-08 requirements: root summary row, state shapes legend,
   branch focus, edge count, and a per-node selection inspector. Files: `SwarmPanel.tsx:13-151`,
   `App.tsx:125-130`, `AgentHierarchy.tsx`, `ControlBar.tsx`.
8. **A2A mailbox + operator messaging.** Thread view grouped by `(from,to,kind)`, timestamps, the tool
   call/approval that caused the message, and an operator compose (new command/route with its own
   authorization + caps). Files: `AgentMessagesPanel.tsx`, `http.rs:674-701`, store `AGENT_MESSAGE`
   handler (`store.ts:1494-1497`), `execution.rs:2416-2609`.
9. **Memory banks.** Scope memory to `(hive/task/role)`, show provenance and last-used, allow
   write/edit/evict with confirmation and audit; extend the memory routes (`http.rs:804-840`) and the
   seed/write path (`execution.rs:880`, `:3017-3020`); UI in `AgentMemoryPanel.tsx`.
10. **Run detail surface (board 12).** A dedicated Run detail panel: run id/status/error code,
    refusal ledger derived from actual refusals, tool activity (move/duplicate the full
    `ToolActivityPanel` there), per-run usage/cost, revisions, and links to files. Reuse
    `ModelsPanel`'s usage block and `HistoryPanel` data.
11. **Projects lifecycle.** Add create/switch/rename (server routes + confirmation), and stop using a
    localStorage-only project id/name (`store.ts:333-362`, `ProjectsPanel.tsx`, `http.rs:947-960`).
    Guard template insertion when no run is active (or disable with an explanation).
12. **Spend/usage truth.** Poll usage during a run (or subscribe to a usage event), show settled cost
    when the provider reports it, and gate/flag submissions when `is_available === false`. Keep
    NULL→"Not reported" honesty (`ModelsPanel.tsx:44-46`, `:174-195`).

### P2 — correctness/honesty cleanups before calling the view "true"

13. **Replay fidelity.** Extend committed snapshots/HISTORY_SNAPSHOT with `kind` (and role/label where
    available) and stop forcing `kind:'worker'`/`task <id>` in `buildReplayTasks`
    (`graph.ts:196-234`); otherwise time travel is visually false for SuperAgents/Subagents.
14. **Node telemetry.** Fill `label` (semantic task names), and `progress`/metrics from measured
    values or remove the progress bars; current cards permanently show `—`
    (`execution.rs:764-779`; `nodes.tsx:183-189`, `:216-219`). Also fix HUD wording: "GREEN" is
    completed/total, not a health score (`execution.rs:653-695`).
15. **Dead visuals.** Either make `error`/`tool` kinds reachable (failure = error card; ToolAgent tasks)
    or remove the cards from the renderer map; remove the dead HyperAgent card or use it in the
    hierarchy; remove/implement `CHAT_PIN`; delete the stale overlay comment/legacy test-only helpers
    or re-wire them (`ApprovalOverlay.tsx:9-13`, `:111-143`).
16. **Panel hygiene.** Add abort/cancel + cleanup to profile/memory/messages fetches, confirmation for
    destructive clears, and error boundaries; currently several handlers swallow errors into local
    states (`AgentProfilesPanel.tsx:78-81`, `:94-130`; `AgentMemoryPanel.tsx:65-84`;
    `AgentMessagesPanel.tsx:75-80`). These are honest but non-retryable in place.
17. **Design parity decisions.** Resolve chat 400×480 vs board-07 420×500 in one authoritative place
    (README/ASSETS/tokens/test), align minimap/library caption conflicts, and raise 8 px micro-labels
    to the board-05 9.5 px floor. Update `designParity.test.ts` to pin the decision.

---

## 9. Twenty-question Q&A (HMARS UI)

1. **Does the chat box start a real run?** Yes — `ChatCore.submit` → `SUBMIT_PROMPT` →
   `ws.rs:744` → `execution.rs:1016`; the run row is created before any provider call. **VERIFIED**
2. **Is the HyperAgent a visible agent?** Only as the fixed chat overlay + root hierarchy row; the
   canvas filters `kind:'hyperagent'` and adds an invisible anchor. **VERIFIED** (`graph.ts:151`,
   `Canvas.tsx:167-189`, `ChatCore.tsx:179-183`).
3. **Is there a SuperAgent card?** Yes, and it renders when the planner emits a `superagent` role;
   it shows a fallback label, a static role caption, and `—` metrics. **VERIFIED/INFERRED**
   (`nodes.tsx:141-173`; `execution.rs:764-779`).
4. **How is a Subagent shown?** As a WorkerAgent card/row; no kind, label, or icon distinguishes it.
   **VERIFIED** (`execution.rs:151-162`, `:97-106`; `nodes.tsx:175-194`).
5. **Can the operator see tool calls?** Yes, two places: full activity list under Agents and last-4
   "ACTIONS" in chat; args are only a truncated hash and text ≤160 chars. **VERIFIED**
   (`ToolActivityPanel.tsx:131-184`; `ChatCore.tsx:274-308`).
6. **Are approvals real?** Yes: engine suspends, arms a waiter, publishes `APPROVAL_REQUEST`, and
   resolves/TTLs/cancels; the overlay sends `APPROVAL_RESOLVE` with a nonce. **VERIFIED**
   (`execution.rs:1998-2165`; `ApprovalOverlay.tsx:164-364`).
7. **Does Approve execute the side effect?** Yes — after grant verification the tools crate commits
   one preflighted side effect (atomic file write / allowlisted shell); absent approval bridge or jail
   it refuses instead of running. **VERIFIED path / INFERRED end-to-end** (`execution.rs:2009-2022`;
   `side_effects.rs:1-13`, `:103`, `:163`).
8. **Do agent profiles affect runs?** Only as persona system prompts when `task.role` exactly equals a
   persona id; profiles cannot be selected for a run, and worker/Title-Case roles miss. **VERIFIED
   lookup; INFERRED effect** (`http.rs:254-275`; `provider_executor.rs:82-87`; `store.ts:1865-1871`).
9. **Does memory affect prompts?** Yes — up to 8 role-level rows are injected as `memory:<key>` facts
   at task creation; new safe text is recorded after success. Not per hive/task. **VERIFIED**
   (`execution.rs:860-890`, `:3017-3020`).
10. **Do agent-to-agent messages appear live?** The `AGENT_MESSAGE` event bumps a version and the
    mailbox panel refetches; there is no toast, inbox, or transcript entry. **VERIFIED**
    (`store.ts:1494-1497`; `AgentMessagesPanel.tsx:27-84`).
11. **Can the operator choose the model?** No. Model/thinking/reasoning/effort are env-only and shown
    read-only; there is no mutation route/command. **VERIFIED** (`SettingsPanel.tsx:64-77`;
    `config.rs:25-35`, `:77-129`; `http.rs:512-529`).
12. **Is balance live?** It is fetched from the provider account on Models mount/refresh; a manual
    Refresh exists; it is not automatically refreshed and does not gate submissions. **VERIFIED**
    (`ModelsPanel.tsx:68-103`, `:196-232`; `http.rs:903-924`).
13. **Is usage shown?** Yes, per-run token counters from `run_usage` (6 fields incl cache hit/miss,
    reasoning), NULL→"Not reported", refetched only when run id/status changes; no USD/cost or
    per-agent split. **VERIFIED** (`ModelsPanel.tsx:105-195`; `http.rs:842-875`; `store.rs:678-730`).
14. **Can the operator stop a run?** Yes via ControlBar Cancel; the chat Stop is disabled during
    planning/thinking/pre-delta. Both send `CANCEL_RUN`. **VERIFIED** (`ControlBar.tsx:97-106`;
    `ChatCore.tsx:339-351`; `store.ts:1929-1936`).
15. **Can the operator spawn/rewire/kill?** Yes from the Workspace tab's ControlBar; commands are
    real and gated by connection/run/pending/selection; roles are sent as display strings. **VERIFIED**
    (`ControlBar.tsx:25-116`; `store.ts:1850-1950`).
16. **Can the operator manage topology or A2A?** No: hierarchy is read-only, mailbox is read-only,
    there is no operator→agent message route. **VERIFIED** (`AgentHierarchy.tsx`; `contract.ts:119-157`).
17. **Is history/replay real?** Yes — committed revisions + snapshots via HTTP and a scrubber; but
    replay forces every node to `worker` with synthetic labels, so semantics are not preserved.
    **VERIFIED** (`store.ts:418-560`, `:1020-1180`; `graph.ts:196-234`).
18. **Are errors shown honestly?** Yes at run level (code/message/how_to_fix banner + named toasts);
    per-node error codes only surface on the unreachable broken-route card. **VERIFIED**
    (`ChatCore.tsx:211-221`; `ToastRegion.tsx`; `store.ts:1470`; `execution.rs:97-106`).
19. **Are files/data real?** Yes — jailed `FILE_TREE_LIST`/`FILE_READ` with truncation/sha display,
    refusal states, and a refresh path; no mock data. **VERIFIED** (`DataPanel.tsx:12-122`;
    `http.rs:512-529`; store `:2245-2270`).
20. **Does it match the FigCraft boards?** Geometry/token parity is pinned for the main composition
    (except the declared 400×480 chat variant); feature parity for boards 08/09/12 is not built, and
    the prior measured audit's severity register is stale relative to this tree. **READ/UNVERIFIED**
    (`designParity.test.ts:15-33`; `README.md:23-58`; `docs/trace/provider/05-figcraft-ux-gaps.md`).

---

## 10. Top 10 findings (HMARS priority order)

1. **Persona identity is lost on the production paths that create agents.** Planner-created workers
   have role `worker` (no persona key); UI-spawned children use `SuperAgent`/`WorkerAgent`/`Subagent`
   while the persona map is lowercase-id keyed; tool continuation hard-codes `TASK_SYSTEM`. Net: most
   leaves and every UI-spawned child run the generic prompt, and only turn 1 of a tool-using task has
   any persona. `provider_executor.rs:25-29`, `:82-87`, `:369-372`; `store.ts:1865-1871`;
   `strings.ts:275-278`; `personas.json`. **VERIFIED path; INFERRED effect.**
2. **No runtime configuration surface for the thing HMARS is:** model, thinking/reasoning, agent
   profile, and spend are not selectable. Values are env-only/read-only; routes cannot mutate them.
   `SettingsPanel.tsx:64-77`; `config.rs:25-35`; `http.rs:512-529`; `contract.ts:119-157`. **VERIFIED**
3. **Subagent has no visual or structural identity.** `subagent` is only a spawn role; tier mapping
   falls through to WorkerAgent, so the UI cannot tell a Subagent from a Worker, and no Subagent
   memory/card/kind exists. `execution.rs:151-162`, `:97-106`; `agentState.ts:126-135`. **VERIFIED**
4. **Governance is blind and can trap the operator.** Approvals show tool + truncated hash, no
   args/target/text; expired/stale approvals without a classified refusal have no Dismiss; no focus
   trap/Escape. `ApprovalOverlay.tsx:24-34`, `:288-332`; `store.ts:2079-2085`. **VERIFIED**
5. **Agent memory and A2A are read-only fragments, not HMAS subsystems.** Memory is role-level,
   8-row capped, no write/scope UI; mailbox is read-only, model-tool-only, and only bumps a version
   counter. `http.rs:804-840`; `execution.rs:880`, `:3017-3020`; `store.ts:1494-1497`;
   `AgentMessagesPanel.tsx`. **VERIFIED**
6. **Tool activity is split and buried, and there is no run-detail/refusal-ledger surface.** Full list
   lives under Agents, chat shows last 4, no approval linkage, no per-run failure view.
   `AgentsPanel.tsx:65-72`; `ChatCore.tsx:274-308`; `ToolActivityPanel.tsx`. **VERIFIED**
7. **Graph taxonomy is half dead, and replay is visually false.** HyperAgent card filtered, error/tool
   kinds not producible on the normal path, replay flattens every node to `worker` with
   `task <id>` labels. `graph.ts:143-165`, `:196-234`; `nodes.tsx:245-275`; `execution.rs:97-106`.
   **VERIFIED**
8. **Server never sends label/progress/metrics, so live cards are generic and permanently `—`.**
   `execution.rs:764-779`; `store.ts:325-331`; `nodes.tsx:183-189`, `:216-219`. **VERIFIED**
9. **Provider account/spend is a passive, stale card.** Live external calls on mount/refresh; no
   spend gate in the composer; usage only refetched when run id/status changes and only after the
   terminal persist; no cost. `ModelsPanel.tsx:68-155`, `:196-232`; `http.rs:877-945`;
   `execution.rs:1305`. **VERIFIED**
10. **The HMAS views (topology, swarm actions, projects, pin/DIFF) are read-only or absent.**
    ControlBar exists only on Workspace; Swarm tab is counters + read-only rows; projects read-only;
    `CHAT_PIN` has no control; `DIFF_*` is decision-gated and unimplemented. `App.tsx:125-130`;
    `SwarmPanel.tsx`; `ProjectsPanel.tsx`; `store.ts:2103-2106`; `contract.ts:152-157`. **VERIFIED**

---

## 11. Evidence index (files read this pass, with hashes for drift)

| File | Bytes | SHA-256 (first 16) |
|---|---:|---|
| `web/src/state/store.ts` | 80,400 | `C6F940DF3E3C6AAA` |
| `web/src/chat/ChatCore.tsx` | 16,134 | `0C4E2DD746EA48E1` |
| `web/src/panels/AgentProfilesPanel.tsx` | 7,638 | `2ADA693528B3634F` |
| `web/src/panels/AgentMemoryPanel.tsx` | 4,493 | `3F9F332F54C364DE` |
| `web/src/panels/AgentMessagesPanel.tsx` | 3,800 | `FF9C7787CCA21DB3` |
| `web/src/panels/AgentHierarchy.tsx` | 2,950 | `F671771EFEE74B89` |
| `web/src/panels/ApprovalOverlay.tsx` | 14,541 | `1595C7BECDFFAE89` |
| `web/src/panels/ModelsPanel.tsx` | 11,544 | `D4AC15D4E1243984` |
| `web/src/panels/ToolActivityPanel.tsx` | 7,223 | `11489E87E25CE7C9` |
| `web/src/panels/SwarmPanel.tsx` | 7,782 | `E3D67DD7E86087AA` |
| `web/src/flow/graph.ts` | 8,442 | `543B850DE91EB234` |
| `web/src/flow/nodes.tsx` | 11,311 | `41A5DA7EDCE8CC8C` |
| `web/src/flow/Canvas.tsx` | 19,745 | (read; hash not captured) |
| `web/src/state/agentState.ts` | 4,697 | `4CFC0735A6E03F34` |
| `web/src/tokens/tokens.ts` | 7,977 | `60AFFD1871C396CD` |
| `web/src/api/contract.ts` | 38,086 | `4E66EB111F8B7FB2` |
| `backend/crates/hiverouter-core/src/http.rs` | 41,657 | `0022145821E5909B` |
| `backend/crates/hiverouter-core/src/execution.rs` | 122,838 | `283A8FD7F7AF62DF` |
| `backend/crates/hiverouter-core/src/provider_executor.rs` | 24,104 | `A2BFFC7D12A29A0A` |
| `backend/crates/hiverouter-tools/src/side_effects.rs` | 31,447 | `4094A1B4AC71ACD4` |
| `docs/provider/AGENT-ARCHITECTURE-TRUTH.md` (working tree, modified) | 10,195 | `D1B8B5F860978315` |
| `docs/ui/figcraft/HiveMind/README.md` | 3,364 | `A98F6583A61AFA3F` |
| `docs/trace/provider/05-figcraft-ux-gaps.md` | 27,453 | `0C3145F01B6057C9` |

Also read (excerpts): `web/src/App.tsx`, `panels/{AgentsPanel,HistoryPanel,LibraryPanel,DataPanel,
SettingsPanel,RoutingPanel,ControlBar,Header,Scrubber,ProjectsPanel,StatePack,HudBadge,ToastRegion,
DevGallery,NavRail}.tsx`, `flow/Canvas.tsx`, `state/strings.ts`, `state/metrics.ts`, `api/client.ts`,
`ws.rs`, `store.rs`, `config.rs`, `planner.rs`, `docs/ui/figcraft/HiveMind/HiveMind-ASSETS.json`,
`web/tests/designParity.test.ts`, `web/tests/approval.integration.test.ts`,
`docs/trace/provider/03-ui-actions.md` (headings/quotes), `docs/trace/provider/05-figcraft-ux-gaps.md`.

---

## 12. Not done / limitations (honest)

- **No live browser capture in this pass.** The UI's rendered output during a run was not observed;
  run-time claims are source-derived. A fresh measured pass on the current bundle
  (`assets/index-a4oJkwuC.js`) is required before quoting screenshot-level parity.
- **No build/test execution** (analysis seat constraint): web unit tests were read, not run.
- **No prompt submitted, no provider/account call, no credential read**; the provider UI was not
  exercised with a keyed run in this pass.
- **No app restart**; the already-running `:8099` server was observed read-only (empty project/run DB).
- **HMARS is not a term in the repo.** A recursive docs grep found no `HMARS`/`HMAR` definition; the
  closest authorities are `docs/provider/AGENT-ARCHITECTURE-TRUTH.md` (agent truth), the FigCraft
  HMAS boards 08–10, and `docs/trace/provider/02-agent-runtime-wiring.md`. This report treats HMARS
  as the operator-facing view over the hierarchical runtime described by those files; if HMARS has a
  stricter definition, the delta is UNVERIFIED here.
- **`AGENT-ARCHITECTURE-TRUTH.md` is uncommitted/modified** (10,195 B, `D1B8B5F860978315`); its claim
  that persona prompts apply to tool turns is contradicted by `provider_executor.rs:369-372` and should
  be corrected before it is used as evidence.
- **The tree is moving** (approval work in `store.ts`/`approval.ts`, docs, pycache, desktop exe).
  Re-hash before merging/committing; line numbers may drift.

## 13. Bottom line

Real runtime, honest shell, wrong view. The engine already does the hard part (planning, hierarchical
spawn policy, streaming, bounded tool loop, single-use approvals, side effects, persistence, A2A
mailbox, role memory). The UI renders a disciplined *run monitor*, not an HMARS control surface: the
agents have no identity, the operators have no configuration, the governance has no evidence, the
topology has no editor, and memory/A2A have no authoring path. The P0 list in §8 is the shortest path
to a view that can honestly be called HMARS.
