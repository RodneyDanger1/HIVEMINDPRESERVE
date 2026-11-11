# L4 — HTTP/WS API, Contract, Provider Adapter/Executor, Config, CLI, Frontend API Client

- **Lane:** 4 of 7 (Codebase Analyst Engineer 4/7)
- **Repository:** `C:\Users\Riley\Desktop\hiverouter-v1`
- **Mode:** READ-ONLY analysis. No source/config/test was modified; no server was started; no provider was called; no key material was read or printed.
- **Report file:** `docs/analysis/lanes/L4-api-provider.md` (the only file written).
- **Evidence rule:** every claim below is tagged:
  - **VERIFIED** — read directly in the cited file at the cited lines (static read).
  - **UNVERIFIED** — inferred, framework-default, or not executed/observed. Tests are cited by name, **not executed**; no runtime behavior is claimed as verified.
  - **PROPOSED** — a recommendation, not current code.
- **Line citations** are repo-relative `path:line` or `path:line-line`.

---

## 0. Executive summary

The backend exposes a **large, mostly read-only HTTP surface (62 method+path routes) plus 12 usable WebSocket commands**, a **frozen contract crate with 17 command names (14 frozen), 24 event names and 78 refusal codes**, a **DeepSeek/OpenAI-compatible provider executor with budget, retry, rate and streaming enforcement**, a **config layer driven by environment + four CLI flags**, and **two CLI implementations** (Rust `hiverouter-cli` and Node `tools/hivemind-cli.mjs`). VERIFIED at `backend/crates/hiverouter-core/src/http.rs:609-703`, `backend/crates/hiverouter-contract/src/command.rs:48-105`, `backend/crates/hiverouter-contract/src/event.rs:72-96`, `backend/crates/hiverouter-contract/src/refusal.rs:73-323`, `backend/crates/hiverouter-provider/src/client.rs:38-115`, `backend/crates/hiverouter-core/src/config.rs:485-527`, `backend/crates/hiverouter-tools/src/cli.rs:82-193`, `tools/hivemind-cli.mjs:62`.

The strongest problems are **not** missing UI endpoints — every URL the React UI calls exists on the backend (VERIFIED by comparing `web/src/**` fetches against `build_router`). They are:

1. **The Node CLI cannot perform its three HTTP mutations** because it never sends the session token the backend now requires (`tools/hivemind-cli.mjs:1048-1069`, `1747-1776` vs `http.rs:764-793`). **HIGH**
2. **The Node CLI is incompatible with the shipped launcher's launch-token gate** (`start-hivemind.ps1:40,113`; `http.rs:743-763,3611-3623`). **HIGH**
3. **Provider proxy GETs can pin a connection for up to 600 s × 3 attempts** with no server-side request timeout (`http.rs:3260-3323`; `client.rs:60-71,460-475`), and the UI calls them with an unbounded `fetch` (`ModelsPanel.tsx:75,106`). **HIGH**
4. **There is no HTTP surface for missions/runs/tasks/tools/approval resolution** — only WebSocket commands. HTTP-only agents, tests and the Rust CLI cannot start or control work. **HIGH**
5. **Any local process can mint the HTTP/WS token from `GET /api/session`** when the launch gate is off (loopback trust), then mutate profiles, memory, evolution and provider config (`http.rs:738-793,3594-3635`). **HIGH (local threat model)**

Full ranked list with fix locations: §10.

---

## 1. HTTP server: construction, middleware, limits

### 1.1 AppState and service wiring

- `AppState` holds SQLite (`db`), boot checks, `boot_id`, resolved `Config`, `EventBus`, `GraphAuthority`, `SessionAuth`, `RunEngine`, `RemoteStore`, `DaemonStore`, `HistoryReadModel`, `ToolsBridge`, and an optional `prompt_assets_error` (`http.rs:65-101`).
- Boot construction loads embedded prompt templates; a failure becomes a **required failed check** and a named refusal instead of an empty registry (`http.rs:103-128`; `prompt_assets_for` `http.rs:118-145`). `/health` then returns 503 (`http.rs:3551-3591`). **VERIFIED**
- Recovery settlement frames are re-published after the bus exists (`http.rs:384-398`). **VERIFIED**
- `serve()` uses `axum::serve` with `into_make_service_with_connect_info::<SocketAddr>()` so handlers can see the peer (`http.rs:3642-3653`, used by `api_session` and `ws_handler`). **VERIFIED**

### 1.2 Router

- `build_router` defines **all routes** and a static-UI fallback (`http.rs:604-704`). Full enumeration: §2.
- Route layers: `host_guard` and `mutation_auth` wrap every route including the fallback (`http.rs:701-702`). **VERIFIED**
- Unknown `/api/*` paths return the fallback JSON 404 `{error:"not found",schema:"hiverouter.v1"}` (`http.rs:700`; `static_ui.rs:46-50,74-83`). **VERIFIED**
- `HEAD`/`OPTIONS` on API paths pass the auth middleware without a token (`http.rs:764-766`). `HEAD` is then served by the GET handler if the framework maps it; **UNVERIFIED** whether axum 0.8 auto-routes HEAD to GET, but no explicit HEAD rejection exists. This is a minor bypass of the launch gate because the launch check only matches `Method::GET` (`http.rs:745-748`).

### 1.3 `host_guard` — Host identity (DNS-rebinding defense)

- Requires a `Host` header exactly equal to one of `127.0.0.1:<port>`, `localhost:<port>`, `[::1]:<port>` when the bound port is fixed; for ephemeral test ports (`port == 0`) any loopback authority is accepted (`http.rs:795-823`). Missing `Host` is rejected (`http.rs:801-803`).
- Rejection is **421 Misdirected Request** with a **partial refusal body** `{code,message,how_to_fix}` and **no `retryable`** (`http.rs:720-728`). This is a contract-shape defect: the frozen `ErrorPayload` always has four fields (`event.rs:151-160`). **VERIFIED**
- No test exercises `host_allowed`/`host_guard` in the repository tests found (only `launch_token_matches` is unit-tested, `http.rs:3708-3714`). **VERIFIED gap**

### 1.4 `mutation_auth` — launch token + session token

Two independent gates in one middleware (`http.rs:738-793`):

1. **Launch gate (read routes).** If `HIVEROUTER_LAUNCH_TOKEN` is set (`http.rs:19-26`), every `GET /api/...` **except `/api/session`** requires header `x-hivemind-launch: <token>`; otherwise **403** `E_ORIGIN_REJECTED` with `{code,message,how_to_fix}` and **no `retryable`** (`http.rs:743-763`). `/health`, `/diagnose`, `/ws`, and the SPA are not covered by this gate.
2. **Session gate (mutations).** All methods other than `GET`/`HEAD`/`OPTIONS` require `Authorization: Bearer <token>` **or** `x-hivemind-token: <token>` equal to the per-boot `SessionAuth` token (`http.rs:764-793`). Mismatch is **401** `E_TOKEN_INVALID` with the four-field body and `Cache-Control: no-store` (`http.rs:783-793`). `OPTIONS` is unconditionally passed (`http.rs:764-766`).

Token source and consequences:

- `AppState::new`/`with_graph` mint `SessionAuth::new(Uuid::now_v7().to_string(), config.allowed_origins)` — a **new token every boot** (`http.rs:408-412,453-463`; `ws.rs:45-86`).
- `GET /api/session` returns the token to any allowlisted Origin **or any loopback peer with no Origin** (`http.rs:3594-3635`). Therefore, with no launch token configured, any local process can obtain full mutation authority. **VERIFIED**
- The launch token is read **from the process environment on every request**, not cached (`http.rs:21-26,743`). **VERIFIED**
- No constant-time comparison is used (`launch_token_matches` uses `==`, `http.rs:35-40`; session comparison `supplied == Some(state.session.token())` at `http.rs:780`). Timing attack surface on loopback; **UNVERIFIED** practical exploitability. **PROPOSED:** constant-time compare.

### 1.5 CORS / Origin policy

- Only three places check `Origin`: `/api/session` (`http.rs:3599-3609`), `/ws` (`ws.rs:474-479`), and the provider proxy GETs (`http.rs:3260-3263,3287-3290` via `provider_origin_allowed` `http.rs:3440-3448`). **VERIFIED**
- Other HTTP routes never inspect `Origin`; mutation safety relies entirely on the unguessable per-boot token. Reads rely on loopback bind + `Host` guard + browser CORS; no `Access-Control-Allow-Origin` header was observed in any handler read for this report. **VERIFIED (observed); absence in unread helper ranges is UNVERIFIED.**
- `provider_origin_allowed` treats a **missing** `Origin` as allowed (`http.rs:3441`), so a non-browser local client can call the provider proxies while a browser from a foreign Origin cannot. **VERIFIED**

### 1.6 Explicit limits, timeouts and body caps

| Control | Value / behavior | Evidence |
|---|---|---|
| No router-wide request timeout | none configured | `http.rs:609-703` (VERIFIED absent) |
| No router-wide body limit layer | none configured; framework default applies (**UNVERIFIED** exact value) | `http.rs:609-703` |
| No HTTP rate limiting | none | `http.rs:609-703` |
| History run list | newest 200, no pagination | `http.rs:856-872` |
| History messages | newest 1 000, `truncated` flag, no pagination | `http.rs:1010-1042` |
| Agent messages per task | 1 000 then in-memory filters | `http.rs:1157-1167` |
| Pending approvals (no run filter) | 20 | `http.rs:1374-1388`; `ws.rs:665-667` |
| A2A pending | 100 | `http.rs:2230-2232` |
| Memory journal | `limit` 1–200, default 50 | `http.rs:1614-1618` |
| Remote audit | `limit` 1–200, default 100 | `http.rs:2050-2054` |
| Memory bank entries | `limit` 1–200, default 50 | `http.rs:3005-3009` |
| Memory summary | 8 entries, values cut at 512 chars | `http.rs:2331-2363` |
| Evolution summary/proposals scan | 256 entries | `http.rs:2379-2398,2535-2538,2733-2736` |
| Lessons projection | first 200 JSONL rows | `http.rs:2654-2659` |
| Profile import / delete key | id ≤ 128 chars | `http.rs:1246-1249`; `ws.rs:104` |
| Operator A2A answer | ≤ 2 000 chars, non-empty, actor forced `operator` | `http.rs:1063-1078` |
| Memory clear reason | ≤ 500 chars, `confirm:true` required | `http.rs:1476-1487` |
| Memory prune | `confirm:true`, `dry_run` defaults **true** | `http.rs:1535-1545` |
| Prompt (WS) | 1–8 000 chars | `command.rs:34-35,393-404` |
| File read (WS) | ≤ 1 MiB, jailed | `command.rs:36-37,429-434`; `ws.rs:1081-1090` |
| Provider request timeout | 600 s non-streaming; 600 s stream idle; connect 10 s | `client.rs:60-71` |
| Provider retries | ≤ 3 attempts, jittered backoff, `Retry-After` capped 60 s | `client.rs:9-14,460-475`; `budget.rs:47-53,366-425` |
| Provider rates | DeepSeek unlimited locally; external 20 req/min, max wait 10 s | `budget.rs:428-505` |
| Provider in-flight | 8 per client | `budget.rs:47-48,658-720`; `client.rs:53-57` |
| Provider budgets | $0.50/run, 400 000 combined tokens, 64 calls, 3 attempts | `budget.rs:25-53,80-110` |
| Spawn caps | depth 3, fan-out 8, 64 agents | `budget.rs:37-45,723-786` |

**Key risk:** `/api/provider/models` and `/api/provider/balance` inherit the 600 s client timeout and up to 3 attempts, and the server has **no** per-request timeout, so a stalled provider can hold an HTTP request for ~30 minutes (`http.rs:3264-3283,3304-3323`; `client.rs:60-71,460-475`). Each call also constructs a **new client**, so the external-endpoint 20 req/min governor is not shared across proxy calls (`http.rs:3264-3270,3304-3310`; `client.rs:82-92`). **VERIFIED (static); impact UNVERIFIED runtime.**

---

## 2. Route inventory and per-route contract

Legend for “Auth”: **H** = host guard only; **L** = launch gate applies to GET; **T** = session token required (mutation); **O** = additional Origin allowlist. Every route is also H. “no-store” means `Cache-Control: no-store`.

Refusal bodies are `{code,message,how_to_fix,retryable}` via `refusal_response` (`ws.rs:487-497`) unless a row says otherwise. Status mapping is handler-specific; the shared helper does not choose status.

### 2.1 System / session / static

| # | Method & path | Handler | Auth | Input → output | Status / refusals | Notes, tests |
|---|---|---|---|---|---|---|
| 1 | `GET /health` | `health` `http.rs:3537-3592` | H | none → `HealthResponse` (structs `http.rs:545-602`; payload built at `http.rs:3565-3585`): status `ok/degraded/failing`, version, schema, uptime, pid, boot_id, data_dir, workspace_root, checks, database, transport `{last_seq,graph_nodes,allowed_origins}`, prompt_assets | 200 if all required checks + DB version OK; 503 if a required check fails. Body has **no refusal envelope**. | Leaks absolute data/workspace paths; no `no-store`; key presence only via check detail (`checks.rs:149-166`). Test `http_boot.rs:630` (not run). |
| 2 | `GET /diagnose` | `diagnose` `http.rs:825-854` | H | none → `ToolsBridge::diagnostics()` + `side_effects_enabled`, `uptime_secs`, `pid`, `boot_id` | 200; 500 with four-field refusal on tool error (`http.rs:843-852`) | Tools-owned; no secrets. No dedicated test found. |
| 3 | `GET /api/runtime` | `runtime_status` `http.rs:3477-3535` | H, L | none → status busy/idle, active/latest run ids, task count, caps, provider booleans/model/wire/endpoint/billing/key_present/thinking/effort/max tokens, `side_effects_enabled`, `last_seq`, `graph_nodes` | 200; 503 `SERVICE_UNAVAILABLE` on store errors | `no-store`; `key_present` from boot checks, not live env (`http.rs:3493-3497`). UI `SwarmPanel.tsx:49`. No dedicated test. |
| 4 | `GET /api/session` | `api_session` `http.rs:3594-3636` | H, O, launch check | optional `Origin`; optional `x-hivemind-launch` → `{token,schema,last_seq,ws:"/ws"}` | 200; 403 `E_ORIGIN_REJECTED` (three-field body) | **Excluded from launch gate** (`http.rs:747`), but performs its own launch check (`http.rs:3611-3623`). No-store. Token in body. e2e `ws-contract.spec.ts:16`; no unit/integration test found. |
| 5 | `GET /ws` | `ws_handler` `ws.rs:462-485` | H, O, query token | query `token`, `last_seq`; first frame `hello` → upgrade | Pre-upgrade 403 `E_ORIGIN_REJECTED`, 401 `E_TOKEN_INVALID`; after upgrade events | §3. Tests `ws_transport.rs:47-513`. |
| 6 | `GET /` + non-API paths | `static_ui` `static_ui.rs:42-72` | H | SPA/embedded assets | 200; 405 non-GET/HEAD; 404 JSON for unknown asset/API path | Path traversal impossible by exact archive lookup (`static_ui.rs:53-59`); `/api` without slash falls through to index.html (`static_ui.rs:46-50,62-69`) — minor. Test `http_boot.rs:1059` (not run). |

### 2.2 History, messages, roster

| # | Method & path | Handler | Auth | Input → output | Status / refusals | Notes/tests |
|---|---|---|---|---|---|---|
| 7 | `GET /api/history/runs` | `history_runs` `http.rs:856-872` | H, L | none → `{runs:[{run_id,project_id,status,error_code,created_at}]}` | 200; 503 on read-model error | Fixed 200 rows; no cursor. Test `snapshot_history.rs` (not read/run). |
| 8 | `GET /api/history/runs/{run_id}/revisions` | `history_revisions` `http.rs:875-898` | H, L | path → `{run_id,latest_revision,revision_count,revisions[]}` | 404 `E_RUN_NOT_FOUND` (bad id or unknown); 404 other | `run_id` parsed as contract `RunId`; bad UUID → 404 not 400. |
| 9 | `GET /api/history/runs/{run_id}/state?revision=N` | `history_state` `http.rs:901-929` | H, L | path + required `revision` → run/status/graph projection (`history_graph_value` `http.rs:936-979`) | 400 `E_SCHEMA_INVALID` missing/bad revision; 404 `E_HISTORY_GAP`; 409 other; 404 bad run id | **Reads all task rows and filters graph fail-closed**; empty index yields empty graph, never unfiltered (`http.rs:941-949`). No-store absent on success. |
| 10 | `GET /api/history/runs/{run_id}/messages` | `history_messages` `http.rs:1000-1043` | H, L | path → `{run_id,messages[],truncated}` | 404 bad/unknown run; 503 store error | Newest 1 000. Tests `http_boot.rs:1326,1373` (not run). |
| 11 | `GET /api/agents/runs/{run_id}/messages?task_id=&from_task_id=&kind=` | `agent_messages` `http.rs:1106-1198` | H, L | filters `task_id`, `from_task_id`, `kind ∈ {ask_parent,send_to_sibling}` → messages with full A2A fields | 404 bad run/task; 400 bad task id/kind; 503 | 1 000 cap only when `task_id` present; run-wide list is unbounded by handler (`http.rs:1157-1160`). `no-store`. Test `http_boot.rs:1294`. |
| 12 | `POST /api/agents/runs/{run_id}/messages/{message_id}/answer` | `answer_agent_message` `http.rs:1049-1104` | H, L, T | `{answer:string, actor?:"operator"}` | 400 schema/empty/>2 000; 404 run/message; 409 `E_TASK_TERMINAL`; 500 other; 200 `{status:"answered"}` | CAS transition; rejects any actor other than `operator`; wakes A2A waiter (`http.rs:1070-1085`). `no-store`. Test `http_boot.rs:434`. |
| 13 | `GET /api/agents/runs/{run_id}/roster` | `agent_roster` `http.rs:1304-1352` | H, L | path → `{run_id,count,agents[]}` with class/rank/profile/ownership/kind/label/state/depth/rev | 404 bad/unknown run; 503 | Roster IDs are real `TaskId`s for A2A. Test `http_boot.rs:673`. |
| 14 | `GET /api/agents/runs/{run_id}/memory?scope=` | `agent_memory_for_run` `http.rs:3037-3086` | H, L | path + optional scope ≤128 → `{run_id,scope,count,memories[]}` | 400 empty/too-long scope; 404 bad run; 503 | Up to 200 rows. `no-store`. Test `http_boot.rs:717`. |
| 15 | `GET /api/agents/memory` | `agent_memory_list` `http.rs:3088-3106` | H, L | none → `{profiles:[{id,display_name,memories:[{key,value}]}]}` | 200; 503 on any store error | Only personas currently registered; up to 8 memories each (`http.rs:3091`). Test `http_boot.rs:1190`. |
| 16 | `DELETE /api/agents/memory/{agent_key}` | `clear_agent_memory` `http.rs:3108-3124` | H, L, T | path key ≤128 → `{agent_key,removed:bool}` | 400 empty/too-long; 503 | **Returns 200 `removed:false` for an unknown key** (`http.rs:3116-3120`) — no 404. UI `AgentMemoryPanel.tsx:189`. |

### 2.3 Memory banks, journal, context, evolution

| # | Method & path | Handler | Auth | Input → output | Status / refusals | Notes/tests |
|---|---|---|---|---|---|---|
| 17 | `GET /api/memory/banks?run_id=` | `memory_banks` `http.rs:2294-2329` | H, L | required `run_id` → banks | 400 missing/bad; 404 `E_RUN_NOT_FOUND` (**any** store error is labelled not-found at `http.rs:2304-2306`); 503 only after that | `no-store`. Test `http_boot.rs:325` (not run). |
| 18 | `GET /api/memory/summary?run_id=` | `memory_summary` `http.rs:2331-2364` | H, L | `run_id` → `raw_access:false`, `bounded_values:true`, `value_truncated_at:512`, up to 8 summaries | same 400/404/503 pattern | Values `.chars().take(512)` (`http.rs:2357`). |
| 19 | `GET /api/memory/banks/{bank_id}/entries?run_id=&limit=` | `memory_bank_entries` `http.rs:2991-3035` | H, L | required run + optional limit 1–200 → entries | 400; 404; 503 | Bank id not validated beyond non-empty path; scoped by run in SQL (`http.rs:3010-3013`). |
| 20 | `POST /api/memory/clear` | `clear_run_memory` `http.rs:1466-1522` | H, L, T | `{run_id,confirm:true,reason≤500}` | 400 schema; 404 run; 409 `E_RUN_ACTIVE`; 409 `E_APPROVAL_REQUIRED` if governance rows exist; 503; 200 receipt | Refuses to wipe `evolution_authorization:`/`evolution_apply:` rows (`http.rs:1495-1507`). `no-store`. Test `http_boot.rs:257` (not run). |
| 21 | `POST /api/memory/prune` | `prune_run_memory` `http.rs:1525-1569` | H, L, T | `{run_id,confirm:true,dry_run?}` | 400; 404; 409 active; 503; 200 `{dry_run,removed}` | `dry_run` defaults **true** (`http.rs:1539-1542`) — safe default. |
| 22 | `POST /api/memory/trust` | `promote_memory` `http.rs:1572-1601` | H, L, T | `{run_id,bank_id,entry_id,trust}` | 400 invalid trust/ids; 404 **`E_TASK_NOT_FOUND`** for missing entry (`http.rs:1598`); 200 `{updated:true,trust}` | Trust validated in store to `verified|authoritative` (`store.rs:1876-1881`). Wrong refusal code for a memory miss. `no-store`. |
| 23 | `GET /api/memory/journal?run_id=&limit=` | `memory_journal` `http.rs:1604-1637` | H, L | required run, limit 1–200 default 50 → actions | 400; 503 | `no-store`. Test `http_boot.rs:123`. |
| 24 | `GET /api/memory/journal/verify?run_id=` | `verify_memory_journal` `http.rs:1640-1663` | H, L | run → `{valid,checked,unchained}` | 400; 503 | Recomputes hash chain via store. `no-store`. No dedicated test found. |
| 25 | `GET /api/memory/context/{task_id}` | `task_memory_context` `http.rs:3189-3219` | H, L | task id → banks, entry_ids, rendered_chars, truncated, prompt_sha256, created_at | 404 bad/unknown; 503 | Test `http_boot.rs:164`. `no-store`. |
| 26 | `GET /api/agents/runs/{run_id}/proposals` | `run_evolution_proposals` `http.rs:2369-2517` | H, L | run → `hivemind.proposals.v1` with governance state, votes, `vote_path` | 400 bad run; 404 unknown; 503 | **`can_apply:false` hardcoded** and top-level `applied:false` (`http.rs:2501,2511`) even when authorized/apply is possible. Vote validity requires completed SuperAgents (`http.rs:2408-2420`). `no-store`. Test `proposal_authorization.rs` (not read). |
| 27 | `POST /api/agents/runs/{run_id}/proposals/{proposal_id}/authorize` | `authorize_evolution_proposal` `http.rs:2522-2644` | H, L, T | path run + proposal UUID; no body | 400 bad ids; 404 run/proposal (`E_TOOL_NOT_FOUND`); 409 `E_APPROVAL_REPLAY`, `E_APPROVAL_REQUIRED`, or invalid voter evidence; 200 authorized/idempotent | Requires **exactly four distinct completed SuperAgent approve votes, zero rejections** (`http.rs:2569-2600`). Records only; never applies code. |
| 28 | `POST .../proposals/{proposal_id}/apply` | `apply_evolution_lesson` `http.rs:2720-2852` | H, L, T | path only | 400; 404; 409 missing authorization/replay/hash mismatch; 422 unsafe text; 503; 200 applied | Writes `data_dir/evolution/LESSONS.jsonl` atomically (`http.rs:2704-2715`), records memory receipt, reloads lessons. Does **not** edit code. |
| 29 | `POST .../proposals/{proposal_id}/rollback` | `rollback_evolution_lesson` `http.rs:2856-2987` | H, L, T | path only | 400; 409 missing apply/replay; 503; 200 rolled back | Verifies per-line hash or whole-file hash before removing exactly one line (`http.rs:2894-2942`). |
| 30 | `GET /api/evolution/lessons` | `evolution_lessons` `http.rs:2648-2670` | H, L | none → `hivemind.evolution-lessons.v1`, first 200 JSONL rows | 200; 503 on read failure | Invalid JSONL lines are silently dropped (`http.rs:2654-2659`) — corruption can be masked. `no-store`. UI `LessonsPanel`. |
| 31 | `GET /api/a2a/pending` | `a2a_pending` `http.rs:2230-2252` | H, L | none → open questions (max 100) | 200; 503 | Cross-run operator mailbox. `no-store`. Test `http_boot.rs:229`. |
| 32 | `GET /api/a2a/messages?run_id=` | `a2a_messages` `http.rs:2254-2291` | H, L | required run → messages | 400; 404 `E_RUN_NOT_FOUND` for any store error (`http.rs:2264-2266`); 503 | Duplicates #11 without filters. `no-store`. |

### 2.4 Remote control plane

Shared JSON projections: `remote_json` `http.rs:1665-1681`, `device_json` 1683-1700, `grant_json` 1702-1713, `action_json` 1715-1731, `audit_json` 1733-1745. Shared status mapping `remote_refusal_response` `http.rs:1747-1760` (`RemoteUnknown|DeviceUnknown`→404; `RemoteCapability`→403; offline/stale/in-flight/approval→409; schema/transport→400; else 503).

| # | Method & path | Handler | Auth | Input → output | Status / refusals | Notes/tests |
|---|---|---|---|---|---|---|
| 33 | `GET /api/remotes` | `remote_list` `http.rs:1763-1785` | H, L | none → `{remotes:[{remote,device_count,grant_count,last_audit}],count}` | 200; mapped refusals | Per-remote N+1 queries; `unwrap_or_default()` hides device/grant/audit read errors (`http.rs:1770-1772`). `no-store`. |
| 34 | `POST /api/remotes` | `remote_create` `http.rs:1788-1834` | H, L, T | `{display_name,owner_kind?,owner_id?,transport_kind?,adapter_kind?,endpoint_ref?}` | 201 created; 400 schema; 400 `E_REMOTE_TRANSPORT` for non-`in_process` (`remote.rs:316-318`) | `capabilities_json` hardcoded `"[]"` (`http.rs:1826`), `credential_ref:None`, `created_by:"operator"` — an HTTP caller can never declare capabilities or agent identity. Test `http_boot.rs:808,980`. |
| 35 | `GET /api/remotes/{remote_id}` | `remote_detail` `http.rs:1867-1889` | H, L | path → remote + devices + grants + first 50 audit | 404; mapped | `no-store`. |
| 36 | `POST /api/remotes/{remote_id}/revoke` | `remote_revoke` `http.rs:1837-1849` | H, L, T | path → `{remote_id,state:"revoked"}` | 200; 404/409/503 mapped | `no-store`. |
| 37 | `POST /api/remotes/{remote_id}/heartbeat` | `remote_heartbeat` `http.rs:1852-1864` | H, L, T | path → remote JSON | 200; mapped | `no-store`. |
| 38 | `POST /api/remotes/{remote_id}/devices` | `remote_device_create` `http.rs:1892-1938` | H, L, T | `{class,name,room?,adapter_ref?,capabilities?,safety_class?}`; capabilities is a **string** of JSON | 201; 400 schema via `remote.rs:497-506`; mapped | **`capabilities` is never parsed/validated** before persistence (`http.rs:1907-1933`; `remote.rs:524`); invalid JSON later surfaces as `null` in `device_json` (`http.rs:1692-1694`). Safety class defaults light/switch→`reversible`, else `critical` (`http.rs:1913-1922`). |
| 39 | `POST /api/remotes/{remote_id}/grants` | `remote_grant_create` `http.rs:1941-1969` | H, L, T | `{device_id,capability_id,action,expires_at?}` | 201; 400 schema; mapped | `constraints_json:"{}"` hardcoded (`http.rs:1960`); `expires_at` unparsed string. |
| 40 | `GET /api/remotes/{remote_id}/audit?limit=` | `remote_audit_list` `http.rs:2045-2066` | H, L | limit 1–200 default 100 → `{remote_id,audit[]}` | 200; mapped | `no-store`. |
| 41 | `GET /api/devices/{device_id}` | `remote_device_get` `http.rs:1972-1985` | H, L | path → device JSON | 200; 404 `E_DEVICE_UNKNOWN`; mapped | `no-store`. |
| 42 | `POST /api/devices/{device_id}/actions` | `remote_action_create` `http.rs:1988-2018` | H, L, T | `{capability_id?,action,params?,expected?,idempotency_key?}` | 200 action; mapped (403 capability, 409 approval/in-flight/stale) | Default capability `power`; idempotency key generated if absent (`http.rs:2000-2004`); actor fixed `operator`. |
| 43 | `POST /api/device-actions/{action_id}/resolve` | `remote_action_resolve` `http.rs:2021-2042` | H, L, T | `{nonce,decision:"approve"|"deny"}` | 200; 400 `E_DEVICE_APPROVAL_INVALID`; mapped | No body `reason`; nonce single-use enforced in store. Test `http_boot.rs:808`. |

### 2.5 Daemons, supervisor

`daemon_json` `http.rs:2068-2088`.

| # | Method & path | Handler | Auth | Input → output | Status / refusals | Notes/tests |
|---|---|---|---|---|---|---|
| 44 | `GET /api/daemons` | `daemon_list` `http.rs:2090-2102` | H, L | none → `{daemons[],count}` | 200; 503 | `no-store`. Test `http_boot.rs:995`. |
| 45 | `POST /api/daemons` | `daemon_create` `http.rs:2104-2143` | H, L, T | `{display_name,owner_kind?,owner_id?,kind?,interval_secs?,job?}` | 201; 400 `E_SCHEMA_INVALID` | Store validates owner kind, kind, interval 5..86400 (`daemon.rs:148-157`). `created_by:"operator"`. `job_json` accepts arbitrary JSON. |
| 46 | `GET /api/daemons/{daemon_id}` | `daemon_detail` `http.rs:2145-2158` | H, L | path → daemon | 200; **404 `E_TASK_NOT_FOUND`** | Wrong refusal code: a daemon is explicitly not a task (`http.rs:88-89`). |
| 47 | `POST /api/daemons/{daemon_id}/switch` | `daemon_switch` `http.rs:2160-2172` | H, L, T | `{enabled:bool}` | 200; 400 schema; **404 whatever store refusal (likely `E_TASK_NOT_FOUND`)** | Wrong code class for unknown daemon. |
| 48 | `POST /api/daemons/{daemon_id}/retire` | `daemon_retire` `http.rs:2174-2186` | H, L, T | path → `{state:"retired"}` | 200; 404 `E_TASK_NOT_FOUND` | Same. `no-store`. |
| 49 | `GET /api/daemons/{daemon_id}/reports` | `daemon_reports` `http.rs:2188-2210` | H, L | path → 50 reports | 200; 503 | Detail JSON parse failure silently `null` (`http.rs:2201-2202`). `no-store`. |
| 50 | `GET /api/supervisor` | `supervisor_status` `http.rs:2212-2227` | H, L | none → lane timestamps/backoff/last error | 200; 503 | `no-store`. |

### 2.6 Approvals, templates, workspace, runs, profiles, usage, projects

| # | Method & path | Handler | Auth | Input → output | Status / refusals | Notes/tests |
|---|---|---|---|---|---|---|
| 51 | `GET /api/approvals?run_id=` | `pending_approvals` `http.rs:1356-1391` | H, L | optional run → `{run_id,count,approvals[]}` | 400 bad run; 404 unknown run; 503 | Read-only projection; does not consume nonces (`http.rs:1354-1355`). For a run: **all** pending events; without: 20 (`http.rs:1374-1377`). `no-store`. e2e `approval-flow.spec.ts:114-260`. |
| 52 | `GET /api/templates` | `templates` `http.rs:1394-1418` | H, L | none → template catalog with inputs, SPDX/source/fetched_at, `insertable` | 200 | `insertable` is a hardcoded 4-id allowlist (`http.rs:1402-1405`). No-store absent. Test `http_boot.rs:152`. |
| 53 | `GET /api/workspace` | `workspace` `http.rs:1422-1432` | H, L | none → `{data_dir,workspace_root,db_path}` | 200 | Discloses absolute paths; no `no-store`. UI DiagnosticsPanel. |
| 54 | `GET /api/runs/active` | `active_run` `http.rs:1436-1463` | H, L | none → `{active_run_id,latest_active_run}` | 200; 503 | Honest nulls. `no-store` absent. Test `http_boot.rs:673`. |
| 55 | `GET /api/agents/profiles` | `agent_profiles` `http.rs:1278-1298` | H, L | none → personas | 200 | Lists in-memory personas (8 embedded + imports, `http_boot.rs:560`). |
| 56 | `POST /api/agents/profiles/import` | `import_agent_profile` `http.rs:1200-1240` | H, L, T | `{profile:{...}}` or bare profile | 200; 400 schema/persona validation; 503 | Persists and installs persona (`http.rs:1214-1221`). Test `http_boot.rs:533,606`. |
| 57 | `DELETE /api/agents/profiles/{profile_id}` | `delete_agent_profile` `http.rs:1242-1276` | H, L, T | path id ≤128 | 200; 400; **404 three-field body** `{code,message}` (no fix/retryable, `http.rs:1255-1262`); 503 | Restores embedded profile if same id (`http.rs:1265-1270`). |
| 58 | `GET /api/usage/runs/{run_id}` | `run_usage` `http.rs:3221-3254` | H, L | path → usage incl. cache/reasoning tokens or `null` | 404; 503 | No-store absent. UI ModelsPanel. Test `http_boot.rs:1262`. |
| 59 | `GET /api/runs/{run_id}/detail` | `run_detail` `http.rs:3127-3186` | H, L | path → run, task count/state counts/tasks, usage, bank count | 404; 503 | UI RunDetailPanel. `no-store`. Test `http_boot.rs:197`. |
| 60 | `GET /api/projects` | `projects` `http.rs:3460-3475` | H, L | none → projects | 200; 503 | Read-only; **no POST** to create a project. Test `http_boot.rs:1244`. |
| 61 | `GET /api/provider/models` | `provider_models` `http.rs:3260-3284` | H, L, O | none → `{object,models,configured_model}` from provider | 200; 403 origin; mapped provider error 503/401/429/502 (`http.rs:3450-3458`) | Creates a new client per request; forwards key from env; 600 s ×3 timeout exposure. `no-store`. UI ModelsPanel. No route test found. |
| 62 | `GET /api/provider/balance` | `provider_balance` `http.rs:3287-3323` | H, L, O | none → `{supported,is_available,balances}` or `supported:false` for external endpoint | 200; 403; mapped provider errors | External endpoint honestly returns unsupported (`http.rs:3291-3302`). New client per request. `no-store`. UI ModelsPanel. No route test found. |
| 63 | `GET /api/provider/profile` | `provider_profile_get` `http.rs:3327-3364` | H, L | none → saved non-secret profile, path, active settings, `restart_required_after_save:true` | 200 | Whitelists only 5 fields from the file (`http.rs:3335-3344`). `no-store`. UI SettingsPanel. No test. |
| 64 | `POST /api/provider/profile` | `provider_profile_set` `http.rs:3367-3438` | H, L, T | `{base_url,model?,api_type?,wire_model?,name?}` | 200 saved; 400 schema; 403 `E_TOOL_NOT_ALLOWED` when side effects disabled; 500 **three-field custom body** on write error (`http.rs:3428-3436`) | Validates http(s) base URL, known model, api_type, wire-model requirements (`http.rs:3374-3410`). `name` unbounded. Writes to `provider_profile_path()` (`config.rs:254-330`). No test. |

**Route count:** 62 HTTP method+path rows (GET/POST/DELETE on the same path counted separately), plus `/ws` and the SPA fallback = 64 surfaces. VERIFIED against `http.rs:609-700` (60 `.route(...)` calls; three of them register both GET and POST: `/api/remotes`, `/api/daemons`, `/api/provider/profile`).

---

## 3. WebSocket transport (`ws.rs`) — contract, commands, refusals, limits

### 3.1 Handshake and auth

- `/ws` is registered as `get(ws_handler)` (`http.rs:699`). Handler: peer must be loopback (403 `E_ORIGIN_REJECTED` otherwise), `Origin` header must exactly match the allowlist (missing Origin is denied by `SessionAuth::is_origin_allowed`, `ws.rs:77-85`), and `?token=` must equal the boot token (401 `E_TOKEN_INVALID`) (`ws.rs:462-484`). **VERIFIED**
- Query type `WsQuery { token: Option<String>, last_seq: Option<u64> }` (`ws.rs:89-98`); the token is therefore in the URL. **VERIFIED**
- After upgrade the first client frame must be `{"type":"hello","last_seq":n|null}` within **10 s** (`HELLO_TIMEOUT`, `ws.rs:43,499-512`); non-hello text is answered with `ERROR E_BAD_FRAME` and the loop continues (`ws.rs:567-590`). On timeout an `ERROR E_BAD_FRAME` is sent and the socket closes (`ws.rs:507-511`). **VERIFIED**
- Events: server sends `SERVER_HELLO`, then replay frames when requested (`ws.rs:593-663`). Replay window is **256 events** (`bus.rs:18-20`); a gap sends `RESYNC_REQUIRED {expected_seq,last_seq}` then `SERVER_HELLO`, with **no graph snapshot** (comment says “Phase 9 API lands later”, stale — the history HTTP routes exist at `http.rs:614-617`) (`ws.rs:637-660`). **VERIFIED static; stale comment.**
- Live fan-out uses a 1 024-slot broadcast (`bus.rs:17-20`); a lagging client gets `RESYNC_REQUIRED` and the socket is closed rather than silently dropped (`ws.rs:542-559`). **VERIFIED**
- PING control frames get a protocol PONG and publish a `PONG` event (`ws.rs:533-536,1241-1244`); binary frames produce `ERROR E_BAD_FRAME` (`ws.rs:530-532`).
- No WS frame-size limit, per-connection rate limit, command timeout, or idle timeout is configured in `ws.rs`; only the hello timeout exists. **VERIFIED absent**; framework defaults **UNVERIFIED**.

### 3.2 Command dispatch

Dispatch entry: `handle_client_text` (`ws.rs:746-1129`). Decode order: JSON → `schema::parse_envelope` → `schema::decode_command` (which does envelope structure → frozen name → JSON Schema → typed deserialization → semantic validation, `command.rs:450-490`). Unknown/gated commands get `ERROR` with the named refusal and the command's `correlation_id` (`ws.rs:769-783`).

| Command | Payload (§) | Effect | ACK/events | Refusals |
|---|---|---|---|---|
| `SUBMIT_PROMPT` | `{text 1..8000, project_id}` `command.rs:166-173` | Key check first (`ws.rs:949-959`), then `RunEngine::submit_prompt` (`ws.rs:960-966`) | `ACK` + engine events | `E_KEY_MISSING`, prompt empty/too long, `E_RUN_ACTIVE`, engine refusals |
| `SPAWN_SUB_HIVE` | `{parent_task_id,role}` `command.rs:175-182` | `spawn_sub_hive` | `ACK` + `NODE_SPAWN` (engine) | caps/role/depth refusals |
| `REWIRE_TASK` | `{task_id,new_parent_id,expected_rev}` `command.rs:184-193` | Full hierarchy validation, graph CAS, task-row parent update, context switch (`ws.rs:786-942`) | `ACK` + `NODE_REWIRE` | `E_TASK_NOT_FOUND`, `E_SPAWN_DEPTH`, role/class refusals, `E_CYCLE`, context refusal |
| `CANCEL_RUN` | `{run_id}` | `cancel_run` | `ACK` | engine refusals |
| `KILL_WORKER` | `{task_id}` | `kill_worker` | `ACK` | `E_KILL_DENIED`/terminal |
| `APPROVAL_RESOLVE` | `{approval_id,nonce,decision,reason?}` `command.rs:209-221` | Ledgered single-use resolution (`ws.rs:1091-1118`) | `ACK` + `APPROVAL_RESOLVED` | `E_APPROVAL_*` |
| `SETTINGS_STATUS` | `{}` | Live provider/key settings projection | `ACK` + `SETTINGS_STATUS_RESULT` | none |
| `FILE_TREE_LIST` | `{path?}` | Jailed tree | `ACK` + `FILE_TREE_RESULT` | jail/path refusals |
| `FILE_READ` | `{path}` | Jailed ≤1 MiB read | `ACK` + `FILE_CONTENT` | jail/size/not found |
| `CHAT_PIN` | `{pinned}` | client-only | `ACK` only (`ws.rs:943-948`) | none |
| `TEMPLATE_INSERT` | `{template_id,project_id}` | Manifest-backed DAG insertion (`ws.rs:1019-1053`; `insert_template_dag` `ws.rs:202-397`) | `ACK` + `NODE_SPAWN` per task | `E_TEMPLATE_UNKNOWN`, `E_RUN_NOT_FOUND`, `E_RUN_TERMINAL`, caps |
| `HISTORY_REVISION_SET` | `{run_id,revision}` | History snapshot | `ACK` + `HISTORY_SNAPSHOT` | `E_HISTORY_GAP` etc. |
| `VIEWPORT_SET` | `{x,y,zoom 0.1..3.0}` | client-only | `ACK` | `E_ZOOM_RANGE` |
| `VIEWPORT_RECENTER` | `{}` | client-only | `ACK` | none |
| `DIFF_PROPOSE/APPLY/REJECT` | gated | never decoded | `ERROR E_DECISION_REQUIRED` | `command.rs:444-446,463-465` |

Post-hello replays:

- `publish_pending_approval_replays` re-publishes up to **20** pending approvals via `state.bus.publish` (`ws.rs:665-696`) on **every** hello.
- `publish_pending_a2a_replays` re-publishes up to **20** pending A2A messages via `bus.publish` (`ws.rs:709-744`).
- Because these go through the global bus, they (a) increment `last_seq`, (b) enter the 256-event replay window, and (c) are fanned out to **all** connected clients, not just the requester (`ws.rs:698-707`; `bus.rs:96-133`). Repeated `pullPendingApprovals()` (`client.ts:638-651`) can therefore churn the replay window and evict real events. **VERIFIED static; production impact depends on UI polling frequency (UNVERIFIED).**

### 3.3 WS tests present (not executed)

`backend/crates/hiverouter-core/tests/ws_transport.rs`: origin rejection `:47`, token refusal `:66`, two-client fan-out `:75`, hello replay/gap `:108`, invalid JSON keeps socket usable `:149`, rewire ack/node fan-out `:174`, cycle refusal `:266`, 1 000 rewires `:308`, run commands `:343`, settings `:391`, file tree/read `:413`, unavailable workspace `:484`, approval resolve `:513`. e2e `e2e/specs/live-security.spec.ts:251-374`, `ws-contract.spec.ts:16`, `approval-flow.spec.ts:114-260`.

**Test gaps:** no test for the 10 s hello timeout, `RESYNC_REQUIRED` on lag, `publish_pending_*` amplification, DIFF gating over the wire, template-insert invalid project/run-terminal paths, WS frame-size limits, or token timing.

---

## 4. `hiverouter-contract` — the frozen wire contract

- **Envelope:** `{schema,type,name,id,correlation_id,run_id?,task_id?,seq?,ts,payload}` (`envelope.rs:1-7,34-61`); `validate_basic` enforces schema id, non-nil ids, non-empty name, command-vs-event name validity, and forbids `seq` on commands (`envelope.rs:132-158`). `parse_envelope` is a serde parse (`schema.rs:26-28`). **VERIFIED**
- **Command names:** 17 total, 14 frozen (`CommandName::ALL` `command.rs:87-105`; `is_frozen` excludes `DIFF_*` `command.rs:140-148`). Wire names are SCREAMING_SNAKE (`command.rs:109-129`). `is_mutating` claims “stable command code used to classify a refusal/ack path” but returns `bool` (`command.rs:150-163`) — comment/API mismatch, cosmetic.
- **Command validation:** prompt bounds, zoom bounds, role non-empty, nonce non-empty + decision ∈ approve/deny, file path non-empty (`command.rs:393-448`); `MAX_PROMPT_CHARS=8000`, `MAX_FILE_BYTES=1 MiB`, `APPROVAL_DECISIONS`, `MIN/MAX_ZOOM` (`command.rs:34-43`). **VERIFIED**
- **Events:** 24 names (`event.rs:72-96,103-126`) — ACK, ERROR, SERVER_HELLO, RESYNC_REQUIRED, RUN_CREATED, RUN_STATUS, REPLY_DELTA, REPLY_DONE, NODE_SPAWN/DRAG/REWIRE/STATE, RUN_METRICS, APPROVAL_REQUEST/RESOLVED, SETTINGS_STATUS_RESULT, FILE_TREE_RESULT, FILE_CONTENT, HISTORY_SNAPSHOT, PING, PONG, TOOL_CALL, TOOL_RESULT, AGENT_MESSAGE. `ErrorPayload` is exactly `{code,message,how_to_fix,retryable}` (`event.rs:149-172`). **VERIFIED**
- **Refusals:** macro table of **78** codes with stable `E_*`, message, fix, retryable (`refusal.rs:11-71,73-323`); wire serde is the code string (`refusal.rs:325-337`); uniqueness test `refusal.rs:345-357`. Web mirror pins 78 codes and the parity test (`web/src/api/contract.ts:426-507`; `web/tests/refusalParity.test.ts:17-27`). Counts match 78 (`refusal.rs` table rows; `WEB contract` list). **VERIFIED static.**
- **Schemas:** generated from Rust structs with `schemars`; payload validation uses `jsonschema` with cached validators (`schema.rs:81-195`); golden snapshot test `golden_schemas.rs:15-110`; command acceptance matrix `schema.rs:197-…` and matrix test `ui_command_matrix.rs:15-113` (every frozen command has schema/ACK/refusal). **VERIFIED tests exist, not run.**
- **Approval-pull design note:** the contract deliberately adds **no** `APPROVAL_HYDRATE` command; pull reuses `hello` and server re-publishes approvals (`command.rs:4-24`). This is documented and consistent with `client.ts:63-85,617-651` — but it is the same mechanism that amplifies the global bus (§3.2).

---

## 5. Provider layer

### 5.1 Crate boundary (`backend/crates/hiverouter-provider`)

- Modules: `accounting`, `budget`, `client`, `dto`, `error`, `pricing`, `secret`, `tool_assembler` (`src/lib.rs:1-61`). One client, one refusal vocabulary; no second error table (`error.rs:1-7`). **VERIFIED**
- `ApiKey`: reads `DEEPSEEK_API_KEY` or a named env (`secret.rs:16-47`), zeroizes (`zeroize::Zeroizing`, `secret.rs:20-22,57-59`), `Debug` prints `ApiKey(***)` (`secret.rs:98-102`), never logs/returns the value; tests `secret.rs:108-118`. **VERIFIED** (no key was read by this analysis).
- `ClientConfig` defaults: base `https://api.deepseek.com` (`dto.rs:17`), connect 10 s, request 600 s, stream idle 600 s, retry default 3 attempts, in-flight 8, rate unlimited (`client.rs:38-72`); non-default base URL automatically gets the external 20 req/min policy (`client.rs:74-92`; `budget.rs:428-505`). **VERIFIED**
- HTTP behavior: bearer header built only at request time (`client.rs:529-545`); retry loop preflights budget and takes a rate permit **before every attempt including retries** (`client.rs:428-477`); 401/402 never retried (`client.rs:560-598`; tests `error_matrix.rs:141-208`); 429 retries then maps to `E_LLM_RATE_LIMIT` (`client.rs:480-494`); transport/timeouts map to `E_PROVIDER_TIMEOUT` (`client.rs:898-918`); malformed JSON → `E_PROVIDER_INVALID_RESPONSE` (`client.rs:496-527`); empty choices refused (`client.rs:519-525`). **VERIFIED static.**
- Streaming: SSE parser handles `: keep-alive`, `data:`, `[DONE]`; malformed chunk → invalid response (`client.rs:176-214`); stream retried only before the first event (`client.rs:600-680`); idle timeout on `response.chunk()` (`client.rs:699-729`); missing `[DONE]` → `E_PROVIDER_INCOMPLETE` (`client.rs:780-795`); usage merged from cumulative chunks without double counting (`client.rs:798-845`; test `rate_and_provenance.rs:363-395`). **VERIFIED static.**
- Budget (`budget.rs`): $0.50/run (`:25-26`), 400 000 combined tokens (`:28-29`), 64 calls including retries (`:31-35`), depth 3/fan-out 8/agents 64 (`:37-45`), 8 in-flight (`:47-48`), 3 attempts (`:50`), Retry-After ≤60 s (`:52-53`). Preflight refuses **before** a provider call with zero requests (`:268-315`; tests `refusal_matrix.rs:215-240`, `streaming.rs:243-264`). Usage absent → `E_USAGE_INCOMPLETE`, never zero (`budget.rs` record path; adapter test `provider_adapter.rs:281-303`; `refusal_matrix.rs:188-213`; `streaming.rs:304`). External endpoints keep token/call caps but not a USD cap (`budget.rs:92-110`; `rate_and_provenance.rs:261-327`). **VERIFIED static.**
- Tool assembler: fragments assembled in index order; missing id/invalid args map to named refusals (`tool_assembler.rs`; tests `tool_assembler.rs:78-154`, `tool_call_transport.rs:123-247`). **VERIFIED tests exist, not run.**
- `MAX_OUTPUT_TOKENS = 393_216` (`dto.rs:23`) is enforced by `ChatRequest::validate` (`dto.rs:561-580`) and config parsing (`config.rs:390-397`); the boundary test is `dto.rs:881-887`. The figure is **UNVERIFIED** as an official provider ceiling (the workspace claims register explicitly says it is unconfirmed). **VERIFIED that the unverified number is a hard limit in code.**

### 5.2 Executor (`backend/crates/hiverouter-core/src/provider_executor.rs`)

- `ProviderExecutorAdapter` fields: client, settings, per-run budgets (`Mutex<HashMap<RunId,RunBudget>>`), persona prompts, persona tool allowlists, pending usage (`provider_executor.rs:56-65`). `Debug` omits secrets (`:67-75`). **VERIFIED**
- Construction: `from_env(settings)` builds `ClientConfig::default().with_base_url(settings.base_url)` and reads the key from the selected env (`:83-87`; key env selection `config.rs:99-107`). `new(client,settings)` for tests (`:130-139`). **VERIFIED**
- Request building: accounting model + optional wire model override (`:89-94`); planner request uses thinking/effort/max tokens/JSON response format (`:98-128`); leaf request advertises tier-appropriate tools on the **first** request so continuation is reachable (`:475-499`); escalation answer is a tool-free JSON request, max 1 024 tokens, invalid JSON/tool call → `E_PROVIDER_INVALID_RESPONSE` (`:504-562`); root orchestration advertises only root tools (`spawn_superagent`, `spawn_remote_agent`, `spawn_daemon`, `read_memory_summary`, `propose_improvement`, `finish`) with `ToolChoice::Auto`, 1 024 tokens (`:564-670`); final root synthesis advertises **no** tools (`:672-698`); tool continuation replays the full transcript with A2A updates labelled data-not-instructions (`:700-795`). **VERIFIED**
- Streaming path: forwards deltas, cancels via `CancellationToken`, treats missing terminal event / transport failure as `E_PROVIDER_INCOMPLETE`, rejects `finish_reason` outside `stop|tool_calls`, rejects empty text+tool calls, returns assembled tool calls (`:255-427`). **VERIFIED static.**
- Accounted calls go through `complete_with_budget`/`stream_with_budget`; budget is held under the async mutex for the whole call (`:231-247,269-290`), meaning **one run serialises its provider calls on this mutex** — a throughput limit per run. **VERIFIED static; performance impact UNVERIFIED.**
- `take_pending_usage` drains planner/control usage for persistence (`:433-438`); `finish_run` removes the run budget (`:797-799`). No eviction of budgets for runs that never call `finish_run` — **UNVERIFIED** whether all terminal paths call it; potential memory leak in a long-lived server. **PROPOSED:** verify/lookup guard.
- Persona tool allowlists filter advertised tools (`:154-200`); hyphenated persona ids are preserved and are used for lookup (`:165-177`). Tests `provider_adapter.rs:123-245` assert tier/persona intersection (not run).
- **Data flow:** engine → `ProviderExecutor` trait (`plan`, `repair_plan`, `execute`, `continue_with_tools`, `answer_escalation`, `orchestrate_root`, `finalize_root`, `finish_run`, `key_available`, `take_pending_usage`) → client → DeepSeek/OpenAI endpoint; usage flows back through `StreamedReply.usage`/pending usage into store accounting. Trait/contract is defined in `execution.rs` (not read in this lane; cross-reference gap).

---

## 6. Configuration (`config.rs`) and launchers

### 6.1 Server config

- Constants: default bind `127.0.0.1:8080`, data dir `data`, DB `hiverouter.db`, model `deepseek-flash`, planner 16 384 / leaf 8 192 max tokens (`config.rs:9-20`). **VERIFIED**
- Env vars: `HIVEROUTER_PROVIDER`, `MODEL`, `THINKING`, `LEAF_THINKING`, `REASONING_EFFORT`, `MAX_OUTPUT_TOKENS`, `PLANNER_MAX_OUTPUT_TOKENS`, `BASE_URL`, `WIRE_MODEL`, `API_KEY`, `API_TYPE`, `PROVIDER_FILE`, `BIND`, `DATA_DIR`, `DB`, `WORKSPACE_ROOT`, `ALLOWED_ORIGINS`, `SIDE_EFFECTS` (`config.rs:23-46,421-426`; `from_args` reads bind/data/db/workspace `:486-492`). **VERIFIED**
- Flags: `--bind`, `--data-dir`, `--db`, `--workspace-root`, plus `--flag=value` forms; unknown flag → error; missing value → error (`config.rs:485-527`). `--help`/`--version` are handled in `main`, not `Config` (`main.rs:33-47`). **VERIFIED**
- Validation: bind parses and must be loopback (`config.rs:558-565`); data dir and DB parent created (`:566-585`); DB path must not be a directory (`:575-577`); provider settings errors are `E_SCHEMA_INVALID` (`:476-477`). **VERIFIED**
- Provider profile is a **non-secret** JSON read from `HIVEROUTER_PROVIDER_FILE`, else `%LOCALAPPDATA%\Hivemind\provider.json`-style path (`config.rs:307-330`), with BOM tolerance (`:630-637`). It is merged as fallback under env vars (`:129-235`); env always wins. `api_type` aliases: deepseek/deep, tokenlab(s) (`:332-340`). External endpoint requires `wire_model` (`:220-235`). **VERIFIED**
- Atomic profile write: write temp, copy old to `.bak`, remove old, rename temp; rollback on failure; Windows rename semantics handled (`config.rs:263-305`). **VERIFIED**
- Allowed origins: defaults `http://127.0.0.1:<port>` and `http://localhost:<port>`; ephemeral port 0 yields an **empty** list, so tests must append an origin (`config.rs:434-444,613-623`); `HIVEROUTER_ALLOWED_ORIGINS` appends comma-separated exact values. **VERIFIED**
- `side_effects_enabled` defaults true; false for `0/false/off/disabled/no` (`config.rs:586-592`). It gates only `POST /api/provider/profile` among HTTP handlers (`http.rs:3371-3373`); tool side effects are gated elsewhere (out of lane).
- **Test-only hazard:** `Config::resolve` **silently disables the provider** (`config.rs:534-545`). It is public and used by `AppState` tests; if production-ish code ever calls `resolve` instead of `from_args`, provider execution is disabled without a named refusal. **VERIFIED static.**
- Config tests: BOM profile, api_type/key-env selection, replace-safe write, defaults/loopback, non-loopback refusal, ephemeral port, missing value (`config.rs:629-738`). **VERIFIED tests exist, not run.**

### 6.2 Launchers vs config

- `tools/start-hivemind.ps1` defaults `Bind=127.0.0.1:8099` (not the binary default 8080) and `DataDir=%LOCALAPPDATA%\HiveMind` (`:3-6`), writes `launch.token`, sets `HIVEROUTER_LAUNCH_TOKEN` **only for the child process** (`:17-40`), decrypts the DPAPI provider key into the child env (`:59-81`), starts `target\release\hiverouter.exe --bind --data-dir --workspace-root` (`:84-86`), waits ≤20 s for `/health`, then opens `$base/?launch=$launchToken` (`:88-113`). **VERIFIED**
- The UI only reads `?launch=` from its own URL and monkey-patches `window.fetch` to attach `x-hivemind-launch` for **same-origin `/api/*`** (`web/src/main.tsx:10-40`). Cross-origin dev (`VITE_API_BASE=http://127.0.0.1:8080` from Vite on 5173) does **not** get the header → 403 under a launch-gated server. **VERIFIED static.**
- The Rust CLI defaults to `http://127.0.0.1:8080` for `health`/`diagnose` (`cli.rs:907-918`) while the launcher uses 8099. Running `hiverouter-cli health` after the launcher therefore probes the wrong port unless `--url` is passed. **VERIFIED static.**
- The Node CLI defaults to port **8099** (`hivemind-cli.mjs:26-27`) and starts the same binary; it does **not** set or send a launch token (no `HIVEROUTER_LAUNCH_TOKEN`/`launch.token` hits in the file). **VERIFIED static.**

---

## 7. CLI surfaces

### 7.1 Rust `hiverouter-cli` (`backend/crates/hiverouter-tools`)

- Entry is a thin wrapper around `cli::run_main(args)` (`src/bin/hiverouter.rs:1-12`). **VERIFIED**
- Output contract: machine JSON envelope `hiverouter.cli.v1` on stdout when non-TTY/`--json`/`--format json`; human lines on stderr; exit codes **0 ok, 2 usage, 3 named refusal, 4 internal** (`cli.rs:1-21,42-58,219-260`). **VERIFIED**
- Commands (single table, drives help + schema): `version`, `preflight`, `schema`, `tools.list`, `tools.describe`, `capabilities.census`, `provider.cost`, `health`, `diagnose`, `db.backup`, `db.restore` (`cli.rs:82-193`). Tier 0 local, tier 1 read HTTP, tier 2 mutating. **VERIFIED**
- **Doc/code contradiction:** the module doc says “v1 ships no mutating CLI command, so `--yes` is intentionally absent” (`cli.rs:16-18`), but `db.backup`/`db.restore` are declared `mutating:true` tier 2 with `--force` on restore (`cli.rs:173-192`). VERIFIED.
- HTTP reads: `health`/`diagnose` GET the configured URL with a bounded timeout (default 10 s), support `--dry-run` that prints the request without sending, and map transport failures to `E_WS_DISCONNECTED` (`cli.rs:907-944`). No auth header is sent; `/health` and `/diagnose` are outside the launch gate, so this works under the launcher. **VERIFIED**
- `preflight` probes Node ≥22.12.0, Rust ≥1.89.0, key presence boolean; `version_probe` kills a hung process after 5 s (`cli.rs:500-575`; `cli_support.rs:35-84`). **VERIFIED**
- `provider.cost` uses the same pricing/accounting crate and refuses unsourced prices/undefined peak windows (`cli.rs:718-800`; `provider_limits` `:946-956`). **VERIFIED static.**
- `db.backup` uses SQLite backup + hash receipt; `db.restore` verifies schema and refuses newer schemas (`cli.rs:824-905`; implementation in `db_admin.rs`, not read). **UNVERIFIED implementation details (not read).**
- No CLI command submits a prompt, cancels a run, kills a task, resolves an approval, lists runs, or executes a tool. **VERIFIED** (`COMMANDS` table `cli.rs:82-193`).

### 7.2 Node `tools/hivemind-cli.mjs` (agent CLI)

- 2 187 lines; commands include `start/stop/status/doctor/logs`, read commands (`runs`, `revisions`, `state`, `messages`, `agent-messages`, `roster`, `approvals`, `templates`, `workspace`, `active-run`, `profiles`, `session`, `memory`, `projects`, `models`, `balance`, `usage`, `health`, `diagnose`, `runtime`), WS mutations (`submit`, `cancel`, `kill`, `rewire`, `spawn-subhive`, `approve`, `deny`, `template`, `select-revision`, `settings`, `file-tree`, `file-read`, `chat-pin`, `viewport`, `recenter`) and three HTTP mutations (`profile-import`, `profile-delete`, `memory-clear`) (`hivemind-cli.mjs:1500-1788`). **VERIFIED static.**
- WS path is correct: gets `/api/session`, builds `ws://…/ws?token&last_seq`, sends `hello`, then frames (`:1343-1382`); it sets an explicit loopback `Origin` header in its own MiniWebSocket handshake because Node's WebSocket cannot (`:1091-1093,1157-1166`). **VERIFIED**
- **HTTP mutation bug (HIGH):** `executeHttpMutation` calls `httpJson` with no token (`:1747-1776`), and `httpJson` sends only `accept`, `origin`, optional `content-type` (`:1048-1057`). The backend requires `Authorization: Bearer` or `x-hivemind-token` for every non-GET (`http.rs:764-793`), so all three commands fail 401 `E_TOKEN_INVALID` against this backend. **VERIFIED static.**
- **Launch-token bug (HIGH):** `httpJson` sends no `x-hivemind-launch` (`:1048-1057`), and the CLI never reads the launcher's `launch.token`. Against a server started by `start-hivemind.ps1`, `GET /api/session` and every `/api/*` GET return 403 (`http.rs:743-763,3611-3623`). The Node CLI can only talk to a server it started itself without the gate. **VERIFIED static.**
- The Node CLI does not read or print provider keys (`:9`; `doctor` only reports key presence via `/health` check, `:1833-1861`). **VERIFIED static.**
- **Test gap:** no repository test drives `tools/hivemind-cli.mjs` against the Rust server; its correctness is UNVERIFIED at runtime.

---

## 8. Frontend API client contract (`web/src/api/**`) and UI usage

### 8.1 Client implementation

- `WsApiClient` bootstraps `GET /api/session` with an 8 s bounded fetch and an optional `x-hivemind-launch` header read from `?launch=` (`client.ts:269-315`; `http.ts:13-104`). Session shape is validated (token/schema/last_seq/ws all typed) or `E_SCHEMA_INVALID` (`client.ts:329-346`). **VERIFIED**
- It then opens `ws(s)://…/ws?token=…&last_seq=…`, sends `hello`, and only resolves `send()` on an `ACK` with matching correlation id; a typed `ERROR` rejects with the server refusal (`client.ts:350-368,466-514,548-594`). Command ACK timeout default **15 s**, handshake timeout **8 s**, HTTP timeout **8 s** (`client.ts:230-240`). **VERIFIED**
- Reconnect: partysocket reconnects (max 12, 0.5–8 s backoff, connection timeout 6 s, no message queue) (`client.ts:107-144`); after a close/error it schedules a full session refresh after 1.5 s because a server restart rotates the token (`client.ts:424-464`). **VERIFIED**
- Refusal construction only uses frozen codes; unknown server-only codes get a generic message, never invented copy (`refusals.ts:20-52`). HTTP failure helpers preserve server `{code,message,how_to_fix,retryable}` and fill missing fields from the local table; bodyless errors become `E_WS_DISCONNECTED` with route+status (`failure.ts:19-53`). **VERIFIED**
- `health.ts` probes `/health`, maps status to `ok|degraded|failing|unknown`, extracts check names, and bounds the request (`health.ts:28-76`). **VERIFIED**
- `contract.ts` mirrors the server: schema/version, prompt/zoom/preview constants (`:15-26`), 14 frozen commands, 12 server commands, 3 gated (`:100-160`), 24 event names (`:385-412`), 78 refusal codes (`:426-507`), envelope/event parsers and precondition validators (`:540-1110`). **VERIFIED**
- Dev mock: only constructed when `import.meta.env.DEV && VITE_UI_MOCK==='1'`, dynamically imported, throws in production (`index.ts:31-80`; `mock.ts:1-76`). Tests enforce no static import in the production entry and no mock marker in a production bundle (`web/tests/apiBoundary.test.ts:21-56`, `apiLifecycle.test.ts:203-244`). **VERIFIED static.** This is a legitimate test double under §7 boundaries; it is not reachable in production.

### 8.2 UI endpoint usage vs backend routes

Every path fetched by `web/src` exists in `build_router` (VERIFIED by path-by-path comparison). Main consumers:

- Session/WS/history: `client.ts`, `state/store.ts:472-491,1051-1367`.
- Remotes/daemons/supervisor: `RemotePanel.tsx:112,143,187-311`; `DaemonPanel.tsx:51-164`; `RemoteLane.tsx:35,75`.
- A2A: `A2AInboxPanel.tsx:77,142,202,221`; `AgentMessagesPanel.tsx:52,120-144`.
- Approvals: WS `APPROVAL_RESOLVE` via store; read-only `GET /api/approvals` (panel/overlay paths).
- Memory: `MemoryBanksPanel.tsx:321-443`; `AgentMemoryPanel.tsx:80,141,189`; `RunDetailPanel.tsx:103`.
- Profiles: `AgentProfilesPanel.tsx:55,112,136`.
- Proposals/lessons: `ProposalsPanel.tsx:143,187,220`; `LessonsPanel`.
- Provider: `ModelsPanel.tsx:75,106,172`; `SettingsPanel.tsx:140,173`.
- Runtime/workspace/active run: `SwarmPanel.tsx:49`; `DiagnosticsPanel.tsx:50-56`.

**UI contract defects found:**

1. **Unbounded provider fetches:** `ModelsPanel` uses raw `fetch` for `/api/provider/models` and `/api/provider/balance` with no `AbortSignal`/timeout (`ModelsPanel.tsx:75,106`), unlike `SettingsPanel` which uses `fetchWithBoundTimeout` (`SettingsPanel.tsx:140,173`). Combined with the 600 s server timeout, the Models panel can spin for minutes. **VERIFIED static.**
2. **Mutation auth duplicated per panel** with three different conventions: `x-hivemind-token` (`RemotePanel.tsx:172`, `DaemonPanel.tsx:95`, `AgentProfilesPanel.tsx:117,138`, `MemoryBanksPanel.tsx:321-443`, `AgentMemoryPanel.tsx:191`, `SettingsPanel.tsx:180`), `Authorization: Bearer` (`ProposalsPanel.tsx:194,227`), and token-less helpers elsewhere. Both header forms are accepted by the server (`http.rs:767-779`), but the duplication is a correctness drift risk. **VERIFIED static.**
3. **`main.tsx` fetch monkey-patch only covers same-origin `/api/*`** (`main.tsx:19-40`). Any panel using a cross-origin `VITE_API_BASE` loses the launch header. **VERIFIED static.**
4. **`ProposalsPanel` ignores the server's `can_apply`:** its `canApplyLesson` is `authorized && !lessonApplied` (`ProposalsPanel.tsx:114-116`), while the server hardcodes `can_apply:false` (`http.rs:2501`). The UI therefore shows Apply where the server says it cannot; the endpoint nevertheless accepts it. Two truths for the same state. **VERIFIED static.**

---

## 9. API truth vs UI vs docs

### 9.1 Docs

- `docs/contracts/UI-COMMANDS-v1.md` is the only contract doc in `docs/contracts` (with `ADAPTERS.md`) and documents the envelope, command/event vocabularies and refusals — **no HTTP route contract exists**. VERIFIED absence (directory listing) and doc content (grep hits limited to envelope/commands/events).
- `docs/plans/HIVEMIND-MASTER-FINISH-PLAN.md` is **stale on API security**:
  - `:34` “Authority/security | PARTIAL | Host guard + approvals; no `/api/*` token auth …” — contradicted by `mutation_auth` (`http.rs:738-793`) and its tests (`http_boot.rs:495-531`). VERIFIED contradiction.
  - `:87` “API auth absent” and `:171` “`/api/*` profile/memory mutations are unauthenticated; `/api/session` hands the WS token to any loopback peer; provider routes accept missing Origin” — first clause false now; the loopback-token and missing-Origin clauses are still true (`http.rs:3602-3607`, `3441`). VERIFIED mixed.
  - `:256` later says “every non-GET `/api/*` route requires … token” — internally inconsistent with `:34/:87/:171`. VERIFIED.
  - `:97` says four authority tiers; code has five (`hiverouter-tools` tier enum, outside this lane) — cross-lane, flagged only as doc drift.
- There is **no documented list of HTTP endpoints** anywhere; the router is the only source of truth (`http.rs:609-700`). This is a documentation gap for agents/CLI implementers. **PROPOSED:** add `docs/contracts/HTTP-API-v1.md` generated from route tests.

### 9.2 UI ↔ API

- No UI call targets a non-existent HTTP route (VERIFIED by comparison). This is stronger than the docs.
- The UI cannot start a run via HTTP; it uses `SUBMIT_PROMPT` over WS (`state/store.ts` send path; `ws.rs:949-967`). That is consistent with the design, but it means any non-browser consumer needs the WS command set.
- The UI's `GET /api/session` requires `?launch=` only under the launcher; the launcher opens it with the parameter (`start-hivemind.ps1:113`). **VERIFIED static.**
- UI panels read `/api/workspace` and `/health`, which expose absolute paths and host checks without `no-store` (`http.rs:1422-1432,3537-3592`). Low-risk local disclosure.

### 9.3 Missing endpoints needed for mission/tool/task UX

The finish plan describes a mission/task/tool operator experience, but the API is **read-mostly + WS commands**. Missing (PROPOSED):

1. `POST /api/runs` (or `/api/missions`) to submit a prompt over HTTP; no HTTP run creation exists. Current only `WS SUBMIT_PROMPT` (`ws.rs:949-967`) and no CLI command.
2. `POST /api/runs/{run_id}/cancel`, `POST /api/tasks/{task_id}/kill`, `POST /api/tasks/{task_id}/rewire`, `POST /api/runs/{run_id}/tasks` (spawn) — all are WS-only (`ws.rs:969-992`).
3. `POST /api/approvals/{approval_id}/resolve` — approval resolution is WS-only (`ws.rs:1091-1118`); HTTP exposes only the read projection (`GET /api/approvals`).
4. `GET /api/tools`, `GET /api/tools/{tool_id}`, `POST /api/tools/{tool_id}/invoke` (approval-gated job) — no HTTP tool registry or invocation; the only catalog is CLI-local (`cli.rs:617-675`) and the model tool path is internal.
5. `GET /api/tasks/{task_id}` and `GET /api/tasks?run_id=&state=` — task detail exists only inside `run_detail`/`roster` (`http.rs:1304-1352,3127-3186`).
6. `GET /api/events?after_seq=&run_id=` — event replay is WS-only and capped at 256 in memory (`bus.rs:18-20`); no durable journal/replay endpoint, which blocks CLI/test/agent observation.
7. `GET /api/missions` / project write endpoints — `GET /api/projects` is read-only (`http.rs:3460-3475`); no create/rename/archive.
8. Pagination/cursors for history, approvals, A2A and memory (fixed windows in §1.6).
9. Scoped auth/actor identity: current token is binary (full authority) and `/api/session` hands it to any loopback peer; no read-only token, no actor, no per-route scope. `RemoteStore` records `created_by:"operator"` hardcoded from HTTP (`http.rs:1827,2012-2013`).
10. Explicit refusal codes for daemon/memory misses (`E_DAEMON_UNKNOWN`, `E_MEMORY_ENTRY_NOT_FOUND`) instead of reusing `E_TASK_NOT_FOUND` (`http.rs:1598,2155,2170,2184`).

---

## 10. Top 10 bugs / risks (ranked) with fix location

| # | Severity | Finding | Evidence | Fix location |
|---|---|---|---|---|
| 1 | **HIGH** | Node CLI's `profile-import`, `profile-delete`, `memory-clear` never send the session token, so they always 401 against the current backend. | `tools/hivemind-cli.mjs:1048-1069,1747-1776`; server gate `http.rs:764-793` | Add `Authorization`/`x-hivemind-token` in `httpJson` after `wsSession()`/`/api/session`; or route these commands over WS. |
| 2 | **HIGH** | Node CLI cannot use a launcher-started server: no `x-hivemind-launch`, while `start-hivemind.ps1` always sets a launch token and opens the UI with `?launch=`. | `tools/hivemind-cli.mjs:1048-1057,1343-1347`; `start-hivemind.ps1:17-40,113`; `http.rs:743-763,3611-3623` | Read `%LOCALAPPDATA%\HiveMind\launch.token` (or a CLI flag) and send the header; add an integration test starting the server with a launch token. |
| 3 | **HIGH** | Provider proxy GETs can occupy a request for ~30 min (600 s × 3 attempts) with no router timeout; each call builds a fresh client so the external 20 rpm governor is not shared; UI Models panel uses unbounded `fetch`. | `http.rs:3260-3323,3450-3458`; `client.rs:60-71,460-475,82-92`; `ModelsPanel.tsx:75,106` | Use a short-timeout shared provider client (`with_request_timeout(10s)`, shared rate governor), cache models/balance, add `tower_http::timeout` to the router. |
| 4 | **HIGH** | No HTTP API for run/mission/task/tool creation or control; only WS commands. HTTP-only agents, Rust CLI and tests cannot start or stop work. | `http.rs:609-700`; `ws.rs:785-1128`; `cli.rs:82-193` | Add §9.3 endpoints (or a documented WS-only guarantee + a first-class WS client in every CLI). |
| 5 | **HIGH (local security)** | With no launch token, `GET /api/session` gives the full-authority token to any loopback process (and loopback `Origin`-less peer); every mutation is then driveable. | `http.rs:3594-3635,738-793`; `ws.rs:480-482` | Require launch token by default when the server is started by a launcher; add read-only/operator token scopes and audit the actor; constant-time compare. |
| 6 | **MEDIUM** | Refusal-shape inconsistency: several errors omit `retryable`/`how_to_fix`, and some reuse unrelated codes. | `http.rs:720-728` (3 fields), `1255-1262` (2 fields), `3428-3436` (3 fields), `3614-3622` (3 fields); `ErrorPayload` 4 fields `event.rs:151-160`; wrong codes `http.rs:1598,2155,2170,2184`; DB errors mislabelled 404 at `http.rs:2304-2306,2264-2266,2341-2343` | Route every error through `refusal_response`/`remote_refusal_response`; add `E_DAEMON_UNKNOWN`/`E_MEMORY_ENTRY_NOT_FOUND`; map DB-busy to 503. |
| 7 | **MEDIUM** | WS hello replay publishes approvals/A2A through the global bus, incrementing `last_seq`, evicting the 256-event replay window and fanning duplicates to every client. | `ws.rs:665-744,698-707`; `bus.rs:18-20,96-133`; client pull `client.ts:638-651` | Send these frames directly on the requesting socket (`send_event_direct`) and skip `bus.publish`; or add a dedicated pull response. |
| 8 | **MEDIUM** | Fixed-window reads with no pagination: runs 200, messages newest 1 000, approvals 20, A2A pending 100, proposals scan 256. Long missions silently lose history in every client. | `http.rs:856-872,1010-1042,1374-1388,2230-2232,2379-2398` | Add `limit`/`before`/`cursor` and return `next_cursor`; update UI and Node CLI. |
| 9 | **MEDIUM** | Evolution proposals report `can_apply:false` and `applied:false` unconditionally while the apply route can succeed; the UI ignores `can_apply` and enables Apply from `authorized && !lessonApplied`. | `http.rs:2501-2513`; `ProposalsPanel.tsx:114-116`; apply `http.rs:2745-2851` | Compute `can_apply = authorized && !applied && !rolled_back && hash matches`; make UI use it; add a route test. |
| 10 | **MEDIUM** | Remote device `capabilities` is persisted unvalidated and surfaces as JSON `null` on parse failure; daemon `detail`/`switch`/`retire` return `E_TASK_NOT_FOUND` for a non-task resource. | `http.rs:1907-1933,1683-1699`; `remote.rs:497-528`; `http.rs:2145-2185` | Parse capabilities at the HTTP boundary and refuse `E_SCHEMA_INVALID`; add `E_DAEMON_UNKNOWN` and use it consistently. |

**Additional risks (below top-10):**

- `MAX_OUTPUT_TOKENS = 393_216` is a hard validation limit for an **UNVERIFIED** official ceiling (`dto.rs:23,881-887`; `config.rs:390-397`). The workspace claims register explicitly refuses to adopt this number; code has adopted it. **Fix:** pin to an official changelog or remove the hard cap.
- `GET /api/evolution/lessons` silently drops malformed JSONL rows (`http.rs:2654-2659`), which can hide artifact corruption. **Fix:** count/surface skipped lines.
- `/health` and `/api/workspace` expose absolute paths and boot checks without `no-store` (`http.rs:1422-1432,3537-3592`); a local proxy cache could retain them.
- `history_state` and `history_revisions` return **404** for a malformed run UUID (`http.rs:879-881,906-908`), while other routes use 400; inconsistent but low impact.
- No HTTP request audit trail for profile import/delete, memory clear/prune/trust, provider profile save, or evolution apply/rollback beyond `tracing`; remote/daemon stores do audit. If the finish plan requires traceability, this is a gap (`http.rs:1200-1238,1466-1601,2720-2987,3367-3438`).
- `POST /api/provider/profile` accepts an arbitrary `name` (up to body limit) and writes it to disk (`http.rs:3411-3417`; `config.rs:254-305`); no schema length bound.
- WS token travels in the URL query (`client.ts:350-352`; `ws.rs:89-98`), which can leak through process/access logs if one is added later.
- `hiverouter-cli` has no `--url` default matching the launcher (8080 vs 8099) (`cli.rs:907-918` vs `start-hivemind.ps1:4`).

---

## 11. API additions needed by the finish plan (PROPOSED)

Priorities are for the “mission / tool / task UX” described in `docs/plans/HIVEMIND-MASTER-FINISH-PLAN.md:110-113` and the missing operator surfaces.

| Priority | Endpoint | Inputs → outputs | Auth | Why it is needed |
|---|---|---|---|---|
| P0 | `POST /api/runs` (alias `POST /api/missions`) | `{prompt≤8000, project_id}` → `{run_id,status}` + `RUN_CREATED`/`RUN_STATUS` events | Bearer, operator | HTTP-only clients/CLI cannot start a run today; only `WS SUBMIT_PROMPT` (`ws.rs:949-967`). |
| P0 | `GET /api/runs?status=&limit=&cursor=` and `GET /api/runs/{run_id}` | list/detail with pagination | read token | `GET /api/history/runs` is capped at 200 and lacks filters (`http.rs:856-872`). |
| P0 | `POST /api/runs/{run_id}/cancel`; `POST /api/tasks/{task_id}/kill`; `POST /api/tasks/{task_id}/rewire`; `POST /api/runs/{run_id}/tasks` (spawn) | mirror WS payloads (`command.rs:175-207`) → ack + event | Bearer, operator | Kill/cancel/spawn/rewire are WS-only (`ws.rs:969-992,786-942`). |
| P0 | `POST /api/approvals/{approval_id}/resolve` | `{nonce,decision,reason?}` → resolved approval | Bearer, operator | Resolution is WS-only (`ws.rs:1091-1118`); HTTP has read-only `GET /api/approvals` (`http.rs:1356-1391`). |
| P1 | `GET /api/tools`; `GET /api/tools/{tool_id}`; `POST /api/tools/{tool_id}/invoke` | registry/tool schema; invoke returns job/approval handle | read / Bearer+approval | Tool catalog is CLI-local (`cli.rs:617-675`); the model tool loop is internal. Needed for a tool UX and external agents. |
| P1 | `GET /api/tasks?run_id=&state=&limit=&cursor=`; `GET /api/tasks/{task_id}` | task rows + identity/class/ownership | read | Task detail exists only inside `run_detail`/`roster` (`http.rs:1304-1352,3127-3186`). |
| P1 | `GET /api/events?after_seq=&run_id=&limit=` | sequenced envelopes from a durable journal | read | Event replay is in-memory 256 and WS-only (`bus.rs:18-20`); no durable journal exists in this lane (VERIFIED no HTTP route; durable journal existence UNVERIFIED). |
| P1 | Pagination on history/approvals/A2A/memory | `cursor`/`before` + `next_cursor` | read | §10 #8. |
| P1 | `GET/POST /api/projects`, `PATCH /api/projects/{id}` | project list/create/rename | operator | Only read-only `GET /api/projects` (`http.rs:3460-3475`). |
| P2 | Scoped tokens (`read`, `operator`, `agent:<task_id>`) + `POST /api/session/refresh` | token metadata/expiry | launcher/operator | Current binary token from `/api/session` is full authority and given to any loopback peer (`http.rs:3594-3635`). |
| P2 | `GET /api/config/effective` (non-secret) | bind/origins/workspace/provider non-secret fields | read | `/api/runtime` + `/api/provider/profile` partially cover it; a single documented shape helps agents (`http.rs:3477-3535,3327-3364`). |

If a REST surface is added, keep the WS commands as the canonical interactive transport and ensure every new route is listed in a new `docs/contracts/HTTP-API-v1.md` with a route-inventory test (currently absent).

---

## 12. Coverage inventory

### Files read (VERIFIED static reads)

- `backend/crates/hiverouter-core/src/http.rs` — lines 1-120, 380-510, 596-3715 (all routes, middleware, handlers, tests); **not read:** 120-380 (boot helpers), 510-596 (`HealthResponse`/transport structs partially seen via grep).
- `backend/crates/hiverouter-core/src/ws.rs` — 1-120, 196-305, 455-1252; tests listed by name; **not read:** 121-196, 305-455 (template DAG remainder), 1254-1619 (tests bodies).
- `backend/crates/hiverouter-core/src/lib.rs` (all 73 lines), `main.rs` (all 255 lines).
- `backend/crates/hiverouter-core/src/config.rs` — 40-300, 400-738; **not read:** 1-40, 300-400 (profile parsing details).
- `backend/crates/hiverouter-core/src/provider_executor.rs` — 56-315, 428-885; **not read:** 1-56 (prompts), 315-428 (stream error handling), 886-1143 (`WirePlan` and tests).
- `backend/crates/hiverouter-core/src/bus.rs` — 90-169 plus grep of constants; **not read:** 170-304.
- `backend/crates/hiverouter-core/src/checks.rs` (all 225 lines), `static_ui.rs` (all 104 lines).
- `backend/crates/hiverouter-core/src/remote.rs` — 316-370, 490-559 plus grep; **not read:** remaining 1 100+ lines (policy/action resolution).
- `backend/crates/hiverouter-core/src/store.rs`, `daemon.rs`, `execution.rs`, `tools_bridge.rs`, `history.rs` — **grep only** (not full reads; out of lane except call sites).
- `backend/crates/hiverouter-provider/src/client.rs` — 38-127, 396-545, plus grep; `secret.rs` all; `lib.rs` grep; `error.rs` grep; `budget.rs` grep of constants/functions; `dto.rs` grep of constants/validation. **Not read full:** `accounting.rs`, `pricing.rs`, `tool_assembler.rs`, `client.rs` 128-396/546-992, `budget.rs` bodies.
- `backend/crates/hiverouter-contract/src/command.rs` (all 595 lines), `envelope.rs` (all 191), `refusal.rs` (all 377), `schema.rs` 1-220, `event.rs` 136-205, 505-619 + grep; **not read:** `event.rs` 206-505/620-868, `schema.rs` 221-384, `ids.rs`, `state.rs`, `timestamp.rs`, `usage.rs`.
- `backend/crates/hiverouter-tools/src/cli.rs` — 1-234, 905-969 + grep; `cli_support.rs` all; `src/bin/hiverouter.rs` all; `cli.rs` 235-905/969-1180 **not read** (delta/event/HTTP outcome bodies partially seen).
- `tools/hivemind-cli.mjs` — header, command registry sections, HTTP/WS sections (1040-1084, 1343-1387, 1747-1801) plus grep; remainder ~1 300 lines **not read line-by-line**.
- `tools/start-hivemind.ps1` (all 113 lines).
- `web/src/api/client.ts` (all 666), `http.ts` (all 104), `index.ts` (all 174), `refusals.ts` (all 52), `failure.ts` (all 81), `health.ts` (all 76), `mock.ts` 1-80; `contract.ts` 98-167 + grep; **not read:** `contract.ts` 1-98/167-1116, `mock.ts` 80-368, `ids.ts`.
- UI panels: `ModelsPanel.tsx` 25-174; `RemotePanel.tsx` 50-120; `main.tsx` 1-51; `App.tsx` via grep; other panels via endpoint grep.
- Docs: `docs/contracts/UI-COMMANDS-v1.md` via grep; `docs/plans/HIVEMIND-MASTER-FINISH-PLAN.md` via grep; **no full read of either**.

### Tests inventoried (existence/name VERIFIED; execution UNVERIFIED)

- HTTP: `backend/crates/hiverouter-core/tests/http_boot.rs` (≈30 named tests, incl. `mutations_require_session_token:496`, `health_endpoint_reports_boot_checks_and_database:631`, `remote_create_without_token_is_refused:980`).
- WS: `backend/crates/hiverouter-core/tests/ws_transport.rs` (14 named tests).
- Provider executor: `provider_adapter.rs`, `provider_streaming.rs`, `provider_live.rs` (live test; not run).
- Provider crate: `error_matrix.rs`, `rate_and_provenance.rs`, `refusal_matrix.rs`, `streaming.rs`, `tool_assembler.rs`, `tool_call_transport.rs`, `tool_continuation.rs`, `tool_conversation.rs`.
- Contract: `ui_command_matrix.rs`, `golden_schemas.rs`, `schema_roundtrip.rs`.
- CLI/Rust: `hiverouter-tools/src/cli.rs` unit tests; `tests/audit_jsonl.rs`, `tests/lead_bugcheck_wp01.rs`.
- Frontend: `web/tests/apiClient.test.ts`, `apiLifecycle.test.ts`, `apiBoundary.test.ts`, `contract.test.ts`, `refusalParity.test.ts`, `failure.test.ts`, `health.test.ts`, `proposals.test.ts`, `lessons.test.ts`.
- e2e: `e2e/specs/live-security.spec.ts`, `ws-contract.spec.ts`, `approval-flow.spec.ts`, `live-app`/`live-replay`/`live-resync` (names listed, not run).

### Explicit gaps in this report

1. No test was executed (read-only lane; `cargo test` builds and some tests bind loopback servers). All behavioral claims are static.
2. `execution.rs` (243 KB), `store.rs` (138 KB), `tools_bridge.rs`, `remote.rs` policy transactions, `budget.rs`/`accounting.rs` internals were not fully read; provider/executor data-flow claims are limited to the files read.
3. The `docs/plans` and `docs/audit` corpus was grepped, not exhaustively read; doc-drift claims are limited to the cited lines.
4. No runtime route enumeration (e.g., hitting `/openapi.json` or logging the router) was performed; the route list is from source.
5. The 393,216 token ceiling, axum body-size default, HEAD-to-GET routing, and timing-attack practicality are explicitly **UNVERIFIED**.
6. Remote/daemon/evolution store semantics (idempotency, CAS, audit rows) were sampled, not audited; refusals were traced to store call sites only where cited.

---

## 13. Verification status

| Topic | Status | Basis |
|---|---|---|
| Route list, handler line ranges, middleware order/behavior | **VERIFIED (static)** | `http.rs:604-704,706-823,846-3660` |
| Auth gates and token sources | **VERIFIED (static)** | `http.rs:17-40,738-793,3594-3635`; `ws.rs:45-98,462-497` |
| WS commands/events/limits | **VERIFIED (static)** | `ws.rs:196-1129`; `bus.rs:17-20` |
| Contract counts (17/14/24/78) | **VERIFIED (static)** | `command.rs:87-105`; `event.rs:72-96`; `refusal.rs:73-323`; web mirror `contract.ts:426-507` |
| Provider defaults/retry/budget/streaming | **VERIFIED (static)** | `client.rs:38-115,396-680`; `budget.rs` constants; `provider_executor.rs:56-885` |
| Config env/flags/profile write | **VERIFIED (static)** | `config.rs:23-330,400-738` |
| CLI command tables and behavior | **VERIFIED (static)** for Rust/Node tables; **UNVERIFIED** runtime |
| Node CLI HTTP mutation 401 / launch-token gap | **VERIFIED (static)** by header construction vs middleware | `hivemind-cli.mjs:1048-1057,1747-1776`; `http.rs:743-793` |
| UI→route existence comparison | **VERIFIED (static)** | `web/src/**` fetch literals vs `http.rs:609-700` |
| Any test passing | **UNVERIFIED** | no test executed |
| Performance/memory-leak impact (budget map, WS replay churn) | **UNVERIFIED** | static inference only |
| Docs claims about API auth | **VERIFIED contradiction / mixed** | `HIVEMIND-MASTER-FINISH-PLAN.md:34,87,171,256` vs `http.rs:738-793` |

---

## 14. Report shape (AGENTS §24)

### Done

| Item | Evidence |
|---|---|
| Full route/middleware inventory (64 HTTP methods + `/ws` + fallback) | `http.rs:604-704`; §2 tables |
| WS handshake/commands/refusals/limits | `ws.rs:462-1129`; §3 |
| Contract crate coverage (envelope/commands/events/refusals/schema) | `command.rs`, `envelope.rs`, `event.rs`, `refusal.rs`, `schema.rs`; §4 |
| Provider adapter/executor + client/budget/secret | `provider_executor.rs`, `client.rs`, `budget.rs`, `secret.rs`; §5 |
| Config + launcher comparison | `config.rs`, `start-hivemind.ps1`, `main.rs`; §6 |
| Both CLI surfaces | `cli.rs`, `cli_support.rs`, `hivemind-cli.mjs`; §7 |
| Frontend API client + UI path comparison | `web/src/api/**`, panel greps; §8-§9 |
| Top-10 ranked bugs with fix locations | §10 |
| API additions for finish plan | §11 |
| Coverage and explicit gaps | §12 |

### Not done

| Item | Reason |
|---|---|
| Execute any test suite | Read-only lane; tests build/run servers; no runtime claims made. |
| Full read of `execution.rs`, `store.rs`, `remote.rs`, `tools_bridge.rs`, full `web/src/api/contract.ts`, full Node CLI | Scope/time; noted in §12 gaps. |
| Runtime verification of 401/403 findings, timeout behavior, WS replay amplification | Requires starting the server and provider calls, which the lane forbids. |
| Editing source/config/tests | Not permitted by lane. |

### Unverified

| Claim | What would prove it |
|---|---|
| Node CLI HTTP mutations return 401 in practice | Start server, run `hivemind-cli profile-import` with a valid file; observe status/body. |
| Launcher-started server rejects Node CLI (`/api/session` 403) | Start via `start-hivemind.ps1`, run Node CLI `session`; observe 403. |
| Provider proxy can hold a request ~30 min | Stalled provider/wiremock with no response; measure wall time and retry count. |
| WS hello pull evicts replay and duplicates frames to other sockets | Two clients, one calls `pullPendingApprovals`, observe seq/replay and duplicate `APPROVAL_REQUEST` on both. |
| 393,216 is the official max output ceiling | Official provider changelog/docs with version pin; currently unconfirmed. |
| Budget map leak for runs without `finish_run` | Code-path audit of every terminal run path calling `finish_run`; runtime memory measurement. |
| No durable event journal exists | Full read/grep of `store.rs`/`db.rs` for an event table and any replay endpoint; not established in this lane. |
