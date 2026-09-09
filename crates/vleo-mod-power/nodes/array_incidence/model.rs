// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// At what mean angle does the sunlight strike the array over an orbit?
///
/// `theta_s = 25`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "pwr_array_incidence";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x13cf7541a46a5e68;

pub fn evaluate() -> Result<Angle, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Angle = match Angle::from_unit(25.0, Unit::Degree) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "theta_s", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Angle = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "theta_s", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "theta_s", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Angle::UNIT, reason: "zero is normal incidence, the best case and the limit" });
    }
    if answer.get() > 1.4835298641951802 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "theta_s", value: answer.get(), bound: 1.4835298641951802, edge: Edge::Upper, unit: Angle::UNIT, reason: "above 85 degrees the cosine loss leaves almost nothing and the array is not pointed at the Sun in any useful sense" });
    }
    Ok(answer)
}
