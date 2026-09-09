// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much angular momentum must the wheels be able to store?
///
/// `h = T*T_orb/2*(1 + margin)`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "gnc_momentum_storage";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xbe5b3da4ce44bb9f;

pub fn evaluate(t: Torque, p: Time) -> Result<AngularMomentum, Fault> {
    // ---- HOLE 1 : accumulate the secular torque over half a revolution with a 50% margin -> AngularMomentum
    let h: AngularMomentum = gnc::momentum_storage_required(t, p, Ratio::new(0.5));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: AngularMomentum = h;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "h_req", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "h_req", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: AngularMomentum::UNIT, reason: "a momentum magnitude cannot be negative" });
    }
    if answer.get() > 100.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "h_req", value: answer.get(), bound: 100.0, edge: Edge::Upper, unit: AngularMomentum::UNIT, reason: "above 100 N.m.s no wheel assembly that fits this vehicle stores it" });
    }
    Ok(answer)
}
