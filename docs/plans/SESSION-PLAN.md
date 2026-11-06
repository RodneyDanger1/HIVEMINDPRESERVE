# SESSION PLAN — HiveMind v1 build (five-field plan)

1. **Task statement.** Build the entire verified HiveMind v1 blueprint from scratch: one Rust binary (axum/tokio/
   reqwest/serde/rusqlite-bundled/tracing/rust-embed) serving a React 19 + TS + Vite 8 + Tailwind 4 + @xyflow/react 12.12.1
   spatial canvas on localhost, one DeepSeek key, no Python/Docker/cloud; Phases 1-12 with gates; FigCraft design assets
   produced by FigCraft subagents and audited by a UI watchdog; final trace/capability/agent-native/claim verification.
2. **Target language/framework + version source.** Rust stable, MSRV 1.89.0 (blueprint/FACT-REGISTER, pinned); Node >=22.12.0
   (FACT-REGISTER F5); exact crates/npm pins in blueprint §3 / EXTERNAL-DEPENDENCIES. `version_pinned: true`;
   `version_source`: FACT-REGISTER (derived 2026-11-06 from primary registries; see FACT-REGISTER rows).
3. **Known unknowns.** (a) GNU toolchain + w64devkit build success on this host (installing); (b) disk headroom (6.05 GB) vs
   cargo target + node_modules; (c) FigCraft 597-command API details (agents must read schemas, not guess); (d) live provider
   not exercised (no key/budget approval) — wiremock/mock only; (e) Playwright browser binaries not downloaded (use installed
   Chrome channel); (f) FigCraft GUI attach not needed (headless CLI path).
4. **Required documentation.** blueprint phases 1-12, UI-WIRING-GUIDE, FACT-REGISTER, EXTERNAL-DEPENDENCIES, PHASE-MAP,
   CHANGE-LEDGER, CANONICAL register, UI-REQUIREMENTS, PRUNED-PLAN-CLAIMS, PDF pages.txt; skills matrix (§2 of
   EXECUTION-PROMPT).
5. **Verification gates / rollback.** Phase preflights + acceptance commands; `cargo test`, `npm test`, Playwright e2e,
   trace 28 controls, capability census, agent-native CLI review, claim register, FigCraft CLI `info`/`export` artifact
   checks + UI watchdog ledger; `cargo build --release --locked` + localhost boot smoke. Rollback: project dir is new and
   git-initialized; phase commits are revertible; blueprint directory is read-only and never modified.

## Team / budgets

- Active children cap 7; planned: B-Core, FigCraft-A, FigCraft-B, UI-Watchdog (wave 1); then B-Provider, Frontend,
  Verifier (wave 2) as dependencies unlock. No child spawns children (maxDepth 1).
- Model-request concurrency: `N/A — no ceiling exposed by any profile row` (per operating contract; report as measured).
- Background jobs: rustup install + w64devkit download (2); tracked with job ids; killed if stalled.

## Phase order

1 → 2 → 3 → 4 → 9 → 5 → 6 → 11 → 12 for backend (B-Core owns 1-4,9; B-Provider owns 5,6,11,12), with 7/8/10 UI in
parallel after contract freeze (FigCraft assets → frontend). Strict gate: Phase 1 contract freeze before FigCraft spawn
(this file and `docs/contracts/UI-COMMANDS-v1.md` are the freeze).
