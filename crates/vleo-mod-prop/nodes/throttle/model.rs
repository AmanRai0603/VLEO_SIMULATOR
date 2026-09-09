// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much of what the thruster asks for can the bus actually give it?
///
/// `k = clamp((P_avail - P_other)/P_prop, 0, 1)`
///
/// Source: `orbitt_case_c1`
///
/// The node that makes the power budget bite on the physics rather than being
/// reported beside it.
pub const NODE_ID: &str = "prop_throttle";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xc16699a360a6c290;

pub fn evaluate(av: Power, pp: Power, pay: Power, ax: Power, cm: Power, th: Power, lh: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : subtract every other load from what is available and give the remainder to propulsion, capped at what it asked for -> Ratio
    let k: Ratio = { let other = (pay.get() + ax.get() + cm.get() + th.get()) * (1.0 + lh.get()); let spare = pmath::max(0.0, av.get() - other); Ratio::new(if pp.get() <= 0.0 { 0.0 } else { pmath::min(1.0, spare / pp.get()) }) };
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = k;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "k_thr", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "k_thr", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a throttle cannot be negative" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "k_thr", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the thruster cannot be given more than it asked for" });
    }
    Ok(answer)
}
