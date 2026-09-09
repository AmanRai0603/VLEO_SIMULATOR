// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// Does the spacecraft fit the mass it is allowed?
///
/// `M = (m_lim - m_wet)/m_lim`
///
/// Source: `ecss_e_st_10_02`
pub const NODE_ID: &str = "mass_margin";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x727f32468ceb4f94;

pub fn evaluate(l: Mass, w: Mass) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : form the signed fractional margin against the declared limit -> Ratio
    let m: Ratio = mass::mass_margin(l, w);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = m;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "M_mass", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < -5.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "M_mass", value: answer.get(), bound: -5.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a margin below -500% means the design is five times over and the inputs are wrong" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "M_mass", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "a margin of one means the spacecraft weighs nothing" });
    }
    Ok(answer)
}
