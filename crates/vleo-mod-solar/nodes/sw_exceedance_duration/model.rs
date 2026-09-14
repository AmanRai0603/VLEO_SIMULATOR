// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// Once the record goes above the design Ap, how long does it stay there?
///
/// `D_exc(Ap_design) = mean length of a run of consecutive days with Ap >= Ap_design`
///
/// Source: `noaa_swpc`
///
/// The mean length of a run of consecutive days at or above sw_ap_design. At
/// G3 it is 1.14 days — an exceedance is one disturbed day, not a siege.
/// That is the reason 1.42 days of exceedance per mission is survivable
/// rather than fatal.
///
/// # Assumptions
///
/// * Just over one day at every level, and that is the finding — fails when a long exceedance is assumed. Measured, the mean run is 1.271845 days at the G1 bound, 1.206897 at G2 and 1.142857 at G3, and the maxima are 5, 3 and 2 days. The higher the bound the shorter the run, which is what a threshold cutting further into a peaked distribution must do. So at G3 the vehicle is above its design Ap for about one day at a time, at most two — long enough to matter to an attitude budget or a drag transient, not long enough to be a sustained environment. The three numbers are so close together that switching G level changes how OFTEN far more than how LONG
/// * The mean is over 7 events at G3, and one of them is half the information — fails when the third decimal place is believed. At G3 the runs are six of one day and one of two, so the mean of 1.142857 is exactly 8 divided by 7. Remove the single two-day event and it is 1.000. The number is a mean over a sample small enough to write out, and it is published as a mean because that is what the study's field holds — but a designer should read it as 'one day, occasionally two'
/// * A one-day dip below the bound ends the exceedance — fails when a storm rides just under the threshold for a day and comes back. Runs break on the first day that fails the test, so a disturbed week with one quieter day in the middle counts as two exceedances rather than one, shortening the mean and raising the event count in sw_exceedance_rate. Allowing a one-day bridge would be as defensible and would give a different pair of numbers. The rule is stated rather than the number being presented as unique — and it is the same rule sw_event_duration uses for F10.7, so the two are at least consistent with each other
/// * Daily means, so a violent six hours and a disturbed day look the same — fails when the exceedance is short and sharp. Ap is the mean of eight three-hourly slots, so a storm that peaks for six hours and subsides can fail to lift the daily mean above the bound at all, and one that sits moderately high all day can pass it. This row therefore measures days on which the DAILY average exceeded the design value, which is a coarser event than the one a spacecraft feels. The record's three-hourly Kp is in observed_daily.csv and nothing in this group reads it yet
pub const NODE_ID: &str = "sw_exceedance_duration";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xcac9dfd4d47b900e;

pub fn evaluate(ap_design: Ratio) -> Result<Time, Fault> {
    // ---- HOLE 1 : read the measured mean length of a consecutive run at or above this Ap -> Time
    // The three design levels sw_ap_design can return, and the mean length of a consecutive run at or above the bound, in days,
    // counted outside this crate from bundles/solar-weather.
    //
    // Three anchors and nothing between them: the G scale is defined on
    // integers and the producer's range is 1 to 3, so Ap 48, 80 and 132 are the
    // only values that reach here. Table1 interpolates because it must and
    // clamps at both ends; the sheet declares that the interpolation is wrong
    // for this quantity and that nothing in the tree can currently reach it.
    use vleo_core::math::Table1;
    const TABLE: Table1 = Table1 {
        x: &[48.0, 80.0, 132.0],
        y: &[1.2718446601941749, 1.206896551724138, 1.1428571428571428],
    };
    let out: Time = match Time::from_unit(TABLE.at(ap_design.get()), Unit::Day) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "D_exc", reason: "the declared unit does not match the declared type" }),
    };
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Time = out;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "D_exc", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 86400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "D_exc", value: answer.get(), bound: 86400.0, edge: Edge::Lower, unit: Time::UNIT, reason: "a run is at least one day by construction. A value below 1 means the run-finding is broken rather than that exceedances are brief" });
    }
    if answer.get() > 432000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "D_exc", value: answer.get(), bound: 432000.0, edge: Edge::Upper, unit: Time::UNIT, reason: "the longest run at any design level in 29 years is 5 days, at the G1 bound. A mean above that would exceed every single event the record contains, which no averaging can produce" });
    }
    Ok(answer)
}
