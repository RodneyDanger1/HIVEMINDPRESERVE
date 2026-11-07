//! Frozen client -> server commands and their payloads
//! (`docs/contracts/UI-COMMANDS-v1.md` §2).
//!
//! # Pending-approval pull (documented; no command is added)
//!
//! The frozen vocabulary deliberately has no `APPROVAL_HYDRATE` command. The
//! transport already carries a real client-initiated pull, so no request or
//! response event is missing: the untyped `{"type":"hello","last_seq":<n>}`
//! control frame (the same handshake a reconnect sends). On every hello the
//! server answers with `SERVER_HELLO` and re-publishes a bounded batch of open
//! approvals as ordinary `APPROVAL_REQUEST` events — the response side needs no
//! new event, and replay is replace-by-id, so it is idempotent and never
//! fabricates a request. A live connection whose `last_seq` is current gets zero
//! event replay (`EventBus::replay_after` returns `Replay::Current`), so the pull
//! is safe mid-session.
//!
//! A typed command was rejected for this wave because it is **not** additive: a
//! new `CommandName`/`Command` variant breaks every exhaustive `match` outside
//! this crate (`schema.rs::command_payload_schema`,
//! `ws.rs::handle_client_text`), needs a new `COMMAND_MATRIX` row, and requires
//! regenerating the golden `schemas/contract-v1.json` — a coordinated contract
//! wave plus a new server handler, not an optional field. Adding the name
//! without those pieces would ship dead protocol surface that the server
//! answers with `E_COMMAND_UNKNOWN`, so it was not added.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::envelope::{Envelope, FrameKind};
use crate::ids::{ApprovalId, RunId, TaskId};
use crate::refusal::Refusal;
use crate::schema;

/// Maximum prompt length accepted by `SUBMIT_PROMPT` (frozen contract §2).
pub const MAX_PROMPT_CHARS: usize = 8_000;
/// Maximum file size readable through `FILE_READ` (frozen contract §2: 1 MiB).
pub const MAX_FILE_BYTES: u64 = 1_048_576;
/// Allowed `APPROVAL_RESOLVE` decision values (frozen contract §2).
pub const APPROVAL_DECISIONS: &[&str] = &["approve", "deny"];
/// Minimum zoom accepted by `VIEWPORT_SET` (frozen contract §2).
pub const MIN_ZOOM: f64 = 0.1;
/// Maximum zoom accepted by `VIEWPORT_SET` (frozen contract §2).
pub const MAX_ZOOM: f64 = 3.0;

/// Command names of the frozen vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CommandName {
    /// `SUBMIT_PROMPT` — user submits a prompt, creates a run.
    SubmitPrompt,
    /// `SPAWN_SUB_HIVE` — spawn a child task.
    SpawnSubHive,
    /// `REWIRE_TASK` — move a task under a different parent.
    RewireTask,
    /// `CANCEL_RUN` — cancel an active run.
    CancelRun,
    /// `KILL_WORKER` — kill a running worker.
    KillWorker,
    /// `APPROVAL_RESOLVE` — resolve a pending tool approval.
    ApprovalResolve,
    /// `SETTINGS_STATUS` — read provider/model/key status (never the key).
    SettingsStatus,
    /// `FILE_TREE_LIST` — list the workspace file tree.
    FileTreeList,
    /// `FILE_READ` — read one workspace file (jailed, <= 1 MiB).
    FileRead,
    /// `CHAT_PIN` — pin chat (client surface only).
    ChatPin,
    /// `TEMPLATE_INSERT` — insert a template node set.
    TemplateInsert,
    /// `HISTORY_REVISION_SET` — request a history revision snapshot.
    HistoryRevisionSet,
    /// `VIEWPORT_SET` — set viewport (client-only, no server effect).
    ViewportSet,
    /// `VIEWPORT_RECENTER` — recenter viewport (client-only, no server effect).
    ViewportRecenter,
    /// `DIFF_PROPOSE` — decision-gated, NOT frozen in v1.
    DiffPropose,
    /// `DIFF_APPLY` — decision-gated, NOT frozen in v1.
    DiffApply,
    /// `DIFF_REJECT` — decision-gated, NOT frozen in v1.
    DiffReject,
}

impl CommandName {
    /// Every command name (declaration order).
    pub const ALL: &'static [CommandName] = &[
        CommandName::SubmitPrompt,
        CommandName::SpawnSubHive,
        CommandName::RewireTask,
        CommandName::CancelRun,
        CommandName::KillWorker,
        CommandName::ApprovalResolve,
        CommandName::SettingsStatus,
        CommandName::FileTreeList,
        CommandName::FileRead,
        CommandName::ChatPin,
        CommandName::TemplateInsert,
        CommandName::HistoryRevisionSet,
        CommandName::ViewportSet,
        CommandName::ViewportRecenter,
        CommandName::DiffPropose,
        CommandName::DiffApply,
        CommandName::DiffReject,
    ];

    /// Stable wire name (SCREAMING_SNAKE_CASE).
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            CommandName::SubmitPrompt => "SUBMIT_PROMPT",
            CommandName::SpawnSubHive => "SPAWN_SUB_HIVE",
            CommandName::RewireTask => "REWIRE_TASK",
            CommandName::CancelRun => "CANCEL_RUN",
            CommandName::KillWorker => "KILL_WORKER",
            CommandName::ApprovalResolve => "APPROVAL_RESOLVE",
            CommandName::SettingsStatus => "SETTINGS_STATUS",
            CommandName::FileTreeList => "FILE_TREE_LIST",
            CommandName::FileRead => "FILE_READ",
            CommandName::ChatPin => "CHAT_PIN",
            CommandName::TemplateInsert => "TEMPLATE_INSERT",
            CommandName::HistoryRevisionSet => "HISTORY_REVISION_SET",
            CommandName::ViewportSet => "VIEWPORT_SET",
            CommandName::ViewportRecenter => "VIEWPORT_RECENTER",
            CommandName::DiffPropose => "DIFF_PROPOSE",
            CommandName::DiffApply => "DIFF_APPLY",
            CommandName::DiffReject => "DIFF_REJECT",
        }
    }

    /// Looks a command up by wire name.
    #[must_use]
    pub fn from_name(name: &str) -> Option<CommandName> {
        CommandName::ALL
            .iter()
            .copied()
            .find(|candidate| candidate.as_str() == name)
    }

    /// True for names frozen in UI-COMMANDS-v1. `DIFF_*` is decision-gated and
    /// explicitly not frozen ("NOT FROZEN — do not implement").
    #[must_use]
    pub fn is_frozen(&self) -> bool {
        !matches!(
            self,
            CommandName::DiffPropose | CommandName::DiffApply | CommandName::DiffReject
        )
    }

    /// Stable command code used to classify a refusal/ack path.
    #[must_use]
    pub fn is_mutating(&self) -> bool {
        matches!(
            self,
            CommandName::SubmitPrompt
                | CommandName::SpawnSubHive
                | CommandName::RewireTask
                | CommandName::CancelRun
                | CommandName::KillWorker
                | CommandName::ApprovalResolve
                | CommandName::TemplateInsert
        )
    }
}

/// `SUBMIT_PROMPT` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SubmitPrompt {
    /// Prompt text, 1..=8000 characters.
    pub text: String,
    /// Project/workspace the run belongs to.
    pub project_id: String,
}

/// `SPAWN_SUB_HIVE` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SpawnSubHive {
    /// Parent task to spawn under.
    pub parent_task_id: TaskId,
    /// Agent role/tier name for the child.
    pub role: String,
}

/// `REWIRE_TASK` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RewireTask {
    /// Task to move.
    pub task_id: TaskId,
    /// New parent task.
    pub new_parent_id: TaskId,
    /// Optimistic-concurrency revision the client observed.
    pub expected_rev: u64,
}

/// `CANCEL_RUN` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CancelRun {
    /// Run to cancel.
    pub run_id: RunId,
}

/// `KILL_WORKER` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct KillWorker {
    /// Task whose worker should be killed.
    pub task_id: TaskId,
}

/// `APPROVAL_RESOLVE` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ApprovalResolve {
    /// Approval request being resolved.
    pub approval_id: ApprovalId,
    /// Single-use nonce from the `APPROVAL_REQUEST` frame.
    pub nonce: String,
    /// Frozen decision value: `approve` or `deny`.
    pub decision: String,
    /// Optional operator reason.
    #[serde(default)]
    pub reason: Option<String>,
}

/// `SETTINGS_STATUS` payload (no fields).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SettingsStatus {}

/// `FILE_TREE_LIST` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FileTreeList {
    /// Optional workspace-relative directory to list; absent means the root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// `FILE_READ` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FileRead {
    /// Workspace-relative path of the file to read.
    pub path: String,
}

/// `CHAT_PIN` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ChatPin {
    /// Pinned state of the chat surface.
    pub pinned: bool,
}

/// `TEMPLATE_INSERT` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TemplateInsert {
    /// Template manifest row id.
    pub template_id: String,
    /// Project/workspace the nodes are inserted into.
    pub project_id: String,
}

/// `HISTORY_REVISION_SET` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct HistoryRevisionSet {
    /// Run whose history is requested.
    pub run_id: RunId,
    /// Revision to read; must be <= latest.
    pub revision: u64,
}

/// `VIEWPORT_SET` payload.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ViewportSet {
    /// Canvas x translation.
    pub x: f64,
    /// Canvas y translation.
    pub y: f64,
    /// Zoom factor, 0.1..=3.0.
    pub zoom: f64,
}

/// `VIEWPORT_RECENTER` payload (no fields; a recenter request is `{}`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ViewportRecenter {}

/// `DIFF_*` payload — decision-gated and not frozen in v1, so it stays opaque.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DiffDecision {
    /// Opaque decision body; never interpreted in v1.
    pub decision: serde_json::Value,
}

/// A decoded command.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Command {
    /// See [`SubmitPrompt`].
    SubmitPrompt(SubmitPrompt),
    /// See [`SpawnSubHive`].
    SpawnSubHive(SpawnSubHive),
    /// See [`RewireTask`].
    RewireTask(RewireTask),
    /// See [`CancelRun`].
    CancelRun(CancelRun),
    /// See [`KillWorker`].
    KillWorker(KillWorker),
    /// See [`ApprovalResolve`].
    ApprovalResolve(ApprovalResolve),
    /// See [`SettingsStatus`].
    SettingsStatus(SettingsStatus),
    /// See [`FileTreeList`].
    FileTreeList(FileTreeList),
    /// See [`FileRead`].
    FileRead(FileRead),
    /// See [`ChatPin`].
    ChatPin(ChatPin),
    /// See [`TemplateInsert`].
    TemplateInsert(TemplateInsert),
    /// See [`HistoryRevisionSet`].
    HistoryRevisionSet(HistoryRevisionSet),
    /// See [`ViewportSet`].
    ViewportSet(ViewportSet),
    /// See [`ViewportRecenter`].
    ViewportRecenter(ViewportRecenter),
    /// Decision-gated, not frozen.
    DiffPropose(DiffDecision),
    /// Decision-gated, not frozen.
    DiffApply(DiffDecision),
    /// Decision-gated, not frozen.
    DiffReject(DiffDecision),
}

impl Command {
    /// Serializes the typed payload.
    ///
    /// # Errors
    /// `E_SCHEMA_INVALID` when the payload cannot be serialized.
    pub fn payload_value(&self) -> Result<serde_json::Value, Refusal> {
        let value = match self {
            Command::SubmitPrompt(payload) => serde_json::to_value(payload),
            Command::SpawnSubHive(payload) => serde_json::to_value(payload),
            Command::RewireTask(payload) => serde_json::to_value(payload),
            Command::CancelRun(payload) => serde_json::to_value(payload),
            Command::KillWorker(payload) => serde_json::to_value(payload),
            Command::ApprovalResolve(payload) => serde_json::to_value(payload),
            Command::SettingsStatus(payload) => serde_json::to_value(payload),
            Command::FileTreeList(payload) => serde_json::to_value(payload),
            Command::FileRead(payload) => serde_json::to_value(payload),
            Command::ChatPin(payload) => serde_json::to_value(payload),
            Command::TemplateInsert(payload) => serde_json::to_value(payload),
            Command::HistoryRevisionSet(payload) => serde_json::to_value(payload),
            Command::ViewportSet(payload) => serde_json::to_value(payload),
            Command::ViewportRecenter(payload) => serde_json::to_value(payload),
            Command::DiffPropose(payload) => serde_json::to_value(payload),
            Command::DiffApply(payload) => serde_json::to_value(payload),
            Command::DiffReject(payload) => serde_json::to_value(payload),
        };
        value.map_err(|_| Refusal::SchemaInvalid)
    }

    /// Convenience: command -> COMMAND envelope.
    ///
    /// # Errors
    /// `E_SCHEMA_INVALID` when the payload cannot be serialized.
    pub fn into_envelope(self) -> Result<Envelope, Refusal> {
        let payload = self.payload_value()?;
        Envelope::command(self.name(), &payload)
    }

    /// Command name of this value.
    #[must_use]
    pub fn name(&self) -> CommandName {
        match self {
            Command::SubmitPrompt(_) => CommandName::SubmitPrompt,
            Command::SpawnSubHive(_) => CommandName::SpawnSubHive,
            Command::RewireTask(_) => CommandName::RewireTask,
            Command::CancelRun(_) => CommandName::CancelRun,
            Command::KillWorker(_) => CommandName::KillWorker,
            Command::ApprovalResolve(_) => CommandName::ApprovalResolve,
            Command::SettingsStatus(_) => CommandName::SettingsStatus,
            Command::FileTreeList(_) => CommandName::FileTreeList,
            Command::FileRead(_) => CommandName::FileRead,
            Command::ChatPin(_) => CommandName::ChatPin,
            Command::TemplateInsert(_) => CommandName::TemplateInsert,
            Command::HistoryRevisionSet(_) => CommandName::HistoryRevisionSet,
            Command::ViewportSet(_) => CommandName::ViewportSet,
            Command::ViewportRecenter(_) => CommandName::ViewportRecenter,
            Command::DiffPropose(_) => CommandName::DiffPropose,
            Command::DiffApply(_) => CommandName::DiffApply,
            Command::DiffReject(_) => CommandName::DiffReject,
        }
    }

    /// Semantic validation after schema validation. Returns the first refusal.
    ///
    /// # Errors
    /// Returns the named refusal for the violated precondition.
    pub fn validate(&self) -> Result<(), Refusal> {
        match self {
            Command::SubmitPrompt(payload) => {
                let chars = payload.text.chars().count();
                if chars == 0 {
                    return Err(Refusal::PromptEmpty);
                }
                if chars > MAX_PROMPT_CHARS {
                    return Err(Refusal::PromptTooLong);
                }
                Ok(())
            }
            Command::ViewportSet(payload) => {
                if !payload.x.is_finite() || !payload.y.is_finite() || !payload.zoom.is_finite() {
                    return Err(Refusal::SchemaInvalid);
                }
                if !(MIN_ZOOM..=MAX_ZOOM).contains(&payload.zoom) {
                    return Err(Refusal::ZoomRange);
                }
                Ok(())
            }
            Command::SpawnSubHive(payload) => {
                if payload.role.is_empty() {
                    return Err(Refusal::SchemaInvalid);
                }
                Ok(())
            }
            Command::ApprovalResolve(payload) => {
                if payload.nonce.trim().is_empty() {
                    return Err(Refusal::SchemaInvalid);
                }
                if !APPROVAL_DECISIONS.contains(&payload.decision.as_str()) {
                    return Err(Refusal::ApprovalDecisionInvalid);
                }
                Ok(())
            }
            Command::FileRead(payload) => {
                if payload.path.trim().is_empty() {
                    return Err(Refusal::SchemaInvalid);
                }
                Ok(())
            }
            Command::ChatPin(_)
            | Command::RewireTask(_)
            | Command::CancelRun(_)
            | Command::KillWorker(_)
            | Command::SettingsStatus(_)
            | Command::FileTreeList(_)
            | Command::TemplateInsert(_)
            | Command::HistoryRevisionSet(_)
            | Command::ViewportRecenter(_) => Ok(()),
            Command::DiffPropose(_) | Command::DiffApply(_) | Command::DiffReject(_) => {
                Err(Refusal::DecisionRequired)
            }
        }
    }

    /// Parses and validates a command envelope.
    ///
    /// Order: envelope structure -> frozen-name lookup -> jsonschema payload
    /// validation -> typed deserialization -> semantic validation.
    ///
    /// # Errors
    /// Returns the named refusal for the first failed gate; never a silent drop.
    pub fn from_envelope(envelope: &Envelope) -> Result<Self, Refusal> {
        envelope.validate_basic()?;
        if envelope.frame_type != FrameKind::Command {
            return Err(Refusal::SchemaInvalid);
        }
        let name = envelope.command_name()?;
        if !name.is_frozen() {
            return Err(Refusal::DecisionRequired);
        }
        schema::validate_command_payload(name, &envelope.payload)?;
        let command = match name {
            CommandName::SubmitPrompt => Command::SubmitPrompt(envelope.decode_payload()?),
            CommandName::SpawnSubHive => Command::SpawnSubHive(envelope.decode_payload()?),
            CommandName::RewireTask => Command::RewireTask(envelope.decode_payload()?),
            CommandName::CancelRun => Command::CancelRun(envelope.decode_payload()?),
            CommandName::KillWorker => Command::KillWorker(envelope.decode_payload()?),
            CommandName::ApprovalResolve => Command::ApprovalResolve(envelope.decode_payload()?),
            CommandName::SettingsStatus => Command::SettingsStatus(envelope.decode_payload()?),
            CommandName::FileTreeList => Command::FileTreeList(envelope.decode_payload()?),
            CommandName::FileRead => Command::FileRead(envelope.decode_payload()?),
            CommandName::ChatPin => Command::ChatPin(envelope.decode_payload()?),
            CommandName::TemplateInsert => Command::TemplateInsert(envelope.decode_payload()?),
            CommandName::HistoryRevisionSet => {
                Command::HistoryRevisionSet(envelope.decode_payload()?)
            }
            CommandName::ViewportSet => Command::ViewportSet(envelope.decode_payload()?),
            CommandName::ViewportRecenter => Command::ViewportRecenter(envelope.decode_payload()?),
            CommandName::DiffPropose | CommandName::DiffApply | CommandName::DiffReject => {
                return Err(Refusal::DecisionRequired);
            }
        };
        command.validate()?;
        Ok(command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_names_match_the_frozen_vocabulary() {
        let frozen = [
            "SUBMIT_PROMPT",
            "SPAWN_SUB_HIVE",
            "REWIRE_TASK",
            "CANCEL_RUN",
            "KILL_WORKER",
            "APPROVAL_RESOLVE",
            "SETTINGS_STATUS",
            "FILE_TREE_LIST",
            "FILE_READ",
            "CHAT_PIN",
            "TEMPLATE_INSERT",
            "HISTORY_REVISION_SET",
            "VIEWPORT_SET",
            "VIEWPORT_RECENTER",
            "DIFF_PROPOSE",
            "DIFF_APPLY",
            "DIFF_REJECT",
        ];
        assert_eq!(CommandName::ALL.len(), frozen.len());
        for (name, expected) in CommandName::ALL.iter().zip(frozen) {
            assert_eq!(name.as_str(), expected);
            assert_eq!(CommandName::from_name(expected), Some(*name));
        }
        assert_eq!(CommandName::from_name("NOT_A_COMMAND"), None);
    }

    #[test]
    fn approval_decision_values_are_frozen() {
        let sample = |decision: &str| {
            Command::ApprovalResolve(ApprovalResolve {
                approval_id: ApprovalId::new(),
                nonce: "nonce-1".to_owned(),
                decision: decision.to_owned(),
                reason: None,
            })
        };
        assert_eq!(sample("approve").validate(), Ok(()));
        assert_eq!(sample("deny").validate(), Ok(()));
        assert_eq!(
            sample("maybe").validate(),
            Err(Refusal::ApprovalDecisionInvalid)
        );
        assert_eq!(
            sample("reject").validate(),
            Err(Refusal::ApprovalDecisionInvalid),
            "the frozen vocabulary is approve/deny, not reject"
        );
    }

    #[test]
    fn file_read_requires_a_path() {
        let empty = Command::FileRead(FileRead {
            path: "  ".to_owned(),
        });
        assert_eq!(empty.validate(), Err(Refusal::SchemaInvalid));
        let valid = Command::FileRead(FileRead {
            path: "src/lib.rs".to_owned(),
        });
        assert_eq!(valid.validate(), Ok(()));
    }

    #[test]
    fn prompt_bounds_are_enforced() {
        let empty = Command::SubmitPrompt(SubmitPrompt {
            text: String::new(),
            project_id: "p".into(),
        });
        assert_eq!(empty.validate(), Err(Refusal::PromptEmpty));
        let too_long = Command::SubmitPrompt(SubmitPrompt {
            text: "x".repeat(MAX_PROMPT_CHARS + 1),
            project_id: "p".into(),
        });
        assert_eq!(too_long.validate(), Err(Refusal::PromptTooLong));
        let exact = Command::SubmitPrompt(SubmitPrompt {
            text: "x".repeat(MAX_PROMPT_CHARS),
            project_id: "p".into(),
        });
        assert_eq!(exact.validate(), Ok(()));
    }

    #[test]
    fn zoom_bounds_are_enforced() {
        let base = |zoom| {
            Command::ViewportSet(ViewportSet {
                x: 0.0,
                y: 0.0,
                zoom,
            })
        };
        assert_eq!(base(0.1).validate(), Ok(()));
        assert_eq!(base(3.0).validate(), Ok(()));
        assert_eq!(base(0.05).validate(), Err(Refusal::ZoomRange));
        assert_eq!(base(3.1).validate(), Err(Refusal::ZoomRange));
        assert_eq!(base(f64::NAN).validate(), Err(Refusal::SchemaInvalid));
    }
}
