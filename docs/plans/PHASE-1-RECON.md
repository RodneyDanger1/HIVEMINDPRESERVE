# PHASE 1 RECON — Contract, Toolchain & Boot

ForgeWright recon ledger (skill: `forgewright-recon`). Written BEFORE any Phase 1 edit.

## Recon

- **Target behavior.** There is no product code yet (empty Rust workspace). Phase 1 must create: (1) the frozen wire
  contract (envelope + commands + events + named refusals + uuid-v7 ids + schema_version) exactly per
  `docs/contracts/UI-COMMANDS-v1.md` (frozen 2026-11-06), (2) pinned Cargo manifests + committed lockfile, (3) a
  `hiverouter-core` binary that boots, serves `GET /health`, bootstraps SQLite, and runs the MSRV/Node checks, and
  (4) the `ui_command_matrix` + schema-roundtrip tests.
- **Entry point.** `backend/crates/hiverouter-core/src/main.rs` (to be created). Frozen vocabulary:
  `docs/contracts/UI-COMMANDS-v1.md:9-71`. Workspace manifest: `backend/Cargo.toml` (moved there from the repo root by
  the Lead during this recon; `members = ["crates/..."]` matches the actual `backend/crates/*` layout — no edit needed).
  Reference: `blueprint/HIVEMIND-BLUEPRINT-v1.md` Phase 1 card (lines 104-165).
- **Callers.** None exist today. Downstream consumers to design for: the React/TS frontend (wire vocabulary),
  B-Provider (Phases 5/6/11/12; consumes `Refusal`, ids, event frames — coordination message sent 2026-11-06), and
  B-Core Phases 2/3/4/9 (transport, graph, runs, persistence import this crate). No caller may be edited in this phase.
- **Tests + command.** From repo root (PATH includes `%USERPROFILE%\.cargo\bin` and `tools\w64devkit\bin`):
  preflight `rustc --version && cargo --version && node --version && npm --version`;
  gate `cargo check`; acceptance `cargo test contract_schema_roundtrip`, `cargo test ui_command_matrix`,
  `cargo test` (core db/boot/check tests). All tests are newly created by this phase (no pre-existing tests exist).
- **Data flow (planned, one path + one failure path).** UI JSON frame → `serde` decode `Envelope` → `schema_version`
  check (`hiverouter.v1`) → `type` check (COMMAND) → command-name lookup → jsonschema payload validation against the
  `schemars`-derived schema → typed payload deserialization → semantic validation (prompt length, zoom range) →
  decoded `Command` (later phases dispatch it).
  Failure path: unknown name → `Refusal::CommandUnknown` → serialized `ERROR` event payload
  `{code,message,how_to_fix,retryable}`; never a silent drop, never a panic. Event path: typed `Event` → envelope with
  monotonic `seq` (transport-owned in Phase 2) + uuid-v7 ids + RFC3339 `ts`.
- **Unknowns / evidence still missing (labelled).**
  - C toolchain (needed by `rusqlite` bundled): **RESOLVED during recon** — gcc 16.2.0 at
    `tools/w64devkit/bin` from the official GitHub release `skeeto/w64devkit v2.11.0`; asset 68,801,240 bytes,
    local SHA256 `54C2BB99...B2AF7B2B` = GitHub API asset digest; extracted and hello-world compiled+ran (evidence in
    PHASE-1-EVIDENCE.md). The earlier `%TEMP%\w64devkit.zip` was a Defender false positive; it was deleted, no
    exclusion was added, and the official asset is not quarantined.
  - `axum 0.8.9` / `rusqlite 0.41.1` / `schemars 1.2.3` / `jsonschema 0.60.2` exact API surface: UNVERIFIED until the
    first compile (FACT-REGISTER F2 risk row: "API differences from plan samples are UNVERIFIED until compilation").
  - Frozen-document conflicts to record, not resolve silently: blueprint Phase 1 card says `E_UNKNOWN_COMMAND` while
    the frozen contract says `E_COMMAND_UNKNOWN` (this implementation uses the frozen `E_COMMAND_UNKNOWN`;
    mismatch = claim-row note). Phase 1 WP 1.6 names `E_TEMPLATE_INVALID`/`E_PERSONA_UNKNOWN` not present in
    UI-COMMANDS §5 (registered as superset variants, codes unchanged). UI-COMMANDS §2 DIFF row names
    `E_DECISION_REQUIRED` while §5 lists `E_DIFF_DECISION_REQUIRED` (both registered; DIFF decode refuses with
    `E_DECISION_REQUIRED` per the per-command table).
  - `npm --version` not yet observed in this session (preflight will quote it).
  - Disk free at recon: 3.89 GB after toolchain install (floor 1.5 GB; monitor per boot).
- **Files expected to touch (exclusive ownership).** `backend/Cargo.toml` (workspace root), `backend/Cargo.lock`,
  `backend/crates/hiverouter-contract/**`, `backend/crates/hiverouter-core/**`, `backend/tests/**`,
  `docs/plans/PHASE-1-{RECON,EVIDENCE}.md`, `docs/claims/CLAIM-REGISTER.{csv,json}`.
  **Never:** `backend/crates/hiverouter-provider|tools`, `web/`, `docs/ui/figcraft`, `C:\Users\Riley\Desktop\hivemind-blueprint`.
- **Rollback.** New files only; root `Cargo.toml` members edit is revertible by git. No database is created inside the
  repo during tests (temp dirs only). No network calls in production code paths.

**Recon completeness:** complete for an empty project — entry points do not exist yet and creating them is the task;
every file this phase will touch has been read (root manifest, frozen contract, blueprint card, FACT-REGISTER pins) or
is created by this phase.
