// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much thrust does the thruster produce from the air it is given?
///
/// `T = mdot_i*v_e*alpha_div*alpha_pp`
///
/// Source: `romano2021`
pub const NODE_ID: &str = "prop_thrust";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x59520bfdfec2a689;

pub fn evaluate(mi: MassFlow, ve: Velocity, ad: Ratio, ap: Ratio) -> Result<Force, Fault> {
    // ---- HOLE 1 : multiply the ion flow by the exhaust velocity and apply the divergence and multiple-charge corrections -> Force
    let t: Force = prop::beam_thrust(mi, ve, ad, ap);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Force = t;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "T", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Force::UNIT, reason: "thrust cannot be negative" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Force::UNIT, reason: "above 1 N no air-breathing thruster in this band produces thrust, at any intake size" });
    }
    Ok(answer)
}
