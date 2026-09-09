// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// Is the flow free-molecular — which is what licenses every aerodynamic relation in this tree?
///
/// `Kn = lambda/L`
///
/// Source: `us_std_1976`
pub const NODE_ID: &str = "env_knudsen";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xaf695b95fd705ba8;

pub fn evaluate(lam: Length, l_body: Length) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : divide the mean free path by the vehicle's characteristic length -> Ratio
    let kn: Ratio = env::knudsen(lam, l_body);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = kn;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kn", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 10.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kn", value: answer.get(), bound: 10.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below Kn = 10 the flow is transitional and Sentman's free-molecular relations are outside their envelope. This bound is the Ariane 501 lesson written as a guard: a component used outside the envelope it was specified for, with nothing re-derived" });
    }
    if answer.get() > 1000000000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kn", value: answer.get(), bound: 1000000000000.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 1e12 the number is meaningless but harmless" });
    }
    Ok(answer)
}
