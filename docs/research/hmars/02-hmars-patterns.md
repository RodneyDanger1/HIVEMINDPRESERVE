# 02 — HMARS patterns: hierarchies, registries, handoffs, termination

**Researcher:** web researcher #2 for the HMARS goal.
**Written:** 2026-11-07. **Scope:** research only. No product code was edited, no paid API was called.
**Question set:** orchestrator/supervisor patterns, sub-orchestrators, worker/subagent roles, agent
registries, A2A, shared buses, branch memory, task DAGs, handoff and termination semantics — and what the
HMARS goal architecture should be, what is unnecessary complexity, which failure modes must be designed
out, and which invariants hold.

HMARS is used here as the **goal name** for HiveMind's hierarchical multi-agent routing system. This
document does not expand the acronym; no authoritative source was found that defines it, and inventing an
expansion would be a fabricated fact. Everything below is about the system the repository already calls
HyperAgent → SuperAgent → Worker, plus `ask_parent` / `send_to_sibling`.

---

## 0. Evidence classes and how to read citations

| Mark | Meaning | Rule |
|---|---|---|
| **VERIFIED** | Claim read on the official page/spec/paper at the cited URL, fetched 2026-11-07 | Safe to build on |
| **READ** | Claim read in this repository at `path:line` | Safe to build on, may change with code |
| **SECONDARY** | Vendor engineering blog, not a spec or peer-reviewed paper | Useful signal, not a contract |
| **UNVERIFIED** | Not proven here | Do not build on it; listed in §13 |

Source IDs (`[A2A-1]`, `[MAST-1]`, …) resolve in the source table, §12. Every source row states its class
(SPEC / PAPER / OFFICIAL-DOCS / SECONDARY-BLOG) and its fetch status.

Three framing rules used throughout:

1. A **pattern** is adopted only if it removes a failure mode HiveMind can actually reach. Patterns that
   only exist to serve multi-vendor, cross-organization, or multi-runtime deployments are cut in §7.
2. **Caps belong in code, not in prompts.** Every framework examined that relies on the model to bound its
   own fan-out has published a story about the model not doing that (Anthropic: "spawning 50 subagents for
   simple queries", **SECONDARY** [ANTH-2]). HiveMind already puts caps in `caps.rs`; that is the correct
   shape and should not regress.
3. **A message is data, never authority.** The one framework that ships adversarial-output handling in
   public docs (Claude Code) marks subagent reports as untrusted prose and states that instructions or
   approval claims inside them carry no authority [CC-1]. This is the single most important security
   invariant for a system that shows agent text to other agents.

---

## 1. Executive answer

**The goal architecture should be: one root orchestrator, at most two orchestrator tiers below it, a
declarative but inert profile registry, a task DAG that is the single source of truth for what exists, two
and only two inter-agent verbs (delegate-and-wait, escalate-and-wait; peer notify is a bounded third), typed
result rows instead of shared transcripts, immutable terminal states, and code-enforced budgets on depth,
fan-out, agent count, provider calls, message count and wall-clock time.**

Concretely, compared to what HiveMind has today:

- **Keep** the plan DAG, the depth/fan-out/agent/call caps, per-task `ScopedContext`, `task_result` rows,
  `agent_profiles` as a validated record store, the `ask_parent`/`send_to_sibling` verbs, and the rule that
  a terminal task never transitions again. These are the parts every authoritative system independently
  converged on. HiveMind is closer to the reference model than most frameworks are.
- **Fix** the one genuinely broken primitive: `ask_parent` writes a durable message and returns a
  `message_id` — it never gets an answer, and nothing delivers that message into the parent's next prompt
  (**READ** `execution.rs:2417-2487`, `store.rs:538-590`, `http.rs:674-690`). A child that asks a question
  currently cannot be unblocked by the answer. That is a design defect, not a missing feature: it converts
  "ask for clarification" (MAST FM-2.2) into "guess or fail".
- **Do not build** an agent pub/sub bus, a registry service with its own API, a fourth tier, a vector memory
  layer, semantic routing, or an internal A2A wire protocol. Each of these is either absent from the systems
  that succeed or exists to solve a problem HiveMind does not have (multi-vendor federation). §7 lists them
  with the reason each is cut.
- **Add** exactly four things that are missing and load-bearing: (a) a parent-visible answer path for
  escalations with correlation and deadline; (b) a no-progress/deadline terminator that cannot be talked out
  of by a model; (c) per-branch context budgeting with artifact-by-reference; (d) a first-class "handoff
  kind" on the tool surface and in the event contract, so delegate/escalate/notify cannot silently drift into
  each other.

The 20 invariants in §9 are the testable form of that answer. The prioritised list is §11.

---

## 2. The reference model: six primitives every serious system has

Across the A2A specification, OpenAI Agents SDK, LangGraph/LangChain, AutoGen, Magentic-One, Claude Code,
and MetaGPT, the same six primitives recur. They are presented here as the vocabulary the rest of the
document uses.

| # | Primitive | What it is | Canonical evidence |
|---|---|---|---|
| P1 | **Unit of work = Task with a lifecycle** | A stateful, identifiable unit that reaches *interrupted* states (needs input/auth) or *terminal* states, and is immutable once terminal | A2A: Message-vs-Task response, terminal `completed`/`canceled`/`rejected`/`failed`, "once a task reaches a terminal state … it cannot restart" [A2A-2]; HiveMind task rows + `run_state` guard (**READ**) |
| P2 | **Identity = a declarative card/record** | Name, description, skills/capabilities, and an endpoint or binding — data, not behaviour | A2A `AgentCard` (identity, url, capabilities, auth, skills) [A2A-3]; HiveMind `Persona` [HM-1]; Claude Code subagent frontmatter [CC-1] |
| P3 | **Two control verbs, not one** | *Delegate* (call a subordinate, get a result back) vs *hand off* (transfer control/history to a peer). Systems that conflate them get both wrong | OpenAI: `Agent.as_tool()` vs `handoff()`, "as though the new agent takes over the conversation" [OAI-1]; LangChain: subagents-as-tools replaced the supervisor package [LG-1] |
| P4 | **Data plane = artifacts/results, not transcripts** | Concrete outputs are addressable objects; prose is not the transport | A2A `Artifact` with `artifactId` [A2A-3]; Anthropic: subagents write to a filesystem and pass lightweight references [ANTH-2] (**SECONDARY**); HiveMind `task_result` rows (**READ**) |
| P5 | **Supervision loop with an explicit ledger** | The orchestrator records plan, progress, and next action; on stall it replans rather than retrying | Magentic-One Task Ledger + Progress Ledger, outer/inner loop, replan on stalled progress [M1-1] |
| P6 | **Hard bounds outside the model** | Depth, concurrency, steps, and time are enforced by the runtime; refusals are named | Claude Code: 3 nesting layers default, 20 concurrent default, "do not retry" error [CC-1]; LangGraph `recursion_limit` → `GRAPH_RECURSION_LIMIT` [LG-2]; HiveMind `caps.rs` (**READ**) |

Two derived observations matter for HMARS:

- **P3 is where most home-grown systems break.** HiveMind's `spawn_agent` is unambiguously a *delegate*
  (the parent awaits the child inline and receives a bounded result, **READ** `execution.rs:2655-2691`),
  while `send_to_sibling` is a *notify* with no control transfer and no reply. That is actually the correct
  split; what is missing is that nothing in the tool schema or event contract says so. §6 and §11 turn that
  implicit split into an explicit one.
- **P5 can be deterministic.** Magentic-One spends model calls on the progress ledger. HiveMind does not
  need that: spawn counters, call counters, elapsed time and repeated-instruction hashes give the same
  termination guarantee without an extra LLM turn per step. This is one of the places where copying the
  published pattern literally would add cost for no reliability.

---

## 3. Pattern inventory, by system

Each row is a claim read at the cited URL (fetched 2026-11-07) unless marked otherwise.

### 3.1 A2A — the protocol-level shape (SPEC)

| Aspect | What the spec establishes | Consequence for HMARS |
|---|---|---|
| Response type | An agent answers with a stateless `Message` or a stateful `Task`; hybrid agents negotiate with messages then commit to a task [A2A-2] | Keep HiveMind's split: chat reply vs task row. Do not model every exchange as a task. |
| Terminal states | `completed`, `canceled`, `rejected`, `failed`; interrupted states include `input-required`, `auth-required` [A2A-2] | HiveMind has Completed/Failed/Cancelled/Killed plus AwaitingApproval — equivalent coverage, better refusal codes. |
| Immutability | "Once a task reaches a terminal state … it cannot restart. Any subsequent interaction related to that task … must initiate a new task within the same `contextId`" [A2A-2] | This is HiveMind's `is_terminal_task_state` guard (**READ** `execution.rs:93`, `:2984`) — keep it and test it as invariant I5. |
| Grouping | `contextId` groups related tasks; `taskId` identifies one; `referenceTaskIds` points at prior work [A2A-2] | Maps to `run_id` (grouping) + `task_id`; the missing analogue is a parent-visible `in_reply_to` on messages. |
| Discovery | `AgentCard` at `/.well-known/agent-card.json` (RFC 8615); curated registries are described but "the current A2A specification does not prescribe a standard API for curated registries" [A2A-4] | A registry *format* is standard; a registry *service* is not. HiveMind should not invent the latter. |
| Interaction | request/response with polling, SSE streaming, push-notification webhooks [A2A-3] | HiveMind's UI event stream is a legitimate equivalent for the observation channel; it is not an agent bus. |
| Opacity | to the client, the remote agent is a black box — "its internal workings, memory, and tools stay hidden" [A2A-3] | This is the strongest argument *against* a shared agent bus: the protocol deliberately does not give peers a shared context. |

**Reading of the protocol for HMARS:** A2A is a federation protocol between *organizations*. Its valuable
transferable content is the task lifecycle and the card, not the transport. Adopting JSON-RPC internally
inside a single Rust binary would add a serialization boundary, a versioning surface and an auth surface in
exchange for nothing HiveMind needs. The card, however, is a very good target for HiveMind's existing
`Persona` record (§6.3).

### 3.2 OpenAI Agents SDK — delegate vs handoff, stated precisely (OFFICIAL-DOCS)

| Aspect | What the docs establish | Consequence for HMARS |
|---|---|---|
| Handoff shape | Handoffs are exposed to the model **as tools**, named `transfer_to_<agent>` [OAI-1] | HiveMind already exposes delegation as a tool (`spawn_agent`). Correct. |
| History | On handoff "it's as though the new agent takes over the conversation, and gets to see the entire previous conversation history"; `input_filter` can rewrite it [OAI-1] | Full-history transfer is the default *because* it is a peer control transfer. Delegation (`as_tool`) deliberately does **not** transfer the conversation. HiveMind's `send_to_sibling` transfers neither control nor history — it is weaker than a handoff, and should be documented as "notify", not "hand off". |
| Metadata vs state | `input_type` carries model-chosen metadata (`reason`, `priority`); application state belongs in `RunContextWrapper.context` [OAI-1] | Separate "what the model says about the delegation" from "what the runtime knows". HiveMind does this implicitly (`parent_task_id` is a claim, the run row is authoritative, **READ** `execution.rs:2635-2646`). |
| Auth pitfall | `is_enabled` is evaluated before arguments exist, so it "cannot authorize values inside an argument-bearing handoff"; do the check in `on_handoff` before side effects [OAI-1] | Any HiveMind approval gate keyed on *tool name* alone is insufficient; it must see the resolved arguments (HiveMind's cross-branch check does, **READ** `execution.rs:2494-2507`). |
| Guardrail scope | Handoffs stay within a single run; input guardrails apply only to the first agent, output guardrails only to the agent producing final output [OAI-1] | A hierarchy must not assume per-hop guardrails exist. HiveMind's equivalent is per-tool permission class, which does apply per call — keep it that way. |
| History compaction | `nest_handoff_history` is opt-in beta; compacts history into summary segments; "does not redact sensitive data" [OAI-1] | Compaction is a context-budget tool, not a security control. |

### 3.3 LangGraph / LangChain — the supervisor pattern has already moved on (OFFICIAL-DOCS)

| Aspect | What the docs establish | Consequence for HMARS |
|---|---|---|
| Supervisor status | `langgraph-supervisor` is "no longer actively maintained"; the recommended pattern is subagents wrapped as tools on a main `create_agent` [LG-1] | The industry's most-used supervisor package converged on *delegate-as-tool*, i.e. HiveMind's `spawn_agent` shape. Do not build a graph-node supervisor. |
| Nesting | Two options: flatten to one supervisor with one tool per leaf, or nest tool calls ("a subagent wrapped as a `@tool` that calls other subagents") [LG-1] | HiveMind's SuperAgent is the nested-tool-call option. It is legitimate, and it is the option with more depth risk — hence the depth cap. |
| Interrupts | For `interrupt` to propagate through nested agents: compile only the outermost graph with a checkpointer; subagents get per-invocation persistence; pass `thread_id` at the top [LG-1] | Exactly one durable owner per run. HiveMind's single run-level snapshot (**READ** `execution.rs:1393`) matches this; do not add per-agent checkpointers. |
| Output mode | `output_mode` (`full_history` vs `last_message`) migrates into the tool wrapper — the wrapper decides what the parent sees [LG-1] | The boundary is the *right* place to bound what a parent sees. HiveMind's `bounded_tool_text` (**READ** `execution.rs:2689`) is that boundary. |
| Limits | `recursion_limit` is configurable; exceeding it raises `GRAPH_RECURSION_LIMIT` with explicit troubleshooting ("you likely have a cycle") [LG-2] | Fail loudly with a named error, never silently stop. HiveMind's `Refusal::CallLimit` is equivalent. |

### 3.4 AutoGen — teams, and the messaging semantics that actually differ (OFFICIAL-DOCS)

| Aspect | What the docs establish | Consequence for HMARS |
|---|---|---|
| Team presets | `RoundRobinGroupChat`, `SelectorGroupChat`, `MagenticOneGroupChat`, `Swarm` (handoff messages) [AG-1] | Group chat is a *shared context* design; it erases parent/child attribution. HiveMind should not adopt it (see §7). |
| Shared context | In round robin, "all agents share the same context and take turns responding… Each agent … broadcasts its response to all other agents" [AG-1] | This is the canonical context-blowup architecture. It works for short debates, not for 64-task hierarchies. |
| Termination | First-class objects: `TextMentionTermination`, `ExternalTermination`, `TextMessageTermination`; `reset()`, resume, abort via `CancellationToken` [AG-1] | Termination should be a value in the system, not a prompt sentence. HiveMind's terminal state machine is the stronger version of this. |
| Direct messaging | Awaiting a direct send returns the handler's return value; if the receiver raises, "the exception will be propagated back to the sender" [AG-2] | Request/response with error propagation = HiveMind `spawn_agent` returning `ToolExecution::refused`. |
| Broadcast | One-way; "if a response is given to a published message, it will be thrown away"; a publisher does not receive its own message "to prevent infinite loops"; handler exceptions are logged, not propagated [AG-2] | If HMARS ever gets a bus, these two rules are mandatory: **no self-delivery** and **failures must not vanish**. Today's HiveMind has no agent bus at all, which is safer. |
| Start simple | "start with a single agent for simpler tasks, and transition to a multi-agent team when a single agent proves inadequate" [AG-1] | Directly supports §7's "cut complexity" list. |

### 3.5 Magentic-One — the ledger pattern and stall detection (OFFICIAL-DOCS; paper cited)

| Aspect | What the docs establish | Consequence for HMARS |
|---|---|---|
| Ledgers | Orchestrator keeps a **Task Ledger** (facts + guesses + plan) and a **Progress Ledger** (self-reflection on progress, completion check) [M1-1] | HiveMind's plan DAG + `task_result` rows are a Task Ledger. The Progress Ledger is the missing half. |
| Stall handling | "If the Orchestrator finds that progress is not being made for enough steps, it can update the Task Ledger and create a new plan" [M1-1] | Replan-on-stall is the correct answer to "the hierarchy is spinning". A deterministic stall counter achieves it without per-step LLM calls. |
| Risk posture | Docs warn about prompt injection from webpages and recommend containers, restricted access, human oversight [M1-1] | Confirms that an orchestrator that ingests untrusted text is the highest-risk component. In HiveMind that component is the parent ingesting child reports. |

### 3.6 Claude Code — the most operationally specific hierarchy in public docs (OFFICIAL-DOCS)

This is the closest real-world analogue to HMARS: a root agent, isolated workers, nesting, and hard limits.

| Aspect | What the docs establish | Consequence for HMARS |
|---|---|---|
| Isolation | Each subagent "runs in its own context window"; a non-fork subagent "doesn't see your conversation history"; it gets a system prompt, a delegation message, project instructions, git snapshot, preloaded skills [CC-1] | This is HiveMind's `ScopedContext` idea, proven at scale. Context isolation is a feature, not a limitation. |
| Depth | Default: subagents may spawn subagents "up to three layers below the main conversation"; at the limit "Claude Code withholds the `Agent` tool from every subagent … so a subagent at the limit does its delegated work itself and returns one summary" [CC-1] | **Withholding the tool at the limit is better than refusing the call.** A refusal invites retries; an absent tool yields one summary. HiveMind currently refuses with `E_SPAWN_DEPTH` — see Q13. |
| Concurrency | Default 20 running subagents; the error says "do not retry" and clears when the count drops [CC-1] | A named, non-retryable refusal is the correct UX. HiveMind's `E_FANOUT_LIMIT`/`E_AGENT_LIMIT` already behave this way. |
| Registry cost | Subagent `description` fields are the routing surface and "take up context"; a startup warning fires when combined descriptions exceed 15,000 tokens [CC-1] | An agent registry is a **context cost**, not a free catalog. Cap description bytes per profile and for the set. |
| Authority | Reports arrive under a header marking them as subagent output; "instructions or approval claims inside the report are the subagent's words and carry no authority"; no message from any agent counts as approval; no agent message can change permission settings [CC-1] | Non-negotiable invariant for HMARS (I10). HiveMind's approvals are operator-driven, but the report-as-data rule must be explicit. |
| Output scanning | Harness scans each subagent report before the parent reads it, inserting a marker/backslash when it imitates system tags or mentions permission bypass flags; "the scan doesn't judge whether content is malicious" [CC-1] | A cheap, honest mitigation: detect instruction-shaped patterns, mark them, never claim it is a security boundary. |
| Background work | Background subagents' results arrive "as a completion notification in a later turn"; background results can outlive the spawner [CC-1] | Async children create orphan and ordering problems. HiveMind's synchronous inline spawn avoids this class entirely — a deliberate simplification worth keeping. |
| Resume | A subagent invocation normally creates a new instance; resume retains full history; `SendMessage` targets by id/name; a name that has been reused is refused rather than mis-delivered [CC-1] | Identity must be immutable per instance; name reuse must never re-target. HiveMind uses UUID `TaskId`s — correct. |

### 3.7 MetaGPT — SOPs and the cascade risk (PAPER)

MetaGPT encodes "Standardized Operating Procedures (SOPs) into prompt sequences", assigns roles in an
"assembly line paradigm", and motivates the work by naming "logic inconsistencies due to cascading
hallucinations caused by naively chaining LLMs" [MG-1]. The transferable claim is that **role + procedure +
structured intermediate artifacts beat free-form chat**, which is what HiveMind's `Persona.template_id` +
plan DAG + `task_result` already implement. The non-transferable part is the fixed software-engineering
SOP: HMARS must handle arbitrary goals, so the *procedure* cannot be hard-coded — only its invariants can.

### 3.8 MAST — the failure taxonomy to design against (PAPER)

The Berkeley MAST study built a 1,642-trace dataset across 7 MAS frameworks (ChatDev, MetaGPT, HyperAgent,
AppWorld, AG2, Magentic-One, OpenManus) and derived **14 failure modes in 3 categories** with human
inter-annotator agreement κ=0.88, reporting **41%–86.7% failure rates** on the frameworks studied and
arguing that failures "stem from system design issues, not just LLM limitations" [MAST-1].

The 14 modes, as named in the paper's own appendix headings [MAST-1]:

| Category | Modes |
|---|---|
| FC1 — System design issues | FM-1.1 Disobey task specification · FM-1.2 Disobey role specification · FM-1.3 Step repetition · FM-1.4 Loss of conversation history · FM-1.5 Unaware of stopping conditions |
| FC2 — Inter-agent misalignment | FM-2.1 Conversation reset · FM-2.2 Fail to ask for clarification · FM-2.3 Task derailment · FM-2.4 Information withholding · FM-2.5 Ignored other agent's input · FM-2.6 Reasoning-action mismatch |
| FC3 — Task verification | FM-3.1 Premature termination · FM-3.2 No or incomplete verification · FM-3.3 Incorrect verification |

Two of these map *directly* onto HiveMind defects rather than hypothetical risks:

- **FM-2.2 (fail to ask for clarification)** is structurally guaranteed by the write-only `ask_parent`
  (**READ** §6.5): a child that asks cannot receive an answer, so the *only* rational behaviour is to guess.
- **FM-1.5 (unaware of stopping conditions)** is the failure mode that `MAX_CALLS`/`MAX_AGENTS` defend
  against. They defend the *run*; a per-branch stall detector is still missing.

### 3.9 Secondary sources, used only as corroboration (**SECONDARY**)

- Anthropic's production report on its research system: orchestrator-worker with parallel subagents gave
  +90.2% over a single agent on an internal eval; token usage explains 80% of BrowseComp variance;
  multi-agent systems use **~15×** the tokens of chat; early versions **spawned 50 subagents for simple
  queries**; vague delegation caused duplicated and missing work; synchronous execution "creates
  bottlenecks… the entire system can be blocked while waiting for a single subagent"; the mitigations were
  effort-scaling rules, explicit task boundaries and output formats, external artifacts with lightweight
  references, and resumable long-running execution [ANTH-2].
- Anthropic's pattern catalogue: orchestrator-workers is the pattern for "complex tasks where you can't
  predict the subtasks needed"; the guidance is to add complexity "only when it demonstrably improves
  outcomes" [ANTH-1].

These are exactly consistent with the SPEC/OFFICIAL-DOCS sources above and are cited as corroboration, not
as the basis of any invariant.

### 3.10 Context and memory papers (PAPER)

| Source | Claim | Use in HMARS |
|---|---|---|
| Lost in the Middle [LITM-1] | Retrieval performance is highest when relevant information is at the start or end of the context and "significantly degrades" when it is in the middle, even for long-context models | A 64-task parent prompt with 20 child reports in the middle is the worst possible shape. Put the instruction first, the child results last, and bound their count. |
| AgentPoison [POIS-1] | Backdoor attack poisoning long-term memory or RAG; >80% average attack success at <0.1% poison rate, with <1% impact on benign performance | Any memory a worker can write and a parent later reads is an attack surface (see §8.5). |

---

## 4. HiveMind's actual concepts (READ)

Everything in this section was read in this repository on 2026-11-07. Line numbers are current as of this
document; a later refactor invalidates them, not the claim.

| Concept | Implementation | Note |
|---|---|---|
| Tiers | `PersonaTier::{Hyper, Super, Worker, Tool}` — `prompts.rs:367-379` | Four enum values; `Tool` is a tool-invoking actor, not an orchestrator. |
| Tier mapping from role strings | `task_agent_tier` — `execution.rs:151-163`: `planner`/`hyper`/`orchestrator` → Hyper; `super`/`sub_hive` → Super; `tool` → Tool; **everything unrecognized → Worker** | Fail-closed capability mapping. Good. |
| Spawn authorization | `validate_spawn_role` — `execution.rs:113-142`: Hyper→{super, worker, subagent}, Super→{worker, subagent}, Worker→{subagent only}, Tool→never; refusal `E_TOOL_NOT_ALLOWED` | A worker cannot escalate its own capability tier. This is the "no privilege escalation by spawning" rule. |
| Root | one `planner` task at `depth = 0`, created after exactly one planning call — `execution.rs:1313-1335` | Root is the run orchestrator; the plan is model-produced but validated by code. |
| Plan DAG | `PlanNode { node_id, role, deps, depth, parent }` — `planner.rs:14-27`; `validate()` at `:44-100`: ≤40 nodes, unique ids, deps resolve, cycles refused, `parent.depth + 1 == node.depth`, parent chain must not loop | Both a DAG (deps) and a tree (parent) — the tree is the authority hierarchy, the DAG is the ordering. |
| Execution order | leaves in topological order, **sequentially** — `execution.rs:1396-1426`; aggregators deepest-first after their children — `:1431-1485`; post-plan spawned tasks drained sequentially — `:1500` | One task at a time inside a run. Concurrency is not currently a HiveMind property. |
| Single active run | `RunEngine.active_run: Mutex<Option<RunId>>` — `execution.rs:434`; `submit_prompt` refuses when a run is active — `:1032` | Server-wide serialization. Simplifies almost everything; also means a stuck child stalls the only run. |
| Caps | `MAX_SPAWN_DEPTH = 3`, `MAX_FANOUT = 8`, `MAX_AGENTS = 64`, `MAX_CALLS = 40` — `caps.rs:9-15`; `check_spawn` refuses by name — `:49-60` | Enforced before a row exists or a dispatch happens. |
| Per-persona caps | `Persona.max_children` / `max_depth`, validated to be non-zero and **≤ the global cap** — `prompts.rs:398-404`, `:421-432`; applied in `persona_spawn_limits` — `execution.rs:550` | A profile can tighten, never loosen. Correct direction. |
| Delegate | `spawn_agent` tool — `execution.rs:2618-2729`; claim check `parent_task_id` — `:2635-2647`; `spawn_sub_hive` enforces caps — `:1061`; child is taken off the deferred queue and awaited inline — `:2668-2681`; result returned as bounded JSON `{child_task_id, role, depth, state, result}` — `:2684-2691` | A synchronous, result-returning call. This is the strongest primitive in the system. |
| Escalate | `ask_parent` — `execution.rs:2417-2487`; args `{text}` only — `tool.rs:223-229`; requires `parent_task_id` — `:2433`; writes `agent_messages` and returns `{message_id, to_task_id, kind}` — `:2465-2479` | **Write-only.** No reply path, no waiting, no correlation to an answer. |
| Peer notify | `send_to_sibling` — `execution.rs:2509-2609`; args `{target_task_id, text}`; must be same run; same parent unless `allow_cross_parent` — `:2550-2559`; cross-branch is a Mutate-class approval — `is_cross_branch_message` `:2494-2507` | No control transfer, no reply. Bounded to the branch by default. |
| Message budget | `MAX_AGENT_MESSAGE_CHARS = 2_000`, `MAX_AGENT_MESSAGES_PER_RUN = 32` — `execution.rs:68-70`; enforced in both verbs — `:2448`, `:2569`; refusal `E_CALL_LIMIT` | A hard loop breaker. Note it is a *run* budget, not a per-branch budget. |
| Result plane | `record_task_result` / `task_result` — `execution.rs:3015`; aggregation reads child results from the task-scoped table, "never from the run-wide chat transcript, so one branch cannot consume another branch's answer" — `:1427-1465` | Branch isolation is explicit and tested. |
| Context plane | `ScopedContext` per task; `prompt_block(4_000)` — `execution.rs:53-58`; `seed_task_context` copies the persona allowlist — `:860`, `:1334`, `:1372` | A 4,000-character hard cap on injected facts. |
| Memory | `agent_memory` keyed by `agent_key` (role) + `key`, upsert; recall latest 8 — `store.rs:608-660`; clear by role — `:825`; written on completion with a 2,000-char bound and a secret-marker filter — `execution.rs:3017-3026`, `:61-65` | Role-scoped, not branch-scoped. Two runs of the same role share memory. |
| Registry | `agent_profiles` persistence (migration v9), `/api/agents/profiles` GET, `/import` POST, `/{id}` DELETE — `http.rs:519-522`, `:782-801`; boot merges persisted over embedded — `http.rs:143-148` | A validated record store with CRUD, no discovery API, and no dispatcher role. |
| Personas shipped | 7 records: `planner` (hyper), `superagent` (super, `max_children: 8`, `max_depth: 3`), 5 workers — `assets/prompts/personas.json:1-67` | One Super persona; workers differ only by template and allowlist. |
| Event bus | `EventBus` — server-global sequence, bounded broadcast fan-out (1,024), replay buffer (256) — `bus.rs:1-20` | **This is a UI/transport bus, not an agent bus.** No agent reads from it. |
| Terminal guard | `is_terminal_task_state` refuses any transition out of a terminal state — `execution.rs:93-95`, `:2984-2999`; run terminal frames are written once under a lifecycle lock — `:1272-1299`; every task is settled before the terminal frame — `:1264-1268`, `reconcile_terminal_tasks` `:1216` | Strong. This is the property most frameworks lack. |

Two structural facts deserve emphasis because they shape every recommendation:

1. **There is exactly one concurrency slot.** A single run, executed sequentially. Deadlock in the
   classical sense (two agents each holding a resource the other needs) is *not currently reachable*,
   because there is no concurrent sibling execution and no lock an agent holds across a model call. What
   *is* reachable is **livelock by blocking**: a child waiting for an answer that will never arrive blocks
   the only run until the call budget is exhausted. §8.1 treats that as the real deadlock risk.
2. **The observation plane and the coordination plane are already separate.** The event bus fans out to UI
   sessions only. That separation is the correct end state and should be defended, not blurred by adding an
   agent-facing topic bus.

## 5. Pattern → HiveMind mapping

Legend: **ADOPT** (build it), **ADAPT** (HiveMind already has the shape; change the semantics), **KEEP**
(already correct; defend it), **REJECT** (do not build).

| Pattern | Source | Verdict | Concrete HiveMind action |
|---|---|---|---|
| Task lifecycle with interrupted + terminal states | A2A [A2A-2] | **KEEP** | `AgentState` + `run_state` already exceed this; expose interrupted states (`AwaitingApproval`) in the contract vocabulary if not already |
| Terminal immutability | A2A [A2A-2] | **KEEP** | `is_terminal_task_state` guard; add the missing parent-completion guard (I6) |
| Task/context grouping | A2A [A2A-2] | **KEEP** | `run_id` groups; `task_id` identifies; add `in_reply_to` on messages for correlation |
| AgentCard as a data record | A2A [A2A-3] | **ADAPT** | `Persona` is the card. Add `description` (bounded), `tags[]`, `source{}`, `profile_version`; publish `/.well-known/agent-card.json` only if external interop is ever needed |
| Curated registry **service** | A2A [A2A-4] | **REJECT** | The spec prescribes no API; a local single-binary app needs a table, not a service |
| Well-known URI discovery | A2A [A2A-4] | **REJECT** (for now) | Nothing discovers HiveMind from outside; `personas.json` + `agent_profiles` is the discovery surface |
| Delegate-as-tool | OpenAI `as_tool` [OAI-1], LangChain subagents [LG-1], HiveMind `spawn_agent` | **KEEP** | Make it explicit in the tool description and in the event contract that this is a blocking call that returns a result |
| Handoff / control transfer | OpenAI `handoff()` [OAI-1] | **REJECT** | Requires a scheduler that can hand branch ownership to a peer; HiveMind has one scheduler and no peer runtimes. `send_to_sibling` is a notify, not a handoff |
| Full-history transfer on handoff | OpenAI [OAI-1] | **REJECT** | The opposite of HiveMind's branch isolation (`ScopedContext`, `task_result`); adopting it would recreate the leak the current design forbids |
| Input filters / history nesting | OpenAI [OAI-1] | **ADAPT** | The equivalent is the bounded child result (`bounded_tool_text`) plus a per-parent cap on how many child results enter one prompt |
| Nested supervisor as tool-calling agent | LangChain [LG-1] | **KEEP** | SuperAgent tier is this pattern; keep depth ≤ 3 |
| Supervisor as graph nodes | `langgraph-supervisor` | **REJECT** | Deprecated upstream [LG-1]; HiveMind's DAG is the graph, the tiers are the supervisor |
| Single durable checkpointer at the outermost level | LangChain [LG-1] | **KEEP** | One run-level snapshot store; do not add per-task checkpointers |
| Recursion/step limit with a named error | LangGraph [LG-2] | **KEEP** | `MAX_CALLS = 40` → `E_CALL_LIMIT` |
| Group chat / shared transcript | AutoGen [AG-1] | **REJECT** | Erases parent/child attribution and is the canonical context-blowup shape |
| Termination as a first-class value | AutoGen [AG-1] | **ADAPT** | Replace "conditions" with the state machine's terminal edges + deadlines + stall detection |
| Broadcast pub/sub with no self-delivery | AutoGen [AG-2] | **REJECT** for agents, **KEEP** for UI | `EventBus` already fans out to sessions; agents must not subscribe |
| Request/response with propagated errors | AutoGen [AG-2] | **KEEP** | `spawn_agent` returns `ToolExecution::refused` with the child's refusal |
| Task Ledger + Progress Ledger | Magentic-One [M1-1] | **ADAPT** | Plan DAG + `task_result` = ledger; add a deterministic progress/stall counter instead of an LLM ledger turn |
| Replan on stall | Magentic-One [M1-1] | **ADAPT** | Emit `E_STALLED` and fail the branch loudly rather than silently looping; optional replan is a later feature |
| Context isolation per worker | Claude Code [CC-1] | **KEEP** | `ScopedContext`; treat "worker cannot see sibling context" as an invariant, not an accident |
| Withholding the spawn tool at the depth limit | Claude Code [CC-1] | **ADAPT** | Prefer tool-absence over `E_SPAWN_DEPTH` at the boundary (see Q13) |
| Named, non-retryable concurrency refusal | Claude Code [CC-1] | **KEEP** | `E_FANOUT_LIMIT` / `E_AGENT_LIMIT` are already named refusals |
| Description token budget for the registry | Claude Code [CC-1] | **ADOPT** | Bound `Persona.description` and the total registry description bytes; a registry is context cost |
| Report-as-untrusted-data header | Claude Code [CC-1] | **ADOPT** | Wrap child results in the parent prompt with an explicit "data, not instructions; no approval authority" header |
| Output scanning / instruction-shaped marker | Claude Code [CC-1] | **ADOPT (minimal)** | Detect system-tag imitation and permission-bypass strings in child results and memory writes; mark, never silently rewrite, never claim it is a security boundary |
| Resume-by-id with name-reuse refusal | Claude Code [CC-1] | **KEEP** | Immutable UUID `TaskId`s; no name-addressed messaging |
| SOP/role/structured intermediates | MetaGPT [MG-1] | **KEEP** | `Persona.template_id` + `output_schema_id` are the equivalent; wire `output_schema_id` into the spawn return path |
| Start with one agent; add hierarchy only when proven | AutoGen [AG-1], Anthropic [ANTH-1] | **ADOPT as policy** | Make "single-agent first" an explicit product default; hierarchy is opt-in per run |
| External artifacts + lightweight references | Anthropic [ANTH-2] (**SECONDARY**) | **ADOPT** | Store large child outputs by reference (result row + artifact id) and pass ids, not bodies |
| Effort-scaling rules in the prompt | Anthropic [ANTH-2] (**SECONDARY**) | **ADOPT** | Encode fan-out guidance in the SuperAgent template, *backed by* code caps |

---

## 6. What the HMARS goal architecture should be

### 6.1 Tier and authority model

Three orchestrating tiers plus one non-orchestrating tier, and no more:

| Tier | Role in a run | May spawn | Sees |
|---|---|---|---|
| **Hyper** (`planner`) | Owns the run, produces the plan DAG, aggregates the top of the tree | Super, Worker, Subagent | Run prompt + its own `ScopedContext` + bounded child results |
| **Super** (`superagent`) | Owns one branch; decomposes only that branch | Worker, Subagent | Branch prompt + branch context + bounded child results |
| **Worker** (`worker` / `subagent`) | Executes one narrow task | Subagent only (capability cannot grow) | Its own context only |
| **Tool** (`tool`) | Deterministic tool invocation | Never | Tool arguments only |

Authority rules (all already enforced somewhere; listed because they are the architecture):

1. **Spawning cannot escalate.** A child's tier is bounded by the caller's tier
   (`validate_spawn_role`, `execution.rs:113-142`). Unknown roles degrade to Worker
   (`task_agent_tier`, `execution.rs:151-163`).
2. **Caps cannot be raised by data.** Profile `max_children`/`max_depth` must be ≤ the global caps
   (`prompts.rs:421-432`). A profile import that would exceed them must be refused at the boundary.
3. **A plan cannot invent depth.** `parent.depth + 1 == node.depth` and no parent loops
   (`planner.rs:76-100`).
4. **The scheduler is singular.** One root, one active run, one owner of lifecycle transitions
   (`execution.rs:1032`, `:1272-1299`).

There is no fourth tier, no "super-duper agent", and no peer tier. Every extra tier multiplies the number of
places where a terminal state can be missed and doubles the prompt-path length for the same result.

### 6.2 The three control verbs — and why there must be exactly three

| Verb | Tool | Blocking? | Returns | Authority conveyed | Default scope |
|---|---|---|---|---|---|
| **Delegate** | `spawn_agent` | **Yes** — the caller awaits the child inline (`execution.rs:2655-2691`) | Bounded child result + `state` | None; the child inherits the caller's ceiling, never exceeds it | One child at a time, capped by fan-out |
| **Escalate** | `ask_parent` | **Should block until answered or deadline; currently does not** | Currently `{message_id, to_task_id, kind}` | None; an answer is information, never an approval | Parent task only |
| **Notify** | `send_to_sibling` | **No** | `{message_id, to_task_id, kind}` | None | Same run, same parent by default; cross-branch requires approval |

**Delegate** is the load-bearing verb, and it should stay synchronous. The published alternative
(asynchronous fan-out with a coordinator that can steer children, [ANTH-2]) buys parallelism at the cost of
result coordination, ordering, cancellation and orphan handling. In a single-binary local app where the
whole run is already serialized (`active_run`), that trade is bad: it adds an entire failure class to save
wall-clock time that a sequential planner does not currently spend.

**Escalate** is the broken one. In A2A terms it produces a `Message` in a task that never moves to
`input-required` and never resumes. The minimal correct design is *not* a full request/response protocol;
it is three fields and one delivery rule:

```
agent_messages: (id, run_id, from_task_id, to_task_id, kind, text,
                 in_reply_to, status, created_at, answered_at)
status ∈ {open, answered, expired}
```

- The parent's next prompt includes its **open** messages (bounded, newest-first, capped — the same
  `prompt_block` discipline used for `ScopedContext`).
- The parent (or the operator) may answer; answering sets `status = answered` and records `in_reply_to`.
- An unanswered message expires at the parent's deadline or when the parent terminates; the child then fails
  with a named refusal rather than silently guessing.
- **An answer never grants tool authority.** It is text in the child's context, not an approval.

**Notify** should stay non-blocking and same-parent by default. Making it reply-capable would recreate a
general bus by accident. If a worker genuinely needs a peer's result, the correct move is to escalate to the
parent, which owns the branch — that is what the hierarchy is *for*.

**Rejected: a fourth verb, "handoff".** OpenAI's `handoff()` transfers control and history to a peer within
one run because the user is talking to whichever agent holds the conversation. HMARS has no user-facing
conversation ownership to transfer; the root owns the run and the scheduler owns the task table. Adding a
handoff verb would create two schedulers and no way to decide which one may write a terminal state.

### 6.3 The registry: a validated record store, not a dispatcher

`Persona` already has the right core: `id`, `display_name`, `tier`, `template_id`, `allowed_tool_ids`,
`output_schema_id`, `refusal_policy`, optional `max_children`, `max_depth` (`prompts.rs:381-404`). The
A2A card and the Claude Code subagent file agree on the missing metadata:

| Proposed field | Why | Bound |
|---|---|---|
| `description` | the routing surface the parent reads; without it, the model cannot choose a role | ≤ 200 characters per profile; ≤ 4,000 characters total (Claude Code warns at 15,000 tokens; HiveMind's prompt budget is far smaller) |
| `tags[]` | coarse filtering without a query language | ≤ 8 |
| `source {spdx, url, fetched_at, version}` | provenance; the repo already has a licence allowlist for imported content (`prompts.rs:7-9,23-30`) | required for imported rows |
| `profile_version` | lets a persisted profile be migrated or refused | integer |
| `output_schema_id` wired into spawn | the field exists but is not enforced on the child return path (`execution.rs:2684-2691` returns free text) | must validate when present |

Registry rules:

1. **The registry never dispatches.** Dispatch is `spawn_agent` → `spawn_sub_hive` → caps. A registry that
   routes would be a second scheduler.
2. **Every tool id must resolve** in the tool registry or the profile row is refused. That check does not
   exist yet: `Persona::validate` validates the template, the refusal policy and the caps only
   (`prompts.rs:411-434`), so `allowed_tool_ids` is currently unvalidated data.
3. **A profile may only tighten caps.**
4. **The registry is context, not capability.** Loading a profile must not grant a tool the engine would
   otherwise refuse; `authorize_tool_call` remains the single authority (`execution.rs:984`).
5. **No discovery API.** A2A explicitly does not standardise a registry API [A2A-4]; a local app has no
   second party to discover. `GET /api/agents/profiles` is a UI read, not a routing protocol.

### 6.4 Data plane: results and artifacts, never transcripts

The single most valuable existing property is that aggregation reads `task_result` rows keyed by task and
"never from the run-wide chat transcript, so one branch cannot consume another branch's answer"
(`execution.rs:1427-1430`). The HMARS data plane should be stated as:

```
Task --1:1--> Result(text, bounded)          # what the parent aggregates
Task --0:n--> Artifact(id, name, media_type, bytes/by-ref)   # what a human opens
Message(from, to, kind, status)              # coordination only, never an input to aggregation
```

Rules:

1. **Aggregation reads results, not messages.** A message is a coordination event; putting it into the
   aggregate prompt would let an unanswered question masquerade as a finding.
2. **Results are bounded at the boundary** (`bounded_tool_text`). Add a per-parent cap: at most
   `MAX_FANOUT` (8) child results enter one prompt, which is already the maximum number of children.
3. **Large outputs go by reference.** Anything above a threshold is stored as an artifact and the parent
   receives `{artifact_id, summary}`. This is the Anthropic "filesystem + lightweight reference" pattern
   [ANTH-2] expressed in data HiveMind already persists.
4. **Ordering matters for attention.** Put the aggregation instruction first and child results last; long
   contexts degrade for material in the middle [LITM-1].

### 6.5 Branch memory: a scope ladder with explicit promotion

Current memory is role-scoped (`agent_key` = role), latest 8, upsert by key, secret-filtered
(`store.rs:608-660`, `execution.rs:3017-3026`). That is a reasonable minimum and should be described as
`role` scope. HMARS needs four scopes with an explicit promotion path:

| Scope | Key | Written by | Read by | Promotion |
|---|---|---|---|---|
| **run** | `run_id` | engine only | that run's tasks | automatic, discarded at run end |
| **branch** | `run_id + parent_task_id` | a SuperAgent or Worker, for its branch | descendants of that parent only | to `role` requires the parent's completion |
| **role** | `role` (today's behaviour) | a completed task | any task with that role | to `global` requires operator action |
| **global** | `"global"` | **operator / import only** | everything | — |

Poisoning defences, in the order they should be built (cheapest first):

1. **Provenance on every row** (`run_id`, `task_id`, `source_task_id`, `created_at`) — already partly
   present; a row without provenance is not recallable.
2. **Never inject memory as instructions.** Render recalled memory inside a delimiter with a header that
   says it is prior work, not a directive. This is the data-not-authority rule applied to memory.
3. **Instruction-shaped detection on write.** If stored text imitates system tags or permission settings,
   store it flagged, and render the flag to the reader. Claude Code ships exactly this as a *marking*
   mechanism and is explicit that it is not a security boundary [CC-1]; copy the honesty as well as the
   mechanism.
4. **Secret filter** — exists (`memory_text_is_safe`, `execution.rs:61-65`) but is a four-marker substring
   check. It is a tripwire, not redaction. Do not upgrade it into a false claim of safety.
5. **Promotion requires a human or a parent**, never the writer. This is the structural defence against
   AgentPoison-style persistence [POIS-1]: a compromised worker can poison its own branch, not the role.
6. **Clear-by-scope** exists for role (`store.rs:825`); extend it to branch and run.

### 6.6 Termination and liveness

The state machine is already stronger than most frameworks: terminal states are absorbing
(`execution.rs:93-95`, `:2984-2999`), the terminal run frame is written exactly once under a lock
(`:1272-1299`), and every task is reconciled before the frame (`:1216-1244`). HMARS should add four things:

| Addition | Mechanism | Why it is needed |
|---|---|---|
| **Deadline** | per-run wall clock + inherited remaining budget per child | The only existing bound on a slow child is `MAX_CALLS`, which is a *count*, not time. A hung provider call can consume the whole budget without making progress |
| **Stall detection** | deterministic counters: no new `task_result` and no state change over N provider calls; repeated `(role, instruction-hash)` spawn signatures | Distinct from the total call cap: a branch can burn 40 calls looping over the same two steps (MAST FM-1.3 step repetition) |
| **Parent-completion guard** | refuse `Completed` for a task whose descendants are non-terminal | The run-level reconcile guarantees no non-terminal task survives the *run*; it does not prevent a parent from having been marked complete ahead of a live child |
| **Cancellation propagation** | `cancel_run` cascades to descendants; `kill_worker` settles one subtree | `cancel_run`/`kill_worker` exist (`execution.rs:1118`, `:1142`); the missing assertion is that a killed parent cannot leave a running descendant |

Preferred shape of the whole lifecycle:

```mermaid
stateDiagram-v2
    [*] --> Queued
    Queued --> Running: Start
    Running --> AwaitingApproval: Suspend
    AwaitingApproval --> Running: Resume
    AwaitingApproval --> Failed: Timeout / Deny
    Running --> Completed: Complete (no live descendants)
    Running --> Failed: Fail / deadline / stall
    Running --> Killed: Kill (operator)
    Queued --> Failed: Fail (before dispatch)
    Running --> Cancelled: Cancel run
    Completed --> [*]
    Failed --> [*]
    Killed --> [*]
    Cancelled --> [*]
```

`input-required` from A2A is deliberately **not** a task state here. In HMARS a question is a *message* with
a status, not a lifecycle state; making it a state would require the scheduler to park a task, and the
scheduler's parking mechanism (approval) is operator-facing. Keeping questions out of the state machine is a
simplification, and it is the reason the escalate verb must carry a status field instead.

---

## 7. Unnecessary complexity — what not to build

Each row names the pattern, why it is cut, and what would have to become true for it to be justified. This
list is the answer to "what is unnecessary complexity", and it is deliberately opinionated.

| # | Pattern | Why it is cut here | Would only be justified if… |
|---|---|---|---|
| X1 | **Agent pub/sub bus** (AutoGen topics, blackboard) | HiveMind has one scheduler and sequential execution; a bus adds ordering, delivery and loop-prevention problems (AutoGen needs explicit non-self-delivery to avoid infinite loops [AG-2]) to solve a coordination problem that does not exist | two or more independent runtimes must coordinate without a common scheduler |
| X2 | **Registry service with its own API** | A2A prescribes no registry API [A2A-4]; a table plus GET is sufficient and auditable | third parties publish profiles into this instance |
| X3 | **Fourth tier / arbitrary nesting** | Depth is the primary multiplier of cost and of missed terminal states; every source caps it (Claude Code 3 layers [CC-1], HiveMind 3 [HM-2]) | measurably better outcomes for tasks that genuinely need 4-level decomposition |
| X4 | **Group chat / shared transcript** | It destroys parent/child attribution and is the canonical context blowup [AG-1] | the task is a short debate with a handful of turns, i.e. not HMARS |
| X5 | **Vector memory as the default store** | Embedding recall is unverifiable at this scale, memory is model-visible input (hence poisonable [POIS-1]), and role+recency recall is already implemented | recall quality is a measured bottleneck, with an evaluation set |
| X6 | **Semantic / LLM router in front of dispatch** | Routing cost is a second model call per step; the plan DAG already decides roles | profiles are too numerous for the planner to enumerate |
| X7 | **A2A JSON-RPC internally** | Serialization + versioning + auth surface inside one process, for zero interop benefit [A2A-3] | HiveMind must accept tasks from another vendor's agent runtime |
| X8 | **Per-agent checkpointers** | LangChain's own guidance is one checkpointer at the outermost level [LG-1]; HiveMind has one run snapshot | independent per-branch replay is a product requirement |
| X9 | **Asynchronous fan-out with a steering coordinator** | The published upsides (steerability, parallelism) come with result coordination, ordering, cancellation and orphan handling [ANTH-2]; the run is serialized anyway | wall-clock latency is the measured product bottleneck |
| X10 | **Retry/backoff at every hop** | Compounding retries multiply cost and hide the failure; a named refusal propagated to the parent is more useful | a specific transient class is measured and classified |
| X11 | **Full profile editor / marketplace** | It is UI, not hierarchy; the record schema is already importable | profiles are user-authored content at volume |
| X12 | **LLM progress-ledger turn per step** | Magentic-One spends a model call to ask "are we progressing?" [M1-1]; counters give the same signal free | progress judgment needs semantic understanding (rare; then keep it as an operator tool, not per-step) |
| X13 | **Cross-run agent identity / long-lived agents** | Every examined system treats an agent instance as ephemeral per task; resumption is by id, not by a persistent mind [CC-1] | a product feature requires an agent that outlives a run |

The common thread: X1–X13 all add a **second source of truth** (a bus, a registry service, a router, a
checkpointer, a retry policy) next to the task table. HiveMind's strength is that the task table plus the
state machine is the only truth. Protecting that is worth more than any individual pattern.

---

## 8. Failure modes that must be designed out

For each mode: how it arises in HMARS specifically, the evidence that it is real, and the design-out rule.
§9 turns the rules into invariants.

### 8.1 Deadlock (and its real form here: blocked-on-a-dead-answer)

**Mechanism.** With one run and sequential execution, mutex deadlock is unreachable. The reachable form is a
child that asks a question (`ask_parent`) and then either (a) proceeds on a guess, because the tool returns
immediately, or (b) would wait, if a future change makes the verb blocking, for a parent that is itself
waiting on that child — a cycle in the wait-for graph. A third form is approval deadlock: a child suspends
for operator approval while the operator's only UI affordance is scoped to the run, so the run cannot finish
and the approval cannot be surfaced.

**Evidence.** A2A models this explicitly with an `input-required` interrupted state and a `taskId`+
`contextId` to resume [A2A-2]; OpenAI notes that handoffs stay within one run and that guardrails do not
follow them, i.e. a paused peer is a loss of control [OAI-1]; Anthropic reports that synchronous subagent
execution lets "the entire system be blocked while waiting for a single subagent to finish"
[ANTH-2] (**SECONDARY**).

**Design-out rules.**
- Every blocking wait has a deadline and a named refusal (`E_ESCALATION_TIMEOUT`, `E_APPROVAL_TIMEOUT`).
  Approval timeouts already exist; escalation timeouts do not.
- A child's deadline is *min(parent remaining, child budget)* — never unbounded.
- No cycle is possible if the wait-for graph follows parent edges only. **Enforce it:** an escalation may
  only target the immediate parent; a notify may only target same-parent siblings; a delegate may only
  target a child. That is a tree, and a tree has no cycles.
- An operator-facing approval must be reachable while a child is suspended: the run's approval queue must
  be enumerable and resolvable without the child returning.

### 8.2 Runaway fan-out

**Mechanism.** A model that decides more agents = better. Multiplies across tiers: 8 children × 8 children ×
8 children = 512 tasks, each with a prompt.

**Evidence.** Anthropic's early system "spawned 50 subagents for simple queries" and ran "scouring the web
endlessly"; the fix was explicit effort-scaling rules in the prompt [ANTH-2] (**SECONDARY**). Claude Code
enforces a hard concurrent cap (20) with a non-retryable error, and shares the slot pool with other features
[CC-1]. HiveMind already caps depth 3, fan-out 8, agents 64 (`caps.rs:9-15`).

**Design-out rules.**
- Keep all four caps, enforced *before* a row is created (already true).
- Add a **per-run budget projection**: if `agents + planned_children > MAX_AGENTS`, refuse at plan
  validation, not at the Nth spawn. A run that dies at task 64 after 40 model calls is a wasted run.
- Make the refusal message tell the parent what to do instead ("return your current best result"), because a
  bare refusal invites retry loops.
- At the depth boundary, prefer **withholding the tool** to refusing the call (Claude Code's choice [CC-1]).
  A model that cannot see `spawn_agent` writes a summary; a model that sees it and gets an error may retry.

### 8.3 Context blowup

**Mechanism.** A parent aggregates 8 children, each of which aggregated 8 children; the children's results
are inlined; the prompt grows to tens of thousands of tokens; the model's attention to any single child
degrades; cost grows quadratically because every subsequent turn re-sends the accumulation.

**Evidence.** Long-context performance degrades for material in the middle of the input [LITM-1]; group chat
broadcasts every turn to every participant [AG-1]; Anthropic caps by spawning *fresh* subagents with clean
contexts and persisting the plan to memory, and passes references rather than bodies [ANTH-2]
(**SECONDARY**). HiveMind already bounds child results (`bounded_tool_text`) and the scoped-context block
(4,000 chars, `execution.rs:53-58`).

**Design-out rules.**
- Bound at every boundary: message text (2,000 chars, exists), result text (exists), scoped context block
  (4,000 chars, exists), number of child results per aggregation (≤ `MAX_FANOUT`).
- Artifact-by-reference above a threshold; the parent gets `{artifact_id, summary}`.
- Put the task instruction first and child results last (the U-shaped attention result [LITM-1]).
- Count and publish tokens per branch (`run_usage` exists) so a blowup is visible before it is fatal.
- No sibling/transcript injection, ever: branch isolation is also a context-budget mechanism.

### 8.4 Duplicated work

**Mechanism.** Two branches do the same search; the planner emits near-duplicate nodes; a SuperAgent spawns
a worker for work a sibling is already doing; a re-run redoes completed work because the parent cannot see
what exists.

**Evidence.** Anthropic's concrete example: "one subagent explored the 2021 automotive chip crisis while 2
others duplicated work investigating current 2025 supply chains, without an effective division of labor";
the fix was an objective, an output format, tool/source guidance and clear task boundaries per subagent
[ANTH-2] (**SECONDARY**). MAST names information withholding (FM-2.4) and ignored input (FM-2.5) as
distinct modes [MAST-1].

**Design-out rules.**
- The planner must emit a **distinct scope statement per node** (HiveMind's `instruction` field exists on
  `spawn_agent`; planned nodes currently carry only `role`, `deps`, `depth`, `parent` — no instruction
  text, `planner.rs:14-27`). Adding a bounded `instruction`/`scope` to `PlanNode` is the single highest-value
  cheap change in this whole document.
- Dedupe at spawn time: refuse a spawn whose `(role, normalised instruction hash)` already exists under the
  same parent, with a named refusal.
- The parent's prompt states what its children have already returned (it already receives their results);
  stating it *before* the spawn decision is what prevents duplicates.
- Cross-branch discovery is explicitly out of scope: a worker cannot see siblings, so the parent must be the
  de-duplicator. That is a hierarchy cost, accepted in exchange for isolation.

### 8.5 Memory poisoning

**Mechanism.** A worker's terminal text is written to `agent_memory` (`execution.rs:3017-3026`) and later
recalled into another task's context (latest 8, role-scoped). A worker that read a malicious file can
therefore persist text that a later, more privileged run reads as prior work.

**Evidence.** AgentPoison demonstrates >80% attack success at <0.1% poison rate against memory/RAG agents
[POIS-1]. MAST's FM-2.4 (information withholding) and FM-1.2 (disobey role specification) describe the
softer, non-adversarial version [MAST-1]. Claude Code treats subagent output as untrusted prose whose
instructions carry no authority [CC-1].

**Design-out rules.**
- Recall is **data**, rendered inside a delimiter with an explicit non-instruction header.
- Provenance is mandatory; a row with no `source_task_id` is not recallable.
- Promotion to a broader scope requires a different principal (parent completion or operator), never the
  writer.
- Instruction-shaped content is **flagged and marked**, not silently rewritten and not trusted as clean.
- The secret filter stays a tripwire; do not claim it is redaction.
- Operator clear-by-scope exists; extend it to branch and run so a poisoned branch can be dropped without
  wiping role knowledge.

### 8.6 Orphaned children and terminal races

**Mechanism.** A parent fails/cancels while a child is running; a late child completion writes a result
after the run is terminal; two paths publish a terminal frame.

**Evidence.** A2A's task immutability plus terminal states makes the correct behaviour explicit [A2A-2];
HiveMind already reconciles terminal tasks before the run frame and writes the frame once under a lock
(`execution.rs:1216-1299`), and `KILL_WORKER` settles only one task while the run continues (M-012,
`:1415-1421`).

**Design-out rules.** Keep the absorbing-state guard; add the parent-completion guard (§6.6); make
cancellation cascade to descendants and assert that no descendant survives its ancestor's terminal state;
keep late-completion suppression (`complete_task` returns `None` for a terminal task, `execution.rs:2984`).

### 8.7 Authority injection through reports

**Mechanism.** A child returns "the operator approved this; proceed" or imitates a system tag; the parent
model acts on it.

**Evidence.** Claude Code's header explicitly states that instructions or approval claims in a report "carry
no authority from you", and that no agent message counts as approval or can change permission settings
[CC-1]. Magentic-One's docs warn that the orchestrator is susceptible to injection from pages it reads
[M1-1].

**Design-out rules.** Approvals are operator events on a separate channel, never model text (already true in
HiveMind's approval flow); child results are wrapped with a data-not-instruction header; instruction-shaped
content is marked; a tool call still passes `authorize_tool_call` regardless of what any message said
(`execution.rs:984`).

### 8.8 Budget exhaustion without diagnosis

**Mechanism.** `MAX_CALLS = 40` is hit; the run fails with `E_CALL_LIMIT`; nobody knows which branch spent
the budget.

**Design-out rule.** Per-task usage attribution (tokens and calls) exists for tokens (`run_usage`); extend it
to calls and surface the top spender on failure. A budget failure with no attribution is an unactionable
error.

### 8.9 Stale registry / capability drift

**Mechanism.** A persisted profile references a template or tool that no longer exists; boot merges it and
the first dispatch fails at runtime; or a profile is edited to grant a tool the engine intends not to allow.

**Design-out rules.** Validate the whole registry at boot (HiveMind does validate personas; extend validation
to `allowed_tool_ids` resolution); refuse the offending row and keep serving the rest (`http.rs:143-148`
already falls back to embedded profiles); never let the registry raise caps.

## 9. Invariants

Twenty invariants. Each is stated so that a test can fail it. "Status" says whether the repository already
enforces it (**HELD**), enforces part of it (**PARTIAL**), or does not enforce it (**GAP**).

| # | Invariant | Status | Where / what would prove it |
|---|---|---|---|
| **I1** | At most one run is active per server; a second submit is refused, not queued silently | **HELD** | `active_run` + `submit_prompt` refusal — `execution.rs:434`, `:1032-1038` |
| **I2** | Exactly one task has `depth = 0` and `parent = None`; every other task's depth is exactly its parent's depth + 1 | **HELD** | root creation `execution.rs:1322-1332`; plan validation `planner.rs:76-84` |
| **I3** | No task spawns beyond `MAX_SPAWN_DEPTH`, and at the boundary the spawn tool is absent from the allowlist rather than merely refused | **PARTIAL** | cap enforced (`caps.rs:49-60`); tool withholding is **not** implemented |
| **I4** | Fan-out ≤ 8 per parent, agents ≤ 64 per run, provider calls ≤ 40 per run; every refusal is a named contract code, never a clamp | **HELD** | `caps.rs:9-15`, `:49-71`; `persona_spawn_limits` `execution.rs:550` |
| **I5** | A terminal task never transitions again; a late completion is suppressed without publishing a done frame | **HELD** | `execution.rs:93-95`, `:2984-2999` |
| **I6** | A run reaches a terminal state only when every task is terminal, and a task reaches `Completed` only when all of its descendants are terminal | **PARTIAL** | run-level reconcile holds (`execution.rs:1216-1244`); the parent-completion guard is absent |
| **I7** | Exactly one terminal run frame is written and published, under a lifecycle lock, and never resurrected | **HELD** | `execution.rs:1272-1299` |
| **I8** | Branch isolation: aggregation reads only task-scoped results; no task's prompt contains another branch's transcript or scoped context | **HELD** | `execution.rs:1427-1465`, `:53-58`, `:860` |
| **I9** | Every inter-agent message is bounded in size (≤ 2,000 chars) and the run has a hard message budget (32) | **HELD** | `execution.rs:68-70`, `:2441`, `:2562` |
| **I10** | A message conveys no authority: it can never satisfy an approval, change a tool allowlist, or alter a cap | **PARTIAL** | approvals are operator-driven (design); the "data, not instructions" header on child results is absent |
| **I11** | A peer message targets the same run and the same parent unless an explicit approval grants cross-branch delivery | **HELD** | `execution.rs:2550-2559`; cross-branch is Mutate-class `:2494-2507` |
| **I12** | A `spawn_agent` call's claimed `parent_task_id` must equal the caller's id; the run row is authoritative | **HELD** | `execution.rs:2635-2647` |
| **I13** | A spawn cannot raise capability tier; an unrecognised role degrades to Worker and is never promoted | **HELD** | `validate_spawn_role` `execution.rs:113-142`; `task_agent_tier` `:151-163` |
| **I14** | The plan is a valid DAG: ≤ 40 nodes, unique ids, resolvable deps, no cycles, parent at depth−1, no parent loops | **HELD** | `planner.rs:44-100` |
| **I15** | Each aggregator runs exactly once, after all of its children, deepest first; leaves run before aggregators | **HELD** | `plan.leaf_order()` / `aggregation_order()` `planner.rs:147-180`; `execution.rs:1396`, `:1431` |
| **I16** | Every blocking wait has a deadline and a named refusal; no operation can wait indefinitely | **GAP** | approval timeout exists (`execution.rs:2261-2390`); escalation has no wait at all; no run deadline |
| **I17** | Budget exhaustion is a named refusal with attribution (which task consumed it), never a silent drop or truncation | **PARTIAL** | `E_CALL_LIMIT` / `E_AGENT_LIMIT` named; per-task call attribution absent |
| **I18** | Memory rows carry provenance, are bounded and secret-tripped, are recalled only within their scope, and are promoted to a wider scope only by a principal other than the writer | **PARTIAL** | role scope + bound + filter held (`store.rs:608-660`, `execution.rs:3017-3026`); branch/run scopes, provenance and promotion rules absent |
| **I19** | Every registry row validates at boot; an invalid row is refused while the rest serve; a profile may tighten but never raise a cap | **PARTIAL** | `Persona::validate` + embedded fallback `http.rs:143-148`; `allowed_tool_ids` resolution and `description` bound absent |
| **I20** | Context injection is bounded at every entry point: child result text, scoped context block, recalled memory, and message text each have an explicit cap | **PARTIAL** | child/scoped/message caps held; recalled-memory and open-escalation caps absent |

Reading the table: HiveMind **holds** twelve of twenty invariants outright, **partially** holds seven, and
**gaps** one. That distribution is the real finding of this research — the architecture is not the problem;
the missing pieces are all at the *edges* of the primitives (answers, deadlines, provenance, attribution),
not in the middle of them.

---

## 10. Twenty questions on HMARS design

**Q1. Should the root be an LLM planner or a deterministic engine?**
Both, in that order. The LLM produces the plan; the engine validates it and owns every transition. HiveMind
already does this: one planning call, then `plan.validate()` before a single row exists
(`execution.rs:1313-1319`, `planner.rs:44-100`). The engine must never ask the model "is the plan valid?".
AutoGen's first-class termination objects show the same division — the runtime decides when to stop, not the
prompt [AG-1]; HiveMind's version is stricter because the plan is rejected before any task row exists
(**READ**).

**Q2. How many orchestrator tiers?**
Three including the root: Hyper → Super → Worker, with Tool as a non-orchestrating actor. Every source that
has an opinion caps nesting (Claude Code: 3 layers default, [CC-1]; HiveMind: `MAX_SPAWN_DEPTH = 3`,
[HM-2]). Each extra tier multiplies the number of terminal states that can be missed and lengthens every
result path. A fourth tier is X3 in §7 and should be rejected in review.

**Q3. Should workers be allowed to spawn workers at all?**
Yes, but only the narrowest kind, and only because HiveMind already forbids escalation: a Worker may spawn a
*Subagent*, never a Super (`execution.rs:130-134`). This matches LangChain's "nest tool calls when you need
intermediate coordination" [LG-1] and Claude Code's default nesting [CC-1]. The alternative — workers cannot
spawn — is simpler and defensible, but it removes the only way to split a leaf task that turns out to be
two tasks. Keep the current rule; do not widen it.

**Q4. Is `ask_parent` a request/response RPC or a mailbox write? Which should it be?**
Today it is a mailbox write with no reader: it persists a row and returns `{message_id, to_task_id, kind}`
(`execution.rs:2465-2479`), and no prompt ever includes those rows. It should become a **mailbox write with
a visible inbox and a status**. Full RPC (blocking the child until the parent runs) is wrong here because
the parent is the scheduler — it cannot answer mid-child without re-entering itself. The correct minimal
semantics: the parent sees its open messages on its next turn; answering marks them answered; unanswered
messages expire. That is A2A's `input-required` intent [A2A-2] without adding a task state.

**Q5. Handoff, delegate, or notify — which should exist?**
Delegate and notify, plus a real escalate. Handoff (control transfer) is rejected for the reasons in §6.2
and is close to X1 in §7: it requires two schedulers. Delegate exists and is correct: synchronous, returns a
bounded result, child bounded by caps (`execution.rs:2655-2691`). Notify exists and is correct as a
*bounded* side channel (`execution.rs:2550-2559`). Escalate exists structurally but not semantically (Q4).
OpenAI's own docs draw the delegate/handoff line with `as_tool()` vs `handoff()` [OAI-1]; HMARS only needs
the left side.

**Q6. Do siblings need direct messaging?**
Only as a bounded notify, and only within a parent. Cross-branch messaging is the seam where branch
isolation dies: today it requires an operator approval (`is_cross_branch_message` +
`allow_cross_parent`, `execution.rs:2494-2559`), which is the right answer. If a worker needs a peer's
*result*, it should escalate to the parent — the parent owns the branch and already receives every child
result. Adding reply-capable sibling channels would quietly recreate a bus (X1).

**Q7. Should there be an agent pub/sub bus?**
No. A2A deliberately keeps the remote agent opaque and gives peers no shared context [A2A-3]; AutoGen's bus
needs two non-obvious safety rules just to avoid loops (no self-delivery) and to avoid silent failure
(exceptions logged, not propagated) [AG-2]. HiveMind's `EventBus` is a UI fan-out with sequence numbers and
replay (`bus.rs:1-20`); using it for agent coordination would couple reconnect semantics to reasoning and
create a second source of truth next to the task table (X1).

**Q8. What goes in a registry, and what must stay out?**
In: identity, description, tier, template, tool allowlist, output schema, refusal policy, optional caps,
provenance. Out: dispatch logic, routing rules, live state, credentials, and any field that can widen a cap.
A2A's `AgentCard` is the best public checklist for the "in" list [A2A-3]; Claude Code's subagent frontmatter
is the best evidence that **description is context, not metadata** — combined descriptions trigger a 15,000
token warning [CC-1]. HiveMind's `Persona` is already a correct card; it lacks `description`, provenance and
an enforced `output_schema_id` (§6.3).

**Q9. Should profiles own caps?**
They may tighten, never raise. `Persona.max_children`/`max_depth` are validated to be ≤ the global caps
(`prompts.rs:421-432`, `execution.rs:550`). This preserves "caps are code" while letting a cautious profile
be more conservative. A profile that could raise `max_agents` would make the registry a second source of
truth for the thing that most needs one source.

**Q10. What is the unit of work — the task or the agent instance?**
The task. Every examined system keys persistence, status and results by task, not by agent ([A2A-2] task
lifecycle; [CC-1] resume-by-id; HiveMind `task_result`, `agent_messages.to_task_id`). Agent identity
(persona) is a *template*; the task is the *instance*. Conflating them is what makes "resume the agent"
ambiguous — Claude Code resolves it with an immutable instance id and refuses name reuse [CC-1].

**Q11. How is branch memory scoped so it cannot leak?**
By key, not by convention. Today memory is keyed by role (`agent_key`), so two different branches running
the same role share memory (`store.rs:608-660`). That is a deliberate trade — role memory is useful — but it
means branch knowledge is not isolated. The scope ladder in §6.5 (run → branch → role → global) with
promotion only by another principal is the fix. Until it exists, the honest statement is: "memory is
role-scoped, therefore cross-branch contamination through memory is possible by design."

**Q12. How do we prevent context blowup when children return long results?**
Four bounds, three of which exist: message text (2,000 chars), child result (`bounded_tool_text`), scoped
context block (4,000 chars), and a per-parent cap on how many child results enter one aggregation prompt
(≤ 8, equal to `MAX_FANOUT`). Add artifact-by-reference above a threshold and put the instruction first,
child results last, because long-context attention degrades for middle content [LITM-1]. The Anthropic
report's "filesystem + lightweight reference" pattern is the production version of the same idea
[ANTH-2] (**SECONDARY**).

**Q13. Should exceeding depth be a refusal or a missing tool?**
A missing tool at the boundary, a refusal everywhere else. Claude Code "withholds the `Agent` tool from every
subagent" at the depth limit so the agent "does its delegated work itself and returns one summary" [CC-1].
A refusal invites a retry; an absent tool produces a different plan. HiveMind currently refuses with
`E_SPAWN_DEPTH` (`caps.rs:50-52`) — keep the refusal as the last line of defence and add allowlist
withholding at depth == `max_depth`. This is invariant I3.

**Q14. What terminates a run — model judgement or engine counters?**
Engine counters, with the model's completion as a *request*. `MAX_CALLS`/`MAX_AGENTS` are already the
backstop; what is missing is time and no-progress. AutoGen makes termination a first-class object
[AG-1]; LangGraph raises a named error at its step limit [LG-2]; Magentic-One replans when progress stalls
[M1-1]. The HMARS composition is: model completes → engine validates the parent/child termination guard →
engine enforces deadline and stall counters → engine writes one terminal frame. No prompt sentence is a
stopping condition.

**Q15. How do we detect stall/no-progress?**
Deterministically: no new `task_result`, no state change, and no new artifact over the last N provider calls,
plus repeated `(role, normalised instruction)` spawn signatures under one parent. That covers MAST FM-1.3
(step repetition) and FM-1.5 (unaware of stopping conditions) [MAST-1] without spending a model call per
step, which is what Magentic-One's Progress Ledger does [M1-1] and what X12 rejects as a default. Emit a
named `E_STALLED` and attribute it to the branch, so the operator can see *where* it spun.

**Q16. How do we avoid deadlock in a blocking hierarchy?**
By making the wait-for graph a tree. Delegate waits only downward; escalate waits only upward; notify never
waits. A tree has no cycles, so mutual wait is impossible. Then bound every wait with a deadline (I16) and
make the deadline inherit the parent's remaining budget. The residual risk is not deadlock but *starvation*:
one slow child consuming the run's only execution slot — which is why the deadline is mandatory and why
asynchronous fan-out (X9) is rejected rather than merely deferred.

**Q17. What happens when a child needs operator approval mid-hierarchy?**
The child suspends (`AwaitingApproval`), the run remains active, and the approval must be resolvable from
the run's approval queue without the child returning. HiveMind has suspend/resume/deny/timeout for tasks
(`execution.rs:2000-2390`); the open question is whether an operator can *see* a nested child's pending
approval while the parent is blocked in the inline `spawn_agent` await. That is a UI/queue question with a
correctness consequence: if the approval is not enumerable, the run deadlocks until timeout. Treat
"pending approvals are enumerable at all times" as part of I16.

**Q18. How do we survive a crash mid-hierarchy?**
By persisting the plan before execution, committing a snapshot after every executed node, and reconciling
task states on restore. HiveMind does all three (`execution.rs:1393`, `:1402-1408`,
`recovery.rs`). LangChain's rule — one checkpointer at the outermost level, subagents get per-invocation
persistence [LG-1] — is the same architecture. What must be added is **idempotency at the tool boundary**:
a replayed node must not double-write a result or double-send a message. Results are keyed by task
(`record_task_result`), which makes replay naturally idempotent; messages are not, so a resumed run can
re-send.

**Q19. How do we stop memory poisoning from worker output?**
Four structural rules, not a classifier: provenance is mandatory; recall is rendered as data with a
non-instruction header; promotion to a wider scope requires a different principal; instruction-shaped text
is flagged for the reader. AgentPoison shows that a sub-0.1% poison rate can win >80% of the time
[POIS-1], so "most memories are fine" is not a defence. Claude Code's report header is the closest shipped
precedent for the data-not-authority framing [CC-1].

**Q20. Is A2A interoperability a goal or a distraction?**
A distraction *inside* the binary, and a cheap option *at the edge*. The spec's own value is the task
lifecycle and the card, both of which HiveMind already approximates; its transport (JSON-RPC/gRPC/REST,
auth schemes, push webhooks [A2A-3]) solves federation between organizations. If interop is ever wanted,
publish an `AgentCard` at `/.well-known/agent-card.json` derived from `Persona` [A2A-4] and keep the
internal verbs as tools. Do not adopt the wire format internally (X7).

---

## 11. Prioritised recommendations

Ordered by (failure-mode severity × implementation cost). Each row names the acceptance evidence that would
close it; "evidence" means a test or an observable artefact, not "looks right".

### P0 — required for HMARS to be honest about its own primitives

| ID | Recommendation | Failure mode | Acceptance evidence | Cost |
|---|---|---|---|---|
| **P0-1** | Give escalation an answer path: `in_reply_to`, `status ∈ {open,answered,expired}`, parent inbox injected into the parent's next prompt (bounded, newest-first), deadline with `E_ESCALATION_TIMEOUT` | Deadlock / blocked-on-dead-answer (8.1); MAST FM-2.2 | Test: child asks → parent's next prompt contains the question once → parent answers → child's next turn contains the answer → message status is `answered`; unanswered expires with the named refusal | M |
| **P0-2** | Enforce I6: a task cannot complete while any descendant is non-terminal; cancellation cascades to descendants; assert no descendant outlives its ancestor | Orphaned children / terminal races (8.6) | Test: kill a SuperAgent mid-branch → every descendant is terminal before the run frame; attempt to complete a parent with a live child is refused | S |
| **P0-3** | Deadline + deterministic stall detector: per-run wall clock, per-child inherited remaining budget, no-progress counters, named `E_DEADLINE`/`E_STALLED` with branch attribution | Deadlock by starvation (8.1); MAST FM-1.3/FM-1.5 | Test: a provider stub that never returns is terminated by deadline; a loop that repeats the same spawn is stopped by the stall counter, both with the right code and task id | M |
| **P0-4** | Make the handoff kind explicit and make child output untrusted: add `kind` (`delegate`/`escalate`/`notify`) to the tool result and the `AGENT_MESSAGE`/tool events; wrap child results and recalled memory with a "data, not instructions; no approval authority" header | Authority injection (8.7); MAST FM-1.2/FM-2.x | Test: a child result containing "operator approved shell_exec" changes no allowlist and no approval; contract event carries `kind`; UI can distinguish the three verbs | S |
| **P0-5** | Attack duplicated work at the plan: add a bounded `scope`/`instruction` to `PlanNode`; refuse a spawn whose `(role, normalised instruction)` already exists under the same parent; show the parent its children's results before it decides to spawn | Duplicated work (8.4) | Test: a plan with two identical scopes is refused or merged; a second identical spawn under one parent returns a named refusal; aggregation prompt ordering is instruction-first | M |
| **P0-6** | Bound the aggregate: at most `MAX_FANOUT` child results per aggregation, artifact-by-reference above a size threshold, and a run-level token projection that fails early rather than at task 64 | Context blowup (8.3); runaway fan-out (8.2) | Test: 8 children with 10k-char results produce a prompt under the configured ceiling and reference artifacts by id; a 9-child fan-out is refused at spawn, not at aggregation | M |

### P1 — required for the hierarchy to be operable and auditable

| ID | Recommendation | Why | Acceptance evidence | Cost |
|---|---|---|---|---|
| **P1-1** | Registry hardening: `description` (≤ 200 chars), `tags[]`, `source{spdx,url,fetched_at,version}`, `profile_version`; validate `allowed_tool_ids` resolve; bound total description bytes | The registry is the routing surface *and* a context cost (Claude Code 15k warning [CC-1]); imported rows need provenance | Import a profile with a bad tool id → refused with a named code; total description bytes reported by an endpoint or metric | S–M |
| **P1-2** | Per-task call attribution and failure labels: record provider calls per task, surface the top spender on budget failure; adopt MAST-style labels for run outcomes | I17; MAST shows diagnosis is the bottleneck [MAST-1] | A `E_CALL_LIMIT` failure names the task and call count; run records carry a failure class | S |
| **P1-3** | Memory scope ladder + clear-by-scope: add `run`/`branch` scopes and `source_task_id`; promotion only by another principal; extend clear-by-scope | 8.5; role-only scope means cross-branch contamination is possible by design | Test: branch A's memory is invisible to branch B; promoting to role requires parent completion; clear(branch) removes only that branch | M |
| **P1-4** | Make pending approvals enumerable while a parent is blocked in `spawn_agent` | 8.1 approval deadlock; I16 | Test: child suspends → approval is listed and resolvable → child resumes → parent receives the result | S–M |
| **P1-5** | Wire `output_schema_id` into the spawn/delegate return path | The field exists but free text is returned (`execution.rs:2684-2691`); MetaGPT's structured-artifact claim [MG-1] depends on it | A child with a schema that returns non-conforming text is refused with a named code | S |

### P2 — valuable, not blocking

| ID | Recommendation | Why | Acceptance evidence | Cost |
|---|---|---|---|---|
| **P2-1** | A2A card **export only**: derive `AgentCard` JSON from `Persona` and serve it read-only; no internal A2A transport | Interop option at the edge, no second scheduler (Q20) | `GET /.well-known/agent-card.json` returns a schema-valid card derived from the registry; no internal code path consumes it | S |
| **P2-2** | Evaluation harness: a 20-case smoke set with end-state evaluation and MAST labels; run it on every engine change | Anthropic's "start small, end-state not trace" [ANTH-2] (**SECONDARY**); MAST's diagnosis framing [MAST-1] | A command runs the 20 cases and reports pass/fail plus failure classes; a regression is caught before merge | M |
| **P2-3** | Optional replan-on-stall: after `E_STALLED`, allow the parent one bounded replan of its own branch | Magentic-One's stall→replan [M1-1] | A stalled branch produces exactly one replan, bounded by the same caps, and never loops | M |
| **P2-4** | Profile UI (read + import + delete) beyond the current panel | Operability, not architecture | Profiles can be inspected without curl | S |

### P3 — record as explicit non-goals

| ID | Non-goal | Reason |
|---|---|---|
| **P3-1** | Agent pub/sub bus, registry service API, semantic router, vector memory, fourth tier, internal A2A | X1–X7, X13 in §7 |
| **P3-2** | Asynchronous fan-out with a steering coordinator | X9; revisit only if wall-clock latency becomes the measured bottleneck |
| **P3-3** | Per-agent checkpointers | X8; one run-level snapshot is correct |

## 12. Sources

All URLs fetched 2026-11-07. "Class" is the evidence class from §0. "Fetched" records what the fetch actually
returned, because a 200 with a navigation-only body is not a read page.

### 12.1 External sources

| ID | Source | Class | URL | Used for | Fetched |
|---|---|---|---|---|---|
| A2A-1 | A2A Protocol — Specification (overview) | SPEC | https://a2a-protocol.org/latest/specification/ | Structure/normative anchors only; no claim rests on this page | HTTP 200, body truncated to TOC |
| A2A-2 | A2A — Life of a Task | SPEC | https://a2a-protocol.org/latest/topics/life-of-a-task/ | Message-vs-Task, terminal vs interrupted states, task immutability, `contextId`/`taskId`/`referenceTaskIds` | HTTP 200, read |
| A2A-3 | A2A — Core Concepts | SPEC | https://a2a-protocol.org/latest/topics/key-concepts/ | AgentCard/Message/Part/Artifact, interaction mechanisms, remote-agent opacity | HTTP 200, read |
| A2A-4 | A2A — Agent Discovery | SPEC | https://a2a-protocol.org/latest/topics/agent-discovery/ | Well-known URI, curated registries, "spec does not prescribe a standard API for curated registries", caching | HTTP 200, read |
| OAI-1 | OpenAI Agents SDK — Handoffs | OFFICIAL-DOCS | https://openai.github.io/openai-agents-python/handoffs/ | Handoff-as-tool, history transfer, `input_filter`, `input_type`, `is_enabled` authorization pitfall, guardrail scope, `as_tool` contrast | HTTP 200, read |
| LG-1 | LangChain — Migrate from langgraph-supervisor | OFFICIAL-DOCS | https://docs.langchain.com/oss/python/migrate/langgraph-supervisor.md | Supervisor package unmaintained; subagents-as-tools; nested supervisors; interrupt propagation; `output_mode` | HTTP 200, read |
| LG-2 | LangGraph — GRAPH_RECURSION_LIMIT | OFFICIAL-DOCS | https://docs.langchain.com/oss/python/langgraph/errors/GRAPH_RECURSION_LIMIT.md | Configurable recursion limit, named error, cycle diagnosis | HTTP 200, read |
| AG-1 | AutoGen — Teams tutorial | OFFICIAL-DOCS | https://microsoft.github.io/autogen/stable/user-guide/agentchat-user-guide/tutorial/teams.html | Team presets, shared context, termination conditions, reset/resume/abort | HTTP 200, read |
| AG-2 | AutoGen Core — Message and Communication | OFFICIAL-DOCS | https://microsoft.github.io/autogen/stable/user-guide/core-user-guide/framework/message-and-communication.html | Direct request/response with error propagation; broadcast one-way, discarded responses, no self-delivery | HTTP 200, read |
| M1-1 | AutoGen — Magentic-One | OFFICIAL-DOCS | https://microsoft.github.io/autogen/stable/user-guide/agentchat-user-guide/magentic-one.html | Task Ledger / Progress Ledger, stall→replan, injection warnings | HTTP 200, read |
| CC-1 | Claude Code — Create custom subagents | OFFICIAL-DOCS | https://code.claude.com/docs/en/sub-agents.md | Context isolation, nesting depth default 3 + tool withholding, concurrency 20, description token budget, report authority header, output scanning, resume/name reuse | HTTP 200, read (truncated after the nesting/concurrency sections; sections read are complete) |
| MG-1 | MetaGPT (arXiv:2308.00352) | PAPER | https://arxiv.org/abs/2308.00352 | SOPs in prompt sequences, assembly-line roles, cascading-hallucination motivation | HTTP 200, abstract read |
| MAST-1 | Why Do Multi-Agent LLM Systems Fail? (arXiv:2503.13657) | PAPER | https://arxiv.org/abs/2503.13657 · HTML https://arxiv.org/html/2503.13657v3 | 14 failure modes / 3 categories, 1,642 traces, κ=0.88, 41%–86.7% failure rates, system-design primacy, mode names | HTTP 200, abstract + HTML TOC/appendix headings read |
| LITM-1 | Lost in the Middle (arXiv:2307.03172) | PAPER | https://arxiv.org/abs/2307.03172 | U-shaped long-context performance; middle-position degradation | HTTP 200, abstract read |
| POIS-1 | AgentPoison (arXiv:2407.12784) | PAPER | https://arxiv.org/abs/2407.12784 | Memory/RAG poisoning, >80% ASR at <0.1% poison rate, <1% benign impact | HTTP 200, abstract read |
| ANTH-1 | Anthropic — Building effective agents | SECONDARY-BLOG | https://www.anthropic.com/engineering/building-effective-agents (redirects to /research/) | Orchestrator-workers definition; "add complexity only when it demonstrably improves outcomes" | HTTP 200, read |
| ANTH-2 | Anthropic — How we built our multi-agent research system | SECONDARY-BLOG | https://www.anthropic.com/engineering/multi-agent-research-system (redirects to /research/) | 90.2% eval gain, ~15× token cost, 50-subagent failure, delegation specificity, synchronous bottleneck, artifacts-by-reference, resumability | HTTP 200, read |

### 12.2 Internal sources (READ)

| ID | File | Used for |
|---|---|---|
| HM-1 | [`backend/crates/hiverouter-core/src/prompts.rs`](../../../backend/crates/hiverouter-core/src/prompts.rs) | `Persona`, `PersonaTier`, `validate`, licence allowlist |
| HM-2 | [`backend/crates/hiverouter-core/src/caps.rs`](../../../backend/crates/hiverouter-core/src/caps.rs) | depth/fan-out/agent/call caps and named refusals |
| HM-3 | [`backend/crates/hiverouter-core/src/execution.rs`](../../../backend/crates/hiverouter-core/src/execution.rs) | tiers, spawn authorization, dispatch order, aggregation, `ask_parent`/`send_to_sibling`, terminal guards, memory writes |
| HM-4 | [`backend/crates/hiverouter-core/src/planner.rs`](../../../backend/crates/hiverouter-core/src/planner.rs) | plan DAG, validation, leaf/aggregation order |
| HM-5 | [`backend/crates/hiverouter-core/src/store.rs`](../../../backend/crates/hiverouter-core/src/store.rs) | `agent_messages`, `agent_memory`, recall and clear semantics |
| HM-6 | [`backend/crates/hiverouter-tools/src/tool.rs`](../../../backend/crates/hiverouter-tools/src/tool.rs) | tool ids, argument schemas, permission classes |
| HM-7 | [`backend/crates/hiverouter-core/src/http.rs`](../../../backend/crates/hiverouter-core/src/http.rs) | profile routes, boot profile merge |
| HM-8 | [`backend/crates/hiverouter-core/src/bus.rs`](../../../backend/crates/hiverouter-core/src/bus.rs) | event bus scope (UI fan-out, not an agent bus) |
| HM-9 | [`backend/crates/hiverouter-core/assets/prompts/personas.json`](../../../backend/crates/hiverouter-core/assets/prompts/personas.json) | shipped profiles and allowlists |
| HM-10 | [`docs/CODEMAP/18-HMAS-RESEARCH.md`](../../CODEMAP/18-HMAS-RESEARCH.md) | prior in-repo research (LangGraph/AutoGen/CrewAI); licence, import-mapping and current-implementation delta reused rather than re-derived |
| HM-11 | [`docs/provider/AGENT-ARCHITECTURE-TRUTH.md`](../../provider/AGENT-ARCHITECTURE-TRUTH.md) | the repo's own statement that there is no global agent directory/pub-sub bus |

---

## 13. Verification status and limits

### 13.1 What the fetches did not give back

Recorded so no later reader assumes more coverage than exists.

- **A2A specification overview** returned HTTP 200 but the body was navigation/TOC only. All A2A claims in
  this document rest on the four topic pages (A2A-2..4), which returned complete bodies. The exact
  `TaskState` **enum string values** (e.g. `TASK_STATE_COMPLETED` seen in examples) were not read from the
  normative enum; the prose terminal-state names were. Treat enum spellings as UNVERIFIED.
- **A2A idempotency** (§3.3.1 of the spec) was not read. No idempotency claim is made about A2A.
- **Google ADK** documentation redirected (`google.github.io/adk-docs` → `adk.dev` → `/workflows/`), and the
  page returned navigation only. ADK is therefore **not** used as a source, even though its
  `sub_agents`/`transfer_to_agent` design would have been a useful fifth framework.
- **LangGraph multi-agent page** returned 404 at `docs.langchain.com/oss/python/langgraph/multi-agent`; the
  supervisor content was taken from the migration guide (LG-1) instead.
- **Claude Code subagents page** was truncated by the fetcher after the nesting/concurrency sections. The
  sections used (isolation, depth, concurrency, description budget, report authority, output scanning,
  resume) were fully read; later sections (worktree isolation, skills, hooks) were not.
- **LangGraph `recursion_limit` default value** is not stated on the page read. The document therefore says
  "configurable" and never quotes a default number.
- **AutoGen `max_turns`** was not verified as a constructor parameter; the document only names the three
  termination conditions that were read.
- **HiveMind runtime behaviour** (as opposed to code paths) was not exercised: no test was run, no server was
  started. Every HM claim is a code read. In particular, "an operator can see a nested child's pending
  approval while the parent is blocked in `spawn_agent`" is *not* proven either way — it is flagged as an
  open question in Q17 and P1-4.

### 13.2 Claims register

| Claim | Source | Status |
|---|---|---|
| `ask_parent` writes a message but no answer path exists | code read `execution.rs:2417-2487`, `store.rs:538-590`, only `http.rs:674-690` consumes messages | **VERIFIED in code** |
| `spawn_agent` runs the child synchronously and returns its result | `execution.rs:2655-2691` | **VERIFIED in code** |
| Execution within a run is sequential | `execution.rs:1396-1426`, `:1431-1485`, `:1500` | **VERIFIED in code** |
| One active run per server | `execution.rs:434`, `:1032-1038` | **VERIFIED in code** |
| Terminal tasks cannot transition | `execution.rs:93-95`, `:2984-2999` | **VERIFIED in code** |
| A parent can complete while a descendant is non-terminal | Not prevented by any read code path; the run-level reconcile happens later (`execution.rs:1216`) | **INFERRED from code** — needs a test to confirm reachability (P0-2) |
| HiveMind has no agent bus | `bus.rs` is UI-only; `AGENT-ARCHITECTURE-TRUTH.md` says so explicitly | **VERIFIED** |
| MAST has 14 modes in 3 categories | abstract + appendix headings [MAST-1] | **VERIFIED** |
| MAST failure rates 41%–86.7% | §1 of the HTML paper [MAST-1] | **VERIFIED** |
| Anthropic multi-agent ~15× tokens | [ANTH-2] body text | **VERIFIED as published** (vendor self-report, **SECONDARY**) |
| Claude Code default nesting = 3 layers, concurrency = 20 | [CC-1] | **VERIFIED** |
| A2A prescribes no curated-registry API | [A2A-4] | **VERIFIED** |
| The A2A `TaskState` enum spellings | not read | **UNVERIFIED** |
| Any HiveMind runtime behaviour under load | not exercised | **UNVERIFIED** |

### 13.3 Method notes

- No paid API was called; all fetches were unauthenticated HTTP GETs.
- Vendor blogs are labelled **SECONDARY** and are never the sole basis of an invariant. Two claims
  (§8.2's "50 subagents" and §8.4's duplicated-work example) are SECONDARY-only and are used as
  illustrations of a failure class that MAST independently documents.
- arXiv IDs were verified by fetching the abstract page and matching the title; none were inferred from
  memory.

---

## 14. Bottom line for the HMARS goal

1. **The architecture is broadly right.** Twelve of twenty invariants already hold outright, and the
   three primitives that matter most — a validated plan DAG, synchronous delegate-with-result, and absorbing
   terminal states — are all present and better than what the surveyed frameworks ship.
2. **The single real defect is the escalation verb.** `ask_parent` cannot be answered. Everything else is an
   edge case; this one is a broken primitive that forces every worker to guess (MAST FM-2.2).
3. **The missing guarantees are all about time and provenance.** No deadline, no stall detection, no message
   correlation, no memory provenance, no per-task call attribution. None of these need new architecture —
   they are fields, counters and one guard.
4. **The complexity to avoid is the complexity that creates a second source of truth.** Bus, registry
   service, router, per-agent checkpoints, asynchronous steering — all of them compete with the task table.
   §7 is as important a deliverable as §6.
5. **The five named failure modes are addressable with six P0 changes**, each with a concrete acceptance
   test in §11. None of them require a new dependency, a new runtime, or a protocol.
6. **Do not adopt A2A internally.** Take its task lifecycle and its card; leave its transport at the edge,
   if it is ever needed at all.



