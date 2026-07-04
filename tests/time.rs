use chrono::{DateTime, TimeZone, Utc};
use timefilter::prelude::*;

// -- TimeOp --

#[test]
fn all_variants() {
    assert_eq!(TimeOp::ALL.len(), 5);
}

#[test]
fn applies() {
    let t1: DateTime<Utc> = Utc.with_ymd_and_hms(2024, 6, 1, 0, 0, 0).unwrap();
    let t2: DateTime<Utc> = Utc.with_ymd_and_hms(2024, 5, 1, 0, 0, 0).unwrap();
    assert!(TimeOp::Gt.applies(t1, t2));
    assert!(!TimeOp::Gt.applies(t2, t1));
    assert!(TimeOp::Ge.applies(t1, t1));
    assert!(TimeOp::Eq.applies(t1, t1));
}

#[test]
fn display() {
    assert_eq!(TimeOp::Gt.to_string(), ">");
    assert_eq!(TimeOp::Ge.to_string(), ">=");
    assert_eq!(TimeOp::Lt.to_string(), "<");
    assert_eq!(TimeOp::Le.to_string(), "<=");
    assert_eq!(TimeOp::Eq.to_string(), "=");
}

// -- error types --

#[test]
fn error_display() {
    assert_eq!(TimeError::EmptyInput.to_string(), "empty input");
    assert_eq!(
        TimeError::InvalidDate.to_string(),
        "failed to parse date/time"
    );
    assert_eq!(TimeError::UnknownSuffix.to_string(), "unknown time suffix");
    assert_eq!(
        TimeError::InvalidNumber.to_string(),
        "failed to parse number"
    );
    assert!(
        TimeError::MissingOperator
            .to_string()
            .contains("time filter must start")
    );
}

#[test]
fn error_is_proper_error() {
    use std::error::Error;
    assert!(TimeError::EmptyInput.source().is_none());
}

// -- send/sync --

#[test]
fn traits() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<TimeFilter>();
    assert_sync::<TimeFilter>();
    assert_send::<TimeOp>();
    assert_sync::<TimeOp>();
}
