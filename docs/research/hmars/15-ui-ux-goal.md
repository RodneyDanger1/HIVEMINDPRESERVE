# 15 — HMARS UI/UX and control-surface goal (second-wave lane 6)

**File:** `docs/research/hmars/15-ui-ux-goal.md`
**Author:** second-wave lane 6 (UI/UX + control surface), read-only seat
**Date:** 2026-11-07 (America/Chicago)
**Base:** repository `bd575ba`; working tree has one relevant uncommitted change,
`backend/crates/hiverouter-core/src/provider_executor.rs` (persona lookup normalization + persona prompt on tool
continuation — see §2.1). No product code, CSS, contract, or design asset was edited by this seat.
**Inputs read first (required):** `OPERATOR-DECISIONS.md`, `09-synthesis-and-goal.md`, `05-ui-ux-patterns.md`,
`08-code-ui.md`, `07-code-backend.md`; plus `02-hmars-patterns.md`, `03-scale-concurrency.md`,
`04-memory-architecture.md`, `06-security-cost.md`, `web/src`, `web/src/styles`, `web/tests`, the frozen
`web/src/api/contract.ts`, the Rust contract/core/provider/tools code, and the FigCraft HiveMind boards 05/08/09/12
(read as images, `docs/ui/figcraft/ASSETS/`).

**Primary discipline.** `/deep-inspection` is **not present** in this session's installed skill catalog (checked
`~/.dsh/skills`: only `deep-investigate`, `trace`, and other `deep-*` skills). Per the lane instruction I used the
closest available pair — **`/deep-investigate` + `/trace`** (both skill files loaded and followed), and state that
substitution here. There is no product-run reproduction available in this read-only seat (no restart, no paid call,
no browser session this pass), so `/deep-investigate`'s "reproduce twice" gate is reported as
`E_NOT_REPRODUCIBLE` for runtime-only claims; static reproduction is the cited source chain plus the contract/core
tests that already exercise it. `/trace` is applied to the controls this lane changes: render → handler → client →
request/route → service → store → provider → render-back, with `path:line` on every hop.

---

## 0. Claim classes and evidence snapshot

| Tag | Meaning in this file |
|---|---|
| **VERIFIED** | Exact `path:line` read in this pass (source as written), or a code fact proven by an existing test/fixture read in this pass. |
| **READ** | Read from a first-wave research document, a FigCraft board/export, or a prior audit; not re-measured. |
| **OPINION** | A design decision or product judgement; it is not a measurement or a contract fact. |
| **UNVERIFIED** | Not observed/proven in this pass; the named experiment is required. |

Runtime state is **not re-measured** in this lane. The last observed runtime facts are in
`08-code-ui.md` §0 (server `1.2.0-dev`, empty project/run DB, seven personas) and are **READ** here, not claimed
fresh. Design decisions that need a running server are marked UNVERIFIED with the test that would settle them.

---

## 1. Executive answer

The first wave proved the right direction (one graph, two projections, card v2, LOD, approval inbox, memory
inspector, budget meters), and `05-ui-ux-patterns.md` already contains most of the design vocabulary. The second
wave must make the design **executable** by the current codebase:

1. **One projection model, not two implementations.** Today `web/src/flow/graph.ts` and
   `web/src/panels/AgentHierarchy.tsx` separately derive a graph and a tree from `TaskRecord[]`; the canvas and
   hierarchy share only `selectedTaskId`. The HMARS view needs a single pure `AgentViewNode[]` projection
   (`web/src/state/agentView.ts`) consumed by both, with one card registry, one inspector, and one filter/selection
   model.
2. **Every entity gets an explicit card, including the ones the engine does not have yet.** HyperAgent (chat
   card), Hive (SuperAgent container card), SuperAgent, Agent/Worker, Subagent, Remote/Watchdog, ToolAgent, Error,
   plus non-agent Branch/Cluster representations. Subagent and Remote/Watchdog are **card classes**, not silent
   aliases: the current wire cannot distinguish them (Subagent → `kind:"worker"`, Remote/Watchdog absent), so the
   view model must read an additive `agent_class` (or `role` fallback) and be allowed to say
   "class not reported" rather than assert `WorkerAgent`.
3. **Density and LOD are separate settings, and both are bounded.** `05` fixes LOD bands B0–B4; the operator asks
   for role/task/state/cost/tools/branch on the card by default. Resolve this with three density presets
   (Compact/Standard/Detailed) applied on top of the LOD band: Standard is the operator default (identity + role +
   state + task + structured cost/tools/branch strip), Compact is the low-attention band, Detailed is the
   inspector-adjacent band. No card may invent a field the producer does not send.
4. **The evidence surfaces become destinations, not duplicate tables.** Tool activity gets one home in Run detail
   (board 12) with a compact chat strip; approvals become an inbox + focused decision dialog + decided ledger;
   memory and A2A get dedicated inspectors. The chat `ACTIONS` table and the Agents-panel tool table are removed.
5. **Truth before decoration.** The design removes/repairs the current lying/dead surfaces: default-kind
   `WorkerAgent` labels, permanent `—`/`0` metrics, dead Error/Tool/HyperAgent renderers, replay flattening to
   worker cards, chat Stop disabled during planning, stale approvals with no Dismiss, `CHAT_PIN` with no control,
   dashboard stat tiles/hero empty state, and three ambient motion loops.
6. **Settings, four autonomy modes, seven meters, provider/cache, tutorial — with honest capability gates.** None of
   these exist in the backend today; the UI must render "not available on this build" until the named route/command
   exists, and the design specifies the additive contract so a later lane can land it without redesigning the
   surface.

**The one-line goal:** the UI should be a **readable, auditable projection of one agent graph** where every visible
number has a producer, every card has an identity, every mutation has a named ACK/refusal, and the operator can
move between the hive map, the hierarchy tree, and the evidence ledger without changing what the data means.

---

## 2. Gap analysis of first-wave UI findings

### 2.1 What changed in the working tree since `08-code-ui.md`

These three facts change first-wave conclusions and are the baseline for this lane. They are **VERIFIED** by
reading the current tree (not by running it):

| Change | Evidence | Effect on first wave |
|---|---|---|
| Persona lookup is normalized (`trim().to_ascii_lowercase()`, `-`/space → `_`) and tool continuation now uses `self.system_prompt_for(task)` instead of `TASK_SYSTEM` | uncommitted diff in `backend/crates/hiverouter-core/src/provider_executor.rs` (`system_prompt_for`; `continue_with_tools` line ~372) | `08` finding #1 / `09` Q6 / `07` G2 are **partially fixed**: UI-spawned `SuperAgent`/`Subagent` now resolve their persona; `worker` still has no persona; planned worker nodes still fall back; this uncommitted change must land and be tested before the UI can claim persona identity |
| `ApprovalRequest` already carries `args_preview` (≤512 UTF-16 units, redacted, display-only) and the producer attaches it for a fresh request | `backend/crates/hiverouter-contract/src/event.rs:508-522` (`args_preview` field, `MAX_ARGS_PREVIEW_CHARS=512`, `redact_args_preview` at `:353-440`); `backend/crates/hiverouter-core/src/tools_bridge.rs:481-493` (`approval_request_event_with_args` sets it) | `08`'s "no preview field" is **stale**: the contract and producer support a bounded preview; **the UI ignores it** (`web/src/panels/ApprovalOverlay.tsx:24-34`, `:234-257` renders only tool/hash). This lane's design must render `args_preview`, not request a new field |
| `NodeEventPayload` already declares optional `label`, `progress`, `error_code` | `web/src/api/contract.ts:237-246` (fields present; comment says server may supply), `backend/crates/hiverouter-contract/src/event.rs:248-268` (NodeEvent has task_id/state/rev/kind/parent only — no label/progress in the **producer** struct) | The card v2 blocker is **producer**, not contract-to-UI: additive optional fields are already tolerated by the web decoder; the engine must fill `label`/`progress` (and a class field) and the store must stop inventing kinds |

Also verified: `error` is explicitly **client-only** and never a server node kind
(`backend/crates/hiverouter-contract/src/event.rs:915-921`; `NODE_KIND_VOCABULARY` at `:240`), so a "fix" that
adds a server `error` kind would violate the contract; the design instead makes the existing client-only
BrokenRoute card reachable from real failed-state tasks.

### 2.2 First-wave finding → current truth → lane-6 decision

Severity keys: **P0** = truth/safety; **P1** = required HMAS representation; **P2** = craft/density.

| # | First-wave finding (source) | Current truth (this pass) | Lane-6 decision | Primary owner files |
|---|---|---|---|---|
| 1 | Persona identity lost on production paths (`08` top-1; `09` Q6; `07` G2) | **Partially fixed (uncommitted):** normalization + continuation persona (`provider_executor.rs` diff); `worker` still has no persona; planner call still `PLANNER_SYSTEM`; planner nodes still emit `role:"worker"` (`provider_executor.rs:25-29`) | P0: land the fix + add `persona_id`/`role` to `NodeEvent`; UI card must show persona/role from the wire, never infer it; keep "persona not reported" fallback | `provider_executor.rs`; `execution.rs:743-779`; `store.ts:1441-1493`; `nodes.tsx` |
| 2 | No model/thinking/profile/spend controls (`08` top-2; `09` §3; operator decisions) | Settings values are env-only (`config.rs:22-131`); no mutation route/command (`http.rs:512-529`; `contract.ts:119-157`) | P0 design for Settings/Modes: add DB-backed settings + `GET/PUT /api/settings`; UI must label current env-only values "boot-configured" until the route exists; no fake selector | `config.rs`; new `core/src/settings.rs`; `http.rs`; `SettingsPanel.tsx`; new `settings/` store slice |
| 3 | Subagent has no identity/card (`08` top-3; `09` Q5; `07` G1) | Tier mapping falls to Worker (`execution.rs:151-162`); `NODE_KIND_VOCABULARY` has no subagent (`event.rs:240`) | P1: add additive `agent_class` (not a tier change); render `SubagentCard`; memory scope remains phase-gated; no capability claim | `contract/src/event.rs`; `execution.rs:97-106,743-779`; `agentView.ts`; `nodes.tsx` |
| 4 | Remote/Watchdog agents absent (`09` §1 row; operator decisions) | No code path, table, route, or kind (`grep` no daemon/watchdog agent code; `tools/watcher.rs` is a file watcher) | P1 design + contract seam: new persisted daemon entity, scheduler, `WATCHDOG_REPORT` event, `WatchdogCard`; UI gated behind capability flag until backend lands | new `core/src/watchdog.rs`; `db.rs` v12; `contract` events/commands; `Nodes.tsx`; `AgentsPanel` |
| 5 | Memory is role-only 8 rows (`08` top-5; `09` Q13/Q14; `04` bank tree) | `agent_memory(agent_key,key)` v7 (`db.rs:180-197`), recall 8 (`execution.rs:880-884`), write role-keyed (`:3017-3026`), read/clear HTTP (`http.rs:804-840`) | P1: phased Memory inspector — Phase A shows Role scope + "bank tree not available on this build"; Phase B after v11 shows Project/Run/Hive/Branch/Role with trust/read-set/truncation; no free-text editor | `db.rs` v11; `store.rs`; `http.rs`; `MemoryInspector.tsx`; `04` |
| 6 | Approvals blind (no preview) and stale entries undismissable (`08` top-4; `09` Q17; `05` P0-4) | `args_preview` exists in contract + producer (`event.rs:508-522`; `tools_bridge.rs:491`); overlay renders only hash (`ApprovalOverlay.tsx:234-257`); dismiss render only inside refusal block (`:303-332`); no focus trap/Escape | P0: render preview + from→to + TTL/consequence; Approval inbox/queue; dismiss/retry for expired/stale/unconfirmed; `alertdialog` focus contract; decided ledger; no optimistic close | `ApprovalOverlay.tsx`; new `ApprovalInbox.tsx`; `store.ts` approval slice; `HudBadge.tsx` |
| 7 | Tool activity split/buried; no run-detail ledger (`08` top-6; `05` §2.4) | Chat `ACTIONS` last 4 (`ChatCore.tsx:274-308`); full table under Agents (`AgentsPanel.tsx:65-72`; `ToolActivityPanel.tsx`); no Run detail destination | P1: one ledger in Run detail, compact chat strip, Agents panel links; board 12 layout | new `RunDetailPanel.tsx`; `ToolActivityPanel.tsx`; `ChatCore.tsx`; `AgentsPanel.tsx` |
| 8 | Graph taxonomy half dead + replay lie (`08` top-7; `09` §3) | `HyperAgentNode` filtered from canvas (`graph.ts:143-165`, `Canvas.tsx:167-189`); `error` client-only (`event.rs:915-921`); Tool card only for tool-tier tasks; replay forces `kind:'worker'` + `task <id>` labels (`graph.ts:196-234`) | P1: define card registry by `agent_class`; HyperAgent card = chat overlay (board 05); Error = client-only failed-state card (reachable); Tool card only when a ToolAgent task exists; extend history snapshot with class/label and stop flattening | `graph.ts`; `nodes.tsx`; `Canvas.tsx`; `event.rs`/history route; `08` |
| 9 | Server sends no label/progress/metrics (`08` top-8) | `NodeEvent` producer omits label/progress (`execution.rs:764-779`); store fallback labels (`store.ts:325-331`); cards show permanent `—` (`nodes.tsx:183-189,216-219`) | P0/P1: fill label + progress where measured (progress may stay `null` honestly; remove the permanent progress/ctx/score chrome rather than fake); identity = label or `class · depth · short-suffix` | `execution.rs:764-779`; `store.ts:1441-1478`; `nodes.tsx` |
| 10 | Provider account/spend passive/stale (`08` top-9; operator budget decisions) | Provider routes proxy live account (`http.rs:877-945`); usage read-only run aggregate (`http.rs:842-875`); cost computed in provider but not persisted/exposed (`budget.rs:24-28`); no cost column in `runs` (`db.rs:53-74,118-119,202-215`) | P1: Provider/Cache panel + seven meters; expose settled cost only when `CostStatus==Complete`; persist `provider_calls`/cost and expose a run budget route; unknown is `—`; no spend gate until policy lands | `provider/src/accounting.rs`; `budget.rs`; `db.rs` v12; `http.rs`; `ModelsPanel.tsx`; new `BudgetStrip.tsx` |
| 11 | HMAS control surfaces read-only/absent (`08` top-10; `05` §3.1) | ControlBar only Workspace/workspace (`App.tsx:125-130`); Swarm counters+rows (`SwarmPanel.tsx:13-151`); routing destination (`NavRail.tsx:9-16`); tabs global (`App.tsx:50-102`) | P1: IA restructure — Workspace, Swarm (Hive/Tree), Run detail, Agents, Data, Models, Settings; Routing folded into Tree "Parent" column; tabs scoped to Workspace | `App.tsx`; `NavRail.tsx`; `SwarmPanel.tsx`; new `AgentTree.tsx`; `RoutingPanel.tsx` removed |
| 12 | Density/settings/LOD/tutorial absent (`05` P1-2/P2-7; operator decisions) | No density/LOD/projection/tutorial state (`store.ts:242-321`; tokens only sizes `tokens.ts:166-179`) | P1: view-state slice (projection/density/band/inspector/filters), LOD module, settings/tutorial surfaces | `store.ts`; new `lod.ts`; `agentView.ts`; new settings/tutorial components |

### 2.3 Contradictions between first-wave documents and operator decisions (resolved here)

| Contradiction | First wave | Operator decision | Lane-6 resolution |
|---|---|---|---|
| Card fact budget | `05`: at most **4 facts** on a card; inspector for the rest | Agent cards show **role, task, state, cost, tools, branch** by default | Standard density uses **two text lines + a structured footer strip**: identity+state, role+branch, task (1 line), cost/tools/progress. This keeps the *prose* facts ≤4 and makes the rest a compact metric strip; Detailed density exposes all six plus model/tokens. Marked OPINION; open question Q5 in §8. |
| Where tool activity lives | `05` says run detail; W11 implementation chose Agents (`08` §7) | "all projections settings, main view hierarchy + live execution + memory/A2A edges" | Run detail is the **only full ledger**; chat keeps a one-line live strip; Agents keeps a *link*, not a second table. |
| Approvals: modal vs inbox | `05` recommends inbox + focused modal | Approval policy is a setting; default side effects only | Inbox + focused `alertdialog`; per-approval decisions only (no bulk/sticky auto-approve until a real model exists); modes 1–4 change the *policy*, not the decision UI. |
| A2A compose | `05`: no compose until a send route exists | A2A UI is a dedicated topology/memory inspector by default; all projections settings | Read-only mailbox + operator **reply to an open Escalate** is the first send-shaped action, and only after an `AGENT_MESSAGE_REPLY` route exists; no generic composer before that. |
| Budget default | Code has **hard** $0.50 / 400k / 40-call caps (`budget.rs:24-31`) | "No hard default budget; token caching is the goal; a visible soft spend guard/emergency stop is acceptable" | Meter row shows the real current cap; settings expose `budget_mode: hard|soft` with hard caps as the safety floor until the security lane signs off a no-cap mode. The UI must never silently imply "no cap" when the provider preflight still refuses at 400k/$0.50. |
| Projections default | `05` density budget: one primary region + one inspector + status strip | "all projections (canvas, chat, graph, dashboard) are settings, all enabled by default" | Two projections, both enabled by default and first-class (Hive/Tree toggle); "dashboard" is explicitly **not** built (no stat-tile dashboard); chat is pinned on Workspace. The setting turns a projection off, never turns a hidden surface on. |
| Subagent representation | `09` P1 "Subagent tier"; `05` "no Subagent card exists" | Every agent, including Tiny Watchdog/Remote, gets a UI card | Additive **`agent_class`** on the wire (not a new runtime tier) drives Subagent/Remote/Watchdog cards; the engine tier remains Worker for subagents until the backend lane changes capability (UI must not claim a tier it does not have). |

### 2.4 First-wave open items (`05` §8 U1–U10) mapped to this lane

| U | Question | Lane-6 status / decision |
|---|---|---|
| U1 | Producer accepts additive `label`/`progress`/`role`/`template_id`? | Contract already tolerates optional `label`/`progress` (`contract.ts:237-246`); producer fields are **still absent** (`execution.rs:764-779`). Design requires them; add `persona_id`/`agent_class` too. **UNVERIFIED** until producer lands. |
| U2 | Tool activity owner: run detail or Agents? | Run detail (this file, §5.6); Agents is a link. |
| U3 | Read route for approval/refusal ledger? | Absent; design defines `GET /api/runs/{id}/approvals` + client session ledger fallback labelled "session-only". |
| U4 | `RUN_METRICS.green_pct` definition? | Formula is `completed/tasks*100` (`execution.rs:653-696`); HUD must label `derived`, or remove; design replaces vanity GREEN with budget chips + explicit derived census. |
| U5 | LOD thresholds/durations authoritative? | Adopt `05` B0–B4 with ±0.04 hysteresis as **OPINION**, pinned by tests; no board change needed. |
| U6 | Tree/canvas shared selection re-render storms? | Design requires a pure `agentView` memo keyed by `(graphRev, projection, filters)` and selectors; test with 64/600 synthetic tasks; no layout recompute on selection. |
| U7 | A2A `contextId`? | Absent; group by run + branch pair; design adds optional `context_id` additive later. |
| U8 | Browser e2e available for a11y/motion gates? | Not established in this lane; acceptance tests are unit/DOM-level in vitest + manual browser checklist; UNVERIFIED. |
| U9 | v11 memory bank before inspector? | Phased: Role-only inspector now (honest label); five scopes after v11. |
| U10 | `agent_messages` gets `in_reply_to`/`status`/`answered_at`? | Design requires these for Escalate status; Notify stays delivery-only; no unread badge. |

---

## 3. The exact view model for every HMARS entity

### 3.1 Definitions (one vocabulary, no shadow model)

| Entity | Definition | Backing today | Card? |
|---|---|---|---|
| **Run** | One HyperAgent execution, one active run per server (`execution.rs:1032-1034`) | `runs` table (`db.rs:53-61`); `RUN_*` events | Run detail destination (not an agent card) |
| **HyperAgent** | The run's root task (`role:"planner"`, depth 0), owner of the plan/budget/policy; never executes a model turn today (`07` §3.2) | `kind:"hyperagent"`, filtered from canvas (`graph.ts:151`) | **Yes — its card is the Hive Chat Core** (board 05 "HYPERAGENT / HIVE CHAT CORE", 400×480; `tokens.ts:170`). It is not a second canvas card. |
| **Hive** | A SuperAgent-owned subtree: the SuperAgent task + its descendants + its bank. Operator ownership modes (department/domain/branch-lead/single task) are all the *same* structural object under the task tree | SuperAgent task + parent links; bank tree in `04` §4 | **HiveCard** = SuperAgent card rendered as a container (honey header + containment outline + branch count/bank badge). One per SuperAgent with descendants; no extra hidden node. |
| **SuperAgent** | Tier `AgentTier::SuperAgent`; owns one branch/hive; may spawn workers/subagents; persona `superagent` (`personas.json`; `execution.rs:130-146`) | `kind:"superagent"` | **Yes — SuperAgentCard** 420×150 (`tokens.ts:174`) |
| **Agent (Worker)** | Tier `WorkerAgent`; executes one narrow task; may spawn Subagent only | `kind:"worker"`; role may be `worker` or a persona id | **Yes — AgentCard** 320×96 (`tokens.ts:175`) |
| **Subagent** | Spawn role `subagent` with a narrow persona; currently maps to `WorkerAgent` tier; no distinct wire kind (`execution.rs:151-162`; `event.rs:240`) | role string; no class field yet | **Yes — SubagentCard** (new, compact; size proposal 280×88 OPINION); driven by additive `agent_class:"subagent"` or `role:"subagent"` fallback; renders "tier: worker (subagent role)" honestly |
| **Remote / Watchdog** | Long-lived daemon agent detached from a run: one job forever (e.g. "watchdog this hive and report every 5 minutes"), low complexity, reports to its owner (operator decisions) | **Not coded** (no table/route/kind; `tools/watcher.rs` is a file watcher, not this) | **Yes — WatchdogCard** (new, proposal 320×120 OPINION) + a "Remote" variant for a managed remote endpoint; gated behind an `agent_class:"watchdog"/"remote"` capability flag until backend lands |
| **ToolAgent** | Tier `ToolAgent`: deterministic tool invocation, no spawn; only if a plan/template creates a tool-tier task | `kind:"tool"`; currently rare (planner prompt offers only worker/superagent, `provider_executor.rs:25-29`) | **Yes — ToolCard** 420×240 (`tokens.ts:176`) when the task exists; tool *calls* are evidence in Run detail, not agent cards |
| **Branch** | The path from a Hive root down to a leaf; not an agent | parent links | No agent card; rendered as **BranchBand/BranchSummaryCard** (collapsed LOD B0 cluster, tree row group, optional band in canvas) |
| **Cluster** | LOD aggregate of a collapsed subtree (B0) | derived | **ClusterCard** (not an agent; `N agents · state`, no identity) |
| **Error** | A client-only broken-route *state* of a task card (never a server kind — `event.rs:915-921`) | task `state: failed/killed/interrupted` + `errorCode` | **Yes — ErrorCard** 320×110 (`tokens.ts:177`), rendered in place of the kind card only when the task is terminal-negative and `error_code` is real |
| **Project** | Registry group above runs (`db.rs:233-248`; `GET /api/projects`) | `projects` table | Not a card; project selector chrome |

### 3.2 `AgentViewNode` — the single projection record (design; new file)

`web/src/state/agentView.ts` (new, pure, no React):

```ts
export type AgentClass =
  | 'hyperagent' | 'superagent' | 'worker' | 'subagent'
  | 'remote' | 'watchdog' | 'tool' | 'error' | 'cluster' | 'branch';

export interface AgentViewNode {
  readonly taskId: string;
  readonly class: AgentClass;            // never guessed: wire class → role → kind → 'worker' only if kind says so
  readonly classSource: 'wire' | 'role' | 'kind' | 'fallback'; // rendered as a "class not reported" hint at Detailed density
  readonly label: string;                // wire label, else `${roleLabel} · d${depth} · ${tail4}`
  readonly role: string | null;          // wire role/persona id; never invented
  readonly depth: number;                // from parent chain (authoritative)
  readonly parentId: string | null;
  readonly hiveId: string | null;        // nearest superagent ancestor, own id for superagent
  readonly branchId: string;             // `${hiveId ?? 'root'}/${taskId}` path key
  readonly state: AgentState;
  readonly stateFamily: 'live'|'steady'|'waiting'|'terminal-negative'|'system';
  readonly progress: number | null;      // null stays null
  readonly errorCode: string | null;
  readonly isHiveRoot: boolean;
  readonly childCount: number;           // derived
  readonly runningChildren: number;      // derived
  readonly metrics: {
    readonly tokens: number | null;      // wire/run attribution; null until producer
    readonly costUsd: string | null;     // settled Decimal as string; null when unknown
    readonly toolCalls: number | null;
    readonly toolTurns: number | null;
    readonly messages: number | null;
  };
  readonly bankHint: { readonly scope: 'role'|'hive'|'branch'|'run'|'project'|'unavailable'; readonly count: number | null };
  readonly watchdog: { readonly cadenceSecs: number; readonly lastReportAt: string | null; readonly nextTickAt: string | null; readonly ownerTaskId: string | null } | null;
}
```

Derivation rules (all testable; **OPINION** except cited source):

1. `class` = wire additive `agent_class` if present and in the union; else `role==='subagent'` → `subagent`;
   else `kind==='hyperagent'|'superagent'|'tool'` → same; else `kind==='worker'` → `worker`; else fallback
   `worker` + `classSource:'fallback'` **only when kind is absent**; absent class is displayed as "class not
   reported", never asserted as a tier.
2. `label`: wire `label` if non-empty; else `${role || classLabel} · d${depth} · ${last4(taskId)}`. Never use the
   first 8 hex of a UUIDv7 as identity (all tasks in a run share the timestamp bucket — `05` §2.2 READ;
   `store.ts:325-331`).
3. `progress`: wire `progress` (0–100) or `null`. No `0` default, no clamp-to-zero; a null bar is **absent**, not
   empty. `metrics` values null until the additive producer/API lands.
4. `hiveId`: nearest ancestor with class `superagent`; a SuperAgent's own `hiveId` is itself. A HyperAgent's
   `hiveId` is null (run-level).
5. `branchId`: `${hiveId ?? 'root'}/${taskId}`; used for row grouping, A2A branch pairs, and memory scope keys.
6. `stateFamily`: map 16 states to the five families in `05` §3.5 (live, steady, waiting, terminal-negative,
   system). The `system` states (`reconnecting/replay/empty/loading`) are surface states and are rendered as a
   banner/skeleton, never painted on an agent card. (`agentState.ts:9-26,58-84,111-123` already provides the set
   and shapes; the view model adds the family mapping.)
7. Sorting: stable topological order (parent before child, then `spawn_seq`/created order); `S` (sibling index) and
   `D` (depth) are derived deterministically; never sort by label alone (labels can collide).

### 3.3 Card registry — every entity gets exactly one representation

| Representation | Entity | Where it renders | Required content | Actions |
|---|---|---|---|---|
| `HiveChatCard` | **HyperAgent** (run root) | Workspace centre (fixed overlay, board 07) | title "Hive AI Assistant" + run state + plan summary (steps) + budget chip + model/thinking (if reported) + chat transcript/composer | submit prompt, stop/cancel (active states), open Run detail, open plan inspector |
| `HiveContainerCard` / `SuperAgentCard` | **SuperAgent** + its **Hive** | Canvas (Honey header, containment outline when expanded) | identity, `SUPERAGENT`, state (dot+glyph+text), `N workers · M running`, hive bank badge, progress if reported | select, expand/collapse children, open inspector, spawn under (if allowed), kill subtree (confirm) |
| `AgentCard` | **Agent/Worker** | Canvas, tree row, inspector | identity, `WORKER` + role/persona chip, state, branch chip, progress or `—`, cost/tools/branch strip | select, open inspector, re-route (dialog), kill (confirm) |
| `SubagentCard` | **Subagent** | Canvas, tree row, inspector | identity, `SUBAGENT` + "worker tier (subagent role)" chip, state, parent link, `ask_parent` status if present, cost/tools strip | select, open inspector, message parent (only after reply route), kill |
| `WatchdogCard` | **Remote/Watchdog** | Canvas (daemon band), Agents registry, Settings list | identity, `WATCHDOG`/`REMOTE`, state (healthy/running/paused/failed), cadence (`every 5m`), last report time, next tick, owner branch | pause/resume, report now, edit cadence (settings), delete (confirm); gated until daemon APIs exist |
| `ToolCard` | **ToolAgent** task | Canvas | identity, `TOOL`, state, args/result preview only when server sends (`args_preview`/`args_sha256`), score only if reported | select, open run-detail ledger filtered to the call |
| `ErrorCard` (client-only) | Failed/killed/interrupted task | Canvas replacement card | identity, real `error_code`, neutral "error state" if code absent, halted-bee wire, retry attempt only if reported | open inspector/run detail; no "Compile failed" copy |
| `ClusterCard` | collapsed subtree (LOD B0; not an agent) | Canvas macro layer | `N agents · state`, tier silhouette; no identity | click to expand/focus; never selectable as an agent |
| `BranchBand` / `BranchSummaryCard` | **Branch** (not an agent) | Canvas grouping band; Tree group header | branch id/label, `N agents`, `M running`, state aggregate, bank badge; collapsed summary | expand/collapse, focus branch, open branch inspector |
| `RunSummary` | **Run** (not an agent) | Run detail header; HUD | run id short + label, state, started/ended, revision, budget strip | replay revisions, cancel, open ledger |

The rule that satisfies "every agent gets a card": the card registry is **total over `AgentClass`** and a
compile-time `satisfies`/exhaustive switch; a class with no producer still renders an honest "not available on
this build" card, not a missing node.

### 3.4 State rendering (one visual vocabulary)

Keep `agentState.ts` as the single vocabulary: 16 states (`agentState.ts:9-26`), 4 dot states
(`:30-31`), shape+text cues for the rest (`:58-84`), terminal/active predicates (`:111-123`). Add only the
family map (live/steady/waiting/terminal-negative/system) and a card rule: **state = dot + glyph + text; colour is
reinforcement.** `completed` gets `verify`, not honey (`05` §3.5); `awaiting_approval` must link to its approval
row; `interrupted` is not `failed`; system states are banners, not painted cards.

---

## 4. One graph, two projections, LOD, and card content by density

### 4.1 One record, two projections, one selection

- **Record:** `AgentViewNode[]` from `web/src/state/agentView.ts`, memoized on
  `(graphRev, run?.id, history.replayGraph, filters, projection)`. All surfaces read this one projection:
  `Canvas` (Hive), `AgentTree` (Treegrid), `RunDetailPanel` (ledger), `AgentInspector`, `SwarmPanel` summary,
  `HudBadge` census.
- **Hive projection** (`web/src/flow/Canvas.tsx` + `graph.ts`): React Flow 12.12.1, same card registry, tier by
  geometry/surface/header (board 05), state colour only, edges as bee direction source→target, branch bands.
- **Tree projection** (new `web/src/panels/AgentTree.tsx`): WAI-ARIA **treegrid** (`role="treegrid"`,
  `aria-expanded`, roving tabindex, Up/Down/Right/Left/Home/End/Enter/Space), columns:
  `Agent (expand/collapse, class glyph, label, role chip)` · `State (dot+glyph+text)` · `Workers` · `Progress` ·
  `Cost` · `Tools` · `Branch` · `Last event`. Root summary row = HyperAgent + `N hives · M agents · state`.
  Virtualize when rows > 200 (`aria-rowcount`/`aria-rowindex`); compact/comfortable row height 32/40 px.
- **Shared selection:** `store.selectedTaskId` remains the single selection; selecting in Tree reveals on Canvas
  (camera action) and vice versa; no second selection model. `filters` (hide finished, focus branch, search,
  class, state family) are store state, shared by both projections. Toggle is first-class (`Hive | Tree`) in Swarm
  and persisted; `Ctrl/Cmd+0` recentres Canvas from either.
- **Routing:** `RoutingPanel` is removed; its parent-edge table becomes the Tree's `Parent` column (developer
  disclosure) — a debug table is not a destination (`05` §3.1).
- **Replay:** one projection from `HISTORY_SNAPSHOT`; both projections are read-only and mutation-disabled at the
  control **and** store level; card class/label come from the additive snapshot fields, not re-derived as
  `worker` (`graph.ts:196-234` must be replaced, not patched).

### 4.2 LOD bands (canvas), hysteresis, culling (OPINION; thresholds adopted from `05`)

| Band | Zoom | Canvas renders | Card content | Edges / motion |
|---|---|---|---|---|
| **B0 Macro** | 0.10–0.34 | Batched SVG layer: Hive clusters + ClusterCards (`N agents · state`), root silhouette; **no card DOM** | cluster only | static bee direction; ≤50 animated edges total; no free-flying bees |
| **B1 Far** | 0.35–0.59 | Card header + state dot/glyph + identity (1 line, truncated) | Compact density: class glyph + identity + state | edges drawn; one 200 ms transition on changes |
| **B2 Fit** | 0.60–1.19 | Full card (board 05 anatomy / registry) | **Standard density** (default) | normal |
| **B3 Detail** | 1.20–2.19 | B2 + role chip, branch chip, last event/turn, worker count for Hive | Detailed density | normal |
| **B4 Inspect** | 2.20–3.00 | B3 + Tool args/result preview (`args_sha256`/`args_preview` when present), error code, memory badge; inspector recommended | Detailed + evidence fields | normal |

Mechanics: ±0.04 hysteresis (band changes via a small store slice, never per-frame React state); culling via
React Flow `onlyRenderVisibleElements` **plus** a B0 batched layer; layout is memoized on a stable topology key
(run id + sorted `id:class:parentId`), never recomputed on selection/drag; camera is owned by React Flow, `Ctrl+0`
recentres (0,0) without zoom change; minimap shows class/state rectangles + viewport outline, is hidden below
1280 with an "Overview" fit button (current minimap is textured/empty and `@media max-width:1279px` hides it —
`index.css:137-142`, `Canvas.tsx:407-435`).

### 4.3 Card content by density (the operator-required facts)

Density is a **setting** (Compact / Standard / Detailed), persisted per projection; LOD band caps what can be
shown at a zoom (effective content = min(band, density)). `—` means "not reported" (honest unknown); no field is
defaulted to zero.

| Card | Compact (B1; default at far) | Standard (default; B2) | Detailed (B3/B4; inspector-adjacent) |
|---|---|---|---|
| **HyperAgent / HiveChat** | run id short + state + plan step count | title, state, model (if reported), budget chip, plan `n steps` | + persona hint, thinking/effort (if reported), cache chip, last event, open-ledger links |
| **Hive/SuperAgent** | identity + state | identity, `SUPERAGENT`, state, `N workers · M running`, hive bank badge (if reported) | + role/persona chip, progress, hive cost/tokens, A2A count, branch list (expand) |
| **Agent/Worker** | identity + state | identity, `WORKER` + role chip, state, branch chip, progress or `—`, cost/tools footer strip | + task text 2 lines, model, tokens, retries, last event, memory bank badge |
| **Subagent** | identity + state | identity, `SUBAGENT` + parent link, state, task text 1 line, cost/tools strip | + persona, ask-parent status, last event, branch path |
| **Remote/Watchdog** | identity + state | identity, `WATCHDOG`/`REMOTE`, cadence, last report, next tick, owner | + last payload summary, error/alert count, settings link |
| **ToolAgent** | identity + state | identity, `TOOL`, state, args hash/short result label | + args/result preview only if wire sends it; score only if reported |
| **Error (client-only)** | identity + state | identity, real `error_code` or neutral "error state", halted wire | + retry attempt if reported, link to run-detail ledger |
| **Branch band / Cluster** | `N agents · state` | branch/collapse summary with `M running` | + branch bank/A2A counts, expand |

Operator default facts (role, task, state, cost, tools, branch) are all present at **Standard** for agent cards
(task clamped to one line; cost/tools/branch as a footer strip), satisfying the operator decision without turning
the card into a table. OPINION; see Q5 for the alternative.

### 4.4 Performance/culling acceptance (from `03` §9.4, READ)

At the HMARS target: ≥50 fps pan/zoom at 600 nodes (1440×900, Chrome trace, 10 s); ≤120 mounted canvas nodes at
any zoom; ≤50 animated edges; initial paint <1 s; snapshots ≤300 KB and ≤4/s; heap <300 MB after 10 min at 150
agents. Today: React Flow renders every node (no culling — `Canvas.tsx:351-435`; `03` §9.1 READ), `dagre` layout
runs from `buildNodes` per projection (`graph.ts:149`), and the minimap has no per-node content. These are P1
blocking gaps for the target scale, not polish.

---

## 5. Inspectors and panels (exact contents and truth rules)

### 5.1 Approval inbox + focused decision (P0)

**Inbox:** a persistent count chip in the HUD only when `approvals.length > 0`; opens `ApprovalInbox` (right rail
or Run detail section). Row: `from → to` (task labels/roles, short ids), tool, task/branch, age, TTL, stale
badge, decision state. Selecting a row opens the focused `alertdialog`.

**Decision dialog (must show):**
- Title, branch narrative (`from` label/role → `to` label/role, or "operator"), tool, task/tool-call short ids;
- `args_sha256` (12 hex + copy) and **`args_preview` when present** (≤512 UTF-16; monospace, `[redacted]` markers
  preserved; absent → "preview unavailable" — never reconstruct from args client-side);
- TTL countdown and `nonce` single-use copy; consequence line: "Approve grants the engine exactly one consume; it
  does not execute by itself";
- Deny first in DOM/focus order; Approve; `Escape` closes without deciding; focus trap and focus return; no
  optimistic close (only `APPROVAL_RESOLVED`/named refusal/timeout closes);
- Expired/stale read-only with **Dismiss** (and Retry only when the store classified a retryable failure);
- Decided ledger rows (approved/denied/expired) with nonce consumed yes/no, time, link to tool result.

**Truth rules:** current `ApprovalOverlay` already has the nonce/TTL state machine (`approval.ts`; store
`resolveApproval/retryApproval/dismissApproval`), but renders only tool/hash and hides Dismiss/Retry inside the
refusal block (`ApprovalOverlay.tsx:234-257`, `:303-332`). The fix is render + layout, not a new protocol. No bulk
approve, no auto-approve, no sticky decision until a backend model with audit exists.

### 5.2 Memory bank inspector (P1; phased)

Phase A (current capability): **Role scope only**, honest banner "bank tree not available on this build",
read-only rows from `GET /api/agents/memory` (≤8 per persona), with provenance (`run_id`/`task_id` columns exist
but aren't returned today — add to route), clear-per-role with confirmation and audit note. No fake scope tabs.

Phase B (after `04` v11 migration): tabs **Project / Run / Hive / Branch / Role**; header states the read set
("own + ancestors at `as_of_rev` + own role bank"), render budget ("12 entries / 4,000 chars; N omitted
(truncated)"). Columns: `Kind`, `Bank`, `Source task`, `Trust` (`unassessed/quarantined/unverified/verified/
authoritative`), `Written rev`, `Superseded`, `Retention`, `Actions`. Actions: **View** (read-only drawer with
content hash/`as_of_rev`), **Promote** (only one level up; confirmation + audit), **Purge** (content gone, receipt
kept; confirmation). No free-text edit until a write route + consent model exists; no "clear all" without scope;
read failure is a refusal/warning, never an empty success (`04` I11). Engine read path changes are `04` §6.2, not
this lane.

### 5.3 A2A mailbox + roster (P1)

- **Mailbox:** ledger rows grouped by branch pair: `from → to` (labels/roles), verb (`ask_parent` = Escalate,
  `send_to_sibling` = Notify), text clamped to 3 lines + expand, `created_at`, link to the approval when present,
  status column. **Status honesty:** today `AgentMessagePayload` has no `status`/`in_reply_to`/`answered_at`
  (`contract.ts:315-322`; db v6 `db.rs:160-178`), so Phase A renders **"status not reported"**; Phase B renders
  Escalate `open/answered/expired` and Notify delivery-only (no reply semantics). Header shows
  `messages N / 32` from a real count (route exists via message list; add a count field), and "read-only ·
  delivery is a branch event" until a send route exists. Do **not** show an unread badge (no read state).
- **Roster:** operator-only list of run agents (id, label, class, parent, depth, state, role) used to make mailbox
  targets and tree navigation intelligible; optional future `list_agents` tool for models with a bounded response;
  no discovery API/exposure of the roster to agents until the backend lane adds it (`02` §6.3). Current A2A is
  **write-only to a mailbox** (no delivery/answer path — `07` §7.4), so the UI must not imply a two-way
  conversation. An operator **reply** control appears only when `AGENT_MESSAGE_REPLY` exists, and it answers an
  open Escalate (not a generic chat composer).

### 5.4 Seven budget meters (P0/P1) — exact seven, exact sources

The seven are the engine/provider caps the operator must see; the per-tool-loop limits are **secondary meters**,
and the global in-flight limiter is a **status chip, not a run meter** (`05` Q14; `budget.rs:24-52`; `caps.rs:9-15`).

| # | Meter | Denominator | Current source | Missing to render truthfully |
|---|---|---|---|---|
| 1 | **Agents** | 64 per run (`caps.rs:13`) | derived from `tasks` (`Object.keys(tasks).length`) | none for live; replay count from HISTORY_SNAPSHOT |
| 2 | **Depth** | 3 (`caps.rs:9`) | derived max `depth` from task parent chain | depth must be carried/derived in `agentView` (parent chain) |
| 3 | **Fan-out** | 8 per parent (`caps.rs:11`) | derived max children per task | same |
| 4 | **Provider calls** | 40 attempts incl. retries (`budget.rs:31`) | **in-memory** `RunBudget.attempts`; not persisted/exposed | persist attempts/calls per run (migration v12) + `GET /api/runs/{id}/budget` |
| 5 | **Tokens** | 400,000 combined (`budget.rs:28`) | `run_usage` six counters (`db.rs:202-215`; `GET /api/usage/runs/{id}`) | null semantics already honest; live update frequency (event or poll) |
| 6 | **Cost** | $0.50 settled (`budget.rs:25`) | `RunAccounting` in provider; **not persisted/exposed**; only COMPLETE cost is valid (`06` §6.1) | persist `cost_usd` + `cost_status`; route; render `$0.50 cap · spend not settled` when unknown |
| 7 | **Messages** | 32 per run (`execution.rs:70`) | `agent_messages` count (route returns list; add count) | expose count; use A2A inspector header |

Secondary tool-loop meters (Run detail only, labelled group "Tool loop"): **turns 4** (`tool_loop.rs:14`), **calls
16** (`:16`), **approval waits 8** (`:21`), result cap 64 KiB (`:18`). Global **in-flight 8** (`budget.rs:43`;
`InFlightLimiter`) is a status chip in the provider panel, not a run meter.

Rules: unknown = `—` / "not reported", never 0; warn at 80 % with text+icon (not colour alone); a cap hit renders
the named refusal (`E_AGENT_LIMIT`, `E_CALL_LIMIT`/`E_TOKEN_BUDGET_EXCEEDED`, `E_BUDGET_EXCEEDED`,
`E_SPAWN_DEPTH`, `E_FANOUT_LIMIT`) and highlights the exact meter; provider `finish_reason` (`length`,
`content_filter`, `insufficient_system_resource`, `aborted`) is labelled "provider stop", not a HiveMind refusal;
the HUD mirrors only **agents**, **calls**, and the approval count; Run detail owns the full strip; a
`budget_mode` setting (hard/soft) is clearly labelled — never show "unlimited" while the provider preflight still
refuses (`budget.rs:235-284`).

### 5.5 Provider & cache panel (P1)

Destination `Models` becomes **Provider & Cache**, three stacked cards (board 12 layout):

1. **Account**: provider/model, key status (`present|missing|unknown` from `SETTINGS_STATUS`/`health`), server
   health/version, balance rows with currency + total/granted/topped-up, `is_available`, **`as of` fetch time**,
   refresh (explicit, bounded, stale-while-revalidate; routes already `http.rs:877-924`, add cache/`as_of`).
2. **Catalogue**: every provider model with id, context window, max output, and wire support derived from a
   server-exposed supported set — **not** the hard-coded `WIRE_MODELS` set (`ModelsPanel.tsx:31`, rendered `:242`);
   unsupported = "not accepted by this build" with the requested-model value shown separately.
3. **Usage & cache**: six run counters (prompt, completion, total, cache hit/miss, reasoning) with null→"Not
   reported"; cache ratio only when hit+miss known (`hit/(hit+miss)`), else `—`; cost only when accounting
   `COMPLETE` (`06` §6.1); `INCOMPLETE` means "settled cost unavailable — a call lacked usage"; per-agent/hive
   attribution after the v12 `provider_calls` migration (call index, task, model, tokens split, cost, at); cache
   goal checklist ("stable prefix: model/system/persona/memory snapshot/transcript") with **measured** hit rate
   only; "no cache claim without measurement" copy. Errors: loading text, `code: message` role=alert, no key
   leakage, no client-side provider calls; balance never gates a submission until a policy lands.

### 5.6 Run detail — the single evidence surface (P1; board 12)

Destination `Run detail` (selected run; also the second half of Workspace? See §6 IA). Sections stacked:
1. **Summary** — run id/label, status, started/ended, revision, project, budget strip (§5.4), model/provider.
2. **Tool activity ledger** — the *only* full table: columns turn, tool, task/branch, outcome (closed enum),
   code, `args_sha256` (8 hex + copy), result `bytes`/`sha256`, time; group by branch→turn; filters
   running/refused/denied/timeout/cancelled/task; empty and 200-cap truncation footers; row click selects task.
   The chat keeps only a one-line strip: `2 tools running · 1 awaiting approval → Open run detail`.
3. **Approvals** — inbox (§5.1) + decided ledger.
4. **Failures & refusals ledger** — run-level error card (code/message/how_to_fix/retryable), refusal ledger
   (session-only unless a read route exists), provider stop reasons labelled as provider.
5. **Files** — read-only tree/preview, jail path, 1 MiB cap; no diff until `DIFF_*` freezes.

### 5.7 Settings, autonomy modes 1–4, and tutorial (P1)

**Settings schema** (DB-backed, versioned, audited; env stays boot-only for secrets/port/path):

| Section | Fields (defaults) | Human-only | Agents can read? |
|---|---|---|---|
| General/UI | density (`standard`), projections (`hive+tree` on), reduced motion (OS), tutorial state | no | no |
| **Autonomy** | `mode` 1–4 (default **1**, operator decision), runner policy, retries, escalation policy | no | no (policy is external to agents) |
| Approvals | policy: side effects only (default), per-tool classes, per-hive overrides, expiry seconds | no | no |
| Agents & templates | profiles import/delete (current routes), built-in templates (read-only), custom templates, capabilities, max children/depth within global caps | no | no |
| **Memory** | scopes enabled, sharing/promotion policy, trust display, retention mode | **yes** (retention/visibility/blindness) | no |
| A2A | mode (tree/sibling/hive/private topics/mute/blind), cross-hive permission, budgets display | no | no |
| Provider & budget | model/thinking/effort, max output, `budget_mode` hard/soft, soft guard thresholds, emergency stop | no | no |
| Data & retention | project persistence, run retention, export/purge, audit retention | **yes** | no |
| Advanced | host allowlist status, origin checks, workspace root (read-only), diagnostics | **yes** | no |

**Autonomy modes (OPINION; default=1; backend `AutonomyMode` policy resolver required):**

| Mode | Name | Spawns | Side effects (write/shell/network/spend) | Cross-hive A2A | Memory promotion | UX |
|---|---|---|---|---|---|---|
| **1** | Supervised | planner/spawn allowed within caps; operator can pause | **every side effect requires approval** | approval | operator only | default; approval inbox active; stop always enabled |
| **2** | Guarded | planner free; operator notifications for spawns | side effects require approval; read-only auto | approval | parent completion + operator confirm | more autonomy, same consent for effects |
| **3** | Trusted | free within caps | allowlisted write/shell inside jail auto **with post-hoc ledger**; network/spend approval | approval for cross-hive; same-hive notify auto | parent completion; operator confirm for role promotion | requires explicit opt-in + jail; no silent bypass |
| **4** | Autonomous | free within caps | side effects auto **within jail + budget + mode-4 allowlist**; network/spend still gated unless explicitly allowlisted | policy matrix | parent completion, promotions audited | emergency stop always visible; no silent bypass of retention/blindness |

Mode changes are human-only, audited, and never apply retroactively to open approvals. The UI must show the
current mode in the header/settings, and every auto-approval must still produce a ledger row (append-only).

**Tutorial:** first-run overlay (dismissible, resumable from Help) with five real steps tied to real surfaces:
(1) what a run is / submit a prompt; (2) Hive vs Tree projections and selection; (3) where budgets/usage live;
(4) approvals and side effects (link to inbox, no fake approval); (5) memory/A2A inspectors (state the current
capability honestly). No fake progress, no autoplay, no claims of unavailable features; `Skip`, `Resume`, and
`Don't show again` persisted in settings; accessible dialog (`alertdialog`, focus trap, Escape closes without
action); links navigate without mutating; a "capability status" line comes from the same settings/status source as
the panels.

### 5.8 Chat and HUD changes (P0/P1)

- **Chat is the HyperAgent card, not an agent** (operator decision: do not silently turn it into a SuperAgent).
  Header: title, state, model/thinking if reported, budget chip, stop (active for **all** active run states —
  current predicate only allows `running`/streaming, `ChatCore.tsx:339-351`), hide button.
- **Composer** stays prompt-only until Settings/model selection exists; a compact run strip above the composer
  shows tool/approval counts and links to Run detail; no duplicate tool table (`ChatCore.tsx:274-308` removed).
- **HUD** keeps board 07 shape (460×44): connection chip, census (`N SUPERAGENTS · M WORKERS`, derived and
  labelled), approval count, up to two budget chips (agents/calls). Replace `98.2% GREEN` unless the server
  defines the metric (`execution.rs:653-696` computes completed/total; label `derived` or remove).
- **Stop** uses `isRunActive(run.status)` (`agentState.ts:121-123`) so planning/thinking/queued/awaiting
  approval are cancellable; show "cancelling…" until terminal.

---

## 6. File-level component / route / state / contract changes

### 6.1 Front-end files (new or changed), with current seam anchors

| File | Change | Current anchor / new |
|---|---|---|
| `web/src/state/agentView.ts` | **NEW**: `AgentClass`, `AgentViewNode`, `buildAgentView`, `classOf`, `labelOf`, `treeRows`, `filters`, `stateFamily`, card/density resolver | — |
| `web/src/state/store.ts` | Extend `UiState` with `view.projection`, `view.density`, `view.inspector`, `view.filters`, `settings` slice (autonomy mode, budget mode, tutorial), `budget`, `providerCache`, `memoryBank`, `mailbox`, `approvalsInbox`; add actions; extend handlers | `store.ts:62-63` (Nav/Tab), `:67-169` types, `:242-321` UiState/actions, `:1441-1497` handlers, `:325-331` fallback label, `:1764-1768` initial |
| `web/src/state/agentState.ts` | Add state-family map and card-class labels; keep 16-state vocabulary; no new states | `:9-26`, `:58-84`, `:111-123`, `:126-135` |
| `web/src/api/contract.ts` | Additive: `AgentClass` type/fields on `NodeEventPayload` (`agent_class`, `role`, `persona_id`, `depth`, `template_id`, `label`, `progress`); `AgentMessagePayload` `status/in_reply_to/answered_at`; new `BudgetPayload`, `WatchdogPayload`, `CacheUsagePayload`; new server events (`BUDGET_UPDATE`, `WATCHDOG_STATE/REPORT`, `MEMORY_WRITE`, `A2A_DELIVERED`) only when backend lands; new HTTP DTOs live in `web/src/api/views.ts` (no WS freeze for read models) | `:100-166` commands, `:195-330` payloads, `:385-411` events, `:213` `NODE_KINDS`; approval `args_preview` already at `:270-292` |
| `web/src/api/views.ts` | **NEW**: typed read-model client using `fetchWithBoundTimeout`: run budget, approvals ledger, memory banks, mailbox count, provider/cache detail, roster, settings | `web/src/api/http.ts:33-77` helper; `store.ts:928` existing pattern |
| `web/src/api/index.ts` | Expose view/read methods on `UiApi` (or separate `ReadApi`) so components/tests can inject a fake; keep `ci` boundary test | `web/src/api/index.ts` whole file |
| `web/src/flow/graph.ts` | Consume `AgentViewNode[]`; card registry by class; layout keyed by topology; edges by branch/hive; remove `kind`-filtering of hyperagent (keep anchor) and replay flattening | `:26-36`, `:99-141`, `:143-165`, `:167-188`, `:196-234` |
| `web/src/flow/lod.ts` | **NEW**: band resolver + hysteresis (`±0.04`), effective content = min(band,density), culling helper, cluster projection, animated-edge cap | — |
| `web/src/flow/nodes.tsx` | Replace 5 fixed renderers with **card registry**: `HiveChatCard`, `Hive/SuperAgentCard`, `AgentCard`, `SubagentCard`, `WatchdogCard`, `ToolCard`, `ErrorCard`, `ClusterCard`, `BranchBand`; remove dead `HyperAgentNode` (chat owns Hyper) or move to chat/ | `:141-275` (`SuperAgentNode :141-173`, `WorkerAgentNode :175-194`, `ToolAgentNode :196-224`, `BrokenRouteNode :226-243`, `HyperAgentNode :245-267`, `NODE_RENDERERS :269-275`) |
| `web/src/flow/Canvas.tsx` | Add `onlyRenderVisibleElements`, band subscription from zoom, cluster layer, edge animation cap; minimap content + below-1280 Overview button; keep camera owner; rewire dialog alternative | `:100-189`, `:351-435` (`ReactFlow`/`MiniMap`), `:445-459` zoom |
| `web/src/panels/AgentTree.tsx` | **NEW** treegrid (APG): columns, virtualization, roving tabindex, selection sync | supersedes `AgentHierarchy.tsx:21-81` |
| `web/src/panels/SwarmPanel.tsx` | Remove stat tiles + hero (`:38-107`); add root summary, Hive/Tree toggle, filters, state legend, census line | `:13-151`, `:21-27` counts, `:38-51` tiles, `:53-107` hero |
| `web/src/panels/RunDetailPanel.tsx` | **NEW**: summary + budget strip + tool ledger + approvals + failures/refusals + files + revisions | — |
| `web/src/panels/ApprovalInbox.tsx` | **NEW** queue; extends overlay; renders `args_preview`, from→to, TTL, consequence, dismiss/retry, ledger | `ApprovalOverlay.tsx:24-34`, `:164-364`; store `:1952-2077` |
| `web/src/panels/HudBadge.tsx` | approval count + two budget chips; GREEN removal/label | `:8-55` |
| `web/src/chat/ChatCore.tsx` | Stop predicate all active states; chat strip instead of ACTIONS table; Hyper chat header extras | `:132-175`, `:274-308`, `:339-351` |
| `web/src/panels/MemoryInspector.tsx` | **NEW** (replaces `AgentMemoryPanel.tsx:20-125`): phased scopes, trust, read-set, promote/purge | — |
| `web/src/panels/MailboxPanel.tsx` | **NEW** (replaces `AgentMessagesPanel.tsx:21-110`): ledger, status honesty, count/32, approval links, no compose | — |
| `web/src/panels/AgentProfilesPanel.tsx` | Derive built-ins from server (remove 5-id `BUILTIN_IDS` at `:23`); link into Settings; validate tools/description display | `:23`, `:34-190` |
| `web/src/panels/ModelsPanel.tsx` | Become Provider & Cache (§5.5); add cost/cache/`as_of`/wire-support-from-server; split independent errors | `:31`, `:48-260` |
| `web/src/panels/SettingsPanel.tsx` | Sections + modes + human-only fields + tutorial link; no fake controls; env-only labels where true | `:8-120`; new `web/src/settings/` components |
| `web/src/panels/TutorialOverlay.tsx` | **NEW** 5-step first-run guide (`alertdialog`, skip/resume) | — |
| `web/src/panels/BudgetStrip.tsx` | **NEW** seven meters + secondary tool-loop group + cap-hit refusal linking | — |
| `web/src/panels/AgentInspector.tsx` | **NEW** selected-agent inspector: identity/class source/role/parent/children/bank/A2A/metrics/actions | — |
| `web/src/App.tsx` | IA: nav destinations Workspace/Swarm/Run/Agents/Data/Models/Settings; tabs scoped to Workspace; Run detail route state; keep banners/toasts/overlays; remove Routing destination | `:29-102`, `:104-140`, `:125-135` |
| `web/src/panels/NavRail.tsx` | 7 destinations (add Run detail, remove Routing); roving focus/keyboard remains | `:9-16` |
| `web/src/panels/LibraryPanel.tsx` | Tabs scoped to Workspace; project identity from `/api/projects` (or hint); no fake chevron | `:8-20`, `:32-100` |
| `web/src/panels/RoutingPanel.tsx` | **REMOVE** as a destination; parent column moves to Tree; keep a developer-only disclosure if useful | `RoutingPanel.tsx` whole file |
| `web/src/styles/tokens.css`, `tokens.ts` | New card sizes (Subagent 280×88, Watchdog 320×120, BranchBand 720×44 — OPINION), density spacing/typography, status/budget colors only if token-approved; meta floor ≥9.5px | `tokens.ts:91-100` (type), `:166-179` (layout), `:185-188` chrome |
| `web/src/styles/index.css` / `motion.css` | One ambient loop maximum; event-driven transitions; reduced motion zero; remove duplicate ambient (`hive-breathe`, sheen, scan, free bees) or gate to one; add density utilities; focus ring ink-700 | `index.css:38-119` reduced motion, `:121-200` ambient; `motion.css:42-348` 12 keyframes |
| `web/tests/*` | New/updated acceptance tests (§6.4) | existing suite: `designParity.test.ts`, `contract.test.ts`, `state.parity.test.ts`, `agentsToolActivityMount.test.ts`, `apiBoundary.test.ts`, `noHardcoded.test.ts` |

### 6.2 Store/state contract (concrete)

Add to `UiState` (`store.ts:242-321` region):

```ts
readonly view: {
  readonly nav: NavSection;                    // + 'run'
  readonly tab: TabId;                         // workspace-scoped only
  readonly projection: 'hive' | 'tree';
  readonly density: 'compact' | 'standard' | 'detailed';
  readonly inspector: null | { kind: 'agent'|'run'|'approval'|'memory'|'mailbox'; id: string };
  readonly filters: { hideFinished: boolean; focusBranch: boolean; query: string; class: AgentClass | null; stateFamily: StateFamily | null };
};
readonly settings: SettingsState;              // + autonomyMode 1|2|3|4, budgetMode 'hard'|'soft', tutorial, a2aMode, memoryRetention (human-only read)
readonly budget: RunBudgetState;               // seven meters + per-meter source/provenance + asOf
readonly providerCache: ProviderCacheState;    // account/catalogue/usage/cache + asOf + errors
readonly memoryBank: MemoryBankState;          // phase A role list; phase B scopes
readonly mailbox: MailboxState;                // rows/count/status
```

New actions: `setProjection`, `setDensity`, `openInspector/closeInspector`, `setFilters`, `loadRunBudget`,
`loadProviderCache`, `loadMemoryBanks`, `loadMailbox`, `loadRoster`, `loadSettings`, `updateSettings`,
`loadApprovalsLedger`, `dismissApproval` (already), `loadTutorialState`, `setTutorialStep`. New event handlers:
`BUDGET_UPDATE` (if event), `WATCHDOG_*`, `AGENT_MESSAGE` appends to `mailbox` (not only version bump), memory
invalidation on write. Existing handlers that must change: `NODE_*` (`:1441-1493`) to carry class/role/label/
progress honestly; `AGENT_MESSAGE` (`:1494-1497`) to buffer rows; `RUN_CREATED/STATUS` (`:1354-1410`) to refresh
budget/provider views; `ERROR` (`:1301-1311`) to write refusal ledger rows (session-only labelled).

### 6.3 Backend/contract additions (only what the UI needs; additive-first)

| Area | Change | Anchor |
|---|---|---|
| Node identity | Producer fills `NodeEvent.label`, `progress`, `agent_class`, `role`, `persona_id`, `depth`; keeps `kind` for back-compat; class vocabulary additive (`subagent`, `remote`, `watchdog`, `hive`?) with `NODE_KIND_VOCABULARY` unchanged or version-bumped deliberately | `execution.rs:743-779`; `contract/src/event.rs:240-268`; web `contract.ts:237-246` |
| History | Extend snapshot graph nodes with `class/role/label` so replay does not flatten; bump snapshot schema; update `HISTORY_SNAPSHOT` route/tests | `graph.ts:196-234`; `core` snapshot writer/reader; `contract` history payload |
| Approval | Render `args_preview` (already produced) in UI; add ledger read route `GET /api/runs/{id}/approvals` (pending + decided); no protocol change | `ApprovalRequest` (`event.rs:478-523`); `tools_bridge.rs:481-497`; `http.rs` routes |
| Budget | Migration v12: `runs.calls_attempted`, `runs.cost_usd`, `runs.cost_status`, `runs.ended_at`; `provider_calls` per call/task/model/tokens/cache/cost; expose `GET /api/runs/{id}/budget` + extend usage detail; persist `RunAccounting.report()` at terminal | `budget.rs:235-305`; `execution.rs:698-719`; `db.rs:32-249`; `http.rs:512-529` |
| Memory | v11 bank tree per `04` §5; routes `GET /api/memory/banks?scope=`, `GET /api/memory/banks/{id}`, `POST /api/memory/promote`, `POST /api/memory/purge`; read path replaces `agent_memory(&role,8)` at task creation with bank set + snapshot; membership visible in events/read model | `execution.rs:856-886`; `db.rs:180-197`; `http.rs:804-840`; `04` |
| A2A | v11: `agent_messages` add `in_reply_to/status/answered_at`; parent inbox injection at turn start; operator reply route/command (`AGENT_MESSAGE_REPLY`) only after delivery; count field on list; roster read route `GET /api/agents/runs/{id}/roster` (operator only) | `db.rs:160-178`; `execution.rs:2417-2609`; `http.rs:674-702` |
| Watchdog/Remote | New `watchdogs` table + daemon scheduler + `WATCHDOG_*` state events + `WatchdogCard`; capabilities: cadence, target task/hive, owner, report text (bounded), pause/resume/report-now; no model loop per tick; reports delivered to owner mailbox/UI; API `GET/POST/PATCH /api/watchdogs` | new `core/src/watchdog.rs`; `db.rs` v12; `contract` events; `execution.rs` scheduler seam |
| Settings | v12 `settings` table (schema_version, key, value, updated_at, actor) + audit rows; `GET/PUT /api/settings` with named refusals; boot config remains env for key/port/path; engine reads mutable policy for autonomy/approval/A2A/budget display; human-only fields require confirmation and are not agent-writable | `config.rs:22-131`; `http.rs:512-529`; new `core/src/settings.rs` |
| Provider/cache | Provider routes return `as_of`, cache status; `provider_calls` table feeds per-agent/cache attribution; expose supported-model set (derive from `Model` enum/client) instead of UI hardcode; never expose key | `http.rs:877-945`; `ModelsPanel.tsx:31`; `provider` accounting |
| Contract tests | Every additive event/command needs a `COMMAND_MATRIX` row (frozen rules) + golden schema regeneration; HTTP read models do not need WS freeze but do need schema/contract tests | `contract/tests/ui_command_matrix.rs`; `golden_schemas.rs`; `schemas/contract-v1.json` |

**Contract strategy (OPINION):** use **HTTP read routes** for inspectors/ledgers/budget/cache/settings GET
(no WS freeze churn, easy loading/error states, bounded fetch). Use **WS commands** only for live agent/runtime
control that must be ordered with events (operator reply, watchdog pause, settings mutation that affects the
running engine, approval dismiss if it must be server-acknowledged). Every mutating HTTP response must use the
same refusal shape (`code/message/how_to_fix/retryable`) so the UI has one error vocabulary.

### 6.4 Acceptance tests (each names the file and the assertion; not run in this pass)

| Test file | Assertion |
|---|---|
| `web/tests/agentView.test.ts` (new) | `buildAgentView` is total over classes; no invented fields; label fallback = role·depth·tail; classSource `fallback` when absent; parent/depth/hive/branch derivation; no first-8 UUID identity; 64/600 synthetic nodes deterministic |
| `web/tests/agentCards.test.ts` (new) | Every `AgentClass` maps to exactly one card component (exhaustive switch); Hyper renders only as chat card; Subagent renders SubagentCard from `agent_class/role`; Watchdog card gates on capability flag with honest "not available"; no `Compile failed`/fake code |
| `web/tests/lod.test.ts` (new) | Band thresholds/hysteresis ±0.04; effective content = min(band,density); B0 renders clusters only; culling helper counts ≤120 mounted; layout key stable; selection does not change layout key |
| `web/tests/treegrid.test.ts` (new) | `role="treegrid"`, `aria-expanded`, root summary, keyboard Up/Down/Right/Left/Home/End/Enter/Space; selection syncs both ways; virtualization rowcount/index correctness |
| `web/tests/budgetMeters.test.ts` (new) | Seven meters exactly; each has source/provenance; unknown → `—`, never 0; 80 % warning; cap-hit maps to named refusal; calls/cost show "not reported" until route; tool-loop group separate; in-flight chip not a run meter |
| `web/tests/approvalInbox.test.ts` (new/update `approval.integration.test.ts`) | Queue rows; `args_preview` rendered when present and "preview unavailable" when absent; deny-first focus; Escape closes without deciding; expired/stale dismiss visible outside failure branch; no optimistic close; decided ledger row |
| `web/tests/memoryInspector.test.ts` (new/update `agentsToolActivityMount` style) | Phase A Role-only + honest banner; phase B five scopes render; read-set/truncation copy; promote one level only; purge confirmation + receipt; no free-text edit; failure never empty success |
| `web/tests/mailbox.test.ts` (new) | Rows grouped by branch pair; status "not reported" until schema; count/32; no unread badge; no compose; live append; approval link |
| `web/tests/providerCache.test.ts` (new/update ModelsPanel tests) | Six counters; cache ratio only when hit+miss; cost only when COMPLETE; `as_of`; wire-support from server not hardcoded; loading/error/refusal states; no key/DOM leak |
| `web/tests/settingsModes.test.ts` (new) | Four modes with policy matrix; default 1; human-only memory/retention fields disabled for non-operator; no fake toggles when route absent; settings changes confirm/refuse; audit note shown |
| `web/tests/tutorial.test.ts` (new) | Five real steps link to real surfaces; skip/resume persisted; no fake completion; focus trap/Escape; no claims of unavailable features |
| `web/tests/designParity.test.ts` (update) | Pins new card sizes/densities/chat card; board 05/07 composition; meta floor ≥9.5; reduced-motion `getAnimations()` = 0; one ambient loop max |
| `web/tests/contract.test.ts` / `state.parity.test.ts` (update) | New payloads decode/mirror; unknown fields tolerated; 16 states; event/command counts; no drifted duplications |
| `backend/crates/hiverouter-contract/tests/ui_command_matrix.rs` / `golden_schemas.rs` (update) | Every new frozen command has matrix row/ACK/refusal; golden schema regenerated deliberately; additive NodeEvent/approval/message fields validate; old payloads still decode |
| `backend/crates/hiverouter-core/tests/agent_view.rs` (new) | `NODE_*` publish carries label/class/role/progress when known and omits when unknown; no guessed kind; history snapshot preserves class/label |
| `backend/crates/hiverouter-core/tests/budget_read.rs` (new) | `GET /api/runs/{id}/budget` returns seven meters with nulls not zeros; cost only COMPLETE; attempt counter persisted; cap-hit refusal row correlates |
| `backend/crates/hiverouter-core/tests/memory_banks.rs` (new, after v11) | Bank tree invariants I1–I12 (`04` §4.2), role→bank migration, read set snapshot, truncation flag, promote one level only, purge receipt |
| `backend/crates/hiverouter-core/tests/watchdog.rs` (new) | Create/pause/resume/report-now; cadence tick produces one report per interval; no model call per tick; terminal/daemon restart recovery; capability refusal when disabled |
| `backend/crates/hiverouter-core/tests/a2a_reply.rs` (new) | Open Escalate appears in parent's next prompt once; answer marks `answered`/`in_reply_to`; Notify has no reply semantics; 32-cap refusal names budget |
| `backend/crates/hiverouter-core/tests/http_boot.rs` (update) | New routes present with origin checks; settings/watchdog/memory mutations refuse from disallowed origin; no key in responses |
| `backend/crates/hiverouter-provider/tests/accounting.rs` (update/new) | Report exposes settled Decimal only on COMPLETE; per-call cache fields recorded; no cost on INCOMPLETE |
| Commands | `cd web; npm.cmd run typecheck && npm.cmd test -- --run && npm.cmd run build` (scripts in `web/package.json`); `cd backend; cargo test -p hiverouter-contract -p hiverouter-core -p hiverouter-provider`; browser manual a11y/motion checks per `05` §4.5 (UNVERIFIED until run) |

---

## 7. Explicit removals and fixes of current lying/dead surfaces

Each item is a **delete or repair** decision, not a restyle. `VERIFIED` line anchors are current code.

1. **Swarm stat tiles + hero empty state** — delete `SwarmPanel.tsx:38-51` (5 tiles) and `:53-107` (hero);
   replace with root summary + hierarchy/tree + one honest empty line. (P0-1 in `05`; board 08.)
2. **Chat `ACTIONS` table** — delete `ChatCore.tsx:274-308`; replace with one-line run strip linking to Run
   detail (the single tool ledger). No second table.
3. **Agents-panel duplicate tool ledger** — remove the table from `AgentsPanel.tsx:65-72`; keep a link/count to
   Run detail. `ToolActivityPanel` becomes the Run-detail implementation.
4. **Dead `HyperAgentNode`** — the canvas filters `kind:'hyperagent'` (`graph.ts:151`), so `nodes.tsx:245-267`
   is unreachable; delete the canvas card and represent HyperAgent with the chat card (board 05). Keep the anchor
   node (`Canvas.tsx:167-189`).
5. **Permanent `—` metric chrome** — remove `ctx`/`score`/progress chrome that no producer fills
   (`nodes.tsx:183-189,216-219`), or fill `progress` from the additive producer; never render an empty progress
   track as if measured. Replace `Compile failed`-style copy with "error state" unless a real `error_code` exists
   (`BrokenRouteNode` already prefers `errorCode`; delete the unused fabricated string).
6. **`WorkerAgent` asserted from a defaulted kind** — `store.ts:1447-1450` defaults missing kind to `worker`;
   `store.ts:325-331` labels it `WorkerAgent <tail>`. Replace with `classSource:'fallback'` + role·depth·tail
   identity; never claim a tier the wire did not send.
7. **Replay flattening** — `graph.ts:196-234` forces every node to `kind:'worker'` and `task <id>` labels. Stop
   flattening: extend the committed snapshot with class/role/label and render the same card registry in replay.
8. **Chat Stop disabled during planning/thinking** — `ChatCore.tsx:339-351` only enables for `streamingRunId` or
   `runStatus==='running'`; use `isRunActive` (`agentState.ts:121-123`) so stop works in every active state.
9. **Stale approvals undismissable** — `ApprovalOverlay.tsx:303-332` renders Dismiss/Retry only inside the
   resolve-failure block; move recovery controls outside it and add an inbox; render `args_preview` (already in
   contract/producer, ignored by UI) at `:234-257`.
10. **`CHAT_PIN` with no control** — store has `pinChat`/`CHAT_PIN` (`store.ts:2103-2106`) and defaults
    `pinned:true` (`:1765`), but ChatCore only has Hide/Close. Board 07 fixes the chat at hive centre: remove the
    dead UI-facing `pinChat` path (keep `CHAT_PIN` as a back-compat ACK no-op) or add a real pin control in
    Settings; do not leave a command with no surface.
11. **Global library tabs that only affect Workspace** — `App.tsx:50-102` resolves tabs only in `workspaceView`,
    but `LibraryPanel.tsx:15-20` paints Workspace/Swarm/Routing/History everywhere; scope tabs to Workspace and
    remove them from other destinations (or make them real routes).
12. **Routing destination** — `NavRail.tsx:9-16` + `RoutingPanel.tsx` expose a debug parent-edge table as a
    product destination; fold the `Parent` column into Tree and remove the destination.
13. **Projects chrome with a local random UUID** — `store.ts:333-362` invents a localStorage project id and shows
    the constant name `HiveMind AI`; `ProjectsPanel` is read-only. Fetch `/api/projects` and show the real
    name/id, or remove project chrome from the library until a project route exists; no fake chevron.
14. **Model/thinking read-only display presented as current settings** — `SettingsPanel.tsx:64-77` shows env
    values without saying they are boot-only; label them "boot-configured; restart required" until
    `GET/PUT /api/settings` exists. No selector may appear before the route.
15. **HUD `GREEN %`** — `HudBadge.tsx:42-47` shows a percentage with no defined server metric label; the engine
    computes `completed/total` (`execution.rs:653-696`). Replace with budget chips + census labelled `derived`, or
    remove GREEN entirely.
16. **Dashboard/motion slop** — three ambient loops + free-flying bees + 12 concurrent animations measured READ
    in `05`/`08`; `motion.css` declares 12 keyframes. Keep at most one ambient loop, event-driven transitions,
    zero under reduced motion (`index.css:101-119`), and a test gate.
17. **8 px micro-labels** — below the design's own 9.5 px meta floor (`tokens.ts:98-99`); remove 8/8.5/9 px
    labels flagged in `05`/prior audit, replace with the mono-meta 11 px step or a measured 9.5.
18. **Profile built-in classification drift** — `AgentProfilesPanel.tsx:23` hard-codes 5 built-in ids while the
    server ships 7 personas (including `superagent`/`subagent`); derive built-in/imported from the server response
    (add a `builtin:boolean` field) instead of a stale UI set.
19. **Approval modal as the only decision surface** — replace the single-modal-only jump-scare with
    inbox+ledger+focused dialog; keep the existing nonce/TTL state machine (it is correct) and add a11y contract
    (focus trap/Escape/return focus); no auto-approve until a real sticky model exists.
20. **`role="application"` on the canvas without a full keyboard model** — `Canvas.tsx:281-283` sets
    `role="application"`; either implement the APG composite-widget keyboard contract and test it, or expose the
    canvas as a labelled region plus the Treegrid peer (`05` §2.11). Until then, the Tree is the accessibility
    path.
21. **Template insert available with no active run** — `LibraryPanel` offers templates; `TEMPLATE_INSERT` needs an
    active run (`ws.rs:797-831`). Disable with an explanation when `run===null` instead of letting the click
    produce a refusal toast.
22. **`DevGallery`** — dev-gated (`DevGallery.tsx:8-10`), keep; ensure it is never reachable in production
    (already `import.meta.env.DEV`), no removal needed.

---

## 8. Twenty unanswered operator questions — with options and recommendations

These are decisions the operator has not settled (from `OPERATOR-DECISIONS.md` deferrals, `05` §8 U1–U10, and
second-wave design forks). Each has options and a recommended default; all are **OPINION** unless the source is
cited.

1. **What is a "Hive" structurally?** (a) SuperAgent subtree in the current task tree (b) persistent named hive
   with its own bank/lifecycle independent of a run (c) project-level grouping. **Recommend (a) now**, with a
   persisted `hive_id` label; revisit (b) only when daemons/persistent memory need it.
2. **Is the HyperAgent the chat card or a separate executing node?** (a) chat card = root anchor (current; board
   05/07) (b) separate HyperAgent canvas card plus operator console (c) a real conversational HyperAgent in chat.
   **Recommend (a)**; add a plan/budget inspector behind the chat card. Adding an executing agent to chat is a
   runtime change, not UI (deferred by operator).
3. **What exactly is a Remote/Watchdog agent?** (a) a scheduled mini-run that reuses the planner/leaf engine per
   tick (b) a daemon outside run lifecycle with one deterministic job and bounded report (c) an external endpoint
   agent. **Recommend (b)**; one job/cadence/report, no model call per tick; model use only for report synthesis
   if a setting enables it.
4. **Is Subagent a new runtime tier or a role class?** (a) new `AgentTier::Subagent` + memory/card/tier (b)
   additive `agent_class` over Worker tier with narrow persona/allowlist (c) remove the label as a Worker alias.
   **Recommend (b)** for UI truth now; (a) is a runtime capability change that must be separately approved.
5. **How do cards satisfy the operator's six default facts without breaking the four-fact rule?** (a) Standard
   density with structured footer strip (recommended) (b) six facts in prose on every card (c) four facts only,
   inspector for the rest. **Recommend (a)**; the "four facts" is a prose budget, not a pixel limit.
6. **How many density presets?** (a) 3: Compact/Standard/Detailed (b) 2: Comfortable/Compact (c) per-card custom.
   **Recommend (a)**; persisted per projection; detailed is not the default.
7. **Are the LOD thresholds authoritative?** (a) adopt `05` B0–B4 with ±0.04 hysteresis and pin by test (b) tune
   from a measured 64/600-node capture before freezing (c) single adaptive band. **Recommend (a) now**, then
   validate at scale; thresholds are design choices, the bands' existence is required.
8. **Which projection is default at run start?** (a) Hive (board 07 identity) (b) Tree (density/readability)
   (c) split view. **Recommend (a)**, with a persisted per-operator toggle; Tree remains first-class.
9. **Where is the tool ledger's home?** (a) Run detail only, chat strip links (recommended) (b) Agents panel
   (W11) (c) both tables. **Recommend (a)**; two tables is a known defect.
10. **What can autonomy mode 4 auto-approve?** (a) nothing beyond mode 3; mode 4 only changes spawn/A2A cadence
    (b) allowlisted write/shell inside the jail with post-hoc ledger (c) all side effects except spend/network.
    **Recommend (b) but only after the jail/allowlist security lane signs off**; never silent, always ledgered.
11. **How are memory retention/visibility/blindness exposed?** (a) settings with a confirmation + audit row
    (human-only) (b) read-only display (c) hidden. **Recommend (a)**, clearly human-only, never agent-writable;
    changes are audited and require confirmation.
12. **Can the operator write memory?** (a) no writes at all; promote/purge only (b) operator-authored `fact`/
    `decision` entries with provenance (c) free-text editor. **Recommend (b) after v11**; no free-text mass edit;
    every row shows provenance/trust.
13. **Can the operator send A2A messages?** (a) read-only forever (b) reply to an open Escalate only (c) full
    compose/Notify. **Recommend (b) first**, then (c) Notify behind settings when a send route and delivery exist.
14. **What A2A status semantics should the UI show?** (a) `open/answered/expired` for Escalate + delivery-only
    Notify (b) unread/read (c) no status. **Recommend (a)**; no unread badge until a read model exists.
15. **Should agents get a roster/discovery tool?** (a) operator-only roster in UI (b) bounded `list_agents` tool
    for agents (c) full registry API. **Recommend (a) now**, (b) only when `send_to_sibling` delivery lands; no
    (c) (A2A spec doesn't standardize registries; `02` §6.3).
16. **Hard caps vs "no hard default budget"?** (a) keep hard $0.50/400k/40-call safety caps + soft warnings
    (recommended for now) (b) configurable hard/soft per run; default soft with a hard ceiling (c) no caps.
    **Recommend (b)** with (a) as the floor until the security lane approves otherwise; never present "unlimited"
    while preflight can refuse.
17. **Where does settled cost come from?** (a) provider accounting with documented prices, only when COMPLETE
    (recommended) (b) estimated from tokens (c) provider-invoiced later. **Recommend (a)**; estimates never shown
    as spend; unknown renders as cap + "spend not settled".
18. **Is model/thinking selection global or per-run?** (a) global default + per-run override before submit
    (recommended) (b) global only (c) per-agent. **Recommend (a)**; no mid-run model switch; selection must state
    "applies to next run".
19. **How are settings persisted and audited?** (a) DB table + append-only audit rows; env only for key/port/path
    (recommended) (b) config file only (c) env only. **Recommend (a)**; every human-only change is confirmed and
    audited; the UI reads the effective value, never the desired-only value.
20. **What is the tutorial's scope and lifetime?** (a) first-run 5-step overlay, resumable from Help, skippable
    (recommended) (b) always-visible onboarding panel (c) no tutorial. **Recommend (a)**; no fake completion, no
    claims about unavailable features, focus-trapped and reduced-motion-safe.

---

## 9. Top 10 design changes

1. **`AgentViewNode` is the one projection record** (`web/src/state/agentView.ts`) driving Canvas, Treegrid,
   inspector, HUD, Run detail; one card registry keyed by `AgentClass`; every entity (HyperAgent, Hive,
   SuperAgent, Agent, Subagent, Remote/Watchdog, ToolAgent, Error, Branch/Cluster) has exactly one defined
   representation.
2. **IA restructure:** Workspace, Swarm (Hive/Tree toggle), Run detail, Agents, Data, Models, Settings; Routing
   folds into the Tree; tabs are Workspace-scoped; no dashboard destination.
3. **LOD bands B0–B4 with ±0.04 hysteresis, culling, batched B0, animated-edge cap, stable topology-keyed
   layout** — the canvas survives 150–600 agents; acceptance gates from `03` §9.4.
4. **Density presets (Compact/Standard/Detailed)** separate from LOD, with Standard as the operator default
   (role/task/state/cost/tools/branch via structured strip); every unknown is `—`.
5. **Approval inbox + focused decision dialog** rendering `args_preview` (already produced), from→to narrative,
   TTL/nonce/consequence, deny-first, expired/stale Dismiss, decided ledger; no optimistic close, no auto-approve.
6. **Memory inspector with the bank-tree scope ladder**, phased: Role-only + honest banner now; five scopes,
   trust, read-set/truncation, promote/purge after v11; no free-text editor.
7. **Mailbox/roster:** branch-pair ledger, Escalate/Notify semantics, status honesty ("not reported" until
   schema), `messages N/32`, operator reply only after a reply route; no chat-style compose.
8. **Seven run-budget meters + Provider & Cache panel** from real sources with provenance: agents/64, depth/3,
   fan-out/8, calls/40, tokens/400k, cost/$0.50 settled-only, messages/32; tool-loop and in-flight separate;
   cache hit/miss and cost settlement honesty; no fake spend.
9. **Settings/autonomy modes 1–4/tutorial:** DB-backed, audited, human-only retention/visibility fields; four
   explicit modes with a policy matrix; realistic 5-step onboarding; no controls that do nothing.
10. **Remove the lying/dead surfaces and fix the identity chain:** stat tiles/hero, duplicate tool tables, dead
    HyperAgent card, default kind/labels, replay flattening, stop gating, `CHAT_PIN`, Routing destination, global
    tabs, local-UUID project chrome, 8 px labels, ambient motion excess, model/thinking read-only fiction.

---

## 10. Risks, limitations, and what is not verified

| # | Risk / limit | Impact | What would settle it |
|---|---|---|---|
| R1 | The persona fix is **uncommitted** and does not cover `worker`/planner personas; the UI identity chain depends on it plus additive `label/class/role` fields | Card identity/persona may still be generic | Land + test `provider_executor` change; add producer fields; `agent_view.rs` test |
| R2 | Remote/Watchdog, settings, budget/cost persistence, memory banks, A2A status/reply all require backend work not in the tree | UI can only ship capability-gated placeholders; risk of overclaiming | Implement the additive contract/routes/migrations named in §6.3; keep "not available on this build" copy |
| R3 | 150–600 agent LOD/culling is untested in this codebase; React Flow + dagre per event is a known cost (`03` §9) | Canvas may not meet perf gates; Tree may be the practical view at scale | 64-node today + synthetic 600-node harness with `mountedNodes/animatedEdges/fps` measurements |
| R4 | The seven meters' calls/cost are not persisted/exposed today; only tokens/messages/agents/depth/fan-out are derivable | Cost/calls meters would show `—`; no fake allowed | Migration v12 + `GET /api/runs/{id}/budget` + provider accounting report |
| R5 | Approval `args_preview` is absent on post-hello replay (persisted row has digest only — `event.rs:508-516`; `tools_bridge.rs:481-493`) | Dialog must say "preview unavailable" on replay; never reconstruct | Render the field when present; add a test for replay absence |
| R6 | Settings/autonomy modes change security behaviour; models 3–4 auto-approval needs jail/allowlist hardening from `06` T10/T12 | Auto mode could widen side-effect exposure | Security lane sign-off + `shell.rs`/`jail.rs` fixes + approval coverage test (`06` §5.2) |
| R7 | No browser/e2e tooling decision (U8); a11y/motion gates are unit/DOM assertions only this pass | A11y claims remain partially unverified | Pin Playwright/axe (or equivalent) + fake-provider fixture; run reduced-motion `getAnimations()` gate |
| R8 | HMARS definition is operator-level; "Hive/branch" semantics are design decisions here, not code facts | Different lanes may use different meanings | Operator answers Q1/Q2/Q3 in §8; record in `OPERATOR-DECISIONS.md` |
| R9 | Current tree is moving (uncommitted provider change, docs edits) | Line anchors drift | Re-hash before implementation; the evidence index records the hashes read |

**Explicitly not verified in this lane:** live rendering of any new surface; performance at 64/600 nodes; the
uncommitted persona fix at runtime; provider balance/cache live values; watchdog/settings/memory/budget backends;
browser a11y/motion gates. No code was run.

---

## 11. Evidence index (files most load-bearing for this design)

| File | Bytes / hash (read this pass) | Used for |
|---|---|---|
| `docs/research/hmars/05-ui-ux-patterns.md` | 97,666 B | P0/P1/P2, LOD, card v2, approvals/memory/mailbox, IA, 20 Q&A, U1–U10 |
| `docs/research/hmars/07-code-backend.md` | 79,182 B | tiers, caps, A2A, memory, gaps G1–G18, seams S1–S12 |
| `docs/research/hmars/09-synthesis-and-goal.md` | 17,632 B | target HMARS, seven meters, P0–P5 plan, UI truth |
| `docs/research/hmars/04-memory-architecture.md` | 117,041 B | bank tree, read policy, v11 schema, invariants I1–I12 |
| `docs/research/hmars/02-hmars-patterns.md` | 98,363 B | tier/authority model, three control verbs, registry, P0/P1 |
| `docs/research/hmars/03-scale-concurrency.md` | 72,122 B | UI virtualization, acceptance thresholds §9.4 |
| `docs/research/hmars/06-security-cost.md` | 116,163 B | approval authority, cost formula/budget layers, T07–T12 |
| `web/src/api/contract.ts` | 38,086 B, `4E66EB111F8B7FB2` (hash prefix) | commands/events/payloads, `args_preview`, node fields |
| `web/src/state/store.ts` | 80,400 B, `C6F940DF3E3C6AAA` | UiState/actions/handlers/fallback/project |
| `web/src/App.tsx` | read | IA, tabs, ControlBar/Scrubber/overlay mounts |
| `web/src/flow/graph.ts` / `nodes.tsx` / `Canvas.tsx` | 8,442 / 11,311 / 19,745 B | node projection, card registry, ReactFlow/minimap/mocks |
| `web/src/panels/{SwarmPanel,AgentHierarchy,AgentsPanel,AgentMemoryPanel,AgentMessagesPanel,AgentProfilesPanel,ApprovalOverlay,ModelsPanel,SettingsPanel,HudBadge}.tsx` | read | current panels and truth gaps |
| `web/src/chat/ChatCore.tsx` | 16,134 B | chat card, ACTIONS, stop predicate, composer |
| `web/src/tokens/tokens.ts` / `web/src/styles/{index,motion,tokens}.css` | read | geometry/type/motion/density constraints |
| `backend/crates/hiverouter-core/src/{execution,caps,tool_loop,db,tools_bridge,provider_executor,config,http}.rs` | read | caps, events, memory, A2A, approvals, settings, routes |
| `backend/crates/hiverouter-provider/src/budget.rs` | 710 lines | seven-meter denominators, cost/token/call caps |
| `backend/crates/hiverouter-contract/src/event.rs` + tests | read | NodeEvent/ApprovalRequest/AgentMessage schemas, golden/matrix tests |
| `docs/ui/figcraft/ASSETS/hivemind-{agent-cards,hmas-hierarchy,hmas-governance,run-detail-and-files}.png` | READ as images | card anatomy, hierarchy/topology, governance, run-detail layout |
| `web/package.json` / `web/vite.config.ts` | read | scripts (`typecheck`, `test`, `build`), vitest node include pattern |
| `backend/crates/hiverouter-core/tests/*`, `contract/tests/*`, `provider/tests/*` | READ list | acceptance-test naming/harness patterns |

**Line anchors for the most-cited current code:** `execution.rs:97-106` (node kind), `:151-162` (tier map),
`:743-779` (publish node/spawn), `:856-886` (memory seed), `:1016-1054` (submit), `:1309-1400` (plan/root/spawn),
`:1595-1754` (tool turn), `:2417-2609` (A2A), `:3017-3026` (memory write); `caps.rs:9-15`; `tool_loop.rs:13-21`;
`budget.rs:24-52,235-305`; `db.rs:32-249` (migrations), `:118-119,652-703` (tool counters), `:198-215`
(run_usage); `http.rs:512-529` (routes); `contract.ts:100-166` (commands), `:195-330` (payloads), `:385-411`
(events), `:213` (kinds), `:237-246` (NodeEvent), `:253-315` (Approval/AgentMessage); `store.ts:62-63,67-169,
242-321,325-331,1441-1497,1764-1768`; `graph.ts:26-36,143-234`; `nodes.tsx:141-275`; `Canvas.tsx:167-189,
351-435`; `ApprovalOverlay.tsx:24-34,234-257,303-332`; `SwarmPanel.tsx:13-151`; `ModelsPanel.tsx:31,48-260`;
`SettingsPanel.tsx:64-77`; `AgentProfilesPanel.tsx:23`; `ChatCore.tsx:274-308,339-351`; `tokens.ts:91-100,
166-179`; `index.css:101-119,137-142`; `motion.css:42-348`.

---

## 12. Bottom line

The first wave designed the right surfaces but did not resolve the **identity chain**, the **evidence homes**, or
the **capability gates** that make those surfaces buildable. This lane's goal is the minimum coherent redesign:
one `AgentViewNode` projection, one total card registry that includes Subagent/Remote/Watchdog/Hive/Branch, one
Hive/Tree pair with LOD and density, one Run-detail evidence surface, one approval inbox, one memory inspector,
one mailbox, seven real meters, a provider/cache panel, and settings/modes/tutorial that only exist when the
backend does. Everything else is subtraction: fewer duplicate tables, fewer dead cards, fewer invented metrics,
fewer fake controls. That is the shortest path from the current honest-but-blind run monitor to a true HMARS
control surface.
