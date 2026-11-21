//! Absolute time parsing and formatting.
//!
//! All absolute times are interpreted as UTC. Timezone-aware input must carry
//! an RFC 3339 offset; every other accepted format is naive and therefore read
//! as UTC.

use jiff::{Timestamp, civil, tz::TimeZone};

use crate::duration::{looks_like_duration, parse_duration};
use crate::error::{TimeError, TimeResult};

/// Naive formats accepted in addition to jiff's RFC 3339 timestamp parser.
const NAIVE_DATETIME_FORMATS: &[&str] = &[
    "%Y-%m-%dT%H:%M:%S%.f",
    "%Y-%m-%d %H:%M:%S%.f",
    "%Y-%m-%dT%H:%M",
    "%Y-%m-%d %H:%M",
    "%Y-%m-%d",
];

/// Parse an absolute time expression into a UTC timestamp.
///
/// Relative expressions are intentionally rejected: parsing is pure and does
/// not read the clock. Use [`parse_time_now`] to resolve a relative expression
/// against the current time, or [`parse_time_at`] for a deterministic
/// reference point.
///
/// Accepted formats:
///
/// * RFC 3339: `2024-05-01T10:00:00Z`, `2024-05-01T10:00:00+08:00`
/// * `2024-05-01T10:00`, `2024-05-01 10:00`
/// * `2024-05-01T10:00:00`, `2024-05-01 10:00:00`, with optional fraction
/// * `2024-05-01` (midnight UTC)
///
/// ```
/// use timefilter::parse_time;
///
/// let t = parse_time("2024-05-01 10:30").unwrap();
/// assert_eq!(t.to_string(), "2024-05-01T10:30:00Z");
///
/// let t = parse_time("2024-05-01T10:30:00+08:00").unwrap();
/// assert_eq!(t.to_string(), "2024-05-01T02:30:00Z");
/// ```
///
/// # Errors
///
/// Returns [`TimeError::EmptyInput`] for blank input and
/// [`TimeError::InvalidDate`] for unrecognised dates, including relative
/// expressions such as `2h`.
pub fn parse_time(s: &str) -> TimeResult<Timestamp> {
    let s = s.trim();
    if s.is_empty() {
        return Err(TimeError::EmptyInput);
    }
    parse_absolute(s)
}

/// Parse an absolute or relative time expression against the current time.
///
/// This is the clock-reading convenience; [`parse_time_at`] is deterministic.
///
/// ```
/// use timefilter::parse_time_now;
///
/// // Relative expressions resolve to `now - duration`.
/// let _two_hours_ago = parse_time_now("2h").unwrap();
/// ```
///
/// # Errors
///
/// See [`parse_time_at`], plus [`TimeError::Overflow`] when the system clock
/// is outside the representable timestamp range.
pub fn parse_time_now(s: &str) -> TimeResult<Timestamp> {
    if s.trim().is_empty() {
        return Err(TimeError::EmptyInput);
    }
    parse_time_at(s, current_timestamp()?)
}

/// Parse a time expression relative to an explicit reference time.
///
/// Absolute expressions ignore `now`; relative expressions resolve as
/// `now - duration`. Days and weeks are fixed 24-hour and 7-day durations; no
/// calendar or DST rules are applied.
///
/// ```
/// use jiff::{SignedDuration, Timestamp};
/// use timefilter::parse_time_at;
///
/// let now: Timestamp = "2024-06-15T12:00:00Z".parse().unwrap();
/// let expected = now.checked_sub(SignedDuration::from_hours(2)).unwrap();
/// assert_eq!(parse_time_at("2h", now).unwrap(), expected);
/// ```
///
/// # Errors
///
/// Returns [`TimeError::EmptyInput`] for blank input, duration errors for
/// malformed relative expressions, [`TimeError::InvalidDate`] for unrecognised
/// dates, and [`TimeError::Overflow`] when the result falls outside the
/// representable timestamp range.
pub fn parse_time_at(s: &str, now: Timestamp) -> TimeResult<Timestamp> {
    let s = s.trim();
    if s.is_empty() {
        return Err(TimeError::EmptyInput);
    }

    match parse_duration(s) {
        Ok(span) => now.checked_sub(span).map_err(|_| TimeError::Overflow),
        Err(duration_error) => parse_absolute(s).map_err(|date_error| {
            if looks_like_duration(s) {
                duration_error
            } else {
                date_error
            }
        }),
    }
}

fn current_timestamp() -> TimeResult<Timestamp> {
    Timestamp::try_from(std::time::SystemTime::now()).map_err(|_| TimeError::Overflow)
}

/// Parse an absolute timestamp, rejecting relative expressions.
pub(crate) fn parse_absolute(s: &str) -> TimeResult<Timestamp> {
    // A timestamp requires an offset; this handles `Z`, `+08:00` and similar.
    if let Ok(timestamp) = s.parse::<Timestamp>() {
        return Ok(timestamp);
    }

    // Naive formats. `strptime` requires the whole input to be consumed, so an
    // invalid or unsupported offset is rejected instead of being ignored.
    for format in NAIVE_DATETIME_FORMATS {
        if let Ok(datetime) = civil::DateTime::strptime(format, s) {
            if let Ok(zoned) = datetime.to_zoned_datetime(TimeZone::UTC) {
                return Ok(zoned.timestamp());
            }
        }
    }

    Err(TimeError::InvalidDate)
}

/// Format a UTC timestamp as RFC 3339.
///
/// The result is unambiguous and can be parsed back with [`parse_time`].
///
/// ```
/// use jiff::Timestamp;
/// use timefilter::format_datetime;
///
/// let t: Timestamp = "2024-05-01T10:30:45Z".parse().unwrap();
/// assert_eq!(format_datetime(&t), "2024-05-01T10:30:45Z");
///
/// let t: Timestamp = "2024-05-01T10:30:45.250Z".parse().unwrap();
/// assert_eq!(format_datetime(&t), "2024-05-01T10:30:45.25Z");
/// ```
#[must_use]
pub fn format_datetime(datetime: &Timestamp) -> String {
    datetime.to_string()
}
