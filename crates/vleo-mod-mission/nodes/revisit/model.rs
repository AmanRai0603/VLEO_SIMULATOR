// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How often does the constellation come back to a given place?
///
/// `t = A_earth/(N*W*V_g)`
///
/// Source: `larson_wertz`
///
/// # Assumptions
///
/// * Uniform coverage of the whole globe, no latitude weighting — fails when a Sun-synchronous constellation revisits the poles far more often than the equator, so this mean is optimistic at low latitudes and pessimistic at high ones
pub const NODE_ID: &str = "mis_revisit";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xa655d3aca1c941c0;

pub fn evaluate(w: Length, v: Velocity, n: Ratio) -> Result<Time, Fault> {
    // ---- HOLE 1 : divide the Earth's surface area by the total swath sweep rate of the constellation -> Time
    let t: Time = mission::mean_revisit_time(w, v, n.get());
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Time = t;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "t_rev", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "t_rev", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Time::UNIT, reason: "a revisit time cannot be negative" });
    }
    if answer.get() > 8640000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "t_rev", value: answer.get(), bound: 8640000.0, edge: Edge::Upper, unit: Time::UNIT, reason: "above 100 days the constellation does not provide a service" });
    }
    Ok(answer)
}
