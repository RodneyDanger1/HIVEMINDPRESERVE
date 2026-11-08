# VERIFIED - independent verification of the agent-class taxonomy

**Status:** verifier report. Not a specification, not an implementation plan.
**Verifier snapshot:** HEAD `6e3294c` ("fix: template authority mapping and verified taxonomy") with
a clean backend worktree (**S4**). Code is unchanged from `de0f4e6`; `6e3294c` updates the template
tests and commits these docs. `execution.rs` is 3,430 lines, `ws.rs` 1,435. The tree moved five
times during the pass (`bb95a44` -> `aa15ec9` -> `66aff50` -> `45914dc` -> `de0f4e6` -> `6e3294c`);
every line number below is from the S4 read and will drift again.
**Method:** read-only source and document inspection. **No** runtime, build, test, server, UI,
provider call, migration, or device was executed. Tags: **[SR]** source-read, **[GV]** grep-verified
presence/absence, **[DR]** repo document statement, **[WR]** official external page. There is **no
[VR]** tag: nothing here is runtime-verified.
**Verdict:** **ACCEPT WITH CORRECTIONS.** The draft's central diagnosis is right and independently
reproduced: one free-form `tasks.role` string is doing four jobs (label, rank selector, capability
selector, memory key), no persisted field records agent class, Subagent is a profile-only rank on the
Worker tier, and Remote has zero code. But the draft is **not safe as an evidence base without the
corrections below**: two of its three Critical findings (F4, F8) are fixed (F4 by the compact
persona match, F8 by S4's template-id-to-spawn-class mapping); the third (F3) is **latent at
admission**, not a live promotion; F7 is half-fixed. The real defect is the string-derived identity
itself (F1/F3/F5), not a live privilege path. F8's residual is narrower: a `"subagent"` template is
still admitted at depth 1 (the F6 depth-floor gap), not a Hyper path.
The five-class taxonomy is implementable, but two definitions need amendment: Subagent cannot be an
*enforced rank* without a persisted class (a profile + depth rule is the honest minimum), and the
Remote memory clause cannot be implemented before a global/system bank exists.

---

## 1. What changed since the draft (the short version)

| # | Draft claim | Current S4 truth | Effect |
|---|---|---|---|
| 1 | F4/C4: `"sub_agent"` bypasses the Subagent persona and gets the 9-tool Worker grant. **Critical.** | `persona_for_role` adds a compact id match (`execution.rs:600-609`, committed in `45914dc`): `sub_agent`/`sub-agent`/case variants resolve to `subagent`, which has the 6-tool grant. | **Refuted.** F4 drops Critical -> Medium; the residual is the latent unknown-role fallback (F3), reachable only for legacy/direct rows or a spawn source that skips admission. |
| 2 | F8/C7: `TEMPLATE_INSERT` bypasses `validate_spawn_role`; a depth-1 Hyper is one command away. **Critical.** | S3 validated the role; **S4 maps the template id to a spawn class** (`ws.rs:248-261`): `planner`/`hyper` refused, `superagent` -> Super, `subagent` -> Subagent, every other id -> `worker`. Library templates are admitted again; metadata is written (`ws.rs:305-311`); `6e3294c` updates the test (`content-writer` inserts, `planner` refused). | **Refuted for Hyper.** Residual: a `"subagent"` template is still admitted at depth 1 (no depth parameter). |
| 3 | F7/C6: root turn runs only for aggregating plans; a flat plan gets no HyperAgent answer. | `execution.rs:1686` runs the root for `has_aggregators || plan.nodes.len() > 1`. A flat **multi-node** plan does get a root turn; only a **single-node** plan does not. | **Half-refuted.** F7 remains a real but narrower gap. |
| 4 | §3.8: execution order is leaf topological, then aggregators deepest-first. | Readiness loop: a node runs when its deps and all direct children are complete, then the spawn queue, then the root (`execution.rs:1559-1758`). | Draft S2 already noted this; the body now says both S1 and S2. Content correct. |
| 5 | §5.1: `RUN_METRICS` counts hyperagents. | `RunMetrics` has only `superagents`/`workers` (`event.rs:274-284`; `execution.rs:708-747`). Hyperagent count is web-local (`SwarmPanel.tsx:84`). | Draft error, patched. |
| 6 | §4: ToolActor is "engine-created for approved tool work". | No public path creates a `ToolAgent` task; approved tool work executes under the caller's tier. The tier exists, the producer does not. | Draft error, patched; it is a dormant allowlist, not a class. |
| 7 | §3.6/§5.1: "There is no run-digest writer" (read as "no run bank"). | Since `aa15ec9`, `engine`/`prompt` and `engine`/`plan` rows are written to the `"run"` scope (`execution.rs:1462-1485`), and tasks re-read run+parent banks at start (`:3005-3033`). | Narrow correct: no **result** digest, but the run bank is not empty. |
| 8 | F14: ownership claim at `21:840-985`. | `21` is a design proposal there and says ownership is absent at `21:347`; the operator requirement is real (`OPERATOR-DECISIONS.md:41-42`). | Misattribution, patched; finding stands. |

The draft's S1/S2 snapshot distinction is legitimate, and its S2 anchor table is approximately
correct for `66aff50`. For S3/S4, add +7 to every `execution.rs` line after 609 (the compact-match
block) and +12 to `ws.rs` lines after 248 (the template mapping); the table in §4 below is
authoritative.

---

## 2. Verified taxonomy

Five words, sorted by the properties that actually separate them (accountability, lifetime, unit of
work), plus the non-class ToolActor.

| Class | What it actually is in S4 | Enforced by | Implementable now? | Target statement |
|---|---|---|---|---|
| **HyperAgent** | The single root task: `role="planner"`, depth 0, no parent, tier `HyperAgent`, 3-tool allowlist never advertised to the model. It produces the run answer in an answer-only root turn for multi-node plans. | Tier inferred from the role string; plan gate `validate_spawn_role(Hyper, …)`; root row inserted before children; root turn only via `finalize_root`. | **Yes**, if a class field and one root-turn policy are added. One-instance and no-parent already hold; "always answers" does not (single-node plans). | Run owner: plan acceptance, branch-result adjudication, one final answer. Answer-only, narrowest tools, root-only, not a chat persona, not a long-lived process. |
| **SuperAgent** | A depth-1 task with tier `SuperAgent` (7 tools: spawn/ask/sibling/report/finish/read/list) and the `superagent` profile (max 8 children, max depth 3). It can spawn Agent and Subagent only; no write/exec; one branch result. | Role string -> tier; `validate_spawn_role`; persona max_children/max_depth applied to its own children; no Super->Super. | **Yes** as a branch lead. The four ownership modes/offices are **not** implementable without new columns/tables; do not claim them. | Branch/hive lead with an assigned scope, exclusive ownership, one branch result, no writer/shell, no second root, no Super peer in the same scope. |
| **Agent** (regular; wire `worker`) | A leaf task with tier `WorkerAgent` (all 9 tools, write/exec approval-gated). It may spawn Subagent only. It has **no built-in profile** named `agent`; it is the resolver's fallback tier for unknown roles, but every production spawn source validates first, so a new unknown role is refused and the fallback is latent. | Role string -> tier; approval gate; `validate_spawn_role`; persona narrowing only when the role matches a profile. | **Yes**, after adding a real `agent` profile and refusing unknown roles in the resolver itself. | One assigned task, one bounded result, all workspace tools behind approval and the jail, may spawn Subagent only, never a Super/Agent/tool actor, never the default for arbitrary strings. |
| **Subagent** | A **profile-only** class: role `subagent` (or a compact-equivalent alias) with tier `WorkerAgent` and a 6-tool grant (ask/sibling/read/list/report/finish). No spawn/write/exec at call time. **No depth floor, no wire kind, no class field.** | Persona lookup at seed (`execution.rs:588-609`, `:918-953`); call-time gate (`:1091-1105`); advertised set is still the 9 Worker tools. | **Partially.** As a profile + admission depth rule: yes. As an *enforced fourth rank*: no, without a persisted class and a contract kind. The alias fix is committed in `45914dc`. | Narrowed leaf at depth >= 2, no spawn/write/exec by grant **and** admission, no children, precise bounded result, visible as a distinct card/kind (or the taxonomy drops the claim). |
| **Remote tiny agent** | **Nothing.** Zero `remote_*`/`device_*`/`daemon` symbols in backend source; no table, route, event, card, or credential binding. The design is a tree-free capability **principal** with its own bank and read-only global/system memory. | Nothing. Design docs only. | **No, not in the current task/run model.** It needs a separate record lane (tables + roster projection + supervisor). It must never be a `tasks` row or a `NODE_*`. The "global memory read" clause is blocked until a global bank exists. | Operator-enrolled principal outside any run: owns devices/capabilities via per-action grants, reads `{global, own}` only, writes only its own quarantined bank, no task/depth/tree edge/cap consumption, no inbound listener, no actuation replay. |

**Non-classes (do not let a card or a name promote them):**

- **ToolActor / `ToolAgent` tier:** a tier and a 4-tool allowlist (`tool.rs:422-430`) with **no
  producer**. `task_agent_tier` maps `"tool"`, but the plan loop, `SPAWN_SUB_HIVE`, `spawn_agent`,
  and `TEMPLATE_INSERT` all reject it. It should be either removed or explicitly an actor, not a rank.
- **Card:** a UI projection, never authority (`web/src/flow/nodes.tsx:302-310`; `graph.ts:235`).
- **Kind:** a 4-value contract projection of tier (`event.rs:240`); `subagent` exists only in the web
  union (`contract.ts:213`) and has no producer.
- **Profile:** policy data (`personas.json`; `prompts.rs:381-435`), not identity; `tier` is never
  read for authorization (`http.rs:943`, `:1002` only serialize it).

---

## 3. Corrected operational definitions

These are the definitions the code can support. Each has an explicit code mapping and the one thing
that must be added to make it real.

### 3.1 HyperAgent

**Is:** the one run-scoped accountable task. Root row `role="planner"`, `depth=0`,
`parent_task_id=NULL`, tier `HyperAgent`; 3-tool allowlist (`spawn_agent`, `report_result`, `finish`);
answer-only root turn through `finalize_root` with no tool specs; plan gate validates every planned
child against the parent tier. **Lifetime:** one run; ends with the run's terminal status. **Memory:**
reads run scope + parent facts + at the root turn up to 32 rows across all scopes; writes its own
final text under its own task-id scope (and the engine writes `engine`/prompt + `engine`/plan to the
`"run"` scope). **Communication:** no outbound verbs; child results arrive as prompt text.
**Spawn:** may accept Super/Agent/Subagent via plan or `SPAWN_SUB_HIVE`; may not accept Hyper/Tool.
**Is not:** a super-user (it has the narrowest tool list), a process, a chat persona, a spawned
child, or a tool user.
**Current gaps:** single-node plans get no root turn; one per run is enforced by admission, but the
class is still a string; S4's template mapping closes the Hyper path and keeps the Library templates,
but a `"subagent"` template is still admitted at depth 1.

### 3.2 SuperAgent

**Is:** a branch/hive sub-orchestrator expressed as a depth-1 task with tier `SuperAgent` and
profile `superagent`; 7 tools, no write/exec; spawns Agent/Subagent only; reports one branch result
to the root; its own task-id memory scope; parent/sibling messages only. **Lifetime:** its task (no
office outlives it). **Is not:** a second root, a writer/shell user, a Super->Super parent, a peer
owner of the same scope, an idle "office" process, or the ChatCore persona.
**Current gaps:** no ownership/branch/scope field; the four operator modes are design-only; the
persona's `max_children`/`max_depth` are enforced only for its own children (`execution.rs:1189-1196`),
which is correct but does not make it an owner.

### 3.3 Agent (regular Agent; wire `worker`)

**Is:** a leaf executor with tier `WorkerAgent` (9 tools); write/exec require approval; spawns
Subagent only; one task, one bounded result; own task-id memory scope; the only class expected to
mutate the workspace. **Is not:** a sub-orchestrator, a Subagent, a Super/Agent spawner, an approver,
or the default for arbitrary role strings.
**Current gaps:** no `agent` profile; the resolver still maps **any unrecognized role** to this
maximally-capable tier (`execution.rs:183`), but every production insert validates first
(`:1180`, `:1509`; `ws.rs:260`), so F3 is latent at admission rather than a live promotion.

### 3.4 Subagent

**Is:** a narrowed profile-only class on the `WorkerAgent` tier: 6 tools, no spawn/write/exec at
call time, no children, depth-design 2-3. **Is not:** a fifth tier, a Worker with a different prompt,
depth 1, a dead card pretending to be a wire kind, or an alias that escapes narrowing.
**Current gaps:** no depth floor (`validate_spawn_role` accepts it under Hyper at depth 1, and the
S4 template mapping still routes a `"subagent"` template to that same depth-1 check); no wire
kind (contract vocabulary is 4 values); no class field; the narrowing depends on the persona
lookup, which S3/S4 canonicalizes (committed in `45914dc`). **Implementable version:** profile +
admission depth rule + per-instance advertised grant + a contract-delta kind; do not add a fourth
`AgentTier`.

### 3.5 Remote tiny agent

**Is (design only):** a tree-free capability principal outside any run; owns devices via a closed
capability inventory (`device.light.switch` etc.); per-device/per-action grants; reads exactly
`{global:default, remote:<own_id>}`; writes only its own quarantined bank; one supervisor lane
leases/probes/reconnects; crash recovery never replays actuation. **Lifetime:** across runs; states
`enrolled -> online|degraded|offline|quarantined|revoked -> retired`; never a task state.
**Is not:** a task row, a depth, a `NODE_*`, a tier, a rank, a daemon with a model loop, a raw
HTTP/shell proxy, a writer to global memory, or a reader of run/hive/branch/role banks.
**Current gaps:** everything. The design (docs `00`-`06`) is detailed enough to build, but the
current `tasks.run_id NOT NULL`/single-active-run model cannot host it; it needs new tables and a
roster lane. **Definition that cannot be implemented today:** the memory clause, because there is no
global/system bank (`db.rs:251-274` only has run/scope memory).

---

## 4. Evidence table (current, S4)

All line numbers re-read at the S4 snapshot. "Draft anchor" shows what the draft claimed, where it
differs.

| # | Claim | Current evidence | Status | Draft anchor |
|---|---|---|---|---|
| E1 | One enum, four tiers, no Subagent variant; labels are `*_agent`. | `tool.rs:72-81`, `:85-93` | Verified | `:72-81`, `:88-92` (off by 3) |
| E2 | The resolver falls back to `WorkerAgent` (9 tools) for an unknown role; every production insert validates first, so a new unknown role is refused. | `execution.rs:173-185` (fallback), `:1180`/`:1509` (validation), `:1208`/`:1528` (insert); `ws.rs:260`/`:303`; `tool.rs:408-421` | Verified (latent) | `:151-163`/`:161` stale; draft called it live |
| E3 | `subagent` has no tier arm; it is Worker by fallback. | `execution.rs:173-185` | Verified | `:151-163` stale |
| E4 | Persona lookup is exact -> lowercase -> `_` normalization -> compact id match. | `execution.rs:588-609` (S3/S4, committed in `45914dc`) | Verified | `:566-580`/`:588-602` |
| E5 | `sub_agent`/`sub-agent`/case variants resolve to the `subagent` profile. | `execution.rs:600-609`; `personas.json:32` | Verified source-read | Draft said they escape |
| E6 | Subagent profile has 6 tools, no spawn/write/exec. | `personas.json:31-39` | Verified | correct |
| E7 | Call-time gate intersects tier and persona; empty matched grant = no tools. | `execution.rs:1091-1105`; `personas.json:7` | Verified | `:1060-1074`/`:1057-1061` stale |
| E8 | Model is advertised the tier set, not the effective grant (Subagent sees 9). | `provider_executor.rs:335-351`; `tool_loop.rs:206-219` | Verified | `:333-349` (off by 2) |
| E9 | Spawn matrix: Hyper -> Super/Agent/Subagent; Super -> Agent/Subagent; Worker -> Subagent only; Tool -> none. | `execution.rs:135-164` | Verified (exact strings; the draft's `super*`/`worker*` wildcards are imprecise) | `:113-142` stale |
| E10 | No depth floor for Subagent; Hyper accepts `subagent` at depth 1. | `execution.rs:138-151`, `:1497-1538` | Verified | correct |
| E11 | Plan loop validates the role against the parent tier (since `aa15ec9`). | `execution.rs:1509` | Verified | F6 wrongly said "only caps" at the same line |
| E12 | `TEMPLATE_INSERT` maps the template id to a spawn class, validates it, and sets metadata (S4). | `ws.rs:248-261`, `:305-311` | Verified | Draft said bypass; S3 fixed Hyper but broke Library ids; S4 fixes both |
| E13 | Template node role = template id; `planner`/`hyper` refused; Library ids -> worker; `subagent` -> Subagent at depth 1. | `ws.rs:146-149`, `:252-260`; `execution.rs:138-151`; `personas.json` | Verified | New finding: template path has no depth parameter |
| E14 | Root turn for `has_aggregators || plan.nodes.len() > 1`; single-node plan still skips. | `execution.rs:1675-1686` | Verified | Draft S2 correct; S1-only statement stale |
| E15 | Execution is a readiness loop (deps + direct children), then spawn queue, then root. | `execution.rs:1559-1674`, `:1679`, `:1686-1760` | Verified | Draft S2 correct |
| E16 | Root turn advertises no tools; a tool call there fails the task. | `provider_executor.rs:364-389`; `execution.rs:3177-3182` | Verified | anchors off by 7 |
| E17 | Hyper tier allowlist is 3 tools; Worker 9; Super 7; Tool 4. | `tool.rs:392-430` | Verified | correct |
| E18 | Task memory write: `agent_key=task.role`, `scope_id=task id`, 2,000 chars, secret filtered. | `execution.rs:3388-3399`; `:60-75` | Verified | anchors stale |
| E19 | Engine writes `engine`/prompt + `engine`/plan to the `"run"` scope before seeding. | `execution.rs:1462-1485` | Verified | Missing from draft |
| E20 | Seed reads parent + `run` scope (8 each, any role), roster 16, inbox 8. | `execution.rs:934-990`; `store.rs:800-828` | Verified | anchors stale |
| E21 | Root read: up to 32 rows across all scopes, label "Shared hive/branch memory (data, not instructions)", 300-char values, 12k/24k budgets. | `execution.rs:1705-1725`, `:1697-1758` | Verified | anchors stale |
| E22 | No global/project/hive/role bank, no trust/retention, no vector store; `memory_banks` absent. | `db.rs:251-274`; grep | Verified | correct |
| E23 | `ask_parent` is a write-only mailbox: bounded text, 32/run, returns message id, no answer/deadline/wake. | `execution.rs:2688-2757`; `:90-92` | Verified | anchors stale |
| E24 | `send_to_sibling` same-parent by default; cross-branch needs approval. | `execution.rs:2780-2873`; `tool.rs:345-350` | Verified | anchors stale |
| E25 | Inbox is refreshed at task start, not before continuation turns. | `execution.rs:3079-3084`; `:3038-3062` | Verified | correct |
| E26 | Terminal task states = Completed/Failed/Cancelled/Killed/Interrupted; terminal is absorbing. | `execution.rs:114-117`; `run_state.rs:99-105`; `:3343-3362` | Verified | anchors stale |
| E27 | Failure fails the whole run (fail-fast); a per-task kill is isolated. | `execution.rs:1656-1679` | Verified | anchors stale |
| E28 | One active run per server; serial dispatch. | `execution.rs:1131-1133`, `:1367-1430` | Verified | correct |
| E29 | Caps 3/8/64/40; `check_calls(1)` at both sites, so the call cap cannot fire. | `caps.rs:9-15`, `:66-71`; `execution.rs:1435`, `:3129` | Verified | anchors stale |
| E30 | Node kind = pure function of tier; contract vocabulary is 4; test pins 4. | `execution.rs:121-128`; `event.rs:240`, `:915-924` | Verified | test range off by 2 |
| E31 | No server path emits `subagent`; the web ships a dead `SubagentNode`. | `event.rs:240`; `http.rs:740-743`, `:1034-1040`; `nodes.tsx:196-219`; `contract.ts:213` | Verified (source-read) | correct |
| E32 | Web coerces absent/unknown kind to `worker` on hydration, replay, and NODE_STATE. | `store.ts:652`, `:1548-1551`; `graph.ts:305` | Verified | draft named 2 of 3 |
| E33 | Root is filtered from canvas and anchored as chat; HyperAgentNode never placed. | `graph.ts:121-126`, `:234-235`; `nodes.tsx:270-292` | Verified | correct |
| E34 | `RUN_METRICS` has only `superagents`/`workers`; hyperagent count is web-local. | `event.rs:274-284`; `execution.rs:708-747`; `SwarmPanel.tsx:84` | Verified | Draft said metrics.ts counted hyperagents |
| E35 | `persona.tier` is never read for authorization; only serialized. | `prompts.rs:389`; `http.rs:943`, `:1002`; grep | Verified | anchors 950/1009 stale |
| E36 | `ToolAgent` tier has no producer on any spawn path. | `execution.rs:157`, `:180`; `ws.rs:252-260`; grep over spawn paths | Verified | Draft called it engine-created |
| E37 | Zero remote/device/daemon symbols/records/routes. | grep over `backend/crates/*/src`: only unrelated "remote timing oracle"/"device prefix" hits | Verified | correct |
| E38 | Roster is over tasks: `GET /api/agents/runs/{run_id}/roster`; handler ~1019. | `http.rs:536`, `:1019-1059` | Verified | handler anchor stale |
| E39 | `api/runtime` advertises `max_calls`. | `http.rs:1412` | Verified | `:1419` stale |
| E40 | A2A remote agents are opaque; card is discovery. | A2A Core Concepts (fetched 2026-11-08) | Verified external | correct |

**Draft anchor corrections used in the patch** (current S4): `node_kind` 121, `validate_spawn_role`
135, `task_agent_tier` 173, `persona_for_role` 588-609, `seed_task_context` 918, `authorize_tool_call`
1091, `spawn_sub_hive` 1167, `check_calls` 1435/3129, root row 1444, plan loop 1497-1538, readiness
1559-1674, `has_aggregators` 1675, root gate 1686, root memory 1705, `refresh_memory_bank` 3005,
`execute_task_with_prompt` 3079, `execute_task_inner` 3120, `complete_task` 3343, completion write
3388-3399; `ws.rs` insert 200, template mapping 248-261, metadata 305-311; `http.rs` kind 740-743,
labels 765-781, roster 1034-1040, max_calls 1412, persona tier 943/1002; `event.rs` test 915-924;
`provider_executor.rs` specs 335-351, `finalize_root` 364-389.

---

## 5. Unresolved questions

1. **Remote tree position.** `OPERATOR-DECISIONS.md:39-40` says Remote agents "may sit under
   HyperAgent or a SuperAgent"; the remote-agents design makes them tree-free and resolves the
   sentence via `owner_kind`/`owner_id` with no canvas edge (`01:24-38`). Confirm the operator
   accepts "owner reference, not a tree edge". This is a product decision, not a code question.
2. **Single-node plan.** Should the root HyperAgent speak on a one-node plan, or is "root not
   required" an explicit metric? The current gate is `has_aggregators || nodes > 1`, so a one-node
   plan has no HyperAgent turn. Recommendation: record the metric; do not force a costly extra model
   call without a decision.
3. **Template ids vs roles.** S4 maps the template id to a spawn class in a `match`
   (`ws.rs:252-260`). Should the catalog instead carry an explicit `spawn_role`/`profile_id`? The
   current mapping is lossy: every unknown template id becomes `worker` (safer than Hyper, but not a
   real identity), and `"subagent"` still has no depth check. Decide before more templates ship.
4. **Subagent kind.** Adding `"subagent"` to `NODE_KIND_VOCABULARY` is a frozen-contract change with
   replay/web-mirror consequences. If the Lead will not take it, the card must be deleted and the
   "fourth rank visible" claim withdrawn; keeping a dead card is not neutral.
5. **Subagent depth floor.** Where does it live: `validate_spawn_role` (needs depth, so a new
   signature), a separate admission function, or the schema? No single function currently sees
   role+depth+profile for all four spawn sources; the template path has no depth parameter beyond
   `root.depth + 1`.
6. **Agent profile.** Add a built-in `agent` profile with the 9-tool grant, or keep unknown ->
   Worker and rely on the resolver? The first is explicit; the second preserves today's silent
   promotion. Recommendation: add the profile and refuse unknown ids.
7. **ToolActor.** Remove the tier/allowlist, or keep it as a non-agent actor with a real producer?
   Today it is unreachable; the draft's "engine-created" statement is false.
8. **Failure policy.** Fail-run vs fail-branch with parent retry. This is a settings/product decision
   and needs a per-class policy field before "HyperAgent decides retry" can be honest.
9. **A2A semantics.** Are `ask_parent` messages notes or requests? If requests, they need an answer,
   expiry, and delivery state; if notes, stop calling them A2A.
10. **Memory identity.** `agent_key` should be a canonical profile id, but the root writes under
    `"planner"` and tasks under their role; the engine writes under `"engine"`. The shared key
    convention is undecided and the writer/reader agreement test does not exist yet.
11. **Moving tree.** The verification snapshot is HEAD `6e3294c` with a clean backend. The tree may
    advance again; re-verify before acting.
12. **Admission vs resolver.** Is `validate_spawn_role` intended to be the authority for new roles,
    with `task_agent_tier` only a legacy read? Today admission refuses unknown new roles at every
    production insert, so F3 is latent, but the resolver still falls back. Decide and test: either
    every insert path must call admission (add a source-level test), or the resolver itself must be
    total. A latent fallback is not a fix.

---

## 6. Minimal code changes (what I would actually do)

Status is current: **[landed]** committed in `66aff50`/`45914dc`/`de0f4e6`/`6e3294c`; **[open]** not
present. No `[worktree]` status remains: the worktree changes seen during the pass were committed as
`45914dc` and `de0f4e6`, and `6e3294c` updated the template tests and docs. No code was edited by
this verification.

### M1 - One identity resolver **[open]**
Add `AgentClass {Hyper, Super, Agent, Subagent, ToolActor}` and
`resolve_identity(role, depth) -> {class, rank, tier, profile_id}`. It must:
accept canonical ids and known aliases (`Subagent`, `sub_agent`, `sub-agent`, case), reject unknown
ids with `E_PERSONA_UNKNOWN`, and be the only source of class/tier. Replace `task_agent_tier`
(`execution.rs:173-185`), `node_kind` (`:121-128`), the four lists in `validate_spawn_role`
(`:135-164`), and the label mapper (`http.rs:765-781`). This removes the latent fallback behind F3
and makes the S3/S4 alias fix durable. Keep `AgentTier` as the capability tier; class and tier are
not the same axis.

### M2 - Persist identity on the task row **[open]**
Migration v12 adds `class`, `rank`, `profile_id` (nullable for legacy) to `tasks`; write them at
`spawn_sub_hive` (`execution.rs:1174-1191`) and the plan loop (`:1504-1519`). Read them for wire
kind, roster, history, card, and memory keys. Legacy NULL = `class_unreported`, never guessed.
This is what makes replay and the card honest, and it removes repeated string parsing.

### M3 - Make the Subagent kind real once **[open, contract delta]**
Add `"subagent"` to `NODE_KIND_VOCABULARY` (`event.rs:240`) and its mirror test (`:915-924`), and
emit it from the M1 resolver in live (`execution.rs:121-128`) and HTTP (`http.rs:740-743`) paths.
The web renderer already exists (`nodes.tsx:196-219`). This is a frozen-contract change: versioned
delta, web mirror, replay compatibility. If refused, delete `subagent` from `NODE_KINDS` and
`SubagentNode`; do not ship a dead card.

### M4 - Admission, depth floor, advertised grant **[partial]**
`validate_spawn_role` exists for plan/command/model and template (`ws.rs:260`, with the S4
template-id mapping at `:252-260`). Add the depth floor: `class=Subagent` refused at `depth < 2`;
`class=Hyper` refused at `depth >= 1`; one Hyper per run. Use one named refusal (or a precise
`E_SPAWN_DEPTH` until the contract delta).
At the advertisement seam (`provider_executor.rs:335-351`), withhold `spawn_agent` for Subagent and
at max depth; keep the call-time refusal as complete mediation.
**M7 has landed for Hyper/Library (S4):** `ws.rs:248-261` maps the template id to a class, so
`planner` is refused and the Library templates insert as workers; `6e3294c` updated the test. The
residual is the depth floor for a `"subagent"` template.

### M5 - Profiles authoritative and complete **[open]**
Validate `PersonaTier` against the resolved tier at load (`prompts.rs:411-434`) or delete
`PersonaTier`; add the missing `agent` profile (`personas.json` has 7, none is `agent`); keep the
S3/S4 canonicalization but move it into the resolver (M1). Do **not** add a fourth `AgentTier`.

### M6 - One Hyper, one answer **[partial]**
S2 widened the gate to `has_aggregators || nodes > 1` (`execution.rs:1686`); the residual is the
one-node plan. Either run the root there or record `root_not_required` as an explicit metric. The
S4 template mapping closes the Hyper-template path; the residual there is the Subagent depth check.

### M7 - Template persona mapping **[landed in S4; residual depth work]**
`ws.rs:248-261` maps the template id to a spawn class (`planner`/`hyper` refused, `superagent` ->
Super, `subagent` -> Subagent, everything else -> `worker`) and `ws.rs:305-311` writes metadata.
`6e3294c` updates `template_insert_spawns_manifest_backed_dag` to insert `content-writer` and assert
`planner` returns `E_TOOL_NOT_ALLOWED`. Residual: a `"subagent"` template is still admitted at depth
1 (the template path has no depth parameter). Add `template_insert_keeps_library_templates` as the
guard.

### M8 - Memory identity and scope **[open]**
Write with `agent_key = profile_id` (not `task.role`) at `execution.rs:3388-3399`; make the memory
surface (`http.rs:1218-1228`) and the writer agree. Derive `scope_id` from class/ownership: root ->
`"run"`; Super -> hive/branch; Agent -> branch; Subagent -> own task. The engine already writes
prompt/plan to `"run"` (`:1462-1485`); document that and make the root aggregate a named "run
digest" with an explicit count/omission reason (currently "Shared hive/branch memory"). Keep the
full bank tree as a separate lane (`22-memory-bank-implementation.md`).

### M9 - Remote: do not fake it **[open, design gate]**
Nothing to patch. State "not implemented" in UI/docs (no card, command, or route); add a guard test
that no `remote_*`/`device_*` task row or `NODE_*` can be produced; only then build Delta R1
(migration + `remote_agents` + `devices` + `remote_audit` + roster projection + one supervisor
lane). Never a task row, never a `NODE_SPAWN`. The memory clause waits for a global bank.

### Tests to add (the ones that actually gate this)
1. `identity_resolver_is_total` - aliases resolve, unknown refuses (kills F3/F4).
2. `subagent_depth_floor` - depth 1 refused by plan, command, model, and template (F6).
3. `template_insert_cannot_create_hyper` **and** `template_insert_keeps_library_templates` (F8/M7).
4. `root_turn_runs_for_one_node_plan` or `root_not_required_metric` (F7).
5. `kind_is_emitted_once` / `unknown_kind_is_not_worker` (F2/F13).
6. `persona_tier_matches_runtime_tier` (F5).
7. `memory_writer_and_reader_agree` (M8).
8. `remote_has_no_task_row` (F10).
9. `caps_check_calls_is_not_a_literal` (F9).

**Done for this taxonomy means:** M1+M2+M4+M7 land together, the six named tests pass, and every
"cannot" sentence in the docs maps to a refusal code and a forging test. A doc claim without a test
is a wish; a card without a producer is a lie.

---

## 7. Limitations of this verification

- **Static only.** No build, test, server, UI, provider call, migration, or device was executed. Every
  "would fail" statement about the template tests is a deterministic source-read consequence of
  `validate_spawn_role(HyperAgent, "content-writer") -> Err`, not an observed test run.
- **No runtime claim.** Whether the provider actually returns tool calls, whether the alias fix
  changes a real run, or whether a single-node root turn is acceptable are runtime/product questions.
- **External sources.** A2A Core Concepts was re-fetched; Akka, arXiv 2508.12683, and the OpenAI
  Agents SDK page were **not** re-fetched in this pass. C13 holds; C14-C16 are cited, not
  independently re-verified, and are not load-bearing for the taxonomy.
- **Moving tree.** HEAD moved from `bb95a44` to `6e3294c` and the worktree changed during the pass.
  The S4 snapshot (`6e3294c`, clean backend; code same as `de0f4e6`) is the one verified; re-grep
  symbols before editing.
- **No edits outside** `docs/research/agent-definitions/`. The only files written are this file and
  the patched `DRAFT.md`.

---

*End of verification. The taxonomy is real enough to implement; the draft's evidence was not
current enough to trust unchanged. The critical correction is not a live privilege path - admission
now refuses unknown roles at every production spawn source - but the design: identity is still a
string, the resolver still falls back, and F3 stays latent until a persisted class lands.*
