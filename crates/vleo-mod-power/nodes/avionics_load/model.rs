// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What does the on-board computer, the harness electronics and the housekeeping draw?
///
/// `P_av = 35`
///
/// Source: `orbitt_case_c1`
pub const NODE_ID: &str = "pwr_avionics_load";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xc518f3a129765efc;

pub fn evaluate() -> Result<Power, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Power = match Power::from_unit(35.0, Unit::Watt) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "P_av", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Power = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "P_av", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 5.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "P_av", value: answer.get(), bound: 5.0, edge: Edge::Lower, unit: Power::UNIT, reason: "below 5 W no spacecraft avionics set runs" });
    }
    if answer.get() > 500.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "P_av", value: answer.get(), bound: 500.0, edge: Edge::Upper, unit: Power::UNIT, reason: "above 500 W the avionics dominate the bus, which no design in this class does" });
    }
    Ok(answer)
}
