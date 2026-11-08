//! Boot recovery (blueprint Phase 9 exit gate: kill mid-run -> restart restores
//! the last committed snapshot and marks the run interrupted).
//!
//! Recovery is the boot-time owner of a crashed run: it marks every active run
//! interrupted **and settles its tasks**, so a terminal run never keeps a
//! `running` or `awaiting_approval` task row behind (the S03/S04 live defect: a
//! later `KILL_WORKER` no-ops on such a task because no waiter survived the
//! restart). The settled state is the one the task state machine already names
//! for a restart: `apply_task(state, Restart) -> interrupted`
//! (`crate::run_state`, W11 §4). Database rows, the authoritative graph and one
//! new snapshot revision are kept equal; settlement never resurrects a task that
//! was already terminal.

use std::collections::{BTreeSet, HashSet};
use std::sync::Arc;

use hiverouter_contract::event::NodeEvent;
use hiverouter_contract::{AgentState, Event, Refusal, RunId, TaskId};

use crate::bus::EventBus;
use crate::db::Database;
use crate::graph::GraphAuthority;
use crate::run_state::{self, is_terminal_task_state, StateEvent};
use crate::snapshot::{self, SnapshotStore};
use crate::store::RunStore;

/// One task that boot recovery moved to a terminal state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettledTask {
    /// Run the task belongs to.
    pub run_id: RunId,
    /// Settled task.
    pub task_id: TaskId,
    /// State the row held when recovery started.
    pub from: AgentState,
    /// Terminal state recovery persisted.
    pub to: AgentState,
}

/// What recovery did at boot.
#[derive(Debug, Default, PartialEq)]
pub struct RecoveryReport {
    /// Runs that were active when the process died and are now interrupted.
    pub interrupted_runs: Vec<RunId>,
    /// The run whose latest snapshot is the image currently in the graph.
    ///
    /// The graph holds one image at a time, so this names at most one run: the
    /// newest loadable snapshot wins. Every other run remains readable through
    /// the history API.
    pub restored_runs: Vec<RunId>,
    /// Snapshot files that could not be loaded (left in place, reported).
    pub corrupt_snapshots: Vec<RunId>,
    /// Non-terminal tasks recovery settled (database + graph).
    pub settled_tasks: Vec<SettledTask>,
    /// Graph nodes that could not be updated in memory (database is still settled).
    pub graph_settle_failures: Vec<TaskId>,
    /// Runs whose settled state could not be committed as a new snapshot revision.
    pub snapshot_write_failures: Vec<RunId>,
}

impl RecoveryReport {
    /// Whether recovery changed anything.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.interrupted_runs.is_empty()
            && self.restored_runs.is_empty()
            && self.corrupt_snapshots.is_empty()
            && self.settled_tasks.is_empty()
            && self.graph_settle_failures.is_empty()
            && self.snapshot_write_failures.is_empty()
    }
}

/// Marks active runs interrupted, settles their tasks, and restores the newest
/// loadable snapshot into the authoritative graph.
///
/// A corrupt snapshot is skipped with `E_SNAPSHOT_CORRUPT` recorded in the
/// report; the workspace is never silently reset. The production boot path
/// (`main.rs`) runs before the event bus exists, so this constructor publishes
/// nothing; [`publish_recovered_node_states`] is the catch-up publisher used
/// once the bus is constructed.
///
/// # Errors
/// Database/graph refusals (a failed recovery must stop the boot edge).
pub fn recover_on_boot(
    db: Arc<Database>,
    graph: &GraphAuthority,
    snapshots: &SnapshotStore,
) -> Result<RecoveryReport, Refusal> {
    recover_on_boot_with_events(db, graph, snapshots, None)
}

/// [`recover_on_boot`] with an event bus for the settlement frames.
///
/// Every settled task publishes exactly one `NODE_STATE` naming the terminal
/// state that was persisted, so a subscriber observes the same outcome the
/// database holds. Pass `None` on a boot path that has no bus yet.
///
/// # Errors
/// Database/graph refusals (a failed recovery must stop the boot edge).
pub fn recover_on_boot_with_events(
    db: Arc<Database>,
    graph: &GraphAuthority,
    snapshots: &SnapshotStore,
    bus: Option<&EventBus>,
) -> Result<RecoveryReport, Refusal> {
    let store = RunStore::new(db);
    let mut report = RecoveryReport::default();

    // 1. Active runs become interrupted; a run already interrupted is still
    //    recovered (a previous crash may have died mid-recovery, before its
    //    tasks were settled).
    let runs = store.list_runs(500)?;
    let mut interrupted: BTreeSet<RunId> = runs
        .iter()
        .filter(|run| run.status == AgentState::Interrupted)
        .map(|run| run.id)
        .collect();
    for run in &runs {
        if run.status.is_terminal() {
            continue;
        }
        let error_code = if run.status == AgentState::AwaitingApproval {
            Some(Refusal::ApprovalTimeout.code())
        } else {
            None
        };
        store.set_run_status(run.id, AgentState::Interrupted, error_code)?;
        interrupted.insert(run.id);
        report.interrupted_runs.push(run.id);
    }

    // 2. Restore snapshots deterministically (oldest first, so the newest
    //    committed image is the one in memory). Loading before settlement lets
    //    the graph nodes be updated by id instead of rebuilt. Every snapshot is
    //    validated and a corrupt one is reported; the in-memory graph holds one
    //    image at a time, so `restored_runs` names only the run whose image is
    //    actually loaded.
    let mut restore_order: Vec<(String, RunId)> = Vec::new();
    for run_id in snapshots.runs_on_disk()? {
        let created = store
            .get_run(run_id)
            .map(|run| run.created_at.as_str().to_owned())
            .unwrap_or_default();
        restore_order.push((created, run_id));
    }
    restore_order.sort();
    let mut restored_run = None;
    for (_, run_id) in restore_order {
        match snapshots.latest(run_id) {
            Ok(Some(envelope)) => match graph.restore_from_snapshot(&envelope.graph) {
                Ok(()) => restored_run = Some(run_id),
                Err(refusal) if refusal == Refusal::SnapshotCorrupt => {
                    tracing::error!(
                        run = %run_id,
                        code = refusal.code(),
                        "snapshot image refused at recovery; workspace not reset"
                    );
                    report.corrupt_snapshots.push(run_id);
                }
                Err(refusal) => return Err(refusal),
            },
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
    if let Some(run_id) = restored_run {
        report.restored_runs.push(run_id);
    }

    // 3. Settle every non-terminal task of every interrupted run; repair the
    //    graph node of an already-terminal task (a crash between the row write
    //    and the graph update, or an earlier recovery that died mid-settlement,
    //    can leave the two disagreeing).
    for run_id in &interrupted {
        for task in store.tasks_for_run(*run_id)? {
            if is_terminal_task_state(task.state) {
                match repair_graph_state(graph, &task) {
                    Ok(true) => {
                        tracing::warn!(
                            task = %task.id,
                            state = task.state.as_str(),
                            "repaired a graph node left non-terminal by an earlier interrupted settlement"
                        );
                        if let Some(bus) = bus {
                            publish_node_state(bus, graph, *run_id, task.id, task.state);
                        }
                    }
                    Ok(false) => {}
                    Err(refusal) => {
                        tracing::error!(
                            task = %task.id,
                            code = refusal.code(),
                            "could not repair a graph node from the persisted task state"
                        );
                        report.graph_settle_failures.push(task.id);
                    }
                }
                continue;
            }
            let to = settlement_target(task.state);
            store.update_task_state(task.id, to)?;
            if let Err(refusal) = settle_graph_node(graph, &task, to) {
                tracing::error!(
                    task = %task.id,
                    code = refusal.code(),
                    "recovered task row settled but its graph node could not be updated"
                );
                report.graph_settle_failures.push(task.id);
            }
            if let Some(bus) = bus {
                publish_node_state(bus, graph, *run_id, task.id, to);
            }
            report.settled_tasks.push(SettledTask {
                run_id: *run_id,
                task_id: task.id,
                from: task.state,
                to,
            });
        }
    }

    // 4. The restored graph is the crash image of the last committed run. At
    //    this point no run is active, so every non-terminal node without a task
    //    row is a leftover of the interrupted snapshot image: settle it in the
    //    graph too (nothing to persist, no task row exists for it).
    let mut graph_only_settlements = 0_usize;
    if let Some(run_id) = restored_run.filter(|run| interrupted.contains(run)) {
        for node in graph.snapshot()?.nodes {
            if node.state.is_terminal() {
                continue;
            }
            match store.get_task(node.task_id) {
                Ok(_) => continue,
                Err(Refusal::TaskNotFound) => {}
                Err(refusal) => return Err(refusal),
            }
            let to = settlement_target(node.state);
            if let Err(refusal) = graph.set_state(node.task_id, to) {
                tracing::error!(
                    task = %node.task_id,
                    code = refusal.code(),
                    "restored graph node could not be settled"
                );
                report.graph_settle_failures.push(node.task_id);
                continue;
            }
            if let Some(bus) = bus {
                publish_node_state(bus, graph, run_id, node.task_id, to);
            }
            graph_only_settlements += 1;
        }
    }

    // 5. Persist the settled graph as one new revision for every run whose
    //    state recovery changed, so a later restore (and the scrubber) shows the
    //    terminal recovery outcome instead of the pre-crash image.
    let mut changed_runs: BTreeSet<RunId> =
        report.settled_tasks.iter().map(|task| task.run_id).collect();
    if graph_only_settlements > 0 {
        if let Some(run_id) = restored_run {
            changed_runs.insert(run_id);
        }
    }
    for run_id in changed_runs {
        let status = store
            .get_run(run_id)
            .map(|run| run.status)
            .unwrap_or(AgentState::Interrupted);
        let revision = next_revision(snapshots, run_id);
        let allowed: HashSet<TaskId> = match store.tasks_for_run(run_id) {
            Ok(tasks) => tasks.into_iter().map(|task| task.id).collect(),
            Err(refusal) => {
                tracing::error!(run = %run_id, code = refusal.code(), "recovery snapshot scope unavailable");
                report.snapshot_write_failures.push(run_id);
                continue;
            }
        };
        let envelope = match snapshot::capture_scoped(graph, run_id, revision, status, &allowed) {
            Ok(envelope) => envelope,
            Err(refusal) => {
                tracing::error!(run = %run_id, code = refusal.code(), "recovery snapshot capture refused");
                report.snapshot_write_failures.push(run_id);
                continue;
            }
        };
        match snapshots.write(&envelope) {
            Ok(path) => tracing::info!(
                run = %run_id,
                revision,
                path = %path.display(),
                "committed recovery snapshot revision"
            ),
            Err(refusal) => {
                tracing::error!(
                    run = %run_id,
                    code = refusal.code(),
                    "recovery snapshot write failed; database and graph are still settled"
                );
                report.snapshot_write_failures.push(run_id);
            }
        }
    }

    Ok(report)
}

/// Publishes the persisted recovery outcome once a bus exists.
///
/// The production boot order is `recover_on_boot` (no bus yet) followed by
/// `AppState::with_graph` (which builds the bus), so the settlement frames are
/// published here instead: one `NODE_STATE` per task that recovery settled to
/// `interrupted`. The state is read from the database and the revision/parent
/// from the authoritative graph, never invented; the call mutates nothing. It
/// exists so the recovered node states are observable in the bus/replay window of
/// the booted process.
///
/// # Errors
/// Database refusal (the bus publishes are logged, never fatal).
pub fn publish_recovered_node_states(
    db: Arc<Database>,
    graph: &GraphAuthority,
    bus: &EventBus,
) -> Result<usize, Refusal> {
    let store = RunStore::new(db);
    let mut published = 0_usize;
    for run in store.list_runs(500)? {
        if run.status != AgentState::Interrupted {
            continue;
        }
        for task in store.tasks_for_run(run.id)? {
            if task.state != AgentState::Interrupted {
                continue;
            }
            publish_node_state(bus, graph, run.id, task.id, task.state);
            published += 1;
        }
    }
    if published > 0 {
        tracing::info!(
            published,
            "published recovered node states after boot recovery"
        );
    }
    Ok(published)
}

/// Terminal state recovery persists for a non-terminal task.
///
/// The task state machine already owns the restart edge for every non-terminal
/// lifecycle state (`queued|running|awaiting_approval + restart -> interrupted`).
/// A row holding a non-lifecycle (UI-only) state cannot be validated, so the
/// recovery owner settles it to `interrupted` and records the refusal in the log
/// rather than leaving a non-terminal task behind.
fn settlement_target(current: AgentState) -> AgentState {
    match run_state::apply_task(current, StateEvent::Restart) {
        Ok(transition) => transition.to,
        Err(error) => {
            tracing::warn!(
                state = current.as_str(),
                code = error.code(),
                "task row held a non-lifecycle state; settling it interrupted"
            );
            AgentState::Interrupted
        }
    }
}

/// Makes the graph node state equal the persisted task state (repair path).
///
/// Returns `true` when the graph had to be changed. A task without a graph node
/// is left alone: the node was never restored, so there is no contradictory
/// state to repair.
fn repair_graph_state(
    graph: &GraphAuthority,
    task: &crate::entities::TaskRecord,
) -> Result<bool, Refusal> {
    match graph.node(task.id)? {
        Some(node) if node.state == task.state => Ok(false),
        Some(_) => {
            graph.set_state(task.id, task.state)?;
            Ok(true)
        }
        None => Ok(false),
    }
}

/// Ensures the task has a graph node, then sets the settled state on it.
fn settle_graph_node(
    graph: &GraphAuthority,
    task: &crate::entities::TaskRecord,
    to: AgentState,
) -> Result<(), Refusal> {
    if !graph.contains(task.id)? {
        match graph.add_task(task.id, task.parent_task_id) {
            Ok(_) => {}
            // Already present: another recovery writer won the insert.
            Err(Refusal::SchemaInvalid) => {}
            Err(Refusal::TaskNotFound) => {
                // The parent node is missing (its row belongs to another run or
                // was never restored); the child state is still authoritative, so
                // add it at the top level and say so instead of dropping it.
                tracing::warn!(
                    task = %task.id,
                    parent = ?task.parent_task_id,
                    "parent node missing at recovery; adding the settled task without a parent edge"
                );
                graph.add_task(task.id, None)?;
            }
            Err(refusal) => return Err(refusal),
        }
    }
    graph.set_metadata(
        task.id,
        crate::execution::task_node_kind(task),
        &task.role,
        None,
    )?;
    graph.set_state(task.id, to)
}

/// Next snapshot revision for a run, monotonic across the index and the disk.
fn next_revision(snapshots: &SnapshotStore, run_id: RunId) -> u64 {
    let indexed = snapshots
        .indexed_revisions(run_id)
        .map(|rows| rows.last().map(|(revision, _)| *revision))
        .unwrap_or(None);
    let on_disk = snapshots
        .list_revisions(run_id)
        .map(|revisions| revisions.last().copied())
        .unwrap_or(None);
    let latest = match (indexed, on_disk) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (left, right) => left.or(right),
    };
    latest.map_or(0, |revision| revision.saturating_add(1))
}

/// Publishes one `NODE_STATE`; a bus failure is logged and never fails recovery.
///
/// `rev` and `parent_task_id` come from the authoritative graph node when it is
/// present, so the frame carries the same revision `REWIRE_TASK.expected_rev`
/// compares against; `kind` stays absent because a task's tier is not threaded
/// through the task record yet (the free-form role is never promoted to a kind).
fn publish_node_state(
    bus: &EventBus,
    graph: &GraphAuthority,
    run_id: RunId,
    task_id: TaskId,
    state: AgentState,
) {
    let (rev, parent_task_id) = match graph.node(task_id) {
        Ok(Some(node)) => (Some(node.rev), node.parent_task_id),
        Ok(None) => (None, None),
        Err(refusal) => {
            tracing::warn!(
                code = refusal.code(),
                task = %task_id,
                "graph node unreadable while publishing a recovered NODE_STATE; revision omitted"
            );
            (None, None)
        }
    };
    if let Err(refusal) = bus.publish_for_run(
        Event::NodeState(NodeEvent {
            task_id,
            state,
            rev,
            kind: None,
            parent_task_id,
        }),
        Some(run_id),
        None,
    ) {
        tracing::error!(
            code = refusal.code(),
            task = %task_id,
            "failed to publish a recovered NODE_STATE"
        );
    }
}

#[cfg(test)]
mod tests {
    use hiverouter_contract::TaskId;

    use super::*;
    use crate::bus::Replay;
    use crate::entities::TaskRecord;

    struct Fixture {
        db: Arc<Database>,
        snapshots: SnapshotStore,
        graph: GraphAuthority,
        store: RunStore,
        _temp: tempfile::TempDir,
    }

    fn fixture() -> Fixture {
        let temp = tempfile::tempdir().expect("temp dir");
        let db = Arc::new(Database::open(&temp.path().join("hiverouter.db")).expect("db"));
        db.migrate().expect("migrate");
        let store = RunStore::new(db.clone());
        let snapshots = SnapshotStore::new(temp.path(), db.clone());
        Fixture {
            db,
            snapshots,
            graph: GraphAuthority::new(),
            store,
            _temp: temp,
        }
    }

    fn seed_task(
        fixture: &Fixture,
        run_id: RunId,
        parent: Option<TaskId>,
        seq: u64,
        state: AgentState,
    ) -> TaskRecord {
        let task = TaskRecord {
            id: TaskId::new(),
            run_id,
            parent_task_id: parent,
            role: "worker".to_owned(),
            depth: u8::try_from(seq).unwrap_or(1),
            spawn_seq: seq,
            state,
            rev: 0,
        };
        fixture.store.insert_task(&task).expect("task row");
        fixture
            .graph
            .add_task(task.id, parent)
            .expect("graph node");
        fixture
            .graph
            .set_state(task.id, state)
            .expect("graph state");
        task
    }

    fn seed_snapshot(fixture: &Fixture, run_id: RunId, revision: u64, status: AgentState) {
        let envelope = snapshot::capture(&fixture.graph, run_id, revision, status).expect("capture");
        fixture.snapshots.write(&envelope).expect("snapshot");
    }

    fn assert_db_and_graph_agree(fixture: &Fixture, run_id: RunId) {
        for task in fixture.store.tasks_for_run(run_id).expect("tasks") {
            assert_eq!(
                fixture.graph.state_of(task.id).expect("graph state"),
                task.state,
                "database and graph disagree for task {}",
                task.id
            );
        }
    }

    #[test]
    fn boot_recovery_terminates_tasks_of_interrupted_run() {
        let fixture = fixture();
        let run = fixture
            .store
            .create_run("p", "crashed while awaiting approval", AgentState::Running)
            .expect("run");
        let root = seed_task(&fixture, run.id, None, 1, AgentState::Running);
        let waiting = seed_task(
            &fixture,
            run.id,
            Some(root.id),
            2,
            AgentState::AwaitingApproval,
        );
        let queued = seed_task(&fixture, run.id, Some(root.id), 3, AgentState::Queued);
        let leaf = seed_task(&fixture, run.id, Some(root.id), 4, AgentState::Completed);
        seed_snapshot(&fixture, run.id, 0, AgentState::Running);

        let report = recover_on_boot(
            fixture.db.clone(),
            &fixture.graph,
            &fixture.snapshots,
        )
        .expect("recovery");

        assert!(report.interrupted_runs.contains(&run.id), "{report:?}");
        assert!(report.restored_runs.contains(&run.id), "{report:?}");
        assert!(
            report.graph_settle_failures.is_empty(),
            "{report:?}"
        );
        assert!(
            report.snapshot_write_failures.is_empty(),
            "{report:?}"
        );
        assert_eq!(
            report.settled_tasks.len(),
            3,
            "root, awaiting-approval and queued tasks settle; the completed leaf is untouched: {report:?}"
        );

        let stored = fixture.store.get_run(run.id).expect("run");
        assert_eq!(stored.status, AgentState::Interrupted);
        assert_db_and_graph_agree(&fixture, run.id);
        for task_id in [root.id, waiting.id, queued.id] {
            assert_eq!(
                fixture.store.get_task(task_id).expect("task").state,
                AgentState::Interrupted,
                "no non-terminal task may survive recovery"
            );
        }
        assert_eq!(
            fixture.store.get_task(leaf.id).expect("leaf").state,
            AgentState::Completed,
            "a terminal task is never resurrected by recovery"
        );

        // One new revision records the settled graph under the terminal status.
        let revisions = fixture
            .snapshots
            .list_revisions(run.id)
            .expect("revisions");
        assert_eq!(revisions, vec![0, 1], "recovery commits one new revision");
        let settled = fixture.snapshots.read(run.id, 1).expect("revision 1");
        assert_eq!(settled.run_status, Some(AgentState::Interrupted));
        let node_states: Vec<AgentState> = settled.graph.nodes.iter().map(|node| node.state).collect();
        assert!(
            node_states.iter().all(|state| state.is_terminal()),
            "the committed recovery snapshot must contain no non-terminal node: {node_states:?}"
        );
    }

    #[test]
    fn recovery_publishes_one_node_state_per_settled_task() {
        let fixture = fixture();
        let bus = EventBus::new();
        let run = fixture
            .store
            .create_run("p", "publish settlement", AgentState::Running)
            .expect("run");
        let root = seed_task(&fixture, run.id, None, 1, AgentState::Running);
        let child = seed_task(&fixture, run.id, Some(root.id), 2, AgentState::Queued);

        let report = recover_on_boot_with_events(
            fixture.db.clone(),
            &fixture.graph,
            &fixture.snapshots,
            Some(&bus),
        )
        .expect("recovery");
        assert_eq!(report.settled_tasks.len(), 2);

        let events = match bus.replay_after(0) {
            Replay::Events(events) => events,
            other => panic!("expected published settlement events, got {other:?}"),
        };
        assert_eq!(events.len(), 2, "one frame per settled task");
        for (sequenced, expected) in events.iter().zip([root.id, child.id]) {
            assert_eq!(sequenced.envelope.name.as_str(), "NODE_STATE");
            let payload = sequenced
                .envelope
                .payload
                .as_object()
                .expect("payload object");
            assert_eq!(payload["task_id"], expected.to_string());
            assert_eq!(payload["state"], "interrupted");
            assert!(
                payload["rev"].is_u64(),
                "the settlement frame carries the authoritative graph revision: {payload:?}"
            );
        }
        // The additive topology field carries the real parent link for a child
        // and is omitted for the root (absent stays absent).
        let child_payload = events[1].envelope.payload.as_object().expect("payload");
        assert_eq!(child_payload["parent_task_id"], root.id.to_string());
        let root_payload = events[0].envelope.payload.as_object().expect("payload");
        assert!(
            !root_payload.contains_key("parent_task_id"),
            "the root has no parent and must omit the field: {root_payload:?}"
        );
    }

    #[test]
    fn recovery_settles_a_run_that_was_already_interrupted() {
        let fixture = fixture();
        let run = fixture
            .store
            .create_run("p", "died mid-recovery", AgentState::Interrupted)
            .expect("run");
        let task = seed_task(&fixture, run.id, None, 1, AgentState::Running);

        let report = recover_on_boot(
            fixture.db.clone(),
            &fixture.graph,
            &fixture.snapshots,
        )
        .expect("recovery");

        assert!(report.interrupted_runs.is_empty(), "no second interruption");
        assert_eq!(report.settled_tasks.len(), 1);
        assert_eq!(report.settled_tasks[0].task_id, task.id);
        assert_eq!(
            fixture.store.get_task(task.id).expect("task").state,
            AgentState::Interrupted
        );
        assert_db_and_graph_agree(&fixture, run.id);
    }

    #[test]
    fn orphan_nodes_of_the_restored_crash_image_are_settled() {
        let fixture = fixture();
        let run = fixture
            .store
            .create_run("p", "snapshot without task rows", AgentState::Running)
            .expect("run");
        // The crash image has graph nodes but no task rows (a writer killed
        // between the graph insert and the row insert).
        let root = TaskId::new();
        let child = TaskId::new();
        fixture.graph.add_task(root, None).expect("root");
        fixture.graph.add_task(child, Some(root)).expect("child");
        fixture
            .graph
            .set_state(root, AgentState::Running)
            .expect("root state");
        seed_snapshot(&fixture, run.id, 0, AgentState::Running);

        let report = recover_on_boot(
            fixture.db.clone(),
            &fixture.graph,
            &fixture.snapshots,
        )
        .expect("recovery");

        assert!(report.restored_runs.contains(&run.id), "{report:?}");
        for task_id in [root, child] {
            assert_eq!(
                fixture.graph.state_of(task_id).expect("graph state"),
                AgentState::Interrupted,
                "a non-terminal node of the interrupted crash image must not survive"
            );
        }
        assert!(
            report.snapshot_write_failures.is_empty(),
            "the settled graph is committed: {report:?}"
        );
        assert_eq!(
            fixture
                .snapshots
                .latest(run.id)
                .expect("latest")
                .map(|envelope| envelope.run_status),
            Some(Some(AgentState::Interrupted))
        );
    }

    #[test]
    fn corrupt_snapshot_is_reported_and_tasks_are_still_settled() {
        let fixture = fixture();
        let run = fixture
            .store
            .create_run("p", "corrupt image", AgentState::Running)
            .expect("run");
        let task = seed_task(&fixture, run.id, None, 1, AgentState::Running);
        let envelope = snapshot::capture(&fixture.graph, run.id, 0, AgentState::Running)
            .expect("capture");
        let path = fixture.snapshots.write(&envelope).expect("write");
        std::fs::write(&path, "{corrupt").expect("corrupt");

        let report = recover_on_boot(
            fixture.db.clone(),
            &fixture.graph,
            &fixture.snapshots,
        )
        .expect("recovery");

        assert_eq!(report.corrupt_snapshots, vec![run.id]);
        assert!(!report.restored_runs.contains(&run.id));
        assert_eq!(report.settled_tasks.len(), 1);
        assert_eq!(
            fixture.store.get_task(task.id).expect("task").state,
            AgentState::Interrupted
        );
        assert_eq!(
            fixture
                .store
                .get_run(run.id)
                .expect("run")
                .status,
            AgentState::Interrupted
        );
    }

    #[test]
    fn catch_up_publisher_publishes_only_recovered_tasks() {
        let fixture = fixture();
        let bus = EventBus::new();
        let interrupted = fixture
            .store
            .create_run("p", "recovered", AgentState::Interrupted)
            .expect("run");
        let settled = seed_task(&fixture, interrupted.id, None, 1, AgentState::Interrupted);
        let completed = seed_task(
            &fixture,
            interrupted.id,
            Some(settled.id),
            2,
            AgentState::Completed,
        );
        let running = fixture
            .store
            .create_run("p", "still active", AgentState::Running)
            .expect("run");
        let active = seed_task(&fixture, running.id, None, 1, AgentState::Running);

        let published = publish_recovered_node_states(fixture.db.clone(), &fixture.graph, &bus).expect("publish");
        assert_eq!(published, 1, "only the recovered task is republished");
        let events = match bus.replay_after(0) {
            Replay::Events(events) => events,
            other => panic!("expected one frame, got {other:?}"),
        };
        assert_eq!(events.len(), 1);
        let payload = events[0].envelope.payload.as_object().expect("payload");
        assert_eq!(payload["task_id"], settled.id.to_string());
        assert_ne!(payload["task_id"], completed.id.to_string());
        assert_ne!(payload["task_id"], active.id.to_string());
        assert_eq!(events[0].envelope.run_id, Some(interrupted.id));
    }

    #[test]
    fn settlement_target_uses_the_task_state_machine() {
        assert_eq!(
            settlement_target(AgentState::Queued),
            AgentState::Interrupted
        );
        assert_eq!(
            settlement_target(AgentState::Running),
            AgentState::Interrupted
        );
        assert_eq!(
            settlement_target(AgentState::AwaitingApproval),
            AgentState::Interrupted
        );
        // A UI-only state is not a lifecycle edge; recovery still settles it.
        assert_eq!(
            settlement_target(AgentState::Thinking),
            AgentState::Interrupted
        );
        // Terminal states are not targets for a settlement decision.
        assert!(is_terminal_task_state(AgentState::Completed));
        assert_eq!(
            run_state::apply_task(AgentState::Completed, StateEvent::Restart)
                .expect_err("a terminal task has no restart edge")
                .code(),
            "E_TASK_TERMINAL"
        );
    }

    #[test]
    fn recovery_is_idempotent_on_a_second_boot() {
        let fixture = fixture();
        let run = fixture
            .store
            .create_run("p", "double boot", AgentState::Running)
            .expect("run");
        seed_task(&fixture, run.id, None, 1, AgentState::Running);
        seed_snapshot(&fixture, run.id, 0, AgentState::Running);
        let first = recover_on_boot(
            fixture.db.clone(),
            &fixture.graph,
            &fixture.snapshots,
        )
        .expect("first recovery");
        let second = recover_on_boot(
            fixture.db.clone(),
            &fixture.graph,
            &fixture.snapshots,
        )
        .expect("second recovery");
        assert_eq!(first.settled_tasks.len(), 1);
        assert!(
            second.settled_tasks.is_empty(),
            "a second boot finds no non-terminal task: {second:?}"
        );
        assert!(second.interrupted_runs.is_empty());
        assert_db_and_graph_agree(&fixture, run.id);
    }

    #[test]
    fn next_revision_is_monotonic_across_index_and_disk() {
        let fixture = fixture();
        let run = fixture
            .store
            .create_run("p", "revisions", AgentState::Completed)
            .expect("run");
        assert_eq!(next_revision(&fixture.snapshots, run.id), 0);
        let envelope = snapshot::capture(&fixture.graph, run.id, 0, AgentState::Completed)
            .expect("capture");
        fixture.snapshots.write(&envelope).expect("write");
        assert_eq!(next_revision(&fixture.snapshots, run.id), 1);
    }

    #[test]
    fn recovery_report_is_empty_for_a_clean_boot() {
        let fixture = fixture();
        fixture
            .store
            .create_run("p", "finished", AgentState::Completed)
            .expect("run");
        let report = recover_on_boot(
            fixture.db.clone(),
            &fixture.graph,
            &fixture.snapshots,
        )
        .expect("recovery");
        assert!(report.is_empty(), "{report:?}");
    }
}
