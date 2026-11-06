# W12 — 1.2 tool-loop interface contract (Lead-owned, frozen for this wave)

**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **Branch:** `master` · **No release builds during coding waves.**
This file is the shared API contract for the four coding waves. Change it only with a Lead note; do not invent alternate names.

## Contract (already uncommitted, 23 events)

- `TOOL_CALL {run_id, task_id, tool_call_id, tool, args_sha256, turn}`.
- `TOOL_RESULT {run_id, task_id, tool_call_id, outcome: ok|refused|denied|timeout|cancelled, code?, bytes?, sha256?}`.
- `APPROVAL_REQUEST` already has optional `nonce, run_id, task_id, tool_call_id, args_sha256, expires_at`.
  **Producer MUST populate all six** for model-originated side effects. Do not make them required on the wire in this wave.
- No new event/command/refusal code in W12. Existing 65 refusals only.

## Provider layer

`backend/crates/hiverouter-provider/src/tool_assembler.rs` (exists, uncommitted):
```rust
pub struct AssembledToolCall { pub index: u32, pub id: String, pub name: String, pub arguments: serde_json::Value }
pub enum ToolCallAssemblyError { /* existing */ }
pub struct ToolCallAssembler { /* existing */ }
```
`backend/crates/hiverouter-provider/src/client.rs`:
- `StreamEvent::ToolCallDelta` stays the fragment type.
- `delta_event` must emit **every** entry of `tool_calls` (not only `.first()`), one `ToolCallDelta` per entry, in index order.
- Add `pub fn tool_specs_from_contracts(...)`? No — tool specs belong to core/tools. Provider only exports `AssembledToolCall`.
`backend/crates/hiverouter-core/src/execution.rs`:
```rust
pub struct StreamedReply {
    pub deltas: Vec<String>,
    pub usage: Usage,
    pub tool_calls: Vec<hiverouter_provider::tool_assembler::AssembledToolCall>,
}
```
All constructors of `StreamedReply` must set `tool_calls` (empty for plain replies). Update all callers/tests.

## Core tool loop

New file `backend/crates/hiverouter-core/src/tool_loop.rs`:
```rust
pub const MAX_TOOL_TURNS: u32 = 4;
pub const MAX_CALLS_PER_RUN: u32 = 16;
pub const MAX_TOOL_RESULT_BYTES: usize = 65_536;

pub struct ToolCallPlan {
    pub tool_call_id: String,
    pub tool: String,
    pub args: serde_json::Value,
    pub args_sha256: String,
    pub turn: u32,
}
pub struct ToolExecution {
    pub tool_call_id: String,
    pub outcome: hiverouter_contract::ToolOutcome,
    pub code: Option<String>,
    pub bytes: Option<u64>,
    pub sha256: Option<String>,
    pub text: String,
}
pub fn args_sha256(args: &serde_json::Value) -> String;
pub fn validate_call(registry: &hiverouter_tools::ToolRegistry, tier: hiverouter_tools::AgentTier, tool: &str, args: &serde_json::Value) -> Result<(), hiverouter_contract::Refusal>;
pub fn classify_permission(registry: &hiverouter_tools::ToolRegistry, tool: &str) -> Option<hiverouter_tools::PermissionClass>;
```
The Lead owns the final integration of this module into `execution.rs`; seats must not duplicate it.

## Persistence (DB v4)

`backend/crates/hiverouter-core/src/db.rs` appends exactly one migration:
```sql
ALTER TABLE runs ADD COLUMN tool_turns INTEGER NOT NULL DEFAULT 0;
ALTER TABLE runs ADD COLUMN tool_calls INTEGER NOT NULL DEFAULT 0;
CREATE TABLE IF NOT EXISTS tool_turns (
  id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  task_id TEXT NOT NULL,
  tool_call_id TEXT NOT NULL,
  tool TEXT NOT NULL,
  args_sha256 TEXT NOT NULL,
  outcome TEXT NOT NULL,
  code TEXT,
  bytes INTEGER,
  sha256 TEXT,
  turn INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  UNIQUE(run_id, tool_call_id)
);
```
Required `Database` methods (exact names):
- `pub fn record_tool_turn(&self, turn: &ToolTurnRecord) -> Result<(), DbError>`
- `pub fn tool_turns_for_run(&self, run_id: RunId) -> Result<Vec<ToolTurnRecord>, DbError>`
- `pub fn increment_run_tool_counters(&self, run_id: RunId, turns: u32, calls: u32) -> Result<(), DbError>`
- `pub fn run_tool_counters(&self, run_id: RunId) -> Result<(u32, u32), DbError>`
`ToolTurnRecord` is a new public struct in `db.rs` with the same fields.

## Tools layer

New file `backend/crates/hiverouter-tools/src/executor.rs`:
```rust
pub enum ExecutedTool { Ok { text: String }, Refused { code: String, detail: String } }
pub fn execute_read_only(
    jail: &WorkspaceJail,
    registry: &ToolRegistry,
    tier: AgentTier,
    tool: &str,
    args: &serde_json::Value,
) -> Result<ExecutedTool, ToolError>;
```
Only `file_read` and `file_list` may execute. `file_write`, `shell_exec`, `spawn_agent`, `report_result`, `finish` return `Refusal::ToolNotAllowed` (they are approval/loop-owned). Register module in `lib.rs`.
`approval.rs`: add `pub fn request_tool_approval(gate: &ApprovalGate, conn: &Connection, run_id: RunId, task_id: TaskId, tool_call_id: &str, tool: &str, args: &serde_json::Value, now_unix_secs: u64) -> Result<ApprovalRequest, ToolError>`; it MUST use `hash_args(args)`, TTL, nonce and return the full request. Add ledger append on resolve in `ledger.rs`: `pub fn append_approval_decision(...)`.

## Core events

New helper in `core/src/tool_events.rs` or `execution.rs`:
- `publish_tool_call(run_id, task_id, plan, turn)` emits `TOOL_CALL`.
- `publish_tool_result(run_id, task_id, execution)` emits `TOOL_RESULT`.
Do not invent payload fields outside the contract.

## UI

`web/src/state/store.ts`:
```ts
export interface ToolActivityRecord {
  readonly toolCallId: string;
  readonly tool: string;
  readonly argsSha256: string;
  readonly turn: number;
  readonly outcome: 'pending' | ToolOutcome;
  readonly code?: string;
  readonly text?: string;
}
```
Add `readonly toolActivity: readonly ToolActivityRecord[]` and reduce `TOOL_CALL`/`TOOL_RESULT`; approval record gains optional `nonce`, `argsSha256`, `expiresAt`, `toolCallId`, `runId`, `taskId`.
New `web/src/panels/ToolActivityPanel.tsx`; extend `ApprovalOverlay.tsx` to show tool, args hash, run/task, expiry, approve/deny disabled while resolving. No new npm deps.

## E2E

New `e2e/specs/tool-loop.spec.ts` and `e2e/specs/approval-flow.spec.ts` using existing mocked WS harness; no paid path. They must fail if the store drops `TOOL_CALL`/`TOOL_RESULT` or if approve/deny is optimistic.

## Gate / docs

`docs/contracts/UI-COMMANDS-v1.md` must list 23 events and 16 states. `plans/waves/README.md` indexes W1–W12 with status + evidence. `tools/qa/check-docs-truth.mjs` must cover the UI-COMMANDS event count. `tools/qa/run-gates.ps1`: add `-RequireE2E`; release mode fails when `source_commit != HEAD`; G4 skip is not PASS.

## Rules for every coding seat

1. Read before edit; record Phase 1 JSON in your plan/receipt.
2. No `cargo build`, no `cargo test`, no Playwright, no release script during Waves 1–4. Static reasoning + file reads only. One final Lead build after all waves.
3. Touch only your assigned files. If an interface is missing, report it; do not edit another seat's file.
4. No TODO/stub/placeholder/mock product path. Tests may use existing test doubles.
5. Every receipt: files changed, exact exported symbols, compile risks, unverified items.
