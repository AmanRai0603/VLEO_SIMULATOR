// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What Ap must the design still work at as a sustained level over the mission window, on the low side?
///
/// `Ap_cold_long = Ap_central - 1.28 * sigma_ap`
///
/// Source: `noaa_swpc`
///
/// The lower edge of the Ap mean band, and the Ap column of the scenario the
/// study calls coldmean. Its hot twin, sw_ap_design_long, is the disturbed
/// sky the design must survive; this is the quiet sky it must still work in.
/// A quiet geomagnetic field is the case where the thermosphere is coolest
/// and thinnest, and on the Ap side it also sets the floor of what a
/// magnetometer sees and what a magnetorquer has to push against. Quiet is
/// not the absence of a design case.
///
/// # Assumptions
///
/// * 1.28 is the confidence, and it is the 90th percentile while the run is called 95 per cent — fails when a reader takes the published band as a 95 per cent bound. Phi(1.28) = 0.8997, so this edge is the 10th percentile and not the 5th. A one-sided 95 per cent bound is 1.645 sigma, a further 1.3 down at this sigma. The daily half of the same band DOES use 0.95, so the two halves are not at one confidence, and this row reproduces that rather than silently repairing it
/// * A symmetric band on a quantity truncated at zero — fails when the centre is small. Ap cannot be negative, so the true low edge of any band is bounded by the centre itself, and a symmetric subtraction of 1.28 sigma ignores that. With this sigma the crossing is at a centre near 4.6, which is a deep-minimum window rather than an impossible one. The guard catches it; the arithmetic does not know about it
/// * One sigma covers the whole window — fails when sigma is not flat across the cycle, and Ap's is least flat of all — geomagnetic activity peaks in the DECLINING phase rather than at maximum, when coronal holes are largest and high-speed streams recur. A window spanning that transition is given one width where it needs two
/// * The quiet edge is a design case and not a nuisance — fails when it is read as the harmless side. A quiet field is the coolest, thinnest thermosphere, the weakest signal a magnetometer has to work with and the least torque a magnetorquer can generate. The last of those sizes an actuator
pub const NODE_ID: &str = "sw_ap_cold_long";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x46b9cc8ade2f2ef2;

pub fn evaluate(central: Ratio, spread: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : subtract 1.28 standard deviations of the rotation-forecast residual from the central Ap expectation -> Ratio
    // The same 1.28 the hot edge adds, declared in the sheet rather than chosen
    // here: one band with two edges. The subtraction is symmetric although Ap is
    // not — Ap floors at zero and a standard deviation has no sides — which the
    // sheet argues about and the guard below catches.
    let sustained: Ratio = central - spread * 1.28;
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = sustained;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Ap_cold_long", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_cold_long", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Ap floors at zero — a perfectly quiet day is Ap 0 and there is nothing below it. On this row the guard is load-bearing rather than decorative: a symmetric band subtracted from a small centre reaches below zero, and what comes out then is arithmetic rather than sky" });
    }
    if answer.get() > 300.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_cold_long", value: answer.get(), bound: 300.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 300 the level exceeds the largest daily Ap in the record, 273. A COLD level there means the spread has been added rather than subtracted, which is the one failure this row has that its twin does not" });
    }
    Ok(answer)
}
