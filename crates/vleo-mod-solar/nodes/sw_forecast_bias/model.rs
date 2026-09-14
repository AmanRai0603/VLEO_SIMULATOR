// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// By how much does the published outlook miss the F10.7 that arrived?
///
/// `B_f107(L) = mean over issues of (F107_forecast - F107_observed) at lead L`
///
/// Source: `noaa_swpc`
///
/// Signed, forecast minus observed, so a negative answer means the outlook
/// was low. It is measured at the lead sw_outlook_lead declares, which is the
/// far end of the window where the miss is largest. A design that sizes drag
/// on a published outlook is exposed by exactly this number, in the unsafe
/// direction: a low F10.7 is a thin atmosphere and an optimistic lifetime.
///
/// # Assumptions
///
/// * The outlook is low at every lead in the window, and that is the unsafe direction — fails when the sign is read as incidental. Measured over 1997-2025 the bias is negative at all 26 verifiable leads, from -0.593 sfu at lead 1 to -3.968 sfu at lead 26, and it never crosses zero. So this is not scatter about a correct central value: the published outlook systematically under-forecasts F10.7, and a design reading it gets a thinner atmosphere, a lower drag and a longer predicted lifetime than it will fly. The remedy is to add the bias back, not to trust the outlook and widen a margin somewhere else
/// * One mean over 29 years of a varying Sun — fails when the bias is cycle-dependent, which it will be, because a forecaster's error on a 250 sfu day is not the error on a 70 sfu day. The sample is 866 to 1257 issue-target pairs per lead, pooled across cycles 23, 24 and the rise of 25 without conditioning on activity. A design at a known cycle phase is owed a phase-conditioned bias and this row does not give one; it gives the average over the record, which is the honest thing to publish from a pooled sample and is not the same thing
/// * The magnitude is small against the quantity and large against the margin — fails when it is compared to F10.7 itself. Minus 4 sfu on a mean of 115 sfu is 3.5%, which sounds negligible, and it is not: sw_uncertainty_growth already carries the spread a design must survive, and this bias sits underneath it as an offset that no amount of spread removes. A symmetric band around a biased centre is still centred in the wrong place
/// * Non-monotone in lead, and the middle of the curve is not noise — fails when somebody fits a straight line through it. The bias deepens to -2.748 sfu at lead 9, recovers to -1.848 at lead 13, then deepens again to -3.968 by lead 26. That interior minimum is present in a sample of over 1200 pairs per lead, so it is a property of how the outlook is constructed and not sampling scatter. The table interpolates between the measured leads rather than fitting a trend, because there is no trend to fit
pub const NODE_ID: &str = "sw_forecast_bias";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xfdb3c6344d7e038d;

pub fn evaluate(lead: Time) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : read the measured mean signed F10.7 error of the issued outlook at this lead -> Ratio
    // The measured table, lead 1 to 26 in days. Not a fit: the curve has an
    // interior extremum and a straight line through it would be a different
    // claim than the record makes. Table1 interpolates between the measured
    // leads and clamps at both ends, so a lead inside the declared domain
    // always lands between two measurements.
    use vleo_core::math::Table1;
    const LEAD_DAYS: [f64; 26] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0];
    const BIAS_SFU: [f64; 26] = [
        -0.5930701048, -0.7977346278, -1.2774613507, -1.7390243902,
        -1.9780309194, -2.4073190135, -2.6042173560, -2.5690072639,
        -2.7477840451, -2.4782258065, -2.1673403395, -1.9443099274,
        -1.8480194018, -1.9268292683, -2.3772522523, -2.3595890411,
        -2.2710706150, -2.2514285714, -2.0547320410, -2.1760000000,
        -2.2803203661, -2.5414746544, -2.9447640967, -3.3091118800,
        -3.7575057737, -3.9677419355,
    ];
    let table: Table1 = Table1 { x: &LEAD_DAYS, y: &BIAS_SFU };
    let out: Ratio = Ratio::new(table.at(lead.days()));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = out;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "B_f107", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < -4.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "B_f107", value: answer.get(), bound: -4.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the deepest measured bias is -3.968 sfu, at lead 26, the last verifiable lead. A value below -4 means the table was misread or the bundle changed underneath it" });
    }
    if answer.get() > -0.5 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "B_f107", value: answer.get(), bound: -0.5, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the shallowest measured bias is -0.593 sfu, at lead 1. A value above -0.5 — and certainly a positive one — would say the outlook over-forecasts F10.7 somewhere in the window, which this record does not show at any lead. The guard is deliberately on the safe side of zero rather than at zero, because an answer of -0.1 sfu would be as wrong as +0.1 and a bound at zero would pass it" });
    }
    Ok(answer)
}
