// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What is the highest level the cycle analogue reaches during the mission?
///
/// `F107_window_peak(T_e, L) = max over t in [T_e, T_e+L] of A(t)`
///
/// Source: `noaa_swpc`
///
/// The companion to sw_central_expectation, and the number a design that must
/// SURVIVE the window wants rather than the one that describes it. The centre
/// is a mean over the window, which for a five-year mission averages a busy
/// opening against a quiet end and reports neither. This publishes the
/// busiest sustained level inside the same window, from the same analogue, so
/// the two can be read together.
///
/// # Assumptions
///
/// * This is the peak of the EXPECTATION, not a peak of the sky — fails when the analogue is a mean over completed cycles of a smoothed level. The record's daily F10.7 reaches 343 sfu and this row's ceiling anywhere is 225.1; a single rotation can exceed this row's answer by a factor of two and nothing here is wrong when it does. A design wanting a level it will not see exceeded needs this plus an excursion, which is sw_uncertainty_growth, or the 95th percentile the design row already composes.
/// * Outside cycle 25 the amplitude is the mean of TWO completed cycles, and their spread is a factor of 1.41 — fails when this row inherits the assumption from the analogue it reads. Cycle 23 peaked at 226.8 sfu and cycle 24 at 160.9, so the 193.9 used for every future cycle is the midpoint of two numbers 41% apart — and because this row reports a MAXIMUM, a long window's answer is usually exactly that amplitude rather than something averaged near it. The fifteen-year case returns 193.858 sfu, which is the assumption showing through undiluted. If the next cycle runs like cycle 23 this row is 17% low.
/// * A maximum is not a duration — fails when the row says how high the expected level gets and says nothing about how long it stays there. A window whose peak is a brief crossing of a cycle maximum and one that sits at maximum for two years return the same number. Anything sizing a propellant budget or a lifetime needs the integral, which is sw_central_expectation, and anything sizing a thermal or power case may need neither.
pub const NODE_ID: &str = "sw_window_peak_level";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xbfee2dafc8999dae;

pub fn evaluate(lead: Time, epoch: Time) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : take the largest value the cycle analogue reaches anywhere inside the mission window -> Ratio
    // The shape, the period, both amplitudes, the handover between them and the
    // exhaustive candidate list are env::solar_cycle_analogue_max's. This row
    // composes it over the window; the sibling composes the mean of the same
    // analogue, which is why the two cannot disagree about the cycle.
    let peak: Ratio = Ratio::new(env::solar_cycle_analogue_max(
        epoch.days(),
        epoch.days() + lead.days(),
    ));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = peak;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107_window_peak", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_window_peak", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the analogue's smallest value anywhere is its smallest shape on its smallest amplitude, 0.324026 x 193.8580 = 62.8 sfu, and a maximum over any window is at least that. 60 is env_f107's floor, below which no F10.7 has been observed, so an answer under it means the shape table or an amplitude has been corrupted rather than that the Sun is quiet" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_window_peak", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the analogue's largest value anywhere is 1.000000 x 225.1358 = 225.1 sfu, cycle 25's own 81-day peak, because the shape is normalised to one at a cycle maximum. 400 is env_f107's ceiling and is unreachable by this relation; it catches a broken amplitude rather than an extreme sky" });
    }
    Ok(answer)
}
