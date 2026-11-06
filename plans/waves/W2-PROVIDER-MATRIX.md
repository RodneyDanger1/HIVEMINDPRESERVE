# W2 — PROVIDER ERROR MATRIX (deterministic offline tests)

**Seat:** W2 (Provider error matrix) · **Write scope:** new `backend/crates/hiverouter-provider/tests/error_matrix.rs`
+ this record only (no product edits; no commit). **Repo:** `C:\Users\Riley\Desktop\hiverouter-v1`
· **HEAD observed:** `c9390f5` ("1.1 wave 3"). **No key, no network, no paid call:** every test points the
client at a local `wiremock` server (or a closed loopback port) — the key is an obvious offline test fixture
(`sk-wiremock-offline-only`), never a real credential.
**Skills used:** `/forgewright-recon` (source recon), `/logic-gatekeeper` (Phase 1 + fail-closed gates),
`/integration-logic` (call-chain boundaries), `/logic-debugger` (failure semantics), `/forgewright-lead` (gate).

## 1. Recon (verified by reading source; no assumption inherited)

Read in full this session (metadata via `Get-FileHash`/`Get-Item`; quoted line anchors are current-tree):

| File | bytes | lines | sha256 | Relevant behavior |
|---|---:|---:|---|---|
| `provider/src/client.rs` | 29,214 | 748 | `f46b6f10…5b62d3` | `complete` 295; `drive` retry loop 317-372; `final_error` 373-388 (exhausted `ProviderRateLimit` ⇒ `LlmRateLimit` + status 429); `classify_status` 437-476 (401/402 ⇒ `ProviderAuth` non-retryable; 429 ⇒ `ProviderRateLimit` retryable + `Retry-After`); `stream`/`stream_inner` 479-548 (open-only retry loop); `open_stream` 550-565; `pump_stream` 569-668 (`[DONE]` or `E_PROVIDER_INCOMPLETE`; no re-open after open); `delta_event` 670-702; `map_transport_error` 710-731 (timeout/connect/body all ⇒ `ProviderTimeout`). |
| `provider/tests/refusal_matrix.rs` | 11,475 | 278 | `80389102…6767e6e` | Existing non-streaming matrix: 401/402 no retry; 429 + Retry-After ⇒ E_LLM_RATE_LIMIT after 3 attempts; timeout; malformed; NULL usage; budget. |
| `provider/tests/streaming.rs` | 11,843 | 336 | (read this session) | Existing streaming matrix: ordered deltas, `[DONE]`, malformed SSE, missing `[DONE]`, **error before first byte retried** (429→200), **not retried after a delta**, absent usage INCOMPLETE, budget preflight. |
| `provider/src/budget.rs` | 23,965 | — | — | `RetryPolicy` public fields; `MAX_ATTEMPTS = 3`; `delay_for` honors `Retry-After` capped at `RETRY_AFTER_CAP` (60 s); default backoff 250 ms base, 8 s cap, 250 ms jitter. |
| `provider/src/error.rs` | 4,483 | — | — | `ProviderError::{new,with_status,with_retry_after,refusal,code,detail,http_status,retry_after,retryable,log_line}`. |
| `provider/src/lib.rs` | 2,408 | — | — | Exports `RetryPolicy`, `MAX_ATTEMPTS`, `RETRY_AFTER_CAP`, `ClientConfig`, `StreamEvent`, `StreamHandle`, etc. |

Current retry contract (from the code above):
- `MAX_ATTEMPTS = 3` including the first try; retry only when `error.retryable()` **and** attempt < max.
- 401/402 ⇒ `Refusal::ProviderAuth`, never retried; 429 ⇒ `Refusal::ProviderRateLimit` retryable with
  `Retry-After`; exhausted retries ⇒ `final_error` maps to `Refusal::LlmRateLimit` (status 429).
- Transport failures (connect/timeout/body/decode) ⇒ `Refusal::ProviderTimeout` via `map_transport_error`
  (single frozen transport code, by design — adding codes would be a contract change).
- Streaming: retries happen **only while opening** (`stream_inner` 514-547); once `open_stream` succeeds,
  `pump_stream` errors are terminal for the caller (no re-open, no replay of deltas). Missing `[DONE]` ⇒
  `E_PROVIDER_INCOMPLETE`; malformed SSE ⇒ `E_PROVIDER_INVALID_RESPONSE`.

## 2. Phase 1 (logic-gatekeeper — written before creating the test file)

```json
{
  "phase1": {
    "architecture": "The integration point is DeepSeekClient in provider/src/client.rs: it owns request validation, the bounded retry loop (drive/stream_inner), status classification, transport-error mapping, and the SSE pump. The new file is a provider integration test (tests/error_matrix.rs) that drives that client against a local wiremock HTTP server or a closed loopback port. No production state or product file is touched; the test binary is isolated per test function.",
    "dataFlow": "test -> DeepSeekClient::complete/stream -> reqwest HTTP POST /chat/completions -> wiremock (or connection refused) -> classify_status / map_transport_error -> retry decision (retryable && attempt < MAX_ATTEMPTS, delay_for honoring Retry-After/backoff) -> final_error maps exhausted ProviderRateLimit to LlmRateLimit -> caller observes ProviderError{code,http_status,retryable,refusal,detail}. Streaming: open_stream retry loop -> pump_stream emits Delta/ToolCallDelta/Finished or terminal Err; a pump failure after open never re-opens.",
    "edgeCases": [
      "429 exhaustion at exactly MAX_ATTEMPTS=3 must be terminal LlmRateLimit with exactly 3 recorded HTTP requests (not 2, not 4)",
      "429 with Retry-After: 0 must retry without sleeping/hanging and still map to LlmRateLimit",
      "401 and 402 must be non-retryable with exactly 1 HTTP request, both for complete and for stream open",
      "request timeout must map to ProviderTimeout and be retried up to MAX_ATTEMPTS but never past it",
      "connect failure to a closed loopback port must map to ProviderTimeout (not panic/hang/fabricated success)",
      "stream open 429 before the first byte must be retried once, then stream successfully (attempts=2, 2 requests)",
      "stream 200 partial SSE without [DONE] after a delta must be E_PROVIDER_INCOMPLETE with exactly 1 request and no retry"
    ],
    "rootCause": "The retry/classification contract is implemented and partially tested, but there is no single deterministic error matrix that pins stream-vs-complete exhaustion mapping (ProviderRateLimit -> LlmRateLimit), transport boundaries (timeout and connect failure), and the retry-before-first-byte vs never-after-first-byte rule together; without those pins a future refactor of drive/stream_inner can silently change attempt counts, the final refusal code, or introduce a retry after a delivered delta."
  }
}
```

Phase 1 gate: all fields non-empty; edge cases are concrete and observable; rootCause names the contradiction
(contract implemented but not exhaustively pinned). **Accepted before creating the test file.**

## 3. Test design (deterministic; each case names the contradiction it kills)

| # | Test | Server behavior | Expected |
|---|---|---|---|
| 1 | `rate_limit_429_exhaustion_maps_provider_rate_limit_to_llm_rate_limit` | always 429 + `Retry-After: 0` | `E_LLM_RATE_LIMIT`, `http_status=429`, retryable, `refusal=LlmRateLimit`, exactly `MAX_ATTEMPTS` requests |
| 2 | `stream_rate_limit_429_exhaustion_maps_to_llm_rate_limit` | always 429 + `Retry-After: 0` | `stream()` ⇒ same mapping, exactly 3 requests (open retry loop exhausted) |
| 3 | `payment_required_402_is_not_retried` | always 402 | `E_PROVIDER_AUTH`, non-retryable, 1 request |
| 4 | `auth_401_is_not_retried` | always 401 | `E_PROVIDER_AUTH`, non-retryable, 1 request |
| 5 | `stream_open_401_is_not_retried` | always 401 | `stream()` ⇒ `E_PROVIDER_AUTH`, 1 request |
| 6 | `request_timeout_maps_to_provider_timeout_with_bounded_retries` | 200 delayed 2 s, client timeout 150 ms | `E_PROVIDER_TIMEOUT`, retryable, exactly 3 requests |
| 7 | `connect_failure_maps_to_provider_timeout` | closed loopback port (`http://127.0.0.1:1`) | `E_PROVIDER_TIMEOUT`, retryable, no panic/hang |
| 8 | `stream_error_before_first_byte_is_retried` | first 429 (`Retry-After: 0`), then 200 SSE | stream opens with `attempts=2`, deltas + `Finished`, exactly 2 requests |
| 9 | `stream_error_after_first_delta_is_not_retried` | 200 SSE with one delta, no `[DONE]` | delta observed, then `E_PROVIDER_INCOMPLETE`, exactly 1 request (no re-open) |

Determinism: `Retry-After: 0` makes retry delays zero; the timeout case uses a 2 s server delay vs a 150 ms
client timeout and asserts counts, never wall-clock; the connect case uses a guaranteed-closed loopback port
and the frozen `ProviderTimeout` mapping. No timing race is asserted, only named errors and HTTP request counts.

## 4. Done (executed evidence — Definition of Done §22: run, compared, quoted)

**Mandated command and observed output** (run from `backend/` with `%USERPROFILE%\.cargo\bin` and
`tools\w64devkit\bin` on `PATH`; `--locked`; no key, no network):

```text
> cargo test -p hiverouter-provider --test error_matrix --locked
     Running tests\error_matrix.rs
running 9 tests
test auth_401_is_not_retried ... ok
test payment_required_402_is_not_retried ... ok
test stream_error_after_first_delta_is_not_retried ... ok
test rate_limit_429_exhaustion_maps_provider_rate_limit_to_llm_rate_limit ... ok
test stream_error_before_first_byte_is_retried ... ok
test stream_open_401_is_not_retried ... ok
test stream_rate_limit_429_exhaustion_maps_to_llm_rate_limit ... ok
test request_timeout_maps_to_provider_timeout_with_bounded_retries ... ok
test connect_failure_maps_to_provider_timeout ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.96s
```

**Integration check (existing provider suites unchanged; run for the lead gate):**

```text
> cargo test -p hiverouter-provider --locked
     Running unittests src\lib.rs      running 29 tests   test result: ok. 29 passed; 0 failed
     Running tests\error_matrix.rs     running 9 tests    test result: ok. 9 passed; 0 failed
     Running tests\refusal_matrix.rs   running 11 tests   test result: ok. 11 passed; 0 failed
     Running tests\streaming.rs        running 8 tests    test result: ok. 8 passed; 0 failed
     (doctests)                        0
=== EXIT: 0 ===
```

**What the matrix proves (each case is RED-if-regressed against the behavior in §1):**

1. `rate_limit_429_exhaustion_maps_provider_rate_limit_to_llm_rate_limit` — non-streaming 429 always ⇒ final
   `Refusal::LlmRateLimit` / `E_LLM_RATE_LIMIT`, `http_status=429`, retryable, exactly `MAX_ATTEMPTS` (3)
   HTTP requests. Kills "exhausted `ProviderRateLimit` leaks as its internal code" and "retries 2 or 4 times".
2. `stream_rate_limit_429_exhaustion_maps_to_llm_rate_limit` — same mapping through the stream-open retry loop,
   exactly 3 requests. Kills "stream path forgets `final_error`".
3. `auth_401_is_not_retried` / `payment_required_402_is_not_retried` — `E_PROVIDER_AUTH`, non-retryable,
   exactly 1 request each.
4. `stream_open_401_is_not_retried` — stream open path also refuses auth without retry (1 request).
5. `request_timeout_maps_to_provider_timeout_with_bounded_retries` — 2 s server delay vs 150 ms client timeout ⇒
   `E_PROVIDER_TIMEOUT`, retryable, retries stop at exactly `MAX_ATTEMPTS` requests (never unbounded).
6. `connect_failure_maps_to_provider_timeout` — closed loopback port ⇒ named `E_PROVIDER_TIMEOUT`, retryable,
   non-empty non-secret detail; no panic, no fabricated success.
7. `stream_error_before_first_byte_is_retried` — 429 before any byte (Retry-After: 0) ⇒ one retry, stream then
   delivers `recovered` + `Finished`, `handle.attempts == 2`, 2 HTTP requests.
8. `stream_error_after_first_delta_is_not_retried` — 200 SSE with one delta and no `[DONE]` ⇒ delta observed,
   then `E_PROVIDER_INCOMPLETE`, exactly 1 request (the pump never re-opens after the first byte).

**Artifacts (scope evidence):**

- New: `backend/crates/hiverouter-provider/tests/error_matrix.rs` — 13,046 B, 310 lines, sha256
  `98b9dcfa6f55e7db01f3bfa18dfbc5fe9b5e7d71a169100d6188c27abd747205`.
- New: `plans/waves/W2-PROVIDER-MATRIX.md` (this record).
- `git status --short -- backend/crates/hiverouter-provider/tests/error_matrix.rs plans/waves/W2-PROVIDER-MATRIX.md`
  ⇒ `?? backend/crates/hiverouter-provider/tests/error_matrix.rs` and `?? plans/waves/W2-PROVIDER-MATRIX.md` —
  **only these two files were added; no product file was edited by this seat.** Other modified files in the
  worktree (`core/*`, `provider/Cargo.toml` version bump, Cargo.lock, `.cargo/config.toml`, e2e/web) pre-date
  this seat's work and were not touched. HEAD `c9390f5`; no commit (Lead owns it).
- **Stub sweep (§23)** over both new files: hits are only (a) the permitted wiremock HTTP test double (§7),
  (b) the words "placeholder"/"mock" inside doc comments describing the offline fixture, and (c) empty-mock
  matcher type names. No fabricated business logic, no TODO/FIXME body, no function that returns fake success.
  Classified: permitted test double, not a defect.
- **Skills gate:** `/forgewright-recon` (source read in §1), `/logic-gatekeeper` (Phase 1 accepted before the
  write in §2; fail-closed assertions on every error path), `/integration-logic` (call-chain boundaries captured
  in Phase-1 dataFlow; producer→consumer mapping proven by the exhaustion tests), `/logic-debugger` (first build
  RED: `StreamHandle` lacks `Debug` for `expect_err`; root-caused, replaced with explicit `match`, rerun green —
  evidence above is from the corrected build), `/forgewright-lead` (arguments compared against the stated
  acceptance, not "no error"; §22 DoD met with quoted output).

## 5. Not done

1. **Stream idle timeout after a successful open** (`pump_stream` 100 ms/`stream_idle_timeout` path) is not
   covered by a deterministic test here: `wiremock` cannot stagger SSE chunks, and the provider dev-deps do not
   enable `tokio`'s `net` feature, so a raw-socket stalling server would require a `Cargo.toml` edit — outside
   this seat's write scope (`tests/error_matrix.rs` only). Marked **not done**, not claimed.
2. **`Retry-After` cap (60 s) end-to-end timing** is not exercised as a matrix timing test (a matrix test would
   sleep); the parser/cap is unit-level policy in `budget.rs::RetryPolicy::delay_for`. Not required by the seat
   mandate; recorded instead of silently skipped.
3. **Non-streaming 429 recovery after Retry-After** is already covered by `refusal_matrix.rs`
   (`rate_limit_recovers_after_retry_after`); not duplicated here to keep this file an error matrix.
4. **No live/paid provider verification and no commit** — prohibited/Lead-owned by the mandate.

## 6. Unverified

- **Real-provider equivalence:** the matrix proves the client's handling of locally injected 401/402/429/timeout/
  connect/partial-stream, not how the live DeepSeek endpoint actually behaves (headers, body shape, 429 cadence).
  The mapped codes are contract facts, but live behavior is UNVERIFIED (no key, no paid call allowed).
- **Wiremock vs real transport:** a wiremock 429 with `Retry-After: 0` is a deterministic stand-in; real
  `Retry-After` parsing (seconds/date forms, proxies) is exercised at unit level only, not end-to-end.
- **Attempt-count under real scheduling:** the timeout case asserts exactly 3 recorded requests and passed
  deterministically on this machine; on a heavily loaded machine the *timing* of retries could vary, though the
  count is governed by the loop, not the clock.
- **Error-body content** for 401/402/429 is not asserted (only status/code/retryability), so a provider body
  change would not be caught by this matrix.
- What would prove the remainder: an operator-key-gated live smoke (error injection cannot be requested from a
  real API), and a raw-socket idle-timeout harness enabled with a dev-dependency addition.
