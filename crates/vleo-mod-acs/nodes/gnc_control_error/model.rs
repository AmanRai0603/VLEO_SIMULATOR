// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How well does the controller hold the commanded attitude?
///
/// `sig_ctl = 40`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "gnc_control_error";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x25876f3c307b20dd;

pub fn evaluate() -> Result<Angle, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Angle = match Angle::from_unit(40.0, Unit::Arcsecond) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "sig_ctl", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Angle = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "sig_ctl", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 4.84813681109536e-6 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "sig_ctl", value: answer.get(), bound: 4.84813681109536e-6, edge: Edge::Lower, unit: Angle::UNIT, reason: "below 1 arcsec no wheel-based controller holds that in the presence of VLEO aerodynamic torque" });
    }
    if answer.get() > 0.03490658503988659 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "sig_ctl", value: answer.get(), bound: 0.03490658503988659, edge: Edge::Upper, unit: Angle::UNIT, reason: "above two degrees the controller does not support any pointing requirement in this design" });
    }
    Ok(answer)
}
