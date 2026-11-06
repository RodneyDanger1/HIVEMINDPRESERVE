//! Multi-call SSE transport tests (provider layer).
//!
//! These tests pin the Wave-1 transport requirement from
//! `plans/waves/W12-INTERFACES.md`: a single SSE `delta` carrying several
//! `tool_calls` entries must produce one [`StreamEvent::ToolCallDelta`] per
//! entry, in ascending `index` order, instead of only the first entry. They
//! also prove that the previous single-call behavior and the named
//! malformed-stream refusals are preserved, and that the provider
//! [`ToolCallAssembler`] can merge parallel calls and fragmented arguments.
//!
//! No key, no network, no paid call: the client points at a local wiremock
//! server with an obvious offline fixture key. Tests are deterministic; the
//! wave rules forbid running them during coding waves, so they are written for
//! the single Lead gate build.

use std::time::Duration;

use hiverouter_provider::{
    ApiKey, AssembledToolCall, ChatMessage, ChatRequest, ClientConfig, DeepSeekClient, Model,
    StreamEvent, StreamHandle, ToolCallAssembler,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const TEST_KEY: &str = "sk-wiremock-tool-transport";

fn client_for(server: &MockServer) -> DeepSeekClient {
    let config = ClientConfig::default()
        .with_base_url(server.uri())
        .with_request_timeout(Duration::from_millis(500));
    DeepSeekClient::new(config, ApiKey::from_secret(TEST_KEY).expect("offline test key"))
        .expect("client")
}

fn request() -> ChatRequest {
    ChatRequest::new(Model::Flash, vec![ChatMessage::user("transport tools")])
}

/// One SSE data frame (no `[DONE]`).
fn frame(payload: &str) -> String {
    format!("data: {payload}\n\n")
}

/// A complete SSE body: frames followed by the documented `[DONE]` terminator.
fn body(frames: &[String]) -> String {
    let mut body = String::new();
    for frame in frames {
        body.push_str(frame);
    }
    body.push_str("data: [DONE]\n\n");
    body
}

/// One choice delta frame (`finish_reason: null`) carrying the given delta.
fn delta_chunk(delta: &str) -> String {
    frame(&format!(
        r#"{{"id":"c1","choices":[{{"index":0,"delta":{delta},"finish_reason":null}}]}}"#
    ))
}

/// One choice delta frame carrying several tool-call entries.
fn tool_chunk(tool_calls: &str) -> String {
    delta_chunk(&format!(r#"{{"tool_calls":{tool_calls}}}"#))
}

/// The final choice chunk that closes the assistant message.
fn final_chunk() -> String {
    frame(r#"{"id":"c1","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}"#)
}

/// Drains the stream until the terminal event; panics on a stream error.
async fn collect(mut handle: StreamHandle) -> Vec<StreamEvent> {
    let mut events = Vec::new();
    while let Some(event) = handle.next().await {
        let event = event.expect("stream event");
        let finished = matches!(event, StreamEvent::Finished { .. });
        events.push(event);
        if finished {
            break;
        }
    }
    events
}

/// Tool-call fragments observed on the stream, with their transport fields.
fn fragments(events: &[StreamEvent]) -> Vec<(u32, Option<String>, Option<String>, Option<String>)> {
    events
        .iter()
        .filter_map(|event| match event {
            StreamEvent::ToolCallDelta {
                index,
                id,
                name,
                arguments,
            } => Some((*index, id.clone(), name.clone(), arguments.clone())),
            StreamEvent::Delta { .. } | StreamEvent::Finished { .. } => None,
        })
        .collect()
}

/// Feeds the events to the assembler and finishes it.
fn assemble(events: &[StreamEvent]) -> Vec<AssembledToolCall> {
    let mut assembler = ToolCallAssembler::new();
    for event in events {
        assembler.push_event(event).expect("valid fragments");
    }
    assembler.finish().expect("complete calls")
}

async fn mount_stream(server: &MockServer, sse: String) {
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(sse),
        )
        .mount(server)
        .await;
}

#[tokio::test]
async fn two_parallel_calls_in_one_delta_are_emitted_in_index_order() {
    let server = MockServer::start().await;
    // Both calls arrive in ONE delta, deliberately out of index order.
    let both = r#"[{"index":1,"id":"call_b","type":"function","function":{"name":"file_list","arguments":"{}"}},{"index":0,"id":"call_a","type":"function","function":{"name":"file_read","arguments":"{\"path\":\"a.txt\"}"}}]"#;
    mount_stream(&server, body(&[tool_chunk(both), final_chunk()])).await;

    let handle = client_for(&server).stream(&request()).await.expect("stream opens");
    assert_eq!(handle.attempts, 1);
    let events = collect(handle).await;

    let fragments = fragments(&events);
    assert_eq!(
        fragments.len(),
        2,
        "every tool_calls entry must be emitted, not only the first"
    );
    assert_eq!(fragments[0].0, 0, "fragments are in ascending index order");
    assert_eq!(fragments[0].1.as_deref(), Some("call_a"));
    assert_eq!(fragments[0].2.as_deref(), Some("file_read"));
    assert_eq!(fragments[0].3.as_deref(), Some(r#"{"path":"a.txt"}"#));
    assert_eq!(fragments[1].0, 1);
    assert_eq!(fragments[1].1.as_deref(), Some("call_b"));
    assert_eq!(fragments[1].2.as_deref(), Some("file_list"));
    assert_eq!(fragments[1].3.as_deref(), Some("{}"));
    assert!(
        events
            .iter()
            .all(|event| !matches!(event, StreamEvent::Delta { .. })),
        "a tool-only delta must not fabricate text deltas"
    );

    let calls = assemble(&events);
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].index, 0);
    assert_eq!(calls[0].arguments, serde_json::json!({"path": "a.txt"}));
    assert_eq!(calls[1].index, 1);
    assert_eq!(calls[1].arguments, serde_json::json!({}));

    let requests = server.received_requests().await.expect("recorded");
    assert_eq!(requests.len(), 1, "one streamed request, no retry");
}

#[tokio::test]
async fn fragmented_arguments_across_chunks_assemble() {
    let server = MockServer::start().await;
    let sse = body(&[
        // index 0: id + name + first argument fragment.
        tool_chunk(
            r#"[{"index":0,"id":"call_a","type":"function","function":{"name":"file_read","arguments":"{\"path\":"}}]"#,
        ),
        // index 1: a complete parallel call between the two fragments.
        tool_chunk(
            r#"[{"index":1,"id":"call_b","type":"function","function":{"name":"file_list","arguments":"{}"}}]"#,
        ),
        // index 0: the argument remainder arrives last.
        tool_chunk(r#"[{"index":0,"function":{"arguments":"\"a.txt\"}"}}]"#),
        final_chunk(),
    ]);
    mount_stream(&server, sse).await;

    let handle = client_for(&server).stream(&request()).await.expect("stream opens");
    let events = collect(handle).await;

    let fragments = fragments(&events);
    assert_eq!(fragments.len(), 3, "one event per fragment chunk");
    assert_eq!(fragments[0].0, 0);
    assert_eq!(fragments[1].0, 1);
    assert_eq!(fragments[2].0, 0, "fragment order is preserved");

    let calls = assemble(&events);
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].id, "call_a");
    assert_eq!(calls[0].name, "file_read");
    assert_eq!(
        calls[0].arguments,
        serde_json::json!({"path": "a.txt"}),
        "argument fragments are concatenated in arrival order"
    );
    assert_eq!(calls[1].id, "call_b");
    assert_eq!(calls[1].arguments, serde_json::json!({}));
}

#[tokio::test]
async fn single_call_delta_keeps_the_original_one_event_shape() {
    let server = MockServer::start().await;
    let one = r#"[{"index":0,"id":"call_a","type":"function","function":{"name":"file_list","arguments":"{}"}}]"#;
    mount_stream(&server, body(&[tool_chunk(one), final_chunk()])).await;

    let handle = client_for(&server).stream(&request()).await.expect("stream opens");
    let events = collect(handle).await;

    let fragments = fragments(&events);
    assert_eq!(
        fragments,
        vec![(
            0,
            Some("call_a".to_owned()),
            Some("file_list".to_owned()),
            Some("{}".to_owned())
        )],
        "a single-entry delta still produces exactly one unchanged ToolCallDelta"
    );
    let calls = assemble(&events);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].index, 0);
}

#[tokio::test]
async fn malformed_sse_still_is_a_named_refusal() {
    let server = MockServer::start().await;
    mount_stream(&server, "data: {not json}\n\n".to_owned()).await;

    let mut handle = client_for(&server).stream(&request()).await.expect("stream opens");
    let error = loop {
        match handle.next().await {
            Some(Err(error)) => break error,
            Some(Ok(_)) => continue,
            None => panic!("malformed SSE must be refused, not silently accepted"),
        }
    };
    assert_eq!(error.code(), "E_PROVIDER_INVALID_RESPONSE");
}

#[tokio::test]
async fn unterminated_stream_still_is_provider_incomplete() {
    let server = MockServer::start().await;
    // A valid tool fragment, but the stream never sends the [DONE] terminator.
    let unterminated = tool_chunk(
        r#"[{"index":0,"id":"call_a","type":"function","function":{"name":"file_list","arguments":"{}"}}]"#,
    );
    mount_stream(&server, unterminated).await;

    let mut handle = client_for(&server).stream(&request()).await.expect("stream opens");
    let error = loop {
        match handle.next().await {
            Some(Err(error)) => break error,
            Some(Ok(_)) => continue,
            None => panic!("an unterminated stream must be refused, not reported as success"),
        }
    };
    assert_eq!(error.code(), "E_PROVIDER_INCOMPLETE");
}
