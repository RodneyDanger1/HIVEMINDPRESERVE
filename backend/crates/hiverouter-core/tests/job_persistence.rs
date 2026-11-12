//! v28 job persistence: terminal snapshots survive process restart.

use std::sync::Arc;

use hiverouter_contract::Timestamp;
use hiverouter_core::db::{Database, MIGRATIONS};
use hiverouter_core::jobs::{
    list_persisted_jobs, persist_job, JobOutput, JobSnapshot, JobState,
};

fn snapshot(id: &str, state: JobState, ended_at: Option<Timestamp>) -> JobSnapshot {
    JobSnapshot {
        id: id.to_owned(),
        session_id: "session-1".to_owned(),
        program: "git.exe".to_owned(),
        args: vec!["status".to_owned(), "--short".to_owned()],
        cwd: ".".to_owned(),
        state,
        pid: Some(4242),
        exit_code: Some(0),
        started_at: Timestamp::now(),
        ended_at,
        timeout_ms: 300_000,
        output: JobOutput {
            tail: "clean\n".to_owned(),
            bytes: 6,
            truncated: false,
        },
    }
}

#[test]
fn v28_records_terminal_jobs_and_reads_them_back() {
    let temp = tempfile::tempdir().expect("temp");
    let db = Arc::new(Database::open(&temp.path().join("hive.db")).expect("db"));
    let version = db.migrate().expect("migrate");
    assert_eq!(version, MIGRATIONS.last().expect("last migration").version);
    assert!(version >= 28, "job persistence migration must be installed");

    let running = snapshot("job-running", JobState::Running, None);
    db.with_connection(|connection| persist_job(connection, &running))
        .expect("persist running job");
    let terminal = snapshot("job-done", JobState::Succeeded, Some(Timestamp::now()));
    db.with_connection(|connection| persist_job(connection, &terminal))
        .expect("persist terminal job");

    let stored = db
        .with_connection(|connection| list_persisted_jobs(connection, 10))
        .expect("list jobs");
    assert_eq!(stored.len(), 2);
    let done = stored.iter().find(|job| job.id == "job-done").expect("done job");
    assert_eq!(done.state, JobState::Succeeded);
    assert_eq!(done.exit_code, Some(0));
    assert_eq!(done.output.tail, "clean\n");
    assert_eq!(done.args, vec!["status", "--short"]);
    let live = stored.iter().find(|job| job.id == "job-running").expect("running job");
    assert_eq!(live.state, JobState::Running);
    assert!(live.ended_at.is_none());

    // Upsert is idempotent and updates the terminal transition.
    let mut updated = running.clone();
    updated.state = JobState::Failed;
    updated.exit_code = Some(1);
    updated.ended_at = Some(Timestamp::now());
    db.with_connection(|connection| persist_job(connection, &updated))
        .expect("update job");
    let stored = db
        .with_connection(|connection| list_persisted_jobs(connection, 10))
        .expect("list jobs again");
    assert_eq!(stored.len(), 2, "upsert must not duplicate rows");
    let live = stored.iter().find(|job| job.id == "job-running").expect("running job");
    assert_eq!(live.state, JobState::Failed);
    assert_eq!(live.exit_code, Some(1));
}
