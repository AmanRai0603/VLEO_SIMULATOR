// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What angle does the Sun make with the orbit plane? It decides eclipse duration and therefore both the power and the thermal design.
///
/// `beta = 30`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "orbit_beta_angle";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xde85f6ead6e478db;

pub fn evaluate() -> Result<Angle, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Angle = match Angle::from_unit(30.0, Unit::Degree) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "beta", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Angle = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "beta", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < -1.5707963267948966 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "beta", value: answer.get(), bound: -1.5707963267948966, edge: Edge::Lower, unit: Angle::UNIT, reason: "the angle between the orbit plane and the Sun is defined on -90..90" });
    }
    if answer.get() > 1.5707963267948966 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "beta", value: answer.get(), bound: 1.5707963267948966, edge: Edge::Upper, unit: Angle::UNIT, reason: "the angle between the orbit plane and the Sun is defined on -90..90" });
    }
    Ok(answer)
}
