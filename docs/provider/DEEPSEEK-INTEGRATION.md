# DeepSeek API integration — HiveMind

Research date: 2026-11-07. Official docs fetched in-session:
- [Your First API Call](https://api-docs.deepseek.com/)
- [Lists Models](https://api-docs.deepseek.com/api/list-models)
- [Get User Balance](https://api-docs.deepseek.com/api/get-user-balance)
- [Thinking Mode](https://api-docs.deepseek.com/guides/thinking_mode)
- [Tool Calls](https://api-docs.deepseek.com/guides/tool_calls)

## 1. Confirmed API facts

| Fact | Verified value |
|---|---|
| Base URL (OpenAI-compatible chat) | `https://api.deepseek.com` |
| Chat endpoint | `POST /chat/completions`; no documented `/v1` prefix |
| Auth | `Authorization: Bearer <DEEPSEEK_API_KEY>` |
| Model IDs | `deepseek-flash` (served by DeepSeek-V4.1-Flash-1017), `deepseek-v4-pro` |
| Flash context / max output | **1,048,576 / 393,216** tokens (live `/models` response) |
| Thinking toggle | `thinking: {"type": "enabled"|"disabled"}` |
| Thinking effort | `reasoning_effort: low|high|max` (default high) |
| Reasoning output | `reasoning_content`; must be passed back when `tools` are used |
| Tool calls | Chat Completions function tools; thinking-mode tool loops supported |
| JSON mode | `response_format: {"type":"json_object"}` |
| Streaming | SSE `data:` frames, `[DONE]` terminator, usage in final chunk |
| Cache counters | `prompt_cache_hit_tokens`, `prompt_cache_miss_tokens`, `cached_tokens` |
| Account models | `GET /models` |
| Account balance | `GET /user/balance` |
| Files API | upload/list/retrieve/delete available by docs; not yet used by HiveMind |
| Search & Fetch | documented endpoints; not yet used by HiveMind |
| Responses / Anthropic APIs | documented; HiveMind uses the OpenAI-compatible Chat Completions path |

## 2. Key handling

- Stored **encrypted with Windows DPAPI (CurrentUser)** at
  `%LOCALAPPDATA%\HiveMind\provider.key`.
- Created/rotated with `tools/set-provider-key.ps1`; the prompt is hidden and the key is not echoed.
- `HiveMind.exe` decrypts the file and injects `DEEPSEEK_API_KEY` into the
  **`hiverouter.exe` child process only**.
- The key is never in source, tracked files, command lines, browser JavaScript,
  HTTP responses, or server logs.
- `hiverouter` reads it through the existing `ApiKey` type: zeroized memory,
  redacted `Debug`, exposed only to build the `Authorization` header.
- User-level `DEEPSEEK_API_KEY` was cleared after the encrypted file was verified.

## 3. What HiveMind already used before this pass

The provider crate already implemented:

- `deepseek-flash` / `deepseek-v4-pro` model selection.
- Streaming and non-streaming Chat Completions.
- Planner thinking mode + `reasoning_effort`, leaf thinking control.
- JSON-mode planner output.
- Function tools + tool-call assembly + tool loop.
- Retry/backoff, no retry on 401/402, rate-limit handling.
- Cache hit/miss accounting and cost accounting.
- Usage counters and incomplete-usage refusal.
- Run budget and in-flight limiter.

## 4. Added in this pass

- `GET /models` and `GET /user/balance` methods on the provider client.
- Local server proxies:
  - `GET /api/provider/models`
  - `GET /api/provider/balance`
- Models UI now shows live balance and available models.
- Models UI auto-runs `SETTINGS_STATUS` so it reports the actual provider/model instead of a stale warning.
- Launcher DPAPI key loading.
- `tools/set-provider-key.ps1` for create/rotate.
- Portable build includes no key material.

## 5. Live verification performed

| Check | Result |
|---|---|
| `/models` with the supplied key | 200; `deepseek-flash`, `deepseek-v4-pro` |
| `/user/balance` with the supplied key | 200; available; USD balance returned |
| Minimal non-streaming chat | 200; `content = HIVE_OK`; usage 11/3/14 |
| Local `/health` | `ok`; `deepseek_api_key` true |
| `/api/provider/models` through localhost | both models returned |
| `/api/provider/balance` through localhost | USD balance returned |
| UI Models surface | shows provider account, balance, both model limits |
| UI prompt through HiveMind | completed with assistant message `HIVE_UI_OK`; no key refusal |

## 6. What is deliberately not claimed

- **Files API** is not wired into HiveMind's file tools. HiveMind's Data surface is a local
  workspace jail, not DeepSeek-hosted files.
- **Search & Fetch** is not exposed as an agent tool. Adding it requires a policy/tool
  design, cost controls, and refusal handling; it is not silently faked.
- **Responses API / Anthropic Messages API** are alternative protocols for the same models;
  HiveMind's frozen path is OpenAI-compatible Chat Completions.
- **FIM / chat-prefix completion** is a different product surface; not used by HiveMind.
- No claim of "full use of every DeepSeek endpoint." The correct claim is:
  **Chat, thinking, tools, JSON, streaming, caching, usage, models and balance are integrated;
  Files, Search/Fetch, Responses, Anthropic, and FIM are documented gaps.**
