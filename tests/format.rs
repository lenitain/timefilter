use chrono::{DateTime, TimeZone, Utc};
use timefilter::prelude::*;

#[test]
fn contains_year() {
    let dt = Utc.with_ymd_and_hms(2024, 5, 1, 10, 30, 45).unwrap();
    let s = format_datetime(&dt);
    assert!(!s.is_empty());
    assert!(s.contains("2024"));
}

#[test]
fn various_times() {
    let dt: DateTime<Utc> = parse_time("2024-01-01").unwrap();
    let s = format_datetime(&dt);
    assert!(s.contains("2024"), "contains year: {}", s);
    assert!(s.contains("01"), "contains month/day: {}", s);
    assert!(s.contains(":"), "contains time separator: {}", s);
}
