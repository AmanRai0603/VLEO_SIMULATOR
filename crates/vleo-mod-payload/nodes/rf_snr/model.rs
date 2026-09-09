// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What signal to noise ratio does the intercepted emitter present, as a linear ratio?
///
/// `SNR_rf = 10`
///
/// Source: `orbitt_case_c2`
pub const NODE_ID: &str = "pay_rf_snr";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x78343456f1d31f98;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(10.0, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "SNR_rf", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "SNR_rf", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.1 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "SNR_rf", value: answer.get(), bound: 0.1, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 0.1 the signal is not detectable at all" });
    }
    if answer.get() > 1000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "SNR_rf", value: answer.get(), bound: 1000.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 1000 the emitter is so strong that geolocation is not the limiting problem" });
    }
    Ok(answer)
}
