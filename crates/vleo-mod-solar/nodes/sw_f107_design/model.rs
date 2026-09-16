// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What F10.7 should this design be sized to?
///
/// `F107_design = F107_central + dF107_p95`
///
/// Source: `noaa_swpc`
///
/// The centre comes from sw_central_expectation, the spread from
/// sw_uncertainty_growth, and the confidence is the 95th percentile that
/// sw_uncertainty_growth publishes — so this is a value the mission should
/// not exceed in 95 per cent of histories, not a value it will see. NOTHING
/// READS THIS ROW ANY MORE, AND THAT IS A DECISION SOMEBODY SHOULD MAKE. It
/// used to be the subsystem's headline: l3_solar_ach_01, l3_solar_ach_02 and
/// l3_solar_interface all restated it. §20 step 5 re-pointed the two
/// closures at sw_f107_design_long and sw_f107_design_short, and §20 step 4
/// gave the interface the whole driver set, so this row now has no consumers
/// at all. It is not wrong and it was not replaced — it answers a DIFFERENT
/// question from the design rows. This is a one-sided 95th-percentile
/// PERSISTENCE growth: how far might the flux drift from today's value over
/// the mission. sw_f107_design_long is a band width: how wrong has the
/// pattern historically been about the level a rotation sits at. Both are
/// defensible and they are not the same question, which sw_f107_design_long's
/// own question note says in as many words. But a published number nothing
/// reads is a number nobody checks, which is the standard this subsystem
/// holds itself to elsewhere. The three options are to give it a consumer, to
/// mark it `state = "deprecated"`, or to leave it published with this
/// paragraph against it. The third is what is here, because retiring a row is
/// a decision about the tree and an agent may not make one.
///
/// # Assumptions
///
/// * 95% and no other confidence, because that is the percentile the spread row publishes — fails when prf_design offers p50, p90, p95 and p99 and expects the caller to pick what the mission needs. This row inherits p95 from sw_uncertainty_growth and cannot be asked for another: a mission needing p99 is reading a number about 32 sfu too small at a one-year lead. Changing the confidence means changing the row underneath, which is where the percentile is chosen and declared.
/// * It cannot tell solar maximum from solar minimum, because no row in this tree publishes a date — fails when the whole of this limitation belongs to sw_central_expectation and it is repeated here because this is the row a system reader opens. THIS IS NOW FIXED AND THE FIX MOVED THIS ROW. The centre used to be the record's unconditional mean, 114.84 sfu, which gave a mission through solar maximum and one through minimum the same number; it is now the cycle analogue averaged over the mission's own dates, 86.85 sfu at the declared epoch, and this row fell from 228.14 to 200.14 with it. What remains is the weaker half of the same limitation: beyond one cycle past cycle 25's maximum the analogue is scaled by the mean amplitude of two completed cycles, whose peaks differ by 41 per cent, so a long mission is reading a level with a wide and unpublished uncertainty on it.
/// * Adding a percentile of the CHANGE to a central value is not the same as the percentile of the VALUE — fails when the record's own 95th percentile of daily F10.7 is 201 sfu, while this construction returns 228 at a five-year lead. The two answer different questions — the highest flux a day is likely to show, against how far the flux can move from its central expectation — and the second is the larger because it compounds where the centre sits with how wrong the centre can be. A reader who wants 'the 95th percentile of F10.7' wants the record, not this row.
pub const NODE_ID: &str = "sw_f107_design";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x83cf717ba776d8c7;

pub fn evaluate(central: Ratio, spread: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : add the 95th-percentile growth over the mission to the central expectation -> Ratio
    // The study's construction, and the whole of it: the centre plus the spread.
    // Both come from rows that declare where their numbers came from and what
    // they cannot do, so there is nothing to measure or choose here. The
    // confidence is whatever sw_uncertainty_growth publishes, which is the 95th
    // percentile, and the sheet says a mission needing another one must change
    // that row rather than this one.
    let design: Ratio = Ratio::new(central.get() + spread.get());
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = design;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107_design", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_design", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 60 sfu has never been observed and every relation reading F10.7 has no support there — env_f107's own floor, and a design value below it means the spread has been subtracted rather than added" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_design", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 400 sfu the exospheric temperature relation is extrapolated past the largest recorded daily value, which is env_f107's stated reason for the same bound. This guard is reachable: a central expectation near the top of its range plus a fifteen-year spread would exceed it, and it should refuse rather than hand a consumer a flux it cannot model" });
    }
    Ok(answer)
}
