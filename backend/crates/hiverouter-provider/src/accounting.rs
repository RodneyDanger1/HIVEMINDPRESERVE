//! Decimal-only cost and token accounting (budget invariants V1–V6).
//!
//! Contract (blueprint §2a, CANONICAL C17, EXT-R5-02):
//! - V1: a provider usage counter that was not reported stays NULL — never 0.
//! - V2: totals over NULL are `INCOMPLETE`, never a number.
//! - V3: money is [`rust_decimal::Decimal`] on every path; `f64` is forbidden
//!   (crate-level `clippy::float_arithmetic` deny).
//! - V4: the provider usage counters are the only billing truth; no local
//!   tokenizer estimate is ever written to a settled cost.
//! - V5: every stored number carries provenance (measured / derived / assumed /
//!   unknown).
//! - V6: NULL usage ⇒ run `INCOMPLETE`, and a settled cost is never exported.
//!
//! Cost formula (documented unit: USD per 1,000,000 tokens):
//! `hit/1e6·hit_price + miss/1e6·miss_price + completion/1e6·output_price`,
//! with the rate class selected from the documented UTC peak windows.

use hiverouter_contract::{Refusal, Usage};
use rust_decimal::Decimal;

use crate::dto::Model;
use crate::error::ProviderError;
use crate::pricing::{peak_at_unix, Peak, PricingTable};

/// Tokens per price unit (documented: prices are per 1,000,000 tokens).
const TOKENS_PER_PRICE_UNIT: u64 = 1_000_000;

/// Provenance label for a stored number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    /// Reported by the provider (usage counters).
    Measured,
    /// Computed from measured inputs (e.g. cost = tokens × documented price).
    Derived,
    /// Pre-send estimate, never settled.
    Assumed,
    /// Not known.
    Unknown,
}

impl Provenance {
    /// Stable label for reports and logs.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Provenance::Measured => "measured",
            Provenance::Derived => "derived",
            Provenance::Assumed => "assumed",
            Provenance::Unknown => "unknown",
        }
    }
}

/// Run-level accounting status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CostStatus {
    /// Every recorded call had complete usage; a settled cost exists.
    Complete,
    /// At least one call was missing usage; the run is INCOMPLETE and no cost
    /// may be exported.
    Incomplete,
}

impl CostStatus {
    /// Stable wire label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            CostStatus::Complete => "COMPLETE",
            CostStatus::Incomplete => "INCOMPLETE",
        }
    }
}

/// The cost of one recorded call.
#[derive(Debug, Clone, PartialEq)]
pub struct CallCost {
    /// Model priced.
    pub model: Model,
    /// Documented rate class applied.
    pub peak: Peak,
    /// Provider usage counters used (measured inputs).
    pub usage: Usage,
    /// Cost in USD (derived from measured usage and a sourced price row).
    pub amount_usd: Decimal,
    /// Provenance of the usage counters (always measured for a settled call).
    pub usage_provenance: Provenance,
    /// Provenance of the derived amount.
    pub amount_provenance: Provenance,
}

/// Exported cost report. `settled_usd` is `None` when the run is INCOMPLETE —
/// the null is the honest answer, never `0.00`.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CostReport {
    /// COMPLETE / INCOMPLETE.
    pub status: &'static str,
    /// Settled cost as a decimal string, or null when INCOMPLETE.
    pub settled_usd: Option<String>,
    /// Number of recorded calls.
    pub calls: u32,
    /// Prompt tokens sum, null when any call omitted them.
    pub prompt_tokens: Option<u64>,
    /// Completion tokens sum, null when any call omitted them.
    pub completion_tokens: Option<u64>,
    /// Total tokens sum, null when any call omitted them.
    pub total_tokens: Option<u64>,
    /// Reasoning tokens sum, null when any call omitted them.
    pub reasoning_tokens: Option<u64>,
    /// Why the run is INCOMPLETE (null when COMPLETE).
    pub incomplete_reason: Option<&'static str>,
    /// Provenance of the counters.
    pub usage_provenance: &'static str,
    /// Provenance of the settled amount.
    pub amount_provenance: &'static str,
}

/// Accumulates usage and cost for one run.
#[derive(Debug, Clone)]
pub struct RunAccounting {
    table: PricingTable,
    usage: Usage,
    calls: u32,
    settled_usd: Decimal,
    has_settled: bool,
    incomplete_reason: Option<Refusal>,
}

impl RunAccounting {
    /// New empty accounting for one run.
    #[must_use]
    pub fn new(table: PricingTable) -> Self {
        Self {
            table,
            usage: Usage::default(),
            calls: 0,
            settled_usd: Decimal::ZERO,
            has_settled: false,
            incomplete_reason: None,
        }
    }

    /// The pricing table owned by this run.
    #[must_use]
    pub fn pricing(&self) -> &PricingTable {
        &self.table
    }

    /// Recorded calls.
    #[must_use]
    pub fn calls(&self) -> u32 {
        self.calls
    }

    /// Accumulated usage (absent stays absent).
    #[must_use]
    pub fn usage(&self) -> &Usage {
        &self.usage
    }

    /// Current status.
    #[must_use]
    pub fn status(&self) -> CostStatus {
        if self.incomplete_reason.is_some() || !self.has_settled && self.calls > 0 && self.settled_usd.is_zero()
        {
            // The second arm only catches a contradictory state (calls without
            // settlement); a fresh run with no calls is still COMPLETE at zero
            // spend because no cost is claimed.
            if self.incomplete_reason.is_none() {
                return CostStatus::Complete;
            }
            CostStatus::Incomplete
        } else {
            CostStatus::Complete
        }
    }

    /// Why the run is incomplete, if it is.
    #[must_use]
    pub fn incomplete_reason(&self) -> Option<Refusal> {
        self.incomplete_reason
    }

    /// Settled cost in USD.
    ///
    /// # Errors
    /// [`Refusal::UsageIncomplete`] when the run is INCOMPLETE — a null usage
    /// can never be exported as a settled figure.
    pub fn settled_usd(&self) -> Result<Decimal, Refusal> {
        match self.incomplete_reason {
            Some(refusal) => Err(refusal),
            None => Ok(self.settled_usd),
        }
    }

    /// Records one provider call's usage and returns its Decimal cost.
    ///
    /// `usage` is `None` when the provider omitted the usage object; that marks
    /// the run INCOMPLETE and never invents zeros.
    ///
    /// # Errors
    /// [`Refusal::UsageIncomplete`] when usage is absent/empty/partial and could
    /// not settle a cost. The run's INCOMPLETE state is recorded before the
    /// error is returned.
    pub fn record(
        &mut self,
        model: Model,
        usage: Option<&Usage>,
        at_unix_secs: u64,
    ) -> Result<CallCost, ProviderError> {
        self.calls = self.calls.saturating_add(1);
        let usage = usage.copied().unwrap_or_default();
        if usage.is_empty() {
            return Err(self.mark_incomplete(
                "provider omitted the usage counters; run is INCOMPLETE and cost is not settled",
            ));
        }

        let completion_tokens = usage.completion_tokens.ok_or_else(|| {
            ProviderError::new(
                Refusal::UsageIncomplete,
                "provider usage has no completion_tokens; output cost cannot be settled",
            )
        });
        let completion_tokens = match completion_tokens {
            Ok(tokens) => tokens,
            Err(error) => {
                self.incomplete_reason = Some(Refusal::UsageIncomplete);
                return Err(error);
            }
        };

        let (cache_hit, cache_miss) =
            match (usage.prompt_cache_hit_tokens, usage.prompt_cache_miss_tokens) {
                (Some(hit), Some(miss)) => (hit, miss),
                (None, None) => {
                    return Err(self.mark_incomplete(
                        "provider usage has no prompt cache hit/miss split; prompt cost cannot be \
                         settled (prompt_tokens alone is never coerced into a price)",
                    ));
                }
                _ => {
                    return Err(self.mark_incomplete(
                        "provider usage has only one side of the prompt cache split; refusing to \
                         guess the missing side",
                    ));
                }
            };

        if let Some(prompt_tokens) = usage.prompt_tokens {
            let split = cache_hit.saturating_add(cache_miss);
            if prompt_tokens != split {
                return Err(self.mark_incomplete(
                    "provider usage is inconsistent: prompt_tokens != prompt_cache_hit_tokens + \
                     prompt_cache_miss_tokens",
                ));
            }
        }

        let peak = peak_at_unix(at_unix_secs);
        let row = self.table.row(model, peak)?;

        let hit_cost = price(cache_hit, row.cache_hit_usd_per_mtok);
        let miss_cost = price(cache_miss, row.cache_miss_usd_per_mtok);
        let output_cost = price(completion_tokens, row.output_usd_per_mtok);
        let amount_usd = hit_cost
            .checked_add(miss_cost)
            .and_then(|partial| partial.checked_add(output_cost))
            .ok_or_else(|| {
                ProviderError::new(Refusal::CostMathInvalid, "decimal cost addition overflowed")
            })?;

        self.settled_usd = match self.settled_usd.checked_add(amount_usd) {
            Some(total) => total,
            None => {
                return Err(ProviderError::new(
                    Refusal::CostMathInvalid,
                    "decimal run-total addition overflowed",
                ));
            }
        };
        self.has_settled = true;
        self.usage = self.usage.checked_add(&usage);

        Ok(CallCost {
            model,
            peak,
            usage,
            amount_usd,
            usage_provenance: Provenance::Measured,
            amount_provenance: Provenance::Derived,
        })
    }

    fn mark_incomplete(&mut self, detail: &str) -> ProviderError {
        if self.incomplete_reason.is_none() {
            self.incomplete_reason = Some(Refusal::UsageIncomplete);
        }
        ProviderError::new(Refusal::UsageIncomplete, detail)
    }

    /// Exported report; a null settled cost stays null (V2/V6).
    #[must_use]
    pub fn report(&self) -> CostReport {
        let status = if self.incomplete_reason.is_some() {
            CostStatus::Incomplete
        } else {
            CostStatus::Complete
        };
        CostReport {
            status: status.label(),
            settled_usd: if status == CostStatus::Complete {
                Some(self.settled_usd.normalize().to_string())
            } else {
                None
            },
            calls: self.calls,
            prompt_tokens: self.usage.prompt_tokens,
            completion_tokens: self.usage.completion_tokens,
            total_tokens: self.usage.total_tokens,
            reasoning_tokens: self.usage.reasoning_tokens,
            incomplete_reason: self.incomplete_reason.map(Refusal::code),
            usage_provenance: Provenance::Measured.label(),
            amount_provenance: Provenance::Derived.label(),
        }
    }
}

/// `tokens × price_per_million / 1_000_000`, exact in Decimal.
fn price(tokens: u64, price_per_mtok: Decimal) -> Decimal {
    Decimal::from(tokens)
        .checked_mul(price_per_mtok)
        .map(|product| product / Decimal::from(TOKENS_PER_PRICE_UNIT))
        .unwrap_or(Decimal::ZERO)
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

    #[test]
    fn cost_math_reproduces_every_f1_row() {
        // 1M hit + 1M miss + 1M output at 1M tokens each is exactly the row's
        // three per-million prices summed.
        let cases = [
            (Model::Flash, "2026-11-06T00:00:00Z", "0.753", "2026-11-06T01:00:00Z", "1.506"),
            (Model::Pro, "2026-11-06T00:00:00Z", "2.662", "2026-11-06T01:00:00Z", "5.324"),
        ];
        for (model, off_peak_at, off_expected, peak_at, peak_expected) in cases {
            let mut accounting = RunAccounting::new(PricingTable::documented());
            let off = accounting
                .record(model, Some(&usage(1_000_000, 1_000_000, 1_000_000)), unix(off_peak_at))
                .expect("off-peak settles");
            assert_eq!(off.peak, Peak::OffPeak);
            assert_eq!(off.amount_usd, Decimal::from_str(off_expected).expect("decimal"), "{model:?} off-peak");

            let mut accounting = RunAccounting::new(PricingTable::documented());
            let peak = accounting
                .record(model, Some(&usage(1_000_000, 1_000_000, 1_000_000)), unix(peak_at))
                .expect("peak settles");
            assert_eq!(peak.peak, Peak::Peak);
            assert_eq!(peak.amount_usd, Decimal::from_str(peak_expected).expect("decimal"), "{model:?} peak");
        }
    }

    #[test]
    fn null_or_partial_usage_marks_incomplete_and_never_settles_zero() {
        let mut accounting = RunAccounting::new(PricingTable::documented());

        let error = accounting.record(Model::Flash, None, unix("2026-11-06T00:00:00Z"));
        let error = error.expect_err("null usage refused");
        assert_eq!(error.code(), "E_USAGE_INCOMPLETE");
        assert_eq!(accounting.report().settled_usd, None);
        assert_eq!(accounting.report().status, "INCOMPLETE");

        let mut accounting = RunAccounting::new(PricingTable::documented());
        let partial = Usage {
            prompt_tokens: Some(100),
            completion_tokens: Some(50),
            // No cache split documented in this response.
            ..Usage::default()
        };
        assert_eq!(
            accounting
                .record(Model::Flash, Some(&partial), 0)
                .expect_err("partial refused")
                .code(),
            "E_USAGE_INCOMPLETE"
        );
        assert_eq!(accounting.report().settled_usd, None);

        // A settled call followed by a null-usage call turns the whole run
        // INCOMPLETE; the earlier figure is not exported either.
        let mut accounting = RunAccounting::new(PricingTable::documented());
        accounting
            .record(Model::Flash, Some(&Usage { completion_tokens: Some(10), prompt_cache_hit_tokens: Some(1), prompt_cache_miss_tokens: Some(1), ..Usage::default() }), 0)
            .expect("first settles");
        accounting.record(Model::Flash, None, 0).expect_err("second incomplete");
        assert_eq!(accounting.report().settled_usd, None);
        assert!(accounting.settled_usd().is_err());
    }

    #[test]
    fn inconsistent_prompt_split_is_refused() {
        let mut accounting = RunAccounting::new(PricingTable::documented());
        let inconsistent = Usage {
            prompt_tokens: Some(100),
            completion_tokens: Some(10),
            prompt_cache_hit_tokens: Some(20),
            prompt_cache_miss_tokens: Some(70),
            ..Usage::default()
        };
        assert_eq!(
            accounting
                .record(Model::Flash, Some(&inconsistent), 0)
                .expect_err("refused")
                .code(),
            "E_USAGE_INCOMPLETE"
        );
    }

    #[test]
    fn report_serialises_null_as_null() {
        let mut accounting = RunAccounting::new(PricingTable::documented());
        accounting
            .record(
                Model::Flash,
                Some(&Usage {
                    prompt_tokens: Some(100),
                    completion_tokens: Some(50),
                    total_tokens: Some(150),
                    prompt_cache_hit_tokens: Some(40),
                    prompt_cache_miss_tokens: Some(60),
                    reasoning_tokens: None,
                }),
                0,
            )
            .expect("settles");
        let value = serde_json::to_value(accounting.report()).expect("serialises");
        assert_eq!(value["status"], "COMPLETE");
        assert_eq!(value["reasoning_tokens"], serde_json::Value::Null, "absent stays null");
        assert!(value["settled_usd"].is_string());
        // 40 hit × 0.003/1M + 60 miss × 0.15/1M + 50 out × 0.6/1M
        assert_eq!(
            accounting.settled_usd().expect("settled"),
            Decimal::from_str("0.00003912").expect("decimal")
        );
    }
}
