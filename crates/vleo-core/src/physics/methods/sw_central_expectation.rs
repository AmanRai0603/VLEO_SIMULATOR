//! GENERATED from the method of `sw_central_expectation` by `cargo xtask docs`, translated by
//! the fixed rules in crates/vleo-sheet/src/method.rs. Do not edit: the method
//! is changed on the node's form, and this is written again from it.

#![allow(
    clippy::all,
    clippy::float_cmp,
    clippy::cast_precision_loss,
    unreachable_code,
    unused_assignments,
    unused_imports,
    unused_mut,
    unused_variables,
    unused_parens,
    non_snake_case
)]

use vleo_units::constants::*;
use vleo_units::method_rt::{self as rt, MethodError};
use vleo_units::pmath;

/// The method of `sw_central_expectation`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_central_expectation/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Numbers: the mean solar cycle (vleo_core::physics::env SOLAR_CYCLE_SHAPE, period 4178 d, cycle 25 peak 225.1358 sfu on day 9013, completed-cycle mean peak 193.858 sfu), measured from bundles solar-weather@2026.09.14 (NOAA SWPC record, noaa_swpc); tau = one synodic rotation, 27 d, prf_design's persistence timescale.
/// # HOLE 1 is env::solar_cycle_analogue_mean(t0, t1): composite Simpson, 512 panels, of solar_cycle_analogue, divided by (t1 - t0).
/// # HOLE 2 blends today's flux against that window mean with w = exp(-L / 27 d).
///
/// const p = 4178 [1]                     # SOLAR_CYCLE_PERIOD_DAYS, one mean completed-cycle length, days
/// const peak_day = 9013 [1]              # SOLAR_CYCLE_PEAK_DAY, cycle 25's 81-day peak, days since 2000-01-01 (2024-09-04)
/// const amp25 = 225.1358024691358 [1]    # SOLAR_CYCLE_PEAK_SFU, cycle 25's own 81-day peak, sfu
/// const amp_mean = 193.85802469135802 [1]   # SOLAR_CYCLE_MEAN_PEAK_SFU, mean of cycles 23 and 24's peaks, sfu
/// const panels = 512 [1]                 # Simpson panels
/// const tau_days = 27 [1]                # one synodic solar rotation, days
///
/// # HOLE 1 — the window, in days since 2000-01-01, as epoch.days() and epoch.days() + lead.days().
/// let t0 = epoch / 1 [d]
/// let t1 = t0 + lead / 1 [d]
/// let h = (t1 - t0) / panels
/// let s = 0 [1]
/// let x = t0
/// let d = 0 [1]
/// let amp = amp25
/// let u = 0 [1]
/// let i = 0 [1]
/// let f = 0 [1]
/// # Simpson, summed in the code's own order: f(a) + f(b) first, then the interior points 1..511.
/// for j = 0 to 512
///   if j == 0 then
///     set x = t0
///   else if j == 1 then
///     set x = t1
///   else
///     set x = t0 + h * (j - 1)
///   end
///   # solar_cycle_analogue(x): fold onto one period from cycle 25's maximum, read the shape, scale by the cycle's amplitude.
///   set d = x - peak_day
///   if floor((d + 0.5 * p) / p) == 0 then
///     set amp = amp25
///   else
///     set amp = amp_mean
///   end
///   set u = fmod(fmod(d, p) + p, p)
///   set i = u / (p / 94)
///   # SOLAR_CYCLE_SHAPE[lo] + (SOLAR_CYCLE_SHAPE[hi] - SOLAR_CYCLE_SHAPE[lo]) * frac, hi = (lo + 1) mod 94: the shape wrapped by repeating knot 0 at 94.
///   set f = amp * interp(i, [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0, 30.0, 31.0, 32.0, 33.0, 34.0, 35.0, 36.0, 37.0, 38.0, 39.0, 40.0, 41.0, 42.0, 43.0, 44.0, 45.0, 46.0, 47.0, 48.0, 49.0, 50.0, 51.0, 52.0, 53.0, 54.0, 55.0, 56.0, 57.0, 58.0, 59.0, 60.0, 61.0, 62.0, 63.0, 64.0, 65.0, 66.0, 67.0, 68.0, 69.0, 70.0, 71.0, 72.0, 73.0, 74.0, 75.0, 76.0, 77.0, 78.0, 79.0, 80.0, 81.0, 82.0, 83.0, 84.0, 85.0, 86.0, 87.0, 88.0, 89.0, 90.0, 91.0, 92.0, 93.0, 94.0], [1.000000, 0.903695, 0.807593, 0.773599, 0.747277, 0.750621, 0.702665, 0.699680, 0.671394, 0.633116, 0.593519, 0.561394, 0.560421, 0.530158, 0.554559, 0.572413, 0.522981, 0.489549, 0.476323, 0.450250, 0.465763, 0.491509, 0.455476, 0.469410, 0.460672, 0.437930, 0.421984, 0.420851, 0.437148, 0.427348, 0.415508, 0.401208, 0.404088, 0.401337, 0.395792, 0.403236, 0.402821, 0.384875, 0.380210, 0.383322, 0.393501, 0.400451, 0.340028, 0.324026, 0.324897, 0.373319, 0.369609, 0.365922, 0.375961, 0.381349, 0.370032, 0.371935, 0.378041, 0.381774, 0.397795, 0.405428, 0.393973, 0.389129, 0.426488, 0.453456, 0.463151, 0.470999, 0.479126, 0.544836, 0.578380, 0.555890, 0.584878, 0.621035, 0.702772, 0.763337, 0.739668, 0.662755, 0.650724, 0.730621, 0.757925, 0.724262, 0.745062, 0.767270, 0.758000, 0.773305, 0.764422, 0.789755, 0.772538, 0.732759, 0.736271, 0.796305, 0.842015, 0.866375, 0.862554, 0.794588, 0.753098, 0.812943, 0.899607, 0.963136, 1.000000])
///   if j <= 1 then
///     set s = s + f
///   else if fmod(j - 1, 2) == 1 then
///     set s = s + 4 * f
///   else
///     set s = s + 2 * f
///   end
/// end
/// let climo = s * h / 3 / (t1 - t0)
///
/// # HOLE 2 — weight today's flux against the window climatology by the lead.
/// let w = exp(-(lead / 1 [d]) / tau_days)
/// let central = w * today + (1 - w) * climo
///
/// # Generated guards: the declared domain of F107_central.
/// if central < 60 then
///   refuse "below 60 sfu has never been observed, and a blend of today's flux and the analogue cannot leave the interval between them"
/// end
/// if central > 400 then
///   refuse "above 400 the exospheric temperature relation is extrapolated past the largest recorded daily value; this catches a broken weight or table"
/// end
/// return central
/// ```
pub fn evaluate(today: f64, lead: f64, epoch: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(today) || pmath::is_nan(today) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(lead) || pmath::is_nan(lead) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(epoch) || pmath::is_nan(epoch) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    let p: f64 = rt::fin(4178.0, 6)?;
    let peak_day: f64 = rt::fin(9013.0, 7)?;
    let amp25: f64 = rt::fin(225.1358024691358, 8)?;
    let amp_mean: f64 = rt::fin(193.85802469135803, 9)?;
    let panels: f64 = rt::fin(512.0, 10)?;
    let tau_days: f64 = rt::fin(27.0, 11)?;
    let t0: f64 = rt::fin(rt::div(epoch, 86400.0, 14)?, 14)?;
    let t1: f64 = rt::fin((t0 + rt::div(lead, 86400.0, 15)?), 15)?;
    let h: f64 = rt::fin(rt::div((t1 - t0), panels, 16)?, 16)?;
    let mut s: f64 = rt::fin(0.0, 17)?;
    let mut x: f64 = rt::fin(t0, 18)?;
    let mut d: f64 = rt::fin(0.0, 19)?;
    let mut amp: f64 = rt::fin(amp25, 20)?;
    let mut u: f64 = rt::fin(0.0, 21)?;
    let mut i: f64 = rt::fin(0.0, 22)?;
    let mut f: f64 = rt::fin(0.0, 23)?;
    for step_1 in (0_i64)..=(512_i64) {
        let j: f64 = step_1 as f64;
        if (j == 0.0) {
            x = rt::fin(t0, 27)?;
        } else if (j == 1.0) {
            x = rt::fin(t1, 29)?;
        } else {
            x = rt::fin((t0 + (h * (j - 1.0))), 31)?;
        }
        d = rt::fin((x - peak_day), 34)?;
        if (pmath::floor(rt::div((d + (0.5 * p)), p, 35)?) == 0.0) {
            amp = rt::fin(amp25, 36)?;
        } else {
            amp = rt::fin(amp_mean, 38)?;
        }
        u = rt::fin(rt::fmod((rt::fmod(d, p, 40)? + p), p, 40)?, 40)?;
        i = rt::fin(rt::div(u, rt::div(p, 94.0, 41)?, 41)?, 41)?;
        f = rt::fin(
            (amp * pmath::interp(
                i,
                &[
                    0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0,
                    15.0, 16.0, 17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0,
                    28.0, 29.0, 30.0, 31.0, 32.0, 33.0, 34.0, 35.0, 36.0, 37.0, 38.0, 39.0, 40.0,
                    41.0, 42.0, 43.0, 44.0, 45.0, 46.0, 47.0, 48.0, 49.0, 50.0, 51.0, 52.0, 53.0,
                    54.0, 55.0, 56.0, 57.0, 58.0, 59.0, 60.0, 61.0, 62.0, 63.0, 64.0, 65.0, 66.0,
                    67.0, 68.0, 69.0, 70.0, 71.0, 72.0, 73.0, 74.0, 75.0, 76.0, 77.0, 78.0, 79.0,
                    80.0, 81.0, 82.0, 83.0, 84.0, 85.0, 86.0, 87.0, 88.0, 89.0, 90.0, 91.0, 92.0,
                    93.0, 94.0,
                ],
                &[
                    1.0, 0.903695, 0.807593, 0.773599, 0.747277, 0.750621, 0.702665, 0.69968,
                    0.671394, 0.633116, 0.593519, 0.561394, 0.560421, 0.530158, 0.554559, 0.572413,
                    0.522981, 0.489549, 0.476323, 0.45025, 0.465763, 0.491509, 0.455476, 0.46941,
                    0.460672, 0.43793, 0.421984, 0.420851, 0.437148, 0.427348, 0.415508, 0.401208,
                    0.404088, 0.401337, 0.395792, 0.403236, 0.402821, 0.384875, 0.38021, 0.383322,
                    0.393501, 0.400451, 0.340028, 0.324026, 0.324897, 0.373319, 0.369609, 0.365922,
                    0.375961, 0.381349, 0.370032, 0.371935, 0.378041, 0.381774, 0.397795, 0.405428,
                    0.393973, 0.389129, 0.426488, 0.453456, 0.463151, 0.470999, 0.479126, 0.544836,
                    0.57838, 0.55589, 0.584878, 0.621035, 0.702772, 0.763337, 0.739668, 0.662755,
                    0.650724, 0.730621, 0.757925, 0.724262, 0.745062, 0.76727, 0.758, 0.773305,
                    0.764422, 0.789755, 0.772538, 0.732759, 0.736271, 0.796305, 0.842015, 0.866375,
                    0.862554, 0.794588, 0.753098, 0.812943, 0.899607, 0.963136, 1.0,
                ],
            )),
            43,
        )?;
        if (j <= 1.0) {
            s = rt::fin((s + f), 45)?;
        } else if (rt::fmod((j - 1.0), 2.0, 46)? == 1.0) {
            s = rt::fin((s + (4.0 * f)), 47)?;
        } else {
            s = rt::fin((s + (2.0 * f)), 49)?;
        }
    }
    let climo: f64 = rt::fin(rt::div(rt::div((s * h), 3.0, 52)?, (t1 - t0), 52)?, 52)?;
    let w: f64 = rt::fin(
        pmath::exp(rt::div((-rt::div(lead, 86400.0, 55)?), tau_days, 55)?),
        55,
    )?;
    let central: f64 = rt::fin(((w * today) + ((1.0 - w) * climo)), 56)?;
    if (central < 60.0) {
        return Err(MethodError::Refused("below 60 sfu has never been observed, and a blend of today's flux and the analogue cannot leave the interval between them"));
    }
    if (central > 400.0) {
        return Err(MethodError::Refused("above 400 the exospheric temperature relation is extrapolated past the largest recorded daily value; this catches a broken weight or table"));
    }
    return Ok(rt::fin(central, 65)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
