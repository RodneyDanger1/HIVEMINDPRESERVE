//! Run, task and chat entities (blueprint Phase 4, WP 4.6).

use hiverouter_contract::{AgentState, RunId, TaskId, Timestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// One run: one user prompt and everything it spawns.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Run {
    /// Run id (uuid-v7).
    pub id: RunId,
    /// Project/workspace binding.
    pub project_id: String,
    /// Original user prompt.
    pub prompt: String,
    /// Lifecycle status (single agent-state enum).
    pub status: AgentState,
    /// Named refusal code when the run failed.
    pub error_code: Option<String>,
    /// Creation timestamp.
    pub created_at: Timestamp,
    /// Last status-change timestamp.
    pub updated_at: Timestamp,
}

/// One task row (one node in the run graph).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskRecord {
    /// Task id (uuid-v7).
    pub id: TaskId,
    /// Owning run.
    pub run_id: RunId,
    /// Parent task (`None` for the run root).
    pub parent_task_id: Option<TaskId>,
    /// Agent role/tier name.
    pub role: String,
    /// Spawn depth, 0 for the root, hard-capped at 3.
    pub depth: u8,
    /// Monotonic spawn sequence within the run.
    pub spawn_seq: u64,
    /// Lifecycle state.
    pub state: AgentState,
    /// Optimistic-concurrency revision.
    pub rev: u64,
}

/// Chat message author role.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatRole {
    /// End-user message.
    User,
    /// Assistant reply.
    Assistant,
    /// System instruction.
    System,
    /// Tool output.
    Tool,
}

impl ChatRole {
    /// Stable database/wire string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            ChatRole::User => "user",
            ChatRole::Assistant => "assistant",
            ChatRole::System => "system",
            ChatRole::Tool => "tool",
        }
    }

    /// Parses the stored role string.
    #[must_use]
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "user" => Some(ChatRole::User),
            "assistant" => Some(ChatRole::Assistant),
            "system" => Some(ChatRole::System),
            "tool" => Some(ChatRole::Tool),
            _ => None,
        }
    }
}

/// One stored chat message.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Message id (uuid-v7).
    pub id: Uuid,
    /// Owning run.
    pub run_id: RunId,
    /// Author role.
    pub role: ChatRole,
    /// Message text.
    pub text: String,
    /// Creation timestamp.
    pub created_at: Timestamp,
}
