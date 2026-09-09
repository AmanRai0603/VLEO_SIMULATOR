// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// Does the downlink close with margin to spare?
///
/// `M = Eb/N0 - EbN0_req - L_imp`
///
/// Source: `ecss_e_st_50`
pub const NODE_ID: &str = "com_link_margin";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x2a8a4b878cfc7a72;

pub fn evaluate(e: Ratio, er: Ratio, li: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : subtract the requirement and the implementation loss from what the link delivers -> Ratio
    let m: Ratio = Ratio::new(comms::link_margin_db(e.get(), er.get(), li.get()));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = m;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "M_link", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < -30.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "M_link", value: answer.get(), bound: -30.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below -30 dB the link is not close to closing and the design is wrong somewhere upstream" });
    }
    if answer.get() > 40.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "M_link", value: answer.get(), bound: 40.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 40 dB the link is overdesigned by orders of magnitude" });
    }
    Ok(answer)
}
