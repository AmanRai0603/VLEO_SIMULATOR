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
use vleo_core::math::{bartlett_halfwidth, centred_mean, lagged_corr, mean_sd, pearson, quantile};
use vleo_core::physics::env::ap_at_kp;
use vleo_core::units::pmath;
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

/// Each day's phase in its cycle: 0 at the opening minimum, 1 at the next.
///
/// A day in no cycle has none. A cycle something else opens after is
/// complete and folds by its own length; the one still running folds by the
/// MEAN length of the complete ones, because the "end" the table gives it is
/// where the record stops — dividing by that stretched cycle 25 across the
/// whole axis by 1.894. A day more than one mean length into the running
/// cycle belongs to a cycle the record cannot name and has no phase, not one
/// wrapped round. This is `sw_cycle_phase`'s fold, day by day.
pub fn cycle_phase(record: &[SolarDay], cycles: &[SolarCycle]) -> Vec<Option<f64>> {
    let done = |c: &SolarCycle| cycles.iter().any(|o| o.start >= c.end);
    let complete: Vec<&SolarCycle> = cycles.iter().filter(|c| done(c)).collect();
    let mean_len = (!complete.is_empty()).then(|| {
        complete
            .iter()
            .map(|c| (c.end - c.start) as f64)
            .sum::<f64>()
            / complete.len() as f64
    });
    record
        .iter()
        .map(|d| {
            let c = cycles.iter().find(|c| d.day >= c.start && d.day < c.end)?;
            let len = if done(c) {
                Some((c.end - c.start) as f64)
            } else {
                mean_len
            };
            let ph = (d.day - c.start) as f64 / len.filter(|l| *l != 0.0)?;
            (ph <= 1.0).then_some(ph)
        })
        .collect()
}

/// The driver a figure of the record is drawn over.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Driver {
    F107,
    Ap,
    /// The sunspot number.
    Ssn,
}

impl Driver {
    fn of(self, d: &SolarDay) -> Option<f64> {
        match self {
            Driver::F107 => d.f107,
            Driver::Ap => d.ap,
            Driver::Ssn => d.ssn,
        }
    }
}

/// The longest lag the recurrence figure reaches, in days.
pub const RECURRENCE_MAX_LAG: usize = 200;

/// The three detrend windows the recurrence figure compares, in days; the
/// first is the one `sw_recurrence_lag` and `sw_recurrence_strength` were
/// measured under, and the band and the peaks are read off it.
pub const RECURRENCE_WINDOWS: [usize; 3] = [365, 181, 731];

/// The recurrence figure's numbers: the autocorrelation of the detrended
/// record against lag.
#[derive(Clone, Debug, PartialEq)]
pub struct Recurrence {
    /// The autocorrelation at lags 1 to [`RECURRENCE_MAX_LAG`], one curve per
    /// window of [`RECURRENCE_WINDOWS`], in that order.
    pub r: [Vec<Option<f64>>; 3],
    /// The pairs behind the first window's correlation at lag 1.
    pub n_lag1: usize,
    /// Bartlett's 95 per cent half-width on the first curve, at every lag.
    pub band: Vec<Option<f64>>,
    /// At how many lags the first curve is outside that band.
    pub outside: u32,
    /// The highest point of the first curve in each of the rotation's first
    /// three bands, lags 18–36, 45–65 and 72–95: `(lag, r)`, the lowest lag
    /// on a tie.
    pub peaks: [Option<(usize, f64)>; 3],
    /// At how many lags the longest window's curve sits above the first's,
    /// and at how many both are drawn.
    pub above_long: u32,
    pub compared: u32,
}

/// The recurrence figure's numbers, over one driver.
///
/// The detrend is a centred mean over each window with 0.6 of it present —
/// the completeness rule the rows were measured under.
pub fn recurrence(record: &[SolarDay], driver: Driver) -> Recurrence {
    let v: Vec<Option<f64>> = record.iter().map(|d| driver.of(d)).collect();
    let mut trend = alloc::vec![None; v.len()];
    let mut acf = |w: usize| {
        centred_mean(&v, w, 0.6, &mut trend);
        let res: Vec<Option<f64>> = v
            .iter()
            .zip(&trend)
            .map(|(x, t)| Some((*x)? - (*t)?))
            .collect();
        (1..=RECURRENCE_MAX_LAG)
            .map(|lag| lagged_corr(&res, lag))
            .collect::<Vec<_>>()
    };
    let a = acf(RECURRENCE_WINDOWS[0]);
    let short = acf(RECURRENCE_WINDOWS[1]);
    let long = acf(RECURRENCE_WINDOWS[2]);
    let (mut acc, mut outside) = (0.0, 0u32);
    let band = a
        .iter()
        .map(|&(r, n)| {
            let b = bartlett_halfwidth(acc, n);
            if let (Some(r), Some(b)) = (r, b) {
                if r.abs() > b {
                    outside += 1;
                }
            }
            if let Some(r) = r {
                acc += r * r;
            }
            b
        })
        .collect();
    let peak = |lo: usize, hi: usize| {
        let mut best: Option<(usize, f64)> = None;
        for lag in lo..=hi.min(RECURRENCE_MAX_LAG) {
            if let Some(r) = a[lag - 1].0 {
                if best.is_none_or(|(_, b)| r > b) {
                    best = Some((lag, r));
                }
            }
        }
        best
    };
    let (mut above_long, mut compared) = (0u32, 0u32);
    for (lo, hi) in a.iter().zip(&long) {
        let (Some(lo), Some(hi)) = (lo.0, hi.0) else {
            continue;
        };
        if !lo.is_finite() || !hi.is_finite() {
            continue;
        }
        compared += 1;
        if hi > lo {
            above_long += 1;
        }
    }
    let curve = |c: &[(Option<f64>, usize)]| c.iter().map(|p| p.0).collect::<Vec<_>>();
    Recurrence {
        r: [curve(&a), curve(&short), curve(&long)],
        n_lag1: a[0].1,
        band,
        outside,
        peaks: [peak(18, 36), peak(45, 65), peak(72, 95)],
        above_long,
        compared,
    }
}

/// How many phase bins the spike figure counts in.
pub const SPIKE_BINS: usize = 20;

/// The spike figure's numbers: which F10.7 days the smooth cycle does not
/// explain, and where in the cycle they fall.
#[derive(Clone, Debug, PartialEq)]
pub struct Spikes {
    /// The ratio to the 81-day centred mean at which a day is a spike: the
    /// ratio's mean plus 2.5 of its standard deviations, over the record.
    pub threshold: Option<f64>,
    /// Spike days that have a phase, and the separate bursts they fall in —
    /// a burst being a run of spike days on consecutive calendar days.
    pub days: u32,
    pub bursts: u32,
    /// The centre of each of [`SPIKE_BINS`] phase bins, and the spike days
    /// per thousand days at that phase; none for a bin the record never
    /// reaches.
    pub phase: Vec<f64>,
    pub rate: Vec<Option<f64>>,
}

/// The spike figure's numbers, from the record and its cycles.
///
/// The baseline is the 81-day centred mean with 0.7 of it present, the
/// threshold 2.5 standard deviations above the ratio's mean — what
/// `sw_spike_threshold` declares.
pub fn spikes(record: &[SolarDay], cycles: &[SolarCycle]) -> Spikes {
    let v: Vec<Option<f64>> = record.iter().map(|d| d.f107).collect();
    let mut base = alloc::vec![None; v.len()];
    centred_mean(&v, 81, 0.7, &mut base);
    let ratio: Vec<Option<f64>> = v
        .iter()
        .zip(&base)
        .map(|(x, b)| Some((*x)? / (*b)?))
        .collect();
    let present: Vec<f64> = ratio.iter().flatten().copied().collect();
    let threshold = mean_sd(&present).map(|(mu, sd)| mu + 2.5 * sd);
    let phase = cycle_phase(record, cycles);
    let (mut by_phase, mut all_phase) = ([0u32; SPIKE_BINS], [0u32; SPIKE_BINS]);
    let (mut days, mut bursts, mut prev) = (0u32, 0u32, None::<i32>);
    for ((d, r), ph) in record.iter().zip(&ratio).zip(&phase) {
        let (Some(r), Some(ph)) = (r, ph) else {
            continue;
        };
        let b = (pmath::floor(ph * SPIKE_BINS as f64) as usize).min(SPIKE_BINS - 1);
        all_phase[b] += 1;
        if threshold.is_some_and(|t| *r >= t) {
            by_phase[b] += 1;
            days += 1;
            if prev != Some(d.day - 1) {
                bursts += 1;
            }
            prev = Some(d.day);
        }
    }
    Spikes {
        threshold,
        days,
        bursts,
        phase: (0..SPIKE_BINS)
            .map(|i| (i as f64 + 0.5) / SPIKE_BINS as f64)
            .collect(),
        rate: by_phase
            .iter()
            .zip(&all_phase)
            .map(|(&c, &n)| (n > 0).then(|| 1000.0 * c as f64 / n as f64))
            .collect(),
    }
}

/// The fewest days a phase bin must hold in BOTH compared cycles for the
/// repeatability correlation to use it. A bin thinned by the 2017 gap — 141
/// days against the usual 201 — is a mean of a different thing, and
/// `sw_cycle_repeatability` is measured without it.
pub const MEAN_CYCLE_MIN_DAYS: usize = 150;

/// The mean-cycle figure's numbers: every cycle the record holds, stacked on
/// phase, the mean of the complete ones, and how well the last two complete
/// cycles repeat each other.
#[derive(Clone, Debug, PartialEq)]
pub struct MeanCycle {
    /// The centre of each phase bin.
    pub phase: Vec<f64>,
    /// The cycles in the table's order, and whether each is complete.
    pub cycles: Vec<u32>,
    pub complete: Vec<bool>,
    /// Each cycle's mean in each bin: `[cycle][bin]`; none for an empty bin.
    pub curves: Vec<Vec<Option<f64>>>,
    /// The mean over the complete cycles of their bin means.
    pub mean_cycle: Vec<Option<f64>>,
    /// The two cycles compared — the last two complete — earlier first.
    pub pair: Option<(u32, u32)>,
    /// Their correlation over the bins both fill with
    /// [`MEAN_CYCLE_MIN_DAYS`], and how many bins that is.
    pub r: Option<f64>,
    pub usable: u32,
    /// In how many of those bins the earlier cycle runs above the later.
    pub above: u32,
    /// The mean of earlier minus later over those bins, before phase 0.6 and
    /// from it.
    pub gap_rise: Option<f64>,
    pub gap_fall: Option<f64>,
    /// Each of the two cycles' largest bin mean, over every bin it fills.
    pub peak: Option<(f64, f64)>,
}

/// The mean-cycle figure's numbers, over one driver in `bins` phase bins; a
/// bin counts toward the comparison when both cycles hold `min_days` in it
/// ([`MEAN_CYCLE_MIN_DAYS`] for the figure).
pub fn mean_cycle(
    record: &[SolarDay],
    cycles: &[SolarCycle],
    driver: Driver,
    bins: usize,
    min_days: usize,
) -> MeanCycle {
    let phase = cycle_phase(record, cycles);
    let mut per: Vec<Vec<Vec<f64>>> = cycles
        .iter()
        .map(|_| alloc::vec![Vec::new(); bins])
        .collect();
    for (d, ph) in record.iter().zip(&phase) {
        let (Some(ph), Some(v)) = (ph, driver.of(d)) else {
            continue;
        };
        let Some(c) = cycles
            .iter()
            .position(|c| d.day >= c.start && d.day < c.end)
        else {
            continue;
        };
        let b = (pmath::floor(ph * bins as f64) as usize).min(bins - 1);
        per[c][b].push(v);
    }
    let mean = |a: &[f64]| (!a.is_empty()).then(|| a.iter().sum::<f64>() / a.len() as f64);
    let curves: Vec<Vec<Option<f64>>> = per
        .iter()
        .map(|c| c.iter().map(|b| mean(b)).collect())
        .collect();
    let complete: Vec<bool> = cycles
        .iter()
        .map(|c| cycles.iter().any(|o| o.start >= c.end))
        .collect();
    let mean_cycle = (0..bins)
        .map(|i| {
            let v: Vec<f64> = curves
                .iter()
                .zip(&complete)
                .filter(|(_, done)| **done)
                .filter_map(|(c, _)| c[i])
                .collect();
            mean(&v)
        })
        .collect();
    let phase_x: Vec<f64> = (0..bins).map(|i| (i as f64 + 0.5) / bins as f64).collect();
    let done: Vec<usize> = (0..cycles.len()).filter(|&i| complete[i]).collect();
    let (mut r, mut usable, mut above, mut gap_rise, mut gap_fall, mut peak) =
        (None, 0, 0, None, None, None);
    let pair = (done.len() >= 2).then(|| (done[done.len() - 2], done[done.len() - 1]));
    if let Some((i, j)) = pair {
        let ok = |k: usize| per[i][k].len() >= min_days && per[j][k].len() >= min_days;
        let both: Vec<(usize, f64, f64)> = (0..bins)
            .filter(|&k| ok(k))
            .filter_map(|k| Some((k, curves[i][k]?, curves[j][k]?)))
            .collect();
        let (a, b): (Vec<f64>, Vec<f64>) = both.iter().map(|&(_, x, y)| (x, y)).unzip();
        r = pearson(&a, &b);
        usable = both.len() as u32;
        above = both.iter().filter(|(_, x, y)| x > y).count() as u32;
        let gap = |lo: f64, hi: f64| {
            let d: Vec<f64> = both
                .iter()
                .filter(|(k, _, _)| phase_x[*k] >= lo && phase_x[*k] < hi)
                .map(|(_, x, y)| x - y)
                .collect();
            mean(&d)
        };
        gap_rise = gap(0.0, 0.6);
        gap_fall = gap(0.6, 1.0);
        let top = |c: usize| curves[c].iter().flatten().copied().reduce(f64::max);
        peak = top(i).zip(top(j));
    }
    MeanCycle {
        phase: phase_x,
        cycles: cycles.iter().map(|c| c.n).collect(),
        complete,
        curves,
        mean_cycle,
        pair: pair.map(|(i, j)| (cycles[i].n, cycles[j].n)),
        r,
        usable,
        above,
        gap_rise,
        gap_fall,
        peak,
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
            ssn: None,
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
    fn a_running_cycle_folds_by_the_mean_of_the_complete_ones() {
        // Cycle 1 is complete (cycle 2 opens where it ends) and ten days long;
        // cycle 2 is still running, and its table end of 30 is where the
        // record stops, so it folds by 10 too.
        let cycles = [cycle(1, 0, 10), cycle(2, 10, 30)];
        let rec: Vec<SolarDay> = [5, 10, 15, 19, 22, 40]
            .iter()
            .map(|&n| day(n, None, None))
            .collect();
        // Day 22 is 1.2 of a mean length in: a cycle nobody can name yet.
        assert_eq!(
            cycle_phase(&rec, &cycles),
            [Some(0.5), Some(0.0), Some(0.5), Some(0.9), None, None]
        );
    }

    #[test]
    fn a_27_day_recurrence_is_found_at_27_and_its_harmonics() {
        // A 27-day sawtooth, 2000 days long: whatever the detrend leaves, the
        // series repeats itself every 27 days and at nothing shorter.
        let rec: Vec<SolarDay> = (0..2000)
            .map(|i| day(i, Some(100.0 + (i % 27) as f64), None))
            .collect();
        let r = recurrence(&rec, Driver::F107);
        let lags = r.peaks.map(|p| p.map(|(lag, _)| lag));
        assert_eq!(lags, [Some(27), Some(54), Some(81)]);
        assert!(r.peaks[0].unwrap().1 > 0.99);
        assert!(r.r.iter().all(|c| c.len() == RECURRENCE_MAX_LAG));
        assert_eq!(r.band.len(), RECURRENCE_MAX_LAG);
        // The 365-day detrend needs more than 219 values in its window, so the
        // first and last 37 days have none: 1926 residuals, 1925 pairs at lag 1.
        assert_eq!(r.n_lag1, 1925);
        assert!(r.outside > 0 && r.compared as usize == RECURRENCE_MAX_LAG);
        // No Ap in this record: every correlation over Ap is refused.
        assert!(recurrence(&rec, Driver::Ap).r[0]
            .iter()
            .all(Option::is_none));
    }

    #[test]
    fn spikes_are_counted_in_bursts_and_per_thousand_days_of_phase() {
        // 300 days of a flat 100 sfu with 200 on days 100, 101 and 200. The
        // 81-day baseline needs 57 values, so days 16 to 283 have a ratio;
        // cycle 1 is complete at 300 days, so a phase bin is 15 days.
        let rec: Vec<SolarDay> = (0..300)
            .map(|i| {
                day(
                    i,
                    Some(if [100, 101, 200].contains(&i) {
                        200.0
                    } else {
                        100.0
                    }),
                    None,
                )
            })
            .collect();
        let s = spikes(&rec, &[cycle(1, 0, 300), cycle(2, 300, 600)]);
        let t = s.threshold.unwrap();
        assert!(t > 1.1 && t < 1.5, "{t}");
        // Three spike days, in two bursts: 100–101 and 200.
        assert_eq!((s.days, s.bursts), (3, 2));
        assert_eq!(s.phase.len(), SPIKE_BINS);
        assert_eq!(s.phase[0], 0.025);
        // Bin 0 has no day with a ratio, bin 1 fourteen (16–29), bin 6 two
        // spikes in fifteen days (90–104), bin 13 one in fifteen (195–209).
        assert_eq!(s.rate[0], None);
        assert_eq!(s.rate[1], Some(0.0));
        assert_eq!(s.rate[6], Some(1000.0 * 2.0 / 15.0));
        assert_eq!(s.rate[13], Some(1000.0 / 15.0));
        assert_eq!(s.rate[19], None);
    }

    #[test]
    fn cycles_are_stacked_on_phase_and_the_last_two_complete_compared() {
        // Cycles 1 and 2 are four days each and complete; cycle 3 is running
        // and folds by their mean, four. Four bins: one day in each.
        let v = [
            10.0, 20.0, 30.0, 40.0, 12.0, 14.0, 50.0, 60.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
        ];
        let rec: Vec<SolarDay> = v
            .iter()
            .enumerate()
            .map(|(i, &x)| day(i as i32, Some(x), None))
            .collect();
        let m = mean_cycle(
            &rec,
            &[cycle(1, 0, 4), cycle(2, 4, 8), cycle(3, 8, 30)],
            Driver::F107,
            4,
            1,
        );
        assert_eq!(m.phase, [0.125, 0.375, 0.625, 0.875]);
        assert_eq!(m.complete, [true, true, false]);
        assert_eq!(
            m.curves[0],
            [Some(10.0), Some(20.0), Some(30.0), Some(40.0)]
        );
        // Day 12 is exactly one mean length in, phase 1, into the last bin;
        // day 13 is past it and has no phase.
        assert_eq!(m.curves[2], [Some(1.0); 4]);
        assert_eq!(
            m.mean_cycle,
            [Some(11.0), Some(17.0), Some(40.0), Some(50.0)]
        );
        assert_eq!(m.pair, Some((1, 2)));
        // 10 20 30 40 against 12 14 50 60: Σdadb 900, Σda² 500, Σdb² 1816.
        assert!((m.r.unwrap() - 0.9444948303625054).abs() < 1e-15);
        assert_eq!((m.usable, m.above), (4, 1));
        // Before 0.6: (−2 + 6)/2; from it: (−20 − 20)/2.
        assert_eq!((m.gap_rise, m.gap_fall), (Some(2.0), Some(-20.0)));
        assert_eq!(m.peak, Some((40.0, 60.0)));
        // A bin either cycle holds fewer days in than asked for is left out.
        let thin = mean_cycle(
            &rec,
            &[cycle(1, 0, 4), cycle(2, 4, 8), cycle(3, 8, 30)],
            Driver::F107,
            4,
            2,
        );
        assert_eq!((thin.r, thin.usable), (None, 0));
    }

    #[test]
    fn a_record_with_nothing_to_pair_says_so() {
        let d = density(&[day(0, Some(70.0), None), day(1, None, Some(3.0))]);
        assert_eq!(d.days, 0);
        assert_eq!((d.r, d.median_f107, d.median_ap), (None, None, None));
        assert_eq!((d.below_both_pct, d.storm_deciles), (0.0, 0));
    }
}
