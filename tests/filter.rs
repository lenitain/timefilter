//! Time filter tests.

use jiff::{SignedDuration, Timestamp};
use timefilter::prelude::*;

fn ts(s: &str) -> Timestamp {
    s.parse().unwrap()
}

fn now() -> Timestamp {
    ts("2024-06-15T12:00:00Z")
}

#[test]
fn parse_relative() {
    let f: TimeFilter = ">=7d".parse().unwrap();
    assert_eq!(f.op(), TimeOp::Ge);
    assert_eq!(f.span(), Some(SignedDuration::from_hours(7 * 24)));
    assert_eq!(f.time(), None);
    assert!(f.is_relative());
    assert_eq!(f.to_string(), ">=7d");
}

#[test]
fn parse_absolute() {
    let f: TimeFilter = "<2024-05-01T10:00:00Z".parse().unwrap();
    assert_eq!(f.op(), TimeOp::Lt);
    assert_eq!(f.span(), None);
    assert_eq!(f.time(), Some(ts("2024-05-01T10:00:00Z")));
    assert!(!f.is_relative());
    assert_eq!(f.to_string(), "<2024-05-01T10:00:00Z");
}

#[test]
fn parse_accepts_surrounding_whitespace() {
    assert_eq!(parse_time_filter(" >=7d ").unwrap().to_string(), ">=7d");
    assert_eq!(parse_time_filter(">= 7d").unwrap().to_string(), ">=7d");
    assert_eq!(
        parse_time_filter("> 2024-05-01").unwrap().to_string(),
        ">2024-05-01T00:00:00Z"
    );
}

#[test]
fn parse_prefers_relative_for_durations() {
    for expr in [">=7d", ">1h30m", ">P1DT2H", ">0s"] {
        let f: TimeFilter = expr.parse().unwrap();
        assert!(f.is_relative(), "{expr} should be relative");
        // Canonical form parses back to the same filter.
        assert_eq!(f.to_string().parse::<TimeFilter>().unwrap(), f);
    }
}

#[test]
fn relative_matches_at_reference_time() {
    let f: TimeFilter = ">=7d".parse().unwrap();
    let reference = now();

    let three_days_ago = reference
        .checked_sub(SignedDuration::from_hours(3 * 24))
        .unwrap();
    let seven_days_ago = reference
        .checked_sub(SignedDuration::from_hours(7 * 24))
        .unwrap();
    let too_old = seven_days_ago
        .checked_sub(SignedDuration::from_secs(1))
        .unwrap();

    assert!(f.matches_at(three_days_ago, reference));
    assert!(f.matches_at(reference, reference));
    assert!(f.matches_at(seven_days_ago, reference));
    assert!(!f.matches_at(too_old, reference));

    // Reusing the filter later must not widen the window: the span is fixed,
    // the reference moves.
    let later = reference
        .checked_add(SignedDuration::from_hours(1))
        .unwrap();
    assert!(!f.matches_at(too_old, later));
}

#[test]
fn relative_filter_does_not_freeze_at_parse_time() {
    let f: TimeFilter = ">=1s".parse().unwrap();
    let parsed_at = now();
    let event = parsed_at
        .checked_add(SignedDuration::from_millis(400))
        .unwrap();
    let checked_at = parsed_at
        .checked_add(SignedDuration::from_millis(1500))
        .unwrap();

    // The event is 1.1 seconds old when checked, so it is outside `>=1s`.
    assert!(!f.matches_at(event, checked_at));
}

#[test]
fn absolute_matches_fixed_instant() {
    let cutoff = now()
        .checked_sub(SignedDuration::from_hours(7 * 24))
        .unwrap();
    let f = TimeFilter::ge(cutoff);
    assert!(f.matches_at(now(), now()));
    assert!(!f.matches_at(
        cutoff.checked_sub(SignedDuration::from_secs(1)).unwrap(),
        now()
    ));
    assert!(f.matches_at(
        cutoff,
        now().checked_add(SignedDuration::from_hours(2400)).unwrap()
    ));
}

#[test]
fn convenience_constructors() {
    let t = now();
    assert_eq!(TimeFilter::gt(t).op(), TimeOp::Gt);
    assert_eq!(TimeFilter::ge(t).op(), TimeOp::Ge);
    assert_eq!(TimeFilter::lt(t).op(), TimeOp::Lt);
    assert_eq!(TimeFilter::le(t).op(), TimeOp::Le);

    assert_eq!(
        TimeFilter::gt_span(SignedDuration::from_hours(1)).op(),
        TimeOp::Gt
    );
    assert_eq!(
        TimeFilter::ge_span(SignedDuration::from_hours(1)).op(),
        TimeOp::Ge
    );
    assert_eq!(
        TimeFilter::lt_span(SignedDuration::from_hours(1)).op(),
        TimeOp::Lt
    );
    assert_eq!(
        TimeFilter::le_span(SignedDuration::from_hours(1)).op(),
        TimeOp::Le
    );

    assert_eq!(
        TimeFilter::newer_than(SignedDuration::from_hours(1)),
        TimeFilter::ge_span(SignedDuration::from_hours(1))
    );
    assert_eq!(
        TimeFilter::within(SignedDuration::from_hours(1)),
        TimeFilter::newer_than(SignedDuration::from_hours(1))
    );
    assert_eq!(
        TimeFilter::older_than(SignedDuration::from_hours(1)),
        TimeFilter::lt_span(SignedDuration::from_hours(1))
    );
}

#[test]
fn signed_spans_are_documented_and_round_trip() {
    let f = TimeFilter::ge_span(SignedDuration::from_hours(-1));
    let reference = now();
    let in_half_an_hour = reference
        .checked_add(SignedDuration::from_mins(30))
        .unwrap();
    let in_two_hours = reference
        .checked_add(SignedDuration::from_hours(2))
        .unwrap();

    assert!(!f.matches_at(in_half_an_hour, reference));
    assert!(f.matches_at(in_two_hours, reference));
    assert_eq!(f.to_string(), ">=-1h");
    assert_eq!(f.to_string().parse::<TimeFilter>().unwrap(), f);

    let iso: TimeFilter = ">=-P1D".parse().unwrap();
    assert_eq!(iso.to_string(), ">=-1d");
    assert_eq!(iso.span(), Some(SignedDuration::from_hours(-24)));
}

#[test]
fn relative_filters_are_const_constructible() {
    const MINUTE: TimeFilter = TimeFilter::ge_span(SignedDuration::from_mins(1));
    assert!(MINUTE.matches_at(now(), now()));
}

#[test]
fn display_round_trips_exactly() {
    let cases = [
        TimeFilter::relative(TimeOp::Ge, SignedDuration::from_hours(7 * 24)),
        TimeFilter::relative(TimeOp::Lt, SignedDuration::from_mins(90)),
        TimeFilter::relative(TimeOp::Gt, SignedDuration::from_millis(500)),
        TimeFilter::relative(TimeOp::Gt, SignedDuration::ZERO),
        TimeFilter::relative(TimeOp::Ge, SignedDuration::MAX),
        TimeFilter::relative(TimeOp::Le, SignedDuration::MIN),
        TimeFilter::new(TimeOp::Ge, ts("2024-05-01T10:00:00.123456789Z")),
        TimeFilter::new(TimeOp::Le, ts("1969-12-31T23:59:59Z")),
    ];
    for filter in cases {
        let rendered = filter.to_string();
        assert_eq!(
            rendered.parse::<TimeFilter>().unwrap(),
            filter,
            "round trip of {rendered:?}"
        );
    }
}

#[test]
fn resolve_returns_threshold() {
    let reference = now();
    let relative: TimeFilter = ">=7d".parse().unwrap();
    assert_eq!(
        relative.resolve(reference),
        Some(
            reference
                .checked_sub(SignedDuration::from_hours(7 * 24))
                .unwrap()
        )
    );

    let absolute = TimeFilter::le(reference);
    assert_eq!(
        absolute.resolve(now().checked_add(SignedDuration::from_hours(24)).unwrap()),
        Some(reference)
    );
}

#[test]
fn extreme_spans_compare_mathematically() {
    let reference = now();

    // Huge positive span: threshold is below every representable instant.
    let below = TimeFilter::ge_span(SignedDuration::MAX);
    assert_eq!(below.resolve(reference), None);
    assert!(below.matches_at(Timestamp::MIN, reference));
    assert!(TimeFilter::gt_span(SignedDuration::MAX).matches_at(Timestamp::MIN, reference));
    assert!(!TimeFilter::lt_span(SignedDuration::MAX).matches_at(Timestamp::MIN, reference));
    assert!(!TimeFilter::le_span(SignedDuration::MAX).matches_at(Timestamp::MIN, reference));

    // Huge negative span: threshold is above every representable instant.
    let above = TimeFilter::lt_span(SignedDuration::MIN);
    assert_eq!(above.resolve(reference), None);
    assert!(above.matches_at(Timestamp::MAX, reference));
    assert!(TimeFilter::le_span(SignedDuration::MIN).matches_at(Timestamp::MAX, reference));
    assert!(!TimeFilter::ge_span(SignedDuration::MIN).matches_at(Timestamp::MAX, reference));
}

#[test]
fn operator_parsing() {
    for (text, op) in [
        (">", TimeOp::Gt),
        (">=", TimeOp::Ge),
        ("<", TimeOp::Lt),
        ("<=", TimeOp::Le),
    ] {
        assert_eq!(text.parse::<TimeOp>().unwrap(), op);
        assert_eq!(op.to_string(), text);
    }
    assert_eq!("".parse::<TimeOp>(), Err(TimeError::EmptyInput));
    assert_eq!(" >= ".parse::<TimeOp>(), Ok(TimeOp::Ge));
    assert_eq!(">=1".parse::<TimeOp>(), Err(TimeError::InvalidOperator));
    assert_eq!("=".parse::<TimeOp>(), Err(TimeError::InvalidOperator));
    assert_eq!("!=".parse::<TimeOp>(), Err(TimeError::InvalidOperator));
}

#[test]
fn operator_applies() {
    let t1 = now();
    let t2 = t1.checked_sub(SignedDuration::from_hours(24)).unwrap();
    assert!(TimeOp::Gt.applies(t1, t2));
    assert!(TimeOp::Ge.applies(t1, t1));
    assert!(!TimeOp::Lt.applies(t1, t2));
    assert!(TimeOp::Le.applies(t2, t1));
    assert_eq!(TimeOp::ALL.len(), 4);
}

#[test]
fn filter_errors() {
    assert_eq!(parse_time_filter(""), Err(TimeError::EmptyInput));
    assert_eq!(parse_time_filter("   "), Err(TimeError::EmptyInput));
    assert_eq!(parse_time_filter(">="), Err(TimeError::MissingValue));
    assert_eq!(parse_time_filter(">"), Err(TimeError::MissingValue));
    assert_eq!(parse_time_filter("1h"), Err(TimeError::MissingOperator));
    assert_eq!(parse_time_filter("!7d"), Err(TimeError::InvalidOperator));
    assert_eq!(parse_time_filter("=7d"), Err(TimeError::InvalidOperator));
    assert_eq!(parse_time_filter("==7d"), Err(TimeError::InvalidOperator));
    assert_eq!(parse_time_filter("> = 7d"), Err(TimeError::InvalidOperator));
    assert_eq!(parse_time_filter(">=1x"), Err(TimeError::UnknownUnit));
    assert_eq!(parse_time_filter(">=abc"), Err(TimeError::InvalidDate));
    assert_eq!(parse_time_filter(">=P1Y"), Err(TimeError::UnsupportedUnit));
    assert_eq!(
        parse_time_filter("<2024-13-01"),
        Err(TimeError::InvalidDate)
    );
}

#[test]
fn try_matches_uses_the_current_clock() {
    let f = TimeFilter::ge(Timestamp::MIN);
    assert!(f.try_matches(now()).unwrap());
}

#[test]
fn canonical_strings_are_stable() {
    assert_eq!(
        TimeFilter::ge_span(SignedDuration::from_hours(7 * 24)).to_string(),
        ">=7d"
    );
    assert_eq!(
        TimeFilter::lt_span(SignedDuration::from_mins(90)).to_string(),
        "<1h30m"
    );
    assert_eq!(
        TimeFilter::new(TimeOp::Ge, ts("2024-05-01T10:00:00.123456789Z")).to_string(),
        ">=2024-05-01T10:00:00.123456789Z"
    );
}
