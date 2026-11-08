//! Offline provider refusal matrix (Phase 5 WP 5.8, EXT-R5-03).
//!
//! Every case uses `wiremock` as a local HTTP server — no key, no network, no
//! live DeepSeek call. The matrix exercises the named refusals:
//! 401 (auth), 402 (auth, not retried), 429 + Retry-After (retried, final
//! E_LLM_RATE_LIMIT), socket/read timeout (E_PROVIDER_TIMEOUT), malformed JSON
//! (E_PROVIDER_INVALID_RESPONSE), and NULL usage (INCOMPLETE, never $0.00).
//! Budget preflight is asserted to refuse with **zero** provider requests.

use std::time::Duration;

use hiverouter_contract::Timestamp;
use hiverouter_provider::{
    ApiKey, CallEstimate, ChatMessage, ChatRequest, ClientConfig, DeepSeekClient, Model, RunBudget,
    BudgetLimits,
};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client_for(server: &MockServer) -> DeepSeekClient {
    let config = ClientConfig::default()
        .with_base_url(server.uri())
        .with_request_timeout(Duration::from_millis(500));
    DeepSeekClient::new(config, ApiKey::from_secret("sk-wiremock-offline-only").expect("key"))
        .expect("client")
}

fn request() -> ChatRequest {
    ChatRequest::new(Model::Flash, vec![ChatMessage::user("offline matrix")])
}

fn unix(rfc3339: &str) -> u64 {
    Timestamp::parse(rfc3339)
        .expect("valid timestamp")
        .unix_secs()
        .expect("representable")
}

const USAGE_JSON: &str = r#""usage":{"prompt_tokens":100,"completion_tokens":50,"total_tokens":150,
    "prompt_cache_hit_tokens":40,"prompt_cache_miss_tokens":60,
    "completion_tokens_details":{"reasoning_tokens":10}}"#;

fn chat_body(usage: &str) -> String {
    format!(
        r#"{{"id":"cmpl-test","model":"deepseek-flash",
            "choices":[{{"index":0,"message":{{"role":"assistant","content":"ok"}},"finish_reason":"stop"}}],
            {usage}}}"#
    )
}

#[tokio::test]
async fn auth_401_is_a_named_refusal_and_is_not_retried() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let error = client.complete(&request()).await.expect_err("401 refused");
    assert_eq!(error.code(), "E_PROVIDER_AUTH");
    assert!(!error.retryable(), "401 is never retried");
    assert_eq!(error.http_status(), Some(401));
    let requests = server.received_requests().await.expect("recorded");
    assert_eq!(requests.len(), 1, "401 must not be retried");
}

#[tokio::test]
async fn payment_required_402_is_auth_and_not_retried() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(402))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let error = client.complete(&request()).await.expect_err("402 refused");
    assert_eq!(error.code(), "E_PROVIDER_AUTH");
    let requests = server.received_requests().await.expect("recorded");
    assert_eq!(requests.len(), 1, "402 must not be retried");
}

#[tokio::test]
async fn rate_limit_429_retries_with_retry_after_then_reports_llm_rate_limit() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "0"))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let error = client.complete(&request()).await.expect_err("429 refused");
    assert_eq!(
        error.code(),
        "E_LLM_RATE_LIMIT",
        "persistent 429 after the retry budget is the documented LLM rate-limit refusal"
    );
    assert!(error.retryable());
    let requests = server.received_requests().await.expect("recorded");
    assert_eq!(requests.len(), 3, "retry budget is 3 attempts");
}

#[tokio::test]
async fn rate_limit_recovers_after_retry_after() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("Authorization", "Bearer sk-wiremock-offline-only"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "0"))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_string(chat_body(USAGE_JSON)))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let completion = client.complete(&request()).await.expect("recovered");
    assert_eq!(completion.attempts, 2);
    assert_eq!(completion.usage.expect("usage").completion_tokens, Some(50));
    let requests = server.received_requests().await.expect("recorded");
    assert_eq!(requests.len(), 2);
}

#[tokio::test]
async fn timeout_is_a_named_refusal() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(2))
                .set_body_string(chat_body(USAGE_JSON)),
        )
        .mount(&server)
        .await;

    let config = ClientConfig::default()
        .with_base_url(server.uri())
        .with_request_timeout(Duration::from_millis(150));
    let client = DeepSeekClient::new(
        config,
        ApiKey::from_secret("sk-wiremock-offline-only").expect("key"),
    )
    .expect("client");
    let error = client.complete(&request()).await.expect_err("timeout");
    assert_eq!(error.code(), "E_PROVIDER_TIMEOUT");
    assert!(error.retryable());
}

#[tokio::test]
async fn malformed_json_is_a_named_refusal() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{not-the-documented-shape"))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let error = client.complete(&request()).await.expect_err("malformed");
    assert_eq!(error.code(), "E_PROVIDER_INVALID_RESPONSE");
    assert!(!error.retryable());
}

#[tokio::test]
async fn transient_500_is_retried_then_refused_by_name() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let error = client.complete(&request()).await.expect_err("500");
    assert_eq!(error.code(), "E_PROVIDER_TIMEOUT");
    assert_eq!(error.http_status(), Some(500));
    assert!(error.retryable(), "transient provider 5xx is retryable");
}

#[tokio::test]
async fn null_usage_marks_the_run_incomplete_and_never_settles_zero() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_string(chat_body(r#""usage":null"#)))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let mut budget = RunBudget::new(BudgetLimits::default());
    let error = client
        .complete_with_budget(
            &mut budget,
            &request(),
            &CallEstimate::default(),
            unix("2026-11-06T00:00:00Z"),
        )
        .await
        .expect_err("NULL usage is INCOMPLETE");
    assert_eq!(error.code(), "E_USAGE_INCOMPLETE");
    let report = budget.report();
    assert_eq!(report.status, "INCOMPLETE");
    assert_eq!(report.settled_usd, None, "NULL usage is never exported as 0.00");
    assert_eq!(report.incomplete_reason, Some("E_USAGE_INCOMPLETE"));
}

#[tokio::test]
async fn budget_preflight_refuses_with_zero_provider_requests() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_string(chat_body(USAGE_JSON)))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let mut limits = BudgetLimits::default();
    limits.call_cap = 0; // already exhausted before the first send
    let mut budget = RunBudget::new(limits);
    let error = client
        .complete_with_budget(
            &mut budget,
            &request(),
            &CallEstimate::default(),
            unix("2026-11-06T00:00:00Z"),
        )
        .await
        .expect_err("preflight refuses");
    assert_eq!(error.code(), "E_CALL_LIMIT");
    let requests = server.received_requests().await.expect("recorded");
    assert_eq!(requests.len(), 0, "budget refusal must happen before the request is sent");
}

#[tokio::test]
async fn success_path_records_usage_and_settles_decimal_cost() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_string(chat_body(USAGE_JSON)))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let mut budget = RunBudget::new(BudgetLimits::default());
    let completion = client
        .complete_with_budget(
            &mut budget,
            &request(),
            &CallEstimate::default(),
            unix("2026-11-06T00:00:00Z"), // Friday off-peak
        )
        .await
        .expect("success");
    assert_eq!(completion.attempts, 1);
    assert_eq!(completion.http_status, 200);
    let report = budget.report();
    assert_eq!(report.status, "COMPLETE");
    // 40 hit × 0.003/1M + 60 miss × 0.15/1M + 50 out × 0.6/1M = 0.00003912
    assert_eq!(report.settled_usd.as_deref(), Some("0.00003912"));
    assert_eq!(report.reasoning_tokens, Some(10));
}

#[tokio::test]
async fn streaming_client_parses_sse_and_reports_usage_none_as_incomplete() {
    let server = MockServer::start().await;
    let sse = concat!(
        ": keep-alive\n\n",
        "data: {\"id\":\"1\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hel\"},\"finish_reason\":null}]}\n\n",
        "data: {\"id\":\"1\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"lo\"},\"finish_reason\":\"stop\"}]}\n\n",
        "data: [DONE]\n\n",
    );
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "text/event-stream")
                .set_body_string(sse),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let mut stream_request = request();
    stream_request = stream_request.with_stream(true);
    let mut handle = client.stream(&stream_request).await.expect("stream opens");

    let mut text = String::new();
    let mut finished = false;
    while let Some(event) = handle.next().await {
        match event.expect("stream event") {
            hiverouter_provider::StreamEvent::Delta { content, .. } => text.push_str(&content),
            hiverouter_provider::StreamEvent::Finished { usage, finish_reason } => {
                assert_eq!(finish_reason.as_deref(), Some("stop"));
                assert!(usage.is_none(), "streaming usage is not documented; stays NULL");
                finished = true;
            }
            hiverouter_provider::StreamEvent::ToolCallDelta { .. } => {}
        }
    }
    assert_eq!(text, "Hello");
    assert!(finished, "stream ends with the terminal event");
}
