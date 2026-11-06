# PHASE 6 RECON — Tools, Approvals & Execution Safety

Written before Phase 6 implementation edits (2026-11-06 03:41 CST). Blueprint card: Phase 6 (WPs 6.1–6.14).

## Recon
- **Target behavior:** `backend/crates/hiverouter-tools` is an empty stub (doc comment only). This phase adds
  the single tool contract + registry (schemars JSON schemas, per-tier allowlists), the approval/consent gate
  (uuid-v7 nonce, single-use replay refusal E_APPROVAL_REPLAY, TTL expiry E_APPROVAL_TIMEOUT, zero side
  effects), Kill/Cancel enforcement (CancellationToken + Job Object teardown), the Windows workspace jail
  (canonical root, six escape vectors → E_JAIL_ESCAPE), a shell allowlist with stdout cap/timeout, and the
  append-only run ledger (permission class + args_sha256, never argument bodies).
- **Entry point:** `backend/crates/hiverouter-tools/src/lib.rs` (modules `tool`, `registry`, `approval`,
  `jail`, `shell`, `cancel`, `ledger`). No binary in this phase (the `hiverouter` CLI lands in Phase 12).
- **Callers:** `hiverouter-core` run executor/spawn path will call `ToolRegistry::resolve`, gate, and execute;
  the Phase 4 scheduler calls the kill/cancel registry; Phase 11 census reads the registry + allowlists.
  Direction: core → tools (never the reverse), so no `hiverouter-core` dependency.
- **Tests + command:** `cargo test -p hiverouter-tools` from `backend/` (offline, tempdir workspaces,
  no network). Windows-only jail/Job Object assertions are `#[cfg(windows)]` and MUST run on this host
  (Windows 11) — a non-Windows fallback would be a BLOCKED item, not a silent pass.
- **Data flow (success + one failure path):** agent tool call (tool id + JSON args) → `ToolRegistry` tier
  allowlist check (`E_TOOL_NOT_ALLOWED` / `E_TOOL_NOT_FOUND`) → schemars schema validation
  (`E_TOOL_SCHEMA`) → permission class: Read tools execute; Mutate/Execute/Network tools create an
  `ApprovalRequest` (args_sha256 binds exact arguments) and return `E_APPROVAL_REQUIRED` with zero side
  effects → on `AllowOnce`/`AllowSession` the single-use nonce is consumed (replay ⇒ `E_APPROVAL_REPLAY`;
  expired ⇒ `E_APPROVAL_TIMEOUT`) → jailed filesystem/shell execution under the workspace root
  (`E_JAIL_ESCAPE`, `E_TOOL_TIMEOUT`, `E_TOOL_CANCELLED`) → ledger row (run/task/tool/permission/args hash/
  outcome/refusal) appended before the caller mutates durable state. Kill: one CancellationToken per run,
  child Job Object killed on close, no running future survives.
- **Unknowns / evidence still missing:**
  - Approval persistence vs core DB: core owns the rusqlite `Db`; this crate exposes `migrate(&Connection)`
    + an `ApprovalStore` trait (SQLite-backed impl over `Arc<Mutex<Connection>>` and an in-process impl for
    tests); exact core `Db` type is pending B-Core's pub surface. Replay is enforced by a `UNIQUE(nonce)`
    table, not by process memory.
  - `win32job 2.0.3` Windows semantics: must be proven on this target by the test (Blueprint risk row).
    If the Job Object spike fails, the phase records BLOCKED rather than substituting another primitive.
  - `cap-std 4.0.3` remains spike-gated/not in the lockfile; the jail uses canonicalise + prefix checks +
    full-path re-join and refuses escapes instead of clamping (blueprint WP 6.14 decision).
- **Files I expect to touch:** `backend/crates/hiverouter-tools/{Cargo.toml,src/*.rs,tests/*.rs}`,
  `docs/plans/PHASE-6-{RECON,EVIDENCE}.md`, `docs/claims/*`, `docs/contracts/ADAPTERS.md`. Never core/contract/web/blueprint.

## Scope check against the blueprint card
- 6.1/6.2 Linux seccomp + Docker deleted (D1); 6.3 Rust registry only, no MCP bridge; 6.4 local retry only.
- 6.5 tool contract; 6.6 approval gate; 6.7 cancel/kill; 6.8 jail + shell allowlist; 6.9 run ledger;
  6.10 capability seed loaded by Phase 11 census; 6.11/6.13 single approval owner + wire contract;
  6.12/6.14 single jail owner (win32job; cap-std spike-gated, not adopted).
- Exit-gate mapping: allowlisted success vs `E_TOOL_NOT_ALLOWED`; unapproved mutation `E_APPROVAL_REQUIRED`
  with unchanged bytes; cancel/kill no surviving task + Cancelled broadcast; 6 escape vectors refused;
  every refusal in the ledger; nonce replay + TTL expiry + jail matrix + child job limits.
