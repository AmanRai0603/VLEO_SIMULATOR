//! Today and now, from the system clock, in the forms this tool writes them.
//!
//! ONE CLOCK, NO SUBPROCESS. Four copies of this used to run the `date`
//! program, which Windows does not have: every result saved there carried an
//! empty timestamp, and every form an empty date. The calendar arithmetic is
//! Howard Hinnant's days-to-civil conversion, eleven lines and exact for any
//! date the tool will see.
//!
//! In ring 0 because every ring above writes a date, and the calendar is
//! integer arithmetic; only reading the system clock needs `std`.

/// Seconds since 1970-01-01T00:00:00Z; 0 if the clock is before it.
fn unix_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// (year, month, day, hour, minute, second) in UTC for a Unix time.
pub fn civil(secs: i64) -> (i64, i64, i64, i64, i64, i64) {
    let z = secs.div_euclid(86_400) + 719_468;
    let tod = secs.rem_euclid(86_400);
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d, tod / 3600, tod % 3600 / 60, tod % 60)
}

/// Today, as the sheets write it: `YYYY-MM-DD`, UTC.
pub fn today() -> String {
    let (y, m, d, ..) = civil(unix_seconds());
    format!("{y:04}-{m:02}-{d:02}")
}

/// The date `days` before today, as `today` writes it.
pub fn days_ago(days: u32) -> String {
    let (y, m, d, ..) = civil(unix_seconds() - i64::from(days) * 86_400);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Now, as a result records it: `YYYY-MM-DDTHH:MM:SSZ`, UTC, to the second.
pub fn now_utc() -> String {
    let (y, m, d, h, mi, s) = civil(unix_seconds());
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

#[cfg(test)]
mod tests {
    use super::civil;

    #[test]
    fn known_instants() {
        assert_eq!(civil(0), (1970, 1, 1, 0, 0, 0));
        // 2000-02-29T12:34:56Z, a leap day in a century leap year.
        assert_eq!(civil(951_827_696), (2000, 2, 29, 12, 34, 56));
        // 2026-09-30T23:59:59Z
        assert_eq!(civil(1_790_812_799), (2026, 9, 30, 23, 59, 59));
    }
}
