// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much does the structure move between the sensor and the payload over an orbit?
///
/// `sig_thm = 20`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "gnc_thermal_distortion";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x7dbe861ec4644a2a;

pub fn evaluate() -> Result<Angle, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Angle = match Angle::from_unit(20.0, Unit::Arcsecond) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "sig_thm", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Angle = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "sig_thm", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "sig_thm", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Angle::UNIT, reason: "a distortion cannot be negative" });
    }
    if answer.get() > 0.017453292519943295 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "sig_thm", value: answer.get(), bound: 0.017453292519943295, edge: Edge::Upper, unit: Angle::UNIT, reason: "above one degree the structure is not stable enough for any payload in this design" });
    }
    Ok(answer)
}
