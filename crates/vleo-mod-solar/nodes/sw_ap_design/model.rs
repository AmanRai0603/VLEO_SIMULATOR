// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What daily Ap is this design built to survive?
///
/// `Ap_design(G) = ap at the top Kp of that G band: G1 -> 48, G2 -> 80, G3 -> 132`
///
/// Source: `iaga_kp_ap`
///
/// The design-side companion to sw_storm_return_level. That row says what the
/// record will present over the mission; this says what the vehicle is built
/// for, from the G level sw_storm_design_level declares. At the default G3
/// the answer is 132 and the record's five-year expectation is 158.4, and the
/// gap between them is the finding.
///
/// # Assumptions
///
/// * The design value is BELOW what the record expects over the mission, and that is not a defect in either row — fails when the two are read as competing answers. At G3 this row gives 132; sw_storm_return_level at the declared five-year mission gives 158.4, and the record's days nearest that Ap all reached Kp_max 9. So the sky a five-year mission should expect is stronger than the sky the vehicle is sized for, by about 20 per cent in daily Ap. The gap is real and is closed by operations rather than by structure: 49 days in 29 years reach G4 and 16 reach G5, and a mission flies through those rather than being built for them. Both numbers belong in the tree precisely so the gap is on a row instead of in somebody's head
/// * The ceiling of the band, not its middle, and that choice is worth twice the number — fails when a typical G3 day is wanted rather than a bound. Days in the record whose Kp_max is exactly 7 have daily Ap from 15 to 96 with a median of 51 — so the typical G3 day is 51 and this row publishes 132. The ceiling is correct for sizing, because a design bound must hold for the worst day inside the level it claims, and 132 is the value a day would reach if all eight slots sat at Kp 7. It is conservative by construction: no G3 day in 29 years came within a third of it
/// * The published ap table is the conversion and it is not linear — fails when a G level is interpolated. The ap values at Kp 5, 6 and 7 are 48, 80 and 132 — ratios of 1.67 and 1.65, so the scale is close to geometric and a linear reading between levels is wrong by tens of nanotesla. The relation is a lookup on three integer levels and nothing between them is defined, which is why sw_storm_design_level is bounded to integers 1 to 3 and why this row does not interpolate
/// * Daily Ap ignores everything about the storm except its size — fails when duration or timing matters. A day at Ap 132 that recovers overnight and the first day of a three-day storm at the same Ap are the same number here. The atmosphere does not treat them alike — density lags the driver and a sustained storm heats the thermosphere far more than a single disturbed day — so a drag design taking this number alone gets the peak and not the integral. sw_event_duration carries the dwell for F10.7; nothing in this group yet carries it for Ap
pub const NODE_ID: &str = "sw_ap_design";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xdd6525adf4372a65;

pub fn evaluate(g_level: Ratio) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::sw_ap_design. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::sw_ap_design::evaluate(g_level.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "Ap_design")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Ap_design", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 40.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_design", value: answer.get(), bound: 40.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the lowest value this relation can return is 48, at G1. A bound at 40 sits just under it and refuses anything that would design to less than a minor storm, which is not a design case: the record has 1358 days at G1 or above, 47 a year" });
    }
    if answer.get() > 140.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_design", value: answer.get(), bound: 140.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the highest this relation can return is 132, at G3, which is the top of the declared G range. A bound at 140 sits just above it and catches a G level outside the range or a misread table. It does NOT bound the sky: the record's largest daily Ap is 273 and the five-year return level is 158.4, both above this bound, and neither passes through this row" });
    }
    Ok(answer)
}
