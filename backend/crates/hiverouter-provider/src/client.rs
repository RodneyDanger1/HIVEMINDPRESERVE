//! The one DeepSeek Chat Completions client (FACT-REGISTER F1, MP-10).
//!
//! - Endpoint: `POST https://api.deepseek.com/chat/completions` (no documented
//!   `/v1` prefix).
//! - One key: read from `DEEPSEEK_API_KEY` into [`crate::secret::ApiKey`]
//!   (zeroized); never logged, committed or echoed.
//! - One local in-flight limiter ([`InFlightLimiter`], 8 permits). Provider
//!   concurrency is account-level (F1) and over-limit answers HTTP 429.
//! - Retries: ≤3 attempts, jittered exponential backoff, `Retry-After` capped
//!   at 60 s, **no retry on 401/402** (B8/B9).
//! - Streaming: documented SSE (`data:` frames, `[DONE]` terminator, keep-alive
//!   comments tolerated). The final chunk before `[DONE]` carries the usage
//!   statistics (documented); if the provider omits them, the caller's
//!   accounting marks the run INCOMPLETE (never 0). The stream is retried only
//!   before the first byte is delivered; once deltas reach the caller they are
//!   never replayed.
//! - No live call is made by this crate's tests; wiremock only.

use std::sync::Arc;
use std::time::Duration;

use hiverouter_contract::{Refusal, Usage};
use reqwest::header::{HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE, RETRY_AFTER};
use tokio::sync::mpsc;
use zeroize::Zeroizing;

use crate::budget::{
    retry_after_from_header_value, BudgetLimits, CallEstimate, InFlightLimiter, RateLimitPolicy,
    RequestRateGovernor, RetryPolicy, RunBudget,
};
use crate::dto::{
    BalanceResponse, ChatRequest, ChatResponse, Delta, ModelsResponse, StreamChunk,
    CHAT_COMPLETIONS_PATH, DEFAULT_BASE_URL,
};
use crate::error::ProviderError;
use crate::secret::ApiKey;

/// Client configuration (documented endpoint + timeouts).
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Base URL; default is the documented `https://api.deepseek.com`.
    pub base_url: String,
    /// TCP/TLS connect timeout.
    pub connect_timeout: Duration,
    /// Whole non-streaming request timeout. The provider closes a request when
    /// inference has not started within 10 minutes (F1), so 600 s is the
    /// documented ceiling for a non-streaming call.
    pub request_timeout: Duration,
    /// Maximum quiet period between streaming chunks (keep-alive tolerated).
    pub stream_idle_timeout: Duration,
    /// Retry policy (attempts/backoff; Retry-After capped at 60 s).
    pub retry: RetryPolicy,
    /// Local in-flight limiter shared by all calls of this client.
    pub in_flight: Arc<InFlightLimiter>,
    /// Shared request-rate governor applied to every HTTP attempt (including
    /// retries). Default: unlimited (documented DeepSeek behavior unchanged).
    pub rate_limit: Arc<RequestRateGovernor>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_owned(),
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(600),
            stream_idle_timeout: Duration::from_secs(600),
            retry: RetryPolicy::default(),
            in_flight: Arc::new(InFlightLimiter::new(crate::budget::IN_FLIGHT_LIMIT)),
            rate_limit: Arc::new(RequestRateGovernor::new(RateLimitPolicy::unlimited())),
        }
    }
}

impl ClientConfig {
    /// Overrides the base URL. A URL other than the documented DeepSeek base is
    /// an external OpenAI-compatible endpoint, so the documented external
    /// request-rate policy (20 requests/minute) is applied automatically when
    /// no explicit rate policy was set. DeepSeek (`DEFAULT_BASE_URL`) keeps the
    /// unchanged unlimited behavior. Call [`Self::with_rate_limit`] afterwards
    /// to override either choice.
    #[must_use]
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        if self.base_url.trim_end_matches('/') != DEFAULT_BASE_URL.trim_end_matches('/')
            && self.rate_limit.policy().is_unlimited()
        {
            self.rate_limit = Arc::new(RequestRateGovernor::new(
                RateLimitPolicy::external_endpoint(),
            ));
        }
        self
    }

    /// Overrides the request-rate policy (applies to every attempt, retries
    /// included).
    #[must_use]
    pub fn with_rate_limit(mut self, policy: RateLimitPolicy) -> Self {
        self.rate_limit = Arc::new(RequestRateGovernor::new(policy));
        self
    }

    /// Overrides the whole-request timeout.
    #[must_use]
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Overrides the per-call retry policy.
    #[must_use]
    pub fn with_retry(mut self, retry: RetryPolicy) -> Self {
        self.retry = retry;
        self
    }
}

/// Outcome of a non-streaming call.
#[derive(Debug, Clone)]
pub struct Completion {
    /// Parsed, validated response.
    pub response: ChatResponse,
    /// Provider usage counters when reported (absent stays absent).
    pub usage: Option<Usage>,
    /// Attempts used (1 = no retry).
    pub attempts: u32,
    /// HTTP status of the successful response.
    pub http_status: u16,
}

/// One parsed streaming event.
#[derive(Debug, Clone, PartialEq)]
pub enum StreamEvent {
    /// Text/reasoning delta.
    Delta {
        /// Assistant text fragment.
        content: String,
        /// Reasoner fragment, when the provider streams one.
        reasoning_content: Option<String>,
    },
    /// Tool-call fragment (assembled by the caller).
    ToolCallDelta {
        /// Index of the tool call in the message.
        index: u32,
        /// Tool call id (first fragment only).
        id: Option<String>,
        /// Function name (first fragment only).
        name: Option<String>,
        /// JSON arguments fragment.
        arguments: Option<String>,
    },
    /// Terminal event after `data: [DONE]`.
    Finished {
        /// Usage reported by the final content chunk (documented). Absent when
        /// the provider omits it; the caller must then record an INCOMPLETE
        /// run, never a zero cost.
        usage: Option<Usage>,
        /// Finish reason from the final choice chunk.
        finish_reason: Option<String>,
    },
}

/// A live streaming completion.
pub struct StreamHandle {
    receiver: mpsc::Receiver<Result<StreamEvent, ProviderError>>,
    /// Attempts used before the first byte arrived.
    pub attempts: u32,
}

impl StreamHandle {
    /// Next stream event.
    pub async fn next(&mut self) -> Option<Result<StreamEvent, ProviderError>> {
        self.receiver.recv().await
    }
}

/// A parsed SSE line (kept pure for tests).
#[derive(Debug, Clone, PartialEq)]
pub enum SseLine {
    /// A `data: {...}` chunk.
    Chunk(Box<StreamChunk>),
    /// A `data: [DONE]` terminator.
    Done,
    /// A `:` comment / keep-alive or a blank line.
    Ignored,
}

/// Parses one SSE line as documented by F1 (`: keep-alive` comments, blank
/// keep-alive lines, `data:` frames, `[DONE]`).
///
/// # Errors
/// [`Refusal::ProviderInvalidResponse`] when a `data:` frame is not documented
/// JSON.
pub fn parse_sse_line(line: &str) -> Result<SseLine, ProviderError> {
    let trimmed = line.trim_end_matches('\r').trim();
    if trimmed.is_empty() || trimmed.starts_with(':') {
        return Ok(SseLine::Ignored);
    }
    let Some(payload) = trimmed.strip_prefix("data:") else {
        return Ok(SseLine::Ignored);
    };
    let payload = payload.trim();
    if payload.is_empty() {
        return Ok(SseLine::Ignored);
    }
    if payload == "[DONE]" {
        return Ok(SseLine::Done);
    }
    let chunk: StreamChunk = serde_json::from_str(payload).map_err(|error| {
        ProviderError::new(
            Refusal::ProviderInvalidResponse,
            format!("stream chunk is not documented JSON: {error}"),
        )
    })?;
    Ok(SseLine::Chunk(Box::new(chunk)))
}

/// The single DeepSeek client.
pub struct DeepSeekClient {
    http: reqwest::Client,
    stream_http: reqwest::Client,
    key: ApiKey,
    config: ClientConfig,
}

impl std::fmt::Debug for DeepSeekClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeepSeekClient")
            .field("base_url", &self.config.base_url)
            .field("connect_timeout", &self.config.connect_timeout)
            .field("request_timeout", &self.config.request_timeout)
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

impl DeepSeekClient {
    /// Builds a client from a configuration and an explicit key.
    ///
    /// # Errors
    /// [`Refusal::InternalError`] when the HTTP client cannot be constructed
    /// (configuration/TLS initialisation failure).
    pub fn new(config: ClientConfig, key: ApiKey) -> Result<Self, ProviderError> {
        let http = reqwest::Client::builder()
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .build()
            .map_err(|error| {
                ProviderError::new(
                    Refusal::InternalError,
                    format!("failed to build the provider HTTP client: {error}"),
                )
            })?;
        let stream_http = reqwest::Client::builder()
            .connect_timeout(config.connect_timeout)
            .build()
            .map_err(|error| {
                ProviderError::new(
                    Refusal::InternalError,
                    format!("failed to build the streaming HTTP client: {error}"),
                )
            })?;
        Ok(Self {
            http,
            stream_http,
            key,
            config,
        })
    }

    /// Builds a client reading the one key from `DEEPSEEK_API_KEY`.
    ///
    /// # Errors
    /// [`Refusal::KeyMissing`] when the environment has no key.
    pub fn from_env(config: ClientConfig) -> Result<Self, ProviderError> {
        Self::new(config, ApiKey::from_env()?)
    }

    /// Builds a client reading the key from a named environment variable
    /// (`DEEPSEEK_API_KEY` for DeepSeek, `HIVEROUTER_API_KEY` for TokenLab).
    ///
    /// # Errors
    /// [`Refusal::KeyMissing`] when the named environment has no key.
    pub fn from_env_with_key_env(
        config: ClientConfig,
        key_env: &str,
    ) -> Result<Self, ProviderError> {
        Self::new(config, ApiKey::from_env_named(key_env)?)
    }

    /// Configured base URL.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }

    /// Reads the documented model catalogue (`GET /models`).
    ///
    /// # Errors
    /// Named [`ProviderError`] for auth, rate limit, transport or malformed
    /// response. The key is never included in the error.
    pub async fn list_models(&self) -> Result<ModelsResponse, ProviderError> {
        self.account_get::<ModelsResponse>("/models").await
    }

    /// Reads the documented account balance (`GET /user/balance`).
    ///
    /// # Errors
    /// Named [`ProviderError`] for auth, rate limit, transport or malformed
    /// response. The key is never included in the error.
    pub async fn balance(&self) -> Result<BalanceResponse, ProviderError> {
        self.account_get::<BalanceResponse>("/user/balance").await
    }

    async fn account_get<T>(&self, path: &str) -> Result<T, ProviderError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = format!("{}{path}", self.config.base_url.trim_end_matches('/'));
        let bearer = self.bearer_header()?;
        let response = self
            .http
            .get(url)
            .header(AUTHORIZATION, bearer)
            .header(ACCEPT, "application/json")
            .send()
            .await
            .map_err(|error| {
                map_transport_error(
                    &error,
                    self.config.request_timeout,
                    "reading account metadata",
                )
            })?;
        let response = self.classify_status(response)?;
        let status = response.status().as_u16();
        let body = response.text().await.map_err(|error| {
            map_transport_error(
                &error,
                self.config.request_timeout,
                "reading account metadata body",
            )
        })?;
        serde_json::from_str(&body).map_err(|error| {
            ProviderError::new(
                Refusal::ProviderInvalidResponse,
                format!(
                    "account metadata body is not the documented shape \
                     (json error line {} column {}; body bytes {})",
                    error.line(),
                    error.column(),
                    body.len()
                ),
            )
            .with_status(status)
        })
    }

    /// The documented model ids this client may call.
    #[must_use]
    pub fn model_ids(&self) -> Vec<&'static str> {
        crate::dto::Model::ALL
            .iter()
            .map(|model| model.id())
            .collect()
    }

    fn endpoint(&self) -> String {
        format!(
            "{}{}",
            self.config.base_url.trim_end_matches('/'),
            CHAT_COMPLETIONS_PATH
        )
    }

    fn bearer_header(&self) -> Result<HeaderValue, ProviderError> {
        let mut value = Zeroizing::new(String::with_capacity(7 + self.key.expose().len()));
        value.push_str("Bearer ");
        value.push_str(self.key.expose());
        HeaderValue::from_str(&value).map_err(|_| {
            ProviderError::new(
                Refusal::KeyMissing,
                "the configured API key is not a valid HTTP header value",
            )
        })
    }

    /// Non-streaming chat completion with bounded retries.
    ///
    /// # Errors
    /// - [`Refusal::SchemaInvalid`] before any send for an invalid request.
    /// - [`Refusal::ProviderAuth`] for HTTP 401/402 (never retried).
    /// - [`Refusal::LlmRateLimit`] for HTTP 429 after the retry budget.
    /// - [`Refusal::ProviderTimeout`] for socket/connect/read failures.
    /// - [`Refusal::ProviderInvalidResponse`] for a body that is not the
    ///   documented response shape.
    pub async fn complete(&self, request: &ChatRequest) -> Result<Completion, ProviderError> {
        self.drive(request, None, &CallEstimate::default(), 0).await
    }

    /// Non-streaming completion that enforces the run budget **before** every
    /// send and records the measured usage afterwards.
    ///
    /// # Errors
    /// Propagates the budget refusals (with zero provider calls when the first
    /// preflight refuses) and [`Refusal::UsageIncomplete`] when the provider
    /// omitted usage (the run is then INCOMPLETE).
    pub async fn complete_with_budget(
        &self,
        budget: &mut RunBudget,
        request: &ChatRequest,
        estimate: &CallEstimate,
        at_unix_secs: u64,
    ) -> Result<Completion, ProviderError> {
        self.drive(request, Some(budget), estimate, at_unix_secs)
            .await
    }

    async fn drive(
        &self,
        request: &ChatRequest,
        mut budget: Option<&mut RunBudget>,
        estimate: &CallEstimate,
        at_unix_secs: u64,
    ) -> Result<Completion, ProviderError> {
        request.validate()?;
        let _permit = self.config.in_flight.acquire().await?;

        let mut attempt = 0u32;
        loop {
            attempt = attempt.saturating_add(1);
            if let Some(active_budget) = budget.as_deref_mut() {
                active_budget.preflight(estimate)?;
            }
            // Request-rate permit for this HTTP attempt (retries included).
            // It is taken before record_attempt so a governor refusal is not
            // counted as a provider send.
            self.config.rate_limit.acquire().await?;
            if let Some(active_budget) = budget.as_deref_mut() {
                active_budget.record_attempt();
            }
            let result = self.send_once(request).await;
            match result {
                Ok((response, status)) => {
                    let usage = response.contract_usage();
                    if let Some(active_budget) = budget.as_deref_mut() {
                        active_budget.record_call(
                            request.accounting_model,
                            usage.as_ref(),
                            at_unix_secs,
                        )?;
                    }
                    return Ok(Completion {
                        response,
                        usage,
                        attempts: attempt,
                        http_status: status,
                    });
                }
                Err(error) => {
                    let retry_budget_left = attempt < self.config.retry.max_attempts;
                    if !error.retryable() || !retry_budget_left {
                        return Err(self.final_error(error));
                    }
                    let delay = self.config.retry.delay_for(attempt, error.retry_after());
                    tracing::warn!(
                        code = error.code(),
                        attempt,
                        delay_ms = u64::try_from(delay.as_millis()).unwrap_or(u64::MAX),
                        "provider call failed; retrying within policy"
                    );
                    if let Some(active_budget) = budget.as_deref_mut() {
                        active_budget.on_retry();
                    }
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    /// Maps an exhausted retry budget to the final named refusal.
    fn final_error(&self, error: ProviderError) -> ProviderError {
        match error.refusal() {
            Refusal::ProviderRateLimit => ProviderError::new(
                Refusal::LlmRateLimit,
                format!(
                    "HTTP 429 persisted after {} attempts ({}); retry after the window",
                    self.config.retry.max_attempts,
                    error.detail()
                ),
            )
            .with_status(429),
            _ => error,
        }
    }

    async fn send_once(&self, request: &ChatRequest) -> Result<(ChatResponse, u16), ProviderError> {
        let response = self.open(&self.http, request).await?;
        let status = response.status().as_u16();
        let body = response.text().await.map_err(|error| {
            map_transport_error(
                &error,
                self.config.request_timeout,
                "reading the response body",
            )
        })?;
        let parsed: ChatResponse = serde_json::from_str(&body).map_err(|error| {
            ProviderError::new(
                Refusal::ProviderInvalidResponse,
                format!(
                    "response body is not the documented chat completion shape \
                     (json error line {} column {}; body bytes {})",
                    error.line(),
                    error.column(),
                    body.len()
                ),
            )
            .with_status(status)
        })?;
        if parsed.choices.is_empty() {
            return Err(ProviderError::new(
                Refusal::ProviderInvalidResponse,
                "response contains no choices",
            )
            .with_status(status));
        }
        Ok((parsed, status))
    }

    async fn open(
        &self,
        http: &reqwest::Client,
        request: &ChatRequest,
    ) -> Result<reqwest::Response, ProviderError> {
        let bearer = self.bearer_header()?;
        let response = http
            .post(self.endpoint())
            .header(AUTHORIZATION, bearer)
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .json(request)
            .send()
            .await
            .map_err(|error| {
                map_transport_error(&error, self.config.request_timeout, "sending the request")
            })?;
        self.classify_status(response)
    }

    fn classify_status(
        &self,
        response: reqwest::Response,
    ) -> Result<reqwest::Response, ProviderError> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        let retry_after = retry_after_from_header_value(
            response
                .headers()
                .get(RETRY_AFTER)
                .and_then(|value| value.to_str().ok()),
        );
        let code = status.as_u16();
        let error = match code {
            401 => ProviderError::new(
                Refusal::ProviderAuth,
                format!("the provider rejected the credentials (HTTP {status}); not retried"),
            ),
            402 => ProviderError::new(
                Refusal::ProviderBalance,
                format!(
                    "the provider account has insufficient balance (HTTP {status}); \
                     top up or replace the key; not retried"
                ),
            ),
            429 => ProviderError::new(
                Refusal::ProviderRateLimit,
                format!("the provider rate-limited the request (HTTP {status})"),
            ),
            500 | 502 | 503 | 504 => ProviderError::new(
                Refusal::ProviderTimeout,
                format!("the provider returned a transient HTTP {status}; retry is allowed"),
            ),
            400 | 422 => ProviderError::new(
                Refusal::ProviderInvalidResponse,
                format!("the provider rejected the request shape (HTTP {status}); not retried"),
            ),
            _ => ProviderError::new(
                Refusal::ProviderInvalidResponse,
                format!("undocumented provider HTTP status {status}"),
            ),
        }
        .with_status(code);
        Err(match retry_after {
            Some(retry_after) => error.with_retry_after(retry_after),
            None => error,
        })
    }

    /// Streaming chat completion. Retries only happen before the first event;
    /// once deltas have been emitted the stream is never replayed.
    ///
    /// # Errors
    /// Same named refusals as [`Self::complete`], plus
    /// [`Refusal::ProviderIncomplete`] when the stream ends without `[DONE]`.
    pub async fn stream(&self, request: &ChatRequest) -> Result<StreamHandle, ProviderError> {
        self.stream_inner(request, None, &CallEstimate::default())
            .await
    }

    /// Streaming completion that enforces the run budget **before every open
    /// attempt** and records each attempt (including retries) in the budget.
    ///
    /// Usage is recorded by the caller after [`StreamEvent::Finished`] via
    /// [`RunBudget::record_call`] (the stream's final chunk is where the
    /// documented usage statistics arrive); absent usage is INCOMPLETE, never 0.
    ///
    /// # Errors
    /// Propagates the budget refusals (with **zero** provider requests when the
    /// first preflight refuses) and the same transport refusals as [`Self::complete`].
    pub async fn stream_with_budget(
        &self,
        budget: &mut RunBudget,
        request: &ChatRequest,
        estimate: &CallEstimate,
    ) -> Result<StreamHandle, ProviderError> {
        self.stream_inner(request, Some(budget), estimate).await
    }

    async fn stream_inner(
        &self,
        request: &ChatRequest,
        mut budget: Option<&mut RunBudget>,
        estimate: &CallEstimate,
    ) -> Result<StreamHandle, ProviderError> {
        request.validate()?;
        let permit = self.config.in_flight.acquire().await?;
        let mut stream_request = request.clone();
        stream_request.stream = Some(true);

        let mut attempt = 0u32;
        loop {
            attempt = attempt.saturating_add(1);
            if let Some(active_budget) = budget.as_deref_mut() {
                active_budget.preflight(estimate)?;
            }
            // Request-rate permit for every open attempt (retries included).
            self.config.rate_limit.acquire().await?;
            if let Some(active_budget) = budget.as_deref_mut() {
                active_budget.record_attempt();
            }
            let opened = self.open_stream(&stream_request).await;
            match opened {
                Ok(response) => {
                    let (sender, receiver) = mpsc::channel(64);
                    let idle_timeout = self.config.stream_idle_timeout;
                    tokio::spawn(async move {
                        let _permit = permit;
                        pump_stream(response, sender, idle_timeout).await;
                    });
                    return Ok(StreamHandle {
                        receiver,
                        attempts: attempt,
                    });
                }
                Err(error) => {
                    let retry_budget_left = attempt < self.config.retry.max_attempts;
                    if !error.retryable() || !retry_budget_left {
                        return Err(self.final_error(error));
                    }
                    let delay = self.config.retry.delay_for(attempt, error.retry_after());
                    if let Some(active_budget) = budget.as_deref_mut() {
                        active_budget.on_retry();
                    }
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    async fn open_stream(&self, request: &ChatRequest) -> Result<reqwest::Response, ProviderError> {
        let bearer = self.bearer_header()?;
        let response = self
            .stream_http
            .post(self.endpoint())
            .header(AUTHORIZATION, bearer)
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "text/event-stream")
            .json(request)
            .send()
            .await
            .map_err(|error| {
                map_transport_error(&error, self.config.request_timeout, "opening the stream")
            })?;
        self.classify_status(response)
    }
}

/// Reads an SSE body and forwards parsed events until `[DONE]` or failure.
async fn pump_stream(
    mut response: reqwest::Response,
    sender: mpsc::Sender<Result<StreamEvent, ProviderError>>,
    idle_timeout: Duration,
) {
    let mut pending: Vec<u8> = Vec::new();
    let mut usage: Option<Usage> = None;
    let mut finish_reason: Option<String> = None;
    let mut done = false;

    loop {
        match tokio::time::timeout(idle_timeout, response.chunk()).await {
            Err(_) => {
                let _ = sender
                    .send(Err(ProviderError::new(
                        Refusal::ProviderTimeout,
                        format!(
                            "stream produced no data for {} ms",
                            idle_timeout.as_millis()
                        ),
                    )))
                    .await;
                return;
            }
            Ok(Err(error)) => {
                let _ = sender
                    .send(Err(map_transport_error(
                        &error,
                        idle_timeout,
                        "reading the stream",
                    )))
                    .await;
                return;
            }
            Ok(Ok(None)) => break,
            Ok(Ok(Some(bytes))) => {
                pending.extend_from_slice(&bytes);
                while let Some(position) = pending.iter().position(|byte| *byte == b'\n') {
                    let line: Vec<u8> = pending.drain(..=position).collect();
                    let text = String::from_utf8_lossy(&line);
                    let parsed = match parse_sse_line(&text) {
                        Ok(parsed) => parsed,
                        Err(error) => {
                            let _ = sender.send(Err(error)).await;
                            return;
                        }
                    };
                    match parsed {
                        SseLine::Ignored => {}
                        SseLine::Chunk(chunk) => {
                            if let Some(chunk_usage) = chunk.usage {
                                let mapped = chunk_usage.to_usage();
                                if !mapped.is_empty() {
                                    usage = Some(merge_stream_usage(usage, mapped));
                                }
                            }
                            for choice in &chunk.choices {
                                if let Some(reason) = &choice.finish_reason {
                                    finish_reason = Some(reason.clone());
                                }
                                for event in delta_event(&choice.delta) {
                                    if sender.send(Ok(event)).await.is_err() {
                                        return;
                                    }
                                }
                            }
                        }
                        SseLine::Done => {
                            done = true;
                            break;
                        }
                    }
                }
                if done {
                    break;
                }
            }
        }
    }

    if !done {
        let _ = sender
            .send(Err(ProviderError::new(
                Refusal::ProviderIncomplete,
                "the provider stream ended without the documented [DONE] terminator; the reply is \
                 not reported as success",
            )))
            .await;
        return;
    }
    let _ = sender
        .send(Ok(StreamEvent::Finished {
            usage,
            finish_reason,
        }))
        .await;
}

/// Merges one streamed usage object into the running total.
///
/// OpenAI-compatible streams that set `stream_options.include_usage` report a
/// *cumulative* usage snapshot, and a provider may repeat that snapshot in more
/// than one chunk. Summing blindly double counts it. A later snapshot whose
/// measured counters are all greater than or equal to the previous ones is
/// therefore treated as cumulative (replace); a lower counter means the chunks
/// are incremental fragments, which are still added.
fn merge_stream_usage(existing: Option<Usage>, next: Usage) -> Usage {
    let Some(previous) = existing else {
        return next;
    };
    if is_cumulative_snapshot(&previous, &next) {
        next
    } else {
        previous.checked_add(&next)
    }
}

/// Whether `next` looks like a cumulative snapshot of `previous`.
fn is_cumulative_snapshot(previous: &Usage, next: &Usage) -> bool {
    let same = previous == next;
    let mut grew = false;
    for (before, after) in [
        (previous.prompt_tokens, next.prompt_tokens),
        (previous.completion_tokens, next.completion_tokens),
        (previous.total_tokens, next.total_tokens),
        (
            previous.prompt_cache_hit_tokens,
            next.prompt_cache_hit_tokens,
        ),
        (
            previous.prompt_cache_miss_tokens,
            next.prompt_cache_miss_tokens,
        ),
        (previous.reasoning_tokens, next.reasoning_tokens),
    ] {
        match (before, after) {
            (Some(before), Some(after)) if after >= before => grew |= after > before,
            (Some(_), Some(_)) => return false,
            (None, Some(_)) => grew = true,
            _ => {}
        }
    }
    same || grew
}

/// Parses every documented delta field into ordered stream events.
///
/// A delta can carry a text fragment, reasoning content and one or more
/// tool-call fragments. Every `tool_calls` entry is emitted as its own
/// [`StreamEvent::ToolCallDelta`] in ascending `index` order (the sort is
/// stable, so equal indices keep the provider's array order); a single-entry
/// delta therefore keeps the original one-event behavior.
fn delta_event(delta: &Delta) -> Vec<StreamEvent> {
    let mut events = Vec::new();

    if let Some(content) = &delta.content {
        if !content.is_empty() {
            events.push(StreamEvent::Delta {
                content: content.clone(),
                reasoning_content: delta.reasoning_content.clone(),
            });
        }
    }

    if let Some(tool_calls) = &delta.tool_calls {
        let mut ordered = tool_calls.iter().collect::<Vec<_>>();
        ordered.sort_by_key(|call| call.index);
        for call in ordered {
            let (name, arguments) = match &call.function {
                Some(function) => (function.name.clone(), function.arguments.clone()),
                None => (None, None),
            };
            events.push(StreamEvent::ToolCallDelta {
                index: call.index,
                id: call.id.clone(),
                name,
                arguments,
            });
        }
    }

    if events.is_empty() {
        if let Some(reasoning) = &delta.reasoning_content {
            if !reasoning.is_empty() {
                events.push(StreamEvent::Delta {
                    content: String::new(),
                    reasoning_content: Some(reasoning.clone()),
                });
            }
        }
    }

    events
}

/// Maps transport failures to the frozen refusal table.
///
/// The frozen table has one transport-level refusal
/// ([`Refusal::ProviderTimeout`]); DNS/TLS/connect/read failures are reported
/// through it with non-secret context (adding a new code would be a contract
/// change, which this phase does not make).
pub(crate) fn map_transport_error(
    error: &reqwest::Error,
    timeout: Duration,
    action: &str,
) -> ProviderError {
    let detail = if error.is_timeout() {
        format!(
            "{action} timed out after {} ms",
            u64::try_from(timeout.as_millis()).unwrap_or(u64::MAX)
        )
    } else if error.is_connect() {
        format!("{action} failed to connect")
    } else if error.is_body() || error.is_decode() {
        format!("{action} failed while reading the response")
    } else {
        format!("{action} failed (transport error)")
    };
    ProviderError::new(Refusal::ProviderTimeout, detail)
}

impl DeepSeekClient {
    /// Convenience: a budgeted call with default limits (one run).
    ///
    /// # Errors
    /// Same as [`Self::complete_with_budget`].
    pub async fn complete_with_default_budget(
        &self,
        request: &ChatRequest,
        estimate: &CallEstimate,
        at_unix_secs: u64,
    ) -> Result<(Completion, RunBudget), ProviderError> {
        let mut budget = RunBudget::new(BudgetLimits::default());
        let completion = self
            .complete_with_budget(&mut budget, request, estimate, at_unix_secs)
            .await?;
        Ok((completion, budget))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{ChatMessage, Model};

    #[test]
    fn sse_lines_are_parsed_as_documented() {
        assert_eq!(
            parse_sse_line(": keep-alive").expect("ignored"),
            SseLine::Ignored
        );
        assert_eq!(parse_sse_line("").expect("ignored"), SseLine::Ignored);
        assert_eq!(parse_sse_line("data: [DONE]").expect("done"), SseLine::Done);
        let chunk = parse_sse_line(
            r#"data: {"id":"1","choices":[{"index":0,"delta":{"content":"Hi"},"finish_reason":null}]}"#,
        )
        .expect("chunk");
        match chunk {
            SseLine::Chunk(chunk) => {
                assert_eq!(chunk.choices[0].delta.content.as_deref(), Some("Hi"))
            }
            other => panic!("expected chunk, got {other:?}"),
        }
        let malformed = parse_sse_line("data: {not json}").expect_err("malformed");
        assert_eq!(malformed.code(), "E_PROVIDER_INVALID_RESPONSE");
    }

    #[test]
    fn client_debug_never_contains_the_key() {
        let key = ApiKey::from_secret("sk-debug-test-value").expect("key");
        let client = DeepSeekClient::new(ClientConfig::default(), key).expect("client");
        let debug = format!("{client:?}");
        assert!(
            !debug.contains("sk-debug-test-value"),
            "key must not appear in Debug"
        );
        assert!(debug.contains("https://api.deepseek.com"));
        assert_eq!(
            client.endpoint(),
            "https://api.deepseek.com/chat/completions",
            "documented endpoint, no /v1"
        );
    }

    #[tokio::test]
    async fn request_validation_precedes_any_send() {
        let key = ApiKey::from_secret("sk-validation-test").expect("key");
        let client = DeepSeekClient::new(ClientConfig::default(), key).expect("client");
        let empty = ChatRequest::new(Model::Flash, Vec::<ChatMessage>::new());
        let error = client.complete(&empty).await.expect_err("refused");
        assert_eq!(error.code(), "E_SCHEMA_INVALID");
    }
}
