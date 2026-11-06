//! Streaming adapter matrix (offline, wiremock): the production adapter must
//! forward ordered SSE fragments to the delta sink while the stream arrives,
//! persist the concatenation, settle usage from the final chunk, and refuse by
//! name on malformed/incomplete streams. Cancellation and pre-first-byte retry
//! are exercised here without any key or network call.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use hiverouter_contract::{AgentState, Refusal, RunId, TaskId};
use hiverouter_core::config::ProviderSettings;
use hiverouter_core::entities::TaskRecord;
use hiverouter_core::execution::{DeltaSink, ProviderExecutor};
use hiverouter_core::provider_executor::ProviderExecutorAdapter;
use hiverouter_provider::client::{ClientConfig, DeepSeekClient};
use hiverouter_provider::secret::ApiKey;
use tokio_util::sync::CancellationToken;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const USAGE_JSON: &str = r#"{"prompt_tokens":10,"completion_tokens":5,"total_tokens":15,"prompt_cache_hit_tokens":2,"prompt_cache_miss_tokens":8}"#;

fn adapter_for(server: &MockServer) -> ProviderExecutorAdapter {
    let client = DeepSeekClient::new(
        ClientConfig::default().with_base_url(server.uri()),
        ApiKey::from_secret("sk-wiremock-adapter-only").expect("test key"),
    )
    .expect("client");
    ProviderExecutorAdapter::new(client, ProviderSettings::default())
}

fn task(run_id: RunId) -> TaskRecord {
    TaskRecord {
        id: TaskId::new(),
        run_id,
        parent_task_id: None,
        role: "worker".to_owned(),
        depth: 1,
        spawn_seq: 1,
        state: AgentState::Running,
        rev: 0,
    }
}

fn delta_chunk(content: &str) -> String {
    format!(
        r#"{{"id":"c1","choices":[{{"index":0,"delta":{{"content":"{content}"}},"finish_reason":null}}]}}"#
    )
}

fn final_chunk(usage: Option<&str>) -> String {
    match usage {
        Some(usage) => format!(
            r#"{{"id":"c1","choices":[{{"index":0,"delta":{{}},"finish_reason":"stop"}}],"usage":{usage}}}"#
        ),
        None => r#"{"id":"c1","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}"#.to_owned(),
    }
}

fn sse(parts: &[String], done: bool) -> String {
    let mut body = String::new();
    for part in parts {
        body.push_str("data: ");
        body.push_str(part);
        body.push_str("\n\n");
    }
    if done {
        body.push_str("data: [DONE]\n\n");
    }
    body
}

fn stream_body(parts: &[&str], usage: Option<&str>) -> String {
    let mut frames: Vec<String> = parts.iter().map(|part| delta_chunk(part)).collect();
    frames.push(final_chunk(usage));
    sse(&frames, true)
}

fn recording_sink() -> (DeltaSink, Arc<Mutex<Vec<String>>>) {
    let fragments: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sink: DeltaSink = {
        let fragments = Arc::clone(&fragments);
        Arc::new(move |delta: &str| {
            if let Ok(mut log) = fragments.lock() {
                log.push(delta.to_owned());
            }
        })
    };
    (sink, fragments)
}

#[tokio::test]
async fn execute_streams_fragments_in_order_and_persists_text() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(stream_body(&["Al", "pha"], Some(USAGE_JSON))),
        )
        .mount(&server)
        .await;

    let adapter = adapter_for(&server);
    let (sink, fragments) = recording_sink();
    let reply = adapter
        .execute(RunId::new(), &task(RunId::new()), "stream this", &CancellationToken::new(), &sink)
        .await
        .expect("streamed reply");

    let fragments = fragments.lock().expect("fragments").clone();
    assert_eq!(fragments, vec!["Al", "pha"], "sink receives chunks in arrival order");
    assert_eq!(reply.text(), "Alpha", "persisted text is the concatenation");
    assert_eq!(reply.usage.prompt_tokens, Some(10));
    assert_eq!(reply.usage.completion_tokens, Some(5));
}

#[tokio::test]
async fn usage_absent_after_stream_is_incomplete_not_zero() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(stream_body(&["visible text"], None)),
        )
        .mount(&server)
        .await;

    let adapter = adapter_for(&server);
    let (sink, fragments) = recording_sink();
    let error = adapter
        .execute(RunId::new(), &task(RunId::new()), "stream this", &CancellationToken::new(), &sink)
        .await
        .expect_err("absent usage is INCOMPLETE");
    assert_eq!(error, Refusal::UsageIncomplete);
    assert!(
        !fragments.lock().expect("fragments").is_empty(),
        "text was streamed before usage settlement failed"
    );
}

#[tokio::test]
async fn malformed_sse_is_a_named_refusal() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string("data: {not json}\n\n"),
        )
        .mount(&server)
        .await;

    let adapter = adapter_for(&server);
    let (sink, _) = recording_sink();
    let error = adapter
        .execute(RunId::new(), &task(RunId::new()), "stream this", &CancellationToken::new(), &sink)
        .await
        .expect_err("malformed SSE refused");
    assert_eq!(error, Refusal::ProviderInvalidResponse);
}

#[tokio::test]
async fn missing_done_terminator_is_a_named_refusal_without_retry() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(sse(&[delta_chunk("partial")], false)),
        )
        .mount(&server)
        .await;

    let adapter = adapter_for(&server);
    let (sink, fragments) = recording_sink();
    let error = adapter
        .execute(RunId::new(), &task(RunId::new()), "stream this", &CancellationToken::new(), &sink)
        .await
        .expect_err("missing [DONE] refused");
    assert_eq!(error, Refusal::ProviderIncomplete);
    assert_eq!(fragments.lock().expect("fragments").len(), 1);
    let requests = server.received_requests().await.expect("recorded");
    assert_eq!(requests.len(), 1, "no retry after a delivered delta");
}

#[tokio::test]
async fn error_before_first_byte_is_retried_then_streams() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "0"))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(stream_body(&["retried"], Some(USAGE_JSON))),
        )
        .mount(&server)
        .await;

    let adapter = adapter_for(&server);
    let (sink, fragments) = recording_sink();
    let reply = adapter
        .execute(RunId::new(), &task(RunId::new()), "stream this", &CancellationToken::new(), &sink)
        .await
        .expect("retried open then streamed reply");
    assert_eq!(reply.text(), "retried");
    assert_eq!(fragments.lock().expect("fragments").as_slice(), ["retried"]);
    let requests = server.received_requests().await.expect("recorded");
    assert_eq!(requests.len(), 2, "one retry before the first byte");
}

#[tokio::test]
async fn cancellation_stops_a_streamed_reply() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(stream_body(&["slow"], Some(USAGE_JSON)))
                .set_delay(Duration::from_millis(400)),
        )
        .mount(&server)
        .await;

    let adapter = Arc::new(adapter_for(&server));
    let token = CancellationToken::new();
    let cancel_for_task = token.clone();
    let run_id = RunId::new();
    let executor = Arc::clone(&adapter);
    let handle = tokio::spawn(async move {
        let (sink, _) = recording_sink();
        executor
            .execute(run_id, &task(run_id), "stream this", &cancel_for_task, &sink)
            .await
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    token.cancel();
    let error = handle
        .await
        .expect("join")
        .expect_err("cancelled reply must not succeed");
    assert_eq!(error, Refusal::ToolCancelled);
}

#[tokio::test]
async fn length_finish_reason_is_incomplete_not_success() {
    let server = MockServer::start().await;
    let body = sse(
        &[
            delta_chunk("partial"),
            format!(
                r#"{{"id":"c1","choices":[{{"index":0,"delta":{{}},"finish_reason":"length"}}],"usage":{USAGE_JSON}}}"#
            ),
        ],
        true,
    );
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(body),
        )
        .mount(&server)
        .await;
    let adapter = adapter_for(&server);
    let (sink, fragments) = recording_sink();
    let error = adapter
        .execute(RunId::new(), &task(RunId::new()), "truncate this", &CancellationToken::new(), &sink)
        .await
        .expect_err("truncated reply must be refused");
    assert_eq!(error, Refusal::ProviderIncomplete);
    assert_eq!(*fragments.lock().expect("fragments"), vec!["partial".to_owned()]);
}
