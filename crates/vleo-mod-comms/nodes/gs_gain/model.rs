// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much does the ground antenna concentrate the received power?
///
/// `G = eta*(pi*D/lambda)^2`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "com_gs_gain";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xe2fb8c5b1bba699a;

pub fn evaluate(d: Length, f: Frequency, e: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : apply the same aperture relation to the ground antenna -> Ratio
    let g: Ratio = Ratio::new(comms::aperture_gain_db(d, f, e));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = g;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "G_r", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "G_r", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a ground aperture of this class always has positive gain" });
    }
    if answer.get() > 80.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "G_r", value: answer.get(), bound: 80.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 80 dBi is a deep-space aperture" });
    }
    Ok(answer)
}
