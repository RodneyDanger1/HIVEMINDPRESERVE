# HiveMind v1 — FINAL REPORT (shipped artifact)

Date: 2026-11-06 (America/Chicago). Workspace: `C:\Users\Riley\Desktop\hiverouter-v1`.
Blueprint source of truth: `C:\Users\Riley\Desktop\hivemind-blueprint` (read-only; hashes verified 9/9 at start).

## Shipped artifact

| Artifact | Value |
|---|---|
| Binary | `tools\bin\hiverouter.exe` — 9,329,152 B, sha256 `184D51A7C3CCA7B9D50954B677C5AD8A14472700A651FDF0D270441BE3EAC9A9` |
| Embedded UI | built after final frontend commit; served asset `index-Cbfm1IuM.js`; `/` 200 with `<div id="root">` |
| Deployment package | `dist\HiveMind-v1-win64.zip` — 6,571,928 B, sha256 `BC7A2BAD98FAB1C849E3340D195C26F165581350B630F7BA37D89EE7CB27FEE7` |
| CLI | 	ools\bin\hiverouter-cli.exe — 5,364,224 B, sha256 51E1ABA6DC7349BFB8904014D7007015E13642EA9710F04D380525DCCBD9B9AD |
| Runtime | `hiverouter.exe --bind 127.0.0.1:8080 --data-dir data [--workspace-root <dir>]`; open `http://127.0.0.1:8080/` |
| Toolchain | Rust 1.99.1 `stable-x86_64-pc-windows-gnu`, w64devkit GCC 16.2.0, Node v26.7.0; no Python/Docker/cloud |

## What was built

- **Rust backend** (`backend/crates`): frozen `hiverouter.v1` contract (14 commands, 21 events, 65 refusal codes,
  schema v3), localhost axum WS transport with Origin allowlist + per-session token, petgraph single graph owner,
  prompty prompt/persona assets + typed handshake/context switch, run/task lifecycle + cancellation + caps, SQLite stores,
  snapshots/kill-restore/history API, embedded static UI + `/diagnose`, provider client (DeepSeek contract; wiremock
  refusal matrix; Decimal money; budget prefs), tools/approval nonce-replay/jail/cancel/audit ledger, capability census,
  and the agent-native `hiverouter-cli`.
- **React UI** (`web/`): Vite 8 / React 19 / TS / Tailwind 4 / @xyflow/react 12.12.1; 20 PDF tokens with provenance,
  corrected contrast, 15-state enum, 26 buttons / 6 nav / 4 tabs / HUD / scrubber / library / pinned chat core /
  minimap / refusal region; 28 UI-WIRING-GUIDE surfaces; frozen-command client with ACK-before-mutation and typed
  refusals; wireframe dev mock is dev-only and throws in production.
- **FigCraft design assets** (`docs/ui/figcraft/`): A foundations/brand/components/specimens + B canvas/zoom/interaction/
  hero, 55 asset rows / 28 surfaces, watchdog PASS 20/20 (FAIL 0), canonical `ASSETS-INDEX.json`, episode records.

## Verification (all re-run on the final tree)

| Gate | Result | Evidence |
|---|---|---|
| Backend tests (4 crates) | **233 passed / 0 failed / 1 ignored**, exit 0 | `docs/audit/evidence/final-gate-tests.log` (contract 32, core 83, provider 40, tools 78) |
| Frontend unit tests | 46 passed / 8 files, typecheck + build exit 0 | `web/` (vitest) |
| Live browser e2e (exact shipped binary) | **93 tests = 79 passed / 0 failed / 14 explicit fixme skips**, exit 0 | `e2e/test-results/run-live-FINAL-artifact.log`; HTML `e2e/playwright-report/index.html` |
| Live server surface | `/health` 200, `/diagnose` 200 JSON, `/api/session` 200, `/` 200, assets 200, SPA fallback 200 | curl / auditor evidence |
| Security | foreign Origin 403 `E_ORIGIN_REJECTED`; bad token 401; traversal vectors 404; loopback-only bind refusal | `docs/audit/PHASE-B-VERIFICATION.md` |
| Frozen-command wiring | `SETTINGS_STATUS` ACK+result; `FILE_TREE_LIST`/`FILE_READ` ACK+content in jail; `APPROVAL_RESOLVE` ACK+resolved + replay refusal | `docs/plans/D2-D3-WIRING-EVIDENCE.md`, live WS probes |
| Capability census | registry == allowlist == prompt (7 tools / 65 refusals / 4 tiers / 4 classes), drift test | `/diagnose`, `hiverouter-cli census` |
| Agent-native CLI | JSON stdout, human stderr, exit codes 0/2/3, `--dry-run`, schema, live `health`/`diagnose` | `docs/audit/PHASE-B-VERIFICATION.md` |
| UI watchdog | PASS 20/20, allow_frontend true, tokens exact, blueprint/source unchanged | `ledgers/ui-watchdog.json/.md` |
| Trace ladder | 43 `WIRED-AND-WORKS` hops with captured evidence; 260 UNVERIFIED; 5 MISSING; 0 fabricated | `docs/trace/controls.json`, `docs/trace/live/trace-capture.json` |
| Claim register | 80 rows: 72 VERIFIED / 8 READ / 0 UNVERIFIED / 0 CONTRADICTED (C-026 + C-340 added) | `docs/claims/CLAIM-REGISTER.csv/.json` |
| Deployment | clean-extraction smoke: `--version`, `/health` 200, `/` 200, `/diagnose` 200 | `runtime-test/pkg-final` |

## Not done / deliberately not claimed

- **No paid provider run** was made (no key/budget approval). Therefore node-card/streaming/terminal-run and
  `usage` NULL-vs-zero behavior remain **UNVERIFIED** (wiremock covers provider refusals/cost math only).
- **14 e2e skips are explicit `test.fixme`**, not passes: 12 G-UI DOM gates not implemented as tests (static contrast +
  live colour/zoom/refusal/WS checks cover their intent), 2 WS stubs (synthetic foreign-Origin page; duplicate hello/ACK).
- **Pixel-level UI/PDF fidelity** stays `needs_human` (PDF is a text layer; no board raster exists).
- **Tauri/installer packaging** stays decision-gated; v1 ships as a localhost server + browser, as the blueprint states.
- Tauri/WebView2, cap-std, and other spike-gated candidates remain **not in the lockfile**, per D7.

## Open decisions (visible, not invented)

CF-UI-03 display face (Playfair vs self-hosted Instrument Serif), CF-UI-04 tabs-vs-rail owner rule, CF-UI-05 honey
semantics, S25 code-diff modal (DECISION-GATED; `DIFF_*` not implemented), CF-UI-06 streaming decision, elevation tiers
printed identically (unreconciled). These are recorded in the blueprint/guide and the UI assets, not silently resolved.

## Reproduce

```powershell
cd backend
$env:PATH="$env:USERPROFILE\.cargo\bin;<repo>\tools\w64devkit\bin;"+$env:PATH
cargo test -p hiverouter-contract -p hiverouter-core -p hiverouter-provider -p hiverouter-tools
cargo build --release --locked
cd ..\web; npm.cmd ci; npm.cmd run typecheck; npm.cmd test; npm.cmd run build
# UI is embedded at compile time: rebuild the Rust binary after web changes.
.\target\release\hiverouter.exe --bind 127.0.0.1:8080 --data-dir data --workspace-root <workspace>
```
