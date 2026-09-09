// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much energy does the source spend to make one ion, including every loss?
///
/// `eps_i = 250`
///
/// Source: `romano2021`
///
/// Not the ionisation potential. A real source spends 2 to 30 times the
/// theoretical minimum on excitation, radiation and wall losses.
pub const NODE_ID: &str = "prop_ionisation_cost";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x8c8b68e371fd28ec;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(250.0, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "eps_i", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "eps_i", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 20.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "eps_i", value: answer.get(), bound: 20.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 20 eV per ion is under twice the 13.6 eV ionisation potential of atomic oxygen; no source has approached it" });
    }
    if answer.get() > 1000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "eps_i", value: answer.get(), bound: 1000.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 1000 eV per ion the ionisation power alone exceeds any plausible bus, whatever the beam does" });
    }
    Ok(answer)
}
