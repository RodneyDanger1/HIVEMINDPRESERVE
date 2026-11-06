//! WebSocket transport acceptance tests (Phase 2).
//!
//! Covers: Origin allowlist refusal, per-session token refusal, two-client
//! fan-out, hello replay/resync, invalid-frame error path, rewire ACK/NODE_REWIRE
//! fan-out, cycle refusal, and 1,000 rewires without a dropped ACK.

mod support;

use hiverouter_contract::command::{CancelRun, RewireTask, ViewportSet};
use hiverouter_contract::event::NodeEvent;
use hiverouter_contract::ids::{RunId, TaskId};
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
        server.state.graph.parent_of(task).expect("parent"),
        Some(root_b)
    );
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
async fn unimplemented_run_command_is_an_error_not_a_silent_drop() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let mut client = connect(&server, None).await;
    recv_until(&mut client, |value| frame_name(value) == "SERVER_HELLO").await;
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
    assert_eq!(error_code(&error), Refusal::InternalError.code());
    assert_eq!(
        error["payload"]["retryable"], true,
        "interim handler must be marked retryable"
    );
    assert_eq!(CommandName::CancelRun.as_str(), "CANCEL_RUN");
    assert_eq!(EventName::NodeRewire.as_str(), "NODE_REWIRE");
}
