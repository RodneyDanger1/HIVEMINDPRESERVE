//! Schema version, decode/encode entry points, jsonschema validation and the
//! command acceptance matrix.
//!
//! One source of truth: the Rust structs derive `schemars::JsonSchema`; the
//! derived schema documents are the golden snapshots under `schemas/` and are
//! what `jsonschema` validates incoming payloads against (blueprint WP 1.7).

use std::collections::HashMap;
use std::sync::OnceLock;

use schemars::JsonSchema;
use serde_json::Value;

use crate::command::{Command, CommandName};
use crate::envelope::Envelope;
use crate::event::{Event, EventName};
use crate::refusal::Refusal;

/// Contract schema id that every frame carries and every decoder checks.
pub const SCHEMA_VERSION: &str = "hiverouter.v1";

/// Parses an envelope from a JSON value.
///
/// # Errors
/// `E_SCHEMA_INVALID` when the value does not match the envelope shape.
pub fn parse_envelope(value: &Value) -> Result<Envelope, Refusal> {
    serde_json::from_value(value.clone()).map_err(|_| Refusal::SchemaInvalid)
}

/// Parses an envelope from a JSON string.
///
/// # Errors
/// `E_BAD_FRAME` when the text is not JSON, `E_SCHEMA_INVALID` when it does not
/// match the envelope shape.
pub fn parse_envelope_json(json: &str) -> Result<Envelope, Refusal> {
    let value: Value = serde_json::from_str(json).map_err(|_| Refusal::BadFrame)?;
    parse_envelope(&value)
}

/// Validates an already-parsed envelope.
///
/// # Errors
/// The named structural refusal.
pub fn validate_envelope(envelope: &Envelope) -> Result<(), Refusal> {
    envelope.validate_basic()
}

/// Decodes a command from an envelope (structure -> name -> schema -> typed ->
/// semantic validation).
///
/// # Errors
/// The named refusal of the first failed gate.
pub fn decode_command(envelope: &Envelope) -> Result<Command, Refusal> {
    Command::from_envelope(envelope)
}

/// Encodes a command as a COMMAND envelope.
///
/// # Errors
/// `E_SCHEMA_INVALID` when the payload cannot be serialized.
pub fn encode_command(command: &Command) -> Result<Envelope, Refusal> {
    Envelope::command(command.name(), &command.payload_value()?)
}

/// Decodes a typed event from an envelope.
///
/// # Errors
/// `E_SCHEMA_INVALID` on name, direction or payload mismatch.
pub fn decode_event(envelope: &Envelope) -> Result<Event, Refusal> {
    Event::from_envelope(envelope)
}

/// Encodes a typed event as an EVENT envelope.
///
/// # Errors
/// `E_SCHEMA_INVALID` when the payload cannot be serialized.
pub fn encode_event(event: &Event) -> Result<Envelope, Refusal> {
    event.clone().into_envelope()
}

fn to_schema_value<T: JsonSchema>() -> Result<Value, Refusal> {
    serde_json::to_value(schemars::schema_for!(T)).map_err(|_| Refusal::SchemaInvalid)
}

/// JSON schema of the envelope shape.
///
/// # Errors
/// `E_SCHEMA_INVALID` if the generated schema cannot be serialized.
pub fn envelope_schema() -> Result<Value, Refusal> {
    to_schema_value::<Envelope>()
}

/// JSON schema of one command payload, when the payload has a schema.
#[must_use]
pub fn command_payload_schema(name: CommandName) -> Option<Result<Value, Refusal>> {
    let schema = match name {
        CommandName::SubmitPrompt => to_schema_value::<crate::command::SubmitPrompt>(),
        CommandName::SpawnSubHive => to_schema_value::<crate::command::SpawnSubHive>(),
        CommandName::RewireTask => to_schema_value::<crate::command::RewireTask>(),
        CommandName::CancelRun => to_schema_value::<crate::command::CancelRun>(),
        CommandName::KillWorker => to_schema_value::<crate::command::KillWorker>(),
        CommandName::ApprovalResolve => to_schema_value::<crate::command::ApprovalResolve>(),
        CommandName::SettingsStatus => to_schema_value::<crate::command::SettingsStatus>(),
        CommandName::FileTreeList => to_schema_value::<crate::command::FileTreeList>(),
        CommandName::FileRead => to_schema_value::<crate::command::FileRead>(),
        CommandName::ChatPin => to_schema_value::<crate::command::ChatPin>(),
        CommandName::TemplateInsert => to_schema_value::<crate::command::TemplateInsert>(),
        CommandName::HistoryRevisionSet => {
            to_schema_value::<crate::command::HistoryRevisionSet>()
        }
        CommandName::ViewportSet => to_schema_value::<crate::command::ViewportSet>(),
        CommandName::ViewportRecenter => to_schema_value::<crate::command::ViewportRecenter>(),
        // DIFF_* is decision-gated and not frozen; its body is opaque in v1.
        CommandName::DiffPropose | CommandName::DiffApply | CommandName::DiffReject => return None,
    };
    Some(schema)
}

/// JSON schema of one event payload.
#[must_use]
pub fn event_payload_schema(name: EventName) -> Option<Result<Value, Refusal>> {
    let schema = match name {
        EventName::Ack => to_schema_value::<crate::event::Ack>(),
        EventName::Error => to_schema_value::<crate::event::ErrorPayload>(),
        EventName::ServerHello => to_schema_value::<crate::event::ServerHello>(),
        EventName::ResyncRequired => to_schema_value::<crate::event::ResyncRequiredPayload>(),
        EventName::RunCreated => to_schema_value::<crate::event::RunCreated>(),
        EventName::RunStatus => to_schema_value::<crate::event::RunStatusPayload>(),
        EventName::ReplyDelta => to_schema_value::<crate::event::ReplyDelta>(),
        EventName::ReplyDone => to_schema_value::<crate::event::ReplyDone>(),
        EventName::NodeSpawn
        | EventName::NodeDrag
        | EventName::NodeRewire
        | EventName::NodeState => to_schema_value::<crate::event::NodeEvent>(),
        EventName::RunMetrics => to_schema_value::<crate::event::RunMetrics>(),
        EventName::ApprovalRequest => to_schema_value::<crate::event::ApprovalRequest>(),
        EventName::ApprovalResolved => to_schema_value::<crate::event::ApprovalResolved>(),
        EventName::SettingsStatusResult => to_schema_value::<crate::event::SettingsStatusResult>(),
        EventName::FileTreeResult => to_schema_value::<crate::event::FileTreeResult>(),
        EventName::FileContent => to_schema_value::<crate::event::FileContent>(),
        EventName::HistorySnapshot => to_schema_value::<crate::event::HistorySnapshot>(),
        EventName::Ping | EventName::Pong => to_schema_value::<crate::event::EmptyPayload>(),
        EventName::ToolCall => to_schema_value::<crate::event::ToolCallPayload>(),
        EventName::ToolResult => to_schema_value::<crate::event::ToolResultPayload>(),
        EventName::AgentMessage => to_schema_value::<crate::event::AgentMessagePayload>(),
    };
    Some(schema)
}

fn command_validators() -> &'static HashMap<CommandName, jsonschema::Validator> {
    static CACHE: OnceLock<HashMap<CommandName, jsonschema::Validator>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut validators = HashMap::new();
        for name in CommandName::ALL {
            if let Some(Ok(schema)) = command_payload_schema(*name) {
                if let Ok(validator) = jsonschema::validator_for(&schema) {
                    validators.insert(*name, validator);
                }
            }
        }
        validators
    })
}

/// Validates a command payload against the schema derived from its Rust type.
///
/// # Errors
/// `E_SCHEMA_INVALID` when the payload does not validate (or when a frozen
/// command has no compiled schema, which is an internal contract defect).
pub fn validate_command_payload(name: CommandName, payload: &Value) -> Result<(), Refusal> {
    let validator = command_validators()
        .get(&name)
        .ok_or(Refusal::SchemaInvalid)?;
    if validator.is_valid(payload) {
        Ok(())
    } else {
        Err(Refusal::SchemaInvalid)
    }
}

/// Validates an event payload against the schema derived from its Rust type.
///
/// # Errors
/// `E_SCHEMA_INVALID` when the payload does not validate.
pub fn validate_event_payload(name: EventName, payload: &Value) -> Result<(), Refusal> {
    let Some(Ok(schema)) = event_payload_schema(name) else {
        return Err(Refusal::SchemaInvalid);
    };
    let validator = jsonschema::validator_for(&schema).map_err(|_| Refusal::SchemaInvalid)?;
    if validator.is_valid(payload) {
        Ok(())
    } else {
        Err(Refusal::SchemaInvalid)
    }
}

/// One row of the command acceptance matrix (frozen contract §7): every frozen
/// command has a payload schema, an ACK path and at least one named refusal.
#[derive(Debug)]
pub struct CommandSpec {
    /// Command name.
    pub name: CommandName,
    /// Rust payload type the schema is derived from.
    pub payload_schema: &'static str,
    /// Whether the command has an ACK path.
    pub ack: bool,
    /// Whether the name is frozen in v1.
    pub frozen: bool,
    /// Named refusals registered for this command.
    pub refusals: &'static [Refusal],
}

/// The command acceptance matrix. Extending a command requires extending this
/// table and re-running `cargo test ui_command_matrix`.
pub const COMMAND_MATRIX: &[CommandSpec] = &[
    CommandSpec {
        name: CommandName::SubmitPrompt,
        payload_schema: "SubmitPrompt",
        ack: true,
        frozen: true,
        refusals: &[
            Refusal::PromptEmpty,
            Refusal::PromptTooLong,
            Refusal::RunActive,
            Refusal::KeyMissing,
        ],
    },
    CommandSpec {
        name: CommandName::SpawnSubHive,
        payload_schema: "SpawnSubHive",
        ack: true,
        frozen: true,
        refusals: &[
            Refusal::SpawnDepth,
            Refusal::FanoutLimit,
            Refusal::TaskNotFound,
            Refusal::RunTerminal,
        ],
    },
    CommandSpec {
        name: CommandName::RewireTask,
        payload_schema: "RewireTask",
        ack: true,
        frozen: true,
        refusals: &[
            Refusal::Cycle,
            Refusal::ResyncRequired,
            Refusal::RewireInFlight,
            Refusal::TaskNotFound,
        ],
    },
    CommandSpec {
        name: CommandName::CancelRun,
        payload_schema: "CancelRun",
        ack: true,
        frozen: true,
        refusals: &[Refusal::RunTerminal, Refusal::RunNotFound],
    },
    CommandSpec {
        name: CommandName::KillWorker,
        payload_schema: "KillWorker",
        ack: true,
        frozen: true,
        refusals: &[
            Refusal::TaskNotFound,
            Refusal::TaskTerminal,
            Refusal::KillDenied,
        ],
    },
    CommandSpec {
        name: CommandName::ApprovalResolve,
        payload_schema: "ApprovalResolve",
        ack: true,
        frozen: true,
        refusals: &[
            Refusal::ApprovalNotFound,
            Refusal::ApprovalReplay,
            Refusal::ApprovalTimeout,
            Refusal::ApprovalDecisionInvalid,
            Refusal::ResyncRequired,
        ],
    },
    CommandSpec {
        name: CommandName::SettingsStatus,
        payload_schema: "SettingsStatus",
        ack: true,
        frozen: true,
        // Frozen contract: missing key is data (`key_present:false`), not a refusal.
        refusals: &[Refusal::CommandUnknown],
    },
    CommandSpec {
        name: CommandName::FileTreeList,
        payload_schema: "FileTreeList",
        ack: true,
        frozen: true,
        refusals: &[
            Refusal::PathOutsideJail,
            Refusal::PathNotFound,
            Refusal::FileTreeUnavailable,
        ],
    },
    CommandSpec {
        name: CommandName::FileRead,
        payload_schema: "FileRead",
        ack: true,
        frozen: true,
        refusals: &[
            Refusal::PathOutsideJail,
            Refusal::PathNotFound,
            Refusal::FileTooLarge,
            Refusal::FileTreeUnavailable,
        ],
    },
    CommandSpec {
        name: CommandName::ChatPin,
        payload_schema: "ChatPin",
        ack: true,
        frozen: true,
        // Frozen contract: CHAT_PIN has no command-specific refusal; an unknown
        // command name is the only named refusal listed for the row.
        refusals: &[Refusal::CommandUnknown],
    },
    CommandSpec {
        name: CommandName::TemplateInsert,
        payload_schema: "TemplateInsert",
        ack: true,
        frozen: true,
        refusals: &[
            Refusal::TemplateUnknown,
            Refusal::TemplateLicenseMissing,
            Refusal::TemplateRefForbidden,
        ],
    },
    CommandSpec {
        name: CommandName::HistoryRevisionSet,
        payload_schema: "HistoryRevisionSet",
        ack: true,
        frozen: true,
        refusals: &[Refusal::HistoryGap, Refusal::RunNotFound],
    },
    CommandSpec {
        name: CommandName::ViewportSet,
        payload_schema: "ViewportSet",
        ack: true,
        frozen: true,
        refusals: &[Refusal::ZoomRange],
    },
    CommandSpec {
        name: CommandName::ViewportRecenter,
        payload_schema: "ViewportRecenter",
        ack: true,
        frozen: true,
        // RECENTER carries no zoom; the frozen row's E_ZOOM_RANGE applies to
        // VIEWPORT_SET. Schema refusal is the operative path for RECENTER.
        refusals: &[Refusal::ZoomRange, Refusal::SchemaInvalid],
    },
    CommandSpec {
        name: CommandName::DiffPropose,
        payload_schema: "opaque (decision-gated; not frozen)",
        ack: false,
        frozen: false,
        refusals: &[Refusal::DecisionRequired],
    },
    CommandSpec {
        name: CommandName::DiffApply,
        payload_schema: "opaque (decision-gated; not frozen)",
        ack: false,
        frozen: false,
        refusals: &[Refusal::DecisionRequired],
    },
    CommandSpec {
        name: CommandName::DiffReject,
        payload_schema: "opaque (decision-gated; not frozen)",
        ack: false,
        frozen: false,
        refusals: &[Refusal::DecisionRequired],
    },
];

/// Matrix row for a command name.
#[must_use]
pub fn command_spec(name: CommandName) -> Option<&'static CommandSpec> {
    COMMAND_MATRIX.iter().find(|spec| spec.name == name)
}
