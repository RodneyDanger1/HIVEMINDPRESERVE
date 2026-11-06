# HiveMind 1.0 — release notes

## What this release is

HiveMind 1.0 is the first official build of the localhost HiveMind runtime: one Rust server binary + embedded React UI, one CLI, and a real DeepSeek provider path. It is the first build in which `SUBMIT_PROMPT` reaches the provider and returns a model-authored plan and reply.

## Verified in the build

- Live key validation: HTTP 200 from `deepseek-flash` (DeepSeek-V4.1-Flash-1017), content `HIVE-KEY-OK`, usage 20 tokens. See `docs/evidence/key-validation.md`.
- Live server path: `SUBMIT_PROMPT` → real planner DAG → real leaf reply → `REPLY_DONE` usage → run `completed`; terse prompt `HIVE-LIVE-OK`, usage 95 tokens. See `docs/evidence/live-provider-smoke.md`.
- Provider wiring: `hiverouter-provider` is a dependency of `hiverouter-core`; `http.rs` builds the adapter when the key is present and falls back to the honest `E_KEY_MISSING` executor when it is absent or provider is disabled.
- Key custody: environment-only (`DEEPSEEK_API_KEY`); never stored in the repository, database, UI, logs, or evidence files; zeroized/redacted provider type; **strict credential-pattern scan (`sk-[A-Za-z0-9]{32,}`) = 0 hits** over the tracked non-vendor tree (literal `sk-` substrings occur only in obvious unit-test fixtures and embedded binary strings).
- Reasoning controls: `HIVEROUTER_MODEL`, `HIVEROUTER_THINKING`, `HIVEROUTER_LEAF_THINKING`, `HIVEROUTER_REASONING_EFFORT`, `HIVEROUTER_MAX_OUTPUT_TOKENS`, `HIVEROUTER_PLANNER_MAX_OUTPUT_TOKENS`, `HIVEROUTER_PROVIDER` (auto/disabled). Settings shows model, thinking, effort and key presence read-only.
- Budget: per-run `RunBudget` ($0.50 cap, 40 calls, 8 in-flight); usage absent is INCOMPLETE, never 0; 401/402 not retried.
- Replay backend: `HISTORY_REVISION_SET` now returns a read-only `HISTORY_SNAPSHOT`; the web canvas projects snapshot nodes/edges and never mutates live state.
- State parity: backend 16 states and web 16 states, including `interrupted`.
- Tests: full Rust workspace green after the change; web 46 tests / typecheck / production build green. Exact final counts/hashes are in `docs/RELEASE-EVIDENCE.md` at build time.

## Known limitations / not claimed

- The provider adapter returns a leaf reply as one `REPLY_DELTA` after a non-streaming completion; true SSE chunk-by-chunk rendering is not claimed in 1.0 (tracked for 1.1).
- Autonomous model tool-calling is not part of 1.0; tools/approval/jail remain available to the application surfaces, but the model does not yet drive multi-turn tool calls.
- Full trace ladder, pixel-level design fidelity, Tauri/installer, and every P2 audit item remain out of scope.
- The key was supplied in a chat transcript; rotate it after this release if the transcript is shared. No key material is in the repository or release artifacts.

## Install / run

1. Set `DEEPSEEK_API_KEY` in the environment of the server process (never in a committed file, never in the UI).
2. Run `hiverouter.exe`; it binds loopback only (`127.0.0.1:8080` by default).
3. Open the printed URL. `/health` and the Settings panel report key presence (boolean only).
4. Use `hiverouter-cli health/diagnose` for status.

## Release gate

The annotated tag is created only after the independent audit and final packaging smoke pass on a quiescent tree. Moving the previous private `v1.0.0` tag requires owner confirmation and records the superseded commit.
