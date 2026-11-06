# W10 — TOOL-CALL LOOP & TAURI DECISION (HiveMind 1.1)

**Seat:** W10 (Tool-call/Tauri decision) · **Mode:** decision document; write-authorized scope `plans/waves/**`
only. **No product code, tests, configs or git writes** (commit reserved to Lead). No key access, no paid calls.
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **HEAD observed:** `8a5b583` ("1.1 wave 2 — true SSE
streaming, replay browser proof + layout fix, deterministic release script, docs truth"; parents `0dfe23b`
wave 1, `23ba74a` release seal). All claims below are READ from the files listed in §1 and §9.

---

## 0. Decision (short)

1. **Model tool-call loop: NO for HiveMind 1.1 as currently scoped.** It is deferred to a **1.2 tool-loop
   wave**, staged A→B→C, and it must not be claimed in 1.1 docs/evidence. The blockers are **contract,
   approval-flow, run-state and persistence** — not model capability (the provider wire types and SSE tool
   deltas already exist).
2. **Tauri/desktop shell: NO for 1.1 (and not required for the tool loop).** v1.1 remains **localhost server +
   browser** (embedded UI via `rust-embed`). Tauri adoption is gated by the six conditions in §7; until then the
   existing truthful packaging statement stands.
3. **No fabricated capability claims:** the shipped release already says autonomous model tool-calling is not
   part of 1.0/1.1 (`docs/RELEASE.md:22`, `docs/RELEASE-EVIDENCE.md:38`). This decision converts that
   statement into an explicit roadmap item instead of an ambient gap. A "yes" requires the Lead to clear §3's
   hard blockers and authorize a contract patch (frozen vocabulary change).

## 1. Recon ledger (files read this session; exact metadata)

| # | File | bytes | lines | sha256 | Relevant anchors |
|---|---|---:|---:|---|---|
| 1 | `backend/crates/hiverouter-provider/src/dto.rs` | 24,723 | 690 | `aa6443f9…26c8a7` | `ToolChoice` 215-222; `ToolCall` 269-277; `FunctionDefinition` 281-289; `ToolSpec` 293-309; `ChatRequest.tools` 331-336; `ResponseMessage.tool_calls` 529-532; `Delta.tool_calls` 579-581; `DeltaToolCall` 598-613; `StreamChunk` 626-641 |
| 2 | `backend/crates/hiverouter-provider/src/client.rs` | 29,214 | 748 | `f46b6f10…5b62d3` | `StreamEvent::ToolCallDelta` 116-126; `pump_stream` 569-668; `delta_event` 670-702; no assembler; no budget defects |
| 3 | `backend/crates/hiverouter-core/src/provider_executor.rs` | 15,136 | 406 | `64eb7a91…cde884` | `stream_reply` 103-208; ToolCallDelta only sets a flag 171-173; empty text ⇒ `E_PROVIDER_INVALID_RESPONSE` 195-203; `message_text` 279-305; no `with_tools` anywhere |
| 4 | `backend/crates/hiverouter-core/src/execution.rs` | 23,530 | 641 | `9b055295…52a82d4` | `ProviderExecutor` trait 49-77; `StreamedReply` 84-98; `dispatch_inner` plan+leaf only 501-574; `execute_task` 591-…; caps.check_calls(1) 505,597; no suspension/resume |
| 5 | `backend/crates/hiverouter-tools/src/tool.rs` | 19,612 | 558 | `353ab4e6…319765` | `ToolId` 102-151 (7 tools); `ToolContract`/schema validation 153-187; registry + tier allowlists 291+ |
| 6 | `backend/crates/hiverouter-tools/src/approval.rs` | 34,944 | 948 | `e32411e6…652c22` | DDL `UNIQUE(nonce)` 141-165; `insert_request` 171; `resolve_with_nonce` 415; `ApprovalGate::request` 580 (only test callers); `verify_grant` 653; TTL 120 s 26 |
| 7 | `backend/crates/hiverouter-core/src/ws.rs` | 21,447 | 537 | `d418dda6…b48307a` | `ApprovalResolve` arm 465-476; `SettingsStatus` 403-422; `HistoryRevisionSet` 423-444; no APPROVAL_REQUEST emitter |
| 8 | `backend/crates/hiverouter-core/src/tools_bridge.rs` | 6,788 | 167 | `4767e3e0…dc5f8e` | `migrate` 73-79; `ApprovalGate::new` 62; `resolve_approval` 141; `approval_resolved_event` 173 |
| 9 | `backend/crates/hiverouter-contract/src/event.rs` | 18,508 | 489 | `c5776ef6…09b2aa` | 21 events; `ApprovalRequest` payload 244-254 = **{approval_id, tool, decision?} only**; `ApprovalResolved` 256+; no TOOL_CALL/TOOL_RESULT |
| 10 | `backend/crates/hiverouter-contract/src/command.rs` | 21,334 | 533 | `2eeafcdd…73964f` | 14 frozen commands (17 incl 3 decision-gated DIFF) incl. only `APPROVAL_RESOLVE`; no resume/tool command |
| 11 | `backend/crates/hiverouter-core/src/db.rs` | 15,761 | 420 | `1822162a…40e8a47c` | `chat_messages` schema: id/run_id/role/text/created_at only (74-82) |
| 12 | `backend/crates/hiverouter-core/src/store.rs` | 15,733 | 431 | `ca2bc6dd…45fdf9eb` | `append_message`/`messages` 268-338; active status includes `awaiting_approval` 101; state mapping 445 |
| 13 | `plans/HIVEMIND-1.0-EXECUTION-PLAN.md` | 8,341 | 64 | `5cdbdba4…3ddfd53` | line 58: "W10 Tool-call loop and Tauri — explicit product decisions"; line 74: tool calls + Tauri are 1.1 |
| 14 | `plans/1.1-WAVE-1-RECOVERY.md` | 3,993 | 38 | `4797b563…d9fd73e` | line 55: tool-call loop (W10) explicitly out of scope of 1.1 wave 1 |
| 15 | `plans/waves/W3-STREAMING.md` | 13,433 | 150 | `c80788a4…c12a3f5` | lines 83/110: `ToolCallDelta` remembered, leaf tool calls unsupported ⇒ `E_PROVIDER_INVALID_RESPONSE` |

Supporting verified checks this session:
- `src-tauri` absent; no `tauri` dependency in `web/package.json` or any backend crate manifest (checked
  all five manifests) — Tauri is NOT implemented anywhere.
- `ApprovalGate::request` (the only producer of pending approval rows) has **no production caller**; the only
  callers are `approval.rs` tests. `tools_bridge.rs` exposes only `resolve_approval`; no code publishes
  `Event::ApprovalRequest`.
- `docs/RELEASE.md:22` — "Autonomous model tool-calling is not part of 1.0; … the model does not yet drive
  multi-turn tool calls." `docs/RELEASE-EVIDENCE.md:38` — "Autonomous model tool-calling … are 1.1."
  → the 1.1 claim is currently **forward-looking**, and this decision makes it a 1.2 item instead.

## 2. Current behavior (verified, no inference from filenames)

1. **The provider layer is tool-aware but the loop is not wired.** DTOs cover the documented function-tool
   contract (`ToolChoice none|auto|required`, `ToolSpec`, `ToolCall`, response `tool_calls`, streamed
   `DeltaToolCall` fragments). The SSE pump forwards each fragment as `StreamEvent::ToolCallDelta`
   (`client.rs:116-126,670-702`) but **no component assembles fragments into a complete `ToolCall`**.
2. **No request ever advertises tools.** `provider_executor.rs` builds the planner request (219-231) and the
   leaf request (255-263) with no `.with_tools(...)` / `.with_tool_choice(...)`; a repo-wide grep for
   `with_tools|ToolSpec|ToolChoice` in `core/` and `tools/` returns no production call site. Therefore, per the
   official docs (tools only usable when supplied), the model cannot legitimately issue a tool call today.
3. **If a provider did return tool calls anyway, the current behavior is a named refusal.** `stream_reply`
   records `tool_calls_seen = true` (171-173) and, if no visible text arrives, returns
   `E_PROVIDER_INVALID_RESPONSE` (195-203); `message_text` logs `has_tool_calls` and refuses the same way
   (279-305). The W3 plan documents this as the accepted 1.0/1.1 behavior (`W3-STREAMING.md:83,110`).
4. **There is no tool-execution seam in the run path.** `ProviderExecutor::execute` returns
   `StreamedReply { deltas, usage }` (execution.rs:84-98); the engine publishes `REPLY_DELTA`s and persists text
   (execution.rs:591-…); neither the engine nor the adapter can invoke `tool.rs`'s registry/executors on behalf
   of a model turn.
5. **The approval gate is real but has no producer and no wire nonce.** `approval.rs` implements the full
   consent contract (args_sha256 binding, `UNIQUE(nonce)`, single-use decisions, TTL 120 s, replay/timeout
   refusals). But (a) `ApprovalGate::request` is not called by production code; (b) the frozen
   `APPROVAL_REQUEST` event payload is only `{approval_id, tool, decision?}` (`event.rs:244-254`) — it carries
   **no nonce, no args_sha256, no run/task/tool-call ids**, while `APPROVAL_RESOLVE` requires a nonce
   (`command.rs` ApprovalResolve; `approval.rs:415-440`). A UI therefore cannot resolve a model-originated
   approval today.
6. **Persistence cannot store a tool conversation.** `chat_messages` has only
   `id, run_id, role, text, created_at` (`db.rs:74-82`; `store.rs:268-338`). There is no place for assistant
   `tool_calls`, `tool_call_id`, `reasoning_content`, provider response id, or turn index — all required for a
   multi-turn tool conversation (and the docs require `reasoning_content` pass-back when tools are present
   with thinking enabled — FACT/docs: HTTP 400 otherwise).
7. **Run state is terminal-linear; `awaiting_approval` has no producer.** `dispatch_inner` goes
   Queued→Planning→leaf execution→terminal (`execution.rs:501-574`); `AgentState::AwaitingApproval` exists in
   the enum and `store.rs:101,445` recognizes it as active, but no transition produces it and no resume API
   exists. Cancellation is a `CancellationToken` per run (execution.rs:128-175).
8. **Tauri is absent.** No `src-tauri`, no tauri crate/npm dependency; UI is served from the embedded
   `web/dist` by the localhost server. Packaging is a zip of two executables + docs (1.0 release record).

## 3. Hard blocker list (why "NO" for 1.1 — each must be cleared before a "yes")

| ID | Blocker | Evidence | Required resolution (owner) | Acceptance test |
|---|---|---|---|---|
| **B1** | **Frozen contract lacks model-tool vocabulary and approval nonce on the wire.** No `TOOL_CALL`/`TOOL_RESULT` events; `APPROVAL_REQUEST` has no `nonce`/`args_sha256`/`run_id`/`task_id`/`tool_call_id`; `APPROVAL_RESOLVE` requires a nonce the UI never receives. | `event.rs:244-254`; `command.rs` ApprovalResolve; `approval.rs:415-440`; no `Event::ApprovalRequest` producer (grep). | Contract patch (CONTRACT owner + Lead): additive `TOOL_CALL`/`TOOL_RESULT` events, extend `APPROVAL_REQUEST`, regenerate golden schemas, add matrix rows; `CHANGE-LEDGER-IMPL.md` row; UI decoders. | `cargo test -p hiverouter-contract` matrix/golden/roundtrip pass with the new rows; UI contract parity test. |
| **B2** | **No run suspension/resume.** Run execution is a single linear dispatch; no persisted phase, no waiter for an approval, no resume-with-tool-results. | `execution.rs:49-98,501-574`; `store.rs:101,445` (state accepted, never produced). | Run-engine change (CORE owner): persisted `awaiting_approval` transition, per-approval waiter (oneshot/broadcast keyed by approval id), resume on APPROVAL_RESOLVE, TTL/cancel/kill handling, restart policy. | Deterministic suspension test: approve → resume → tool executes once; deny/timeout/cancel paths; restart during wait ⇒ run fails `E_APPROVAL_TIMEOUT`, never auto-executes. |
| **B3** | **Conversation persistence cannot represent tool turns.** `chat_messages` lacks tool_calls/tool_call_id/reasoning_content/response-id/turn fields. | `db.rs:74-82`; `store.rs:268-338`. | Additive migration (CORE owner): e.g. `meta_json TEXT NULL`, `turn INTEGER NOT NULL DEFAULT 0`; write/read APIs; replay/history includes tool turns; down migration. | Migration up/down roundtrip test; append assistant-with-tool_calls + tool result + reasoning_content and read back byte-equal. |
| **B4** | **No tool-call assembly/execution seam.** Client emits fragments only; adapter discards them; no `ToolCall → ToolRegistry → execute → tool result` path; no result-size/redaction policy for returning outputs to the model. | `client.rs:116-126,670-702`; `provider_executor.rs:171-173`; `tool.rs` registry; `jail.rs`/`shell.rs` executors exist but unwired to model turns. | Adapter/engine work (PROVIDER + CORE): index-keyed assembler, JSON arguments parse/validate, tier allowlist resolution, jailed execution, result truncation (bounded bytes + sha256), ledger row. | Wiremock multi-turn: turn 1 returns tool-call fragments → tool executes in temp jail → turn 2 request contains assistant `tool_calls` + `role=tool` result; malformed/unknown/not-allowed/oversize cases refuse or feed back by name. |
| **B5** | **Budget/caps are not loop-aware.** Caps count one planning call + one call per leaf; per-run budget map lives in the adapter and is evicted at terminal; no per-turn/per-tool caps; a loop can multiply calls and tokens. | `execution.rs:505,597`; `provider_executor.rs:35,79-95,274-276`; `budget.rs` policy caps. | Loop turn/call accounting (PROVIDER + CORE): `MAX_TOOL_TURNS`, per-turn call count, per-run tool-call cap, wall-clock/tool-timeout, budget persists across suspension (no eviction until terminal). | Loop-cap tests: turn cap, tool-call cap, call/budget refusal names; suspension does not evict budget; usage accounting across turns is complete or INCOMPLETE. |
| **B6** | **Approval UX/security for model-originated side effects is undefined/unbuilt.** No operator card with args hash/args preview, TTL countdown, deny path, cancel-during-wait, session grant semantics, or threat review for model-driven shell/file-write/spawn. | `approval.rs`; `event.rs:244-254`; no UI approval producer; `shell.rs`/`jail.rs` exist. | Security + UX decision (Lead + FRONTEND): approval payload fields, args display/redaction, deny/timeout behavior, kill/cancel while waiting, staged tool allowlist (read-only first), threat model review. | Approval e2e: request→approve→single execution; request→deny→tool result "denied"; TTL→`E_APPROVAL_TIMEOUT`; replay→`E_APPROVAL_REPLAY`; cancel while waiting→`E_TOOL_CANCELLED`; no second execution. |
| **B7** | **Deterministic evidence harness for loops does not exist.** No wiremock scripted multi-turn provider in core tests; no paid-call budget is available for live verification; e2e cannot drive a real model deterministically. | `provider_executor.rs::new` accepts an injected client (59-67); `wiremock` is provider-crate dev-only; core tests use `MockExecutor` for lifecycle only. | Test infrastructure (CORE): injectable wiremock-backed adapter in core integration tests; e2e scenario with fake provider; gate logs must carry HEAD + exit code. | `cargo test -p hiverouter-core --test tool_loop` (new) green offline; e2e replay/tool activity with fake provider; no key/paid call. |
| **B8** | **Tauri/platform decision is separate and not required for the tool loop.** | no `src-tauri`, no tauri deps; release = localhost+browser (`docs/RELEASE.md:22-23`). | Product decision (Lead/user): clear the six gates in §7 first. | Installer build/sign/uninstall/e2e evidence before any installer claim. |

If the Lead wants the loop **inside 1.1**, B1+B2+B3+B4+B5+B7 must all clear first plus B6 for mutating tools;
they are not small flag flips, and B1 is a frozen-contract change. That is why the decision is NO for 1.1.

## 4. If-yes exact design (target 1.2; decision-complete, staged A/B/C)

### 4.1 Staging

| Phase | Tools exposed to the model | Contract change? | Approval needed? | Outcome |
|---|---|---|---|---|
| **A — read-only loop** | `file_read`, `file_list` (WorkerAgent tier only) | No wire change (ledger-only visibility; reply text only) | No (Read class) | model can inspect the jailed workspace across ≤N turns and answer; every tool call in the JSONL run ledger + `chat_messages.meta_json` |
| **B — gated side effects** | + `file_write`, `shell_exec` (execute via approval) | **Yes** (B1: TOOL_CALL/TOOL_RESULT + APPROVAL_REQUEST nonce/args) | Yes (Mutate/Execute) | model can mutate/run only after operator approval; suspension/resume |
| **C — multi-agent tools** | + `spawn_agent`/`report_result`/`finish` wiring into the DAG engine | Additive | Spawn policy (caps) | model-driven delegation inside existing depth/fan-out/agent caps |

### 4.2 Turn loop (adapter-level; provider crate stays transport-only)

1. Leaf request (Phase A) = existing leaf messages + `.with_tools(tier_tools(WorkerAgent, {file_read,
   file_list}))` + `.with_tool_choice(ToolChoice::Auto)`; leaf thinking stays **disabled** (current default)
   so the docs' forced-tools-in-thinking 400 rule and `reasoning_content` pass-back rule do not bite; if leaf
   thinking is enabled later (Phase C), the loop must persist `reasoning_content` and replay it on the
   assistant tool-call message (docs requirement; otherwise provider 400).
2. Each turn: `client.stream_with_budget(...)` → collect text (emit deltas) and assemble tool calls via
   `ToolCallAssembler` (index-keyed: `id` first fragment, `name` first fragment, `arguments` concatenated in
   order; JSON-parse argv; require `finish_reason == "tool_calls"` when calls present; missing id/name ⇒
   `E_PROVIDER_INVALID_RESPONSE`; invalid JSON argv ⇒ tool result `E_TOOL_SCHEMA` fed back, not run-fatal).
3. If no tool calls: final answer = concatenated text; return `StreamedReply` (existing behavior); end run.
4. If tool calls: for each call in index order:
   a. `ToolId::from_id(name)` → unknown ⇒ tool result `E_TOOL_NOT_FOUND` (model may recover).
   b. `ToolRegistry::resolve(tier, id)` → `E_TOOL_NOT_ALLOWED` ⇒ tool result by name (never execute).
   c. `validate_args(argv)` → `E_TOOL_SCHEMA` ⇒ tool result with the validation message.
   d. Phase A: execute read tool through `WorkspaceJail` (`file_read` 1 MiB/UTF-8 rules; `file_list`); Phase B:
      if `requires_approval()` → insert pending approval, suspend (below), then `verify_grant` (args_sha256
      single-use) → execute; `shell_exec` through `ShellPolicy` allowlist + timeout + job object; `file_write`
      through jail.
   e. Result policy: text result ≤ 32 KiB (truncate with `[…truncated sha256=<hash>]` marker); binary → refusal
      code text; persist each call (ledger + DB); publish `TOOL_CALL`/`TOOL_RESULT` (Phase B contract).
   f. Append assistant message `{role: assistant, content: text, tool_calls: [...]}` (and reasoning_content if
      thinking) + one `role=tool` message per call with `tool_call_id`; continue next turn.
5. Bounds: `MAX_TOOL_TURNS = 4`, `MAX_TOOL_CALLS_PER_TURN = 4`, `MAX_TOOL_CALLS_PER_RUN = 12`, per-call
   timeout (shell 60 s; file ops bounded), per-run provider call cap unchanged (40) and budget caps
   (50¢/400k/40 calls). On bounds: synthesize a tool result "tool budget exhausted" for the current call and
   let the model finalize within the remaining turn budget; if provider call cap/budget refuses → run fails with
   the existing named code (`E_CALL_LIMIT`/`E_BUDGET_EXCEEDED`/`E_TOKEN_BUDGET_EXCEEDED`). A new
   `E_TOOL_LOOP_LIMIT` code, if product wants a distinct name, is part of the B1 contract patch.
6. Provider failures mid-loop: 401/402 no retry (run fails `E_PROVIDER_AUTH`); 429 respects retry policy
   (run fails `E_LLM_RATE_LIMIT` after attempts); stream without `[DONE]` → `E_PROVIDER_INCOMPLETE` (no tool
   execution on a truncated turn); cancel token checked before each turn and before each tool call →
   `E_TOOL_CANCELLED`, no further side effects.

### 4.3 Approval lifecycle (Phase B)

- Producer: adapter on a `requires_approval` tool → `ApprovalGate::request(run, task, tool_call_id, tool,
  permission_class, args, now)` inserts the pending row (nonce, args_sha256, TTL 120 s).
- Wire: `APPROVAL_REQUEST` (extended payload) with `approval_id`, `nonce`, `run_id`, `task_id`,
  `tool_call_id`, `tool`, `args_sha256`, `expires_at`, optional redacted args preview.
- State: run → `awaiting_approval` (persisted + `RUN_STATUS` event + task `NODE_STATE`); executor waits on a
  per-approval wake channel; `APPROVAL_RESOLVE(nonce, decision)` → gate `resolve_with_nonce` (single-use) →
  wake waiter → approve ⇒ `verify_grant` then execute once; deny ⇒ tool result "denied by operator" (model
  continues); replay ⇒ `E_APPROVAL_REPLAY`; TTL expiry ⇒ run fails `E_APPROVAL_TIMEOUT` with no side effect;
  `CANCEL_RUN`/`KILL_WORKER` while waiting ⇒ `E_TOOL_CANCELLED`, pending row consumed/expired, no execution.
- Restart policy: runs persisted in `awaiting_approval` without a live waiter are transitioned to
  `failed`/`E_APPROVAL_TIMEOUT` on boot (ledger row); pending approvals are never auto-approved or replayed.
- Session grants stay internal-only; the frozen wire exposes `approve|deny` (AllowSession remains programmatic
  until a future contract adds a wire value).

### 4.4 Events / commands (contract patch B1)

- Add `TOOL_CALL` event payload `{run_id, task_id, tool_call_id, tool, args_sha256, turn}`.
- Add `TOOL_RESULT` event payload `{run_id, task_id, tool_call_id, outcome: ok|refused|denied|timeout|cancelled,
  code?: E_*, bytes?: u64, sha256?: string}`.
- Extend `APPROVAL_REQUEST` payload with `nonce`, `run_id`, `task_id`, `tool_call_id`, `args_sha256`,
  `expires_at` (existing `approval_id`/`tool`/`decision?` stay).
- No new command is strictly required: `APPROVAL_RESOLVE` (exists) resolves; `CANCEL_RUN`/`KILL_WORKER`
  (exist) cancel. Optionally document that a future `TOOL_APPROVE_SESSION` needs a contract decision.
- Process: `CHANGE-LEDGER-IMPL.md` row, golden schemas regenerated, matrix rows added, UI `contract.ts`
  decoders + parity test, frozen counts become 21+2 events; command count unchanged; refusal count unchanged
  unless `E_TOOL_LOOP_LIMIT` is added (then +1 with a test).

### 4.5 Persistence (B3)

- Additive migration: `chat_messages` gains `meta_json TEXT NULL` and `turn INTEGER NOT NULL DEFAULT 0`
  (assistant: `{tool_calls:[…], reasoning_content?, response_id?}`; tool: `{tool_call_id, outcome, code?}`);
  dedicated `tool_calls` ledger rows (JSONL `RunLedger` already exists) carry args_sha256/permission/outcome.
- Replay/history: snapshot + `HISTORY_SNAPSHOT` include tool turns (state shown via existing node states;
  tool activity readable from messages); revision semantics unchanged.
- Down migration: drop the added columns via table rebuild (SQLite DROP COLUMN needs ≥3.35; rusqlite bundled is
  newer) or keep columns and no-op; the migration must be reversible per `db.rs` migration style.

## 5. Failure modes (design-level; each needs RED before GREEN in the wave)

| # | Failure | Expected behavior |
|---|---|---|
| F1 | Tool-call fragments missing id/name, or concatenated argv is invalid JSON | `E_PROVIDER_INVALID_RESPONSE` (missing identity) / tool result `E_TOOL_SCHEMA` (bad argv, model may retry within caps); never execute unparsed args |
| F2 | Unknown / not-allowed tool name from model | tool result with `E_TOOL_NOT_FOUND` / `E_TOOL_NOT_ALLOWED`; no execution; ledger row |
| F3 | Read tool escapes jail / oversize file / binary | tool result with `E_PATH_OUTSIDE_JAIL` / `E_FILE_TOO_LARGE` / `E_TOOL_SCHEMA`; no content leak |
| F4 | Side-effect tool without approval / approval denied | no side effect; `awaiting_approval` round-trip; deny ⇒ tool result "denied", model finalizes |
| F5 | Approval TTL expires / replayed nonce / invalid decision | run fails `E_APPROVAL_TIMEOUT` (no side effect) / `E_APPROVAL_REPLAY` / `E_APPROVAL_DECISION_INVALID` |
| F6 | Cancel/kill during a turn, a tool call, or an approval wait | `E_TOOL_CANCELLED`; pending approval consumed; no further side effect; run terminal cancelled |
| F7 | Provider 401/402/429/timeout/stream-without-DONE mid-loop | run fails with the existing provider refusal; no tool execution on the failed turn; usage INCOMPLETE if unknown |
| F8 | Turn/tool-call/call/budget caps | cap refusal by name before the next provider send; no silent unbounded loop |
| F9 | DB/ledger write failure during a tool turn | run fails `E_INTERNAL_ERROR`/`E_DB_BUSY`; no execution continues after an unpersisted decision (fail closed) |
| F10 | Restart while awaiting approval | boot marks run failed `E_APPROVAL_TIMEOUT`; pending approvals expire; never auto-execute |
| F11 | Malformed/partial 200 stream with tool deltas and no `[DONE]` | `E_PROVIDER_INCOMPLETE`; no tool execution; no retry after first delta |
| F12 | Tool result too large / contains key-shaped text | truncate + hash marker; audit/ledger redaction probe; never echo a key (child env already sanitized) |

## 6. Tests (named; commands; all deterministic/offline unless stated)

Backend (`cd backend`; PATH with cargo + `tools/w64devkit/bin`):
1. `cargo test -p hiverouter-provider` — `tool_call_fragments_assemble_in_order` (new assembler unit:
   id/name first, argv concatenation, malformed JSON → `E_TOOL_SCHEMA`), existing SSE/refusal matrix stays green.
2. `cargo test -p hiverouter-core --test tool_loop` (new) — wiremock-injected adapter scripted two turns:
   turn1 tool_calls → tempdir `file_read` executes → turn2 final text; assert second request body contains
   assistant `tool_calls` + `role=tool` message with `tool_call_id`; assert `TOOL_CALL`/`TOOL_RESULT` (B) and
   persistence rows; unknown tool / invalid argv / jail escape / oversize cases; bounds tests
   (`MAX_TOOL_TURNS`, per-run tool cap, call cap); cancel mid-turn.
3. `cargo test -p hiverouter-tools --lib approval::tests` — existing replay/TTL/args-binding; add B-phase
   approval-with-suspension integration (approve→execute once, deny→tool result, TTL→timeout, replay→replay).
4. `cargo test -p hiverouter-contract` — matrix/golden/roundtrip with B1 payloads; census test updated counts.
5. `cargo test -p hiverouter-tools --lib shell::tests` — unchanged child-env isolation; tool timeouts.

UI (`cd web`; `npm.cmd run typecheck; npm.cmd test -- --run`):
6. `contract.test.ts` — decodes `TOOL_CALL`/`TOOL_RESULT`/extended `APPROVAL_REQUEST`; refusal parity list
   unchanged; unknown tool events tolerated.
7. `store.test.ts` — tool activity recorded from events; approval card renders nonce/args hash/TTL; approve/deny
   sends `APPROVAL_RESOLVE` once; replay includes tool turns; no optimistic success.
8. `toolActivity.test.ts` (new) — a tool error result renders a refusal code + how_to_fix, not a spinner.

E2E (`cd e2e`; `E2E_APP_READY=1`, fake provider):
9. `tool-loop.spec.ts` (new) — read-only loop (A): submit → tool calls visible → final answer; no fake success.
10. approval flow (B): request card → deny path, approve path (temp workspace), TTL path; exactly one execution.
11. gate logs must record HEAD + command + exit code (S7 finding) for every claim.

No paid provider call is required by any of these; a manual live smoke is optional, operator-key-gated, and
must not be a release gate (budget policy unchanged).

## 7. Tauri decision (separate, explicit)

**Decision: NO now; HiveMind 1.1/1.2 stay localhost-server + browser** (`rust-embed` UI; current zip release).
Rationale: no Tauri code/deps exist; an installer/desktop shell is a packaging claim with real distribution
risks (WebView2 runtime, code signing, uninstall, update) that the project cannot evidence offline today; the
tool-call loop does not require it. Adoption gates (all must be green before a "yes"):
1. WebView2 runtime detection + honest fallback/instructions on Win10/11 (official runtime check).
2. Code-signing certificate decision (the current binaries are unsigned; SmartScreen consequence documented).
3. Installer + uninstaller + upgrade e2e on a clean Windows profile (VM/sandbox), evidence tracked.
4. Build-time/size budget measured (cargo-tauri deps; release build time; installer size) versus the localhost
   zip; no dependency added without the pin/licence row.
5. Truthful packaging docs update (desktop shell claim only after 1–4 evidence).
6. Product/user decision recorded (Tauri is a distribution choice, not a capability fix).

## 8. Rollback

- Phase A: feature flag `HIVEROUTER_TOOL_LOOP=off` default; code behind it; revert = remove flag/commits, no DB
  change if the tool-call persistence is behind the migration. Kill switch keeps 1.1 behavior
  (`E_PROVIDER_INVALID_RESPONSE` on tool calls).
- Phase B: contract patch is a separate commit with CHANGE-LEDGER row; rollback = revert patch + restore
  golden schemas (versioned files); DB migration additive with documented down (table rebuild/no-op); UI
  decoders tolerate absent events.
- Phase C: spawn integration behind caps; revert restores leaf-only planning.
- Tauri: not implemented now ⇒ no rollback needed; if adopted later, the localhost build remains the fallback
  artifact and the release gate can ship it without the installer.

## 9. Out of scope / non-claims

- No product code was written by this seat; no tests executed; no builds; no key access; no paid calls; no git
  writes (Lead owns the commit). W10 writes only `plans/waves/W10-TOOL-LOOP-DECISION.md`.
- The decision does **not** claim the tool loop exists today; §2 facts are verified reads, §4/§6 are designs
  for a future wave and are not evidence of implementation.
- Out of scope: live provider tool-call behavior (needs a paid/key-gated smoke; UNVERIFIED), pixel fidelity,
  full trace ladder, installer/Tauri implementation, contract version bump beyond the specified additive patch,
  multi-agent delegation semantics beyond existing caps.
- **UNVERIFIED:** exact provider behavior for tool calls under `thinking` enabled with tools (docs say HTTP 400
  unless reasoning_content is passed back; not exercised live); model quality of tool-argument JSON; cost of a
  real loop; wiremock fidelity vs live SSE ordering on very long arguments; whether the Lead/user wants the
  loop at all (product decision).
- **What would change this decision:** Lead approval of B1 contract patch + a dedicated 1.2 tool-loop wave with
  B2–B7 accepted; or a product mandate that 1.1 must be autonomous-tools-capable (in which case the honest
  release statement in `docs/RELEASE.md:22`/`docs/RELEASE-EVIDENCE.md:38` must change only after the blockers
  clear — never before).
