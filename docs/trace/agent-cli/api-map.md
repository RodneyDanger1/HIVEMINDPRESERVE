# HiveMind v1 — callable surface map for a headless CLI/agent

Audit-only recon. No product code, test, or config was edited; the server was **not**
started, stopped, or probed over the network in this pass. Every claim below is either
`read from a file` at the cited `path:line`, or explicitly listed under
[Unverified](#unverified).

- Binary version reported by the built artifact: `hiverouter 1.2.0-dev` (contract schema
  `hiverouter.v1`).
- Contract schema id: `hiverouter.v1`
  (`backend/crates/hiverouter-contract/src/schema.rs:20`).
- Recon base: working tree `C:\Users\Riley\Desktop\hiverouter-v1`.
- Skill applied: `/forgewright-recon` (entry point -> callers -> data flow -> ledger) and
  `/integration-logic` (transport gate ordering, idempotency, refusal-by-name).

**Scope note (read this before using the map to plan work).** This map documents the
**Rust server surface** and the **existing Rust CLI** only. While this audit was being
written, another seat wrote untracked files into this same working tree
(`docs/AGENT-CLI.md` 01:17:25, `tools/hivemind-cli.mjs` 01:18:42, `tools/hivemind.cmd`
01:14:40, `HiveMind-Agent.cmd`/`.ps1` 01:15:36, `skills/`, plus edits to `README.md`). They
were **not** audited here — only their filenames and first lines were read, and they are
outside this seat's scope. Treat every "no CLI does X" statement below as
"the Rust `hiverouter-cli` does not do X"; a JS CLI claiming the same routes/contract is a
separate, unreviewed artifact. See [Unverified](#unverified).

---

## Recon ledger

- **Target behavior:** what a non-browser process can call today on the localhost server,
  over which transport, with which auth, and which refusals.
- **Entry points:**
  - `backend/crates/hiverouter-core/src/http.rs:511` — `build_router` (the single route table).
  - `backend/crates/hiverouter-core/src/ws.rs:364` — `ws_handler` (`GET /ws` upgrade gate).
  - `backend/crates/hiverouter-core/src/ws.rs:610` — `handle_client_text` (the only command
    dispatcher).
  - `backend/crates/hiverouter-core/src/main.rs:187` — server CLI usage/argv surface.
  - `backend/crates/hiverouter-tools/src/cli.rs:81` — `COMMANDS`, the CLI command table.
  - `tools/desktop-launcher/HiveMindLauncher.cs:23` — Windows launcher argv/env surface.
- **Callers walked:**
  - `crate::static_ui::static_ui` as the router fallback (`http.rs:533`).
  - `host_guard` layer applied to every route including `/ws` (`http.rs:534`, `http.rs:543`).
  - `SessionAuth` used by `/api/session` (`http.rs:1206`) and `/ws` (`ws.rs:379`, `ws.rs:382`).
  - `EventBus` as the single event sequencer/fan-out (`bus.rs:92`, `bus.rs:129`).
  - `RunEngine::{submit_prompt,cancel_run,kill_worker,spawn_sub_hive}` from `ws.rs`
    (`ws.rs:776`, `ws.rs:794`, `ws.rs:801`, `ws.rs:786`).
  - `ToolsBridge::{settings_status,file_tree_list,file_read,resolve_approval_ledgered}`
    (`ws.rs:808`, `ws.rs:880`, `ws.rs:890`, `ws.rs:907`).
- **Tests that pin this behavior:**
  - `backend/crates/hiverouter-core/tests/ws_transport.rs` (handshake, replay, seq).
  - `backend/crates/hiverouter-core/tests/http_boot.rs` (`/health`, `/api/session`, 404).
  - `backend/crates/hiverouter-contract/tests/ui_command_matrix.rs` +
    `.../golden_schemas.rs` + `.../schema_roundtrip.rs` (frozen names, schemas, refusals).
  - `backend/crates/hiverouter-tools/src/cli.rs:829` (CLI dispatch/exit-code tests).
  - Commands (from the repo's own docs, not executed here):
    `cargo test -p hiverouter-core --test ws_transport`,
    `cargo test -p hiverouter-core --test http_boot`,
    `cargo test -p hiverouter-contract`,
    `cargo test -p hiverouter-tools cli`.
- **Data flow (happy path):** `GET /api/session` -> token (`http.rs:1224`) ->
  `GET /ws?token=<t>` with `Origin` + `Host` (`ws.rs:373-384`) -> upgrade -> client sends
  `{"type":"hello","last_seq":n}` (`ws.rs:477`) -> `SERVER_HELLO` (+ replay, + pending
  approval replay) (`ws.rs:495-565`, `ws.rs:579`) -> COMMAND envelope
  (`schema.rs:53` -> `command.rs:457`) -> dispatch (`ws.rs:649-936`) -> `ACK` or `ERROR`
  published on the bus with the request `correlationId` (`ws.rs:952`, `ws.rs:943`).
- **Data flow (failure path):** COMMAND with an unknown `name` -> `validate_basic` ->
  `E_COMMAND_UNKNOWN` -> `ERROR` event (`envelope.rs:147`, `ws.rs:636`); no state mutated.
- **Unknowns:** see [Unverified](#unverified).

---

## 1. Transport gates (all routes)

Two independent gates run before any handler. Order: fallback/layer composition is
`layer(host_guard)` over `fallback(static_ui)` over the route table (`http.rs:533-535`), so
`host_guard` sees every request first.

| Gate | Check | Failure | Body | Source |
|---|---|---|---|---|
| Host (all routes) | `Host` header must equal `127.0.0.1:<port>`, `localhost:<port>`, or `[::1]:<port>` (case-insensitive), exact port. When bound port is `0` (tests) any loopback authority passes. | `421 MISDIRECTED_REQUEST` | `{code:"E_ORIGIN_REJECTED", message, how_to_fix}` — **3 fields, no `retryable`** | `http.rs:543-564`, `http.rs:571-584`, `http.rs:586-594` |
| WebSocket peer | Peer socket IP must be loopback | `403` `E_ORIGIN_REJECTED` | 4-field refusal | `ws.rs:373-375` |
| WebSocket Origin | `Origin` header must be on the allowlist; a **missing** Origin is rejected | `403` `E_ORIGIN_REJECTED` | 4-field refusal | `ws.rs:75-83`, `ws.rs:376-381` |
| WebSocket token | `?token=` must string-equal the boot token | `401` `E_TOKEN_INVALID` | 4-field refusal | `ws.rs:382-384` |
| `/api/session` Origin | If `Origin` present it must be allowlisted; if absent the peer must be loopback | `403` `E_ORIGIN_REJECTED` | 4-field refusal | `http.rs:1211-1222` |
| `/api/provider/*` Origin | Same allowlist; **absent Origin is accepted** | `403` `E_ORIGIN_REJECTED` | 4-field refusal | `http.rs:1115-1124` |

Origin allowlist default for a bound port `P`: `http://127.0.0.1:P`, `http://localhost:P`
(`config.rs:204-213`). Extra origins: `HIVEROUTER_ALLOWED_ORIGINS` (comma-separated,
case-insensitive dedupe) (`config.rs:191`, `config.rs:382-392`). A `::1` bind therefore has
**no** `http://[::1]:P` entry in the allowlist — bind `127.0.0.1`.

The per-session token is a fresh UUID v7 minted once when `AppState` is constructed
(`http.rs:337-340`, `http.rs:382-385`), constant for the process lifetime, and there is no
route that rotates it.

---

## 2. HTTP route table

19 routes plus one SPA/asset fallback. **No route mutates run/task/approval state by
design** — every mutation is a WebSocket command (section 5).

Auth legend: **none** = only the Host gate; **session** = Host + token (WS only);
**origin** = Host + Origin rule.

| # | Method | Path | Handler (`path:line`) | Auth | Request shape | Success | Refusals / status |
|---|---|---|---|---|---|---|---|
| 1 | GET | `/health` | `health` `http.rs:1153` | none | none | `200` (see payload below) | `503` when a **required** check failed or the DB is unreadable (`http.rs:1198-1202`). Body is the same payload. |
| 2 | GET | `/diagnose` | `diagnose` `http.rs:597` | none | none | `200` `{schema:"hiverouter.diagnose.v1", version, uptime_secs, db_ok, provider_key_present, provider, refusals, tools, capability_census, audit_log_dir, agent_tiers, permission_classes, side_effects_enabled}` | `500` `{code,message,how_to_fix,retryable}` for a census failure (`http.rs:608-618`) |
| 3 | GET | `/api/session` | `api_session` `http.rs:1206` | origin | none | `200` `{token, schema:"hiverouter.v1", last_seq, ws:"/ws"}` (`http.rs:1224-1231`) | `403` `E_ORIGIN_REJECTED` |
| 4 | GET | `/api/history/runs` | `history_runs` `http.rs:621` | none | none; server caps at **200** rows (`http.rs:622`) | `200` `{runs:[{run_id, project_id, status, error_code, created_at}]}` | `503` store refusal (`http.rs:636`) |
| 5 | GET | `/api/history/runs/{run_id}/revisions` | `history_revisions` `http.rs:640` | none | `run_id` = UUID string | `200` `{run_id, latest_revision:u64\|null, revision_count, revisions:[{revision, created_at, run_status}]}` | `404` `E_RUN_NOT_FOUND` only when the id is **not a UUID** (`http.rs:644-646`). A valid-but-unknown run returns `200` with `revisions: []` (`snapshot.rs:126-133`). |
| 6 | GET | `/api/history/runs/{run_id}/state?revision=N` | `history_state` `http.rs:666` | none | `revision` query **required** | `200` `{run_id, revision, schema_version, run_status, graph:{revision,nodes[],edges[][],toposort[]}}` (`http.rs:680-686`, graph projection `http.rs:701-741`) | `404` bad UUID; `400` `E_SCHEMA_INVALID` when `revision` is missing/unparsable (`http.rs:674-676`); `404` `E_HISTORY_GAP` (`http.rs:689-691`); `409` any other refusal (e.g. `E_SNAPSHOT_CORRUPT`) (`http.rs:692`) |
| 7 | GET | `/api/history/runs/{run_id}/messages` | `history_messages` `http.rs:772` | none | none; newest 1000 kept, `truncated` flag (`http.rs:782-791`) | `200` `{run_id, messages:[{id, run_id, role, text, created_at}], truncated}` | `404` bad UUID / unknown run (`http.rs:776-781`); `503` store refusal (`http.rs:813`) |
| 8 | GET | `/api/agents/runs/{run_id}/messages` | `agent_messages` `http.rs:817` | none | none | `200` `{run_id, messages:[{id, from_task_id, to_task_id, kind, text, created_at}]}` | `404` bad UUID / unknown run (`http.rs:821-826`); `503` (`http.rs:843`) |
| 9 | GET | `/api/agents/runs/{run_id}/roster` | `agent_roster` `http.rs:951` | none | none | `200` `{run_id, count, agents:[{task_id, parent_task_id, role, kind, label, state, depth, rev}]}` (`http.rs:963-985`) | `404` bad UUID / unknown run (`http.rs:955-960`); `503` (`http.rs:989`) |
| 10 | GET | `/api/agents/profiles` | `agent_profiles` `http.rs:925` | none | none | `200` `{profiles:[{id, display_name, tier, template_id, allowed_tool_ids, output_schema_id, refusal_policy, max_children, max_depth}]}` | none |
| 11 | POST | `/api/agents/profiles/import` | `import_agent_profile` `http.rs:847` | none | JSON body: either `{"profile":{…Persona…}}` or the Persona object itself (`http.rs:851`). Persona fields: `id, display_name, tier(hyper\|super\|worker\|tool), template_id, allowed_tool_ids[], output_schema_id?, refusal_policy("strict"\|"report"), max_children?, max_depth?` (`prompts.rs:367-404`) | `200` `{profile:{…}, source:"api"}` (`http.rs:870-885`) | `400` `E_SCHEMA_INVALID` (bad JSON) / `E_PERSONA_UNKNOWN` / `E_TEMPLATE_UNKNOWN` (`http.rs:854-860`, `prompts.rs:411-434`); `503` store (`http.rs:866`) |
| 12 | DELETE | `/api/agents/profiles/{profile_id}` | `delete_agent_profile` `http.rs:889` | none | path id, 1..=128 chars | `200` `{id, removed:true}` | `400` `E_SCHEMA_INVALID` (`http.rs:894-896`); `404` **`{code:"E_PROFILE_NOT_FOUND", message}` — 2-field body, no `how_to_fix`/`retryable`** (`http.rs:901-910`); `503` (`http.rs:899`) |
| 13 | GET | `/api/agents/memory` | `agent_memory_list` `http.rs:993` | none | none; 8 entries per persona | `200` `{profiles:[{id, display_name, memories:[{key, value}]}]}` | `503` on any per-persona read failure (`http.rs:1005-1008`) |
| 14 | DELETE | `/api/agents/memory/{agent_key}` | `clear_agent_memory` `http.rs:1013` | none | path key, 1..=128 chars | `200` `{agent_key, removed:bool}` | `400` `E_SCHEMA_INVALID` (`http.rs:1017-1020`); `503` (`http.rs:1027`) |
| 15 | GET | `/api/usage/runs/{run_id}` | `run_usage` `http.rs:1031` | none | none | `200` `{run_id, usage:{prompt_tokens, completion_tokens, total_tokens, prompt_cache_hit_tokens, prompt_cache_miss_tokens, reasoning_tokens}}` or `{run_id, usage:null}` (`http.rs:1042-1061`) | `404` bad UUID / unknown run (`http.rs:1035-1040`); `503` (`http.rs:1062`) |
| 16 | GET | `/api/projects` | `projects` `http.rs:1136` | none | none | `200` `{projects:[{id, name, created_at}]}` | `503` (`http.rs:1149`) |
| 17 | GET | `/api/provider/models` | `provider_models` `http.rs:1070` | origin | none; server reads `DEEPSEEK_API_KEY` | `200` `{object, models:[…]}` + `Cache-Control: no-store` (`http.rs:1079-1087`) | `403` `E_ORIGIN_REJECTED`; `503` `E_KEY_MISSING`; `401` `E_PROVIDER_AUTH`; `429` `E_PROVIDER_RATE_LIMIT`; `502` anything else (`http.rs:1126-1134`) |
| 18 | GET | `/api/provider/balance` | `provider_balance` `http.rs:1093` | origin | none | `200` `{is_available, balances:[…]}` + `no-store` | same mapping as #17 |
| 19 | GET | `/ws` | `ws_handler` `ws.rs:364` | origin + session | query `token` (required), `last_seq` (optional) (`ws.rs:86-96`) | `101` upgrade | `403` / `401` as in section 1 |
| — | any | (fallback) | `static_ui::static_ui` `http.rs:533`, `static_ui.rs:42` | none | — | `GET`/`HEAD`: embedded `web/dist` asset, else SPA `index.html`, else `404` `{error:"not found", schema:"hiverouter.v1"}` | `405` for non-GET/HEAD (`static_ui.rs:43-45`) |

Notes that matter to a caller:

- A matched route with the wrong method returns axum's `405`; an unmatched path falls to the
  SPA fallback, where `api/*`, `health`, and `ws` are forced to the JSON `404`
  (`static_ui.rs:48-50`).
- `run_id`/`task_id`/`approval_id` are plain UUID strings serialized hyphenated
  (`ids.rs:95-106`, `ids.rs:67-77`); any UUID version parses (`ids.rs:59-65`).
- `created_at`/`ts` values are second-precision `YYYY-MM-DDTHH:MM:SSZ`
  (`timestamp.rs:34-50`).

### `/health` payload (`http.rs:456-507`, `http.rs:1181-1203`)

```json
{"status":"ok|degraded|failing","version":"1.2.0-dev","schema":"hiverouter.v1",
 "uptime_secs":0,
 "checks":[{"name":"rustc|node|deepseek_api_key|prompt_assets|persona_assets|tools_migration",
            "required":bool,"ok":bool,"detail":"string","code":"E_*|null"}],
 "database":{"path":"string","schema_version":123|null,"error_code":"E_*|null"},
 "transport":{"last_seq":0,"graph_nodes":0|null,"allowed_origins":2},
 "prompt_assets":{"loaded":bool,"error_code":"E_*|null"}}
```

`status` logic: `ok` when every check passes and the DB is readable; `degraded` when only
**optional** checks failed; `failing` when a **required** check failed or the DB is
unreadable (`http.rs:1167-1180`). Only `failing` changes the status code to `503`.

The provider-key check is **optional** and is only inserted when the provider is enabled
(`main.rs:109-114`, `checks.rs:143-161`). With `HIVEROUTER_PROVIDER=off` there is no
`deepseek_api_key` row at all.

### HTTP error-body variants (a CLI must tolerate all three)

| Variant | Shape | Where |
|---|---|---|
| Full refusal | `{code, message, how_to_fix, retryable}` | `ws.rs:389-399` (WS pre-upgrade + most handlers), `http.rs:608-618` |
| Host-gate refusal | `{code, message, how_to_fix}` | `http.rs:553-561` |
| Deleted-profile 404 | `{code:"E_PROFILE_NOT_FOUND", message}` | `http.rs:902-909` |
| SPA fallback 404 | `{error:"not found", schema:"hiverouter.v1"}` | `static_ui.rs:74-83` |

---

## 3. WebSocket frame envelope

Wire shape (frozen): `docs/contracts/UI-COMMANDS-v1.md:19-26`, implemented at
`envelope.rs:36-61`.

```json
{"schema":"hiverouter.v1","type":"COMMAND|EVENT","name":"<STRING>","id":"<uuid-v7>",
 "correlation_id":"<uuid-v7>","run_id":"<uuid|null>","task_id":"<uuid|null>",
 "seq":<u64|null>,"ts":"<RFC3339>","payload":{...}}
```

| Field | Rule | Source |
|---|---|---|
| `schema` | must equal `hiverouter.v1` | `envelope.rs:133-135` |
| `type` | `COMMAND` or `EVENT` (`UPPERCASE`) | `envelope.rs:25-32` |
| `name` | frozen command/event name, else refused | `envelope.rs:142-156` |
| `id` | non-nil UUID; generated by the client for commands | `envelope.rs:136-138`, `envelope.rs:65` |
| `correlation_id` | non-nil; shared across request/ACK/ERROR/result | `envelope.rs:136-138`, `ws.rs:640`, `ws.rs:952` |
| `run_id`, `task_id` | optional (`default`) | `envelope.rs:49-53` |
| `seq` | **must be absent on a COMMAND**; assigned by the server on events, monotonic from 1 | `envelope.rs:144-146`, `bus.rs:106` |
| `ts` | RFC3339 second precision | `envelope.rs:57-58`, `timestamp.rs:34-50` |
| `payload` | typed per name; validated by JSON Schema then by typed decode | `schema.rs:170-179`, `command.rs:466-487` |

### Client -> server frames

There are exactly two kinds. Anything else is refused.

| Client frame | Shape | Effect | Source |
|---|---|---|---|
| hello / resync control | `{"type":"hello","last_seq":<u64\|null>}` — **not** an envelope | `SERVER_HELLO` + optional replay + pending-approval replay | `ws.rs:469-493` (handshake), `ws.rs:626-631` (mid-session) |
| COMMAND | the envelope above with `type:"COMMAND"` | dispatch -> `ACK` (+ result event) or `ERROR` | `ws.rs:633-936` |

Client failure behavior:

| Condition | Server answer | Source |
|---|---|---|
| Text that is not JSON | bus `ERROR` `E_BAD_FRAME` (session stays open) | `ws.rs:615-624` |
| JSON that is not an envelope | `ERROR` `E_SCHEMA_INVALID` | `ws.rs:633-639` |
| Envelope with an unknown `name` | `ERROR` `E_COMMAND_UNKNOWN` | `envelope.rs:147-149` |
| Envelope with a bad payload | `ERROR` `E_SCHEMA_INVALID` (jsonschema or typed decode) | `command.rs:466-487` |
| Binary frame | `ERROR` `E_BAD_FRAME` | `ws.rs:432-434`, `ws.rs:488-490` |
| No hello within `HELLO_TIMEOUT` = 10 s | `ERROR` `E_BAD_FRAME`, then close | `ws.rs:41`, `ws.rs:406-414` |
| `DIFF_*` command | `ERROR` `E_DECISION_REQUIRED` | `command.rs:463-465`, `command.rs:484-486` |
| Unhandled frozen command (defensive) | `ERROR` `E_INTERNAL_ERROR` | `ws.rs:927-935` |

### Server -> client frames

Every server frame is an `EVENT` envelope. **`seq` is populated only on bus-published
frames**; three frames are written straight to the socket and therefore carry `seq: null`:
`SERVER_HELLO` (`ws.rs:503-561`), the `RESYNC_REQUIRED` emitted during the hello exchange
(`ws.rs:543-550`), and the pre-loop `E_BAD_FRAME` on hello timeout (`ws.rs:410`). A client
must read `seq` as `Option`, and take the connect-time high-water mark from
`SERVER_HELLO.payload.last_seq` (`event.rs:175-182`), not from `envelope.seq`.

Broker semantics: bus-published events (everything else, including `ACK`/`ERROR` for
commands and the lag-time `RESYNC_REQUIRED`) go through one process-wide bus and are fanned
out to **every** subscribed session (`bus.rs:52-54`, `bus.rs:120`, `ws.rs:451-461`). There is
no per-session event routing, so `ERROR`/`ACK` for one client's command are observed by all
connected clients; filtering must be done on `correlation_id`.

---

## 4. Handshake, replay, resync, keepalive

| Behavior | Detail | Source |
|---|---|---|
| First frame deadline | 10 s; the query `last_seq` is only used when the hello omits it (`last_seq.or(query_last_seq)`) | `ws.rs:41`, `ws.rs:406-407` |
| Fresh client (`last_seq` absent) | `SERVER_HELLO{last_seq: current, schema}` only | `ws.rs:501-511` |
| Client exactly current | `SERVER_HELLO` only (`Replay::Current`) | `bus.rs:181-183`, `ws.rs:512-522` |
| Client behind, within buffer | `SERVER_HELLO`, then each missing envelope oldest-first (`Replay::Events`) | `bus.rs:200-205`, `ws.rs:523-538` |
| Gap (too old, or client claims a future seq) | `RESYNC_REQUIRED{expected_seq, last_seq}`, then `SERVER_HELLO` (stream resumes live; no snapshot is pushed) | `bus.rs:174-199`, `ws.rs:539-562` |
| Replay buffer | 256 events retained (`REPLAY_CAPACITY`) | `bus.rs:20`, `bus.rs:113-118` |
| Fan-out buffer | 1024 (`FANOUT_CAPACITY`); a lagging session gets `RESYNC_REQUIRED` and the socket **closes** | `bus.rs:18`, `ws.rs:451-461` |
| Seq numbering | process-global `u64`, starts at 1, `+1` per publish | `bus.rs:106`, `bus.rs:135` |
| Seq on connect frames | frames written directly to the socket (`SERVER_HELLO`, hello-time `RESYNC_REQUIRED`, hello-timeout `ERROR`) carry `seq: null`; only bus-published frames carry a `seq` | `ws.rs:1036-1039`, `ws.rs:503-561` |
| Approval pull / re-hello | sending `hello{last_seq}` mid-session replays the hello, any missing events, and re-publishes up to 20 pending approvals as ordinary `APPROVAL_REQUEST` frames | `ws.rs:569`, `ws.rs:579-598`, `ws.rs:626-631` |
| Keepalive in | WS `Ping` control frame -> server `Pong` control frame **and** a `PONG` event | `ws.rs:435-438`, `ws.rs:1050-1052` |
| Keepalive out | `PING` is a frozen event name but **no production code publishes it** | `event.rs:122`, grep over `backend/` (only tests/bus-tests reference `Event::Ping`) |
| Close | `Message::Close` or read error ends the session | `ws.rs:440-442` |

`RESYNC_REQUIRED` is *not* terminal for a re-connecting client: disconnect and reconnect with
`hello{last_seq:null}`, or read history over HTTP (section 2).

---

## 5. Commands (client -> server)

Dispatch table at `ws.rs:649-936`. Payload structs in
`backend/crates/hiverouter-contract/src/command.rs`. Refusal sets are declared in
`COMMAND_MATRIX` (`schema.rs:215-378`) and may be narrower than the code path can emit —
the *handler* column is what actually runs.

| Command | Payload (exact fields) | Handler | Success | Observed / declared refusals |
|---|---|---|---|---|
| `SUBMIT_PROMPT` | `text` (1..=8000 chars), `project_id` | `ws.rs:763-782` -> `execution.rs:1036` | `ACK` + `RUN_CREATED{run_id,status}` (`execution.rs:1060-1067`) | `E_PROMPT_EMPTY`, `E_PROMPT_TOO_LONG` (`command.rs:395-404`); `E_KEY_MISSING` from the check gate (`ws.rs:766-773`); `E_RUN_ACTIVE` (`execution.rs:1052-1054`); store refusals |
| `SPAWN_SUB_HIVE` | `parent_task_id`, `role` | `ws.rs:783-792` -> `execution.rs:1081` | `ACK` + `NODE_SPAWN` | `E_TASK_NOT_FOUND`, `E_RUN_TERMINAL`, `E_SPAWN_DEPTH`, `E_FANOUT_LIMIT`, `E_AGENT_LIMIT`, `E_PERSONA_UNKNOWN`/`E_SCHEMA_INVALID` (bad role) |
| `REWIRE_TASK` | `task_id`, `new_parent_id`, `expected_rev` | `ws.rs:650-756` | `ACK` + `NODE_REWIRE` | `E_TASK_NOT_FOUND`, `E_CYCLE`, `E_RESYNC_REQUIRED`, `E_REWIRE_IN_FLIGHT`, `E_INTERNAL_ERROR` (context switch / row write failure after the graph commit — the command is then refused, not ACKed: `ws.rs:704-740`) |
| `CANCEL_RUN` | `run_id` | `ws.rs:793-799` -> `execution.rs:1138` | `ACK`, then the engine publishes terminal `RUN_STATUS{cancelled}` when dispatch settles | `E_RUN_NOT_FOUND`, `E_RUN_TERMINAL` (`execution.rs:1139-1145`) |
| `KILL_WORKER` | `task_id` | `ws.rs:800-806` -> `execution.rs:1162` | `ACK` + `NODE_STATE{killed\|cancelled}` | `E_TASK_NOT_FOUND`, `E_TASK_TERMINAL`, `E_KILL_DENIED` (state not `running`/`awaiting_approval`) |
| `APPROVAL_RESOLVE` | `approval_id`, `nonce`, `decision` (`approve`\|`deny`), `reason?` | `ws.rs:899-926` | `ACK` + `APPROVAL_RESOLVED{approval_id,tool,decision}` | `E_SCHEMA_INVALID` (empty nonce), `E_APPROVAL_DECISION_INVALID` (any value but `approve`/`deny`, incl. `reject`), `E_APPROVAL_NOT_FOUND`, `E_APPROVAL_REPLAY`, `E_APPROVAL_TIMEOUT`, `E_INTERNAL_ERROR` if the ledger row cannot be appended after the decision committed |
| `SETTINGS_STATUS` | `{}` | `ws.rs:807-826` | `ACK` + `SETTINGS_STATUS_RESULT{provider,model,key_present,thinking?,reasoning_effort?}` | none; a missing key is data (`key_present:false`), never a refusal (`schema.rs:288-289`) |
| `FILE_TREE_LIST` | `path?` (workspace-relative dir) | `ws.rs:879-888` | `ACK` + `FILE_TREE_RESULT{root, entries:[{path,name,is_dir,kind,size_bytes}]}` | `E_FILE_TREE_UNAVAILABLE`, `E_PATH_OUTSIDE_JAIL`, `E_PATH_NOT_FOUND` |
| `FILE_READ` | `path` (workspace-relative) | `ws.rs:889-898` | `ACK` + `FILE_CONTENT{path,sha256,bytes,text}` | `E_FILE_TREE_UNAVAILABLE`, `E_PATH_OUTSIDE_JAIL`, `E_PATH_NOT_FOUND`, `E_FILE_TOO_LARGE` (1 MiB), `E_SCHEMA_INVALID` (non-UTF-8) |
| `TEMPLATE_INSERT` | `template_id`, `project_id` (both <=128 chars) | `ws.rs:827-861` -> `ws.rs:200-293` | `ACK` + one `NODE_SPAWN` per inserted node (inserted `queued`, **no execution claimed**) | boot `prompt_assets_error` first (`ws.rs:831-834`), `E_SCHEMA_INVALID`, `E_TEMPLATE_UNKNOWN`, `E_RUN_NOT_FOUND` (no active run), `E_RUN_TERMINAL`, `E_TASK_NOT_FOUND` (no root task), `E_SPAWN_DEPTH`/`E_FANOUT_LIMIT`/`E_AGENT_LIMIT` |
| `HISTORY_REVISION_SET` | `run_id`, `revision` | `ws.rs:862-878` | `ACK` + `HISTORY_SNAPSHOT{run_id,revision,graph}` | `E_HISTORY_GAP`, `E_RUN_NOT_FOUND`, `E_SNAPSHOT_CORRUPT` |
| `CHAT_PIN` | `pinned:bool` | `ws.rs:757-762` | `ACK` only (no server effect) | none |
| `VIEWPORT_SET` | `x,y,zoom` | `ws.rs:757-762` | `ACK` only (no server effect) | `E_ZOOM_RANGE` (outside 0.1..=3.0), `E_SCHEMA_INVALID` (non-finite) |
| `VIEWPORT_RECENTER` | `{}` | `ws.rs:757-762` | `ACK` only | none |
| `DIFF_PROPOSE` / `DIFF_APPLY` / `DIFF_REJECT` | opaque | refused before dispatch | — | `E_DECISION_REQUIRED` (`command.rs:463-465`) |

Command validation order (a CLI can rely on it):
`envelope structure` -> `frozen-name lookup` -> `jsonschema payload` -> `typed decode` ->
`semantic validation` (`command.rs:457-490`, `schema.rs:53-55`).

Semantic bounds: `SUBMIT_PROMPT` 1..=8000 chars, `VIEWPORT_SET` zoom 0.1..=3.0 and finite,
`APPROVAL_RESOLVE` non-empty nonce + `approve`/`deny`, `FILE_READ` non-blank path,
`SPAWN_SUB_HIVE` non-empty role (`command.rs:393-448`).

**Idempotency:** only `SUBMIT_PROMPT` is idempotent, and only by `correlation_id` — a repeat
returns the original run instead of creating a second one (`execution.rs:1049-1051`,
`store.rs` receipt table). No other command is deduplicated.

**Template insertion is narrow:** the catalog is derived from the loaded prompt-asset
manifest and emits **one single-node DAG per registered template id**; multi-node DAG data
does not exist in the assets (`ws.rs:122-173`). Ship assets with template ids
`content-writer`, `data-analyst`, `code-reviewer`, `chatbot` are asserted resolvable by the
crate's own test (`ws.rs:1181-1199`).

---

## 6. Events (server -> client)

24 names (`event.rs:19-97`). Payload struct and producer:

| Event | Payload fields (exact) | Producer (`path:line`) |
|---|---|---|
| `ACK` | `command`, `correlation_id` | `ws.rs:952-962` (envelope `correlation_id` forced to the request's, `ws.rs:958`) |
| `ERROR` | `code`, `message`, `how_to_fix`, `retryable` | `ws.rs:943-950` (`ErrorPayload::from_refusal`, `event.rs:163-172`) |
| `SERVER_HELLO` | `last_seq:u64\|null`, `schema` | `ws.rs:503-561` |
| `RESYNC_REQUIRED` | `expected_seq`, `last_seq` | `ws.rs:453-459`, `ws.rs:543-550` |
| `RUN_CREATED` | `run_id`, `status` | `execution.rs:1060-1067` |
| `RUN_STATUS` | `run_id`, `status`, `error_code?` | `execution.rs:986`, `1314`, `1413`, `1994`, `2016`, `2277`, `2407` |
| `REPLY_DELTA` | `run_id`, `text_delta` | `execution.rs:1810`, `1842` |
| `REPLY_DONE` | `run_id`, `usage` | `execution.rs:3036` |
| `NODE_SPAWN` | `task_id`, `state`, `rev?`, `kind?`, `parent_task_id?` | `ws.rs:1003-1034` (template insert), `execution.rs:768` |
| `NODE_STATE` | same `NodeEvent` shape | `execution.rs:751`, recovery catch-up `http.rs:318-326` |
| `NODE_REWIRE` | same `NodeEvent` shape | `ws.rs:743-751` |
| `NODE_DRAG` | same `NodeEvent` shape | no production producer (`ws.rs:993-996` only maps the name) |
| `RUN_METRICS` | `superagents`, `workers`, `green_pct`, `delta` | `execution.rs:687` |
| `APPROVAL_REQUEST` | `approval_id`, `tool`, `decision?`, `nonce?`, `run_id?`, `task_id?`, `tool_call_id?`, `args_sha256?`, `args_preview?` (<=512 UTF-16 units), `expires_at?` | `execution.rs:2092`, post-hello replay `ws.rs:584` |
| `APPROVAL_RESOLVED` | `approval_id`, `tool`, `decision` | `ws.rs:910-912` |
| `SETTINGS_STATUS_RESULT` | `provider`, `model`, `key_present`, `thinking?`, `reasoning_effort?` | `ws.rs:824` |
| `FILE_TREE_RESULT` | `root`, `entries[]` | `ws.rs:883` |
| `FILE_CONTENT` | `path`, `sha256`, `bytes`, `text` | `ws.rs:893` |
| `HISTORY_SNAPSHOT` | `run_id`, `revision`, `graph` | `ws.rs:867-873` |
| `PING` | `{}` | none in production |
| `PONG` | `{}` | `ws.rs:1050-1052` |
| `TOOL_CALL` | `run_id`, `task_id`, `tool_call_id`, `tool`, `args_sha256`, `turn` | `execution.rs:1922` |
| `TOOL_RESULT` | `run_id`, `task_id`, `tool_call_id`, `outcome`(`ok`\|`refused`\|`denied`\|`timeout`\|`cancelled`), `code?`, `bytes?`, `sha256?` | `execution.rs:1938` |
| `AGENT_MESSAGE` | `run_id`, `from_task_id`, `to_task_id`, `kind`, `text`, `created_at` | `execution.rs:730` |

`NodeEvent.kind` is one of `hyperagent|superagent|worker|tool`, populated only from the
task's authoritative tier, omitted when unknown (`event.rs:240-271`).

`NODE_*`, `APPROVAL_*`, `ACK`, `ERROR`, `RUN_STATUS` envelopes carry `run_id` when the
producer knows it (`bus.rs:129-142`); `ACK` sets only the correlation id (`ws.rs:953-959`).

---

## 7. Refusal schema

Wire form (identical in `ERROR` payloads and HTTP bodies):

```json
{"code":"E_*","message":"human cause","how_to_fix":"actionable remedy","retryable":true}
```

- Rust enum with stable codes: `backend/crates/hiverouter-contract/src/refusal.rs:11-71`
  (macro emits `code()`, `message()`, `how_to_fix()`, `retryable()`, `from_code()`).
- Serialized as a bare string code, deserialized by code lookup
  (`refusal.rs:284-296`); an unknown code fails deserialization.
- Table: `refusal.rs:73-282`. **65 variants**; `refusal.rs:315` asserts `>= 40` and
  `refusal.rs:304-316` asserts uniqueness and `E_[A-Z_]+` shape.
- `E_*` codes are the stable identity; `message`/`how_to_fix` text may change
  (`refusal.rs:5-7`).

| Code | Retryable | Line |
|---|---|---|
| `E_ORIGIN_REJECTED` | false | `refusal.rs:75` |
| `E_TOKEN_INVALID` | false | `refusal.rs:78` |
| `E_RESYNC_REQUIRED` | true | `refusal.rs:81` |
| `E_ZOOM_RANGE` | false | `refusal.rs:84` |
| `E_HISTORY_GAP` | false | `refusal.rs:87` |
| `E_DB_BUSY` | true | `refusal.rs:90` |
| `E_WS_DISCONNECTED` | true | `refusal.rs:93` |
| `E_BAD_FRAME` | false | `refusal.rs:96` |
| `E_COMMAND_UNKNOWN` | false | `refusal.rs:101` |
| `E_SCHEMA_INVALID` | false | `refusal.rs:104` |
| `E_PROMPT_EMPTY` | false | `refusal.rs:107` |
| `E_PROMPT_TOO_LONG` | false | `refusal.rs:110` |
| `E_RUN_ACTIVE` | false | `refusal.rs:113` |
| `E_KEY_MISSING` | false | `refusal.rs:116` |
| `E_SPAWN_DEPTH` | false | `refusal.rs:119` |
| `E_FANOUT_LIMIT` | false | `refusal.rs:122` |
| `E_AGENT_LIMIT` | false | `refusal.rs:125` |
| `E_SEMAPHORE_FULL` | true | `refusal.rs:128` |
| `E_CYCLE` | false | `refusal.rs:131` |
| `E_REWIRE_IN_FLIGHT` | true | `refusal.rs:134` |
| `E_RUN_TERMINAL` | false | `refusal.rs:137` |
| `E_RUN_NOT_FOUND` | false | `refusal.rs:140` |
| `E_TASK_NOT_FOUND` | false | `refusal.rs:143` |
| `E_TASK_TERMINAL` | false | `refusal.rs:146` |
| `E_KILL_DENIED` | false | `refusal.rs:149` |
| `E_BUDGET_EXCEEDED` | false | `refusal.rs:152` |
| `E_TOKEN_BUDGET_EXCEEDED` | false | `refusal.rs:155` |
| `E_CALL_LIMIT` | false | `refusal.rs:158` |
| `E_USAGE_INCOMPLETE` | false | `refusal.rs:161` |
| `E_PRICE_UNSOURCED` | false | `refusal.rs:164` |
| `E_PEAK_UNDEFINED` | false | `refusal.rs:167` |
| `E_COST_MATH_INVALID` | false | `refusal.rs:170` |
| `E_MULTIKEY_FORBIDDEN` | false | `refusal.rs:173` |
| `E_LLM_RATE_LIMIT` | true | `refusal.rs:176` |
| `E_APPROVAL_REQUIRED` | false | `refusal.rs:179` |
| `E_APPROVAL_REPLAY` | false | `refusal.rs:182` |
| `E_APPROVAL_TIMEOUT` | true | `refusal.rs:185` |
| `E_APPROVAL_NOT_FOUND` | false | `refusal.rs:188` |
| `E_APPROVAL_DECISION_INVALID` | false | `refusal.rs:191` |
| `E_DIFF_DECISION_REQUIRED` | false | `refusal.rs:194` |
| `E_DECISION_REQUIRED` | false | `refusal.rs:197` |
| `E_TEMPLATE_UNKNOWN` | false | `refusal.rs:200` |
| `E_TEMPLATE_LICENSE_MISSING` | false | `refusal.rs:203` |
| `E_TEMPLATE_REF_FORBIDDEN` | false | `refusal.rs:206` |
| `E_TEMPLATE_INVALID` | false | `refusal.rs:209` |
| `E_PERSONA_UNKNOWN` | false | `refusal.rs:212` |
| `E_SNAPSHOT_CORRUPT` | false | `refusal.rs:215` |
| `E_PATH_OUTSIDE_JAIL` | false | `refusal.rs:220` |
| `E_PATH_NOT_FOUND` | false | `refusal.rs:223` |
| `E_FILE_TOO_LARGE` | false | `refusal.rs:226` |
| `E_FILE_TREE_UNAVAILABLE` | false | `refusal.rs:229` |
| `E_TOOL_NOT_FOUND` | false | `refusal.rs:234` |
| `E_TOOL_SCHEMA` | false | `refusal.rs:237` |
| `E_JAIL_ESCAPE` | false | `refusal.rs:240` |
| `E_TOOL_TIMEOUT` | true | `refusal.rs:243` |
| `E_TOOL_CANCELLED` | true | `refusal.rs:246` |
| `E_TOOL_NOT_ALLOWED` | false | `refusal.rs:249` |
| `E_PROVIDER_AUTH` | false | `refusal.rs:254` |
| `E_PROVIDER_RATE_LIMIT` | true | `refusal.rs:257` |
| `E_PROVIDER_TIMEOUT` | true | `refusal.rs:260` |
| `E_PROVIDER_INCOMPLETE` | false | `refusal.rs:263` |
| `E_PROVIDER_INVALID_RESPONSE` | false | `refusal.rs:266` |
| `E_TOOLCHAIN_MSRV` | false | `refusal.rs:271` |
| `E_NODE_MISSING` | false | `refusal.rs:274` |
| `E_INTERNAL_ERROR` | true | `refusal.rs:279` |

---

## 8. Launcher / CLI / argv surface

### 8.1 Server binary `hiverouter` (`hiverouter-core`)

Verified output of `.\target\release\hiverouter.exe --help` (exit 0):

```
hiverouter 1.2.0-dev - HiveMind v1 backend (contract schema hiverouter.v1)
USAGE: hiverouter [--bind <ADDR>] [--data-dir <DIR>] [--db <FILE>]
OPTIONS/JOIN: --bind, --data-dir, --db, -h/--help, -V/--version
ENVIRONMENT: HIVEROUTER_BIND, HIVEROUTER_DATA_DIR, HIVEROUTER_DB, RUST_LOG
Only loopback bind addresses are accepted in v1.
```

| Flag | Env | Default | Accepted forms | Source |
|---|---|---|---|---|
| `--bind` | `HIVEROUTER_BIND` | `127.0.0.1:8080` | `--bind X`, `--bind=X` | `config.rs:10`, `config.rs:255`, `config.rs:266-268`, `config.rs:280-282` |
| `--data-dir` | `HIVEROUTER_DATA_DIR` | `data` (relative to CWD) | `--data-dir X`, `--data-dir=X` | `config.rs:12`, `config.rs:256`, `config.rs:269-271`, `config.rs:283-285` |
| `--db` | `HIVEROUTER_DB` | `<data-dir>/hiverouter.db` | `--db X`, `--db=X` | `config.rs:14`, `config.rs:258`, `config.rs:272-275`, `config.rs:286-288`, `config.rs:340-346` |
| `--workspace-root` | `HIVEROUTER_WORKSPACE_ROOT` | process CWD | `--workspace-root X`, `--workspace-root=X` | `config.rs:276-279`, `config.rs:289-291`, `config.rs:198-201` — **accepted by the parser but absent from `--help`** (`main.rs:187-214`) |
| `-h`/`--help`, `-V`/`--version` | — | — | exact tokens | `main.rs:28-42` |
| (unknown token) | — | — | refused: `unknown argument ... (try --help)`, exit 1 | `config.rs:292`, `config.rs:242-243`, `main.rs:44-50` |

Non-loopback bind is refused **before** binding with `E_ORIGIN_REJECTED`
(`config.rs:332-334`, `config.rs:219-222`, exit 1).

Boot order (`main.rs:26-176`): help/version -> config -> audit sink at
`<data-dir>/audit` (`main.rs:52`) -> open SQLite (`main.rs:60`) -> migrate (`main.rs:72`) ->
snapshots + `recover_on_boot` (`main.rs:85-106`) -> environment checks (`main.rs:108`) ->
provider-key check (`main.rs:109-114`) -> `AppState::with_graph` (`main.rs:129`) -> bind
(`main.rs:136`) -> print the bound URL to stdout (`main.rs:153-158`) -> `serve` with
Ctrl+C graceful shutdown (`main.rs:166`, `main.rs:178-184`, `http.rs:1239-1250`).

### 8.2 Provider/environment variables

| Env | Values | Default | Source |
|---|---|---|---|
| `DEEPSEEK_API_KEY` | key material; presence only is ever read | unset | `main.rs:109-114`, `checks.rs:140-161` |
| `HIVEROUTER_PROVIDER` | `auto\|on\|enabled\|true\|1` / `off\|disabled\|false\|0` | enabled | `config.rs:23`, `config.rs:79-90` |
| `HIVEROUTER_MODEL` | `deepseek-flash`, `deepseek-v4-pro` | `deepseek-flash` | `config.rs:16`, `config.rs:25`, `config.rs:91-101` |
| `HIVEROUTER_THINKING` | `enabled`/`disabled` (aliases on/off/true/false/1/0) | enabled | `config.rs:27`, `config.rs:102-105`, `config.rs:135-143` |
| `HIVEROUTER_LEAF_THINKING` | same | disabled | `config.rs:29`, `config.rs:106-109` |
| `HIVEROUTER_REASONING_EFFORT` | `none`, `low\|minimal`, `high\|medium\|xhigh`, `max\|ultra` | `high` | `config.rs:31`, `config.rs:110-113`, `config.rs:146-156` |
| `HIVEROUTER_MAX_OUTPUT_TOKENS` | 1..=documented max | 8192 | `config.rs:33`, `config.rs:118-121` |
| `HIVEROUTER_PLANNER_MAX_OUTPUT_TOKENS` | 1..=documented max | 16384 | `config.rs:35`, `config.rs:114-117` |
| `HIVEROUTER_ALLOWED_ORIGINS` | comma-separated extra origins | — | `config.rs:191`, `config.rs:382-392` |
| `HIVEROUTER_SIDE_EFFECTS` | `0\|false\|off\|disabled\|no` disables approved side effects | enabled | `config.rs:195`, `config.rs:355-361` |
| `RUST_LOG` | tracing filter | `info` | `main.rs:205` |

Any unknown value for the provider vars is a **hard config error**
(`E_SCHEMA_INVALID: …`, exit 1) — `config.rs:84-86`, `config.rs:94-98`,
`config.rs:139-141`, `config.rs:152-154`, `config.rs:160-167`.

### 8.3 `hiverouter-cli` (`hiverouter-tools`, artifact `hiverouter-cli.exe`)

Verified output of `.\target\release\hiverouter-cli.exe --help --format=json` (exit 0):
9 commands, **none mutating**.

| Command | Tier | Positionals | Flags | Source |
|---|---|---|---|---|
| `version` | 0 | — | — | `cli.rs:82-91` |
| `preflight` | 0 | — | — | `cli.rs:92-101` |
| `schema [command]` | 0 | `command?` | — | `cli.rs:102-111` |
| `tools list` | 0 | — | — | `cli.rs:112-121` |
| `tools describe <tool_id>` | 0 | `tool_id` | — | `cli.rs:122-131` |
| `capabilities census` | 0 | — | — | `cli.rs:132-141` |
| `provider cost` | 0 | — | `--model`, `--cache-hit`, `--cache-miss`, `--output`, `--at` | `cli.rs:142-151` |
| `health` | 1 (local HTTP) | — | `--url`, `--timeout` | `cli.rs:152-161` |
| `diagnose` | 1 (local HTTP) | — | `--url`, `--timeout` | `cli.rs:162-171` |

- Globals: `--format=json|human`, `--json`, `--dry-run` (only `health`/`diagnose` preview),
  `--help`, plus `--url=` / `--timeout=` read by the HTTP commands (`cli.rs:304-341`,
  `cli.rs:435`).
- stdout is one `hiverouter.cli.v1` JSON envelope (`{schema, ok, command, data|error, meta}`)
  when stdout is not a TTY or JSON is requested; human lines go to **stderr**
  (`cli.rs:1-21`, `cli.rs:199-231`, `cli.rs:389-409`).
- Exit codes: `0` ok, `2` usage, `3` named refusal, `4` internal (`cli.rs:51-57`).
- Default server URL for the HTTP commands: `http://127.0.0.1:8080` (`cli.rs:783`);
  `--timeout` default 10 s (`cli.rs:784-788`).
- `fetch_json` treats **any non-2xx as an error and discards the body**
  (`cli_support.rs:111-116`), which the command surfaces as `E_WS_DISCONNECTED`
  (`cli.rs:813`).
- **There is no CLI command for any mutating action**, no run listing, no prompt, no
  cancellation, no approvals, no file tools — in the Rust `hiverouter-cli` (the JS
  `tools/hivemind-cli.mjs` in this tree is a separate, unaudited artifact; see the scope
  note above).

### 8.4 Windows desktop launcher (`tools/desktop-launcher`)

| Behavior | Value | Source |
|---|---|---|
| Server argv | `--bind 127.0.0.1:8099 --data-dir "%LOCALAPPDATA%\HiveMind" --workspace-root "<root>"` | `HiveMindLauncher.cs:16`, `HiveMindLauncher.cs:38-41`, `HiveMindLauncher.cs:61` |
| Working directory | the resolved workspace root | `HiveMindLauncher.cs:62` |
| Port already in use by a non-HiveMind app | hard error dialog, no start | `HiveMindLauncher.cs:45-54` |
| Existing healthy server | launcher opens a window only, never stops it | `HiveMindLauncher.cs:55-83`, `HiveMindLauncher.cs:96-108` |
| Readiness probe | `GET http://127.0.0.1:8099/health`, 60 s budget, 300 ms poll | `HiveMindLauncher.cs:217-245` |
| Key injection | DPAPI `%LOCALAPPDATA%\HiveMind\provider.key` -> child env `DEEPSEEK_API_KEY`; falls back to the ambient env var | `HiveMindLauncher.cs:129-161` |
| Workspace root resolution | `HIVEMIND_WORKSPACE_ROOT` if it exists, else the hardcoded checkout path, else CWD | `HiveMindLauncher.cs:163-178` |
| Server lookup | beside the launcher, else `HIVEMIND_ROOT`, else the hardcoded checkout, else `target\release\hiverouter.exe` | `HiveMindLauncher.cs:180-199` |
| UI | same embedded web UI in an Edge/Chrome `--app` window | `HiveMindLauncher.cs:85-95` |
| `HiveMind.cmd` | forwards to `tools\start-hivemind.ps1` | `HiveMind.cmd:1-5` (script not read: see Unverified) |

---

## 9. Data-dir / DB / workspace semantics

| Item | Value | Source |
|---|---|---|
| Data dir | `--data-dir` / `HIVEROUTER_DATA_DIR` / default `data` relative to CWD; created with `create_dir_all` at config resolve | `config.rs:12`, `config.rs:255-257`, `config.rs:335-339` |
| SQLite file | `--db` / `HIVEROUTER_DB` / `<data-dir>/hiverouter.db`; a path that is a directory is refused | `config.rs:340-346`, `config.rs:236-237` |
| Audit sink | `<data-dir>/audit` — must be initialisable or the process exits | `main.rs:52-58` |
| Snapshots | `<data-dir>/snapshots/<run-id>/<revision:010>.json`, indexed in SQLite `snapshots` | `snapshot.rs:44-56`, `snapshot.rs:82-90`, `snapshot.rs:111-112` |
| Approval ledger | `<data-dir>/ledger/run-ledger.jsonl` (append-only JSONL) | `tools_bridge.rs:335-337`, `tools_bridge.rs:408` |
| Workspace root | `--workspace-root` / `HIVEROUTER_WORKSPACE_ROOT` / process CWD | `config.rs:198-201`, `config.rs:367-369` |
| Workspace jail | opened at boot; if it fails, `FILE_*` refuse `E_FILE_TREE_UNAVAILABLE` rather than touching disk | `http.rs:297-308`, `tools_bridge.rs:267-269` |
| Side effects | approved side effects enabled unless `HIVEROUTER_SIDE_EFFECTS` is a falsy token | `config.rs:355-361`, `http.rs:223-226` |
| Bind | loopback only; non-loopback refused at config time | `config.rs:332-334` |
| Server output | `hiverouter <version> listening on http://<addr> (db <path>, schema v<n>, contract hiverouter.v1)` on **stdout** | `main.rs:153-158` |
| Shutdown | Ctrl+C -> graceful (in-flight requests drain); no HTTP shutdown route | `main.rs:178-184`, `http.rs:1239-1250` |
| Boot recovery | interrupted runs are marked `interrupted`; a corrupt workspace **refuses to start** | `main.rs:87-106` |

---

## 10. What an agent must do (headless playbooks)

All steps below are code-derived. Only section 8's `--help` invocations were executed in
this pass; see [Unverified](#unverified).

### 10.1 Start the server keyless

1. Unset `DEEPSEEK_API_KEY` (do **not** set `HIVEROUTER_PROVIDER=off` if you want the fast
   `E_KEY_MISSING` refusal — see 10.3).
2. `target\release\hiverouter.exe --bind 127.0.0.1:8080 --data-dir <abs-data-dir>
   --workspace-root <abs-workspace>` (`main.rs:187-214`, `config.rs:264-296`).
3. Read the printed `listening on http://<addr>` line from stdout (`main.rs:153-158`); with
   `--bind 127.0.0.1:0` parse the port from it.
4. Expect `ExitCode::FAILURE` with a named message and no fake start when: config invalid,
   audit sink unavailable, DB open/migrate failure, boot recovery refusal, or bind failure
   (`main.rs:44-145`). The bind-failure message is prefixed `E_WS_DISCONNECTED`
   (`main.rs:139-143`) — that code names a *bind* failure here, not a lost session.
5. Set `Host` on every request. Nothing else is needed for HTTP reads.

### 10.2 Query status

| Goal | Call | Expect (keyless) |
|---|---|---|
| Liveness + checks | `GET /health` | `200` with `status:"degraded"` and a `deepseek_api_key` check `ok:false, code:"E_KEY_MISSING"` (optional check: `checks.rs:154-159`, `required:false`, so `required_ok` stays true and the status code is 200 per `http.rs:1198-1202`) |
| Capability/derived state | `GET /diagnose` | `200`, `{schema:"hiverouter.diagnose.v1", provider_key_present:false, side_effects_enabled, …}` |
| Session token | `GET /api/session` (no `Origin`, loopback peer) | `200` `{token, schema, last_seq, ws:"/ws"}` |
| Existing runs | `GET /api/history/runs` | `200` `{runs:[…]}` (<=200) |
| Configured model/thinking | WS `SETTINGS_STATUS` | `ACK` + `SETTINGS_STATUS_RESULT`; the model id is **not** in `/diagnose` |

### 10.3 Submit a prompt and expect `E_KEY_MISSING`

1. Open `GET /ws?token=<token>` as a WebSocket. You **must** send
   `Origin: http://127.0.0.1:8080` (or another allowlisted value) and the matching `Host`;
   a missing `Origin` is rejected (`ws.rs:75-83`, `ws.rs:376-381`).
2. First text frame, within 10 s: `{"type":"hello","last_seq":null}` (`ws.rs:477`).
   Read `SERVER_HELLO` (`ws.rs:501-511`).
3. Send a COMMAND envelope:

```json
{"schema":"hiverouter.v1","type":"COMMAND","name":"SUBMIT_PROMPT",
 "id":"<uuid-v7>","correlation_id":"<uuid-v7>",
 "ts":"2026-11-08T07:10:24Z",
 "payload":{"text":"hello","project_id":"project-1"}}
```

4. With the provider **enabled** and no key: expect
   `ERROR{code:"E_KEY_MISSING", retryable:false, message:"No DeepSeek API key is configured.",
   how_to_fix:"Set the DEEPSEEK_API_KEY environment variable and restart the server."}`
   with `correlation_id` equal to the request's (`ws.rs:766-773`, `refusal.rs:116-118`).
   No run row is created on this path (the refusal precedes `submit_prompt`).
5. With `HIVEROUTER_PROVIDER=off` the check row does not exist, so the gate at
   `ws.rs:766-773` cannot fire: expect `ACK` + `RUN_CREATED`, then a later failed reply from
   `UnconfiguredExecutor::…-> Err(Refusal::KeyMissing)` (`execution.rs:290-311`,
   `execution.rs:300`, `execution.rs:311`) surfacing as `ERROR`/`RUN_STATUS{status:"failed",
   error_code:"E_KEY_MISSING"}`. **Ordering of those later frames is unverified.**

### 10.4 List runs / state / roster / messages

| Data | Endpoint | Notes |
|---|---|---|
| Runs | `GET /api/history/runs` | newest first, cap 200 (`http.rs:622`) |
| Revision index | `GET /api/history/runs/{id}/revisions` | `latest_revision` is `null` when no snapshot exists; unknown-but-valid run is `200` with `[]` |
| Graph at revision | `GET /api/history/runs/{id}/state?revision=N` | `revision` is mandatory; the graph is filtered to the run's own tasks and fail-closed (`http.rs:701-714`) |
| Roster (task ids for A2A/rewire) | `GET /api/agents/runs/{id}/roster` | authoritative `TaskId`s (`http.rs:947-991`) |
| Chat messages | `GET /api/history/runs/{id}/messages` | newest 1000, `truncated` flag |
| Agent-to-agent messages | `GET /api/agents/runs/{id}/messages` | durable A2A rows |
| Usage | `GET /api/usage/runs/{id}` | `usage:null` when nothing recorded |
| Profiles / memory / projects | `GET /api/agents/profiles`, `/api/agents/memory`, `/api/projects` | read models |

There is **no** HTTP "current live graph" or "active run" endpoint; the live graph is the
`NODE_*` event stream plus `HISTORY_SNAPSHOT`.

### 10.5 Select a replay revision

Two independent mechanisms:

1. **Static read (HTTP):** `GET /api/history/runs/{run_id}/state?revision=N` returns the
   graph at `N` (`http.rs:666-694`). Out-of-range `N` -> `404 E_HISTORY_GAP`. Revision `0`
   is the first committed snapshot; `latest` = last entry of `/revisions`.
2. **Live view switch (WS):** `HISTORY_REVISION_SET{run_id, revision}` -> `ACK` +
   `HISTORY_SNAPSHOT{run_id, revision, graph}` (`ws.rs:862-878`). A historical selection
   does not stop the live stream: subsequent `NODE_*`/`REPLY_*` frames keep arriving, so a
   client must hold its own "viewing revision" flag.

### 10.6 Cancel a run

`CANCEL_RUN{run_id}` -> `ACK` (`ws.rs:793-799`). The engine cancels the run token
(`execution.rs:1143`) and the dispatch loop publishes the single terminal
`RUN_STATUS{status:"cancelled"}` (`execution.rs:1146-1150`). Refusals:
`E_RUN_NOT_FOUND`, `E_RUN_TERMINAL` (also returned when no token is live for the run,
`execution.rs:1143-1145`).

### 10.7 Rewire a task

1. Get the task id and its current `rev` from `/api/agents/runs/{id}/roster` (or the last
   `NODE_*` frame — `rev` is the **graph** revision, not the client event counter).
2. `REWIRE_TASK{task_id, new_parent_id, expected_rev}`.
3. Success: `ACK` then `NODE_REWIRE` carrying `rev`, `kind`, `parent_task_id`
   (`ws.rs:742-751`).
4. Failure modes to handle explicitly: `E_CYCLE`, `E_RESYNC_REQUIRED` (stale
   `expected_rev`), `E_REWIRE_IN_FLIGHT`, `E_TASK_NOT_FOUND`, and the fail-loud case where
   the graph commit succeeded but the row/context write failed — that returns `ERROR`
   instead of `ACK` (`ws.rs:704-740`), so the graph may already have moved. Re-read the
   roster and resync.

### 10.8 Approve or deny a tool call

1. Obtain a pending `APPROVAL_REQUEST` — either live during a run, or replayed after a
   `hello` (max 20, `ws.rs:569`, `ws.rs:579-598`). Persisted replays carry **no**
   `args_preview` (`event.rs:511-519`); `args_sha256` is the authoritative binding.
2. Send `APPROVAL_RESOLVE{approval_id, nonce, decision:"approve"|"deny", reason?}` with the
   nonce from that frame. `decision:"reject"` is invalid (`command.rs:527-547`).
3. Success: `ACK` + `APPROVAL_RESOLVED{approval_id, tool, decision}` (`ws.rs:909-912`); the
   suspended engine turn is woken afterwards (`ws.rs:916-921`).
4. Refusals: `E_APPROVAL_NOT_FOUND`, `E_APPROVAL_REPLAY` (nonce already used),
   `E_APPROVAL_TIMEOUT`, `E_APPROVAL_DECISION_INVALID`, `E_SCHEMA_INVALID`; plus
   `E_INTERNAL_ERROR` if the decision committed but the ledger row could not be appended —
   in that case the approval is cancelled and no `APPROVAL_RESOLVED` is published
   (`tools_bridge.rs:381-389`).
5. There is **no HTTP endpoint to list pending approvals**. The only pull is the `hello`
   replay.

### 10.9 Stream events

1. Connect, send `hello{last_seq}` (persist the last `seq` you saw).
2. Read frames indefinitely; each is an `EVENT` envelope with monotonic `seq`.
3. Correlate: keep a map `correlation_id -> pending command`; resolve on matching `ACK` or
   `ERROR`. Ignore frames with foreign correlations (the bus is broadcast).
4. On `RESYNC_REQUIRED`, either reconnect with `hello{last_seq:null}` or rebuild from
   `/api/history/*`; the server then resumes live only (`ws.rs:551-562`).
5. If the socket closes after a lag warning, reconnect (`ws.rs:451-461`).
6. Keepalive: send a WS `Ping` control frame; expect a `Pong` control frame and a `PONG`
   event. Do not wait for a server-initiated `PING` — none is produced.

### 10.10 Stop safely

- Send Ctrl+C / SIGINT to the server process; `serve` drains and exits 0 (`main.rs:166-175`,
  `http.rs:1239-1250`).
- There is no HTTP shutdown/quit route; killing the process instead leaves runs to be marked
  `interrupted` on the next boot (`main.rs:87-106`).
- Close WS sockets first if you care about clean session teardown; the server closes on
  `Message::Close` (`ws.rs:440-442`).

---

## 11. Gaps — where WebSocket is mandatory, and where HTTP is mandatory

### 11.1 No HTTP endpoint exists (WS is the only path)

| Capability | Only path | Source |
|---|---|---|
| Submit a prompt / create a run | `SUBMIT_PROMPT` | `ws.rs:763` (no `POST /api/*` route in `http.rs:513-535`) |
| Cancel a run | `CANCEL_RUN` | `ws.rs:793` |
| Kill a worker | `KILL_WORKER` | `ws.rs:800` |
| Rewire a task | `REWIRE_TASK` | `ws.rs:650` |
| Spawn a sub-hive | `SPAWN_SUB_HIVE` | `ws.rs:783` |
| Approve / deny a tool call | `APPROVAL_RESOLVE` | `ws.rs:899` |
| List **pending** approvals | `hello` replay only | `ws.rs:579-598` |
| Insert a template DAG | `TEMPLATE_INSERT` | `ws.rs:827` |
| List insertable templates | none (derive from `prompt_assets`/assets) | `ws.rs:133-173` |
| Read configured model / thinking / effort | `SETTINGS_STATUS` | `ws.rs:807-826` |
| List/read workspace files (UI path) | `FILE_TREE_LIST`, `FILE_READ` | `ws.rs:879`, `ws.rs:889` |
| Live node topology / reply stream | `NODE_*`, `REPLY_*`, `TOOL_*`, `AGENT_MESSAGE` events | `event.rs:19-97` |
| Resync after a lag | `hello` / reconnect | `ws.rs:451-461`, `bus.rs:172-206` |
| Enforce the keyless gate before run creation | WS dispatch | `ws.rs:766-773` |

### 11.2 No WS command exists (HTTP is the only path)

| Capability | Only path |
|---|---|
| Health / checks / DB status | `GET /health` |
| Derived diagnostics, refusal census, tool census | `GET /diagnose` |
| Session token + current `last_seq` | `GET /api/session` |
| Run list (all runs, newest first) | `GET /api/history/runs` |
| Revision index | `GET /api/history/runs/{id}/revisions` |
| History state (static read; the WS variant swaps the live view instead) | `GET /api/history/runs/{id}/state` |
| Chat message history | `GET /api/history/runs/{id}/messages` |
| Agent-to-agent message history | `GET /api/agents/runs/{id}/messages` |
| Agent roster | `GET /api/agents/runs/{id}/roster` |
| Agent profile CRUD | `GET/POST/DELETE /api/agents/profiles*` |
| Agent memory list/clear | `GET/DELETE /api/agents/memory*` |
| Per-run usage | `GET /api/usage/runs/{id}` |
| Project list | `GET /api/projects` |
| Provider model catalogue / balance (server-side key) | `GET /api/provider/models`, `/api/provider/balance` |

### 11.3 Asymmetries worth flagging to whoever builds the CLI

1. **No mutation over HTTP at all** — a headless CLI cannot be HTTP-only.
2. **No HTTP read for a single run's status** — you must scan `/api/history/runs` (cap 200).
3. **No HTTP endpoint for the live graph**; history snapshots are per committed revision.
4. **No `E_RUN_NOT_FOUND` from `/revisions`** for an unknown run; only `/messages`,
   `/state`, `/usage`, `/roster` distinguish it.
5. **`HEAD`/`OPTIONS` and unknown `api/*` paths** return the SPA fallback's JSON 404, not a
   refusal envelope (`static_ui.rs:42-50`, `static_ui.rs:74-83`).
6. **The `Host` gate is the DNS-rebinding defense**; an HTTP client that rewrites or omits
   `Host` is rejected with `421` and a 3-field body (`http.rs:543-564`).
7. **Broadcast bus**: ACK/ERROR are not unicast; a multi-client CLI must filter by
   `correlation_id` (`bus.rs:52-54`, `ws.rs:943-962`).
8. **`E_KEY_MISSING` has two shapes** (pre-run refusal vs late failure) depending on
   `HIVEROUTER_PROVIDER` (section 10.3).
9. **`TEMPLATE_INSERT` refuses `E_RUN_NOT_FOUND` when no run is active** and requires the
   caller to know the run's `project_id` (`ws.rs:231-238`) — the payload has no run id.
10. **The existing CLI cannot mutate and hides non-2xx bodies** (`cli_support.rs:111-116`,
    `cli.rs:813`); `hiverouter-cli health` against a `503 /health` reports
    `E_WS_DISCONNECTED` and loses the check list.
11. **`--workspace-root` is undocumented in `--help`** although it is parsed
    (`config.rs:276-279` vs `main.rs:192-201`).
12. **`::1` binds produce an allowlist without `http://[::1]:P`**
    (`config.rs:204-213`), so a WS client must use `Origin: http://localhost:P` — or the
    server should be bound to `127.0.0.1` (unverified which browsers do).

---

## 12. CLI implications

Concrete requirements for a production headless CLI/agent against this server:

1. **Two transports, one process.** HTTP for reads (no token), WS for every mutation and for
   the event stream. An HTTP-only client can implement exactly the Rust CLI's 9 read-only
   commands plus the `/api/*` reads — nothing that mutates state.
2. **Send `Host` exactly.** `127.0.0.1:<port>` (or `localhost:<port>`, `[::1]:<port>`) or
   the request is refused with `421` + `E_ORIGIN_REJECTED`. Never let an HTTP library
   override `Host`.
3. **Send `Origin` on the WS upgrade.** Non-browser clients do not, by default; a missing
   `Origin` is a hard `403` (`ws.rs:75-83`). Use an allowlisted value; if the server was
   started with `HIVEROUTER_ALLOWED_ORIGINS`, any listed value works.
4. **Bootstrap the token** with `GET /api/session` and pass it as `?token=` byte-for-byte
   (no trimming, no URL re-encoding surprises). The token changes on every restart.
5. **Speak `hello` first, within 10 s.** Query `last_seq` is a fallback; the hello field
   wins (`ws.rs:407`).
6. **Generate a fresh UUID v7 `id` per frame** and keep `correlation_id`; never send `seq`
   on a command (`envelope.rs:136-146`). Retry `SUBMIT_PROMPT` only by reusing the same
   `correlation_id` (that is the only deduped command).
7. **Parse three error-body variants** (section 2) and four refusal surfaces; never assume
   `retryable` exists.
8. **Map codes, not messages.** Build the `code -> handler` table from `refusal.rs:73-282`
   (stable) and treat `message`/`how_to_fix` as display strings.
9. **Exit-code mapping** for a CLI that mirrors the existing contract: `0` success, `2`
   usage/schema, `3` named refusal, `4` internal (`cli.rs:51-57`); `retryable:true` codes
   should be retried with backoff, not surfaced as terminal.
10. **Keyless mode is a first-class path.** `SUBMIT_PROMPT` must be expected to return
    `E_KEY_MISSING` on a machine with no key, and the CLI must not treat that as a server
    defect — `/health` is still `200` (degraded).
11. **Do not poll `/health` for readiness with a 200-only expectation** if you also need the
    check list: a `failing` boot is `503` and a naive client loses the body (see gap 10).
    Read the body regardless of status.
12. **Event correlation is client-side.** One connection sees every client's ACK/ERROR; keep
    a pending map and a seq watermark, and persist the watermark across restarts.
13. **Approval work needs the nonce.** Store the full `APPROVAL_REQUEST` frame (or at least
    `approval_id` + `nonce` + `args_sha256`); a replayed request may lack `args_preview`, and
    a nonce is single-use (`event.rs:488-493`, `command.rs:211-221`,
    `tools_bridge.rs:355-391`).
14. **Rewire needs `rev`, not a counter.** Use the `rev` from `NODE_*`/roster, never the
    client's event `seq` (`event.rs:251-260`).
15. **Template insertion needs a live run + its project id**, and refuses
    `E_TEMPLATE_UNKNOWN` unless the id is in the loaded prompt-asset manifest
    (`ws.rs:218-229`).
16. **Shutdown must be Ctrl+C/SIGINT.** There is no API to stop the server; a hard kill is
    recovered on next boot as `interrupted` (`main.rs:87-106`).
17. **Port discovery**: parse the `listening on http://…` line, or pin `--bind` to a known
    port. Default is `8080`; the desktop launcher uses `8099`.
18. **Sandbox/workspace**: file tools are jailed to `--workspace-root`; a CLI that needs to
    read files outside it must pass a different root at boot, not fight the jail
    (`tools_bridge.rs:267-269`).

---

## Unverified

Not executed in this audit (read-only, no server start). Each item names what would settle it.

| Claim / behavior | Why unverified | How to prove it |
|---|---|---|
| Live `/health` body and `200 degraded` on a keyless boot | No server started | Start keyless and `GET /health`; compare `status`, `checks[]`, `prompt_assets.loaded` |
| The `421` Host-rejection and `403`/`401` WS rejections in practice | No HTTP/WS client run | `curl -H "Host: evil.test" http://127.0.0.1:8080/health`; connect `/ws` without `Origin`, with a bad token |
| `E_KEY_MISSING` ordering when `HIVEROUTER_PROVIDER=off` (run created, then late failure) | Code-read only; the dispatch/failure frame order was not observed | Start with `HIVEROUTER_PROVIDER=off`, submit, record the frame order |
| `GET /api/history/runs/{unknown-uuid}/revisions` returning `200 []` | Code-read; `list_revisions` returns `Ok(vec![])` on a missing directory, but the SQLite index path (`indexed_revisions`) was not exercised | Call it with a random UUID against a live server |
| Whether two servers can share one `--data-dir` (SQLite lock, audit sink contention) | Not tested | Start a second instance on the same data dir and read the failure |
| Whether `AXUM` returns `405` vs the SPA `404` for `POST /health` | Code-read of axum semantics, not observed | `curl -X POST /health` |
| Whether `Message::Ping` from a non-browser client also yields the `PONG` event | Code-read (`ws.rs:435-438`) | Connect and send a WS ping; read the next frames |
| `PING` is never published in production | Negative grep result across `backend/`; absence is harder to prove than presence | Instrument the bus, or run a long session and assert no `PING` |
| `tools/start-hivemind.ps1` behavior (args, env, port) | File not read; `HiveMind.cmd` only forwards to it | Read `tools/start-hivemind.ps1` |
| `target\release\hiverouter.exe` matches the working-tree sources (built 2026-11-07 23:43) | Binary not rebuilt in this pass | `cargo build --release` and compare behavior, or `git status` + rebuild |
| The exact `prompt_assets`/`persona_assets` check rows in `/health` | Only the failure path was read (`http.rs:82-108`, `http.rs:110-136`); success rows are pushed by `checks` helpers not read here | Start and read `/health` |
| Provider `/api/provider/models` and `/balance` shapes | Documented from `http.rs` wiring; the upstream DeepSeek response body was not inspected | Call with a key present |
| Windows Ctrl+C graceful drain actually completing | Not observed | Start, submit a slow request, Ctrl+C, watch for clean exit |
| CLI `health`/`diagnose` against a live 503 | Only the 503-body-discard code path was read | Run against a deliberately `failing` boot |

---

## File index (one line each)

- `backend/crates/hiverouter-core/src/http.rs` — the 19-route table, host guard, history/agent/usage/provider handlers.
- `backend/crates/hiverouter-core/src/ws.rs` — upgrade gate, hello/replay, the only command dispatcher, template DAG insert.
- `backend/crates/hiverouter-core/src/bus.rs` — seq assignment, 256-event replay buffer, 1024 fan-out, lag -> resync.
- `backend/crates/hiverouter-core/src/main.rs` — boot order, argv usage, stdout listening line, Ctrl+C shutdown.
- `backend/crates/hiverouter-core/src/config.rs` — flags/env, loopback enforcement, data dir/DB/workspace/origins.
- `backend/crates/hiverouter-core/src/checks.rs` — `rustc`/`node`/optional `deepseek_api_key` checks.
- `backend/crates/hiverouter-core/src/execution.rs` — `submit_prompt`, `spawn_sub_hive`, `cancel_run`, `kill_worker`, `UnconfiguredExecutor`.
- `backend/crates/hiverouter-core/src/history.rs`, `snapshot.rs` — revisions, `E_HISTORY_GAP`, snapshot paths.
- `backend/crates/hiverouter-core/src/tools_bridge.rs`, `static_ui.rs` — tools wiring, approval ledger, SPA/404 fallback.
- `backend/crates/hiverouter-contract/src/{envelope,command,event,refusal,schema,ids,state,timestamp}.rs` — the frozen wire contract.
- `backend/crates/hiverouter-tools/src/cli.rs`, `cli_support.rs`, `health.rs`, `commands.rs` — the 9-command agent CLI and its HTTP helper.
- `tools/desktop-launcher/HiveMindLauncher.cs` — Windows launcher argv/env/key injection.
- `docs/contracts/UI-COMMANDS-v1.md` — the frozen prose contract this map is checked against.
