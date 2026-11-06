//! SQLite bootstrap and migration engine.
//!
//! Scope (blueprint Phase 9, WP 9.8): SQLite stores cache/metrics/usage/run rows;
//! the graph lives in memory plus versioned JSON snapshots. This module owns the
//! connection, pragmas and the ordered migration list; later phases add
//! migrations rather than editing earlier ones.

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use rusqlite::Connection;
use thiserror::Error;

use hiverouter_contract::{Refusal, Timestamp};

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
        assert_eq!(database.migrate().expect("first migrate"), 2);
        assert_eq!(database.migrate().expect("second migrate"), 2);
        let applied = database.applied_migrations().expect("migrations");
        assert_eq!(applied.len(), 2);
        assert_eq!(applied[0].0, 1);
        assert_eq!(applied[0].1, "bootstrap");
        assert_eq!(applied[1].0, 2);
        assert_eq!(applied[1].1, "runs_tasks_chat");
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
        assert_eq!(database.migrate().expect("re-migrate"), 2);
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
        assert_eq!(reopened.schema_version().expect("version"), 2);
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
}
