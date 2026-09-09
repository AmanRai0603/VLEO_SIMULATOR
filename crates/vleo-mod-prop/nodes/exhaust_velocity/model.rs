// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How fast does the accelerated beam leave the thruster?
///
/// `v_e = sqrt(2*q*V_b/m_i)`
///
/// Source: `romano2021`
///
/// Air is lighter than xenon, so the same voltage buys a far higher exhaust
/// velocity. It is the one respect in which air-breathing propulsion is
/// easier rather than harder.
pub const NODE_ID: &str = "prop_exhaust_velocity";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x4eb97c906369687a;

pub fn evaluate(vb: Voltage, m: MolarMass) -> Result<Velocity, Fault> {
    // ---- HOLE 1 : accelerate a singly-charged ion of the local mean mass through the beam potential -> Velocity
    let ve: Velocity = prop::beam_exhaust_velocity(vb, m);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Velocity = ve;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "v_e", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 1000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "v_e", value: answer.get(), bound: 1000.0, edge: Edge::Lower, unit: Velocity::UNIT, reason: "below 1 km/s the exhaust is slower than a cold gas thruster and the concept has no advantage" });
    }
    if answer.get() > 500000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "v_e", value: answer.get(), bound: 500000.0, edge: Edge::Upper, unit: Velocity::UNIT, reason: "above 500 km/s the relativistic and space-charge assumptions in the relation break down" });
    }
    Ok(answer)
}
