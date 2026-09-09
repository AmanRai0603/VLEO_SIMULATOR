// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How many particles per cubic metre are there at the flight altitude?
///
/// `n = SUM_i n_i(z)`
///
/// Source: `jacchia1971`
pub const NODE_ID: &str = "env_number_density";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xcac4e1f17182d74a;

pub fn evaluate(h: Length, t_inf: Temperature) -> Result<NumberDensity, Fault> {
    // ---- HOLE 1 : sum the per-species diffusive-equilibrium densities -> NumberDensity
    let n: NumberDensity = env::composition(h, t_inf).total();
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: NumberDensity = n;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "n", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 100000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "n", value: answer.get(), bound: 100000000.0, edge: Edge::Lower, unit: NumberDensity::UNIT, reason: "below 1e8 per cubic metre there is nothing for an intake to collect" });
    }
    if answer.get() > 1e20 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "n", value: answer.get(), bound: 1e20, edge: Edge::Upper, unit: NumberDensity::UNIT, reason: "above 1e20 the flow is continuum and the intake model does not apply" });
    }
    Ok(answer)
}
