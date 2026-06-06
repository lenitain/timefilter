# timefilter

[![Crates.io](https://img.shields.io/crates/v/timefilter.svg)](https://crates.io/crates/timefilter)
[![Docs.rs](https://docs.rs/timefilter/badge.svg)](https://docs.rs/timefilter)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/lenitain/timefilter/actions/workflows/ci.yml/badge.svg)](https://github.com/lenitain/timefilter/actions/workflows/ci.yml)

Human-readable time string parsing and **filtering with comparison operators** — `">=7d"`, `"<2h"`, `"2024-05-01"`.

## Installation

```bash
cargo add timefilter
```

## Features

- **Parse relative time**: `"7d"`, `"2h"`, `"30m"`, `"30s"` → `DateTime<Utc>` (now — duration)
- **Parse absolute time**: `"2024-05-01"`, `"2024-05-01 10:00"`, `"2024-05-01 10:00:00"`
- **Parse duration**: `"7d"`, `"2h"`, `"P7D"`, `"PT2H"` → `Duration` (without subtracting from now)
- **Filter**: `TimeFilter::ge(threshold)` or `">=7d".parse::<TimeFilter>()`
- **Format**: `format_datetime(&dt)` → local timezone string
- **Serde** (optional): serialize/deserialize [`TimeFilter`] as strings

## Testing

```bash
cargo test
```

42 tests covering parsing, filtering, edge cases, and formatting.

## Quick example

```rust
use timefilter::prelude::*;

// Parse absolute time
let dt = parse_time("2024-05-01 10:00").unwrap();

// Parse a filter expression
let f: TimeFilter = ">=2024-05-01 09:00".parse().unwrap();
assert!(f.matches(dt));

// Relative time (hours ago)
let dt = parse_time("2h").unwrap();
let age = chrono::Utc::now() - dt;
assert!(age.num_hours() >= 1);
```

## Parse duration

```rust
use timefilter::parse_duration;
use chrono::Duration;

// Human-readable formats
assert_eq!(parse_duration("7d").unwrap(), Duration::days(7));
assert_eq!(parse_duration("2h").unwrap(), Duration::hours(2));
assert_eq!(parse_duration("30m").unwrap(), Duration::minutes(30));

// ISO 8601 duration format
assert_eq!(parse_duration("P7D").unwrap(), Duration::days(7));
assert_eq!(parse_duration("PT2H").unwrap(), Duration::hours(2));
assert_eq!(parse_duration("P1DT12H").unwrap(), Duration::days(1) + Duration::hours(12));
```

## License

[MIT License](./LICENSE)
