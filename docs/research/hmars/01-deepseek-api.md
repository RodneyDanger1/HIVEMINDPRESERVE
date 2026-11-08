# 01 — DeepSeek API surface: exact endpoints, wire shapes, billing, and HiveMind wiring

**Status:** research complete — read-only pass. No product code, credential, or paid endpoint was touched.
**Research date:** 2026-11-07 (America/Chicago). Every fact below was retrieved on that date.
**Repository:** `hiverouter-v1` — local HMAS: one localhost Rust server, one DeepSeek key, planner → worker DAG, jailed tools, SQLite run/usage ledger.
**Deliverable:** this file only — `docs/research/hmars/01-deepseek-api.md`.
**Documentation pin:** `version_pinned: false`. `version_source: https://api-docs.deepseek.com/updates/`. The API docs are live and unversioned; the newest documented change is **2026-10-17**. Every "documented" claim is dated 2026-11-07 and will rot.

**Evidence rules used in this file**

| Mark | Meaning |
|---|---|
| **DOCUMENTED** | Read verbatim on an official page listed in §11 during this pass. |
| **WIRED** | A production code path constructs the documented endpoint/field, a caller reaches it, and the result is consumed (persisted, displayed, or costed). Seam cited as `path:line`. |
| **NOT WIRED** | Documented, absent from HiveMind, and adoption is a live product question with a named cost. |
| **NOT NEEDED** | Documented, absent, and excluded by a named product decision — no surface or no honest use in a text-only local HMAS. |
| **READ** | Read in this repository (`path:line`). |
| **UNVERIFIED** | Not proven here; do not build on it. |

No live DeepSeek call was made in this pass. Statements about the live account (which models the key returns, actual balance, actual cache-hit rate) are marked **UNVERIFIED**. No key value, token, or `Authorization` header value appears in this document; the only credential name mentioned is the environment variable `DEEPSEEK_API_KEY`, which is a name.

---

## 0. One-screen answer

DeepSeek exposes **one model family behind six wire surfaces and three auxiliary APIs**:

- **Six wire surfaces:** Chat Completions (`POST /chat/completions`), Responses (`POST /responses`), Anthropic Messages (`POST /anthropic/v1/messages`), Files (`POST|GET|DELETE /files…`), Search (`POST /v1/tools/search`), Fetch (`POST /v1/tools/fetch`). Two Beta paths sit on a separate `https://api.deepseek.com/beta` base: FIM (`POST /beta/completions`) and Chat Prefix (`POST /beta/chat/completions`).
- **Three auxiliary APIs:** `GET /models`, `GET /user/balance`, and the four Files operations.
- **All of it uses the same DeepSeek API key.** There is no Anthropic key, no separate search key, no separate files key. The Anthropic-format endpoint accepts a DeepSeek key as either `x-api-key` or `Authorization: Bearer`.

**What is separately billed:** exactly two things besides tokens — **Search and Fetch, at $0.75 per 1,000 successful requests each**. Everything else (chat, thinking, tools, JSON, vision, FIM, prefix, Files storage, Responses, Messages) is billed as ordinary input/output tokens at the model's cache-hit / cache-miss / output rates. Files upload and storage itself carries no documented line-item price.

**What is image-only:** the **Files API is image storage only** (JPEG/PNG/GIF/WebP, `purpose=user_data`). It cannot store text, PDFs, or documents. **Vision input is `deepseek-flash`-only**; `deepseek-v4-pro` does not process images.

**HiveMind today:** the entire production data path is **Chat Completions + thinking + JSON mode (planner) + function tools + streaming + usage/cache accounting + `GET /models` + `GET /user/balance`**, over one Bearer-authenticated client. Everything else is absent, and §10 states for each absent surface whether that absence is a gap (**NOT WIRED**) or a decision (**NOT NEEDED**).

**The honest headline:** HiveMind's core chat path is genuinely integrated and now internally consistent (tools are advertised on the first turn; prompt tokens are estimated; every non-success `finish_reason` and 5xx is handled). The long tail — Search/Fetch, Files, Vision, Responses, Messages, FIM, Prefix, strict tools — is not wired, and adding any of them is a product decision with a second protocol, a second cost model, or a data-egress consequence, not a checkbox.

---

## 1. Transport and authentication invariants

| # | Invariant | DOCUMENTED value | HiveMind |
|---|---|---|---|
| T1 | Base URL, OpenAI format | `https://api.deepseek.com` | **WIRED** — `DEFAULT_BASE_URL`, `backend/crates/hiverouter-provider/src/dto.rs:17` |
| T2 | Base URL, Anthropic format | `https://api.deepseek.com/anthropic` | **NOT WIRED** — no `/anthropic` path anywhere in `backend/` (searched) |
| T3 | Base URL, Beta features | `https://api.deepseek.com/beta` | **NOT WIRED** — single non-beta base URL, `backend/crates/hiverouter-provider/src/client.rs:321-327` |
| T4 | Auth header | `Authorization: Bearer <DEEPSEEK_API_KEY>` | **WIRED** — `client.rs:329-339` (zeroizing buffer, redacted `Debug` at `:202-211`) |
| T5 | Anthropic-format auth | `x-api-key: <key>` **or** `Authorization: Bearer <key>`; if both are sent, `Authorization` takes precedence | **NOT WIRED** |
| T6 | Key source | `https://platform.deepseek.com/api_keys` | **WIRED** — DPAPI-encrypted file, injected as `DEEPSEEK_API_KEY` into the child process only; `backend/crates/hiverouter-provider/src/secret.rs` |
| T7 | Key count | One key per client (`SUPPORTED_KEY_COUNT: usize = 1`, `backend/crates/hiverouter-provider/src/budget.rs:52`) | **WIRED** — `enforce_single_key` |
| T8 | Keep-alive | Non-streaming: server may return empty lines. Streaming: server may send `: keep-alive` SSE comments. Connection is closed if inference has not started after **10 minutes**. | **WIRED** — `parse_sse_line` returns `SseLine::Ignored` for blank lines and `:`-prefixed comments, `client.rs:170-177` |

**Only one credential type exists for all of this.** The Anthropic-format endpoint is a DeepSeek endpoint with Anthropic framing; no third-party key is involved at any point.

---

## 2. Master surface matrix

Status column uses the four §0 marks. "Wired scope" names exactly how far the production path goes. Seams are `path:line` under `backend/crates/`.

| # | Surface | Endpoint | Method | Auth | Billing class | Status | Wired scope / seam |
|---|---|---|---|---|---|---|---|
| 1 | **Quick Start** | `https://api.deepseek.com/chat/completions` | POST | Bearer | tokens | **WIRED** | base URL, Bearer, model IDs, first call: `client.rs:321-339`, `:350-370` |
| 2 | **API guides / reference index** | — (documentation surface) | — | — | — | **DOCUMENTED** | 12 guides + 12 reference pages enumerated in §3.2 |
| 3 | **News / Change Log** | `https://api-docs.deepseek.com/updates` | GET (docs) | none | — | **DOCUMENTED** | newest change 2026-10-17; not mirrored in the repo |
| 4 | **Chat Completions** | `/chat/completions` | POST | Bearer | tokens | **WIRED** | full request/response DTO, stream and non-stream: `dto.rs:396-575`, `client.rs:350-370`, `:538-620` |
| 5 | **Thinking Mode** | field on `/chat/completions` | POST | Bearer | tokens (+reasoning tokens) | **WIRED** | planner on, leaves off: `dto.rs:219-253`, `provider_executor.rs:301-302`, `:332` |
| 6 | **Multi-round Conversation** | stateless replay on `/chat/completions` | POST | Bearer | tokens | **WIRED** | cumulative transcript replay incl. tool turns: `provider_executor.rs:357-424` |
| 7 | **JSON Output** | `response_format:{type:"json_object"}` | POST | Bearer | tokens | **WIRED** (planner only) | `dto.rs:281-305`, `provider_executor.rs:304`, `:447-459` |
| 8 | **Tool Calls** | `tools[]`, `tool_choice`, `role:"tool"` | POST | Bearer | tokens | **WIRED** | first turn advertises tools: `provider_executor.rs:323-339`; continuation `:409-415`; assembly `tool_assembler.rs` |
| 9 | **Context Caching** | automatic; `usage.prompt_cache_*` | POST | Bearer | cache-hit vs cache-miss rate | **WIRED** | counters required or run INCOMPLETE: `accounting.rs:234-263`; priced `pricing.rs:93-115` |
| 10 | **Chat Prefix Completion (Beta)** | `/beta/chat/completions` + last message `prefix:true` | POST | Bearer | tokens | **NOT NEEDED** | no `prefix` field: `dto.rs:88-106`; no beta base: `client.rs:321-327` |
| 11 | **FIM Completion (Beta)** | `/beta/completions` | POST | Bearer | tokens | **NOT NEEDED** | no `/completions` method; Chat-only request DTO `dto.rs:396-441` |
| 12 | **Vision** | `image_url` / `file` content blocks | POST | Bearer | tokens (≤1024 tok/image) | **NOT NEEDED** | `ChatMessage.content: Option<String>` — no content parts: `dto.rs:93` |
| 13 | **Files API** | `/files` | POST/GET/DELETE | Bearer | none documented (token cost only when referenced) | **NOT NEEDED** | no `/files` method; `client.rs:266-313` is `GET`-only |
| 14 | **Upload / List / Retrieve / Delete Files** | `/files`, `/files`, `/files/{id}`, `/files/{id}` | POST/GET/GET/DELETE | Bearer | none documented | **NOT NEEDED** | same as #13 |
| 15 | **Search & Fetch** | `/v1/tools/search`, `/v1/tools/fetch` | POST | Bearer | **$0.75 / 1,000 successful requests** | **NOT WIRED** | no route, no tool id, no network executor: `tool.rs:108-127`, `executor.rs` |
| 16 | **Responses API** | `/responses` | POST | Bearer | tokens | **NOT NEEDED** | semantic SSE, no `[DONE]`; parser assumes `data:`+`[DONE]` `client.rs:170-192` |
| 17 | **Anthropic API** | `/anthropic/v1/messages` (alias `/anthropic/messages`) | POST | `x-api-key` or Bearer | tokens | **NOT NEEDED** | no Anthropic base or content blocks |
| 18 | **Messages** | `/anthropic/v1/messages` (same endpoint as #17) | POST | `x-api-key` or Bearer | tokens | **NOT NEEDED** | same as #17 |
| 19 | **Models** | `/models` | GET | Bearer | free | **WIRED (partial)** | 6 of 10 documented fields: `dto.rs:892-910`, proxy `http.rs:877-901`, UI `ModelsPanel.tsx:247-248` |
| 20 | **Balance** | `/user/balance` | GET | Bearer | free | **WIRED** | `dto.rs:913-933`, `http.rs:903-924`, `is_available` consumed `ModelsPanel.tsx:89`, `:219` |

Row 9 and row 15 are the two rows where **money is charged differently from raw token count**. Row 15 is the only per-request-priced surface in the whole API.

### 2.1 Documented status codes, per surface

Where a surface publishes its own table it is listed exactly; where it does not, that is stated rather than filled in from the generic page.

| Surface | Documented status codes | Source |
|---|---|---|
| Chat Completions (and the two Beta chat/completion paths) | `400` invalid format · `401` auth fails · `402` insufficient balance · `422` invalid parameters · `429` rate limit · `500` server error · `503` overloaded; `500`/`503` say **retry** | S5 |
| Models, Balance | `200` only is documented; the generic error set is not restated on these pages | S7, S8 |
| Files (upload/list/retrieve/delete) | `200` documented per operation; **no per-status table is published** — do not invent one | S12, S22 |
| Search, Fetch | `400` · `401` · `402` · `403` security-policy refusal · `404` no usable content · `408` body read timeout · `413` body too large · `429` QPS · `500` · `504`; retry **408/429/5xx** only | S23 |
| Responses | `200`; `400` documented for over-context (`truncation` unsupported), for `input_image` with neither/both of `image_url`/`file_id`, and for images in `system`/`assistant`; unsupported parameters are **silently ignored**, not rejected | S9, S25 |
| Messages (Anthropic) | `200`; `400` documented for a non-`claude` unsupported model name, for failed thinking replay, and for a named `tool_choice` in thinking mode; `422` documented for `type:"custom"` on a client tool | S10, S26 |

---

## 3. Entry points and meta

### 3.1 Quick Start — "Your First API Call"

- **Endpoint:** `POST https://api.deepseek.com/chat/completions`.
- **Auth:** `Authorization: Bearer ${DEEPSEEK_API_KEY}` and `Content-Type: application/json`.
- **Documented models:** `deepseek-flash` and `deepseek-v4-pro`.
  - `deepseek-flash` is served by **DeepSeek-V4.1-Flash-1017**.
  - Legacy names `deepseek-v4-flash` and `deepseek-v4-flash-vision-exp` are **still accepted** but the underlying models are retired; requests are served by DeepSeek-V4.1-Flash-1017 and **billed at the Flash price**.
- **Documented minimal body:** `model`, `messages[]`, and optionally `thinking:{"type":"enabled"}`, `reasoning_effort:"high"`, `stream:false`.
- **Also documented on this page:** the Anthropic base URL `https://api.deepseek.com/anthropic`, and the pointer to the agent-integration guides (Claude Code, Codex, OpenCode, OpenClaw, Hermes, Reasonix, WorkBuddy/CodeBuddy, Qoder).
- **Pricing:** tokens only.
- **Streaming/tools:** the page shows `stream:false` and points to `stream:true`; tools are covered in the tool-calls guide.
- **HiveMind:** **WIRED** — base URL `dto.rs:17`, Bearer `client.rs:329-339`, chat path `client.rs:321-327`, model enum `dto.rs:27-56`.

### 3.2 API guides / API reference index

The official docs publish **12 guides** and **12 reference pages**. This is the complete documented surface as of 2026-11-07:

| Guides (`/guides/…`) | Reference (`/api/…`) |
|---|---|
| `vision` | `create-chat-completion` |
| `thinking_mode` | `create-response` |
| `multi_round_chat` | `create-message` |
| `chat_prefix_completion` | `create-completion` (FIM, Beta) |
| `fim_completion` | `list-models` |
| `json_mode` | `get-user-balance` |
| `tool_calls` | `create-file` |
| `files_api` | `list-files` |
| `search` | `retrieve-file` |
| `kv_cache` | `delete-file` |
| `responses_api` | `search` |
| `anthropic_api` | `fetch` |

Quick Start adds five more pages: `pricing`, `token_usage`, `rate_limit`, `error_codes`, plus the agent-integration subtree.

**HiveMind reads none of these at runtime** (they are documentation, not an endpoint). The only runtime-relevant entries are the ones enumerated in §4–§8.

### 3.3 News / Change Log

- **News index:** `https://api-docs.deepseek.com/news/news261017` (latest), plus 18 archived release notes back to 2024-07-25.
- **Change Log:** `https://api-docs.deepseek.com/updates`, newest entry **2026-10-17**.

Documented change history that constrains HiveMind's assumptions:

| Date | Change | Consequence for HiveMind |
|---|---|---|
| 2026-10-17 | In-house **Search API opened**; **V4.1-Flash-1017** update; pricing unchanged | Search/Fetch became a purchasable surface HiveMind does not use |
| 2026-09-10 | V4.1-Flash release; native multimodal; `deepseek-v4-flash*` retired and re-routed; prices reduced | HiveMind's model enum accepts only the two current IDs (`dto.rs:27-56`) — legacy IDs are refused, which is now the correct choice, not a bug |
| 2026-08-21 | V4-Flash-Vision-Exp (experimental) | superseded by 2026-09-10 |
| 2026-08-13 | V4-Pro GA; **native Responses API**; **three effort levels** low/high/max; **peak/off-peak pricing from 2026-08-16 16:00 UTC** | HiveMind's peak-window calculator and effort enum date from this change |
| 2026-07-31 | V4-Flash public beta | — |
| 2026-04-24 | V4 family; `deepseek-chat`/`deepseek-reasoner` announced for discontinuation 2026-07-24 | Both names are now gone from the docs; HiveMind never sends them |
| 2025-12-01 | V3.2 maps `deepseek-chat`→non-thinking, `deepseek-reasoner`→thinking | historical |
| 2024-07-25 | JSON mode, function calling, Chat Prefix (Beta), FIM (Beta) | the four Beta-era features HiveMind partially uses |

**2026-10-17 news item that matters for billing:** Search & Fetch calls triggered by **server-side tools** in DeepSeek Harness, Claude Code, and Codex were "currently free" with billing starting **2026-10-31 16:00 UTC**. As of this research date that start date has passed; the docs do not state a later exemption. Treat server-side search as billable now. **UNVERIFIED** for this account (no paid call made).

---

## 4. Core chat surfaces

### 4.1 Chat Completions

**Endpoint:** `POST https://api.deepseek.com/chat/completions`. No `/v1` prefix is documented for this path.

**Auth:** `Authorization: Bearer <DEEPSEEK_API_KEY>`.

**Documented status codes** (from `/quick_start/error_codes`): `400` invalid format, `401` authentication fails, `402` insufficient balance, `422` invalid parameters, `429` rate limit, `500` server error, `503` server overloaded. `500`/`503` documentation explicitly says *retry after a brief wait*.

**Request body (documented fields):**

| Field | Type | Documented constraint |
|---|---|---|
| `messages` | array, min 1 | roles `system` / `user` / `assistant` / `tool`; `system` and others accept optional `name` |
| `model` | string | `deepseek-flash` \| `deepseek-v4-pro` |
| `thinking` | object, nullable | `{type: "enabled"\|"disabled"}`; **required `type` when the object is present**; takes precedence over `reasoning_effort` |
| `reasoning_effort` | string | `none` \| `low` \| `high` \| `max`; aliases `minimal`→`low`, `medium`/`xhigh`→`high`, `ultra`→`max`; default `high` |
| `max_tokens` | int, nullable | **1 – 384K (393216)**; default **8K** non-thinking, **64K** thinking, **128K** at `reasoning_effort:"max"` |
| `response_format` | object, nullable | `{type:"text"}` \| `{type:"json_object"}` |
| `stop` | string \| string[] | up to **16** sequences |
| `stream` | bool | default `false`; data-only SSE terminated by `data: [DONE]` |
| `stream_options.include_usage` | bool | must accompany `stream:true`, else **400**; usage then present (null) on every chunk and populated on the last |
| `temperature` | number | 0–2, default 1; **no effect in thinking mode** |
| `top_p` | number | (0,1]; model- and mode-dependent effective value (see §4.2) |
| `tools` | array, nullable | **only `type:"function"`**; `function.name` ≤128 chars, `[A-Za-z0-9_-]`; optional `strict` (Beta) |
| `tool_choice` | string \| object | `none` \| `auto` \| `required` \| `{type:"function",function:{name}}`; **`none` is the default when no tools are present; `auto` when tools are present**; `required` and named choice **return 400 in thinking mode** |
| `logprobs` / `top_logprobs` | bool / int | `top_logprobs` 0–20, requires `logprobs:true` |
| `user_id` | string | ≤**512 bytes** UTF-8; purpose is content-safety review, **KVCache isolation**, and scheduling isolation |
| `frequency_penalty`, `presence_penalty` | — | **deprecated — accepted and ignored** |

**Response shape:** `id`, `object:"chat.completion"`, `created`, `model`, `system_fingerprint`, `choices[]` with `finish_reason`, `index`, `message{content, reasoning_content, tool_calls[{id,type,function{name,arguments}}], role}` and optional `logprobs`, plus `usage`.

**`finish_reason` documented values:** `stop`, `length`, `content_filter`, `tool_calls`, `insufficient_system_resource`, `aborted`. Only `stop` and `tool_calls` are successes.

**Usage shape:** `prompt_tokens` (**equals** `prompt_cache_hit_tokens + prompt_cache_miss_tokens`), `completion_tokens`, `total_tokens`, `prompt_tokens_details.cached_tokens` (same value as `prompt_cache_hit_tokens`), `prompt_cache_hit_tokens`, `prompt_cache_miss_tokens`, `completion_tokens_details.reasoning_tokens`.

**Streaming:** data-only SSE; **no separate usage-only chunk** — the statistics ride on the last content chunk, whose `choices` array has exactly one element with no new content and a non-null `finish_reason`.

**Pricing:** tokens. `prompt_tokens` are split into cache-hit and cache-miss at different rates (§4.6).

**HiveMind:** **WIRED**. Request DTO `backend/crates/hiverouter-provider/src/dto.rs:396-441`; model enum `:27-56`; streaming parser `client.rs:170-192`; stream pump `:624-723`; usage DTO `dto.rs:578-628`; cost `accounting.rs:196-293`.
Two documented-code gaps remain: `400` and `422` fall into the catch-all arm that reports *"undocumented provider HTTP status"* as `ProviderInvalidResponse` (`client.rs:520-523`), although both are documented statuses with distinct causes. `500/502/503/504` are correctly mapped to a retryable refusal (`client.rs:516-519`).

### 4.2 Thinking Mode

**Mechanism:** a `thinking` object on Chat Completions, or `reasoning_effort` alone. Documented toggle semantics:

- Thinking is **enabled by default**, default effort **`high`**.
- `{"thinking":{"type":"disabled"}}` turns it off; when the object is present its `type` **takes precedence** over `reasoning_effort`.
- `reasoning_effort:"none"` also disables thinking.

**Effort mapping (documented table):**

| Requested | Actual |
|---|---|
| `minimal` | `low` |
| `low` | `low` |
| `medium` | `high` |
| `high` | `high` |
| `xhigh` | `high` |
| `max` | `max` |
| `ultra` | `max` |

**Parameters that do nothing in thinking mode:** `temperature`, `presence_penalty`, `frequency_penalty` — accepted, no error, no effect.

**`top_p` behaviour (documented, model-specific):**

- `deepseek-flash`: thinking mode default 1.0 and **values below 0.95 are treated as 0.95**; non-thinking mode is **fixed at 1.0 and the supplied value is ignored**.
- `deepseek-v4-pro`: thinking mode **ignores** the supplied value (fixed 0.95 when tool calling is enabled, else 1.0); non-thinking mode honours it, default 1.0. Setting `tool_choice:"none"` disables tool calling even when `tools` is supplied.

**`reasoning_content` pass-back rule (this is the 400 trap):**

- Request **carries `tools`** → `reasoning_content` of **all previous turns** must be passed back and is concatenated into context. *"even for turns where the model did not perform a tool call"*; omission → **HTTP 400**.
- Request **does not carry `tools`** → `reasoning_content` need not be passed back; if passed it is **ignored**.

**Pricing:** reasoning tokens are ordinary output tokens (`completion_tokens_details.reasoning_tokens` is informational), so thinking costs output rate × reasoning tokens.

**HiveMind:** **WIRED**. `thinking`/`reasoning_effort` types `dto.rs:219-267`; planner thinking + effort `provider_executor.rs:301-302`; leaf thinking `:332`; planner-only effort config `backend/crates/hiverouter-core/src/config.rs:63`, `:110-113`; reasoning pass-back on replayed tool turns `provider_executor.rs:397`.
Scope limit: leaves send **no** `reasoning_effort`, so a thinking-enabled leaf gets the documented default `high`, not the configured planner effort. Supported effort levels are not validated against the model (see §8.1).

### 4.3 Multi-round Conversation

**Mechanism:** Chat Completions is **stateless**. The client must concatenate the full history and re-send it every turn. The documented example is append-only: round 1 `[user]`; round 2 `[user, assistant, user]`.

**Pricing consequence:** every turn re-bills the whole prefix as input; the disk cache is what makes that cheap (§4.6).

**HiveMind:** **WIRED**. The engine replays the cumulative transcript, converting each prior tool turn into one assistant tool-call message plus one `role:"tool"` message per bounded result, so a multi-turn loop never loses earlier rounds (`provider_executor.rs:357-424`). Empty transcript and all-empty-result transcripts are refused rather than sent (`:366-368`, `:406-408`).

### 4.4 JSON Output

**Mechanism:** `response_format: {"type":"json_object"}` **plus** an instruction containing the word "json" in a system or user message (documented as a requirement, not a suggestion).

**Critical documented caveat:** if the model's output cannot be recognised as JSON, **the API replaces the non-JSON content with spaces (`U+0020`)**. The returned `content` may therefore be *only spaces* and appear empty **even when `finish_reason` is `stop`**. The docs advise handling blank content, e.g. by retrying.

**Truncation caveat:** content may be partially cut off when `finish_reason:"length"`.

**Pricing:** tokens only.

**HiveMind:** **WIRED (planner only)**. `ResponseFormat::json_object()` `dto.rs:281-305`; applied to the planner `provider_executor.rs:304`; blank content is refused honestly through `message_text` (`:447-459`) rather than parsed as an empty plan. **No retry** on a blank or malformed plan — one bad plan fails the run.

### 4.5 Tool Calls

**Mechanism (Chat Completions):** `tools[{type:"function",function:{name,description,parameters}}]`; the model answers with `tool_calls[{id,type,function{name,arguments}}]`; the client executes and appends `{"role":"tool","tool_call_id":…, "content":…}` and re-requests.

**Documented constraints:**

1. **Only `function` tools** exist on Chat Completions. `parameters` is JSON Schema; omitting it means an empty parameter list. Tool names must be unique.
2. **`tool_choice` defaults:** `none` when no tools are present, `auto` when tools are present. `required` and named choices **are not supported in thinking mode and return 400** — disable thinking first.
3. **The model does not execute anything.** *"the functionality … needs to be provided by the user."*
4. **Arguments are not guaranteed valid JSON** and may contain parameters not in the schema — validate before use.
5. **Thinking-mode tool use** is supported (since DeepSeek-V3.2) and requires the full `reasoning_content` replay described in §4.2.
6. **Mid-conversation insertion is NOT supported on Chat Completions.** Inserting tool calls the model did not generate, plus their results, is supported only by the **Anthropic `/messages`** and **Responses** formats. Chat Completions *does* support inserting `system` messages mid-conversation.
7. **`strict` mode (Beta):** requires `base_url="https://api.deepseek.com/beta"`, `strict:true` on **every** function, and server-side JSON-Schema validation. Supported types: `object`, `string`, `number`, `integer`, `boolean`, `array`, `enum`, `anyOf`, plus `$ref`/`$def`. Every `object` must mark **all** properties `required` and set `additionalProperties:false`. **Unsupported:** `minLength`, `maxLength`, `minItems`, `maxItems`. Works in both thinking and non-thinking mode.

**Pricing:** tokens only (tool schemas are input tokens; results are input tokens; the model's calls are output tokens).

**HiveMind:** **WIRED**. The first leaf request now builds tier-scoped specs and advertises them with `tool_choice:"auto"` (`provider_executor.rs:323-339`), and the continuation re-offers them (`:409-415`). Streamed fragments are assembled index-keyed in `backend/crates/hiverouter-provider/src/tool_assembler.rs` and executed through the jailed registry. `tool_choice:"required"` combined with thinking is refused **before** sending (`dto.rs:554-572`), matching the documented 400.
**NOT WIRED:** `strict` (Beta) — no `strict` field in `ToolSpec` (`dto.rs:361-392`) and no beta base URL.

### 4.6 Context Caching

**Mechanism:** automatic disk cache, on by default, no code change. The current documented model is **not** simple prefix overlap — it is **cache prefix units** under Sliding Window Attention:

- A hit requires a request to **fully match** a **cache prefix unit**.
- Units are persisted at three moments:
  1. **Request boundaries** — at the end of the user input and at the end of the model output. A later request that fully matches either unit hits.
  2. **Common-prefix detection** — when several requests share a prefix, that prefix is persisted as its own unit.
  3. **Fixed token intervals** — for long inputs/outputs, units are carved at fixed intervals so a long prefix is not left uncacheable.
- Worked example 2 is the important one: `A+B` then `A+C` **does not hit**, because `A+C` is not a full match of the unit `A+B`; after the system detects the shared `A`, a third request `A+D` **does** hit.

**Counters:** `prompt_cache_hit_tokens`, `prompt_cache_miss_tokens` in `usage`; `prompt_tokens` equals their sum.

**Billing:** cache hits are billed at the cache-hit rate, misses at the cache-miss rate — a 50× difference on Flash and 30× on Pro (§8.3). Caching is "best effort", construction takes seconds, and entries clear "usually within a few hours to a few days".

**HiveMind:** **WIRED**. The hit/miss split is mandatory for a settled cost — a response without it marks the run INCOMPLETE instead of assuming a rate (`accounting.rs:234-263`); prices come from sourced rows (`pricing.rs:93-147`); the UI shows the split (`web/src/panels/ModelsPanel.tsx:161-164`).
Operational caveat: because the documented hit rule is *full match of a unit*, HiveMind's planner/leaf prompts hit only when their literal prefix is reproduced. Nothing in the code measures the realised hit rate, so any "cache saves us X" claim is **UNVERIFIED**.

---

## 5. Beta completion surfaces

Both Beta surfaces require the separate base URL `https://api.deepseek.com/beta`. Both are token-billed at the model's normal rates.

### 5.1 Chat Prefix Completion (Beta)

- **Endpoint:** `POST https://api.deepseek.com/beta/chat/completions` (documented as "follows the Chat Completion API" with `base_url="…/beta"`; the concrete path is the beta base plus the chat path).
- **Auth:** Bearer, identical to Chat Completions.
- **Required request shape:** the **last** message must have `role:"assistant"` **and** `prefix:true`. The example pairs it with `stop:["```"]`.
- **Documented use:** force a shape — the canonical example forces `"```python\n"` so the model emits code only.
- **Response shape:** the normal chat completion object; `message.content` is the continuation.
- **Limits:** inherits Chat Completions limits (1M context, 384K max output, same status codes).
- **Pricing:** tokens.
- **Streaming/tools:** inherits both; the guide shows neither.
- **HiveMind:** **NOT NEEDED**. `ChatMessage` has no `prefix` field (`dto.rs:88-106`), `ChatRequest` has no `stop` (`:396-441`), and there is one non-beta base URL (`client.rs:321-327`). Adding it would fork the transport for a benefit JSON mode plus the "json" instruction already provides.

### 5.2 FIM Completion (Beta)

- **Endpoint:** `POST https://api.deepseek.com/beta/completions` (reference: `POST /completions` with `base_url="…/beta"`).
- **Auth:** Bearer.
- **Request body:** `model` (required), `prompt` (required), `suffix`, `max_tokens`, `echo` (cannot combine with `suffix` or `logprobs`), `logprobs` 0–20, `stop` (≤16), `stream`, `stream_options.include_usage`, `temperature`, `top_p`. `frequency_penalty`/`presence_penalty` are deprecated and ignored.
- **Documented limits:** **max tokens 4K** (guide) and **non-thinking mode only** (pricing feature table). The reference's `max_tokens` schema itself only states `>= 0`; the 4K ceiling comes from the guide, so treat 4K as the operative bound.
- **Response shape:** `object:"text_completion"`, `choices[].text`, `finish_reason` ∈ `stop|length|content_filter|insufficient_system_resource|aborted` (**no `tool_calls`**), plus the same `usage` block incl. cache counters and `reasoning_tokens`.
- **Pricing:** tokens.
- **Streaming/tools:** streams identically (`data:` frames + `[DONE]`); **no tool support**.
- **HiveMind:** **NOT NEEDED**. No `/completions` method and a Chat-only DTO (`client.rs:266-313`, `dto.rs:396-441`). This is a code-completion surface for an editor plugin, not an agent-chat surface.

---

## 6. Multimodal and storage surfaces

### 6.1 Vision

- **Model support:** **`deepseek-flash` only**. `deepseek-v4-pro` does **not** process images; on the Anthropic endpoint it replaces them with the literal text `[Unsupported Image]`.
- **Formats:** JPEG, PNG, GIF, WebP; **detected from file content**, not filename or declared MIME type.
- **Three input methods, all Chat Completions content blocks:**
  1. **Base64 inline** — `{"type":"image_url","image_url":{"url":"data:image/jpeg;base64,…"}}`. Counts against the **48 MiB request body** limit.
  2. **External URL** — `{"type":"image_url","image_url":{"url":"https://…"}}`; URL ≤**8192** chars, image ≤**32 MiB**, download must finish within **60 s**.
  3. **Files API reference** — `{"type":"file","file_id":"file-api-…"}`; may be up to **64 MiB** and is exempt from the 32 MiB per-image check. A `file` block may instead carry inline base64 via `file_data` plus optional `filename`; `file_id` and `file_data` are **mutually exclusive**.
- **`detail`** on `image_url`: `low` (downscale to 512×512), `high` (original; alias of `original`), `original`, `auto` (currently `original`).
- **Documented limits:** request body 48 MiB; ≤**600** images per request; total image size **64 MiB** without `file_id` images, up to **200 MiB** including them; max dimension **8192 px** per side, dropping to **4096 px** when a request contains **15 or more** images.
- **Restriction:** images are allowed in **`user` messages only**; images in `system` or `assistant` messages return **400**.
- **Token cost:** images are resized (small images upscaled toward ~544×544, large downscaled toward ~1300×1300 total pixel count) and **capped at 1024 tokens per image**, counted independently per image. Billed as ordinary input tokens — **no separate vision line item**.
- **Also available in** Responses (`input_image` parts) and Anthropic (`image` blocks with `source.type` `base64`/`url`/`file`).
- **HiveMind:** **NOT NEEDED** for the current text-only HMAS. `ChatMessage.content` is `Option<String>` (`dto.rs:93`), so there is no content-part model at all; the Data surface is a local text workspace jail. Wiring vision requires a content-part type, the limits above, Flash-only gating, and an image UI.

### 6.2 Files API

- **Purpose:** store **images** and reference them later by `file_id`. Documented as the recommended way to reuse an image across requests or to exceed the 48 MiB body / 32 MiB inline limits.
- **Formats:** JPEG, PNG, GIF, WebP — **image content only**. There is no documented text, PDF, or arbitrary-document upload path.
- **Auth:** Bearer.
- **Base URL:** `https://api.deepseek.com`.
- **Anthropic-compatible variant:** base URL `https://api.deepseek.com/anthropic`, endpoints under `/anthropic/v1/files…`, and **all requests require the header `anthropic-beta: files-api-2025-04-14`**. With a plain HTTP client the full path must be written out; the Anthropic SDK appends `/v1` itself.
- **Pricing:** no documented per-upload or per-GB charge. Cost appears only as tokens when a file is actually referenced in a request.
- **HiveMind:** **NOT NEEDED**. `client.rs:266-313` is `GET`-only (`list_models`, `balance`); there is no `/files` method, no multipart client, and no `file_id` field in any DTO. Wiring it would add a second data domain (remote image storage) next to the local workspace jail, with retention/privacy obligations and no product gain for a text agent.

### 6.3 Upload / List / Retrieve / Delete Files

| Operation | Endpoint | Method | Documented request | Documented response |
|---|---|---|---|---|
| **Upload** | `/files` | POST | `multipart/form-data`: `file` (required, ≤**64 MiB**, upload must finish within **10 minutes**), `purpose` (required, must be `user_data`), optional `expires_after[anchor]` (must be `created_at`) **and** `expires_after[seconds]` (3600–2592000) | `{id:"file-api-…", object:"file", bytes, created_at, filename, purpose:"user_data", expires_at?}` — `expires_at` only when an expiry was set |
| **List** | `/files` | GET | query `after` (file_id cursor), `limit` **1–1000**, `order` `asc` (default) \| `desc`, `purpose` (only `user_data`) | `{object:"list", data:[file…], first_id, last_id, has_more}` |
| **Retrieve** | `/files/{file_id}` | GET | path parameter | the file object |
| **Delete** | `/files/{file_id}` | DELETE | path parameter | `{id, object:"file", deleted:true}` |

**Anthropic-compatible field differences (documented):**

| | OpenAI-compatible | Anthropic-compatible |
|---|---|---|
| List pagination | `after` | `after_id` / `before_id` (mutually exclusive) |
| List `limit` | 1–1000, default **1000** | 1–1000, default **20** |
| `order` / `purpose` | supported | **not supported** |
| Top-level list `object` | `"list"` | omitted |
| Size field | `bytes` | `size_bytes` |
| Type field | `object` | `type` |
| `created_at` | Unix seconds | **RFC 3339 string** |
| Required header | none | `anthropic-beta: files-api-2025-04-14` |

Delete on the Anthropic variant returns `{id, type:"file_deleted"}`.

**Documented quotas:** max upload **64 MiB**; max filename **512 bytes**; **25 GiB** storage per user; **10 000** stored files per user; expiry **1 hour to 30 days** or permanent (omit both `expires_after` fields). Files are account-scoped: all keys under the account share them, and they are referenceable from either API family.

**HiveMind:** **NOT NEEDED** (same reasoning as §6.2). Note the asymmetry that makes this a real decision rather than an oversight: 25 GiB of *images* is storage HiveMind cannot use, while its actual data surface is local text files.

---

## 7. Tools and alternative protocols

### 7.1 Search & Fetch

These are **standalone raw JSON APIs**, not tools on the chat endpoint. They use the **same Bearer auth** as the model API.

| API | Endpoint | Method | Auth |
|---|---|---|---|
| Search | `POST https://api.deepseek.com/v1/tools/search` | POST | `Authorization: Bearer` + `Content-Type: application/json` |
| Fetch | `POST https://api.deepseek.com/v1/tools/fetch` | POST | same |

**Search request fields:**

| Field | Type | Required | Default | Documented constraint |
|---|---|---|---|---|
| `query` | string | yes | — | **1–128** Unicode characters, not all whitespace; supports operators |
| `count` | int | no | `10` | **1–50** |
| `ori_question` | string, nullable | no | — | 1–128 chars; `null` = not provided |
| `localization` | string | no | `""` | an **IP address** used to localize results |
| `need_content` | bool | no | `false` | `content` is always `""` when `false` |

Unknown fields are ignored.

**Documented query operators:** `site:`, `-site:`, `-term`, `"exact phrase"`, `-"phrase"`, `lang:zh,en`, `after:2025[-01-01]`, `before:2026[-06-01]`; multiple `site:` clauses join with `OR`.

**Search response:** `{results:[{rerank_score (0–3; ≥1.5 = high quality), title, url, snippet (~100–200 chars, for UI), summary (~600–800 chars, for the model), content (""), site_name|null, site_icon|null, date (RFC 3339 UTC)|null, authority (0–4; -1 = unknown; ≥3 = highly authoritative)}]}`. No matches ⇒ empty array with HTTP 200.

**Fetch request:** `url` — required, ≤**8192** chars, HTTP or HTTPS with a host, **must not contain a username or password**.

**Fetch response:** `{title, content}` where `content` is **Markdown**.

**Documented error semantics (this endpoint set has its own richer table):**

| Status | Meaning |
|---|---|
| 400 | invalid headers/body, malformed JSON, invalid field value |
| 401 | missing/invalid key |
| 402 | insufficient balance |
| **403** | **Fetch request rejected by the security policy** |
| **404** | **Fetch could not retrieve usable content** (unavailable URL, site policy, target error, unsupported content type, no extractable Markdown) |
| 408 | reading the request body timed out |
| 413 | request body too large |
| 429 | Search **or** Fetch QPS exceeded |
| 500 / 504 | internal error / downstream timeout |

Retry guidance: retry **408, 429, 5xx**; top up on **402**; do **not** retry other 4xx without changing the request.

**Billing — the only per-request-priced surface:** **$0.75 per 1,000 successful requests** for each API. A request is billable **only when it returns HTTP 200 and produces a result** (Search: non-empty `results`; Fetch: page content). Failed requests, empty searches, and timeouts are **not billed**. `count` does not change the price.

**Rate limits:** per account, independent of each other — **Search QPS 100**, **Fetch QPS 25**, with an additional **per-domain** QPS limit on Fetch. Account-level, regardless of which key is used, separate from model concurrency.

**HiveMind:** **NOT WIRED**, and this is the one absent surface with obvious product value for an HMAS (research agents). Three concrete blockers, each documented in the repo:
1. No tool id or executor — the registry has no network tool (`backend/crates/hiverouter-tools/src/tool.rs:108-127`), although `PermissionClass::Network` exists as an unused class (`:38`, `:49`).
2. No per-request cost dimension — `RunAccounting`/`PriceRow` model cache-hit/miss/output token prices only (`backend/crates/hiverouter-provider/src/pricing.rs:49-64`, `accounting.rs:196-293`). A search call cannot be costed honestly without a new dimension.
3. No policy — Fetch needs SSRF/URL rules, approval, and bounded results; 403/404 must surface as refusals, not empty content.

### 7.2 Responses API

- **Endpoint:** `POST https://api.deepseek.com/responses`.
- **Auth:** Bearer.
- **Documented purpose:** Codex compatibility; **stateless** — responses and conversations are not stored.
- **Request:** `model`, `input` (string or item list; at least one of `input`/`instructions` required), `instructions` (inserted as the first system message), `reasoning{effort}` (`none|low|high|max` + aliases), `max_output_tokens`, `stream`, `temperature`, `top_p`, `text.format` (`text` | `json_object` | **`json_schema`** with `name`+`schema`), `tools`, `tool_choice`, `top_logprobs` 0–20, `user`.
- **Tools:** `function` supported; `custom` **only** `apply_patch`; **`web_search` supported with `deepseek-v4-pro` only**; other built-in tools (`file_search`, `code_interpreter`, `computer_use`, `mcp`) are **ignored**.
- **Image input:** `input_image` parts with `image_url` (http(s) ≤8192 chars, or base64 data URL) **or** `file_id` — mutually exclusive; passing neither or both is a **400**. `detail` `low|high|original|auto`. Images allowed in `user`/`developer` messages and in `function_call_output`/`custom_tool_call_output`; images in `system`/`assistant` return **400**. File inputs as documents are **not** supported.
- **Streaming:** **semantic SSE** — each event has an `event` name and a monotonically increasing `sequence_number`; the stream ends with `response.completed` / `response.incomplete` / `response.failed`. **There is no `data: [DONE]`.**
- **Documented event set:** `response.created`, `response.in_progress`, `response.output_item.added|done`, `response.content_part.added|done`, `response.reasoning_text.delta|done`, `response.output_text.delta|done`, `response.function_call_arguments.delta|done`, `response.custom_tool_call_input.delta|done`, `response.web_search_call.in_progress|searching|completed`, and the three terminals.
- **Usage:** `input_tokens`, `output_tokens`, `total_tokens`, `input_tokens_details.cached_tokens`, `output_tokens_details.reasoning_tokens`, plus `server_tool_use{tool_search, tool_fetch}` (omitted entirely when both are 0; both fields present when it appears; counts **operations**, not tokens; only successful operations counted).
- **Documented unsupported parameters**, all **silently ignored** (no error): `previous_response_id`, `conversation`, `store` (always `false`), `background`, `metadata`, `include`, `prompt`, `truncation` (over-context ⇒ **400**), `service_tier`, `safety_identifier`, `prompt_cache_key`/`prompt_cache_retention`, `context_management`, `stream_options`; `parallel_tool_calls` and `max_tool_calls` are ignored; `reasoning.summary` accepted but not generated; `text.verbosity` accepted but has no effect.
- **Pricing:** tokens; server-side `web_search` additionally consumes billable Search operations (`server_tool_use` is documented as usable "as a basis for billing").
- **HiveMind:** **NOT NEEDED**. The parser assumes `data:` frames plus `[DONE]` and would treat semantic events as ignorable lines (`client.rs:170-192`); the DTO set is Chat-only; usage names differ (`input_tokens`/`output_tokens`). Justified only by external-client interop (Codex), not by HiveMind's own loop.

### 7.3 Anthropic API / 7.4 Messages

"Anthropic API" and "Messages" are the **same endpoint** documented twice (a guide and a reference page). Treated as one surface here, listed as two rows in §2 because the research surface named them separately.

- **Endpoint:** `POST https://api.deepseek.com/anthropic/v1/messages`. A compatible path `/anthropic/messages` is also documented for direct HTTP. The SDK base URL is `https://api.deepseek.com/anthropic` and the SDK appends `/v1/messages`.
- **Auth:** `x-api-key: <key>` **or** `Authorization: Bearer <key>`; if both are supplied, **`Authorization` takes precedence**. `anthropic-version` is **ignored**. `anthropic-beta` is ignored for `/messages` **except** that an image using `source.type:"file"` requires `files-api-2025-04-14`.
- **Model mapping:** `claude-opus*` → `deepseek-v4-pro` (**billed at V4 Pro price**); `claude-haiku*` / `claude-sonnet*` → `deepseek-flash`; other `claude*` names → `deepseek-flash`; unsupported names **not** starting with `claude` ⇒ **400**.
- **Request:** `model`, `messages` (min 1; roles `user`/`assistant`), top-level `system` (string, text-block array, or null), `max_tokens` (optional at HTTP level; **the SDK requires it**; values above the model limit are capped; for certain Claude Code / OpenCode main-agent thinking requests the effective limit **can be raised above the requested value** — this parameter is not a strict output cap), `stream`, `thinking{type: enabled|disabled|adaptive, budget_tokens ignored}`, `output_config{effort}` (only `effort`; `format` ignored), `tools`, `tool_choice`, `stop_sequences` (≤16), `temperature`, `top_p`, `metadata{user_id}`.
- **Tools:** client tools supply `name`, `input_schema` (top-level `type` must be `object`), optional `description`, and **omit `type`** — passing `type:"custom"` returns **422**. `tool_choice`: `none`, `auto` (default), `any` (**treated as `auto`; does not force a call**), named `tool` (non-thinking only; **400** in thinking mode). `cache_control` and `disable_parallel_tool_use` are ignored. Plus the server tool `web_search_20250305` with `max_uses`.
- **Tool-result replay (documented, strict):** after an assistant message containing `tool_use`, the **immediately following** message must have role `user` and include a `tool_result` for **every** tool call with matching `tool_use_id`, with all `tool_result` blocks **before any text**. In thinking mode, preserve and pass back the assistant's **complete `content` array** including `thinking` blocks; the service *may* recover omitted thinking from tool-call IDs, but if recovery fails ⇒ **400**. *"Do not rely on automatic recovery."*
- **Content blocks supported:** `text`, `image` (base64/url/file), `thinking`, `tool_use`, `tool_result`, `server_tool_use`, `web_search_tool_result`. **Not supported:** `document`, `search_result`, `redacted_thinking`, `code_execution_tool_result`, `mcp_tool_use`, `mcp_tool_result`, `container_upload`.
- **Server-side web search:** returns a `server_tool_use` block plus a matching `web_search_tool_result`; the service runs the search and continues — the client supplies no result. `usage.server_tool_use` reports `web_search_requests` (attempts, including rejected/over-`max_uses`/failed, so it **can exceed `max_uses`**) and `tool_search` (succeeded and returned ≥1 result). Read from `message_delta.usage` when streaming — **not** from `message_start`.
- **Streaming:** named SSE — `message_start`, `content_block_start|delta|stop`, `message_delta`, `message_stop`, with possible `ping` events. **No `data: [DONE]`.** Keep-alive comments may appear. A timeout before generation may send an **unnamed `data` frame** containing `{"error":{"message":…}}` with no `event` and no top-level `type`, then close. **A connection closing before `message_stop` means an incomplete response.**
- **Usage:** take the **final `message_delta.usage`** and do **not** add it to `message_start`. `input_tokens` **excludes** cache hits; `cache_read_input_tokens` reports hits; `cache_creation_input_tokens` is currently **0**; total input = sum of the three. Also `service_tier`.
- **Pricing:** tokens, at the mapped model's rates.
- **HiveMind:** **NOT NEEDED**. No `/anthropic` base URL, no content-block model (`dto.rs:93`), no `x-api-key` path, and a `data:`+`[DONE]` parser that would mis-handle named events (`client.rs:170-192`). It exists to let Claude Code and similar clients point at DeepSeek; HiveMind is its own client.

---

## 8. Account surfaces

### 8.1 Models

**Endpoint:** `GET https://api.deepseek.com/models`. **Auth:** Bearer. **Cost:** free.

**Response:** `{object:"list", data:[…]}` where each item is:

| Field | Req. | Documented meaning |
|---|---|---|
| `id` | yes | identifier usable in requests |
| `object` | yes | always `"model"` |
| `owned_by` | yes | owning organization |
| `name` | no | display name for model pickers |
| `context_window` | no | total token capacity, **input + output** |
| `max_output_tokens` | no | max accepted `max_tokens` |
| `input_modalities` | no | `text`, `image` |
| `output_modalities` | no | `text` |
| `effort.supported_levels` | yes (nested) | levels accepted by `reasoning_effort` when thinking is on — **`none` is not included** |
| `effort.default_level` | no | server default; always one of `supported_levels`; only present when a default exists |
| `api_capabilities.anthropic_messages.system_prompt_update` | yes (nested) | `leading-only` or `in-history` |

Documented caveat: *"a metadata field may be omitted when the information is not available for that model."*

**HiveMind:** **WIRED (partial)**. `ModelsResponse`/`ModelInfo` parse only `id`, `object`, `owned_by`, `name`, `context_window`, `max_output_tokens` (`dto.rs:883-910`); the local proxy re-serialises `models.data` verbatim (`http.rs:890-898`) so the unparsed fields are **dropped at the provider boundary**, not at the UI. Consequences: the configured `reasoning_effort` is never validated against `effort.supported_levels`; the UI cannot show protocol capability; `context_window` is displayed but never enforced (`ModelsPanel.tsx:247-248`).

### 8.2 Balance

**Endpoint:** `GET https://api.deepseek.com/user/balance`. **Auth:** Bearer. **Cost:** free.

**Response:** `{is_available: bool, balance_infos:[{currency: "CNY"|"USD", total_balance, granted_balance, topped_up_balance}]}` — amounts are **decimal strings**, not numbers.

**Documented deduction rule:** fees are deducted from topped-up or granted balance, **preferring the granted balance** when both exist.

**HiveMind:** **WIRED**. `BalanceResponse`/`BalanceInfo` `dto.rs:913-933`; proxy `http.rs:903-924`; the UI stores `is_available` (`ModelsPanel.tsx:89`) and renders "Account balance is insufficient." when false (`:219`), plus per-currency totals (`:225-228`).

### 8.3 Model limits, prices and concurrency (one table)

Per 1M tokens, USD. Source: `/quick_start/pricing`, fetched 2026-11-07.

| | `deepseek-flash` | `deepseek-v4-pro` |
|---|---|---|
| Served by | DeepSeek-V4.1-Flash-1017 | DeepSeek-V4-Pro-0813 |
| Context | **1M** | **1M** |
| Max output | **384K (393 216)** | **384K (393 216)** |
| Cache-hit input, off-peak / peak | **0.003 / 0.006** | **0.022 / 0.044** |
| Cache-miss input, off-peak / peak | **0.15 / 0.30** | **0.66 / 1.32** |
| Output, off-peak / peak | **0.60 / 1.20** | **1.98 / 3.96** |
| Concurrency limit | **2500** | **500** |
| JSON / tools / Responses / Anthropic / Prefix | ✓ all | ✓ all |
| FIM (Beta) | ✓ non-thinking only | ✓ non-thinking only |
| **Vision** | **✓** | **✗ not supported** |

**Peak windows:** `01:00–04:00` and `06:00–10:00` **UTC, Monday–Friday, excluding Chinese public holidays**. All other hours are off-peak, **including weekends and Chinese public holidays in full**. Off-peak = **half** of peak.

**HiveMind price rows match these numbers exactly** (`pricing.rs:89-115`, tests at `:213-235`; `PRICE_FETCHED_AT = "2026-11-06"`, `pricing.rs:22`). The peak-window arithmetic is `[start, end)` UTC Mon–Fri (`pricing.rs:25`, `:177-197`) — **Chinese public holidays are not excluded**, so cost is overstated on those days (conservative, never understated).

---

## 9. Flag table — image-only and separately billed

Two properties are easy to miss and both change a cost or a product decision. They are stated once, explicitly.

| Surface | Image-only? | Separately billed? | Evidence |
|---|---|---|---|
| **Files API** (`/files`, all four ops) | **YES — images only.** JPEG/PNG/GIF/WebP, format detected from content. No text, PDF, or document path exists. `purpose` must be `user_data`. | **NO documented per-request or per-GB charge.** Cost appears only when the stored image is referenced in a request, as ordinary input tokens (≤1024 tok/image). | `/guides/files_api`; `/api/create-file` |
| **Vision** (`image_url` / `file` blocks) | image **input** only (it is a modality, not a storage surface) | **NO** — billed as ordinary input tokens; documented cap **1024 tokens per image** | `/guides/vision#token-usage`; `/quick_start/pricing` |
| **Search** (`/v1/tools/search`) | no | **YES — $0.75 per 1,000 successful requests** | `/guides/search#billing`; `/quick_start/pricing#search--fetch-pricing` |
| **Fetch** (`/v1/tools/fetch`) | no | **YES — $0.75 per 1,000 successful requests** | same |
| **Responses `web_search` tool** | no | **YES, indirectly** — consumes Search/Fetch operations reported in `usage.server_tool_use`; documented as "can be used as a basis for billing". `deepseek-v4-pro` only. | `/guides/responses_api#server-tool-usage` |
| **Anthropic `web_search_20250305` tool** | no | **YES, indirectly** — `usage.server_tool_use.web_search_requests` / `tool_search` | `/guides/anthropic_api#server-tool-usage` |
| **Server-side search via Harness / Claude Code / Codex** | no | **YES since 2026-10-31 16:00 UTC** — the free window announced on 2026-10-17 has closed as of this research date | `/news/news261017` |
| Chat / Thinking / JSON / Tools / FIM / Prefix / Responses / Messages | no | **NO** — pure token billing (cache-hit, cache-miss, output rates in §8.3) | `/quick_start/pricing` |
| `GET /models`, `GET /user/balance` | no | **NO** — free account reads | `/api/list-models`, `/api/get-user-balance` |

**One consequence worth stating plainly:** HiveMind's money model is token-only and therefore *honest for everything it currently does* and *structurally unable to represent* the only per-request-priced surface in the API. Any Search/Fetch adoption must add a cost dimension before the first call, or the run report becomes false.

---

## 10. Status ledger — the four buckets, separated

### 10.1 DOCUMENTED (read verbatim, official, 2026-11-07)

All 20 named surfaces are documented: Quick Start, the guide/reference index, News/Change Log, Vision, Thinking Mode, Multi-round Conversation, Chat Prefix Completion (Beta), FIM Completion (Beta), JSON Output, Tool Calls, Files API, Search & Fetch, Context Caching, Responses API, Anthropic API, Chat Completions, Messages, Models, Balance, and the four Files operations. §12 is the retrieval record.

Two documentation-quality notes that affect engineering:
- The **FIM 4K cap appears only in the guide**, not in the reference's `max_tokens` schema (`>= 0`). Treat the guide as the operative bound and re-check on change.
- **Chat Prefix's concrete URL** (`/beta/chat/completions`) is derived from the documented `base_url="https://api.deepseek.com/beta"` plus "follows the Chat Completion API"; the page never prints the full path.

### 10.2 WIRED (production path exists and its result is consumed)

| Surface | Scope wired | Seam |
|---|---|---|
| Quick Start transport (base URL, Bearer, key handling) | full | `dto.rs:17`, `client.rs:321-339`, `secret.rs` |
| Chat Completions (non-stream + stream) | full, minus `strict` and `user_id` | `dto.rs:396-575`, `client.rs:350-370`, `:538-620` |
| Thinking Mode | full (planner on, leaves off) | `dto.rs:219-253`, `provider_executor.rs:301-302`, `:332` |
| Multi-round Conversation | full, incl. tool transcript replay | `provider_executor.rs:357-424` |
| JSON Output | **planner only**, no retry | `dto.rs:281-305`, `provider_executor.rs:304`, `:447-459` |
| Tool Calls (function tools, thinking-mode loop) | full first-turn + continuation | `provider_executor.rs:323-339`, `:409-415`, `tool_assembler.rs` |
| Context Caching (counters + pricing) | full counters, priced; hit rate unmeasured | `accounting.rs:234-263`, `pricing.rs:93-115` |
| Models | **partial** — 6 of 10 documented fields parsed | `dto.rs:892-910`, `http.rs:877-901` |
| Balance | full, incl. `is_available` | `dto.rs:913-933`, `http.rs:903-924`, `ModelsPanel.tsx:89` |

### 10.3 NOT WIRED (documented, absent, and adoption is a live product question)

| Surface | Why it is a question, not a decision | What adoption requires |
|---|---|---|
| **Search & Fetch** | An HMAS that researches has no live web. This is the only absent surface with clear product value. | A `PermissionClass::Network` tool (the class exists unused at `tool.rs:38`), approval gate, URL/SSRF policy, bounded results, and a **per-request cost dimension** in `RunAccounting` |
| **Strict tool schema (Beta)** | Tool arguments are validated only *after* generation; a schema violation costs a call and a retry. | `strict:true` per function, the `/beta` base URL, and the restricted schema subset (all properties required, `additionalProperties:false`, no `minLength`/`maxLength`/`minItems`/`maxItems`) |
| **`user_id`** | Documented lever for KVCache isolation and scheduling isolation; single-user local today, so low value — but a multi-tenant or multi-profile future changes that. | a `user_id` field on `ChatRequest` (`dto.rs:396-441` has none), ≤512 UTF-8 bytes |

### 10.4 NOT NEEDED (documented, absent, excluded by a named product decision)

| Surface | Decision and reason |
|---|---|
| **Files API** + **Upload/List/Retrieve/Delete** | Image-only storage (25 GiB / 10 000 files) with no text path; HiveMind's data surface is a local text workspace jail. Wiring adds a second, remote data domain with retention and privacy obligations and no gain. |
| **Vision** | `deepseek-flash`-only image input; HiveMind has no image surface and `ChatMessage.content` is `Option<String>` (`dto.rs:93`). Revisit only if the product gains an image workflow. |
| **Responses API** | Different protocol: semantic SSE with `response.completed/incomplete/failed` and **no `[DONE]`**, different usage names, stateless with no `previous_response_id`. Justified by Codex interop, not by HiveMind's own loop. |
| **Anthropic API / Messages** | Same models, Anthropic framing; exists so Claude Code and similar clients can point at DeepSeek. HiveMind is its own client. Adopting it forks the one-client invariant (`backend/crates/hiverouter-provider/src/lib.rs:3-7`). |
| **Chat Prefix Completion (Beta)** | Forces a prefix the planner does not need; JSON mode plus the "json" instruction already constrains the plan. Would fork the transport to a beta base URL. |
| **FIM Completion (Beta)** | 4K, non-thinking, no tools — an editor code-completion surface, not an agent-chat surface. |

---

## 11. Twenty questions about DeepSeek capability

Each answer is what the official documentation states on 2026-11-07, with the section that carries the detail.

**Q1. Which model IDs are valid today, and is a legacy name safe to send?**
Two: `deepseek-flash` (served by DeepSeek-V4.1-Flash-1017) and `deepseek-v4-pro` (DeepSeek-V4-Pro-0813). The legacy names `deepseek-v4-flash` and `deepseek-v4-flash-vision-exp` are still accepted but the models are retired — requests are served by V4.1-Flash-1017 and billed at the Flash price. (§3.1, §8.3)

**Q2. What are the context and output ceilings, and what is `max_tokens` when I omit it?**
Context **1M** tokens (input + output) on both models; max output **384K = 393 216**. `max_tokens` accepts **1–393216**. Default when omitted: **8K** non-thinking, **64K** thinking, **128K** at `reasoning_effort:"max"`. (§4.1, §8.3)

**Q3. Is thinking on by default, and how do I turn it off so nothing silently reasons?**
On by default at effort `high`. Turn it off with `{"thinking":{"type":"disabled"}}`; a present `thinking.type` **takes precedence** over `reasoning_effort`. `reasoning_effort:"none"` also disables thinking. (§4.2)

**Q4. Which effort values exist, what do the aliases map to, and where do I learn the supported set?**
`none` / `low` / `high` / `max`; aliases `minimal`→`low`, `medium` and `xhigh`→`high`, `ultra`→`max`. The authoritative per-model set is `effort.supported_levels` from `GET /models`, where **`none` is never included** because it disables the mode rather than tuning it. (§4.2, §8.1)

**Q5. When must `reasoning_content` be passed back, and what happens if I drop it?**
If the request **carries `tools`**, the `reasoning_content` of **all prior turns** — including turns with no tool call — must be passed back, and it is concatenated into context. Omitting it returns **HTTP 400**. If the request carries **no `tools`**, the field is not needed and is ignored if sent. (§4.2)

**Q6. Can Chat Completions accept an image, and on which model?**
Yes, via `image_url` (base64 data URL or http(s) link) or a `file` block with a Files API `file_id`; `deepseek-flash` **only** — `deepseek-v4-pro` does not process images (on the Anthropic endpoint it substitutes `[Unsupported Image]`). Images are allowed in **`user` messages only**; `system`/`assistant` images return 400. (§6.1)

**Q7. Can I insert a tool call the model never generated, mid-conversation?**
**Not on Chat Completions.** Mid-conversation tool-call insertion is documented only for the **Anthropic `/messages`** and **Responses** formats. Chat Completions does allow inserting `system` messages mid-conversation. (§4.5)

**Q8. Is there JSON Schema structured output on Chat Completions?**
No — Chat Completions offers only `response_format` `text` or `json_object`, and `json_object` requires a "json" instruction in the prompt. **`json_schema` exists only in the Responses API** (`text.format` with `name` + `schema`). (§4.4, §7.2)

**Q9. How is a stream framed, and where does usage arrive?**
Data-only SSE lines, terminated by `data: [DONE]`. There is **no separate usage-only chunk**: the statistics ride on the **last content chunk**, whose `choices` array has exactly one element with no new content and a non-null `finish_reason`. `stream_options.include_usage` only makes `usage` visible (null) on every earlier chunk; setting it without `stream:true` returns 400. (§4.1)

**Q10. Which `finish_reason` values mean "this is not a success"?**
Successes are only `stop` and `tool_calls`. `length`, `content_filter`, `insufficient_system_resource`, and `aborted` are all non-success outcomes, and `length` can also mean the context window was exceeded, not just the output cap. FIM adds the same set minus `tool_calls`. (§4.1, §5.2)

**Q11. How does a cache hit actually happen now?**
Not by loose prefix overlap. The cache is organised as **cache prefix units**, and a request must **fully match** a persisted unit. Units are persisted (a) at the end of the user input and the end of the model output of each request, (b) when the system detects a common prefix across requests, and (c) at fixed token intervals inside long inputs/outputs. So `A+B` → `A+C` does **not** hit; after the system learns `A`, a later `A+D` does. (§4.6)

**Q12. What does a cache hit actually save?**
The input rate difference: Flash cache-hit **$0.003/$0.006** vs cache-miss **$0.15/$0.30** per 1M (off-peak/peak) — a **50×** gap; Pro **$0.022/$0.044** vs **$0.66/$1.32** — a **30×** gap. (`prompt_tokens` always equals hit + miss, so the split is the only thing that matters.) (§8.3)

**Q13. Which endpoints are billed per request rather than per token?**
Exactly two: **Search** and **Fetch**, at **$0.75 per 1,000 successful requests** each. Everything else — chat, thinking, tools, JSON, vision, FIM, prefix, Responses, Messages, Files — is token-billed, and the two `GET` account reads are free. (§9)

**Q14. What exactly makes a Search or Fetch call billable?**
HTTP **200** *and* a produced result — Search must return a **non-empty `results` array**; Fetch must return page content. Failed requests, empty result sets, and timeouts are **not billed**. `count` does not change the price. (§7.1)

**Q15. What are the Search/Fetch rate limits and which errors are retryable?**
Per account and independent: **Search QPS 100**, **Fetch QPS 25**, plus an additional **per-domain** QPS limit on Fetch; all account-level regardless of key. Retry **408, 429, 5xx**; top up on **402**; do not retry other 4xx unchanged. Note the endpoint-specific statuses **403** (Fetch rejected by security policy) and **404** (no usable content), which are refusals, not retries. (§7.1)

**Q16. Can the Files API store a text document or a PDF?**
No. It is **image storage only** — JPEG, PNG, GIF, WebP, detected from content; `purpose` must be `user_data`. There is no documented document or text purpose, and the Responses API states plainly that **file inputs are not supported**. (§6.2, §7.2)

**Q17. Do I need an Anthropic API key to use the Anthropic-format endpoint?**
No. `POST /anthropic/v1/messages` authenticates with the **same DeepSeek key**, as `x-api-key` or `Authorization: Bearer`; if both are sent, `Authorization` wins. `anthropic-version` is ignored; `anthropic-beta` is only needed for `file`-sourced images and the Files endpoints. (§7.3)

**Q18. Are Responses and Anthropic conversations stateful?**
No, both are documented **stateless**. Responses has no `previous_response_id` and no `conversation`; `store` is always `false`. Anthropic requires the full `messages` history each turn. Chat Completions is likewise stateless. (§7.2, §7.3, §4.3)

**Q19. What are the concurrency limits and what does exceeding them return?**
**2500** concurrent requests for `deepseek-flash` and **500** for `deepseek-v4-pro`, counted per account from send until the response completes, regardless of key. Exceeding the limit returns **HTTP 429**. Capacity expansion is available at no additional documented cost. HiveMind's local limiter is **8** (`budget.rs:43`). (§8.3, §4.1)

**Q20. Is there an official offline tokenizer I can use for prompt-token estimation?**
Yes — `/quick_start/token_usage` publishes a downloadable tokenizer (`https://cdn.deepseek.com/api-docs/deepseek_v4_tokenizer.zip`) and documented ratios: **1 English character ≈ 0.3 token**, **1 Chinese character ≈ 0.6 token**, with the API's returned `usage` as the source of truth. It also publishes an image-token calculator and states the **1024 tokens/image** cap. (§12; §6.1)

---

## 12. Source table — official pages retrieved in this pass

Every row was fetched with `HTTP 200` on **2026-11-07** (America/Chicago). No page was read from cache or memory.

| # | URL | What it establishes here |
|---|---|---|
| S1 | https://api-docs.deepseek.com/ | base URLs (OpenAI + Anthropic), model IDs, legacy re-routing, Bearer auth, first call |
| S2 | https://api-docs.deepseek.com/quick_start/pricing/ | model versions, 1M context, 384K output, per-model feature matrix (Vision ✗ on pro), all prices, peak windows + holiday exclusion, concurrency, Search/Fetch pricing, deduction rule |
| S3 | https://api-docs.deepseek.com/quick_start/token_usage/ | character→token ratios, official offline tokenizer download, image token calculator |
| S4 | https://api-docs.deepseek.com/quick_start/rate_limit/ | concurrency numbers, `user_id` semantics, keep-alive behaviour, 10-minute inference start cutoff |
| S5 | https://api-docs.deepseek.com/quick_start/error_codes/ | 400 / 401 / 402 / 422 / 429 / 500 / 503 causes and retry guidance |
| S6 | https://api-docs.deepseek.com/api/create-chat-completion | full Chat Completions request/response schema, `max_tokens` bounds and defaults, `thinking`, `reasoning_effort` aliases, `tools`/`tool_choice`, `response_format`, `stream_options`, `user_id`, deprecated penalties, `finish_reason` set, usage fields, SSE `[DONE]` |
| S7 | https://api-docs.deepseek.com/api/list-models/ | `/models` schema incl. `effort.supported_levels`, `default_level`, `input_modalities`, `api_capabilities.anthropic_messages.system_prompt_update` |
| S8 | https://api-docs.deepseek.com/api/get-user-balance/ | `/user/balance` schema, `is_available`, decimal-string amounts |
| S9 | https://api-docs.deepseek.com/api/create-response/ | `POST /responses`, statelessness, `text.format` incl. `json_schema`, `web_search` (pro only), output item types, usage + `server_tool_use` |
| S10 | https://api-docs.deepseek.com/api/create-message/ | `POST /anthropic/v1/messages` + `/anthropic/messages` alias, dual auth precedence, `max_tokens` capping behaviour, `thinking`/`output_config`, tool schemas, `metadata.user_id` |
| S11 | https://api-docs.deepseek.com/api/create-completion/ | `POST /completions` on the beta base, FIM fields, `text_completion` response, FIM `finish_reason` set (no `tool_calls`) |
| S12 | https://api-docs.deepseek.com/api/create-file/ | `POST /files` multipart fields, 64 MiB, 10-minute upload, file object schema |
| S13 | https://api-docs.deepseek.com/api/search/ | `POST /v1/tools/search` fields, bounds (1–128 chars, count 1–50), result schema incl. `rerank_score`/`authority` |
| S14 | https://api-docs.deepseek.com/api/fetch/ | `POST /v1/tools/fetch`, `url` ≤8192 chars, `{title, content}` response |
| S15 | https://api-docs.deepseek.com/guides/vision/ | image support on `deepseek-flash` only, three input methods, `detail`, token cost and 1024-token cap, full limit table, `user`-only restriction, Anthropic/Responses shapes |
| S16 | https://api-docs.deepseek.com/guides/thinking_mode/ | toggle/effort matrix, alias table, parameter no-effects, `reasoning_content` pass-back rules with the 400 |
| S17 | https://api-docs.deepseek.com/guides/multi_round_chat/ | statelessness and the append-only history pattern |
| S18 | https://api-docs.deepseek.com/guides/chat_prefix_completion/ | Beta base URL, last-message `role:assistant` + `prefix:true`, `stop` pairing |
| S19 | https://api-docs.deepseek.com/guides/fim_completion/ | Beta base URL, **4K max tokens** |
| S20 | https://api-docs.deepseek.com/guides/json_mode/ | `json_object` + "json" instruction requirement, **non-JSON replaced with spaces** caveat |
| S21 | https://api-docs.deepseek.com/guides/tool_calls/ | function-tool flow, thinking-mode tools, **no mid-conversation insertion on Chat Completions**, `strict` Beta requirements and supported JSON-Schema subset |
| S22 | https://api-docs.deepseek.com/guides/files_api/ | image-only scope, all four operations, query params, file object, use-in-chat `file` block, **Anthropic-compatible Files API differences**, limits |
| S23 | https://api-docs.deepseek.com/guides/search/ | Search/Fetch request+response tables, query operators, **error table with 403/404/408/413/504**, **per-request billing rules**, QPS limits |
| S24 | https://api-docs.deepseek.com/guides/kv_cache/ | **cache prefix units**, the three persistence triggers, full-match hit rule, worked examples, counters |
| S25 | https://api-docs.deepseek.com/guides/responses_api/ | semantic SSE event list, no `[DONE]`, image input, full parameter/tool compatibility tables, `server_tool_use` |
| S26 | https://api-docs.deepseek.com/guides/anthropic_api/ | Anthropic base URL, dual auth, model mapping, compatibility tables, tool-result replay rules, `server_tool_use`, `message_stop` streaming and incomplete-connection semantics |
| S27 | https://api-docs.deepseek.com/updates/ | change log; newest entry **2026-10-17**; full dating of model and pricing changes |
| S28 | https://api-docs.deepseek.com/news/news261017 | V4.1-Flash-1017 release; Search/Fetch opening at $0.75/1,000; **server-side search free until 2026-10-31 16:00 UTC** |

**Not fetched directly (documented only via S22):** `https://api-docs.deepseek.com/api/list-files`, `/api/retrieve-file`, `/api/delete-file`. Their endpoints, query parameters and responses are taken from the Files API guide (S22) and from the reference navigation, not from a direct page read — marked here so the gap is visible rather than implied.
**Referenced but not fetched:** `https://platform.deepseek.com/api_keys`, `https://platform.deepseek.com/top_up`, `https://status.deepseek.com/` (operational surfaces, not API contracts).

---

## 13. Local corroboration read in this pass

Read-only; no file was modified except this one.

| Path | What it established |
|---|---|
| `backend/crates/hiverouter-provider/src/lib.rs:1-40` | one client, one budget/cost owner; `f64` banned on cost paths; module map |
| `backend/crates/hiverouter-provider/src/dto.rs:17`, `:27-56`, `:88-106`, `:219-305`, `:396-575`, `:578-628`, `:883-933` | base URL, model enum, string-only `content`, thinking/effort/format types, request DTO (no `user_id`/`stop`/`prefix`/`strict`/`stream_options`), usage fields, models/balance DTOs |
| `backend/crates/hiverouter-provider/src/client.rs:170-192`, `:266-339`, `:321-327`, `:350-370`, `:488-530`, `:538-620` | keep-alive-tolerant SSE parser; `GET`-only account helpers; single non-beta endpoint; SSRF-free Bearer construction; status mapping (5xx retryable, 400/422 catch-all); stream handle |
| `backend/crates/hiverouter-provider/src/tool_assembler.rs` | index-keyed assembly of streamed tool-call fragments |
| `backend/crates/hiverouter-provider/src/accounting.rs:196-330` | Decimal cost, cache-hit/miss required or INCOMPLETE, NULL never zero |
| `backend/crates/hiverouter-provider/src/pricing.rs:19-25`, `:89-147`, `:177-197` | sourced price rows matching §8.3, `[start,end)` UTC peak windows, no holiday exclusion |
| `backend/crates/hiverouter-provider/src/budget.rs:24-52`, `:94-103`, `:269-280` | $0.50 run cap, 400K combined-token budget, 40-call cap, in-flight 8, single key, retry/`Retry-After` policy |
| `backend/crates/hiverouter-core/src/provider_executor.rs:89-151`, `:239-281`, `:284-348`, `:357-437`, `:447-459` | prompt-token estimate, streamed finish-reason refusal, first-turn tool advertisement, tool transcript replay, `bytes/4` estimator, non-stream finish-reason check |
| `backend/crates/hiverouter-core/src/config.rs:16-31`, `:61-65`, `:102-128` | model default, planner/leaf caps 16K/8K, effort and thinking env knobs |
| `backend/crates/hiverouter-core/src/http.rs:527-528`, `:877-924` | `/api/provider/models` and `/api/provider/balance` proxies; no key reaches the browser |
| `backend/crates/hiverouter-tools/src/tool.rs:38`, `:49`, `:108-127`, `:319` | `PermissionClass::Network` exists and is unused; tool id enum; standard registry |
| `web/src/panels/ModelsPanel.tsx:20-28`, `:72-89`, `:219-248` | UI consumes `is_available`, balance rows, `context_window`, `max_output_tokens` |

**Secret hygiene:** no API key, token, `Authorization` header value, account identifier, or balance figure appears in this document. `DEEPSEEK_API_KEY` is named as an environment variable, which is a name, not a value. No paid endpoint was called; no credential was read.

---

## 14. Top-10 gap list

Ordered by consequence. "H" = can break a run or make a money claim false; "M" = degrades reliability, diagnosis, or a capability claim; "L" = future-only.

| # | Gap | Risk | Evidence | Minimal correct action |
|---|---|---|---|---|
| 1 | **Prompt-token estimate is `serialized_bytes / 4`**, an unquantified heuristic. It understates ASCII English (~0.25 vs documented ~0.3 token/char) and overstates CJK (~0.75 vs ~0.6 token/char). The 400K combined-token preflight therefore has an unknown error direction and magnitude. | **H** | `provider_executor.rs:431-437`; docs S3 | Adopt the **official offline tokenizer** (S3) or apply a documented worst-case multiplier with an explicit margin, and state which. |
| 2 | **No context-window preflight.** The 1M `context_window` from `/models` is displayed but never enforced; an over-long prompt is discovered only by a 400 after the request is built (and, on a stream, after the connection opens). | **H** | `ModelsPanel.tsx:247-248`; no check in `provider_executor.rs`/`budget.rs` | Parse `context_window`, refuse before send, test that a pre-send refusal makes zero HTTP calls. |
| 3 | **`/models` metadata is dropped at the DTO boundary.** `effort.supported_levels`, `default_level`, `input_modalities`, `output_modalities`, `api_capabilities` are never parsed, so a configured `reasoning_effort` cannot be validated and protocol capability cannot be shown. | **M** | `dto.rs:892-910`; docs S7 | Extend `ModelInfo`, validate configured effort at boot, surface capability in the UI. |
| 4 | **`400` and `422` are reported as "undocumented provider HTTP status"** and mapped to non-retryable `ProviderInvalidResponse`, although both are documented with distinct causes (invalid format vs invalid parameters). This misdirects every schema bug. | **M** | `client.rs:520-523`; docs S5 | Map 400/422 to a request-schema refusal naming the documented cause. |
| 5 | **Chinese public holidays are not excluded from peak windows.** Cost is overstated on those days — conservative, but wrong in a money report that claims to follow the documented table. | **M** | `pricing.rs:177-197`; docs S2 | Add a dated, sourced holiday table; test a known holiday. |
| 6 | **Search & Fetch are absent**, so HMARS research agents have no live-web capability. This is the highest-value missing surface and the only per-request-billed one. | **M** | `tool.rs:108-127`; `pricing.rs:49-64`; docs S23 | New `Network` tool + approval + URL policy + bounded results, and a **per-request cost dimension** before the first call. |
| 7 | **JSON-mode planning has no retry.** The docs explicitly warn that `json_object` output can be **only spaces even with `finish_reason:"stop"`** and advise retrying; HiveMind refuses honestly but fails the run on the first such response. | **M** | `provider_executor.rs:304`, `:447-459`; docs S20 | One bounded retry on blank/short plan content, with the retry counted against the call cap. |
| 8 | **`strict` tool schema (Beta) is not wired**, so a schema-violating tool call is only caught after generation — one wasted call per violation. | **M** | no `strict` field `dto.rs:361-392`; docs S21 | Only if measured violations justify it: `strict:true` + `/beta` base + schema-subset validation. |
| 9 | **Files and Vision are absent** (image-only surfaces; no content-part model; `content` is a string). Harmless today, blocking for any multimodal roadmap. | **L** | `dto.rs:93`; docs S15, S22 | A content-part type plus the documented limit table, gated on Flash; or keep the decision written down. |
| 10 | **Responses, Anthropic/Messages, FIM, Prefix, and `user_id` are absent** — four alternative protocols plus one isolation lever. Each is correct to omit for HiveMind's own loop; the risk is only that a future claim of "full API coverage" would be false. | **L** | `client.rs:321-327`; `dto.rs:396-441`; docs S9, S10, S11, S18, S19 | Keep the §10.4 decision table current; do not add a protocol to tick a box. |

**Sweep performed:** the mandatory stub sweep (§23 pattern list) was run over this file with `Select-String -AllMatches`. It returns **exactly one hit — this paragraph**, because the phrase naming the sweep contains one of the pattern words. Classification: **permitted**, not a defect — it is the sweep record, not unfinished code. Every other line is clean, and no endpoint, price, field, limit, or status code in this file is invented: each traces to a §12 URL or a §13 path.

---

## 15. Unknowns — not proven in this pass

| # | Unknown | Why it matters | What would prove it |
|---|---|---|---|
| U1 | Live account state: which models the key returns, `is_available`, current balance | The UI claims live account data; this pass made **no** paid call | A keyed `GET /models` + `GET /user/balance`, values recorded with no key material |
| U2 | Whether the live `/models` payload actually contains `effort.supported_levels` and `api_capabilities` for both models | Docs say metadata "may be omitted" | A keyed `/models` body captured verbatim |
| U3 | Realised cache-hit rate for HiveMind's planner/leaf/tool-continuation prompts under the **cache prefix unit** rule | Any "caching saves us X" claim depends on it | A keyed multi-turn run with `prompt_cache_hit_tokens`/`miss_tokens` recorded |
| U4 | Whether `deepseek-flash` on this account advertises `image` in `input_modalities` | Vision is out of scope, but the catalogue is the source of truth | Same capture as U2 |
| U5 | Whether server-side search billing has actually begun for this account (the 2026-10-31 16:00 UTC date has passed) | Affects any future Responses/Anthropic `web_search` use | A keyed request with `usage.server_tool_use` recorded, plus a billing statement |
| U6 | Chinese public-holiday dates and whether the exemption is applied per billing | Row 5 of §14 overstates on those days | An official holiday calendar + a keyed usage test on a holiday |
| U7 | Observed distribution of `finish_reason` values in production streams (all six, or only `stop`/`length`?) | §4.1 treats the documented set as complete; that is a schema argument, not an observation | A long-output / filtered / cancelled run with the final chunk captured |
| U8 | Whether the FIM 4K guide bound is enforced as a hard `max_tokens` cap (the reference schema says only `>= 0`) | Only matters if FIM is ever adopted | A keyed FIM call at 4096 and at 4097 |
| U9 | Whether the local `bytes/4` estimator over- or under-counts for **this** prompt corpus | Decides whether gap 1 needs a multiplier or a tokenizer | A tokenizer run over N stored prompts compared with reported `usage.prompt_tokens` |

---

## 16. Change control

- **Re-verify before relying on any number here.** Re-fetch the §12 URLs and diff `/updates` against **2026-10-17**. The API docs are live and unversioned (`version_pinned: false`), so this file's facts expire.
- **Prices, peak windows, and the model list are the fastest-rotting rows.** They are the ones that make a money report false.
- **Two rows are derived, not printed:** Chat Prefix's full path (§5.1) and FIM's 4K bound (§5.2, guide-only). Flag them again if the docs change.
- **Supersedes nothing automatically.** `docs/trace/provider/01-deepseek-capability-matrix.md` and `docs/provider/DEEPSEEK-INTEGRATION.md` describe an earlier tree; three of that matrix's high-risk findings are no longer true on the current code (first-turn tools now advertised, 5xx now retried, prompt tokens now estimated, `is_available` now consumed). This file's §10 reflects the tree as read on 2026-11-07 and should be treated as the current status ledger for the DeepSeek surface.


