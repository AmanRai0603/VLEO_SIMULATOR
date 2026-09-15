// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What F10.7 does the record say this mission will present, at 95% confidence?
///
/// `F107_ach = sw_f107_design`
///
/// Source: `noaa_swpc`
///
/// The F10.7 driver on the achieved side. It is the same quantity as ach_01
/// in this subsystem because the solar flux IS F10.7 here — the layer-2
/// rows separate them and the subsystem does not, which is a finding rather
/// than a design and is stated in the assumptions.
///
/// # Assumptions
///
/// * It inherits every limitation of the row it restates, and a closure reading it sees none of them — fails when the same cost as the interface, and worth repeating on the row a closure actually binds. The number is sized on the record's unconditional mean rather than on the mean cycle at the mission's epoch — the epoch is published and this chain does not yet read it, which is an open decision on sw_central_expectation worth 6.7 sfu; where it is a percentile it is the 95th and not a worst case; and where it is a return level its top end rests on two observations in 28.2 years. A margin computed from this row against a capability carries none of that, and will look like a clean number either way.
/// * ach_01 and ach_02 carry the same number, because at layer 3 the solar flux IS F10.7 — fails when sys_space_environment declares sys_space_environment_solar_flux and sys_space_environment_f10_7 as separate rows, and the seeder mirrored both into this group. In the study they are one quantity: F10.7 is the solar flux index, and nothing in prf_drivers distinguishes them. So two rows here answer with one number, which is honest but redundant, and the redundancy belongs to the layer-2 decomposition rather than to this subsystem. Whoever settles what an interface publishes should settle this at the same time — either sys_space_environment_solar_flux means something else, such as the headline with its credibility, or one of the two rows should not exist.
pub const NODE_ID: &str = "l3_solar_ach_02";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x7d362c230887ce84;

pub fn evaluate(conclusion: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : restate the subsystem's conclusion on the achieved side of the closure -> Ratio
    let ach: Ratio = conclusion;
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = ach;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107_ach", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_ach", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the same floor as env_f107: below 60 sfu has never been observed" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_ach", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the same ceiling as env_f107: above 400 sfu every consumer of F10.7 is extrapolating" });
    }
    Ok(answer)
}
