//! Comparison operators.

use std::fmt;
use std::str::FromStr;

use jiff::Timestamp;

use crate::error::{TimeError, TimeResult};

/// Comparison operator used by [`TimeFilter`](crate::TimeFilter).
///
/// Time filters support ordering only. Timestamps are continuous, so an
/// equality filter such as `=7d` would only match one exact nanosecond; use a
/// range (`>=day` and `<next_day`) instead.
///
/// The enum is `#[non_exhaustive]`: new operators may be added without a
/// breaking change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TimeOp {
    /// Greater than (`>`).
    Gt,
    /// Greater than or equal to (`>=`).
    Ge,
    /// Less than (`<`).
    Lt,
    /// Less than or equal to (`<=`).
    Le,
}

impl TimeOp {
    /// All variants, in display order.
    pub const ALL: [TimeOp; 4] = [TimeOp::Gt, TimeOp::Ge, TimeOp::Lt, TimeOp::Le];

    /// Apply this operator to `value` and `threshold`.
    #[inline]
    #[must_use]
    pub fn applies(self, value: Timestamp, threshold: Timestamp) -> bool {
        match self {
            TimeOp::Gt => value > threshold,
            TimeOp::Ge => value >= threshold,
            TimeOp::Lt => value < threshold,
            TimeOp::Le => value <= threshold,
        }
    }

    /// Split a leading operator off `s`.
    ///
    /// Two-character operators are matched before their one-character prefixes.
    pub(crate) fn strip_prefix(s: &str) -> Option<(TimeOp, &str)> {
        const PREFIXES: &[(&str, TimeOp)] = &[
            (">=", TimeOp::Ge),
            ("<=", TimeOp::Le),
            (">", TimeOp::Gt),
            ("<", TimeOp::Lt),
        ];
        PREFIXES
            .iter()
            .find_map(|&(prefix, op)| s.strip_prefix(prefix).map(|rest| (op, rest)))
    }

    /// Whether `s` starts with a character that belongs to an operator.
    ///
    /// Used to distinguish a missing operator from an unsupported or
    /// malformed one (`=`, `!=`, `!7d`).
    pub(crate) fn looks_like_operator(s: &str) -> bool {
        s.starts_with(['<', '>', '=', '!'])
    }
}

impl fmt::Display for TimeOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            TimeOp::Gt => ">",
            TimeOp::Ge => ">=",
            TimeOp::Lt => "<",
            TimeOp::Le => "<=",
        })
    }
}

impl FromStr for TimeOp {
    type Err = TimeError;

    /// Parse a bare operator such as `">="`.
    fn from_str(s: &str) -> TimeResult<Self> {
        let s = s.trim();
        if s.is_empty() {
            return Err(TimeError::EmptyInput);
        }
        match TimeOp::strip_prefix(s) {
            Some((op, "")) => Ok(op),
            Some(_) => Err(TimeError::InvalidOperator),
            None if TimeOp::looks_like_operator(s) => Err(TimeError::InvalidOperator),
            None => Err(TimeError::MissingOperator),
        }
    }
}
