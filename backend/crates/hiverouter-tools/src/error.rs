//! Typed tool-layer failures.
//!
//! Every failure carries a canonical [`hiverouter_contract::Refusal`] identity
//! (`E_*` code + message + how_to_fix + retryable). The wrapper only adds
//! non-secret context (tool id, path class, status) for the run ledger and logs.
//! Argument bodies are never stored: the ledger keeps `args_sha256` only.

use std::fmt;

use hiverouter_contract::Refusal;

/// A tool-layer failure with its stable refusal identity.
#[derive(Debug, Clone)]
pub struct ToolError {
    refusal: Refusal,
    detail: String,
}

impl ToolError {
    /// Builds an error from a named refusal plus non-secret context.
    #[must_use]
    pub fn new(refusal: Refusal, detail: impl Into<String>) -> Self {
        Self {
            refusal,
            detail: detail.into(),
        }
    }

    /// The canonical refusal identity.
    #[must_use]
    pub fn refusal(&self) -> Refusal {
        self.refusal
    }

    /// Stable wire code (`E_*`).
    #[must_use]
    pub fn code(&self) -> &'static str {
        self.refusal.code()
    }

    /// Non-secret context for the ledger and diagnosis.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }

    /// Whether retrying the same operation may succeed without a change.
    #[must_use]
    pub fn retryable(&self) -> bool {
        self.refusal.retryable()
    }
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} ({})",
            self.refusal.code(),
            self.refusal.message(),
            self.detail
        )
    }
}

impl std::error::Error for ToolError {}

impl From<Refusal> for ToolError {
    fn from(refusal: Refusal) -> Self {
        Self::new(refusal, refusal.message())
    }
}

impl From<rusqlite::Error> for ToolError {
    fn from(error: rusqlite::Error) -> Self {
        Self::new(
            Refusal::InternalError,
            format!("SQLite failure while recording tool state: {error}"),
        )
    }
}

impl From<std::io::Error> for ToolError {
    fn from(error: std::io::Error) -> Self {
        Self::new(
            Refusal::InternalError,
            format!("I/O failure in the tool layer: {error}"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_come_from_the_frozen_table() {
        let error = ToolError::new(Refusal::JailEscape, "path outside the workspace root");
        assert_eq!(error.code(), "E_JAIL_ESCAPE");
        assert!(!error.retryable());
        assert_eq!(
            Refusal::from_code(error.code()),
            Some(Refusal::JailEscape),
            "no ad-hoc code can be constructed"
        );
    }
}
