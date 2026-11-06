# FINDINGS — b-core partition (contract + core)

Checkpoint `3988622`. Severity: HIGH = user-visible capability missing/broken in the shipped binary; MEDIUM = important
integration gap or data-integrity risk; LOW = debt/drift with small blast radius. Evidence labels follow WAVE-SHEET §5.

## F-01 — HIGH — the shipped server cannot perform a live model run (`UnconfiguredExecutor`)
- **Observation (READ):** `backend/crates/hiverouter-core/Cargo.toml:19-38` depends on `hiverouter-tools` but **not** on
  `hiverouter-provider`; all three `AppState` constructors pass `Arc::new(UnconfiguredExecutor)` to `RunEngine::new`
  (`src/http.rs:77-82`, `:117-122`, `:154-159`); `UnconfiguredExecutor::plan/execute` return `E_KEY_MISSING`
  (`src/execution.rs`). `DEEPSEEK_API_KEY` presence is reported by `SETTINGS_STATUS`/`/diagnose` but no code path reads a
  client or sends a request from core.
- **Impact:** `SUBMIT_PROMPT` over the live WS/HTTP UI always terminates with a run failed `E_KEY_MISSING` even when the
  key is configured. The engine, planner, caps, streaming and persistence are exercised only by test doubles.
- **Evidence class:** READ (no rebuild/live run performed this audit; previous phase evidence shows engine tests are
  green with a test-only mock, and D-2/D-3 live probe only exercised settings/file/approval commands).
- **Roadmap:** wire the Phase-5 provider crate as the production `ProviderExecutor` (adapter maps provider stream to
  `StreamedReply` deltas + `Usage`), add a live-env integration test tagged `#[ignore]` for key-gated runs.

## F-02 — MEDIUM — prompt/persona assets are not loaded by the production server
- **Observation (READ):** `src/http.rs:82/122/159` pass `Arc::new(TemplateRegistry::default())` (empty); the authored
  assets under `assets/prompts/` are loaded only by `tests/prompt_assets.rs:10`. `execution.rs::planning_prompt` therefore
  always falls back to the raw prompt string; `handshake`/`persona` records are unused in production.
- **Impact:** prompt engineering, persona tiers and licence gating have no product effect today; the raw prompt is sent
  to the planner executor (if one were wired).
- **Roadmap:** load `TemplateRegistry::load(assets_dir)` + `load_personas` at boot (fail with `E_TEMPLATE_*`/`E_PERSONA_*`
  by name), pass the registry into `RunEngine`, and record the chosen persona/template id on the run row.

## F-03 — MEDIUM — Phase 3 handshake/context-switch modules are not wired to the live rewire/engine paths
- **Observation (READ):** `src/ws.rs` rewire arm (`:338-353`) calls only `graph.rewire`; `HandshakeLog`/`HandshakeFrame`
  have no `src/` caller (grep: only `tests/handshake.rs`); `ContextStore` is constructed at `execution.rs:208` but
  `switch_on_rewire` has no production call site. Their behaviour is proven by unit tests only
  (`context_switch.rs:11`, `handshake.rs:28/67`).
- **Impact:** a live rewire moves the graph node but does not clear/inject scoped context facts, and no child handshake
  is routed through the engine — the Phase 3 acceptance criteria hold only at module level.
- **Roadmap:** in the WS rewire handler, after a successful graph rewire, call `ContextStore::switch_on_rewire` with the
  new parent's facts and publish the outcome; route child handshake frames through `RunEngine` with `HandshakeLog`
  idempotency; add an end-to-end WS test for the switch.

## F-04 — LOW — migration v1 hardcodes the contract schema version instead of binding the Rust const
- **Observation (READ):** `src/db.rs` v1 `up` SQL writes `'hiverouter.v1'` into `app_meta.schema_version`, while
  `hiverouter_contract::SCHEMA_VERSION` is the source of truth. The DB test asserts equality, so a contract bump would
  fail a test (good) but with a confusing DB-migration error rather than a compile-time one.
- **Roadmap:** parameterise the migration value from `SCHEMA_VERSION` (or add a semantic assertion at boot).

## F-05 — LOW — history read model silently degrades a corrupt revision instead of failing the revision read
- **Observation (READ):** `src/history.rs::revisions` calls `self.snapshots.read(...).ok().and_then(|e| e.run_status)`;
  a corrupt file yields `run_status=None` in the scrubber metadata while `state_at` on the same revision correctly
  returns `E_SNAPSHOT_CORRUPT`.
- **Impact:** inconsistent user-visible signal for one corrupt revision; the no-silent-reset rule is preserved elsewhere
  (recovery reports it). **Roadmap:** propagate the refusal or record a `corrupt: bool` per revision in `RevisionMeta`.

## F-06 — LOW/MEDIUM — store row mappers use silent fallbacks on corrupt rows
- **Observation (READ):** `src/store.rs` row mappers use `parse().unwrap_or_else(...)`, `parse_state(_) => Failed`,
  `Timestamp::parse(...).unwrap_or_else(epoch)`, `ChatRole::from_str(...).unwrap_or(System)`. A corrupted/tampered DB
  row silently becomes nil id/failed/epoch instead of a named `E_SCHEMA_INVALID`/`E_DB_BUSY` style refusal.
- **Impact:** low likelihood (local single-writer DB) but violates "refuse by name, never fake data" if it happens.
- **Roadmap:** map decode failures to a typed `DbError::Decode` → refusal, and fail the read instead of substituting.

## F-07 — LOW — three near-duplicate `AppState` constructors
- **Observation (READ):** `src/http.rs:51-159` repeats bus/graph/snapshot/engine/history/tools construction three times
  (`new`, `with_graph`, `with_services`) with small differences. **Roadmap:** one builder with optional seams; reduces
  the chance a future service is wired in two of three paths (exactly the class of defect that produced D-2/D-3).

## F-08 — LOW — `/assets/` (directory-like path) resolves to SPA index instead of 404
- **Observation (READ):** `src/static_ui.rs` treats any extension-less path as an SPA route, so `/assets/` returns
  index.html 200; missing specific assets correctly return 404. Not observable as broken UI, but it can mask a mistyped
  asset directory. **Roadmap:** exclude paths under `assets/` from the SPA fallback.

## F-09 — LOW — `ws.rs` is a 500-line module mixing auth, handshake, replay and command dispatch
- **Observation (READ):** single module carries `SessionAuth`, upgrade gating, hello/replay, socket loop, and every
  command arm. It is well-tested (13 tests) but has the highest change-coupling in the partition.
  **Roadmap:** split `ws::{session,auth,dispatch}` without behaviour change; keep the 13-test suite as the pin.

## F-10 — LOW — no test creates an approval request through the product tool-call path
- **Observation (READ):** `APPROVAL_RESOLVE` is covered end-to-end over WS, but the test seeds the pending approval row
  directly through the tools gate (`ws_transport.rs:306`). No core test exercises provider-tool-call → `APPROVAL_REQUEST`
  event → UI resolve. **Roadmap:** with F-01 fixed, add a provider-mock tool-call test that emits `APPROVAL_REQUEST` and
  resolves it through the WS command.

## F-11 — INFO — deliberately decision-gated / non-goals (not defects)
- `DIFF_PROPOSE/APPLY/REJECT` are refused `E_DECISION_REQUIRED` by contract (frozen doc: "NOT FROZEN — do not implement").
- `docs/contracts/UI-COMMANDS-v1.md` §5 was extended with `E_FILE_TREE_UNAVAILABLE` for the truthful unavailable-workspace
  refusal (D-2 fix); this is a recorded contract superset and the matrix row lists it.
- Recovery restores only the latest snapshot per run; older revisions stay readable through the history API by design.

## Provenance / donor evidence
- The contract/engine/graph/transport modules are original implementations for this repository (no donor source copied);
  the only external behavioural reference is `microsoft/prompty` (pinned crate, F8) and upstream API docs captured in
  `blueprint/FACT-REGISTER.md`. No copied code was found in the partition; licence headers are MIT/Apache-2.0 project
  licence in the manifests. `UNVERIFIED`: independent code-origin scanning (e.g. snippet matching against donors) was
  not performed in this read-only audit.
