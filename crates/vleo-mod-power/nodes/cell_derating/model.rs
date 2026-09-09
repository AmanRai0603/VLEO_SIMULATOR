// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much output does the array lose because it is hotter than its rating?
///
/// `f_T = 1 + k_T*(T_eq - 298.15)`
///
/// Source: `larson_wertz`
///
/// This node closes the loop. Without it power, thermal and propulsion are
/// three independent chains; with it they are one system, and the resolver
/// has to relax them together.
pub const NODE_ID: &str = "pwr_cell_derating";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x04fbe8e14dbad38f;

pub fn evaluate(k: Ratio, t: Temperature) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : apply the linear temperature coefficient about the 298.15 K reference, floored at 30% so the relaxation cannot chase a negative array -> Ratio
    let f: Ratio = Ratio::new(pmath::max(0.3, 1.0 + k.get() * (t.get() - 298.15)));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = f;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "f_T", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "f_T", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a derating factor cannot be negative" });
    }
    if answer.get() > 1.5 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "f_T", value: answer.get(), bound: 1.5, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 1.5 the array would be producing half again its rating, which the coefficient does not permit" });
    }
    Ok(answer)
}
