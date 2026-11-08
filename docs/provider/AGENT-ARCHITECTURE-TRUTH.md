# HiveMind agent architecture — plain-English truth

Date: 2026-11-07. This file separates **VERIFIED (code/runtime read)**, **INFERRED**, and **NOT CODED**.
It answers the operator's questions directly; it is not a design pitch.

## 1. The one-sentence truth

HiveMind **is** a real hierarchical multi-agent runtime: a Rust engine creates a root HyperAgent/planner,
plans a DAG, spawns SuperAgent/WorkerAgent/ToolAgent tasks, streams their DeepSeek replies, gates tools,
and persists runs/messages/memory. It is **not** yet the full "HyperAgent with a conversational
sub-orchestrator in the chat box, per-hive memory banks, 300–600 concurrent agents" architecture.

## 2. What exists, in plain English

| Concept | Is it real? | Where |
|---|---|---|
| HyperAgent | **Yes, as the root planner task** (`role: "planner"`, depth 0), mapped to `AgentTier::HyperAgent`. The fixed chat card is the UI surface for this root, not a separate graph node. | `backend/crates/hiverouter-core/src/execution.rs:97-163`, `:1321-1335`; `web/src/flow/graph.ts` filters the root and anchors it to the hive; `web/src/chat/ChatCore.tsx` |
| SuperAgent | **Tier/policy exists**, and it may spawn workers/subagents and use file/ask/sibling tools. **No dedicated SuperAgent prompt template or persona exists** in the embedded assets. | `execution.rs:108-142`; `backend/crates/hiverouter-tools/src/tool.rs:391-430`; `backend/crates/hiverouter-core/assets/prompts/personas.json` (no `superagent`) |
| WorkerAgent | **Yes.** Four worker personas/templates ship: content-writer, data-analyst, code-reviewer, chatbot. They carry allowed tools/output schema. | `personas.json`; `execution.rs:860-886`; `tool.rs:408-421` |
| Subagent | **Not a separate tier.** The spawn policy accepts role `"subagent"` under a Worker, but `task_agent_tier` maps unknown roles to `WorkerAgent`. There is no `Subagent` enum, prompt, persona, card, or memory type. | `execution.rs:108-142`, `:151-162`; `tool.rs:391-430` |
| ToolAgent | **Yes, as a tier/allowlist** (read/write/shell only, no spawning). | `execution.rs:103-105`; `tool.rs:422-430` |
| Agent creation/templates | **Partially real.** Manifest/persona validation, import/delete, template catalog, `TEMPLATE_INSERT`, and tool allowlists exist. **Leaf execution uses the generic `TASK_SYSTEM` prompt plus role/depth, not the persona's `.prompty` body.** Only the planner template body is rendered. | `prompts.rs`; `execution.rs:860-886`; `provider_executor.rs:task_prompt` / `execute`; `ws.rs` TEMPLATE_INSERT |
| Chat box powered by API | **Yes.** ChatCore → `SUBMIT_PROMPT` WebSocket → engine → DeepSeek Chat Completions planner/leaf calls → `REPLY_DELTA` stream → ChatCore. | `web/src/chat/ChatCore.tsx`; `web/src/state/store.ts`; `backend/.../ws.rs`; `provider_executor.rs` |
| Chat box is a SuperAgent | **No.** It is an operator UI, not an agent. The root HyperAgent/planner is the orchestrator; there is no conversational sub-orchestrator living in the chat surface. | `ChatCore.tsx`; `execution.rs` root/dispatch |
| Agent-to-agent messages | **Yes, bounded.** `ask_parent` and `send_to_sibling` tools persist `agent_messages` and emit events. Cross-parent sibling sends are restricted; there is no global agent directory/pub-sub bus. | `execution.rs:721-780`, `:2433-2592`; `db.rs agent_messages` |
| Event bus | **Yes**, the server event bus fans out WS events (run/node/reply/tool/approval). It is not an agent knowledge bus. | `backend/crates/hiverouter-core/src/bus.rs` |
| Per-task context | **Yes.** Each task gets a `ScopedContext` with parent facts copied as a snapshot; rewire clears the old parent facts. | `context.rs`; `execution.rs:856-886` |
| Persistent agent memory | **Role-level only.** `agent_memory` is keyed by persona/role id; each task loads up to **8** rows at creation. It is not per task, not per SuperAgent hive, and not a vector store. | `db.rs agent_memory`; `execution.rs:880-884` |
| HyperAgent access to all child memories | **No.** The root loads planner/root facts and planner-role memory; there is no aggregate read of child memory banks. | `execution.rs:860-886` |
| Agents know about each other | **Only via explicit parent/sibling tools or the event/UI graph.** There is no shared agent directory in the model context. | `execution.rs` A2A tools; `graph.rs` |
| Per-hive memory bank | **Not coded.** No table/context partition by SuperAgent hive. | `context.rs`, `db.rs` |
| Persistent thought storage | **Not coded as a thought store.** Reasoning is used for tool-turn continuation when present, and messages are persisted; there is no separate CoT archive exposed to agents. | `provider_executor.rs`; `execution.rs` |
| 300–600 agents, 50–150 running | **Impossible under current caps.** Hard caps: **64 agents/run, fan-out 8, depth 3, 40 provider calls/run**. The planner prompt now says 1..8 nodes. | `backend/crates/hiverouter-core/src/caps.rs:9-15`; `provider_executor.rs` planner prompt |

## 3. Actual orchestration path

```text
ChatCore (operator UI)
  → SUBMIT_PROMPT (WS)
  → RunEngine.submit_prompt creates run/root planner task + RUN_CREATED
  → planner call: deepseek-flash, JSON mode, thinking enabled/high
  → WirePlan → Plan::validate (≤8 nodes, depth≤3)
  → spawn root + planned tasks (NODE_SPAWN)
  → leaf executions: streaming Chat Completions
  → tool_calls? → tool turn → TOOL_CALL/TOOL_RESULT → continuation
  → REPLY_DONE / usage / run terminal
```

Evidence: `execution.rs:1016-1054` (submit), `:1309-1369` (plan/spawn), `:2726-2804`
(leaf execute), `:2811-2944` (tool loop), `provider_executor.rs` (requests/stream).

The tool loop was **dead before the 2026-11-07 fix**: the first leaf request omitted
`.with_tools(...)`. A real keyed `file_read` run now completed with `ACTIONS: FILE_READ OK` after the
fix, proving the path is reachable. The new test `provider_adapter.rs:leaf_execute_advertises_tools_on_the_first_request`
pins the request shape.

## 4. Agent/template design situation

- FigCraft board 05 already contains **HyperAgent chat core, SuperAgent, WorkerAgent, ToolAgent and error** cards.
- There is **no distinct Subagent design** because there is no distinct Subagent tier. Subagent is a
  spawn role that lands in the WorkerAgent tier.
- No **SuperAgent persona/template** ships, so the planner can emit a `superagent` task but the engine
  has no super-specific prompt or persona allowlist for it.
- The four worker persona templates are **read/validated/importable and shown in the Agents surface**,
  but their prompt bodies are not applied to leaf model calls; leaf calls use `TASK_SYSTEM` plus role/depth.
- Therefore you do **not** need to pull Figma designs for Hyper/Super/Worker: the FigCraft set is already
  the design source. You **would** need to create a SuperAgent/Subagent design if you want distinct
  visual/capability tiers.

## 5. DeepSeek API coverage

**Wired:** Chat Completions, `deepseek-flash`/`deepseek-v4-pro`, thinking toggle + `reasoning_effort`,
JSON mode, function tools + tool result turns, streaming SSE, usage/cache counters, model list,
account balance, DPAPI key injection, 5xx retry, finish-reason handling, per-parent fan-out.

**Not wired:** Files API, Search, Fetch, Responses API, Anthropic Messages API, FIM/Prefix Beta,
vision images, strict tools, server-side web search. These are documented gaps, not silent features.
The local HMAS does not need Anthropic; DeepSeek-only is correct.

Evidence: `docs/trace/provider/01-deepseek-capability-matrix.md`,
`docs/provider/DEEPSEEK-INTEGRATION.md`, `docs/trace/provider/02-agent-runtime-wiring.md`.

## 6. What should be built next (honest solution ideas)

1. **SuperAgent persona/template** — add a `superagent.prompty` + persona with spawn/file/sibling tools,
   and apply persona prompt bodies to task execution (requires an executor prompt seam).
2. **Subagent as a real tier** — add `AgentTier::Subagent`, its own allowlist/memory/prompt/card, or
   explicitly document that Subagent is a Worker role. Do not add UI cards without a capability model.
3. **Per-hive memory** — add a hive/branch key to `agent_memory` and a read policy for HyperAgent;
   persist a per-hive summary, not raw chain-of-thought.
4. **Agent directory / A2A bus** — expose a bounded roster/topology to agents (not all global memory);
   keep `ask_parent`/`send_to_sibling` as the write path.
5. **Raise concurrency only with caps + cost** — 64 agents is current hard cap; 300–600 requires changing
   `MAX_AGENTS`, fan-out/depth, provider concurrency, budget, DB/event fan-out, UI virtualization, and
   a real scheduler. Rust speed is not the blocker.
6. **UI** — show SuperAgent/Subagent distinctions only after 1/2; show tool results/approvals; add
   per-hive memory/topology views.
7. **DeepSeek Files/Search** — optional; Files is image-oriented, Search/Fetch are separately billed and
   need a money path. Do not wire them into a token-only budget silently.

## 7. Claims register

| Claim | Evidence | Status |
|---|---|---|
| HyperAgent root exists as planner task | `execution.rs:1321-1335`, tier mapping | VERIFIED |
| Chat box is not a SuperAgent | `ChatCore.tsx` + engine path | VERIFIED |
| SuperAgent has no bundled persona/template | `personas.json` has 5 entries, no superagent | VERIFIED |
| Subagent is not a separate tier | `AgentTier` enum + fallback mapping | VERIFIED |
| Memory is role-level, 8 rows | `execution.rs:880-884`, `store.agent_memory` | VERIFIED |
| Per-hive memory absent | schema/context search | VERIFIED (not present) |
| 64-agent cap | `caps.rs:9-15` | VERIFIED |
| Tool loop now advertised on first request | code + new test + live file_read run | VERIFIED |
| 300–600 concurrent agents supported | counter-evidence caps | **REFUTED** |

**Verdict:** the HMAS foundation is real and now has a working tool-calling DeepSeek path, but the
agent-role/templates/memory/concurrency model is incomplete relative to the “HyperAgent + SuperAgent
sub-orchestrator + per-hive memory + hundreds of agents” ambition.
