// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

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
pub const SHEET_HASH: u64 = 0x49908ec9f78b357d;

pub fn evaluate(h: Length, t_inf: Temperature) -> Result<Temperature, Fault> {
    // ---- HOLE 1 : evaluate the Bates profile between the 120 km base and the exospheric limit -> Temperature
    let t: Temperature = env::temperature(h, t_inf);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Temperature = t;
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
