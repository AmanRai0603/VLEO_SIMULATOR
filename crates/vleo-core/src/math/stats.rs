//! Statistics over a sample: the two estimators the figures of the record
//! read — a percentile and a correlation.
//!
//! These were JavaScript in the page (`web/js/record.js`) until phase 10, and
//! they are the page's to the last convention, because a figure that
//! illustrates a row must compute the row's quantity the row's way. The page
//! now asks the engine for the numbers and draws them; these are what answer.

use vleo_units::pmath;

/// A percentile of a SORTED sample, interpolated between order statistics.
///
/// Linear interpolation at `h = (n − 1)·q` — the convention the rows were
/// measured under. A nearest-rank percentile (the element at `floor(q·n)`)
/// disagreed with `sw_uncertainty_growth` by up to 0.7 sfu, because that row's
/// table holds values between two adjacent observations. `None` for an empty
/// sample, the one value for a sample of one.
pub fn quantile(sorted: &[f64], q: f64) -> Option<f64> {
    match sorted.len() {
        0 => None,
        1 => Some(sorted[0]),
        n => {
            let h = (n - 1) as f64 * q;
            let lo = pmath::floor(h) as usize;
            let hi = (lo + 1).min(n - 1);
            Some(sorted[lo] + (h - lo as f64) * (sorted[hi] - sorted[lo]))
        }
    }
}

/// Pearson's correlation of two samples of paired values.
///
/// `None` below three pairs, where a correlation is not a measurement, and
/// where either sample does not vary — a correlation of a constant is not
/// zero, it is undefined. The sums run in the order the pairs are given, the
/// order the page summed them in, so the two agree to the last bit the
/// arithmetic allows.
pub fn pearson(x: &[f64], y: &[f64]) -> Option<f64> {
    let n = x.len().min(y.len());
    if n < 3 {
        return None;
    }
    let (x, y) = (&x[..n], &y[..n]);
    let mx = x.iter().sum::<f64>() / n as f64;
    let my = y.iter().sum::<f64>() / n as f64;
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for i in 0..n {
        let (dx, dy) = (x[i] - mx, y[i] - my);
        sxy += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }
    if sxx == 0.0 || syy == 0.0 {
        return None;
    }
    Some(sxy / pmath::sqrt(sxx * syy))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_percentile_sits_between_the_observations_it_falls_between() {
        let s = [1.0, 2.0, 3.0, 4.0];
        assert_eq!(quantile(&s, 0.0), Some(1.0));
        assert_eq!(quantile(&s, 1.0), Some(4.0));
        // h = 3 × 0.5 = 1.5: halfway between the second and third.
        assert_eq!(quantile(&s, 0.5), Some(2.5));
        // h = 3 × 0.9 = 2.7: seven tenths of the way from 3 to 4.
        assert!((quantile(&s, 0.9).unwrap() - 3.7).abs() < 1e-12);
        assert_eq!(quantile(&[], 0.5), None);
        assert_eq!(quantile(&[7.0], 0.3), Some(7.0));
    }

    #[test]
    fn a_correlation_is_measured_or_it_is_not_given() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0];
        let up = [2.0, 4.0, 6.0, 8.0, 10.0];
        let down = [5.0, 4.0, 3.0, 2.0, 1.0];
        assert!((pearson(&x, &up).unwrap() - 1.0).abs() < 1e-12);
        assert!((pearson(&x, &down).unwrap() + 1.0).abs() < 1e-12);
        // A worked case: r = 0.8 for these five pairs (Σdxdy = 8, Σdx² = 10,
        // Σdy² = 10 → 8/10).
        let y = [1.0, 3.0, 2.0, 5.0, 4.0];
        assert!((pearson(&x, &y).unwrap() - 0.8).abs() < 1e-12);
        assert_eq!(pearson(&[1.0, 2.0], &[1.0, 2.0]), None);
        assert_eq!(pearson(&x, &[3.0; 5]), None);
    }
}
