//! Settings/key-status surface (contract patch: `SETTINGS_STATUS`).
//!
//! Returns provider/model identity and a **boolean** key presence only. The key
//! value is never read into a string here, never formatted and never returned:
//! the UI and the CLI both see `key_present`. The payload type comes from the
//! frozen contract crate (single source of truth, no duplicate DTO).

use hiverouter_contract::{event::SettingsStatusResult, Refusal};
use hiverouter_provider::{ApiKey, Model};

use crate::error::ToolError;

/// Default model id shown by settings (documented default: Flash).
pub const DEFAULT_MODEL: Model = Model::Flash;

/// Builds the `SETTINGS_STATUS_RESULT` payload without exposing the key.
#[must_use]
pub fn settings_status() -> SettingsStatusResult {
    SettingsStatusResult {
        provider: "deepseek".to_owned(),
        model: DEFAULT_MODEL.id().to_owned(),
        key_present: ApiKey::present_in_env(),
    }
}

/// Requires the key for an action that needs a live provider call.
///
/// # Errors
/// [`Refusal::KeyMissing`] with how_to_fix when `DEEPSEEK_API_KEY` is absent.
pub fn require_provider_key() -> Result<(), ToolError> {
    if ApiKey::present_in_env() {
        Ok(())
    } else {
        Err(ToolError::new(
            Refusal::KeyMissing,
            "DEEPSEEK_API_KEY is not set; live provider calls are refused by name",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_never_contains_a_key_value() {
        let status = settings_status();
        let rendered = serde_json::to_string(&status).expect("serialises");
        assert!(!rendered.contains("sk-"));
        assert!(!rendered.contains("Bearer "));
        assert_eq!(status.provider, "deepseek");
        assert_eq!(status.model, "deepseek-flash");
    }

    #[test]
    fn require_key_is_a_named_refusal_when_absent() {
        match require_provider_key() {
            Ok(()) => {}
            Err(error) => assert_eq!(error.code(), "E_KEY_MISSING"),
        }
    }
}
