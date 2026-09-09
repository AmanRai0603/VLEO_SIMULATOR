// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much power does making the ions cost, before any of them is accelerated?
///
/// `P_ion = (mdot_i/m_i)*eps_i*q`
///
/// Source: `romano2021`
pub const NODE_ID: &str = "prop_ionisation_power";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x9fa00e51e687ad45;

pub fn evaluate(mi: MassFlow, m: MolarMass, eps: Ratio) -> Result<Power, Fault> {
    // ---- HOLE 1 : convert the ion mass flow into ions per second and charge each one its stated energy cost -> Power
    let p: Power = prop::ionisation_power(mi, m, eps.get());
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Power = p;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "P_ion", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "P_ion", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Power::UNIT, reason: "ionisation power cannot be negative" });
    }
    if answer.get() > 100000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "P_ion", value: answer.get(), bound: 100000.0, edge: Edge::Upper, unit: Power::UNIT, reason: "above 100 kW no bus in this mass class supplies it" });
    }
    Ok(answer)
}
