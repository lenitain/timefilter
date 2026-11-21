//! Duration parsing and canonical formatting.
//!
//! Two input grammars are supported:
//!
//! * **Human readable** — one or more `<number><unit>` pairs, where the unit is
//!   `w`/`week(s)`, `d`/`day(s)`, `h`/`hr(s)`/`hour(s)`,
//!   `m`/`min(s)`/`minute(s)` or `s`/`sec(s)`/`second(s)`. Components are
//!   summed: `1h30m`, `2 days`.
//! * **ISO 8601** — `PnWnDTnHnMnS`, with components in that order and at most
//!   one fractional component, which must be the smallest one. A week is
//!   standalone: `P1W` is valid, `P1W2D` is not. A leading `+` or `-` is
//!   accepted. Years and months are rejected because their length depends on a
//!   calendar.
//!
//! Numbers may be fractional (`1.5h`, `.5h`, `P1.5D`). Sub-nanosecond
//! precision is truncated, matching [`SignedDuration`]'s resolution. Every
//! arithmetic operation is checked: no input can overflow or panic.

use std::fmt;

use jiff::SignedDuration;

use crate::error::{TimeError, TimeResult};

const NANOS_PER_SECOND: i128 = 1_000_000_000;

/// Which half of an ISO 8601 duration is being parsed.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Date,
    Time,
}

/// Parse a human readable or ISO 8601 duration.
///
/// ```
/// use jiff::SignedDuration;
/// use timefilter::parse_duration;
///
/// assert_eq!(parse_duration("7d").unwrap(), SignedDuration::from_hours(7 * 24));
/// assert_eq!(parse_duration("1h30m").unwrap(), SignedDuration::from_mins(90));
/// assert_eq!(parse_duration("1.5h").unwrap(), SignedDuration::from_mins(90));
/// assert_eq!(parse_duration("1w").unwrap(), SignedDuration::from_hours(7 * 24));
/// assert_eq!(parse_duration("P1DT12H").unwrap(), SignedDuration::from_hours(36));
/// assert_eq!(parse_duration("P1W").unwrap(), SignedDuration::from_hours(7 * 24));
/// assert_eq!(parse_duration("-P1D").unwrap(), SignedDuration::from_hours(-24));
/// assert_eq!(parse_duration("P0D").unwrap(), SignedDuration::ZERO);
/// ```
///
/// Unit matching is ASCII case-insensitive. In the human grammar, `m`/`M`
/// mean minutes; months are written `mo`/`month(s)` and are rejected.
///
/// # Errors
///
/// Returns an error for empty input, malformed numbers, unknown units,
/// unsupported units (years, months), invalid ISO ordering or week
/// combinations, and values outside [`SignedDuration`]'s range.
pub fn parse_duration(s: &str) -> TimeResult<SignedDuration> {
    let s = s.trim();
    if s.is_empty() {
        return Err(TimeError::EmptyInput);
    }
    if s.starts_with(['P', 'p']) {
        return parse_iso_duration(s);
    }
    if let Some(rest) = s.strip_prefix('-') {
        if rest.starts_with(['P', 'p']) {
            return parse_iso_duration(rest)?
                .checked_neg()
                .ok_or(TimeError::Overflow);
        }
    } else if let Some(rest) = s.strip_prefix('+') {
        if rest.starts_with(['P', 'p']) {
            return parse_iso_duration(rest);
        }
    }
    parse_human_duration(s)
}

/// Format a duration in canonical human readable form.
///
/// The output is accepted by [`parse_duration`], so display/parse round trips
/// preserve the exact nanosecond value.
///
/// ```
/// use jiff::SignedDuration;
/// use timefilter::format_duration;
///
/// assert_eq!(format_duration(SignedDuration::from_hours(7 * 24)), "7d");
/// assert_eq!(format_duration(SignedDuration::from_mins(90)), "1h30m");
/// assert_eq!(format_duration(SignedDuration::ZERO), "0s");
/// assert_eq!(format_duration(SignedDuration::from_millis(-1500)), "-1.5s");
/// ```
#[must_use]
pub fn format_duration(span: SignedDuration) -> String {
    let mut out = String::new();
    write_duration(&mut out, span).expect("writing to a String is infallible");
    out
}

/// Write a duration in canonical human readable form.
pub(crate) fn write_duration<W: fmt::Write>(out: &mut W, span: SignedDuration) -> fmt::Result {
    let total_nanos = span.as_nanos();
    if total_nanos == 0 {
        return out.write_str("0s");
    }

    let negative = total_nanos < 0;
    let remainder = total_nanos.unsigned_abs();
    if negative {
        out.write_char('-')?;
    }

    let seconds = remainder / NANOS_PER_SECOND as u128;
    // The remainder is below one second, so the cast cannot truncate.
    #[allow(clippy::cast_possible_truncation)]
    let nanos = (remainder % NANOS_PER_SECOND as u128) as u32;

    let days = seconds / 86_400;
    let seconds = seconds % 86_400;
    let hours = seconds / 3_600;
    let seconds = seconds % 3_600;
    let minutes = seconds / 60;
    let seconds = seconds % 60;

    if days > 0 {
        write!(out, "{days}d")?;
    }
    if hours > 0 {
        write!(out, "{hours}h")?;
    }
    if minutes > 0 {
        write!(out, "{minutes}m")?;
    }
    if seconds > 0 || nanos > 0 {
        if nanos == 0 {
            write!(out, "{seconds}s")?;
        } else {
            let mut fraction = format!("{nanos:09}");
            while fraction.ends_with('0') {
                fraction.pop();
            }
            write!(out, "{seconds}.{fraction}s")?;
        }
    }
    Ok(())
}

/// Whether `s` is shaped like a duration rather than a date.
///
/// Used only to pick the more informative error when both grammars fail.
pub(crate) fn looks_like_duration(s: &str) -> bool {
    let s = s.trim();
    let s = s.strip_prefix(['+', '-']).unwrap_or(s).trim_start();
    if s.starts_with(['P', 'p']) {
        return true;
    }
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
        i += 1;
    }
    i > 0 && i < bytes.len() && bytes[i].is_ascii_alphabetic()
}

fn parse_human_duration(s: &str) -> TimeResult<SignedDuration> {
    let (negative, mut rest) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s.strip_prefix('+').unwrap_or(s)),
    };

    let mut total_nanos: i128 = 0;
    let mut components = 0usize;
    loop {
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        let (value, digits, after_number) = parse_decimal(rest, true)?;
        let (unit_seconds, after_unit) = parse_human_unit(after_number.trim_start())?;
        let unit_nanos = i128::from(unit_seconds) * NANOS_PER_SECOND;
        let component_nanos =
            value.checked_mul(unit_nanos).ok_or(TimeError::Overflow)? / pow10(digits)?;
        total_nanos = total_nanos
            .checked_add(component_nanos)
            .ok_or(TimeError::Overflow)?;
        components += 1;
        rest = after_unit;
    }

    if components == 0 {
        return Err(TimeError::InvalidDuration);
    }
    if negative {
        total_nanos = total_nanos.checked_neg().ok_or(TimeError::Overflow)?;
    }
    signed_duration_from_nanos(total_nanos)
}

fn parse_iso_duration(s: &str) -> TimeResult<SignedDuration> {
    let body = &s[1..];
    if body.is_empty() {
        return Err(TimeError::InvalidDuration);
    }

    let bytes = body.as_bytes();
    let mut phase = Phase::Date;
    let mut last_rank = 0u8;
    let mut total_nanos: i128 = 0;
    let mut components = 0usize;
    let mut time_components = 0usize;
    let mut index = 0usize;

    while index < bytes.len() {
        if bytes[index] == b'T' || bytes[index] == b't' {
            if phase == Phase::Time {
                return Err(TimeError::InvalidDuration);
            }
            phase = Phase::Time;
            last_rank = 0;
            index += 1;
            if index == bytes.len() {
                return Err(TimeError::InvalidDuration);
            }
            continue;
        }

        let (value, digits, after_number) = parse_decimal(&body[index..], false)?;
        index = body.len() - after_number.len();
        if index == bytes.len() {
            return Err(TimeError::InvalidDuration);
        }

        let unit = bytes[index].to_ascii_lowercase();
        index += 1;
        // A week is a standalone alternative in ISO 8601; it may not be
        // combined with days, time components, or anything else.
        if unit == b'w' && index != bytes.len() {
            return Err(TimeError::InvalidDuration);
        }
        // The two `InvalidDuration` arms below look identical but represent
        // different grammar violations: a time unit without `T`, and a date
        // unit after `T`. Keeping them separate documents each rule.
        #[allow(clippy::match_same_arms)]
        let (rank, unit_seconds) = match (phase, unit) {
            // Calendar-dependent units have no fixed duration.
            (Phase::Date, b'y' | b'm') => return Err(TimeError::UnsupportedUnit),
            (Phase::Date, b'w') => (1, 604_800),
            (Phase::Date, b'd') => (2, 86_400),
            // Time units in the date part: the `T` separator is missing.
            (Phase::Date, b'h' | b's') => return Err(TimeError::InvalidDuration),
            (Phase::Time, b'h') => (1, 3_600),
            (Phase::Time, b'm') => (2, 60),
            (Phase::Time, b's') => (3, 1),
            // Date units after `T` are out of order.
            (Phase::Time, b'y' | b'w' | b'd') => return Err(TimeError::InvalidDuration),
            _ => return Err(TimeError::UnknownUnit),
        };
        if rank <= last_rank {
            return Err(TimeError::InvalidDuration);
        }
        last_rank = rank;
        if digits > 0 && index != bytes.len() {
            // A fractional component must be the smallest (last) one.
            return Err(TimeError::InvalidDuration);
        }

        let unit_nanos = i128::from(unit_seconds) * NANOS_PER_SECOND;
        let component_nanos =
            value.checked_mul(unit_nanos).ok_or(TimeError::Overflow)? / pow10(digits)?;
        total_nanos = total_nanos
            .checked_add(component_nanos)
            .ok_or(TimeError::Overflow)?;

        components += 1;
        if phase == Phase::Time {
            time_components += 1;
        }
    }

    if components == 0 || (phase == Phase::Time && time_components == 0) {
        return Err(TimeError::InvalidDuration);
    }
    signed_duration_from_nanos(total_nanos)
}

/// Parse an unsigned decimal number, returning its scaled value and digit count.
///
/// `allow_leading_dot` makes `.5h` valid in the human grammar while keeping
/// ISO 8601 strict (`P.5D` is rejected).
fn parse_decimal(s: &str, allow_leading_dot: bool) -> TimeResult<(i128, u32, &str)> {
    let bytes = s.as_bytes();
    let mut index = 0;
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
    }
    if index == 0 && !(allow_leading_dot && bytes.first() == Some(&b'.')) {
        return Err(TimeError::InvalidNumber);
    }
    let integer: i128 = if index == 0 {
        0
    } else {
        s[..index].parse().map_err(|_| TimeError::InvalidNumber)?
    };

    let mut digits = 0u32;
    let mut fraction: i128 = 0;
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        let start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        if index == start {
            return Err(TimeError::InvalidNumber);
        }
        digits = u32::try_from(index - start).map_err(|_| TimeError::InvalidNumber)?;
        fraction = s[start..index]
            .parse()
            .map_err(|_| TimeError::InvalidNumber)?;
    }

    let value = integer
        .checked_mul(pow10(digits)?)
        .and_then(|value| value.checked_add(fraction))
        .ok_or(TimeError::Overflow)?;
    Ok((value, digits, &s[index..]))
}

fn parse_human_unit(s: &str) -> TimeResult<(i64, &str)> {
    let bytes = s.as_bytes();
    let mut index = 0;
    while index < bytes.len() && bytes[index].is_ascii_alphabetic() {
        index += 1;
    }
    if index == 0 {
        return Err(TimeError::InvalidDuration);
    }
    let unit = &s[..index];
    let seconds = match unit {
        _ if unit.eq_ignore_ascii_case("w")
            || unit.eq_ignore_ascii_case("wk")
            || unit.eq_ignore_ascii_case("wks")
            || unit.eq_ignore_ascii_case("week")
            || unit.eq_ignore_ascii_case("weeks") =>
        {
            604_800
        }
        _ if unit.eq_ignore_ascii_case("d")
            || unit.eq_ignore_ascii_case("day")
            || unit.eq_ignore_ascii_case("days") =>
        {
            86_400
        }
        _ if unit.eq_ignore_ascii_case("h")
            || unit.eq_ignore_ascii_case("hr")
            || unit.eq_ignore_ascii_case("hrs")
            || unit.eq_ignore_ascii_case("hour")
            || unit.eq_ignore_ascii_case("hours") =>
        {
            3_600
        }
        _ if unit.eq_ignore_ascii_case("m")
            || unit.eq_ignore_ascii_case("min")
            || unit.eq_ignore_ascii_case("mins")
            || unit.eq_ignore_ascii_case("minute")
            || unit.eq_ignore_ascii_case("minutes") =>
        {
            60
        }
        _ if unit.eq_ignore_ascii_case("s")
            || unit.eq_ignore_ascii_case("sec")
            || unit.eq_ignore_ascii_case("secs")
            || unit.eq_ignore_ascii_case("second")
            || unit.eq_ignore_ascii_case("seconds") =>
        {
            1
        }
        _ if unit.eq_ignore_ascii_case("y")
            || unit.eq_ignore_ascii_case("yr")
            || unit.eq_ignore_ascii_case("yrs")
            || unit.eq_ignore_ascii_case("year")
            || unit.eq_ignore_ascii_case("years")
            || unit.eq_ignore_ascii_case("mo")
            || unit.eq_ignore_ascii_case("mon")
            || unit.eq_ignore_ascii_case("mons")
            || unit.eq_ignore_ascii_case("month")
            || unit.eq_ignore_ascii_case("months") =>
        {
            return Err(TimeError::UnsupportedUnit);
        }
        _ => return Err(TimeError::UnknownUnit),
    };
    Ok((seconds, &s[index..]))
}

fn pow10(digits: u32) -> TimeResult<i128> {
    10i128.checked_pow(digits).ok_or(TimeError::InvalidNumber)
}

fn signed_duration_from_nanos(nanos: i128) -> TimeResult<SignedDuration> {
    SignedDuration::try_from_nanos_i128(nanos).ok_or(TimeError::Overflow)
}
