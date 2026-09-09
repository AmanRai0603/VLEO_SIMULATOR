// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What is the finest detail the optics can resolve, whatever the detector?
///
/// `GSD = 1.22*lambda*h/D`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "pay_gsd_diffraction";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x61dfbce5e713c72c;

pub fn evaluate(h: Length, lam: Length, d: Length) -> Result<Length, Fault> {
    // ---- HOLE 1 : apply the Rayleigh criterion at the flight altitude -> Length
    let g: Length = payload::diffraction_limited_gsd(h, lam, d);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Length = g;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "GSD_d", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.001 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "GSD_d", value: answer.get(), bound: 0.001, edge: Edge::Lower, unit: Length::UNIT, reason: "below a millimetre the relation is being fed an aperture that is not physical" });
    }
    if answer.get() > 1000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "GSD_d", value: answer.get(), bound: 1000.0, edge: Edge::Upper, unit: Length::UNIT, reason: "above a kilometre no product in this design is being made" });
    }
    Ok(answer)
}
