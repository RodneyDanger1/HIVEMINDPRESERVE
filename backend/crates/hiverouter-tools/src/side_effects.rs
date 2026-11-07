//! Granted side-effect executor for the 1.2 tool loop (WP-02).
//!
//! This is the ONLY module that may write files for a model-originated tool
//! call. It performs **read-only preflight** only; the single-use approval grant
//! is verified by the core bridge *between* preflight and commit
//! (`ToolsBridge::verify_tool_grant`), so a doomed call never burns consent.
//! Nothing here resolves approvals, reads the approval table, retries, or
//! publishes events; the core engine owns suspension/events/persistence.
//!
//! File-write and allowlisted shell execution wave: `file_write` goes through
//! [`WorkspaceJail`] atomically, `shell_exec` goes through the allowlisted
//! [`crate::shell`] Job Object path. Both are preflighted read-only before the
//! grant, then committed exactly once after the grant is verified.

use std::fmt;
use std::fs;

use hiverouter_contract::Refusal;
use tokio_util::sync::CancellationToken;

use crate::error::ToolError;
use crate::executor::ExecutedTool;
use crate::jail::WorkspaceJail;
use crate::sha256::sha256_hex;
use crate::shell::{execute as execute_shell, ShellPolicy, ShellRequest};
use crate::tool::{AgentTier, FileWriteArgs, PermissionClass, ShellExecArgs, ToolId, ToolRegistry};

/// Maximum UTF-8 content a `file_write` may write (mirrors the 1 MiB read cap).
pub const MAX_WRITE_BYTES: usize = 1024 * 1024;
/// Hard cap on the returned tool-result text (mirrors core `MAX_TOOL_RESULT_BYTES`).
pub const MAX_SIDE_EFFECT_TEXT_BYTES: usize = 65_536;
/// Maximum shell program-name bytes (bare executable name only).
pub const MAX_SHELL_PROGRAM_BYTES: usize = 260;
/// Maximum number of shell arguments.
pub const MAX_SHELL_ARGS: usize = 64;
/// Maximum bytes of one shell argument.
pub const MAX_SHELL_ARG_BYTES: usize = 4 * 1024;
/// Maximum total argv bytes for one shell call.
pub const MAX_SHELL_ARGV_BYTES: usize = 16 * 1024;

/// Fully pre-checked file write; no effect has happened yet.
///
/// Fields are private: the engine receives the value from
/// [`preflight_side_effect`] and must hand it back to [`commit_side_effect`]
/// after the grant is verified. It cannot inspect or construct a prepared
/// write, so it cannot bypass the preflight checks.
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

/// Fully pre-checked shell call; no child has been spawned yet.
///
/// Fields are private for the same reason as [`PreparedWrite`]; the `Debug`
/// rendering prints the bare program and the argument count, never the argv
/// values (they can carry operator data).
pub struct PreparedShell {
    request: ShellRequest,
}

impl fmt::Debug for PreparedShell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedShell")
            .field("program", &self.request.program)
            .field("args", &self.request.args.len())
            .field("cwd", &self.request.cwd)
            .finish()
    }
}

/// The one preflighted side effect the engine may commit after granting it.
#[derive(Debug)]
pub enum PreparedSideEffect {
    /// A checked atomic file replacement.
    FileWrite(PreparedWrite),
    /// A checked allowlisted child-process execution.
    ShellExec(PreparedShell),
}

/// Read-only preflight for one model-originated side-effect tool call.
///
/// Order: registry lookup/tier/schema -> permission class -> tool-specific
/// read-only checks -> (return). Never reads the approvals table, never consumes
/// a grant, never writes or spawns.
///
/// # Errors
/// Frozen codes only: `E_TOOL_NOT_FOUND`, `E_TOOL_NOT_ALLOWED`, `E_TOOL_SCHEMA`,
/// `E_SCHEMA_INVALID`, `E_FILE_TOO_LARGE`, `E_PATH_OUTSIDE_JAIL`,
/// `E_PATH_NOT_FOUND`, `E_TOOL_CANCELLED`.
pub fn preflight_side_effect(
    jail: &WorkspaceJail,
    registry: &ToolRegistry,
    shell_policy: &ShellPolicy,
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
    // reach the filesystem or spawn a child, and never touch the grant.
    let contract = registry.resolve_call(tier, tool, args)?;
    match contract.id {
        ToolId::FileWrite => {
            if contract.permission_class != PermissionClass::Mutate {
                return Err(ToolError::new(
                    Refusal::ToolNotAllowed,
                    "file_write registry permission class is not Mutate; refusing before any effect",
                ));
            }
            Ok(PreparedSideEffect::FileWrite(preflight_file_write(
                jail, args,
            )?))
        }
        ToolId::ShellExec => {
            if contract.permission_class != PermissionClass::Execute {
                return Err(ToolError::new(
                    Refusal::ToolNotAllowed,
                    "shell_exec registry permission class is not Execute; refusing before any spawn",
                ));
            }
            Ok(PreparedSideEffect::ShellExec(preflight_shell_exec(
                jail,
                shell_policy,
                args,
            )?))
        }
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
/// the single-use grant immediately before this call. The shell arm awaits the
/// allowlisted child process (Job Object containment in [`crate::shell`]).
///
/// # Errors
/// `E_INTERNAL_ERROR` (I/O/temporary file/rename/spawn) only after the grant was
/// consumed; the grant stays consumed (at-most-once, no retry).
pub async fn commit_side_effect(
    jail: &WorkspaceJail,
    shell_policy: &ShellPolicy,
    prepared: PreparedSideEffect,
    cancellation: Option<&CancellationToken>,
) -> Result<ExecutedTool, ToolError> {
    match prepared {
        PreparedSideEffect::FileWrite(write) => commit_file_write(jail, write),
        PreparedSideEffect::ShellExec(shell) => {
            commit_shell_exec(jail, shell_policy, shell, cancellation).await
        }
    }
}

fn preflight_shell_exec(
    jail: &WorkspaceJail,
    policy: &ShellPolicy,
    args: &serde_json::Value,
) -> Result<PreparedShell, ToolError> {
    let spec: ShellExecArgs = serde_json::from_value(args.clone()).map_err(|error| {
        ToolError::new(
            Refusal::SchemaInvalid,
            format!("shell_exec arguments do not deserialize after schema validation: {error}"),
        )
    })?;

    // Program: bare allowlisted executable name only; no path separators.
    let program = spec.program.trim();
    if program.is_empty() || program.len() > MAX_SHELL_PROGRAM_BYTES {
        return Err(ToolError::new(
            Refusal::SchemaInvalid,
            format!(
                "shell_exec program must be 1..={MAX_SHELL_PROGRAM_BYTES} bytes after trim"
            ),
        ));
    }
    if program.contains('/') || program.contains('\\') || program.contains('\0') {
        return Err(ToolError::new(
            Refusal::ToolNotAllowed,
            format!(
                "shell_exec program {program:?} must be a bare allowlisted name; \
                 path-qualified programs are refused"
            ),
        ));
    }
    if !policy.allows(program) {
        return Err(ToolError::new(
            Refusal::ToolNotAllowed,
            format!(
                "shell_exec program {program:?} is not on the shell allowlist ({}); \
                 nothing spawned and no approval was consumed",
                policy.allowed_programs().join(", ")
            ),
        ));
    }
    // Argument bounds and injection hygiene: direct argv only, never a shell
    // command string; NUL bytes and oversized argv are refused before the grant.
    if spec.args.len() > MAX_SHELL_ARGS {
        return Err(ToolError::new(
            Refusal::SchemaInvalid,
            format!(
                "shell_exec carries {} arguments; the cap is {MAX_SHELL_ARGS}",
                spec.args.len()
            ),
        ));
    }
    let mut argv_bytes = 0_usize;
    for argument in &spec.args {
        if argument.len() > MAX_SHELL_ARG_BYTES {
            return Err(ToolError::new(
                Refusal::SchemaInvalid,
                format!(
                    "one shell_exec argument is {} bytes; the cap is {MAX_SHELL_ARG_BYTES}",
                    argument.len()
                ),
            ));
        }
        if argument.contains('\0') {
            return Err(ToolError::new(
                Refusal::SchemaInvalid,
                "shell_exec arguments must not contain NUL bytes",
            ));
        }
        argv_bytes = argv_bytes.saturating_add(argument.len());
    }
    if argv_bytes > MAX_SHELL_ARGV_BYTES {
        return Err(ToolError::new(
            Refusal::SchemaInvalid,
            format!(
                "shell_exec argv is {argv_bytes} bytes; the cap is {MAX_SHELL_ARGV_BYTES}"
            ),
        ));
    }
    // cwd must already resolve inside the jail and be a directory (the child
    // cannot be spawned for a doomed cwd after consent is consumed).
    if let Some(cwd) = spec.cwd.as_deref() {
        let resolved = jail.resolve(cwd)?;
        if !resolved.is_dir() {
            return Err(ToolError::new(
                Refusal::ToolSchema,
                "shell_exec working directory is not a directory inside the workspace jail",
            ));
        }
    }
    Ok(PreparedShell {
        request: ShellRequest {
            program: program.to_owned(),
            args: spec.args,
            cwd: spec.cwd,
        },
    })
}

async fn commit_shell_exec(
    jail: &WorkspaceJail,
    policy: &ShellPolicy,
    prepared: PreparedShell,
    cancellation: Option<&CancellationToken>,
) -> Result<ExecutedTool, ToolError> {
    let output = execute_shell(jail, policy, &prepared.request, cancellation).await?;
    // Result text: program, exit status, truncation flag and bounded stdout/
    // stderr. The argv is deliberately omitted from tool-result text (it can
    // carry operator data); stderr is kept because it is child output the model
    // needs to recover, and is bounded by the shell policy caps.
    let stdout = bounded_text(&output.stdout);
    let stderr = bounded_text(&output.stderr);
    let text = format!(
        "shell_exec {:?} exit_code={:?} success={} truncated={} stdout={:?} stderr={:?}",
        output.program, output.exit_code, output.success, output.truncated, stdout, stderr
    );
    Ok(ExecutedTool::Ok {
        text: bounded_text(&text),
    })
}

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
        return Err(ToolError::new(
            Refusal::SchemaInvalid,
            "file_write path is empty",
        ));
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
                format!(
                    "file_write target {:?} is not a regular file (symlink/non-file refused)",
                    spec.path
                ),
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

fn commit_file_write(
    jail: &WorkspaceJail,
    prepared: PreparedWrite,
) -> Result<ExecutedTool, ToolError> {
    // Re-resolve at commit time (TOCTOU defence in depth): a target that escaped
    // after preflight refuses before any write.
    let resolved = jail.resolve(&prepared.display_path)?;
    if resolved != prepared.target {
        return Err(ToolError::new(
            Refusal::PathOutsideJail,
            "file_write target changed containment between preflight and commit; nothing written",
        ));
    }
    // Atomic replace: same-directory unique temp (create_new, no symlink
    // following) -> write_all -> sync_all -> rename over target. `write_atomic`
    // re-resolves/contains the path at commit time; preflight plus this second
    // containment is defence in depth (TOCTOU window documented in WP-02 §9).
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approval::{migrate, request_tool_approval, resolve_with_nonce, ApprovalGate};
    use hiverouter_contract::{RunId, TaskId};
    use rusqlite::Connection;

    const NOW: u64 = 1_800_000_000;

    fn seeded() -> (
        tempfile::TempDir,
        WorkspaceJail,
        Connection,
        ApprovalGate,
        RunId,
    ) {
        let temp = tempfile::tempdir().expect("tempdir");
        let jail = WorkspaceJail::open(temp.path()).expect("jail");
        let conn = Connection::open_in_memory().expect("memory db");
        migrate(&conn).expect("approval table");
        let gate = ApprovalGate::with_ttl(120);
        let run = RunId::new();
        (temp, jail, conn, gate, run)
    }

    fn registry() -> ToolRegistry {
        ToolRegistry::standard()
    }

    fn executed_at(conn: &Connection) -> Option<String> {
        conn.query_row(
            "SELECT executed_at FROM approvals ORDER BY created_at ASC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .expect("executed_at column")
    }

    fn grant(
        conn: &Connection,
        gate: &ApprovalGate,
        run: RunId,
        args: &serde_json::Value,
        tool: &str,
    ) {
        let request = request_tool_approval(
            gate,
            conn,
            run,
            TaskId::new(),
            "call-wp02",
            tool,
            args,
            NOW,
        )
        .expect("request approval");
        let approval_id: String = conn
            .query_row("SELECT approval_id FROM approvals LIMIT 1", [], |row| {
                row.get(0)
            })
            .expect("approval id");
        resolve_with_nonce(conn, &approval_id, &request.nonce, "approve", None, NOW)
            .expect("approve");
    }

    fn preflight<'a>(
        jail: &'a WorkspaceJail,
        args: &'a serde_json::Value,
        cancellation: Option<&'a CancellationToken>,
    ) -> Result<PreparedSideEffect, ToolError> {
        preflight_side_effect(
            jail,
            &registry(),
            &ShellPolicy::conservative_default(),
            AgentTier::WorkerAgent,
            "file_write",
            args,
            cancellation,
        )
    }

    #[test]
    fn preflight_refuses_read_only_and_unknown_tools() {
        let (_temp, jail, _conn, _gate, _run) = seeded();
        let registry = registry();
        let policy = ShellPolicy::conservative_default();
        for tool in ["file_read", "file_list"] {
            let error = preflight_side_effect(
                &jail,
                &registry,
                &policy,
                AgentTier::WorkerAgent,
                tool,
                &serde_json::json!({"path": "note.txt"}),
                None,
            )
            .expect_err(tool);
            assert_eq!(error.code(), "E_TOOL_NOT_ALLOWED", "{tool}");
        }
        let error = preflight_side_effect(
            &jail,
            &registry,
            &policy,
            AgentTier::WorkerAgent,
            "unknown_tool",
            &serde_json::json!({}),
            None,
        )
        .expect_err("unknown tool");
        assert_eq!(error.code(), "E_TOOL_NOT_FOUND");
        assert!(!jail.root().join("note.txt").exists());
    }

    #[test]
    fn doomed_write_does_not_consume_grant() {
        let (temp, jail, conn, gate, run) = seeded();
        let valid = serde_json::json!({"path": "notes.txt", "content": "hello"});
        let valid_hash = sha256_hex(b"hello");
        let target = temp.path().join("target-dir");
        std::fs::create_dir(&target).expect("dir target");
        let existing = temp.path().join("existing.txt");
        std::fs::write(&existing, b"old-content").expect("existing file");

        grant(&conn, &gate, run, &valid, "file_write");
        assert_eq!(executed_at(&conn), None, "grant starts unconsumed");

        let doomed = [
            ("../escape.txt", valid_hash.clone()),
            (".", valid_hash.clone()),
            ("target-dir", valid_hash.clone()),
            ("notes.txt", "zz".to_string()),
            ("notes.txt", "a".repeat(64)),
            ("missing.txt", "b".repeat(64)),
        ];
        for (path, expected) in doomed {
            let args = serde_json::json!({
                "path": path,
                "content": "hello",
                "expected_sha256": expected,
            });
            assert!(
                preflight(&jail, &args, None).is_err(),
                "doomed preflight must fail for {path:?}"
            );
            assert_eq!(
                executed_at(&conn),
                None,
                "doomed preflight burned the grant"
            );
        }
        let oversize = serde_json::json!({
            "path": "big.txt",
            "content": "x".repeat(MAX_WRITE_BYTES + 1),
        });
        let error = preflight(&jail, &oversize, None).expect_err("oversize");
        assert_eq!(error.code(), "E_FILE_TOO_LARGE");
        assert_eq!(executed_at(&conn), None);

        let prepared = preflight(&jail, &valid, None).expect("valid preflight");
        gate.verify_grant(&conn, run, ToolId::FileWrite, &valid, NOW)
            .expect("grant verified");
        assert!(
            executed_at(&conn).is_some(),
            "commit path consumes the grant"
        );
        drop(prepared);
    }

    #[tokio::test]
    async fn commit_writes_and_reports_path_size_sha_without_content() {
        let (temp, jail, conn, gate, run) = seeded();
        let content = "hello hive side effect";
        let args = serde_json::json!({"path": "notes/a.txt", "content": content});
        grant(&conn, &gate, run, &args, "file_write");
        let prepared = preflight(&jail, &args, None).expect("preflight");
        gate.verify_grant(&conn, run, ToolId::FileWrite, &args, NOW)
            .expect("grant");
        let outcome = commit_side_effect(&jail, &ShellPolicy::conservative_default(), prepared, None).await.expect("commit");
        let ExecutedTool::Ok { text } = outcome else {
            panic!("approved write must execute");
        };
        assert!(text.contains("wrote"), "{text}");
        assert!(text.contains("notes/a.txt"), "{text}");
        assert!(text.contains("sha256="), "{text}");
        assert!(
            !text.contains(content),
            "result text must not leak content: {text}"
        );
        assert_eq!(
            std::fs::read(temp.path().join("notes").join("a.txt")).expect("read back"),
            content.as_bytes()
        );
    }

    #[tokio::test]
    async fn commit_replaces_existing_file_atomically_without_temp_residue() {
        let (temp, jail, conn, gate, run) = seeded();
        std::fs::write(temp.path().join("notes.txt"), b"old").expect("old");
        let args = serde_json::json!({"path": "notes.txt", "content": "new"});
        grant(&conn, &gate, run, &args, "file_write");
        let prepared = preflight(&jail, &args, None).expect("preflight");
        gate.verify_grant(&conn, run, ToolId::FileWrite, &args, NOW)
            .expect("grant");
        commit_side_effect(&jail, &ShellPolicy::conservative_default(), prepared, None).await.expect("commit");
        assert_eq!(
            std::fs::read(temp.path().join("notes.txt")).expect("read back"),
            b"new"
        );
        let residue = std::fs::read_dir(temp.path())
            .expect("dir")
            .filter_map(Result::ok)
            .any(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".hive-write-")
            });
        assert!(!residue, "atomic replace left a temp file behind");
    }

    #[tokio::test]
    async fn stale_expected_sha256_accepts_match_and_refuses_mismatch_without_grant() {
        let (temp, jail, conn, gate, run) = seeded();
        std::fs::write(temp.path().join("notes.txt"), b"current").expect("current");
        let current_hash = sha256_hex(b"current");
        let matching = serde_json::json!({
            "path": "notes.txt",
            "content": "next",
            "expected_sha256": current_hash,
        });
        grant(&conn, &gate, run, &matching, "file_write");
        let prepared = preflight(&jail, &matching, None).expect("matching hash");
        gate.verify_grant(&conn, run, ToolId::FileWrite, &matching, NOW)
            .expect("grant");
        commit_side_effect(&jail, &ShellPolicy::conservative_default(), prepared, None).await.expect("commit");
        assert_eq!(
            std::fs::read(temp.path().join("notes.txt")).expect("read back"),
            b"next"
        );

        let mismatch = serde_json::json!({
            "path": "notes.txt",
            "content": "should-not-write",
            "expected_sha256": "f".repeat(64),
        });
        let error = preflight(&jail, &mismatch, None).expect_err("mismatch");
        assert_eq!(error.code(), "E_SCHEMA_INVALID");
        assert_eq!(
            std::fs::read(temp.path().join("notes.txt")).expect("unchanged"),
            b"next"
        );

        let malformed = serde_json::json!({
            "path": "notes.txt",
            "content": "should-not-write",
            "expected_sha256": "not-hex",
        });
        let error = preflight(&jail, &malformed, None).expect_err("malformed");
        assert_eq!(error.code(), "E_SCHEMA_INVALID");

        let missing = serde_json::json!({
            "path": "missing.txt",
            "content": "should-not-write",
            "expected_sha256": "a".repeat(64),
        });
        let error = preflight(&jail, &missing, None).expect_err("missing");
        assert_eq!(error.code(), "E_PATH_NOT_FOUND");
    }

    #[tokio::test]
    async fn grant_is_consumed_exactly_once() {
        let (temp, jail, conn, gate, run) = seeded();
        let args = serde_json::json!({"path": "once.txt", "content": "one"});
        grant(&conn, &gate, run, &args, "file_write");
        let prepared = preflight(&jail, &args, None).expect("preflight");
        gate.verify_grant(&conn, run, ToolId::FileWrite, &args, NOW)
            .expect("first grant");
        commit_side_effect(&jail, &ShellPolicy::conservative_default(), prepared, None).await.expect("commit");
        let error = gate
            .verify_grant(&conn, run, ToolId::FileWrite, &args, NOW)
            .expect_err("second grant must be refused");
        assert_eq!(error.code(), "E_APPROVAL_REPLAY");
        assert_eq!(
            std::fs::read(temp.path().join("once.txt")).expect("read back"),
            b"one"
        );
    }

    #[test]
    fn cancelled_token_refuses_before_grant() {
        let (temp, jail, _conn, _gate, _run) = seeded();
        let token = CancellationToken::new();
        token.cancel();
        let args = serde_json::json!({"path": "cancelled.txt", "content": "x"});
        let error = preflight(&jail, &args, Some(&token)).expect_err("cancelled");
        assert_eq!(error.code(), "E_TOOL_CANCELLED");
        assert!(!temp.path().join("cancelled.txt").exists());
    }

    #[test]
    fn result_text_is_bounded_on_char_boundary() {
        let text = format!("wrote {} bytes to notes.txt; sha256={}", 10, "a".repeat(64));
        assert_eq!(bounded_text(&text), text);
        let long = "é".repeat(MAX_SIDE_EFFECT_TEXT_BYTES);
        let bounded = bounded_text(&long);
        assert!(bounded.len() <= MAX_SIDE_EFFECT_TEXT_BYTES + 64);
        assert!(bounded.contains("[truncated at"));
        assert!(bounded.is_char_boundary(bounded.len().min(MAX_SIDE_EFFECT_TEXT_BYTES)));
    }

    #[test]
    fn shell_preflight_refuses_disallowed_program_and_bad_bounds() {
        let (_temp, jail, _conn, _gate, _run) = seeded();
        let registry = registry();
        let policy = ShellPolicy::conservative_default();
        let attempt = |args: serde_json::Value| {
            preflight_side_effect(
                &jail,
                &registry,
                &policy,
                AgentTier::WorkerAgent,
                "shell_exec",
                &args,
                None,
            )
        };
        let path_qualified = attempt(serde_json::json!({
            "program": "C:\\Windows\\cmd.exe",
            "args": []
        }))
        .expect_err("path-qualified");
        assert_eq!(path_qualified.code(), "E_TOOL_NOT_ALLOWED");

        let unlisted = attempt(serde_json::json!({
            "program": "powershell.exe",
            "args": []
        }))
        .expect_err("not allowlisted");
        assert_eq!(unlisted.code(), "E_TOOL_NOT_ALLOWED");

        let too_many = attempt(serde_json::json!({
            "program": "cmd.exe",
            "args": vec!["x"; MAX_SHELL_ARGS + 1]
        }))
        .expect_err("arg count");
        assert_eq!(too_many.code(), "E_SCHEMA_INVALID");

        let nul = attempt(serde_json::json!({
            "program": "cmd.exe",
            "args": ["a\0b"]
        }))
        .expect_err("NUL argument");
        assert_eq!(nul.code(), "E_SCHEMA_INVALID");

        let escaped_cwd = attempt(serde_json::json!({
            "program": "cmd.exe",
            "args": [],
            "cwd": "../outside"
        }))
        .expect_err("cwd escape");
        assert_eq!(escaped_cwd.code(), "E_PATH_OUTSIDE_JAIL");
    }

    #[tokio::test]
    async fn shell_commit_runs_allowlisted_program_once_after_grant() {
        let (_temp, jail, conn, gate, run) = seeded();
        let args = serde_json::json!({
            "program": "cmd.exe",
            "args": ["/C", "echo", "hive"],
        });
        grant(&conn, &gate, run, &args, "shell_exec");
        let registry = registry();
        let policy = ShellPolicy::conservative_default();
        let prepared = preflight_side_effect(
            &jail,
            &registry,
            &policy,
            AgentTier::WorkerAgent,
            "shell_exec",
            &args,
            None,
        )
        .expect("shell preflight");
        gate.verify_grant(&conn, run, ToolId::ShellExec, &args, NOW)
            .expect("shell grant");
        let outcome = commit_side_effect(&jail, &policy, prepared, None)
            .await
            .expect("shell commit");
        let ExecutedTool::Ok { text } = outcome else {
            panic!("approved shell must execute");
        };
        assert!(text.contains("exit_code=Some(0)"), "{text}");
        assert!(text.contains("success=true"), "{text}");
        assert!(text.contains("hive"), "stdout must be visible to the model: {text}");
        assert!(!text.contains("/C"), "argv must not leak into tool-result text: {text}");

        let error = gate
            .verify_grant(&conn, run, ToolId::ShellExec, &args, NOW)
            .expect_err("second shell grant must be refused");
        assert_eq!(error.code(), "E_APPROVAL_REPLAY");
    }
}
