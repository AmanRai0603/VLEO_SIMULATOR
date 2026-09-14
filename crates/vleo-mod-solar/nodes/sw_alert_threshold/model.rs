// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// At what geomagnetic level does the issuing centre put out an alert?
///
/// `K_alert = the lowest threshold_value on threshold_var K in alerts.csv = 4`
///
/// Source: `noaa_swpc`
///
/// Not a property of the sky but of the people watching it, and it is the
/// operational floor a mission's own alerting should not sit below. Read from
/// the thresholds attached to the alerts as issued rather than chosen here.
///
/// # Assumptions
///
/// * Kp 4 is the floor and is not a storm — fails when it is read as a storm level. Kp 4 is 'active', one step below the G-scale, which starts at Kp 5 for G1. The record shows 3318 days reaching Kp_max 4 or more — 32.2 per cent of all days, 114 a year — so a mission that treated every alert as a storm would be reacting a third of the time. The alerts carry six distinct K thresholds and their counts fall steeply: 5262 alerts at K 4, 2953 at 5, 901 at 6, 233 at 7, 38 at 8 and 3 at 9. This row publishes the floor because it is the floor that says when watching begins
/// * There is a second threshold variable and this row does not carry it — fails when somebody needs the A-index alerts. 119 of the alerts are on threshold_var A rather than K, at values 20 (77 alerts), 30 (27) and 50 (15). Those are daily-index alerts and this row is the three-hourly K one. 8444 alerts carry no threshold at all — they are summaries, cancellations and electron-flux notices — so the 9408 K-threshold alerts are the population this number comes from
/// * Eleven of the parsed thresholds are nonsense and were not filtered out of the source — fails when the threshold column is trusted without looking. Alongside the K values 4 to 9 sit 414, 413, 425, 419, 430 and 408 — two alerts each for the first four, one each for the last two, eleven rows in total. They are a parser running two numbers together, not a Kp of 414. They do not affect this row, which takes the minimum of the legitimate values, but any relation that takes a maximum or a mean of that column will be wrong and nothing in the bundle marks them
/// * It is what was alerted on, not what should be — fails when a mission adopts it as its own trigger. The number reflects one centre's operational choices over 29 years, including changes in practice nobody recorded in this bundle. A spacecraft whose sensitivity to geomagnetic activity differs from the assumptions behind a public alert service needs its own threshold, and this row is the reference point for setting one rather than the answer
pub const NODE_ID: &str = "sw_alert_threshold";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x59407de4d5bccae5;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(4.0, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "K_alert", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "K_alert", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 4.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "K_alert", value: answer.get(), bound: 4.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9 and no alert in the record was issued below 4. A value under it would mean an alert on a quiet sky, which the issuing centre has never done in 29 years" });
    }
    if answer.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "K_alert", value: answer.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Kp 9 is the top of the scale; there is nothing above it to alert on. Only three alerts in the record carry a threshold of 9" });
    }
    Ok(answer)
}
