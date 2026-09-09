// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How long does the sensor have to collect photons from one ground sample?
///
/// `t = GSD/V_g`
///
/// Source: `larson_wertz`
///
/// The constraint low altitude imposes and higher orbits do not: the ground
/// moves under the sensor faster in angular terms, which partly gives back
/// the signal-to-noise the shorter range won.
pub const NODE_ID: &str = "pay_dwell_time";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xae7273b1409107fc;

pub fn evaluate(g: Length, v: Velocity) -> Result<Time, Fault> {
    // ---- HOLE 1 : divide the ground sample distance by the ground track speed -> Time
    let t: Time = payload::dwell_time(g, v);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Time = t;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "t_dwell", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 1e-9 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "t_dwell", value: answer.get(), bound: 1e-9, edge: Edge::Lower, unit: Time::UNIT, reason: "below a nanosecond no detector integrates" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "t_dwell", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Time::UNIT, reason: "above one second the ground has moved many samples during the integration" });
    }
    Ok(answer)
}
