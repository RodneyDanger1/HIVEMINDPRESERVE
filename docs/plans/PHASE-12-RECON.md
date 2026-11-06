# PHASE 12 RECON — Packaging, Release & Runbook

Written before Phase 12 implementation edits (2026-11-06 03:41 CST). Blueprint card: Phase 12 (WPs 12.1–12.4).

## Recon
- **Target behavior:** there is no CLI and no release/runbook artifact yet. This phase delivers the
  agent-native `hiverouter` CLI (JSON on stdout, human text on stderr, stable exit codes, `--dry-run`,
  `schema` introspection, safety tiers, generated docs), the release build `cargo build --release --locked`,
  the Windows runbook with first-run dependency checks (Node ≥22.12.0, MSRV floor 1.89.0, provider key
  presence; named refusals `E_NODE_MISSING` / `E_TOOLCHAIN_MSRV`), `notify 8.2.1`-based workspace watching
  with 150 ms coalesce (watcher module lives with the tools/phase-12 code), and the truthful packaging
  statement (localhost server + browser; NO installer/desktop-shell claim).
- **Entry point:** `backend/crates/hiverouter-tools/src/bin/hiverouter.rs` (`[[bin]] name = "hiverouter"` in
  the tools manifest, so `cargo build --release --locked` from `backend/` produces it), with
  `tools/{runbook.md,test.ps1,build-release.ps1}`.
- **Callers:** a human operator or an agent at a terminal. The CLI must not require a running server for
  discovery/schema/preflight/cost/health commands; `hiverouter health` talks to the local server URL.
- **Tests + command:** `cargo test -p hiverouter-tools --bin hiverouter` and CLI smoke assertions in
  `tools/test.ps1`; release gate `cargo build --release --locked` + SHA-256 recorded in the evidence file.
- **Data flow (success + one failure path):** `hiverouter <resource> <action> [--json|--dry-run]` →
  boundary validation (typed flags, no free-form) → command executes against local state (tool registry,
  provider pricing/budget math, preflight env probe) → stdout JSON envelope
  `{"schema":"hiverouter.cli.v1","ok":true,"command":...,"data":...}`; human diagnostics on stderr; exit code
  0 ok / 2 usage / 3 refusal / 4 internal. Mutating commands (future) refuse without `--yes` and print the
  dry-run plan on `--dry-run`. Failure path: missing Node → exit 3 with `code:"E_NODE_MISSING"` +
  `how_to_fix`, no crash; missing key → `E_KEY_MISSING` (never prints the key); unknown command → exit 2
  `E_COMMAND_UNKNOWN`; MSRV below 1.89.0 → exit 3 `E_TOOLCHAIN_MSRV`.
- **Unknowns / evidence still missing:**
  - The v1 artifact is a localhost server + browser: no installer/desktop shell is claimed (blueprint
    CF-04/D3); Tauri/WebView2 remains optional/UNVERIFIED and is not built.
  - `notify 8.2.1` watcher debounce test (5 writes in 50 ms → 1 event) runs on this host; if the Windows
    backend cannot coalesce deterministically the observed behaviour is recorded, never faked.
  - Node/npm: `npm` via PowerShell script is blocked by execution policy on this host (`npm.ps1`), so
    node/npm probes use `npm.cmd`; the first-run check tests `node --version` and the MSRV via `rustc
    --version` only, and never shells out to npm.
- **Files I expect to touch:** `backend/crates/hiverouter-tools/{Cargo.toml,src/bin/hiverouter.rs,
  src/watcher.rs,tests/cli_*.rs}`, `tools/{runbook.md,build-release.ps1,test.ps1}`,
  `docs/plans/PHASE-12-{RECON,EVIDENCE}.md`, `docs/claims/*`. Never core/contract/web/blueprint.

## Scope check against the blueprint card
- 12.1 CLI trimmed to local-only commands (no daemon plugin/OS daemon); 12.2 `cargo build --release`;
  12.3 Windows install/runbook + first-run checks; 12.4 `notify 8.2.1` 150 ms coalesce watcher.
- Exit-gate mapping: release artifact named with path+version+hash; runbook dry-run (binary starts, localhost
  health 200); no installer claim; first-run dependency checks refuse by name; watcher debounce test.
