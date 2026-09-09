// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How hot is the upper thermosphere today, given the Sun and the geomagnetic field?
///
/// `T_inf = 379 + 3.24*F10.7A + 1.3*(F10.7 - F10.7A) + 28*Kp + 0.03*exp(Kp)`
///
/// Source: `jacchia1971`
///
/// # Assumptions
///
/// * Night-time minimum, with no diurnal or seasonal term — fails when the diurnal bulge adds up to 30% at 14:00 local solar time; a design sized on this value alone is sized on the quiet side
pub const NODE_ID: &str = "env_exospheric_temperature";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x2e87228e844416ab;

pub fn evaluate(f107: Ratio, f107a: Ratio, kp: Ratio) -> Result<Temperature, Fault> {
    // ---- HOLE 1 : apply the Jacchia 1971 night-time minimum relation with its geomagnetic correction -> Temperature
    let t_inf: Temperature = env::exospheric_temperature(f107.get(), f107a.get(), kp.get());
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Temperature = t_inf;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "T_inf", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_inf", value: answer.get(), bound: 400.0, edge: Edge::Lower, unit: Temperature::UNIT, reason: "no observed thermosphere is colder than 400 K; below that the Bates profile inverts" });
    }
    if answer.get() > 2500.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_inf", value: answer.get(), bound: 2500.0, edge: Edge::Upper, unit: Temperature::UNIT, reason: "above 2500 K is beyond any recorded storm and beyond the fit" });
    }
    Ok(answer)
}
