//! Daemon plane: one-job records that are deliberately **not** task rows.
//!
//! A daemon is a deterministic, low-token office: a watchdog/report entry, a
//! relay heartbeat, or an operator switch. It never invokes a model, never
//! consumes run caps, and never appears in the run DAG. The supervisor lane
//! advances due daemons and records bounded reports; durable rows survive a
//! restart without turning into tasks.

use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use hiverouter_contract::{Refusal, Timestamp};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use crate::db::Database;

/// One durable daemon record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Daemon {
    pub daemon_id: String,
    pub display_name: String,
    pub owner_kind: String,
    pub owner_id: String,
    pub kind: String,
    pub job_json: String,
    pub state: String,
    pub interval_secs: i64,
    pub next_tick_at: Option<String>,
    pub last_tick_at: Option<String>,
    pub last_report_at: Option<String>,
    pub restart_attempts: i64,
    pub last_error_code: Option<String>,
    pub created_by: String,
    pub created_at: String,
    pub retired_at: Option<String>,
}

/// One bounded daemon report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DaemonReport {
    pub report_id: String,
    pub daemon_id: String,
    pub outcome: String,
    pub detail_json: String,
    pub created_at: String,
}

struct JobOutcome {
    outcome: &'static str,
    error_code: Option<&'static str>,
    detail: serde_json::Value,
}

impl JobOutcome {
    fn ok(detail: serde_json::Value) -> Self {
        Self {
            outcome: "ok",
            error_code: None,
            detail,
        }
    }

    fn failed(code: &'static str, detail: serde_json::Value) -> Self {
        Self {
            outcome: "failed",
            error_code: Some(code),
            detail,
        }
    }

    fn degraded(code: &'static str, detail: serde_json::Value) -> Self {
        Self {
            outcome: "degraded",
            error_code: Some(code),
            detail,
        }
    }
}

/// Input for creating a daemon.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewDaemon {
    pub display_name: String,
    pub owner_kind: String,
    pub owner_id: String,
    pub kind: String,
    pub job_json: String,
    pub interval_secs: i64,
    pub created_by: String,
}

const DAEMON_SELECT: &str = "SELECT daemon_id, display_name, owner_kind, owner_id, kind,
    job_json, state, interval_secs, next_tick_at, last_tick_at, last_report_at,
    restart_attempts, last_error_code, created_by, created_at, retired_at FROM daemons";

fn daemon_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Daemon> {
    Ok(Daemon {
        daemon_id: row.get(0)?,
        display_name: row.get(1)?,
        owner_kind: row.get(2)?,
        owner_id: row.get(3)?,
        kind: row.get(4)?,
        job_json: row.get(5)?,
        state: row.get(6)?,
        interval_secs: row.get(7)?,
        next_tick_at: row.get(8)?,
        last_tick_at: row.get(9)?,
        last_report_at: row.get(10)?,
        restart_attempts: row.get(11)?,
        last_error_code: row.get(12)?,
        created_by: row.get(13)?,
        created_at: row.get(14)?,
        retired_at: row.get(15)?,
    })
}

/// Read/write facade for daemon records.
#[derive(Debug)]
pub struct DaemonStore {
    db: Arc<Database>,
    /// Operator-owned allowlist root for script daemons. Without it, script
    /// jobs are refused/degraded rather than executed from an arbitrary path.
    script_root: Option<PathBuf>,
}

impl DaemonStore {
    #[must_use]
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            script_root: None,
        }
    }

    /// Enables allowlisted daemon scripts rooted at `root`.
    #[must_use]
    pub fn with_script_root(mut self, root: PathBuf) -> Self {
        self.script_root = Some(root);
        self
    }

    /// Creates one daemon. It starts `enrolled`; only an explicit switch starts
    /// its clock. No task row is created.
    pub fn create(&self, input: NewDaemon) -> Result<Daemon, Refusal> {
        let name = input.display_name.trim();
        if name.is_empty()
            || name.chars().count() > 120
            || !matches!(input.owner_kind.as_str(), "operator" | "hive" | "remote")
            || input.owner_id.trim().is_empty()
            || !matches!(input.kind.as_str(), "watchdog" | "relay" | "switch" | "satellite")
            || !(5..=86_400).contains(&input.interval_secs)
        {
            return Err(Refusal::SchemaInvalid);
        }
        let daemon_id = format!("daemon:{}", Uuid::now_v7());
        let now = Timestamp::now().as_str().to_owned();
        self.db
            .with_connection(|connection| {
                connection.execute(
                    "INSERT INTO daemons
                     (daemon_id, display_name, owner_kind, owner_id, kind, job_json,
                      state, interval_secs, next_tick_at, last_tick_at, last_report_at,
                      restart_attempts, restart_window_at, last_error_code, created_by,
                      created_at, retired_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'enrolled', ?7, NULL, NULL, NULL,
                             0, NULL, NULL, ?8, ?9, NULL)",
                    params![
                        daemon_id,
                        name,
                        input.owner_kind,
                        input.owner_id,
                        input.kind,
                        input.job_json,
                        input.interval_secs,
                        input.created_by,
                        now,
                    ],
                )?;
                Ok(())
            })
            .map_err(map)?;
        self.get(&daemon_id)?.ok_or(Refusal::TaskNotFound)
    }

    pub fn get(&self, daemon_id: &str) -> Result<Option<Daemon>, Refusal> {
        self.db
            .with_connection(|connection| {
                connection
                    .query_row(
                        &format!("{DAEMON_SELECT} WHERE daemon_id = ?1"),
                        params![daemon_id],
                        daemon_from_row,
                    )
                    .optional()
            })
            .map_err(map)
    }

    pub fn list(&self) -> Result<Vec<Daemon>, Refusal> {
        self.db
            .with_connection(|connection| {
                let mut statement = connection.prepare(&format!(
                    "{DAEMON_SELECT} WHERE state <> 'retired' ORDER BY created_at ASC"
                ))?;
                let rows = statement.query_map([], daemon_from_row)?;
                let mut daemons = Vec::new();
                for row in rows {
                    daemons.push(row?);
                }
                Ok(daemons)
            })
            .map_err(map)
    }

    /// Operator switch: start (schedule now) or stop (clear next tick).
    pub fn set_enabled(&self, daemon_id: &str, enabled: bool) -> Result<Daemon, Refusal> {
        let now_secs = Timestamp::now()
            .unix_secs()
            .map_err(|_| Refusal::InternalError)?;
        let now = Timestamp::from_unix_secs(now_secs).as_str().to_owned();
        let changed = self
            .db
            .with_connection(|connection| {
                let state = if enabled { "running" } else { "stopped" };
                let next = if enabled {
                    Some(now.clone())
                } else {
                    None
                };
                connection.execute(
                    "UPDATE daemons SET state = ?2, next_tick_at = ?3, retired_at = NULL,
                        last_error_code = NULL
                     WHERE daemon_id = ?1 AND state <> 'retired'",
                    params![daemon_id, state, next],
                )
            })
            .map_err(map)?;
        if changed == 0 {
            return Err(Refusal::TaskNotFound);
        }
        self.get(daemon_id)?.ok_or(Refusal::TaskNotFound)
    }

    /// Retires a daemon. Its records remain for audit; it never runs again.
    pub fn retire(&self, daemon_id: &str) -> Result<(), Refusal> {
        let now = Timestamp::now().as_str().to_owned();
        let changed = self
            .db
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE daemons SET state = 'retired', next_tick_at = NULL, retired_at = ?2
                     WHERE daemon_id = ?1 AND state <> 'retired'",
                    params![daemon_id, now],
                )
            })
            .map_err(map)?;
        if changed == 0 {
            return Err(Refusal::TaskNotFound);
        }
        Ok(())
    }

    /// Advances every due daemon exactly once. The supervisor owns the cadence;
    /// this method is deterministic and never calls a model.
    ///
    /// The row is selected first, then the job runs outside the database lock.
    /// DB checks and allowlisted scripts report only what actually ran.
    pub fn tick_due(&self, limit: usize) -> Result<usize, Refusal> {
        let now_secs = Timestamp::now()
            .unix_secs()
            .map_err(|_| Refusal::InternalError)?;
        let now = Timestamp::from_unix_secs(now_secs).as_str().to_owned();
        let due: Vec<(String, i64, String)> = self
            .db
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    "SELECT daemon_id, interval_secs, job_json FROM daemons
                     WHERE state = 'running'
                       AND (next_tick_at IS NULL OR next_tick_at <= ?1)
                     ORDER BY next_tick_at ASC LIMIT ?2",
                )?;
                let rows = statement.query_map(
                    params![now, i64::try_from(limit).unwrap_or(10)],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )?;
                let mut due = Vec::new();
                for row in rows {
                    due.push(row?);
                }
                Ok(due)
            })
            .map_err(map)?;
        for (daemon_id, interval, job_json) in &due {
            let outcome = self.execute_daemon_job(job_json);
            self.record_daemon_tick(daemon_id, *interval, now_secs, &outcome)?;
        }
        Ok(due.len())
    }

    fn execute_daemon_job(&self, job_json: &str) -> JobOutcome {
        let job: serde_json::Value =
            serde_json::from_str(job_json).unwrap_or(serde_json::Value::Null);
        if job.get("check").and_then(|value| value.as_str()) == Some("db") {
            return match self
                .db
                .with_connection(|connection| connection.query_row("SELECT 1", [], |row| row.get::<_, i64>(0)))
            {
                Ok(_) => JobOutcome::ok(serde_json::json!({"executed": true, "check": "db"})),
                Err(error) => JobOutcome::failed(
                    "E_DAEMON_DB_CHECK",
                    serde_json::json!({"executed": true, "check": "db", "error": error.to_string()}),
                ),
            };
        }
        if job.get("kind").and_then(|value| value.as_str()) == Some("script") {
            return self.execute_script_job(&job);
        }
        let reason = if job_json.trim().is_empty() || job_json.trim() == "{}" {
            "daemon has no executable job"
        } else {
            "daemon job runtime is not implemented for this job kind"
        };
        JobOutcome::degraded(
            "E_DAEMON_JOB_UNIMPLEMENTED",
            serde_json::json!({"executed": false, "reason": reason, "job": job_json}),
        )
    }

    fn execute_script_job(&self, job: &serde_json::Value) -> JobOutcome {
        let Some(root) = self.script_root.as_deref() else {
            return JobOutcome::degraded(
                "E_DAEMON_SCRIPT_UNCONFIGURED",
                serde_json::json!({"executed": false, "reason": "no daemon script root is configured"}),
            );
        };
        let root = match root.canonicalize() {
            Ok(root) => root,
            Err(error) => {
                return JobOutcome::degraded(
                    "E_DAEMON_SCRIPT_REFUSED",
                    serde_json::json!({"executed": false, "error": error.to_string()}),
                )
            }
        };
        let Some(program) = job.get("program").and_then(|value| value.as_str()) else {
            return JobOutcome::degraded(
                "E_DAEMON_SCRIPT_REFUSED",
                serde_json::json!({"executed": false, "reason": "script job has no program"}),
            );
        };
        if program.is_empty()
            || program.chars().count() > 200
            || program.contains('/')
            || program.contains('\\')
            || program.contains("..")
            || program.contains('\0')
        {
            return JobOutcome::degraded(
                "E_DAEMON_SCRIPT_REFUSED",
                serde_json::json!({"executed": false, "reason": "program must be a plain file name"}),
            );
        }
        let canonical = match root.join(program).canonicalize() {
            Ok(path) if path.starts_with(&root) && path.is_file() => path,
            Ok(_) => {
                return JobOutcome::degraded(
                    "E_DAEMON_SCRIPT_REFUSED",
                    serde_json::json!({"executed": false, "reason": "program is outside the allowlist root"}),
                )
            }
            Err(error) => {
                return JobOutcome::degraded(
                    "E_DAEMON_SCRIPT_REFUSED",
                    serde_json::json!({"executed": false, "error": error.to_string()}),
                )
            }
        };
        let extension = canonical
            .extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase)
            .unwrap_or_default();
        if extension != "exe" && extension != "ps1" {
            return JobOutcome::degraded(
                "E_DAEMON_SCRIPT_REFUSED",
                serde_json::json!({"executed": false, "reason": "only .exe and .ps1 scripts are allowed"}),
            );
        }
        let mut args: Vec<String> = Vec::new();
        if let Some(raw) = job.get("args") {
            let Some(values) = raw.as_array() else {
                return JobOutcome::degraded(
                    "E_DAEMON_SCRIPT_REFUSED",
                    serde_json::json!({"executed": false, "reason": "args must be an array"}),
                );
            };
            if values.len() > 16 {
                return JobOutcome::degraded(
                    "E_DAEMON_SCRIPT_REFUSED",
                    serde_json::json!({"executed": false, "reason": "too many script arguments"}),
                );
            }
            for value in values {
                let Some(value) = value.as_str() else {
                    return JobOutcome::degraded(
                        "E_DAEMON_SCRIPT_REFUSED",
                        serde_json::json!({"executed": false, "reason": "script arguments must be strings"}),
                    );
                };
                if value.chars().count() > 512 || value.contains('\0') {
                    return JobOutcome::degraded(
                        "E_DAEMON_SCRIPT_REFUSED",
                        serde_json::json!({"executed": false, "reason": "script argument is too long or invalid"}),
                    );
                }
                args.push(value.to_owned());
            }
        }
        let timeout_secs = job
            .get("timeout_secs")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(15)
            .clamp(1, 60);
        let mut command = if extension == "exe" {
            Command::new(&canonical)
        } else {
            let mut command = Command::new("powershell.exe");
            command.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File"]);
            command.arg(&canonical);
            command
        };
        command
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // Daemon scripts are operator-created but must never inherit provider
        // credentials or unrelated server state (the launcher injects the key
        // into the server process).
        hiverouter_tools::env_safety::sanitize_std(&mut command);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x0800_0000);
        }
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(error) => {
                return JobOutcome::failed(
                    "E_DAEMON_SCRIPT_FAILED",
                    serde_json::json!({"executed": true, "error": error.to_string()}),
                )
            }
        };
        let stdout_handle = child.stdout.take().map(drain_capped);
        let stderr_handle = child.stderr.take().map(drain_capped);
        let deadline = Instant::now() + Duration::from_secs(timeout_secs);
        let mut timed_out = false;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Some(status),
                Ok(None) if Instant::now() >= deadline => {
                    timed_out = true;
                    kill_process_tree(&mut child);
                    break child.wait().ok();
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(25)),
                Err(error) => {
                    return JobOutcome::failed(
                        "E_DAEMON_SCRIPT_FAILED",
                        serde_json::json!({"executed": true, "error": error.to_string()}),
                    )
                }
            }
        };
        let (stdout, stdout_truncated) = join_reader(stdout_handle);
        let (stderr, stderr_truncated) = join_reader(stderr_handle);
        let exit_code = status.and_then(|status| status.code());
        let detail = serde_json::json!({
            "executed": true,
            "kind": "script",
            "program": program,
            "exit_code": exit_code,
            "timed_out": timed_out,
            "stdout": stdout,
            "stderr": stderr,
            "stdout_truncated": stdout_truncated,
            "stderr_truncated": stderr_truncated,
        });
        if timed_out {
            return JobOutcome::failed("E_DAEMON_SCRIPT_TIMEOUT", detail);
        }
        if status.is_some_and(|status| status.success()) {
            JobOutcome::ok(detail)
        } else {
            JobOutcome::failed("E_DAEMON_SCRIPT_FAILED", detail)
        }
    }

    fn record_daemon_tick(
        &self,
        daemon_id: &str,
        interval: i64,
        now_secs: u64,
        outcome: &JobOutcome,
    ) -> Result<(), Refusal> {
        let now = Timestamp::from_unix_secs(now_secs).as_str().to_owned();
        let next = Timestamp::from_unix_secs(
            now_secs.saturating_add(u64::try_from(interval).unwrap_or(5)),
        )
        .as_str()
        .to_owned();
        let detail = outcome.detail.to_string();
        self.db
            .with_connection(|connection| {
                let tx = connection.unchecked_transaction()?;
                if outcome.outcome == "ok" {
                    tx.execute(
                        "UPDATE daemons SET last_tick_at = ?2, last_report_at = ?2,
                            next_tick_at = ?3, last_error_code = NULL
                         WHERE daemon_id = ?1 AND state = 'running'",
                        params![daemon_id, now, next],
                    )?;
                } else {
                    tx.execute(
                        "UPDATE daemons SET state = 'degraded', last_tick_at = ?2,
                            last_report_at = ?2, next_tick_at = NULL,
                            last_error_code = ?3
                         WHERE daemon_id = ?1 AND state = 'running'",
                        params![daemon_id, now, outcome.error_code],
                    )?;
                }
                tx.execute(
                    "INSERT INTO daemon_reports
                     (report_id, daemon_id, outcome, detail_json, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        format!("report:{}", Uuid::now_v7()),
                        daemon_id,
                        outcome.outcome,
                        detail,
                        now
                    ],
                )?;
                tx.commit()?;
                Ok(())
            })
            .map_err(map)
    }
    pub fn reports(&self, daemon_id: &str, limit: usize) -> Result<Vec<DaemonReport>, Refusal> {
        self.db
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    "SELECT report_id, daemon_id, outcome, detail_json, created_at
                     FROM daemon_reports WHERE daemon_id = ?1
                     ORDER BY created_at DESC LIMIT ?2",
                )?;
                let rows = statement.query_map(
                    params![daemon_id, i64::try_from(limit).unwrap_or(50)],
                    |row| {
                        Ok(DaemonReport {
                            report_id: row.get(0)?,
                            daemon_id: row.get(1)?,
                            outcome: row.get(2)?,
                            detail_json: row.get(3)?,
                            created_at: row.get(4)?,
                        })
                    },
                )?;
                let mut reports = Vec::new();
                for row in rows {
                    reports.push(row?);
                }
                Ok(reports)
            })
            .map_err(map)
    }
}

fn join_reader(
    handle: Option<std::thread::JoinHandle<(String, bool)>>,
) -> (String, bool) {
    match handle {
        Some(handle) => handle.join().unwrap_or_default(),
        None => (String::new(), false),
    }
}

fn drain_capped(mut reader: impl Read + Send + 'static) -> std::thread::JoinHandle<(String, bool)> {
    std::thread::spawn(move || {
        const CAP: usize = 8 * 1024;
        let mut captured = Vec::new();
        let mut truncated = false;
        let mut buffer = [0_u8; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => {
                    if captured.len() < CAP {
                        let remaining = CAP - captured.len();
                        captured.extend_from_slice(&buffer[..read.min(remaining)]);
                    }
                    if captured.len() >= CAP {
                        truncated = true;
                    }
                }
                Err(_) => break,
            }
        }
        (String::from_utf8_lossy(&captured).into_owned(), truncated)
    })
}

fn kill_process_tree(child: &mut std::process::Child) {
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args([
                "/PID",
                &child.id().to_string(),
                "/T",
                "/F",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
}

fn map(error: crate::db::DbError) -> Refusal {
    error.refusal()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (DaemonStore, Arc<Database>) {
        let db = Arc::new(Database::open_in_memory().expect("db"));
        db.migrate().expect("migrate");
        (DaemonStore::new(db.clone()), db)
    }

    #[test]
    fn daemon_is_never_a_task_and_ticks_once_per_due_window() {
        let (store, db) = fixture();
        let daemon = store
            .create(NewDaemon {
                display_name: "Watchdog".to_owned(),
                owner_kind: "operator".to_owned(),
                owner_id: "operator".to_owned(),
                kind: "watchdog".to_owned(),
                job_json: "{}".to_owned(),
                interval_secs: 60,
                created_by: "operator".to_owned(),
            })
            .expect("create");
        assert_eq!(daemon.state, "enrolled");
        assert_eq!(store.tick_due(10).expect("tick"), 0);
        store.set_enabled(&daemon.daemon_id, true).expect("start");
        assert_eq!(store.tick_due(10).expect("tick"), 1);
        assert_eq!(store.tick_due(10).expect("not yet due"), 0);
        let reports = store.reports(&daemon.daemon_id, 10).expect("reports");
        assert_eq!(reports.len(), 1);
        assert_eq!(
            reports[0].outcome, "degraded",
            "a daemon with no executable job must not claim an alive success"
        );
        let fresh = store
            .get(&daemon.daemon_id)
            .expect("query")
            .expect("daemon");
        assert_eq!(fresh.state, "degraded");
        assert_eq!(
            fresh.last_error_code.as_deref(),
            Some("E_DAEMON_JOB_UNIMPLEMENTED")
        );
        let tasks: i64 = db
            .with_connection(|connection| {
                connection.query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))
            })
            .expect("task count");
        assert_eq!(tasks, 0, "a daemon must never become a task");
    }

    #[test]
    fn a_db_check_daemon_executes_and_reports_ok() {
        let (store, _db) = fixture();
        let daemon = store
            .create(NewDaemon {
                display_name: "Db watchdog".to_owned(),
                owner_kind: "operator".to_owned(),
                owner_id: "operator".to_owned(),
                kind: "watchdog".to_owned(),
                job_json: r#"{"check":"db"}"#.to_owned(),
                interval_secs: 60,
                created_by: "operator".to_owned(),
            })
            .expect("create");
        store.set_enabled(&daemon.daemon_id, true).expect("start");
        assert_eq!(store.tick_due(10).expect("tick"), 1);
        assert_eq!(store.tick_due(10).expect("not yet due"), 0);
        let reports = store.reports(&daemon.daemon_id, 10).expect("reports");
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].outcome, "ok");
        assert!(reports[0].detail_json.contains("\"executed\":true"));
        let fresh = store
            .get(&daemon.daemon_id)
            .expect("query")
            .expect("daemon");
        assert_eq!(fresh.state, "running");
        assert_eq!(fresh.last_error_code, None);
    }

    #[cfg(windows)]
    #[test]
    fn allowlisted_script_job_executes_and_reports_measured_output() {
        let (db_store, _db) = fixture();
        let root = tempfile::tempdir().expect("script root");
        std::fs::write(
            root.path().join("ok.ps1"),
            "param([string]$Message)\nWrite-Output $Message\nexit 0\n",
        )
        .expect("script");
        let store = db_store.with_script_root(root.path().to_path_buf());
        let daemon = store
            .create(NewDaemon {
                display_name: "Script".to_owned(),
                owner_kind: "operator".to_owned(),
                owner_id: "operator".to_owned(),
                kind: "watchdog".to_owned(),
                job_json: serde_json::json!({
                    "kind": "script",
                    "program": "ok.ps1",
                    "args": ["hello-from-script"],
                    "timeout_secs": 15
                })
                .to_string(),
                interval_secs: 60,
                created_by: "operator".to_owned(),
            })
            .expect("create");
        store.set_enabled(&daemon.daemon_id, true).expect("start");
        assert_eq!(store.tick_due(10).expect("tick"), 1);
        let reports = store.reports(&daemon.daemon_id, 10).expect("reports");
        assert_eq!(reports[0].outcome, "ok");
        assert!(reports[0].detail_json.contains("hello-from-script"));
        assert!(reports[0].detail_json.contains("\"timed_out\":false"));
        let fresh = store.get(&daemon.daemon_id).expect("get").expect("daemon");
        assert_eq!(fresh.state, "running");
    }

    #[cfg(windows)]
    #[test]
    fn script_job_cannot_escape_the_allowlist_root() {
        let (db_store, _db) = fixture();
        let root = tempfile::tempdir().expect("script root");
        let store = db_store.with_script_root(root.path().to_path_buf());
        let daemon = store
            .create(NewDaemon {
                display_name: "Escape".to_owned(),
                owner_kind: "operator".to_owned(),
                owner_id: "operator".to_owned(),
                kind: "watchdog".to_owned(),
                job_json: r#"{"kind":"script","program":"../escape.ps1"}"#.to_owned(),
                interval_secs: 60,
                created_by: "operator".to_owned(),
            })
            .expect("create");
        store.set_enabled(&daemon.daemon_id, true).expect("start");
        store.tick_due(10).expect("tick");
        let reports = store.reports(&daemon.daemon_id, 10).expect("reports");
        assert_eq!(reports[0].outcome, "degraded");
        let fresh = store.get(&daemon.daemon_id).expect("get").expect("daemon");
        assert_eq!(
            fresh.last_error_code.as_deref(),
            Some("E_DAEMON_SCRIPT_REFUSED")
        );
    }

    #[cfg(windows)]
    #[test]
    fn timed_out_script_is_killed_and_reported() {
        let (db_store, _db) = fixture();
        let root = tempfile::tempdir().expect("script root");
        std::fs::write(
            root.path().join("slow.ps1"),
            "Start-Sleep -Seconds 10\nexit 0\n",
        )
        .expect("script");
        let store = db_store.with_script_root(root.path().to_path_buf());
        let daemon = store
            .create(NewDaemon {
                display_name: "Slow".to_owned(),
                owner_kind: "operator".to_owned(),
                owner_id: "operator".to_owned(),
                kind: "watchdog".to_owned(),
                job_json: r#"{"kind":"script","program":"slow.ps1","timeout_secs":1}"#.to_owned(),
                interval_secs: 60,
                created_by: "operator".to_owned(),
            })
            .expect("create");
        store.set_enabled(&daemon.daemon_id, true).expect("start");
        store.tick_due(10).expect("tick");
        let reports = store.reports(&daemon.daemon_id, 10).expect("reports");
        assert_eq!(reports[0].outcome, "failed");
        assert!(reports[0].detail_json.contains("\"timed_out\":true"));
        let fresh = store.get(&daemon.daemon_id).expect("get").expect("daemon");
        assert_eq!(
            fresh.last_error_code.as_deref(),
            Some("E_DAEMON_SCRIPT_TIMEOUT")
        );
    }

    #[cfg(windows)]
    #[test]
    fn script_job_does_not_inherit_provider_key() {
        let (db_store, _db) = fixture();
        let root = tempfile::tempdir().expect("script root");
        std::fs::write(
            root.path().join("env-check.ps1"),
            "if ([string]::IsNullOrEmpty($env:DEEPSEEK_API_KEY)) { 'CLEAN' } else { 'LEAKED' }\n",
        )
        .expect("script");
        std::env::set_var("DEEPSEEK_API_KEY", "must-not-leak-to-daemon");
        let store = db_store.with_script_root(root.path().to_path_buf());
        let daemon = store
            .create(NewDaemon {
                display_name: "EnvCheck".to_owned(),
                owner_kind: "operator".to_owned(),
                owner_id: "operator".to_owned(),
                kind: "watchdog".to_owned(),
                job_json: r#"{"kind":"script","program":"env-check.ps1"}"#.to_owned(),
                interval_secs: 60,
                created_by: "operator".to_owned(),
            })
            .expect("create");
        store.set_enabled(&daemon.daemon_id, true).expect("start");
        store.tick_due(10).expect("tick");
        std::env::remove_var("DEEPSEEK_API_KEY");
        let reports = store.reports(&daemon.daemon_id, 10).expect("reports");
        assert_eq!(reports[0].outcome, "ok");
        assert!(reports[0].detail_json.contains("CLEAN"));
        assert!(!reports[0].detail_json.contains("must-not-leak-to-daemon"));
    }

    #[test]
    fn retired_daemon_cannot_be_re_enabled() {
        let (store, _db) = fixture();
        let daemon = store
            .create(NewDaemon {
                display_name: "Relay".to_owned(),
                owner_kind: "hive".to_owned(),
                owner_id: "root".to_owned(),
                kind: "relay".to_owned(),
                job_json: "{}".to_owned(),
                interval_secs: 30,
                created_by: "hyper:root".to_owned(),
            })
            .expect("create");
        store.retire(&daemon.daemon_id).expect("retire");
        assert_eq!(
            store.set_enabled(&daemon.daemon_id, true),
            Err(Refusal::TaskNotFound)
        );
        assert!(store.list().expect("list").is_empty());
    }
}
