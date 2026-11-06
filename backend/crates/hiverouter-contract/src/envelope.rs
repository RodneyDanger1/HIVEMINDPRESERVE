//! The wire envelope (`docs/contracts/UI-COMMANDS-v1.md` §1).
//!
//! ```json
//! {"schema":"hiverouter.v1","type":"<COMMAND|EVENT>","name":"<STRING>","id":"<uuid-v7>",
//!  "correlation_id":"<uuid-v7>","run_id":"<uuid|null>","task_id":"<uuid|null>",
//!  "seq":<u64|null>,"ts":"<RFC3339>","payload":{...}}
//! ```
//!
//! Command and event payloads live in [`crate::command`] / [`crate::event`];
//! decoding/encoding entry points live in [`crate::schema`].

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::command::CommandName;
use crate::event::EventName;
use crate::ids::{RunId, TaskId};
use crate::refusal::Refusal;
use crate::schema::SCHEMA_VERSION;
use crate::timestamp::Timestamp;

/// Envelope `type` discriminator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum FrameKind {
    /// Client -> server frame.
    Command,
    /// Server -> client frame.
    Event,
}

/// One wire frame.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Envelope {
    /// Contract schema id; always [`SCHEMA_VERSION`].
    pub schema: String,
    /// Frame direction discriminator (wire field `type`).
    #[serde(rename = "type")]
    pub frame_type: FrameKind,
    /// Command or event name from the frozen vocabulary.
    pub name: String,
    /// Unique frame id (uuid-v7).
    pub id: Uuid,
    /// Stable across a request/ACK/ERROR/event chain (uuid-v7).
    pub correlation_id: Uuid,
    /// Run this frame belongs to, if any.
    #[serde(default)]
    pub run_id: Option<RunId>,
    /// Task this frame belongs to, if any.
    #[serde(default)]
    pub task_id: Option<TaskId>,
    /// Server event sequence number; `None` on commands and pre-sequenced events.
    #[serde(default)]
    pub seq: Option<u64>,
    /// RFC3339 creation timestamp.
    pub ts: Timestamp,
    /// Command/event specific body.
    pub payload: Value,
}

impl Envelope {
    fn build(kind: FrameKind, name: &str, payload: Value) -> Self {
        let id = Uuid::now_v7();
        Self {
            schema: SCHEMA_VERSION.to_owned(),
            frame_type: kind,
            name: name.to_owned(),
            id,
            correlation_id: id,
            run_id: None,
            task_id: None,
            seq: None,
            ts: Timestamp::now(),
            payload,
        }
    }

    /// Builds a COMMAND envelope; `correlation_id` starts equal to `id`.
    ///
    /// # Errors
    /// Returns `E_SCHEMA_INVALID` if the payload cannot be serialized.
    pub fn command(name: CommandName, payload: &impl Serialize) -> Result<Self, Refusal> {
        let payload = serde_json::to_value(payload).map_err(|_| Refusal::SchemaInvalid)?;
        Ok(Self::build(FrameKind::Command, name.as_str(), payload))
    }

    /// Builds an EVENT envelope; `correlation_id` starts equal to `id`.
    ///
    /// # Errors
    /// Returns `E_SCHEMA_INVALID` if the payload cannot be serialized.
    pub fn event(name: EventName, payload: &impl Serialize) -> Result<Self, Refusal> {
        let payload = serde_json::to_value(payload).map_err(|_| Refusal::SchemaInvalid)?;
        Ok(Self::build(FrameKind::Event, name.as_str(), payload))
    }

    /// Sets the correlation id (ACK/ERROR chains share the request's value).
    #[must_use]
    pub fn with_correlation(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = correlation_id;
        self
    }

    /// Sets the run id.
    #[must_use]
    pub fn with_run(mut self, run_id: RunId) -> Self {
        self.run_id = Some(run_id);
        self
    }

    /// Sets the task id.
    #[must_use]
    pub fn with_task(mut self, task_id: TaskId) -> Self {
        self.task_id = Some(task_id);
        self
    }

    /// Sets the server sequence number.
    #[must_use]
    pub fn with_seq(mut self, seq: u64) -> Self {
        self.seq = Some(seq);
        self
    }

    /// Structural validation used before dispatch: schema id, frame kind,
    /// non-nil ids, command-vs-event naming and sequence rules.
    ///
    /// # Errors
    /// Returns `E_SCHEMA_INVALID` for structural violations and
    /// `E_COMMAND_UNKNOWN` for an unknown command name.
    pub fn validate_basic(&self) -> Result<(), Refusal> {
        if self.schema != SCHEMA_VERSION {
            return Err(Refusal::SchemaInvalid);
        }
        if self.id.is_nil() || self.correlation_id.is_nil() {
            return Err(Refusal::SchemaInvalid);
        }
        if self.name.is_empty() {
            return Err(Refusal::SchemaInvalid);
        }
        match self.frame_type {
            FrameKind::Command => {
                if self.seq.is_some() {
                    return Err(Refusal::SchemaInvalid);
                }
                if CommandName::from_name(&self.name).is_none() {
                    return Err(Refusal::CommandUnknown);
                }
            }
            FrameKind::Event => {
                if EventName::from_name(&self.name).is_none() {
                    return Err(Refusal::SchemaInvalid);
                }
            }
        }
        Ok(())
    }

    /// The command name of this envelope.
    ///
    /// # Errors
    /// `E_COMMAND_UNKNOWN` when the name is not a frozen command.
    pub fn command_name(&self) -> Result<CommandName, Refusal> {
        CommandName::from_name(&self.name).ok_or(Refusal::CommandUnknown)
    }

    /// The event name of this envelope.
    ///
    /// # Errors
    /// `E_SCHEMA_INVALID` when the name is not a frozen event.
    pub fn event_name(&self) -> Result<EventName, Refusal> {
        EventName::from_name(&self.name).ok_or(Refusal::SchemaInvalid)
    }

    /// Serializes the envelope to JSON.
    ///
    /// # Errors
    /// `E_SCHEMA_INVALID` if serialization fails.
    pub fn to_json(&self) -> Result<String, Refusal> {
        serde_json::to_string(self).map_err(|_| Refusal::SchemaInvalid)
    }

    /// Deserializes the payload into a typed value.
    ///
    /// # Errors
    /// `E_SCHEMA_INVALID` if the payload does not match the typed payload.
    pub fn decode_payload<T: for<'de> Deserialize<'de>>(&self) -> Result<T, Refusal> {
        serde_json::from_value(self.payload.clone()).map_err(|_| Refusal::SchemaInvalid)
    }
}
