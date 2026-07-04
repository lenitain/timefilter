use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use timefilter::prelude::*;

// -- TimeFilter type --

#[test]
fn from_str() {
    let f: TimeFilter = ">=7d".parse().unwrap();
    assert_eq!(f.op, TimeOp::Ge);
}

#[test]
fn display() {
    let dt = Utc.with_ymd_and_hms(2024, 5, 1, 10, 30, 45).unwrap();
    let f = TimeFilter::gt(dt);
    let s = f.to_string();
    assert!(s.starts_with('>'));
    assert!(s.contains("2024"));
}

#[test]
fn convenience_ctors() {
    let now = Utc::now();
    assert_eq!(TimeFilter::gt(now).op, TimeOp::Gt);
    assert_eq!(TimeFilter::ge(now).op, TimeOp::Ge);
    assert_eq!(TimeFilter::lt(now).op, TimeOp::Lt);
    assert_eq!(TimeFilter::le(now).op, TimeOp::Le);
    assert_eq!(TimeFilter::eq(now).op, TimeOp::Eq);
}

#[test]
fn roundtrip_absolute() {
    let dt: DateTime<Utc> = parse_time("2024-06-15 08:30").unwrap();
    let f = TimeFilter::ge(dt);
    let s = f.to_string();
    assert!(s.starts_with(">="), "starts with >=: {}", s);
    assert!(s.contains("2024-06-15"), "contains date: {}", s);
    assert!(s.contains(":"), "contains time: {}", s);
}

// -- parse_time_filter --

#[test]
fn parse_gt() {
    let f = parse_time_filter(">1h").unwrap();
    assert_eq!(f.op, TimeOp::Gt);
}

#[test]
fn parse_ge() {
    let f = parse_time_filter(">=7d").unwrap();
    assert_eq!(f.op, TimeOp::Ge);
}

#[test]
fn parse_lt() {
    let f = parse_time_filter("<2026-05-01").unwrap();
    assert_eq!(f.op, TimeOp::Lt);
    assert_eq!(f.time.year(), 2026);
    assert_eq!(f.time.month(), 5);
    assert_eq!(f.time.day(), 1);
}

#[test]
fn parse_le() {
    let f = parse_time_filter("<=30m").unwrap();
    assert_eq!(f.op, TimeOp::Le);
}

#[test]
fn parse_eq() {
    let f = parse_time_filter("=2026-05-01 10:00").unwrap();
    assert_eq!(f.op, TimeOp::Eq);
    assert_eq!(f.time.year(), 2026);
    assert_eq!(f.time.month(), 5);
    assert_eq!(f.time.day(), 1);
    assert_eq!(f.time.hour(), 10);
}

#[test]
fn no_operator_errors() {
    assert_eq!(parse_time_filter("1h"), Err(TimeError::MissingOperator));
    assert_eq!(parse_time_filter("30d"), Err(TimeError::MissingOperator));
    assert_eq!(
        parse_time_filter("2026-05-01"),
        Err(TimeError::MissingOperator)
    );
}

#[test]
fn invalid() {
    assert_eq!(parse_time_filter(">abc"), Err(TimeError::InvalidDate));
}

#[test]
fn empty_after_operator() {
    assert_eq!(parse_time_filter(">="), Err(TimeError::EmptyInput));
    assert_eq!(parse_time_filter("<"), Err(TimeError::EmptyInput));
    assert_eq!(parse_time_filter(">"), Err(TimeError::EmptyInput));
    assert_eq!(parse_time_filter("<="), Err(TimeError::EmptyInput));
}
