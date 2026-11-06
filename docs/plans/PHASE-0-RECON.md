# PHASE 0 RECON — new project, empty workspace

- **Target behavior:** create HiveMind v1 from scratch (no existing product code). Entry point is the build root
  `C:\Users\Riley\Desktop\hiverouter-v1`; reference-only blueprint at `C:\Users\Riley\Desktop\hivemind-blueprint`.
- **Entry point:** `backend/crates/hiverouter-core/src/main.rs` (to be created), served web `web/index.html` (to be created).
- **Callers:** none (empty project); the reference contract is the blueprint (read-only).
- **Tests + command:** none yet; first test command will be `cargo check` (Phase 1) and `npm run typecheck` (web scaffold).
- **Data flow (planned):** UI command frame → localhost WS (axum) → contract validation → run/task store (SQLite) →
  provider/tool execution → event fan-out → React state → render-back. Failure path: validation refusal → Error frame →
  named toast + ledger row; no silent success.
- **Unknowns / evidence missing:** Rust toolchain absent (installing per-user), MSVC absent (GNU+winlibs route),
  FigCraft command schemas (CLI `commands --json` available; agents must read), disk headroom.
- **Files expected to touch:** `backend/**`, `web/**`, `tools/**`, `docs/**`, root manifests. Never
  `C:\Users\Riley\Desktop\hivemind-blueprint\**`.
- **Recon completeness:** complete for an empty project (entry points do not exist yet; creation is the task).
