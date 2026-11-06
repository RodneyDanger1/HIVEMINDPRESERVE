# FINDINGS — b-provider partition

HEAD `3988622`. Read-only audit; every finding cites a file:line read in this session. Severity:
HIGH = correctness/security exploitable now; MEDIUM = contract/safety/reliability gap or gate mismatch;
LOW = debt/robustness; INFO = documented deviation. No product or git write was made.

## H-1 — MEDIUM (correctness, money) · silent zero cost on Decimal overflow
`accounting.rs:330` `fn price(...)` returns `.unwrap_or(Decimal::ZERO)`; if `tokens × price_per_mtok` ever
overflows Decimal, the call contributes **0.00** to the settled cost instead of refusing with
`E_COST_MATH_INVALID`. This contradicts V1/V4 ("provider usage is the only billing truth; never fabricate a
number"). Reachability requires an extreme token count (>~10^26), so it is defensive rather than observed,
but the failure mode is exactly the one the budget contract forbids. Fix: return `Option`/`Result` and map
`None` to `E_COST_MATH_INVALID`; add a unit test with `Decimal::MAX`-adjacent inputs. Roadmap: correctness
pass, 0.5 day.

## H-2 — LOW (observability) · incomplete run reports zero spend
`budget.rs:176` `spent_usd()` does `self.accounting.settled_usd().unwrap_or(Decimal::ZERO)`; once a run is
INCOMPLETE, diagnostics/CLI show spend `0` even if earlier calls cost real money. `preflight` refuses via
`incomplete_reason` first, so there is no budget bypass (verified by reading 235-247), but the reported number
is misleading. Fix: expose `spent_usd: Option<Decimal>` (or a separate `spent_known` boolean) and keep the
`None` visible through `/diagnose`. Roadmap: observability pass, 0.5 day.

## H-3 — MEDIUM (authority/security) · nonce-less `resolve` remains a public API
`approval.rs` exports both `pub fn resolve(conn, approval_id, decision, now)` (~line 300 region) and
`resolve_with_nonce`/`ApprovalGate::resolve_command`. `resolve` skips the `APPROVAL_RESOLVE` one-time nonce
check while still consuming the row; any internal caller that uses it bypasses the frozen wire contract's
replay/forgery guard. Current callers are tests and `decide()`, so no live bypass was found, but the
capability is exported. Fix: make `resolve`/`resolve_with_reason` private, or add an explicit
`#[doc(hidden)]` test-only feature and route production through `resolve_command`. Roadmap: authority pass,
0.5 day.

## H-4 — MEDIUM (verification gap) · stream-without-`[DONE]` path untested
`client.rs:528` `pump_stream` maps a stream ending before `data: [DONE]` to `E_PROVIDER_INCOMPLETE`, and
`client.rs:474` retries only before the first byte, but the 11-test wiremock matrix only covers the happy SSE
path (`streaming_client_parses_sse_and_reports_usage_none_as_incomplete`). A truncated/aborted stream is the
exact failure the code exists for and has no executable proof. Fix: add wiremock responses that end mid-SSE
and that close the connection after the first delta; assert `E_PROVIDER_INCOMPLETE` and no duplicate text.
Roadmap: verification pass, 1 day.

## H-5 — LOW (dependency hygiene) · `async-trait` declared and unused in both crates
Provider `Cargo.toml:14` and tools `Cargo.toml:?` declare `async-trait.workspace = true`; a repository-wide
grep for `async_trait`/`async-trait` in both `src/**` trees returns **zero** matches. Unused deps inflate the
lockfile/compile graph and violate the "every pin is used" dependency discipline. Fix: remove both rows (no
code change needed). Roadmap: dependency pass, 0.1 day. (`thiserror` is used only for the `#[error]` derive in
contract; tools/provider error.rs implement Display manually — so provider's `thiserror` is also unused;
verify and remove there too.)

## H-6 — MEDIUM (dependency gate mismatch) · lockfile contains duplicate majors of F2-pinned crates
`backend/Cargo.lock` resolves `thiserror 1.0.69` + `2.0.21` and `tower-http 0.6.11` + `0.7.1` (plus 24 other
duplicate names, mostly windows/rand/getrandom transitive). FACT F9 / Phase-1 gate language says
`cargo tree -d` must show no duplicate major of any F2/F8 crate. Whether the gate actually passes was not
re-verified here (`cargo tree -d` is read-only but not executed in this timebox; no rebuild performed) —
**UNVERIFIED**. The duplication is transitive (wiremock/criterion/axum ecosystem), so the likely truthful fix
is an explicit waiver + lock-diff policy rather than a version bump. Roadmap: dependency decision, 1 day.

## H-7 — LOW (maintainability) · `RunAccounting::status()` dead branch
`accounting.rs:163` computes
`if self.incomplete_reason.is_some() || !self.has_settled && self.calls > 0 && self.settled_usd.is_zero()`
and then, inside that branch, `if self.incomplete_reason.is_none() { return CostStatus::Complete; }`. The
inner early-return makes the second condition redundant and the outer/or-inner structure contradictory;
behaviour happens to be correct (status is INCOMPLETE iff a reason exists). Fix: `if
self.incomplete_reason.is_some() { Incomplete } else { Complete }` + a note that a settled-zero run is still
COMPLETE. Roadmap: cleanup, 0.2 day.

## H-8 — LOW (usability/robustness) · jail refuses every `~` path segment
`jail.rs:124` refuses any component containing `~` to block 8.3 short-name aliases. Consequence: a legitimate
workspace file named e.g. `notes~old.txt` or `data~1` is refused as `E_PATH_OUTSIDE_JAIL`. Refusal is safe
(fail-closed) but over-broad. Fix: narrow the pattern to 8.3-shaped aliases (`^[^~]{1,8}~[0-9]+(\.\w{0,3})?$`)
and keep the explicit test vector. Roadmap: robustness, 0.5 day.

## H-9 — LOW (maintainability/drift) · duplicate NODE/MSRV floor definitions
`cli.rs` defines `NODE_FLOOR = "22.12.0"` and `MSRV = "1.89.0"` as string constants; `cli_support.rs:25,31`
defines the same floors as `(u64,u64,u64)` via `node_floor()`/`rust_msrv_floor()`. Two sources of truth for a
first-run gate can drift. Fix: one `CliFloor` struct/const consumed by both. Roadmap: cleanup, 0.2 day.

## H-10 — LOW (defence-in-depth) · `cargo.exe` on the default shell allowlist
`shell.rs:66` conservative allowlist is `cmd.exe`, `git.exe`, `cargo.exe`. Any `cargo` invocation can run
repository build scripts (`build.rs`) with the server's privileges; the only barrier is the `Execute`
permission class + approval gate. This is within the designed approval model, but worth an explicit threat
note and a narrower default (git read-only subcommands / explicit opt-in for cargo). Roadmap: security
hardening, 1 day.

## H-11 — MEDIUM (integration integrity) · census comparison is tier-blind; prompt claims are a second copy
`census.rs:74` compares the **set** of registry ids to the **union** of allowlist ids; a tool deliberately
granted to the "wrong" tier still passes as long as some tier grants it, so census drift cannot detect
tier-policy regressions (the per-tier exact grants are pinned only by `tool.rs` unit tests). Additionally
`PROMPT_CLAIMED_TOOL_IDS` is a hard-coded Rust const that duplicates what Phase-3 prompt assets claim; two
manual copies can drift silently. Fix: compare per-tier grant vectors against a frozen expected matrix, and
source prompt claims from a single checked manifest. Roadmap: integration invariant, 1–1.5 days.

## H-12 — LOW (data integrity/ops) · run ledger grows without bound
`ledger.rs:142` opens an append-only JSONL file with no rotation/size cap/retention policy; long-running
servers or many runs grow the file indefinitely (and `read_all` reads everything into memory). Local-only
scope makes this non-critical. Fix: size-based rollover (e.g. same daily pattern as the audit sink) or a
documented retention policy + `read_all` pagination. Roadmap: ops, 0.5 day.

## H-13 — LOW (docs-vs-manifest) · reqwest TLS feature name discrepancy
`backend/Cargo.toml` sets reqwest features `["json","rustls","http2"]`; the blueprint/FACT text speaks of
`rustls-tls`. Both may be true across reqwest versions, but the audit could not confirm the 0.13.5 feature
name offline (no docs fetch in this read-only timebox) — **UNVERIFIED**. The resolved lock shows
`rustls 0.23.46` + `aws-lc-rs/ring`, so TLS is present. Fix: reconcile the FACT-REGISTER wording or manifest
comment; no behaviour change proven. Roadmap: docs/dependency pass, 0.1 day.

## H-14 — LOW (authority clarity) · approval gate trusts caller-provided permission class
`approval.rs` `ApprovalGate::request(...)` accepts `permission_class` from the caller instead of deriving it
from `ToolRegistry::contract(tool).permission_class`. A mislabelled call could ask approval for a
Mutate-labelled read tool (harmless) or fail to require approval for a read-labelled mutating tool if the
caller lies. Current core wiring passes the contract class (read in `tools_bridge.rs`), so it is an API-shape
weakness rather than an observed bypass. Fix: derive the class inside the gate from the registry. Roadmap:
authority pass, 0.5 day.

## H-15 — INFO (documented deviation) · `/health` has no `handle_count`
EXT-R5-06 mentions `rss_bytes/cpu_pct/handle_count`; `health.rs` exposes `rss_bytes`, `cpu_pct`, `children`
with null-vs-zero semantics and deliberately omits `handle_count` (no pinned crate surfaces it; adding the
`windows` crate would be a new dependency decision). Already documented in PHASE-11-EVIDENCE; recorded here
for completeness (UNVERIFIED vs blueprint wording).

## H-16 — INFO (justified deviation) · `criterion` inline dev pin
`hiverouter-tools/Cargo.toml` pins `criterion = "=0.8.2"` outside `[workspace.dependencies]`. This is
dev-only, exact-pinned, and the bench is the required D9 measurement instrument; acceptable, but the workspace
"single source of versions" rule is bent for one line. Options: add criterion to workspace deps as a
`[workspace.dependencies]` dev-only row. Roadmap: dependency hygiene, 0.1 day.

## Donor / provenance notes (per harness-code-adapter)

- `wiremock` (MIT/Apache-2.0, dev-only) — HTTP test double at the provider boundary; no product mock.
- `win32job` (MIT/Apache-2.0) — the single containment owner for child processes; `cap-std` spike-gated,
  not in the lock (`FINDINGS` of Phase 6).
- `schemars` + `jsonschema` — schema as single source; `jsonschema` compiled `default-features=false`.
- rusqlite `bundled`, notify 8.2.1, sysinfo 0.40.1, tracing-appender 0.2.5 — all local-only; no cloud egress.
- In-house SHA-256 (no donor) — documented, NIST-tested; a `sha2` dependency decision is a future option.
- No donor source code was pasted; no GPL/AGPL crate appears in this partition's direct dependencies.

## Roadmap summary (for Lead's ROADMAP-TASKS.csv)

| ID | Task | Severity | Est. |
|---|---|---|---|
| H-1 | Decimal overflow → `E_COST_MATH_INVALID` + test | MEDIUM | 0.5 d |
| H-3 | Make nonce-less approval resolve private/test-only | MEDIUM | 0.5 d |
| H-4 | Truncated-SSE `E_PROVIDER_INCOMPLETE` wiremock tests | MEDIUM | 1 d |
| H-6 | Resolve/waive duplicate-major lock entries; re-run `cargo tree -d` gate | MEDIUM | 1 d |
| H-11 | Per-tier census matrix + single prompt-claim source | MEDIUM | 1.5 d |
| H-2 | `spent_usd: Option<Decimal>` end-to-end | LOW | 0.5 d |
| H-5 | Remove unused `async-trait` (both crates) / `thiserror` (provider) | LOW | 0.1 d |
| H-7 | Simplify `RunAccounting::status` | LOW | 0.2 d |
| H-8 | Narrow 8.3 short-name refusal pattern | LOW | 0.5 d |
| H-9 | Single CLI floor constants | LOW | 0.2 d |
| H-10 | Threat note + narrower default shell allowlist | LOW | 1 d |
| H-12 | Ledger rotation/retention + paged read | LOW | 0.5 d |
| H-13 | Reconcile reqwest TLS feature wording | LOW | 0.1 d |
| H-14 | Derive permission class inside approval gate | LOW | 0.5 d |
| H-15 | Decide `handle_count` (crate or documented omission) | INFO | 0.5 d |
| H-16 | Move criterion into workspace dev-dependencies | INFO | 0.1 d |
