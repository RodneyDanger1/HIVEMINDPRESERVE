//! API-key handling.
//!
//! The one DeepSeek account key is read from the process environment
//! (`DEEPSEEK_API_KEY`, fact F1/D2) into [`zeroize::Zeroizing`] memory. The
//! value is never logged, never committed, never serialised and never included
//! in an error: `Debug` prints `ApiKey(***)`, and [`ApiKey::expose`] is the only
//! accessor, used solely to build the `Authorization` header.

use std::env;

use hiverouter_contract::Refusal;
use zeroize::Zeroizing;

use crate::error::ProviderError;

/// Environment variable that carries the single DeepSeek API key.
pub const API_KEY_ENV: &str = "DEEPSEEK_API_KEY";

/// The one provider credential, zeroized on drop.
pub struct ApiKey {
    value: Zeroizing<String>,
}

impl ApiKey {
    /// Reads the key from [`API_KEY_ENV`].
    ///
    /// # Errors
    /// Returns [`Refusal::KeyMissing`] when the variable is absent or contains
    /// only whitespace. The value itself is never echoed.
    pub fn from_env() -> Result<Self, ProviderError> {
        let raw = env::var(API_KEY_ENV).map_err(|_| {
            ProviderError::new(
                Refusal::KeyMissing,
                format!("{API_KEY_ENV} is not set in the process environment"),
            )
        })?;
        Self::from_secret(&raw)
    }

    /// Builds a key from an already-read secret.
    ///
    /// Tests use a clearly fake value; production reads [`ApiKey::from_env`].
    ///
    /// # Errors
    /// Returns [`Refusal::KeyMissing`] when `value` is empty or whitespace.
    pub fn from_secret(value: &str) -> Result<Self, ProviderError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ProviderError::new(
                Refusal::KeyMissing,
                format!("{API_KEY_ENV} is present but empty"),
            ));
        }
        Ok(Self {
            value: Zeroizing::new(trimmed.to_owned()),
        })
    }

    /// Whether the process environment carries a usable key (no value exposed).
    #[must_use]
    pub fn present_in_env() -> bool {
        env::var(API_KEY_ENV)
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
    }

    /// Borrows the secret to build the bearer header.
    ///
    /// This is the only accessor; callers must not log, format or persist the
    /// returned value.
    #[must_use]
    pub(crate) fn expose(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ApiKey(***)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_never_prints_key_bytes() {
        let key = ApiKey::from_secret("sk-unit-test-only-value").expect("non-empty key");
        let debug = format!("{key:?}");
        assert_eq!(debug, "ApiKey(***)");
        assert!(!debug.contains("sk-unit-test"));
        // The value must not leak through the public error rendering either.
        let error = ApiKey::from_secret("   ").expect_err("whitespace rejected");
        assert_eq!(error.code(), "E_KEY_MISSING");
        assert!(!error.to_string().contains("sk-"));
    }

    #[test]
    fn empty_key_is_a_named_refusal() {
        let error = ApiKey::from_secret("").expect_err("empty rejected");
        assert_eq!(error.refusal(), Refusal::KeyMissing);
    }
}
