// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How fast does the orbit fall if nothing compensates the drag?
///
/// `da/dt = -rho*sqrt(mu*a)/BC`
///
/// Source: `vallado2013`
pub const NODE_ID: &str = "orbit_decay_rate";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x5488a3d4ea2782d2;

pub fn evaluate(rho: MassDensity, bc: Ratio, r: Length) -> Result<Velocity, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::orbit_decay_rate. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Velocity = match methods::orbit_decay_rate::evaluate(rho.get(), bc.get(), r.get()) {
        Ok(v) => Velocity::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "da_dt")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Velocity = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "da_dt", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < -10.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "da_dt", value: answer.get(), bound: -10.0, edge: Edge::Lower, unit: Velocity::UNIT, reason: "a decay faster than 10 m/s of semi-major axis per second is re-entry, not an orbit" });
    }
    if answer.get() > 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "da_dt", value: answer.get(), bound: 0.0, edge: Edge::Upper, unit: Velocity::UNIT, reason: "drag cannot raise an orbit; a positive value here is a sign error upstream" });
    }
    Ok(answer)
}
