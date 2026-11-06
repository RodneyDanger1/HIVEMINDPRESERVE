//! Frozen HiveMind v1 wire contract.
//!
//! Single source of truth for the command/event vocabulary frozen in
//! `docs/contracts/UI-COMMANDS-v1.md` (freeze date 2026-11-06). Every frame the
//! backend emits or accepts is built or parsed here; no other module may define
//! its own envelope, command name, event name, refusal code or agent state.
//!
//! Layout:
//! - [`envelope`] — the `{"schema":"hiverouter.v1","type":...}` wire envelope
//! - [`command`] — frozen client to server commands and their payloads
//! - [`event`] — frozen server to client events and their payloads
//! - [`refusal`] — the named refusal table (stable `E_*` codes)
//! - [`ids`] — uuid-v7 newtypes for run/task/approval identifiers
//! - [`state`] — the single agent-state enum
//! - [`timestamp`] — RFC3339 timestamp newtype (UTC-normalised)
//! - [`usage`] — provider usage counters; absent values stay absent (never 0)
//! - [`schema`] — `schema_version`, decoders/encoders, jsonschema validation

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations)]

pub mod command;
pub mod envelope;
pub mod event;
pub mod ids;
pub mod refusal;
pub mod schema;
pub mod state;
pub mod timestamp;
pub mod usage;

pub use command::{
    ApprovalResolve, CancelRun, ChatPin, Command, CommandName, DiffDecision, FileRead, FileTreeList,
    HistoryRevisionSet, KillWorker, RewireTask, SettingsStatus, SpawnSubHive, SubmitPrompt,
    TemplateInsert, ViewportRecenter, ViewportSet,
};
pub use envelope::{Envelope, FrameKind};
pub use event::{Event, EventName, ToolCallPayload, ToolOutcome, ToolResultPayload};
pub use ids::{ApprovalId, RunId, TaskId};
pub use refusal::Refusal;
pub use schema::{
    decode_command, decode_event, encode_command, encode_event, parse_envelope, validate_envelope,
    SCHEMA_VERSION,
};
pub use state::AgentState;
pub use timestamp::Timestamp;
pub use usage::Usage;
