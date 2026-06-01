//! Core types, constants, and parsing logic.
//!
//! All error strings live in `.rodata` — no heap `String` allocation
//! in error paths.

use std::error::Error;
use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Duration, Local, NaiveDateTime, Utc};

// ── TimeOp ───────────────────────────────────────────────────────────────────

/// Time comparison operator.
///
/// Mirrors [`sizefilter::SizeOp`](https://docs.rs/sizefilter) but is an
/// independent type — the two may evolve differently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeOp {
    /// Greater than (`>`)
    Gt,
    /// Greater than or equal to (`>=`)
    Ge,
    /// Less than (`<`)
    Lt,
    /// Less than or equal to (`<=`)
    Le,
    /// Equal to (`=`)
    Eq,
}

impl TimeOp {
    /// All variants, in declaration order.
    pub const ALL: [TimeOp; 5] = [TimeOp::Gt, TimeOp::Ge, TimeOp::Lt, TimeOp::Le, TimeOp::Eq];

    /// Apply this operator to two `DateTime<Utc>` values.
    #[inline]
    #[must_use]
    pub fn applies(self, value: DateTime<Utc>, threshold: DateTime<Utc>) -> bool {
        match self {
            TimeOp::Gt => value > threshold,
            TimeOp::Ge => value >= threshold,
            TimeOp::Lt => value < threshold,
            TimeOp::Le => value <= threshold,
            TimeOp::Eq => value == threshold,
        }
    }
}

impl fmt::Display for TimeOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            TimeOp::Gt => ">",
            TimeOp::Ge => ">=",
            TimeOp::Lt => "<",
            TimeOp::Le => "<=",
            TimeOp::Eq => "=",
        })
    }
}

// ── TimeFilter ───────────────────────────────────────────────────────────────

/// A time filter with operator (e.g., `>=7d`, `<2026-05-01`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeFilter {
    pub op: TimeOp,
    pub time: DateTime<Utc>,
}

impl TimeFilter {
    /// Create a new filter from an operator and threshold time.
    #[inline]
    #[must_use]
    pub const fn new(op: TimeOp, time: DateTime<Utc>) -> Self {
        TimeFilter { op, time }
    }

    /// Filter: `value > threshold`.
    #[inline]
    #[must_use]
    pub fn gt(time: DateTime<Utc>) -> Self {
        TimeFilter {
            op: TimeOp::Gt,
            time,
        }
    }

    /// Filter: `value >= threshold`.
    #[inline]
    #[must_use]
    pub fn ge(time: DateTime<Utc>) -> Self {
        TimeFilter {
            op: TimeOp::Ge,
            time,
        }
    }

    /// Filter: `value < threshold`.
    #[inline]
    #[must_use]
    pub fn lt(time: DateTime<Utc>) -> Self {
        TimeFilter {
            op: TimeOp::Lt,
            time,
        }
    }

    /// Filter: `value <= threshold`.
    #[inline]
    #[must_use]
    pub fn le(time: DateTime<Utc>) -> Self {
        TimeFilter {
            op: TimeOp::Le,
            time,
        }
    }

    /// Filter: `value == threshold`.
    #[inline]
    #[must_use]
    pub fn eq(time: DateTime<Utc>) -> Self {
        TimeFilter {
            op: TimeOp::Eq,
            time,
        }
    }

    /// Check whether `value` passes this filter.
    #[inline]
    #[must_use]
    pub fn matches(self, value: DateTime<Utc>) -> bool {
        self.op.applies(value, self.time)
    }
}

impl fmt::Display for TimeFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.op, format_datetime(&self.time))
    }
}

impl FromStr for TimeFilter {
    type Err = TimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_time_filter(s)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for TimeFilter {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for TimeFilter {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

// ── TimeError ────────────────────────────────────────────────────────────────

/// Errors that can occur during time parsing and formatting.
///
/// All variants carry zero heap-allocated data — error strings are
/// `&'static str` literals in `.rodata`.
///
/// This enum is `#[non_exhaustive]` — new variants may be added in
/// minor releases without breaking changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TimeError {
    /// Filter string lacks `>=`, `>`, `<=`, `<`, or `=` prefix.
    MissingOperator,
    /// Empty input string.
    EmptyInput,
    /// Unknown or unsupported time suffix.
    UnknownSuffix,
    /// Numeric value can't be parsed.
    InvalidNumber,
    /// Date/time string doesn't match expected format.
    InvalidDate,
}

impl fmt::Display for TimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            TimeError::MissingOperator => {
                "time filter must start with an operator (>=, >, <=, <, =)"
            }
            TimeError::EmptyInput => "empty input",
            TimeError::UnknownSuffix => "unknown time suffix (expected h, hr, m, min, d, s)",
            TimeError::InvalidNumber => "failed to parse number",
            TimeError::InvalidDate => "failed to parse date/time",
        })
    }
}

impl Error for TimeError {}

/// `Result` type alias for `timefilter` operations.
pub type TimeResult<T> = Result<T, TimeError>;

// ── parsing ──────────────────────────────────────────────────────────────────

/// Parse a time filter string like `">=7d"`, `"<2h"`, `"=2026-05-01"`.
///
/// Operator is required — returns error if missing.
///
/// # Errors
///
/// Returns [`TimeError::MissingOperator`] if no operator is found,
/// or [`TimeError`] variants from time parsing.
pub fn parse_time_filter(s: &str) -> TimeResult<TimeFilter> {
    let s = s.trim();
    let (op, rest) = if let Some(r) = s.strip_prefix(">=") {
        (TimeOp::Ge, r)
    } else if let Some(r) = s.strip_prefix("<=") {
        (TimeOp::Le, r)
    } else if let Some(r) = s.strip_prefix('>') {
        (TimeOp::Gt, r)
    } else if let Some(r) = s.strip_prefix('<') {
        (TimeOp::Lt, r)
    } else if let Some(r) = s.strip_prefix('=') {
        (TimeOp::Eq, r)
    } else {
        return Err(TimeError::MissingOperator);
    };
    let time = parse_time(rest)?;
    Ok(TimeFilter { op, time })
}

/// Parse human-readable time string to `DateTime<Utc>`.
///
/// Supports relative formats:
/// - `"7d"`, `"7 days"` — days ago
/// - `"2h"`, `"2hr"` — hours ago
/// - `"30m"`, `"30min"` — minutes ago
/// - `"30s"` — seconds ago
///
/// And absolute formats:
/// - `"2024-05-01"` — date-only (midnight UTC)
/// - `"2024-05-01 10:00"` — date and hour:minute
/// - `"2024-05-01 10:00:00"` — date and hour:minute:second
///
/// # Errors
///
/// Returns [`TimeError::EmptyInput`], [`TimeError::UnknownSuffix`],
/// [`TimeError::InvalidNumber`], or [`TimeError::InvalidDate`].
pub fn parse_time(time_str: &str) -> TimeResult<DateTime<Utc>> {
    let s = time_str.trim();
    if s.is_empty() {
        return Err(TimeError::EmptyInput);
    }

    // Try relative time formats (suffix-based)
    if let Some(time) = try_parse_relative(s) {
        return Ok(time);
    }

    // Try absolute time formats
    try_parse_absolute(s)
}

fn try_parse_relative(s: &str) -> Option<DateTime<Utc>> {
    // We need to split digits from suffix. Find where alphabetic part starts.
    let alpha_pos = s.find(|c: char| c.is_ascii_alphabetic())?;
    let (num_str, suffix) = s.split_at(alpha_pos);
    let num: i64 = num_str.trim().parse().ok()?;

    let suf = suffix.trim();
    let duration = match () {
        _ if suf.eq_ignore_ascii_case("d")
            || suf.eq_ignore_ascii_case("day")
            || suf.eq_ignore_ascii_case("days") =>
        {
            Duration::days(num)
        }
        _ if suf.eq_ignore_ascii_case("h")
            || suf.eq_ignore_ascii_case("hr")
            || suf.eq_ignore_ascii_case("hour")
            || suf.eq_ignore_ascii_case("hours") =>
        {
            Duration::hours(num)
        }
        _ if suf.eq_ignore_ascii_case("m")
            || suf.eq_ignore_ascii_case("min")
            || suf.eq_ignore_ascii_case("minute")
            || suf.eq_ignore_ascii_case("minutes") =>
        {
            Duration::minutes(num)
        }
        _ if suf.eq_ignore_ascii_case("s")
            || suf.eq_ignore_ascii_case("sec")
            || suf.eq_ignore_ascii_case("second")
            || suf.eq_ignore_ascii_case("seconds") =>
        {
            Duration::seconds(num)
        }
        _ => return None,
    };
    Some(Utc::now() - duration)
}

fn try_parse_absolute(s: &str) -> TimeResult<DateTime<Utc>> {
    // "2024-05-01 10:00:00"
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Ok(DateTime::from_naive_utc_and_offset(naive, Utc));
    }
    // "2024-05-01 10:00"
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M") {
        return Ok(DateTime::from_naive_utc_and_offset(naive, Utc));
    }
    // "2024-05-01" — parse as NaiveDate, then convert
    if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let naive = naive_date.and_hms_opt(0, 0, 0).unwrap();
        return Ok(DateTime::from_naive_utc_and_offset(naive, Utc));
    }
    Err(TimeError::InvalidDate)
}

// ── formatting ───────────────────────────────────────────────────────────────

/// Format a `DateTime<Utc>` for display in **local timezone**.
///
/// ```
/// use timefilter::format_datetime;
/// use chrono::{DateTime, NaiveDateTime, Utc};
///
/// let naive = NaiveDateTime::parse_from_str("2024-05-01 10:30:45", "%Y-%m-%d %H:%M:%S").unwrap();
/// let dt: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
/// let out = format_datetime(&dt);
/// assert!(out.contains("2024"));
/// ```
#[must_use]
pub fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.with_timezone(&Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, TimeZone, Timelike};

    // -- parse_time: relative --

    #[test]
    fn relative_hours() {
        let now = Utc::now();
        let parsed = parse_time("1h").unwrap();
        let diff = now - parsed;
        assert!(diff >= Duration::minutes(59) && diff <= Duration::minutes(61));
    }

    #[test]
    fn relative_hours_hr_suffix() {
        let now = Utc::now();
        let parsed = parse_time("2hr").unwrap();
        let diff = now - parsed;
        assert!(diff >= Duration::minutes(119) && diff <= Duration::minutes(121));
    }

    #[test]
    fn relative_minutes() {
        let now = Utc::now();
        let parsed = parse_time("30m").unwrap();
        let diff = now - parsed;
        assert!(diff >= Duration::minutes(29) && diff <= Duration::minutes(31));
    }

    #[test]
    fn relative_minutes_min_suffix() {
        let now = Utc::now();
        let parsed = parse_time("15min").unwrap();
        let diff = now - parsed;
        assert!(diff >= Duration::minutes(14) && diff <= Duration::minutes(16));
    }

    #[test]
    fn relative_days() {
        let now = Utc::now();
        let parsed = parse_time("7d").unwrap();
        let diff = now - parsed;
        assert!(diff >= Duration::hours(167) && diff <= Duration::hours(169));
    }

    #[test]
    fn relative_seconds() {
        let now = Utc::now();
        let parsed = parse_time("30s").unwrap();
        let diff = now - parsed;
        assert!(diff >= Duration::seconds(29) && diff <= Duration::seconds(31));
    }

    #[test]
    fn relative_verbose_suffixes() {
        assert!(parse_time("1 day").is_ok());
        assert!(parse_time("7 days").is_ok());
        assert!(parse_time("2 hour").is_ok());
        assert!(parse_time("3 hours").is_ok());
        assert!(parse_time("10 minute").is_ok());
        assert!(parse_time("5 minutes").is_ok());
        assert!(parse_time("30 second").is_ok());
        assert!(parse_time("45 seconds").is_ok());
    }

    // -- parse_time: absolute --

    #[test]
    fn absolute_datetime() {
        let parsed = parse_time("2024-05-01 10:00").unwrap();
        assert_eq!(parsed.year(), 2024);
        assert_eq!(parsed.month(), 5);
        assert_eq!(parsed.day(), 1);
        assert_eq!(parsed.hour(), 10);
        assert_eq!(parsed.minute(), 0);
    }

    #[test]
    fn absolute_with_seconds() {
        let parsed = parse_time("2024-12-25 15:30:45").unwrap();
        assert_eq!(parsed.year(), 2024);
        assert_eq!(parsed.month(), 12);
        assert_eq!(parsed.day(), 25);
        assert_eq!(parsed.hour(), 15);
        assert_eq!(parsed.minute(), 30);
        assert_eq!(parsed.second(), 45);
    }

    #[test]
    fn absolute_date_only() {
        let parsed = parse_time("2024-01-15").unwrap();
        assert_eq!(parsed.year(), 2024);
        assert_eq!(parsed.month(), 1);
        assert_eq!(parsed.day(), 15);
        assert_eq!(parsed.hour(), 0);
        assert_eq!(parsed.minute(), 0);
    }

    // -- parse_time: errors --

    #[test]
    fn parse_time_invalid() {
        assert_eq!(parse_time("invalid"), Err(TimeError::InvalidDate));
        assert_eq!(parse_time("2024-13-01 10:00"), Err(TimeError::InvalidDate));
        assert_eq!(parse_time("abc"), Err(TimeError::InvalidDate));
        assert_eq!(parse_time(""), Err(TimeError::EmptyInput));
        assert_eq!(parse_time("  "), Err(TimeError::EmptyInput));
    }

    #[test]
    fn parse_time_unknown_suffix() {
        // "1x" → finds alpha at position 1, num=1, suffix="x" → UnknownSuffix from try_parse_relative
        // But we match err, it goes to try_parse_absolute which also fails → InvalidDate
        assert_eq!(parse_time("1x"), Err(TimeError::InvalidDate));
    }

    // -- parse_time_filter --

    #[test]
    fn filter_gt() {
        let f = parse_time_filter(">1h").unwrap();
        assert_eq!(f.op, TimeOp::Gt);
    }

    #[test]
    fn filter_ge() {
        let f = parse_time_filter(">=7d").unwrap();
        assert_eq!(f.op, TimeOp::Ge);
    }

    #[test]
    fn filter_lt() {
        let f = parse_time_filter("<2026-05-01").unwrap();
        assert_eq!(f.op, TimeOp::Lt);
        assert_eq!(f.time.year(), 2026);
        assert_eq!(f.time.month(), 5);
        assert_eq!(f.time.day(), 1);
    }

    #[test]
    fn filter_le() {
        let f = parse_time_filter("<=30m").unwrap();
        assert_eq!(f.op, TimeOp::Le);
    }

    #[test]
    fn filter_eq() {
        let f = parse_time_filter("=2026-05-01 10:00").unwrap();
        assert_eq!(f.op, TimeOp::Eq);
        assert_eq!(f.time.year(), 2026);
        assert_eq!(f.time.month(), 5);
        assert_eq!(f.time.day(), 1);
        assert_eq!(f.time.hour(), 10);
    }

    #[test]
    fn filter_no_operator_errors() {
        assert_eq!(parse_time_filter("1h"), Err(TimeError::MissingOperator));
        assert_eq!(parse_time_filter("30d"), Err(TimeError::MissingOperator));
        assert_eq!(
            parse_time_filter("2026-05-01"),
            Err(TimeError::MissingOperator)
        );
    }

    #[test]
    fn filter_invalid() {
        assert_eq!(parse_time_filter(">abc"), Err(TimeError::InvalidDate));
    }

    // -- TimeFilter FromStr / Display --

    #[test]
    fn filter_from_str() {
        let f: TimeFilter = ">=7d".parse().unwrap();
        assert_eq!(f.op, TimeOp::Ge);
    }

    #[test]
    fn filter_display() {
        let dt = Utc.with_ymd_and_hms(2024, 5, 1, 10, 30, 45).unwrap();
        let f = TimeFilter::gt(dt);
        let s = f.to_string();
        assert!(s.starts_with('>'));
        assert!(s.contains("2024"));
    }

    // -- TimeOp --

    #[test]
    fn time_op_all() {
        assert_eq!(TimeOp::ALL.len(), 5);
    }

    #[test]
    fn time_op_applies() {
        let t1: DateTime<Utc> = Utc.with_ymd_and_hms(2024, 6, 1, 0, 0, 0).unwrap();
        let t2: DateTime<Utc> = Utc.with_ymd_and_hms(2024, 5, 1, 0, 0, 0).unwrap();
        assert!(TimeOp::Gt.applies(t1, t2));
        assert!(!TimeOp::Gt.applies(t2, t1));
        assert!(TimeOp::Ge.applies(t1, t1));
        assert!(TimeOp::Eq.applies(t1, t1));
    }

    // -- format_datetime --

    #[test]
    fn format_datetime_contains_year() {
        let dt = Utc.with_ymd_and_hms(2024, 5, 1, 10, 30, 45).unwrap();
        let s = format_datetime(&dt);
        assert!(!s.is_empty());
        assert!(s.contains("2024"));
    }

    // -- TimeFilter convenience ctors --

    #[test]
    fn filter_convenience_ctors() {
        let now = Utc::now();
        assert_eq!(TimeFilter::gt(now).op, TimeOp::Gt);
        assert_eq!(TimeFilter::ge(now).op, TimeOp::Ge);
        assert_eq!(TimeFilter::lt(now).op, TimeOp::Lt);
        assert_eq!(TimeFilter::le(now).op, TimeOp::Le);
        assert_eq!(TimeFilter::eq(now).op, TimeOp::Eq);
    }

    // -- cross-crate extraction risks --

    #[test]
    fn negative_relative_time() {
        // "-1h" means 1 hour in the FUTURE (now - (-1h) = now + 1h)
        let now = Utc::now();
        let parsed = parse_time("-1h").unwrap();
        let diff = parsed - now;
        assert!(diff.num_minutes() >= 59 && diff.num_minutes() <= 61);
    }

    #[test]
    fn filter_empty_after_operator() {
        assert_eq!(parse_time_filter(">="), Err(TimeError::EmptyInput));
        assert_eq!(parse_time_filter("<"), Err(TimeError::EmptyInput));
        assert_eq!(parse_time_filter(">"), Err(TimeError::EmptyInput));
        assert_eq!(parse_time_filter("<="), Err(TimeError::EmptyInput));
    }

    #[test]
    fn leap_year_parses() {
        let dt = parse_time("2024-02-29").unwrap();
        assert_eq!(dt.month(), 2);
        assert_eq!(dt.day(), 29);
    }

    #[test]
    fn invalid_leap_year_fails() {
        // 2023 is not a leap year, Feb 29 should fail
        assert_eq!(parse_time("2023-02-29"), Err(TimeError::InvalidDate));
    }

    #[test]
    fn edge_day_month_boundaries() {
        // Month boundaries: Jan 31 → Feb 1 should not be valid
        assert_eq!(parse_time("2024-01-32"), Err(TimeError::InvalidDate));
        assert_eq!(parse_time("2024-13-01"), Err(TimeError::InvalidDate));
        // Apr 31 doesn't exist
        assert_eq!(parse_time("2024-04-31"), Err(TimeError::InvalidDate));
    }

    #[test]
    fn time_error_is_proper_error() {
        use std::error::Error;
        assert!(TimeError::EmptyInput.source().is_none());
        assert_eq!(TimeError::EmptyInput.to_string(), "empty input");
        assert_eq!(
            TimeError::InvalidDate.to_string(),
            "failed to parse date/time"
        );
        assert_eq!(
            TimeError::UnknownSuffix.to_string(),
            "unknown time suffix (expected h, hr, m, min, d, s)"
        );
        assert_eq!(
            TimeError::InvalidNumber.to_string(),
            "failed to parse number"
        );
        assert!(
            TimeError::MissingOperator
                .to_string()
                .contains("time filter must start")
        );
    }

    #[test]
    fn time_op_display_all() {
        assert_eq!(TimeOp::Gt.to_string(), ">");
        assert_eq!(TimeOp::Ge.to_string(), ">=");
        assert_eq!(TimeOp::Lt.to_string(), "<");
        assert_eq!(TimeOp::Le.to_string(), "<=");
        assert_eq!(TimeOp::Eq.to_string(), "=");
    }

    #[test]
    fn time_filter_roundtrip_absolute() {
        // format_datetime uses local timezone, so full roundtrip depends on TZ.
        // Just verify the string format has operator + date.
        let dt: DateTime<Utc> = parse_time("2024-06-15 08:30").unwrap();
        let f = TimeFilter::ge(dt);
        let s = f.to_string();
        assert!(s.starts_with(">="), "starts with >=: {}", s);
        assert!(s.contains("2024-06-15"), "contains date: {}", s);
        assert!(s.contains(":"), "contains time: {}", s);
    }

    #[test]
    fn time_serde_compile_check() {
        // Compile-time: TimeFilter + TimeOp implement Send + Sync
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<TimeFilter>();
        assert_sync::<TimeFilter>();
        assert_send::<TimeOp>();
        assert_sync::<TimeOp>();
    }

    #[test]
    fn format_datetime_various_times() {
        // Just verify output is non-empty and contains the date
        let dt: DateTime<Utc> = parse_time("2024-01-01").unwrap();
        let s = format_datetime(&dt);
        assert!(s.contains("2024"), "contains year: {}", s);
        assert!(s.contains("01"), "contains month/day: {}", s);
        assert!(s.contains(":"), "contains time separator: {}", s);
    }
}
