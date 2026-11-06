//! SQLite bootstrap and migration engine.
//!
//! Scope (blueprint Phase 9, WP 9.8): SQLite stores cache/metrics/usage/run rows;
//! the graph lives in memory plus versioned JSON snapshots. This module owns the
//! connection, pragmas and the ordered migration list; later phases add
//! migrations rather than editing earlier ones.

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use rusqlite::{Connection, OptionalExtension};
use thiserror::Error;

use hiverouter_contract::event::ToolOutcome;
use hiverouter_contract::{Refusal, RunId, TaskId, Timestamp};

/// One ordered schema migration with an exercised inverse.
#[derive(Clone, Copy, Debug)]
pub struct Migration {
    /// Monotonic schema version starting at 1.
    pub version: i64,
    /// Short migration name recorded in `schema_migrations`.
    pub name: &'static str,
    /// Forward SQL.
    pub up: &'static str,
    /// Inverse SQL (exercised by the migration-roundtrip test).
    pub down: &'static str,
}

/// Ordered migrations. Append only; never edit a migration that has shipped.
pub const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "bootstrap",
    up: r#"
        CREATE TABLE IF NOT EXISTS app_meta (
            key   TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL
        );
        INSERT INTO app_meta(key, value) VALUES ('schema_version', 'hiverouter.v1')
            ON CONFLICT(key) DO UPDATE SET value = excluded.value;
        INSERT INTO app_meta(key, value) VALUES ('bootstrapped_at', strftime('%Y-%m-%dT%H:%M:%SZ','now'))
            ON CONFLICT(key) DO NOTHING;
    "#,
    down: r#"
        DROP TABLE IF EXISTS app_meta;
    "#,
},
Migration {
    version: 2,
    name: "runs_tasks_chat",
    up: r#"
        CREATE TABLE IF NOT EXISTS runs (
            id          TEXT PRIMARY KEY NOT NULL,
            project_id  TEXT NOT NULL,
            prompt      TEXT NOT NULL,
            status      TEXT NOT NULL,
            error_code  TEXT,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_runs_status ON runs(status);
        CREATE TABLE IF NOT EXISTS tasks (
            id             TEXT PRIMARY KEY NOT NULL,
            run_id         TEXT NOT NULL,
            parent_task_id TEXT,
            role           TEXT NOT NULL,
            depth          INTEGER NOT NULL,
            spawn_seq      INTEGER NOT NULL,
            state          TEXT NOT NULL,
            rev            INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY(run_id) REFERENCES runs(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_tasks_run ON tasks(run_id);
        CREATE TABLE IF NOT EXISTS chat_messages (
            id         TEXT PRIMARY KEY NOT NULL,
            run_id     TEXT NOT NULL,
            role       TEXT NOT NULL,
            text       TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(run_id) REFERENCES runs(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_messages_run ON chat_messages(run_id);
        CREATE TABLE IF NOT EXISTS command_receipts (
            correlation_id TEXT PRIMARY KEY NOT NULL,
            run_id         TEXT NOT NULL,
            created_at     TEXT NOT NULL
        );
    "#,
    down: r#"
        DROP TABLE IF EXISTS command_receipts;
        DROP TABLE IF EXISTS chat_messages;
        DROP TABLE IF EXISTS tasks;
        DROP TABLE IF EXISTS runs;
    "#,
},
Migration {
    version: 3,
    name: "snapshot_index",
    up: r#"
        CREATE TABLE IF NOT EXISTS snapshots (
            run_id         TEXT NOT NULL,
            revision       INTEGER NOT NULL,
            path           TEXT NOT NULL,
            created_at     TEXT NOT NULL,
            schema_version TEXT NOT NULL,
            PRIMARY KEY (run_id, revision)
        );
    "#,
    down: r#"
        DROP TABLE IF EXISTS snapshots;
    "#,
},
Migration {
    version: 4,
    name: "tool_turns",
    up: r#"
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
    "#,
    down: r#"
        DROP TABLE IF EXISTS tool_turns;
        ALTER TABLE runs DROP COLUMN tool_calls;
        ALTER TABLE runs DROP COLUMN tool_turns;
    "#,
},
];

/// Database errors; each maps to a named refusal for the wire.
#[derive(Debug, Error)]
pub enum DbError {
    /// SQLite reported lock contention (`E_DB_BUSY`, retryable).
    #[error("E_DB_BUSY: the local SQLite database is locked: {0}")]
    Busy(String),
    /// Any other SQLite error.
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    /// Filesystem error while opening/creating the database.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Another thread panicked while holding the connection lock.
    #[error("database connection lock is poisoned")]
    Poisoned,
    /// The requested down-migration target is not a known version.
    #[error("unknown migration version {0}")]
    UnknownMigration(i64),
    /// A tool-turn row could not be stored or read truthfully (integrity defect,
    /// not a transport failure).
    #[error("invalid tool-turn persistence: {0}")]
    InvalidToolTurn(String),
}

impl DbError {
    /// Maps the error to the named refusal used on the wire.
    #[must_use]
    pub fn refusal(&self) -> Refusal {
        match self {
            DbError::Busy(_) => Refusal::DbBusy,
            DbError::Sqlite(rusqlite::Error::SqliteFailure(code, _))
                if matches!(
                    code.code,
                    rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
                ) =>
            {
                Refusal::DbBusy
            }
            _ => Refusal::InternalError,
        }
    }
}

/// The local SQLite database.
pub struct Database {
    path: PathBuf,
    connection: Mutex<Connection>,
}

impl fmt::Debug for Database {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Database").field("path", &self.path).finish()
    }
}

impl Database {
    /// Opens (creating if needed) the database file and applies pragmas.
    ///
    /// # Errors
    /// [`DbError`] when the file cannot be opened or configured.
    pub fn open(path: &Path) -> Result<Self, DbError> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let connection = Connection::open(path)?;
        Self::configure(&connection)?;
        Ok(Self {
            path: path.to_path_buf(),
            connection: Mutex::new(connection),
        })
    }

    /// Opens a fresh in-memory database (tests and throwaway sessions only).
    ///
    /// # Errors
    /// [`DbError`] when the in-memory database cannot be configured.
    pub fn open_in_memory() -> Result<Self, DbError> {
        let connection = Connection::open_in_memory()?;
        Self::configure(&connection)?;
        Ok(Self {
            path: PathBuf::from(":memory:"),
            connection: Mutex::new(connection),
        })
    }

    fn configure(connection: &Connection) -> Result<(), DbError> {
        connection.busy_timeout(std::time::Duration::from_millis(5_000))?;
        connection.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;",
        )?;
        Ok(())
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>, DbError> {
        self.connection.lock().map_err(|_| DbError::Poisoned)
    }

    /// Path of the database file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Current applied schema version (`0` for a fresh database).
    ///
    /// # Errors
    /// [`DbError`] when the query fails.
    pub fn schema_version(&self) -> Result<i64, DbError> {
        let connection = self.lock()?;
        Self::current_version(&connection)
    }

    fn current_version(connection: &Connection) -> Result<i64, DbError> {
        if !Self::table_exists(connection, "schema_migrations")? {
            return Ok(0);
        }
        let version: Option<i64> =
            connection.query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })?;
        Ok(version.unwrap_or(0))
    }

    fn table_exists(connection: &Connection, name: &str) -> Result<bool, DbError> {
        let tables: i64 = connection.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            rusqlite::params![name],
            |row| row.get(0),
        )?;
        Ok(tables > 0)
    }

    /// Applies every pending migration in order (idempotent).
    ///
    /// # Errors
    /// [`DbError`] when a migration fails; the failed migration is not recorded.
    pub fn migrate(&self) -> Result<i64, DbError> {
        let mut connection = self.lock()?;
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version    INTEGER PRIMARY KEY,
                name       TEXT NOT NULL,
                applied_at TEXT NOT NULL
            );",
        )?;
        let current = Self::current_version(&connection)?;
        for migration in MIGRATIONS {
            if migration.version <= current {
                continue;
            }
            let transaction = connection.transaction()?;
            transaction.execute_batch(migration.up)?;
            transaction.execute(
                "INSERT INTO schema_migrations(version, name, applied_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![
                    migration.version,
                    migration.name,
                    Timestamp::now().as_str()
                ],
            )?;
            transaction.commit()?;
        }
        Self::current_version(&connection)
    }

    /// Rolls back migrations down to (and including) `target` being the highest
    /// version kept. Used by the migration-roundtrip test.
    ///
    /// # Errors
    /// [`DbError`] when a down migration fails or `target` is unknown.
    pub fn migrate_down_to(&self, target: i64) -> Result<i64, DbError> {
        if target != 0 && !MIGRATIONS.iter().any(|m| m.version == target) {
            return Err(DbError::UnknownMigration(target));
        }
        let mut connection = self.lock()?;
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version    INTEGER PRIMARY KEY,
                name       TEXT NOT NULL,
                applied_at TEXT NOT NULL
            );",
        )?;
        loop {
            let current = Self::current_version(&connection)?;
            if current <= target {
                return Ok(current);
            }
            let migration = MIGRATIONS
                .iter()
                .find(|m| m.version == current)
                .ok_or(DbError::UnknownMigration(current))?;
            let transaction = connection.transaction()?;
            transaction.execute_batch(migration.down)?;
            transaction.execute(
                "DELETE FROM schema_migrations WHERE version = ?1",
                rusqlite::params![migration.version],
            )?;
            transaction.commit()?;
        }
    }

    /// Applied migrations as `(version, name, applied_at)`.
    ///
    /// # Errors
    /// [`DbError`] when the query fails.
    pub fn applied_migrations(&self) -> Result<Vec<(i64, String, String)>, DbError> {
        let connection = self.lock()?;
        if !Self::table_exists(&connection, "schema_migrations")? {
            return Ok(Vec::new());
        }
        let mut statement =
            connection.prepare("SELECT version, name, applied_at FROM schema_migrations ORDER BY version")?;
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        })?;
        let mut migrations = Vec::new();
        for row in rows {
            migrations.push(row?);
        }
        Ok(migrations)
    }

    /// Reads one `app_meta` value.
    ///
    /// # Errors
    /// [`DbError`] when the query fails.
    pub fn app_meta(&self, key: &str) -> Result<Option<String>, DbError> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT value FROM app_meta WHERE key = ?1")?;
        let mut rows = statement.query(rusqlite::params![key])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    /// Runs a closure with exclusive access to the connection.
    ///
    /// # Errors
    /// [`DbError`] when the lock is poisoned or the closure returns a SQLite
    /// error.
    pub fn with_connection<T>(
        &self,
        f: impl FnOnce(&mut Connection) -> rusqlite::Result<T>,
    ) -> Result<T, DbError> {
        let mut connection = self.lock()?;
        Ok(f(&mut connection)?)
    }

    /// Runs a closure that reports named refusals directly (tools-layer calls).
    ///
    /// This keeps a business refusal such as `E_APPROVAL_REPLAY` from being
    /// flattened into a generic database error by a transport wrapper.
    ///
    /// # Errors
    /// The closure's named refusal, or `E_INTERNAL_ERROR` when the lock is
    /// poisoned.
    pub fn with_connection_refusal<T>(
        &self,
        f: impl FnOnce(&mut Connection) -> Result<T, Refusal>,
    ) -> Result<T, Refusal> {
        let mut connection = self.lock().map_err(|_| Refusal::InternalError)?;
        f(&mut connection)
    }

    /// Persists one tool-call record for a run (1.2 tool loop, schema v4).
    ///
    /// `UNIQUE(run_id, tool_call_id)` makes a model tool call single-effect:
    /// recording the same provider tool-call id twice fails with a duplicate
    /// constraint error instead of silently duplicating side-effect history.
    ///
    /// # Errors
    /// [`DbError::Sqlite`] on SQLite failure (including the uniqueness
    /// constraint), [`DbError::InvalidToolTurn`] when a value cannot be stored
    /// truthfully (for example a byte count outside the SQLite INTEGER range),
    /// and [`DbError::Poisoned`] when the connection lock is poisoned.
    pub fn record_tool_turn(&self, turn: &ToolTurnRecord) -> Result<(), DbError> {
        let bytes = tool_turn_bytes_to_sql(turn.bytes)?;
        let id = turn.id.clone();
        let run_id = turn.run_id.to_string();
        let task_id = turn.task_id.to_string();
        let tool_call_id = turn.tool_call_id.clone();
        let tool = turn.tool.clone();
        let args_sha256 = turn.args_sha256.clone();
        let outcome = turn.outcome.as_str();
        let code = turn.code.clone();
        let sha256 = turn.sha256.clone();
        let turn_number = i64::from(turn.turn);
        let created_at = turn.created_at.as_str().to_owned();
        let connection = self.lock()?;
        connection.execute(
            "INSERT INTO tool_turns(
                 id, run_id, task_id, tool_call_id, tool, args_sha256,
                 outcome, code, bytes, sha256, turn, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                id,
                run_id,
                task_id,
                tool_call_id,
                tool,
                args_sha256,
                outcome,
                code,
                bytes,
                sha256,
                turn_number,
                created_at
            ],
        )?;
        Ok(())
    }

    /// Reads one run's persisted tool turns, ordered by turn then creation time.
    ///
    /// # Errors
    /// [`DbError`] when the query fails or a stored row cannot be decoded
    /// truthfully (invalid id/timestamp/outcome/negative byte count).
    pub fn tool_turns_for_run(&self, run_id: RunId) -> Result<Vec<ToolTurnRecord>, DbError> {
        let connection = self.lock()?;
        let mut statement = connection.prepare(
            "SELECT id, run_id, task_id, tool_call_id, tool, args_sha256,
                    outcome, code, bytes, sha256, turn, created_at
             FROM tool_turns
             WHERE run_id = ?1
             ORDER BY turn, created_at, id",
        )?;
        let rows = statement.query_map(rusqlite::params![run_id.to_string()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<i64>>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, i64>(10)?,
                row.get::<_, String>(11)?,
            ))
        })?;
        let mut records = Vec::new();
        for row in rows {
            let (
                id,
                run_raw,
                task_raw,
                tool_call_id,
                tool,
                args_sha256,
                outcome_raw,
                code,
                bytes,
                sha256,
                turn_raw,
                created_at_raw,
            ) = row?;
            let parsed_run = run_raw.parse::<RunId>().map_err(|_| {
                DbError::InvalidToolTurn(format!("invalid run_id `{run_raw}` in tool_turns"))
            })?;
            let parsed_task = task_raw.parse::<TaskId>().map_err(|_| {
                DbError::InvalidToolTurn(format!("invalid task_id `{task_raw}` in tool_turns"))
            })?;
            let parsed_outcome = ToolOutcome::from_str(&outcome_raw).ok_or_else(|| {
                DbError::InvalidToolTurn(format!(
                    "unknown tool outcome `{outcome_raw}` in tool_turns"
                ))
            })?;
            let parsed_bytes = tool_turn_bytes_from_sql(bytes)?;
            let parsed_turn = u32::try_from(turn_raw).map_err(|_| {
                DbError::InvalidToolTurn(format!("invalid turn {turn_raw} in tool_turns"))
            })?;
            let created_at = Timestamp::parse(&created_at_raw).map_err(|_| {
                DbError::InvalidToolTurn(format!(
                    "invalid created_at `{created_at_raw}` in tool_turns"
                ))
            })?;
            records.push(ToolTurnRecord {
                id,
                run_id: parsed_run,
                task_id: parsed_task,
                tool_call_id,
                tool,
                args_sha256,
                outcome: parsed_outcome,
                code,
                bytes: parsed_bytes,
                sha256,
                turn: parsed_turn,
                created_at,
            });
        }
        Ok(records)
    }

    /// Adds to the per-run tool counters (`runs.tool_turns`, `runs.tool_calls`).
    ///
    /// # Errors
    /// [`DbError::InvalidToolTurn`] when the run does not exist, plus the usual
    /// SQLite/lock errors. A missing run is never reported as `(0, 0)`.
    pub fn increment_run_tool_counters(
        &self,
        run_id: RunId,
        turns: u32,
        calls: u32,
    ) -> Result<(), DbError> {
        let connection = self.lock()?;
        let changed = connection.execute(
            "UPDATE runs
             SET tool_turns = tool_turns + ?2, tool_calls = tool_calls + ?3
             WHERE id = ?1",
            rusqlite::params![
                run_id.to_string(),
                i64::from(turns),
                i64::from(calls)
            ],
        )?;
        if changed != 1 {
            return Err(DbError::InvalidToolTurn(format!(
                "run `{run_id}` not found while incrementing tool counters"
            )));
        }
        Ok(())
    }

    /// Reads the per-run tool counters as `(tool_turns, tool_calls)`.
    ///
    /// # Errors
    /// [`DbError::InvalidToolTurn`] when the run does not exist or a stored
    /// counter is not a non-negative integer; never fabricates `(0, 0)`.
    pub fn run_tool_counters(&self, run_id: RunId) -> Result<(u32, u32), DbError> {
        let connection = self.lock()?;
        let row: Option<(i64, i64)> = connection
            .query_row(
                "SELECT tool_turns, tool_calls FROM runs WHERE id = ?1",
                rusqlite::params![run_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let (turns_raw, calls_raw) = row.ok_or_else(|| {
            DbError::InvalidToolTurn(format!("run `{run_id}` not found while reading tool counters"))
        })?;
        let turns = u32::try_from(turns_raw).map_err(|_| {
            DbError::InvalidToolTurn(format!("negative tool_turns {turns_raw} for run {run_id}"))
        })?;
        let calls = u32::try_from(calls_raw).map_err(|_| {
            DbError::InvalidToolTurn(format!("negative tool_calls {calls_raw} for run {run_id}"))
        })?;
        Ok((turns, calls))
    }
}

fn tool_turn_bytes_to_sql(bytes: Option<u64>) -> Result<Option<i64>, DbError> {
    match bytes {
        Some(value) => i64::try_from(value).map(Some).map_err(|_| {
            DbError::InvalidToolTurn(format!("bytes {value} exceeds the SQLite INTEGER range"))
        }),
        None => Ok(None),
    }
}

fn tool_turn_bytes_from_sql(bytes: Option<i64>) -> Result<Option<u64>, DbError> {
    match bytes {
        Some(value) => u64::try_from(value).map(Some).map_err(|_| {
            DbError::InvalidToolTurn(format!("negative byte count {value} in tool_turns"))
        }),
        None => Ok(None),
    }
}

/// One persisted tool call in a run (schema v4, 1.2 tool loop).
///
/// Mirrors the `tool_turns` row exactly: `args_sha256` is the only argument
/// representation stored (never the raw arguments), and nullable `code`/`bytes`/
/// `sha256` stay absent rather than being coerced to zero/empty.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolTurnRecord {
    /// Row id (uuid-v7 string minted by the caller).
    pub id: String,
    /// Run the tool call belongs to.
    pub run_id: RunId,
    /// Task that issued the tool call.
    pub task_id: TaskId,
    /// Provider tool-call id (unique per run).
    pub tool_call_id: String,
    /// Tool id from the frozen tool registry.
    pub tool: String,
    /// SHA-256 (lowercase hex) of the canonical JSON arguments.
    pub args_sha256: String,
    /// Tool-call outcome vocabulary (`ok|refused|denied|timeout|cancelled`).
    pub outcome: ToolOutcome,
    /// Named refusal code when the outcome carries one.
    pub code: Option<String>,
    /// Result size in bytes when known; absent stays absent.
    pub bytes: Option<u64>,
    /// SHA-256 of the result body when known.
    pub sha256: Option<String>,
    /// Loop turn (1-based) the call belongs to.
    pub turn: u32,
    /// Row creation timestamp (RFC3339 UTC).
    pub created_at: Timestamp,
}

#[cfg(test)]
mod tests {
    use super::*;
    use hiverouter_contract::SCHEMA_VERSION;

    #[test]
    fn fresh_database_is_version_zero() {
        let database = Database::open_in_memory().expect("in-memory db");
        assert_eq!(database.schema_version().expect("version"), 0);
        assert!(database.applied_migrations().expect("migrations").is_empty());
    }

    #[test]
    fn migrate_is_idempotent_and_records_bootstrap() {
        let database = Database::open_in_memory().expect("in-memory db");
        assert_eq!(database.migrate().expect("first migrate"), 4);
        assert_eq!(database.migrate().expect("second migrate"), 4);
        let applied = database.applied_migrations().expect("migrations");
        assert_eq!(applied.len(), 4);
        assert_eq!(applied[0].0, 1);
        assert_eq!(applied[0].1, "bootstrap");
        assert_eq!(applied[1].0, 2);
        assert_eq!(applied[1].1, "runs_tasks_chat");
        assert_eq!(applied[2].0, 3);
        assert_eq!(applied[2].1, "snapshot_index");
        assert_eq!(applied[3].0, 4);
        assert_eq!(applied[3].1, "tool_turns");
        assert_eq!(
            database.app_meta("schema_version").expect("meta").as_deref(),
            Some(SCHEMA_VERSION)
        );
        assert!(database.app_meta("bootstrapped_at").expect("meta").is_some());
    }

    #[test]
    fn down_then_up_restores_the_schema() {
        let database = Database::open_in_memory().expect("in-memory db");
        database.migrate().expect("migrate");
        assert_eq!(database.migrate_down_to(0).expect("down"), 0);
        assert_eq!(database.schema_version().expect("version"), 0);
        assert!(database.app_meta("schema_version").is_err());
        assert_eq!(database.migrate().expect("re-migrate"), 4);
        assert_eq!(
            database.app_meta("schema_version").expect("meta").as_deref(),
            Some(SCHEMA_VERSION)
        );
    }

    #[test]
    fn file_database_persists_across_handles() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("hiverouter.db");
        {
            let database = Database::open(&path).expect("open");
            database.migrate().expect("migrate");
        }
        let reopened = Database::open(&path).expect("reopen");
        assert_eq!(reopened.schema_version().expect("version"), 4);
        assert_eq!(reopened.path(), path);
    }

    #[test]
    fn unknown_down_target_is_refused() {
        let database = Database::open_in_memory().expect("in-memory db");
        database.migrate().expect("migrate");
        assert!(matches!(
            database.migrate_down_to(99),
            Err(DbError::UnknownMigration(99))
        ));
    }

    // --- schema v4: tool-turn persistence ---------------------------------

    use super::ToolTurnRecord;
    use hiverouter_contract::event::ToolOutcome;
    use hiverouter_contract::{RunId, TaskId, Timestamp};

    fn insert_run(database: &Database, run_id: RunId) {
        database
            .with_connection(|connection| {
                connection.execute(
                    "INSERT INTO runs(id, project_id, prompt, status, created_at, updated_at)
                     VALUES (?1, 'project-1', 'persist tool turns', 'queued', ?2, ?2)",
                    rusqlite::params![run_id.to_string(), Timestamp::now().as_str()],
                )?;
                Ok(())
            })
            .expect("insert run row");
    }

    fn sample_turn(run_id: RunId, task_id: TaskId) -> ToolTurnRecord {
        ToolTurnRecord {
            id: uuid::Uuid::now_v7().to_string(),
            run_id,
            task_id,
            tool_call_id: "call_01".to_owned(),
            tool: "file_read".to_owned(),
            args_sha256: "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
                .to_owned(),
            outcome: ToolOutcome::Refused,
            code: Some("E_TOOL_NOT_ALLOWED".to_owned()),
            bytes: Some(0),
            sha256: None,
            turn: 1,
            created_at: Timestamp::now(),
        }
    }

    #[test]
    fn tool_turn_roundtrip_and_run_counters() {
        let database = Database::open_in_memory().expect("in-memory db");
        database.migrate().expect("migrate");
        let run_id = RunId::new();
        let task_id = TaskId::new();
        insert_run(&database, run_id);

        assert_eq!(
            database.run_tool_counters(run_id).expect("fresh counters"),
            (0, 0)
        );

        let record = sample_turn(run_id, task_id);
        database.record_tool_turn(&record).expect("record turn");
        database
            .increment_run_tool_counters(run_id, 1, 1)
            .expect("increment");
        database
            .increment_run_tool_counters(run_id, 0, 1)
            .expect("increment");
        assert_eq!(
            database.run_tool_counters(run_id).expect("counters"),
            (1, 2)
        );

        let turns = database.tool_turns_for_run(run_id).expect("read turns");
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0], record);

        let empty_run = RunId::new();
        insert_run(&database, empty_run);
        assert!(database
            .tool_turns_for_run(empty_run)
            .expect("no turns")
            .is_empty());
        assert_eq!(
            database.run_tool_counters(empty_run).expect("counters"),
            (0, 0)
        );

        let unknown = RunId::new();
        assert!(database.run_tool_counters(unknown).is_err());
        assert!(database
            .increment_run_tool_counters(unknown, 1, 0)
            .is_err());
    }

    #[test]
    fn duplicate_tool_turn_is_refused_and_not_persisted() {
        let database = Database::open_in_memory().expect("in-memory db");
        database.migrate().expect("migrate");
        let run_id = RunId::new();
        let task_id = TaskId::new();
        insert_run(&database, run_id);

        database
            .record_tool_turn(&sample_turn(run_id, task_id))
            .expect("first record");
        let mut duplicate = sample_turn(run_id, task_id);
        duplicate.id = uuid::Uuid::now_v7().to_string();

        let error = database
            .record_tool_turn(&duplicate)
            .expect_err("same (run_id, tool_call_id) must be refused");
        assert!(
            matches!(error, DbError::Sqlite(_)),
            "duplicate is a SQLite constraint error, got {error:?}"
        );
        assert_eq!(
            database.tool_turns_for_run(run_id).expect("turns").len(),
            1,
            "the duplicate must not be persisted"
        );
    }

    #[test]
    fn tool_turn_migration_rolls_back_and_reapplies() {
        let database = Database::open_in_memory().expect("in-memory db");
        assert_eq!(database.migrate().expect("migrate"), 4);
        let run_id = RunId::new();
        let task_id = TaskId::new();
        insert_run(&database, run_id);
        database
            .record_tool_turn(&sample_turn(run_id, task_id))
            .expect("record turn");
        database
            .increment_run_tool_counters(run_id, 1, 1)
            .expect("increment");
        assert_eq!(
            database.run_tool_counters(run_id).expect("counters"),
            (1, 1)
        );

        assert_eq!(database.migrate_down_to(3).expect("down v4"), 3);
        assert!(
            database.tool_turns_for_run(run_id).is_err(),
            "v3 has no tool_turns table"
        );
        assert!(
            database.run_tool_counters(run_id).is_err(),
            "v3 runs has no tool counter columns"
        );
        let tool_tables: i64 = database
            .with_connection(|connection| {
                connection.query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'tool_turns'",
                    [],
                    |row| row.get(0),
                )
            })
            .expect("table probe");
        assert_eq!(tool_tables, 0, "down migration must drop tool_turns");

        assert_eq!(database.migrate().expect("re-migrate"), 4);
        assert_eq!(
            database.run_tool_counters(run_id).expect("recreated counters"),
            (0, 0),
            "re-added counter columns start at their default"
        );
        database
            .record_tool_turn(&sample_turn(run_id, task_id))
            .expect("record after re-up");
        assert_eq!(
            database.tool_turns_for_run(run_id).expect("turns").len(),
            1
        );
    }
}
