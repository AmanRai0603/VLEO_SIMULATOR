// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// Is the spacecraft inside its temperature limit?
///
/// `M = T_max - T_eq`
///
/// Source: `ecss_e_st_10_02`
pub const NODE_ID: &str = "thm_margin";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xbea93c4906da4c9a;

pub fn evaluate(t: Temperature, lim: Temperature) -> Result<Temperature, Fault> {
    // ---- HOLE 1 : subtract the predicted temperature from the declared limit -> Temperature
    let m: Temperature = thermal::thermal_margin(t, lim);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Temperature = m;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "M_thm", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < -500.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "M_thm", value: answer.get(), bound: -500.0, edge: Edge::Lower, unit: Temperature::UNIT, reason: "a margin below -500 K means the balance is being fed a nonsense load" });
    }
    if answer.get() > 500.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "M_thm", value: answer.get(), bound: 500.0, edge: Edge::Upper, unit: Temperature::UNIT, reason: "a margin above 500 K means the limit is not the one that binds" });
    }
    Ok(answer)
}
