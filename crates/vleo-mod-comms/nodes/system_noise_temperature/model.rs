// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much noise does the receiving system contribute?
///
/// `T_s = T_a/L + T0*(L-1)/L + T0*(F-1)`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "com_system_noise_temperature";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x7c0723a21a23bdfd;

pub fn evaluate(ta: Temperature, ll: Ratio) -> Result<Temperature, Fault> {
    // ---- HOLE 1 : combine the antenna, line and receiver noise contributions at a 290 K physical temperature and a 1.2 dB noise figure -> Temperature
    let t: Temperature = comms::system_noise_temperature(ta, ll.get(), 1.2, Temperature::new(290.0));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Temperature = t;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "T_s", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 10.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_s", value: answer.get(), bound: 10.0, edge: Edge::Lower, unit: Temperature::UNIT, reason: "below 10 K requires cryogenic cooling not present in this ground segment" });
    }
    if answer.get() > 3000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_s", value: answer.get(), bound: 3000.0, edge: Edge::Upper, unit: Temperature::UNIT, reason: "above 3000 K the receiving system is not usable" });
    }
    Ok(answer)
}
