# REPORT — Seat S2 (Contract-Security), HiveMind 1.0

HEAD `f4c780d` · read-only · 2026-11-06 · deliverable paths: `plans/seats/SEAT-02/`.
Scope: `HIVEMIND-AUDIT-PAPER.md` (full), four official DeepSeek docs (full/partial per READLOG), audit
WAVE-SHEET + 2 diagrams, and the assigned contract/core/tools/web product slice (12 files).
Skills loaded: forgewright-recon/plan/lead, integration-logic, research, auditor, logic-debugger, trace,
weaver, visuals, agent-experience, capability-auditor, truthseeker.

## 1. Verified facts (current source, READ)

1. Frozen command vocabulary = **14** names; `CommandName` has 17 variants incl. 3 decision-gated `DIFF_*`
   (`contract/command.rs:26-61,65-83,121-126`; test lists all 17).
2. Event vocabulary = **21** names (`contract/event.rs:18-61,65-87`).
3. Refusal table = **65** variants, unique/well-formed by test (`contract/refusal.rs:73-282,304-316`).
4. Agent state = **16** variants incl. `Interrupted`, 5 terminal (`contract/state.rs:18-72,146`).
5. Every command path is ACK-or-named-refusal, never silent drop: dispatch publishes ACK/ERROR and an
   `unhandled frozen command` fallback logs + `E_INTERNAL_ERROR` (`core/ws.rs:336-480`); the ERROR payload is
   built only from the frozen `Refusal` (`event.rs:149-160`; `ws.rs:496-503`).
6. Transport gate order = loopback peer → Origin allowlist (missing Origin refused) → token, before upgrade
   (`core/ws.rs:95-118`); `/api/session` returns the token only to allowlisted Origin or loopback peer
   (`core/http.rs:398-425`). Covered by `origin_rejected_when_not_allowlisted` / `token_invalid_is_refused`
   (`core/tests/ws_transport.rs:46-72`).
7. Settings truth: `SETTINGS_STATUS_RESULT` = provider/model/`key_present` bool (+ additive thinking/effort
   from config) — key never read/echoed (`tools/settings.rs:18-26`; `ws.rs:397-415`; test
   `settings_status_command_returns_result_over_ws` asserts no `sk-`/`DEEPSEEK_API_KEY`).
8. History truth: `HISTORY_SNAPSHOT{run_id,revision,graph}` is emitted only from `history.state_at(...)` after
   ACK (`ws.rs:417-437`); UI decodes exactly those fields (`web/src/api/contract.ts:733-739`).
9. Key-material audit (assigned paths): no path reads/logs/returns the DeepSeek key; the only wire credential is
   the local session token (design). `/diagnose` and `/health` carry `provider_key_present` bool only
   (`tools/health.rs:60,419-422`; tests assert no `sk-` in payload).
10. Direct `ProviderExecutorAdapter::from_env` is wired in http with honest `UnconfiguredExecutor` fallback —
    the earlier "provider not wired" P0 is stale at HEAD (paper A-F01 vs `core/http.rs:25,67-82`); a live model
    call remains UNVERIFIED (no key/paid call in this seat).
11. Paper's parity claim (14/21/65/16) reproduces exactly at HEAD; the UI command/event lists also match
    (14/21) but the **UI refusal table has only 45 of 65 codes**.

## 2. Key / refusal security verdict
- **No key-leak defect found in the slice.** Env key is read only via `ApiKey::present_in_env()`/provider
  adapter (outside slice); refusals reference the env *name*, never a value; config/payload/DB/log/UI paths
  audited by class greps and by tests that assert absence.
- **Refusal identity** is a closed enum on the server (65 codes, unique, serialized as stable codes) but the
  wire `ERROR.code` and the UI accept free strings — unknown/forged codes render without a canonical mapping
  (F-S2-09). UI's canonical table is incomplete (F-S2-01), so UI copy/how-to-fix can fall back to raw text for
  20 server codes.
- **Contract parity:** backend 65/16/14-frozen/21 — PASS. UI: 14/21 — PASS; refusal codes 45/65 — FAIL (medium).
  UI agent-state parity is UNVERIFIED (file owned by watchdog-1; paper claims 16 vs 15).

## 3. Remaining defects (all `path:line`, no fixes made)

| ID | Severity | Defect |
|---|---|---|
| F-S2-01 | MEDIUM | UI `KNOWN_REFUSAL_CODES` = 45/65 (`web/src/api/contract.ts:332-378`); 20 contract codes absent (E_AGENT_LIMIT, E_SEMAPHORE_FULL, E_TOKEN_BUDGET_EXCEEDED, E_CALL_LIMIT, E_USAGE_INCOMPLETE, E_PRICE_UNSOURCED, E_PEAK_UNDEFINED, E_COST_MATH_INVALID, E_MULTIKEY_FORBIDDEN, E_LLM_RATE_LIMIT, E_APPROVAL_REQUIRED, E_DECISION_REQUIRED present? no—E_DIFF_DECISION_REQUIRED present, E_DECISION_REQUIRED absent, E_TEMPLATE_INVALID, E_PERSONA_UNKNOWN, E_SNAPSHOT_CORRUPT, E_FILE_TREE_UNAVAILABLE, E_TOOL_NOT_ALLOWED, E_PROVIDER_INVALID_RESPONSE, E_INTERNAL_ERROR) |
| F-S2-02 | MEDIUM | `NODE_REWIRE` broadcasts hardcoded `AgentState::Running` (`core/ws.rs:344-352,483-490`) — state truthfulness defect for non-running rewired tasks |
| F-S2-03 | LOW | "per-session token" is one per-server boot token shared by clients (`core/http.rs:89-92`; `ws.rs:35-56`) |
| F-S2-04 | LOW | token compared with non-constant-time `==` (`core/ws.rs:113`) |
| F-S2-05 | LOW | `/health` exposes absolute DB path (`core/http.rs:222-229,349-362`) |
| F-S2-06 | LOW | `FILE_TREE_RESULT` decoder defaults missing `kind` to `'file'` (`contract.ts:696-705`) |
| F-S2-07 | INFO | resync sends RESYNC_REQUIRED+HELLO without snapshot; stale Phase-9 comment (`core/ws.rs:269-292`) |
| F-S2-08 | INFO | paper A-F01 stale at HEAD (provider wired, honest fallback) |
| F-S2-09 | LOW | wire `ERROR.code` free String + UI accepts unknown codes (`event.rs:137-147`; `contract.ts:507-519`) |
| F-S2-10 | INFO | SETTINGS_STATUS carries undocumented additive thinking/effort (`event.rs:274-279`) |

## 4. File-by-file implementation plan (ForgeWright-plan sections; future writer, not this seat)

**4.1 Outcome.** After the change, the UI can render every one of the 65 server refusals from a single
canonical table, rewire events carry the task's real state, and the localhost auth token is honestly named and
compared; no behaviour regresses and no key material is ever rendered.

**4.2 Current behavior.** Evidence in §1/§3: UI 45/65 table (`contract.ts:332-378`); `ws.rs:344` running
constant; `http.rs:89` single token; `ws.rs:113` plain compare; `/health` path leak; `contract.ts:698` default
kind.

**4.3 Decisions.** (D-a) Single source for refusal metadata: extend the UI table from the backend enum by a
generated artifact (JSON emitted by contract tests) rather than hand-copying — rejected alternative: hand-edit
45→65 (drifts again). (D-b) NODE_REWIRE carries the graph's actual state, read from `GraphAuthority` after a
successful rewire; if the graph has no state API, add a read-only accessor in core (owner b-core) — rejected:
keep the hardcoded running value (lies). (D-c) Token compare via `subtle`-style constant-time compare without
adding a dependency: compare fixed-length SHA-256 digests using the existing in-house SHA-256 (`tools/sha256.rs`)
— rejected: adding a crate for one byte-lane (dependency policy). (D-d) `/health` reports a redacted DB
identifier (`<data-dir>/…/file.db` basename) rather than absolute path — rejected: removing the field
(loses diagnostics). (D-e) UI decoder fails closed (`E_SCHEMA_INVALID`) when a file-tree entry lacks both `kind`
and `is_dir` — rejected: default to `'file'`. (D-f) Rename docs/field comments from "per-session" to
"per-server-instance boot token" OR bind token per connection at `/api/session`; choose rename (cheaper) and
record the decision in the contract doc — rejected: rotation (out of scope, local threat model).

**4.4 File-by-file change.**
- `web/src/api/contract.ts`: replace `KNOWN_REFUSAL_CODES` with the generated full list (65); add
  `decodePayload` strict check at 696-705; add unit test comparing generated list count with 65.
- `backend/crates/hiverouter-contract/tests/*`: add a golden export of `Refusal::ALL` (code,message,how_to_fix,
  retryable) to `web/src/api/refusals.generated.json`; test fails when enum and artifact drift.
- `backend/crates/hiverouter-core/src/ws.rs`: after successful rewire, fetch real task/run state and pass it to
  `publish_node_event` (replace `AgentStateForEvent::running()` at 344, remove the helper 483-490); add
  regression test "rewire a terminal task → NODE_REWIRE state is terminal, not running".
- `backend/crates/hiverouter-core/src/http.rs`: format DB path as file name only (222-229,353); rename
  `SessionAuth` docs to "boot token" (89-92) + contract doc wording; add test `session_token_not_in_health`.
- `backend/crates/hiverouter-core/src/ws.rs:113`: constant-time compare helper in `ws.rs` using
  `hiverouter_tools::sha256_hex` over the two tokens; test wrong token still 401 (`token_invalid_is_refused`).
- `web/src/api/contract.ts:507-519`: optional strict mode: reject codes not in the canonical table only when
  `decodeStrict` is requested (UI still shows unknown codes as `E_UNKNOWN_CODE`); test added.

**4.5 Failure modes.** Missing/unknown code → decodes to `unknown_code` marker, refusal toast still shows raw
code + generic fix (no crash); graph state read fails → ERROR + no NODE_REWIRE (never lie); token compare
mismatch → 401 before upgrade; DB path field absent → diagnostics shows `unavailable` (null), not a fake path;
generated artifact absent at build → test fails the build (drift gate), not a runtime fallback.

**4.6 Tests.** (1) contract test: generated refusals artifact == `Refusal::ALL` (65, unique); (2) UI test
`refusal_codes_cover_contract`; (3) `rewire_terminal_task_state_is_truthful`; (4) `session_token_constant_time`
(unit: equal/unequal digests); (5) `health_redacts_db_path`; (6) `file_tree_decode_requires_kind_or_is_dir`;
(7) re-run `cargo test -p hiverouter-core --test ws_transport` + `npm.cmd run typecheck && npm.cmd test -- --run`.
Commands: `cargo test -p hiverouter-contract -p hiverouter-core`; `npm.cmd --prefix web run typecheck`;
`npm.cmd --prefix web test -- --run`.

**4.7 Rollback.** Each change is a separate commit; inverse = revert commit (generated artifact re-emitted by
the pinned contract test; token-compare helper removed; `/health` path field restored). No migrations, no data
shape changes; UI decoder fallback behaviour is additive (unknown codes already decode today).

**4.8 Out of scope / blockers.** No live provider run (no key/paid call); no DB/HSM work; no change to the
frozen 65/21/14/16 sets (only adding a UI mirror + state truth); `web/src/state/agentState.ts` parity belongs to
watchdog-1/Lead (UNVERIFIED here); history read-model correctness and DB key-freedom are b-core scope.

## 5. Tests / commands for the future implementer
`cargo test -p hiverouter-contract` (matrix/roundtrip/golden), `cargo test -p hiverouter-core --test ws_transport`
(origin/token/fan-out/replay/cycle/settings/files/approval), `cargo test -p hiverouter-tools --lib settings`
(key boolean), `npm.cmd --prefix web run typecheck`, `npm.cmd --prefix web test -- --run`. This read-only seat did
not run them (no rebuilds).

## 6. UNVERIFIED (explicit)
- UI agent-state parity 16 vs 15 (`web/src/state/agentState.ts` not in S2 slice; paper A-F03).
- HISTORY_SNAPSHOT read-model truth (owner b-core; payload/emit path verified only).
- DB schema key-freedom (db.rs outside slice).
- Live provider run / real key behaviour (prohibited in this seat).
- Two official docs pages truncated by fetch (thinking-mode tail, streaming-response schema tail) — no invented
  contract facts used beyond the fetched sections.
- Paper-vs-HEAD stale findings (A-F01) and all runtime claims from the paper — treated as context, not evidence.
