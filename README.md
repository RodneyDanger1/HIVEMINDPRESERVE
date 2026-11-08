# HiveMind — implementation workspace (sealed release 1.1.0)

Built from the verified blueprint (read-only reference: `C:\Users\Riley\Desktop\hivemind-blueprint`).

> **Version truth:** this README documents the sealed **1.1.0** release. The working tree is on an unreleased
> **1.2 pre-release** source line (exact version and evidence: [docs/RELEASE-EVIDENCE.md](docs/RELEASE-EVIDENCE.md)).
> No 1.2 feature is released, built into an artifact, tagged, or end-to-end verified; do not present 1.2 work as current.

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

## Status — sealed release 1.1.0 (2026-11-06)

- **Sealed release: `1.1.0`** — at seal time, `web/package.json` and all four `backend/crates/*/Cargo.toml` agreed on
  1.1.0; the working tree has since moved to the 1.2 pre-release line (see the version-truth note above). The 1.1 wave
  delivers true SSE streaming on top of the real provider path.
- **Streaming is real in 1.1.0:** the leaf reply is streamed as ordered `REPLY_DELTA` frames while the provider
  response arrives (`backend/crates/hiverouter-core/src/provider_executor.rs` `stream_reply`), with usage settled
  from the documented final chunk and named refusals on malformed/incomplete streams. The offline wiremock matrix
  `backend/crates/hiverouter-core/tests/provider_streaming.rs` pins ordered fragments, cancellation and
  pre-first-byte retry without any key or network call. The retired 1.0.0 one-delta behavior is historical only.
- **Replay and state parity:** `HISTORY_REVISION_SET` returns a read-only `HISTORY_SNAPSHOT`; the web canvas
  projects snapshot nodes/edges (browser proof in 1.1) and the UI/backend state vocabulary is 16 states including
  `interrupted`.
- **Release artifacts / exact hashes:** the sealed 1.1.0 build ran; its authoritative exact-hash registry is
  `tools/release-manifest.json`, summarised in `docs/RELEASE-EVIDENCE.md`. No 1.2 manifest or artifacts exist — do not
  hand-copy hashes into this README, and do not present 1.0.0 or 1.2 values as current.
- **Key custody:** `DEEPSEEK_API_KEY` only, never committed or shown in UI; provider type zeroized/redacted; child
  processes are spawned with an allowlisted environment (no key inheritance). Model/thinking/effort settings are
  documented in `tools/runbook.md`.
- **Known limits (stated, not hidden):** the sealed 1.1.0 artifact has a recorded paid live streaming/replay smoke
  (`docs/evidence/live-1.1-paid.log`, 5 passed, exit 0; frames in `docs/trace/live/captures/w11/`). No 1.2 live run,
  1.2 release build, full 1.2 E2E/release gate, or 1.2 trace capture exists. Autonomous model tool-calling remains
  unclaimed for 1.1.0; full trace-ladder/pixel reviews remain partial. `docs/evidence/1.1-backend-tests.log` is the
  1.1 backend run log.
- **Audit / release notes:** `audit/HIVEMIND-AUDIT-PAPER.md`; release notes `docs/RELEASE.md`; exact hash registry
  `docs/RELEASE-EVIDENCE.md` + `tools/release-manifest.json`.

## Agent-native CLI (headless)

`HiveMind-Agent.cmd` (or `tools/hivemind-cli.mjs`) is the same authority surface
the UI uses, exposed for agents and orchestration systems:

```powershell
.\HiveMind-Agent.cmd start --json
.\HiveMind-Agent.cmd health --json
.\HiveMind-Agent.cmd commands --json
.\HiveMind-Agent.cmd schema submit --json
.\HiveMind-Agent.cmd stop --yes --json
```

stdout is JSON when piped; refusals carry `code`, `message`, `how_to_fix` and a
stable exit code. It never reads or prints the provider key, and keyless
provider commands refuse by name. Full command list: [`docs/AGENT-CLI.md`](docs/AGENT-CLI.md);
agent skill: [`skills/hivemind/SKILL.md`](skills/hivemind/SKILL.md).

<!-- docs-truth:historical:start -->
## Historical: HiveMind 1.0.0 (SUPERSEDED)

- 1.0.0 was the first provider-wired build. Its sealed artifact hashes, live smoke and retired limitations
  (leaf reply as a single delta after a non-streaming completion) are preserved in the historical section of
  `docs/RELEASE-EVIDENCE.md` and `docs/claims/CLAIM-REGISTER.csv` (rows marked SUPERSEDED). They are not current
  and must not be presented as the current release.
<!-- docs-truth:historical:end -->
