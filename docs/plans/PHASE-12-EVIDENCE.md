# PHASE 12 EVIDENCE — Packaging, Release & Runbook

Owner: B-Provider. Date: 2026-11-06 (CST). Blueprint card: Phase 12 (WPs 12.1–12.4).
Truthful packaging statement: **v1 = localhost server + browser UI.** No installer, no Tauri/WebView2 desktop
shell, no service registration is built or claimed (blueprint D3/CF-04). Tauri remains optional/UNVERIFIED.

## Preflight (blueprint: `cargo build --release --locked`)

```
cd C:\Users\Riley\Desktop\hiverouter-v1\backend
$env:PATH="$env:USERPROFILE\.cargo\bin;C:\Users\Riley\Desktop\hiverouter-v1\tools\w64devkit\bin;"+$env:PATH
cargo build --release --locked
    Finished `release` profile [optimized] target(s) in 36m 30s     (first full build after target wipe)
RELEASE BUILD EXIT: 0

# re-run after the final source edits (same command, incremental):
cargo build --release --locked
    Finished `release` profile [optimized] target(s) in 5m 10s
FINAL RELEASE EXIT: 0
```

Artifacts (observed `Get-FileHash -Algorithm SHA256`, identical before and after the incremental rebuild):

```
hiverouter.exe      bytes=8384512  sha256=31aea08347ab695beee268828903d9834a5086e8433c4896c55c433eb4d39d05
hiverouter-cli.exe  bytes=5363712  sha256=9e06311eca12f8e7efb825bef2949f8809030e6927e73526df28400b17b8ccc3
```

`hiverouter.exe` is the single localhost server (owner: `hiverouter-core`, B-Core's Phase 12 gate);
`hiverouter-cli.exe` is the agent-native operator CLI (this phase). `--locked` succeeded against the committed
`backend/Cargo.lock`.

## CLI smoke (observed; full transcript in `backend/cli-smoke.log`)

`hiverouter-cli.exe <command> --format=json` — stdout is the machine envelope, stderr is human text, exit codes
`0 ok / 2 usage / 3 refusal / 4 internal`:

```
$ hiverouter-cli version --format=json                      [exit 0]
{"schema":"hiverouter.cli.v1","ok":true,"command":"version","data":{"version":"0.1.0",
 "cli_schema":"hiverouter.cli.v1","health_schema":"hiverouter.health.v1",
 "diagnose_schema":"hiverouter.diagnose.v1","contract_schema":"hiverouter.v1",
 "provider_version":"0.1.0","target":"x86_64-windows"},...,"meta":{"safety":{"tier":0,"mutating":false}}}

$ hiverouter-cli preflight --format=json                    [exit 0]
{"ok":true,"data":{"ready":true,"checks":[{"name":"node","ok":true,"observed":"v26.7.0"},
 {"name":"rustc","ok":true,"observed":"rustc 1.99.1 (58cf8f9c3 2026-10-13)"},
 {"name":"deepseek_api_key","ok":true,"present":false, "code":"E_KEY_MISSING","blocking":false}],
 "warnings":["E_KEY_MISSING"]}}

$ hiverouter-cli preflight --format=json   (PATH cleared of node/rustc)   [exit 3]
{"schema":"hiverouter.cli.v1","ok":false,"command":"preflight",
 "error":{"code":"E_NODE_MISSING","message":"Node.js is missing or older than the required 22.12.0.",
           "how_to_fix":"Install Node.js 22.12.0 or newer; the web build and typecheck need it.",
           "retryable":false},
 "data":{"ready":false,"checks":[{"name":"node","ok":false,"code":"E_NODE_MISSING"},
           {"name":"rustc","ok":false,"code":"E_TOOLCHAIN_MSRV"},...]}}

$ hiverouter-cli tools list --format=json                   [exit 0]
{"ok":true,"data":{"count":7,"tools":[{"id":"spawn_agent","permission_class":"execute",
  "requires_approval":true},{"id":"report_result",...},{"id":"finish",...},{"id":"file_read",...},
  {"id":"file_list",...},{"id":"file_write","permission_class":"mutate","requires_approval":true},
  {"id":"shell_exec",...}]}}

$ hiverouter-cli tools describe file_write --format=json     [exit 0]
{"ok":true,"data":{"id":"file_write","permission_class":"mutate","requires_approval":true,
 "input_schema":{"$schema":"https://json-schema.org/draft/2020-12/schema","title":"FileWriteArgs",
  "type":"object","properties":{...},"additionalProperties":false,"required":["path","content"]},
 "allowed_for_tiers":{"hyper_agent":false,"super_agent":false,"worker_agent":true,"tool_agent":true}}}

$ hiverouter-cli capabilities census --format=json           [exit 0]
{"ok":true,"data":{"census":{"tool_count":7,"refusal_count":64,"tier_count":4,
 "permission_class_count":4,"tools_by_tier":{...},"registry_tool_ids":[...7 ids...],
 "allowlist_tool_ids":[...7 ids...],"prompt_claim_tool_ids":[...7 ids...]}}}

$ hiverouter-cli provider cost --model=flash --cache-hit=1000000 --cache-miss=1000000 \
    --output=1000000 --at=2026-11-06T01:00:00Z --format=json       [exit 0]
{"ok":true,"data":{"peak":"peak","status":"COMPLETE","settled_usd":"1.506", ...}}
$ ... --at=2026-11-06T00:00:00Z                                    [exit 0]
{"ok":true,"data":{"peak":"off_peak","status":"COMPLETE","settled_usd":"0.753", ...}}

$ hiverouter-cli health --dry-run --format=json             [exit 0]
{"ok":true,"data":{"dry_run":true,"request":{"method":"GET","url":"http://127.0.0.1:8080/health",
 "timeout_secs":10}}}
$ hiverouter-cli diagnose --dry-run --format=json           [exit 0]
{"ok":true,"data":{"dry_run":true,"request":{"method":"GET","url":"http://127.0.0.1:8080/diagnose",...}}}

$ hiverouter-cli frobnicate --format=json                   [exit 2]
{"ok":false,"error":{"code":"E_COMMAND_UNKNOWN","how_to_fix":"Use a command name listed in
 docs/contracts/UI-COMMANDS-v1.md section 2.","retryable":false}}

$ hiverouter-cli tools describe rm_rf --format=json         [exit 3]
{"ok":false,"error":{"code":"E_TOOL_NOT_FOUND",...}}
```

Human mode (`--format=human`, or a TTY) writes human lines to stderr and leaves stdout empty (observed:
`hiverouter 0.1.0` on stderr). `schema` (no arg) returns the whole command table with `tier`/`mutating`
fields and generated per-command schemas (`cli::tests::schema_is_generated_from_the_single_command_table`,
`help_is_progressive`, `json_envelope_is_always_parseable` — all `ok` in PHASE-6/11 test run).

## Runbook + first-run checks

- `tools/runbook.md` — prerequisites table (Node >= 22.12.0, Rust MSRV 1.89.0, gcc via `tools/w64devkit/bin`,
  one `DEEPSEEK_API_KEY`), build/first-run/stop/cleanup, no-installer statement, named-refusal troubleshooting
  (`E_NODE_MISSING`, `E_TOOLCHAIN_MSRV`, `E_KEY_MISSING`, `E_PROVIDER_AUTH`, `E_LLM_RATE_LIMIT`,
  `E_BUDGET_EXCEEDED`, `E_USAGE_INCOMPLETE`, `E_PATH_OUTSIDE_JAIL`, `E_APPROVAL_REPLAY`, `E_WS_DISCONNECTED`).
- First-run check is executable (`hiverouter-cli preflight`), refuses by name at exit 3 (observed above),
  never prints the key (only `key_present` boolean), and does not require a running server or network.
- `tools/build-release.ps1` runs `cargo build --release --locked`, prints artifact path + SHA-256 and writes
  `tools/release-manifest.json`; `tools/test.ps1` is the one-command local test harness.
- Workspace watcher: `notify 8.2.1`, 150 ms coalesce — `watcher::tests::watcher_debounce_five_writes_in_50ms_yield_one_event ... ok`
  (5 writes in 50 ms → exactly one notification) in the PHASE-6/11 test run above.

## Files

`backend/crates/hiverouter-tools/src/bin/hiverouter.rs` (`[[bin]] hiverouter-cli`),
`backend/crates/hiverouter-tools/src/{cli,cli_support,watcher}.rs`, `tools/{runbook.md,build-release.ps1,test.ps1}`,
this file.

## Not done / UNVERIFIED (honest list)

- **Live-server boot e2e is not claimed here**: `hiverouter.exe` (core) starts the localhost server, but at the
  time of this evidence `cargo check -p hiverouter-core` was mid-edit by its owner (external dependency), so the
  "binary starts → localhost health 200" runbook dry-run is executed by the core owner's gate, not faked here.
  The CLI's `health`/`diagnose` against a live server therefore remain UNVERIFIED (dry-run request shape is
  verified).
- **No installer / desktop shell**: intentionally absent; Tauri/WebView2 detection remains optional and
  UNVERIFIED (blueprint CF-05). No packaging claim beyond the two executables.
- **Fresh-machine runbook verification** (a truly clean Windows profile without Rust/Node) is not executable on
  this host; the named-refusal path is verified by clearing PATH (observed above), the full fresh-machine run is
  UNVERIFIED.
- `npm`/web gates are the frontend owner's evidence; the CLI does not shell out to npm.
