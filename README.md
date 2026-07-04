# timefilter

[![Crates.io](https://img.shields.io/crates/v/timefilter.svg)](https://crates.io/crates/timefilter)
[![Docs.rs](https://docs.rs/timefilter/badge.svg)](https://docs.rs/timefilter)

Human-readable time string parsing and **filtering with comparison operators** — `">=7d"`, `"<2h"`, `"2024-05-01"`.

## Installation

```bash
cargo add timefilter
```

## Features

- **Parse relative time**: `"7d"`, `"2h"`, `"30m"`, `"30s"` → `DateTime<Utc>` (now — duration)
- **Parse absolute time**: `"2024-05-01"`, `"2024-05-01 10:00"`, `"2024-05-01 10:00:00"`
- **Filter**: `TimeFilter::ge(threshold)` or `">=7d".parse::<TimeFilter>()`
- **Format**: `format_datetime(&dt)` → local timezone string
- **Serde** (optional): serialize/deserialize [`TimeFilter`] as strings

## Testing

```bash
cargo test
```

30 tests (25 unit + 5 doc-tests) covering parsing, filtering, edge cases, and formatting.

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

## License

[MIT License](./LICENSE)
