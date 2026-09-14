// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How long can the mission be before the record expects to exceed the design Ap?
///
/// `T_safe(Ap_design) = exp((Ap_design - 92.515531) / 40.926516)`
///
/// Source: `noaa_swpc`
///
/// The inverse of sw_storm_return_level, read against sw_ap_design. At the
/// declared G3 it is 2.62 years against a declared mission of 5 — so the
/// answer to whether the design is exceeded is yes, and this is when it
/// starts.
///
/// # Assumptions
///
/// * It is exceeded, and the number is when rather than whether — fails when the row is read as a safety margin. At G3 the answer is 2.6242 years and orbit_mission_duration is declared at 5, so the mission is 1.9 times longer than the design bound survives. At G2 it is 0.7365 years and at G1 0.3370 — every level the G scale offers below G4 is exceeded well inside a five-year mission. A design that must not be exceeded at five years needs Ap 158.4, which is G4 territory, and sw_storm_design_level refuses G4 on purpose
/// * It inherits the whole fit, including the part of it that rests on two storms — fails when the answer lands near the top of the fitted range. This is sw_storm_return_level's relation read backwards, so every limitation of that row applies here unchanged: the fit is log-linear through ranks 2 to 14 of a 28.197-year sample, the fitted domain is 0.5035 to 14.0986 years, and the top of it rests on two observations. At the G3 bound the answer of 2.62 years sits comfortably inside the fitted range, which is the one thing that makes this row trustworthy at the declared level and would not survive a design bound near Ap 200
/// * A return period is not a countdown and a mission is not guaranteed its share — fails when 2.62 years is read as time before failure. A once-per-2.62-years storm can arrive in the first month or not in ten years; what the number means is that the expected count of such days over a 2.62-year window is one. Over the declared five-year mission the expected count is 1.42 days, which sw_exceedance_rate measures directly and which is the figure to plan with. This row is the threshold where that count passes one, not a date on which anything happens
/// * Closed form, so it is exact where the table rows are not — fails when it is compared against an interpolated row and they disagree. The three exceedance rows read a three-point table and interpolate between anchors, which their sheets declare is wrong between them. This row evaluates the fit itself at any Ap the producer can supply, so it is exact at every point including the ones between G levels. If the two ever disagree about a value between anchors, this row is right and the tables are the approximation
pub const NODE_ID: &str = "sw_design_safe_duration";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xc1857582e6869082;

pub fn evaluate(ap_design: Ratio) -> Result<Time, Fault> {
    // ---- HOLE 1 : invert the storm return fit: the mission length at which this Ap recurs once -> Time
    // sw_storm_return_level's own fit, read backwards. Its constants, not a
    // second fit: Ap(T) = A + B ln(T) inverts to T = exp((Ap - A) / B), so the
    // two rows cannot disagree about any point.
    //
    // Closed form rather than a table, so this is exact at every Ap the
    // producer can supply and not only at the three G anchors.
    const A: f64 = 92.515531;
    const B: f64 = 40.926516;
    let years: f64 = pmath::exp((ap_design.get() - A) / B);
    let out: Time = match Time::from_unit(years, Unit::Year) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "T_safe", reason: "the declared unit does not match the declared type" }),
    };
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Time = out;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "T_safe", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 9467280.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_safe", value: answer.get(), bound: 9467280.0, edge: Edge::Lower, unit: Time::UNIT, reason: "the shortest this relation can return is 0.3370 years, at the G1 bound of Ap 48. A bound at 0.3 sits just under it. A value below would mean a design level under Ap 46, which is beneath anything the G scale calls a storm" });
    }
    if answer.get() > 94672800.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_safe", value: answer.get(), bound: 94672800.0, edge: Edge::Upper, unit: Time::UNIT, reason: "the longest is 2.6242 years, at the G3 bound of Ap 132, which is the top of the declared G range. A bound at 3 sits just above it. It is NOT a claim that no design survives longer: Ap 207 at G4 would reach 16.4 years, and sw_storm_design_level refuses G4 deliberately rather than this guard forbidding it" });
    }
    Ok(answer)
}
