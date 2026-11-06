//! W1 core-recovery gates: the optional provider-key boot check must be
//! truthful and secret-free.
//!
//! Pins the env → bool → `Check` boundary used by `main.rs` and served by
//! `/health`: only a presence boolean may cross into `checks.rs`; a secret value
//! in `DEEPSEEK_API_KEY` must never appear in a `Check` detail or in its
//! serialized JSON. The absent-key refusal must stay `E_KEY_MISSING` on the wire
//! and must stay `required = false` (the server boots degraded, it does not
//! fail closed on a missing key).
//!
//! Reference (not duplicated here because that file is outside this seat's write
//! scope): the rejected truncation path is covered by the existing wiremock test
//! `tests/provider_streaming.rs::length_finish_reason_is_incomplete_not_success`,
//! which streams `finish_reason: "length"` and asserts `E_PROVIDER_INCOMPLETE`.

use hiverouter_core::checks::{provider_key_check, Check};
use hiverouter_provider::ApiKey;
use serde_json::Value;

/// Secret-shaped canary: if any path captures the key value, this string would
/// show up in a `Check.detail` or in the serialized check payload.
const CANARY: &str = "sk-w1-canary-never-emit-9f3c1b7e";

const KEY_ENV: &str = "DEEPSEEK_API_KEY";

/// Saves and restores the process environment value so the test is
/// deterministic and leaves no canary behind.
struct EnvGuard {
    previous: Option<String>,
}

impl EnvGuard {
    fn install_canary() -> Self {
        let previous = std::env::var(KEY_ENV).ok();
        std::env::set_var(KEY_ENV, CANARY);
        Self { previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match self.previous.take() {
            Some(value) => std::env::set_var(KEY_ENV, value),
            None => std::env::remove_var(KEY_ENV),
        }
    }
}

fn serialized(check: &Check) -> Value {
    serde_json::to_value(check).expect("Check derives Serialize")
}

fn assert_secret_free(value: &Value) {
    let encoded = value.to_string();
    assert!(
        !encoded.contains(CANARY),
        "serialized check contains the key value: {encoded}"
    );
    assert!(!encoded.contains("sk-"), "serialized check contains a secret shape: {encoded}");
    assert!(!encoded.contains("Bearer"), "serialized check contains a bearer header: {encoded}");
}

#[test]
fn provider_key_check_is_optional_when_provider_is_disabled() {
    assert!(provider_key_check(false, true).is_none());
    assert!(provider_key_check(false, false).is_none());
}

#[test]
fn provider_key_check_states_never_expose_a_secret() {
    let _guard = EnvGuard::install_canary();
    let present = ApiKey::present_in_env();
    assert!(present, "the canary value must be observed as present");

    let disabled = provider_key_check(false, present);
    assert!(disabled.is_none(), "a disabled provider must not report a key check");

    let enabled_absent = provider_key_check(true, false).expect("enabled absent check");
    let enabled_present = provider_key_check(true, present).expect("enabled present check");

    for check in [&enabled_absent, &enabled_present] {
        assert!(
            check.detail.contains(KEY_ENV),
            "the detail must name the environment variable: {}",
            check.detail
        );
        assert!(
            !check.detail.contains(CANARY),
            "the detail must never carry the key value: {}",
            check.detail
        );
        let value = serialized(check);
        assert_secret_free(&value);
    }
}

#[test]
fn absent_key_check_serializes_named_refusal_and_is_not_required() {
    let absent = provider_key_check(true, false).expect("enabled absent check");
    let value = serialized(&absent);
    assert_eq!(value["name"], "deepseek_api_key");
    assert_eq!(value["required"], false);
    assert_eq!(value["ok"], false);
    assert_eq!(value["code"], "E_KEY_MISSING");
    let detail = value["detail"].as_str().expect("string detail");
    assert!(
        detail.contains("E_KEY_MISSING"),
        "the detail must carry the named refusal: {detail}"
    );
    assert_secret_free(&value);

    let present = provider_key_check(true, true).expect("enabled present check");
    let value = serialized(&present);
    assert_eq!(value["name"], "deepseek_api_key");
    assert_eq!(value["required"], false);
    assert_eq!(value["ok"], true);
    assert!(value["code"].is_null(), "a passing check carries no refusal code");
    assert_secret_free(&value);
}
