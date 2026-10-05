// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// Where in the solar cycle do the exceedances of the design Ap fall?
///
/// `P_exc(Ap_design) = median cycle phase of the days with Ap >= Ap_design`
///
/// Source: `noaa_swpc`
///
/// The median cycle phase of the days above sw_ap_design, on the 0-to-1 scale
/// sw_cycle_phase uses. At G3 it is 0.603 — the declining side, just past
/// maximum — and the declared epoch sits at 0.619.
///
/// # Assumptions
///
/// * A median with no spread beside it, and the spread is the part a schedule needs — fails when 0.603 is read as when exceedances happen. It is where the middle one happened. At G3 the eight days run from phase 0.294 to 0.736 — a band nearly half a cycle wide, about five years — and they fall in phase deciles 0.2-0.3, 0.3-0.4, 0.5-0.6 twice, 0.6-0.7 three times and 0.7-0.8 once. Half of them sit between 0.2 and 0.6. A design that reads only this row plans for a date; a design that reads the assumption plans for a five-year window
/// * It is stable across G level, which means it is the cycle rather than the threshold — fails when the median is expected to move with the bound. It barely does: 0.571692 at the G1 bound, 0.576302 at G2 and 0.602697 at G3, a drift of three hundredths across a factor of nearly three in threshold. So the timing is a property of the solar cycle — disturbed days cluster after maximum, on the declining phase, where coronal holes dominate — and not of where the line is drawn. That stability is the reason the row is worth having: it would be meaningless if it tracked the threshold
/// * Eight days at G3, and a median of eight numbers is a coarse instrument — fails when precision is read into 0.603. The sample is 131 days at G1, 35 at G2 and 8 at G3, so at the declared level the median sits between the fourth and fifth of eight values. Moving one storm moves the median by a few hundredths. The three levels agreeing to within 0.03 on samples of 8, 35 and 131 is better evidence for the timing than any one of them alone
/// * Phase is folded on three cycles, one of which is incomplete, and 273 days are missing from the risky part — fails when the phase scale is assumed uniform. Cycle 25's end in solar_cycles.csv is the record's end rather than a real minimum, so phases inside it are computed against a cycle whose length is not yet known; cycles 23 and 24 differ in length by 8 per cent, so equal phase is unequal time. And the 2017 gap sits at phase 0.74 to 0.80 of cycle 24, just past the upper edge of the G3 exceedance band — so whatever fell there is absent from this median and from sw_exceedance_rate's count
pub const NODE_ID: &str = "sw_exceedance_phase";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x293957a43390f2f3;

pub fn evaluate(ap_design: Ratio) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::sw_exceedance_phase. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::sw_exceedance_phase::evaluate(ap_design.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "P_exc")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "P_exc", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "P_exc", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "cycle phase runs 0 at minimum to 1 at the next minimum, so 0 is the floor by definition. A median at 0 would mean every exceedance fell on the first day of a cycle" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "P_exc", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "1 is the end of a cycle by the same definition. A median at 1 would mean every exceedance fell on the last day of one" });
    }
    Ok(answer)
}
