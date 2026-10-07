// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What fraction of each revolution is spent in the Earth's shadow?
///
/// `f_ecl = (1/pi)*acos(sqrt(r^2 - Re^2)/(r*cos(beta)))`
///
/// Source: `larson_wertz`
///
/// # Assumptions
///
/// * Cylindrical shadow, no penumbra — fails when the penumbra adds about 10 seconds per orbit, which matters for a precise thermal transient and not for a power budget
pub const NODE_ID: &str = "orbit_eclipse_fraction";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x091ebb3484ae9c66;

pub fn evaluate(r: Length, beta: Angle) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::orbit_eclipse_fraction. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::orbit_eclipse_fraction::evaluate(r.get(), beta.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "f_ecl")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "f_ecl", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "f_ecl", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a negative eclipse fraction is not a fraction" });
    }
    if answer.get() > 0.45 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "f_ecl", value: answer.get(), bound: 0.45, edge: Edge::Upper, unit: Ratio::UNIT, reason: "no circular orbit in this band spends more than 45% of a revolution in shadow" });
    }
    Ok(answer)
}
