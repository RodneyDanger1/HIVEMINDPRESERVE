//! Adapter run-path matrix without a live provider: a local wiremock server
//! proves valid plan parsing and the named error mappings.

use std::sync::Arc;

use hiverouter_contract::{Refusal, RunId};
use hiverouter_core::config::ProviderSettings;
use hiverouter_core::execution::ProviderExecutor;
use hiverouter_core::provider_executor::ProviderExecutorAdapter;
use hiverouter_provider::client::{ClientConfig, DeepSeekClient};
use hiverouter_provider::secret::ApiKey;
use serde_json::json;
use tokio_util::sync::CancellationToken;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn adapter(server: &MockServer) -> ProviderExecutorAdapter {
    let client = DeepSeekClient::new(
        ClientConfig::default().with_base_url(server.uri()),
        ApiKey::from_secret("sk-wiremock-test-only").expect("test key"),
    )
    .expect("client");
    ProviderExecutorAdapter::new(client, ProviderSettings::default())
}

fn success_body(content: &str) -> serde_json::Value {
    json!({
        "id": "resp-test",
        "model": "deepseek-flash",
        "choices": [{
            "index": 0,
            "message": { "role": "assistant", "content": content },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 5,
            "total_tokens": 15,
            "prompt_cache_hit_tokens": 0,
            "prompt_cache_miss_tokens": 10,
            "prompt_tokens_details": { "cached_tokens": 0 }
        }
    })
}

#[tokio::test]
async fn valid_plan_json_is_mapped_and_validated() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(success_body(
            r#"{"nodes":[{"id":"n1","role":"worker","deps":[],"depth":1}]}"#,
        )))
        .mount(&server)
        .await;
    let plan = adapter(&server)
        .plan(RunId::new(), "plan this", &CancellationToken::new())
        .await
        .expect("plan");
    assert_eq!(plan.nodes.len(), 1);
    assert_eq!(plan.nodes[0].role, "worker");
}

#[tokio::test]
async fn malformed_plan_is_a_named_schema_refusal() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(success_body("not-json")))
        .mount(&server)
        .await;
    let error = adapter(&server)
        .plan(RunId::new(), "plan this", &CancellationToken::new())
        .await
        .expect_err("malformed plan");
    assert_eq!(error, Refusal::SchemaInvalid);
}

#[tokio::test]
async fn auth_status_is_mapped_to_provider_auth() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": { "message": "invalid key" }
        })))
        .mount(&server)
        .await;
    let error = adapter(&server)
        .plan(RunId::new(), "plan this", &CancellationToken::new())
        .await
        .expect_err("401");
    assert_eq!(error, Refusal::ProviderAuth);
}

#[tokio::test]
async fn payment_required_is_mapped_to_provider_auth() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(402).set_body_json(json!({
            "error": { "message": "billing" }
        })))
        .mount(&server)
        .await;
    let error = adapter(&server)
        .plan(RunId::new(), "plan this", &CancellationToken::new())
        .await
        .expect_err("402");
    assert_eq!(error, Refusal::ProviderAuth);
}

#[tokio::test]
async fn usage_absent_is_incomplete_not_zero() {
    let server = MockServer::start().await;
    let body = json!({
        "id": "resp-no-usage",
        "model": "deepseek-flash",
        "choices": [{
            "index": 0,
            "message": { "role": "assistant", "content": "hello" },
            "finish_reason": "stop"
        }]
    });
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;
    let error = adapter(&server)
        .plan(RunId::new(), "plan this", &CancellationToken::new())
        .await
        .expect_err("usage absent");
    assert_eq!(error, Refusal::UsageIncomplete);
}

#[test]
fn adapter_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Arc<ProviderExecutorAdapter>>();
}

