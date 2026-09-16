// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What daily planetary Ap must this design survive?
///
/// `Ap_req = ap(G4) = 207`
///
/// Source: `orbitt_case_c1`
///
/// The G3 design capability of Ap 132 with a 14 per cent margin on it. The
/// record expects 158.4 over the declared five-year mission, so this
/// requirement is NOT met — and the three exceedance rows say what that
/// costs: about 1.42 days outside the bound over the mission, in one or two
/// one-day events, on the declining side of the cycle where the epoch sits.
///
/// # Assumptions
///
/// * 207 is a SURVIVAL commitment and not an operating capability, and the two are easy to confuse — fails when a reader takes this as the level the vehicle works through. It is not. sw_storm_design_level declares G3 and sw_ap_design turns it into 132, and that is the operating level; nothing about it moved. This row says a G4 storm does not end the mission, which sw_storm_design_level's own reason_upper describes as the right treatment for G4 — handled 'by operating through the event rather than by building for it'. A design that claimed to OPERATE at G4 would change sw_storm_design_level, and four exceedance rows would move with it.
/// * This closure now holds and the OPERATING one still does not, and the second is the one that matters — fails when a passing closure here is read as the storm case being closed. 158.4 sits between the two levels: above the 132 the vehicle operates through, below the 207 it must survive. So the mission meets a storm it cannot work through, and the size of that is what a designer needs — 0.284 days a year above 132, 1.42 days over the mission, about 1.24 events averaging 1.14 days each, longest run in 29 years 2 days. Roughly thirty-four hours outside the operating environment across five years. Nothing in the tree compares 158.4 against 132 automatically; the three exceedance rows measure it and a person reads them.
/// * Switching the G level moves the design value and the exceedance statistics, and not this row — fails when somebody expects the requirement to follow the switch. sw_storm_design_level is the input a design turns to ask what a different storm level costs — G1 gives 48, G2 gives 80, G3 gives 132 — and sw_ap_design and all three exceedance rows move with it. At G2 the exceedance rate is 1.24 days a year, 6.21 days over the mission in 5.1 events. This row does not move: it is a commitment, and a commitment that silently tracked the design would never be violated and would therefore never be a requirement. Changing it is a separate, deliberate act.
/// * A daily mean, which is the wrong shape for what a storm does — fails when the storm is short or long. Daily Ap averages eight three-hourly slots, so a violent six-hour storm and a mild day-long disturbance can share a value, and a requirement written on the daily mean is satisfied by both. The atmosphere responds to the integral with a lag, not to the daily mean. The record's largest daily Ap is 273 — above this requirement — and it is one day in 29 years.
/// * It is met by the record and the record is 29 years long — fails when the mission meets something the record has not seen. The return level this is checked against is fitted through ranks 2 and 3 of a 28.2-year sample at the mission's five-year period, and the largest event in that sample, Ap 273, has an apparent return period of 28.2 years for no reason but that it is the largest thing in 28.2 years. Events well beyond this requirement are known from longer proxy records. A requirement that a 29-year record cannot violate is not thereby safe.
pub const NODE_ID: &str = "l3_solar_req_03";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xd3d28334dd6938b2;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(207.0, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "Ap_req", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Ap_req", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 20.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_req", value: answer.get(), bound: 20.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 20 the requirement would be under the level at which the record's storms begin — the median day is Ap 7 and sw_storm_return_level's own floor is 20 — so a requirement there could not be met by any mission and is not a requirement" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_req", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "400 is the top of the published ap table, the value at Kp 9. A requirement above it is off the scale the G levels are defined on and could not be expressed as a G level at all" });
    }
    Ok(answer)
}
