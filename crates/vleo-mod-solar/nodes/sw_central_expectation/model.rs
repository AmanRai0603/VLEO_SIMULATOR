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
/// `F107_central(T_e, L) = w*F107_today + (1-w)*mean_[T_e, T_e+L] A(t),  w = exp(-L / 27 d),  A(t) = amp(t) * R((t - T_max) mod P)`
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
/// * Outside cycle 25 the amplitude is the mean of TWO completed cycles, and their spread is a factor of 1.41 — fails when cycle 23 peaked at 226.8 sfu and cycle 24 at 160.9, so the 193.9 this row uses for every future cycle is the midpoint of two numbers that differ by 41%. A window reaching past about 2030 is reading a level whose size is that average, and if the next cycle runs like cycle 23 the answer is 17% low, if like cycle 24 it is 17% high. That is an honest estimate rather than a repeat of the current cycle, which is what this row used to do, but two cycles cannot support an uncertainty on it and none is published. The row does not know, and does not claim to know, which kind of cycle comes next.
/// * The answer is a window MEAN, so it understates the early years of a long mission — fails when a five-year mission opening at the declared epoch averages 90.4 sfu, but its first ninety days average 104.6 and it falls to 73.2 by the end of the window — a spread of 39 sfu inside one number. Drag is not linear in flux and a vehicle does not average its propellant over five years, so a design whose sizing case is its worst sustained period is reading the wrong statistic here. This row publishes a centre because it is a centre; the maximum of the analogue over the window is a different number and nothing publishes it yet.
/// * The grid is 94 knots and the window integral is numerical, and both errors are measured rather than assumed — fails when the shape is stored on 94 knots of 44.4468 days and interpolated linearly. The knot count is even and divides the period exactly, which is not cosmetic: it makes the wrap from the last knot to the first one step like any other, and it puts the amplitude handover on knot 47 rather than somewhere inside a segment. An earlier form of this row used a round 45-day step, which does not divide 4178, and carried a 1.2 sfu discontinuity at the wrap as a result. Simpson's rule with 512 panels adds at most 0.0045 sfu. Both are far inside the spread between the two cycles the shape is built from, which is the error that actually matters and which this row does not publish.
/// * Persistence is carried for completeness and is worth nothing at any mission lead — fails when the weight is exp(-L/27) with L in days, so at the shortest mission the declared input range allows — half a year — today's flux contributes 0.114% and by one year it contributes 0.00013%. The term is right and it is inert: this row's answer is the window climatology to four decimal places for every lead a mission can ask about. It is kept because the relation is the study's, and removing it would make the row silently wrong for the short-lead use nothing in this tree currently makes.
/// * This moves the row AWAY from the MATLAB tool, deliberately — fails when the MATLAB tool answers 158.33 sfu for its own 2027 window by freezing its last 27-day rotation forecast and holding it flat; this row now answers 97.0 sfu for the same window and used to answer 114.8. The port therefore agrees with MATLAB LESS than it did. That is the intended direction: over the same span past maximum the two completed cycles ran at 91 sfu scaled onto cycle 25's amplitude, so the record puts MATLAB 74% high and the old answer 26% high, and this row within a few per cent of it. tools/mat_parity.py records the disagreement and its size so that it stays a decision rather than becoming a surprise.
pub const NODE_ID: &str = "sw_central_expectation";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xf466d7e9c4204c31;

pub fn evaluate(today: Ratio, lead: Time, epoch: Time) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : average the amplitude-scaled cycle analogue over the mission window -> Ratio
    // The shape, the period, both amplitudes and the handover between them are
    // env::solar_cycle_analogue's; this row composes the window mean of it.
    let climo: Ratio = Ratio::new(env::solar_cycle_analogue_mean(
        epoch.days(),
        epoch.days() + lead.days(),
    ));
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
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_central", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the answer is a weighted blend of today's F10.7 and a window mean of the analogue, so it cannot leave the interval between them. The analogue is bounded below by 0.324026 x 193.8580 = 62.8 sfu, its smallest shape on its smallest amplitude, and env_f107 by 60 because below 60 sfu has never been observed; the blend therefore floors at 60" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_central", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "env_f107's upper bound is 400, above which the exospheric temperature relation is extrapolated past the largest recorded daily value. The analogue cannot exceed cycle 25's own 81-day peak of 225.1 sfu, and a blend cannot exceed its larger input, so this bound catches a broken weight or a broken table rather than an extreme sky" });
    }
    Ok(answer)
}
