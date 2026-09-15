// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What does the solar-weather subsystem conclude, for a system reader who will not open it?
///
/// `F107_crossing = F107_design`
///
/// Source: `noaa_swpc`
///
/// One row, one number, no reaching in. Everything the subsystem establishes
/// — the record, the cycle, the storms, the slot bias, the spread —
/// arrives at sw_f107_design, and this carries that across to
/// sys_space_environment. A reader who wants the working opens l3_solar and
/// finds forty rows; a reader who wants the answer opens this one.
///
/// # Assumptions
///
/// * It carries the F10.7 driver only, and the subsystem concludes more than that — fails when sys_space_environment holds six rows — solar flux, F10.7, Ap, atmospheric density, thermospheric wind, atomic oxygen fluence — and this crossing answers one of them. The Ap side is measured and published inside the subsystem (sw_storm_return_level, and the peak-slot correction in sw_kp_slot_bias) and does not cross yet, because one row publishes one number and the convention allows exactly one crossing per subsystem. How a subsystem with more than one conclusion crosses is the same unsettled question as the kind above, and it is unsettled for all twenty-one interfaces, not just this one.
/// * It inherits every limitation of the row beneath it, and a system reader sees none of them — fails when this is the ordinary cost of a seam and it is worth stating where the seam is. The number crossing here is a 95th percentile and not a worst case, and its centre is the record's unconditional mean rather than the mean cycle at the mission's epoch — the epoch is published and this chain does not read it yet. A reader at layer 2 sees 228 sfu and a credibility vector, and would have to open sw_f107_design and then sw_central_expectation to learn any of that. The credibility travels; the assumptions do not.
pub const NODE_ID: &str = "l3_solar_interface";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xf3e237172ccd32a9;

pub fn evaluate(conclusion: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : carry the subsystem's F10.7 conclusion across the seam unchanged -> Ratio
    // A crossing carries; it does not compute. The one thing that can go wrong
    // here is that the seam alters what it is handed — a stray factor, an
    // unasked-for unit conversion, a clamp inherited from the wrong row — and
    // both sides would still look plausible. So this is the identity, and the
    // fixtures beside it pin the identity at four real values.
    //
    // The declared range is the producer's own, restated so a system reader sees
    // the limit without opening the subsystem. It therefore guards nothing this
    // line can break, and that is correct: a crossing that narrowed the range it
    // carried would be changing the answer.
    let crossing: Ratio = conclusion;
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = crossing;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107_crossing", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_crossing", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the same floor as env_f107 and sw_f107_design: below 60 sfu has never been observed and every relation reading F10.7 has no support there. A crossing that narrowed or widened the range it carries would be changing the answer, so it declares the producer's own bounds" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_crossing", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the same ceiling as env_f107 and sw_f107_design: above 400 sfu the exospheric temperature relation is extrapolated past the largest recorded daily value. Restating it here means a system reader sees the limit without opening the subsystem" });
    }
    Ok(answer)
}
