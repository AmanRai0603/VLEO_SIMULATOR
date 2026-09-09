// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much mass arrives on a square metre facing the flow each second?
///
/// `phi = rho*V`
///
/// Source: `romano2021`
///
/// The ceiling on everything downstream. At 200 km it is about 2 milligrams
/// per square metre per second.
pub const NODE_ID: &str = "prop_incident_flux";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xc9cacfd9b90887e4;

pub fn evaluate(rho: MassDensity, v: Velocity) -> Result<MassFlux, Fault> {
    // ---- HOLE 1 : multiply the free-stream density by the flight speed -> MassFlux
    let f: MassFlux = prop::incident_mass_flux(rho, v);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: MassFlux = f;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "phi", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "phi", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: MassFlux::UNIT, reason: "an incident flux cannot be negative" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "phi", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: MassFlux::UNIT, reason: "above 1 kg/m2/s the flow is continuum and the intake model does not apply" });
    }
    Ok(answer)
}
