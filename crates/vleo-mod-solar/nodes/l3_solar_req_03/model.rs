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
/// `Ap_req = 200`
///
/// Source: `orbitt_case_c1`
///
/// A G4-class ceiling at 200. The vehicle is designed to G3, which is Ap 132,
/// and the record expects 158.4 over a five-year mission — so the
/// requirement is met while the design value is exceeded. The comment above
/// sets out the three numbers and why they are in that order.
///
/// # Assumptions
///
/// * 200 is the record's expectation with a round margin, and it lands one G level above the design — fails when the requirement is read as describing the vehicle. sw_storm_return_level at the declared five-year mission gives 158.4 and 200 is the next round number with usable headroom, about 26 per cent. On the G scale 200 sits between G3's ceiling of 132 and G4's of 207, so what has been committed to is G4-class survival while sw_storm_design_level declares G3. A requirement and a design value that disagree by a whole level is a thing to resolve, not a rounding.
/// * Switching the G level moves the design value and not this row — fails when somebody expects the requirement to follow the switch. sw_storm_design_level is the input a design turns to ask what a different storm level costs — G1 gives 48, G2 gives 80, G3 gives 132 — and sw_ap_design moves with it. This row does not: it is a commitment, and a commitment that silently tracked the design would never be violated and would therefore never be a requirement. Changing it is a separate, deliberate act.
/// * A daily mean, which is the wrong shape for what a storm does — fails when the storm is short or long. Daily Ap averages eight three-hourly slots, so a violent six-hour storm and a mild day-long disturbance can share a value, and a requirement written on the daily mean is satisfied by both. The atmosphere responds to the integral with a lag, not to the daily mean. The record's largest daily Ap is 273 — above this requirement — and it is one day in 29 years.
/// * It is met by the record and the record is 29 years long — fails when the mission meets something the record has not seen. The return level this is checked against is fitted through ranks 2 and 3 of a 28.2-year sample at the mission's five-year period, and the largest event in that sample, Ap 273, has an apparent return period of 28.2 years for no reason but that it is the largest thing in 28.2 years. Events well beyond this requirement are known from longer proxy records. A requirement that a 29-year record cannot violate is not thereby safe.
pub const NODE_ID: &str = "l3_solar_req_03";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x240b045afae8a9a4;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(200.0, Unit::One) {
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
