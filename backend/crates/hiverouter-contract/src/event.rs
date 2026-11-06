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

/// `NODE_SPAWN` / `NODE_DRAG` / `NODE_REWIRE` / `NODE_STATE` payload (frozen
/// contract: `task_id`, `state`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NodeEvent {
    /// Task id.
    pub task_id: TaskId,
    /// New node state.
    pub state: AgentState,
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
        ];
        assert_eq!(EventName::ALL.len(), frozen.len());
        for (name, expected) in EventName::ALL.iter().zip(frozen) {
            assert_eq!(name.as_str(), expected);
            assert_eq!(EventName::from_name(expected), Some(*name));
        }
        assert_eq!(EventName::from_name("NOT_AN_EVENT"), None);
    }
}
