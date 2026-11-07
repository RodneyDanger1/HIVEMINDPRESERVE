//! `APPROVAL_RESOLVE` evidence wiring: exactly one run-ledger row per committed
//! decision, and never a ledger row for a refused command.
//!
//! These are transport-level acceptance tests: they drive the real frozen WS
//! command through the in-process server, then read the append-only JSONL ledger
//! the command is required to write (`<data_dir>/ledger/run-ledger.jsonl`).

mod support;

use hiverouter_contract::command::ApprovalResolve;
use hiverouter_contract::ids::{ApprovalId, RunId, TaskId};
use hiverouter_contract::{schema, Command, Refusal, Timestamp};
use serde_json::Value;
use support::{error_code, frame_name, recv_until, TestServer, WsClient, TEST_ORIGIN};

async fn connect(server: &TestServer) -> WsClient {
    let mut client = WsClient::connect(server.addr, TEST_ORIGIN, None)
        .await
        .expect("websocket connect");
    client.hello(None).await.expect("send hello");
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

fn seed_args() -> serde_json::Value {
    serde_json::json!({"path": "note.txt", "content": "x"})
}

/// Seeds one committed, pending approval row (the same shape the future
/// tool-loop approval producer commits) and returns its resolution material.
fn seed_pending_approval(server: &TestServer) -> (ApprovalId, String, RunId) {
    let gate = server.state.tools.approval_gate();
    let now = Timestamp::now().unix_secs().unwrap_or(0);
    let run = RunId::new();
    let (approval_id, nonce) = server
        .state
        .db
        .with_connection(|conn| {
            let request = gate
                .request(
                    conn,
                    run,
                    TaskId::new(),
                    "call-approval-wiring",
                    hiverouter_tools::tool::ToolId::FileWrite,
                    hiverouter_tools::tool::PermissionClass::Mutate,
                    &seed_args(),
                    now,
                )
                .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
            Ok((request.approval_id, request.nonce))
        })
        .expect("seed pending approval");
    (approval_id, nonce, run)
}

fn ledger_rows(server: &TestServer) -> Vec<Value> {
    let file = hiverouter_core::ToolsBridge::approval_ledger_file(&server.state.config.data_dir);
    if !file.exists() {
        return Vec::new();
    }
    std::fs::read_to_string(&file)
        .expect("ledger readable")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("valid ledger json line"))
        .collect()
}

#[tokio::test]
async fn approval_resolve_writes_exactly_one_ledger_row_and_replay_writes_none() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let (approval_id, nonce, run) = seed_pending_approval(&server);
    let mut client = connect(&server).await;
    recv_until(&mut client, |value| frame_name(value) == "SERVER_HELLO").await;

    let payload = ApprovalResolve {
        approval_id,
        nonce: nonce.clone(),
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
    assert_eq!(
        resolved["payload"]["decision"], "approve",
        "APPROVAL_RESOLVED must echo the truthful wire decision"
    );

    // The command appended exactly one row for this decision, bound to the run
    // and to the approved argument digest (never the argument body).
    let rows = ledger_rows(&server);
    assert_eq!(rows.len(), 1, "one committed decision = one ledger row");
    assert_eq!(rows[0]["outcome"].as_str(), Some("allowed"));
    assert_eq!(rows[0]["run_id"].as_str(), Some(run.to_string().as_str()));
    assert_eq!(rows[0]["tool_call_id"].as_str(), Some("call-approval-wiring"));
    assert_eq!(
        rows[0]["args_sha256"].as_str(),
        Some(hiverouter_tools::approval::hash_args(&seed_args()).as_str())
    );
    assert!(!rows[0].to_string().contains("\"content\""), "argument body never ledgered");

    // Replaying the consumed nonce is a named refusal and writes no second row.
    let correlation = send_command(&mut client, &Command::ApprovalResolve(payload)).await;
    let error = recv_until(&mut client, |value| {
        frame_name(value) == "ERROR" && value["correlation_id"] == correlation.to_string()
    })
    .await;
    assert_eq!(error_code(&error), Refusal::ApprovalReplay.code());
    assert_eq!(ledger_rows(&server).len(), 1, "refused replay is never ledgered");
}

#[tokio::test]
async fn concurrent_committed_decisions_append_exactly_one_ledger_row() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let (approval_id, nonce, run) = seed_pending_approval(&server);
    let bridge = server.state.tools.clone();
    let ledger_dir =
        hiverouter_core::ToolsBridge::approval_ledger_dir(&server.state.config.data_dir);
    let payload = ApprovalResolve {
        approval_id,
        nonce,
        decision: "approve".to_owned(),
        reason: Some("concurrent operator decision".to_owned()),
    };

    // Two committed decision attempts race for the same single-use nonce on real
    // blocking threads (the gate/DB CAS is synchronous). Only one may win; only
    // the winning command may append one ledger row, and the loser must observe
    // the named replay refusal.
    let first_bridge = bridge.clone();
    let second_bridge = bridge.clone();
    let first_payload = payload.clone();
    let second_payload = payload.clone();
    let first_ledger = ledger_dir.clone();
    let second_ledger = ledger_dir.clone();
    let first_task = tokio::task::spawn_blocking(move || {
        first_bridge.resolve_approval_ledgered(&first_payload, &first_ledger)
    });
    let second_task = tokio::task::spawn_blocking(move || {
        second_bridge.resolve_approval_ledgered(&second_payload, &second_ledger)
    });
    let (first, second) = tokio::join!(first_task, second_task);
    let first = first.expect("first decision task");
    let second = second.expect("second decision task");
    assert!(
        first.is_ok() ^ second.is_ok(),
        "exactly one concurrent decision may win: first={first:?} second={second:?}"
    );
    let loser = first
        .as_ref()
        .err()
        .or(second.as_ref().err())
        .expect("exactly one refusal");
    assert_eq!(*loser, Refusal::ApprovalReplay);

    let rows = ledger_rows(&server);
    assert_eq!(
        rows.len(),
        1,
        "one committed decision = exactly one ledger row, even under a race"
    );
    assert_eq!(rows[0]["outcome"].as_str(), Some("allowed"));
    assert_eq!(rows[0]["run_id"].as_str(), Some(run.to_string().as_str()));
    assert_eq!(rows[0]["tool_call_id"].as_str(), Some("call-approval-wiring"));

    // A later replay is refused and appends nothing.
    let replay = bridge
        .resolve_approval_ledgered(&payload, &ledger_dir)
        .expect_err("late replay is refused");
    assert_eq!(replay, Refusal::ApprovalReplay);
    assert_eq!(ledger_rows(&server).len(), 1);
}

#[tokio::test]
async fn refused_approval_resolve_never_writes_a_ledger_row() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let mut client = connect(&server).await;
    recv_until(&mut client, |value| frame_name(value) == "SERVER_HELLO").await;

    let payload = ApprovalResolve {
        approval_id: ApprovalId::new(),
        nonce: "nonce-for-a-request-that-does-not-exist".to_owned(),
        decision: "approve".to_owned(),
        reason: None,
    };
    let correlation = send_command(&mut client, &Command::ApprovalResolve(payload)).await;
    let error = recv_until(&mut client, |value| {
        frame_name(value) == "ERROR" && value["correlation_id"] == correlation.to_string()
    })
    .await;
    assert_eq!(error_code(&error), Refusal::ApprovalNotFound.code());
    assert!(
        ledger_rows(&server).is_empty(),
        "a refused APPROVAL_RESOLVE must never be ledgered"
    );
}
