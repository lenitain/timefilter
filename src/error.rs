//! Error type shared by every parser in this crate.

use std::error::Error;
use std::fmt;

/// Errors produced while parsing time expressions and filters.
///
/// Variants carry no heap-allocated data, so error handling stays allocation
/// free. The enum is `#[non_exhaustive]`; new variants may be added without a
/// breaking change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TimeError {
    /// The filter expression does not begin with a comparison operator.
    MissingOperator,
    /// The expression begins with a malformed or unsupported comparison
    /// operator.
    InvalidOperator,
    /// The expression ends after its comparison operator.
    MissingValue,
    /// The input is empty or contains only whitespace.
    EmptyInput,
    /// A numeric component is malformed (empty, contains junk, or is not a number).
    InvalidNumber,
    /// A duration unit is not recognised.
    UnknownUnit,
    /// A unit is recognised but has no fixed-length representation.
    ///
    /// Years and months vary in length, so they cannot be converted to a
    /// duration without a calendar and a reference date.
    UnsupportedUnit,
    /// A duration expression is structurally invalid.
    ///
    /// Examples: out-of-order components (`PT1M1H`), repeated components
    /// (`PT1H1H`), an empty time part (`P1DT`), or a missing unit (`7`).
    InvalidDuration,
    /// A date/time string does not match any supported format.
    InvalidDate,
    /// The value does not fit the representable range.
    Overflow,
}

impl fmt::Display for TimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            TimeError::MissingOperator => "time filter must start with one of >, >=, <, <=",
            TimeError::InvalidOperator => "invalid or unsupported comparison operator",
            TimeError::MissingValue => "missing value after comparison operator",
            TimeError::EmptyInput => "empty input",
            TimeError::InvalidNumber => "invalid number",
            TimeError::UnknownUnit => "unknown time unit",
            TimeError::UnsupportedUnit => {
                "unsupported time unit: years and months are not fixed durations"
            }
            TimeError::InvalidDuration => "invalid duration expression",
            TimeError::InvalidDate => "invalid date/time",
            TimeError::Overflow => "value out of range",
        })
    }
}

impl Error for TimeError {}

/// `Result` alias for time parsing operations.
pub type TimeResult<T> = Result<T, TimeError>;
