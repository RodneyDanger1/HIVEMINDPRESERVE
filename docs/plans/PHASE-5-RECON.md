# PHASE 5 RECON — Provider Client, Budget & Cost Accounting

Written before Phase 5 implementation edits (2026-11-06 03:41 CST). Blueprint card: Phase 5 (WPs 5.1–5.8).

## Recon
- **Target behavior:** the repo has no DeepSeek provider client yet — `backend/crates/hiverouter-provider`
  is an empty stub (`src/lib.rs` doc comment only) created by the workspace scaffold. This phase creates the
  one DeepSeek client (documented endpoint + F1 DTOs, streaming-capable, timeouts, 429 backoff), the
  pre-send budget core (B1–B12), Decimal-only cost/token accounting (V1–V6: NULL usage ⇒ INCOMPLETE) and the
  offline refusal matrix. No live call is made or permitted (no key read, wiremock only).
- **Entry point:** `backend/crates/hiverouter-provider/src/lib.rs` (new modules `dto`, `secret`, `error`,
  `pricing`, `budget`, `client`, `accounting`). Consumed as a library by `hiverouter-core` (run executor,
  Phase 4) and by the Phase 12 `hiverouter` CLI; no binary in this crate.
- **Callers:** none today (crate unreferenced). Contract dependency is `hiverouter-contract::Refusal` /
  `Usage` / `Timestamp` (observed source at `backend/crates/hiverouter-contract/src/{refusal,usage,timestamp}.rs`);
  `hiverouter-core` will call the pub API once its run executor lands (B-Core owns core; direction is
  core → provider, so this crate must not depend on core).
- **Tests + command:** `cargo test -p hiverouter-provider` from `backend/` with
  `PATH=$USERPROFILE\.cargo\bin;<repo>\tools\w64devkit\bin`. Offline: `wiremock 0.6.5` dev-dependency only;
  no `DEEPSEEK_API_KEY` and no outbound call in any test.
- **Data flow (success + one failure path):** typed `ChatRequest` → `budget::RunBudget::preflight` (cost/token/
  call caps, Decimal) → `Semaphore` in-flight gate (8) → HTTPS POST `https://api.deepseek.com/chat/completions`
  with bearer key read once from `DEEPSEEK_API_KEY` into `Zeroizing<String>` → serde `ChatResponse` DTO
  (`usage: Option<UsageDto>`) → mapping to `contract::Usage` (absent stays absent) → `accounting::RunAccounting`
  (usage NULL ⇒ `INCOMPLETE`, never `$0.00`; else Decimal cost from the F1 price rows × UTC peak window).
  Failure path: HTTP 401/402 → `ProviderAuth` with **0 retries**; 429 → Retry-After (capped 60 s) + jittered
  retry ≤3 attempts → `LlmRateLimit`; socket/read timeout → `ProviderTimeout`; malformed body →
  `ProviderInvalidResponse`; missing key → `KeyMissing`; cap breach before send → `BudgetExceeded` /
  `TokenBudgetExceeded` / `CallLimit` with provider request count 0.
- **Unknowns / evidence still missing:**
  - No live DeepSeek key or budget approval: every latency/cache-hit number stays UNVERIFIED; only the mock
    matrix is evidence for this phase.
  - Streaming `usage` field presence in SSE chunks is NOT DOCUMENTED → parsed only if present; absence keeps
    the run INCOMPLETE (never coerced to 0).
  - `rig-core 0.46.0` is spike-gated and NOT adopted (Phase 5 stays fully custom per blueprint WP 5.5 scope
    rewrite); no Rig spike is claimed, and Rig stays out of the lockfile.
  - MSRV/runtime toolchain: cargo 1.99.1 GNU + gcc 16.2.0 from `tools/w64devkit/bin` (Lead-installed; the
    earlier quarantined archive is not used by this crate).
- **Files I expect to touch:** `backend/crates/hiverouter-provider/{Cargo.toml,src/*.rs,tests/*.rs,benches/*.rs}`,
  `docs/plans/PHASE-5-{RECON,EVIDENCE}.md`, `docs/claims/*`, `docs/contracts/ADAPTERS.md` (if a donor mechanism
  is adapted). Never core/contract/web/blueprint.

## Scope check against the blueprint card
- 5.1/5.2: token metering + ONE-key concurrency limiter/backoff (multi-key rotation deleted).
- 5.3 model client + payload contract; 5.4 B1–B12/V1–V6 pre-send budget; 5.6 zeroize key hygiene;
  5.7 Decimal-only money; 5.8 wiremock refusal matrix (dev-only).
- Exit gate mapping: 4 distinct refusals offline (auth/rate-limit/timeout/invalid-response), budget refusal
  before send (request count 0), NULL usage never settled, cost math reproduces F1 rows incl. UTC peak windows,
  matrix offline + Decimal-only.
