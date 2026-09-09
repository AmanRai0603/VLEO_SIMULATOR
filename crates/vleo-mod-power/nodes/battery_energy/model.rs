// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much energy must the battery hold to carry the load through eclipse?
///
/// `E = P_ecl*t_ecl/(DoD*eta_dis)`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "pwr_battery_energy";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xeae3279078fd1cbf;

pub fn evaluate(dem: Power, fe: Ratio, t: Time, dod: Ratio, ed: Ratio) -> Result<Energy, Fault> {
    // ---- HOLE 1 : carry the full demand through the eclipse fraction of one revolution at the declared depth of discharge -> Energy
    let e: Energy = power::battery_energy_required(dem, Time::new(fe.get() * t.get()), dod, ed);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Energy = e;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "E_batt", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "E_batt", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Energy::UNIT, reason: "battery energy cannot be negative" });
    }
    if answer.get() > 100000800.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "E_batt", value: answer.get(), bound: 100000800.0, edge: Edge::Upper, unit: Energy::UNIT, reason: "above 27 kWh the battery is heavier than the whole spacecraft" });
    }
    Ok(answer)
}
