// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much noise does the ground antenna see?
///
/// `T_a = 60`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "com_gs_noise_temperature";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x525e5c2d6e6e27fd;

pub fn evaluate() -> Result<Temperature, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Temperature = match Temperature::from_unit(60.0, Unit::Kelvin) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "T_a", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Temperature = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "T_a", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 10.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_a", value: answer.get(), bound: 10.0, edge: Edge::Lower, unit: Temperature::UNIT, reason: "below 10 K requires cryogenic cooling that no commercial station in this network has" });
    }
    if answer.get() > 300.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_a", value: answer.get(), bound: 300.0, edge: Edge::Upper, unit: Temperature::UNIT, reason: "above 300 K the antenna is looking at the ground" });
    }
    Ok(answer)
}
