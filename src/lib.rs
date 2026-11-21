//! `timefilter` — human-readable time parsing, filtering, and formatting.
//!
//! Handles filter expressions like `">=7d"` and `"<2h"`, timestamps like
//! `"2024-05-01"`, plus duration parsing and RFC 3339 formatting. Timestamps
//! and durations are [jiff](https://docs.rs/jiff) types, re-exported here as
//! [`Timestamp`] and [`SignedDuration`]. The whole jiff crate is re-exported as
//! [`jiff`] so callers stay on the same version.
//!
//! # Design
//!
//! A filter parsed from `">=7d"` stays relative: it stores the span and
//! resolves it against a reference time when checked. Parsing a config once
//! therefore keeps the intended meaning instead of freezing "seven days ago"
//! to the moment the process started.
//!
//! Parsing is pure. [`parse_time`] accepts absolute timestamps only;
//! [`parse_time_now`] is the explicit clock-reading convenience, and
//! [`parse_time_at`] takes the reference time from the caller. Likewise,
//! [`TimeFilter::matches_at`] is the deterministic evaluator and
//! [`TimeFilter::try_matches`] is the fallible current-time form.
//!
//! All absolute times are UTC. Naive date/time strings are read as UTC; supply
//! an RFC 3339 offset when the input carries a timezone.
//!
//! # Scope
//!
//! The crate parses human durations, absolute timestamps, and one ordering
//! filter. It is UTC-only, treats days and weeks as fixed 24-hour and 7-day
//! durations, and rejects calendar units. Time zones, DST, natural language,
//! recurrence, and query languages are out of scope.
//!
//! # Quick start
//!
//! ```
//! use jiff::{SignedDuration, Timestamp};
//! use timefilter::prelude::*;
//!
//! // Absolute time
//! let t = parse_time("2024-05-01 10:00").unwrap();
//! assert_eq!(t.to_string(), "2024-05-01T10:00:00Z");
//!
//! // Relative filter, resolved at check time
//! let f: TimeFilter = ">=7d".parse().unwrap();
//! assert_eq!(f.op(), TimeOp::Ge);
//! assert!(f.is_relative());
//!
//! let now: Timestamp = "2024-06-15T12:00:00Z".parse().unwrap();
//! let three_days_ago = now.checked_sub(SignedDuration::from_hours(3 * 24)).unwrap();
//! let ten_days_ago = now.checked_sub(SignedDuration::from_hours(10 * 24)).unwrap();
//! assert!(f.matches_at(three_days_ago, now));
//! assert!(!f.matches_at(ten_days_ago, now));
//! ```
//!
//! # Durations
//!
//! [`parse_duration`] accepts human readable and ISO 8601 durations; fractions
//! are supported and sub-nanosecond precision is truncated:
//!
//! ```
//! use jiff::SignedDuration;
//! use timefilter::{format_duration, parse_duration};
//!
//! assert_eq!(parse_duration("1h30m").unwrap(), SignedDuration::from_mins(90));
//! assert_eq!(parse_duration("1.5h").unwrap(), SignedDuration::from_mins(90));
//! assert_eq!(parse_duration("1w").unwrap(), SignedDuration::from_hours(7 * 24));
//! assert_eq!(parse_duration("P1DT12H").unwrap(), SignedDuration::from_hours(36));
//! assert_eq!(parse_duration("PT0.5S").unwrap(), SignedDuration::from_millis(500));
//! assert_eq!(parse_duration("-P1D").unwrap(), SignedDuration::from_hours(-24));
//!
//! // Canonical formatting round trips exactly
//! assert_eq!(format_duration(SignedDuration::from_mins(90)), "1h30m");
//! ```
//!
//! ISO weeks are standalone (`P1W`, not `P1W2D`). Years and months are rejected
//! because they have no fixed length:
//!
//! ```
//! use timefilter::{TimeError, parse_duration};
//!
//! assert_eq!(parse_duration("P1Y"), Err(TimeError::UnsupportedUnit));
//! assert_eq!(parse_duration("P1M"), Err(TimeError::UnsupportedUnit));
//! assert_eq!(parse_duration("P1W2D"), Err(TimeError::InvalidDuration));
//! assert_eq!(parse_duration("1year"), Err(TimeError::UnsupportedUnit));
//! ```
//!
//! # Absolute times
//!
//! ```
//! use timefilter::parse_time;
//!
//! parse_time("2024-05-01").unwrap();                       // midnight UTC
//! parse_time("2024-05-01 10:00").unwrap();
//! parse_time("2024-05-01T10:00:00.250").unwrap();
//! parse_time("2024-05-01T10:00:00+08:00").unwrap();        // offset honoured
//! ```
//!
//! # Filters
//!
//! The operator compares the timestamp, not the age. `>=7d` means
//! "timestamp at least as new as seven days ago", i.e. no older than seven
//! days:
//!
//! | Expression | Meaning |
//! | --- | --- |
//! | `>=7d` | timestamp >= now - 7d (no older than 7 days) |
//! | `<7d` | timestamp < now - 7d (older than 7 days) |
//! | `>=2024-05-01` | timestamp >= 2024-05-01T00:00:00Z |
//!
//! Intent-named constructors make the direction explicit:
//!
//! ```
//! use jiff::{SignedDuration, Timestamp};
//! use timefilter::TimeFilter;
//!
//! let now: Timestamp = "2024-06-15T12:00:00Z".parse().unwrap();
//!
//! let absolute: TimeFilter = ">=2024-06-01T00:00:00Z".parse().unwrap();
//! assert!(absolute.matches_at(now, now));
//!
//! let relative = TimeFilter::newer_than(SignedDuration::from_hours(2));
//! let half_an_hour_ago = now.checked_sub(SignedDuration::from_mins(30)).unwrap();
//! let three_hours_ago = now.checked_sub(SignedDuration::from_hours(3)).unwrap();
//! assert!(relative.matches_at(half_an_hour_ago, now));
//! assert!(!relative.matches_at(three_hours_ago, now));
//! ```
//!
//! [`TimeFilter::matches_at`] takes an explicit reference for deterministic
//! behaviour; [`TimeFilter::matches`] uses the current time and panics only if
//! the system clock is outside the representable range.
//!
//! Equality is not supported: a timestamp comparison at nanosecond resolution
//! would almost never match. Use a range such as `>=2024-05-01` and
//! `<2024-05-02` instead.
//!
//! # Errors
//!
//! ```
//! use timefilter::{TimeError, parse_duration, parse_time, parse_time_filter};
//!
//! assert_eq!(parse_duration("").unwrap_err(), TimeError::EmptyInput);
//! assert_eq!(parse_duration("7x").unwrap_err(), TimeError::UnknownUnit);
//! assert_eq!(parse_time_filter("1h").unwrap_err(), TimeError::MissingOperator);
//! assert_eq!(parse_time_filter(">=7d").unwrap().to_string(), ">=7d");
//! assert_eq!(parse_time_filter(">=").unwrap_err(), TimeError::MissingValue);
//! assert_eq!(parse_time_filter("=7d").unwrap_err(), TimeError::InvalidOperator);
//! assert_eq!(parse_time("2h").unwrap_err(), TimeError::InvalidDate);
//! ```
//!
//! # Serde support
//!
//! With the `serde` feature enabled, [`TimeFilter`] serializes as its canonical
//! string form and deserializes through [`parse_time_filter`], preserving both
//! relative and absolute filters.
//!
//! ```toml
//! [dependencies]
//! timefilter = { version = "0.3", features = ["serde"] }
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod datetime;
mod duration;
mod error;
mod filter;
mod op;

pub use jiff;
pub use jiff::{SignedDuration, Timestamp};

pub use datetime::{format_datetime, parse_time, parse_time_at, parse_time_now};
pub use duration::{format_duration, parse_duration};
pub use error::{TimeError, TimeResult};
pub use filter::{TimeFilter, parse_time_filter};
pub use op::TimeOp;

/// Easy import of the crate's most common items.
pub mod prelude {
    pub use crate::{
        SignedDuration, TimeError, TimeFilter, TimeOp, TimeResult, Timestamp, format_datetime,
        format_duration, parse_duration, parse_time, parse_time_at, parse_time_filter,
        parse_time_now,
    };
}
