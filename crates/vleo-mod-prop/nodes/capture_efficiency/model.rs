// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What fraction of the flow entering the mouth actually reaches the thruster?
///
/// `eta_c = A_out*eta_geo/(A_out + beta*A_in)`
///
/// Source: `romano2021`
///
/// Read the relation slowly: the collection efficiency does not depend on the
/// flight speed at all, only on geometry. Speed buys compression, not
/// capture. A design trying to raise capture by flying faster is optimising
/// the wrong term.
///
/// # Assumptions
///
/// * Free-molecular flow throughout the duct and the chamber — fails when if the chamber compresses far enough to become collisional the balance is no longer a simple flux balance, which happens above a compression ratio of roughly 1e4
/// * The chamber gas is fully thermalised at the chamber temperature — fails when a short duct passes part of the beam straight through, which raises the delivered flow above this and is not modelled
pub const NODE_ID: &str = "prop_capture_efficiency";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xa67dd2ff75479fc1;

pub fn evaluate(a_in: Area, a_out: Area, eta_geo: Ratio, beta: Ratio, t_c: Temperature, m: MolarMass, n: NumberDensity, v: Velocity) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : balance the hyperthermal inflow against the thermal outflow through the throat and back out of the mouth -> Ratio
    let r: Ratio = prop::intake_balance(n, v, a_in, a_out, eta_geo, beta, t_c, m).collection_efficiency;
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = r;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "eta_c", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "eta_c", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a collection efficiency cannot be negative" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "eta_c", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "an intake cannot deliver more than enters its mouth" });
    }
    Ok(answer)
}
