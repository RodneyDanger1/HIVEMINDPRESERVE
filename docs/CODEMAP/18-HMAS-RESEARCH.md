# 18 — HMAS/AWN research: LangGraph, AutoGen, CrewAI

Research only; no product code edited. Sources fetched 2026-11-07.
**VERIFIED** = read in the official source at that URL. **READ** = read in this repo (path:line).
**UNVERIFIED** = not proven here; do not build on it.

## Question

What can HiveMind learn from LangGraph, AutoGen (AgentChat+Core) and CrewAI for a local-first HMAS/AWN with
hierarchy, peer messaging, persistent memory and agent profiles? Per framework: (1) declarative profile format
for import, (2) hierarchy primitives, (3) messaging and state handoff, (4) persistence, (5) licence and
runtime-dependency risk.

## Confirmed

| # | Claim | Mark | Source |
|---|---|---|---|
| C1 | CrewAI is **JSONC-first** now: `agents/<name>.jsonc` + `crew.jsonc`; `config/agents.yaml` is the **classic** layout, "remains supported". Required `role`/`goal`/`backstory`; also `llm`,`tools`,`allow_delegation`,`max_iter`,`memory`; `settings` wins over top level. | VERIFIED | docs.crewai.com/v1.15.37/en/concepts/agents |
| C2 | CrewAI hierarchy = `Process.hierarchical` with `manager_llm`/`manager_agent`; the manager plans, allocates, validates — tasks are not pre-assigned. Peer delegation = `allow_delegation`. | VERIFIED | …/concepts/processes, …/concepts/agents |
| C3 | CrewAI persistence = Flow `@persist` (class or method), SQLite default; resume by `kickoff(inputs={"id":…})`, fork by `restore_from_state_id`; `remember`/`recall`/`extract_memories` over a disk-backed memory store (LanceDB). | VERIFIED | …/concepts/flows |
| C4 | AutoGen serialises agents/teams to declarative `ComponentModel` JSON (`provider`,`component_type`,`version`,`config`), nested team→participants→model_client, via `dump_component()`/`load_component()`. Tool serialisation "not yet supported". Loading may execute code — trusted sources only. | VERIFIED | microsoft.github.io/autogen/stable/user-guide/agentchat-user-guide/serialize-components.html |
| C5 | AutoGen state = `save_state()`/`load_state()` on agents and teams; `TeamState` is JSON keyed per participant; dump to file/DB; `reset()` clears; run resumes mid-team. | VERIFIED | …/agentchat-user-guide/tutorial/state.html, …/tutorial/teams.html |
| C6 | AutoGen messaging: **direct** `send_message` (request/response, exceptions propagate) vs **broadcast** `publish_message` on a `TopicId` (pub/sub, one-way, replies discarded, no self-delivery). "Messages are the only means through which agents can communicate." | VERIFIED | …/core-user-guide/framework/message-and-communication.html |
| C7 | AutoGen hierarchy is **group-chat shaped**, not parent/child: `RoundRobinGroupChat`, `SelectorGroupChat`, `MagenticOneGroupChat`, `Swarm` (handoffs); participants share one context and broadcast each turn. | VERIFIED | …/agentchat-user-guide/tutorial/teams.html |
| C8 | LangGraph hierarchy = subgraph-as-node (shared keys) or subgraph-in-node (schema mapping); `checkpoint_ns` is `""` at root, `node:uuid` for a subgraph, nested joined by `|`. Subgraph persistence is per-invocation (`None`, default), per-thread (`True`, no parallel calls into the same subgraph) or stateless (`False`). | VERIFIED | docs.langchain.com/oss/python/langgraph/use-subgraphs.md |
| C9 | LangGraph persistence = checkpointer (thread-scoped, `thread_id`, one checkpoint per super-step, `StateSnapshot` with `next`/`metadata`/`parent_config`, history, replay, `update_state`), durability `exit`/`async`/`sync`; plus a separate **Store** for cross-thread memory. In-memory savers die on restart; SQLite is the local option. | VERIFIED | …/langgraph/checkpointers.md, …/langgraph/persistence.md |
| C10 | LangGraph has no declarative profile format in the pages read: graphs are Python `StateGraph`/`add_node`/`compile`; the only declarative artefact is serialised state, not an agent spec. | READ | same pages (all examples are code) |

## Local HiveMind evidence (READ)

- `SPAWN_SUB_HIVE {parent_task_id, role}`, precondition `depth < 3, fan-out < 8`, refusals `E_SPAWN_DEPTH`,
  `E_FANOUT_LIMIT` — `docs/contracts/UI-COMMANDS-v1.md:33`. `REWIRE_TASK` re-parents a live node with
  `expected_rev` and `E_CYCLE` (`:34`) — live graph mutation none of the three frameworks offers.
- Caps are code: `MAX_SPAWN_DEPTH: u8 = 3` (`caps.rs:9`, `budget.rs:34`), `FANOUT_LIMIT: u8 = 8`
  (`budget.rs:37`), `AGENT_LIMIT: u16 = 64` (`:40`), `IN_FLIGHT_LIMIT: usize = 8` (`:43`); `check_spawn`
  refuses by name (`caps.rs:50-53`, `budget.rs:469-480`) — under `backend/crates/hiverouter-core/src/` and
  `…/hiverouter-provider/src/`.
- A typed declarative persona record already exists — the natural import target:
  `Persona { id, display_name, tier, template_id, allowed_tool_ids, output_schema_id, refusal_policy }`
  (`prompts.rs:354-369`), `PersonaTier::{Hyper,Super,Worker,Tool}` (`:340-350`), `validate()` →
  `E_PERSONA_UNKNOWN`/`E_TEMPLATE_UNKNOWN` (`:376-388`), `load_personas()` (`:394-405`). Shipped data: one
  record, `planner`/`hyper`/`plan-v1`/`strict` (`assets/prompts/personas.json:1-11`); test-pinned
  (`tests/prompt_assets.rs:73-84`) — all under `backend/crates/hiverouter-core/`.
- Licence gate for content already exists: `ALLOWED_LICENCES = [MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause,
  ISC, CC0-1.0]`, manifest rows need `{spdx, source_url, fetched_at}`, violation ⇒ `E_TEMPLATE_LICENSE_MISSING`
  (`prompts.rs:7-9,23-30`; `UI-COMMANDS-v1.md:42`). Pinned prompt format `prompty 2.1.0` (`prompts.rs:3-4`).
- Run/replay shape: plan DAG → tasks → `NODE_SPAWN` → terminal snapshot, SQLite (`02-ARCHITECTURE.md:22-53`);
  `HISTORY_SNAPSHOT {run_id, revision, graph}` (`UI-COMMANDS-v1.md:43,57`) is a replay axis comparable to
  `get_state_history`.
- Declared gaps bounding any import: memory/research/learning loops absent (`09-RISKS-GAPS.md:12`); persona
  loader "absent"/not wired (`:8`); no first-party LICENSE/NOTICE, licence census incomplete (`:28`).
- **Resolved 2026-11-07:** the production boot now loads `personas.json` from the embedded asset
  (`load_personas_embedded`), and `RunEngine::seed_task_context` copies the persona allowlist into a
  task's scoped context. The engine wiring is now READ/verified by tests; this was UNVERIFIED when the
  research was first written.
- Roadmap v3 here = `audit/ROADMAP.md` + `audit/ROADMAP-TASKS.csv` (101 rows, `_incoming/DUMP-ANALYSIS.md:470`;
  listed `08-FILE-CATALOG.md:96-97`).

## Import mapping recommendation

Import **formats and invariants, never the frameworks**: HiveMind is a single Rust binary, none of the three
is embeddable, so the importer must be a content/build-time transform.

Extend `Persona` additively (existing `personas.json` must keep validating): `description`, `tags[]`,
`source {framework, url, spdx, fetched_at, version}`, `profile_version`. Compile prose into a `prompty`
template referenced by `template_id`; never store raw `role/goal/backstory`.

| Source | Target | Rule |
|---|---|---|
| CrewAI `role` | `display_name` + slugged `id` | blank ⇒ `E_PERSONA_UNKNOWN` |
| CrewAI `goal`+`backstory` | `template_id` → `.prompty` body | record source hash at import |
| CrewAI `tools[]` | `allowed_tool_ids` | each id must resolve in the tool registry, else refuse the row |
| CrewAI `allow_delegation`,`max_iter`,`memory`,`reasoning` | dropped | policy belongs to engine caps, not a profile |
| CrewAI `llm`,`function_calling_llm` | dropped | one provider binding per product |
| AutoGen `config.system_message` | `template_id` → `.prompty` | same path as above |
| AutoGen `model_client` | dropped | pins a cloud provider |
| AutoGen `handoffs[]` | plan-DAG edges, not persona fields | a handoff is topology, not identity |
| AutoGen `tools` | unsupported | source cannot serialise tools (C4) |
| AutoGen `provider`/`component_type`/`version` | `source{}` metadata | provenance, not dispatch |
| LangGraph | nothing importable | code-first (C10); shape only: `checkpoint_ns` addressing, checkpointer-vs-store split |

Importer contract: deterministic (stable ids, sorted keys), idempotent, no network at boot, no code execution
from input, licence precondition reusing the existing gate (outside `ALLOWED_LICENCES` ⇒
`E_TEMPLATE_LICENSE_MISSING`), and it may never raise `MAX_SPAWN_DEPTH`/`FANOUT_LIMIT` or become a second
source of truth for caps.

## Licence/dependency notes

| Framework | Licence (VERIFIED) | Dependency risk |
|---|---|---|
| CrewAI | MIT — `api.github.com/repos/crewAIInc/crewAI/license` (`spdx_id: MIT`, "Copyright (c) 2025 crewAI, Inc.") | Python; LiteLLM model layer, LanceDB memory, SQLite. Gate-compatible; not embeddable. |
| LangGraph | MIT — `…/repos/langchain-ai/langgraph/license` (`spdx_id: MIT`, "Copyright (c) 2024 LangChain, Inc.") | Python + `langchain-core` + separate checkpoint packages; LangSmith tracing and Agent Server are hosted-by-default and must stay optional for local-first. |
| AutoGen | **Dual**: `LICENSE` = CC-BY-4.0 (content; GitHub reports `cc-by-4.0`), `LICENSE-CODE` = MIT (Microsoft). | Python + pydantic + asyncio. **CC-BY-4.0 is not on `ALLOWED_LICENCES`** (`prompts.rs:23-30`) ⇒ doc/example-derived content fails the gate by design; only MIT code-derived material is admissible, and `load_component` executes code (C4) ⇒ import data only. |

Cross-cutting: (1) each adds an interpreter and unpinned tree to a pinned single binary; (2) all default to
cloud providers, so local-first is a redesign; (3) version confidence differs — CrewAI pinned in the URL
(`v1.15.37`), AutoGen `stable` with no version, LangGraph only `langgraph>=1.2` ⇒ `version_pinned: false` for
two of three; (4) no first-party LICENSE/NOTICE and an incomplete census (`09-RISKS-GAPS.md:28`) already.

## Unknowns (UNVERIFIED)

- Whether an AutoGen team can nest as a participant of another team; `AgentTool` unread (C7 is flat).
- Exact CrewAI memory scoping/slicing/embedder semantics (only the Flows summary was read).
- Whether HiveMind's persona loader is wired into `RunEngine` at runtime (conflict above).
- Licence of CrewAI/AutoGen example repositories and prompt corpora, as distinct from core code.
- Whether HiveMind has any peer-to-peer channel: **resolved 2026-11-07** with durable
  `ask_parent`/`send_to_sibling` tools and the `agent_messages` table; cross-branch peer messaging
  remains intentionally blocked pending an approval/policy gate.

## Design implications for Roadmap v3

1. **Profile import is a content pipeline, not a dependency.** Add the optional `source`/`description`/`tags`
   fields to `Persona`, keep one schema, gate every row through the allowlist. Additive work on `prompts.rs`
   plus an importer tool; no Python runtime.
2. **Keep hierarchy a capped task graph, not a group chat.** LangGraph's subgraph/`checkpoint_ns` model is the
   nearest analogue to `SPAWN_SUB_HIVE` and preserves typed refusals and at-most-once effects; AutoGen's
   broadcast group chat erases parent/child attribution and fights node-scoped approvals.
3. **Name the handoff contract.** A child result is the analogue of a subgraph output, and `output_schema_id`
   already exists on `Persona` but is unused by the spawn path — make child→parent return a validated payload
   before the v3 wire format freezes.
4. **Adopt the two-axis persistence split, not a framework.** Keep run-scoped checkpoints (runs/tasks/snapshots
   + `HISTORY_SNAPSHOT` revisions) separate from a cross-run store, which is absent (`09-RISKS-GAPS.md:12`).
   A scoped SQLite memory table with explicit `scope` and importance is smaller and testable.
5. **Keep and document both messaging modes.** HiveMind already has request/response (command + `ACK`),
   broadcast (events) and typed refusal — the distinction AutoGen formalises (C6). Document it as an AWN
   primitive rather than importing a bus.
6. **Publish the import policy first:** MIT/Apache/BSD/ISC/CC0 in, CC-BY-4.0 out; provenance
   (`spdx`,`source_url`,`fetched_at`,`version`) mandatory; no code loading; caps immutable by import.
7. **Resolve the persona-wiring conflict before any profile work.** Importing profiles into a registry the
   engine never reads would create a second source of truth — verify the `http.rs` → engine boot path first.
   **Resolved 2026-11-07:** boot loads embedded personas and seeds task allowlists; see the tests
   `embedded_personas_load_without_a_data_dir` and `boot_persona_allowlist_is_seeded_on_worker_contexts`.

## Current implementation delta (2026-11-07, after research)

- R1 real parents: `PlanNode.parent`/`WireNode.parent`, depth-chain validation, real parent rows; test
  `planned_parent_chain_is_persisted_not_flattened_to_root`.
- R2 context: task-creation seeding + bounded/safe prompt facts; context tests and prompt helper tests.
- R3 aggregation: leaves then deepest-first aggregators with per-task results; tree test proves 3 executions.
- Personas: embedded `EMBEDDED_PERSONAS`, boot load, `with_personas`, allowlist seeding; five shipped
  profiles (`planner` + four Library roles) exposed at `/api/agents/profiles` and in the read-only UI.
- Messaging: migration v6 `agent_messages`; `ask_parent`/`send_to_sibling`; same-parent/run auth; 32-message
  per-run budget; store + tool-loop tests; `AGENT_MESSAGE` contract event (24 events) emitted on write;
  `/api/agents/runs/{id}/messages` + AgentMessagesPanel.
- Memory: migration v7 `agent_memory`; role-scoped upsert/recall (latest 8) into task context; terminal task
  text is remembered when it carries no obvious secret marker. No editor/clear/consent UI yet.
- Usage: migration v8 `run_usage`; measured token totals persist with NULL-not-zero; cost is not settled.
- Hierarchy policy: Workers may spawn `subagent` only; Super escalation from a Worker is refused by `spawn_sub_hive`;
  Hyper/Super retain broader spawn rights. Profiles may tighten `max_children`/`max_depth`; per-run cap profiles still absent.
- Profiles: migration v9 `agent_profiles` stores validated imported profiles; import/delete API + UI;
  boot merges persisted profiles over embedded ones; delete restores built-ins. No full visual editor yet.
- Memory controls: `GET /api/agents/memory` + `DELETE /api/agents/memory/{role}` + AgentMemoryPanel.
- Still **not** done: full profile editor, cross-branch approval gate, semantic retrieval, HMAS controller,
  full HMAS UI, per-run cap profiles. Do not claim HMAS complete.
