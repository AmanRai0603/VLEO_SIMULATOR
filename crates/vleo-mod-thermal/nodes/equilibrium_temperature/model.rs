// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What temperature does the spacecraft settle at?
///
/// `T = (Q_total/(eps*sigma*A_rad))^(1/4)`
///
/// Source: `larson_wertz`
///
/// # Assumptions
///
/// * One node — the whole spacecraft is at one temperature — fails when a real design has a gradient across every panel; this sizes a radiator and bounds a temperature, and it does not predict a gradient
pub const NODE_ID: &str = "thm_equilibrium_temperature";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x0b60557bf95b3e31;

pub fn evaluate(qs: Power, qa: Power, qi: Power, qh: Power, qd: Power, ar: Area, e: Ratio) -> Result<Temperature, Fault> {
    // ---- HOLE 1 : balance every absorbed and dissipated term against grey-body radiation from the radiator -> Temperature
    let t: Temperature = thermal::equilibrium_temperature(Power::new(qs.get() + qa.get() + qi.get() + qh.get()), qd, ar, e);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Temperature = t;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "T_eq", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 100.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_eq", value: answer.get(), bound: 100.0, edge: Edge::Lower, unit: Temperature::UNIT, reason: "below 100 K the spacecraft would be colder than deep space plus the Earth's infrared, which is not possible in this orbit" });
    }
    if answer.get() > 1000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_eq", value: answer.get(), bound: 1000.0, edge: Edge::Upper, unit: Temperature::UNIT, reason: "above 1000 K nothing survives and the balance is being fed a nonsense load" });
    }
    Ok(answer)
}
