# DIRECTORY / CATEGORY CENSUS

Tracked files: **507**

## By top-level directory

| Directory | Files |
|---|---:|
| `docs` | 236 |
| `backend` | 87 |
| `runtime-test` | 82 |
| `web` | 55 |
| `e2e` | 24 |
| `tools` | 16 |
| `(root)` | 4 |
| `ledgers` | 2 |
| `.cargo` | 1 |

## By category

| Category | Files |
|---|---:|
| documentation | 202 |
| source | 158 |
| runtime-artifact | 82 |
| documentation-asset | 36 |
| config-manifest | 16 |
| other | 9 |
| generated-artifact | 4 |

## By language

| Language | Files |
|---|---:|
| JSON | 88 |
| Rust | 79 |
| Markdown | 57 |
| Log | 47 |
| TypeScript | 40 |
| Python | 27 |
| TypeScript/TSX | 21 |
| Text | 19 |
| FigCraft | 18 |
| SQLite | 13 |
| SQLite-SHM | 13 |
| SQLite-WAL | 13 |
| JavaScript | 12 |
| PNG | 12 |
| PE-binary | 10 |
| TOML | 7 |
| Python-bytecode | 6 |
| Mermaid | 5 |
| SVG | 5 |
| PowerShell | 3 |
| unknown | 2 |
| C | 2 |
| HTML | 2 |
| CSS | 2 |
| Lockfile | 1 |
| Prompty | 1 |
| Zip | 1 |
| CSV | 1 |

## By audit seat

| Seat | Teammate | Files |
|---|---|---:|
| b-core | 54 |
| b-provider | 33 |
| figcraft-a | 103 |
| figcraft-b | 91 |
| ui-watchdog | 49 |
| watchdog-1 | 55 |
| watchdog-2 | 122 |

## Deep-analysis set (8-12 paragraph target)

- `backend/Cargo.lock`
- `backend/Cargo.toml`
- `backend/crates/hiverouter-contract/src/command.rs`
- `backend/crates/hiverouter-contract/src/envelope.rs`
- `backend/crates/hiverouter-contract/src/event.rs`
- `backend/crates/hiverouter-contract/src/lib.rs`
- `backend/crates/hiverouter-contract/src/refusal.rs`
- `backend/crates/hiverouter-contract/src/schema.rs`
- `backend/crates/hiverouter-contract/src/state.rs`
- `backend/crates/hiverouter-core/src/bus.rs`
- `backend/crates/hiverouter-core/src/context.rs`
- `backend/crates/hiverouter-core/src/db.rs`
- `backend/crates/hiverouter-core/src/execution.rs`
- `backend/crates/hiverouter-core/src/graph.rs`
- `backend/crates/hiverouter-core/src/handshake.rs`
- `backend/crates/hiverouter-core/src/http.rs`
- `backend/crates/hiverouter-core/src/main.rs`
- `backend/crates/hiverouter-core/src/prompts.rs`
- `backend/crates/hiverouter-core/src/recovery.rs`
- `backend/crates/hiverouter-core/src/snapshot.rs`
- `backend/crates/hiverouter-core/src/static_ui.rs`
- `backend/crates/hiverouter-core/src/tools_bridge.rs`
- `backend/crates/hiverouter-core/src/ws.rs`
- `backend/crates/hiverouter-provider/src/accounting.rs`
- `backend/crates/hiverouter-provider/src/budget.rs`
- `backend/crates/hiverouter-provider/src/client.rs`
- `backend/crates/hiverouter-provider/src/dto.rs`
- `backend/crates/hiverouter-provider/src/pricing.rs`
- `backend/crates/hiverouter-tools/src/approval.rs`
- `backend/crates/hiverouter-tools/src/census.rs`
- `backend/crates/hiverouter-tools/src/cli.rs`
- `backend/crates/hiverouter-tools/src/health.rs`
- `backend/crates/hiverouter-tools/src/jail.rs`
- `backend/crates/hiverouter-tools/src/ledger.rs`
- `backend/crates/hiverouter-tools/src/shell.rs`
- `backend/crates/hiverouter-tools/src/tool.rs`
- `e2e/playwright.config.ts`
- `e2e/specs/live-app.spec.ts`
- `e2e/specs/trace-hops.spec.ts`
- `tools/qa/verify-harness.mjs`
- `web/package.json`
- `web/src/App.tsx`
- `web/src/a11y/keyboard.ts`
- `web/src/api/client.ts`
- `web/src/api/contract.ts`
- `web/src/chat/ChatCore.tsx`
- `web/src/flow/Canvas.tsx`
- `web/src/flow/nodes.tsx`
- `web/src/panels/DataPanel.tsx`
- `web/src/panels/SettingsPanel.tsx`
- `web/src/state/agentState.ts`
- `web/src/state/store.ts`
- `web/src/state/strings.ts`
- `web/src/tokens/contrast.ts`
- `web/src/tokens/tokens.ts`
- `web/vite.config.ts`

## Generated/ignored classes (counted, not per-file prose)

| Tree | Files | MB | Class |
|---|---:|---:|---|
| `target/` | 13,525 | 8,087.2 | Rust build artifacts |
| `web/node_modules/` | 3,976 | 110.2 | npm vendor |
| `e2e/node_modules/` | 194 | 18.2 | npm vendor |
| `tools/w64devkit/` | 9,452 | 716.7 | third-party toolchain |
| `dist/` | 6 | 20.3 | packaged release output |
| `runtime-test/` | 82 (tracked) | 33.6 | runtime/test DB + logs (tracked by mistake: finding) |
| `.git/` | not counted | - | VCS internals, excluded by convention |
