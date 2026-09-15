// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How strong is the solar-rotation recurrence in F10.7?
///
/// `r_rot = corr(F107', F107' shifted by L_rot) = 0.375988`
///
/// Source: `noaa_swpc`
///
/// The correlation at the recurrence lag sw_recurrence_lag reports. It says
/// how much of a future day a design can actually infer from an active region
/// coming back round, and the answer is a useful tendency rather than a
/// prediction: 0.376 is 14 per cent of the variance.
///
/// # Assumptions
///
/// * A correlation of 0.376 explains 14 per cent of the variance, and that is the number a design should hear — fails when the correlation is read as the fraction of the signal that recurs. Squared it is 0.1414, so a rotation ahead the recurrence accounts for one seventh of the detrended variation and the other six sevenths is new. It is enough to make a 27-day outlook better than nothing — sw_forecast_skill measures +0.018 still at lead 23 — and nowhere near enough to size anything on
/// * It is the strength at the FIRST peak and the signal keeps going — fails when the recurrence is assumed to die within one rotation. The second harmonic at lag 54 still carries +0.136 and the third at lag 81 carries +0.067, so an active region is faintly detectable three rotations out. This row reports only the first peak, which is what the MATLAB field C.rot_peak_r holds. The decay across harmonics — 0.376, 0.136, 0.067 — is roughly geometric and is the physical lifetime of an active region showing up in the statistics
/// * Pooled over cycles 23, 24 and the rise of 25, and the recurrence is certainly not constant across them — fails when a design at a known cycle phase wants the recurrence it will actually see. At solar maximum there are many active regions and their overlap blurs the rotation signal; near minimum a single long-lived region can dominate it. This is one correlation over 10242 detrended days spanning all three, so it averages regimes in which the mechanism differs. A phase-conditioned version would be a separate row and would need the epoch, which now exists
pub const NODE_ID: &str = "sw_recurrence_strength";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x61ce3646326011e0;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(0.375988, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "r_rot", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "r_rot", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.1 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "r_rot", value: answer.get(), bound: 0.1, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 0.1 the rotation bump would be indistinguishable from the noise floor of the detrended series, whose correlation sits between -0.09 and +0.03 across lags 36 to 47. A value there means the detrending removed the signal" });
    }
    if answer.get() > 0.7 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "r_rot", value: answer.get(), bound: 0.7, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 0.7 a rotation-ahead correlation would be stronger than the measured one-day correlation of the same series at lag 5 (+0.531), which would mean F10.7 is more predictable 26 days out than 5 days out. The measured value is 0.376 and nothing in the record approaches 0.7" });
    }
    Ok(answer)
}
