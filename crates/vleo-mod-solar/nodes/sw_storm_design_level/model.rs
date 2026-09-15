// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// Which NOAA G level is this design built to survive?
///
/// `G_design = 3   (NOAA G3, strong, Kp 7)`
///
/// Source: `noaa_swpc`
///
/// 1, 2 or 3 on the G scale — minor, moderate or strong. It is the switch:
/// sw_ap_design reads it and turns it into a daily Ap, so moving this row is
/// how a design asks what a different storm level would cost. Default G3.
///
/// # Assumptions
///
/// * G3 is a choice and the record says a five-year mission will exceed it — fails when the choice is read as sufficient. sw_storm_return_level, fitted on the record and read at the declared five-year mission, gives a daily Ap of 158.4; the days in the record nearest that value all reached Kp_max 9, which is G5. So the storm a five-year mission should expect is two levels above what this row designs to. That is not an error in either row — it is the gap between what the sky does and what a vehicle can be built for, and putting both numbers in the tree is the point. What closes it is operations, not structure
/// * The scale is on Kp and the design quantity is daily Ap, which are not the same measurement — fails when a G level is converted to a daily Ap as though the mapping were exact. Kp is three-hourly and the G level is the WORST slot in a day; daily Ap is the mean of eight slots. So a G3 day has one slot at Kp 7 and seven that may be anything below it, and the record shows exactly that spread: days whose Kp_max is 7 run from daily Ap 15 to 96, median 51. sw_ap_design takes the ceiling of that band rather than its median, and the sheet there says why
/// * Stopping at 3 is a statement about this vehicle class, not about the scale — fails when a mission that must survive G4 or G5 reads this row. The declared range refuses 4 and 5 rather than letting them be selected quietly, because a design sized for G4 is a different vehicle and the rest of this group's numbers — the return level, the band, the requirement — would all need revisiting together. A mission with that requirement should change this bound deliberately and re-run the closure, which is a visible act
pub const NODE_ID: &str = "sw_storm_design_level";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xdfa9bfd94d5e918f;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(3.0, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "G_design", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "G_design", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "G_design", value: answer.get(), bound: 1.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "G1 is the bottom of the NOAA scale. Below it there is no storm to design for — Kp 4 is 'active' and is where alerts begin, which sw_alert_threshold carries, not where storms do" });
    }
    if answer.get() > 3.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "G_design", value: answer.get(), bound: 3.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "G3 is the highest level this vehicle class is sized for. G4 occurs on 49 days in 29 years and G5 on 16, and both are handled by operating through the event rather than by building for it. A design that genuinely requires G4 changes this bound on purpose and re-runs the closure" });
    }
    Ok(answer)
}
