// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How strong must the torque rods be to dump that momentum?
///
/// `m = h/(B*t_dump)`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "gnc_magnetorquer_dipole";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xf66190f61f3da001;

pub fn evaluate(h: AngularMomentum, b: MagneticFluxDensity, td: Time) -> Result<DipoleMoment, Fault> {
    // ---- HOLE 1 : divide the stored momentum by the field and the available dump interval -> DipoleMoment
    let m: DipoleMoment = gnc::magnetorquer_dipole_required(h, b, td);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: DipoleMoment = m;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "m_req", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "m_req", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: DipoleMoment::UNIT, reason: "a dipole magnitude cannot be negative" });
    }
    if answer.get() > 1000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "m_req", value: answer.get(), bound: 1000.0, edge: Edge::Upper, unit: DipoleMoment::UNIT, reason: "above 1000 A.m2 no torque rod that fits this vehicle produces it" });
    }
    Ok(answer)
}
