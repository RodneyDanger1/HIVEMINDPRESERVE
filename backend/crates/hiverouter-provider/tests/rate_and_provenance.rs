//! Request-rate governor and external cost-provenance tests.
//!
//! Two live-failure classes are pinned here without any live call:
//!
//! - `E_LLM_RATE_LIMIT` storm: every HTTP attempt (retries included) takes a
//!   permit from a shared sliding-window governor, waits only up to a bounded
//!   ceiling, otherwise refuses by name; 429 `Retry-After` is honoured and the
//!   retry count is bounded.
//! - Cost honesty: external OpenAI-compatible usage is measured but never
//!   settled as DeepSeek USD (`UNPRICED`, `settled_usd: null`), while token and
//!   call caps still work and DeepSeek pricing is byte-for-byte unchanged.
//!
//! Wiremock is local; the API key fixture is an obvious offline placeholder.

use std::str::FromStr;
use std::time::{Duration, Instant};

use hiverouter_contract::Usage;
use hiverouter_provider::accounting::{CostMode, Provenance, RunAccounting};
use hiverouter_provider::budget::{
    BudgetLimits, CallEstimate, RateLimitPolicy, RequestRateGovernor, RetryPolicy, RunBudget,
    EXTERNAL_ENDPOINT_REQUESTS_PER_MINUTE,
};
use hiverouter_provider::{ApiKey, ChatMessage, ChatRequest, ClientConfig, DeepSeekClient, Model};
use rust_decimal::Decimal;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const TEST_KEY: &str = "sk-wiremock-offline-only";

fn request() -> ChatRequest {
    ChatRequest::new(
        Model::Flash,
        vec![ChatMessage::user("rate/provenance matrix")],
    )
}

fn usage(hit: u64, miss: u64, completion: u64) -> Usage {
    Usage {
        prompt_tokens: Some(hit + miss),
        completion_tokens: Some(completion),
        total_tokens: Some(hit + miss + completion),
        prompt_cache_hit_tokens: Some(hit),
        prompt_cache_miss_tokens: Some(miss),
        reasoning_tokens: None,
    }
}

/// External Token Lab shape: prompt/completion totals, no DeepSeek cache split.
fn external_usage() -> Usage {
    Usage {
        prompt_tokens: Some(120),
        completion_tokens: Some(30),
        total_tokens: Some(150),
        prompt_cache_hit_tokens: None,
        prompt_cache_miss_tokens: None,
        reasoning_tokens: None,
    }
}

fn client_with_rate(server: &MockServer, policy: RateLimitPolicy) -> DeepSeekClient {
    let config = ClientConfig::default()
        .with_base_url(server.uri())
        .with_rate_limit(policy);
    DeepSeekClient::new(
        config,
        ApiKey::from_secret(TEST_KEY).expect("offline test key"),
    )
    .expect("client")
}

// ---------------------------------------------------------------------------
// Governor accounting with an injected clock (no real waiting).
// ---------------------------------------------------------------------------

#[test]
fn governor_counts_permits_per_window_with_injected_clock() {
    let governor = RequestRateGovernor::new(RateLimitPolicy::per_minute(3));
    let t0 = Duration::ZERO;
    for grant in 0..3 {
        assert!(
            governor.try_acquire_at(t0).is_ok(),
            "grant {grant} inside the window must succeed"
        );
    }
    let wait = governor
        .try_acquire_at(t0)
        .expect_err("fourth permit refused");
    assert_eq!(
        wait.wait,
        Duration::from_secs(60),
        "must wait a full window"
    );
    let wait = governor
        .try_acquire_at(Duration::from_secs(30))
        .expect_err("still inside the window");
    assert_eq!(wait.wait, Duration::from_secs(30), "wait shrinks with time");
    assert!(
        governor.try_acquire_at(Duration::from_secs(60)).is_ok(),
        "oldest grant leaves the 60 s window at t=60"
    );
    assert_eq!(
        governor.granted_total(),
        4,
        "exactly four permits were granted"
    );
}

#[test]
fn governor_disabled_policy_grants_without_waiting() {
    let governor = RequestRateGovernor::new(RateLimitPolicy::unlimited());
    for _ in 0..1_000 {
        governor
            .try_acquire_at(Duration::ZERO)
            .expect("unlimited grants every permit at the same instant");
    }
    assert_eq!(governor.granted_total(), 1_000);
}

#[tokio::test]
async fn governor_waits_for_short_window_and_refuses_too_far_permits() {
    // Short window proves real spacing without sleeping for a minute.
    let governor = RequestRateGovernor::new(
        RateLimitPolicy::per_minute(2)
            .with_window(Duration::from_millis(80))
            .with_max_wait(Duration::from_secs(1)),
    );
    governor.acquire().await.expect("permit 1");
    governor.acquire().await.expect("permit 2");
    let started = Instant::now();
    governor
        .acquire()
        .await
        .expect("permit 3 waits for the window");
    let elapsed = started.elapsed();
    assert!(
        elapsed >= Duration::from_millis(50),
        "third permit must wait for the window, measured {elapsed:?}"
    );
    assert!(elapsed < Duration::from_secs(1), "wait is bounded");

    // A permit further away than max_wait is a named refusal, not a queue.
    let refusing = RequestRateGovernor::new(
        RateLimitPolicy::per_minute(1).with_max_wait(Duration::from_millis(10)),
    );
    refusing.acquire().await.expect("first permit");
    let started = Instant::now();
    let error = refusing.acquire().await.expect_err("refused by name");
    assert_eq!(error.code(), "E_LLM_RATE_LIMIT");
    assert_eq!(error.http_status(), Some(429));
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "a refusal must not wait for the full window"
    );
}

// ---------------------------------------------------------------------------
// Governor is wired into every HTTP attempt, retries included.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn every_retry_takes_a_rate_permit() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(429))
        .expect(3)
        .mount(&server)
        .await;

    // Keep the retry backoff out of the measurement so the governor spacing is
    // what the elapsed time proves.
    let config = ClientConfig::default()
        .with_base_url(server.uri())
        .with_rate_limit(
            RateLimitPolicy::per_minute(2)
                .with_window(Duration::from_millis(150))
                .with_max_wait(Duration::from_secs(1)),
        )
        .with_retry(RetryPolicy {
            max_attempts: 3,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(1),
            jitter_max: Duration::ZERO,
        });
    let client = DeepSeekClient::new(
        config,
        ApiKey::from_secret(TEST_KEY).expect("offline test key"),
    )
    .expect("client");

    let started = Instant::now();
    let error = client.complete(&request()).await.expect_err("exhausted");
    assert_eq!(error.code(), "E_LLM_RATE_LIMIT");
    let elapsed = started.elapsed();
    assert!(
        elapsed >= Duration::from_millis(100),
        "the third attempt had to wait for the 150 ms window, measured {elapsed:?}"
    );
    assert_eq!(
        server
            .received_requests()
            .await
            .expect("wiremock records")
            .len(),
        3,
        "three attempts, each governed"
    );
}

// ---------------------------------------------------------------------------
// 429 Retry-After honoured + retries bounded.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn retry_after_is_honoured_and_retries_are_bounded() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "1"))
        .expect(3)
        .mount(&server)
        .await;

    let config = ClientConfig::default()
        .with_base_url(server.uri())
        .with_request_timeout(Duration::from_millis(500))
        .with_retry(RetryPolicy {
            max_attempts: 3,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(1),
            jitter_max: Duration::ZERO,
        });
    let client = DeepSeekClient::new(
        config,
        ApiKey::from_secret(TEST_KEY).expect("offline test key"),
    )
    .expect("client");

    let started = Instant::now();
    let error = client.complete(&request()).await.expect_err("exhausted");
    let elapsed = started.elapsed();
    assert_eq!(error.code(), "E_LLM_RATE_LIMIT");
    assert!(
        elapsed >= Duration::from_millis(1_800),
        "two retries must each honour the 1 s Retry-After, measured {elapsed:?}"
    );
    let attempts = server
        .received_requests()
        .await
        .expect("wiremock records")
        .len();
    assert_eq!(attempts, 3, "retries are capped at the 3-attempt budget");
}

// ---------------------------------------------------------------------------
// Cost provenance: external usage is measured, never settled as DeepSeek USD.
// ---------------------------------------------------------------------------

#[test]
fn external_usage_is_measured_but_never_settled_usd() {
    let mut accounting =
        RunAccounting::external(hiverouter_provider::pricing::PricingTable::documented());
    assert_eq!(accounting.mode(), CostMode::ExternalUnpriced);
    let cost = accounting
        .record(Model::Flash, Some(&external_usage()), 0)
        .expect("external usage without a cache split is accepted");
    assert_eq!(cost.usage_provenance, Provenance::Measured);
    assert_eq!(cost.amount_provenance, Provenance::Unpriced);
    assert!(cost.amount_usd.is_zero(), "no USD amount is derived");

    let report = accounting.report();
    assert_eq!(report.status, "UNPRICED");
    assert_eq!(report.settled_usd, None, "settled_usd must stay null");
    assert_eq!(report.amount_provenance, "unpriced");
    assert_eq!(report.prompt_tokens, Some(120));
    assert_eq!(report.completion_tokens, Some(30));
    assert_eq!(
        accounting
            .settled_usd()
            .expect_err("unpriced has no settlement"),
        hiverouter_contract::Refusal::PriceUnsourced
    );
}

#[test]
fn external_budget_keeps_token_and_call_caps_but_no_usd_cap() {
    let mut budget = RunBudget::new(BudgetLimits::external_endpoint());
    assert_eq!(budget.cost_mode(), CostMode::ExternalUnpriced);
    budget
        .record_call(Model::Flash, Some(&external_usage()), 0)
        .expect("measured usage recorded");
    assert_eq!(budget.spent_usd(), Decimal::ZERO, "no settled USD");
    assert_eq!(budget.report().status, "UNPRICED");
    assert_eq!(budget.report().settled_usd, None);

    // Token cap still gates an external send.
    let mut limits = BudgetLimits::external_endpoint();
    limits.token_budget_combined = 100;
    let token_budget = RunBudget::new(limits);
    let estimate = CallEstimate {
        prompt_tokens: Some(90),
        max_output_tokens: Some(20),
    };
    assert_eq!(
        token_budget
            .preflight(&estimate)
            .expect_err("token cap")
            .code(),
        "E_TOKEN_BUDGET_EXCEEDED"
    );

    // Call cap still gates an external send, retries included.
    let mut limits = BudgetLimits::external_endpoint();
    limits.call_cap = 1;
    let mut call_budget = RunBudget::new(limits);
    call_budget.record_attempt();
    assert_eq!(
        call_budget
            .preflight(&CallEstimate::default())
            .expect_err("call cap")
            .code(),
        "E_CALL_LIMIT"
    );
}

#[test]
fn deepseek_pricing_is_unchanged() {
    assert_eq!(BudgetLimits::default().cost_mode, CostMode::DeepSeekUsd);
    assert_eq!(
        RunBudget::new(BudgetLimits::default()).cost_mode(),
        CostMode::DeepSeekUsd
    );

    let mut budget = RunBudget::new(BudgetLimits::default());
    budget
        .record_call(Model::Flash, Some(&usage(40, 60, 50)), 0)
        .expect("DeepSeek usage settles");
    assert_eq!(
        budget.spent_usd(),
        Decimal::from_str("0.00003912").expect("decimal"),
        "documented DeepSeek pricing must be exact"
    );
    let report = budget.report();
    assert_eq!(report.status, "COMPLETE");
    assert_eq!(report.settled_usd.as_deref(), Some("0.00003912"));
    assert_eq!(report.amount_provenance, "derived");
    assert_eq!(
        RunBudget::new(BudgetLimits::default())
            .report()
            .settled_usd
            .as_deref(),
        Some("0"),
        "a fresh DeepSeek run still reports complete zero"
    );
}

// ---------------------------------------------------------------------------
// Repeated cumulative usage and transport-failure separation.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn repeated_cumulative_stream_usage_is_not_double_counted() {
    let server = MockServer::start().await;
    let chunk = r#"{"id":"c1","choices":[{"index":0,"delta":{},"finish_reason":"stop"}],"usage":{"prompt_tokens":7,"completion_tokens":3,"total_tokens":10}}"#;
    let body = format!("data: {chunk}\n\ndata: {chunk}\n\ndata: [DONE]\n\n");
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(body),
        )
        .mount(&server)
        .await;

    let client = client_with_rate(&server, RateLimitPolicy::unlimited());
    let mut handle = client.stream(&request()).await.expect("stream opens");
    let mut reported = None;
    while let Some(event) = handle.next().await {
        if let hiverouter_provider::StreamEvent::Finished { usage, .. } = event.expect("event") {
            reported = usage;
            break;
        }
    }
    let usage = reported.expect("usage reported");
    assert_eq!(
        usage.prompt_tokens,
        Some(7),
        "a repeated cumulative snapshot must not be summed twice"
    );
    assert_eq!(usage.completion_tokens, Some(3));
    assert_eq!(usage.total_tokens, Some(10));
}

#[test]
fn transport_failure_keeps_cost_null_but_does_not_poison_later_sends() {
    let mut budget = RunBudget::new(BudgetLimits::default());
    budget.record_transport_failure(hiverouter_contract::Refusal::ProviderIncomplete);
    assert_eq!(
        budget.transport_failure(),
        Some(hiverouter_contract::Refusal::ProviderIncomplete)
    );
    assert_eq!(
        budget.report().status,
        "INCOMPLETE",
        "unknown spend is never presented as settled"
    );
    assert_eq!(budget.report().settled_usd, None);
    assert_eq!(
        budget.report().incomplete_reason,
        Some("E_PROVIDER_INCOMPLETE")
    );
    budget
        .preflight(&CallEstimate::default())
        .expect("a transient transport failure must not fail every later sibling send");

    // A genuine missing-usage record still poisons the run.
    let mut poisoned = RunBudget::new(BudgetLimits::default());
    poisoned
        .record_call(Model::Flash, None, 0)
        .expect_err("null usage is incomplete");
    assert_eq!(
        poisoned
            .preflight(&CallEstimate::default())
            .expect_err("usage integrity failure still refuses")
            .code(),
        "E_USAGE_INCOMPLETE"
    );
}

#[test]
fn external_base_url_gets_documented_rate_policy_only() {
    let deepseek = ClientConfig::default();
    assert!(
        deepseek.rate_limit.policy().is_unlimited(),
        "DeepSeek default unchanged"
    );
    assert_eq!(deepseek.base_url, "https://api.deepseek.com");

    let external = ClientConfig::default().with_base_url("https://token-lab.example/v1");
    assert_eq!(
        external.rate_limit.policy().requests_per_minute,
        EXTERNAL_ENDPOINT_REQUESTS_PER_MINUTE,
        "an explicitly different base URL is an external endpoint"
    );
    assert_eq!(external.rate_limit.policy().requests_per_minute, 20);

    let overridden = ClientConfig::default()
        .with_base_url("https://token-lab.example/v1")
        .with_rate_limit(RateLimitPolicy::unlimited());
    assert!(
        overridden.rate_limit.policy().is_unlimited(),
        "an explicit policy overrides the automatic external one"
    );
}
