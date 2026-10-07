// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How well does this vehicle resist the atmosphere, per unit of its own mass?
///
/// `BC = m/(Cd*A)`
///
/// Source: `vallado2013`
///
/// Higher is better here. A cubesat is around 50; a slender VLEO platform
/// reaches 150, and that difference is worth tens of kilometres of altitude.
pub const NODE_ID: &str = "aero_ballistic_coefficient";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x16f00403b97750d2;

pub fn evaluate(m: Mass, cd: Ratio, a: Area) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::aero_ballistic_coefficient. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::aero_ballistic_coefficient::evaluate(m.get(), cd.get(), a.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "BC")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "BC", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 5.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "BC", value: answer.get(), bound: 5.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 5 kg/m2 the vehicle is a sail, and nothing in this band closes" });
    }
    if answer.get() > 500.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "BC", value: answer.get(), bound: 500.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 500 kg/m2 the vehicle is denser than any spacecraft ever flown" });
    }
    Ok(answer)
}
