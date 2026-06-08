//! `timefilter` — human-readable time string parsing, filtering, and formatting.
//!
//! Handles strings like `">=7d"`, `"<2h"`, `"2024-05-01"` — parsing them into
//! `DateTime<Utc>` values and applying comparison filters.
//!
//! Unlike existing crates (`duration-str`, `human-time`), this crate focuses on
//! **relative time → timestamp** resolution and **filter operators** — the
//! `TimeOp`/`TimeFilter` types that let you write `>=7d` or `<2h` as a single
//! filter expression.
//!
//! # Quick Start
//!
//! ```
//! use timefilter::prelude::*;
//!
//! // Parse absolute time
//! let dt = parse_time("2024-05-01 10:00").unwrap();
//!
//! // Parse a filter expression against absolute time
//! let f: TimeFilter = ">=2024-05-01 09:00".parse().unwrap();
//! assert_eq!(f.op, TimeOp::Ge);
//! assert!(f.matches(dt));
//!
//! // Relative time (hours ago) — just check it parses
//! let dt = parse_time("2h").unwrap();
//! let age = chrono::Utc::now() - dt;
//! assert!(age.num_hours() >= 1 && age.num_hours() <= 3);
//! ```
//!
//! # Relative time
//!
//! `parse_time` parses relative durations to `DateTime<Utc>` (now - duration):
//!
//! ```
//! use timefilter::parse_time;
//!
//! let dt = parse_time("7d").unwrap();    // 7 days ago
//! let dt = parse_time("2h").unwrap();    // 2 hours ago
//! let dt = parse_time("30m").unwrap();   // 30 minutes ago
//! let dt = parse_time("30s").unwrap();   // 30 seconds ago
//! ```
//!
//! # Parse duration only
//!
//! `parse_duration` parses time strings to `Duration` without subtracting from current time:
//!
//! ```
//! use timefilter::parse_duration;
//! use chrono::Duration;
//!
//! let dur = parse_duration("7d").unwrap();    // Duration::days(7)
//! let dur = parse_duration("2h").unwrap();    // Duration::hours(2)
//! let dur = parse_duration("P7D").unwrap();   // ISO 8601: 7 days
//! let dur = parse_duration("PT2H").unwrap();  // ISO 8601: 2 hours
//! ```
//!
//! # Absolute time
//!
//! ```
//! use timefilter::parse_time;
//!
//! parse_time("2024-05-01").unwrap();          // midnight UTC
//! parse_time("2024-05-01 10:00").unwrap();    // hour:minute
//! parse_time("2024-05-01 10:00:00").unwrap(); // with seconds
//! ```
//!
//! # Time filters
//!
//! ```
//! use timefilter::TimeFilter;
//!
//! let f: TimeFilter = ">=7d".parse().unwrap();
//! assert!(f.to_string().starts_with('>'));
//!
//! // Convenience constructors
//! use chrono::{Utc, DateTime};
//! let now: DateTime<Utc> = Utc::now();
//! let f = TimeFilter::lt(now);
//! ```
//!
//! # TimeFilter convenience constructors
//!
//! ```
//! use timefilter::TimeFilter;
//! use chrono::Utc;
//!
//! let now = Utc::now();
//! let f = TimeFilter::gt(now);   // value > now
//! let f = TimeFilter::ge(now);   // value >= now
//! let f = TimeFilter::lt(now);   // value < now
//! let f = TimeFilter::le(now);   // value <= now
//! let f = TimeFilter::eq(now);   // value == now
//! ```
//!
//! # Using `TimeFilter::matches`
//!
//! ```
//! use timefilter::{parse_time, TimeFilter};
//!
//! let start = parse_time("2024-06-01").unwrap();
//! let end   = parse_time("2024-06-30").unwrap();
//! let mid   = parse_time("2024-06-15").unwrap();
//!
//! let f = TimeFilter::ge(start);
//! assert!(f.matches(mid));
//! assert!(f.matches(end));
//! assert!(!f.matches(parse_time("2024-05-31").unwrap()));
//! ```
//!
//! # Using `TimeOp::applies`
//!
//! ```
//! use timefilter::{TimeOp, parse_time};
//!
//! let t1 = parse_time("2024-06-15").unwrap();
//! let t2 = parse_time("2024-05-01").unwrap();
//!
//! assert!(TimeOp::Gt.applies(t1, t2));
//! assert!(!TimeOp::Lt.applies(t1, t2));
//! assert!(TimeOp::Eq.applies(t1, t1));
//! ```
//!
//! # Error handling
//!
//! ```
//! use timefilter::{parse_time, parse_time_filter, TimeError};
//!
//! assert_eq!(parse_time(""), Err(TimeError::EmptyInput));
//! assert_eq!(parse_time("abc"), Err(TimeError::InvalidDate));
//! assert_eq!(parse_time_filter("1h"), Err(TimeError::MissingOperator));
//! ```
//!
//! # Serde support
//!
//! With the `serde` feature enabled, [`TimeFilter`] can be serialized and
//! deserialized as a human-readable string:
//!
//! ```toml
//! [dependencies]
//! timefilter = { version = "0.1", features = ["serde"] }
//! ```

#![forbid(unsafe_code)]

mod time;

pub use time::*;

/// Easy import of the crate's most common items.
pub mod prelude {
    pub use crate::{TimeError, TimeFilter, TimeOp, TimeResult};
    pub use crate::{format_datetime, parse_duration, parse_time, parse_time_filter};
}
