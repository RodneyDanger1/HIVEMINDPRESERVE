# 05 — E2E and Tooling

## Playwright

- `e2e/playwright.config.ts` — projects (chrome/msedge), web server, base URL, trace/screenshot policy.
- `e2e/run-guarded.mjs` — grouped gate runner; live/paid specs skip by name unless gates set.
- `e2e/helpers/live-server.ts` — starts the real binary, temp workspace, token/origin options, `paidGate('prompt'|'tool-turn')`, frames/DOM/evidence capture.
- `e2e/helpers/replay-harness.ts`, `ws-probe.ts`, `mock-*` — mocked transport, frame capture, deterministic replay.
- Specs: tool-loop, approval-flow, control-matrix, gui-gates/residual, a11y, trace-hops, ws-contract, replay-state, live-app/streaming/replay/security/resync/tool-turn/trace-capture.
- Skips are explicit named skips for live/paid/axe, not silent passes.

## QA/release scripts (`tools/`)

- `tools/qa/run-gates.ps1` — orchestrated pre-release/release gates; pre-release exits 3, sealed manifest acceptance via checker.
- `tools/build-release.ps1` — source-only/sealed build, manifest v2, hashes, tag topology, `-RecordSeal`.
- `tools/qa/check-seal-topology.mjs` — source-C/seal-D allowlist, tag/diff/artifact verification.
- `tools/qa/verify-harness.mjs` — 28 structural checks incl. evidence-pointer resolution.
- `tools/qa/apply-live-trace.mjs` — capture validation (`E_CAPTURE_EMPTY`, evidence pointers, stale capture).
- `tools/qa/check-docs-truth.mjs` — docs version/claims checks.
- `tools/qa/*tests*` — node:test suites for seal/trace tools.
- `tools/build_assets_index.py`, `figcraft-assets.mjs`, `build-trace-seeds.mjs` — asset indexes and trace seeds.
- `tools/bin/` — tracked built `hiverouter.exe` / `hiverouter-cli.exe`; `tools/cc-test/` C link fixtures.
- `tools/w64devkit/` is ignored vendored MinGW (9,452 files); build scripts require it on PATH on this host.

## Test counts

Backend 475 passed / 0 failed / 2 ignored; web 179/179; guarded E2E 106 passed / 0 failed / 166 named skips; docs 19/19; harness 28/28. See `10-VERIFICATION.md`.
