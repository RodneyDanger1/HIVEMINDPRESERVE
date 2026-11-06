# W11 — 1.2 contract patch: tool-call vocabulary (Seat A)

**Seat:** A (contract) · **Repo:** `C:\Users\Riley\Desktop\hiverouter-v1`
**HEAD at recon:** `9651722` (`hivemind-1.1.0-2-g9651722`), clean tree; release tag 1.1 sealed (no rebuild in this seat).
**Write scope (as assigned):** `backend/crates/hiverouter-contract/src/{event,command,schema,refusal}.rs`, its
tests/golden schemas, and `web/src/api/contract.ts` **only**. No runtime tool execution, no core/provider/tools edits,
no commit. **Skills applied:** `/forgewright-recon`, `/forgewright-lead`, `/logic-gatekeeper`,
`/integration-logic`, `/logic-debugger`.

## Recon

### W10 blockers (read in full: `plans/waves/W10-TOOL-LOOP-DECISION.md`)

| ID | Blocker | Relevance to this seat |
|---|---|---|
| **B1** | Frozen contract lacks model-tool vocabulary and approval nonce on the wire (`event.rs` §3: no `TOOL_CALL`/`TOOL_RESULT`; `APPROVAL_REQUEST` = `{approval_id, tool, decision?}` only; `APPROVAL_RESOLVE` requires a nonce the UI never receives). | **This wave's target**: additive contract patch only (events + approval fields + golden + matrix/decoders). |
| B2 | No run suspension/resume (core) | Out of scope; needs CORE owner wave. |
| B3 | Conversation persistence cannot represent tool turns (DB) | Out of scope; needs CORE migration. |
| B4 | No tool-call assembly/execution seam (provider+core) | Out of scope; provider assembler wave. |
| B5 | Budget/caps are not loop-aware | Out of scope; provider/budget wave. |
| B6 | Approval UX/security for model-originated side effects undefined | Partially enabled by B1 fields; UI/store work is a later wave. |
| B7 | Deterministic loop evidence harness absent | Out of scope; test-infra wave. |
| B8 | Tauri/platform decision separate | No action; unchanged. |

W10 §4.4 is the authoritative additive spec: `TOOL_CALL` payload `{run_id, task_id, tool_call_id, tool,
args_sha256, turn}`; `TOOL_RESULT` payload `{run_id, task_id, tool_call_id, outcome: ok|refused|denied|timeout|
cancelled, code?: E_*, bytes?: u64, sha256?: string}`; extend `APPROVAL_REQUEST` with `nonce`, `run_id`, `task_id`,
`tool_call_id`, `args_sha256`, `expires_at`; no new command; refusal count unchanged (no `E_TOOL_LOOP_LIMIT` in this
patch); process = matrix/golden/counts + UI `contract.ts` decoders + parity tests.

### Current counts (verified from source at HEAD `9651722`)

- Backend contract: **21 events** (`event.rs:18-61`, `EventName::ALL:65-87`), **17 commands** (14 frozen + 3
  decision-gated DIFF; `schema.rs` matrix 17 rows), **65 refusals** (`refusal.rs`), **16 agent states**.
- Golden snapshot `schemas/contract-v1.json`: `contract = hiverouter.v1`, `commands = 14`, `events = 21` (python count).
- UI `web/src/api/contract.ts`: `EVENT_NAMES` 21 (line 294-316), `EventPayloads` 21 (268-290), `DecodedEvent` 21
  variants (490-511), `EVENT_NAMES` satisfies `EventName`; `FROZEN_COMMAND_NAMES` 14; `KNOWN_REFUSAL_CODES` 65;
  `ApprovalRequestPayload` already carries optional `nonce` (line 223-229) but no `args_sha256`/run/task/tool-call/
  expires fields; no tool-call decoders.
- UI decoder entry points: `validateEnvelope` (433), `decodePayload` (517, one switch case per event),
  `parseEventFrame` (768); store `applyEvent` (`web/src/state/store.ts:217`) has no TOOL_* case (out of scope).
- Downstream exhaustive matches checked: core `ws.rs:534` and execution/bus matches use wildcards or explicit
  variants; adding contract event variants does not break core compile (verified by reading `ws.rs:523-543`,
  `bus.rs:209-245`, `execution.rs:300-369`; compile check still runs).
- Existing count assertions to update (contract tests only): `schema_roundtrip.rs:159,167` (`21`).
  **Web scope conflict:** `web/tests/contract.test.ts:51` also hardcodes `toHaveLength(21)`; that file is outside the
  assigned web scope (only `contract.ts`), so it is reported as blocked-on-scope rather than edited (Lead asked).
- Existing `ApprovalRequest` struct literal sites: contract test fixture only (`tests/common/mod.rs:140`);
  production never emits `APPROVAL_REQUEST` (`grep Event::ApprovalRequest` → only contract source/name arms + fixture).
  `hiverouter-tools::approval::ApprovalRequest` is a different struct (DB row) and is untouched.

### Data flow / integration seam

`provider SSE ToolCallDelta -> (future) adapter assembler -> ToolCallPayload -> bus publish -> envelope serialize ->
WS -> UI decodePayload` ; approval: `ApprovalGate::request -> ApprovalRequestPayload (nonce/args_sha256/ids) ->
WS APPROVAL_REQUEST -> UI resolve -> APPROVAL_RESOLVE(nonce)`; `ToolResultPayload` (future adapter -> bus) reports the
outcome/refusal/bytes/sha without carrying file content. This patch defines the wire only; no producer/consumer runtime
exists yet (W10 B2/B4/B5 remain).

## Phase 1 (logic-gatekeeper) — JSON for the mutation

```json
{
  "phase1": {
    "architecture": "hiverouter-contract owns the frozen wire vocabulary: EventName/Event/payload structs (event.rs), the schema map (schema.rs), refusal table (refusal.rs) and golden snapshot; web/src/api/contract.ts mirrors EventPayloads/EVENT_NAMES/DecodedEvent/decodePayload. Adding events is append-only; extending ApprovalRequest is optional-field-only.",
    "dataFlow": "Untrusted payload JSON enters decode via Envelope.decode_payload (serde + jsonschema). New optional only: TOOL_CALL/TOOL_RESULT payloads validate required ids/outcome; APPROVAL_REQUEST optional nonce/args_sha256/run/task/tool_call/expires fields decode from old and new frames. Nothing executes tools in this patch; UI only parses.",
    "edgeCases": [
      "old APPROVAL_REQUEST without new fields still decodes (all new fields Option+serde default) and reserializes without null noise",
      "TOOL_RESULT with unknown outcome string is refused by jsonschema (E_SCHEMA_INVALID), not silently coerced",
      "TOOL_CALL with empty/blank tool_call_id or non-64-hex args_sha256 is schema-invalid (identity is required before any future execution)",
      "golden snapshot/UI parity: backend 23 events == EventPayloads/EVENT_NAMES keys == DecodedEvent variants"
    ],
    "rootCause": "W10 B1: the frozen contract has no model-tool event vocabulary and its APPROVAL_REQUEST payload omits the nonce/args binding that APPROVAL_RESOLVE requires, so a 1.2 tool loop cannot be expressed on the wire without an additive contract patch."
  }
}
```

Phase 2 (after Phase 1 accepted): append-only enum variants + optional payload fields; existing 21 event decoders and
the 65-code refusal table stay stable; new payloads are validated by the same derived-schema/jschema path; callers
(core/provider/tools) remain compiler-compatible because no existing struct field/type changes.

## Design (decisions, with rejected alternatives)

1. **Events appended, not inserted**: `ToolCall`/`ToolResult` appended after `Pong`; wire names `TOOL_CALL`,
   `TOOL_RESULT`; `EventName::ALL` 21→23. Rejected: inserting after NODE_STATE (reorders the frozen census for no
   benefit).
2. **Payload shapes exactly W10 §4.4**: `ToolCallPayload {run_id: RunId, task_id: TaskId, tool_call_id: String,
   tool: String, args_sha256: String, turn: u32}`; `ToolResultPayload {run_id, task_id, tool_call_id, outcome:
   ToolOutcome, code: Option<String>, bytes: Option<u64>, sha256: Option<String>}`; `ToolOutcome` snake_case enum
   `ok|refused|denied|timeout|cancelled` with `ALL`/`as_str`/`from_str` (mirrors `AgentState` style). Rejected:
   free-string outcome (loses schema refusal); embedding result text (size/redaction policy is a later wave).
3. **`code` is a `String` on the wire** (like existing `RUN_STATUS.error_code` / `ERROR.code`), not the `Refusal`
   Rust enum, because `Refusal` has no `JsonSchema` derive and the schema should not freeze the full code list into
   every payload. Rejected: adding `JsonSchema` to `Refusal` (contract-wide blast radius, unnecessary for B1).
4. **ApprovalRequest extension is optional-only** (`Option<...>` + `#[serde(default, skip_serializing_if =
   "Option::is_none")]`): `nonce`, `run_id`, `task_id`, `tool_call_id`, `args_sha256`, `expires_at: Option<Timestamp>`.
   Old frames remain valid; no existing field changes. Rejected: required fields (breaks old frames/fixtures) and a
   separate `APPROVAL_REQUEST_V2` event (new vocabulary without need).
5. **No new command, no new refusal code** in this patch (W10 §4.4 allows optional `E_TOOL_LOOP_LIMIT`; not requested
   here, and loop limits are B5). Commands/refusal counts stay 17/65. Rejected: adding `E_TOOL_LOOP_LIMIT` early
   (unused code, UI parity churn).
6. **Golden snapshot regenerated through the existing `UPDATE_GOLDEN=1` test path**, never hand-edited. Rejected:
   manual JSON edit (drift risk).
7. **UI parity**: `EventPayloads`/`EVENT_NAMES`/`DecodedEvent`/`decodePayload` all gain the two events; the two
   decoders require the identity/outcome fields and read optionals null-safe; `ApprovalRequestPayload` gains the
   optional fields and its decoder preserves them. Rejected: decode-to-unknown (violates typed-client rule) or
   treating TOOL_* as ERROR (hides activity).

## File-by-file change

1. `contract/src/event.rs` — append `EventName::ToolCall`/`ToolResult` (+docs), extend `ALL`, `as_str`;
   add `ToolOutcome` enum + `ToolCallPayload`/`ToolResultPayload` structs; extend `ApprovalRequest` with the six
   optional fields; add `Event` variants; wire `name()`, `from_envelope()`, `payload_value()`; update the in-file
   frozen-name unit test to 23 names.
2. `contract/src/schema.rs` — add `event_payload_schema` arms for ToolCall/ToolResult (map to new payload types).
3. `contract/tests/common/mod.rs` — add one `TOOL_CALL` and one `TOOL_RESULT` sample to `event_samples()` (result uses
   `outcome=refused`, `code=Some(E_TOOL_NOT_ALLOWED)`, bytes/sha); extend the `ApprovalRequest` sample with all new
   fields so the derived schema validates the extended shape.
4. `contract/tests/schema_roundtrip.rs` — event count assertions 21→23 (both places).
5. `contract/schemas/contract-v1.json` — regenerated via `UPDATE_GOLDEN=1` (23 event schemas; commands stay 14).
6. `web/src/api/contract.ts` — add `ToolCallPayload`/`ToolResultPayload` + `ToolOutcome` type; extend
   `ApprovalRequestPayload`; add `TOOL_CALL`/`TOOL_RESULT` to `EventPayloads`, `EVENT_NAMES`, `DecodedEvent`;
   add `decodePayload` cases (required ids/tool/turn/args_sha256/outcome; optional code/bytes/sha256; optional
   approval fields preserved null-safe).
7. **Not edited:** `command.rs`/`refusal.rs` (no new names/codes in this patch), `web/src/state/store.ts` and other
   web files, core/provider/tools, docs (Lead/doc owner), `web/tests/**` (blocked-on-scope, see below).
8. `plans/waves/W11-CONTRACT.md` — this report (allowed deliverable).

## Failure modes

- Old `APPROVAL_REQUEST` frame without new fields: decodes (all optional); UI shows existing nonce-gap behaviour.
- Malformed tool payload (missing id, wrong outcome, non-integer turn): `E_SCHEMA_INVALID` from jsonschema; never
  decoded into a partial tool event.
- Unknown future event name: existing `E_SCHEMA_INVALID`/`E_COMMAND_UNKNOWN` path unchanged.
- Golden drift: normal test fails with the explicit message; only UPDATE_GOLDEN regenerates.
- UI parity drift: web `satisfies readonly EventName[]` + count test catches it (web test update blocked on scope).
- No runtime change: tool events are never produced by this patch; no execution, no approval auto-resolution.

## Tests + commands (exact)

```powershell
$env:PATH="$env:USERPROFILE\.cargo\bin;C:\Users\Riley\Desktop\hiverouter-v1\tools\w64devkit\bin;"+$env:PATH
$env:CARGO_PROFILE_DEV_DEBUG='0'; $env:CARGO_INCREMENTAL='0'
cd backend
$env:UPDATE_GOLDEN='1'; cargo test -p hiverouter-contract golden_schemas --locked   # regenerate snapshot
Remove-Item Env:UPDATE_GOLDEN
cargo test -p hiverouter-contract --locked                                          # 32+ tests incl. matrix/roundtrip/golden
cd ..\web; npm.cmd run typecheck; npm.cmd test -- --run tests/contract.test.ts      # UI parity (blocked: hardcoded 21)
```

## Rollback / out-of-scope

- Rollback: revert the contract patch commit(s) + restore previous `schemas/contract-v1.json` (versioned file) + revert
  `web/src/api/contract.ts`; no DB/migration/runtime change, no release rebuild.
- Out of scope: runtime tool loop (W10 B2/B4/B5/B7), persistence (B3), approval UX/store (B6), Tauri (B8), command or
  refusal additions, docs/contracts UI-COMMANDS update (Lead/doc owner), `web/tests/**` (scope conflict), no commit.
- **Blocked-on-scope:** `web/tests/contract.test.ts:51` asserts 21 events; updating it needs Lead approval because the
  assigned web write scope is `contract.ts` ONLY. Pending Lead reply, the change is not made and the web parity test
  fails by exactly that assertion.

## Done / Not done / Unverified

### OLD→NEW (counts, explicitly)

| Surface | OLD | NEW | Note |
|---|---:|---:|---|
| Backend `EventName` / `EventName::ALL` | 21 | **23** | `+TOOL_CALL, +TOOL_RESULT` appended after `PONG` |
| Golden schema `events` | 21 | **23** | regenerated via `UPDATE_GOLDEN=1`; `commands` stays 14 (frozen) |
| `APPROVAL_REQUEST` payload fields | 3 (`approval_id`, `tool`, `decision?`) | **9** | adds `nonce?`, `run_id?`, `task_id?`, `tool_call_id?`, `args_sha256?`, `expires_at?` (all optional, old frames valid) |
| UI `EVENT_NAMES` / `EventPayloads` / `DecodedEvent` | 21 | **23** | `TOOL_CALL`, `TOOL_RESULT`; typecheck green |
| Commands | 17 (14 frozen + 3 gated) | **17** | unchanged; `command.rs` not edited |
| Refusals | 65 | **65** | unchanged; `refusal.rs` not edited |
| Agent states | 16 | **16** | unchanged |
| Web contract test count assertion | `21` | **23** | Lead-approved single-file edit (`web/tests/contract.test.ts`) |

### Done (all verified by run)

- `backend/crates/hiverouter-contract/src/event.rs` — `EventName::ToolCall/ToolResult` (+ALL/as_str), `ToolOutcome`
  (`ok|refused|denied|timeout|cancelled`, `as_str`/`from_str`), `ToolCallPayload`, `ToolResultPayload`, extended
  `ApprovalRequest` (6 optional fields), `Event` variants + `name`/`from_envelope`/`payload_value`, frozen-name unit
  test 23.
- `backend/crates/hiverouter-contract/src/schema.rs` — derived-schema arms for the two new events.
- `tests/common/mod.rs` — TOOL_CALL + TOOL_RESULT samples, extended APPROVAL_REQUEST sample (roundtrip/schema covered).
- `tests/schema_roundtrip.rs` — event-count assertions 21→23 (both); `tests/golden_schemas.rs` — negative
  `TOOL_RESULT outcome:"exploded"` schema refusal.
- `schemas/contract-v1.json` — regenerated (23 events; APPROVAL_REQUEST has 9 properties; ToolOutcome ref).
- `web/src/api/contract.ts` — `TOOL_OUTCOMES`/`ToolOutcome`/`isToolOutcome`, tool payload interfaces, extended
  `ApprovalRequestPayload`, `EventPayloads`/`EVENT_NAMES`/`DecodedEvent` + typed `decodePayload` cases (identity/turn/
  outcome required; optionals null-safe; legacy approval frames still decode).
- `web/tests/contract.test.ts` (Lead-approved scope extension) — 21→23 with OLD→NEW comment; decoder tests for
  TOOL_CALL/TOOL_RESULT (incl. unknown outcome refused) and extended/legacy APPROVAL_REQUEST.

### Evidence (commands + observed)

```
$ $env:UPDATE_GOLDEN='1'; cargo test -p hiverouter-contract golden_schemas --locked
test result: ok. 1 passed; 0 failed; ...            # snapshot regenerated, exit 0
$ cargo test -p hiverouter-contract --locked
running 18 tests ... test result: ok. 18 passed; 0 failed
running 3 tests  ... test result: ok. 3 passed; 0 failed   # golden
running 6 tests  ... test result: ok. 6 passed; 0 failed   # roundtrip (23 events)
running 5 tests  ... test result: ok. 5 passed; 0 failed   # matrix
CONTRACT_TEST_EXIT=0
$ cargo check -p hiverouter-core --tests --locked
Finished `dev` profile ... (1m 17s)                         # downstream consumer still compiles
CORE_CHECK_EXIT=0
$ cd web; npm.cmd run typecheck
> tsc --noEmit
WEB_TYPECHECK_EXIT=0
$ cd web; npm.cmd test -- --run
Test Files  15 passed (15) / Tests  95 passed (95)          # includes tests/refusalParity.test.ts (2)
WEB_TEST_EXIT=0
$ golden snapshot: contract=hiverouter.v1 commands=14 events=23
$ APPROVAL_REQUEST properties = approval_id, args_sha256, decision, expires_at, nonce, run_id, task_id, tool, tool_call_id
```

Deliverable hashes (sha256, lowercase): `event.rs d9d5a40b…fb53`, `schema.rs bc6ab685…b6a5`,
`tests/common/mod.rs 087baaa5…9dbb`, `tests/schema_roundtrip.rs 6a01458a…36ce`,
`tests/golden_schemas.rs d406302f…9383`, `schemas/contract-v1.json 07d45c69…a60c`,
`web/src/api/contract.ts a6989dac…fbc0`, `web/tests/contract.test.ts c137471a…9012`, this report.

### Not done

- No runtime tool execution, assembler, approval producer, store/UI activity wiring, persistence, or loop caps (W10
  B2–B7; other waves). No new command/refusal code. No docs update (`docs/contracts/UI-COMMANDS-v1.md` is outside
  this seat's write scope; Lead/doc owner must record the 21→23 and approval-field extension in the frozen doc and
  CHANGE-LEDGER before 1.2 release). No commit (Lead-owned). No release rebuild (1.1 tag untouched).
- `web/src/state/store.ts` not edited (outside scope): the two new events decode at the transport boundary but the
  store currently ignores them (no tool-activity UI yet; that is the separate UI activity wave).

### Unverified / residual

- The wire shape is validated only by schema/roundtrip/decoder tests — no producer exists, so no live `TOOL_CALL`/
  `TOOL_RESULT` frame has been observed end-to-end; UNVERIFIED until adapter/engine wave.
- `args_sha256` format is documented as lowercase hex but not schema-pattern-enforced (matching the existing `sha256`
  fields); a future patch could add a regex pattern.
- UI store/refusal-parity for tool errors not exercised (decoder-only); the UI activity wave must add the refusal
  rendering test.
- `expires_at` uses the contract `Timestamp` (RFC3339) and is optional; no TTL semantics are implemented here.
- The upstream `docs/contracts/UI-COMMANDS-v1.md` still lists 21 events/3 approval fields until the doc owner updates
  it (out of scope); parity between docs and code is intentionally flagged for the Lead.

