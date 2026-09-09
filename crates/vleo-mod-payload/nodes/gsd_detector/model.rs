// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What is the finest detail the detector can sample, whatever the optics?
///
/// `GSD = h*p/f`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "pay_gsd_detector";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xe24423a42f96f60e;

pub fn evaluate(h: Length, p: Length, f: Length) -> Result<Length, Fault> {
    // ---- HOLE 1 : project one pixel onto the ground through the focal length -> Length
    let g: Length = payload::detector_limited_gsd(h, p, f);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Length = g;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "GSD_p", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.001 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "GSD_p", value: answer.get(), bound: 0.001, edge: Edge::Lower, unit: Length::UNIT, reason: "below a millimetre the relation is being fed a focal length that is not physical" });
    }
    if answer.get() > 1000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "GSD_p", value: answer.get(), bound: 1000.0, edge: Edge::Upper, unit: Length::UNIT, reason: "above a kilometre no product in this design is being made" });
    }
    Ok(answer)
}
