//! History / replay read API and the undo-time-travel read model
//! (blueprint Phase 9, WP 9.9).

use std::sync::Arc;

use hiverouter_contract::{AgentState, Refusal, RunId, Timestamp};

use crate::db::Database;
use crate::entities::Run;
use crate::snapshot::{SnapshotEnvelope, SnapshotStore};
use crate::store::RunStore;

/// Metadata of one readable revision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RevisionMeta {
    /// Revision number.
    pub revision: u64,
    /// When the revision was written.
    pub created_at: Timestamp,
    /// Run status captured at that revision, when known (v2 files).
    pub run_status: Option<AgentState>,
}

/// Time-travel read model for one run's scrubber.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeTravel {
    /// Run id.
    pub run_id: RunId,
    /// Revision metadata, ascending.
    pub revisions: Vec<RevisionMeta>,
    /// Latest revision (`None` when the run has no snapshots yet).
    pub latest_revision: Option<u64>,
}

impl TimeTravel {
    /// Whether a revision can be read.
    #[must_use]
    pub fn has_revision(&self, revision: u64) -> bool {
        self.revisions.iter().any(|meta| meta.revision == revision)
    }

    /// Revision count as a real number (scrubber "state 12 of 38").
    #[must_use]
    pub fn revision_count(&self) -> usize {
        self.revisions.len()
    }
}

/// Read-only history facade over the snapshot store + run rows.
#[derive(Debug, Clone)]
pub struct HistoryReadModel {
    db: Arc<Database>,
    snapshots: SnapshotStore,
}

impl HistoryReadModel {
    /// Builds the read model.
    #[must_use]
    pub fn new(db: Arc<Database>, snapshots: SnapshotStore) -> Self {
        Self { db, snapshots }
    }

    /// Runs known to the workspace, newest first.
    ///
    /// # Errors
    /// Database refusal.
    pub fn runs(&self, limit: u32) -> Result<Vec<Run>, Refusal> {
        RunStore::new(self.db.clone()).list_runs(limit)
    }

    /// Revision metadata for a run (from the SQLite index, sorted).
    ///
    /// # Errors
    /// Database refusal.
    pub fn revisions(&self, run_id: RunId) -> Result<Vec<RevisionMeta>, Refusal> {
        let indexed = self.snapshots.indexed_revisions(run_id)?;
        let mut revisions = Vec::with_capacity(indexed.len());
        for (revision, created_at) in indexed {
            let created = Timestamp::parse(&created_at).unwrap_or_else(|_| Timestamp::from_unix_secs(0));
            let run_status = self.snapshots.read(run_id, revision).ok().and_then(|envelope| envelope.run_status);
            revisions.push(RevisionMeta {
                revision,
                created_at: created,
                run_status,
            });
        }
        Ok(revisions)
    }

    /// Time-travel read model for the scrubber.
    ///
    /// # Errors
    /// Database refusal.
    pub fn time_travel(&self, run_id: RunId) -> Result<TimeTravel, Refusal> {
        let revisions = self.revisions(run_id)?;
        let latest_revision = revisions.last().map(|meta| meta.revision);
        Ok(TimeTravel {
            run_id,
            revisions,
            latest_revision,
        })
    }

    /// State at one revision.
    ///
    /// # Errors
    /// `E_HISTORY_GAP` when the revision is not readable, `E_SNAPSHOT_CORRUPT`
    /// when the file is corrupt.
    pub fn state_at(&self, run_id: RunId, revision: u64) -> Result<SnapshotEnvelope, Refusal> {
        let known = self.snapshots.list_revisions(run_id)?;
        if !known.contains(&revision) {
            return Err(Refusal::HistoryGap);
        }
        self.snapshots.read(run_id, revision)
    }
}
