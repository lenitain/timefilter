# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-11-21

Correctness release. The public API changed; there is no compatibility shim.

### Changed

- `TimeFilter` is now a `#[non_exhaustive]` enum: `Absolute { op, time }` or
  `Relative { op, span }`. Relative filters resolve their threshold at check
  time instead of being frozen to `now - duration` during parsing. A negative
  span places the threshold in the future; the comparison is still against the
  resolved timestamp.
- Added `TimeFilter::matches_at(value, now)` and `TimeFilter::resolve(now)`.
  `matches` remains as the current-time convenience, and `try_matches` is its
  fallible form. Comparisons whose resolved threshold falls outside the
  representable timestamp range are still evaluated mathematically instead of
  panicking.
- Added relative constructors: `relative`, `gt_span`, `ge_span`, `lt_span`,
  `le_span`, plus intent-named `newer_than`, `older_than`, and `within`
  (const where the inputs allow it).
- `Display` and serde now share one canonical representation. Absolute filters
  render as RFC 3339 UTC with sub-second precision; relative filters render as
  canonical durations (`>=7d`, `>=1h30m`). Round trips preserve the exact value.
- `parse_time` now accepts absolute input only; parsing no longer reads the
  clock. Added `parse_time_now` for current-time resolution and kept
  `parse_time_at(s, now)` for deterministic use.
- Absolute parsing accepts RFC 3339 (including offsets and fractional seconds),
  `T` or space separators, and date-only input. Naive input is UTC.
- `format_datetime` now returns RFC 3339 UTC. `format_datetime_local` was
  removed: converting to the process timezone is an application concern, and
  jiff's time zone types are available to callers that need it.
- `parse_duration` accepts compound and fractional human durations (`1h30m`,
  `.5h`, `1.25w`) and implements ISO 8601 ordering rules: units must be in
  order, may not repeat, `T` must introduce at least one time component, and
  only the smallest component may carry a fraction. Zero durations are valid.
  Human durations now accept `w`/`week(s)`; ISO weeks are standalone (`P1W`,
  not `P1W2D`), and a leading ISO `+`/`-` is accepted (`-P1D`).
- Years and months are rejected with `UnsupportedUnit` in both grammars.
- `TimeError` variants are explicit: `MissingOperator`, `InvalidOperator`,
  `MissingValue`, `EmptyInput`, `InvalidNumber`, `UnknownUnit`,
  `UnsupportedUnit`, `InvalidDuration`, `InvalidDate`, `Overflow`.
- `TimeOp` implements `FromStr`; derived `Ord` was removed because declaration
  order is not a meaningful order for operators. Time filters now support
  ordering only: `Eq` was removed because exact nanosecond equality is not a
  useful filter. `TimeOp` is `#[non_exhaustive]`.
- `TimeFilter` fields are only reachable through the enum variants and getters;
  `time()` and `span()` return `Option`.
- Replaced the unmaintained `chrono` dependency
  ([RUSTSEC-2026-0543](https://rustsec.org/advisories/RUSTSEC-2026-0543)) with
  `jiff` 1.0. Public timestamps are `jiff::Timestamp` and durations are
  `jiff::SignedDuration`, both re-exported from the crate root. The whole jiff
  crate is re-exported as `timefilter::jiff` so callers stay on one version.
- Added `rust-version = "1.85"`, docs.rs `all-features`, and kept tests and the
  changelog in the published package.

### Fixed

- `parse_time_now` and `try_matches` no longer panic when the system clock is
  outside jiff's representable range; they return `Overflow`.
- Parsing huge durations (`"200000000000000000d"`,
  `"P200000000000000000D"`) no longer panics; it returns `Overflow`.
- `Display`/`FromStr` and serde round trips no longer shift absolute filters by
  the local timezone or drop sub-second precision.
- Zero ISO durations (`P0D`, `PT0S`) are accepted; structurally invalid ISO
  durations (`P1DT`, `PT1H1H`, `PT1M1H`) are rejected.
- `parse_time_filter(">=")` now returns `TimeError::MissingValue`; `=`, `!=`,
  and other unsupported or malformed operators return `InvalidOperator`
  instead of `MissingOperator`.
- Serde round trips of relative filters no longer change their span.

### Migration

- `TimeOp::Eq` and `TimeFilter::{eq, eq_span}` were removed. Use a range such
  as `>=2024-05-01` and `<2024-05-02`; relative filters can use `>=`/`<` with
  a span.
- `parse_time("2h")` now returns `InvalidDate`. Use `parse_time_now("2h")` for
  the old current-time behavior, or `parse_time_at("2h", now)` for determinism.
- Code that matched `TimeOp` or `TimeFilter` exhaustively must add a wildcard
  arm because both enums are now `#[non_exhaustive]`.
- `parse_time_filter("=")`/`"=7d"` and `"!7d"` now return
  `TimeError::InvalidOperator`; operator-only input returns `MissingValue`.
- ISO `P1W2D` and `P1WT1H` are now rejected; `P1W` remains valid. `-P1D` is
  now accepted.

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

- `TimeFilter` constructors (`gt`, `ge`, `lt`, `le`, `eq`) are now `const fn`, enabling compile-time construction.
- Simplified operator parsing in `parse_time_filter` using `find_map` instead of chained `if-else`.
- Simplified `UnknownSuffix` error message — removed incomplete expected suffix list.

### Fixed

- Clippy `doc_lazy_continuation` warning in `parse_duration` docs.

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
