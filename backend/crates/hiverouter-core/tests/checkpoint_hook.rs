//! CAP-5 acceptance tests: the checkpoint hook in the approved-side-effect path.
//!
//! These tests drive the real `ToolsBridge` execution path (preflight -> grant
//! CAS -> commit) with a real SQLite database and a real workspace, so the
//! load-bearing ordering is proven end to end:
//!
//! 1. the before-image is captured after preflight and **before** the grant CAS,
//!    so a capture failure never consumes the approval;
//! 2. a refused grant voids the capture, so a denied/failed call leaves no
//!    restorable checkpoint;
//! 3. a successful `file_write` / `apply_patch` records its after-image and
//!    `restore_entry` returns the exact original bytes;
//! 4. a byte-identical outcome (a rejected patch) does not fabricate a
//!    checkpoint for a mutation that never happened;
//! 5. a bridge without a journal is byte-for-byte the pre-CAP-5 path.

use std::fs;
use std::path::Path;
use std::sync::Arc;

use hiverouter_contract::command::ApprovalResolve;
use hiverouter_contract::{ApprovalId, Refusal, RunId, TaskId, Timestamp};
use hiverouter_core::checkpoint::{CheckpointJournal, FileState};
use hiverouter_core::db::Database;
use hiverouter_core::ToolsBridge;
use hiverouter_tools::executor::ExecutedTool;
use hiverouter_tools::sha256_hex;
use hiverouter_tools::shell::ShellPolicy;
use hiverouter_tools::tool::{AgentTier, ToolId, ToolRegistry};

// --- fixtures --------------------------------------------------------------

/// A real workspace + migrated database + journal + bridge.
///
/// `wire_journal` controls whether the bridge actually receives the journal;
/// the journal is always returned so tests can read the (absent) rows.
fn seeded(
    max_file_bytes: Option<u64>,
    wire_journal: bool,
) -> (
    tempfile::TempDir,
    Arc<Database>,
    CheckpointJournal,
    ToolsBridge,
) {
    let temp = tempfile::tempdir().expect("temp dir");
    let workspace = temp.path().join("workspace");
    fs::create_dir_all(&workspace).expect("workspace dir");
    let data_dir = temp.path().join("data");
    let db = Arc::new(Database::open(&temp.path().join("hive.db")).expect("database"));
    db.migrate()
        .expect("schema migrations must include the checkpoint journal");
    ToolsBridge::migrate(&db).expect("approvals migration");
    let mut journal = CheckpointJournal::open(db.clone(), &workspace).expect("journal");
    if let Some(cap) = max_file_bytes {
        journal = journal.with_max_file_bytes(cap);
    }
    let mut bridge = ToolsBridge::new_with_data_dir(&workspace, &data_dir, db.clone());
    if wire_journal {
        bridge = bridge.with_checkpoint_journal(journal.clone());
    }
    (temp, db, journal, bridge)
}

fn insert_run(db: &Database) -> RunId {
    let run_id = RunId::new();
    db.with_connection(|connection| {
        connection.execute(
            "INSERT INTO runs(id, project_id, prompt, status, created_at, updated_at)
             VALUES (?1, 'project', 'checkpoint hook test', 'running', ?2, ?2)",
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

/// Commits one approval request and returns its id, without deciding it.
fn request(
    bridge: &ToolsBridge,
    run: RunId,
    tool_call_id: &str,
    tool: &str,
    args: &serde_json::Value,
) -> (ApprovalId, String) {
    let event = bridge
        .request_model_tool_approval(run, TaskId::new(), tool_call_id, tool, args)
        .expect("approval request");
    (event.approval_id, event.nonce.expect("request nonce"))
}

fn decide(
    bridge: &ToolsBridge,
    data_dir: &Path,
    approval_id: ApprovalId,
    nonce: &str,
    decision: &str,
) {
    let payload = ApprovalResolve {
        approval_id,
        nonce: nonce.to_owned(),
        decision: decision.to_owned(),
        reason: None,
    };
    bridge
        .resolve_approval_ledgered(&payload, &ToolsBridge::approval_ledger_dir(data_dir))
        .expect("approval decision");
}

fn approve(
    bridge: &ToolsBridge,
    data_dir: &Path,
    run: RunId,
    tool_call_id: &str,
    tool: &str,
    args: &serde_json::Value,
) -> ApprovalId {
    let (approval_id, nonce) = request(bridge, run, tool_call_id, tool, args);
    decide(bridge, data_dir, approval_id, &nonce, "approve");
    approval_id
}

/// Executes the bound grant through the real bridge path.
async fn execute_bound(
    bridge: &ToolsBridge,
    run: RunId,
    approval: ApprovalId,
    tool_call_id: &str,
    tool: &str,
    args: &serde_json::Value,
) -> Result<ExecutedTool, Refusal> {
    let approval_id = approval.to_string();
    let policy = ShellPolicy::new(Vec::new());
    let registry = ToolRegistry::standard();
    bridge
        .execute_approved_bound(
            &policy,
            &registry,
            AgentTier::WorkerAgent,
            run,
            &approval_id,
            tool_call_id,
            tool,
            args,
            None,
        )
        .await
}

// --- tests -----------------------------------------------------------------

#[tokio::test]
async fn file_write_captures_the_exact_before_image_and_restores_it() {
    let (temp, db, journal, bridge) = seeded(None, true);
    let data_dir = temp.path().join("data");
    let run = insert_run(&db);
    let root = journal.jail().root().to_path_buf();
    write_file(&root, "notes/one.txt", b"before-bytes");

    let args = serde_json::json!({"path": "notes/one.txt", "content": "after-bytes"});
    let approval = approve(&bridge, &data_dir, run, "call-write", "file_write", &args);
    let executed = execute_bound(&bridge, run, approval, "call-write", "file_write", &args)
        .await
        .expect("approved write executes");
    assert!(matches!(executed, ExecutedTool::Ok { .. }));
    assert_eq!(
        fs::read(root.join("notes/one.txt")).expect("written"),
        b"after-bytes"
    );

    let entries = journal.entries_for_run(run).expect("entries");
    assert_eq!(entries.len(), 1, "exactly one mutation was journalled");
    let entry = &entries[0];
    assert_eq!(entry.seq, 1);
    assert_eq!(entry.path, "notes/one.txt");
    assert_eq!(
        entry.before,
        FileState::Present {
            sha256: sha256_hex(b"before-bytes"),
            size: 12,
        },
        "the before-image must be the exact pre-mutation bytes"
    );
    assert_eq!(
        entry.after,
        Some(FileState::Present {
            sha256: sha256_hex(b"after-bytes"),
            size: 11,
        }),
        "a successful write marks the entry committed with the real after-image"
    );
    assert!(
        entry.completed_at.is_some(),
        "a committed entry carries its completion timestamp"
    );

    let restored = journal.restore_entry(run, 1).expect("restore");
    assert_eq!(restored.restored, entry.before);
    assert_eq!(
        fs::read(root.join("notes/one.txt")).expect("restored"),
        b"before-bytes",
        "restore returns the original bytes"
    );
}

#[tokio::test]
async fn denied_and_failed_grants_leave_no_restorable_checkpoint() {
    let (temp, db, journal, bridge) = seeded(None, true);
    let data_dir = temp.path().join("data");
    let root = journal.jail().root().to_path_buf();

    // Denied grant: the operator said no, so no effect and no checkpoint.
    let denied_run = insert_run(&db);
    write_file(&root, "denied.txt", b"denied-original");
    let denied_args = serde_json::json!({"path": "denied.txt", "content": "denied-write"});
    let (denied_id, denied_nonce) = request(
        &bridge,
        denied_run,
        "call-denied",
        "file_write",
        &denied_args,
    );
    decide(&bridge, &data_dir, denied_id, &denied_nonce, "deny");
    let error = execute_bound(
        &bridge,
        denied_run,
        denied_id,
        "call-denied",
        "file_write",
        &denied_args,
    )
    .await
    .expect_err("a denied grant cannot execute");
    assert_eq!(error, Refusal::ApprovalRequired);
    assert_eq!(
        fs::read(root.join("denied.txt")).expect("unchanged"),
        b"denied-original"
    );
    assert!(
        journal
            .entries_for_run(denied_run)
            .expect("entries")
            .is_empty(),
        "a denied grant must void the captured before-image"
    );

    // Approved grant, wrong provider tool-call binding: never executes.
    let bound_run = insert_run(&db);
    write_file(&root, "bound.txt", b"bound-original");
    let bound_args = serde_json::json!({"path": "bound.txt", "content": "bound-write"});
    let bound_id = approve(
        &bridge,
        &data_dir,
        bound_run,
        "call-bound",
        "file_write",
        &bound_args,
    );
    let error = execute_bound(
        &bridge,
        bound_run,
        bound_id,
        "call-other",
        "file_write",
        &bound_args,
    )
    .await
    .expect_err("a different tool-call binding cannot consume the grant");
    assert_eq!(error, Refusal::ApprovalRequired);
    assert_eq!(
        fs::read(root.join("bound.txt")).expect("unchanged"),
        b"bound-original"
    );
    assert!(
        journal
            .entries_for_run(bound_run)
            .expect("entries")
            .is_empty(),
        "a failed grant binding must void the captured before-image"
    );

    // The refused attempt burned nothing: the exact bound call still executes
    // and is journalled exactly once.
    execute_bound(
        &bridge,
        bound_run,
        bound_id,
        "call-bound",
        "file_write",
        &bound_args,
    )
    .await
    .expect("the exact bound call still executes");
    assert_eq!(
        fs::read(root.join("bound.txt")).expect("written"),
        b"bound-write"
    );
    assert_eq!(
        journal.entries_for_run(bound_run).expect("entries").len(),
        1
    );
}

#[tokio::test]
async fn capture_failure_refuses_before_the_grant_is_consumed() {
    // The existing workspace file is larger than the journal bound, so the
    // before-image cannot be captured; the write itself would be legal.
    let (temp, db, journal, bridge) = seeded(Some(4), true);
    let data_dir = temp.path().join("data");
    let run = insert_run(&db);
    let root = journal.jail().root().to_path_buf();
    write_file(&root, "large.txt", b"01234567");

    let args = serde_json::json!({"path": "large.txt", "content": "small"});
    let approval = approve(&bridge, &data_dir, run, "call-cap", "file_write", &args);
    let error = execute_bound(&bridge, run, approval, "call-cap", "file_write", &args)
        .await
        .expect_err("capture must refuse before the grant");
    assert_eq!(error, Refusal::FileTooLarge);
    assert_eq!(
        fs::read(root.join("large.txt")).expect("unchanged"),
        b"01234567",
        "nothing was written"
    );
    assert!(
        journal.entries_for_run(run).expect("entries").is_empty(),
        "a failed capture leaves no journal row"
    );

    // The grant was never verified, so it is still intact and consumable.
    let now = Timestamp::now().unix_secs().unwrap_or(0);
    bridge
        .verify_tool_grant_bound(
            &approval.to_string(),
            "call-cap",
            run,
            ToolId::FileWrite,
            &args,
            now,
        )
        .expect("a capture failure must not consume the approval grant");
}

#[tokio::test]
async fn apply_patch_is_journalled_and_restores_the_original_bytes() {
    let (temp, db, journal, bridge) = seeded(None, true);
    let data_dir = temp.path().join("data");
    let run = insert_run(&db);
    let root = journal.jail().root().to_path_buf();
    write_file(&root, "patch.txt", b"old\n");

    let args = serde_json::json!({
        "path": "patch.txt",
        "patch": "--- a/patch.txt\n+++ b/patch.txt\n@@ -1,1 +1,1 @@\n-old\n+new\n",
    });
    let approval = approve(&bridge, &data_dir, run, "call-patch", "apply_patch", &args);
    let executed = execute_bound(&bridge, run, approval, "call-patch", "apply_patch", &args)
        .await
        .expect("approved patch executes");
    let ExecutedTool::Ok { text } = executed else {
        panic!("an applied patch must execute");
    };
    assert!(text.contains("Applied"), "{text}");
    assert_eq!(fs::read(root.join("patch.txt")).expect("patched"), b"new\n");

    let entries = journal.entries_for_run(run).expect("entries");
    assert_eq!(entries.len(), 1);
    let entry = &entries[0];
    assert_eq!(entry.path, "patch.txt");
    assert_eq!(
        entry.before,
        FileState::Present {
            sha256: sha256_hex(b"old\n"),
            size: 4,
        }
    );
    assert_eq!(
        entry.after,
        Some(FileState::Present {
            sha256: sha256_hex(b"new\n"),
            size: 4,
        })
    );

    journal.restore_entry(run, 1).expect("restore patch");
    assert_eq!(
        fs::read(root.join("patch.txt")).expect("restored"),
        b"old\n"
    );
}

#[tokio::test]
async fn rejected_patch_does_not_fabricate_a_checkpoint() {
    // A context mismatch is a successful tool call (`PatchOutcome::Rejected`)
    // that writes nothing; the journal must not claim a mutation happened.
    let (temp, db, journal, bridge) = seeded(None, true);
    let data_dir = temp.path().join("data");
    let run = insert_run(&db);
    let root = journal.jail().root().to_path_buf();
    write_file(&root, "reject.txt", b"actual\n");

    let args = serde_json::json!({
        "path": "reject.txt",
        "patch": "--- a/reject.txt\n+++ b/reject.txt\n@@ -1,1 +1,1 @@\n-missing\n+x\n",
    });
    let approval = approve(&bridge, &data_dir, run, "call-reject", "apply_patch", &args);
    let executed = execute_bound(&bridge, run, approval, "call-reject", "apply_patch", &args)
        .await
        .expect("a rejected patch is still a completed tool call");
    let ExecutedTool::Ok { text } = executed else {
        panic!("a rejected patch returns Ok with the rejected outcome");
    };
    assert!(text.contains("Rejected"), "{text}");
    assert_eq!(
        fs::read(root.join("reject.txt")).expect("byte-identical"),
        b"actual\n",
        "the rejected patch must not write"
    );
    assert!(
        journal.entries_for_run(run).expect("entries").is_empty(),
        "no checkpoint may be recorded for a mutation that did not happen"
    );
}

#[tokio::test]
async fn a_bridge_without_a_journal_never_writes_a_checkpoint() {
    let (temp, db, journal, bridge) = seeded(None, false);
    let data_dir = temp.path().join("data");
    let run = insert_run(&db);
    let root = journal.jail().root().to_path_buf();
    write_file(&root, "plain.txt", b"original\n");

    let args = serde_json::json!({"path": "plain.txt", "content": "changed\n"});
    let approval = approve(&bridge, &data_dir, run, "call-plain", "file_write", &args);
    let executed = execute_bound(&bridge, run, approval, "call-plain", "file_write", &args)
        .await
        .expect("approved write executes without a journal");
    assert!(matches!(executed, ExecutedTool::Ok { .. }));
    assert_eq!(
        fs::read(root.join("plain.txt")).expect("written"),
        b"changed\n"
    );
    assert!(
        journal.entries_for_run(run).expect("entries").is_empty(),
        "the journal must stay untouched when it is not wired into the bridge"
    );
    assert!(
        bridge.checkpoint_journal().is_none(),
        "the bridge reports no installed journal"
    );
}
