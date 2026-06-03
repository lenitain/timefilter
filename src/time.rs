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
