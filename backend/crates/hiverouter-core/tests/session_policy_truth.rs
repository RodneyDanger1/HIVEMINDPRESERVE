//! Independent CAP-1 truth audit: one-product session/permission-policy and
//! checkpoint routes, exercised through the real `build_router`/`AppState`
//! harness and the real SQLite migration engine.
//!
//! This file is deliberately independent of `tests/http_boot.rs`: it owns its
//! fixture, its HTTP client, its v24 seed and its checkpoint setup. It does not
//! mock the router, the store, the migration engine or the checkpoint journal.
//!
//! Frozen claim source (read-only): `docs/plans/HIVEMIND-FINISH-PLAN-v3.md`
//! CAP-1/CAP-5 and `docs/plans/pseudocode/CAP-PATCH-IMPACT.md`.
//!
//! The `*_truth_defect_*` tests assert the claimed (correct) contract. They are
//! expected to fail on this HEAD and their panic text is the captured wrong
//! output; nothing in this repo is fixed by this audit.

use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;

use hiverouter_contract::{AgentState, PermissionPolicy, RunId, TaskId, Timestamp};
use hiverouter_core::checkpoint::CheckpointJournal;
use hiverouter_core::config::Config;
use hiverouter_core::db::{Database, MIGRATIONS};
use hiverouter_core::http::{self, AppState};
use hiverouter_core::run_checks;
use hiverouter_core::store::RunStore;
use hiverouter_core::tool_loop::{decide_policy, PolicyDecision};
use hiverouter_tools::PermissionClass;
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

const POLICY_CASES: [&str; 4] = ["classic", "plan", "standard", "trusted"];

// --- fixture ---------------------------------------------------------------

struct TestServer {
    addr: SocketAddr,
    state: Arc<AppState>,
    shutdown: Option<oneshot::Sender<()>>,
    handle: tokio::task::JoinHandle<()>,
}

impl TestServer {
    async fn start(temp: &tempfile::TempDir) -> Self {
        let workspace = temp.path().join("workspace");
        fs::create_dir_all(&workspace).expect("workspace dir");
        let mut config = Config::resolve(
            "127.0.0.1:0",
            temp.path().to_str().expect("utf8 temp path"),
            None,
        )
        .expect("test config");
        // The checkpoint routes jail themselves against this root; it must be
        // the fixture's own directory, never the process working directory.
        config.workspace_root = workspace;
        let database = Database::open(config.db_file()).expect("database");
        database.migrate().expect("migrations");
        let state = Arc::new(AppState::new(
            Arc::new(database),
            run_checks(),
            Arc::new(config),
        ));
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("local addr");
        let (shutdown, shutdown_rx) = oneshot::channel();
        let served_state = state.clone();
        let handle = tokio::spawn(async move {
            let _ = http::serve(listener, served_state, async move {
                let _ = shutdown_rx.await;
            })
            .await;
        });
        Self {
            addr,
            state,
            shutdown: Some(shutdown),
            handle,
        }
    }

    fn store(&self) -> RunStore {
        RunStore::new(self.state.db.clone())
    }

    fn token(&self) -> &str {
        self.state.session.token()
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        // The task is detached like the existing boot harness; the oneshot
        // shutdown lets axum stop gracefully.
        let _ = &self.handle;
    }
}

async fn http_json(
    addr: SocketAddr,
    method: &str,
    path: &str,
    body: Option<&Value>,
    token: Option<&str>,
) -> (u16, Value) {
    let body_text = body.map(ToString::to_string).unwrap_or_default();
    let auth = token
        .map(|token| format!("Authorization: Bearer {token}\r\n"))
        .unwrap_or_default();
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\nAccept: application/json\r\n{auth}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{body_text}",
        body_text.len()
    );
    let mut stream = TcpStream::connect(addr).await.expect("connect");
    stream
        .write_all(request.as_bytes())
        .await
        .expect("write request");
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await.expect("read response");
    let text = String::from_utf8_lossy(&raw).into_owned();
    let status = text
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .expect("http status line");
    let payload = text.split("\r\n\r\n").nth(1).unwrap_or_default();
    let value = if payload.trim().is_empty() {
        Value::Null
    } else {
        serde_json::from_str(payload)
            .unwrap_or_else(|error| panic!("response body is not JSON ({error}): {payload:?}"))
    };
    (status, value)
}

async fn get(server: &TestServer, path: &str) -> (u16, Value) {
    http_json(server.addr, "GET", path, None, None).await
}

async fn post(server: &TestServer, path: &str, body: &Value) -> (u16, Value) {
    http_json(server.addr, "POST", path, Some(body), Some(server.token())).await
}

fn session_body(name: &str, policy: Option<&str>) -> Value {
    let mut body = json!({
        "name": name,
        "project_id": "truth-project",
        "workspace_root": "C:\\truth-workspace",
    });
    if let Some(policy) = policy {
        body["permission_policy"] = Value::String(policy.to_owned());
    }
    body
}

fn memory_store() -> (Arc<Database>, RunStore) {
    let database = Arc::new(Database::open_in_memory().expect("in-memory db"));
    database.migrate().expect("migrations");
    let store = RunStore::new(database.clone());
    (database, store)
}

// --- policy defaults and explicit opt-in -----------------------------------

#[test]
fn permission_policy_default_is_classic_and_unknown_is_not_a_value() {
    assert_eq!(
        PermissionPolicy::default(),
        PermissionPolicy::Classic,
        "the default policy must remain classic"
    );
    for (label, expected) in [
        ("classic", PermissionPolicy::Classic),
        ("plan", PermissionPolicy::Plan),
        ("standard", PermissionPolicy::Standard),
        ("trusted", PermissionPolicy::Trusted),
    ] {
        assert_eq!(
            PermissionPolicy::from_str_strict(label),
            Some(expected),
            "strict parser lost {label}"
        );
        assert_eq!(expected.as_str(), label);
    }
    assert_eq!(
        PermissionPolicy::from_str_strict("yolo"),
        None,
        "an unknown policy must not parse as a real policy"
    );
    // The compatibility parser is documented to fall back to classic so an old
    // row can never become more permissive. That is a store-side safety net,
    // not permission for an explicit API value to be silently accepted.
    assert_eq!(PermissionPolicy::from_str("yolo"), PermissionPolicy::Classic);
}

#[test]
fn create_run_defaults_to_classic_and_no_session() {
    let (database, store) = memory_store();
    let run = store
        .create_run("project", "default policy probe", AgentState::Queued)
        .expect("create run");
    assert_eq!(
        run.permission_policy,
        PermissionPolicy::Classic,
        "create_run must default to classic"
    );
    assert_eq!(run.session_id, None, "create_run must not invent a session");

    let stored = store.get_run(run.id).expect("read back run");
    assert_eq!(stored.permission_policy, PermissionPolicy::Classic);
    assert_eq!(stored.session_id, None);

    let raw: String = database
        .with_connection(|connection| {
            connection.query_row(
                "SELECT permission_policy FROM runs WHERE id = ?1",
                rusqlite::params![run.id.to_string()],
                |row| row.get(0),
            )
        })
        .expect("raw policy");
    assert_eq!(raw, "classic", "the database row must say classic");
}

#[test]
fn explicit_trusted_is_persisted_only_when_asked_for() {
    let (database, store) = memory_store();
    let session = store
        .create_session("project", "trusted session", "C:\\workspace", PermissionPolicy::Trusted)
        .expect("trusted session");
    assert_eq!(session.permission_policy, PermissionPolicy::Trusted);

    let scoped = store
        .create_run_scoped(
            "project",
            "explicit trusted run",
            AgentState::Queued,
            Some(&session.id),
            PermissionPolicy::Trusted,
        )
        .expect("scoped run");
    assert_eq!(scoped.permission_policy, PermissionPolicy::Trusted);
    assert_eq!(scoped.session_id.as_deref(), Some(session.id.as_str()));
    assert_eq!(
        store.get_run(scoped.id).expect("read back").permission_policy,
        PermissionPolicy::Trusted
    );

    // A plain run must stay classic even though a trusted session exists. This
    // is the "no inference from context/prompt" half of the claim.
    let plain = store
        .create_run("project", "no inference", AgentState::Queued)
        .expect("plain run");
    assert_eq!(plain.permission_policy, PermissionPolicy::Classic);

    // Only an explicit write may change a run's policy.
    store
        .set_run_policy(plain.id, PermissionPolicy::Trusted)
        .expect("explicit run policy update");
    assert_eq!(
        store.get_run(plain.id).expect("read back").permission_policy,
        PermissionPolicy::Trusted
    );
    let raw: String = database
        .with_connection(|connection| {
            connection.query_row(
                "SELECT permission_policy FROM runs WHERE id = ?1",
                rusqlite::params![plain.id.to_string()],
                |row| row.get(0),
            )
        })
        .expect("raw policy");
    assert_eq!(raw, "trusted");
    assert_eq!(
        store.set_run_policy(RunId::new(), PermissionPolicy::Trusted),
        Err(hiverouter_contract::Refusal::RunNotFound)
    );
}

#[test]
fn policy_matrix_resolves_each_class_as_the_cap_contract_states() {
    // Read/control never ask under any policy.
    for policy in [
        PermissionPolicy::Classic,
        PermissionPolicy::Plan,
        PermissionPolicy::Standard,
        PermissionPolicy::Trusted,
    ] {
        assert_eq!(decide_policy(policy, PermissionClass::Read, false), PolicyDecision::Execute);
        assert_eq!(decide_policy(policy, PermissionClass::Control, false), PolicyDecision::Execute);
    }
    // Classic is exactly the old approval behavior.
    assert_eq!(decide_policy(PermissionPolicy::Classic, PermissionClass::Mutate, true), PolicyDecision::AskApproval);
    assert_eq!(decide_policy(PermissionPolicy::Classic, PermissionClass::Execute, true), PolicyDecision::AskApproval);
    assert_eq!(decide_policy(PermissionPolicy::Classic, PermissionClass::Network, true), PolicyDecision::AskApproval);
    // Plan refuses every side effect, even inside the workspace.
    assert_eq!(decide_policy(PermissionPolicy::Plan, PermissionClass::Mutate, true), PolicyDecision::Refuse);
    assert_eq!(decide_policy(PermissionPolicy::Plan, PermissionClass::Execute, true), PolicyDecision::Refuse);
    // Standard auto-approves workspace mutation only.
    assert_eq!(decide_policy(PermissionPolicy::Standard, PermissionClass::Mutate, true), PolicyDecision::Execute);
    assert_eq!(decide_policy(PermissionPolicy::Standard, PermissionClass::Mutate, false), PolicyDecision::AskApproval);
    assert_eq!(decide_policy(PermissionPolicy::Standard, PermissionClass::Execute, true), PolicyDecision::AskApproval);
    // Trusted auto-approves workspace mutation/execute only; outside the jail
    // it still asks, and network always asks.
    assert_eq!(decide_policy(PermissionPolicy::Trusted, PermissionClass::Mutate, true), PolicyDecision::Execute);
    assert_eq!(decide_policy(PermissionPolicy::Trusted, PermissionClass::Execute, true), PolicyDecision::Execute);
    assert_eq!(decide_policy(PermissionPolicy::Trusted, PermissionClass::Mutate, false), PolicyDecision::AskApproval);
    assert_eq!(decide_policy(PermissionPolicy::Trusted, PermissionClass::Network, true), PolicyDecision::AskApproval);
}

#[test]
fn trusted_execute_decision_is_wired_to_the_execution_path() {
    // `decide_policy` is a pure resolver; its only caller is in `execution.rs`.
    // A correct wiring must distinguish `Execute` (run without an interactive
    // approval) from `AskApproval` (suspend). This is a structural probe because
    // this test crate cannot drive a provider turn without a live tool loop;
    // the router/AppState surface has no endpoint that executes a tool call.
    let source = include_str!("../src/execution.rs");
    let call_site = source
        .find("tool_loop::decide_policy(")
        .expect("the policy resolver is called from execution.rs");
    let window = &source[call_site..(call_site + 2_400).min(source.len())];
    assert!(
        window.contains("PolicyDecision::Execute"),
        "TRUTH DEFECT CAP1-D3 (execution.rs:2894-2922): the only `decide_policy` caller \
         matches `Refuse` and sends every other decision (including `Execute` for \
         standard/trusted) to `suspend_for_approval`, so a trusted run still asks for \
         every Mutate/Execute tool. Actual call-site window:\n{window}"
    );
}

// --- POST /api/sessions ----------------------------------------------------

#[tokio::test]
async fn session_create_stores_and_reads_back_each_policy() {
    let temp = tempfile::tempdir().expect("tempdir");
    let server = TestServer::start(&temp).await;
    let store = server.store();

    for policy in POLICY_CASES {
        let (status, body) = post(
            &server,
            "/api/sessions",
            &session_body(&format!("session-{policy}"), Some(policy)),
        )
        .await;
        assert_eq!(status, 201, "create {policy}: {body}");
        assert_eq!(
            body["permission_policy"], policy,
            "create response for {policy}: {body}"
        );
        let id = body["id"].as_str().expect("session id").to_owned();

        let (status, detail) = get(&server, &format!("/api/sessions/{id}")).await;
        assert_eq!(status, 200, "detail {policy}: {detail}");
        assert_eq!(detail["session"]["permission_policy"], policy);
        assert_eq!(detail["session"]["id"], id.as_str());

        let (status, list) = get(&server, "/api/sessions").await;
        assert_eq!(status, 200, "list: {list}");
        assert!(
            list["sessions"]
                .as_array()
                .expect("sessions array")
                .iter()
                .any(|session| session["id"] == id.as_str() && session["permission_policy"] == policy),
            "list is missing {policy}: {list}"
        );

        let raw: String = server
            .state
            .db
            .with_connection(|connection| {
                connection.query_row(
                    "SELECT permission_policy FROM sessions WHERE id = ?1",
                    rusqlite::params![id],
                    |row| row.get(0),
                )
            })
            .expect("raw session policy");
        assert_eq!(raw, policy, "database row for {policy}");
    }

    // A missing policy is the documented classic default, not trusted.
    let (status, body) = post(&server, "/api/sessions", &session_body("default-policy", None)).await;
    assert_eq!(status, 201, "default create: {body}");
    assert_eq!(body["permission_policy"], "classic");
    assert_eq!(store.sessions().expect("sessions").len(), 5);
}

#[tokio::test]
async fn session_create_rejects_missing_and_blank_required_fields() {
    let temp = tempfile::tempdir().expect("tempdir");
    let server = TestServer::start(&temp).await;
    let cases = [
        ("empty object", json!({})),
        ("missing name", json!({"project_id": "p", "workspace_root": "C:\\w"})),
        ("missing project", json!({"name": "n", "workspace_root": "C:\\w"})),
        ("missing workspace", json!({"name": "n", "project_id": "p"})),
        ("blank name", json!({"name": "   ", "project_id": "p", "workspace_root": "C:\\w"})),
        ("blank project", json!({"name": "n", "project_id": " ", "workspace_root": "C:\\w"})),
        ("blank workspace", json!({"name": "n", "project_id": "p", "workspace_root": "  "})),
        ("non-string name", json!({"name": 7, "project_id": "p", "workspace_root": "C:\\w"})),
    ];
    for (label, body) in cases {
        let (status, response) = post(&server, "/api/sessions", &body).await;
        assert_eq!(status, 400, "{label} must be a schema refusal: {response}");
        assert_eq!(response["code"], "E_SCHEMA_INVALID", "{label}: {response}");
    }
    assert!(
        server.store().sessions().expect("sessions").is_empty(),
        "no refused request may create a session"
    );
}

#[tokio::test]
async fn session_create_truth_defect_rejects_unknown_explicit_policy() {
    // CAP-1: the policy is one of classic|plan|standard|trusted and `trusted`
    // is explicit opt-in. An explicit unknown value must be a schema refusal;
    // silently turning it into classic fabricates a policy the caller did not
    // ask for and hides a typo.
    let temp = tempfile::tempdir().expect("tempdir");
    let server = TestServer::start(&temp).await;
    let before = server.store().sessions().expect("sessions").len();

    let mut violations = Vec::new();
    let cases = [
        ("unknown string", json!("yolo")),
        ("wrong case", json!("TRUSTED")),
        ("numeric", json!(7)),
        ("null", Value::Null),
    ];
    for (index, (label, value)) in cases.iter().enumerate() {
        let mut body = session_body(&format!("invalid-policy-{index}"), None);
        body["permission_policy"] = value.clone();
        let (status, response) = post(&server, "/api/sessions", &body).await;
        if status != 400 || response["code"] != "E_SCHEMA_INVALID" {
            violations.push(format!(
                "{label}: expected 400 E_SCHEMA_INVALID, got {status} {response}"
            ));
        }
        if response["permission_policy"] == "classic" {
            violations.push(format!(
                "{label}: the wrong policy was materialised as `classic` (a fabricated default)"
            ));
        }
    }
    let after = server.store().sessions().expect("sessions").len();
    if after != before {
        violations.push(format!(
            "{} invalid request(s) still created a session (before={before}, after={after})",
            after - before
        ));
    }
    assert!(
        violations.is_empty(),
        "TRUTH DEFECT CAP1-D1 (http.rs:919-923): POST /api/sessions accepts an explicit \
         unknown policy with `unwrap_or(PermissionPolicy::Classic)` instead of refusing it.\n\
         Captured wrong output:\n{}",
        violations.join("\n")
    );
}

// --- session todos ----------------------------------------------------------

#[tokio::test]
async fn session_todo_routes_add_list_and_refuse_unknown_sessions() {
    let temp = tempfile::tempdir().expect("temp");
    let server = TestServer::start(&temp).await;
    let (status, session) = post(
        &server,
        "/api/sessions",
        &session_body("todo-session", Some("classic")),
    )
    .await;
    assert_eq!(status, 201, "session create: {session:?}");
    let session_id = session["id"].as_str().expect("session id");

    let (status, todo) = post(
        &server,
        &format!("/api/sessions/{session_id}/todos"),
        &json!({ "text": "inspect the failing test" }),
    )
    .await;
    assert_eq!(status, 201, "todo add: {todo:?}");
    assert_eq!(todo["text"], "inspect the failing test");
    assert_eq!(todo["state"], "open");

    let (status, list) = get(&server, &format!("/api/sessions/{session_id}/todos")).await;
    assert_eq!(status, 200);
    assert_eq!(list["todos"].as_array().map(Vec::len), Some(1));
    assert_eq!(list["todos"][0]["text"], "inspect the failing test");

    let (status, refusal) = post(
        &server,
        "/api/sessions/does-not-exist/todos",
        &json!({ "text": "orphan" }),
    )
    .await;
    assert_eq!(status, 400, "unknown session must refuse: {refusal:?}");
    assert_eq!(refusal["code"], "E_SESSION_NOT_FOUND");

    let (status, refusal) = post(
        &server,
        &format!("/api/sessions/{session_id}/todos"),
        &json!({ "text": "   " }),
    )
    .await;
    assert_eq!(status, 400, "blank todo must refuse: {refusal:?}");
    assert_eq!(refusal["code"], "E_SCHEMA_INVALID");
}

// --- POST /api/sessions/{id}/policy ---------------------------------------

#[tokio::test]
async fn session_policy_update_reads_back_and_does_not_retroactively_change_runs() {
    let temp = tempfile::tempdir().expect("tempdir");
    let server = TestServer::start(&temp).await;
    let store = server.store();
    let session = store
        .create_session("truth-project", "policy target", "C:\\workspace", PermissionPolicy::Classic)
        .expect("session");
    let run = store
        .create_run_scoped(
            "truth-project",
            "existing run",
            AgentState::Queued,
            Some(&session.id),
            PermissionPolicy::Trusted,
        )
        .expect("run");

    let (status, body) = post(
        &server,
        &format!("/api/sessions/{}/policy", session.id),
        &json!({"permission_policy": "trusted"}),
    )
    .await;
    assert_eq!(status, 200, "policy update: {body}");
    assert_eq!(body["permission_policy"], "trusted", "{body}");
    assert_eq!(body["id"], session.id.as_str());

    let (status, detail) = get(&server, &format!("/api/sessions/{}", session.id)).await;
    assert_eq!(status, 200, "detail: {detail}");
    assert_eq!(detail["session"]["permission_policy"], "trusted");

    // The session policy is the default for future runs; it must not rewrite
    // an already-created run's explicit policy.
    assert_eq!(
        store.get_run(run.id).expect("run readback").permission_policy,
        PermissionPolicy::Trusted,
        "the existing run's explicit policy must survive a session default change"
    );

    let (status, body) = post(
        &server,
        &format!("/api/sessions/{}/policy", session.id),
        &json!({"permission_policy": "plan"}),
    )
    .await;
    assert_eq!(status, 200, "second update: {body}");
    assert_eq!(body["permission_policy"], "plan");
}

#[tokio::test]
async fn session_policy_update_refuses_bad_input_and_unknown_sessions() {
    let temp = tempfile::tempdir().expect("tempdir");
    let server = TestServer::start(&temp).await;
    let session = server
        .store()
        .create_session("truth-project", "refusal target", "C:\\workspace", PermissionPolicy::Classic)
        .expect("session");
    let path = format!("/api/sessions/{}/policy", session.id);

    for (label, body) in [
        ("unknown policy", json!({"permission_policy": "yolo"})),
        ("wrong case", json!({"permission_policy": "TRUSTED"})),
        ("numeric policy", json!({"permission_policy": 7})),
        ("missing field", json!({})),
    ] {
        let (status, response) = post(&server, &path, &body).await;
        assert_eq!(status, 400, "{label} must be 400: {response}");
        assert_eq!(response["code"], "E_SCHEMA_INVALID", "{label}: {response}");
    }
    assert_eq!(
        server.store().session(&session.id).expect("session").permission_policy,
        PermissionPolicy::Classic,
        "a refused update must not change the stored policy"
    );

    let (status, response) = post(
        &server,
        "/api/sessions/does-not-exist/policy",
        &json!({"permission_policy": "trusted"}),
    )
    .await;
    assert_eq!(status, 404, "unknown session: {response}");
    assert_eq!(response["code"], "E_SESSION_NOT_FOUND", "{response}");
}

// --- v24 -> v27 migration --------------------------------------------------

fn seed_v24(database: &Database) {
    database
        .with_connection(|connection| {
            connection.execute_batch(
                "CREATE TABLE IF NOT EXISTS schema_migrations (
                    version    INTEGER PRIMARY KEY,
                    name       TEXT NOT NULL,
                    applied_at TEXT NOT NULL
                );",
            )?;
            for migration in MIGRATIONS.iter().filter(|migration| migration.version <= 24) {
                connection.execute_batch(migration.up)?;
                connection.execute(
                    "INSERT INTO schema_migrations(version, name, applied_at) VALUES (?1, ?2, ?3)",
                    rusqlite::params![migration.version, migration.name, Timestamp::now().as_str()],
                )?;
            }
            Ok(())
        })
        .expect("seed v24 schema");
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

#[test]
fn v24_database_migrates_to_v28_without_losing_legacy_rows() {
    let database = Arc::new(Database::open_in_memory().expect("in-memory db"));
    seed_v24(&database);
    assert_eq!(database.schema_version().expect("version"), 24);

    let run_id = RunId::new();
    let task_id = TaskId::new();
    let now = "2026-01-02T03:04:05Z";
    database
        .with_connection(|connection| {
            connection.execute(
                "INSERT INTO runs(id, project_id, prompt, status, created_at, updated_at)
                 VALUES (?1, 'legacy-project', 'legacy prompt', 'queued', ?2, ?2)",
                rusqlite::params![run_id.to_string(), now],
            )?;
            connection.execute(
                "INSERT INTO tasks(id, run_id, parent_task_id, role, depth, spawn_seq, state, rev)
                 VALUES (?1, ?2, NULL, 'planner', 0, 0, 'queued', 0)",
                rusqlite::params![task_id.to_string(), run_id.to_string()],
            )?;
            connection.execute(
                "INSERT INTO chat_messages(id, run_id, role, text, created_at)
                 VALUES ('legacy-message', ?1, 'user', 'hello', ?2)",
                rusqlite::params![run_id.to_string(), now],
            )?;
            connection.execute(
                "INSERT INTO command_receipts(correlation_id, run_id, created_at)
                 VALUES ('legacy-correlation', ?1, ?2)",
                rusqlite::params![run_id.to_string(), now],
            )?;
            connection.execute(
                "INSERT INTO snapshots(run_id, revision, path, created_at, schema_version)
                 VALUES (?1, 1, 'snap.json', ?2, 'hiverouter.v1')",
                rusqlite::params![run_id.to_string(), now],
            )?;
            Ok(())
        })
        .expect("seed v24 rows");

    let migrated = database.migrate().expect("migrate to head");
    assert_eq!(migrated, 28, "the CAP head is v28 (25 sessions, 26 repo, 27 checkpoints, 28 jobs)");
    assert_eq!(database.schema_version().expect("version"), 28);
    let applied = database.applied_migrations().expect("applied migrations");
    for (version, name) in [
        (25_i64, "sessions_policy_todos"),
        (26, "repo_index"),
        (27, "checkpoint_journal"),
        (28, "jobs"),
    ] {
        assert!(
            applied.iter().any(|(applied_version, applied_name, _)| {
                *applied_version == version && applied_name == name
            }),
            "migration {version} ({name}) missing from {applied:?}"
        );
    }

    let tables = table_names(&database);
    for expected in [
        "sessions",
        "session_events",
        "todos",
        "repo_files",
        "repo_chunks",
        "checkpoints",
        "checkpoint_blobs",
        "jobs",
    ] {
        assert!(
            tables.iter().any(|name| name == expected),
            "{expected} missing after v24 -> v28: {tables:?}"
        );
    }

    let raw: (String, String, String, Option<String>, String) = database
        .with_connection(|connection| {
            connection.query_row(
                "SELECT project_id, prompt, status, session_id, permission_policy
                 FROM runs WHERE id = ?1",
                rusqlite::params![run_id.to_string()],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
        })
        .expect("legacy run after upgrade");
    assert_eq!(raw.0, "legacy-project");
    assert_eq!(raw.1, "legacy prompt");
    assert_eq!(raw.2, "queued");
    assert_eq!(raw.3, None, "a v24 run has no session");
    assert_eq!(raw.4, "classic", "a v24 run must adopt classic");

    let store = RunStore::new(database.clone());
    let legacy = store.get_run(run_id).expect("legacy run through the store");
    assert_eq!(legacy.project_id, "legacy-project");
    assert_eq!(legacy.permission_policy, PermissionPolicy::Classic);
    assert_eq!(legacy.session_id, None);
    for (table, expected) in [
        ("tasks", 1_i64),
        ("chat_messages", 1),
        ("command_receipts", 1),
        ("snapshots", 1),
    ] {
        let count: i64 = database
            .with_connection(|connection| {
                connection.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| row.get(0))
            })
            .expect("count");
        assert_eq!(count, expected, "{table} rows were not preserved by v24 -> v27");
    }

    // An explicitly opted-in legacy run still gets its policy, while a plain
    // new run defaults to classic.
    store
        .set_run_policy(run_id, PermissionPolicy::Trusted)
        .expect("explicit policy on legacy run");
    assert_eq!(
        store.get_run(run_id).expect("readback").permission_policy,
        PermissionPolicy::Trusted
    );

    // Reversibility: 28 -> 24 -> 28 must retain the legacy rows and restore
    // the classic default on the forward pass.
    assert_eq!(database.migrate_down_to(24).expect("down to v24"), 24);
    let legacy_rows: i64 = database
        .with_connection(|connection| {
            connection.query_row(
                "SELECT COUNT(*) FROM runs WHERE id = ?1 AND project_id = 'legacy-project'",
                rusqlite::params![run_id.to_string()],
                |row| row.get(0),
            )
        })
        .expect("legacy row after down");
    assert_eq!(legacy_rows, 1, "the down migration must not delete legacy runs");
    assert_eq!(database.migrate().expect("re-up to head"), 28);
    let after_roundtrip = store.get_run(run_id).expect("legacy run after roundtrip");
    assert_eq!(after_roundtrip.project_id, "legacy-project");
    assert_eq!(
        after_roundtrip.permission_policy,
        PermissionPolicy::Classic,
        "the down/up roundtrip drops the v25 policy column, so re-up restores classic"
    );
}

// --- checkpoint routes -----------------------------------------------------

fn write_file(root: &std::path::Path, relative: &str, bytes: &[u8]) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent dir");
    }
    fs::write(path, bytes).expect("write fixture file");
}

#[tokio::test]
async fn checkpoint_routes_return_real_list_detail_and_restore_data() {
    let temp = tempfile::tempdir().expect("tempdir");
    let server = TestServer::start(&temp).await;
    let store = server.store();
    let run = store
        .create_run("truth-project", "checkpoint probe", AgentState::Running)
        .expect("run");
    let journal = CheckpointJournal::open(
        server.state.db.clone(),
        server.state.config.workspace_root.clone(),
    )
    .expect("journal");
    let root = journal.jail().root().to_path_buf();
    write_file(&root, "notes/a.txt", b"alpha");
    journal
        .journal_before(run.id, 1, "notes/a.txt")
        .expect("journal before");
    write_file(&root, "notes/a.txt", b"beta");
    journal.record_after(run.id, 1).expect("record after");

    let (status, body) = get(&server, &format!("/api/runs/{}/checkpoints", run.id)).await;
    assert_eq!(status, 200, "list: {body}");
    assert_eq!(body["run_id"], run.id.to_string());
    let entries = body["checkpoints"].as_array().expect("checkpoints array");
    assert_eq!(entries.len(), 1, "list must return the real journal row: {body}");
    assert_eq!(entries[0]["seq"], 1);
    assert_eq!(entries[0]["path"], "notes/a.txt");
    assert_eq!(entries[0]["before"]["kind"], "present");
    assert_eq!(
        entries[0]["before"]["sha256"],
        hiverouter_tools::sha256_hex(b"alpha")
    );
    assert_eq!(entries[0]["after"]["kind"], "present");
    assert_eq!(
        entries[0]["after"]["sha256"],
        hiverouter_tools::sha256_hex(b"beta")
    );

    let (status, detail) = get(&server, &format!("/api/runs/{}/checkpoints/1", run.id)).await;
    assert_eq!(status, 200, "detail: {detail}");
    assert_eq!(detail["path"], "notes/a.txt");
    assert_eq!(detail["seq"], 1);

    let (status, restored) = post(
        &server,
        &format!("/api/runs/{}/checkpoints/1/restore", run.id),
        &json!({}),
    )
    .await;
    assert_eq!(status, 200, "restore: {restored}");
    assert_eq!(restored["path"], "notes/a.txt");
    assert_eq!(restored["undone_seqs"], json!([1]));
    assert_eq!(restored["restored"]["kind"], "present");
    assert_eq!(
        fs::read(root.join("notes/a.txt")).expect("restored file"),
        b"alpha",
        "the route must restore the real pre-image, not claim success"
    );
    assert!(
        journal.get(run.id, 1).expect("get").expect("row").restored_at.is_some(),
        "a successful restore must be recorded in the journal"
    );

    // Repeating the restore must be a named refusal: the file no longer
    // matches the recorded after-state (external drift).
    let (status, response) = post(
        &server,
        &format!("/api/runs/{}/checkpoints/1/restore", run.id),
        &json!({}),
    )
    .await;
    assert_eq!(status, 409, "drift restore: {response}");
    assert_eq!(response["code"], "E_INTERNAL_ERROR", "{response}");
    assert_eq!(
        fs::read(root.join("notes/a.txt")).expect("file after refused restore"),
        b"alpha",
        "a refused restore must leave the file untouched"
    );

    // A real run with no journal entries is an empty list, not an error.
    let empty_run = store
        .create_run("truth-project", "empty checkpoint run", AgentState::Queued)
        .expect("empty run");
    let (status, empty) = get(&server, &format!("/api/runs/{}/checkpoints", empty_run.id)).await;
    assert_eq!(status, 200, "empty list: {empty}");
    assert_eq!(empty["checkpoints"], json!([]));
}

#[tokio::test]
async fn checkpoint_detail_and_restore_refuse_absent_entries() {
    let temp = tempfile::tempdir().expect("tempdir");
    let server = TestServer::start(&temp).await;
    let run = server
        .store()
        .create_run("truth-project", "absent entry run", AgentState::Running)
        .expect("run");

    let (status, response) = get(&server, &format!("/api/runs/{}/checkpoints/99", run.id)).await;
    assert_eq!(status, 404, "absent detail: {response}");
    assert_eq!(response["code"], "E_RUN_NOT_FOUND", "{response}");

    let (status, response) = post(
        &server,
        &format!("/api/runs/{}/checkpoints/99/restore", run.id),
        &json!({}),
    )
    .await;
    assert_eq!(status, 409, "absent restore: {response}");
    assert_eq!(response["code"], "E_INTERNAL_ERROR", "{response}");

    let (status, response) = post(
        &server,
        &format!("/api/runs/{}/checkpoints/revert", run.id),
        &json!({"sequence": 1}),
    )
    .await;
    assert_eq!(status, 200, "no-op revert: {response}");
    assert_eq!(response["restored"], json!([]));
}

#[tokio::test]
async fn checkpoint_truth_defect_refuses_absent_run() {
    // A route parameterised by run id must distinguish "this run exists and has
    // no checkpoints" from "this run does not exist". Returning 200 with an
    // empty list for a nonexistent run is fabricated success: the caller cannot
    // tell it from a real empty journal.
    let temp = tempfile::tempdir().expect("tempdir");
    let server = TestServer::start(&temp).await;
    let absent = RunId::new();
    let mut violations = Vec::new();

    let (status, body) = get(&server, &format!("/api/runs/{absent}/checkpoints")).await;
    if status != 404 || body["code"] != "E_RUN_NOT_FOUND" {
        violations.push(format!(
            "GET list for absent run: expected 404 E_RUN_NOT_FOUND, got {status} {body}"
        ));
    }

    let (status, body) = post(
        &server,
        &format!("/api/runs/{absent}/checkpoints/revert"),
        &json!({"sequence": 1}),
    )
    .await;
    if status != 404 || body["code"] != "E_RUN_NOT_FOUND" {
        violations.push(format!(
            "POST revert for absent run: expected 404 E_RUN_NOT_FOUND, got {status} {body}"
        ));
    }

    let (status, body) = post(
        &server,
        &format!("/api/runs/{absent}/checkpoints/1/restore"),
        &json!({}),
    )
    .await;
    if status != 404 || body["code"] != "E_RUN_NOT_FOUND" {
        violations.push(format!(
            "POST restore for absent run: expected 404 E_RUN_NOT_FOUND, got {status} {body}"
        ));
    }

    assert!(
        violations.is_empty(),
        "TRUTH DEFECT CAP1-D2 (http.rs:990-1064): checkpoint routes let an absent run \
         masquerade as a real empty journal instead of refusing it.\n\
         Captured wrong output:\n{}",
        violations.join("\n")
    );
}

// --- keep the warning-free build honest --------------------------------

#[allow(dead_code)]
fn _assert_server_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<TestServer>();
}
