// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What system-level margin is held above the sum of the subsystem masses?
///
/// `M_sys = 0.2`
///
/// Source: `ecss_e_st_10_02`
///
/// Separate from and additional to the per-item maturity allowances. It
/// covers what is not on the list yet, which on any real programme is the
/// term that bites.
pub const NODE_ID: &str = "mass_system_margin";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x5e38ea02bba11fd2;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(0.2, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "M_sys", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "M_sys", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "M_sys", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a margin cannot be negative" });
    }
    if answer.get() > 0.5 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "M_sys", value: answer.get(), bound: 0.5, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 50% the design is a sketch and the number is not a budget" });
    }
    Ok(answer)
}
