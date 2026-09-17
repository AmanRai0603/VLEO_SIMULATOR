// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What Kp does the system design to, in the worst three-hour slot of its design day?
///
/// `Kp_sys = l3_solar_interface.kp_peak_hotmean`
///
/// Source: `iaga_kp_ap`
///
/// The peak slot and not the daily mean, and the distinction is the whole
/// reason the subsystem measures a slot offset at all. Ap is a daily mean, Kp
/// is reported in eight three-hourly slots, and the published conversion run
/// on a daily mean lands ABOVE the mean of the eight and far BELOW the peak.
/// A design that converts Ap to Kp and stops is sized on a sky quieter than
/// the record's, on exactly the days a drag design is sized by.
///
/// # Assumptions
///
/// * It names one of ten Kp members and they span a quiet day to a severe storm — fails when the wrong one is named. The crossing's ten Kp values run from 1.27 to 8.00 at the declared window. All ten are dimensionless, in the same declared domain, and produced by one node, so assembly checks that the variable exists and that its type matches and nothing checks that it is the one this row meant
/// * The peak slot is a MEDIAN correction, so half the days in its Ap bin exceed it — fails when this is read as a bound. sw_kp_slot_bias measures the median of max_8(Kp) - table(Ap) in nine bins of Ap. The median is the middle of a spread, not its top, and the correction knows only the daily mean and the bin it falls in — not whether the day was one long storm or seven quiet slots and one severe
/// * It receives and does not compute, and a reader here sees none of the subsystem's limitations — fails when a margin is taken against this number. It rests on an Ap that stacks two one-sided percentiles, a conversion that is a lookup with straight lines between 28 points, and a slot offset measured over one bundle version. None of that crosses the seam
pub const NODE_ID: &str = "sys_space_environment_kp";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xdad48006ee870c21;

pub fn evaluate(crossing: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : receive the peak-slot Kp of the disturbed design day across the seam -> Ratio
    // A layer-2 row receives; it does not compute. The identity is the point.
    //
    // The member received is kp_peak_hotday: the worst three-hour slot of the
    // disturbed single day. The crossing carries nine others and they run down
    // to 1.27, so the sheet says which this is and why.
    let received: Ratio = crossing;
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = received;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kp_sys", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_sys", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the crossing's own floor, restated: Kp is defined on 0 to 9 and a negative index is a sign error, not a quiet sky" });
    }
    if answer.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_sys", value: answer.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the crossing's own ceiling, restated: Kp is defined on 0 to 9. This member is the worst slot of the worst day, so it is the one of the ten nearest it — 8.00 at the declared window" });
    }
    Ok(answer)
}
