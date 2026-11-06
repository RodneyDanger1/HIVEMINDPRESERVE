//! Source-only serde-shape tests for the second-tool-turn continuation path.
//!
//! They pin the documented DeepSeek Chat Completions shape built by
//! [`ChatRequest::with_tool_turn`]: an assistant message carrying `tool_calls`
//! (no text content, optional `reasoning_content` pass-back) followed by exactly
//! one `tool` result message per call, with the tool specs re-advertised on the
//! request.
//!
//! Source-only: no wiremock, no network, no key, no paid call. Wave rules forbid
//! builds/tests during coding waves; the tests are written for the single Lead
//! gate build.

use hiverouter_provider::dto::{
    ChatMessage, ChatRequest, FunctionDefinition, Model, ToolCall, ToolChoice, ToolSpec,
};

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

fn file_list_spec() -> ToolSpec {
    ToolSpec::function(FunctionDefinition {
        name: "file_list".to_owned(),
        description: Some("List a directory inside the workspace jail.".to_owned()),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"],
            "additionalProperties": false
        }),
    })
}

fn read_call() -> ToolCall {
    ToolCall::function("call_1", "file_read", r#"{"path":"a.txt"}"#)
}

fn list_call() -> ToolCall {
    ToolCall::function("call_2", "file_list", r#"{"path":"."}"#)
}

fn result_for_read() -> (String, String) {
    ("call_1".to_owned(), "file a".to_owned())
}

fn result_for_list() -> (String, String) {
    ("call_2".to_owned(), "dir: a.txt".to_owned())
}

#[test]
fn second_tool_turn_serialises_two_calls_then_two_results() {
    let request = ChatRequest::new(
        Model::Flash,
        vec![ChatMessage::system("sys"), ChatMessage::user("do it")],
    )
    .with_tools(vec![read_file_spec(), file_list_spec()])
    .with_tool_turn(
        vec![read_call(), list_call()],
        vec![result_for_read(), result_for_list()],
        None,
    );
    let value = serde_json::to_value(&request).expect("continuation request serialises");

    assert_eq!(value["model"], "deepseek-flash");
    let messages = value["messages"].as_array().expect("messages array");
    let roles: Vec<&str> = messages
        .iter()
        .map(|message| message["role"].as_str().expect("role string"))
        .collect();
    assert_eq!(
        roles,
        vec!["system", "user", "assistant", "tool", "tool"],
        "the tip appends one assistant tool-call message then one tool result per call"
    );

    let assistant = &messages[2];
    assert_eq!(assistant["role"], "assistant");
    assert!(
        assistant.get("content").is_none(),
        "a tool-call-only assistant message has no content key"
    );
    assert!(assistant.get("reasoning_content").is_none());
    assert!(assistant.get("tool_call_id").is_none());
    let calls = assistant["tool_calls"].as_array().expect("tool_calls array");
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0]["id"], "call_1");
    assert_eq!(calls[0]["type"], "function");
    assert_eq!(calls[0]["function"]["name"], "file_read");
    assert_eq!(calls[0]["function"]["arguments"], "{\"path\":\"a.txt\"}");
    assert_eq!(calls[1]["id"], "call_2");
    assert_eq!(calls[1]["function"]["name"], "file_list");

    let first_result = &messages[3];
    assert_eq!(first_result["role"], "tool");
    assert_eq!(first_result["tool_call_id"], "call_1");
    assert_eq!(first_result["content"], "file a");
    assert!(first_result.get("tool_calls").is_none());
    let second_result = &messages[4];
    assert_eq!(second_result["role"], "tool");
    assert_eq!(second_result["tool_call_id"], "call_2");
    assert_eq!(second_result["content"], "dir: a.txt");

    let tools = value["tools"].as_array().expect("tools array");
    assert_eq!(tools.len(), 2, "tool specs are re-advertised");
    assert_eq!(tools[0]["type"], "function");
    assert_eq!(tools[0]["function"]["name"], "file_read");
    assert_eq!(tools[1]["function"]["name"], "file_list");
}

#[test]
fn no_content_assistant_message_is_kept_in_the_continuation() {
    let request = ChatRequest::new(Model::Flash, vec![ChatMessage::user("go")])
        .with_tools(vec![read_file_spec()])
        .with_tool_turn(vec![read_call()], vec![result_for_read()], None);
    let value = serde_json::to_value(&request).expect("serialises");

    let assistant = &value["messages"].as_array().expect("messages")[1];
    assert_eq!(assistant["role"], "assistant");
    assert!(
        assistant.get("content").is_none(),
        "no content is fabricated for the assistant tool-call turn"
    );
    assert!(assistant["tool_calls"].is_array());
}

#[test]
fn reasoning_pass_back_is_attached_when_configured() {
    let request = ChatRequest::new(Model::Flash, vec![ChatMessage::user("go")])
        .with_tools(vec![read_file_spec()])
        .with_tool_turn(
            vec![read_call()],
            vec![result_for_read()],
            Some("the model reasoned about a.txt".to_owned()),
        );
    let value = serde_json::to_value(&request).expect("serialises");

    let assistant = &value["messages"].as_array().expect("messages")[1];
    assert_eq!(
        assistant["reasoning_content"],
        "the model reasoned about a.txt",
        "reasoning_content is passed back when configured"
    );
    assert!(assistant.get("content").is_none());
}

#[test]
fn prior_messages_serialise_exactly_as_before_the_helper() {
    let system = ChatMessage::system("sys");
    let user = ChatMessage::user("do it");
    let expected_system = serde_json::to_value(&system).expect("system serialises");
    let expected_user = serde_json::to_value(&user).expect("user serialises");

    let request = ChatRequest::new(Model::Flash, vec![system, user])
        .with_tool_turn(vec![read_call()], vec![result_for_read()], None);
    let value = serde_json::to_value(&request).expect("serialises");
    let messages = value["messages"].as_array().expect("messages");

    assert_eq!(
        messages[0], expected_system,
        "existing system-message serialization is preserved byte-for-byte"
    );
    assert_eq!(
        messages[1], expected_user,
        "existing user-message serialization is preserved byte-for-byte"
    );
}

#[test]
fn a_second_continuation_appends_another_ordered_turn() {
    let request = ChatRequest::new(Model::Flash, vec![ChatMessage::user("go")])
        .with_tools(vec![read_file_spec(), file_list_spec()])
        .with_tool_choice(ToolChoice::Auto)
        .with_tool_turn(vec![read_call()], vec![result_for_read()], None)
        .with_tool_turn(
            vec![list_call()],
            vec![result_for_list()],
            Some("second reasoning".to_owned()),
        );
    let value = serde_json::to_value(&request).expect("serialises");

    let messages = value["messages"].as_array().expect("messages");
    let roles: Vec<&str> = messages
        .iter()
        .map(|message| message["role"].as_str().expect("role string"))
        .collect();
    assert_eq!(
        roles,
        vec!["user", "assistant", "tool", "assistant", "tool"],
        "successive turns are appended in loop order"
    );
    assert_eq!(messages[1]["tool_calls"][0]["id"], "call_1");
    assert_eq!(messages[2]["tool_call_id"], "call_1");
    assert_eq!(
        messages[3]["reasoning_content"],
        "second reasoning",
        "the newest assistant turn carries its own reasoning pass-back"
    );
    assert_eq!(messages[3]["tool_calls"][0]["id"], "call_2");
    assert_eq!(messages[4]["tool_call_id"], "call_2");
    assert_eq!(value["tool_choice"], "auto");
}

#[test]
fn continuation_on_an_empty_prior_conversation_keeps_the_documented_order() {
    let request = ChatRequest::new(Model::Flash, vec![]).with_tool_turn(
        vec![read_call(), list_call()],
        vec![result_for_read(), result_for_list()],
        None,
    );
    let value = serde_json::to_value(&request).expect("serialises");

    let messages = value["messages"].as_array().expect("messages");
    assert_eq!(messages.len(), 3, "assistant + one tool message per call");
    assert_eq!(messages[0]["role"], "assistant");
    assert_eq!(messages[1]["role"], "tool");
    assert_eq!(messages[2]["role"], "tool");
    assert_eq!(messages[1]["tool_call_id"], "call_1");
    assert_eq!(messages[2]["tool_call_id"], "call_2");
}
