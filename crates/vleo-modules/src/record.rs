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
use vleo_core::physics::env::ap_at_kp;
use vleo_data::{SolarCycle, SolarDay};

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

/// The storm levels the storm-scale figure counts, by name and by the `Kp`
/// that opens each: NOAA's G1 to G3. Their daily-Ap thresholds are the
/// published table's `ap` at that `Kp` ([`ap_at_kp`]), 48, 80 and 132.
pub const STORM_LEVELS: [(&str, f64); 3] = [("G1", 5.0), ("G2", 6.0), ("G3", 7.0)];

/// The storm-scale figure's numbers: how storms are shared out between the
/// cycles the record holds.
#[derive(Clone, Debug, PartialEq)]
pub struct StormScale {
    /// Each level's daily-Ap threshold, in [`STORM_LEVELS`] order.
    pub level_ap: Vec<f64>,
    /// The cycles, in the table's order.
    pub cycles: Vec<u32>,
    /// Days in each cycle that carry an Ap.
    pub days: Vec<usize>,
    /// The largest daily Ap in each cycle; none for a cycle with no days.
    pub max_ap: Vec<Option<f64>>,
    /// Days a year at or above each level, per cycle: `[level][cycle]`. Per
    /// year of the cycle rather than per cycle, because the record holds all
    /// of two cycles and six years of the third.
    pub per_year: Vec<Vec<Option<f64>>>,
    /// The busiest cycle's rate over the quietest's, at G1, among the cycles
    /// that reach it; none unless two do.
    pub evenness: Option<f64>,
    /// The cycle with the most days a year at each level; the first on a tie,
    /// none where no cycle has a rate.
    pub busiest: Vec<Option<u32>>,
}

/// The storm-scale figure's numbers, from the record and its cycles.
///
/// A day belongs to the first cycle whose `[start, end)` holds it, as the
/// page assigns it.
pub fn storm_scale(record: &[SolarDay], cycles: &[SolarCycle]) -> StormScale {
    let mut per: Vec<Vec<f64>> = cycles.iter().map(|_| Vec::new()).collect();
    for d in record {
        let Some(ap) = d.ap else { continue };
        if let Some(i) = cycles
            .iter()
            .position(|c| d.day >= c.start && d.day < c.end)
        {
            per[i].push(ap);
        }
    }
    let level_ap: Vec<f64> = STORM_LEVELS
        .iter()
        .map(|(_, kp)| ap_at_kp(*kp).expect("G1 to G3 are on the Kp scale"))
        .collect();
    let per_year: Vec<Vec<Option<f64>>> = level_ap
        .iter()
        .map(|&thr| {
            per.iter()
                .map(|v| {
                    (!v.is_empty()).then(|| {
                        v.iter().filter(|&&a| a >= thr).count() as f64 / (v.len() as f64 / 365.25)
                    })
                })
                .collect()
        })
        .collect();
    let g1: Vec<f64> = per_year[0]
        .iter()
        .flatten()
        .copied()
        .filter(|v| v.is_finite() && *v > 0.0)
        .collect();
    let evenness = (g1.len() > 1).then(|| {
        g1.iter().copied().fold(f64::MIN, f64::max) / g1.iter().copied().fold(f64::MAX, f64::min)
    });
    let busiest = per_year
        .iter()
        .map(|rates| {
            let mut best: Option<usize> = None;
            for (i, v) in rates.iter().enumerate() {
                let Some(v) = v.filter(|v| v.is_finite()) else {
                    continue;
                };
                if best.is_none_or(|b| v > rates[b].unwrap_or(f64::NAN)) {
                    best = Some(i);
                }
            }
            best.map(|i| cycles[i].n)
        })
        .collect();
    StormScale {
        level_ap,
        cycles: cycles.iter().map(|c| c.n).collect(),
        days: per.iter().map(Vec::len).collect(),
        max_ap: per
            .iter()
            .map(|v| v.iter().copied().reduce(f64::max))
            .collect(),
        per_year,
        evenness,
        busiest,
    }
}

/// The Kp-against-Ap figure's numbers: the daily Ap the record holds at each
/// worst three-hourly `Kp`, beside the published table's `ap` at that `Kp`.
#[derive(Clone, Debug, PartialEq)]
pub struct KpAp {
    /// Every worst-slot `Kp` the record holds, ascending.
    pub kp: Vec<f64>,
    /// At each: the median, 10th and 90th percentile of the day's Ap.
    pub median: Vec<Option<f64>>,
    pub p10: Vec<Option<f64>>,
    pub p90: Vec<Option<f64>>,
    /// The published three-hourly `ap` at each `Kp` ([`ap_at_kp`]).
    pub table: Vec<Option<f64>>,
    /// At how many of those `Kp` the table sits above the median day, and
    /// above the 90th percentile.
    pub above_median: u32,
    pub above_p90: u32,
    /// The table and the median day at `Kp` 7, where the figure states the
    /// bias; none where the record has no such day.
    pub kp7_table: Option<f64>,
    pub kp7_median: Option<f64>,
}

/// The Kp-against-Ap figure's numbers, from the record.
pub fn kp_ap(record: &[SolarDay]) -> KpAp {
    let mut per: Vec<(f64, Vec<f64>)> = Vec::new();
    for d in record {
        let (Some(ap), Some(kp)) = (d.ap, d.kp_max) else {
            continue;
        };
        match per.iter_mut().find(|(k, _)| *k == kp) {
            Some((_, v)) => v.push(ap),
            None => per.push((kp, alloc::vec![ap])),
        }
    }
    per.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (_, v) in per.iter_mut() {
        v.sort_by(f64::total_cmp);
    }
    let at = |q: f64| -> Vec<Option<f64>> { per.iter().map(|(_, v)| quantile(v, q)).collect() };
    let (median, p10, p90) = (at(0.5), at(0.1), at(0.9));
    let kp: Vec<f64> = per.iter().map(|p| p.0).collect();
    let table: Vec<Option<f64>> = kp.iter().map(|&k| ap_at_kp(k)).collect();
    let above = |of: &[Option<f64>]| {
        table
            .iter()
            .zip(of)
            .filter(|(t, m)| matches!((t, m), (Some(t), Some(m)) if t > m))
            .count() as u32
    };
    let i7 = kp.iter().position(|&k| k == 7.0);
    KpAp {
        above_median: above(&median),
        above_p90: above(&p90),
        kp7_table: ap_at_kp(7.0),
        kp7_median: i7.and_then(|i| median[i]),
        kp,
        median,
        p10,
        p90,
        table,
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

    fn cycle(n: u32, start: i32, end: i32) -> SolarCycle {
        SolarCycle { n, start, end }
    }

    #[test]
    fn storms_are_counted_per_year_of_each_cycle() {
        // Cycle 1 is days 0-9, one of them without an Ap; cycle 2 is days
        // 10-14; cycle 3 holds no day; day 20 is in no cycle and its Ap of 300
        // must count nowhere.
        let aps = [
            Some(50.0),
            Some(90.0),
            Some(140.0),
            Some(5.0),
            Some(5.0),
            Some(5.0),
            Some(5.0),
            Some(5.0),
            Some(5.0),
            None,
            Some(50.0),
            Some(5.0),
            Some(5.0),
            Some(5.0),
            Some(5.0),
        ];
        let mut rec: Vec<SolarDay> = aps
            .iter()
            .enumerate()
            .map(|(i, a)| day(i as i32, None, *a))
            .collect();
        rec.push(day(20, None, Some(300.0)));
        let cycles = [cycle(1, 0, 10), cycle(2, 10, 15), cycle(3, 100, 200)];
        let s = storm_scale(&rec, &cycles);
        assert_eq!(s.level_ap, [48.0, 80.0, 132.0]);
        assert_eq!(s.days, [9, 5, 0]);
        assert_eq!(s.max_ap, [Some(140.0), Some(50.0), None]);
        // Cycle 1: three of nine days at G1, two at G2, one at G3; cycle 2:
        // one of five at G1. Per year: count ÷ (days ÷ 365.25).
        let y = |c: f64, n: f64| Some(c / (n / 365.25));
        assert_eq!(s.per_year[0], [y(3.0, 9.0), y(1.0, 5.0), None]);
        assert_eq!(s.per_year[1], [y(2.0, 9.0), y(0.0, 5.0), None]);
        assert_eq!(s.per_year[2], [y(1.0, 9.0), y(0.0, 5.0), None]);
        // (3/9) ÷ (1/5) = 5/3 at G1, and cycle 1 is busiest at every level.
        assert!((s.evenness.unwrap() - 5.0 / 3.0).abs() < 1e-12);
        assert_eq!(s.busiest, [Some(1), Some(1), Some(1)]);
    }

    #[test]
    fn a_kp_is_set_against_the_days_that_reached_it() {
        let d = |n: i32, kp: Option<f64>, ap: Option<f64>| SolarDay {
            kp_max: kp,
            ..day(n, None, ap)
        };
        let rec = [
            d(0, Some(7.0), Some(60.0)),
            d(1, Some(1.33), Some(6.0)),
            d(2, Some(7.0), Some(40.0)),
            d(3, Some(9.5), Some(500.0)),
            d(4, Some(1.33), Some(4.0)),
            d(5, Some(7.0), Some(50.0)),
            d(6, Some(4.0), None),
        ];
        let k = kp_ap(&rec);
        assert_eq!(k.kp, [1.33, 7.0, 9.5]);
        // Kp 1.33: Ap 4, 6. Kp 7: Ap 40, 50, 60. Percentiles interpolate at
        // h = (n−1)q: 4.2 and 5.8; 42 and 58.
        assert_eq!(k.median, [Some(5.0), Some(50.0), Some(500.0)]);
        let near = |a: &[Option<f64>], b: [f64; 3]| {
            a.iter().zip(b).all(|(x, y)| (x.unwrap() - y).abs() < 1e-12)
        };
        assert!(near(&k.p10, [4.2, 42.0, 500.0]), "{:?}", k.p10);
        assert!(near(&k.p90, [5.8, 58.0, 500.0]), "{:?}", k.p90);
        // The table: 5 at Kp 1⅓, 132 at 7, nothing off the scale at 9.5.
        assert_eq!(k.table, [Some(5.0), Some(132.0), None]);
        // Above the median only at Kp 7 (5 is not above 5), and above the 90th
        // there too.
        assert_eq!((k.above_median, k.above_p90), (1, 1));
        assert_eq!((k.kp7_table, k.kp7_median), (Some(132.0), Some(50.0)));
        assert_eq!(kp_ap(&[]).kp7_median, None);
    }

    #[test]
    fn a_record_with_nothing_to_pair_says_so() {
        let d = density(&[day(0, Some(70.0), None), day(1, None, Some(3.0))]);
        assert_eq!(d.days, 0);
        assert_eq!((d.r, d.median_f107, d.median_ap), (None, None, None));
        assert_eq!((d.below_both_pct, d.storm_deciles), (0.0, 0));
    }
}
