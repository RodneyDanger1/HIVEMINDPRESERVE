# Key validation evidence (no key material)

- Date: 2026-11-06 (America/Chicago)
- Custody: key set only in the process/user environment as `DEEPSEEK_API_KEY`; never written to the repository, logs, UI, database, or this file.
- Endpoint: `POST https://api.deepseek.com/chat/completions`
- Model: `deepseek-flash` (served by DeepSeek-V4.1-Flash-1017)
- Validation request: one user message, `thinking.type=disabled`, `max_tokens=16`, `stream=false`
- Result: **HTTP 200**
  - response id: `8d5af788-1777-4c0d-b52e-56bdfbfdef25`
  - content: `HIVE-KEY-OK`
  - finish_reason: `stop`
  - usage: prompt 14, completion 6, total 20 (cache hit 0, miss 14)
- Env presence after setup: process `True`; user scope `True`.
- Docs read before the call: https://api-docs.deepseek.com/ , https://api-docs.deepseek.com/quick_start/pricing/ , https://api-docs.deepseek.com/guides/thinking_mode/ , https://api-docs.deepseek.com/api/create-chat-completion/
- No key substring appears in this evidence file; the raw request header is intentionally absent.
