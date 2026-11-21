//! Absolute time parsing and formatting tests.

use jiff::{SignedDuration, Timestamp};
use timefilter::prelude::*;

fn ts(s: &str) -> Timestamp {
    s.parse().unwrap()
}

#[test]
fn absolute_formats() {
    let expected = ts("2024-05-01T10:00:00Z");
    assert_eq!(parse_time("2024-05-01T10:00").unwrap(), expected);
    assert_eq!(parse_time("2024-05-01 10:00").unwrap(), expected);
    assert_eq!(parse_time("2024-05-01T10:00:00").unwrap(), expected);
    assert_eq!(parse_time("2024-05-01 10:00:00").unwrap(), expected);
    assert_eq!(parse_time("2024-05-01T10:00:00Z").unwrap(), expected);
    assert_eq!(parse_time("2024-05-01 10:00:00 ").unwrap(), expected);
}

#[test]
fn absolute_with_offset() {
    let expected = ts("2024-05-01T10:00:00Z");
    assert_eq!(parse_time("2024-05-01T18:00:00+08:00").unwrap(), expected);
    assert_eq!(parse_time("2024-05-01T05:00:00-05:00").unwrap(), expected);
}

#[test]
fn absolute_fractional_seconds() {
    let parsed = parse_time("2024-05-01T10:00:00.250").unwrap();
    assert_eq!(parsed, ts("2024-05-01T10:00:00.25Z"));

    let parsed = parse_time("2024-05-01T10:00:00.123456789Z").unwrap();
    assert_eq!(parsed, ts("2024-05-01T10:00:00.123456789Z"));
}

#[test]
fn date_only_is_midnight_utc() {
    assert_eq!(
        parse_time("2024-01-15").unwrap(),
        ts("2024-01-15T00:00:00Z")
    );
}

#[test]
fn leap_year() {
    assert_eq!(
        parse_time("2024-02-29").unwrap(),
        ts("2024-02-29T00:00:00Z")
    );
    assert_eq!(parse_time("2023-02-29"), Err(TimeError::InvalidDate));
}

#[test]
fn invalid_absolute() {
    assert_eq!(parse_time("2024-01-32"), Err(TimeError::InvalidDate));
    assert_eq!(parse_time("2024-13-01"), Err(TimeError::InvalidDate));
    assert_eq!(parse_time("2024-04-31"), Err(TimeError::InvalidDate));
    assert_eq!(parse_time("2024-05-01 24:00"), Err(TimeError::InvalidDate));
    assert_eq!(
        parse_time("2024-05-01T10:00:00+26:00"),
        Err(TimeError::InvalidDate)
    );
    // A civil datetime with a Zulu offset is not a naive value; jiff requires
    // parsing it as a timestamp, which our timestamp branch already does.
    assert_eq!(
        parse_time("2024-05-01T10:00:00Z").unwrap(),
        ts("2024-05-01T10:00:00Z")
    );
}

#[test]
fn invalid_input_errors() {
    assert_eq!(parse_time(""), Err(TimeError::EmptyInput));
    assert_eq!(parse_time("  "), Err(TimeError::EmptyInput));
    assert_eq!(parse_time("abc"), Err(TimeError::InvalidDate));
    assert_eq!(parse_time("1x"), Err(TimeError::InvalidDate));
    assert_eq!(parse_time("P1Y"), Err(TimeError::InvalidDate));
}

#[test]
fn parse_time_rejects_relative_input() {
    assert_eq!(parse_time("2h"), Err(TimeError::InvalidDate));
    assert_eq!(parse_time("P1D"), Err(TimeError::InvalidDate));
    assert_eq!(parse_time("-2h"), Err(TimeError::InvalidDate));
}

#[test]
fn parse_time_now_resolves_relative_input() {
    let before = Timestamp::now();
    let parsed = parse_time_now("2h").unwrap();
    let after = Timestamp::now();

    assert!(parsed >= before.checked_sub(SignedDuration::from_hours(2)).unwrap());
    assert!(parsed <= after.checked_sub(SignedDuration::from_hours(2)).unwrap());

    // Absolute input still ignores the clock.
    assert_eq!(
        parse_time_now("2024-05-01").unwrap(),
        ts("2024-05-01T00:00:00Z")
    );
}

#[test]
fn relative_is_relative_to_explicit_now() {
    let now = ts("2024-06-15T12:00:00Z");
    assert_eq!(
        parse_time_at("2h", now).unwrap(),
        now.checked_sub(SignedDuration::from_hours(2)).unwrap()
    );
    assert_eq!(
        parse_time_at("1d12h", now).unwrap(),
        now.checked_sub(SignedDuration::from_hours(36)).unwrap()
    );
    assert_eq!(
        parse_time_at("-2h", now).unwrap(),
        now.checked_add(SignedDuration::from_hours(2)).unwrap()
    );
    assert_eq!(
        parse_time_at("P1DT2H", now).unwrap(),
        now.checked_sub(SignedDuration::from_hours(26)).unwrap()
    );
    // Absolute input ignores the reference time.
    assert_eq!(
        parse_time_at("2024-05-01", now).unwrap(),
        ts("2024-05-01T00:00:00Z")
    );
}

#[test]
fn relative_overflow_is_an_error() {
    let now = ts("2024-06-15T12:00:00Z");
    assert_eq!(
        parse_time_at("200000000000000000d", now),
        Err(TimeError::Overflow)
    );
    assert_eq!(
        parse_time_at("P200000000000000000D", now),
        Err(TimeError::Overflow)
    );
}

#[test]
fn format_is_rfc3339_utc() {
    let t = ts("2024-05-01T10:30:45Z");
    assert_eq!(format_datetime(&t), "2024-05-01T10:30:45Z");

    let t = ts("2024-05-01T10:30:45.123456789Z");
    assert_eq!(format_datetime(&t), "2024-05-01T10:30:45.123456789Z");
}

#[test]
fn format_round_trips() {
    let cases = [
        ts("2024-05-01T10:30:45Z"),
        ts("1969-12-31T23:59:59Z"),
        Timestamp::now(),
    ];
    for t in cases {
        assert_eq!(parse_time(&format_datetime(&t)).unwrap(), t);
    }
}
