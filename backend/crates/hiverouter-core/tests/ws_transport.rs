//! WebSocket transport acceptance tests (Phase 2).
//!
//! Covers: Origin allowlist refusal, per-session token refusal, two-client
//! fan-out, hello replay/resync, invalid-frame error path, rewire ACK/NODE_REWIRE
//! fan-out, cycle refusal, and 1,000 rewires without a dropped ACK.

mod support;

use hiverouter_contract::command::{
    ApprovalResolve, CancelRun, FileRead, FileTreeList, RewireTask, SettingsStatus, ViewportSet,
};
use hiverouter_contract::event::NodeEvent;
use hiverouter_contract::ids::{ApprovalId, RunId, TaskId};
use hiverouter_contract::{schema, AgentState, Command, CommandName, Event, EventName, Refusal};
use serde_json::Value;
use support::{error_code, frame_name, recv_until, recv_with_timeout, WsClient, TestServer, TEST_ORIGIN};

async fn connect(server: &TestServer, last_seq: Option<u64>) -> WsClient {
    let mut client = WsClient::connect(server.addr, TEST_ORIGIN, last_seq)
        .await
        .expect("websocket connect");
    client.hello(last_seq).await.expect("send hello");
    client
}

async fn send_command(client: &mut WsClient, command: &Command) -> uuid::Uuid {
    let envelope = schema::encode_command(command).expect("encode command");
    let correlation = envelope.correlation_id;
    client
        .send_text(&envelope.to_json().expect("serialize command"))
        .await
        .expect("send command");
    correlation
}

fn ack_correlation(value: &Value) -> Option<&str> {
    if frame_name(value) != "ACK" {
        return None;
    }
    value
        .get("payload")
        .and_then(|payload| payload.get("correlation_id"))
        .and_then(Value::as_str)
}

#[tokio::test]
async fn origin_rejected_when_not_allowlisted() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let (status, body) =
        support::upgrade_status(server.addr, "/ws", Some("http://evil.example"), support::TEST_TOKEN)
            .await;
    assert_eq!(status, 403, "non-allowlisted Origin must be refused: {body}");
    assert!(
        body.contains("E_ORIGIN_REJECTED"),
        "refusal body must name the code: {body}"
    );

    // Missing Origin is refused too: browsers always send one for WS upgrades.
    let (status, body) = support::upgrade_status(server.addr, "/ws", None, support::TEST_TOKEN).await;
    assert_eq!(status, 403, "missing Origin must be refused: {body}");
    assert!(body.contains("E_ORIGIN_REJECTED"));
}

#[tokio::test]
async fn token_invalid_is_refused() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let (status, body) = support::upgrade_status(server.addr, "/ws", Some(TEST_ORIGIN), "wrong-token").await;
    assert_eq!(status, 401, "bad token must be refused: {body}");
    assert!(body.contains("E_TOKEN_INVALID"), "body: {body}");
}

#[tokio::test]
async fn ws_fanout_two_clients() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let mut first = connect(&server, None).await;
    let mut second = connect(&server, None).await;

    let hello_first = recv_until(&mut first, |value| frame_name(value) == "SERVER_HELLO").await;
    let hello_second = recv_until(&mut second, |value| frame_name(value) == "SERVER_HELLO").await;
    assert_eq!(hello_first["payload"]["schema"], "hiverouter.v1");
    assert_eq!(hello_second["payload"]["schema"], "hiverouter.v1");

    let task_id = TaskId::new();
    server
        .state
        .bus
        .publish(Event::NodeDrag(NodeEvent {
            task_id,
            state: AgentState::Running,
            rev: None,
            kind: None,
            parent_task_id: None,
        }))
        .expect("publish drag");

    let first_view = recv_until(&mut first, |value| frame_name(value) == "NODE_DRAG").await;
    let second_view = recv_until(&mut second, |value| frame_name(value) == "NODE_DRAG").await;
    assert_eq!(first_view["payload"]["task_id"], task_id.to_string());
    assert_eq!(second_view["payload"]["task_id"], task_id.to_string());
    assert_eq!(first_view["seq"], second_view["seq"], "both clients see the same sequenced event");
    assert_eq!(first_view["seq"].as_u64(), Some(1));
}

#[tokio::test]
async fn hello_replay_and_gap_resync() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;

    // Three events are published before the reconnecting client arrives.
    for _ in 0..3 {
        server
            .state
            .bus
            .publish(Event::NodeDrag(NodeEvent {
                task_id: TaskId::new(),
                state: AgentState::Running,
                rev: None,
                kind: None,
                parent_task_id: None,
            }))
            .expect("publish");
    }

    let mut reconnecting = connect(&server, Some(1)).await;
    let hello = recv_until(&mut reconnecting, |value| frame_name(value) == "SERVER_HELLO").await;
    assert_eq!(hello["payload"]["last_seq"], 3);
    let replay_first = recv_with_timeout(&mut reconnecting).await;
    let replay_second = recv_with_timeout(&mut reconnecting).await;
    assert_eq!(replay_first["seq"], 2, "replay starts after the client sequence");
    assert_eq!(replay_second["seq"], 3);

    // A sequence older than the replay buffer forces a resync.
    for _ in 0..hiverouter_core::bus::REPLAY_CAPACITY + 5 {
        server.state.bus.publish(Event::Pong(Default::default())).expect("publish");
    }
    let mut far_behind = connect(&server, Some(1)).await;
    let resync = recv_until(&mut far_behind, |value| frame_name(value) == "RESYNC_REQUIRED").await;
    assert!(
        resync["payload"]["expected_seq"].as_u64().expect("expected_seq") > 1,
        "resync must name the first available sequence: {resync}"
    );
    assert_eq!(resync["payload"]["last_seq"], 1);
}

#[tokio::test]
async fn invalid_json_frame_keeps_connection_usable() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let mut client = connect(&server, None).await;
    recv_until(&mut client, |value| frame_name(value) == "SERVER_HELLO").await;

    client.send_text("{not json").await.expect("send bad frame");
    let error = recv_until(&mut client, |value| frame_name(value) == "ERROR").await;
    assert_eq!(error_code(&error), Refusal::BadFrame.code());

    // The connection is still usable: a valid command is ACKed.
    let command = Command::ViewportSet(ViewportSet {
        x: 10.0,
        y: 20.0,
        zoom: 1.5,
    });
    let correlation = send_command(&mut client, &command).await;
    let ack = recv_until(&mut client, |value| {
        ack_correlation(value) == Some(correlation.to_string().as_str())
    })
    .await;
    assert_eq!(ack["payload"]["command"], "VIEWPORT_SET");
}

#[tokio::test]
async fn rewire_ack_and_node_event_fan_out_to_both_clients() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let root_a = TaskId::new();
    let root_b = TaskId::new();
    let task = TaskId::new();
    server.state.graph.add_task(root_a, None).expect("root a");
    server.state.graph.add_task(root_b, None).expect("root b");
    server.state.graph.add_task(task, Some(root_a)).expect("task");
    let run = server
        .state
        .engine
        .store
        .create_run("p", "rewire", AgentState::Running)
        .expect("run");
    server
        .state
        .engine
        .store
        .insert_task(&hiverouter_core::entities::TaskRecord {
            id: root_b,
            run_id: run.id,
            parent_task_id: None,
            role: "planner".to_owned(),
            depth: 0,
            spawn_seq: 0,
            state: AgentState::Running,
            rev: 0,
        })
        .expect("root b row");
    server
        .state
        .engine
        .store
        .insert_task(&hiverouter_core::entities::TaskRecord {
            id: task,
            run_id: run.id,
            parent_task_id: Some(root_a),
            role: "super".to_owned(),
            depth: 1,
            spawn_seq: 2,
            state: AgentState::Running,
            rev: 0,
        })
        .expect("task row");

    let mut sender = connect(&server, None).await;
    let mut watcher = connect(&server, None).await;
    recv_until(&mut sender, |value| frame_name(value) == "SERVER_HELLO").await;
    recv_until(&mut watcher, |value| frame_name(value) == "SERVER_HELLO").await;

    let correlation = send_command(
        &mut sender,
        &Command::RewireTask(RewireTask {
            task_id: task,
            new_parent_id: root_b,
            expected_rev: 0,
        }),
    )
    .await;
    let ack = recv_until(&mut watcher, |value| {
        ack_correlation(value) == Some(correlation.to_string().as_str())
    })
    .await;
    assert_eq!(ack["payload"]["command"], "REWIRE_TASK");
    let node_event = recv_until(&mut watcher, |value| frame_name(value) == "NODE_REWIRE").await;
    assert_eq!(node_event["payload"]["task_id"], task.to_string());
    assert_eq!(
        node_event["payload"]["kind"], "superagent",
        "NODE_REWIRE must carry the authoritative task kind"
    );
    assert_eq!(
        server.state.graph.parent_of(task).expect("parent"),
        Some(root_b)
    );
    let stored = server.state.engine.store.get_task(task).expect("task row");
    assert_eq!(
        stored.parent_task_id,
        Some(root_b),
        "the task row must follow the graph rewire, not keep the stale parent"
    );
    let graph_rev = server
        .state
        .graph
        .node(task)
        .expect("node")
        .expect("node exists")
        .rev;
    assert_eq!(stored.rev, graph_rev, "row and graph revision must agree");
}

#[tokio::test]
async fn rewire_cycle_is_refused_over_ws_and_graph_is_unchanged() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let root = TaskId::new();
    let middle = TaskId::new();
    let leaf = TaskId::new();
    server.state.graph.add_task(root, None).expect("root");
    server.state.graph.add_task(middle, Some(root)).expect("middle");
    server.state.graph.add_task(leaf, Some(middle)).expect("leaf");
    let edges_before = server.state.graph.edge_count().expect("edge count");

    let mut client = connect(&server, None).await;
    recv_until(&mut client, |value| frame_name(value) == "SERVER_HELLO").await;

    // middle is an ancestor of leaf: moving middle under leaf closes a cycle.
    let correlation = send_command(
        &mut client,
        &Command::RewireTask(RewireTask {
            task_id: middle,
            new_parent_id: leaf,
            expected_rev: 0,
        }),
    )
    .await;
    let error = recv_until(&mut client, |value| {
        frame_name(value) == "ERROR" && value["correlation_id"] == correlation.to_string()
    })
    .await;
    assert_eq!(error_code(&error), Refusal::Cycle.code());
    assert_eq!(error["payload"]["retryable"], false);
    assert_eq!(
        server.state.graph.edge_count().expect("edge count"),
        edges_before,
        "E_CYCLE must leave the graph unchanged"
    );
    assert_eq!(
        server.state.graph.parent_of(middle).expect("parent"),
        Some(root)
    );
}

#[tokio::test]
async fn thousand_rewires_without_a_dropped_ack() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let first_parent = TaskId::new();
    let second_parent = TaskId::new();
    let task = TaskId::new();
    server.state.graph.add_task(first_parent, None).expect("p1");
    server.state.graph.add_task(second_parent, None).expect("p2");
    server.state.graph.add_task(task, Some(first_parent)).expect("task");

    let mut client = connect(&server, None).await;
    recv_until(&mut client, |value| frame_name(value) == "SERVER_HELLO").await;

    for index in 0..1_000_u64 {
        let new_parent = if index % 2 == 0 { second_parent } else { first_parent };
        let correlation = send_command(
            &mut client,
            &Command::RewireTask(RewireTask {
                task_id: task,
                new_parent_id: new_parent,
                expected_rev: index,
            }),
        )
        .await;
        let expected = correlation.to_string();
        let ack = recv_until(&mut client, |value| {
            ack_correlation(value) == Some(expected.as_str())
        })
        .await;
        assert_eq!(ack["payload"]["command"], "REWIRE_TASK");
    }
    assert_eq!(server.state.graph.revision().expect("revision") >= 1_003, true);
}

#[tokio::test]
async fn run_commands_are_handled_by_the_phase4_engine() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let mut client = connect(&server, None).await;
    recv_until(&mut client, |value| frame_name(value) == "SERVER_HELLO").await;

    // CANCEL_RUN now reaches the Phase 4 engine and refuses a missing run by name.
    let correlation = send_command(
        &mut client,
        &Command::CancelRun(CancelRun {
            run_id: RunId::new(),
        }),
    )
    .await;
    let error = recv_until(&mut client, |value| {
        frame_name(value) == "ERROR" && value["correlation_id"] == correlation.to_string()
    })
    .await;
    assert_eq!(error_code(&error), Refusal::RunNotFound.code());
    assert_eq!(CommandName::CancelRun.as_str(), "CANCEL_RUN");
    assert_eq!(EventName::NodeRewire.as_str(), "NODE_REWIRE");
}

fn seed_pending_approval(server: &TestServer) -> (ApprovalId, String) {
    let gate = server.state.tools.approval_gate();
    let now = hiverouter_contract::Timestamp::now().unix_secs().unwrap_or(0);
    server
        .state
        .db
        .with_connection(|conn| {
            let request = gate
                .request(
                    conn,
                    RunId::new(),
                    TaskId::new(),
                    "call-1",
                    hiverouter_tools::tool::ToolId::FileWrite,
                    hiverouter_tools::tool::PermissionClass::Mutate,
                    &serde_json::json!({"path": "note.txt", "content": "x"}),
                    now,
                )
                .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
            Ok((request.approval_id, request.nonce))
        })
        .expect("seed pending approval")
}

#[tokio::test]
async fn settings_status_command_returns_result_over_ws() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let mut client = connect(&server, None).await;
    recv_until(&mut client, |value| frame_name(value) == "SERVER_HELLO").await;

    let correlation = send_command(&mut client, &Command::SettingsStatus(SettingsStatus {})).await;
    let ack = recv_until(&mut client, |value| {
        frame_name(value) == "ACK" && value["correlation_id"] == correlation.to_string()
    })
    .await;
    assert_eq!(ack["payload"]["command"], "SETTINGS_STATUS");
    let result = recv_until(&mut client, |value| frame_name(value) == "SETTINGS_STATUS_RESULT").await;
    assert_eq!(result["payload"]["provider"], "deepseek");
    assert_eq!(result["payload"]["model"], "deepseek-flash");
    assert!(result["payload"]["key_present"].is_boolean());
    let rendered = result.to_string();
    assert!(!rendered.contains("sk-"), "settings must never expose the key");
    assert!(!rendered.contains("DEEPSEEK_API_KEY"));
}

#[tokio::test]
async fn file_tree_and_file_read_commands_use_the_jail() {
    let temp = tempfile::tempdir().expect("temp dir");
    std::fs::write(temp.path().join("note.txt"), b"hello jail").expect("write note");
    let server = TestServer::start(&temp).await;
    let mut client = connect(&server, None).await;
    recv_until(&mut client, |value| frame_name(value) == "SERVER_HELLO").await;

    // FILE_TREE_LIST -> ACK + FILE_TREE_RESULT with the real jail root.
    let correlation = send_command(&mut client, &Command::FileTreeList(FileTreeList { path: None })).await;
    recv_until(&mut client, |value| {
        frame_name(value) == "ACK" && value["correlation_id"] == correlation.to_string()
    })
    .await;
    let tree = recv_until(&mut client, |value| frame_name(value) == "FILE_TREE_RESULT").await;
    let entries = tree["payload"]["entries"].as_array().expect("entries");
    assert!(
        entries.iter().any(|entry| entry["name"] == "note.txt"),
        "file tree must list the workspace file: {tree}"
    );

    // FILE_READ -> ACK + FILE_CONTENT with real bytes/sha/text.
    let correlation = send_command(
        &mut client,
        &Command::FileRead(FileRead {
            path: "note.txt".to_owned(),
        }),
    )
    .await;
    recv_until(&mut client, |value| {
        frame_name(value) == "ACK" && value["correlation_id"] == correlation.to_string()
    })
    .await;
    let content = recv_until(&mut client, |value| frame_name(value) == "FILE_CONTENT").await;
    assert_eq!(content["payload"]["path"], "note.txt");
    assert_eq!(content["payload"]["text"], "hello jail");
    assert_eq!(content["payload"]["bytes"], 10);
    assert_eq!(
        content["payload"]["sha256"].as_str().expect("sha").len(),
        64
    );

    // Missing file -> named refusal.
    let correlation = send_command(
        &mut client,
        &Command::FileRead(FileRead {
            path: "nope.txt".to_owned(),
        }),
    )
    .await;
    let error = recv_until(&mut client, |value| {
        frame_name(value) == "ERROR" && value["correlation_id"] == correlation.to_string()
    })
    .await;
    assert_eq!(error_code(&error), Refusal::PathNotFound.code());

    // Escape attempt -> named refusal, never an outside read.
    let correlation = send_command(
        &mut client,
        &Command::FileRead(FileRead {
            path: "../outside.txt".to_owned(),
        }),
    )
    .await;
    let error = recv_until(&mut client, |value| {
        frame_name(value) == "ERROR" && value["correlation_id"] == correlation.to_string()
    })
    .await;
    assert_eq!(error_code(&error), Refusal::PathOutsideJail.code());
}

#[tokio::test]
async fn file_commands_refuse_by_name_when_workspace_is_unavailable() {
    let temp = tempfile::tempdir().expect("temp dir");
    let missing_root = temp.path().join("no-such-workspace");
    let server = TestServer::start_with_workspace(&temp, Some(&missing_root)).await;
    let mut client = connect(&server, None).await;
    recv_until(&mut client, |value| frame_name(value) == "SERVER_HELLO").await;
    assert!(!server.state.tools.file_access_available());

    for command in [
        Command::FileTreeList(FileTreeList { path: None }),
        Command::FileRead(FileRead {
            path: "anything.txt".to_owned(),
        }),
    ] {
        let correlation = send_command(&mut client, &command).await;
        let error = recv_until(&mut client, |value| {
            frame_name(value) == "ERROR" && value["correlation_id"] == correlation.to_string()
        })
        .await;
        assert_eq!(
            error_code(&error),
            Refusal::FileTreeUnavailable.code(),
            "unavailable workspace must refuse truthfully, never E_INTERNAL_ERROR"
        );
        assert_ne!(error_code(&error), Refusal::InternalError.code());
    }
}

#[tokio::test]
async fn approval_resolve_command_acks_and_emits_resolved_event() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let (approval_id, nonce) = seed_pending_approval(&server);
    let mut client = connect(&server, None).await;
    recv_until(&mut client, |value| frame_name(value) == "SERVER_HELLO").await;

    let payload = ApprovalResolve {
        approval_id,
        nonce,
        decision: "approve".to_owned(),
        reason: Some("operator approved".to_owned()),
    };
    let correlation = send_command(&mut client, &Command::ApprovalResolve(payload.clone())).await;
    recv_until(&mut client, |value| {
        frame_name(value) == "ACK" && value["correlation_id"] == correlation.to_string()
    })
    .await;
    let resolved = recv_until(&mut client, |value| frame_name(value) == "APPROVAL_RESOLVED").await;
    assert_eq!(resolved["payload"]["approval_id"], approval_id.to_string());
    assert_eq!(resolved["payload"]["tool"], "file_write");
    assert_eq!(resolved["payload"]["decision"], "approve");

    // Replaying the same nonce is a named refusal, never a second grant.
    let correlation = send_command(&mut client, &Command::ApprovalResolve(payload)).await;
    let error = recv_until(&mut client, |value| {
        frame_name(value) == "ERROR" && value["correlation_id"] == correlation.to_string()
    })
    .await;
    assert_eq!(error_code(&error), Refusal::ApprovalReplay.code());
}
