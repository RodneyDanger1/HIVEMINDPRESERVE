//! WP-04 deterministic wiremock tool-loop matrix (source-only; no paid path).
//!
//! These tests drive the *production* `ProviderExecutorAdapter` against a local
//! wiremock SSE server and the real `RunEngine` with a tempdir workspace jail.
//! They cover the five matrix cases requested by Run-3:
//!   1. truncated tool call (missing id / missing arguments) is refused by name;
//!   2. two parallel read-only calls execute in one turn;
//!   3. a multi-turn continuation finishes with the final answer;
//!   4. malformed SSE is a named refusal;
//!   5. the persisted turn cap is a named budget refusal (never a fake answer).
//!
//! Fixture: wiremock responds in request order (plan completion, then streamed
//! turns). No key, no live network, no side-effect approval path.
//! Source-only: the wave rules forbid build/run, so these are compiled by the
//! Lead gate build.

use std::collections::VecDeque;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use hiverouter_core::bus::{EventBus, SequencedEvent};
use hiverouter_core::config::ProviderSettings;
use hiverouter_core::db::Database;
use hiverouter_core::entities::Run;
use hiverouter_core::execution::{ProviderExecutor, RunEngine};
use hiverouter_core::graph::GraphAuthority;
use hiverouter_core::prompts::TemplateRegistry;
use hiverouter_core::provider_executor::ProviderExecutorAdapter;
use hiverouter_core::tool_loop::MAX_TOOL_TURNS;
use hiverouter_provider::client::{ClientConfig, DeepSeekClient};
use hiverouter_provider::secret::ApiKey;
use hiverouter_tools::WorkspaceJail;
use serde_json::{json, Value};
use tokio::sync::broadcast::error::RecvError;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const NOTE_TEXT: &str = "tool-loop-matrix note";
const USAGE_JSON: &str = r#"{"prompt_tokens":10,"completion_tokens":5,"total_tokens":15,"prompt_cache_hit_tokens":2,"prompt_cache_miss_tokens":8}"#;
const PLAN_CONTENT: &str = r#"{"nodes":[{"id":"n1","role":"worker","deps":[],"depth":1}]}"#;

fn usage_value() -> Value {
    json!({
        "prompt_tokens": 10,
        "completion_tokens": 5,
        "total_tokens": 15,
        "prompt_cache_hit_tokens": 2,
        "prompt_cache_miss_tokens": 8
    })
}

/// Non-streaming planner response consumed by `ProviderExecutorAdapter::plan`.
fn plan_response() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "id": "plan-response-1",
        "model": "deepseek-flash",
        "choices": [{
            "index": 0,
            "message": { "role": "assistant", "content": PLAN_CONTENT },
            "finish_reason": "stop"
        }],
        "usage": usage_value()
    }))
}

fn frame(payload: &str) -> String {
    format!("data: {payload}\n\n")
}

/// One streamed assistant turn that requests tool calls, followed by the
/// documented terminal chunk (usage is mandatory for the adapter to settle).
fn tool_call_stream(tool_calls: &str) -> String {
    let mut body = String::new();
    body.push_str(&frame(&format!(
        r#"{{"id":"stream-tool","choices":[{{"index":0,"delta":{{"tool_calls":{tool_calls}}},"finish_reason":null}}]}}"#
    )));
    body.push_str(&frame(&format!(
        r#"{{"id":"stream-tool","choices":[{{"index":0,"delta":{{}},"finish_reason":"tool_calls"}}],"usage":{USAGE_JSON}}}"#
    )));
    body.push_str("data: [DONE]\n\n");
    body
}

/// One streamed assistant turn that produces visible final text.
fn text_stream(text: &str) -> String {
    let mut body = String::new();
    body.push_str(&frame(&format!(
        r#"{{"id":"stream-text","choices":[{{"index":0,"delta":{{"content":"{text}"}},"finish_reason":null}}]}}"#
    )));
    body.push_str(&frame(&format!(
        r#"{{"id":"stream-text","choices":[{{"index":0,"delta":{{}},"finish_reason":"stop"}}],"usage":{USAGE_JSON}}}"#
    )));
    body.push_str("data: [DONE]\n\n");
    body
}

fn sse_response(body: String) -> ResponseTemplate {
    ResponseTemplate::new(200)
        .insert_header("content-type", "text/event-stream")
        .set_body_string(body)
}

/// Mounts a request-ordered response script: the first request (planning) gets
/// the plan, then each streamed call gets the next scripted SSE body.
async fn scripted_server(responses: Vec<ResponseTemplate>) -> MockServer {
    let server = MockServer::start().await;
    let queue: Arc<Mutex<VecDeque<ResponseTemplate>>> =
        Arc::new(Mutex::new(responses.into_iter().collect()));
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(move |_request: &Request| {
            queue
                .lock()
                .ok()
                .and_then(|mut queued| queued.pop_front())
                .unwrap_or_else(|| {
                    ResponseTemplate::new(500).set_body_string("tool_loop_matrix script exhausted")
                })
        })
        .mount(&server)
        .await;
    server
}

struct Harness {
    engine: Arc<RunEngine>,
    bus: Arc<EventBus>,
    db: Arc<Database>,
    server: MockServer,
    _data: tempfile::TempDir,
}

async fn harness(responses: Vec<ResponseTemplate>, workspace: &Path) -> Harness {
    let server = scripted_server(responses).await;
    let data = tempfile::tempdir().expect("data tempdir");
    let db =
        Arc::new(Database::open(&data.path().join("hiverouter.db")).expect("open matrix database"));
    db.migrate().expect("matrix migrations");
    let bus = Arc::new(EventBus::new());
    let client = DeepSeekClient::new(
        ClientConfig::default().with_base_url(server.uri()),
        ApiKey::from_secret("sk-wiremock-tool-loop-matrix").expect("offline test key"),
    )
    .expect("wiremock client");
    let executor: Arc<dyn ProviderExecutor> = Arc::new(ProviderExecutorAdapter::new(
        client,
        ProviderSettings::default(),
    ));
    let engine = RunEngine::new(
        db.clone(),
        bus.clone(),
        Arc::new(GraphAuthority::new()),
        executor,
        Arc::new(TemplateRegistry::default()),
    )
    .with_tool_workspace(WorkspaceJail::open(workspace).expect("workspace jail"));
    Harness {
        engine: Arc::new(engine),
        bus,
        db,
        server,
        _data: data,
    }
}

#[derive(Debug, Default)]
struct Collected {
    tool_calls: Vec<Value>,
    tool_results: Vec<Value>,
    reply_deltas: Vec<String>,
    final_status: Value,
}

async fn next_event(
    receiver: &mut tokio::sync::broadcast::Receiver<SequencedEvent>,
) -> SequencedEvent {
    loop {
        match tokio::time::timeout(Duration::from_secs(10), receiver.recv()).await {
            Ok(Ok(event)) => return event,
            Ok(Err(RecvError::Lagged(_))) => continue,
            Ok(Err(RecvError::Closed)) => panic!("engine event bus closed"),
            Err(_) => panic!("timed out waiting for an engine event"),
        }
    }
}

fn is_terminal_status(payload: &Value) -> bool {
    matches!(
        payload["status"].as_str(),
        Some("completed" | "failed" | "cancelled" | "killed" | "interrupted")
    )
}

/// Drives one submitted run until its terminal `RUN_STATUS` frame. Non-terminal
/// RUN_STATUS frames (for example the post-plan `running` frame) are ignored so
/// the matrix does not mistake a progress update for the final state.
async fn drive(harness: &Harness, prompt: &str) -> (Run, Collected) {
    let mut receiver = harness.bus.subscribe();
    let run = harness
        .engine
        .submit_prompt(prompt, "project-tool-loop-matrix", Uuid::now_v7())
        .expect("submit matrix prompt");
    let mut collected = Collected::default();
    loop {
        let event = next_event(&mut receiver).await;
        match event.envelope.name.as_str() {
            "TOOL_CALL" => collected.tool_calls.push(event.envelope.payload.clone()),
            "TOOL_RESULT" => collected.tool_results.push(event.envelope.payload.clone()),
            "REPLY_DELTA" => {
                if let Some(text) = event.envelope.payload["text_delta"].as_str() {
                    collected.reply_deltas.push(text.to_owned());
                }
            }
            "RUN_STATUS" => {
                collected.final_status = event.envelope.payload.clone();
                if is_terminal_status(&collected.final_status) {
                    break;
                }
            }
            _ => {}
        }
    }
    (run, collected)
}

fn workspace_with_note() -> tempfile::TempDir {
    let workspace = tempfile::tempdir().expect("workspace");
    std::fs::write(workspace.path().join("note.txt"), NOTE_TEXT.as_bytes())
        .expect("write workspace note");
    workspace
}

fn read_call(index: u32, id: &str) -> String {
    format!(
        r#"{{"index":{index},"id":"{id}","type":"function","function":{{"name":"file_read","arguments":"{{\"path\":\"note.txt\"}}"}}}}"#
    )
}

#[tokio::test]
async fn tool_loop_matrix_truncated_tool_call_is_refused() {
    let workspace = workspace_with_note();
    // Case A: the assembled call never gets an id. Case B: it never gets
    // arguments. Both are provider-invalid responses, never executed calls.
    let missing_id = format!(
        r#"[{{"index":0,"type":"function","function":{{"name":"file_read","arguments":"{{\"path\":\"note.txt\"}}"}}}}]"#
    );
    let missing_arguments = r#"[{"index":0,"id":"call_missing_args","type":"function","function":{"name":"file_read"}}]"#;

    for tool_calls in [missing_id.as_str(), missing_arguments] {
        let harness = harness(
            vec![plan_response(), sse_response(tool_call_stream(tool_calls))],
            workspace.path(),
        )
        .await;
        let (run, collected) = drive(&harness, "refuse a truncated tool call").await;

        assert_eq!(
            collected.final_status["status"], "failed",
            "a truncated provider tool call must fail the run"
        );
        assert_eq!(
            collected.final_status["error_code"], "E_PROVIDER_INVALID_RESPONSE",
            "the refusal is the named provider-invalid code, never a fake completion"
        );
        assert!(
            collected.tool_results.is_empty(),
            "nothing is announced/executed for an unassembled call"
        );
        assert_eq!(
            harness.db.run_tool_counters(run.id).expect("counters"),
            (0, 0),
            "no tool turn or call is counted"
        );
        assert!(harness
            .db
            .tool_turns_for_run(run.id)
            .expect("tool turns")
            .is_empty());
        let requests = harness.server.received_requests().await.expect("requests");
        assert_eq!(
            requests.len(),
            2,
            "plan completion + one failed stream; no continuation is attempted"
        );
    }
}

#[tokio::test]
async fn tool_loop_matrix_two_parallel_read_calls_execute_in_one_turn() {
    let workspace = workspace_with_note();
    let two_calls = format!(
        r#"[{},{{"index":1,"id":"call_list","type":"function","function":{{"name":"file_list","arguments":"{{}}"}}}}]"#,
        read_call(0, "call_read")
    );
    let harness = harness(
        vec![
            plan_response(),
            sse_response(tool_call_stream(&two_calls)),
            sse_response(text_stream("parallel turn answer")),
        ],
        workspace.path(),
    )
    .await;
    let (run, collected) = drive(&harness, "read two things in parallel").await;

    assert_eq!(collected.final_status["status"], "completed");
    assert_eq!(
        collected.tool_calls.len(),
        2,
        "both calls of the parallel turn are announced"
    );
    assert_eq!(
        collected.tool_results.len(),
        2,
        "both parallel results are published"
    );
    assert!(
        collected
            .tool_results
            .iter()
            .all(|result| result["outcome"] == "ok"),
        "both read-only calls execute: {collected:?}"
    );
    assert_eq!(
        collected.reply_deltas,
        vec!["parallel turn answer".to_owned()],
        "the final continuation supplies the visible answer"
    );
    assert_eq!(
        harness.db.run_tool_counters(run.id).expect("counters"),
        (1, 2),
        "one completed tool turn with two calls"
    );
    let turns = harness.db.tool_turns_for_run(run.id).expect("tool turns");
    assert_eq!(turns.len(), 2);
    assert_eq!(
        turns.iter().map(|turn| turn.turn).collect::<Vec<_>>(),
        vec![1, 1]
    );
    let requests = harness.server.received_requests().await.expect("requests");
    assert_eq!(
        requests.len(),
        3,
        "plan + one parallel-tool turn + one final continuation"
    );
}

#[tokio::test]
async fn tool_loop_matrix_multi_turn_continuation_completes() {
    let workspace = workspace_with_note();
    let harness = harness(
        vec![
            plan_response(),
            sse_response(tool_call_stream(&format!(
                "[{}]",
                read_call(0, "call_read_1")
            ))),
            sse_response(tool_call_stream(&format!(
                "[{}]",
                read_call(0, "call_read_2")
            ))),
            sse_response(text_stream("multi-turn final answer")),
        ],
        workspace.path(),
    )
    .await;
    let (run, collected) = drive(&harness, "read twice then answer").await;

    assert_eq!(collected.final_status["status"], "completed");
    assert_eq!(collected.tool_calls.len(), 2);
    assert_eq!(collected.tool_results.len(), 2);
    assert_eq!(
        collected.reply_deltas,
        vec!["multi-turn final answer".to_owned()]
    );
    assert_eq!(
        harness.db.run_tool_counters(run.id).expect("counters"),
        (2, 2),
        "two completed turns and two calls"
    );
    let turns = harness.db.tool_turns_for_run(run.id).expect("tool turns");
    assert_eq!(turns.len(), 2);
    assert_eq!(turns[0].turn, 1);
    assert_eq!(turns[1].turn, 2);
    assert_eq!(
        turns[0].tool_call_id, "call_read_1",
        "the cumulative transcript keeps the first turn's call"
    );
    let requests = harness.server.received_requests().await.expect("requests");
    assert_eq!(
        requests.len(),
        4,
        "plan + two streamed tool turns + one final continuation"
    );
}

#[tokio::test]
async fn tool_loop_matrix_malformed_sse_is_named_refusal() {
    let workspace = workspace_with_note();
    let harness = harness(
        vec![
            plan_response(),
            sse_response("data: {not json}\n\n".to_owned()),
        ],
        workspace.path(),
    )
    .await;
    let (run, collected) = drive(&harness, "feed a malformed stream").await;

    assert_eq!(collected.final_status["status"], "failed");
    assert_eq!(
        collected.final_status["error_code"], "E_PROVIDER_INVALID_RESPONSE",
        "malformed SSE is a named provider refusal"
    );
    assert!(collected.tool_results.is_empty());
    assert_eq!(
        harness.db.run_tool_counters(run.id).expect("counters"),
        (0, 0)
    );
    let requests = harness.server.received_requests().await.expect("requests");
    assert_eq!(
        requests.len(),
        2,
        "plan + one malformed stream; the adapter does not retry an opened stream"
    );
}

#[tokio::test]
async fn tool_loop_matrix_turn_cap_is_named_call_refusal() {
    let workspace = workspace_with_note();
    let mut responses = vec![plan_response()];
    for index in 0..=MAX_TOOL_TURNS {
        let calls = format!("[{}]", read_call(0, &format!("call_cap_{index}")));
        responses.push(sse_response(tool_call_stream(&calls)));
    }
    let harness = harness(responses, workspace.path()).await;
    let (run, collected) = drive(&harness, "keep calling the tool").await;

    assert_eq!(
        collected.final_status["status"], "failed",
        "the turn cap must fail the run, never fabricate a final answer"
    );
    assert_eq!(
        collected.final_status["error_code"], "E_CALL_LIMIT",
        "the persisted turn cap is a named call refusal, not a money refusal"
    );
    assert!(
        collected.reply_deltas.is_empty(),
        "no assistant text is invented for a cap-failed tool loop"
    );
    assert_eq!(
        harness.db.run_tool_counters(run.id).expect("counters"),
        (MAX_TOOL_TURNS, MAX_TOOL_TURNS),
        "exactly MAX_TOOL_TURNS calls execute"
    );
    assert_eq!(
        harness
            .db
            .tool_turns_for_run(run.id)
            .expect("tool turns")
            .len(),
        (MAX_TOOL_TURNS + 1) as usize,
        "executed turns plus the cap-refused call row"
    );
    let requests = harness.server.received_requests().await.expect("requests");
    assert_eq!(
        requests.len(),
        (MAX_TOOL_TURNS + 2) as usize,
        "plan + execute + continuations; no further request after the cap"
    );
}
