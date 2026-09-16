// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What single-day Ap does the record say this mission will present?
///
/// `Ap_ach_short = sw_ap_design_short`
///
/// Source: `noaa_swpc`
///
/// An achieved row restates the subsystem's conclusion on the side of the
/// comparison the closure reads. It computes nothing of its own, and it
/// exists rather than the closure reading sw_ap_design_short directly so that
/// the comparison binds two rows of the same shape at the same layer —
/// visible on the tree instead of an edge somebody has to trace.
///
/// # Assumptions
///
/// * It inherits every limitation of the row it restates, and a closure reading it sees none of them — fails when a margin is computed from this row against a capability. The centre beneath it is a cycle analogue scaled, beyond one cycle past cycle 25's maximum, by the mean amplitude of two completed cycles whose peaks differ by 41 per cent; the band is 1.28 sigma, which is the 90th percentile and not the 95 per cent the run is labelled; and the residuals it is a sigma of are skewed. None of that travels across the closure, and the margin looks like a clean number either way
/// * It restates the single-day level and not one of its two siblings — fails when somebody reads it as the other. The subsystem publishes three Ap conclusions — 26.70 sustained, 90.55 for the worst day of the design band, 158.38 for the one storm expected in the mission — and they answer three different questions. Restating the wrong one would move this closure from passing to failing or back without anything in the tree noticing
/// * Nothing compares this row with its requirement automatically — fails when a reader assumes the tree checks the closure. The pairing is a convention the matrix draws; `sense` is declared on the requirement row and the gate checks only that it is present. TWO closures in this group now fail — this one at 90.55 against 80, and the Ap storm at 158.38 against 150 — and nothing in the tree says so about either. A failing closure is invisible to every machine check in this repository
pub const NODE_ID: &str = "l3_solar_ach_05";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x4f467829c475f841;

pub fn evaluate(conclusion: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : restate the subsystem's conclusion on the achieved side of the closure -> Ratio
    // A crossing carries and so does a closure's achieved side; neither computes.
    // The one thing that can go wrong here is that the restatement alters what it
    // is handed — a stray factor, an unasked-for unit conversion, a clamp
    // inherited from the wrong row — and both sides would still look plausible.
    //
    // The declared range is sw_ap_design_short's own, restated so a reader of the
    // closure sees the limit without opening the producing row. It therefore
    // guards nothing this line can break, which is correct: an achieved row that
    // narrowed the range it carried would be changing the answer.
    let ach: Ratio = conclusion;
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = ach;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Ap_ach_short", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_ach_short", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the same floor as the row it restates: Ap floors at zero, and a perfectly quiet day is Ap 0" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_ach_short", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the same ceiling as the row it restates: 400 is the top of the Ap index itself, and a value above it is not a geomagnetic index at all" });
    }
    Ok(answer)
}
