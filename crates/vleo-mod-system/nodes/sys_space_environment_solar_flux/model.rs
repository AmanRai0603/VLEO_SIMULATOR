// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What solar flux does the system design to, as a level sustained for months at a time?
///
/// `F107_sys = l3_solar_interface`
///
/// Source: `noaa_swpc`
///
/// The headline of the space-environment heading, and the number an array is
/// sized on and a drag budget integrates. Its siblings carry what one DAY
/// inside it reaches: sys_space_environment_f10_7 for the flux and
/// sys_space_environment_ap for the geomagnetic index. Sustained and
/// single-day are two rows here for the same reason they are two rows in the
/// subsystem below: a design reads one or the other depending on what it is
/// sizing, and a tool that published only one would have decided for the
/// reader which of the two their problem is.
///
/// # Assumptions
///
/// * It receives and does not compute, and a system reader sees none of the subsystem's limitations — fails when a margin is taken against this number. It is a 1.28-sigma band edge — the 90th percentile, while the run is labelled 95 per cent — on a centre that beyond one cycle past cycle 25's maximum is scaled by the mean amplitude of two completed cycles whose peaks differ by 41 per cent. The credibility vector crosses the seam; the assumptions do not, and that is the ordinary cost of having a seam at all
/// * Kp does not reach layer 2 at all, and a density model needs it — fails when somebody writes sys_space_environment_atmospheric_density from the flux rows alone. The daily flux and the 81-day mean both arrive now; Kp does not cross, because the subsystem's three Kp relations each answer at one Ap and a driver set needs them at five. §20.8 names the three ways out. Until one is chosen a density row would have to convert Ap to Kp itself, which is the duplication the crossing exists to prevent
/// * It is the sustained level and not the single day — fails when somebody sizes a thermal transient on it. The single day is sys_space_environment_f10_7 at 124.14, twenty sfu higher. Reading the wrong one of the two under-sizes a transient case or over-sizes a steady one
pub const NODE_ID: &str = "sys_space_environment_solar_flux";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x1319a361ca5679a6;

pub fn evaluate(crossing: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : receive the subsystem's sustained flux conclusion across the seam -> Ratio
    // A layer-2 row receives; it does not compute. The one thing that can go
    // wrong here is that the seam alters what it carries — a stray factor, an
    // unasked-for unit conversion, a clamp inherited from the wrong row — and
    // both sides would still look plausible. So this is the identity, and the
    // fixtures beside it pin the identity at real values.
    //
    // The declared range is the crossing's own, restated so a system reader sees
    // the limit without opening the subsystem. It therefore guards nothing this
    // line can break, and that is correct.
    let received: Ratio = crossing;
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = received;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107_sys", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_sys", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the crossing's own floor, restated: below 60 sfu has never been observed and every relation reading F10.7 has no support there. A row that narrowed the range it received would be changing the answer" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_sys", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the crossing's own ceiling, restated: above 400 sfu the exospheric temperature relation is extrapolated past the largest recorded daily value" });
    }
    Ok(answer)
}
