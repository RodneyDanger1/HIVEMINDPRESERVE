# W3 — TRUE STREAMING plan (recon → plan, Forge protocol)

**Seat:** W3 (write-authorized: `backend/crates/hiverouter-core/**`, `backend/crates/hiverouter-provider/**` only)
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **Planning HEAD:** `0dfe23b` (clean tree at plan time)
**Skills:** `/forge` (root-cause repair, RED→GREEN) + `/forgewright-lead` (review gate, evidence or it did not happen).

## 1. Root cause (proven from source, before any edit)

`ProviderExecutor::execute` has no delta sink (`core/src/execution.rs:49-55`), so
`ProviderExecutorAdapter::execute` performs a **non-streaming** `complete_with_budget` call and returns
`StreamedReply { deltas: vec![full_text], usage }` after completion (`core/src/provider_executor.rs:129-160`).
`RunEngine::execute_task` then publishes every `reply.deltas` entry as a `REPLY_DELTA` **after** `execute`
returns (`core/src/execution.rs:599-611`) — i.e. the UI receives one burst after the model has finished, not
real SSE chunks. The provider crate already has the documented streaming transport
(`provider/src/client.rs:474-507 stream()`, `StreamEvent::Delta/Finished` at `:103-145`, SSE parser and pump at
`:164-186,527-627`) and its own wiremock test suite, but **streaming is not wired into the run path and is not
budgeted** (`stream()` takes no `RunBudget`; no preflight/accounting). Root cause: **adapter seam uses the
non-streaming client method and the engine has no per-chunk publish path.**

RED representation: trait/API has no sink (the new adapter test cannot compile against old API) and the old
behaviour is pinned by the single-delta `provider_executor.rs:156-159`. New tests below assert the required
behaviour and fail before the change.

## 2. Outcome

`RunEngine` publishes ordered `REPLY_DELTA` events **while** the provider SSE chunks arrive, through a
`Arc<dyn Fn(&str) + Send + Sync>` delta sink passed to `ProviderExecutor::execute`; the persisted assistant text
still comes from `StreamedReply::text()`; no duplicate emission (engine publishes returned deltas only for
executors that never invoked the sink). Streaming calls go through the existing documented client and the
existing per-run `RunBudget` (preflight before send, usage recorded after `Finished`). Usage absent after a
stream returns `E_USAGE_INCOMPLETE`, never 0. No retry after the first delta. Planner stays non-streaming JSON.

## 3. Current behavior (evidence)

- `execution.rs:49-55` — `execute` returns `StreamedReply` only; no sink.
- `execution.rs:599-611` — deltas published after `execute` returns (single burst).
- `provider_executor.rs:129-160` — non-streaming `complete_with_budget`, `deltas: vec![text]`.
- `client.rs:474-507` — `stream()` exists (SSE, retries only before first byte) but no budget argument;
  module doc says streaming usage "NOT documented" (`client.rs:11-13`) — stale vs official docs (usage rides the
  final chunk before `[DONE]`; already aggregated in `pump_stream` `:578-586,621-626`).
- `budget.rs:235-298` — `preflight`/`record_attempt`/`record_call` exist; `RunBudget` is the single cost owner.
- `execution.rs:476` — engine already evicts the per-run budget after terminal via `finish_run` (adapter `:162-164`).

## 4. Decisions (rejected alternatives)

1. **Callback sink on the trait** (`Arc<dyn Fn(&str)+Send+Sync>`), not an mpsc channel: engine stays the single
   event publisher, ordering is the provider loop order, no extra task/queue. Rejected channel: adds lifecycle
   and ordering risk for no benefit.
2. **Both invocation modes remain legal** (sink for streaming executors, returned `deltas` for non-streaming/test
   executors) with a "sink used" counter so the engine never double-publishes. Rejected "always ignore returned
   deltas": breaks the existing non-streaming mocks and would drop text for executors that cannot stream.
3. **Budget ownership in the adapter** (`Mutex<HashMap<RunId, RunBudget>>`, already present) — preflight before
   opening the stream, record usage after `Finished`. Rejected: engine-side budget (would duplicate the money owner
   and cross the crate boundary the wrong way).
4. **Usage**: only the documented final-chunk usage is accepted; absent/partial ⇒ `record_call(None)` ⇒
   `E_USAGE_INCOMPLETE`; never fabricate 0. Rejected: defaulting `Usage::default()` as success.
5. **No retry after first delta**: client retries only failed *opens* (already true); adapter never re-calls after
   any delta. Rejected: transparent stream replay (duplicates user-visible text and spend).
6. **Planner stays `complete()`** (non-streaming JSON plan); only leaf/task replies stream. Rejected: streaming the
   planner (plan needs the whole JSON; no UX benefit, more failure modes).

## 5. File-by-file change

1. `backend/crates/hiverouter-provider/src/client.rs`
   - Add `stream_with_budget(&self, budget: &mut RunBudget, request: &ChatRequest, estimate: &CallEstimate)`
     → `Result<StreamHandle, ProviderError>`: `request.validate()`, in-flight permit (propagate error, not `.ok()`),
     per-attempt `budget.preflight(estimate)?` + `budget.record_attempt()`, `open_stream` with the existing retry
     policy, `budget.on_retry()` on retries, `final_error` on exhaustion, then spawn `pump_stream`.
   - Refactor `stream()` to delegate to a shared `stream_inner(None, …)` (budgeted path passes `Some(budget)`).
   - Fix the stale module comment: streaming usage is documented on the final chunk; absent ⇒ INCOMPLETE.
   - Caller records usage via `RunBudget::record_call` after `StreamEvent::Finished`.
2. `backend/crates/hiverouter-core/src/execution.rs`
   - Add `pub type DeltaSink = Arc<dyn Fn(&str) + Send + Sync>;`
   - `ProviderExecutor::execute(..., on_delta: &DeltaSink)`; update `UnconfiguredExecutor`.
   - `execute_task`: build a sink that publishes `REPLY_DELTA` immediately (empty fragments skipped), count
     invocations; pass `&sink`; after success publish `REPLY_DELTA` from `reply.deltas` **only when the sink was
     never used**; persist `reply.text()` and publish `REPLY_DONE` as today.
   - Update trait/`StreamedReply` docs to state the two legal modes and no-duplicate rule.
3. `backend/crates/hiverouter-core/src/provider_executor.rs`
   - `execute`: build the same request (leaf thinking/max_tokens/response format unchanged), then drive
     `client.stream_with_budget(budget, …)`:
     - select cancellation; `Delta{content}` non-empty → append text + `on_delta(&content)`;
     - `ToolCallDelta` → remember (leaf tool calls are unsupported → named `E_PROVIDER_INVALID_RESPONSE` if no text);
     - `Finished{usage, finish_reason}` → `budget.record_call(model, usage.as_ref(), now)`; absent/partial usage
       ⇒ `E_USAGE_INCOMPLETE`; empty text ⇒ `E_PROVIDER_INVALID_RESPONSE` with finish_reason logged;
     - `Err(e)` after deltas ⇒ mark accounting incomplete best-effort and return the named refusal; no retry.
   - Keep `plan` on `complete_with_budget` (non-streaming JSON); keep `finish_run` eviction.
4. `backend/crates/hiverouter-core/src/lib.rs` — re-export `DeltaSink`.
5. `backend/crates/hiverouter-provider/tests/streaming.rs` (new) — wiremock SSE tests: ordered deltas + `[DONE]` +
   usage; malformed SSE; missing `[DONE]` ⇒ `E_PROVIDER_INCOMPLETE`; 429-before-first-byte retried then streams;
   budget preflight refusal with **zero** requests; no retry after first delta; absent usage ⇒ `E_USAGE_INCOMPLETE`.
6. `backend/crates/hiverouter-core/tests/provider_streaming.rs` (new) — wiremock adapter tests: sink receives chunks
   in order, persisted `reply.text()` equals concatenation, usage measured; usage absent ⇒ `E_USAGE_INCOMPLETE`;
   cancellation before first byte ⇒ `E_TOOL_CANCELLED`; malformed SSE ⇒ named refusal; error-before-first-byte
   retry observed (2 server requests) then streams.
7. `backend/crates/hiverouter-core/tests/run_lifecycle.rs` — update `MockExecutor::execute` signature; call the sink
   for its deltas; add `streamed_deltas_are_published_before_reply_done_and_not_duplicated` (event order + exact
   counts + persisted text).
8. `backend/crates/hiverouter-core/tests/provider_live.rs` — update the ignored live test to pass a recording sink;
   assert at least one streamed delta arrived.

## 6. Failure modes

- Cancel mid-stream: select drops the handle → pump task exits; run/task cancelled by the engine path (unchanged).
- Provider error after deltas: named refusal, no retry, no fabricated `REPLY_DONE`; accounting best-effort INCOMPLETE.
- Missing `[DONE]`: client emits `E_PROVIDER_INCOMPLETE`; adapter maps it; reply never marked complete.
- Usage absent/partial: `E_USAGE_INCOMPLETE`, never 0; further sends refused by the same budget.
- Budget/limit refusal before open: zero HTTP requests (wiremock assertion).
- Sink publishing failure (bus closed): logged; execute result unaffected (event loss is observable, not silent success).
- Empty content with tool calls: `E_PROVIDER_INVALID_RESPONSE` (leaf tool calls unsupported, unchanged).

## 7. Tests + commands

```powershell
$env:PATH="$env:USERPROFILE\.cargo\bin;C:\Users\Riley\Desktop\hiverouter-v1\tools\w64devkit\bin;"+$env:PATH
$env:CARGO_PROFILE_DEV_DEBUG='0'; $env:CARGO_INCREMENTAL='0'
cd C:\Users\Riley\Desktop\hiverouter-v1\backend
cargo test -p hiverouter-provider --test streaming          # new wiremock SSE matrix
cargo test -p hiverouter-core --test provider_streaming    # adapter streaming matrix
cargo test -p hiverouter-core --test run_lifecycle         # sink ordering/no-duplicate
cargo test --workspace --locked                            # required acceptance run
```

## 8. Rollback / out-of-scope

- Rollback: trait sink + engine sink publishing is additive; revert the two crates' files (no DB/contract change).
  Behavioural fallback for non-streaming executors remains, so reverting the adapter alone restores the old
  single-delta path without touching the engine.
- Out of scope (not touched): `web/**`, `e2e/**`, `tools/**`, `docs/**`, `audit/**`; contract/schema changes;
  retry-policy changes; planner streaming; provider accounting formula changes; UI rendering of streamed deltas.

## 9. Execution log (observed; Forge RED→GREEN + ForgeWright-lead self-review)

**RED (pre-change).** The production adapter used `complete_with_budget` and returned `deltas: vec![full_text]`
(`provider_executor.rs:129-160` before this change); the engine published them only after `execute` returned
(`execution.rs:599-611` before). The new tests target the sink/streaming API and cannot compile against the old trait
signature — the compile-level RED — and the old single-burst behaviour is exactly what the new adapter/engine tests
assert against.

**GREEN (observed, workspace HEAD `0dfe23b` + these edits):**

```
$ cargo test -p hiverouter-provider --test streaming --locked
test result: ok. 8 passed; 0 failed; 0 ignored
$ cargo test -p hiverouter-core --test provider_streaming --test run_lifecycle --locked
provider_streaming: 6 passed; 0 failed
run_lifecycle:     7 passed; 0 failed
$ cargo test --workspace --locked
TOTAL passed=261 failed=0 ignored=2        # ignored: kill_restore helper + provider_live (paid, opt-in)
WORKSPACE_TEST_EXIT=0
```

New coverage: provider SSE order/`[DONE]`/usage, malformed SSE, missing-`[DONE]`, retry-before-first-byte,
no-retry-after-delta, budget-preflight-zero-requests, budgeted usage settlement, absent-usage INCOMPLETE;
core adapter sink order/persistence/usage, absent-usage INCOMPLETE, malformed, missing-`[DONE]` no-retry,
retry-before-first-byte, cancellation; engine-level: REPLY_DELTA order before REPLY_DONE and no duplicate emission.

**Changed files (sha256):** `provider/src/client.rs f46b6f10…`, `provider/tests/streaming.rs 165e5c6b…`,
`core/src/execution.rs 9b055295…`, `core/src/provider_executor.rs 44b3ed02…`, `core/src/lib.rs ffe93e28…`,
`core/tests/run_lifecycle.rs 47592677…`, `core/tests/provider_live.rs d202e266…`,
`core/tests/provider_streaming.rs 6c0b7fbf…`. No other paths were written by this seat.

**Review notes / deviations:**
- The per-run budget is the existing adapter `Mutex<HashMap<RunId, RunBudget>>`; the guard is held for the duration of
  one stream so usage settles against the same budget. The engine enforces a single active run, so this cannot block a
  concurrent run's planner; `finish_run` still evicts the entry after the terminal status.
- Cancellation is selected inside the provider read loop; cancellation while the HTTP stream is still opening is
  handled by the engine's existing `select!` around `execute` (same as before) — adapter returns `E_TOOL_CANCELLED` once
  the open returns and the token is already cancelled (covered by the wiremock cancellation test).
- `rustfmt --check` is not clean on the repository baseline (many untouched files differ), so formatting was kept
  consistent with the surrounding code rather than reformatting unrelated files. Clippy exits 0; the one new
  `field_reassign_with_default` warning in the new test was fixed; repo-wide warnings are pre-existing.
- Stub sweep over the changed product files (`TODO|FIXME|XXX|HACK|placeholder|not implemented|stub|dummy|fake`)
  returned **zero hits**. No fake chunking: every `REPLY_DELTA` originates from a parsed `data:` SSE frame.
- Commit is reserved to the Lead; this seat leaves its own scope modified/untracked (no partial writes, no commits).

