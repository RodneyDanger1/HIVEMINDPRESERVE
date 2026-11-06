# HiveMind v1 — implementation workspace

Built from the verified blueprint (read-only reference: `C:\Users\Riley\Desktop\hivemind-blueprint`).

## Layout
- `backend/` — Rust workspace (`hiverouter-core`, `-contract`, `-provider`, `-tools`), single localhost binary.
- `web/` — React 19 + TypeScript + Vite 8 + Tailwind 4 + @xyflow/react 12.12.1 canvas UI.
- `tools/` — `hiverouter` CLI, dev scripts, portable `w64devkit` (C compiler for rusqlite bundled).
- `docs/` — plans, claims registers, diagrams, contracts, FigCraft assets/episodes, trace, audit, reports.
- `ledgers/` — UI watchdog + implementation ledgers.

## Toolchain (this host)
Rust is installed per-user as `stable-x86_64-pc-windows-gnu` (no MSVC available); `tools/w64devkit/bin` supplies gcc for
`rusqlite` bundled. Node >=22.12.0 required. No Python runtime, no Docker, no cloud dependency.

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$PWD\tools\w64devkit\bin;" + $env:PATH
$env:CARGO_PROFILE_DEV_DEBUG = "0"; $env:CARGO_INCREMENTAL = "0"
cargo build --release --locked
```

## Status
Phases 1-12 per `docs/plans/PHASE-<n>-EVIDENCE.md`; UI assets per `docs/ui/figcraft/ASSETS-INDEX.json` and
`ledgers/ui-watchdog.json`. Nothing is claimed done without a command and observed output.
