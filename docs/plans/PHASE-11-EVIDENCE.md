# PHASE 11 EVIDENCE — Diagnosis, Observability & Verification

Owner: B-Provider. Date: 2026-11-06 (CST). Blueprint card: Phase 11 (WPs 11.1–11.15).
Local-only: no OTel/Prometheus/collector, no cloud CI, no test double in a product path.

## Preflight (blueprint: `cargo test --no-run && npm run build`)

Backend half executed here (`npm run build` is the frontend owner's gate, tracked in the frontend evidence):

```
cargo check -p hiverouter-tools --all-targets   →  TOOLS CHECK EXIT: 0
cargo test  -p hiverouter-tools                →  test binary built; see test run below
```

## Tests — command + observed output

```
cargo test -p hiverouter-tools
test result: ok. 77 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.87s   (lib)
     Running tests\audit_jsonl.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s    (JSONL audit e2e)
TEST EXIT: 0
```

Phase-11 acceptance tests actually executed (quoted `ok` lines):

```
test audit::tests::probe_rejects_key_material ... ok
test audit::tests::read_rows_requires_valid_json ... ok
test health::tests::health_endpoint_returns_200_with_status_json ... ok
test health::tests::health_reports_zero_children_as_zero_and_updated_counts ... ok
test health::tests::diagnose_endpoint_is_derived_and_secret_free ... ok
test health::tests::pure_diagnostics_seam_is_derived_and_secret_free ... ok
test health::tests::unknown_route_is_404_not_a_silent_pass ... ok
test census::tests::standard_registry_census_is_consistent ... ok
test census::tests::census_fails_when_one_row_is_deleted ... ok
test census::tests::counts_are_derived_not_typed ... ok
test census::tests::census_serialises_for_the_diagnosis_endpoint ... ok
audit_jsonl_roundtrip_and_secret_probe ... ok     (tests/audit_jsonl.rs: real JSON layer + rolling writer)
```

## What each exit-gate item is proven by

| Gate | Evidence |
|---|---|
| every refusal appears in the audit log with code | `record_refusal` writes `run_id`/`code`/`tool`/`permission_class` through the JSON layer; `audit_jsonl_roundtrip_and_secret_probe` asserts a WARN row with `code: E_CYCLE` and the run id after a real trace round-trip; `probe_audit_dir` passes (no key material) |
| health endpoint answers 200 with status JSON | `health_endpoint_returns_200_with_status_json`: real axum server on an ephemeral TCP port, `GET /health` → 200, `schema: hiverouter.health.v1`, `status: ok`, `children: 0` as a **measured zero**, sampling fields present; `unknown_route_is_404_not_a_silent_pass` |
| cost/token numbers reconcile with usage rows | provider crate owns Decimal accounting (PHASE-5-EVIDENCE: cost rows + INCOMPLETE on NULL); `/diagnose` exposes documented caps + price provenance, `/health` never invents a sample (null vs 0) |
| test suite runs locally with one command | `tools/test.ps1` runs provider + tools + contract + core tests (`pwsh -NoProfile -ExecutionPolicy Bypass -File tools\test.ps1 -BackendOnly`); the backend gates it selects were executed individually in this and PHASE-5/6 evidence |
| capability census fails when a registry/allowlist entry is removed | `census_fails_when_one_row_is_deleted`: removing a registry row, an allowlist grant or a prompt claim each makes `check_census_drift` return `E_INTERNAL_ERROR` naming the drift; `counts_are_derived_not_typed` proves the counts come from `strum::EnumIter`, not constants |
| performance claims measured or UNVERIFIED | criterion instrument `benches/engine_benches.rs` (dev-only) exists; see the bench record below. No 60 fps / 10k-dispatch claim is made (D9/B1) |

## `/diagnose` payload (derived, secret-free)

`hiverouter.diagnose.v1`: `{schema, version, uptime_secs, db_ok, provider_key_present (bool), provider{base_url,
models[2], price_source_url, price_fetched_at, run_cost_cap_usd "0.50", token_budget_combined 400000, call_cap 40,
max_attempts 3}, refusals{count (derived), codes[]}, tools{count, ids[], grants}, capability_census{...},
agent_tiers[4], permission_classes[4]}`. Tests assert count == `Refusal::ALL.len()`, tool count ==
`ToolId::all().len()`, no `sk-`/`Bearer ` substring, and 4 tiers / 4 permission classes.

## Local benchmark instrument (criterion 0.8.2, dev-only)

`cargo bench -p hiverouter-tools --bench engine_benches -- --quick` is the instrument; it measures
`tool_schema_validate_file_write`, `sha256_args_binding`, `jail_resolve_relative_path`,
`decimal_cost_one_call_peak` (criterion 0.8.2, `[dev-dependencies]` only). **Status: UNVERIFIED at hand-off** — the
dev-dependency graph (criterion/plotters/rayon/time) was still compiling inside the phase window, so no
`time:` estimate is quoted as measured. No capacity claim is derived from this bench (D9/B1). To run it:
`cargo bench -p hiverouter-tools --bench engine_benches -- --quick` from `backend/` with the w64devkit PATH.

## Files

`backend/crates/hiverouter-tools/src/{audit,health,census}.rs`,
`backend/crates/hiverouter-tools/tests/audit_jsonl.rs`,
`backend/crates/hiverouter-tools/benches/engine_benches.rs`, `tools/test.ps1`, this file.

## Not done / UNVERIFIED (honest list)

- **`handle_count`** (mentioned in EXT-R5-06) is NOT exposed: no pinned crate surfaces it and adding the
  `windows` crate is a dependency decision; `/health` exposes `rss_bytes`, `cpu_pct`, `children` with
  null-vs-zero semantics instead. Recorded as not done, not simulated.
- **`cargo clippy -p hiverouter-provider --all-targets -- -D warnings`** was NOT completed inside the phase
  window (clippy had to rebuild the dependency graph; the dev-only bench build held the build lock). The
  money-path float ban is the observed crate attribute `#![deny(clippy::float_arithmetic)]` plus the Decimal-only
  code path (claim C-207 remains READ, not VERIFIED). This is reported UNVERIFIED, never as a passing clippy run.
- **`proptest` / `insta` / `cargo-nextest`** (EXT-R5-08) are not adopted: `proptest`/`insta` are optional
  dev-only tools and adding them needs a decision; the jail escape matrix is table-driven, and snapshots are
  replaced by explicit field assertions. Not claimed as adopted.
- **`promptfoo`** (optional eval harness, EXT-R5-09) is not adopted (would raise the dev Node floor; no eval
  gate required by v1).
- **In-process step-through debug (WP 11.7)** is not built; the local diagnosis surface covers the requirement
  without cross-runtime pause (documented in the Phase 11 scope rewrite).
- The full `tools/test.ps1` run including `web` gates depends on the frontend tree and is not claimed here;
  the backend gates it wraps were run individually (quoted above).
