//! Duration parser and formatter tests.

use jiff::SignedDuration;
use timefilter::prelude::*;

fn dur(s: &str) -> SignedDuration {
    parse_duration(s).unwrap()
}

#[test]
fn human_units() {
    assert_eq!(dur("7d"), SignedDuration::from_hours(7 * 24));
    assert_eq!(dur("1w"), SignedDuration::from_hours(7 * 24));
    assert_eq!(dur("2 weeks"), SignedDuration::from_hours(14 * 24));
    assert_eq!(dur("1 wk"), SignedDuration::from_hours(7 * 24));
    assert_eq!(dur("1 day"), SignedDuration::from_hours(24));
    assert_eq!(dur("3 DAYS"), SignedDuration::from_hours(3 * 24));
    assert_eq!(dur("2h"), SignedDuration::from_hours(2));
    assert_eq!(dur("1hr"), SignedDuration::from_hours(1));
    assert_eq!(dur("24 hours"), SignedDuration::from_hours(24));
    assert_eq!(dur("30m"), SignedDuration::from_mins(30));
    assert_eq!(dur("1M"), SignedDuration::from_mins(1)); // M is minutes; months are `mo`
    assert_eq!(dur("15min"), SignedDuration::from_mins(15));
    assert_eq!(dur("1 minute"), SignedDuration::from_mins(1));
    assert_eq!(dur("2 mins"), SignedDuration::from_mins(2));
    assert_eq!(dur("30s"), SignedDuration::from_secs(30));
    assert_eq!(dur("1sec"), SignedDuration::from_secs(1));
    assert_eq!(dur("45 seconds"), SignedDuration::from_secs(45));
}

#[test]
fn human_compound() {
    assert_eq!(dur("1h30m"), SignedDuration::from_mins(90));
    assert_eq!(
        dur("1d2h3m4s"),
        SignedDuration::new(86_400 + 2 * 3_600 + 3 * 60 + 4, 0)
    );
    assert_eq!(
        dur("7 days 6 hours 5 minutes 4 seconds"),
        SignedDuration::new(7 * 86_400 + 6 * 3_600 + 5 * 60 + 4, 0)
    );
    assert_eq!(dur("1h 30m"), SignedDuration::from_mins(90));
}

#[test]
fn human_fractions() {
    assert_eq!(dur("1.5h"), SignedDuration::from_mins(90));
    assert_eq!(dur(".5h"), SignedDuration::from_mins(30));
    assert_eq!(dur("1.25w"), SignedDuration::from_hours(210));
    assert_eq!(dur("0.5s"), SignedDuration::from_millis(500));
    assert_eq!(dur("1.25d"), SignedDuration::from_hours(30));
    assert_eq!(dur("0.1s"), SignedDuration::from_millis(100));
    assert_eq!(dur("1.000000001s"), SignedDuration::new(1, 1));
    // Sub-nanosecond precision is truncated.
    assert_eq!(dur("0.0000000009s"), SignedDuration::ZERO);
}

#[test]
fn human_sign_and_zero() {
    assert_eq!(dur("-1h"), SignedDuration::from_hours(-1));
    assert_eq!(dur("+2h"), SignedDuration::from_hours(2));
    assert_eq!(dur("-1h30m"), SignedDuration::from_mins(-90));
    assert_eq!(dur("0s"), SignedDuration::ZERO);
    assert_eq!(dur("0.0s"), SignedDuration::ZERO);
}

#[test]
fn human_errors() {
    assert_eq!(parse_duration(""), Err(TimeError::EmptyInput));
    assert_eq!(parse_duration("   "), Err(TimeError::EmptyInput));
    assert_eq!(parse_duration("abc"), Err(TimeError::InvalidNumber));
    assert_eq!(parse_duration("d"), Err(TimeError::InvalidNumber));
    assert_eq!(parse_duration("7"), Err(TimeError::InvalidDuration));
    assert_eq!(parse_duration("1."), Err(TimeError::InvalidNumber));
    assert_eq!(parse_duration("1.5"), Err(TimeError::InvalidDuration));
    assert_eq!(parse_duration("1h30"), Err(TimeError::InvalidDuration));
    assert_eq!(parse_duration("1..5h"), Err(TimeError::InvalidNumber));
    assert_eq!(parse_duration("1x"), Err(TimeError::UnknownUnit));
    assert_eq!(parse_duration("1y"), Err(TimeError::UnsupportedUnit));
    assert_eq!(parse_duration("1year"), Err(TimeError::UnsupportedUnit));
    assert_eq!(parse_duration("1mo"), Err(TimeError::UnsupportedUnit));
    assert!(parse_duration("1h-").is_err());
}

#[test]
fn iso_basic() {
    assert_eq!(dur("P7D"), SignedDuration::from_hours(7 * 24));
    assert_eq!(dur("p7d"), SignedDuration::from_hours(7 * 24));
    assert_eq!(dur("PT2H"), SignedDuration::from_hours(2));
    assert_eq!(dur("PT30M"), SignedDuration::from_mins(30));
    assert_eq!(dur("PT45S"), SignedDuration::from_secs(45));
    assert_eq!(dur("P1DT12H"), SignedDuration::from_hours(36));
    assert_eq!(
        dur("P1DT2H30M"),
        SignedDuration::new(86_400 + 2 * 3_600 + 30 * 60, 0)
    );
    assert_eq!(
        dur("PT1H30M45S"),
        SignedDuration::new(3_600 + 30 * 60 + 45, 0)
    );
    assert_eq!(dur("P1W"), SignedDuration::from_hours(7 * 24));
    assert_eq!(dur("P2W"), SignedDuration::from_hours(14 * 24));
    assert_eq!(parse_duration("P2W3D"), Err(TimeError::InvalidDuration));
}

#[test]
fn iso_zero_and_fractions() {
    assert_eq!(dur("P0D"), SignedDuration::ZERO);
    assert_eq!(dur("PT0S"), SignedDuration::ZERO);
    assert_eq!(dur("P0.0D"), SignedDuration::ZERO);
    assert_eq!(dur("PT0.5S"), SignedDuration::from_millis(500));
    assert_eq!(dur("PT1.5S"), SignedDuration::from_millis(1500));
    assert_eq!(dur("P1.5D"), SignedDuration::from_hours(36));
    assert_eq!(
        dur("P1DT2.5H"),
        SignedDuration::new(86_400 + 2 * 3_600 + 30 * 60, 0)
    );
    assert_eq!(parse_duration("P.5D"), Err(TimeError::InvalidNumber));
}

#[test]
fn iso_signed() {
    assert_eq!(dur("-P1D"), SignedDuration::from_hours(-24));
    assert_eq!(dur("+P1D"), SignedDuration::from_hours(24));
    assert_eq!(dur("-P1DT12H"), SignedDuration::from_hours(-36));
    assert_eq!(parse_duration("-P1Y"), Err(TimeError::UnsupportedUnit));
}

#[test]
fn iso_rejects_calendar_units() {
    assert_eq!(parse_duration("P1Y"), Err(TimeError::UnsupportedUnit));
    assert_eq!(parse_duration("P1M"), Err(TimeError::UnsupportedUnit));
    assert_eq!(parse_duration("P1Y2M3D"), Err(TimeError::UnsupportedUnit));
}

#[test]
fn iso_rejects_bad_structure() {
    assert_eq!(parse_duration("P"), Err(TimeError::InvalidDuration));
    assert_eq!(parse_duration("PT"), Err(TimeError::InvalidDuration));
    assert_eq!(parse_duration("P1DT"), Err(TimeError::InvalidDuration));
    assert_eq!(parse_duration("P1D2H"), Err(TimeError::InvalidDuration));
    assert_eq!(parse_duration("PT1D"), Err(TimeError::InvalidDuration));
    assert_eq!(parse_duration("PT1H1H"), Err(TimeError::InvalidDuration));
    assert_eq!(parse_duration("PT1M1H"), Err(TimeError::InvalidDuration));
    assert_eq!(parse_duration("P1D1D"), Err(TimeError::InvalidDuration));
    assert_eq!(parse_duration("P1DT1H2H"), Err(TimeError::InvalidDuration));
    assert_eq!(parse_duration("P1.5D2H"), Err(TimeError::InvalidDuration));
    assert_eq!(parse_duration("PT1.5S2S"), Err(TimeError::InvalidDuration));
    assert_eq!(parse_duration("P1W2D"), Err(TimeError::InvalidDuration));
    assert_eq!(parse_duration("P1W1D"), Err(TimeError::InvalidDuration));
    assert_eq!(parse_duration("P1WT1H"), Err(TimeError::InvalidDuration));
    assert_eq!(parse_duration("P1X"), Err(TimeError::UnknownUnit));
}

#[test]
fn duration_overflow_is_an_error_not_a_panic() {
    assert_eq!(
        parse_duration("200000000000000000d"),
        Err(TimeError::Overflow)
    );
    assert_eq!(
        parse_duration("P200000000000000000D"),
        Err(TimeError::Overflow)
    );
    assert!(parse_duration("P999999999999999999999999999999999999999D").is_err());
    assert!(parse_duration("999999999999999999999999999999999999999d").is_err());
}

#[test]
fn format_round_trips() {
    let cases = [
        SignedDuration::ZERO,
        SignedDuration::from_secs(1),
        SignedDuration::from_secs(-1),
        SignedDuration::from_mins(90),
        SignedDuration::from_hours(7 * 24),
        SignedDuration::new(86_400 + 2 * 3_600 + 3 * 60 + 4, 0),
        SignedDuration::from_millis(500),
        SignedDuration::from_millis(-1500),
        SignedDuration::new(1, 1),
        SignedDuration::from_hours(365 * 24),
        SignedDuration::MAX,
        SignedDuration::MIN,
    ];
    for span in cases {
        let rendered = format_duration(span);
        assert_eq!(
            parse_duration(&rendered),
            Ok(span),
            "round trip of {rendered:?}"
        );
    }
}

#[test]
fn format_is_minimal() {
    assert_eq!(format_duration(SignedDuration::ZERO), "0s");
    assert_eq!(format_duration(SignedDuration::from_hours(7 * 24)), "7d");
    assert_eq!(format_duration(SignedDuration::from_mins(90)), "1h30m");
    assert_eq!(format_duration(SignedDuration::from_millis(500)), "0.5s");
    assert_eq!(format_duration(SignedDuration::from_millis(-1500)), "-1.5s");
    assert_eq!(format_duration(SignedDuration::new(1, 1)), "1.000000001s");
}

#[test]
fn iso_accepts_what_format_emits() {
    let span = SignedDuration::new(2 * 86_400 + 3 * 3_600, 0);
    assert_eq!(parse_duration("P2DT3H"), Ok(span));
    assert_eq!(format_duration(span), "2d3h");
}

#[test]
fn format_round_trips_over_a_range() {
    for secs in (-1_000i64..=1_000).step_by(7) {
        for nanos in [0, 1, 999_999_999] {
            let span = SignedDuration::new(secs, nanos);
            let rendered = format_duration(span);
            assert_eq!(
                parse_duration(&rendered),
                Ok(span),
                "secs={secs} nanos={nanos} rendered={rendered:?}"
            );
        }
    }
}
