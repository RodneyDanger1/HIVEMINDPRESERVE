# HiveMind v1 — Windows runbook

Truthful packaging statement: **v1 is a localhost server + a browser UI.** The
release build produces two executables (`hiverouter.exe` local server,
`hiverouter-cli.exe` operator CLI). There is **no installer, no Tauri/WebView2
desktop shell, and no service registration** in v1 (blueprint D3/CF-04); Tauri
remains an optional, UNVERIFIED future decision. No cloud component, no Docker,
no Python runtime.

## 1. Prerequisites (first-run dependency checks)

| Dependency | Required | Why | Refusal when missing |
|---|---|---|---|
| Windows 10/11 x64 | Windows 10/11 | target platform (blueprint §2 rule 10) | n/a |
| Node.js | **>= 22.12.0** | web build/typecheck only (not needed to run the server binary) | `E_NODE_MISSING` |
| Rust toolchain | **>= 1.89.0** (MSRV) | building from source only | `E_TOOLCHAIN_MSRV` |
| C toolchain | gcc 16 provided in `tools/w64devkit/bin` | `rusqlite` bundled + TLS build | build error (named at build time) |
| DeepSeek API key | one account key in `DEEPSEEK_API_KEY` | live provider calls | `E_KEY_MISSING` (UI + CLI; no fake run) |

Check them in one command (JSON on stdout for agents, human lines on stderr):

```
hiverouter-cli preflight            # exit 0 ready; 3 named refusal; 2 usage
hiverouter-cli preflight --json     # machine envelope hiverouter.cli.v1
```

The check reports `node`, `rustc` and `deepseek_api_key` rows; the key is only
ever reported as `present: true/false` — never printed.

## 2. Build (from a checkout)

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;C:\path\to\hiverouter-v1\tools\w64devkit\bin;" + $env:PATH
$env:CARGO_PROFILE_DEV_DEBUG = "0"; $env:CARGO_INCREMENTAL = "0"
cd C:\path\to\hiverouter-v1\backend
cargo build --release --locked
```

or: `pwsh -NoProfile -ExecutionPolicy Bypass -File tools\build-release.ps1`
(prints artifact paths + SHA-256; writes `tools/release-manifest.json`).

## 3. First run

1. Set the one key in the environment of the server process:
   `$env:DEEPSEEK_API_KEY = "<your-key>"` (never commit it; never paste it into the UI).
2. Start the local server (binds 127.0.0.1 only): `.\target\release\hiverouter.exe`
3. Open the local UI in a browser: the server prints its `http://127.0.0.1:8080` URL.
4. Verify health: `hiverouter-cli health --url http://127.0.0.1:8080` → 200 JSON
   with `schema: hiverouter.health.v1`. Diagnose: `hiverouter-cli diagnose`.
5. Watch the audit log written by the server under its data directory (`audit.jsonl.*`
   JSONL files). The audit probe refuses key/token substrings.

## 4. Stopping / cleanup

- Stop the server: close the process (Ctrl+C). Child processes are contained in a
  kill-on-close Windows Job Object, so they are reaped with the server.
- Data lives in the configured data directory (`Config { bind, data_dir, db_path }`);
  deleting it is an explicit operator action. The graph snapshot is versioned JSON;
  SQLite holds cache/metrics/logs only (blueprint C12).

## 5. Troubleshooting (named refusals)

| Code | Meaning | Fix |
|---|---|---|
| `E_KEY_MISSING` | no `DEEPSEEK_API_KEY` in the server environment | set it and restart the server |
| `E_NODE_MISSING` | Node absent or < 22.12.0 | install Node >= 22.12.0 (web build only) |
| `E_TOOLCHAIN_MSRV` | rustc < 1.89.0 | install Rust >= 1.89.0 |
| `E_PROVIDER_AUTH` | DeepSeek rejected the key (401/402, never retried) | check the key/account balance |
| `E_LLM_RATE_LIMIT` | HTTP 429 persisted after 3 attempts | wait for the Retry-After window |
| `E_BUDGET_EXCEEDED` | run hit the $0.50 cost cap before the call | start a new run or lower scope |
| `E_USAGE_INCOMPLETE` | provider omitted usage; cost is INCOMPLETE (never $0.00) | inspect the run; do not settle a cost |
| `E_PATH_OUTSIDE_JAIL` | a tool path escaped the workspace root | use workspace-relative paths |
| `E_APPROVAL_REPLAY` | approval nonce was already used | resolve the pending request once |
| `E_WS_DISCONNECTED` | local server not reachable | start the server / check `--url` |

## 6. Rollback

- The project directory is git-initialized; every phase is a commit and is
  revertible. The blueprint directory is read-only reference material.
- Runtime inverses: dropping the audit guard flushes/stops the JSONL writer;
  `ProcessRegistry::unregister` removes a finished run; deleting the data
  directory resets local state (explicit operator action only).
