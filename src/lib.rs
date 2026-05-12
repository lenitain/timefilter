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
    pub use super::{TimeFilter, TimeOp, TimeError, TimeResult};
    pub use super::{parse_time, parse_time_filter, format_datetime};
}
