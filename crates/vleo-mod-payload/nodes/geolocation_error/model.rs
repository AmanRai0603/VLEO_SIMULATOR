// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How accurately can an emitter on the ground be located?
///
/// `e = c*sigma_tau*GDOP`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "pay_geolocation_error";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x11554f19c2ed118f;

pub fn evaluate(s: Time, g: Ratio) -> Result<Length, Fault> {
    // ---- HOLE 1 : convert the timing uncertainty into a range uncertainty and apply the geometric dilution -> Length
    let e: Length = payload::tdoa_geolocation_error(s, g.get());
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Length = e;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "e_geo", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.01 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "e_geo", value: answer.get(), bound: 0.01, edge: Edge::Lower, unit: Length::UNIT, reason: "below a centimetre no time-difference system in this design performs that well" });
    }
    if answer.get() > 10000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "e_geo", value: answer.get(), bound: 10000000.0, edge: Edge::Upper, unit: Length::UNIT, reason: "above 10000 km the answer carries no information" });
    }
    Ok(answer)
}
