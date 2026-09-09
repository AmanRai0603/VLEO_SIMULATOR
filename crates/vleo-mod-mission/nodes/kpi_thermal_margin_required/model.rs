// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What does the customer require for thermal margin?
///
/// `R_req = 10`
///
/// Source: `ecss_e_st_10_02`
pub const NODE_ID: &str = "kpi_thermal_margin_required";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xfb6c41d21c6e2ee6;

pub fn evaluate() -> Result<Temperature, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Temperature = match Temperature::from_unit(10.0, Unit::Kelvin) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "R_req", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Temperature = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "R_req", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "R_req", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Temperature::UNIT, reason: "a negative required margin is not a requirement" });
    }
    if answer.get() > 100.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "R_req", value: answer.get(), bound: 100.0, edge: Edge::Upper, unit: Temperature::UNIT, reason: "above 100 K the requirement is not one this design is written against" });
    }
    Ok(answer)
}
