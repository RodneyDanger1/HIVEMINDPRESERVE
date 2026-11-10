# HiveMind HMARS - code-derived capability matrix

- **Task:** task-18
- **Repository:** `C:\Users\Riley\Desktop\hiverouter-v1`
- **Pinned HEAD:** `919c3398c5bf47b4deb9ed9fd2004d2a0916cb9a` (`git status` clean at audit start)
- **Mode:** read-only static audit. No source edit, commit, build, test run, server, browser, provider key, or TokenLab point. The only written file is this document.
- **Skill note:** the requested `capability-auditor` / `integration-logic` skill is not installed in this workspace (only `skills/hivemind/SKILL.md` exists). The audit was performed directly against the source at the pinned HEAD.
- **Evidence rule:** every row cites `file:line` read at this HEAD. Code-path statements are static facts; runtime behavior is not claimed. Where a path is unreachable, that is stated rather than treated as a working capability.
- **Tree movement during the audit:** this audit started on a clean tree at `919c339` and all reads were taken then. Parallel tasks later left uncommitted changes in `execution.rs`, `remote.rs`, `store.rs`, and `tests/run_lifecycle.rs` (root-remote/daemon dedupe, audit attribution, and store changes). Those changes post-date the snapshot and are **not** part of this audit; all line numbers are pinned to commit `919c339` and can be re-read with `git show 919c339:<path>`. This audit wrote only this document.

## Path legend

| Prefix in this document | Full path |
|---|---|
| `execution.rs`, `planner.rs`, `tool_loop.rs`, `provider_executor.rs`, `prompts.rs`, `http.rs`, `ws.rs`, `store.rs`, `db.rs`, `caps.rs`, `context.rs`, `graph.rs`, `remote.rs`, `daemon.rs` | `backend/crates/hiverouter-core/src/<file>` |
| `tools/tool.rs`, `tools/executor.rs`, `tools/census.rs` | `backend/crates/hiverouter-tools/src/<file>` |
| `contract/event.rs`, `contract/command.rs` | `backend/crates/hiverouter-contract/src/<file>` |
| `assets/prompts/<file>` | `backend/crates/hiverouter-core/assets/prompts/<file>` |
| `tests/<file>` | `backend/crates/hiverouter-core/tests/<file>` |

## 0. Verdict at a glance

| Level / capability | Verdict | One-line reason |
|---|---|---|
| **HyperAgent** | **Partial; registry set is a label** | Real planner root, multi-node final synthesis, and root-control turn; its root-control branch spawn is unreachable and its 3 registry tool grants are never authorized. |
| **SuperAgent** | **Genuine, but role-spelling dependent** | `superagent` gets the 7-tool profile; `super` gets the full 9-tool tier and a generic prompt because no profile matches. |
| **Agent (Worker)** | **Genuine** | 9-tool tier with approval-gated `file_write` / `shell_exec`; may spawn Subagents. |
| **Subagent** | **Genuine narrow leaf** | 5-tool read-only tier; no spawn, no write, no shell. |
| **ToolAgent** | **Label only** | 4-tool tier exists, but no production path can create a ToolAgent task. |
| **Remote agent** | **Label / fixture** | `in_process` fixture adapter only; no network transport; no model-facing device-action tool. |
| **Daemon** | **Partial** | Real record + supervisor; only `watchdog` DB checks (and operator allowlisted scripts) execute; other root-created kinds report degraded. |
| **Ownership** | **Label** | `ownership_mode/scope/owner_task_id` are persisted and displayed; no engine authorization check reads them. |
| **Memory banks** | **Partial** | Real bank rows and scoped read sets; no model write tool; root summary is names-only; root answer is written to the wrong bank. |
| **A2A** | **Genuine** | Blocking `ask_parent` with deadline/attempt cap, durable sibling messages, cross-branch approval gate. |
| **Rewire** | **Genuine but under-validated** | Depth/role/class/cycle checks exist; same-run, fan-out, terminal-state and ownership checks do not. |
| **Approvals** | **Genuine for the two workspace side effects** | `file_write` and `shell_exec` are approval-gated; `send_to_sibling` same-parent and `spawn_agent` bypass the gate by design. |
| **Parallel dispatch** | **Genuine** | 32-way local pool, clamped to 2 for external endpoints. |

**Overall verdict:** HiveMind is a credible **single-run hierarchical HMAS** with five tier labels, four real execution levels, strict class-to-class spawn validation, a real A2A path, scoped memory banks, and approval-gated file/shell effects. It is **not** true HMARS at the capability matrix it advertises: the HyperAgent's branch-spawn tool is dead, the two most-used role strings bypass the persona/profile layer, ToolAgent and RemoteAgent are labels, ownership is a projection, and rewire/remote/root-control paths have unvalidated authority edges.

---

## 1. Every role string and class mapping

`role_class` (`execution.rs:152-166`) returns one of `hyper`, `super`, `agent`, `subagent`, or `None`. `task_agent_tier` (`execution.rs:235-250`) maps the role string to an `AgentTier`; unknown roles fall back to `WorkerAgent` (`execution.rs:248`). `validate_spawn_role` (`execution.rs:195-226`) separately gates the child role by the parent tier. The DB identity trigger (`db.rs:305-382`) derives `class/rank/profile_id`. The persona registry (`assets/prompts/personas.json`) is keyed by exact persona id and resolved by `persona_for_role` (`execution.rs:825-846`).

**Table 1 - all named role strings (28) plus empty/unknown.** `H/S/W` in "child accepted" means the role is accepted by a HyperAgent / SuperAgent / WorkerAgent parent respectively; `-` means refused by every parent tier. **Source for every row:** `role_class` `execution.rs:152-166`; `task_agent_tier` `execution.rs:235-250`; spawn acceptance `execution.rs:195-226`; DB class/rank/profile `db.rs:305-382`; persona lookup `execution.rs:825-846`; node kind `execution.rs:182-188`.

| # | Role string | `role_class` | `task_agent_tier` | Child accepted by | DB class/rank/profile | Persona match | `task_node_kind` | Note |
|---|---|---|---|---|---|---|---|---|
| 1 | `planner` | hyper | HyperAgent | - | hyper/1/planner | `planner` | hyperagent | Root role used by dispatch (`execution.rs:1784`). |
| 2 | `hyper` | hyper | HyperAgent | - | hyper/1/planner | - | hyperagent | Not spawnable (no parent accepts hyper). |
| 3 | `hyperagent` | hyper | HyperAgent | - | hyper/1/planner | - | hyperagent | Not spawnable. |
| 4 | `hyper_agent` | hyper | HyperAgent | - | hyper/1/planner | - | hyperagent | Not spawnable. |
| 5 | `hyper-agent` | hyper | **WorkerAgent** | - | hyper/1/planner | - | **worker** | **Alias mismatch:** class says hyper, tier falls through to Worker (`execution.rs:238` has no hyphen form). |
| 6 | `orchestrator` | hyper | HyperAgent | - | hyper/1/planner | - | hyperagent | Not spawnable. |
| 7 | `super` | super | SuperAgent | H | super/2/superagent | - | superagent | Root-control allow path is dead (M-04); `super` is still produced by planner/operator spawns. **No persona** (Table 3). |
| 8 | `superagent` | super | SuperAgent | H | super/2/superagent | `superagent` | superagent | Planner's canonical depth-1 role; 7-tool profile. |
| 9 | `super_agent` | super | SuperAgent | H | super/2/superagent | `superagent` (compact) | superagent | Persona scope applies; prompt lookup misses (`provider_executor.rs:151-161`). |
| 10 | `super-agent` | super | **WorkerAgent** | **-** | super/2/superagent | `superagent` (compact) | **worker** | **Alias mismatch:** `validate_spawn_role` Hyper list omits it (`execution.rs:199-202`), tier misses it. |
| 11 | `sub_hive` | super | SuperAgent | H | super/2/superagent | - | superagent | No persona. |
| 12 | `subhive` | super | SuperAgent | H | super/2/superagent | - | superagent | No persona. |
| 13 | `sub-hive` | super | SuperAgent | **-** | super/2/superagent | - | superagent | **Alias mismatch:** class/tier say super, Hyper spawn list omits `sub-hive`. |
| 14 | `worker` | agent | WorkerAgent | S | agent/3/agent | - | worker | Planner/root commonly emits it; **no persona**, so full 9-tool tier + generic prompt. |
| 15 | `workeragent` | agent | WorkerAgent | S | agent/3/agent | - | worker | No persona. |
| 16 | `worker_agent` | agent | WorkerAgent | S | agent/3/agent | - | worker | No persona. |
| 17 | `worker-agent` | agent | WorkerAgent | **-** | agent/3/agent | - | worker | **Alias mismatch:** Super spawn list omits `worker-agent` (`execution.rs:204-215`). |
| 18 | `agent` | agent | WorkerAgent | S | agent/3/agent | `agent` | worker | 9-tool profile. |
| 19 | `content-writer` | agent | WorkerAgent | S | agent/3/content-writer | `content-writer` | worker | 6-tool profile. |
| 20 | `data-analyst` | agent | WorkerAgent | S | agent/3/data-analyst | `data-analyst` | worker | 6-tool profile. |
| 21 | `code-reviewer` | agent | WorkerAgent | S | agent/3/code-reviewer | `code-reviewer` | worker | 6-tool profile. |
| 22 | `chatbot` | agent | WorkerAgent | S | agent/3/chatbot | `chatbot` | worker | 4-tool profile. |
| 23 | `subagent` | subagent | Subagent | W | subagent/4/subagent | `subagent` | subagent | 5-tool tier. |
| 24 | `sub_agent` | subagent | Subagent | W | subagent/4/subagent | `subagent` (compact) | subagent | 5-tool tier. |
| 25 | `sub-agent` | subagent | Subagent | W | subagent/4/subagent | `subagent` (compact) | subagent | 5-tool tier. |
| 26 | `tool` | - | ToolAgent | - | NULL/NULL/NULL | - | tool | No producer (Table 4, M-06). |
| 27 | `toolagent` | - | ToolAgent | - | NULL/NULL/NULL | - | tool | No producer. |
| 28 | `tool_agent` | - | ToolAgent | - | NULL/NULL/NULL | - | tool | No producer. |
| 29 | `""` (empty) | - | HyperAgent only when `depth == 0`, else WorkerAgent | - | NULL/NULL/NULL | - | hyperagent / worker | `execution.rs:247-248`. |
| 30 | any unknown (e.g. `researcher`) | - | WorkerAgent (fallback) | - | NULL/NULL/NULL | - | worker | Spawn is refused earlier by `role_class(...).ok_or(PersonaUnknown)` (`execution.rs:1453`), but the tier fallback is fail-open if any future insert path skips that check. |

**Counts and drift.**

- 5 classes (`hyper`, `super`, `agent`, `subagent`, plus the tool tier with no class), 5 tiers (`execution.rs:72-83`), 4 permission classes (`tools/tool.rs:30-39`).
- 28 named role strings: 25 class-mapped (6 hyper + 7 super + 9 agent + 3 subagent) and 3 tool-only strings (`execution.rs:157-164`, `execution.rs:244`).
- 8 personas (`assets/prompts/personas.json:3,12,21,32,41,50,59,68`); 12 of the 28 named strings resolve to a persona by exact/lower/separator/compact lookup; 16 do not. The important gaps are `super` and `worker`, the role strings used by the root-control spawn and by common planner output.
- **4 alias mismatches** among 28 strings: `hyper-agent`, `super-agent`, `sub-hive`, `worker-agent`. The DB trigger is the most alias-complete table (`db.rs:305-382`); the Rust validator is the least complete.
- `PersonaTier` has no Subagent variant; the `subagent` persona declares `"tier": "worker"` (`assets/prompts/personas.json:34`), and the engine never reads `Persona.tier` (M-07).
- `task_node_kind` maps a role by tier, not by persisted class (`execution.rs:182-188`), so the mismatched aliases render the wrong node kind in the graph/UI.

### 1.1 Planner validation (wire plan -> task rows)

Plan validation is split between `Plan::validate` (structure), `WirePlan::into_plan` (wire normalization), and the dispatch insert loop (authority). A plan that passes structure can still be refused at insert time, which is the correct fail-closed order.

| Check | Enforced? | Evidence |
|---|---|---|
| Plan non-empty; at most 40 nodes | Yes | `planner.rs:50-56` |
| Unique node ids | Yes | `planner.rs:57-62` |
| Non-empty role; depth 1..=3 | Yes | `planner.rs:63-69` |
| Dependencies refer to known nodes | Yes | `planner.rs:70-75` |
| Parent exists and is exactly one depth above | Yes | `planner.rs:78-87` |
| Parent is not also a direct dependency (deadlock guard) | Yes | `planner.rs:88-94` |
| Parentless depth > 1 refused | Yes | `planner.rs:96-100` |
| Dependency on a descendant (cross-edge cycle) refused | Yes | `planner.rs:103-113` |
| Looping parent chain refused | Yes | `planner.rs:114-124` |
| Topological order is total | Yes | `planner.rs:125-128`; `planner.rs:135-170` |
| Wire parent inferred from a depth-matching dependency | Yes | `provider_executor.rs:878-896` |
| Inferred parent removed from `deps` (no parent/child deadlock) | Yes | `provider_executor.rs:891-896` |
| Exactly one repair attempt for an invalid plan | Yes | `execution.rs:1758-1776`; `provider_executor.rs:411-426` |
| Role class resolves (`PersonaUnknown` otherwise) | Yes | `execution.rs:1845` |
| Parent-tier spawn matrix | Yes | `execution.rs:1842-1846` |
| Persisted-class spawn matrix | Yes, when the parent class resolves | `execution.rs:1847-1862` |
| Subagent depth >= 2 | Yes | `execution.rs:1863-1865` |
| Global depth/fan-out/agent caps | Yes | `execution.rs:1866-1872`; `caps.rs:52-63` |
| Persona `max_children` / `max_depth` | **No** | planned loop calls only global `caps.check_spawn` (`execution.rs:1866-1872`) vs `spawn_sub_hive` (`execution.rs:1475-1482`) (M-10) |

---

## 2. Every callable tool per level/tier

### 2.1 Registered tool contracts and tier allowlists

The frozen registry has 9 contracts and 30 tier grants: Hyper 3, Super 9, Worker 9, Subagent 5, ToolAgent 4 (`tools/tool.rs:394-445`; the census asserts exactly `3 + 9 + 9 + 5 + 4`, `tools/census.rs:228`).

**Table 2 - registry tools x tiers.** `Y` = tier allowlist grants the tool. The "Approval" column is the actual dispatch behavior, not just the class label.

| Tool | Class (`tools/tool.rs`) | Approval behavior | Dispatch lane | Hyper | Super | Worker | Subagent | ToolAgent | Evidence |
|---|---|---|---|---|---|---|---|---|---|
| `spawn_agent` | Execute | **No approval** (loop-owned) | `execute_spawn_agent` | Y | Y | Y | - | - | `tools/tool.rs:336-341`; `execution.rs:2461-2469` |
| `ask_parent` | Read | No approval; **writes DB + may spend a provider call** | `execute_ask_parent` | - | Y | Y | Y | - | `tools/tool.rs:342-347`; `execution.rs:3193-3439` |
| `send_to_sibling` | Mutate | Same-parent: **no approval**. Cross-branch: approval required (or refused without bridge/jail) | `execute_send_to_sibling` | - | Y | Y | - | - | `tools/tool.rs:348-353`; `execution.rs:2472-2497` |
| `report_result` | Read | No approval; **completes the task** | read executor + engine completion | Y | Y | Y | Y | - | `tools/tool.rs:354-359`; `execution.rs:4221-4246` |
| `finish` | Read | No approval; **completes the task** | read executor + engine completion | Y | Y | Y | Y | - | `tools/tool.rs:360-365`; `execution.rs:4195-4214` |
| `file_read` | Read | No approval | read executor / jail | - | Y | Y | Y | Y | `tools/tool.rs:366-371`; `execution.rs:2621-2662` |
| `file_list` | Read | No approval | read executor / jail | - | Y | Y | Y | Y | `tools/tool.rs:372-377` |
| `file_write` | Mutate | **Approval required** | approval bridge -> side-effect executor | - | Y | Y | - | Y | `tools/tool.rs:378-383`; `execution.rs:2498-2513` |
| `shell_exec` | Execute | **Approval required** | approval bridge -> shell policy | - | Y | Y | - | Y | `tools/tool.rs:384-389` |

**Class-to-effect drift.** The module doc says "Mutate / Execute / Network tools always require the approval gate" (`tools/tool.rs:12-13`), but the dispatch has four exceptions: `spawn_agent` (Execute, no approval), same-parent `send_to_sibling` (Mutate, no approval), `ask_parent` (Read, but writes durable rows and can call the provider), and `report_result`/`finish` (Read, but drive terminal state). See M-08/M-09.

### 2.2 Effective tool sets per role (tier intersection persona)

The provider advertises `tool_specs_for_tier` = the tier allowlist (`tool_loop.rs:206-219`; `provider_executor.rs:436-450`, `provider_executor.rs:723-729`). Authorization then intersects the persona allowlist only when `persona_for_role` matches (`execution.rs:1360-1373`; `execution.rs:1208-1211`). So "advertised" and "authorized" can differ.

**Table 3 - effective sets.** Parenthetical counts are concrete tool ids. **Source for every row:** persona allowlist `assets/prompts/personas.json`; tier allowlist `tools/tool.rs:394-445`; advertised set `tool_loop.rs:206-219`; authorization `execution.rs:1360-1373`.

| Role string | Tier | Persona | Advertised (tier) | Authorized (tier x persona intersection) | Root-only | Note |
|---|---|---|---|---|---|---|
| `planner` (root) | HyperAgent | `planner`, `[]` | 0 on initial final turn; 3 on a hallucinated continuation | **0** | 5 root-control names | `finalize_root` has no tools (`provider_executor.rs:617-641`); persona empty (`personas.json:7`); root-control tools are separate. |
| `super` | SuperAgent | none | 9 | **9** | - | No persona means no narrowing, no persona prompt, no persona caps. |
| `superagent` | SuperAgent | `superagent` | 9 | **7** (`spawn_agent`, `ask_parent`, `send_to_sibling`, `file_read`, `file_list`, `report_result`, `finish`) | - | `file_write`/`shell_exec` are advertised then refused at call time. |
| `super_agent` | SuperAgent | `superagent` (compact) | 9 | 7 | - | Prompt falls back to `TASK_SYSTEM` (`provider_executor.rs:151-161`). |
| `super-agent` | WorkerAgent (fallback) | `superagent` (compact) | 9 | 7 if inserted | - | Unspawnable; latent class/tier contradiction. |
| `worker` | WorkerAgent | none | 9 | **9** | - | No persona; full mutate/exec tier, generic prompt. |
| `agent` | WorkerAgent | `agent` | 9 | **9** | - | Full 9-tool profile; `file_write`/`shell_exec` approval-gated. |
| `content-writer` | WorkerAgent | `content-writer` | 9 | **6** | - | No spawn/write/shell. |
| `data-analyst` | WorkerAgent | `data-analyst` | 9 | **6** | - | No spawn/write/shell. |
| `code-reviewer` | WorkerAgent | `code-reviewer` | 9 | **6** | - | No spawn/write/shell. |
| `chatbot` | WorkerAgent | `chatbot` | 9 | **4** (`ask_parent`, `send_to_sibling`, `report_result`, `finish`) | - | No file read at all. |
| `subagent` / `sub_agent` / `sub-agent` | Subagent | `subagent` | 5 | **5** | - | Persona lists 6 including `send_to_sibling`; the tier excludes it (M-18). |
| `tool` / `toolagent` / `tool_agent` | ToolAgent | none | 4 | 4 if inserted | - | No producer; Tier `tool.rs:437-445`. |

### 2.3 Root-only tools (not in the registry)

These are hardcoded in `provider_executor.rs:544-609` and dispatched in `execution.rs:3915-3993`. They never enter `ToolRegistry`, the census, or any tier allowlist (`execution.rs:3854-3856`).

**Table 4 - root-only tools.**

| Tool | Schema source | Dispatch | Approval | Caps | Actual behavior | Status |
|---|---|---|---|---|---|---|
| `spawn_superagent` | `provider_executor.rs:545-553` | `execution.rs:3916-3927` | none | 4 calls/turn, 1 turn (`execution.rs:3890-3891`) | Refused whenever `has_planned_branch` is true; that is true for every valid plan (M-04). | **Dead / claimed-but-uncallable** |
| `spawn_remote_agent` | `provider_executor.rs:554-569` | `execution.rs:3929-3931`, `4005-4086` | none | 4 calls/turn | Creates a remote + device + **permanent grant**; `transport_kind: "in_process"`, `endpoint_ref: "fixture"` (`execution.rs:4033-4035`, `4073-4080`). | Callable; description says "no grant" (M-03). |
| `spawn_daemon` | `provider_executor.rs:570-586` | `execution.rs:3933-3971` | none | 4 calls/turn | Creates an `enrolled` daemon; only `kind == "watchdog"` gets `{"check":"db"}`, all other root-created kinds get `{}` and later report degraded (`daemon.rs:304-330`). | Callable; partial; omitted from the per-run control prompt (M-20). |
| `read_memory_summary` | `provider_executor.rs:587-599` | `execution.rs:3973-3987` | none | 4 calls/turn | Ignores `scope` and `limit`; always calls `memory_hyper_summary(run, 8)` and formats only `bank_id:key`, never values. | Callable but weak (M-13). |
| `finish` | `provider_executor.rs:600-608` | `execution.rs:3989` | none | 1st finish returns | Ends the control turn; no args. This is a different contract from registry `finish` (verdict string). | Callable. |

Root-control errors are not converted to `TOOL_RESULT` refusals: a missing `display_name`, invalid `kind`, cap refusal, or unknown tool returns `Err` and fails the run (`execution.rs:3916-3993`). Leaf tools fail per call instead (M-25).

---

## 3. Spawn rights, caps, rewire, A2A, memory, remote/daemon

### 3.1 Spawn rights

Two checks run for every persisted spawn: `validate_spawn_role` by parent tier (`execution.rs:195-226`) and `class_may_spawn` by persisted class (`execution.rs:170-177`, used at `execution.rs:1453-1466`, `1845-1862`, `ws.rs:812-839`). The matrix is strict: **Hyper -> Super only; Super -> Agent only; Agent -> Subagent only; Subagent and ToolAgent spawn nothing.**

**Table 5 - spawn rights by parent.**

| Parent | Allowed child classes / role strings | Depth | Persona caps | Gaps |
|---|---|---|---|---|
| HyperAgent | super/superagent/super_agent/sub_hive/subhive (`execution.rs:199-202`) | child depth 1 | root persona has none | `super-agent`, `sub-hive` are class-mapped but refused; `spawn_superagent` path is dead (M-04). |
| SuperAgent | worker/workeragent/worker_agent/agent/content-writer/data-analyst/code-reviewer/chatbot (`execution.rs:204-215`) | child depth 2 | `superagent` max_children=8, max_depth=3 (`personas.json:28-29`) | `worker-agent` refused; role `super` has no persona. |
| WorkerAgent | any `is_subagent_role` normalization (`execution.rs:217`, `execution.rs:142-147`) | child depth >= 2 (`execution.rs:1483-1485`) | none for `worker`; `agent` has none | Subagent at depth 2 is legal only if a Worker sits at depth 1, which the hierarchy cannot normally produce. |
| Subagent | none (`execution.rs:218`) | - | - | Holds. |
| ToolAgent | none (`execution.rs:219`) | - | - | Tier has no producer. |
| Operator (`SPAWN_SUB_HIVE`) | whatever the parent's matrix allows (`ws.rs:956-963` -> `execution.rs:1440`) | inherited | inherited | No approval/ownership check; relies on the parent's class. |
| Root control (`spawn_superagent`) | super only | 1 | root | Unreachable (M-04). |

### 3.2 Caps

**Table 6 - hard caps and where they are (not) enforced.**

| Cap | Value | Enforced at | Gap |
|---|---|---|---|
| Spawn depth | 3 | `caps.rs:9,52-54`; plan `planner.rs:67-69`; planned `execution.rs:1872`; template `ws.rs:318-337`; rewire depth check `ws.rs:808-810` | Persona `max_depth` only in `spawn_sub_hive` (`execution.rs:1475-1482`), not planned/template/rewire (M-10). |
| Fan-out | 8 per parent | `caps.rs:11,56-58`; `execution.rs:1467-1479`; planned `execution.rs:1868-1872`; template `ws.rs:302-337` | Rewire does not re-check the new parent's fan-out (M-11). |
| Agents/run | 64 | `caps.rs:13,59-61`; `execution.rs:1473,1486`; `execution.rs:1834,1872` | Template insert checks caps outside the spawn lock (`ws.rs:318-337`). |
| Provider calls/run | 64 | `caps.rs:18,65-73`; claimed at plan, root control, task turn, parent answer | - |
| Plan nodes | 40 | `planner.rs:10,54-56` | - |
| Tool turns/task | 12 | `tool_loop.rs:14,254-258` | - |
| Tool calls/run | 40 | `tool_loop.rs:16,265-269`; `execution.rs:2416-2427` | - |
| Approval waits/run | 8 | `tool_loop.rs:21`; `execution.rs:2800-2810` | - |
| A2A messages/run | 32 | `execution.rs:93`; `execution.rs:3259-3273`, `3528-3555` | - |
| A2A text | 2,000 chars | `execution.rs:91`; `execution.rs:3217-3224`, `3520-3527` | - |
| A2A answer timeout / attempts | 120 s / 2 | `execution.rs:95,97`; `execution.rs:3302-3314` | - |
| Root-control tool calls / turns | 4 / 1 | `execution.rs:3890-3891` | Root tools are not counted in the run tool-call budget. |
| Parallel dispatch | 32 local, 2 external | `http.rs:353-367` | Test engines keep 1. |
| Persona `max_children` / `max_depth` | <= global | `spawn_sub_hive` only (`execution.rs:1475-1482`) | Planner and template paths use global caps only (M-10); test `tests/run_lifecycle.rs:1169-1223` covers only `spawn_sub_hive`. |

### 3.3 Rewire rules

`REWIRE_TASK` is handled in `ws.rs:780-928`; the graph mutation is `graph.rs:174-234`; the row write is `store.rs:644-665`.

**Table 7 - rewire checks.**

| Check | Enforced? | Evidence | Gap |
|---|---|---|---|
| Task row exists | Yes (policy checks only if row exists) | `ws.rs:788-846` | A graph-only node bypasses all hierarchy checks. |
| New parent row exists | Yes | `ws.rs:801-807` | - |
| `new_parent.depth + 1 == task.depth` | Yes | `ws.rs:808-810` | - |
| Role/tier matrix | Yes | `ws.rs:812-815` | - |
| Persisted class matrix | Yes, when both classes resolve | `ws.rs:816-839` | Missing class means the check is skipped. |
| Self/cycle | Yes | `graph.rs:193-203`, `216-226` | - |
| `expected_rev` CAS | Yes | `graph.rs:183-188`; `ws.rs:849` | - |
| Rewire-in-flight | Yes | `graph.rs:189-191` | - |
| Context switch (clear old parent, inject new parent) | Yes | `ws.rs:840-845`, `886-913`; `context.rs:164-213` | The new parent's facts are injected without a run check. |
| **Same run** | **No** | `ws.rs:801-839`; `graph.rs` has no run concept; `store.rs:644-665` updates by task id only | Cross-run rewire + cross-run fact/memory leak (M-02). |
| **Fan-out of new parent** | **No** | `ws.rs` never calls `caps.check_spawn`; `caps.rs:56-58` | Can exceed 8 children (M-11). |
| **Terminal task/parent state** | **No** | `ws.rs:788-846` checks neither state | A completed task can be moved (M-11). |
| **Ownership** | **No** | `store.rs:1158-1207` is read-only display; no engine caller | Ownership is a label (M-15). |
| Descendant depth consistency | N/A | Only the moved node's depth is checked; its descendants keep their old depths, which remain `parent+1` relative to it | - |

### 3.4 A2A rights

**Table 8 - A2A rights by tier.**

| Path | Tiers allowed | Approval | Target/terminal checks | Budget / timeout | Evidence |
|---|---|---|---|---|---|
| `ask_parent` | Super, Worker, Subagent | none | Parent must exist and be non-terminal; duplicate open question refused | 32 messages/run; 2,000 chars; 120 s; 2 answer attempts; one provider call | `tools/tool.rs:342-347`, `427-436`; `execution.rs:3210-3233`, `3259-3282`, `3302-3314` |
| `send_to_sibling` same-parent | Super, Worker | **none** | Same run, same parent, not self, target non-terminal | same shared budget | `execution.rs:2496-2497`, `3502-3538` |
| `send_to_sibling` cross-branch | Super, Worker | **required** (refused if no bridge/jail) | Same run; target non-terminal; exact args bound by approval | one approval wait | `execution.rs:2472-2495`, `2978-2981` |
| Subagent sibling message | Subagent | - | Tier excludes `send_to_sibling`; persona lists it (dead entry) | - | `personas.json:36`; `tools/tool.rs:427-436` |
| Root / HyperAgent A2A | - | - | Root has no parent; persona empty | - | `personas.json:7` |
| ToolAgent A2A | - | - | Tier has no ask/sibling | - | `tools/tool.rs:437-445` |

A2A messages are data in the prompt, labelled untrusted; recipient inbox facts are filtered from child inheritance (`execution.rs:1201-1205`, `1247-1260`). Continuation turns receive bounded A2A updates (`execution.rs:3735-3763`, `4258`).

### 3.5 Memory bank access

**Table 9 - memory bank rights.**

| Bank / surface | Read by | Written by | Evidence | Limit / gap |
|---|---|---|---|---|
| `bank:run:<run>` | Every task in the run via `memory_read_set` | Engine root prompt/plan; `record_agent_memory_scoped` when scope_id is `run`/`legacy` | `store.rs:2180-2228`, `execution.rs:1798-1820` | Root final answer is **not** written here (M-23). |
| `bank:hive:<super>` | Super (own), Agent/Subagent (nearest ancestor) | Super task completion | `store.rs:2712-2727`, `2753-2780` | - |
| `bank:branch:<task>` | The task itself and its direct children (parent branch) | Agent/Subagent completion; Super completion routes to its hive bank instead | `store.rs:1342-1353`, `2729-2746` | Root completion lands in a branch bank, not run (M-23). |
| Sibling / foreign-run branch | **Not readable** by design | - | `store.rs:2175-2202` (bank id list excludes siblings) | Cross-run rewire can make a foreign parent branch addressable (M-02). |
| Root summary | HyperAgent root-control `read_memory_summary` | - | `execution.rs:3973-3987` | Names only, no values; scope/limit ignored (M-13). |
| Model memory write | - | **No model tool** | `tools/tool.rs:111-130`; `store.rs:1495-1533` (`record_memory_entry` has no caller in product code) | M-28. |
| Operator memory | HTTP `GET/POST /api/memory/*` | Operator | `http.rs:613-623` | Not a model capability. |

Memory text is screened for secret markers (`execution.rs:61-76`), bounded to 2,000 chars on write (`execution.rs:4354-4363`), and rendered with a 4,000-char prompt budget (`execution.rs:54-59`; `context.rs:69-100`).

### 3.6 Remote and daemon rights

**Table 10 - remote/daemon authority.**

| Operation | Who can invoke | Approval | Transport / reality | Evidence |
|---|---|---|---|---|
| Create remote | Operator HTTP; HyperAgent root control | **none** | `in_process` only; fixture adapter; no network transport | `http.rs:1746-1790`; `execution.rs:4005-4086`; `remote.rs:308-315` |
| Create device | Operator HTTP; root control auto-creates one | none | Fixture device row; `fixture:<id>` | `http.rs:1848-1894`; `execution.rs:4064-4072` |
| Create grant | Operator HTTP; root control auto-creates one | **none** | Permanent (`expires_at: None`) for root path | `http.rs:1897-1925`; `execution.rs:4073-4080` |
| Request device action | Operator HTTP only | Critical actions: nonce-bound approval; reversible actions execute | Mutates JSON state via `fixture_apply` | `http.rs:1944-1973`; `remote.rs:893-1099`, `1358-1396` |
| Create daemon | Operator HTTP; HyperAgent root control | **none** | Record only; starts `enrolled`; no task row | `http.rs:2060-2099`; `execution.rs:3933-3971`; `daemon.rs:148-187` |
| Switch / retire daemon | Operator HTTP | none | Changes state and schedules ticks | `daemon.rs:219-265`; `http.rs:640-643` |
| Run daemon job | Supervisor lane | - | `{"check":"db"}` real; operator allowlisted scripts; other kinds degraded | `daemon.rs:304-330`; `main.rs:186-193` |
| Model device action | **No tool exists** | - | Only `request_action` callers are HTTP and tests | `grep` call sites; `http.rs:1961` |

No tier has a remote/daemon/device capability; the only non-operator creation path is the root control turn. Remote action and daemon switch are operator-only.

---

## 4. Claimed-but-uncallable, callable-but-unintended, and drift

Class legend: **C-U** = claimed but uncallable; **U-C** = callable but unintended; **DRIFT** = prompt/manifest/contract drift; **LABEL** = declared but not enforced; **GAP** = coverage/validation gap; **BUG** = concrete implementation defect; **TEST** = production/test divergence.

### 4.1 High and medium findings

**Table 11A - high/medium mismatch register.**

| ID | Sev | Class | Finding | Evidence | RED test proposal |
|---|---|---|---|---|---|
| M-01 | High | C-U, fail-open | The two role strings the runtime uses for Super/Agent (`super`, `worker`) resolve to **no persona**. They get the full tier, the generic `TASK_SYSTEM`, and no persona spawn caps; an imported restrictive `superagent`/`agent` profile is bypassed by using the alias. | `execution.rs:825-846`, `1208-1211`, `3922`; `assets/prompts/personas.json:3-75`; `db.rs:331-340`; `provider_executor.rs:151-161` | `red_profile_alias_scope`: import `superagent` with `allowed_tool_ids:["file_read"]` and `max_children:1`; spawn `super`; assert the context allowlist is `["file_read"]` and the second child is refused. Currently the context has no persona scope and full 9 tools apply. |
| M-02 | High | U-C | `REWIRE_TASK` has no same-run check. A task can be reparented under another run's task; the new parent's context facts are injected into the task's prompt and the row/graph parent is moved. A subsequent memory read includes the foreign parent's branch bank because `memory_read_set` filters by bank id, not run id. | `ws.rs:780-849`, `886-913`; `graph.rs:174-234`; `store.rs:644-665`, `2180-2202`; `context.rs:438-448` | `red_rewire_cross_run`: create two runs with depth-aligned tasks; send `REWIRE_TASK` across runs; expect `E_SCHEMA_INVALID`/`E_RUN_MISMATCH`; assert no foreign fact key. Currently succeeds and injects facts. |
| M-03 | High | U-C, DRIFT | The root `spawn_remote_agent` description says the principal "has no grant" (`provider_executor.rs:557`), but the code creates a device and a permanent grant, with no approval; it does the same for `daemon_relay` (switch + grant), contradicting the report's "relay remains records-only" (`docs/evidence/HMARS-BUGFIX-REPORT.md:92`). Root daemon enrollment is also approval-free. Up to 4 root tool calls per run. | `provider_executor.rs:554-569`; `execution.rs:4043-4080`, `3933-3971`, `3914` | `red_root_remote_no_grant`: root control calls `spawn_remote_agent`; assert `list_grants(remote)` is empty and no approval row exists. Currently 1 grant, 0 approvals. |
| M-04 | Medium | C-U | **Root `spawn_superagent` is dead.** `has_planned_branch` is true for every valid plan: `Plan::validate` rejects empty plans, depth 0, and parentless depth > 1, so every finite valid plan has at least one depth-1 node. The allow branch can never run. HyperAgent therefore has no working branch-spawn tool. | `execution.rs:3881-3927`; `planner.rs:50-100` | `red_root_can_add_one_branch`: one-node depth-1 plan; root control requests `spawn_superagent`; expect a new child task (2 pre-existing tasks -> 3). Currently no child. **Correction:** the prior taxonomy audit's "up to 4 branches" claim (`docs/audit/AGENT-TAXONOMY-VERIFICATION.md:478`, a post-pin parallel document) is not reachable; the path is unconditionally refused. |
| M-05 | Medium | C-U | HyperAgent's registry grants (`spawn_agent`, `report_result`, `finish`) are never authorized. The initial root final turn offers no tools (`finalize_root`); a hallucinated tool call can enter a continuation that re-advertises the 3 Hyper tools, but the `planner` persona's empty allowlist refuses all of them. Net callable = 0. | `tools/tool.rs:395-398`; `execution.rs:4125-4129`, `2048-2120`, `1367-1372`; `provider_executor.rs:617-641`, `723-729`; `personas.json:7` | `red_hyper_tools_callable`: capture provider requests for a 2-node run; assert the root never receives a callable Hyper registry tool. Currently 0 authorized and 0 offered on the initial final turn. |
| M-06 | Medium | C-U | `ToolAgent` (4 tools) has no producer. `role_class` has no `tool` arm; `validate_spawn_role` refuses it from every parent; planned roles require `role_class`; the template catalog only uses manifest ids; root control has no such name. Only direct DB insertion can create one. | `tools/tool.rs:437-445`; `execution.rs:152-166`, `195-226`, `1845-1846`, `1453`; `ws.rs:143-156` | `red_toolagent_has_producer`: enumerate every insert path (`spawn_sub_hive`, plan, template, root control) with `tool`/`toolagent`/`tool_agent`; assert at least one creates a ToolAgent. Currently all refuse or map to worker. |
| M-07 | Medium | LABEL | `Persona.tier`, `refusal_policy`, and `output_schema_id` are not enforced. Tier is only serialized to the UI; refusal policy is only validated as a string; output schema is copied into context and never checked against a child result. A hot-imported profile changes allowlists immediately but not the executor's boot-built system prompt. | `prompts.rs:371-408`, `415-438`; `execution.rs:1211`; `http.rs:1199`, `1258`, `1192`; `provider_executor.rs:146-149`, `276-279` | `red_profile_fields_enforced`: import a persona with `tier:"tool"`, `output_schema_id:"x"`, `refusal_policy:"report"`; run a child; expect a tier/schema/policy refusal or validation. Currently runs unchanged; assert the prompt update is not observable until restart (fails). |
| M-08 | Medium | U-C, DRIFT | `send_to_sibling` is Mutate-class but the same-parent path executes with **no approval**; the registry doc says all Mutate tools require the gate. | `tools/tool.rs:348-353`, `12-13`; `execution.rs:2496-2497`, `3528-3538` | `red_same_sibling_requires_approval`: worker sends same-parent `send_to_sibling` with no approval bridge; expect `E_TOOL_NOT_ALLOWED` or an approval request. Currently the message row is written. |
| M-09 | Medium | DRIFT | Permission classes do not match effects: `ask_parent` is Read but writes durable rows and can spend a provider call; `report_result`/`finish` are Read but drive terminal state; `spawn_agent` is Execute but approval-free. | `tools/tool.rs:335-365`, `55-57`; `execution.rs:2461-2469`, `3193-3439`, `4195-4246` | `red_permission_class_matches_effect`: table test asserting every tool that writes DB, spends provider budget, or terminalizes is Mutate/Execute. Fails for these four. |
| M-10 | Medium | U-C | Persona spawn caps are enforced only in `spawn_sub_hive`. Planned nodes and template inserts check global caps only; a restrictive `max_children`/`max_depth` cannot constrain the main planner/template paths. The test covers only direct spawn. | `execution.rs:1475-1482` vs `1866-1872`; `ws.rs:318-337`; `tests/run_lifecycle.rs:1169-1223` | `red_planner_respects_profile_caps`: persona `superagent` with `max_children:1`; plan two children under it; expect `E_FANOUT_LIMIT`. Currently both inserted. |
| M-11 | Medium | U-C | Rewire checks depth/role/class, but not same run, new-parent fan-out, terminal state, or ownership. It can put a 9th child under a full parent and can move a completed task. | `ws.rs:808-839`; `caps.rs:56-58`; `graph.rs:174-234` | `red_rewire_fanout`: parent with 8 children; rewire a 9th in; expect `E_FANOUT_LIMIT`. `red_rewire_terminal`: rewire a completed task; expect `E_TASK_TERMINAL`. Currently both succeed. |
| M-13 | Medium | C-U, DRIFT | `read_memory_summary` advertises `scope` (run/hive/branch) and `limit` (1-8), but the handler ignores both, always reads run+hive with limit 8, and formats only `bank_id:key` names (no values). | `provider_executor.rs:587-599`; `execution.rs:3973-3987`; `store.rs:2236-2244` | `red_read_memory_summary_scope`: call with `scope:"branch"` and a 2-entry branch bank; expect those entries/values. Currently returns run/hive key names only. |
| M-14 | Medium | C-U, DRIFT | Alias table drift: `hyper-agent` (class hyper, tier Worker), `super-agent` (class super, tier Worker, Hyper spawn refused), `sub-hive` (class super, Hyper spawn refused), `worker-agent` (class agent, Super spawn refused). | `execution.rs:152-166`, `195-226`, `235-250`; `db.rs:305-340` | `red_role_alias_tables_agree`: parameterized over all 28 strings; assert `role_class`, `task_agent_tier`, `validate_spawn_role`, and DB class agree. 4 fail. |
| M-15 | Medium | LABEL | Persisted ownership (`ownership_mode`, `ownership_scope`, `owner_task_id`), `rank`, and `profile_id` are displayed in the roster but never consulted by spawn, rewire, kill, or tool authorization. "SuperAgent ownership" is a projection. | `db.rs:570-608`; `http.rs:1293-1303`; `store.rs:1158-1207` | `red_ownership_enforced`: two runs/owners; attempt a cross-scope rewire/kill; expect an ownership refusal. Currently allowed by the role/depth matrix. |
| M-16 | Medium | DRIFT | The provider is offered the tier set, while authorization intersects the persona set. `superagent` sees 9 and may call 7; `content-writer` sees 9 and may call 6; `chatbot` sees 9 and may call 4. This burns turns and produces call-time refusals. | `tool_loop.rs:206-219`; `provider_executor.rs:436-450`, `723-729`; `execution.rs:1367-1371` | `red_advertised_equals_authorized`: for each persona task, assert the advertised specs equal the tier x persona intersection. Fails for `superagent`, `content-writer`, `data-analyst`, `code-reviewer`, `chatbot`. |
| M-17 | Medium | GAP | The capability census compares registry <-> tier allowlists <-> prompt claims, but not persona grants and not root-only tools. The 5 root-control names are outside the census entirely. | `tools/census.rs:20-45`, `93-127`, `164-195`; `execution.rs:3854-3856` | `red_census_covers_root_and_persona`: add a root tool/persona grant to the surface and assert census drift detection. Currently no coverage. |

### 4.2 Low findings

**Table 11B - low mismatch register.**

| ID | Sev | Class | Finding | Evidence | RED test proposal |
|---|---|---|---|---|---|
| M-12 | Low | DRIFT | The primary planner prompt's JSON schema omits `parent`; production relies on `WirePlan` inferring a parent from a depth-matching dependency, and on the one repair call when deps do not express hierarchy. | `provider_executor.rs:26-33`, `878-896`, `411-426`; `planner.rs:96-100`; `assets/prompts/planner.prompty:16-24` | `red_planner_prompt_parent`: assert `PLANNER_SYSTEM` tells the model to include `parent` (or proves dependency inference). Currently the prompt omits it; a flat depth-2 plan needs the repair call. |
| M-18 | Low | C-U | The `subagent` persona lists `send_to_sibling`, but the Subagent tier excludes it. The persona allowlist is not a subset of the tier. | `assets/prompts/personas.json:36`; `tools/tool.rs:427-436`; `execution.rs:1366` | `red_persona_subset_of_tier`: assert every persona `allowed_tool_ids` is a subset of its tier allowlist. Fails for `subagent`. |
| M-19 | Low | DRIFT | `ROOT_SYSTEM` tells the final root it may call `spawn_remote_agent` at most once, but `finalize_root` advertises no tools. | `provider_executor.rs:46-52`, `617-641` | `red_root_final_prompt_matches_tools`: assert the final-root system prompt claims no tool calls and the request has no tools. Prompt text fails. |
| M-20 | Low | DRIFT | The per-run root-control prompt omits `spawn_daemon` while `ROOT_CONTROL_SYSTEM` advertises it; it also advertises the dead `spawn_superagent`. | `execution.rs:3873-3878`; `provider_executor.rs:39-44`, `570-586` | `red_root_prompt_enumerates_tools`: assert the control prompt lists every advertised root tool. Fails for `spawn_daemon`. |
| M-21 | Low | DRIFT | `planner.prompty` is rendered into the planner **user** message via `format!("{:?}: {}", ...)`; its `system:` body and model front matter are not used as the system prompt/model. The actual system is `PLANNER_SYSTEM`; the model is `ProviderSettings`. | `execution.rs:2325-2338`; `provider_executor.rs:96-125`, `151-161`; `assets/prompts/planner.prompty:16-24` | `red_planner_asset_is_system`: assert the planner request system equals the prompty `system:` body. Fails (PLANNER_SYSTEM is used). |
| M-22 | Low | DRIFT | The template catalog doc claims only `planner` has a prompt asset and four library ids have none; the manifest embeds all eight. `insert_template_dag` accepts 6 ids, but `/api/templates` marks only 4 as `insertable`. | `ws.rs:124-156`; `http.rs:1373-1376`; `assets/prompts/manifest.json:3-57`; `ws.rs:1288-1318` | `red_template_insertable_matches`: compare endpoint `insertable` to the actual accepted set. Fails for `agent`/`superagent`; the doc comment is stale. |
| M-23 | Low | BUG | The root's completed reply is written with `scope_id = task.id` and class `hyper`, which routes to `bank:branch:<root>`, not the run bank. The run prompt/plan are correctly written to the run bank. | `execution.rs:4356-4363`; `store.rs:1342-1353` | `red_root_answer_in_run_bank`: complete a root task; assert the entry is in `bank:run:<run>`. Currently in `bank:branch:<root>`. |
| M-24 | Low | U-C | `spawn_sub_hive` checks only that the **run** is non-terminal, not the parent task's state. A completed/failed parent can still spawn a child; the child is not aggregated by the already-complete parent. | `execution.rs:1448-1452` | `red_no_spawn_under_terminal_parent`: complete a parent, then spawn under it; expect `E_TASK_TERMINAL`. Currently succeeds. |
| M-25 | Low | U-C | Root-control tool errors abort the run (missing arg, invalid kind, cap refusal, unknown tool). Leaf tools return named `TOOL_RESULT` refusals instead. | `execution.rs:3916-3993` | `red_root_tool_refusal_is_a_result`: root sends malformed `spawn_remote_agent`; expect a `TOOL_RESULT` refusal and a continuing run. Currently the run fails. |
| M-26 | Low | TEST | Most integration tests build the engine with empty personas, no approval bridge, and no root orchestrator; production loads 8 personas, the approval bridge, and the root turn. The tested matrix is not the shipped matrix. | `tests/run_lifecycle.rs:453-479`; `tests/tool_loop_engine.rs:206-227`; `http.rs:318-369` | `red_matrix_with_production_wiring`: run the tool matrix with embedded personas/root/approval; assert `super`/`worker` scoping. Fails because the test harness omits it. |
| M-27 | Low | LABEL | `task_agent_tier` maps any unknown role to `WorkerAgent`. Spawn currently refuses unknown roles first (`role_class`), but any future insertion path that skips `role_class` would grant the full 9-tool Worker set. | `execution.rs:248`, `1453`, `1845` | `red_unknown_role_no_tools`: insert `researcher` directly; assert 0 tools. Currently the Worker tier set applies. |
| M-28 | Low | C-U | `record_memory_entry` is documented as a "model-facing tool surface" but no `ToolId` or handler invokes it; only tests call it. | `store.rs:1495-1533`; `tools/tool.rs:111-130`; `tests/mem_isolation.rs:132` | `red_memory_write_tool_exists`: assert a memory-write tool is in `ToolRegistry`. Currently absent. |
| M-29 | Low | LABEL | Remote transport is `in_process` only; device actions mutate JSON in SQLite via `fixture_apply`. No network adapter and no model action tool. "Remote agent" is a fixture record. | `remote.rs:3-5`, `308-315`, `1358-1396`; `http.rs:1764-1775`; `execution.rs:4033-4035` | `red_real_remote_transport`: assert a non-`in_process` transport can be created; currently `E_REMOTE_TRANSPORT`. |
| M-30 | Low | LABEL | Daemon jobs other than `watchdog` DB checks (and operator allowlisted scripts) are declarative; root-created `relay`/`switch`/`satellite` get `{}` and report degraded on tick. | `daemon.rs:304-330`, `148-158`; `execution.rs:3960-3964` | `red_root_daemon_has_a_job`: enroll a relay via root, tick, expect a real job or an explicit unsupported-capability refusal. Currently degraded with `E_DAEMON_JOB_UNIMPLEMENTED`. |

---

## 5. Production vs test doubles

**Table 12 - doubles and fixtures.**

| Item | Location | Scope | Classification | Effect on this audit |
|---|---|---|---|---|
| `MockExecutor`, `RootRemoteExecutor`, `RootSuperExecutor`, `RootToolExecutor` | `tests/run_lifecycle.rs:70-108`, `207-242`, `305-329`, `395-423` | Tests only | Permitted test doubles (provider seam) | Lifecycle/root tests are not production evidence. `RootSuperExecutor`'s second `finish` is never used because root control runs one turn. |
| wiremock SSE server | `tests/tool_loop_matrix.rs:107-125`, `135-166` | Tests only | Permitted HTTP client double | Tool-loop wire behavior is tested offline, not in this audit. |
| Empty `TemplateRegistry::default()` | `tests/run_lifecycle.rs:473`; `tests/tool_loop_engine.rs:216` | Tests only | Test seam | Production uses `load_embedded` (`prompts.rs:187-195`). |
| `RunEngine::new` without personas/root/tools/jail | `tests/tool_loop_engine.rs:206-227`; `tests/run_lifecycle.rs:453-479` | Tests only | Test configuration, not a mock | Most tool-loop tests exercise a different capability matrix than production (M-26). |
| `UnconfiguredExecutor` | `execution.rs:475-478`; `http.rs:272-288` | Production | Honest keyless seam, not a fake | Refuses by name; no fabricated replies. |
| Remote `in_process` fixture | `remote.rs:308-315`, `1358-1396`; `http.rs:1764-1775` | Production | Production fixture adapter | Remote is a simulated device backend, not a network transport (M-29). |
| Root-created remote/device/grant | `execution.rs:4033-4035`, `4064-4080` | Production | Production fixture records + auto-grant | The tool description says "no grant"; the records are real rows (M-03). |
| `record_memory_entry` | `store.rs:1495-1533`; test callers only | Tests only in practice | Dead production API | Doc says model-facing; no tool invokes it (M-28). |
| Injected personas in tests | `tests/run_lifecycle.rs:1033-1044`, `1170-1181` | Tests only | Fabricated profiles | They prove seeding works; they do not prove the shipped 8-profile set. |
| `AppState` test constructor `with_services` | `http.rs:491` | Tests only | Test seam | Production uses `AppState::new`/`with_graph`. |

**No mock or stub was found in the production execution/tool registry path.** The production fixtures are the remote `in_process` adapter and the root-created fixture records; both are documented as fixtures in code, but the remote tool description contradicts the grant behavior (M-03).

---

## 6. Counts (derived from source)

| Count | Value | Evidence |
|---|---|---|
| Agent tiers | 5 (`HyperAgent`, `SuperAgent`, `WorkerAgent`, `Subagent`, `ToolAgent`) | `tools/tool.rs:72-83` |
| Permission classes | 4 (`Read`, `Mutate`, `Execute`, `Network`; Network unused) | `tools/tool.rs:30-39` |
| Named role strings | 28 (25 class-mapped + 3 tool-only), plus empty/unknown fallback | `execution.rs:152-166`, `195-226`, `235-250` |
| Personas | 8 | `assets/prompts/personas.json` |
| Named aliases resolving to a persona | 12 of 28 | `execution.rs:825-846` |
| Alias-table mismatches | 4 (`hyper-agent`, `super-agent`, `sub-hive`, `worker-agent`) | Table 1; `execution.rs:152-166`, `195-226`, `235-250` |
| Registered tools | 9 | `tools/tool.rs:111-130`, `335-392` |
| Tier grants | 30 (3 + 9 + 9 + 5 + 4) | `tools/tool.rs:394-445`; `tools/census.rs:228` |
| Root-only tool names | 5 (`spawn_superagent`, `spawn_remote_agent`, `spawn_daemon`, `read_memory_summary`, `finish`) | `provider_executor.rs:544-609` |
| Unique callable wire names | 13 (9 registry + 4 non-registry; root `finish` duplicates registry `finish`) | `tools/tool.rs:135-147`; `provider_executor.rs:544-609` |
| Effective registry tools: root / super / superagent / worker / content-writer / chatbot / subagent / ToolAgent | 0 / 9 / 7 / 9 / 6 / 4 / 5 / 4 | Tables 2-3 |

---

## 7. RED test proposals (consolidated)

The per-finding tests are in Tables 11A/11B. The highest-value new tests are:

1. `red_root_can_add_one_branch` - root `spawn_superagent` must create a child for a one-node plan; currently no child (M-04).
2. `red_profile_alias_scope` - `super`/`worker` must inherit `superagent`/`agent` profile restrictions and prompts (M-01).
3. `red_rewire_cross_run` - reject cross-run rewire and assert no foreign fact key (M-02).
4. `red_root_remote_no_grant` - root `spawn_remote_agent` must not create a grant without approval, matching its description (M-03).
5. `red_hyper_tools_callable` / `red_toolagent_has_producer` - the two advertised-but-unreachable sets must either be removed from the census or made reachable (M-05/M-06).
6. `red_planner_respects_profile_caps` - profile `max_children`/`max_depth` constrain planned nodes (M-10).
7. `red_advertised_equals_authorized` - the provider offers only the effective tier x persona intersection (M-16).
8. `red_same_sibling_requires_approval` / `red_permission_class_matches_effect` - approval and class tables match the actual effects (M-08/M-09).
9. `red_read_memory_summary_scope` - scope/limit/values are honored (M-13).
10. `red_role_alias_tables_agree` - all 28 aliases agree across Rust validators and the DB trigger (M-14).

---

## 8. Verdict: which levels are genuinely capable, which are labels

| Level | Verdict | Evidence summary |
|---|---|---|
| **HyperAgent** | **Partial; its tool grants are labels** | Planner + final synthesis (`execution.rs:2048-2119`) + root control (remote/daemon/memory summary/finish) are real. `spawn_superagent` is dead (M-04); the 3 Hyper registry grants are never authorized (M-05); root final has no tools (`provider_executor.rs:617-641`). The root cannot add a branch. |
| **SuperAgent** | **Genuine for `superagent`; alias-dependent** | 7-tool persona + spawn Agent + hive bank. Role `super` gets 9 tools, no persona prompt, no persona caps (M-01). The class is real; the identity key is a label. |
| **Agent (Worker)** | **Genuine** | 9-tool tier, `file_write`/`shell_exec` approval-gated, spawns Subagent only. Role `worker` gets the same 9 tools but no profile (M-01). |
| **Subagent** | **Genuine narrow leaf** | 5 tools, no spawn/write/shell, depth 3, branch bank. Persona's 6th tool (`send_to_sibling`) is dead (M-18). |
| **ToolAgent** | **Label only** | 4-tool allowlist, no producer (M-06). The UI/design card has no runtime occupant. |
| **Remote agent** | **Label / fixture** | `in_process` fixture only; no network transport; no model action tool; root auto-grants (M-03/M-29). |
| **Daemon** | **Partial** | Durable record + supervisor; watchdog DB check and operator scripts work; root-created relay/switch/satellite degrade (M-30). |
| **Ownership** | **Label** | Persisted/displayed, not enforced (M-15). |
| **Memory banks** | **Partial** | Real run/hive/branch banks and scoped read sets; no model write tool; root summary weak; root answer lands in a branch bank (M-13/M-23/M-28). |
| **A2A** | **Genuine** | Blocking ask/answer, durable sibling messages, 32/run budget, cross-branch approval (Table 8). |
| **Rewire** | **Genuine but under-validated** | Real graph/row/context mutation; cross-run/fanout/terminal/ownership gaps (M-02/M-11). |
| **Approvals** | **Genuine for workspace writes/shell; exceptions elsewhere** | `file_write`/`shell_exec` are gated; `spawn_agent` and same-parent `send_to_sibling` are not (M-08/M-09). |
| **Parallel dispatch** | **Genuine** | 32-way local, 2 external (`http.rs:353-367`). |

**Bottom line.** The hierarchy is real at Hyper -> Super -> Agent -> Subagent for *execution*: a planner DAG, strict class spawn checks, real provider/tool turns, A2A, scoped memory, and approval-gated workspace effects. The capability matrix is not real at the *authority* layer: one of the four levels has no branch-spawn tool, two role aliases bypass the profile layer, one tier has no producer, remote agents are fixtures, ownership is a projection, and rewire/root-control/remote paths bypass checks that the leaf tool path enforces. Treat the product as a credible hierarchical HMAS, not as HMARS-complete, until M-01 through M-04 and M-08 through M-11 are closed with the RED tests above.

---

## 9. Limitations and provenance

- **Static only.** No build, test, server, provider call, or database mutation was executed. Code-path claims are reads at HEAD `919c339`; runtime behavior is not claimed. Existing test files are cited as evidence of intended behavior, not executed proof.
- **No provider keys or TokenLab points were used.** No secret was read or printed.
- **Read-only for this audit.** The only write is this file. `git status` was clean at audit start. Parallel tasks later left uncommitted changes in `execution.rs`, `remote.rs`, `store.rs`, and `tests/run_lifecycle.rs`; those are outside this audit and the citations remain pinned to `919c339`.
- **Skill availability.** `capability-auditor` and `integration-logic` are not installed in this workspace; only `skills/hivemind/SKILL.md` exists. The audit is a direct code audit, not a skill-driven one.
- **Overlap with prior audits.** `docs/audit/AGENT-TAXONOMY-VERIFICATION.md` (task-17) and `docs/audit/REMOTE-PERMISSION-GAPS.md` (task-19) cover adjacent ground at the same HEAD. This document independently derives the matrix and **corrects** task-17's G04: the `spawn_superagent` allow path is unreachable, not "up to 4 branch leads"; the root-control gate is unconditionally refused for every valid plan.
- **Unreachable paths are marked as such.** A finding that depends on direct DB insertion or a hypothetical future insert path is labelled latent (M-06, M-27) rather than presented as a live capability.
