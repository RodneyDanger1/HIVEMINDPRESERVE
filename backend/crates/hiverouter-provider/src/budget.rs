//! Pre-send budget core and the B1–B12 invariants (blueprint §2a, C17).
//!
//! Policy defaults are PROPOSED-POLICY values from the blueprint; they are not
//! measured facts and are only changed with a decision + ledger row.
//!
//! Enforcement order in [`RunBudget::preflight`] happens **before** any HTTP
//! request is built: an exceeded cap refuses with zero provider calls.
//! Retries are bounded per call (≤3 attempts, jittered, Retry-After capped at
//! 60 s, no retry on 401/402 — the client honours this policy).

use std::sync::Arc;
use std::time::Duration;

use hiverouter_contract::{Refusal, Usage};
use rust_decimal::Decimal;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use uuid::Uuid;

use crate::accounting::{CallCost, CostReport, CostStatus, RunAccounting};
use crate::dto::Model;
use crate::error::ProviderError;
use crate::pricing::PricingTable;

/// B1: run cost cap, USD per run (PROPOSED-POLICY).
pub const RUN_COST_CAP_USD_CENTS: i64 = 50;

/// B2: combined prompt+completion token budget per run (PROPOSED-POLICY).
pub const TOKEN_BUDGET_COMBINED: u64 = 400_000;

/// B3: provider call cap per run, including retries (PROPOSED-POLICY).
pub const CALL_CAP: u32 = 40;

/// B4: maximum spawn depth (Hyper → Super → Worker).
pub const MAX_SPAWN_DEPTH: u8 = 3;

/// B5: maximum children per task.
pub const FANOUT_LIMIT: u8 = 8;

/// B6: maximum agents per run.
pub const AGENT_LIMIT: u16 = 64;

/// B7: local in-flight semaphore size (provider ceiling: 2500 flash / 500 pro).
pub const IN_FLIGHT_LIMIT: usize = 8;

/// B8: maximum attempts per provider call (first try + retries).
pub const MAX_ATTEMPTS: u32 = 3;

/// B9: Retry-After is capped at 60 s (FACT F1 + §2a).
pub const RETRY_AFTER_CAP: Duration = Duration::from_secs(60);

/// B10: exactly one account key; multi-key rotation is forbidden.
pub const SUPPORTED_KEY_COUNT: usize = 1;

/// The documented run cost cap as a Decimal.
#[must_use]
pub fn run_cost_cap_usd() -> Decimal {
    Decimal::new(RUN_COST_CAP_USD_CENTS, 2)
}

/// Budget limits for one run (defaults are the blueprint PROPOSED-POLICY values).
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetLimits {
    /// B1: run cost cap in USD.
    pub run_cost_cap_usd: Decimal,
    /// B2: combined token budget.
    pub token_budget_combined: u64,
    /// B3: call cap including retries.
    pub call_cap: u32,
    /// B8: attempts per call.
    pub max_attempts: u32,
}

impl Default for BudgetLimits {
    fn default() -> Self {
        Self {
            run_cost_cap_usd: run_cost_cap_usd(),
            token_budget_combined: TOKEN_BUDGET_COMBINED,
            call_cap: CALL_CAP,
            max_attempts: MAX_ATTEMPTS,
        }
    }
}

/// A pre-send estimate. Values are `Assumed` provenance and never settled as
/// cost; they may only gate a send against the documented budgets.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CallEstimate {
    /// Estimated prompt tokens (assumed, not measured).
    pub prompt_tokens: Option<u64>,
    /// Maximum output tokens the call may generate (from the request's cap).
    pub max_output_tokens: Option<u64>,
}

impl CallEstimate {
    /// Total estimate when both sides are known.
    #[must_use]
    pub fn total_tokens(&self) -> Option<u64> {
        match (self.prompt_tokens, self.max_output_tokens) {
            (Some(prompt), Some(output)) => Some(prompt.saturating_add(output)),
            _ => None,
        }
    }
}

/// Proof that a send passed the pre-send budget checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BudgetPermit {
    /// Zero-based call index this permit allows.
    pub call_index: u32,
}

/// Per-run budget and cost accounting (single money owner: [`RunAccounting`]).
#[derive(Debug, Clone)]
pub struct RunBudget {
    limits: BudgetLimits,
    accounting: RunAccounting,
    attempts: u32,
    retries: u32,
}

impl RunBudget {
    /// New budget for one run using the documented pricing table.
    #[must_use]
    pub fn new(limits: BudgetLimits) -> Self {
        Self {
            limits,
            accounting: RunAccounting::new(PricingTable::documented()),
            attempts: 0,
            retries: 0,
        }
    }

    /// New budget over an explicit pricing table (tests / future price updates).
    #[must_use]
    pub fn with_pricing(limits: BudgetLimits, table: PricingTable) -> Self {
        Self {
            limits,
            accounting: RunAccounting::new(table),
            attempts: 0,
            retries: 0,
        }
    }

    /// Configured limits.
    #[must_use]
    pub fn limits(&self) -> &BudgetLimits {
        &self.limits
    }

    /// Recorded calls (each settled usage record).
    #[must_use]
    pub fn calls(&self) -> u32 {
        self.accounting.calls()
    }

    /// Provider send attempts including retries (B3 call cap counts these).
    #[must_use]
    pub fn attempts(&self) -> u32 {
        self.attempts
    }

    /// Records one send attempt (called immediately before each provider send,
    /// including retries, so the B3 call cap includes retries).
    pub fn record_attempt(&mut self) {
        self.attempts = self.attempts.saturating_add(1);
    }

    /// Retries recorded for diagnostics (bounded per call by the client policy).
    #[must_use]
    pub fn retries(&self) -> u32 {
        self.retries
    }

    /// Spent cost in USD (Decimal only).
    #[must_use]
    pub fn spent_usd(&self) -> Decimal {
        self.accounting
            .settled_usd()
            .unwrap_or(Decimal::ZERO)
    }

    /// Remaining run cost budget (never negative).
    #[must_use]
    pub fn remaining_usd(&self) -> Decimal {
        match self
            .limits
            .run_cost_cap_usd
            .checked_sub(self.spent_usd())
        {
            Some(remaining) if remaining > Decimal::ZERO => remaining,
            _ => Decimal::ZERO,
        }
    }

    /// Combined prompt+completion tokens accumulated so far, `None` when any
    /// side is unknown (never fabricated as zero).
    #[must_use]
    pub fn combined_tokens(&self) -> Option<u64> {
        let usage = self.accounting.usage();
        let prompt = usage.prompt_tokens.or_else(|| match (
            usage.prompt_cache_hit_tokens,
            usage.prompt_cache_miss_tokens,
        ) {
            (Some(hit), Some(miss)) => Some(hit.saturating_add(miss)),
            _ => None,
        })?;
        let completion = usage.completion_tokens?;
        Some(prompt.saturating_add(completion))
    }

    /// Run cost status.
    #[must_use]
    pub fn status(&self) -> CostStatus {
        self.accounting.status()
    }

    /// Accounting report (null stays null; INCOMPLETE exports no cost).
    #[must_use]
    pub fn report(&self) -> CostReport {
        self.accounting.report()
    }

    /// Pre-send budget check (B1/B2/B3/B7 and the V1–V6 integrity rule).
    ///
    /// Must be called **before** the HTTP request is built; a refusal means the
    /// caller sends nothing.
    ///
    /// # Errors
    /// - [`Refusal::UsageIncomplete`] when a previous call left the run cost
    ///   incomplete (the token/cost totals cannot be verified).
    /// - [`Refusal::CallLimit`] when the call cap is reached.
    /// - [`Refusal::BudgetExceeded`] when the run cost cap is reached.
    /// - [`Refusal::TokenBudgetExceeded`] when known spend + the estimate
    ///   exceeds the combined token budget.
    pub fn preflight(&self, estimate: &CallEstimate) -> Result<BudgetPermit, ProviderError> {
        if self.accounting.incomplete_reason().is_some() {
            return Err(ProviderError::new(
                Refusal::UsageIncomplete,
                "a previous call left the run INCOMPLETE; refusing another send because the \
                 token and cost totals cannot be verified",
            ));
        }
        if self.attempts >= self.limits.call_cap {
            return Err(ProviderError::new(
                Refusal::CallLimit,
                format!(
                    "call cap reached ({} of {} attempts, including retries)",
                    self.attempts, self.limits.call_cap
                ),
            ));
        }
        let spent = self.spent_usd();
        if spent >= self.limits.run_cost_cap_usd {
            return Err(ProviderError::new(
                Refusal::BudgetExceeded,
                format!(
                    "run cost cap reached (spent {spent} of {})",
                    self.limits.run_cost_cap_usd
                ),
            ));
        }
        if let (Some(used), Some(estimated)) = (self.combined_tokens(), estimate.total_tokens()) {
            let projected = used.saturating_add(estimated);
            if projected > self.limits.token_budget_combined {
                return Err(ProviderError::new(
                    Refusal::TokenBudgetExceeded,
                    format!(
                        "combined token budget would be exceeded ({projected} > {})",
                        self.limits.token_budget_combined
                    ),
                ));
            }
        }
        Ok(BudgetPermit {
            call_index: self.calls(),
        })
    }

    /// Records the outcome of an allowed call (usage may be absent).
    ///
    /// # Errors
    /// [`Refusal::UsageIncomplete`] when usage is absent or too partial to
    /// settle a cost; the run is marked INCOMPLETE and no cost is exported.
    pub fn record_call(
        &mut self,
        model: Model,
        usage: Option<&Usage>,
        at_unix_secs: u64,
    ) -> Result<CallCost, ProviderError> {
        self.accounting.record(model, usage, at_unix_secs)
    }

    /// Records one retry attempt (diagnostics; the per-call cap is enforced by
    /// the client's [`RetryPolicy`]).
    pub fn on_retry(&mut self) {
        self.retries = self.retries.saturating_add(1);
    }
}

/// Backoff policy for provider retries (B8/B9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryPolicy {
    /// Attempts including the first try.
    pub max_attempts: u32,
    /// Base exponential backoff.
    pub base_delay: Duration,
    /// Cap for the jittered exponential delay.
    pub max_delay: Duration,
    /// Maximum jitter added to an exponential delay.
    pub jitter_max: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: MAX_ATTEMPTS,
            base_delay: Duration::from_millis(250),
            max_delay: Duration::from_secs(8),
            jitter_max: Duration::from_millis(250),
        }
    }
}

impl RetryPolicy {
    /// Delay before attempt `attempt` (1-based; attempt 1 is the first retry).
    ///
    /// A provider `Retry-After` wins (documented server instruction) and is
    /// capped at [`RETRY_AFTER_CAP`]; otherwise jittered exponential backoff is
    /// used. No retry happens on 401/402 (enforced by the client).
    #[must_use]
    pub fn delay_for(&self, attempt: u32, retry_after: Option<Duration>) -> Duration {
        if let Some(retry_after) = retry_after {
            return retry_after.min(RETRY_AFTER_CAP);
        }
        let exponent = attempt.saturating_sub(1).min(16);
        let exponential = self.base_delay.saturating_mul(1u32 << exponent);
        let jitter = jitter_within(self.jitter_max);
        exponential
            .saturating_add(jitter)
            .min(self.max_delay.max(exponential.min(self.max_delay)))
    }
}

/// Deterministic-bounded jitter drawn from a uuid-v7 (no extra RNG crate).
fn jitter_within(max: Duration) -> Duration {
    let budget_nanos = u64::try_from(max.as_nanos()).unwrap_or(u64::MAX);
    if budget_nanos == 0 {
        return Duration::ZERO;
    }
    let seed = Uuid::now_v7().as_u128() as u64;
    Duration::from_nanos(seed % budget_nanos)
}

/// Parses a `Retry-After` header value (integer seconds) and caps it at 60 s.
///
/// HTTP-date values are not documented by the provider and are ignored (`None`),
/// which falls back to jittered exponential backoff rather than guessing.
#[must_use]
pub fn retry_after_from_header_value(value: Option<&str>) -> Option<Duration> {
    let seconds: u64 = value?.trim().parse().ok()?;
    Some(Duration::from_secs(seconds).min(RETRY_AFTER_CAP))
}

/// Local in-flight concurrency limiter (B7; provider concurrency is
/// account-level per FACT F1 — this local semaphore is the only limiter).
#[derive(Debug, Clone)]
pub struct InFlightLimiter {
    semaphore: Arc<Semaphore>,
    limit: usize,
}

impl InFlightLimiter {
    /// Creates a limiter with the documented local in-flight size (8).
    #[must_use]
    pub fn new(limit: usize) -> Self {
        assert!(limit > 0, "in-flight limit must be positive");
        Self {
            semaphore: Arc::new(Semaphore::new(limit)),
            limit,
        }
    }

    /// Documented local limit.
    #[must_use]
    pub fn limit(&self) -> usize {
        self.limit
    }

    /// Permits currently available.
    #[must_use]
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Waits for a permit (queued work waits; `E_SEMAPHORE_FULL` is returned by
    /// [`Self::try_acquire`] for non-blocking callers).
    ///
    /// # Errors
    /// [`Refusal::InternalError`] only if the semaphore is closed (it never is
    /// in this crate).
    pub async fn acquire(&self) -> Result<OwnedSemaphorePermit, ProviderError> {
        Arc::clone(&self.semaphore)
            .acquire_owned()
            .await
            .map_err(|_| {
                ProviderError::new(Refusal::InternalError, "in-flight semaphore was closed")
            })
    }

    /// Non-blocking acquire.
    ///
    /// # Errors
    /// [`Refusal::SemaphoreFull`] when every permit is in flight.
    pub fn try_acquire(&self) -> Result<OwnedSemaphorePermit, ProviderError> {
        Arc::clone(&self.semaphore)
            .try_acquire_owned()
            .map_err(|error| match error {
                tokio::sync::TryAcquireError::NoPermits => ProviderError::new(
                    Refusal::SemaphoreFull,
                    format!("in-flight semaphore full (limit {})", self.limit),
                ),
                tokio::sync::TryAcquireError::Closed => {
                    ProviderError::new(Refusal::InternalError, "in-flight semaphore was closed")
                }
            })
    }
}

/// Spawn caps for one run (B4/B5/B6), checked before any spawn is committed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunCaps {
    /// Maximum depth.
    pub max_depth: u8,
    /// Maximum children per task.
    pub max_fanout: u8,
    /// Maximum total agents per run.
    pub max_agents: u16,
}

impl Default for RunCaps {
    fn default() -> Self {
        Self {
            max_depth: MAX_SPAWN_DEPTH,
            max_fanout: FANOUT_LIMIT,
            max_agents: AGENT_LIMIT,
        }
    }
}

impl RunCaps {
    /// Checks a proposed spawn against the caps.
    ///
    /// # Errors
    /// [`Refusal::SpawnDepth`], [`Refusal::FanoutLimit`] or
    /// [`Refusal::AgentLimit`] — refused by name before the spawn is recorded.
    pub fn check_spawn(
        &self,
        child_depth: u8,
        current_children: u8,
        current_agents: u16,
    ) -> Result<(), ProviderError> {
        if child_depth > self.max_depth {
            return Err(ProviderError::new(
                Refusal::SpawnDepth,
                format!("spawn depth {child_depth} exceeds the maximum {}", self.max_depth),
            ));
        }
        if current_children >= self.max_fanout {
            return Err(ProviderError::new(
                Refusal::FanoutLimit,
                format!(
                    "fan-out {current_children} already at the limit {}",
                    self.max_fanout
                ),
            ));
        }
        if current_agents.saturating_add(1) > self.max_agents {
            return Err(ProviderError::new(
                Refusal::AgentLimit,
                format!(
                    "spawn would exceed the {} agent cap",
                    self.max_agents
                ),
            ));
        }
        Ok(())
    }
}

/// Enforces the one-key rule (D2/B10). Multi-key rotation is forbidden.
///
/// # Errors
/// [`Refusal::KeyMissing`] for zero keys, [`Refusal::MultikeyForbidden`] for
/// more than one.
pub fn enforce_single_key(key_count: usize) -> Result<(), ProviderError> {
    match key_count {
        0 => Err(ProviderError::new(
            Refusal::KeyMissing,
            "no DeepSeek API key configured (exactly one key is required)",
        )),
        SUPPORTED_KEY_COUNT => Ok(()),
        other => Err(ProviderError::new(
            Refusal::MultikeyForbidden,
            format!("{other} provider keys configured; exactly one account key is supported"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hiverouter_contract::Timestamp;
    use std::str::FromStr;

    fn unix(rfc3339: &str) -> u64 {
        Timestamp::parse(rfc3339)
            .expect("valid")
            .unix_secs()
            .expect("representable")
    }

    fn settled_usage() -> Usage {
        Usage {
            prompt_tokens: Some(1_000),
            completion_tokens: Some(500),
            total_tokens: Some(1_500),
            prompt_cache_hit_tokens: Some(400),
            prompt_cache_miss_tokens: Some(600),
            reasoning_tokens: None,
        }
    }

    #[test]
    fn defaults_match_blueprint_policy_values() {
        let limits = BudgetLimits::default();
        assert_eq!(limits.run_cost_cap_usd, Decimal::from_str("0.50").unwrap());
        assert_eq!(limits.token_budget_combined, 400_000);
        assert_eq!(limits.call_cap, 40);
        assert_eq!(limits.max_attempts, 3);
        assert_eq!(MAX_SPAWN_DEPTH, 3);
        assert_eq!(FANOUT_LIMIT, 8);
        assert_eq!(AGENT_LIMIT, 64);
        assert_eq!(IN_FLIGHT_LIMIT, 8);
        assert_eq!(RETRY_AFTER_CAP, Duration::from_secs(60));
    }

    #[test]
    fn budget_preflight_refuses_before_any_send() {
        // Cost cap reached by a real measured call; the next preflight refuses
        // without any provider interaction.
        let mut budget = RunBudget::new(BudgetLimits::default());
        let expensive = Usage {
            prompt_tokens: Some(1_000_000),
            completion_tokens: Some(1_000_000),
            total_tokens: Some(2_000_000),
            prompt_cache_hit_tokens: Some(500_000),
            prompt_cache_miss_tokens: Some(500_000),
            reasoning_tokens: None,
        };
        budget
            .record_call(Model::Pro, Some(&expensive), unix("2026-11-06T01:00:00Z"))
            .expect("measured call settles");
        assert!(budget.spent_usd() > budget.limits().run_cost_cap_usd);
        let error = budget
            .preflight(&CallEstimate::default())
            .expect_err("cap reached");
        assert_eq!(error.code(), "E_BUDGET_EXCEEDED");
        assert_eq!(budget.calls(), 1, "no call was attempted by preflight");
    }

    #[test]
    fn call_cap_and_token_budget_refuse_by_name() {
        // The cap counts send attempts including retries.
        let mut limits = BudgetLimits::default();
        limits.call_cap = 1;
        let mut budget = RunBudget::new(limits);
        budget.record_attempt();
        assert_eq!(
            budget.preflight(&CallEstimate::default()).expect_err("cap").code(),
            "E_CALL_LIMIT"
        );

        let mut limits = BudgetLimits::default();
        limits.call_cap = 2;
        let mut budget = RunBudget::new(limits);
        budget
            .record_call(Model::Flash, Some(&settled_usage()), 0)
            .expect("settles");
        budget
            .record_call(Model::Flash, Some(&settled_usage()), 0)
            .expect("settles");
        budget.record_attempt();
        budget.record_attempt();
        assert_eq!(
            budget.preflight(&CallEstimate::default()).expect_err("cap").code(),
            "E_CALL_LIMIT"
        );

        let mut limits = BudgetLimits::default();
        limits.token_budget_combined = 1_000;
        let budget = RunBudget::new(limits);
        let estimate = CallEstimate {
            prompt_tokens: Some(900),
            max_output_tokens: Some(200),
        };
        assert_eq!(
            budget.preflight(&estimate).expect_err("tokens").code(),
            "E_TOKEN_BUDGET_EXCEEDED"
        );
    }

    #[test]
    fn incomplete_usage_blocks_further_sends() {
        let mut budget = RunBudget::new(BudgetLimits::default());
        budget
            .record_call(Model::Flash, None, 0)
            .expect_err("null usage is incomplete");
        assert_eq!(
            budget.preflight(&CallEstimate::default()).expect_err("incomplete").code(),
            "E_USAGE_INCOMPLETE"
        );
    }

    #[test]
    fn retry_after_is_capped_and_parsed() {
        assert_eq!(
            retry_after_from_header_value(Some("120")),
            Some(Duration::from_secs(60))
        );
        assert_eq!(
            retry_after_from_header_value(Some("7")),
            Some(Duration::from_secs(7))
        );
        assert_eq!(retry_after_from_header_value(Some("Wed, 21 Oct 2026 07:28:00 GMT")), None);
        assert_eq!(retry_after_from_header_value(None), None);

        let policy = RetryPolicy::default();
        assert_eq!(
            policy.delay_for(1, Some(Duration::from_secs(120))),
            Duration::from_secs(60),
            "Retry-After wins and is capped"
        );
        for attempt in 1..=5 {
            let delay = policy.delay_for(attempt, None);
            assert!(delay >= policy.base_delay, "backoff has a floor");
            assert!(delay <= policy.max_delay.max(policy.base_delay), "backoff is bounded");
        }
    }

    #[test]
    fn one_key_rule_refuses_by_name() {
        assert_eq!(enforce_single_key(1), Ok(()));
        assert_eq!(enforce_single_key(0).expect_err("none").code(), "E_KEY_MISSING");
        assert_eq!(
            enforce_single_key(2).expect_err("multi").code(),
            "E_MULTIKEY_FORBIDDEN"
        );
    }

    #[test]
    fn spawn_caps_refuse_by_name() {
        let caps = RunCaps::default();
        assert!(caps.check_spawn(3, 7, 63).is_ok());
        assert_eq!(
            caps.check_spawn(4, 0, 0).expect_err("depth").code(),
            "E_SPAWN_DEPTH"
        );
        assert_eq!(
            caps.check_spawn(2, 8, 0).expect_err("fanout").code(),
            "E_FANOUT_LIMIT"
        );
        assert_eq!(
            caps.check_spawn(2, 0, 64).expect_err("agents").code(),
            "E_AGENT_LIMIT"
        );
    }

    #[tokio::test]
    async fn in_flight_limiter_bounds_concurrency() {
        let limiter = InFlightLimiter::new(2);
        let first = limiter.try_acquire().expect("permit 1");
        let second = limiter.try_acquire().expect("permit 2");
        assert_eq!(limiter.available_permits(), 0);
        assert_eq!(
            limiter.try_acquire().expect_err("full").code(),
            "E_SEMAPHORE_FULL"
        );
        drop(first);
        let _third = limiter.try_acquire().expect("permit released");
        drop(second);
        let _fourth = limiter.acquire().await.expect("queued acquire works");
    }

    #[test]
    fn report_totals_are_null_when_unknown() {
        let budget = RunBudget::new(BudgetLimits::default());
        let report = budget.report();
        assert_eq!(report.status, "COMPLETE");
        assert_eq!(report.prompt_tokens, None);
        assert_eq!(report.completion_tokens, None);
        assert_eq!(report.settled_usd.as_deref(), Some("0"));
    }
}
