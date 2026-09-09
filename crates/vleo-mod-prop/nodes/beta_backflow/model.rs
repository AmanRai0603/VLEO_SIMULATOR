// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What fraction of thermal molecules in the chamber find their way back out of the mouth?
///
/// `beta = 0.06`
///
/// Source: `romano2021`
///
/// This single number is what an intake design is for. A long, narrow,
/// honeycombed duct makes it small; that is the whole device.
pub const NODE_ID: &str = "prop_beta_backflow";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x464477ab5a2a8869;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(0.06, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "beta", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "beta", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.001 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "beta", value: answer.get(), bound: 0.001, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 0.001 no manufacturable duct is that good at trapping thermal molecules" });
    }
    if answer.get() > 0.5 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "beta", value: answer.get(), bound: 0.5, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 0.5 the duct traps nothing and the compression ratio collapses to one" });
    }
    Ok(answer)
}
