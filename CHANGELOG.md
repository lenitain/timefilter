# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-06-04

- GitHub Actions CI workflow (build + test + fmt + clippy)
- Additional doc-tests covering edge cases in time parsing and filtering.

## [0.1.0] - 2026-05-13

### Added

- GitHub Actions CI workflow (build + test + fmt + clippy)

- Initial release of `timefilter`.
- `parse_time`: parse relative time strings (`"1h"`, `"30m"`, `"7d"`, `"2hr"`, `"15min"`, `"30s"`)
  and absolute datetime strings (`"2024-05-01 10:00"`, `"2024-12-25 15:30:45"`, `"2024-01-15"`).
- `format_datetime`: format `DateTime<Utc>` to human-readable string.
- `TimeFilter` / `TimeOp`: typed time filter with operators (`>=`, `>`, `<=`, `<`, `=`).
- `parse_time_filter`: parse combined operator + time strings (`">1h"`, `"<2026-05-01"`).
