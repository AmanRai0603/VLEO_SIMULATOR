//! Statistics over a sample: the estimators the figures of the record read —
//! a percentile, a correlation, a centred moving mean, a lagged correlation
//! with the pairs behind it, Bartlett's band, and a mean with its scatter.
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

/// The mean and the population standard deviation of a sample, summed in the
/// order given; `None` for an empty sample.
pub fn mean_sd(x: &[f64]) -> Option<(f64, f64)> {
    if x.is_empty() {
        return None;
    }
    let n = x.len() as f64;
    let mu = x.iter().sum::<f64>() / n;
    let var = x.iter().map(|v| (v - mu) * (v - mu)).sum::<f64>() / n;
    Some((mu, pmath::sqrt(var)))
}

/// A centred moving mean of a series with gaps, into `out`, one per value.
///
/// Centred, not trailing: a trailing mean sits below the series during a rise
/// and above it during a fall, so a ratio against it is not symmetric. The
/// window is `w` VALUES wide — `floor(w/2)` either side, cut at the ends — not
/// `w` calendar days: across a hole in the record it reaches past it. A window
/// with no more than `min_frac·w` values present is `None`, not a mean of
/// whatever was there; across the record's 2017 gap that would be a mean of
/// one side.
pub fn centred_mean(vals: &[Option<f64>], w: usize, min_frac: f64, out: &mut [Option<f64>]) {
    let (half, n) = (w / 2, vals.len());
    for (i, o) in out.iter_mut().enumerate().take(n) {
        let (mut s, mut k) = (0.0, 0usize);
        for v in vals[i.saturating_sub(half)..n.min(i + half + 1)]
            .iter()
            .flatten()
        {
            s += v;
            k += 1;
        }
        *o = (k as f64 > w as f64 * min_frac).then(|| s / k as f64);
    }
}

/// The correlation of a series with itself `lag` values on, and how many
/// pairs it rests on.
///
/// Pairs are `x[i]` with `x[i + lag]` where both are present — a gap is
/// skipped, never read as zero, because counting 273 missing days as no
/// departure would pull every correlation toward the mean. The count comes
/// back because [`bartlett_halfwidth`] needs it, and a correlation that has
/// dropped its `n` leaves the band to guess the number that sets its width.
/// The correlation is `None` below three pairs or where either side does not
/// vary.
pub fn lagged_corr(x: &[Option<f64>], lag: usize) -> (Option<f64>, usize) {
    let pairs = || {
        x.iter()
            .zip(x.iter().skip(lag))
            .filter_map(|(a, b)| Some(((*a)?, (*b)?)))
    };
    let (mut sa, mut sb, mut k) = (0.0, 0.0, 0usize);
    for (a, b) in pairs() {
        sa += a;
        sb += b;
        k += 1;
    }
    if k < 3 {
        return (None, k);
    }
    let (ma, mb) = (sa / k as f64, sb / k as f64);
    let (mut sab, mut saa, mut sbb) = (0.0, 0.0, 0.0);
    for (a, b) in pairs() {
        let (da, db) = (a - ma, b - mb);
        sab += da * db;
        saa += da * da;
        sbb += db * db;
    }
    let r = (saa != 0.0 && sbb != 0.0).then(|| sab / pmath::sqrt(saa * sbb));
    (r, k)
}

/// Bartlett's 95 per cent half-width for an autocorrelation at one lag, from
/// the sum of the squared correlations at every shorter lag and the pairs at
/// this one: `1.96·sqrt((1 + 2·Σ r_j²) / n)`.
///
/// Not `2/sqrt(n)`. That band tests the whole series against white noise, and
/// a series that decays from 0.94 at lag 1 passes it everywhere; Bartlett's
/// asks whether a bump is more than the decay below it already produces, so it
/// widens with lag. `None` at two pairs or fewer.
pub fn bartlett_halfwidth(sum_sq_below: f64, n: usize) -> Option<f64> {
    (n > 2).then(|| 1.96 * pmath::sqrt((1.0 + 2.0 * sum_sq_below) / n as f64))
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

    #[test]
    fn a_scatter_is_the_population_one() {
        // The textbook eight: mean 5, population sd 2.
        assert_eq!(
            mean_sd(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]),
            Some((5.0, 2.0))
        );
        assert_eq!(mean_sd(&[]), None);
    }

    #[test]
    fn a_centred_mean_counts_what_is_there_and_refuses_what_is_not() {
        let v = [Some(1.0), Some(2.0), None, Some(4.0), Some(5.0)];
        let mut out = [None; 5];
        // Three wide, one either side, and more than 1.5 values needed: every
        // window holds two or three.
        centred_mean(&v, 3, 0.5, &mut out);
        assert_eq!(out, [Some(1.5), Some(1.5), Some(3.0), Some(4.5), Some(4.5)]);
        // More than 2.1 needed, and no window holds three present values.
        centred_mean(&v, 3, 0.7, &mut out);
        assert_eq!(out, [None; 5]);
    }

    #[test]
    fn a_lagged_correlation_skips_the_gaps_and_says_how_many_pairs() {
        // (1,2) (2,3) (3,4) (4,5): the pairs touching the gap are dropped.
        let v = [
            Some(1.0),
            Some(2.0),
            Some(3.0),
            Some(4.0),
            Some(5.0),
            None,
            Some(7.0),
        ];
        let (r, n) = lagged_corr(&v, 1);
        assert_eq!(n, 4);
        assert!((r.unwrap() - 1.0).abs() < 1e-12);
        // A worked case: lag 1 of 1 3 2 5 4 pairs a = 1 3 2 5 with b = 3 2 5 4;
        // Σdadb = 0.5, Σda² = 8.75, Σdb² = 5, so r = 0.5/√43.75.
        let w = [Some(1.0), Some(3.0), Some(2.0), Some(5.0), Some(4.0)];
        let (r, n) = lagged_corr(&w, 1);
        assert_eq!(n, 4);
        assert!((r.unwrap() - 0.07559289460184544).abs() < 1e-15);
        // Two pairs is not a measurement; a constant does not correlate.
        assert_eq!(lagged_corr(&w, 3), (None, 2));
        assert_eq!(lagged_corr(&[Some(3.0); 6], 1), (None, 5));
    }

    #[test]
    fn bartletts_band_widens_with_what_came_before() {
        // 1.96·√(1/100), and 1.96·√((1 + 2·0.5)/4).
        assert!((bartlett_halfwidth(0.0, 100).unwrap() - 0.196).abs() < 1e-15);
        assert!((bartlett_halfwidth(0.5, 4).unwrap() - 1.3859292911256331).abs() < 1e-15);
        assert_eq!(bartlett_halfwidth(0.0, 2), None);
    }
}
