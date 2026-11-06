//! Provider error matrix — deterministic, offline, wiremock-backed.
//!
//! Every case is a fault-injection unit for the retry/classification contract in
//! `provider/src/client.rs`:
//!
//! - 429 exhaustion maps the internal `ProviderRateLimit` to the final
//!   `E_LLM_RATE_LIMIT` (both for a non-streaming call and for a stream open),
//!   after exactly `MAX_ATTEMPTS` HTTP requests;
//! - 401/402 are named `E_PROVIDER_AUTH` refusals and are **never** retried
//!   (complete and stream open);
//! - request timeout and connect failure both map to `E_PROVIDER_TIMEOUT`
//!   (the frozen single transport refusal), with timeout retried up to the cap;
//! - a stream error **before** the first byte is retried (open loop), while a
//!   stream failure **after** a delivered delta is terminal (`E_PROVIDER_INCOMPLETE`)
//!   and never re-opens the request.
//!
//! No key material, no network and no paid call: the client targets a local
//! `wiremock` server or a closed loopback port, and the API-key fixture is an
//! obvious offline placeholder.

use std::time::Duration;

use hiverouter_contract::Refusal;
use hiverouter_provider::{
    ApiKey, ChatMessage, ChatRequest, ClientConfig, DeepSeekClient, Model, StreamEvent, MAX_ATTEMPTS,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Offline placeholder. Never a real credential; the server is local wiremock.
const TEST_KEY: &str = "sk-wiremock-offline-only";

/// Valid non-streaming chat-completion body (used only when a 200 would parse).
const CHAT_BODY: &str = r#"{"id":"cmpl-matrix","model":"deepseek-flash",
    "choices":[{"index":0,"message":{"role":"assistant","content":"ok"},"finish_reason":"stop"}],
    "usage":{"prompt_tokens":10,"completion_tokens":5,"total_tokens":15}}"#;

/// The documented retry budget is part of the contract under test.
const ATTEMPTS: usize = 3;

fn attempts() -> usize {
    let expected = usize::try_from(MAX_ATTEMPTS).expect("MAX_ATTEMPTS fits usize");
    assert_eq!(expected, ATTEMPTS, "contract: 3 attempts including the first");
    expected
}

fn client_for(server: &MockServer) -> DeepSeekClient {
    let config = ClientConfig::default()
        .with_base_url(server.uri())
        .with_request_timeout(Duration::from_millis(500));
    DeepSeekClient::new(config, ApiKey::from_secret(TEST_KEY).expect("offline test key"))
        .expect("client")
}

fn request() -> ChatRequest {
    ChatRequest::new(Model::Flash, vec![ChatMessage::user("offline error matrix")])
}

/// One SSE data frame carrying `content` (no terminal `[DONE]`).
fn delta_frame(content: &str) -> String {
    format!(
        "data: {{\"id\":\"c1\",\"choices\":[{{\"index\":0,\"delta\":{{\"content\":\"{content}\"}},\"finish_reason\":null}}]}}\n\n"
    )
}

/// A complete SSE success body: one delta + `[DONE]`.
fn complete_sse(content: &str) -> String {
    format!(
        "{}data: {{\"id\":\"c1\",\"choices\":[{{\"index\":0,\"delta\":{{}},\"finish_reason\":\"stop\"}}]}}\n\ndata: [DONE]\n\n",
        delta_frame(content)
    )
}

async fn recorded_requests(server: &MockServer) -> usize {
    server.received_requests().await.expect("wiremock records").len()
}

// ---------------------------------------------------------------------------
// 429 exhaustion: ProviderRateLimit -> E_LLM_RATE_LIMIT (final mapping)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn rate_limit_429_exhaustion_maps_provider_rate_limit_to_llm_rate_limit() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "0"))
        .mount(&server)
        .await;

    let error = client_for(&server)
        .complete(&request())
        .await
        .expect_err("429 must exhaust the retry budget and refuse");

    assert_eq!(error.code(), "E_LLM_RATE_LIMIT");
    assert_eq!(
        error.refusal(),
        Refusal::LlmRateLimit,
        "the internal ProviderRateLimit is mapped to the final LlmRateLimit"
    );
    assert!(error.retryable(), "rate limiting is retryable");
    assert_eq!(error.http_status(), Some(429));
    assert_eq!(
        recorded_requests(&server).await,
        attempts(),
        "the retry budget is exhausted at exactly MAX_ATTEMPTS requests"
    );
}

#[tokio::test]
async fn stream_rate_limit_429_exhaustion_maps_to_llm_rate_limit() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "0"))
        .mount(&server)
        .await;

    let error = match client_for(&server).stream(&request()).await {
        Ok(_) => panic!("stream open must exhaust the retry budget and refuse"),
        Err(error) => error,
    };

    assert_eq!(error.code(), "E_LLM_RATE_LIMIT");
    assert_eq!(error.refusal(), Refusal::LlmRateLimit);
    assert!(error.retryable());
    assert_eq!(error.http_status(), Some(429));
    assert_eq!(
        recorded_requests(&server).await,
        attempts(),
        "the stream open retries at exactly MAX_ATTEMPTS requests, then stops"
    );
}

// ---------------------------------------------------------------------------
// 401/402: named auth refusal, never retried (complete and stream open)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn auth_401_is_not_retried() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    let error = client_for(&server)
        .complete(&request())
        .await
        .expect_err("401 must be refused");

    assert_eq!(error.code(), "E_PROVIDER_AUTH");
    assert_eq!(error.refusal(), Refusal::ProviderAuth);
    assert!(!error.retryable(), "401 is never retried");
    assert_eq!(error.http_status(), Some(401));
    assert_eq!(recorded_requests(&server).await, 1, "401 sends one request only");
}

#[tokio::test]
async fn payment_required_402_is_not_retried() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(402))
        .mount(&server)
        .await;

    let error = client_for(&server)
        .complete(&request())
        .await
        .expect_err("402 must be refused");

    assert_eq!(error.code(), "E_PROVIDER_AUTH");
    assert_eq!(error.refusal(), Refusal::ProviderAuth);
    assert!(!error.retryable(), "402 is never retried");
    assert_eq!(error.http_status(), Some(402));
    assert_eq!(recorded_requests(&server).await, 1, "402 sends one request only");
}

#[tokio::test]
async fn stream_open_401_is_not_retried() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    let error = match client_for(&server).stream(&request()).await {
        Ok(_) => panic!("stream open 401 must be refused"),
        Err(error) => error,
    };

    assert_eq!(error.code(), "E_PROVIDER_AUTH");
    assert!(!error.retryable(), "stream open 401 is never retried");
    assert_eq!(
        recorded_requests(&server).await,
        1,
        "a non-retryable open failure sends one request only"
    );
}

// ---------------------------------------------------------------------------
// Transport boundaries: request timeout and connect failure
// ---------------------------------------------------------------------------

#[tokio::test]
async fn request_timeout_maps_to_provider_timeout_with_bounded_retries() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(2))
                .set_body_string(CHAT_BODY),
        )
        .mount(&server)
        .await;

    let config = ClientConfig::default()
        .with_base_url(server.uri())
        .with_request_timeout(Duration::from_millis(150));
    let client = DeepSeekClient::new(config, ApiKey::from_secret(TEST_KEY).expect("test key"))
        .expect("client");

    let error = client
        .complete(&request())
        .await
        .expect_err("a timed-out request must refuse, not fabricate success");

    assert_eq!(error.code(), "E_PROVIDER_TIMEOUT");
    assert_eq!(error.refusal(), Refusal::ProviderTimeout);
    assert!(error.retryable(), "timeouts are retried up to the budget");
    assert_eq!(
        recorded_requests(&server).await,
        attempts(),
        "timeout retries stop at the contract cap, never beyond it"
    );
}

#[tokio::test]
async fn connect_failure_maps_to_provider_timeout() {
    // Port 1 on loopback is closed: the connect fails deterministically and is
    // mapped by `map_transport_error` to the frozen transport refusal.
    let config = ClientConfig::default()
        .with_base_url("http://127.0.0.1:1")
        .with_request_timeout(Duration::from_millis(300));
    let client = DeepSeekClient::new(config, ApiKey::from_secret(TEST_KEY).expect("test key"))
        .expect("client");

    let error = client
        .complete(&request())
        .await
        .expect_err("a refused connection must be a named refusal");

    assert_eq!(error.code(), "E_PROVIDER_TIMEOUT");
    assert_eq!(error.refusal(), Refusal::ProviderTimeout);
    assert!(error.retryable());
    assert!(!error.detail().is_empty(), "the refusal keeps non-secret context");
}

// ---------------------------------------------------------------------------
// Streaming: retry before the first byte, never after a delivered delta
// ---------------------------------------------------------------------------

#[tokio::test]
async fn stream_error_before_first_byte_is_retried() {
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
                .set_body_string(complete_sse("recovered")),
        )
        .mount(&server)
        .await;

    let mut handle = client_for(&server)
        .stream(&request())
        .await
        .expect("the open failure before any byte is retried");

    let attempts = handle.attempts;
    let mut text = String::new();
    let mut finished = false;
    while let Some(event) = handle.next().await {
        match event.expect("stream event") {
            StreamEvent::Delta { content, .. } => text.push_str(&content),
            StreamEvent::ToolCallDelta { .. } => {}
            StreamEvent::Finished { .. } => {
                finished = true;
                break;
            }
        }
    }

    assert_eq!(attempts, 2, "exactly one retry before the first byte");
    assert_eq!(text, "recovered");
    assert!(finished, "the retried stream reaches its documented terminator");
    assert_eq!(recorded_requests(&server).await, 2, "two HTTP requests were sent");
}

#[tokio::test]
async fn stream_error_after_first_delta_is_not_retried() {
    let server = MockServer::start().await;
    // A 200 SSE that delivers one delta and then ends without `[DONE]`.
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(delta_frame("partial")),
        )
        .mount(&server)
        .await;

    let mut handle = client_for(&server)
        .stream(&request())
        .await
        .expect("the stream opened successfully");

    let mut saw_delta = false;
    let error = loop {
        match handle.next().await {
            Some(Ok(StreamEvent::Delta { .. })) => saw_delta = true,
            Some(Ok(StreamEvent::Finished { .. })) => {
                panic!("a stream without [DONE] must not report success")
            }
            Some(Ok(StreamEvent::ToolCallDelta { .. })) => {}
            Some(Err(error)) => break error,
            None => panic!("the stream closed without a named refusal"),
        }
    };

    assert!(saw_delta, "the delta was delivered before the failure");
    assert_eq!(
        error.code(),
        "E_PROVIDER_INCOMPLETE",
        "an unterminated stream is incomplete, not success"
    );
    assert_eq!(
        recorded_requests(&server).await,
        1,
        "no retry is sent after a delta reached the caller"
    );
}
