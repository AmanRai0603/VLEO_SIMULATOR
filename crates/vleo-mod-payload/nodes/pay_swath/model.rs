// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How wide a strip does one optical pass image?
///
/// `W = GSD*N_x`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "pay_swath";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x591f9bd891c5d9e6;

pub fn evaluate(g: Length, n: Ratio) -> Result<Length, Fault> {
    // ---- HOLE 1 : multiply the ground sample distance by the across-track pixel count -> Length
    let w: Length = payload::optical_swath(g, n.get());
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Length = w;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "W_o", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 100.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "W_o", value: answer.get(), bound: 100.0, edge: Edge::Lower, unit: Length::UNIT, reason: "below 100 m the swath is not a swath" });
    }
    if answer.get() > 1000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "W_o", value: answer.get(), bound: 1000000.0, edge: Edge::Upper, unit: Length::UNIT, reason: "above 1000 km the field of view is not one this telescope has" });
    }
    Ok(answer)
}
