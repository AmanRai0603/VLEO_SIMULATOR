// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much room does the sustained Ap requirement have left?
///
/// `M_ap_long = closure(Ap_req_long, Ap_long, AtMost).margin`
///
/// Source: `noaa_swpc`
///
/// An achieved row restates the subsystem's conclusion on the side of the
/// comparison the closure reads. It computes nothing of its own, and it
/// exists rather than the closure reading sw_ap_design_long directly so that
/// the comparison binds two rows of the same shape at the same layer —
/// visible on the tree instead of an edge somebody has to trace.
///
/// # Assumptions
///
/// * It inherits every limitation of the row it restates, and a closure reading it sees none of them — fails when a margin is computed from this row against a capability. The centre beneath it is a cycle analogue scaled, beyond one cycle past cycle 25's maximum, by the mean amplitude of two completed cycles whose peaks differ by 41 per cent; the band is 1.28 sigma, which is the 90th percentile and not the 95 per cent the run is labelled; and the residuals it is a sigma of are skewed. None of that travels across the closure, and the margin looks like a clean number either way
/// * It restates the sustained level and not one of its two siblings — fails when somebody reads it as the other. The subsystem publishes three Ap conclusions — 26.70 sustained, 41.70 for the worst day of the design band, 158.38 for the one storm expected in the mission — and they answer three different questions. Restating the wrong one would move this closure from passing to failing or back without anything in the tree noticing
/// * Nothing compares this row with its requirement automatically — fails when a reader assumes the tree checks the closure. The pairing is a convention the matrix draws; `sense` is declared on the requirement row and the gate checks only that it is present. The Ap storm closure beside this one, l3_solar_req_03 against l3_solar_ach_03, currently FAILS at 158.38 against 150 and nothing in the tree says so
pub const NODE_ID: &str = "l3_solar_ach_04";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x1b206c2ef28b8732;

pub fn evaluate(ach: Ratio, req: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : compare achieved against required in the declared sense and return the signed fractional margin -> Ratio
    // The requirement declares sense "<=", so the achieved value must stay UNDER
    // the bound and the margin is (required - achieved) / required. Positive is
    // room; negative is a violation and its size. mission::closure is the twelve
    // KPI closures' own function rather than the arithmetic written out again,
    // because a second way of computing a margin is a second way of getting its
    // sign wrong -- and a sign error here still produces a plausible number.
    let m: Ratio = Ratio::new(mission::closure(req.get(), ach.get(), mission::Sense::AtMost).margin);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = m;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "M_ap_long", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < -10.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "M_ap_long", value: answer.get(), bound: -10.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a margin of -10 is a requirement exceeded by eleven times its own value. Below that the comparison has stopped being a design margin and become a sign that one side is in the wrong unit, and it should refuse rather than report a number nobody will read as a fraction" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "M_ap_long", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "1 is the whole of the requirement: an achieved value of zero against a positive bound. A margin above 1 under this sense would mean the achieved value is negative, which none of these drivers can be" });
    }
    Ok(answer)
}
