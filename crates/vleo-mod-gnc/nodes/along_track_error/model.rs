// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How far out is a one-day orbit prediction, given how wrong the density model is?
///
/// `e = 1.5*da*t^2`
///
/// Source: `doornbos2011`
///
/// The term that dominates orbit prediction in this regime, and the reason
/// the density uncertainty is a published output rather than a footnote.
pub const NODE_ID: &str = "gnc_along_track_error";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x36ebbec60610e596;

pub fn evaluate(a: Acceleration, s: Ratio) -> Result<Length, Fault> {
    // ---- HOLE 1 : propagate the drag acceleration error, which is the density uncertainty applied to the drag, over one day -> Length
    let e: Length = gnc::along_track_error_from_drag(Acceleration::new(a.get() * s.get()), Time::from_days(1.0));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Length = e;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "e_at", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "e_at", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Length::UNIT, reason: "an error magnitude cannot be negative" });
    }
    if answer.get() > 10000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "e_at", value: answer.get(), bound: 10000000.0, edge: Edge::Upper, unit: Length::UNIT, reason: "above 10000 km the prediction is meaningless and the answer is that the model is unusable there" });
    }
    Ok(answer)
}
