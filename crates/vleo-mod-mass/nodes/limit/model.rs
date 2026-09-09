// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What is the mass the launch slot or the bus allows?
///
/// `m_lim = 250`
///
/// Source: `orbitt_case_c1`
pub const NODE_ID: &str = "mass_limit";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x57b8e54dee63c323;

pub fn evaluate() -> Result<Mass, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Mass = match Mass::from_unit(250.0, Unit::Kilogram) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "m_lim", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Mass = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "m_lim", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 5.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "m_lim", value: answer.get(), bound: 5.0, edge: Edge::Lower, unit: Mass::UNIT, reason: "below 5 kg no multipayload mission fits" });
    }
    if answer.get() > 5000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "m_lim", value: answer.get(), bound: 5000.0, edge: Edge::Upper, unit: Mass::UNIT, reason: "above 5 tonnes the launch and the class are not the ones costed here" });
    }
    Ok(answer)
}
