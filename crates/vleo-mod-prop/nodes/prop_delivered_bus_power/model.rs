// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much power does the thruster actually take, after throttling?
///
/// `P = k*P_prop`
///
/// Source: `orbitt_case_c1`
pub const NODE_ID: &str = "prop_delivered_bus_power";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xac8d27d860133e6a;

pub fn evaluate(pp: Power, k: Ratio) -> Result<Power, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::prop_delivered_bus_power. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Power = match methods::prop_delivered_bus_power::evaluate(pp.get(), k.get()) {
        Ok(v) => Power::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "P_del")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Power = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "P_del", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "P_del", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Power::UNIT, reason: "power drawn cannot be negative" });
    }
    if answer.get() > 100000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "P_del", value: answer.get(), bound: 100000.0, edge: Edge::Upper, unit: Power::UNIT, reason: "above 100 kW no bus in this mass class supplies it" });
    }
    Ok(answer)
}
