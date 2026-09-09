// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much atomic oxygen is there — the species that erodes materials and the one an intake actually collects?
///
/// `n_O(z) = n_O(z0)*(T0/T)*exp(-INT M_O*g/(R*T) dz)`
///
/// Source: `jacchia1971`
pub const NODE_ID: &str = "env_atomic_oxygen_density";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xdfc6709ddcc473b9;

pub fn evaluate(h: Length, t_inf: Temperature) -> Result<NumberDensity, Fault> {
    // ---- HOLE 1 : integrate diffusive equilibrium for atomic oxygen alone -> NumberDensity
    let n_o: NumberDensity = env::composition(h, t_inf).o;
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: NumberDensity = n_o;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "n_O", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 100000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "n_O", value: answer.get(), bound: 100000000.0, edge: Edge::Lower, unit: NumberDensity::UNIT, reason: "below 1e8 there is effectively no atomic oxygen and the accommodation model returns a specular surface" });
    }
    if answer.get() > 1e19 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "n_O", value: answer.get(), bound: 1e19, edge: Edge::Upper, unit: NumberDensity::UNIT, reason: "above 1e19 is denser than the total number density at the model base" });
    }
    Ok(answer)
}
