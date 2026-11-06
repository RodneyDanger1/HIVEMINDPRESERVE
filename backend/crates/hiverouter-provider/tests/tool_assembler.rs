//! Wiremock integration tests for `ToolCallAssembler` (provider layer).
//!
//! These tests prove the assembler works on the real transport path: a local
//! wiremock server streams documented SSE tool-call fragments, the provider
//! `stream()` parses them into [`StreamEvent::ToolCallDelta`] values, and the
//! assembler merges them into typed calls. Nothing is executed — the assembler
//! only produces values.
//!
//! No key, no network, no paid call: the client targets the local mock server
//! with an obvious offline fixture key.

use std::time::Duration;

use hiverouter_provider::{
    ApiKey, AssembledToolCall, ChatMessage, ChatRequest, ClientConfig, DeepSeekClient, Model,
    StreamEvent, StreamHandle, ToolCallAssembler, ToolCallAssemblyError,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const TEST_KEY: &str = "sk-wiremock-tool-assembler";

fn client_for(server: &MockServer) -> DeepSeekClient {
    let config = ClientConfig::default()
        .with_base_url(server.uri())
        .with_request_timeout(Duration::from_millis(500));
    DeepSeekClient::new(config, ApiKey::from_secret(TEST_KEY).expect("offline test key"))
        .expect("client")
}

fn request() -> ChatRequest {
    ChatRequest::new(Model::Flash, vec![ChatMessage::user("assemble tools")])
}

/// One SSE data frame (no `[DONE]`).
fn frame(payload: &str) -> String {
    format!("data: {payload}\n\n")
}

/// A complete streaming body: frames + the documented `[DONE]` terminator.
fn body(frames: &[String]) -> String {
    let mut body = String::new();
    for frame in frames {
        body.push_str(frame);
    }
    body.push_str("data: [DONE]\n\n");
    body
}

/// A chunk whose delta carries exactly one tool-call fragment.
fn tool_chunk(tool_call: &str) -> String {
    frame(&format!(
        r#"{{"id":"c1","choices":[{{"index":0,"delta":{{"tool_calls":[{tool_call}]}},"finish_reason":null}}]}}"#
    ))
}

/// The final choice chunk that closes the assistant message.
fn final_chunk() -> String {
    frame(r#"{"id":"c1","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}"#)
}

/// Feeds every stream event to a fresh assembler and returns `finish()`.
async fn assemble(mut handle: StreamHandle) -> Result<Vec<AssembledToolCall>, ToolCallAssemblyError> {
    let mut assembler = ToolCallAssembler::new();
    while let Some(event) = handle.next().await {
        let event = event.expect("stream event");
        if assembler.push_event(&event).expect("fragments are valid") {
            continue;
        }
        if matches!(event, StreamEvent::Finished { .. }) {
            break;
        }
    }
    assembler.finish()
}

#[tokio::test]
async fn wiremock_stream_assembles_two_calls_in_index_order() {
    let server = MockServer::start().await;
    let sse = body(&[
        // index 0: id + name + first half of the arguments.
        tool_chunk(
            r#"{"index":0,"id":"call_a","type":"function","function":{"name":"file_read","arguments":"{\"path\":"}}"#,
        ),
        // index 1: a complete single-fragment call.
        tool_chunk(
            r#"{"index":1,"id":"call_b","type":"function","function":{"name":"file_list","arguments":"{}"}}"#,
        ),
        // index 0: the argument remainder arrives last.
        tool_chunk(r#"{"index":0,"function":{"arguments":"\"a.txt\"}"}}"#),
        final_chunk(),
    ]);
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(sse),
        )
        .mount(&server)
        .await;

    let handle = client_for(&server).stream(&request()).await.expect("stream opens");
    assert_eq!(handle.attempts, 1);
    let calls = assemble(handle).await.expect("both calls assemble");

    assert_eq!(calls.len(), 2, "both indices are present");
    assert_eq!(calls[0].index, 0, "output is sorted by index");
    assert_eq!(calls[0].id, "call_a");
    assert_eq!(calls[0].name, "file_read");
    assert_eq!(calls[0].arguments, serde_json::json!({"path": "a.txt"}));
    assert_eq!(calls[1].index, 1);
    assert_eq!(calls[1].id, "call_b");
    assert_eq!(calls[1].name, "file_list");
    assert_eq!(calls[1].arguments, serde_json::json!({}));

    let requests = server.received_requests().await.expect("recorded");
    assert_eq!(requests.len(), 1, "one streamed request, no retry");
}

#[tokio::test]
async fn wiremock_stream_missing_id_is_provider_invalid_response() {
    let server = MockServer::start().await;
    let sse = body(&[
        // A truncated call: the id never arrives on any fragment.
        tool_chunk(
            r#"{"index":0,"type":"function","function":{"name":"file_read","arguments":"{}"}}"#,
        ),
        final_chunk(),
    ]);
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(sse),
        )
        .mount(&server)
        .await;

    let handle = client_for(&server).stream(&request()).await.expect("stream opens");
    let error = assemble(handle)
        .await
        .expect_err("a call without an id must be refused at finish");
    assert_eq!(error.code(), "E_PROVIDER_INVALID_RESPONSE");
    assert_eq!(
        error,
        ToolCallAssemblyError::MissingId { index: 0 },
        "the truncation is attributed to the call index"
    );
}

#[tokio::test]
async fn wiremock_stream_invalid_arguments_maps_to_tool_schema() {
    let server = MockServer::start().await;
    let sse = body(&[
        tool_chunk(
            r#"{"index":0,"id":"call_a","type":"function","function":{"name":"file_read","arguments":"{oops"}}"#,
        ),
        final_chunk(),
    ]);
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(sse),
        )
        .mount(&server)
        .await;

    let handle = client_for(&server).stream(&request()).await.expect("stream opens");
    let error = assemble(handle)
        .await
        .expect_err("invalid argument JSON is refused at finish");
    assert!(
        matches!(error, ToolCallAssemblyError::InvalidArgumentsJson { index: 0, .. }),
        "argument-shape failures are distinct from truncation"
    );
    assert_eq!(error.code(), "E_TOOL_SCHEMA");
}
