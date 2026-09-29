//! The calendar every face stamps its files with, against dates fixed by
//! definition: the epoch, leap days either side of a century rule, and the end
//! of a year.

use vleo_units::calendar::Civil;

#[test]
fn known_instants_come_back_as_their_dates() {
    for (secs, want) in [
        (0, "1970-01-01T00:00:00Z"),
        (86_399, "1970-01-01T23:59:59Z"),
        (951_782_400, "2000-02-29T00:00:00Z"),
        (1_709_164_800, "2024-02-29T00:00:00Z"),
        (4_107_542_400, "2100-03-01T00:00:00Z"),
        (1_798_761_599, "2026-12-31T23:59:59Z"),
        (-1, "1969-12-31T23:59:59Z"),
    ] {
        assert_eq!(Civil::from_unix(secs).to_string(), want, "{secs}");
    }
    assert_eq!(
        Civil::from_unix(1_790_553_600).date().to_string(),
        "2026-09-28"
    );
}
