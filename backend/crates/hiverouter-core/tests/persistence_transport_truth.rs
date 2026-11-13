//! Independent persistence/transport truth audit for the CAP-1 capability work.
//!
//! Frozen claim sources (read-only): `docs/plans/HIVEMIND-FINISH-PLAN-v3.md`
//! §0/§3/§6, `docs/plans/HIVEMIND-CAPABILITY-EXPANSION-PLAN.md` §5 and
//! `docs/plans/pseudocode/CAP-PATCH-IMPACT.md` CAP-1/CAP-5.
//!
//! The tests drive the real migration engine, the real `RunStore`, the real
//! `RunEngine` and the real `/ws` handler (through `tests/support`). They do not
//! mock the database, the store or the transport. Provider calls are avoided by
//! construction: the only WS command sent is refused during contract decoding,
//! and the engine test uses a test double whose `key_available()` is the trait
//! default (`None`, "cannot answer") and whose `plan` never reaches a network.
//!
//! The `*_truth_defect_*` cases assert the correct contract; their panic text is
//! the observed wrong behaviour on the pre-fix tree.

mod support;

use std::sync::Arc;

use async_trait::async_trait;
use hiverouter_contract::{
    schema, AgentState, Command, CommandName, Envelope, PermissionPolicy, Refusal, RunId, Timestamp,
};
use hiverouter_core::bus::EventBus;
use hiverouter_core::db::{Database, MIGRATIONS};
use hiverouter_core::entities::TaskRecord;
use hiverouter_core::execution::{DeltaSink, ProviderExecutor, RunEngine, StreamedReply};
use hiverouter_core::graph::GraphAuthority;
use hiverouter_core::history::HistoryReadModel;
use hiverouter_core::planner::Plan;
use hiverouter_core::snapshot::SnapshotStore;
use hiverouter_core::prompts::TemplateRegistry;
use hiverouter_core::repo_index;
use hiverouter_core::store::RunStore;
use serde_json::json;
use support::{error_code, frame_name, recv_until, TestServer, WsClient, TEST_ORIGIN};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

// --- fixtures ---------------------------------------------------------------

fn memory_store() -> (Arc<Database>, RunStore) {
    let database = Arc::new(Database::open_in_memory().expect("in-memory db"));
    database.migrate().expect("migrations");
    let store = RunStore::new(database.clone());
    (database, store)
}

fn table_names(database: &Database) -> Vec<String> {
    database
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
        .expect("table names")
}

fn column_names(database: &Database, table: &str) -> Vec<String> {
    database
        .with_connection(|connection| {
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

/// Applies only the migrations up to `target`, producing a genuine `target`-era
/// database without relying on the down migrations.
fn seed_schema_to(database: &Database, target: i64) {
    database
        .with_connection(|connection| {
            connection.execute_batch(
                "CREATE TABLE IF NOT EXISTS schema_migrations (
                    version    INTEGER PRIMARY KEY,
                    name       TEXT NOT NULL,
                    applied_at TEXT NOT NULL
                );",
            )?;
            for migration in MIGRATIONS.iter().filter(|migration| migration.version <= target) {
                connection.execute_batch(migration.up)?;
                connection.execute(
                    "INSERT INTO schema_migrations(version, name, applied_at) VALUES (?1, ?2, ?3)",
                    rusqlite::params![migration.version, migration.name, Timestamp::now().as_str()],
                )?;
            }
            Ok(())
        })
        .expect("seed schema");
}

/// Installs the probe-gated v26 FTS5 objects exactly as the app does.
fn repo_ensure(database: &Database) {
    database
        .with_connection(|connection| {
            repo_index::ensure_schema(connection)
                .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
        })
        .expect("repo index schema");
}

fn seed_repo_chunk(database: &Database, content: &str) {
    database
        .with_connection(|connection| {
            connection.execute(
                "INSERT INTO repo_files(id, workspace_id, rel_path, size_bytes, line_count, content_hash, indexed_at)
                 VALUES (7, 'workspace', 'src/lib.rs', 10, 1, 'hash', 0)",
                [],
            )?;
            connection.execute(
                "INSERT INTO repo_chunks(id, file_id, chunk_index, start_line, end_line, content)
                 VALUES (7, 7, 0, 1, 1, ?1)",
                rusqlite::params![content],
            )?;
            Ok(())
        })
        .expect("seed repo chunk");
}

fn fts_hits(database: &Database, needle: &str) -> i64 {
    database
        .with_connection(|connection| {
            connection.query_row(
                "SELECT COUNT(*) FROM repo_chunks_fts WHERE repo_chunks_fts MATCH ?1",
                rusqlite::params![needle],
                |row| row.get(0),
            )
        })
        .expect("fts query")
}

// --- 1. every run SELECT must carry the v25 columns -------------------------

#[test]
fn every_run_read_path_selects_the_v25_columns() {
    let (database, store) = memory_store();
    let session = store
        .create_session(
            "project-read",
            "read-path session",
            "C:\\ws",
            PermissionPolicy::Classic,
        )
        .expect("session");
    let run = store
        .create_run_scoped(
            "project-read",
            "read the run",
            AgentState::Running,
            Some(&session.id),
            PermissionPolicy::Trusted,
        )
        .expect("scoped run");

    let read = store.get_run(run.id).expect("get_run");
    assert_eq!(
        read.session_id.as_deref(),
        Some(session.id.as_str()),
        "get_run dropped runs.session_id (a 7-column SELECT would fail with InvalidColumnIndex)"
    );
    assert_eq!(
        read.permission_policy,
        PermissionPolicy::Trusted,
        "get_run dropped runs.permission_policy"
    );

    let active = store
        .latest_active_run()
        .expect("latest_active_run")
        .expect("the running run must be active");
    assert_eq!(active.id, run.id);
    assert_eq!(
        active.session_id.as_deref(),
        Some(session.id.as_str()),
        "latest_active_run dropped runs.session_id"
    );
    assert_eq!(active.permission_policy, PermissionPolicy::Trusted);

    let listed = store.list_runs(10).expect("list_runs");
    let listed = listed
        .iter()
        .find(|candidate| candidate.id == run.id)
        .expect("the run must be listed");
    assert_eq!(
        listed.session_id.as_deref(),
        Some(session.id.as_str()),
        "list_runs dropped runs.session_id"
    );
    assert_eq!(listed.permission_policy, PermissionPolicy::Trusted);

    let raw: (Option<String>, String) = database
        .with_connection(|connection| {
            connection.query_row(
                "SELECT session_id, permission_policy FROM runs WHERE id = ?1",
                rusqlite::params![run.id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
        })
        .expect("raw run row");
    assert_eq!(raw.0.as_deref(), Some(session.id.as_str()));
    assert_eq!(raw.1, "trusted");
}

// --- 2. a real v24 fixture must stay readable after migrating --------------

#[test]
fn v24_fixture_upgrades_and_legacy_runs_stay_classic_without_a_session() {
    let database = Arc::new(Database::open_in_memory().expect("in-memory db"));
    seed_schema_to(&database, 24);
    assert_eq!(database.schema_version().expect("version"), 24);

    let run_id = RunId::new();
    let now = "2026-01-02T03:04:05Z";
    database
        .with_connection(|connection| {
            connection.execute(
                "INSERT INTO runs(id, project_id, prompt, status, error_code, created_at, updated_at)
                 VALUES (?1, 'legacy', 'legacy prompt', 'running', NULL, ?2, ?2)",
                rusqlite::params![run_id.to_string(), now],
            )?;
            Ok(())
        })
        .expect("legacy v24 run");

    assert_eq!(
        database.migrate().expect("migrate to head"),
        MIGRATIONS.last().expect("migrations").version
    );
    let store = RunStore::new(database.clone());

    let run = store.get_run(run_id).expect("get_run after upgrade");
    assert_eq!(run.project_id, "legacy");
    assert_eq!(run.prompt, "legacy prompt");
    assert_eq!(run.created_at.as_str(), now);
    assert_eq!(run.session_id, None, "a v24 run has no session");
    assert_eq!(
        run.permission_policy,
        PermissionPolicy::Classic,
        "a v24 run must adopt the classic default"
    );

    let active = store
        .latest_active_run()
        .expect("latest_active_run after upgrade")
        .expect("the legacy running run must still be active");
    assert_eq!(active.id, run_id);
    assert_eq!(active.session_id, None);
    assert_eq!(active.permission_policy, PermissionPolicy::Classic);

    let listed = store.list_runs(10).expect("list_runs after upgrade");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, run_id);
    assert_eq!(listed[0].session_id, None);
}

// --- 3. migration order, down migrations and the FTS boundary --------------

#[test]
fn migration_order_is_compiled_order_and_down_restores_the_v24_shape() {
    let database = Database::open_in_memory().expect("in-memory db");
    assert_eq!(
        database.migrate().expect("migrate"),
        MIGRATIONS.last().expect("migrations").version
    );

    let applied = database.applied_migrations().expect("applied migrations");
    let versions: Vec<i64> = applied.iter().map(|(version, _, _)| *version).collect();
    let expected: Vec<i64> = MIGRATIONS.iter().map(|migration| migration.version).collect();
    assert_eq!(
        versions, expected,
        "schema_migrations must record the compiled v1..v27 order"
    );
    for name in ["sessions_policy_todos", "repo_index", "checkpoint_journal"] {
        assert!(
            applied.iter().any(|(_, applied_name, _)| applied_name == name),
            "{name} missing from {applied:?}"
        );
    }

    // v26 creates the base tables; the FTS5 objects come from the probe-gated
    // `ensure_schema` (the documented dependency boundary).
    repo_ensure(&database);
    seed_repo_chunk(&database, "needle");
    assert_eq!(
        fts_hits(&database, "needle"),
        1,
        "the FTS index must see the seeded chunk"
    );

    assert_eq!(database.migrate_down_to(24).expect("down to v24"), 24);
    assert_eq!(database.schema_version().expect("version"), 24);
    let tables = table_names(&database);
    for gone in [
        "sessions",
        "session_events",
        "todos",
        "repo_files",
        "repo_chunks",
        "checkpoints",
        "checkpoint_blobs",
    ] {
        assert!(
            !tables.iter().any(|name| name == gone),
            "{gone} survived the v25..v27 down migrations: {tables:?}"
        );
    }
    assert!(
        !tables
            .iter()
            .any(|name| name.starts_with("repo_chunks_fts")),
        "the v26 down migration must drop the FTS table and its shadow tables: {tables:?}"
    );
    let triggers: i64 = database
        .with_connection(|connection| {
            connection.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'trigger' AND name LIKE 'repo_chunks_%'",
                [],
                |row| row.get(0),
            )
        })
        .expect("trigger count");
    assert_eq!(
        triggers, 0,
        "the v26 down migration must drop the FTS sync triggers"
    );
    let columns = column_names(&database, "runs");
    for gone in ["session_id", "permission_policy"] {
        assert!(
            !columns.iter().any(|name| name == gone),
            "runs.{gone} survived the v25 down migration: {columns:?}"
        );
    }

    // Re-up must rebuild every table; the FTS objects come back through the probe
    // and remain usable (no half-dropped shadow state).
    assert_eq!(
        database.migrate().expect("re-migrate"),
        MIGRATIONS.last().expect("migrations").version
    );
    repo_ensure(&database);
    seed_repo_chunk(&database, "needle again");
    assert_eq!(
        fts_hits(&database, "needle"),
        1,
        "the rebuilt FTS index must be usable after a down/up roundtrip"
    );
}

// --- 4/5. corrupt timestamps must refuse, not fabricate 1970 ----------------

#[test]
fn session_list_truth_defect_refuses_a_corrupt_timestamp_instead_of_1970() {
    let (database, store) = memory_store();
    let session = store
        .create_session("project-corrupt", "corrupt", "C:\\ws", PermissionPolicy::Classic)
        .expect("session");
    database
        .with_connection(|connection| {
            connection.execute(
                "UPDATE sessions SET created_at = 'not-a-timestamp' WHERE id = ?1",
                rusqlite::params![session.id],
            )?;
            Ok(())
        })
        .expect("corrupt the timestamp");

    // The single-row reader already refuses the corrupt row; the list reader must
    // agree instead of inventing 1970-01-01.
    let single = store
        .session(&session.id)
        .expect_err("session() must refuse a corrupt created_at");
    match store.sessions() {
        Ok(sessions) => panic!(
            "TRUTH DEFECT: sessions() silently accepted a corrupt created_at and fabricated \
             {:?}; session() refuses the same row with {single:?}",
            sessions
                .iter()
                .map(|session| session.created_at.as_str().to_owned())
                .collect::<Vec<_>>()
        ),
        Err(refusal) => assert_eq!(
            refusal, single,
            "the list and single-row session readers must agree on the refusal"
        ),
    }

    // Valid rows still read normally (the fix must not refuse healthy data).
    let healthy = store
        .create_session("project-corrupt", "healthy", "C:\\ws", PermissionPolicy::Plan)
        .expect("healthy session");
    database
        .with_connection(|connection| {
            connection.execute(
                "UPDATE sessions SET created_at = ?2 WHERE id = ?1",
                rusqlite::params![session.id, Timestamp::now().as_str()],
            )?;
            Ok(())
        })
        .expect("repair timestamp");
    let sessions = store.sessions().expect("healthy sessions");
    assert!(sessions.iter().any(|candidate| candidate.id == healthy.id));
}

#[test]
fn run_reader_truth_defect_refuses_a_corrupt_timestamp_instead_of_1970() {
    let (database, store) = memory_store();
    let run = store
        .create_run("project-run-corrupt", "corrupt run", AgentState::Queued)
        .expect("run");
    database
        .with_connection(|connection| {
            connection.execute(
                "UPDATE runs SET created_at = 'not-a-timestamp' WHERE id = ?1",
                rusqlite::params![run.id.to_string()],
            )?;
            Ok(())
        })
        .expect("corrupt run timestamp");

    assert!(
        store.get_run(run.id).is_err(),
        "get_run must refuse a corrupt created_at instead of fabricating 1970"
    );
    assert!(
        store.list_runs(10).is_err(),
        "list_runs must refuse a corrupt created_at instead of fabricating 1970"
    );
    assert!(
        full_history(&database).runs(10).is_err(),
        "history must not surface a fabricated 1970 timestamp"
    );
}

fn full_history(database: &Arc<Database>) -> HistoryReadModel {
    HistoryReadModel::new(
        database.clone(),
        SnapshotStore::new(std::env::temp_dir().as_path(), database.clone()),
    )
}

#[test]
fn todo_list_truth_defect_refuses_a_corrupt_timestamp_instead_of_1970() {
    let (database, store) = memory_store();
    let session = store
        .create_session("project-todo", "todo", "C:\\ws", PermissionPolicy::Classic)
        .expect("session");
    let todo = store.add_todo(&session.id, "corrupt me").expect("todo");
    database
        .with_connection(|connection| {
            connection.execute(
                "UPDATE todos SET created_at = 'garbage' WHERE id = ?1",
                rusqlite::params![todo.id],
            )?;
            Ok(())
        })
        .expect("corrupt the timestamp");

    match store.todos(&session.id) {
        Ok(todos) => panic!(
            "TRUTH DEFECT: todos() silently accepted a corrupt created_at and fabricated {:?}",
            todos
                .iter()
                .map(|todo| todo.created_at.as_str().to_owned())
                .collect::<Vec<_>>()
        ),
        Err(refusal) => assert_eq!(
            refusal,
            Refusal::InternalError,
            "a corrupt stored timestamp is an integrity failure, not a healthy row"
        ),
    }
}

// --- 6. unknown sessions are refused by name, never E_INTERNAL_ERROR -------

#[test]
fn missing_session_writes_are_refused_by_name_without_orphan_rows() {
    let (database, store) = memory_store();
    let missing = Uuid::now_v7().to_string();

    assert_eq!(
        store.add_todo(&missing, "orphan"),
        Err(Refusal::SessionNotFound),
        "add_todo must refuse an unknown session with E_SESSION_NOT_FOUND, not the retryable \
         E_INTERNAL_ERROR a mapped foreign-key violation produces"
    );
    assert_eq!(
        store.append_session_event(&missing, "created", &json!({"probe": true})),
        Err(Refusal::SessionNotFound),
        "append_session_event must refuse an unknown session with E_SESSION_NOT_FOUND"
    );

    let (todos, events): (i64, i64) = database
        .with_connection(|connection| {
            Ok((
                connection.query_row("SELECT COUNT(*) FROM todos", [], |row| row.get(0))?,
                connection.query_row("SELECT COUNT(*) FROM session_events", [], |row| row.get(0))?,
            ))
        })
        .expect("counts");
    assert_eq!(
        (todos, events),
        (0, 0),
        "a refused write must not leave an orphan todo or audit row"
    );

    // The named refusals already used by the policy/session methods stay as-is.
    assert_eq!(store.session(&missing), Err(Refusal::SessionNotFound));
    assert_eq!(
        store.set_session_policy(&missing, PermissionPolicy::Trusted),
        Err(Refusal::SessionNotFound)
    );
}

// --- 7. a run may not claim a session that does not exist ------------------

#[test]
fn run_creation_truth_defect_refuses_a_dangling_session_binding() {
    let (database, store) = memory_store();
    let missing = Uuid::now_v7().to_string();

    let refused = store.create_run_scoped(
        "project-dangling",
        "bind a ghost session",
        AgentState::Queued,
        Some(&missing),
        PermissionPolicy::Classic,
    );
    let runs: i64 = database
        .with_connection(|connection| {
            connection.query_row("SELECT COUNT(*) FROM runs", [], |row| row.get(0))
        })
        .expect("run count");
    assert_eq!(
        runs, 0,
        "TRUTH DEFECT: a run was persisted with session_id={missing}, which exists in no \
         sessions row (runs.session_id has no REFERENCES in v25): {refused:?}"
    );
    assert_eq!(
        refused,
        Err(Refusal::SessionNotFound),
        "a dangling session binding must be refused by name"
    );

    // A real session binding is still accepted and stored.
    let session = store
        .create_session("project-ok", "real", "C:\\ws", PermissionPolicy::Trusted)
        .expect("session");
    let run = store
        .create_run_scoped(
            "project-ok",
            "bind a real session",
            AgentState::Queued,
            Some(&session.id),
            PermissionPolicy::Classic,
        )
        .expect("real session binding");
    assert_eq!(run.session_id.as_deref(), Some(session.id.as_str()));
}

// --- 8/9. WS SubmitPrompt validation and engine persistence ----------------
//
// `State` with a key can make a real provider call, so the WS test never sends a
// runnable prompt: the explicit-invalid-policy frame is refused during contract
// decoding, before the key check and before `create_run`. Valid session/policy
// decoding is asserted with `schema::decode_command`, which is the exact gate
// `handle_client_text` calls, and its persistence is asserted against the real
// engine with a test double that cannot reach the network.

#[tokio::test]
async fn ws_submit_prompt_refuses_an_explicit_unknown_policy() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let store = RunStore::new(server.state.db.clone());

    let mut client = WsClient::connect(server.addr, TEST_ORIGIN, None)
        .await
        .expect("websocket connect");
    client.hello(None).await.expect("send hello");
    let _ = recv_until(&mut client, |value| frame_name(value) == "SERVER_HELLO").await;

    let envelope = Envelope::command(
        CommandName::SubmitPrompt,
        &json!({
            "text": "no silent classic fallback",
            "project_id": "project-ws",
            "permission_policy": "yolo",
        }),
    )
    .expect("command envelope");
    let correlation = envelope.correlation_id;
    client
        .send_text(&envelope.to_json().expect("serialize envelope"))
        .await
        .expect("send command");

    let error = recv_until(&mut client, |value| frame_name(value) == "ERROR").await;
    assert_eq!(
        error_code(&error),
        Refusal::SchemaInvalid.code(),
        "an explicit unknown permission_policy must be refused by the schema gate: {error}"
    );
    assert_eq!(
        error["correlation_id"],
        json!(correlation.to_string()),
        "the refusal must answer the refused command: {error}"
    );
    assert_eq!(
        store.list_runs(10).expect("runs").len(),
        0,
        "a refused SubmitPrompt must not create a run row"
    );
}

#[tokio::test]
async fn archived_session_cannot_admit_a_new_run() {
    let database = Arc::new(Database::open_in_memory().expect("in-memory db"));
    database.migrate().expect("migrations");
    let store = RunStore::new(database.clone());
    let session = store
        .create_session(
            "project-archive",
            "archive gate",
            "C:\\ws",
            PermissionPolicy::Classic,
        )
        .expect("session");
    store
        .set_session_archived(&session.id, true)
        .expect("archive session");
    let engine = Arc::new(RunEngine::new(
        database.clone(),
        Arc::new(EventBus::new()),
        Arc::new(GraphAuthority::new()),
        Arc::new(KeylessExecutor) as Arc<dyn ProviderExecutor>,
        Arc::new(TemplateRegistry::default()),
    ));
    let error = engine
        .submit_prompt_scoped(
            "must not run",
            "project-archive",
            Some(&session.id),
            None,
            Uuid::now_v7(),
        )
        .expect_err("archived session must refuse admission");
    assert_eq!(error, Refusal::DecisionRequired);
    assert!(
        store
            .list_runs(10)
            .expect("runs")
            .iter()
            .all(|run| run.session_id.as_deref() != Some(session.id.as_str())),
        "a refused archived-session submit must not create a run row"
    );
}

#[tokio::test]
async fn valid_session_and_policy_decode_and_the_engine_persists_them() {
    // Build the exact envelope the UI would send; `handle_client_text` passes it
    // through `schema::decode_command`, so decoding here is the transport gate.
    let session_id = Uuid::now_v7().to_string();
    let envelope = Envelope::command(
        CommandName::SubmitPrompt,
        &json!({
            "text": "scoped prompt",
            "project_id": "project-ws",
            "session_id": session_id,
            "permission_policy": "trusted",
        }),
    )
    .expect("command envelope");
    match schema::decode_command(&envelope).expect("valid additive fields must decode") {
        Command::SubmitPrompt(payload) => {
            assert_eq!(payload.session_id.as_deref(), Some(session_id.as_str()));
            assert_eq!(payload.permission_policy, Some(PermissionPolicy::Trusted));
        }
        other => panic!("decoded the wrong command: {other:?}"),
    }

    // The engine path: a real RunEngine + RunStore persist the binding and the
    // explicit policy (default `classic` is covered by `create_run` elsewhere).
    let database = Arc::new(Database::open_in_memory().expect("in-memory db"));
    database.migrate().expect("migrations");
    let store = RunStore::new(database.clone());
    let session = store
        .create_session(
            "project-engine",
            "engine session",
            "C:\\ws",
            PermissionPolicy::Classic,
        )
        .expect("session");
    let engine = Arc::new(RunEngine::new(
        database.clone(),
        Arc::new(EventBus::new()),
        Arc::new(GraphAuthority::new()),
        Arc::new(KeylessExecutor) as Arc<dyn ProviderExecutor>,
        Arc::new(TemplateRegistry::default()),
    ));

    let run = engine
        .submit_prompt_scoped(
            "scoped prompt",
            "project-engine",
            Some(&session.id),
            Some(PermissionPolicy::Trusted),
            Uuid::now_v7(),
        )
        .expect("scoped submit");
    assert_eq!(
        run.session_id.as_deref(),
        Some(session.id.as_str()),
        "the engine dropped the submitted session id"
    );
    assert_eq!(
        run.permission_policy,
        PermissionPolicy::Trusted,
        "the engine dropped the submitted explicit policy"
    );

    let stored = store.get_run(run.id).expect("stored run");
    assert_eq!(stored.session_id.as_deref(), Some(session.id.as_str()));
    assert_eq!(stored.permission_policy, PermissionPolicy::Trusted);
    let raw: String = database
        .with_connection(|connection| {
            connection.query_row(
                "SELECT permission_policy FROM runs WHERE id = ?1",
                rusqlite::params![run.id.to_string()],
                |row| row.get(0),
            )
        })
        .expect("raw policy");
    assert_eq!(raw, "trusted");

    let events: i64 = database
        .with_connection(|connection| {
            connection.query_row(
                "SELECT COUNT(*) FROM session_events WHERE session_id = ?1 AND kind = 'run_started'",
                rusqlite::params![session.id],
                |row| row.get(0),
            )
        })
        .expect("session event count");
    assert_eq!(
        events, 1,
        "a session-bound run must persist its run_started audit row"
    );
}

/// A test double that cannot reach the network. The trait's default
/// `key_available()` returns `None` ("cannot answer"), so admission proceeds and
/// the run row is created; `plan` fails immediately inside the spawned dispatch.
struct KeylessExecutor;

#[async_trait]
impl ProviderExecutor for KeylessExecutor {
    async fn plan(
        &self,
        _run_id: RunId,
        _prompt: &str,
        _cancel: &CancellationToken,
    ) -> Result<Plan, Refusal> {
        Err(Refusal::KeyMissing)
    }

    async fn execute(
        &self,
        _run_id: RunId,
        _task: &TaskRecord,
        _prompt: &str,
        _cancel: &CancellationToken,
        _on_delta: &DeltaSink,
    ) -> Result<StreamedReply, Refusal> {
        Err(Refusal::KeyMissing)
    }
}
