# PHASE 1 EVIDENCE — Contract, Toolchain & Boot

Date: 2026-11-06 (America/Chicago). Executor: B-Core. Recon: `PHASE-1-RECON.md`.
Every command below was run from `C:\Users\Riley\Desktop\hiverouter-v1\backend` with
`$env:PATH="$env:USERPROFILE\.cargo\bin;C:\Users\Riley\Desktop\hiverouter-v1\tools\w64devkit\bin;"+$env:PATH`,
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_INCREMENTAL=0`.

## 1. Preflight (blueprint Phase 1 card)

Command: `rustc --version; cargo --version; node --version; & 'C:\Program Files\nodejs\npm.cmd' --version`

```
rustc 1.99.1 (58cf8f9c3 2026-10-13)
cargo 1.99.1 (5f94df478 2026-08-27)
v26.7.0
11.19.0
```

Note: `npm --version` through `npm.ps1` is blocked by PowerShell execution policy on this host
(`File C:\Program Files\nodejs\npm.ps1 cannot be loaded because running scripts is disabled on this system`);
the observed npm version above comes from `npm.cmd`. Node floor `>= 22.12.0` is satisfied by v26.7.0.

## 2. Toolchain security audit (environment incident)

| Check | Observed | Verdict |
|---|---|---|
| Earlier temp download `%TEMP%\w64devkit.zip` | `Get-MpThreatDetection` → ThreatID `2147714384`, InitialDetectionTime `11/6/2026 3:14:19 AM`, `ActionSuccess=True`, resource `file:_C:\Users\Riley\AppData\Local\Temp\w64devkit.zip` | quarantined before use; file deleted, nothing extracted from it |
| Official replacement source | `https://github.com/skeeto/w64devkit/releases/tag/v2.11.0` (asset `w64devkit-x64-2.11.0.7z.exe`) via GitHub API `releases/latest` | official GitHub release only |
| Asset integrity | local size `68,801,240` bytes = API size; local SHA256 `54C2BB9962E8A30C8270E34C5EFC3D5FEE579F53087FA7D435B95FBBA2AF7B2B` = API digest `sha256:54c2bb99…b2af7b2b` | match |
| Authenticode | `Get-AuthenticodeSignature` → `Valid` | signed |
| Extraction + smoke | `gcc --version` → `gcc.exe (GCC) 16.2.0`; C hello compiled (`cc_exit=0`, printed `gcc-ok`, `run_exit=0`) | toolchain works |
| Defender after official install | no new threat detection for the official asset; only the old temp zip row remains | official asset accepted |

### 2a. Packaging defect and documented workaround (host toolchain, not product code)

w64devkit 2.11 / GCC 16.2.0 does not ship `libgcc_eh.a`, which the Rust GNU target requests on every link:

```
$ gcc -print-file-name=libgcc_eh.a
libgcc_eh
error: linking with `x86_64-w64-mingw32-gcc` failed: exit code: 1
  = note: ...ld.exe: cannot find -lgcc_eh: No such file or directory
```

Workaround applied once on this host: `ar rcs tools/w64devkit/lib/gcc/x86_64-w64-mingw32/16.2.0/libgcc_eh.a`
(empty archive; GCC 16 keeps the unwinder in `libgcc.a`). Verification:

```
$ ar rcs .../libgcc_eh.a                     ; ar_exit=0
$ rustc hm-rust-link-test.rs -o hm-rust-link-test.exe   ; rustc_exit=0
$ .\hm-rust-link-test.exe                     ; link-ok / run_exit=0
```

## 3. Manifest corrections (FACT-REGISTER F2 API-diff checkpoint)

- Workspace root is `backend/Cargo.toml` with all four members (`crates/hiverouter-contract`,
  `crates/hiverouter-core`, `crates/hiverouter-provider`, `crates/hiverouter-tools`); `backend/Cargo.lock` committed
  (93,521 bytes).
- `reqwest 0.13.5` no longer has the `rustls-tls` feature (the 0.12 name). Observed:
  `error: ... package 'hiverouter-provider' depends on 'reqwest' with feature 'rustls-tls' but 'reqwest' does not have that feature.`
  Workspace pin corrected to `features = ["json", "rustls", "http2"]`; resolution then passed
  (`cargo check -p hiverouter-contract` exit 0).
- Contract patch (Lead decision): five new frozen commands (`APPROVAL_RESOLVE`, `SETTINGS_STATUS`, `FILE_TREE_LIST`,
  `FILE_READ`), three new events (`SETTINGS_STATUS_RESULT`, `FILE_TREE_RESULT`, `FILE_CONTENT`), five new refusal codes
  (`E_APPROVAL_NOT_FOUND`, `E_APPROVAL_DECISION_INVALID`, `E_PATH_OUTSIDE_JAIL`, `E_PATH_NOT_FOUND`,
  `E_FILE_TOO_LARGE`). `DIFF_*` stays decision-gated (`E_DECISION_REQUIRED`).

## 4. Contract acceptance (blueprint: `cargo test contract_schema_roundtrip`)

Command: `cargo test -p hiverouter-contract`

```
running 18 tests
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.41s

running 3 tests        (golden_schemas.rs)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

running 6 tests        (schema_roundtrip.rs)
test contract_schema_roundtrip ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s

running 5 tests        (ui_command_matrix.rs)
test every_frozen_command_has_an_ack_path ... ok
test every_frozen_command_has_a_named_refusal_path ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s
CONTRACT_TEST_EXIT=0
```

32 tests pass. Golden schema snapshot: `backend/crates/hiverouter-contract/schemas/contract-v1.json`, 48,340 bytes.
`cargo check -p hiverouter-contract` → `CONTRACT_CHECK_EXIT=0`.

## 5. Core acceptance

Command: `cargo test -p hiverouter-core`

```
running 19 tests       (lib: db/db migrations, checks, config, bus, graph)
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.74s
running 0 tests
test result: ok. 0 passed
running 7 tests        (graph_authority.rs)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.21s
running 5 tests        (http_boot.rs)
test binary_refuses_a_non_loopback_bind_by_name ... ok
test health_endpoint_reports_boot_checks_and_database ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.27s
running 9 tests        (ws_transport.rs — Phase 2)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.33s
CORE_TEST_EXIT=0
```

40 tests pass. `cargo check -p hiverouter-core` → `CORE_CHECK_EXIT=0`.

## 6. Boot + `/health` live smoke

Command: `cargo build -p hiverouter-core --bin hiverouter; start target\debug\hiverouter.exe --bind 127.0.0.1:8092 --data-dir %TEMP%\hm-smoke2; GET /health`

```
BINARY_BUILD_EXIT=0
HEALTH_HTTP=200
{"status":"ok","version":"0.1.0","schema":"hiverouter.v1","uptime_secs":1,
 "checks":[{"name":"rustc","required":true,"ok":true,"detail":"rustc 1.99.1 (58cf8f9c3 2026-10-13) (MSRV 1.89.0)","code":null},
           {"name":"node","required":false,"ok":true,"detail":"v26.7.0 (>= 22.12.0)","code":null}],
 "database":{"path":"C:\\Users\\Riley\\AppData\\Local\\Temp\\hm-smoke2\\hiverouter.db","schema_version":1,"error_code":null},
 "transport":{"last_seq":0,"graph_nodes":0,"allowed_origins":2}}
NOPE_HTTP=404
```

Server stdout (quoted): `hiverouter 0.1.0 listening on http://127.0.0.1:8092 (db ...\hiverouter.db, schema v1, contract hiverouter.v1)`.

## 7. Error paths exercised

- Non-loopback bind → exit non-zero + `E_ORIGIN_REJECTED` on stderr (`binary_refuses_a_non_loopback_bind_by_name` ok).
- Unknown route → honest 404 (observed `NOPE_HTTP=404`; `unknown_route_is_an_honest_404` ok).
- Bad frame JSON / unknown command / wrong schema → typed refusals (`bad_frames_are_refused_by_name_never_dropped` ok,
  `E_BAD_FRAME` / `E_SCHEMA_INVALID` / `E_COMMAND_UNKNOWN`).
- `--help`/`--version` exit 0 (`binary_help_and_version_are_successful` ok).
- `FILE_READ` path / `APPROVAL_RESOLVE` decision validation (`file_read_requires_a_path`,
  `approval_decision_values_are_frozen` ok; frozen decision vocabulary is `approve`/`deny`).

## 8. Workspace-wide gate status (honest, not claimed green)

`cargo check --workspace` at this point exits 101 because `hiverouter-tools` (owned by B-Provider, in progress) fails to
compile:

```
error: could not compile `hiverouter-tools` (lib) due to 1 previous error    (E0433; crates/hiverouter-tools/src/cli.rs)
```

Therefore `cargo build --release --locked` remains **UNVERIFIED** and is re-run at the Phase 2/4 gates once the external
crate compiles. Per-crate gates for this phase are green (sections 4-6).

## 9. Done / Not done / Unverified

| Done | Evidence |
|---|---|
| Frozen contract crate (envelope, 17 commands, 21 events, 60+ named refusals, uuid-v7 ids, RFC3339 ts, schema_version, golden schemas, matrix) | §4, 32 tests, exit 0 |
| Pinned manifests + committed `backend/Cargo.lock`; reqwest 0.13 API-diff corrected | §3 |
| `hiverouter-core` binary with boot, `GET /health`, `/api/session`, SQLite bootstrap + migration engine, MSRV/Node checks | §5, §6 |
| Named refusal table with unique codes and retryable/how_to_fix | §4 (`codes_are_unique_and_well_formed`) |
| MSRV gate (build.rs + `rust-version = 1.89.0`) and Node floor check | §1, §6 (rustc 1.99.1, MSRV 1.89.0; node v26.7.0) |

| Not done (reason) | |
|---|---|
| `cargo check --workspace` / `cargo build --release --locked` | blocked by external `hiverouter-tools` compile error (not this phase's code) |
| `npm run build` / `vite build` | web scaffold owned by frontend teammate; not present in this phase |
| `rig-core =0.46.0` spike | Phase 5/6 concern (B-Provider); not implemented by core |

| Unverified | What would prove it |
|---|---|
| release build from pinned lockfile | `cargo build --release --locked` exit 0 after tools compiles |
| rustls TLS path compiles for provider | `cargo check -p hiverouter-provider` (B-Provider gate) |

## 10. Addendum — embedded web UI serving (integration blocker, added after the Phase 1 gate)

The single Rust binary now serves the built frontend (`web/dist`) from memory via `rust-embed`
(`backend/crates/hiverouter-core/src/static_ui.rs`): `GET /` → `index.html`, real asset paths → embedded bytes with
content type, SPA routes (no extension) → `index.html`, unknown API paths → JSON 404, missing assets → 404, and
path-traversal requests cannot escape `web/dist` (exact archive lookup; the test asserts no file leak).

```
Running tests\http_boot.rs
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.48s
```

The new test `static_ui_serves_index_assets_and_spa_fallback` asserts `GET /` 200 + `<div id="root">`, a real
`/assets/index-*.js` 200, SPA fallback 200, missing asset 404, `/api/nope` JSON 404 and traversal safety.

After the disk-wipe rebuild the full core suite (Phases 1-9 + static UI) is:

```
running 34 tests (lib)          test result: ok. 34 passed; 0 failed
chat_store 5 / context_switch 2 / graph_authority 7 / handshake 2 / http_boot 5
kill_restore 2 (+1 ignored helper) / prompt_assets 2 / run_lifecycle 6 / snapshot_history 4 / ws_transport 9
CORE_TEST_EXIT=0
```

= 78 passed, 0 failed, 1 ignored helper; contract suite unchanged at 32 passed.

### 10a. Release binary probe (embedded UI + APIs)

`cargo build --release --locked -p hiverouter-core --bin hiverouter` → `Finished release profile ... in 24m 24s`,
`RELEASE_BUILD_EXIT=0` (that build produced `target/release/hiverouter.exe` = 8,354,816 bytes,
sha256 `B763B8F0B0FBD612D7DB43DD1B4587C4BF5EA0B65F83ACA3C4DE2FEB476DA714`). The shared workspace release build later
replaced the artifact; the binary probed below is `target/release/hiverouter.exe` = 9,099,776 bytes,
sha256 `8C777A786FACE7C586910E8BD854B8C5AADF1C3765FE4DD7D1A11503C8EC3827` (same source tree, shared build).

Started with `--bind 127.0.0.1:8812 --data-dir %TEMP%\hm-release-probe2`, observed (curl, explicit timeouts):

```
GET /                    -> 200   (body starts "<!doctype html>..." and contains <div id="root"></div>)
GET /health              -> 200   {"status":"ok","version":"0.1.0","schema":"hiverouter.v1", ... "database":{"schema_version":3,...}}
GET /api/session         -> 200   {"token":"...","schema":"hiverouter.v1","last_seq":0,"ws":"/ws"}
GET /api/history/runs    -> 200   {"runs":[]}
GET /assets/index-_C6E8rly.js -> 200 (669,129 chars of embedded JS served from the binary)
GET /assets/does-not-exist.js -> 404
GET /runs/example        -> 200   (SPA fallback serves index.html)
server stdout: hiverouter 0.1.0 listening on http://127.0.0.1:8812 (db ...\hiverouter.db, schema v3, contract hiverouter.v1)
```

The UI was served from the binary itself (no separate static server); the release probe closes the static-serving
integration blocker.


