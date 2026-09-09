// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much data comes down in one pass?
///
/// `V = R*t*eta_link`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "com_pass_volume";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xeb9d83e91c885863;

pub fn evaluate(r: DataRate, t: Time) -> Result<DataVolume, Fault> {
    // ---- HOLE 1 : multiply rate by contact time and apply a 60% mean-pass and protocol efficiency -> DataVolume
    let v: DataVolume = comms::pass_data_volume(r, t, Ratio::new(0.60));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: DataVolume = v;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "V_pass", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "V_pass", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: DataVolume::UNIT, reason: "a data volume cannot be negative" });
    }
    if answer.get() > 1000000000000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "V_pass", value: answer.get(), bound: 1000000000000000.0, edge: Edge::Upper, unit: DataVolume::UNIT, reason: "above a petabit per pass no modem in this class runs" });
    }
    Ok(answer)
}
