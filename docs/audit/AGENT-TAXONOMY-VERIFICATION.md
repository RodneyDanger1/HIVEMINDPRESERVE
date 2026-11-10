# HiveMind HMARS — Agent Taxonomy Verification

- **Task:** task-17
- **Repository:** `C:\Users\Riley\Desktop\hiverouter-v1`
- **Pinned commit:** `919c3398c5bf47b4deb9ed9fd2004d2a0916cb9a` (“docs: verify 375px veil and usage checkpoint after hard kill”)
- **Audit date:** 2026-11-10
- **Mode:** read-only static audit. No source edit, commit, provider key, TokenLab point, test run, server, browser, or device action.
- **Only write:** this file (`docs/audit/AGENT-TAXONOMY-VERIFICATION.md`).
- **REA:** absent in this profile. No REA capability or evidence is claimed.

> **Tree movement during the audit.** The audit started at `919c339`. While it was running, the shared tree advanced to `197ee28` (docs/evidence additions only). At audit end, the working tree also had uncommitted changes in `execution.rs` (two prompt-wording lines), `remote.rs` (an apparent audit-actor fix: root-created remotes now audit as `system`), and `tests/run_lifecycle.rs` (the corresponding assertions). The verdict is pinned to `919c339`; those uncommitted changes are not used as evidence and are noted where they touch a finding. Source citations are line-compatible with `919c339`; post-pin documents are not used for the verdict and are marked where mentioned.

---

## 0. Provenance labels and method

### 0.1 Labels used

| Label | Meaning in this audit |
|---|---|
| **VERIFIED (static)** | The exact code/test/doc line was read in the pinned tree and the stated fact is directly checkable there. This is **not** runtime verification. |
| **READ** | An official external page/repository document was fetched and quoted in this audit. |
| **READ (proxy)** | An official URL was fetched through a text renderer because the direct HTML was truncated by site navigation. The quote is from the official page body, but a browser re-read is required before screenshot-grade reliance. |
| **INFERRED** | A conclusion drawn by combining two or more verified/read facts; not a direct line statement. |
| **UNVERIFIED** | Not read, not fetched, or not executed in this audit. |

No runtime label is used. No build, test, server, provider call, database migration, remote action, or UI action was executed. Statements about observed behavior are therefore static/inferred, not runtime-proven.

### 0.2 What was read

**Local source and tests**

- `backend/crates/hiverouter-core/src/execution.rs` (full structural read; key sections cited below)
- `backend/crates/hiverouter-core/src/planner.rs`
- `backend/crates/hiverouter-core/src/provider_executor.rs`
- `backend/crates/hiverouter-core/src/remote.rs`
- `backend/crates/hiverouter-core/src/daemon.rs`
- `backend/crates/hiverouter-core/src/remote_supervisor.rs`
- `backend/crates/hiverouter-core/src/prompts.rs`
- `backend/crates/hiverouter-core/src/tool_loop.rs`
- `backend/crates/hiverouter-core/src/http.rs` (persona/boot/node-label/remote-route sections)
- `backend/crates/hiverouter-core/src/ws.rs` (template-insert authority section)
- `backend/crates/hiverouter-core/src/store.rs` (identity, ownership, memory-bank, chat sections)
- `backend/crates/hiverouter-core/src/db.rs` (migrations v13, v14, v17, v19; chat schema)
- `backend/crates/hiverouter-core/src/entities.rs`
- `backend/crates/hiverouter-core/src/caps.rs`
- `backend/crates/hiverouter-tools/src/tool.rs`
- `backend/crates/hiverouter-contract/src/event.rs`
- `backend/crates/hiverouter-core/assets/prompts/{planner,superagent,agent,subagent}.prompty`, `personas.json`, `manifest.json`
- `backend/crates/hiverouter-core/tests/{run_lifecycle,scale_s1,snapshot_history,ws_transport,prompt_assets,prompt_assets_wiring}.rs`

**Local docs**

- `docs/provider/AGENT-ARCHITECTURE-TRUTH.md`
- `docs/research/agent-definitions/VERIFIED.md`
- `docs/plans/HIVEMIND-MASTER-FINISH-PLAN.md`
- `docs/ui/HIVEMIND-UI-GAP-AUDIT.md`
- `docs/ui/figcraft/HiveMind/SPEC/HIVEMIND-TARGET-SPEC.md`
- `docs/research/remote-agents/00-README.md`, `01-remote-agent-model.md`
- `audit/UNSLOP-REPORT.md`
- `docs/audit/AGENTIC-LIVE-TESTS.md` (post-pin; read for context, not used for the verdict)

**External sources actually fetched**

- Meta publication page: <https://ai.meta.com/research/publications/hyperagents/> — **READ**
- Meta arXiv abstract: <https://arxiv.org/abs/2603.19461> — **READ** (abstract only; PDF fetch unsupported)
- Meta research repository: <https://github.com/facebookresearch/Hyperagents> and `raw.githubusercontent.com/facebookresearch/Hyperagents/main/README.md` — **READ**
- DeepSeek research prototype: <https://github.com/deepseek-ai/dsh-superagent-prototype> and `README.md`, `docs/001-domain.md` — **READ**
- NVIDIA AI Agents glossary: <https://www.nvidia.com/en-us/glossary/ai-agents/> — **READ (proxy)**; direct fetch was truncated by navigation and the body was read through `r.jina.ai`.
- NVIDIA ToolOrchestra blog: <https://developer.nvidia.com/blog/train-small-orchestration-agents-to-solve-big-problems/> — **READ**
- Anthropic Claude Code subagent docs: <https://code.claude.com/docs/en/sub-agents> — **READ**
- OpenAI Agents SDK orchestration docs: <https://openai.github.io/openai-agents-python/multi_agent/> — **READ**
- A2A core concepts: <https://a2a-protocol.org/latest/topics/key-concepts/> — **READ**

**Not read:** the full Meta PDF, any DeepSeek article other than the repository docs, any NVIDIA page that uses “SuperAgent” as a product term, any social/aggregator source, and any runtime log produced by this audit.

**Citation shorthand.** After this section, source basenames (`execution.rs`, `planner.rs`, `provider_executor.rs`, `remote.rs`, `daemon.rs`, `remote_supervisor.rs`, `prompts.rs`, `tool_loop.rs`, `http.rs`, `ws.rs`, `store.rs`, `db.rs`, `entities.rs`, `caps.rs`, `tool.rs`, `event.rs`, `personas.json`, `planner.prompty`, `superagent.prompty`, `subagent.prompty`) refer to the full paths listed above. Doc basenames (`AGENT-ARCHITECTURE-TRUTH.md`, `VERIFIED.md`, `MASTER-FINISH-PLAN.md`, `UI-GAP-AUDIT.md`, `HIVEMIND-TARGET-SPEC.md`) refer to the docs listed above. All paths are repository-relative.

---

## 1. Definitions, each with source and status

### 1.1 Meta “HyperAgents” / “hyperagents”

**Definition (quoted).** Meta’s publication page states:

> “We introduce hyperagents, self-referential agents that integrate a task agent (which solves the target task) and a meta agent (which modifies itself and the task agent) into a single editable program. Crucially, the meta-level modification procedure is itself editable, enabling metacognitive self-modification…”

The companion repository describes itself as:

> “Self-referential self-improving agents that can optimize for any computable task”

and lists `meta_agent.py`, `task_agent.py`, and `run_meta_agent.py` as the main implementation files.

**Source / status**

- <https://ai.meta.com/research/publications/hyperagents/> — **READ**
- <https://arxiv.org/abs/2603.19461> — **READ** (abstract; the PDF was not read)
- <https://github.com/facebookresearch/Hyperagents> and its README — **READ**

**What it is.** A single editable program containing both a task-solving agent and a meta-level agent that can modify the task agent and itself; an open-ended self-improvement/evolution loop (DGM-H).

**What it is not.** It is not a hierarchy of “HyperAgent → SuperAgent → Agent → Subagent”. It does not describe a root orchestrator that dispatches a fixed DAG of worker tasks. Self-modification and a meta-agent are essential, not optional.

**HiveMind comparison.** HiveMind’s `HyperAgent` is a single root task (`role="planner"`, depth 0) that accepts a plan and later synthesizes results. There is no meta-agent, no editable self-modification procedure, and no evolution loop in the runtime. The shared word is a name collision, not an implementation of Meta’s concept. **INFERRED** from the code below plus the quoted definition.

### 1.2 DeepSeek “Super-agent” / SuperAgent

**Definition (quoted).** The official DeepSeek research-prototype repository states:

> “**Research prototype for reference only, not an official DeepSeek product.** … dsh-superagent is a proactive coding agent for one GitHub repository … It listens to GitHub and, optionally, Feishu. When someone allowed asks the agent for work, it opens a work item, works on it in its own agent session and directory, and answers in the original conversation…”

Its domain model defines `Source`, `Trigger`, `Thread`, `Eval`, `Mandate`, `Ticket`, `Assignment`, `Subscription`, and `Executor`. The glossary distinguishes:

- `Main Session` — “the DSH Session that evaluates Triggers, accepts and dispatches Tickets, and replies; `orchestrator` in code and prompts”
- `Executor Session` — “the DSH Session an Executor runs for one Ticket; `worker` in code and prompts”

**Source / status**

- <https://github.com/deepseek-ai/dsh-superagent-prototype> — **READ**
- `README.md` at `raw.githubusercontent.com/deepseek-ai/dsh-superagent-prototype/main/README.md` — **READ**
- `docs/001-domain.md` at the same repository — **READ**

**What it is.** A proactive work-management system for one repository: event sources create triggers, an evaluation/mandate/ticket path authorizes work, and an executor session performs it. “Super-agent” names the whole system, not a rank between a hyper-agent and a worker.

**What it is not.** It is not a four-level agent hierarchy, and DeepSeek itself calls it a research prototype, not an official product. The “Super-agent” label does not imply a depth-1 branch-lead task.

**HiveMind comparison.** HiveMind’s `SuperAgent` is a depth-1 task with a sub-orchestrator prompt, a spawn tool, and a hive memory bank. DeepSeek’s Super-agent is a repository-scoped work system with a main orchestrator session and executor sessions. The concepts overlap only in “something orchestrates work”; the structures are different. **INFERRED**.

### 1.3 NVIDIA “super agent” / “sub-agent” (and the absence of a SuperAgent product term)

**Definition (quoted).** NVIDIA’s official AI Agents glossary says:

> “A specialized agent, or sub-agent, is a domain-specific AI agent designed to execute a focused, well-defined task—such as chip verification, supply chain optimization, or private deep research—within a larger multi-agent system. Unlike a general-purpose ‘super agent’ (such as Claude Code or Codex, which orchestrates work at a high level), a specialized agent is purpose-built and optimized for a specific function…”

It also classifies a “Hierarchical” agent type as:

> “Multi-tiered system with higher-level agents managing specialized agents”

**Source / status**

- <https://www.nvidia.com/en-us/glossary/ai-agents/> — **READ (proxy)**. The direct fetch returned the site’s navigation shell and truncated the body; the quoted text is from the same official URL via `r.jina.ai`. A browser re-read is required before treating the wording as screenshot-grade.
- <https://developer.nvidia.com/blog/train-small-orchestration-agents-to-solve-big-problems/> — **READ** (NVIDIA Research uses the term **orchestrator**, not “SuperAgent”, for the supervisor model that manages other models and tools).

**What it is.** NVIDIA uses “super agent” generically for a general-purpose, high-level orchestrator (with Claude Code and Codex as examples) and “sub-agent” for a specialized agent inside a larger multi-agent system.

**What it is not.** No official NVIDIA page found in this audit defines “SuperAgent” as a product or as a fixed rank in a four-level hierarchy. NVIDIA’s research vocabulary for the supervisor role is “orchestrator” (ToolOrchestra).

**HiveMind comparison.** HiveMind’s `HyperAgent` is closer to NVIDIA’s “super agent” (high-level orchestrator) than HiveMind’s `SuperAgent` is. HiveMind’s `SuperAgent` is a mid-level branch lead, which NVIDIA would more naturally call an orchestrating sub-agent. HiveMind’s `Agent` and `Subagent` are both specialized/delegated children. **INFERRED**.

### 1.4 Ordinary Agent vs Subagent

**OpenAI Agents SDK (quoted).**

> “An agent is an LLM equipped with instructions, tools and handoffs. This means that given an open-ended task, the LLM can autonomously plan how it will tackle the task, using tools to take actions and acquire data, and using handoffs to delegate tasks to sub-agents.”

The same page distinguishes two orchestration patterns:

- **Agents as tools** — “A manager agent keeps control of the conversation and calls specialist agents through `Agent.as_tool()`.”
- **Handoffs** — “A triage agent routes the conversation to a specialist, and that specialist becomes the active agent for the rest of the turn.”

**Anthropic Claude Code (quoted).**

> “Subagents are specialized AI assistants that handle specific types of tasks. … Each subagent runs in its own context window with a custom system prompt, specific tool access, and independent permissions. … When Claude encounters a task that matches a subagent’s description, it delegates to that subagent, which works independently and returns results.”

**Source / status**

- <https://openai.github.io/openai-agents-python/multi_agent/> — **READ**
- <https://code.claude.com/docs/en/sub-agents> — **READ**

**Synthesis.** In current primary usage, an **agent** is the unit that can own a turn (LLM + instructions + tools + delegation ability). A **subagent** is an agent invoked by another agent, with its own context/prompt/tool scope and a bounded delegated task. The distinction is about delegation, context, and authority — not a fixed four-rank company org chart.

**HiveMind comparison.** HiveMind’s root is the only task that owns the run’s final turn. Its `Agent` (depth 2) is already a delegated child; its `Subagent` (depth 3) is a delegated child of a delegated child. In OpenAI/Anthropic terms, HiveMind has one main agent and three ranks of subagents. HiveMind’s labels are internal ranks, not the generic definitions. **INFERRED**.

### 1.5 Remote worker / satellite

**Audit-brief criterion (user-provided, not an external standard).** A remote worker/satellite should be a **tree-free detached remote** — not a task row, not a DAG edge, not a depth — ideally spawned by the HyperAgent, or requested by a lower level and approved by the HyperAgent.

**A2A primary definition (quoted).**

> “**A2A Server (Remote Agent):** An AI agent or agentic system that exposes an HTTP endpoint implementing the A2A protocol. It receives requests from clients, processes tasks, and returns results or status updates. To the client, the remote agent is an opaque (black-box) system: its internal workings, memory, and tools stay hidden.”

The same page defines `Agent Card`, A2A `Task`, `Message`, `Part`, and `Artifact` as the protocol’s discovery and interaction surface.

**Source / status**

- <https://a2a-protocol.org/latest/topics/key-concepts/> — **READ**
- Task criterion — user-provided; **not** an external source

**What it is.** A remote agent, in A2A, is an opaque agentic system behind an HTTP/A2A endpoint, discoverable by an Agent Card, with its own task/message lifecycle. It is federated, not a local task row.

**What it is not.** It is not an in-process fixture record, not a device grant, not a daemon job, and not a model call made by the local engine.

**HiveMind comparison.** HiveMind’s `remote.rs` is a tree-free control plane for remote principals, devices, grants, and actions. Its only transport is `in_process` and its only adapter is a fixture that changes normalized device state; it makes no network call and no model call. It is tree-free and detached, but it is not an A2A remote agent and not a worker. Its `daemon` kind `satellite` is an inert job record, not a remote worker. **VERIFIED (static)** for the code shape; **INFERRED** for the “not a worker” conclusion.

### 1.6 HiveMind’s own claims (the claims being tested)

| Claim | Source | Status |
|---|---|---|
| “one HyperAgent governs SuperAgents, Agents, and temporary Subagents” | `audit/UNSLOP-REPORT.md:5`; product framing repeated in UI docs | **READ (local doc)** |
| The HyperAgent is the root planner/orchestrator, `role="planner"`, depth 0, tier `HyperAgent` | `docs/provider/AGENT-ARCHITECTURE-TRUTH.md:17-18`; `backend/crates/hiverouter-core/src/execution.rs:1780-1794` | **VERIFIED (static)** for code |
| The SuperAgent is a “sub-orchestrator of one branch under the HyperAgent” | `backend/crates/hiverouter-core/assets/prompts/superagent.prompty:17`; `docs/provider/AGENT-ARCHITECTURE-TRUTH.md:18` | **VERIFIED (static)** for prompt text |
| A SuperAgent may only plan Agent workers at depth 2; an Agent may only plan Subagents at depth 3; no direct Hyper→Agent edge | `backend/crates/hiverouter-core/assets/prompts/planner.prompty:17-19`; `backend/crates/hiverouter-core/src/execution.rs:195-226` | **VERIFIED (static)** for prompt and matrix |
| The Subagent is a narrow temporary child that may not spawn or mutate the workspace | `backend/crates/hiverouter-tools/src/tool.rs:79-80`; `backend/crates/hiverouter-core/assets/prompts/subagent.prompty:17` | **VERIFIED (static)** for tier doc; prompt is READ |
| A ToolAgent is a level (UI/design) that invokes approved tools | `docs/ui/figcraft/HiveMind/SPEC/HIVEMIND-TARGET-SPEC.md:87-88,101-102`; `docs/provider/AGENT-ARCHITECTURE-TRUTH.md:21` | **READ (local doc)** |
| A remote worker/satellite can be spawned by the HyperAgent / requested by a lower level and approved | task-17 brief; `docs/research/remote-agents/00-README.md:25-31`; root tool at `execution.rs:4005-4086` | **READ** for design; **VERIFIED (static)** for root-only code |

The local docs disagree with each other and with the current code. Those disagreements are catalogued in §6.

### 1.7 Term mapping (external → HiveMind)

| External term | Closest HiveMind term | Match? | Why |
|---|---|---|---|
| Meta **hyperagent** (task agent + meta agent in one editable program) | `HyperAgent` root task | **Name only** | HiveMind has no meta-agent, no self-edit, no evolution loop. |
| DeepSeek **Super-agent** (proactive repository work system) | `SuperAgent` branch lead | **No** | DeepSeek’s term names a whole work system; HiveMind’s names a depth-1 task. |
| NVIDIA **super agent** (general-purpose high-level orchestrator) | `HyperAgent` (closer) / `SuperAgent` (weaker) | **Partial** | HiveMind’s HyperAgent is the high-level orchestrator; its SuperAgent is a mid-level branch lead. |
| NVIDIA/Anthropic **sub-agent** (specialized delegated child) | `SuperAgent`, `Agent`, and `Subagent` | **Partial** | All three are delegated children; HiveMind splits delegation into ranks. |
| OpenAI/Anthropic **agent** (unit that can own a turn) | `HyperAgent` only | **Partial** | Only the root owns the run’s final turn; every other “Agent” is a delegated child. |
| A2A **remote agent** (opaque HTTP/A2A endpoint) | `RemoteAgent` fixture principal | **No** | HiveMind’s remote has no A2A endpoint, Agent Card, model, or task lifecycle. |
| Task-brief **remote worker/satellite** | `RemoteAgent` + `daemon` kind `satellite` | **Partial (tree-free only)** | Tree-free/detached is real; “worker” and lower-level request/approval are not. |

**Single-sentence mapping.** HiveMind is a real four-rank task hierarchy (hyper → super → agent → subagent) wearing names borrowed from research and product vocabularies that mean different things.

---

## 2. HiveMind actual behavior per level

### 2.0 Identity and dispatch spine (the part that is genuinely hierarchical)

HiveMind has **five** `AgentTier` variants, not four:

- `AgentTier::{HyperAgent, SuperAgent, WorkerAgent, Subagent, ToolAgent}` — `backend/crates/hiverouter-tools/src/tool.rs:72-83`.
- `node_kind` maps them to `hyperagent`, `superagent`, `worker`, `subagent`, `tool` — `execution.rs:131-139`.
- The contract vocabulary now has five values, including `subagent` — `backend/crates/hiverouter-contract/src/event.rs:240`.

The hierarchy is persisted, not merely a UI color:

- Migration **v13** adds `tasks.class`, `tasks.rank`, `tasks.profile_id`, and an insert trigger. Mapping: `hyper` rank 1, `super` rank 2, `agent` rank 3, `subagent` rank 4; `tool` is not mapped (`NULL`) — `db.rs:298-383`.
- `task_identity` reads that persisted class/rank/profile — `store.rs:1184-1208`.
- Migration **v17** adds `ownership_mode`, `ownership_scope`, `owner_task_id`: hyper = `run`, super = `branch`/`hive:<id>`, agent/subagent = `none` — `db.rs:566-609`.
- Migration **v19** adds a unique partial index for one `run` owner per run — `db.rs:633-643`.
- Migration **v14** adds `memory_banks`; `ensure_bank_for_inserted_task` creates a run bank for every task, a hive bank for `super`, and a branch bank for `agent`/`subagent` — `db.rs:393-471`, `store.rs:2686-2749`.

The spawn matrix is enforced at every production insert path:

- `role_class` recognizes `hyper`, `super`, `agent`, and `subagent`; unknown roles return `None` — `execution.rs:152-166`.
- `class_may_spawn`: `hyper → super`, `super → agent`, `agent → subagent`, everything else false — `execution.rs:170-177`.
- `validate_spawn_role`: Hyper may only spawn Super aliases; Super may only spawn Agent/worker/worker-persona aliases (not Subagent); Worker may only spawn Subagent; Subagent and ToolAgent may not spawn — `execution.rs:195-226`.
- `task_agent_tier`: `planner`/`hyper` → Hyper; `super` → Super; `subagent` → Subagent; `worker`/`agent`/worker personas → Worker; unknown → Worker (latent fallback); empty role at depth 0 → Hyper — `execution.rs:235-250`.
- Plan insertion validates the child class against the real parent class and tier before a row exists — `execution.rs:1833-1895`.
- `spawn_sub_hive` performs the same validation under the task-state lock, then inserts, adds the graph edge, seeds context, and enqueues — `execution.rs:1440-1513`.
- The template-insert path maps template ids to spawn classes and refuses `planner`/`hyper`; it adds an implicit Super branch when a root-level worker template would otherwise skip a rank — `ws.rs:268-300`.
- Depth/fan-out/agent caps: max depth 3, max 8 children per parent, max 64 tasks per run — `caps.rs:8-18`, `caps.rs:52-63`; plan validation refuses depth >3 and parentless depth >1 — `planner.rs:50-128`.

Dispatch is hierarchical:

- Planned children are inserted depth-first so a parent row/edge exists before a child — `execution.rs:1830-1895`.
- The main loop runs a node only when all dependencies and all direct children are complete — `execution.rs:1944-1955`.
- After planned nodes, `dispatch_spawned_tasks` drains the dynamic spawn queue — `execution.rs:2041`, `execution.rs:2273-2323`.
- The root final synthesis runs only if the plan has aggregators or more than one node — `execution.rs:2048`.

**The central tension.** The class/rank are persisted, but key authority decisions still read the free-form role string:

- `task_agent_tier` derives the tier from `task.role`, not from `tasks.class` — `execution.rs:235-250`.
- `seed_task_context` sets the persona tool scope by `persona_for_role(&task.role)` — `execution.rs:1208-1212`.
- `persona_for_role` does exact/lowercase/separator/compact matching against persona ids — `execution.rs:825-846`.
- `TaskRecord` itself has only `role`, `depth`, `parent_task_id`, and state; it carries no class/profile — `entities.rs:28-45`.
- The provider advertises tools from the **tier** allowlist only, not the effective tier ∩ persona set — `provider_executor.rs:437`, `provider_executor.rs:724`, `tool_loop.rs:206-219`.

So the hierarchy is real, but it is still **role-label-derived in the paths that determine prompts and tools**. Two spellings of the same class can behave differently. This is the most important cross-cutting finding in the audit.

### 2.1 HyperAgent

**What the code actually creates**

- The root is a task with `role="planner"`, `parent_task_id=None`, `depth=0`, `state=Running`; it is inserted **after** the planning provider call — `execution.rs:1747-1794`.
- The graph metadata is `"hyperagent"` with label `"HyperAgent"` — `execution.rs:1793`.
- The root is the `run` owner (migration v17) and the unique `run` owner (migration v19) — `db.rs:566-609`, `db.rs:633-643`.
- The run bank is seeded with the prompt and a plan digest before children are seeded — `execution.rs:1798-1821`.

**Planning is not done by the root task**

- The engine claims one provider call and asks `executor.plan(...)` to produce the plan before the root row exists — `execution.rs:1747-1777`.
- The production planner system prompt is `PLANNER_SYSTEM` — `provider_executor.rs:26-33`.
- A repair call is allowed once on `SchemaInvalid`/`ProviderInvalidResponse` — `execution.rs:1758-1776`.
- Therefore “the HyperAgent plans the DAG” is conceptually true only at the run level; the root task never executes a planning turn.

**The bounded root control turn**

`root_checkpoint` runs once, after planned task rows are inserted and before dispatch — `execution.rs:1899`, `execution.rs:3857-4001`.

- It is disabled unless `root_orchestrator` is true; production boot enables it — `execution.rs:3864`, `http.rs:337`.
- It builds a prompt listing `spawn_superagent`, `spawn_remote_agent`, `read_memory_summary`, and `finish` (it omits `spawn_daemon` even though that tool is advertised) — `execution.rs:3873-3880`; compare `provider_executor.rs:39-44`, `provider_executor.rs:570-586`.
- It computes `has_planned_branch` once, before the turn — `execution.rs:3885`.
- It allows at most four tool calls per response — `execution.rs:3890`, `execution.rs:3905-3914`.
- It runs exactly one control round (`for _turn in 0..1`) — `execution.rs:3891`.
- Tool dispatch:
  - `spawn_superagent` calls `spawn_sub_hive(root.id, "super")` only if the plan had no depth-1 node — `execution.rs:3916-3927`.
  - `spawn_remote_agent` calls `execute_root_remote_call` — `execution.rs:3929-3931`, `execution.rs:4005-4086`.
  - `spawn_daemon` creates a daemon record; only `watchdog` gets a real `{"check":"db"}` job, every other kind gets `{}` — `execution.rs:3933-3971`.
  - `read_memory_summary` calls `store.memory_hyper_summary(run.id, 8)` — `execution.rs:3973-3988`.
  - `finish` returns — `execution.rs:3989`.
  - An unknown tool returns `E_TOOL_NOT_ALLOWED` — `execution.rs:3990-3993`.
- The accumulated `_result_text` is **not fed back** to the model and is not used after the loop — `execution.rs:3996-3998`.

This is not a monitoring loop. It is a single fire-and-forget dispatcher: the model cannot observe the result of its own spawn/remote/daemon/memory calls, cannot take a second control turn, and cannot adapt. The comment says this is intentional to avoid an unbounded controller — `execution.rs:3996-3998`.

**Root control tools are not ordinary tools**

- They are deliberately not in the `ToolRegistry` — `execution.rs:3854-3856`.
- They do not publish `TOOL_CALL`/`TOOL_RESULT` events; `execute_tool_turn` does that for leaf tools but `root_checkpoint` calls `orchestrate_root` directly — compare `execution.rs:3914-3995` with `execution.rs:2459-2460`.
- They do not consume `MAX_CALLS_PER_RUN` (the per-run tool budget) — `tool_loop.rs:16`, `execution.rs:2417-2460`; the root control turn only claims a provider call at `execution.rs:3896`.
- They are not approval-gated; they execute directly in the engine.
- The only cap is the four-call loop and the provider-call budget.

**The final synthesis**

- It runs only when `has_aggregators || plan.nodes.len() > 1` — `execution.rs:2048`.
- It repairs the root graph state to `Running`, snapshots, builds a prompt from the original prompt plus every non-root task result (24,000-char budget, 2,000-char per result, digest capped at 6,000), and calls `finalize_root` — `execution.rs:2052-2120`.
- `finalize_root` advertises **no tools** — `provider_executor.rs:615-641`.
- `ROOT_SYSTEM` nevertheless tells the model it “may call spawn_remote_agent at most once” — `provider_executor.rs:46-52`. That promise is not reachable in the no-tool final turn; a compliant provider cannot call it, and a hallucinated call would hit the registry and be refused as an unknown tool.
- If the root was killed/terminal before the final turn, the engine returns `E_TASK_TERMINAL` — `execution.rs:2052-2055`.

**The “authoritative answer” is a convention, not a contract**

- Every task completion with non-empty text appends its reply to the run’s chat as an `Assistant` message — `execution.rs:4337-4341`.
- `chat_messages` has only `id`, `run_id`, `role`, `text`, `created_at`; it has no `task_id` and no `is_final` flag — `db.rs:75-82`.
- The test `run_lifecycle_plans_and_dispatches` expects four messages for a two-leaf run: user prompt + two leaf replies + root synthesis — `run_lifecycle.rs:788-793`.
- A one-node plan skips the root final turn entirely — `execution.rs:2048`; the root is still reconciled to `Completed` by `reconcile_terminal_tasks` — `execution.rs:1635-1660`.
- The root’s final text is persisted as a task result and as the last assistant message, but nothing in the message schema identifies it as the authoritative answer. A UI that displays the last assistant message is relying on order, not on a contract.
- The root’s engine memory write is inconsistent: `complete_task` calls `record_agent_memory_scoped(..., scope_id=task.id)` (`execution.rs:4354-4363`); for class `hyper` and a non-`run` scope id that function resolves to `bank:branch:<task id>` (`store.rs:1345-1352`), while `ensure_bank_for_inserted_task` creates only a run bank for hyper (`store.rs:2712-2747`) and `caller_bank_for_class` says hyper → run bank (`store.rs:261-265`). The root final entry can therefore land in an orphan branch bank that `memory_banks_for_run` does not list. This is a memory-attribution defect, not a taxonomy-level issue.

**HyperAgent verdict in one line.** A real run-scoped root task and final synthesizer; not a Meta hyperagent, not a continuous orchestrator, and not a fully auditable tool user.

### 2.2 SuperAgent

**What is real**

- A depth-1 task with class `super`, rank 2, ownership `branch`, scope `hive:<task id>`, owner `task id` — `db.rs:576-580`, `db.rs:566-609`.
- A hive memory bank is created on insert — `store.rs:2712-2728`; its read set includes the run bank, parent branch, own branch, and nearest hive — `store.rs:2175-2228`, `store.rs:2752-2780`.
- It may spawn only Agent/worker aliases, not Super or Subagent — `execution.rs:195-226`, `execution.rs:1440-1513`.
- It has per-persona `max_children=8`, `max_depth=3` — `personas.json:21-30`; these are enforced in `spawn_sub_hive` for its own children — `execution.rs:1475-1485`.
- It can spawn children through the `spawn_agent` tool; the child is created with `parent_task_id=super.id`, depth 2, and is executed inline, with its result returned to the SuperAgent’s tool turn — `execution.rs:3570-3681`.
- If the global plan already gave it Agent children, the readiness loop runs those children before the SuperAgent, and the SuperAgent’s prompt includes their results with an aggregate instruction — `execution.rs:1944-1955`, `execution.rs:2214-2271`.

This is the strongest genuine sub-orchestration in HiveMind: a SuperAgent can decompose a branch imperatively, spawn multiple Agents sequentially, consume their results, and produce a branch result. It is more than a label.

**What is not real**

- There is no persisted branch **plan**. The only plan is the run-level plan produced before the root exists. A SuperAgent’s runtime decomposition is an imperative sequence of `spawn_agent` calls, not a stored DAG.
- There is no branch **budget**. It consumes the run-level provider-call budget; it has no per-branch call/token cap.
- There is no branch **rollup** separate from its own task result. The run-level root synthesis is the only rollup; the SuperAgent’s result is one task result among many.
- `ownership_mode='branch'` is persisted metadata. It is not used by spawn, kill, memory read, or budget enforcement; those paths use class/depth and the task id. The UI gap audit itself notes missing branch budget/ownership enforcement, branch summary/bank, branch-level kill/cancel, and card/panel — `docs/ui/HIVEMIND-UI-GAP-AUDIT.md:218-238` (the bank part is now implemented; the budget/rollup/kill parts are not).
- The SuperAgent cannot spawn a Subagent directly, despite an older local verification saying Super→Agent/Subagent; the current matrix is Super→Agent only — `execution.rs:203-216`, `execution.rs:4481-4488`.

**The role-spelling authority bug**

This is the sharpest concrete defect in the SuperAgent level.

- The `superagent` persona has seven tools: `spawn_agent`, `ask_parent`, `send_to_sibling`, `file_read`, `file_list`, `report_result`, `finish`; no `file_write`, no `shell_exec` — `personas.json:21-30`.
- The `AgentTier::SuperAgent` allowlist has **nine** tools, including `file_write` and `shell_exec` — `tool.rs:399-412`.
- `seed_task_context` applies the persona scope only if `persona_for_role(task.role)` finds a persona — `execution.rs:1208-1212`.
- `persona_for_role("superagent")` finds the persona. `persona_for_role("super")` does **not**: exact, lowercase, separator-normalized, and compact comparisons all miss (`super` vs `superagent`) — `execution.rs:825-846`.
- Therefore a SuperAgent named `"super"` gets:
  - no persona system prompt; `system_prompt_for` falls back to the generic `TASK_SYSTEM` — `provider_executor.rs:151-161`;
  - no persona tool scope; the tier’s nine tools are authorized, including `file_write` and `shell_exec` behind approval — `execution.rs:1360-1373`, `tool.rs:399-412`;
  - no persona spawn limits (though the global caps still apply).
- A SuperAgent named `"superagent"` gets the seven-tool persona prompt and call-time narrowing, but the provider is still advertised the tier’s nine tools, so the model sees `file_write`/`shell_exec` and only discovers at call time that they are refused — `provider_executor.rs:437`, `tool_loop.rs:206-219`, `execution.rs:2439-2457`.

This is not a theoretical alias: the provider planner example uses `"superagent"` and the planner prompt says SuperAgent branch leads (`provider_executor.rs:26-33`, `planner.prompty:17-19`), but the engine’s own tests and fixtures use `"super"` (`run_lifecycle.rs:30-43`, `run_lifecycle.rs:45-68`), and the DB maps both to class `super` (`db.rs:309-310,331-332`). Two spellings of the same class have different prompts and different effective grants. The persisted `profile_id` could resolve this — it maps `"super"` to `"superagent"` (`db.rs:331-332`) — but `seed_task_context` ignores it.

**SuperAgent verdict in one line.** A genuine depth-1 branch lead and sub-orchestrator, but not an owner of a branch plan/budget/rollup, and its prompt/tool authority depends on how the role string is spelled.

### 2.3 Agent (the wire calls it `worker`)

**What is real**

- A depth-2 task with tier `WorkerAgent` (nine tools: spawn/ask/sibling/report/finish/read/list/write/shell) — `tool.rs:413-426`.
- The `agent` persona also has nine tools, including `spawn_agent`, `file_write`, and `shell_exec` — `personas.json:68-75`.
- It may spawn only a Subagent; `validate_spawn_role(WorkerAgent, "subagent")` is the only allowed child class — `execution.rs:217-219`.
- It has a branch memory bank — `store.rs:2729-2745`; its read set includes the run bank, parent branch, own branch, and nearest hive — `store.rs:2175-2228`.
- `file_write` and `shell_exec` require operator approval — `tool.rs:378-389` (contract permission classes), `execution.rs:2498-2511` (approval branch).
- It can spawn a Subagent inline and receive its result — `execution.rs:3570-3681`.
- It aggregates planned children/dependencies from its prompt — `execution.rs:2214-2271`.

**What is not real**

- It is never a top-level agent. The root is the only depth-0 task; every `Agent` is a delegated child at depth 2. In OpenAI/Anthropic terms, it is a subagent.
- The wire kind for `WorkerAgent` is `"worker"`, not `"agent"` — `execution.rs:135`; the node-kind vocabulary has no `"agent"` kind (`event.rs:240`). The class `agent` is persisted (`db.rs:313-315`), but the node kind is `worker`.
- A role named `"worker"` finds no persona (`personas.json` has `agent`, not `worker`), so it gets the generic `TASK_SYSTEM` rather than `agent.prompty` — `provider_executor.rs:151-161`, `execution.rs:825-846`. Its tool authority is the same nine-tool tier, so this is an identity/prompt gap, not an authority gap.

**Agent verdict in one line.** A real bounded worker/leaf executor with approvals and a branch bank; not an “ordinary agent” in the top-level sense, and its persona is only applied when the role is spelled `agent`.

### 2.4 Subagent

**What is real**

- A distinct `AgentTier::Subagent` variant — `tool.rs:80-82`.
- A five-tool allowlist: `ask_parent`, `report_result`, `finish`, `file_read`, `file_list`; no spawn, no write, no shell — `tool.rs:427-436`.
- A depth-3 task class `subagent`, rank 4, ownership `none` — `db.rs:312-323,352-363`, `db.rs:579-580`.
- A branch memory bank — `store.rs:2729-2745`.
- Node kind `"subagent"` and contract vocabulary includes it — `execution.rs:182-188`, `event.rs:240`.
- It cannot spawn: `validate_spawn_role(AgentTier::Subagent, ...)` is always false — `execution.rs:218-219`; the tests pin this — `execution.rs:4447-4474`.
- A depth floor of 2 is enforced for subagent roles in plan insertion, `spawn_sub_hive`, and template insertion — `execution.rs:1483-1485`, `execution.rs:1863-1865`, `ws.rs:280-282`.

**What is not aligned**

- The `subagent` persona lists six tools, including `send_to_sibling` — `personas.json:32-39`. The tier allowlist does not include `send_to_sibling`, so the effective set is five; the persona entry is dead for that tool. The provider advertises the tier set, so the model never sees `send_to_sibling` anyway.
- “Subagent” here is a bottom rank under an Agent, not the generic “a subagent is any delegated child” definition. It is a real narrow leaf, but the name is a rank label.
- The HTTP label mapping falls through to `"{role} {suffix}"` for `subagent` (no dedicated arm), so the UI label is `subagent <suffix>` rather than a capitalized `Subagent` — `http.rs:962-968`. Cosmetic.

**Subagent verdict in one line.** A real, enforced, narrow leaf rank; not a fifth autonomous level and not the generic subagent concept.

### 2.5 ToolAgent

- `AgentTier::ToolAgent` exists with a four-tool allowlist: `file_read`, `file_list`, `file_write`, `shell_exec`; no `finish` and no `report_result` — `tool.rs:80-82`, `tool.rs:437-445`.
- `task_agent_tier` maps a role `"tool"` to `ToolAgent` — `execution.rs:244`.
- But `role_class` has no `"tool"` arm, so a planned/spawned `"tool"` role fails `role_class(...).ok_or(Refusal::PersonaUnknown)` before any row is created — `execution.rs:152-166`, `execution.rs:1845`, `execution.rs:1453`. The template path would map a hypothetical `"tool"` template id through its worker fallback; the embedded manifest has no such template.
- `validate_spawn_role` returns false for `ToolAgent` as parent and no parent tier accepts `"tool"` — `execution.rs:195-226`.
- The template path maps every unknown template id to `"worker"`; it does not produce `ToolAgent` — `ws.rs:272-279`.
- No shipped production path creates a `ToolAgent` task. The only hypothetical path would be a `"tool"` template id; the embedded manifest has no such template, and `role_class`/`validate_spawn_role` refuse `"tool"` on the plan and spawn paths. The prior taxonomy verification already called this out at an earlier HEAD — `docs/research/agent-definitions/VERIFIED.md:63-65,186`.

**ToolAgent verdict in one line.** A dormant allowlist/tier with no shipped producer and no terminal verbs; it is not a HiveMind level, despite the target spec showing a ToolAgent card.

### 2.6 Remote principal and satellite daemon

**Remote principal: what the code actually does**

- `remote.rs` is explicitly “tree-free principals, devices, grants, actions, audit. This is deliberately not a task/DAG model: a remote has no run, parent or depth and consumes no agent/call cap.” — `remote.rs:1-6`.
- The only transport is `in_process`; the only adapter is a fixture that updates normalized light/switch/cover state; no network call is made — `remote.rs:5-6`, `remote.rs:308-315`, `remote.rs:1358-1400`.
- `RemoteAgent` has no `run_id`, no `parent_task_id`, no depth, no model, and no memory bank; grep over `remote.rs` finds no `memory`, `model`, `provider`, `prompt`, `run_id`, or `depth` code path (only the doc comment).
- Root-only creation: `execute_root_remote_call` is called from `root_checkpoint`’s `spawn_remote_agent` branch — `execution.rs:3929-3931`, `execution.rs:4005-4086`.
- It creates a remote, then a device, then a grant, all with `owner_kind="hive"` and `owner_id=root.id` — `execution.rs:4029-4081`.
- It never inserts a task row or graph edge; the test asserts the remote id is not a task id and the task count is unchanged — `run_lifecycle.rs:565-597`.
- Lower levels cannot request a remote: `spawn_remote_agent` is not in `ToolRegistry`, not in any AgentTier allowlist, and not in the `spawn_agent` schema; the only non-operator path is the root control turn — `tool.rs:394-445`, `execution.rs:3854-3856`.
- There is no child→HyperAgent approval path for a remote. `ask_parent` can ask a question and get a model answer, but it cannot create a remote; the root control turn runs before children exist and is not re-entered — `execution.rs:3193+` (ask_parent), `execution.rs:1899` (root control timing), `execution.rs:2041-2048` (post-child root final has no tools).
- Audit attribution is wrong for root-created remotes at the pinned commit: `create_remote`, `create_device`, and `create_grant` hardcode the audit actor to `"operator"`/`"operator"` even when `created_by` is `"hyper:<root id>"` — `remote.rs:352-355,424-426,489-491,584-586`; the test only checks `created_by`, not the audit actor — `run_lifecycle.rs:581-586`. The current dirty worktree contains an apparent fix (`system` actor when `created_by` starts with `hyper:`) plus assertions; that fix is not part of the `919c339` verdict.

**Remote principal: what the design docs say vs what shipped**

- The design says a remote agent is a “tree-free capability principal” that “can be invoked by an operator or (later) an approved task tool” and “connector-defined (may call a model locally, outside HiveMind’s budget)” — `docs/research/remote-agents/01-remote-agent-model.md:9-15`.
- The design says remotes may have a system-bank read and one private bank — `docs/research/remote-agents/00-README.md:32-35`. The shipped code has no remote memory bank.
- The design says node vocabulary is four values; the current contract has five — `docs/research/remote-agents/01-remote-agent-model.md:19-20` vs `event.rs:240`. The design is stale on that point.
- The older taxonomy verification says “Remote tiny agent: Nothing. Zero `remote_*`/`device_*`/`daemon` symbols” — `docs/research/agent-definitions/VERIFIED.md:59,187`. That was true at its pinned HEAD, but it is false at `919c339`: remote/daemon code exists.

**A2A comparison.** A2A defines a remote agent as an HTTP endpoint implementing the protocol, opaque to the client, with an Agent Card and task lifecycle — §1.5. HiveMind’s remote is an in-process fixture record with none of those. It is tree-free and detached, but it is not an A2A remote agent and not a worker.

**Satellite daemon: what the code actually does**

- A daemon is “one-job records that are deliberately **not** task rows … It never invokes a model, never consumes run caps, and never appears in the run DAG.” — `daemon.rs:1-7`.
- `daemon.create` accepts kind `watchdog`, `relay`, `switch`, or `satellite`; it starts `enrolled` — `daemon.rs:146-187`, `daemon.rs:154`.
- The root tool `spawn_daemon` creates a daemon; only `watchdog` gets `{"check":"db"}`; every other kind gets `{}` — `execution.rs:3952-3967`.
- `tick_due`/`execute_daemon_job` only implements `{"check":"db"}` and `{"kind":"script"}`; any other job returns `degraded` with `E_DAEMON_JOB_UNIMPLEMENTED`, reason “daemon has no executable job” or “daemon job runtime is not implemented for this job kind” — `daemon.rs:270-331`.
- The supervisor lane expires leases and ticks daemons; it “is not a scheduler and never invokes a model” — `remote_supervisor.rs:1-6`, `remote_supervisor.rs:96-101`.
- The test asserts a daemon never becomes a task row — `run_lifecycle.rs:691-719`.

**Remote/satellite verdict in one line.** A real tree-free control plane for fixture principals/devices/grants and inert daemon records; not a remote worker, not an A2A remote agent, and not requestable by lower levels.

### 2.7 Cross-cutting behaviors that affect every level

**Caps vs prompts.** The planner prompt says “at most 8 children” (`planner.prompty:17`) and the provider planner prompt says “1..=8 nodes” (`provider_executor.rs:32`), but `Plan::validate` allows up to 40 nodes (`planner.rs:10,50-56`) and the caps allow 8 children per parent and 64 tasks per run (`caps.rs:11-13`). The scale test explicitly builds 41 tasks, “the plan cap” (`scale_s1.rs:1-2`). The 8-node limit is a soft model instruction, not a validated contract.

**Memory banks are real but not a full branch tree.** Run/hive/branch banks and read-set isolation exist (`store.rs:2175-2228`, `store.rs:2686-2780`). The root can call `memory_hyper_summary` only during its pre-dispatch control turn, before branch tasks have written any hive entries; the final root turn has no tools. So the root cannot actually consume post-dispatch hive summaries. The UI gap audit’s “root reads run bank/digest, not branch summaries” is still effectively true — `docs/ui/HIVEMIND-UI-GAP-AUDIT.md:171`.

**Chat attribution.** Every task reply is a run-level assistant message (`execution.rs:4337-4341`); the schema has no task id or final flag (`db.rs:75-82`). The root’s final answer is the last message by convention, not by contract.

**A2A at the task level is real, but it is not remote A2A.** `ask_parent`/`send_to_sibling` persist agent messages and can answer through `answer_escalation` (`execution.rs:3193+`, `provider_executor.rs:462-523`). This is intra-run agent messaging, not the A2A protocol and not a remote-agent transport. The post-pin live-test note reports one successful `ask_parent` answer (`docs/audit/AGENTIC-LIVE-TESTS.md:54-90`); that is a local doc claim, not independently reproduced here.

---

## 3. Gap matrix

Severity is about **taxonomy truth and authority**, not exploitability. A “High” means the level’s name/claim materially misdescribes what the code enforces or exposes.

| # | Claim (source) | Actual (file:line) | Gap | Severity | Smallest fix |
|---|---|---|---|---|---|
| G01 | The name “HyperAgent” implies Meta’s hyperagent: a task agent + meta agent in one self-modifying program | Root task `role="planner"`, depth 0, class `hyper`, owner `run`; no meta agent, no self-edit, no evolution loop — `execution.rs:1780-1794`, `db.rs:298-340`, `store.rs:1189-1208` | Name collision, not an implementation of Meta’s concept | High | Rename the UI/term or add an explicit “not Meta Hyperagents” note; do not claim research lineage |
| G02 | The HyperAgent plans the DAG and governs branches | Planning is a detached provider call before the root row exists; the root never plans — `execution.rs:1747-1794`, `provider_executor.rs:26-33` | Root owns plan acceptance and final synthesis, not planning | Medium | Say “run planner; root synthesizer,” or move planning into the root task |
| G03 | The HyperAgent is an orchestrator that can monitor, query, spawn, re-plan, and terminate (root prompt; UI gap audit target) — `execution.rs:3874-3878`, `docs/ui/HIVEMIND-UI-GAP-AUDIT.md:173` | One bounded control turn before dispatch; 4 calls max; one round; result text discarded; no post-dispatch control; `read_memory_summary` runs before branch memories exist — `execution.rs:3857-4001`, `execution.rs:3996-3998`, `store.rs:2230-2243` | No monitor/re-plan/terminate; root control is fire-and-forget; summary tool is temporally useless | High | Implement a bounded post-dispatch root turn that consumes results; or rename and remove the monitor/re-plan claims |
| G04 | The root adds at most one branch lead | `has_planned_branch` is computed once before the turn; every `spawn_superagent` call re-reads the same false value; up to 4 calls per response. The same loop also permits up to 4 remote/daemon creations with no per-run cap — `execution.rs:3885`, `execution.rs:3890-3914`, `execution.rs:3916-3971` | A flat plan can gain up to 4 SuperAgents in one turn, not 1; remote/daemon creations are likewise unbounded within the turn | Medium | Set a `spawned_branch` flag after the first successful spawn and a per-run cap for remote/daemon; add a 4-call test |
| G05 | Root tools are real tools, audited and capped | Root-only tools bypass `ToolRegistry`, `TOOL_CALL`/`TOOL_RESULT`, and `MAX_CALLS_PER_RUN`; only the provider-call budget applies — `execution.rs:3854-3856`, `execution.rs:3914-3995`, `tool_loop.rs:16`, `execution.rs:2417-2460` | No tool audit, no tool-counter consumption, no approval for root actions | High | Publish root `TOOL_CALL`/`TOOL_RESULT` and count them, or label them engine actions with durable audit rows |
| G06 | The HyperAgent produces the authoritative final answer — `execution.rs:2043-2065` | Every task completion with non-empty text appends an assistant chat message; the root final is just the last one; no `task_id`/`is_final`; one-node plans skip the root turn — `execution.rs:4337-4341`, `db.rs:75-82`, `execution.rs:2048` | “Authoritative” is last-writer order, not a contract; child replies are indistinguishable; single-node plans have no root answer | Medium | Add `task_id` and `is_final` to chat messages; always root-answer or state the single-node policy explicitly |
| G07 | The SuperAgent “owns this branch” (sub-orchestrator; branch bank/ownership) — `superagent.prompty:17,20` | Depth-1 class `super`, ownership `branch`/`hive:<id>`, hive bank, Super→Agent spawn, inline child execution — `db.rs:576-580`, `store.rs:2712-2728`, `execution.rs:1440-1513`, `execution.rs:3570-3681` | Branch is real as a task subtree + bank; no branch plan, budget, rollup, or kill/cancel; ownership is metadata | Medium | Add branch budget/plan/rollup and branch kill, or rename to “branch lead” and drop “owns” |
| G08 | The SuperAgent has 7 tools and no write/exec — `personas.json:21-30` | Tier allowlist is 9, including `file_write`/`shell_exec`; persona 7 narrows only when role is `superagent`; role `super` gets no persona, a generic prompt, and 9 tools — `tool.rs:399-412`, `personas.json:21-30`, `execution.rs:1208-1212`, `provider_executor.rs:151-161`, `provider_executor.rs:437` | Authority and prompt depend on role spelling; two names for the same class behave differently | High | Resolve persona by persisted `profile_id`; add a `super` alias; advertise the effective tier ∩ persona set |
| G09 | “Agent” is an ordinary agent | Always a depth-2 delegated child; root is the only top-level task; can spawn only Subagent — `execution.rs:195-226`, `tool.rs:413-426` | HiveMind “Agent” is a subagent in OpenAI/Anthropic terms; wire kind is `worker` | Medium | Keep `worker` as the wire name; document the rank; stop calling it an ordinary agent |
| G10 | The Subagent is a distinct narrow level | Real `AgentTier::Subagent`, 5 tools, no spawn, depth 3, branch bank; persona lists 6 incl. `send_to_sibling`, which the tier forbids — `tool.rs:427-436`, `personas.json:32-39`, `execution.rs:4412-4474` | Tier is real; persona/tier mismatch; the label is a rank, not the generic subagent concept | Low/Medium | Align the persona allowlist with the tier; document “leaf under an Agent” |
| G11 | The ToolAgent is a level (target spec/UI) — `docs/ui/figcraft/HiveMind/SPEC/HIVEMIND-TARGET-SPEC.md:87-88,101-102` | Tier + 4-tool allowlist, no shipped producer; `role_class` has no `tool`; template maps to worker; no `finish`/`report_result` — `tool.rs:80-82,437-445`, `execution.rs:152-166`, `ws.rs:272-279` | A dormant allowlist/tier, not a level; a card without a producer | Medium | Remove/relabel the ToolAgent card and taxonomy row, or add a real producer with terminal verbs |
| G12 | Remote worker/satellite | Tree-free fixture principal/device/grant/action; `in_process` only; no model/memory/task; root-only creation; no lower-level request/approval; audit actor hardcoded `operator` at `919c339` — `remote.rs:1-6,308-315,352-355,424-426,489-491,584-586`, `execution.rs:4005-4086` | Not a worker, not an A2A remote agent, not requestable by lower levels; AI action audited as operator in the pinned commit (the dirty worktree has an apparent fix) | High | Rename to “remote principal”; add a lower-level request + HyperAgent approval path and a real adapter if “worker/agent” is claimed; fix the audit actor |
| G13 | A satellite daemon is a remote worker | Kind `satellite` accepted; root sets `{}` for non-watchdog; `execute_daemon_job` returns degraded “no executable job”/“runtime not implemented” — `daemon.rs:154,304-330`, `execution.rs:3960-3964` | Inert record; no satellite job runtime, no model, no tools | Medium | Remove the `satellite` kind/tool until a job runtime exists, or implement the job and an explicit start policy |
| G14 | The plan is limited to 8 children/nodes | `Plan::validate` allows 40 nodes; caps allow 8 per parent/64 agents; prompts say 8; the scale test uses 41 tasks — `planner.rs:10,50-56`, `caps.rs:11-13`, `provider_executor.rs:26-33`, `scale_s1.rs:1-2` | Soft prompt vs hard validation mismatch | Low | Enforce 8 nodes in `Plan::validate`, or align the prompts/docs to 40 |
| G15 | Current docs describe the taxonomy | Several docs say 4 tiers, no Subagent tier, no persisted class, no hive bank, no remote code; the pinned tree has 5 tiers, v13 class, v14/v17 banks/ownership, and remote/daemon code — `docs/provider/AGENT-ARCHITECTURE-TRUTH.md:18-31`, `docs/research/agent-definitions/VERIFIED.md:55-59,121-140`, `docs/plans/HIVEMIND-MASTER-FINISH-PLAN.md:97`, `docs/ui/HIVEMIND-UI-GAP-AUDIT.md:218` | Docs are materially stale and contradict each other/code | High (doc truth) | Add superseded banners and update the truth doc to the v13/v14/v17/v19 model; add a doc-vs-code drift check |
| G16 | The root final turn can spawn a remote agent | `ROOT_SYSTEM` says “You may call spawn_remote_agent at most once”; `finalize_root` advertises no tools — `provider_executor.rs:46-52`, `provider_executor.rs:615-641` | Prompt promises a tool the turn cannot call | Low | Remove the sentence from `ROOT_SYSTEM`, or add a bounded final-turn tool with an explicit no-mutation-after-final policy |
| G17 | The root final answer is written to the run bank — `store.rs:261-265` | `complete_task` passes `scope_id=task.id`; for class `hyper` with a non-`run` scope, `record_agent_memory_scoped` resolves to `bank:branch:<root id>`; hyper insertion creates only a run bank; `caller_bank_for_class` says hyper → run bank — `execution.rs:4354-4363`, `store.rs:1345-1352`, `store.rs:2712-2747`, `store.rs:261-265` | Root memory entry can be an orphan branch-bank row; the run bank lacks the root final; bank listing may omit it | Medium | Special-case hyper to `bank:run` in `record_agent_memory_scoped` or pass `scope_id="run"` for the root; add a bank-row test |

**Positive findings (not gaps)** — these are the parts the evidence supports:

- One root per run is enforced by the unique partial index and by spawn validation — `db.rs:633-643`, `execution.rs:195-226`.
- The spawn matrix refuses direct Hyper→Agent, Hyper→Subagent, Super→Super, Super→Subagent, Agent→Super, and Subagent→anything — `execution.rs:195-226`, `execution.rs:4477-4489`.
- Depth/fan-out/agent caps are checked before row creation — `caps.rs:52-63`, `execution.rs:1474-1486`.
- Parent-child edges and readiness dispatch are real; a parent runs after its direct children — `execution.rs:1944-1955`.
- Class/rank/profile/ownership are persisted by migrations, not guessed at read time — `db.rs:298-383`, `db.rs:566-609`.
- Run/hive/branch memory banks exist and sibling branch entries are not in the read set — `store.rs:2175-2228`, `store.rs:2686-2780`.
- The root final turn is answer-only (no tools), so it cannot mutate after the final answer — `provider_executor.rs:615-641`.
- A root `spawn_remote_agent` creates no task row/edge — `run_lifecycle.rs:565-597`; a daemon creates no task row — `run_lifecycle.rs:691-719`.

---

## 4. Pseudocode for each level lifecycle

These blocks describe **actual code paths** at `919c339`; comments name the refusal or gap where relevant.

### 4.1 HyperAgent (root) lifecycle

```text
on SUBMIT_PROMPT(prompt):
  reject empty / too long / key missing / another active run
  run = create_run(status=queued)
  append user message; set active; spawn dispatch(run)

dispatch(run):
  run.status = planning
  claim_provider_call
  plan = executor.plan(planning_prompt)          # detached call; root row does not exist yet
  if SchemaInvalid/ProviderInvalidResponse:
      claim_provider_call; plan = executor.repair_plan(...)
  plan.validate()                                # <=40 nodes, depth<=3, parent chain, no cycles

  root = insert_task(role="planner", depth=0, parent=None, state=Running)
  graph.metadata(root, kind="hyperagent", label="HyperAgent")
  seed run bank: engine/prompt, engine/plan

  for node in plan sorted by depth:
      validate role class against real parent class + tier
      check depth / fan-out / agent caps
      insert task; add graph edge; seed context; publish NODE_SPAWN

  root_checkpoint(root, plan)                    # once, before dispatch
  run.status = running; snapshot

  while planned nodes ready (deps + direct children complete):
      execute_task_with_prompt(node)

  drain_spawned_tasks()                          # dynamic super/agent/subagent rows

  if has_aggregators or plan.nodes > 1:
      if root terminal: return E_TASK_TERMINAL
      build root prompt from all non-root task results (24k budget)
      executor.finalize_root(root, prompt)       # no tools advertised
  reconcile all tasks to terminal; run.status = terminal
```

```text
root_checkpoint(root, plan):
  if not root_orchestrator: return
  reply = executor.orchestrate_root(prompt + plan_digest)   # one provider turn
  for call in reply.tool_calls[0..4]:
      "spawn_superagent":
          if plan has any depth-1 node: refuse (no new row)
          else: spawn_sub_hive(root, "super")              # bug: flag not updated, up to 4 can spawn
      "spawn_remote_agent":
          create in_process remote + device + grant        # no task row; audit actor = "operator"
      "spawn_daemon":
          create daemon; watchdog gets {"check":"db"}, others get {}
      "read_memory_summary":
          read run/hive summaries; result is discarded
      "finish": return
      unknown: return E_TOOL_NOT_ALLOWED
  # one round only; no result feedback; no TOOL_CALL events; no tool-budget charge
```

### 4.2 SuperAgent lifecycle

```text
on spawn(parent = hyper root, role in {"super","superagent",...}):
  validate_spawn_role(HyperAgent, role)          # Hyper may spawn Super only
  depth = 1; check caps
  insert task; v13 trigger -> class="super", rank=2, profile_id="superagent"
  v17 trigger -> ownership_mode="branch", owner_task_id=self
  create hive bank "bank:hive:<id>"

on run:
  task.state = running
  seed context: persona lookup by raw role
      "superagent" -> 7-tool persona + sub-orchestrator prompt
      "super"      -> no persona -> generic TASK_SYSTEM + 9-tier tools
  provider.execute(system, tools = tier allowlist)   # advertises 9, even for "superagent"

  for each tool turn (<=12):
      "spawn_agent":
          validate Super -> Agent; create depth-2 child
          execute child inline; return child result to this turn
      "ask_parent" / "send_to_sibling" / "report_result" / "finish":
          normal tool path
  if planned Agent children existed:
      their results are already in the prompt; aggregate
  complete -> branch result; write result to hive bank
```

No branch plan, branch budget, branch rollup, branch-level kill, or ownership-based authorization is applied.

### 4.3 Agent (WorkerAgent) lifecycle

```text
on spawn(parent = super, role in {"agent","worker",...}):
  validate_spawn_role(SuperAgent, role)          # Super may spawn Agent only
  depth = 2; check caps
  insert task; v13 -> class="agent", rank=3
  v17 -> ownership_mode="none"
  create branch bank "bank:branch:<id>"

on run:
  task.state = running
  persona lookup: "agent" -> 9-tool agent prompt; "worker" -> generic TASK_SYSTEM
  provider.execute(tools = WorkerAgent 9-tool allowlist)
  file_write / shell_exec require operator approval

  for each tool turn:
      "spawn_agent" -> validate Agent -> Subagent; create depth-3 child; execute inline
      read/list/write/shell/ask/sibling/report/finish
  if planned Subagent children existed: aggregate their results
  complete -> task result; write to branch bank
```

### 4.4 Subagent lifecycle

```text
on spawn(parent = agent, role in {"subagent","sub_agent","sub-agent"}):
  validate_spawn_role(WorkerAgent, role)         # Worker may spawn Subagent only
  depth = 3; check caps; depth floor >=2
  insert task; v13 -> class="subagent", rank=4
  v17 -> ownership_mode="none"
  create branch bank "bank:branch:<id>"

on run:
  task.state = running
  persona lookup -> subagent.prompty (6 listed tools)
  effective tier := 5-tool Subagent allowlist     # send_to_sibling dropped
  provider.execute(tools = 5)
  read/list/ask_parent/report_result/finish
  no spawn, no write, no shell
  complete -> task result; write to branch bank
```

### 4.5 ToolAgent (unreachable) lifecycle

```text
role "tool" -> task_agent_tier = ToolAgent       # read path only

production producer:
  plan:   role_class("tool") = None -> E_PERSONA_UNKNOWN
  spawn:  role_class("tool") = None -> E_PERSONA_UNKNOWN
  template: unknown id -> "worker"; "tool" is not special-cased -> worker
  validate_spawn_role: no parent tier accepts "tool" -> E_TOOL_NOT_ALLOWED

if a row is inserted by a non-production path:
  tools = {file_read, file_list, file_write, shell_exec}
  no finish, no report_result; it can work but cannot report or terminate cleanly
```

### 4.6 Remote principal / satellite lifecycle

```text
actual remote principal:
  only root_checkpoint "spawn_remote_agent" (or operator HTTP POST /api/remotes)
  create_remote(transport="in_process") -> state="enrolled"
  create_device(class="light"|"switch"|"cover")
  create_grant(capability/action)
  heartbeat -> "online"; request_action -> fixture_apply -> device state
  no task row, no run_id, no parent, no depth, no model, no memory, no TOOL_CALL
  audit row actor = "operator" even when created_by = "hyper:<root>"

actual daemon/satellite:
  root_checkpoint "spawn_daemon" (or operator HTTP POST /api/daemons)
  create(kind="watchdog"|"relay"|"switch"|"satellite") -> state="enrolled"
  operator switch -> "running"; tick_due
  watchdog {"check":"db"} -> ok/failed
  {"kind":"script"} -> allowlisted script, bounded output/timeout
  everything else (incl. plain "satellite") -> degraded "no executable job"/"runtime not implemented"

criterion path (NOT IMPLEMENTED):
  lower-level agent requests remote -> HyperAgent approves -> remote created
  A2A Agent Card / HTTP endpoint / task-message lifecycle
```

---

## 5. Verdict

### 5.1 Is the hierarchy genuine, or are these role labels?

**It is a genuine hierarchy, but the names are not the standard definitions.**

Genuine evidence:

1. **One root per run**, enforced by a unique partial index and by the spawn matrix — `db.rs:633-643`, `execution.rs:195-226`.
2. **Persisted class/rank/profile** on every task insert, with a DB trigger — `db.rs:298-383`, `store.rs:1189-1208`.
3. **Persisted ownership**: hyper owns the run, super owns a branch, agent/subagent own none — `db.rs:566-609`.
4. **Strict spawn matrix** at every production insert path: plan, command, model `spawn_agent`, template insert — `execution.rs:195-226`, `execution.rs:1440-1513`, `execution.rs:1833-1895`, `ws.rs:268-300`.
5. **Hard caps** on depth/fan-out/agents, checked before row creation — `caps.rs:52-63`, `execution.rs:1474-1486`.
6. **Real parent-child graph edges and readiness dispatch**: a parent runs only after its direct children — `execution.rs:1944-1955`.
7. **Real memory partition**: run/hive/branch banks, with sibling branch entries excluded from the read set — `store.rs:2175-2228`, `store.rs:2686-2780`.
8. **A real root final synthesis** for multi-node plans, answer-only — `execution.rs:2043-2121`, `provider_executor.rs:615-641`.

So this is not merely a UI skin: a `"worker"` cannot become a `"super"`, a second Hyper owner is blocked, a Subagent cannot spawn, a SuperAgent cannot skip to Subagent, and the root is unique. The hierarchy is enforced by the engine, not just drawn on a canvas.

**But the labels are role labels, not the external concepts:**

- `HyperAgent` is a root planner/synthesizer, not a Meta hyperagent (no meta-agent, no self-edit, no evolution).
- `SuperAgent` is a depth-1 branch lead, not DeepSeek’s proactive work system and not NVIDIA’s high-level “super agent”.
- `Agent` is already a delegated child; the only agent that owns a turn is the root.
- `Subagent` is a bottom leaf rank, not the generic “any delegated child”.
- `ToolAgent` is a dormant tier, not a level.
- `RemoteAgent`/`satellite` are a tree-free fixture control plane and inert daemon records, not workers.

And there is a deeper identity issue: **the persisted class is not always the authority**. `task_agent_tier` and `seed_task_context` still derive from the role string (`execution.rs:235-250`, `execution.rs:1208-1212`), so `"super"` and `"superagent"`—same DB class—get different prompts and different effective tool grants. The hierarchy is real, but its identity key is still partly a label.

### 5.2 Per-level verdict

| Level | Is it a real runtime level? | Is it what the name/claim says? | Verdict |
|---|---|---|---|
| **HyperAgent** | Yes: one root task, run owner, final synthesizer | No: not a Meta hyperagent, not a continuous orchestrator, not a fully audited tool user | **Root planner/synthesizer with an overstated name** |
| **SuperAgent** | Yes: depth-1 branch lead, hive bank, real spawn/aggregate | Partly: a genuine sub-orchestrator, but not an owner of a branch plan/budget/rollup; prompt/tools depend on role spelling | **Real branch lead; overclaims “ownership”** |
| **Agent** | Yes: depth-2 worker, 9 tools, approvals, branch bank | No: not an ordinary top-level agent; always a delegated child; wire kind is `worker` | **Delegated worker** |
| **Subagent** | Yes: depth-3 narrow leaf, 5 tools, no spawn | Partly: a real rank, not the generic subagent concept; persona/tier mismatch on `send_to_sibling` | **Real narrow leaf rank** |
| **ToolAgent** | No: no shipped producer | No: a 4-tool allowlist with no terminal verbs and no creation path | **Dormant non-level** |
| **Remote/satellite** | Only as a tree-free control/record lane | No: not a worker, not A2A, not lower-level-requestable; audit misattributes to operator | **Fixture principal + inert daemon record** |

### 5.3 No AGI overclaim

Nothing in the pinned code supports an AGI claim:

- No self-modification, no meta-agent, no editable improvement procedure (contrast Meta’s hyperagent).
- No learning, fine-tuning, memory-driven self-improvement, or open-ended evolution loop.
- No persistent autonomous process: a run is one prompt, one plan, one bounded dispatch, one terminal status.
- No general tool autonomy: leaf tools are allowlisted and approval-gated; root tools are four fixed local actions.
- No global world model or shared agent bus; memory is run/hive/branch-scoped, with bounded reads.
- The “hive” and “HyperAgent” language is product framing, not evidence of general intelligence.

The accurate description is: **a deterministic, run-scoped, multi-agent task orchestrator with LLM planning, bounded delegation, approval-gated tools, and a persisted task hierarchy.** It is more than role labels in its enforcement, and less than its borrowed names in its autonomy.

### 5.4 Final taxonomy sentence

**HiveMind is a genuine four-rank task hierarchy (hyper → super → agent → subagent) with a dormant fifth tier and a tree-free remote/daemon lane, wearing names from Meta, DeepSeek, NVIDIA, and general agent literature that mean different things; the hierarchy is real, the taxonomy is mislabeled, and the system is not an AGI.**

---

## 6. Source conflicts

| # | Conflict | Evidence A | Evidence B | Consequence |
|---|---|---|---|---|
| C1 | **Meta hyperagent vs HiveMind HyperAgent** | Meta: task agent + meta agent in one editable self-modifying program — <https://ai.meta.com/research/publications/hyperagents/>; repo files `meta_agent.py`/`task_agent.py` — **READ** | HiveMind: root task `role="planner"`, no meta agent/self-edit — `execution.rs:1780-1794` — **VERIFIED (static)** | The shared name suggests a research lineage that the code does not implement |
| C2 | **DeepSeek Super-agent vs HiveMind SuperAgent** | DeepSeek: repository-scoped proactive work system; `Main Session` orchestrator / `Executor Session` worker; “research prototype, not an official product” — **READ** | HiveMind: depth-1 branch-lead task with `spawn_agent` and hive bank — `execution.rs:1440-1513`, `personas.json:21-30` — **VERIFIED (static)** | “SuperAgent” does not mean the same thing; DeepSeek’s term is a system, not a rank |
| C3 | **NVIDIA “super agent” vs HiveMind SuperAgent** | NVIDIA glossary: general-purpose high-level orchestrator (Claude Code/Codex) vs specialized “sub-agent” — **READ (proxy)**; NVIDIA Research uses “orchestrator” — **READ** | HiveMind: SuperAgent is a mid-level branch lead; HyperAgent is the high-level root — `execution.rs:195-226` — **VERIFIED (static)** | HiveMind’s HyperAgent is closer to NVIDIA’s super agent than HiveMind’s SuperAgent is; no official NVIDIA “SuperAgent” product term found |
| C4 | **Generic Agent/Subagent vs HiveMind ranks** | OpenAI: agent = LLM + instructions + tools + handoffs; sub-agent via handoff/agents-as-tools — **READ**; Anthropic: subagent = delegated specialized assistant with own context/prompt/tools — **READ** | HiveMind: Agent is depth 2, Subagent is depth 3; only the root owns the final turn — `execution.rs:195-226`, `execution.rs:2048` — **VERIFIED (static)** | HiveMind’s “Agent” is already a subagent in the generic sense; the labels are ranks, not standard roles |
| C5 | **A2A remote agent vs HiveMind remote** | A2A: remote agent = HTTP endpoint implementing A2A, opaque, Agent Card, task lifecycle — **READ** | HiveMind: `in_process` fixture principal/device/grant; no network/model/Agent Card — `remote.rs:1-6,308-315` — **VERIFIED (static)** | “Remote agent” in HiveMind is not an A2A remote agent; the tree-free property is real, the agent property is not |
| C6 | **Local docs vs current code** | Docs: 4 tiers, Subagent not a separate tier, no persisted class, no hive bank, remote zero code — `docs/provider/AGENT-ARCHITECTURE-TRUTH.md:18-31`, `docs/research/agent-definitions/VERIFIED.md:55-59,121-140`, `docs/plans/HIVEMIND-MASTER-FINISH-PLAN.md:97`, `docs/ui/HIVEMIND-UI-GAP-AUDIT.md:218` — **READ (local docs)** | Code: 5 tiers (`tool.rs:72-83`), v13 class (`db.rs:298-383`), v14/v17 banks/ownership (`db.rs:393-471,566-609`), remote/daemon (`remote.rs`, `daemon.rs`) — **VERIFIED (static)** | Readers of the docs get a materially wrong taxonomy; the docs must be superseded, not trusted |
| C7 | **Prompts vs runtime** | `ROOT_SYSTEM` says the root may call `spawn_remote_agent` in the final turn — `provider_executor.rs:46-52`; planner prompt says 8 nodes — `planner.prompty:17`, `provider_executor.rs:32`; `caps.rs:8` says “Hyper → Super → Worker” | `finalize_root` advertises no tools — `provider_executor.rs:615-641`; `Plan::validate` allows 40 — `planner.rs:10,50-56`; the chain includes Subagent — `tool.rs:80`, `execution.rs:217-219` | The model is told things the runtime does not enforce/provide; prompt claims are not contracts |
| C8 | **Role string vs persisted class** | DB v13 persists `class/rank/profile_id` and maps `super` → `superagent` — `db.rs:298-383` | `task_agent_tier` and `seed_task_context` derive from `task.role`; `super` finds no persona, `superagent` does — `execution.rs:235-250,1208-1212,825-846` | The persisted class is not the single source of truth; two spellings of the same class have different prompts/tools |

---

## 7. Evidence gaps and what closes them

| # | Gap | Why it matters | What closes it |
|---|---|---|---|
| E1 | **No runtime execution in this audit.** No build, test, server, provider call, DB migration, remote action, or UI action. | All behavior claims are static/inferred; a code path could be unreachable at runtime for a reason not visible in the source. | Run the targeted tests (`cargo test -p hiverouter-core --locked`) and a fake-provider harness; record commands and output. No provider key/TokenLab points required. |
| E2 | **Meta PDF not read.** The abstract, Meta publication page, and repo README were read; the PDF fetch was unsupported. | The definition could contain nuance about hierarchy/delegation that the abstract omits. | Download and read the PDF in a browser/pdf tool; quote the section on task agent/meta agent. |
| E3 | **NVIDIA glossary read via proxy.** Direct official HTML was truncated; the body came through `r.jina.ai`. | The exact wording/status of “super agent” matters for the comparison. | Re-read <https://www.nvidia.com/en-us/glossary/ai-agents/> in a browser; quote the “super agent”/“sub-agent” paragraph. Also search for any NVIDIA page that actually uses “SuperAgent” as a product term; none was found. |
| E4 | **DeepSeek article not read.** The repo README references “Exploring Super-agent Design,” but no official article URL was fetched. | The repo docs are sufficient for the domain model, but the article may add design intent. | Locate and read the official DeepSeek article; if unavailable, mark it UNVERIFIED and rely on the repo. |
| E5 | **No test for multi-spawn root control (G04).** No test returns four `spawn_superagent` calls on a flat plan. | The “one extra branch” claim is unproven at runtime; the code suggests up to four. | Add a scripted executor returning four calls; assert exactly one Super task row. |
| E6 | **No test for `super` vs `superagent` effective grants (G08).** Tests use `"super"` but do not assert persona/tool scope. | The authority divergence is the sharpest taxonomy defect; it needs a RED/GREEN pin. | Seed a `"super"` and a `"superagent"` context; assert system prompt and effective/advertised tool sets; then fix by `profile_id`. |
| E7 | **No test for root tool audit/caps (G05).** No test counts `TOOL_CALL` events or `run_tool_counters` for root tools. | “Root tools are tools” is currently a code comment, not a contract. | Subscribe to the bus during a root control turn; assert presence/absence of `TOOL_CALL`/`TOOL_RESULT`; assert counter consumption. |
| E8 | **No negative test for lower-level remote request (G12).** No test proves an Agent/SuperAgent cannot request a remote. | The task criterion explicitly includes “requested by a lower level and approved by HyperAgent.” | Add a test that `spawn_remote_agent` is absent from every non-root tool set and that `ask_parent` cannot create a remote. |
| E9 | **No satellite daemon runtime test (G13).** The tests cover a no-job daemon and a script daemon, not `kind="satellite"`. | The “satellite” label currently implies a worker-like role that does not exist. | Create a `satellite` daemon, enable it, tick it, assert `degraded`/`E_DAEMON_JOB_UNIMPLEMENTED` (or implement a real job). |
| E10 | **No one-node root test (G06).** The gate is `has_aggregators || plan.nodes.len() > 1`; no test asserts the single-node policy. | The “HyperAgent always answers” claim depends on this. | Add a one-node plan test; assert the root final count and the message attribution policy. |
| E11 | **No chat attribution test/schema (G06).** `chat_messages` has no `task_id`/`is_final`. | “Authoritative answer” is currently last-message order. | Add `task_id`/`is_final` (or a `final_answer_id` on the run) and a test that only the root final is marked. |
| E12 | **No A2A conformance test (G12).** The remote has no Agent Card/HTTP endpoint. | If “remote agent” is claimed, the A2A definition is the primary comparison. | Add an A2A adapter/Agent Card and a conformance test, or rename the feature to “remote principal/device.” |
| E13 | **Tree moved and working tree is dirty.** HEAD advanced to `197ee28`; `execution.rs` has an uncommitted two-line prompt-wording diff, `remote.rs` has an apparent audit-actor fix, and `tests/run_lifecycle.rs` has matching assertions. | Line citations are still valid for `919c339`, but a future reader could see different text/structure. | Re-pin to the intended commit, re-run the audit’s citation check, and record the full diff. |
| E14 | **Post-pin live-test doc not independently reproduced.** `docs/audit/AGENTIC-LIVE-TESTS.md` (added at `197ee28`) reports a 7-task exact hierarchy and a root branch refusal. | It is a useful signal but not evidence produced by this audit; it covers the “plan already has branches” case, not the flat-plan multi-spawn case. | Reproduce with a fake provider; extend the test matrix to the flat-plan case and the exact-count case. |
| E15 | **No device/transport test beyond the fixture.** The remote adapter is in-process; no network/A2A path exists. | “Remote” could be read as a real network worker. | Add a transport contract test or explicitly document “fixture only, no network.” |

---

## Appendix A — Material claim ledger (grades)

| ID | Claim | Grade | Evidence |
|---|---|---|---|
| L1 | HiveMind has five `AgentTier` variants including `Subagent` and `ToolAgent` | **VERIFIED (static)** | `tool.rs:72-83` |
| L2 | Class/rank/profile and ownership are persisted by migrations | **VERIFIED (static)** | `db.rs:298-383`, `db.rs:566-609` |
| L3 | The spawn matrix is strict at production insert paths | **VERIFIED (static)** | `execution.rs:195-226`, `execution.rs:1440-1513`, `ws.rs:268-300` |
| L4 | The root is created after the planning call; planning is detached from the root task | **VERIFIED (static)** | `execution.rs:1747-1794` |
| L5 | The root control turn is one round, ≤4 calls, result discarded, no tool audit | **VERIFIED (static)** | `execution.rs:3857-4001`, `execution.rs:3996-3998` |
| L6 | Root tools bypass `ToolRegistry` and `MAX_CALLS_PER_RUN` | **VERIFIED (static)** | `execution.rs:3854-3856`, `execution.rs:2417-2460`, `tool_loop.rs:16` |
| L7 | Task completions with non-empty text append a run-level assistant message; there is no task/final attribution | **VERIFIED (static)** | `execution.rs:4337-4341`, `db.rs:75-82` |
| L8 | SuperAgent persona has 7 tools; tier has 9; `super` finds no persona | **VERIFIED (static)** | `personas.json:21-30`, `tool.rs:399-412`, `execution.rs:825-846`, `execution.rs:1208-1212` |
| L9 | Subagent is real but its persona lists a tool the tier forbids | **VERIFIED (static)** | `tool.rs:427-436`, `personas.json:32-39` |
| L10 | ToolAgent has no shipped producer and no terminal verbs | **VERIFIED (static)** | `tool.rs:80-82,437-445`, `execution.rs:152-166`, `ws.rs:272-279` |
| L11 | Remote is an in-process fixture principal, not a model/worker/A2A endpoint | **VERIFIED (static)** | `remote.rs:1-6,308-315,1358-1400` |
| L12 | Root-created remote audit is attributed to `operator` | **VERIFIED (static)** | `remote.rs:352-355,424-426,489-491,584-586` |
| L13 | Satellite daemon has no executable job runtime | **VERIFIED (static)** | `daemon.rs:154,304-330`, `execution.rs:3960-3964` |
| L14 | Local truth docs are stale relative to `919c339` | **VERIFIED (static)** | `AGENT-ARCHITECTURE-TRUTH.md:18-31`, `VERIFIED.md:55-59,121-140`, `MASTER-FINISH-PLAN.md:97`, `UI-GAP-AUDIT.md:218` |
| L15 | Meta hyperagent is a task+meta editable program, not a four-level hierarchy | **READ** | <https://ai.meta.com/research/publications/hyperagents/>; <https://github.com/facebookresearch/Hyperagents> |
| L16 | DeepSeek Super-agent is a research prototype work system, not a rank | **READ** | <https://github.com/deepseek-ai/dsh-superagent-prototype>; `docs/001-domain.md` |
| L17 | NVIDIA uses “super agent” generically and “orchestrator” in research; no official SuperAgent product term found | **READ (proxy for glossary; READ for blog)** | <https://www.nvidia.com/en-us/glossary/ai-agents/>; <https://developer.nvidia.com/blog/train-small-orchestration-agents-to-solve-big-problems/> |
| L18 | A2A remote agent is an HTTP/A2A endpoint with an Agent Card, opaque to the client | **READ** | <https://a2a-protocol.org/latest/topics/key-concepts/> |
| L19 | The root final memory write can resolve to an orphan branch bank instead of the run bank | **VERIFIED (static)** | `execution.rs:4354-4363`, `store.rs:1345-1352`, `store.rs:2712-2747`, `store.rs:261-265` |

---

## Appendix B — Audit limitations

1. **Static only.** No code was executed. Every “runs/does” sentence is a source-read consequence, not an observed run.
2. **No runtime evidence produced.** The only runtime-adjacent document read is `docs/audit/AGENTIC-LIVE-TESTS.md`, added after the pinned commit; it is not independent evidence for this audit.
3. **NVIDIA wording is proxy-read.** The exact glossary paragraph should be re-read in a browser before external publication.
4. **The Meta PDF was not read.** The abstract, publication page, and repository README are the basis for the Meta comparison.
5. **The DeepSeek article referenced by the repo was not read.** The repository’s own domain doc is the basis.
6. **The tree moved.** `919c339` → `197ee28` (docs-only) plus uncommitted changes in `execution.rs` (prompt wording), `remote.rs` (apparent audit-actor fix), and `tests/run_lifecycle.rs` (matching assertions). The line numbers in this audit are compatible with `919c339`, but the intended review commit should be re-pinned.
7. **No provider key, TokenLab point, server, browser, or device was used.** Live behavior remains UNVERIFIED.

---

*End of audit — task-17.*
