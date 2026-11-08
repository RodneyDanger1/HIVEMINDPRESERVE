# 04 — DeepSeek key integration & provider account endpoints — independent audit

**Auditor:** independent subagent (read-only pass; this file is the only artifact written)
**Date/time:** 2026-11-07, 18:07–18:35 America/Chicago
**Target:** portable app at `http://127.0.0.1:8099`
**Repository under audit:** `C:\Users\Riley\Desktop\hiverouter-v1` (working tree, not a commit)
**Binaries observed live:** `C:\Users\Riley\Desktop\HiveMind-Portable\HiveMind.exe` (built 18:00:51) and `hiverouter.exe` (built 18:00:49)
**Server process:** `hiverouter.exe --bind 127.0.0.1:8099 --data-dir "C:\Users\Riley\AppData\Local\HiveMind"`
**Constraints honoured:** no product source, tests, git state, or environment variables were edited; the app was never restarted or killed by this audit; the DeepSeek key value was never read, decrypted, or printed.

> **Snapshot warning (important).** The working tree changed while this audit ran. At 18:08 the uncommitted set was the provider/client/http/dto/lib changes plus UI and launcher changes. By 18:30 it additionally included `hiverouter-core/src/execution.rs`, `hiverouter-core/src/provider_executor.rs`, and `hiverouter-provider/tests/refusal_matrix.rs` (mtimes 18:25–18:26), and `client.rs` had been edited again at 18:19. Those edits were not made by this audit. The running portable binary was built at 18:00, before the later edits, so **live behaviour below describes the 18:00 binary**; source citations describe the 18:30 working tree. This is called out wherever it matters.
>
> **Post-audit addendum (18:35–18:36).** The portable app was externally relaunched at 18:33:37 (`hiverouter` PID 15572) and was up at 18:36. The run set grew from 1 (18:19) to 5 (18:36): four runs were created 00:25–00:34Z by external activity (two completed, two cancelled), none by this audit. The `HIVE_UI_OK` run remains but is no longer the latest; the UI then displayed a newer completed run (a `file_read` prompt) with an assistant reply. Provider models were unchanged; balance re-probed at 18:35:30 was USD **8.74** (it was 9.17 at 18:20, consistent with the external runs using the account). The 18:19 snapshot below is retained as the primary keyed-UI evidence; this addendum records the moving environment.

---

## Verdict

**PASS-WITH-NOTES.**

The DeepSeek key is integrated and working end-to-end on the running portable app: the live `/health` reports the key present; `/api/provider/models` and `/api/provider/balance` return real account data; no key material was found in HTTP bodies, HTTP headers, the browser DOM, the served JS bundle, tracked files, runtime logs, the SQLite DB/WAL, or the DPAPI file; and the session/WebSocket auth checks behave as documented.

Notes / defects (details in §8):

1. **Security (Low–Medium):** the two new provider metadata endpoints are unauthenticated and ignore `Origin`. Any local process can read the account metadata, and any web page can **blind-trigger** the two fixed outbound DeepSeek calls (no response read because no CORS header). No key exposure, no cost, no arbitrary path.
2. **Information disclosure (Low):** `/health` returns the absolute database path and key-presence state to unauthenticated local callers.
3. **Error-code semantics (Low, in-flight):** the current working tree maps transient provider HTTP 5xx to `E_PROVIDER_TIMEOUT` (retryable). That is retryable but semantically misleading; the live 18:00 binary predates this edit.
4. **UI correctness (Low):** the Models panel ignores `is_available` from the balance payload, so an unavailable account still renders balance rows without an explicit “not available” flag.
5. **Operability (Low):** the launcher silently falls back to a process environment variable if DPAPI decryption fails (empty `catch`), and `/health`/the 503 message do not say which source supplied the key.
6. **Hygiene (Informational):** one tracked test fixture at `backend/crates/hiverouter-contract/src/event.rs:990` is key-shaped (masked `xx-xxxx-…`, 34 chars) and is synthetic; it is not the production key.
7. **Coverage (Low):** the new `/api/provider/*` routes and the new Models panel section have no dedicated automated tests; they were verified live only.

Nothing found rises to **FAIL**: there is no key disclosure, no fabricated provider data, no auth bypass on the WebSocket/chat path, and no evidence that the browser can make arbitrary provider calls.

---

## Method and limits

- Live HTTP probes with `curl.exe` / PowerShell `Invoke-WebRequest`, including deliberate bad `Origin` headers and a wrong WebSocket token.
- A headless Chromium session via the repository’s Playwright install (bundled browser, separate profile) opened the app, read the chat DOM, clicked the Models nav item, and captured provider network responses. No prompt was submitted and no paid run was created.
- Two short-lived **ephemeral** `hiverouter.exe` attempts were started on `127.0.0.1:8096` with an empty temporary data directory and **no key** to exercise the key-missing path; each was stopped and its temp directory removed. These were the only processes the audit stopped, and neither touched the portable app on 8099.
- Static read of routes/handlers, provider client/error/DTO/secret code, session/WS auth, launcher/wrapper source, and web source/bundle.
- Key-handling checks were **pattern/boolean only**: file size/magic/printable ratios, presence of the ASCII prefix `sk-`, key-shaped regex counts, and source/history inspection. The DPAPI blob was not decrypted and the key value was never compared to any file.

**Availability during audit.** The portable app was observed up from 18:07:59 and went down between probes; both PIDs (`HiveMind.exe`, `hiverouter.exe`) were gone and port 8099 was closed at 18:12:15. It was relaunched externally at 18:17:56 (new PIDs 5432/2744), served the 18:18–18:20 checks, and was gone again by 18:29. It was externally relaunched again at 18:33:37 (PID 15572) and was up at 18:36. This audit did **not** restart or kill the portable app. The exits are consistent with the launcher’s designed behaviour (“closing the app window stops the server”, forced child kill, no shutdown log), but the cause was not directly observed; see claims register.

---

## 1. `/health` — key check and status

### 1.1 Live keyed instance (18:18, HTTP 200)

Exact JSON (no secrets present):

```json
{"status":"ok","version":"1.2.0-dev","schema":"hiverouter.v1","uptime_secs":60,"checks":[{"name":"rustc","required":true,"ok":true,"detail":"rustc 1.99.1 (58cf8f9c3 2026-10-13) (MSRV 1.89.0)","code":null},{"name":"node","required":false,"ok":true,"detail":"v26.7.0 (>= 22.12.0)","code":null},{"name":"deepseek_api_key","required":false,"ok":true,"detail":"DEEPSEEK_API_KEY is present (value never logged)","code":null}],"database":{"path":"C:\\Users\\Riley\\AppData\\Local\\HiveMind\\hiverouter.db","schema_version":10,"error_code":null},"transport":{"last_seq":2,"graph_nodes":2,"allowed_origins":2},"prompt_assets":{"loaded":true,"error_code":null}}
```

Key check fields: `required:false`, `ok:true`, `code:null`, `detail:"DEEPSEEK_API_KEY is present (value never logged)"`. Overall `status:"ok"`, HTTP 200.

### 1.2 Key-missing instance (negative path, HTTP 200)

A second ephemeral instance with no key returned:

- HTTP **200**
- overall `status:"degraded"`
- `deepseek_api_key`: `required:false`, `ok:false`, `code:"E_KEY_MISSING"`, `detail:"DEEPSEEK_API_KEY is not set; live prompts will refuse E_KEY_MISSING"`

The earlier boot log also records the false case before the DPAPI key was installed: `audit.jsonl.2026-11-07` at `2026-11-07T23:37:02.908343Z` has `"check":"deepseek_api_key","detail":"DEEPSEEK_API_KEY is not set; live prompts will refuse E_KEY_MISSING","code":"E_KEY_MISSING"`.

**Behavioural note:** the key check is `required:false`. A missing key does **not** make `/health` 503; it makes the status `degraded` while HTTP stays 200. That is consistent with the code (`required_ok && all_ok` → `ok`; `required_ok` only → `degraded`; required failure → 503), but an operator must not read HTTP 200 as “key present”. The explicit `checks[].ok` / `code` fields are the signal.

**Classification:** live true/ok — **VERIFIED**; false/degraded — **VERIFIED** (ephemeral instance) and **READ** (audit log); status logic — **READ** (`backend/crates/hiverouter-core/src/http.rs:914-964`, `checks.rs:140-161`).

---

## 2. `/api/provider/models` and `/api/provider/balance`

### 2.1 Live responses (final probe 2026-11-07T18:20:09-06:00)

`GET /api/provider/models` — HTTP **200**, `Content-Type: application/json`, body 313 bytes:

```json
{"object":"list","models":[{"id":"deepseek-flash","object":"model","owned_by":"deepseek","name":"DeepSeek-V4.1-Flash","context_window":1048576,"max_output_tokens":393216},{"id":"deepseek-v4-pro","object":"model","owned_by":"deepseek","name":"DeepSeek-V4-Pro","context_window":1048576,"max_output_tokens":393216}]}
```

`GET /api/provider/balance` — HTTP **200**, `Content-Type: application/json`, body 128 bytes:

```json
{"is_available":true,"balances":[{"currency":"USD","total_balance":"9.17","granted_balance":"0.00","topped_up_balance":"9.17"}]}
```

Model IDs observed: `deepseek-flash`, `deepseek-v4-pro`. Currency: `USD`; available: `true`; total `9.17`, granted `0.00`, topped-up `9.17`. A probe roughly 13 minutes earlier returned `9.50`; the account balance is live and changed during the audit (consistent with real account activity, not a fixed fixture). A later post-audit probe at 18:35:30 returned `8.74` (see the addendum; external runs used the account).

Response headers observed on both endpoints: only `Content-Length`, `Content-Type: application/json`, and `Date`. No `Authorization`, no `Set-Cookie`, no CORS header. Body and header scans for the ASCII prefix `sk-` were false; no key-like token was present.

### 2.2 Auth/origin behaviour of these endpoints

The endpoints are **not** behind the session/origin gate:

| Probe | HTTP | `Access-Control-Allow-Origin` | Notes |
|---|---:|---|---|
| `/api/provider/models` with `Origin: http://evil.example` | 200 | absent | request processed, response unreadable to cross-origin JS |
| `/api/provider/balance` with `Origin: http://evil.example` | 200 | absent | same |
| `/api/session` with `Origin: http://evil.example` | 403 | — | `E_ORIGIN_REJECTED` |
| `/api/session` with no `Origin`, loopback peer | 200 | absent | returns a session token (not printed) |
| `/api/provider/nope` | 404 | — | empty body; no static-UI fallback for this path |

The `Origin`-bearing 200s are the basis for defect D1. A cross-origin page cannot read the JSON (no `Access-Control-Allow-Origin`), but the browser still sends the GET and the server still calls DeepSeek with the key. Any local process can read the JSON directly. There is no caching/rate-limit layer on these two routes (`Cache-Control` absent; every Models mount issues both calls).

### 2.3 Key-missing behaviour

On the ephemeral no-key instance both endpoints returned HTTP **503** with the same refusal:

```json
{"code":"E_KEY_MISSING","message":"No DeepSeek API key is configured.","how_to_fix":"Set the DEEPSEEK_API_KEY environment variable and restart the server.","retryable":false}
```

No `sk-` in the body. This verifies the 503 mapping for `E_KEY_MISSING`. The remaining mappings are source-read: `E_PROVIDER_AUTH` → 401, `E_PROVIDER_RATE_LIMIT` → 429, other errors → 502 (`http.rs:887-895`). Provider auth 401/402 is not retried in the client (`client.rs:492-526`).

**Classification:** responses/ids/balance — **VERIFIED**; no key in bodies/headers — **VERIFIED**; 503 key-missing — **VERIFIED**; 401/429/502 mappings — **READ**; unauthenticated/origin-ignoring behaviour — **VERIFIED**.

---

## 3. Backend routes/handlers and client code — auth, error mapping, secret handling, arbitrary-call surface

### 3.1 Route surface (read)

`build_router` (`backend/crates/hiverouter-core/src/http.rs:477-501`) exposes:

- `/health`, `/diagnose`, `/api/session`
- history/agents/usage/projects read APIs
- `GET /api/provider/models`, `GET /api/provider/balance`
- `GET /ws`
- fallback static UI

The only two provider-account handlers take **no** `State`, token, or origin parameter (`http.rs:846-885`). They build `DeepSeekClient::from_env(ClientConfig::default())` per request and call hard-coded `client.list_models()` / `client.balance()`. No request field, query, path segment, or header controls the provider URL, method, path, or body.

### 3.2 Session and WebSocket auth (live)

`GET /api/session` with an allowlisted origin (or no origin on a loopback peer) returns `{token, schema, last_seq, ws}`; token value not printed. `GET /api/session` with `Origin: http://evil.example` returned **403**:

```json
{"code":"E_ORIGIN_REJECTED","message":"The WebSocket Origin header is not on the localhost allowlist.","how_to_fix":"Connect from http://127.0.0.1:<port> or http://localhost:<port>; do not proxy the UI from another origin.","retryable":false}
```

WebSocket `/ws` probes (handshake only, no prompt):

| Origin | Token | HTTP | Refusal |
|---|---|---:|---|
| `http://evil.example` | `wrong` | 403 | `E_ORIGIN_REJECTED` |
| `http://127.0.0.1:8099` | `wrong` | 401 | `E_TOKEN_INVALID` |
| absent | `wrong` | 403 | `E_ORIGIN_REJECTED` |

`E_TOKEN_INVALID` body:

```json
{"code":"E_TOKEN_INVALID","message":"The per-session WebSocket token is missing or does not match.","how_to_fix":"Reload the UI so it fetches a fresh session token from the local server.","retryable":false}
```

The handler order is loopback peer → `Origin` allowlist → token (`backend/crates/hiverouter-core/src/ws.rs:359-387`). Missing `Origin` is never accepted for a WebSocket upgrade (`ws.rs:72-83`). This is consistent with the live probes.

### 3.3 Can the browser cause arbitrary provider calls?

**No arbitrary URL/path/model.** Findings:

- `SubmitPrompt` payload is only `{text, project_id}` with `text` 1..=8000 chars (`backend/crates/hiverouter-contract/src/command.rs:166-173`). There is no command to set the provider, model, base URL, key, or arbitrary request.
- The model comes from server `ProviderSettings` at boot (`config.rs:77-120`); the base URL comes from `ClientConfig::default()` (`client.rs:40-75`, `DEFAULT_BASE_URL`), not from a browser field. The provider adapter builds `ChatRequest::new(self.settings.model, …)` (`provider_executor.rs:394`).
- Chat is reachable only through `/ws` with the origin+token checks above; the HTTP router has no prompt/chat POST route.
- The two provider metadata routes are fixed GETs. Unknown `/api/provider/*` paths 404.

**What the browser can cause:** (a) the intended chat prompt through the authenticated WS (bounded by run budget/limits), and (b) blind, unauthenticated GETs to the two fixed metadata endpoints from any origin (D1). It cannot choose a provider endpoint, inject an arbitrary URL, or read the metadata cross-origin.

### 3.4 Secret handling (read + live scans)

- `ApiKey` stores `Zeroizing<String>`, `Debug` is `ApiKey(***)`, and the only accessor is `pub(crate) expose()`, used solely to build the `Authorization: Bearer` header (`backend/crates/hiverouter-provider/src/secret.rs:20-81`; `client.rs:329-339`). `present_in_env()` returns a boolean only.
- Provider errors carry a refusal code plus static/status-derived detail, never request headers or bodies (`provider/error.rs:1-124`). The HTTP refusal response emits only `code`, `message`, `how_to_fix`, `retryable` (`ws.rs:391-399`; `http.rs:887-895`). Account parse errors add only JSON line/column and body byte count — not body content (`client.rs:300-312`).
- The live `/health`, models, balance, 403, 401, and 503 bodies were scanned for `sk-`: all false. Provider response headers were scanned: none.
- The served web source/bundle was scanned for `DEEPSEEK`/`sk-`: only labels/error copy and CSS false positives (`mask-…`), no key. The live DOM scan for `sk-` was false.
- The launcher injects the key only via the child’s `ProcessStartInfo.EnvironmentVariables`; it is not on the command line, in a log, or in browser JS (`tools/desktop-launcher/HiveMindLauncher.cs:123-160`). Observed process command lines contained only `--bind` and `--data-dir`.

### 3.5 Current-tree error-mapping caveat (read; not in live binary)

At 18:30 the working tree maps provider HTTP `500 | 502 | 503 | 504` to `Refusal::ProviderTimeout` (`client.rs:516-518`) and the corresponding test was renamed to `transient_500_is_retried_then_refused_by_name` (`tests/refusal_matrix.rs:172-184`). That is retryable, but labelling an HTTP 500/503 as `E_PROVIDER_TIMEOUT` is semantically misleading for operators. The portable binary (built 18:00) predates this edit; no live 5xx path was exercised. Flagged as D3, not as a live failure.

**Classification:** route surface — **READ**; session/WS auth — **VERIFIED**; no arbitrary provider calls — **VERIFIED** (route/contract inspection + live auth probes); secret-handling code — **READ**; no key in responses/DOM/bundle — **VERIFIED**; 5xx mapping — **READ** (current tree only).

---

## 4. DPAPI key file

Path: `%LOCALAPPDATA%\HiveMind\provider.key` (`C:\Users\Riley\AppData\Local\HiveMind\provider.key`).

Observed without reading the key:

| Property | Observed |
|---|---|
| Exists | `true` |
| Size | 262 bytes |
| First 8 bytes | `01000000d08c9ddf` — Windows DPAPI blob signature prefix |
| ASCII `sk-` anywhere in file | `false` |
| ASCII `Bearer` anywhere in file | `false` |
| Printable-ASCII ratio | `0.2901` |
| Trimmed content looks like a raw key (`^sk-[A-Za-z0-9]{16,}$`) | `false` |
| Last write (UTC) | `2026-11-07T23:48:21.2941213Z` |

A raw DeepSeek key would be a short ASCII string and would contain the `sk-` prefix; this file is a binary DPAPI blob. No `provider.key` copy was found under `C:\Users\Riley\Desktop` (search excluded `node_modules`, `target`, `.git`); the portable folder contains only `HiveMind.exe`, `hiverouter.exe`, and `README.txt`.

**Source path:** the launcher calls `ProtectedData.Unprotect(encrypted, UTF8("HiveMind.ProviderKey.v1"), DataProtectionScope.CurrentUser)` and injects the plaintext into the `hiverouter.exe` child environment only (`HiveMindLauncher.cs:128-160`); the key-creation script uses `ProtectedData.Protect` with hidden `Read-Host` input and the same entropy (`tools/set-provider-key.ps1:20-39`). The current launcher process (PID 5432) and child (2744, observed before the second exit) had no key on their command lines.

**Environment scopes:** in the auditor shell, `Process`, `User`, and `Machine` scopes for `DEEPSEEK_API_KEY` were all absent. The child reported the key present via `/health`, and the only configured key store is the DPAPI file. Therefore DPAPI decryption + injection is the **only plausible configured source**, but the child environment itself was not read (doing so would risk exposing the key). The causal claim is **BELIEVED**, not directly observed; the file properties are **VERIFIED**; the launcher decrypt/inject code is **READ**.

**Threat-model note:** `DataProtectionScope.CurrentUser` plus a hard-coded entropy string (present in source/README) protects against other Windows users and offline copies, not against a process already running as the same user. That is the standard DPAPI CurrentUser model, not a defect in this integration.

---

## 5. Plaintext-key scan (counts only)

All searches were for the literal prefix / key-shaped patterns; no values are reproduced.

| Scan | Scope | Result |
|---|---|---|
| `git grep -I -F -c -- 'sk-'` | tracked text files | **97 files, 242 matching lines** (mostly ordinary substrings such as `mask-`, `task-`, `risk-`, plus clearly synthetic test constants) |
| `git grep -I -E -c -- 'sk-[A-Za-z0-9]{24,}'` | tracked text files | **0 matches** |
| `git grep -I -E -c -- 'sk-[A-Fa-f0-9]{32}'` | tracked text files | **0 matches** |
| Word-boundary broad key-shape `(?<![A-Za-z0-9])sk-[A-Za-z0-9_-]{30,}` | tracked text files | **1 match**: `backend/crates/hiverouter-contract/src/event.rs:990`, a synthetic redaction-test fixture (masked shape `xx-xxxx-…`, 34 chars; committed 2026-11-06, before the DPAPI key file was created 2026-11-07 17:48). Not the production key. |
| Same broad scan | working-tree text files under `backend`, `web/src`, `web/tests`, `docs`, `tools`, `e2e` (515 files) | same single synthetic fixture; **0** other matches |
| ASCII key-like regex | `HiveMind.exe`, `hiverouter.exe` (portable), repo `runtime-test`/`tools` binaries | **0** key-like, **0** 32-hex |
| `sk-` / key-like | `hiverouter.db`, `-wal`, `-shm`, `provider.key`, audit logs | **0** |
| `.env` files | working tree | **none found** |
| Web bundle | `web/src`, `web/dist` | only error copy and CSS false positives; **0** key-like |

The single synthetic fixture is the only key-shaped literal in the source/docs scan. Its placeholder nature, its commit date (2026-11-06 23:41) versus the key file creation (2026-11-07 17:48), and its use in an arguments-redaction test all support “test fixture, not production key”. It is still worth renaming to an obviously invalid prefix because secret scanners will flag it (D6).

**Exact-key absence is not proven by value comparison** because the audit was forbidden to read the key. The evidence supports “no DeepSeek-shaped plaintext key is present in tracked/working-tree/served/runtime artifacts”; it does not support a claim that a specific secret string was compared byte-for-byte.

---

## 6. Tests and typecheck

### 6.1 Web

Final snapshot (2026-11-07 18:31):

- `npm run typecheck` (`tsc --noEmit`): **exit 0**
- `npm test` (vitest): **23 files passed, 186 tests passed, 0 failed, 0 skipped; exit 0**

No dedicated web test references `ModelsPanel`, `/api/provider/models`, or `/api/provider/balance` (`web/tests` grep returned none). The new Models-account surface is verified by live DOM only (D7).

### 6.2 Cargo provider tests

Final snapshot (after the concurrent edit settled, 18:31):

`cargo test -p hiverouter-provider --locked --lib --tests` → **exit 0**; 8 test binaries; **99 passed, 0 failed**, 0 ignored; 0 doc tests:

- 48 unit tests (`src/lib.rs` modules)
- 9 `error_matrix`
- 11 `refusal_matrix`
- 8 `streaming`
- 3 `tool_assembler`
- 5 `tool_call_transport`
- 6 `tool_continuation`
- 9 `tool_conversation`

Earlier snapshot at 18:15 (same command, before the concurrent edits): also **99/99 passed**.

During the concurrent edit window (18:19–18:26) one run reported **98/99**: `undocumented_status_is_refused_by_name` expected `E_PROVIDER_INVALID_RESPONSE` but got `E_PROVIDER_TIMEOUT`, because `client.rs` was being changed to retry 5xx while the test still expected the old mapping. After the test was renamed/updated at 18:26 and the edit settled, the final run above is **99/99**. This was a moving-tree artefact, not a reproducible product defect; the same audit did not edit either file.

### 6.3 Relevant test coverage (read)

Provider tests cover auth 401/402 non-retry, 429/Retry-After, transport timeout, malformed JSON, usage/incomplete, DTO parsing, and key redaction. They do **not** cover the new HTTP `/api/provider/models` and `/api/provider/balance` routes, their origin/auth behaviour, or the Models panel DOM. D7.

---

## 7. Keyed UI path

### 7.1 Models page DOM

Headless Chromium (Playwright, bundled browser, separate profile) loaded `http://127.0.0.1:8099/`, then clicked the `Models` nav button. The `section[aria-label="Provider account"]` text was:

```text
PROVIDER ACCOUNT
BALANCE
USD
9.17
AVAILABLE MODELS
deepseek-flash
DEEPSEEK-V4.1-FLASH · 1048576 CTX · 393216 MAX OUT
deepseek-v4-pro
DEEPSEEK-V4-PRO · 1048576 CTX · 393216 MAX OUT
```

Network during the Models visit: `GET /api/provider/models` → 200, 313 bytes; `GET /api/provider/balance` → 200, 128 bytes; both `application/json`; no `sk-`; no console errors. Clicking Models did not submit a prompt and did not create a run; no chat provider call was observed.

### 7.2 Existing run / assistant reply

`GET /api/history/runs` returned one run:

```json
{"runs":[{"run_id":"01a1b86d-8b15-748e-abde-2c97f347d90c","project_id":"01a1b86d-6f38-7377-ab83-1a2a1f5b147e","status":"completed","error_code":null,"created_at":"2026-11-07T23:52:29Z"}]}
```

`GET /api/history/runs/<id>/messages` returned the persisted transcript:

```json
{"run_id":"01a1b86d-8b15-748e-abde-2c97f347d90c","messages":[{"id":"01a1b86d-8b17-728c-a4b9-99a1c907efaf","run_id":"01a1b86d-8b15-748e-abde-2c97f347d90c","role":"user","text":"Reply with exactly HIVE_UI_OK","created_at":"2026-11-07T23:52:29Z"},{"id":"01a1b86d-952b-71f9-94ec-76c3af92f131","run_id":"01a1b86d-8b15-748e-abde-2c97f347d90c","role":"assistant","text":"HIVE_UI_OK","created_at":"2026-11-07T23:52:31Z"}],"truncated":false}
```

`GET /api/usage/runs/<id>` returned real usage: `prompt_tokens 88`, `completion_tokens 5`, `total_tokens 93`, `prompt_cache_hit_tokens 0`, `prompt_cache_miss_tokens 88`, `reasoning_tokens null`.

The live UI body contained both the user prompt and `HIVE_UI_OK`; a Playwright exact-text locator for `HIVE_UI_OK` counted **1**. This run predates the audit (created 23:52:29Z / 17:52 local, before the 18:07 audit start). **No new paid run was created by this audit.** The app exited again before a second, article-level DOM count could be taken at that point; the body/locator evidence above is from the 18:19 session.

**18:36 addendum.** After the external 18:33 relaunch, a Playwright page load found **2** `article.chat-message` elements and the latest completed run’s assistant reply: the user prompt was “Use the file_read tool to read README.txt, then report its first line exactly. Do not guess.” and the assistant answered with the first line plus a note that it was read with `file_read`. The audit log independently records a real `tool_call` row for that run (`tool:"file_read"`, `outcome:"ok"`, `2026-11-08T00:34:03.189094Z`), so this is not a fabricated tool claim. This confirms that the keyed UI path still renders an assistant reply; the `HIVE_UI_OK` run above is no longer the latest. The run count changed from 1 to 5 between 18:19 and 18:36 due to external runs, not this audit (no send/click-submit occurred).

**Classification:** Models DOM balance+models — **VERIFIED**; assistant reply appears — **VERIFIED** (body + exact locator count 1); run/usage data — **VERIFIED**; no new run created — **VERIFIED** by actions/network (no submit, only GETs) and the one-run history observed before/after the UI visit.

---

## 8. Defects and notes

### D1 — Provider metadata endpoints are unauthenticated and ignore Origin (Security, Low–Medium)

**Evidence:** `/api/provider/models` and `/api/provider/balance` returned 200 with `Origin: http://evil.example`, no `Access-Control-Allow-Origin`. They have no token, peer, or origin check (`http.rs:846-885`), unlike `/api/session` and `/ws`.

**Impact:** any local process can read account metadata; any web page can blind-trigger the two fixed outbound DeepSeek calls (the response is unreadable to cross-origin JS, but the request is processed). No key exposure, no charge, no arbitrary path. Repeated calls could hit provider rate limits and create network noise; a DNS-rebinding page could become same-origin and read the balance/model data (not directly exercised in this audit).

**Recommendation:** require the session token or apply the same origin gate in front of these handlers; add `Cache-Control: no-store`; consider a short server-side cache to avoid a provider call on every Models mount; add rate limiting.

### D2 — `/health` exposes absolute DB path and key-presence state (Information disclosure, Low)

**Evidence:** live health body includes `database.path` (`C:\Users\Riley\AppData\Local\HiveMind\hiverouter.db`) and the `deepseek_api_key` check. `/health` is unauthenticated and unconditionally served. `/api/history/*`, `/api/usage/*`, and `/api/projects` are also unauthenticated loopback reads.

**Impact:** local information disclosure (paths, key presence, prompts/history). Not key material. Low for a loopback-only local app, but worth aligning with the session/origin policy if the local threat model includes other browser tabs or DNS rebinding.

### D3 — Transient provider 5xx is labelled `E_PROVIDER_TIMEOUT` in the current tree (Correctness/UX, Low; in-flight)

**Evidence:** current `client.rs:516-518` maps `500 | 502 | 503 | 504` to `Refusal::ProviderTimeout`; the current test expects `E_PROVIDER_TIMEOUT` and `retryable=true` (`tests/refusal_matrix.rs:172-184`). The live 18:00 binary predates this edit. An HTTP 500/503 is a server failure, not necessarily a timeout; `E_*` codes are operator-facing.

**Recommendation:** use a distinct retryable `E_PROVIDER_UNAVAILABLE`/`E_PROVIDER_SERVER` (or keep `E_PROVIDER_INVALID_RESPONSE` with `retryable=true`), and update the test accordingly.

### D4 — Models panel ignores `is_available` (UI correctness, Low)

**Evidence:** `/api/provider/balance` returns `is_available:true`; `ModelsPanel.tsx` types/stores only `currency`, `total_balance`, `granted_balance`, `topped_up_balance` and renders rows regardless of `is_available`. If the account becomes unavailable, the UI still shows “Balance” rows without an explicit unavailable flag.

**Recommendation:** surface an “account unavailable / cannot make calls” state when `is_available` is false; do not imply usable balance.

### D5 — Launcher silently falls back on DPAPI failure (Operability/Security hygiene, Low)

**Evidence:** `HiveMindLauncher.cs:137-160` wraps the whole load in `try { … } catch { }` and then returns `Environment.GetEnvironmentVariable("DEEPSEEK_API_KEY") ?? ""`. A decrypt/permission/entropy failure is indistinguishable from “no file”; `/health` reports only present/absent, not the source.

**Impact:** an operator may see key-present or key-missing without knowing whether DPAPI was used; a stale user/process env key could be used silently. No key is logged.

**Recommendation:** report a named warning (no key material) on DPAPI failure; do not silently fall back (or make the fallback explicit/opt-in); include the source in the health/log line without the value.

Also note: the 503 `how_to_fix` says “Set the DEEPSEEK_API_KEY environment variable”, which is correct for the server process but does not mention the production `tools/set-provider-key.ps1` / DPAPI path. Low UX inconsistency.

### D6 — Key-shaped synthetic fixture in a tracked test (Hygiene, Informational)

**Evidence:** `backend/crates/hiverouter-contract/src/event.rs:990` contains a 34-char key-shaped literal used by `args_preview_redacts_secret_keys_and_secret_shaped_values`. It is synthetic, committed 2026-11-06 (before the DPAPI key was created), and is not the production key. It is the only key-shaped literal found in the tracked/working-tree scans.

**Impact:** secret scanners and future auditors will flag it; it trains reviewers to ignore key-shaped matches.

**Recommendation:** replace with an obviously invalid placeholder (e.g. a `TEST-…` / `example-…` value) or keep it only behind a scanner allowlist with a comment.

### D7 — No dedicated tests for the new provider routes/UI (Coverage, Low)

**Evidence:** no `web/tests` reference to `ModelsPanel`, `/api/provider/models`, or `/api/provider/balance`; no cargo test exercises the HTTP routes. Only the provider client/DTO unit tests and the live audit cover them.

**Recommendation:** add an axum route test for 200/503/401/429 mappings and an origin/unauth regression test; add a ModelsPanel test for the balance/models rendering and `is_available` handling.

### D8 — Metadata endpoints have no cache/rate limit (Reliability, Low)

**Evidence:** each Models mount issues both calls; the server constructs a new `DeepSeekClient` and calls DeepSeek per request (`http.rs:846-885`); no `Cache-Control`, no limiter. Combined with D1 this is a small amplification/DoS surface.

**Recommendation:** apply D1's gating plus a short cache and/or shared in-flight limiter.

---

## 9. Claims register

| # | Claim | Evidence | Classification |
|---|---|---|---|
| 1 | The portable app served `127.0.0.1:8099` during the audit. | Live probes at 18:07–18:12 and 18:18–18:20; PIDs 452/11920 then 5432/2744; port closed at 18:12:15 and by 18:29; externally relaunched at 18:33:37 (PID 15572) and up at 18:36. | **VERIFIED** |
| 2 | `/health` reported `status:"ok"` and `deepseek_api_key.ok:true` with a “present (value never logged)” detail. | Exact JSON in §1.1; HTTP 200. | **VERIFIED** |
| 3 | `/health` reports `degraded` + `E_KEY_MISSING` when the key is absent, while HTTP stays 200. | Ephemeral no-key instance on 8096; earlier audit log; code path. | **VERIFIED** (live negative instance), **READ** (log/code) |
| 4 | `/api/provider/models` returns the real catalogue with `deepseek-flash` and `deepseek-v4-pro`, their names, 1,048,576 context and 393,216 max output. | Exact JSON in §2.1; HTTP 200. | **VERIFIED** |
| 5 | `/api/provider/balance` returns `is_available:true` and a USD balance row. | Exact JSON in §2.1 (`9.17`, granted `0.00`, topped `9.17`); HTTP 200. | **VERIFIED** |
| 6 | No DeepSeek key material in provider response bodies or headers. | Body/header `sk-` scans false; header names only `Content-Length`, `Content-Type`, `Date`. | **VERIFIED** |
| 7 | No key material in the browser DOM, served JS, or web source. | Playwright DOM `sk-` false; web source/dist scan only labels/CSS false positives; no `sk-[hex32]`. | **VERIFIED** |
| 8 | `/api/session` enforces the origin allowlist. | Evil-origin 403 `E_ORIGIN_REJECTED`; allowed/no-origin loopback 200 (token not printed). | **VERIFIED** |
| 9 | `/ws` enforces loopback + origin allowlist + per-session token. | Evil/no origin 403; allowed origin + wrong token 401 `E_TOKEN_INVALID`; source `ws.rs:359-387`. | **VERIFIED** |
| 10 | Provider metadata endpoints bypass the session/origin gate. | Evil-origin 200 on both; no ACAO; source handlers have no auth parameters. | **VERIFIED** |
| 11 | The browser cannot make arbitrary provider calls (no attacker-controlled URL/path/model/key). | Route list; `SubmitPrompt {text, project_id}`; fixed `ClientConfig::default()` base URL; model from server config; WS origin+token. | **VERIFIED** |
| 12 | The browser can blind-trigger the two fixed metadata calls. | Evil-origin 200 with no ACAO; browser sends the GET, response blocked. | **VERIFIED** |
| 13 | The DPAPI key file exists, is 262 bytes, starts with the DPAPI blob signature, and is not plaintext. | `%LOCALAPPDATA%\HiveMind\provider.key`; `01000000d08c9ddf`; no ASCII `sk-`/`Bearer`; printable ratio 0.29. | **VERIFIED** |
| 14 | The key is injected into the child from the DPAPI file, not from user/machine/process env. | Launcher source `Unprotect` + `EnvironmentVariables`; User/Machine/Process scopes absent in auditor shell; live key-present health. Child env not read. | **READ** (code) + **BELIEVED** (causal source); key-present **VERIFIED** |
| 15 | No plaintext production key in tracked files. | 97/242 literal `sk-` lines (mostly words); 0 `sk-[alnum24+]`; 0 `sk-[hex32]`; one synthetic fixture `event.rs:990`. Exact-key byte comparison not done. | **VERIFIED** (pattern-level); exact-value **UNVERIFIED** by design |
| 16 | The only key-shaped tracked literal is a synthetic test fixture, not the production key. | `event.rs:990`; 34-char masked `xx-xxxx-…`; committed 2026-11-06 before the DPAPI key file (2026-11-07 17:48); used in redaction test. | **VERIFIED** (file/history); **BELIEVED** (not production key) |
| 17 | `hiverouter.db`, WAL/SHM, audit logs, and the DPAPI file contain no ASCII `sk-`/key-like text. | Direct binary scans in §5; audit logs 0/0. | **VERIFIED** |
| 18 | Web typecheck passes. | `tsc --noEmit` exit 0 at 18:31. | **VERIFIED** |
| 19 | Web tests pass. | 23 files / 186 tests, 0 failed, exit 0 at 18:31 (also at 18:21). | **VERIFIED** |
| 20 | Cargo provider tests pass on the final snapshot. | 8 binaries, 99 passed / 0 failed, exit 0 at 18:31; also 99/99 at 18:15. Transient 98/99 during concurrent edit was resolved. | **VERIFIED** |
| 21 | The Models page DOM shows balance and both models with limits. | Playwright `Provider account` text in §7.1; 200s for both endpoints; no console errors. | **VERIFIED** |
| 22 | An existing run’s assistant reply appears in the UI. | Run/message/usage JSON in §7.2; live body contains user prompt + `HIVE_UI_OK`; exact locator count 1 at 18:19; at 18:36 the UI showed 2 `article.chat-message` elements and the latest run’s assistant reply, backed by a real `file_read` tool-call row in the audit log. | **VERIFIED** |
| 23 | This audit created no new paid run. | No submit/click on send in any Playwright session; network capture only two metadata GETs; run count was 1 at 18:19 and 5 at 18:36, the four new rows created 00:25–00:34Z by external activity. Audit-created count: 0. | **VERIFIED** (no submit from audit); attribution of the four external rows is **BELIEVED** from timestamps/actions |
| 24 | The app exited twice during the audit, was externally relaunched twice, and was not restarted/killed by this audit. | PIDs/port gone at 18:12:15 and by 18:29; external launches 18:17:56 (5432/2744) and 18:33:37 (15572); no kill/start against the app; only the ephemeral 8096 test instances were stopped. | **VERIFIED** (exits/relaunches/no action); cause **BELIEVED** (launcher window-close design) |
| 25 | The live binary matches the current 18:30 source tree. | Binary built 18:00; source edited 18:19–18:26 by another actor; no rebuild observed. | **UNVERIFIED** — the timestamps show the binary predates the later edits, so it cannot include them; byte-level identity to any tree was not checked |

---

## 10. Done / Not done / Unverified

### Done

| Item | Evidence |
|---|---|
| Live `/health` keyed true path + exact fields | §1.1 exact JSON, HTTP 200 |
| Live `/health` key-missing false/degraded path | §1.2 ephemeral no-key instance, exact fields |
| `/api/provider/models` status, IDs, limits, no key | §2.1 exact JSON, header/body scans |
| `/api/provider/balance` status, currency, availability, no key | §2.1 exact JSON, header/body scans |
| Session/WS auth and error mapping | §3.2 exact 403/401 bodies |
| Backend/client secret handling + arbitrary-call surface | §3.3–3.5 source read + live DOM/network |
| DPAPI file existence/non-plaintext check | §4 size/magic/printable/no-`sk-` |
| Plaintext-key scan counts | §5 (97/242 literal; 0 key-like; 1 synthetic fixture) |
| Web typecheck and tests | §6.1 exit 0; 186/186 |
| Cargo provider tests | §6.2 exit 0; 99/99 |
| Keyed Models DOM + existing assistant reply | §7.1–7.2; exact locator count 1 |
| No new paid run | §7.2; only metadata GETs, no submit |

### Not done

| Item | Reason |
|---|---|
| Exact byte-for-byte comparison of the production key against every file | The audit was required not to read the key; pattern/source/history checks used instead. |
| Live verification of the provider 401/429/500 mappings | Would require a bad key / forced upstream status; only the 503 key-missing path was exercised live. The 401/429/502 mappings were read from source; 5xx mapping is in flux. |
| A second article-level UI DOM capture after the final app exit | The app was down by 18:29; the 18:19 body/locator evidence is retained. |
| DPAPI decryption round-trip proof without reading the key | Not performed by design; the launcher source + live key-present health + absent env scopes support it. |

### Unverified

| Item | What would prove it |
|---|---|
| The child’s key came specifically from the DPAPI blob (not from an inherited parent environment) | Reading the child’s environment would expose the key; an alternative is a test harness that records a hash/source marker without the value. |
| The live 18:00 binary exactly matches the current source tree | The tree was edited at 18:19–18:26 after the 18:00 build; a rebuild + hash/manifest comparison would prove it. |
| The app exit cause | Direct observation of the launcher/browser close; the launcher design and absence of a shutdown log are consistent with window close but not proof. |
| A provider 500/503 live response maps correctly | A controlled bad/upstream 5xx test against the current binary; no such live path was exercised. |

---

## 11. Reproduction notes

Commands used (no secrets):

```powershell
# health / provider endpoints
Invoke-WebRequest http://127.0.0.1:8099/health -UseBasicParsing
Invoke-WebRequest http://127.0.0.1:8099/api/provider/models -UseBasicParsing
Invoke-WebRequest http://127.0.0.1:8099/api/provider/balance -UseBasicParsing

# auth probes
curl.exe -s -i -H "Origin: http://evil.example" http://127.0.0.1:8099/api/session
curl.exe -s -i -H "Origin: http://127.0.0.1:8099" -H "Connection: Upgrade" `
  -H "Upgrade: websocket" -H "Sec-WebSocket-Version: 13" `
  -H "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==" `
  "http://127.0.0.1:8099/ws?token=wrong&last_seq=0"
```

The live app was down at 18:29, but was externally relaunched at 18:33:37 and was up at 18:36; the last verified live checks were at 18:35–18:36. Availability can change externally. No key value is present in this document.
