// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How many oxygen atoms strike a forward-facing surface over the mission?
///
/// `F = n_O*V*t`
///
/// Source: `moe2005`
///
/// Polyimide erodes at about 3e-24 cm3 per incident atom, so 1e22 removes
/// roughly 30 micrometres of it.
pub const NODE_ID: &str = "aero_ao_fluence";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x9771f23449c0b7b1;

pub fn evaluate(n_o: NumberDensity, v: Velocity, t: Time) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::aero_ao_fluence. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::aero_ao_fluence::evaluate(n_o.get(), v.get(), t.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "F_AO")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F_AO", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F_AO", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a fluence cannot be negative" });
    }
    if answer.get() > 1e26 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F_AO", value: answer.get(), bound: 1e26, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 1e26 atoms per square metre no known external material survives, so the design is not a design" });
    }
    Ok(answer)
}
