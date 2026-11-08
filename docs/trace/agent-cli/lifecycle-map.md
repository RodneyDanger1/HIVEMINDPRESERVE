# Lifecycle map — how HiveMind is launched, supervised and shut down

**Seat:** audit-only recon (no product edit, no test edit, no config edit, no server start/stop).
**Date:** 2026-11-08T01:15–01:25 CST (America/Chicago).
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1`, HEAD `36ce7d3` (`feat: scoped memory, newest history, approval keyboard, and A2A roster context`).
**Working tree at map time (two snapshots, because another lane is actively editing the CLI):** at 01:10 the tree held `M tools/desktop-launcher/HiveMind.exe` plus untracked `.claude/`, `AGENTS.md`, `CLAUDE.md`, `tools/hivemind-cli.mjs`, `tools/hivemind.cmd`, `tools/hivemind.ps1`. By 01:20 it had grown `M README.md`, `M backend/crates/hiverouter-core/src/main.rs` (help text only, `--workspace-root` added), `HiveMind-Agent.cmd`, `HiveMind-Agent.ps1`, `docs/AGENT-CLI.md`, `skills/`, and `tools/hivemind-cli.mjs` had grown from 58,886 B (01:14:22) to 68,085 B (01:19:06); by ~01:22 `M tools/start-hivemind.ps1` added `--workspace-root` and the CLI reached 72,160 B. **This document is a snapshot: HEAD `36ce7d3` is the citation baseline, and where a working-tree edit changed the fact it is called out inline.**
**Live runtime at map time:** no `hiverouter.exe`, no `HiveMind.exe`, and nothing listening on `127.0.0.1:8099` (`Get-CimInstance Win32_Process` → 0/0; `Get-NetTCPConnection -State Listen -LocalPort 8099` → 0 rows). No HiveMind process was started, stopped, signalled, or probed during this seat.

**Evidence classes used below**

| Class | Meaning |
|---|---|
| `SRC` | Read from the file at the cited line(s) in this checkout. |
| `RUN` | Observed by executing a command in this session; output quoted. |
| `INFER` | Follows from `SRC`/`RUN` but is not itself observed. |
| `OPEN` | Not verified; named as a gap in §12. |

**Method / scope note.** Two throwaway `powershell.exe` sleep processes (never HiveMind, never the server binary) were started and terminated to characterise Windows stop semantics (§7). One temp directory under `%TEMP%` was created and removed to characterise `.NET` log-redirection truncation (§2.2). Nothing else was executed. `forgewright-recon` was applied: entry points read, callers walked, tests located (commands in §1), data flow named. No `/integration-logic` skill file exists in this checkout (`glob **/integration-logic*` → none), so the integration contract is derived from the code and recorded in §9–§10 instead of a skill document; that substitution is stated rather than implied.

---

## 1. Recon ledger

- **Target behaviour:** how the HiveMind local server process is created, waited on, observed, and terminated today — and what a machine-operated CLI must add so `start --json` / `status` / `stop` / `restart` are truthful and safe.
- **Entry points (server):** `backend/crates/hiverouter-core/src/main.rs:19` (`main` → `run`), `:44` (`Config::from_args`), `:136` (`TcpListener::bind`), `:166` (`http::serve`), `:178` (`shutdown_signal`).
- **Entry points (launchers):** `HiveMind.cmd:3` → `tools/start-hivemind.ps1:1`; `tools/desktop-launcher/HiveMindLauncher.cs:22` (`Main`); `HiveMind-Agent.cmd` / `tools/hivemind.cmd` → `tools/hivemind-cli.mjs` (`main`, in flight); `e2e/helpers/live-server.ts:368` (`startLiveServer`, test-only).
- **Callers of the server contract:** `tools/start-hivemind.ps1:17-24,53-65` (health poll + reuse), `HiveMindLauncher.cs:45-54,70,217-245` (port/health/wait), `e2e/helpers/live-server.ts:214-257` (health + schema + child-exit), `tools/hivemind-cli.mjs` (`startServer`/`waitForHealth`: spawn + poll, in flight), `backend/crates/hiverouter-tools/src/cli.rs:778-815` (`health`/`diagnose` HTTP commands).
- **Tests that pin current behaviour + commands:**
  - `cargo test -p hiverouter-core --test http_boot --locked` — `/health` 200, `schema == hiverouter.v1`, `status ∈ {ok,degraded}`, `database.schema_version == 11` (`backend/crates/hiverouter-core/tests/http_boot.rs:213-230`).
  - `cargo test -p hiverouter-core --test kill_restore --locked` — hard-kill then boot recovery marks the run interrupted (`backend/crates/hiverouter-core/tests/kill_restore.rs:1-5,48-80`).
  - `cargo test -p hiverouter-tools --locked` — CLI dispatch/exit codes/JSON envelope (`backend/crates/hiverouter-tools/src/cli.rs:829-1022`).
  - `cargo test -p hiverouter-core --locked` — `serve` + graceful shutdown path.
  - e2e: `cd e2e; node run-guarded.mjs` / `npm run e2e` (`e2e/package.json` scripts), live boot proven by `e2e/specs/live-streaming.spec.ts:52`.
  - Repo gate: `pwsh -NoProfile -ExecutionPolicy Bypass -File tools\test.ps1` (`tools/test.ps1:36-72`).
- **Data flow (happy path):** CLI/launcher resolves `{bind, data_dir, db_path, workspace_root}` → spawns `hiverouter.exe` with those flags + provider key in the child env only → server creates data dir, opens `<data-dir>/hiverouter.db`, migrates, runs boot recovery, runs env checks, binds loopback, prints `hiverouter <v> listening on http://<addr> (db …, schema v…, contract hiverouter.v1)` → `/health` answers 200 `ok|degraded` → UI served from embedded `web/dist` → `Ctrl+C` completes the shutdown future → axum drains → `run()` returns → audit guard drops and flushes.
- **Data flow (failure path):** second instance on a busy port → `TcpListener::bind` fails → `eprintln!("hiverouter: E_WS_DISCONNECTED: cannot bind …")` → `ExitCode::FAILURE` (`main.rs:136-145`); the launching script sees the child exit and throws with the stderr tail (`tools/start-hivemind.ps1:60-63`); a CLI must not mistake this for "server unhealthy".
- **Unknowns:** whether an agent CLI should own lifecycle in Rust (`hiverouter-cli`, which already owns the exit-code contract) or Node (`tools/hivemind-cli.mjs`, in flight); whether a graceful remote stop is allowed to require a small server-side change (§7); which build fingerprint a `status` may claim (`/health` has no build id, §10-#6). See §12.
- **Files I expect to touch:** this document only (§13). The implementation lanes touched by the findings are `tools/hivemind-cli.mjs`, `tools/start-hivemind.ps1`, `tools/desktop-launcher/HiveMindLauncher.cs`, and — only if a graceful stop requires it — `backend/crates/hiverouter-core/src/main.rs`; none was edited here.

---

## 2. Launch surfaces that exist today

There is **no service registration, no installer, no scheduled task, and no supervisor process**. Everything is "spawn a console binary and hope". Four production launch paths and one test path:

| # | Surface | Spawn | cwd / workspace | Data dir | Bind | Key | Logs | Waits for health | Opens UI | Stop |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `.\target\release\hiverouter.exe` (documented direct run) | human shell | cwd / cwd | `data` (cwd-relative) | `127.0.0.1:8080` | env only | console | no | manual | Ctrl+C in that console |
| 2 | `HiveMind.cmd` → `tools\start-hivemind.ps1` | `Start-Process -PassThru` with redirected stdio | **HEAD: inherits caller cwd (no `--workspace-root`)** → workspace = cwd; a 01:2x working-tree edit now passes `--workspace-root $root` (repo root) | `%LOCALAPPDATA%\HiveMind` | `127.0.0.1:8099` | DPAPI `provider.key` → child env | `<data>\server.stdout.log`, `server.stderr.log` (**truncated each start**, §2.2) | 20 s / 300 ms | **yes, always** `Start-Process $base` | prints `Stop-Process -Id <pid>` (forced, §7) |
| 3 | `dist\HiveMind\HiveMind.exe` (`HiveMindLauncher.cs`) | `Process.Start` (hidden) with **explicit `--workspace-root`** | workspace root (hard-coded repo path or env) | `%LOCALAPPDATA%\HiveMind` | `127.0.0.1:8099` (fixed) | DPAPI → child env, env fallback | redirected into **undrained pipes** (never read, no file path set) | 60 s / 300 ms | **yes**, Edge/Chrome `--app` window | closing the app window kills the child (heuristic, §8) |
| 4 | `HiveMind-Agent.cmd` / `HiveMind-Agent.ps1` (root) and `tools\hivemind.cmd` / `.ps1` → `tools\hivemind-cli.mjs` (in-flight, untracked) | Node `spawn(detached:true, windowsHide:true)` | `--workspace-root` (validated, default repo root) | `%LOCALAPPDATA%\HiveMind` (`--data-dir`, `HIVEROUTER_DATA_DIR`) | `127.0.0.1:8099` (`--port`, `--bind`, `--url`) | **none handled** (inherits `process.env` only) | `<data>\server.stdout.log` / `server.stderr.log` (**appended**) | 60 s / 300 ms, `--no-wait` opt-out, aborts on child exit | never (separate `open` command) | state-file PID + `taskkill /T` (§11) |
| 5 | `e2e\helpers\live-server.ts` (test-only, the best-behaved one) | Node `spawn(stdio:pipe, windowsHide:true)` | temp `workspace/` | temp `data/` | **reserved ephemeral** loopback port, 3 attempts | key stripped unless explicitly granted | in-memory ring (max 200 lines, redacted) | 20 s / 200 ms, **aborts on child exit**, verifies `schema` | never | `child.kill()` → 5 s → `SIGKILL`, temp dir removed |

### 2.1 Server CLI contract (`SRC`)

`backend/crates/hiverouter-core/src/config.rs:254-296` (+ env seeds at `:255-261`):

| Flag | Env fallback | Default | Notes |
|---|---|---|---|
| `--bind <ADDR>` | `HIVEROUTER_BIND` | `127.0.0.1:8080` (`config.rs:10`) | non-loopback refused: `E_ORIGIN_REJECTED` (`:332-334`) |
| `--data-dir <DIR>` | `HIVEROUTER_DATA_DIR` | `data` relative to **process cwd** (`:12`) | created if missing (`:336-339`) |
| `--db <FILE>` | `HIVEROUTER_DB` | `<data-dir>/hiverouter.db` (`:14,340-343`) | parent dir created (`:347-354`); a directory path is refused |
| `--workspace-root <DIR>` | `HIVEROUTER_WORKSPACE_ROOT` | **`std::env::current_dir()`** (`:197-201,367-369`) | **at HEAD `36ce7d3` it is missing from `hiverouter --help` (`main.rs:187-214`); a 01:19:35 working-tree edit added it, so the gap is being closed** |

Env-only knobs: `HIVEROUTER_ALLOWED_ORIGINS` (append-only, exact loopback origins, `config.rs:382-392`), `HIVEROUTER_SIDE_EFFECTS` (`:355-361`), provider settings `HIVEROUTER_PROVIDER|MODEL|THINKING|LEAF_THINKING|REASONING_EFFORT|MAX_OUTPUT_TOKENS|PLANNER_MAX_OUTPUT_TOKENS` (`:77-131`; unknown values refuse `E_SCHEMA_INVALID` at boot).

Data layout under `--data-dir`: `hiverouter.db` (+ `-wal`, `-shm`), `audit/audit.jsonl.YYYY-MM-DD` (`main.rs:52`, `tools/src/audit.rs:27-31,71`), `snapshots/<run-id>/<0000000000>.json` (`backend/crates/hiverouter-core/src/snapshot.rs:4-6,56,68,82`). Outside those, the launcher creates `window/` (browser profile) in the same data dir (`HiveMindLauncher.cs:85-86`) and `tools\set-provider-key.ps1` writes `provider.key` there (`:35-38`).

### 2.2 Two log-writing divergences worth knowing (`RUN` + `SRC`)

- PowerShell `Start-Process -RedirectStandardOutput` **truncates** the target on every start. Probe: run `cmd /c echo first-run` then `cmd /c echo second-run` to the same file → file contained only `second-run`. So `%LOCALAPPDATA%\HiveMind\server.stdout.log` (the `.cmd` path, `tools/start-hivemind.ps1:47-51`) keeps **one boot only**.
- The in-flight Node CLI opens the same filenames with flag `'a'` (`startServer`, `fs.openSync(stdoutPath, 'a')`) → append. Two launchers, two retention semantics, same file. An agent CLI should own one policy (append + size cap + a rotated name) and not silently depend on either.

### 2.3 The `.cmd` path did not pin the workspace (`SRC` at HEAD; being fixed in the working tree)

At HEAD `36ce7d3`, `tools/start-hivemind.ps1:49-51` passes only `--bind` and `--data-dir`; it computes `$root` (`:9`) but never forwards it. The child inherits the caller's cwd, and `Config` then uses cwd as `workspace_root` (`config.rs:197-201`). Consequence: launching `HiveMind.cmd` from `dist\HiveMind` (double-click, or a shortcut whose "Start in" is the app folder) jails the file tools to the two-binary folder; the packaged README even tells the operator the folder is portable. This exact failure is recorded in `docs/research/hmars/10-gap-meta.md:37,555` ("Run 1 analysed dist\HiveMind … no source code at all"). **A working-tree edit observed at ~01:22 now passes `--workspace-root $root`**, which closes the "falls back to cwd" half of the gap but pins every `.cmd` launch to the repo root (a caller cannot choose a project). The launcher (surface 3) is the other path that passes `--workspace-root`, and it defaults to a hard-coded `C:\Users\Riley\Desktop\hiverouter-v1` (`HiveMindLauncher.cs:17,57,61,173-176`).

---

## 3. Boot sequence, health, and refusal semantics (server side)

### 3.1 Boot order (`SRC`, `main.rs`)

1. `--help` / `--version` short-circuit with exit 0 (`:27-42`).
2. `Config::from_args` — every flag/env value validated **before** any I/O; failure prints one line and exits 1 (`:44-50`).
3. Audit sink `init_json_audit(<data-dir>/audit)`; a second subscriber or an unwritable dir aborts boot (`:52-58`; `tools/src/audit.rs:56-100`). The guard must live for the process lifetime — dropping it flushes and stops the JSONL writer (`audit.rs:10-11,33-38`).
4. Open + migrate SQLite (`:60-82`).
5. Kill-restore recovery (`:84-106`) — a corrupt workspace refuses boot rather than silently resetting.
6. Environment checks (rustc required; node optional; `deepseek_api_key` optional) (`:108-126`; `checks.rs:58-68,90-132,143-161`).
7. Bind loopback (`:136-145`).
8. Print the one machine-readable boot line (`:153-158`) — **before** serving, but only after the listener exists and recovery is done.
9. Serve with `.with_graceful_shutdown(shutdown)` (`:166-175`; `http.rs:1239-1250`).

### 3.2 Exit-code taxonomy (`SRC`)

Success `0`; help/version `0`; **everything else `1`** (`main.rs:18-24,44-50,52-58,60-71,72-82,87-106,136-145`). There is no distinct code for "port busy", "DB locked", "bad flag", or "missing binary". A supervisor cannot classify a failed boot from the exit code — only from the stderr text. (`E_WS_DISCONNECTED` is even reused for the bind failure message, `main.rs:140`.)

### 3.3 `/health` payload and status truth (`SRC`)

`http.rs:1153-1204`; field list `http.rs:456-476`. No `pid`, no `workspace_root`, no `build`/`git` id, no URL. It carries `status`, `version` (crate version), `schema` (`hiverouter.v1`), `uptime_secs`, `checks[]`, `database{path,schema_version,error_code}`, `transport{last_seq,graph_nodes,allowed_origins}`, `prompt_assets{loaded,error_code}`.

Status mapping: `ok` = all checks pass; `degraded` = only optional checks failed; `failing` = a required check or the DB version failed (`:1167-1180`). HTTP status is **503 only when a required check failed** (`:1198-1202`). `rustc`, `prompt_assets`, `persona_assets` are required; `node` and `deepseek_api_key` are optional (`checks.rs:60,97-101,148-152`; `http.rs:91-101,122-130`). Therefore **a keyless server with the provider enabled (the default) boots `degraded` with HTTP 200** — the single most important fact for CLI readiness logic. (`HIVEROUTER_PROVIDER=disabled` removes the key check entirely, so such a server reports `ok`.)

`/diagnose` (`http.rs:596-619`) is the only place with `provider_key_present: bool` (`tools/src/health.rs:106-131,219`), plus `side_effects_enabled`. `/health` expresses key presence only as `checks[name="deepseek_api_key"].ok`.

`/api/session` (`http.rs:1206-1233`) hands the WS token to any **loopback peer with no `Origin`**; the Host guard (`:543-584`) rejects a foreign `Host` with `421 E_ORIGIN_REJECTED`. A CLI must keep its Host header correct (default for `127.0.0.1:<port>`/`localhost:<port>`; `--url` with another authority gets 421).

### 3.4 Embedded UI is a compile-time embed (`SRC`)

`static_ui.rs:14-16` embeds `web/dist`; `GET /` returns `index.html`; unknown non-API paths return the SPA; `api/*`, `health`, `ws` always 404 if unrouted (`:42-72`). A "UI is served" smoke check is therefore `GET /` → 200 and body contains `<div id="root">` (the e2e harness does exactly this, `e2e/helpers/live-server.ts:291-313`). A stale binary serves a stale UI even when `web/dist` is newer — nothing exposes a UI bundle id.

### 3.5 `--port 0` is not usable for the real app (`SRC`)

The server binds `:0` and prints the real address, but `default_allowed_origins` returns an **empty** allowlist when the configured port is 0 (`config.rs:205-208`), so the browser's WS `Origin` is rejected. Ephemeral binding is a test-only affordance; a product CLI must reserve a free fixed port and retry (as the e2e harness does, `e2e/helpers/live-server.ts:166-187,383-387`).

### 3.6 Graceful shutdown today

The shutdown future is `tokio::signal::ctrl_c()` only (`main.rs:178-184`). In the pinned tokio (1.54.1, `backend/Cargo.lock`) the cross-platform `ctrl_c()` resolves to the **CtrlC** console event only (`tokio-1.54.1/src/signal/ctrl_c.rs:59-62` → `signal/windows.rs:46-50` → `signal/windows/sys.rs:47`). Tokio exposes `ctrl_break()` (`signal/windows.rs:231-235`) but **nothing in this server installs it**, and nothing handles `CTRL_CLOSE`/`CTRL_SHUTDOWN`/`CTRL_LOGOFF`. Consequences: closing the console or killing the process never reaches the graceful path; axum's drain and the audit flush are skipped; only an in-console Ctrl+C (or an injected `CTRL_C_EVENT`) is graceful. See §7 for the measured stop behaviour.

---

## 4. Port choice and readiness polling, surface by surface

| Surface | Port decision | Conflict handling | Health policy |
|---|---|---|---|
| Direct exe | `--bind` / `HIVEROUTER_BIND` / 8080 | none; bind failure → exit 1 | none |
| `HiveMind.cmd` | fixed `127.0.0.1:8099` (`start-hivemind.ps1:4`) | probes `/health` first; if healthy, **reuses any HiveMind and opens the browser** (`:17-24`); else spawns and lets the bind fail | 300 ms poll, 20 s deadline, aborts on child exit, prints stderr tail (`:53-69`) |
| `HiveMindLauncher.cs` | fixed `8099` (`:16`) | `PortOpen && !HealthOk` → modal "Port 8099 is in use by another application" (`:45-54`); `PortOpen && HealthOk` → reuse and open UI (`:45,55`) | 300 ms poll, 60 s deadline; `HealthOk` accepts **any 2xx without checking the body/schema** (`:217-231,233-245`) |
| Node CLI (in flight) | `--port`/`--bind`/`--url`, default 8099 (`hivemind-cli.mjs:27`; `startServer`) | health first, validates `schema` + `database.path`; then TCP probe → `E_PORT_IN_USE` (`startServer`); no retry on bind failure | 300 ms poll, 60 s default, **aborts on child exit with the stderr tail** (`waitForHealth`), at the 01:19:06 revision |
| e2e harness | reserve `:0`, close, retry ≤3 (`live-server.ts:166-187,383-387`) | retries a fresh port on any start failure | 200 ms poll, 20 s, verifies `schema == hiverouter.v1`, aborts the moment the child exits (`:214-257`) |

Three different boot timeouts, three different conflict policies, two different health contracts. The launcher's "port in use" dialog also **misclassifies a healthy-but-`failing` HiveMind** (HTTP 503) as "another application" — the process is HiveMind, but `HealthOk` is false.

Also note the packaged README promises a **free loopback port** ("It starts hiverouter.exe … on a free loopback port", `tools/build-desktop.ps1:69-70`) while the code binds the fixed `8099` and never scans. That sentence is false today.

---

## 5. Provider key handling (no key is read or printed by this seat)

| Path | Read | Inject | Persist | Failure mode |
|---|---|---|---|---|
| `tools\set-provider-key.ps1` | hidden `Read-Host -AsSecureString`, BSTR zero-freed (`:20-24,40-49`) | — | DPAPI `Protect(bytes, UTF8 "HiveMind.ProviderKey.v1", CurrentUser)` → `%LOCALAPPDATA%\HiveMind\provider.key` (`:28-38`) | throws on empty; never logs the value |
| `HiveMindLauncher.cs` | DPAPI `Unprotect` same entropy/scope (`:138-155`) | `start.EnvironmentVariables["DEEPSEEK_API_KEY"]` — child only (`:129-136`) | no | empty `catch` (`:156-159`) then env fallback (`:160`); a failed decrypt is indistinguishable from "no key" |
| `tools\start-hivemind.ps1` | DPAPI `Unprotect` (`:31-42`), zeroes the plaintext buffer (`:42`) | `$env:DEEPSEEK_API_KEY` in the (short-lived) PowerShell parent → inherited by the child (`:41,49-51`) | no | warns and continues keyless (`:43-45`) |
| Node CLI (in flight) | **reads nothing** (header contract `hivemind-cli.mjs:9-12`; `startServer` spawns with `env: process.env`) | inherits `process.env` only | no | start succeeds; server is `degraded`; no explicit key warning in the payload |
| Server | `ApiKey::from_env` / `present_in_env`, boot only (`secret.rs:30-38,59-65`; `config.rs:370`) | — | — | missing/blank → `UnconfiguredExecutor`, prompts refuse `E_KEY_MISSING`; **no reload path** — rotate then restart |

Hygiene facts: `Debug` is `ApiKey(***)` (`secret.rs:77-81`); `/health` reports `present: true/false` only (`checks.rs:143-161`); `/diagnose` reports the boolean (`tools/src/health.rs:116,219`); the CLI `preflight` reports presence only (`cli.rs:7-21`). The decrypted key lives in the server **child environment**, readable by same-user processes — at-rest protection only (`docs/trace/provider/02-agent-runtime-wiring.md:169`). A lifecycle CLI must therefore: never accept the key as a flag, never store it, never print it, and never claim "key ready" without reading the boolean (from `/diagnose` or the `deepseek_api_key` check) and saying which one it read.

**Doc-truth correction:** the older trace note `docs/trace/provider/02-agent-runtime-wiring.md:152` says the `.cmd` path "starts the server with no DPAPI read at all". That is **stale** against the current `tools/start-hivemind.ps1:27-46`, which does DPAPI-unprotect `provider.key` and exports it to the child. The remaining real gap is narrower: the `.cmd` path reads DPAPI but does not pass `--workspace-root` (§2.3), and neither the `.cmd` path nor the launcher can reload a rotated key without a restart.

---

## 6. Process / PID / state conventions: what exists, what does not

**Product code persists nothing.** No PID file, no lock file, no instance registry, no URL file. Evidence: repo-wide search for `pid|\.pid|state\.json|instance` in `backend/crates` returns only the `sysinfo` sampling field and the `ProcessRegistry` (a per-run cancellation registry for tool children, `tools/src/cancel.rs:66-70` — not server supervision). The kill-on-close Job Object in `tools/src/shell.rs:10,221-223,331-353` contains **tool** grandchildren, not the server and not the server's parent.

The only in-memory handle is the launcher's local `Process serverProcess` (`HiveMindLauncher.cs:44,69,72,104-111`) and the scripts' `$process` (`start-hivemind.ps1:49,60,71`). Both die with the launcher process. The printed PID (`start-hivemind.ps1:71-73`) is console text only.

The in-flight Node CLI introduces the first persisted convention:

```
%LOCALAPPDATA%\HiveMind\agent-cli-state.json     (or <--data-dir>\agent-cli-state.json)
{ state_schema: "hivemind.cli.state.v1", status: "starting"|"ready", url, pid, external,
  started_at, exe, workspace_root, data_dir, db_path, provider_key_present?,
  stdout_log?, stderr_log? }
```
(`tools/hivemind-cli.mjs`, functions `statePath`/`readState`/`writeState`/`clearState`). The shape moved twice during this seat: at 01:19:06 only the already-running branch wrote `state_schema`; at 01:21:23 both branches write it and the spawned branch now writes `status:"starting"` before the health wait and upgrades to `status:"ready"` (with `provider_key_present`) after it. It is `mode 0o600`, contains no secret, and is read by `status`/`stop`/`logs`. It still has no process-creation time and no lock — see §11.

**Identity anchors that do exist for a CLI**

| Want to know | Available source | Confidence |
|---|---|---|
| Is this the instance that owns my data dir? | `/health.database.path` (`http.rs:1156-1165`) vs resolved `<data-dir>/hiverouter.db` | strong, no product change |
| Is this the right build/schema? | `/health.schema` (`hiverouter.v1`), `/health.version`, `/health.database.schema_version` | strong |
| Which workspace root is it jailing? | **not exposed** by `/health` or `/diagnose` — only the OS command line (`Get-CimInstance Win32_Process.CommandLine`, the method used in `docs/research/hmars/26-scale-security-audit.md:45,74`) or a state file the CLI wrote itself | weak / out-of-band |
| Is the PID still the process I started? | PID + process creation time + image path (Win32 `GetProcessTimes`/`QueryFullProcessImageName`) | must be implemented; nothing does it today |
| Is a run active right now (restart risk)? | `/api/history/runs` (`http.rs:517`) / `/health.transport.last_seq` | partial |

---

## 7. Shutdown reality — measured, not assumed

**What the server offers:** one graceful path (Ctrl+C completing `shutdown_signal`, §3.6). **What the launchers do:**

- `start-hivemind.ps1:73` tells the operator `Stop: Stop-Process -Id <pid>` — `Stop-Process` is `TerminateProcess`: no drain, no audit flush.
- The launcher calls `serverProcess.Kill()` (`HiveMindLauncher.cs:72-75,104-112`) — also forced.
- The e2e harness calls `child.kill()` then `SIGKILL` after 5 s (`live-server.ts:315-331`) — forced.
- The Node CLI calls `taskkill /PID <pid> /T` **without `/F`** and treats "still alive after 10 s" as `E_STOP_TIMEOUT` (`stopServer`).

**Measured Windows behaviour (`RUN`, throwaway `powershell.exe … Start-Sleep`, never HiveMind):**

| Target process shape | Command | Observed |
|---|---|---|
| `CreateNoWindow=true`, redirected stdio (what Node `windowsHide:true` and a Rust CLI would spawn) | `taskkill /PID <pid>` | `SUCCESS: Sent termination signal to the process with PID 9644.` → **process still alive** after 700 ms; `/F` then killed it, exit code `1` |
| Inherited console, redirected stdio (exactly `start-hivemind.ps1`'s `Start-Process -Redirect*` shape) | `taskkill /PID <pid>` | `SUCCESS: Sent termination signal …` → **process still alive**; `/F` killed it, exit code `1` |
| Hidden **window** (plain `Start-Process -WindowStyle Hidden`, no redirection) | `taskkill /PID <pid>` | process exited with `0xC000013A` (`STATUS_CONTROL_C_EXIT`) — the Windows close/terminate family, not a graceful application exit; that shape never occurs in the headless launches, and the server installs no close-event handler anyway (§3.6) |

Conclusion: for every headless/redirected server this product actually launches, `taskkill` without `/F` reports success and does nothing. The in-flight CLI's graceful attempt therefore **always** ends in `E_STOP_TIMEOUT` (exit 5) and needs a second `--force` invocation. That is a verified defect, not a hypothesis.

**Options for a truthful graceful stop (design, not implemented):**

| Option | Mechanism | Cost | Status |
|---|---|---|---|
| A. Console-control injection | `FreeConsole` → `AttachConsole(serverPid)` → `SetConsoleCtrlHandler(NULL, TRUE)` → `GenerateConsoleCtrlEvent(CTRL_C_EVENT, 0)` → wait → restore own console | ~80 lines of `windows-sys`; CLI output must be buffered before detaching | `OPEN` — plausible, needs a spike against a running server; cannot be verified in this seat |
| B. Server-side break signal | install `tokio::signal::windows::ctrl_break()` in the shutdown future (`main.rs:178-184`), spawn the server with `CREATE_NEW_PROCESS_GROUP`, then `GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, pid)` after attaching to its console | small server change + product contract ("we handle Ctrl+Break too") | `OPEN` — smallest code change, needs a test |
| C. Local HTTP shutdown route | e.g. `POST /api/admin/shutdown`, gated by the Host guard + the `/api/session` token | a route, a refusal path, a test, and a security review (any local process can already `taskkill`, so the gate is CSRF not auth) | `OPEN` — cleanest for remote agents, biggest contract surface |
| D. Forced stop, told truthfully | `taskkill /F`; report `graceful:false`; rely on boot recovery (`main.rs:84-106`; `kill_restore.rs`) | none | **works today**; loses unflushed audit lines (`audit.rs:10-11`) |

Recommended minimum now: **D with explicit reporting**, and an automatic escalation (one command, grace window, `graceful:false` in the JSON) instead of forcing every agent to run `stop` twice. Do not describe a `taskkill` without `/F` as graceful.

---

## 8. WebView2 / browser app mode

There is **no WebView2 control and no Tauri**; the desktop shell is `HiveMind.exe` (a WinForms console-less `.NET` exe compiled by `csc` with `/target:winexe`, `tools/build-desktop.ps1:46-54`), which launches **installed Edge or Chrome in `--app=` mode** (`HiveMindLauncher.cs:18-19,87-95`): `--app=http://127.0.0.1:8099/ --window-size=1440,900 --user-data-dir=%LOCALAPPDATA%\HiveMind\window`. If neither browser path exists, `Process.Start` throws and the generic `catch` shows a modal (`:88,114-121`). Doc drift: the README line says "Edge **WebView2** app window" (`build-desktop.ps1:70`) and `tools/runbook.md:3-8` says "no Tauri/WebView2 desktop shell" at the same time.

Lifetime coupling is a heuristic: after the browser process exits, the launcher kills the server **only if the window lived ≥ 3 s** and the server is still alive (`:96-112`); otherwise it leaves the server running (a browser crash or an Edge "hand-off to an existing instance" can therefore orphan or double-own the server). A machine CLI must not copy this heuristic; it must own the child explicitly and expose an idempotent stop.

---

## 9. What an agent-owned CLI must actually do

The surface must be **idempotent, identity-checked, non-destructive by default, secret-tight, and browser-free**. The in-flight Node CLI already declares `start/stop/restart/status/doctor/logs/open` (its `COMMANDS` registry); the shipped Rust CLI has no lifecycle commands at all — its table is `version, preflight, schema, tools.list, tools.describe, capabilities.census, provider.cost, health, diagnose` (`cli.rs:81-172`) and it explicitly documents "v1 ships no mutating CLI command" (`cli.rs:16-18`). One of those two must own lifecycle; §11 states the trade-off. "Secret-tight" means: it may inject the DPAPI `provider.key` exactly as the launcher does, but it never echoes, stores, logs, or reports the value — only `provider_key_present`.

### 9.1 Required commands and flags

```
start   --json [--bind ADDR|--port N] [--data-dir DIR] [--workspace-root DIR] [--exe PATH]
        [--ready-timeout SECS] [--require-key] [--no-open] [--foreground] [--dry-run]
status  --json [--url URL]
stop    --json [--timeout SECS] [--force] [--dry-run]        # idempotent
restart --json [same as start] [--timeout SECS]               # stop-then-start, same identity
```

Rules that apply to all four:

- **stdout is only the envelope.** The server's own stdout (the boot line) must be redirected to a file, never inherited — otherwise it corrupts `--json` (the Node CLI already does this in `startServer`).
- **exit 0 = the requested end state holds.** Already running (`start`), already stopped (`stop`), and "determined not running" (`status`) are successes, not errors. `--require-running` / `--require-key` are the opt-ins that make them exit 3.
- **`--dry-run` must have no side effects** — including no health probe that writes state. The 01:14:22 draft violated this (it wrote state in the already-running branch before checking `dry-run`); the 01:19:06 revision fixed it — `startServer` now returns the dry-run plan before any probe, `mkdir`, spawn, or state write (`hivemind-cli.mjs`, `startServer`, dry-run guard before the health probe).
- **never delete data.** No `rm -rf` on stop/restart/failed start; a `reset` command, if ever added, needs `--purge-data --yes`, prints the exact paths in `--dry-run`, and must not be reachable from `stop`.
- **never read/print/store the key.** Presence only, from `/diagnose.provider_key_present` or the `deepseek_api_key` check, with the source named in the payload.
- **never open a browser** unless `--open`/`open` is given. `HiveMind.cmd` and the launcher always open; that is wrong for agents.
- **never kill a PID the CLI cannot prove it owns.**

### 9.2 `start` — required sequence

1. Resolve and validate: loopback bind (refuse `0.0.0.0`, refuse port 0 with a clear reason per §3.5), absolute `--data-dir` (default stable, not cwd), absolute existing `--workspace-root` (refuse a missing dir — the server would jail to cwd), server exe (see §11 for the correct search order).
2. Acquire an instance lock for the data dir (exclusive file handle or lock file with pid+timeout). Two concurrent starts on one data dir must not both spawn: today they can, and two servers can share one SQLite file.
3. Read state. Classify (table §9.6). Never overwrite a state file that records a live managed PID with `pid:null`.
4. Probe `/health` with a 2 s timeout **and validate the body**: `schema == "hiverouter.v1"`, `database.path` canonical-equals `<data-dir>/hiverouter.db`. A 200 from something that is not this instance is `E_PORT_IN_USE`/`E_INSTANCE_FOREIGN`, never `already_running`.
5. If no server: spawn detached, `windowsHide`, no inherited stdio, cwd = workspace root, args `--bind --data-dir --workspace-root`, env = parent env + (only if implemented) the DPAPI key. Record the child's **creation time** for identity checks, and its **stdout/stderr file paths**.
6. Write a `starting` state (pid + created_at + deadline) **before** waiting, so a crash mid-boot is recoverable; then poll readiness (§9.3), aborting immediately if the child exits with the stderr tail in the error.
7. On ready: rewrite state as `ready` with `{pid, pid_created_at, exe(path,size,mtime,sha256), url, bind, data_dir, workspace_root, server_version, contract_schema, db_schema_version, provider_key_present, started_at, logs}`. Atomic write (temp + rename). On failure: clear state, kill only the child you spawned, report the reason.
8. Print the envelope: `{started|already_running, pid, url, health, provider_key_present, managed, data_dir, workspace_root, logs}`.

Readiness itself should consume **both** signals the server already gives: the stdout line `hiverouter <v> listening on http://<addr>` (`main.rs:153-158`; s6-style "readiness is signalled", cf. `docs/research/hmars/27-open-source-daemon-catalog.md:244,251-259`) and `/health` (for `status` and `schema`). Treat the line as "listening", `/health` as "healthy"; a `degraded` 200 is ready with a warning, a 503 is `failing` and must not be reported as ready.

### 9.3 Polling policy (recommended, unifies the four current numbers)

`interval 250 ms`, `deadline 30 s` default (`--ready-timeout`), per-probe HTTP timeout 2 s, abort on child exit, keep the last 20 stderr lines for the failure payload. This sits between the `.cmd` 20 s and the launcher 60 s and matches the e2e's child-exit discipline (the only implementation that does not burn 60 s on a crash).

### 9.4 `status` — required fields

`{running: bool, managed: bool, health|error, pid(from state|OS), url, data_dir, workspace_root(source: "state"|"cmdline"), server_version, contract_schema, db_schema_version, db_path, uptime_secs, provider_key_present(from diagnose|checks), logs{stdout,stderr}, stale_state: bool}`.
Must be **read-only** — no state write, no spawn. It cannot report the PID of an unmanaged server from HTTP (`/health` has no pid, §3.3); either query the OS for the port owner or report `pid: null` honestly.

### 9.5 `stop` — required sequence

1. No `--yes` ceremony for stopping the CLI's **own recorded** instance (that is the contract the caller asked for); `--force` escalation, killing an unmanaged PID, and any `reset` remain explicitly confirmed. (Today: `stop` demands `--yes`, `restart` silently passes `yes:true` — `hivemind-cli.mjs`, `stopServer` and the `restart` dispatch branch.)
2. Read state; if absent → probe `/health`; unreachable → exit 0 `{stopped:false, already_stopped:true}`.
3. Identity-check: pid alive **and** creation time matches **and** (`/health.database.path` matches the state data dir **or** the process command line contains it). Any mismatch → `stale_state` cleanup only; **never kill**.
4. If the server is running but unmanaged (`external:true` / no state) → refuse `E_NOT_OWNED` (exit 3) unless `--force` names the exact pid; `--pid` must equal the recorded pid when both exist (the draft's `stopServer` gets this right and it should stay).
5. Attempt graceful stop (§7; today there is **no working graceful mechanism**), wait `--timeout` (default 10 s) for process exit **and** `/health` unreachable.
6. On timeout: with `--force` → `taskkill /F /PID` (+ `/T` only if you intend to reap tool children) → confirm exit → exit 0 `{stopped:true, graceful:false, escalated:true}`; without `--force` → exit 3 `E_STOP_TIMEOUT` with how-to-fix.
7. Delete state only after both confirmations. Never delete the data dir.

### 9.6 Scenario decision table (the contract this seat was asked to pin down)

| # | Scenario | Detection | Required behaviour | Exit | Envelope fields |
|---|---|---|---|---|---|
| 1 | Already running, managed, identity matches | state.ready + pid alive + pid_created_at match + `/health.database.path` match | no-op success, **do not spawn**, do not rewrite ownership | 0 | `already_running:true, managed:true, pid, url` |
| 2 | Healthy HiveMind at bind, no/other state | `/health` 200 + correct schema, `database.path` ≠ my data dir, or no state | report `already_running:true, managed:false`; **adopt nothing**; `stop` refuses later | 0 | `managed:false, pid:null, data_dir_of_server` |
| 3 | Port listening, not HiveMind (any non-JSON, wrong schema, or non-HiveMind JSON) | `/health` fails or body fails schema/path check **and** TCP connect succeeds | refuse, never kill | 3 | `E_PORT_IN_USE`, `how_to_fix:"choose another --port"` |
| 4 | Stale state, PID dead | state exists, `processAlive(pid)==false` | clear state, then behave as "not running"; `start` proceeds | 0 (stop/status) | `stale_state:true, cleared:true` |
| 5 | PID reused by another process | pid alive but creation time/image path mismatch | treat as stale/managed-unknown; **never kill** | 0 status / 3 stop | `stale_state:true, pid_reused:true` |
| 6 | Wrong workspace chosen | requested `--workspace-root` missing/not a dir, or state's workspace ≠ resolved | refuse before spawn | 2 or 3 | `E_FILE_TREE_UNAVAILABLE` (existing) or a new `E_WORKSPACE_INVALID` only with a contract-change row (§9.8) |
| 7 | No server, no state | health unreachable, port closed | spawn per §9.2 | 0 | `started:true, pid, url` |
| 8 | Server answers `failing` (required check / DB) | `/health` 503 | not ready; keep the tail; kill the child you just spawned | 3 | `health.status:"failing", checks[]` |
| 9 | Server answers `degraded` (key or Node missing) | `/health` 200 `degraded` | ready, with an explicit warning; `provider_key_present:false` | 0 | `health.status:"degraded"` |
| 10 | No provider key | `deepseek_api_key` check ok=false or `/diagnose` false | start is still allowed by default; `--require-key` refuses | 0 / 3 | `provider_key_present:false, key_source:"checks"` |
| 11 | Server exe missing | resolution fails | refuse with the expected paths, no spawn | 3 | `E_SERVER_NOT_FOUND` |
| 12 | Non-loopback bind requested | parse | refuse before spawn (server would refuse `E_ORIGIN_REJECTED`) | 2 | `E_CLI_UNSAFE` |
| 13 | Port 0 requested | parse | refuse with the origin-allowlist reason (§3.5) | 2 | `E_CLI_USAGE` |
| 14 | Concurrent `start` on one data dir | lock held | wait briefly, then report the winner; never spawn a second server | 0 or 3 | `locked_by_pid` |
| 15 | Second `start` with same data dir, different port | lock + running managed instance | refuse: one data dir = one server | 3 | `E_INSTANCE_CONFLICT` |
| 16 | Child exits during boot | `waitpid`/exit event | fail fast with stderr tail; clear state; do not retry blindly | 3 | `E_SERVER_START_FAILED, stderr_tail` |
| 17 | Unreachable mid-command | connect/read error | named transport failure, retryable | 3 | `E_WS_DISCONNECTED, retryable:true` |
| 18 | `stop` with nothing running | no state, health unreachable | idempotent success | 0 | `already_stopped:true` |
| 19 | `restart` while a run is active | `/api/history/runs` shows active, or health `last_seq` changing | **warn and refuse without `--force`**: restart interrupts runs (`main.rs:84-106`, `kill_restore.rs`) and expires approvals | 3 / 0 with force | `active_runs:[…], destructive:true` |

### 9.7 Exit codes and envelope — one table, not two

Today there are **two incompatible contracts** for the same idea:

| Condition | Shipped Rust CLI (`cli.rs:50-57`) | In-flight Node CLI (`hivemind-cli.mjs:31-39`) |
|---|---|---|
| success | 0 | 0 |
| usage / parse | 2 | 2 |
| named refusal | 3 | 1 |
| internal | 4 | — (falls back to 1) |
| "not running" | — | 3 |
| auth | — | 4 |
| timeout | — | 5 |
| unsafe/confirmation | — | 6 |

Recommendation: if lifecycle lands in `hiverouter-cli`, keep its table (0/2/3/4) and map lifecycle conditions to named refusals; `status`/idempotent no-ops stay 0; do **not** add a "not running" code that collides with "refusal". If the Node CLI supersedes it, publish the Node table in the runbook and change the Rust CLI in the same release — never ship both tables. Envelope should stay `hiverouter.cli.v1` with `{schema, ok, command, data|error, meta}`; the Node draft emits `{ok, command, data, meta{cli_schema}}` with **no top-level `schema`** (`hivemind-cli.mjs`, `emitSuccess`/`emitFailure`) — an agent parsing `schema` breaks.

### 9.8 Refusal vocabulary

New lifecycle codes are a **contract change** — `refusal.rs:5-7` says adding a variant needs a `CHANGE-LEDGER` row and a command-matrix re-run. Truthful reuse without a contract change:

| Need | Existing code | Fit |
|---|---|---|
| server unreachable / mid-command drop | `E_WS_DISCONNECTED` (retryable) | good |
| no usable workspace root | `E_FILE_TREE_UNAVAILABLE` ("start the server with `--workspace-root`", `refusal.rs:229-231`) | good for the workspace error |
| key absent | `E_KEY_MISSING` | good |
| DB locked by the other instance | `E_DB_BUSY` (retryable) | good |
| start failed / internal | `E_INTERNAL_ERROR` | acceptable |
| port held by a non-HiveMind | **none** | needs `E_PORT_IN_USE` (new) or the CLI may use a CLI-local code, clearly not a wire `E_*` |
| already running / not running / stale pid / pid reused | **none** | same decision |

So the honest plan is: ship lifecycle diagnostics as `data`/`detail` strings plus the closest existing code, and only introduce `E_PORT_IN_USE`/`E_INSTANCE_CONFLICT`/`E_NOT_OWNED` if the owner accepts a contract-change row.

---

## 10. Missing supervision pieces (ranked)

| # | Gap | Evidence | Severity | Recommended shape |
|---|---|---|---|---|
| 1 | No persistent instance record (pid/url/owner/identity) | repo search: none in `backend/crates`; only console text (`start-hivemind.ps1:71-73`) and the untracked Node draft | **P0** for agents | state file per data dir, atomic, with creation-time identity (§9.2) |
| 2 | No graceful remote stop | §7 measured: `taskkill` without `/F` no-ops on headless servers; tokio handles only `CTRL_C` (`main.rs:178-184`) | **P0** | pick option A/B/C in §7; until then report `graceful:false` |
| 3 | Health contract does not identify the instance | `/health` has no pid/workspace/data-dir-of-UI/build (`http.rs:456-476`) | **P0** (wrong-instance starts) | add `pid`, `workspace_root`, `data_dir`, `build` to `/health` (+ schema note) or require OS-level checks |
| 4 | "Already running" is decided without identity | `.cmd` reuses any healthy 8099 (`start-hivemind.ps1:17-24`); launcher reuses on 2xx (`HiveMindLauncher.cs:45,217-231`); the 01:19:06 draft now checks schema + `database.path` (`startServer`, E_INSTANCE_FOREIGN) but not the workspace root | **P0** | compare `database.path` (state) + workspace (cmdline) |
| 5 | One data dir can host two servers | no lock; draft spawns on a second port with the same `--data-dir` (`startServer`, no lock) | **P0** | exclusive lock file in the data dir; refuse second start |
| 6 | No build/deploy fingerprint | `/health.version` only; the "Live/source identity … no build hash … MISSING-GATE" row in `docs/research/hmars/26-scale-security-audit.md:91` records the same gap | P1 | record exe `{size,mtime,sha256}` in state; optionally add a build id to `/health` |
| 7 | Port policy is inconsistent and partly false | 8080 default (`config.rs:10`) vs 8099 twice (`start-hivemind.ps1:4`, `HiveMindLauncher.cs:16`); README promises a free port (`build-desktop.ps1:69-70`) | P1 | one default, explicit `--port`, reserve-then-bind retry (e2e pattern) |
| 8 | No readiness contract | four timeouts/poll rates (§4); `failing` misread by the launcher | P1 | single `--ready-timeout`, schema+path validation, child-exit abort |
| 9 | No supervisor / restart policy | nothing restarts a crashed server; `docs/research/hmars/27-open-source-daemon-catalog.md:251-259` names the six missing facts | P1 | operator-driven `start` after crash detection; if auto-restart is added, rate-limit it and record attempts (no fork-bomb) |
| 10 | Logs are not a service | `.cmd` truncates each start (`RUN`), Node appends forever, no rotation/size cap, the draft's `logs` reads the whole file into memory (`readTail` in `executeLocal`) | P1 | append + size cap + `logs --lines` with a bounded tail read |
| 11 | Workspace root is silent and defaulted | `.cmd` did not pass it at HEAD (§2.3; a ~01:22 working-tree edit passes repo root); draft validates it (01:19:06) but defaults to the repo root; `/health` cannot verify it | P1 | require an explicit existing `--workspace-root`; expose it in `/health` |
| 12 | No key provisioning on the CLI path | draft reads no DPAPI (`spawn({env: process.env})`), so `start` from a CLI can silently produce a keyless server even though `provider.key` exists | P1 | optional DPAPI injection matching `HiveMindLauncher.cs:129-161`, presence-only reporting |
| 13 | Shutdown/close events unhandled | `CTRL_CLOSE`/`CTRL_SHUTDOWN`/`CTRL_LOGOFF` not installed (`main.rs:178-184`) | P2 | add them to the shutdown future; log which event fired |
| 14 | Exit-code taxonomy on the server | every failure is exit 1 (`main.rs:18-24`) | P2 | distinct codes for bind/db/config (helps supervisors) |
| 15 | Launcher/script portability | hard-coded `C:\Users\Riley\Desktop\hiverouter-v1` (`HiveMindLauncher.cs:17,190`); `dist` README calls the folder portable | P2 | derive from exe location; env override already exists (`HIVEMIND_WORKSPACE_ROOT`, `HIVEMIND_ROOT`) |
| 16 | Doc drift on the shell itself | `runbook.md:3-8` says no WebView2 shell; `build-desktop.ps1:70` says WebView2; both are Edge/Chrome `--app` | P2 | pick one sentence and use it everywhere |

---

## 11. Snapshot assessment of the in-flight CLI (`tools/hivemind-cli.mjs`, untracked)

**Revisions read:** 58,886 B at `01:14:22`, 68,085 B at `01:19:06`, and 72,160 B at `01:21:23` — the file kept changing while this document was written, so **this section is a time-stamped audit, not a current-state claim**. Surface files at the second read: `HiveMind-Agent.cmd` / `HiveMind-Agent.ps1` (root wrappers, `01:15:36`), `tools/hivemind.cmd` / `tools/hivemind.ps1` (`01:14:40`), `docs/AGENT-CLI.md` (`01:17:25`), `skills/` (new). It is a genuine dual-surface CLI (32 commands) with a good safety vocabulary (`--yes` for destructive, `--dry-run`, `--json`, tier/transport metadata) and correct instincts: detached spawn with `windowsHide`, stdout/stderr to files, a state file, `--format`, redacted `/api/session` token, and a refusal to kill a PID that does not match its state. Line numbers below are from the `01:19:06` read; function names are the stable anchors, and the 01:21:23 read already showed two of the items moving (see 11.1).

### 11.1 Fixed across the revisions read (do not re-report as open)

| Was | Now | Evidence |
|---|---|---|
| No instance identity check (any 200 = "already running") | validates `schema`, `database.path` against the requested data dir, refuses `E_INSTANCE_FOREIGN`, and reports `managed:false` for an unowned healthy instance | `startServer` (`:716-752`) |
| `--dry-run` could write state | dry-run returns the plan **before** any probe/`mkdir`/spawn/state write | `startServer` dry-run guard (`:708-715`) |
| No child-exit detection while waiting | `waitForHealth(baseUrl, timeoutMs, pid, stderrPath)` aborts with `E_SERVER_EXITED` and the last 12 stderr lines | `waitForHealth` (`:648-690`), call `:800` |
| No workspace validation | `resolveWorkspaceRoot` stats the path and refuses `E_WORKSPACE_INVALID` | `:606-618` |
| One-phase state (dead PID could be recorded as `ready`) | the 01:21:23 read shows a `starting` state before the health wait and a `ready` upgrade (with `provider_key_present`) after it | `startServer` `writeState` pair |

### 11.2 Still open at the last read (line numbers from 01:19:06; behaviour re-checked at 01:21:23)

| # | Defect | Evidence | Impact |
|---|---|---|---|
| D1 | **Graceful stop cannot work** — `taskkill /PID /T` without `/F` | `stopServer` (taskkill call, last read `:884`); measured no-op in §7 | every `stop` without `--force` burns 10 s then exits 5; `restart` forces or fails |
| D2 | **Binary search order is wrong** — `backend/target/{release,debug}`, not the documented `target/{release,debug}` | `resolveExe` (`:594-604`) vs `tools/build-desktop.ps1:59` and `e2e/helpers/live-server.ts:24-28`; on this host `target\release\hiverouter.exe` exists, `backend\target\release\hiverouter.exe` does not, but a stale `backend\target\debug\hiverouter.exe` does | can launch a stale/mismatched server; `dist\HiveMind\hiverouter.exe` (first candidate) can be older than `target\release` |
| D4 | **A failed boot leaves a `starting` state that is never cleared** — better than a false `ready`, still stale | `startServer` writes `starting`, then `waitForHealth`; no `clearState` in the failure path | `status` must treat an expired `starting` state as stale; `stop` must not trust its PID forever |
| D5 | **No stale-PID/PID-reuse check** — `processAlive` only | `processAlive` (`:639-647`), `stopServer` | if the recorded PID is recycled, `stop` can kill an unrelated process (only guard is `--pid` equality) |
| D6 | **No lock / second instance allowed** | `startServer` spawn block (`:760-819`): no lock, no "a managed instance already exists" check when the requested port differs; a second server can share `--data-dir` | two servers on one SQLite file, two audit writers, graph authority split |
| D7 | **Destructive-action policy is inconsistent** — `stop` needs `--yes` but `restart` passes `yes:true`; `start` is `mutating` with no confirmation | `stopServer` (`:804-812`), dispatch (`:1866`) | an agent that learns `stop` needs `--yes` is surprised by `restart` |
| D9 | **Envelope/exit table diverges from the shipped CLI** — no top-level `schema`; refusal=1 vs the Rust CLI's 3 | `emitSuccess`/`emitFailure`; `EXIT` map | agent parsers keyed on `schema`/`0/2/3/4` break; two contracts in one repo |
| D11 | **No process-creation-time capture** | state has `pid` but no creation time | cannot prove later that the PID is the process it started |
| D12 | **`logs` reads whole files into memory** | `readTail` in `executeLocal` | a long-running server with a large log can exhaust the CLI; no rotation policy |
| D13 | **No DPAPI key path** | `spawn({ env: process.env })` (`:773`) | keyless starts even when `%LOCALAPPDATA%\HiveMind\provider.key` exists; no key-presence warning in the payload |
| D15 | **`restart` reuses the caller's flags, not the state** | dispatch (`:1865-1867`) spreads `options` into `stopServer`/`startServer` | `restart` without flags can target a different identity than the running instance |
| D16 | **New CLI-local codes are not in the frozen refusal table** — `E_CLI_UNSAFE`, `E_CONFIRMATION_REQUIRED`, `E_INSTANCE_FOREIGN`, `E_WORKSPACE_INVALID`, `E_SERVER_EXITED`, `E_PORT_IN_USE` | `stopServer`, `startServer`, `resolveWorkspaceRoot`, `waitForHealth` | fine as CLI diagnostics, but they must not be presented as wire `E_*` codes; see §9.8 |

What it gets right and should be preserved: the state file beside the DB (`statePath`), `mode 0o600`, no secret in state, `--dry-run` for `stop`/`open`, the "refuse to kill an arbitrary pid" stance, the `--pid` equality check, the loopback-only URL guard, the `stop`/`open`/`logs` separation from read commands, and the `status`/`doctor` split.

---

## 12. Unknowns / evidence still missing

1. **No live runtime proof** in this seat: `/health`, `/diagnose`, and the boot line were read from source, not observed (no HiveMind process was started or probed by instruction). The e2e harness is the existing live-boot proof (`e2e/helpers/live-server.ts:368-460`, run by `node run-guarded.mjs`).
2. **Console-control injection is unproven** (option A/B in §7). It needs one spike against a real server: attach, send `CTRL_C_EVENT`, assert the audit file flushes and the process exits 0. Until then, "graceful stop" must be described as unavailable.
3. **Does `CTRL_CLOSE` reach a `windowsHide` child's console?** Not tested; the server does not handle it either way (§3.6), so it is a non-graceful exit regardless.
4. **`/health` 503 vs `failing` for a `prompt_assets` failure** is source-read (`http.rs:91-101,1167-1202`); no test asserts the 503 body shape for that case.
5. **Whether the Node CLI or the Rust CLI is the intended owner** of lifecycle — unresolved; §9.7's recommendation depends on it.
6. **The Node draft changed three times during this seat** (`tools/hivemind-cli.mjs` 58,886 → 68,085 → 72,160 B at 01:14:22 → 01:19:06 → 01:21:23; wrappers and `docs/AGENT-CLI.md` appeared inside the same window). Line citations in §11 are a snapshot; function names are the stable anchors, and D1/D2/D5/D6 are the ones to re-check first because they were still open at the last read.
7. **Whether any external supervisor (task scheduler, WinSW, user script) starts HiveMind on this machine** — nothing in the repo does; not checked outside the repo.
8. **`/integration-logic`** has no file in this checkout; the integration contract in §9 is derived, not adopted from that skill.

---

## 13. Files this seat touched

Only this file: `docs/trace/agent-cli/lifecycle-map.md` (new directory `docs/trace/agent-cli/`). No product code, test, config, wrapper, or state file was created, modified, or deleted; no HiveMind process was started, stopped, signalled, or queried over HTTP; no browser was launched; no provider key was read, decrypted, printed, or stored.
