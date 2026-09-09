// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What temperature does the collected gas reach in the collection chamber?
///
/// `T_c = 600`
///
/// Source: `romano2021`
///
/// Drives the mean thermal speed, and therefore back-flow: a hotter chamber
/// leaks harder.
pub const NODE_ID: &str = "env_chamber_temperature";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x92f006e4ba0d21a3;

pub fn evaluate() -> Result<Temperature, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Temperature = match Temperature::from_unit(600.0, Unit::Kelvin) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "T_c", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Temperature = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "T_c", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 300.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_c", value: answer.get(), bound: 300.0, edge: Edge::Lower, unit: Temperature::UNIT, reason: "the chamber cannot be colder than the walls that bound it" });
    }
    if answer.get() > 1500.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_c", value: answer.get(), bound: 1500.0, edge: Edge::Upper, unit: Temperature::UNIT, reason: "above 1500 K the assumption that the chamber gas is thermalised and un-ionised stops holding" });
    }
    Ok(answer)
}
