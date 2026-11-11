# HiveMind downloads and first-run truth (2026-11-10)

There is **no public 1.2 download, MSI, code-signed installer, or auto-updater**. The only current package is the local `dist/HiveMind.zip` built from the working tree.

## Current local package

- Path: `dist/HiveMind.zip`
- SHA-256: `B4BAC18D29560D5C63404649C2953E359FFC6170C1214A3AA1B54DE127811061`
- Built from: `806251b`, `dirty:false`
- Version: `1.2.0-dev`; schema 24.
- Install receipt: `docs/evidence/live/install-1.2.0-dev.json`.

## Build and install (per-user, Windows x64)

```powershell
$root = "C:\Users\Riley\Desktop\hiverouter-v1"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$root\tools\build-desktop.ps1"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$root\tools\install-desktop.ps1" -SkipProvider
```

Prerequisites: Node/npm, Rust/cargo with the bundled `tools\w64devkit`, Git, and .NET Framework `csc.exe`. The build embeds the web UI, so `web/dist` must be built first. Install is per-user under `%LOCALAPPDATA%\Programs\HiveMind\app`; data under `%LOCALAPPDATA%\HiveMind`; Desktop and Start Menu shortcuts.

## First run

- Without a provider key, `/health` is `degraded` with `E_KEY_MISSING`; the UI, templates, reads, approvals surface, snapshots and QA drills work, but **every prompt refuses by name**. Nothing is faked.
- With a key, runs are bounded (depth 3, fan-out 8, 64 agents, 64 provider calls, 40 plan nodes), approval-gated for writes/shell/network, and single-run.
- `shell_exec` is approval-gated arbitrary code execution (git/cargo allowlist), **not an OS sandbox**.
- No continuous daemon, no network remotes, no autonomous code mutation, no vector memory, no HiveWire.

## Uninstall / data

`tools/uninstall-desktop.ps1` removes the installed app and shortcuts. Data under `%LOCALAPPDATA%\HiveMind` is retained unless purged with the documented flags; back up `hiverouter.db` first.

## What is not available

No 1.2 tag or release artifact; `-Release` is PRE-RELEASE exit 3. No clean-profile install receipt, no live paid proof, no 40-node live proof. See the truth audit.
