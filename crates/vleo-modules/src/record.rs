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
use vleo_data::{IssuedForecast, MonthlyMean, SolarCycle, SolarDay};

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
    /// The naive white-noise band on that correlation, ±2/√n — the band the
    /// figure explains it does not draw. None with no pairs.
    pub naive_band: Option<f64>,
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
        naive_band: (a[0].1 > 0).then(|| 2.0 / pmath::sqrt(a[0].1 as f64)),
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

/// Which band a day's value falls in, as the row that defines the bands
/// says: `sw_activity_band` for F10.7, `sw_regime` for Ap. Counted from 0.
/// `Err` is the row refusing the day, which is counted and never guessed.
fn band_of(driver: Driver, x: f64) -> Result<usize, ()> {
    use vleo_core::units::Ratio;
    use vleo_mod_solar::nodes::{sw_activity_band, sw_regime};
    let b = match driver {
        Driver::F107 => sw_activity_band::model::evaluate(Ratio::new(x)),
        Driver::Ap => sw_regime::model::evaluate(Ratio::new(x)),
        Driver::Ssn => return Err(()),
    }
    .map_err(|_| ())?;
    Ok(b.get() as usize - 1)
}

/// How many bands each row defines — its declared range, 1 to 4 for
/// `sw_activity_band` and 1 to 3 for `sw_regime`.
fn bands(driver: Driver) -> usize {
    match driver {
        Driver::F107 => 4,
        _ => 3,
    }
}

/// The segmentation figure's numbers: where the record sits, and how much of
/// it each band holds, with the bands the rows themselves assign.
#[derive(Clone, Debug, PartialEq)]
pub struct Segments {
    /// Days with a value, and how many of them the row refused to band.
    pub days: u32,
    pub refused: u32,
    /// The histogram: its bin width, and the days in each bin from 0.
    pub bin_width: f64,
    pub counts: Vec<u32>,
    /// The largest value in the record.
    pub max: Option<f64>,
    /// Days in each band, and their share of the days banded, in per cent.
    pub band_days: Vec<u32>,
    pub band_pct: Vec<f64>,
    /// Where the record crosses each cut between band k and k + 1: the
    /// largest value it holds in band k, and the smallest in band k + 1.
    pub below_cut: Vec<Option<f64>>,
    pub above_cut: Vec<Option<f64>>,
}

/// The segmentation figure's numbers over Ap or F10.7. Bins are 2 wide for
/// Ap and 5 sfu for F10.7, as the page drew them.
pub fn segments(record: &[SolarDay], driver: Driver) -> Segments {
    let vals: Vec<f64> = record.iter().filter_map(|d| driver.of(d)).collect();
    let bin_width = if driver == Driver::Ap { 2.0 } else { 5.0 };
    let max = vals.iter().copied().reduce(f64::max);
    let nb = max.map_or(0, |hi| pmath::ceil(hi / bin_width) as usize + 1);
    let mut counts = alloc::vec![0u32; nb];
    for x in &vals {
        counts[pmath::floor(x / bin_width) as usize] += 1;
    }
    let nbands = bands(driver);
    let (mut band_days, mut refused) = (alloc::vec![0u32; nbands], 0u32);
    let (mut lo, mut hi) = (
        alloc::vec![None::<f64>; nbands],
        alloc::vec![None::<f64>; nbands],
    );
    for &x in &vals {
        match band_of(driver, x) {
            Ok(k) if k < nbands => {
                band_days[k] += 1;
                lo[k] = Some(lo[k].map_or(x, |m: f64| m.min(x)));
                hi[k] = Some(hi[k].map_or(x, |m: f64| m.max(x)));
            }
            _ => refused += 1,
        }
    }
    let banded = vals.len() as u32 - refused;
    Segments {
        days: vals.len() as u32,
        refused,
        bin_width,
        counts,
        max,
        band_pct: band_days
            .iter()
            .map(|&n| 100.0 * n as f64 / banded as f64)
            .collect(),
        band_days,
        below_cut: hi[..nbands - 1].to_vec(),
        above_cut: lo[1..].to_vec(),
    }
}

/// The regime-by-phase figure's numbers: the share of days at each cycle
/// phase that `sw_regime` calls storm, and that it calls quiet.
#[derive(Clone, Debug, PartialEq)]
pub struct RegimePhase {
    pub phase: Vec<f64>,
    /// Per cent of the days in each bin; none for a bin the record never
    /// reaches.
    pub storm_pct: Vec<Option<f64>>,
    pub quiet_pct: Vec<Option<f64>>,
    /// The highest storm share and the phase it is at, the first on a tie;
    /// the lowest quiet share and its phase.
    pub storm_peak: Option<(f64, f64)>,
    pub quiet_low: Option<(f64, f64)>,
    /// How closely the two shares move against each other across the bins.
    pub mirror: Option<f64>,
}

/// The regime-by-phase figure's numbers, in 20 phase bins.
pub fn regime_phase(record: &[SolarDay], cycles: &[SolarCycle]) -> RegimePhase {
    const NB: usize = 20;
    let phase = cycle_phase(record, cycles);
    let (mut tot, mut storm, mut quiet) = ([0u32; NB], [0u32; NB], [0u32; NB]);
    for (d, ph) in record.iter().zip(&phase) {
        let (Some(ph), Some(ap)) = (ph, d.ap) else {
            continue;
        };
        let b = (pmath::floor(ph * NB as f64) as usize).min(NB - 1);
        tot[b] += 1;
        match band_of(Driver::Ap, ap) {
            Ok(2) => storm[b] += 1,
            Ok(0) => quiet[b] += 1,
            _ => {}
        }
    }
    let pct = |c: &[u32; NB]| -> Vec<Option<f64>> {
        (0..NB)
            .map(|i| (tot[i] > 0).then(|| 100.0 * c[i] as f64 / tot[i] as f64))
            .collect()
    };
    let (storm_pct, quiet_pct) = (pct(&storm), pct(&quiet));
    let x: Vec<f64> = (0..NB).map(|i| (i as f64 + 0.5) / NB as f64).collect();
    let extreme = |v: &[Option<f64>], high: bool| {
        let mut best: Option<(f64, f64)> = None;
        for (i, s) in v.iter().enumerate() {
            let Some(s) = s.filter(|s| s.is_finite()) else {
                continue;
            };
            if best.is_none_or(|(b, _)| if high { s > b } else { s < b }) {
                best = Some((s, x[i]));
            }
        }
        best
    };
    let (a, b): (Vec<f64>, Vec<f64>) = storm_pct
        .iter()
        .zip(&quiet_pct)
        .filter_map(|(s, q)| Some(((*s)?, (*q)?)))
        .unzip();
    RegimePhase {
        storm_peak: extreme(&storm_pct, true),
        quiet_low: extreme(&quiet_pct, false),
        mirror: pearson(&a, &b),
        phase: x,
        storm_pct,
        quiet_pct,
    }
}

/// How the climate figure groups the record's days.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grouping {
    /// By calendar year.
    Year,
    /// By day of the year, in bins of five days named by their last day.
    DayOfYear,
    /// By calendar month.
    Month,
}

/// The climate figure's numbers: the record's mean in each group, and how
/// far the groups spread about the record's own mean.
#[derive(Clone, Debug, PartialEq)]
pub struct Climate {
    /// Each group, in order, where it is drawn, its mean and the days it
    /// rests on. A month is drawn at `floor(k/12) + (k mod 12)/12` for
    /// `k = 12·year + month`, as the page drew it.
    pub x: Vec<f64>,
    pub mean: Vec<f64>,
    pub days_in: Vec<u32>,
    /// The key of each group — a year, a day-of-year bin, `12·year + month`.
    pub key: Vec<i64>,
    /// The mean over the record's DAYS, not over its groups: a year the
    /// record holds 357 days of is not weighted as a full one.
    pub overall: Option<f64>,
    pub days: u32,
    /// The highest and lowest group mean, and where each is drawn — the first
    /// on a tie.
    pub high: Option<(f64, f64)>,
    pub low: Option<(f64, f64)>,
    /// How many groups sit below the record's mean.
    pub below: u32,
}

/// The climate figure's numbers over one driver.
pub fn climate(record: &[SolarDay], driver: Driver, by: Grouping) -> Climate {
    let mut groups: alloc::collections::BTreeMap<i64, Vec<f64>> = Default::default();
    let mut all = Vec::new();
    for d in record {
        let Some(v) = driver.of(d) else { continue };
        let (y, m, _, doy) = vleo_data::civil_from_days(d.day);
        let k = match by {
            Grouping::Year => y as i64,
            Grouping::DayOfYear => (doy as i64 + 4) / 5 * 5,
            Grouping::Month => y as i64 * 12 + m as i64,
        };
        groups.entry(k).or_default().push(v);
        all.push(v);
    }
    let mean = |a: &[f64]| a.iter().sum::<f64>() / a.len() as f64;
    let key: Vec<i64> = groups.keys().copied().collect();
    let x: Vec<f64> = key
        .iter()
        .map(|&k| match by {
            Grouping::Month => pmath::floor(k as f64 / 12.0) + (k % 12) as f64 / 12.0,
            _ => k as f64,
        })
        .collect();
    let means: Vec<f64> = groups.values().map(|v| mean(v)).collect();
    let overall = (!all.is_empty()).then(|| mean(&all));
    let pick = |high: bool| {
        let mut best: Option<(f64, f64)> = None;
        for (i, &v) in means.iter().enumerate() {
            if best.is_none_or(|(b, _)| if high { v > b } else { v < b }) {
                best = Some((v, x[i]));
            }
        }
        best
    };
    Climate {
        below: overall.map_or(0, |o| means.iter().filter(|&&v| v < o).count() as u32),
        high: pick(true),
        low: pick(false),
        days_in: groups.values().map(|v| v.len() as u32).collect(),
        days: all.len() as u32,
        overall,
        mean: means,
        x,
        key,
    }
}

/// The smoother figure's numbers: each month's mean beside the 13-month
/// smoothed value the cycles are counted on, and what the smoothing removes.
#[derive(Clone, Debug, PartialEq)]
pub struct Smoother {
    /// Where each month is drawn, in years: days since 2000 over 365.25, plus
    /// 2000.
    pub x: Vec<f64>,
    pub raw: Vec<Option<f64>>,
    pub smooth: Vec<Option<f64>>,
    /// Months the smoother is undefined for, of all the months.
    pub missing: u32,
    pub months: u32,
    /// The range of the monthly means, and of the smoothed values, over the
    /// months carrying both; the rms of their difference; how often the
    /// monthly line crosses the smoother; its largest departure from it.
    pub raw_range: f64,
    pub smooth_range: f64,
    pub rms: f64,
    pub crossings: u32,
    pub max_departure: f64,
}

/// The smoother figure's numbers over one driver.
pub fn smoother(months: &[MonthlyMean], driver: Driver) -> Smoother {
    let (raw, smooth): (Vec<Option<f64>>, Vec<Option<f64>>) = months
        .iter()
        .map(|m| match driver {
            Driver::F107 => (m.f107_mean, m.f107_smooth),
            Driver::Ap => (m.ap_mean, m.ap_smooth),
            Driver::Ssn => (m.ssn_mean, m.ssn_smooth),
        })
        .unzip();
    let both: Vec<(f64, f64)> = raw
        .iter()
        .zip(&smooth)
        .filter_map(|(a, b)| Some(((*a)?, (*b)?)))
        .collect();
    let range = |v: &mut dyn Iterator<Item = f64>| {
        let (lo, hi) = v.fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), x| {
            (lo.min(x), hi.max(x))
        });
        if both.is_empty() {
            0.0
        } else {
            hi - lo
        }
    };
    let rms = if both.is_empty() {
        0.0
    } else {
        pmath::sqrt(both.iter().map(|(a, b)| (a - b) * (a - b)).sum::<f64>() / both.len() as f64)
    };
    let (mut crossings, mut max_departure, mut prev) = (0u32, 0.0f64, 0i8);
    for (a, b) in &both {
        let dv = a - b;
        if dv.abs() > max_departure {
            max_departure = dv.abs();
        }
        let sign = if dv > 0.0 {
            1
        } else if dv < 0.0 {
            -1
        } else {
            0
        };
        if sign != 0 && prev != 0 && sign != prev {
            crossings += 1;
        }
        if sign != 0 {
            prev = sign;
        }
    }
    Smoother {
        x: months
            .iter()
            .map(|m| m.day as f64 / 365.25 + 2000.0)
            .collect(),
        missing: smooth.iter().filter(|v| v.is_none()).count() as u32,
        months: months.len() as u32,
        raw_range: range(&mut both.iter().map(|p| p.0)),
        smooth_range: range(&mut both.iter().map(|p| p.1)),
        rms,
        crossings,
        max_departure,
        raw,
        smooth,
    }
}

/// The percentiles the growth figure draws, the published one (95th) third.
pub const GROWTH_PERCENTILES: [f64; 4] = [0.50, 0.90, 0.95, 0.99];

/// The longest lead the growth figure reaches, in days — fifteen years.
pub const GROWTH_MAX_LEAD: u32 = 5478;

/// The leads a growth curve is drawn at: from 30 days, each the one before
/// times `step` rounded to a whole day, up to `max`.
fn leads(step: f64, max: u32) -> Vec<u32> {
    let mut out = Vec::new();
    let mut l = 30u32;
    while l <= max {
        out.push(l);
        l = pmath::round(l as f64 * step) as u32;
    }
    out
}

/// The signed change in a driver over each lead, every pair of days `lead`
/// apart that both carry it, sorted: `(day, value)` in record order.
fn changes(by_day: &alloc::collections::BTreeMap<i32, f64>, lead: u32) -> Vec<f64> {
    let mut ch: Vec<f64> = by_day
        .iter()
        .filter_map(|(t, v)| Some(by_day.get(&(t + lead as i32))? - v))
        .collect();
    ch.sort_by(f64::total_cmp);
    ch
}

/// The growth figure's numbers: how far a driver moves over a lead, at four
/// percentiles, pooled over the whole record.
#[derive(Clone, Debug, PartialEq)]
pub struct Growth {
    /// Each lead, in years, and the pairs of days behind it.
    pub lead_years: Vec<f64>,
    pub pairs: Vec<u32>,
    /// The change at each of [`GROWTH_PERCENTILES`], at each lead:
    /// `[percentile][lead]`.
    pub change: Vec<Vec<Option<f64>>>,
    /// The lead nearest a year, where the row is read — the first on a tie.
    pub at_year: usize,
    /// The 95th's highest point between 3 and 6 years and lowest between 9
    /// and 12 — the eleven-year cycle showing through — and whether the one
    /// stands above the other.
    pub hump: Option<usize>,
    pub dip: Option<usize>,
    pub humped: bool,
}

/// The growth figure's numbers over one driver.
pub fn growth(record: &[SolarDay], driver: Driver) -> Growth {
    let by_day: alloc::collections::BTreeMap<i32, f64> = record
        .iter()
        .filter_map(|d| Some((d.day, driver.of(d)?)))
        .collect();
    let ls = leads(1.35, GROWTH_MAX_LEAD);
    let mut pairs = Vec::new();
    let mut change: Vec<Vec<Option<f64>>> = GROWTH_PERCENTILES.iter().map(|_| Vec::new()).collect();
    for &l in &ls {
        let ch = changes(&by_day, l);
        pairs.push(ch.len() as u32);
        for (k, q) in GROWTH_PERCENTILES.iter().enumerate() {
            change[k].push(quantile(&ch, *q));
        }
    }
    let x: Vec<f64> = ls.iter().map(|&l| l as f64 / 365.25).collect();
    let y95 = &change[2];
    let at_year = (0..x.len()).fold(0, |b, i| {
        if pmath::abs(x[i] - 1.0) < pmath::abs(x[b] - 1.0) {
            i
        } else {
            b
        }
    });
    let pick = |lo: f64, hi: f64, max: bool| {
        let mut best: Option<usize> = None;
        for i in 0..x.len() {
            let Some(v) = y95[i].filter(|v| v.is_finite()) else {
                continue;
            };
            if x[i] < lo || x[i] > hi {
                continue;
            }
            if best.is_none_or(|b| {
                let bv = y95[b].unwrap_or(f64::NAN);
                if max {
                    v > bv
                } else {
                    v < bv
                }
            }) {
                best = Some(i);
            }
        }
        best
    };
    let in_range = |lo: f64, hi: f64| -> Vec<f64> {
        (0..x.len())
            .filter(|&i| x[i] >= lo && x[i] <= hi)
            .filter_map(|i| y95[i])
            .collect()
    };
    let (mid, late) = (in_range(3.0, 6.0), in_range(9.0, 12.0));
    let top = |v: &[f64]| v.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    Growth {
        humped: !mid.is_empty() && !late.is_empty() && top(&mid) > top(&late),
        hump: pick(3.0, 6.0, true),
        dip: pick(9.0, 12.0, false),
        at_year,
        lead_years: x,
        pairs,
        change,
    }
}

/// The by-cycle growth figure's numbers: the 95th-percentile change over each
/// lead, one curve per cycle, pairs taken only within a cycle.
#[derive(Clone, Debug, PartialEq)]
pub struct GrowthByCycle {
    pub lead_years: Vec<f64>,
    pub cycles: Vec<u32>,
    /// `[cycle][lead]`; none at a lead with thirty pairs or fewer.
    pub change: Vec<Vec<Option<f64>>>,
    /// The longest lead every cycle reaches, in years, and the lowest and
    /// highest cycle there.
    pub shared: Option<f64>,
    pub spread: Option<(f64, f64)>,
}

/// The by-cycle growth figure's numbers over one driver, to five years.
pub fn growth_by_cycle(
    record: &[SolarDay],
    cycles: &[SolarCycle],
    driver: Driver,
) -> GrowthByCycle {
    let ls = leads(1.5, GROWTH_MAX_LEAD.min(1826));
    let change: Vec<Vec<Option<f64>>> = cycles
        .iter()
        .enumerate()
        .map(|(ci, _)| {
            let by_day: alloc::collections::BTreeMap<i32, f64> = record
                .iter()
                .filter(|d| {
                    cycles
                        .iter()
                        .position(|c| d.day >= c.start && d.day < c.end)
                        == Some(ci)
                })
                .filter_map(|d| Some((d.day, driver.of(d)?)))
                .collect();
            ls.iter()
                .map(|&l| {
                    let ch = changes(&by_day, l);
                    if ch.len() > 30 {
                        quantile(&ch, 0.95)
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect();
    let x: Vec<f64> = ls.iter().map(|&l| l as f64 / 365.25).collect();
    let (mut shared, mut spread) = (None, None);
    for i in (0..ls.len()).rev() {
        let vs: Vec<f64> = change
            .iter()
            .filter_map(|c| c[i].filter(|v| v.is_finite()))
            .collect();
        if vs.len() == cycles.len() {
            shared = Some(x[i]);
            spread = Some((
                vs.iter().copied().fold(f64::INFINITY, f64::min),
                vs.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            ));
            break;
        }
    }
    GrowthByCycle {
        lead_years: x,
        cycles: cycles.iter().map(|c| c.n).collect(),
        change,
        shared,
        spread,
    }
}

/// The furthest back persistence looks for an observation, in days.
pub const PERSISTENCE_REACH: i32 = 15;

/// The leads the by-lead forecast figure scores: the outlook's nominal span.
pub const FORECAST_LEADS: u32 = 27;

/// The lead band every year of the archive populates, and the fewest pairs a
/// year needs to be scored.
pub const FORECAST_YEAR_LEADS: (f64, f64) = (1.0, 14.0);
pub const FORECAST_YEAR_MIN_PAIRS: u32 = 200;

/// Where a gap between issues is counted as "30 or more".
pub const ISSUE_GAP_CAP: u32 = 30;

/// The running sums one score is made of: the forecast's errors over every
/// pair with an observation, and each persistence baseline's over the pairs it
/// reaches.
#[derive(Clone, Copy, Default)]
struct Sums {
    e2: f64,
    se: f64,
    n: u32,
    e2s: f64,
    p2s: f64,
    ns: u32,
    e2l: f64,
    p2l: f64,
    nl: u32,
}

impl Sums {
    fn add(&mut self, e: f64, obs: f64, strict: Option<f64>, leaky: Option<f64>) {
        self.e2 += e * e;
        self.se += e;
        self.n += 1;
        if let Some(p) = strict {
            self.e2s += e * e;
            self.p2s += (p - obs) * (p - obs);
            self.ns += 1;
        }
        if let Some(p) = leaky {
            self.e2l += e * e;
            self.p2l += (p - obs) * (p - obs);
            self.nl += 1;
        }
    }

    /// One minus the ratio of mean squared errors; none where the baseline
    /// reached no pair or never missed.
    fn skill(e2: f64, p2: f64, n: u32) -> Option<f64> {
        (n != 0 && p2 != 0.0).then(|| 1.0 - (e2 / n as f64) / (p2 / n as f64))
    }
}

/// The record's F10.7 by day, and whether a day is in the record at all —
/// persistence needs the second: an issue on a day the record lacks has no
/// baseline, even where the days before it do.
struct Observed<'a> {
    record: &'a [SolarDay],
    f107: alloc::collections::BTreeMap<i32, f64>,
}

impl<'a> Observed<'a> {
    fn new(record: &'a [SolarDay]) -> Self {
        let f107 = record
            .iter()
            .filter_map(|d| Some((d.day, d.f107?)))
            .collect();
        Observed { record, f107 }
    }

    /// Persistence at an issue: the latest observation from `first` days
    /// before it back to [`PERSISTENCE_REACH`]. `first = 1` is what a
    /// forecaster had; `first = 0` hands it the issue date itself, which most
    /// issues also forecast — the leak.
    fn persistence(&self, issue: i32, first: i32) -> Option<f64> {
        self.record.binary_search_by_key(&issue, |d| d.day).ok()?;
        (first..=PERSISTENCE_REACH).find_map(|b| self.f107.get(&(issue - b)).copied())
    }

    /// Every issued forecast with an observation on its target day, as
    /// `(forecast, error, observed, strict, leaky)`, in the file's order.
    fn scored<'f>(
        &'f self,
        issued: &'f [IssuedForecast],
    ) -> impl Iterator<Item = (&'f IssuedForecast, f64, f64, Option<f64>, Option<f64>)> + 'f {
        issued.iter().filter_map(move |r| {
            let f = r.f107?;
            let obs = *self.f107.get(&r.target)?;
            Some((
                r,
                f - obs,
                obs,
                self.persistence(r.issue, 1),
                self.persistence(r.issue, 0),
            ))
        })
    }
}

/// The by-lead forecast figure's numbers: the issued outlook scored against
/// what arrived at each lead from 1 to [`FORECAST_LEADS`].
#[derive(Clone, Debug, PartialEq)]
pub struct ForecastByLead {
    /// The leads with at least one scored pair, and the pairs behind each
    /// score: every pair with an observation, and those each baseline reaches.
    pub lead: Vec<u32>,
    pub pairs: Vec<u32>,
    pub pairs_strict: Vec<u32>,
    pub pairs_leaky: Vec<u32>,
    /// Skill against persistence from the day before the issue, and from the
    /// issue date itself.
    pub skill_strict: Vec<Option<f64>>,
    pub skill_leaky: Vec<Option<f64>>,
    /// Forecast minus observed, its mean and its root mean square.
    pub bias: Vec<f64>,
    pub rmse: Vec<f64>,
    /// Where the strict skill peaks — the first on a tie.
    pub peak: Option<usize>,
    /// The lead where the strict baseline misses the most pairs the forecast
    /// has — the first on a tie.
    pub widest: Option<u32>,
}

/// The by-lead forecast figure's numbers.
pub fn forecast_by_lead(record: &[SolarDay], issued: &[IssuedForecast]) -> ForecastByLead {
    let obs = Observed::new(record);
    let mut sums = [Sums::default(); FORECAST_LEADS as usize];
    for (r, e, o, ps, pl) in obs.scored(issued) {
        let Some(l) = r.lead else { continue };
        if (1..=FORECAST_LEADS).any(|k| l == k as f64) {
            sums[l as usize - 1].add(e, o, ps, pl);
        }
    }
    let mut out = ForecastByLead {
        lead: Vec::new(),
        pairs: Vec::new(),
        pairs_strict: Vec::new(),
        pairs_leaky: Vec::new(),
        skill_strict: Vec::new(),
        skill_leaky: Vec::new(),
        bias: Vec::new(),
        rmse: Vec::new(),
        peak: None,
        widest: None,
    };
    for (i, s) in sums.iter().enumerate() {
        if s.n == 0 {
            continue;
        }
        out.lead.push(i as u32 + 1);
        out.pairs.push(s.n);
        out.pairs_strict.push(s.ns);
        out.pairs_leaky.push(s.nl);
        out.bias.push(s.se / s.n as f64);
        out.rmse.push(pmath::sqrt(s.e2 / s.n as f64));
        out.skill_strict.push(Sums::skill(s.e2s, s.p2s, s.ns));
        out.skill_leaky.push(Sums::skill(s.e2l, s.p2l, s.nl));
    }
    for (i, v) in out.skill_strict.iter().enumerate() {
        let Some(v) = v else { continue };
        if out
            .peak
            .is_none_or(|b| *v > out.skill_strict[b].unwrap_or(f64::NAN))
        {
            out.peak = Some(i);
        }
    }
    let mut by = 0;
    for i in 0..out.lead.len() {
        let miss = out.pairs[i] - out.pairs_strict[i];
        if miss > by {
            by = miss;
            out.widest = Some(out.lead[i]);
        }
    }
    out
}

/// The by-year forecast figure's numbers: the outlook scored on leads
/// [`FORECAST_YEAR_LEADS`] in the calendar year it was issued in.
#[derive(Clone, Debug, PartialEq)]
pub struct ForecastByYear {
    /// Every year from the first issue to the last. A year with fewer than
    /// [`FORECAST_YEAR_MIN_PAIRS`] pairs, or none, is a hole: none in every
    /// series below, so a line breaks there rather than stepping over it.
    pub year: Vec<i32>,
    pub pairs: Vec<Option<u32>>,
    pub pairs_strict: Vec<Option<u32>>,
    pub pairs_leaky: Vec<Option<u32>>,
    pub skill_strict: Vec<Option<f64>>,
    pub skill_leaky: Vec<Option<f64>>,
    pub bias: Vec<Option<f64>>,
    pub rmse: Vec<Option<f64>>,
    /// The years that hold some pairs but too few, in order.
    pub thin: Vec<i32>,
    /// The scored years where the strict skill is lowest and highest, and
    /// the year the bias is lowest — each the first on a tie.
    pub worst: Option<i32>,
    pub best: Option<i32>,
    pub lowest_bias: Option<i32>,
}

/// The by-year forecast figure's numbers.
pub fn forecast_by_year(record: &[SolarDay], issued: &[IssuedForecast]) -> ForecastByYear {
    let obs = Observed::new(record);
    let (lo, hi) = FORECAST_YEAR_LEADS;
    let mut acc: alloc::collections::BTreeMap<i32, Sums> = alloc::collections::BTreeMap::new();
    for (r, e, o, ps, pl) in obs.scored(issued) {
        if !r.lead.is_some_and(|l| l >= lo && l <= hi) {
            continue;
        }
        let y = vleo_data::civil_from_days(r.issue).0;
        acc.entry(y).or_default().add(e, o, ps, pl);
    }
    let span: Vec<i32> = match (acc.keys().next(), acc.keys().next_back()) {
        (Some(&a), Some(&b)) => (a..=b).collect(),
        _ => Vec::new(),
    };
    let ok = |y: &i32| acc.get(y).filter(|s| s.n >= FORECAST_YEAR_MIN_PAIRS);
    let col = |f: &dyn Fn(&Sums) -> Option<f64>| -> Vec<Option<f64>> {
        span.iter().map(|y| ok(y).and_then(f)).collect()
    };
    let count = |f: &dyn Fn(&Sums) -> u32| -> Vec<Option<u32>> {
        span.iter().map(|y| ok(y).map(f)).collect()
    };
    let skill_strict = col(&|s| Sums::skill(s.e2s, s.p2s, s.ns));
    let bias = col(&|s| Some(s.se / s.n as f64));
    let pick = |v: &[Option<f64>], low: bool| -> Option<usize> {
        let mut best: Option<usize> = None;
        for (i, x) in v.iter().enumerate() {
            let Some(x) = x.filter(|x| x.is_finite()) else {
                continue;
            };
            if best.is_none_or(|b| {
                let bv = v[b].unwrap_or(f64::NAN);
                if low {
                    x < bv
                } else {
                    x > bv
                }
            }) {
                best = Some(i);
            }
        }
        best
    };
    ForecastByYear {
        pairs: count(&|s| s.n),
        pairs_strict: count(&|s| s.ns),
        pairs_leaky: count(&|s| s.nl),
        skill_leaky: col(&|s| Sums::skill(s.e2l, s.p2l, s.nl)),
        rmse: col(&|s| Some(pmath::sqrt(s.e2 / s.n as f64))),
        thin: acc
            .iter()
            .filter(|(_, s)| s.n < FORECAST_YEAR_MIN_PAIRS)
            .map(|(y, _)| *y)
            .collect(),
        worst: pick(&skill_strict, true).map(|i| span[i]),
        best: pick(&skill_strict, false).map(|i| span[i]),
        lowest_bias: pick(&bias, true).map(|i| span[i]),
        skill_strict,
        bias,
        year: span,
    }
}

/// The issue-age figure's numbers: how many days pass between one outlook
/// and the next.
#[derive(Clone, Debug, PartialEq)]
pub struct IssueAge {
    /// The issues the index lists.
    pub issues: usize,
    /// Each gap length in days, [`ISSUE_GAP_CAP`] standing for that or more,
    /// and how many gaps have it. Two issues on one day make no gap.
    pub gap: Vec<u32>,
    pub count: Vec<u32>,
    pub median: Option<f64>,
    pub mean: Option<f64>,
    /// The commonest gap — the shortest on a tie — and the share of gaps
    /// longer than a day, in per cent.
    pub commonest: Option<u32>,
    pub over_a_day_pct: Option<f64>,
}

/// The issue-age figure's numbers from the index of issue dates.
pub fn issue_age(issues: &[i32]) -> IssueAge {
    let mut ds = issues.to_vec();
    ds.sort_unstable();
    let mut gaps: Vec<f64> = ds
        .windows(2)
        .map(|w| w[1] - w[0])
        .filter(|g| *g > 0)
        .map(|g| g as f64)
        .collect();
    let mut hist: alloc::collections::BTreeMap<u32, u32> = alloc::collections::BTreeMap::new();
    for g in &gaps {
        *hist.entry((*g as u32).min(ISSUE_GAP_CAP)).or_default() += 1;
    }
    // The mean in date order, before the sort the median needs.
    let mean = (!gaps.is_empty()).then(|| gaps.iter().sum::<f64>() / gaps.len() as f64);
    let over = gaps.iter().filter(|g| **g > 1.0).count();
    gaps.sort_by(f64::total_cmp);
    let mut commonest: Option<(u32, u32)> = None;
    for (&g, &c) in &hist {
        if commonest.is_none_or(|(_, bc)| c > bc) {
            commonest = Some((g, c));
        }
    }
    IssueAge {
        issues: issues.len(),
        median: quantile(&gaps, 0.5),
        mean,
        commonest: commonest.map(|(g, _)| g),
        over_a_day_pct: (!gaps.is_empty()).then(|| 100.0 * over as f64 / gaps.len() as f64),
        gap: hist.keys().copied().collect(),
        count: hist.values().copied().collect(),
    }
}

/// The five design scenarios the solar crossing publishes, cold to hot — a
/// ladder, so every quantity drawn across them should rise.
pub const DRIVER_SCENARIOS: [&str; 5] = ["coldday", "coldmean", "nominal", "hotmean", "hotday"];

/// The five quantities it publishes at each.
pub const DRIVER_QUANTITIES: [&str; 5] = ["f107", "f107bar", "ap", "kp_mean", "kp_peak"];

/// How close a ratio to the legacy run has to be to one to count as agreeing:
/// a tenth of a per cent.
pub const PARITY_AGREES: f64 = 0.001;

/// The drivers figure's numbers: the crossing's twenty-five cells against the
/// legacy run's, `[quantity][scenario]` in the order of [`DRIVER_QUANTITIES`]
/// and [`DRIVER_SCENARIOS`].
#[derive(Clone, Debug, PartialEq)]
pub struct DriversParity {
    /// This tree over the legacy run; none where either is missing or the
    /// legacy cell is zero.
    pub ratio: [[Option<f64>; 5]; 5],
    /// This tree minus the legacy run.
    pub gap: [[Option<f64>; 5]; 5],
    /// For each quantity, the scenario where the gap is widest either way —
    /// the first on a tie.
    pub widest: [Option<usize>; 5],
    /// Of the ratios there are, how many agree, how many sit below and how
    /// many above.
    pub agree: u32,
    pub below: u32,
    pub above: u32,
    /// The furthest ratio from one on a log scale, as a factor of at least one
    /// either way — how far the worst cell is out. None with no ratio.
    pub factor: Option<f64>,
}

/// The drivers figure's numbers from the two tables.
pub fn drivers_parity(
    ours: &[[Option<f64>; 5]; 5],
    theirs: &[[Option<f64>; 5]; 5],
) -> DriversParity {
    let mut ratio = [[None; 5]; 5];
    let mut gap = [[None; 5]; 5];
    let mut widest = [None; 5];
    let (mut agree, mut below, mut above) = (0, 0, 0);
    let mut worst: Option<f64> = None;
    for q in 0..5 {
        for sc in 0..5 {
            let (Some(a), Some(b)) = (ours[q][sc], theirs[q][sc]) else {
                continue;
            };
            gap[q][sc] = Some(a - b);
            if widest[q]
                .is_none_or(|w: usize| pmath::abs(a - b) > pmath::abs(gap[q][w].unwrap_or(0.0)))
            {
                widest[q] = Some(sc);
            }
            if b == 0.0 {
                continue;
            }
            let r = a / b;
            ratio[q][sc] = Some(r);
            if pmath::abs(r - 1.0) < PARITY_AGREES {
                agree += 1;
            } else if r < 1.0 - PARITY_AGREES {
                below += 1;
            } else {
                above += 1;
            }
            let m = worst.unwrap_or(1.0);
            worst = Some(if pmath::abs(pmath::ln(r)) > pmath::abs(pmath::ln(m)) {
                r
            } else {
                m
            });
        }
    }
    DriversParity {
        ratio,
        gap,
        widest,
        agree,
        below,
        above,
        factor: worst.map(|w| if w > 1.0 { w } else { 1.0 / w }),
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
        // And the naive band on it, ±2/√1925 — 0.0456.
        assert!((r.naive_band.unwrap() - 2.0 / 1925f64.sqrt()).abs() < 1e-15);
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
    fn the_bands_are_the_rows_own_and_the_cuts_are_where_the_record_crosses_them() {
        // Ap either side of sw_regime's cuts, quiet to 6, active to 25.
        let ap: Vec<SolarDay> = [
            Some(0.0),
            Some(6.0),
            Some(7.0),
            Some(25.0),
            Some(26.0),
            Some(40.0),
            None,
        ]
        .iter()
        .enumerate()
        .map(|(i, a)| day(i as i32, None, *a))
        .collect();
        let s = segments(&ap, Driver::Ap);
        assert_eq!(
            (s.days, s.refused, s.bin_width, s.max),
            (6, 0, 2.0, Some(40.0))
        );
        // Up to 40 in bins of 2 is 21 bins; 6 and 7 share bin 3.
        assert_eq!(s.counts.len(), 21);
        assert_eq!(
            (
                s.counts[0],
                s.counts[3],
                s.counts[12],
                s.counts[13],
                s.counts[20]
            ),
            (1, 2, 1, 1, 1)
        );
        assert_eq!(s.band_days, [2, 2, 2]);
        assert_eq!(
            (s.below_cut.clone(), s.above_cut.clone()),
            (vec![Some(6.0), Some(25.0)], vec![Some(7.0), Some(26.0)])
        );
        // F10.7 either side of sw_activity_band's 90, 130 and 170: a flux ON
        // an edge belongs to the band the edge opens.
        let f: Vec<SolarDay> = [89.9, 90.0, 129.9, 130.0, 169.9, 170.0, 343.0]
            .iter()
            .enumerate()
            .map(|(i, x)| day(i as i32, Some(*x), None))
            .collect();
        let s = segments(&f, Driver::F107);
        assert_eq!(s.band_days, [1, 2, 2, 2]);
        assert_eq!(s.above_cut, [Some(90.0), Some(130.0), Some(170.0)]);
        assert_eq!(s.below_cut, [Some(89.9), Some(129.9), Some(169.9)]);
        assert!((s.band_pct[0] - 100.0 / 7.0).abs() < 1e-12);
        // Up to 343 sfu in bins of 5 is 70 bins.
        assert_eq!(s.counts.len(), 70);
    }

    #[test]
    fn storm_and_quiet_shares_are_counted_per_phase_by_the_regime_row() {
        // Twenty days of one complete cycle: one day in each phase bin. Day
        // 0 is a storm, day 2 active, the rest quiet.
        let rec: Vec<SolarDay> = (0..20)
            .map(|i| {
                day(
                    i,
                    None,
                    Some(match i {
                        0 => 30.0,
                        2 => 10.0,
                        _ => 3.0,
                    }),
                )
            })
            .collect();
        let r = regime_phase(&rec, &[cycle(1, 0, 20), cycle(2, 20, 40)]);
        assert_eq!(r.storm_pct[0], Some(100.0));
        assert_eq!(r.storm_pct[1], Some(0.0));
        assert_eq!(
            (r.quiet_pct[0], r.quiet_pct[2], r.quiet_pct[3]),
            (Some(0.0), Some(0.0), Some(100.0))
        );
        assert_eq!(r.storm_peak, Some((100.0, 0.025)));
        // The lowest quiet share is 0 at bins 0 and 2; the first wins.
        assert_eq!(r.quiet_low, Some((0.0, 0.025)));
        assert!((r.mirror.unwrap() + 0.6882472016116853).abs() < 1e-12);
    }

    #[test]
    fn the_record_is_grouped_by_its_own_calendar() {
        let at = |s: &str, v: Option<f64>| day(vleo_data::days_since_2000(s).unwrap(), v, None);
        let rec = [
            at("1999-12-31", Some(1.0)),
            at("2000-01-01", Some(3.0)),
            at("2000-01-02", Some(5.0)),
            at("2000-02-01", Some(10.0)),
            at("2000-02-02", None),
        ];
        // By year: 1999 holds 1, 2000 holds 3, 5, 10. The record's mean is over
        // its four DAYS, 4.75, not over its two years.
        let y = climate(&rec, Driver::F107, Grouping::Year);
        assert_eq!(
            (y.key.clone(), y.mean.clone(), y.days_in.clone()),
            (vec![1999, 2000], vec![1.0, 6.0], vec![1, 3])
        );
        assert_eq!((y.overall, y.days, y.below), (Some(4.75), 4, 1));
        assert_eq!((y.high, y.low), (Some((6.0, 2000.0)), Some((1.0, 1999.0))));
        // By month: December 1999 is key 24000 and is drawn at 2000.0.
        let m = climate(&rec, Driver::F107, Grouping::Month);
        assert_eq!(m.key, [24000, 24001, 24002]);
        assert_eq!(m.x, [2000.0, 2000.0 + 1.0 / 12.0, 2000.0 + 2.0 / 12.0]);
        assert_eq!(m.mean, [1.0, 4.0, 10.0]);
        // By day of year in fives: day 365 → 365, days 1 and 2 → 5, day 32 → 35.
        let d = climate(&rec, Driver::F107, Grouping::DayOfYear);
        assert_eq!((d.key, d.mean), (vec![5, 35, 365], vec![4.0, 10.0, 1.0]));
    }

    #[test]
    fn the_smoother_removes_what_the_months_add() {
        let mo = |i: i32, raw: f64, sm: Option<f64>| MonthlyMean {
            day: i * 30,
            f107_mean: Some(raw),
            ap_mean: None,
            ssn_mean: None,
            f107_smooth: sm,
            ap_smooth: None,
            ssn_smooth: None,
        };
        let months = [
            mo(0, 10.0, Some(9.0)),
            mo(1, 8.0, Some(9.0)),
            mo(2, 9.0, Some(9.0)),
            mo(3, 12.0, None),
            mo(4, 11.0, Some(10.0)),
        ];
        let s = smoother(&months, Driver::F107);
        assert_eq!((s.missing, s.months), (1, 5));
        // Departures +1, −1, 0, +1: it crosses twice, and a zero is not a
        // crossing.
        assert_eq!((s.crossings, s.max_departure), (2, 1.0));
        assert_eq!((s.raw_range, s.smooth_range), (3.0, 1.0));
        assert!((s.rms - 0.8660254037844386).abs() < 1e-15);
        assert_eq!(s.x[1], 30.0 / 365.25 + 2000.0);
        // No month carries Ap: nothing to measure, and none of it invented.
        let a = smoother(&months, Driver::Ap);
        assert_eq!(
            (a.raw_range, a.rms, a.crossings, a.missing),
            (0.0, 0.0, 0, 5)
        );
    }

    #[test]
    fn a_driver_that_rises_a_unit_a_day_changes_by_its_lead() {
        // Value = day, for 200 days: every change over a lead is the lead
        // itself, at every percentile, over 200 − lead pairs.
        let rec: Vec<SolarDay> = (0..200).map(|i| day(i, Some(i as f64), None)).collect();
        let g = growth(&rec, Driver::F107);
        // 30, then ×1.35 to the whole day: 41, 55, 74, 100, 135, 182, …
        assert_eq!(g.lead_years.len(), 18);
        assert_eq!(
            g.lead_years[..3],
            [30.0 / 365.25, 41.0 / 365.25, 55.0 / 365.25]
        );
        assert_eq!(g.pairs[..6], [170, 159, 145, 126, 100, 65]);
        for q in &g.change {
            assert_eq!(q[0], Some(30.0));
            assert_eq!(q[5], Some(135.0));
            // 246 days is past the record: no pair, no change.
            assert_eq!(q[7], None);
        }
        // 332 days is the lead nearest a year.
        assert_eq!(g.at_year, 8);
        // Nothing past 200 days, so no hump, no dip, no cycle to see.
        assert_eq!((g.hump, g.dip, g.humped), (None, None, false));
    }

    #[test]
    fn a_cycles_changes_are_taken_inside_it() {
        // The same ramp in two cycles of 100 and 200 days; leads 30, 45, 68,
        // 102, 153, 230 … ×1.5. A lead is drawn only above thirty pairs.
        let rec: Vec<SolarDay> = (0..300).map(|i| day(i, Some(i as f64), None)).collect();
        let g = growth_by_cycle(&rec, &[cycle(1, 0, 100), cycle(2, 100, 300)], Driver::F107);
        // Cycle 1 has 32 pairs at 68 days and none at 102.
        assert_eq!(g.change[0][..4], [Some(30.0), Some(45.0), Some(68.0), None]);
        // Cycle 2 has 47 at 153 and none at 230.
        assert_eq!(
            g.change[1][..6],
            [
                Some(30.0),
                Some(45.0),
                Some(68.0),
                Some(102.0),
                Some(153.0),
                None
            ]
        );
        // 68 days is the longest lead both reach, and there they agree.
        assert_eq!(g.shared, Some(68.0 / 365.25));
        assert_eq!(g.spread, Some((68.0, 68.0)));
    }

    fn issued(issue: i32, target: i32, lead: Option<f64>, f107: f64) -> IssuedForecast {
        IssuedForecast {
            issue,
            target,
            lead,
            f107: Some(f107),
        }
    }

    #[test]
    fn a_forecast_is_scored_against_the_day_it_forecast() {
        // Days 0 to 9 at F10.7 = 100 + day, day 5 missing.
        let rec: Vec<SolarDay> = (0..10)
            .map(|i| day(i, (i != 5).then_some(100.0 + i as f64), None))
            .collect();
        let fc = [
            // Lead 1 from day 3: error +2; strict persistence is day 2 (102,
            // off by −2), leaky is day 3 itself (103, off by −1).
            issued(3, 4, Some(1.0), 106.0),
            // Its target is the missing day: no observation, no pair at all.
            issued(3, 5, Some(2.0), 110.0),
            // Lead 2: error −6; persistence off by −4 and −3.
            issued(3, 6, Some(2.0), 100.0),
            // Lead 1 from day 6: error 0. The day before is missing, so the
            // strict baseline reaches back to day 4 (off by −3); leaky is day
            // 6 (off by −1).
            issued(6, 7, Some(1.0), 107.0),
            // Issued on a day the record lacks: scored, but no baseline.
            issued(20, 8, Some(1.0), 110.0),
            // No lead, and a lead past the span: neither is scored.
            issued(3, 4, None, 50.0),
            issued(3, 4, Some(28.0), 50.0),
        ];
        let f = forecast_by_lead(&rec, &fc);
        assert_eq!(f.lead, [1, 2]);
        assert_eq!(
            (f.pairs, f.pairs_strict, f.pairs_leaky),
            (vec![3, 1], vec![2, 1], vec![2, 1])
        );
        // Lead 1: errors 2, 0, 2 — mean 4/3, RMS √(8/3). Strict: forecast
        // squares 4 + 0 against baseline 4 + 9, skill 1 − 4/13 = 9/13; leaky:
        // 4 against 1 + 1, skill −1.
        assert_eq!(f.bias, [4.0 / 3.0, -6.0]);
        assert!((f.rmse[0] - (8.0f64 / 3.0).sqrt()).abs() < 1e-12);
        assert!((f.rmse[1] - 6.0).abs() < 1e-12);
        assert!((f.skill_strict[0].unwrap() - 9.0 / 13.0).abs() < 1e-12);
        assert_eq!(f.skill_leaky[0], Some(-1.0));
        // Lead 2: 36 against 16 and against 9.
        assert_eq!(
            (f.skill_strict[1], f.skill_leaky[1]),
            (Some(-1.25), Some(-3.0))
        );
        // Skill peaks at lead 1, and the strict baseline misses a pair only
        // there.
        assert_eq!((f.peak, f.widest), (Some(0), Some(1)));
    }

    #[test]
    fn a_forecast_year_is_its_issue_year_and_a_thin_one_is_a_hole() {
        // Every day from 2000-01-01 (day 0) to 2004, F10.7 on a three-day
        // cycle so persistence always misses.
        let rec: Vec<SolarDay> = (0..1500)
            .map(|i| day(i, Some(100.0 + 10.0 * (i % 3) as f64), None))
            .collect();
        let obs = |t: i32| 100.0 + 10.0 * (t % 3) as f64;
        let mut fc = Vec::new();
        // 2000: 250 issues at lead 1, each 1 high.
        fc.extend((10..260).map(|i| issued(i, i + 1, Some(1.0), obs(i + 1) + 1.0)));
        // 2001 (from day 366): ten — too few to score.
        fc.extend((400..410).map(|i| issued(i, i + 1, Some(1.0), obs(i + 1))));
        // 2002 (from day 731): only leads past the band, so not in it at all.
        fc.extend((800..805).map(|i| issued(i, i + 15, Some(15.0), obs(i + 15))));
        // 2003 (from day 1096): 300 issues, each 2 low.
        fc.extend((1100..1400).map(|i| issued(i, i + 1, Some(1.0), obs(i + 1) - 2.0)));
        let f = forecast_by_year(&rec, &fc);
        assert_eq!(f.year, [2000, 2001, 2002, 2003]);
        assert_eq!(f.pairs, [Some(250), None, None, Some(300)]);
        assert_eq!(f.thin, [2001]);
        assert_eq!(f.bias, [Some(1.0), None, None, Some(-2.0)]);
        assert_eq!(f.rmse, [Some(1.0), None, None, Some(2.0)]);
        assert_eq!(f.lowest_bias, Some(2003));
        // The baseline misses by the same spread in both years, so the year
        // the forecast misses by more has the lower skill.
        assert_eq!((f.worst, f.best), (Some(2003), Some(2000)));
        assert!(f.skill_strict[1].is_none() && f.skill_leaky[2].is_none());
    }

    #[test]
    fn the_age_of_an_issue_is_the_gap_before_the_next() {
        // Sorted: 0 3 7 7 10 11 50 — gaps 3 4 3 1 39, the repeat making none.
        let a = issue_age(&[0, 7, 7, 10, 50, 3, 11]);
        assert_eq!(a.issues, 7);
        // 39 days counts as "30 or more".
        assert_eq!((a.gap, a.count), (vec![1, 3, 4, 30], vec![1, 2, 1, 1]));
        // Mean 50 / 5; sorted 1 3 3 4 39, median 3.
        assert_eq!((a.mean, a.median), (Some(10.0), Some(3.0)));
        assert_eq!(a.commonest, Some(3));
        // Four of the five gaps are longer than a day.
        assert_eq!(a.over_a_day_pct, Some(80.0));
    }

    #[test]
    fn the_drivers_are_compared_cell_by_cell() {
        let mut ours = [[None; 5]; 5];
        let mut theirs = [[None; 5]; 5];
        // F10.7: half the legacy value at every scenario but the last, which
        // the legacy run has no number for.
        for sc in 0..4 {
            ours[0][sc] = Some(50.0 + sc as f64);
            theirs[0][sc] = Some(2.0 * (50.0 + sc as f64));
        }
        ours[0][4] = Some(80.0);
        // Ap: agreeing to 1e-5 at two, a factor of 3 high at the worst day,
        // and a legacy zero, which has a gap but no ratio.
        ours[2] = [Some(10.0), Some(20.00002), Some(0.5), None, Some(30.0)];
        theirs[2] = [Some(10.0), Some(20.0), Some(0.0), Some(7.0), Some(10.0)];
        let p = drivers_parity(&ours, &theirs);
        assert_eq!(
            p.ratio[0],
            [Some(0.5), Some(0.5), Some(0.5), Some(0.5), None]
        );
        assert_eq!(p.ratio[2][2], None);
        assert_eq!(p.gap[2][2], Some(0.5));
        assert_eq!(p.ratio[2][4], Some(3.0));
        // F10.7's gap widens with the value: widest at the fourth scenario.
        // Ap's is widest at the worst day (20).
        assert_eq!(p.widest, [Some(3), None, Some(4), None, None]);
        // Seven ratios: two agree, four sit below, one above.
        assert_eq!((p.agree, p.below, p.above), (2, 4, 1));
        // A third is ln 3 from one and a half ln 2, so the worst is Ap's 3.
        assert_eq!(p.factor, Some(3.0));
    }

    #[test]
    fn a_record_with_nothing_to_pair_says_so() {
        let d = density(&[day(0, Some(70.0), None), day(1, None, Some(3.0))]);
        assert_eq!(d.days, 0);
        assert_eq!((d.r, d.median_f107, d.median_ap), (None, None, None));
        assert_eq!((d.below_both_pct, d.storm_deciles), (0.0, 0));
    }
}
