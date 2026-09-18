// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What daily planetary Ap does the record say this mission will present?
///
/// `Ap_ach = sw_storm_return_level`
///
/// Source: `noaa_swpc`
///
/// The geomagnetic driver on the achieved side: the storm that recurs once
/// per mission lifetime. Unlike the F10.7 pair this has no confidence
/// attached, because geomagnetic activity has no usable long-term forecast
/// and the study designs to a return period instead.
///
/// # Assumptions
///
/// * It inherits every limitation of the row it restates, and a closure reading it sees none of them — fails when the same cost as the interface, and worth repeating on the row a closure actually binds. The number is sized on the cycle analogue at the mission's own epoch rather than on the record's unconditional mean, which this chain now reads the epoch to do — it was an open decision on sw_central_expectation and it has been made, moving this row by 28.0 sfu; where it is a percentile it is the 95th and not a worst case; and where it is a return level its top end rests on two observations in 28.2 years. A margin computed from this row against a capability carries none of that, and will look like a clean number either way.
pub const NODE_ID: &str = "l3_solar_ach_03";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xc3f5fe862655aa6d;

pub fn evaluate(conclusion: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : restate the subsystem's conclusion on the achieved side of the closure -> Ratio
    let ach: Ratio = conclusion;
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = ach;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Ap_ach_return", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 20.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_ach_return", value: answer.get(), bound: 20.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the same floor as sw_storm_return_level: below 20 the answer is not a storm at all, and the record's median day is 7" });
    }
    if answer.get() > 230.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_ach_return", value: answer.get(), bound: 230.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the same ceiling as sw_storm_return_level: the fit's own reach at a return period equal to the record, 28.1971 years, which is Ap 229.18" });
    }
    Ok(answer)
}
