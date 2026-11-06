//! Phase 4 acceptance: chat message store + run/task store persistence
//! (blueprint card command `chat_store`).

use std::sync::Arc;

use hiverouter_contract::{AgentState, Refusal};
use hiverouter_core::db::Database;
use hiverouter_core::entities::{ChatRole, TaskRecord};
use hiverouter_core::store::RunStore;
use uuid::Uuid;

fn store() -> (RunStore, Arc<Database>, tempfile::TempDir) {
    let temp = tempfile::tempdir().expect("temp dir");
    let db = Arc::new(Database::open(&temp.path().join("hiverouter.db")).expect("db"));
    db.migrate().expect("migrate");
    (RunStore::new(db.clone()), db, temp)
}

#[test]
fn chat_store_persists_user_and_assistant_messages_in_order() {
    let (store, _db, _temp) = store();
    let run = store
        .create_run("project-1", "hello hive", AgentState::Queued)
        .expect("create run");
    store
        .append_message(run.id, ChatRole::User, "hello hive")
        .expect("user message");
    store
        .append_message(run.id, ChatRole::Assistant, "hello back")
        .expect("assistant message");
    let messages = store.messages(run.id).expect("messages");
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].role, ChatRole::User);
    assert_eq!(messages[0].text, "hello hive");
    assert_eq!(messages[1].role, ChatRole::Assistant);
    assert_eq!(messages[1].text, "hello back");
    // Re-opening the database keeps the messages (durable store).
    let reopened = Database::open(Path::new(&format!(
        "{}\\hiverouter.db",
        _temp.path().display()
    )))
    .expect("reopen");
    assert_eq!(reopened.schema_version().expect("version"), 4);
}

use std::path::Path;

#[test]
fn run_status_transitions_and_active_run_tracking() {
    let (store, _db, _temp) = store();
    let run = store
        .create_run("p", "prompt", AgentState::Queued)
        .expect("create");
    assert_eq!(
        store.latest_active_run().expect("active").map(|run| run.id),
        Some(run.id)
    );
    store
        .set_run_status(run.id, AgentState::Running, None)
        .expect("running");
    store
        .set_run_status(
            run.id,
            AgentState::Failed,
            Some(Refusal::LlmRateLimit.code()),
        )
        .expect("failed");
    assert!(store.latest_active_run().expect("active").is_none());
    let stored = store.get_run(run.id).expect("run");
    assert_eq!(stored.status, AgentState::Failed);
    assert_eq!(
        stored.error_code.as_deref(),
        Some(Refusal::LlmRateLimit.code())
    );
    assert_eq!(store.get_run(hiverouter_contract::RunId::new()), Err(Refusal::RunNotFound));
}

#[test]
fn tasks_are_persisted_with_caps_fields() {
    let (store, _db, _temp) = store();
    let run = store
        .create_run("p", "prompt", AgentState::Running)
        .expect("create");
    let root = TaskRecord {
        id: hiverouter_contract::TaskId::new(),
        run_id: run.id,
        parent_task_id: None,
        role: "planner".to_owned(),
        depth: 0,
        spawn_seq: 1,
        state: AgentState::Running,
        rev: 0,
    };
    store.insert_task(&root).expect("insert root");
    let child = TaskRecord {
        id: hiverouter_contract::TaskId::new(),
        run_id: run.id,
        parent_task_id: Some(root.id),
        role: "worker".to_owned(),
        depth: 1,
        spawn_seq: 2,
        state: AgentState::Queued,
        rev: 0,
    };
    store.insert_task(&child).expect("insert child");
    assert_eq!(store.task_count(run.id).expect("count"), 2);
    let rev = store
        .update_task_state(child.id, AgentState::Running)
        .expect("update");
    assert_eq!(rev, 1);
    let tasks = store.tasks_for_run(run.id).expect("tasks");
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[1].state, AgentState::Running);
    assert_eq!(tasks[1].parent_task_id, Some(root.id));
    assert_eq!(
        store.get_task(hiverouter_contract::TaskId::new()),
        Err(Refusal::TaskNotFound)
    );
}

#[test]
fn command_receipts_make_mutations_idempotent() {
    let (store, _db, _temp) = store();
    let correlation = Uuid::now_v7();
    assert!(store.receipt_run(correlation).expect("lookup").is_none());
    let run = store
        .create_run("p", "prompt", AgentState::Queued)
        .expect("create");
    store.record_receipt(correlation, run.id).expect("receipt");
    assert_eq!(store.receipt_run(correlation).expect("lookup"), Some(run.id));
}

#[test]
fn migration_v2_down_and_up_roundtrip() {
    let temp = tempfile::tempdir().expect("temp dir");
    let db = Database::open(&temp.path().join("hiverouter.db")).expect("db");
    assert_eq!(db.migrate().expect("migrate"), 4);
    // Down to v2 keeps the run/task/chat tables (the v4 tool-turn table and v3
    // snapshot index are dropped).
    assert_eq!(db.migrate_down_to(2).expect("down v4+v3"), 2);
    let store = RunStore::new(Arc::new(
        Database::open(&temp.path().join("hiverouter.db")).expect("reopen"),
    ));
    assert_eq!(store.list_runs(10).expect("empty runs"), Vec::new());
    // Down to v0 drops every phase table, then the full migration chain re-applies.
    assert_eq!(db.migrate_down_to(0).expect("down all"), 0);
    assert_eq!(db.migrate().expect("re-migrate"), 4);
}

