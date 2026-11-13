//! Read-only log and diagnosis truth for the local HTTP surface.
//!
//! Two callers share this module:
//! - `GET /api/logs` (the operator/agent log tail), and
//! - `/diagnose` (audit directory + recent refusal metadata).
//!
//! The module owns the security boundary for log reads: a request may name only
//! one of four [`LogSource`] values; the file behind it is a fixed basename under
//! the configured `data_dir` (or the newest rolling audit file under
//! `data_dir/audit`); the returned body is at most [`MAX_LOG_BYTES`]; and every
//! returned byte passes [`redact_secrets`] first. It never accepts a path from
//! the caller, never follows a file outside `data_dir`, and never returns an
//! unredacted body.

use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use hiverouter_contract::Refusal;
use hiverouter_tools::audit::AuditRow;
use regex::Regex;
use serde::Serialize;

/// Schema id of the `/api/logs` payload.
pub const LOGS_SCHEMA: &str = "hiverouter.logs.v1";
/// Maximum bytes returned for one log tail (64 KiB).
pub const MAX_LOG_BYTES: u64 = 65_536;
/// Default number of trailing lines when `tail` is omitted.
pub const DEFAULT_TAIL_LINES: usize = 200;
/// Hard cap on the requested trailing line count (server clamp).
pub const MAX_TAIL_LINES: usize = 5_000;
/// Hard cap on recent refusal rows in `/diagnose` metadata.
pub const MAX_RECENT_REFUSALS: usize = 50;
/// Prefix of the rolling JSONL audit files written by `hiverouter-tools`.
pub const AUDIT_FILE_PREFIX: &str = "audit.jsonl";

/// The only log sources a request may name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogSource {
    /// Launcher runtime log (`launcher.log`).
    Launcher,
    /// Server stdout capture (`server.stdout.log`).
    Stdout,
    /// Server stderr capture (`server.stderr.log`).
    Stderr,
    /// Newest rolling JSONL audit file under `<data_dir>/audit`.
    Audit,
}

impl LogSource {
    /// Every accepted source, in declaration order.
    pub const ALL: [LogSource; 4] = [
        LogSource::Launcher,
        LogSource::Stdout,
        LogSource::Stderr,
        LogSource::Audit,
    ];

    /// Parses the exact wire value; unknown values are refused (never defaulted).
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "launcher" => Some(LogSource::Launcher),
            "stdout" => Some(LogSource::Stdout),
            "stderr" => Some(LogSource::Stderr),
            "audit" => Some(LogSource::Audit),
            _ => None,
        }
    }

    /// Stable wire label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            LogSource::Launcher => "launcher",
            LogSource::Stdout => "stdout",
            LogSource::Stderr => "stderr",
            LogSource::Audit => "audit",
        }
    }

    /// Fixed whitelisted basename under `data_dir` (`None` for the rolling
    /// audit directory, whose newest file is selected by the server).
    #[must_use]
    pub fn file_name(self) -> Option<&'static str> {
        match self {
            LogSource::Launcher => Some("launcher.log"),
            LogSource::Stdout => Some("server.stdout.log"),
            LogSource::Stderr => Some("server.stderr.log"),
            LogSource::Audit => None,
        }
    }
}

/// One named refusal from a bounded log/diagnosis read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogError {
    refusal: Refusal,
    detail: String,
}

impl LogError {
    fn new(refusal: Refusal, detail: impl Into<String>) -> Self {
        Self {
            refusal,
            detail: detail.into(),
        }
    }

    /// Named refusal code (`E_PATH_NOT_FOUND`, `E_PATH_OUTSIDE_JAIL`, ...).
    #[must_use]
    pub fn refusal(&self) -> Refusal {
        self.refusal
    }

    /// Non-secret detail for the HTTP body and the audit log.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

/// One bounded log tail.
#[derive(Debug, Clone, Serialize)]
pub struct LogTail {
    /// Payload schema id.
    pub schema: &'static str,
    /// Requested source label.
    pub source: &'static str,
    /// Whitelisted path relative to `data_dir` (forward slashes).
    pub file: String,
    /// Requested trailing line count after the server clamp.
    pub tail: usize,
    /// File size in bytes at read time (may exceed the returned body).
    pub total_bytes: u64,
    /// Returned (redacted) body size in bytes.
    pub returned_bytes: usize,
    /// Number of trailing lines in the returned body.
    pub line_count: usize,
    /// Lines omitted because of the line cap or the 64 KiB byte cap.
    pub lines_omitted: usize,
    /// True when the 64 KiB byte cap cut the start of the file.
    pub truncated: bool,
    /// Redacted body (never the raw file content).
    pub content: String,
}

/// Metadata for one whitelisted log file.
#[derive(Debug, Clone, Serialize)]
pub struct LogFileInfo {
    /// Stable source label.
    pub source: &'static str,
    /// Path relative to `data_dir` (forward slashes).
    pub file: String,
    /// Whether the file exists and is a regular file.
    pub exists: bool,
    /// File size in bytes when it exists.
    pub bytes: Option<u64>,
    /// Last modification time (Unix seconds) when it exists.
    pub modified_unix: Option<u64>,
}

/// One recent refusal row (selected fields only; audit rows carry no secrets).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecentRefusal {
    /// RFC3339 timestamp emitted by the JSON tracing layer.
    pub timestamp: String,
    /// Stable refusal code.
    pub code: String,
    /// Run id when the refusal is run-scoped.
    pub run_id: Option<String>,
    /// Tool id when the refusal is tool-scoped.
    pub tool: Option<String>,
    /// Permission class when the refusal is tool-scoped.
    pub permission_class: Option<String>,
}

/// Bounded scan of the newest audit file.
#[derive(Debug, Clone, Serialize)]
pub struct RefusalScan {
    /// Newest audit file name, when one exists.
    pub file: Option<String>,
    /// File size in bytes at read time.
    pub bytes: u64,
    /// True when only the last 64 KiB of the file was scanned.
    pub truncated: bool,
    /// Non-JSON lines skipped (a partial first line after a bounded seek is
    /// expected; corruption is reported rather than silently hidden).
    pub corrupt_lines: usize,
    /// Recent refusals, oldest first (bounded by the caller).
    pub recent: Vec<RecentRefusal>,
}

/// Canonical audit directory for a configured data directory.
#[must_use]
pub fn audit_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("audit")
}

/// Names of environment variables whose values must never appear in a log body.
const SECRET_ENV_VARS: &[&str] = &[
    "HIVEROUTER_LAUNCH_TOKEN",
    "HIVEROUTER_API_KEY",
    "DEEPSEEK_API_KEY",
    "OPENAI_API_KEY",
    "ANTHROPIC_API_KEY",
];

static BEARER_TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(bearer\s+)[A-Za-z0-9._~+/=-]{8,}")
        .expect("the bearer-token redaction pattern compiles")
});

static SK_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bsk-[A-Za-z0-9_-]{8,}\b").expect("the provider-key redaction pattern compiles")
});

static ENV_ASSIGNMENT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?im)\b([A-Z0-9_]*(?:API[_-]?KEY|TOKEN|SECRET|PASSWORD)[A-Z0-9_]*\s*=\s*)([^\s"']{6,})"#,
    )
    .expect("the env-assignment redaction pattern compiles")
});

static KEY_VALUE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)(["']?(?:api[_-]?key|apikey|secret|password|passwd|client[_-]?secret|access[_-]?token|refresh[_-]?token|auth[_-]?token|launch[_-]?token|provider[_-]?key|authorization|token)["']?\s*[:=]\s*["']?)([^"'\s,;&}]{4,})"#,
    )
    .expect("the key/value redaction pattern compiles")
});

static QUERY_SECRET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)([?&](?:launch|token|key|api_key)=)[^&\s"']+"#)
        .expect("the query-secret redaction pattern compiles")
});

/// Redacts key and bearer-token material from a log body.
///
/// Exact secret values are replaced first (the launcher/session token and the
/// configured provider key, when present in the process environment), then
/// structural patterns: `Bearer <token>`, `sk-...`, `*_API_KEY=...`/`*TOKEN=...`,
/// JSON/CLI `key: value` pairs, and `?launch=`/`?token=` query values.
#[must_use]
pub fn redact_secrets(text: &str, extra_secrets: &[String]) -> String {
    let mut redacted = text.to_owned();
    for secret in extra_secrets {
        let secret = secret.trim();
        if secret.len() >= 8 && redacted.contains(secret) {
            redacted = redacted.replace(secret, "[REDACTED]");
        }
    }
    redacted = BEARER_TOKEN
        .replace_all(&redacted, "${1}[REDACTED]")
        .into_owned();
    redacted = SK_KEY.replace_all(&redacted, "sk-[REDACTED]").into_owned();
    redacted = ENV_ASSIGNMENT
        .replace_all(&redacted, "${1}[REDACTED]")
        .into_owned();
    redacted = KEY_VALUE
        .replace_all(&redacted, "${1}[REDACTED]")
        .into_owned();
    QUERY_SECRET
        .replace_all(&redacted, "${1}[REDACTED]")
        .into_owned()
}

/// Secret values present in the process environment (never returned as-is).
#[must_use]
pub fn configured_secret_values() -> Vec<String> {
    SECRET_ENV_VARS
        .iter()
        .filter_map(|name| std::env::var(name).ok())
        .map(|value| value.trim().to_owned())
        .filter(|value| value.len() >= 8)
        .collect()
}

/// Reads one whitelisted log tail.
///
/// # Errors
/// - `E_SCHEMA_INVALID` when `tail` is zero;
/// - `E_PATH_NOT_FOUND` when the whitelisted file (or the audit directory) does
///   not exist;
/// - `E_PATH_OUTSIDE_JAIL` when a canonical target is not a direct child of the
///   configured directory (a symlink escape);
/// - `E_INTERNAL_ERROR` for other I/O failures.
pub fn read_log_tail(data_dir: &Path, source: LogSource, tail: usize) -> Result<LogTail, LogError> {
    read_log_tail_with_secrets(data_dir, source, tail, &[])
}

/// [`read_log_tail`] with extra in-process secret values to redact (for example
/// the per-boot WebSocket session token).
///
/// # Errors
/// See [`read_log_tail`].
pub fn read_log_tail_with_secrets(
    data_dir: &Path,
    source: LogSource,
    tail: usize,
    extra_secrets: &[String],
) -> Result<LogTail, LogError> {
    if tail == 0 {
        return Err(LogError::new(
            Refusal::SchemaInvalid,
            "tail must be at least 1 line",
        ));
    }
    let tail = tail.min(MAX_TAIL_LINES);
    let data_dir = canonical_dir(data_dir)?;
    let (path, expected_parent) = resolve_source(&data_dir, source)?;
    let canonical = fs::canonicalize(&path).map_err(|error| {
        LogError::new(
            Refusal::PathNotFound,
            format!(
                "cannot resolve {}: {error}",
                relative_label(&data_dir, &path)
            ),
        )
    })?;
    if canonical.parent() != Some(expected_parent.as_path()) {
        return Err(LogError::new(
            Refusal::PathOutsideJail,
            format!(
                "{} resolves outside its whitelisted directory",
                source.label()
            ),
        ));
    }
    let metadata = fs::metadata(&canonical).map_err(|error| {
        LogError::new(
            Refusal::PathNotFound,
            format!("cannot read {} metadata: {error}", source.label()),
        )
    })?;
    if !metadata.is_file() {
        return Err(LogError::new(
            Refusal::PathNotFound,
            format!("{} is not a regular file", source.label()),
        ));
    }
    let total_bytes = metadata.len();
    let (buffer, truncated) = read_last_bytes(&canonical, MAX_LOG_BYTES)?;
    let text = String::from_utf8_lossy(&buffer).into_owned();
    let all_lines: Vec<&str> = text.lines().collect();
    let line_count = all_lines.len();
    let selected = if line_count > tail {
        &all_lines[line_count - tail..]
    } else {
        all_lines.as_slice()
    };
    let raw = selected.join("\n");
    let mut secrets = configured_secret_values();
    secrets.extend(extra_secrets.iter().cloned());
    let redacted = redact_secrets(&raw, &secrets);
    // Redaction can expand a short secret into `[REDACTED]`. Enforce the byte
    // cap after redaction too, so the wire body is never larger than 64 KiB.
    let (content, redaction_truncated) = bound_bytes(&redacted, MAX_LOG_BYTES);
    let returned_lines = content.lines().count();
    Ok(LogTail {
        schema: LOGS_SCHEMA,
        source: source.label(),
        file: relative_label(&data_dir, &canonical),
        tail,
        total_bytes,
        returned_bytes: content.len(),
        line_count: returned_lines,
        lines_omitted: line_count.saturating_sub(returned_lines),
        truncated: truncated || redaction_truncated,
        content,
    })
}

/// Metadata for the three fixed log files plus the newest audit file.
///
/// A missing file is reported with `exists: false` rather than omitted, so the
/// diagnosis surface distinguishes "not created yet" from "not configured".
#[must_use]
pub fn log_file_metadata(data_dir: &Path) -> Vec<LogFileInfo> {
    let data_dir = fs::canonicalize(data_dir).unwrap_or_else(|_| data_dir.to_path_buf());
    let mut files = Vec::with_capacity(LogSource::ALL.len());
    for source in [LogSource::Launcher, LogSource::Stdout, LogSource::Stderr] {
        let Some(name) = source.file_name() else {
            continue;
        };
        let path = data_dir.join(name);
        files.push(metadata_for(&data_dir, source, &path));
    }
    match newest_audit_file(&audit_dir(&data_dir)) {
        Some(path) => files.push(metadata_for(&data_dir, LogSource::Audit, &path)),
        None => files.push(LogFileInfo {
            source: LogSource::Audit.label(),
            file: "audit/".to_owned(),
            exists: false,
            bytes: None,
            modified_unix: None,
        }),
    }
    files
}

/// Scans the newest audit file for recent named refusals.
///
/// Only the last [`MAX_LOG_BYTES`] are read, so `/diagnose` stays bounded even
/// when the audit log has grown for weeks. Non-JSON lines are counted, never
/// silently discarded: a partial first line after the bounded seek is expected,
/// but any other value is visible in `corrupt_lines`.
#[must_use]
pub fn recent_refusals(audit_dir: &Path, limit: usize) -> RefusalScan {
    let limit = limit.clamp(1, MAX_RECENT_REFUSALS);
    let Some(path) = newest_audit_file(audit_dir) else {
        return RefusalScan {
            file: None,
            bytes: 0,
            truncated: false,
            corrupt_lines: 0,
            recent: Vec::new(),
        };
    };
    let bytes = fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0);
    let (buffer, truncated) = match read_last_bytes(&path, MAX_LOG_BYTES) {
        Ok(value) => value,
        Err(_) => {
            return RefusalScan {
                file: path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned()),
                bytes,
                truncated: false,
                corrupt_lines: 1,
                recent: Vec::new(),
            };
        }
    };
    let text = String::from_utf8_lossy(&buffer);
    let mut corrupt_lines = 0usize;
    let mut rows = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<AuditRow>(line) {
            Ok(row) => {
                if row.message == "refusal" {
                    let Some(code) = row.code.filter(|code| !code.trim().is_empty()) else {
                        continue;
                    };
                    rows.push(RecentRefusal {
                        timestamp: row.timestamp,
                        code,
                        run_id: row.run_id,
                        tool: row.tool.filter(|tool| !tool.trim().is_empty()),
                        permission_class: row
                            .permission_class
                            .filter(|class| !class.trim().is_empty()),
                    });
                }
            }
            Err(_) => corrupt_lines += 1,
        }
    }
    let recent = rows
        .into_iter()
        .rev()
        .take(limit)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    RefusalScan {
        file: path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned()),
        bytes,
        truncated,
        corrupt_lines,
        recent,
    }
}

fn canonical_dir(data_dir: &Path) -> Result<PathBuf, LogError> {
    fs::canonicalize(data_dir).map_err(|error| {
        LogError::new(
            Refusal::PathNotFound,
            format!("cannot resolve the data directory: {error}"),
        )
    })
}

fn resolve_source(data_dir: &Path, source: LogSource) -> Result<(PathBuf, PathBuf), LogError> {
    match source {
        LogSource::Audit => {
            let dir = audit_dir(data_dir);
            let canonical = fs::canonicalize(&dir).map_err(|error| {
                LogError::new(
                    Refusal::PathNotFound,
                    format!("no audit directory is readable: {error}"),
                )
            })?;
            if !canonical.starts_with(data_dir) {
                return Err(LogError::new(
                    Refusal::PathOutsideJail,
                    "the audit directory resolves outside the data directory",
                ));
            }
            let newest = newest_audit_file(&canonical).ok_or_else(|| {
                LogError::new(
                    Refusal::PathNotFound,
                    "no audit.jsonl.* file exists in the audit directory",
                )
            })?;
            Ok((newest, canonical))
        }
        _ => {
            let name = source
                .file_name()
                .expect("non-audit sources have a fixed whitelisted basename");
            let path = data_dir.join(name);
            if !path.exists() {
                return Err(LogError::new(
                    Refusal::PathNotFound,
                    format!("{name} does not exist under the data directory"),
                ));
            }
            Ok((path, data_dir.to_path_buf()))
        }
    }
}

fn newest_audit_file(dir: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;
    let mut best: Option<(u128, String, PathBuf)> = None;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with(AUDIT_FILE_PREFIX) {
            continue;
        }
        let path = entry.path();
        let metadata = match fs::metadata(&path) {
            Ok(metadata) if metadata.is_file() => metadata,
            _ => continue,
        };
        let modified = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let replace = match &best {
            None => true,
            Some((best_modified, best_name, _)) => {
                modified > *best_modified || (modified == *best_modified && name > *best_name)
            }
        };
        if replace {
            best = Some((modified, name, path));
        }
    }
    best.map(|(_, _, path)| path)
}

fn metadata_for(data_dir: &Path, source: LogSource, path: &Path) -> LogFileInfo {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_file() => LogFileInfo {
            source: source.label(),
            file: relative_label(data_dir, path),
            exists: true,
            bytes: Some(metadata.len()),
            modified_unix: metadata.modified().ok().and_then(unix_secs),
        },
        _ => LogFileInfo {
            source: source.label(),
            file: relative_label(data_dir, path),
            exists: false,
            bytes: None,
            modified_unix: None,
        },
    }
}

fn unix_secs(time: std::time::SystemTime) -> Option<u64> {
    time.duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs())
}

fn relative_label(data_dir: &Path, path: &Path) -> String {
    path.strip_prefix(data_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Truncates text to at most `max` bytes on a UTF-8 character boundary.
fn bound_bytes(text: &str, max: u64) -> (String, bool) {
    let max = usize::try_from(max).unwrap_or(usize::MAX);
    if text.len() <= max {
        return (text.to_owned(), false);
    }
    let mut end = max;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    (text[..end].to_owned(), true)
}

/// Reads at most `max` bytes from the end of `path`.
///
/// Returns `(bytes, truncated)`. When the start was cut, the leading partial
/// line is dropped so the returned body never begins with a fragment.
fn read_last_bytes(path: &Path, max: u64) -> Result<(Vec<u8>, bool), LogError> {
    let mut file = File::open(path).map_err(|error| {
        LogError::new(
            Refusal::PathNotFound,
            format!("cannot open {}: {error}", path.display()),
        )
    })?;
    let length = file
        .metadata()
        .map_err(|error| {
            LogError::new(
                Refusal::InternalError,
                format!("cannot stat {}: {error}", path.display()),
            )
        })?
        .len();
    let start = length.saturating_sub(max);
    if start > 0 {
        file.seek(SeekFrom::Start(start)).map_err(|error| {
            LogError::new(
                Refusal::InternalError,
                format!("cannot seek {}: {error}", path.display()),
            )
        })?;
    }
    let mut buffer = Vec::new();
    (&mut file)
        .take(max)
        .read_to_end(&mut buffer)
        .map_err(|error| {
            LogError::new(
                Refusal::InternalError,
                format!("cannot read {}: {error}", path.display()),
            )
        })?;
    if start > 0 {
        match buffer.iter().position(|byte| *byte == b'\n') {
            Some(newline) => {
                buffer.drain(..=newline);
            }
            None => buffer.clear(),
        }
    }
    Ok((buffer, start > 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_bearer_keys_env_assignments_and_query_tokens() {
        let text = concat!(
            "Authorization: Bearer abcdef1234567890\n",
            "provider key sk-live-abcdef1234567890\n",
            "DEEPSEEK_API_KEY=topsecretvalue123\n",
            "{\"api_key\":\"sk-jsonvalue123456\",\"token\":\"tok_123456789\"}\n",
            "launch http://127.0.0.1:8099/?launch=deadbeefdeadbeef\n"
        );
        let redacted = redact_secrets(text, &["session-secret-value".to_owned()]);
        assert!(!redacted.contains("abcdef1234567890"), "{redacted}");
        assert!(!redacted.contains("topsecretvalue123"), "{redacted}");
        assert!(!redacted.contains("sk-jsonvalue123456"), "{redacted}");
        assert!(!redacted.contains("tok_123456789"), "{redacted}");
        assert!(!redacted.contains("deadbeefdeadbeef"), "{redacted}");
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn redacts_exact_supplied_secrets_even_without_a_key_pattern() {
        let redacted = redact_secrets(
            "value=session-secret-value",
            &["session-secret-value".to_owned()],
        );
        assert!(!redacted.contains("session-secret-value"), "{redacted}");
    }

    #[test]
    fn tail_reads_only_the_last_lines_and_bounds_the_body() {
        let temp = tempfile::tempdir().expect("temp");
        let data = temp.path();
        std::fs::write(
            data.join("server.stdout.log"),
            b"first\nsecond\nthird\nfourth\n",
        )
        .expect("write");
        let tail = read_log_tail(data, LogSource::Stdout, 2).expect("tail");
        assert_eq!(tail.content, "third\nfourth");
        assert_eq!(tail.line_count, 2);
        assert_eq!(tail.lines_omitted, 2);
        assert!(!tail.truncated);
    }

    #[test]
    fn oversized_logs_are_capped_and_start_on_a_line_boundary() {
        let temp = tempfile::tempdir().expect("temp");
        let data = temp.path();
        let mut body = String::new();
        body.push_str("HEAD-MARKER\n");
        while body.len() < usize::try_from(MAX_LOG_BYTES * 2).expect("fits") {
            body.push_str("0123456789abcdef0123456789abcdef\n");
        }
        body.push_str("TAIL-MARKER\n");
        std::fs::write(data.join("launcher.log"), body.as_bytes()).expect("write");
        let tail = read_log_tail(data, LogSource::Launcher, MAX_TAIL_LINES).expect("tail");
        assert!(tail.truncated);
        assert!(tail.returned_bytes <= usize::try_from(MAX_LOG_BYTES).expect("fits"));
        assert!(!tail.content.contains("HEAD-MARKER"));
        assert!(tail.content.contains("TAIL-MARKER"));
    }

    #[cfg(any(windows, unix))]
    #[test]
    fn symlink_escape_is_refused_by_canonical_parent() {
        // Windows symlink creation needs privileges; the test is skipped when
        // the platform refuses it, so it never fabricates a pass.
        let temp = tempfile::tempdir().expect("temp");
        let data = temp.path().join("data");
        std::fs::create_dir_all(&data).expect("data");
        let outside = temp.path().join("outside.log");
        std::fs::write(&outside, b"outside").expect("outside");
        let link = data.join("server.stderr.log");
        #[cfg(windows)]
        let created = std::os::windows::fs::symlink_file(&outside, &link);
        #[cfg(unix)]
        let created = std::os::unix::fs::symlink(&outside, &link);
        if created.is_err() {
            return;
        }
        let error = read_log_tail(&data, LogSource::Stderr, 10).expect_err("escape refused");
        assert_eq!(error.refusal(), Refusal::PathOutsideJail);
    }

    #[test]
    fn audit_scan_selects_only_audit_jsonl_files_and_reports_refusals() {
        let temp = tempfile::tempdir().expect("temp");
        let audit = temp.path().join("audit");
        std::fs::create_dir_all(&audit).expect("audit");
        std::fs::write(
            audit.join("audit.jsonl.2026-11-13"),
            concat!(
                "{\"timestamp\":\"2026-11-13T00:00:00Z\",\"level\":\"WARN\",\"message\":\"refusal\",\"code\":\"E_APPROVAL_TIMEOUT\",\"run_id\":\"run-1\",\"tool\":\"shell_exec\",\"permission_class\":\"execute\"}\n",
                "{\"timestamp\":\"2026-11-13T00:01:00Z\",\"level\":\"INFO\",\"message\":\"tool_call\",\"code\":\"\"}\n"
            ),
        )
        .expect("audit");
        std::fs::write(audit.join("not-audit.txt"), b"ignored").expect("noise");
        let scan = recent_refusals(&audit, 10);
        assert_eq!(scan.file.as_deref(), Some("audit.jsonl.2026-11-13"));
        assert_eq!(scan.recent.len(), 1);
        assert_eq!(scan.recent[0].code, "E_APPROVAL_TIMEOUT");
        assert_eq!(scan.recent[0].tool.as_deref(), Some("shell_exec"));
        assert_eq!(scan.corrupt_lines, 0);
    }
}
