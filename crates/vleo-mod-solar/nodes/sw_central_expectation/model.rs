// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// Where is F10.7 expected to be by the time the mission is there?
///
/// `F107_central(T_e, L) = w*F107_today + (1-w)*mean_[T_e, T_e+L] A(t),  w = exp(-L / 27 d),  A(t) = P25 * R((t - T_max) mod P)`
///
/// Source: `noaa_swpc`
///
/// The centre of the F10.7 design value; sw_uncertainty_growth supplies the
/// spread around it and sw_f107_design adds the two together. Two things
/// decide the answer and the row needs both: how far ahead it is being asked
/// about, and WHEN. A mission through 2027-2032 flies the declining half of
/// cycle 25 and meets a different sky from one through 2032-2037, and no
/// single number can be right for both.
///
/// # Assumptions
///
/// * Two completed cycles is the whole sample, and eleven of the ninety-three grid points rest on one of them — fails when the record spans cycles 23, 24 and the incomplete 25, so the shape R is a mean of TWO curves and its spread between them is not published by this row. Where the two cycles' differing lengths leave only one of them covering a point — eleven of ninety-three, near the wrap — the value is that one cycle's shape rather than an average. Two cycles cannot establish that a shape repeats; they can only establish what the last two did, and this row says the next one resembles them because that is the best the record supports, not because it is known.
/// * Wrapping assumes future cycles repeat cycle 25's AMPLITUDE, not just its shape — fails when beyond one mean cycle length past 2024-09-04 the analogue repeats, and it repeats scaled by cycle 25's own peak of 225.1 sfu. Cycle amplitude is the thing that does NOT repeat — 226.5 sfu for cycle 23 against 160.6 for cycle 24, a factor of 1.41 — so any window reaching past about 2036 is reading a level whose size has no support at all. It affects the long end of the declared range: a fifteen-year mission spends most of its window there. The fifteen-year answer happens to land near the unconditional climatology, which makes it reasonable by accident rather than by evidence.
/// * The answer is a window MEAN, so it understates the early years of a long mission — fails when a five-year mission opening at the declared epoch averages 90.4 sfu, but its first ninety days average 104.6 and it falls to 73.2 by the end of the window — a spread of 39 sfu inside one number. Drag is not linear in flux and a vehicle does not average its propellant over five years, so a design whose sizing case is its worst sustained period is reading the wrong statistic here. This row publishes a centre because it is a centre; the maximum of the analogue over the window is a different number and nothing publishes it yet.
/// * The grid is 45 days and the window integral is numerical, and both errors are measured rather than assumed — fails when the shape is stored on a 45-day grid and interpolated linearly, which over 328 (epoch, duration) combinations spanning the whole declared domain departs from the same construction on a 10-day grid by at most 0.962 sfu, worst at the shortest windows where the grid is coarsest relative to the window. Simpson's rule with 512 panels adds at most 0.0045 sfu. Both are far inside the spread between the two cycles the shape is built from, which is the error that actually matters and which this row does not publish.
/// * Persistence is carried for completeness and is worth nothing at any mission lead — fails when the weight is exp(-L/27) with L in days, so at the shortest mission the declared input range allows — half a year — today's flux contributes 0.114% and by one year it contributes 0.00013%. The term is right and it is inert: this row's answer is the window climatology to four decimal places for every lead a mission can ask about. It is kept because the relation is the study's, and removing it would make the row silently wrong for the short-lead use nothing in this tree currently makes.
/// * This moves the row AWAY from the MATLAB tool, deliberately — fails when the MATLAB tool answers 158.33 sfu for its own 2027 window by freezing its last 27-day rotation forecast and holding it flat; this row now answers 96.5 sfu for the same window and used to answer 114.8. The port therefore agrees with MATLAB LESS than it did. That is the intended direction: over the same span past maximum the two completed cycles ran at 91 sfu scaled onto cycle 25's amplitude, so the record puts MATLAB 74% high and the old answer 26% high. tools/mat_parity.py records the disagreement and its size so that it stays a decision rather than becoming a surprise.
pub const NODE_ID: &str = "sw_central_expectation";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x3aaab26b71d86fd9;

pub fn evaluate(today: Ratio, lead: Time, epoch: Time) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : average the amplitude-scaled cycle analogue over the mission window -> Ratio
    // The cycle SHAPE, measured on solar-weather@2026.09.14 outside this crate:
    // for each completed cycle, its centred 81-day mean F10.7 divided by that same
    // cycle's own 81-day peak, sampled every 45 days forward from its maximum and
    // averaged over the cycles. Normalising by a peak measured the SAME way is
    // what keeps R inside 0..1; eleven of these ninety-three points rest on one
    // cycle rather than two, which the sheet declares.
    const SHAPE: &[f64] = &[
        1.000000, 0.900844, 0.805356, 0.770000, 0.746428, 0.751101,
        0.700513, 0.693061, 0.672702, 0.629521, 0.587056, 0.555654,
        0.561512, 0.525606, 0.561663, 0.569811, 0.517998, 0.483344,
        0.465382, 0.437840, 0.469029, 0.498476, 0.471968, 0.463695,
        0.448868, 0.439450, 0.418816, 0.421880, 0.428582, 0.426012,
        0.402372, 0.396879, 0.409511, 0.396353, 0.395619, 0.406135,
        0.391849, 0.384483, 0.380332, 0.386172, 0.401062, 0.348846,
        0.325169, 0.325985, 0.326203, 0.370261, 0.366773, 0.367170,
        0.380647, 0.375386, 0.371073, 0.376138, 0.378425, 0.389045,
        0.405370, 0.399551, 0.381763, 0.413764, 0.442563, 0.455251,
        0.472536, 0.470577, 0.530599, 0.583287, 0.559839, 0.576960,
        0.607149, 0.682233, 0.772594, 0.750614, 0.670980, 0.641612,
        0.719618, 0.761199, 0.723642, 0.745599, 0.767564, 0.755899,
        0.771999, 0.764130, 0.789755, 0.772538, 0.732655, 0.738427,
        0.800573, 0.837048, 0.863930, 0.865378, 0.780473, 0.754023,
        0.812385, 0.901009, 0.966113,
    ];
    // One mean completed-cycle length (cycle 23 at 11.88 yr, cycle 24 at 11.00),
    // the period the shape is wrapped on, and the 45-day spacing of the grid.
    const PERIOD_DAYS: f64 = 4178.0;
    const STEP_DAYS: f64 = 45.0;
    // Cycle 25's own 81-day peak and the day it fell on, days since 2000-01-01.
    // Cycle 25 is NOT finished, so both move when the bundle does.
    const PEAK_SFU: f64 = 225.1358024691358;
    const PEAK_DAY: f64 = 9013.0;

    // The expected flux at one date: the shape at that date's distance past the
    // maximum, wrapped, put back on this cycle's size. Rust's `%` keeps the sign
    // of the dividend, so a date before the maximum needs the second fold.
    let analogue = |t: f64| -> f64 {
        let u = ((t - PEAK_DAY) % PERIOD_DAYS + PERIOD_DAYS) % PERIOD_DAYS;
        let i = u / STEP_DAYS;
        let lo = i as usize % SHAPE.len();
        let hi = (lo + 1) % SHAPE.len();
        PEAK_SFU * (SHAPE[lo] + (SHAPE[hi] - SHAPE[lo]) * (i - i.floor()))
    };

    // The mean over the window the mission actually flies, not the value at its
    // end: the end of a five-year mission opening in 2027 sits near minimum and
    // would size the design for the quietest sky it meets. 512 panels holds the
    // quadrature error under 0.005 sfu across the whole declared domain.
    let t0 = epoch.days();
    let t1 = t0 + lead.days();
    use vleo_core::math::integrate;
    let climo: Ratio = Ratio::new(integrate::simpson(t0, t1, 512, analogue) / (t1 - t0));
    // ---- end HOLE 1
    // ---- HOLE 2 : weight today's flux against that window climatology by the lead, one solar rotation as the timescale -> Ratio
    // One synodic solar rotation, prf_design's own timescale for persistence. At
    // every lead the declared input range allows this weight is below 0.0012, so
    // the term is inert and kept only because the relation is the study's.
    const TAU_DAYS: f64 = 27.0;
    let w: f64 = pmath::exp(-lead.days() / TAU_DAYS);
    let central: Ratio = Ratio::new(w * today.get() + (1.0 - w) * climo.get());
    // ---- end HOLE 2

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = central;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107_central", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_central", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the answer is a weighted blend of today's F10.7 and a window mean of the analogue, so it cannot leave the interval between them. The analogue is bounded below by 0.325169 x 225.1358 = 73.2 sfu, and env_f107 by 60 because below 60 sfu has never been observed; the blend therefore floors at 60" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_central", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "env_f107's upper bound is 400, above which the exospheric temperature relation is extrapolated past the largest recorded daily value. The analogue cannot exceed cycle 25's own 81-day peak of 225.1 sfu, and a blend cannot exceed its larger input, so this bound catches a broken weight or a broken table rather than an extreme sky" });
    }
    Ok(answer)
}
