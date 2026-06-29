# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- `TimeFilter` constructors (`gt`, `ge`, `lt`, `le`, `eq`) are now `const fn`, enabling compile-time construction.
- Simplified operator parsing in `parse_time_filter` using `find_map` instead of chained `if-else`.
- Simplified `UnknownSuffix` error message — removed incomplete expected suffix list.

### Fixed

- Clippy `doc_lazy_continuation` warning in `parse_duration` docs.

### Docs

- Added License and CI badges to README.

## [0.2.0] - 2026-06-29

### Changed

- **`TimeFilter` fields are now private** ([C-STRUCT-PRIVATE](https://rust-lang.github.io/api-guidelines/interoperability.html#types-are-send-and-sync-where-possible-c-send-sync)):
  - `op` and `time` fields are now private
  - Added `op()` getter method (returns `TimeOp`)
  - Added `time()` getter method (returns `DateTime<Utc>`)
  - `new()` constructor remains unchanged

- **`TimeOp` implements `Hash`, `Ord`, `PartialOrd`** ([C-COMMON-TRAITS](https://rust-lang.github.io/api-guidelines/interoperability.html#commonly-used-types-should-be-the-same-c-common-traits)):
  ```rust
  use std::collections::HashSet;
  use timefilter::TimeOp;

  let ops = HashSet::from([TimeOp::Ge, TimeOp::Lt]);
  assert!(ops.contains(&TimeOp::Ge));
  ```

- **`TimeFilter` implements `Hash`**: can be used as `HashSet` or `HashMap` key.

- **Serde serialization preserves UTC time** (breaking): `TimeFilter` now serializes with UTC time instead of local timezone to ensure roundtrip accuracy.

### Migration Guide

**Struct field access** (breaking):
```rust
// Before (0.1.x)
let f: TimeFilter = ">=2024-05-01".parse().unwrap();
assert_eq!(f.op, TimeOp::Ge);
assert_eq!(f.time, some_datetime);

// After (0.2.0)
let f: TimeFilter = ">=2024-05-01".parse().unwrap();
assert_eq!(f.op(), TimeOp::Ge);
assert_eq!(f.time(), some_datetime);
```

## [0.1.2] - 2026-06-05

### Added

- `parse_duration`: parse time strings to `Duration` without subtracting from current time.
  Supports human-readable formats (`"7d"`, `"2h"`, `"30m"`, `"30s"`) and ISO 8601 duration format (`"P7D"`, `"PT2H"`, `"P1DT12H"`).
- Doc-tests for `parse_duration` covering human-readable and ISO 8601 formats.
- Updated README with `parse_duration` examples and feature description.
- Test coverage increased from 30 to 43 tests.

## [0.1.1] - 2026-06-04

- GitHub Actions CI workflow (build + test + fmt + clippy)
- Additional doc-tests covering edge cases in time parsing and filtering.

## [0.1.0] - 2026-05-13

### Added

- Initial release of `timefilter`.
- `parse_time`: parse relative time strings (`"1h"`, `"30m"`, `"7d"`, `"2hr"`, `"15min"`, `"30s"`)
  and absolute datetime strings (`"2024-05-01 10:00"`, `"2024-12-25 15:30:45"`, `"2024-01-15"`).
- `format_datetime`: format `DateTime<Utc>` to human-readable string.
- `TimeFilter` / `TimeOp`: typed time filter with operators (`>=`, `>`, `<=`, `<`, `=`).
- `parse_time_filter`: parse combined operator + time strings (`">1h"`, `"<2026-05-01"`).
