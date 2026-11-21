//! Serde tests. Run with `--features serde`.

#![cfg(feature = "serde")]

use jiff::{SignedDuration, Timestamp};
use timefilter::{TimeFilter, TimeOp};

#[test]
fn relative_filter_round_trips() {
    let filter: TimeFilter = ">=7d".parse().unwrap();
    let json = serde_json::to_string(&filter).unwrap();
    assert_eq!(json, "\">=7d\"");

    let parsed: TimeFilter = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, filter);
    assert_eq!(parsed.op(), TimeOp::Ge);
    assert_eq!(parsed.span(), Some(SignedDuration::from_hours(7 * 24)));
}

#[test]
fn relative_filter_does_not_freeze_on_serialize() {
    let filter: TimeFilter = ">=1h".parse().unwrap();
    let first = serde_json::to_string(&filter).unwrap();
    let second = serde_json::to_string(&filter).unwrap();
    assert_eq!(first, second);
    assert_eq!(first, "\">=1h\"");
}

#[test]
fn absolute_filter_round_trips_including_nanos() {
    let time: Timestamp = "2024-05-01T10:00:00.123456789Z".parse().unwrap();
    let filter = TimeFilter::le(time);

    let json = serde_json::to_string(&filter).unwrap();
    assert_eq!(json, "\"<=2024-05-01T10:00:00.123456789Z\"");

    let parsed: TimeFilter = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, filter);
}

#[test]
fn invalid_json_is_rejected() {
    assert!(serde_json::from_str::<TimeFilter>("\"7d\"").is_err());
    assert!(serde_json::from_str::<TimeFilter>("\">=1x\"").is_err());
    assert!(serde_json::from_str::<TimeFilter>("\"=7d\"").is_err());
    assert!(serde_json::from_str::<TimeFilter>("\">=P1W2D\"").is_err());
}
