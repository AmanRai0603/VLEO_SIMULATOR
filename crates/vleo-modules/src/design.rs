//! What the design and closure figures say about the curves they draw.
//!
//! Every curve on those two panels is the engine's — a sweep of a row across a
//! declared range — and the page used to work out what it said about them
//! itself: where a curve crosses a level, how many days of the record sit above
//! one, how wide the F10.7 window is, where a margin runs out. Phase 10 moves
//! those here, beside the numbers they are about, so the page only draws and a
//! script asking the same question gets the same answer.
//!
//! Each is the page's to the last convention — the same points, the same
//! order, the first crossing — so the pictures a person approved are the
//! pictures it still draws.

use alloc::vec::Vec;
use vleo_core::units::pmath;

/// The five closures, by their pair number, and the ACHIEVED QUANTITY's row
/// each one compares with its requirement.
///
/// Not the closure row: since §20 an achieved row publishes the signed margin
/// and reads the quantity as an input, so a picture of "what the record gives"
/// has to sweep the input rather than the row named for it. Getting that wrong
/// would draw a margin on an axis labelled sfu and look entirely plausible.
pub const CLOSURE_PAIRS: [(&str, &str); 5] = [
    ("01", "sw_f107_design_long"),
    ("02", "sw_f107_design_short"),
    ("03", "sw_storm_return_level"),
    ("04", "sw_ap_design_long"),
    ("05", "sw_ap_design_short"),
];

/// Where a swept curve first reaches `level`, read off the drawn points by
/// straight-line interpolation. A level at or below the first point is reached
/// at once; one the curve never reaches is None.
pub fn level_crossing(xs: &[f64], ys: &[f64], level: f64) -> Option<f64> {
    for i in 1..ys.len().min(xs.len()) {
        if (ys[i - 1] - level) * (ys[i] - level) <= 0.0 && ys[i] != ys[i - 1] {
            let f = (level - ys[i - 1]) / (ys[i] - ys[i - 1]);
            return Some(xs[i - 1] + f * (xs[i] - xs[i - 1]));
        }
    }
    match (ys.first(), xs.first()) {
        (Some(&y0), Some(&x0)) if level <= y0 => Some(x0),
        _ => None,
    }
}

/// Where a signed curve first changes sign between two drawn points, by
/// straight-line interpolation. Points that are not numbers are stepped over;
/// a curve that keeps one sign throughout is None.
pub fn zero_crossing(xs: &[f64], ys: &[f64]) -> Option<f64> {
    for k in 1..ys.len().min(xs.len()) {
        let (a, b) = (ys[k - 1], ys[k]);
        if !a.is_finite() || !b.is_finite() {
            continue;
        }
        if (a > 0.0) != (b > 0.0) {
            return Some(xs[k - 1] + (xs[k] - xs[k - 1]) * a / (a - b));
        }
    }
    None
}

/// Whether two crossings of one axis are the same point: both found, and
/// within a millionth of the axis's own span.
pub fn same_crossing(a: Option<f64>, b: Option<f64>, xs: &[f64]) -> bool {
    match (a, b, xs.first(), xs.last()) {
        (Some(a), Some(b), Some(x0), Some(x1)) => pmath::abs(a - b) <= pmath::abs(x1 - x0) * 1e-6,
        _ => false,
    }
}

/// The record above a level: the days that carry the driver, the years they
/// span, how many of them sit at or above it, in how many separate runs, and
/// the days a year that makes.
#[derive(Clone, Debug, PartialEq)]
pub struct Exceedance {
    pub days: usize,
    pub years: f64,
    pub above: usize,
    pub runs: usize,
    pub rate: f64,
}

/// Count the record's days at or above `level`, and the runs they fall in. A
/// run is broken wherever the next such day is not the day after the last.
/// `days` is each day's number and its value, None where the record is blank.
pub fn exceedance(days: &[(i32, Option<f64>)], level: f64) -> Exceedance {
    let mut n = 0usize;
    let (mut above, mut runs, mut prev) = (0usize, 0usize, i64::MIN);
    for (t, v) in days {
        let Some(v) = v else { continue };
        n += 1;
        if *v >= level {
            above += 1;
            if *t as i64 != prev + 1 {
                runs += 1;
            }
            prev = *t as i64;
        }
    }
    let years = n as f64 / 365.25;
    Exceedance {
        days: n,
        years,
        above,
        runs,
        rate: above as f64 / years,
    }
}

/// The narrowest and widest two curves on one grid stand apart, `upper` minus
/// `lower` at each of the first `n` points both have. None where they share
/// none.
pub fn band_extent(upper: &[f64], lower: &[f64], n: usize) -> Option<(f64, f64)> {
    let w: Vec<f64> = (0..n.min(upper.len()).min(lower.len()))
        .map(|k| upper[k] - lower[k])
        .filter(|v| v.is_finite())
        .collect();
    (!w.is_empty()).then(|| {
        (
            w.iter().copied().fold(f64::INFINITY, f64::min),
            w.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        )
    })
}

/// The first x at which curve `a` stands above curve `b`, both drawn on `xs`.
pub fn first_above(xs: &[f64], a: &[f64], b: &[f64]) -> Option<f64> {
    (0..xs.len().min(a.len()).min(b.len()))
        .find(|&k| a[k] > b[k])
        .map(|k| xs[k])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_level_is_crossed_where_the_drawn_segment_reaches_it() {
        let xs = [0.0, 1.0, 2.0, 3.0];
        let ys = [10.0, 20.0, 40.0, 80.0];
        // 30 is half way from 20 to 40.
        assert_eq!(level_crossing(&xs, &ys, 30.0), Some(1.5));
        // Below the first point, the curve is there already.
        assert_eq!(level_crossing(&xs, &ys, 5.0), Some(0.0));
        // Above the last, never.
        assert_eq!(level_crossing(&xs, &ys, 81.0), None);
        // A flat segment on the level says nothing about where it is crossed.
        assert_eq!(
            level_crossing(&[0.0, 1.0, 2.0], &[5.0, 5.0, 9.0], 7.0),
            Some(1.5)
        );
        assert_eq!(level_crossing(&[], &[], 1.0), None);
    }

    #[test]
    fn a_margin_runs_out_at_its_first_change_of_sign() {
        let xs = [0.0, 1.0, 2.0, 3.0, 4.0];
        // +0.3 to -0.1 crosses three quarters of the way; the later
        // crossing back is not the one asked for.
        let m = [0.5, 0.3, -0.1, -0.2, 0.4];
        assert_eq!(zero_crossing(&xs, &m), Some(1.75));
        assert_eq!(zero_crossing(&xs, &[1.0, f64::NAN, -1.0]), None);
        assert_eq!(zero_crossing(&xs, &[1.0, 2.0, 3.0]), None);
        assert!(same_crossing(Some(1.75), Some(1.75 + 1e-7), &xs));
        assert!(!same_crossing(Some(1.75), Some(1.8), &xs));
        assert!(!same_crossing(Some(1.75), None, &xs));
    }

    #[test]
    fn days_above_a_level_are_counted_in_runs() {
        // Days 1, 2 and 5 are at or above 100; day 3 is blank and day 4 is
        // under, so they are two runs. Six days carry a value: 6/365.25 yr.
        let days = [
            (0, Some(40.0)),
            (1, Some(100.0)),
            (2, Some(150.0)),
            (3, None),
            (4, Some(99.0)),
            (5, Some(120.0)),
            (6, Some(10.0)),
        ];
        let e = exceedance(&days, 100.0);
        assert_eq!((e.days, e.above, e.runs), (6, 3, 2));
        assert_eq!(e.years, 6.0 / 365.25);
        assert_eq!(e.rate, 3.0 / (6.0 / 365.25));
        // A blank day between two above it still breaks the run.
        let gap = [(1, Some(200.0)), (2, None), (3, Some(200.0))];
        assert_eq!(exceedance(&gap, 100.0).runs, 2);
    }

    #[test]
    fn a_window_is_measured_between_its_edges() {
        let hot = [200.0, 210.0, 190.0];
        let cold = [80.0, 70.0, 100.0, 1.0];
        assert_eq!(band_extent(&hot, &cold, 3), Some((90.0, 140.0)));
        // Only the first n points count.
        assert_eq!(band_extent(&hot, &cold, 1), Some((120.0, 120.0)));
        assert_eq!(band_extent(&hot, &[], 3), None);
        assert_eq!(
            first_above(&[1.0, 2.0, 3.0], &[5.0, 9.0, 9.0], &[6.0, 8.0, 1.0]),
            Some(2.0)
        );
        assert_eq!(first_above(&[1.0], &[5.0], &[6.0]), None);
    }
}
