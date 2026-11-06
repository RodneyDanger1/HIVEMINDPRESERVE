# PHASE 5 EVIDENCE — Provider Client, Budget & Cost Accounting

Owner: B-Provider. Date: 2026-11-06 (CST). Blueprint card: Phase 5 (WPs 5.1–5.8).
No live DeepSeek call was made and no `DEEPSEEK_API_KEY` was read: every provider test uses local
`wiremock 0.6.5` HTTP mocks (dev-dependency only; never linked into the release binary).

## Environment (observed)

```
$ rustc --version   →  rustc 1.99.1 (58cf8f9c3 2026-10-13)
$ cargo --version   →  cargo 1.99.1 (5f94df478 2026-08-27)
$ gcc --version     →  gcc.exe (GCC) 16.2.0      (tools/w64devkit/bin, prepended to PATH)
toolchain target: stable-x86_64-pc-windows-gnu; workspace root backend/Cargo.toml
```

## Preflight (blueprint: `cargo check && cargo test --no-run`)

Command (run from `backend/`; `PATH` includes `%USERPROFILE%\.cargo\bin` and `tools\w64devkit\bin`):

```
cargo check -p hiverouter-provider --all-targets        →  CHECK EXIT: 0
cargo test  -p hiverouter-provider --no-run             →  test targets build (exit 0)
```

Observed compile result (first full check after the code landed):

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in ... 
CHECK EXIT: 0
```

## Tests — command + observed output

```
cargo test -p hiverouter-provider
...
test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.68s   (lib)
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.68s   (tests/refusal_matrix.rs)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s    (doc-tests)
TEST EXIT: 0
```

Acceptance / refusal matrix tests actually executed (all `ok`, quoted from the run):

```
refusal_matrix: auth_401_is_a_named_refusal_and_is_not_retried            ok
refusal_matrix: payment_required_402_is_auth_and_not_retried              ok
refusal_matrix: rate_limit_429_retries_with_retry_after_then_reports_llm_rate_limit  ok
refusal_matrix: rate_limit_recovers_after_retry_after                     ok
refusal_matrix: timeout_is_a_named_refusal                                ok
refusal_matrix: malformed_json_is_a_named_refusal                         ok
refusal_matrix: undocumented_status_is_refused_by_name                    ok
refusal_matrix: null_usage_marks_the_run_incomplete_and_never_settles_zero ok
refusal_matrix: budget_preflight_refuses_with_zero_provider_requests      ok
refusal_matrix: success_path_records_usage_and_settles_decimal_cost       ok
refusal_matrix: streaming_client_parses_sse_and_reports_usage_none_as_incomplete ok
```

Unit tests include: F1 price table equality (both models × peak/off-peak, Decimal), UTC peak-window
boundaries (01:00 inclusive / 04:00 exclusive / 06:00 / 10:00 / weekend off-peak), cost math for every
F1 row, NULL/partial/inconsistent usage ⇒ INCOMPLETE with `settled_usd: null`, budget preflight refusals
by name with **0 provider requests**, Retry-After parse + 60 s cap, jittered backoff bounds, one-key rule,
spawn caps, in-flight semaphore (`E_SEMAPHORE_FULL`), zeroised key (`Debug` prints `ApiKey(***)`), and
SSE line parsing including malformed `data:` frames.

## Exit-gate mapping (blueprint Phase 5)

| Gate | Evidence |
|---|---|
| mock 401/402/429/timeout produce distinct refusals | `E_PROVIDER_AUTH` (401 & 402, **0 retries**), `E_LLM_RATE_LIMIT` (429 after 3 attempts), `E_PROVIDER_TIMEOUT`, `E_PROVIDER_INVALID_RESPONSE` — all executed above |
| budget refusal happens before the request is sent | `budget_preflight_refuses_with_zero_provider_requests`: wiremock `received_requests().len() == 0` |
| NULL usage can never be exported as a settled cost | `null_usage_marks_the_run_incomplete_and_never_settles_zero`: report `status: INCOMPLETE`, `settled_usd: null`, `incomplete_reason: E_USAGE_INCOMPLETE`; run also blocks further sends (`incomplete_usage_blocks_further_sends`) |
| cost math reproduces FACT-REGISTER rows incl. UTC peak windows | `cost_math_reproduces_every_f1_row` (Flash off 0.753 / peak 1.506; Pro off 2.662 / peak 5.324 for 1M hit + 1M miss + 1M output) |
| provider matrix runs offline; costs Decimal-only | wiremock only, no key/network; crate root has `#![deny(clippy::float_arithmetic)]` and `#![forbid(unsafe_code)]`; money type is `rust_decimal::Decimal` everywhere on the cost path |
| rig-core spike passed before Rig is used | **N/A**: Phase 5 stays fully custom per blueprint WP 5.5 scope rewrite; `rig-core` is not in the lockfile and no Rig spike is claimed |

## Files

- `backend/crates/hiverouter-provider/src/{lib,dto,secret,error,pricing,budget,accounting,client}.rs`
- `backend/crates/hiverouter-provider/tests/refusal_matrix.rs`
- `backend/crates/hiverouter-provider/Cargo.toml` (workspace deps; `wiremock` dev-only)
- `docs/plans/PHASE-5-RECON.md` (written before edits), this file.

## Deliberate decisions recorded

- **402 ⇒ `E_PROVIDER_AUTH`** (the frozen table has no payment code; "401/402 never retried" per §2a).
  Four distinct refusals across the matrix: ProviderAuth / LlmRateLimit / ProviderTimeout /
  ProviderInvalidResponse.
- **429 final mapping**: an in-call 429 is `E_PROVIDER_RATE_LIMIT` while retries remain; after the retry
  budget it is reported as `E_LLM_RATE_LIMIT` (matches blueprint Phase 4/5 wording).
- **Transport failures** (connect/DNS/TLS/read) map to `E_PROVIDER_TIMEOUT` — the frozen refusal table has
  one transport code; adding another needs a contract change + ledger row.
- **Prompt-cache split is required to settle a cost**: if the provider reports `prompt_tokens` without
  `prompt_cache_hit_tokens`/`prompt_cache_miss_tokens`, the run is INCOMPLETE instead of guessing a price.
- `temperature` / `frequency_penalty` / `presence_penalty` are not sent: F1 documents the first as having
  no effect in thinking mode and the last two as unsupported.
- Streaming usage is NOT DOCUMENTED; absent usage in the stream stays NULL (run INCOMPLETE if used for billing).

## Not done / UNVERIFIED (honest list)

- **No live call**: per-request latency, cache-hit rate, real 429 behaviour against DeepSeek are UNVERIFIED
  (deliberate: no key use in this phase).
- Streaming *tool-call assembly across chunks* is parsed and emitted as deltas, but the final typed
  `ToolCall` assembly belongs to Phase 6/4 callers and is not exercised against a live provider.
- `cargo clippy -- -D warnings` evidence for the float lint is the crate attribute plus code inspection;
  a clippy run is scheduled with the Phase 11 verification pass (see Phase 11 evidence).
