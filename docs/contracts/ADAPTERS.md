# ADAPTERS — donor mechanisms adapted into HiveMind v1

Owner: B-Provider (Phase 5/6/11/12). Written per `harness-code-adapter`: a donor repo is an architecture
source, never a file to paste. Every row records the pinned version, licence, adapter file, the invariant
that was kept, the test that proves it, and the rollback (delete the adapter / revert the edit).

**No donor source code was copied into this repository.** Where a mechanism is "in-house", it was written
from the documented contract (blueprint/research register) and is tested against that contract.

## A1. `wiremock 0.6.5` — HTTP provider test double (DEV-ONLY)

| Field | Value |
|---|---|
| Pin / licence | `wiremock = 0.6.5`, MIT OR Apache-2.0 (FACT F10; dev-dependency) |
| Adapter file | `backend/crates/hiverouter-provider/tests/refusal_matrix.rs` |
| Invariant kept | The provider client sees a real HTTP server on localhost and is exercised through its real request/response path; the test double never links into the release binary and no production branch keys off a test flag. |
| Tests + command | `cargo test -p hiverouter-provider` — 11 matrix tests (401/402 no-retry, 429 retry + Retry-After, timeout, malformed JSON, undocumented 500, NULL usage, budget refusal with 0 requests, success path, SSE stream) |
| Rollback | Delete the test file + the `[dev-dependencies] wiremock` row; product code is untouched (SECTION: no `wiremock` in `[dependencies]`). |
| Known difference | The mock's `Retry-After: 0` keeps the retry timing measurable; the 60 s cap is proven by the pure `retry_after_from_header_value` unit test instead of a 60 s sleep. |

## A2. `win32job 2.0.3` — Windows containment owner (Phase 6)

| Field | Value |
|---|---|
| Pin / licence | `win32job = 2.0.3`, MIT OR Apache-2.0 (FACT F2/F10; `cfg(windows)` product dependency) |
| Adapter file | `backend/crates/hiverouter-tools/src/shell.rs` (`create_kill_on_close_job`, `assign_to_job`) |
| Invariant kept | Exactly one containment owner; every child process is assigned to a kill-on-close Job Object immediately after spawn, so a timeout/cancel/server exit reaps the child and its descendants. No `cap-std`/`process-wrap` second primitive exists in the tree. |
| Tests + command | `cargo test -p hiverouter-tools` — `timeout_terminates_the_child`, `cancellation_returns_named_refusal`, `allowlisted_command_runs_and_captures_stdout`; the jail escape matrix proves path containment separately |
| Rollback | Remove the `cfg(windows)` dependency + the two helper fns; the shell executor still works unjailed but the phase gate then fails (job limits are an acceptance criterion). |
| Known difference | The Job Object is created per execution (not process-wide at boot) so the owner lifetime is the tool call; a boot-wide job would also capture the server process. |

## A3. `schemars 1.2.3` + `jsonschema 0.60.2` — one schema source (Phase 6)

| Field | Value |
|---|---|
| Pin / licence | `schemars = 1.2.3` (MIT), `jsonschema = 0.60.2` (MIT, `default-features = false`) — FACT F2 |
| Adapter file | `backend/crates/hiverouter-tools/src/tool.rs` (`schema_of!`, `ToolContract::validate_args`) |
| Invariant kept | Tool argument schemas are derived once from typed Rust structs (`JsonSchema` derive), patched to `additionalProperties: false`, and validated with the pinned validator; no handwritten second schema copy. |
| Tests + command | `cargo test -p hiverouter-tools` — `every_tool_id_has_exactly_one_contract`, `schemas_are_valid_and_forbid_unknown_fields`, `resolve_call_validates_args_by_name`, census drift tests |
| Rollback | Delete the derive/schema rows; the registry type remains and validation becomes an explicit BLOCKED item (no silent fallback). |
| Known difference | `jsonschema` is compiled with `default-features = false` (per FACT F2) so it cannot pull a second reqwest/rustls; the CLI's `tools.describe` exposes the same schema JSON. |

## A4. `rusqlite 0.41.1` (bundled) — approval persistence (Phase 6)

| Field | Value |
|---|---|
| Pin / licence | `rusqlite = 0.41.1`, `bundled` (MIT) — FACT F2/F5 |
| Adapter file | `backend/crates/hiverouter-tools/src/approval.rs` (DDL + `insert_request`/`resolve*`/`session_allowed`) |
| Invariant kept | Approval rows are committed **before** any side effect; `UNIQUE(nonce)` + single-use `decision` column make replay a named refusal (`E_APPROVAL_REPLAY`); TTL expiry consumes the row as `expired` so a late decision can never execute; the ledger/DB stores only `args_sha256`. |
| Integration contract | Core owns the single SQLite connection: `hiverouter_tools::migrate_approvals(&Connection)` from core's migration path, then the handlers in `commands.rs` receive the connection core passes (`Database::with_connection`). This crate never opens its own connection to the same file. |
| Tests + command | `cargo test -p hiverouter-tools` — `approval_suspend_resume_roundtrip`, `approval_args_binding_refuses_different_arguments`, `approval_replay_is_refused_after_one_execution`, `approval_ttl_expiry_has_zero_side_effects`, `deny_leaves_no_grant`, `allow_session_is_scoped_to_run_tool_and_args`, `wire_approval_resolve_uses_nonce_and_replay_refusal`, `ledger_row_never_stores_argument_body` |
| Rollback | Drop the `approvals` table (migration inverse) and remove the module; the gate then has no owner and the phase gate fails. |
| Known difference | One SQLite file with a `rusqlite` Mutex rather than the rejected `tokio-rusqlite` (F2 explicitly forbids it: `links=sqlite3` conflict). |

## A5. `tracing-subscriber 0.3.23` + `tracing-appender 0.2.5` — local JSONL audit sink (Phase 11)

| Field | Value |
|---|---|
| Pin / licence | `tracing-subscriber = 0.3.23` (env-filter/fmt/json), `tracing-appender = 0.2.5` (MIT) — FACT F2/F10 |
| Adapter file | `backend/crates/hiverouter-tools/src/audit.rs` |
| Invariant kept | One local JSON sink with a rolling non-blocking writer; the guard must live for the process lifetime; refusal rows carry `run_id`/`code`/`tool`/`permission_class`; a file probe refuses key/token substrings. No OTel/Prometheus/collector. |
| Tests + command | `cargo test -p hiverouter-tools` — `probe_rejects_key_material`, `read_rows_requires_valid_json`; end-to-end initialisation probe recorded in PHASE-11-EVIDENCE.md |
| Rollback | Drop the guard, delete the module/rows; core then has no audit sink (phase gate fails). |
| Known difference | Global subscriber is installed once (`try_init` error on a second attempt) — that is the single-owner rule, not a limitation. |

## A6. `sysinfo 0.40.1` — `/health` sampling (Phase 11)

| Field | Value |
|---|---|
| Pin / licence | `sysinfo = 0.40.1` (MIT) — FACT F10 |
| Adapter file | `backend/crates/hiverouter-tools/src/health.rs` (`ServiceState::sample_process`) |
| Invariant kept | null-vs-zero: when sampling is unavailable the fields are `None` (JSON null); a measured zero stays `0`. No fabricated metrics. |
| Tests + command | `cargo test -p hiverouter-tools` — `health_endpoint_returns_200_with_status_json`, `health_reports_zero_children_as_zero_and_updated_counts` (real ephemeral TCP server, no mocks) |
| Rollback | Delete the sampler; `/health` then reports nulls for sampled fields (still honest) or is removed with the router. |
| Known difference | `handle_count` is not exposed: no pinned crate surfaces it and adding a `windows` dependency needs a decision — recorded as a Not-done item in PHASE-11-EVIDENCE.md rather than faked. |

## A7. `notify 8.2.1` — workspace watcher (Phase 12)

| Field | Value |
|---|---|
| Pin / licence | `notify = 8.2.1` (CC0-1.0; stable, not 9.0.0-rc.7) — FACT F2/F9 |
| Adapter file | `backend/crates/hiverouter-tools/src/watcher.rs` |
| Invariant kept | A burst of writes inside one 150 ms window produces exactly one coalesced notification; the watcher is read-only (never writes the workspace). |
| Tests + command | `cargo test -p hiverouter-tools` — `watcher_debounce_five_writes_in_50ms_yield_one_event` (5 writes in 50 ms → 1 notification) |
| Rollback | Delete the module + dependency row; the release phase then has no file-watching surface (documented as not delivered). |
| Known difference | Debounce policy is ours (plain thread + receiving window), not the donor's; the OS backend is the platform default. |

## A8. In-house SHA-256 — approval argument binding (no donor)

| Field | Value |
|---|---|
| Why in-house | F2/F10 pin no hashing crate, and `EXT-R5-04` needs only `args_sha256` for the approval contract; adding an unpinned dependency needs a decision. |
| Adapter file | `backend/crates/hiverouter-tools/src/sha256.rs` |
| Invariant kept | FIPS 180-4 byte-exact digest; the approval request binds the exact argument JSON bytes; the ledger stores the digest only. |
| Tests + command | `cargo test -p hiverouter-tools` — `nist_vectors` (empty/`abc`/448-bit multi-block NIST vectors), `same_arguments_same_digest_and_one_byte_changes_it` |
| Rollback | Delete the module and the two call sites; approval binding would need a pinned hash crate (a dependency decision). |

## A9. Concept ports (no code, no dependency)

| Donor | Mechanism | Where expressed | Proof |
|---|---|---|---|
| LangGraph (docs, MIT) | approval as persisted interrupt with single-use resume | `approval.rs` state machine (`pending → decided`), `E_APPROVAL_REPLAY` on second use | `approval_replay_is_refused_after_one_execution`, `approval_suspend_resume_roundtrip` |
| LangGraph Graph API (docs, MIT) | bounded super-step run graph, cycle refused at validation | consumed by core's Phase 4 (not this crate) | recorded in PHASE-1/4 evidence (B-Core) |
| OpenAI Agents SDK (docs, MIT) | typed handshake frame, parent row authoritative | consumed by core's Phase 3 (not this crate) | recorded in PHASE-1 evidence |
| DeepSeek API (official docs, fetched 2026-11-06) | documented request/response shape, usage counters, peak windows | `dto.rs`, `pricing.rs`, `accounting.rs` | `documented_table_matches_f1_values`, `peak_windows_are_utc_weekday_brackets`, `cost_math_reproduces_every_f1_row`, refusal matrix |

## Rejected duplicates (recorded so they are not re-litigated)

`mockito`/`httpmock` (wiremock is the single mock owner), `cap-std`/`process-wrap` (win32job is the single
containment owner; cap-std stays spike-gated and OUT of the lockfile), `async-openai`/`rig-core` (the custom
client is the single provider owner; Rig spike not performed and not claimed), `opentelemetry` (local JSONL
only), `sha2` (in-house SHA-256 for the one digest need), `tokio-rusqlite`/`sqlx`/`sea-orm` (rusqlite is the
single SQLite owner per F2).
