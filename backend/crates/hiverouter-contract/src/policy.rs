//! Permission policy for a run. This is not a product mode.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Permission policy for a run.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PermissionPolicy {
    /// Current behavior: every Mutate/Execute/Network asks.
    #[default]
    Classic,
    /// Read-only: no writes or commands.
    Plan,
    /// Workspace edits auto-approved; shell/network ask.
    Standard,
    /// Explicit opt-in: full local coding without per-action prompts.
    Trusted,
}

impl PermissionPolicy {
    /// Stable database/wire label.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Plan => "plan",
            Self::Standard => "standard",
            Self::Trusted => "trusted",
        }
    }

    /// Strict parser; returns `None` for unknown values.
    #[must_use]
    pub fn from_str_strict(value: &str) -> Option<Self> {
        match value {
            "classic" => Some(Self::Classic),
            "plan" => Some(Self::Plan),
            "standard" => Some(Self::Standard),
            "trusted" => Some(Self::Trusted),
            _ => None,
        }
    }

    /// Compatibility parser used by the store; unknown values fall back to
    /// `classic` so an old database row can never silently become more permissive.
    #[must_use]
    pub fn from_str(value: &str) -> Self {
        Self::from_str_strict(value).unwrap_or(Self::Classic)
    }
}
