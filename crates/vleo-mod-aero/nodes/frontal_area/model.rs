// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What area does the flow actually see?
///
/// `A = pi*D^2/4 + A_app`
///
/// Source: `orbitt_case_c1`
pub const NODE_ID: &str = "aero_frontal_area";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x9ede9c56e1a0d5b8;

pub fn evaluate(d: Length, a_app: Area) -> Result<Area, Fault> {
    // ---- HOLE 1 : add the circular body cross-section to the appendage area -> Area
    let a: Area = Area::new(0.25 * pmath::PI * d.get() * d.get() + a_app.get());
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Area = a;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "A", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.005 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "A", value: answer.get(), bound: 0.005, edge: Edge::Lower, unit: Area::UNIT, reason: "below 50 cm2 there is no vehicle" });
    }
    if answer.get() > 20.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "A", value: answer.get(), bound: 20.0, edge: Edge::Upper, unit: Area::UNIT, reason: "above 20 m2 nothing in this design closes" });
    }
    Ok(answer)
}
