// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// Through what potential is the ion beam accelerated?
///
/// `V_b = 300`
///
/// Source: `romano2021`
pub const NODE_ID: &str = "prop_beam_voltage";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xbc57a56e9a94da5a;

pub fn evaluate() -> Result<Voltage, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Voltage = match Voltage::from_unit(300.0, Unit::Volt) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "V_b", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Voltage = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "V_b", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 50.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "V_b", value: answer.get(), bound: 50.0, edge: Edge::Lower, unit: Voltage::UNIT, reason: "below 50 V the exhaust velocity is too low to produce useful thrust from this mass flow" });
    }
    if answer.get() > 2000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "V_b", value: answer.get(), bound: 2000.0, edge: Edge::Upper, unit: Voltage::UNIT, reason: "above 2000 V grid erosion by atomic oxygen becomes the life-limiting mechanism, and the design has no demonstrated basis" });
    }
    Ok(answer)
}
