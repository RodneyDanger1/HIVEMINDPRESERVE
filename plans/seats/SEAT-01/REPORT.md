# SEAT S1 — REPORT: Provider Runtime (HiveMind 1.0 program)

**Seat:** S1 Provider Runtime · **Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **HEAD read:** `5c69b66`
· **Mode:** read-only (no product/git/config edits; no key access; no paid calls). Deliverables under
`plans/seats/SEAT-01/`. Timebox 30 min; interim READLOG written first, report last.

## 0. Headline truth finding

The mandated `audit/HIVEMIND-AUDIT-PAPER.md` (rev draft-v1) and `audit/FINDINGS-REGISTER.md` audit **HEAD `3988622`**
(paper line 4). The working tree is one commit ahead: **`5c69b66` "feat: wire DeepSeek provider into core; live smoke,
replay arm, 16-state parity, settings"**. Therefore the audit's P0 **A-F01 ("shipped server cannot perform a live model
run") is stale at HEAD** — the provider is now a core dependency and the production executor adapter is wired
(`core/Cargo.toml:21`, `core/src/http.rs:65–101`, `core/src/provider_executor.rs:32–160`). The module diagram
`audit/diagrams/03-module-dependencies.mmd:11–13` likewise lacks the `core → provider` edge. Any landing decision must
re-baseline the paper/register on `5c69b66` (or later) before quoting A-F01/A-F04 as open defects.

## 1. Verified facts (VERIFIED = quoted command/line; READ = source read)

1. **VERIFIED** HEAD is `5c69b66e36…`; `git log` shows the provider-wiring commit above `3988622` (EVIDENCE E-2).
2. **VERIFIED** `hiverouter-core` depends on `hiverouter-provider` (`Cargo.toml:21`) and all three `AppState`
   constructors inject `build_executor(&config)` (`http.rs:101,141,178`).
3. **VERIFIED** `build_executor` (`http.rs:65–82`): provider disabled → `UnconfiguredExecutor`; key/client error →
   warn + `UnconfiguredExecutor`; otherwise `ProviderExecutorAdapter::from_env(config.provider.clone())`.
4. **VERIFIED** `UnconfiguredExecutor` returns `Refusal::KeyMissing` for both `plan` and `execute`
   (`execution.rs:75–100`) — a live run without a key fails by name, never fake success.
5. **VERIFIED** adapter planner call: hardened JSON-DAG prompt (`provider_executor.rs:22–25,108–113`), plan parsed with
   fence stripping (`:124–126,191–200`), invalid JSON → `Refusal::SchemaInvalid`.
6. **VERIFIED** adapter completion path: `CallEstimate` → `client.complete_with_budget(budget, request, &estimate,
   now_unix_secs())` (`provider_executor.rs:75–95`); per-run `RunBudget` map keyed by `RunId` (`:32–36,79–82`).
7. **VERIFIED** adapter leaf path is **non-streaming**: one `ChatRequest`, one completion, `deltas: vec![text]`
   (`provider_executor.rs:150–159`); empty/tool-call-only content → `E_PROVIDER_INVALID_RESPONSE` + warn
   (`:163–189`).
8. **VERIFIED** provider client: documented endpoint `https://api.deepseek.com/chat/completions` (no `/v1`; test
   `client.rs:737–741`), models flash/pro only with unknown-id refusal (`dto.rs:34–70`), SSE parser supports
   `data:`/`[DONE]`/keep-alive (`client.rs:150–186`), 401/402 classified `E_PROVIDER_AUTH` and not retried
   (`client.rs:432–460`).
9. **VERIFIED** key custody: `ApiKey` is `Zeroizing<String>`, `Debug` prints `ApiKey(***)`, one `pub(crate) expose()`
   for the bearer header (`secret.rs:20–81`); tests prove no key in Debug/errors (`secret.rs:87–103`).
10. **VERIFIED** budget: `RunBudget::preflight` runs before the request is built and refuses
    `E_USAGE_INCOMPLETE`/`E_CALL_LIMIT`/`E_BUDGET_EXCEEDED`/`E_TOKEN_BUDGET_EXCEEDED` (`budget.rs:235–284`); caps
    B1 50¢/B2 400k/B3 40 calls/B7 in-flight 8/B8 3 attempts/B9 Retry-After ≤60 s/B10 single key (`:24–52`);
    usage-absent after a call poisons the run (refuses further sends) rather than recording 0 (`:235–242`).
11. **VERIFIED** pricing table carries `source_url`+`fetched_at`; missing provenance → `E_PRICE_UNSOURCED`
    (`pricing.rs:66–72,267–272`); peak windows UTC `[(1,4),(6,10)]` Mon–Fri (`pricing.rs:25`) match the pricing page.
12. **READ** live evidence docs: `docs/evidence/key-validation.md` (HTTP 200, `HIVE-KEY-OK`, usage 14/6/20) and
    `docs/evidence/live-provider-smoke.md` (completed run on 127.0.0.1:8799, `HIVE-LIVE-OK`, usage 89/6/95; declares
    single-delta non-streaming + replay UI as open gaps, line 12). Self-attested prose, no raw request trace.
13. **READ** tests: `core/tests/provider_live.rs` is `#[ignore]` by design (no spend in normal suite) and exercises the
    same adapter; `http_boot` 6 tests, `ws_transport` 13, `run_lifecycle` 6 (test-only MockExecutor). Seat did not run
    them (read-only / no build).
14. **READ** docs cross-check: 4 official pages fetched HTTP 200; base URL, model ids, thinking toggles, `max_tokens`
    1–393216, usage counters, peak windows all agree with `dto.rs`/`config.rs`/`pricing.rs`.

## 2. Provider / key / budget assessment

| Dimension | Verdict | Evidence |
|---|---|---|
| Provider wiring into the binary run path | **wired (fixed at HEAD)** | `Cargo.toml:21`; `http.rs:67,72,101`; `provider_executor.rs:32–160` |
| Endpoint/model/params vs official docs | **compliant** | `client.rs:737–741`; `dto.rs:17,20,23,34–70`; 4 docs pages |
| Key custody (single key, no leak) | **honest** | `secret.rs:20–81,87–103`; `error.rs:136–150`; `client.rs:731–751` |
| No-key behaviour | **named refusal, no fake success** | `execution.rs:75–100`; `http.rs:65–82` |
| Budget preflight before send | **enforced** | `budget.rs:235–284`; `provider_executor.rs:75–95` |
| Usage honesty (absent ≠ 0) | **enforced** | `budget.rs:200–208,235–242`; `usage.rs` invariants |
| Cost arithmetic | **Decimal-only, but A-F06 overflow hole remains** | `pricing.rs` Decimal; register A-F06 (`provider/src/accounting.rs:330`, outside this seat slice, inherited) |
| Streaming UI deltas | **gap: one delta per leaf** | `provider_executor.rs:150–159`; `live-provider-smoke.md:12` |
| Live proof at current HEAD | **READ only, not re-run** | evidence docs; `provider_live.rs` ignored test |

**Assessment:** the provider/key/budget stack is structurally sound and honest at HEAD: documented-only DTO surface,
zeroized key with a single accessor, pre-send budget refusals, refusal-preserving errors, Decimal money, and a
by-name fallback when unconfigured. The remaining substantive provider-runtime gaps are the **non-streaming leaf reply**
and the **unwired prompt-template registry** (below), plus evidence-doc provenance. No paid call was made by this seat.

## 3. Remaining defects (path:line)

| ID | Sev | Defect | Evidence |
|---|---|---|---|
| S1-R1 | P1 | Leaf replies are non-streaming: adapter performs one full completion and returns a single `REPLY_DELTA` (no SSE deltas), so UI "streaming" is a burst. Client already has an SSE parser (`client.rs:150–186`) but the adapter does not use a stream path. | `provider_executor.rs:150–159`; `docs/evidence/live-provider-smoke.md:12` |
| S1-R2 | P1 | Planner empty-content is a hard failure with no retry: terse prompts once produced an empty planner response (evidence doc); adapter now refuses `E_PROVIDER_INVALID_RESPONSE` and the run fails — no second attempt/hardening loop. | `provider_executor.rs:163–189`; `live-provider-smoke.md:12` |
| S1-R3 | P2 | Prompt/persona assets still not loaded by production (A-F04): every constructor passes `TemplateRegistry::default()`, so planner uses the raw prompt and licence gating is test-only. | `http.rs:102,142,179` |
| S1-R4 | P2 | Per-run budgets never evicted: adapter `Mutex<HashMap<RunId,RunBudget>>` grows for the process lifetime (one entry per run). | `provider_executor.rs:35,79–82` |
| S1-R5 | P2 | Engine call cap only counts the planning call (`check_calls(1)`); leaf calls are bounded only by the provider-side `RunBudget` (40 incl. retries), so the two cap systems are not unified and leaf overrun surfaces later. | `execution.rs:477–478` vs `budget.rs:24–52` |
| S1-R6 | P2 | Money overflow silently yields `Decimal::ZERO` (inherited A-F06, violates no-silent-zero rule). | `audit/FINDINGS-REGISTER.md` A-F06 → `provider/src/accounting.rs:330` (outside seat slice) |
| S1-R7 | P2 | Audit paper/register stale: audit HEAD `3988622`; P0 A-F01 contradicted by HEAD wiring; module diagram lacks core→provider edge. | paper line 4; `03-module-dependencies.mmd:11–13`; this report §0 |
| S1-R8 | P3 | Live evidence is self-attested prose (no raw request/response log, no machine-readable transcript), so it cannot be independently verified read-only. | `docs/evidence/key-validation.md:8–12`; `docs/evidence/live-provider-smoke.md:8–11` |
| S1-R9 | P3 | `now_unix_secs()` silently falls back to 0 on pre-epoch system clock (`unwrap_or(0)`), tainting budget time math instead of refusing. | `provider_executor.rs:202–207` |
| S1-R10 | P3 | Tool-call responses are unsupported by the leaf executor (tool-call-only content = invalid response); contract/tool event surface exists but leaf path cannot exercise approval/tool calls from a live model turn. | `provider_executor.rs:163–189` |
| S1-R11 | P3 | Provider caps are local policy (50¢/400k/40 calls) presented in `/diagnose` as configured; they are PROPOSED-POLICY, not measured facts — any landing report must keep that label. | `budget.rs:1–10`; `docs/contracts` plan copy |

## 4. File-by-file plan (Outcome / Current / Decisions / Changes / Failure modes / Tests+commands / Rollback / Out-of-scope)

1. **`backend/crates/hiverouter-core/Cargo.toml`** — *Outcome:* provider stays a first-class core dep. *Current:* path
   dep present (`:21`). *Decision:* keep; no inference/mock crates in product. *Changes:* none now. *Failure:* a future
   revert to `UnconfiguredExecutor`-only must fail a test. *Tests:* `cargo test -p hiverouter-core`. *Rollback:* `git
   revert` of a future commit. *Out-of-scope:* dependency upgrades.
2. **`core/src/config.rs`** — *Outcome:* provider settings validated at boot. *Current:* `ProviderSettings::from_env`
   refuses unknown values; keys never in config. *Decision:* keep env-only key; no config-file secrets. *Changes:* none;
   optionally surface `provider.enabled` + key-present in `/health` (not key value). *Failure:* unknown env value must
   keep refusing, never default. *Tests:* unit tests + `cargo test -p hiverouter-core config`. *Rollback:* n/a (no
   change). *Out-of-scope:* multi-key/rotation (B10 forbids).
3. **`core/src/http.rs`** — *Outcome:* production executor wired once. *Current:* `build_executor` at `:67`; three
   constructors duplicate wiring. *Decision:* single builder seam; keep fallback to `E_KEY_MISSING`. *Changes:* load
   template registry (S1-R3), collapse constructors (debt), expose provider status in `/health`. *Failure:* key absent →
   honest refusal; adapter build error → warn + unconfigured. *Tests:* `cargo test -p hiverouter-core --test http_boot`;
   unreachable-key integration test. *Rollback:* keep `HIVEROUTER_PROVIDER=disabled` flag (already present,
   `config.rs:79–88`) as the kill switch. *Out-of-scope:* adding routes.
4. **`core/src/execution.rs`** — *Outcome:* engine calls provider adapter with cancel-aware trait. *Current:* trait
   `plan/execute(run_id,…,cancel)`; single active run; `check_calls(1)` for planning. *Decision:* unify call accounting
   with provider budget (S1-R5), keep trait object for tests. *Changes:* pass a call-budget handle into leaves; propagate
   provider refusal codes unchanged. *Failure:* plan invalid → refusal; provider error → run failed with named code,
   partial state preserved; cancel → `E_TOOL_CANCELLED`. *Tests:* `--test run_lifecycle`, new wiremock adapter test.
   *Rollback:* revert engine commit; provider flag off. *Out-of-scope:* true SSE plumbing here (owns in adapter).
5. **`core/src/main.rs`** — *Outcome:* boot path wires recovery + provider. *Current:* fail-closed, prints named codes;
   `AppState::with_graph`. *Decision:* keep. *Changes:* none. *Failure:* DB/recovery/bind failures exit non-zero.
   *Tests:* `--test http_boot` binary tests (help/version/non-loopback). *Rollback:* n/a. *Out-of-scope:* service host.
6. **`core/src/checks.rs`** — *Outcome:* boot checks honest. *Current:* MSRV + node check. *Decision:* add optional
   provider reachability? No — no network at boot (cost/permission); only key-present status via diagnostics.
   *Changes:* none. *Failure:* node missing → `E_NODE_MISSING`, server serves degraded. *Tests:* unit tests.
   *Rollback:* n/a. *Out-of-scope:* network probes.
7. **`core/src/ws.rs`** — *Outcome:* commands drive engine/tools with ACK+events. *Current:* dispatch arms; no direct
   provider call. *Decision:* keep provider behind engine seam. *Changes:* none; add WS test for run-completion once
   adapter streaming lands. *Failure:* bad frame → `E_BAD_FRAME`, socket usable; origin/token refused pre-upgrade.
   *Tests:* `--test ws_transport`; live ClientWebSocket smoke. *Rollback:* n/a. *Out-of-scope:* transport changes.
8. **`core/src/provider_executor.rs`** — *Outcome:* production adapter is the single provider seam with budget and
   honest usage. *Current:* non-streaming completion per call; per-run budget map; JSON-DAG planner prompt. *Decision:*
   keep `ProviderExecutor` trait; add SSE streaming path (`client` stream method) and emit ordered deltas; evict
   budgets on run terminal; clamp/repair planner JSON retry once. *Changes:* implement `plan` unchanged; `execute` uses
   stream client and forwards `StreamedReply.deltas`; add `forget(run_id)` on completion. *Failure modes:* empty
   content → `E_PROVIDER_INVALID_RESPONSE`; cancel → `E_TOOL_CANCELLED`; budget refusal propagates unchanged; usage
   absent → `Usage::default` keeps INCOMPLETE semantics upstream. *Tests:* rust unit tests for WirePlan/fences; new
   wiremock-driven adapter test (plan+leaf, assert >1 delta, assert usage); `--ignored provider_live` gated by env.
   *Rollback:* revert adapter commit; set `HIVEROUTER_PROVIDER=disabled`. *Out-of-scope:* tool-call orchestration.
9. **`provider/src/client.rs`** — *Outcome:* documented transport, retry and SSE. *Current:* endpoint + SSE parser +
   401/402 no-retry. *Decision:* add public streaming method if not exposed (verify at implementation), keep retry
   policy. *Changes:* wire adapter to streaming method. *Failure:* timeout/429/5xx classified; Retry-After capped.
   *Tests:* `cargo test -p hiverouter-provider` + wiremock integration. *Rollback:* revert. *Out-of-scope:* Anthropic
   format.
10. **`provider/src/dto.rs`** — *Outcome:* documented-only DTOs. *Current:* strict model enum, documented fields,
    unknown fields ignored. *Decision:* keep. *Changes:* none. *Failure:* undocumented model id refused. *Tests:*
    in-file DTO/stream tests. *Rollback:* n/a. *Out-of-scope:* new API surfaces.
11. **`provider/src/secret.rs`** — *Outcome:* one zeroized key. *Current:* env-only, `ApiKey(***)`. *Decision:* keep;
    never add a file/DB key path without decision. *Changes:* none. *Failure:* missing/empty → `E_KEY_MISSING`.
    *Tests:* in-file plus adapter no-leak tests. *Rollback:* n/a. *Out-of-scope:* key rotation (B10).
12. **`provider/src/budget.rs`** — *Outcome:* pre-send caps + null-vs-zero accounting. *Current:* preflight before
    send; INCOMPLETE poisons run. *Decision:* keep policy constants; fix S1-R6 overflow in accounting (separate seat).
    *Changes:* none in this seat; document caps as policy. *Failure:* each refusal named; no silent zero.
    *Tests:* in-file cap tests + adapter budget test. *Rollback:* n/a. *Out-of-scope:* cost model changes.
13. **`provider/src/pricing.rs`** — *Outcome:* sourced price rows + peak windows. *Current:* provenance required,
    Decimal-only. *Decision:* keep; re-fetch prices only via decision + ledger row. *Changes:* none. *Failure:*
    unsourced row → `E_PRICE_UNSOURCED`. *Tests:* in-file peak/boundary tests. *Rollback:* n/a. *Out-of-scope:*
    Search/Fetch pricing (unused).
14. **`provider/src/error.rs`** — *Outcome:* one refusal identity. *Current:* secret-free wrapper. *Decision:* keep.
    *Changes:* none. *Failure:* never invents codes. *Tests:* in-file. *Rollback:* n/a. *Out-of-scope:* new error enum.
15. **`core/tests/provider_live.rs`** — *Outcome:* opt-in live proof of the same adapter. *Current:* `#[ignore]`,
    requires env key, asserts measured usage and non-empty text. *Decision:* keep opt-in (no spend in CI); add
    assertions for >1 delta after S1-R1 lands. *Changes:* post-fix test update. *Failure:* no key → test asserts
    environment precondition, not fake pass. *Tests:* `cargo test -p hiverouter-core --test provider_live -- --ignored
    --nocapture` (requires key env; paid call — only by authorized operator). *Rollback:* n/a. *Out-of-scope:* CI key
    provisioning.
16. **`core/tests/http_boot.rs`** — *Outcome:* boot surface acceptance. *Current:* 6 tests incl `/diagnose`. *Decision:*
    keep; add provider-status assertion (key present/absent without leaking). *Changes:* small. *Tests:*
    `cargo test -p hiverouter-core --test http_boot`. *Rollback:* revert. *Out-of-scope:* browser e2e.
17. **`core/tests/ws_transport.rs`** — *Outcome:* transport acceptance. *Current:* 13 tests. *Decision:* add a
    live-run-over-WS test using a wiremock adapter (no paid call) to prove streamed deltas. *Changes:* new test only.
    *Tests:* same. *Rollback:* revert. *Out-of-scope:* 24 h soak.
18. **`core/tests/run_lifecycle.rs`** — *Outcome:* engine semantics with test double. *Current:* MockExecutor; caps,
    partial state, cancel. *Decision:* keep mock test-only (allowed double; never in product path). *Changes:* add
    adapter-integration test behind wiremock, separate file. *Tests:* same. *Rollback:* n/a. *Out-of-scope:* provider
    contract tests.
19. **`docs/evidence/key-validation.md`** — *Outcome:* verifiable key validation record. *Current:* prose with HTTP
    200/usage. *Decision:* keep the fact, add scrubber-verified artifacts (status code + redacted response id) as
    machine-readable file in evidence dir; never the key. *Changes:* evidence only (owner: release seat). *Tests:* none.
    *Rollback:* n/a. *Out-of-scope:* key rotation.
20. **`docs/evidence/live-provider-smoke.md`** — *Outcome:* live server smoke record. *Current:* self-attested run
    sequence/completed/usage; declares streaming gap. *Decision:* re-run after S1-R1/S1-R2 fixes on the release binary
    and attach machine-readable transcript + timestamps + sha of binary. *Changes:* evidence doc update. *Tests:*
    smoke script. *Rollback:* n/a. *Out-of-scope:* paid-run authorization.

## 5. UNVERIFIED (must not be quoted as proven)

1. `provider_live.rs` passing at HEAD `5c69b66` — **UNVERIFIED** (not run; no key access / no paid call).
2. Live run behaviour at HEAD (streaming shape, completion time, actual cost) — **UNVERIFIED**; evidence docs are
   READ-only, dated 2026-11-06 and self-attested.
3. `docs/evidence/*.md` git-tracked status and whether they are inside the 507-file manifest — **UNVERIFIED** (seat
   did not run `git ls-files` on them).
4. Paper byte count / hash (READLOG #1) — **UNVERIFIED** exact (read via tool, not hashed).
5. A-F06 overflow still present at HEAD (`accounting.rs:330`) — **READ via register**, file outside seat slice; not
   independently re-read by this seat.
6. Provider-crate test counts / `cargo tree -d` duplicate-major verdict (A-F07) — **UNVERIFIED** in this seat (no build,
   no tree run).
7. Wiremock coverage of the two adapter calls at HEAD — **UNVERIFIED**; provider-crate transport tests were not in the
   seat slice; adapter unit tests cover WirePlan/fence only.
