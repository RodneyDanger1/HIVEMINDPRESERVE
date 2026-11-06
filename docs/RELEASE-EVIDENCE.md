# HiveMind release evidence

Current release line: **1.1.0**. The **authoritative exact-hash registry is `tools/release-manifest.json`**, written
by `tools/build-release.ps1` for a sealed build. This file is the human-readable evidence record; it never invents
artifact hashes. Exact 1.1.0 artifact hashes appear here only after the sealed 1.1.0 build runs.

## Current: HiveMind 1.1.0

- Version `1.1.0` declared in `web/package.json` and all four `backend/crates/*/Cargo.toml` (verified by
  `node tools/qa/check-docs-truth.mjs`).
- **Streaming:** true SSE streaming is implemented — the provider adapter forwards ordered text fragments to the
  delta sink (`backend/crates/hiverouter-core/src/provider_executor.rs` `stream_reply`), usage settles from the
  documented final chunk, malformed/incomplete streams refuse by name. Offline matrix:
  `backend/crates/hiverouter-core/tests/provider_streaming.rs`; run log `docs/evidence/1.1-backend-tests.log`.
- **Replay:** `HISTORY_REVISION_SET` → read-only `HISTORY_SNAPSHOT`; canvas projects snapshot nodes/edges; browser
  proof recorded in the 1.1 wave.
- **State parity:** 16 states including `interrupted` (backend and web).
- **Artifact hashes:** not yet sealed for 1.1.0. The authoritative values will be the `artifacts` / `package` blocks
  of `tools/release-manifest.json` after `tools/build-release.ps1` runs on the tagged, clean tree; this document must
  then quote those values verbatim (no hand-copied or estimated hashes). Until then `tools/release-manifest.json`
  still records the 1.0.0 sealed build as historical.
- **Live paid-provider streaming smoke on a 1.1.0 artifact:** not yet run (no sealed 1.1.0 binary; no key access).
  Streaming is proven offline (wiremock matrix) plus the 1.1 backend test log, not by a paid call.

## Test evidence at the 1.1 baseline

- Backend 1.1 log: `docs/evidence/1.1-backend-tests.log` (recorded 2026-11-06 15:55; includes the streaming adapter
  matrix). Earlier `docs/evidence/final-gate-tests.log` is the historical 1.0.0 gate log, not the 1.1 gate.
- Web: 46+ unit tests / typecheck / production build at the 1.0.0 baseline; re-run on the sealed 1.1.0 build before
  quoting green counts.

<!-- docs-truth:historical:start -->
## Historical: HiveMind 1.0.0 (SUPERSEDED)

The 1.0.0 artifacts below were sealed before the 1.1 streaming work and are retained for provenance only. They are
**not current** and must not be presented as the current release; the current authority is
`tools/release-manifest.json` after the sealed 1.1.0 build.

### 1.0.0 artifacts (historical)

| Artifact | SHA-256 | Bytes |
|---|---|---:|
| `hiverouter.exe` | `b17eb3d6469c347f623a4dedc94537b15bb633a76215828742624fa6fe062a9a` | 13,253,120 |
| `hiverouter-cli.exe` | `49f46e2be488dd675280f3dae7e8d455ac5eeec2aa46d7fd44f1c2557c386623` | 5,373,952 |
| `dist/HiveMind-1.0.0-win64.zip` | `46305fcb1fe1bee23d3645a94a82635a68ee47fbcd3ce0537df747de3c6105ef` | 8,444,021 |

The 1.0.0 zip contains both binaries, `README.md`, `RUNBOOK.md`, `RELEASE.md`, and `CHECKSUMS.txt`; an extraction
smoke listed all six files and verified the archive opened.

### 1.0.0 release-binary live smoke (historical)

Started `target/release/hiverouter.exe` on `127.0.0.1:8800` with a fresh data directory and the key only in the
process environment.

- `GET /health` → `ok`, version `1.0.0`.
- `GET /` → 200, embedded UI HTML present (`id="root"`), 774 bytes.
- `SETTINGS_STATUS` → `{"provider":"deepseek","model":"deepseek-flash","key_present":true,"thinking":"enabled","reasoning_effort":"high"}`.
- `SUBMIT_PROMPT` `Reply with exactly HIVE-ARTIFACT-OK` → `RUN_CREATED` → nodes/reply events → `REPLY_DONE` usage
  `{prompt:91, completion:8, total:99}` → `RUN_STATUS completed`; reply `HIVE-ARTIFACT-OK`.
- `HISTORY_REVISION_SET` revision 0 → `HISTORY_SNAPSHOT` with 2 graph nodes (real snapshot; not fabricated).
- The key never appeared in the server log, settings payload, or evidence.
- Child-process isolation: shell/tool spawns remove `DEEPSEEK_API_KEY` before execution, pinned by the
  `child_process_does_not_inherit_the_provider_key` test.
- Source commit for that build: `6a2b1c6d461a81e7ee66951d0d0eb6b1063bf540`; manifest: `tools/release-manifest.json`
  (records the 1.0.0 build until the sealed 1.1.0 build replaces it); web-dist hashes were recorded per file.
- Retired limitation: the 1.0.0 leaf reply was emitted as one delta after a non-streaming completion. This behavior
  is superseded by the 1.1.0 SSE streaming; it is historical and must not be stated as current.

### 1.0.0 test evidence (historical)

- Rust workspace on sealed commit `6a2b1c6`: 237 passed / 0 failed / 2 ignored (`docs/evidence/final-gate-tests.log`).
- Web: 46/46 unit tests, typecheck 0 errors, Vite production build green (`docs/evidence/final-web-tests.log`,
  `final-web-build.log`).
- Live provider adapter: planner node + `HIVE-PROVIDER-OK`, usage 99 tokens.
- Key scan: strict credential pattern (`sk-[A-Za-z0-9]{32,}`) = 0 occurrences across tracked non-vendor files.

### 1.0.0 final tag gate (historical)

`v1.0.0` remains immutable. Moving it requires owner confirmation and records the superseded commit.
<!-- docs-truth:historical:end -->
