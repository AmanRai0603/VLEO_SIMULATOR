// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What fraction of the Earth does one satellite see at any instant?
///
/// `f = (1 - cos(lambda))/2`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "mis_coverage_fraction";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xf37f87b4cebb9512;

pub fn evaluate(lam: Angle) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : take the spherical cap as a fraction of the whole sphere -> Ratio
    let f: Ratio = mission::instantaneous_coverage_fraction(lam);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = f;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "f_cov", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "f_cov", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a fraction cannot be negative" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "f_cov", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "a satellite cannot see more than the whole Earth" });
    }
    Ok(answer)
}
