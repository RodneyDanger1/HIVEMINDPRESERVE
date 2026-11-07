//! BUG-CHECK RUN 2 reproduction: WP-01 approvals-table integrity.
//!
//! Read-only investigation initially; this file is the executable reproduction
//! for the two migration/CAS suspicions. It uses an in-memory SQLite connection
//! and no network/paid path.

use hiverouter_contract::{RunId, TaskId};
use hiverouter_tools::approval::{
    cancel_pending, count_for_run, decision_label, expire, expire_orphaned_pending, migrate,
    ApprovalGate,
};
use hiverouter_tools::tool::{PermissionClass, ToolId};
use rusqlite::Connection;

const NOW: u64 = 1_700_000_000;

fn memory() -> Connection {
    let conn = Connection::open_in_memory().expect("in-memory db");
    migrate(&conn).expect("migration");
    conn
}

fn args() -> serde_json::Value {
    serde_json::json!({"path": "notes.txt", "content": "hello"})
}

fn other_args() -> serde_json::Value {
    serde_json::json!({"path": "other.txt", "content": "hello"})
}

fn index_exists(conn: &Connection) -> bool {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master
             WHERE type = 'index' AND name = 'approvals_one_pending_per_call'",
            [],
            |row| row.get(0),
        )
        .expect("sqlite_master query");
    count == 1
}

#[test]
fn unique_pending_index_rejects_second_pending_row_for_the_same_call() {
    let conn = memory();
    let gate = ApprovalGate::default();
    let run = RunId::new();
    let task = TaskId::new();
    gate.request(
        &conn,
        run,
        task,
        "call-dup",
        ToolId::FileWrite,
        PermissionClass::Mutate,
        &args(),
        NOW,
    )
    .expect("first pending request");

    let second = gate
        .request(
            &conn,
            run,
            task,
            "call-dup",
            ToolId::FileWrite,
            PermissionClass::Mutate,
            &other_args(),
            NOW,
        )
        .expect_err("the partial unique index must reject a second pending call");
    println!(
        "second pending insert refusal: code={} display={second}",
        second.code()
    );
    assert_eq!(second.code(), "E_APPROVAL_REPLAY");

    assert_eq!(count_for_run(&conn, run).expect("count"), 1);
}

#[test]
fn migration_on_legacy_duplicate_pending_rows_is_safe_and_named() {
    let conn = memory();
    assert!(index_exists(&conn), "fresh DB has the WP-01 index");
    // Simulate a legacy/concurrent dirty DB: remove the new index, then create
    // two pending rows for the same (run_id, tool_call_id).
    conn.execute_batch("DROP INDEX approvals_one_pending_per_call")
        .expect("drop index");
    let gate = ApprovalGate::default();
    let run = RunId::new();
    let task = TaskId::new();
    gate.request(
        &conn,
        run,
        task,
        "call-legacy",
        ToolId::FileWrite,
        PermissionClass::Mutate,
        &args(),
        NOW,
    )
    .expect("first legacy pending row");
    gate.request(
        &conn,
        run,
        task,
        "call-legacy",
        ToolId::FileWrite,
        PermissionClass::Mutate,
        &other_args(),
        NOW,
    )
    .expect("second legacy pending row (pre-index DB)");
    assert!(!index_exists(&conn), "fixture has no unique index");

    // A booting process runs migrate() before the boot orphan reaper. Migration
    // must not silently proceed without making the duplicate state explicit.
    let migration = migrate(&conn);
    match &migration {
        Ok(()) => panic!("duplicate pending rows must not silently create the index"),
        Err(error) => println!(
            "migrate duplicate refusal: code={} display={error}",
            error.code()
        ),
    }
    let error = migration.expect_err("migration must refuse while duplicates exist");
    assert_eq!(
        error.code(),
        "E_APPROVAL_REPLAY",
        "the unique-index failure must use the existing unique-violation approval refusal"
    );
    assert!(
        !index_exists(&conn),
        "failed migration leaves the index absent"
    );

    // Boot reaper then consumes every pending row; the next migrate creates the
    // index without deleting any audit row.
    assert_eq!(
        expire_orphaned_pending(&conn, NOW).expect("boot orphan reaper"),
        2
    );
    migrate(&conn).expect("migration succeeds after orphan expiry");
    assert!(index_exists(&conn), "index is created on the retry");

    // CAS/label/count semantics on the retained audit rows.
    let rows: Vec<String> = conn
        .prepare("SELECT decision FROM approvals WHERE run_id = ?1 ORDER BY tool_call_id")
        .expect("prepare")
        .query_map([run.to_string()], |row| row.get(0))
        .expect("query")
        .collect::<Result<_, _>>()
        .expect("rows");
    assert_eq!(rows, vec!["expired".to_owned(), "expired".to_owned()]);
    assert_eq!(count_for_run(&conn, run).expect("count"), 2);
}

#[test]
fn expire_and_cancel_are_cas_guarded_and_labels_are_readable() {
    let conn = memory();
    let gate = ApprovalGate::default();
    let run = RunId::new();
    let task = TaskId::new();

    let first = gate
        .request(
            &conn,
            run,
            task,
            "call-expire",
            ToolId::FileWrite,
            PermissionClass::Mutate,
            &args(),
            NOW,
        )
        .expect("first request");
    let id = first.approval_id.to_string();
    assert_eq!(decision_label(&conn, &id).expect("label"), None);
    assert!(expire(&conn, &id, NOW).expect("expire CAS wins"));
    assert!(!expire(&conn, &id, NOW).expect("expire CAS loses"));
    assert_eq!(
        decision_label(&conn, &id).expect("label"),
        Some("expired".to_owned())
    );

    let second = gate
        .request(
            &conn,
            run,
            task,
            "call-cancel",
            ToolId::FileWrite,
            PermissionClass::Mutate,
            &args(),
            NOW,
        )
        .expect("second request");
    let second_id = second.approval_id.to_string();
    assert!(cancel_pending(&conn, &second_id, NOW).expect("cancel CAS wins"));
    assert!(!cancel_pending(&conn, &second_id, NOW).expect("cancel CAS loses"));
    assert_eq!(
        decision_label(&conn, &second_id).expect("label"),
        Some("cancelled".to_owned())
    );
    assert_eq!(count_for_run(&conn, run).expect("count"), 2);
}
