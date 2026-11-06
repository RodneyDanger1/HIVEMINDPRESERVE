//! Append-only run ledger (Phase 6 WP 6.9).
//!
//! Every tool call, approval decision, refusal and partial verdict is written as
//! one JSON line with: `run_id`, optional `task_id`, `tool_call_id`, tool id,
//! permission class, `args_sha256` (never the argument body), outcome, refusal
//! code and optional partial verdict. Lines are appended and flushed
//! immediately; the file is the run's agent-use telemetry and is local only.
//!
//! Replay/undo safety: the ledger has no update or delete operation — the only
//! inverse is deleting the file, which is an explicit operator action.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use hiverouter_contract::{Refusal, RunId, TaskId, Timestamp};
use serde::{Deserialize, Serialize};

use crate::approval::{ApprovalDecision, ApprovalResolution};
use crate::error::ToolError;
use crate::tool::{PermissionClass, ToolId};

/// Default ledger file name inside the ledger directory.
pub const LEDGER_FILE_NAME: &str = "run-ledger.jsonl";

/// Outcome recorded for one tool call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LedgerOutcome {
    /// Approval requested before a side effect.
    ApprovalRequested,
    /// Approval granted.
    Allowed,
    /// Approval denied.
    Denied,
    /// Tool executed (side effect committed).
    Executed,
    /// Tool call refused before any side effect.
    Refused,
    /// Tool timed out and was terminated.
    TimedOut,
    /// Tool call cancelled by run cancellation.
    Cancelled,
    /// Tool failed for an internal reason.
    Failed,
}

impl LedgerOutcome {
    /// Stable label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            LedgerOutcome::ApprovalRequested => "approval_requested",
            LedgerOutcome::Allowed => "allowed",
            LedgerOutcome::Denied => "denied",
            LedgerOutcome::Executed => "executed",
            LedgerOutcome::Refused => "refused",
            LedgerOutcome::TimedOut => "timed_out",
            LedgerOutcome::Cancelled => "cancelled",
            LedgerOutcome::Failed => "failed",
        }
    }

    /// Whether this outcome must carry a named refusal code.
    #[must_use]
    pub fn requires_refusal_code(self) -> bool {
        matches!(
            self,
            LedgerOutcome::Denied
                | LedgerOutcome::Refused
                | LedgerOutcome::TimedOut
                | LedgerOutcome::Cancelled
                | LedgerOutcome::Failed
        )
    }
}

/// One immutable ledger line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerRow {
    /// Monotonic sequence (per ledger file).
    pub seq: u64,
    /// Record time (UTC RFC3339).
    pub ts: String,
    /// Run id.
    pub run_id: String,
    /// Task id, when the call belongs to a task.
    pub task_id: Option<String>,
    /// Provider tool-call id.
    pub tool_call_id: String,
    /// Tool id.
    pub tool: String,
    /// Permission class label.
    pub permission_class: String,
    /// SHA-256 of the exact arguments (never the body).
    pub args_sha256: String,
    /// Outcome label.
    pub outcome: LedgerOutcome,
    /// Named refusal code for refused/denied/timed-out/cancelled/failed rows.
    pub refusal_code: Option<String>,
    /// Optional partial verdict text (bounded by the caller).
    pub partial_verdict: Option<String>,
}

/// What the caller records for one ledger line.
#[derive(Debug, Clone)]
pub struct LedgerRecord<'a> {
    /// Run the call belongs to.
    pub run_id: RunId,
    /// Task the call belongs to, when applicable.
    pub task_id: Option<TaskId>,
    /// Provider tool-call id.
    pub tool_call_id: &'a str,
    /// Tool called.
    pub tool: ToolId,
    /// Permission class of the tool.
    pub permission_class: PermissionClass,
    /// SHA-256 of the exact arguments.
    pub args_sha256: &'a str,
    /// Outcome.
    pub outcome: LedgerOutcome,
    /// Named refusal for refusing outcomes.
    pub refusal: Option<Refusal>,
    /// Optional partial verdict.
    pub partial_verdict: Option<&'a str>,
}

/// Append-only JSONL run ledger.
#[derive(Debug)]
pub struct RunLedger {
    path: PathBuf,
    write_lock: Mutex<()>,
    seq: AtomicU64,
}

impl RunLedger {
    /// Opens (creating parents and file when missing) the ledger at `path`.
    ///
    /// The per-file sequence continues from the rows already present, so a
    /// restarted process never rewrites `seq` 0 over an existing ledger. (The
    /// file is append-only; reopening must not reset its monotonic sequence.)
    ///
    /// # Errors
    /// [`Refusal::InternalError`] for I/O failures.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, ToolError> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                ToolError::new(
                    Refusal::InternalError,
                    format!("cannot create the ledger directory: {error}"),
                )
            })?;
        }
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|error| {
                ToolError::new(
                    Refusal::InternalError,
                    format!("cannot open the run ledger: {error}"),
                )
            })?;
        let seq = Self::existing_row_count(&path)?;
        Ok(Self {
            path,
            write_lock: Mutex::new(()),
            seq: AtomicU64::new(seq),
        })
    }

    /// Counts non-blank JSONL rows already in an existing ledger file.
    fn existing_row_count(path: &Path) -> Result<u64, ToolError> {
        let bytes = std::fs::read(path).map_err(|error| {
            ToolError::new(
                Refusal::InternalError,
                format!("cannot read the existing run ledger: {error}"),
            )
        })?;
        let rows = bytes
            .split(|byte| *byte == b'\n')
            .filter(|line| line.iter().any(|byte| !byte.is_ascii_whitespace()))
            .count();
        u64::try_from(rows).map_err(|_| {
            ToolError::new(
                Refusal::InternalError,
                "existing run ledger has too many rows to continue its sequence",
            )
        })
    }

    /// Opens `<dir>/run-ledger.jsonl`.
    ///
    /// # Errors
    /// See [`Self::open`].
    pub fn open_in(dir: impl AsRef<Path>) -> Result<Self, ToolError> {
        Self::open(dir.as_ref().join(LEDGER_FILE_NAME))
    }

    /// Ledger file path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Appends one row and flushes it to disk.
    ///
    /// # Errors
    /// [`Refusal::InternalError`] for I/O failures, invalid `args_sha256`, or a
    /// refusing outcome without a refusal code.
    pub fn record(&self, record: LedgerRecord<'_>) -> Result<LedgerRow, ToolError> {
        validate_sha256(record.args_sha256)?;
        if record.outcome.requires_refusal_code() && record.refusal.is_none() {
            return Err(ToolError::new(
                Refusal::InternalError,
                format!(
                    "ledger outcome {} must carry a named refusal code",
                    record.outcome.label()
                ),
            ));
        }
        let seq = self.seq.fetch_add(1, Ordering::SeqCst);
        let row = LedgerRow {
            seq,
            ts: Timestamp::now().as_str().to_owned(),
            run_id: record.run_id.to_string(),
            task_id: record.task_id.map(|task| task.to_string()),
            tool_call_id: record.tool_call_id.to_owned(),
            tool: record.tool.id().to_owned(),
            permission_class: record.permission_class.label().to_owned(),
            args_sha256: record.args_sha256.to_owned(),
            outcome: record.outcome,
            refusal_code: record.refusal.map(|refusal| refusal.code().to_owned()),
            partial_verdict: record.partial_verdict.map(ToOwned::to_owned),
        };
        let line = serde_json::to_string(&row).map_err(|error| {
            ToolError::new(
                Refusal::InternalError,
                format!("cannot serialise the ledger row: {error}"),
            )
        })?;

        let _guard = self
            .write_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut file: File = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|error| {
                ToolError::new(
                    Refusal::InternalError,
                    format!("cannot open the run ledger for append: {error}"),
                )
            })?;
        file.write_all(line.as_bytes()).and_then(|()| file.write_all(b"\n")).map_err(
            |error| {
                ToolError::new(
                    Refusal::InternalError,
                    format!("cannot append to the run ledger: {error}"),
                )
            },
        )?;
        file.flush().map_err(|error| {
            ToolError::new(
                Refusal::InternalError,
                format!("cannot flush the run ledger: {error}"),
            )
        })?;
        Ok(row)
    }

    /// Reads every row back (used by tests and `hiverouter` diagnosis).
    ///
    /// # Errors
    /// [`Refusal::InternalError`] when a line is not a valid ledger row.
    pub fn read_all(&self) -> Result<Vec<LedgerRow>, ToolError> {
        let file = File::open(&self.path).map_err(|error| {
            ToolError::new(
                Refusal::InternalError,
                format!("cannot read the run ledger: {error}"),
            )
        })?;
        let mut rows = Vec::new();
        for (index, line) in BufReader::new(file).lines().enumerate() {
            let line = line.map_err(|error| {
                ToolError::new(
                    Refusal::InternalError,
                    format!("cannot read ledger line {}: {error}", index + 1),
                )
            })?;
            if line.trim().is_empty() {
                continue;
            }
            let row: LedgerRow = serde_json::from_str(&line).map_err(|error| {
                ToolError::new(
                    Refusal::InternalError,
                    format!("ledger line {} is not valid JSON: {error}", index + 1),
                )
            })?;
            rows.push(row);
        }
        Ok(rows)
    }
}

fn validate_sha256(digest: &str) -> Result<(), ToolError> {
    if digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(ToolError::new(
            Refusal::InternalError,
            "args_sha256 must be 64 hexadecimal characters (argument bodies are never stored)",
        ))
    }
}

/// Appends the ledger row for one resolved approval decision (W12 tool loop).
///
/// `allow_once` / `allow_session` record [`LedgerOutcome::Allowed`]; `deny`
/// records [`LedgerOutcome::Denied`] carrying the named
/// `E_APPROVAL_REQUIRED` refusal (the side effect must not happen, and the
/// ledger rejects a denied row without a refusal code). The binding is copied
/// from the committed [`ApprovalResolution`], so exactly the approved
/// `args_sha256` is recorded — never the argument body.
///
/// # Errors
/// [`Refusal::InternalError`] when the ledger rejects the row (invalid digest or
/// missing refusal code) or an I/O failure occurs.
pub fn append_approval_decision(
    ledger: &RunLedger,
    resolution: &ApprovalResolution,
) -> Result<LedgerRow, ToolError> {
    let (outcome, refusal) = match resolution.decision {
        ApprovalDecision::AllowOnce | ApprovalDecision::AllowSession => {
            (LedgerOutcome::Allowed, None)
        }
        ApprovalDecision::Deny => (LedgerOutcome::Denied, Some(Refusal::ApprovalRequired)),
    };
    ledger.record(LedgerRecord {
        run_id: resolution.request.run_id,
        task_id: Some(resolution.request.task_id),
        tool_call_id: &resolution.request.tool_call_id,
        tool: resolution.request.tool,
        permission_class: resolution.request.permission_class,
        args_sha256: &resolution.request.args_sha256,
        outcome,
        refusal,
        partial_verdict: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approval::{hash_args, migrate, ApprovalDecision, ApprovalGate};
    use crate::tool::{PermissionClass, ToolId};

    fn ledger() -> (tempfile::TempDir, RunLedger) {
        let temp = tempfile::tempdir().expect("tempdir");
        let ledger = RunLedger::open_in(temp.path()).expect("ledger");
        (temp, ledger)
    }

    fn record<'a>(
        run_id: RunId,
        digest: &'a str,
        outcome: LedgerOutcome,
        refusal: Option<Refusal>,
    ) -> LedgerRecord<'a> {
        LedgerRecord {
            run_id,
            task_id: Some(TaskId::new()),
            tool_call_id: "call-1",
            tool: ToolId::FileWrite,
            permission_class: PermissionClass::Mutate,
            args_sha256: digest,
            outcome,
            refusal,
            partial_verdict: None,
        }
    }

    #[test]
    fn jsonl_roundtrips_with_monotonic_seq_and_no_arg_body() {
        let (_temp, ledger) = ledger();
        let run = RunId::new();
        let secret_args = serde_json::json!({"path": "a.txt", "content": "TOP-SECRET-BODY"});
        let digest = hash_args(&secret_args);

        let first = ledger
            .record(record(run, &digest, LedgerOutcome::ApprovalRequested, None))
            .expect("row 1");
        let second = ledger
            .record(record(
                run,
                &digest,
                LedgerOutcome::Refused,
                Some(Refusal::ApprovalRequired),
            ))
            .expect("row 2");
        assert_eq!(first.seq, 0);
        assert_eq!(second.seq, 1);
        assert_eq!(second.refusal_code.as_deref(), Some("E_APPROVAL_REQUIRED"));
        assert_eq!(second.permission_class, "mutate");
        assert_eq!(second.tool, "file_write");

        let rows = ledger.read_all().expect("read back");
        assert_eq!(rows, vec![first, second]);

        let raw = std::fs::read_to_string(ledger.path()).expect("raw");
        assert!(!raw.contains("TOP-SECRET-BODY"), "argument body never stored");
        for line in raw.lines() {
            let parsed: serde_json::Value = serde_json::from_str(line).expect("valid json line");
            assert!(parsed.get("run_id").is_some());
            assert!(parsed.get("args_sha256").is_some());
        }
    }

    #[test]
    fn refusing_outcome_without_code_is_rejected() {
        let (_temp, ledger) = ledger();
        let error = ledger
            .record(record(RunId::new(), &hash_args(&serde_json::json!({})), LedgerOutcome::Denied, None))
            .expect_err("missing code");
        assert_eq!(error.code(), "E_INTERNAL_ERROR");
    }

    #[test]
    fn raw_argument_body_cannot_be_smuggled_as_digest() {
        let (_temp, ledger) = ledger();
        let error = ledger
            .record(record(
                RunId::new(),
                "not-a-digest-but-argument-body",
                LedgerOutcome::Executed,
                None,
            ))
            .expect_err("invalid digest");
        assert_eq!(error.code(), "E_INTERNAL_ERROR");
    }

    #[test]
    fn append_only_file_survives_reopen() {
        let (temp, ledger) = ledger();
        let run = RunId::new();
        ledger
            .record(record(run, &hash_args(&serde_json::json!({"path": "x"})), LedgerOutcome::Executed, None))
            .expect("row");
        drop(ledger);
        let reopened = RunLedger::open_in(temp.path()).expect("reopen");
        let rows = reopened.read_all().expect("read");
        assert_eq!(rows.len(), 1);
        let next = reopened
            .record(record(run, &hash_args(&serde_json::json!({"path": "y"})), LedgerOutcome::Executed, None))
            .expect("row after reopen");
        assert_eq!(next.seq, 1, "seq continues after reopen; it never restarts at 0");
    }

    fn resolved(decision: ApprovalDecision, now: u64) -> (rusqlite::Connection, ApprovalResolution) {
        let conn = rusqlite::Connection::open_in_memory().expect("db");
        migrate(&conn).expect("migrate");
        let gate = ApprovalGate::default();
        let args = serde_json::json!({"path": "notes.txt", "content": "SECRET-BODY"});
        let request = gate
            .request(
                &conn,
                RunId::new(),
                TaskId::new(),
                "call-ledger-1",
                ToolId::FileWrite,
                PermissionClass::Mutate,
                &args,
                now,
            )
            .expect("request");
        let resolution = gate
            .decide(&conn, &request.approval_id.to_string(), decision, now)
            .expect("decision");
        (conn, resolution)
    }

    #[test]
    fn append_approval_decision_records_allowed_row_without_args_body() {
        let (_temp, ledger) = ledger();
        let (_conn, resolution) =
            resolved(ApprovalDecision::AllowOnce, 1_800_000_000);
        let row = append_approval_decision(&ledger, &resolution).expect("appended");
        assert_eq!(row.outcome, LedgerOutcome::Allowed);
        assert!(row.refusal_code.is_none());
        assert_eq!(row.tool, "file_write");
        assert_eq!(row.permission_class, "mutate");
        assert_eq!(row.tool_call_id, "call-ledger-1");
        assert_eq!(row.args_sha256, hash_args(&serde_json::json!({"path": "notes.txt", "content": "SECRET-BODY"})));
        assert_eq!(row.seq, 0);
        assert!(!row.ts.is_empty());
        assert_eq!(row.task_id, Some(resolution.request.task_id.to_string()));

        let raw = std::fs::read_to_string(ledger.path()).expect("raw");
        assert!(!raw.contains("SECRET-BODY"), "argument bodies are never ledgered");
        let rows = ledger.read_all().expect("read back");
        assert_eq!(rows, vec![row]);
    }

    #[test]
    fn append_approval_decision_allow_session_is_allowed() {
        let (_temp, ledger) = ledger();
        let (_conn, resolution) =
            resolved(ApprovalDecision::AllowSession, 1_800_000_000);
        let row = append_approval_decision(&ledger, &resolution).expect("appended");
        assert_eq!(row.outcome, LedgerOutcome::Allowed);
        assert!(row.refusal_code.is_none());
    }

    #[test]
    fn append_approval_decision_deny_carries_named_refusal() {
        let (_temp, ledger) = ledger();
        let (_conn, resolution) = resolved(ApprovalDecision::Deny, 1_800_000_000);
        let row = append_approval_decision(&ledger, &resolution).expect("appended");
        assert_eq!(row.outcome, LedgerOutcome::Denied);
        assert_eq!(row.refusal_code.as_deref(), Some("E_APPROVAL_REQUIRED"));
        assert_eq!(row.tool, "file_write");
        assert_eq!(row.args_sha256.len(), 64);
    }
}
