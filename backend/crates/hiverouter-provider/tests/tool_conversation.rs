//! Source-only serde-shape tests for the tool-turn request helpers.
//!
//! These tests pin the documented DeepSeek Chat Completions wire shape that the
//! future tool loop will send (W12 interface): an assistant message carrying
//! `tool_calls` entries (`id`, `type: "function"`, `function {name, arguments}`
//! with `arguments` as a JSON **string**), a `tool` role result message with
//! `tool_call_id` + `content`, and a request advertising the documented `tools`
//! array via [`ChatRequest::with_tools`].
//!
//! They are source-only: no wiremock, no network, no key, no paid call. Wave
//! rules forbid running builds/tests during coding waves, so they are written
//! for the single Lead gate build.

use hiverouter_provider::dto::{
    ChatMessage, ChatRequest, FunctionDefinition, Model, Role, ToolCall, ToolChoice, ToolSpec,
};

/// The documented one-item `tools` array entry used by the request test.
fn read_file_spec() -> ToolSpec {
    ToolSpec::function(FunctionDefinition {
        name: "file_read".to_owned(),
        description: Some("Read a file inside the workspace jail.".to_owned()),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"],
            "additionalProperties": false
        }),
    })
}

#[test]
fn tool_call_serialises_documented_function_shape() {
    let call = ToolCall::function("call_1", "file_read", r#"{"path":"a.txt"}"#);
    let value = serde_json::to_value(&call).expect("tool call serialises");

    assert_eq!(
        value,
        serde_json::json!({
            "id": "call_1",
            "type": "function",
            "function": {"name": "file_read", "arguments": "{\"path\":\"a.txt\"}"}
        })
    );
    assert!(
        value["function"]["arguments"].is_string(),
        "arguments must be a JSON-encoded string, never an object"
    );
}

#[test]
fn assistant_tool_calls_serialises_parallel_calls_without_content() {
    let message = ChatMessage::assistant_tool_calls(vec![
        ToolCall::function("call_1", "file_read", r#"{"path":"a.txt"}"#),
        ToolCall::function("call_2", "file_list", "{}"),
    ]);
    let value = serde_json::to_value(&message).expect("assistant message serialises");

    assert_eq!(value["role"], "assistant");
    assert!(
        value.get("content").is_none(),
        "a tool-call-only assistant message omits content (documented null semantics)"
    );
    assert!(value.get("tool_call_id").is_none());
    assert!(value.get("name").is_none());
    assert!(value.get("reasoning_content").is_none());

    let calls = value["tool_calls"].as_array().expect("tool_calls array");
    assert_eq!(calls.len(), 2, "both parallel calls are serialized");
    assert_eq!(calls[0]["id"], "call_1");
    assert_eq!(calls[0]["type"], "function");
    assert_eq!(calls[0]["function"]["name"], "file_read");
    assert_eq!(calls[0]["function"]["arguments"], "{\"path\":\"a.txt\"}");
    assert_eq!(calls[1]["id"], "call_2");
    assert_eq!(calls[1]["function"]["name"], "file_list");
    assert_eq!(calls[1]["function"]["arguments"], "{}");
}

#[test]
fn assistant_tool_calls_with_content_keeps_text_and_calls() {
    let message = ChatMessage::assistant_tool_calls_with_content(
        "I will read the file.",
        vec![ToolCall::function("call_1", "file_read", r#"{"path":"a.txt"}"#)],
    );
    let value = serde_json::to_value(&message).expect("serialises");

    assert_eq!(value["role"], "assistant");
    assert_eq!(value["content"], "I will read the file.");
    assert_eq!(
        value["tool_calls"].as_array().expect("tool_calls").len(),
        1
    );
}

#[test]
fn tool_result_serialises_documented_tool_message() {
    let value = serde_json::to_value(ChatMessage::tool_result("call_1", "file-bytes"))
        .expect("tool result serialises");

    assert_eq!(
        value,
        serde_json::json!({
            "role": "tool",
            "content": "file-bytes",
            "tool_call_id": "call_1"
        })
    );
    assert!(value.get("tool_calls").is_none());
    assert!(value.get("name").is_none());
    assert!(value.get("reasoning_content").is_none());
}

#[test]
fn tool_result_alias_matches_the_existing_tool_constructor() {
    let existing = serde_json::to_value(ChatMessage::tool("ok", "call_1"))
        .expect("existing constructor serialises");
    let alias = serde_json::to_value(ChatMessage::tool_result("call_1", "ok"))
        .expect("alias serialises");

    assert_eq!(existing, alias, "the alias must not change the wire shape");
}

#[test]
fn documented_assistant_tool_call_json_deserialises() {
    let raw = r#"{"role":"assistant","tool_calls":[
        {"id":"call_1","type":"function","function":{"name":"file_read","arguments":"{\"path\":\"a.txt\"}"}}
    ]}"#;
    let message: ChatMessage =
        serde_json::from_str(raw).expect("documented tool-call message deserialises");

    assert_eq!(message.role, Role::Assistant);
    assert!(message.content.is_none());
    let calls = message.tool_calls.expect("tool calls present");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].id, "call_1");
    assert_eq!(calls[0].function.name, "file_read");
    assert_eq!(calls[0].function.arguments, r#"{"path":"a.txt"}"#);
}

#[test]
fn full_tool_turn_request_serialises_documented_shape() {
    let messages = vec![
        ChatMessage::system("You are a HiveMind agent."),
        ChatMessage::user("read a.txt"),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "call_1",
            "file_read",
            r#"{"path":"a.txt"}"#,
        )]),
        ChatMessage::tool_result("call_1", "file-bytes"),
    ];
    let request = ChatRequest::new(Model::Flash, messages)
        .with_tools(vec![read_file_spec()])
        .with_tool_choice(ToolChoice::Auto)
        .with_max_tokens(1024);
    let value = serde_json::to_value(&request).expect("request serialises");

    assert_eq!(value["model"], "deepseek-flash");
    assert_eq!(value["max_tokens"], 1024);
    assert!(value.get("stream").is_none(), "stream stays unset by default");
    assert!(value.get("temperature").is_none(), "undocumented field not sent");
    assert!(value.get("frequency_penalty").is_none());
    assert_eq!(value["tool_choice"], "auto");

    let tools = value["tools"].as_array().expect("tools array");
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["type"], "function");
    assert_eq!(tools[0]["function"]["name"], "file_read");
    assert_eq!(
        tools[0]["function"]["description"],
        "Read a file inside the workspace jail."
    );
    assert_eq!(
        tools[0]["function"]["parameters"]["properties"]["path"]["type"],
        "string"
    );
    assert_eq!(
        tools[0]["function"]["parameters"]["additionalProperties"],
        serde_json::json!(false)
    );

    let messages = value["messages"].as_array().expect("messages array");
    let roles: Vec<&str> = messages
        .iter()
        .map(|message| message["role"].as_str().expect("role string"))
        .collect();
    assert_eq!(roles, vec!["system", "user", "assistant", "tool"]);

    assert!(messages[2].get("content").is_none());
    assert_eq!(messages[2]["tool_calls"][0]["id"], "call_1");
    assert_eq!(messages[2]["tool_calls"][0]["function"]["name"], "file_read");
    assert_eq!(messages[3]["tool_call_id"], "call_1");
    assert_eq!(messages[3]["content"], "file-bytes");
    assert!(messages[3].get("tool_calls").is_none());
}

#[test]
fn with_tools_is_chainable_and_the_last_call_wins() {
    let spec = read_file_spec();
    let request = ChatRequest::new(Model::Flash, vec![ChatMessage::user("x")])
        .with_tools(vec![])
        .with_tools(vec![spec]);
    let value = serde_json::to_value(&request).expect("serialises");

    assert_eq!(
        value["tools"].as_array().expect("tools").len(),
        1,
        "with_tools sets the offered tools; the last call wins"
    );
    assert!(
        request.validate().is_ok(),
        "a plain user turn with tools is a valid documented request"
    );
}

#[test]
fn reasoning_content_is_serialised_only_when_set() {
    let without = serde_json::to_value(ChatMessage::assistant_tool_calls_with_content(
        "ok",
        vec![ToolCall::function("call_1", "file_list", "{}")],
    ))
    .expect("serialises");
    assert!(without.get("reasoning_content").is_none());

    let with = serde_json::to_value(
        ChatMessage::assistant_tool_calls_with_content(
            "ok",
            vec![ToolCall::function("call_1", "file_list", "{}")],
        )
        .with_reasoning_content("reasoned first"),
    )
    .expect("serialises");
    assert_eq!(with["reasoning_content"], "reasoned first");
}
