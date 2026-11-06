# EVIDENCE — Seat S2 (Contract-Security)

Read-only investigation: only `plans/seats/SEAT-02/**` was written. No product/git/config edits, no key read,
no paid call, no rebuild, no server restart. HEAD at start `f4c780d`. All commands are read-only PowerShell.

## 1. Read-order evidence
Full `read`-tool reads in the mandated order are logged in `READLOG.md` with bytes/lines/sha256/timestamp and
one-line verified facts per row (paper #1, four docs #2–5?, audit docs #6–8, product files #9–20). Two official
docs fetches were truncated by the fetch tool after the contract-relevant sections (thinking-mode tool-call
sample; chat-completions streaming response schema) — recorded as PARTIAL, never inferred.

## 2. Key-material grep (class-based, across the assigned slice)
Grep classes: `DEEPSEEK_API_KEY`, `sk-`, `Bearer`, `api_key`, `key_present`, `expose`, `Zeroizing`, `Debug`.
- `refusal.rs:118` — `E_KEY_MISSING` how_to_fix mentions the env var name only (no value); the same for
  `E_PROVIDER_AUTH` (`refusal.rs:256`). No secret literal.
- `settings.rs:22` — `key_present: ApiKey::present_in_env()`; module docs state the value is never read into a
  string. The unit test `status_never_contains_a_key_value` asserts the serialized payload contains no `sk-` or
  `Bearer ` (`settings.rs:47-55`).
- `ws.rs:397-415` — SETTINGS_STATUS merges provider/model/thinking/effort from `state.config`; never reads the
  key. `ws_transport.rs:331-350` asserts `key_present` is boolean and the rendered frame contains no
  `sk-`/`DEEPSEEK_API_KEY`.
- `health.rs` `/diagnose` exposes `provider_key_present: bool` only (`health.rs:60,419-422`); its test asserts
  the full payload contains no `sk-`/`Bearer ` (`health.rs:411-413`).
- `contract.ts:242-248,659-682` — decoder accepts `key_present` boolean only; `thinking`/`reasoning_effort`
  optional. No key-shaped field exists in any TS payload type.
- `http.rs` contains no key read; the only credential it emits is the **session token** from `/api/session`
  (`http.rs:398-425`), which is auth material by design (not the DeepSeek key) and is gated by Origin allowlist
  or loopback peer.
- `ws.rs` logs only refusal codes (`tracing::warn!(skipped…)`, `tracing::error!(code = …)`); no headers/bodies.
- DB: no key column appears in any assigned file; `/health` DB section exposes path + schema version only
  (`http.rs:222-229`). Full DB schema is outside the S2 slice → UNVERIFIED for key persistence (would require
  reading `core/src/db.rs` and migrations).
**Verdict: no API-key material can reach wire/logs/UI/DB through the assigned paths.** The only credential on
the wire is the localhost session token, by design.

## 3. Parity verification (commands + observed values)
Counted from source with `read` (line ranges cited):
- Commands: `CommandName::ALL` = 17 variants, 3 of which are `DIFF_*` decision-gated → **14 frozen**
  (`command.rs:65-83`, `is_frozen` at 121-126; test `wire_names_match_the_frozen_vocabulary` also lists 17).
- Events: `EventName::ALL` = **21** (`event.rs:65-87`, test at 491-521).
- Refusals: refusal_table = **65** variants (`refusal.rs:73-282`; groups: 8 transport/UI + 39 command/schema +
  4 files/jail + 6 tool/safety + 5 provider + 2 build + 1 internal). `from_code` roundtrip test covers all.
- States: `AgentState::ALL` = **16** incl `Interrupted` (`state.rs:55-72`; test asserts 16 at 146).
- UI: `FROZEN_COMMAND_NAMES` = 14 (`contract.ts:102-117`); `EVENT_NAMES` = 21 (`contract.ts:294-316`);
  `KNOWN_REFUSAL_CODES` = **45** (`contract.ts:332-378`) → **20 contract codes missing from the UI table**.
- Paper claim (14/21/65/16) reproduced exactly from current source. UI state-list parity (`agentState.ts`) is
  outside the S2 slice → UNVERIFIED in this seat; paper A-F03 claims 16 vs 15 and that file was not assigned.

## 4. Truthfulness probes
- SETTINGS_STATUS: tools payload (`settings.rs:18-26`) → WS merge of provider/model/thinking/effort
  (`ws.rs:397-415`) → `contract.ts:242-248,659-682` decode → test asserts provider `deepseek`, model
  `deepseek-flash`, boolean key presence, no secret strings (`ws_transport.rs:331-350`). **Truthful: it reports
  key presence only and never the key.**
- HISTORY_SNAPSHOT: payload `{run_id, revision, graph}` (`event.rs:315-324`) → emitted only from
  `history.state_at(run_id, revision)` with ACK first (`ws.rs:417-437`) → UI decodes the same three fields
  (`contract.ts:733-739`). **Truthful shape and source**; whether the read model returns the claimed revision is
  owned by b-core (outside slice) → UNVERIFIED here.
- No fake-success path in the slice: refusals are typed events (`ErrorPayload::from_refusal`, `event.rs:149-160`;
  `ws.rs:496-503`), invalid frames keep the connection usable (`ws_transport.rs:142-165`).

## 5. Findings evidence (path:line)
| ID | Severity | Evidence |
|---|---|---|
| F-S2-01 UI refusal-table subset (45/65) | MEDIUM | `web/src/api/contract.ts:332-378` vs `backend/crates/hiverouter-contract/src/refusal.rs:73-282`; missing: E_AGENT_LIMIT, E_SEMAPHORE_FULL, E_TOKEN_BUDGET_EXCEEDED, E_CALL_LIMIT, E_USAGE_INCOMPLETE, E_PRICE_UNSOURCED, E_PEAK_UNDEFINED, E_COST_MATH_INVALID, E_MULTIKEY_FORBIDDEN, E_LLM_RATE_LIMIT, E_APPROVAL_REQUIRED, E_DECISION_REQUIRED, E_DECISION_REQUIRED-adjacent DIFF code present? (E_DIFF_DECISION_REQUIRED present), E_TEMPLATE_INVALID, E_PERSONA_UNKNOWN, E_SNAPSHOT_CORRUPT, E_FILE_TREE_UNAVAILABLE, E_TOOL_NOT_ALLOWED, E_PROVIDER_INVALID_RESPONSE, E_INTERNAL_ERROR + E_SCHEMA...; exact set recomputed at write time = 20 missing |
| F-S2-02 NODE_REWIRE hardcodes running state | MEDIUM | `core/src/ws.rs:344-352` builds `AgentStateForEvent::running()`; `AgentStateForEvent` at 483-490; a rewire of a queued/terminal task would broadcast `running` |
| F-S2-03 "per-session" token is per-server boot token | LOW | `http.rs:89-92` creates one token per AppState; all clients share it; `ws.rs:35-56` docs say per-session |
| F-S2-04 non-constant-time token compare | LOW | `core/src/ws.rs:113` `query.token.as_deref() != Some(state.session.token())` |
| F-S2-05 `/health` leaks absolute DB path | LOW | `core/src/http.rs:222-229,349-362` (`path: state.db.path().display()`) |
| F-S2-06 FILE_TREE decoder defaults `kind` to file | LOW | `web/src/api/contract.ts:696-705` (both `kind` and `is_dir` absent → `'file'` silently) |
| F-S2-07 resync without snapshot; stale comment | INFO | `core/src/ws.rs:269-292` sends RESYNC_REQUIRED + SERVER_HELLO and a comment says the snapshot API "lands in Phase 9" though history API exists (`http.rs:249-251,293-347`) |
| F-S2-08 paper A-F01 stale at HEAD | INFO | paper line 104 says core constructs `UnconfiguredExecutor`; current `http.rs:25,67-82` wires `ProviderExecutorAdapter::from_env` with honest fallback → live-model claim still UNVERIFIED but the "never wired" P0 is stale at f4c780d |
| F-S2-09 ERROR `code` is a free String on the wire | LOW | `event.rs:137-147`; UI decoder accepts any string (`contract.ts:507-519`) — an unknown/forged code renders without a canonical mapping |
| F-S2-10 SETTINGS_STATUS carries undocumented fields | INFO | `event.rs:274-279` (thinking/reasoning_effort) additive; contract.ts tolerates (`contract.ts:666-679`); frozen doc §3 lists only provider/model/key_present |

## 6. Tests/commands not run (read-only mandate)
`cargo test`/`cargo build` were **not** run (no rebuilds allowed). Test claims above are source-read (tests
exist and assert the behaviours); their last observed green run belongs to the program's test evidence, not to
this seat. UNVERIFIED here: UI state parity (`web/src/state/agentState.ts`, not assigned), history read-model
correctness, DB schema key-freedom, live provider run. What would prove them: read those files; run
`cargo test -p hiverouter-core --test ws_transport` and the frontend contract/state tests.

## 7. Files read / unreadable
Read: 3 audit docs + 4 official docs pages + 12 product files (all assigned; full-file). Unreadable: none.
Truncated external fetches: 2 (documented). Files not assigned but referenced for cross-check: `core/src/db.rs`,
`experiment`/history files, `web/src/state/agentState.ts` — recorded UNVERIFIED rather than read (scope fence).
