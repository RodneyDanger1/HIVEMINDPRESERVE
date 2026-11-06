# W5 — PROMPT ASSET WIRING (plan; ForgeWright greenfield/repair protocol)

**Seat:** 6 (prompt-asset wiring) · **HEAD at start:** `c9390f5` (1.1 wave 3) · **Write scope:** `backend/crates/hiverouter-core/src/prompts.rs`, `backend/crates/hiverouter-core/src/http.rs`, new tests under `backend/crates/hiverouter-core/tests/` only · **No commit.**

## Phase 1 — logic-gatekeeper (written before any mutation)

```json
{
  "phase1": {
    "architecture": "Production server state is assembled in backend/crates/hiverouter-core/src/http.rs (AppState::new / AppState::with_graph for the binary; AppState::with_services for tests) and passes Arc<TemplateRegistry> into RunEngine (execution.rs:215 pub templates). TemplateRegistry lives in src/prompts.rs and loads a manifest directory; #[derive(Default)] on TemplateRegistry yields an EMPTY map. The prompt assets exist only at crates/hiverouter-core/assets/prompts/{manifest.json,planner.prompty}; the release ships one binary and already embeds web/dist with rust-embed (src/static_ui.rs:14), so a runtime data_dir asset path is not shipped.",
    "dataFlow": "Compile-time include_str! bytes (manifest.json + planner.prompty) -> serde_json manifest parse -> per-row licence/source/fetched_at gate -> raw file string ref gate (${env:}: ${file:}) -> prompty 2.1.0 load_from_string -> BTreeMap<template_id, LoadedTemplate> -> Arc<TemplateRegistry> -> RunEngine.templates -> planner render at SUBMIT_PROMPT time. Frame: asset bytes are repo-controlled but can be malformed (the failure the wiring must surface), and the manifest row is the licence/source authority.",
    "edgeCases": [
      "manifest.json is malformed JSON -> must be E_TEMPLATE_INVALID, never an empty registry",
      "manifest row names a file absent from the embedded set -> must be E_TEMPLATE_UNKNOWN",
      "planner.prompty has malformed front matter / no role body -> must be E_TEMPLATE_INVALID",
      "manifest row licence is not on ALLOWED_LICENCES (or source_url/fetched_at empty) -> E_TEMPLATE_LICENSE_MISSING",
      "template body contains ${env:...} or ${file:...} -> E_TEMPLATE_REF_FORBIDDEN",
      "the test constructor with_services keeps the documented Default (empty) registry; that is a test seam, not a production path"
    ],
    "rootCause": "All three AppState constructors pass TemplateRegistry::default() — an empty map — into RunEngine, so the production boot path silently runs without the planner template that exists in the repository, and planner renders fail only later as E_TEMPLATE_UNKNOWN instead of the boot path proving the assets loaded."
  }
}
```

### Phase 2 — where the checks go
- `TemplateRegistry::load_embedded()` parses embedded bytes through the same policy as `load(directory)`; every failure maps to an existing frozen refusal (`E_TEMPLATE_INVALID`, `E_TEMPLATE_UNKNOWN`, `E_TEMPLATE_LICENSE_MISSING`, `E_TEMPLATE_REF_FORBIDDEN`) — no new contract code.
- `AppState::new` / `AppState::with_graph` call the embedded loader once and pass the loaded registry to `RunEngine`. A load failure is recorded as a **required failed Check** (`prompt_assets`, `code = refusal.code()`), logged at error, stored in `AppState.prompt_assets_error`, and reported by `/health` (`prompt_assets: {loaded:false, error_code}`) → `/health` answers 503 (required check failed). Nothing silently succeeds.
- `AppState::with_services` keeps `TemplateRegistry::default()` and is documented as the test-only seam.
- Callers unchanged: `main.rs:123 AppState::with_graph(...)` keeps its signature (main.rs is outside this seat's write scope); a truly fatal boot path needs a follow-up `try_with_graph` wiring in main.rs (recorded as Not done).

## Recon (forgewright-recon)
- Target behaviour: the production server constructs `RunEngine` with the real planner template from `assets/prompts`, refusing by name when the assets are missing/invalid.
- Entry points: `src/prompts.rs:108 TemplateRegistry::load`, `src/http.rs:102,142,179 TemplateRegistry::default()` call sites, `src/execution.rs:215 pub templates`.
- Assets: `backend/crates/hiverouter-core/assets/prompts/manifest.json` (1 row: planner / MIT / authored in repo / 2026-11-06), `planner.prompty` (front matter `name: planner`, inputs `task`, body with `system:` + `user:` markers), `personas.json` (test-only persona view).
- Callers: `AppState::new` (http_boot tests + production API), `with_graph` (**production boot: main.rs:123**), `with_services` (tests/support/mod.rs:65); direct `TemplateRegistry::load(&assets())` in `tests/prompt_assets.rs:28,72`.
- Tests + commands: `cargo test -p hiverouter-core --test prompt_assets`; new `--test prompt_assets_wiring`; workspace regression `cargo test -p hiverouter-core --tests`.
- Data flow (happy): embedded manifest → licence gate → planner.prompty → registry → engine → `render("planner", {task})` = system+user messages.
- Failure path: invalid manifest/planner → named Refusal at construction → required failed Check + `/health` 503 + `prompt_assets_error`; planner render never pretends to work.
- Unknowns before change: whether `prompty` can parse from a string (confirmed `loader::load_from_string`, prompty 2.1.0 loader.rs:144); whether release ships asset files (it does not — web/dist is embedded, so embedded prompts are the correct single-binary mechanism).
- Files expected to touch (planned): `src/prompts.rs`, `src/http.rs`, `tests/prompt_assets_wiring.rs` (new), this plan.

## 1. Outcome
The production binary runs with the real `planner` prompt template loaded from embedded assets; if the manifest or planner asset is missing/malformed the server reports the named refusal (`E_TEMPLATE_INVALID` / `E_TEMPLATE_UNKNOWN` / `E_TEMPLATE_LICENSE_MISSING` / `E_TEMPLATE_REF_FORBIDDEN`) on `/health` and in logs and never silently serves with an empty registry. Tests pin planner presence and invalid-manifest refusal.

## 2. Current behaviour (evidence)
- `http.rs:102,142,179` each pass `Arc::new(TemplateRegistry::default())` into `RunEngine::new`; `TemplateRegistry` derives `Default` (`prompts.rs:98`), so the registry is an empty `BTreeMap`.
- Assets are present and valid at `crates/hiverouter-core/assets/prompts/` (manifest 192 B, planner.prompty 462 B); `tests/prompt_assets.rs:28` loads them from disk with `CARGO_MANIFEST_DIR`, proving the loader works only in tests.
- The binary (`src/main.rs:123`) calls `AppState::with_graph`, so production boots the empty registry.

## 3. Decisions
1. **Embedded assets, not runtime paths.** The release is a single binary that already embeds `web/dist` via rust-embed; a runtime `data_dir/assets/prompts` path would ship nothing. Embedded via `include_str!` also turns "missing at runtime" into a compile error — a *stronger* fail-closed than a runtime refusal; runtime **invalid** content still refuses by name.
2. **Reuse frozen refusals** (`E_TEMPLATE_*`, `E_TEMPLATE_UNKNOWN`) instead of adding a new contract code (contract crate is outside scope).
3. **Non-fatal boot with explicit failed health gate**, because main.rs (out of scope) calls the infallible constructor. The named refusal is recorded as a required failed `Check`, `AppState.prompt_assets_error`, an error log, and `/health` 503 — not a silent fallback. A fatal `try_with_graph` in main.rs is the recorded follow-up.
4. **Keep `TemplateRegistry::default()` only for `with_services` (tests)**, documented as a test seam.
5. **No temp-file or process-env tricks**: `prompty::loader::load_from_string(raw, base)` parses the embedded content exactly as the file loader does.

## 4. File-by-file change
- `src/prompts.rs`
  - `pub const EMBEDDED_MANIFEST: &str = include_str!("../assets/prompts/manifest.json");`
  - `pub const EMBEDDED_TEMPLATE_FILES: &[(&str, &str)]` (planner.prompty via `include_str!`).
  - `pub fn load_embedded() -> Result<Self, Refusal>` — same policy as `load()` over embedded bytes.
  - `pub fn load_from_manifest_text(manifest_text: &str, files: &[(&str, &str)]) -> Result<Self, Refusal>` — injectable form used by tests to prove invalid manifest/asset refusal; production uses `load_embedded`.
  - Shared `parse_prompty(raw, source_path, manifest)` used by both file and embedded paths (licence/ref/parse/empty-body checks unchanged).
  - Keep `load(directory)` for tests/fixtures; keep `Default` derive with a "test-only empty registry" doc.
- `src/http.rs`
  - `pub fn load_prompt_assets() -> Result<Arc<TemplateRegistry>, Refusal>`.
  - `AppState.prompt_assets_error: Option<Refusal>` + `prompt_assets_loaded()` accessor.
  - `AppState::new` / `AppState::with_graph`: load embedded templates; on failure push required failed `Check {name:"prompt_assets", code: refusal.code()}` and log; on success no error.
  - `AppState::with_services`: unchanged `Default` (documented test seam).
  - `HealthResponse.prompt_assets: {loaded, error_code}` populated by `/health`; required-check failure keeps `/health` at 503.
- `tests/prompt_assets_wiring.rs` (new): planner present + render; production constructor loads planner into `engine.templates`; invalid manifest JSON → `E_TEMPLATE_INVALID`; missing embedded file → `E_TEMPLATE_UNKNOWN`; bad licence → `E_TEMPLATE_LICENSE_MISSING`; env ref → `E_TEMPLATE_REF_FORBIDDEN`; empty-body prompty → `E_TEMPLATE_INVALID`; embedded constants non-empty.

## 5. Failure modes
- Embedded asset malformed at release build: compile succeeds (include_str! only checks existence) → production constructor records named refusal + `/health` 503; tests cover the parse refusal.
- Compile-time missing file: build fails with the include_str! path — fail closed at build.
- Loader panic on malformed front matter: existing `catch_unwind` in `parse_prompty` maps to `E_TEMPLATE_INVALID` (unchanged).
- Test constructor keeps empty registry by design; tests that need templates must load them explicitly.
- `/health` consumers: new payload field is additive; existing assertions read `status`/checks only.

## 6. Tests / commands
```powershell
$env:PATH="$env:USERPROFILE\.cargo\bin;C:\Users\Riley\Desktop\hiverouter-v1\tools\w64devkit\bin;"+$env:PATH
cd C:\Users\Riley\Desktop\hiverouter-v1\backend
cargo test -p hiverouter-core --test prompt_assets
cargo test -p hiverouter-core --test prompt_assets_wiring
cargo test -p hiverouter-core --tests
```
Acceptance: all green; new tests prove planner present and invalid manifest refuses by name.

## 7. Rollback
Revert `src/prompts.rs` + `src/http.rs` and delete `tests/prompt_assets_wiring.rs` + this plan; no contract, DB, or config change; `TemplateRegistry::default()` remains for tests either way.

## 8. Out of scope / blockers
- `src/main.rs` (out of write scope) still calls the infallible `with_graph`; fatal process exit on bad assets needs `try_with_graph` wired there (Not done; recorded).
- `personas.json` production wiring is a separate surface (prompt persona loading is only used by tests today).
- No commit (Lead owns commits).

## 9. Done / Not done / Unverified

### Done (observed, quoted)
- `src/prompts.rs`: embedded `EMBEDDED_MANIFEST` + `EMBEDDED_TEMPLATE_FILES` (`include_str!`, compile-time fail closed); `TemplateRegistry::load_embedded()`; `TemplateRegistry::load_from_manifest_text()` (injectable policy seam); shared `parse_prompty` (same licence/ref/parse/empty-body policy for file and embedded bytes); `load(directory)` kept for fixtures; `Default` kept and documented as the test-only empty seam.
- `src/http.rs`: `AppState::new` and `AppState::with_graph` (binary boot path) now load the embedded planner registry; `AppState.prompt_assets_error` + `prompt_assets_loaded()`; failed load = required failed `Check{name:"prompt_assets", code: refusal.code()}` + `tracing::error!` + `/health` `prompt_assets:{loaded:false,error_code}` (required check failure → `/health` 503); `with_services` keeps `TemplateRegistry::default()` documented as the test-only seam; `HealthResponse` gains the additive `prompt_assets` field.
- `tests/prompt_assets_wiring.rs` (new, 11 tests): embedded manifest/planner present; embedded registry renders planner (system+user, `{{ task }}` substituted); production `AppState::new` **and** boot `AppState::with_graph` put `["planner"]` into `engine.templates` (not empty); `with_services` empty seam pinned; refusals by name: malformed manifest JSON → `E_TEMPLATE_INVALID`; manifest naming a missing file → `E_TEMPLATE_UNKNOWN`; GPL licence → `E_TEMPLATE_LICENSE_MISSING`; `${env:}` → `E_TEMPLATE_REF_FORBIDDEN`; empty body → `E_TEMPLATE_INVALID`; live `/health` reports `prompt_assets.loaded=true`, no error code, status not failing.
- Test evidence (commands run this seat, `CARGO_EXIT` quoted):
```
$ cargo test -p hiverouter-core --test prompt_assets --test prompt_assets_wiring
running 2 tests  (prompt_assets.rs)      test result: ok. 2 passed; 0 failed; 0 ignored
running 11 tests (prompt_assets_wiring.rs) test result: ok. 11 passed; 0 failed; 0 ignored
CARGO_EXIT=0

$ cargo test -p hiverouter-core --tests
unittests src/lib.rs: 39 passed; 0 failed
chat_store 5, context_switch 2, graph_authority 7, handshake 2, http_boot 6,
kill_restore 2 passed/1 ignored, prompt_assets 2, prompt_assets_wiring 11,
provider_adapter 6, provider_live 0 passed/1 ignored (live key gated),
provider_streaming 7, recovery_gates 3, run_lifecycle 7, snapshot_history 4,
ws_transport 13  → all suites ok; 0 failed
CARGO_EXIT=0
```
- Formatting: new code adjusted to rustfmt style. `rustfmt --check` on the touched files still reports **pre-existing** baseline diffs (`load_personas` signature wrap, existing route/refusal call wrapping, import order) that exist at HEAD and are untouched by this change; no repository-wide reformat was done (keeps the diff surgical).

### Not done (by design / out of scope)
- **Fatal boot exit on invalid assets**: `src/main.rs` calls the infallible `AppState::with_graph` and is outside this seat's write scope. In scope, a bad asset load is non-silent (required failed check, `/health` 503, error log, `prompt_assets_error`) and planner renders refuse `E_TEMPLATE_UNKNOWN`; a follow-up should add `AppState::try_with_graph(...) -> Result<Self, Refusal>` wiring in main.rs so the process exits with the named code before binding.
- Runtime `data_dir/assets/prompts` path intentionally not used: the release is a single binary (web/dist embedded), so a runtime path would ship nothing; embedded is the correct mechanism.
- Persona (`personas.json`) production wiring is a separate surface; only tests load it today.
- No commit (Lead owns commits); no changelog entry.

### Unverified
- The `prompt_assets` **failed-check + `/health` 503** branch cannot be exercised end-to-end here because the assets are compile-time embedded and valid: producing it needs a deliberately mutated build (out-of-tree). The refusal *policy* it depends on (`load_from_manifest_text` → `E_TEMPLATE_*`) is covered by 5 tests; the degrade branch itself is only source-verified.
- No release-binary smoke (`cargo build --release` + boot) was run by this seat; the unit/integration test binary boots the router and `AppState`, not the packaged artifact.
- Working tree was **not quiescent**: other lanes have uncommitted changes (`src/main.rs`, `src/checks.rs`, `src/provider_executor.rs`, `Cargo.lock`, several docs/e2e/web files). This seat touched only `src/prompts.rs`, `src/http.rs`, the new test, and this plan; the test run is against that mixed tree.

### Quiescence record (observed `git status --porcelain`, excerpt)
```
 M backend/crates/hiverouter-core/src/http.rs          <- this seat
 M backend/crates/hiverouter-core/src/prompts.rs       <- this seat
 M backend/crates/hiverouter-core/src/main.rs          <- other lane (provider key check)
 M backend/crates/hiverouter-core/src/checks.rs        <- other lane
 M backend/crates/hiverouter-core/src/provider_executor.rs
 M backend/crates/hiverouter-core/tests/provider_streaming.rs
?? backend/crates/hiverouter-core/tests/prompt_assets_wiring.rs   <- this seat (new)
?? plans/waves/W5-PROMPT-ASSETS.md                                <- this seat (new)
```

