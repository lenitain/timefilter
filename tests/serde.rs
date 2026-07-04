//! Integration tests for serde feature.

#![cfg(feature = "serde")]

use chrono::{DateTime, NaiveDateTime, Utc};
use timefilter::{TimeFilter, TimeOp};

fn make_time(s: &str) -> DateTime<Utc> {
    let naive = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").unwrap();
    DateTime::from_naive_utc_and_offset(naive, Utc)
}

#[test]
fn serde_roundtrip_time_filter() {
    let f = TimeFilter::ge(make_time("2024-05-01 10:00:00"));
    let json = serde_json::to_string(&f).unwrap();
    assert!(json.contains(">="));

    let parsed: TimeFilter = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.op(), TimeOp::Ge);
    // Compare as strings to avoid timezone issues
    assert_eq!(
        parsed.time().format("%Y-%m-%d %H:%M:%S").to_string(),
        "2024-05-01 10:00:00"
    );
}

#[test]
fn serde_relative_time_filter() {
    let f = TimeFilter::lt(make_time("2024-01-01 00:00:00"));
    let json = serde_json::to_string(&f).unwrap();

    let parsed: TimeFilter = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.op(), TimeOp::Lt);
}
