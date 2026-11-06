# EVIDENCE — b-provider partition

Read-only audit. No product/tests/docs/git writes, no rebuilds, no installs, no network calls. Only
`audit/parts/b-provider/**` was written. All commands below are `pwsh` + read-only cmdlets
(`Get-Content`, `Select-String`, `Get-FileHash`, `Import-Csv`, `git log/show/diff`).

## 1. Partition and checkpoint

```
$ Get-Date -Format o
2026-11-06T09:39:57.3928361-06:00
$ git rev-parse --short HEAD
3988622                                   # matches audit/WAVE-SHEET.md checkpoint
$ (Get-Content audit/inventory/ASSIGNMENTS.json | ConvertFrom-Json)['b-provider'].Count
33                                        # assigned files (15 deep, 18 standard)
$ Get-Content audit/inventory/DEEP-LIST.md   # 15 b-provider deep files confirmed by name
$ Get-Content audit/WAVE-SHEET.md            # scope/output rules read before any write
```

## 2. Manifest/hash verification

```
$ Import-Csv audit/inventory/FILE-MANIFEST.csv → rows joined to ASSIGNMENTS['b-provider']
33/33 assigned paths present; bytes + sha256 captured for every row; depth/language/category read.
Spot hash re-computation (Get-FileHash -Algorithm SHA256):
backend/crates/hiverouter-provider/src/accounting.rs
  manifest=7BDA95F2A98177ED6E12FBEE898E1B809B1CDA4F392475FA514828A7BD2CC8D0 actual=same match=True
backend/crates/hiverouter-tools/src/approval.rs
  manifest=E32411E62FFEC925A2C2F9FFDCC6BCB6C7BC16FBA42944253DACA70A35652C22 actual=same match=True
backend/Cargo.lock
  manifest=7B704C18E67F56DA0AF26FEB3914DB0AAA841A2B66F2066DE206A915FF7C859A actual=same match=True
```

`FILES.md` metadata was injected mechanically from the manifest (33/33 headers matched; 33 metadata blocks
written) and the final file was re-read to confirm 33 `## ` sections.

## 3. Source-state evidence

```
$ git diff --stat 3206fa3..HEAD -- backend/crates/hiverouter-provider backend/crates/hiverouter-tools backend/Cargo.toml backend/Cargo.lock
(no output)                                # my partition source unchanged since the last B-Provider commit
$ git log --oneline -1
3988622 W…                                 # current audit checkpoint
```

Line counts + import maps collected per file, e.g.:

```
accounting.rs lines=430 pubfns=11 imports: crate, hiverouter_contract, rust_decimal
budget.rs     lines=645 pubfns=26 imports: crate, hiverouter_contract, rust_decimal, std, tokio, uuid
client.rs     lines=709 pubfns=13 imports: crate, hiverouter_contract, reqwest, std, tokio, zeroize
approval.rs   lines=948 pubfns=18 imports: crate, hiverouter_contract, rusqlite, serde, uuid
cli.rs        lines=963 pubfns=4  imports: crate, hiverouter_contract, hiverouter_provider, serde, std
shell.rs      lines=425 pubfns=10 imports: crate, hiverouter_contract, serde, std, tokio, tokio_util
```

Function anchors used for `path:line` citations were collected with `Select-String`, e.g.
`accounting.rs:163 pub fn status`, `accounting.rs:330 fn price`, `budget.rs:176 pub fn spent_usd`,
`budget.rs:235 pub fn preflight`, `client.rs:312 async fn drive`, `client.rs:474 pub async fn stream`,
`pricing.rs:128 pub fn row`, `jail.rs:92 pub fn resolve`, `ledger.rs:180 pub fn record`,
`census.rs:74 pub fn check_census_drift`, `cli.rs:357 pub fn dispatch`, `health.rs:265/267 router`,
`approval.rs` resolve/resolve_with_nonce/verify_grant regions.

## 4. Dependency evidence

```
$ Get-Content backend/Cargo.toml            # 50 lines, 27 exact pins, 4 members, profiles
$ Get-Content backend/crates/hiverouter-provider/Cargo.toml   # 26 lines
$ Get-Content backend/crates/hiverouter-tools/Cargo.toml      # 44 lines
$ regex over backend/Cargo.lock → name = "…" / version = "…"
total packages in lock: 405
duplicate-name count: 26
thiserror = 1.0.69, 2.0.21
tower-http = 0.6.11, 0.7.1
windows 0.61.3/0.62.2 · windows-sys 0.52.0/0.61.2 · rand 0.8.8/0.9.5/0.10.3
rustls 0.23.46 · ring 0.17.14 · aws-lc-rs 1.19.0 · aws-lc-sys 0.47.0 · h2 0.4.20
libsqlite3-sys 0.39.1 · rusqlite 0.41.1 · reqwest 0.13.5 · tokio 1.54.1 · axum 0.8.9
$ Select-String -Path backend/crates/hiverouter-{provider,tools}/src/*.rs -Pattern 'async_trait|async-trait'
(no matches)                               # declared-but-unused dependency finding H-5
```

Cross-partition caller evidence (read-only grep in b-core's partition, cited not modified):

```
tools_bridge.rs:21,32,62,75,99,114,123,133,167,175-177
  use hiverouter_tools::approval::…; ApprovalGate; migrate(conn); commands::{settings_status_command,
  file_tree_list, file_read}; health::diagnostics; ApprovalDecision mapping
```

## 5. Test/verification claims used in FILES.md

Test outcomes cited as VERIFIED come from the last runs on this exact source state (source unchanged since
`3206fa3`, verified by the empty `git diff` above; the audit itself ran no tests to honour the read-only
mandate):

- `cargo test -p hiverouter-provider` → 29 lib + 11 wiremock matrix passed (Phase 5 run).
- `cargo test -p hiverouter-tools` → 77 lib + 1 `tests/audit_jsonl.rs` passed; shell module 6/6 after the
  BusyBox-ping test fix (Phase 6/11 run, commit `3206fa3`).
- `cargo build --release --locked` → exit 0 (Phase 12 evidence, different commit); not re-run here.

Anything not covered by those runs (bench numbers, clippy run, `cargo tree -d` duplicate-major gate,
`cargo deny`) is explicitly **UNVERIFIED** in FINDINGS/QUALITY with the command that would prove it.

## 6. Unreadable/unread files

None. All 33 assigned files are UTF-8 text and were read (some in full, some via targeted regions + line
anchors). No binary/generated file is in this partition (the manifest language column confirms Rust/TOML/
Lockfile only).

## 7. Outputs written (only paths written, per wave sheet)

```
audit/parts/b-provider/FILES.md        33 sections (15 deep × 8 labelled paragraphs, 18 standard records)
audit/parts/b-provider/DEPENDENCIES.md internal graph + cross-partition callers + exact crate/features
audit/parts/b-provider/QUALITY.csv     33 rows × 7 scored columns + evidence notes
audit/parts/b-provider/FINDINGS.md     16 findings (H-1…H-16) with severity/evidence/roadmap
audit/parts/b-provider/EVIDENCE.md     this file
```
