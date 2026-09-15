// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How many days a year does the record go above the design Ap?
///
/// `R_exc(Ap_design) = days with Ap >= Ap_design, per year of record`
///
/// Source: `noaa_swpc`
///
/// The exceedance rate of sw_ap_design, measured over 28.197 years. Multiply
/// by the mission length for the number of days the vehicle spends outside
/// what it was built for: at G3 and five years that is 1.42 days.
///
/// # Assumptions
///
/// * The design bound IS exceeded, and the rate is the whole argument for accepting it — fails when the bound is read as a limit the sky respects. At G3, Ap 132, the record holds 8 days above it in 28.197 years — 0.284 a year, one day in 1287. Over the declared five-year mission that is 1.42 days. At G2, Ap 80, it is 35 days, 1.24 a year, 6.21 days per mission; at G1, Ap 48, it is 131 days, 4.65 a year, 23.2 days per mission. A design that must never be exceeded cannot be built at any of these levels, and the decision is about how many days of exceedance are tolerable rather than whether there are any
/// * Three points and a straight line between them, on a quantity that is anything but straight — fails when the answer is read anywhere except at 48, 80 or 132. Those are the only three values sw_ap_design can return, because the G scale is defined on integers and the producer's range is 1 to 3. The relation interpolates between them because Table1 must do something, and the interpolation is wrong: exceedance rate against threshold is close to exponential — 4.65, 1.24, 0.284 falls by a factor of 3.7 then 4.4 — so a linear reading between anchors overstates the rate badly in the middle. Nothing in the tree can reach those points, and this assumption exists so that the day something can, it is not believed
/// * It is a rate over a record, not a forecast, and the record is short where it matters — fails when the mission is the one that meets the rare event. 8 days above G3 in 28 years is a small sample: remove the single worst storm and the count is 7. The rate has a counting uncertainty of roughly its own square root, so 0.284 a year is 0.284 give or take 0.10 — a third of itself. Sizing on 1.42 days per mission when the honest range is roughly 0.9 to 2.0 is the correct use of it; treating 1.42 as a number with two decimal places of meaning is not
/// * 273 absent days, and they are not spread evenly over the risk — fails when the count is read as complete. observed_daily.csv is missing 2017-01-01 to 2017-09-30. That window sits on cycle 24's decline at phase 0.74 to 0.80, which is inside the band where exceedances concentrate — sw_exceedance_phase puts their median at 0.603 and their spread at 0.29 to 0.74. So the gap is in the risky part of the cycle, not the quiet part, and the true rate is more likely above this figure than below it. The denominator used is 28.197 years, the days actually present, so the gap does not dilute the rate; what it does is remove whatever happened in it
pub const NODE_ID: &str = "sw_exceedance_rate";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x7251b4673e76443c;

pub fn evaluate(ap_design: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : read the measured count of days at or above this Ap, per year of record -> Ratio
    // The three design levels sw_ap_design can return, and the days at or above the bound, per year of the 28.197 years the record actually holds,
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
        y: &[4.6458636761, 1.2412612875, 0.2837168657],
    };
    let out: Ratio = Ratio::new(TABLE.at(ap_design.get()));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = out;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "R_exc", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.1 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "R_exc", value: answer.get(), bound: 0.1, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the lowest rate this relation can return is 0.284, at the G3 bound of Ap 132. A bound at 0.1 sits under it and catches a table read at the wrong end; it is NOT a claim that no design level is exceeded less often, because a higher bound than G3 is outside the declared G range" });
    }
    if answer.get() > 6.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "R_exc", value: answer.get(), bound: 6.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the highest is 4.65, at the G1 bound of Ap 48. A bound at 6 sits just above it. A rate above 6 days a year would mean the design level had fallen to around Ap 40, which is below anything the G scale defines as a storm and below what sw_ap_design can return" });
    }
    Ok(answer)
}
