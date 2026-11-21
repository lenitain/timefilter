//! Time filters: an operator plus either an absolute instant or a relative span.

use std::fmt;
use std::str::FromStr;

use jiff::{SignedDuration, Timestamp};

use crate::datetime::parse_absolute;
use crate::duration::{looks_like_duration, parse_duration, write_duration};
use crate::error::{TimeError, TimeResult};
use crate::op::TimeOp;

/// A time filter such as `>=7d` or `<2024-05-01T10:00:00Z`.
///
/// Relative filters keep the span, not a pre-resolved timestamp. They are
/// resolved against a reference time when checked, so a filter parsed once can
/// be reused indefinitely:
///
/// ```
/// use jiff::{SignedDuration, Timestamp};
/// use timefilter::TimeFilter;
///
/// let filter: TimeFilter = ">=7d".parse().unwrap();
/// let now: Timestamp = "2024-06-15T12:00:00Z".parse().unwrap();
///
/// let three_days_ago = now.checked_sub(SignedDuration::from_hours(3 * 24)).unwrap();
/// let ten_days_ago = now.checked_sub(SignedDuration::from_hours(10 * 24)).unwrap();
/// assert!(filter.matches_at(three_days_ago, now));
/// assert!(!filter.matches_at(ten_days_ago, now));
/// ```
///
/// Absolute filters compare the value with a fixed instant. Naive input is
/// interpreted as UTC; use RFC 3339 to supply an offset.
///
/// A relative span is signed: a negative span places the threshold in the
/// future. Time filters support ordering only; equality is not offered because
/// a timestamp comparison at nanosecond resolution would almost never match.
///
/// The enum is `#[non_exhaustive]`: new filter kinds may be added without a
/// breaking change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TimeFilter {
    /// Comparison against a fixed instant.
    Absolute {
        /// Comparison operator.
        op: TimeOp,
        /// Fixed threshold.
        time: Timestamp,
    },
    /// Comparison against `reference - span`.
    Relative {
        /// Comparison operator.
        op: TimeOp,
        /// Span subtracted from the reference time.
        span: SignedDuration,
    },
}

/// Where a resolved threshold sits relative to the representable range.
enum Threshold {
    Below,
    Above,
    At(Timestamp),
}

impl TimeFilter {
    /// Create an absolute filter.
    #[must_use]
    pub fn new(op: TimeOp, time: Timestamp) -> Self {
        TimeFilter::Absolute { op, time }
    }

    /// Create a relative filter evaluated as `reference - span`.
    ///
    /// A negative span places the threshold in the future; the comparison is
    /// still against the resolved timestamp.
    #[must_use]
    pub const fn relative(op: TimeOp, span: SignedDuration) -> Self {
        TimeFilter::Relative { op, span }
    }

    /// Filter: `value > time`.
    #[must_use]
    pub fn gt(time: Timestamp) -> Self {
        TimeFilter::new(TimeOp::Gt, time)
    }

    /// Filter: `value >= time`.
    #[must_use]
    pub fn ge(time: Timestamp) -> Self {
        TimeFilter::new(TimeOp::Ge, time)
    }

    /// Filter: `value < time`.
    #[must_use]
    pub fn lt(time: Timestamp) -> Self {
        TimeFilter::new(TimeOp::Lt, time)
    }

    /// Filter: `value <= time`.
    #[must_use]
    pub fn le(time: Timestamp) -> Self {
        TimeFilter::new(TimeOp::Le, time)
    }

    /// Relative filter: `value > reference - span`.
    #[must_use]
    pub const fn gt_span(span: SignedDuration) -> Self {
        TimeFilter::relative(TimeOp::Gt, span)
    }

    /// Relative filter: `value >= reference - span`.
    #[must_use]
    pub const fn ge_span(span: SignedDuration) -> Self {
        TimeFilter::relative(TimeOp::Ge, span)
    }

    /// Relative filter: `value < reference - span`.
    #[must_use]
    pub const fn lt_span(span: SignedDuration) -> Self {
        TimeFilter::relative(TimeOp::Lt, span)
    }

    /// Relative filter: `value <= reference - span`.
    #[must_use]
    pub const fn le_span(span: SignedDuration) -> Self {
        TimeFilter::relative(TimeOp::Le, span)
    }

    /// Relative filter matching timestamps no older than `span`.
    ///
    /// This is [`TimeFilter::ge_span`] with an intent-revealing name:
    /// `value >= reference - span`.
    #[must_use]
    pub const fn newer_than(span: SignedDuration) -> Self {
        TimeFilter::ge_span(span)
    }

    /// Relative filter matching timestamps older than `span`.
    ///
    /// This is [`TimeFilter::lt_span`] with an intent-revealing name:
    /// `value < reference - span`.
    #[must_use]
    pub const fn older_than(span: SignedDuration) -> Self {
        TimeFilter::lt_span(span)
    }

    /// Alias for [`TimeFilter::newer_than`]: match timestamps within `span`.
    #[must_use]
    pub const fn within(span: SignedDuration) -> Self {
        TimeFilter::ge_span(span)
    }

    /// The comparison operator.
    #[must_use]
    pub const fn op(self) -> TimeOp {
        match self {
            TimeFilter::Absolute { op, .. } | TimeFilter::Relative { op, .. } => op,
        }
    }

    /// Whether this filter is relative.
    #[must_use]
    pub const fn is_relative(self) -> bool {
        matches!(self, TimeFilter::Relative { .. })
    }

    /// The absolute threshold, if this is an absolute filter.
    #[must_use]
    pub const fn time(self) -> Option<Timestamp> {
        match self {
            TimeFilter::Absolute { time, .. } => Some(time),
            TimeFilter::Relative { .. } => None,
        }
    }

    /// The relative span, if this is a relative filter.
    #[must_use]
    pub const fn span(self) -> Option<SignedDuration> {
        match self {
            TimeFilter::Relative { span, .. } => Some(span),
            TimeFilter::Absolute { .. } => None,
        }
    }

    /// Resolve the threshold against `now`.
    ///
    /// Returns `None` when `now - span` falls outside the representable
    /// timestamp range. [`TimeFilter::matches_at`] still evaluates such
    /// filters mathematically.
    #[must_use]
    pub fn resolve(self, now: Timestamp) -> Option<Timestamp> {
        match self.threshold(now) {
            Threshold::At(threshold) => Some(threshold),
            Threshold::Below | Threshold::Above => None,
        }
    }

    /// Check `value` against this filter at the current time.
    ///
    /// # Panics
    ///
    /// Panics only for relative filters when the system clock is outside the
    /// representable timestamp range (years -9999 to 9999). Use
    /// [`TimeFilter::try_matches`] for a fallible form, or
    /// [`TimeFilter::matches_at`] when the reference time should be controlled
    /// by the caller.
    #[must_use]
    pub fn matches(self, value: Timestamp) -> bool {
        self.try_matches(value)
            .expect("system clock is outside the representable timestamp range")
    }

    /// Check `value` against this filter at the current time, fallibly.
    ///
    /// Returns [`TimeError::Overflow`] when a relative filter needs the
    /// current time and the system clock is outside the representable
    /// timestamp range. This avoids the panic in [`TimeFilter::matches`].
    pub fn try_matches(self, value: Timestamp) -> TimeResult<bool> {
        let now =
            Timestamp::try_from(std::time::SystemTime::now()).map_err(|_| TimeError::Overflow)?;
        Ok(self.matches_at(value, now))
    }

    /// Check `value` against this filter at an explicit reference time.
    #[must_use]
    pub fn matches_at(self, value: Timestamp, now: Timestamp) -> bool {
        match self.threshold(now) {
            Threshold::At(threshold) => self.op().applies(value, threshold),
            // The threshold lies below every representable instant.
            Threshold::Below => matches!(self.op(), TimeOp::Gt | TimeOp::Ge),
            // The threshold lies above every representable instant.
            Threshold::Above => matches!(self.op(), TimeOp::Lt | TimeOp::Le),
        }
    }

    fn threshold(self, now: Timestamp) -> Threshold {
        match self {
            TimeFilter::Absolute { time, .. } => Threshold::At(time),
            TimeFilter::Relative { span, .. } => match now.checked_sub(span) {
                Ok(time) => Threshold::At(time),
                Err(_) if span > SignedDuration::ZERO => Threshold::Below,
                Err(_) => Threshold::Above,
            },
        }
    }
}

impl fmt::Display for TimeFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = self.op();
        match self {
            TimeFilter::Absolute { time, .. } => write!(f, "{op}{time}"),
            TimeFilter::Relative { span, .. } => {
                write!(f, "{op}")?;
                write_duration(f, *span)
            }
        }
    }
}

impl FromStr for TimeFilter {
    type Err = TimeError;

    fn from_str(s: &str) -> TimeResult<Self> {
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

/// Parse a filter expression such as `">=7d"` or `"<2024-05-01"`.
///
/// Whitespace after the operator is allowed. A bare duration is relative; a
/// date/time is absolute. Equality is not supported; use a range instead.
///
/// ```
/// use timefilter::{TimeFilter, TimeOp};
///
/// let f: TimeFilter = ">=7d".parse().unwrap();
/// assert_eq!(f.op(), TimeOp::Ge);
/// assert!(f.is_relative());
///
/// let f: TimeFilter = "<2024-05-01 10:00".parse().unwrap();
/// assert!(!f.is_relative());
/// ```
///
/// # Errors
///
/// Returns [`TimeError::MissingOperator`] when no operator is present,
/// [`TimeError::InvalidOperator`] for malformed or unsupported operators,
/// [`TimeError::MissingValue`] when the operator has no value, and parsing
/// errors from the value part.
pub fn parse_time_filter(s: &str) -> TimeResult<TimeFilter> {
    let s = s.trim();
    if s.is_empty() {
        return Err(TimeError::EmptyInput);
    }
    let (op, rest) = match TimeOp::strip_prefix(s) {
        Some(parts) => parts,
        None if TimeOp::looks_like_operator(s) => return Err(TimeError::InvalidOperator),
        None => return Err(TimeError::MissingOperator),
    };
    let rest = rest.trim();
    if rest.is_empty() {
        return Err(TimeError::MissingValue);
    }
    if TimeOp::looks_like_operator(rest) {
        return Err(TimeError::InvalidOperator);
    }

    match parse_duration(rest) {
        Ok(span) => Ok(TimeFilter::relative(op, span)),
        Err(duration_error) => parse_absolute(rest)
            .map(|time| TimeFilter::new(op, time))
            .map_err(|date_error| {
                if looks_like_duration(rest) {
                    duration_error
                } else {
                    date_error
                }
            }),
    }
}
