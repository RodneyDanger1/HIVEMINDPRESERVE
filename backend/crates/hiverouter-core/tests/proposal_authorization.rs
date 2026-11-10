//! End-to-end authorization, apply, and rollback over the real HTTP routes with
//! four synthetic completed SuperAgent tasks. This is an API/DB test double for
//! the model calls, not a mock of the routes themselves.

mod support;

use std::time::Duration;

use hiverouter_contract::{AgentState, TaskId};
use hiverouter_core::entities::TaskRecord;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

async fn post_json(addr: std::net::SocketAddr, path: &str, token: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(addr).await.expect("connect");
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {addr}\r\nAuthorization: Bearer {token}\r\nAccept: application/json\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(request.as_bytes()).await.expect("write");
    let mut raw = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(5), stream.read_to_end(&mut raw)).await;
    let text = String::from_utf8_lossy(&raw).into_owned();
    let status = text
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .expect("status");
    let body = text.split("\r\n\r\n").nth(1).unwrap_or_default().to_owned();
    (status, body)
}

#[tokio::test]
async fn four_superagents_authorize_apply_and_rollback_a_lesson() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = support::TestServer::start(&temp).await;
    let store = &server.state.engine.store;

    let run = store
        .create_run("p", "authorized lesson", AgentState::Running)
        .expect("run");
    let root = TaskRecord {
        id: TaskId::new(),
        run_id: run.id,
        parent_task_id: None,
        role: "planner".to_owned(),
        depth: 0,
        spawn_seq: 0,
        state: AgentState::Completed,
        rev: 0,
    };
    store.insert_task(&root).expect("root");

    let proposal_id = uuid::Uuid::now_v7();
    let proposal = serde_json::json!({
        "proposal_id": proposal_id.to_string(),
        "area": "governance",
        "problem": "four votes are not enough",
        "proposal": "Apply a reviewed lesson after the fifth seat",
        "evidence": "test evidence",
        "applied": false,
        "governance": {
            "required_approvals": 5,
            "superagent_approvals": 4,
            "operator_approvals": 1,
            "unanimous": true,
            "state": "awaiting_votes",
            "can_apply": false
        }
    });
    store
        .record_agent_memory_scoped(
            "hyper",
            &format!("improvement:{proposal_id}"),
            &proposal.to_string(),
            run.id,
            root.id,
            "run",
        )
        .expect("proposal");

    let mut voters = Vec::new();
    for sequence in 1..=4 {
        let id = TaskId::new();
        store
            .insert_task(&TaskRecord {
                id,
                run_id: run.id,
                parent_task_id: Some(root.id),
                role: "superagent".to_owned(),
                depth: 1,
                spawn_seq: sequence,
                state: AgentState::Completed,
                rev: 0,
            })
            .expect("superagent task");
        store
            .record_agent_memory_scoped(
                "hyper",
                &format!("evolution_vote:{proposal_id}:{id}"),
                &serde_json::json!({
                    "proposal_id": proposal_id.to_string(),
                    "voter_task_id": id.to_string(),
                    "vote": "approve",
                    "rationale": "approved in test",
                })
                .to_string(),
                run.id,
                id,
                "run",
            )
            .expect("vote");
        voters.push(id);
    }
    assert_eq!(voters.len(), 4);

    let base = format!("/api/agents/runs/{}/proposals/{}", run.id, proposal_id);
    let (status, body) = post_json(server.addr, &format!("{base}/authorize"), support::TEST_TOKEN).await;
    assert_eq!(status, 200, "authorize: {body}");
    let value: Value = serde_json::from_str(&body).expect("authorize JSON");
    assert_eq!(value["state"], "authorized_not_applied");
    assert_eq!(value["applied"], false);

    let (status, body) = post_json(server.addr, &format!("{base}/apply"), support::TEST_TOKEN).await;
    assert_eq!(status, 200, "apply: {body}");
    let applied: Value = serde_json::from_str(&body).expect("apply JSON");
    assert_eq!(applied["applied"], true);
    assert_eq!(applied["scope"], "lesson_artifact_only");

    let lessons = std::fs::read_to_string(
        server
            .state
            .config
            .data_dir
            .join("evolution")
            .join("LESSONS.jsonl"),
    )
    .expect("lesson file");
    assert!(lessons.contains(&proposal_id.to_string()));

    let (status, body) = post_json(server.addr, &format!("{base}/rollback"), support::TEST_TOKEN).await;
    assert_eq!(status, 200, "rollback: {body}");
    let rolled: Value = serde_json::from_str(&body).expect("rollback JSON");
    assert_eq!(rolled["rolled_back"], true);
    let lessons = std::fs::read_to_string(
        server
            .state
            .config
            .data_dir
            .join("evolution")
            .join("LESSONS.jsonl"),
    )
    .expect("lesson file");
    assert!(!lessons.contains(&proposal_id.to_string()));
}
