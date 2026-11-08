# HiveMind HMARS — synthesis, truth map, and goal

Date: 2026-11-07. Source HEAD: `bd575ba`.
This file is the Lead's synthesis of the six web research lanes and two codebase-analysis
lanes. It is deliberately harsher than a pitch: a thing is "have" only when the code path and
the UI/route are both present.

Research corpus:

- `01-deepseek-api.md` — DeepSeek surface, limits, billing, gaps.
- `02-hmars-patterns.md` — HMARS/reference patterns, invariants, failure modes.
- `03-scale-concurrency.md` — provider and local ceilings, S0–S5 scale plan.
- `04-memory-architecture.md` — bank tree, migration, policy.
- `05-ui-ux-patterns.md` — projection/LOD/card/approval/memory recommendations.
- `06-security-cost.md` — 26 threats, cost model, error mapping.
- `07-code-backend.md` — backend ownership, caps, A2A, memory, gaps.
- `08-code-ui.md` — UI truth, routes, controls, dead/missing surfaces.

## 0. The one-paragraph truth

HiveMind **is** a real hierarchical multi-agent runtime with a Rust execution engine, a planner
root, streaming DeepSeek calls, a tool loop, persisted runs/messages, and a distinctive spatial
UI. It is **not yet** the HMARS the operator describes: the HyperAgent is a planner row rather
than a continuously executing orchestrator; SuperAgents have a persona/template now but no own
memory bank; Subagents are a Worker-role alias; `ask_parent` is structurally one-way; there is no
roster or agent bus; memory is role-global with 8 rows; the engine is serial with an 8-permit
provider limiter and a 64-agent cap; and the UI cannot show the hierarchy, memory banks, A2A, or
approvals truthfully. The architecture is sound enough to extend; the gaps are at the edges of
the primitives, not a reason to restart.

## 1. Twenty questions, answered

Status legend: **V** verified from source/runtime, **R** read in a research report/source,
**U** unverified, **F** false/contradicted.

1. **What is HMARS here?**
   HiveMind Multi-Agent Runtime System: HyperAgent → SuperAgents (hives/branches) → Agents/
   Workers → Subagents, plus Remote/Watchdog agents that are simple switches. It is a tiered
   runtime, not a mesh and not a single chat agent. **Target, not current.**

2. **What actually orchestrates a run?**
   `RunEngine`: planner call → validated plan → root task + spawned tasks → sequential dispatch
   → aggregation → terminal snapshot. The HyperAgent is the root task (`role="planner"`,
   depth 0), mapped to `AgentTier::HyperAgent`; the actual planning prompt is separate.
   `execution.rs:1321-1335`, `provider_executor.rs:286-313`, `planner.rs:143-193`. **V**

3. **Is the HyperAgent an executing agent?**
   Not in the strong sense. It owns the run and is published as `kind=hyperagent`, but only
   planned leaf/aggregator tasks and the spawn queue are executed. The chat card is its visual
   anchor, not a node that takes tool actions. `execution.rs:97-106,764-779`. **V**

4. **Is there a SuperAgent?**
   Yes as a tier, spawn policy, allowlist, persona, and prompt. `superagent.prompty` now ships;
   `personas.json` has id `superagent`; it may spawn workers/subagents and read files.
   `execution.rs:108-142`, `tool.rs:391-430`. **V** (runtime use still **U**).

5. **Is there a Subagent tier?**
   No. `AgentTier` is Hyper/Super/Worker/Tool. `subagent` is a spawn role that maps to
   `WorkerAgent`; a `subagent` persona exists now, but there is no tier enum, no distinct
   allowlist, no memory bank, and no card. `execution.rs:151-163`, `event.rs:240`. **V**

6. **Do persona prompt bodies reach the model?**
   The first leaf call now uses a rendered per-role system prompt; the tool-turn continuation
   still hard-codes `TASK_SYSTEM`, and the planner uses `PLANNER_SYSTEM`, not the planner
   persona. UI-spawned children use Title-Case roles while the persona map is lowercase-id keyed.
   `provider_executor.rs:82-87,315-348,357-372`; `store.ts:1865-1871`. **V**

7. **Are the shipped worker personas reachable from the planner?**
   No. `validate_spawn_role` permits `worker`/`superagent`, not `content-writer`,
   `data-analyst`, `code-reviewer`, `chatbot`; those are reachable only through
   `TEMPLATE_INSERT`, which sets `role=template_id` and bypasses the validator. `execution.rs:113-142`;
   `ws.rs:200-293`. **V**

8. **Is the chat powered by the DeepSeek API?**
   Yes. ChatCore sends `SUBMIT_PROMPT`; the engine plans and streams planner/leaf Chat
   Completions; assistant deltas, tool calls, results and terminal status return through the WS
   store. Live proof exists (`HIVE_UI_OK`, `FILE_READ OK`). **V**

9. **Is the chat a SuperAgent?**
   No. ChatCore is operator UI. There is no conversational sub-orchestrator under the
   HyperAgent. The root HyperAgent/planner owns the run. **V**

10. **How do agents communicate?**
    `ask_parent` and `send_to_sibling` persist `agent_messages` and publish `AGENT_MESSAGE`.
    Cross-parent messaging has a separate guard/approval path. There is no global agent bus,
    topic bus, or roster discovery. `execution.rs:2417-2609`, `db.rs:159-177`. **V**

11. **Does `ask_parent` actually work as a question/answer?**
    No. It inserts and publishes a one-way message; no engine path reads the parent's answer and
    injects it into the child task. The only consumer is the HTTP mailbox/UI. A child asking a
    question can only guess. This is the highest-severity A2A defect. `execution.rs:2465-2479`;
    `http.rs:674-702`. **V**

12. **Is there a roster / do agents know who exists?**
    No. No route, table, or prompt context exposes task ids/roles. `send_to_sibling` requires an
    exact `TaskId` the model cannot discover. **V**

13. **What memory exists?**
    A `ScopedContext` per task, seeded from the parent's facts at creation (snapshot), plus an
    `agent_memory` table keyed by role, newest 8 rows, injected as `memory:<key>` facts.
    `context.rs`, `execution.rs:856-886`, `store.rs:645-671`. **V**

14. **Is there per-hive or per-branch memory?**
    No. `agent_memory` keys on role only; every content-writer in every run/hive shares one
    channel. No hive/branch table, trust, expiry, or prompt snapshot. **V**

15. **Does the HyperAgent have access to all banks?**
    No. Root context seeding and role memory are its only inputs; no aggregate child-bank read
    exists. **V**

16. **Is there a real tool loop?**
    Yes after the previous fix: first leaf request advertises tools with `tool_choice=auto`;
    tool results replay into a continuation; finish reasons, budget preflight, fan-out, 5xx retry
    and reasoning pass-back were fixed. Live `file_read` proved it. **V**

17. **Are approvals enforced?**
    Not uniformly. `spawn_agent` is registered as an Execute-class tool but execution explicitly
    skips approval ("No approval is consumed"), so injected content can delegate without consent.
    Approval UI also lacks args/target text and dismiss for stale/expired entries.
    `tool.rs:334-339`, `execution.rs:1684-1691`, `ApprovalOverlay.tsx`. **V**

18. **Is the local HTTP boundary secure?**
    No Host allowlist. Origin checks exist for WS and provider metadata routes, but mutations
    such as profile import/delete and memory clear are reachable from a same-origin/DNS-rebinding
    context; the session token is in the WS query string. `http.rs:511-529,1017-1044`. **V**

19. **Does HiveMind use every DeepSeek API?**
    No. Wired: Chat Completions, thinking/effort, multi-round, JSON, tool calls, streaming,
    usage/cache counters, models, balance. Not wired: Files (image-only), Search/Fetch
    (per-request billed), Responses, Anthropic Messages, FIM/Prefix, Vision, strict tools,
    `user_id`. Anthropic requires no Anthropic key; DeepSeek is the only provider. **V**

20. **Can Rust deliver 300–600 agents today?**
    No. Provider capacity is not the binding limit (2,500 Flash / 500 Pro in-flight). Local
    policy is: 8 in-flight permits process-wide, 64 agents/run, 40 calls/run, 400K tokens,
    $0.50/run, one active run, one SQLite connection, 1,024/256 event bus, no UI culling.
    50–150 active and 300–600 total need a scheduler, storage writer, event coalescing, budget
    governor, hierarchical planning, and UI LOD. Rust speed alone is not enough. **V**

## 2. What each thing really is

| Thing | Have | Missing / false | Evidence |
|---|---|---|---|
| HyperAgent | Root planner task, UI anchor, plan ownership | No continuous agent loop/tool use; no global memory access | `execution.rs:1321-1335`, `provider_executor.rs:286-313` |
| SuperAgent | Tier, spawn policy, allowlist, persona/template | No own bank; no distinct runtime/UI identity beyond kind | `execution.rs:108-142`, `personas.json` |
| Agent/Worker | Tier, four personas, tools (read/list/write/shell/spawn/ask/sibling) | Persona prompt only first turn; planner emits `worker`, so most leaves use fallback | `provider_executor.rs:357-372`, `execution.rs:151-163` |
| Subagent | Persona/prompt, spawn role | No tier, allowlist, memory, card, contract kind | `execution.rs:151-163`, `event.rs:240` |
| Remote/Watchdog agent | **Not coded** | User wants low-complexity switch/remote agents with cards and reports | This is a target |
| Chat | Real API-backed operator surface | Not an agent; no chat-resident sub-orchestrator | `ChatCore.tsx`, `store.ts` |
| Templates | Manifest/persona validation, 7 profiles, `TEMPLATE_INSERT` | No super/sub template cards; worker templates unreachable from planner; UI no profile editor | `personas.json`, `ws.rs:200-293` |
| A2A | ask_parent/send_to_sibling rows+events | One-way; no answer path; no roster; no cross-hive bridge UI | `execution.rs:2417-2609` |
| Bus | Server event bus for UI | Not an agent knowledge bus; no topics | `bus.rs` |
| Memory | Task context + role memory (8) | No hive/branch banks, trust, retention, prompt snapshots, Hyper aggregate | `execution.rs:856-886`, `db.rs:180-197` |
| UI | Spatial canvas, cards, chat, provider panels, design system | Subagent/remote cards absent; no memory/A2A inspector; no model/thinking; approval blind; graph taxonomy/replay lie | `08-code-ui.md` |
| DeepSeek | Chat/thinking/tools/JSON/stream/cache/models/balance | Files/Search/Fetch/Responses/FIM/Vision/strict/user_id | `01-deepseek-api.md` |
| Scale | Bounded correct single-run engine | No scheduler; 8 in-flight; serial dispatch; one DB writer; 64/40/400K/$0.50 caps | `03-scale-concurrency.md`, `07-code-backend.md` |
| Security/cost | DPAPI launch, origin checks, approval gate exists | spawn approval bypass, no Host allowlist, 402 mislabeled, no per-hive budget/cost persistence, key in same-user reach | `06-security-cost.md` |

## 3. UI/UX truth

**What is strong:** the spatial canvas, hive anchor, chat core, card language, topbar/rail/library
geometry, FigCraft system, and the live run surface are distinctive and worth keeping.

**What is not true yet:**

- There is no visual vocabulary for Subagent or Remote/Watchdog agents; they render as Worker.
- Cards show fallback labels and permanently unknown progress/context/score because the server
  does not send label/progress/metrics.
- Replay flattens every node to `kind:'worker'` with synthetic labels; the graph taxonomy is
  half dead (`error`/`tool` are not produced on the normal path).
- Tool activity has two homes (chat ACTIONS and Agent tool panel) and no run-detail ledger.
- Approvals show tool + truncated `args_sha256`, not what the agent will do; stale entries can
  trap the operator.
- Memory is a read-only role list; there are no project/run/hive/branch bank views.
- Mailbox is read-only and no operator/agent can answer a pending question.
- No model, thinking, agent-profile, autonomy, budget or memory-sharing settings.
- Provider account is passive; no spend gate, no cache-hit/cost truth.
- The current Swarm dashboard tiles/hero empty state are not in the FigCraft direction.

**Target UI (from research and operator answers):** one graph, multiple projections — Hive
canvas, Treegrid, and run-detail ledger — sharing one `TaskRecord[]`; tier by size/containment/
header/role-chip, color reserved for state; all agents get a card; LOD/culling for 150–600;
approval inbox; memory bank inspector; A2A/mailbox queue; seven budget meters; settings for
autonomy, memory sharing, A2A visibility, and UI density; accessibility and reduced motion.

## 4. The target HMARS

```text
Operator / Settings
  → ChatCore (UI surface, not an agent)
  → HyperAgent (run owner; plan, budget, policy, memory access)
      → SuperAgent Hive A (branch bank; spawns agents/subagents; summarizes)
          → Agent (worker persona; tools)
              → Subagent (narrow task; asks parent; reports)
      → SuperAgent Hive B ...
      → Remote/Watchdog agents (simple monitored switches; report to owner)
  → RunEngine: admission scheduler → provider lease → task executor → verification
  → Memory bank tree: project → run → hive → branch + role banks
  → A2A: Escalate (ask), Notify (sibling), Delegate (inline spawn); roster/topics by policy
  → Event bus/UI: one graph, treegrid, ledger, approvals, budget, memory/A2A inspector
```

**Memory model:** explicit bank tree with snapshot reads, not a global soup. A child reads its
branch and ancestors; a SuperAgent owns its hive bank; the HyperAgent reads run bank + child hive
summaries/counts under policy; cross-bank promotion is one level and approval-gated. Global,
per-run, per-hive, per-branch, per-role are *settings scopes*, not one shared vector store.

**Settings model (from operator answers):** autonomy modes 1–4; approval policy; memory scope,
trust, visibility and retention (human-only changes); A2A mode (tree/siblings/hive/private
topics/mute/blind); model/thinking/effort; budget on/off (no hard budget by default), spend
alerts and emergency stop; UI density/projections; agent readability/blindness.

## 5. DeepSeek decisions

- **Wired:** Chat Completions; Thinking/effort; multi-round; JSON; tools; streaming; context
  cache accounting; models; balance.
- **Not wired:** Files (image-only; 64 MiB/file; 25 GiB; 10k files), Search/Fetch ($0.75/1k
  successful requests, only per-request billed surface), Responses, Anthropic Messages,
  FIM/Prefix, Vision (Flash only), strict tools, `user_id`.
- **Correction:** max output is verified **393,216** (384K × 1024), context 1M; prompts over the
  context window still fail after send because no preflight. Context cache requires a stable
  **full prefix-unit match**, not overlapping prefixes.
- **Token-cache goal:** build a canonical prefix: immutable identity/system → persona → stable
  loaded memory snapshot → transcript; no random IDs/timestamps before the cacheable prefix;
  measure `prompt_cache_hit_tokens` per agent/run; reuse prefixes across tasks; persist cache
  metrics. Do not claim "better than DeepSeek's harness" until measured.
- **Budget:** operator wants no hard default; security requires denial-of-wallet control. We can
  default to **no hard token cap but a visible soft spend guard + emergency stop**, with hard
  caps opt-in. This is a settings decision, not a silent removal of the current $0.50 cap.

## 6. Scale plan

Provider capacity (2,500 Flash / 500 Pro) allows the target; local design does not. Minimum
architecture from research #3: admission queue with deadlines; per-model AIMD lease; DB writer
actor + read-only connections; delta coalescing + viewport subscription; bounded per-session
outbound queue; two-level cost governor; culled/LOD UI; hierarchical planning (one 16K planner
cannot emit 300–600 nodes).

Stages: S0 reproduce 8-permit ceiling → S1 admission (25) → S2 storage/coalescing (50) →
S3 sessions (100) → S4 budget/AIMD (150 active) → S5 600 tasks + UI. No scale claim until the
stage acceptance tests pass.

## 7. Priority plan

| Phase | Work | Gate |
|---|---|---|
| P0 | ask_parent answer/deadline; parent-completion guard; spawn approval; Host allowlist; 402/400/422 mapping; persona continuation + role normalization; context-window preflight; planner JSON blank retry | targeted tests + live probe |
| P1 | Subagent tier; Remote/Watchdog role/cards; roster; A2A inbox/status; memory bank tree v11; settings schema/UI | migration + route + UI trace |
| P2 | admission scheduler; DB actor/read pool; event coalescing; 150 active; UI LOD | S0–S4 load criteria |
| P3 | Search/Fetch with cost; Files/Vision later; cache-prefix optimizer/tokenizer; Responses/FIM decision | provider capability tests |
| P4 | one graph/two projections; approval inbox; budget meters; memory/A2A inspector; remove dashboard slop | FigCraft parity + a11y |
| P5 | independent audit/security/QA/load; docs and claims | evidence-check pass |

## 8. Unverified / must not overclaim

- Live SuperAgent/Subagent run with the new persona prompt.
- `ask_parent` answer path (not built).
- Memory bank design not implemented.
- 150 concurrent agents not implemented/tested; 300–600 not supported.
- Search/Fetch, Files, Vision, Responses, FIM not wired.
- Context-window preflight absent; prompt-token estimate is bytes/4.
- 402/400/422 provider error identity not fixed.
- No Host allowlist; spawn approval bypass is real.
- Current app data is empty; UI claims about live hierarchy are not demonstrated.

**Verdict:** HiveMind has a credible HMARS foundation and a working DeepSeek tool loop, but it is
not yet the goal. The next work must be correctness/security first, then bank-tree memory + A2A,
then the scheduler and UI projections. Do not add dashboard decoration before those.
