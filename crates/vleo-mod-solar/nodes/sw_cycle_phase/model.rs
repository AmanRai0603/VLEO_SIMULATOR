// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How far through its solar cycle is the mission epoch?
///
/// `phase(T) = (T - start_of_cycle) / mean_cycle_length`
///
/// Source: `noaa_swpc`
///
/// Zero at the cycle's start and one at its end. This is the row the whole
/// cycle-dependent half of the subsystem turns on: at the declared epoch it
/// is 0.619, which is past maximum on the declining side — the storm-rich
/// phase.
///
/// # Assumptions
///
/// * The epoch is past the record, so the phase is FOLDED with a mean cycle length rather than measured — fails when solar_cycles.csv gives cycle 25 a start of 2019-12-01 and an end of 2025-12-15 with a length of 6.04 years. That end and that length are artefacts of where the RECORD stops, not where the cycle stops: cycles 23 and 24 ran 11.88 and 11.00 years. The epoch, day 9862, is 382 days past that recorded end, so no cycle in the table contains it. The phase is therefore computed against the mean of the two COMPLETE cycles, 11.44 years, which is what prf_design's meanCycleAt does beyond its last cycle. If cycle 25 turns out short or long the phase moves, and every row reading it moves with it
/// * It is a fraction of a cycle and not a measure of activity — fails when phase 0.619 does not mean 61.9% of anything physical. The cycle is not symmetric — the record's mean F10.7 by phase rises from 72 sfu at phase 0.025 to about 165 at 0.425 and falls to 68 by 0.975, so the rise is faster than the decline and equal phase steps are not equal activity steps. sw_mean_cycle_level is the row that turns a phase into a flux; reading the phase as a proxy for activity would get the asymmetry backwards
/// * It cannot exceed one, and an epoch late in an unobserved cycle would want it to — fails when the fold divides elapsed time by a mean length, so an epoch more than 11.44 years past 2019-12-01 gives a phase above 1 and the declared bound refuses it. That is correct rather than convenient: past one cycle length the right answer is a new cycle number, and this row cannot supply one because the data does not say when cycle 26 begins
pub const NODE_ID: &str = "sw_cycle_phase";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x8a24d999e8437404;

pub fn evaluate(epoch: Time) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::sw_cycle_phase. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::sw_cycle_phase::evaluate(epoch.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "phase")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "phase", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "phase", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a phase is a fraction of the way through a cycle and cannot be negative. A negative value means the epoch precedes the cycle it was assigned to" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "phase", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "one is the end of the cycle. Above it the epoch belongs to the next cycle, and this row cannot name it because the record does not contain its start — so it refuses instead of folding round silently" });
    }
    Ok(answer)
}
