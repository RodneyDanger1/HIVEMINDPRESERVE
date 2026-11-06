# DEPENDENCIES — b-provider partition

HEAD `3988622`; workspace root `backend/Cargo.toml`; all versions read from `backend/Cargo.toml` /
`backend/Cargo.lock` (405 resolved packages) and the two member manifests. No rebuild performed.

## 1. Partition-internal call/import graph

```
hiverouter-provider (lib hiverouter_provider)
  lib.rs  ── dto.rs, secret.rs, error.rs, pricing.rs, budget.rs, accounting.rs, client.rs
  client.rs ── dto.rs (ChatRequest/ChatResponse/StreamChunk/Model)
             ── error.rs (ProviderError)
             ── secret.rs (ApiKey)
             ── budget.rs (InFlightLimiter, RetryPolicy, RunBudget, CallEstimate,
                           retry_after_from_header_value)
             ── pricing.rs (via accounting.rs for cost rows, peak_at_unix)
             ── accounting.rs (RunAccounting + cost math)
  accounting.rs ── pricing.rs (PricingTable, Peak, peak_at_unix), hiverouter_contract::Usage
  budget.rs     ── accounting.rs (RunAccounting/CallCost/CostReport), pricing.rs,
                   hiverouter_contract::Refusal, uuid (jitter), tokio::sync::Semaphore
  pricing.rs    ── dto::Model, hiverouter_contract::Refusal (errors)
  dto.rs        ── hiverouter_contract::Usage
  error.rs      ── hiverouter_contract::Refusal
  secret.rs     ── zeroize::Zeroizing, hiverouter_contract::Refusal

hiverouter-tools (lib hiverouter_tools + bin hiverouter-cli + bench engine_benches)
  lib.rs ── every module below
  tool.rs      ── schemars (derive), jsonschema (validate), strum, serde, contract::Refusal
  approval.rs  ── rusqlite, uuid (v7 nonce/id), serde, crate::{sha256,error,tool},
                  contract::{ApprovalId,RunId,TaskId,Timestamp,Refusal}
  jail.rs      ── std::fs, contract::Refusal, serde, crate::error
  shell.rs     ── tokio::process/io, tokio_util::sync::CancellationToken, win32job (cfg windows),
                  contract::Refusal, crate::{error,jail}
  cancel.rs    ── tokio_util::sync::CancellationToken, contract::{RunId,TaskId,Timestamp,Refusal}
  ledger.rs    ── serde/serde_json, std::fs, contract ids/Timestamp, crate::{error,tool}
  audit.rs     ── tracing-subscriber (JSON layer), tracing-appender (rolling non-blocking),
                  contract::{Refusal,RunId}, crate::{error,tool}
  census.rs    ── strum (EnumIter), serde, contract::Refusal, crate::{error,tool}
  health.rs    ── axum (Router/Json/State), sysinfo 0.40.1, serde,
                  hiverouter_provider::{pricing, DEFAULT_BASE_URL, constants},
                  contract::Refusal, crate::{census,error,tool}
  commands.rs  ── contract::{command,event}, rusqlite, crate::{approval,error,jail,settings,sha256}
  settings.rs  ── hiverouter_provider::{ApiKey,Model}, contract::{event,Refusal}, crate::error
  cli.rs       ── contract::{Refusal,Timestamp,Usage}, hiverouter_provider
                  {run_cost_cap_usd,ApiKey,Model,PricingTable,RunAccounting,CALL_CAP,
                   DEFAULT_BASE_URL,MAX_ATTEMPTS,TOKEN_BUDGET_COMBINED},
                  crate::{census,cli_support,tool}
  cli_support.rs ── std::process, tokio runtime, reqwest
  watcher.rs   ── notify 8.2.1, std::sync::mpsc, contract::Refusal, crate::error
  sha256.rs    ── std only (no external hash crate)
  bin/hiverouter.rs ── hiverouter_tools::cli
  tests/audit_jsonl.rs ── hiverouter_tools (real sink), contract::Refusal/RunId
  benches/engine_benches.rs ── criterion, hiverouter_provider::{Model,PricingTable,RunAccounting},
                               hiverouter_tools::{sha256_hex,AgentTier,ToolRegistry,WorkspaceJail}
```

No dependency edge from provider or tools to `hiverouter-core` (enforced direction: core → tools/provider).
The tools crate has one internal edge to provider (`hiverouter-provider = { path = "../hiverouter-provider" }`);
provider has no edge to tools.

## 2. Cross-partition callers (owned by b-core, cited read-only)

| Caller (b-core file:line) | Consumes |
|---|---|
| `hiverouter-core/src/tools_bridge.rs:21,23` | `hiverouter_tools::approval::ApprovalResolution`, `ToolError` |
| `tools_bridge.rs:32,62,99,175-177` | `approval::ApprovalGate`, `ApprovalDecision` mapping |
| `tools_bridge.rs:75` | `approval::migrate(conn)` (approval DDL on core's single SQLite connection) |
| `tools_bridge.rs:114` | `commands::settings_status_command()` |
| `tools_bridge.rs:123,133` | `commands::file_tree_list(jail)`, `commands::file_read(jail, payload)` |
| `tools_bridge.rs:167` | `health::diagnostics(db_ok, key_present)` for `/diagnose` |
| `hiverouter-core/src/http.rs` (+ ws/execution) | tools command dispatch, provider client (see b-core FILES) |

Contract crate (`hiverouter-contract`) is the shared wire vocabulary both crates import; provider consumes
`Refusal/Usage/Timestamp/Model-adjacent ids`; tools consumes `command/event/ids/Refusal`.

## 3. External crates in use (exact versions + features)

### Provider crate (direct)

| Crate | Version | Features declared | Use site |
|---|---|---|---|
| tokio | 1.54.1 | `rt,time,sync,macros` (+dev rt-multi-thread) | client/budget (semaphore, sleep, mpsc) |
| reqwest | 0.13.5 | workspace: `json,rustls,http2` | client HTTP/HTTPS/SSE |
| serde | 1.0.229 | `derive` | dto/http bodies |
| serde_json | 1.0.145 | `preserve_order` | dto JSON, tests |
| thiserror | 2.0.21 | — | (workspace) error Display impls |
| tracing | 0.1.44 | — | retry warnings |
| rust_decimal | 1.43.0 | `serde-with-str` | pricing/accounting (Decimal-only money) |
| uuid | 1.29.0 | `v7,serde` | retry jitter entropy |
| zeroize | 1.9.1 | `derive` | ApiKey Zeroizing |
| async-trait | 0.1.89 | — | **declared, zero uses** (FINDINGS H-5) |
| hiverouter-contract | path 0.1.0 | — | Refusal/Usage/Timestamp |
| dev: wiremock | 0.6.5 | — | offline refusal matrix (dev-only) |
| dev: tracing-subscriber | 0.3.23 | `env-filter,json` | **declared dev-dep, no current use** (info) |

### Tools crate (direct)

| Crate | Version | Features declared | Use site |
|---|---|---|---|
| tokio | 1.54.1 | workspace all (rt-multi-thread…io-util) | shell/health/CLI runtime |
| axum | 0.8.9 | `ws` | health router |
| reqwest | 0.13.5 | workspace `json,rustls,http2` | CLI `health`/`diagnose` |
| serde / serde_json | 1.0.229 / 1.0.145 | `derive` / `preserve_order` | rows/events/CLI envelopes |
| thiserror | 2.0.21 | — | ToolError Display |
| tracing | 0.1.44 | — | audit events |
| tracing-subscriber | 0.3.23 | `env-filter,json` | audit JSON layer |
| tracing-appender | 0.2.5 | — | rolling non-blocking JSONL file |
| sysinfo | 0.40.1 | — | /health rss/cpu sample |
| schemars | 1.2.3 | — | tool argument schemas |
| jsonschema | 0.60.2 | `default-features=false` | tool arg validation |
| strum | 0.28.0 | `derive` | EnumIter census |
| uuid | 1.29.0 | `v7,serde` | approval ids/nonce |
| async-trait | 0.1.89 | — | **declared, zero uses** (FINDINGS H-5) |
| rusqlite | 0.41.1 | `bundled` | approval persistence (core passes connection) |
| tokio-util | 0.7.20 | `rt` | CancellationToken |
| notify | 8.2.1 | — | workspace watcher |
| win32job (cfg windows) | 2.0.3 | — | kill-on-close Job Object |
| dev: tempfile | 3.23.0 | — | jail/ledger/audit tests |
| dev: criterion | **0.8.2 (inline pin, NOT workspace-inherited)** | — | bench instrument only |

### Notable transitive resolution (from `backend/Cargo.lock`)

- TLS stack: `rustls 0.23.46` + `ring 0.17.14` + `aws-lc-rs 1.19.0` + `aws-lc-sys 0.47.0` (C build; gcc
  required), `rustls-platform-verifier`, `h2 0.4.20`, `ipnet 2.12.2`.
- SQLite: `rusqlite 0.41.1` → `libsqlite3-sys 0.39.1` (bundled C).
- Workspace-shared: `hiverouter-contract 0.1.0` path package; `tokio-util 0.7.20`; `tower-http` resolves
  **both 0.6.11 (transitive) and 0.7.1 (workspace pin)**; `thiserror` resolves **both 1.0.69 and 2.0.21**.
- Dev-only transitive tree for tests/benches: `wiremock 0.6.5` (deadpool, futures, assert-json-diff),
  `criterion 0.8.2` (plotters, rayon, tinytemplate, ciborium).

## 4. Local/partition seams and their contracts

- provider → contract: `Refusal` is the only error identity; `Usage` absent-stays-absent; `Timestamp` for
  ETAs/row times.
- tools → contract: `command::{ApprovalResolve,SettingsStatus,FileTreeList,FileRead}` payloads and
  `event::{SettingsStatusResult,FileTreeResult,FileContent}` are produced/consumed 1:1 (commands.rs).
- tools → provider: pricing/accounting/F1 caps/`ApiKey` presence feed `/diagnose` and CLI cost.
- tools → core (inverted): core calls `migrate`, `ApprovalGate`, `commands::*`, `health::diagnostics`; the
  tools crate never imports core.
- tools → OS: win32job Job Object (process containment), rusqlite file (approval rows), notify watcher,
  sysinfo sampling, tracing-appender file sink. All local; no network egress except provider base URL in the
  CLI/server provider path.
