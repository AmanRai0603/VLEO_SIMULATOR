// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What 81-day centred mean F10.7 should a density model be given at the mission epoch?
///
/// `F107A(epoch) = F107_cyc(phase(epoch))`
///
/// Source: `noaa_swpc`
///
/// The second driver every empirical density model wants, beside the daily
/// flux. At an epoch past the record it is a prediction rather than an
/// observation, and the prediction is the mean cycle at that phase.
///
/// # Assumptions
///
/// * It is a prediction, not a centred mean, because there is nothing to centre on — fails when the epoch is inside the record and a caller expects the measured value. The declared epoch is day 9862 and the record ends at day 9496, so this row is 366 days beyond any observation and 407 days beyond the last day that could have a full centred window. What it returns is the mean of cycles 23 and 24 at the same phase — a climatology with two samples behind it — and the difference between that and a real 81-day mean is a whole cycle's individuality. sw_cycle_repeatability puts the shape agreement between those two cycles at 0.770 and the peak disagreement at 28 per cent
/// * The mean-cycle bin is 217 days wide and this row calls it an 81-day mean — fails when the smoothing width matters to the caller. sw_mean_cycle_level bins a cycle into twenty phase bins, which for cycle 23 is 217 days and for cycle 24 is 201 — between two and three times the 81-day window this row is named for. So the number is smoother than a true F10.7A, and a density model given it sees less structure than it would from the real driver. In the direction that matters the error is conservative for a design point and wrong for a time history: it cannot reproduce a rotation, because it is already averaged over eight of them
/// * A single day's F10.7 strays from this by about twelve per cent, one standard deviation — fails when F10.7A is used where the daily value belongs. Measured over 10284 days the ratio of daily F10.7 to its own 81-day centred mean has mean 0.999340 and standard deviation 0.122210, and reaches 2.11 at the extreme — sw_f107a_ratio carries that figure. Handing a density model F10.7A for both of its two drivers loses every spike the atmosphere actually felt
pub const NODE_ID: &str = "sw_f107_81day";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xfef1984cfd53b87b;

pub fn evaluate(level: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : take the mean-cycle level at the epoch's phase as the 81-day mean a density model should be given -> Ratio
    // A pass-through, and the sheet says why at length: at an epoch past the
    // record there are no 81 days to centre on, so the honest F10.7A is the
    // phase-conditioned mean-cycle level. What this row adds is the question,
    // the declared smoothing width and the named density-model contract, not
    // arithmetic.
    let out: Ratio = level;
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = out;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107A", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107A", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the same floor as env_f107: below 60 sfu has never been observed and no relation reading F10.7 has support there" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107A", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the same ceiling as env_f107: above 400 sfu every consumer of F10.7 is extrapolating. An 81-day mean is smoother than the daily value and the largest binned mean-cycle level is 165.3, so this bound is unreachable by the relation and catches a broken input" });
    }
    Ok(answer)
}
