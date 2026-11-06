# W11 — TOOL-CALL ASSEMBLER (provider layer, no execution)

**Seat:** B (Tool-call assembler, provider layer) · **Write scope:** `backend/crates/hiverouter-provider/src/**`
and new provider tests only (no core/contract/web edits; no commit). **Repo:** `C:\Users\Riley\Desktop\hiverouter-v1`
· **HEAD observed:** `9651722` ("docs: record sealed 1.1.0 hashes and live streaming/replay proof").
**Skills used:** `/forgewright-recon` (source recon), `/logic-gatekeeper` (Phase 1 JSON + fail-closed rules),
`/integration-logic` (SSE pump → StreamEvent → assembler boundary), `/logic-debugger` (RED→GREEN test loop),
`/forgewright-lead` (acceptance gate, quoted evidence).

**Non-goals (explicit, from the seat mandate):** no tool execution, no approval flow, no contract events, no
core/web changes, no model loop. The assembler only turns validated streamed fragments into typed calls; the
future core adapter decides whether to execute them (W10 §4).

## 1. Recon (verified reads; current-tree anchors)

| Source | Key facts |
|---|---|
| `provider/src/client.rs` (29,214 B, 748 lines, sha `f46b6f10…5b62d3`) | `StreamEvent::ToolCallDelta { index: u32, id: Option<String>, name: Option<String>, arguments: Option<String> }` at 116-126, doc comment: "assembled by the caller". `pump_stream` 569-668 forwards one `delta_event` per SSE choice delta; `delta_event` 670-702 returns the **first** tool-call entry of a delta only (multi-entry tool_calls in a single chunk beyond the first are not emitted — existing limitation, unchanged by this seat). `[DONE]` is required; missing terminator ⇒ `E_PROVIDER_INCOMPLETE`. |
| `provider/src/dto.rs` (24,723 B, sha `aa6443f9…26c8a7`) | Wire types: `ToolCall` 269-277 (`id`, `type=function`, `function{name,arguments:String}`), `FunctionCall.arguments` is a **JSON-encoded string** (documented), `DeltaToolCall` 598-613 (`index`, first-chunk `id`/`type`, `function{name?,arguments?}`), `DeltaFunction` 616-624. |
| `provider/src/lib.rs` (2,408 B, sha `83343c8a…644e6f7`) | Module map + re-exports; `#![warn(missing_docs)]`, `#![forbid(unsafe_code)]`. New module must be registered and its public types re-exported with docs. |
| `provider/tests/{streaming,refusal_matrix,error_matrix}.rs` | Existing suites cover SSE/deltas/refusals; none assembles tool calls. Wiremock patterns and offline `sk-wiremock-offline-only` fixture convention reused. |
| `plans/waves/W10-TOOL-LOOP-DECISION.md` §4.2/§6 (lines 125-129, 212-213) | Design intent: `ToolCallAssembler` index-keyed, `id` first fragment, `name` first fragment, `arguments` concatenated in order; missing identity ⇒ `E_PROVIDER_INVALID_RESPONSE`; invalid JSON argv ⇒ `E_TOOL_SCHEMA`; wiremock multi-turn later. This seat implements exactly that assembler, without the loop. |
| `contract/src/refusal.rs` | `Refusal::ToolSchema` = `E_TOOL_SCHEMA` (non-retryable, "fix the arguments"); `Refusal::ProviderInvalidResponse` = `E_PROVIDER_INVALID_RESPONSE` (non-retryable). Both frozen; the assembler maps its typed errors to these, adding no codes. |

Current gap: the client comment says fragments are "assembled by the caller", and no caller or helper exists
(repo grep: only comments, no `Assembler` symbol). Therefore a future loop has no deterministic way to build a
complete call, and nothing rejects truncated/malformed/duplicate-id calls before execution.

## 2. Phase 1 (logic-gatekeeper — accepted before any write)

```json
{
  "phase1": {
    "architecture": "New pure provider module `tool_assembler` holds per-message assembly state (BTreeMap<u32, PendingCall>) and is the only owner of that state. It consumes StreamEvent::ToolCallDelta values produced by the existing client and exposes typed AssembledToolCall values. It performs no I/O, no tool resolution, no approval and no execution; it is a fail-closed validation state machine at the provider boundary. The existing client/transport is unchanged.",
    "dataFlow": "Untrusted provider bytes -> pump_stream (SSE parse, [DONE] required) -> delta_event -> StreamEvent::ToolCallDelta{index,id?,name?,arguments?} -> ToolCallAssembler::push_event/push_fragment (validate identifiers, accumulate arguments per index, reject conflicting identity, bound call count/size) -> finish() (require complete identity, JSON-object arguments, unique ids) -> Vec<AssembledToolCall> sorted by index. A future core adapter is the only consumer that may execute; this module never executes anything.",
    "edgeCases": [
      "two indices interleaved: index 0 fragment, index 1 id/name/args, then index 0 args remainder -> two calls with per-index order preserved and output sorted by index",
      "id repeated identically on a later chunk (docs say first chunk only) -> tolerated idempotently; a different id for the same index -> ConflictingId",
      "two different indices resolving to the same tool_call id -> DuplicateId (rejected, never merged)",
      "stream ends after an id but no name, or no arguments fragments at all -> MissingName/MissingArguments (truncated call)",
      "arguments string is empty, not JSON, or a JSON scalar/array instead of an object -> InvalidArgumentsJson/ArgumentsNotAnObject mapped to E_TOOL_SCHEMA",
      "identifier empty, containing control characters, or longer than 128 bytes -> InvalidIdentifier/IdentifierTooLong mapped to E_PROVIDER_INVALID_RESPONSE",
      "a ninth distinct index in one message -> TooManyCalls (bound, deterministic)",
      "arguments accumulate beyond the 1 MiB bound -> ArgumentsTooLarge instead of unbounded memory growth",
      "finish() with no fragments at all -> Ok(empty vec) so a text-only reply is not an error",
      "non-tool StreamEvents passed to push_event -> Ok(false), no state change"
    ],
    "rootCause": "The provider client emits tool-call fragments that its own comment says are 'assembled by the caller', but no caller/helper exists, so any future tool loop would have to invent assembly and could silently mis-order argument fragments, accept truncated calls, merge duplicate ids, or execute arguments that are not a JSON object; the provider layer must own one deterministic, fail-closed assembler before an execution path is allowed to exist."
  }
}
```

Phase-1 gate: all fields non-empty; concrete inputs/outputs; rootCause names the contradiction. **Accepted.**

## 3. Contract of the assembler (as implemented)

- `ToolCallAssembler::new()` / `Default`; `push_fragment(index, id, name, arguments)`; `push_event(&StreamEvent)`
  (returns `Ok(true)` when it consumed a tool-call fragment, `Ok(false)` for other events); `finish()` (consumes
  the assembler and returns calls sorted by `index`); `pending_len()`.
- Per index: `id`/`name` may appear once (identical repeats are idempotent; conflicts rejected); `arguments`
  fragments are concatenated **in arrival order**; no tool is resolved or run.
- Bounds: `MAX_TOOL_CALLS_PER_MESSAGE = 8`, `MAX_IDENTIFIER_BYTES = 128`, `MAX_ARGUMENTS_BYTES = 1 MiB`.
- Typed error `ToolCallAssemblyError` with failure-specific variants and `refusal()` mapping:
  structural/truncation/identity errors → `E_PROVIDER_INVALID_RESPONSE`; argument JSON/object errors → `E_TOOL_SCHEMA`.
- `AssembledToolCall { index: u32, id: String, name: String, arguments: serde_json::Value }` — the arguments
  value is a JSON **object**; the type carries no capability to execute.
- No contract event, command, refusal code or core type is added or changed.

## 4. Done (executed evidence — Definition of Done §22: run, compared, quoted)

**Mandated command and observed output** (from `backend/`, `PATH` = cargo bin + `tools/w64devkit/bin`,
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_INCREMENTAL=0`; `--locked`; no key, network or paid call):

```text
> cargo test -p hiverouter-provider --locked
     Running unittests src\lib.rs      running 45 tests   test result: ok. 45 passed; 0 failed; 0 ignored
     Running tests\error_matrix.rs     running 9 tests    test result: ok. 9 passed; 0 failed; 0 ignored
     Running tests\refusal_matrix.rs   running 11 tests   test result: ok. 11 passed; 0 failed; 0 ignored
     Running tests\streaming.rs        running 8 tests    test result: ok. 8 passed; 0 failed; 0 ignored
     Running tests\tool_assembler.rs   running 3 tests    test result: ok. 3 passed; 0 failed; 0 ignored
     (doctests)                        0 tests
=== EXIT: 0 ===
```

Targeted evidence during development: `cargo test -p hiverouter-provider --lib --locked` ⇒ `running 45 tests`
⇒ `test result: ok. 45 passed`; `cargo test -p hiverouter-provider --test tool_assembler --locked` ⇒
`running 3 tests` ⇒ `test result: ok. 3 passed` (names: `wiremock_stream_assembles_two_calls_in_index_order`,
`wiremock_stream_missing_id_is_provider_invalid_response`,
`wiremock_stream_invalid_arguments_maps_to_tool_schema`).

**RED → GREEN (logic-debugger):** the first lib run failed one *test expectation*: 
`oversized_arguments_are_refused_before_unbounded_growth` supplied exactly `MAX_ARGUMENTS_BYTES` (allowed by
the bound; refusal starts at MAX+1), so `push_fragment` correctly returned `Ok` and `expect_err` panicked.
Root cause was the test's off-by-one, not the bound; the test now supplies `MAX_ARGUMENTS_BYTES + 1` and the
suite is 45/45 green (quoted above). No production behavior changed to make a test pass.

**What was delivered (provider layer only):**

1. **`src/tool_assembler.rs` (new, 25,928 B, 673 lines, sha256
   `df3b79565e8c61ec8b5c1fda8ce0346da2d50b979e604e96c2f3b84d54230c05`)** — `ToolCallAssembler` (BTreeMap by
   index), `AssembledToolCall { index, id, name, arguments: serde_json::Value }`, typed
   `ToolCallAssemblyError` (12 variants) with `refusal()`/`code()` mapping to the **existing** frozen codes and
   `Display`/`Error` that never echo argument content. Rules implemented: per-index merge of
   `id`/`name`/`arguments`; arguments concatenated **in arrival order**; identical repeated id/name idempotent,
   conflicting values refused; **duplicate ids across indices refused**; truncation (missing id/name/arguments)
   refused at `finish()`; concatenated arguments must be a **JSON object** (else `E_TOOL_SCHEMA`); bounds
   `MAX_TOOL_CALLS_PER_MESSAGE = 8`, `MAX_IDENTIFIER_BYTES = 128`, `MAX_ARGUMENTS_BYTES = 1 MiB`; output sorted
   by index; empty assembler finishes cleanly (text-only reply). `push_event(&StreamEvent)` consumes fragments
   and ignores `Delta`/`Finished`. **No execution, registry, approval or contract-event code.**
2. **`src/lib.rs` (modified, +7 lines only)** — registers `pub mod tool_assembler`, documents it in the module
   map, re-exports the types/bounds. Diff is limited to those 7 insertion lines, 0 deletions (verified with
   `git diff --stat`: `1 file changed, 7 insertions(+)`).
3. **`tests/tool_assembler.rs` (new, 6,617 B, 163 lines, sha256
   `ce2001d1042653e546f905be93b31ffee4ec8547a9a2aa32f07d23572cc82e41`)** — 3 wiremock integration tests that
   drive the real `DeepSeekClient::stream` SSE path and feed the resulting `StreamEvent`s to the assembler:
   two calls assembled across interleaved fragments (index 1 before index 0; index 0 arguments split over two
   chunks) with sorted output and parsed JSON objects; truncated call (no id) ⇒ `E_PROVIDER_INVALID_RESPONSE`;
   invalid argument JSON ⇒ `E_TOOL_SCHEMA`.
4. **`plans/waves/W11-TOOL-ASSEMBLER.md`** (this record).

**Scope evidence:** `git status --short -- backend/crates/hiverouter-provider plans/waves` shows only
`M backend/crates/hiverouter-provider/src/lib.rs`, `?? .../src/tool_assembler.rs`, `?? .../tests/tool_assembler.rs`,
`?? plans/waves/W11-TOOL-ASSEMBLER.md` for this seat. The other shown changes (`contract/*`, `web/*`, other
`W11-*.md` files) belong to concurrent seats and were **not touched** by this seat. HEAD `9651722`; no commit.

**Stub sweep (§23)** over the three code/doc files: hits are only `wiremock`/`MockServer`/`Mock` (permitted
HTTP test double, §7) and the word "mock" inside test/docs prose; no TODO/FIXME body, no fabricated logic, no
function returning fake success. Classified: permitted; zero defects.
**Skills gate:** `/forgewright-recon` (§1 anchors), `/logic-gatekeeper` (Phase 1 accepted in §2 before the write;
throwing/refusing validators at the boundary), `/integration-logic` (SSE pump → `StreamEvent` → assembler
contract exercised by wiremock tests), `/logic-debugger` (RED→GREEN above), `/forgewright-lead` (§22 evidence
quoted and compared, not "no error").

## 5. Not done

1. **Client transport improvement for >1 tool call in a single SSE `delta`**: `client.rs::delta_event`
   currently emits only the **first** `delta.tool_calls` entry per chunk (existing behavior, 670-702). The
   assembler is index-keyed and ready, but a provider that puts two calls in one delta would lose the second
   until `pump_stream`/`StreamEvent` is changed. This seat's mandate was the assembler only; the client was
   deliberately left unchanged. Follow-up: emit one `StreamEvent::ToolCallDelta` per entry in W11/next wave.
2. **No core loop, execution, approval, registry or persistence wiring** — explicit non-goal of this seat
   ("no product execution path yet"); the core adapter still discards `ToolCallDelta` (W10 §2, unchanged).
3. **No contract events/commands** — explicit non-goal; the assembler maps to existing frozen refusals only.
4. **No fuzz/property tests** — deterministic enumerated cases only (16 unit + 3 wiremock); a `proptest` state
   machine is a reasonable later addition, not part of the mandate.
5. **No commit** — Lead-owned.

## 6. Unverified

- **Real-provider fragmentation/ordering**: the assembler's merge rules are proven against documented SSE
  shapes replayed by wiremock, not against a live DeepSeek stream (no key, no paid call). In particular, whether
  the live provider always sends `id`/`name` on the first fragment (as documented) and never repeats a
  *different* value is UNVERIFIED; the assembler tolerates identical repeats and fails closed on conflicts.
- **Multi-tool single-chunk behavior** is not exercised end-to-end because the client drops all but the first
  entry (see §5); the assembler unit tests cover multiple indices arriving via separate chunks only.
- **Reasoning-content pass-back / thinking-mode tool rules** (docs: HTTP 400 unless `reasoning_content` is
  passed back when tools are present with thinking enabled) are a future loop concern, not exercised here.
- **Consumer integration**: no core/UI consumer exists yet, so the assembler's public API has not been exercised
  by a downstream caller (only unit + wiremock tests).
- What would prove the remainder: the W10 Phase-A loop wiremock multi-turn test (assembler + fake tool) once
  the core adapter is allowed to consume it, and an operator-key-gated live smoke for real fragmentation.
