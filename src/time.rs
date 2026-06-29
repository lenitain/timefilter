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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    op: TimeOp,
    time: DateTime<Utc>,
}

impl TimeFilter {
    /// Create a new filter from an operator and threshold time.
    #[inline]
    #[must_use]
    pub const fn new(op: TimeOp, time: DateTime<Utc>) -> Self {
        TimeFilter { op, time }
    }

    /// Get the comparison operator.
    #[inline]
    #[must_use]
    pub const fn op(self) -> TimeOp {
        self.op
    }

    /// Get the threshold time.
    #[inline]
    #[must_use]
    pub fn time(self) -> DateTime<Utc> {
        self.time
    }

    /// Filter: `value > threshold`.
    #[inline]
    #[must_use]
    pub const fn gt(time: DateTime<Utc>) -> Self {
        TimeFilter {
            op: TimeOp::Gt,
            time,
        }
    }

    /// Filter: `value >= threshold`.
    #[inline]
    #[must_use]
    pub const fn ge(time: DateTime<Utc>) -> Self {
        TimeFilter {
            op: TimeOp::Ge,
            time,
        }
    }

    /// Filter: `value < threshold`.
    #[inline]
    #[must_use]
    pub const fn lt(time: DateTime<Utc>) -> Self {
        TimeFilter {
            op: TimeOp::Lt,
            time,
        }
    }

    /// Filter: `value <= threshold`.
    #[inline]
    #[must_use]
    pub const fn le(time: DateTime<Utc>) -> Self {
        TimeFilter {
            op: TimeOp::Le,
            time,
        }
    }

    /// Filter: `value == threshold`.
    #[inline]
    #[must_use]
    pub const fn eq(time: DateTime<Utc>) -> Self {
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
        // Serialize as "op UTC_time" to preserve timezone
        serializer.collect_str(&format!("{}{}", self.op, self.time.format("%Y-%m-%d %H:%M:%S")))
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
            TimeError::UnknownSuffix => "unknown time suffix",
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
    let prefixes: &[(&str, TimeOp)] = &[
        (">=", TimeOp::Ge),
        ("<=", TimeOp::Le),
        (">", TimeOp::Gt),
        ("<", TimeOp::Lt),
        ("=", TimeOp::Eq),
    ];
    let (op, rest) = prefixes
        .iter()
        .find_map(|&(prefix, op)| s.strip_prefix(prefix).map(|r| (op, r)))
        .ok_or(TimeError::MissingOperator)?;
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
    let duration = parse_duration_inner(s)?;
    Some(Utc::now() - duration)
}

/// Parse human-readable duration string to `Duration`.
///
/// Supports relative formats:
/// - `"7d"`, `"7 days"` — days
/// - `"2h"`, `"2hr"` — hours
/// - `"30m"`, `"30min"` — minutes
/// - `"30s"` — seconds
///
/// Also supports ISO 8601 duration format:
/// - `"P7D"` — 7 days
/// - `"PT2H"` — 2 hours
/// - `"P1DT12H"` — 1 day and 12 hours
///
/// ```
/// use timefilter::parse_duration;
/// use chrono::Duration;
///
/// // Human-readable formats
/// assert_eq!(parse_duration("7d").unwrap(), Duration::days(7));
/// assert_eq!(parse_duration("2h").unwrap(), Duration::hours(2));
/// assert_eq!(parse_duration("30m").unwrap(), Duration::minutes(30));
/// assert_eq!(parse_duration("30s").unwrap(), Duration::seconds(30));
///
/// // Verbose suffixes
/// assert_eq!(parse_duration("1 day").unwrap(), Duration::days(1));
/// assert_eq!(parse_duration("2 hours").unwrap(), Duration::hours(2));
///
/// // ISO 8601 duration format
/// assert_eq!(parse_duration("P7D").unwrap(), Duration::days(7));
/// assert_eq!(parse_duration("PT2H").unwrap(), Duration::hours(2));
/// assert_eq!(parse_duration("P1DT12H").unwrap(), Duration::days(1) + Duration::hours(12));
/// ```
///
/// # Errors
///
/// Returns [`TimeError::EmptyInput`], [`TimeError::UnknownSuffix`],
/// or [`TimeError::InvalidNumber`].
pub fn parse_duration(s: &str) -> TimeResult<Duration> {
    let s = s.trim();
    if s.is_empty() {
        return Err(TimeError::EmptyInput);
    }

    // Try ISO 8601 duration format first
    if s.starts_with('P') || s.starts_with('p') {
        return parse_iso8601_duration(s);
    }

    // Try human-readable format
    parse_duration_inner(s).ok_or(TimeError::UnknownSuffix)
}

fn parse_duration_inner(s: &str) -> Option<Duration> {
    // We need to split digits from suffix. Find where alphabetic part starts.
    let alpha_pos = s.find(|c: char| c.is_ascii_alphabetic())?;
    let (num_str, suffix) = s.split_at(alpha_pos);
    let num: i64 = num_str.trim().parse().ok()?;

    let suf = suffix.trim();
    if suf.eq_ignore_ascii_case("d")
        || suf.eq_ignore_ascii_case("day")
        || suf.eq_ignore_ascii_case("days")
    {
        Some(Duration::days(num))
    } else if suf.eq_ignore_ascii_case("h")
        || suf.eq_ignore_ascii_case("hr")
        || suf.eq_ignore_ascii_case("hour")
        || suf.eq_ignore_ascii_case("hours")
    {
        Some(Duration::hours(num))
    } else if suf.eq_ignore_ascii_case("m")
        || suf.eq_ignore_ascii_case("min")
        || suf.eq_ignore_ascii_case("minute")
        || suf.eq_ignore_ascii_case("minutes")
    {
        Some(Duration::minutes(num))
    } else if suf.eq_ignore_ascii_case("s")
        || suf.eq_ignore_ascii_case("sec")
        || suf.eq_ignore_ascii_case("second")
        || suf.eq_ignore_ascii_case("seconds")
    {
        Some(Duration::seconds(num))
    } else {
        None
    }
}

fn parse_iso8601_duration(s: &str) -> TimeResult<Duration> {
    // Trim leading 'P' (case-insensitive) without allocating
    let s = if s.as_bytes().first() == Some(&b'P') || s.as_bytes().first() == Some(&b'p') {
        &s[1..]
    } else {
        return Err(TimeError::UnknownSuffix);
    };

    if s.is_empty() {
        return Err(TimeError::InvalidNumber);
    }

    let mut total_seconds = 0i64;
    let mut num_start: Option<usize> = None;
    let mut in_time = false;
    let bytes = s.as_bytes();

    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'T' | b't' => {
                in_time = true;
                if num_start.is_some() {
                    return Err(TimeError::InvalidNumber);
                }
            }
            b'0'..=b'9' => {
                if num_start.is_none() {
                    num_start = Some(i);
                }
            }
            b'D' | b'd' => {
                let Some(start) = num_start.take() else {
                    return Err(TimeError::InvalidNumber);
                };
                let days: i64 = s[start..i].parse().map_err(|_| TimeError::InvalidNumber)?;
                total_seconds += days * 86400;
            }
            b'H' | b'h' => {
                let Some(start) = num_start.take() else {
                    return Err(TimeError::InvalidNumber);
                };
                if !in_time {
                    return Err(TimeError::InvalidNumber);
                }
                let hours: i64 = s[start..i].parse().map_err(|_| TimeError::InvalidNumber)?;
                total_seconds += hours * 3600;
            }
            b'M' | b'm' => {
                let Some(start) = num_start.take() else {
                    return Err(TimeError::InvalidNumber);
                };
                if !in_time {
                    return Err(TimeError::InvalidNumber);
                }
                let minutes: i64 = s[start..i].parse().map_err(|_| TimeError::InvalidNumber)?;
                total_seconds += minutes * 60;
            }
            b'S' | b's' => {
                let Some(start) = num_start.take() else {
                    return Err(TimeError::InvalidNumber);
                };
                if !in_time {
                    return Err(TimeError::InvalidNumber);
                }
                let seconds: i64 = s[start..i].parse().map_err(|_| TimeError::InvalidNumber)?;
                total_seconds += seconds;
            }
            _ => {
                return Err(TimeError::UnknownSuffix);
            }
        }
    }

    // If there's remaining unparsed number, it's an error
    if num_start.is_some() {
        return Err(TimeError::InvalidNumber);
    }

    // Must have parsed at least some duration
    if total_seconds == 0 {
        return Err(TimeError::InvalidNumber);
    }

    Ok(Duration::seconds(total_seconds))
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
        let Some(naive) = naive_date.and_hms_opt(0, 0, 0) else {
            return Err(TimeError::InvalidDate);
        };
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
