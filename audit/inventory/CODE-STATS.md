# CODE STATISTICS

- Tracked files: **507**
- First-party code files: **188**
- Total source LOC: **36,802**
- Total tracked LOC (all text incl. docs/data): **106,840**
- Total tracked bytes: **63.98 MiB**

## By directory

| Directory | Files | Source files | Source LOC | Total LOC |
|---|---:|---:|---:|---:|
| `backend` | 87 | 77 | 21,665 | 27,585 |
| `web` | 55 | 52 | 6,583 | 9,040 |
| `docs` | 236 | 30 | 6,469 | 66,919 |
| `e2e` | 24 | 18 | 1,458 | 1,704 |
| `tools` | 16 | 10 | 626 | 723 |
| `runtime-test` | 82 | 1 | 1 | 435 |
| `.cargo` | 1 | 0 | 0 | 13 |
| `(root)` | 4 | 0 | 0 | 46 |
| `ledgers` | 2 | 0 | 0 | 375 |

## By language

| Language | Files | LOC |
|---|---:|---:|
| JSON | 88 | 60,049 |
| Rust | 79 | 21,669 |
| Python | 27 | 6,293 |
| TypeScript | 40 | 5,870 |
| Markdown | 57 | 4,193 |
| Lockfile | 1 | 4,047 |
| TypeScript/TSX | 21 | 1,708 |
| Text | 19 | 1,155 |
| JavaScript | 12 | 798 |
| Mermaid | 5 | 277 |
| CSS | 2 | 243 |
| TOML | 7 | 214 |
| PowerShell | 3 | 198 |
| CSV | 1 | 82 |
| Prompty | 1 | 21 |
| HTML | 2 | 17 |
| C | 2 | 6 |
| unknown | 2 | 0 |
| Log | 47 | 0 |
| PNG | 12 | 0 |
| Zip | 1 | 0 |
| PE-binary | 10 | 0 |
| SVG | 5 | 0 |
| FigCraft | 18 | 0 |
| Python-bytecode | 6 | 0 |
| SQLite | 13 | 0 |
| SQLite-SHM | 13 | 0 |
| SQLite-WAL | 13 | 0 |

## Largest first-party source files (LOC)

| File | LOC | Bytes |
|---|---:|---:|
| `backend/crates/hiverouter-tools/src/cli.rs` | 1,023 | 37,923 |
| `backend/crates/hiverouter-tools/src/approval.rs` | 1,011 | 34,944 |
| `web/src/api/contract.ts` | 862 | 28,868 |
| `docs/ui/figcraft/A/build/pages_components.py` | 799 | 43,816 |
| `backend/crates/hiverouter-provider/src/client.rs` | 753 | 27,461 |
| `backend/crates/hiverouter-provider/src/dto.rs` | 743 | 24,305 |
| `web/src/state/store.ts` | 741 | 23,407 |
| `backend/crates/hiverouter-provider/src/budget.rs` | 711 | 23,965 |
| `backend/crates/hiverouter-core/src/execution.rs` | 612 | 20,243 |
| `backend/crates/hiverouter-tools/src/tool.rs` | 610 | 19,612 |
| `backend/crates/hiverouter-contract/src/command.rs` | 570 | 21,334 |
| `backend/crates/hiverouter-core/src/ws.rs` | 537 | 19,462 |
| `backend/crates/hiverouter-contract/src/event.rs` | 516 | 18,156 |
| `web/src/api/client.ts` | 498 | 15,930 |
| `backend/crates/hiverouter-core/tests/ws_transport.rs` | 484 | 19,177 |
| `backend/crates/hiverouter-provider/src/accounting.rs` | 470 | 17,056 |
| `backend/crates/hiverouter-tools/src/shell.rs` | 464 | 16,201 |
| `backend/crates/hiverouter-core/src/store.rs` | 455 | 15,733 |
| `backend/crates/hiverouter-tools/src/health.rs` | 453 | 15,752 |
| `backend/crates/hiverouter-core/src/db.rs` | 450 | 15,761 |
| `backend/crates/hiverouter-tools/src/jail.rs` | 442 | 16,815 |
| `docs/ui/figcraft/B/build/boards_common.py` | 438 | 19,072 |
| `docs/ui/figcraft/A/build/gen_common.py` | 436 | 21,202 |
| `backend/crates/hiverouter-core/src/http.rs` | 429 | 14,376 |
| `docs/ui/figcraft/A/build/gen_manifest.py` | 410 | 17,760 |

## Test-related files

- `backend/crates/hiverouter-contract/tests/common/mod.rs`
- `backend/crates/hiverouter-contract/tests/golden_schemas.rs`
- `backend/crates/hiverouter-contract/tests/schema_roundtrip.rs`
- `backend/crates/hiverouter-contract/tests/ui_command_matrix.rs`
- `backend/crates/hiverouter-core/tests/chat_store.rs`
- `backend/crates/hiverouter-core/tests/context_switch.rs`
- `backend/crates/hiverouter-core/tests/graph_authority.rs`
- `backend/crates/hiverouter-core/tests/handshake.rs`
- `backend/crates/hiverouter-core/tests/http_boot.rs`
- `backend/crates/hiverouter-core/tests/kill_restore.rs`
- `backend/crates/hiverouter-core/tests/prompt_assets.rs`
- `backend/crates/hiverouter-core/tests/run_lifecycle.rs`
- `backend/crates/hiverouter-core/tests/snapshot_history.rs`
- `backend/crates/hiverouter-core/tests/support/mod.rs`
- `backend/crates/hiverouter-core/tests/ws_transport.rs`
- `backend/crates/hiverouter-provider/tests/refusal_matrix.rs`
- `backend/crates/hiverouter-tools/tests/audit_jsonl.rs`
- `e2e/.gitignore`
- `e2e/README.md`
- `e2e/data-probe.mjs`
- `e2e/data/control-matrix.json`
- `e2e/helpers/app.ts`
- `e2e/helpers/contrast.ts`
- `e2e/helpers/evidence.ts`
- `e2e/helpers/matrix.ts`
- `e2e/helpers/trace.ts`
- `e2e/helpers/ws-tap.ts`
- `e2e/live-check2.mjs`
- `e2e/live-inspect.mjs`
- `e2e/live-probe.mjs`
- `e2e/live-shot.mjs`
- `e2e/live-smoke.mjs`
- `e2e/package-lock.json`
- `e2e/package.json`
- `e2e/playwright.config.ts`
- `e2e/specs/a11y-gates.spec.ts`
- `e2e/specs/control-matrix.spec.ts`
- `e2e/specs/live-app.spec.ts`
- `e2e/specs/trace-hops.spec.ts`
- `e2e/specs/ws-contract.spec.ts`
- `e2e/tsconfig.json`
- `web/tests/agentState.test.ts`
- `web/tests/apiClient.test.ts`
- `web/tests/contract.test.ts`
- `web/tests/keyboard.test.ts`
- `web/tests/metrics.test.ts`
- `web/tests/noHardcoded.test.ts`
- `web/tests/store.test.ts`
- `web/tests/tokens.test.ts`
