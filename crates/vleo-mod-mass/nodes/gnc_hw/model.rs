// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What do the sensors, wheels and torque rods weigh?
///
/// `m_gnc = 11`
///
/// Source: `orbitt_case_c1`
pub const NODE_ID: &str = "mass_gnc_hw";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x08d859aff48c005c;

pub fn evaluate() -> Result<Mass, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Mass = match Mass::from_unit(11.0, Unit::Kilogram) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "m_gnc", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Mass = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "m_gnc", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.5 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "m_gnc", value: answer.get(), bound: 0.5, edge: Edge::Lower, unit: Mass::UNIT, reason: "below half a kilogram no sensor and actuator set exists" });
    }
    if answer.get() > 200.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "m_gnc", value: answer.get(), bound: 200.0, edge: Edge::Upper, unit: Mass::UNIT, reason: "above 200 kg the control hardware is not in this class" });
    }
    Ok(answer)
}
