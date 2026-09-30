//! The numbers the record's figures draw, worked out by the engine.
//!
//! The solar panels drew the record and worked out their own statistics in the
//! page (`web/js/solar.js`). A number worked out in the page is a number only
//! the page can give: no test holds it, and Python or the command line cannot
//! ask for it. Phase 10 moves them here, one panel at a time — the estimators
//! into the kernel (`vleo_core::math::stats`), the reading of the record into
//! this — and the page asks and draws. The page still draws the record's own
//! days as its dots: drawing the data is drawing, and it is the same bytes the
//! engine reads.
//!
//! Each figure's numbers are the page's to the last convention — the same
//! days, the same order, the same percentile — so the picture a person
//! approved is the picture it still draws.

use alloc::vec::Vec;
use vleo_core::math::{pearson, quantile};
use vleo_data::SolarDay;

/// The storm-level daily Ap the density figure counts: G1, the level the
/// panels call a storm.
pub const STORM_AP: f64 = 26.0;

/// The density figure's numbers: how the two drivers a density model takes
/// sit against each other over the record.
#[derive(Clone, Debug, PartialEq)]
pub struct Density {
    /// Days that carry both an F10.7 and an Ap.
    pub days: usize,
    /// Pearson's r of F10.7 against Ap over those days.
    pub r: Option<f64>,
    pub median_f107: Option<f64>,
    pub median_ap: Option<f64>,
    /// The share of those days below BOTH medians, in per cent. Two
    /// independent drivers put a quarter of their days there.
    pub below_both_pct: f64,
    /// How many of the ten F10.7 deciles hold a day at storm-level Ap.
    pub storm_deciles: u32,
}

/// The density figure's numbers, from the record.
pub fn density(record: &[SolarDay]) -> Density {
    let both: Vec<(f64, f64)> = record
        .iter()
        .filter_map(|d| Some((d.f107?, d.ap?)))
        .collect();
    let f: Vec<f64> = both.iter().map(|p| p.0).collect();
    let a: Vec<f64> = both.iter().map(|p| p.1).collect();
    let r = pearson(&f, &a);
    let sorted = |v: &[f64]| {
        let mut s = v.to_vec();
        s.sort_by(|x, y| x.partial_cmp(y).unwrap_or(core::cmp::Ordering::Equal));
        s
    };
    let (fs, as_) = (sorted(&f), sorted(&a));
    let (mf, ma) = (quantile(&fs, 0.5), quantile(&as_, 0.5));
    let below = match (mf, ma) {
        (Some(mf), Some(ma)) => both.iter().filter(|(x, y)| *x < mf && *y < ma).count(),
        _ => 0,
    };
    let below_both_pct = if both.is_empty() {
        0.0
    } else {
        100.0 * below as f64 / both.len() as f64
    };
    // Whether the tail is everywhere along the flux axis, counted by decile
    // rather than seen in a cloud of ten thousand dots. The last decile is
    // open above, so the highest flux day is in it.
    let edge = |k: u32| quantile(&fs, k as f64 / 10.0);
    let mut storm_deciles = 0;
    for k in 0..10 {
        let (Some(lo), hi) = (
            edge(k),
            if k == 9 {
                Some(f64::INFINITY)
            } else {
                edge(k + 1)
            },
        ) else {
            continue;
        };
        let Some(hi) = hi else { continue };
        if both
            .iter()
            .any(|(x, y)| *x >= lo && *x < hi && *y >= STORM_AP)
        {
            storm_deciles += 1;
        }
    }
    Density {
        days: both.len(),
        r,
        median_f107: mf,
        median_ap: ma,
        below_both_pct,
        storm_deciles,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(n: i32, f107: Option<f64>, ap: Option<f64>) -> SolarDay {
        SolarDay {
            day: n,
            f107,
            ap,
            kp: [None; 8],
            kp_max: None,
        }
    }

    #[test]
    fn the_density_numbers_are_the_pages() {
        // Ten days, one without an Ap: the nine with both are what count.
        let rec: Vec<SolarDay> = (0..10)
            .map(|i| {
                let ap = if i == 4 {
                    None
                } else {
                    Some([3.0, 5.0, 30.0, 8.0, 0.0, 12.0, 4.0, 40.0, 7.0, 6.0][i as usize])
                };
                day(i, Some(70.0 + 10.0 * i as f64), ap)
            })
            .collect();
        let d = density(&rec);
        assert_eq!(d.days, 9);
        // F10.7 of the nine: 70 80 90 100 120 130 140 150 160; median 120.
        assert_eq!(d.median_f107, Some(120.0));
        // Ap of the nine sorted: 3 4 5 6 7 8 12 30 40; median 7.
        assert_eq!(d.median_ap, Some(7.0));
        // Below both (F < 120 and Ap < 7): (70,3) (80,5) — two of nine.
        assert!((d.below_both_pct - 200.0 / 9.0).abs() < 1e-12);
        // Storm-level Ap (≥ 26) on the days at F 90 and 140. With nine sorted
        // fluxes the decile edges (h = 8·k/10) fall at 70, 78, 86, 94, 104,
        // 120, 128, 136, 144, 152: 90 is in [86, 94) and 140 in [136, 144) —
        // two deciles.
        assert_eq!(d.storm_deciles, 2);
        assert!(d.r.is_some());
    }

    #[test]
    fn a_record_with_nothing_to_pair_says_so() {
        let d = density(&[day(0, Some(70.0), None), day(1, None, Some(3.0))]);
        assert_eq!(d.days, 0);
        assert_eq!((d.r, d.median_f107, d.median_ap), (None, None, None));
        assert_eq!((d.below_both_pct, d.storm_deciles), (0.0, 0));
    }
}
