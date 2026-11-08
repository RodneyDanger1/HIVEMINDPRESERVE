//! DeepSeek Chat Completions wire DTOs (FACT-REGISTER F1).
//!
//! Documented surface only: `POST https://api.deepseek.com/chat/completions`,
//! model ids `deepseek-flash` / `deepseek-v4-pro`, `thinking` /
//! `reasoning_effort`, function tools, JSON mode, and the usage counters.
//! Fields the docs do not document are not invented; undocumented response
//! fields are ignored by serde and unusable by callers.
//!
//! Undocumented request fields that the plan mentions are deliberately absent:
//! `frequency_penalty` / `presence_penalty` (documented as having no effect) and
//! `temperature` (documented as having no effect in thinking mode).

use hiverouter_contract::Usage;
use serde::{Deserialize, Serialize};

/// Documented base URL (FACT F1). There is no documented `/v1` prefix.
pub const DEFAULT_BASE_URL: &str = "https://api.deepseek.com";

/// Documented chat completions path.
pub const CHAT_COMPLETIONS_PATH: &str = "/chat/completions";

/// Documented maximum `max_tokens` (F1: 1–393216).
pub const MAX_OUTPUT_TOKENS: u32 = 393_216;

/// The documented model ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Model {
    /// `deepseek-flash` (served by DeepSeek-V4.1-Flash-1017).
    Flash,
    /// `deepseek-v4-pro` (DeepSeek-V4-Pro-0813).
    Pro,
}

impl Model {
    /// Every documented model, in F1 declaration order.
    pub const ALL: &'static [Model] = &[Model::Flash, Model::Pro];

    /// The model id sent on the wire.
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Model::Flash => "deepseek-flash",
            Model::Pro => "deepseek-v4-pro",
        }
    }

    /// Resolves a documented model id; unknown ids are refused, never guessed.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Model> {
        match id {
            "deepseek-flash" => Some(Model::Flash),
            "deepseek-v4-pro" => Some(Model::Pro),
            _ => None,
        }
    }
}

impl Serialize for Model {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.id())
    }
}

impl<'de> Deserialize<'de> for Model {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Model::from_id(&raw)
            .ok_or_else(|| serde::de::Error::custom(format!("undocumented model id: {raw}")))
    }
}

/// Chat message role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System directive.
    System,
    /// User turn.
    User,
    /// Assistant turn.
    Assistant,
    /// Tool result turn.
    Tool,
}

/// One message in the request/response conversation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Message role.
    pub role: Role,
    /// Text content; absent for assistant tool-call-only messages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Optional display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Tool calls requested by the assistant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Id of the tool call this message answers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Reasoner output reported by the provider (documented response field).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
}

impl ChatMessage {
    /// System message.
    #[must_use]
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: Some(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            reasoning_content: None,
        }
    }

    /// User message.
    #[must_use]
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: Some(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            reasoning_content: None,
        }
    }

    /// Assistant message.
    #[must_use]
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: Some(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            reasoning_content: None,
        }
    }

    /// Assistant message that requests one or more documented function calls.
    ///
    /// The text content is absent: the docs describe an assistant tool-call-only
    /// message as carrying no text (`content` null semantics), so the field is
    /// omitted from the serialized message rather than fabricated as `""`.
    /// Use [`Self::assistant_tool_calls_with_content`] when the model produced
    /// visible text for the same turn.
    #[must_use]
    pub fn assistant_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: Role::Assistant,
            content: None,
            name: None,
            tool_calls: Some(tool_calls),
            tool_call_id: None,
            reasoning_content: None,
        }
    }

    /// Assistant message carrying visible text plus one or more documented
    /// function calls (the normal shape when a streaming turn produced both).
    #[must_use]
    pub fn assistant_tool_calls_with_content(
        content: impl Into<String>,
        tool_calls: Vec<ToolCall>,
    ) -> Self {
        Self {
            role: Role::Assistant,
            content: Some(content.into()),
            name: None,
            tool_calls: Some(tool_calls),
            tool_call_id: None,
            reasoning_content: None,
        }
    }

    /// Sets the documented `reasoning_content` pass-back field.
    ///
    /// The docs require the previous assistant turn's `reasoning_content` to be
    /// passed back when `tools` are present and thinking mode is enabled;
    /// otherwise the request is rejected with HTTP 400. It is optional for every
    /// other message shape.
    #[must_use]
    pub fn with_reasoning_content(mut self, reasoning_content: impl Into<String>) -> Self {
        self.reasoning_content = Some(reasoning_content.into());
        self
    }

    /// Tool result message answering `tool_call_id`.
    ///
    /// Same documented shape as [`Self::tool`], with the parameter order matching
    /// the docs (`tool_call_id` then `content`) for tool-turn construction.
    #[must_use]
    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self::tool(content, tool_call_id)
    }

    /// Tool result message answering `tool_call_id`.
    #[must_use]
    pub fn tool(content: impl Into<String>, tool_call_id: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: Some(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
            reasoning_content: None,
        }
    }
}

/// The documented `thinking` switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThinkingType {
    /// Reasoning enabled.
    Enabled,
    /// Reasoning disabled.
    Disabled,
}

/// The documented `thinking` request object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Thinking {
    /// `{"type": "enabled" | "disabled"}`.
    #[serde(rename = "type")]
    pub kind: ThinkingType,
}

impl Thinking {
    /// Enabled thinking.
    #[must_use]
    pub const fn enabled() -> Self {
        Self {
            kind: ThinkingType::Enabled,
        }
    }

    /// Disabled thinking (used for forced tool calls; see F1 tools row).
    #[must_use]
    pub const fn disabled() -> Self {
        Self {
            kind: ThinkingType::Disabled,
        }
    }
}

/// Documented reasoning effort levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    /// No reasoning budget.
    None,
    /// Low reasoning budget.
    Low,
    /// High reasoning budget (documented default).
    High,
    /// Maximum reasoning budget.
    Max,
}

/// Documented tool-choice values (named-function selection is not adopted).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoice {
    /// Never call a tool.
    None,
    /// Model decides.
    Auto,
    /// Model must call one of the supplied tools.
    Required,
}

/// The documented `response_format` type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormatType {
    /// `{"type": "json_object"}`.
    JsonObject,
}

/// Documented JSON-mode request object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseFormat {
    /// `json_object`.
    #[serde(rename = "type")]
    pub kind: ResponseFormatType,
}

impl ResponseFormat {
    /// JSON-mode response format.
    #[must_use]
    pub const fn json_object() -> Self {
        Self {
            kind: ResponseFormatType::JsonObject,
        }
    }
}

/// The only documented tool type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    /// Function tool.
    Function,
}

/// A completed function call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Function name.
    pub name: String,
    /// JSON-encoded arguments string (provider contract: a string, not an object).
    pub arguments: String,
}

/// A tool call in a message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool call id.
    pub id: String,
    /// Tool type (documented: `function`).
    #[serde(rename = "type")]
    pub kind: ToolType,
    /// Invoked function.
    pub function: FunctionCall,
}

impl ToolCall {
    /// Builds one documented function tool call for an assistant message.
    ///
    /// `arguments` is the JSON-encoded argument **string** (for example
    /// `r#"{"path":"a.txt"}"#`), exactly as the model returns it and exactly as
    /// the docs require when a tool call is passed back in the conversation. It
    /// must not be replaced by a JSON object: the wire contract for
    /// `function.arguments` is a string.
    #[must_use]
    pub fn function(
        id: impl Into<String>,
        name: impl Into<String>,
        arguments: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            kind: ToolType::Function,
            function: FunctionCall {
                name: name.into(),
                arguments: arguments.into(),
            },
        }
    }
}

/// A function definition sent in a request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// Function name.
    pub name: String,
    /// Function description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema (object) for the arguments.
    pub parameters: serde_json::Value,
}

/// One tool offered to the model (`{"type":"function","function":{...}}`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolSpec {
    /// Tool type (documented: `function`).
    #[serde(rename = "type")]
    pub kind: ToolType,
    /// Function definition.
    pub function: FunctionDefinition,
}

impl ToolSpec {
    /// Wraps a function definition as a documented function tool.
    #[must_use]
    pub fn function(function: FunctionDefinition) -> Self {
        Self {
            kind: ToolType::Function,
            function,
        }
    }
}

/// A chat completions request (documented subset of F1).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChatRequest {
    /// Model id.
    pub model: Model,
    /// Conversation.
    pub messages: Vec<ChatMessage>,
    /// Stream flag (documented SSE streaming).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// Maximum output tokens (documented 1–393216).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Reasoning switch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking: Option<Thinking>,
    /// Reasoning effort (documented default: high).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,
    /// Function tools.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolSpec>>,
    /// Tool choice.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    /// JSON mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
}

impl ChatRequest {
    /// Builds a request for one model and conversation (non-streaming).
    #[must_use]
    pub fn new(model: Model, messages: Vec<ChatMessage>) -> Self {
        Self {
            model,
            messages,
            stream: None,
            max_tokens: None,
            thinking: None,
            reasoning_effort: None,
            tools: None,
            tool_choice: None,
            response_format: None,
        }
    }

    /// Marks the request as streaming or non-streaming.
    #[must_use]
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Sets `max_tokens`.
    #[must_use]
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Sets the thinking switch.
    #[must_use]
    pub fn with_thinking(mut self, thinking: Thinking) -> Self {
        self.thinking = Some(thinking);
        self
    }

    /// Sets the documented reasoning effort.
    #[must_use]
    pub fn with_reasoning_effort(mut self, effort: ReasoningEffort) -> Self {
        self.reasoning_effort = Some(effort);
        self
    }

    /// Sets the documented JSON-mode response format.
    #[must_use]
    pub fn with_response_format(mut self, format: ResponseFormat) -> Self {
        self.response_format = Some(format);
        self
    }

    /// Sets `tool_choice`.
    #[must_use]
    pub fn with_tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Sets the function tools offered to the model.
    ///
    /// With [`Self::with_tool_choice`] this is the documented `tools` array for
    /// a tool turn; pair it with [`ChatMessage::assistant_tool_calls`] /
    /// [`ChatMessage::tool_result`] messages in `messages`.
    #[must_use]
    pub fn with_tools(mut self, tools: Vec<ToolSpec>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Appends one documented tool turn (the second-turn continuation shape).
    ///
    /// Appends an assistant message carrying `tool_calls` (with no text content,
    /// as the docs describe for a tool-call-only assistant message) followed by
    /// exactly one `tool` result message per call, in call order. Use it after
    /// the model requested tools and the executor produced their results; call
    /// [`Self::with_tools`] with the same specs so the model can continue the
    /// loop.
    ///
    /// `reasoning_content` is optional: the docs require the previous assistant
    /// turn's `reasoning_content` to be passed back when `tools` are present and
    /// thinking mode is enabled (otherwise HTTP 400). Pass `Some` when the
    /// configured mode produced reasoning; `None` leaves the field absent.
    ///
    /// `tool_results` entries are `(tool_call_id, content)` and must match the
    /// calls one-for-one; the builder does not fabricate missing results or
    /// reorder them. Mismatched ids are a caller defect and are rejected by the
    /// provider API, not silently repaired here.
    #[must_use]
    pub fn with_tool_turn(
        mut self,
        tool_calls: Vec<ToolCall>,
        tool_results: Vec<(String, String)>,
        reasoning_content: Option<String>,
    ) -> Self {
        let mut assistant = ChatMessage::assistant_tool_calls(tool_calls);
        if let Some(reasoning_content) = reasoning_content {
            assistant = assistant.with_reasoning_content(reasoning_content);
        }
        self.messages.push(assistant);
        for (tool_call_id, content) in tool_results {
            self.messages
                .push(ChatMessage::tool_result(tool_call_id, content));
        }
        self
    }

    /// Pre-send validation of documented ranges/conflicts.
    ///
    /// # Errors
    /// Returns [`hiverouter_contract::Refusal::SchemaInvalid`] for an empty
    /// conversation, an out-of-range `max_tokens`, a `required` tool choice with
    /// no tools, or a forced tool call in thinking mode (documented HTTP 400).
    pub fn validate(&self) -> Result<(), crate::error::ProviderError> {
        use hiverouter_contract::Refusal;

        if self.messages.is_empty() {
            return Err(crate::error::ProviderError::new(
                Refusal::SchemaInvalid,
                "chat request has no messages",
            ));
        }
        if let Some(max_tokens) = self.max_tokens {
            if max_tokens == 0 || max_tokens > MAX_OUTPUT_TOKENS {
                return Err(crate::error::ProviderError::new(
                    Refusal::SchemaInvalid,
                    format!("max_tokens {max_tokens} is outside the documented 1..={MAX_OUTPUT_TOKENS}"),
                ));
            }
        }
        if self.tool_choice == Some(ToolChoice::Required)
            && self.tools.as_ref().map_or(true, Vec::is_empty)
        {
            return Err(crate::error::ProviderError::new(
                Refusal::SchemaInvalid,
                "tool_choice=required requires at least one tool",
            ));
        }
        let forced_tool_choice = matches!(self.tool_choice, Some(ToolChoice::Required));
        let thinking_enabled = matches!(
            self.thinking.map(|thinking| thinking.kind),
            Some(ThinkingType::Enabled)
        );
        if forced_tool_choice && thinking_enabled {
            return Err(crate::error::ProviderError::new(
                Refusal::SchemaInvalid,
                "forced tool calls are rejected in thinking mode (documented HTTP 400); \
                 disable thinking for forced tool calls",
            ));
        }
        Ok(())
    }
}

/// Provider usage counters as documented (every field optional: absent ≠ zero).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
pub struct UsageDto {
    /// Prompt tokens billed.
    #[serde(default)]
    pub prompt_tokens: Option<u64>,
    /// Completion tokens generated.
    #[serde(default)]
    pub completion_tokens: Option<u64>,
    /// Total tokens reported.
    #[serde(default)]
    pub total_tokens: Option<u64>,
    /// Prompt tokens served from the provider cache.
    #[serde(default)]
    pub prompt_cache_hit_tokens: Option<u64>,
    /// Prompt tokens not served from the provider cache.
    #[serde(default)]
    pub prompt_cache_miss_tokens: Option<u64>,
    /// Provider cache-hit counter alias documented in F1.
    #[serde(default)]
    pub cached_tokens: Option<u64>,
    /// Completion breakdown.
    #[serde(default)]
    pub completion_tokens_details: Option<CompletionTokensDetails>,
}

impl UsageDto {
    /// Converts provider counters to the frozen contract usage type; absent
    /// values stay absent (V1-V6) and never become zero.
    #[must_use]
    pub fn to_usage(&self) -> Usage {
        Usage {
            prompt_tokens: self.prompt_tokens,
            completion_tokens: self.completion_tokens,
            total_tokens: self.total_tokens,
            prompt_cache_hit_tokens: self.prompt_cache_hit_tokens,
            prompt_cache_miss_tokens: self.prompt_cache_miss_tokens,
            reasoning_tokens: self
                .completion_tokens_details
                .and_then(|details| details.reasoning_tokens),
        }
    }
}

/// Documented completion-token breakdown.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
pub struct CompletionTokensDetails {
    /// Reasoning tokens included in the completion.
    #[serde(default)]
    pub reasoning_tokens: Option<u64>,
}

/// One response choice.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Choice {
    /// Choice index.
    pub index: u32,
    /// Assistant message.
    pub message: ResponseMessage,
    /// Documented finish reason (`stop`, `length`, `tool_calls`, ...).
    #[serde(default)]
    pub finish_reason: Option<String>,
}

/// Assistant message in a response.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ResponseMessage {
    /// Role (always assistant in a response).
    pub role: Role,
    /// Text content (documented: can be null when only tool calls are returned).
    #[serde(default)]
    pub content: Option<String>,
    /// Reasoner output.
    #[serde(default)]
    pub reasoning_content: Option<String>,
    /// Tool calls requested by the model.
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// A non-streaming chat completion response.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ChatResponse {
    /// Response id.
    #[serde(default)]
    pub id: String,
    /// Served model id.
    #[serde(default)]
    pub model: String,
    /// Choices (documented: at least one).
    pub choices: Vec<Choice>,
    /// Usage counters; `None` when the provider omitted them (never coerced to 0).
    #[serde(default)]
    pub usage: Option<UsageDto>,
}

impl ChatResponse {
    /// The first choice's assistant message.
    #[must_use]
    pub fn first_message(&self) -> Option<&ResponseMessage> {
        self.choices.first().map(|choice| &choice.message)
    }

    /// Contract usage when the provider reported any counter.
    #[must_use]
    pub fn contract_usage(&self) -> Option<Usage> {
        let usage = self.usage?.to_usage();
        if usage.is_empty() {
            None
        } else {
            Some(usage)
        }
    }
}

/// One streaming chunk delta.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Delta {
    /// Text delta.
    #[serde(default)]
    pub content: Option<String>,
    /// Reasoner delta.
    #[serde(default)]
    pub reasoning_content: Option<String>,
    /// Tool-call deltas.
    #[serde(default)]
    pub tool_calls: Option<Vec<DeltaToolCall>>,
}

/// One streaming choice.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct StreamChoice {
    /// Choice index.
    #[serde(default)]
    pub index: u32,
    /// Delta payload.
    #[serde(default)]
    pub delta: Delta,
    /// Finish reason on the final chunk of the choice.
    #[serde(default)]
    pub finish_reason: Option<String>,
}

/// A tool-call delta assembled across streaming chunks.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct DeltaToolCall {
    /// Index of the tool call in the message.
    #[serde(default)]
    pub index: u32,
    /// Tool call id (first chunk only).
    #[serde(default)]
    pub id: Option<String>,
    /// Tool type (first chunk only).
    #[serde(default, rename = "type")]
    pub kind: Option<ToolType>,
    /// Function name/arguments delta.
    #[serde(default)]
    pub function: Option<DeltaFunction>,
}

/// Function name/arguments delta.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct DeltaFunction {
    /// Function name (first chunk only).
    #[serde(default)]
    pub name: Option<String>,
    /// JSON arguments fragment.
    #[serde(default)]
    pub arguments: Option<String>,
}

/// One parsed SSE chunk from a streaming completion.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct StreamChunk {
    /// Response id (repeated on each chunk).
    #[serde(default)]
    pub id: Option<String>,
    /// Served model id.
    #[serde(default)]
    pub model: Option<String>,
    /// Choices.
    #[serde(default)]
    pub choices: Vec<StreamChoice>,
    /// Usage, only if the provider reports it in the stream (not documented).
    #[serde(default)]
    pub usage: Option<UsageDto>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_ids_are_the_documented_ids() {
        assert_eq!(Model::Flash.id(), "deepseek-flash");
        assert_eq!(Model::Pro.id(), "deepseek-v4-pro");
        assert_eq!(Model::from_id("deepseek-v4-flash"), None, "legacy id refused");
        for model in Model::ALL {
            assert_eq!(Model::from_id(model.id()), Some(*model));
        }
    }

    #[test]
    fn request_serialises_documented_fields_only() {
        let request = ChatRequest::new(Model::Flash, vec![ChatMessage::user("hello")])
            .with_max_tokens(1024)
            .with_thinking(Thinking::disabled());
        let value = serde_json::to_value(&request).expect("serialises");
        assert_eq!(value["model"], "deepseek-flash");
        assert_eq!(value["max_tokens"], 1024);
        assert_eq!(value["thinking"]["type"], "disabled");
        assert!(value.get("temperature").is_none(), "undocumented field not sent");
        assert!(value.get("frequency_penalty").is_none());
        assert_eq!(value["messages"][0]["role"], "user");
    }

    #[test]
    fn validation_refuses_documented_conflicts_before_send() {
        let forced_without_tools = ChatRequest::new(Model::Flash, vec![ChatMessage::user("x")])
            .with_tool_choice(ToolChoice::Required);
        assert_eq!(
            forced_without_tools.validate().expect_err("refused").code(),
            "E_SCHEMA_INVALID"
        );

        let forced_in_thinking = ChatRequest::new(Model::Flash, vec![ChatMessage::user("x")])
            .with_thinking(Thinking::enabled())
            .with_tools(vec![ToolSpec::function(FunctionDefinition {
                name: "spawn_agent".to_owned(),
                description: None,
                parameters: serde_json::json!({"type": "object"}),
            })])
            .with_tool_choice(ToolChoice::Required);
        assert_eq!(
            forced_in_thinking.validate().expect_err("refused").code(),
            "E_SCHEMA_INVALID"
        );

        assert!(ChatRequest::new(Model::Flash, vec![ChatMessage::user("x")])
            .with_max_tokens(393_217)
            .validate()
            .is_err());
        assert!(ChatRequest::new(Model::Flash, vec![ChatMessage::user("x")])
            .with_max_tokens(393_216)
            .validate()
            .is_ok());
    }

    #[test]
    fn absent_usage_stays_absent() {
        let missing: ChatResponse =
            serde_json::from_str(r#"{"choices":[{"index":0,"message":{"role":"assistant","content":"hi"},"finish_reason":"stop"}]}"#)
                .expect("deserialises");
        assert_eq!(missing.contract_usage(), None);

        let null_usage: ChatResponse =
            serde_json::from_str(r#"{"choices":[{"index":0,"message":{"role":"assistant","content":"hi"}}],"usage":null}"#)
                .expect("deserialises");
        assert_eq!(null_usage.contract_usage(), None);

        let empty_usage: ChatResponse = serde_json::from_str(
            r#"{"choices":[{"index":0,"message":{"role":"assistant","content":"hi"}}],"usage":{}}"#,
        )
        .expect("deserialises");
        assert_eq!(empty_usage.contract_usage(), None);
    }

    #[test]
    fn usage_maps_documented_counters() {
        let response: ChatResponse = serde_json::from_str(
            r#"{"choices":[{"index":0,"message":{"role":"assistant","content":"hi"}}],
                "usage":{"prompt_tokens":100,"completion_tokens":50,"total_tokens":150,
                         "prompt_cache_hit_tokens":40,"prompt_cache_miss_tokens":60,
                         "cached_tokens":40,
                         "completion_tokens_details":{"reasoning_tokens":20}}}"#,
        )
        .expect("deserialises");
        let usage = response.contract_usage().expect("usage present");
        assert_eq!(usage.prompt_tokens, Some(100));
        assert_eq!(usage.completion_tokens, Some(50));
        assert_eq!(usage.prompt_cache_hit_tokens, Some(40));
        assert_eq!(usage.prompt_cache_miss_tokens, Some(60));
        assert_eq!(usage.reasoning_tokens, Some(20));
        assert_eq!(usage.total_tokens, Some(150));
    }

    #[test]
    fn streaming_chunk_parses_delta_and_ignores_unknown_fields() {
        let chunk: StreamChunk = serde_json::from_str(
            r#"{"id":"1","choices":[{"index":0,"delta":{"content":"He","tool_calls":[{"index":0,"id":"call_1","type":"function","function":{"name":"spawn_agent","arguments":"{\"a\":"}}]},"finish_reason":null}],"future_field":1}"#,
        )
        .expect("unknown fields ignored");
        assert_eq!(chunk.choices[0].delta.content.as_deref(), Some("He"));
        let tool_delta = &chunk.choices[0].delta.tool_calls.as_ref().expect("tool delta")[0];
        assert_eq!(tool_delta.index, 0);
        assert_eq!(tool_delta.id.as_deref(), Some("call_1"));
        assert_eq!(
            tool_delta.function.as_ref().and_then(|f| f.name.as_deref()),
            Some("spawn_agent")
        );
    }
}

/// `GET /models` response (documented account surface).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelsResponse {
    /// Always `list`.
    pub object: String,
    /// Available model offerings.
    pub data: Vec<ModelInfo>,
}

/// One model offering from `GET /models`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Wire model id.
    pub id: String,
    /// Object type, always `model`.
    pub object: String,
    /// Owner organization.
    #[serde(default)]
    pub owned_by: String,
    /// Display name, when reported.
    #[serde(default)]
    pub name: Option<String>,
    /// Total context capacity, when reported.
    #[serde(default)]
    pub context_window: Option<u64>,
    /// Maximum output tokens, when reported.
    #[serde(default)]
    pub max_output_tokens: Option<u64>,
}

/// `GET /user/balance` response (documented account surface).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceResponse {
    /// Whether the account can make calls.
    pub is_available: bool,
    /// One row per currency.
    pub balance_infos: Vec<BalanceInfo>,
}

/// One currency balance row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceInfo {
    /// Currency code (`USD`/`CNY`).
    pub currency: String,
    /// Total available balance as a decimal string.
    pub total_balance: String,
    /// Unexpired granted balance.
    #[serde(default)]
    pub granted_balance: String,
    /// Topped-up balance.
    #[serde(default)]
    pub topped_up_balance: String,
}

#[cfg(test)]
mod account_dto_tests {
    use super::*;

    #[test]
    fn models_response_keeps_documented_metadata() {
        let parsed: ModelsResponse = serde_json::from_str(
            r#"{"object":"list","data":[{"id":"deepseek-flash","object":"model","owned_by":"deepseek","name":"DeepSeek-V4.1-Flash","context_window":1048576,"max_output_tokens":393216}]}"#,
        )
        .expect("documented models shape");
        assert_eq!(parsed.object, "list");
        assert_eq!(parsed.data[0].id, "deepseek-flash");
        assert_eq!(parsed.data[0].context_window, Some(1_048_576));
        assert_eq!(parsed.data[0].max_output_tokens, Some(393_216));
    }

    #[test]
    fn balance_response_keeps_decimal_strings() {
        let parsed: BalanceResponse = serde_json::from_str(
            r#"{"is_available":true,"balance_infos":[{"currency":"USD","total_balance":"9.68","granted_balance":"0.00","topped_up_balance":"9.68"}]}"#,
        )
        .expect("documented balance shape");
        assert!(parsed.is_available);
        assert_eq!(parsed.balance_infos[0].total_balance, "9.68");
    }
}
