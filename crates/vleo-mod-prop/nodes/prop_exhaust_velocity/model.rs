// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

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
pub const SHEET_HASH: u64 = 0x25af469bf116a4ba;

pub fn evaluate(vb: Voltage, m: MolarMass) -> Result<Velocity, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::prop_exhaust_velocity. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Velocity = match methods::prop_exhaust_velocity::evaluate(vb.get(), m.get()) {
        Ok(v) => Velocity::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "v_e")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Velocity = method_answer;
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
