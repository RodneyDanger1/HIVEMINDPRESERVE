//! Typed provider failures.
//!
//! Every error that can reach a caller carries a canonical
//! [`hiverouter_contract::Refusal`] identity (`E_*` code + message +
//! how_to_fix + retryable). The wrapper only adds transport context that the
//! refusal table does not encode (HTTP status, Retry-After, sanitised detail).
//! There is no second refusal vocabulary in this crate.
//!
//! Secrets never appear here: `detail` is built from static text and status
//! codes only, never from request headers or bodies.

use std::fmt;
use std::time::Duration;

use hiverouter_contract::Refusal;

/// A provider call failure with its stable refusal identity.
#[derive(Debug, Clone)]
pub struct ProviderError {
    refusal: Refusal,
    detail: String,
    http_status: Option<u16>,
    retry_after: Option<Duration>,
}

impl ProviderError {
    /// Builds an error from a named refusal plus non-secret context.
    #[must_use]
    pub fn new(refusal: Refusal, detail: impl Into<String>) -> Self {
        Self {
            refusal,
            detail: detail.into(),
            http_status: None,
            retry_after: None,
        }
    }

    /// Adds the HTTP status that produced this failure (never the body).
    #[must_use]
    pub fn with_status(mut self, status: u16) -> Self {
        self.http_status = Some(status);
        self
    }

    /// Adds the provider's requested backoff duration (already capped).
    #[must_use]
    pub fn with_retry_after(mut self, retry_after: Duration) -> Self {
        self.retry_after = Some(retry_after);
        self
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

    /// Non-secret context for logs and diagnosis.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }

    /// HTTP status when the failure came from a response.
    #[must_use]
    pub fn http_status(&self) -> Option<u16> {
        self.http_status
    }

    /// Provider-requested backoff, capped by policy.
    #[must_use]
    pub fn retry_after(&self) -> Option<Duration> {
        self.retry_after
    }

    /// Whether retrying the same request may succeed without a change.
    #[must_use]
    pub fn retryable(&self) -> bool {
        self.refusal.retryable()
    }

    /// Log-safe one-line rendering (code + detail; no secrets, no bodies).
    #[must_use]
    pub fn log_line(&self) -> String {
        match (self.http_status, self.retry_after) {
            (Some(status), Some(retry_after)) => format!(
                "{} status={status} retry_after_ms={} detail={}",
                self.refusal.code(),
                retry_after.as_millis(),
                self.detail
            ),
            (Some(status), None) => {
                format!("{} status={status} detail={}", self.refusal.code(), self.detail)
            }
            (None, Some(retry_after)) => format!(
                "{} retry_after_ms={} detail={}",
                self.refusal.code(),
                retry_after.as_millis(),
                self.detail
            ),
            (None, None) => format!("{} detail={}", self.refusal.code(), self.detail),
        }
    }
}

impl fmt::Display for ProviderError {
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

impl std::error::Error for ProviderError {}

impl From<Refusal> for ProviderError {
    fn from(refusal: Refusal) -> Self {
        Self::new(refusal, refusal.message())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_carries_code_and_never_invents_one() {
        let error = ProviderError::new(Refusal::ProviderTimeout, "connect timeout after 10000ms")
            .with_status(0);
        assert_eq!(error.code(), "E_PROVIDER_TIMEOUT");
        assert_eq!(error.refusal(), Refusal::ProviderTimeout);
        assert!(error.retryable());
        assert!(!error.log_line().contains("sk-"), "no key material in log line");
    }

    #[test]
    fn refusal_conversion_preserves_identity() {
        let error: ProviderError = Refusal::UsageIncomplete.into();
        assert_eq!(error.code(), "E_USAGE_INCOMPLETE");
        assert!(!error.retryable());
    }
}
