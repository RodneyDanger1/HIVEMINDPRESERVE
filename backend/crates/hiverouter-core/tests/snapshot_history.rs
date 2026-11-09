//! Phase 9 acceptance: snapshot schema migration round-trip, corrupt-snapshot
//! refusal, history read API (HTTP) and time-travel read model.
//!
//! Blueprint card commands: `migration_roundtrip`, `corrupt_snapshot`,
//! `history_api`.

mod support;

use std::net::SocketAddr;
use std::sync::Arc;

use hiverouter_contract::{AgentState, Refusal, RunId, TaskId};
use hiverouter_core::db::Database;
use hiverouter_core::entities::TaskRecord;
use hiverouter_core::graph::GraphAuthority;
use hiverouter_core::history::HistoryReadModel;
use hiverouter_core::snapshot::{
    capture, migrate_to_current, migrate_v1_to_v2, migrate_v2_to_v1, SnapshotStore,
    SNAPSHOT_SCHEMA_V1, SNAPSHOT_SCHEMA_V2,
};
use hiverouter_core::store::RunStore;
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

fn setup() -> (Arc<Database>, SnapshotStore, tempfile::TempDir) {
    let temp = tempfile::tempdir().expect("temp dir");
    let db = Arc::new(Database::open(&temp.path().join("hiverouter.db")).expect("db"));
    db.migrate().expect("migrate");
    let store = SnapshotStore::new(temp.path(), db.clone());
    (db, store, temp)
}

fn graph_with_two_tasks() -> (GraphAuthority, TaskId, TaskId) {
    let graph = GraphAuthority::new();
    let root = TaskId::new();
    let child = TaskId::new();
    graph.add_task(root, None).expect("root");
    graph.add_task(child, Some(root)).expect("child");
    (graph, root, child)
}

#[test]
fn snapshot_preserves_agent_kind_and_role() {
    let (_db, _store, _temp) = setup();
    let (graph, root, child) = graph_with_two_tasks();
    graph
        .set_metadata(root, "hyperagent", "planner", Some("HyperAgent"))
        .expect("root metadata");
    graph
        .set_metadata(child, "superagent", "super", Some("SuperAgent"))
        .expect("child metadata");
    let envelope = capture(&graph, RunId::new(), 1, AgentState::Running).expect("capture");
    let root_node = envelope
        .graph
        .nodes
        .iter()
        .find(|node| node.task_id == root)
        .expect("root node");
    assert_eq!(root_node.kind.as_deref(), Some("hyperagent"));
    assert_eq!(root_node.role.as_deref(), Some("planner"));
    let child_node = envelope
        .graph
        .nodes
        .iter()
        .find(|node| node.task_id == child)
        .expect("child node");
    assert_eq!(child_node.kind.as_deref(), Some("superagent"));
    assert_eq!(child_node.label.as_deref(), Some("SuperAgent"));
}

#[test]
fn migration_roundtrip() {
    let (_db, _store, _temp) = setup();
    let (graph, root, child) = graph_with_two_tasks();
    let run_id = RunId::new();
    let envelope = capture(&graph, run_id, 3, AgentState::Completed).expect("capture");
    assert_eq!(envelope.schema_version, SNAPSHOT_SCHEMA_V2);
    let v2 = serde_json::to_value(&envelope).expect("serialize v2");
    let graph_before = v2["graph"].clone();

    // v2 -> v1 drops run_status, graph payload is unchanged.
    let v1 = migrate_v2_to_v1(v2.clone()).expect("downgrade");
    assert_eq!(v1["schema_version"], SNAPSHOT_SCHEMA_V1);
    assert!(v1.get("run_status").is_none());
    assert_eq!(v1["graph"], graph_before);

    // v1 -> v2 re-adds the field and re-parses to an equal envelope (data equality).
    let upgraded = migrate_v1_to_v2(v1.clone()).expect("upgrade");
    let restored = migrate_to_current(&upgraded).expect("parse upgraded");
    assert_eq!(restored.graph, envelope.graph);
    assert_eq!(restored.revision, envelope.revision);
    assert_eq!(restored.run_id, run_id);
    let round_tripped_graph = serde_json::to_value(&restored.graph).expect("graph json");
    assert_eq!(round_tripped_graph, graph_before);
    assert_eq!(restored.graph.nodes.len(), 2);
    assert_eq!(restored.graph.edges, vec![(root, child)]);
    assert_eq!(restored.run_status, None, "v1 carried no run status");

    // Unknown schema version is refused by name.
    let mut unknown = v2.clone();
    unknown["schema_version"] = json!("hivemind.snapshot.v9");
    assert_eq!(migrate_to_current(&unknown), Err(Refusal::SnapshotCorrupt));
}

#[test]
fn corrupt_snapshot() {
    let (db, store, temp) = setup();
    let run_store = RunStore::new(db.clone());
    let run = run_store
        .create_run("p", "corrupt me", AgentState::Running)
        .expect("run");
    let (graph, _root, _child) = graph_with_two_tasks();
    let envelope = capture(&graph, run.id, 0, AgentState::Running).expect("capture");
    let path = store.write(&envelope).expect("write snapshot");

    // A corrupt file is refused with E_SNAPSHOT_CORRUPT; the workspace is not reset.
    std::fs::write(&path, "{ this is not valid json").expect("corrupt");
    let error = store.read(run.id, 0).expect_err("corrupt snapshot");
    assert_eq!(error, Refusal::SnapshotCorrupt);
    assert_eq!(error.code(), "E_SNAPSHOT_CORRUPT");
    assert!(path.exists(), "corrupt snapshot is left in place for inspection");
    let _ = temp;

    // The SQLite row survives too.
    let stored = run_store.get_run(run.id).expect("run row");
    assert_eq!(stored.status, AgentState::Running);
}

#[test]
fn history_api_returns_real_revisions() {
    let (db, store, temp) = setup();
    let run_store = RunStore::new(db.clone());
    let run = run_store
        .create_run("p", "history run", AgentState::Running)
        .expect("run");
    let (graph, _root, _child) = graph_with_two_tasks();
    store
        .write(&capture(&graph, run.id, 0, AgentState::Running).expect("r0"))
        .expect("write r0");
    store
        .write(&capture(&graph, run.id, 1, AgentState::Completed).expect("r1"))
        .expect("write r1");
    let _ = temp;

    let history = HistoryReadModel::new(db.clone(), store.clone());
    let view = history.time_travel(run.id).expect("time travel");
    assert_eq!(view.revision_count(), 2, "real revision count, not a fabricated one");
    assert_eq!(view.latest_revision, Some(1));
    assert!(view.has_revision(1));
    assert!(!view.has_revision(7));
    let state = history.state_at(run.id, 1).expect("state at 1");
    assert_eq!(state.run_status, Some(AgentState::Completed));
    assert_eq!(state.graph.nodes.len(), 2);
    assert_eq!(
        history.state_at(run.id, 9).expect_err("gap"),
        Refusal::HistoryGap
    );
}

async fn raw_get(addr: SocketAddr, path: &str) -> (u16, Value) {
    let mut stream = TcpStream::connect(addr).await.expect("connect");
    let request = format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\nAccept: application/json\r\n\r\n");
    stream.write_all(request.as_bytes()).await.expect("write");
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await.expect("read");
    let text = String::from_utf8_lossy(&raw).into_owned();
    let status = text
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .expect("status");
    let body = text.split("\r\n\r\n").nth(1).unwrap_or_default();
    let value: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    (status, value)
}

#[tokio::test]
async fn history_http_api() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = support::TestServer::start(&temp).await;
    let state = server.state.clone();
    let run_store = RunStore::new(state.db.clone());
    let run = run_store
        .create_run("p", "http history", AgentState::Running)
        .expect("run");
    let (graph, root, child) = graph_with_two_tasks();
    // The read projection is task-row authoritative and fail-closed. Seed the
    // two rows so this HTTP test exercises a real run, not an unfiltered graph.
    for (id, parent, role, depth, spawn_seq) in [
        (root, None, "planner", 0, 1),
        (child, Some(root), "worker", 1, 2),
    ] {
        run_store
            .insert_task(&TaskRecord {
                id,
                run_id: run.id,
                parent_task_id: parent,
                role: role.to_owned(),
                depth,
                spawn_seq,
                state: AgentState::Completed,
                rev: 1,
            })
            .expect("seed task");
    }
    let store = SnapshotStore::new(&state.config.data_dir, state.db.clone());
    store
        .write(&capture(&graph, run.id, 0, AgentState::Running).expect("r0"))
        .expect("write");
    store
        .write(&capture(&graph, run.id, 1, AgentState::Completed).expect("r1"))
        .expect("write");
    assert!(!child.as_uuid().is_nil());

    let (status, runs) = raw_get(server.addr, "/api/history/runs").await;
    assert_eq!(status, 200);
    assert_eq!(runs["runs"][0]["run_id"], run.id.to_string());

    let (status, revisions) =
        raw_get(server.addr, &format!("/api/history/runs/{}/revisions", run.id)).await;
    assert_eq!(status, 200);
    assert_eq!(revisions["revision_count"], 2);
    assert_eq!(revisions["revisions"][1]["revision"], 1);
    assert_eq!(revisions["revisions"][1]["run_status"], "completed");

    let (status, state_json) =
        raw_get(server.addr, &format!("/api/history/runs/{}/state?revision=1", run.id)).await;
    assert_eq!(status, 200);
    assert_eq!(state_json["revision"], 1);
    assert_eq!(state_json["run_status"], "completed");
    assert_eq!(state_json["graph"]["nodes"].as_array().expect("nodes").len(), 2);

    let (status, gap) =
        raw_get(server.addr, &format!("/api/history/runs/{}/state?revision=9", run.id)).await;
    assert_eq!(status, 404);
    assert_eq!(gap["code"], "E_HISTORY_GAP");

    let (status, roster) = raw_get(
        server.addr,
        &format!("/api/agents/runs/{}/roster", run.id),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(roster["run_id"], run.id.to_string());
    assert_eq!(roster["count"], 2);
    assert_eq!(roster["agents"][0]["role"], "planner");
    assert_eq!(roster["agents"][0]["kind"], "hyperagent");
    assert_eq!(roster["agents"][1]["role"], "worker");
    assert_eq!(roster["agents"][1]["parent_task_id"], root.to_string());
    assert_eq!(roster["agents"][1]["state"], "completed");

    let (status, unknown) = raw_get(
        server.addr,
        &format!("/api/agents/runs/{}/roster", RunId::new()),
    )
    .await;
    assert_eq!(status, 404);
    assert_eq!(unknown["code"], "E_RUN_NOT_FOUND");
}

#[test]
fn terminal_prune_keeps_the_newest_revisions_and_latest_is_readable() {
    let (_db, store, _temp) = setup();
    let (graph, _root, _child) = graph_with_two_tasks();
    let run_id = RunId::new();
    for revision in 0..6 {
        store
            .write(&capture(&graph, run_id, revision, AgentState::Running).expect("capture"))
            .expect("write");
    }
    assert_eq!(store.list_revisions(run_id).expect("list").len(), 6);
    assert_eq!(store.prune_run(run_id, 2).expect("prune"), 4);
    assert_eq!(
        store.list_revisions(run_id).expect("list after prune"),
        vec![4, 5]
    );
    assert_eq!(store.latest(run_id).expect("latest").expect("latest").revision, 5);
    assert_eq!(
        store.indexed_revisions(run_id).expect("index").len(),
        2,
        "the index row is removed with the file"
    );
    assert_eq!(
        store.read(run_id, 2).expect_err("deleted revision").code(),
        "E_SNAPSHOT_CORRUPT"
    );
}

