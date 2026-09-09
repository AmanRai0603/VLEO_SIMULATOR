// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How many charge and discharge cycles does the battery see?
///
/// `N = t_mission/T_orbit`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "pwr_battery_cycles";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x65d0729cb5ce91c2;

pub fn evaluate(tm: Time, to: Time) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : divide the mission duration by the orbital period -> Ratio
    let n: Ratio = Ratio::new(power::battery_cycles(tm, to));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = n;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "N_cyc", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "N_cyc", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a cycle count cannot be negative" });
    }
    if answer.get() > 1000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "N_cyc", value: answer.get(), bound: 1000000.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above a million cycles no cell chemistry has data" });
    }
    Ok(answer)
}
