// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How far does a molecule travel between collisions?
///
/// `lambda = 1/(sqrt(2)*n*sigma)`
///
/// Source: `us_std_1976`
pub const NODE_ID: &str = "env_mean_free_path";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xb898a6fb2272a28b;

pub fn evaluate(n: NumberDensity) -> Result<Length, Fault> {
    // ---- HOLE 1 : apply the hard-sphere mean free path with an effective cross-section of 1e-19 m2 -> Length
    let l: Length = env::mean_free_path(n);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Length = l;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "lambda", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.001 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "lambda", value: answer.get(), bound: 0.001, edge: Edge::Lower, unit: Length::UNIT, reason: "a mean free path below a millimetre would put a spacecraft-scale body in continuum flow" });
    }
    if answer.get() > 1000000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "lambda", value: answer.get(), bound: 1000000000.0, edge: Edge::Upper, unit: Length::UNIT, reason: "beyond 1e9 m the concept of a mean free path has no operational meaning here" });
    }
    Ok(answer)
}
