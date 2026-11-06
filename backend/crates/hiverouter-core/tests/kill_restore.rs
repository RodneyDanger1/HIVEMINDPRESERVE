//! Phase 9 acceptance: kill-restore. A helper process writes a running run + a
//! committed snapshot and is then hard-killed (`std::process::abort`); the next
//! boot must mark the run interrupted and restore the snapshot into the graph.
//!
//! Blueprint card command: `kill_restore`.

use std::process::Command;
use std::sync::Arc;

use hiverouter_contract::{AgentState, Refusal, RunId, TaskId};
use hiverouter_core::db::Database;
use hiverouter_core::graph::GraphAuthority;
use hiverouter_core::recovery::recover_on_boot;
use hiverouter_core::snapshot::{capture, SnapshotStore};
use hiverouter_core::store::RunStore;

/// Helper process body. Only runs when the parent test sets the directory env
/// var; it never returns (hard kill) so no shutdown path can hide a durability
/// bug.
#[test]
#[ignore = "spawned by the kill_restore test as the crashing writer process"]
fn crash_writer_helper() {
    let Ok(directory) = std::env::var("HIVEROUTER_CRASH_HELPER_DIR") else {
        return;
    };
    let path = std::path::Path::new(&directory);
    std::fs::create_dir_all(path).expect("helper dir");
    let db = Arc::new(Database::open(&path.join("hiverouter.db")).expect("helper db"));
    db.migrate().expect("helper migrate");
    let store = RunStore::new(db.clone());
    let run = store
        .create_run("p", "killed mid-run", AgentState::Running)
        .expect("helper run");
    let graph = GraphAuthority::new();
    let root = TaskId::new();
    let child = TaskId::new();
    graph.add_task(root, None).expect("root");
    graph.add_task(child, Some(root)).expect("child");
    let snapshots = SnapshotStore::new(path, db.clone());
    let envelope = capture(&graph, run.id, 0, AgentState::Running).expect("capture");
    snapshots.write(&envelope).expect("write snapshot");
    std::fs::write(path.join("run_id.txt"), run.id.to_string()).expect("run id");
    // Hard kill: no destructors, no SQLite close, no graceful shutdown.
    std::process::abort();
}

#[test]
fn kill_restore() {
    let temp = tempfile::tempdir().expect("temp dir");
    let executable = std::env::current_exe().expect("current test exe");
    let status = Command::new(&executable)
        .args([
            "crash_writer_helper",
            "--exact",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("HIVEROUTER_CRASH_HELPER_DIR", temp.path())
        .status()
        .expect("spawn crash helper");
    assert!(
        !status.success(),
        "the crash helper must be killed, exit status was {status:?}"
    );

    let run_id: RunId = std::fs::read_to_string(temp.path().join("run_id.txt"))
        .expect("run id file")
        .trim()
        .parse()
        .expect("run uuid");
    let db = Arc::new(Database::open(&temp.path().join("hiverouter.db")).expect("recover db"));
    db.migrate().expect("migrate is idempotent after the kill");

    let graph = GraphAuthority::new();
    let snapshots = SnapshotStore::new(temp.path(), db.clone());
    let report = recover_on_boot(db.clone(), &graph, &snapshots).expect("recovery");
    assert!(
        report.interrupted_runs.contains(&run_id),
        "run killed mid-flight must be marked interrupted: {report:?}"
    );
    assert!(
        report.restored_runs.contains(&run_id),
        "latest committed snapshot must be restored: {report:?}"
    );
    let run = RunStore::new(db).get_run(run_id).expect("run row");
    assert_eq!(run.status, AgentState::Interrupted);
    assert_eq!(graph.node_count().expect("nodes restored"), 2);
}

#[test]
fn corrupt_snapshot_at_recovery_is_refused_not_reset() {
    let temp = tempfile::tempdir().expect("temp dir");
    let db = Arc::new(Database::open(&temp.path().join("hiverouter.db")).expect("db"));
    db.migrate().expect("migrate");
    let store = RunStore::new(db.clone());
    let run = store
        .create_run("p", "corrupt", AgentState::Running)
        .expect("run");
    let graph = GraphAuthority::new();
    let root = TaskId::new();
    graph.add_task(root, None).expect("root");
    let snapshots = SnapshotStore::new(temp.path(), db.clone());
    let envelope = capture(&graph, run.id, 0, AgentState::Running).expect("capture");
    let path = snapshots.write(&envelope).expect("write");
    std::fs::write(&path, "{corrupt").expect("corrupt");

    let report = recover_on_boot(db.clone(), &graph, &snapshots).expect("recovery");
    assert_eq!(report.corrupt_snapshots, vec![run.id]);
    assert!(!report.restored_runs.contains(&run.id));
    let stored = RunStore::new(db).get_run(run.id).expect("run");
    assert_eq!(stored.status, AgentState::Interrupted, "run row still recovered");
    assert_eq!(
        snapshots.read(run.id, 0).expect_err("corrupt"),
        Refusal::SnapshotCorrupt
    );
}
