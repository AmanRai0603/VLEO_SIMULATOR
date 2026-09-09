// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// If propulsion stops, how long before the satellite re-enters?
///
/// `t = INT da/(da/dt) from h to 120 km`
///
/// Source: `vallado2013`
///
/// # Assumptions
///
/// * Atmospheric conditions fixed at the current activity for the whole decay — fails when over a solar cycle the density at a given altitude changes by more than an order of magnitude, so this is an order-of-magnitude answer and is reported as one
pub const NODE_ID: &str = "orbit_lifetime_uncontrolled";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xbd35717f3d5b3da4;

pub fn evaluate(h: Length, bc: Ratio, t_inf: Temperature) -> Result<Time, Fault> {
    // ---- HOLE 1 : integrate the decay rate down to the 120 km model base at fixed atmospheric conditions -> Time
    let t: Time = orbit::lifetime_estimate(h, Length::from_km(120.0), bc.get(), 64, |z| env::mass_density(z, t_inf));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Time = t;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "t_life", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "t_life", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Time::UNIT, reason: "a lifetime cannot be negative" });
    }
    if answer.get() > 3153600000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "t_life", value: answer.get(), bound: 3153600000.0, edge: Edge::Upper, unit: Time::UNIT, reason: "above 100 years the 25-year debris rule is violated and the number's precision is irrelevant" });
    }
    Ok(answer)
}
