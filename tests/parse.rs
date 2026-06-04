use chrono::{Datelike, Duration, Timelike, Utc};
use timefilter::{parse_duration, prelude::*};

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

// -- parse_duration --

#[test]
fn parse_duration_days() {
    assert_eq!(parse_duration("7d").unwrap(), Duration::days(7));
    assert_eq!(parse_duration("1 day").unwrap(), Duration::days(1));
    assert_eq!(parse_duration("3 days").unwrap(), Duration::days(3));
}

#[test]
fn parse_duration_hours() {
    assert_eq!(parse_duration("2h").unwrap(), Duration::hours(2));
    assert_eq!(parse_duration("1hr").unwrap(), Duration::hours(1));
    assert_eq!(parse_duration("24 hours").unwrap(), Duration::hours(24));
}

#[test]
fn parse_duration_minutes() {
    assert_eq!(parse_duration("30m").unwrap(), Duration::minutes(30));
    assert_eq!(parse_duration("15min").unwrap(), Duration::minutes(15));
    assert_eq!(parse_duration("1 minute").unwrap(), Duration::minutes(1));
}

#[test]
fn parse_duration_seconds() {
    assert_eq!(parse_duration("30s").unwrap(), Duration::seconds(30));
    assert_eq!(parse_duration("1sec").unwrap(), Duration::seconds(1));
    assert_eq!(parse_duration("45 seconds").unwrap(), Duration::seconds(45));
}

#[test]
fn parse_duration_iso8601() {
    assert_eq!(parse_duration("P7D").unwrap(), Duration::days(7));
    assert_eq!(parse_duration("p7d").unwrap(), Duration::days(7));
    assert_eq!(parse_duration("PT2H").unwrap(), Duration::hours(2));
    assert_eq!(parse_duration("PT30M").unwrap(), Duration::minutes(30));
    assert_eq!(parse_duration("PT45S").unwrap(), Duration::seconds(45));
    assert_eq!(
        parse_duration("P1DT12H").unwrap(),
        Duration::days(1) + Duration::hours(12)
    );
    assert_eq!(
        parse_duration("P1DT2H30M").unwrap(),
        Duration::days(1) + Duration::hours(2) + Duration::minutes(30)
    );
    assert_eq!(
        parse_duration("PT1H30M45S").unwrap(),
        Duration::hours(1) + Duration::minutes(30) + Duration::seconds(45)
    );
}

#[test]
fn parse_duration_errors() {
    assert_eq!(parse_duration("").unwrap_err(), TimeError::EmptyInput);
    assert_eq!(parse_duration("  ").unwrap_err(), TimeError::EmptyInput);
    assert_eq!(parse_duration("abc").unwrap_err(), TimeError::UnknownSuffix);
    assert_eq!(parse_duration("1x").unwrap_err(), TimeError::UnknownSuffix);
    assert_eq!(parse_duration("P").unwrap_err(), TimeError::InvalidNumber);
    assert_eq!(parse_duration("PT").unwrap_err(), TimeError::InvalidNumber);
}
