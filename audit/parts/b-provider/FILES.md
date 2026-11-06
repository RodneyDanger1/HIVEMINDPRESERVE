# FILES — b-provider partition (33 files, 15 deep)

Audit of HEAD `3988622`; all metadata (`bytes`, `sha256`) is injected from `audit/inventory/FILE-MANIFEST.csv`
and spot-verified. Deep files carry 8 labelled paragraphs (mechanics, logic, edge/failure, dependencies,
callers, tests/evidence, quality, risks); standard files carry a structured record. Every behavioural claim
below is from reading the file (READ) unless a test/command is named (VERIFIED).

## backend/Cargo.lock

- **Metadata:** 100124 B · sha256 `7B704C18E67F56DA0AF26FEB3914DB0AAA841A2B66F2066DE206A915FF7C859A` · language Lockfile · category config-manifest · depth deep.

- **Mechanics.** Resolves the 4-workspace-member crate graph into 405 external packages with exact versions
  for the GNU target (`libsqlite3-sys 0.39.1`, `rustls 0.23.46`, `ring 0.17.14`, `aws-lc-rs 1.19.0`,
  `aws-lc-sys 0.47.0`, `h2 0.4.20`, `ipnet 2.12.2`). The four local packages are all `0.1.0` path packages.
- **Logic/algorithm.** The lock is a pure resolution artifact: `--locked` builds fail if any manifest demand
  is unsatisfied, so the lock is the reproducibility anchor for the release gate. It records 26 duplicate
  crate names, meaning several shared dependencies coexist at different majors/versions.
- **Edge/failure behaviour.** Duplicate majors include two of the blueprint's F2 pins: `thiserror 1.0.69` +
  `2.0.21` and `tower-http 0.6.11` + `0.7.1`; the rest are transitive (windows 0.61.3/0.62.2, rand
  0.8.8/0.9.5/0.10.3, getrandom 0.2/0.3/0.4, syn 2.0/3.0, base64 0.22/0.23). `cargo tree -d` evidence was not
  re-run (read-only audit, no builds) so whether the Phase-1 duplicate-major gate tolerates these is UNVERIFIED.
- **Dependencies.** `backend/Cargo.toml` workspace pins; transitive graph dominated by tokio/reqwest/rustls,
  rusqlite bundled (C build via `libsqlite3-sys`), and the dev-only wiremock/criterion trees.
- **Callers.** Cargo itself (all four members), the release/build gates, `cargo build --release --locked`.
- **Tests/evidence.** Spot-verified hash equals manifest (`7B704C18…C859A`, 100,124 B); duplicate-name map
  produced by regex over the lock (385+ name/version pairs).
- **Quality.** Reproducible and honest; the duplicate-major presence is a gate-language mismatch rather than a
  correctness defect. 4/5.
- **Risks/limits.** Lock contains `aws-lc-sys` (C build) and `ring` simultaneously via rustls provider
  selection; build environment must keep gcc; the F9 "no duplicate major" claim needs a waiver or alignment.

## backend/Cargo.toml

- **Metadata:** 1765 B · sha256 `323A5865A3ED81F1D2BBD00131195E7CC5EAD96ACE998606D640461AFC258F10` · language TOML · category config-manifest · depth deep.

- **Mechanics.** Workspace root: 4 members, `edition 2021`, `rust-version 1.89.0`, `resolver 2`, all pins
  `=`-exact for 27 external crates, plus `[profile.dev]` (debug 0, opt-level 1, no incremental) and
  `[profile.release]` (thin LTO, codegen-units 1, strip, panic=abort).
- **Logic/algorithm.** Pins implement the FACT-REGISTER F2/F10 stack; every version is exact (`tokio
  =1.54.1`, `axum =0.8.9`, `reqwest =0.13.5`, `serde =1.0.229`, `rusqlite =0.41.1`, `schemars =1.2.3`,
  `jsonschema =0.60.2`, `rust_decimal =1.43.0`, `win32job =2.0.3`, `notify =8.2.1`, `sysinfo =0.40.1`,
  `strum =0.28.0`, `zeroize =1.9.1`, `uuid =1.29.0`, `tracing-subscriber =0.3.23`).
- **Edge/failure behaviour.** `[profile.release] panic = "abort"` means a panic is a process abort (no unwinding
  across FFI), consistent with a local server; `strip=true` removes symbols from the shipped binary, so crash
  diagnosis depends on the JSONL logs rather than backtraces.
- **Dependencies.** The provider and tools manifests inherit most entries via `workspace = true`; the tools
  crate uses `criterion` outside the workspace table (inline `=0.8.2`, dev-only).
- **Callers.** All member manifests, the release gate, CI/harness scripts.
- **Tests/evidence.** Read-only; 50 lines; hash `323A5865…258F10` matches manifest. Feature list read:
  reqwest features are `json, rustls, http2` (see FINDINGS: reconciling the `rustls-tls` wording).
- **Quality.** Clean, exact-pinned, MSRV declared; one deviation (inline criterion) is documented dev-only.
- **Risks/limits.** reqwest 0.13 TLS feature naming is the only version-sensitive line; `cargo deny` was not
  run in this audit (UNVERIFIED licence gate).

## backend/crates/hiverouter-provider/Cargo.toml

- **Metadata:** 752 B · sha256 `6F32DD277F25B2627D2696D740513069C832098A93A529245F3BBA0317209F4D` · language TOML · category config-manifest · depth standard.

- **Record.** 26 lines. Library `hiverouter_provider` depends on `hiverouter-contract` (path), serde,
  serde_json, thiserror, tokio (rt/time/sync/macros), reqwest (workspace), tracing, rust_decimal, uuid,
  zeroize, async-trait; dev-deps wiremock, tokio (rt-multi-thread/macros/time), tracing-subscriber.
- **Findings.** `async-trait` is declared but has zero uses in `src/**` (grep evidence); the rest map 1:1 to
  modules. Quality 4/5; exact workspace pins; no test double in `[dependencies]` (wiremock stays dev-only).

## backend/crates/hiverouter-provider/src/accounting.rs

- **Metadata:** 17056 B · sha256 `7BDA95F2A98177ED6E12FBEE898E1B809B1CDA4F392475FA514828A7BD2CC8D0` · language Rust · category source · depth deep.

- **Mechanics.** V1–V6 accounting: `RunAccounting::new/record/status/report/settled_usd`; `CallCost`,
  `CostReport`, `Provenance`, `CostStatus`. Cost = hit×hit-price + miss×miss-price + completion×output-price
  per 1M tokens, Decimal-only.
- **Logic/algorithm.** `record` (205) requires completion tokens and a full cache hit/miss split; `price`
  (330) multiplies tokens by the per-million Decimal and divides by 1,000,000; `usage.checked_add` keeps
  absent counters absent; a NULL/partial usage sets `incomplete_reason = E_USAGE_INCOMPLETE`.
- **Edge/failure behaviour.** `report` (304) exports `settled_usd: null` when INCOMPLETE and never `0.00`;
  inconsistent `prompt_tokens != hit+miss` is refused; a run that settled once then records null usage turns
  INCOMPLETE and the earlier figure is no longer exported.
- **Dependencies.** `rust_decimal` (Decimal), `hiverouter_contract::Usage/Refusal`; used by provider
  budget/client and by the CLI's `provider cost`.
- **Callers.** `RunBudget::record_call` (budget.rs:291), `DeepSeekClient::drive` (client.rs:312) via
  `complete_with_budget`, CLI `provider.cost` (cli.rs).
- **Tests/evidence.** 4 unit tests include `cost_math_reproduces_every_f1_row`, `null_or_partial_usage…`,
  `report_serialises_null_as_null` (cargo test -p hiverouter-provider: 29 passed).
- **Quality.** Strong separation, Decimal purity, explicit provenance; `status()` (163) carries a confusing
  double condition (see FINDINGS H-7) and `price()` hides overflow as `ZERO` (H-1). 4/5.
- **Risks/limits.** Overflow path would silently under-report cost; no fuzz/property test on Decimal rounding.

## backend/crates/hiverouter-provider/src/budget.rs

- **Metadata:** 23965 B · sha256 `0B685A5ED36B97DC4D3BEC729CC5F844E3A0CD80AE82D182EE88F779A6CD038C` · language Rust · category source · depth deep.

- **Mechanics.** B1–B12 policy core: `BudgetLimits` defaults ($0.50 cap, 400k tokens, 40 calls, 3 attempts),
  `CallEstimate`, `BudgetPermit`, `RunBudget` (preflight/record_call/on_retry/spent_usd/remaining_usd),
  `RetryPolicy` (jittered exponential, Retry-After capped 60 s), `InFlightLimiter` (semaphore 8),
  `RunCaps` (depth 3/fan-out 8/agents 64), `enforce_single_key`.
- **Logic/algorithm.** `preflight` (235) refuses before any send: INCOMPLETE run, call/attempt cap (via
  `attempts`), spent ≥ cap, projected token budget (fresh run counts 0). `record_attempt` counts sends
  including retries; `record_call` delegates money to `RunAccounting`.
- **Edge/failure behaviour.** Retry-After parse accepts integer seconds only, caps at 60 s, ignores HTTP-date;
  jitter uses uuid-v7 low bits (no RNG crate); semaphore queue is bounded by the 8 permits and
  `try_acquire` returns `E_SEMAPHORE_FULL`; spawn caps refuse by name.
- **Dependencies.** tokio Semaphore, uuid (jitter), rust_decimal, contract Refusal. Consumed by
  client.rs and (indirectly) core execution.
- **Callers.** `DeepSeekClient::complete_with_budget` (client.rs:301) and `drive` (312); CLI provider cost
  indirectly; tests/refusal_matrix.rs.
- **Tests/evidence.** 9 unit tests + the matrix test `budget_preflight_refuses_with_zero_provider_requests`
  (wiremock received_requests == 0) — VERIFIED in the Phase-5 run.
- **Quality.** Clean policy/accounting split; constants documented PROPOSED-POLICY. `spent_usd` (176) hides
  spend as ZERO when incomplete (H-2). 4/5.
- **Risks/limits.** Token estimate is `Assumed` provenance and never settled; budget can overshoot by one
  in-flight call (cap is pre-send), which is the documented policy.

## backend/crates/hiverouter-provider/src/client.rs

- **Metadata:** 27461 B · sha256 `A8B0A5F88C86F2942155CC52F2E9924CB291609ADB38A8307F0A008D0D19A705` · language Rust · category source · depth deep.

- **Mechanics.** The single DeepSeek client: two reqwest clients (bounded non-stream and streaming),
  `ClientConfig` (documented base URL, connect 10 s, request 600 s, stream idle 600 s), `complete`,
  `complete_with_budget`, `drive`, `stream`, SSE parser (`parse_sse_line`), `pump_stream`, refusal mapping.
- **Logic/algorithm.** `drive` (312) validates the request, acquires the in-flight permit, then per attempt
  runs `preflight`+`record_attempt` (budgeted mode) and `send_once`; retry only when `Refusal::retryable()`
  and attempts remain; 429 ends as `E_LLM_RATE_LIMIT`; streaming retries only before the first byte.
- **Edge/failure behaviour.** 401/402 never retried; 429 honours Retry-After (capped); connect/DNS/TLS/read
  failures map to `E_PROVIDER_TIMEOUT`; non-documented statuses map to `E_PROVIDER_INVALID_RESPONSE`;
  malformed body and empty choices are named refusals; stream without `[DONE]` maps to `E_PROVIDER_INCOMPLETE`.
- **Dependencies.** reqwest (rustls via workspace), tokio mpsc/time, zeroize (bearer), contract Refusal,
  budget limiter/retry, dto DTOs, secret ApiKey.
- **Callers.** Core execution (via `hiverouter_provider` re-exports), CLI cost (no network), provider tests.
- **Tests/evidence.** 3 unit tests (SSE parsing, Debug redaction, endpoint/model ids) + 11 wiremock matrix
  tests — VERIFIED; 401/402 request-count assertions and 429 3-attempt assertion executed.
- **Quality.** Single owner, no provider SDK, no live-call fabrication; streaming incomplete path lacks a
  dedicated test (H-4). 4/5.
- **Risks/limits.** 5xx responses are non-retryable `E_PROVIDER_INVALID_RESPONSE` (deliberate but debatable);
  transport refusals collapse into one code because the frozen table has one transport refusal.

## backend/crates/hiverouter-provider/src/dto.rs

- **Metadata:** 24305 B · sha256 `A1C0E0587814058506B85A8E70261D0A13BF2A52F11C43EB0BE80D4698017FF8` · language Rust · category source · depth deep.

- **Mechanics.** Documented Chat Completions DTOs: `Model` (flash/pro ids), `Role`, `ChatMessage`,
  `Thinking`/`ThinkingType`, `ReasoningEffort`, `ToolChoice`, `ResponseFormat`, `ToolSpec`,
  `FunctionDefinition`, `ChatRequest` (314), response `ChatResponse`/`Choice`/`ResponseMessage`/`UsageDto`
  (+completion details), and stream `StreamChunk`/`Delta`/`DeltaToolCall`/`StreamChoice`.
- **Logic/algorithm.** `ChatRequest::validate` (400) enforces non-empty messages, `max_tokens` 1..=393216,
  `tool_choice=required` needs tools, and rejects forced tool calls with thinking enabled (documented 400).
  `UsageDto::to_usage` maps absent counters to `None` (never zero); `contract_usage` filters all-absent usage.
- **Edge/failure behaviour.** Unknown response fields are ignored by serde; `usage: null`, `{}` and absent all
  map to `None`; a body missing `choices` fails serde → `E_PROVIDER_INVALID_RESPONSE`; undocumented model ids
  are refused by `Model::from_id`.
- **Dependencies.** serde only (+ contract Usage); no provider SDK, no f64 fields (money-safe).
- **Callers.** client.rs (send/parse/stream), CLI cost (constructs Usage separately), core prompt/exec code.
- **Tests/evidence.** 6 unit tests: model ids, serialises documented fields only, validation conflicts,
  absent usage, usage mapping, streaming chunk parsing — all pass (VERIFIED).
- **Quality.** Faithful to FACT F1; explicit omissions (`temperature`, penalty params) documented. 5/5.
- **Risks/limits.** Streaming tool-call final assembly is caller-owned; no dedicated test for `[DONE]`-absent
  stream (client-level gap H-4).

## backend/crates/hiverouter-provider/src/error.rs

- **Metadata:** 4483 B · sha256 `56E7116E9155C315AF52CE7C623464B03683C0A1458DB06AFEF70CE15EE1EC09` · language Rust · category source · depth standard.

- **Record.** 133 lines. `ProviderError { refusal, detail, http_status, retry_after }` with
  `new/with_status/with_retry_after/code/refusal/detail/retryable/log_line`, Display+Error, `From<Refusal>`.
  One refusal identity only (contract `Refusal`); detail is non-secret context; `log_line` never includes
  headers/bodies. Deps: contract Refusal + std. Used by every provider module. 2 unit tests (code identity,
  no key material). Quality 5/5 — single named-refusal owner, redaction-safe.

## backend/crates/hiverouter-provider/src/lib.rs

- **Metadata:** 2408 B · sha256 `83343C8A7FA13A58AD3EB2199186464593F2331AC465A4F7218112D644E6F704` · language Rust · category source · depth standard.

- **Record.** 49 lines. Module map + crate-level guarantees: `#![forbid(unsafe_code)]`,
  `#![deny(clippy::float_arithmetic)]`, `#![warn(missing_docs)]`, re-exports of every public type and the
  PROPOSED-POLICY constants; `PROVIDER_VERSION`. Purpose: one provider client/one budget-cost owner
  (CANONICAL C16/C17). No logic of its own. Quality 5/5 (lint fence is the money rule enforcement point).

## backend/crates/hiverouter-provider/src/pricing.rs

- **Metadata:** 9959 B · sha256 `28E627929595016A02C74949FAC4EB6A48003003D24F3BCB2DA5ADCC53D55F24` · language Rust · category source · depth deep.

- **Mechanics.** FACT F1 price rows with provenance (`source_url`, `fetched_at`), `Peak::{Peak,OffPeak}`,
  `PriceRow`, `PricingTable::documented/row/rows`, UTC peak windows `[(1,4),(6,10)]`, weekday helpers
  (`weekday_monday_zero`, `weekday_label`), `is_peak_at_unix`, `peak_at_unix`.
- **Logic/algorithm.** `documented` (93) builds four rows from Decimal::new values (flash 0.003/0.15/0.6 off,
  0.006/0.30/1.2 peak; pro 0.022/0.66/1.98 off, 0.044/1.32/3.96 peak); `row` (128) refuses a row whose
  provenance is empty with `E_PRICE_UNSOURCED`; `is_peak_at_unix` (179) is Mon–Fri only with `[start,end)`.
- **Edge/failure behaviour.** Missing (model,rate) pair → `E_PRICE_UNSOURCED`; weekend inside window is
  off-peak; boundary hours proven by tests (01:00 in, 04:00 out, 06:00 in, 10:00 out); row lookup clones a
  `Copy` row so no memoization staleness.
- **Dependencies.** rust_decimal, contract Refusal/Timestamp (tests). Callers: accounting.rs, CLI provider cost,
  health `/diagnose` (provenance fields).
- **Callers.** `accounting.rs::record` (row lookup + peak class), CLI `provider.cost`, and `health.rs::diagnose` (price provenance fields); tests pin the F1 rows.
- **Tests/evidence.** 3 unit tests pin every F1 value, peak boundaries and unsourced-row refusal — VERIFIED.
- **Quality.** Provenance enforced at lookup, no f64. 5/5.
- **Risks/limits.** Peak windows are hard-coded policy constants (documented); no test for leap/UTC edge dates.

## backend/crates/hiverouter-provider/src/secret.rs

- **Metadata:** 3349 B · sha256 `297E6A7E4A783DD253E6E358870D2E01C2CB227D600253763B615D1F721D059A` · language Rust · category source · depth standard.

- **Record.** 91 lines. `ApiKey` wraps `Zeroizing<String>`; `from_env`/`from_secret` refuse empty key with
  `E_KEY_MISSING`; `present_in_env` boolean; `expose` is crate-only; custom `Debug` prints `ApiKey(***)`.
  Deps zeroize. Callers: client.rs (bearer header), settings/commands key presence, CLI preflight.
  2 unit tests (Debug redaction, empty refusal). Quality 5/5 — no key logging/printing path.

## backend/crates/hiverouter-provider/tests/refusal_matrix.rs

- **Metadata:** 11475 B · sha256 `80389102FB8A2CC3142CD0071839CAAE43F2306BB1870738504D24F576767E6E` · language Rust · category source · depth standard.

- **Record.** 278 lines, 11 wiremock tests (offline): 401/402 no-retry (request-count == 1 each), 429 retried
  to `E_LLM_RATE_LIMIT` (request-count == 3) and 429→200 recovery, read timeout, malformed JSON, undocumented
  500, NULL usage INCOMPLETE, budget preflight with zero provider requests, success-path Decimal settlement,
  SSE stream parse with usage NULL. Purpose: prove the documented refusal matrix without a key or network.
  Quality 5/5 — dev-only double at the HTTP boundary, no product fake.

## backend/crates/hiverouter-tools/Cargo.toml

- **Metadata:** 1198 B · sha256 `179D5B83474FB430FF1B81E5A9B69D39F77B8ECC53C2606E0423763F4B433057` · language TOML · category config-manifest · depth standard.

- **Record.** 44 lines. Library `hiverouter_tools` + `[[bin]] hiverouter-cli` + `[[bench]] engine_benches`
  (harness=false). Deps (workspace): contract, provider, serde/serde_json, thiserror, tokio, axum,
  tracing/tracing-subscriber/tracing-appender, sysinfo, schemars, jsonschema, strum, uuid, async-trait,
  rusqlite (bundled), tokio-util, notify, reqwest; windows-only win32job; dev-deps tempfile, criterion
  `=0.8.2`. Findings: `async-trait` unused; criterion inline dev-only (documented). Quality 4/5.

## backend/crates/hiverouter-tools/benches/engine_benches.rs

- **Metadata:** 3270 B · sha256 `2CE34ADF5B6C3B57EE106AE1DB861B209FAB44C013F08974AA235B6A9DF78A92` · language Rust · category source · depth standard.

- **Record.** 83 lines, criterion harness=false. Four benches: tool_schema_validate_file_write,
  sha256_args_binding, jail_resolve_relative_path, decimal_cost_one_call_peak (arithmetic inputs are fixed;
  no absolute-time assertions). Dev-only, never linked into release. Status: instrument exists; measurement
  recorded UNVERIFIED at hand-off (criterion graph compile exceeded the window; no number was claimed).
  Quality 4/5 (useful, but no baseline file/threshold gate).

## backend/crates/hiverouter-tools/src/approval.rs

- **Metadata:** 34944 B · sha256 `E32411E62FFEC925A2C2F9FFDCC6BCB6C7BC16FBA42944253DACA70A35652C22` · language Rust · category source · depth deep.

- **Mechanics.** The single approval/consent owner: `ApprovalDecision` (AllowOnce/AllowSession/Deny + wire
  approve|deny), `ApprovalRequest`, `ApprovalResolution`, `hash_args` (in-house SHA-256), `migrate` DDL
  (`UNIQUE(nonce)`, single-use decision, TTL, `executed_at`), `insert_request`, `resolve_with_nonce`,
  `resolve_with_reason`, `session_allowed`, `pending_for_run`, `ApprovalGate`.
- **Logic/algorithm.** A row is committed before any side effect; `resolve_with_nonce` parses the wire
  decision, matches the one-time nonce, then atomically `UPDATE … WHERE decision IS NULL`; expiry marks the
  row `expired` first so a late decision can never execute; `AllowSession` is scoped to
  (run, tool, args_sha256); `verify_grant` consumes an `AllowOnce` grant exactly once (second use →
  `E_APPROVAL_REPLAY`).
- **Edge/failure behaviour.** Replay → `E_APPROVAL_REPLAY`; unknown id → `E_APPROVAL_NOT_FOUND`; bad decision
  → `E_APPROVAL_DECISION_INVALID`; TTL → `E_APPROVAL_TIMEOUT` with zero side effects; duplicate nonce at
  insert → replay; read-only class cannot request approval.
- **Dependencies.** rusqlite (single-owner connection passed by core), uuid-v7, serde, contract ids/Refusal,
  `crate::sha256`, `crate::error`, `crate::tool`.
- **Callers.** Core `tools_bridge.rs:21,32,62,75` (migrate + gate), `commands.rs::approval_resolve`, WS command
  dispatch; ledger tests.
- **Tests/evidence.** 10 unit tests incl. `approval_suspend_resume_roundtrip` (single-use consume + replay),
  TTL zero-side-effect, args binding, session scope, nonce/replay wire path, decision validation — VERIFIED.
- **Quality.** Strong invariants at DB boundary; `pub fn resolve` (no-nonce path) remains exported alongside
  `resolve_with_nonce` (H-3) and `permission_class` is caller-supplied (H-14). 4/5.
- **Risks/limits.** SQLite single-file lock is delegated to core; no concurrency stress test.

## backend/crates/hiverouter-tools/src/audit.rs

- **Metadata:** 10695 B · sha256 `960E793A5369765F7476D593C99B822349BE2E495E65DA0545B8656A19E7BE35` · language Rust · category source · depth standard.

- **Record.** 294 lines. One local JSONL sink: `AuditGuard`, `init_json_audit` (tracing-subscriber JSON layer
  + tracing-appender rolling non-blocking daily file, flatten_event), `record_refusal`/`record_tool_call`
  (run_id/code/tool/permission_class, no key), `AuditRow`, `list/read` (every line must parse),
  `probe_audit_file/dir` rejection of `DEEPSEEK_API_KEY`/`sk-`/`Bearer `. 2 unit tests + 1 integration
  (`tests/audit_jsonl.rs`) — VERIFIED at Phase-11. Quality 5/5; limitation: single global subscriber (by
  design), `handle_count` N/A.

## backend/crates/hiverouter-tools/src/cancel.rs

- **Metadata:** 10411 B · sha256 `61C76D60BBF75E97EC4C782295AB978D584C9679EB5C86873E69299F3C9F9141` · language Rust · category source · depth standard.

- **Record.** 281 lines. `TerminalState` (Cancelled/Killed), `TerminalOutcome`, `ProcessRegistry`
  (register_run/register_task/token_for/cancel_run/kill_task/is_cancelled/terminal/unregister/active_runs)
  with per-run `tokio_util::sync::CancellationToken` and child tokens. Named refusals for
  unknown/terminal/task-missing. 4 unit tests including broadcast and select-on-token — VERIFIED. Quality
  5/5; limitation: in-process only (documented; restart loses registry).

## backend/crates/hiverouter-tools/src/census.rs

- **Metadata:** 9205 B · sha256 `3F67B9D0C66155E7FF209F2BC04F230B9974C835CCB3302129DA2856C5EEB4F7` · language Rust · category source · depth deep.

- **Mechanics.** Derived capability census (`CENSUS_SCHEMA`, `CapabilityCensus`, `grant_count`,
  `check_census_drift`, `derive_census`, `standard_census`, `PROMPT_CLAIMED_TOOL_IDS`, `contracts`).
- **Logic/algorithm.** Compares three sets: registry ids (from ToolId::iter map), deduplicated allowlist ids,
  and declared prompt claims; any difference → `E_INTERNAL_ERROR` with the missing/extra ids; counts come
  from `strum::EnumIter`, never typed constants.
- **Edge/failure behaviour.** Removing one registry row, one allowlist grant or one prompt claim each fails
  the drift check (unit-tested); serialisation is consumed by `/diagnose` and the CLI census command.
- **Dependencies.** strum, serde, contract Refusal, `crate::tool`. Callers: health.rs, CLI, core `/diagnose`.
- **Callers.** `health.rs::ServiceState::new`/`diagnose`, CLI `capabilities.census`, and the Phase-11 drift test consume the census; no core import edge.
- **Tests/evidence.** 4 unit tests (`standard_registry_census_is_consistent`, `census_fails_when_one_row_is_deleted`,
  `counts_are_derived_not_typed`, `census_serialises_for_the_diagnosis_endpoint`) — VERIFIED.
- **Quality.** The right anti-drift mechanism; but set equality cannot detect a *tier-specific* mis-grant
  when the same tool is granted elsewhere (H-5 tier-blind comparison; `tool.rs` tests pin current grants).
- **Risks/limits.** Prompt claims are a hard-coded const in this crate (single source is Rust, not the prompt
  asset) — a second manual copy that must be kept in sync with Phase-3 prompts.

## backend/crates/hiverouter-tools/src/cli.rs

- **Metadata:** 37923 B · sha256 `A9F583EECAFB19611D58BE384F5F7DAB69EFDF16B28D132507C4DDB18721E704` · language Rust · category source · depth deep.

- **Mechanics.** Agent-native CLI core: `CLI_SCHEMA`, `COMMANDS` (9 command specs with tier/mutating),
  `CliFailure`/`CliOutcome` (envelope + human lines), `dispatch` (357), `run_main` (389), help/schema
  generation, `preflight`, `tools.*`, `capabilities.census`, `provider.cost`, `health`/`diagnose` HTTP and
  dry-run; 13 unit tests.
- **Logic/algorithm.** Hand-rolled arg parser with global flags (`--format=json|human`, `--json`, `--dry-run`,
  `--url`, `--timeout`); JSON envelope on stdout for machines, human lines on stderr for TTY; exit codes
  0/2/3/4; every schema/help row is derived from the `COMMANDS` table; model/peak/Decimal cost computed via
  provider crate; HTTP calls use a per-invocation current-thread tokio runtime.
- **Edge/failure behaviour.** Unknown command → `E_COMMAND_UNKNOWN` exit 2; unknown tool → `E_TOOL_NOT_FOUND`
  exit 3; bad flag → `E_SCHEMA_INVALID` exit 2; missing node/rust → `E_NODE_MISSING`/`E_TOOLCHAIN_MSRV`
  exit 3 (observed); unreachable server → `E_WS_DISCONNECTED` exit 3; `--dry-run` never sends.
- **Dependencies.** contract Refusal/Timestamp/Usage, provider (pricing/accounting/ApiKey), crate::census,
  crate::tool, crate::cli_support; no clap (hand-rolled parser, ~80 lines).
- **Callers.** `src/bin/hiverouter.rs` main, `tools/bin` packaging, E2E/QA harnesses.
- **Tests/evidence.** 13 unit tests (version/unknown/schema/help/preflight/tools/census/cost/health dry-run/
  unreachable/flag/envelope) — VERIFIED; release binary smoke observed exits 0/2/3.
- **Quality.** Self-describing and machine-first; duplicate floor constants (H-9); human-mode stdout silence
  is documented. 4/5.
- **Risks/limits.** `--format=human` on a TTY leaves stdout empty by design; no shell completion; no clap
  means every new flag needs parser care.

## backend/crates/hiverouter-tools/src/cli_support.rs

- **Metadata:** 6270 B · sha256 `54AAF4ABE2872C280202A04F036A29C22AA442FB2BF97B95E1D994D1889AD556` · language Rust · category source · depth standard.

- **Record.** 146 lines. `parse_version_triple`, `node_floor()` (22.12.0), `rust_msrv_floor()` (1.89.0),
  `version_probe` (5 s bounded child wait, kills on timeout), `fetch_json` (current-thread runtime, reqwest
  timeout, status/parse errors). Deps std/tokio/reqwest. 3 unit tests (parsing + host versions + unreachable
  port). Quality 4/5; the floor helpers duplicate the string constants in `cli.rs` (H-9).

## backend/crates/hiverouter-tools/src/commands.rs

- **Metadata:** 7350 B · sha256 `639E2376892FD4622B4506916D18ADECB87B91EC1A870770D86FE7FAAE84AE2E` · language Rust · category source · depth standard.

- **Record.** 194 lines. Contract-patch handlers: `settings_status_command` (24), `file_tree_list` (34),
  `file_read` (56, jailed, UTF-8, sha256/bytes/text), `approval_resolve` (bridge to the gate). Deps contract
  command/event payloads, rusqlite, crate::{approval,error,jail,settings,sha256}. Callers: core
  `tools_bridge.rs:114,123,133`; 4 unit tests (jail read/tree, patched refusals, approval nonce, settings
  secret-free) — VERIFIED. Quality 5/5 (thin, contract-typed adapters).

## backend/crates/hiverouter-tools/src/error.rs

- **Metadata:** 2717 B · sha256 `D987C3E066AA08C68D55C887BDF2F725AF43B9FEE7FBBB7D0DBF74A4B77F2858` · language Rust · category source · depth standard.

- **Record.** 92 lines. `ToolError { refusal, detail }` with code/refusal/retryable/Display/Error,
  `From<Refusal>`, `From<rusqlite::Error>` (→ `E_INTERNAL_ERROR`), `From<std::io::Error>`. Single refusal
  identity; argument bodies never stored. 1 unit test. Quality 5/5.

## backend/crates/hiverouter-tools/src/health.rs

- **Metadata:** 15752 B · sha256 `91AFD4D1FA92A032B6783067CFCB179A2B3C868539FD483B2CE4449AECBF2B2D` · language Rust · category source · depth deep.

- **Mechanics.** `/health` + `/diagnose`: `HealthReport`, `ProviderDiagnosis`, `RefusalCensus`, `ToolCensus`,
  `DiagnoseReport`, `ServiceState` (census, cached sysinfo, children/db flags), `router` (267), handlers, and
  pure seams `diagnostics`/`health_report` for core's route wiring; 5 unit tests (real ephemeral server).
- **Logic/algorithm.** Sampling uses sysinfo process refresh; missing sample stays `None` (null), measured zero
  stays `0`; `/diagnose` returns derived census + F1 caps + price provenance + key presence boolean, never
  the key; census failure refuses startup.
- **Edge/failure behaviour.** Unknown route 404; no-sampling fields null; `children` setter lets core report
  real zeros; a drifted census returns `E_INTERNAL_ERROR` before serving inconsistent counts.
- **Dependencies.** axum, sysinfo, serde, provider pricing/constants, contract, crate::census/tool.
- **Callers.** Core `http.rs`/`tools_bridge.rs:167` (diagnostics), CLI `health`/`diagnose`, tests.
- **Tests/evidence.** 5 unit tests: 200/schema/status/children-zero, updated counts, derived+secret-free
  diagnose, pure seam, 404 — VERIFIED (Phase-11 run).
- **Quality.** Honest null-vs-zero; `handle_count` explicitly not implemented (documented deviation). 4/5.
- **Risks/limits.** Sysinfo CPU/memory semantics are platform-dependent (bytes/percent) and only smoke-tested.

## backend/crates/hiverouter-tools/src/jail.rs

- **Metadata:** 16815 B · sha256 `1F6072AD7622971B271DBB2404AD32B9FEFB612A6D864721F5263B8164AB0949` · language Rust · category source · depth deep.

- **Mechanics.** Single Windows workspace jail: `WorkspaceJail::open/root/resolve/read/write/list`,
  `DirEntryInfo`, `MAX_READ_BYTES = 1 MiB`, six escape vectors refused, symlink/junction canonicalisation.
- **Logic/algorithm.** `resolve` (92) rejects NUL/UNC/device/absolute/drive prefixes, `..`, `:` and `~`
  segments, joins under the canonical root, then `verify_containment` canonicalises the nearest existing
  ancestor and compares lowercased component prefixes; read cap and I/O errors are mapped to
  `E_PATH_NOT_FOUND`/`E_FILE_TOO_LARGE`/`E_INTERNAL_ERROR`.
- **Edge/failure behaviour.** Path escapes are refused, never clamped; symlink/junction pointing outside is
  refused after canonicalisation; oversized read refused; a file replaced by a directory fails schema check.
- **Dependencies.** std::fs, contract Refusal, serde. Callers: commands.rs, shell.rs (cwd), core tools_bridge.
- **Callers.** `commands.rs::file_read`/`file_tree_list`, `shell.rs::execute` (cwd containment), core `tools_bridge.rs` (jail opened once at boot), and the escape-matrix tests.
- **Tests/evidence.** 9 unit tests: internal read/write/list, six escape vectors (parent/absolute/UNC/drive/
  8.3/symlink-junction via mklink), read cap, nested write — VERIFIED (Phase-6 run).
- **Quality.** Refuse-never-clamp is the right posture; `~` over-refusal is a documented limitation (H-8).
  5/5 within scope.
- **Risks/limits.** Windows-only semantics; case-insensitivity handled by lowercasing (locale-safe for ASCII
  paths); no reparse-point-specific test beyond the junction case.

## backend/crates/hiverouter-tools/src/ledger.rs

- **Metadata:** 13381 B · sha256 `F5878DA79A93C394C780E92FD73E9D03450E2F9584810FAA27B6D5C97164CE5F` · language Rust · category source · depth deep.

- **Mechanics.** Append-only JSONL run ledger: `LedgerOutcome` (8 states, refusing states require a code),
  `LedgerRow`, `LedgerRecord`, `RunLedger::open/open_in/record/read_all`; monotonic `seq`, RFC3339 `ts`,
  `args_sha256` validation (64 hex), permission class + tool + refusal code, never argument bodies.
- **Logic/algorithm.** `record` (180) rejects a refusing outcome without a refusal code, rejects a non-hex
  digest, appends+flushes one JSON line under a mutex, and returns the written row; `read_all` refuses any
  line that fails JSON parsing (corruption is loud, not skipped).
- **Edge/failure behaviour.** Reopening the file appends (no truncation); an empty line is skipped; a row with
  a raw argument body smuggled as digest is rejected by the 64-hex check.
- **Dependencies.** serde/serde_json, std fs, contract ids/Timestamp, crate::tool.
- **Callers.** Core tool execution path (wiring in `tools_bridge`/WS dispatch), tests; ledger is local-only.
- **Tests/evidence.** 4 unit tests (JSONL roundtrip + monotonic seq + no arg body, refusing-outcome code
  required, raw-body rejection, reopen append) — VERIFIED.
- **Quality.** Good integrity checks; no rotation/size cap (long runs grow unboundedly — debt, H-12). 4/5.
- **Risks/limits.** Unbounded growth and no fsync policy beyond flush; concurrent writers serialized only in
  process (single process is the design).

## backend/crates/hiverouter-tools/src/sha256.rs

- **Metadata:** 5379 B · sha256 `8135BCDC28EA121522D793004B75FA4F91467DE76D665A5D9BCBA5BB0314170D` · language Rust · category source · depth standard.

- **Record.** 133 lines. In-house FIPS 180-4 SHA-256 (`sha256_hex`) used for approval `args_sha256` binding;
  no external hash crate (dependency decision documented in ADAPTERS.md). 2 unit tests (NIST empty/abc/
  multi-block vectors; same-args/one-byte-diff) — VERIFIED. Quality 4/5: correct and tested, but a
  hand-rolled crypto primitive is a maintenance/optimization risk versus a pinned `sha2` in a future phase.

## backend/crates/hiverouter-tools/src/shell.rs

- **Metadata:** 16201 B · sha256 `00E19826DC6970B04D4FF158C0574C16D736A20B1A874C3BB982149688EDDE68` · language Rust · category source · depth deep.

- **Mechanics.** Allowlisted child execution: `ShellPolicy` (allowlist, caps 1 MiB/256 KiB, 60 s timeout),
  `ShellRequest`, `ShellOutput`, `execute`, capped pipe reader, kill-on-close Job Object helpers,
  `system_ping_command` (test-only root-cause doc).
- **Logic/algorithm.** `execute` (173) validates the bare-name allowlist, resolves cwd through the jail, spawns
  with `kill_on_drop`, assigns the child to a kill-on-close Windows Job Object, reads stdout/stderr with caps
  and waits inside a timeout; cancellation uses `select!` on the run token; timeout/cancel drop the future
  (killing the child) and return named refusals.
- **Edge/failure behaviour.** Non-allowlisted or path-qualified program → `E_TOOL_NOT_ALLOWED`; non-zero exit
  is data (Ok with exit_code); timeout → `E_TOOL_TIMEOUT`; cancel → `E_TOOL_CANCELLED`; truncated output is
  flagged, never deadlocks (drains while discarding beyond cap).
- **Dependencies.** tokio process/io, tokio-util token, win32job (cfg windows), contract Refusal, jail.
- **Callers.** Core tool execution (worker `shell_exec` after approval), tests.
- **Tests/evidence.** 6 unit tests: allowlist shape, non-allowlisted refusal, echo capture, timeout (system
  ping by absolute path after the BusyBox-shadowing fix), stdout cap, cancellation — VERIFIED.
- **Quality.** Good containment chain; `cargo.exe` on the default allowlist can run arbitrary build scripts,
  so approval is the only barrier (H-10 note). 4/5.
- **Risks/limits.** Job Object was proven by timeout/cancel kills; a memory-ceiling kill is not configured.

## backend/crates/hiverouter-tools/src/tool.rs

- **Metadata:** 19612 B · sha256 `353AB4E67695C7AAD0B1C04E059B4D77F1F7DCBF25FF3DF70E131AE082319765` · language Rust · category source · depth deep.

- **Mechanics.** Tool contract + registry: `PermissionClass`, `AgentTier`, `ToolId` (7 variants, strum),
  `ToolContract` (schemars schema + `additionalProperties:false`), argument structs (spawn/report/finish/
  file_read/file_list/file_write/shell_exec), `ToolRegistry::standard/contract/is_allowed/allowlist/resolve/
  resolve_call/registered_ids/contract_count/grant_count`; 6 unit tests.
- **Logic/algorithm.** Tier allowlists are frozen constants (hyper 3, super 5, worker 6, tool 4 grants); a
  call resolves id → registry membership (`E_TOOL_NOT_FOUND`) → tier grant (`E_TOOL_NOT_ALLOWED`) → schema
  (`E_TOOL_SCHEMA`); mutating/execute/network classes require approval, read-only does not.
- **Edge/failure behaviour.** Empty args fail required-field validation for every tool except file_list;
  unknown fields rejected; unknown tool id refused by name; unknown wire name refused before allowlist check.
- **Dependencies.** schemars (derive), jsonschema (validate), strum (EnumIter), serde, contract Refusal.
- **Callers.** commands.rs, approval requests, core WS dispatch, CLI tools.*, census.
- **Tests/evidence.** 6 unit tests: enum/registry census equality, schema validity + additionalProperties,
  tier allowlists, schema validation, approval-required classes, allowlist membership — VERIFIED.
- **Quality.** Single schema source; strong; `file_list` having all-optional fields is intentional. 5/5.
- **Risks/limits.** Tool schemas and the CLI's `tools describe` share the same JSON (good); prompt claims are
  duplicated in census.rs (H-5).

## backend/crates/hiverouter-tools/src/watcher.rs

- **Metadata:** 7947 B · sha256 `1D894543C17557228CEE3BBC46C60073CF99FB0EEBA79A0EB7B5BAE8F25ED763` · language Rust · category source · depth standard.

- **Record.** 209 lines. `WATCH_DEBOUNCE = 150 ms`, `FsChanged`, `WorkspaceWatcher::start/root/recv_timeout/
  try_recv`, `debounce_loop`, `absorb`; notify 8.2.1 RecommendedWatcher recursive. 2 unit tests: debounce
  (5 writes in 50 ms → exactly 1 notification, warm-up drains registration race) and missing-root refusal —
  VERIFIED. Quality 4/5: debounce thread owns batching, timestamped removal hint; no ignore-list and no
  stop-token (drop-only lifecycle).

## backend/crates/hiverouter-tools/src/bin/hiverouter.rs

- **Metadata:** 579 B · sha256 `6A0FA448CA080530ED96D2EF407EC8C81FE1DEE4816231CF206DD865EB77D057` · language Rust · category source · depth standard.

- **Record.** 11 lines. Thin `main`: collects argv (skip program) and exits with
  `hiverouter_tools::cli::run_main`; artifact named `hiverouter-cli` to avoid colliding with the server
  binary `hiverouter` from core. No logic; process exit code is the CLI's. Quality 5/5.

## backend/crates/hiverouter-tools/src/lib.rs

- **Metadata:** 3397 B · sha256 `406B7D19BCD243281CC7F7F327047A5DDFA0F0919A326B8EBBA10BC88266495D` · language Rust · category source · depth standard.

- **Record.** 76 lines. Module map (approval, audit, cancel, census, cli, cli_support, commands, error,
  health, jail, ledger, sha256, shell, settings, tool, watcher) + re-exports + dependency-direction rule
  (core → tools, never the reverse). `forbid(unsafe_code)`, `warn(missing_docs)`. No logic. Quality 5/5.

## backend/crates/hiverouter-tools/src/settings.rs

- **Metadata:** 2027 B · sha256 `BE807D709B7564019D23B6957159BA75DB450A8DF2957B9CBB1C2117C04140A4` · language Rust · category source · depth standard.

- **Record.** 54 lines. `DEFAULT_MODEL = Model::Flash`, `settings_status()` → contract
  `event::SettingsStatusResult {provider:"deepseek", model:"deepseek-flash", key_present:bool}`,
  `require_provider_key()` → `E_KEY_MISSING`. 2 unit tests (no key material, named refusal). Callers: core
  tools_bridge SETTINGS_STATUS, CLI/MCP. Quality 5/5 (boolean presence only).

## backend/crates/hiverouter-tools/tests/audit_jsonl.rs

- **Metadata:** 2163 B · sha256 `336D4431A0545AE8C231DD1B32593661D647CD6421DB5F0D7426091F5FA61E2A` · language Rust · category source · depth standard.

- **Record.** 53 lines, one integration test: initialises the real JSON sink in its own process, logs a tool
  call + refusal, drops the guard to flush, asserts the rolling file exists, every line parses, the refusal
  row carries `E_CYCLE` and the run id, and the secret probe passes. VERIFIED at Phase-11 (1 passed).
  Purpose: single-owner subscriber + real writer roundtrip without mocks. Quality 5/5.
