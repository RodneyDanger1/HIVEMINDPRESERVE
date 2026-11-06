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

## Status — SHIPPED v1 (2026-11-06)

- Final binary: `tools/bin/hiverouter.exe` sha256 `184D51A7C3CCA7B9D50954B677C5AD8A14472700A651FDF0D270441BE3EAC9A9`; package `dist/HiveMind-v1-win64.zip` sha256 `9B2ED5A7B785F1E8422A79BD26672DF176325F982DB9BC68778C7050EBC2B144`.
- Backend: 233 passed / 0 failed / 1 ignored across contract/core/provider/tools. Frontend: 46 unit tests, typecheck/build green.
- Live browser e2e on the exact binary: 79 passed / 0 failed / 14 explicit fixme skips (no paid provider run).
- Full evidence + limitations: `docs/reports/FINAL-REPORT.md`; audit: `docs/audit/PHASE-B-VERIFICATION.md` / `FINAL-DELTA-VERIFICATION.md`; trace: `docs/trace/`.