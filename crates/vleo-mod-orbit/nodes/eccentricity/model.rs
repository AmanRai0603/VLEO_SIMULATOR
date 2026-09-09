// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How circular is the orbit?
///
/// `e = 0`
///
/// Source: `orbitt_case_c1`
///
/// Moving this reaches 3 nodes. Moving the altitude reaches 54. The
/// difference is the answer to how much of the design a change just
/// invalidated.
pub const NODE_ID: &str = "orbit_eccentricity";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x4d8312be8d106a5c;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(0.0, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "e", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "e", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "e", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a negative eccentricity is not an orbit" });
    }
    if answer.get() > 0.05 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "e", value: answer.get(), bound: 0.05, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 0.05 the perigee of a 250 km orbit is below 200 km, where the drag per revolution stops being a small perturbation and the circular-orbit relations used throughout this tree no longer apply" });
    }
    Ok(answer)
}
