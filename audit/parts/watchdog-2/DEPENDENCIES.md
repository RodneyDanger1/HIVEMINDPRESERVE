# DEPENDENCIES — watchdog-2 partition

Read-only static import scan (regex on `from`/`require`) + manifests; no execution/build.

## Internal import graph (partition files)

| from | import | resolves to |
|---|---|---|
| `e2e/README.md` | `./node_modules/@playwright/test/package.json` | **unresolved/cross-partition** `e2e/node_modules/@playwright/test/package.json` |
| `e2e/specs/a11y-gates.spec.ts` | `../helpers/contrast` | `e2e/helpers/contrast.ts` |
| `e2e/specs/a11y-gates.spec.ts` | `../helpers/matrix` | `e2e/helpers/matrix.ts` |
| `e2e/specs/control-matrix.spec.ts` | `../helpers/matrix` | `e2e/helpers/matrix.ts` |
| `e2e/specs/live-app.spec.ts` | `../helpers/matrix` | `e2e/helpers/matrix.ts` |
| `e2e/specs/live-app.spec.ts` | `../helpers/evidence` | `e2e/helpers/evidence.ts` |
| `e2e/specs/live-app.spec.ts` | `../helpers/contrast` | `e2e/helpers/contrast.ts` |
| `e2e/specs/trace-hops.spec.ts` | `../helpers/matrix` | `e2e/helpers/matrix.ts` |
| `e2e/specs/trace-hops.spec.ts` | `../helpers/evidence` | `e2e/helpers/evidence.ts` |
| `e2e/specs/ws-contract.spec.ts` | `../helpers/app` | `e2e/helpers/app.ts` |
| `e2e/specs/ws-contract.spec.ts` | `../helpers/matrix` | `e2e/helpers/matrix.ts` |
| `e2e/specs/ws-contract.spec.ts` | `../helpers/ws-tap` | `e2e/helpers/ws-tap.ts` |

## External imports detected

- `@playwright/test`
- `node:crypto`
- `node:fs`
- `node:path`
- `node:url`

## Manifests / runtime dependencies

- `e2e/package.json` + `e2e/package-lock.json`: devDependency `@playwright/test` 1.65.0 (installed under `e2e/node_modules`, ignored class); scripts call `playwright test`, `node ../tools/qa/*.mjs`.
- Installed browsers (not npm): Chrome 156.0.8078.75 / Edge 156.0.4314.58 via Playwright `channel`; no browser download (config comments; `playwright.config.ts:2-8`).
- Server under test: `tools/bin/hiverouter.exe` (tracked artifact) served at `http://127.0.0.1:8799`; tests depend on external process start (no `webServer`; `playwright.config.ts:10-14`).
- Runtime artifacts: `runtime-test/data/hiverouter.db` (+ `-shm/-wal`) written by the server; `runtime-test/workspace/{note.txt,README.md}` read via FILE_TREE_LIST/FILE_READ (live-app tests 14/15 assert them).
- `tools/qa/*.mjs` read tracked docs/contracts/e2e data: `docs/contracts/UI-COMMANDS-v1.md`, `e2e/data/control-matrix.json`, `docs/trace/*.json`, `docs/ui/figcraft/**`; `apply-live-trace.mjs` writes `docs/trace/controls.json` (a cross-partition write tool — only run by QA).

## Cross-partition links

- e2e specs assert DOM/contract implemented in `web/src/**` (watchdog-1) and server contract behaviour in `backend/**` (b-core/b-provider); no code imports cross the boundary — coupling is via frozen `docs/contracts/UI-COMMANDS-v1.md` and rendered ARIA/role names.
- `docs/trace/**` (ui-watchdog partition) is read/written by trace tooling; `audit/parts/watchdog-2` is the only audit write location.
- `runtime-test/data/hiverouter.db` was produced by `tools/bin/hiverouter.exe` against `backend` schema; provenance is runtime, not source.
