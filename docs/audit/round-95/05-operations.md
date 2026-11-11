# Round 95 — Operations / Install-Truth Audit (HiveMind)

> **Post-audit correction (final tree):** the `http.rs` `/api/evidence/ledger` candidate
> observed during this audit was **discarded**. The current product tree has no such
> modification; installed build remains `806251b`. The “uncommitted” references below
> are audit-time history and are not current-tree claims.

**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **HEAD:** `1129349` ("evidence(qa): add real kill/restart and 41-task scale receipts") · **Working tree at audit time:** one `http.rs` ledger-summary candidate; **discarded before final publication** (final product tree clean). **Installed build:** `806251b`, version `1.2.0-dev`, `dirty:false`.
**Live state observed 2026-11-10T23:27-06:00:** server pid 26008 on `127.0.0.1:8099`, `/health` = `status:"degraded"`, `E_KEY_MISSING`.
**Labels:** `VERIFIED` = I ran it and saw the output; `READ` = read the cited code/doc at HEAD; `UNVERIFIED` = not proven; `NOT DOCUMENTED` = searched, not found.

> **Task claim (task-28).** No shared-task tool is mounted in this session, and `task-28` has **0 matches** in the repo: `audit/ROADMAP-TASKS.csv` uses `T001…` ids (no `task-28` row), and a depth-3 filename/content scan (excluding `w64devkit`, `target`, `node_modules`, `.git`) found nothing. The same limitation was recorded for `task-24` in `01-core.md:184`. The audit is complete; the claim/complete action is **not executable** from here. That is a tooling gap, not a HiveMind finding.

---

## 0. What "the current build" means

Three identities are in play, and they are not the same:

| Identity | Value | Evidence |
|---|---|---|
| Repo HEAD | `1129349` | `git rev-parse HEAD` `VERIFIED` |
| Built/installed package | `806251b`, `1.2.0-dev`, `dirty:false`, server sha256 `d9d769…2c78` | `%LOCALAPPDATA%\Programs\HiveMind\app\BUILD.json` and `dist\HiveMind\BUILD.json` `VERIFIED` (both files byte-equal) |
| Package vs HEAD | installed app == `dist\HiveMind` bytes; HEAD is **two commits ahead** (`0af9ed6` install receipt, `1129349` two live receipts + two QA scripts) | `BUILD.json` commit `806251b` vs HEAD `1129349`; `git log 806251b..1129349` `VERIFIED` |

`1129349` and its parent `0af9ed6` add evidence/QA files only (two live receipts plus `run-crash-drill.ps1`/`run-scale-receipt.ps1`); **no product code changes**. The installed app is therefore code-identical to HEAD. A rebuild today would produce new bytes with a new timestamp and *no* new behaviour. The `/api/evidence/ledger` candidate was **discarded before final publication**, so the final product tree is clean (`http.rs` unmodified).

**Verdict on rebuild/recommit:** **No.** Rebuilding buys zero capability. The ledger endpoint candidate is absent from the final tree and was never in the installed build; it had no UI consumer or route-contract test. Do not build from the audit docs; if the endpoint is revived, commit it with a test/consumer and rerun the battery.

---

## 1. What a new user experiences today, step by step

Prerequisites, in build order (all `READ`):

S1. **Windows x64.** `csc.exe` is resolved at `%WINDIR%\Microsoft.NET\Framework64\v4.0.30319\csc.exe` (`build-desktop.ps1:26,59`) — 64-bit .NET Framework 4.0+ is mandatory.
S2. **Git repo + git on PATH.** The build freezes identity with `git rev-parse HEAD` / `git status --porcelain` (`:33-34`) and refuses to stamp a package if the tree moves mid-build (`:104-108`).
S3. **Node/npm.** Step 1 is `npm.cmd run build` in `web/` (`:38-45`). Required because the UI is compile-time embedded (`static_ui.rs:14-17`) and `web/dist` is **not tracked** (`git ls-files web/dist` → 0; `.gitignore:3` `dist/`).
S4. **Rust/cargo + the bundled w64devkit.** Step 2 prepends `~/.cargo/bin` and `tools\w64devkit\bin`, then `cargo build --release --locked` (`:47-56`). The locked build requires the committed `Cargo.lock`.
S5. **No administrator rights.** Everything is per-user: install root `%LOCALAPPDATA%\Programs\HiveMind\app`, data `%LOCALAPPDATA%\HiveMind`, workspace `%LOCALAPPDATA%\HiveMind\workspace` (`install-desktop.ps1:12-14`).

Then install (`install-desktop.ps1`):

S6. Refuses if `dist\HiveMind\HiveMind.exe` or `hiverouter.exe` is missing — **there is no download/bootstrap path** (`:29-31`, throws "run tools\build-desktop.ps1 first").
S7. Refuses to overwrite a non-empty install root unless `-Force` (`:32-37`).
S8. Creates the three directories, copies the two exes, optional `README.txt`, and `set-provider-key.ps1` / `set-provider-profile.ps1` / `CHECKSUMS.txt` / `BUILD.json` when present (`:39-53`).
S9. Creates **two shortcuts**: Desktop and `Start Menu\Programs\HiveMind` (`:55-67`). No Add/Remove Programs entry, no service, no file association.
S10. Provider setup (default `-Provider deepseek`): writes a **non-secret** `provider.json` (`name`, `api_type`, `model:"deepseek-flash"`), then prompts `Read-Host -AsSecureString` (`:69-90`). Enter with no input = **skip key**, no error. The key, if typed, is DPAPI-encrypted `CurrentUser` scope with entropy `HiveMind.ProviderKey.v1` and written as `provider.key` (`:96-105`). The key never appears on a command line.
S11. Writes `install.json` with install/data/workspace roots and timestamp (`:114-124`).
S12. Unless `-NoLaunch`: `Start-Process HiveMind.exe`, sleep 3 s, then poll **`http://127.0.0.1:8099/health`** up to 20×500 ms (`:131-147`). On failure it prints a *warning* ("launch it from the shortcut and inspect %LOCALAPPDATA%\HiveMind"), not an error.
S13. First real launch runs `HiveMind.exe` (a 10 KB C# winexe). The launcher picks the **first free port in 8099..8199**, or reuses an already-healthy HiveMind (`HiveMindLauncher.cs:16-17,44,277-291`), spawns `hiverouter.exe --bind 127.0.0.1:<port> --data-dir … --workspace-root …` (`:61-62`), waits up to 60 s for `/health` (`:73`), then opens `msedge.exe --app=…` or `chrome.exe --app=…` at 1440×900 with a per-app `--user-data-dir` (`:18-19,90-96`).
S14. Server boot (`main.rs:31-214`): audit sink → SQLite open → migrate → remote reconciliation → kill-restore recovery → environment checks → loopback bind → serve. Every failure prints a named code and exits non-zero; it never pretends to have started.

**Failure modes a new user can actually hit** (each observable):

| # | Failure | What the user sees | Ground |
|---|---|---|---|
| F1 | Build before web build | `rust-embed` has no `web/dist` folder | `static_ui.rs:16` + untracked `web/dist` `READ` |
| F2 | Build without csc | "csc.exe not found at …" | `build-desktop.ps1:59` |
| F3 | Build with a moved tree | "source identity changed during the build … refusing to stamp ambiguous BUILD.json" | `:106-108` |
| F4 | Install before build | "portable package missing at …; run tools\build-desktop.ps1 first" | `install-desktop.ps1:29-31` |
| F5 | Re-install over existing | "install root already exists … (use -Force …)" | `:32-37` |
| F6 | TokenLab/custom without URLs | "TokenLab requires -BaseUrl." / "Custom provider requires -BaseUrl and -WireModel." | `:76-86` |
| F7 | Key skipped / absent | UI loads but every prompt fails **E_KEY_MISSING**; `/health` = `degraded` | `checks.rs:160-166`, live `VERIFIED` |
| F8 | Port 8099 already taken | Launcher serves on 8100+; installer's hardcoded 8099 probe warns spuriously | `install-desktop.ps1:137` vs launcher `:16-17`; 8099 held by pid 26008 `VERIFIED` |
| F9 | Both Edge and Chrome absent | Launcher finds no browser path; window cannot open | launcher `:18-19,90` `READ` (branch not exercised) |
| F10 | Non-IP bind string | `hiverouter: invalid bind address 'localhost:8099': invalid socket address syntax`, non-zero exit | installed `server.stderr.log` `VERIFIED` |
| F11 | Non-loopback bind | Refused: "Only loopback bind addresses are accepted in v1 … E_ORIGIN_REJECTED" | `main.rs:248-249` `READ` |
| F12 | Port bind lost by race | `E_WS_DISCONNECTED: cannot bind …` | `main.rs:155-163` `READ` |
| F13 | Corrupt DB / failed migration / unreconciled control plane | Named refusal printed, process exits failure | `main.rs:65-102,116-123` `READ` |

---

## 2. Without a provider key: what works, what refuses

**Live, `VERIFIED` (pid 26008, `/health`):**

- Server boots, serves the embedded SPA, opens SQLite (`schema_version: 24`), passes `rustc` and `node` checks, loads prompt assets (`prompt_assets.loaded:true`), reports `status:"degraded"`.
- The only failing check is `deepseek_api_key`: "DEEPSEEK_API_KEY is not set; live prompts will refuse E_KEY_MISSING", `code:"E_KEY_MISSING"`, `required:false` — so it is **degraded, not fatal**. The env name is `DEEPSEEK_API_KEY` because the installed `provider.json` selects api_type `deepseek`; the binary default is `HIVEROUTER_API_KEY` (`config.rs:42,103-105`).
- The install receipt confirms the UI renders at 1440×900 and 375×812 with all eight templates while `missing_key:true` (`docs/evidence/live/install-1.2.0-dev.json` `VERIFIED` to contain).
- Key-free and offline: health, provider-profile save, templates, workspace/file browsing (Read class), approvals surface, ledger/audit files, snapshots, kill-restore recovery, and both QA drills (`run-crash-drill.ps1`, `run-scale-receipt.ps1`) which drive local binaries/tests only.

**What refuses, with named codes (`READ`):**

| Code | Trigger | Site |
|---|---|---|
| `E_KEY_MISSING` | any live prompt/LLM call with no key on the selected provider | `checks.rs:160-166`; `execution.rs:585` |
| `E_SPAWN_DEPTH` / `E_FANOUT_LIMIT` / `E_AGENT_LIMIT` / `E_CALL_LIMIT` | depth >3, siblings ≥8, agents ≥64, provider calls ≥64 | `caps.rs:7-16,55-70` |
| `E_TOOL_NOT_ALLOWED` | child spawn across a non-adjacent tier | `execution.rs:250-310` (via `01-core.md` §1) |
| `E_ORIGIN_REJECTED` | non-loopback bind / origin | `main.rs:248-249` |
| `E_WS_DISCONNECTED` | listener bind failure | `main.rs:159` |
| `E_APPROVAL_REPLAY` | reused approval nonce | `approval.rs:1-16` (via `01-core.md` §6) |

Consequence: **without a key the product is a browsable, inspectable shell.** No planning, no agent run, no tool execution, no lesson application, no evidence production. That is honest and named — it does not fake success — but it is also the entire live surface for a keyless user.

---

## 3. Unsafe / unsupported areas (top of the list)

U1. **No OS sandbox.** `shell_exec` runs `git.exe`/`cargo.exe` with the user's full privileges; repo-controlled code can execute. The code says so itself (`01-core.md` §6, `shell.rs:169-181`).
U2. **Non-expiring session approvals.** `AllowSession` never expires (`approval.rs:12-13` via `01-core.md` §6) — one "allow for session" widens blast radius for the rest of the process lifetime.
U3. **EVO-GATE validates shape, not truth.** It never runs the evaluator, never binds `head_commit` to HEAD, never hash-checks artifacts against files, so a hand-authored receipt passes (`01-core.md` §2/§6).
U4. **Exfiltration surface.** `web_fetch` is HTTPS + SSRF-filtered with a 1 MiB cap, but it remains an outbound channel available to approved runs.
U5. **Test residue in the production data dir.** `hiverouter.db-shm.testrun-184830`, `hiverouter.db-wal.testrun-184830`, `hiverouter.db.testrun-184830.testrun-184830`, `hiverouter.pre-v10-2026-11-11T03-02-08Z.bak` sit beside `hiverouter.db` `VERIFIED`; a 4.1 MB stale WAL is present and nothing prunes these.
U6. **A disabled key file is kept in place:** `provider.key.DISABLED-20261107-222030` (262 bytes) remains in the data root `VERIFIED`. It is DPAPI-scoped, but it is still key material on disk.
U7. **Uninstall is destructive and partial.** It removes the whole install root and both shortcuts; `-PurgeData` deletes the entire data root (DB, ledger, snapshots, workspace) irreversibly (`uninstall-desktop.ps1:16-18,30-32`). It only stops a `hiverouter` whose path equals the install root (`:11-15`), so a server spawned from `dist\` survives.

---

## 4. One hundred things HiveMind cannot do right now

Each item is falsifiable against the cited file/line or observed state. `bin/` = `backend/crates/hiverouter-core/src/`.

### A. Install, build and packaging (1–14)

1. Cannot install without a locally built `dist\HiveMind` package; there is no download, bootstrap or release-artifact path. `install-desktop.ps1:29-31`
2. Cannot install from a fresh clone without running the full npm+cargo build first. `build-desktop.ps1:38-56`
3. Cannot obtain the app as an MSI/MSIX/EXE installer; the only artifact is a copied folder plus an optional zip. `build-desktop.ps1:64-71,124-131`
4. Cannot install for all users or into Program Files; the root is hardcoded under `%LOCALAPPDATA%`. `install-desktop.ps1:12`
5. Cannot elevate or register a service; no service/registry/installer API call exists in the install script.
6. Cannot supply the provider key non-interactively; the only path is `Read-Host -AsSecureString`. `install-desktop.ps1:90`
7. Cannot pass the provider key as a parameter; no `-Key`/`-ApiKey` parameter is declared. `install-desktop.ps1:11-22`
8. Cannot reinstall in place without `-Force`; a non-empty root aborts. `:32-37`
9. Cannot build without .NET Framework 4 `csc.exe`; the build hard-fails. `build-desktop.ps1:26,59`
10. Cannot build without npm; the web step precedes the Rust step and its failure aborts. `:38-45`
11. Cannot build without git; identity freezing requires `rev-parse`/`status`. `:33-34`
12. Cannot build without the bundled `tools\w64devkit` on PATH for the C toolchain. `:50`
13. Cannot ship a code-signed binary; no signing step, certificate or timestamp exists anywhere in the build.
14. Cannot auto-update; no update check, feed, or downloader exists in launcher, installer or server.

### B. Launch, UI and runtime (15–24)

15. Cannot choose the listening port from the shortcut; the launcher picks 8099–8199 automatically. `HiveMindLauncher.cs:16-17,44`
16. Cannot run two isolated GUI instances from the shortcut; a healthy existing HiveMind is reused. `:277-291`
17. Cannot open the UI without Edge or Chrome; only those two absolute paths are probed. `:18-19,90`
18. Cannot open the UI in Firefox or the system default browser. `:18-19,90`
19. Cannot run headless as a Windows service; the entry point is a `winexe` that opens a browser app window.
20. Cannot update the UI without rebuilding the Rust binary; `web/dist` is embedded at compile time. `static_ui.rs:14-17`
21. Cannot serve a changed UI by copying new web files into the install root; the assets are inside the exe.
22. Cannot deep-link to an asset missing from the embedded archive; unknown asset paths are a hard 404. `static_ui.rs:12-13,60-70`
23. Cannot get an installer health verdict on a non-8099 launch; the probe is hardcoded to 8099. `install-desktop.ps1:137`
24. Cannot ask the server for its bound port over a well-known port; discovering it requires the launcher scan or the log line. `main.rs:172-177`

### C. Provider and model (25–34)

25. Cannot make any live model call with no key configured; every prompt path refuses `E_KEY_MISSING`. `bin/checks.rs:160-166`
26. Cannot run a fully offline/local model; no local-inference adapter exists in the provider crate surface.
27. Cannot use two providers at once or fail over between them; one `provider.json` profile is loaded. `bin/config.rs:308-334`
28. Cannot route different tasks to different models; one profile model (default `deepseek-flash`) is written by the installer. `install-desktop.ps1:70-74`
29. Cannot select a model per run from the CLI; `--help` exposes bind/data/db/workspace only. `main.rs:230-239`
30. Cannot use a custom endpoint without both `-BaseUrl` and `-WireModel`; the installer throws. `install-desktop.ps1:80-86`
31. Cannot store the key outside DPAPI-CurrentUser scope; another user or a service account cannot decrypt it. `:96-105`
32. Cannot recover the key after a Windows profile change; DPAPI CurrentUser scope binds it to that profile.
33. Cannot exceed 64 provider calls per run; the cap refuses with `E_CALL_LIMIT`. `bin/caps.rs:7-16,55-70`
34. Cannot avoid one extra provider call per hierarchical run for the post-dispatch monitor. `01-core.md` §1 (`provider_executor.rs:709-733`)

### D. Orchestration, caps and settlement (35–50)

35. Cannot exceed depth 3; deeper spawns refuse `E_SPAWN_DEPTH`. `bin/caps.rs:7-16`
36. Cannot exceed 8 siblings per parent; refuses `E_FANOUT_LIMIT`. `bin/caps.rs:7-16`
37. Cannot exceed 64 agents per run; refuses `E_AGENT_LIMIT`. `bin/caps.rs:7-16`
38. Cannot spawn across a non-adjacent tier (Hyper→Agent, Super→Subagent); refuses `E_TOOL_NOT_ALLOWED`. `01-core.md` §1 (`execution.rs:250-310`)
39. Cannot give the root more than two control turns of at most four tool calls each. `01-core.md` §1 (`execution.rs:4475-4536`)
40. Cannot add a second depth-1 branch once one exists; the root spawn gate refuses. `01-core.md` §1 (`execution.rs:2342-2398`)
41. Cannot supervise mid-flight; the post-dispatch monitor is read-only and after settlement. `01-core.md` §1
42. Cannot cancel, repair or re-dispatch a task from the monitor; if it emits tool calls the run fails. `01-core.md` §1 (`execution.rs:2384-2387`)
43. Cannot run two runs at once; `active_run` is singular and reported as `busy`. `bin/http.rs:3622`
44. Cannot execute tasks in parallel within a run; dispatch is a single-consumer loop. `01-core.md` §6 (`execution.rs:2585-2606`)
45. Cannot retry settlement indefinitely; it stops at 64 iterations with an internal error. `01-core.md` §6 (`execution.rs:2557-2583`)
46. Cannot reassign a failed task by the HyperAgent; failure handling is structural, not reasoned.
47. Cannot replan after dispatch; the plan is produced by one planner call up front.
48. Cannot exceed the README's documented 40 tool calls / 12 tool turns per run if it is enforced. `README.md:44-46` `READ` (doc; enforcement site not re-read)
49. Cannot run a ToolAgent; the tier has grants but no producer. `README.md:28`; `01-core.md` §6
50. Cannot rely on a persona to widen tools; effective surface is tier ∩ persona. `README.md:40-42`; `01-core.md` §6

### E. Tools, permissions and containment (51–64)

51. Cannot execute shell commands without an approval; `shell_exec` is an `Execute`-class tool. `01-core.md` §6 (`tool.rs:57-61`)
52. Cannot run arbitrary interpreters; python/node-style interpreters are denied by the shell allowlist. `01-core.md` §6 (`shell.rs:169-181`)
53. Cannot contain shell side effects; there is no OS sandbox, job object or container. `01-core.md` §6
54. Cannot make approvals expire within a session; `AllowSession` is non-expiring. `01-core.md` §6 (`approval.rs:12-13`)
55. Cannot reuse an approval; the nonce is single-use and replays refuse `E_APPROVAL_REPLAY`. `01-core.md` §6 (`approval.rs:1-16,31`)
56. Cannot extend an approval past its 120-second TTL. `01-core.md` §6
57. Cannot bind an approval to anything but `(run, tool, args_sha256)`. `01-core.md` §6
58. Cannot write files outside the workspace jail. `main.rs:237`; `shell.rs` jail contract
59. Cannot `web_fetch` over plain HTTP; HTTPS only. `README.md:316-324` (via `01-core.md` §5)
60. Cannot fetch more than 1 MiB per `web_fetch`. `README.md:316-324`
61. Cannot use more than 12 tool names; the registry is fixed and static. `01-core.md` §6 (`tool.rs:462-518`)
62. Cannot add a tool at runtime; grants are compile-time tier tables. `01-core.md` §6
63. Cannot exceed 2 A2A attempts per run if the documented budget is enforced. `README.md:44-46`
64. Cannot make an un-approved `Network` or `Mutate` call succeed by design; both are approval-gated. `01-core.md` §6

### F. Memory, learning and evolution (65–76)

65. Cannot do semantic/vector recall; retrieval is recency-ordered SQL over banks. `01-core.md` §3 (`store.rs:2217-2270`)
66. Cannot embed or index text; no embedding call or ANN index exists. `01-core.md` §3
67. Cannot read another run's memory; banks are run/hive/branch scoped. `01-core.md` §3
68. Cannot address a sibling branch's bank. `01-core.md` §3
69. Cannot exclude unverified memory; the trust filter admits `unverified` entries. `01-core.md` §3 (`store.rs:2241-2242`)
70. Cannot dedupe or reconcile conflicting memory beyond a unique active key. `01-core.md` §3 (`db.rs:432-433`)
71. Cannot change weights, skills or code through the lesson loop; applied lessons are prompt text only. `01-core.md` §2
72. Cannot auto-apply an improvement; proposals record `applied:false` and `requires_human_review:true`. `01-core.md` §2 (`README.md:131-134`)
73. Cannot apply a lesson without four SuperAgent approvals plus the operator seat. `01-core.md` §2 (`http.rs:2522+`)
74. Cannot apply a lesson without a pre-placed eval receipt JSON; the gate never runs the evaluator. `01-core.md` §2 (`evolution_gate.rs:13-15,76-166`)
75. Cannot make EVO-GATE compare the receipt's `head_commit` to HEAD; it is parsed and never compared. `01-core.md` §2
76. Cannot verify an artifact hash in a receipt corresponds to a real file; only the 64-hex shape is checked. `01-core.md` §2 (`evolution_gate.rs:144-157`)

### G. Observability, API and evidence (77–86)

77. Cannot see a ledger summary in the UI; no ledger-summary route exists in the final tree. `git status`; repo grep → 0 matches `VERIFIED`
78. Cannot get exact ledger totals above 8 MiB; no ledger-summary route exists (candidate discarded). Final tree grep → 0
79. Cannot filter a ledger summary by time, run or tool; no such endpoint exists in the final tree.
80. Cannot claim the committed full-battery receipt is green for HEAD; it records `source_commit: be03094…` with `dirty:true`. `docs/evidence/live/full-battery-20261110-215831.json` (per `01-core.md` preamble)
81. Cannot claim the install receipt proves a live provider call; it states `missing_key:true` and says so in `what_this_does_not_prove`. `install-1.2.0-dev.json` `VERIFIED`
82. Cannot claim the scale receipt is a 40-node run; it is a 41-task cargo test and its `note` says so. `scale-drill-20261110-230653.json` `VERIFIED`
83. Cannot claim the crash receipt is the installed app; it drives `target\release\hiverouter.exe`. `crash-drill-20261110-230629.json` `VERIFIED`
84. Cannot query metrics, traces or a dashboard; health exposes checks and counters only. `bin/http.rs` health `VERIFIED`
85. Cannot get structured logs by default; boot noise goes to fixed `server.stdout.log` / `server.stderr.log` files. observed `VERIFIED`
86. Cannot rotate or bound those logs; nothing truncates or rotates them.

### H. Remote, daemon and multi-device (87–92)

87. Cannot connect a remote agent over the network; the remote principal is an in-process fixture. `01-core.md` §4 (`README.md:26`)
88. Cannot enroll a second device; there is no transport, pairing or discovery. `01-core.md` §4
89. Cannot schedule a daemon tick in the OS; the watchdog has a DB check but no live scheduled process. `01-core.md` §4 (`README.md:27,187-188`)
90. Cannot prove daemon liveness; only reconcile-on-boot is exercised. `main.rs:90-102`
91. Cannot speak any third-party agent protocol; `HiveWire` returns 0 matches repo-wide. `01-core.md` §4
92. Cannot sync or share a workspace between machines; the workspace is a local directory. `main.rs:237`

### I. Data, lifecycle and operations (93–100)

93. Cannot uninstall from Windows Settings; no Add/Remove Programs entry is created. `install-desktop.ps1:55-67` (shortcuts only)
94. Cannot uninstall while preserving the workspace selectively; only "keep everything" or `-PurgeData` (all). `uninstall-desktop.ps1:30-32`
95. Cannot guarantee the server dies on uninstall; only a process whose path equals the install-root exe is stopped. `:11-15`
96. Cannot back up or restore the DB through a product command; recovery is boot-time migration plus an ad-hoc `pre-v10` `.bak`. observed `VERIFIED`
97. Cannot keep the production data dir clean of test artifacts; `.testrun-184830` DB/WAL/SHM files and a 4.1 MB WAL remain. observed `VERIFIED`
98. Cannot pin schema downgrades; migration is forward-only and boot refuses on failure. `main.rs:77-87`
99. Cannot run multi-user or multi-tenant; one loopback user, one data root, one workspace. `install-desktop.ps1:12-14`
100. Cannot report crashes anywhere; there is no crash upload, telemetry or support bundle (`server.stderr.log` is the only trace, 81 bytes `VERIFIED`).

---

## 5. Operational risks

| Risk | Severity | Why | Evidence |
|---|---|---|---|
| R1 — Keyless by default after install | High | S10 lets the user press Enter and skip; the product then looks "installed" but every run fails | `install-desktop.ps1:90-95`; live `E_KEY_MISSING` |
| R2 — Installer health check lies on busy 8099 | Medium | Hardcoded 8099 vs 8099–8199 launcher scan → warning on a healthy launch | `:137` vs launcher `:16-17` |
| R3 — `dirty:true` builds possible | Medium | Any uncommitted change (today: `http.rs`) makes the next build unverifiable | `build-desktop.ps1:34,105-108` |
| R4 — Approval fatigue → non-expiring session grants | High | `AllowSession` never expires and shell is unsandboxed | `01-core.md` §6 |
| R5 — EVO-GATE is an admission filter, not measurement | High | Operator-authored receipt passes without an evaluator run | `01-core.md` §2 |
| R6 — WAL growth in data root | Medium | 4.1 MB `hiverouter.db-wal` plus a stale test WAL; no checkpoint/prune observed | observed `VERIFIED` |
| R7 — Disabled key retained on disk | Low-Med | `provider.key.DISABLED-…` (262 B) still present | observed `VERIFIED` |
| R8 — Stale package vs HEAD confusion | Medium | `dist`/installed = `806251b`; HEAD = `1129349`; a reader may think the receipts commit changed the product | `BUILD.json` vs `git rev-parse` |
| R9 — Crash drill tests a different binary than installed | Medium | Drill uses `target\release\hiverouter.exe`, not the installed copy | crash receipt `server` field |
| R10 — No code signing | Medium | SmartScreen/AV friction on first run; integrity rests on SHA-256 files only | absent from build |

## 6. Evidence gaps

G1. I did not launch the app from the shortcut during this audit; the browser-window path (launcher `:73-104`) is `READ`, not exercised. The running server predates this session (uptime 1331 s).
G2. I did not run `build-desktop.ps1` or `install-desktop.ps1`; build/install behaviour is `READ`, with the installer's own receipt (`install-1.2.0-dev.json`) as corroboration only.
G3. The latest battery receipt (`full-battery-20261110-232942.json`) is PASS but `working_tree_dirty:true` at `1129349`; it cannot support a **clean**-HEAD green-suite claim. Earlier receipts cite older commits.
G4. The ledger endpoint candidate was discarded and never committed or installed; its error paths were not tested.
G5. No live provider call was possible (`E_KEY_MISSING`), so every claim about model behaviour is code/doc-derived.
G6. Launcher behaviour when neither Edge nor Chrome exists was not exercised (F9).
G7. The 100-item list leans on `01-core.md` for code-cited facts I did not re-open line by line (`caps.rs`, `tool.rs`, `approval.rs`, `evolution_gate.rs`); those are second-hand `READ`-via-audit and flagged where used.

---

## 7. Report shape

**Done**

| Item | Evidence |
|---|---|
| Read all five operational scripts | `tools\build-desktop.ps1` (135 L), `install-desktop.ps1` (148 L), `uninstall-desktop.ps1` (36 L), `qa\run-crash-drill.ps1` (73 L), `qa\run-scale-receipt.ps1` (37 L) `VERIFIED` |
| Read backend entry point | `bin/main.rs:1-255` `VERIFIED` |
| Inspected installed app and data root | `%LOCALAPPDATA%\Programs\HiveMind\app` (BUILD.json `806251b`, dirty false, same sha as `dist`) and `%LOCALAPPDATA%\HiveMind` listing `VERIFIED` |
| Queried live health | `/health` = `degraded`, `E_KEY_MISSING`, schema 24, pid 26008 `VERIFIED` |
| Read 3 live receipts | install / crash / scale JSON `VERIFIED` |
| Answered rebuild question | HEAD `1129349` = evidence-only; no capability delta; rebuild yields `dirty:true` `VERIFIED` |
| 100 grouped cannot-do items | §4, each with file:line or observed state |
| This report | `docs/audit/round-95/05-operations.md` |

**Not done**

| Item | Reason |
|---|---|
| Claim/complete `task-28` | No shared-task tool mounted; `task-28` has 0 matches in repo (`ROADMAP-TASKS.csv` uses `T###`) |
| Execute build/install end-to-end | 8-minute budget; the installed app is running on 8099 and would have to be stopped |
| Launch UI and screenshot from the shortcut | Not exercised; existing GUI evidence reused, flagged |
| Compile/test the ledger endpoint candidate | Discarded before publication; not tested |

**Unverified**

| Claim | What would prove it |
|---|---|
| Launcher failure when no Edge/Chrome exists | Temporarily rename both browser paths and launch |
| Installer warning path on busy 8099 | Run install with 8099 occupied (already true) |
| `AllowSession` never expiring in practice | Approve with session scope, wait past 120 s, retry |
| Ledger endpoint correctness/limits | `cargo test` plus a seeded ledger > 8 MiB |
| README's 40-call/12-turn/2-A2A budgets are enforced | Read the enforcement sites (`caps.rs` / `tool_loop.rs`) |
