// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// By what factor is the imagery compressed before downlink?
///
/// `CR_img = 3`
///
/// Source: `ccsds122`
pub const NODE_ID: &str = "pay_compression";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x08316cadbabda146;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(3.0, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "CR_img", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "CR_img", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "CR_img", value: answer.get(), bound: 1.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "unity is no compression, the limit" });
    }
    if answer.get() > 20.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "CR_img", value: answer.get(), bound: 20.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 20:1 the compression is lossy enough to destroy the radiometry the product is sold on" });
    }
    Ok(answer)
}
