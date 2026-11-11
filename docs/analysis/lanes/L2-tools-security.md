# L2 — Tool registry, permissions, approvals, jail, shell, side effects, tool loop, personas and capability census

**Lane:** 2/7 · **Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **Task date:** 2026-11-10
**Mode:** read-only analysis. No source/config/test edit, no build, no server, no provider call, no key output.
**Only artifact written:** this file.

**Evidence tags used throughout**
- **[V]** verified by reading the cited file/lines in this session.
- **[U]** unverified — not executed here (all test pass/fail status is [U]; tests were read, not run).
- **[P]** proposal (no code changed).

**Scope covered:** every file under `backend/crates/hiverouter-tools/src/**` (20 modules + `bin/` + 2 integration tests + bench + Cargo manifest), core `tools_bridge.rs`, `tool_loop.rs`, `approval_wait.rs`, provider tool assembly (`hiverouter-provider/src/tool_assembler.rs` + `provider_executor.rs` tool paths), prompt assets/personas (`assets/prompts/*`, `prompts.rs`), and the tool-loop/approval/shell/jail/side-effect tests. `execution.rs` is 5,768 lines; this lane read ~2,000 of them (ranges listed in §13).

---

## 0. Verdict in one paragraph

The read/execute plumbing is real and mostly well-built: a frozen 12-tool vocabulary with JSON Schemas, per-tier allowlists, a single approval table with nonce/replay CAS, a Windows path jail with an escape matrix, an allowlisted shell runner with a Job Object and environment allowlist, and a preflight-before-consent side-effect executor. **But the security story does not hold at the boundaries.** `cmd.exe` is on the shell allowlist, so an approved `shell_exec` is arbitrary code execution as the user with the jail bounding only the working directory [V]. The `ToolsBridge` jail never denies the data directory (the engine jail does), so file tools can read/overwrite the SQLite DB and the approval ledger [V]. The HyperAgent root runs a second, unregistered tool vocabulary that bypasses the registry, tier allowlists and census entirely [V]. The grant verifier ignores the approval TTL and matches grants by `(run, tool, args)` rather than the approval/tool-call identity [V]. `finish` and `report_result` are advertised but are pure text echoes with no engine effect [V]. The census compares the registry to a hand-written constant, not to the persona/prompt assets it claims to validate [V]. Net: **agents on the Worker/Agent tier can really use read/write/shell/web tools, with a working approval UI path; the root agent cannot use its registered tools at all, and the containment claims overstate what the code enforces.**

---

## 1. File coverage inventory (including incomplete reads)

### 1.1 `backend/crates/hiverouter-tools/`

| File | Lines | Read depth | Notes |
|---|---:|---|---|
| `Cargo.toml` | — | **UNREAD** | dependencies not audited beyond imports seen in source |
| `benches/engine_benches.rs` | — | **UNREAD** | out of lane; not cited |
| `src/lib.rs` | 94 | **FULL** | module/public-export map |
| `src/tool.rs` | 755 | **FULL** | tool enum, classes, contracts, allowlists, schema validation |
| `src/census.rs` | 309 | **FULL** | derived census + drift checks |
| `src/error.rs` | 107 | **FULL** | refusal wrapper |
| `src/executor.rs` | 433 | **FULL** | read-only executor |
| `src/jail.rs` | 651 | **FULL** | path jail, read/list/write/atomic write + escape tests |
| `src/shell.rs` | 491 | **FULL** | allowlist, job object, timeout, caps + tests |
| `src/side_effects.rs` | 838 | **FULL** | preflight/commit write & shell + tests |
| `src/approval.rs` | 1,615 | **FULL** | approval state machine, TTL, nonce, grant, tests |
| `src/env_safety.rs` | 79 | **FULL** | child env allowlist |
| `src/sha256.rs` | 146 | **FULL** | in-house SHA-256 + NIST vectors |
| `src/watcher.rs` | 236 | **FULL** | 150 ms coalescing watcher |
| `src/approval.rs` tests | — | included above | 18 test functions read |
| `src/ledger.rs` | 529 | **PARTIAL** (120–239 + public-API scan) | append-only JSONL, seq, refusal-code rule |
| `src/audit.rs` | 294 | **SCAN** (public API only) | JSONL audit sink, secret substrings |
| `src/cancel.rs` | 281 | **SCAN** (public API only) | per-run/task cancellation registry |
| `src/cli.rs` | 1,116 | **SCAN** (headers + grep for fs/Command/approval) | appears discovery/schema-only; no direct fs/process calls found |
| `src/cli_support.rs` | 148 | **SCAN** (public API only) | version probes, HTTP JSON fetch |
| `src/commands.rs` | 226 | **PARTIAL** (55–134 + public API) | `FILE_READ` cap/UTF-8, `APPROVAL_RESOLVE` wrapper |
| `src/db_admin.rs` | 226 | **SCAN** (public API only) | backup/restore; out of lane |
| `src/health.rs` | 452 | **PARTIAL** (300–419 + public API) | `/health`, `/diagnose`, census exposure |
| `src/settings.rs` | 56 | **SCAN** (public API only) | key presence only |
| `src/bin/hiverouter.rs` | 579 B | **UNREAD** | CLI entrypoint |
| `tests/audit_jsonl.rs` | 2,163 B | **SCAN** (test names) | 1 test |
| `tests/lead_bugcheck_wp01.rs` | 6,404 B | **SCAN** (test names) | 3 tests: unique index, migration safety, CAS labels |

### 1.2 `backend/crates/hiverouter-core/` and provider/prompt assets

| File | Lines | Read depth | Notes |
|---|---:|---|---|
| `src/tools_bridge.rs` | 995 | **FULL** | core↔tools wiring, approval ledger, replay projections |
| `src/tool_loop.rs` | 349 | **FULL** | planning primitives, budgets, specs |
| `src/prompts.rs` | 475 | **FULL** | template/persona loading, licence/ref gates |
| `src/approval_wait.rs` | 174 | **FULL** | waiter map (arm/wake/disarm) |
| `src/execution.rs` | 5,768 | **PARTIAL (~2,000 lines)**: 170–349, 1290–1429, 1430–1629, 1900–2059, 2500–2799, 2810–3069, 3100–3479, 3600–4059, 4300–4449, 4800–5089 | tool authorization/loop/approval/side-effect/root paths |
| `src/provider_executor.rs` | 1,143 | **PARTIAL (~700 lines)**: 1–150, 150–279, 430–629, 700–879 + greps | tool specs, root control/finalize, persona prompt/allowlist filtering |
| `src/config.rs` | 738 | **PARTIAL** (400–479 + grep) | data dir/workspace defaults |
| `src/http.rs` | 3,715 | **PARTIAL** (250–368 + grep) | bridge/jail construction, approvals route |
| `src/ws.rs` | 63,952 B | **SCAN** (grep only) | APPROVAL_RESOLVE/REPLAY wiring references |
| `../hiverouter-provider/src/tool_assembler.rs` | 736 | **PARTIAL** (1–400; test names scanned) | streamed tool-call assembly |
| `../hiverouter-contract/src/event.rs` | 1,102 | **PARTIAL** (284–423 + grep) | approval event, args preview bounds/redaction |
| `assets/prompts/manifest.json` | 1,555 B | **FULL** | licence/source rows |
| `assets/prompts/personas.json` | 2,500 B | **FULL** | 8 personas + tool allowlists |
| `assets/prompts/*.prompty` (8 files) | 517–890 B | **FULL** | planner, superagent, subagent, agent, content-writer, data-analyst, code-reviewer, chatbot |
| `tests/suspension.rs` | 44,041 B | **PARTIAL** (1–85 + test-name/line scan) | 14 approval/side-effect integration tests |
| `tests/tool_loop_engine.rs` | 26,744 B | **SCAN** (test names) | read-only loop, budgets, continuation |
| `tests/tool_loop_matrix.rs` | 16,833 B | **SCAN** (test names) | SSE-level loop matrix |
| `tests/approval_ledger.rs` | 8,870 B | **SCAN** (test names + grep) | ledger wiring |
| `tests/prompt_assets.rs`, `tests/prompt_assets_wiring.rs` | 5,266 / 9,707 B | **SCAN** (test names) | template/persona loading |
| `tests/provider_adapter.rs`, `tests/ws_transport.rs`, `tests/http_boot.rs`, `tests/lead_bugcheck_boot.rs` | — | **SCAN** (grep) | persona filters / approval transport |

**Honesty note:** nothing in this report was executed. Every "passes/works" statement about tests is **not** made; test existence and assertions are [V-read], outcomes are [U].

---

## 2. The registry contract (`tool.rs`)

### 2.1 Vocabulary and classes

- 12 `ToolId` variants, enum-derived (never a typed count): `spawn_agent`, `ask_parent`, `send_to_sibling`, `report_result`, `finish`, `file_read`, `file_list`, `file_write`, `shell_exec`, `request_detached`, `web_fetch`, `vote_evolution` — `tool.rs:115-173`.
- 5 permission classes: Read, Mutate, Execute, Network, Control — `tool.rs:30-42`.
- Approval rule: `requires_approval() = !(Read | Control)` — `tool.rs:59-61`. So **Mutate/Execute/Network require approval; Read and Control do not**.
- 5 tiers: HyperAgent, SuperAgent, WorkerAgent, Subagent, ToolAgent — `tool.rs:76-107`.
- Each contract carries id, display name, description (offered to the model), class and a schemars-derived JSON Schema with `additionalProperties:false` — `tool.rs:176-224`.
- Validation uses `jsonschema::validate` and maps failure to `E_TOOL_SCHEMA` — `tool.rs:201-208`; `resolve_call` maps unknown name to `E_TOOL_NOT_FOUND`, missing tier grant to `E_TOOL_NOT_ALLOWED`, then validates args — `tool.rs:573-588`.

**Design note [V]:** `resolve()` first looks up `contracts`, but `standard()` inserts a contract for every enum variant (`tool.rs:382-458`, test `tool.rs:612-623`), so `E_TOOL_NOT_FOUND` from `resolve` is unreachable in v1; only `resolve_call`'s `from_id` path can produce it.

### 2.2 Frozen allowlists (`tool.rs:460-516`)

| Tier | Grants | Count |
|---|---|---:|
| HyperAgent | spawn_agent, report_result, finish | 3 |
| SuperAgent | all 12 | 12 |
| WorkerAgent | all except vote_evolution | 11 |
| Subagent | ask_parent, report_result, finish, file_read, file_list | 5 |
| ToolAgent | file_read, file_list, file_write, shell_exec | 4 |
| **Total** | | **35** (matches `census.rs:234`) |

**Bug/risk [V]:** ToolAgent is unreachable from the run graph. Planned node roles are validated with `role_class` (`execution.rs:2002`), and `role_class("tool")` returns `None` (`execution.rs:201-214`); spawn roles are limited by `validate_spawn_role`, which never permits `tool` (`execution.rs:244-275`). The AgentTier mapping supports `"tool"` (`execution.rs:293`) but nothing can create such a task. Therefore 4 of the 35 grants (ToolAgent) are dead. HyperAgent's 3 grants are also effectively dead because the root never goes through the tool loop (§7.3). **7/35 grants (20%) are unreachable.** [V]

---

## 3. Tool-by-tool record (contract → actual lane)

Legend: **Class/Approval** from `tool.rs`; **Actual lane** from the executor branch that really handles it. "Tier allow" = registry allowlist (not persona).

### 3.1 `spawn_agent`
- **Contract:** "Spawn a child agent (depth/fan-out/agent caps enforced before any spawn)." Class **Execute** → approval required by class. Schema: `role` (req), `instruction` (req), optional `parent_task_id` (claim only) — `tool.rs:226-237,384-389`.
- **Tier allow:** Hyper, Super, Worker (not Subagent/ToolAgent) — `tool.rs:461-497`.
- **Actual executor lane:** loop-owned, **no approval**. `execution.rs:2652-2659` calls `execute_spawn_agent` before the approval branch; `spawn_sub_hive` checks run terminal, role class, spawn matrix, fan-out, depth, caps — `execution.rs:1550-1616`; the model's `parent_task_id` claim must equal the caller — `execution.rs:4007-4019`; empty role refused `E_TOOL_SCHEMA` — `4020-4026`; child run inline via `execute_task_with_prompt` — `4052-4053`.
- **Refusal codes:** `E_TOOL_NOT_ALLOWED`, `E_TASK_NOT_FOUND`, `E_RUN_TERMINAL`, `E_SPAWN_DEPTH`, `E_FANOUT_LIMIT`, `E_AGENT_LIMIT`, `E_PERSONA_UNKNOWN` (`execution.rs:1550-1616, 4007-4038`).
- **Tests:** registry/allowlist tests only (`tool.rs:653-685`); no tool-loop spawn test found in the files scanned. **Approval-class mismatch:** Execute normally means approval, but the branch is explicitly loop-owned. [V]

### 3.2 `ask_parent`
- **Contract:** bounded message to parent; Class **Read** → no approval. Schema `text` — `tool.rs:239-245,390-395`.
- **Tier allow:** Super, Worker, Subagent. (Not HyperAgent, not ToolAgent.)
- **Actual:** `execute_ask_parent` — `execution.rs:3603-3858`; deserializes args, requires a parent, bounds text by chars, refuses duplicate open question, records an agent message with per-run budget, sets an A2A deadline, may spend a provider call for the parent answer, otherwise publishes and waits on a oneshot/cancellation/timeout. Returns answered JSON or named refusal.
- **Effect:** writes DB rows and publishes events; can block the task for the A2A deadline; calls the provider (`execution.rs:3736-3751`). A "Read" class tool that mutates state and spends budget — classification mismatch (see risk R7).
- **Tests:** `ask_parent_writes_a_durable_message` (`tests/tool_loop_engine.rs:324`). [V-read]

### 3.3 `send_to_sibling`
- **Contract:** Class **Mutate** → approval required by class; description says cross-branch requires operator approval — `tool.rs:396-401`.
- **Tier allow:** Super, Worker only.
- **Actual:** same-parent message executes **without approval** (`execution.rs:2687-2688`); cross-branch is detected by `is_cross_branch_message` (`3866-3879`) and goes through `suspend_for_approval` (`2663-2686`). On an approve decision, `resume_and_execute` calls `execute_send_to_sibling(..., true)` (`execution.rs:3352-3355`) **without** `verify_tool_grant`; the decided row is the only gate. Bounds: target must exist, same run, same/approved parent, not self, non-terminal, bounded text, per-run message cap (`3881-3981`).
- **Tests:** `cross_branch_message_requires_approval_then_writes_once` (`tests/suspension.rs:500`). [V-read]
- **Mismatch:** Mutate-class same-branch path is unapproved; Subagent persona claims this tool but the Subagent tier does not grant it (`personas.json:36` vs `tool.rs:498-507`), and `authorize_tool_call` requires both (`execution.rs:1476-1481`), so the persona claim is a dead advertisement.

### 3.4 `report_result`
- **Contract:** "Report a task result to the parent task." Class **Read**; `{summary, success}` — `tool.rs:402-407,257-265`.
- **Tier allow:** Hyper, Super, Worker, Subagent.
- **Actual:** `execute_report_result` deserializes and returns `{"summary","success"}` as the tool-result text — `executor.rs:88-103`. **No engine effect**; no parent row update, no run event beyond the generic `TOOL_RESULT`, no task completion.
- **Tests:** `executor.rs:270-302` asserts the text echo only. [V]

### 3.5 `finish`
- **Contract:** "Finish the current task with a final verdict." Class **Read**; `{verdict}` — `tool.rs:408-413,267-273`.
- **Tier allow:** all five tiers.
- **Actual:** `execute_finish` echoes `{"verdict": ...}` — `executor.rs:105-117`. The comment says "the engine still owns the actual terminal state transition" (`executor.rs:105-106`), but the tool-turn dispatcher has no `finish` branch: it checks `spawn_agent`, `ask_parent`, `send_to_sibling`, `request_detached`, `vote_evolution`, then approval and read-only fallbacks (`execution.rs:2652-2709`). The loop ends only when the model returns no tool calls or the turn cap is hit.
- **Consequence:** the model can call `finish` and the task keeps going; the description is false in effect. **High severity functional deception.** [V]
- **Tests:** echo test only (`executor.rs:289-301`). No test asserts `finish` terminates a task.

### 3.6 / 3.7 `file_read`, `file_list`
- **Contract:** Read, no approval — `tool.rs:414-425`; schemas `path` and optional `path` — `tool.rs:295-310`.
- **Tier allow:** Super, Worker, Subagent, ToolAgent (not Hyper).
- **Actual:** `execute_read_only` routes after `registry.resolve_call` — `executor.rs:51-85`; `jail.read` enforces containment, regular-file, 1 MiB cap, UTF-8 — `jail.rs:216-245`; `file_list` returns sorted `{name,is_dir,size}` JSON — `jail.rs:380-417`, `executor.rs:139-167`.
- **Refusals:** `E_PATH_OUTSIDE_JAIL`, `E_PATH_NOT_FOUND`, `E_FILE_TOO_LARGE`, `E_TOOL_SCHEMA`, `E_SCHEMA_INVALID` (binary/UTF-8) — `jail.rs:216-245`, `executor.rs:119-137`.
- **Tests:** jail unit tests (`jail.rs:446-650`), executor tests (`executor.rs:187-383`), engine read test (`tool_loop_engine.rs:359`), matrix tests (`tool_loop_matrix.rs:291`). [V-read]

### 3.8 `file_write`
- **Contract:** Mutate, approval required; `{path, content, expected_sha256?}` — `tool.rs:426-431,312-323`.
- **Tier allow:** Super, Worker, ToolAgent. Persona `agent` includes it; persona `superagent` does not (`personas.json:25,72`).
- **Actual:** approval path only. `preflight_side_effect` → `preflight_file_write` (empty path, 1 MiB content cap, jail resolve, root/dir/symlink refusals, optional 64-hex stale-hash check via `jail.read`) — `side_effects.rs:298-391`; then `verify_tool_grant`; then `commit_side_effect` → `commit_file_write` re-resolves and compares the prepared target, then `jail.write_atomic` (same-dir temp, `create_new`, fsync, rename) — `side_effects.rs:393-420`, `jail.rs:290-373`.
- **Result text:** bytes/path/sha only, never content — `side_effects.rs:411-419`; bounded at 64 KiB — `side_effects.rs:28-31,423-435`.
- **Tests:** `commit_writes_and_reports_path_size_sha_without_content`, `commit_replaces_existing_file_atomically_without_temp_residue`, stale-hash tests (`side_effects.rs:605-704`), approval e2e `approve_resumes_and_executes_the_file_write_exactly_once` (`tests/suspension.rs:618`). [V-read]

### 3.9 `shell_exec`
- **Contract:** Execute, approval required; `{program,args,cwd?}` — `tool.rs:432-437,325-336`.
- **Tier allow:** Super, Worker, ToolAgent. Persona `agent` includes it; `superagent` excludes it.
- **Actual:** preflight bounds program to a bare allowlisted name, ≤64 args, ≤4 KiB/arg, ≤16 KiB argv, NUL-free, cwd inside jail and existing — `side_effects.rs:177-274`; grant verified; `commit_shell_exec` → `shell::execute` — `side_effects.rs:276-296`. Runtime contract in §9.
- **Critical:** the default allowlist includes `cmd.exe` (`shell.rs:66-72`), so an approved call can execute arbitrary commands. See R1.
- **Tests:** unit tests in `shell.rs:368-474`; shell preflight/commit tests `side_effects.rs:747-837`; e2e `shell_exec_runs_only_the_allowlisted_program_after_approval` (`tests/suspension.rs:1041`). Note the suspension header still claims the shell arm is a planned RED test (`tests/suspension.rs:36-39`) while a shell arm test now exists at line 1041 — stale documentation [V]; suite status [U].

### 3.10 `request_detached`
- **Contract:** Control → **no approval**; `{kind, display_name, reason, request_id?}` — `tool.rs:438-443,338-351`.
- **Tier allow:** Super, Worker.
- **Actual:** `execute_request_detached` — `execution.rs:4931-5089+`: caller must be SuperAgent or WorkerAgent (`4948-4954`), `kind` ∈ {remote,daemon} (`5020-5025`), name non-empty ≤120 and unique across remotes/daemons (`4958-4972,5036-5047`), reason ≥12 chars (`4973-4984`), optional idempotency key with fingerprint and conflict refusal (`4985-5019`), per-run detached cap 4 (`5048-5062`), then creates a remote/daemon row and records admission/control decision. **This is a real control-plane mutation with no operator approval**, gated only by deterministic engine policy and a 12-character reason.
- **Tests:** not read beyond grep; `remote_control_plane.rs`/`run_lifecycle.rs` cover detached principals, but the tool-loop entry itself is [U].

### 3.11 `web_fetch`
- **Contract:** Network → approval required; one public HTTPS URL, no redirects, no credentials/private networks, 1 MiB — `tool.rs:444-449,287-293`.
- **Tier allow:** Super, Worker.
- **Actual:** `resume_and_execute` first checks the global side-effect kill switch, then **verifies and consumes the grant**, then calls `execute_web_fetch` (`execution.rs:3356-3390`). `execute_web_fetch` validates URL scheme/port/userinfo/localhost, resolves DNS, rejects private/loopback/link-local/unspecified/multicast addresses, builds a no-redirect client, bounds the body to 1 MiB and the request to 15 s, and selects on cancellation (`execution.rs:2819-2993`).
- **Ordering bug:** URL/SSRF validation happens **after** the single-use grant is consumed; a doomed URL burns the approval (`execution.rs:3378-3390` vs `2828-2913`). There is no preflight for `web_fetch` comparable to `file_write`/`shell_exec`.
- **DNS rebinding:** the pre-check resolves the host (`2870-2906`), then `reqwest::Client` resolves it again when connecting (`2914-2931`); the client is not pinned to the checked IPs.
- **Tests:** none dedicated found in the scanned test files; the code path is exercised only indirectly. [U]

### 3.12 `vote_evolution`
- **Contract:** Control → no approval; SuperAgent only; `{proposal_id, vote, rationale}` — `tool.rs:450-455,275-285`.
- **Tier allow:** SuperAgent only.
- **Actual:** `execute_vote_evolution` — `execution.rs:4824-4924`: tier check, proposal must exist as `improvement:<id>` in run memory, vote ∈ approve/reject, rationale 8..=400 chars, duplicate voter/task refused with `E_APPROVAL_REPLAY`, then records `evolution_vote:<proposal>:<task>` memory with `applied:false`. Votes never apply code.
- **Tests:** `tests/proposal_authorization.rs` and `run_lifecycle.rs` cover proposal/approval flows (grep); this tool's own loop path is [U].

---

## 4. The tier × tool matrix — registry vs persona vs reality

Legend: **R** = registry grant, **P** = persona allowlist (effective when a matching persona is loaded), **A** = actually offered/executable in the tool loop, **X** = dead/blocked.

| Tool | Hyper R | Super R/P | Worker R/P (agent) | Sub R/P | ToolAgent R | Actual execution lane | Approval |
|---|---|---|---|---|---|---|---|
| spawn_agent | R (dead) | R/P | R/P | — | — | loop-owned `execute_spawn_agent` | **none** |
| ask_parent | — | R/P | R/P | R/P | — | loop `execute_ask_parent` | none (Read) |
| send_to_sibling | — | R/P | R/P | X (P only) | — | same-branch loop; cross-branch approval | **none same-branch**; approval cross |
| report_result | R (dead) | R/P | R/P | R/P | — | read-only echo | none |
| finish | R (dead) | R/P | R/P | R/P | R | read-only echo | none |
| file_read | — | R/P | R/P | R/P | R | jail read | none |
| file_list | — | R/P | R/P | R/P | R | jail list | none |
| file_write | — | R / (P excludes) | R/P | — | R | preflight+commit atomic | **yes** |
| shell_exec | — | R / (P excludes) | R/P | — | R | allowlist shell | **yes** |
| request_detached | — | R/P | R/P | — | — | loop `execute_request_detached` | none (Control) |
| web_fetch | — | R/P | R/P | — | — | verify→fetch | **yes** |
| vote_evolution | — | R/P | — | — | — | loop `execute_vote_evolution` | none (Control) |

Notes:
- **Hyper column:** the root task role is `"planner"` (`execution.rs:1937-1941`) → `AgentTier::HyperAgent` (`284-299`). But the root is driven by `orchestrate_root` with six **unregistered** tools (`spawn_superagent`, `spawn_remote_agent`, `spawn_daemon`, `read_memory_summary`, `propose_improvement`, `finish`) — `provider_executor.rs:567-666`, dispatched by name in `execution.rs:4381-4811`. Its persona allowlist is empty (`personas.json:3-7`), and `seed_task_context` marks it persona-scoped (`execution.rs:1318-1322`), which would zero its registry tools if it ever used the tool loop. The registry HyperAgent row is therefore a **fiction**; the real root surface is not in `ToolId`, not schema-validated, not in the census, and not approval-gated. [V]
- **Worker personas:** `content-writer`, `data-analyst`, `code-reviewer` exclude write/shell/spawn/request_detached (`personas.json:16,45,54`); `agent` includes the full Worker set minus vote (`personas.json:72`). The provider filters its advertised specs by persona (`provider_executor.rs:191-200`, wired at `http.rs:276-284`), so the actual offer is the intersection, and `authorize_tool_call` re-checks both (`execution.rs:1470-1483`).
- **Subagent:** persona lists `send_to_sibling` (`personas.json:36`) but the tier does not (`tool.rs:498-507`), so it is offered nowhere and would be refused if called.
- **ToolAgent:** no persona exists and no graph path creates it; its four grants are dead (§2.2).

**Can agents really use their tools?**
- A normal `agent`/`worker` task (role `agent`, `worker`, `content-writer`, ...) **can**: read/list, approved write, approved shell, approved web fetch, ask/send/report/finish primitives, spawn one subagent, request a detached principal, with call/turn/approval budgets enforced (`execution.rs:2540-2734`).
- The root **cannot** use its registered `spawn_agent`/`report_result`/`finish`; it uses a different vocabulary.
- `finish`/`report_result` **do not do what their descriptions say**.
- `ToolAgent` and Subagent `send_to_sibling` are advertised capabilities that cannot execute.

---

## 5. Capability census — what it proves and what it hides

`census.rs` derives `tool_count`, `tier_count`, `permission_class_count`, `refusal_count`, `tools_by_tier`, `registry_tool_ids`, `allowlist_tool_ids`, `prompt_claim_tool_ids`, `executor_dispositions` — `census.rs:57-201`. `standard_census()` = standard registry + `PROMPT_CLAIMED_TOOL_IDS` (`census.rs:207-209`). Counts in the test: 12 tools, 5 tiers, 5 classes, 35 grants, `refusal_count >= 40` — `census.rs:224-241`. The `/diagnose` payload exposes the census (`health.rs:124,166,257,315-322`; test `health.rs:391-419`).

What it checks [V]:
- Registry id set == deduplicated allowlist id set == hand-written `PROMPT_CLAIMED_TOOL_IDS` (`census.rs:99-133`).
- Every registry row has a declared `EXECUTOR_DISPOSITIONS` lane (`census.rs:38-51,166-181`), values: `read_executor`, `approval_executor`, `loop_owned`.

What it does **not** check [V]:
1. **It is not derived from the prompt/persona assets.** `PROMPT_CLAIMED_TOOL_IDS` is a literal list (`census.rs:20-33`); no test parses `personas.json` or the `.prompty` files. A persona can claim a tool the tier forbids (Subagent/`send_to_sibling`) and the census stays green.
2. **It does not validate class ↔ disposition.** `spawn_agent` is Execute (approval-required by class) but disposition `loop_owned`; `send_to_sibling` is Mutate but same-branch execution has no approval; `ask_parent` is Read but writes DB and calls the provider. None of these contradictions fails `derive_census`.
3. **It does not require the declared lane to be reachable.** ToolAgent's four grants and HyperAgent's three grants are dead, yet the census counts them. 7/35 grants are unreachable.
4. **It is blind to the root control vocabulary** (`spawn_superagent`, `spawn_remote_agent`, `spawn_daemon`, `read_memory_summary`, `propose_improvement`) which is defined only in `provider_executor.rs:583-666` and dispatched manually in `execution.rs:4381-4811`.
5. **The "prompt claims" union happens to match** (the 8 personas' union is all 12 ids), but the per-persona mapping is inconsistent and unverified by any test.
6. Drift tests only delete a row from the derived arrays (`census.rs:243-280`) — they do not edit the actual registry or personas; the "delete one row and expect failure" claim tests the checker, not the source of truth.

---

## 6. Approval gate and UI wiring

### 6.1 State machine (`approval.rs`)
- Table `approvals` with `UNIQUE(nonce)`, decision/decided_at/reason/executed_at; two identical indexes; one-pending-per-call partial unique index created only after a duplicate check; duplicates are never deleted — `approval.rs:149-205`.
- Request binds `(run, task, tool_call_id, tool, class, args_sha256 = SHA-256(JSON args), created_at, expires_at = now + 120 s, nonce=uuidv7)` — `approval.rs:25-26,83-106,851-882`. Argument bodies are never stored; only the digest (`approval.rs:130-134`; tests `1424-1446`).
- Decision: `approve→allow_once`, `deny→deny` on the wire; `allow_session` is programmatic only and rejected from the wire (`approval.rs:62-80`; test `1410-1422`). **The UI cannot grant "allow for rest of run".** [V]
- Resolution: load, reject decided/expired, CAS `WHERE decision IS NULL`, else `E_APPROVAL_REPLAY`/`E_APPROVAL_TIMEOUT` — `approval.rs:357-430,498-600`. Nonce comparison is byte-wise constant-time-ish over the fixed length — `466-477`; tests `1570-1612`.
- TTL: 120 s; expiry consumes the row as `decision='expired'` (`397-412`); live TTL reaper `718-748`; boot orphan reaper unconditionally expires all pending (`694-707`); cancel/expire CAS helpers `618-646`.
- Session: `session_allowed(run, tool, args_sha256)` counts `allow_session` rows — no expiry, no single-use, no revocation (`436-450`).
- Grant verification: `verify_grant` checks session first, then finds the oldest `allow_once` row by `(run, tool, args_sha256)`, and CASes `executed_at` — `924-982`.
- **TTL bug:** `now_unix_secs` is explicitly discarded (`approval.rs:932-935`) and the query never looks at `expires_at` (`942-949`). A decided `allow_once` row is executable indefinitely, and by any later call with identical args, because the query ignores `approval_id` and `tool_call_id`. The module doc claims TTL expiry returns `E_APPROVAL_TIMEOUT` with zero side effects (`approval.rs:4-10`), and the method doc claims `E_APPROVAL_TIMEOUT` (`917-923`); neither is enforced at grant time. Only the *decision* window is TTL-bounded.
- **Ledger fail-open [V, high]:** `resolve_approval_ledgered` resolves (commits the decision), then appends the ledger row; on append failure it calls `cancel_approval` claiming it will "consume/cancel the row before returning so the TTL executor can never perform the effect without a ledger row" — `tools_bridge.rs:364-396`. But `cancel_pending` only updates `WHERE decision IS NULL` (`approval.rs:634-646`), and the decision is already set, so it is a no-op. The engine remains suspended; when its wait reaches `expires_at`, `finish_timeout_wait` calls `expire_approval` (also `decision IS NULL`, fails), then reads the committed `allow_once` label and **executes it** (`execution.rs:3447-3469`). Execution after a ledger-write failure therefore happens without ledger evidence, contrary to the stated fail-closed design.
- **Session behavior:** `verify_grant` returns `Ok` for any number of matching session grants, with no expiry — intended "rest of run", but there is no revocation path; an operator cannot withdraw an allow-session decision. [V]

### 6.2 Engine suspension (`execution.rs`)
- Order: budget check → `request_model_tool_approval` (row committed first) → arm waiter → persist awaiting transitions → snapshot → publish `APPROVAL_REQUEST` → check if already decided → wait on timeout/cancel/waiter — `execution.rs:3151-3280`.
- Waiter map arms before publish and refuses duplicate arming (`approval_wait.rs:37-52`); wake is one-shot (`57-66`). 14 integration tests claim suspend/approve/deny/timeout/cancel/kill/restart/dual-resolve/wait-cap/wrong-args/shell coverage (`tests/suspension.rs:8-14,500-1110`).
- Wait cap: `MAX_APPROVAL_WAITS_PER_RUN = 8` (`tool_loop.rs:19-21`); enforced by counting **all approval rows ever** for the run before requesting (`execution.rs:3174-3185`). Test asserts exactly 8 approvals and no ninth row (`tests/suspension.rs:940-991`). Note this is a lifetime row count, not a live-wait count, and it counts refusals.
- Deny resumes running with a `Denied` result (`3424-3443`); timeout/cancel are terminal and name the refusal (`3447-3472`, `3578+`); approve re-authorizes tier+persona before execution (`3319-3355`).
- `execute_approved` checks the global `HIVEROUTER_SIDE_EFFECTS` kill switch before preflight, preflights, verifies the grant, then commits exactly once (`tools_bridge.rs:213-253`).
- `web_fetch` and cross-branch `send_to_sibling` bypass `execute_approved` (`execution.rs:3352-3390`); `send_to_sibling` does not consume/verify the grant at all, while `web_fetch` consumes it before validation (see §3.11).

### 6.3 UI / event wiring
- Fresh request event carries `approval_id`, tool, `nonce`, run/task/tool_call ids, `args_sha256`, `expires_at`, and a redacted bounded `args_preview` — `tools_bridge.rs:492-524`; `approval_request_event_with_args` is used only when the producer still holds the args (`470,483`).
- Replay after reconnect (`pending_approval_events_all`) uses `approval_request_event`, which sets `args_preview: None` — `tools_bridge.rs:508-524,556-581`. **The operator reconnecting sees no arguments at all**, only the digest and tool name, and must approve blind. This is the main UI/approval wiring break.
- Preview bounds: 512 UTF-16 units total, 160 chars per string, 12 keys, 8 array items, depth 4 (`event.rs:294-313,370-420`). For `shell_exec`, a command is one string truncated at 160 chars; with up to 64 args the operator cannot review the full command. The full args are still bound by hash, so execution is exact even when the display is not.
- Transport wiring exists and is tested by grep: replay on `SERVER_HELLO` (`ws.rs:678`, `MAX_APPROVAL_REPLAY_EVENTS`), `APPROVAL_RESOLVE` → `resolve_approval_ledgered` → publish `APPROVAL_RESOLVED` and notify the waiter (`ws.rs:1095-1108`); tests `ws_transport.rs:513`, `http_boot.rs:678-690`, `approval_ledger.rs:80-193`. End-to-end status [U].
- `approval_resolved_event` maps `AllowSession` to the wire label `"approve"` (`tools_bridge.rs:596-607`), losing the distinction — moot while session is unreachable from the wire.
- `/api/approvals` returns at most 20 pending events for non-terminal runs (`http.rs:1376`, `tools_bridge.rs:556-581`); expired rows are skipped.

---

## 7. Personas, prompts and the root surface

### 7.1 Assets and loading
- 8 personas, each with id, display name, tier, template, `allowed_tool_ids`, output schema, refusal policy, optional child/depth caps — `personas.json:1-75`; typed model `prompts.rs:371-408`.
- Licence gate accepts MIT/Apache/BSD/ISC/CC0 and requires `source_url` + `fetched_at` — `prompts.rs:22-30,219-244`; reference gate rejects `${env:}`/`${file:}` — `prompts.rs:284-286`; malformed assets map to named refusals — `287-305`.
- All 8 templates and both manifests are compile-time embedded (`prompts.rs:38-77`), so a release cannot silently omit them; tests `prompt_assets*` check loading, bad licence, missing file, empty body, env refs.
- Rendering splits `system:`/`user:` markers and substitutes `{{ key }}` — `prompts.rs:320-369`.
- Persona prompt selection: `persona_key` normalizes `super→superagent`, `worker→agent`, `planner|hyper→planner`, `sub→subagent`, otherwise the id — `provider_executor.rs:165-177`; the system prompt is looked up by role and falls back to `TASK_SYSTEM` — `179-189`. Tool specs are filtered by the persona allowlist map — `191-200`, wired from `http.rs:276-284`.

### 7.2 Claims vs reality
- Persona `planner` has an **empty** allowlist (`personas.json:7`), and `seed_task_context` sets `persona_tool_scope=true` with that empty list (`execution.rs:1318-1322`); `authorize_tool_call` treats empty as "no tools" (`1477-1481`). This is masked only because the root does not use the tool loop. If the root were ever routed through it (or a planner-role worker were created), it would have **zero** tools — `[P]` fix: treat "persona absent" as unscoped, but keep an explicit empty list as "no tools" and never mark the root scoped.
- Persona `subagent` declares `tier: "worker"` (`personas.json:34`) while the engine maps role `subagent` to `AgentTier::Subagent` (`execution.rs:294`); the `PersonaTier` enum is not used to derive the execution tier. The record is internally inconsistent with the runtime.
- `PersonaTier::Tool` exists (`prompts.rs:371-383`) but no persona uses it and no spawn path creates a ToolAgent — dead capability.
- The root's prompts say it may call `spawn_superagent`, `spawn_remote_agent`, `spawn_daemon`, `read_memory_summary`, `propose_improvement`, `finish` (`provider_executor.rs:39-53,583-666`) and `execution.rs:4300-4307` says the same. None of these are `ToolId`s. The `planner.prompty` system text says "return a task DAG" and is not the system prompt actually used for planning (`provider_executor.rs:26-33,98-116` uses `PLANNER_SYSTEM`; `execution.rs:2515-2527` renders the planner template into the user prompt). So prompt claims, persona claims and registry claims describe **three different tool surfaces**.

### 7.3 Root control loop
- `root_checkpoint` runs at most two turns (`execution.rs:4328`), at most four tool calls per turn (`4319,4349-4358`), and dispatches manually with custom gates:
  - `spawn_superagent` — only if the planner produced no depth-1 branch and this turn hasn't spawned one (`4382-4408`).
  - `spawn_remote_agent` / `spawn_daemon` — duplicate-name and turn-2 creation refusals; record admission (`4410-4470+`).
  - `propose_improvement`, `read_memory_summary`, `convene_evolution_vote`, `finish` — additional branches; unknown names are logged and refused (`4808-4811`).
- These calls do **not** go through `ToolRegistry::resolve_call`, `plan_call`, schema validation, `TOOL_CALL`/`TOOL_RESULT` events, `tool_turns`, approval, or the capability census. They do get run-gate audit rows for some actions (`record_root_gate_decision`) but not a tool-call audit trail. [V]
- `finalize_root` advertises **no tools** (`provider_executor.rs:672-698`), so the root cannot mutate after its final answer. Good boundary.

---

## 8. Jail analysis (`jail.rs`)

**Containment model [V]:**
- Absolute root canonicalized at open; relative roots refused (`jail.rs:57-81`).
- Six escape classes refused, never clamped: `..` (`141-143`), prefix/drive (`133-135`), rooted absolute (`136-140`), UNC/device (`125-127`), `:` (drive/ADS) in any segment (`147-151`), `~` (8.3 alias) (`152-156`), NUL (`119-121`).
- Containment verifies the canonical form of the target or its nearest existing ancestor, catching symlink/junction escapes (`171-196`); case-insensitive boundary-safe comparison (`201-207`); denied subtrees checked after canonicalization (`186-188`, `100-108`).
- Read: regular file only, 1 MiB cap, UTF-8 enforced at the executor (`216-245`, `executor.rs:129-135`).
- Write: parent containment check + `create_dir_all` + plain `fs::write` (`254-274`); atomic write: refuses root/dir/symlink targets, same-directory unique temp with `create_new`, fsync, rename, temp cleanup on failure (`290-373`).
- Tests exercise all six vectors plus a real Windows junction/symlink (`472-569`), the read cap (`571-578`), denied data subtree (`458-469`), atomic create/replace/no residue (`599-641/643-650`). **These are read, not run [U].**
- Deny semantics: `deny_path` silently ignores a path that cannot be canonicalized (`92-98`). If the data directory is created after the jail (it is created before boot in `config.rs:566-570`, so the canonical path normally exists), or a denied dir is renamed/recreated at the same path, the deny list holds the old identity; a new directory at the same spelling is still caught because comparison is by canonical path string, not by inode. The bigger issue is that the **bridge jail never calls `deny_path` at all** (see R2).
- TOCTOU: all operations are path-based `resolve → metadata → fs::*`. A junction/symlink swapped in after `verify_containment` can redirect the actual read/write. `write_atomic` narrows this with `create_new` temp + rename, and `commit_file_write` re-resolves and compares the prepared target (`side_effects.rs:399-405`), but the window remains and is explicitly documented (`side_effects.rs:406-410`). `cap-std` was deliberately not adopted (`jail.rs:14-17`). [V]

**Does the data directory escape?**
- Engine read jail: `build_engine` opens the workspace and calls `.deny_path(&config.data_dir)` — `http.rs:347-351`.
- ToolsBridge jail (used by `FILE_TREE_LIST`, `FILE_READ`, and **all approved `file_write`/`shell_exec` via `execute_approved`**): `ToolsBridge::new(&config.workspace_root, db)` with no deny path — `http.rs:259-262`, `tools_bridge.rs:53-80`.
- Defaults: `data_dir = "data"` relative to the process cwd and `workspace_root = std::env::current_dir()` — `config.rs:12,428-432,566-600`. **So by default `data/` is inside the workspace.**
- Consequence: `file_read` can read `data/hiverouter.db` (SQLite binary is refused as non-UTF-8, but `data/ledger/run-ledger.jsonl` is text), and an approved `file_write` can target `data/hiverouter.db` or `data/ledger/run-ledger.jsonl` and corrupt/forge the approval evidence or the database. `shell_exec` can also `cd data` because the cwd jail resolves it. This directly contradicts the engine jail's data-dir protection and is a top finding (R2). [V]

---

## 9. Shell analysis (`shell.rs`)

- Policy is an explicit program allowlist by **bare executable name**; separator-bearing names refused; comparison case-insensitive/trimmed — `shell.rs:94-103,179-188`. Preflight repeats the check and bounds args (`side_effects.rs:189-255`).
- Default allowlist: **`cmd.exe`, `git.exe`, `cargo.exe`** — `shell.rs:61-72`.
- Spawn: `Command::new(program)` with args directly, `current_dir` resolved through the jail and checked `is_dir`, stdin null, stdout/stderr piped, `kill_on_drop`, environment cleared and repopulated from a 12-variable allowlist (`PATH, PATHEXT, SystemRoot, WINDIR, TEMP, TMP, USERPROFILE, HOME, COMSPEC, OS, PROCESSOR_ARCHITECTURE, NUMBER_OF_PROCESSORS`) — `shell.rs:189-216`, `env_safety.rs:11-44`. Test proves `DEEPSEEK_API_KEY` is not inherited (`shell.rs:403-428`, `env_safety.rs:50-70`).
- Windows containment: a kill-on-close Job Object is created and the child assigned **after** spawn (`shell.rs:218-223,331-356`); dropping the job kills remaining descendants. A grandchild spawned before assignment is not contained; a grandchild created after assignment is.
- Timeout: `tokio::time::timeout` over the read+wait future; dropping the future drops the child (kill_on_drop), then the job is dropped at function return — `241-282`. Cancellation selects on the token and returns `E_TOOL_CANCELLED` (`274-277`).
- Output: stdout 1 MiB / stderr 256 KiB caps, continuously drained so no pipe deadlock, `truncated` flag when bytes arrive past the cap — `25-30,306-328`; tests `432-456`.
- Result text: program, exit code, success, truncation, bounded stdout/stderr; argv deliberately omitted — `side_effects.rs:283-295`.
- **Critical:** because `cmd.exe` is allowlisted, `shell_exec(program="cmd.exe", args=["/C", "..."])` executes arbitrary commands with the user's filesystem/network rights. `git.exe` can run hooks/pagers and `cargo.exe` can run arbitrary build scripts or `cargo run`; the allowlist is not a capability boundary. The approval gate is the only control, and the preview truncates long commands. The jail bounds only the child's initial cwd; it is not an OS sandbox (no AppContainer/job filesystem limits). **"No shell interpolation" is true of Rust/SQL quoting, but false as a security property** because the allowed program is itself a shell. [V]
- API footgun: `shell::execute` is public and does not enforce the arg-count/size bounds; those exist only in `side_effects::preflight_side_effect` (`side_effects.rs:220-255`). Direct callers bypass them. [V]

---

## 10. Side effects and tool loop flow

### 10.1 Side-effect protocol
- One module owns granted effects; read-only preflight, then the core bridge verifies the single-use grant between preflight and commit — `side_effects.rs:1-13,103-154`.
- Preflight checks cancellation first, resolves the registry (unknown/tier/schema), asserts the class matches (`Mutate` for write, `Execute` for shell), then tool-specific read-only checks. `WebFetch` has no analogous preflight (see §3.11).
- Caps: write content 1 MiB; result text 64 KiB; shell program 260 B, 64 args, 4 KiB/arg, 16 KiB argv — `side_effects.rs:28-39`.
- File write: empty path refused, content cap before I/O, jail resolve, root/dir/symlink refused, optional stale hash (64 hex; current content via jail.read so the 1 MiB read cap applies), then prepared bytes/hash — `298-391`. Commit re-resolves, compares, atomic replace, returns path/size/hash only — `393-420`.
- Shell: bounds + allowlist + cwd existence before grant; commit awaits the child and returns bounded text — `177-296`.
- Prepared values have private fields and custom `Debug` that never prints content/argv — `47-82`. Good.
- `MAX_SIDE_EFFECT_TEXT_BYTES` matches core `MAX_TOOL_RESULT_BYTES` (65,536) — `side_effects.rs:31`, `tool_loop.rs:18`.

### 10.2 Tool loop (`tool_loop.rs` + `execution.rs`)
- Budgets: 12 turns/run, 40 calls/run, 8 approval waits/run, 64 KiB result body, 8 tool calls per provider message (`tool_loop.rs:14-21`, `tool_assembler.rs:42-49`).
- `plan_call`: non-empty id/name, args must be an object, args JSON text ≤ 64 KiB, SHA-256 binding; errors `E_PROVIDER_INVALID_RESPONSE` / `E_TOOL_SCHEMA` / `E_INTERNAL_ERROR` — `tool_loop.rs:147-168`.
- `validate_plan` = registry resolve_call against the task tier — `175-184`; then persona scoping (`execution.rs:1470-1483`).
- Per call: budget check counts every attempted call including refusals; `plan_call`; authorize (tier+persona) with `record_refusal` audit; publish `TOOL_CALL`; dispatch to loop-owned handlers, approval suspension, or the read-only executor; publish `TOOL_RESULT` and persist `tool_turns` — `execution.rs:2596-2734,3050-3094`.
- Continuation replays the cumulative transcript (assistant tool calls + tool results), re-offers the tier/persona-filtered specs, and either calls another tool or answers — `provider_executor.rs:700-795`. Deltas are only published once per turn (`2769-2792`).
- The read-only executor refuses every non-{file_read,file_list,report_result,finish} name with `E_TOOL_NOT_ALLOWED` before I/O, then enforces registry tier/schema before touching disk — `executor.rs:51-85`; tests prove no file is created by refused tools (`385-419`).
- Provider assembly: fragments merged per index, conflicting/duplicate ids refused, truncation/argument non-object refused, 1 MiB args cap, calls returned sorted by index — `tool_assembler.rs:1-31,319-400`; 17 tests cover the state machine (`tool_assembler.rs:437-736`).
- No parallel execution of calls in one turn: the loop `for` iterates sequentially (`execution.rs:2600-2719`). The test name "two_parallel_read_calls_execute_in_one_turn" (`tool_loop_matrix.rs:291`) means one assistant message, not concurrency. [V]

### 10.3 Root loop vs tool loop
- Root: separate method `orchestrate_root` with custom tool list, 2 turns, 4 calls/turn, manual dispatch, custom gates, no registry/approval/census (`provider_executor.rs:567-666`, `execution.rs:4300-4811`).
- Child: frozen registry tool loop with budgets, schema, tier/persona authorization and approval (§10.2).
- The two surfaces share no validation code except the provider's assembler. This is the central architectural split in the lane.

---

## 11. Tests: what exists, what is missing (all outcomes [U])

**Approval/side-effect integration** (`tests/suspension.rs`, 14 tests): suspend event and waiter, approve-once execution, deny, timeout with `E_APPROVAL_TIMEOUT`, cancel/kill, restart/orphan expiry, dual resolve one winner, wait cap 8 (`940-991`), wrong-args grant refusal (`999-1033`), shell allowlisted after approval (`1041-1108`), inline parent answer without operator (`1111`). Header caveats admit the shell arm was historically RED and the CAS-race test is best-effort (`tests/suspension.rs:32-45`) [V-read].
**Tool loop** (`tests/tool_loop_engine.rs`, 12 tests; `tests/tool_loop_matrix.rs`, 5 tests): read execution/event persistence, non-read refusal, missing jail, budget/turn caps, continuation, duplicate delta suppression, malformed/truncated SSE. **No approval, file_write, shell, web_fetch, spawn or finish test in these two files.** [V-read]
**Approval/ledger/transport**: `approval_ledger.rs` (one row per decision, replay no row, refused no row), `ws_transport.rs:513` (ACK + APPROVAL_RESOLVED), `http_boot.rs:678-690` (`/api/approvals`), `lead_bugcheck_boot.rs` (boot orphan expiry), `lead_bugcheck_wp01.rs` (unique index, migration, CAS). [V-read]
**Tool unit tests**: registry/census/executor/jail/shell/side_effects/approval/sha256/watcher/env_safety — all read, none run. `tests/audit_jsonl.rs` covers JSONL round-trip and secret probe (test name only).
**Prompt assets**: 3 tests in `prompt_assets.rs`, 11 in `prompt_assets_wiring.rs` (embedded assets, licence/ref/empty-body refusals, production constructor, health reports assets).
**Weak/tautological test [V]:** `allow_session_is_scoped_to_run_tool_and_args` ends with
`assert!(!session_allowed(...) == (hash_args(other) == hash_args(args)))` — a comparison that is true regardless of behavior (`approval.rs:1353-1355`). The preceding lines do test run/tool scoping, but **different arguments are never actually asserted** for session grants. This is a test-weakening defect, though a separate allow_once args-binding test exists (`1228-1242`).
**Missing coverage (highest value):** root control tools; `finish`/`report_result` effects; ToolAgent/Subagent persona contradictions; `request_detached` and `vote_evolution` through the model tool loop; `web_fetch` preflight/SSRF/DNS rebinding; grant TTL at execution; ledger-failure fail-open; data-dir denial on the bridge jail; session-args assertion; `cmd.exe` arbitrary-command test (the current shell test only echoes, so it proves the hole without asserting it is closed).

---

## 12. Top-10 bugs / risks (severity, evidence, exact fix location)

| # | Sev | Finding | Evidence | Fix location |
|---|---|---|---|---|
| 1 | **CRITICAL** | Approved `shell_exec` is arbitrary code execution: default allowlist contains `cmd.exe`, plus `git.exe`/`cargo.exe` (hooks/build scripts) — the jail bounds only cwd, and argv is passed straight to the shell. The allowlist promise is void. | `shell.rs:61-72,94-103,189-216`; `side_effects.rs:283-295`; e2e test relies on `cmd /C` (`tests/suspension.rs:1041-1108`) | `shell.rs:66-72` — remove shell hosts and effect-capable tools; allow only programs that cannot execute other programs (or add a per-program arg allowlist). Treat `shell_exec` as "arbitrary code exec" in docs/UI. |
| 2 | **HIGH** | Data-directory escape in the tool bridge: `ToolsBridge` jail does not `deny_path(data_dir)`, while the engine jail does. Default `data/` is inside the workspace, so file tools can read the ledger and approved write can overwrite the SQLite DB/ledger. | `http.rs:259-262` vs `http.rs:347-351`; `tools_bridge.rs:53-80`; `config.rs:12,428-432,566-600`; `jail.rs:92-108` | `tools_bridge.rs:53-80` accept `data_dir` and `.deny_path(data_dir)` (and ledger dir); `http.rs:259-262` pass it. Add a test that `file_write("data/...")` and `file_read("data/ledger/...")` refuse. |
| 3 | **HIGH** | Root runs an unregistered second tool vocabulary (`spawn_superagent`, `spawn_remote_agent`, `spawn_daemon`, `read_memory_summary`, `propose_improvement`, `finish`) with no registry/tier/schema/approval/census coverage. Registry HyperAgent grants are dead. | `provider_executor.rs:39-53,567-666`; `execution.rs:4300-4811`; `tool.rs:115-173`; `census.rs:20-51` | `provider_executor.rs:583-666` + `execution.rs:4381-4811` — register the control tools (new ids/tier or `Control` class with explicit gating), call `resolve_call`/`plan_call`, publish `TOOL_CALL`/`TOOL_RESULT`, add census rows. |
| 4 | **HIGH** | Grant verification ignores TTL and identity: `now` is discarded, and the query matches only `(run, tool, args_sha256)`, so an `allow_once` grant can be executed after expiry or by a different call with the same args. | `approval.rs:924-982` (esp. `935`, `942-949`); docs claim timeout at `approval.rs:4-10,917-923`; decision TTL `approval.rs:397-412` | `approval.rs:924-982` — require `expires_at > now`, bind `approval_id`/`tool_call_id` in the lookup, and add an expiry test on `verify_grant`. |
| 5 | **HIGH** | Ledger-write failure is fail-open: the decision is committed, `cancel_approval` cannot cancel a decided row, and the engine later executes the `allow_once` grant on timeout without a ledger row. | `tools_bridge.rs:364-396`; `approval.rs:634-646`; `execution.rs:3447-3469` | `approval.rs:634-646` add a CAS from `allow_once` with `executed_at IS NULL` to `cancelled`; `tools_bridge.rs:387-394` use it; test the failure path. |
| 6 | **HIGH** | `finish` and `report_result` are advertised as task-terminal/report tools but only echo JSON text; the dispatcher has no branch for them, so `finish` cannot finish. | `executor.rs:88-117`; dispatcher `execution.rs:2652-2709`; descriptions `tool.rs:402-413` | `execution.rs:2693-2709` handle `finish` (apply terminal task state, emit result) and `report_result` (parent result row); or reclassify/remove them. Add effect tests. |
| 7 | **MEDIUM** | Permission classes contradict actual approval behavior: `spawn_agent` (Execute) never asks; `send_to_sibling` (Mutate) same-branch never asks; `ask_parent` (Read) writes DB/spends provider calls; `request_detached` (Control) creates control-plane principals with no human approval. Census does not cross-check class↔disposition. | `tool.rs:59-61,384-455`; `execution.rs:2652-2692,3603-3858,4931-5089`; `census.rs:38-51,166-181` | `tool.rs` classes + `census.rs:166-181` — add a build-invariant check (`requires_approval` ⇒ `approval_executor` or an explicit allowlisted loop-owned exception), and reclassify/approve `request_detached` creation. |
| 8 | **MEDIUM** | `web_fetch` consumes the grant before URL/SSRF validation, and DNS is resolved twice (pre-check and `reqwest` connect), leaving a rebinding window; a doomed URL burns consent. | `execution.rs:3378-3390` vs `2828-2913,2914-2931` | `execution.rs:3356-3390` — preflight/validate URL + resolved IPs before `verify_tool_grant`, pin the checked IPs via `ClientBuilder::resolve`/custom connector, verify peer address on connect. |
| 9 | **MEDIUM** | Census is self-referential: `PROMPT_CLAIMED_TOOL_IDS` is a literal, not parsed from `personas.json`/`.prompty`; persona/tier contradictions (Subagent `send_to_sibling`), dead ToolAgent/Hyper grants, and the root vocabulary are invisible. | `census.rs:20-33,99-133`; `personas.json:36` vs `tool.rs:498-507`; `execution.rs:2002,244-275` | `census.rs:20-33` — derive claims from loaded personas/prompt assets at boot/test; add "advertised ⇒ reachable+" and "class ⇒ approval lane" invariants; fail on dead grants. |
| 10 | **MEDIUM** | Path TOCTOU in the jail and a Job-Object assignment race: all checks are path-based before the actual `fs::*`, and the child is assigned to the job after spawn, so a grandchild can escape. | `jail.rs:164-196,216-245,290-373`; `side_effects.rs:406-410`; `shell.rs:211-223` | `jail.rs` — adopt `cap-std`/handle-relative open with reparse refusal (or document the residual risk as accepted only for reads); `shell.rs:211-223` — create the job first, spawn suspended, assign, resume; refuse if assignment fails before the child runs. |

Additional issues worth fixing (not in top ten):
- Approval replay after reconnect has **no args preview** — operator approves blind (`tools_bridge.rs:508-524,556-581`); persist a bounded preview or make replay fetch display args.
- `shell_exec` preview truncates each string at 160 chars / 512 total (`event.rs:294-313`), so long commands cannot be fully reviewed before consent.
- `allow_session` is unreachable from the wire (`approval.rs:62-80`) and, once programmatically set, has no expiry/revocation (`436-450`).
- `unknown_tool_name` yields `E_TOOL_NOT_ALLOWED` rather than `E_TOOL_NOT_FOUND` in the read-only executor (`executor.rs:58-70`; pinned by test `305-324`) — inconsistent taxonomy.
- `deny_path` silently ignores non-canonicalizable paths (`jail.rs:92-98`).
- `shell::execute` is public without the arg bounds that `preflight_side_effect` enforces (`shell.rs:173-178` vs `side_effects.rs:220-255`).
- The session-scoping unit test is tautological for arguments (`approval.rs:1353-1355`).
- Persona tool maps fall back to the **full tier** list for an unknown role (`provider_executor.rs:191-200`), not fail-closed to empty.
- `request_model_tool_approval` dedupes by `tool_call_id` + digest, ignoring the tool name (`tools_bridge.rs:459-470`) — a collision with identical JSON across two tools is theoretically possible.

---

## 13. Minimum tool expansion recommendation

**Do not expand the surface before fixing R1/R2/R3/R4/R6.** New tools added on top of `cmd.exe` + unprotected data dir + a parallel root vocabulary widen an already-porous boundary.

When the fixes are in, the **minimum useful expansion** is:

1. **Wire the existing advertised tools to real effects** — no new IDs needed: `finish` → terminal task/run result; `report_result` → durable parent-facing result. This is the largest capability gain per line of code.
2. **`file_grep`** (Read, no approval; args `{path?, pattern, max_results}`; jail-scoped, regex bound, result bound): the single highest-value read tool for coding agents; reuses `WorkspaceJail::resolve` + walker, no new risky primitive.
3. **`file_edit`** (Mutate, approval; args `{path, expected_sha256, old, new}` or unified patch; atomic `write_atomic`; stale-hash required): avoids full-file rewrites and makes approval previews meaningful; reuses the existing preflight/commit split.
4. **Register the root control tools** (`spawn_superagent`, `spawn_remote_agent`, `spawn_daemon`, `read_memory_summary`, `propose_improvement`) as first-class `ToolId`s with schemas, a dedicated HyperAgent tier row, census rows and executor dispositions — this is "expansion" in governance terms more than new capability.
5. **Add a census invariant test** that every advertised tool is (a) on a reachable tier, (b) has an executor branch with a matching class, and (c) appears in the persona/prompt claim set derived from assets. Fail the build otherwise.

**Do not add** more shell surface, network fetch variants, or write targets until R1 and R2 are closed. If a "command" tool is required, replace `shell_exec` with a **program+subcommand allowlist** (for example `git status|diff|log`, `cargo check|test`, no hooks, no `-c`) rather than an executable allowlist.

**Target matrix after expansion:** Worker/agent gains `file_grep` (Read) and `file_edit` (Mutate/approval); SuperAgent gains them except write; Subagent gains `file_grep` only; ToolAgent is either removed from the enum/census or made reachable with `file_read/list/grep/write/edit` — do not keep a 4-grant ghost tier.

---

## 14. Explicit gaps / incomplete analysis

- **No test was executed.** All pass/fail statements are [U]. The build could not be verified either.
- `execution.rs` read ranges are listed in §1.2; ~3,700 lines of that file (including recovery/snapshot/final-answer paths outside the tool loop) were not read. Claims about `finalize_root`, spawn-result handling beyond line 4059, and the tail of `execute_request_detached` are based on the read ranges.
- `provider_executor.rs` lines 280–429 and 880–1143 were not read; `stream_reply` internals and budget code are outside this lane.
- `ws.rs` was inspected only through grep hits (`678`, `1095-1108`); the WS command/event framing and client (web) approval UI were not read. UI wiring statements are backend-side only.
- `cli.rs` was scanned for fs/process/approval use; no direct fs/`Command` calls were found, but the whole 1,116-line file was not read. A CLI-based bypass cannot be ruled out from this scan alone.
- `audit.rs`, `cancel.rs`, `ledger.rs`, `health.rs`, `db_admin.rs`, `settings.rs`, `cli_support.rs` were only API-scanned (except the ranges noted); their internal correctness is not assessed beyond the interfaces used by the tool loop.
- `Cargo.toml` and the bench were not read; dependency/feature audit (e.g., `win32job`, `jsonschema` versions) is not covered.
- The web client (approval dialogs, nonce handling in the browser) is out of scope; only the event payloads and WS/HTTP handlers were read.
- No live runtime, provider, shell escape, or filesystem-race experiment was performed. All "would execute" statements are code-path deductions [V].

---

## 15. Report shape — Done / Not done / Unverified

### Done

| Item | Evidence |
|---|---|
| Enumerated every `hiverouter-tools/src/**` file plus bin/tests/bench, with read depth | §1.1 table; line counts from filesystem listing |
| Full read of registry, census, error, executor, jail, shell, side effects, approval, env safety, SHA-256, watcher, lib | §2–§10 citations; 12 files read end-to-end |
| Full read of `tools_bridge.rs`, `tool_loop.rs`, `prompts.rs`, `approval_wait.rs` | §6–§8 citations |
| Full read of all prompt assets (manifest, personas, 8 prompty files) | §7; `personas.json` line map |
| Built the complete tier × tool matrix and compared it to registry, personas/prompts and actual executors | §4 table |
| Assessed approval/UI wiring, data-dir escape, shell escape, and agent tool usability | §5–§9, §12 |
| Top-10 risks with severity and exact fix locations | §12 table |
| Minimum tool expansion recommendation | §13 |
| Coverage inventory including incomplete reads | §1, §14 |

### Not done

| Item | Reason |
|---|---|
| Running the test suites (`cargo test` for tools/core/provider) | Task is read-only; no build/output writes, and a live repo test run is outside the mandate. |
| Runtime verification of approval/SSRF/shell behavior | No server or provider calls permitted. |
| Reading `approval.rs`-adjacent `db.rs`/`store.rs` approval-table SQL | Out of lane size budget; approval SQL is centralized in `approval.rs` and was read. |
| Full `execution.rs`, `ws.rs`, `cli.rs`, `provider_executor.rs` | 20-minute lane budget; ranges listed in §1.2/§14. |
| Web UI approval dialog verification | Out of scope (backend lane). |
| `Cargo.toml` dependency/feature audit | Not read; no claim made beyond imports. |

### Unverified

| Claim | What would prove it |
|---|---|
| Every listed test passes | `cargo test -p hiverouter-tools -p hiverouter-core -p hiverouter-provider` with output captured |
| The suspension suite's 14 tests are green (header admits a historically RED shell test) | Run `tests/suspension.rs`; check `shell_exec_runs_only_the_allowlisted_program_after_approval` |
| `cmd.exe` indeed executes arbitrary commands through `shell_exec` | Reproduce in a sandbox: approve `shell_exec {"program":"cmd.exe","args":["/C","type C:\\..."]}`; observe file read |
| Data-dir read/write through `ToolsBridge` | Test `file_read("data/ledger/run-ledger.jsonl")` and approved `file_write("data/x")` against the production bridge |
| Ledger-failure fail-open | Inject a ledger write failure after resolve and observe later execution |
| Approval grant post-TTL execution | Decide an `allow_once`, advance clock past `expires_at`, call `verify_grant` |
| ToolAgent unreachable / root vocabulary bypass in production | Drive a run and capture the offered tool specs per task and root control calls |
| `web_fetch` DNS rebinding | Authoritative resolver test with a domain that changes answer between pre-check and connect |
| UI replay blind-approval | Inspect the browser's `APPROVAL_REQUEST` after reconnect: `args_preview` is absent (`tools_bridge.rs:508-524`) |
| No CLI-side fs/process bypass | Full read of `cli.rs` and a runtime trace of its commands |

---

*End of L2 report. Every code claim carries a path:line; every test outcome is explicitly unverified. This analysis did not modify source, config, tests, or any file other than this report.*
