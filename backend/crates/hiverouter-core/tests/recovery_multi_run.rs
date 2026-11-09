//! Boot recovery across two interrupted runs: every interrupted run is
//! represented, the single-image degradation is named, and a missing graph node
//! is rebuilt from its authoritative task row (crash between graph insert and
//! row insert, or a superseded snapshot image).
//!
//! HIGH/MED audit finding #7/#8, store/recovery side only: execution.rs is not
//! touched.

use std::sync::Arc;

use hiverouter_contract::{AgentState, TaskId};
use hiverouter_core::db::Database;
use hiverouter_core::entities::TaskRecord;
use hiverouter_core::graph::GraphAuthority;
use hiverouter_core::recovery::recover_on_boot;
use hiverouter_core::snapshot::{self, SnapshotStore};
use hiverouter_core::store::RunStore;

fn seed_task(
    store: &RunStore,
    run_id: hiverouter_contract::RunId,
    parent: Option<TaskId>,
    seq: u64,
    state: AgentState,
) -> TaskRecord {
    let task = TaskRecord {
        id: TaskId::new(),
        run_id,
        parent_task_id: parent,
        role: if seq == 0 { "planner" } else { "worker" }.to_owned(),
        depth: u8::try_from(seq).unwrap_or(1),
        spawn_seq: seq,
        state,
        rev: 0,
    };
    store.insert_task(&task).expect("task row");
    task
}

fn write_snapshot(
    graph: &GraphAuthority,
    snapshots: &SnapshotStore,
    run_id: hiverouter_contract::RunId,
    tasks: &[&TaskRecord],
) {
    for task in tasks {
        graph
            .add_task(task.id, task.parent_task_id)
            .expect("graph node");
    }
    let envelope =
        snapshot::capture(graph, run_id, 0, AgentState::Running).expect("snapshot capture");
    snapshots.write(&envelope).expect("snapshot write");
}

#[test]
fn two_interrupted_runs_are_both_represented_and_degradation_is_named() {
    let temp = tempfile::tempdir().expect("temp dir");
    let db = Arc::new(Database::open(&temp.path().join("hiverouter.db")).expect("db"));
    db.migrate().expect("migrate");
    let store = RunStore::new(db.clone());
    let snapshots = SnapshotStore::new(temp.path(), db.clone());

    // Two active runs, each with a still-running root and a live child.
    let run_old = store
        .create_run("p", "older interrupted run", AgentState::Running)
        .expect("run old");
    let root_old = seed_task(&store, run_old.id, None, 0, AgentState::Running);
    let done_old = seed_task(
        &store,
        run_old.id,
        Some(root_old.id),
        1,
        AgentState::Completed,
    );

    let run_new = store
        .create_run("p", "newer interrupted run", AgentState::Running)
        .expect("run new");
    let root_new = seed_task(&store, run_new.id, None, 0, AgentState::Running);
    let child_new = seed_task(&store, run_new.id, Some(root_new.id), 1, AgentState::Queued);

    // Each run has a committed crash image. Only one can live in the graph.
    let old_graph = GraphAuthority::new();
    write_snapshot(&old_graph, &snapshots, run_old.id, &[&root_old, &done_old]);
    let new_graph = GraphAuthority::new();
    write_snapshot(&new_graph, &snapshots, run_new.id, &[&root_new, &child_new]);

    let graph = GraphAuthority::new();
    let report = recover_on_boot(db.clone(), &graph, &snapshots).expect("recover");

    // Both runs interrupted, exactly one snapshot image restored, the other
    // named as degraded rather than silently dropped.
    assert_eq!(report.interrupted_runs.len(), 2, "{report:?}");
    assert!(report.interrupted_runs.contains(&run_old.id), "{report:?}");
    assert!(report.interrupted_runs.contains(&run_new.id), "{report:?}");
    assert_eq!(report.restored_runs.len(), 1, "{report:?}");
    assert_eq!(report.not_restored_runs.len(), 1, "{report:?}");
    let restored = report.restored_runs[0];
    let degraded = report.not_restored_runs[0];
    assert_ne!(restored, degraded);
    assert!(
        [run_old.id, run_new.id].contains(&restored) && [run_old.id, run_new.id].contains(&degraded),
        "restored/degraded must partition the two runs: {report:?}"
    );

    // Every task row of every interrupted run has a graph node, including the
    // terminal child that no settlement pass would otherwise create.
    for task in [&root_old, &done_old, &root_new, &child_new] {
        assert!(
            graph.contains(task.id).expect("graph contains"),
            "task {} has no graph node after recovery",
            task.id
        );
    }
    assert_eq!(
        graph.state_of(root_old.id).expect("root old"),
        AgentState::Interrupted
    );
    assert_eq!(
        graph.state_of(root_new.id).expect("root new"),
        AgentState::Interrupted
    );
    assert_eq!(
        graph.state_of(child_new.id).expect("child new"),
        AgentState::Interrupted
    );
    // The terminal row is preserved, not rewritten, and its rebuilt node agrees.
    assert_eq!(
        graph.state_of(done_old.id).expect("done old"),
        AgentState::Completed,
        "a terminal task must not be resurrected or rewritten by recovery"
    );

    // No fake success: the database says interrupted for both runs and no
    // terminal task was re-settled.
    for run_id in [run_old.id, run_new.id] {
        assert_eq!(
            store.get_run(run_id).expect("run").status,
            AgentState::Interrupted
        );
    }
    // root_old, root_new and child_new were non-terminal; done_old stays.
    assert_eq!(report.settled_tasks.len(), 3, "{report:?}");
    assert!(
        report
            .settled_tasks
            .iter()
            .any(|settled| settled.task_id == child_new.id),
        "the restored image's queued child must be settled: {report:?}"
    );
    assert!(
        report
            .settled_tasks
            .iter()
            .all(|settled| settled.task_id != done_old.id),
        "terminal task must not appear in settled_tasks"
    );
    assert!(report.graph_settle_failures.is_empty(), "{report:?}");
    assert!(report.snapshot_write_failures.is_empty(), "{report:?}");
}
