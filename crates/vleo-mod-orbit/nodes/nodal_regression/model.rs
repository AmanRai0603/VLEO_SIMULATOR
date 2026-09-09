// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How fast does the orbit plane rotate under the second zonal harmonic?
///
/// `dOmega/dt = -1.5*J2*sqrt(mu)*Re^2*cos(i)/((1-e^2)^2*a^(7/2))`
///
/// Source: `vallado2013`
pub const NODE_ID: &str = "orbit_nodal_regression";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x9762affa57436b4d;

pub fn evaluate(r: Length, e: Ratio, i: Angle) -> Result<AngularRate, Fault> {
    // ---- HOLE 1 : evaluate the secular J2 node rate -> AngularRate
    let d: AngularRate = orbit::nodal_regression(r, e.get(), i);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: AngularRate = d;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "dOmega", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < -3.028700089570008e-6 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "dOmega", value: answer.get(), bound: -3.028700089570008e-6, edge: Edge::Lower, unit: AngularRate::UNIT, reason: "a regression faster than 15 degrees per day does not occur in this altitude band" });
    }
    if answer.get() > 3.028700089570008e-6 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "dOmega", value: answer.get(), bound: 3.028700089570008e-6, edge: Edge::Upper, unit: AngularRate::UNIT, reason: "a regression faster than 15 degrees per day does not occur in this altitude band" });
    }
    Ok(answer)
}
