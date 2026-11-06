# SEAT S1 — EVIDENCE (Provider Runtime)

Read-only seat; no product/git/config edits, no key access, no paid calls. All writes under `plans/seats/SEAT-01/`.
Repo `C:\Users\Riley\Desktop\hiverouter-v1`; **HEAD observed `5c69b66`** (`git rev-parse HEAD` →
`5c69b66e363e9f1d0801ea5c2e66a1fc12c8943c`), working tree status was not modified by this seat (only new untracked
`plans/seats/SEAT-01/` files). Session clock at `git` call: `2026-11-06T10:59:55-06:00`.

## E-1 Inventory / hash evidence (observed)

Command (read-only): per-file `Test-Path` + `Get-Item.Length` + `Get-FileHash -Algorithm SHA256` + line count over the
assigned slice. Observed: **23/23 assigned local files exist** (2 audit + 8 core-slice + 6 provider + 4 tests + 2
evidence docs + 1 core manifest; the 4 official docs are external), with the hashes/sizes/lines recorded verbatim in
`READLOG.md` rows 1, 6–27. Two files are *missing from the audit paper's world but present at HEAD*:
`backend/crates/hiverouter-core/src/provider_executor.rs` (10,589 B, sha256 `08AD935C…`) and
`backend/crates/hiverouter-core/tests/provider_live.rs` (2,491 B, sha256 `9F24E9B3…`) — both required by this seat and
both absent from the `3988622`-era module diagram (see E-3).

## E-2 Git / tree-state evidence

```
$ git rev-parse HEAD
5c69b66e363e9f1d0801ea5c2e66a1fc12c8943c
$ git log --oneline -5
5c69b66 feat: wire DeepSeek provider into core; live smoke, replay arm, 16-state parity, settings
f4c780d audit: close verifier doc-sync notes (LOC breakdown, roadmap wording already 101)
cf70b22 audit: 507-file exhaustive audit -- paper, 30 findings, quality matrix, 6 diagrams, 101-task roadmap, verification
3988622 closeout: sync FINAL-REPORT with packaged CLI + zip hash; process cleanup
2817326 closeout: final landing checks PASS, CLI packaged, processes cleaned; FINAL-CLOSEOUT evidence
```

This is the seat's **primary truth finding**: the mandated audit paper (`audit/HIVEMIND-AUDIT-PAPER.md`, revision
draft-v1) and `audit/FINDINGS-REGISTER.md` audit **HEAD `3988622`**; the repository has since advanced one commit
(`5c69b66`) whose message is exactly the provider-wiring fix. The paper's headline P0 (`A-F01`) and the stale module
diagram describe the pre-`5c69b66` tree, not the current one.

## E-3 Wiring evidence (current HEAD, READ)

```
backend/crates/hiverouter-core/Cargo.toml:21   hiverouter-provider = { path = "../hiverouter-provider" }
backend/crates/hiverouter-core/src/http.rs:25  use crate::provider_executor::ProviderExecutorAdapter;
backend/crates/hiverouter-core/src/http.rs:67  fn build_executor(config: &Config) -> Arc<dyn ProviderExecutor>
backend/crates/hiverouter-core/src/http.rs:72  match ProviderExecutorAdapter::from_env(config.provider.clone())
backend/crates/hiverouter-core/src/http.rs:101/141/178  RunEngine::new(..., build_executor(&config), ...)
backend/crates/hiverouter-core/src/execution.rs:38  pub trait ProviderExecutor { plan(...); execute(...) }
backend/crates/hiverouter-core/src/provider_executor.rs:32 pub struct ProviderExecutorAdapter
backend/crates/hiverouter-core/src/provider_executor.rs:99 impl ProviderExecutor for ProviderExecutorAdapter
```
`build_executor` falls back to `UnconfiguredExecutor` when the provider is disabled or the key/client is unavailable
(`http.rs:65–82`) — an honest named `E_KEY_MISSING` path, not a fake success. The stale diagram
`audit/diagrams/03-module-dependencies.mmd` lacks the `core → provider` edge that exists at HEAD.

## E-4 Provider/key/budget evidence (READ; no new network call)

- Endpoint/model/auth: `client.rs` builds `https://api.deepseek.com/chat/completions` with Bearer key; tests assert no
  `/v1` and that Debug never prints key bytes (`client.rs:731–751`); `secret.rs` zeroizes the key and is the only
  accessor; `api-docs.deepseek.com` root + pricing + thinking-mode + create-chat-completion pages were **fetched and read
  in full** (HTTP 200) and agree with `dto.rs` constants (base URL, model ids, max_tokens 393,216, usage counters,
  thinking toggles, peak windows).
- Budget enforcement: `budget.rs::preflight` is called before the request is built; refusals are
  `E_USAGE_INCOMPLETE`, `E_CALL_LIMIT`, `E_BUDGET_EXCEEDED`, `E_TOKEN_BUDGET_EXCEEDED`; caps B1–B10 match the blueprint
  values; pricing rows carry `source_url`+`fetched_at`, unsourced rows are refused `E_PRICE_UNSOURCED`.
- Self-attested live evidence (READ, not re-run): `docs/evidence/key-validation.md` records HTTP 200 / `HIVE-KEY-OK` /
  usage 14-6-20; `docs/evidence/live-provider-smoke.md` records a completed live run through the binary at
  `127.0.0.1:8799` with `HIVE-LIVE-OK` and usage 89-6-95, and itself flags single-delta non-streaming + replay UI as
  open gaps. Neither document contains key material. **The seat did not access the key and made no paid call.**

## E-5 Commands executed (complete list; all read-only)

1. `read` of paper, findings register, module diagram, 20 product files, 2 evidence docs (paged reads for the largest
   files: provider client 150–449, budget 200–499, execution 240–499, http 96–325, provider_executor 90–209, plus the
   head/tail pages shown by the tool; all pages needed for the claims were read).
2. `Get-Date`, `git rev-parse HEAD`, `git log --oneline -5`, `git status --porcelain` (untracked seat output only).
3. `Get-Item.Length` / `Get-FileHash -Algorithm SHA256` / line counts over the assigned slice (E-1).
4. `Select-String` over core sources for `ProviderExecutorAdapter|provider_executor|DeepSeekClient|UnconfiguredExecutor`
   and over `Cargo.toml` for `provider|tools` (E-3).
5. `web_fetch` × 4 official docs pages (HTTP 200; two pages returned truncated tails — noted in READLOG #4/#5).
6. `Test-Path plans/seats/SEAT-01` + file listing of `backend/crates/hiverouter-provider/{src,tests}` and
   `docs/evidence` (scope inventory).

**Not executed (deliberate, read-only mandate):** `cargo build`, `cargo test`, any server start, any DeepSeek request,
any git write, any edit outside `plans/seats/SEAT-01/`.

## E-6 Claims classification

| # | Claim | Class |
|---|---|---|
| 1 | Current HEAD is `5c69b66`; paper/register describe `3988622` | **VERIFIED** (git commands above) |
| 2 | Core depends on provider at HEAD; executor adapter wired in all AppState constructors | **VERIFIED** (source lines E-3) |
| 3 | A-F01 "shipped server cannot perform live run" is stale at HEAD | **VERIFIED** (contradicting source at HEAD + commit message) |
| 4 | Provider client targets documented endpoint/model; key zeroized and never logged | **VERIFIED** (source + official docs) |
| 5 | Budget preflight refuses before send; caps B1–B10 implemented | **VERIFIED** (source lines; unit tests in-file) |
| 6 | Leaf execution is non-streaming (one REPLY_DELTA after full completion) | **VERIFIED** (provider_executor.rs:156–159 + evidence doc line 12) |
| 7 | Live HTTP 200 key validation and live run occurred on 2026-11-06 | **READ** (self-attested evidence docs; no raw headers/logs reproduced) |
| 8 | `provider_live` ignored test passes at current HEAD | **UNVERIFIED** — not run (no key use permitted) |
| 9 | Paper byte size / hash | **UNVERIFIED** — not re-hashed (read via tool) |
| 10 | Budget cap numbers are policy, not measured cost facts | **READ** (budget.rs doc header says PROPOSED-POLICY) |
