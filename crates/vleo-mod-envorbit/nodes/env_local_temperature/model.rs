// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What is the gas temperature at the flight altitude?
///
/// `T(z) = T_inf - (T_inf - T_120)*exp(-s*(z - z_120))`
///
/// Source: `jacchia1971`
pub const NODE_ID: &str = "env_local_temperature";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xd2f9d109a57ca737;

pub fn evaluate(h: Length, t_inf: Temperature) -> Result<Temperature, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::env_local_temperature. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Temperature = match methods::env_local_temperature::evaluate(h.get(), t_inf.get()) {
        Ok(v) => Temperature::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "T")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Temperature = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "T", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 200.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T", value: answer.get(), bound: 200.0, edge: Edge::Lower, unit: Temperature::UNIT, reason: "below the 120 km base temperature the profile is not defined" });
    }
    if answer.get() > 2500.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T", value: answer.get(), bound: 2500.0, edge: Edge::Upper, unit: Temperature::UNIT, reason: "cannot exceed the exospheric temperature it approaches" });
    }
    Ok(answer)
}
