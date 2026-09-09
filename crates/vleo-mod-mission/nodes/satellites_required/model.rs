// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How many satellites would the revisit requirement need?
///
/// `N = A_earth/(t_target*W*V_g)`
///
/// Source: `larson_wertz`
///
/// The inverse question, and the one a concept trade actually asks.
pub const NODE_ID: &str = "mis_satellites_required";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x88ada5e2e1cc89a1;

pub fn evaluate(t: Time, w: Length, v: Velocity) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : invert the revisit relation for the satellite count -> Ratio
    let n: Ratio = Ratio::new(mission::satellites_for_revisit(t, w, v));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = n;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "N_req", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "N_req", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a satellite count cannot be negative" });
    }
    if answer.get() > 100000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "N_req", value: answer.get(), bound: 100000.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 100000 the concept is not the one being designed" });
    }
    Ok(answer)
}
