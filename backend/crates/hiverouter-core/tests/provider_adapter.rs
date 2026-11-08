//! Adapter run-path matrix without a live provider: a local wiremock server
//! proves valid plan parsing and the named error mappings.

use std::sync::Arc;

use hiverouter_contract::{AgentState, Refusal, RunId, TaskId};
use hiverouter_core::config::ProviderSettings;
use hiverouter_core::execution::{DeltaSink, ProviderExecutor};
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
async fn leaf_execute_advertises_tools_on_the_first_request() {
    let server = MockServer::start().await;
    let body = concat!(
        "data: {\"id\":\"c1\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"ok\"},\"finish_reason\":null}]}\n\n",
        "data: {\"id\":\"c1\",\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":10,\"completion_tokens\":2,\"total_tokens\":12,\"prompt_cache_hit_tokens\":0,\"prompt_cache_miss_tokens\":10}}\n\n",
        "data: [DONE]\n\n",
    );
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(body),
        )
        .mount(&server)
        .await;

    let task = hiverouter_core::entities::TaskRecord {
        id: TaskId::new(),
        run_id: RunId::new(),
        parent_task_id: None,
        role: "worker".to_owned(),
        depth: 1,
        spawn_seq: 1,
        state: AgentState::Running,
        rev: 0,
    };
    let cancel = CancellationToken::new();
    let sink: DeltaSink = Arc::new(|_| {});
    adapter(&server)
        .execute(RunId::new(), &task, "read a file", &cancel, &sink)
        .await
        .expect("leaf execute");

    let requests = server.received_requests().await.expect("captured requests");
    let first = requests.first().expect("one request");
    let body: serde_json::Value = serde_json::from_slice(&first.body).expect("request JSON");
    let tools = body.get("tools").and_then(|value| value.as_array()).expect("tools array");
    assert!(!tools.is_empty(), "the first leaf request must advertise tool specs");
    assert_eq!(body.get("tool_choice"), Some(&json!("auto")));
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
async fn payment_required_is_mapped_to_provider_balance() {
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
    assert_eq!(error, Refusal::ProviderBalance);
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

