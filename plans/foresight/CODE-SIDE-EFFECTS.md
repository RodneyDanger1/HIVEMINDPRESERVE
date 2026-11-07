# CODE RESEARCH 4 — Side-effect executor blueprint (`execute_approved`)

**Seat:** Code Research Planner 4 · **READ-ONLY** (this file is the only write) · Repo `C:\Users\Riley\Desktop\hiverouter-v1`
· **HEAD observed:** `37f0e7c` · **Date:** 2026-11-06 · **Status:** implementation blueprint (Rust shapes + exact anchors);
**no product code edited, no build/test run, no key access**.
**Input:** `plans/foresight/SIDE-EFFECT-EXECUTOR-PLAN.md` (Foresight Plan 2, 383 lines, sha256 `4821d642…a85655`),
read in full, plus the current source files listed in §1.
**Skills applied:** `/forgewright-recon`, `/forgewright-plan`, `/logic-gatekeeper` (Phase-1 contract, §12),
`/integration-logic`, `/logic-debugger`, `/truthseeker`, `/capability-auditor`.
**Purpose:** the exact Rust blueprint for items 3 & 16 (side-effect execution + grant enforcement) so a future
implementation wave can write code without re-deriving the design. Code blocks are **blueprints**, not compiled
product code; line anchors are current at `37f0e7c`.

## 1. Current line anchors (verified this session; file hashes recorded)

| # | File | Current facts (anchors) | bytes / sha256 |
|---|---|---|---|
| A1 | `tools/src/executor.rs` | `ExecutedTool` enum `:25`; `execute_read_only` `:52`; read/list helpers `:82,:102`; read-only refusal `E_TOOL_NOT_ALLOWED` `:59-67`. | 13,045 / `79969b4e…f3a9e98` |
| A2 | `tools/src/shell.rs` | caps `:26,:28`, timeout `:30`; `ShellPolicy` `:34`; `conservative_default` `:66` (cmd.exe/git.exe/cargo.exe); `allows` `:96`; `ShellRequest` `:114`; `ShellOutput` `:145`; `pub async fn execute` `:173` (allowlist → jailed cwd → env sanitize → spawn → Job Object → timeout/caps/cancel). | 17,299 / `d1ddd6fe…7345cb` |
| A3 | `tools/src/jail.rs` | `MAX_READ_BYTES` `:32`; `resolve` `:92`; `verify_containment` `:143`; `read` `:185`; **`write` `:223`** (parents created; caller owns approval); `list` `:250`; `escape` `:298`. | 16,815 / `1f6072ad…ab0949` |
| A4 | `tools/src/approval.rs` | TTL `:26`; `ApprovalRequest` `:85`; `hash_args` `:132`; `session_allowed` `:384`; `resolve_with_nonce` `:415`; `ApprovalGate` `:583`; `request` `:621`; **`verify_grant` `:694`** (AllowSession check + AllowOnce `executed_at` CAS); `request_tool_approval` `:771`. DDL `UNIQUE(nonce)`, `decision`, `decided_at`, `executed_at` (`:141-202`). | 44,475 / `c1b67554…e518128` |
| A5 | `tools/src/tool.rs` | `PermissionClass` `:30`/`requires_approval` `:55,:171`; `FileWriteArgs` `:255` (`path`,`content`,`expected_sha256?`); `ShellExecArgs` `:268` (`program`,`args`,`cwd?`); `standard()` `:295`; `resolve_call` `:446`. | 19,612 / `353ab4e6…319765` |
| A6 | `tools/src/env_safety.rs` | `ALLOWED_ENV` `:11`; `sanitize_std` `:27`; `sanitize_tokio` `:37` (used by `shell::execute` at `shell.rs:209`; key excluded). | 2,351 / (unchanged) |
| A7 | `tools/src/lib.rs` | exports: `approval` `:50`, `executor` `:68`, `shell` `:78`, `tool` `:83`; `env_safety` module `:38`. | 3,702 / (unchanged) |
| A8 | `core/src/tool_loop.rs` | `MAX_TOOL_TURNS` `:14`, `MAX_CALLS_PER_RUN` `:16`, `MAX_TOOL_RESULT_BYTES` `:18`; `plan_call` `:144`; `validate_plan` `:172`; `permission_for` `:185`; `requires_approval` `:193`; `tool_specs_for_tier` `:203`; `bounded_tool_text` `:236`. | 12,150 / `35668077…40ef2` |
| A9 | `core/src/execution.rs` | `RunEngine` struct `:260` (fields: store, db, bus, graph, contexts, tokens, executor, templates, caps, snapshots, `tool_workspace`, `tool_registry`, active_run — **no tools bridge, shell policy or waiter**); `with_tool_workspace` `:345`; `execute_tool_turn` `:684`; **`execute_tool_plan` `:879`** (refuses every non-Read tool `:880-892`); `publish_tool_call` `:923`; `finish_tool_call` `:934`; `execute_task` `:968`. | 41,920 / `160be09e…c728` |
| A10 | `core/src/tools_bridge.rs` | `ToolsBridge` `:35`; `expire_pending_approvals` `:96`; `approval_gate` `:126`; `resolve_approval_ledgered` `:217`; **`request_model_tool_approval` `:287`** (commits row + returns full payload); `approval_request_event` `:336`; `pending_approval_events` `:358`. | 33,063 / `df7d8f0c…efea4` |
| A11 | `contract/src/event.rs` | `ApprovalRequest` `:252` (approval_id, tool, decision?, nonce?, run_id?, task_id?, tool_call_id?, args_sha256?, expires_at?); `ToolOutcome` `:286` (`ok/refused/denied/timeout/cancelled`). | current |
| A12 | `core/src/run_state.rs` | approval effects `:205-267` (`RequestApproval`, `ResumeFromApproval`, `DenyApproval`, `FailOnApprovalTimeout`, task variants); `apply_run` `:465`, `apply_task` `:549`; illegal/replay edges fail closed. | current |
| A13 | `core/src/recovery.rs` | boot marks active runs interrupted; `awaiting_approval` gets `E_APPROVAL_TIMEOUT` (`:57-62`); never auto-executes. | current |

Live check: the current engine has **no** approval wiring (`execution.rs` grep shows only the read-only refusal text at
`:888`), so the blueprint below is additive and the read-only path stays untouched.

## 2. Design invariants (must hold in every code path)

1. **Grant before effect**: the single-use approval grant is verified **before** any filesystem write / child spawn.
   One `verify_grant` call per execution; it is the only authority (no "trusted caller" bypass).
2. **No grant consumption for a doomed call**: all read-only pre-checks (schema, tier, path containment, size, stale
   hash, program allowlist, cwd, argv bounds) run **before** `verify_grant`, so a call that cannot succeed never burns
   the operator's single-use approval.
3. **At-most-once**: a consumed AllowOnce grant can never execute twice (`executed_at` CAS); no retry loop inside the
   executor; a crash between consumption and effect leaves the run failed/interrupted at boot — never re-executed.
4. **Truthful results**: `Ok` only after the executor itself succeeded; shell non-zero exit is data in the bounded
   summary (never a fake success); refusal codes always come from the frozen 65.
5. **No body leakage**: argument content/argv/file bytes never appear in events, ledger, `tool_turns` or logs; only
   `args_sha256`, relative path, sizes, hashes, exit codes.
6. **Single owner per concern**: `executor.rs` stays read-only; the new `side_effects.rs` is the only granted executor;
   `jail.rs`/`shell.rs` own containment/process primitives; the engine owns suspension/wait; `ToolsBridge` owns the
   approval table/ledger seam; `run_state` owns legal transitions.

## 3. File/API layout (exact shapes)

### 3.1 New `backend/crates/hiverouter-tools/src/side_effects.rs`

```rust
//! Granted side-effect executor for the 1.2 tool loop (items 3 + 16).
//!
//! This is the ONLY module that may write files or spawn children on behalf of a
//! model-originated tool call. It requires a verified single-use approval grant:
//! every read-only pre-check runs before `verify_grant`, and exactly one tool is
//! executed after it. Nothing here retries, resolves approvals or publishes
//! events; the core engine owns suspension, events and persistence.

use std::fs;
use std::io::Write as _;
use std::path::PathBuf;

use hiverouter_contract::{Refusal, RunId};
use rusqlite::Connection;
use tokio_util::sync::CancellationToken;

use crate::approval::ApprovalGate;
use crate::error::ToolError;
use crate::executor::{execute_read_only, ExecutedTool};
use crate::jail::WorkspaceJail;
use crate::sha256::sha256_hex;
use crate::shell::{self, ShellOutput, ShellPolicy, ShellRequest};
use crate::tool::{
    AgentTier, FileWriteArgs, PermissionClass, ShellExecArgs, ToolId, ToolRegistry,
};

/// Maximum UTF-8 content a `file_write` may write (mirrors the 1 MiB read cap).
pub const MAX_WRITE_BYTES: usize = 1024 * 1024;
/// Hard cap on the returned tool-result text (mirrors core `MAX_TOOL_RESULT_BYTES`).
pub const MAX_SIDE_EFFECT_TEXT_BYTES: usize = 65_536;
/// Shell argv bounds enforced before the grant is consumed.
pub const MAX_SHELL_ARGS: usize = 64;
pub const MAX_SHELL_ARG_BYTES: usize = 4 * 1024;
pub const MAX_SHELL_ARGV_BYTES: usize = 16 * 1024;
/// Maximum program-name bytes accepted before the policy check.
pub const MAX_SHELL_PROGRAM_BYTES: usize = 260;

/// Everything the granted executor needs; assembled by the engine, borrowed only.
pub struct GrantedExecution<'a> {
    pub jail: &'a WorkspaceJail,
    pub registry: &'a ToolRegistry,
    pub shell: &'a ShellPolicy,
    pub tier: AgentTier,
    pub conn: &'a Connection,
    pub gate: &'a ApprovalGate,
    pub run_id: RunId,
    pub now_unix_secs: u64,
}

/// Fully pre-checked write; no effect has happened yet.
struct PreparedWrite {
    target: PathBuf,
    display_path: String,
    bytes: Vec<u8>,
    sha256: String,
}

/// Fully pre-checked shell invocation; no process has been spawned yet.
struct PreparedShell {
    program: String,
    args: Vec<String>,
    cwd: Option<String>,
}

/// The one granted entry point. Order is load-bearing (see §4).
pub async fn execute_approved(
    ctx: GrantedExecution<'_>,
    tool: &str,
    args: &serde_json::Value,
    cancellation: Option<&CancellationToken>,
) -> Result<ExecutedTool, ToolError> {
    // 1. Registry-first: unknown tool / tier not allowed / schema invalid refuse
    //    before any approval/DB/disk/process work.
    let contract = ctx.registry.resolve_call(ctx.tier, tool, args)?;
    if !matches!(
        contract.permission_class,
        PermissionClass::Mutate | PermissionClass::Execute
    ) {
        return Err(ToolError::new(
            Refusal::ToolNotAllowed,
            format!(
                "tool {:?} is not a granted side-effect tool; read-only calls use execute_read_only",
                tool
            ),
        ));
    }

    // 2. Read-only pre-checks (nothing effectful, no grant consumed yet).
    match contract.id {
        ToolId::FileWrite => {
            let prepared = preflight_file_write(ctx.jail, args)?;
            // 3. Grant verification is the last step before the effect.
            ctx.gate
                .verify_grant(ctx.conn, ctx.run_id, ToolId::FileWrite, args, ctx.now_unix_secs)?;
            commit_file_write(prepared)
        }
        ToolId::ShellExec => {
            let prepared = preflight_shell_exec(ctx.jail, ctx.shell, args, cancellation)?;
            ctx.gate
                .verify_grant(ctx.conn, ctx.run_id, ToolId::ShellExec, args, ctx.now_unix_secs)?;
            run_shell_exec(ctx.jail, ctx.shell, prepared, cancellation).await
        }
        _ => Err(ToolError::new(
            Refusal::ToolNotAllowed,
            format!("tool {:?} has no granted side-effect executor", tool),
        )),
    }
}
```

> `AgentTier`/`ToolContract` are imported from `crate::tool`, not the contract crate (contract has no
> `AgentTier`); the snippet's import line is corrected in the implementation.

### 3.2 `file_write` blueprint (`preflight_file_write` → `commit_file_write`)

```rust
fn preflight_file_write(
    jail: &WorkspaceJail,
    args: &serde_json::Value,
) -> Result<PreparedWrite, ToolError> {
    // Registry already validated the JSON schema; deserialize the typed args.
    let spec: FileWriteArgs = serde_json::from_value(args.clone()).map_err(|error| {
        ToolError::new(Refusal::SchemaInvalid, format!("file_write arguments: {error}"))
    })?;

    // B3 size bound before any I/O or grant consumption.
    if spec.content.len() > MAX_WRITE_BYTES {
        return Err(ToolError::new(
            Refusal::FileTooLarge,
            format!(
                "file_write content is {} bytes; the write cap is {MAX_WRITE_BYTES} bytes",
                spec.content.len()
            ),
        ));
    }
    if spec.path.trim().is_empty() {
        return Err(ToolError::new(Refusal::SchemaInvalid, "file_write path is empty"));
    }

    // B4 jail containment: the same six escape vectors as reads (jail.resolve).
    let target = jail.resolve(&spec.path)?;
    if target == jail.root() {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            "file_write cannot target the workspace root",
        ));
    }
    match fs::symlink_metadata(&target) {
        Ok(metadata) => {
            if metadata.is_dir() {
                return Err(ToolError::new(
                    Refusal::ToolSchema,
                    format!("file_write target {:?} is a directory", spec.path),
                ));
            }
            if !metadata.is_file() {
                return Err(ToolError::new(
                    Refusal::ToolSchema,
                    format!("file_write target {:?} is not a regular file", spec.path),
                ));
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    // B5 stale-write guard: 64 hex chars, current content hash must match.
    if let Some(expected) = spec.expected_sha256.as_deref() {
        if expected.len() != 64 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ToolError::new(
                Refusal::SchemaInvalid,
                "expected_sha256 must be 64 hexadecimal characters",
            ));
        }
        match fs::read(&target) {
            Ok(current) => {
                let actual = sha256_hex(&current);
                if !actual.eq_ignore_ascii_case(expected) {
                    return Err(ToolError::new(
                        Refusal::ToolSchema,
                        format!(
                            "stale write refused for {:?}: current sha256 {actual} does not match expected_sha256 {expected}; nothing written, approval not consumed",
                            spec.path
                        ),
                    ));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(ToolError::new(
                    Refusal::PathNotFound,
                    format!("expected_sha256 supplied but {:?} does not exist", spec.path),
                ));
            }
            Err(error) => return Err(error.into()),
        }
    }

    let bytes = spec.content.into_bytes();
    let sha256 = sha256_hex(&bytes);
    Ok(PreparedWrite {
        target,
        display_path: spec.path,
        bytes,
        sha256,
    })
}

fn commit_file_write(prepared: PreparedWrite) -> Result<ExecutedTool, ToolError> {
    let parent = prepared
        .target
        .parent()
        .ok_or_else(|| ToolError::new(Refusal::InternalError, "file_write target has no parent"))?;
    // Atomic replace: unique temp file in the SAME directory, create_new (no
    // symlink following), write+sync, then rename over the target (Windows
    // MoveFileEx replace semantics used by std::fs::rename).
    let temp = parent.join(format!(".hive-write-{}.tmp", uuid::Uuid::now_v7()));
    let guard = TempFileGuard { path: temp.clone() };
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)?;
    file.write_all(&prepared.bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(&temp, &prepared.target)?;
    std::mem::forget(guard); // rename succeeded: the temp path is gone

    let text = format!(
        "wrote {} bytes to {:?}; sha256={}",
        prepared.bytes.len(),
        prepared.display_path,
        prepared.sha256
    );
    Ok(ExecutedTool::Ok {
        text: bounded_text(&text),
    })
}

/// Removes the temp file if an error returns before the rename commits.
struct TempFileGuard {
    path: PathBuf,
}
impl Drop for TempFileGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
```

### 3.3 `shell_exec` blueprint (`preflight_shell_exec` → `run_shell_exec`)

```rust
fn preflight_shell_exec(
    jail: &WorkspaceJail,
    policy: &ShellPolicy,
    args: &serde_json::Value,
    cancellation: Option<&CancellationToken>,
) -> Result<PreparedShell, ToolError> {
    let spec: ShellExecArgs = serde_json::from_value(args.clone()).map_err(|error| {
        ToolError::new(Refusal::SchemaInvalid, format!("shell_exec arguments: {error}"))
    })?;
    if cancellation.is_some_and(CancellationToken::is_cancelled) {
        return Err(ToolError::new(Refusal::ToolCancelled, "shell_exec cancelled before preflight"));
    }
    // S3/S4 policy + bounds before the grant is consumed.
    if spec.program.trim().is_empty() || spec.program.len() > MAX_SHELL_PROGRAM_BYTES {
        return Err(ToolError::new(
            Refusal::SchemaInvalid,
            "shell_exec program is empty or longer than the program-name bound",
        ));
    }
    if !policy.allows(&spec.program) {
        return Err(ToolError::new(
            Refusal::ToolNotAllowed,
            format!(
                "program {:?} is not on the shell allowlist ({}); path-qualified programs are refused",
                spec.program,
                policy.allowed_programs().join(", ")
            ),
        ));
    }
    if spec.args.len() > MAX_SHELL_ARGS {
        return Err(ToolError::new(
            Refusal::SchemaInvalid,
            format!("shell_exec has {} arguments; the cap is {MAX_SHELL_ARGS}", spec.args.len()),
        ));
    }
    let mut argv_bytes = 0usize;
    for argument in &spec.args {
        if argument.contains('\0') {
            return Err(ToolError::new(
                Refusal::SchemaInvalid,
                "shell_exec argument contains a NUL byte and cannot be passed to a process",
            ));
        }
        if argument.len() > MAX_SHELL_ARG_BYTES {
            return Err(ToolError::new(
                Refusal::SchemaInvalid,
                "shell_exec argument exceeds the per-argument byte bound",
            ));
        }
        argv_bytes = argv_bytes.saturating_add(argument.len());
    }
    if argv_bytes > MAX_SHELL_ARGV_BYTES {
        return Err(ToolError::new(
            Refusal::SchemaInvalid,
            "shell_exec argv exceeds the total byte bound",
        ));
    }
    // cwd through the jail; None = root (same semantics as shell::execute).
    if let Some(cwd) = spec.cwd.as_deref() {
        let resolved = jail.resolve(cwd)?;
        if !resolved.is_dir() {
            return Err(ToolError::new(
                Refusal::ToolSchema,
                "shell_exec cwd is not a directory inside the jail",
            ));
        }
    }
    Ok(PreparedShell {
        program: spec.program,
        args: spec.args,
        cwd: spec.cwd,
    })
}

async fn run_shell_exec(
    jail: &WorkspaceJail,
    policy: &ShellPolicy,
    prepared: PreparedShell,
    cancellation: Option<&CancellationToken>,
) -> Result<ExecutedTool, ToolError> {
    let mut request = ShellRequest::new(prepared.program, prepared.args);
    if let Some(cwd) = prepared.cwd {
        request = request.with_cwd(cwd);
    }
    let output = shell::execute(jail, policy, &request, cancellation).await?;
    Ok(ExecutedTool::Ok {
        text: bounded_text(&shell_summary(&output)),
    })
}

fn shell_summary(output: &ShellOutput) -> String {
    // Bounded, secret-free summary; exit non-zero is data, not fake success.
    serde_json::json!({
        "program": output.program,
        "exit_code": output.exit_code,
        "success": output.success,
        "duration_ms": output.duration_ms,
        "truncated": output.truncated,
        "stdout": output.stdout,
        "stderr": output.stderr,
    })
    .to_string()
}

/// Char-boundary-safe truncation used for every side-effect result text.
fn bounded_text(text: &str) -> String {
    if text.len() <= MAX_SIDE_EFFECT_TEXT_BYTES {
        return text.to_owned();
    }
    let mut end = MAX_SIDE_EFFECT_TEXT_BYTES;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    format!(
        "{}\n[truncated at {MAX_SIDE_EFFECT_TEXT_BYTES} bytes]",
        &text[..end]
    )
}
```

### 3.4 `WorkspaceJail::write_atomic` (add to `jail.rs`; `write` `:223` stays for tests/internal)

```rust
/// Atomically replaces a workspace file: temp file in the same directory →
/// rename over the target. Refuses anything `resolve` refuses plus directories.
pub fn write_atomic(&self, requested: &str, bytes: &[u8]) -> Result<PathBuf, ToolError> {
    let target = self.resolve(requested)?;
    if target == self.root() {
        return Err(ToolError::new(Refusal::ToolSchema, "cannot atomically write the workspace root"));
    }
    let parent = target.parent().ok_or_else(|| {
        ToolError::new(Refusal::InternalError, "write target has no parent directory")
    })?;
    let temp = parent.join(format!(".hive-write-{}.tmp", uuid::Uuid::now_v7()));
    // Same-directory temp keeps the rename on one volume; create_new avoids
    // following an attacker-planted symlink; guard removes stale temp on error.
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&temp)?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp, &target)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result.map(|()| target)
}
```
(The module already imports `std::fs`; add `std::io::Write` there. `uuid` is already a tools dependency.)

## 4. Grant verification + one-shot semantics (exact path)

`execute_approved` step order (all failure codes from the frozen table):

```
registry.resolve_call(tier, tool, args)            -> E_TOOL_NOT_FOUND / E_TOOL_NOT_ALLOWED / E_TOOL_SCHEMA
permission_class in {Mutate, Execute}              -> E_TOOL_NOT_ALLOWED (Read tools must not reach this module)
preflight_* (schema/path/size/stale/allowlist/cwd/argv/cancel) -> ToolError (no effect, no grant consumed)
ApprovalGate::verify_grant(conn, run_id, ToolId, args, now) -> E_APPROVAL_REQUIRED / E_APPROVAL_TIMEOUT / E_APPROVAL_REPLAY
execute exactly one tool                           -> success / E_TOOL_TIMEOUT / E_TOOL_CANCELLED / E_INTERNAL_ERROR
```

- `verify_grant` is `approval.rs:694`: re-hashes the **same** `args` value with `hash_args` (`:132`), accepts a
  scoped AllowSession grant (`session_allowed` `:384`), else selects an `allow_once` row for
  `(run_id, tool_name, args_sha256)` and performs `UPDATE approvals SET executed_at=… WHERE approval_id=? AND
  executed_at IS NULL`. `changed == 1` ⇒ the one execution may proceed; `Some(executed_at)` or a lost CAS ⇒
  `E_APPROVAL_REPLAY`; no row ⇒ `E_APPROVAL_REQUIRED`.
- The wire is `approve|deny` only (`ApprovalDecision::from_wire`), so a model-loop approval is an **AllowOnce** grant:
  exactly one execution. AllowSession remains internal/programmatic and cannot be created through the frozen command.
- Engine-side one-shot: one `execute_approved` call per planned call; the engine never retries a consumed grant. If the
  process dies between `verify_grant` and the effect, boot recovery (`recovery.rs:57-62`) already marks an
  `awaiting_approval` run interrupted with `E_APPROVAL_TIMEOUT` and never re-executes.
- Timeout after spawn: `shell::execute` returns `E_TOOL_TIMEOUT` **after** the child was started; the grant is already
  consumed and must stay consumed (no retry). The tool result is `Timeout`, and the run fails closed by the engine.

## 5. Engine wiring blueprint (item 16; `core/src/`)

### 5.1 New `core/src/approval_wait.rs`

```rust
//! Wake seats for the approval suspension seam (no DB authority, no events).
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::oneshot;
use hiverouter_contract::RunId;

#[derive(Default)]
pub struct ApprovalBroker {
    waiters: Mutex<HashMap<(RunId, String), oneshot::Sender<()>>>,
}
impl ApprovalBroker {
    pub fn new() -> Self { Self::default() }
    /// Registers the single waiter for one (run, tool_call) decision.
    pub fn register(&self, run_id: RunId, tool_call_id: &str) -> oneshot::Receiver<()> { … }
    /// Wakes exactly one waiter; false when the wait already ended (crash/duplicate).
    pub fn notify(&self, run_id: RunId, tool_call_id: &str) -> bool { … }
    /// Drops every waiter of a run on cancel/kill and at run finish.
    pub fn drop_run(&self, run_id: RunId) { … }
}
```

### 5.2 `RunEngine` additions (`execution.rs:260-347`)

```rust
// fields (private, after tool_registry)
tools: Option<Arc<crate::tools_bridge::ToolsBridge>>, // approval producer/resolver/DB seam
shell_policy: hiverouter_tools::ShellPolicy,
side_effects_enabled: bool,
approval_broker: Arc<crate::approval_wait::ApprovalBroker>,

// builders, next to with_tool_workspace(:345)
pub fn with_tools_bridge(mut self, tools: Arc<ToolsBridge>) -> Self { … }
pub fn with_shell_policy(mut self, policy: ShellPolicy) -> Self { … }
pub fn with_side_effects_enabled(mut self, enabled: bool) -> Self { … }
pub fn approval_broker(&self) -> Arc<ApprovalBroker> { Arc::clone(&self.approval_broker) }
```

### 5.3 `execute_tool_plan` / `execute_tool_turn` become async; non-Read branch blueprint

```rust
async fn execute_tool_plan(&self, run: &Run, task: &TaskRecord, plan: &ToolCallPlan,
                           token: &CancellationToken) -> ToolExecution {
    let permission = tool_loop::permission_for(&self.tool_registry, &plan.tool);
    if permission != Some(PermissionClass::Read) {
        return self.execute_side_effect_plan(run, task, plan, token).await;
    }
    /* existing read-only body unchanged (execution.rs:879-920) */
}

async fn execute_side_effect_plan(&self, run: &Run, task: &TaskRecord,
                                  plan: &ToolCallPlan, token: &CancellationToken)
                                  -> ToolExecution {
    if !self.side_effects_enabled { return refused(E_TOOL_NOT_ALLOWED, "lane disabled"); }
    let Some(tools) = self.tools.as_ref() else { return refused(E_TOOL_NOT_ALLOWED, "no approval seam"); };
    let Some(workspace) = self.tool_workspace.as_ref() else { return refused(E_FILE_TREE_UNAVAILABLE, …); };
    let Some(tool) = tool_loop::… // registry contract id already validated by validate_plan

    // 1) run_state: Running -> AwaitingApproval (Effect::RequestApproval)
    //    apply_run returns the typed effect; persist statuses exactly once.
    // 2) bridge.request_model_tool_approval(run.id, task.id, &plan.tool_call_id, &plan.tool, &plan.args)
    //    -> event::ApprovalRequest (all six fields populated; row already committed)
    // 3) publish Event::ApprovalRequest(payload) via bus.publish_for_run(..., Some(run.id), None);
    //    persist run+task awaiting_approval (store.set_run_status / update_task_state)
    // 4) let rx = self.approval_broker.register(run.id, &plan.tool_call_id);
    //    let expires = payload.expires_at…; tokio::select! {
    //        _ = rx => wake, _ = sleep_until(deadline) => Timeout, _ = token.cancelled() => Cancel }
    // 5) after wake: read the committed decision (new ToolsBridge::approval_decision(run.id, &tool_call_id))
    //      Approved => apply_run Resume; tools.execute_approved(ctx, &plan.tool, &plan.args, Some(token)).await
    //      Denied   => ToolExecution::denied(plan.tool_call_id.clone())
    //      None     => refused(E_APPROVAL_REQUIRED)  // fail closed, never auto-approve
    // 6) timeout => expire the row (existing expire_pending_approvals) + run_state Timeout +
    //      ToolExecution::timed_out; cancel => broker.drop_run + ToolExecution::cancelled
    // 7) map ExecutedTool/ToolError -> ToolExecution (ok / refused(code, detail)); finish_tool_call persists
}
```

`ToolsBridge` additions (exact new helpers, same file, after `request_model_tool_approval` `:287`):

```rust
/// The committed decision for one (run, tool_call) — read-only, no consumption.
pub fn approval_decision(&self, run_id: RunId, tool_call_id: &str)
    -> Result<Option<hiverouter_tools::approval::ApprovalDecision>, Refusal>;

/// Wakes the engine waiter for one (run, tool_call); called by the ws command path.
pub fn notify_approval_waiters(&self, broker: &crate::approval_wait::ApprovalBroker,
                               resolution: &ApprovalResolution) -> bool;

/// Verifies the single-use grant through the one gate (no execution).
pub fn verify_tool_grant(&self, run_id: RunId, tool: hiverouter_tools::ToolId,
                         args: &serde_json::Value) -> Result<(), Refusal>;
```

`ws.rs` (`ApprovalResolve` arm): after `resolve_approval_ledgered` succeeds:
`state.engine.notify_approval_resolved(&resolution)` — this **only wakes** the engine waiter
(`broker.notify(resolution.request.run_id, &resolution.request.tool_call_id)`); `ws.rs` keeps publishing ACK +
`APPROVAL_RESOLVED` exactly as it does now (no duplicate event). Duplicate resolution/notify without a waiter ⇒ gate
refuses before ledger; notify returns false; no effect.

**Signature changes to existing code (explicit):** `RunEngine::execute_tool_turn` (`execution.rs:684`) and
`execute_tool_plan` (`:879`) become `async fn`; their call sites in `execute_task` (`:968`, the follow-up loop around
`:1014`) `await`s them. `run_lifecycle`/`tool_loop_engine` test doubles are unaffected except that any `StreamedReply`
constructors must set `tool_calls: Vec::new()` when they do not exercise the loop.

Config/boot (`config.rs`, `http.rs`): `HIVEROUTER_SIDE_EFFECTS` (default `0`), `HIVEROUTER_SHELL_ALLOWLIST`
(comma-separated, validated bare names; default `ShellPolicy::conservative_default()` minus `cmd.exe` per §11 Q1
recommendation), `HIVEROUTER_SHELL_TIMEOUT_SECS` (default 60, hard cap 300). Engine construction:
`engine.with_tools_bridge(tools.clone()).with_shell_policy(policy).with_side_effects_enabled(flag)`.

## 6. Refusal mapping (frozen 65 only; no new codes)

| Stage / condition | Code | Produced by |
|---|---|---|
| Unknown tool / tier not allowed / schema invalid | `E_TOOL_NOT_FOUND` / `E_TOOL_NOT_ALLOWED` / `E_TOOL_SCHEMA` | `registry.resolve_call` (`tool.rs:446`) |
| Read-class tool reaching granted executor | `E_TOOL_NOT_ALLOWED` | `execute_approved` step 1 |
| No grant / grant denied / expired / args mismatch | `E_APPROVAL_REQUIRED` | `verify_grant` (`approval.rs:694`) |
| Once-only grant already consumed (second execution) | `E_APPROVAL_REPLAY` | `verify_grant` CAS |
| Approval waited past `expires_at` | `E_APPROVAL_TIMEOUT` | engine timeout branch + `run_state` `FailOnApprovalTimeout` |
| Write: content > 1 MiB | `E_FILE_TOO_LARGE` | `preflight_file_write` |
| Write: directory target / stale sha mismatch / bad expected hex / empty path / root target | `E_TOOL_SCHEMA` (+ `E_SCHEMA_INVALID` for deserialize/empty) | preflight |
| Write: expected hash but file missing | `E_PATH_NOT_FOUND` | preflight |
| Path escape (six vectors, symlink/junction, ADS, short-name) | `E_PATH_OUTSIDE_JAIL` | `jail.resolve` (`jail.rs:92`) |
| Shell: program outside allowlist / separators | `E_TOOL_NOT_ALLOWED` | preflight + `shell::execute` |
| Shell: cwd outside naught jail / not a directory | `E_PATH_OUTSIDE_JAIL` / `E_TOOL_SCHEMA` | preflight + `shell::execute` |
| Shell: NUL/oversize program or argv, args > 64 | `E_SCHEMA_INVALID` | preflight |
| Shell: timeout / cancelled | `E_TOOL_TIMEOUT` / `E_TOOL_CANCELLED` | `shell::execute` (`shell.rs:173`) |
| I/O, SQLite, temp-file write/rename failure | `E_INTERNAL_ERROR` (DB: `E_DB_BUSY`) | preflight/commit / bridge |
| Operator deny | no code — `ToolOutcome::Denied` + text `"operator denied; nothing executed"` | engine |
| Lane disabled / no workspace / no approval seam | `E_TOOL_NOT_ALLOWED` / `E_FILE_TREE_UNAVAILABLE` | engine refusal (today's behavior) |

## 7. Bounds, timeouts, atomicity summary

| Bound | Constant / owner | Enforced in |
|---|---|---|
| Write content ≤ 1 MiB (UTF-8 bytes) | `side_effects::MAX_WRITE_BYTES` | `preflight_file_write` before grant |
| Write result ≤ 65,536 bytes | `MAX_SIDE_EFFECT_TEXT_BYTES` + core `bounded_tool_text` (`tool_loop.rs:236`) | commit + engine |
| Shell program ≤ 260 bytes; args ≤ 64; arg ≤ 4 KiB; argv ≤ 16 KiB | `MAX_SHELL_*` | `preflight_shell_exec` before grant |
| Shell wall clock 60 s default / 300 s cap | `ShellPolicy` (`shell.rs:30`) + config cap | `shell::execute` |
| Shell stdout/stderr caps 1 MiB / 256 KiB | `DEFAULT_MAX_*` (`shell.rs:26-28`) | `read_capped` in `shell::execute` |
| Approval TTL 120 s | `APPROVAL_TTL_SECS` (`approval.rs:26`) | row `expires_at`; engine deadline + boot reaper |
| Tool turns 4 / calls 16 / result 65,536 | `tool_loop.rs:14-18` | `execute_tool_turn` (unchanged) |
| Atomic write | temp `.hive-write-<uuid>.tmp` + `rename` | `commit_file_write` / `jail.write_atomic` |

## 8. Required tests (implementation wave; deterministic, offline)

Tools-level (in-memory SQLite via `approval::migrate` + `tempfile::TempDir`; no network/pay):
1. `grant_required_before_any_effect` — file_write & shell_exec with no grant ⇒ `E_APPROVAL_REQUIRED`; temp dir
   unchanged; no child spawned.
2. `doomed_call_does_not_consume_grant` — escape path / directory target / >1 MiB / stale hash / disallowed program,
   each with a valid AllowOnce grant ⇒ named refusal; `executed_at IS NULL` (grant still usable); then the fixed call
   succeeds once.
3. `file_write_atomic_replace` — writes/overwrites inside jail; content sha matches; no `.hive-write-*.tmp` remains;
   read-back equals content; result summary has path+bytes+sha, **does not contain content**.
4. `file_write_stale_sha256` — matching expected accepted; mismatch/missing file/malformed hex refused with code table
   §6; file unchanged.
5. `file_write_once_only` — second execution of the same AllowOnce grant ⇒ `E_APPROVAL_REPLAY`; content unchanged.
6. `shell_exec_happy_path` — allowlisted program under grant; bounded JSON summary; exit code/success truthful;
   zero-exit and non-zero-exit cases; output capped.
7. `shell_exec_preflight_refusals` — non-allowlisted program, path-qualified program, bad cwd, NUL arg, >64 args,
   >16 KiB argv ⇒ codes before grant; `executed_at` null.
8. `shell_exec_timeout_cancel` — timeout ⇒ `E_TOOL_TIMEOUT`; cancellation token ⇒ `E_TOOL_CANCELLED`;
   child env cannot see `DEEPSEEK_API_KEY` (extend existing canary test).
9. `read_only_tools_rejected_by_granted_executor` — `file_read`/`file_list` ⇒ `E_TOOL_NOT_ALLOWED`.
10. `result_text_is_bounded_on_char_boundary` — truncation marker, valid UTF-8, ≤ cap.

Core/engine-level:
11. `engine_suspends_for_approval` — publish `TOOL_CALL` then `APPROVAL_REQUEST` (six fields), persist
    `awaiting_approval`, decision wakes waiter, `APPROVAL_RESOLVED` then `TOOL_RESULT`, `tool_turns` row.
12. `engine_deny_timeout_replay` — Denied outcome; timeout ⇒ `E_APPROVAL_TIMEOUT` (run failed via `run_state`); duplicate
    resolution ⇒ `E_APPROVAL_REPLAY`, no second execution.
13. `lane_flag_off_preserves_read_only_1_2` — refusal `E_TOOL_NOT_ALLOWED`, no approval row.
14. `boot_recovery_fails_awaiting_approval` — persisted `awaiting_approval` + restart ⇒ interrupted with
    `E_APPROVAL_TIMEOUT`; row expired; nothing executed.
15. Existing suites stay green: `approval.rs` 14 tests, `jail.rs` escape matrix, `shell.rs` env/timeout/caps,
    `executor.rs` read-only policy, `tool_loop_engine.rs` read-only loop; `run_lifecycle` test doubles must set
    `StreamedReply.tool_calls = Vec::new()`.

## 9. Security and non-claims (must be repeated in the implementation PR/runbook)

**Security rules implemented by this blueprint**
- Single-use, TTL-bound, args-hash-bound consent is the only side-effect authority; no bypass, default-off lane.
- Pre-checks before grant consumption: no wasted/consumed consent for invalid calls; jail/allowlist/bounds fail closed.
- Atomic replace (temp+rename, `create_new`) prevents partial-file success and symlink-preplant attacks; jail resolves
  the same six escape vectors as reads.
- Shell: bare allowlisted program only, direct argv (no shell interpolation), jailed cwd, explicit env allowlist
  (`env_safety`, `DEEPSEEK_API_KEY` excluded), stdin null, kill-on-close Job Object, timeout and output caps,
  cancellation kills the child; non-zero exit is truthful data.
- No argument bodies/argv/file content in events, ledger, `tool_turns` or logs — only `args_sha256`, paths, sizes,
  hashes, exit codes. No key material anywhere in this path.
- At-most-once: `executed_at` CAS + no retry + boot fail-closed.

**Explicit non-claims (what this will NOT do)**
- **No OS sandbox / container**: the shell child runs as the server user. Jail constrains cwd and `file_write` paths,
  not the child's other filesystem or network access. Approval is the only gate.
- **`cmd.exe` is a general shell**: if left allowlisted, `cmd /C …` can run arbitrary commands; recommendation: remove
  `cmd.exe` from the model-lane default allowlist or accept it explicitly (Lead/product decision, Q1).
- `git.exe`/`cargo.exe` can execute repository-controlled code (hooks, `cargo run`); approval UI currently sees only
  `tool` + `args_sha256` (no argv preview) — an argument-content preview needs a contract payload addition (out of scope).
- No elevation/admin, no PTY/interactive stdin, no long-running daemons, no binary writes, no directory deletion,
  no network isolation, no concurrent-write merge, no undo of completed effects.
- No exactly-once across a crash: grant is consumed before the effect (at-most-once); a crash can leave an unknown
  outcome, recorded as failed/interrupted, never retried.

## 10. Rollback

Everything is additive behind `HIVEROUTER_SIDE_EFFECTS=0` (default): revert the wave commit and restart; read-only
1.2 behavior is unchanged. No schema migration (approvals/tool_turns already exist); pending `awaiting_approval` rows
are expired at boot and runs fail closed with `E_APPROVAL_TIMEOUT`. No release/tag changes in the implementation wave.

## 11. Open questions for the implementation/Lead gate

1. **Q1 (must decide):** remove `cmd.exe` from the model-lane shell default allowlist? (recommended: yes; operator can
   opt in explicitly). Evidence: `shell.rs:66-72`, plan §12.
2. **Q2:** approval UI argument preview: contract payload has no argv/content field; is hash+tool sufficient for
   operator consent, or is a redacted preview a required contract addition (W12+)?
3. **Q3:** confirm `expected_sha256` mismatch maps to `E_TOOL_SCHEMA` (no grant consumed) rather than
   `E_APPROVAL_REQUIRED`.
4. **Q4:** timeout/hard cap 60 s/300 s acceptable; per-run wall-clock budget is not yet a first-class cap.
5. **Q5:** crash-between-grant-and-effect: accept at-most-once + interrupted run (this blueprint) as the honest answer.

## 12. Logic-gatekeeper Phase 1 (for the implementing wave)

```json
{"phase1": {
  "architecture": "tools/src/side_effects.rs will own the only granted write/spawn path; jail.rs owns path containment and atomic replace; shell.rs owns process containment/allowlist/env/caps; approval.rs owns the single-use grant; tools_bridge.rs owns the approval table/ledger seam; execution.rs owns suspension/wait/events; run_state.rs owns legal transitions; config/http wire a default-off flag and shell policy.",
  "dataFlow": "assembled tool call -> engine plan/validate -> (Mutate/Execute) Suspend effect -> bridge request (row: nonce/args_sha256/expires_at) -> publish TOOL_CALL + APPROVAL_REQUEST -> persisted awaiting_approval -> waiter <- ws APPROVAL_RESOLVE(nonce) -> gate decision+ledger -> notify -> decision read -> execute_approved: registry -> read-only preflight -> verify_grant (AllowOnce executed_at CAS) -> atomic file_write | allowlisted shell_exec -> bounded text -> TOOL_RESULT + tool_turns; deny/timeout/cancel/replay are named outcomes with zero side effects; boot recovery fails closed.",
  "edgeCases": [
    "no grant / denied / expired / wrong args => E_APPROVAL_REQUIRED, zero effects",
    "doomed call (escape/size/stale/disallowed program) => refusal before verify_grant, executed_at still NULL",
    "second execution of the same AllowOnce grant => E_APPROVAL_REPLAY, no second effect",
    "crash after verify_grant before effect => boot marks run interrupted E_APPROVAL_TIMEOUT, never retried",
    "shell timeout after spawn => grant stays consumed, ToolOutcome::Timeout, no fake success; children killed by Job Object",
    "deny and duplicate resolution => Denied outcome / E_APPROVAL_REPLAY, nothing executed",
    "lane disabled or no workspace/bridge => E_TOOL_NOT_ALLOWED / E_FILE_TREE_UNAVAILABLE, preserving read-only behavior"
  ],
  "rootCause": "The grant/approval/run-state machinery exists but the loop has no bridge to it: engine refuses Mutate/Execute tools while the read-only executor has no write/spawn authority. The fix is a strict ordered chain with read-only pre-checks before grant verification and exactly one effect after it; any reordering either burns consent on invalid calls or permits unreviewed side effects."
}}
```

**Read-only statement:** source was read, this document was written; no product file was edited, no `cargo build`/`test`
or Playwright was run, no key was accessed, no commit was made.
