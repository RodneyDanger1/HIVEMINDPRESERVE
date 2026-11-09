//! Daemon plane: one-job records that are deliberately **not** task rows.
//!
//! A daemon is a deterministic, low-token office: a watchdog/report entry, a
//! relay heartbeat, or an operator switch. It never invokes a model, never
//! consumes run caps, and never appears in the run DAG. The supervisor lane
//! advances due daemons and records bounded reports; durable rows survive a
//! restart without turning into tasks.

use std::sync::Arc;

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
}

impl DaemonStore {
    #[must_use]
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
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
    /// No daemon job runner exists yet. A tick therefore marks the daemon
    /// `degraded` and records that fact instead of fabricating an `alive`
    /// success for work that never ran.
    pub fn tick_due(&self, limit: usize) -> Result<usize, Refusal> {
        let now_secs = Timestamp::now()
            .unix_secs()
            .map_err(|_| Refusal::InternalError)?;
        let now = Timestamp::from_unix_secs(now_secs).as_str().to_owned();
        self.db
            .with_connection(|connection| {
                let tx = connection.transaction()?;
                let mut ids: Vec<(String, i64, String)> = Vec::new();
                {
                    let mut statement = tx.prepare(
                        "SELECT daemon_id, interval_secs, job_json FROM daemons
                         WHERE state = 'running'
                           AND (next_tick_at IS NULL OR next_tick_at <= ?1)
                         ORDER BY next_tick_at ASC LIMIT ?2",
                    )?;
                    let rows = statement.query_map(
                        params![now, i64::try_from(limit).unwrap_or(10)],
                        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                    )?;
                    for row in rows {
                        ids.push(row?);
                    }
                }
                for (daemon_id, interval, job_json) in &ids {
                    let next = Timestamp::from_unix_secs(
                        now_secs.saturating_add(u64::try_from(*interval).unwrap_or(5)),
                    )
                    .as_str()
                    .to_owned();
                    let job: serde_json::Value =
                        serde_json::from_str(job_json).unwrap_or(serde_json::Value::Null);
                    let db_check = job.get("check").and_then(|value| value.as_str()) == Some("db");
                    if db_check {
                        let result: Result<i64, _> =
                            tx.query_row("SELECT 1", [], |row| row.get(0));
                        match result {
                            Ok(_) => {
                                tx.execute(
                                    "UPDATE daemons SET last_tick_at = ?2, last_report_at = ?2,
                                        next_tick_at = ?3, last_error_code = NULL
                                     WHERE daemon_id = ?1 AND state = 'running'",
                                    params![daemon_id, now, next],
                                )?;
                                tx.execute(
                                    "INSERT INTO daemon_reports
                                     (report_id, daemon_id, outcome, detail_json, created_at)
                                     VALUES (?1, ?2, 'ok', ?3, ?4)",
                                    params![
                                        format!("report:{}", Uuid::now_v7()),
                                        daemon_id,
                                        serde_json::json!({"executed": true, "check": "db"})
                                            .to_string(),
                                        now
                                    ],
                                )?;
                                continue;
                            }
                            Err(error) => {
                                tx.execute(
                                    "UPDATE daemons SET state = 'degraded', last_tick_at = ?2,
                                        last_report_at = ?2, next_tick_at = NULL,
                                        last_error_code = 'E_DAEMON_DB_CHECK'
                                     WHERE daemon_id = ?1 AND state = 'running'",
                                    params![daemon_id, now],
                                )?;
                                tx.execute(
                                    "INSERT INTO daemon_reports
                                     (report_id, daemon_id, outcome, detail_json, created_at)
                                     VALUES (?1, ?2, 'failed', ?3, ?4)",
                                    params![
                                        format!("report:{}", Uuid::now_v7()),
                                        daemon_id,
                                        serde_json::json!({
                                            "executed": true,
                                            "check": "db",
                                            "error": error.to_string(),
                                        })
                                        .to_string(),
                                        now
                                    ],
                                )?;
                                continue;
                            }
                        }
                    }
                    let reason = if job_json.trim().is_empty() || job_json.trim() == "{}" {
                        "daemon has no executable job"
                    } else {
                        "daemon job runtime is not implemented"
                    };
                    tx.execute(
                        "UPDATE daemons SET state = 'degraded', last_tick_at = ?2,
                            last_report_at = ?2, next_tick_at = NULL,
                            last_error_code = 'E_DAEMON_JOB_UNIMPLEMENTED'
                         WHERE daemon_id = ?1 AND state = 'running'",
                        params![daemon_id, now],
                    )?;
                    tx.execute(
                        "INSERT INTO daemon_reports
                         (report_id, daemon_id, outcome, detail_json, created_at)
                         VALUES (?1, ?2, 'degraded', ?3, ?4)",
                        params![
                            format!("report:{}", Uuid::now_v7()),
                            daemon_id,
                            serde_json::json!({
                                "executed": false,
                                "reason": reason,
                                "job": job_json,
                            })
                            .to_string(),
                            now
                        ],
                    )?;
                }
                tx.commit()?;
                Ok(ids.len())
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
