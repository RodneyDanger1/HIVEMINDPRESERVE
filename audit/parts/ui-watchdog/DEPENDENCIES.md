# DEPENDENCIES.md — ui-watchdog seat

**Generated:** 2026-11-06 · read-only extraction from `backend/Cargo.lock` (405 packages), `backend/Cargo.toml` + 4 crate manifests, `web/package.json` (7 deps + 8 devDeps), `web/package-lock.json` v3 (147 packages, 139 top-level installed).

## 1. Exact dependency sets (evidence)

- **Cargo:** workspace pins direct deps with `==`; lockfile has **405 packages** (incl. 4 workspace crates `hiverouter-*@0.1.0`). Vendored matrix includes axum 0.8.9, tokio 1.54.1, rusqlite 0.41.1 (`bundled` → `libsqlite3-sys 0.39.1`), reqwest 0.13.5 (rustls 0.23.46, hyper 1.12.0), petgraph 0.8.3, rust_decimal 1.43.0, uuid 1.29.0, jsonschema 0.60.2, wiremock 0.6.5, tokio-tungstenite 0.29.0/tungstenite 0.29.0, rust-embed 8.13.0, tower-http 0.6.11 *and* 0.7.1, aws-lc-rs 1.19.0 / aws-lc-sys 0.47.0 (rustls crypto backend), ring 0.17.14.
- **npm:** runtime deps exactly 7 (`@dagrejs/dagre`, `@xyflow/react`, `motion`, `partysocket`, `react`, `react-dom`, `zustand`); devDeps exactly 8 (`@tailwindcss/vite`, `@types/react`, `@types/react-dom`, `@vitejs/plugin-react`, `tailwindcss`, `typescript`, `vite`, `vitest`). lockfile v3 → 147 package entries, 139 top-level installed (platform-specific rolldown/lightningcss/typescript binaries dominate).
- **Toolchain/external binaries:** Rust `stable-x86_64-pc-windows-gnu` (rust-toolchain.toml), w64devkit gcc/ar (`.cargo/config.toml` + `tools/w64devkit`), Node >=22.12.0 (README), Playwright 1.65.0 (e2e partition), Mermaid CLI mmdc (diagram renders).

## 2. Direct crates (workspace pins → role)

| crate | pin | role in product |
|---|---|---|
| `tokio` | `=1.54.1` | runtime (rt-multi-thread, macros, net, sync, time, signal, fs, process, io-util) |
| `axum` | `=0.8.9` | HTTP + ws routes |
| `reqwest` | `=0.13.5` | provider HTTP client (json, rustls, http2; default-features=false) |
| `serde` | `=1.0.229` | (de)serialisation derive |
| `serde_json` | `=1.0.145` | envelopes/JSON (preserve_order) |
| `rusqlite` | `=0.41.1` | SQLite (bundled) |
| `tracing-subscriber` | `=0.3.23` | structured logging (env-filter, json) |
| `rust-embed` | `=8.13.0` | embed web assets in binary |
| `tokio-util` | `=0.7.20` | CancellationToken/util |
| `tower-http` | `=0.7.1` | trace/cors/limit/request-id/fs middleware |
| `jsonschema` | `=0.60.2` | schema validation |
| `zeroize` | `=1.9.1` | secret zeroisation |
| `strum` | `=0.28.0` | enum mapping |
| `rust_decimal` | `=1.43.0` | Decimal money/token accounting |
| `uuid` | `=1.29.0` | v7 correlation/run ids |
| `schemars` | `=1.2.3` | JSON-schema derivation |
| `thiserror` | `=2.0.21` | typed errors |
| `petgraph` | `=0.8.3` | graph authority / cycle refusal |
| `prompty` | `=2.1.0` | prompt templates |
| `async-trait` | `=0.1.89` | async traits |
| `tempfile` | `=3.23.0` | tests |
| `wiremock` | `=0.6.5` | offline provider refusal matrix (dev/test) |
| `notify` | `8.2.1` | file watching (per workspace lock) |

## 3. Direct npm packages (package.json → role)

| package | version | role |
|---|---|---|
| `@xyflow/react` | `12.12.1` | canvas/nodes/minimap (MIT) |
| `react / react-dom` | `19.3.1` | UI runtime (MIT) |
| `zustand` | `5.0.16` | client store (MIT) |
| `motion` | `14.9.1` | animations (MIT; reduced-motion rules) |
| `partysocket` | `1.3.0` | WS reconnect helper (MIT) |
| `@dagrejs/dagre` | `3.1.3` | layered graph layout (MIT) |
| `vite` | `8.3.6` | build/dev server (MIT) |
| `tailwindcss / @tailwindcss/vite` | `4.3.4` | styling (MIT) |
| `typescript` | `7.0.2` | type checker (Apache-2.0) |
| `vitest` | `5.1.0` | unit tests (MIT) |

## 4. Local import/call graph for this partition

Assigned files are **docs, diagrams and root configs — no code imports**. Their dependency edges are references, not imports:

- `.cargo/config.toml` → `tools/w64devkit/bin/{gcc,ar}.exe` (toolchain) and `[build] target-dir=target`.
- `rust-toolchain.toml` → host rustup channel `stable-x86_64-pc-windows-gnu` + rustfmt/clippy.
- `README.md` → `backend/`, `web/`, `tools/`, `docs/`, `ledgers/` layout; toolchain/build commands; shipped hashes.
- `CHANGE-LEDGER-IMPL.md` → `docs/contracts/UI-COMMANDS-v1.md` (CL-I-001 names added), `backend` contract crate tests, `docs/ui/figcraft/**`, `web/**`, `docs/claims/**`.
- `docs/diagrams/*.mmd` → blueprint phases (Phase 4/5/6/9/10), `blueprint/UI-WIRING-GUIDE.md`, `docs/contracts/UI-COMMANDS-v1.md`; renders consumed by Lead paper integration.
- `docs/trace/TRACE-SCHEMA.md` → `e2e/data/control-matrix.json`, `tools/qa/build-trace-seeds.mjs`, `tools/qa/verify-harness.mjs`, `docs/trace/{controls,traces,events,terminal-truth}`, `docs/contracts/UI-COMMANDS-v1.md`.
- `docs/trace/QA-HARNESS.md` → `backend/crates/hiverouter-core/src/{main,http,config}.rs`, `web/{package.json,vite.config.ts}`, `web/src/App.tsx`, `web/src/styles/tokens.css`, `e2e/**`, `tools/qa/**`, `tools/bin/hiverouter.exe`.
- `docs/trace/A11Y-CONTRAST-PLAN.md` → `reports/ui/UI-REQUIREMENTS.md`, `blueprint/...`, `e2e/helpers/contrast.ts`, `e2e/specs/a11y-gates.spec.ts`, `web/src/styles/tokens.css`.
- `docs/trace/live/README.md` + `live-app/*.json` → `e2e/` run log/report, `tools/bin/hiverouter.exe` (sha 184D51A7…C9A9, asset index-Cbfm1IuM.js, PID 33580), `web/src/panels/DataPanel.tsx:44`.
- `docs/trace/live/inspect.json` → base `http://127.0.0.1:8799`, `/ws` session capture.
- Root docs reference cross-partition owners: b-core/b-provider (backend), watchdog-1 (web), watchdog-2 (e2e/tools).

## 5. Cross-partition links (for Lead integration)

| this partition | links to | evidence |
|---|---|---|
| trace docs | e2e harness + tools/qa | TRACE-SCHEMA §1, QA-HARNESS §4 |
| trace docs | backend routes/core | QA-HARNESS §3 (`GET /health`, `/api/session`, `/ws`) |
| trace docs | web app + tokens | A11Y-CONTRAST-PLAN §1/§3, `web/src/styles/tokens.css` |
| diagrams | blueprint + contracts | header comments in each `.mmd` |
| live evidence | shipped binary/dist | README status hashes; live/README base URL + PID |
