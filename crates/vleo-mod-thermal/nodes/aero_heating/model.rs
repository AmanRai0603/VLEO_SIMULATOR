// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much does the air itself heat the spacecraft?
///
/// `Q = 0.5*rho*V^3*A*C_h`
///
/// Source: `larson_wertz`
///
/// Usually neglected, and at 200 km it should not be. It is a real term in
/// the balance of a small, fast, low-flying vehicle.
pub const NODE_ID: &str = "thm_aero_heating";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xaa3d9a875436e9ee;

pub fn evaluate(rho: MassDensity, v: Velocity, a: Area) -> Result<Power, Fault> {
    // ---- HOLE 1 : form the incident kinetic energy flux and apply a heat transfer coefficient of 0.9 -> Power
    let q: Power = thermal::free_molecular_heating(rho, v, a, Ratio::new(0.9));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Power = q;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Q_aero", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Q_aero", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Power::UNIT, reason: "heating cannot be negative" });
    }
    if answer.get() > 100000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Q_aero", value: answer.get(), bound: 100000.0, edge: Edge::Upper, unit: Power::UNIT, reason: "above 100 kW the vehicle is re-entering" });
    }
    Ok(answer)
}
