use chrono::{Datelike, Duration, Timelike, Utc};
use timefilter::prelude::*;

// -- relative time --

#[test]
fn hours() {
    let now = Utc::now();
    let parsed = parse_time("1h").unwrap();
    let diff = now - parsed;
    assert!(diff >= Duration::minutes(59) && diff <= Duration::minutes(61));
}

#[test]
fn hours_hr_suffix() {
    let now = Utc::now();
    let parsed = parse_time("2hr").unwrap();
    let diff = now - parsed;
    assert!(diff >= Duration::minutes(119) && diff <= Duration::minutes(121));
}

#[test]
fn minutes() {
    let now = Utc::now();
    let parsed = parse_time("30m").unwrap();
    let diff = now - parsed;
    assert!(diff >= Duration::minutes(29) && diff <= Duration::minutes(31));
}

#[test]
fn minutes_min_suffix() {
    let now = Utc::now();
    let parsed = parse_time("15min").unwrap();
    let diff = now - parsed;
    assert!(diff >= Duration::minutes(14) && diff <= Duration::minutes(16));
}

#[test]
fn days() {
    let now = Utc::now();
    let parsed = parse_time("7d").unwrap();
    let diff = now - parsed;
    assert!(diff >= Duration::hours(167) && diff <= Duration::hours(169));
}

#[test]
fn seconds() {
    let now = Utc::now();
    let parsed = parse_time("30s").unwrap();
    let diff = now - parsed;
    assert!(diff >= Duration::seconds(29) && diff <= Duration::seconds(31));
}

#[test]
fn verbose_suffixes() {
    assert!(parse_time("1 day").is_ok());
    assert!(parse_time("7 days").is_ok());
    assert!(parse_time("2 hour").is_ok());
    assert!(parse_time("3 hours").is_ok());
    assert!(parse_time("10 minute").is_ok());
    assert!(parse_time("5 minutes").is_ok());
    assert!(parse_time("30 second").is_ok());
    assert!(parse_time("45 seconds").is_ok());
}

#[test]
fn negative_relative_time() {
    // "-1h" means 1 hour in the FUTURE (now - (-1h) = now + 1h)
    let now = Utc::now();
    let parsed = parse_time("-1h").unwrap();
    let diff = parsed - now;
    assert!(diff.num_minutes() >= 59 && diff.num_minutes() <= 61);
}

// -- absolute time --

#[test]
fn datetime() {
    let parsed = parse_time("2024-05-01 10:00").unwrap();
    assert_eq!(parsed.year(), 2024);
    assert_eq!(parsed.month(), 5);
    assert_eq!(parsed.day(), 1);
    assert_eq!(parsed.hour(), 10);
    assert_eq!(parsed.minute(), 0);
}

#[test]
fn with_seconds() {
    let parsed = parse_time("2024-12-25 15:30:45").unwrap();
    assert_eq!(parsed.year(), 2024);
    assert_eq!(parsed.month(), 12);
    assert_eq!(parsed.day(), 25);
    assert_eq!(parsed.hour(), 15);
    assert_eq!(parsed.minute(), 30);
    assert_eq!(parsed.second(), 45);
}

#[test]
fn date_only() {
    let parsed = parse_time("2024-01-15").unwrap();
    assert_eq!(parsed.year(), 2024);
    assert_eq!(parsed.month(), 1);
    assert_eq!(parsed.day(), 15);
    assert_eq!(parsed.hour(), 0);
    assert_eq!(parsed.minute(), 0);
}

#[test]
fn leap_year() {
    let dt = parse_time("2024-02-29").unwrap();
    assert_eq!(dt.month(), 2);
    assert_eq!(dt.day(), 29);
}

#[test]
fn invalid_leap_year() {
    assert_eq!(parse_time("2023-02-29"), Err(TimeError::InvalidDate));
}

#[test]
fn edge_day_month_boundaries() {
    assert_eq!(parse_time("2024-01-32"), Err(TimeError::InvalidDate));
    assert_eq!(parse_time("2024-13-01"), Err(TimeError::InvalidDate));
    assert_eq!(parse_time("2024-04-31"), Err(TimeError::InvalidDate));
}

// -- errors --

#[test]
fn invalid() {
    assert_eq!(parse_time("invalid"), Err(TimeError::InvalidDate));
    assert_eq!(parse_time("2024-13-01 10:00"), Err(TimeError::InvalidDate));
    assert_eq!(parse_time("abc"), Err(TimeError::InvalidDate));
    assert_eq!(parse_time(""), Err(TimeError::EmptyInput));
    assert_eq!(parse_time("  "), Err(TimeError::EmptyInput));
}

#[test]
fn unknown_suffix() {
    assert_eq!(parse_time("1x"), Err(TimeError::InvalidDate));
}
