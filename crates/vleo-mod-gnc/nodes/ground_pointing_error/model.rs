// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How far off the ground does the pointing error put the aim point?
///
/// `e = sigma*d`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "gnc_ground_pointing_error";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xc988332d760c0455;

pub fn evaluate(s: Angle, d: Length) -> Result<Length, Fault> {
    // ---- HOLE 1 : multiply the pointing error by the slant range -> Length
    let e: Length = gnc::pointing_to_ground_error(s, d);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Length = e;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "e_gnd", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "e_gnd", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Length::UNIT, reason: "a ground error cannot be negative" });
    }
    if answer.get() > 100000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "e_gnd", value: answer.get(), bound: 100000.0, edge: Edge::Upper, unit: Length::UNIT, reason: "above 100 km the pointing is not usable for any payload here" });
    }
    Ok(answer)
}
