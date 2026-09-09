// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much of the Earth's own thermal emission does the spacecraft absorb?
///
/// `Q = q_IR*A*eps*F`
///
/// Source: `larson_wertz`
///
/// Unlike albedo this does not switch off in eclipse, which is what stops a
/// low-orbiting spacecraft getting as cold as an intuition built on higher
/// orbits expects.
pub const NODE_ID: &str = "thm_absorbed_ir";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x7e4b29a2028023fd;

pub fn evaluate(a: Area, e: Ratio, f: Ratio) -> Result<Power, Fault> {
    // ---- HOLE 1 : apply the mean outgoing longwave radiation through the view factor -> Power
    let q: Power = thermal::absorbed_earth_ir(a, e, f);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Power = q;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Q_ir", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Q_ir", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Power::UNIT, reason: "absorbed power cannot be negative" });
    }
    if answer.get() > 100000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Q_ir", value: answer.get(), bound: 100000.0, edge: Edge::Upper, unit: Power::UNIT, reason: "above 100 kW the surface is not this spacecraft" });
    }
    Ok(answer)
}
