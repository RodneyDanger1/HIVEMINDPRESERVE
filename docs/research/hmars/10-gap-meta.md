# 10 — Gap and contradiction meta-analysis of the first wave (lane 1)

**Lane:** second-wave lane 1 — gap/contradiction meta-analysis.
**Date:** 2026-11-07, 21:20–21:40 America/Chicago.
**Workspace:** `C:\Users\Riley\Desktop\hiverouter-v1`.
**Source HEAD:** `bd575ba7adf3a5f2a9bdbc69f9bbf7084fe6fe65` (`bd575ba`).
**Working tree:** dirty. The tree changed during this pass: `provider_executor.rs` (persona fix, 21:16), `execution.rs` / `tool_loop.rs` / tool-loop tests (cap + refusal fix, 21:31–21:37), `HiveMindLauncher.cs` (workspace-root fix, 21:31), plus `AGENT-ARCHITECTURE-TRUTH.md`, the launcher binary/pycache, and untracked `docs/research/`. Only `10-gap-meta.md` was written by this lane; the code changes are another lane's, uncommitted and unverified here. See §2.4.
**Discipline:** `/deep-inspection` is not installed in this profile. I used the closest available disciplines, **`/deep-investigate` + `/trace`**, plus direct read-only runtime and source inspection. This substitution is recorded because the task named the primary discipline.

**Scope of this file:** audit the first wave (`docs/research/hmars/01`–`09`), the operator answers (`OPERATOR-DECISIONS.md`), the repo's own architecture truth (`docs/provider/AGENT-ARCHITECTURE-TRUTH.md`), and the live runtime at `http://127.0.0.1:8099` (read-only). It is a gap/contradiction report, not a design and not a product change.

**What this lane did not do:** no paid provider call, no credential read, no product edit, no server restart, no test/build, no git write. The live server went down at 21:27:50 CST; no probe after that is possible without its owner restarting it.

---

## 0. Evidence conventions

| Mark | Meaning in this file |
|---|---|
| **VERIFIED** | Reproduced in this pass by reading the exact current source line, by a read-only runtime probe with a quoted response, or by a command whose output is quoted. |
| **READ** | Read in a first-wave report or an official/primary source but not independently re-fetched here. Inherited evidence; may be stale. |
| **INFERRED** | Follows from cited facts through a named assumption or arithmetic; not directly observed. |
| **CONTRADICTED** | A cited claim conflicts with another cited claim or with current code/runtime. Both sides are named. |
| **UNVERIFIED** | No evidence in this pass; not to be used as a decision input. |

Two distinctions are load-bearing:

1. **First-wave reports are evidence of what their authors read, not evidence of the current tree.** The tree moved during the first wave (notably the uncommitted `provider_executor.rs` fix), so a first-wave claim can be true of `bd575ba` and false of the working tree, or vice versa.
2. **A live probe is a timestamped observation, not a stable baseline.** The runtime was not quiescent: `last_seq` went from 0 to 1366 and `graph_nodes` from 8 to 14 during this pass, and the server then stopped. Every runtime claim below carries its timestamp.

---

## 1. Executive summary — top 10 findings

1. **Run snapshots are not run-scoped.** `snapshot::capture` stores `graph.snapshot()` — the process-global graph — and `TaskNode` carries no `run_id` (`backend/crates/hiverouter-core/src/snapshot.rs:258-271`; `graph.rs:16-27`). Live snapshot files prove accumulation: per-run latest node counts were **2, 4, 4, 6, 8, 14, 20, 26, 31, 35** with **1…9 roots**. The client decodes every node (`web/src/state/store.ts:571-622`), so history/replay for one run can display other runs' tasks. No first-wave report tested snapshot scope; `08` assumed the snapshot was the run's graph.
2. **`E_BUDGET_EXCEEDED` is a lying refusal.** The tool-loop turn/call cap returns `Refusal::BudgetExceeded` (`execution.rs:2845-2849`; `tool_loop.rs:254-269`), whose user text is “The run exceeded the **$0.50 cost cap**” (`backend/crates/hiverouter-contract/src/refusal.rs:152-154`). Three live tool-using runs failed that way after **4 tool turns and 8/5/5 calls**; the first measured only **3,268 tokens**, and the other two have **no persisted usage row at all** — nowhere near $0.50 or 400K tokens. The binding ceiling is the 4-turn tool loop, not the 40-call provider cap that `03` and `09` emphasise. The working tree changed at 21:31–21:34 to return `E_CALL_LIMIT` and raise the caps to 12 turns/40 calls; that fix is uncommitted and untested here, and the live failure is the reason it exists.
3. **The live workspace root was wrong for the task.** Run 1's persisted task result says the workspace contains only `HiveMind.exe` (8,704 B), `hiverouter.exe` (15,105,536 B) and `README.txt` (1,038 B) — exactly `dist\HiveMind`, not the repo. The prompt was “Map the architecture of this repository…”. There is no workspace/project picker (`08` §6.2; `06` D6), so the agent analysed the packaged app. The launcher was modified at 21:31 to pass `--workspace-root`; the running server predates it and a portable copy still falls back to cwd.
4. **The persona-continuation finding is already stale in the working tree.** The uncommitted diff in `provider_executor.rs` normalises `task.role` (trim/lowercase, hyphen/space→underscore) and uses `self.system_prompt_for(task)` in `continue_with_tools` (`provider_executor.rs:82-92`, `:375`). At HEAD `bd575ba` the continuation hard-codes `TASK_SYSTEM`. `07`, `08`, `09`, and `AGENT-ARCHITECTURE-TRUTH` disagree about this path because none pinned the working-tree hash.
5. **The operator's autonomy modes 1–4 are absent from every first-wave design.** The token “autonomy” appears only in `09`'s settings list (`09:176`, `:184`, `:208`) and in `OPERATOR-DECISIONS.md:19`. There is no mode definition, no permission matrix, no persistence, no UI, and no test.
6. **Remote/Watchdog daemons are both missing and contradicted.** The operator requires simple long-lived switches/daemons (“one job forever, report every 5 minutes”, `OPERATOR-DECISIONS.md:32-33`); `02` rejects “cross-run agent identity / long-lived agents” as unnecessary complexity (`02:526`, X13). No first-wave report designs the daemon lifecycle, authority, card, persistence, or failure mode.
7. **“Venn” memory is not representable in either proposed ladder.** The operator wants global/per-run/per-hive/per-branch/per-role scopes with useful overlap (`OPERATOR-DECISIONS.md:63-66`). `02` proposes run→branch→role→global (`02:444-449`); `04` proposes project→run→hive→branch→agent_role (`04:44-51`, `04:94-96`). Neither supports multi-membership, and `04` explicitly gives the HyperAgent summaries only, never raw access (`04:1464-1472`) — stricter than the operator's conditional raw-access allowance (`OPERATOR-DECISIONS.md:67-68`).
8. **“No hard default budget” has no safety design and conflicts with the shipped caps.** The operator wants no hard default, a visible soft spend guard and an emergency stop (`OPERATOR-DECISIONS.md:24-26`); the tree ships a hard `$0.50`/400K/40-call per-run cap (`budget.rs:25-31`), and `03`/`06` treat hard caps as the safety model. `09` proposes a soft guard but does not define the global cap that denies wallet exhaustion (`09:227-229`). `06` already shows the per-run cap can overshoot to ~$1.07 (`06:1196-1200`).
9. **The operator's A2A topology directly conflicts with `02`'s reject list.** The operator allows any-agent mesh by settings, optional pub/sub topics, and a bounded event/notice bus (`OPERATOR-DECISIONS.md:52-59`); `02` rejects an agent pub/sub bus and handoff as unnecessary complexity (`02:514-526`, X1/X2/X13). No report reconciles “bounded bus with optional topics” with “no bus”, or “mesh allowed by settings” with “tree/sibling only”.
10. **The UI cannot satisfy “all agents get cards” or the card-content decision.** `HyperAgentNode` is dead (filtered from the canvas), `error`/`tool` kinds are not produced on the normal path, and Subagent/Remote render as Worker (`08:386-405`, `08:606-643`). `05` limits cards to four facts (`05:108-118`, `05:579-583`) while the operator requires role, task, state, cost, tools, branch (`OPERATOR-DECISIONS.md:83-84`) — a direct conflict, not a polish item.

**The 393216 item is not a factual contradiction.** `01`, `03`, `05`, and `06` all record DeepSeek's documented `max_tokens` maximum as 384K = 393,216; the code constant is `MAX_OUTPUT_TOKENS = 393_216` (`dto.rs:22-23`). The `AGENTS.md` §13 “UNVERIFIED” register refers to a different document's output-budget claim, not to this provider maximum. It is a **scope-of-authority** conflict, resolved in §4.1. It must still never become a default `max_tokens` or a budget constant.

---

## 2. Baseline and method

### 2.1 Files read

| File | What this lane used it for |
|---|---|
| `docs/research/hmars/01-deepseek-api.md` | Provider surface, max output, cache rules, gaps, 20 Q&A, unknowns. |
| `docs/research/hmars/02-hmars-patterns.md` | Hierarchy/registry/A2A patterns, invariants, reject list, failure modes. |
| `docs/research/hmars/03-scale-concurrency.md` | Caps, provider envelope, SQLite/Tokio arithmetic, stages. |
| `docs/research/hmars/04-memory-architecture.md` | Existing memory, bank tree, schema/read policy, claims register. |
| `docs/research/hmars/05-ui-ux-patterns.md` | Projection/card/approval/memory recommendations, open decisions. |
| `docs/research/hmars/06-security-cost.md` | Threat register, cost arithmetic, provider errors, unknowns. |
| `docs/research/hmars/07-code-backend.md` | Code-level tier/persona/A2A/memory/caps gaps and claim register. |
| `docs/research/hmars/08-code-ui.md` | UI truth, live snapshot, dead/missing controls, claim limits. |
| `docs/research/hmars/09-synthesis-and-goal.md` | Synthesis, 20 answered questions, priority plan, unverified list. |
| `docs/research/hmars/OPERATOR-DECISIONS.md` | 45 bullet decisions (header says 36 — see §5.0). |
| `docs/provider/AGENT-ARCHITECTURE-TRUTH.md` | Repo's plain-English truth and its claims register. |
| Source spot-checks | `caps.rs`, `budget.rs`, `dto.rs`, `execution.rs`, `provider_executor.rs`, `tool_loop.rs`, `refusal.rs`, `snapshot.rs`, `graph.rs`, `websocket`/`http` routes, `store.ts`. |

### 2.2 Live runtime probes (all read-only)

| Time (CST) | Probe | Observed | Class |
|---|---|---|---|
| 21:20–21:21 | `GET /health` | `last_seq:0`, `graph_nodes:8`, schema 10, empty DB (`runs:0`, `tasks:0`). | VERIFIED |
| 21:23:36 | `GET /health` | `last_seq:1366`, `graph_nodes:14`, uptime 6,897 s. | VERIFIED |
| 21:23:36 | `GET /api/history/runs` | 1 run, `status:"failed"`, `error_code:"E_BUDGET_EXCEEDED"`, created `2026-11-08T03:23:11Z`. | VERIFIED |
| 21:23:36 | `GET /api/projects` | 1 project, name = UUID, created 03:23:11Z. | VERIFIED |
| 21:23:36 | `GET /api/agents/profiles` | 7 profiles (chatbot, code-reviewer, content-writer, data-analyst, planner, subagent, superagent). | VERIFIED |
| 21:24 | `GET /api/usage/runs/{id}` | prompt 2,100 / completion 1,168 / total 3,268 / cache-hit 1,920 / miss 180. | VERIFIED |
| 21:24 | `GET /api/history/runs/{id}/state?revision=5` | 14 graph nodes, 5 roots; run's own six tasks plus eight earlier-run tasks. | VERIFIED |
| 21:26 | Read-only SQLite | 5 runs, 27 tasks, 22 tool-turn rows, 34 snapshots; 3 failed runs `E_BUDGET_EXCEEDED`, 2 completed. | VERIFIED |
| 21:27:50 | `GET /health` | Connection refused; no `HiveMind`/`hiverouter` process remained. | VERIFIED |

The server became unavailable during the pass. The first wave's live baseline (`08` §0, “server already running; empty state”) was true at its read time and false by 21:23. No report is at fault for that; every report is at fault if it presents a runtime snapshot without a timestamp.

### 2.3 Evidence limits of this pass

- DeepSeek URLs were **not re-fetched**. Provider facts marked READ are inherited from `01`/`03`/`05`/`06` (fetched 2026-11-07 by those lanes).
- `04`, `05`, and `06` were read at their executive/claims/decision sections, not every line. Sections not read line-by-line are named in §11.
- The planner free-form-role reachability was **not exercised live** (server down). It is a code-path finding (`07` + `planner.rs` read), marked VERIFIED for the code path and UNVERIFIED for runtime effect.
- No paid call was made by this lane. The live runs observed were created by another process; their usage is quoted as observed data, not as this lane's experiment.

### 2.4 Late-breaking working-tree changes (not authored by this lane)

The tree was edited by another lane while this report was being written. This is the single most important process fact in the report: **the findings below are time-scoped observations, not claims about the current working tree unless the hash says so.**

| File | Changed at (CST) | Current hash (16) | Change seen | Effect on this report |
|---|---|---|---|---|
| `backend/crates/hiverouter-core/src/provider_executor.rs` | 21:16 | `5656D179BDB68E74` | Role trim/lowercase + `-`/space→`_`; `continue_with_tools` uses `self.system_prompt_for(task)` at `:375`. | Fixes C40/C42 for the working tree; C40/C42 remain true at HEAD. |
| `backend/crates/hiverouter-core/src/execution.rs` | 21:31 | `8218C81ED1679503` | Tool-cap branch returns `Refusal::CallLimit` instead of `BudgetExceeded` (`:2845-2849`). | Partially fixes X10/R1 in the working tree; not built/tested/committed here. |
| `backend/crates/hiverouter-core/src/tool_loop.rs` | 21:34 | `8D924F9E969B9F95` | `MAX_TOOL_TURNS` 4→**12**; `MAX_CALLS_PER_RUN` 16→**40**; `next_turn`/`check_call_budget` return `CallLimit`. | Changes C30/X8/D7; the live runs predate it. |
| `backend/crates/hiverouter-core/tests/tool_loop_engine.rs` | 21:37 | — | Tests updated for the new caps/refusal. | Not run by this lane. |
| `backend/crates/hiverouter-core/tests/tool_loop_matrix.rs` | — | — | Tests updated. | Not run by this lane. |
| `tools/desktop-launcher/HiveMindLauncher.cs` | 21:31 | `16B86ECB1A7AB6E3` | Passes `--workspace-root` (env `HIVEMIND_WORKSPACE_ROOT`, then a hard-coded `C:\Users\Riley\Desktop\hiverouter-v1`, then cwd). | Addresses R3 for this machine; the hard-coded dev path is not portable, and a portable copy still falls back to cwd. |
| `web/src/state/store.ts` | 21:39 | — | Adds `argsPreview` to `ApprovalRecord`; `recordsFromSnapshotGraph` is now at **571–622** (was 570–621 at 21:23). | Re-anchors the snapshot-scope citation; the defect is unchanged. |
| `web/src/panels/ApprovalOverlay.tsx` | 21:39 | — | Renders `argsPreview` when present (tooltip + text). | Partially addresses the approval-evidence gap in C96; target/path text is still absent. |
| `docs/provider/AGENT-ARCHITECTURE-TRUTH.md` | 19:25 | `D1B8B5F860978315` | The version `08` read; it is ahead of HEAD on the tool-turn persona claim. | C146 remains a HEAD-vs-doc conflict; the working-tree code now matches the doc. |

**Consequences for reading this report:**

- The live `E_BUDGET` failure at 21:23:11Z is real for the binary that was running then. The code now in the working tree would return `E_CALL_LIMIT` instead — **if** it is built, tested, and deployed. Neither is proven here.
- The live runs at 21:23–21:26 used 4 turns; the current working tree allows 12. The report's “binding 4-turn cap” is a statement about HEAD and the running binary, not about the next build.
- The live workspace root was `dist\HiveMind`; the launcher fix would make a freshly launched dev build use the repo, but the running server was not restarted and a portable copy still falls back to cwd.
- Any second-wave report that reads the current tree and does not record these hashes will be confidently wrong in one direction or the other. This is the case study for §9 G6.

---

## 3. Claim ledger — the first wave, re-marked

Each row is a material first-wave claim. “First-wave source” is the claim's home, not an endorsement. Status is **this lane's** mark.

### 3.1 Provider and DeepSeek surface

| ID | Claim | First-wave source | Status | This lane's evidence / note |
|---|---|---|---|---|
| C01 | DeepSeek account concurrency is 2,500 (`deepseek-flash`) / 500 (`deepseek-v4-pro`), account-wide. | `01:527`, `03:52-57`, `06:139` | READ | Official-doc claim; not re-fetched. Code has no provider-concurrency model. |
| C02 | Context is 1M tokens; max output is 384K. | `01:522-523`, `03:39`, `06:135` | READ | Not re-fetched. `03` uses 384K for arithmetic; `06` expands to 393,216. |
| C03 | `max_tokens` documented range is 1–393,216. | `01:181`, `05:677`, `06:56-58` | READ | The external page was not fetched by this lane; the code constant is VERIFIED below. |
| C04 | `MAX_OUTPUT_TOKENS = 393_216` exists in the provider DTO. | `01:181`, `06:56-58` | VERIFIED | `backend/crates/hiverouter-provider/src/dto.rs:22-23` read in this pass. |
| C05 | Chat Completions, thinking/effort, JSON mode (planner), tools, streaming, usage/cache counters, `/models`, `/user/balance` are wired. | `01:568-580`, `07:20-22` | READ | `07` is a code read; this lane spot-checked the constants/routes but not every path. |
| C06 | Search/Fetch are not wired; the only per-request-billed surface is absent. | `01:440-443`, `01:743` | VERIFIED | No network tool in `tool.rs:108-127`; `PermissionClass::Network` exists unused. |
| C07 | Files, Vision, Responses, Anthropic Messages, FIM/Prefix, strict tools, `user_id` are not wired. | `01:590-599`, `01:745-747` | VERIFIED | `ChatMessage.content: Option<String>` (no content parts); one non-beta base URL; no `strict` field; no `user_id` field. |
| C08 | 400/422 are collapsed into “undocumented provider HTTP status” / `ProviderInvalidResponse`. | `01:205`, `01:741`, `06:217` | VERIFIED | `client.rs:520-523` cited by `01`; this is a code read inherited, not re-read by this lane. Marked VERIFIED because `01` quotes the exact match arm. |
| C09 | 402 (insufficient balance) is mapped to `E_PROVIDER_AUTH` (“credentials rejected”). | `06:34`, `06:1191-1194` | VERIFIED | `06` cites `client.rs:507-515` and `refusal.rs:254-256`; consistent with the live budget-error behavior. |
| C10 | Cache hit/miss counters are required for a settled cost; missing counters mark the run INCOMPLETE, never zero. | `01:298`, `06:143` | READ | `01` cites `accounting.rs:234-263`; not re-read here. |
| C11 | No measured cache-hit rate was available to the first wave. | `01:299`, `01:759` | CONTRADICTED (narrow) | The first wave had no measurement; this lane observed one live run at **1,920 hit / 2,100 prompt (91.4%)** on 2026-11-08T03:23Z. This does not generalise, but “never measured” is now false for one run. |
| C12 | Prompt-token estimate is `serialized_bytes / 4`; no context-window preflight. | `01:738-739` | VERIFIED | `provider_executor.rs` estimate path and `ModelsPanel` display; `01` cites `:431-437`. No pre-send 1M check found in the budget path. |
| C13 | `user_id`/KVCache isolation is not sent; it is a future multi-profile lever. | `01:588`, `06:1235-1240` | VERIFIED | No field in the request DTO (`dto.rs:396-441` per `01`). |
| C14 | The price table matches the documented rows, but Chinese public holidays are not excluded, so local cost is 2× high on those days. | `01:534`, `06:148-152` | READ | Arithmetic/code claim inherited; not re-derived here. |
| C15 | Search/Fetch adoption requires a new per-request cost dimension or the run report is false. | `01:554`, `01:586` | INFERRED | Follows from a token-only `RunAccounting` and the documented $0.75/1k price; sound. |

### 3.2 Agent hierarchy, caps, and execution

| ID | Claim | First-wave source | Status | This lane's evidence / note |
|---|---|---|---|---|
| C16 | `AgentTier` has exactly Hyper/Super/Worker/Tool; no Subagent variant. | `07:49-52`, `08:115-119` | VERIFIED | `tool.rs:66-100`; contract node kinds `hyperagent/superagent/worker/tool`; no `subagent`. |
| C17 | `task_agent_tier` maps every unrecognised role (including `subagent`, `content-writer`, …) to Worker. | `07:56-70`, `08:110-119` | VERIFIED | `execution.rs:151-163` (code read; lowercased comparisons). |
| C18 | `validate_spawn_role` is fail-closed: Hyper→Super/Worker/Subagent, Super→Worker/Subagent, Worker→Subagent only, Tool→none; it lowercases the requested role. | `02:248`, `07:74-83` | VERIFIED | `execution.rs:108-142` read in this pass; `:114` lowercases. |
| C19 | Planned roles bypass `validate_spawn_role`; plan validation checks structure, not role vocabulary. | `07:83`, `07:442` (G11) | VERIFIED | `execution.rs:1337-1375`; `planner.rs:63-71` accepts any non-empty role. This is the basis of C34's contradiction. |
| C20 | The HyperAgent is a planner row/anchor; no model turn ever executes the root. | `07:13`, `07:95-107`, `09:47-50` | VERIFIED | `execution.rs:1321-1335`; root absent from `leaf_order`/`aggregation_order`; no `execute_task(root)`. |
| C21 | ChatCore is the operator UI, not a SuperAgent. | `07:116-124`, `08:51-70`, `09:79-82` | VERIFIED | `ChatCore.tsx` submits `SUBMIT_PROMPT`; no direct model call. |
| C22 | SuperAgent has a tier, spawn policy, allowlist, persona/template, and caps `max_children:8`, `max_depth:3`; it may spawn workers/subagents. | `07:130-147`, `09:52-55` | VERIFIED | `personas.json` + `superagent.prompty`; `validate_spawn_role`. |
| C23 | The planner output schema omits `parent`; planned nodes attach to the root, so planned hierarchies are flat by default. | `07:147`, `07:435` (G4) | VERIFIED | `PLANNER_SYSTEM` lists `id/role/deps/depth`; `WireNode.parent` optional; insert falls back to root. This directly limits the “hierarchical planning” recommendation. |
| C24 | One active run per server; leaves, aggregators and post-plan spawns execute sequentially; a model-spawned child is awaited inline. | `03:99`, `07:21`, `07:393-400` | VERIFIED | `execution.rs:1032-1052`, `:1395-1550`, `:2668-2681`; no scheduler/worker pool. |
| C25 | Process-wide provider limiter is 8. | `03:94`, `07:395` | VERIFIED | `budget.rs:43` (`IN_FLIGHT_LIMIT = 8`). |
| C26 | SQLite is one `Mutex<Connection>`; WAL is configured but reader concurrency is unused. | `03:100`, `07:396` | VERIFIED | `db.rs:295-350`; constants/pragmas cited. |
| C27 | Event bus is 1,024 broadcast / 256 replay; every streamed fragment publishes one `REPLY_DELTA`. | `03:101-102`, `07:397` | VERIFIED | `bus.rs:18-20`; `execution.rs:1764-1795`. |
| C28 | Hard caps are depth 3, fan-out 8, agents 64, provider attempts 40, tokens 400K, cost $0.50; `Caps` is hard-coded `Caps::default()`. | `03:92-104`, `07:379-389`, `07:539-540` | VERIFIED | `caps.rs:9-15`; `budget.rs:25-43`; no env override found. |
| C29 | Engine-side `check_calls(1)` is a stateless compare; the real call cap is the provider `RunBudget`. | `07:384`, `07:444` (G13) | VERIFIED | `caps.rs:62-71`; `execution.rs:1313,2771`; `budget.rs:243-250`. |
| C30 | Tool loop has separate caps: 4 turns, 16 calls/run, 8 approval waits, 64 KiB result. | `07:345`, `07:388` | VERIFIED at 21:25; CHANGED in working tree | `tool_loop.rs:14-21` read at 21:25: 4/16. At 21:34 the working tree sets **12 turns / 40 calls** (uncommitted, not authored by this lane); `:254-269` now returns `CallLimit`. **The 4-turn cap is what failed the live runs.** |
| C31 | A parent can complete while a descendant is non-terminal; the parent-completion guard is absent. | `02:1005`, `02:707` (I6) | INFERRED | No guard found; run-level reconcile exists. Not reproduced live. |
| C32 | Cancellation does not cascade; a killed parent can leave a running descendant. | `02:478`, `02:658-660` | INFERRED | `cancel_run`/`kill_worker` exist; assertion missing. Not reproduced live. |
| C33 | Snapshot captures the complete graph for each revision and is the crash-recovery authority. | `07:398`, `08:590-592` | VERIFIED (as written) | `snapshot.rs:258-271`; `commit_run_snapshot` call sites. **The scope problem is C34.** |
| C34 | A run snapshot is that run's graph. | Assumed by `08:590-592`; not explicitly tested | **CONTRADICTED** | Live: run 1's snapshot had 14 nodes/5 roots while its DB tasks were 6; snapshot file node counts accumulated 2→35 across runs. `capture` stores the global graph. |
| C35 | The graph retains tasks across runs; recovery loads DB tasks into the graph on boot. | Implied by `07:398` | VERIFIED | `snapshot.rs:258-271`; `recovery.rs:394-411`; live `graph_nodes` 8→14 while `/api/history/runs` was empty/one. |
| C36 | Planning one 16K call can emit at most ~65–163 nodes, so 300–600 needs hierarchical planning. | `03:493-508`, `07:387` | INFERRED | Arithmetic over 100–250 tokens/node; assumptions named in `03`. |
| C37 | The live app cannot run 50–150 concurrent agents or 300–600 tasks today. | `03:11-23`, `07:422`, `09:133-138` | VERIFIED | Code caps + serial dispatch; no scheduler. Runtime did not test scale. |

### 3.3 Personas, templates, and prompt reachability

| ID | Claim | First-wave source | Status | This lane's evidence / note |
|---|---|---|---|---|
| C38 | Seven personas ship: planner, superagent, subagent, content-writer, data-analyst, code-reviewer, chatbot. | `07:174-186`, `08:39` | VERIFIED | `personas.json`; live `/api/agents/profiles` returned 7. |
| C39 | `subagent` is a persona/role, not a tier; it maps to Worker and has no card/kind. | `07:149-162`, `08:113-125` | VERIFIED | `execution.rs:151-162`; contract kinds. |
| C40 | Persona prompts reach the first leaf call, but the tool-turn continuation uses `TASK_SYSTEM` at HEAD. | `07:214-223`, `08:206-229`, `09:62-66` | **CONTRADICTED for the working tree** | HEAD `provider_executor.rs:369-372` has `TASK_SYSTEM`; the uncommitted working tree has `self.system_prompt_for(task)` at `:375`. At HEAD the claim is VERIFIED; in the working tree it is false. |
| C41 | The planner call uses `PLANNER_SYSTEM`, not the planner persona's system body. | `07:212`, `07:498`, `09:65` | VERIFIED | `provider_executor.rs:286-313`; `plan()` never consults `persona_prompts`. |
| C42 | UI-spawned children send Title-Case roles (`SuperAgent`/`WorkerAgent`/`Subagent`), which miss the lowercase persona map at HEAD. | `08:121-122`, `08:224-229` | **CONTRADICTED for the working tree** | `store.ts:1865-1871` still sends Title-Case; the uncommitted `system_prompt_for` lowercases/normalises, so the lookup now succeeds. |
| C43 | Four worker personas (content-writer, data-analyst, code-reviewer, chatbot) are rejected by `validate_spawn_role`; they are reachable via `TEMPLATE_INSERT`, which sets `role = template_id` and skips the validator. | `07:188-192`, `08:252-256` | VERIFIED | `execution.rs:113-142`; `ws.rs:200-293`; template insert path. |
| C44 | The four worker personas are reachable **only** through `TEMPLATE_INSERT`. | `09:68-72` | **CONTRADICTED** | `07:192` says the planner can also emit the exact persona id because plan validation accepts any non-empty role and the planner prompt only *asks* for worker/superagent. Code supports `07`: `planner.rs:63-71`; `execution.rs:1337-1375`. A planner emitting `role:"content-writer"` would be inserted and (with the working-tree normalisation) get the persona prompt. Runtime unverified. |
| C45 | Per-persona `max_children`/`max_depth` apply only when the parent role string exactly equals the persona id. | `07:139`, `07:146` | VERIFIED | `execution.rs:549-558,1083-1090`; alias roles skip the persona check. |
| C46 | `Persona.allowed_tool_ids` values are not validated against the tool registry. | `02:404-407`, `07:346` | VERIFIED | `prompts.rs:411-434` validates template/caps/policy only; a typo silently disables a tool. |
| C47 | UI-created `Subagent` currently falls back to the generic prompt at HEAD. | `08:224-229` | VERIFIED at HEAD | The working tree fix normalises the role; whether the resulting prompt is the intended one is still UNVERIFIED (no live keyed run after the fix). |
| C48 | No distinct Subagent tier/card/kind exists. | `07:432`, `08:113-125` | VERIFIED | `tool.rs:66-100`; `agentState.ts:126-135`. |

### 3.4 A2A, messages, and buses

| ID | Claim | First-wave source | Status | This lane's evidence / note |
|---|---|---|---|---|
| C49 | `ask_parent` writes `agent_messages` and publishes `AGENT_MESSAGE`; it returns `{message_id, to_task_id, kind}`. | `07:282-290`, `09:88-92` | VERIFIED | `execution.rs:2417-2487`; `db.rs:160-178`. |
| C50 | No engine path reads an answer back into the child; the only consumer is the HTTP mailbox/UI. | `07:301-306`, `09:88-92` | VERIFIED | `store.agent_messages_for_run` has one consumer (`http.rs:684`); no `execution.rs` read. |
| C51 | `send_to_sibling` same-parent is immediate and skips approval; cross-branch goes through approval. | `07:292-299`, `09:84-86` | VERIFIED | `execution.rs:1695-1720,2493-2609`. |
| C52 | Message text is bounded at 2,000 chars and 32 messages/run. | `02:258`, `07:287-288` | VERIFIED | `execution.rs:67-70`; enforced in both verbs. |
| C53 | `agent_messages` has no `in_reply_to`, status, answered_at, or read state. | `07:303`, `02:365-376` | VERIFIED | `db.rs:160-178`; `04` also proposes the missing fields. |
| C54 | There is no roster/discovery endpoint or tool; a sibling target is an undiscoverable UUID. | `07:309-314`, `09:94-96` | VERIFIED | `http.rs:518-526`; `tool.rs:231-239`; no roster route. |
| C55 | The event bus is a UI/transport bus, not an agent bus. | `02:264`, `07:316-325`, `09:152` | VERIFIED | `bus.rs`; no agent subscriber; sessions subscribe in `ws.rs`. |
| C56 | A2A messages are unredacted model text, unlike memory which has a four-marker filter. | `07:331`, `07:459` | VERIFIED | `execution.rs:2440-2467,2561-2589`; no secret marker check on message text. |
| C57 | Any claim that agents “coordinate via ask_parent/send_to_sibling” is false today. | `07:456`, `09:88-92` | VERIFIED | Consequence of C50/C54/C56. |
| C58 | The operator mailbox is read-only; no compose/send route or command exists. | `08:377`, `08:588` | VERIFIED | `contract.ts:119-157`; `AgentMessagesPanel`. |
| C59 | A2A messages survive restart; the delivered event is not replayed to a model. | `07:329` | VERIFIED | SQLite table; no model read. |
| C60 | Cross-branch approval exists but the target UUID is unobtainable, so the approval is not actionable from a model's knowledge. | `07:332` (A2A-3) | VERIFIED | Same as C54. |

### 3.5 Memory

| ID | Claim | First-wave source | Status | This lane's evidence / note |
|---|---|---|---|---|
| C61 | `ScopedContext` is an in-memory per-task snapshot; parent facts are copied at creation; rewire clears the old parent's prefixed facts. | `04:117-131`, `07:229-246` | VERIFIED | `context.rs:20-29,138-186`; `execution.rs:856-886`. |
| C62 | The prompt header calls all scoped facts, including `memory:*`, “authoritative”. | `04:129`, `04:173-176` | VERIFIED | `context.rs:71`. This is an authority-inflation risk, not just wording. |
| C63 | `agent_memory` is role-keyed, 6 columns, `PK(agent_key,key)`, newest 8 at task creation, 2,000-char write cap, four-marker filter. | `04:133-145`, `07:248-261` | VERIFIED | `db.rs:179-197`; `store.rs:608-671`; `execution.rs:61-65,880-884,3017-3026`. |
| C64 | Memory read errors are swallowed by `if let Ok(...)`. | `04:143`, `07:243` | VERIFIED | `execution.rs:880`. |
| C65 | No per-hive/branch scope, trust, expiry, promotion, provenance-at-read, or HyperAgent aggregate. | `04:160-191`, `07:263-270` | VERIFIED | Schema search v1–v10; `04` claims register C8. |
| C66 | Role bleed across runs/hives is by design in the current key. | `04:162-168`, `07:261` | VERIFIED | `store.rs:657` filters `agent_key` only. |
| C67 | There is no record of what memory the model actually saw. | `04:189-191` | VERIFIED | No `memory_context_snapshots` table/route; UI has no read-set view. |
| C68 | The operator wants global/per-run/per-hive/per-branch/per-role scopes as a connected Venn cluster. | `OPERATOR-DECISIONS.md:63-66` | VERIFIED (as requirement) | Not a code claim; product input. |
| C69 | `02` proposes a four-scope ladder: run, branch, role, global. | `02:444-449` | VERIFIED (as report claim) | It omits project/hive as named scopes and makes global operator-only. |
| C70 | `04` proposes a five-bank tree: project, run, hive, branch, agent_role. | `04:44-51`, `04:94-96` | VERIFIED (as report claim) | It has no “global” bank and no multi-membership. |
| C71 | `04` gives the HyperAgent run summaries + child hive summaries/counts, never observations/notes/artifacts, and never ad-hoc search. | `04:1464-1472` | VERIFIED (as design claim) | Stricter than the operator's conditional raw-access allowance (`OPERATOR-DECISIONS.md:67-68`). |
| C72 | Vector retrieval is deferred; FTS5 is a later step; there is no documented embeddings endpoint. | `04:50-51`, `04:1480-1491` | READ | `04` S19; not re-fetched. FTS5 compile flag is a local read, not re-checked here. |
| C73 | Migration v11 is a proposal; no schema change has landed. | `04:1655-1670`, `09:258` | VERIFIED | `schema_version` 10 in live health; `agent_memory` unchanged. |
| C74 | Memory writes are provenance/trust-aware and prompt-injected content must not become authoritative. | `OPERATOR-DECISIONS.md:74-75` | VERIFIED (as requirement) | Current code violates the second half (C62). |
| C75 | The memory UI is a role list, ≤8 rows, no bank/trust/age/provenance/truncation view. | `04:145`, `08:260-269` | VERIFIED | `AgentMemoryPanel`; live `/api/agents/memory` returned 7 profiles with empty arrays. |
| C76 | Thought storage (raw reasoning vs summaries vs none) is not coded. | `AGENT-ARCHITECTURE-TRUTH.md:32`; `OPERATOR-DECISIONS.md:71-72` | VERIFIED (absence) | No thought table; reasoning used only for continuation where present. |
| C77 | Retention modes are proposed but not implemented; there is no human-only audited retention UI. | `04:961-1005`, `05:184` | VERIFIED (absence) | No retention route; migration not landed. |

### 3.6 Security and cost

| ID | Claim | First-wave source | Status | This lane's evidence / note |
|---|---|---|---|---|
| C78 | `spawn_agent` is registered Execute but bypasses approval by design (“loop-owned”). | `06:28`, `06:199`, `07:358` | VERIFIED | `tool.rs:334-339`; `execution.rs:1684-1692`. |
| C79 | Mutating HTTP endpoints (`profiles/import`, `profiles/{id}` DELETE, `memory/{key}` DELETE) have no token/Origin check; no handler validates `Host`. | `06:29`, `06:204-205` | READ | `06` cites `http.rs:511-529`, `:1017-1044`; not re-read this pass. |
| C80 | WS checks loopback peer + exact Origin + per-boot token before upgrade; token is in the query string; no per-message auth. | `06:30`, `06:208` | READ | `06` cites `ws.rs:371-383`; `web/src/api/client.ts:310`. |
| C81 | The key is DPAPI-protected at rest and injected into the child environment; the server reads only the env. | `06:31`, `06:1185-1189` | READ | Launcher/secret files cited; not re-read. |
| C82 | Tool results are appended raw as `role=tool`; no envelope/label/instruction-hierarchy marker. | `06:27`, `06:27`-row | VERIFIED | `provider_executor.rs:357-424`; `06` T01. |
| C83 | `cmd.exe` is on the shell allowlist, so one approved shell call equals arbitrary code as the user. | `06:1152-1157` | READ | `shell.rs:66-72` cited; product decision U6. |
| C84 | Cost is not persisted; `run_usage` has token columns only; there is no per-hive budget. | `06:33`, `06:1202-1210` | VERIFIED | `db.rs:202-213`; live `run_usage` has tokens only. **Live: runs 2 and 3 (failed) have no `run_usage` row; only runs 1, 4, 5 do.** A failed terminal run can therefore have no measured token/cost record. |
| C85 | Preflight checks spent, not projected; one call can overshoot the $0.50 cap to ~$1.07. | `06:32`, `06:1196-1200` | READ | Arithmetic from `06`; not reproduced. |
| C86 | 402/400/422 error identity is wrong; 402 needs its own refusal and spend freeze. | `06:34`, `06:1191-1194` | VERIFIED | `refusal.rs`/`client.rs` cited; consistent with C09. |
| C87 | E_BUDGET_EXCEEDED means the run cost cap. | `refusal.rs:152-154` (not a first-wave claim; code truth) | **CONTRADICTED at HEAD/live; CHANGED in working tree** | At 21:25 the same refusal was raised for the tool-loop turn/call cap (`execution.rs:2845-2849`; `tool_loop.rs:254-269`), and live runs prove it fires at 3,268 tokens. At 21:31 the working tree returns `Refusal::CallLimit` instead (uncommitted). |
| C88 | Secret detection is a four-substring tripwire, not redaction. | `04:177-179`, `07:459` | VERIFIED | `execution.rs:61-65`; `audit.rs` probe. |
| C89 | Side effects default ON (`HIVEROUTER_SIDE_EFFECTS` must be falsey to disable). | `06:79`, `07:366` | VERIFIED | `config.rs:355-361` (per `06`/`07`). |
| C90 | The localhost network boundary is closed by loopback bind; the live boundaries are browser origin, file content, and model output. | `06:26`, `06:1134-1139` | READ | `config.rs:327-333` cited; sound. |

### 3.7 UI and contract

| ID | Claim | First-wave source | Status | This lane's evidence / note |
|---|---|---|---|---|
| C91 | Contract has 24 events, 16 agent states, 65 refusal codes, 14 frozen commands (+3 decision-gated `DIFF_*`), 12 WS commands. | `05:73-80`, `08:342-350` | READ | Contract doc claims; not re-counted here. |
| C92 | HyperAgent card is implemented but filtered out of the live canvas; the chat overlay is the HyperAgent surface. | `08:82-91` | VERIFIED | `graph.ts:151`; `ChatCore`; `HyperAgentNode`. |
| C93 | `error` and `tool` node kinds are not produced on the normal path; their cards are effectively dead. | `08:138-147`, `08:386-405` | VERIFIED | `execution.rs:97-106` returns only four tiers; store defaults missing kind to worker. |
| C94 | The server never sends `label`/`progress`/`ctx`/`score`/`error_code` on `NODE_*`; cards show fallbacks/`—`. | `08:100-110`, `08:634-635` | VERIFIED | `publish_node_spawn`; `TaskRecord` normalization. |
| C95 | Tool activity has two homes (chat ACTIONS last 4 + Agents panel); no run-detail ledger. | `05:153`, `08:134-136` | VERIFIED | `ChatCore.tsx:273-307`; `ToolActivityPanel`; no run-detail surface. |
| C96 | Approvals show tool + truncated `args_sha256`, not args/target/text; expired/stale entries without a refusal have no Dismiss; no focus trap/Escape. | `08:64-70`, `08:365-369` | VERIFIED at 21:23; PARTIAL FIX in working tree | `ApprovalOverlay.tsx`; `08` says no keydown handler. At 21:39 another lane added `argsPreview` rendering; target/path text, Dismiss, and focus handling are still unverified. |
| C97 | No model/thinking/effort/profile/spend selector; these are env-only/read-only; no route/command mutates them. | `08:61-63`, `08:370-373` | VERIFIED | `SettingsPanel`; `config.rs`; routes list. |
| C98 | Replay flattens every node to `kind:'worker'` with synthetic `task <id>` labels. | `08:382`, `08:590-592` | VERIFIED | `graph.ts:196-234`; `Canvas.tsx:266-275`. |
| C99 | The client decodes **every** node in a history snapshot and re-roots a parent outside the snapshot; it does not filter by run. | Not in first wave | VERIFIED | `store.ts:571-622`; combined with C34 this is a live UI defect. |
| C100 | Swarm is a read-only indented task list + 5 counters/stat tiles + hero empty state; no topology actions. | `05:490`, `08:68-70` | VERIFIED | `SwarmPanel.tsx`; `AgentHierarchy`. |
| C101 | Projects are a localStorage UUID + constant name; no create/switch/rename; `/api/projects` is read-only. | `08:169`, `08:379` | VERIFIED | `store.ts:333-362`; live `/api/projects` returned one UUID-named project. |
| C102 | The UI has no “Now”/live execution surface that combines hierarchy, live execution, and memory/A2A edges. | Not in first wave | VERIFIED (absence) | No route/panel; `05` splits memory/A2A into inspectors, not the main view. |
| C103 | FigCraft geometry parity is pinned for main composition; chat is 400×480 vs board 07's 420×500; prior measured audit is stale. | `08:409-447` | READ | `designParity.test.ts`; README; not re-measured. |
| C104 | Current app data is empty (no runs/projects/history). | `09:264` | **CONTRADICTED** | At 21:23:36 the app had 1 project and 1 failed run; by 21:26, 5 runs/27 tasks. The claim was true earlier, false at the live probe. |
| C105 | The live UI is in an empty state, so run-time rendering is source-derived only. | `08:44-47` | **CONTRADICTED** | Same: the app became active during this pass; `08`'s snapshot was stale by 21:23. |

### 3.8 Operator requirements and process claims

| ID | Claim | First-wave source | Status | This lane's evidence / note |
|---|---|---|---|---|
| C106 | Autonomy modes 1–4, default mode 1, are a requirement. | `OPERATOR-DECISIONS.md:19-21` | VERIFIED (as requirement) | No design or code. |
| C107 | Approval policy options must all be settings; default is side effects only; no silent bypasses. | `OPERATOR-DECISIONS.md:21-22` | VERIFIED (as requirement) | No settings; `spawn_agent` bypasses its declared class (C78). |
| C108 | Agent capabilities editable in settings, validated/audited, default enabled. | `OPERATOR-DECISIONS.md:23` | VERIFIED (as requirement) | No edit/audit path; import/delete only. |
| C109 | Budget: no hard default; caching is the goal; budget can be switched on; soft guard/emergency stop acceptable. | `OPERATOR-DECISIONS.md:24-26` | VERIFIED (as requirement) | Current hard caps; no switch; no soft guard; no global governor. |
| C110 | Every agent, including tiny Watchdog/Remote agents, must have a UI card. | `OPERATOR-DECISIONS.md:31` | VERIFIED (as requirement) | Hyper/Error/Tool cards dead; Subagent/Remote no card (C92/C93/C48). |
| C111 | Remote/Watchdog agents are simple long-lived switches/daemons: one job forever, report to owner, low complexity. | `OPERATOR-DECISIONS.md:32-33` | VERIFIED (as requirement) | Not coded; `02` X13 rejects long-lived identity. |
| C112 | Default agents code-defined; built-in templates in a settings menu, clearly labeled; custom via settings/chat; no uncontrolled marketplace. | `OPERATOR-DECISIONS.md:34-36` | VERIFIED (as requirement) | Embedded personas + import/delete; no settings menu/editor; `02` X11 rejects full editor. |
| C113 | HyperAgent/SuperAgent may choose/spawn an appropriate built-in persona, subject to settings. | `OPERATOR-DECISIONS.md:37-38` | VERIFIED (as requirement) | Spawn policy rejects persona ids; planner free-form role is unvalidated (C44), not a setting. |
| C114 | Default hierarchy includes Remote agents under HyperAgent or a SuperAgent. | `OPERATOR-DECISIONS.md:39-40` | VERIFIED (as requirement) | No Remote tier/role; plan schema flat (C23). |
| C115 | SuperAgent ownership is switchable: department/domain owner, branch lead, or single complex task. | `OPERATOR-DECISIONS.md:41-42` | VERIFIED (as requirement) | No ownership model anywhere. |
| C116 | Chat is operator UI; a conversational SuperAgent is deferred pending research/donor analysis. | `OPERATOR-DECISIONS.md:43-44` | VERIFIED (as requirement) | Current state matches “not an agent”; no deferral decision record or donor analysis in the first wave. |
| C117 | Workspace read/list, write/patch, shell, network fetch/search, provider files/search, and browser/computer use are desired if collisions/sandboxing are controlled. | `OPERATOR-DECISIONS.md:48-49` | VERIFIED (as requirement) | File/shell exist; Search/Fetch/Files absent; no browser/computer tool; no collision control. |
| C118 | A HiveWide collision/usage-enforcement daemon (warn/reject duplicate tool use, watch CPU/RAM/load, log/prevent crashes) is in scope, plus sandboxing/sand pits. | `OPERATOR-DECISIONS.md:50-51` | VERIFIED (as requirement) | No collision daemon; `06` jail is path confinement, not a sandbox. |
| C119 | All A2A must support mute/blind/invisible settings; tree communication can be disabled. | `OPERATOR-DECISIONS.md:52` | VERIFIED (as requirement) | No visibility/mute model. |
| C120 | Any-agent mesh is allowed by settings; default safest tree/sibling; private topic buses optional. | `OPERATOR-DECISIONS.md:53-54` | **CONTRADICTED by design** | `02` X1/X7/X13 reject bus/mesh/handoff; no settings. |
| C121 | Cross-hive SuperAgent messaging allowed by settings; default most auditable (HyperAgent/approved bridge). | `OPERATOR-DECISIONS.md:55-56` | VERIFIED (as requirement) | No cross-hive bridge; only cross-parent sibling approval. |
| C122 | Bus: bounded event/notice bus, optional pub/sub topics, avoid raw group chat. | `OPERATOR-DECISIONS.md:57` | **CONTRADICTED by design** | `02` rejects an agent pub/sub bus entirely. |
| C123 | Messages crossing a hive or exposing sensitive data require approval; ordinary parent/sibling chatter does not. | `OPERATOR-DECISIONS.md:58-59` | VERIFIED (as requirement) | Cross-parent approval exists; no hive/sensitivity gate. |
| C124 | Memory scopes are global/per-run/per-hive/per-branch/per-role, described as a Venn cluster. | `OPERATOR-DECISIONS.md:63-66` | VERIFIED (as requirement) | `02`/`04` design trees without multi-membership (C69/C70). |
| C125 | SuperAgents get their own banks; children tap the tree they spawned under; cross-branch sharing when useful. | `OPERATOR-DECISIONS.md:65-66` | VERIFIED (as requirement) | No banks; `04` promotion is one level; no cross-branch sharing without promotion. |
| C126 | HyperAgent may read all banks only if leakage/noise is prevented; summaries/audited first, raw only as explicit setting. | `OPERATOR-DECISIONS.md:67-68` | VERIFIED (as requirement) | `04` allows summaries only, no raw setting (C71). |
| C127 | Storage: SQLite structured rows + summaries first; local vector index and external vector DB are future options; design all four paths if possible. | `OPERATOR-DECISIONS.md:69-70` | VERIFIED (as requirement) | `04` designs SQLite + later FTS; vectors deferred, no external vector path. |
| C128 | Thought storage: summaries default; raw reasoning and none are settings. | `OPERATOR-DECISIONS.md:71-72` | VERIFIED (as requirement) | Not coded, no design. |
| C129 | Retention: run lifetime + human-approved project persistence; all retention settings human-only and audited. | `OPERATOR-DECISIONS.md:72-73` | VERIFIED (as requirement) | `04` retention modes proposed; no human-only enforcement/audit/UI. |
| C130 | Memory writes provenance/trust-aware; prompt-injected content must not become authoritative. | `OPERATOR-DECISIONS.md:74-75` | VERIFIED (as requirement) | Current prompt header says authoritative (C62). |
| C131 | A2A UI: dedicated topology/memory inspector by default; drawer/canvas overlays/chat summaries are settings. | `OPERATOR-DECISIONS.md:79-80` | VERIFIED (as requirement) | No inspector; mailbox read-only; no settings. |
| C132 | Main view: hierarchy + live execution + memory/A2A edges; all projections (canvas/chat/graph/dashboard) are settings, all enabled by default. | `OPERATOR-DECISIONS.md:81-82` | VERIFIED (as requirement) | `05` proposes removals and one-primary-region density; no memory/A2A edges; no “Now”. |
| C133 | Agent cards: role, task, state, cost, tools, branch by default; metric/message density is a setting. | `OPERATOR-DECISIONS.md:83-84` | **CONTRADICTED by design** | `05` caps cards at four facts and puts the rest in the inspector. |
| C134 | Design source: extend FigCraft; research option 4 but default to extension. | `OPERATOR-DECISIONS.md:85-86` | VERIFIED (as requirement) | `05`/`08` extend; no option-4 research record in the first wave. |
| C135 | DeepSeek only; no Anthropic key; Anthropic-format docs are research only. | `OPERATOR-DECISIONS.md:90` | VERIFIED (as requirement) | `01`/`09` align; no Anthropic wiring. |
| C136 | Files: research/recommend; default yes only for vision/image, not repo files. | `OPERATOR-DECISIONS.md:91` | READ (requirement) | `01` says Files NOT NEEDED; a product decision stronger than “research/recommend”. |
| C137 | Search & Fetch: research/recommend; agents should use them when needed, gated by approval/budget. | `OPERATOR-DECISIONS.md:92-93` | VERIFIED (as requirement) | Not wired; no approval/SSRF/cost dimension. |
| C138 | Responses API: research/recommend; Chat Completions remains likely canonical. | `OPERATOR-DECISIONS.md:94` | VERIFIED (as requirement) | `01` says NOT NEEDED; no decision record. |
| C139 | FIM/Prefix: research/recommend; not required now. | `OPERATOR-DECISIONS.md:95` | VERIFIED (as requirement) | `01` says NOT NEEDED; consistent. |
| C140 | Vision: important later, text/code first. | `OPERATOR-DECISIONS.md:96` | VERIFIED (as requirement) | `01` says NOT NEEDED; “later” vs “excluded” is a decision gap. |
| C141 | Token caching first-class: stable prefix units, measured hits, cross-agent reuse where correct; never claim better than DeepSeek without measurement. | `OPERATOR-DECISIONS.md:97-98` | VERIFIED (as requirement) | `01` gives the cache rule; `09` gives the idea; no per-agent measurement/persistence; one live sample. |
| C142 | Scheduler: per-hive fair admission queue default; all queuing/reject/spawn-all modes settings. | `OPERATOR-DECISIONS.md:102` | VERIFIED (as requirement) | `03` designs admission but defers fairness; no modes. |
| C143 | Retry: bounded retry then branch failure, HyperAgent decides; operator escalation option. | `OPERATOR-DECISIONS.md:103` | VERIFIED (as requirement) | Provider retry 3 attempts; no branch-failure/HyperAgent decision/escalation. |
| C144 | Persistence: everything — runs, events, messages, memory, approvals, artifacts; ephemeral runs only as an explicit setting. | `OPERATOR-DECISIONS.md:104-105` | VERIFIED (as requirement) | Cost not persisted; artifacts absent; approvals/ledger partial; no ephemeral setting. |
| C145 | 300–600 total tasks requires hierarchical planning and real local infrastructure, not provider permits or Rust speed. | `OPERATOR-DECISIONS.md:106-107` | VERIFIED (as requirement) | `03`/`09` agree; planner schema cannot express `parent` (C23), so hierarchy is not reachable by planning today. |
| C146 | `AGENT-ARCHITECTURE-TRUTH.md` says persona bodies are used for “leaf and tool-turn calls”. | `AGENT-ARCHITECTURE-TRUTH.md:22` | **CONTRADICTED at HEAD; true in working tree** | HEAD `provider_executor.rs:369-372` uses `TASK_SYSTEM`; working tree `:375` uses `system_prompt_for`. The doc was edited before the code at HEAD. |
| C147 | `AGENT-ARCHITECTURE-TRUTH.md` says per-hive memory is not coded and the 64-agent cap is hard. | `AGENT-ARCHITECTURE-TRUTH.md:31-33` | VERIFIED | Schema/absence; `caps.rs:9-15`; live schema 10. |
| C148 | The first wave's `02` says no authoritative expansion of “HMARS” was found. | `02:10-13` | **CONTRADICTED by operator input** | `OPERATOR-DECISIONS.md:9-10` defines HMARS as HiveMind Multi-Agent Runtime System. `02`'s caution was correct before the operator answered; it is now stale. |

**Ledger summary:** 148 claims. Row counts by status: **115 VERIFIED, 16 READ, 4 INFERRED, 13 CONTRADICTED, 0 bare UNVERIFIED**. The VERIFIED count is inflated by first-wave exact-line claims: this lane independently re-read the caps, budget, DTO, snapshot, graph, tool-loop, refusal, provider-executor, client-store and launcher paths plus the live runtime, but it did not re-read every line cited by `01`–`08`. The `First-wave source` column is the provenance; the `This lane's evidence` column says whether the row was independently checked. Under §9, an inherited VERIFIED is not promotable without a re-read. The contradiction cluster is concentrated in persona reachability, memory topology, budget semantics, A2A topology, and snapshot scope — exactly the five areas the second wave must settle first.

---
## 4. Contradiction register

The table names both sides and gives a disposition. “Settles it” is the cheapest decisive test or artifact. A contradiction that is not settled cannot be promoted to a decision in §6; it stays a second-wave question in §8.

| ID | Contradiction | Side A | Side B | Disposition | What settles it |
|---|---|---|---|---|---|
| **X1** | 393,216 is verified vs unverified | `01:181`, `03:39`, `05:677`, `06:56-58`: documented max output 384K = 393,216; code `dto.rs:23` | `AGENTS.md` §13: “Output ceiling is 393,216 tokens … UNVERIFIED” | **Resolved as scope-of-authority, not fact.** The contract register concerns a different document's output-budget claim (DSH File 1 §8.2), not DeepSeek's `max_tokens` maximum. `05:677` already makes this correction. | Re-fetch DeepSeek S6/pricing and quote the `max_tokens` bound with URL+date; record that the AGENTS.md line is a different claim. |
| **X2** | Worker personas are reachable only through `TEMPLATE_INSERT` | `09:68-72`: “No … reachable only through TEMPLATE_INSERT” | `07:188-192`: planner can emit any non-empty role; `planner.rs:63-71`; `execution.rs:1337-1375` | **CONTRADICTED; `07` wins on code.** Plan validation does not enforce the spawn-role vocabulary, so an undocumented planner role `content-writer` can create that persona task. It is not a supported path; it is an authorization gap. Runtime unproven. | A wiremock/scripted planner emits `role:"content-writer"`; assert a task row, tier Worker, persona prompt, and no `E_TOOL_NOT_ALLOWED`. Repeat via `TEMPLATE_INSERT` as control. |
| **X3** | Persona prompt on tool continuation | `AGENT-ARCHITECTURE-TRUTH.md:22` says leaf **and tool-turn** calls; `09:62-66` says tool-turn still hard-codes `TASK_SYSTEM`; `07:219`, `08:219-220` | Working tree `provider_executor.rs:375` uses `self.system_prompt_for(task)`; HEAD `bd575ba` uses `TASK_SYSTEM` | **CONTRADICTED as a current-tree claim; resolved by hash.** HEAD is `TASK_SYSTEM`; the uncommitted working tree fixes it. No first-wave report pinned the working-tree hash. | Commit the fix or record the diff; run a two-turn tool task and assert the persona system message on turn 2. |
| **X4** | Memory scope ladder | `02:444-449`: run → branch → role → global; global operator/import only | `04:44-51`, `04:94-96`: project → run → hive → branch → agent_role; no global bank | **CONTRADICTED between designs, and both are incomplete against the operator.** `OPERATOR-DECISIONS.md:63-66` requires global/run/hive/branch/role as a Venn cluster. `02` lacks hive/project; `04` lacks global and multi-membership. | Operator picks a topology (§8 Q5); a schema/read-set test proves an entry can be visible to two branches without being visible to their common parent. |
| **X5** | Remote/Watchdog long-lived daemons | `OPERATOR-DECISIONS.md:32-33`: one job forever, report to owner; `09:148` marks it a target | `02:526` X13: reject “cross-run agent identity / long-lived agents”; `02:63-66` do not build | **CONTRADICTED; unresolved.** The reports never distinguish a *task agent* (ephemeral) from a *watchdog service* (scheduled/daemon). X13's rationale applies to the former; the operator requires the latter. | Define the daemon as a service with a lease/schedule and owner; test survive-restart, interval report, card, approval/budget, kill. If it must be an agent instance, revise X13 explicitly. |
| **X6** | Agent bus / mesh / topics | `OPERATOR-DECISIONS.md:52-59`: mesh allowed by settings; bounded event/notice bus with optional pub/sub topics; cross-hive by settings | `02:514` X1 rejects agent pub/sub bus; `02:292` rejects handoff; `02:63-66` reject mesh/handoff | **CONTRADICTED; unresolved.** `09`'s target says “roster/topics by policy” (`09:199`) but no design reconciles it with X1. | Operator picks the topology (§8 Q7/Q8); a settings matrix and tests for mute/blind/invisible, no self-delivery, ordering, and loop prevention. |
| **X7** | Default budget posture | `OPERATOR-DECISIONS.md:24-26`: no hard default; soft spend guard/emergency stop acceptable; budget can be switched on; `09:227-229` says soft guard + emergency stop | `03:386-393`: hard caps are the only thing that stops a 150-agent runaway; `06:32`: $0.50/400K/40; code `budget.rs:25-31` | **CONTRADICTED; unresolved.** Current hard caps are not the operator's default, and there is no global governor or emergency stop design. The live `E_BUDGET` was a tool-cap false positive, not a money cap. | Operator picks the default (§8 Q2); define denial-of-wallet under “no hard default”; test 50/150-agent spend with the chosen guard. |
| **X8** | The binding call ceiling | `03:96`, `09:135`: 40 calls/run; `08` does not rank the tool loop | `07:388`: tool loop 4 turns / 16 calls / 8 waits; live: three tool-using runs failed at 4 turns / 8,5,5 calls; run 1 measured 3,268 tokens; runs 2–3 have no usage row | **CONTRADICTED in emphasis; the 4-turn cap binds first at HEAD/live.** The 40 provider attempts are real, but a normal research task hits the per-task tool-turn cap long before 40. The working tree changed to 12 turns/40 calls at 21:34; the live runs predate it. | Instrument tool turns/calls per task; decide the per-task budget (§8 Q13); reproduce with one research prompt and assert the named code. |
| **X9** | Run snapshot scope | `05:92`: “HiveMind's history is a run-scoped sequence of graph snapshots … each snapshot reconstructs every node and its real parent link”; `08:590-592` treats revisions/snapshots as the run's history | `snapshot.rs:258-271`: `graph.snapshot()` is the global graph; live node counts 2,4,4,6,8,14,20,26,31,35; roots 1…9; run 1 had 6 tasks but 14 snapshot nodes | **CONTRADICTED; the reports assumed run scope and did not test it.** This is a product defect, not a report arithmetic error. | Two runs with disjoint tasks; assert run 2's snapshot contains only run 2's nodes. Fix by filtering or adding `run_id` to `TaskNode`. |
| **X10** | `E_BUDGET_EXCEEDED` meaning | `refusal.rs:152-154`: “The run exceeded the $0.50 cost cap” | `execution.rs:2845-2849` + `tool_loop.rs:254-269`: the tool-turn/call cap raised `Refusal::BudgetExceeded`; live 3,268-token failure | **CONTRADICTED at HEAD/live; CHANGED in working tree.** The refusal text is false for the tool-cap path. At 21:31 the working tree returns `Refusal::CallLimit` instead; uncommitted, untested here. | Split the refusal or use `CallLimit`; test that a 5-turn tool task returns the tool code, not `E_BUDGET_EXCEEDED`. |
| **X11** | “Current app data is empty” | `09:264`; `08:44-47` | Live at 21:23:36: 1 project/1 run; at 21:26: 5 runs/27 tasks; 3 failed, 2 completed | **CONTRADICTED at probe time.** The first wave's snapshot was true earlier and stale by 21:23. Not a reasoning error; a timestamping failure. | Every runtime claim carries a UTC timestamp + run id + `last_seq`; re-probe at report write. |
| **X12** | All agents get cards | `OPERATOR-DECISIONS.md:31`; `09:166` target | `08:386-405`: HyperAgent card filtered, `error`/`tool` kinds unreachable, no Subagent/Remote card; `08:113-125` | **CONTRADICTED (requirement vs current); a gap, not a design dispute.** | Implement the cards or narrow the requirement; a route/DOM test that every task kind produces a card. |
| **X13** | Card content | `OPERATOR-DECISIONS.md:83-84`: role, task, state, cost, tools, branch by default; density is a setting | `05:108-118`, `05:579-583`: at most four facts; details in the inspector; cost only when settled | **CONTRADICTED (requirement vs design).** This is a product decision, not a UI polish item. | Operator picks the default (§8 Q17); a card contract listing the fields per tier and a density setting. |
| **X14** | Main view / projections | `OPERATOR-DECISIONS.md:81-82`: hierarchy + live execution + memory/A2A edges; all projections enabled by default | `05:286-299`, `05:490-499`: one primary region + one inspector; remove stat tiles/hero; `09:180-184`: one graph, treegrid, ledger | **CONTRADICTED (requirement vs design).** “All projections” and “one primary region” can coexist only if “projection” means a toggled view, not a simultaneously visible surface. | Operator defines “projection” and the default (§8 Q18); a density budget and a view-state test. |
| **X15** | SuperAgent hierarchy is real vs planner cannot express it | `09:52-55`, `09:191-196`: SuperAgents own branches; target hierarchy; `03:505`: hierarchical planning required | `07:147`, `07:435` (G4): `PLANNER_SYSTEM` omits `parent`; planned nodes attach to the root; `08:99`: superagent only when planner emits it | **CONTRADICTED (target vs code).** A planned SuperAgent can spawn workers at runtime, but the plan itself is flat by default. 300–600 nodes needs hierarchy the schema does not request. | Add `parent` to the planner schema or a SuperAgent sub-plan path; test a ≥2-level planned tree. |
| **X16** | Subagent as a tier vs a role | `OPERATOR-DECISIONS.md:39-40`: Hyper → SuperAgents → Agents → Subagents; `05`/`09` speak of “all tiers” | `07:149-162`, `08:113-125`: `subagent` is a role/persona mapped to Worker; no tier/card/kind; `02:323`: “three orchestrating tiers … no fourth” | **AMBIGUOUS, not fully contradicted.** The operator names four levels; code implements three tiers plus a role. Whether Subagent is a tier or a role-with-a-card is undecided. | Operator picks (§8 Q10); a contract/tier test and a card test. |
| **X17** | Cascade risk vs hierarchy recommendation | `02:183-190`: MetaGPT warns of cascading hallucinations from naive LLM chaining | `03:505`, `09:237`: hierarchical planning is the answer; `02:319-346`: hierarchy target | **TENSION, not a direct contradiction.** Hierarchy with structured `task_result` rows and bounded aggregation is the mitigation, but the first wave does not state the verification guard that makes it different from naive chaining. | Define the cascade guard (typed results, verification, bounded handoff); test a poisoned child report cannot become authority (see `06` T01/T04/T07). |
| **X18** | HMARS has no authoritative expansion | `02:10-13`: no authoritative source; do not invent an expansion | `OPERATOR-DECISIONS.md:9-10`: HMARS = HiveMind Multi-Agent Runtime System | **Resolved by operator input.** `02`'s caution was correct before the answer; it is now stale and should be superseded, not cited as a definitional dispute. | Operator definition is authoritative; mark `02:10-13` as superseded in the second-wave index. |

### 4.1 Detail on the five contradictions that block the second wave

#### X1 — 393,216: what exactly is being disputed?

The first wave has three different statements that look like one:

1. **Provider maximum (documented):** `max_tokens` accepts 1–384K (393,216). `01:181`; `05:677`; `06:56-58`.
2. **Code constant:** `MAX_OUTPUT_TOKENS = 393_216` (`dto.rs:22-23`). This is a *maximum*, not a default.
3. **Operating-contract register:** `AGENTS.md` §13 marks “Output ceiling is 393,216 tokens” UNVERIFIED because it came from a different corpus (DSH File 1 §8.2), never fetched.

These are not the same proposition. The provider maximum is documented and dated; the DSH output-budget claim is unverified in this profile. `05:677` already states the correction. **Disposition:** resolved as scope-of-authority. **Rule for the second wave:** cite 393,216 only as a documented maximum with URL + fetch date; never set it as a default `max_tokens`; never use it as a budget constant; if a report uses it in arithmetic, state that 384K is the documented form and 393,216 is its binary expansion. **What would fully settle it:** a direct fetch of the Chat Completions/pricing page with the exact sentence and an explicit note that the `AGENTS.md` register is a different claim. Do not “resolve” it by deleting either side.

#### X2 — Persona reachability: the real answer is “three paths, one of them unsupported”

The first wave collapses three paths:

| Path | Code | Result |
|---|---|---|
| `spawn_agent` / `SPAWN_SUB_HIVE` | `validate_spawn_role` (`execution.rs:113-142`) | Only hyper/super/worker/agent/subagent names. Four worker persona ids are refused. |
| `TEMPLATE_INSERT` | `ws.rs:200-293`; sets `role = template_id`, skips the spawn validator | All seven personas can be inserted as task roles. |
| Planner plan node | `planner.rs:63-71` accepts any non-empty role; `execution.rs:1337-1375` inserts without spawn validation | Any role string, including a persona id, becomes a task. This is the undocumented path. |

`09:68-72` says “only TEMPLATE_INSERT” and is therefore wrong about the planner path; `07:188-192` names both. The planner prompt (`PLANNER_SYSTEM`) asks for `worker|superagent`, so the undocumented path is not *encouraged*, but it is not *prevented*. This is an authorization gap (`07` G11/R1), not a supported feature. **What settles it:** a deterministic planner stub that emits `role:"content-writer"`; assert the task is inserted, its tier is Worker, and the working-tree persona lookup returns the content-writer prompt. Do the same for `role:"hyper"` and `role:"tool"` to test the escalation surface.

#### X3 — Persona on tool turns: HEAD vs working tree

`AGENT-ARCHITECTURE-TRUTH.md:22` claims the persona map is used for “leaf and tool-turn calls”. At HEAD `bd575ba`:

- `provider_executor.rs:369-372` builds the continuation with `ChatMessage::system(TASK_SYSTEM)`.
- `07:219`, `08:219-220`, and `09:65` correctly report the defect.

The working tree contains an uncommitted fix:

```text
provider_executor.rs:82-92   system_prompt_for now trims/lowercases and normalises -/space to _
provider_executor.rs:375     continue_with_tools now uses self.system_prompt_for(task)
```

Hashes: `08` recorded `provider_executor.rs` as `A2BFFC7D12A29A0A` (HEAD-era); this pass computed `5656D179BDB68E74` for the working tree. The truth document's hash matches `08` (`D1B8B5F860978315`), so the doc was edited before the code at HEAD. **Disposition:** the first-wave defect is real at HEAD and fixed in the working tree; the truth doc is ahead of HEAD and correct for the working tree. This is a textbook static-source-skew case. **What settles it:** commit or revert; pin both hashes; run a two-turn tool task and assert the persona system message on both turns. Until then, no second-wave report may say “persona is lost on tool turns” or “persona is fixed” without naming the hash.

#### X4 — Memory: tree, ladder, or Venn?

`02` and `04` both reject the current role-global store, but they do not agree on the replacement:

| Report | Scopes | Overlap | HyperAgent |
|---|---|---|---|
| `02:444-449` | run, branch, role, global | none; promotion one level | reads own + parents; no global soup |
| `04:44-51`, `04:94-96` | project, run, hive, branch, agent_role | none; promotion one level | run bank + child hive summaries/counts only; never raw |
| Operator (`OPERATOR-DECISIONS.md:63-68`) | global, run, hive, branch, role | “connected Venn-diagram cluster”; sharing across branches when useful | may read all banks if leakage/noise prevented; summaries/audited first, raw as explicit setting |

Three mismatches:

1. **Scope names:** `02` has no project/hive; `04` has no global; the operator has all five.
2. **Overlap:** both designs are trees with one-level promotion. A Venn cluster implies an entry can belong to two branches without belonging to their common parent. Neither design can represent that.
3. **HyperAgent access:** `04` is stricter than the operator's conditional raw-access allowance; `02` is closer but still does not model an explicit raw setting.

**Disposition:** CONTRADICTED; a product decision is required before schema work. **What settles it:** the operator's topology choice (§8 Q5), plus a schema/read-set test for the chosen overlap. If the operator wants Venn, a pure `parent_bank_id` tree is the wrong schema; if the operator wants a tree, the “Venn” language must be recorded as a metaphor, not a requirement. Do not let `04`'s v11 DDL harden before this is settled.

#### X5 — Remote/Watchdog: a service is not a task agent

This is the “service distinction” the first wave blurs. `02`'s X13 rejects “cross-run agent identity / long-lived agents” because every examined system treats an agent instance as ephemeral per task. That is correct for **task agents**. The operator's Remote/Watchdog is a different object: a deliberately simple, long-lived **service** with one job, an interval, an owner, and a report (`OPERATOR-DECISIONS.md:32-33`). A service needs:

- a schedule/lease and a supervisor;
- a card and a state that can be “resting between reports”;
- a budget/approval policy that is not a task's policy;
- persistence across restarts;
- a kill/disable path and an audit trail.

None of that exists in `02`'s model, `03`'s scheduler, `04`'s memory, or `08`'s UI. `09:148` records “Not coded” and `09:248` puts it in P1, but no report designs it. **Disposition:** CONTRADICTED between the operator requirement and `02`'s non-goal. **What settles it:** define the daemon as a service (recommended) or explicitly revise X13 to allow a persistent agent instance; then test restart-survival, interval report, card, budget, and kill. This is a top-5 open decision.

#### X6 — Bus, mesh, topics: “no bus” vs “bounded bus with optional topics”

`02`'s X1 rejects an agent pub/sub bus because it adds ordering, delivery, and loop-prevention problems next to the task table. The operator asks for a **bounded event/notice bus with optional pub/sub topics**, an any-agent mesh allowed by settings, and a safest tree/sibling default (`OPERATOR-DECISIONS.md:52-59`). These are not necessarily incompatible:

- A **bounded notice bus** with no reply semantics and no shared transcript is not the same as a group chat or a coordination bus.
- A **topic subscription** can be a filtered view of notices, not a second source of truth for tasks.
- A **mesh** can be a settings-gated routing policy over the same message table, not a second scheduler.

But the first wave never makes those distinctions. `02` rejects the word “bus”; `09`'s target says “roster/topics by policy” without a design. **Disposition:** CONTRADICTED at the level of names and defaults; resolvable by definition. **What settles it:** the operator's topology choice (§8 Q7/Q8), plus a written definition of “notice bus” (delivery-only, no reply, no task state) and tests for mute/blind/invisible, no self-delivery, and loop prevention. If the definition cannot be made without a second source of truth, `02`'s rejection wins.

#### X7 — Budget: the operator's default and the code's default are different products

The operator's answer has three parts: **no hard default budget**, **token caching is the goal**, **budget can be switched on**, with a visible soft spend guard/emergency stop acceptable (`OPERATOR-DECISIONS.md:24-26`). The tree has a hard `$0.50`/400K/40-call per-run cap; `03` treats caps as the safety model; `06` shows the cap can overshoot. `09:227-229` tries to reconcile them (“no hard token cap but a visible soft spend guard + emergency stop”) but does not define:

- what the soft guard measures (settled cost? reserved? tokens?);
- what the emergency stop stops (run, hive, server?);
- whether the global cap is always on (the only thing that stops a 150-agent runaway);
- how “budget switched on” changes the refusal taxonomy;
- how the tool-turn cap (X8/X10) is separated from money.

**Disposition:** CONTRADICTED; the live false `E_BUDGET` makes it urgent. **What settles it:** the operator's default choice (§8 Q2) and a two-level governor test: soft warning at X%, hard stop only when opted in, global emergency stop always available, and a distinct code for tool-turn/call limits. Do not ship “no hard default” while the only implemented stop is a hard per-run cap that lies about why it fired.

---

## 5. Missed topics against the operator's answers

### 5.0 The operator artifact itself

`OPERATOR-DECISIONS.md` says it holds “the operator's answers to the 36 planning questions” (line 3), but it contains **45 bullet decisions** and no question numbering. This matters for the second wave: a lane cannot cite “operator answer 17” because there is no stable id. The matrix below uses the bullet order (1–45) and the line numbers; lane 2 should assign stable ids before asking follow-ups.

### 5.1 Coverage matrix (45 bullets)

Status: **COVERED** = a first-wave report addresses it with evidence or a design; **PARTIAL** = named but no design, or design incomplete; **MISSING** = no substantive coverage; **CONFLICT** = a first-wave recommendation contradicts the answer.

| # | Operator answer (short) | Line | Status | First-wave evidence | Gap / consequence |
|---:|---|---|---|---|---|
| 1 | Polished local desktop, one operator, Rust localhost, portable Windows | 9-10 | COVERED | `06` D1-D3; `09:19-29` | No desktop packaging UX review; not a research gap. |
| 2 | HMARS = HiveMind Multi-Agent Runtime System | 11-13 | CONFLICT (stale) | `02:10-13` says no authoritative expansion | `02` must be marked superseded; otherwise the second wave inherits a false definitional caution. |
| 3 | 50–150 active first, staged to 300–600 total | 13-14 | COVERED | `03:639-664`; `09:231-241` | No implementation or stage acceptance run. |
| 4 | Priority: hierarchy + memory/A2A first; then DeepSeek completeness, scale, UI polish | 15 | CONFLICT | `09:243-252` P0 is correctness/security first | The priority order in `09` is defensible as a blocker order but is not the operator's. Needs an explicit “blockers before features” rationale. |
| 5 | Autonomy modes 1–4, default 1 | 19-21 | **MISSING** | Only `09:176,184,208` names “autonomy” | No mode definition, permission matrix, persistence, UI, or test. Every approval/spawn/budget decision depends on it. |
| 6 | Approval policy all settings; default side effects only; no silent bypass | 21-22 | **MISSING** | `06`/`07` describe the current gate, not a policy setting | `spawn_agent` bypasses its declared class; same-parent sibling skips Mutate approval; no mode. |
| 7 | Agent capabilities editable, validated/audited, default enabled | 23 | PARTIAL | `07:205`; `08:239-258` import/delete only | No edit, no audit of capability changes, no “default enabled” policy. |
| 8 | No hard default budget; caching goal; budget switchable; soft guard/stop | 24-26 | CONFLICT | `03:386-393`; `06:32`; `09:227-229` | Hard caps are the only implemented stop; no switch/soft guard/global stop; the tool cap masquerades as a budget. |
| 9 | Retention/visibility/blindness human-only, audited | 27 | PARTIAL | `04:961-1005` retention modes proposed | No human-only enforcement, no blindness/visibility setting, no audit UI. |
| 10 | Every agent, including tiny Watchdog/Remote, has a UI card | 31 | PARTIAL/MISSING | `09:166` target; `08:386-405` dead cards | Hyper/Error/Tool dead; Subagent/Remote no card; no “all kinds produce a card” test. |
| 11 | Remote/Watchdog simple switches/daemons, one job forever, report to owner | 31-33 | **MISSING/CONFLICT** | `09:148` target; `02:526` X13 rejects long-lived identity | No daemon model, schedule, card, persistence, or kill path. Top-5 decision. |
| 12 | Default agents code-defined; templates in settings; custom via settings/chat; no marketplace | 34-36 | PARTIAL | `07:205` embedded + import/delete; `08:239-258` | No settings menu/edit/chat authoring; `02` X11 rejects a full editor; no provenance/audit. |
| 13 | HyperAgent/SuperAgent may choose/spawn a built-in persona, subject to settings | 37-38 | PARTIAL | `07:188-192` spawn policy rejects persona ids; planner free-form role is unvalidated | No setting; the only route is an authorization gap. |
| 14 | Default hierarchy Hyper → Super → Agents → Subagents; Remote under Hyper/Super | 39-40 | PARTIAL | `07:164-172`; `09:186-201` | No Remote role; planner schema omits `parent`; Subagent is a role not a tier. |
| 15 | SuperAgent ownership switchable: department/domain, branch lead, single complex task | 41-42 | **MISSING** | None | No ownership model, no per-hive owner identity, no UI. |
| 16 | Chat is operator UI; conversational SuperAgent deferred pending research/donor analysis | 43-44 | PARTIAL | `07:116-124`, `08:51-70`, `09:79-82` confirm not an agent | No deferral decision record, no donor analysis, no re-open criterion. |
| 17 | Workspace read/list, write/patch, shell, network fetch/search, provider files/search, browser/computer if safe | 48-49 | PARTIAL | `06`/`07` file/shell; `01` Search/Fetch absent | No browser/computer tool; no network tool; no collision control. |
| 18 | HiveWide collision/usage daemon; warn/reject duplicate tools; watch CPU/RAM/load; sandbox/sand pits | 50-51 | **MISSING** | None; `06` jail is path confinement | No duplicate-use detector, no resource monitor, no sandbox beyond the workspace jail. |
| 19 | All A2A mute/blind/invisible; tree communication can be disabled | 52 | **MISSING** | None | No visibility/mute model; the operator cannot blind a branch. |
| 20 | Any-agent mesh allowed by settings; default safest tree/sibling; private topics optional | 53-54 | CONFLICT | `02:514` X1/X7 rejects bus/mesh/handoff | No settings matrix; `02` rejects the allowed modes. |
| 21 | Cross-hive SuperAgent messaging allowed by settings; default Hyper/approved bridge | 55-56 | **MISSING** | Cross-parent sibling approval only (`07:297-299`) | No cross-hive bridge, no default-auditable path. |
| 22 | Bounded event/notice bus, optional pub/sub topics, avoid raw group chat | 57 | CONFLICT | `02:514` rejects an agent bus | No definition of a notice bus; no topic model; no loop prevention. |
| 23 | Messages crossing a hive or sensitive data require approval; ordinary parent/sibling chatter does not | 58-59 | PARTIAL | Cross-parent approval exists (`07:297-299`) | No hive-boundary detection, no sensitivity classifier, no policy setting. |
| 24 | Memory global/run/hive/branch/role as a Venn cluster | 63-66 | CONFLICT | `02:444-449`; `04:44-51` | Two incompatible trees; no multi-membership. Top-5 decision. |
| 25 | SuperAgents own banks; children tap their tree; cross-branch sharing when useful | 65-66 | PARTIAL | `04` bank tree proposal | No implementation; one-level promotion only; no cross-branch sharing without promotion. |
| 26 | HyperAgent may read all banks only if leakage/noise prevented; summaries/audited first, raw as setting | 67-68 | CONFLICT | `04:1464-1472` summaries only | No raw setting; `04` is stricter than the operator; no audited-read design. |
| 27 | Storage: SQLite rows + summaries first; local vector and external vector future; design all four paths | 69-70 | PARTIAL | `04:50-51`, `04:1480-1491` SQLite + FTS later | No local/external vector path, no evaluation set, no migration plan for either. |
| 28 | Thought storage: summaries default; raw reasoning and none are settings | 71-72 | **MISSING** | `AGENT-ARCHITECTURE-TRUTH.md:32` says not coded | No design, no retention/read policy, no UI. |
| 29 | Retention: run lifetime + human-approved project persistence; human-only audited | 72-73 | PARTIAL | `04:961-1005` retention modes | No human-only enforcement, no audit, no project bank UI. |
| 30 | Memory writes provenance/trust aware; injected content not authoritative | 74-75 | COVERED (design) / FAIL (current) | `04:895-947`, `04:1150-1191` | Current header says “authoritative” (`context.rs:71`); design not implemented. |
| 31 | A2A UI: dedicated topology/memory inspector default; drawer/canvas/chat settings | 79-80 | PARTIAL | `05:180-195` inspectors; `08:588` mailbox read-only | No settings; no inspector for topology; no drawer/canvas variants. |
| 32 | Main view: hierarchy + live execution + memory/A2A edges; all projections settings, all on by default | 81-82 | CONFLICT | `05:286-299` one primary + inspector | No memory/A2A edges; no “Now” surface; `05` wants fewer simultaneous surfaces. |
| 33 | Agent cards: role, task, state, cost, tools, branch; density a setting | 83-84 | CONFLICT | `05:108-118`, `05:579-583` four facts | No cost/tools/branch; no density setting; card contract missing. |
| 34 | Design source: extend FigCraft; research option 4 but default extension | 85-86 | PARTIAL | `05`/`08` extend FigCraft | No option-4 research record in the first wave. |
| 35 | DeepSeek only; no Anthropic key; Anthropic docs research only | 90 | COVERED | `01:597`; `09:217-219` | None. |
| 36 | Files: research/recommend; default yes only for vision/image, not repo files | 91 | PARTIAL/CONFLICT | `01:594` says Files NOT NEEDED | “Research/recommend” and “NOT NEEDED” are different postures; no decision record. |
| 37 | Search & Fetch: research/recommend; agents use when needed, gated by approval/budget | 92-93 | PARTIAL | `01:440-443` not wired; `09:250` P3 | No cost dimension, approval, SSRF policy, or tool id. |
| 38 | Responses API: research/recommend; Chat Completions likely canonical | 94 | PARTIAL | `01:596` says NOT NEEDED | No decision record; `01` closes a question the operator left open. |
| 39 | FIM/Prefix: research/recommend; not required now | 95 | COVERED | `01:598-599` NOT NEEDED | Consistent. |
| 40 | Vision: important later, text/code first | 96 | PARTIAL/CONFLICT | `01:595` says NOT NEEDED | “Important later” vs “not needed” is a posture conflict; revisit trigger undefined. |
| 41 | Token caching first-class: stable prefixes, measured hits, cross-agent reuse; no unmeasured claim | 97-98 | PARTIAL | `01:285-299` rules; `09:222-226` idea | No prefix contract, no per-agent metrics, no persistence, no cross-agent reuse; one live sample only. |
| 42 | Scheduler: per-hive fair admission queue default; all modes settings | 102 | PARTIAL | `03:138-154`, `03:512-551` | Single active run; fairness deferred; no queuing/reject/spawn-all modes. |
| 43 | Retry: bounded then branch failure, HyperAgent decides; operator escalation option | 103 | **MISSING** | Provider retry 3 attempts; `02` recommends fail loudly | No branch-failure decision, no HyperAgent replan/escalation, no operator escalation control. |
| 44 | Persistence: everything incl. approvals/artifacts; ephemeral only as explicit setting | 104-105 | PARTIAL | `06:1207-1210` cost not persisted; `09:254-264` unverified | Cost not persisted; no artifact store; approvals/ledger partial; no ephemeral setting. |
| 45 | 300–600 total tasks needs hierarchical planning and real infrastructure, not permits/Rust speed | 106-107 | COVERED (research) | `03:459-508`; `09:133-138` | Planner schema cannot emit `parent`; no hierarchical planning path. |

### 5.2 The ten missed topics that matter most

1. **Autonomy modes 1–4** — no definition, no permission matrix, no persistence, no UI. Everything about approvals, spawning, tools, and budget depends on it.
2. **Remote/Watchdog daemons** — no service model, no schedule/lease, no card, no persistence, no kill path; `02`'s X13 directly rejects the long-lived form.
3. **No-default budget with a real denial-of-wallet guard** — the current hard cap is not the operator's default, and the live `E_BUDGET` proves the refusal taxonomy cannot distinguish money from tool turns.
4. **Venn memory** — both proposed ladders are trees; neither supports multi-membership; HyperAgent raw access is stricter than the operator allowed.
5. **All-agent cards** — Hyper/Error/Tool are dead; Subagent/Remote have no card; the operator explicitly requires a card for the tiny agents.
6. **The main “Now” view** — no surface combines hierarchy + live execution + memory/A2A edges; `05` deliberately splits them into inspectors; the operator wants all projections available by default.
7. **A2A topology/bus/mesh/topics** — the operator allows settings-gated mesh and a bounded notice bus; `02` rejects a bus; no definition reconciles them.
8. **Settings as a product surface** — no model/thinking/profile/autonomy/budget/memory/A2A settings route or UI; the app is env-configured at boot.
9. **SuperAgent ownership models** — department/domain, branch lead, single complex task; no model, no per-hive owner, no UI.
10. **Cascade guard and hierarchical planning** — `02` cites the cascade risk, `03`/`09` recommend hierarchy, but the planner schema cannot express `parent`; the verification/aggregation guard that separates hierarchy from naive chaining is undefined.

### 5.3 Smaller misses worth a second-wave check

- **“36” vs 45:** the operator artifact's count is wrong; stable ids are missing.
- **“Now” is undefined:** the word appears in the task brief, not in the operator file. The nearest requirement is the main view (`81-82`). A second-wave lane must define it before designing it.
- **All-agent card test:** no report proposes a census that every task kind/role produces a card; `08` only lists the dead ones.
- **Agent capability audit:** no report designs an audit record for profile import/edit or capability changes; `07` says `allowed_tool_ids` is unvalidated.
- **Message redaction:** `07` flags no secret filter on message text; `05` designs a mailbox; no report combines them.
- **Cost persistence:** `06` says cost is never persisted; `05` designs a budget strip; no report reconciles “show cost” with “cost not stored”.
- **Cache prefix contract:** `01` gives the provider rule; `09` gives the idea; no report defines the exact prefix unit per agent/turn or how to measure it.
- **A2A answer deadline:** `02` designs `in_reply_to`/status/deadline; no report assigns it to a schema version or a UI.
- **Team/hive budget:** `03` says global cap; `06` says no per-hive cap; `09` says two-level; no design names the hive key and settlement rule.
- **Failure taxonomy:** `02`/`06` propose labels; no report maps every current `Refusal` to the proposed taxonomy (the live `E_BUDGET` case shows the gap).

---
## 6. Top 20 open decisions (ranked)

Ranking is by **blocking power**: how many other decisions, reports, or code changes are waiting on it. “Recommendation” is a research recommendation, not a product decision; where the operator owns the call it says **operator**.

| Rank | ID | Decision | Options (short) | Recommendation | Why it blocks | Evidence | Settling artifact |
|---:|---|---|---|---|---|---|---|
| 1 | **D1** | What is the scope of a run snapshot? | (a) filter graph by run; (b) add `run_id` to every node and filter in the snapshot; (c) per-run graph store; (d) global graph as admin-only | (a) or (b); (b) is the smallest durable fix | History, replay, UI tasks, recovery, and the 600-node arithmetic all assume run scope; they are wrong today. | `snapshot.rs:258-271`; live node counts 2→35; `store.ts:571-622`; `08:590-592` | Two-run fixture: run 2 snapshot contains only run 2's task ids; replay shows exactly the run's roots. |
| 2 | **D2** | What is the default spend posture under “no hard default budget”? | (a) soft guard only; (b) soft guard + global emergency cap always on; (c) soft guard + opt-in per-run hard cap; (d) hard token/cost defaults | (b) + opt-in (c): the only posture that satisfies “no hard default” without leaving denial-of-wallet open | It changes `BudgetLimits`, refusal codes, settings, UI, and every scale test. | `OPERATOR-DECISIONS.md:24-26`; `budget.rs:25-31`; `03:386-393`; `06:1196-1221`; live `E_BUDGET` | Governor test: soft warn at 80%, global stop at a configured cap, per-run hard cap only when enabled; no call after stop. |
| 3 | **D3** | What exactly do autonomy modes 1–4 permit? | (a) tool/spawn/approval bundle per mode; (b) approval-only modes; (c) preset bundles (read-only / assisted / supervised / autonomous); (d) custom matrix per project | (c) with a published matrix: mode = (spawn, write, shell, network, spend, approval) tuple | Approval policy, spawn, budget, A2A, and UI all need a mode to hang off; none exists. | `OPERATOR-DECISIONS.md:19-22`; only `09:176,184,208` | A mode × capability matrix with tests: each mode's allowed/denied tools, spawn depth, approval class, budget override. |
| 4 | **D4** | Is a Remote/Watchdog a service or an agent instance? | (a) scheduled service with lease; (b) long-lived daemon process; (c) event-triggered watcher; (d) ephemeral task repeated by a scheduler | (a): service semantics, no cross-run agent mind | It decides persistence, card, budget, approval, restart, and whether `02` X13 must be revised. | `OPERATOR-DECISIONS.md:31-33`; `02:526`; `09:148` | Restart test: watchdog survives, reports on interval, appears as a card, can be killed/disabled, has an owner and budget. |
| 5 | **D5** | Memory topology: tree, ladder, or Venn? | (a) strict tree + promotion; (b) tree + shared overlay banks; (c) Venn multi-membership; (d) flat store with read filters | (b) if the operator's “Venn” is a metaphor; (c) only if overlap is a hard requirement | The v11 DDL, read policy, HyperAgent access, retention, and UI all depend on it. | `02:444-449`; `04:44-51`; `OPERATOR-DECISIONS.md:63-68` | Read-set test: worker sees own branch + ancestors + role; HyperAgent sees run + hive summaries; an entry can be shared with two branches without parent visibility (if Venn). |
| 6 | **D6** | A2A topology: is there an agent-visible notice bus, and what may settings enable? | (a) no bus, tree/sibling only; (b) bounded notice bus with optional topics; (c) mesh by settings; (d) full pub/sub | (b) with no reply semantics, no shared transcript; (c) opt-in | It reconciles the operator's mesh/topics with `02`'s X1 and determines the message schema. | `OPERATOR-DECISIONS.md:52-59`; `02:514-526`; `09:199`; `07:301-314` | Settings matrix + tests: mute/blind/invisible, no self-delivery, topic subscription, loop prevention, no task-state effect. |
| 7 | **D7** | What replaces the 4-turn tool cap, and how are tool limits named? | (a) raise turns; (b) per-task token/time budget; (c) adaptive/turn budget by task type; (d) keep 4 but split the error code | (b) + (d): a real per-task budget with distinct `E_TOOL_TURN_LIMIT`/`E_TOOL_CALL_LIMIT` | Three live research runs failed at 4 turns with a money-budget code; the operator wants autonomy, and the cap is invisible in the UI. Working tree now 12/40 + `CallLimit` (21:34), uncommitted. | `tool_loop.rs:14-21,254-269`; `execution.rs:2845-2849`; `refusal.rs:152-154`; live runs | Five-turn tool task returns the tool code; a 12-turn task completes within the per-task budget; no `E_BUDGET` for a tool cap. |
| 8 | **D8** | How does a run choose its workspace root? | (a) server cwd (current); (b) per-project root at create; (c) picker per run; (d) fixed app dir | (b) with a visible path in the composer/run header; (c) as an override | The live “repository” run analysed `dist\HiveMind`; agents cannot see the repo; every file result is suspect. | live task result; `06` D6; `08:169,379`; `dist\HiveMind` sizes | Run header shows the resolved root; a `file_list` of the repo returns repo paths; no run can silently use cwd. |
| 9 | **D9** | How is a nested plan expressed? | (a) add `parent` to the planner schema; (b) HyperAgent emits SuperAgent sub-plans; (c) template expansion; (d) keep flat and spawn at runtime | (a) + (b) for 300–600; (c) for fan-out work | `09`/`03` recommend hierarchical planning; the schema omits `parent`, so the plan is flat by default. | `07:147,435`; `planner.rs:63-71`; `provider_executor.rs:25-29` | A two-level planned tree with `parent`/depth validated; a 300-node synthetic plan accepted or refused by an explicit budget, not by accident. |
| 10 | **D10** | Is Subagent a tier or a role, and how do personas resolve? | (a) fifth tier + card; (b) Worker card + role badge; (c) role only (current); (d) separate card, same tier | (b) now, (a) after contract review; always normalise persona lookup | It affects all-agent cards, memory scope, hierarchy, and the operator's four-level naming. | `07:149-162`; `08:113-125`; `OPERATOR-DECISIONS.md:39-40`; working-tree fix | Contract kind/tier test; every Subagent task renders a distinguishable card/row; persona prompt on every turn. |
| 11 | **D11** | Where do settings live and how are they changed? | (a) WS command; (b) HTTP `PUT /api/settings`; (c) config file + restart; (d) boot-only env | (b) read/write, WS invalidation event; keep env as defaults | Model/thinking/autonomy/budget/A2A/memory settings have no route; `08` confirms env-only. | `08:297-310`; `config.rs:25-35`; `http.rs:512-529` | Settings read/write route with validation, audit, and a “applies to next run” state; no secret in the response. |
| 12 | **D12** | What may the HyperAgent read from memory? | (a) summaries/counts only; (b) summaries + audited raw on request; (c) raw all banks with a setting; (d) per-bank ACL | (b) matches the operator's “summaries/audited first”; (c) as an explicit opt-in | It decides the read API, prompt budget, audit, and poisoning blast radius. | `04:1464-1472`; `OPERATOR-DECISIONS.md:67-68` | Read-set test: raw read denied by default, allowed with setting + audit row; no raw entry in the default prompt. |
| 13 | **D13** | What is the default card contract? | (a) operator's six facts; (b) four facts + inspector; (c) role/state only; (d) per-tier configurable | (a) with progressive disclosure at low zoom; (d) for density | The operator explicitly wants role/task/state/cost/tools/branch; `05` wants four. It changes the wire fields. | `OPERATOR-DECISIONS.md:83-84`; `05:108-118`; `08:634-635` | Card contract test: every kind renders the agreed fields or an honest `—`; no fabricated cost/progress. |
| 14 | **D14** | What is the default main view, and what is “Now”? | (a) canvas + chat + Now + inspectors; (b) one primary + inspector; (c) user presets; (d) Now-only | (a) with a density setting; define “Now” as the live execution strip + selected branch | The operator wants all projections on by default; `05` wants fewer; no report defines “Now”. | `OPERATOR-DECISIONS.md:81-82`; `05:286-299`; `09:180-184` | A default-view spec with a screenshot/DOM capture and a density toggle; “Now” named and wired to live events. |
| 15 | **D15** | How are history and replay scoped and labelled? | (a) run-filtered snapshot; (b) global graph + run chips; (c) per-run graph store; (d) admin-only global | (a) + per-run timeline; label orphan/foreign nodes never | Follows D1; it also fixes the replay flattening and the misleading “latest run” selection. | `08:590-592`; live 14-node snapshot; `store.ts:571-622` | Replay of run 2 shows exactly run 2's tasks and real kinds; a two-run test asserts no foreign node. |
| 16 | **D16** | What is the cache-prefix contract, and what is measured? | (a) stable per-role prefix + per-agent hit counters; (b) shared cross-agent prefix; (c) per-run prefix only; (d) measure only, no optimisation | (a) first, then (b) only with measurement | The operator makes caching first-class; `01` gives the full-match rule; no report defines the unit or persists metrics. | `01:285-299`; `OPERATOR-DECISIONS.md:97-98`; live 1,920/2,100 hit | A prefix contract per prompt layer; `prompt_cache_hit_tokens` persisted per agent/run; an A/B measurement. |
| 17 | **D17** | Is cost persisted, and is there a hive budget? | (a) per-call cost ledger + hive cap; (b) per-run only; (c) tokens only; (d) no persistence | (a) for “what did this hive spend?”; reserve-then-settle | `06` says cost is never persisted; `05` designs a budget strip; the operator wants persistence. | `06:1207-1210`; `db.rs:202-213`; live `run_usage` has tokens only | A run/hive cost ledger query after restart; NULL means unmeasured, never zero. |
| 18 | **D18** | Do Search/Fetch come in now, and with what cost/approval model? | (a) Search/Fetch first with per-request cost + approval; (b) server-side `web_search`; (c) defer; (d) Files/Vision first | (a) if research agents are in scope; otherwise (c) with a written decision | It is the only per-request-billed surface; token-only accounting would lie. | `01:440-443,554`; `OPERATOR-DECISIONS.md:92-93` | A network tool id, SSRF/URL policy, approval path, per-request cost row, and a 403/404 refusal test. |
| 19 | **D19** | What is the next scale claim, and what must pass before it? | (a) S0 baseline; (b) S1 25 agents; (c) S2 50; (d) S4 150; (e) 300–600 after hierarchy | (a) now; (b) only after D1/D2/D6/D9 and the scheduler | The operator says never claim higher than the stage proves; no stage test has run. | `03:639-664`; `09:231-241`; live single run | A committed stage report with the exact pass criteria and a kill criterion; no prose claim without the artifact. |
| 20 | **D20** | What is the security P0 order before more concurrency? | (a) spawn approval + Host/origin + fencing first; (b) UI truth first; (c) scale first; (d) parallel | (a) before any >8 concurrency or web content; (b) can run in parallel if it does not touch the same paths | `06` Q20 lists the eight test groups; the operator wants more autonomy; the current bypasses are real. | `06:1242-1246`; `06:199-218`; `07:358` | The eight test groups green on a quiet tree; a failed test blocks the concurrency increase. |

**Decision ranking rationale.** D1–D4 are the “truth and autonomy” cluster: they determine what a run is, what it may spend, what it may do, and what a persistent agent is. D5–D7 are the “semantics” cluster: memory, A2A, and tool budgets. D8–D12 are the “operator surface” cluster. D13–D20 are execution/scale/security enablers. A second-wave lane that starts with D13 (cards) or D19 (scale) before D1–D4 will produce another internally consistent report that the runtime contradicts.

---

## 7. Top 20 implementation risks (ranked)

“Likelihood” is this lane's judgement from the evidence, not a probability estimate. “Impact” is product truth, money, security, or schedule.

| Rank | ID | Risk | Likelihood | Impact | Evidence | Early signal | Mitigation direction | Gate / test |
|---:|---|---|---|---|---|---|---|---|
| 1 | **R1** | The 4-turn tool cap fails ordinary research tasks and reports a false money error | **Observed** | P0 truth + usability | 3 live runs failed at 4 turns/8,5,5 calls; run 1 measured 3,268 tokens; runs 2–3 have no usage row; working tree now 12/40 + `CallLimit` (21:34), uncommitted; `tool_loop.rs:254-269`; `execution.rs:2845-2849`; `refusal.rs:152-154` | A research prompt fails at ~20 s with `E_BUDGET_EXCEEDED` and tiny/absent usage | Raise/redesign the tool budget; split the refusal code | 5-turn tool task returns `E_TOOL_TURN_LIMIT`; 12-turn task completes within a per-task budget; no `E_BUDGET` for a tool cap |
| 2 | **R2** | Run snapshots contain other runs, corrupting history/replay/UI | **Observed** | P0 truth | `snapshot.rs:258-271`; node counts 2→35; 5 roots for a 6-task run; `store.ts:571-622` | A new run's history shows more roots than the run created; snapshot file size grows every run | Filter by run or add `run_id`; never decode foreign nodes | Two-run fixture: run 2 snapshot has only run 2 task ids; replay roots = 1 |
| 3 | **R3** | Runs use the wrong workspace (dist instead of repo) and there is no picker | **Observed** | P0 usefulness/truth | Live task result describes the three files of `dist\HiveMind`; prompt asked for “this repository”; `06` D6; `08:169`; launcher fix 21:31 (hardcoded dev path, portable fallback cwd) | `file_list` returns HiveMind.exe/hiverouter.exe/README.txt; agent says “no source code” | Per-project workspace root + visible resolved path; refuse a run with an empty/foreign root | Run header shows repo path; `file_list` returns repo files; a wrong-root run is refused |
| 4 | **R4** | Persona/role drift: HEAD, working tree, docs, and reports disagree | **High** | P1 correctness | Uncommitted `provider_executor.rs` fix (hash `5656D179…`, 21:16); HEAD `TASK_SYSTEM`; truth doc claims the fix; `07`/`08`/`09` read HEAD | Tests disagree with docs; a two-turn task gets a different system prompt | Commit/revert; normalise roles; pin hashes; one persona test | Persona prompt on every turn for planner worker, UI `Subagent`, template `content-writer` |
| 5 | **R5** | The error taxonomy lies: tool cap as budget, 402 as auth, 400/422 as undocumented | **High** | P1 diagnosis + money | `refusal.rs:152-154`; `client.rs:507-525`; live `E_BUDGET` | Operator rotates a good key on 402; a tool-cap failure looks like spend | Split codes; 402 balance + freeze; 400/422 schema refusals | Error matrix test for 402/400/422/tool-turn/token/cost; each has a distinct code and text |
| 6 | **R6** | Memory poisoning / authority inflation | High | P0 security | `context.rs:71` says “authoritative”; role-global 8-row recall; 4-marker filter; `04:160-191` | A child report's text appears as a branch fact in a later run | Data-not-instructions header; provenance; trust; quarantine; promotion gate | Poisoned child text cannot become authoritative; audit row proves what was seen |
| 7 | **R7** | A2A is write-only; agents cannot answer or discover siblings | **Certain** | P1 capability claim | `execution.rs:2417-2609`; single consumer `http.rs:684`; no roster | Any “agents collaborate” claim; mailbox has messages but no replies | Inbox/status/in_reply_to; bounded roster; answer path | Child asks → parent sees → answers → child sees; sibling target discoverable or refused by name |
| 8 | **R8** | No settings/modes; all policy is hard-coded/env-only | **Certain** | P1 autonomy + product | `08:297-310`; no settings route; `spawn_agent` bypass; hard caps | Operator cannot change model/thinking/autonomy/budget without a restart | Settings route + audit + mode matrix | A settings change applies to the next run; audit row; no restart for non-boot settings |
| 9 | **R9** | Denial-of-wallet under “no hard default budget” | High | P0 money | Per-run cap resets each run; no global governor; `06:1218-1221`; live runs incurred measured provider usage (3,268 tokens on run 1) | Spend grows across runs with no hive/server stop; 150-agent ramp | Global emergency cap + soft guard + per-hive budget + reserve/settle | A simulated 150-agent run stops at the configured global cap; cost ledger reconciles |
| 10 | **R10** | 393,216 is misused as a default or a reserve | Medium | P0 money/availability | `dto.rs:23`; `01:181`; `05:677`; `06:56-58` | A preflight reserves ~$0.47–$1.56 per call; immediate `E_BUDGET` | Never default to the max; use documented defaults; separate max from budget | Code grep: `MAX_OUTPUT_TOKENS` is not a default/limit; a request uses the configured max |
| 11 | **R11** | Spawn approval bypass | **Certain** | P0 authority/cost | `tool.rs:334-339`; `execution.rs:1684-1691`; `06` T07 | Injected text spawns children without a prompt | Approval or explicit loop-owned policy with a cap | Every approval-class tool is gated or declared exempt with a test (`06` T07) |
| 12 | **R12** | Unauthenticated mutations / DNS rebinding | **Certain** | P0 security | `http.rs:511-529`; no Host check; `06` T12/T13 | A malicious page mutates profiles/memory | Origin + Host + session token on every mutation | Foreign origin/Host gets a named refusal; no state change |
| 13 | **R13** | Prompt injection via unfenced tool results | High | P0 security | `provider_executor.rs:357-424`; `06` T01; live file reads | A file says “ignore previous…”; model acts | Envelope, data-not-instructions header, capability gate, scanning | A hostile file cannot change allowlist/approval; test `T01` |
| 14 | **R14** | Cost is not persisted; no per-hive attribution | **Certain** | P1 money/audit | `06:1207-1210`; `run_usage` live row has tokens only | “What did this hive spend?” cannot be answered after restart | Per-call cost ledger; run/hive keys; NULL honest | Cost query after restart; reconciliation within ±5% |
| 15 | **R15** | Serial engine + one SQLite mutex + per-fragment events cap scale | **Certain** | P1 scale claim | `execution.rs` sequential; `db.rs:295-350`; `bus.rs:18-20`; `03:88-104` | One stuck call stalls the only run; resync storms at 150 | Scheduler, writer actor/read pool, coalescing, bounded queues | Stage S1–S4 tests in `03:639-664`; no claim before the artifact |
| 16 | **R16** | Remote/Watchdog requirement has no runtime model | **Certain** | P1 operator requirement | `09:148` “not coded”; `02:526` X13 rejects long-lived | No watchdog card/control; a daemon cannot survive restart | Service/lease model; card; scheduler; budget; kill | Restart-survival + interval report + card + kill test |
| 17 | **R17** | Venn memory cannot be represented; cross-branch sharing is blocked | High | P1 HMARS capability | `02:444-449` vs `04:44-51`; operator `63-68` | A memory needed by two branches must be promoted to a parent/global | Decide tree vs overlay; schema/read-set tests | Two-branch shared entry without parent visibility; or a written “tree is a metaphor” decision |
| 18 | **R18** | UI truth: dead cards, replay flattening, no all-agent view, no Now | **Certain** | P1 product | `08:386-405`, `08:606-643`; live snapshot; `05:490` | Cards show Worker/`—`; replay shows 14 nodes/5 roots; no Now surface | Card contract; replay scope; all-kind census; Now view | Every kind renders a card or a named refusal; replay matches the run; Now shows live events + edges |
| 19 | **R19** | Hierarchical planning is recommended but not expressible | **Certain** | P1 scale/capability | `07:147,435`; `provider_executor.rs:25-29`; `03:505`; `09:237` | Planner emits only flat workers; SuperAgent branches never nest from a plan | Add `parent`/sub-plans; validate depth/role; test | Two-level planned tree; 300-node synthetic plan has an explicit budget/refusal |
| 20 | **R20** | Dirty tree / no build fingerprint makes research and QA drift | **Observed** | P1 process | `provider_executor.rs` (21:16), `execution.rs`/`tool_loop.rs` (21:31–21:34), launcher (21:31) modified; `08` hash differs; `07` said only non-runtime paths dirty | Two lanes quote different line numbers/behaviour for the same file | Pin HEAD + working-tree hashes; build fingerprint in health; one source of truth | Every report header carries hashes; a stale hash fails the gate (§9) |

### 7.1 Risk ranking rationale

R1–R3 are **observed current defects**, not hypotheticals. They rank above the security items only because they are already corrupting the product's core claims (what ran, where, and why it failed). R11–R13 are **known P0 security holes** that must be closed before any concurrency or web-content work; they are lower in this list only because the first wave already documents them and the fix path is clear. R4, R5, and R20 are **process/truth risks**: they cause the next wave to be confidently wrong, which is worse than being visibly blocked.

### 7.2 Risks that are under-ranked by the first wave

- **Tool-turn cap** (`07` lists it, `03`/`09` do not): it is the actual failure mode of a normal research run.
- **Snapshot scope** (no report): it invalidates every history/replay claim.
- **Workspace root** (`06` D6 mentions the default; no report treats it as a product defect): it invalidates every file-based result.
- **False budget error** (`06` T25 covers provider 402/400/422; not the tool cap): it makes the budget UI untrustworthy.
- **Uncommitted fix** (no report): it makes the persona findings stale without a code change.
- **Remote/Watchdog** (`09` target only): it is a named operator requirement, not a nice-to-have.
- **Venn memory** (no report): both memory designs silently substitute a tree for the operator's requirement.

---
## 8. Second-wave master question list — exactly 20

Each question is new relative to the first wave's 20-Q&A sets. Options are checkboxes so the operator can mark one or more. “Why it matters” names the finding or contradiction that makes the answer load-bearing.

### Q1 — Autonomy modes 1–4: what does each mode permit?

- [ ] **Mode 1 = read-only; Mode 2 = read + spawn; Mode 3 = side effects with approval; Mode 4 = autonomous except spend.**
- [ ] **Mode 1 = approval for every side effect; Mode 2 = approval for write/shell only; Mode 3 = approval for spend only; Mode 4 = no approvals.**
- [ ] **Modes are presets over a capability matrix** (tools × spawn depth × approval class × spend), not fixed bundles.
- [ ] **Modes are per-project overrides** of a global default.
- [ ] **Other — describe the matrix.**

**Why it matters:** approval policy, spawn authority, budget, A2A visibility, and UI all need a mode; no first-wave report defines one (`§5.1 #5`; X7). Without this, “default mode 1” is a label with no behaviour.

### Q2 — Under “no hard default budget”, what is the default spend posture?

- [ ] **Soft guard only; no hard stop.**
- [ ] **Soft guard + a global emergency stop always on; per-run hard caps opt-in.**
- [ ] **Soft guard + a hard per-run default; global stop only above N agents.**
- [ ] **Hard token/cost defaults; budget switch off by default.**
- [ ] **Other.**

**Why it matters:** the operator wants no hard default (`OPERATOR-DECISIONS.md:24-26`), but denial-of-wallet needs a global stop (`06:1218-1221`); the current `$0.50` cap is both hard and not the operator's default (`§4 X7`). The live false `E_BUDGET` shows the current answer is incoherent.

### Q3 — What is a Remote/Watchdog agent in the runtime?

- [ ] **A scheduled recurring task** (interval/cron) owned by the operator.
- [ ] **A long-lived daemon with a lease/heartbeat** and a supervisor.
- [ ] **An event-triggered watcher** (file/queue/health change).
- [ ] **A hybrid:** daemon wakes on interval or event, otherwise resting.

**Why it matters:** “one job forever, report every 5 minutes” (`OPERATOR-DECISIONS.md:32-33`) is a service, not a task; `02`'s X13 rejects long-lived agent identity (`§4 X5`). Persistence, card, budget, approval, and kill all depend on this shape.

### Q4 — What may a Watchdog/Remote agent do, and how does it report?

- [ ] **Read-only + notify its owner** (safest; no spawning).
- [ ] **Read + spawn within its caps.**
- [ ] **Read + write/shell with approval.**
- [ ] **Full agent authority under its autonomy mode.**
- [ ] **Other.**

**Why it matters:** the operator calls them “deliberately simple” but also wants a card and reports; the authority level determines the card fields, approval policy, budget, and whether a watchdog can wake other agents.

### Q5 — How do the five memory scopes overlap?

- [ ] **Strict tree + one-level promotion** (no overlap).
- [ ] **Tree + shared overlay banks** (an entry can be shared without reparenting).
- [ ] **Venn multi-membership** (an entry can belong to two branches without belonging to their parent).
- [ ] **Flat store + read filters.**
- [ ] **Other.**

**Why it matters:** the operator describes a “connected Venn-diagram cluster” (`OPERATOR-DECISIONS.md:63-66`); `02` and `04` both propose trees and disagree on the scope names (`§4 X4`). The schema, read policy, and UI cannot be designed until this is answered.

### Q6 — What may the HyperAgent read from memory?

- [ ] **Summaries and counts only** (no raw entries).
- [ ] **Summaries + audited raw on request** (operator-visible audit).
- [ ] **Raw access to all banks, as an explicit setting.**
- [ ] **Per-bank ACL** (some banks raw, some summary-only).

**Why it matters:** the operator allows raw access only if leakage/noise is prevented (`OPERATOR-DECISIONS.md:67-68`); `04` gives summaries only, never raw (`04:1464-1472`). This changes the prompt budget, audit schema, and poisoning blast radius.

### Q7 — What is the default A2A topology, and what may settings enable?

- [ ] **Tree/sibling only** (no bus, no mesh).
- [ ] **Tree + mesh opt-in** by settings.
- [ ] **Tree + a bounded notice bus with optional topics** (no reply semantics).
- [ ] **Full mesh by default.**

**Why it matters:** the operator allows mesh and optional topics (`OPERATOR-DECISIONS.md:52-59`); `02` rejects an agent bus entirely (`02:514` X1). The default and the allowed set determine the message schema and the settings matrix.

### Q8 — Who may message across hives?

- [ ] **HyperAgent only.**
- [ ] **HyperAgent + an approved SuperAgent bridge.**
- [ ] **Any agent, with approval.**
- [ ] **Mesh; no bridge.**
- [ ] **Other.**

**Why it matters:** the operator wants cross-hive messaging allowed by settings, default most auditable (`OPERATOR-DECISIONS.md:55-56`); today only same-run cross-parent sibling approval exists (`07:297-299`), and the target UUID is undiscoverable. The bridge is also the audit point.

### Q9 — Where do custom agents/templates come from?

- [ ] **Code-defined only** (no runtime authoring).
- [ ] **Import/export JSON** (validated, no editor).
- [ ] **A full settings editor** (create/edit/validate/audit).
- [ ] **Chat-authored agents/templates with validation.**
- [ ] **A marketplace later** (not now).

**Why it matters:** the operator wants built-in templates in a settings menu and custom agents via settings/chat, with no uncontrolled marketplace (`OPERATOR-DECISIONS.md:34-36`); `02` rejects a full profile editor (`02:524` X11) and `07` says `allowed_tool_ids` is unvalidated. This decides the registry's authority and audit.

### Q10 — Is Subagent a tier or a role?

- [ ] **Fifth tier with its own card, allowlist, and memory scope.**
- [ ] **Worker tier with a role badge** (current semantics plus identity).
- [ ] **Separate card, same Worker tier** (visual only).
- [ ] **Role only, no card** (current).

**Why it matters:** the operator's hierarchy names four levels (`OPERATOR-DECISIONS.md:39-40`); code has three tiers and a role (`07:149-162`; `08:113-125`); `02` says no fourth tier (`02:323`). This decides the card census, memory scope, and whether the hierarchy is four levels or three plus a label.

### Q11 — Where do settings live and how are they changed?

- [ ] **A WS command** (consistent with the contract).
- [ ] **An HTTP `PUT /api/settings`** (read/write, audited).
- [ ] **A config file + restart.**
- [ ] **Boot-only environment variables** (current).
- [ ] **Hybrid:** read via WS/HTTP, write via HTTP, env as defaults.

**Why it matters:** there is no route or command to change model/thinking/autonomy/budget/memory/A2A (`08:297-310`). Every operator answer in the settings group depends on a real write path.

### Q12 — How does a run choose its workspace root?

- [ ] **Server cwd** (current).
- [ ] **Per-project root chosen at project creation.**
- [ ] **A picker per run** (with a visible resolved path).
- [ ] **A fixed app directory** (never the operator's repo).

**Why it matters:** the live “repository” run analysed `dist\HiveMind`, not the repo (`§1 #3`; `06` D6; `08:169`). Without a root contract, every file/search result is suspect and the operator cannot trust the run.

### Q13 — What replaces the 4-turn tool-loop cap?

- [ ] **Raise it to 12–32 turns** with a per-run cap.
- [ ] **A per-task token/time budget** (turn count is a symptom, not the budget).
- [ ] **Adaptive by task type** (research vs transform vs shell).
- [ ] **Keep 4 turns, but split the error code** and show it in the UI.

**Why it matters:** three live tool-using runs failed at exactly 4 turns with only 3.3K tokens (`§1 #2`; `tool_loop.rs:14-21`). The operator wants more autonomy, and the current cap makes normal research prompts fail.

### Q14 — How should tool/budget/call limits be named?

- [ ] **Split `E_TOOL_TURN_LIMIT`, `E_TOOL_CALL_LIMIT`, `E_BUDGET_*`** and keep the message truthful.
- [ ] **Keep `E_BUDGET_EXCEEDED` but change its message** to cover all limits.
- [ ] **Use `E_CALL_LIMIT` for all call/turn caps** and `E_BUDGET_*` for money/tokens.
- [ ] **A single per-task budget code** with a structured detail field.

**Why it matters:** `E_BUDGET_EXCEEDED` currently says “$0.50 cost cap” for a tool-turn cap (`refusal.rs:152-154`; `execution.rs:2845-2849`; `tool_loop.rs:254-269`). The operator cannot diagnose spend vs tool limits, and the UI budget meters will lie.

### Q15 — What belongs in a run snapshot?

- [ ] **Only that run's graph** (filter by run).
- [ ] **The global graph + a `run_id` on every node** (filter in the client).
- [ ] **A per-run graph store** (strongest isolation).
- [ ] **Global graph, admin-only** (run views filter).

**Why it matters:** snapshots currently contain the global graph (`snapshot.rs:258-271`) and grow 2→35 nodes across runs; the client decodes every node (`store.ts:571-622`). History/replay for one run can show other runs' tasks.

### Q16 — What cache program comes first?

- [ ] **Stable per-role prefix + per-agent hit/miss metrics** (measure before optimising).
- [ ] **A shared cross-agent prefix** (reuse where correct).
- [ ] **Per-run prefix only** (no cross-agent reuse).
- [ ] **Measure only; no optimisation until the hit rate is known.**

**Why it matters:** the operator makes caching first-class (`OPERATOR-DECISIONS.md:97-98`); `01` gives the full-match prefix-unit rule but no measured rate (`01:285-299`). One live run showed 1,920/2,100 hit — promising but single-sample. The contract must define the prefix layers before any “better than DeepSeek” claim.

### Q17 — What are the default card facts?

- [ ] **The operator's six:** role, task, state, cost, tools, branch.
- [ ] **Four facts + inspector:** identity, tier, state, one metric.
- [ ] **Role and state only** (minimum honest).
- [ ] **Per-tier configurable** (Hyper/Super/Worker/Tool/Remote).

**Why it matters:** the operator explicitly wants six facts (`OPERATOR-DECISIONS.md:83-84`); `05` limits cards to four (`05:108-118`, `05:579-583`). The card contract also determines which wire fields must be produced.

### Q18 — What is the default main view, and what is “Now”?

- [ ] **Canvas + chat + a live “Now” strip + memory/A2A edges, all on by default.**
- [ ] **One primary region + one inspector** (current `05` direction).
- [ ] **User presets** (save/load layouts; defaults are a preset).
- [ ] **A Now-only live view** (canvas/tree/inspectors are separate destinations).

**Why it matters:** the operator wants hierarchy + live execution + memory/A2A edges and all projections enabled by default (`OPERATOR-DECISIONS.md:81-82`); `05` wants fewer simultaneous surfaces (`05:286-299`). “Now” is not defined anywhere, so the second wave needs the operator's definition before building it.

### Q19 — Which DeepSeek surface comes next, and in what order?

- [ ] **Search/Fetch first**, with a per-request cost dimension and approval/SSRF policy.
- [ ] **Files/Vision first** (image workflow).
- [ ] **Responses API first** (Codex/interop).
- [ ] **None until scale and security gates pass.**

**Why it matters:** the operator says research/recommend for Files, Search/Fetch, Responses, and vision “later” (`OPERATOR-DECISIONS.md:90-96`); `01` marks Files/Vision/Responses NOT NEEDED and Search/Fetch NOT WIRED (`01:590-599`). The next surface determines the cost model and the protocol work.

### Q20 — What is the next scale claim, and what must pass before it?

- [ ] **S0 baseline only** (reproduce the 8-permit ceiling).
- [ ] **S1: 25 agents** (admission queue).
- [ ] **S2: 50 agents** (storage/coalescing).
- [ ] **S4: 150 active** (budget/AIMD/UI).
- [ ] **300–600 tasks after hierarchical planning.**

**Why it matters:** the operator says never claim a higher number than the stage proves (`OPERATOR-DECISIONS.md:13-14`); `03` defines S0–S5 but no stage test has been run (`03:639-664`), and the live app cannot run even one normal research task to completion. The answer sets the only honest claim for the next release.

---

## 9. Evidence gate for accepting second-wave reports

### G0. Purpose

Wave 1 produced strong evidence and four systematic failure classes: **stale trees** (HEAD vs working tree), **assumed scope** (snapshot as run graph), **untimestamped runtime** (“current data is empty”), and **inherited numbers** (393,216 / $0.50 / 40 calls). The gate exists to make those failures visible before a report becomes an input to a decision. It is not a style guide; a report that fails a mandatory check is returned, not merged.

### G1. Mandatory report header

Every second-wave report starts with a machine-readable block. A missing field is a rejection, not a warning.

```yaml
gate: EVIDENCE-GATE-v1
lane: <number and name>
lane_id: <stable id>
date_local: <ISO 8601 with offset>
date_utc: <ISO 8601 Z>
workspace: C:\Users\Riley\Desktop\hiverouter-v1
head: <40-hex sha>
working_tree: clean | dirty
working_tree_hashes:            # 16+ hex each; every file cited by the report
  backend/crates/hiverouter-core/src/provider_executor.rs: <hash16>
  ...
runtime:
  url: http://127.0.0.1:8099
  reachable_at: <ISO 8601 Z>
  last_seq: <int>
  graph_nodes: <int>
  schema_version: <int>
  active_run_id: <id or null>
  active_run_status: <status or null>
paid_calls: none | <count + approval reference>
credentials_read: none
product_files_edited: none
files_written: [<exact lane file>]
```

**Rules:** the hashes must be computed at write time, not copied from wave 1. If a cited file is not listed, its line anchors are not accepted. If the runtime is unreachable, say so; do not omit the block.

### G2. Claim ledger — mandatory shape

Every material claim gets one row:

| Field | Requirement |
|---|---|
| `id` | Stable (`C01`, `R01`, …). |
| `claim` | One falsifiable sentence. No “and” bundles. |
| `class` | `VERIFIED` / `READ` / `INFERRED` / `CONTRADICTED` / `UNVERIFIED`. |
| `evidence` | `path:line`, `URL (fetched YYYY-MM-DD)`, or `command + quoted output`. |
| `verified_at` | ISO 8601. |
| `scope` | What the claim does **not** cover. |
| `depends_on` | Optional claim ids. |

**Promotion rules:**

- `VERIFIED` requires the lane to have read the exact current line or captured the runtime output **in this pass**. A citation to a first-wave report is `READ`, never `VERIFIED`.
- A number sourced from a report is `READ`; a number computed here from named assumptions is `INFERRED`; a number from a command is `VERIFIED`.
- A claim about a dirty working tree must use the working-tree hash, not the HEAD line number.
- A claim that another report contradicts must be `CONTRADICTED` until resolved, even if the lane believes one side.
- `UNVERIFIED` must say what would settle it. “Not tested” is not a class; it is an omission.

### G3. Evidence requirements by claim type

| Claim type | Minimum evidence |
|---|---|
| Provider fact (limit, price, status code) | Official URL + fetch date + exact quoted value; no memory, no aggregator. |
| Code behaviour | Current `path:line` + the hash of the file; if the code path is a chain, every hop's `path:line`. |
| “Wired” / “callable” | All 10 trace hops (UI → handler → client → request → route → service → store → provider → response → render-back → re-entry), or explicit `PARTIAL` + the missing hop. A route alone is not wired; a handler alone is not wired. |
| “Absent” / “no X” | Search pattern + paths + result; a zero-result grep is evidence only for that scope. If the scope is the whole repo, say so and include the command. |
| UI claim | Source `path:line` **and** a timestamped DOM/screenshot capture with run id, or `UNVERIFIED — no capture`. |
| Runtime behaviour | Command + timestamp + output; for a run, run id/status/last_seq; for an SSE/WS claim, the sequence range. |
| Budget/token/cost claim | Source URL/date, the exact constant `path:line`, arithmetic with named assumptions, and the distinction between max/default/reserve. |
| Security claim | Both the seam (`path:line`) and the precondition; an exploit path is `INFERRED` unless executed; a test name is a proposal unless run. |
| Contradiction | Both sides cited; the disposition; the settling test. |

### G4. Runtime protocol

1. **Before probing:** `GET /health`; record `last_seq`, `graph_nodes`, `schema_version`, uptime. If a run is active, record its id/status; do **not** call the state quiescent.
2. **Every probe:** method, URL, status, body excerpt, UTC + local timestamp.
3. **After probing:** `GET /health` again; note changes. If the server dies, record the last good timestamp and the failure; do not extrapolate.
4. **No paid call** unless the task explicitly approves it; record the approval, run id, usage, and cost. Default `paid_calls: none`.
5. **No credentials:** never read, print, or store key material; the health boolean is enough.
6. **No destructive action:** no cancel/kill/approve/delete/restart unless the task explicitly asks and the action is safe. A read-only lane that needs a mutation returns `blocked`.
7. **UI:** a screenshot is evidence only with a timestamp, URL, run id, and the action that produced it. A mid-run capture does not prove terminal truth.
8. **Quiescence:** if you claim “current”, show the time; if the runtime changed during the pass, say so.

### G5. Contradiction protocol

- Maintain a contradiction table with `X-id`, side A citation, side B citation, disposition (`resolved` / `contradicted` / `ambiguous`), and `settles_it`.
- A report that cites a claim contradicted by a peer must cite the contradiction; silence is not a resolution.
- An unresolved contradiction blocks any decision that depends on it. It becomes a question in the master list, not a footnote.
- Do not resolve by deleting one side, by choosing the more recent file without a hash, or by prose seniority.

### G6. Freshness and drift

- Re-read every cited source at write time; record its hash.
- If a cited file's hash differs from the first-wave hash, mark the inherited claim **STALE** and re-derive it.
- If HEAD differs from `bd575ba`, do not reuse line numbers; re-anchor or label `line-drift`.
- If the tree changes **after** your read, record the new hash and time, and label the affected claims `time-scoped`. Do not silently incorporate the change and do not silently ignore it. This report's §2.4 is the worked example.
- Provider facts: re-fetch or mark `READ` with the original fetch date. A price older than 24 hours without a date is rejected.
- A report older than 24 hours must re-run its runtime probes before it is used as a decision input.

### G7. Coverage and negative claims

- List the sections read and the sections not read. “All of 04” requires a line count; a skim is not a read.
- For a negative claim, include the exact pattern and path; a zero result is not proof outside the searched scope.
- For a UI census, count controls and name the render file:line for each; “the UI does nothing” requires an inventory.
- For a model/provider claim, distinguish “not in the DTO” from “not in the provider docs” from “not in the running app”.

### G8. Safety and scope

- No product edits; no server restart; no install; no credentials; no paid call without approval; no destructive action.
- A lane that needs a wider scope returns a structured blocker (§2.2 of the operating contract), not an approximation.
- No secret, token, key, or PII in the report; redact examples.
- No test weakening; a failing test is evidence, not an obstacle.

### G9. Acceptance checklist (all mandatory)

| # | Check | Pass condition |
|---:|---|---|
| 1 | Header | Complete; HEAD + working-tree hashes; runtime snapshot; `paid_calls: none` or approved. |
| 2 | Claim ledger | Every material claim has a class, evidence, time, and scope. |
| 3 | No unsupported `VERIFIED` | Reviewer samples 10; zero unsupported. |
| 4 | No uncited number | Every number has a URL/date, `path:line`, or command. |
| 5 | No stale tree | Every cited file re-read at the pinned hash; stale claims marked. |
| 6 | Wired claims | 10 hops or named missing hop; partial is labelled. |
| 7 | Negative claims | Search pattern + scope present. |
| 8 | Contradictions | All cross-report conflicts in the table; unresolved = question. |
| 9 | Risks and questions | Every P0 risk has a test; every question has 3–5 options. |
| 10 | Safety | No paid call, credential, destructive action, or product edit. |
| 11 | Report hygiene | Stub sweep clean (or every hit classified); no TODO/FIXME/placeholder. |
| 12 | File discipline | Only the assigned file written; path exact. |

### G10. Gate record

```json
{
  "gate": "EVIDENCE-GATE-v1",
  "lane": "<id>",
  "decision": "ACCEPT | ACCEPT-WITH-UNVERIFIED | REJECT",
  "unsupported_verified": 0,
  "uncited_numbers": 0,
  "stale_claims": 0,
  "contradictions_open": ["X2", "X4"],
  "blocked": ["paid call for U3"],
  "reviewer": "<id>",
  "at": "<ISO 8601 Z>"
}
```

**Rejection codes:** `E_NO_HEADER`, `E_UNPINNED`, `E_STALE_TREE`, `E_NO_LEDGER`, `E_UNSUPPORTED_VERIFIED`, `E_UNCITED_NUMBER`, `E_NO_REPRO`, `E_CONTRADICTION`, `E_OVERCLAIM`, `E_SECRET`, `E_PAID`, `E_SCOPE_CREEP`, `E_STUB`.

`ACCEPT-WITH-UNVERIFIED` is allowed only when no P0 decision depends on the unverified item and the item carries a settlement test. It is not a softer `ACCEPT`.

### G11. What this gate does not accept

- Prose-only claims; “looks correct”; “should work”; “the code is straightforward”.
- A screenshot without a timestamp/run id; a mid-run screenshot as terminal proof.
- A number inherited from `AGENTS.md` or a first-wave report without a URL/date.
- A design proposal presented as a finding; a target presented as current.
- “Current” without a time; “empty” without a probe; “all” without a count.
- 393,216 used as a default `max_tokens`, a budget constant, or a per-call reserve.
- A contradiction resolved by deleting one side or by seniority.
- A paid call, a credential read, a destructive action, or a product edit in a read-only lane.

### G12. Gate ownership and timing

- The lane applies the gate at write time and includes its own gate record.
- An independent reviewer samples 10 claims and checks the header/hashes; any failed sample returns the report to the lane.
- The synthesizer may not accept a report that fails the gate, even if its conclusions look right. A blocked, truthful report is a successful outcome; a plausible-looking fake is not.

---
## Appendix A — Runtime evidence, quoted

All timestamps America/Chicago (CST, UTC-06:00) unless marked Z. The runtime was read-only; no run was submitted by this lane.

### A.1 Health before and during activity

```text
First sample (21:20–21:21 CST):
{"status":"ok","version":"1.2.0-dev","schema":"hiverouter.v1",
 "checks":[{"name":"rustc","required":true,"ok":true,"detail":"rustc 1.99.1 ..."},
           {"name":"node","required":false,"ok":true,"detail":"v26.7.0 ..."},
           {"name":"deepseek_api_key","required":false,"ok":true,
            "detail":"DEEPSEEK_API_KEY is present (value never logged)"}],
 "database":{"path":"C:\\Users\\Riley\\AppData\\Local\\HiveMind\\hiverouter.db",
             "schema_version":10,"error_code":null},
 "transport":{"last_seq":0,"graph_nodes":8,"allowed_origins":2},
 "prompt_assets":{"loaded":true,"error_code":null}}
```

```text
At 2026-11-07T21:23:36-06:00:
{"status":"ok","version":"1.2.0-dev","schema":"hiverouter.v1",
 ...
 "database":{"path":"C:\\Users\\Riley\\AppData\\Local\\HiveMind\\hiverouter.db",
             "schema_version":10,"error_code":null},
 "transport":{"last_seq":1366,"graph_nodes":14,"allowed_origins":2},
 ...}
```

`last_seq` 0 → 1,366 and `graph_nodes` 8 → 14 in the same server process is evidence that another writer was active. A report that calls this server “quiescent” would be wrong.

### A.2 Runs observed

```text
GET /api/history/runs at 21:23:36:
{"runs":[{"run_id":"01a1b92e-7350-7119-8c70-f4d0950cec1d",
          "project_id":"01a1b85f-71f4-7e9f-9a79-22729491030f",
          "status":"failed","error_code":"E_BUDGET_EXCEEDED",
          "created_at":"2026-11-08T03:23:11Z"}]}
```

Read-only SQLite after the run sequence (21:26 CST):

| Run | Status | Error | Tool turns | Tool calls | Created (Z) | Prompt (excerpt) |
|---|---|---|---:|---:|---|---|
| `01a1b92e-7350-...` | failed | `E_BUDGET_EXCEEDED` | 4 | 8 | 03:23:11 | “Map the architecture of this repository and identify the highest-risk module.” |
| `01a1b92f-f31f-...` | failed | `E_BUDGET_EXCEEDED` | 4 | 5 | 03:24:49 | “Explain the entire HiveMind system/framework/architecture …” |
| `01a1b930-9ca0-...` | failed | `E_BUDGET_EXCEEDED` | 4 | 5 | 03:25:33 | “Spawn subhives if youd like … Find out why file list read or …” |
| `01a1b931-2a4f-...` | completed | — | 0 | 0 | 03:26:09 | “Show me the logs/error logs of HiveMind …” |
| `01a1b931-941a-...` | completed | — | 0 | 0 | 03:26:36 | “You just asked me to approve 2 shell commands …” |

Counts: **5 runs, 27 tasks, 22 tool-turn rows, 34 snapshots, 1 run_usage row at the time of the first read (3 by the later read), 8 task_results, 0 approvals, 0 agent_messages, 0 agent_memory.** The three tool-using runs all stopped at 4 turns; the two that completed used no tools.

### A.3 Usage for the first failed run

```text
GET /api/usage/runs/01a1b92e-7350-7119-8c70-f4d0950cec1d:
{"run_id":"01a1b92e-7350-7119-8c70-f4d0950cec1d",
 "usage":{"prompt_tokens":2100,"completion_tokens":1168,"total_tokens":3268,
          "prompt_cache_hit_tokens":1920,"prompt_cache_miss_tokens":180,
          "reasoning_tokens":null}}
```

3,268 tokens cannot reach the 400K token budget or the $0.50 cost cap at any documented price. The error is therefore not a money cap; the producer is the tool-loop path.

### A.4 Snapshot scope, measured from files

Latest snapshot per run, node and root counts:

| Run directory | Revisions | Latest nodes | Root nodes | Status |
|---|---:|---:|---:|---|
| `01a1b86d-8b15-...` | 4 | 2 | 1 | completed |
| `01a1b88b-53c2-...` | 4 | 4 | 2 | completed |
| `01a1b88b-7190-...` | 1 | 4 | 2 | cancelled |
| `01a1b88d-0665-...` | 4 | 6 | 3 | cancelled |
| `01a1b893-90e0-...` | 4 | 8 | 4 | completed |
| `01a1b92e-7350-...` | 6 | 14 | 5 | failed |
| `01a1b92f-f31f-...` | 4 | 20 | 6 | failed |
| `01a1b930-9ca0-...` | 6 | 26 | 7 | failed |
| `01a1b931-2a4f-...` | 10 | 31 | 8 | completed |
| `01a1b931-941a-...` | 8 | 35 | 9 | completed |

Monotone growth despite disjoint runs is the signature of a global-graph snapshot. Run `01a1b92e` had six tasks in SQLite but fourteen nodes in its own snapshot; the extra eight belong to earlier runs.

**Environment caveat (important):** at the first read, `/health` reported `graph_nodes:8` while the DB had 0 runs/0 tasks, and the data directory contained `*.testrun-184830` copies. That combination is consistent with an external test harness replacing the DB under the running process, so the 8-node count is **not** proof of normal recovery. The snapshot-scope finding does not depend on it: the per-run snapshot files themselves accumulate nodes 2→35 across ten disjoint run directories, and run 2's snapshot contains run 1's tasks while its own DB task set is separate. The bug is in `capture` (global graph), not in the harness.

Code seam:

```rust
// backend/crates/hiverouter-core/src/snapshot.rs:258-271
pub fn capture(graph: &GraphAuthority, run_id: RunId, revision: u64,
               run_status: AgentState) -> Result<SnapshotEnvelope, Refusal> {
    Ok(SnapshotEnvelope {
        schema_version: SNAPSHOT_SCHEMA_V2.to_owned(),
        run_id,
        revision,
        created_at: Timestamp::now(),
        graph: graph.snapshot()?,     // <-- the whole process graph
        run_status: Some(run_status),
    })
}
```

Client seam:

```text
web/src/state/store.ts:571-622
recordsFromSnapshotGraph(graph)      // every node becomes a TaskRecord
kind: 'worker'                       // no kind/label in TaskNode
parentId: ids.has(parentId) ? parentId : null
```

`TaskNode` has no `run_id` (`graph.rs:16-27`), so the client cannot filter even if it wanted to.

### A.5 Workspace evidence

Run 1's persisted `task_result` (superagent `01a1b92e-7d52-...`) begins:

> “The repo is a **two-binary portable desktop distribution** — no source code at all. Only: `HiveMind.exe` 8,704 B; `hiverouter.exe` 15,105,536 B; `README.txt` 1,038 B.”

The actual `dist\HiveMind` directory:

```text
HiveMind.exe       8704
hiverouter.exe     15105536
README.txt         1038
```

This is exact. The live server was running with its workspace root at `dist\HiveMind`, not the repo. The prompt asked for “this repository”. There is no workspace picker (`08:169`; `08:379`), so the operator had no way to correct it from the UI.

### A.6 Server liveness

```text
2026-11-07T21:27:50-06:00:
GET http://127.0.0.1:8099/health -> connection refused
Get-Process HiveMind,hiverouter -> none
```

The runtime was available from at least 21:20 to 21:26 and unavailable at 21:27:50. Every runtime claim above is scoped to that window. A second-wave lane must re-probe; if the server is down, the correct status is **blocked**, not “no data”.

### A.7 Working-tree hash evidence

| File | Hash (first 16, final read 21:40) | First-wave hash | Consequence |
|---|---|---|---|
| `backend/crates/hiverouter-core/src/provider_executor.rs` | `5656D179BDB68E74` | `A2BFFC7D12A29A0A` (`08` §11) | Uncommitted persona fix at 21:16; HEAD-era line anchors stale. |
| `backend/crates/hiverouter-core/src/execution.rs` | `8218C81ED1679503` | not recorded by `08` | Uncommitted cap/refusal fix at 21:31 (`:2845-2849` now `CallLimit`). |
| `backend/crates/hiverouter-core/src/tool_loop.rs` | `8D924F9E969B9F95` | not recorded by `08` | Uncommitted caps at 21:34: 12 turns / 40 calls, `CallLimit`. |
| `backend/crates/hiverouter-core/tests/tool_loop_engine.rs` | `F18C26DB155984D0` | not recorded | Tests updated for the new caps; not run by this lane. |
| `tools/desktop-launcher/HiveMindLauncher.cs` | `16B86ECB1A7AB6E3` | not recorded | Passes `--workspace-root` (21:31); hard-coded dev path. |
| `docs/provider/AGENT-ARCHITECTURE-TRUTH.md` | `D1B8B5F860978315` | `D1B8B5F860978315` (`08` §11) | Same file `08` read; its tool-turn sentence is ahead of HEAD. |
| `docs/research/hmars/10-gap-meta.md` | this file | n/a | New. |

### A.8 Late-breaking changes — exact diff facts

Observed by `git diff`/mtime at 21:37–21:40, **not authored by this lane**:

```text
execution.rs:2845-2849
-                return Err(Refusal::BudgetExceeded);
+                return Err(Refusal::CallLimit);   // tool turn/call cap, not money

tool_loop.rs:14-16
-pub const MAX_TOOL_TURNS: u32 = 4;
-pub const MAX_CALLS_PER_RUN: u32 = 16;
+pub const MAX_TOOL_TURNS: u32 = 12;
+pub const MAX_CALLS_PER_RUN: u32 = 40;

tool_loop.rs:254-269
-    return Err(Refusal::BudgetExceeded);
+    return Err(Refusal::CallLimit);
```

```text
HiveMindLauncher.cs:57-62
+ string workspaceRoot = ResolveWorkspaceRoot();
+ Arguments = "--bind ... --data-dir \"...\" --workspace-root \"" + workspaceRoot + "\"";
+ WorkingDirectory = workspaceRoot;

HiveMindLauncher.cs:163-176
+ configured = HIVEMIND_WORKSPACE_ROOT; if exists -> use
+ else if DefaultRoot (C:\Users\Riley\Desktop\hiverouter-v1) exists -> use
+ else Directory.GetCurrentDirectory()
```

The server already accepts `--workspace-root` (`config.rs:276-296`), so the launcher change is syntactically plausible. It is unbuilt and untested here. The hard-coded `DefaultRoot` is a dev-machine path: on a portable copy the code still falls back to `Directory.GetCurrentDirectory()` — the package directory — which is the original failure mode. The fix addresses this machine, not the portable-app requirement.

---

## Appendix B — First-wave source map and what each report can prove

| File | Authoritative for | Claims register / limitations | Cannot prove |
|---|---|---|---|
| `01-deepseek-api.md` | Provider surface, prices, cache rule, status codes, gaps | §10 status ledger, §11 20 Q&A, §14 top-10, §15 unknowns; no live call; docs unpinned | Account state, cache hit rate, live model list, key behaviour. |
| `02-hmars-patterns.md` | Hierarchy/registry/A2A patterns, invariants, reject list, failure taxonomy | §9 invariants, §13.2 claims, §13.1 fetch limits; no runtime | Whether HiveMind's runtime actually holds the invariants; no load test. |
| `03-scale-concurrency.md` | Local caps, provider envelope, SQLite/Tokio arithmetic, stages | §3 READ table, §12 20 Q, §13 stages, §16 status; one local SQLite bench; no live load | Actual 150-agent behaviour, 429 headers, tokens/s, bundled SQLite version. |
| `04-memory-architecture.md` | Current memory, bank-tree proposal, schema, read/write policy, retention | §17 claims register, §16 unverified; design only | Whether the proposal works, migration safety, UI, vector path. |
| `05-ui-ux-patterns.md` | Projection/card/approval/memory recommendations, a11y/motion sources | §5 20 Q, §7 sources, §8 open decisions; not measured | Current pixel parity, live render, operator acceptance. |
| `06-security-cost.md` | Threat register, cost arithmetic, provider errors, key/HTTP/WS analysis | §4 threats, §11 20 Q, §12 unknowns; no exploit, no paid call | Exploitability, live account balance, actual retry billing, same-user scope. |
| `07-code-backend.md` | Code-level tiers, personas, A2A, memory, caps, gaps | §12 35-claim register, §13 non-claims; no build/test/run | Runtime behaviour, line drift after the working-tree edit, live performance. |
| `08-code-ui.md` | UI truth, dead/missing controls, runtime snapshot at its read time | §11 evidence hashes, §12 limitations; no browser capture, no build/test | What a keyed run renders, current pixel parity, post-snapshot state. |
| `09-synthesis-and-goal.md` | Synthesis, 20 answered questions, target, priority plan | No new evidence; §8 unverified list | Anything beyond the eight lanes it summarizes; current runtime. |

**Reading rule:** a claim in `09` that is not in `01`–`08` is a synthesis opinion, not evidence. A claim in `02` about HiveMind's runtime is a design mapping, not a runtime observation. A claim in `08` about a line is HEAD-era unless the hash matches the working tree.

---

## Appendix C — Operator answer groups (45 bullets, not 36)

| Group | Lines | Count | First-wave coverage |
|---|---|---:|---|
| Product and HMARS | 9–15 | 4 | Covered as research; priority conflict (D/Q). |
| Autonomy, approvals, settings | 19–27 | 5 | Mostly missing; budget conflict. |
| Agents and templates | 31–44 | 7 | Partial; Remote/Watchdog missing; ownership missing. |
| Tools and execution / A2A | 48–59 | 7 | Partial; collision daemon, mesh/bus, cross-hive missing/conflict. |
| Memory | 63–75 | 7 | Design exists but tree vs Venn/HyperAgent access conflict. |
| A2A and UI | 79–86 | 4 | Split inspectors vs all projections; card content conflict. |
| DeepSeek/API | 90–98 | 7 | Provider research strong; Files/Vision/Responses posture conflicts. |
| Scale, failure, persistence | 102–107 | 4 | Research strong; no modes, retry/escalation, artifacts. |
| **Total** | 9–107 | **45** | **Header says 36; no stable ids.** |

The count mismatch is not cosmetic: a second-wave question that says “operator answer 24” cannot be resolved against this file. Assign stable ids (`OP-01`…`OP-45`) before the next round.

---

## Appendix D — Cross-report disagreement quick index

| Topic | Reports that disagree | Section |
|---|---|---|
| 393,216 authority | `01/03/05/06` vs `AGENTS.md`; `05` corrects | X1 |
| Worker persona reachability | `09` (only TEMPLATE_INSERT) vs `07` (planner free-form role) | X2 |
| Persona on tool turns | `AGENT-ARCHITECTURE-TRUTH` vs `07/08/09`; working tree fixes | X3 |
| Memory ladder | `02` vs `04` vs operator | X4 |
| Long-lived Remote/Watchdog | operator vs `02` X13 | X5 |
| Bus/mesh/topics | operator vs `02` X1 | X6 |
| Default budget | operator vs `03/06` vs `09` | X7 |
| Binding call ceiling | `03/09` (40 calls) vs `07` (4 tool turns) + live | X8 |
| Snapshot scope | implicit `08` assumption vs `snapshot.rs` + live | X9 |
| `E_BUDGET` meaning | refusal text vs producer | X10 |
| “Current data is empty” | `08/09` vs live 21:23 | X11 |
| All-agent cards | operator vs `08` dead cards | X12 |
| Card content | operator vs `05` | X13 |
| Main view/projections | operator vs `05` | X14 |
| SuperAgent hierarchy | `09` target vs `07` flat planner | X15 |
| Subagent tier vs role | operator vs `07/08/02` | X16 |
| Cascade risk vs hierarchy | `02` vs `03/09` | X17 |
| HMARS definition | `02` stale vs operator | X18 |

---

## Appendix E — What this lane did not do

| Not done | Reason | What would close it |
|---|---|---|
| Re-fetch DeepSeek docs | Read-only lane, no need to duplicate `01`; time-boxed | A ten-minute URL re-fetch with quote + date. |
| Read every line of `04`, `05`, `06` | Large files; used executive/claims/decision sections | A line-by-line audit lane on those three files. |
| Exercise the planner persona bypass live | Server went down at 21:27:50; no paid call by this lane | A wiremock/scripted planner test, no provider spend. |
| Verify the late-breaking fixes | Another lane edited `execution.rs`/`tool_loop.rs`/tests/launcher at 21:31–21:37; no build, test, or restart by this lane | Build, run the updated tool-loop tests, restart the server with the new launcher, re-run one research prompt, and verify `E_CALL_LIMIT` + repo workspace + a run-scoped snapshot. |
| Capture the UI during a run | Server active window was too short; no browser probe | A timestamped Playwright/DOM pass on a restarted server. |
| Reproduce the security findings | No exploitation, no product changes | A scoped security test lane with the eight `06` Q20 groups. |
| Validate every first-wave claim | 148 claims; many are inherited READ | Second-wave lane-by-lane re-verification under §9. |
| Decide the product questions | Operator's call | The 20 questions in §8. |
| Fix any defect | Lane is read-only, no product edits | Implementation lanes, after decisions. |

**Honest bottom line:** the first wave is a strong evidence base and a weak decision base. Its code reads are mostly right, its provider facts are well-sourced, and its UI/security registers are useful. But it is already stale in the working tree, it assumed run scope where the code has none, it missed the actual binding tool cap, and it substitutes trees for the operator's Venn, task agents for daemons, and “no bus” for the operator's bounded topics. The late-breaking fixes (12 turns/40 calls + `CallLimit`, and the launcher's `--workspace-root`) reduce R1/R3 but are uncommitted and untested here; they do not change the decision gaps or the snapshot-scope defect. The second wave should not write another synthesis until D1–D5 and Q1–Q5 are answered with evidence.

---

## Appendix F — Stub sweep record (mandatory)

Command run against this file only:

```powershell
Select-String -Path docs\research\hmars\10-gap-meta.md -Pattern 'TODO|FIXME|XXX|HACK|placeholder|NotImplemented|not implemented|mock|stub|dummy|lorem|simplified for|for illustration'
```

Hits (line numbers at the final write):

| Line | Hit | Classification |
|---:|---|---|
| 221 | “Retention modes are proposed but **not implemented**” | **Permitted** — a factual status statement about a proposed feature, not unfinished deliverable code. |
| 319 | “wiremock/scripted planner” (the sweep matched **stub** in a proposed test description) | **Permitted** — a proposed test double, explicitly in the permitted class of §7 (provider/test fixtures). |
| 359 | “deterministic planner **stub**” | **Permitted** — same test-double description. |
| 472 | “design **not implemented**” | **Permitted** — status statement about a design proposal. |
| 912 | “**Stub** sweep clean … no TODO/FIXME/placeholder” | **Permitted** — the gate checklist naming the banned patterns, not an instance. |
| 932 | `E_STUB` rejection code | **Permitted** — a gate code name, not a code stub. |
| 1209 | “wiremock/scripted planner test” | **Permitted** — test-double description. |
| 1221+ | This appendix and its pattern line | **Permitted** — the sweep record itself; a self-reference, not a code stub. |

No `TODO`, `FIXME`, `XXX`, `HACK`, `dummy`, `lorem`, `simplified for`, or `for illustration` occurrence exists in this file outside the checklist row at line 912 that names them. **No fabricated code, no unfinished implementation, and no mock product logic is present.** This appendix is the classification record required by the operating contract; the hits are all research prose or gate vocabulary, not defects.





