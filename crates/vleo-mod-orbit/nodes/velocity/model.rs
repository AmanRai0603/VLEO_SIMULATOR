// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How fast is the spacecraft moving through the atmosphere?
///
/// `V = sqrt(mu/r)`
///
/// Source: `vallado2013`
///
/// # Assumptions
///
/// * Two-body, circular — fails when at e = 0.05 the speed varies by 5% around the orbit and a drag figure computed at the mean underestimates the perigee pass
pub const NODE_ID: &str = "orbit_velocity";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xf3181e3719f1fb4a;

pub fn evaluate(r: Length) -> Result<Velocity, Fault> {
    // ---- HOLE 1 : apply the circular two-body speed relation -> Velocity
    let v: Velocity = orbit::circular_velocity(r);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Velocity = v;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "V", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 7000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "V", value: answer.get(), bound: 7000.0, edge: Edge::Lower, unit: Velocity::UNIT, reason: "below 7 km/s corresponds to an orbit above 1000 km, outside this tool's band" });
    }
    if answer.get() > 8200.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "V", value: answer.get(), bound: 8200.0, edge: Edge::Upper, unit: Velocity::UNIT, reason: "8.2 km/s exceeds the surface circular speed and is not reachable" });
    }
    Ok(answer)
}
