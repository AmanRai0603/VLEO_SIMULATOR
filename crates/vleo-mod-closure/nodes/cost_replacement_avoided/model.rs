// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What does air-breathing propulsion save by removing the need to replace satellites when propellant runs out?
///
/// `C = (C_unit + C_launch)*N_replacements`
///
/// Source: `orbitt_case_c1`
///
/// The commercial argument for the whole programme, as a node with a source
/// and evidence rather than a slide.
pub const NODE_ID: &str = "cost_replacement_avoided";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x30d28eb931fc1dc7;

pub fn evaluate(cb: Money, cp: Money, lc: Money, n: Ratio, td: Ratio) -> Result<Money, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::cost_replacement_avoided. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Money = match methods::cost_replacement_avoided::evaluate(cb.get(), cp.get(), lc.get(), n.get(), td.get()) {
        Ok(v) => Money::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "C_avd")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Money = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "C_avd", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "C_avd", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Money::UNIT, reason: "a saving cannot be negative" });
    }
    if answer.get() > 1000000000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "C_avd", value: answer.get(), bound: 1000000000000.0, edge: Edge::Upper, unit: Money::UNIT, reason: "above a trillion dollars the programme is not the one being designed" });
    }
    Ok(answer)
}
