# PHASE 3 RECON — Agent Context & Orchestration Handshake

ForgeWright recon ledger (skill: `forgewright-recon`). Written BEFORE any Phase 3 edit.

## Recon

- **Target behavior today.** Phases 1-2 provide: frozen wire contract (`hiverouter-contract`), boot/`GET /health`/
  `GET /api/session`, SQLite migration engine, server event bus, `/ws` transport with Origin+token gate and petgraph task
  graph. There is **no** prompt/persona asset loading, no typed handshake frame and no context switching. Phase 3 adds
  those on top of the single Phase 2 transport (no second socket).
- **Entry point.** New core modules `src/prompts.rs` (prompty-backed template + persona registry), `src/handshake.rs`
  (typed `HandshakeFrame`), `src/context.rs` (scoped context + rewire context switch); wired into
  `src/lib.rs`/`http.rs`/`ws.rs` for the rewire path.
- **Callers.** `ws.rs` rewire handler will trigger `context::ContextStore::switch_on_rewire`; Phase 4 planner consumes
  `prompts::TemplateRegistry::render`; Phase 4 dispatch validates child `HandshakeFrame` results. Existing callers of
  these modules: none yet (`lib.rs` re-export is the first consumer after this phase).
- **Tests + command.** `cargo test -p hiverouter-core prompt_schema_roundtrip rewire_context_switch
  template_schema_roundtrip`; the phase preflight is `cargo check` (quoted in PHASE-3-EVIDENCE).
- **Data flow (one path + failure paths).** `assets/prompts/manifest.json` (template id + SPDX + source_url + fetched_at)
  → `TemplateRegistry::load` validates the licence allowlist (`E_TEMPLATE_LICENSE_MISSING`), pre-scans the raw
  `.prompty` file for `${env:}`/`${file:}` (`E_TEMPLATE_REF_FORBIDDEN`) → `prompty::loader::load` parses the YAML
  front-matter into a typed `Agent` (parse failure → `E_TEMPLATE_INVALID`) → `render(template_id, inputs)` substitutes
  `{{ input }}` placeholders and splits the markdown body on `system:`/`user:`/`assistant:` role markers into
  `Vec<RenderedMessage>`.
  Failure path: rendering an unknown template id returns `E_TEMPLATE_UNKNOWN`; loading a persona that references an
  unknown template returns `E_PERSONA_UNKNOWN`; a malformed child `Result` handshake payload returns
  `E_SCHEMA_INVALID` to the parent (never fabricated child data).
- **Rewire context switch.** `ContextStore` keeps one `ScopedContext` per task keyed by facts
  `parent:<parent_task_id>:<key>`. On rewire: remove every fact whose key is prefixed with the old parent id, then inject
  the new parent's facts, update `parent_task_id` and `allowed_tool_ids`; the outcome records cleared/injected keys.
  Test `rewire_context_switch` asserts the previous parent's scoped facts are **absent** from the new token set and the
  new parent's facts are present.
- **Unknowns / evidence still missing.**
  - `prompty 2.1.0` exact loader/render API: the loader returns a typed `Agent` from a `.prompty` file (docs.rs source
    read: `loader::load(path) -> Result<Agent, LoadError>`); the pipeline `prepare()` signature is not documented on the
    fetched pages, so role rendering is implemented in this crate over the typed `Agent` (deterministic, tested) rather
    than relying on an unverified API. Prompty's own `${env:}/${file:}` resolution is refused before load by policy.
  - `prompty` panics on some invalid inputs (`Agent::load_from_value` calls `panic!` after validation); loads are wrapped
    in `catch_unwind` and mapped to `E_TEMPLATE_INVALID` so a bad user asset can never kill the server.
  - Handshake transport: `HandshakeFrame` is a typed in-process type in this phase; the Phase 2 socket carries the
    existing command/event envelopes, so handshake frames are dispatched through the run engine in Phase 4 (no second
    socket, per blueprint WP 3.5).
- **Files expected to touch.** new `backend/crates/hiverouter-core/src/{prompts.rs,handshake.rs,context.rs}`, edits to
  `src/{lib.rs,ws.rs}`; new assets `backend/crates/hiverouter-core/assets/prompts/{planner.prompty,manifest.json}`; new
  `backend/crates/hiverouter-core/tests/{prompt_assets.rs,handshake.rs,context_switch.rs}`;
  `docs/plans/PHASE-3-{RECON,EVIDENCE}.md`; `docs/claims/*`. **Never:** provider/tools, `web/`, FigCraft assets,
  blueprint.
- **Rollback.** New modules + one rewire hook; removing them restores Phase 2 behaviour. Assets are additive files.

**Recon completeness:** complete — every existing module this phase edits (`lib.rs`, `ws.rs` rewire branch) has been
read; the only new external API (`prompty` loader) was checked against its published source before use.
