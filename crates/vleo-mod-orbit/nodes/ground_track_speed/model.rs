// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How fast does the sub-satellite point move over the ground?
///
/// `V_g = V*Re/r`
///
/// Source: `larson_wertz`
///
/// Not the orbital speed. The difference is what sets sensor dwell time and
/// therefore signal-to-noise.
pub const NODE_ID: &str = "orbit_ground_track_speed";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xe15473059019bed1;

pub fn evaluate(v: Velocity, r: Length) -> Result<Velocity, Fault> {
    // ---- HOLE 1 : scale the orbital speed by the ratio of Earth radius to orbital radius -> Velocity
    let vg: Velocity = orbit::ground_track_speed(v, r);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Velocity = vg;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "V_g", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 6000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "V_g", value: answer.get(), bound: 6000.0, edge: Edge::Lower, unit: Velocity::UNIT, reason: "below 6 km/s is outside this tool's altitude band" });
    }
    if answer.get() > 8000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "V_g", value: answer.get(), bound: 8000.0, edge: Edge::Upper, unit: Velocity::UNIT, reason: "the ground track can never move faster than the orbital speed" });
    }
    Ok(answer)
}
