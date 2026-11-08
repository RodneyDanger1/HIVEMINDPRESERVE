# 05 — UI/UX patterns for visualizing hierarchical agents

**HMARS research seat 5 (UI/UX).** Written 2026-11-07. No product code, CSS, FigCraft file, or contract was edited by this seat. This document is research and a prioritized recommendation set; it is not an implementation.

**Scope.** How a production-grade interface should show a hierarchical multi-agent run: topology/spatial canvas, agent cards, tier/role differentiation, live tool-call activity, approvals, branch memory, A2A messages, run budgets, error/refusal states, density without clutter, accessibility, and motion. Framework-level findings (import formats, persistence, licensing) live in `docs/CODEMAP/18-HMAS-RESEARCH.md` and are not repeated here. The current backend contract is frozen in `docs/contracts/UI-COMMANDS-v1.md`. Sibling HMARS files are used for cross-seat consistency: `03-scale-concurrency.md` for the 50–150 agent / 300–600 task target, `04-memory-architecture.md` for the memory-bank model, and `02-hmars-patterns.md` for the three control verbs. Where this document differs, it says so.

**The one-line answer.** HiveMind already has a distinctive and defensible visual language (honeycomb canvas, beehive chat core, bee-direction edges, honey/crimson state semantics). The UI problem is therefore *not* "find a prettier dashboard"; it is to **discipline the existing language into a small number of honest projections of one graph**, and to delete the generic dashboard patterns that have crept in. Every recommendation below is grounded in a FigCraft board, a frozen contract fact, or a primary design-system/agent-UI source — never in "modern SaaS" taste alone.

---

## 0. Claim classes used in this document

| Tag | Meaning |
|---|---|
| **VERIFIED** | I fetched the primary URL on 2026-11-07 and read the stated claim there. |
| **READ** | I read the claim in this repository at the cited `path:line` (or in a repo audit that cites `path:line`). |
| **OPINION** | A design judgement. It is labelled inline and is not a measurement, contract fact, or source claim. |
| **UNVERIFIED** | Not proven here. Do not build on it without the named experiment. |

Two consequences, stated up front:

- Every rendered fact in a HiveMind surface must still obey the "no fake controller" boundary in `docs/trace/ui-rebuild/04-hmas-surface-parity.md:1107-1174`: server event, real HTTP route, or a **derived-and-labelled** value. A beautiful pattern that needs invented data is a defect, not a design.
- **Today** the backend caps a run at **64 task nodes**, depth **3**, fan-out **8**, **40 provider calls**, **400,000 combined tokens**, **$0.50 settled cost**, **32 A2A messages**, and one live run at a time, plus a **global 8-slot provider in-flight limiter** (`backend/crates/hiverouter-core/src/caps.rs:9-15`; `backend/crates/hiverouter-core/src/execution.rs:68-70`; `backend/crates/hiverouter-provider/src/budget.rs:25-43`; READ). **The HMARS target is 50–150 concurrently active agents / 300–600 tasks under a HyperAgent** (`docs/research/hmars/03-scale-concurrency.md:1-23`, `417-455`; READ). Design the canvas for the target, not only the current cap: at 600 nodes, viewport culling, LOD bands, batched macro rendering, and a hard cap on animated edges are mandatory; a WebGL/Pixi rewrite is still not justified, because the cards are DOM/typography and the accessibility layer is DOM (`docs/trace/ui-rebuild/07-ui-stack.md:189-236`). That last sentence is [OPINION]; the performance gate is the threshold in `03-scale-concurrency.md:446-455` (≥50 fps at 600 nodes, ≤120 mounted nodes, ≤50 animated edges), not a vendor claim.

---

## 1. What is already fixed: HiveMind's design system baseline

### 1.1 Tokens and geometry (do not reinvent)

The token module is the only legal source of colour/type/space values (`web/src/tokens/tokens.ts:1-10`). The parts that constrain every pattern below:

| Dimension | Fixed values | Source |
|---|---|---|
| Surfaces | Canvas `#F8F4EB`, Paper `#FFFFFF`, Panel Alt `#FBF8F1`, Recess `#F1E9CF`, Hairline `#E5E1D8` | `web/src/tokens/tokens.ts:22-27` |
| Honey (accent/healthy) | Honey 100–800, `#FBF3DC` → `#604818`; honey-300 `#F0C050`, honey-400 `#E8B850` | `web/src/tokens/tokens.ts:28-35` |
| Ink (text) | Ink 900 `#282010`, 800 `#3F3115`, 700 `#4E3A15`, 500 `#75663F` | `web/src/tokens/tokens.ts:36-39` |
| State | Signal `#B23A2E` (error), Verify `#3F7A55` (success), Amber `#B7791F` (large text only) | `web/src/tokens/tokens.ts:40-42`, `72-79` |
| Type | Display XXL 96/100, display L 56/60, h1 40/48, h3 24/32, body L 17/28, label 12/16, mono code 13/22, mono meta **11/16** | `web/src/tokens/tokens.ts:91-100` |
| Space | 4, 8, 12, 16, 20, 24, 32, 40, 48, 64, 80, 96 | `web/src/tokens/tokens.ts:103-104` |
| Radii | 4, 8, 12, 16, 24, 48, 999; exceptions button 10, Super card 14 | `web/src/tokens/tokens.ts:107-111` |
| Cards | Hyper chat 400×480; Super 420×150; Worker 320×96; Tool 420×240; Error 320×110 | `web/src/tokens/tokens.ts:166-179` |
| Chrome | Topbar 52, rail 72, library 240, HUD 460×44, minimap 220×150 | `web/src/tokens/tokens.ts:166-179` |
| Routing | Snap radius 150 px; zoom 0.1–3.0 | `web/src/tokens/tokens.ts:127-138` |

Already-correct decisions that patterns must not "improve" away:

- **Colour is reserved for state.** Board 05 states it explicitly: "Colour is reserved for state — honey = healthy, crimson = broken route — so a swarm can be read across the canvas." (`docs/ui/figcraft/HiveMind/exports/hivemind-agent-cards.png`; READ as image.)
- **Meta text floor**: board 05 sets the meta floor at 9.5 px mono; the token module's mono-meta step is 11/16, so nothing should render below 9.5 px (the audited build still had 8 px labels). The implementation already treats `#8A7A52` as a WCAG failure and replaces it with ink-500 (`web/src/tokens/tokens.ts:71-79`, `91-100`; `docs/trace/A11Y-CONTRAST-PLAN.md:18-21`).
- **Focus ring is ink-700, not honey-300**, because honey-300 fails 3:1 on canvas (`web/src/tokens/tokens.ts:75-76`; `docs/trace/A11Y-CONTRAST-PLAN.md:20-21`).
- **The bee is a routing arrow, not decoration** (`web/src/tokens/tokens.ts:126-132`; board 05: "bee faces + flies along the data direction / the bee is a routing arrow, never decoration").
- **Elevation tiers are unresolved** — all three printed tiers carry the same value, so the UI must not claim tier semantics from shadow (`web/src/tokens/tokens.ts:113-124`).

### 1.2 The board set, and what each board is authoritative for

| Board | File | Authoritative for |
|---|---|---|
| 05 Agent cards | `hivemind-agent-cards.png` | Card anatomy, sizes, state pill/dot, meta type, bee edge, error card |
| 06 Controls & HUD | `hivemind-controls-and-hud.png` | Buttons and states, tabs, HUD census, time-travel scrubber, refusal toast, nav rail, library |
| 07 Spatial canvas | `hivemind-spatial-canvas.png` | Composition: 52 topbar / 72 rail / 240 library / hive chat at origin / 220×150 minimap / bottom hint |
| 08 HMAS hierarchy | `hivemind-hmas-hierarchy.png` | Root summary, tier rows, nesting rule, topology panel, state legend, Spawn/Kill |
| 09 HMAS governance | `hivemind-hmas-governance.png` | Profiles/import, memory, mailbox, projects, usage counters, message approval |
| 10 States & refusals | `hivemind-states-and-refusals.png` | Named refusal, reconnect, empty/loading, kill confirm, replay read-only, decision-gated diff |
| 11 Motion & live | `hivemind-motion-and-live.png` | **Proposed** durations/easing (not measured; board README says so), chat app variant 400×480 |
| 12 Run detail & files | `hivemind-run-detail-and-files.png` | Data/files, tool activity table, run detail, failure/refusal ledger, model status/usage six counters |

Sources: `docs/ui/figcraft/HiveMind/README.md:23-39`, `45-58`; READ. Board 07 is authoritative for shell composition; board 11's 400×480 chat is a documented app variant, not a second truth (`docs/trace/ui-rebuild/10-representation-audit.md:30-31`, `62-64`).

### 1.3 The frozen contract facts that decide the UI

| Fact | Value | Source |
|---|---|---|
| Commands | 14 frozen + 3 decision-gated `DIFF_*` | `docs/contracts/UI-COMMANDS-v1.md:7-15`, `28-45` |
| Events | **24** (`TOOL_CALL`, `TOOL_RESULT`, `AGENT_MESSAGE` added) | `docs/contracts/UI-COMMANDS-v1.md:47-61` |
| Agent states | **16** (`healthy, running, queued, failed, completed, cancelled, killed, planning, thinking, awaiting_approval, streaming, reconnecting, replay, empty, loading, interrupted`) | `docs/contracts/UI-COMMANDS-v1.md:68-74` |
| Refusal codes | 65 stable codes; UI must show code + `how_to_fix` | `docs/contracts/UI-COMMANDS-v1.md:76-86` |
| Mutation rule | Every mutating control must show ACK or a **named** refusal **before** mutating UI state | `docs/contracts/UI-COMMANDS-v1.md:88-93` |
| Approvals | `APPROVAL_RESOLVE` requires fresh, unused nonce; replay/timeout/not-found are named refusals | `docs/contracts/UI-COMMANDS-v1.md:37` |
| Replay | `HISTORY_REVISION_SET` → `HISTORY_SNAPSHOT`; read-only, no re-execution | `docs/contracts/UI-COMMANDS-v1.md:43`; `docs/trace/ui-rebuild/07-ui-stack.md:137` |
| Run budgets | 64 agents · 40 calls · 400k combined tokens · $0.50 settled cost · 32 messages · depth 3 · fan-out 8; plus a global 8-slot provider in-flight limiter | `backend/crates/hiverouter-core/src/caps.rs:9-15`; `backend/crates/hiverouter-core/src/execution.rs:68-70`; `backend/crates/hiverouter-provider/src/budget.rs:25-43` |

This is a *stronger* position than most agent UIs — [OPINION], but grounded in the frozen contract: typed refusals, explicit ACK, at-most-once effects, and a read-only replay axis already exist. The UI should surface that discipline, not hide it behind generic spinners and dashboard tiles.

---

## 2. What production/OSS agent UIs actually do (and what to take)

### 2.1 Topology and the spatial canvas

**Langfuse (OSS, production observability).** Its Agent Graphs view draws the same trace two ways: **Aggregated** (one node per step *name*, repeated calls collapse with a counter, loops draw as cycles) and **Expanded** (one node per call, loops unrolled into a DAG in execution order). The graph is inferred from observation types, sits next to **Tree** and **Timeline** tabs, and uses a deterministic ELK layout so "the graph lays out the same way every time" and "the viewport stays put as you pan and zoom" (**VERIFIED**, [docs](https://langfuse.com/docs/observability/features/agent-graphs), [changelog](https://langfuse.com/changelog/2026-07-13-graph-view-modes)). The lesson is not "copy Langfuse"; it is that **one dataset needs more than one projection, and each projection answers a different question** (shape vs. exact run).

**LangGraph Studio (production agent IDE).** It visualises the graph, runs it from the UI, lets you **edit state and rerun**, manages threads and long-term memory, and sets interrupts before/after nodes; interrupt response schemas render as **typed input fields**, not a raw JSON editor (**VERIFIED**, [Studio doc](https://raw.githubusercontent.com/langchain-ai/langgraphjs/c9e96aa500ac6c6c3410dd45bdf02bc9c1738503/docs/docs/concepts/langgraph_studio.md); [interrupts doc](https://raw.githubusercontent.com/langchain-ai/docs/main/src/oss/langgraph/interrupts.mdx)). LangGraph's time travel is *re-execution*: "Replay re-executes nodes—it doesn't just read from cache. LLM calls, API requests, and interrupts fire again" (**VERIFIED**, [time travel](https://raw.githubusercontent.com/langchain-ai/docs/main/src/oss/langgraph/use-time-travel.mdx)). That is the opposite of HiveMind's read-only `HISTORY_SNAPSHOT`, and the UI must say so (see §2.9 and §3.6). Subgraph checkpoint granularity is also a warning: without an own checkpointer, a subgraph is one parent super-step, so you cannot time-travel to a point inside it (**VERIFIED**, same URL). HiveMind's history is a run-scoped sequence of graph snapshots rather than a per-node checkpoint stack; each snapshot reconstructs every node and its real parent link, so branch boundaries stay visible at every revision even though the axis is run-level — a different, safer model, not a gap.

**Apache Airflow (production workflow UI).** The Dag details page separates **Grid View** (a task × run matrix, colour + tooltip, click a cell to inspect logs and retry) from **Graph View** (structural dependencies and branching), with separate Runs/Tasks/Events tabs (**VERIFIED**, [UI overview](https://airflow.apache.org/docs/apache-airflow/stable/ui.html)). This is the cleanest existing answer [OPINION] to "topology vs. history": the *graph* is structure, the *grid* is time, and the *run detail* is evidence. HiveMind should adopt that separation rather than putting a time-travel scrubber under a canvas and calling it history.

**AutoGen Studio (OSS prototype).** Team Builder composes agents/teams; Playground runs them with "live message streaming between agents" and a "visual representation of message flow through a control transition graph" (**VERIFIED**, [Studio](https://microsoft.github.io/autogen/stable/user-guide/autogenstudio-user-guide/index.html)). Microsoft is explicit that Studio is **not production-ready** and lacks auth/security hardening (same URL). Take the pattern (live message flow on a graph), reject the assumption that a graph equals a product.

**Temporal (production durable execution).** The Event History is an **append-only log** used for durability and audit; activity retries/timeouts are written as discrete events (`ActivityTaskScheduled/Started/Completed/Failed/TimedOut`), with limits (warning at 10,240 events, termination at 51,200) and optional principal attribution (**VERIFIED**, [Events](https://docs.temporal.io/workflow-execution/event)). HiveMind's `ERROR` frames + ledger are a smaller version of the same idea. The UI implication: history is a **record**, not an animation.

**Synthesis for HiveMind** [OPINION]:

1. Keep the **Hive canvas** as the primary *spatial* projection (board 07). It is the product's identity and it is reachable at ≤64 nodes.
2. Add a **Tree** projection of the same graph (board 08's hierarchy list, upgraded to a WAI-ARIA treegrid). A canvas alone cannot be the only representation: it is poor for long labels, screen readers, and "what is the 14th child of this SuperAgent?".
3. Keep a **Run timeline** (board 10's scrubber + board 12's run detail), separate from both. Do not make the canvas carry history semantics it cannot show.
4. Offer **Aggregated / Expanded** at the *card* level: collapsed SuperAgent card summarizes its branch (`3 workers · 2 running`), expanded card lists children. This is Langfuse's idea applied to hierarchy, not traces.
5. Never auto-reshuffle positions on selection or state change. Determinism is a usability property, not a rendering detail (Langfuse's deterministic layout is evidence; `VERIFIED`).

### 2.2 Agent cards: what a card is *for*

HiveMind's board 05 defines five card types with a shared anatomy: name, role tag, live state, progress; Super has a light honey header, Worker has no header fill and a 5 px track/3 px bar, Tool has a dark syntax box and gold execution bar, Error has a crimson border and dashed crimson wire (`docs/ui/figcraft/HiveMind/exports/hivemind-agent-cards.png`; READ). Board 05 also says "Five node types share one anatomy" — that is the design-system constraint.

OpenAI's Agents SDK visualization uses **shape** to distinguish node kinds: agents as rectangles (yellow), tools as ellipses (green), MCP servers as rectangles (grey), start/end nodes; **solid** arrows for agent-to-agent handoffs, **dotted** for tool invocations, **dashed** for MCP invocations, and identity is by underlying object rather than display name so two agents with the same label remain distinct nodes (**VERIFIED**, [Agent visualization](https://openai.github.io/openai-agents-python/visualization/)). Langfuse distinguishes observation **types** (`agent`, `tool`, `generation`, `chain`, `retriever`, `guardrail`, `evaluator`, `embedding`, `span`, `event`) and lets you filter by type (**VERIFIED**, [Observation types](https://langfuse.com/docs/observability/features/observation-types)).

What to take [OPINION]:

- **Kind is shape/surface, not hue.** Super = honey header + sub-hive container; Worker = flat body + thin bar; Tool = dark code surface; Error = crimson border + broken wire; Hyper = beehive containment. That vocabulary already exists on board 05 and survives greyscale.
- **A card shows at most four facts**: identity/name, kind/tier, state, and one live metric (progress or worker count). Everything else belongs in the inspector. Cards that try to show ctx, score, retry count, model, and cost at 320 px become unreadable and, worse, invite fabricated values.
- **Display name is not identity.** HiveMind already suffers from UUIDv7 prefix collisions: the first 8 hex characters of a UUIDv7 are a ~65-second timestamp bucket, so all tasks in one run collide, and the 48 px chip truncates to 6 characters and collides again (`docs/trace/provider/05-figcraft-ux-gaps.md:155-166`; READ/MEASURED there). Cards need a stable human label **from the producer** (additive `label` on `NODE_*`) or an honest role+depth+short-suffix label. OpenAI's object-identity rule is the generalized version: never rely on a display string as the key.

### 2.3 Tier/role differentiation without breaking state colour

HiveMind's tiers are `Hyper, Super, Worker, Tool` (`docs/CODEMAP/18-HMAS-RESEARCH.md:38-43`, citing `prompts.rs:340-369`; READ). The board set already encodes tier by **size and containment**:

- Hyper: 400×480 beehive chat core, the only eternal node, world origin.
- Super: 420×150 with honey header and "spawns its own workers underneath itself".
- Worker: 320×96, no header fill, thin progress track.
- Tool: 420×240, dark syntax box, tool chip, execution bar.
- Error: 320×110, crimson border, halted bee, warning glyph.

WCAG 1.4.1 requires that colour is **not the only visual means** of conveying information, action, response, or distinguishing an element (**VERIFIED**, [Use of Color](https://www.w3.org/WAI/WCAG22/Understanding/use-of-color.html)). HiveMind's state legend already pairs dot + text ("colour is never the only channel", board 08). The risk is tier: if tier is also encoded by hue (e.g. blue Super, green Worker, purple Tool), the UI creates **two competing hue scales** and state colour loses meaning.

**Recommendation** [OPINION]: encode the three axes on three different channels.

| Axis | Channel | Example |
|---|---|---|
| Tier/kind | geometry + surface + header | Super = wide + honey header + sub-hive container; Worker = narrow + flat; Tool = dark result surface; Error = crimson border + broken wire |
| Role/template | short text chip (mono meta) | `react/vite`, `db-schema`, `planner` |
| State | dot **+ glyph + text**, colour as reinforcement | `● running`, `▲ failed`, `⏸ awaiting_approval` |
| Selection | border/shadow change, never size change | ink-700 ring ≥3:1 |

This preserves board 05's "colour is reserved for state" rule and keeps the canvas readable in greyscale, at 55 % zoom, and for colour-vision deficiency.

### 2.4 Live tool-call activity: one home, three levels of detail

OpenAI's tracing model is the cleanest data model to mirror: a **trace** is one end-to-end workflow, composed of **spans** with `started_at/ended_at`, `trace_id`, and `parent_id`; default spans exist for the runner, task, turn, agent, generation, function tool, guardrail, handoff (and audio) (**VERIFIED**, [Tracing](https://openai.github.io/openai-agents-python/tracing/)). Sensitive inputs/outputs are gated by `trace_include_sensitive_data`, and rejected approval calls store only a generic error text when redaction is off (same URL). HiveMind's wire is deliberately leaner — `TOOL_CALL` carries `tool`, `args_sha256`, `turn`; `TOOL_RESULT` carries `outcome` (`ok|refused|denied|timeout|cancelled`), optional `code`, `bytes`, `sha256` (`docs/contracts/UI-COMMANDS-v1.md:58-59`) — but the hierarchy is the same: run → task/branch → turn → call → result.

The existing W11 design is already production-grade and should be treated as the spec, not re-litigated:

- `role="log" aria-live="polite"`, newest-first, `(turn, seq, arrival)` ordering, 200-entry FIFO with a truncation footer, dedupe by `tool_call_id`, unmatched results never dropped (`plans/waves/W11-UI-TOOL-ACTIVITY.md:100-116`, `202-208`, `238-256`; READ).
- Outcome is a **closed five-value enum**; unknown tokens are rejected by the decoder as `E_SCHEMA_INVALID`, never coerced to `ok` (`plans/waves/W11-UI-TOOL-ACTIVITY.md:106`; READ).
- No spinner is ever a stand-in for an outcome; `requested`/`running` are static text/dot states; refused/denied/timeout/cancelled always show code + `how_to_fix` where known (`plans/waves/W11-UI-TOOL-ACTIVITY.md:256`; READ).

Where the current product disagrees with itself: `ChatCore.tsx:273-307` has an in-chat `ACTIONS` block (last 4 entries, no turn grouping, no result body), while `ToolActivityPanel` renders the panel table; board 12 puts `TOOL ACTIVITY` in **run detail** (`docs/trace/provider/05-figcraft-ux-gaps.md:73-86`; READ/MEASURED there). [OPINION] Two tables is not "live feel"; it is duplicate state and a future inconsistency.

**Recommendation** [OPINION]: three levels, one source.

1. **Canvas/tool card**: a single event-driven pulse + `tool · running` label; the card is a *signpost*, not a log.
2. **Chat**: one compact strip above the composer while tools run (`2 tools running · 1 awaiting approval` + link), not a table. Chat is for conversation, not forensic evidence.
3. **Run detail (board 12)**: the full `TOOL ACTIVITY` ledger with branch/turn grouping, outcome legend, `args_sha256 → result sha256/bytes`, filters (`task`, `turn`, `outcome`), and empty/truncated states. This is where an operator goes to prove what happened.

### 2.5 Approvals: a queue with a consequence, not a modal jump-scare

OpenAI's HITL flow is **run-wide**, including approvals raised after a handoff or inside nested `Agent.as_tool()`; the run pauses, `RunResult.interruptions` carries `agent.name`, `tool_name`, `arguments`, and `RunState` is serialized/approved/rejected and resumed; per-call approvals are scoped to the call ID, sticky decisions are opt-in, and callable approval rules **fail closed** on unparseable arguments (**VERIFIED**, [Human-in-the-loop](https://openai.github.io/openai-agents-python/human_in_the_loop/)). LangGraph's `interrupt()` pauses indefinitely, resumes with `Command(resume=...)`, **restarts the node from the beginning**, and therefore requires idempotent side effects before the interrupt; multiple interrupts are matched by call order; a typed `response_schema` lets clients render typed forms (**VERIFIED**, [Interrupts](https://raw.githubusercontent.com/langchain-ai/docs/main/src/oss/langgraph/interrupts.mdx)).

HiveMind is stricter in some ways (nonce, TTL, single-use, server ledger) and weaker in others (one modal per approval, no queue surface, expired card can trap). The W11 and foresight plans already define the correct state machine: `pending → resolving → awaiting_server → closed`, with `expired`, `stale`, `cancelling`; ACK is **not** terminal; only `APPROVAL_RESOLVED`/`ERROR`/timeout closes; deny-first focus; Escape never approves or denies; no optimistic close; retry max 3 with the same nonce, never after `E_APPROVAL_REPLAY` (`plans/waves/W11-UI-TOOL-ACTIVITY.md:111-116`, `212-236`; `plans/foresight/APPROVAL-UX-PLAN.md:127-171`; READ).

**Recommendation** [OPINION]: present approvals as **inbox + focused decision**:

- A persistent **Approval inbox** count in the HUD/rail (only when approvals exist) opens a queue panel: one row per approval with `from → to`, tool, task/branch, age, TTL; board 09's `MESSAGE APPROVAL` card is the visual reference.
- Selecting a row opens the **modal** (board 09/10): title, branch narrative, tool, task/tool-call ids (short), `args_sha256` (12-hex + copy), optional server-redacted preview (≤2 KiB), TTL, nonce single-use, "Approve grants the engine one consume" consequence copy, Deny-first, Approve. This is not a generic confirm dialog; it is a **decision record**.
- Decision outcomes (approved/denied/expired) stay visible in the run's **approval ledger** row; never vanish into a toast.
- Never add "remember this decision" / auto-approve to the UI until the backend has a sticky-decision model with an audit trail. OpenAI's sticky decisions are opt-in and scoped; HiveMind's nonce is single-use by design.

### 2.6 Branch memory: two different memories, two different surfaces

LangGraph separates **checkpointers** (thread-scoped graph state; conversation continuity, HITL, time travel, fault tolerance) from **Stores** (long-term, cross-thread application data) and explicitly recommends most applications use both; in-memory savers die on restart, and checkpoints need pruning (**VERIFIED**, [Persistence](https://raw.githubusercontent.com/langchain-ai/docs/main/src/oss/langgraph/persistence.mdx)). A2A separates a `contextId` that groups tasks/messages from each task's state; terminal tasks are immutable and refinements start a new task in the same context; artifact versioning is the client's responsibility (**VERIFIED**, [Life of a Task](https://a2a-protocol.org/latest/topics/life-of-a-task/)).

HiveMind's current memory is role-scoped (`agent_memory`, migration v7), upserts/recalls the latest 8 rows into task context, and has **no editor/clear/consent UI** yet (`docs/CODEMAP/18-HMAS-RESEARCH.md:145-152`; READ). Board 09 shows a Memory panel with run summary, decision log, refusal ledger, and a "Clear memory" button with confirmation-first copy; the ledger is append-only (`docs/ui/figcraft/HiveMind/exports/hivemind-hmas-governance.png`; READ). Sibling research proposes a bank tree as the HMAS-scope successor: project, run, hive, branch, and agent-role banks; snapshot inheritance by `as_of_rev`; deny-by-default reads; one-level promotion; trust levels; a hash-chained journal and purge receipts (`docs/research/hmars/04-memory-architecture.md:92-102`, `449-527`; READ). The UI should be designed for that model, not for a generic "memory store".

**Recommendation** [OPINION]: two surfaces, never merged:

1. **Run state** (checkpoints/revisions) is the replay axis: read-only, revision scrubber, no edit. Do not put "memory" language on it, and do not let replay write.
2. **Durable branch memory** is a **Memory inspector** with the five bank kinds as scope tabs: `Project`, `Run`, `Hive`, `Branch`, `Role`. Each row shows `kind`, `bank`, `source task`, `trust` (`unassessed/quarantined/unverified/verified/authoritative`), `written_rev`, `superseded`, `retention`, and a provenance link. The panel declares the read set ("own + ancestors at `as_of_rev` + role bank") and the render budget ("12 entries / 4,000 chars; 3 omitted, truncated"), because those are the rules that make the view trustworthy.
3. Allowed writes are **promote one level** and **purge**; both are confirmation-first and audited, and purge leaves a receipt with no content. There is no free-text edit until a write route and consent model exist. Show `not measured` / `not exposed` for anything without a route (there is no memory-ledger read endpoint today).

### 2.7 A2A messages: a mailbox ledger, not a chat room

A2A's core distinction is **stateless `Message` vs stateful `Task`**: a message can be a self-contained exchange; a task has a lifecycle (`input-required`, `auth-required`, `completed`, `canceled`, `rejected`, `failed`), progress, artifacts, and immutability once terminal; `contextId` groups related interactions across tasks (**VERIFIED**, [Life of a Task](https://a2a-protocol.org/latest/topics/life-of-a-task/)). AutoGen Studio's Playground shows live message streaming between agents on a control-transition graph (**VERIFIED**, [Studio](https://microsoft.github.io/autogen/stable/user-guide/autogenstudio-user-guide/index.html)). HiveMind's wire is `AGENT_MESSAGE {run_id, from_task_id, to_task_id, kind, text, created_at}` (`docs/contracts/UI-COMMANDS-v1.md:60`); tools are durable `ask_parent`/`send_to_sibling`; there is a **32-message per-run budget**; cross-branch sends suspend for approval (`docs/CODEMAP/18-HMAS-RESEARCH.md:142-146`, `backend/.../execution.rs:68-70`; READ).

**Recommendation** [OPINION]: the mailbox is a **ledger of branch events**, not a chat panel.

- Row: `from → to` (short task labels, role if available), `kind` (`ask_parent`, `send_to_sibling`, …), message text (clamped), `created_at`, and a link to the owning task/run. Group by branch pair; oldest-first with a jump-to-latest control; `role="log"`.
- Board 09's "Mailbox · 2 unread" is only honest when a read/unread field exists. Today it does not (`docs/trace/ui-rebuild/04-hmas-surface-parity.md:1053-1055`; READ). Show **total** and "unread not reported", or add a read model; do not fabricate a badge.
- No compose box until a send route exists. [OPINION] The current read-only design is correct; the UI should state "Mailbox is read-only · delivery is a branch event" (board 09 copy).
- Budget visibility: show `messages 7 / 32` in the mailbox header, sourced from a real counter or labelled absent. A refusal due to the 32-cap must name the budget and the reset condition.

### 2.8 Run budgets: meters, not vanity metrics

OpenTelemetry's GenAI conventions (v1.45.0, Development status) define agent-scoped metrics: `gen_ai.invoke_agent.duration`, `gen_ai.invoke_agent.inference_calls`, `gen_ai.invoke_agent.tool_calls`, with the rule that nested invocations count their own calls so each call is counted **exactly once across the call tree**, and `error.type` when an operation fails; `deepseek` is a recognized `gen_ai.provider.name` (**VERIFIED**, [gen-ai-metrics.md](https://github.com/open-telemetry/semantic-conventions-genai/blob/main/docs/gen-ai/gen-ai-metrics.md)). DeepSeek's API defines `max_tokens` between 1 and 384K (393216), returns usage with `prompt_cache_hit_tokens`/`prompt_cache_miss_tokens`/`reasoning_tokens`, and `finish_reason` ∈ `stop | length | content_filter | tool_calls | insufficient_system_resource | aborted` (**VERIFIED**, [Chat Completions](https://api-docs.deepseek.com/api/create-chat-completion)). Anthropic adds distinct reasons for truncation and refusal: `max_tokens`, `model_context_window_exceeded`, `pause_turn`, `refusal` (**VERIFIED**, [Stop reasons](https://platform.claude.com/docs/en/build-with-claude/handling-stop-reasons)).

HiveMind's actual run budgets are engine caps, not provider caps: max depth 3, fan-out 8, 64 agents, 40 provider calls, 400,000 combined tokens, $0.50 settled cost, 32 messages, one live run; the provider client also has a **global** 8-permit in-flight limiter (`backend/crates/hiverouter-core/src/caps.rs:9-15`; `backend/crates/hiverouter-core/src/execution.rs:68-70`; `backend/crates/hiverouter-provider/src/budget.rs:25-43`; READ). The current HUD shows `SUPERAGENTS · WORKERS · GREEN %` (board 07) but no budgets; `ModelsPanel` shows usage counters after the fact.

**Recommendation** [OPINION]: one **Run budget** strip, owned by run detail, mirrored compactly in the HUD:

| Meter | Source | Render |
|---|---|---|
| Agents | task count / 64 | `agents 12 / 64` + bar; derived, labelled `derived` if needed |
| Provider calls | run calls / 40 | `calls 18 / 40` |
| Messages | `agent_messages` count / 32 | `messages 7 / 32` |
| Tokens | `run_usage` total / 400,000 | `tokens 12,400 / 400k`; `—` when not reported |
| Cost | settled spend / $0.50 | `cost $0.00 / $0.50` only when settled; otherwise `$0.50 cap · spend not priced` |
| Depth / fan-out | max depth / 3, max children / 8 | text `depth 2 / 3 · max fan-out 3 / 8` |
| In-flight (global) | provider client limiter / 8 | status chip `provider 3/8 in flight`; **not** a per-run budget, so never put it in the run meter row |

Rules: unknown is `—` / `not reported`, never 0; warn at 80 % with text + icon, not colour alone; a cap hit renders the named refusal (`E_AGENT_LIMIT`, `E_CALL_LIMIT`, `E_FANOUT_LIMIT`, `E_SPAWN_DEPTH`, `E_BUDGET_EXCEEDED`) and the budget row that caused it; `finish_reason=length` and `content_filter` are **provider stop reasons**, not HiveMind refusals, and must be labelled as such. Do not show a dollar spend derived from a guessed price; the cap is real, the spend number is not settled (`docs/CODEMAP/18-HMAS-RESEARCH.md:147`; READ).

### 2.9 Error, refusal, and truncation are three different things

This is [OPINION] the most common category error in agent UIs, and HiveMind's contract already lets the UI get it right.

| Class | Examples | Where it comes from | UI treatment |
|---|---|---|---|
| **HiveMind refusal** | `E_SPAWN_DEPTH`, `E_FANOUT_LIMIT`, `E_APPROVAL_REPLAY`, `E_TOOL_TIMEOUT`, `E_KEY_MISSING` | frozen contract; 65 stable codes | Named error frame: code + message + `how_to_fix`; inline near the source; `role="alert"`; ledger row |
| **Provider stop reason** | `length`, `content_filter`, `insufficient_system_resource`, `aborted` (DeepSeek); `refusal`, `max_tokens`, `model_context_window_exceeded`, `pause_turn` (Anthropic) | provider response, not an error frame | Inline status on the message/tool result: "Stopped: content filter" / "Truncated: max tokens"; offer continue/raise-budget only where the command exists |
| **Tool outcome** | `ok`, `refused`, `denied`, `timeout`, `cancelled` | `TOOL_RESULT` | Outcome chip + text + shape; never a spinner; code/`how_to_fix` when known |

NN/g's error-message guidance maps almost one-to-one to the W11 plan: display the message close to the source, use redundant accessible indicators (not colour/animation alone), design by impact (toast/banner for low-impact, modal only for severe), avoid premature errors, use human language and hide codes to diagnostics, offer constructive advice, preserve input, and never blame the user (**VERIFIED**, [Error-Message Guidelines](https://www.nngroup.com/articles/error-message-guidelines/)). Board 10 already encodes the house version: `E_APPROVAL_TIMEOUT` with message + `how_to_fix`, `ERROR FRAME → TOAST + LOG + LEDGER`, "REFUSED BY NAME · NOT SILENT".

**Recommendation** [OPINION]: a single `RefusalRecord` shape (`code`, `message`, `how_to_fix`, `retryable`, `surface`, `correlation_id`, `run_id`, `task_id`, `at`) rendered by three placements: inline strip (form/control), toast (transient, max 4, dismissible, `aria-live`), and run ledger (persistent). A refusal is never only a toast, and never silently swallowed. The toast is the *notification*; the ledger is the *record*.

### 2.10 Density without clutter: the overview-first ladder

Shneiderman's visual information-seeking mantra — **"overview first, zoom and filter, then details on demand"** — was coined for interactive data graphics and applies directly to a graph of agents (**VERIFIED** summary and attribution, [data.europa.eu guide](https://data.europa.eu/apps/data-visualisation-guide/the-information-seeking-mantra), citing Shneiderman 1996). NN/g's progressive disclosure adds the operational rule: show a few important options first, disclose the rest on request, keep the split right, make progression obvious, and **avoid more than two disclosure levels in practice** (**VERIFIED**, [Progressive Disclosure](https://www.nngroup.com/articles/progressive-disclosure/)). Carbon's spacing guidance adds the visual mechanism: proximity creates relationships, more space creates hierarchy, and "sections of a UI are allowed to be dense, but the whole page should not be crowded" (**VERIFIED**, [Spacing](https://www.carbondesignsystem.com/building-blocks/foundations/spacing/overview)).

Airflow's Grid View is the counter-example that works: a genuinely dense task × run matrix, but it is *one* dense region with a clear axis, filters, and drill-down — not a page of unrelated stat tiles (**VERIFIED**, [UI overview](https://airflow.apache.org/docs/apache-airflow/stable/ui.html)).

**Recommendation** [OPINION]: a **three-level ladder** and a hard density budget:

| Level | Surface | Shows | Hides |
|---|---|---|---|
| L0 Overview | Hive canvas at fit zoom (or Tree root) | tier shapes, state, parent links, one metric per card, HUD census | labels below LOD, tool detail, memory, messages |
| L1 Focus | selected card + inspector | identity, state, progress, parent/children, actions | full tool ledger, memory rows, approval history |
| L2 Evidence | run detail / drawers | tool calls, approvals, budgets, refusals, memory, mailbox | nothing (this is the audit level) |

Density budget: **one primary region + one inspector + one status strip**; at most two side panels visible at 1440; each card ≤4 facts; no stat-tile row that duplicates the HUD; no hero empty state with numbered onboarding steps. The current Swarm view violates this with 5 stat tiles and a hero empty state (`docs/trace/provider/05-figcraft-ux-gaps.md:92-117`, `138-151`; READ/MEASURED there).

### 2.11 Accessibility: the canvas is a view, not the only view

The relevant standards are specific and testable:

- **WCAG 2.2 SC 2.5.8 Target Size (Minimum), AA**: pointer targets at least **24×24 CSS px**, with a spacing exception; for important controls aim for 2.5.5's 44×44 (**VERIFIED**, [Target Size](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html)).
- **SC 2.4.13 Focus Appearance, AAA**: focus indicator at least a 2 px perimeter and ≥3:1 change of contrast; SC 2.4.7/1.4.11 still require a visible, adjacent-contrast indicator (**VERIFIED**, [Focus Appearance](https://www.w3.org/WAI/WCAG22/Understanding/focus-appearance.html)).
- **SC 1.4.1 Use of Color, A**: colour is never the only visual means (**VERIFIED**, [Use of Color](https://www.w3.org/WAI/WCAG22/Understanding/use-of-color.html)).
- **SC 2.3.3 Animation from Interactions, AAA**: non-essential interaction-triggered motion must be disableable (**VERIFIED**, [Animation from Interactions](https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions.html)). `prefers-reduced-motion` exists precisely because scaling/panning large objects can trigger vestibular symptoms (**VERIFIED**, [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@media/prefers-reduced-motion)).
- **WAI-ARIA APG**: the **treegrid** pattern gives hierarchical rows/cells, `aria-expanded`, and a complete keyboard model (arrows, Home/End, Page Up/Down, Enter/Space) (**VERIFIED**, [Treegrid](https://www.w3.org/WAI/ARIA/apg/patterns/treegrid/)); the **feed** pattern gives an interoperability contract for dynamically loading article streams, with `aria-posinset`, `aria-setsize`, `aria-busy`, and Page Up/Down navigation (**VERIFIED**, [Feed](https://www.w3.org/WAI/ARIA/apg/patterns/feed/)).

HiveMind's current canvas already has `role="application"`, an `aria-label`, a skip-link target `#hive-canvas`, `aria-live` node announcements, and focusable nodes/edges (`docs/trace/ui-rebuild/07-ui-stack.md:118-140`; READ). That is a good start, but `role="application"` is a promise that the widget implements its own keyboard model; if it does not, screen-reader users are trapped. The repo's own A11Y plan records that DOM gates are still blocked and that 19/33 visible buttons were under 40 px in the audited build (`docs/trace/provider/05-figcraft-ux-gaps.md:208-210`; READ).

**Recommendation** [OPINION]:

1. **Tree is the accessibility peer of Hive**, not a fallback. Both project the same `TaskRecord[]`; the Tree uses a treegrid with `aria-expanded`, roving tabindex, and the APG key map. A "View: Hive / Tree" toggle is a first-class control, not a settings checkbox.
2. Every canvas action has a keyboard path: select (Tab/arrows), open inspector (Enter), rewire via a **"Re-route…" command** (not drag-only), kill/approve via their dialogs. Drag remains an accelerator, never the only method.
3. Live activity uses `role="log" aria-live="polite"` and announces **state transitions**, not every token; approvals use `alertdialog` with focus moved to Deny and returned on close (W11 already specifies this).
4. 24 px minimum for every interactive element; 40 px for primary actions; never a 16 px icon button without a 24 px hit area. The focus ring is ink-700 at ≥3:1, with a 2 px perimeter and offset.
5. Do not use `role="application"` unless the full APG composite-widget keyboard contract is implemented and tested; otherwise expose the canvas as a labelled region plus a treegrid alternative.

### 2.12 Motion: event-driven, one ambient loop, zero under reduced motion

Carbon's motion system distinguishes **productive** (subtle, task-focused microinteractions) from **expressive** (occasional, significant moments), gives easing tokens and a non-linear duration scale (`fast-01` 70 ms, `fast-02` 110 ms, `moderate-01` 150 ms, `moderate-02` 240 ms, `slow-01` 400 ms, `slow-02` 700 ms), and asks whether motion is purposeful, responsive, meticulous, and unobtrusive (**VERIFIED**, [Motion](https://www.carbondesignsystem.com/building-blocks/foundations/motion/overview)). Board 11 proposes 1.2 s working-bee, 2.4 s bee flow, 6 s sheen, and 18 s ambient durations (as recorded in the later audit), but the README explicitly labels the board **PROPOSED, not measured** (`docs/trace/provider/05-figcraft-ux-gaps.md:127-136`; `docs/ui/figcraft/HiveMind/README.md:53-55`; READ).

The current build measured **12 concurrent animations** on the 1440 workspace — three ambient loops (glow, sheen, scan) plus three free-flying bees plus a hidden pulse — against the motion layer's own comment "one calm movement at a time" (`docs/trace/provider/05-figcraft-ux-gaps.md:127-136`; READ/MEASURED there). That is [OPINION] motion volume substituting for event-driven feedback.

**Recommendation** [OPINION]:

- Motion is **event-driven**: card state change (one 120–160 ms transition), edge re-route (one 200–240 ms draw), approval arrival (one 180 ms rise), toast (one 150 ms fade). These are productive motions; none carries meaning alone.
- **One ambient loop maximum**, and only if it is below the threshold of attention; no free-flying bees. The bee is a direction arrow; SMIL dash travel on an edge is decoration, not data.
- Reduced motion (OS or in-app) sets duration to 0 for camera, cards, edges, toasts, and countdowns; state still updates. Never gate information behind motion.
- A motion budget is testable: count `document.getAnimations()` under reduced motion = 0; under normal motion ≤3 concurrent, none infinite except one ambient. Add it to the visual QA gate.

---

## 3. The HiveMind pattern set (recommendation)

This section is the design answer: a small set of surfaces, each with a single job, all projecting the same `TaskRecord[]` / event stream. It is deliberately conservative about new chrome — the product already has enough surfaces; the main work is **removing** the generic ones and **sharpening** the Hive-specific ones.

### 3.1 Information architecture: four work destinations, two projections, one record

Current IA: nav rail (Workspace, Agents, Data, Models, Routing, Settings) plus library tabs (Workspace/Swarm/Routing/History) that are painted globally but only affect Workspace (`docs/trace/provider/05-figcraft-ux-gaps.md:115-117`; READ/MEASURED). That is a lying control: `HISTORY` can be highlighted while Models is displayed.

**Proposed IA** [OPINION]:

| Destination | Job | Surfaces |
|---|---|---|
| **Workspace** | Do the work: prompt, watch the hive, steer | Hive canvas + pinned hive chat + control bar + HUD |
| **Swarm** | Understand the hierarchy | Same graph, **Hive / Tree toggle**, root summary, census, filters, state legend |
| **Run detail** | Prove what happened | Tool ledger, approvals ledger, run budget, failures/refusals, files (board 12) |
| **Agents** | Govern the mesh | Profiles/import, memory inspector, mailbox (board 09) |
| Data / Models / Settings | Resource and provider configuration | unchanged, but each gets a real empty/error state |

Rules:

- The library tabs become **Workspace-scoped** and disappear (or disable with a reason) on other destinations. A tab that cannot change the view is removed.
- `Routing` is not a separate destination. A parent-edge table is a debug view, not a product surface; fold it into Swarm's Tree as a "Parent" column, or keep it behind a developer disclosure. [OPINION]
- `History` is not a tab. Replay belongs to **Run detail** (revision rail + scrubber), and the scrubber remains on Workspace because it changes what the canvas shows. [OPINION]
- No new "HMAS console" destination. [OPINION] The four work destinations above already cover the capability set; a fifth top-level surface would dilute the hive metaphor and add another empty state to maintain.

### 3.2 Hive canvas: LOD bands, hysteresis, and a stable camera

The canvas is the product's signature. It should behave like a **map**, not like a dashboard with cards on it.

**Zoom bands** [OPINION; thresholds chosen against the 0.1–3.0 range and the fixed card sizes]:

| Band | Zoom | What renders | Why |
|---|---|---|---|
| **B0 Macro** | 0.10–0.34 | One batched SVG layer: each SuperAgent subtree collapses to a cluster hex (`N workers · state`); individual silhouettes only for roots/clusters; labels above 0.25; no card DOM | At 0.1× even 64 cards (today) or 600 (target) are unreadable; clustering is the honest macro view (`docs/research/hmars/03-scale-concurrency.md:436-444`; READ) |
| **B1 Far** | 0.35–0.59 | Header + state dot only; name truncated to one line; no progress/metrics | You are navigating, not reading |
| **B2 Fit** | 0.60–1.19 | Full current card anatomy: name, tier/role, state pill, one metric, progress | Default working band |
| **B3 Detail** | 1.20–2.19 | B2 + role chip + last event/turn + worker count (Super) | Inspecting a branch |
| **B4 Inspect** | 2.20–3.00 | B3 + args hash / code preview (Tool) / error code (Error); inspector recommended | Reading evidence on the card |

Mechanics:

- Hysteresis **±0.04** around each boundary; a band change re-renders `nodeTypes`, not the graph. No per-frame React state.
- **Culling**: render only nodes intersecting the viewport plus a one-screen margin (`onlyRenderVisibleElements` in React Flow is the library-level start; the batch layer handles B0). The current build has no culling (`docs/trace/ui-rebuild/07-ui-stack.md:134`; READ).
- **Layout**: one layout per topology change, memoised by a stable `topologyKey` (run + sorted `id:kind:parentId`). Root at world `(0,0)` center; SuperAgents on a six-direction ring; children in 12 slots (column pitch 440, row pitch 381.0512, odd-row offset 220) — the lattice already specified in `docs/trace/ui-rebuild/07-ui-stack.md:344-379` (READ). Never re-run layout on selection; never re-run on drag.
- **Camera**: one owner (React Flow's transform). Pointer-anchored zoom; `Ctrl/Cmd+0` recentres `(0,0)` without changing zoom; camera is not persisted; reduced motion forces duration 0 (`docs/trace/ui-rebuild/07-ui-stack.md:323-342`; READ).
- **Minimap**: nodes as tier/state rectangles + viewport outline; `220×150` at ≥1280; below 1280 hide it and offer an "Overview" button that fits the graph (do not leave a blank box). Current minimap is textured but empty and vanishes at 1024 (`docs/trace/provider/05-figcraft-ux-gaps.md:167`; READ/MEASURED there).
- **Empty state**: one line + one action ("Submit a prompt to create the first run") on the canvas or in chat, not both. Never a hero panel with numbered steps.

**What not to do** [OPINION]: no infinite auto-pan, no particle field, no glow behind every node, no colour-by-tier. The honeycomb lattice is the atmosphere; the cards are the information.

### 3.3 Tree projection: the accessibility and density peer

The Tree is not a fallback for old browsers; it is the second projection of the same graph and the only one that scales to long labels and screen readers.

**Spec** [OPINION]:

```
role="treegrid" aria-label="Agent hierarchy"
Columns: Agent (tree column: expand/collapse, tier glyph, name, role)
         State (dot + text)
         Workers (Super only; count or —)
         Progress (reported % or —)
         Last event (time + short label)
Rows: root summary row (HyperAgent: run status, 4 superagents, 12 workers, healthy)
      indented Super rows; Worker/Tool/Error children
      depth announced as aria-level; expanded as aria-expanded
```

- Row height 32 px (compact) / 40 px (comfortable), density toggle persisted; `aria-rowcount`/`aria-rowindex` when virtualized; `role="row"` + `gridcell` (APG treegrid, **VERIFIED** [Treegrid](https://www.w3.org/WAI/ARIA/apg/patterns/treegrid/)).
- Keyboard: Up/Down rows, Right expand/first cell, Left collapse/parent, Home/End, Enter selects and reveals on canvas, Space toggles selection; `Ctrl+0` still recentres the canvas (shared shortcut).
- Root summary is derivable from `tasks` (root + children) and is therefore honest UI work, not a contract gap (`docs/trace/ui-rebuild/04-hmas-surface-parity.md:187-188`; READ).
- The Tree and the canvas share selection; selecting in either highlights the other. "Reveal on canvas" is a camera action, not a second selection model.
- The Tree is where **aggregated vs expanded** lives as row expansion; the canvas uses the same state (collapsed Super card = aggregated branch).

### 3.4 Card anatomy v2: four facts, honest unknowns

Keep board 05's sizes and surfaces. Change the content rules, not the look.

| Kind | Card content (B2) | Forbidden / honest fallback |
|---|---|---|
| **Hyper** | Name ("Hive AI Assistant"), pinned chat, run state, model (if reported) | No worker count on the chat header; the HUD owns census |
| **Super** | Name, `SUPERAGENT` chip, state pill, `N workers · M running` (derived), progress if reported | Do not put progress under a "workers" label (current defect, `docs/trace/provider/05-figcraft-ux-gaps.md:155-166`); no ctx/score without a field |
| **Worker** | Name, `WORKER` + role chip, state, progress bar **or** `—` | Do not render `ctx` (always null today; `web/src/flow/nodes.tsx:183-189`); do not assert `WorkerAgent` when `kind` was defaulted |
| **Tool** | Tool name, `TOOL` chip, state, dark result box (code/args preview when available), execution bar, score only if reported | No invented code text; args body only if server-redacted preview lands; otherwise `args_sha256` |
| **Error** | Crimson border, warning glyph, **real** error code or neutral "error state", retry attempt if reported | Never hardcode "Compile failed" (`web/src/flow/nodes.tsx:202`; `docs/trace/ui-rebuild/04-hmas-surface-parity.md:1023`) |
| **Broken route** | Dashed crimson wire + halted bee + warning glyph | Wire direction must still follow data flow (source→target) |

Card interaction rules:

- The card is a single focusable control: a real `<button>` when it contains no other interactive elements, otherwise a `div role="button"` with key handlers and a separate overflow menu. `aria-pressed` reflects selection. Any nested control must stopPropagation and remain reachable by Tab; never put a `<button>` inside a `<button>`.
- Selection is a border/shadow change, never a size change (size changes force edge recompute; `docs/trace/ui-rebuild/07-ui-stack.md:412-419`).
- A card shows state as **dot + glyph + text**; the pill is reserved for Super/Tool (board 05), the dot for Worker/Error, but both carry text in the accessible name.
- Unknown is not zero: `progress === null` renders no bar and `—`; `formatPercent/formatCount/formatCtx` remain the only formatters (READ, `docs/trace/ui-rebuild/07-ui-stack.md:412-415`).

**Card identity** [OPINION]: render `role · depth · short-suffix` when no server label exists, e.g. `worker · d2 · 7f3a`. A UUID prefix alone is not identity. When the additive `label` field lands, prefer `label` and keep the suffix as the disambiguator. This is a P0 truth fix, not typography.

### 3.5 State vocabulary: 16 states, 5 visual families

The contract has 16 states (`docs/contracts/UI-COMMANDS-v1.md:68-74`); a user cannot hold 16 visual variants. Map them to five families, then differentiate within a family by glyph + text.

| Family | States | Visual | Notes |
|---|---|---|---|
| **Live** | `running`, `streaming`, `thinking`, `planning` | honey-500 dot + activity glyph + text; one subtle transition on change | No endless spinner; streaming text is the feedback |
| **Steady** | `healthy`, `completed` | honey-300 dot / verify check + text | `completed` gets verify, not honey, so it is not confused with running |
| **Waiting** | `queued`, `awaiting_approval`, `interrupted` | amber non-text dot + pause glyph + text (amber passes 3:1 as an icon; never small amber text) | `awaiting_approval` must link to the approval; `interrupted` is not `failed` |
| **Terminal-negative** | `failed`, `killed`, `cancelled` | signal dot + distinct glyphs (`▲`, `⊘`, `—`) + text | Never one crimson blob for all three |
| **System** | `reconnecting`, `replay`, `empty`, `loading` | ink-500; banner/skeleton/ghost, not a node | These are surface states, not agent states; do not paint them on cards |

Rules: state is never colour-only (**VERIFIED** WCAG 1.4.1); the state legend lives in Swarm and is always visible there; the HUD's "green %" must be derived and labelled, or removed.

### 3.6 Run detail: the evidence surface (board 12)

One destination, five stacked sections, each with one job:

1. **Run summary**: project, run status, started/ended, model/provider (if reported), revision count, budget strip. No vanity charts.
2. **Tool activity** (the ledger): see §2.4. Columns: `turn`, `tool`, `task/branch`, `outcome`, `code`, `args_sha256` (8-hex + copy), `result` (bytes + 8-hex sha), `time`. Group headers by branch, then turn. Default filter: all; quick filters `running`, `refused/denied`, `timeout/cancelled`, `task`. Empty: "No tool calls yet — tool calls appear here when a run uses tools." Truncated: "Earlier activity not retained (cap 200)." Row click selects the task and opens the inspector.
3. **Approvals**: queue + decided ledger. Pending rows have TTL and a "Review" button; decided rows show decision, operator/agent, time, nonce consumed (yes/no), and link to the tool result. Expired/stale rows are read-only and dismissible. If the ledger read route is absent, label "session-only" and do not claim persistence.
4. **Failures & refusals**: run-level error card (`code`, `message`, `how_to_fix`, retryable), plus the refusal ledger (persistent only when a read route exists; otherwise "session-only"). Provider stop reasons render as `Stopped: content_filter` / `Truncated: length`, visually distinct from HiveMind refusals.
5. **Files** (board 12): tree + preview, read-only, path jail, 1 MiB cap; no diff until `DIFF_*` freezes.

Run detail is also where **budgets** live at full size; the HUD only mirrors the two most urgent meters (agents, calls) plus connection state.

### 3.7 Memory inspector

A single panel under Agents, shaped by the bank tree in `04-memory-architecture.md` (not by a generic key-value store):

```
Scope: [Project] [Run] [Hive] [Branch] [Role]      Search: ______
Read set: own + ancestors at as_of_rev + role bank · showing 12 entries / 4,000 chars · 3 omitted (truncated)
Kind         Bank              Source task    Trust        Written rev   Superseded   Retention   Actions
observation  branch:<task>     worker-7f3a    unverified   rev 12        no           unset       View · Promote · Purge
summary      hive:<super>      super-2c91     verified     rev 10        no           forever     View
```

Rules:

- `View` opens the stored value in a read-only drawer with kind, bank id, source task, `written_rev`, content hash, and `as_of_rev`; no inline editing.
- `Promote` is available only when the target is the bank's parent; it is audited and confirmation-first. `Purge` removes content but leaves a receipt (id + content hash) and is confirmation-first. Never offer "Clear all memory" without a scope.
- Trust is shown as a text chip with an icon; only an operator or a validating actor can raise it, and the UI must not offer a trust toggle to an agent-driven surface.
- The empty state names the scope and the read set ("No memory visible for this branch at rev 12"); a read failure renders a warning/refusal, never an empty success (`04-memory-architecture.md:481`; READ).
- Current `agent_memory` has only a role-scoped bank and an 8-row recall; until the v11 migration lands, the panel shows `Role` only and says "bank tree not available on this build". Do not fake the other scopes.

**Do not merge this with replay.** A revision is not a memory; a memory is not a checkpoint. The UI copy should say so once, in the panel subtitle.

### 3.8 Mailbox

The mailbox is the ledger for the two message verbs in `02-hmars-patterns.md`, not for delegation: **Escalate** (`ask_parent`, should block until answered or deadline; it does not today) and **Notify** (`send_to_sibling`, non-blocking, same-parent by default, cross-branch requires approval). **Delegate** (`spawn_agent`) is an inline parent→child call, not a mailbox event, and must not appear as a message.

Rows grouped by branch pair, with verb, status, and time; message text clamped to 3 lines with "expand"; no compose box; header shows `messages 7 / 32` (or "budget not reported") and "read-only · delivery is a branch event". Target schema from `02-hmars-patterns.md:365-376` adds `in_reply_to`, `status`, and `answered_at`: **Escalate** uses `open/answered/expired`; **Notify** is delivery-only and must not grow reply semantics. Until that lands, the current `AGENT_MESSAGE` has no status, so render "status not reported" and do not invent an unread count. A message that triggered an approval links to it. `AGENT_MESSAGE` appends live into a `role="log"` region. Use **oldest-first** for a conversation-like mailbox, with a "Jump to latest" control; tool ledgers are newest-first because they are logs. Do not mix the two orderings in one surface.

### 3.9 HUD and status strip

The HUD is a **status strip**, not a dashboard. Keep board 07's shape (`460×44`, top centre) and content: connection state, census (`4 SUPERAGENTS · 12 WORKERS`), and one health signal. Add at most two budget chips (`agents 12/64`, `calls 18/40`) and an approval count. Remove: "98.2 % GREEN +3.4 %" unless the server reports a defined green metric; derived values get a `derived` badge (`docs/trace/ui-rebuild/04-hmas-surface-parity.md:1119`; READ). The HUD never covers the chat at 1440; at 1280 it collapses to a single line.

### 3.10 Keyboard map (minimum)

| Key | Action | Scope |
|---|---|---|
| `Tab` / `Shift+Tab` | Move focus; skip link to `#hive-canvas` | global |
| `Arrows` | Move selection between nodes (spatial: nearest in direction) / tree rows | canvas/tree |
| `Enter` | Open inspector for selected node / expand tree row | canvas/tree |
| `Space` | Toggle selection (multi-select later) | canvas |
| `Ctrl/Cmd+0` | Recentre world origin; keep zoom | global |
| `+` / `-` / `0` / `F` | Zoom in/out, reset 100 %, fit | canvas |
| `R` | Open "Re-route…" dialog for selected Worker | canvas |
| `K` | Open kill confirmation for selected running Worker | canvas |
| `Escape` | Close topmost dialog/popover; never approves/denies | global |
| `?` | Keyboard help | global |

Rules: no drag-only action; the re-route dialog has a target listbox with the same validation as the canvas drop; destructive actions always confirm; shortcuts do not fire while a text input/textarea/contenteditable has focus.

### 3.11 Motion budget (testable)

| Motion | Duration | Trigger | Reduced motion |
|---|---|---|---|
| Card state transition | 120–160 ms opacity/border | event | 0 |
| Edge re-route draw | 200–240 ms dash offset | `NODE_REWIRE` | 0 |
| Approval arrival | 180 ms translate/opacity | `APPROVAL_REQUEST` | 0 |
| Toast | 150 ms opacity | refusal/notice | 0 |
| Camera | 0 or ≤200 ms | `Ctrl+0`/fit | 0 |
| Working bee | 1.2 s linear loop | run running | static glyph |
| Ambient sheen/scan | *none* | — | — |

Gate: `document.getAnimations()` under reduced motion = 0; under normal = ≤3 concurrent, no more than one infinite loop; no pulsing indicator for `running` (use text/static glyph). This follows Carbon's "purposeful/responsive/meticulous/unobtrusive" checklist (**VERIFIED**) and SC 2.3.3 (**VERIFIED**).

### 3.12 Error placement matrix

| Impact | Placement | Persistence | Example |
|---|---|---|---|
| Field/control error | Inline strip under the control, `role="alert"` | until corrected | `E_PROMPT_EMPTY` |
| Command refusal with a recoverable fix | Toast (max 4, dismissible) + ledger row | ledger persists | `E_FANOUT_LIMIT` |
| Approval/stale/expired | Approval card strip + inbox row | until dismissed | `E_APPROVAL_TIMEOUT` |
| Run-level failure | Run detail error card + chat error message | run record | `RUN_STATUS.error_code` |
| Transport | Banner + connection chip; no toast storm | until reconnect | `E_WS_DISCONNECTED` |
| Provider stop | Inline on the message/tool result | message record | `content_filter`, `length` |

Never: a refusal only in the console; a toast that disappears before it can be read; an error card that says "Compile failed" without a code; a spinner that never times out.

---

## 4. Prioritized UI change list (add / change / remove)

Priorities: **P0** = truth, safety, or a core pattern that is currently lying/broken; **P1** = required representation for the HMAS capability; **P2** = craft/density polish. Effort is [OPINION] and assumes the current React 19 + React Flow stack.

### 4.1 P0 — truth and safety (do first)

| # | Action | Type | Surface / seam | Evidence | Effort |
|---|---|---|---|---|---|
| P0-1 | **Remove the 5 dashboard stat tiles** (`SUB-HIVES / WORKERS / TOOLS / ERRORS / ACTIVE`) and the hero empty state with numbered steps; replace with board 08's one-line intro + hierarchy + one honest empty line | Remove | `web/src/panels/SwarmPanel.tsx:38-107` | `docs/trace/provider/05-figcraft-ux-gaps.md:92-117`, `138-151` (MEASURED there) | S |
| P0-2 | **Task identity**: stop asserting `WorkerAgent` from a defaulted kind; render role+depth+short suffix (or server label when it lands); widen the chip and use the full accessible name | Change | `store.ts` `fallbackTaskLabel`/`kind`; `web/src/panels/LibraryPanel.tsx` chips; `ControlBar` TARGET select | `docs/trace/provider/05-figcraft-ux-gaps.md:155-166` (MEASURED collisions) | M |
| P0-3 | **One tool-activity home**: keep `ToolActivityPanel` in run detail; replace the chat `ACTIONS` table with a compact "N tools running · M awaiting approval" strip linking to run detail | Remove + Add | `web/src/chat/ChatCore.tsx:273-307`; `web/src/panels/ToolActivityPanel.tsx`; board 12 | `docs/trace/provider/05-figcraft-ux-gaps.md:73-86`; W11 D7 (`plans/waves/W11-UI-TOOL-ACTIVITY.md:110`) | M |
| P0-4 | **Approval inbox + focused modal**: queue count, from→to branch narrative, args hash, TTL/nonce/consequence copy, deny-first, stale/expired read-only, no optimistic close | Add + Change | `ApprovalOverlay.tsx`; board 09/10; `plans/waves/W11-UI-TOOL-ACTIVITY.md:212-236` | W11/approval plan; `docs/trace/ui-rebuild/04-hmas-surface-parity.md:124`, `1069` | L |
| P0-5 | **Replay guard**: disabled controls **and** a store guard while `history.replayGraph !== null`; the banner says "read-only replay — no live mutations" | Change | `ControlBar.tsx`, `ChatCore.tsx`, `store.ts` | Board 10; `docs/trace/ui-rebuild/04-hmas-surface-parity.md:1020` | S |
| P0-6 | **Real project identity**: fetch `/api/projects`; show name/id honestly; remove the decorative chevron and the local random UUID as display name | Change + Remove | `LibraryPanel.tsx:29-34`; `store.ts` project init; `/api/projects` | `docs/trace/ui-rebuild/04-hmas-surface-parity.md:122`, `1013`, `1035` | M |
| P0-7 | **Remove dead/lying controls**: `Code-diff review` if still rendered; any tab that cannot change the view; "Unpin chat" if still present | Remove | `SettingsPanel.tsx`; `Header.tsx`; `App.tsx` tab scope | Board 10 (`DIFF_*` decision-gated); `docs/trace/ui-rebuild/10-representation-audit.md:41-45` | S |
| P0-8 | **Refusal is never toast-only**: every `ERROR` creates a ledger row (session-only label until a read route exists); toast shows code + `how_to_fix`; `aria-live="assertive"` for errors, polite for notices | Change | `store.ts` refusal path; `ToastRegion.tsx`; new run-detail section | `docs/contracts/UI-COMMANDS-v1.md:76-86`; board 10; NN/g error guidance | M |
| P0-9 | **Budget truth**: show `agents/calls/messages/tokens/cost/depth/fanout/in-flight` from real counters or `—`; never 0 for unknown; cap hit names the refusal; provider `finish_reason` labelled as provider, not HiveMind | Add | HUD + run detail; `backend/crates/hiverouter-core/src/caps.rs:9-15`; `backend/crates/hiverouter-core/src/execution.rs:68-70`; `backend/crates/hiverouter-provider/src/budget.rs:25-43`; usage route | READ caps and budgets; DeepSeek/Anthropic docs (VERIFIED) | M |
| P0-10 | **Stop fabricating card copy**: remove `errorCode ?? 'Compile failed'`; remove the `ctx` label until a field exists; fix the Super card's "workers" value (it is progress) | Change | `web/src/flow/nodes.tsx:136-141`, `160-162`, `202` | `docs/trace/ui-rebuild/04-hmas-surface-parity.md:1021-1023`; `docs/trace/provider/05-figcraft-ux-gaps.md:155-166` | S |

### 4.2 P1 — required HMAS representation

| # | Action | Type | Surface / seam | Evidence | Effort |
|---|---|---|---|---|---|
| P1-1 | **Hive / Tree toggle** in Swarm, sharing selection and filters; Tree is an APG treegrid with root summary, expand/collapse, parent column, honest `—` | Add | `SwarmPanel.tsx`; new `AgentTree.tsx`; `AgentHierarchy.tsx` | APG treegrid (VERIFIED); board 08 | L |
| P1-2 | **LOD bands + culling + world-anchored comb**; nodeTypes per band; batched SVG B0; hysteresis; no layout on selection | Add | `Canvas.tsx`, `graph.ts`, `HiveField.tsx` | `docs/trace/ui-rebuild/07-ui-stack.md:118-164`, `189-206` | L |
| P1-3 | **Card anatomy v2**: tier by geometry/surface, role chip, state dot+glyph+text, 4-fact limit; `aria-pressed` cards; no nested inline buttons | Change | `web/src/flow/nodes.tsx`; board 05 | Boards 05/08; WCAG 1.4.1 | M |
| P1-4 | **Root summary + per-row meta** in hierarchy: `root · N superagents · M workers · state`; Super row `N workers · M running`; Worker `role · progress · retry` only if reported | Add | `AgentHierarchy.tsx` / Tree | Board 08; `docs/trace/ui-rebuild/04-hmas-surface-parity.md:178-191` | M |
| P1-5 | **Tool ledger details**: branch/turn grouping, outcome legend (`ok/refused/denied/timeout/cancelled/requested/running`), code+`how_to_fix`, `args_sha256 → result sha256/bytes`, filters, 200-cap footer | Change | `ToolActivityPanel.tsx`; `store.ts` tool activity | W11 §6.2; board 12 | M |
| P1-6 | **Approval ledger + queue** (decided/expired/stale rows; nonce consumed; link to tool result; session-only label) | Add | Run detail approvals section | Board 09; W11 D8-D12 | M |
| P1-7 | **Memory inspector**: five bank scopes (project/run/hive/branch/role), trust chips, `as_of_rev`, read-set + 12-entry/4,000-char truncation, promote-one-level and purge-with-receipt; no free-text editor | Add/Change | `AgentMemoryPanel.tsx`; board 09 | `docs/research/hmars/04-memory-architecture.md:92-102`, `449-527`; board 09 | M |
| P1-8 | **Mailbox live + budget**: `AGENT_MESSAGE` appends; total count; "unread not reported"; 32-cap; branch grouping; verb + status (Escalate `open/answered/expired`; Notify delivery-only) only when the schema lands; no compose | Change | `store.applyEvent`; `AgentMessagesPanel.tsx`; board 09 | `docs/trace/ui-rebuild/04-hmas-surface-parity.md:1028`, `1053-1054`; `docs/research/hmars/02-hmars-patterns.md:365-376` | M |
| P1-9 | **State legend** (shape + text) in Swarm; remove any colour-only state cue; 16-state mapping documented in the legend's "show all states" | Add | Swarm; `agentState.ts` | Board 08; WCAG 1.4.1 | S |
| P1-10 | **Minimap content + fallback**: nodes as tier/state rects, viewport outline; hide below 1280 with an "Overview" fit button | Change | `Canvas.tsx` minimap; `MiniMap.tsx` | `docs/trace/provider/05-figcraft-ux-gaps.md:167` | S |
| P1-11 | **Keyboard and focus**: 24 px minimum targets; 40 px primary; ink-700 2 px focus ring; re-route dialog as keyboard alternative to drag; `role="application"` only if full key model | Change | `index.css`, `Canvas.tsx`, `ControlBar.tsx` | WCAG 2.5.8/2.4.13 (VERIFIED); `docs/trace/A11Y-CONTRAST-PLAN.md` | M |
| P1-12 | **Motion budget**: cut to one ambient loop, event-driven transitions, zero under reduced motion; test `getAnimations()` | Remove + Change | `styles/motion.css`; `HiveField.tsx`; `BeeEdge.tsx` | `docs/trace/provider/05-figcraft-ux-gaps.md:127-136`; Carbon motion (VERIFIED) | M |

### 4.3 P2 — craft and density

| # | Action | Type | Surface |
|---|---|---|---|
| P2-1 | Scrubber: wall-clock time + tick marks + `routing state N of M`; keep LIVE exit | Change | `Scrubber.tsx` |
| P2-2 | Meta floor 9.5 px: replace 8 px/8.5 px/9 px labels (14 occurrences) | Change | `styles`, panels |
| P2-3 | Replace text glyph icons (`◆`, `└`, `▲`, `×`) with one line-icon set; keep text labels | Change | `NavIcon`, cards |
| P2-4 | Model panel: six-counter 2×2 grid, key status, server health, balance `as of`, independent error states, no wrapped model rows | Change | `ModelsPanel.tsx`; board 12 |
| P2-5 | Completion review: `finish_reason` chips (`length`, `content_filter`, `insufficient_system_resource`, `aborted`) on the message/run | Add | Chat, run detail |
| P2-6 | Filters: `hide finished`, `focus branch` promoted from local toggles to a filter bar with counts; persist per run | Change | `Canvas.tsx` |
| P2-7 | Density toggle (compact/comfortable) for Tree and ledger; persisted; targets stay ≥24 px | Add | Tree/ledger |
| P2-8 | Copy pass: replace UUID primary copy with `role + short id`; "memory" vs "revision" language; "provider stopped" vs "refused" | Change | Global strings |

### 4.4 What to remove (explicit)

Removing the wrong thing is as important as adding the right one. [OPINION] Remove, do not restyle:

1. **5 stat tiles on Swarm** — duplicate the HUD, generic dashboard, not on board 08.
2. **hero empty state + numbered onboarding steps** — pattern-slop; boards use one honest line.
3. **The duplicate tool-activity table in chat** (or the panel, but not both).
4. **Global library tabs** that only affect Workspace (or scope them visually and behaviourally).
5. **Dead code-diff button** if present; `DIFF_*` is decision-gated.
6. **Fake project chevron / random UUID as project name**; fetch `/api/projects`.
7. **`Compile failed` default** and any other invented error cause.
8. **`WorkerAgent` asserted from a defaulted kind.**
9. **8 px micro-labels** (below the design's own 9.5 px meta floor).
10. **Three ambient loops + free-flying bees**; keep at most one ambient and static directional bees.
11. **"98.2 % GREEN +3.4 %"** unless the server defines and reports the metric; otherwise derived+label or remove.
12. **UUID primary copy** in Projects/Agents/Approvals; use human label + short id.
13. **Notification bell / operator menu / project create / diff viewer / unread badge** — do not add; no domain/route exists (`docs/trace/ui-rebuild/10-representation-audit.md:66-74`; READ).
14. **The old `docs/ui/figcraft/C` board set** as a design reference (superseded; evidence only).

### 4.5 Acceptance checks for the change set

- **Truth**: every visible metric has a named source (event/HTTP/derived+label); no `—` becomes `0`; no invented label/code/score.
- **One projection, two views**: Tree and Hive select the same task and show the same state; changing one does not desync the other.
- **No lying controls**: every enabled mutation calls a frozen command or real HTTP route and waits for ACK/2xx; replay disables all mutations at the control **and** store level.
- **A11Y**: keyboard-only run of submit → select → re-route dialog → approval → replay exit; axe serious = 0; focus ring ≥3:1; no target <24 px; treegrid key map works; reduced motion leaves 0 animations.
- **Density**: at 1440, the workspace shows one primary canvas + chat + status strip; no scrollbars on the page itself; the inspector overlays rather than reflows the canvas.
- **Performance at the real cap**: 64 nodes at 0.1× and 3.0×; selection does not re-run layout; no frame drop on select (the current dagre-in-render defect is `docs/trace/ui-rebuild/07-ui-stack.md:151-164`). At the HMARS target, hold `03-scale-concurrency.md:446-455` as the gate: ≥50 fps at 600 nodes, ≤120 mounted nodes, ≤50 animated edges, ≤300 KB snapshot.
- **Evidence**: screenshot/DOM capture per state and the command/test that produced it, attached to the change, not prose.

---

## 5. Twenty questions and answers

**Q1. Should HiveMind replace the spatial canvas with a conventional dashboard?**
No. [OPINION] The canvas is the product's identity and its strongest comprehension aid at ≤64 nodes; boards 05/07/08 are built around it. What should be removed is the *dashboard layer* that has accumulated around the canvas (stat tiles, hero empty states, duplicate tables). Keep the map, delete the lobby. Evidence: board 07 composition; 64-node cap (`backend/crates/hiverouter-core/src/caps.rs:9-15`); current drift (`docs/trace/provider/05-figcraft-ux-gaps.md:92-151`).

**Q2. Canvas only, or canvas plus tree?**
Both, from one dataset. [OPINION] A canvas is poor for long labels, screen readers, and deep inspection; a tree is poor for spatial relationships and branch shape. Langfuse ships Graph/Tree/Timeline as peers (**VERIFIED**, [Agent Graphs](https://langfuse.com/docs/observability/features/agent-graphs)), and the APG treegrid gives a tested keyboard model (**VERIFIED**, [Treegrid](https://www.w3.org/WAI/ARIA/apg/patterns/treegrid/)). Make Hive/Tree a first-class toggle, not a settings checkbox.

**Q3. What should the canvas show at 0.1× versus 3.0×?**
A five-band LOD with explicit thresholds and ±0.04 hysteresis: B0 clusters/silhouettes, B1 header + state, B2 full card, B3 detail, B4 evidence. [OPINION] The exact thresholds are design decisions, but the *existence* of bands is not: 64 full cards today (600 at the HMARS target) at 0.1× are unreadable, and the macro view is a stated design goal (board 07 / prompt 9; `docs/research/hmars/03-scale-concurrency.md:436-444`; READ). Langfuse's deterministic layout is evidence that stable spatial memory matters (**VERIFIED**, [changelog](https://langfuse.com/changelog/2026-07-13-graph-view-modes)).

**Q4. How do we keep layout stable?**
One layout per topology change, memoised by a stable key; never on selection or drag. The repo's proposed lattice (root at world origin, 12 slots, 440/381.05 pitch) is the right owner for geometry; React Flow is the transform host, not the graph authority (`docs/trace/ui-rebuild/07-ui-stack.md:344-379`; READ). Determinism is a usability property: users build a mental map, and a re-shuffle destroys it. Langfuse explicitly keeps the same layout every time and preserves the viewport (**VERIFIED**).

**Q5. How should Hyper/Super/Worker/Tool tiers be differentiated?**
By geometry, surface, header, and text — not by a second hue scale. Board 05 already uses size, header fill, and surface (honey header for Super, flat for Worker, dark code surface for Tool, crimson border for Error), and reserves colour for state. WCAG 1.4.1 forbids colour as the only distinction (**VERIFIED**). [OPINION] Tier = size/containment + a mono role chip; state = dot+glyph+text; selection = ring. This survives greyscale and low zoom.

**Q6. How many facts can a card show?**
At most four at B2: identity/name, tier/role, state, one live metric. [OPINION] A 320 px Worker card with name + role + state + progress + ctx + score + retry + model is noise; the inspector exists for the rest. This is also an honesty rule: every extra fact is an opportunity to fabricate when the producer does not send it (`docs/trace/ui-rebuild/04-hmas-surface-parity.md:1149-1163`).

**Q7. What do we do when the server does not send label/progress/ctx?**
Render an honest unknown, never a plausible default. The contract already lacks `label`, `progress`, `error_code`, `role`, `template_id`, `score`, `ctx` on `NODE_*` (`docs/trace/ui-rebuild/04-hmas-surface-parity.md:154-168`, `1046-1060`; READ). [OPINION] Use `role · depth · short-suffix` as identity, `—` for metrics, and neutral "error state" instead of "Compile failed"; add additive producer fields before adding richer cards. The UI must not become a second source of truth to fill the gap.

**Q8. Where do live tool calls belong?**
Run detail, with a compact live strip in chat and a signpost on the canvas tool card. [OPINION] Board 12 puts `TOOL ACTIVITY` in run detail; W11 chose the Agents panel for v1.2 (`plans/waves/W11-UI-TOOL-ACTIVITY.md:110`; READ). The current duplicate (chat `ACTIONS` + panel) is the worst option (`docs/trace/provider/05-figcraft-ux-gaps.md:73-86`). One ledger, three levels of detail, one source of truth.

**Q9. What does a tool-call row need to be trustworthy?**
Turn, tool, branch/task, outcome, code (if any), `args_sha256`, result `bytes`/`sha256`, and time. Outcome is a closed enum with text + shape; refused/denied/timeout/cancelled always show a code and `how_to_fix` where known. [OPINION] Do not render a result body unless the server sends one; hash+bytes is evidence, invented text is not. W11 already specifies the reducer, cap, dedupe, and `role="log"` semantics (`plans/waves/W11-UI-TOOL-ACTIVITY.md:202-256`; READ).

**Q10. Should approvals be modal, toast, or inbox?**
Inbox + modal. [OPINION] A toast is not a decision surface; a single modal per event loses the queue and the history. Board 09 shows a `MESSAGE APPROVAL` card with from→to, hash, TTL and consequence; board 10 shows the named-refusal card. OpenAI's HITL model is run-wide and can carry multiple interruptions (**VERIFIED**, [HITL](https://openai.github.io/openai-agents-python/human_in_the_loop/)). So: a persistent queue, a focused `alertdialog`, and a decided/expired ledger.

**Q11. What evidence must an approval show before Approve is safe?**
Who is asking whom, what tool, what arguments are bound (`args_sha256`), what the consequence is, and when the grant expires. [OPINION] Optional server-redacted preview only if it lands with the contract patch; never reconstruct args client-side. W11's deny-first focus, Escape-never-resolves, and nonce/TTL copy are the baseline (`plans/waves/W11-UI-TOOL-ACTIVITY.md:111-116`, `212-236`; `plans/foresight/APPROVAL-UX-PLAN.md:127-171`; READ). OpenAI's callable approval rules fail closed on unparseable arguments — a good principle to mirror in copy and in tests (**VERIFIED**).

**Q12. How should branch memory be represented versus replay?**
Two surfaces, never merged. Replay is run state (checkpoints/revisions), read-only. Memory is durable, scoped data with provenance and an audit trail. LangGraph explicitly separates checkpointers (thread state) from Stores (cross-thread) (**VERIFIED**, [Persistence](https://raw.githubusercontent.com/langchain-ai/docs/main/src/oss/langgraph/persistence.mdx)). HiveMind's `agent_memory` is role-scoped, latest 8, and has no editor/clear UI yet (`docs/CODEMAP/18-HMAS-RESEARCH.md:145-152`; READ). The target model is the bank tree in `04-memory-architecture.md`: project/run/hive/branch/agent-role banks, snapshot reads at `as_of_rev`, one-level promotion, trust levels, a hash-chained journal and purge receipts (`docs/research/hmars/04-memory-architecture.md:92-102`, `449-527`; READ). [OPINION] Build the inspector with those five scopes, trust, and truncation; do not add a free-text editor until a write route and consent model exist, and do not present agent-written text as authoritative.

**Q13. Should the mailbox look like chat?**
No. [OPINION] A mailbox is a ledger of the two message verbs — **Escalate** (`ask_parent`) and **Notify** (`send_to_sibling`) — not of **Delegate** (`spawn_agent`), which is an inline parent→child call (`docs/research/hmars/02-hmars-patterns.md:347-380`; READ). Rows carry from→to, verb, status (Escalate `open/answered/expired`; Notify delivery-only), time, task/context, clamped text, and a link to the approval. A chat transcript implies a conversation with one interlocutor and encourages a compose box the backend cannot honor. A2A's `contextId`/task/artifact model is the right mental model (**VERIFIED**, [Life of a Task](https://a2a-protocol.org/latest/topics/life-of-a-task/)); AutoGen Studio's live message flow is a visualization, not a chat log (**VERIFIED**). Keep it read-only until a send route exists, and never fabricate an unread count.

**Q14. How should run budgets be shown without fake precision?**
Seven meters with real sources: agents/64, calls/40, messages/32, tokens/400k, cost/$0.50 (settled only), depth/3, max fan-out/8; the global in-flight/8 is a status chip, not a run meter. Unknown = `—`/`not reported`, never 0. [OPINION] Put the full strip in run detail, mirror only the two urgent meters in the HUD, warn at 80 % with text+icon, and make a cap hit name the refusal. OTel's agent metrics define the counting discipline (nested calls counted once) (**VERIFIED**, [gen-ai-metrics.md](https://github.com/open-telemetry/semantic-conventions-genai/blob/main/docs/gen-ai/gen-ai-metrics.md)); HiveMind's caps and budgets are engine values, not provider limits (`backend/crates/hiverouter-core/src/caps.rs:9-15`; `backend/crates/hiverouter-provider/src/budget.rs:25-43`; READ).

**Q15. How do we distinguish refusal vs provider stop vs truncation?**
Three classes, three visuals. HiveMind refusal = stable `E_*` code + message + `how_to_fix`, ledger row. Provider stop = `finish_reason` (`length`, `content_filter`, `insufficient_system_resource`, `aborted` for DeepSeek; `refusal`, `max_tokens`, `model_context_window_exceeded`, `pause_turn` for Anthropic) rendered as "Stopped/Truncated: <reason>", not as an error card. Tool outcome = closed enum chip. [OPINION] This prevents the two worst failures: treating a content filter as a bug, and treating a refusal as a silent success. Sources: DeepSeek (**VERIFIED**), Anthropic (**VERIFIED**), `docs/contracts/UI-COMMANDS-v1.md:76-86`.

**Q16. How do we prevent the page becoming a generic dashboard?**
Enforce the density budget: one primary region + one inspector + one status strip; no stat-tile row; no hero empty state; no more than two side panels; each card ≤4 facts. [OPINION] Ground every new element in a board or a contract fact; if it cannot cite one, it is probably "SaaS wallpaper". Carbon's spacing guidance and Shneiderman's overview-first/zoom-filter/details-on-demand give the positive rule (**VERIFIED**); the current Swarm drift gives the negative example (`docs/trace/provider/05-figcraft-ux-gaps.md:138-151`).

**Q17. How do we make a spatial canvas accessible?**
Provide a DOM treegrid peer, a full keyboard model, and `role="application"` only if that model is implemented. Cards are real focusable controls with accessible names (`label (kind, state)`), edges are focusable and named, a skip link targets `#hive-canvas`, and live updates use `role="log"`/`aria-live` for state transitions, not tokens. [OPINION] Drag is an accelerator, never the only rewire path. WCAG 2.2 target size 24×24 AA and focus appearance 2 px/3:1 AAA are the hard floors (**VERIFIED**); APG treegrid/feed are the patterns (**VERIFIED**).

**Q18. What keyboard model should the canvas use?**
Roving focus across nodes, arrows move spatially (nearest node in direction), Enter opens the inspector, Space toggles selection, `Ctrl/Cmd+0` recentres, `R` opens the re-route dialog, `K` opens kill confirm, Escape closes the topmost layer and never decides. [OPINION] This mirrors the treegrid where possible so users learn one model; the spatial arrows are an enhancement, not a replacement for Tab/arrow row navigation. All destructive shortcuts require a dialog; no shortcut fires inside a text field.

**Q19. How much motion is acceptable?**
At most one ambient loop and ≤3 concurrent animations, all event-driven and all disabled under reduced motion. Current build: 12 concurrent animations, three ambient loops plus free-flying bees (`docs/trace/provider/05-figcraft-ux-gaps.md:127-136`; READ/MEASURED there). [OPINION] Motion should communicate a state change (card transition, edge re-route, approval arrival), not simulate a living hive. Carbon's productive/expressive split and duration tokens are the reference (**VERIFIED**); WCAG 2.3.3 and `prefers-reduced-motion` are the floor (**VERIFIED**).

**Q20. What should we remove first, and what should we not add?**
Remove first: the Swarm stat tiles and hero empty state, the duplicate tool table, dead/lying controls, `Compile failed`/`WorkerAgent` assertions, and the ambient motion excess (§4.4). Do not add: a notification bell, operator menu, project create/rename, diff viewer, fake unread badge, a second tool table, a WebGL/Pixi rewrite justified by node count alone, or auto-approve/sticky approvals. [OPINION] Each of those is either a false affordance (no domain/route) or a duplication of an existing source of truth (`docs/trace/ui-rebuild/10-representation-audit.md:66-74`; READ). The product gets better by subtraction here.

---

## 6. Suggested implementation sequence

[OPINION] Sequence by dependency and risk, not by visual appeal:

1. **Truth fixes** (P0-2, P0-6, P0-7, P0-10): identity, projects, dead controls, fabricated copy. These are small and stop the UI from lying while the bigger work lands.
2. **One home for evidence** (P0-3, P1-5, P0-8): move tool activity to run detail, add the refusal ledger, delete the chat table.
3. **Topology peer** (P1-1, P1-4, P1-9): Tree + root summary + legend; this unlocks accessibility and gives the canvas a stable reference.
4. **LOD + layout** (P1-2, P1-3, P1-10): banding, culling, world-anchored comb, card v2, minimap. Do this before motion polish; a stable map matters more than a lively one.
5. **Decision surfaces** (P0-4, P1-6, P1-7, P1-8, P0-9): approval inbox/ledger, memory inspector, mailbox live, budget strip. These need contract/producer work; keep the UI honest about what is missing.
6. **Accessibility + motion** (P1-11, P1-12, P2): keyboard/focus/targets, then the motion budget. Run the a11y gate after each step, not at the end.
7. **Craft** (P2): scrubber ticks, meta floor, icons, model panel, copy.

Do not start with a visual restyle. The largest user-visible gains in this list are deletions and one honest ledger.

---

## 7. Sources (primary, fetched 2026-11-07)

| # | Source | What it supports in this document | Class |
|---|---|---|---|
| S1 | [Langfuse — Agent Graphs](https://langfuse.com/docs/observability/features/agent-graphs) | Aggregated vs Expanded; graph next to Tree/Timeline; inferred from observations | VERIFIED |
| S2 | [Langfuse — Graph view modes changelog](https://langfuse.com/changelog/2026-07-13-graph-view-modes) | Deterministic ELK layout; viewport stays put; same layout every time | VERIFIED |
| S3 | [LangGraph Studio (doc source)](https://raw.githubusercontent.com/langchain-ai/langgraphjs/c9e96aa500ac6c6c3410dd45bdf02bc9c1738503/docs/docs/concepts/langgraph_studio.md) | Graph visualization, state edit/rerun, threads, memory, interrupts | VERIFIED |
| S4 | [LangGraph — Time travel](https://raw.githubusercontent.com/langchain-ai/docs/main/src/oss/langgraph/use-time-travel.mdx) | Replay re-executes; fork; subgraph checkpoint granularity | VERIFIED |
| S5 | [LangGraph — Persistence](https://raw.githubusercontent.com/langchain-ai/docs/main/src/oss/langgraph/persistence.mdx) | Checkpointer vs Store; in-memory saver dies on restart | VERIFIED |
| S6 | [LangGraph — Interrupts](https://raw.githubusercontent.com/langchain-ai/docs/main/src/oss/langgraph/interrupts.mdx) | Pause/resume; node restarts; idempotency before interrupt; typed schemas | VERIFIED |
| S7 | [OpenAI Agents SDK — Tracing](https://openai.github.io/openai-agents-python/tracing/) | Trace/span hierarchy; span types; sensitive-data redaction | VERIFIED |
| S8 | [OpenAI Agents SDK — Human-in-the-loop](https://openai.github.io/openai-agents-python/human_in_the_loop/) | Run-wide approvals incl. nested; interruptions; sticky decisions; fail-closed | VERIFIED |
| S9 | [OpenAI Agents SDK — Agent visualization](https://openai.github.io/openai-agents-python/visualization/) | Shape/edge vocabulary; object identity over display name | VERIFIED |
| S10 | [A2A — Life of a Task](https://a2a-protocol.org/latest/topics/life-of-a-task/) | Message vs Task; contextId; task states; immutability; artifact versioning | VERIFIED |
| S11 | [Temporal — Events and Event History](https://docs.temporal.io/workflow-execution/event) | Append-only history; activity retry/timeout events; limits; principal attribution | VERIFIED |
| S12 | [Apache Airflow — UI Overview](https://airflow.apache.org/docs/apache-airflow/stable/ui.html) | Grid vs Graph vs Runs/Tasks/Events tabs; dense-but-structured matrix | VERIFIED |
| S13 | [AutoGen Studio](https://microsoft.github.io/autogen/stable/user-guide/autogenstudio-user-guide/index.html) | Team Builder; Playground live message flow; prototype disclaimer | VERIFIED |
| S14 | [Carbon — Motion](https://www.carbondesignsystem.com/building-blocks/foundations/motion/overview) | Productive/expressive; easing; duration tokens; evaluation checklist | VERIFIED |
| S15 | [Carbon — Spacing](https://www.carbondesignsystem.com/building-blocks/foundations/spacing/overview) | Proximity→relationship; whitespace; "dense sections, uncrowded page" | VERIFIED |
| S16 | [NN/g — Progressive Disclosure](https://www.nngroup.com/articles/progressive-disclosure/) | Two-level disclosure; clear progression; information scent | VERIFIED |
| S17 | [NN/g — Error-Message Guidelines](https://www.nngroup.com/articles/error-message-guidelines/) | Proximity; redundant cues; human language; constructive advice; preserve input | VERIFIED |
| S18 | [WAI-ARIA APG — Treegrid](https://www.w3.org/WAI/ARIA/apg/patterns/treegrid/) | Hierarchical rows/cells; `aria-expanded`; full keyboard model | VERIFIED |
| S19 | [WAI-ARIA APG — Feed](https://www.w3.org/WAI/ARIA/apg/patterns/feed/) | Dynamic log/feed semantics; `aria-posinset`/`aria-busy` | VERIFIED |
| S20 | [WCAG 2.2 — Use of Color](https://www.w3.org/WAI/WCAG22/Understanding/use-of-color.html) | Colour never the only visual means | VERIFIED |
| S21 | [WCAG 2.2 — Target Size (Minimum)](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html) | 24×24 CSS px AA floor; spacing exception | VERIFIED |
| S22 | [WCAG 2.2 — Focus Appearance](https://www.w3.org/WAI/WCAG22/Understanding/focus-appearance.html) | 2 px perimeter / 3:1 change of contrast | VERIFIED |
| S23 | [WCAG 2.2 — Animation from Interactions](https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions.html) | Disable non-essential interaction motion | VERIFIED |
| S24 | [MDN — `prefers-reduced-motion`](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@media/prefers-reduced-motion) | Vestibular triggers; scaling/panning large objects | VERIFIED |
| S25 | [DeepSeek — Chat Completions API](https://api-docs.deepseek.com/api/create-chat-completion) | `finish_reason` enum; `max_tokens` range; usage fields | VERIFIED |
| S26 | [Anthropic — Stop reasons and fallback](https://platform.claude.com/docs/en/build-with-claude/handling-stop-reasons) | `refusal`, `max_tokens`, `model_context_window_exceeded`, `pause_turn` | VERIFIED |
| S27 | [OpenTelemetry — GenAI metrics](https://github.com/open-telemetry/semantic-conventions-genai/blob/main/docs/gen-ai/gen-ai-metrics.md) | Agent duration/inference_calls/tool_calls; nested counted once; `error.type`; `deepseek` | VERIFIED |
| S28 | [data.europa.eu — The information-seeking mantra](https://data.europa.eu/apps/data-visualisation-guide/the-information-seeking-mantra) | Shneiderman "overview first, zoom and filter, details on demand" + attribution | VERIFIED |
| S29 | [React Flow — Accessibility](https://reactflow.dev/learn/advanced-use/accessibility) / [Performance](https://reactflow.dev/learn/advanced-use/performance) | Library-level reference for the shipped renderer (page content JS-truncated; implementation claims are taken from repo evidence) | VERIFIED (URL), READ (repo) |
| S30 | [Langfuse — Observation types](https://langfuse.com/docs/observability/features/observation-types) | Type vocabulary for tool/agent/generation/guardrail nodes and filters | VERIFIED |

**Correction to the repo's claims register.** `AGENTS.md` §13 marks the 393,216 output ceiling as UNVERIFIED. As of 2026-11-07, the official DeepSeek Chat Completions page states `max_tokens` "must be between 1 and 384K (393216)" (**VERIFIED**, S25); sibling `01-deepseek-api.md:181` records the same 1–384K bound, so the two seats agree. This does **not** authorize inventing a token budget: HiveMind already enforces a 400,000 combined-token cap and a $0.50 cost cap (`backend/crates/hiverouter-provider/src/budget.rs:25-28`; READ), and whether those numbers move is a product decision. It does mean the provider limit may be cited with the URL and date, instead of being repeated as unverified.

**Repository sources used (READ, not re-listed as web sources):** `docs/contracts/UI-COMMANDS-v1.md`; `backend/crates/hiverouter-core/src/caps.rs`; `backend/crates/hiverouter-core/src/execution.rs`; `backend/crates/hiverouter-provider/src/budget.rs`; `web/src/tokens/tokens.ts`; `web/src/flow/nodes.tsx`; `docs/ui/figcraft/HiveMind/README.md`; boards 05–12; `docs/trace/ui-rebuild/04-hmas-surface-parity.md`; `docs/trace/ui-rebuild/07-ui-stack.md`; `docs/trace/ui-rebuild/10-representation-audit.md`; `docs/trace/provider/05-figcraft-ux-gaps.md`; `docs/trace/A11Y-CONTRAST-PLAN.md`; `plans/waves/W11-UI-TOOL-ACTIVITY.md`; `plans/foresight/APPROVAL-UX-PLAN.md`; sibling HMARS files `01`–`04`.

---

## 8. Open decisions and unverified items

| # | Item | Why it matters | What would settle it |
|---|---|---|---|
| U1 | Does the producer team accept additive `label`, `progress`, `error_code`, `role`/`template_id` on `NODE_*`? | Without them, card v2 can only show `—` and role+depth identity; the design's named agents remain impossible | A contract-patch row + producer implementation + store mapping; W11/04 already list the seam (`docs/trace/ui-rebuild/04-hmas-surface-parity.md:209-220`) |
| U2 | Which surface owns tool activity for v1.2: run detail (board 12) or Agents (W11 D7)? | Duplicate tables are the current defect; the decision changes the IA | Product sign-off; W11 D7 is the current written decision (Agents panel), board 12 says run detail |
| U3 | Is there a read route for the approval/refusal ledger? | Without it, the "ledger" is session-only and must be labelled so | Backend endpoint + read model; otherwise label session-only |
| U4 | Does `RUN_METRICS.green_pct` have a defined server formula? | The HUD's "98.2 % GREEN" is otherwise an unexplained number | Metric definition + test; until then derive and label, or remove |
| U5 | Which zoom-band thresholds and durations are authoritative? | Board 11 is explicitly PROPOSED, not measured; LOD thresholds are this document's opinion | A design sign-off board + perf capture at 64 nodes today and 600 at the HMARS target |
| U6 | Can the Tree and canvas share selection without re-render storms? | The current node identity churn is a known defect (`docs/trace/ui-rebuild/07-ui-stack.md:151-164`); a second projection can amplify it | Selector rules (S-1..S-5) + memoized projection + frame trace |
| U7 | Does A2A-style `contextId` exist in HiveMind? | The mailbox grouping in §3.8 assumes run/task; a context id would allow cross-run grouping | Contract/schema read; if absent, group by run + branch pair only |
| U8 | Is browser e2e available for the a11y/motion gates? | The repo's A11Y plan is blocked on Playwright/axe decisions (`docs/trace/A11Y-CONTRAST-PLAN.md:48-63`) | Lead approval for the pinned tooling + a fake-provider fixture |
| U9 | Does the v11 memory-bank migration land before the memory inspector? | The five-scope inspector is unbuildable on the current role-only `agent_memory`; without it the UI must show `Role` only | Migration v11 + read endpoints per `04-memory-architecture.md:92-102`, `531-533` |
| U10 | Does `agent_messages` gain `in_reply_to`/`status`/`answered_at`? | Without it the mailbox cannot show `open/answered/expired`, only text; an unread badge stays impossible | Schema patch per `02-hmars-patterns.md:365-376` + event-field additive change |

**Bottom line.** [OPINION] HiveMind does not need a new visual language; it needs **fewer, more honest projections of the same graph**. Keep the hive canvas and the cards, add a treegrid peer, move evidence into one run-detail ledger, turn approvals into a queue with a consequence, show budgets as meters with unknowns, delete the dashboard wallpaper, and make motion event-driven. The result is still unmistakably HiveMind — and it is defensible to an operator, an auditor, and a screen-reader user.


