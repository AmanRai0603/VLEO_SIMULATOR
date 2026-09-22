// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What F10.7 must the design still work at as a sustained level over the mission window, on the low side?
///
/// `F107_cold_long = F107_central - 1.28 * sigma_total`
///
/// Source: `noaa_swpc`
///
/// The lower edge of the mean band, and the scenario the study calls
/// coldmean. Its hot twin, sw_f107_design_long, is the level the mission must
/// SURVIVE; this is the level it must still WORK at, and they bound different
/// failures. A cold sky is thin air. Thin air is less drag, which sounds like
/// good news and is not: it is the case where an aerodynamically controlled
/// vehicle has the least authority, where a differential-drag constellation
/// phases slowest, where a de-orbit at end of life takes longest, and where a
/// power budget gets the least flux into the array. A tool that publishes
/// only the hot edge has told the reader about half of their problem.
///
/// # Assumptions
///
/// * 1.28 is the confidence, and it is the 90th percentile while the run is called 95 per cent — fails when a reader takes the published band as a 95 per cent bound. Phi(1.28) = 0.8997, so this edge is the 10th percentile and not the 5th. A one-sided 95 per cent bound is 1.645 sigma, which at this sigma is a further 4.9 sfu DOWN. The daily half of the same band DOES use 0.95, so the two halves are not at one confidence, and this row reproduces that rather than silently repairing it Which number that is, is now a row of its own — sw_band_confidence, seeded by §30 B2 and unanswered, because the value needs a person. Until it carries one the multiplier is a literal in this hole and three others, each saying it is declared in the sheet while no sheet declares it. §44 measures what each answer costs: moving to 1.645 moves 47 rows and no KPI closure, leaves all five solar closures closing with 3.1 to 6.2 per cent less margin, and breaks no parity check in this repository
/// * The band is symmetric because sigma is a standard deviation — fails when the residuals are skewed, which they are. A forecast that misses hardest when activity is highest has a long high tail and a short low one, so the true 10th percentile of the residuals is nearer the centre than 1.28 sigma and this edge is a little too cold. It errs toward the conservative on THIS side — a colder cold case is a harder case for drag authority — but it is the wrong number, not a safe one
/// * One sigma covers the whole window — fails when sigma is not flat across the cycle — the source says so about its own number — so a window spanning a rise or a fall is given one width where it needs two
/// * The cold edge is a design case and not a nuisance — fails when it is read as the harmless side. Thin air is the case with the least aerodynamic control authority, the slowest differential-drag phasing, the longest end-of-life de-orbit and the least array flux. Two of those are mission-ending in their own way
pub const NODE_ID: &str = "sw_f107_cold_long";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xc6685d30fb5cc2f5;

pub fn evaluate(central: Ratio, spread: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : subtract 1.28 standard deviations of the rotation-forecast residual from the central expectation -> Ratio
    // 1.28 is declared in the sheet, not chosen here, and it is the SAME 1.28 the
    // hot edge adds: one band with two edges, not two bands. A coefficient that
    // appears only in a hole body is exactly what the gate's portable-maths
    // check exists to refuse, and a band whose two edges could drift apart is
    // what having it in one place prevents.
    let sustained: Ratio = central - spread * 1.28;
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = sustained;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107_cold_long", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_cold_long", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 60 sfu has never been observed and every relation reading F10.7 has no support there. On this row the guard means something specific: a sustained level below the record's own floor says the band is wider than the sky, which happens when a window near solar minimum is given a sigma measured across a whole cycle" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_cold_long", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 400 sfu the exospheric temperature relation is extrapolated past the largest recorded daily value. A COLD level there is arithmetic rather than sky — it means the spread has been added rather than subtracted, which is the one failure this row has that its twin does not" });
    }
    Ok(answer)
}
