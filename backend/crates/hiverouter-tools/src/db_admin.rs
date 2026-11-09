//! Local SQLite backup/restore helpers used by the operator CLI.
//!
//! Backups use SQLite `VACUUM INTO` so a live WAL database is captured
//! consistently. Restore verifies the backup first, refuses a schema newer than
//! this binary, copies the current database aside, and removes stale WAL/SHM
//! sidecars. Nothing here starts a server or invents a successful restore.

use std::path::{Path, PathBuf};

use hiverouter_contract::{Refusal, Timestamp};
use rusqlite::Connection;

use crate::sha256::sha256_hex;

/// Newest schema this binary understands. Keep in step with core migrations.
pub const SUPPORTED_SCHEMA: i64 = 17;

/// One backup/restore receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbReport {
    /// Absolute path written/read.
    pub path: PathBuf,
    /// File size in bytes.
    pub bytes: u64,
    /// SHA-256 of the file.
    pub sha256: String,
    /// Schema version found in the file.
    pub schema_version: i64,
}

fn integrity_ok(path: &Path) -> Result<bool, Refusal> {
    let connection = Connection::open(path).map_err(|_| Refusal::PathNotFound)?;
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|_| Refusal::SnapshotCorrupt)?;
    Ok(integrity == "ok")
}

fn schema_version(path: &Path) -> Result<i64, Refusal> {
    let connection = Connection::open(path).map_err(|_| Refusal::PathNotFound)?;
    let exists: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_migrations'",
            [],
            |row| row.get(0),
        )
        .map_err(|_| Refusal::SnapshotCorrupt)?;
    if exists == 0 {
        return Ok(0);
    }
    connection
        .query_row("SELECT COALESCE(MAX(version), 0) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .map_err(|_| Refusal::SnapshotCorrupt)
}

fn digest(path: &Path) -> Result<(u64, String), Refusal> {
    let bytes = std::fs::read(path).map_err(|_| Refusal::PathNotFound)?;
    Ok((u64::try_from(bytes.len()).unwrap_or(u64::MAX), sha256_hex(&bytes)))
}

/// Captures a consistent backup of `db_path` into `out_path`.
///
/// # Errors
/// `E_PATH_NOT_FOUND` when the source is absent, `E_SNAPSHOT_CORRUPT` when the
/// backup fails integrity, or a path/write refusal.
pub fn backup(db_path: &Path, out_path: &Path) -> Result<DbReport, Refusal> {
    if !db_path.is_file() {
        return Err(Refusal::PathNotFound);
    }
    if out_path.exists() {
        return Err(Refusal::DecisionRequired);
    }
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| Refusal::PathOutsideJail)?;
    }
    let connection = Connection::open(db_path).map_err(|_| Refusal::SnapshotCorrupt)?;
    connection
        .execute(
            "VACUUM INTO ?1",
            rusqlite::params![out_path.to_string_lossy().to_string()],
        )
        .map_err(|_| Refusal::SnapshotCorrupt)?;
    if !integrity_ok(out_path)? {
        return Err(Refusal::SnapshotCorrupt);
    }
    let schema = schema_version(out_path)?;
    let (bytes, sha256) = digest(out_path)?;
    Ok(DbReport {
        path: out_path.to_path_buf(),
        bytes,
        sha256,
        schema_version: schema,
    })
}

/// Restores `backup_path` over `db_path` after verifying it.
///
/// A newer backup schema is refused before any write. Existing data is copied
/// to `<db>.pre-restore-<timestamp>.bak`; stale `-wal`/`-shm` files are removed.
///
/// # Errors
/// `E_DB_SCHEMA_NEWER`, `E_SNAPSHOT_CORRUPT`, `E_DECISION_REQUIRED` when the
/// target exists without `force`, or a path/write refusal.
pub fn restore(backup_path: &Path, db_path: &Path, force: bool) -> Result<DbReport, Refusal> {
    if !backup_path.is_file() {
        return Err(Refusal::PathNotFound);
    }
    if !integrity_ok(backup_path)? {
        return Err(Refusal::SnapshotCorrupt);
    }
    let schema = schema_version(backup_path)?;
    if schema > SUPPORTED_SCHEMA {
        return Err(Refusal::SchemaNewer);
    }
    if db_path.exists() && !force {
        return Err(Refusal::DecisionRequired);
    }
    if db_path.exists() {
        let stamp: String = Timestamp::now()
            .as_str()
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() {
                    character
                } else {
                    '-'
                }
            })
            .collect();
        let mut previous = db_path.with_file_name(format!(
            "{}.pre-restore-{stamp}.bak",
            db_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("hiverouter.db")
        ));
        let mut suffix = 1_u32;
        while previous.exists() {
            previous = db_path.with_file_name(format!(
                "{}.pre-restore-{stamp}-{suffix}.bak",
                db_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("hiverouter.db")
            ));
            suffix = suffix.saturating_add(1);
        }
        std::fs::copy(db_path, &previous).map_err(|_| Refusal::PathNotFound)?;
    }
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| Refusal::PathOutsideJail)?;
    }
    std::fs::copy(backup_path, db_path).map_err(|_| Refusal::PathNotFound)?;
    for suffix in ["-wal", "-shm"] {
        let sidecar = PathBuf::from(format!("{}{suffix}", db_path.display()));
        if sidecar.exists() {
            std::fs::remove_file(sidecar).map_err(|_| Refusal::PathNotFound)?;
        }
    }
    if !integrity_ok(db_path)? {
        return Err(Refusal::SnapshotCorrupt);
    }
    let (bytes, sha256) = digest(db_path)?;
    Ok(DbReport {
        path: db_path.to_path_buf(),
        bytes,
        sha256,
        schema_version: schema_version(db_path)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seeded(path: &Path) {
        let connection = Connection::open(path).expect("open");
        connection
            .execute_batch(
                "CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY, name TEXT NOT NULL, applied_at TEXT NOT NULL);
                 CREATE TABLE payload(value TEXT NOT NULL);
                 INSERT INTO schema_migrations VALUES (11, 'test', 'now');
                 INSERT INTO payload VALUES ('kept');",
            )
            .expect("seed");
    }

    #[test]
    fn backup_and_restore_roundtrip_preserve_rows() {
        let temp = tempfile::tempdir().expect("temp");
        let db = temp.path().join("hiverouter.db");
        let saved = temp.path().join("saved.db");
        seeded(&db);
        let receipt = backup(&db, &saved).expect("backup");
        assert_eq!(receipt.schema_version, 11);
        assert!(receipt.bytes > 0);
        assert_eq!(receipt.sha256.len(), 64);

        std::fs::remove_file(&db).expect("remove target");
        let restored = restore(&saved, &db, true).expect("restore");
        assert_eq!(restored.schema_version, 11);
        let connection = Connection::open(&db).expect("open restored");
        let value: String = connection
            .query_row("SELECT value FROM payload", [], |row| row.get(0))
            .expect("row");
        assert_eq!(value, "kept");
    }

    #[test]
    fn newer_backup_schema_is_refused_before_copy() {
        let temp = tempfile::tempdir().expect("temp");
        let db = temp.path().join("hiverouter.db");
        let future = temp.path().join("future.bak");
        seeded(&db);
        let connection = Connection::open(&future).expect("future");
        connection
            .execute_batch(
                "CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY, name TEXT NOT NULL, applied_at TEXT NOT NULL);
                 INSERT INTO schema_migrations VALUES (18, 'future', 'now');",
            )
            .expect("future schema");
        assert_eq!(
            restore(&future, &db, true),
            Err(Refusal::SchemaNewer)
        );
    }

    #[test]
    fn existing_target_requires_force() {
        let temp = tempfile::tempdir().expect("temp");
        let db = temp.path().join("hiverouter.db");
        let saved = temp.path().join("saved.db");
        seeded(&db);
        backup(&db, &saved).expect("backup");
        assert_eq!(
            restore(&saved, &db, false),
            Err(Refusal::DecisionRequired)
        );
    }
}
