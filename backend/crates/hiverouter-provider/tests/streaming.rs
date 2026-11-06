//! Streaming SSE matrix (offline, wiremock): ordered deltas, `[DONE]`, final-chunk
//! usage, retries only before the first byte, malformed SSE and budget preflight.
//!
//! No key, no network, no paid call: the client points at a local wiremock server.

use std::time::Duration;

use hiverouter_contract::{Timestamp, Usage};
use hiverouter_provider::{
    ApiKey, BudgetLimits, CallEstimate, ChatMessage, ChatRequest, ClientConfig, DeepSeekClient, Model,
    RunBudget, StreamEvent, StreamHandle,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const USAGE_JSON: &str = r#"{"prompt_tokens":7,"completion_tokens":3,"total_tokens":10,"prompt_cache_hit_tokens":2,"prompt_cache_miss_tokens":5}"#;

fn client_for(server: &MockServer) -> DeepSeekClient {
    let config = ClientConfig::default()
        .with_base_url(server.uri())
        .with_request_timeout(Duration::from_millis(500));
    DeepSeekClient::new(
        config,
        ApiKey::from_secret("sk-wiremock-stream-only").expect("test key"),
    )
    .expect("client")
}

fn request() -> ChatRequest {
    ChatRequest::new(Model::Flash, vec![ChatMessage::user("stream me")])
}

fn unix(rfc3339: &str) -> u64 {
    Timestamp::parse(rfc3339)
        .expect("valid timestamp")
        .unix_secs()
        .expect("representable")
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

fn text_stream(parts: &[&str], usage: Option<&str>) -> String {
    let mut frames: Vec<String> = parts.iter().map(|part| delta_chunk(part)).collect();
    frames.push(final_chunk(usage));
    sse(&frames, true)
}

async fn collect(mut handle: StreamHandle) -> (Vec<String>, Option<Usage>, Option<String>, u32) {
    let attempts = handle.attempts;
    let mut deltas = Vec::new();
    let mut usage = None;
    let mut finish_reason = None;
    while let Some(event) = handle.next().await {
        match event.expect("stream event") {
            StreamEvent::Delta { content, .. } => {
                if !content.is_empty() {
                    deltas.push(content);
                }
            }
            StreamEvent::ToolCallDelta { .. } => {}
            StreamEvent::Finished {
                usage: reported,
                finish_reason: reason,
            } => {
                usage = reported;
                finish_reason = reason;
                break;
            }
        }
    }
    (deltas, usage, finish_reason, attempts)
}

#[tokio::test]
async fn stream_yields_ordered_deltas_and_final_usage() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(text_stream(&["Hel", "lo", " world"], Some(USAGE_JSON))),
        )
        .mount(&server)
        .await;

    let handle = client_for(&server).stream(&request()).await.expect("stream");
    let (deltas, usage, finish, attempts) = collect(handle).await;
    assert_eq!(deltas, vec!["Hel", "lo", " world"], "deltas arrive in provider order");
    assert_eq!(finish.as_deref(), Some("stop"));
    assert_eq!(attempts, 1, "one successful open, no retry");
    let usage = usage.expect("final chunk carries usage");
    assert_eq!(usage.prompt_tokens, Some(7));
    assert_eq!(usage.completion_tokens, Some(3));
    assert_eq!(usage.total_tokens, Some(10));
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

    let mut handle = client_for(&server).stream(&request()).await.expect("stream");
    let error = loop {
        match handle.next().await {
            Some(Ok(StreamEvent::Finished { .. })) => panic!("malformed chunk must not finish"),
            Some(Ok(_)) => continue,
            Some(Err(error)) => break error,
            None => panic!("stream closed without a named refusal"),
        }
    };
    assert_eq!(error.code(), "E_PROVIDER_INVALID_RESPONSE");
}

#[tokio::test]
async fn missing_done_terminator_is_provider_incomplete() {
    let server = MockServer::start().await;
    let body = sse(&[delta_chunk("Hi"), final_chunk(Some(USAGE_JSON))], false);
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(body),
        )
        .mount(&server)
        .await;

    let mut handle = client_for(&server).stream(&request()).await.expect("stream");
    let mut saw_error = None;
    while let Some(event) = handle.next().await {
        if let Err(error) = event {
            saw_error = Some(error);
            break;
        }
    }
    let error = saw_error.expect("missing [DONE] must be refused");
    assert_eq!(error.code(), "E_PROVIDER_INCOMPLETE");
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
                .set_body_string(text_stream(&["ok"], Some(USAGE_JSON))),
        )
        .mount(&server)
        .await;

    let handle = client_for(&server).stream(&request()).await.expect("retried open");
    let (deltas, _, _, attempts) = collect(handle).await;
    assert_eq!(deltas, vec!["ok"]);
    assert_eq!(attempts, 2, "one retry before the first byte");
    let requests = server.received_requests().await.expect("recorded");
    assert_eq!(requests.len(), 2, "exactly one retry was sent");
}

#[tokio::test]
async fn stream_is_not_retried_after_a_delta() {
    let server = MockServer::start().await;
    // A 200 stream that fails after one delta and never emits [DONE].
    let body = sse(&[delta_chunk("partial")], false);
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(body),
        )
        .mount(&server)
        .await;

    let mut handle = client_for(&server).stream(&request()).await.expect("stream");
    let mut saw_delta = false;
    let mut saw_error = None;
    while let Some(event) = handle.next().await {
        match event {
            Ok(StreamEvent::Delta { .. }) => saw_delta = true,
            Ok(StreamEvent::Finished { .. }) => {
                panic!("stream without [DONE] must not report success")
            }
            Ok(StreamEvent::ToolCallDelta { .. }) => {}
            Err(error) => {
                saw_error = Some(error);
                break;
            }
        }
    }
    assert!(saw_delta, "the delta arrived before the failure");
    assert_eq!(
        saw_error.expect("missing [DONE] is refused").code(),
        "E_PROVIDER_INCOMPLETE"
    );
    let requests = server.received_requests().await.expect("recorded");
    assert_eq!(requests.len(), 1, "no retry after a delta was delivered");
}

#[tokio::test]
async fn budget_preflight_refuses_with_zero_requests() {
    let server = MockServer::start().await;
    let client = client_for(&server);
    let limits = BudgetLimits {
        call_cap: 0,
        ..BudgetLimits::default()
    };
    let mut budget = RunBudget::new(limits);

    let error = match client
        .stream_with_budget(&mut budget, &request(), &CallEstimate::default())
        .await
    {
        Ok(_) => panic!("call cap must refuse before any send"),
        Err(error) => error,
    };
    assert_eq!(error.code(), "E_CALL_LIMIT");
    assert_eq!(budget.attempts(), 0, "no attempt was recorded");
    let requests = server.received_requests().await.expect("recorded");
    assert!(requests.is_empty(), "preflight refusal sends nothing");
}

#[tokio::test]
async fn budgeted_stream_records_measured_usage() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(text_stream(&["A", "B"], Some(USAGE_JSON))),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let mut budget = RunBudget::new(BudgetLimits::default());
    let estimate = CallEstimate {
        prompt_tokens: Some(1),
        max_output_tokens: Some(100),
    };
    let handle = client
        .stream_with_budget(&mut budget, &request(), &estimate)
        .await
        .expect("stream");
    let (deltas, usage, _, attempts) = collect(handle).await;
    assert_eq!(deltas, vec!["A", "B"]);
    assert_eq!(attempts, 1);
    assert_eq!(budget.attempts(), 1, "the send attempt was counted");
    budget
        .record_call(Model::Flash, usage.as_ref(), unix("2026-11-06T01:00:00Z"))
        .expect("measured usage settles");
    assert_eq!(budget.calls(), 1);
    assert_eq!(budget.report().status, "COMPLETE");
    assert!(
        budget.remaining_usd() < budget.limits().run_cost_cap_usd,
        "a measured call consumes budget"
    );
}

#[tokio::test]
async fn absent_stream_usage_settles_incomplete_not_zero() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(text_stream(&["only text"], None)),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let mut budget = RunBudget::new(BudgetLimits::default());
    let handle = client
        .stream_with_budget(&mut budget, &request(), &CallEstimate::default())
        .await
        .expect("stream");
    let (_, usage, _, _) = collect(handle).await;
    assert!(usage.is_none(), "the stream omitted usage");
    let error = budget
        .record_call(Model::Flash, None, unix("2026-11-06T01:00:00Z"))
        .expect_err("absent usage is INCOMPLETE");
    assert_eq!(error.code(), "E_USAGE_INCOMPLETE");
    assert_eq!(budget.report().status, "INCOMPLETE");
    assert_eq!(
        budget
            .preflight(&CallEstimate::default())
            .expect_err("incomplete run refuses further sends")
            .code(),
        "E_USAGE_INCOMPLETE"
    );
}
