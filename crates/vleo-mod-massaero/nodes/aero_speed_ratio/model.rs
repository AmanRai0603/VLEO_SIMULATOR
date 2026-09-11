// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How hyperthermal is the flow — does the gas arrive as a beam or as a cloud?
///
/// `s = V/sqrt(2*R*T/M)`
///
/// Source: `sentman1961`
pub const NODE_ID: &str = "aero_speed_ratio";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x9a432528dfbb8276;

pub fn evaluate(v: Velocity, t: Temperature, m: MolarMass) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : divide the bulk speed by the most probable thermal speed of the local mixture -> Ratio
    let s: Ratio = aero::speed_ratio(v, t, m);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = s;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "s", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "s", value: answer.get(), bound: 1.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below a speed ratio of one the flow is thermal rather than hyperthermal, and Sentman's relations lose their meaning" });
    }
    if answer.get() > 30.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "s", value: answer.get(), bound: 30.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 30 the exponential terms underflow and the relations are numerically dead" });
    }
    Ok(answer)
}
