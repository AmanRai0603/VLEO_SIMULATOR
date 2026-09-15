// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// Where is F10.7 expected to be by the time the mission is there?
///
/// `F107_central(L) = w*F107_today + (1-w)*114.8437,  w = exp(-L / 27 d)`
///
/// Source: `noaa_swpc`
///
/// The centre of the F10.7 design value; sw_uncertainty_growth supplies the
/// spread around it and sw_f107_design adds the two together. Today's flux is
/// worth something at a lead of days and nothing at a lead of years, so this
/// is the one number that has to know how far ahead it is being asked about.
///
/// # Assumptions
///
/// * The climatology is the record's UNCONDITIONAL mean, not the mean cycle at the date the mission flies — fails when this is the honest limit of the row and it is a missing input, not a modelling choice. prf_design evaluates the mean cycle AT THE TARGET DATE, so a mission flying through solar maximum gets a different central value from one flying through minimum. Doing that needs a mission epoch, and THE OBSTACLE THAT PUT THIS ROW HERE IS GONE: sys_mission_requirements_mission_epoch is published at 9862 d, seven rows in this subsystem read it across the layer boundary, and sw_mean_cycle_level already publishes the mean cycle at a phase — 108.14 sfu at the declared epoch's 0.6194, against the 114.84 this row uses. So this is now a pending DECISION rather than a missing input, and it is a decision because this row is on the main branch: switching it changes a published output, moving sw_f107_design from 228.14 to 221.44 sfu and every drag and lifetime number downstream of it. Until somebody makes that call the row remains sized on no particular part of the cycle, and the record says that is worth a great deal — its F10.7 runs from 64 to 343 sfu, with a 5th percentile of 68 and a 95th of 201.
/// * Persistence is carried for completeness and is worth nothing at any mission lead — fails when the weight is exp(-L/27) with L in days, so at the shortest mission the declared input range allows — half a year — today's flux contributes 0.114% and by one year it contributes 0.00013%. The term is right and it is inert: this row's answer is the climatology to four decimal places for every lead a mission can ask about. It is kept because the relation is the study's, and removing it would make the row silently wrong for the short-lead use nothing in this tree currently makes. Anyone wanting the 27-day outlook wants a different row.
/// * The blend shape is a choice, and 27 days is the solar rotation rather than a fitted constant — fails when an exponential decay into a constant is one of several defensible ways to hand over from persistence to climatology, and prf_design cites no source for the form. 27 days is the synodic solar rotation, so the choice says persistence dies over about one turn of the Sun — physically reasonable and not measured here. Nothing in this row's declared range is sensitive to it, because every mission lead is far past the handover.
pub const NODE_ID: &str = "sw_central_expectation";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x04ee9e360dd3e1c5;

pub fn evaluate(today: Ratio, lead: Time) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : weight today's flux against the record climatology by the lead, one solar rotation as the timescale -> Ratio
    // The record's mean F10.7 over its 10,316 observed days, measured on
    // solar-weather@2026.09.14. It is the climatology this hands over to, and the
    // sheet says plainly that it is the UNCONDITIONAL mean rather than the mean
    // cycle at the mission's date — which needs an epoch no row publishes yet.
    const CLIMATOLOGY: f64 = 114.8437378829;
    // One synodic solar rotation, prf_design's own timescale for persistence. At
    // every lead the declared input range allows this weight is below 0.0012, so
    // the term is inert and kept only because the relation is the study's.
    const TAU_DAYS: f64 = 27.0;
    let w: f64 = pmath::exp(-lead.days() / TAU_DAYS);
    let central: Ratio = Ratio::new(w * today.get() + (1.0 - w) * CLIMATOLOGY);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = central;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107_central", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_central", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the answer is a weighted blend of today's F10.7 and the record's mean, so it can never leave the interval between them. env_f107's own lower bound is 60 because below 60 sfu has never been observed and the fit has no support there; the same floor applies to a blend of it" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_central", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "env_f107's upper bound is 400, above which the exospheric temperature relation is extrapolated past the largest recorded daily value. A blend cannot exceed its larger input, so this bound catches a broken weight rather than an extreme sky" });
    }
    Ok(answer)
}
