//! Local-only JSONL audit sink (Phase 11 WPs 11.5/11.10/11.13, CANONICAL C21).
//!
//! One sink: `tracing-subscriber 0.3.23` JSON layer writing through
//! `tracing-appender 0.2.5` rolling non-blocking files. Local only — no OTel,
//! no Prometheus, no collector (D1 scope fence). Every refusal row carries
//! `run_id`/`code`/`tool`/`permission_class`; a file probe rejects key/token
//! substrings, and an audit file with such a substring fails the round-trip
//! test rather than being shipped.
//!
//! The non-blocking guard must be held for the process lifetime: dropping
//! [`AuditGuard`] flushes and stops the writer.

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use hiverouter_contract::{Refusal, RunId};
use serde::{Deserialize, Serialize};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use crate::error::ToolError;
use crate::tool::{PermissionClass, ToolId};

/// Rolling audit file prefix (daily rotation → `audit.jsonl.YYYY-MM-DD`).
pub const AUDIT_FILE_PREFIX: &str = "audit.jsonl";

/// Substrings that must never appear in an audit file (key/token material).
pub const FORBIDDEN_AUDIT_SUBSTRINGS: &[&str] = &["DEEPSEEK_API_KEY", "sk-", "Bearer "];

/// Holds the non-blocking writer guard for the process lifetime.
#[derive(Debug)]
pub struct AuditGuard {
    log_dir: PathBuf,
    _guard: WorkerGuard,
}

impl AuditGuard {
    /// Directory receiving the rolling audit files.
    #[must_use]
    pub fn log_dir(&self) -> &Path {
        &self.log_dir
    }
}

/// Installs the one JSON audit sink as the process tracing subscriber.
///
/// Returns a guard that must live for the process lifetime; dropping it stops
/// the non-blocking writer and flushes pending lines.
///
/// # Errors
/// [`Refusal::InternalError`] when the directory cannot be created or a global
/// subscriber is already installed (a second sink would be a duplicate owner).
pub fn init_json_audit(log_dir: impl AsRef<Path>) -> Result<AuditGuard, ToolError> {
    let log_dir = log_dir.as_ref();
    fs::create_dir_all(log_dir).map_err(|error| {
        ToolError::new(
            Refusal::InternalError,
            format!("cannot create the audit directory: {error}"),
        )
    })?;
    let log_dir = fs::canonicalize(log_dir).map_err(|error| {
        ToolError::new(
            Refusal::InternalError,
            format!("cannot canonicalise the audit directory: {error}"),
        )
    })?;

    let appender = tracing_appender::rolling::daily(&log_dir, AUDIT_FILE_PREFIX);
    let (writer, guard) = tracing_appender::non_blocking(appender);
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let json_layer = tracing_subscriber::fmt::layer()
        .json()
        .flatten_event(true)
        .with_ansi(false)
        .with_target(true)
        .with_current_span(true)
        .with_writer(writer);

    tracing_subscriber::registry()
        .with(filter)
        .with(json_layer)
        .try_init()
        .map_err(|error| {
            ToolError::new(
                Refusal::InternalError,
                format!(
                    "a tracing subscriber is already installed; the audit sink is the single \
                     owner and must be initialised once: {error}"
                ),
            )
        })?;

    Ok(AuditGuard {
        log_dir,
        _guard: guard,
    })
}

/// Emits one audit event for a named refusal (run/tool/permission context).
///
/// This is the only helper that produces refusal audit rows; it writes no key,
/// token or argument body.
pub fn record_refusal(
    run_id: RunId,
    refusal: Refusal,
    tool: Option<ToolId>,
    permission_class: Option<PermissionClass>,
) {
    tracing::warn!(
        target: "hiverouter.audit",
        run_id = %run_id,
        code = refusal.code(),
        retryable = refusal.retryable(),
        tool = tool.map(ToolId::id).unwrap_or(""),
        permission_class = permission_class.map(PermissionClass::label).unwrap_or(""),
        "refusal"
    );
}

/// Emits one audit event for a tool call (no argument body, only the hash/class).
pub fn record_tool_call(
    run_id: RunId,
    tool: ToolId,
    permission_class: PermissionClass,
    outcome: &str,
    refusal_code: Option<&str>,
) {
    tracing::info!(
        target: "hiverouter.audit",
        run_id = %run_id,
        tool = tool.id(),
        permission_class = permission_class.label(),
        outcome = outcome,
        code = refusal_code.unwrap_or(""),
        "tool_call"
    );
}

/// One parsed audit line (subset of fields the local tests and diagnosis check).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditRow {
    /// RFC3339 timestamp emitted by the JSON layer.
    pub timestamp: String,
    /// Log level.
    pub level: String,
    /// Message target.
    #[serde(default)]
    pub target: String,
    /// Message text.
    #[serde(default)]
    pub message: String,
    /// Run id when the event is run-scoped.
    #[serde(default)]
    pub run_id: Option<String>,
    /// Refusal code when the event is a refusal.
    #[serde(default)]
    pub code: Option<String>,
    /// Tool id when the event is tool-scoped.
    #[serde(default)]
    pub tool: Option<String>,
    /// Permission class when the event is tool-scoped.
    #[serde(default)]
    pub permission_class: Option<String>,
}

/// Lists audit files under `log_dir`, newest last.
///
/// # Errors
/// [`Refusal::InternalError`] for I/O failures.
pub fn list_audit_files(log_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>, ToolError> {
    let mut files = Vec::new();
    for entry in fs::read_dir(log_dir).map_err(|error| {
        ToolError::new(
            Refusal::InternalError,
            format!("cannot list the audit directory: {error}"),
        )
    })? {
        let entry = entry.map_err(|error| {
            ToolError::new(
                Refusal::InternalError,
                format!("cannot read an audit directory entry: {error}"),
            )
        })?;
        let path = entry.path();
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(AUDIT_FILE_PREFIX))
        {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

/// Reads every audit row across the rolling files.
///
/// # Errors
/// [`Refusal::InternalError`] when a line is not valid JSON (a corrupted audit
/// file is a failure, never silently skipped).
pub fn read_audit_rows(log_dir: impl AsRef<Path>) -> Result<Vec<AuditRow>, ToolError> {
    let mut rows = Vec::new();
    for path in list_audit_files(log_dir)? {
        let file = fs::File::open(&path).map_err(|error| {
            ToolError::new(
                Refusal::InternalError,
                format!("cannot open audit file: {error}"),
            )
        })?;
        for (index, line) in BufReader::new(file).lines().enumerate() {
            let line = line.map_err(|error| {
                ToolError::new(
                    Refusal::InternalError,
                    format!("cannot read audit line {}: {error}", index + 1),
                )
            })?;
            if line.trim().is_empty() {
                continue;
            }
            rows.push(serde_json::from_str(&line).map_err(|error| {
                ToolError::new(
                    Refusal::InternalError,
                    format!("audit line {} is not valid JSON: {error}", index + 1),
                )
            })?);
        }
    }
    Ok(rows)
}

/// Probes an audit file for forbidden key/token substrings.
///
/// # Errors
/// [`Refusal::InternalError`] when a forbidden substring is found (the file must
/// be treated as a leak: rotate/delete, never ship) or reading fails.
pub fn probe_audit_file(path: impl AsRef<Path>) -> Result<(), ToolError> {
    let content = fs::read_to_string(path.as_ref()).map_err(|error| {
        ToolError::new(
            Refusal::InternalError,
            format!("cannot read the audit file for the secret probe: {error}"),
        )
    })?;
    for forbidden in FORBIDDEN_AUDIT_SUBSTRINGS {
        if content.contains(forbidden) {
            return Err(ToolError::new(
                Refusal::InternalError,
                format!(
                    "audit file contains forbidden secret material substring {forbidden:?}; \
                     treat the file as leaked and rotate the key"
                ),
            ));
        }
    }
    Ok(())
}

/// Runs the secret probe across every audit file in `log_dir`.
///
/// # Errors
/// See [`probe_audit_file`].
pub fn probe_audit_dir(log_dir: impl AsRef<Path>) -> Result<(), ToolError> {
    for path in list_audit_files(log_dir)? {
        probe_audit_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn probe_rejects_key_material() {
        let temp = tempfile::tempdir().expect("tempdir");
        let clean = temp.path().join("audit.jsonl.2026-11-06");
        fs::write(&clean, "{\"level\":\"INFO\",\"message\":\"ok\"}\n").expect("write");
        probe_audit_file(&clean).expect("clean file passes");

        let leaky = temp.path().join("audit.jsonl.2026-11-07");
        let mut file = fs::File::create(&leaky).expect("create");
        writeln!(
            file,
            "{{\"message\":\"key sk-abcdef leaked\"}}"
        )
        .expect("write");
        let error = probe_audit_file(&leaky).expect_err("leak detected");
        assert_eq!(error.code(), "E_INTERNAL_ERROR");
        assert_eq!(
            list_audit_files(temp.path()).expect("list").len(),
            2,
            "only audit.* files are listed"
        );
    }

    #[test]
    fn read_rows_requires_valid_json() {
        let temp = tempfile::tempdir().expect("tempdir");
        let file = temp.path().join("audit.jsonl.2026-11-06");
        fs::write(
            &file,
            "{\"timestamp\":\"2026-11-06T00:00:00Z\",\"level\":\"WARN\",\"target\":\"hiverouter.audit\",\"message\":\"refusal\",\"run_id\":\"0192\",\"code\":\"E_CYCLE\",\"tool\":\"\",\"permission_class\":\"\"}\n",
        )
        .expect("write");
        let rows = read_audit_rows(temp.path()).expect("parse");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].code.as_deref(), Some("E_CYCLE"));
        assert_eq!(rows[0].level, "WARN");

        fs::write(&file, "not json\n").expect("write");
        assert!(read_audit_rows(temp.path()).is_err());
    }
}
