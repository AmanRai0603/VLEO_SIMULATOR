// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// At what altitude does the constellation fly? This is the one parameter the whole architecture is shaped around.
///
/// `h = 250`
///
/// Source: `orbitt_case_c1`
///
/// Move this one value and 54 other nodes become stale. That is the
/// requirement the tool exists to satisfy.
pub const NODE_ID: &str = "orbit_altitude";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x99fff4489598eabc;

pub fn evaluate() -> Result<Length, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Length = match Length::from_unit(250.0, Unit::Kilometre) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "h", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Length = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "h", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 150000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "h", value: answer.get(), bound: 150000.0, edge: Edge::Lower, unit: Length::UNIT, reason: "below 150 km the flow stops being free-molecular for a body of this size and every aerodynamic node in the tree is outside its envelope" });
    }
    if answer.get() > 450000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "h", value: answer.get(), bound: 450000.0, edge: Edge::Upper, unit: Length::UNIT, reason: "above 450 km there is not enough air for the air-breathing concept to be the reason for the design" });
    }
    Ok(answer)
}
