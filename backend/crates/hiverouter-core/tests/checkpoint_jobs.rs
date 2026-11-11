//! CAP-5 acceptance tests: content-addressed checkpoint journal + bounded
//! background jobs.
//!
//! Checkpoint tests are platform-neutral except the junction-escape case (the
//! workspace jail is Windows-first). Job tests execute real allowlisted
//! children (`git.exe`) and therefore run on Windows, matching the existing
//! `shell.rs` timeout/kill tests.

use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use hiverouter_contract::{Refusal, RunId, Timestamp};
use hiverouter_core::checkpoint::{
    CheckpointError, CheckpointJournal, FileState, CHECKPOINT_MIGRATION_NAME,
};
use hiverouter_core::db::Database;
use hiverouter_core::jobs::{
    JobError, JobPolicy, JobRegistry, JobRequest, JobState, MAX_CONCURRENT_JOBS,
};
use hiverouter_tools::sha256_hex;

// --- checkpoints -----------------------------------------------------------

fn fixture() -> (tempfile::TempDir, Arc<Database>, CheckpointJournal) {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path().join("workspace");
    fs::create_dir_all(&workspace).expect("workspace dir");
    let database = Arc::new(Database::open_in_memory().expect("in-memory database"));
    database.migrate().expect("migrations");
    assert!(
        database
            .applied_migrations()
            .expect("migrations")
            .iter()
            .any(|(_, name, _)| name == CHECKPOINT_MIGRATION_NAME),
        "the Lead must append checkpoint::checkpoint_migration(<version>) to db::MIGRATIONS"
    );
    let journal = CheckpointJournal::open(database.clone(), &workspace).expect("journal");
    (temp, database, journal)
}

fn insert_run(database: &Database) -> RunId {
    let run_id = RunId::new();
    database
        .with_connection(|connection| {
            connection.execute(
                "INSERT INTO runs(id, project_id, prompt, status, created_at, updated_at)
                 VALUES (?1, 'project', 'checkpoint test', 'running', ?2, ?2)",
                rusqlite::params![run_id.to_string(), Timestamp::now().as_str()],
            )?;
            Ok(())
        })
        .expect("insert run row");
    run_id
}

fn write_file(root: &Path, relative: &str, bytes: &[u8]) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent dir");
    }
    fs::write(path, bytes).expect("write test file");
}

#[test]
fn checkpoint_records_sha_and_restores_exact_bytes() {
    let (_temp, database, journal) = fixture();
    let run = insert_run(&database);
    let root = journal.jail().root().to_path_buf();
    write_file(&root, "notes/one.txt", b"alpha");

    assert_eq!(journal.next_seq(run).expect("next seq"), 1);
    let before = journal
        .journal_before(run, 1, "notes/one.txt")
        .expect("journal before");
    assert_eq!(
        before.before,
        FileState::Present {
            sha256: sha256_hex(b"alpha"),
            size: 5,
        },
        "the before-state must be the byte-exact pre-image"
    );
    assert!(
        before.after.is_none(),
        "no after-state is known before the caller mutates the file"
    );
    assert_eq!(before.path, "notes/one.txt");
    assert_eq!(
        fs::read(root.join("notes/one.txt")).expect("still there"),
        b"alpha",
        "journalling must not mutate the file"
    );

    write_file(&root, "notes/one.txt", b"beta");
    let completed = journal.record_after(run, 1).expect("record after");
    assert_eq!(
        completed.after,
        Some(FileState::Present {
            sha256: sha256_hex(b"beta"),
            size: 4,
        })
    );
    assert!(completed.completed_at.is_some());

    let restored = journal.restore_entry(run, 1).expect("restore entry");
    assert_eq!(restored.path, "notes/one.txt");
    assert_eq!(restored.undone_seqs, vec![1]);
    assert_eq!(
        restored.restored,
        FileState::Present {
            sha256: sha256_hex(b"alpha"),
            size: 5,
        }
    );
    assert_eq!(
        fs::read(root.join("notes/one.txt")).expect("restored file"),
        b"alpha",
        "restore must be byte-exact"
    );
    let stored = journal.get(run, 1).expect("get").expect("row");
    assert!(stored.restored_at.is_some(), "restore must be recorded");
}

#[test]
fn checkpoint_of_a_new_file_restores_absence() {
    let (_temp, database, journal) = fixture();
    let run = insert_run(&database);
    let root = journal.jail().root().to_path_buf();

    let before = journal
        .journal_before(run, 1, "created/later.txt")
        .expect("journal absent file");
    assert_eq!(before.before, FileState::Absent);

    write_file(&root, "created/later.txt", b"new content");
    let completed = journal.record_after(run, 1).expect("record after");
    assert_eq!(
        completed.after,
        Some(FileState::Present {
            sha256: sha256_hex(b"new content"),
            size: 11,
        })
    );

    let restored = journal.restore_entry(run, 1).expect("restore");
    assert_eq!(restored.restored, FileState::Absent);
    assert!(
        !root.join("created/later.txt").exists(),
        "restoring a created file must remove it again"
    );
}

#[test]
fn external_drift_is_refused_and_the_file_is_left_alone() {
    let (_temp, database, journal) = fixture();
    let run = insert_run(&database);
    let root = journal.jail().root().to_path_buf();
    write_file(&root, "drift.txt", b"v0");

    journal
        .journal_before(run, 1, "drift.txt")
        .expect("journal");
    write_file(&root, "drift.txt", b"v1");
    journal.record_after(run, 1).expect("record after");

    write_file(&root, "drift.txt", b"external edit");
    let error = journal
        .restore_entry(run, 1)
        .expect_err("external drift must be refused");
    assert!(
        matches!(error, CheckpointError::Drift { .. }),
        "expected drift refusal, got {error:?}"
    );
    assert_eq!(
        fs::read(root.join("drift.txt")).expect("file"),
        b"external edit",
        "a refused restore must not overwrite the external edit"
    );
    assert!(
        journal
            .get(run, 1)
            .expect("get")
            .expect("row")
            .restored_at
            .is_none(),
        "a refused restore must not be recorded as restored"
    );

    // Deletion is drift too: the recorded after-state says the file exists.
    fs::remove_file(root.join("drift.txt")).expect("remove");
    let error = journal.restore_entry(run, 1).expect_err("deletion drift");
    assert!(
        matches!(error, CheckpointError::Drift { .. }),
        "got {error:?}"
    );
    assert!(
        !root.join("drift.txt").exists(),
        "a refused restore must not recreate the deleted file"
    );
}

#[test]
fn checkpoint_refuses_paths_outside_the_jail() {
    let (_temp, database, journal) = fixture();
    let run = insert_run(&database);
    for attempt in ["../escape.txt", "a/../../escape.txt", "/etc/passwd"] {
        let error = journal.journal_before(run, 1, attempt).expect_err(attempt);
        assert_eq!(
            error.refusal(),
            Refusal::PathOutsideJail,
            "vector {attempt:?}: {error}"
        );
    }
    for attempt in [
        r"C:\Windows\win.ini",
        r"\\server\share\x.txt",
        "PROGRA~1\\x.txt",
    ] {
        let error = journal.journal_before(run, 1, attempt).expect_err(attempt);
        assert_eq!(
            error.refusal(),
            Refusal::PathOutsideJail,
            "vector {attempt:?}: {error}"
        );
    }
}

#[cfg(windows)]
#[test]
fn checkpoint_refuses_symlink_or_junction_escape() {
    let (temp, database, journal) = fixture();
    let outside = tempfile::tempdir().expect("outside tempdir");
    fs::write(outside.path().join("secret.txt"), b"outside the jail").expect("secret");
    let link = journal.jail().root().join("link");
    // Junction creation needs no administrator privilege on NTFS.
    let status = std::process::Command::new("cmd")
        .args([
            "/C",
            "mklink",
            "/J",
            &link.to_string_lossy(),
            &outside.path().to_string_lossy(),
        ])
        .output()
        .expect("run mklink");
    if !status.status.success() {
        let symlink = std::os::windows::fs::symlink_dir(outside.path(), &link);
        assert!(
            symlink.is_ok(),
            "cannot create a junction or symlink to exercise the escape vector: {}",
            String::from_utf8_lossy(&status.stderr)
        );
    }
    let run = insert_run(&database);
    let error = journal
        .journal_before(run, 1, "link/secret.txt")
        .expect_err("symlink escape refused");
    assert_eq!(error.refusal(), Refusal::PathOutsideJail);
    assert!(
        !format!("{error}").contains("outside the jail"),
        "refusal detail must not echo escaped content"
    );
    let _ = temp;
}

#[test]
fn restore_to_rewinds_the_run_sequence_and_refuses_chain_drift() {
    let (_temp, database, journal) = fixture();
    let run = insert_run(&database);
    let root = journal.jail().root().to_path_buf();
    write_file(&root, "sequence.txt", b"v0");

    journal
        .journal_before(run, 1, "sequence.txt")
        .expect("seq 1");
    write_file(&root, "sequence.txt", b"v1");
    journal.record_after(run, 1).expect("after 1");
    journal
        .journal_before(run, 2, "sequence.txt")
        .expect("seq 2");
    write_file(&root, "sequence.txt", b"v2");
    journal.record_after(run, 2).expect("after 2");

    let outcome = journal.restore_to(run, 1).expect("rewind");
    assert_eq!(outcome.from_seq, 1);
    assert_eq!(outcome.restored.len(), 1);
    assert_eq!(outcome.restored[0].undone_seqs, vec![2, 1]);
    assert_eq!(
        outcome.restored[0].restored,
        FileState::Present {
            sha256: sha256_hex(b"v0"),
            size: 2,
        }
    );
    assert_eq!(
        fs::read(root.join("sequence.txt")).expect("sequence"),
        b"v0",
        "rewinding both mutations restores the original bytes"
    );
}

#[test]
fn restore_to_validates_every_path_before_writing_anything() {
    let (_temp, database, journal) = fixture();
    let run = insert_run(&database);
    let root = journal.jail().root().to_path_buf();
    write_file(&root, "first.txt", b"a0");
    write_file(&root, "second.txt", b"b0");

    journal
        .journal_before(run, 1, "first.txt")
        .expect("first seq 1");
    write_file(&root, "first.txt", b"a1");
    journal.record_after(run, 1).expect("first after");
    journal
        .journal_before(run, 2, "second.txt")
        .expect("second seq 2");
    write_file(&root, "second.txt", b"b1");
    journal.record_after(run, 2).expect("second after");

    // Drift on the *second* path: the first path must not be restored.
    write_file(&root, "second.txt", b"external");
    let error = journal.restore_to(run, 1).expect_err("drift");
    assert!(
        matches!(error, CheckpointError::Drift { .. }),
        "got {error:?}"
    );
    assert_eq!(
        fs::read(root.join("first.txt")).expect("first"),
        b"a1",
        "no path may be restored when the set fails validation"
    );
    assert_eq!(
        fs::read(root.join("second.txt")).expect("second"),
        b"external"
    );
}

#[test]
fn restore_refuses_an_incomplete_checkpoint() {
    let (_temp, database, journal) = fixture();
    let run = insert_run(&database);
    let root = journal.jail().root().to_path_buf();
    write_file(&root, "incomplete.txt", b"before");
    journal
        .journal_before(run, 1, "incomplete.txt")
        .expect("journal");
    write_file(&root, "incomplete.txt", b"after");

    let error = journal.restore_entry(run, 1).expect_err("incomplete");
    assert!(
        matches!(error, CheckpointError::Incomplete { .. }),
        "got {error:?}"
    );
    assert_eq!(
        fs::read(root.join("incomplete.txt")).expect("file"),
        b"after",
        "an incomplete entry must never be guessed"
    );
}

#[test]
fn checkpoint_size_cap_is_enforced() {
    let (_temp, database, journal) = fixture();
    let run = insert_run(&database);
    let root = journal.jail().root().to_path_buf();
    write_file(&root, "big.txt", b"12345");
    let journal = journal.with_max_file_bytes(4);
    let error = journal
        .journal_before(run, 1, "big.txt")
        .expect_err("too large");
    assert_eq!(error.refusal(), Refusal::FileTooLarge);
    assert!(
        journal.get(run, 1).expect("get").is_none(),
        "an oversized file must not create a journal entry"
    );
}

#[test]
fn corrupt_content_addressed_blob_is_refused() {
    let (_temp, database, journal) = fixture();
    let run = insert_run(&database);
    let root = journal.jail().root().to_path_buf();
    write_file(&root, "corrupt.txt", b"original");
    let checkpoint = journal
        .journal_before(run, 1, "corrupt.txt")
        .expect("journal");
    write_file(&root, "corrupt.txt", b"mutated");
    journal.record_after(run, 1).expect("record after");

    let sha = checkpoint.before.sha256().expect("sha").to_owned();
    database
        .with_connection(|connection| {
            connection.execute(
                "UPDATE checkpoint_blobs SET bytes = X'00' WHERE sha256 = ?1",
                rusqlite::params![sha],
            )?;
            Ok(())
        })
        .expect("corrupt blob");

    let error = journal
        .restore_entry(run, 1)
        .expect_err("corrupt blob refused");
    assert!(
        matches!(error, CheckpointError::Corrupt(_)),
        "got {error:?}"
    );
    assert_eq!(
        fs::read(root.join("corrupt.txt")).expect("file"),
        b"mutated",
        "a corrupt pre-image must never be written"
    );
}

// --- background jobs -------------------------------------------------------

#[cfg(windows)]
fn sleep_request(session: &str, seconds: &str) -> JobRequest {
    JobRequest::new(
        session,
        "git.exe",
        vec![
            "-c".to_owned(),
            format!("alias.hive-sleep=!sleep {seconds}"),
            "hive-sleep".to_owned(),
        ],
    )
}

#[cfg(windows)]
fn process_exists(pid: u32) -> bool {
    let output = std::process::Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
        .stdin(std::process::Stdio::null())
        .output()
        .expect("tasklist runs");
    assert!(
        output.status.success(),
        "tasklist failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8_lossy(&output.stdout);
    text.split(',')
        .any(|field| field.trim().trim_matches('"') == pid.to_string())
}

#[cfg(windows)]
async fn wait_process_gone(pid: u32) -> bool {
    for _ in 0..80 {
        if !process_exists(pid) {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    false
}

#[cfg(windows)]
#[tokio::test]
async fn job_cap_is_four_and_the_fifth_is_refused() {
    let temp = tempfile::tempdir().expect("tempdir");
    let registry = JobRegistry::open(
        temp.path(),
        JobPolicy::conservative_default().with_timeout(Duration::from_secs(30)),
    )
    .expect("registry");

    let mut ids = Vec::new();
    for index in 0..MAX_CONCURRENT_JOBS {
        let snapshot = registry
            .start(sleep_request(&format!("session-{index}"), "30"))
            .await
            .expect("start within cap");
        assert_eq!(snapshot.state, JobState::Running);
        ids.push(snapshot.id);
    }
    assert_eq!(registry.running_count(), MAX_CONCURRENT_JOBS);

    let error = registry
        .start(sleep_request("session-overflow", "30"))
        .await
        .expect_err("the fifth job must be refused");
    assert!(
        matches!(error, JobError::ConcurrencyLimit { cap } if cap == MAX_CONCURRENT_JOBS),
        "got {error:?}"
    );
    assert_eq!(error.refusal(), Refusal::SemaphoreFull);
    assert_eq!(
        registry.running_count(),
        MAX_CONCURRENT_JOBS,
        "the refused job must not have started"
    );

    for id in &ids {
        let snapshot = registry.kill(id).await.expect("kill");
        assert_eq!(snapshot.state, JobState::Killed);
    }
    assert_eq!(registry.running_count(), 0);
    // The permit must be released: capacity is available again.
    let replacement = registry
        .start(sleep_request("session-replacement", "30"))
        .await
        .expect("capacity released");
    registry
        .kill(&replacement.id)
        .await
        .expect("kill replacement");
}

#[cfg(windows)]
#[tokio::test]
async fn kill_terminates_the_job_process_tree() {
    let temp = tempfile::tempdir().expect("tempdir");
    let registry = JobRegistry::open(
        temp.path(),
        JobPolicy::conservative_default().with_timeout(Duration::from_secs(30)),
    )
    .expect("registry");
    let snapshot = registry
        .start(sleep_request("session-kill", "30"))
        .await
        .expect("start");
    let pid = snapshot.pid.expect("pid");
    assert!(process_exists(pid), "job process must be alive before kill");

    let killed = registry.kill(&snapshot.id).await.expect("kill");
    assert_eq!(killed.state, JobState::Killed);
    assert!(killed.ended_at.is_some());
    assert!(
        wait_process_gone(pid).await,
        "job process {pid} must be gone after kill"
    );
}

#[cfg(windows)]
#[tokio::test]
async fn timeout_terminates_the_job() {
    let temp = tempfile::tempdir().expect("tempdir");
    let registry = JobRegistry::open(
        temp.path(),
        JobPolicy::conservative_default().with_timeout(Duration::from_millis(600)),
    )
    .expect("registry");
    let snapshot = registry
        .start(sleep_request("session-timeout", "30"))
        .await
        .expect("start");
    let pid = snapshot.pid.expect("pid");
    let terminal = tokio::time::timeout(Duration::from_secs(15), registry.wait(&snapshot.id))
        .await
        .expect("wait timed out")
        .expect("terminal");
    assert_eq!(terminal.state, JobState::TimedOut);
    assert!(
        wait_process_gone(pid).await,
        "timed-out process {pid} must be terminated"
    );
    assert_eq!(registry.running_count(), 0);
}

#[cfg(windows)]
#[tokio::test]
async fn output_tail_is_bounded_and_captured() {
    let temp = tempfile::tempdir().expect("tempdir");
    let registry = JobRegistry::open(
        temp.path(),
        JobPolicy::conservative_default()
            .with_timeout(Duration::from_secs(30))
            .with_output_tail_bytes(8),
    )
    .expect("registry");
    let snapshot = registry
        .start(JobRequest::new(
            "session-output",
            "git.exe",
            vec!["--version".to_owned()],
        ))
        .await
        .expect("start");
    let terminal = registry.wait(&snapshot.id).await.expect("terminal");
    assert_eq!(terminal.state, JobState::Succeeded);
    assert_eq!(terminal.exit_code, Some(0));
    assert!(terminal.output.bytes > 0, "output must be observed");
    assert!(
        terminal.output.tail.len() <= 8,
        "tail {:?} exceeds the configured bound",
        terminal.output.tail
    );
    assert!(terminal.output.truncated, "the tail must report truncation");
}

#[cfg(windows)]
#[tokio::test]
async fn session_cleanup_kills_and_removes_only_that_session() {
    let temp = tempfile::tempdir().expect("tempdir");
    let registry = JobRegistry::open(
        temp.path(),
        JobPolicy::conservative_default().with_timeout(Duration::from_secs(30)),
    )
    .expect("registry");
    let first = registry
        .start(sleep_request("session-a", "30"))
        .await
        .expect("start a");
    let second = registry
        .start(sleep_request("session-b", "30"))
        .await
        .expect("start b");
    let first_pid = first.pid.expect("pid");

    let removed = registry
        .cleanup_session("session-a")
        .await
        .expect("cleanup");
    assert_eq!(removed, 1);
    assert!(
        matches!(registry.get(&first.id), Err(JobError::UnknownJob(_))),
        "cleaned job must be removed"
    );
    assert!(
        registry.list_session("session-a").is_empty(),
        "session cleanup must leave no entries"
    );
    assert_eq!(
        registry.get(&second.id).expect("b still registered").state,
        JobState::Running,
        "another session's job must keep running"
    );
    assert!(
        wait_process_gone(first_pid).await,
        "session cleanup must terminate the process"
    );

    assert_eq!(
        registry
            .cleanup_session("session-b")
            .await
            .expect("cleanup b"),
        1
    );
    assert_eq!(registry.running_count(), 0);
}

#[cfg(windows)]
#[tokio::test]
async fn shell_interpreters_paths_and_escaping_cwds_are_refused() {
    let temp = tempfile::tempdir().expect("tempdir");
    let registry =
        JobRegistry::open(temp.path(), JobPolicy::conservative_default()).expect("registry");
    for program in [
        "cmd.exe",
        "powershell.exe",
        "C:\\Windows\\System32\\cmd.exe",
        "..\\git.exe",
        "where.exe",
    ] {
        let error = registry
            .start(JobRequest::new("session-refusal", program, vec![]))
            .await
            .expect_err(program);
        assert_eq!(
            error.refusal(),
            Refusal::ToolNotAllowed,
            "program {program:?}: {error}"
        );
    }

    // An interpreter host is denied even when a policy explicitly lists it.
    let interpreter_policy =
        JobRegistry::open(temp.path(), JobPolicy::new(["cmd.exe".to_owned()])).expect("registry");
    let error = interpreter_policy
        .start(JobRequest::new(
            "session-refusal",
            "cmd.exe",
            vec!["/c".to_owned()],
        ))
        .await
        .expect_err("interpreter");
    assert_eq!(error.refusal(), Refusal::ToolNotAllowed);

    let escaping =
        JobRequest::new("session-refusal", "git.exe", vec!["--version".to_owned()]).with_cwd("..");
    let error = registry.start(escaping).await.expect_err("cwd escape");
    assert_eq!(error.refusal(), Refusal::PathOutsideJail);
}
