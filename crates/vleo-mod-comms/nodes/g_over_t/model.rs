// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What is the ground station's sensitivity, as gain over system noise temperature?
///
/// `G/T = G_r - 10*log10(T_s)`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "com_g_over_t";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x19b40aeecbb760bb;

pub fn evaluate(g: Ratio, t: Temperature) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : subtract the system noise temperature in decibels from the receive gain -> Ratio
    let gt: Ratio = Ratio::new(comms::g_over_t_db(g.get(), t));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = gt;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "GT", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < -30.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "GT", value: answer.get(), bound: -30.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below -30 dB/K no ground station in this network performs that badly" });
    }
    if answer.get() > 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "GT", value: answer.get(), bound: 60.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 60 dB/K is a deep-space station" });
    }
    Ok(answer)
}
