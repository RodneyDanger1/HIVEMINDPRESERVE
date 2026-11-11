# Round 95 — Use-Case Truth Audit (HiveMind)

> **Post-audit correction (final tree):** the `http.rs` `/api/evidence/ledger` candidate
> observed during this audit was **discarded**; it is absent from the current tree and
> the installed build. References below to an “uncommitted” route are audit-time history.
> Counts remain exactly 100 use cases and 100 cannots.

**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **HEAD:** `1129349` · **Installed build:** `806251b` (`feat(root): add bounded post-dispatch monitor review…`), **2 commits behind HEAD**.
**Installed-vs-HEAD delta:** `git log 806251b..1129349` shows only evidence/QA files (`docs/evidence/live/*`, `tools/qa/*`); **no product code differs**. A `/api/evidence/ledger` summary candidate existed during the audit but was **discarded**; it is absent from the final tree and installed build, and is excluded from every claim here.
**Provider key state:** `DEEPSEEK_API_KEY` and `HIVEROUTER_API_KEY` are **both absent** from this process environment (checked). A model turn therefore refuses with `E_KEY_MISSING` (`README.md:115-117`). Everything model-dependent below is marked **blocked until key**.
**Evidence labels:** `READ` = code/doc read at HEAD with cited lines; `VERIFIED` = command run and output observed; `DOC` = README statement, enforcement site not re-read; `UNVERIFIED` = stated as a limit, not proven.

**Claim status for `task-29`:** **NOT CLAIMABLE in this session.** No shared-task tool is mounted in this subagent session, and a repo-wide search for `task-29` returned 0 matches (no board file, no ledger row). This is a tooling limitation, not a HiveMind finding. The same limitation is recorded for `task-24` in `01-core.md:184`. The deliverable was produced regardless.

**What "today" means here.** No key ⇒ no run, no planner call, no agent tool loop. The capabilities genuinely usable *right now* are the operator/read surfaces (HTTP API, UI panels, CLI tier-0/1 commands, DB backup) against a running server and an existing database. Use cases that need a key or an existing run say so in-line.

---

## 0. The surface being described (measured, not remembered)

- **12 tools, 5 tiers, 35 grants** (`tool.rs:115-140`, `:462-518`). Effective agent surface is **tier ∩ persona** allowlists (`README.md:40-42`; `personas.json`).
- **Caps:** depth 3, fan-out 8, 64 agents/run, 64 provider calls/run (`caps.rs:7-18`); 40 plan nodes (`planner.rs:9-10`); 40 tool calls/run, 12 tool turns, 8 approval waits/run, 64 KiB tool result, **$0.50/run**, **400 000 combined tokens** (`tool_loop.rs:14-29`); 8 in-flight provider requests, 3 attempts, 60 s retry cap, 1 provider key (`budget.rs:35-56`).
- **Approval gate:** `Mutate`/`Execute`/`Network` require approval; `Read`/`Control` do not (`tool.rs:59-61`); 120 s TTL, single-use, replay-refused, `AllowSession` non-expiring (`approval.rs:7-13,31`).
- **Containment:** workspace jail, read/write cap 1 MiB (`jail.rs:33`; `side_effects.rs:34-35`); shell allowlist is **`git.exe` + `cargo.exe` only** by default, 60 s timeout, no interpreters, no shell interpolation (`shell.rs:41-54,169-181`). This is **not an OS sandbox**.
- **UI transport:** 24 events / 16 states / 14 frozen commands + 3 decision-gated `DIFF_*` (`docs/contracts/UI-COMMANDS-v1.md` §2-§4).
- **Prompt templates:** 8 (`planner`, `content-writer`, `superagent`, `subagent`, `data-analyst`, `code-reviewer`, `chatbot`, `agent`) in `backend/crates/hiverouter-core/assets/prompts/manifest.json`.

---

## 1. 100 concrete use cases HiveMind can perform today

Labels: **SAFE** = read-only / no side effect. **APPROVAL-GATED** = any write, execution, network, or operator-mutating command. The line says whether it passes the 120 s approval table or a deterministic admission gate; do not read the label as "the model asked a human".

### A. Inspect state without a model key (1-30)

1. **SAFE** — Open Settings and confirm `key_present:false` without the key ever being rendered (`SETTINGS_STATUS` → `SETTINGS_STATUS_RESULT`; `SettingsPanel.tsx`).
2. **SAFE** — Read server liveness and version via `GET /health` (`http.rs` route `/health`; CLI `health`).
3. **SAFE** — Read the secret-free diagnostics report via `GET /diagnose` (`/diagnose`; CLI `diagnose`).
4. **SAFE** — Read effective caps (`max_depth`, `max_fanout`, `max_agents`) from `GET /api/runtime` (`http.rs:3627-3629`).
5. **SAFE** — Read session/provider identity via `GET /api/session` (`/api/session`).
6. **SAFE** — Read supervisor/process state via `GET /api/supervisor` (`/api/supervisor`).
7. **SAFE** — Read the template catalog (8 ids, names, inputs, SPDX, source) via `GET /api/templates` (`http.rs:1395-1419`; `LibraryPanel.tsx`).
8. **SAFE** — List projects via `GET /api/projects` (`/api/projects`; `ProjectsPanel.tsx`).
9. **SAFE** — List historical runs via `GET /api/history/runs` (`/api/history/runs`; `HistoryPanel.tsx`).
10. **SAFE** — Read a run's revision list via `GET /api/history/runs/{run_id}/revisions` (`HistoryPanel.tsx`).
11. **SAFE** — Read a run's persisted graph snapshot via `GET /api/history/runs/{run_id}/state` (`HistoryPanel.tsx` / `Scrubber.tsx`).
12. **SAFE** — Read run detail via `GET /api/runs/{run_id}/detail` (`RunDetailPanel.tsx`).
13. **SAFE** — Read the active run, if any, via `GET /api/runs/active` (`RunTranscriptPanel.tsx`).
14. **SAFE** — Read per-run usage/cost rows via `GET /api/usage/runs/{run_id}` (`DataPanel.tsx`).
15. **SAFE** — List memory banks with kind/scope via `GET /api/memory/banks` (`MemoryBanksPanel.tsx`).
16. **SAFE** — Read bounded memory summaries via `GET /api/memory/summary` (`/api/memory/summary`).
17. **SAFE** — Read the memory journal via `GET /api/memory/journal` (`/api/memory/journal`).
18. **SAFE** — Verify journal hash-chain integrity via `GET /api/memory/journal/verify` (`/api/memory/journal/verify`).
19. **SAFE** — Read the memory context actually injected for a task via `GET /api/memory/context/{task_id}` (`/api/memory/context/{task_id}`).
20. **SAFE** — Read applied lessons via `GET /api/evolution/lessons` (`LessonsPanel.tsx`).
21. **SAFE** — Read a run's recorded proposals, votes, authorizations and apply state via `GET /api/agents/runs/{run_id}/proposals` (read projection only; `http.rs:2368-2400`; `ProposalsPanel.tsx`) — **requires existing run**.
22. **SAFE** — List pending approval requests via `GET /api/approvals` (`ApprovalOverlay.tsx`; `http.rs` route).
23. **SAFE** — Read the agent roster for a run via `GET /api/agents/runs/{run_id}/roster` (`AgentRosterPanel.tsx` / `AgentHierarchy.tsx`).
24. **SAFE** — Read agent profiles via `GET /api/agents/profiles` and `{profile_id}` (`AgentProfilesPanel.tsx`).
25. **SAFE** — Read per-agent memory via `GET /api/agents/memory/{agent_key}` (`AgentMemoryPanel.tsx`).
26. **SAFE** — Read run messages via `GET /api/agents/runs/{run_id}/messages` (`AgentMessagesPanel.tsx`).
27. **SAFE** — Read the A2A inbox via `GET /api/a2a/messages` (`A2AInboxPanel.tsx`).
28. **SAFE** — Read pending A2A requests via `GET /api/a2a/pending` (`/api/a2a/pending`).
29. **SAFE** — List remotes via `GET /api/remotes` (`RemotePanel.tsx`).
30. **SAFE** — Read a remote's audit trail and grants via `GET /api/remotes/{remote_id}/audit` and `/grants` (`RemotePanel.tsx`).

### B. Read workspace/history through the frozen UI commands (31-38)

31. **SAFE** — List the workspace tree through `FILE_TREE_LIST` → `FILE_TREE_RESULT` (`UI-COMMANDS-v1.md` §2; `FILE_TREE_LIST` is a read command).
32. **SAFE** — Read one workspace file ≤ 1 MiB through `FILE_READ` → `FILE_CONTENT`; path must resolve inside the jail (`jail.rs:33`; `UI-COMMANDS-v1.md` §2).
33. **SAFE** — Replay a persisted run at an earlier revision through `HISTORY_REVISION_SET` → `HISTORY_SNAPSHOT` (read-only snapshot; `UI-COMMANDS-v1.md` §2).
34. **SAFE** — Inspect live `TOOL_CALL`/`TOOL_RESULT` events in `ToolActivityPanel.tsx` (outcome ∈ ok/refused/denied/timeout/cancelled).
35. **SAFE** — Inspect `APPROVAL_REQUEST`/`APPROVAL_RESOLVED` events without deciding them (`ApprovalOverlay.tsx`).
36. **SAFE** — Inspect agent states (16 values) colour+shape+text in `AgentsPanel.tsx` / `AgentHierarchy.tsx` (`UI-COMMANDS-v1.md` §4).
37. **SAFE** — Read the daemon roster via `GET /api/daemons` (`DaemonPanel.tsx`).
38. **SAFE** — Read daemon reports via `GET /api/daemons/{daemon_id}/reports` (`DaemonPanel.tsx`).

### C. CLI and database operations that work without a key (39-48)

39. **SAFE** — `version` — print CLI/binary version and schema ids (`cli.rs` CommandSpec `version`).
40. **SAFE** — `preflight` — dependency + **key-presence** check; reports the missing key by name rather than failing silently (`cli.rs` `preflight`).
41. **SAFE** — `schema` — list command schemas or print one (`cli.rs` `schema`).
42. **SAFE** — `tools.list` — list all 12 tools, permission classes, approval requirements (`cli.rs` `tools.list`).
43. **SAFE** — `tools.describe` — print one tool's JSON Schema and tier allowlists (`cli.rs` `tools.describe`).
44. **SAFE** — `capabilities.census` — derived counts from enums (tool/refusal/tier), not hand-typed (`cli.rs` `capabilities.census`).
45. **SAFE** — `provider.cost` — deterministic decimal cost for one usage sample at a UTC instant (`cli.rs` `provider.cost`).
46. **SAFE** — `health` — GET `/health` against a running local server (`cli.rs` `health`, tier 1).
47. **SAFE** — `diagnose` — GET `/diagnose`, no secrets (`cli.rs` `diagnose`, tier 1).
48. **APPROVAL-GATED** — `db.backup` — consistent SQLite backup with a hash receipt (CLI tier 2, `mutating:true`); operator-run, not model-run.

### D. Operator control plane over an existing run (49-58)

49. **APPROVAL-GATED** — `CANCEL_RUN` a live run; emits `RUN_STATUS(CANCELLED)` (`UI-COMMANDS-v1.md` §2; `ControlBar.tsx`).
50. **APPROVAL-GATED** — `KILL_WORKER` a running task; emits `NODE_STATE(KILLED)`; refusals `E_TASK_TERMINAL`/`E_KILL_DENIED` (`UI-COMMANDS-v1.md` §2).
51. **APPROVAL-GATED** — `APPROVAL_RESOLVE` approve/deny a pending model side effect with nonce+decision; replay is `E_APPROVAL_REPLAY` (`ApprovalOverlay.tsx`; `approval.rs`).
52. **APPROVAL-GATED** — `REWIRE_TASK` a worker to a new parent with `expected_rev` CAS; cycle/late-rewire refused by name (`UI-COMMANDS-v1.md` §2; `execution.rs`).
53. **APPROVAL-GATED** — `SPAWN_SUB_HIVE` under depth<3 and fan-out<8 (requires an active run; **blocked until key** for a new run).
54. **APPROVAL-GATED** — `TEMPLATE_INSERT` one Library template (`content-writer`, `data-analyst`, `code-reviewer`, `chatbot`) into an active run; creates a manifest-backed one-node DAG (`ws.rs:126-153`; `LibraryPanel.tsx`).
55. **APPROVAL-GATED** — `SUBMIT_PROMPT` (1..8000 chars) to start a run — **blocked until key** (`E_KEY_MISSING`; `UI-COMMANDS-v1.md` §2).
56. **SAFE** — Attach a `CHAT_PIN` client-side pin (ACK only; no server effect) (`UI-COMMANDS-v1.md` §2).
57. **SAFE** — `VIEWPORT_SET` / `VIEWPORT_RECENTER` zoom/recenter graph (client-only, zoom 0.1-3.0) (`UI-COMMANDS-v1.md` §2; `SwarmPanel.tsx`).
58. **APPROVAL-GATED** — Submit an operator action against a device via `POST /api/devices/{device_id}/actions` (Device/Remote lane).

### E. Agent tool surface, per tier (model-dependent; 59-76)

59. **SAFE** — `file_read` by SuperAgent/Worker/Subagent (tier allowlist; `tool.rs:467-509`) — **blocked until key**.
60. **SAFE** — `file_list` by SuperAgent/Worker/Subagent/ToolAgent (`tool.rs:467-518`) — **blocked until key**.
61. **APPROVAL-GATED** — `ask_parent` sends a durable message; registry class `Read` ⇒ **no approval row**, capped at 2 A2A attempts (`tool.rs:391-395`) — **blocked until key**.
62. **APPROVAL-GATED** — `report_result` writes the task result to the parent; class `Read` ⇒ no approval row (`tool.rs:402-407`) — **blocked until key**.
63. **APPROVAL-GATED** — `finish` sets the terminal verdict; class `Read` ⇒ no approval row (`tool.rs:408-413`) — **blocked until key**.
64. **APPROVAL-GATED** — `send_to_sibling` durable message (`Mutate` ⇒ approval; cross-branch additionally operator-approved) (`tool.rs:396-401`) — **blocked until key**.
65. **APPROVAL-GATED** — `spawn_agent` (`Execute` ⇒ approval), checked against depth/fan-out/agent caps before a row exists (`tool.rs:383-387`; `caps.rs:52-63`) — **blocked until key**.
66. **APPROVAL-GATED** — `file_write` with optional `expected_sha256` stale-write guard; ≤1 MiB; jail-contained (`tool.rs:312-323`; `side_effects.rs:34-35`) — **blocked until key**.
67. **APPROVAL-GATED** — `shell_exec git.exe status|diff|log` in a workspace-relative cwd; 60 s timeout, 1 MiB stdout cap (`shell.rs:41-45`) — **blocked until key**.
68. **APPROVAL-GATED** — `shell_exec cargo.exe test|check` (same caps; executes repo-controlled build scripts — explicit arbitrary-code caveat, `shell.rs:5-13`) — **blocked until key**.
69. **APPROVAL-GATED** — `web_fetch` one public HTTPS URL; no redirects, no credentials, no private networks, ≤1 MiB (`tool.rs:446-451`) — **blocked until key**.
70. **APPROVAL-GATED** — `request_detached` (Control class ⇒ no approval row) by SuperAgent/Worker; creates a remote/daemon only after deterministic admission checks reason ≥12 chars, name uniqueness, lower-level cap <4 (`README.md:67-88`; `tool.rs:440-445`) — **blocked until key**.
71. **APPROVAL-GATED** — `vote_evolution` (Control class ⇒ no approval row) by a **SuperAgent** only, one vote/task, rationale 8..400 chars; votes are data (`tool.rs:452-457`) — **blocked until key**.
72. **SAFE** — HyperAgent root-control `read_memory_summary` (local vocabulary, ≤4 calls/turn, 2 turns) (`execution.rs:4435-4536`) — **blocked until key**.
73. **APPROVAL-GATED** — HyperAgent root-control `spawn_superagent` once, only when the planner supplied no depth-1 branch; turn 2 may not create (`execution.rs:4454-4536`) — **blocked until key**.
74. **APPROVAL-GATED** — HyperAgent root-control `propose_improvement` → run-bank entry `applied:false`, `requires_human_review:true` (`README.md:131-134`) — **blocked until key**.
75. **APPROVAL-GATED** — HyperAgent root-control `convene_evolution_vote` / `assign_task` / `finish` (bounded creation vocabulary) — **blocked until key**.
76. **APPROVAL-GATED** — Planner produces a DAG (state change) validated to ≤40 nodes and depth ≤3 before dispatch (`planner.rs:9-10,44-56`) — **blocked until key**.

### F. Memory, profiles, and operator maintenance (77-86)

77. **APPROVAL-GATED** — Promote a memory entry's trust via `POST /api/memory/trust`; journaled with actor `operator` (`http.rs:1573-1600`) — works without key.
78. **APPROVAL-GATED** — Prune memory via `POST /api/memory/prune` (`/api/memory/prune`).
79. **APPROVAL-GATED** — Clear a memory scope via `POST /api/memory/clear` (`/api/memory/clear`).
80. **APPROVAL-GATED** — Import an agent profile from JSON via `POST /api/agents/profiles/import`; validated against templates (`http.rs:1213-1236`).
81. **SAFE** — Read resolved store paths (`data_dir`, `workspace_root`, `db_path`) via `GET /api/workspace` (`http.rs:1424-1434`) — local operator only, no secrets.
82. **SAFE** — Read provider model catalog via `GET /api/provider/models` — **blocked until key** (provider call).
83. **SAFE** — Read provider balance via `GET /api/provider/balance` — **blocked until key** (provider call).
84. **APPROVAL-GATED** — Revoke a remote via `POST /api/remotes/{remote_id}/revoke` (`RemotePanel.tsx`).
85. **APPROVAL-GATED** — Retire a daemon via `POST /api/daemons/{daemon_id}/retire` (`DaemonPanel.tsx`).
86. **APPROVAL-GATED** — Switch a daemon profile via `POST /api/daemons/{daemon_id}/switch` (`DaemonPanel.tsx`).

### G. Higher-level workflows that are genuinely supported (87-100)

87. **SAFE** — Read a persisted run's task digest + bounded child results (2 000 chars each, 24 000-char total) from the transcript (`execution.rs:2282-2341`) — **requires existing run**.
88. **SAFE** — Inspect lessons injected into prompts as bounded labelled data (8 lines × 400 chars) (`README.md:377-382`) — **requires existing run/lessons**.
89. **APPROVAL-GATED** — Apply a lesson **after** an operator-authored EVO-GATE receipt passes schema checks; appends one JSONL line and hot-reloads prompts (`http.rs:2827-2949`; `evolution_gate.rs:76-166`) — **requires 4 SuperAgent votes + operator seat**.
90. **APPROVAL-GATED** — Roll back an applied lesson, removing its JSONL line (`http.rs:2943-2965`).
91. **APPROVAL-GATED** — Author a proposal through the HyperAgent `propose_improvement` path and review it in Proposals; never applies code (**blocked until key**).
92. **APPROVAL-GATED** — Back up and restore the SQLite database with hash receipts (`db.backup`, `db.restore`, CLI tier 2).
93. **SAFE** — Verify memory journal integrity after a crash/restart (`GET /api/memory/journal/verify`) — designed for the interrupted-run recovery story (`README.md:125-129`).
94. **SAFE** — Recover task states as `interrupted` after a hard kill and inspect the recovered graph (`README.md:125-129`; `recovery.rs`) — **requires a prior run**.
95. **APPROVAL-GATED** — Record the operator authorization seat for a proposal via `POST /api/agents/runs/{run_id}/proposals/{proposal_id}/authorize` (`http.rs:642-643`) — **requires existing run + vote quorum**.
96. **SAFE** — Inspect the A2A fallback semantics: `ask_parent` writes a durable message and falls back to the operator mailbox with a 120 s deadline, capped at 2 attempts (`README.md:60-62`; `execution.rs:155`) — **requires an existing run**.
97. **SAFE** — Inspect that a blocked approval expires after 120 s with **zero side effects** (`approval.rs:8-10,412-414`; exercised by `approval_ttl_expiry_has_zero_side_effects`) — **requires a pending approval**.
98. **SAFE** — Inspect `AllowSession` behaviour: non-expiring, scoped to `(run, tool, args_sha256)` (`approval.rs:12-13,441`) — **requires a session grant**.
99. **SAFE** — Inspect the frozen wire vocabulary — 24 events, 16 agent states, 14 commands + 3 decision-gated `DIFF_*` — in `docs/contracts/UI-COMMANDS-v1.md` §2-§4; this is the contract the UI tests pin.
100. **APPROVAL-GATED** — Stop the local server and restart it against the same SQLite file; interrupted tasks recover rather than silently disappearing (`README.md:125-129`; `HiveMind.cmd`).

---

## 2. 100 things HiveMind cannot do (grouped, falsifiable)

Each item names the observation that would falsify it. "Falsified by" means: if that evidence appeared at HEAD, the item would be wrong.

### A. Model execution and autonomy (1-10)

1. **Run any model turn with no key.** Cannot produce a planner reply, task reply, or final synthesis — submit refuses `E_KEY_MISSING`. Falsified by a successful run with both key env vars absent.
2. **Operate continuously / autonomously in the background.** There is no scheduler; one run at a time, and boot marks orphaned work interrupted. Falsified by a daemon that starts runs on a timer without an operator.
3. **Learn across runs (weights, skills, tools).** Lessons are prompt text only; no weight update, no skill library. Falsified by a persisted behavioural change not represented as prompt text.
4. **Execute tasks in parallel.** Dispatch/settlement is a single-consumer loop; concurrency is provider-call-bound. Falsified by two tasks executing simultaneously in one run.
5. **Supervise mid-flight from the root.** The post-dispatch monitor is one read-only turn; it cannot re-dispatch, cancel, or repair. Falsified by a monitor turn that issues tools and changes the run.
6. **Self-modify source code.** Proposals are `applied:false`; no code write path from agents to the repo. Falsified by an agent-authored code change applied without an operator.
7. **Pursue open-ended goals / be AGI.** Capped single-run orchestrator with a fixed vocabulary. Falsified by autonomous goal generation beyond the submitted prompt.
8. **Fail over between provider keys.** `SUPPORTED_KEY_COUNT = 1`. Falsified by a second concurrent key being accepted (`budget.rs:56`).
9. **Retry indefinitely.** Max 3 attempts, retry-after capped at 60 s. Falsified by a 4th automatic attempt (`budget.rs:50-53`).
10. **Exceed 64 provider calls or 400 000 tokens per run.** Hard caps. Falsified by a recorded run above either figure (`caps.rs:18`; `tool_loop.rs:29`).

### B. Search, memory, and recall (11-20)

11. **Vector / embedding / semantic search.** No embedding call, vector DB, or ANN index exists (repo search returns none; `01-core.md:45-47`). Falsified by an embeddings call or similarity query path.
12. **Query another run's memory.** Banks are run/hive/branch scoped and retrieval is recency-ordered. Falsified by a cross-run bank read.
13. **Read a sibling's or foreign branch's memory.** Retrieval is run + own branch + parent branch + nearest hive. Falsified by a sibling bank in an injected context.
14. **Ingest documents/web pages into a knowledge base.** `web_fetch` returns one page to the model; there is no ingestion/index pipeline. Falsified by a persistent document index.
15. **Automatically promote memory trust.** Trust changes require the operator endpoint; agents cannot self-promote. Falsified by an agent-driven trust promotion.
16. **Deduplicate or resolve conflicting memory.** Only `status` and a unique active key; no conflict resolution. Falsified by a merge/conflict resolver.
17. **Create memory entries from the UI.** UI exposes read, clear, prune, trust — no create command. Falsified by a UI create-memory control.
18. **Rank memory by relevance.** Retrieval is `ORDER BY created_at DESC LIMIT ?`. Falsified by a relevance/similarity score in the query.
19. **Keep memory across database deletion.** Memory is the SQLite file; no external store. Falsified by recall after deleting the DB.
20. **Expose memory to an external agent over the network.** A2A is internal to the run/operator mailbox. Falsified by an inbound external A2A peer.

### C. Code, diffs, and self-evolution (21-30)

21. **Autonomously write or refactor code.** No coding agent loop; `file_write` is an approval-gated primitive, not an autonomous coding capability. Falsified by an unattended multi-file code change.
22. **Apply a diff through the UI.** `DIFF_PROPOSE`/`DIFF_APPLY`/`DIFF_REJECT` are decision-gated and "NOT FROZEN — do not implement" (`UI-COMMANDS-v1.md` §2). Falsified by a working diff-apply route.
23. **Commit/push via a HiveMind feature.** No git tool; only `shell_exec git.exe`, approval-gated, operator-approved. Falsified by a product command that commits.
24. **Let the HyperAgent edit files.** HyperAgent's registry allowlist is `spawn_agent`, `report_result`, `finish` only. Falsified by HyperAgent `file_write`.
25. **Create a new tool or skill at runtime.** Vocabulary is a frozen 12-variant enum. Falsified by a runtime-registered tool id.
26. **Rewrite its own prompts.** Prompt assets are compile-time embedded; no write path. Falsified by a self-edited `.prompty`.
27. **Run an evaluation benchmark automatically.** EVO-GATE validates a receipt; it does not run the evaluator (`evolution_gate.rs:13-15`). Falsified by an evaluator invocation inside the gate.
28. **Apply a lesson without four SuperAgent votes + operator seat.** Falsified by a lesson line with fewer authorizations.
29. **Let votes change code.** Votes are data, one per SuperAgent task (`tool.rs:454`). Falsified by a vote triggering a code mutation.
30. **Claim a Gödel machine / meta-HyperAgent.** No self-referential proof search, no procedure self-improvement. Falsified by the agent modifying its own search procedure.

### D. Tool surface and tiers (31-40)

31. **Give the HyperAgent file or shell tools.** Not on its allowlist (`tool.rs:463-466`). Falsified by `file_read` succeeding for HyperAgent.
32. **Let a Subagent write, exec, fetch, spawn, or vote.** Subagent allowlist is `ask_parent`, `report_result`, `finish`, `file_read`, `file_list` (`tool.rs:500-509`). Falsified by a Subagent `file_write`.
33. **Let a WorkerAgent vote on evolution.** `vote_evolution` is SuperAgent-only (`tool.rs:666`). Falsified by a worker vote accepted.
34. **Use the ToolAgent lane in production.** Grants exist; no shipped producer (README:28). Falsified by a real ToolAgent task record.
35. **Call MCP tools.** No MCP tool id exists in the 12-tool registry, and the tools crate has no MCP client bridge. Falsified by an `mcp__*` tool contract or a registered MCP client.
36. **Use a browser, screenshot, vision, or computer control.** None registered. Falsified by a browser/vision tool id.
37. **Run arbitrary programs through `shell_exec`.** Default policy allows only `git.exe`/`cargo.exe`; path-qualified programs refused. Falsified by `python.exe` succeeding under default policy.
38. **Use a shell interpreter.** `cmd.exe`, PowerShell, bash, etc. are refused even if allowlisted. Falsified by an accepted `cmd.exe /c` call.
39. **Pass shell metacharacters or newlines.** `/c`, `/k`, `&`, `|`, `>`, CR/LF are refused before spawn. Falsified by one accepted.
40. **Discover or add a tool after boot.** Registry is frozen at construction. Falsified by a tool listed after boot that is not one of the 12.

### E. Approvals and containment (41-50)

41. **Run a registry `Mutate`/`Execute`/`Network` tool without approval.** All three classes require approval (`tool.rs:59-61`); root-control creation is a separate deterministic gate, not a registry call. Falsified by one such registry call executing with no approval row.
42. **Auto-approve model side effects.** No auto-approval policy; approval is human/operator. Falsified by an approval resolved by the model.
43. **Use a one-shot approval after 120 s.** `allow_once` is valid only before `expires_at`; TTL expiry consumes it with zero side effects. Falsified by a one-shot execution after expiry.
44. **Replay an approval.** Unique nonce + single execution ⇒ `E_APPROVAL_REPLAY`. Falsified by a second execution from one request.
45. **Make `allow_once` last forever.** It is valid only before `expires_at`. Falsified by execution after TTL.
46. **Require approval for `ask_parent`/`finish`/`vote_evolution`.** These are `Read`/`Control`; no approval. Falsified by an approval request for them.
47. **Read/write outside the workspace through the file tools.** Canonical-path jail check on `file_read`/`file_write`; escapes are `E_PATH_OUTSIDE_JAIL`/`E_JAIL_ESCAPE`. Shell is a separate, non-sandboxed path (#50). Falsified by a file-tool read outside the root.
48. **Read a file over 1 MiB.** `MAX_READ_BYTES` 1 MiB. Falsified by a 2 MiB read succeeding (`jail.rs:33`).
49. **Write a file over 1 MiB.** `MAX_WRITE_BYTES` 1 MiB. Falsified by a 2 MiB write succeeding (`side_effects.rs:34-35`).
50. **Sandbox executed code.** Shell runs with user privileges; README and module docs say so explicitly. Falsified by an OS containment boundary around `cargo.exe`.

### F. Network and detached principals (51-60)

51. **Use remotes as real network satellites.** Remote is an `in_process` fixture principal + device + narrow grant. Falsified by an outbound socket from a remote.
52. **Run a model on a remote/device.** No remote execution path. Falsified by a remote-side model call.
53. **Rely on a daemon as a scheduled OS service.** Enrolled record + DB-check job, no live tick proof. Falsified by a continuously scheduled OS process with receipt.
54. **Receive A2A from an external agent framework.** A2A is the internal parent/sibling mailbox. Falsified by an external peer id in the inbox.
55. **Deploy to cloud/hosted infrastructure.** No deploy tool or provider. Falsified by a deploy command.
56. **Open a remote shell.** No remote shell tool. Falsified by a remote command execution path.
57. **Create more than 4 lower-level detached principals.** Admission cap <4. Falsified by a 5th admitted.
58. **Create remotes/daemons directly from a Worker/SuperAgent.** Lower tiers use `request_detached`; only root-control creates directly. Falsified by a lower-tier direct creation.
59. **Fetch over HTTP, follow redirects, send credentials, or hit private IPs/localhost.** `web_fetch` refuses all four (`tool.rs:446-451`). Falsified by any succeeding.
60. **Fetch a body over 1 MiB.** Refused. Falsified by a 2 MiB response accepted.

### G. UI, API, and packaging (61-70)

61. **Chat with the model without a key.** `SUBMIT_PROMPT` refuses; the transcript stays empty. Falsified by a reply with no key.
62. **Write/upload a file from the UI.** Only `FILE_TREE_LIST`/`FILE_READ` exist; no UI write command. Falsified by a UI file-write control.
63. **Apply a code diff from the UI.** `DIFF_*` are decision-gated/unimplemented. Falsified by a working apply button.
64. **Manage multiple user accounts/roles.** Single local operator, origin/token gate; no account model. Falsified by a user table or login flow.
65. **Install as a native mobile app.** Responsive web UI (375×812 evidence) only. Falsified by an app package.
66. **Edit prompt templates from the UI.** Library inserts templates; no template editor or write route. Falsified by a template mutation route.
67. **Enter/store a provider key in the UI.** Keys come from the environment; UI shows only `key_present`. Falsified by a key-input field that persists.
68. **Change a task's prompt mid-run.** No such command (rewire changes parent only). Falsified by a prompt-edit command.
69. **Run two runs concurrently.** One active run; `E_RUN_ACTIVE`. Falsified by two active runs.
70. **Use a ledger-summary endpoint.** No ledger-summary route exists in the current tree or installed build; the candidate was discarded. Falsified by a route answering on `806251b`.

### H. Scale, budget, and performance (71-80)

71. **Plan more than 40 nodes.** `MAX_PLAN_NODES = 40`. Falsified by a 41-node accepted plan (`planner.rs:9-10`).
72. **Spawn more than 64 agents in a run.** `MAX_AGENTS = 64`. Falsified by task 65 (`caps.rs:13`).
73. **Give one parent more than 8 children.** `MAX_FANOUT = 8`. Falsified by child 9 (`caps.rs:11`).
74. **Exceed depth 3.** `MAX_SPAWN_DEPTH = 3`. Falsified by depth 4 (`caps.rs:9`).
75. **Exceed 40 tool calls per run.** `MAX_CALLS_PER_RUN = 40`. Falsified by call 41 (`tool_loop.rs:16`).
76. **Exceed 12 tool turns per task.** `MAX_TOOL_TURNS = 12`. Falsified by turn 13 (`tool_loop.rs:14`).
77. **Exceed 8 approval waits per run.** `MAX_APPROVAL_WAITS_PER_RUN = 8`. Falsified by wait 9 (`tool_loop.rs:21`).
78. **Exceed $0.50 or 400 000 tokens per run.** `RUN_COST_CAP_USD_CENTS = 50`, `TOKEN_BUDGET_COMBINED = 400_000`. Falsified by a run above either (`tool_loop.rs:26-29`).
79. **Run more than 8 provider requests in flight.** `IN_FLIGHT_LIMIT = 8`. Falsified by 9 concurrent (`budget.rs:47`).
80. **Claim a live 40-node provider run.** The HEAD receipt (`docs/evidence/live/scale-drill-20261110-230653.json`) records **repository probes** — 41-task serial and 41-task bounded-parallel — and its own note says "This is not a live 40-node provider run." Falsified by a provider-backed 40-node run receipt bound to HEAD.

### I. Evolution, learning, AGI claims (81-90)

81. **Self-improve without human review.** Every proposal is `applied:false`, `requires_human_review:true`. Falsified by an auto-applied proposal.
82. **Run the evaluator behind EVO-GATE.** It validates a receipt file only. Falsified by evaluator execution in `evolution_gate.rs`.
83. **Bind a receipt to the real HEAD.** `head_commit` is present but never compared. Falsified by a mismatch rejection test.
84. **Bind receipt artifacts to real files.** `artifacts[].sha256` is format-checked only. Falsified by a nonexistent artifact hash being rejected.
85. **Fine-tune or update model weights.** No training path. Falsified by a weight delta.
86. **Build a reusable skill library.** Lessons are text lines. Falsified by an executable skill artifact.
87. **Promote a lesson automatically after votes.** Operator seat + receipt still required. Falsified by vote-only application.
88. **Be a held-out benchmark harness.** No benchmark runner. Falsified by a built-in eval suite.
89. **Replace its own planner or policy.** Planner template is fixed at boot. Falsified by runtime planner replacement.
90. **Claim AGI-adjacent generality.** Fixed tool vocabulary, fixed caps, single provider. Falsified by open-domain tool acquisition.

### J. Data, deployment, and environment (91-100)

91. **Use a server database (Postgres/MySQL).** Single SQLite file. Falsified by a non-SQLite backend.
92. **Replicate/HA the store.** No replication. Falsified by a second writer with consensus.
93. **Isolate multiple tenants.** Local single-operator DB; no tenant column/policy. Falsified by tenant-scoped rows and auth.
94. **Store secrets in the DB or UI.** Keys are env-only; UI shows a boolean. Falsified by a key value in a DB row or HTTP body.
95. **Rotate keys without restarting.** Key is read from the environment at process start. Falsified by a live rotation path.
96. **Sync state to the cloud.** No sync client. Falsified by an outbound sync.
97. **Run in a container sandbox.** Windows desktop app + Job Object; no container. Falsified by a container image/runtime.
98. **Survive deletion of `data_dir`.** All state is the SQLite file + JSONL lessons. Falsified by recall after deletion.
99. **Guarantee tamper-proof audit.** The ledger is append-only JSONL with hashes, but a local user can edit/delete files. Falsified by a verifier that detects out-of-band rewrite (journal verify covers memory only).
100. **Build from source on a machine with no Rust/Node.** The installed desktop package runs without them, but a source build needs MSRV 1.89 / Node ≥22.12 (`README.md:153-171`; `E_TOOLCHAIN_MSRV`, `E_NODE_MISSING`). Falsified by a clean source build with neither toolchain.

---

## 3. Best 10 things to use it for today (no key, honest preconditions)

1. **Read the frozen tool contract** with `tools.list` / `tools.describe` — the only trustworthy statement of what agents may call (12 tools, 35 grants).
2. **Get the derived census** with `capabilities.census` — counts come from enums, so they cannot drift from code.
3. **Check first-run readiness** with `preflight` — it names the missing key and toolchain instead of failing vaguely.
4. **Read `/health` and `/diagnose`** on the local server — liveness + secret-free diagnostics without a model.
5. **Audit effective caps** at `/api/runtime` and in `caps.rs` — depth 3 / fan-out 8 / 64 agents / 64 calls, plus the `tool_loop` budgets.
6. **Inspect provider state honestly** via `SETTINGS_STATUS` — `key_present:false`, never the key.
7. **Browse the template/persona catalog** (`/api/templates`) — 8 manifest-backed templates, licence and source included.
8. **Verify memory-journal integrity** (`/api/memory/journal/verify`) — the strongest existing tamper signal, and it needs no model.
9. **Review history and approvals** (`/api/history/runs`, `/api/approvals`) — see prior runs and any pending side effects before touching anything.
10. **Back up the SQLite store** with `db.backup` before experimentation — the one operation that protects everything else.

Precondition for 4-10: a running server and an existing database. On an empty install they return empty lists/booleans — still safe, just not informative.

---

## 4. Top 10 misleading things never to claim

1. **"Autonomous coding agent."** It has approval-gated `file_write`/`shell_exec` primitives, no coding loop, no diff apply.
2. **"Self-modifying / self-evolving code."** Proposals are `applied:false`; lessons are prompt text; votes are data.
3. **"Vector/semantic memory."** Retrieval is SQLite recency over fixed banks; no embeddings exist.
4. **"AGI / Gödel machine / meta-HyperAgent."** Fixed vocabulary, fixed caps, single provider, no self-rewrite.
5. **"Runs 40 agents in parallel."** Caps are 64 agents/64 calls, but dispatch is a single consumer and the 40-node scale run is not live-tested at HEAD.
6. **"Sandboxed shell."** It is approval-gated arbitrary code with the user's privileges; `git.exe`/`cargo.exe` can execute repo-controlled scripts.
7. **"Network-capable remotes/daemons."** Remotes are `in_process` fixtures; daemons lack a live scheduled tick.
8. **"Works without a key."** Only read/operator surfaces work; every model turn refuses `E_KEY_MISSING`.
9. **"Lessons apply automatically once agents vote."** Four SuperAgent votes + operator seat + a passing (operator-authored) EVO-GATE receipt are required.
10. **"Production multi-tenant SaaS."** Local single-operator desktop app: one SQLite file, one key, one active run.

---

## 5. Report shape

### Done

| Item | Evidence |
|---|---|
| Read the tool registry, permission classes, tier allowlists | `tool.rs:115-140,383-518` (`READ`) |
| Read spawn/call/plan/tool-loop caps | `caps.rs:7-70`; `planner.rs:9-10`; `tool_loop.rs:14-29`; `budget.rs:35-56` (`READ`) |
| Read approval/TTL/replay/session semantics | `approval.rs:7-16,31,412-414,441` (`READ`) |
| Read jail/shell/network limits | `jail.rs:33`; `side_effects.rs:34-35`; `shell.rs:41-54,169-181`; `tool.rs:446-451` (`READ`) |
| Read prompt/persona manifest | `assets/prompts/manifest.json`; `personas.json` (`READ`) |
| Read UI command/event/state contract and panel list | `docs/contracts/UI-COMMANDS-v1.md`; `web/src/panels/*.tsx` (`READ`) |
| Read HTTP routes and CLI command specs | `http.rs` `.route(...)`; `cli.rs` `CommandSpec` (`READ`) |
| Verified key absence | `DEEPSEEK_API_KEY`/`HIVEROUTER_API_KEY` absent in this process (`VERIFIED`) |
| Verified installed-vs-HEAD delta | `git log 806251b..1129349` = 2 evidence/QA commits, no product code (`VERIFIED`) |
| Wrote this deliverable | `docs/audit/round-95/06-use-cases.md` (this file) |

### Not done

| Item | Reason |
|---|---|
| **Claiming/completing `task-29` on the shared board** | No shared-task tool is mounted in this subagent session; repo-wide search for `task-29` = 0 matches. Recorded, not faked (same as `task-24`, `01-core.md:184`). |
| **Live end-to-end run of any model-dependent use case** | No provider key; every run path refuses `E_KEY_MISSING`. Those items are marked **blocked until key**. |
| **Verifying the running server's exact build serves every route** | Only the binary delta was checked; the server was not started. The ledger-summary candidate was discarded and is not a route claim. |

### Unverified

| Claim | What would prove it |
|---|---|
| README's 40 tool calls/run, 32 messages/run, 2 A2A attempts | Enforcement sites not re-read; only the `tool_loop`/`budget` constants were verified. Grep the message and A2A counters in `execution.rs`. |
| "Installed product code == HEAD" | Diff shows only evidence/QA files, but `806251b` was not executed. Run `version` on the installed build and compare. |
| The UI panels are all wired to the listed routes | Panel files and routes both exist; no click-through was run. Run the E2E UI suite and open each panel. |
| 40-node scale is achievable in practice | A signed receipt bound to `1129349` from a fresh run. |
| My 100-use-case list is exhaustive | Falsified by any operator-facing route/command not listed here. Enumerate `http.rs` routes + `cli.rs` specs and diff. |
