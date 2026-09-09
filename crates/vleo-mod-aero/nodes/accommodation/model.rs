// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How completely does an incoming molecule thermalise with the surface before leaving?
///
/// `alpha = K*n_O*T_inf/(1 + K*n_O*T_inf),  K = 7.5e-17`
///
/// Source: `moe2005`
///
/// A design that assumes a fixed accommodation has a margin that moves when
/// the Sun does.
///
/// # Assumptions
///
/// * The surface is covered in adsorbed atomic oxygen and behaves as that coverage dictates — fails when a freshly cleaned or a fluorinated surface adsorbs far less, and its drag differs by tens of per cent from this
pub const NODE_ID: &str = "aero_accommodation";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x78377e9d2f7decb8;

pub fn evaluate(n_o: NumberDensity, t_inf: Temperature) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : apply the Langmuir adsorption isotherm for atomic oxygen coverage -> Ratio
    let a: Ratio = Ratio::new(aero::accommodation_coefficient(n_o, t_inf));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = a;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "alpha", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "alpha", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "accommodation is a fraction and cannot be negative" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "alpha", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "full accommodation is one; above it the reflected molecules would carry away energy the surface does not have" });
    }
    Ok(answer)
}
