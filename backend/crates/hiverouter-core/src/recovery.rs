//! Boot recovery (blueprint Phase 9 exit gate: kill mid-run -> restart restores
//! the last committed snapshot and marks the run interrupted).

use std::sync::Arc;

use hiverouter_contract::{AgentState, Refusal, RunId};

use crate::db::Database;
use crate::graph::GraphAuthority;
use crate::snapshot::SnapshotStore;
use crate::store::RunStore;

/// What recovery did at boot.
#[derive(Debug, Default, PartialEq)]
pub struct RecoveryReport {
    /// Runs that were active when the process died and are now interrupted.
    pub interrupted_runs: Vec<RunId>,
    /// Runs whose latest snapshot was restored into the graph.
    pub restored_runs: Vec<RunId>,
    /// Snapshot files that could not be loaded (left in place, reported).
    pub corrupt_snapshots: Vec<RunId>,
}

impl RecoveryReport {
    /// Whether recovery changed anything.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.interrupted_runs.is_empty()
            && self.restored_runs.is_empty()
            && self.corrupt_snapshots.is_empty()
    }
}

/// Marks active runs interrupted and restores each run's latest snapshot into
/// the authoritative graph.
///
/// A corrupt snapshot is skipped with `E_SNAPSHOT_CORRUPT` recorded in the
/// report; the workspace is never silently reset.
///
/// # Errors
/// Database/graph refusals (a failed recovery must stop the boot edge).
pub fn recover_on_boot(
    db: Arc<Database>,
    graph: &GraphAuthority,
    snapshots: &SnapshotStore,
) -> Result<RecoveryReport, Refusal> {
    let store = RunStore::new(db);
    let mut report = RecoveryReport::default();

    let active: Vec<RunId> = store
        .list_runs(500)?
        .into_iter()
        .filter(|run| !run.status.is_terminal() && run.status != AgentState::Interrupted)
        .map(|run| run.id)
        .collect();
    for run_id in active {
        store.set_run_status(run_id, AgentState::Interrupted, None)?;
        report.interrupted_runs.push(run_id);
    }

    for run_id in snapshots.runs_on_disk()? {
        match snapshots.latest(run_id) {
            Ok(Some(envelope)) => {
                if graph.restore_from_snapshot(&envelope.graph).is_ok() {
                    report.restored_runs.push(run_id);
                } else {
                    report.corrupt_snapshots.push(run_id);
                }
            }
            Ok(None) => {}
            Err(refusal) if refusal == Refusal::SnapshotCorrupt => {
                tracing::error!(
                    run = %run_id,
                    code = refusal.code(),
                    "corrupt snapshot refused at recovery; workspace not reset"
                );
                report.corrupt_snapshots.push(run_id);
            }
            Err(refusal) => return Err(refusal),
        }
    }
    Ok(report)
}
