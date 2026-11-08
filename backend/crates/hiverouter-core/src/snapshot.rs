//! Versioned JSON graph snapshots + snapshot-schema migration
//! (blueprint Phase 9, WP 9.2/9.3/9.6).
//!
//! SQLite stays the index/metadata store; the graph lives in memory and is
//! persisted as one versioned JSON file per revision under
//! `<data-dir>/snapshots/<run-id>/<revision>.json`. Writes are atomic
//! (temp file + rename). Loading validates the schema version and refuses a
//! corrupt snapshot with `E_SNAPSHOT_CORRUPT` instead of resetting the workspace.

use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use hiverouter_contract::{AgentState, Refusal, RunId, TaskId, Timestamp};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::db::{Database, DbError};
use crate::graph::GraphSnapshot;

/// Original snapshot schema version.
pub const SNAPSHOT_SCHEMA_V1: &str = "hivemind.snapshot.v1";
/// Current snapshot schema version (adds `run_status`).
pub const SNAPSHOT_SCHEMA_V2: &str = "hivemind.snapshot.v2";

/// One versioned graph snapshot.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SnapshotEnvelope {
    /// Snapshot schema version (see [`SNAPSHOT_SCHEMA_V2`]).
    pub schema_version: String,
    /// Run this snapshot belongs to.
    pub run_id: RunId,
    /// Monotonic revision number (0-based).
    pub revision: u64,
    /// Creation timestamp.
    pub created_at: Timestamp,
    /// Graph state at this revision.
    pub graph: GraphSnapshot,
    /// Run status at this revision (v2+; absent in v1 files).
    #[serde(default)]
    pub run_status: Option<AgentState>,
}

/// Durable snapshot store rooted at `<data-dir>/snapshots`.
#[derive(Debug, Clone)]
pub struct SnapshotStore {
    root: PathBuf,
    db: Arc<Database>,
}

impl SnapshotStore {
    /// Creates the store (directory created on first write).
    #[must_use]
    pub fn new(data_dir: &Path, db: Arc<Database>) -> Self {
        Self {
            root: data_dir.join("snapshots"),
            db,
        }
    }

    /// Snapshot root directory.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn run_dir(&self, run_id: RunId) -> PathBuf {
        self.root.join(run_id.to_string())
    }

    /// Writes one snapshot atomically and indexes it in SQLite.
    ///
    /// # Errors
    /// `E_SNAPSHOT_CORRUPT` when the file cannot be written or the index row
    /// cannot be recorded.
    pub fn write(&self, envelope: &SnapshotEnvelope) -> Result<PathBuf, Refusal> {
        if envelope.schema_version != SNAPSHOT_SCHEMA_V2 {
            return Err(Refusal::SnapshotCorrupt);
        }
        let directory = self.run_dir(envelope.run_id);
        std::fs::create_dir_all(&directory).map_err(|_| Refusal::SnapshotCorrupt)?;
        let path = directory.join(format!("{:010}.json", envelope.revision));
        let temporary = path.with_extension("json.tmp");
        let json = serde_json::to_string_pretty(envelope).map_err(|_| Refusal::SnapshotCorrupt)?;
        // Durable atomic replace: write, fsync the file, rename, then fsync the
        // directory where the platform exposes it. A power loss must not leave a
        // boot-failing truncated newest revision.
        {
            let mut file =
                std::fs::File::create(&temporary).map_err(|_| Refusal::SnapshotCorrupt)?;
            file.write_all(json.as_bytes())
                .map_err(|_| Refusal::SnapshotCorrupt)?;
            file.sync_all().map_err(|_| Refusal::SnapshotCorrupt)?;
        }
        std::fs::rename(&temporary, &path).map_err(|_| Refusal::SnapshotCorrupt)?;
        #[cfg(unix)]
        if let Ok(directory) = std::fs::File::open(&directory) {
            let _ = directory.sync_all();
        }
        self.db
            .with_connection(|connection| {
                connection.execute(
                    "INSERT OR REPLACE INTO snapshots(run_id, revision, path, created_at, schema_version)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![
                        envelope.run_id.to_string(),
                        i64::try_from(envelope.revision).unwrap_or(i64::MAX),
                        path.display().to_string(),
                        envelope.created_at.as_str(),
                        envelope.schema_version,
                    ],
                )?;
                Ok(())
            })
            .map_err(|error: DbError| error.refusal())?;
        Ok(path)
    }

    /// Reads one revision, migrating older schema versions on load.
    ///
    /// # Errors
    /// `E_SNAPSHOT_CORRUPT` when the file is missing/corrupt or the schema
    /// version cannot be migrated.
    pub fn read(&self, run_id: RunId, revision: u64) -> Result<SnapshotEnvelope, Refusal> {
        let path = self.run_dir(run_id).join(format!("{revision:010}.json"));
        let raw = std::fs::read_to_string(&path).map_err(|_| Refusal::SnapshotCorrupt)?;
        let value: Value = serde_json::from_str(&raw).map_err(|_| Refusal::SnapshotCorrupt)?;
        let envelope = migrate_to_current(&value)?;
        if envelope.run_id != run_id || envelope.revision != revision {
            return Err(Refusal::SnapshotCorrupt);
        }
        Ok(envelope)
    }

    /// Lists stored revision numbers for a run (ascending).
    ///
    /// # Errors
    /// `E_SNAPSHOT_CORRUPT` if the directory cannot be read.
    pub fn list_revisions(&self, run_id: RunId) -> Result<Vec<u64>, Refusal> {
        let directory = self.run_dir(run_id);
        let mut revisions = Vec::new();
        let entries = match std::fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(revisions),
            Err(_) => return Err(Refusal::SnapshotCorrupt),
        };
        for entry in entries {
            let entry = entry.map_err(|_| Refusal::SnapshotCorrupt)?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(stem) = name.strip_suffix(".json") {
                if let Ok(revision) = stem.parse::<u64>() {
                    revisions.push(revision);
                }
            }
        }
        revisions.sort_unstable();
        Ok(revisions)
    }

    /// Latest snapshot for a run, if any.
    ///
    /// # Errors
    /// Propagates the read refusal (never silently resets).
    pub fn latest(&self, run_id: RunId) -> Result<Option<SnapshotEnvelope>, Refusal> {
        let revisions = self.list_revisions(run_id)?;
        match revisions.last() {
            Some(revision) => self.read(run_id, *revision).map(Some),
            None => Ok(None),
        }
    }

    /// Run ids that have at least one snapshot on disk.
    ///
    /// # Errors
    /// `E_SNAPSHOT_CORRUPT` if the root cannot be read.
    pub fn runs_on_disk(&self) -> Result<Vec<RunId>, Refusal> {
        let entries = match std::fs::read_dir(&self.root) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(_) => return Err(Refusal::SnapshotCorrupt),
        };
        let mut runs = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|_| Refusal::SnapshotCorrupt)?;
            if !entry.path().is_dir() {
                continue;
            }
            if let Ok(run_id) = entry.file_name().to_string_lossy().parse::<RunId>() {
                runs.push(run_id);
            }
        }
        Ok(runs)
    }

    /// Indexed revision metadata from SQLite for a run.
    ///
    /// # Errors
    /// Database refusal.
    pub fn indexed_revisions(&self, run_id: RunId) -> Result<Vec<(u64, String)>, Refusal> {
        self.db
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    "SELECT revision, created_at FROM snapshots WHERE run_id = ?1 ORDER BY revision ASC",
                )?;
                let rows = statement.query_map(rusqlite::params![run_id.to_string()], |row| {
                    let revision: i64 = row.get(0)?;
                    let created_at: String = row.get(1)?;
                    Ok((u64::try_from(revision).unwrap_or(0), created_at))
                })?;
                let mut revisions = Vec::new();
                for row in rows {
                    revisions.push(row?);
                }
                Ok(revisions)
            })
            .map_err(|error: DbError| error.refusal())
    }
}

/// Migrates a stored snapshot JSON value to the current schema version.
///
/// # Errors
/// `E_SNAPSHOT_CORRUPT` for unknown versions or malformed envelopes.
pub fn migrate_to_current(value: &Value) -> Result<SnapshotEnvelope, Refusal> {
    let version = value
        .get("schema_version")
        .and_then(Value::as_str)
        .ok_or(Refusal::SnapshotCorrupt)?;
    let upgraded = match version {
        SNAPSHOT_SCHEMA_V1 => migrate_v1_to_v2(value.clone())?,
        SNAPSHOT_SCHEMA_V2 => value.clone(),
        _ => return Err(Refusal::SnapshotCorrupt),
    };
    serde_json::from_value(upgraded).map_err(|_| Refusal::SnapshotCorrupt)
}

/// v1 -> v2: add the `run_status` field (unknown for old files -> `None`).
///
/// # Errors
/// `E_SNAPSHOT_CORRUPT` when the value is not an object.
pub fn migrate_v1_to_v2(mut value: Value) -> Result<Value, Refusal> {
    let object = value.as_object_mut().ok_or(Refusal::SnapshotCorrupt)?;
    object.insert(
        "schema_version".to_owned(),
        Value::String(SNAPSHOT_SCHEMA_V2.to_owned()),
    );
    object
        .entry("run_status".to_owned())
        .or_insert(Value::Null);
    Ok(value)
}

/// v2 -> v1: drop the `run_status` field (round-trip inverse).
///
/// # Errors
/// `E_SNAPSHOT_CORRUPT` when the value is not an object.
pub fn migrate_v2_to_v1(mut value: Value) -> Result<Value, Refusal> {
    let object = value.as_object_mut().ok_or(Refusal::SnapshotCorrupt)?;
    object.insert(
        "schema_version".to_owned(),
        Value::String(SNAPSHOT_SCHEMA_V1.to_owned()),
    );
    object.remove("run_status");
    Ok(value)
}

/// Captures one run's committed graph, not the process-wide graph.
///
/// The graph authority is process-wide and deliberately retains completed
/// runs for rewind; a run snapshot must retain only the tasks owned by that
/// run. Edges and the topological order are filtered by the same allow-set so
/// the persisted revision can never leak another run's nodes into replay.
///
/// # Errors
/// Graph lock or task-index refusal.
pub fn capture_scoped(
    graph: &crate::graph::GraphAuthority,
    run_id: RunId,
    revision: u64,
    run_status: AgentState,
    allowed: &HashSet<TaskId>,
) -> Result<SnapshotEnvelope, Refusal> {
    let mut envelope = capture(graph, run_id, revision, run_status)?;
    envelope
        .graph
        .nodes
        .retain(|node| allowed.contains(&node.task_id));
    envelope
        .graph
        .edges
        .retain(|(parent, child)| allowed.contains(parent) && allowed.contains(child));
    envelope
        .graph
        .toposort
        .retain(|task_id| allowed.contains(task_id));
    Ok(envelope)
}

/// Builds a snapshot envelope for a run from the current graph.
///
/// # Errors
/// Graph lock refusal.
pub fn capture(
    graph: &crate::graph::GraphAuthority,
    run_id: RunId,
    revision: u64,
    run_status: AgentState,
) -> Result<SnapshotEnvelope, Refusal> {
    Ok(SnapshotEnvelope {
        schema_version: SNAPSHOT_SCHEMA_V2.to_owned(),
        run_id,
        revision,
        created_at: Timestamp::now(),
        graph: graph.snapshot()?,
        run_status: Some(run_status),
    })
}
