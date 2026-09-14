// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What F10.7 must this design survive, at 95 per cent confidence?
///
/// `F107_req = 250 sfu at 95% confidence`
///
/// Source: `orbitt_case_c1`
///
/// The same quantity as req_01 and the same number, because in this subsystem
/// the solar flux IS F10.7. The layer-2 rows separate them and the subsystem
/// does not. That is a finding about the tree rather than a design decision,
/// and it is stated here rather than resolved by inventing a difference.
///
/// # Assumptions
///
/// * It is the same number as req_01 and the duplication is real — fails when the two are treated as independent requirements. l3_solar_ach_01 and l3_solar_ach_02 already say this about the achieved side — the solar flux this subsystem publishes is F10.7 and nothing else — so the two requirement rows are one commitment written twice. They are both written because the layer-2 rows sys_space_environment_solar_flux and sys_space_environment_f10_7 are separate and each needs a partner. Merging them is a change to the layer-2 tree, not to this subsystem.
/// * The 95 per cent is inherited from the achieved side, not chosen here — fails when a different confidence is wanted. There is no confidence row anywhere in this tree; the figure is 95 because sw_uncertainty_growth publishes a 95th percentile and sw_f107_design adds it. sw_band_coverage checks that the band is worth its label and measures the coverage at 0.9509, so the stated confidence is honest. A requirement at a different confidence would need that percentile re-measured, which is a change to sw_uncertainty_growth.
/// * A percentile is not a worst case, and a requirement written on one is not a survival guarantee — fails when the mission meets a day in the upper five per cent. By construction one day in twenty exceeds the band, and over a five-year mission that is a great many days. What 250 buys is that the design point is not exceeded by the typical excursion — not that it is never exceeded. The record's largest daily F10.7 is 343 sfu, well above this requirement, and a design that must survive that day needs a worst-case row rather than a percentile one.
pub const NODE_ID: &str = "l3_solar_req_02";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xef81e7b798882556;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(250.0, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "F107_req_95", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107_req_95", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_req_95", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the same floor as env_f107: below 60 sfu has never been observed, so a requirement there could never be met and is not a requirement" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_req_95", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the same ceiling as env_f107: above 400 sfu every consumer of F10.7 is extrapolating, and a requirement written past the range its consumers support is not checkable" });
    }
    Ok(answer)
}
