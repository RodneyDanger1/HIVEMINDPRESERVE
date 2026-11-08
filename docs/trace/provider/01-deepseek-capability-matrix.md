# DeepSeek capability matrix — HiveMind

**Status:** RESEARCH COMPLETE — read-only pass. No product source was edited.
**Research date:** 2026-11-07 (America/Chicago).
**Repository:** `hiverouter-v1` (local HMAS: one localhost Rust server, one DeepSeek key, planner → worker DAG, jailed tools, SQLite run/usage ledger).
**Deliverable:** this file only — `docs/trace/provider/01-deepseek-capability-matrix.md`.
**Documentation pin:** `version_pinned: false`; `version_source: https://api-docs.deepseek.com/updates/`; newest documented change is **2026-10-17**. The API docs are live/unversioned, so every "confirmed" fact below is dated 2026-11-07 and can rot.
**Evidence basis:** local source read at the current working tree; official docs fetched in this session (all HTTP 200). **No live DeepSeek API call was made in this pass**, and no credential value appears in this document. Statements about live account state are marked unknown.

---

## How to read the status column

| Status | Meaning in this document |
|---|---|
| **WIRED** | A production code path constructs the documented field/endpoint, a caller reaches it, and the result is consumed (persisted, displayed, or accounted). |
| **PARTIAL** | A code path exists, but a documented sub-field, a required first step, or an end-to-end leg is missing, so the capability cannot be claimed as complete. |
| **NOT WIRED** | No production code path exists for the endpoint/field. The library may have unused pieces, but nothing sends or consumes it. |
| **NOT APPLICABLE** | Documented, but this product has no honest use for it (no product surface, or the docs say it has no effect). |

**Risk** is the consequence of leaving the row as-is: **H** = can break a run, misstate money/usage, or make a claimed capability false; **M** = can degrade reliability or mislead an operator; **L** = cosmetic/future-only.

---

## One-screen answer

- **Wired end-to-end:** Chat Completions transport and Bearer auth, both documented model IDs, thinking toggle, planner reasoning effort, `reasoning_content` pass-back, JSON mode on the planner, SSE streaming, usage counters (including cache and reasoning tokens), Decimal cost accounting, usage API/UI, `GET /models` (IDs and limits), `GET /user/balance` (rows).
- **Half-wired:** function tools. DTOs, streamed assembly, tool execution and continuation replay all exist, but the **first** leaf request never advertises `tools`, so a compliant model cannot start the tool loop.
- **Not wired:** strict tools (Beta), Files API, Search API, Fetch API, Responses API, Anthropic Messages API, FIM (Beta), Chat Prefix Completion (Beta), vision/image input, server-side `web_search`, `user_id` cache isolation, pre-send context/token-window enforcement.
- **Not applicable:** `temperature` / `top_p` / penalties / `logprobs` / `stop` tuning and `stream_options.include_usage` for this product's use of the API.

The honest headline is therefore: **the core chat data path is genuinely integrated; the tool loop is not usable end-to-end; the documented long tail is a set of product decisions, not a checklist to implement.**

---

## Capability matrix

### A. Core Chat Completions path

| # | Capability | Documented endpoint / field | Source URL | Status | Exact seam (`file:line`) | Risk |
|---|---|---|---|---|---|---|
| 1 | Chat transport, base URL, Bearer auth | `POST https://api.deepseek.com/chat/completions`; `Authorization: Bearer` | https://api-docs.deepseek.com/ | **WIRED** | `backend/crates/hiverouter-provider/src/client.rs:321-327` (endpoint), `:472-490` (send); `backend/crates/hiverouter-provider/src/secret.rs:16-17`, `:72-74` (key); `backend/crates/hiverouter-provider/src/dto.rs:17-20` | **L** — one global base URL; no `/beta` or `/anthropic` variant. |
| 2 | Model IDs and selection | `model` = `deepseek-flash` \| `deepseek-v4-pro`; legacy `deepseek-v4-flash*` names still routed | https://api-docs.deepseek.com/ | **WIRED** | `backend/crates/hiverouter-provider/src/dto.rs:27-56`; `backend/crates/hiverouter-core/src/config.rs:91-101` | **M** — a new model ID requires a code change; legacy IDs are deliberately refused; one model serves both planner and leaves. |
| 3 | Model limits and `max_tokens` | Context 1M; max output 384K (`max_tokens` 1–393216); defaults 8K non-thinking, 64K thinking, 128K at max effort | https://api-docs.deepseek.com/quick_start/pricing/ ; https://api-docs.deepseek.com/api/create-chat-completion/ | **PARTIAL** | `backend/crates/hiverouter-provider/src/dto.rs:22-23`, `:404-406`, `:546-552`; `backend/crates/hiverouter-core/src/config.rs:18-20`, `:159-169` | **H** — `max_tokens` is validated, but the 1M context window is never measured/enforced; local planner/leaf caps are 16K/8K, not the documented defaults; oversized prompts can 400 after a spend. |
| 4 | Thinking toggle | `thinking: {"type": "enabled" \| "disabled"}` | https://api-docs.deepseek.com/guides/thinking_mode | **WIRED** | `backend/crates/hiverouter-provider/src/dto.rs:219-253`, `:407-409`; `backend/crates/hiverouter-core/src/provider_executor.rs:274`, `:303`; `backend/crates/hiverouter-core/src/config.rs:102-109` | **L** — planner on, leaves off by default; both are explicit, never left to server default. |
| 5 | Reasoning effort | `reasoning_effort: "none" \| "low" \| "high" \| "max"`; aliases `minimal/medium/xhigh/ultra` mapped by the server | https://api-docs.deepseek.com/guides/thinking_mode ; https://api-docs.deepseek.com/api/create-chat-completion/ | **WIRED (planner only)** | `backend/crates/hiverouter-provider/src/dto.rs:255-267`, `:410-412`; `backend/crates/hiverouter-core/src/provider_executor.rs:275`; `backend/crates/hiverouter-core/src/config.rs:30-31`, `:110-113`, `:146-156` | **M** — leaves send no effort (server default `high` when thinking is on); no validation against the model's `/models` `effort.supported_levels`. |
| 6 | `reasoning_content` pass-back with tools | Prior assistant `reasoning_content` must be passed back in requests carrying `tools`; omitted → HTTP 400 | https://api-docs.deepseek.com/guides/thinking_mode | **WIRED** | `backend/crates/hiverouter-provider/src/dto.rs:184-194`; `backend/crates/hiverouter-core/src/provider_executor.rs:362-366` | **M** — only transcript turns that produced tool results are replayed; the docs require all prior turns once `tools` is present. Correct only if the transcript is complete. |
| 7 | Function tools | `tools: [{type:"function", function:{name,description,parameters}}]`; response `tool_calls`; `role:"tool"` + `tool_call_id` | https://api-docs.deepseek.com/guides/tool_calls ; https://api-docs.deepseek.com/api/create-chat-completion/ | **PARTIAL — first-turn offer missing; loop unreachable in production** | `backend/crates/hiverouter-provider/src/dto.rs:361-392`; `backend/crates/hiverouter-core/src/tool_loop.rs:200-219`; `backend/crates/hiverouter-core/src/execution.rs:2806-2929`; **`backend/crates/hiverouter-core/src/provider_executor.rs:296-304` (no `.with_tools`)**, `:378-384` (tools only in continuation) | **H** — `execute()` builds `ChatRequest` without `tools`; the engine only calls `continue_with_tools()` after `final_reply.tool_calls` is non-empty (`execution.rs:2814`). A compliant model cannot emit `tool_calls` when no tools are supplied (`tool_choice` defaults to `none`). Tests inject tool calls unconditionally (`tests/tool_loop_matrix.rs:106-124`) and assert request counts, not bodies (`:280-285`). |
| 8 | `tool_choice` | `none` \| `auto` \| `required` \| named function; `required`/named rejected in thinking mode | https://api-docs.deepseek.com/api/create-chat-completion/ | **PARTIAL** | `backend/crates/hiverouter-provider/src/dto.rs:269-279`, `:476-481`; `backend/crates/hiverouter-core/src/provider_executor.rs:384` | **M** — named choice is not modeled; `required` is refused in thinking mode (correct per docs); continuation always sends `auto`. |
| 9 | Strict tool schema (Beta) | `function.strict: true`; requires `base_url = https://api.deepseek.com/beta` | https://api-docs.deepseek.com/guides/tool_calls | **NOT WIRED** | `backend/crates/hiverouter-provider/src/dto.rs:361-392` (no `strict` field); `backend/crates/hiverouter-provider/src/client.rs:321-327` (non-beta endpoint) | **M** — local tool arguments are validated only after generation; adding strict requires a beta base URL and the restricted JSON-Schema subset (all properties required, `additionalProperties:false`). |
| 10 | JSON output | `response_format: {"type":"json_object"}`; prompt must contain "json"; server replaces unrecognised JSON with spaces | https://api-docs.deepseek.com/guides/json_mode | **WIRED (planner only)** | `backend/crates/hiverouter-provider/src/dto.rs:281-305`, `:419-421`; `backend/crates/hiverouter-core/src/provider_executor.rs:25-28`, `:277` | **M** — blank/space content is refused honestly (`message_text`), but there is no retry; a single malformed plan fails the run; 16K planner cap can truncate a large DAG. |
| 11 | Streaming SSE | `stream: true`; data-only SSE; `data: [DONE]`; usage rides on the last content chunk with non-null `finish_reason` | https://api-docs.deepseek.com/api/create-chat-completion/ | **WIRED** | `backend/crates/hiverouter-provider/src/client.rs:164-192` (parser), `:558-620` (open), `:624-723` (pump); `backend/crates/hiverouter-core/src/execution.rs:169-177`, `:1763-1789` (delta sink) | **M** — retries stop once response headers arrive, so a stream that fails before the first delta still fails; only `length` finish reason is treated as incomplete. |
| 12 | Usage counters | `usage.prompt_tokens`, `completion_tokens`, `total_tokens`, `prompt_tokens_details.cached_tokens`, `prompt_cache_hit_tokens`, `prompt_cache_miss_tokens`, `completion_tokens_details.reasoning_tokens` | https://api-docs.deepseek.com/api/create-chat-completion/ | **WIRED** | `backend/crates/hiverouter-provider/src/dto.rs:578-628`, `:749-764`; `backend/crates/hiverouter-provider/src/client.rs:674-681`; `backend/crates/hiverouter-provider/src/accounting.rs:196-293`; `backend/crates/hiverouter-core/src/execution.rs:698-719`; `backend/crates/hiverouter-core/src/http.rs:811-844`; `web/src/panels/ModelsPanel.tsx:84-134` | **L** — absent usage becomes `E_USAGE_INCOMPLETE`, never 0 (honest); the nested `prompt_tokens_details.cached_tokens` alias is not parsed, but the required top-level hit/miss fields are; no prompt-side estimate. |
| 13 | Context cache counters and pricing | Automatic disk cache; `prompt_cache_hit_tokens` / `prompt_cache_miss_tokens`; cache-hit and cache-miss prices | https://api-docs.deepseek.com/guides/kv_cache ; https://api-docs.deepseek.com/quick_start/pricing/ | **WIRED** | `backend/crates/hiverouter-provider/src/dto.rs:590-598`; `backend/crates/hiverouter-provider/src/accounting.rs:233-265`; `backend/crates/hiverouter-provider/src/pricing.rs:86-115`; `web/src/panels/ModelsPanel.tsx:161-164` | **M** — the hit/miss split is required or the run is INCOMPLETE; peak-window holiday gap is row 31; no request-side cache controls are documented or needed. |
| 14 | Run usage API and UI | `GET /api/usage/runs/{run_id}` | Local surface (not a DeepSeek endpoint) | **WIRED** | `backend/crates/hiverouter-core/src/http.rs:494`, `:811-844`; `web/src/panels/ModelsPanel.tsx:84-134`, `:153-168` | **L** — usage is persisted only when the run reaches terminal status (`execution.rs:1305`); the UI shows "not reported" before that. |

### B. Account APIs

| # | Capability | Documented endpoint / field | Source URL | Status | Exact seam (`file:line`) | Risk |
|---|---|---|---|---|---|---|
| 15 | Model catalogue and limits | `GET /models`; `data[].id`, `object`, `owned_by`, `name`, `context_window`, `max_output_tokens`, `input_modalities`, `output_modalities`, `effort.supported_levels/default_level`, `api_capabilities.anthropic_messages.system_prompt_update` | https://api-docs.deepseek.com/api/list-models | **PARTIAL** | `backend/crates/hiverouter-provider/src/client.rs:261-268`; `backend/crates/hiverouter-provider/src/dto.rs:881-909` (only 6 of the documented fields); `backend/crates/hiverouter-core/src/http.rs:496`, `:846-866`; `web/src/panels/ModelsPanel.tsx:52-82`, `:194-206` | **M** — effort levels, modalities and protocol capabilities are silently dropped; the UI lists every provider model as "available" even though `Model::from_id` accepts only two; limits are displayed but not used for validation. |
| 16 | Account balance | `GET /user/balance`; `is_available`, `balance_infos[].{currency,total_balance,granted_balance,topped_up_balance}` | https://api-docs.deepseek.com/api/get-user-balance | **PARTIAL** | `backend/crates/hiverouter-provider/src/client.rs:270-277`; `backend/crates/hiverouter-provider/src/dto.rs:911-933`; `backend/crates/hiverouter-core/src/http.rs:497`, `:868-885`; `web/src/panels/ModelsPanel.tsx:180-191` | **L** — `is_available` is returned by the proxy but never read by the UI; an unavailable account is shown as balance rows without the explicit "not available" flag. |
| 17 | Retry and error mapping | 400 invalid format, 401 auth, 402 balance, 422 invalid parameters, 429 rate limit, 500 server error, 503 overloaded; 500/503 say retry | https://api-docs.deepseek.com/quick_start/error_codes | **PARTIAL** | `backend/crates/hiverouter-provider/src/client.rs:492-526` (status map), `:404-441` (retry loop); `backend/crates/hiverouter-contract/src/refusal.rs:254-268`; `backend/crates/hiverouter-provider/src/budget.rs:43-46` | **H** — 401/402 correct and never retried; 429 retried with `Retry-After`; **500/503 are mapped to non-retryable `E_PROVIDER_INVALID_RESPONSE`**; 400/422 are reported as "response shape" failures, not request-schema failures, which misdirects diagnosis. |

### C. Alternative protocols (same models, different wire formats)

| # | Capability | Documented endpoint / field | Source URL | Status | Exact seam (`file:line`) | Risk |
|---|---|---|---|---|---|---|
| 18 | Responses API | `POST /responses`; `input`, `instructions`, `reasoning.effort`, `max_output_tokens`, `text.format` (`json_object`/`json_schema`), `tools` (`function`, `custom:apply_patch`, `web_search`), semantic SSE ending `response.completed/incomplete/failed` (no `[DONE]`) | https://api-docs.deepseek.com/guides/responses_api ; https://api-docs.deepseek.com/api/create-response | **NOT WIRED** | No `/responses` path: `backend/crates/hiverouter-provider/src/client.rs:321-327`; parser assumes `data:` + `[DONE]`: `client.rs:164-192`; DTO set is Chat-only: `dto.rs:749-764` | **H if adopted** — not a drop-in: different SSE framing, different terminal events, different usage names (`input_tokens`/`output_tokens`), no `previous_response_id`. A second DTO + parser + accounting path would be required, contradicting the one-client rule in `backend/crates/hiverouter-provider/src/lib.rs:3-7`. |
| 19 | Anthropic Messages API | `POST /anthropic/v1/messages` (compatible `/anthropic/messages`); `system`, `messages`, `max_tokens`, `thinking`, `output_config.effort`, `tools`, named SSE ending `message_stop` (no `[DONE]`); `x-api-key` or Bearer | https://api-docs.deepseek.com/guides/anthropic_api ; https://api-docs.deepseek.com/api/create-message | **NOT WIRED** | Same Chat-only transport: `client.rs:321-327`; parser `client.rs:164-192`; request DTO `dto.rs:394-422` | **H if adopted** — content blocks, thinking blocks, `tool_result` ordering and Anthropic usage names (`cache_read_input_tokens`) are a separate protocol. Only justified by external-client interop (Claude Code), not by HiveMind's own loop. |
| 20 | Files API | `POST /files` (multipart, `purpose=user_data`, ≤64 MiB), `GET /files`, `GET /files/{id}`, `DELETE /files/{id}`; reference by `file_id` in a `file` content block; Anthropic endpoint needs `anthropic-beta: files-api-2025-04-14` | https://api-docs.deepseek.com/guides/files_api | **NOT WIRED** | No `/files` method: `backend/crates/hiverouter-provider/src/client.rs:279-313` (only `account_get`); `ChatMessage.content` is `Option<String>`, so no image/file block: `dto.rs:86-106`; local Data surface is the workspace jail: `web/src/panels/DataPanel.tsx:1-5`, `:12-22` | **L for this product** — Files API stores **images only** (JPEG/PNG/GIF/WebP), and HiveMind is text-only; wiring it would send local files to DeepSeek for no product gain and add privacy/cost/lifecycle obligations. |
| 21 | Search API | `POST https://api.deepseek.com/v1/tools/search`; `query`, `count` (1–50), `need_content`, `ori_question`, `localization`; $0.75 per 1,000 successful requests; QPS 100 | https://api-docs.deepseek.com/guides/search | **NOT WIRED** | No method/route: `client.rs:279-313`; no tool id: `backend/crates/hiverouter-tools/src/tool.rs:108-127`; no network executor: `backend/crates/hiverouter-tools/src/executor.rs:58-79`; token-only cost model: `accounting.rs:196-293` | **M** — adding it needs a `Network` tool (class exists, unused: `tool.rs:30-39`), approval, bounded result, and a **per-request** cost dimension that the current `PriceRow`/`RunAccounting` cannot represent. |
| 22 | Fetch API | `POST https://api.deepseek.com/v1/tools/fetch`; `url` (HTTP/S, host, no user/password, ≤8192 chars); $0.75 per 1,000 successful requests; QPS 25, per-domain limit | https://api-docs.deepseek.com/guides/search | **NOT WIRED** | Same as row 21; no URL policy or network tool | **M** — same cost/approval gap plus SSRF/URL policy, 403 security-policy refusals, 404 no-content handling; must not be exposed to model output blindly. |
| 23 | FIM Completion (Beta) | `POST https://api.deepseek.com/beta/completions`; `prompt`, optional `suffix`, `max_tokens`; max 4K; non-thinking only; `text_completion` response | https://api-docs.deepseek.com/guides/fim_completion ; https://api-docs.deepseek.com/api/create-completion | **NOT WIRED** | No `/beta` base or `/completions` method: `client.rs:321-327`; Chat-only request DTO: `dto.rs:394-422` | **L** — a code-completion product surface, not an agent-chat surface; beta base URL, separate response type, 4K cap. |
| 24 | Chat Prefix Completion (Beta) | `POST https://api.deepseek.com/beta/chat/completions`; last message `role:assistant` + `prefix: true`; often `stop` | https://api-docs.deepseek.com/guides/chat_prefix_completion | **NOT WIRED** | `ChatMessage` has no `prefix` field: `dto.rs:86-106`; base URL is non-beta: `config.rs:16`; `ChatRequest` has no `stop`: `dto.rs:394-422` | **L** — could force the planner's JSON prefix, but JSON mode + the "json" instruction already covers it; adopting the beta base URL would fork the transport for marginal benefit. |

### D. Other documented surfaces

| # | Capability | Documented endpoint / field | Source URL | Status | Exact seam (`file:line`) | Risk |
|---|---|---|---|---|---|---|
| 25 | Vision / image input | Image content parts (base64 or `file_id`); `deepseek-flash` only; v4-pro does not process images | https://api-docs.deepseek.com/guides/files_api ; https://api-docs.deepseek.com/quick_start/pricing/ | **NOT WIRED** | `ChatMessage.content` is `Option<String>`: `dto.rs:86-106`; no image surface in the UI, Data is a local text jail: `DataPanel.tsx:12-22` | **M if the product adds images** — would need content blocks, image limits (32 MiB inline / 64 MiB file), model gating, and a UI. Not applicable to the current text-only HMAS. |
| 26 | Server-side `web_search` tool | `tools:[{type:"web_search"}]` in **Responses** or Anthropic Messages; `deepseek-v4-pro` only; `usage.server_tool_use` | https://api-docs.deepseek.com/guides/responses_api ; https://api-docs.deepseek.com/guides/anthropic_api | **NOT WIRED** | Only `ToolType::Function` exists: `dto.rs:307-313`; tools are client-side only: `tool_loop.rs:200-219` | **M** — this is the only way to get server-run search without a client tool loop, but it is not available on Chat Completions; it requires switching protocol, not adding a field. |
| 27 | `user_id` / KV-cache isolation | Chat `user_id` (≤512 UTF-8 bytes); also Responses `user` and Anthropic `metadata.user_id` | https://api-docs.deepseek.com/api/create-chat-completion/ | **NOT WIRED** | `ChatRequest` has no `user_id`: `dto.rs:394-422` | **L** — HiveMind is single-user/local; adding it would improve cache/scheduling isolation only if the product becomes multi-tenant. |
| 28 | Sampling / penalty / logprob / stop knobs | `temperature`, `top_p`, `frequency_penalty`, `presence_penalty`, `logprobs`, `top_logprobs`, `stop` | https://api-docs.deepseek.com/api/create-chat-completion/ | **NOT APPLICABLE** | Deliberately omitted: `dto.rs:9-11` | **L** — docs say temperature/penalties have no effect in thinking mode; penalties are deprecated; `top_p` behavior is model/mode-dependent. Sending them would add noise, not capability. |
| 29 | `stream_options.include_usage` | `stream_options.include_usage` (all chunks carry `usage`, null except the last) | https://api-docs.deepseek.com/api/create-chat-completion/ | **NOT APPLICABLE** | `ChatRequest` has no `stream_options`: `dto.rs:394-422` | **L** — docs guarantee the last content chunk carries usage either way; HiveMind already reads it. Not needed. |

### E. Cross-cutting enforcement

| # | Capability | Documented endpoint / field | Source URL | Status | Exact seam (`file:line`) | Risk |
|---|---|---|---|---|---|---|
| 30 | Context-window / token preflight | `context_window` from `/models`; 1M context; `max_tokens` is bounded by the context | https://api-docs.deepseek.com/api/list-models ; https://api-docs.deepseek.com/quick_start/pricing/ | **NOT WIRED** | `CallEstimate.prompt_tokens` is always `None` for live calls: `provider_executor.rs:78-81`, `:114-117`; `total_tokens()` returns `None` unless both sides known: `budget.rs:94-103`; token check is skipped: `budget.rs:269-280` | **H** — the 400K combined-token run budget cannot fire on any call made by the production adapter; a prompt larger than the context window is sent and rejected (400) after the request; there is no tokenizer or conservative bound. |
| 31 | Peak / off-peak pricing | Peak 01:00–04:00 and 06:00–10:00 UTC, Mon–Fri, **excluding Chinese public holidays**; off-peak is half rate | https://api-docs.deepseek.com/quick_start/pricing/ | **PARTIAL** | `backend/crates/hiverouter-provider/src/pricing.rs:18-25`, `:177-197`; costs use `peak_at_unix`: `accounting.rs:260-265` | **M** — weekends are handled; Chinese public holidays are not. On those days the local cost report overstates spend (conservative, never understates). No holiday calendar is in the repo. |
| 32 | Concurrency / rate limiting | Account concurrency 2,500 (flash) / 500 (v4-pro); over-limit → 429 | https://api-docs.deepseek.com/quick_start/pricing/ ; https://api-docs.deepseek.com/quick_start/error_codes | **PARTIAL** | Local limiter 8: `budget.rs:43`; retry on 429: `client.rs:492-526`, `:404-441` | **L** — the local cap is far below account limits and is per process; multiple HiveMind/server processes do not coordinate, but 429 is retried with `Retry-After` capped at 60 s. |

---

## Five findings that change what to do next

### 1. The tool loop is dead on the first turn (critical)

`ProviderExecutorAdapter::execute` builds the first leaf `ChatRequest` with only `model`, `messages`, `thinking`, and `max_tokens`:

- `backend/crates/hiverouter-core/src/provider_executor.rs:296-304`

Tools are attached only in `continue_with_tools`:

- `backend/crates/hiverouter-core/src/provider_executor.rs:378-384`

The engine calls `continue_with_tools` only after a tool-call response:

- `backend/crates/hiverouter-core/src/execution.rs:2814` (`if final_reply.tool_calls.is_empty() { break; }`)
- `:2922` (the continuation call site)

A compliant DeepSeek model will not invent `tool_calls` when `tools` is absent; the Chat Completions reference says `tool_choice` defaults to `none` when no tools are present. Therefore the W12 tool pipeline (planning → call → result → continuation) is unreachable in production except when a server/test violates the documented contract.

Why the tests do not catch it: the tool-loop matrix uses a wiremock that returns scripted tool-call SSE for every streamed request, regardless of the request body (`backend/crates/hiverouter-core/tests/tool_loop_matrix.rs:106-124`), and the assertions check request counts, not whether the first request carried `tools` (`:280-285`). The HTTP double is legitimate under the test-double rules; the missing assertion is the defect.

Minimal correct seam: build the tier's tool specs in `execute` exactly as `continue_with_tools` does (`tool_loop::tool_specs_for_tier(&ToolRegistry::standard(), task_agent_tier(task))`) and add `.with_tools(...)` + `.with_tool_choice(ToolChoice::Auto)` before the first stream. Add a test that parses the first request body and asserts `tools` is present and `tool_choice` is `auto`. Do not use `required` with thinking enabled (docs: 400).

### 2. The token budget is decorative and context overflow is unchecked

Every live call sets `prompt_tokens: None`:

- `backend/crates/hiverouter-core/src/provider_executor.rs:78-81` (non-streaming)
- `:114-117` (streaming)

`CallEstimate::total_tokens()` returns `None` unless both prompt and output are known:

- `backend/crates/hiverouter-provider/src/budget.rs:94-103`

`preflight` then skips the combined-token check entirely:

- `backend/crates/hiverouter-provider/src/budget.rs:269-280`

So the 400,000-token run budget (`budget.rs:28`) can never fire on a live call; only the call cap and the $0.50 cost cap do, and the cost cap is checked against spend already measured. There is also no context-window check against the 1M limit or the `context_window` value returned by `/models`.

This is a truthfulness problem, not only a cost problem: the product claims a pre-send budget, but the prompt side is never estimated. A safe fix needs either a documented token-estimation method or a conservative bound with an explicit margin; do not fabricate exact prompt token counts.

### 3. Non-`length` finish reasons can be reported as success

Streaming checks exactly one value:

- `backend/crates/hiverouter-core/src/provider_executor.rs:222-229` (`finish_reason == "length"`)

The documented `finish_reason` values include `stop`, `length`, `content_filter`, `tool_calls`, `insufficient_system_resource`, and `aborted` (`api/create-chat-completion`). With any text present, `content_filter`, `aborted`, or `insufficient_system_resource` falls through to `StreamedReply` and can be persisted as a completed assistant message.

The non-streaming planner has the same gap from the other direction: `message_text` returns any non-empty `content` and never inspects `finish_reason`:

- `backend/crates/hiverouter-core/src/provider_executor.rs:408-434`

Minimal fix: treat every documented non-success reason as a named incomplete/refused outcome (`E_PROVIDER_INCOMPLETE` is the existing code) before persisting; in `message_text`, check `finish_reason` before returning content. This is exactly the "never report a partial reply as success" rule the code already applies to `[DONE]`.

### 4. The long tail is NOT WIRED, and it is not a drop-in

Targeted source search over `backend/` and `web/src/` for `/files`, `tools/search`, `tools/fetch`, `/responses`, `/anthropic`, `api.deepseek.com/beta`, `file_id`, and `input_image` returns no provider/DTO/tool/route hits. The only near-matches are unrelated: `"strict"` refusal-policy strings in persona tests and `profile_id` route parameters.

That is the correct current state, not an accident to "fix" by adding endpoints:

- **Files API is image storage.** The guide says JPEG/PNG/GIF/WebP, `purpose=user_data`, 64 MiB per file, 25 GiB per user. HiveMind's Data surface is a local workspace jail (`web/src/panels/DataPanel.tsx:1-5`, `:12-22`), and `ChatMessage.content` is a string (`dto.rs:86-106`). Wiring Files would add a second data domain, not extend the existing one.
- **Search/Fetch are per-request billed** at $0.75 per 1,000 successful requests, QPS 100/25. The current money path is token-only (`accounting.rs:196-293`; `PriceRow` has hit/miss/output token prices only, `pricing.rs:49-64`). A search call cannot be costed honestly without a new per-request dimension in `RunAccounting`/`RunBudget`.
- **Responses and Anthropic are different protocols.** They stream named events and end with `response.completed` / `message_stop`, not `data: [DONE]`; their usage fields are `input_tokens`/`output_tokens` (Responses) and `input_tokens`/`cache_read_input_tokens` (Anthropic). The current parser (`client.rs:164-192`) and usage DTO (`dto.rs:578-628`) cannot be reused without translation. Adopting either is justified only by external-client interop (Codex/Claude Code), not by HiveMind's own loop.
- **FIM and Chat Prefix are Beta surfaces** on `https://api.deepseek.com/beta`: FIM is `POST /beta/completions`, max 4K, non-thinking; prefix needs `prefix: true` on the last assistant message (`ChatMessage` has no such field). Neither is an agent-chat capability.

The correct integration seam for any of these is a new, explicit surface — not a conditional branch inside the one `DeepSeekClient` that `lib.rs:3-7` declares canonical.

### 5. The wired accounting is strong; three operational edges can still misstate or fail

The Decimal cost path is genuinely careful: no `f64` (`lib.rs:25`), cache hit/miss required or INCOMPLETE (`accounting.rs:233-258`), unsourced price rows refused (`pricing.rs:128-147`), NULL usage never zero (`accounting.rs:205-217`). The remaining edges:

- **Chinese public holidays are not in the peak calculator.** Docs exclude them from peak; `pricing.rs:177-197` checks weekday + UTC hour only. Cost is overstated on those days. This is conservative but still a truthfulness gap in a money report.
- **500/503 are not retried.** `client.rs:492-526` maps every other non-2xx status to `ProviderInvalidResponse`, which is non-retryable (`refusal.rs:266-268`), while the official error table says retry 500/503 after a brief wait. A transient server error fails the run.
- **`/models` metadata is dropped.** `dto.rs:890-909` parses only `id`, `object`, `owned_by`, `name`, `context_window`, `max_output_tokens`; the documented `effort.supported_levels` and `api_capabilities` are ignored, so `HIVEROUTER_REASONING_EFFORT` cannot be validated against the model and the UI cannot show protocol capability. The UI also lists every provider model as "available" (`ModelsPanel.tsx:194-206`) although the client accepts only two IDs.

These are fixable without a product redesign; they should be fixed before any claim of "full use".

---

## Correct integration seams, if a capability is adopted

| Capability | Do not reuse | Correct seam to extend | What "done" must include |
|---|---|---|---|
| First-turn tools | `continue_with_tools` as the only tool-advertising path | `provider_executor.rs:296-304` | `tools` + `tool_choice:auto` on the first request; request-body assertion in tests; reasoning pass-back once tools are present. |
| Strict tools | Chat `FunctionDefinition` as-is | `dto.rs:361-392` add `strict`; `client.rs:321-327` needs a beta base URL override | Beta base URL, schema-subset validation, refusal path for server schema rejection. |
| Context/token preflight | `CallEstimate::default()` / `prompt_tokens: None` | `provider_executor.rs:78-81`, `:114-117`; `budget.rs:87-103`, `:235-280` | A documented estimate or conservative bound, a margin, and a test proving a pre-send refusal sends zero HTTP requests. |
| Finish reasons | `== "length"` | `provider_executor.rs:222-229`, `:408-434` | All documented values mapped; `content_filter`/`aborted`/`insufficient_system_resource` never persisted as success. |
| 5xx retry | `classify_status` blanket non-2xx | `client.rs:492-526`; `refusal.rs:254-268` | Retry 500/503 with jitter and the existing attempt cap; 400/422 mapped to a schema refusal, not "invalid response". |
| Holiday pricing | `weekday_monday_zero` only | `pricing.rs:177-197` + an official holiday calendar with provenance | A dated, sourced holiday table; a test for a known holiday; no silent default. |
| Model metadata | Hardcoded `ReasoningEffort` variants | `dto.rs:890-909`; `config.rs:146-156`; `ModelsPanel.tsx:194-206` | Parse `effort.supported_levels`; refuse an unsupported configured effort before boot; mark unsupported catalogue IDs as not selectable. |
| Search / Fetch | `account_get` pattern alone | `client.rs:279-313` for POST JSON; `tool.rs:108-127` for `ToolId`; `tool.rs:302-434` for contracts/allowlists; `executor.rs:58-79` for execution; `accounting.rs:196-293` for cost | New `Network` tool with approval, URL policy, bounded results, and a **per-request** cost dimension; do not hide it inside token pricing. |
| Files / vision | `ChatMessage.content: Option<String>` | `dto.rs:86-106`; `client.rs:279-313`; `DataPanel.tsx:12-22` if a UI is needed | Content-part model, image limits, `deepseek-flash` gating, upload lifecycle, privacy decision. |
| Responses / Anthropic | `parse_sse_line` / `ChatResponse` | New transport module; `client.rs:164-192` only as a counter-example | Named-event parser, terminal-event handling, protocol-specific usage mapping, separate tests. Do not put a second protocol inside the one client. |
| FIM / Prefix | `ChatRequest` / chat endpoint | New methods; `client.rs:321-327`; `dto.rs:394-422` | Beta base URL, separate response/stream handling, `prefix` field and `stop` support, and a product reason. |

---

## What "full use" should honestly mean for this local HMAS

"Full use" is not "every endpoint exists in the client." For a local hierarchical multi-agent product with one key, a $0.50 run cap, truthful usage accounting, and a jailed tool surface, it should mean:

**Bar 1 — the product's dependency set is complete and exercised.** That set is: Chat Completions, model selection, thinking/effort, function tools, JSON output, streaming, usage/cache counters, models, balance. Today, eight of those nine are wired; tools are not usable end-to-end because of finding 1. Until that is fixed, the honest claim is **"chat, thinking, JSON, streaming, usage/cache/cost, models and balance are integrated; tools are partially integrated."**

**Bar 2 — every excluded capability is excluded by a named product decision, not by silence.** The exclusion list should live in this file and in `docs/provider/DEEPSEEK-INTEGRATION.md`: Files (image storage, text-only product), Search/Fetch (needs an agent-tool policy and per-request billing), Responses/Anthropic (external-client interop only), FIM/prefix (different product surface), strict tools (beta; consider only if tool-schema violations become a measured problem), vision (no image surface), `user_id` (single-user local), sampling knobs (no effect/deprecated).

**Bar 3 — no surface may claim more than the code proves.** Specifically: do not say "tools are wired" while `execute` omits them; do not say "budget is enforced" while the token half is skipped; do not say "all DeepSeek APIs are used" when seven documented surfaces are not; do not show a provider model as available when `Model::from_id` would refuse it.

A defensible "full use" statement after the Bar-1 and Bar-3 fixes would be:

> HiveMind uses DeepSeek's Chat Completions path for all model calls, with explicit thinking/effort control, JSON-mode planning, streaming replies, function-tool continuation, and measured usage/cache/cost accounting. It reads the account's model catalogue and balance. It deliberately does not use the Files, Search/Fetch, Responses, Anthropic Messages, FIM, Chat Prefix, vision, or strict-tool Beta surfaces; each exclusion is recorded with its reason and the seam it would require.

That is more useful than a client that has a method for every endpoint but cannot prove which ones its agent loop actually exercises.

**What not to do:** do not add Files/Search/Responses/Anthropic/FIM "for completeness." Each adds a second protocol, a second cost model, a second failure surface, or a data-egress decision. The one-client/one-accounting invariant in `backend/crates/hiverouter-provider/src/lib.rs:3-7` exists for a reason; bypassing it to tick boxes would reduce, not increase, real capability.

---

## Confirmed vs unknown (dated)

### Confirmed by official docs + local code, 2026-11-07

| # | Claim | Evidence | Date |
|---|---|---|---|
| C1 | OpenAI base URL is `https://api.deepseek.com`; chat is `POST /chat/completions`; no `/v1` prefix | docs home; `client.rs:321-327` | 2026-11-07 |
| C2 | Model IDs are `deepseek-flash` (served by DeepSeek-V4.1-Flash-1017) and `deepseek-v4-pro` (DeepSeek-V4-Pro-0813) | docs home; pricing table; `dto.rs:27-56` | 2026-11-07 |
| C3 | Context length is 1M; max output is 384K (393,216); `max_tokens` range 1–393216 | pricing table; chat API reference; `dto.rs:23`, `:546-552` | 2026-11-07 |
| C4 | Default `max_tokens` is 8K non-thinking, 64K thinking, 128K at `reasoning_effort=max` | chat API reference | 2026-11-07 |
| C5 | Thinking defaults to enabled; `thinking.type` takes precedence over `reasoning_effort`; effort default is `high` | thinking guide; chat API reference | 2026-11-07 |
| C6 | Effort values are `none/low/high/max`; `minimal→low`, `medium/xhigh→high`, `ultra→max` | thinking guide; chat API reference; `config.rs:146-156` | 2026-11-07 |
| C7 | With `tools`, prior `reasoning_content` must be passed back or the API returns 400 | thinking guide; `dto.rs:184-194`; `provider_executor.rs:362-366` | 2026-11-07 |
| C8 | Only `function` tools are documented for Chat Completions; `strict` is Beta and requires `/beta` | tool-calls guide; chat API reference | 2026-11-07 |
| C9 | `tool_choice` defaults to `none` when no tools are present; `required`/named are rejected in thinking mode | chat API reference | 2026-11-07 |
| C10 | JSON mode is `response_format:{type:"json_object"}` plus a "json" instruction; invalid JSON can become spaces | JSON guide; chat API reference; `provider_executor.rs:25-28`, `:277` | 2026-11-07 |
| C11 | Streaming is data-only SSE ending in `data: [DONE]`; usage rides on the last content chunk with a non-null `finish_reason` | chat API reference; `client.rs:164-192`, `:624-723` | 2026-11-07 |
| C12 | `finish_reason` values include `stop`, `length`, `content_filter`, `tool_calls`, `insufficient_system_resource`, `aborted` | chat API reference; `provider_executor.rs:222-229` covers only `length` | 2026-11-07 |
| C13 | Context caching is automatic; counters are `prompt_cache_hit_tokens` / `prompt_cache_miss_tokens` | KV-cache guide; `dto.rs:590-598`; `accounting.rs:233-265` | 2026-11-07 |
| C14 | `GET /models` returns per-model `context_window`, `max_output_tokens`, modalities, effort levels, protocol capabilities | list-models reference; `dto.rs:890-909` parses only 4 fields | 2026-11-07 |
| C15 | `GET /user/balance` returns `is_available` and per-currency decimal-string balances | get-user-balance reference; `dto.rs:911-933` | 2026-11-07 |
| C16 | 500/503 are retryable per the error table; 400 is invalid format; 422 is invalid parameters | error-codes page; `client.rs:492-526` does not retry 5xx | 2026-11-07 |
| C17 | Files API is image-only (JPEG/PNG/GIF/WebP), 64 MiB upload, 25 GiB / 10,000 files per user, `purpose=user_data` | Files guide | 2026-11-07 |
| C18 | Search is `POST /v1/tools/search` (QPS 100); Fetch is `POST /v1/tools/fetch` (QPS 25); both $0.75/1,000 successful requests | Search guide | 2026-11-07 |
| C19 | Responses is `POST /responses`, stateless, semantic SSE, no `[DONE]`, `input_tokens`/`output_tokens` usage | Responses guide + reference | 2026-11-07 |
| C20 | Anthropic Messages is `POST /anthropic/v1/messages`, named SSE ending `message_stop`, no `[DONE]`, `x-api-key` or Bearer | Anthropic guide + reference | 2026-11-07 |
| C21 | FIM is `POST /beta/completions`, max 4K, non-thinking only; Chat Prefix needs `prefix:true` on the last assistant message and `/beta` | FIM guide/reference; Chat Prefix guide; pricing table | 2026-11-07 |
| C22 | Peak windows are 01:00–04:00 and 06:00–10:00 UTC, Mon–Fri, excluding Chinese public holidays; off-peak is half rate; concurrency 2,500/500 | pricing table; `pricing.rs:18-25`, `:177-197` | 2026-11-07 |
| C23 | Local source contains no `/files`, `tools/search`, `tools/fetch`, `/responses`, `/anthropic`, `/beta`, `file_id`, or `input_image` path | targeted source search over `backend/` and `web/src/`; nearest hits are unrelated `"strict"` persona strings and `profile_id` routes | 2026-11-07 |
| C24 | The first leaf request does not carry `tools`; continuation does | `provider_executor.rs:296-304` vs `:378-384`; `execution.rs:2814`, `:2922` | 2026-11-07 |
| C25 | The combined-token preflight is skipped for live calls | `provider_executor.rs:78-81`, `:114-117`; `budget.rs:94-103`, `:269-280` | 2026-11-07 |

### Unknown / not verified in this pass

| # | Unknown | Why it matters | What would prove it |
|---|---|---|---|
| U1 | Live account state (which models the key actually returns, available balance, `is_available`) | The UI claims live account data; this pass made no live call | A keyed `GET /models` and `GET /user/balance` with the response recorded (no key value in the record). |
| U2 | Whether the live `/models` payload actually contains `effort.supported_levels` and `api_capabilities` for both models | Docs describe the schema; fields "may be omitted" | A keyed `/models` body captured verbatim. |
| U3 | Whether a model emits `tool_calls` when `tools` is absent (should be never per docs) | Finding 1 assumes it does not; a violating response would mask the bug | A live non-tool request with a prompt that would otherwise call a tool; expect plain text. |
| U4 | Actual cache-hit behavior for HiveMind's planner/leaf/tool-continuation prompts | Cost and latency claims depend on prefix reuse | A keyed multi-turn run with `prompt_cache_hit_tokens` / `prompt_cache_miss_tokens` recorded. |
| U5 | Whether `reasoning_effort:"none"` with `thinking:{"type":"enabled"}` behaves as docs imply (thinking wins, effort none) | Local config can produce this combination | A keyed A/B run with both fields; record whether `reasoning_content` appears. |
| U6 | Exact `finish_reason` values seen in production streams (all six? only stop/length?) | Finding 3 is a schema-completeness argument, not an observed failure | A keyed long-output / filtered / cancelled run with the final chunk captured. |
| U7 | Chinese public-holiday dates and whether the API applies the exemption per billing | Row 31 overstates on holidays today | Official holiday calendar + a keyed billing/usage test on a holiday; or a docs statement of the calendar. |
| U8 | Whether 500/503 are common enough to matter, and whether retrying them is safe for non-idempotent calls | Chat completions are stateless/idempotent in this product, but the retry policy is still a choice | Production telemetry over a meaningful window; not available here. |
| U9 | Responses/Anthropic server-side `web_search` billing and behavior under this account | Relevant only if the product adopts those protocols | A keyed Responses/Anthropic run with `server_tool_use` recorded. |
| U10 | Whether `input_modalities` includes `image` for `deepseek-flash` on this account | Vision is out of scope now, but the model catalogue is the source of truth | A keyed `/models` body (same as U2). |

---

## Official sources fetched in this pass

All fetched **2026-11-07**; all returned HTTP 200.

| Source | Used for |
|---|---|
| https://api-docs.deepseek.com/ | base URLs, model IDs, auth, legacy-name routing |
| https://api-docs.deepseek.com/api/list-models | `/models` schema and metadata fields |
| https://api-docs.deepseek.com/api/get-user-balance | `/user/balance` schema |
| https://api-docs.deepseek.com/api/create-chat-completion | full Chat Completions request/response, `max_tokens`, `thinking`, `reasoning_effort`, `tools`, `tool_choice`, `response_format`, `stream`, `stream_options`, usage, `finish_reason`, `user_id` |
| https://api-docs.deepseek.com/guides/thinking_mode | toggle/effort semantics, aliases, `reasoning_content` pass-back |
| https://api-docs.deepseek.com/guides/tool_calls | function tools, thinking-mode tools, `strict` Beta, mid-conversation tool-call restrictions |
| https://api-docs.deepseek.com/guides/json_mode | JSON-mode requirements and blank-content caveat |
| https://api-docs.deepseek.com/guides/files_api | Files endpoints, image-only scope, limits, `file_id`, Anthropic beta header |
| https://api-docs.deepseek.com/guides/search | Search/Fetch endpoints, fields, billing, QPS, errors |
| https://api-docs.deepseek.com/guides/responses_api | Responses endpoint, streaming events, tools, usage, image input, compatibility |
| https://api-docs.deepseek.com/guides/anthropic_api | Anthropic endpoint, model mapping, headers, tools, streaming, usage |
| https://api-docs.deepseek.com/guides/kv_cache | automatic cache, hit/miss counters, prefix rules |
| https://api-docs.deepseek.com/guides/fim_completion | FIM Beta endpoint, 4K, `/beta` base |
| https://api-docs.deepseek.com/guides/chat_prefix_completion | `prefix:true`, `/beta`, `stop` example |
| https://api-docs.deepseek.com/api/create-response | Responses API exact endpoint/schema |
| https://api-docs.deepseek.com/api/create-message | Anthropic Messages exact endpoint/schema |
| https://api-docs.deepseek.com/api/create-completion | FIM exact endpoint/schema |
| https://api-docs.deepseek.com/quick_start/pricing/ | context/output, pricing, peak windows, holiday exclusion, concurrency |
| https://api-docs.deepseek.com/quick_start/error_codes | 400/401/402/422/429/500/503 causes and retry guidance |
| https://api-docs.deepseek.com/updates/ | newest documented change (2026-10-17); docs version pin |

---

## Local sources read (no edits)

- `backend/crates/hiverouter-provider/src/lib.rs`, `dto.rs`, `client.rs`, `secret.rs`, `error.rs`
- `backend/crates/hiverouter-provider/src/budget.rs`, `accounting.rs`, `pricing.rs` (supporting cost/budget rows)
- `backend/crates/hiverouter-core/src/http.rs`, `config.rs`, `provider_executor.rs`, `execution.rs`, `tool_loop.rs`, `tools_bridge.rs`, `ws.rs`
- `backend/crates/hiverouter-tools/src/tool.rs`, `executor.rs`, `commands.rs`, `settings.rs` (tool registry and `SETTINGS_STATUS` truthfulness)
- `backend/crates/hiverouter-contract/src/refusal.rs` (retryability of provider refusals)
- `web/src/panels/ModelsPanel.tsx`, `SettingsPanel.tsx`, `DataPanel.tsx`
- `docs/provider/DEEPSEEK-INTEGRATION.md` (current integration note; its "tools wired" claim is the one this matrix corrects)

**Secret hygiene:** no API key, token, Authorization header value, or account identifier is printed in this document. The only credential name mentioned is the environment variable `DEEPSEEK_API_KEY`, which is a name, not a value.

---

## Corrections to the current integration note

`docs/provider/DEEPSEEK-INTEGRATION.md` is accurate on the core data path, but three statements need narrowing:

1. **"Function tools + tool-call assembly + tool loop" (line 53)** — DTOs, assembly, execution and continuation exist, but the first request does not offer tools, so the loop is not usable end-to-end. The honest wording is "tool-call assembly and continuation are implemented; first-turn tool advertisement is missing."
2. **"Models UI ... reports the actual provider/model" (lines 65-66)** — it does: `ws.rs:777-795` overrides the static `SETTINGS_STATUS` payload with `config.provider.model`, planner thinking and effort. That part is correct. What it does not report is the leaf thinking mode or the per-model supported effort levels.
3. **"Chat, thinking, tools, JSON, streaming, caching, usage, models and balance are integrated" (lines 93-95)** — true except tools (partial) and except the fact that the model-metadata and balance-availability legs are partial. The corrected claim is the one in "What full use should honestly mean" above.

No source edit was made to that note; this file supersedes its status claims for the purposes of the capability matrix.
