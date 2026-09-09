// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How long does one revolution take?
///
/// `T = 2*pi*sqrt(a^3/mu)`
///
/// Source: `vallado2013`
pub const NODE_ID: &str = "orbit_period";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xa1b930e9f6077007;

pub fn evaluate(r: Length) -> Result<Time, Fault> {
    // ---- HOLE 1 : apply Kepler's third law at the circular radius -> Time
    let t: Time = orbit::period(r);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Time = t;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "T_orb", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 4980.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_orb", value: answer.get(), bound: 4980.0, edge: Edge::Lower, unit: Time::UNIT, reason: "a period below 5000 s corresponds to an orbit inside the Earth" });
    }
    if answer.get() > 6240.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_orb", value: answer.get(), bound: 6240.0, edge: Edge::Upper, unit: Time::UNIT, reason: "6200 s corresponds to about 1000 km, outside this tool's band" });
    }
    Ok(answer)
}
