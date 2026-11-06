//! RFC3339 timestamp newtype.
//!
//! The contract requires `ts` to be an RFC3339 string. This module parses any
//! RFC3339 instant (UTC `Z` or numeric offset) and normalises it to UTC with
//! second precision, so the value on the wire is always canonical. No external
//! date crate is pinned for the workspace, so the civil-date conversion is
//! implemented here and pinned by tests.

use std::borrow::Cow;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

/// Error returned when a string is not a valid RFC3339 timestamp.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TimestampError {
    /// The string does not match `YYYY-MM-DDTHH:MM:SS[.fraction](Z|+HH:MM|-HH:MM)`.
    #[error("timestamp is not RFC3339: {0}")]
    Malformed(String),
    /// A calendar field is out of range (month 13, day 32, hour 24, ...).
    #[error("timestamp field out of range: {0}")]
    OutOfRange(String),
}

/// Canonical UTC RFC3339 timestamp (`YYYY-MM-DDTHH:MM:SSZ`).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(String);

impl Timestamp {
    /// Current system time, normalised to UTC seconds.
    #[must_use]
    pub fn now() -> Self {
        let unix_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs());
        Self::from_unix_secs(unix_secs)
    }

    /// Builds a timestamp from seconds since the Unix epoch.
    #[must_use]
    pub fn from_unix_secs(unix_secs: u64) -> Self {
        let days = i64::try_from(unix_secs / 86_400).unwrap_or(i64::MAX);
        let seconds_of_day = unix_secs % 86_400;
        let (year, month, day) = civil_from_days(days);
        let hour = seconds_of_day / 3_600;
        let minute = (seconds_of_day % 3_600) / 60;
        let second = seconds_of_day % 60;
        Self(format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"))
    }

    /// Parses an RFC3339 instant and normalises it to UTC seconds.
    ///
    /// # Errors
    /// Returns [`TimestampError`] when the string is malformed or a field is out
    /// of range.
    pub fn parse(input: &str) -> Result<Self, TimestampError> {
        ParsedInstant::parse(input).map(|instant| Self::from_unix_secs(instant.unix_secs))
    }

    /// Borrows the canonical string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Seconds since the Unix epoch.
    ///
    /// # Errors
    /// Returns [`TimestampError`] if the stored string somehow no longer parses.
    pub fn unix_secs(&self) -> Result<u64, TimestampError> {
        Ok(ParsedInstant::parse(&self.0)?.unix_secs)
    }
}

impl fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Timestamp").field(&self.0).finish()
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for Timestamp {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(|error| de::Error::custom(error.to_string()))
    }
}

impl JsonSchema for Timestamp {
    fn schema_name() -> Cow<'static, str> {
        "Timestamp".into()
    }

    fn schema_id() -> Cow<'static, str> {
        concat!(module_path!(), "::Timestamp").into()
    }

    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        json_schema!({"type": "string", "format": "date-time"})
    }
}

struct ParsedInstant {
    unix_secs: u64,
}

impl ParsedInstant {
    fn parse(input: &str) -> Result<Self, TimestampError> {
        let bytes = input.as_bytes();
        if bytes.len() < 20 {
            return Err(TimestampError::Malformed(input.to_owned()));
        }
        if bytes[4] != b'-' || bytes[7] != b'-' || bytes[10] != b'T' || bytes[13] != b':' || bytes[16] != b':' {
            return Err(TimestampError::Malformed(input.to_owned()));
        }
        let year = parse_digits(input, 0, 4)?;
        let month = parse_digits(input, 5, 2)?;
        let day = parse_digits(input, 8, 2)?;
        let hour = parse_digits(input, 11, 2)?;
        let minute = parse_digits(input, 14, 2)?;
        let second = parse_digits(input, 17, 2)?;
        let mut cursor = 19;
        // Optional fractional seconds (`.<1..=9 digits>`), discarded on normalisation.
        if cursor < bytes.len() && bytes[cursor] == b'.' {
            cursor += 1;
            let fraction_start = cursor;
            while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
                cursor += 1;
            }
            if cursor == fraction_start {
                return Err(TimestampError::Malformed(input.to_owned()));
            }
        }
        let offset_secs = parse_offset(input, cursor)?;
        validate_calendar(year, month, day, hour, minute, second, input)?;
        let days = days_from_civil(i64::from(year), month, day);
        let seconds_of_day = u64::from(hour) * 3_600 + u64::from(minute) * 60 + u64::from(second);
        // Offset is the local-time displacement from UTC: local = utc + offset.
        let local = i128::from(days) * 86_400 + i128::from(seconds_of_day);
        let utc = local - i128::from(offset_secs);
        if utc < 0 {
            return Err(TimestampError::OutOfRange(format!(
                "{input} is before the Unix epoch and cannot be represented as unix seconds"
            )));
        }
        let unix_secs = u64::try_from(utc).map_err(|_| TimestampError::OutOfRange(input.to_owned()))?;
        Ok(Self { unix_secs })
    }
}

fn parse_digits(input: &str, start: usize, len: usize) -> Result<u32, TimestampError> {
    let slice = input
        .get(start..start + len)
        .ok_or_else(|| TimestampError::Malformed(input.to_owned()))?;
    if !slice.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(TimestampError::Malformed(input.to_owned()));
    }
    slice
        .parse::<u32>()
        .map_err(|_| TimestampError::Malformed(input.to_owned()))
}

fn parse_offset(input: &str, cursor: usize) -> Result<i32, TimestampError> {
    let rest = &input[cursor..];
    if rest == "Z" || rest == "z" {
        return Ok(0);
    }
    let bytes = rest.as_bytes();
    if bytes.len() != 6 || (bytes[0] != b'+' && bytes[0] != b'-') || bytes[3] != b':' {
        return Err(TimestampError::Malformed(input.to_owned()));
    }
    let hours = parse_digits(rest, 1, 2)?;
    let minutes = parse_digits(rest, 4, 2)?;
    if hours > 23 || minutes > 59 {
        return Err(TimestampError::OutOfRange(input.to_owned()));
    }
    let magnitude = i32::try_from(hours * 3_600 + minutes * 60)
        .map_err(|_| TimestampError::OutOfRange(input.to_owned()))?;
    Ok(if bytes[0] == b'-' { -magnitude } else { magnitude })
}

fn validate_calendar(
    year: u32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    input: &str,
) -> Result<(), TimestampError> {
    if !(1..=12).contains(&month) {
        return Err(TimestampError::OutOfRange(input.to_owned()));
    }
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return Err(TimestampError::OutOfRange(input.to_owned())),
    };
    if day == 0 || day > max_day || hour > 23 || minute > 59 || second > 59 {
        return Err(TimestampError::OutOfRange(input.to_owned()));
    }
    Ok(())
}

/// Days since 1970-01-01 for a proleptic-Gregorian civil date (Howard Hinnant).
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_index = i64::from(month) + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_index + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// Civil date for days since 1970-01-01 (Howard Hinnant).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = u32::try_from(day_of_year - (153 * month_index + 2) / 5 + 1).unwrap_or(1);
    let month = u32::try_from(if month_index < 10 { month_index + 3 } else { month_index - 9 }).unwrap_or(1);
    (if month <= 2 { year + 1 } else { year }, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_formats_canonically() {
        assert_eq!(Timestamp::from_unix_secs(0).as_str(), "1970-01-01T00:00:00Z");
        assert_eq!(
            Timestamp::from_unix_secs(1_767_000_000).as_str(),
            "2025-12-29T09:20:00Z"
        );

        // Known anchor: 2026-01-01T00:00:00Z == 1_767_225_600.
        assert_eq!(
            Timestamp::from_unix_secs(1_767_225_600).as_str(),
            "2026-01-01T00:00:00Z"
        );
    }

    #[test]
    fn offset_and_fraction_normalise_to_utc() {
        let timestamp = Timestamp::parse("2026-11-06T03:15:41-06:00").expect("valid rfc3339");
        assert_eq!(timestamp.as_str(), "2026-11-06T09:15:41Z");
        let with_fraction = Timestamp::parse("2026-11-06T09:15:41.987Z").expect("valid rfc3339");
        assert_eq!(with_fraction.as_str(), "2026-11-06T09:15:41Z");
    }

    #[test]
    fn rejects_malformed_and_out_of_range() {
        assert!(Timestamp::parse("2026-13-01T00:00:00Z").is_err());
        assert!(Timestamp::parse("2026-02-30T00:00:00Z").is_err());
        assert!(Timestamp::parse("2026-11-06 09:15:41Z").is_err());
        assert!(Timestamp::parse("not-a-timestamp").is_err());
        assert!(Timestamp::parse("2026-11-06T09:15:41+25:00").is_err());
    }

    #[test]
    fn leap_day_roundtrips() {
        let leap = Timestamp::from_unix_secs(1_709_164_800);
        assert_eq!(leap.as_str(), "2024-02-29T00:00:00Z");
        assert_eq!(leap.unix_secs().expect("parses"), 1_709_164_800);
    }

    #[test]
    fn civil_conversion_is_inverse() {
        for day in [-1000_i64, -1, 0, 1, 10_000, 20_000] {
            let (year, month, date) = civil_from_days(day);
            assert_eq!(days_from_civil(year, month, date), day);
        }
    }
}
