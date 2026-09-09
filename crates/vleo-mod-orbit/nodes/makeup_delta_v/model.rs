// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much delta-v a year does holding this altitude cost?
///
/// `dv = (D/m)*t`
///
/// Source: `vallado2013`
///
/// This is the number an air-breathing system exists to make free. A
/// stored-propellant design at 250 km spends several kilometres per second a
/// year on it.
pub const NODE_ID: &str = "orbit_makeup_delta_v";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xa3bb0f9792348d54;

pub fn evaluate(a_drag: Acceleration) -> Result<Velocity, Fault> {
    // ---- HOLE 1 : integrate the drag deceleration over one Julian year -> Velocity
    let dv: Velocity = orbit::drag_makeup_delta_v(a_drag, Time::new(31_557_600.0));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Velocity = dv;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "dv_dm", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "dv_dm", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Velocity::UNIT, reason: "make-up delta-v cannot be negative" });
    }
    if answer.get() > 100000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "dv_dm", value: answer.get(), bound: 100000.0, edge: Edge::Upper, unit: Velocity::UNIT, reason: "above 100 km/s a year no propulsion system of any kind closes, and the answer is that the altitude is wrong" });
    }
    Ok(answer)
}
