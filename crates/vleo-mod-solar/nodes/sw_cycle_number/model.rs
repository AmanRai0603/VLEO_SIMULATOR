// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// Which numbered solar cycle is the mission epoch in?
///
/// `cyc(T) = 22 + count(cycle_starts <= T),  starts = 1997-01-15, 2008-12-01, 2019-12-01`
///
/// Source: `noaa_swpc`
///
/// The coarsest dated fact about the mission: which cycle it flies in.
/// Everything else that depends on the date depends on this first, because a
/// phase means nothing without a cycle to be a phase of.
///
/// # Assumptions
///
/// * The cycle boundaries are the record's three, and a fourth would need a source this bundle does not contain — fails when solar_cycles.csv holds cycles 23, 24 and 25 with starts 1997-01-15, 2008-12-01 and 2019-12-01 — the boundaries prf_cycles calls standard. The record begins mid-cycle-23 and ends mid-cycle-25, so cycle 22 and cycle 26 are outside it entirely. An epoch after cycle 26 begins would still return 25 here, because nothing in the data says when 26 starts, and a row that guessed would be inventing a boundary. The declared upper bound is what stops that being silent
/// * The epoch is past the record, so the phase is FOLDED with a mean cycle length rather than measured — fails when solar_cycles.csv gives cycle 25 a start of 2019-12-01 and an end of 2025-12-15 with a length of 6.04 years. That end and that length are artefacts of where the RECORD stops, not where the cycle stops: cycles 23 and 24 ran 11.88 and 11.00 years. The epoch, day 9862, is 382 days past that recorded end, so no cycle in the table contains it. The phase is therefore computed against the mean of the two COMPLETE cycles, 11.44 years, which is what prf_design's meanCycleAt does beyond its last cycle. If cycle 25 turns out short or long the phase moves, and every row reading it moves with it
pub const NODE_ID: &str = "sw_cycle_number";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xb536b9a649d52093;

pub fn evaluate(epoch: Time) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::sw_cycle_number. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::sw_cycle_number::evaluate(epoch.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "cyc")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "cyc", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 23.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "cyc", value: answer.get(), bound: 23.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the record's first cycle is 23 and it begins mid-cycle, so no epoch this tree allows can sit in an earlier one. A lower number means the counting started in the wrong place" });
    }
    if answer.get() > 26.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "cyc", value: answer.get(), bound: 26.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the record names three cycles and ends inside the third. 26 is one beyond what the data can place, and the bound exists so that an epoch far enough out to need cycle 27 refuses rather than quietly returning 25" });
    }
    Ok(answer)
}
