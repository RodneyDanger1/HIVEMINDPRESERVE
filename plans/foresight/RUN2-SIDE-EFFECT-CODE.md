# RUN-2 CODE BLUEPRINT — WP-02 side-effect executor, `file_write` first (shell deferred)

**Planner:** Run-2 Planner 1 (`/research`, `/forgewright-lead`, `/forgewright-plan`, `/deep-naming`, `/truthseeker`, `/scientific-thinking`, `/forge`, `/evidence-check`)
**Mode:** READ-ONLY planning. This file is the only write. **No product edit, no build, no test run, no commit, no key access.**
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1`
**Stated HEAD:** `90cb7a7` `feat(1.2): reasoning passback, approval boot expiry, recovery timeout; foresight run-1 plans/ledger` (clean at Lead's statement).
**Observed tree at 18:51 CT:** HEAD `90cb7a7`, **dirty** — a concurrent WP-01 lane is already implementing approval primitives: `M core/src/tools_bridge.rs` (+76 lines), `M tools/src/approval.rs` (+73: `expire`, `cancel_pending`, `decision_label`, `count_for_run`), `M tools/src/lib.rs` (approval re-exports), `?? tools/qa/apply_wp01.py`. All runtime statements in this blueprint are `UNVERIFIED`; the dirty tree is `READ [WIP]`.
**Inputs read:** `plans/foresight/COMPILED-LEDGER.md` (WP-01/02 + C1/C2/C4/C11/C13), `plans/foresight/CODE-SIDE-EFFECTS.md` (729 lines, full), `plans/foresight/RUN-1.md`, and the current `tools/src/{approval,executor,jail,shell,tool}.rs` + `lib.rs` + `Cargo.toml` + `error.rs` + `core/src/tools_bridge.rs`.

**Source labels:** `[READ file:lines @HEAD]` = committed HEAD 90cb7a7; `[WIP]` = uncommitted lane; `[R2 §n]` = `CODE-SIDE-EFFECTS.md`; `[LEDGER …]` = compiled ledger; `[INFERRED]`; `[UNKNOWN]`/`[UNVERIFIED]`.

---

## 0. Scope and the two design reconciliations (Lead decisions flagged)

**WP-02 scope in this blueprint = `file_write` only.**
- New file `backend/crates/hiverouter-tools/src/side_effects.rs`: registry/permission check → read-only preflight (schema, size, containment, target kind, stale hash) → *grant is verified by the caller between preflight and commit* → commit (atomic write) → bounded result text.
- `backend/crates/hiverouter-tools/src/jail.rs`: add `WorkspaceJail::write_atomic` (+ tests). `write` `[READ jail.rs:223]` stays untouched for existing callers/tests.
- `backend/crates/hiverouter-tools/src/lib.rs`: add `pub mod side_effects;` + re-exports (**rebase behind the live WP-01 lane**, §8).
- **No edits** to `approval.rs`, `tools_bridge.rs`, `executor.rs`, `shell.rs`, `tool.rs`, core, contract, db, web. Shell is designed as a later additive arm (§4.5) but not implemented here: `shell_exec` is refused by name with zero effect and no grant consumed.

**C1 reconciliation (this is a change from R2's monolithic `execute_approved`; Lead must accept).**
R2 §3.1 proposed one `execute_approved(GrantedExecution{conn,gate,…}, …)` that does preflight + `verify_grant` + effect inside `tools`. The **live WP-01 lane already provides the grant authority in core**: `ToolsBridge::verify_tool_grant(&self, run_id: RunId, tool: hiverouter_tools::ToolId, args: &serde_json::Value, now_unix_secs: u64) -> Result<(), Refusal>` `[WIP core/src/tools_bridge.rs:166-181]`, and the ledger's C1 recommends "adopt R1 waiter + R2 module/ordering; engine calls one bridge delegate/executor function; keep preflight-before-grant; remove engine-side duplicate verify". Therefore WP-02 ships **two pure halves** and lets the engine call the bridge grant check between them:

```
preflight_side_effect(...)   // tools, sync, read-only, NO grant consumption
   → ToolsBridge::verify_tool_grant(...)   // core, sync DB CAS, consumes the AllowOnce grant
      → commit_side_effect(...).await      // tools, exactly one effect, async for future shell
```

Benefits: the tools crate never needs `&Connection`/`&ApprovalGate` (no `rusqlite` borrow across `.await` problem — `Database::with_connection` closures are synchronous, so R2's async `execute_approved(&Connection)` could not be called from core anyway); exactly one grant authority (live bridge); shell later only extends the prepared enum + commit match; engine call site unchanged when shell lands.

**C2 reconciliation (rollout flag).** WP-02 contains **no feature flag and no caller**: it is a library capability that is unreachable until WP-03 wires the engine. The default-off `HIVEROUTER_SIDE_EFFECTS` decision `[LEDGER C2]` belongs to WP-03/http config, not to this module. Shipping WP-02 alone is a behavior-neutral, compile-only addition; do not claim runtime reachability from WP-02.

**Security/ordering invariant (load-bearing, from `[R2 §2]`):** all read-only pre-checks run **before** `verify_tool_grant`; exactly one grant consumption per planned call; exactly one effect after it; a doomed call must never burn consent. `verify_tool_grant` is called by the engine **immediately** before `commit_side_effect`, never in `preflight` and never twice.

---

## 1. Exact current anchors (HEAD 90cb7a7; live-tree offsets noted)

| File | Fact | HEAD anchor | Live-tree note |
|---|---|---|---|
| `tools/src/jail.rs` | imports `std::fs`, `std::path`; no `std::io::Write`; no `uuid` | `:19-25` | unchanged (`git diff` excludes it) |
| | `MAX_READ_BYTES: u64 = 1024*1024` | `:32` | unchanged |
| | `WorkspaceJail { root: PathBuf }`, `open`, `root`, `resolve`, `verify_containment`, `is_inside`, `read`, `write`, `list` | `:36`, `:55`, `:80`, `:92`, `:143`, `:170`, `:185`, `:223`, `:250` | unchanged |
| | `escape(detail)` private helper | `:298` | unchanged |
| | tests module (`reads_writes…`, six escape vectors, read cap, write) | `:302-437` | unchanged |
| `tools/src/executor.rs` | `ExecutedTool::{Ok{text},Refused{code,detail}}` | `:25` | unchanged |
| | `execute_read_only(jail, registry, tier, tool, args)`; read-only policy refusal | `:52`, `:59-67` | unchanged |
| `tools/src/approval.rs` | `APPROVAL_TTL_SECS`; `hash_args`; DDL `migrate` (UNIQUE nonce; no unique pending index) | `:26`, `:132`, `:141-202` | live `expire/cancel_pending/decision_label/count_for_run` added after `resolve_with_reason`; +73 lines |
| | `ApprovalGate`; `verify_grant` (AllowSession; AllowOnce `executed_at IS NULL` CAS; absent/denied/expired ⇒ `E_APPROVAL_REQUIRED`; executed ⇒ `E_APPROVAL_REPLAY`) | `:583-905` (`verify_grant :772`) | live `verify_grant` at `:843` |
| | `request_tool_approval` | `:849` | live `:920` |
| `tools/src/tool.rs` | `PermissionClass` (`requires_approval` = not Read) | `:30`, `:55` | unchanged |
| | `ToolId` (SpawnAgent, ReportResult, Finish, FileRead, FileList, FileWrite, ShellExec) | `:108-151` | unchanged |
| | `FileWriteArgs { path, content, expected_sha256? }` | `:255-263` | unchanged |
| | `ShellExecArgs { program, args, cwd? }` | `:268-276` | unchanged |
| | `ToolRegistry::standard` (WorkerAgent incl. FileWrite/ShellExec); `resolve_call` | `:295`, `:370-380`, `:446` | unchanged |
| `tools/src/shell.rs` | `ShellPolicy`, `conservative_default`, `allows`, `ShellRequest`, `ShellOutput`, `pub async fn execute` | `:34`, `:66`, `:96`, `:114`, `:145`, `:173` | unchanged |
| `tools/src/lib.rs` | modules; `pub use executor::{execute_read_only, ExecutedTool}`; `pub use shell::…` | `:31-48`, `:68`, `:78-81` | live tree already edits `pub use approval::{…}` (+6/-4) — **merge conflict point** |
| `tools/src/error.rs` | `ToolError::new(refusal, detail)`, `refusal()`, `code()`, `detail()` | `:22`, `:31`, `:37`, `:43` | unchanged |
| `tools/Cargo.toml` | deps include `uuid`, `rusqlite`, `tokio-util`; dev-deps `tempfile`, tokio | file | no new dependency needed |
| `core/src/tools_bridge.rs` | live `verify_tool_grant(run_id, ToolId, args, now) -> Result<(), Refusal>`; live `expire_approval/cancel_approval/approval_decision/approval_count_for_run/with_approval_ttl` | `[WIP :101-181]` | **WP-01 lane active** |
| absent | `write_atomic`, `side_effects` module, `execute_approved`, `GrantedExecution`, `bounded_text` | grep 0 | absent in HEAD and current tree (`side_effects` grep hits are test/fn names only) |

---

## 2. New module `tools/src/side_effects.rs` (exact shape, file_write wave)

### 2.1 Module header, imports, constants

```rust
//! Granted side-effect executor for the 1.2 tool loop (WP-02).
//!
//! This is the ONLY module that may write files or (later) spawn children for a
//! model-originated tool call. It performs **read-only preflight** only; the
//! single-use approval grant is verified by the core bridge *between* preflight
//! and commit (`ToolsBridge::verify_tool_grant`), so a doomed call never burns
//! consent. Nothing here resolves approvals, reads the approval table, retries,
//! or publishes events; the core engine owns suspension/events/persistence.
//!
//! File-write wave: `shell_exec` and every other tool are refused by name; shell
//! preflight/commit are added in the next wave by extending `PreparedSideEffect`
//! and `commit_side_effect` only (no public signature change).

use std::fmt;
use std::fs;

use hiverouter_contract::Refusal;
use tokio_util::sync::CancellationToken;

use crate::error::ToolError;
use crate::executor::ExecutedTool;
use crate::jail::WorkspaceJail;
use crate::sha256::sha256_hex;
use crate::tool::{AgentTier, FileWriteArgs, PermissionClass, ToolId, ToolRegistry};

/// Maximum UTF-8 content a `file_write` may write (mirrors the 1 MiB read cap).
pub const MAX_WRITE_BYTES: usize = 1024 * 1024;
/// Hard cap on the returned tool-result text (mirrors core `MAX_TOOL_RESULT_BYTES`).
pub const MAX_SIDE_EFFECT_TEXT_BYTES: usize = 65_536;
```

### 2.2 Opaque prepared effect (no content leak through `Debug`)

```rust
/// Fully pre-checked file write; no effect has happened yet.
///
/// Fields are private: the engine receives the value from
/// `preflight_side_effect` and must hand it back to `commit_side_effect` after
/// the grant is verified. It cannot inspect or construct a prepared write, so it
/// cannot bypass the preflight checks.
pub struct PreparedWrite {
    target: std::path::PathBuf,
    display_path: String,
    bytes: Vec<u8>,
    sha256: String,
}

impl fmt::Debug for PreparedWrite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Never print the content: only path/size/hash are non-secret.
        f.debug_struct("PreparedWrite")
            .field("display_path", &self.display_path)
            .field("bytes", &self.bytes.len())
            .field("sha256", &self.sha256)
            .finish()
    }
}

/// The one preflighted side effect the engine may commit after granting it.
#[derive(Debug)]
pub enum PreparedSideEffect {
    /// A checked atomic file replacement.
    FileWrite(PreparedWrite),
    // ShellExec(PreparedShell) is added in the shell wave; adding a variant is
    // additive for the engine because it only passes the value back.
}
```

### 2.3 Entry points (exact signatures)

```rust
/// Read-only preflight for one model-originated side-effect tool call.
///
/// Order: registry lookup/tier/schema -> permission class -> tool-specific
/// read-only checks -> (return). Never reads the approvals table, never consumes
/// a grant, never writes or spawns.
///
/// # Errors
/// Frozen codes only: `E_TOOL_NOT_FOUND`, `E_TOOL_NOT_ALLOWED`, `E_TOOL_SCHEMA`,
/// `E_SCHEMA_INVALID`, `E_FILE_TOO_LARGE`, `E_PATH_OUTSIDE_JAIL`,
/// `E_PATH_NOT_FOUND`, `E_TOOL_CANCELLED`, `E_INTERNAL_ERROR`.
pub fn preflight_side_effect(
    jail: &WorkspaceJail,
    registry: &ToolRegistry,
    tier: AgentTier,
    tool: &str,
    args: &serde_json::Value,
    cancellation: Option<&CancellationToken>,
) -> Result<PreparedSideEffect, ToolError> {
    if cancellation.is_some_and(CancellationToken::is_cancelled) {
        return Err(ToolError::new(
            Refusal::ToolCancelled,
            "side effect cancelled before preflight; nothing executed and no approval was consumed",
        ));
    }
    // Registry first: unknown tool / tier not allowed / invalid JSON args never
    // reach the filesystem and never touch the grant.
    let contract = registry.resolve_call(tier, tool, args)?;
    match contract.id {
        ToolId::FileWrite => {
            if contract.permission_class != PermissionClass::Mutate {
                return Err(ToolError::new(
                    Refusal::ToolNotAllowed,
                    "file_write registry permission class is not Mutate; refusing before any effect",
                ));
            }
            Ok(PreparedSideEffect::FileWrite(preflight_file_write(jail, args)?))
        }
        ToolId::ShellExec => Err(ToolError::new(
            Refusal::ToolNotAllowed,
            "shell_exec granted executor is not enabled in this wave (file_write only); \
             nothing executed and no approval was consumed",
        )),
        _ => Err(ToolError::new(
            Refusal::ToolNotAllowed,
            format!(
                "tool {tool:?} is not a granted side-effect tool; \
                 read-only calls use execute_read_only"
            ),
        )),
    }
}

/// Commits exactly one preflighted side effect. The caller MUST have verified
/// the single-use grant immediately before this call. Async from day one so the
/// future shell arm can await `shell::execute` without changing the engine call
/// site; the file-write body itself is synchronous.
///
/// # Errors
/// `E_INTERNAL_ERROR` (I/O/temporary file/rename) only after the grant was
/// consumed; the grant stays consumed (at-most-once, no retry).
pub async fn commit_side_effect(
    jail: &WorkspaceJail,
    prepared: PreparedSideEffect,
) -> Result<ExecutedTool, ToolError> {
    match prepared {
        PreparedSideEffect::FileWrite(write) => commit_file_write(jail, write),
    }
}
```

### 2.4 `preflight_file_write` (exact checks and order)

```rust
fn preflight_file_write(
    jail: &WorkspaceJail,
    args: &serde_json::Value,
) -> Result<PreparedWrite, ToolError> {
    // The registry already validated the JSON Schema; deserialize typed fields.
    let spec: FileWriteArgs = serde_json::from_value(args.clone()).map_err(|error| {
        ToolError::new(
            Refusal::SchemaInvalid,
            format!("file_write arguments do not deserialize after schema validation: {error}"),
        )
    })?;

    if spec.path.trim().is_empty() {
        return Err(ToolError::new(Refusal::SchemaInvalid, "file_write path is empty"));
    }
    // Size before any I/O and before the engine can consume the grant.
    if spec.content.len() > MAX_WRITE_BYTES {
        return Err(ToolError::new(
            Refusal::FileTooLarge,
            format!(
                "file_write content is {} bytes; the write cap is {MAX_WRITE_BYTES} bytes",
                spec.content.len()
            ),
        ));
    }

    // Jail containment: the same six escape vectors as reads, refused not clamped.
    let target = jail.resolve(&spec.path)?;
    if target == jail.root() {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            "file_write cannot target the workspace root",
        ));
    }
    // Existing target must be a regular file; directories and symlinks/junctions
    // are refused (the atomic rename would replace a link itself; refusing keeps
    // operator intent unambiguous).
    match fs::symlink_metadata(&target) {
        Ok(metadata) if metadata.is_dir() => {
            return Err(ToolError::new(
                Refusal::ToolSchema,
                format!("file_write target {:?} is a directory", spec.path),
            ));
        }
        Ok(metadata) if !metadata.is_file() => {
            return Err(ToolError::new(
                Refusal::ToolSchema,
                format!("file_write target {:?} is not a regular file (symlink/non-file refused)", spec.path),
            ));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    // Stale-write guard: 64 hex chars; the current content hash must match.
    // `jail.read` is used (not fs::read) so the 1 MiB read cap applies before the
    // grant; a missing file with expected_sha256 is E_PATH_NOT_FOUND.
    if let Some(expected) = spec.expected_sha256.as_deref() {
        if expected.len() != 64 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ToolError::new(
                Refusal::SchemaInvalid,
                "expected_sha256 must be 64 hexadecimal characters",
            ));
        }
        let current = jail.read(&spec.path)?;
        let actual = sha256_hex(&current);
        if !actual.eq_ignore_ascii_case(expected) {
            return Err(ToolError::new(
                Refusal::SchemaInvalid,
                format!(
                    "stale write refused for {:?}: current sha256 {actual} does not match \
                     expected_sha256 {expected}; nothing was written and no approval was consumed",
                    spec.path
                ),
            ));
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
```

**Refusal mapping decision (`expected_sha256`).** The compiled ledger says this field has no dedicated refusal code and maps to `E_SCHEMA_INVALID` (a known semantic gap) `[LEDGER C-SE-03, C13, §4.9; R1 §12.8]`. R2 §6/Q3 proposed `E_TOOL_SCHEMA` for stale mismatch. **This blueprint follows the ledger:** mismatch/malformed hex ⇒ `E_SCHEMA_INVALID`; root/dir/non-regular ⇒ `E_TOOL_SCHEMA` (argument/target shape, not stale content). The rejected alternative is recorded in §9/U-A; no new code is added either way.

### 2.5 `commit_file_write` (effect only)

```rust
fn commit_file_write(
    jail: &WorkspaceJail,
    prepared: PreparedWrite,
) -> Result<ExecutedTool, ToolError> {
    // Atomic replace: same-directory unique temp (create_new, no symlink
    // following) -> write_all -> sync_all -> rename over target. `write_atomic`
    // re-resolves/contains the path at commit time; preflight plus this second
    // containment is defence in depth (TOCTOU window documented in §9).
    let _committed = jail.write_atomic(&prepared.display_path, &prepared.bytes)?;
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

/// Char-boundary-safe truncation for every side-effect result text.
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

Result text contains **path + byte count + sha256 only** — never the content, argv or a file body `[R2 §2.5]`.

### 2.6 Shell-later extension contract (no code in this wave)

When the shell wave lands it must be **additive only**:
1. add `pub struct PreparedShell { program: String, args: Vec<String>, cwd: Option<String> }` with a `Debug` that omits nothing secret (program/argv are operator-visible only through hash today, so `Debug` should print only lengths/program? — decide in that wave; do **not** log argv);
2. add `PreparedSideEffect::ShellExec(PreparedShell)`;
3. add the `ToolId::ShellExec` arm in `preflight_side_effect` (allowlist/argv/NUL/cwd bounds exactly `[R2 §3.3]`) and `async` arm in `commit_side_effect` (`shell::execute` `[READ shell.rs:173]` + bounded `shell_summary`);
4. engine call sites do not change (same two functions, same order).
The current shell arm returns `E_TOOL_NOT_ALLOWED` **before** any grant check; no row is created and no grant is consumed (tested).

---

## 3. `WorkspaceJail::write_atomic` (exact addition to `jail.rs`)

Add to imports `[READ jail.rs:19-20]`:

```rust
use std::io::Write as _;
```

`uuid` is already a tools dependency with the `v7` feature `[READ tools/Cargo.toml; backend/Cargo.toml:40]`; use `uuid::Uuid::now_v7()` fully qualified (no new import needed if fully qualified).

Insert after `write` `[READ jail.rs:223-243]` (before `list :245`):

```rust
    /// Atomically replaces a workspace file: a unique temp file in the target's
    /// directory is written and fsynced, then renamed over the target.
    ///
    /// Refuses the workspace root, directories and non-regular existing targets,
    /// plus every escape vector [`Self::resolve`] already refuses. Parent
    /// directories are created inside the jail (same rule as [`Self::write`]).
    ///
    /// The caller owns both the approval gate and the content bound: this helper
    /// guarantees containment + atomic replacement only, so no partial file is
    /// ever visible under the target name.
    ///
    /// # Errors
    /// [`Refusal::PathOutsideJail`] for escapes, [`Refusal::ToolSchema`] for the
    /// root/directory/non-regular target, [`Refusal::InternalError`] for I/O.
    pub fn write_atomic(&self, requested: &str, bytes: &[u8]) -> Result<PathBuf, ToolError> {
        let target = self.resolve(requested)?;
        if target == self.root() {
            return Err(ToolError::new(
                Refusal::ToolSchema,
                "cannot atomically write the workspace root",
            ));
        }
        let parent = target.parent().ok_or_else(|| {
            ToolError::new(Refusal::InternalError, "write target has no parent directory")
        })?;
        // Mirror `write`: check the parent's canonical containment, then create it.
        if !self.is_inside(&fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf())) {
            return Err(escape("write parent resolves outside the workspace root"));
        }
        fs::create_dir_all(parent).map_err(|error| {
            ToolError::new(
                Refusal::InternalError,
                format!("cannot create parent directory for {requested:?}: {error}"),
            )
        })?;
        match fs::symlink_metadata(&target) {
            Ok(metadata) if metadata.is_dir() => {
                return Err(ToolError::new(
                    Refusal::ToolSchema,
                    format!("{requested:?} is a directory"),
                ));
            }
            Ok(metadata) if !metadata.is_file() => {
                return Err(ToolError::new(
                    Refusal::ToolSchema,
                    format!("{requested:?} is not a regular file (symlink/non-file refused)"),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }

        // Same-directory temp keeps the rename on one volume; `create_new`
        // refuses to follow a pre-planted symlink and refuses collisions.
        let temp = parent.join(format!(".hive-write-{}.tmp", uuid::Uuid::now_v7()));
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|error| {
                ToolError::new(
                    Refusal::InternalError,
                    format!("cannot create the temporary write file: {error}"),
                )
            })?;
        let write_outcome = (|| -> Result<(), ToolError> {
            file.write_all(bytes)?;
            file.sync_all()?;
            Ok(())
        })();
        drop(file);
        if let Err(error) = write_outcome {
            let _ = fs::remove_file(&temp);
            return Err(error);
        }
        // Windows `std::fs::rename` maps to MoveFileExW with replace-existing
        // semantics [INFERRED from the Rust std docs; proven by the overwrite
        // test]. On failure the stale temp is removed, never the target.
        if let Err(error) = fs::rename(&temp, &target) {
            let _ = fs::remove_file(&temp);
            return Err(ToolError::from(error));
        }
        Ok(target)
    }
```

The explicit `write_outcome` closure above is the required shape: it keeps the temp-file cleanup on every error path and avoids a `file` borrow held inside `.and_then(...)`.

Doc-comment/lint: `uuid` is a public dependency already; no new crate.

**Tests to add in `jail.rs::tests` (`[READ jail.rs:302-437]` style):**
- `write_atomic_creates_and_replaces_file` — first write, read-back; second write with different bytes; read-back equals second; no `.hive-write-*.tmp` remains in the parent (list directory and assert no entry starts with `.hive-write-`).
- `write_atomic_creates_nested_parents_inside_jail` — `"a/b/c.txt"` succeeds; returned path starts with canonical root.
- `write_atomic_refuses_root_and_directory_targets` — root (`"."`/empty) ⇒ `E_TOOL_SCHEMA`; pre-created directory target ⇒ `E_TOOL_SCHEMA`.
- `write_atomic_escape_vectors_stay_refused` — `../x`, `C:\x`, `PROGRA~1\x`, NUL path ⇒ `E_PATH_OUTSIDE_JAIL` (reuses resolve; one representative per vector is enough here because the matrix test already exists).
- `write_atomic_preserves_existing_file_on_refused_target` — directory target refusal leaves any existing file untouched? (directory case has no file; use root case). Optional.

---

## 4. Preflight-before-grant order (the exact engine-facing protocol)

The engine (WP-03, not this wave) must call, per planned side-effect call:

```
1. tool_loop::plan_call / validate_plan            (existing, WorkerAgent tier + schema)
2. side_effects::preflight_side_effect(...)         (this wave: sync, read-only)
      -> on Err: publish TOOL_RESULT refused(code) + tool_turns row; do NOT create an
         approval row; continue the model loop (engine policy; no suspension)
3. ToolsBridge::request_model_tool_approval(...)    (existing live producer)
      -> commit row; arm waiter; publish APPROVAL_REQUEST; persist awaiting_approval
      -> wait (engine)
4. after an approve wake:
   a. side_effects::preflight_side_effect(...)      (re-run! the workspace may have
                                                     changed while waiting; a stale
                                                     expected_sha256 must not consume
                                                     the grant)
   b. ToolsBridge::verify_tool_grant(run_id, ToolId::FileWrite, &plan.args, now)
         -> on Err (E_APPROVAL_REQUIRED/REPLAY): no effect, publish refused TOOL_RESULT;
            run policy decides fail/continue (R1 S10 fails the run by name)
   c. side_effects::commit_side_effect(jail, prepared).await
         -> Ok(ExecutedTool::Ok{text}) => TOOL_RESULT ok + tool_turns
         -> Err(ToolError) => TOOL_RESULT refused(named code); grant STAYS consumed
5. exactly one finish_tool_call per planned call (engine); never retry a consumed grant.
```

- **Why preflight twice:** the first preflight prevents a pointless approval card for a call that can never succeed; the second is mandatory because the wait can last up to the TTL and the workspace/stale-hash preconditions can change. Both are read-only; neither consumes a grant. The engine may keep only the second `PreparedWrite` (the first is dropped).
- **Grant consumption point:** `verify_tool_grant` `[WIP core/src/tools_bridge.rs:166]` wraps `ApprovalGate::verify_grant` `[READ approval.rs:772 HEAD / :843 live]` (AllowSession short-circuit; AllowOnce `UPDATE … executed_at IS NULL` CAS; second call ⇒ `E_APPROVAL_REPLAY`; missing/denied/expired row ⇒ `E_APPROVAL_REQUIRED` at the current code). `E_APPROVAL_TIMEOUT` is produced by the engine's wait/`run_state` timeout path, not by `verify_grant` today (`[READ approval.rs verify_grant]`; doc-comment claims timeout but the code path returns `E_APPROVAL_REQUIRED` for a non-allow row — recorded as risk R6).
- **No approval.rs change in WP-02.** The live WP-01 lane owns that file.

### 4.5 What the shell wave will change (additive)

`preflight_side_effect` gains `ToolId::ShellExec => Ok(PreparedSideEffect::ShellExec(preflight_shell_exec(...)?))` with the R2 §3.3 bounds (program ≤260 B and allowlisted bare name, args ≤64, arg ≤4 KiB, argv ≤16 KiB, no NUL, cwd resolved+directory, cancellation before grant); `commit_side_effect` gains an async arm calling `shell::execute` `[READ shell.rs:173]`, then `bounded_text(shell_summary)`. Engine protocol unchanged. No `cmd.exe` policy decision is made here (`[R2 §11 Q1]` still open).

---

## 5. Refusal mapping (frozen 65 only; no new codes)

| Stage / condition | Code | Produced by (this module) |
|---|---|---|
| Unknown tool name | `E_TOOL_NOT_FOUND` | `registry.resolve_call` `[READ tool.rs:446]` |
| Tier not allowed | `E_TOOL_NOT_ALLOWED` | `registry.resolve_call` |
| Args fail JSON Schema | `E_TOOL_SCHEMA` | `registry.resolve_call` → `validate_args` `[READ tool.rs:179]` |
| Args fail typed deserialize after schema | `E_SCHEMA_INVALID` | `preflight_file_write` |
| Read/other tool reaches the granted executor | `E_TOOL_NOT_ALLOWED` | `preflight_side_effect` match |
| `shell_exec` in this wave (not enabled) | `E_TOOL_NOT_ALLOWED` | `preflight_side_effect` match (before grant) |
| Cancelled before preflight | `E_TOOL_CANCELLED` | `preflight_side_effect` entry |
| Empty path | `E_SCHEMA_INVALID` | `preflight_file_write` |
| Content > 1 MiB | `E_FILE_TOO_LARGE` | `preflight_file_write` |
| Path escape (six vectors) | `E_PATH_OUTSIDE_JAIL` | `jail.resolve` `[READ jail.rs:92]` |
| Target is the root | `E_TOOL_SCHEMA` | `preflight_file_write` |
| Target is a directory / non-regular (symlink etc.) | `E_TOOL_SCHEMA` | `preflight_file_write` (and defensive re-check in `write_atomic`) |
| `expected_sha256` malformed (not 64 hex) | `E_SCHEMA_INVALID` | `preflight_file_write` |
| `expected_sha256` mismatch | `E_SCHEMA_INVALID` (ledger choice; R2 proposed `E_TOOL_SCHEMA` — see §9/U-A) | `preflight_file_write` |
| `expected_sha256` given but file missing | `E_PATH_NOT_FOUND` | `jail.read` `[READ jail.rs:185]` |
| Stale-target read exceeds 1 MiB read cap | `E_FILE_TOO_LARGE` | `jail.read` |
| No/denied/expired approval, args-hash mismatch | `E_APPROVAL_REQUIRED` | `ApprovalGate::verify_grant` (core bridge, caller stage) |
| AllowOnce already executed / CAS lost | `E_APPROVAL_REPLAY` | `verify_grant` |
| Approval wait expired | `E_APPROVAL_TIMEOUT` | engine wait/`run_state` (not this module) |
| Temp create/write/sync/rename I/O failure (after grant) | `E_INTERNAL_ERROR` | `write_atomic`/`commit_file_write` |
| SQLite/DB failure | `E_DB_BUSY`/`E_INTERNAL_ERROR` | core bridge `verify_tool_grant` |

No new refusal code, event, command or DB column is introduced by WP-02 `[LEDGER §4.11]`.

---

## 6. Engine hand-off contract (for WP-03; sketched, NOT implemented in WP-02)

Exact call sites the engine will need after both waves:

```rust
// WP-03 sketch only — do not edit core in this wave.
use hiverouter_tools::side_effects::{commit_side_effect, preflight_side_effect};
use hiverouter_tools::{AgentTier, ToolId};

// (a) before suspending: reject doomed calls without creating an approval row
let prepared_precheck = preflight_side_effect(
    workspace, &self.tool_registry, AgentTier::WorkerAgent,
    &plan.tool, &plan.args, Some(token),
); // Err -> refused TOOL_RESULT, no suspension
drop(prepared_precheck); // pre-approval value is never committed

// (b) after the approve wake and run_state Resume:
let prepared = preflight_side_effect(/* same args */)?;          // re-run, no grant
let tool_id = ToolId::from_id(&plan.tool).ok_or(Refusal::ToolNotFound)?;
self.tools.as_ref().ok_or(Refusal::InternalError)?
    .verify_tool_grant(run.id, tool_id, &plan.args, self.now_unix_secs())?; // consumes grant
let executed = commit_side_effect(workspace, prepared).await;    // exactly one effect
// map Ok(ExecutedTool::Ok{text}) / Err(ToolError) -> engine ToolExecution then finish_tool_call
```

This uses the **live WP-01 bridge** `verify_tool_grant` (`[WIP core/src/tools_bridge.rs:166-181]`), so WP-02 does not modify core and the engine has one grant authority. The `commit_side_effect` future is `Send` (`PreparedSideEffect` contains `PathBuf/String/Vec<u8>`; `WorkspaceJail` is `PathBuf`), so it can run in the engine's async approve path `[INFERRED]`.

---

## 7. Tests (tools-level; deterministic, offline)

In `side_effects.rs::tests` (in-memory SQLite via `approval::migrate`, `tempfile::TempDir`, `ApprovalGate`; no network/pay). Helper:

```rust
fn seeded() -> (tempfile::TempDir, WorkspaceJail, rusqlite::Connection, ApprovalGate, RunId) {
    let temp = tempfile::tempdir().expect("tempdir");
    let jail = WorkspaceJail::open(temp.path()).expect("jail");
    let conn = rusqlite::Connection::open_in_memory().expect("memory db");
    crate::approval::migrate(&conn).expect("approval table");
    (temp, jail, conn, ApprovalGate::with_ttl(120), RunId::new())
}
```

Test names and assertions (each must fail if its guard is removed — `/forge` RED-first):

1. `preflight_refuses_read_only_unknown_and_shell_tools` — `file_read`/`file_list`/`unknown`/`shell_exec` ⇒ `E_TOOL_NOT_ALLOWED` (or `E_TOOL_NOT_FOUND` for unknown); temp dir unchanged; no approval row.
2. `doomed_write_does_not_consume_grant` — create a valid AllowOnce grant; for each doomed args (`"../escape.txt"`, `"."` root, pre-created directory target, `content` > `MAX_WRITE_BYTES`, malformed `expected_sha256`, mismatched `expected_sha256`) assert `preflight_side_effect` is `Err` **and** `executed_at IS NULL`; then a valid preflight + `verify_grant` + commit succeeds exactly once.
3. `commit_writes_and_reports_path_size_sha_without_content` — nested path created; read-back equals content; result text contains `wrote`, byte count, sha256 and path; `!result_text.contains(content)`.
4. `commit_replaces_existing_file_atomically_without_temp_residue` — overwrite existing content; read-back is new; no `.hive-write-*.tmp` entry remains.
5. `stale_expected_sha256_accepts_match_and_refuses_mismatch_without_grant` — matching expected succeeds; mismatch/missing/malformed refused with the §5 codes and no write.
6. `grant_is_consumed_exactly_once` — `verify_grant` Ok once; second `verify_grant` ⇒ `E_APPROVAL_REPLAY`; the file content from the first commit is unchanged after the failed second attempt.
7. `cancelled_token_refuses_before_grant` — pre-cancelled `CancellationToken` ⇒ `E_TOOL_CANCELLED`; grant row untouched.
8. `result_text_is_bounded_on_char_boundary` — direct `bounded_text` unit case (long non-ASCII) returns valid UTF-8, ≤ cap + marker.
9. `write_atomic_*` tests live in `jail.rs` (§3) — nested parents, overwrite, root/dir refusals, no temp residue.

Existing suites that must stay green `[READ]`: `approval.rs` (now 16 tests after live WP-01 additions), `executor.rs` read-only matrix, `jail.rs` escape matrix + read cap, `shell.rs` allowlist/timeout/env/cancel, `tool.rs` registry, core `tool_loop_engine` read-only loop (no core edits).

Commands (Lead runs; this planner ran none):
```powershell
cd backend
$env:PATH = "$env:USERPROFILE\.cargo\bin;$PWD\..\tools\w64devkit\bin;$env:PATH"
cargo test -p hiverouter-tools --lib jail::tests --locked
cargo test -p hiverouter-tools --lib side_effects::tests --locked
cargo test -p hiverouter-tools --lib --locked
cargo test --workspace --locked
```

---

## 8. Merge / coordination notes (live tree is dirty)

1. **WP-01 lane owns `approval.rs`, `tools_bridge.rs`, and currently `lib.rs`** `[WIP; `git status` at 18:51 CT]`. WP-02 must land **after** WP-01 is committed (or be rebased by the Lead): `lib.rs` has one trivial collision (approval export list vs new `pub mod side_effects; pub use side_effects::{…}`); do not edit `lib.rs` concurrently.
2. WP-02 file set is `tools/src/side_effects.rs` (new), `tools/src/jail.rs` (append `write_atomic`), `tools/src/lib.rs` (module + exports only), and tests inside those files. `approval.rs`/`core` untouched.
3. Line anchors above are HEAD `90cb7a7`; live WP-01 adds +73 to `approval.rs` (verify_grant `:772 → :843`, request_tool_approval `:849 → :920`) and +76 to `tools_bridge.rs`. Re-run `git rev-parse HEAD && git status --porcelain` and `Select-String` the anchors before editing; do not build against stale line numbers (ledger C11).
4. C1 is resolved by this blueprint's split API + live bridge method (Lead must accept in the run plan); C2 remains a WP-03 decision (default-off flag vs always-wired) and does not change WP-02's code. WP-02 has **no caller** after landing: nothing is user-visible until WP-03 wires it — do not claim runtime reachability.
5. No new dependency: `uuid`/`rusqlite`/`tokio-util` already in `tools/Cargo.toml`; `tempfile` already a dev-dependency `[READ tools/Cargo.toml]`.

---

## 9. Risks, open decisions, and what must not be claimed

**Risks**
- **R1 (coordination):** live WP-01 lane → dirty tree, `lib.rs` collision; sequence after its commit.
- **R2 (TOCTOU):** preflight → verify → commit re-resolves the path, but another process can change the parent/target between checks; `create_new` temp, root/dir/symlink refusals, and same-directory rename bound the damage, not eliminate a same-user race. Documented; no OS sandbox.
- **R3 (grant burned on I/O failure):** if `commit_file_write` fails after `verify_tool_grant`, the AllowOnce grant stays consumed (at-most-once). Correct by design; must be reported as refused `E_INTERNAL_ERROR`, never retried.
- **R4 (no shell yet):** `shell_exec` is refused by name in this wave; no process spawn, allowlist, env-sanitize, timeout or cancellation claims.
- **R5 (`expected_sha256` semantics):** no dedicated frozen code; this blueprint maps mismatch to `E_SCHEMA_INVALID` (ledger choice); R2/Q3 preferred `E_TOOL_SCHEMA` — Lead decision needed (U-A), do not claim a dedicated stale-write code.
- **R6 (`verify_grant` docs vs code):** the doc-comment claims `E_APPROVAL_TIMEOUT`, but the current code returns `E_APPROVAL_REQUIRED` for a non-allow row; timeout is the engine's path. Do not claim `verify_grant` returns `E_APPROVAL_TIMEOUT`.
- **R7 (Windows rename):** `std::fs::rename` replace semantics are `INFERRED` from std docs; the overwrite test proves it on this host (Lead run required).
- **R8 (memory):** up to 1 MiB content is held in the args `Value`, the typed `String` and `Vec<u8>` (~2–3×) — bounded, but not zero-copy.
- **R9 (API reachability):** WP-02 compiles as dead code until WP-03; no behavior change/claim.
- **R10 (`write_atomic` parent TOCTOU):** parent creation follows the same containment pattern as `write`; a symlink planted between check and `create_dir_all` is not fully preventable at this layer (same pre-existing risk as `write`).

**Open decisions (Lead)**
- **U-A:** `expected_sha256` mismatch code: `E_SCHEMA_INVALID` (this blueprint, ledger) vs `E_TOOL_SCHEMA` (R2). Recommend ledger.
- **U-B:** accept the split preflight/commit API (C1) instead of R2's monolithic `execute_approved`; required because the live bridge owns grant verification and core DB closures are sync.
- **U-C:** shell allowlist policy / `cmd.exe` removal `[R2 Q1]` — later shell wave.
- **U-D:** WP-01 commit timing (live lane) so WP-02 can touch `lib.rs`.

**What will work after WP-02 (design target, to be proven by tests)**
- `file_write` inside the jail: bounded content, atomic replace (no partial file under the target name), parents created inside the jail, root/dir/symlink target refusals, stale-hash guard, path-escape refusals.
- Strict preflight→grant→commit ordering with tests proving a doomed call leaves the grant unconsumed and a valid grant executes exactly once.
- Result text path/size/sha only; no content/argv in events, ledger or logs.
- Existing read-only and approval suites unchanged.

**What will NOT work / is not claimed**
- No `shell_exec` execution, no child spawn/allowlist/env/timeout claims (later wave).
- No engine integration, suspension, approval card or runtime reachability in this wave (WP-03).
- No OS sandbox/container; the server user's filesystem/network beyond the jail and allowlist is unchanged.
- No exactly-once across crash (at-most-once; crash between grant and effect => consumed, failed/interrupted, never retried).
- No undo, directory deletion, binary writes, concurrent-write merge, network tools, elevation, PTY.
- No new refusal code, event, command, DB migration; no claim `expected_sha256` has a dedicated code.
- No build/test result by this planner.

---

## 10. Logic-gatekeeper Phase 1 (for the implementing wave)

```json
{"phase1":{"architecture":"tools/src/side_effects.rs owns the only granted write path; jail.rs owns containment + atomic replace; approval.rs/ApprovalGate owns the single-use grant (unchanged); core tools_bridge.rs owns the DB/grant seam via the live verify_tool_grant; the core engine (WP-03) owns suspension/order/events; tool.rs owns registry/permission contracts.","dataFlow":"model tool call -> engine plan_call/validate_plan -> side_effects::preflight_side_effect (registry/tier/schema, permission class, size, jail containment, target kind, stale hash; read-only) -> engine requests approval + waits -> after approve: re-preflight -> ToolsBridge::verify_tool_grant (AllowOnce executed_at CAS) -> side_effects::commit_side_effect (atomic temp+rename) -> bounded result text (path/size/sha only) -> engine TOOL_RESULT + tool_turns. Deny/timeout/cancel/replay never reach commit.","edgeCases":["doomed call (escape/dir/>1MiB/stale) before grant => named refusal, executed_at IS NULL","second verify_grant for the same AllowOnce => E_APPROVAL_REPLAY, content unchanged","I/O failure after grant => E_INTERNAL_ERROR, grant stays consumed, no retry","symlink/non-regular target => E_TOOL_SCHEMA before grant; rename never follows a link","shell_exec in file_write wave => E_TOOL_NOT_ALLOWED before grant, no row created"],"rootCause":"The approval gate and atomic filesystem primitives exist, but no code path can execute an approved write: executor.rs is read-only and no module owns side effects. The fix is a strict two-phase API with read-only preflight before the single-use grant CAS and exactly one atomic effect after it; any reordering burns consent or executes unreviewed."}}
```

---

## 11. Evidence-check table

| Claim | Evidence | Status |
|---|---|---|
| No side-effect executor exists at HEAD/current tree | `executor.rs` only `execute_read_only [READ :52]`; grep `write_atomic`/`execute_approved`/`GrantedExecution`/`side_effects` module = 0 (only test fn names) | VERIFIED (read/grep) |
| `jail.write` is non-atomic `fs::write`; `write_atomic` absent | `jail.rs:223-243`; grep 0 | VERIFIED |
| `jail.resolve` refuses six escape vectors; `read` capped 1 MiB; `write` creates parents | `jail.rs:92-139`, `:185-214`, `:223-243` | VERIFIED |
| Approval gate enforces single-use `executed_at` CAS | `approval.rs:772` (HEAD)/`:843` (live), `UPDATE … executed_at IS NULL` | VERIFIED |
| Live WP-01 lane provides `ToolsBridge::verify_tool_grant(run_id, ToolId, args, now)` and approval primitives | `git status` + `git diff` at 18:51 CT; live `tools_bridge.rs:166-181` | READ (WIP, may change) |
| `shell::execute` is async and allowlisted | `shell.rs:173`, `:66`, `:96` | VERIFIED |
| `FileWriteArgs` has `path/content/expected_sha256`; schema denies unknown fields | `tool.rs:255-263`, `:446` | VERIFIED |
| No dedicated stale-write refusal code exists | refusal code list; ledger C-SE-03/C13 | VERIFIED (read; ledger) |
| No new dependency needed | `tools/Cargo.toml` has uuid/rusqlite/tokio-util; tempfile dev-dep | VERIFIED (manifest read) |
| Documented HEAD was clean; observed tree is dirty from another lane | `git status --porcelain` at 18:51 CT | VERIFIED (read); WIP may change |
| Runtime behavior of the proposed code / tests | not implemented, no build/test by planner | **UNVERIFIED** |
| Windows `fs::rename` replaces existing file | Rust std docs (not fetched this session); overwrite test will prove | **UNVERIFIED / INFERRED** |

---

## 12. UNKNOWN register

| ID | Unknown | Settle by |
|---|---|---|
| U-A | `expected_sha256` mismatch code (`E_SCHEMA_INVALID` vs `E_TOOL_SCHEMA`) | Lead decision; recommend ledger (`E_SCHEMA_INVALID`) |
| U-B | Accept split preflight/commit API vs R2 monolithic `execute_approved` | Lead C1 decision |
| U-C | Shell allowlist policy / `cmd.exe` removal | Lead/product, shell wave |
| U-D | WP-01 lane commit timing/final shape | `git log`/`git diff` before implementing |
| U-E | Windows rename-over-existing on this host | `write_atomic_creates_and_replaces_file` (Lead run) |
| U-F | Whether any existing DB rows would block the WP-01 unique pending index (not WP-02) | WP-01 query (ledger U2) |

---

## 13. Skill application evidence

- **/research**: questions answered from repository primary sources only (does an executor exist? what owns grants? what primitives exist?); every answer carries `file:line`; no external sources needed; UNKNOWNs kept in §12.
- **/forgewright-lead**: read current code, callers and tests before the hypothesis; root cause evidenced by grep/read (no side-effect path); claims labelled VERIFIED/READ/INFERRED/UNVERIFIED; complete code shapes only.
- **/forgewright-plan**: outcome, current anchors, decisions with rejected alternatives (C1 split, expected_sha256 mapping), file-by-file steps, failure modes, tests, rollback (delete new module + jail hunk; no caller), out-of-scope.
- **/deep-naming**: vocabulary kept consistent with the project: `preflight_*` (read-only check), `commit_*` (effect), `PreparedWrite`, `PreparedSideEffect`, `write_atomic`, `MAX_WRITE_BYTES`; rejected generic `execute_side_effect` for the sync half and R2's `GrantedExecution` (no DB borrow in tools); no rename applied.
- **/truthseeker**: falsifiable invariants — one preflight → one grant consumption → at most one effect; doomed call never consumes; every refusal is a frozen code; one result per planned call.
- **/scientific-thinking**: alternative hypotheses evaluated — (a) engine-side execute (rejected: jail/shell ownership + no preflight before grant), (b) tools monolithic R2 API (rejected for core's sync DB closure + live bridge authority), (c) this split API (chosen, evidence-based).
- **/forge**: every predicted defect (grant burned, double execution, partial file, temp residue, content leak, symlink target) has a RED test in §7.
- **/evidence-check**: §11 table; runtime claims remain UNVERIFIED; no contradicted claim found.

**Read-only statement:** this planner read the corpus and wrote only this file; no product/test/config/git write, no build, no test, no key access, no commit.
