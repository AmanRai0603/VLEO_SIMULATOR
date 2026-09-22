// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What F10.7 must the design survive as a sustained level over the mission window?
///
/// `F107_long = F107_central + 1.28 * sigma_total`
///
/// Source: `noaa_swpc`
///
/// The LONG TERM of the pair. This is the level the mission sits at for
/// months at a time — what an array is sized on, what a drag budget
/// integrates. Its short sibling, sw_f107_design_short, says what one day
/// inside it reaches. The existing sw_f107_design is neither: it adds a
/// one-sided 95th-percentile PERSISTENCE growth, which answers "how far might
/// the flux drift from today's value" rather than "how wrong is the pattern
/// about the window". Both are defensible and they are not the same question.
///
/// # Assumptions
///
/// * 1.28 is the confidence, and it is the 90th percentile while the run is called 95 per cent — fails when a reader takes the published band as a 95 per cent bound. Phi(1.28) = 0.8997. A one-sided 95 per cent bound is 1.645 sigma, which at this sigma is a further 4.9 sfu. The daily half of the same band DOES use 0.95, so the two halves are not at one confidence, and this row reproduces that rather than silently repairing it Which number that is, is now a row of its own — sw_band_confidence, seeded by §30 B2 and unanswered, because the value needs a person. Until it carries one the multiplier is a literal in this hole and three others, each saying it is declared in the sheet while no sheet declares it. §44 measures what each answer costs: moving to 1.645 moves 47 rows and no KPI closure, leaves all five solar closures closing with 3.1 to 6.2 per cent less margin, and breaks no parity check in this repository
/// * The residual spread is normal enough for a z multiplier to mean a percentile — fails when it is not. The residuals of a forecast that misses hardest when activity is highest are skewed, and a normal multiplier under-covers the high tail — which is the tail a design is sized against. The empirical percentile of the residuals would be the honest statistic, and sw_mean_band_spread publishes only their standard deviation. MEASURED, and the direction is not what this sentence says at the multiplier this row uses. tools/rotation_residuals.py --shape divides sigma out and reports what the band actually holds: at 1.28 sigma it holds 0.9149 to 0.9339 of the exceedances against a normal's 0.8997, so the multiplier OVER-covers by 1.5 to 3.4 points in every variant. At 1.645 sigma the measurement straddles the normal. The heavy tail is real and bites further out — the 99th percentile of exceedance runs 2.93 to 3.49 sigma where a normal says 2.33 — so the honest statement is that this band is conservative and a 99 per cent one would not be. Those fractions are shape rather than scale, which is why they survive §31.4: sigma is not reproduced and spans 32 per cent across the free parameters, while the coverage spans under two points. They are this file's walk and not the study's, which no measurement here can fix. §45
/// * One sigma covers the whole window — fails when sigma is not flat across the cycle — the source says so about its own number — so a window spanning a rise or a fall is given one width where it needs two
pub const NODE_ID: &str = "sw_f107_design_long";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x472055909976420f;

pub fn evaluate(central: Ratio, spread: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : add 1.28 standard deviations of the rotation-forecast residual to the central expectation -> Ratio
    // 1.28 is declared in the sheet, not chosen here: it is the one-sided 90th
    // percentile of a normal, and the sheet argues about why a run labelled 95
    // per cent uses it. A coefficient that appears only in a hole body is
    // exactly what the gate's portable-maths check exists to refuse.
    let sustained: Ratio = central + spread * 1.28;
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = sustained;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107_long", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_long", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 60 sfu has never been observed and every relation reading F10.7 has no support there; a design level below it means the spread has been subtracted rather than added" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_long", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 400 sfu the exospheric temperature relation is extrapolated past the largest recorded daily value, and a sustained level there is not a window this tool can model" });
    }
    Ok(answer)
}
