// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What temperature are the surfaces the atmosphere strikes?
///
/// `T_w = 300`
///
/// Source: `sentman1961`
pub const NODE_ID: &str = "env_wall_temperature";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xc5fe6d3d83041757;

pub fn evaluate() -> Result<Temperature, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Temperature = match Temperature::from_unit(300.0, Unit::Kelvin) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "T_w", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Temperature = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "T_w", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 150.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_w", value: answer.get(), bound: 150.0, edge: Edge::Lower, unit: Temperature::UNIT, reason: "below 150 K no external surface of this design is predicted to fall, and the accommodation fit has no data there" });
    }
    if answer.get() > 500.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_w", value: answer.get(), bound: 500.0, edge: Edge::Upper, unit: Temperature::UNIT, reason: "above 500 K the surface materials are outside their qualification range, so a drag figure computed there is meaningless anyway" });
    }
    Ok(answer)
}
