//! What the thermosphere figure says about the temperatures it draws.
//!
//! Every temperature on that panel is the engine's — a run, or a probe of
//! `env_exospheric_temperature` at chosen drivers — and the page used to work
//! out what it said about them itself: a slope, the gap between two Kp
//! readings, how far a curve stands above the line its own quiet end sets.
//! Phase 10 moves those here, beside the numbers they are about, so the page
//! only draws and a script asking the same question gets the same answer.
//!
//! Each is the page's to the last convention — the same points, the same
//! order, the first on a tie — so the pictures a person approved are the
//! pictures it still draws.

use alloc::vec::Vec;
use vleo_core::units::pmath;

/// The slope between the first and last drawn points of a curve, in the
/// curve's own units. None with fewer than two points or no run in x.
pub fn end_slope(xs: &[f64], ys: &[f64]) -> Option<f64> {
    let ok: Vec<(f64, f64)> = xs
        .iter()
        .zip(ys)
        .filter(|(_, y)| y.is_finite())
        .map(|(x, y)| (*x, *y))
        .collect();
    if ok.len() < 2 {
        return None;
    }
    let (a, b) = (ok[0], ok[ok.len() - 1]);
    (b.0 != a.0).then(|| (b.1 - a.1) / (b.0 - a.0))
}

/// The most two curves drawn on one grid ever differ by, point by point where
/// both have a value. Zero when they share no point.
pub fn widest_apart(a: &[Option<f64>], b: &[Option<f64>]) -> f64 {
    let mut close = 0.0;
    for (x, y) in a.iter().zip(b) {
        if let (Some(x), Some(y)) = (x, y) {
            close = f64::max(close, pmath::abs(y - x));
        }
    }
    close
}

/// Where a list of gaps is widest — the first on a tie — and its narrowest
/// and widest values.
pub fn gap_extent(gaps: &[Option<f64>]) -> (Option<usize>, Option<(f64, f64)>) {
    let mut at: Option<usize> = None;
    for (i, g) in gaps.iter().enumerate() {
        let Some(g) = g else { continue };
        if at.is_none_or(|b| *g > gaps[b].unwrap_or(f64::NAN)) {
            at = Some(i);
        }
    }
    let ok: Vec<f64> = gaps.iter().flatten().copied().collect();
    let range = (!ok.is_empty()).then(|| {
        (
            ok.iter().copied().fold(f64::INFINITY, f64::min),
            ok.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        )
    });
    (at, range)
}

/// A curve against the straight line its own quiet end sets.
#[derive(Clone, Debug, PartialEq)]
pub struct Departure {
    /// The curve measured from its first value, and the straight line through
    /// that point and the one nearest `quiet`.
    pub d: Vec<f64>,
    pub line: Vec<f64>,
    /// How far the curve stands above the line at its last point.
    pub excess: f64,
    /// Where the curve first stands clear of the line by more than a fiftieth
    /// of that — none if it never rises above it.
    pub split_at: Option<f64>,
}

/// The departure of a curve from the line its quiet end sets, the quiet end
/// being the point nearest `quiet` in x. None for a curve with no point.
pub fn departure(xs: &[f64], ys: &[f64], quiet: f64) -> Option<Departure> {
    let i0 = ys.iter().position(|y| y.is_finite())?;
    let base = ys[i0];
    let d: Vec<f64> = ys.iter().map(|y| y - base).collect();
    let mut at = i0;
    for i in 0..xs.len() {
        if d[i].is_finite() && pmath::abs(xs[i] - quiet) < pmath::abs(xs[at] - quiet) {
            at = i;
        }
    }
    let m = if xs[at] == xs[i0] {
        0.0
    } else {
        (d[at] - d[i0]) / (xs[at] - xs[i0])
    };
    let line: Vec<f64> = xs.iter().map(|x| m * (x - xs[i0])).collect();
    let last = d.len() - 1;
    let excess = d[last] - line[last];
    let split_at = (excess > 0.0)
        .then(|| {
            (0..=last)
                .find(|&i| d[i] - line[i] > excess / 50.0)
                .map(|i| xs[i])
        })
        .flatten();
    Some(Departure {
        d,
        line,
        excess,
        split_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_slope_is_taken_between_the_ends_that_were_drawn() {
        // The last point is missing, so the ends are (0, 1) and (2, 5).
        assert_eq!(
            end_slope(&[0.0, 1.0, 2.0, 3.0], &[1.0, 9.0, 5.0, f64::NAN]),
            Some(2.0)
        );
        assert_eq!(end_slope(&[0.0], &[1.0]), None);
        assert_eq!(end_slope(&[1.0, 1.0], &[1.0, 2.0]), None);
    }

    #[test]
    fn two_curves_are_compared_where_both_are_drawn() {
        let a = [Some(1.0), Some(2.0), None, Some(4.0)];
        let b = [Some(1.5), Some(5.0), Some(99.0), Some(3.0)];
        // 0.5, 3, skipped, 1.
        assert_eq!(widest_apart(&a, &b), 3.0);
        assert_eq!(widest_apart(&a[..1], &[None]), 0.0);
    }

    #[test]
    fn the_widest_gap_is_the_first_of_the_widest() {
        let (at, range) = gap_extent(&[None, Some(2.0), Some(7.0), Some(7.0), Some(1.0)]);
        assert_eq!((at, range), (Some(2), Some((1.0, 7.0))));
        assert_eq!(gap_extent(&[None, None]), (None, None));
    }

    #[test]
    fn a_curve_bending_up_parts_from_its_quiet_line() {
        // y = x for x ≤ 2, then x + (x − 2)²: straight at the quiet end, then
        // bending. The line through x = 0 and x = 2 is y = x.
        let xs = [0.0, 1.0, 2.0, 3.0, 4.0];
        let ys = [10.0, 11.0, 12.0, 14.0, 18.0];
        let dep = departure(&xs, &ys, 2.0).unwrap();
        assert_eq!(dep.d, [0.0, 1.0, 2.0, 4.0, 8.0]);
        assert_eq!(dep.line, [0.0, 1.0, 2.0, 3.0, 4.0]);
        // 8 − 4 at the top; clear of the line by more than 4/50 first at x = 3.
        assert_eq!(dep.excess, 4.0);
        assert_eq!(dep.split_at, Some(3.0));
        // A curve that never rises above its line has no split.
        assert_eq!(departure(&xs, &xs, 2.0).unwrap().split_at, None);
        assert_eq!(departure(&[], &[], 2.0), None);
    }
}
