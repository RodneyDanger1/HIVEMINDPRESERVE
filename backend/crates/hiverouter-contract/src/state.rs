//! The single agent-state enum (frozen contract §4, superset allowed).
//!
//! Every node state, run status and task state on the wire uses this enum, so
//! the UI has exactly one vocabulary to render.

use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// One agent/node/run state.
///
/// The frozen minimum set is 15 values (`healthy .. loading`); `Interrupted` is
/// a recorded superset addition needed by the Phase 9 crash-recovery read model
/// (a run killed mid-flight must be distinguishable from `cancelled`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    /// Node is healthy and idle.
    Healthy,
    /// Node is executing.
    Running,
    /// Node is queued and has not started.
    Queued,
    /// Node/run failed.
    Failed,
    /// Node/run completed.
    Completed,
    /// Run was cancelled by the user.
    Cancelled,
    /// Worker was killed by `KILL_WORKER`.
    Killed,
    /// Agent is planning (intent -> task DAG).
    Planning,
    /// Agent is thinking (model reasoning phase).
    Thinking,
    /// Agent is waiting for an approval decision.
    AwaitingApproval,
    /// Agent reply is streaming.
    Streaming,
    /// Client-side transport state: reconnecting.
    Reconnecting,
    /// Client-side transport state: replaying buffered events.
    Replay,
    /// No data yet.
    Empty,
    /// Data is loading.
    Loading,
    /// Run was interrupted by a crash/restart and recovered from a snapshot.
    Interrupted,
}

impl AgentState {
    /// Every state value (declaration order).
    pub const ALL: &'static [AgentState] = &[
        AgentState::Healthy,
        AgentState::Running,
        AgentState::Queued,
        AgentState::Failed,
        AgentState::Completed,
        AgentState::Cancelled,
        AgentState::Killed,
        AgentState::Planning,
        AgentState::Thinking,
        AgentState::AwaitingApproval,
        AgentState::Streaming,
        AgentState::Reconnecting,
        AgentState::Replay,
        AgentState::Empty,
        AgentState::Loading,
        AgentState::Interrupted,
    ];

    /// Stable snake-case wire value.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentState::Healthy => "healthy",
            AgentState::Running => "running",
            AgentState::Queued => "queued",
            AgentState::Failed => "failed",
            AgentState::Completed => "completed",
            AgentState::Cancelled => "cancelled",
            AgentState::Killed => "killed",
            AgentState::Planning => "planning",
            AgentState::Thinking => "thinking",
            AgentState::AwaitingApproval => "awaiting_approval",
            AgentState::Streaming => "streaming",
            AgentState::Reconnecting => "reconnecting",
            AgentState::Replay => "replay",
            AgentState::Empty => "empty",
            AgentState::Loading => "loading",
            AgentState::Interrupted => "interrupted",
        }
    }

    /// True when no further transition is expected without a new run.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            AgentState::Failed
                | AgentState::Completed
                | AgentState::Cancelled
                | AgentState::Killed
                | AgentState::Interrupted
        )
    }
}

impl fmt::Display for AgentState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_the_frozen_minimum_set() {
        let frozen = [
            "healthy",
            "running",
            "queued",
            "failed",
            "completed",
            "cancelled",
            "killed",
            "planning",
            "thinking",
            "awaiting_approval",
            "streaming",
            "reconnecting",
            "replay",
            "empty",
            "loading",
        ];
        for state in frozen {
            assert!(
                AgentState::ALL.iter().any(|known| known.as_str() == state),
                "missing frozen agent state {state}"
            );
        }
        assert_eq!(AgentState::ALL.len(), 16);
    }

    #[test]
    fn serializes_as_snake_case() {
        assert_eq!(serde_json::to_string(&AgentState::AwaitingApproval).expect("ser"), "\"awaiting_approval\"");
        let parsed: AgentState = serde_json::from_str("\"interrupted\"").expect("de");
        assert_eq!(parsed, AgentState::Interrupted);
        assert!(AgentState::Cancelled.is_terminal());
        assert!(!AgentState::Running.is_terminal());
    }
}
