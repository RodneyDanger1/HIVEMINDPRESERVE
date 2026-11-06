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

## Status — HiveMind 1.0 (2026-11-06)

- First official build with the real provider path: `SUBMIT_PROMPT` → planner DAG → leaf reply → `REPLY_DONE` usage → run `completed`. Live evidence: `docs/evidence/key-validation.md` and `docs/evidence/live-provider-smoke.md`.
- Release artifacts: `target/release/hiverouter.exe` + `hiverouter-cli.exe`; package/checksums in `dist/` (see `docs/RELEASE.md` for the final hashes). Current 1.0.0 values: server `b17eb3d6…062a9a` 13,253,120 B; CLI `49f46e2b…386623` 5,373,952 B; package `dist/HiveMind-1.0.0-win64.zip` `46305fcb…6105ef` 8,444,021 B (`docs/RELEASE-EVIDENCE.md`, `tools/release-manifest.json`). `docs/reports/FINAL-REPORT.md` is the historical pre-1.0.0 report and carries a dated supersession note.
- Backend: full Rust workspace green after provider wiring; frontend: 46 unit tests, typecheck and production build green.
- Key: `DEEPSEEK_API_KEY` only, never committed or shown in UI; model/thinking/effort settings documented in `tools/runbook.md`.
- Known limits (stated, not hidden): leaf replies are returned as one delta after a non-streaming completion; autonomous model tool-calling and full trace/pixel reviews are 1.1; the normal test suite never calls the provider (the `provider_live` test is `#[ignore]` and gated by the key).
- Audit: `audit/HIVEMIND-AUDIT-PAPER.md`; release notes: `docs/RELEASE.md`.