# 02 — Agent runtime wiring: prompt → planner JSON → spawn → tool loop → reply

**Status:** read-only wiring audit. No product source was edited; this file is the only write.
**Audited tree:** `C:\Users\Riley\Desktop\hiverouter-v1`, HEAD `d91f826` **plus uncommitted working-tree changes**.
**Method:** static read of the named files, plus official DeepSeek docs fetched 2026-11-07. No app was run, no live provider call was made, and no API-key value was read or printed. Findings are `READ` (seen in file) or `INFERRED` (reasoned from code + docs); runtime behaviour is `UNVERIFIED`.

### Revision note — the tree changed mid-audit

The audit began at ~18:08 against the pre-change tree. Between **18:19:33 and 18:19:56** a concurrent editor landed uncommitted changes in three files. This document describes the **post-change** bytes (hashes at write time):

| File | SHA-256 (first 16) | mtime | What changed during the audit |
|---|---|---|---|
| `hiverouter-core/src/provider_executor.rs` | `A1101EB7DD0FD525` | 18:19:33 | **tools now added to the first leaf request**; `finish_reason` validation added; prompt-token estimate added |
| `hiverouter-provider/src/client.rs` | `AAC009F1CD508991` | 18:19:39 | 500/502/503/504 now map to retryable `E_PROVIDER_TIMEOUT` |
| `web/src/panels/ModelsPanel.tsx` | `66BDE6EE7E21E55C` | 18:19:56 | `is_available` now displayed; `WIRE_MODELS` badge added |
| `hiverouter-core/src/http.rs` | `C18EA34C10C7A533` | 17:42:00 | unchanged during audit |
| `web/src/state/store.ts` | `2C05314D1B1C9B04` | 18:07:31 | unchanged during audit |
| `web/src/chat/ChatCore.tsx` | `01401BB80B22F04A` | 18:09:29 | unchanged during audit |

**Consequence:** the audit caught a genuine **P0 (leaf never offered tools)** and a **P1 (non-success `finish_reason` persisted as success)** on the pre-change bytes; both are *provisionally fixed* in the current uncommitted tree. They are recorded below as **fixed-but-unverified** — no Rust test was added (only `web/tests/store.test.ts` changed), and the fix has not been compiled or run in this audit. Treat them as `WIRED (unverified)`, never `WIRED-AND-WORKS`.

**Official anchors (fetched 2026-11-07; docs state no API version → `version_pinned: false`):**
[Chat Completions](https://api-docs.deepseek.com/api/create-chat-completion) ·
[Thinking Mode](https://api-docs.deepseek.com/guides/thinking_mode) ·
[Tool Calls](https://api-docs.deepseek.com/guides/tool_calls) ·
[JSON Output](https://api-docs.deepseek.com/guides/json_mode) ·
[Lists Models](https://api-docs.deepseek.com/api/list-models) ·
[Get User Balance](https://api-docs.deepseek.com/api/get-user-balance) ·
[Error Codes](https://api-docs.deepseek.com/quick_start/error_codes)

---

## 0. Bottom line

The prompt → planner-JSON → spawn → reply path is **real**: the UI submits, the engine makes one non-streaming JSON-mode planning call on `deepseek-flash`, validates the DAG, inserts task rows, streams each leaf reply, and settles usage.

On the **current** bytes the first leaf turn also advertises the tier's tools (`provider_executor.rs:306-322`), so the documented tool loop is now *reachable*. That fix is uncommitted, untested, and unverified; on the pre-change bytes it was a P0 and the whole loop (`spawn_agent`, `finish`, `report_result`, `file_*`, `shell_exec`, `TOOL_CALL`/`TOOL_RESULT`) was dead code.

The provider account proxies are **key-safe** (no key in browser, response, or log) but **unauthenticated**. The provider UI is read-only; there is no control for model/thinking/effort, no refresh on the Models surface, no stop in chat, and no retry-run.

---

## 1. The actual path (hop by hop)

### 1.1 Hop table (current bytes)

| # | Hop | Where it happens | Evidence |
|---|---|---|---|
| 1 | Operator submits | `ChatCore.tsx:131-169,318-348` → `store.ts:1826-1839` (`SUBMIT_PROMPT`) | READ |
| 2 | WS command dispatch | `ws.rs:743-752` → `engine.submit_prompt` | READ |
| 3 | Run row + background dispatch | `execution.rs:1016-1054` (create run, receipt, user message, `RUN_CREATED`, `tokio::spawn(dispatch)`) | READ |
| 4 | One planning call | `execution.rs:1309-1319` (`caps.check_calls(1)`, `planning_prompt`, `executor.plan`) | READ |
| 5 | Planner request built | `provider_executor.rs:269-296`; `client.rs:350-490` (`complete` → `drive` → `send_once` → `open`) | READ |
| 6 | Plan JSON parsed | `provider_executor.rs:292-295,434-481`; `WirePlan` `:491+`; `Plan::validate` `planner.rs:50-100` | READ |
| 7 | Spawn planned tasks | `execution.rs:1321-1369` (root + one row per node, caps check, `NODE_SPAWN`) | READ |
| 8 | Leaf execution | `execution.rs:1390-1420` → `execute_task` `:2726-2734` → `execute_task_with_prompt` `:2741-2756` → `execute_task_inner` `:2758-2804` | READ |
| 9 | Leaf provider call | `provider_executor.rs:298-331` → `stream_reply` `:106-264` → `client.rs:562-626,628-777` (`stream_inner` → `open_stream` → `pump_stream` → `delta_event`) | READ |
| 10 | Tool loop | `execution.rs:2811-2944`; tool turn `:1570-1756`; continuation `provider_executor.rs:340-411` | READ — reachable only because of the 18:19 fix |
| 11 | Complete / persist / usage | `execution.rs:2967-3024`; deltas `:1763-1814`; usage `:698-719`; run terminal `:1245-1307` | READ |
| 12 | Usage read by UI | `http.rs:811-844` → `ModelsPanel.tsx:88-138` | READ |
| 13 | Cancel | `ControlBar.tsx:97-106` → `store.ts:1920-1927` → `ws.rs:763-768` → `execution.rs:1118-1130` → `provider_executor.rs:141-145` | READ |

### 1.2 Exactly what the planner sends

`planning_prompt` (`execution.rs:1546-1559`) renders the embedded template when present. The template flattens its own system/user sections into one string:

```
System: You are the HiveMind planner. Return a task DAG of at most 8 children with depth at most 3.

User: Plan the run for: <run.prompt>
```
(`backend/crates/hiverouter-core/assets/prompts/planner.prompty:16-20`; flattening at `execution.rs:1550-1555`.)

`ProviderExecutorAdapter::plan` (`provider_executor.rs:275-287`) then sends **one** non-streaming request:

- `system` = `PLANNER_SYSTEM` (`:25-28`): *"You are the HiveMind planner. Return ONE JSON object only, no markdown fences, no prose. Schema: {"nodes":[{"id":"n1","role":"worker","deps":[],"depth":1}]}. … depth must be >= 1 and <= 3. Produce the smallest useful DAG (1..=40 nodes)."*
- `user` = `"Create the execution-plan DAG for this user task. Return JSON only.\n\nTask:\n" + planning_prompt`
- `model` = `settings.model` (default `deepseek-flash`, `config.rs:16,60`)
- `thinking` = `planner_thinking` (default **enabled**, `config.rs:61`)
- `reasoning_effort` = `planner_reasoning_effort` (default `high`, `config.rs:63`)
- `max_tokens` = `planner_max_tokens` (16 384, `config.rs:18`)
- `response_format` = `{"type":"json_object"}` (`:287`)
- **no `tools`, no `tool_choice`** (correct for a planner)

Wire shape is documented (`dto.rs:394-422`) and matches the live doc surface: `thinking`, `reasoning_effort`, `max_tokens` 1..393 216, `response_format`. The template's "at most 8 children" contradicts the system prompt's "1..=40 nodes"; the runtime actually allows only 8 planned nodes (P1-2).

### 1.3 Exactly what the leaf sends

`ProviderExecutorAdapter::execute` (`provider_executor.rs:298-331`) sends a **streaming** request:

- `system` = `TASK_SYSTEM` (`:30-32`)
- `user` = `"Original user prompt:\n{prompt}\n\nAssigned role: {role}\nSpawn depth: {depth}\nReturn the final answer for this task only."` (`:426-432`, plus optional scoped context at `execution.rs:53-59`)
- `thinking` = `leaf_thinking` (**default disabled**, `config.rs:62,108`)
- `max_tokens` = `leaf_max_tokens` (8 192, `config.rs:20`)
- `tools` = WorkerAgent allowlist (`tool_loop.rs:206-219`; registry allowlist `tool.rs:408-421`), `tool_choice` = `auto` — **added at 18:19** (`provider_executor.rs:306-307,317-322`)
- **no `reasoning_effort`** for the leaf (only the planner sets it; `ProviderSettings` has no leaf-effort field, `config.rs:39-54`)

The continuation request (`continue_with_tools`, `:352-402`) rebuilds the same tool array and replays the transcript: system + user, then for each turn one assistant message (`content` = that turn's visible text, `tool_calls` = the planned calls, `reasoning_content` only when non-empty, `:380-384`) followed by one `role=tool` message per bounded result (`:386-391`); `thinking` = leaf, `max_tokens` = leaf, `tools` = WorkerAgent allowlist, `tool_choice` = `auto` (`:396-402`). This matches the documented second-turn shape (`dto.rs:494-529`).

### 1.4 Tool-loop entry — what changed mid-audit

**Pre-change (the P0 the audit caught):** `execute` built `ChatRequest::new(...).with_thinking(...).with_max_tokens(...)` and never called `.with_tools(...)`; the only `.with_tools` on a `ChatRequest` was the unreachable continuation. A conforming DeepSeek endpoint cannot return `tool_calls` for a request that supplies no `tools`, so `reply.tool_calls` was always empty (`tool_loop.rs` assembler fed only by `client.rs:744-759`), the loop guard at `execution.rs:2814-2816` exited immediately, and all of `spawn_agent` (`:1678-1686`), `finish`/`report_result` (`:2863-2910`), the read-only executor (`:1838-1879`) and `TOOL_CALL`/`TOOL_RESULT` (`:1882-1937`) was dead code. The engine tests hid it by fabricating `tool_calls` in a mock (`tests/tool_loop_engine.rs:125-177`); the live smoke asserts only text/usage (`tests/provider_live.rs:66-93`).

**Current (post-change):** `execute` computes `tool_loop::tool_specs_for_tier(&ToolRegistry::standard(), task_agent_tier(task))` and, when non-empty, adds `.with_tools(tools).with_tool_choice(ToolChoice::Auto)` (`provider_executor.rs:306-322`). The engine's authorization registry is also `ToolRegistry::standard()` (`execution.rs:476,621`), so offered and authorized sets agree.

**Status:** `WIRED (unverified)`. There is still no test asserting that a real `execute` request carries `tools`; `git status` shows no Rust test change. Compile/run evidence is absent. If this fix is reverted or fails to compile, the loop is dead again.

### 1.5 Tool loop (now reachable)

1. `execute_tool_turn` (`execution.rs:1570-1756`) counts turns/calls, validates each call (`tool_loop::plan_call` `:1646`), authorizes it against the task tier (`:1656`), dispatches `spawn_agent` inline (`:1678-1686`), approval-gated side effects (`:1715-1728`), or read-only execution (`:1730`).
2. `finish` / `report_result` become the terminal assistant message and complete the task without another round-trip (`:2863-2910`); refused finishes fall through to the transcript.
3. Otherwise the transcript is pushed (`:2911-2915`) and `continue_with_tools` is called (`:2922-2929`), which re-offers the tools and replays every turn's assistant tool-call message plus `role=tool` results (`provider_executor.rs:352-402`).
4. Caps: 4 turns, 16 calls per run (`tool_loop.rs:14-16`); per-run provider attempts are capped by `RunBudget` (`budget.rs:31,243`).

### 1.6 The reply path (works today)

1. SSE text deltas → `on_delta` → `reply_delta_sink` publishes `REPLY_DELTA` (`execution.rs:1763-1789`) → `store.ts:1402-1419` appends to the streaming assistant message → `ChatCore.tsx:256-271` renders it.
2. `Finished { usage, finish_reason }` → `budget.record_call` (`provider_executor.rs:211-221`). Absent/partial usage becomes `E_USAGE_INCOMPLETE`, never 0 (`accounting.rs:205-258`). Consistent with the documented rule that the last chunk before `[DONE]` carries usage.
3. `finish_reason` handling (current): `length` → a specific incomplete log (`:222-229`); any reason other than `stop`/`tool_calls` → `E_PROVIDER_INCOMPLETE` (`:230-239`). This closes the pre-change false-success hole for `insufficient_system_resource`, `aborted`, and `content_filter`.
4. Empty text and no tool calls → `E_PROVIDER_INVALID_RESPONSE` (`:249-257`).
5. `complete_task` (`execution.rs:2967-3024`) persists the assistant message once, publishes `REPLY_DONE` with usage, marks the task completed, records the task result, adds run usage, and records bounded agent memory.
6. `dispatch` persists run usage at terminal (`:1305`); `GET /api/usage/runs/{id}` (`http.rs:811-844`) serves it; `ModelsPanel.tsx:88-138` renders it.

### 1.7 Cancellation

`CANCEL_RUN` cancels the run token (`execution.rs:1118-1130`). The provider adapter observes it in `stream_reply` (`provider_executor.rs:141-145`) and while planning (`:288-291`), returning `ToolCancelled`; `dispatch` maps that to `AgentState::Cancelled` (`execution.rs:1257-1261`). `KILL_WORKER` is wired per task (`:1142+`, `ControlBar.tsx:107-116`).

---

## 2. Where `DEEPSEEK_API_KEY` is read and injected

### 2.1 Read sites (all presence-only except the actual bearer build)

| Site | Behaviour | Evidence |
|---|---|---|
| `ApiKey::from_env` | reads `DEEPSEEK_API_KEY`, trims, rejects empty, wraps in `Zeroizing` | `secret.rs:30-38,46-57` |
| `DeepSeekClient::from_env` | builds the one client from that key | `client.rs:251-253` |
| `ProviderExecutorAdapter::from_env` | the adapter the server uses | `provider_executor.rs:57-60` |
| `build_executor` | called **once** at `AppState::with_graph`; missing key → `UnconfiguredExecutor` | `http.rs:229-244,364` |
| `ApiKey::present_in_env` | boolean only for checks/settings | `secret.rs:59-65`; `checks.rs:143-161`; `settings.rs:22` |
| `bearer_header` | the only `expose()` use; builds `Authorization` | `client.rs:329-339` |

No key value is ever logged or serialised: `ApiKey` Debug is `ApiKey(***)` (`secret.rs:77-81`), `DeepSeekClient` Debug redacts (`client.rs:202-211`), transport errors use fixed strings (`client.rs:785-803`), and `ProviderError` carries static detail only (`error.rs:9-10`).

### 2.2 Injection paths (they disagree)

**A. `HiveMind.exe` launcher (the only DPAPI path)** — `tools/desktop-launcher/HiveMindLauncher.cs`
- starts `hiverouter.exe --bind 127.0.0.1:8099 --data-dir %LOCALAPPDATA%\HiveMind` **only if the port is closed** (`:55-68`);
- `ApplyProviderKey(start)` (`:128-135`) reads `LoadProviderKey()` (`:137-160`): DPAPI `ProtectedData.Unprotect(file, entropy "HiveMind.ProviderKey.v1", CurrentUser)` → `start.EnvironmentVariables["DEEPSEEK_API_KEY"] = key`;
- on any failure the `catch` is empty (`:155-158`) and it falls back to the launcher's own environment variable (`:159`).

**B. `HiveMind.cmd` → `tools/start-hivemind.ps1`** — starts the server (`:29-31`) with **no DPAPI read at all**; the stored `provider.key` is ignored on this path.

**C. Provisioning** — `tools/set-provider-key.ps1:20-38` prompts hidden, DPAPI-protects, writes `%LOCALAPPDATA%\HiveMind\provider.key`. It is a separate manual step; nothing in the app calls it.

**D. Portable build** — `build-desktop.ps1:75-80` deliberately ships no key; the README tells the operator to run the script.

### 2.3 Why a UI-launched app can lack the key (six concrete causes)

1. **Nothing provisions the file.** A fresh install has no `provider.key`; the launcher silently skips injection (`:131-134`) and the server boots with `UnconfiguredExecutor` (`http.rs:229-244`).
2. **The `.cmd` path never reads DPAPI** (`start-hivemind.ps1:29-31`), so storing the key and launching via `HiveMind.cmd` still yields no key.
3. **Already-running server is reused.** If port 8099 answers `/health`, the launcher only opens the window (`HiveMindLauncher.cs:45-55`) and never injects — a server previously started without the key stays keyless.
4. **DPAPI failure is swallowed** (`:155-158`): a key file copied from another user/machine, or a profile change, decrypts to nothing and looks identical to "no key".
5. **No reload/rotation path.** The server reads the environment at boot only (`config.rs:370`; `http.rs:364`; `provider_executor.rs:57-60`). Creating/rotating `provider.key` after boot has no effect until restart; there is no HTTP/WS command to set or reload it.
6. **The failure looks like success.** The key check is `required: false` (`checks.rs:148-152`), `/health` returns HTTP 200 `degraded` (`http.rs:928-963`), and the launcher accepts any 2xx (`HiveMindLauncher.cs:199-213`). The window opens normally; the first prompt then fails at planning with `E_KEY_MISSING` (`UnconfiguredExecutor`, `execution.rs:290-313`; UI renders the run refusal at `ChatCore.tsx:87-89,210-220`).

**Current DPAPI path in one line:** `HiveMind.exe` → DPAPI-unprotect `%LOCALAPPDATA%\HiveMind\provider.key` (CurrentUser, entropy `HiveMind.ProviderKey.v1`) → inject `DEEPSEEK_API_KEY` into the `hiverouter.exe` child it starts → `ApiKey::from_env` in the server → `Authorization: Bearer …` only when calling DeepSeek.

**Hygiene note:** the key survives as an environment variable of the server child, readable by same-user processes (`HiveMindLauncher.cs:133`). The DPAPI file protects at rest only.

---

## 3. `/api/provider/models` and `/api/provider/balance` — are they safe?

Routes: `http.rs:496-497`. Handlers: `provider_models` `:850-866`, `provider_balance` `:869-885`, error mapper `:887-895`. Client methods: `client.rs:266-313`.

| Question | Verdict | Evidence |
|---|---|---|
| Key never reaches the browser | **YES** | Responses contain only `object` + `models.data` / `is_available` + `balances` (`http.rs:856-881`); `ModelInfo`/`BalanceInfo` are public metadata (`dto.rs:891-933`). No key field exists in either DTO. |
| Key never in a response body | **YES** | Errors go through `provider_error_response` → `refusal_response(status, &error.refusal())` (`http.rs:894`; `ws.rs:391-399`), which emits only static `code`/`message`/`how_to_fix`/`retryable`. `ProviderError::detail()` is **not** included. |
| Key never in a log | **YES** | Neither handler has a `tracing` call; `account_get` logs nothing (`client.rs:279-313`); Debug impls redact (`secret.rs:77-81`, `client.rs:202-211`). |
| Auth | **NO** | Handlers take no `State`; there is no session/origin/loopback gate, unlike `/api/session` (`http.rs:967-983`) and `/ws` (`ws.rs:364-387`). The router has no middleware/layer (`http.rs:479-501`). |
| Error mapping | **PARTIAL** | `E_KEY_MISSING`→503, `E_PROVIDER_AUTH`→401, `E_PROVIDER_RATE_LIMIT`→429, everything else→502 (`http.rs:887-895`). Gaps: timeouts→502 (not 504); 401 and 402 both collapse to `E_PROVIDER_AUTH` (`client.rs:508-511`), so "bad key" and "out of balance" are indistinguishable; `account_get` does not retry at all. 5xx now maps to retryable `E_PROVIDER_TIMEOUT` (`client.rs:516-519`, changed mid-audit). |
| Provider disabled | **ignored** | Handlers build a client unconditionally (`http.rs:851,870`); `config.provider.enabled` is not consulted. |

**Assessment:** the proxies are safe with respect to the key, and that is the property the design claims. Their real weakness is the missing gate — any local caller (and any cross-origin `GET`, which browsers can send even though they cannot read the response without CORS) can make the local server perform authenticated account calls. Impact is limited to outbound models/balance requests; the key cannot be exfiltrated. They also allocate a fresh `DeepSeekClient` (and a fresh `InFlightLimiter`) per request (`client.rs:65`), bypassing the shared limiter.

---

## 4. Proposed UI actions — exact seams and verdicts

### 4.1 Model list
- **Read/display: WIRED.** `ModelsPanel.tsx:55-86` fetches both proxies; models render at `:200-219`; route `http.rs:496/850-866`.
- **Refresh: MISSING.** One-shot `useEffect(..., [])` (`:86`); no button/retry. Seam: `ModelsPanel.tsx:44-47,55-86`, or lift to a store action modelled on `probeHealth` (`store.ts:2214-2234`).
- **Select: MISSING.** Model is environment-only (`config.rs:91-101`) and snapshotted into the adapter at boot (`provider_executor.rs:36-39,57-59`). A UI selector without a server command + runtime re-read would lie.
- **Badge caveat:** `WIRE_MODELS` is a hard-coded browser-side set (`ModelsPanel.tsx:27`, used `:208`). It duplicates the server's model vocabulary (`dto.rs:25-56`); if the server's accepted set changes, the badge lies.

### 4.2 Balance
- **Display: WIRED** (`ModelsPanel.tsx:176-198`), route `http.rs:869-885`.
- **Truthfulness: WIRED** — `is_available` is parsed (`:68,74`) and an insufficient-balance warning is rendered (`:184-186`); this was fixed mid-audit (pre-change it was dropped).
- **Refresh: MISSING** (same one-shot effect).

### 4.3 Tools
- **Route exists, UI missing.** `GET /diagnose` (`http.rs:482,504-526`) returns tools diagnostics with no secrets; nothing in `web/src` fetches it.
- Seams: a Settings/Agents section; store action `probeDiagnose()` copied from `probeHealth` (`store.ts:2214-2234`); the HTTP pattern is `fetchHealth` in `web/src/api/http.ts`.
- A real *tool policy* control (allowlist/approval) is server-owned (`tool.rs:391-430`) with no command. MISSING.

### 4.4 Thinking / effort
- **Display: PARTIAL.** `SETTINGS_STATUS_RESULT` carries both (`event.rs:637-650`), filled in `ws.rs:777-795` from `planner_thinking` / `planner_reasoning_effort`; store maps them (`store.ts:1622-1637`); panel renders them (`SettingsPanel.tsx:69-74`). The labels ("Thinking mode", "Reasoning effort", `strings.ts:156-157`) imply a global mode, but the values are **planner-scope**: the leaf default is thinking disabled (`config.rs:62`) and `ProviderSettings` has no `leaf_reasoning_effort` (`config.rs:39-54`).
- **Control: MISSING.** Configuration is environment-only (`config.rs:102-121`), resolved at boot, snapshotted in the adapter. A control needs a command + mutable settings + per-call read, and a new leaf-effort field.

### 4.5 Streaming text → actions
**NOT APPLICABLE.** The authoritative structured channel already exists: server `TOOL_CALL`/`TOOL_RESULT` (`execution.rs:1882-1937`), contract decoding (`web/src/api/contract.ts:887-940`), store state (`store.ts:1493-1560`), render (`ChatCore.tsx:273-307`, `AgentsPanel.tsx:65-72`). DeepSeek documents tool calls as structured `tool_calls`, not prose; parsing assistant text into buttons would fabricate actions the model never requested and duplicate the frozen event contract. The defect was the missing `tools` declaration (fixed mid-audit), not a missing text parser.

### 4.6 Provider-status refresh
- **Settings: WIRED.** `SettingsPanel.tsx:39-47,84-92` → `probeSettings` (`store.ts:2190-2212`, WS `SETTINGS_STATUS`) and `probeHealth` (`store.ts:2214-2234`, HTTP `/health`).
- **Models: MISSING** (no refresh at all).
- Best seam: one `providerStatus` slice in the store used by both surfaces; reuse the existing `fetchHealth`/`SETTINGS_STATUS` paths to avoid a second source of truth.

### 4.7 Cancel / retry / stop
- **Cancel run: WIRED** (`ControlBar.tsx:97-106` → `store.ts:1920-1927` → `ws.rs:763-768` → `execution.rs:1118-1130`).
- **Kill task: WIRED** (`ControlBar.tsx:107-116` → `store.ts:1929-1941`).
- **Stop in chat: MISSING.** `ChatCore.tsx:318-348` has no cancel control and no Esc handler.
- **Retry failed run: MISSING.** No `RETRY_RUN` (or equivalent) exists in the command contract (`web/src/api/contract.ts:104-145`; `hiverouter-contract/src/command.rs` has no retry command). Only approvals retry (`store.ts:2042-2068`). A failed run must be followed by a fresh `SUBMIT_PROMPT`, which the engine refuses while any run is active (`execution.rs:1032-1034`).

---

## 5. Dead / lying provider UI, missing controls, and tool-result risks

### 5.1 Now reachable, still unproven
- `ChatCore.tsx:273-307` "ACTIONS" card and `AgentsPanel.tsx:65-72` tool region depend on `TOOL_CALL` frames. They were dead on the pre-change bytes; they are *reachable* on the current bytes but unverified — no run has been observed producing a tool call through this adapter.

### 5.2 Misleading
- `SettingsPanel.tsx:69-74`: planner-only thinking/effort shown under global labels (§4.4).
- `ModelsPanel.tsx:62-65`: both proxies collapse into `provider metadata HTTP a/b`, discarding the named refusal and `how_to_fix` the handlers do return; the provider account is fetched even when the key is known missing.
- `ModelsPanel.tsx:27,208`: hard-coded `WIRE_MODELS` vocabulary in the browser.
- The planner root row is a structural task that never gets its own provider turn; it is set to the run outcome only at terminal reconciliation (`execution.rs:1216-1243`). The graph shows a "planner" agent whose status is derived, not executed.

### 5.3 Documentation overclaim
`docs/provider/DEEPSEEK-INTEGRATION.md:53` lists "Function tools + tool-call assembly + tool loop" as already integrated, and `:82` reports a UI prompt success. Neither exercises a real tool call. The document's own §6 gap list does not include this one.

### 5.4 Agents not receiving tool results / reasoning_content
- **Tool results:** implementation is sound — each turn's assistant tool-call message is followed by one `role=tool` message per bounded result with matching `tool_call_id` (`provider_executor.rs:356-391`), matching the documented loop. Reachability now depends on the unverified 18:19 fix.
- **`reasoning_content`:** captured (`:136-137,173-180,258-263`) and passed back only when non-empty (`:380-384`). The docs require it to be **fully** passed back for every request that carries `tools`, including turns that did not call a tool; otherwise HTTP 400. An empty/absent value is silently omitted instead of failing loudly. `delta_event` also drops `reasoning_content` from a delta that simultaneously carries `tool_calls` (`client.rs:748-763`; the fallback `:765-774` only runs when there is no content and no tool call). Latent with the default leaf thinking **off**; live the moment `HIVEROUTER_LEAF_THINKING=enabled` and a tool turn occurs.
- **Truncated generations:** current code rejects every `finish_reason` except `stop`/`tool_calls` (`provider_executor.rs:230-239`), closing the pre-change false-success hole.
- **New budget behaviour (18:19):** `estimate_prompt_tokens` serialises the request and divides bytes by 4 (`provider_executor.rs:418-424`) to make the 400K pre-send budget checkable. It is an assumption used only as a pre-send gate (provider usage remains the accounting truth, `accounting.rs:205-258`); it is conservative for ASCII prompts and can over-count JSON overhead.

---

## 6. Classification against documented DeepSeek behaviour

Labels are the requested four: **WIRED / PARTIAL / MISSING / NOT APPLICABLE**. "WIRED" here means the code path is connected; runtime proof is separately `UNVERIFIED`.

| # | Seam | Documented anchor | Verdict | Evidence |
|---|---|---|---|---|
| 1 | Prompt → planner `POST /chat/completions` (JSON mode, thinking, effort, max_tokens) | chat-completions body; JSON Output | **WIRED** | `provider_executor.rs:275-287`; `dto.rs:394-422` |
| 2 | Planner JSON → validated `Plan` | JSON Output (client must parse) | **WIRED** | `provider_executor.rs:292-295,434-481`; `planner.rs:50-100` |
| 3 | Plan → spawn task rows | — | **PARTIAL** | rows created (`execution.rs:1343-1369`) but global-index fan-out check refuses node 9 (`:1348`; `caps.rs:53-55`) |
| 4 | Leaf → streaming reply | SSE `data:`, `[DONE]`, usage in last chunk | **WIRED** | `client.rs:562-626,628-777`; `provider_executor.rs:106-264` |
| 5 | Leaf offers tools | tools / tool_choice | **WIRED (unverified)** | `provider_executor.rs:306-322` (18:19 fix); no test asserts it |
| 6 | Tool call → bounded result → model | tool-calls second-turn shape | **PARTIAL** | implementation correct (`provider_executor.rs:352-402`) but unproven end-to-end |
| 7 | `reasoning_content` pass-back | Thinking Mode §Tool Calls (mandatory with `tools`, else 400) | **PARTIAL** | conditional on non-empty (`:380-384`); delta drops it on tool-call deltas (`client.rs:748-763`) |
| 8 | Usage settle; absent ≠ 0 | usage required fields; last-chunk rule | **WIRED** | `provider_executor.rs:211-221`; `accounting.rs:205-258` |
| 9 | `finish_reason` handling | `stop/length/content_filter/tool_calls/insufficient_system_resource/aborted` | **WIRED** | `provider_executor.rs:222-239` (18:19 fix) |
| 10 | Cancel run | — | **WIRED** | §1.7 |
| 11 | Stop in chat | — | **MISSING** | `ChatCore.tsx:318-348` |
| 12 | Retry failed run | error codes are retryable classes | **MISSING** | no command (`contract.ts:104-145`) |
| 13 | Model catalogue proxy | `GET /models` | **WIRED** | `http.rs:850-866`; `client.rs:266-268` |
| 14 | Balance proxy | `GET /user/balance` | **WIRED** | `http.rs:869-885`; `client.rs:275-277` |
| 15 | Balance truthfulness (`is_available`) | `is_available` in response | **WIRED** | `ModelsPanel.tsx:68,74,184-186` (18:19 fix) |
| 16 | Proxy auth | — (local API precedent) | **MISSING** | `http.rs:496-497,850-885` vs `:967-983`, `ws.rs:364-387` |
| 17 | Proxy error mapping | 401/402/429/5xx semantics | **PARTIAL** | 401/402 collapse; timeout→502 (`http.rs:887-895`; `client.rs:507-525`) |
| 18 | Tools surface | — | **MISSING** | route `/diagnose` exists (`http.rs:504-526`), no UI consumer |
| 19 | Thinking/effort display | thinking / reasoning_effort | **PARTIAL** | planner scope only (`ws.rs:780-792`; `config.rs:39-68`) |
| 20 | Thinking/effort control | — | **MISSING** | env-only (`config.rs:102-121`) |
| 21 | Streaming text → actions | tool calls are structured | **NOT APPLICABLE** | server frames already exist; parsing text would fabricate |
| 22 | Provider-status refresh (Settings) | — | **WIRED** | `SettingsPanel.tsx:39-47,84-92`; `store.ts:2190-2234` |
| 23 | Provider-status refresh (Models) | — | **MISSING** | one-shot fetch (`ModelsPanel.tsx:55-86`) |
| 24 | Key never in browser/response/log | — | **WIRED** | §3 |
| 25 | Key availability at launch | — | **PARTIAL** | DPAPI path only for `HiveMind.exe`; `.cmd` ignores it; no reload (§2.2-2.3) |

---

## 7. P0 / P1 / P2 register

### P0 — none outstanding on the audited (current) bytes

**P0-1 (caught mid-audit, provisionally fixed): the production leaf turn did not offer tools.**
Pre-change: `ProviderExecutorAdapter::execute` built `ChatRequest::new(...).with_thinking(...).with_max_tokens(...)` with no `.with_tools(...)`; a conforming endpoint cannot return `tool_calls`, so the continuation loop and every tool/spawn/finish path was unreachable. Proof: uncommitted `git diff` on `provider_executor.rs` adds the missing `.with_tools(tools).with_tool_choice(ToolChoice::Auto)` (`provider_executor.rs:306-322`), and the pre-change test blind spot stands (`tests/tool_loop_engine.rs:125-177` fabricates tool calls; `tests/provider_live.rs:66-93` never asserts tools).
**Current status: fixed-but-unverified.** No Rust test asserts the request carries `tools`; no compile/run evidence exists in this audit. If this diff is not kept and tested, the P0 returns. Suggested proof: a wiremock test that serialises `execute`'s request and asserts `tools` + `tool_choice:auto`, plus one live/recorded reply that enters `execute_tool_turn`.

### P1

- **P1-1 — One mutex serialises every provider call.** `ProviderExecutorAdapter.budgets` (`provider_executor.rs:38`) is acquired in `complete` (`:82-85`, held to `:98`) and `stream_reply` (`:118-121`, held until the function returns at `:264`) and the borrow lives across the whole stream. One adapter instance (`http.rs:229-244,364`) makes it process-wide. The `InFlightLimiter(8)` (`budget.rs:43`; `client.rs:65,380,569`) is unreachable and a plan call for a second run blocks behind any live stream. The dispatch loop is also sequential (`execution.rs:1390-1420`), so agent fan-out is sequential by construction.
- **P1-2 — The 9th planned node always fails `E_FANOUT_LIMIT`.** `execution.rs:1346-1348` passes the global plan index as the `siblings` argument; `caps.rs:53-55` refuses at `>= 8`. The planner is prompted for 1..=40 nodes (`provider_executor.rs:28`) and the template says "at most 8 children" (`assets/prompts/planner.prompty:17`). Any DAG with more than 8 nodes fails before the first leaf.
- **P1-3 — `reasoning_content` pass-back is conditional and can be dropped.** See §5.4 and §6 row 7. Mandatory with `tools` per docs; omission is a 400. Live when leaf thinking is enabled (default off, `config.rs:62`).
- **P1-4 — No stop in chat and no retry-run.** Cancel exists only in `ControlBar` (§4.7); `ChatCore` has no stop; no retry command exists. After a failure the operator must submit a new prompt (blocked while a run is active, `execution.rs:1032`).
- **P1-5 — Key delivery differs by launch path and cannot be reloaded.** `HiveMind.exe` injects DPAPI; `HiveMind.cmd` ignores DPAPI; a reused server is never injected; unprotect failures are swallowed; the server reads the key only at boot; `/health` stays 200 "degraded"; there is no in-app provisioning or reload (§2.3).

### P2

- **P2-1 — Provider proxies have no session/origin gate** (`http.rs:496-497,850-885`) unlike `/api/session` (`:967-983`) and `/ws` (`ws.rs:364-387`). Key is not exposed; outbound account calls can be triggered by any local/cross-origin `GET`.
- **P2-2 — Proxy client churn / ignored config.** New `DeepSeekClient` + `InFlightLimiter` per request (`http.rs:851,870`; `client.rs:57-68`); `config.provider.enabled` ignored.
- **P2-3 — Models surface quality.** One-shot fetch, no refresh/retry, opaque `provider metadata HTTP a/b` error (`ModelsPanel.tsx:55-86`), account fetched even when the key is known missing, hard-coded `WIRE_MODELS` vocabulary (`:27,208`).
- **P2-4 — Settings scope label.** Thinking/effort are planner values (`ws.rs:780-792`) under global labels; leaf thinking default off (`config.rs:62`); no leaf effort field (`config.rs:39-54`).
- **P2-5 — JSON-mode blank content is terminal.** Docs: JSON mode may return spaces-only content with `finish_reason: stop` and recommend handling/retrying; `message_text` now returns `E_PROVIDER_INCOMPLETE` for non-success reasons (`provider_executor.rs:435-449`) but a `stop` with whitespace-only content still returns `E_PROVIDER_INVALID_RESPONSE` (`:450-470`), and `complete` will not retry a non-retryable error (`client.rs:405-407,596-598`).
- **P2-6 — Remaining error-mapping gaps.** 401 and 402 collapse to `E_PROVIDER_AUTH` (`client.rs:508-511`); proxy timeout → 502 not 504 (`http.rs:887-895`); 400/422 still map to `E_PROVIDER_INVALID_RESPONSE` (`client.rs:520-523`) rather than a schema/parameter refusal. 5xx retry was fixed mid-audit (`client.rs:516-519`).
- **P2-7 — Cancelled streams can hold an in-flight permit for up to 600 s.** The pump task keeps the permit (`client.rs:583-588`) until it next tries to send; on a quiet stream after cancel it waits out `stream_idle_timeout` (`:63,639-651`).
- **P2-8 — `caps.check_calls(1)` is a no-op.** `execution.rs:1313,2765` always pass 1 against a 40 cap (`caps.rs:66-71`); the effective call bound is the `RunBudget` cap (`budget.rs:31,243`).
- **P2-9 — Tool counters are run-wide, not task-wide.** `execution.rs:1584-1588` reads run counters, so a second task's first tool turn starts at the run's current turn count.
- **P2-10 — Planner prompt contradiction / flattening.** PLANNER_SYSTEM says "1..=40 nodes" (`provider_executor.rs:28`), the template says "at most 8 children" (`planner.prompty:17`), the runtime allows 8; the template's system/user sections are flattened into user content (`execution.rs:1550-1555`).
- **P2-11 — Continuation text merges into one streaming bubble.** `store.ts:1402-1419` appends every `REPLY_DELTA` for the run to the same streaming message, so "let me check …" text and the final answer concatenate with no separator.
- **P2-12 — Planner root status is derived.** The root row never executes a turn and is set to the run outcome only at terminal reconciliation (`execution.rs:1216-1243`).

---

## 8. Verified / unverified

- **Verified by file read:** every file:line cited above; current hashes/mtimes in the revision note; pre-change diff via `git diff`.
- **Verified against official docs:** model ids, `thinking`/`reasoning_effort` semantics and defaults, `max_tokens` 1..393216, `response_format` blank-content behaviour, tools/`tool_choice` rules (required/named are 400 in thinking mode), `reasoning_content` pass-back rule, streaming usage in the last chunk, usage field names, `GET /models`, `GET /user/balance`, 500/503 retry guidance.
- **UNVERIFIED:** runtime behaviour of the whole path (no app run, no live provider call, no key read); whether the 18:19 `execute` fix compiles and produces a tool call; whether the concurrent edits are complete or will change again. **No test was added for the P0 fix** — `git status` shows no Rust test change.
- **Not claimed:** no key material was read or printed; no product source was modified by this audit; the uncommitted working tree may move under these line numbers.

## 9. Coverage manifest

- **Read in full:** `hiverouter-provider/src/{client.rs,dto.rs,error.rs,secret.rs}`, `hiverouter-core/src/{provider_executor.rs,config.rs,checks.rs,tool_loop.rs,planner.rs,caps.rs,main.rs}`, `hiverouter-tools/src/settings.rs`, `hiverouter-core/assets/prompts/planner.prompty`, `web/src/chat/ChatCore.tsx`, `web/src/panels/{ModelsPanel,SettingsPanel,AgentsPanel,ControlBar}.tsx`, `tools/desktop-launcher/HiveMindLauncher.cs`, `tools/{start-hivemind.ps1,set-provider-key.ps1}`, `docs/provider/DEEPSEEK-INTEGRATION.md`.
- **Re-read after the 18:19 changes:** `provider_executor.rs` (current 647 lines: 70-144, 195-279, 375-474), `client.rs` (current 868 lines: 505-544 plus the full uncommitted diff), `ModelsPanel.tsx` (current 226 lines: 40-226).
- **Read in targeted regions (provider/runtime relevant):** `hiverouter-provider/src/budget.rs` (1-509 of 710), `accounting.rs` (1-379 of 469), `pricing.rs` (1-160 of 273); `hiverouter-core/src/execution.rs` (118-339, 645-735, 980-1174, 1210-1774, 1762-1951, 2600-3053), `http.rs` (1-80, 210-330, 330-410, 455-555, 555-685, 760-1065), `ws.rs` (360-430, 743-805); `web/src/state/store.ts` (200-320, 1280-1730, 1810-1950, 1950-2070, 2180-2279), `web/src/api/{contract.ts,agentState.ts}`, `web/src/state/strings.ts` (140-250).
- **Not read:** approval/suspension internals (`execution.rs` 1952-2600), memory/history handlers, most UI panels unrelated to provider wiring.
