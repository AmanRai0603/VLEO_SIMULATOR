// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What energy per bit does the link actually deliver at the design rate?
///
/// `Eb/N0 = C/N0 - 10*log10(R)`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "com_ebn0";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x02f5e9962027c2e5;

pub fn evaluate(c: Ratio, r: DataRate) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : subtract the data rate in decibels from the carrier to noise density -> Ratio
    let e: Ratio = Ratio::new(comms::eb_over_n0_db(c.get(), r));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = e;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "EbN0", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < -10.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "EbN0", value: answer.get(), bound: -10.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below -10 dB no coded link closes" });
    }
    if answer.get() > 40.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "EbN0", value: answer.get(), bound: 40.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 40 dB the link is overdesigned by orders of magnitude" });
    }
    Ok(answer)
}
