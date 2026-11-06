# PHASE 11 RECON — Diagnosis, Observability & Verification

Written before Phase 11 implementation edits (2026-11-06 03:41 CST). Blueprint card: Phase 11 (WPs 11.1–11.15).

## Recon
- **Target behavior:** no observability/test-infrastructure code exists yet. This phase adds: the local-only
  JSONL audit sink (tracing-subscriber JSON layer + tracing-appender rolling non-blocking writer; one span
  per run/tool with run_id/tool/permission_class/refusal_code; append-only; audit file probe rejects
  key/token substrings), the `/health` + `/diagnose` axum routes implemented as `hiverouter-tools` handlers
  that core mounts (`add_routes(router, state)`), the derived capability census (strum `EnumIter` over tool +
  refusal enums; counts derived, never typed; test fails on drift), the local test/e2e harness scripts, and
  the criterion bench instrument (dev-only; measured numbers or UNVERIFIED label).
- **Entry point:** `backend/crates/hiverouter-tools/src/{audit,health,census}.rs` + `benches/`;
  `tools/test.ps1` / `tools/e2e.ps1`; docs `docs/plans/PHASE-11-EVIDENCE.md`.
- **Callers:** `hiverouter-core` main/bootstrap installs the audit layer and merges `health::add_routes`
  (core owns the axum Router + `sysinfo` sampling state), Phase 12 CLI calls `hiverouter diagnose`/`health`.
  `hiverouter-core` provides the router builder; this crate must not own a second server.
- **Tests + command:** `cargo test -p hiverouter-tools` from `backend/` + `cargo bench -p hiverouter-tools
  --bench provider_benches` (or the tools benches) + `tools/test.ps1`. Offline only.
- **Data flow (success + one failure path):** event → tracing event with span fields → JSON layer →
  `tracing_appender::rolling::daily` non-blocking writer → `<audit_dir>/audit.*.jsonl` (every line parses as
  JSON, required fields present, no DEEPSEEK_API_KEY substring); `/health` → sysinfo sample (rss_bytes,
  cpu_pct, handle_count; absent sampling → null, never 0) → 200 JSON with named status; `/diagnose` →
  refusal table + census counts + provider key presence boolean (never the key) → 200 JSON.
  Failure path: audit layer write error is surfaced as `E_INTERNAL_ERROR` (never swallowed); key/token
  substring probe fails the audit round-trip test; census test fails when one side of
  registry/allowlist/prompt-claims is edited alone.
- **Unknowns / evidence still missing:**
  - Core Router/AppState pub surface pending B-Core; until then `health::router()` is a standalone
    `axum::Router` exercised over a real ephemeral TCP port in tests, and `add_routes` is the integration
    seam. No second server ships.
  - Bench baselines: no prior baseline exists (new project) → criterion output is recorded as MEASURED on
    this host, and any claim beyond that stays UNVERIFIED (D9/B1). No 60fps/10k claims.
  - D9/W6: performance claims either measured with command+hardware or labelled UNVERIFIED.
- **Files I expect to touch:** `backend/crates/hiverouter-tools/{Cargo.toml,src/{audit,health,census}.rs,
  tests/*.rs,benches/*.rs}`, `tools/*.ps1`, `docs/plans/PHASE-11-{RECON,EVIDENCE}.md`, `docs/claims/*`.
  Never core/contract/web/blueprint.

## Scope check against the blueprint card
- 11.1 local test harness; 11.2/11.3/11.4/11.8 trimmed to local-only (no OTel/Prometheus/cloud CI);
  11.5 JSONL audit; 11.6 health probes; 11.7 in-process only (documented as limited); 11.9/11.12 derived
  census; 11.10 tracing-subscriber JSON; 11.11 test stack is frontend-owned (out of my scope; local e2e
  harness records ownership); 11.13 tracing-appender + sysinfo; 11.14 dev-only criterion/proptest/insta/
  nextest (criterion adopted in this phase; proptest/insta remain optional and are NOT added without a
  decision); 11.15 promptfoo optional dev tool — not adopted.
- Exit-gate mapping: every refusal in the audit log with code; health answers; one-command local test suite;
  census fails when a registry/allowlist entry is removed; benchmark measured or UNVERIFIED.
