//! Frozen server -> client events and their payloads
//! (`docs/contracts/UI-COMMANDS-v1.md` §3).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::command::CommandName;
use crate::envelope::{Envelope, FrameKind};
use crate::ids::{ApprovalId, RunId, TaskId};
use crate::refusal::Refusal;
use crate::state::AgentState;
use crate::usage::Usage;

/// Event names of the frozen vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventName {
    /// Command accepted.
    Ack,
    /// Named refusal / error.
    Error,
    /// Server hello after a websocket connect.
    ServerHello,
    /// Client must resnapshot.
    ResyncRequired,
    /// A run was created.
    RunCreated,
    /// Run status changed.
    RunStatus,
    /// One assistant reply delta.
    ReplyDelta,
    /// Assistant reply finished.
    ReplyDone,
    /// Node spawned.
    NodeSpawn,
    /// Node dragged (position update).
    NodeDrag,
    /// Node rewired.
    NodeRewire,
    /// Node state changed.
    NodeState,
    /// Run metrics update.
    RunMetrics,
    /// Tool approval requested.
    ApprovalRequest,
    /// Tool approval resolved.
    ApprovalResolved,
    /// Provider/model/key status (never the key itself).
    SettingsStatusResult,
    /// Workspace file-tree listing.
    FileTreeResult,
    /// One workspace file's content.
    FileContent,
    /// History snapshot for the scrubber.
    HistorySnapshot,
    /// Keepalive request.
    Ping,
    /// Keepalive reply.
    Pong,
    /// A model-issued tool call started (1.2 additive; no execution implied).
    ToolCall,
    /// A tool call finished with its outcome (1.2 additive).
    ToolResult,
}

impl EventName {
    /// Every event name (declaration order).
    pub const ALL: &'static [EventName] = &[
        EventName::Ack,
        EventName::Error,
        EventName::ServerHello,
        EventName::ResyncRequired,
        EventName::RunCreated,
        EventName::RunStatus,
        EventName::ReplyDelta,
        EventName::ReplyDone,
        EventName::NodeSpawn,
        EventName::NodeDrag,
        EventName::NodeRewire,
        EventName::NodeState,
        EventName::RunMetrics,
        EventName::ApprovalRequest,
        EventName::ApprovalResolved,
        EventName::SettingsStatusResult,
        EventName::FileTreeResult,
        EventName::FileContent,
        EventName::HistorySnapshot,
        EventName::Ping,
        EventName::Pong,
        EventName::ToolCall,
        EventName::ToolResult,
    ];

    /// Stable wire name (SCREAMING_SNAKE_CASE).
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            EventName::Ack => "ACK",
            EventName::Error => "ERROR",
            EventName::ServerHello => "SERVER_HELLO",
            EventName::ResyncRequired => "RESYNC_REQUIRED",
            EventName::RunCreated => "RUN_CREATED",
            EventName::RunStatus => "RUN_STATUS",
            EventName::ReplyDelta => "REPLY_DELTA",
            EventName::ReplyDone => "REPLY_DONE",
            EventName::NodeSpawn => "NODE_SPAWN",
            EventName::NodeDrag => "NODE_DRAG",
            EventName::NodeRewire => "NODE_REWIRE",
            EventName::NodeState => "NODE_STATE",
            EventName::RunMetrics => "RUN_METRICS",
            EventName::ApprovalRequest => "APPROVAL_REQUEST",
            EventName::ApprovalResolved => "APPROVAL_RESOLVED",
            EventName::SettingsStatusResult => "SETTINGS_STATUS_RESULT",
            EventName::FileTreeResult => "FILE_TREE_RESULT",
            EventName::FileContent => "FILE_CONTENT",
            EventName::HistorySnapshot => "HISTORY_SNAPSHOT",
            EventName::Ping => "PING",
            EventName::Pong => "PONG",
            EventName::ToolCall => "TOOL_CALL",
            EventName::ToolResult => "TOOL_RESULT",
        }
    }

    /// Looks an event up by wire name.
    #[must_use]
    pub fn from_name(name: &str) -> Option<EventName> {
        EventName::ALL
            .iter()
            .copied()
            .find(|candidate| candidate.as_str() == name)
    }
}

/// `ACK` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Ack {
    /// Command that was accepted.
    pub command: CommandName,
    /// Correlation id shared with the request frame.
    pub correlation_id: Uuid,
}

/// `ERROR` payload — a typed refusal, never a silent drop.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ErrorPayload {
    /// Stable refusal code (`E_*`).
    pub code: String,
    /// Human-readable cause.
    pub message: String,
    /// Actionable remedy shown by the UI.
    pub how_to_fix: String,
    /// Whether retrying the same request may succeed.
    pub retryable: bool,
}

impl ErrorPayload {
    /// Builds an `ERROR` payload from a refusal.
    #[must_use]
    pub fn from_refusal(refusal: &Refusal) -> Self {
        Self {
            code: refusal.code().to_owned(),
            message: refusal.message().to_owned(),
            how_to_fix: refusal.how_to_fix().to_owned(),
            retryable: refusal.retryable(),
        }
    }
}

/// `SERVER_HELLO` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ServerHello {
    /// Last server sequence number the client had seen; `None` for a fresh client.
    pub last_seq: Option<u64>,
    /// Contract schema id.
    pub schema: String,
}

/// `RESYNC_REQUIRED` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ResyncRequiredPayload {
    /// First sequence number the client needs before applying diffs.
    pub expected_seq: u64,
    /// Last sequence number the client reported.
    pub last_seq: u64,
}

/// `RUN_CREATED` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RunCreated {
    /// New run id.
    pub run_id: RunId,
    /// Initial run status (`queued`).
    pub status: AgentState,
}

/// `RUN_STATUS` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RunStatusPayload {
    /// Run id.
    pub run_id: RunId,
    /// New run status.
    pub status: AgentState,
    /// Refusal code when the run failed; absent otherwise.
    #[serde(default)]
    pub error_code: Option<String>,
}

/// `REPLY_DELTA` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReplyDelta {
    /// Run the delta belongs to.
    pub run_id: RunId,
    /// Incremental assistant text.
    pub text_delta: String,
}

/// `REPLY_DONE` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReplyDone {
    /// Run the reply belongs to.
    pub run_id: RunId,
    /// Provider usage for the reply; absent counters stay absent.
    pub usage: Usage,
}

/// Vocabulary of [`NodeEvent::kind`] (mirror of the web `NODE_KINDS` in
/// `web/src/api/contract.ts`).
///
/// A producer populates `kind` only from the task's authoritative tier. A task
/// whose tier is not threaded yet must omit the field rather than infer one from
/// its free-form role string (`planner`, `worker`, ...): an invented kind is
/// worse than an absent one. `error` is a client-only broken-route state and is
/// never produced by the server.
pub const NODE_KIND_VOCABULARY: &[&str] = &["hyperagent", "superagent", "worker", "tool"];

/// `NODE_SPAWN` / `NODE_DRAG` / `NODE_REWIRE` / `NODE_STATE` payload.
///
/// Frozen fields: `task_id`, `state`. The topology fields `rev`, `kind` and
/// `parent_task_id` are additive and optional on the wire: they are omitted when
/// absent, so a pre-1.3 producer and a pre-1.3 consumer keep working unchanged.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NodeEvent {
    /// Task id.
    pub task_id: TaskId,
    /// New node state.
    pub state: AgentState,
    /// Server node revision this frame describes — the same per-node value
    /// `REWIRE_TASK.expected_rev` is compared against.
    ///
    /// Absent stays absent. This is never the client's event counter and is
    /// never inferred from another revision: a producer that does not own the
    /// graph node revision omits it instead of guessing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<u64>,
    /// Topology kind, one of [`NODE_KIND_VOCABULARY`].
    ///
    /// Populated only from the task's authoritative tier; omitted when the tier
    /// is unknown. Consumers ignore an unknown value rather than coercing it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Current parent task id (`None` for the run root), carried so a re-parent
    /// moves the canvas edge in the same frame that reports the new state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_task_id: Option<TaskId>,
}

/// `RUN_METRICS` payload.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RunMetrics {
    /// Super-agent count.
    pub superagents: u32,
    /// Worker count.
    pub workers: u32,
    /// Percentage of green nodes (0..=100).
    pub green_pct: f64,
    /// Change in green percentage since the previous metrics frame.
    pub delta: f64,
}

/// Hard cap on an [`ApprovalRequest::args_preview`] length, measured in UTF-16
/// code units — the same unit as JavaScript's `String.length`, so the client
/// mirror (`MAX_ARGS_PREVIEW_CHARS` in `web/src/api/contract.ts`) enforces the
/// identical bound.
///
/// The preview is display-only. [`ApprovalRequest::args_sha256`] stays the
/// authoritative binding for the approval, and argument content above this cap
/// must never reach the wire.
pub const MAX_ARGS_PREVIEW_CHARS: usize = 512;

/// Placeholder rendered by [`redact_args_preview`] in place of a secret value.
pub const ARGS_PREVIEW_REDACTED: &str = "[redacted]";

/// Placeholder rendered where a preview was cut by a size bound.
pub const ARGS_PREVIEW_TRUNCATED: &str = "[truncated]";

/// Placeholder rendered when an argument value cannot be rendered at all.
pub const ARGS_PREVIEW_UNAVAILABLE: &str = "[unavailable]";

/// Per-string cap inside a preview: one long argument cannot spend the whole
/// budget.
const PREVIEW_STRING_CHARS: usize = 160;
/// Per-object key cap inside a preview.
const PREVIEW_OBJECT_KEYS: usize = 12;
/// Per-array item cap inside a preview.
const PREVIEW_ARRAY_ITEMS: usize = 8;
/// Nesting cap inside a preview.
const PREVIEW_MAX_DEPTH: usize = 4;

/// Lowercased key fragments that mark a value as a secret. Matching is by
/// containment, so `GITHUB_TOKEN`, `api_key` and `client_secret` are all caught;
/// the price is fail-closed over-redaction (e.g. `max_tokens`), which is the
/// safe direction for a preview.
const PREVIEW_SECRET_KEY_FRAGMENTS: &[&str] = &[
    "apikey",
    "api_key",
    "authorization",
    "bearer",
    "cookie",
    "credential",
    "nonce",
    "passwd",
    "password",
    "private_key",
    "privatekey",
    "secret",
    "session",
    "token",
];

/// Minimum length before a well-known token prefix is treated as a secret, so a
/// short ordinary word is not redacted just because it starts like a key.
const PREVIEW_SECRET_PREFIX_MIN_LEN: usize = 16;

/// Well-known credential prefixes (compared lowercased).
const PREVIEW_SECRET_PREFIXES: &[&str] = &[
    "aiza",
    "akia",
    "ghp_",
    "gho_",
    "ghs_",
    "github_pat_",
    "sk-",
    "xoxb-",
    "xoxp-",
];

/// Renders a bounded, redacted preview of one tool-argument value for
/// [`ApprovalRequest::args_preview`].
///
/// Guarantees, in order:
/// 1. **Bounded** — the result is never longer than [`MAX_ARGS_PREVIEW_CHARS`]
///    UTF-16 code units; a cut result ends with `…`.
/// 2. **Redacted** — the value of any key whose lowercased name contains one of
///    [`PREVIEW_SECRET_KEY_FRAGMENTS`], and any string shaped like a bearer /
///    well-known API token or a JWT, becomes [`ARGS_PREVIEW_REDACTED`].
/// 3. **Small** — per-string, per-object, per-array and depth caps keep one huge
///    argument from dominating the frame.
/// 4. **Deterministic** — no clock, randomness or locale involvement, so the
///    same arguments always render the same preview.
///
/// This is a presentation helper only. It is never a substitute for the digest:
/// a consumer that needs certainty compares [`ApprovalRequest::args_sha256`].
#[must_use]
pub fn redact_args_preview(args: &serde_json::Value) -> String {
    let projection = redact_preview_value(args, 0);
    let rendered = serde_json::to_string(&projection)
        .unwrap_or_else(|_| format!("\"{ARGS_PREVIEW_UNAVAILABLE}\""));
    truncate_utf16(&rendered, MAX_ARGS_PREVIEW_CHARS)
}

fn redact_preview_value(value: &serde_json::Value, depth: usize) -> serde_json::Value {
    use serde_json::Value;
    if depth >= PREVIEW_MAX_DEPTH {
        return Value::String(ARGS_PREVIEW_TRUNCATED.to_owned());
    }
    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (index, (key, nested)) in map.iter().enumerate() {
                if index >= PREVIEW_OBJECT_KEYS {
                    out.insert(
                        "…".to_owned(),
                        Value::String(ARGS_PREVIEW_TRUNCATED.to_owned()),
                    );
                    break;
                }
                if is_secret_key(key) {
                    out.insert(key.clone(), Value::String(ARGS_PREVIEW_REDACTED.to_owned()));
                    continue;
                }
                out.insert(key.clone(), redact_preview_value(nested, depth + 1));
            }
            Value::Object(out)
        }
        Value::Array(items) => {
            let mut out = Vec::new();
            for (index, item) in items.iter().enumerate() {
                if index >= PREVIEW_ARRAY_ITEMS {
                    out.push(Value::String(ARGS_PREVIEW_TRUNCATED.to_owned()));
                    break;
                }
                out.push(redact_preview_value(item, depth + 1));
            }
            Value::Array(out)
        }
        Value::String(text) => {
            if looks_like_secret(text) {
                Value::String(ARGS_PREVIEW_REDACTED.to_owned())
            } else {
                Value::String(truncate_utf16(text, PREVIEW_STRING_CHARS))
            }
        }
        other => other.clone(),
    }
}

fn is_secret_key(key: &str) -> bool {
    let lowered = key.to_ascii_lowercase();
    PREVIEW_SECRET_KEY_FRAGMENTS
        .iter()
        .any(|fragment| lowered.contains(fragment))
}

fn looks_like_secret(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.len() < PREVIEW_SECRET_PREFIX_MIN_LEN {
        return false;
    }
    let lowered = trimmed.to_ascii_lowercase();
    if lowered.starts_with("bearer ") || lowered.starts_with("basic ") {
        return true;
    }
    if PREVIEW_SECRET_PREFIXES
        .iter()
        .any(|prefix| lowered.starts_with(prefix))
    {
        return true;
    }
    // JWT shape: exactly three dot-separated segments whose header is the
    // base64url encoding of `{"` (`eyJ`).
    let segments: Vec<&str> = trimmed.split('.').collect();
    segments.len() == 3
        && segments.iter().all(|segment| !segment.is_empty())
        && trimmed.starts_with("eyJ")
}

/// Truncates `text` to at most `max_units` UTF-16 code units on a `char`
/// boundary, marking a cut with `…`.
fn truncate_utf16(text: &str, max_units: usize) -> String {
    if max_units == 0 {
        return String::new();
    }
    if text.encode_utf16().count() <= max_units {
        return text.to_owned();
    }
    // Reserve one unit for the ellipsis so the result never overshoots.
    let budget = max_units - 1;
    let mut out = String::new();
    let mut used = 0_usize;
    for character in text.chars() {
        let width = character.len_utf16();
        if used + width > budget {
            break;
        }
        out.push(character);
        used += width;
    }
    out.push('…');
    out
}

/// `APPROVAL_REQUEST` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ApprovalRequest {
    /// Approval id.
    pub approval_id: ApprovalId,
    /// Tool name awaiting approval.
    pub tool: String,
    /// Pending decision; absent while the request is open.
    #[serde(default)]
    pub decision: Option<String>,
    /// Single-use nonce required by `APPROVAL_RESOLVE` (1.2 additive).
    ///
    /// Optional for wire compatibility: pre-1.2 frames carry no nonce; a frame
    /// without it cannot be resolved from the UI (the existing honest gap).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    /// Run the tool call belongs to (1.2 additive).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<RunId>,
    /// Task that issued the tool call (1.2 additive).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<TaskId>,
    /// Provider tool-call id this approval gates (1.2 additive).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// SHA-256 of the canonical tool arguments the nonce is bound to (1.2 additive).
    ///
    /// Authoritative: a preview is never a substitute for this digest.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args_sha256: Option<String>,
    /// Bounded, redacted, display-only preview of the gated tool arguments
    /// (additive; at most [`MAX_ARGS_PREVIEW_CHARS`] UTF-16 code units).
    ///
    /// Build it with [`redact_args_preview`], which enforces the redaction and
    /// size rules. Absent stays absent: a producer that does not hold the
    /// arguments (the tools approval row persists only the digest) omits the
    /// field rather than reconstructing it, so a post-hello replay of a
    /// persisted row carries no preview. [`Self::args_sha256`] remains the
    /// authoritative binding on every path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(max = 512))]
    pub args_preview: Option<String>,
    /// Approval expiry (TTL) so the UI can show a countdown (1.2 additive).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<crate::timestamp::Timestamp>,
}

/// Tool-call outcome vocabulary (1.2 additive; wire values are snake_case).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ToolOutcome {
    /// The tool executed and produced a successful result.
    Ok,
    /// The tool refused by name (validation/jail/limit).
    Refused,
    /// The operator denied the approval request; nothing executed.
    Denied,
    /// The call exceeded its time bound; no side effect is claimed.
    Timeout,
    /// The run/task was cancelled before or during the call.
    Cancelled,
}

impl ToolOutcome {
    /// Stable wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            ToolOutcome::Ok => "ok",
            ToolOutcome::Refused => "refused",
            ToolOutcome::Denied => "denied",
            ToolOutcome::Timeout => "timeout",
            ToolOutcome::Cancelled => "cancelled",
        }
    }

    /// Parses a wire label.
    #[must_use]
    pub fn from_str(label: &str) -> Option<ToolOutcome> {
        match label {
            "ok" => Some(ToolOutcome::Ok),
            "refused" => Some(ToolOutcome::Refused),
            "denied" => Some(ToolOutcome::Denied),
            "timeout" => Some(ToolOutcome::Timeout),
            "cancelled" => Some(ToolOutcome::Cancelled),
            _ => None,
        }
    }
}

/// `TOOL_CALL` payload (1.2 additive): identity of one model-issued tool call.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ToolCallPayload {
    /// Run the tool call belongs to.
    pub run_id: RunId,
    /// Task that issued the tool call.
    pub task_id: TaskId,
    /// Provider tool-call id (stable across the call and its result).
    pub tool_call_id: String,
    /// Tool id from the frozen tool registry (`file_read`, `shell_exec`, ...).
    pub tool: String,
    /// SHA-256 (lowercase hex) of the canonical JSON arguments.
    pub args_sha256: String,
    /// Loop turn (1-based) the call belongs to.
    pub turn: u32,
}

/// `TOOL_RESULT` payload (1.2 additive): outcome summary, never the tool body.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ToolResultPayload {
    /// Run the tool call belongs to.
    pub run_id: RunId,
    /// Task that issued the tool call.
    pub task_id: TaskId,
    /// Provider tool-call id this result answers.
    pub tool_call_id: String,
    /// Outcome vocabulary value.
    pub outcome: ToolOutcome,
    /// Named refusal code when the outcome is `refused` (and optionally others).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Result size in bytes when known; absent stays absent (never fabricated 0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: Option<u64>,
    /// SHA-256 of the result body when known (redaction-safe fingerprint).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

/// `APPROVAL_RESOLVED` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ApprovalResolved {
    /// Approval id.
    pub approval_id: ApprovalId,
    /// Tool name.
    pub tool: String,
    /// Resolved decision (`approve` / `deny`).
    pub decision: String,
}

/// `SETTINGS_STATUS_RESULT` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SettingsStatusResult {
    /// Provider id (`deepseek`).
    pub provider: String,
    /// Configured model id.
    pub model: String,
    /// Whether an API key is configured; the key itself is never sent.
    pub key_present: bool,
    /// Planner thinking mode (`enabled`/`disabled`), when configured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    /// Planner reasoning effort (`none`/`low`/`high`/`max`), when configured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
}

/// One entry of a workspace file tree.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FileTreeEntry {
    /// Workspace-relative path.
    pub path: String,
    /// Final path component.
    pub name: String,
    /// Whether the entry is a directory.
    pub is_dir: bool,
}

/// `FILE_TREE_RESULT` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FileTreeResult {
    /// Workspace root the tree was listed from.
    pub root: String,
    /// Directory entries (one level).
    pub entries: Vec<FileTreeEntry>,
}

/// `FILE_CONTENT` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FileContent {
    /// Workspace-relative path of the file.
    pub path: String,
    /// SHA-256 of the file bytes (lowercase hex).
    pub sha256: String,
    /// File size in bytes.
    pub bytes: u64,
    /// UTF-8 file text (the transport refuses non-UTF-8 files).
    pub text: String,
}

/// `HISTORY_SNAPSHOT` payload.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HistorySnapshot {
    /// Run id.
    pub run_id: RunId,
    /// Revision number (0-based, monotonic).
    pub revision: u64,
    /// Versioned graph snapshot body (schema owned by the Phase 9 snapshot store).
    pub graph: serde_json::Value,
}

/// Payload for events that carry no fields (`PING` / `PONG`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EmptyPayload {}

/// A typed event.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Event {
    /// See [`Ack`].
    Ack(Ack),
    /// See [`ErrorPayload`].
    Error(ErrorPayload),
    /// See [`ServerHello`].
    ServerHello(ServerHello),
    /// See [`ResyncRequiredPayload`].
    ResyncRequired(ResyncRequiredPayload),
    /// See [`RunCreated`].
    RunCreated(RunCreated),
    /// See [`RunStatusPayload`].
    RunStatus(RunStatusPayload),
    /// See [`ReplyDelta`].
    ReplyDelta(ReplyDelta),
    /// See [`ReplyDone`].
    ReplyDone(ReplyDone),
    /// See [`NodeEvent`].
    NodeSpawn(NodeEvent),
    /// See [`NodeEvent`].
    NodeDrag(NodeEvent),
    /// See [`NodeEvent`].
    NodeRewire(NodeEvent),
    /// See [`NodeEvent`].
    NodeState(NodeEvent),
    /// See [`RunMetrics`].
    RunMetrics(RunMetrics),
    /// See [`ApprovalRequest`].
    ApprovalRequest(ApprovalRequest),
    /// See [`ApprovalResolved`].
    ApprovalResolved(ApprovalResolved),
    /// See [`SettingsStatusResult`].
    SettingsStatusResult(SettingsStatusResult),
    /// See [`FileTreeResult`].
    FileTreeResult(FileTreeResult),
    /// See [`FileContent`].
    FileContent(FileContent),
    /// See [`HistorySnapshot`].
    HistorySnapshot(HistorySnapshot),
    /// Keepalive request.
    Ping(EmptyPayload),
    /// Keepalive reply.
    Pong(EmptyPayload),
    /// See [`ToolCallPayload`].
    ToolCall(ToolCallPayload),
    /// See [`ToolResultPayload`].
    ToolResult(ToolResultPayload),
}

impl Event {
    /// Event name of this value.
    #[must_use]
    pub fn name(&self) -> EventName {
        match self {
            Event::Ack(_) => EventName::Ack,
            Event::Error(_) => EventName::Error,
            Event::ServerHello(_) => EventName::ServerHello,
            Event::ResyncRequired(_) => EventName::ResyncRequired,
            Event::RunCreated(_) => EventName::RunCreated,
            Event::RunStatus(_) => EventName::RunStatus,
            Event::ReplyDelta(_) => EventName::ReplyDelta,
            Event::ReplyDone(_) => EventName::ReplyDone,
            Event::NodeSpawn(_) => EventName::NodeSpawn,
            Event::NodeDrag(_) => EventName::NodeDrag,
            Event::NodeRewire(_) => EventName::NodeRewire,
            Event::NodeState(_) => EventName::NodeState,
            Event::RunMetrics(_) => EventName::RunMetrics,
            Event::ApprovalRequest(_) => EventName::ApprovalRequest,
            Event::ApprovalResolved(_) => EventName::ApprovalResolved,
            Event::SettingsStatusResult(_) => EventName::SettingsStatusResult,
            Event::FileTreeResult(_) => EventName::FileTreeResult,
            Event::FileContent(_) => EventName::FileContent,
            Event::HistorySnapshot(_) => EventName::HistorySnapshot,
            Event::Ping(_) => EventName::Ping,
            Event::Pong(_) => EventName::Pong,
            Event::ToolCall(_) => EventName::ToolCall,
            Event::ToolResult(_) => EventName::ToolResult,
        }
    }

    /// Parses and validates a typed event from an event envelope.
    ///
    /// # Errors
    /// `E_SCHEMA_INVALID` on name, direction or payload mismatch.
    pub fn from_envelope(envelope: &Envelope) -> Result<Self, Refusal> {
        envelope.validate_basic()?;
        if envelope.frame_type != FrameKind::Event {
            return Err(Refusal::SchemaInvalid);
        }
        let name = envelope.event_name()?;
        let event = match name {
            EventName::Ack => Event::Ack(envelope.decode_payload()?),
            EventName::Error => Event::Error(envelope.decode_payload()?),
            EventName::ServerHello => Event::ServerHello(envelope.decode_payload()?),
            EventName::ResyncRequired => Event::ResyncRequired(envelope.decode_payload()?),
            EventName::RunCreated => Event::RunCreated(envelope.decode_payload()?),
            EventName::RunStatus => Event::RunStatus(envelope.decode_payload()?),
            EventName::ReplyDelta => Event::ReplyDelta(envelope.decode_payload()?),
            EventName::ReplyDone => Event::ReplyDone(envelope.decode_payload()?),
            EventName::NodeSpawn => Event::NodeSpawn(envelope.decode_payload()?),
            EventName::NodeDrag => Event::NodeDrag(envelope.decode_payload()?),
            EventName::NodeRewire => Event::NodeRewire(envelope.decode_payload()?),
            EventName::NodeState => Event::NodeState(envelope.decode_payload()?),
            EventName::RunMetrics => Event::RunMetrics(envelope.decode_payload()?),
            EventName::ApprovalRequest => Event::ApprovalRequest(envelope.decode_payload()?),
            EventName::ApprovalResolved => Event::ApprovalResolved(envelope.decode_payload()?),
            EventName::SettingsStatusResult => {
                Event::SettingsStatusResult(envelope.decode_payload()?)
            }
            EventName::FileTreeResult => Event::FileTreeResult(envelope.decode_payload()?),
            EventName::FileContent => Event::FileContent(envelope.decode_payload()?),
            EventName::HistorySnapshot => Event::HistorySnapshot(envelope.decode_payload()?),
            EventName::Ping => Event::Ping(envelope.decode_payload()?),
            EventName::Pong => Event::Pong(envelope.decode_payload()?),
            EventName::ToolCall => Event::ToolCall(envelope.decode_payload()?),
            EventName::ToolResult => Event::ToolResult(envelope.decode_payload()?),
        };
        Ok(event)
    }

    /// Serializes the typed payload.
    ///
    /// # Errors
    /// `E_SCHEMA_INVALID` when the payload cannot be serialized.
    pub fn payload_value(&self) -> Result<serde_json::Value, Refusal> {
        let value = match self {
            Event::Ack(payload) => serde_json::to_value(payload),
            Event::Error(payload) => serde_json::to_value(payload),
            Event::ServerHello(payload) => serde_json::to_value(payload),
            Event::ResyncRequired(payload) => serde_json::to_value(payload),
            Event::RunCreated(payload) => serde_json::to_value(payload),
            Event::RunStatus(payload) => serde_json::to_value(payload),
            Event::ReplyDelta(payload) => serde_json::to_value(payload),
            Event::ReplyDone(payload) => serde_json::to_value(payload),
            Event::NodeSpawn(payload) => serde_json::to_value(payload),
            Event::NodeDrag(payload) => serde_json::to_value(payload),
            Event::NodeRewire(payload) => serde_json::to_value(payload),
            Event::NodeState(payload) => serde_json::to_value(payload),
            Event::RunMetrics(payload) => serde_json::to_value(payload),
            Event::ApprovalRequest(payload) => serde_json::to_value(payload),
            Event::ApprovalResolved(payload) => serde_json::to_value(payload),
            Event::SettingsStatusResult(payload) => serde_json::to_value(payload),
            Event::FileTreeResult(payload) => serde_json::to_value(payload),
            Event::FileContent(payload) => serde_json::to_value(payload),
            Event::HistorySnapshot(payload) => serde_json::to_value(payload),
            Event::Ping(payload) => serde_json::to_value(payload),
            Event::Pong(payload) => serde_json::to_value(payload),
            Event::ToolCall(payload) => serde_json::to_value(payload),
            Event::ToolResult(payload) => serde_json::to_value(payload),
        };
        value.map_err(|_| Refusal::SchemaInvalid)
    }

    /// Convenience: event -> envelope.
    ///
    /// # Errors
    /// `E_SCHEMA_INVALID` when the payload cannot be serialized.
    pub fn into_envelope(self) -> Result<Envelope, Refusal> {
        let name = self.name();
        Envelope::event(name, &self.payload_value()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_names_match_the_frozen_vocabulary() {
        let frozen = [
            "ACK",
            "ERROR",
            "SERVER_HELLO",
            "RESYNC_REQUIRED",
            "RUN_CREATED",
            "RUN_STATUS",
            "REPLY_DELTA",
            "REPLY_DONE",
            "NODE_SPAWN",
            "NODE_DRAG",
            "NODE_REWIRE",
            "NODE_STATE",
            "RUN_METRICS",
            "APPROVAL_REQUEST",
            "APPROVAL_RESOLVED",
            "SETTINGS_STATUS_RESULT",
            "FILE_TREE_RESULT",
            "FILE_CONTENT",
            "HISTORY_SNAPSHOT",
            "PING",
            "PONG",
            "TOOL_CALL",
            "TOOL_RESULT",
        ];
        assert_eq!(EventName::ALL.len(), frozen.len());
        for (name, expected) in EventName::ALL.iter().zip(frozen) {
            assert_eq!(name.as_str(), expected);
            assert_eq!(EventName::from_name(expected), Some(*name));
        }
        assert_eq!(EventName::from_name("NOT_AN_EVENT"), None);
    }

    #[test]
    fn node_kind_vocabulary_matches_the_web_mirror() {
        assert_eq!(
            NODE_KIND_VOCABULARY,
            &["hyperagent", "superagent", "worker", "tool"]
        );
        assert!(
            !NODE_KIND_VOCABULARY.contains(&"error"),
            "`error` is a client-only broken-route state, never a server kind"
        );
    }

    #[test]
    fn node_event_legacy_payload_keeps_the_frozen_wire_shape() {
        let legacy = serde_json::json!({
            "task_id": TaskId::new().to_string(),
            "state": "running",
        });
        let decoded: NodeEvent =
            serde_json::from_value(legacy).expect("a pre-1.3 NODE_* payload still decodes");
        assert_eq!(decoded.rev, None);
        assert_eq!(decoded.kind, None);
        assert_eq!(decoded.parent_task_id, None);

        let encoded = serde_json::to_value(&decoded).expect("re-encodes");
        let object = encoded.as_object().expect("payload is an object");
        assert_eq!(
            object.len(),
            2,
            "absent topology fields must be omitted, not sent as null: {encoded}"
        );
        assert!(object.contains_key("task_id") && object.contains_key("state"));
    }

    #[test]
    fn node_event_carries_the_actual_rev_kind_and_parent() {
        let task = TaskId::new();
        let parent = TaskId::new();
        let payload = NodeEvent {
            task_id: task,
            state: AgentState::Running,
            rev: Some(9),
            kind: Some("superagent".to_owned()),
            parent_task_id: Some(parent),
        };
        let encoded = serde_json::to_value(&payload).expect("encodes");
        assert_eq!(encoded["rev"], serde_json::json!(9));
        assert_eq!(encoded["kind"], serde_json::json!("superagent"));
        assert_eq!(
            encoded["parent_task_id"],
            serde_json::json!(parent.to_string())
        );
        let decoded: NodeEvent = serde_json::from_value(encoded).expect("decodes");
        assert_eq!(decoded, payload);
    }

    #[test]
    fn approval_request_without_a_preview_keeps_the_frozen_wire_shape() {
        let legacy = serde_json::json!({
            "approval_id": ApprovalId::new().to_string(),
            "tool": "file_write",
        });
        let decoded: ApprovalRequest =
            serde_json::from_value(legacy).expect("a pre-2.0 APPROVAL_REQUEST still decodes");
        assert_eq!(decoded.args_preview, None);
        let encoded = serde_json::to_value(&decoded).expect("encodes");
        assert!(
            encoded.get("args_preview").is_none(),
            "an absent preview must stay absent on the wire: {encoded}"
        );
    }

    #[test]
    fn args_preview_redacts_secret_keys_and_secret_shaped_values() {
        let args = serde_json::json!({
            "path": "src/main.rs",
            "api_key": "sk-live-0123456789abcdefghijklmnop",
            "nested": {"password": "hunter2", "note": "Bearer abcdefghijklmnop"},
            "env": {"GITHUB_TOKEN": "ghp_abcdefghijklmnopqrst"},
            "count": 3,
        });
        let preview = redact_args_preview(&args);
        assert!(
            preview.contains("src/main.rs"),
            "non-secret content is preserved: {preview}"
        );
        for secret in ["sk-live", "hunter2", "Bearer", "ghp_"] {
            assert!(
                !preview.contains(secret),
                "secret {secret:?} leaked into {preview}"
            );
        }
        assert!(preview.contains(ARGS_PREVIEW_REDACTED));
        assert!(
            preview.contains("\"count\":3"),
            "non-string leaves keep their type: {preview}"
        );
    }

    #[test]
    fn args_preview_caps_each_string_array_and_depth() {
        let args = serde_json::json!({
            "content": "é".repeat(4_000),
            "astral": "𝄞".repeat(1_000),
            "blob": vec!["x"; 100],
            "deep": {"a": {"b": {"c": {"d": {"e": "f"}}}}},
        });
        let preview = redact_args_preview(&args);
        let units = preview.encode_utf16().count();
        assert!(
            units <= MAX_ARGS_PREVIEW_CHARS,
            "preview is {units} UTF-16 units, cap is {MAX_ARGS_PREVIEW_CHARS}"
        );
        assert!(
            !preview.contains(&"é".repeat(PREVIEW_STRING_CHARS + 1)),
            "the per-string cap holds: {preview}"
        );
        assert!(
            preview.contains(ARGS_PREVIEW_TRUNCATED),
            "a capped array/depth is marked: {preview}"
        );
    }

    #[test]
    fn args_preview_is_cut_and_marked_at_the_total_cap() {
        let mut wide = serde_json::Map::new();
        for index in 0..PREVIEW_OBJECT_KEYS {
            // 160 astral chars = 320 UTF-16 units per value, so the rendered
            // object necessarily exceeds the total cap.
            wide.insert(format!("k{index}"), serde_json::json!("𝄞".repeat(160)));
        }
        let preview = redact_args_preview(&serde_json::Value::Object(wide));
        let units = preview.encode_utf16().count();
        assert!(
            (MAX_ARGS_PREVIEW_CHARS - 1..=MAX_ARGS_PREVIEW_CHARS).contains(&units),
            "a budgeted preview is cut to the cap (surrogate-pair granularity): {units} units"
        );
        assert!(preview.ends_with('…'), "a cut preview is marked: {preview}");
        assert!(
            preview.chars().count() <= MAX_ARGS_PREVIEW_CHARS,
            "chars never exceed UTF-16 units"
        );
    }

    #[test]
    fn utf16_truncation_never_overshoots_the_bound() {
        for max in 0..=8_usize {
            let rendered = truncate_utf16("𝄞𝄞𝄞", max);
            assert!(
                rendered.encode_utf16().count() <= max,
                "max={max} rendered {rendered:?}"
            );
        }
        assert_eq!(truncate_utf16("abcdef", 3), "ab…");
        assert_eq!(truncate_utf16("abc", 3), "abc");
        assert_eq!(truncate_utf16("abc", 0), "");
    }

    #[test]
    fn approval_request_schema_bounds_the_preview_length() {
        let schema = serde_json::to_value(schemars::schema_for!(ApprovalRequest))
            .expect("schema serializes")
            .to_string();
        assert!(
            schema.contains("maxLength"),
            "the preview cap must be part of the derived schema: {schema}"
        );
        let oversized = serde_json::json!({
            "approval_id": ApprovalId::new().to_string(),
            "tool": "file_write",
            "args_preview": "x".repeat(MAX_ARGS_PREVIEW_CHARS + 1),
        });
        assert_eq!(
            crate::schema::validate_event_payload(EventName::ApprovalRequest, &oversized),
            Err(Refusal::SchemaInvalid),
            "an oversized preview is refused by the derived schema"
        );
        let bounded = serde_json::json!({
            "approval_id": ApprovalId::new().to_string(),
            "tool": "file_write",
            "args_preview": "x".repeat(MAX_ARGS_PREVIEW_CHARS),
        });
        assert_eq!(
            crate::schema::validate_event_payload(EventName::ApprovalRequest, &bounded),
            Ok(()),
            "a preview at the cap is accepted"
        );
    }
}
