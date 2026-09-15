// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What F10.7 does the average solar cycle show at this phase?
///
/// `F107_cyc(phase) = mean over complete cycles of F10.7 at that phase`
///
/// Source: `noaa_swpc`
///
/// The climatology that knows where in the cycle it is.
/// sw_central_expectation currently hands over to the record's unconditional
/// mean of 114.84 sfu because no date was available; at the declared epoch's
/// phase of 0.6194 this row says 108.14 instead. That difference — 6.7 sfu
/// — is what having a date buys. The table is INTERPOLATED between bin
/// centres, so the answer at a phase is not the nearest bin's mean: 0.6194
/// sits between the 0.575 bin at 126.23 and the 0.625 bin at 105.84, nine
/// tenths of the way toward the second.
///
/// # Assumptions
///
/// * Two cycles, stacked on phase, in twenty bins — fails when cycles 23 and 24 are the only complete ones in the record, so every bin is the mean of two cycles and nothing more. Two samples cannot separate a cycle's shape from a cycle's individuality: cycle 23 peaked at 196 sfu and cycle 24 at 146, a 34% difference, and this row averages them into one curve that matches neither. The bins hold 358 to 418 days each except the 0.775 bin, which holds 217 because the two cycles' lengths differ and the stacking leaves it thin
/// * It is a mean and not a band — fails when half the days at any phase sit above this line. It is the CENTRE for a design value, and sw_uncertainty_growth supplies the spread that makes it safe. Sizing anything on this row alone would be sizing on the average day of the average cycle, which is the one thing a mission is guaranteed not to get
/// * The curve is not symmetric and the asymmetry is real — fails when the mean rises from 71.8 sfu at phase 0.025 to 165.3 at 0.425 and falls to 67.7 by 0.975 — a fast rise and a slow decline, which is the known shape of a solar cycle and not a binning artefact. A design at phase 0.2 and one at phase 0.8 are both 'mid-cycle' and are owed 111 and 76 sfu respectively
pub const NODE_ID: &str = "sw_mean_cycle_level";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x9c8ab20dbc4cf7b2;

pub fn evaluate(phase: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : read the phase-binned mean F10.7 of the complete cycles at this phase -> Ratio
    // The mean F10.7 of cycles 23 and 24 stacked on phase, twenty bins, counted on
    // solar-weather@2026.09.14. x is the bin centre.
    //
    // The curve is asymmetric — 71.8 sfu at phase 0.025 rising to 165.3 at 0.425 and
    // falling to 67.7 by 0.975 — and that is the known shape of a solar cycle, a
    // fast rise and a slow decline. It is not smoothed. Two cycles is all the record
    // has, and the sheet says what that costs: cycle 23 peaked at 196 sfu and 24 at
    // 146, so this curve matches neither.
    use vleo_core::math::Table1;
    const LEVEL: Table1 = Table1 {
        x: &[
            0.025, 0.075, 0.125, 0.175, 0.225, 0.275, 0.325, 0.375, 0.425, 0.475, 0.525, 0.575,
            0.625, 0.675, 0.725, 0.775, 0.825, 0.875, 0.925, 0.975,
        ],
        y: &[
            71.7632, 83.8947, 99.2177, 111.0502, 135.6555, 159.6603, 145.3702, 146.7273, 165.2799,
            160.3301, 135.3182, 126.2321, 105.8445, 95.5084, 86.9916, 80.9171, 76.2094, 71.0478,
            71.5383, 67.6538,
        ],
    };
    let out: Ratio = Ratio::new(LEVEL.at(phase.get()));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = out;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107_cyc", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_cyc", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the same floor as env_f107: below 60 sfu has never been observed and no relation reading F10.7 has support there" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_cyc", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the same ceiling as env_f107: above 400 sfu every consumer of F10.7 is extrapolating. The largest binned mean is 165.3, so this bound is unreachable by the relation and catches a broken table" });
    }
    Ok(answer)
}
