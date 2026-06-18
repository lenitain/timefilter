# timefilter

Human-readable time string parsing and filtering with comparison operators.

[![Crates.io](https://img.shields.io/crates/v/timefilter.svg)](https://crates.io/crates/timefilter)
[![Docs.rs](https://docs.rs/timefilter/badge.svg)](https://docs.rs/timefilter)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/lenitain/timefilter/actions/workflows/ci.yml/badge.svg)](https://github.com/lenitain/timefilter/actions/workflows/ci.yml)

## Overview

**timefilter** provides human-readable time string parsing and filtering with comparison operators. It supports both relative time expressions (`>=7d`, `<2h`) and absolute timestamps (`2024-05-01`), with optional serde integration for configuration files. The library offers intuitive time manipulation with support for ISO 8601 duration formats.

### Why timefilter?

Unlike other time parsing libraries that only handle basic conversions, **timefilter** provides a complete solution for working with human-readable time expressions in Rust. It supports comparison operators (`>=7d`, `<2h`, `=0`), relative time parsing from "now", and seamless integration with chrono for time arithmetic. The library's support for both relative and absolute time expressions makes it ideal for configuration files, log filtering, and any application that needs to parse time durations from user input. For tools that need to filter events by time or parse human-readable time expressions, timefilter offers the most ergonomic and complete solution.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
timefilter = "0.1.2"
```

### Quick start

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

## Building from Source

Requires Rust toolchain.

```bash
git clone https://github.com/lenitain/timefilter.git
cd timefilter
cargo build --release
```
