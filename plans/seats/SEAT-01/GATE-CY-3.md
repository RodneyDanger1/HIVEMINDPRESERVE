# GATE CY-3 — recon → plan → lead (provider slice + tests + evidence docs → report)

Scope of this cycle: `backend/crates/hiverouter-provider/src/{client,dto,secret,budget,pricing,error}.rs`,
`core/tests/{http_boot,ws_transport,run_lifecycle,provider_live}.rs`, `docs/evidence/{key-validation,
live-provider-smoke}.md`. Read-only.

## 1. Recon (skills: forgewright-recon, research, auditor, truthseeker)
- **Target behavior:** does the provider client match the official documented contract (endpoint, model ids, thinking,
  max_tokens, usage), is the key custody honest, is the budget enforced before send, and what evidence exists?
- **Entry points:** `client.rs` (endpoint/streaming/retry), `dto.rs` (wire shapes), `secret.rs` (key), `budget.rs`
  (preflight/caps), `pricing.rs` (source+peak), `error.rs` (refusal identity); evidence docs.
- **Docs cross-check:** base URL/`/chat/completions` (client.rs:737–741 vs docs), models flash/pro (dto.rs:34–55),
  max output 393,216 (dto.rs:23 vs docs `max_tokens` 1–393216), thinking toggles (config.rs:61–63 vs thinking-mode
  guide), peak windows UTC `[(1,4),(6,10)]` Mon–Fri (pricing.rs:25 vs pricing page), usage fields incl cache hit/miss
  (dto.rs tests).
- **Key custody:** `secret.rs` Zeroizing + `ApiKey(***)` + single `expose()`; errors/logs secret-free (tests at
  secret.rs:87–103, error.rs:136–150, client.rs:731–751). No key is present in the seat's environment usage.
- **Budget:** `budget.rs:235–284 preflight` before send; refusals named; B1 50¢/B2 400k/B3 40 calls/B8 3 attempts/
  B9 60 s Retry-After/B10 one key; null-vs-zero report.
- **Evidence:** `docs/evidence/key-validation.md` HTTP 200 `HIVE-KEY-OK` usage 14/6/20; `docs/evidence/
  live-provider-smoke.md` live completed run `HIVE-LIVE-OK` usage 89/6/95, and self-declares single-delta + replay UI
  gaps (line 12). Both self-attested prose; no raw request log available.
- **Tests:** `provider_live.rs` is `#[ignore]` (no spend in normal suite); adapter tests in provider_executor.rs
  (WirePlan parse, fence strip) are unit-level; no wiremock assertion visible inside the seat slice for the adapter's
  two calls (client tests in provider crate cover transport separately at crate level).
- **Unknowns:** live run at current HEAD (UNVERIFIED, no key use allowed); exact git-tracked status of evidence docs.

## 2. Plan (skills: forgewright-plan, integration-logic, capability-auditor)
- **Outcome:** provider/key/budget assessment + remaining defects + file-by-file plan + UNVERIFIED list, with the
  stale-paper truth finding foregrounded.
- **Acceptance:** docs↔code agreement table; refusal/budget paths proven; live evidence classified READ (not
  VERIFIED); every remaining defect has `path:line`; no product writes.
- **Steps:** write READLOG rows; write EVIDENCE; write GATE-CY-1..3; write REPORT; final receipt.
- **Risks:** over-claiming the live evidence; mitigated by READ classification and explicit UNVERIFIED list.

## 3. Lead verdict (skills: forgewright-lead, auditor, truthseeker, forgewright-plan)
- **Cycle result: PASS with corrections.** Provider runtime is wired and documented-contract-aligned at HEAD; budget
  preflight is before send; key custody is honest. **The mandated audit paper's P0 A-F01 is stale at HEAD** — the
  current tree includes the provider-wiring commit `5c69b66` (paper audits `3988622`); the consolidated register and
  module diagram must be re-baselined before they are used as landing evidence. Remaining defects S1-R1…S1-R11 are
  recorded in REPORT with line anchors. No blocker; seat deliverables complete.
- **Evidence:** READLOG #16–#27; EVIDENCE E-3/E-4; REPORT §3/§4.
