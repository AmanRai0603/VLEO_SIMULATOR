// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How far does the carrier move in frequency as the satellite passes?
///
/// `df = f*v/c`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "com_doppler";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x59e27edeff3abd6a;

pub fn evaluate(f: Frequency, v: Velocity) -> Result<Frequency, Fault> {
    // ---- HOLE 1 : apply the first-order Doppler relation at the ground track speed -> Frequency
    let d: Frequency = comms::doppler_shift(f, v);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Frequency = d;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "df", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "df", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Frequency::UNIT, reason: "a shift magnitude cannot be negative" });
    }
    if answer.get() > 10000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "df", value: answer.get(), bound: 10000000.0, edge: Edge::Upper, unit: Frequency::UNIT, reason: "above 10 MHz the receiver acquisition range is exceeded and the link never locks" });
    }
    Ok(answer)
}
