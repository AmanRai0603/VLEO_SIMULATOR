// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What is the specific impulse, referred honestly to the collected flow rather than the ionised part?
///
/// `Isp = T/(mdot*g0)`
///
/// Source: `romano2021`
///
/// Referring this to the ion flow alone flatters the number by the reciprocal
/// of the utilisation, which is how brochure figures are made.
pub const NODE_ID: &str = "prop_specific_impulse";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x308cca69e0838e48;

pub fn evaluate(t: Force, md: MassFlow) -> Result<Time, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::prop_specific_impulse. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Time = match methods::prop_specific_impulse::evaluate(t.get(), md.get()) {
        Ok(v) => Time::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "Isp")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Time = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Isp", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 100.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Isp", value: answer.get(), bound: 100.0, edge: Edge::Lower, unit: Time::UNIT, reason: "below 100 s an air-breathing system offers nothing a cold gas thruster does not" });
    }
    if answer.get() > 50000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Isp", value: answer.get(), bound: 50000.0, edge: Edge::Upper, unit: Time::UNIT, reason: "above 50000 s the relation is being fed an ion flow that is not physical" });
    }
    Ok(answer)
}
