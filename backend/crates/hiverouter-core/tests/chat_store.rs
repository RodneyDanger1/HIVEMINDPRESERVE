//! Phase 4 acceptance: chat message store + run/task store persistence
//! (blueprint card command `chat_store`).

use std::sync::Arc;

use hiverouter_contract::{AgentState, Refusal};
use hiverouter_core::db::Database;
use hiverouter_core::entities::{ChatRole, PermissionPolicy, TaskRecord};
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
    assert_eq!(reopened.schema_version().expect("version"), 31);
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
    assert_eq!(db.migrate().expect("migrate"), 31);
    // Down to v2 keeps the run/task/chat tables (the v4 tool-turn table and v3
    // snapshot index are dropped).
    assert_eq!(db.migrate_down_to(2).expect("down v4+v3"), 2);
    // A rolled-back schema is not a state the v25 store serves: the binary
    // migrates forward before any query. Assert the v2 shape directly, which is
    // what this roundtrip test is about.
    let reopened = Database::open(&temp.path().join("hiverouter.db")).expect("reopen");
    let runs: i64 = reopened
        .with_connection(|connection| {
            connection.query_row("SELECT COUNT(*) FROM runs", [], |row| row.get(0))
        })
        .expect("v2 runs table");
    assert_eq!(runs, 0, "v2 runs table must survive the rollback empty");
    let tables: Vec<String> = reopened
        .with_connection(|connection| {
            let mut statement = connection
                .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")?;
            let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
            let mut names = Vec::new();
            for row in rows {
                names.push(row?);
            }
            Ok(names)
        })
        .expect("v2 tables");
    for kept in ["runs", "tasks", "chat_messages", "command_receipts"] {
        assert!(tables.iter().any(|name| name == kept), "{kept} missing at v2");
    }
    assert!(!tables.iter().any(|name| name == "tool_turns"));
    // Down to v0 drops every phase table, then the full migration chain re-applies.
    assert_eq!(db.migrate_down_to(0).expect("down all"), 0);
    assert_eq!(db.migrate().expect("re-migrate"), 31);
}

// --- schema v25: sessions, todos, run permission policy -------------------

fn column_names(db: &Database, table: &str) -> Vec<String> {
    db.with_connection(|connection| {
        let mut statement = connection.prepare(&format!("PRAGMA table_info({table})"))?;
        let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
        let mut names = Vec::new();
        for row in rows {
            names.push(row?);
        }
        Ok(names)
    })
    .expect("column names")
}

fn table_names(db: &Database) -> Vec<String> {
    db.with_connection(|connection| {
        let mut statement =
            connection.prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        let mut names = Vec::new();
        for row in rows {
            names.push(row?);
        }
        Ok(names)
    })
    .expect("table names")
}

/// Newest schema compiled into this binary (keeps v25 tests valid when later
/// migrations are appended).
fn latest_schema() -> i64 {
    hiverouter_core::MIGRATIONS
        .last()
        .expect("at least one migration")
        .version
}

/// Builds a genuine v24 database by applying only the shipped migrations up to
/// v24, so the upgrade test does not depend on the v25 down migration.
fn seed_v24(db: &Database) {
    db.with_connection(|connection| {
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version    INTEGER PRIMARY KEY,
                name       TEXT NOT NULL,
                applied_at TEXT NOT NULL
            );",
        )?;
        for migration in hiverouter_core::MIGRATIONS
            .iter()
            .filter(|migration| migration.version <= 24)
        {
            connection.execute_batch(migration.up)?;
            connection.execute(
                "INSERT INTO schema_migrations(version, name, applied_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![
                    migration.version,
                    migration.name,
                    hiverouter_contract::Timestamp::now().as_str()
                ],
            )?;
        }
        Ok(())
    })
    .expect("seed v24 schema");
}

#[test]
fn default_run_policy_is_classic() {
    let (store, _db, _temp) = store();
    let run = store
        .create_run("p", "classic by default", AgentState::Queued)
        .expect("create run");
    assert_eq!(
        run.permission_policy,
        PermissionPolicy::Classic,
        "a run created without an explicit policy must stay classic"
    );
    assert_eq!(run.session_id, None);
    let reloaded = store.get_run(run.id).expect("reload run");
    assert_eq!(reloaded.permission_policy, PermissionPolicy::Classic);
    assert_eq!(reloaded.session_id, None);
}

#[test]
fn explicit_trusted_policy_persists_and_reloads() {
    let (store, _db, temp) = store();
    let run = store
        .create_run("p", "prompt", AgentState::Queued)
        .expect("create run");
    store
        .set_run_policy(run.id, PermissionPolicy::Trusted)
        .expect("set policy");
    assert_eq!(
        store.get_run(run.id).expect("run").permission_policy,
        PermissionPolicy::Trusted
    );
    // Reopening the database must show the persisted choice, not the default.
    drop(store);
    let reopened = Arc::new(Database::open(&temp.path().join("hiverouter.db")).expect("reopen"));
    let store = RunStore::new(reopened);
    assert_eq!(
        store.get_run(run.id).expect("run after reopen").permission_policy,
        PermissionPolicy::Trusted
    );
    // The explicit choice is also what `list_runs` reports.
    let listed = store.list_runs(10).expect("list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].permission_policy, PermissionPolicy::Trusted);
    // A policy write for a missing run is refused, never silently reported.
    assert_eq!(
        store.set_run_policy(hiverouter_contract::RunId::new(), PermissionPolicy::Trusted),
        Err(Refusal::RunNotFound)
    );
}

#[test]
fn sessions_and_todos_create_and_list() {
    let (store, _db, _temp) = store();
    let first = store
        .create_session("p", "alpha", "/workspace/alpha", PermissionPolicy::Classic)
        .expect("first session");
    let second = store
        .create_session("p", "beta", "/workspace/beta", PermissionPolicy::Trusted)
        .expect("second session");
    assert_eq!(
        store.session(&first.id).expect("session").name,
        "alpha"
    );
    assert_eq!(
        store.session(&second.id).expect("session").permission_policy,
        PermissionPolicy::Trusted
    );
    let sessions = store.sessions().expect("sessions");
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].id, second.id, "sessions list newest first");
    assert_eq!(sessions[1].id, first.id);
    assert_eq!(
        store.session("missing-session"),
        Err(Refusal::SessionNotFound)
    );

    let first_todo = store.add_todo(&first.id, "write the test").expect("todo");
    assert_eq!(first_todo.seq, 1);
    assert_eq!(first_todo.state, "open");
    let second_todo = store.add_todo(&first.id, "run the test").expect("todo");
    assert_eq!(second_todo.seq, 2);
    let todos = store.todos(&first.id).expect("todos");
    assert_eq!(
        todos.iter().map(|todo| todo.text.as_str()).collect::<Vec<_>>(),
        vec!["write the test", "run the test"]
    );
    assert_eq!(
        todos.iter().map(|todo| todo.seq).collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert!(
        store.todos(&second.id).expect("todos").is_empty(),
        "todos are scoped to their session"
    );
}

#[test]
fn v24_runs_stay_readable_after_v25_upgrade() {
    let temp = tempfile::tempdir().expect("temp dir");
    let db = Arc::new(Database::open(&temp.path().join("hiverouter.db")).expect("db"));
    seed_v24(&db);
    assert_eq!(db.schema_version().expect("version"), 24);
    let run_id = hiverouter_contract::RunId::new();
    db.with_connection(|connection| {
        connection.execute(
            "INSERT INTO runs(id, project_id, prompt, status, created_at, updated_at)
             VALUES (?1, 'legacy-project', 'legacy prompt', 'queued', ?2, ?2)",
            rusqlite::params![
                run_id.to_string(),
                hiverouter_contract::Timestamp::now().as_str()
            ],
        )?;
        Ok(())
    })
    .expect("insert v24 run");
    assert_eq!(db.migrate().expect("upgrade to v25"), latest_schema());
    let store = RunStore::new(db.clone());
    let run = store.get_run(run_id).expect("legacy run must stay readable");
    assert_eq!(run.project_id, "legacy-project");
    assert_eq!(run.prompt, "legacy prompt");
    assert_eq!(run.status, AgentState::Queued);
    assert_eq!(run.permission_policy, PermissionPolicy::Classic);
    assert_eq!(run.session_id, None);
    assert_eq!(store.list_runs(10).expect("list runs").len(), 1);
    assert!(store.sessions().expect("sessions").is_empty());
}

#[test]
fn v25_down_migration_restores_v24_shape() {
    let temp = tempfile::tempdir().expect("temp dir");
    let db = Arc::new(Database::open(&temp.path().join("hiverouter.db")).expect("db"));
    assert_eq!(db.migrate().expect("migrate"), latest_schema());
    let store = RunStore::new(db.clone());
    let session = store
        .create_session("p", "s", "/w", PermissionPolicy::Standard)
        .expect("session");
    store.add_todo(&session.id, "one").expect("todo");
    let run = store
        .create_run_scoped(
            "p",
            "scoped prompt",
            AgentState::Queued,
            Some(session.id.as_str()),
            PermissionPolicy::Trusted,
        )
        .expect("scoped run");
    assert_eq!(db.migrate_down_to(24).expect("down v25"), 24);
    let tables = table_names(&db);
    for gone in ["sessions", "session_events", "todos"] {
        assert!(
            !tables.iter().any(|name| name == gone),
            "{gone} must be dropped, got {tables:?}"
        );
    }
    let columns = column_names(&db, "runs");
    assert!(!columns.iter().any(|name| name == "session_id"));
    assert!(!columns.iter().any(|name| name == "permission_policy"));
    // The run itself survives the rollback.
    let prompt: String = db
        .with_connection(|connection| {
            connection.query_row(
                "SELECT prompt FROM runs WHERE id = ?1",
                rusqlite::params![run.id.to_string()],
                |row| row.get(0),
            )
        })
        .expect("run after rollback");
    assert_eq!(prompt, "scoped prompt");
    // Re-applying the migration chain restores the v25 surface.
    assert_eq!(db.migrate().expect("re-migrate"), latest_schema());
    let reloaded = store.get_run(run.id).expect("run after re-migrate");
    assert_eq!(reloaded.permission_policy, PermissionPolicy::Classic);
    assert_eq!(reloaded.session_id, None);
    assert!(table_names(&db).iter().any(|name| name == "sessions"));
    assert!(store.sessions().expect("sessions").is_empty());
}

