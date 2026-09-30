//! The solar-weather record, read back.
//!
//! The date arithmetic is the part that can be quietly wrong: an off-by-one in
//! a leap year moves every row after it by a day and nothing complains, so it
//! is checked against dates whose answer is known independently.

use std::fs;
use vleo_data::{load_bundle, read_solar_days, solar_day, solar_window};

fn bundle() -> vleo_data::Bundle {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("bundles/solar-weather/2026.09.14");
    load_bundle(&root).expect("the committed bundle loads")
}

#[test]
fn the_record_verifies_and_reads() {
    let b = bundle();
    assert!(b.verified, "the committed bundle must verify");
    let rows = read_solar_days(&b).expect("reads");
    assert_eq!(rows.len(), 10_319, "one row per day of the record");
}

#[test]
fn the_epoch_is_the_one_the_driver_table_uses() {
    // 2000-01-01 is day zero. Every other case below is counted from a date
    // whose offset can be worked out by hand, so a wrong epoch or a dropped
    // leap day shows up rather than shifting everything equally.
    let b = bundle();
    let rows = read_solar_days(&b).unwrap();
    // 1997-01-01 is 1095 days before 2000-01-01: 1997, 1998 and 1999 are three
    // common years, 365 each. (This assertion first said 1096, from counting a
    // leap day for 1996 — which is not inside the span. The reader was right
    // and the hand-check was wrong, which is the whole reason for checking the
    // arithmetic against dates worked out separately.)
    assert_eq!(rows[0].day, -1095, "the record starts 1997-01-01");
    // 2025-12-31 is 9496 days after 2000-01-01.
    assert_eq!(rows[rows.len() - 1].day, 9496, "and ends 2025-12-31");
}

#[test]
fn a_known_day_carries_its_known_numbers() {
    let b = bundle();
    let rows = read_solar_days(&b).unwrap();
    // Read straight out of the CSV, so this pins the parse rather than the
    // physics: if a column is inserted and the reader goes by position, this
    // returns some other quantity and fails.
    let csv = fs::read_to_string(b.dir.join("observed_daily.csv")).unwrap();
    let hdr: Vec<&str> = csv
        .lines()
        .find(|l| l.starts_with("date,"))
        .unwrap()
        .split(',')
        .collect();
    let line = csv
        .lines()
        .find(|l| l.starts_with("2003-10-29,"))
        .expect("the Halloween storm is in the record");
    let f: Vec<&str> = line.split(',').collect();
    let col = |n: &str| f[hdr.iter().position(|c| *c == n).unwrap()];

    let d = solar_day(&rows, vleo_data::days_since_2000("2003-10-29").unwrap())
        .expect("that day is in the record");
    assert_eq!(d.f107.unwrap().to_string(), col("f107"));
    assert_eq!(d.ap.unwrap().to_string(), col("ap_planetary"));
    assert_eq!(d.kp_max.unwrap().to_string(), col("kp_max"));
}

#[test]
fn a_window_is_the_days_it_asked_for() {
    let b = bundle();
    let rows = read_solar_days(&b).unwrap();
    let from = vleo_data::days_since_2000("2015-01-01").unwrap();
    let to = vleo_data::days_since_2000("2015-12-31").unwrap();
    let w = solar_window(&rows, from, to);
    assert_eq!(w.len(), 365, "2015 is not a leap year");
    assert_eq!(w[0].day, from);
    assert_eq!(w[w.len() - 1].day, to);

    let leap = solar_window(
        &rows,
        vleo_data::days_since_2000("2016-01-01").unwrap(),
        vleo_data::days_since_2000("2016-12-31").unwrap(),
    );
    assert_eq!(leap.len(), 366, "2016 is");
}

#[test]
fn a_window_outside_the_record_is_empty_not_wrong() {
    let b = bundle();
    let rows = read_solar_days(&b).unwrap();
    let after = vleo_data::days_since_2000("2030-01-01").unwrap();
    assert!(solar_window(&rows, after, after + 365).is_empty());
    assert!(solar_day(&rows, after).is_none());
    // A backwards window is empty rather than a panic or a reversed slice.
    assert!(solar_window(&rows, 100, 50).is_empty());
}

#[test]
fn a_gap_stays_a_gap() {
    // 10,319 days carry f107 on all but three, and Ap on twenty fewer. A
    // reader that turned a missing value into zero would make those days look
    // like the quietest in the record.
    let b = bundle();
    let rows = read_solar_days(&b).unwrap();
    let no_f107 = rows.iter().filter(|r| r.f107.is_none()).count();
    let no_ap = rows.iter().filter(|r| r.ap.is_none()).count();
    assert_eq!(no_f107, 3, "three days have no F10.7");
    assert_eq!(no_ap, 20, "twenty have no Ap");
    assert!(
        rows.iter().all(|r| r.f107.is_none_or(|v| v > 0.0)),
        "no present value is zero"
    );
}

#[test]
fn the_record_has_a_hole_and_says_so() {
    // The source's own metadata calls this record "gap-free daily observed".
    // It is not: 2017-01-01 to 2017-09-30 is absent, 273 days of it. This test
    // exists so the claim cannot quietly come back, and so anyone who fixes the
    // record upstream is told by a failing test rather than by a surprise.
    let b = bundle();
    let rows = read_solar_days(&b).unwrap();
    let from = vleo_data::days_since_2000("2017-01-01").unwrap();
    let to = vleo_data::days_since_2000("2017-09-30").unwrap();
    assert_eq!(
        solar_window(&rows, from, to).len(),
        0,
        "the whole run is absent"
    );
    assert_eq!(vleo_data::days_missing(&rows, from, to), 273);

    // A window that straddles it comes back short, and says by how much.
    let y = vleo_data::days_since_2000("2017-01-01").unwrap();
    let y_end = vleo_data::days_since_2000("2017-12-31").unwrap();
    assert_eq!(vleo_data::days_missing(&rows, y, y_end), 273);
    assert_eq!(solar_window(&rows, y, y_end).len(), 365 - 273);

    // And a window clear of it is complete.
    assert_eq!(
        vleo_data::days_missing(
            &rows,
            vleo_data::days_since_2000("2015-01-01").unwrap(),
            vleo_data::days_since_2000("2015-12-31").unwrap()
        ),
        0
    );
}

#[test]
fn a_date_read_in_and_written_back_is_the_date_it_was() {
    // Every day from 1990 to 2040, through both directions of the one
    // calendar: a figure that groups by month must put a day in the month
    // its row says.
    let from = vleo_data::days_since_2000("1990-01-01").unwrap();
    let to = vleo_data::days_since_2000("2040-12-31").unwrap();
    for day in from..=to {
        let (y, m, d, _) = vleo_data::civil_from_days(day);
        let back = vleo_data::days_since_2000(&format!("{y:04}-{m:02}-{d:02}"));
        assert_eq!(back, Some(day), "{y}-{m}-{d}");
    }
    // And the day of the year, across a leap year and not.
    assert_eq!(vleo_data::civil_from_days(0), (2000, 1, 1, 1));
    let at = |s: &str| vleo_data::civil_from_days(vleo_data::days_since_2000(s).unwrap());
    assert_eq!(at("2000-12-31"), (2000, 12, 31, 366));
    assert_eq!(at("2017-03-01"), (2017, 3, 1, 60));
    assert_eq!(at("1997-01-15"), (1997, 1, 15, 15));
}

#[test]
fn the_monthly_means_are_the_file_as_written() {
    let months = vleo_data::read_monthly_means(&bundle()).unwrap();
    // 1997-01 to 2025-12: the file's 339 rows.
    assert_eq!(months.len(), 339);
    let first = months[0];
    assert_eq!(first.day, vleo_data::days_since_2000("1997-01-15").unwrap());
    assert_eq!(
        (first.f107_mean, first.ssn_mean, first.ap_mean),
        (Some(74.0), Some(8.7), Some(8.82))
    );
    assert_eq!(
        (first.f107_smooth, first.ssn_smooth, first.ap_smooth),
        (Some(73.4), Some(18.1), Some(8.64))
    );
    assert!(months.windows(2).all(|w| w[1].day > w[0].day));
}

#[test]
fn the_issued_forecasts_are_the_file_as_written() {
    let b = bundle();
    let fc = vleo_data::read_forecast_issued(&b).unwrap();
    // The file's 30047 rows, first and last read straight off it.
    assert_eq!(fc.len(), 30_047);
    let d = |s| vleo_data::days_since_2000(s).unwrap();
    assert_eq!(
        fc[0],
        vleo_data::IssuedForecast {
            issue: d("1997-08-12"),
            target: d("1997-08-13"),
            lead: Some(1.0),
            f107: Some(82.0),
        }
    );
    let last = fc[fc.len() - 1];
    assert_eq!(
        (last.issue, last.target, last.lead, last.f107),
        (d("2025-12-29"), d("2026-01-24"), Some(26.0), Some(175.0))
    );
    // Its index: one date per issue, the quoted flags after it not in the way.
    let issues = vleo_data::read_forecast_issues(&b).unwrap();
    assert_eq!(issues.len(), 1437);
    assert_eq!(issues[0], d("1997-06-10"));
    assert_eq!(issues[issues.len() - 1], d("2025-12-29"));
}
