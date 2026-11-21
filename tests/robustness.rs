//! Regression tests for robustness: garbage input must not panic, and the
//! canonical string forms must round trip.

use jiff::{SignedDuration, Timestamp};
use timefilter::prelude::*;

fn next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state
}

#[test]
fn parsing_garbage_never_panics() {
    let alphabet: Vec<char> =
        "0123456789.DHMSWTKMGTPEibkKBMGTPE/ +-:=<>,;[]{}()abcxyz\u{00b5}\u{041a}\u{ff11}"
            .chars()
            .collect();
    let mut state = 0x5eed_1234_9876_abcd;
    let now = Timestamp::from_second(1_700_000_000).unwrap();
    let mut buffer = String::new();
    for _ in 0..20_000 {
        buffer.clear();
        let len = (next(&mut state) % 48) as usize;
        for _ in 0..len {
            buffer.push(alphabet[(next(&mut state) as usize) % alphabet.len()]);
        }
        let _ = parse_duration(&buffer);
        let _ = parse_time(&buffer);
        let _ = parse_time_at(&buffer, now);
        let _ = parse_time_filter(&buffer);
    }
}

#[test]
fn durations_round_trip() {
    let mut state = 0x1234_5678_9abc_def0;
    for _ in 0..20_000 {
        let secs = (next(&mut state) as i64) % 9_000_000_000_000;
        let nanos = (next(&mut state) % 1_000_000_000) as i32;
        let span = SignedDuration::new(secs, nanos);
        let rendered = format_duration(span);
        assert_eq!(parse_duration(&rendered), Ok(span), "{rendered:?}");
    }
}

#[test]
fn timestamps_and_filters_round_trip() {
    let mut state = 0x0bad_f00d_1234_5678;
    for _ in 0..20_000 {
        let secs = (next(&mut state) as i64) % 200_000_000_000;
        let nanos = (next(&mut state) % 1_000_000_000) as i32;
        let Ok(base) = Timestamp::from_second(secs) else {
            continue;
        };
        let Ok(timestamp) = base.checked_add(SignedDuration::new(0, nanos)) else {
            continue;
        };

        let rendered = format_datetime(&timestamp);
        assert_eq!(parse_time(&rendered).unwrap(), timestamp);

        let filter = TimeFilter::ge(timestamp);
        assert_eq!(filter.to_string().parse::<TimeFilter>().unwrap(), filter);
    }
}
