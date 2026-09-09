// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How large is the opening the intake presents to the flow?
///
/// `A_in = 0.2`
///
/// Source: `romano2021`
///
/// Both the numerator and part of the denominator of the closure. A bigger
/// mouth collects more and drags more, which is why there is an optimum
/// rather than a maximum.
pub const NODE_ID: &str = "prop_intake_area";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xbdb4e7749f52fb6c;

pub fn evaluate() -> Result<Area, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Area = match Area::from_unit(0.2, Unit::SquareMetre) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "A_in", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Area = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "A_in", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.01 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "A_in", value: answer.get(), bound: 0.01, edge: Edge::Lower, unit: Area::UNIT, reason: "below 100 cm2 the collected flow is below the ionisation threshold of any thruster considered" });
    }
    if answer.get() > 5.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "A_in", value: answer.get(), bound: 5.0, edge: Edge::Upper, unit: Area::UNIT, reason: "above 5 m2 the mouth is itself more drag than the thrust it can produce, at every altitude in the band" });
    }
    Ok(answer)
}
