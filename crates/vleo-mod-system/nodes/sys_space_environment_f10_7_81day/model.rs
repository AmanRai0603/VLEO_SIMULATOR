// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What 81-day mean F10.7 does the single-day design value ride on?
///
/// `F107bar_sys = l3_solar_interface`
///
/// Source: `noaa_swpc`
///
/// The companion to sys_space_environment_f10_7, and NOT the same number. A
/// hot day rides on the sustained hot level beneath it, so the pair for the
/// design case is 124.14 daily on 104.07 background, not 124.14 on 124.14.
/// That distinction is the study's own and it is why its driver set has
/// separate f107 and f107bar columns: the three *mean scenarios have the two
/// equal, because they ARE the window mean, and the two *day scenarios do
/// not.
///
/// # Assumptions
///
/// * f107bar_hotday is the hot MEAN, not the 81-day mean of the hot day, and the two are different numbers — fails when a reader assumes the name means the latter. It is 104.07 where the hot day is 124.14. A *day scenario is a single day riding on the sustained level beneath it, so its 81-day companion is that sustained level; only the three *mean scenarios have the daily value and the 81-day mean equal. Taking the wrong one puts the atmosphere's background state twenty sfu out
/// * It pairs with the SINGLE-DAY flux row and not the sustained one — fails when somebody pairs it with sys_space_environment_solar_flux and thinks they have two numbers. That row carries 104.07 and so does this one, because the hot day's 81-day companion IS the hot sustained level — the two rows are the same value seen from two roles. The pair a density model wants is (124.14 daily, 104.07 background); the pair (104.07, 104.07) is the sustained scenario, which is a different case and not wrong, only different
/// * It receives and does not compute, and a reader here sees none of the subsystem's limitations — fails when a margin is taken against this number. It is a 1.28-sigma band edge — the 90th percentile, while the run is labelled 95 per cent — on a centre that beyond one cycle past cycle 25's maximum is scaled by the mean amplitude of two completed cycles whose peaks differ by 41 per cent
pub const NODE_ID: &str = "sys_space_environment_f10_7_81day";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xc1065c44dfd1ed58;

pub fn evaluate(crossing: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : receive the 81-day mean the design day rides on across the seam -> Ratio
    // A layer-2 row receives; it does not compute. The identity is the point, and
    // the fixtures beside it pin that the number arriving is the number leaving.
    //
    // The member received is f107bar_hotday, which is the hot MEAN and not the
    // 81-day mean of the hot day — a *day scenario rides on the sustained level
    // beneath it. The sheet argues about why that name is easy to misread.
    let received: Ratio = crossing;
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = received;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107bar_sys", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107bar_sys", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the crossing's own floor, restated: an 81-day mean cannot sit below a floor every day of it respects" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107bar_sys", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the crossing's own ceiling, restated: above 400 sfu every consumer of F10.7 is extrapolating" });
    }
    Ok(answer)
}
