# 04 — CAPABILITY audit (SEAT D)

Status: **AUDIT ONLY.** No product file was edited **by this seat**, no process was restarted or killed, no
provider was called, and no key was read. The only file written by this seat is this file.

- Owner: SEAT D (`/capability-auditor`).
- Date: 2026-11-07T23:10–23:40 America/Chicago (`2026-11-08T05:10–05:40Z`).
- Workspace: `C:\Users\Riley\Desktop\hiverouter-v1`.
- Report path: `docs/trace/orca-wave/04-capability.md`.
- Question: **what is claimed vs what is actually callable**, across UI, backend, wire contract and the
  keyless live runtime.

---

## 0. Method, snapshot and evidence classes

### 0.1 Constraints honoured

| Constraint | How it was honoured |
|---|---|
| Audit only | No source/test/config file was created or edited; the only write is this report. |
| No restarts | The live server was no longer running by 23:28; this seat did not stop or restart it. |
| No providers | Key is absent (`E_KEY_MISSING`); no provider call was made. Provider routes were probed only. |
| Key disabled | No `DEEPSEEK_API_KEY` was read, printed or used. |
| Read-only live probes | HTTP GETs; one WebSocket session that sent read-only `SETTINGS_STATUS` plus deliberately invalid ids (all refused before mutation). |

### 0.2 Pinned snapshot

HEAD is `240fe0fd5d9b5a0845e670da51e7d75bfb1e934a` (`chore: checkpoint hierarchy fixture and hide-finished
layout work`, 2026-11-07T23:10:02-06:00). **The tree moved during this audit.** `backend/http.rs` (23:33),
`backend/ws.rs` (23:30), `web/src/state/store.ts` (23:35), `web/src/flow/graph.ts` (23:31),
`AgentProfilesPanel.tsx` (23:34), `ApprovalOverlay.tsx` (23:34) and several cosmetic panels were edited by
concurrent seats. Two findings were rechecked and changed: the built-in profile delete button was fixed
(D1) and replay identity gained role/kind/label enrichment (§4.1). Line numbers below were re-verified at
23:42 against the hashes in this table; re-pin before applying any fix.

| File | SHA-256 at analysis |
|---|---|
| `backend/crates/hiverouter-tools/src/tool.rs` | `0DBD32E74A3F9AC094C634CA3920557FCB8FC22939972F3754D82885F74073F0` |
| `backend/crates/hiverouter-core/src/execution.rs` | `F02F032A99CAD91E593CCB0140423AB990112515C86F7F417D9345E945A53AC0` |
| `backend/crates/hiverouter-core/src/http.rs` | `A3E52E55AC387B39B8A7336D2A7572E3E12B5315F5AA31EB6C008ABA3EC431CC` |
| `backend/crates/hiverouter-core/src/ws.rs` | `6B64424229EDDB72EA63CB012408683197035E604FC054F8A937D5A5249005AA` |
| `backend/crates/hiverouter-core/src/db.rs` | read; migration v10 |
| `backend/crates/hiverouter-core/assets/prompts/personas.json` | `48C1004B55E67917A1FCBE110F0536509A1309504C9663EDCFD0BB90282020C8` |
| `web/src/state/store.ts` | `696C46BFC2F7A52B5B48CF7F50FF2F7DD8558C15AB5A3CC100D6F87E05A61D8A` |
| `web/src/flow/graph.ts` | `441800DD59E16FEF6348A23FB661487A097EFDEFC128F0E0B8334F967DE861A7` |
| `web/src/panels/AgentProfilesPanel.tsx` | `3250B5A6E9F551ABD3F07380B5F8E57459AE85E16434CA9A3139D0550FB06062` |

Line numbers are for the content hashed above, not for HEAD. The concurrent edits were re-read before this
report was finalised; where an edit fixed a finding it is called out explicitly (D1, §4.1 replay).

### 0.3 Evidence classes

- **CODE** — read directly from the pinned file, `file:line`.
- **TEST-SOURCE** — a test exists and pins the behaviour, but it was **not re-run** by this seat.
- **LIVE** — observed from the running keyless server before it exited; verbatim in §2.
- **DOC** — a README/contract/research claim; explicitly separated from runtime evidence.
- **UNVERIFIED** — no callable path or no live proof; listed in §7.

### 0.4 Version truth

The live server reports `version: 1.2.0-dev`; the README documents the sealed **1.1.0** release and states
the tree is an unreleased 1.2 pre-release (`README.md:5-7`). `docs/RELEASE.md:37-38` says autonomous model
tool-calling is **not** claimed in 1.1.0. Everything below describes the **1.2-dev working tree and its
keyless runtime**, not the sealed release. The live UI bundle served by the running binary was older than
the working tree (SEAT C recorded the same skew; `03-forge.md:49-51`), so UI verdicts are source/bundle
verdicts, not pixel proof.

---

## 1. Executive summary

The HMAS foundation is real but much smaller than the target model:

- **Callable now:** root HyperAgent task, SuperAgent/Worker tasks, role/persona prompts, depth/fan-out/
  agent caps, tool approvals for `file_write`/`shell_exec`, cross-branch message approval, role-level
  durable memory, durable A2A messages, `SETTINGS_STATUS`, provider model/balance proxies (key-gated),
  token/cache usage, and a read-only history/replay surface.
- **Not callable despite being in the model/UI/contract:** approved spawning, parent answers, roster/
  discovery, mute/blind, per-hive/branch memory, ownership switching, Remote/Watchdog agents, a Subagent
  card/tier, autonomy modes, a runtime model/thinking control, a readable cost/budget surface, and a
  scheduler.
- **The two P0 truth failures are both “silent capability”:** `spawn_agent` is declared `Execute` and
  approval-gated in the tool contract, but the runtime explicitly executes it with **no approval**
  (`execution.rs:1694-1702`); and `ask_parent` returns success after writing a message, but **no answer
  path exists** (`execution.rs:2427-2497`), while the shipped SuperAgent/Subagent prompts tell agents to
  use it when blocked.

### 1.1 Top 10 gaps (ranked)

| # | Sev | Gap | Evidence |
|---|---|---|---|
| 1 | **P0** | `spawn_agent` bypasses the approval gate even though it is `PermissionClass::Execute` and `requires_approval()` is true | `tool.rs:12-13,55-57,333-338`; bypass `execution.rs:1694-1702`; census labels it `loop_owned` `census.rs:42` |
| 2 | **P0** | `ask_parent` is write-only: no answer, no parent inbox, no deadline; a blocked child still “succeeds” | `execution.rs:2427-2497`; message table has no status/reply `db.rs:159-177`; prompt claim `superagent.prompty:17`, `subagent.prompty:17` |
| 3 | **P1** | No A2A roster/discovery; `send_to_sibling` needs an exact `TaskId` the model cannot learn; no roster command/route (live `ROSTER_GET` → `E_COMMAND_UNKNOWN`, `/roster` → 404) | `tool.rs:231-239`; `http.rs:514-531`; live §2.2/§2.1 |
| 4 | **P1** | Memory surface lies by omission: DB has 7 durable rows keyed `worker`, `/api/agents/memory` lists only personas and returns all-empty | live §2.1/§2.3; `http.rs:940-958`; `execution.rs:3027-3035` |
| 5 | **P1** | No per-run/per-hive/per-branch memory; recall is role-only, newest-8 across runs (cross-run bleed; `run_id`/`task_id` stored but unused on read) | `db.rs:179-197`; `store.rs:645-671`; requirement `OPERATOR-DECISIONS.md:63-68` |
| 6 | **P1** | Subagent is not a tier/card, and the UI’s mixed-case `"Subagent"` role bypasses the narrow persona allowlist, so it can spawn (contrary to the code comment and persona) | `execution.rs:108-142,151-163,550-558,871-879`; `store.ts:1903-1908`; `personas.json:31-39` |
| 7 | **P1** | No SuperAgent ownership model or switch; the UI displays “owns the sub-hive” with no owner/hive field behind it | `entities.rs:28-45`; `nodes.tsx:159-161`; `strings.ts:281`; requirement `OPERATOR-DECISIONS.md:41-42` |
| 8 | **P1** | No scheduler; dispatch is sequential and the advertised `E_SEMAPHORE_FULL` “scheduler retries” path is unreachable (only `try_acquire` in tests) | `execution.rs:1405-1499,1510-1560`; `budget.rs:408-430`; `client.rs:380,580`; `refusal.rs:128-130` |
| 9 | **P1** | Autonomy modes 1–4 absent; model/thinking are boot-env settings only; leaf thinking is not even reported by `SETTINGS_STATUS` | `config.rs:22-35`; `ws.rs:777-796`; no `autonom` match in code; requirement `OPERATOR-DECISIONS.md:19-20` |
| 10 | **P1** | Cost is enforced internally but has no read surface: `/api/runs/{id}/budget` 404 and 3/5 live runs failed `E_BUDGET_EXCEEDED` with no auditable cost report | live §2.1/§2.3; `budget.rs:217-221`; `provider_executor.rs:431-433` |

Additional P1: A2A **mute/blind/invisible** is absent entirely (`OPERATOR-DECISIONS.md:52`). The
built-in profile-delete false affordance was **fixed by a concurrent seat at 23:34** (D1); only the
non-contract `E_PROFILE_NOT_FOUND` refusal text remains (P2).

---

## 2. Live probe record (keyless, read-only)

### 2.1 HTTP

Environment: `dist\HiveMind\hiverouter.exe` (pid 15484), started 2026-11-07 23:10:13, bound
`127.0.0.1:8099`, `/health` `status=degraded`, `version=1.2.0-dev`, `database.schema_version=10`,
`graph_nodes=35`, `allowed_origins=2`, `deepseek_api_key` absent.

| Probe | Result (verbatim/derived) | Meaning |
|---|---|---|
| `GET /health` | `200`; `{"status":"degraded","version":"1.2.0-dev",...,"checks":[...deepseek_api_key ok:false, code:E_KEY_MISSING]}` | Server real; keyless |
| `GET /diagnose` | `200`; census `tool_count:9`, `refusal_count:65`, `tier_count:4`; caps `run_cost_cap_usd:"0.50"`, `token_budget_combined:400000`, `call_cap:40`; `side_effects_enabled:true` | Declared capability census |
| `GET /api/session` | `200` `{"token":"[redacted]","schema":"hiverouter.v1","last_seq":0,"ws":"/ws"}` | WS auth token issued |
| `GET /api/agents/profiles` | `200`, 1,901 B; prefix shows `chatbot`, `code-reviewer`, `content-writer`, `data-analyst`, `planner` (response truncated by probe at 300 chars) | Persona registry reachable; 7 total per `personas.json` |
| `GET /api/agents/memory` | `200`, 473 B; seven personas, **every `memories` array empty** | See §2.3: DB is not empty |
| `GET /api/projects` | `200`; one project `{"id":"01a1b85f...","name":"01a1b85f..."}` | Server name == id |
| `GET /api/history/runs` | `200`; 5 runs: 2 `completed`, 3 `failed` with `error_code:"E_BUDGET_EXCEEDED"` | Cost gate fired |
| `GET /api/history/runs/{id}/revisions` / `state?revision=0` / `messages` | `200` each | Read surfaces real |
| `GET /api/usage/runs/{id}` | `200` | Usage read surface real |
| `GET /api/agents/runs/{id}/messages` | `200`; empty mailbox for this run | A2A read route real |
| `GET /api/provider/models` | `503`; body `{"code":"E_KEY_MISSING",...}` (174 B) | Key-gated, named refusal |
| `GET /api/provider/balance` | `503`; body `{"code":"E_KEY_MISSING",...}` | Key-gated, named refusal |
| `GET /api/agents/runs/{id}/roster` | `404` | No roster |
| `GET /api/a2a/topics`, `/api/a2a/settings` | `404` | No A2A control plane |
| `GET /api/agents/remote` | `404` | No Remote agents |
| `GET /api/daemons`, `/api/agents/offices`, `/api/agents/audit` | `404` | No daemon/office/audit surface |
| `GET /api/approvals` | `404` | No approval read route |
| `GET /api/settings` | `404` | Settings are WS `SETTINGS_STATUS` only |
| `GET /api/runs/{id}/budget` | `404` | No budget/cost read route |
| `GET /api/memory/banks` | `404` | No memory bank surface |

### 2.2 WebSocket (raw client with a real `Origin`)

Sent: `hello`, then `SETTINGS_STATUS`, `SPAWN_SUB_HIVE` (random parent), `CANCEL_RUN` (random run),
`KILL_WORKER` (random task), `REWIRE_TASK` (random task), `ROSTER_GET`.

| Frame | Payload |
|---|---|
| `SERVER_HELLO` | `{"last_seq":4,"schema":"hiverouter.v1"}` |
| `ACK` | `{"command":"SETTINGS_STATUS",...}` |
| `SETTINGS_STATUS_RESULT` | `{"provider":"deepseek","model":"deepseek-flash","key_present":false,"thinking":"enabled","reasoning_effort":"high"}` |
| `ERROR` (spawn) | `E_TASK_NOT_FOUND` — refused before any child row (`execution.rs:1069-1074`) |
| `ERROR` (cancel) | `E_RUN_NOT_FOUND` |
| `ERROR` (kill) | `E_TASK_NOT_FOUND` |
| `ERROR` (rewire) | `E_TASK_NOT_FOUND` |
| `ERROR` (`ROSTER_GET`) | `E_COMMAND_UNKNOWN` — not in the 17-name command enum (`command.rs:48-105`) |

All mutating probes used non-existent ids, so they prove the refusal surface without mutating state.

### 2.3 Runtime database (read-only SQLite open)

DB: `C:\Users\Riley\AppData\Local\HiveMind\hiverouter.db` (schema 10).

| Query | Observed |
|---|---|
| `agent_memory` group by `agent_key` | **7 rows, all `agent_key = 'worker'`** (keys `task:<task-id>`; `run_id`/`task_id` columns populated) |
| `tasks` group by `role` | `planner` 5, `superagent` 3, `worker` 19; **no `subagent`, `remote`, `watchdog` or `tool` row** |
| `agent_messages` | 0 rows |
| `agent_profiles` | 0 rows |
| `runs` group by `status` | `completed` 2, `failed` 3 (`E_BUDGET_EXCEEDED`) |
| `run_usage` | 3 rows: 3,268 / 6,894 / 8,057 total tokens, cache hit/miss populated |

The key finding: `/api/agents/memory` reported **all seven personas empty** while the table held **seven
durable `worker` memories**. The route iterates personas only (`http.rs:940-958`), and `worker` is a tier,
not a persona id (`personas.json:2-66`).

### 2.4 Live-evidence limits

- The server **was no longer running mid-audit** (present at 23:16, gone by 23:28); this seat did not
  stop or restart it. SEAT C recorded the same exit (`03-forge.md:47`).
- No keyed run and no provider call, so approval TTL, spawn, cost settlement and streamed tool loops were
  not exercised live.
- `/` (the embedded UI) was not captured before the process exited; UI evidence is source + the built
  `web/dist` bundle, and the served bundle is known to be older than the tree.
- No screenshot or DOM claim is made.

---

## 3. Capability matrix

Verdicts: **CALLABLE** (real code path + live or test-source evidence), **PARTIAL** (works only under
conditions / read-only), **DISPLAY-ONLY** (value rendered, no action/backing), **MISSING** (no code path),
**BYPASSED** (declared gate not enforced), **FALSE** (surface reports a value/capability the backend
contradicts), **FALSE-AFFORDANCE** (control/claim cannot work).

### 3.1 Agent kinds and cards

| ID | Capability | Claimed by | Callable path (file:line) | Evidence | Verdict / P |
|---|---|---|---|---|---|
| K1 | HyperAgent root task | `AGENT-ARCHITECTURE-TRUTH.md:17` | `execution.rs:99-105,1332-1345` | Live DB 5 planner tasks; `publish_node_spawn` emits `hyperagent` | **CALLABLE** |
| K2 | SuperAgent tier + persona | `AGENT-ARCHITECTURE-TRUTH.md:18` | `tool.rs:72-81`; `personas.json:21-29`; `execution.rs:108-142` | Live DB 3 superagent tasks; test `run_lifecycle.rs:583-590` | **CALLABLE** |
| K3 | WorkerAgent tier | `AGENT-ARCHITECTURE-TRUTH.md:19` | `tool.rs:72-81`; `execution.rs:159-161` | Live DB 19 worker tasks | **CALLABLE** |
| K4 | ToolAgent tier/card | card renderer `nodes.tsx:196-224`; tier `tool.rs:80` | `task_agent_tier` maps role `tool` (`execution.rs:158`), but no production spawn path emits it: `validate_spawn_role` allows no `tool` role (`execution.rs:115-136`), planner prompt says worker/superagent (`provider_executor.rs:25-29`), template roles are template ids (`ws.rs:144-151`) | Live DB has **no tool task**; renderer is reachable only if a non-conforming planner returns `role:"tool"` | **PARTIAL / P2** |
| K5 | Subagent role/tier/card | `AGENT-ARCHITECTURE-TRUTH.md:20` (“not a separate tier”) | role accepted (`execution.rs:127-134`), but `task_agent_tier` maps it to **WorkerAgent** (`execution.rs:151-162`); no `subagent` in `NODE_KIND_VOCABULARY` (`event.rs:240`) or web `NODE_KINDS` (`contract.ts:213`); no renderer (`nodes.tsx:277-284`) | Live DB has no subagent row; UI can send role `"Subagent"` (`store.ts:1903-1908`), which renders as a worker card | **PARTIAL / P1** |
| K6 | Remote/Watchdog agents + cards | `OPERATOR-DECISIONS.md:31-33,39-40` | **No code:** zero matches for `RemoteAgent`, `remote_agent`, `Watchdog`, `watchdog`, `heartbeat`, `Daemon`, `daemon` in `backend/**/*.rs`; no route (`http.rs:514-531`); no kind (`event.rs:240`) | Live `/api/agents/remote` and `/api/daemons` → 404 | **MISSING / P1** |
| K7 | All-agent card coverage + replay identity | `OPERATOR-DECISIONS.md:31,83-84` | only 4 kinds + client-only `error` (`event.rs:240`; `contract.ts:213`); raw snapshot is topology-only (`graph.rs:16-27`), but the read projection now enriches `role`/`kind`/`label` from the run's task rows (`http.rs:700-740`) and both HTTP and WS replay use it (`http.rs:684`; `ws.rs:832-839`); the UI preserves kind if present (`graph.ts:301`) | Live DB has 3 roles (`planner`/`superagent`/`worker`); no remote/watchdog/subagent/tool kinds | **PARTIAL / P1** |

### 3.2 SuperAgent ownership / hierarchy

| ID | Capability | Claimed by | Callable path (file:line) | Evidence | Verdict / P |
|---|---|---|---|---|---|
| O1 | SuperAgent owns a hive/branch; ownership switchable | `OPERATOR-DECISIONS.md:41-42`; UI “owns the sub-hive” `strings.ts:281` | No owner/hive field on `TaskRecord` (`entities.rs:28-45`); no scope partition in `ScopedContext` (`context.rs:20-29`); no ownership/profile command in `command.rs:48-105` | Live `/diagnose` has 4 tiers but no ownership surface; no ownership route | **MISSING / P1** |
| O2 | Per-persona `max_children`/`max_depth` enforced | `personas.json:28-29`; `AGENT-ARCHITECTURE-TRUTH.md:18` | `persona_spawn_limits(&parent.role)` does **case-sensitive** map lookup (`execution.rs:550-558`); UI sends `"SuperAgent"` (`store.ts:1903-1908`); map key is `"superagent"` (`personas.json:22`) | Code-derived: UI-spawned `SuperAgent` bypasses persona lookup; lowercase model-spawned role would apply it | **BYPASSED / P1** |
| O3 | Subagent persona narrows tools (no spawn) | `AGENT-ARCHITECTURE-TRUTH.md:20`; `subagent.prompty:17`; `execution.rs:111-112` (“Subagents … cannot spawn”) | persona `allowed_tool_ids` excludes `spawn_agent` (`personas.json:36`), but context seeding is case-sensitive (`execution.rs:871-879`); empty scoped allowlist means **no narrowing** (`execution.rs:991-994`); Worker tier grants `spawn_agent` (`tool.rs:408-421`) | Code-derived: role `"Subagent"` gets full Worker toolset, including `spawn_agent` and `file_write`/`shell_exec` approval paths | **BYPASSED / P1** |
| O4 | SuperAgent “owns the sub-hive” display | UI `nodes.tsx:159-161`; `strings.ts:281` | No field backs the caption; `NodeEvent` carries only kind/parent (`event.rs:261-270`) | UI renders the caption on every SuperAgent card | **DISPLAY-ONLY / P1** |
| O5 | HyperAgent/SuperAgent spawn workers or subagents | `AGENT-ARCHITECTURE-TRUTH.md:18`; `execution.rs:108-142` | `validate_spawn_role` + `spawn_sub_hive`; tests `run_lifecycle.rs:583-590` | Live DB roles show planner→superagent→worker plans | **CALLABLE** |

### 3.3 Memory

| ID | Capability | Claimed by | Callable path (file:line) | Evidence | Verdict / P |
|---|---|---|---|---|---|
| M1 | Durable role-level memory write/read | `AGENT-ARCHITECTURE-TRUTH.md:28` | `store.rs:612-671`; write `execution.rs:3027-3035`; load `execution.rs:880-884` | Live DB **7 rows, all `worker`**; test `store.rs:1081-1095` | **CALLABLE** |
| M2 | Memory surface shows the durable memory | `AgentMemoryPanel.tsx:87-123`; `http.rs:940-958` | Route iterates **personas**, not role banks | Live: all 7 personas empty while 7 `worker` rows exist | **FALSE / P1** |
| M3 | Per-run / per-hive / per-branch memory | `OPERATOR-DECISIONS.md:63-68`; `AGENT-ARCHITECTURE-TRUTH.md:31` says not coded | `agent_memory(agent_key,key,value,run_id,task_id,created_at)` has no scope/hive/branch column (`db.rs:179-197`); recall filters by `agent_key` only and orders by time (`store.rs:649-671`) | Live DB shows all rows under one role key; no bank route (`/api/memory/banks` 404) | **MISSING / P1** |
| M4 | HyperAgent reads all child banks | `OPERATOR-DECISIONS.md:65-68` | Root loads only `planner` role memory (`execution.rs:880-884`); no aggregate read | Code-derived | **MISSING / P1** |
| M5 | Memory isolation/provenance | requirement `OPERATOR-DECISIONS.md:74-75` | `run_id`/`task_id` stored but **not returned** by the API; newest-8 across runs; no trust/as_of | Live `/api/agents/memory` returns only `{key,value}` | **PARTIAL / P1** |
| M6 | Memory clear | `AgentMemoryPanel.tsx:75-84`; `http.rs:960-975` | DELETE by role key, no confirmation, no receipt | Code only (not exercised) | **CALLABLE / P2** |

### 3.4 A2A: roster / answer / mute

| ID | Capability | Claimed by | Callable path (file:line) | Evidence | Verdict / P |
|---|---|---|---|---|---|
| A1 | `ask_parent` writes a durable message | `tool.rs:223-229`; `execution.rs:2427-2497` | insert + `AGENT_MESSAGE` event; per-run 32-message budget | Test `tool_loop_engine.rs:302-326`; live DB 0 messages | **CALLABLE (write only)** |
| A2 | Parent answer / delivery to the blocked child | `superagent.prompty:17`; `subagent.prompty:17`; `14-a2a-bus-roster.md:38-44,55-62` | No answer tool/command/route; `ask_parent` returns `Ok` immediately (`execution.rs:2475-2490`); table has no `status`/`in_reply_to` (`db.rs:159-177`) | Code-derived; no answer path in 17-command enum (`command.rs:48-105`) | **MISSING / P0** |
| A3 | `send_to_sibling` same-parent notice | `tool.rs:231-239`; `execution.rs:2519-2625` | requires exact `target_task_id`; same-parent writes immediately | Test covers cross-branch; code path real | **CALLABLE (if id known)** |
| A4 | Cross-branch message approval | `execution.rs:1705-1728`; `docs/CODEMAP/17-CLAIM-REGISTER.md:25` | Mutate-class → `suspend_for_approval`; test `suspension.rs:472` | TEST-SOURCE | **CALLABLE (test)** |
| A5 | Roster / discovery | `OPERATOR-DECISIONS.md:52-56`; `14-a2a-bus-roster.md:68-74,92-93` | No route (`http.rs:514-531`), no tool (`tool.rs:108-127`), no command | Live `/roster` 404; `ROSTER_GET` → `E_COMMAND_UNKNOWN` | **MISSING / P1** |
| A6 | Answer verb / operator reply | `14-a2a-bus-roster.md:38-44,91-96` | No `AGENT_MESSAGE_REPLY`; no send endpoint; no `answered_at` | Live no route; DB schema | **MISSING / P1** |
| A7 | Mute / blind / invisible A2A settings | `OPERATOR-DECISIONS.md:52`; `14-a2a-bus-roster.md:75` | No settings field/command/route; `SETTINGS_STATUS_RESULT` carries only provider/model/key/thinking/effort (`event.rs:635-650`) | Code search zero matches for mute settings | **MISSING / P1** |
| A8 | Live mailbox updates in the UI | `store.ts:1525-1527`; `AgentMessagesPanel.tsx:21-84` | `AGENT_MESSAGE` bumps `agentMessageVersion`; panel refetches | Code only; live DB 0 messages | **CALLABLE (code)** |
| A9 | Message status/reply/deadline columns | `14-a2a-bus-roster.md:89-96` | `agent_messages` has id/run/from/to/kind/text/created_at only (`db.rs:159-177`) | Code | **MISSING / P2** |

### 3.5 Approvals

| ID | Capability | Claimed by | Callable path (file:line) | Evidence | Verdict / P |
|---|---|---|---|---|---|
| P1 | Tool approval for `file_write`/`shell_exec` | `tool.rs:12-13`; `event.rs:478-523` | `requires_approval` → `suspend_for_approval` (`execution.rs:1731-1745`); nonce/args hash/TTL | Tests `suspension.rs:590`; live overlay not exercised | **CALLABLE (test)** |
| P2 | Cross-branch message approval | `docs/CODEMAP/17-CLAIM-REGISTER.md:25`; `execution.rs:1705-1728` | same gate for `send_to_sibling` | Test `suspension.rs:472` | **CALLABLE (test)** |
| P3 | Spawn approval | tool contract header `tool.rs:12-13`; `SpawnAgent` is `Execute` `tool.rs:333-338`; `requires_approval` true `tool.rs:55-57` | **Bypassed:** branch order executes spawn before the approval branch and says “No approval is consumed” (`execution.rs:1694-1702`) | No test covers spawn approval; census labels spawn `loop_owned` (`census.rs:42`) | **BYPASSED / P0** |
| P4 | Single-use nonce / replay / TTL | `refusal.rs:179-193`; `ws.rs:869-896` | approval gate consumes nonce once; TTL path `execution.rs:2010-2400` | Tests exist (not re-run); live pending replay code `ws.rs:567-598` | **CALLABLE (test)** |
| P5 | Pending approval replay after hello | `command.rs:4-24`; `ws.rs:567-598` | bounded batch of open approvals republished | Code; live none pending | **CALLABLE (code)** |
| P6 | Approval history / read route | recorded as session-only in `24-ui-implementation.md:310` | No HTTP route; only append-only JSONL ledger on disk | Live `/api/approvals` 404 | **MISSING / P1** |
| P7 | Redacted `args_preview` in the approval UI | `event.rs:504-522`; `ApprovalOverlay.tsx:262-268` | producer redacts and caps; overlay renders it | Code + unit tests in `event.rs:986-1100` (not re-run) | **CALLABLE (code)** |

### 3.6 Model / thinking / settings / autonomy

| ID | Capability | Claimed by | Callable path (file:line) | Evidence | Verdict / P |
|---|---|---|---|---|---|
| S1 | Provider/model/key status | `tools/runbook.md` §3 step 2; `SettingsPanel.tsx:64-78` | `SETTINGS_STATUS` → result (`ws.rs:777-796`) | **LIVE** verified (§2.2) | **CALLABLE** |
| S2 | Planner thinking/effort visible | `tools/runbook.md` §3 step 2 | result uses **planner** thinking/effort only (`ws.rs:780-791`) | LIVE `thinking:"enabled"`, `reasoning_effort:"high"` | **PARTIAL (read-only)** |
| S3 | Leaf thinking / planner max tokens visible | `config.rs:26-35` | leaf thinking exists in config (`config.rs:28-29,106-109`) but is not in `SETTINGS_STATUS_RESULT` (`event.rs:635-650`) | Code-derived | **MISSING / P2** |
| S4 | Runtime model/thinking change | `tools/runbook.md` §3 step 2 (env only; no runtime-control claim) | no command/route to mutate; settings are boot env only (`config.rs:77-131`) | Live `SETTINGS_STATUS` is read-only; no SET command | **MISSING / P1** |
| S5 | Autonomy modes 1–4 | `OPERATOR-DECISIONS.md:19-20` | **No code:** zero `autonom` matches in `backend/**/*.rs` or `web/src`; no mode field/command/setting | Code search | **MISSING / P1** |

### 3.7 Provider models / balance / cache / cost

| ID | Capability | Claimed by | Callable path (file:line) | Evidence | Verdict / P |
|---|---|---|---|---|---|
| G1 | Provider model catalogue | `AGENT-ARCHITECTURE-TRUTH.md:71-73` | `http.rs:1017-1037` proxies `client.list_models()` | LIVE keyless `503 E_KEY_MISSING`; keyed path not re-run | **CALLABLE (key-gated)** |
| G2 | Account balance | `AGENT-ARCHITECTURE-TRUTH.md:71-73` | `http.rs:1040-1060` | LIVE keyless `503 E_KEY_MISSING` | **CALLABLE (key-gated)** |
| G3 | Usage + cache counters | `ModelsPanel.tsx:174-188`; `http.rs:978-1010` | `run_usage` persists hit/miss; panel renders all six fields | Live DB rows show hit/miss; `/api/usage` 200 | **CALLABLE** |
| G4 | Cost settlement readable by the operator | not claimed as a route; internal `budget.rs:217-221` | `RunBudget::report` exists but the live run's budget is dropped on finish (`provider_executor.rs:431-433`); the CLI `provider.cost` is a calculator from supplied token counts, not the run report | Live `/api/runs/{id}/budget` 404; no cost in UI | **MISSING / P1** |
| G5 | Cost cap enforcement | `refusal.rs:152-160`; `budget.rs:235-261` | `preflight` refuses at cap | Live 3/5 runs `E_BUDGET_EXCEEDED`; **no way to audit why** (G4) | **CALLABLE / P1 truth** |
| G6 | Retries / 429 / timeout refusals | `provider/error.rs`; docs matrix | client retry policy, named refusals | TEST-SOURCE not re-run | **CALLABLE (test)** |

### 3.8 Scheduler / scale limits

| ID | Capability | Claimed by | Callable path (file:line) | Evidence | Verdict / P |
|---|---|---|---|---|---|
| Q1 | Depth 3 / fan-out 8 / 64 agents | `caps.rs:8-15`; `AGENT-ARCHITECTURE-TRUTH.md:33` | `Caps::check_spawn`, used by spawn and plan (`execution.rs:1091,1368`) | Tests `run_lifecycle.rs:284-318`; live not exercised | **CALLABLE (test)** |
| Q2 | 40-call / 12-turn caps | `tool_loop.rs:13-21`; `budget.rs:30-31` | `check_call_budget` + `RunBudget::preflight` | Tests `tool_loop_engine.rs:457-465`; live 3 runs stopped at budget | **CALLABLE (test)** |
| Q3 | Approval-wait / message budgets | `tool_loop.rs:21`; `execution.rs:70,2035,2459` | 8 waits; 32 messages | Code | **CALLABLE (code)** |
| Q4 | Scheduler / fair admission queue | `OPERATOR-DECISIONS.md:102`; refusal copy `refusal.rs:128-130` claims “the scheduler retries queued work” | **No scheduler:** leaf loop and queued-spawn drain are sequential (`execution.rs:1405-1436,1510-1560`); no queue retry | Code; live run has ≤6 tasks | **MISSING / P1** |
| Q5 | In-flight provider semaphore | `budget.rs:42-43` (8) | `client.rs:380,580` use **blocking** `acquire`; `try_acquire` (the only `E_SEMAPHORE_FULL` source) is used only in tests (`budget.rs:688-698`) | Code-derived: `E_SEMAPHORE_FULL` is unreachable via a live request | **PARTIAL / P2** |
| Q6 | 50–150 active / 300–600 total scale | `OPERATOR-DECISIONS.md:13-14` | caps 64/run, sequential dispatch | `AGENT-ARCHITECTURE-TRUTH.md:33,111` already refutes; live max 6 tasks/run | **REFUTED / P1** |
| Q7 | Budget/limit surface in UI | research requirement `24-ui-implementation.md:963` | no budget route (404); no caps in UI (only counts) | Live §2.1 | **MISSING / P2** |

---

## 4. Findings by requested area

### 4.1 Agent kinds and cards — Subagent / Remote / Watchdog are missing

**What is callable:** 4 `AgentTier` variants (`tool.rs:72-81`), 4 server node kinds
(`event.rs:240`), 4 web kinds + client-only `error` (`contract.ts:213`), and 5 renderers
(`nodes.tsx:277-284`). Live DB proves planner/superagent/worker tasks were created and executed.

**What is not callable:**

- **Subagent** exists only as a role/persona. `task_agent_tier` maps every unknown/subagent role to
  `WorkerAgent` (`execution.rs:151-162`), so the server emits `kind:"worker"`; there is no Subagent kind,
  renderer, memory scope or card. The UI can nevertheless send `role:"Subagent"` (`store.ts:1903-1908`).
- **Remote/Watchdog** has **zero backend or agent-UI code**. A code search over `backend/**/*.rs` for
  `RemoteAgent|remote_agent|Watchdog|watchdog|heartbeat|Daemon|daemon` returns zero matches; the only
  “watchdog” hits in `web/src` are the UI design-ledger strings (`manifest.ts:2-4,33,109`;
  `strings.ts:60,62`). Live `/api/agents/remote`, `/api/daemons`, `/api/agents/offices` are 404.
- **ToolAgent** has a tier and a card renderer, but no production path creates the role: `validate_spawn_role`
  accepts no `tool` role (`execution.rs:115-136`), the planner prompt says worker/superagent
  (`provider_executor.rs:25-29`), and template insertion uses template ids (`ws.rs:144-151`). It is
  reachable only if a non-conforming planner return includes `role:"tool"`; live DB has no tool task.
- **Replay identity was fixed during this audit (23:33):** the raw snapshot is topology-only
  (`graph.rs:16-27`), but `history_graph_value` now filters to the run's tasks and enriches
  `role`/`kind`/`label` from the task authority (`http.rs:700-740`); both the HTTP history read
  (`http.rs:684`) and the WS `HISTORY_SNAPSHOT` (`ws.rs:832-839`) use it, and `buildReplayTasks` keeps a
  supplied kind (`graph.ts:301`). No test covers this new projection (UNVERIFIED). It does **not** add a
  Subagent/Remote/Watchdog kind: `node_kind_for_role` still maps `subagent` to `worker` (`http.rs:742-749`).

**Claimed vs callable:** the operator requirement is explicit — “Every agent, including tiny
Watchdog/Remote agents, must have a UI card” (`OPERATOR-DECISIONS.md:31-33`). The carrier truth doc marks
Subagent as a role rather than a tier (`AGENT-ARCHITECTURE-TRUTH.md:20`), and the A2A research records
Remote/Watchdog as “Nothing” (`14-a2a-bus-roster.md:96`); `09-synthesis-and-goal.md:148` lists
“Remote/Watchdog agent — Not coded”. So this is a **requirement gap**, not a product lie: the current UI
never displays Remote/Watchdog cards.
The one genuine UI falsehood in this area is the Subagent-role tool narrowing bypass (O3): the prompt
calls it a “narrow worker” (`subagent.prompty:17`), but the mixed-case role gets the full Worker toolset.

### 4.2 SuperAgent ownership

There is **no ownership object**. `TaskRecord` has `id, run_id, parent_task_id, role, depth, spawn_seq,
state, rev` (`entities.rs:28-45`); `ScopedContext` has parent + facts + tool ids + output schema
(`context.rs:20-29`). There is no hive id, owner id, ownership mode, scope, switch command, or ownership
row. `SPAWN_SUB_HIVE`/`REWIRE_TASK` change parent links only.

The UI nevertheless renders “decomposes intent, owns the sub-hive” on every SuperAgent card
(`nodes.tsx:159-161`; `strings.ts:281`), and the shipped prompt says “Own this branch”
(`superagent.prompty:20`). Those are **display/prompt claims with no runtime authority**: a SuperAgent
cannot enumerate or act on an owned set; ownership is just `parent_task_id`.

The one enforced ownership-adjacent rule — per-persona spawn limits — is **case-sensitive**:
`persona_spawn_limits` and `seed_task_context` look up `task.role` verbatim (`execution.rs:550-558,871-879`)
while the UI sends `"SuperAgent"`/`"WorkerAgent"`/`"Subagent"` (`store.ts:1903-1908`) and the persona map
keys are lowercase (`personas.json:21-39`). The system prompt lookup lowercases first
(`provider_executor.rs:82-92`), so the prompt applies but the tool/limit narrowing does not — an
inconsistent capability set for the same task. This is a concrete **P1 bypass**, independent of whether
ownership is implemented.

### 4.3 Per-hive / branch memory

Schema: `agent_memory(agent_key, key, value, run_id, task_id, created_at)` with primary key
`(agent_key,key)` (`db.rs:179-197`). Writes use `task.role` as `agent_key` (`execution.rs:3027-3035`);
reads filter by `agent_key` only, order by `created_at DESC`, and limit 8 (`store.rs:645-671`). There is no
project/hive/branch/run filter, no trust, no `as_of_rev`, no provenance in the response, no promotion/purge,
and no aggregate read for HyperAgent.

Live proof of the surface gap:

- `GET /api/agents/memory` → 200, seven personas, **all arrays empty**.
- SQLite → **7 rows, all `agent_key='worker'`**, each with a real `run_id`/`task_id`.
- The route iterates personas (`http.rs:940-958`); `worker` is not a persona id.

Consequences: the operator is told “No durable memories for the boot profiles” while memory exists; a new
run’s worker can recall up to 8 memories written by **previous runs** because recall ignores `run_id`; and
the HyperAgent cannot read child banks at all. This contradicts the stated memory scopes
(`OPERATOR-DECISIONS.md:63-68`) and the “SuperAgents get their own banks” requirement
(`OPERATOR-DECISIONS.md:65`), and is honestly recorded as not coded in `AGENT-ARCHITECTURE-TRUTH.md:31`.

### 4.4 A2A roster / answer / mute

**What works:** `ask_parent` writes an `agent_messages` row and emits `AGENT_MESSAGE`
(`execution.rs:2427-2497`; test `tool_loop_engine.rs:302-326`); `send_to_sibling` writes a same-parent
message and cross-parent sends suspend for approval (`execution.rs:1705-1728,2519-2625`; test
`suspension.rs:472`); the read route exists (`http.rs:810-838`); the UI refreshes on the event
(`store.ts:1525-1527`; `AgentMessagesPanel.tsx:21-84`).

**What does not work:**

- **No answer path.** `ask_parent` returns `Ok` with a `message_id` and the child continues; no engine
  reader consumes the parent mailbox, no parent-answer turn exists, the table has no `status`,
  `in_reply_to`, `answered_at` or deadline (`db.rs:159-177`), and there is no `AGENT_MESSAGE_REPLY`
  command (17-name enum, `command.rs:48-105`). The shipped prompts still instruct “ask_parent when the
  branch needs authority or missing information” (`superagent.prompty:17`) and “ask_parent when blocked”
  (`subagent.prompty:17`) — an unenforced behavioural promise. The 14-a2a research calls this the honest
  P0 (`14-a2a-bus-roster.md:55-62,88-96`).
- **No roster/discovery.** There is no route (`http.rs:514-531`), tool (`tool.rs:108-127`), command or
  event. Live `/roster` 404; `ROSTER_GET` → `E_COMMAND_UNKNOWN`. `send_to_sibling` needs an exact
  `target_task_id` (`tool.rs:231-239`) that the model has no way to learn; the prompt carries only role and
  depth (`provider_executor.rs:444-450`). The tool is advertised in every Worker/Super allowlist
  (`tool.rs:397-421`), so this is a **usable-on-paper, unusable-in-practice** capability.
- **No mute/blind/invisible.** Zero code matches; the settings result has no A2A field
  (`event.rs:635-650`); requirement `OPERATOR-DECISIONS.md:52`.

Live DB has **0 agent_messages**, so even the write path has no live proof in this workspace; it is
TEST-SOURCE only.

### 4.5 Approvals / spawn approval

The approval system itself is real: nonce-bound, args-hash-bound, single-use, TTL, replay of pending
approvals after hello, and an append-only decision ledger (`event.rs:478-523`; `ws.rs:567-598,869-896`;
`ApprovalOverlay.tsx`). Tests cover `file_write` (`suspension.rs:590`) and cross-branch `send_to_sibling`
(`suspension.rs:472`). The live app had no pending approval and no key, so the flow was not exercised.

The contract says “Mutate / Execute / Network tools always require the approval gate” (`tool.rs:12-13`);
`SpawnAgent` is declared `PermissionClass::Execute` (`tool.rs:333-338`) and `requires_approval()` is true
(`tool.rs:55-57`). **The runtime contradicts it:** in the tool-turn branch, `spawn_agent` is handled before
the generic approval branch and the comment says “No approval is consumed: spawning is loop-owned, never a
workspace side effect” (`execution.rs:1694-1702`). The capability census even encodes the exception
(`spawn_agent → loop_owned`, `census.rs:42`). No test covers spawn approval (grep for `spawn_agent` in
`backend/crates/*/tests` returns only a persona fixture in `run_lifecycle.rs:602`).

Operator policy requires “No silent bypasses” (`OPERATOR-DECISIONS.md:21-22`). Either the gate must cover
spawn or the contract/census/header must stop claiming Execute+approval. Today a model can create and run
a child — and spend its provider budget — without the operator seeing a decision. **P0.**

Also missing: an approval **history/read** route (`/api/approvals` 404); only the on-disk JSONL ledger
exists. The UI shows pending approvals only.

### 4.6 Model / thinking / settings / autonomy

- `SETTINGS_STATUS` is real and live-verified: provider `deepseek`, model `deepseek-flash`, key absent,
  thinking `enabled`, effort `high` (`ws.rs:777-796`; live §2.2). The Settings panel displays these values
  read-only (`SettingsPanel.tsx:64-78`).
- The **leaf thinking** and both max-token caps exist only as boot env (`config.rs:26-35,118-121`) and are
  not reported; the result uses planner fields only (`ws.rs:780-791`).
- There is **no runtime mutation**: no command/route sets model, thinking, effort, caps or autonomy; the
  only path is process environment (`config.rs:77-131`), which requires a restart. This matches the runbook
  (`tools/runbook.md` §3 step 2) but not an operator-facing “settings” capability.
- **Autonomy modes 1–4 are entirely absent**: zero `autonom` matches in backend or web. The only trace is
  the requirement (`OPERATOR-DECISIONS.md:19-20`) and research (`10-gap-meta.md:265`). No mode field,
  permission bundle, persistence or UI.
- Chat is operator UI plus the fixed root surface; `AGENT-ARCHITECTURE-TRUTH.md:24` is honest that it is not
  a conversational agent. The aria label “HyperAgent hive chat core” (`strings.ts:77`) is a surface label,
  not a second agent; no capability finding beyond documentation precision.

### 4.7 Provider models / balance / cache / cost

- Models and balance are real proxy routes (`http.rs:1017-1060`) and correctly return a named
  `E_KEY_MISSING` 503 when keyless (live §2.1). With a key they call the documented DeepSeek endpoints.
- Usage and cache hit/miss are persisted (`db.rs:198-216`), returned (`http.rs:978-1010`) and rendered by
  `ModelsPanel.tsx:174-188`; live DB rows contain real hit/miss splits.
- **Cost is enforced but not readable.** `RunAccounting`/`RunBudget` computes Decimal cost and exposes
  `report()` (`budget.rs:217-221`; `accounting.rs`), but the adapter drops the per-run budget on
  `finish_run` (`provider_executor.rs:431-433`), no HTTP route exposes it (live `/api/runs/{id}/budget`
  404), and no UI renders cost. Three of five live runs failed `E_BUDGET_EXCEEDED`: only one of those has a
  persisted `run_usage` row and it records 3,268 total tokens; the other two have no usage row. The two
  completed runs record 6,894 and 8,057 tokens. With no cost report, the operator cannot audit whether the
  cap fired correctly. This is a **P1 truth/observability gap**, not a fake display.

### 4.8 Scheduler / scale limits

- The caps are real: depth 3, fan-out 8, 64 agents (`caps.rs:8-15`), 40 provider calls
  (`budget.rs:30-31`), 12 tool turns, 8 approval waits, 32 agent messages (`tool_loop.rs:13-21`;
  `execution.rs:70`). Live `/diagnose` exposes the cap values; the live runs stayed at ≤6 tasks.
- **There is no scheduler.** `dispatch_inner` executes leaves in a `for` loop and then drains the spawned
  queue sequentially (`execution.rs:1405-1436,1510-1560`). The `E_SEMAPHORE_FULL` refusal text says “the
  scheduler retries queued work” (`refusal.rs:128-130`) but no scheduler exists; the only callers of
  `try_acquire` are tests, while production uses blocking `acquire` (`client.rs:380,580`), so that refusal
  is effectively unreachable.
- **Scale is refuted, not hidden:** the operator target is 50–150 active / 300–600 total
  (`OPERATOR-DECISIONS.md:13-14,106`); the hard caps and sequential engine cannot reach it, and
  `AGENT-ARCHITECTURE-TRUTH.md:33,111` already says so.
- There is no cap/budget surface in the UI; `/api/runs/{id}/budget` is 404 and the HUD shows counts, not
  limits. The research requirement for seven budget meters (`24-ui-implementation.md:963`) is not met.

### 4.9 Capabilities displayed but not wired (UI truth defects)

| # | Display | Backing | Verdict / P |
|---|---|---|---|
| D1 | Built-in profile delete button | **Fixed during this audit (23:34):** `BUILTIN_IDS` now includes `superagent`/`subagent` (`AgentProfilesPanel.tsx:23-31`), so the delete button renders only for imported profiles (`:164-171`). Residual: DELETE of an absent row returns a non-contract `E_PROFILE_NOT_FOUND` (`http.rs:894-902`, absent from `refusal.rs`/`contract.ts:426-492`) and the hardcoded built-in list can drift | **FIXED / P2 residual** |
| D2 | Agent memory “No durable memories for the boot profiles” | 7 durable `worker` rows exist; route only lists personas (`http.rs:940-958`; live §2.3) | **FALSE / P1** |
| D3 | SuperAgent card meta `workers` + percentage | the percentage is `task.progress`, not a child count; server never sends progress, so it renders “NO METRICS” (`nodes.tsx:166-167`; `event.rs:247-271`) | **FALSE-LABEL / P2** |
| D4 | Routing table column `role` | renders `visualFor(task.state).label` — the state, not the role (`RoutingPanel.tsx:20,31`) | **FALSE-LABEL / P2** |
| D5 | “Re-parenting is only possible while a branch is not streaming” | no streaming/state check in `rewireBlocker` (`store.ts:704-721`) or `GraphAuthority::rewire` (`graph.rs:141-201`); UI gates only kind/run (`ControlBar.tsx:34-35`) | **FALSE-CLAIM / P1** |
| D6 | Library project name “HiveMind AI” | client constant (`store.ts:336-365`, `strings.ts:119`) while the server project name is its id (live `/api/projects`); `ProjectsPanel` shows the real row, Library does not | **PARTIAL / P2** |
| D7 | Settings header comment / old docs reference a code-diff surface | `openDiffModal`/`diffOpen` remain in the store (`store.ts:287,315-316,2227-2232`) with no component/control; `DIFF_*` is decision-gated | **DEAD CODE / P2** |
| D8 | Docs claim Data tree uses `GET /api/files` | actual route is WS `FILE_TREE_LIST`/`FILE_READ` (`ws.rs:849-868`); `docs/trace/ui-rebuild/11-control-wiring.md:37` is wrong | **DOC DEFECT / P2** |
| D9 | ToolAgent card exists | no production spawn path emits role `tool`; only a non-conforming planner can create one | **PARTIAL / P2** |
| D10 | Projects surface looks like a selectable project | it is read-only by design and says so; no create/rename route (`http.rs:514-531`, `ProjectsPanel.tsx:1-5`) | **HONEST READ-ONLY** |

---

## 5. Document claims vs runtime evidence

This section deliberately separates **current README/doc claims** from **research requirements** and from
**stale wave documents**. Stale docs are not treated as current product claims.

| Claim (source) | Runtime evidence | Status |
|---|---|---|
| Sealed release is 1.1.0; tree is unreleased 1.2 | Live `/health` says `1.2.0-dev`; `README.md:5-7` | **TRUE (boundary)** |
| 1.1.0 does not claim autonomous model tool-calling | `docs/RELEASE.md:37-38`; 1.2-dev code has the tool loop | **TRUE for 1.1.0 / not applicable to tree** |
| SuperAgent persona/template carries spawn/file/ask/sibling tools and max_children/max_depth | persona exists (`personas.json:21-29`), but limits/tool narrowing bypass on UI’s mixed-case role (`execution.rs:550-558,871-879`) | **TRUE-CODE / PARTIAL runtime** |
| Subagent is a role/persona, not a separate tier/card | `AGENT-ARCHITECTURE-TRUTH.md:20`; `execution.rs:151-162`; no kind | **TRUE** |
| Per-hive memory absent | `db.rs:179-197`; live surface | **TRUE (honest gap)** |
| Memory is role-level, 8 rows | `store.rs:645-671`; live 7 `worker` rows | **TRUE** |
| 300–600 concurrent agents impossible under caps | `caps.rs:8-15`; live max 6 tasks/run | **TRUE (refuted ambition)** |
| HMAS/AWN complete | none of ownership/roster/Remote/mute/scopes exist | **FALSE** (`docs/CODEMAP/17-CLAIM-REGISTER.md:27` already says CONTRADICTED) |
| `ask_parent`/`send_to_sibling` exist | write paths + tests; no answer/roster | **TRUE for write only** |
| Cross-branch approval gate done | code + test (`suspension.rs:472`) | **TRUE (test-source)** |
| “Five shipped profiles” | `personas.json` has **7** (planner, content-writer, superagent, subagent, data-analyst, code-reviewer, chatbot) | **STALE** (`docs/CODEMAP/18-HMAS-RESEARCH.md:140-141`) |
| “Cross-branch approval gate not done” / “cost is not settled” / “no clear UI” | code has the gate, internal settlement, and memory Clear | **STALE** (`18-HMAS-RESEARCH.md:147,154-158`) — internal contradiction at 154 vs 158 |
| “14 commands, 21 events, 40 refusal codes” | contract is **17 commands / 24 events / 65 refusals** (`command.rs:48-105`, `event.rs:19-97`, `refusal.rs:73-282`) | **STALE** (`CLAIM-REGISTER.csv` C-021; `FINAL-REPORT.md:21-22` is superseded) |
| Mailbox live event is dropped | store handles `AGENT_MESSAGE` (`store.ts:1525-1527`) | **STALE** (`ui-rebuild/04:396-400`) |
| Project id is random per reload | project id is persisted in localStorage (`store.ts:336-365`) | **STALE** (`ui-rebuild/04:450-463`) |
| `SPAWN_SUB_HIVE` hardcodes `SuperAgent` | role now derives from parent tier (`store.ts:1903-1908`) | **STALE** (`ui-rebuild/04:223-229`) |
| Models panel shows 3 of 6 usage fields / no refresh after run | it renders 6 fields and depends on `runStatus` (`ModelsPanel.tsx:105-188`) | **STALE** (`ui-rebuild/04:524-533`) |
| Data tree is `GET /api/files` | it is WS `FILE_TREE_LIST` (`ws.rs:849-858`) | **WRONG DOC** (`ui-rebuild/11:37`) |
| Operator requirements: Remote/Watchdog cards, ownership switch, A2A mute, memory scopes, autonomy, scheduler | no code for any of them (see §4) | **REQUIREMENT, NOT IMPLEMENTED** (`OPERATOR-DECISIONS.md:19-20,31-33,41-42,52,63-68,102`) |

---

## 6. P0/P1 gap register (no fix performed)

| ID | Sev | Gap | Evidence | Fix seam (for the owner) |
|---|---|---|---|---|
| **CAP-P0-1** | P0 | `spawn_agent` bypasses approval despite `Execute`+`requires_approval` | `tool.rs:12-13,55-57,333-338` vs `execution.rs:1694-1702`; `census.rs:42` | gate spawn in `execute_tool_turn` (or reclassify contract + census + header); add a RED test for spawn approval/denial |
| **CAP-P0-2** | P0 | `ask_parent` has no answer; blocked child still “succeeds” | `execution.rs:2427-2497`; `db.rs:159-177`; `superagent.prompty:17`; `subagent.prompty:17` | bounded parent-answer turn or operator answer; message lifecycle + status; test |
| **CAP-P1-1** | P1 | No roster; `send_to_sibling` target id unknowable | `tool.rs:231-239`; `http.rs:514-531`; live 404/`E_COMMAND_UNKNOWN` | read-only bounded roster projection + prompt ids/tool |
| **CAP-P1-2** | P1 | Memory surface hides 7 durable rows | live §2.1/§2.3; `http.rs:940-958` | list role banks too (or a bank route), with counts/provenance |
| **CAP-P1-3** | P1 | No per-hive/branch memory; role-only cross-run recall | `db.rs:179-197`; `store.rs:645-671` | scope columns + read policy; migration |
| **CAP-P1-4** | P1 | Subagent has no tier/card; mixed-case role bypasses persona tools/limits | `execution.rs:151-163,550-558,871-879`; `store.ts:1903-1908` | normalize role before persona lookup; add Subagent kind or remove the claim; test mixed case |
| **CAP-P1-5** | P1 | No SuperAgent ownership/switch; “owns the sub-hive” display only | `entities.rs:28-45`; `nodes.tsx:159-161`; `OPERATOR-DECISIONS.md:41-42` | owner/scope field + command or remove caption |
| **CAP-P1-6** | P1 | No scheduler; `E_SEMAPHORE_FULL` unreachable and refusal copy claims a scheduler | `execution.rs:1405-1499`; `budget.rs:408-430`; `client.rs:380,580`; `refusal.rs:128-130` | either implement queued admission or fix the refusal copy and census |
| **CAP-P1-7** | P1 | Autonomy modes absent; model/thinking runtime control absent; leaf thinking hidden | `config.rs:22-35`; `ws.rs:777-796`; no `autonom` code; `OPERATOR-DECISIONS.md:19-20` | settings model/mode + persistence + command |
| **CAP-P1-8** | P1 | Cost enforced but not readable; budget route 404 | `budget.rs:217-221`; `provider_executor.rs:431-433`; live 404 | expose persisted cost report + budget route/UI |
| **CAP-P1-9** | P1 | A2A mute/blind/invisible absent | `OPERATOR-DECISIONS.md:52`; `event.rs:635-650` | settings field + enforcement |
| **CAP-P1-10** | P1 | No approval history/read route | live `/api/approvals` 404; only JSONL ledger | read-only ledger endpoint |
| **CAP-P1-11** | P1 | “Rewire only while not streaming” is unenforced | `store.ts:704-721`; `graph.rs:141-201` | add state precondition or remove copy |
| **CAP-P2-1** | P2 | Routing `role` column shows state | `RoutingPanel.tsx:20,31` | relabel/render role |
| **CAP-P2-2** | P2 | SuperAgent `workers` metric is progress | `nodes.tsx:166-167` | derive child count or relabel |
| **CAP-P2-3** | P2 | Project name constant vs server id | `store.ts:336-365`; live `/api/projects` | adopt server name/id |
| **CAP-P2-4** | P2 | ToolAgent card has no deterministic producer | `execution.rs:115-136`; `provider_executor.rs:25-29` | add role path or drop the tier claim |
| **CAP-P2-5** | P2 | Replay identity enrichment (`history_graph_value`) has no test | `http.rs:700-740`; `ws.rs:832-839`; no matching test in `backend/crates/hiverouter-core/tests` | add a projection test (role/kind/label, run filtering) |
| **CAP-P2-6** | P2 | Stale docs/claim counts (14/21/40 vs 17/24/65; five profiles; cost not settled; mailbox dropped; random project) | §5 rows | docs-truth sweep |
| **CAP-P2-7** | P2 | Dead `diffOpen` state/action | `store.ts:287,315-316,2227-2232` | remove or wire decision-gated UI |
| **CAP-P2-8** | P2 | Non-contract `E_PROFILE_NOT_FOUND` from profile delete; hardcoded `BUILTIN_IDS` | `http.rs:894-902`; `AgentProfilesPanel.tsx:23-31` | use a frozen refusal code; derive built-ins from the server |

---

## 7. Unverified / not proven here

| Item | Why unverified | What would prove it |
|---|---|---|
| Approval suspension (approve/deny/timeout) at runtime | keyless app; no approval was pending; tests not re-run | one keyed run with `file_write`/`shell_exec` and a decision; or `cargo test -p hiverouter-core --test suspension` |
| Model-spawned child (`spawn_agent`) end-to-end | no keyed run; no test covers spawn as a model tool | keyed run whose leaf returns `spawn_agent`; assert no approval and child result |
| Profile import/delete runtime | server exited before mutation probes; not exercised | live POST/DELETE with a temp profile |
| Memory write during this audit | no run was submitted | keyed run; then `/api/agents/memory` + SQLite compare |
| Scale caps firing | live runs ≤6 tasks | a synthetic over-cap plan/spawn test (already test-source at `run_lifecycle.rs:284-318`) |
| UI rendering of current source | served bundle older than tree; `/` not captured | rebuild + browser capture on the current bundle |
| Replay identity enrichment (`history_graph_value`) | added by a concurrent seat during the audit; no test in `backend/crates/hiverouter-core/tests` | a projection test for run filtering + role/kind/label |
| Provider models/balance with a key | no key | keyed `GET /api/provider/models`/`balance` |
| Cost-cap correctness (`E_BUDGET_EXCEEDED` at <8k tokens) | no cost report/route | persist and expose the cost report; recompute the three failed runs |

---

## 8. Reproduction

HTTP (server was `127.0.0.1:8099`, keyless; do not restart if it is not running):

```powershell
curl.exe -s -i http://127.0.0.1:8099/health
curl.exe -s -i http://127.0.0.1:8099/diagnose
curl.exe -s -i http://127.0.0.1:8099/api/agents/memory
curl.exe -s -i http://127.0.0.1:8099/api/provider/models
curl.exe -s -o NUL -w "%{http_code}\n" http://127.0.0.1:8099/api/agents/runs/<run>/roster
```

WebSocket (read-only; requires the token from `/api/session` and `Origin` equal to the bound origin):

```text
ws://127.0.0.1:8099/ws?token=<token>&last_seq=0
hello {"type":"hello","last_seq":0}
COMMAND SETTINGS_STATUS {}
COMMAND ROSTER_GET {}          -> ERROR E_COMMAND_UNKNOWN
```

Database (read-only):

```powershell
python -c "import sqlite3; c=sqlite3.connect(r'file:C:\Users\Riley\AppData\Local\HiveMind\hiverouter.db?mode=ro',uri=True); print(c.execute('select agent_key,count(*) from agent_memory group by agent_key').fetchall())"
```

Code anchors: §3 matrix and §4/§6 tables give the exact `file:line` for every verdict. Tests cited as
TEST-SOURCE were **not re-run**; no test output is claimed.

