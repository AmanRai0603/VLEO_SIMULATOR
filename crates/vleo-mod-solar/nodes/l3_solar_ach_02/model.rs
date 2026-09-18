// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much room does the single-day F10.7 requirement have left?
///
/// `M_f107_short = closure(F107_req_short, F107_short, AtMost).margin`
///
/// Source: `noaa_swpc`
///
/// The single-day half of the F10.7 closure. Its partner, l3_solar_req_02,
/// commits the design to surviving 350 sfu — above the largest daily value
/// in the record — and this says what the window's own band actually
/// reaches.
///
/// # Assumptions
///
/// * It inherits every limitation of the row it restates, and a closure reading it sees none of them — fails when a margin is computed from this row against a capability. The centre beneath it is a cycle analogue at the mission's own epoch, scaled beyond one cycle past cycle 25's maximum by the mean amplitude of two completed cycles whose peaks differ by 41 per cent; the band around it is 1.28 sigma, the 90th percentile, while the run is labelled 95 per cent; and the daily term stacked on top is a separate one-sided percentile, so the combination is nearer a 1-in-100 day than a 1-in-20 one. None of that travels across the closure, and the margin looks like a clean number either way
/// * It restates the single-day level and not one of its two siblings — fails when somebody reads it as the sustained level or as the persistence drift. The three are 124.14, 104.07 and 200.14 — all fluxes, same unit, same declared domain — so nothing in the tree would catch the substitution, and each would report a different margin against the same requirement
/// * Nothing compares this row with its requirement automatically — fails when a reader assumes the tree checks the closure. The pairing is a convention the matrix draws; `sense` is declared on the requirement row and the gate checks only that it is present. The Ap storm closure in this same group, l3_solar_req_03 against l3_solar_ach_03, currently FAILS at 158.38 against 150 and nothing in the tree says so
pub const NODE_ID: &str = "l3_solar_ach_02";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xdc32df008176d18a;

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
        return Err(Fault::Degenerate { node: NODE_ID, field: "M_f107_short", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < -10.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "M_f107_short", value: answer.get(), bound: -10.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a margin of -10 is a requirement exceeded by eleven times its own value. Below that the comparison has stopped being a design margin and become a sign that one side is in the wrong unit, and it should refuse rather than report a number nobody will read as a fraction" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "M_f107_short", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "1 is the whole of the requirement: an achieved value of zero against a positive bound. A margin above 1 under this sense would mean the achieved value is negative, which none of these drivers can be" });
    }
    Ok(answer)
}
