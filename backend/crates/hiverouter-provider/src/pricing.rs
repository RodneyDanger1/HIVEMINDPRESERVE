//! Price table and peak-window arithmetic (FACT-REGISTER F1).
//!
//! Every price row carries `source_url` + `fetched_at` provenance; a row
//! without provenance is refused with [`Refusal::PriceUnsourced`] instead of
//! being used. Money is [`rust_decimal::Decimal`] only — `f64` is forbidden on
//! every cost path (crate-level lint in `lib.rs`).
//!
//! Documented unit: USD per 1,000,000 tokens. Documented peak windows: UTC
//! 01:00–04:00 and 06:00–10:00, Monday–Friday, `[start, end)`; off-peak prices
//! are the documented half-rate rows.

use rust_decimal::Decimal;

use crate::dto::Model;
use crate::error::ProviderError;
use hiverouter_contract::Refusal;

/// Documented pricing page used for every row (F1).
pub const PRICE_SOURCE_URL: &str = "https://api-docs.deepseek.com/quick_start/pricing/";

/// Date the rows were fetched for the blueprint (F1).
pub const PRICE_FETCHED_AT: &str = "2026-11-06";

/// Documented peak windows in UTC hours, `[start, end)`.
pub const PEAK_WINDOWS_UTC: [(u8, u8); 2] = [(1, 4), (6, 10)];

/// Peak / off-peak rate class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Peak {
    /// Documented peak window.
    Peak,
    /// Documented off-peak window.
    OffPeak,
}

impl Peak {
    /// Stable label for logs and reports.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Peak::Peak => "peak",
            Peak::OffPeak => "off_peak",
        }
    }
}

/// One documented price row (USD per 1M tokens).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PriceRow {
    /// Model the row prices.
    pub model: Model,
    /// Rate class.
    pub peak: Peak,
    /// Cache-hit prompt price per 1M tokens.
    pub cache_hit_usd_per_mtok: Decimal,
    /// Cache-miss prompt price per 1M tokens.
    pub cache_miss_usd_per_mtok: Decimal,
    /// Completion (output) price per 1M tokens.
    pub output_usd_per_mtok: Decimal,
    /// Documented source URL for this row.
    pub source_url: &'static str,
    /// Fetch date for this row.
    pub fetched_at: &'static str,
}

impl PriceRow {
    /// Whether the row carries usable provenance.
    #[must_use]
    pub fn has_provenance(&self) -> bool {
        !self.source_url.trim().is_empty() && !self.fetched_at.trim().is_empty()
    }
}

/// The documented pricing table (both models, both rate classes).
#[derive(Debug, Clone)]
pub struct PricingTable {
    rows: Vec<PriceRow>,
}

impl Default for PricingTable {
    fn default() -> Self {
        Self::documented()
    }
}

impl PricingTable {
    /// Builds the table from the FACT-REGISTER F1 values.
    ///
    /// Flash (per 1M): cache-hit 0.003/0.006, cache-miss 0.15/0.30, output
    /// 0.6/1.2 (off-peak/peak). Pro (per 1M): cache-hit 0.022/0.044,
    /// cache-miss 0.66/1.32, output 1.98/3.96 (off-peak/peak).
    #[must_use]
    pub fn documented() -> Self {
        let row = |model: Model,
                   peak: Peak,
                   hit: (i64, u32),
                   miss: (i64, u32),
                   out: (i64, u32)| PriceRow {
            model,
            peak,
            cache_hit_usd_per_mtok: Decimal::new(hit.0, hit.1),
            cache_miss_usd_per_mtok: Decimal::new(miss.0, miss.1),
            output_usd_per_mtok: Decimal::new(out.0, out.1),
            source_url: PRICE_SOURCE_URL,
            fetched_at: PRICE_FETCHED_AT,
        };
        Self {
            rows: vec![
                row(Model::Flash, Peak::OffPeak, (3, 3), (15, 2), (6, 1)),
                row(Model::Flash, Peak::Peak, (6, 3), (30, 2), (12, 1)),
                row(Model::Pro, Peak::OffPeak, (22, 3), (66, 2), (198, 2)),
                row(Model::Pro, Peak::Peak, (44, 3), (132, 2), (396, 2)),
            ],
        }
    }

    /// Every row, in documented order.
    #[must_use]
    pub fn rows(&self) -> &[PriceRow] {
        &self.rows
    }

    /// Looks up one row and refuses an unsourced row by name.
    ///
    /// # Errors
    /// [`Refusal::PriceUnsourced`] when the row has no source_url/fetched_at or
    /// the (model, rate) pair is missing (never substituted with a default row).
    pub fn row(&self, model: Model, peak: Peak) -> Result<PriceRow, ProviderError> {
        let row = self
            .rows
            .iter()
            .find(|row| row.model == model && row.peak == peak)
            .copied()
            .ok_or_else(|| {
                ProviderError::new(
                    Refusal::PriceUnsourced,
                    format!("no documented price row for model={} rate={}", model.id(), peak.label()),
                )
            })?;
        if !row.has_provenance() {
            return Err(ProviderError::new(
                Refusal::PriceUnsourced,
                format!("price row for model={} rate={} has no source_url/fetched_at", model.id(), peak.label()),
            ));
        }
        Ok(row)
    }
}

/// Days since the Unix epoch for a UTC instant.
fn days_since_epoch(unix_secs: u64) -> u64 {
    unix_secs / 86_400
}

/// Monday-based weekday index (0 = Monday … 6 = Sunday) for a UTC day.
///
/// 1970-01-01 was a Thursday, so `(days + 3) % 7` maps 0 to Monday.
#[must_use]
pub fn weekday_monday_zero(unix_secs: u64) -> u8 {
    u8::try_from((days_since_epoch(unix_secs) + 3) % 7).unwrap_or(0)
}

/// Human label for the UTC weekday (diagnostics only).
#[must_use]
pub fn weekday_label(unix_secs: u64) -> &'static str {
    match weekday_monday_zero(unix_secs) {
        0 => "Monday",
        1 => "Tuesday",
        2 => "Wednesday",
        3 => "Thursday",
        4 => "Friday",
        5 => "Saturday",
        _ => "Sunday",
    }
}

/// Whether the instant falls in a documented UTC peak window (Mon–Fri).
#[must_use]
pub fn is_peak_at_unix(unix_secs: u64) -> bool {
    if weekday_monday_zero(unix_secs) >= 5 {
        return false;
    }
    let hour = u8::try_from((unix_secs % 86_400) / 3_600).unwrap_or(0);
    PEAK_WINDOWS_UTC
        .iter()
        .any(|(start, end)| hour >= *start && hour < *end)
}

/// Rate class for a UTC instant.
#[must_use]
pub fn peak_at_unix(unix_secs: u64) -> Peak {
    if is_peak_at_unix(unix_secs) {
        Peak::Peak
    } else {
        Peak::OffPeak
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hiverouter_contract::Timestamp;
    use std::str::FromStr;

    fn unix(rfc3339: &str) -> u64 {
        Timestamp::parse(rfc3339)
            .expect("valid timestamp")
            .unix_secs()
            .expect("representable")
    }

    #[test]
    fn documented_table_matches_f1_values() {
        let table = PricingTable::documented();
        assert_eq!(table.rows().len(), 4);

        let flash_off = table.row(Model::Flash, Peak::OffPeak).expect("row");
        assert_eq!(flash_off.cache_hit_usd_per_mtok, Decimal::from_str("0.003").unwrap());
        assert_eq!(flash_off.cache_miss_usd_per_mtok, Decimal::from_str("0.15").unwrap());
        assert_eq!(flash_off.output_usd_per_mtok, Decimal::from_str("0.6").unwrap());

        let flash_peak = table.row(Model::Flash, Peak::Peak).expect("row");
        assert_eq!(flash_peak.cache_hit_usd_per_mtok, Decimal::from_str("0.006").unwrap());
        assert_eq!(flash_peak.cache_miss_usd_per_mtok, Decimal::from_str("0.30").unwrap());
        assert_eq!(flash_peak.output_usd_per_mtok, Decimal::from_str("1.2").unwrap());

        let pro_off = table.row(Model::Pro, Peak::OffPeak).expect("row");
        assert_eq!(pro_off.cache_hit_usd_per_mtok, Decimal::from_str("0.022").unwrap());
        assert_eq!(pro_off.cache_miss_usd_per_mtok, Decimal::from_str("0.66").unwrap());
        assert_eq!(pro_off.output_usd_per_mtok, Decimal::from_str("1.98").unwrap());

        let pro_peak = table.row(Model::Pro, Peak::Peak).expect("row");
        assert_eq!(pro_peak.cache_hit_usd_per_mtok, Decimal::from_str("0.044").unwrap());
        assert_eq!(pro_peak.cache_miss_usd_per_mtok, Decimal::from_str("1.32").unwrap());
        assert_eq!(pro_peak.output_usd_per_mtok, Decimal::from_str("3.96").unwrap());

        for row in table.rows() {
            assert!(row.has_provenance(), "every documented row carries provenance");
            assert_eq!(row.source_url, PRICE_SOURCE_URL);
            assert_eq!(row.fetched_at, PRICE_FETCHED_AT);
        }
    }

    #[test]
    fn peak_windows_are_utc_weekday_brackets() {
        // 2026-11-06 is a Friday.
        assert_eq!(weekday_label(unix("2026-11-06T00:00:00Z")), "Friday");
        assert!(!is_peak_at_unix(unix("2026-11-06T00:59:59Z")), "before 01:00 is off-peak");
        assert!(is_peak_at_unix(unix("2026-11-06T01:00:00Z")), "01:00 inclusive");
        assert!(is_peak_at_unix(unix("2026-11-06T03:59:59Z")), "03:59:59 peak");
        assert!(!is_peak_at_unix(unix("2026-11-06T04:00:00Z")), "04:00 exclusive");
        assert!(!is_peak_at_unix(unix("2026-11-06T05:30:00Z")), "between windows off-peak");
        assert!(is_peak_at_unix(unix("2026-11-06T06:00:00Z")), "06:00 peak");
        assert!(is_peak_at_unix(unix("2026-11-06T09:59:59Z")), "09:59:59 peak");
        assert!(!is_peak_at_unix(unix("2026-11-06T10:00:00Z")), "10:00 exclusive");
        // Weekend is off-peak even inside the hours.
        assert_eq!(weekday_label(unix("2026-11-07T02:00:00Z")), "Saturday");
        assert!(!is_peak_at_unix(unix("2026-11-07T02:00:00Z")));
        assert_eq!(weekday_label(unix("2026-11-08T07:00:00Z")), "Sunday");
        assert!(!is_peak_at_unix(unix("2026-11-08T07:00:00Z")));
        // Monday inside the window is peak.
        assert_eq!(weekday_label(unix("2026-11-09T01:30:00Z")), "Monday");
        assert!(is_peak_at_unix(unix("2026-11-09T01:30:00Z")));
    }

    #[test]
    fn unsourced_row_is_refused_not_substituted() {
        let mut table = PricingTable::documented();
        table.rows[0].source_url = "";
        let error = table.row(Model::Flash, Peak::OffPeak).expect_err("unsourced");
        assert_eq!(error.code(), "E_PRICE_UNSOURCED");
    }
}
