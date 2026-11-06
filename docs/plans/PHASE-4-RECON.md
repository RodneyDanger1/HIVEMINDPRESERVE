# PHASE 4 RECON — Scheduling, Delegation & Run Lifecycle

ForgeWright recon ledger (skill: `forgewright-recon`). Written BEFORE any Phase 4 edit.

## Recon

- **Target behavior today.** Phases 1-3 provide the contract, boot/health, `/ws` transport with Origin+token gate, event
  bus, petgraph graph authority, prompt/persona loading, typed handshake and context store. There is **no** run/task
  entity, no chat store, no planner and no dispatch loop — `SUBMIT_PROMPT`/`SPAWN_SUB_HIVE`/`CANCEL_RUN`/`KILL_WORKER`
  are currently refused by the interim transport fallback with `E_INTERNAL_ERROR` (documented in PHASE-2-EVIDENCE §5).
- **Entry point.** New core modules `src/entities.rs`, `src/store.rs`, `src/caps.rs`, `src/execution.rs`; migration v2
  added to `src/db.rs`; dispatch arms added to `src/ws.rs`; `AppState` grows an `Arc<RunEngine>`.
- **Callers.** `ws.rs` command dispatcher (4 commands); `main.rs` boot (default engine with no provider configured);
  Phase 6 tools will be executed through the engine's `ProviderExecutor` seam; Phase 9 snapshot/history reads the same
  run/task rows.
- **Tests + command.** `cargo test -p hiverouter-core run_lifecycle_plans_and_dispatches`, `cargo test -p hiverouter-core
  caps_refused_by_name`, error path `cargo test -p hiverouter-core provider_error_preserves_partial_state`, plus
  `cargo test -p hiverouter-core chat_store` for the message store.
- **Data flow (one path + one failure path).** `SUBMIT_PROMPT` envelope → `Command::SubmitPrompt` → engine
  `submit_prompt(text, project_id, correlation_id)` → prompt validation (1..=8000 chars) → single active run rule
  (`E_RUN_ACTIVE`) → `runs` row inserted (uuid v7, status `queued`) → `RUN_CREATED` event → background dispatch task:
  one provider `plan()` call (per-plan, capped by the 40-call/40-step ceiling) → plan validates (acyclic, nodes <= 40) →
  task rows + graph nodes created with hard caps (depth <= 3, fan-out <= 8, agents <= 64) → leaf tasks execute through
  `ProviderExecutor::execute` streaming `REPLY_DELTA` chunks and a final `REPLY_DONE(usage)` → assistant text stored in
  `chat_messages` → run row `completed` → `RUN_STATUS(completed)`.
  Failure path: a provider 429 mid-plan → run row `failed` with `error_code=E_LLM_RATE_LIMIT`, completed tasks/chat
  messages stay persisted, `RUN_STATUS(failed, error_code)` is published, and no reply is fabricated. Cancellation:
  `CANCEL_RUN` cancels the run's `tokio_util::sync::CancellationToken`; in-flight `execute` selects on it and the
  `JoinSet` drains to empty before the run row becomes `cancelled`.
- **Unknowns / evidence still missing.**
  - No live provider is configured in this phase (Phase 5 owns the client). Production default is an
    `UnconfiguredExecutor` that refuses with `E_KEY_MISSING` — never fake success; tests use a mock executor defined in
    the test files only, which is permitted by the blueprint ("Phase 4 spawn tests use a tool-less mock executor").
  - Token/cost budgets are Phase 5's; this phase consumes a usage type but hardcodes no prices.
  - `E_AGENT_LIMIT` is not in the frozen UI-COMMANDS §5 list but is in blueprint §2a budget invariants; it is registered in
    the contract refusal table (already present) and used by the cap gate (superset already recorded).
- **Files expected to touch.** new `backend/crates/hiverouter-core/src/{entities.rs,store.rs,caps.rs,execution.rs}`;
  edits to `src/{db.rs,lib.rs,ws.rs,http.rs,main.rs}`; new
  `backend/crates/hiverouter-core/tests/{run_lifecycle.rs,chat_store.rs}`;
  `docs/plans/PHASE-4-{RECON,EVIDENCE}.md`; `docs/claims/*`. **Never:** provider/tools crates, `web/`, FigCraft,
  blueprint.
- **Rollback.** Migration v2 has a tested down-migration; removing the dispatch arms restores the interim refusals.
  No destructive writes outside the configured data directory.

**Recon completeness:** complete — every module this phase edits has been read (`db.rs`, `ws.rs`, `http.rs`, `lib.rs`)
and the new external APIs are limited to `tokio_util::sync::CancellationToken`, `tokio::task::JoinSet` and
`async_trait`, all already pinned in the workspace.
