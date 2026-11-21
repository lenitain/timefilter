# timefilter

Human-readable time parsing, filtering, and formatting with comparison operators.

[![Crates.io](https://img.shields.io/crates/v/timefilter.svg)](https://crates.io/crates/timefilter)
[![Docs.rs](https://docs.rs/timefilter/badge.svg)](https://docs.rs/timefilter)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/lenitain/timefilter/actions/workflows/ci.yml/badge.svg)](https://github.com/lenitain/timefilter/actions/workflows/ci.yml)

## Overview

`timefilter` turns expressions such as `>=7d`, `<2h`, and `2024-05-01` into
typed filters and timestamps. Relative filters keep their span and resolve it
against a reference time when checked, so a filter parsed from a config file
does not go stale while the process runs.

- Absolute times use RFC 3339 or naive `YYYY-MM-DD [HH:MM[:SS]]` forms; naive
  input is UTC. Offsets are honored.
- Durations accept human-readable (`1h30m`, `2 days`, `1.5h`, `1w`) and ISO 8601
  (`P1DT12H`, `PT0.5S`, `P1W`, `-P1D`) forms. Years and months are rejected
  because they have no fixed length. ISO weeks are standalone (`P1W`, not
  `P1W2D`).
- Parsing is pure: `parse_time` accepts absolute timestamps only.
  `parse_time_now` is the explicit clock-reading convenience, and
  `parse_time_at` takes the reference time from the caller.
- Relative filters resolve at check time. `matches_at` is deterministic;
  `matches` uses the current time and `try_matches` is its fallible form.
- Display output is canonical and parses back to the same value.
- Optional `serde` support uses the canonical string form.

Timestamps and durations are [jiff](https://docs.rs/jiff) types, re-exported as
`timefilter::Timestamp` and `timefilter::SignedDuration`. The whole jiff crate
is re-exported as `timefilter::jiff`, so callers stay on the same version.
jiff 1.0 is used because chrono is unmaintained
([RUSTSEC-2026-0543](https://rustsec.org/advisories/RUSTSEC-2026-0543)).
The examples import `jiff` directly; `timefilter::{Timestamp, SignedDuration}`
and `timefilter::jiff` expose the same types without a separate dependency.

## Scope

The crate parses human durations, absolute timestamps, and **one ordering
filter**. It is UTC-only, treats days and weeks as fixed 24-hour and 7-day
durations, and rejects calendar units.

It is not a general datetime library:

- no time zones, DST, or calendar arithmetic;
- no months/years as durations, and no natural language such as "yesterday";
- no recurrence, rounding, or "start of day" operations;
- no equality filters; use a range instead;
- no configuration loading, environment access, or CLI parsing.

## Filter semantics

The operator compares the **timestamp**, not the age. `>=7d` means "timestamp
at least as new as seven days ago", i.e. no older than seven days:

| Expression | Meaning |
| --- | --- |
| `>=7d` | timestamp >= now - 7d (no older than 7 days) |
| `<7d` | timestamp < now - 7d (older than 7 days) |
| `>=2024-05-01` | timestamp >= 2024-05-01T00:00:00Z |

Intent-named constructors make the direction explicit:

```rust
use jiff::SignedDuration;
use timefilter::TimeFilter;

let f = TimeFilter::newer_than(SignedDuration::from_hours(2));
assert_eq!(f.to_string(), ">=2h");

let f = TimeFilter::older_than(SignedDuration::from_hours(2));
assert_eq!(f.to_string(), "<2h");

let f = TimeFilter::within(SignedDuration::from_hours(2));
assert_eq!(f.to_string(), ">=2h");
```

Equality is not supported: a timestamp comparison at nanosecond resolution
would almost never match. Use a range such as `>=2024-05-01` and `<2024-05-02`
instead. A negative relative span places the threshold in the future; the
comparison is still against the resolved timestamp.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
timefilter = "0.3"
```

Requires Rust 1.85 or newer.

### Filters

```rust
use jiff::{SignedDuration, Timestamp};
use timefilter::{TimeFilter, TimeOp};

// Relative filter: "not older than seven days"
let f: TimeFilter = ">=7d".parse().unwrap();
assert_eq!(f.op(), TimeOp::Ge);
assert!(f.is_relative());

let now: Timestamp = "2024-06-15T12:00:00Z".parse().unwrap();
let three_days_ago = now.checked_sub(SignedDuration::from_hours(3 * 24)).unwrap();
let ten_days_ago = now.checked_sub(SignedDuration::from_hours(10 * 24)).unwrap();
assert!(f.matches_at(three_days_ago, now));
assert!(!f.matches_at(ten_days_ago, now));

// `matches` uses the current time; `matches_at` is deterministic.
assert!(f.matches(Timestamp::now()));

// Absolute filter with an explicit offset
let cutoff: TimeFilter = "<2024-07-01T08:00:00+08:00".parse().unwrap();
assert!(cutoff.matches_at(now, now));
```

### Parsing times and durations

```rust
use jiff::SignedDuration;
use timefilter::{format_duration, parse_duration, parse_time};

assert_eq!(
    parse_time("2024-05-01 10:00").unwrap().to_string(),
    "2024-05-01T10:00:00Z",
);
assert_eq!(
    parse_time("2024-05-01T10:00:00+08:00").unwrap().to_string(),
    "2024-05-01T02:00:00Z",
);

assert_eq!(parse_duration("1h30m").unwrap(), SignedDuration::from_mins(90));
assert_eq!(parse_duration("1w").unwrap(), SignedDuration::from_hours(7 * 24));
assert_eq!(parse_duration("PT0.5S").unwrap(), SignedDuration::from_millis(500));
assert_eq!(parse_duration("-P1D").unwrap(), SignedDuration::from_hours(-24));
assert_eq!(format_duration(SignedDuration::from_mins(90)), "1h30m");
```

Human unit matching is case-insensitive: `m`/`M` mean minutes, while months
(`mo`, `month(s)`) are rejected. `parse_time` deliberately rejects relative
input. Use `parse_time_now` to resolve against the current time, or
`parse_time_at` for determinism:

```rust
use jiff::Timestamp;
use timefilter::{parse_time_at, parse_time_now};

let now: Timestamp = "2024-06-15T12:00:00Z".parse().unwrap();
assert_eq!(
    parse_time_at("2h", now).unwrap().to_string(),
    "2024-06-15T10:00:00Z",
);

// Reads the system clock; `parse_time_at` is preferred when determinism matters.
let _two_hours_ago = parse_time_now("2h").unwrap();
```

### Formatting

```rust
use jiff::Timestamp;
use timefilter::format_datetime;

let t: Timestamp = "2024-05-01T10:30:45Z".parse().unwrap();
assert_eq!(format_datetime(&t), "2024-05-01T10:30:45Z");

let t: Timestamp = "2024-05-01T10:30:45.250Z".parse().unwrap();
assert_eq!(format_datetime(&t), "2024-05-01T10:30:45.25Z");
```

`format_datetime` always emits UTC. For local presentation, convert the
re-exported jiff types with `timefilter::jiff::tz::TimeZone`; the library does
not read the process timezone.

### Serde

```toml
[dependencies]
timefilter = { version = "0.3", features = ["serde"] }
```

```rust
use timefilter::TimeFilter;

let f: TimeFilter = ">=7d".parse().unwrap();
let json = serde_json::to_string(&f).unwrap();
assert_eq!(json, "\">=7d\"");
assert_eq!(serde_json::from_str::<TimeFilter>(&json).unwrap(), f);
```

## Error handling

```rust
use timefilter::{parse_duration, parse_time, parse_time_filter, TimeError};

assert_eq!(parse_duration("P1Y"), Err(TimeError::UnsupportedUnit));
assert_eq!(parse_duration("1x"), Err(TimeError::UnknownUnit));
assert_eq!(parse_time_filter("1h"), Err(TimeError::MissingOperator));
assert_eq!(parse_time_filter(">="), Err(TimeError::MissingValue));
assert_eq!(parse_time_filter("=7d"), Err(TimeError::InvalidOperator));
assert_eq!(parse_time("2h"), Err(TimeError::InvalidDate));
assert_eq!(parse_time_filter(">=7d").unwrap().to_string(), ">=7d");
```

## License

MIT
