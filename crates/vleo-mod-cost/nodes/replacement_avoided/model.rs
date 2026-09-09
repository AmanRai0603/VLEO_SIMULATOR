// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What does air-breathing propulsion save by removing the need to replace satellites when propellant runs out?
///
/// `C = (C_unit + C_launch)*N_replacements`
///
/// Source: `orbitt_case_c1`
///
/// The commercial argument for the whole programme, as a node with a source
/// and evidence rather than a slide.
pub const NODE_ID: &str = "cost_replacement_avoided";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xd78b109d944002ff;

pub fn evaluate(cb: Money, cp: Money, lc: Money, n: Ratio, td: Ratio) -> Result<Money, Fault> {
    // ---- HOLE 1 : count one avoided replacement of the whole constellation when thrust exceeds drag, and none when it does not -> Money
    let c: Money = cost::replacement_avoided(Money::new(cb.get() + cp.get()), lc, if td.get() >= 1.0 { n.get() } else { 0.0 });
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Money = c;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "C_avd", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "C_avd", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Money::UNIT, reason: "a saving cannot be negative" });
    }
    if answer.get() > 1000000000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "C_avd", value: answer.get(), bound: 1000000000000.0, edge: Edge::Upper, unit: Money::UNIT, reason: "above a trillion dollars the programme is not the one being designed" });
    }
    Ok(answer)
}
