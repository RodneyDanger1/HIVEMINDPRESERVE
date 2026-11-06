# GATE CY-2 — recon → plan → lead (core runtime slice)

Scope of this cycle: `backend/crates/hiverouter-core/Cargo.toml` + `src/{config,http,execution,main,checks,ws,
provider_executor}.rs`, read fully and in order. Read-only.

## 1. Recon (skills: forgewright-recon, logic-debugger, integration-logic)
- **Target behavior:** how the run path reaches the provider at HEAD, what the provider gates are, and what happens on
  missing key / cancel / bad plan / budget refusal.
- **Wiring entry point:** `src/http.rs:67–82 fn build_executor` → `ProviderExecutorAdapter::from_env` at `:72`; all
  AppState constructors pass it (`http.rs:101/141/178`); core manifest has the provider path dep at `Cargo.toml:21`.
- **Callers:** WS `SUBMIT_PROMPT` arm (ws.rs) → `RunEngine::submit_prompt` (`execution.rs:300–338`) → `tokio::spawn` →
  `dispatch` → `executor.plan(run.id, planning_prompt, token)` (`execution.rs:480–483`) → plan.validate → leaf
  `executor.execute` (log node) → REPLY_DELTA/REPLY_DONE; settings/file/approval via `ToolsBridge`.
- **Tests + command:** `cargo test -p hiverouter-core` (http_boot 6, ws_transport 13, run_lifecycle 6 by closeout
  evidence); live path test `tests/provider_live.rs` (`#[ignore]`).
- **Data flow (success / failure):** prompt → run row+receipt → Planning → adapter completion (budget preflight inside
  client) → JSON plan → caps → leaves → completed; failure: key absent → `build_executor` falls back to
  `UnconfiguredExecutor` → `E_KEY_MISSING` run failure (never fake success).
- **Findings:** paper's P0 A-F01 is **contradicted at HEAD** (provider dep + adapter wired); template registry is still
  `TemplateRegistry::default()` (`http.rs:102/142/179`); the adapter emits one delta per leaf (non-streaming).
- **Unknowns:** whether the checked-in live evidence matches HEAD's adapter output (READ only).

## 2. Plan (skills: forgewright-plan, capability-auditor)
- **Outcome:** prove wiring at HEAD with line evidence, classify key/cancel/plan-failure refusal paths, list remaining
  defects with `path:line`, and write a per-file migration plan with tests/rollback.
- **Acceptance:** every core-file claim has a line anchor in READLOG; `build_executor` fallback and `E_KEY_MISSING`
  proven; stale P0 corrected; no code change proposed inside this seat.
- **Steps:** verify HEAD/commit message; grep adapter refs; read all core slice files; trace budget/refusal calls;
  compare against paper register; record.
- **Risks:** confusing test doubles with product (run_lifecycle MockExecutor is test-only — allowed); stale docs.

## 3. Lead verdict (skills: forgewright-lead, auditor, truthseeker)
- **Cycle result: PASS.** Core slice verified at HEAD `5c69b66`; A-F01 marked FIXED-at-HEAD with line evidence
  (`Cargo.toml:21`, `http.rs:67–101`); the honest refusal path preserved (`UnconfiguredExecutor` → `E_KEY_MISSING`);
  remaining template-registry gap and non-streaming delta recorded as findings S1-R3/S1-R1.
- **Evidence:** READLOG #8–#15; EVIDENCE E-2/E-3.
