// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much uncompensated magnetic moment does the spacecraft carry?
///
/// `m_res = 0.05`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "gnc_residual_dipole";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xcee00197f4033c70;

pub fn evaluate() -> Result<DipoleMoment, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: DipoleMoment = match DipoleMoment::from_unit(0.05, Unit::AmpereSquareMetre) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "m_res", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: DipoleMoment = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "m_res", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "m_res", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: DipoleMoment::UNIT, reason: "a residual dipole magnitude cannot be negative" });
    }
    if answer.get() > 5.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "m_res", value: answer.get(), bound: 5.0, edge: Edge::Upper, unit: DipoleMoment::UNIT, reason: "above 5 A.m2 the magnetic disturbance dominates every other torque and the design is uncontrollable" });
    }
    Ok(answer)
}
