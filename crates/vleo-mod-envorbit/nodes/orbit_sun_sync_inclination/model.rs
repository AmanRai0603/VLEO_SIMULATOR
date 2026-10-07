// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What inclination makes the orbit plane keep pace with the Sun?
///
/// `cos(i) = -Omega_sun*(1-e^2)^2*a^(7/2)/(1.5*J2*sqrt(mu)*Re^2)`
///
/// Source: `vallado2013`
pub const NODE_ID: &str = "orbit_sun_sync_inclination";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xef1bd67caed7e0be;

pub fn evaluate(r: Length, e: Ratio) -> Result<Angle, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::orbit_sun_sync_inclination. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Angle = match methods::orbit_sun_sync_inclination::evaluate(r.get(), e.get()) {
        Ok(v) => Angle::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "i_ss")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Angle = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "i_ss", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 1.4835298641951802 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "i_ss", value: answer.get(), bound: 1.4835298641951802, edge: Edge::Lower, unit: Angle::UNIT, reason: "a Sun-synchronous inclination is always retrograde and above about 95 degrees in this band" });
    }
    if answer.get() > 2.007128639793479 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "i_ss", value: answer.get(), bound: 2.007128639793479, edge: Edge::Upper, unit: Angle::UNIT, reason: "above 115 degrees no Sun-synchronous solution exists below 2000 km" });
    }
    Ok(answer)
}
