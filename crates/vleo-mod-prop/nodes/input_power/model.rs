// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much electrical power does the thruster itself draw?
///
/// `P_in = (P_jet + P_ion)/(1 - L_other)`
///
/// Source: `romano2021`
pub const NODE_ID: &str = "prop_input_power";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xaf669d8a1f081784;

pub fn evaluate(pj: Power, pi: Power, lo: Ratio) -> Result<Power, Fault> {
    // ---- HOLE 1 : sum the useful terms and inflate them by the other-loss fraction -> Power
    let p: Power = prop::thruster_input_power(pj, pi, lo);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Power = p;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "P_in", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "P_in", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Power::UNIT, reason: "input power cannot be negative" });
    }
    if answer.get() > 100000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "P_in", value: answer.get(), bound: 100000.0, edge: Edge::Upper, unit: Power::UNIT, reason: "above 100 kW no bus in this mass class supplies it" });
    }
    Ok(answer)
}
