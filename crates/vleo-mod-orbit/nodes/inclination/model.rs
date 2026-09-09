// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What is the angle between the orbit plane and the equator?
///
/// `i = 96.6`
///
/// Source: `orbitt_case_c1`
pub const NODE_ID: &str = "orbit_inclination";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x0374723eea6bec80;

pub fn evaluate() -> Result<Angle, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Angle = match Angle::from_unit(96.6, Unit::Degree) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "i", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Angle = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "i", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "i", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Angle::UNIT, reason: "inclination is defined on 0..180 degrees" });
    }
    if answer.get() > 3.141592653589793 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "i", value: answer.get(), bound: 3.141592653589793, edge: Edge::Upper, unit: Angle::UNIT, reason: "inclination is defined on 0..180 degrees" });
    }
    Ok(answer)
}
