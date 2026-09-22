// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much of the next rotation's Ap does the pattern fail to explain?
///
/// `sigma_ap = std(pred - truth) over 361 walk-forward next-rotation forecasts = 3.5937`
///
/// Source: `noaa_swpc`
///
/// The Ap twin of sw_mean_band_spread, by the same method on the same
/// rotations. It is a separate row because it is a separate measurement of a
/// separate quantity, and because a row publishes one number.
///
/// # Assumptions
///
/// * One sigma holds across the whole cycle — fails when it does not, and the source says so about its own number. Geomagnetic activity is burstier near the declining phase than at minimum, so a window there is given a band too narrow
/// * The residual spread is a usable margin for a non-negative index — fails when Ap floors at zero and its residuals are strongly skewed — a quiet rotation cannot undershoot far but an active one can overshoot a long way. A symmetric sigma understates the high tail, which is the tail a design is sized against. MEASURED the same way, and the same correction applies: at 1.28 sigma the Ap band holds 0.9148 to 0.9239 of the exceedances against a normal's 0.8997, so it OVER-covers by 1.5 to 2.4 points rather than understating. Ap's skew is stronger than F10.7's, -0.84 to -1.32 against -0.61 to -1.01, and its far tail is milder: the 99th percentile of exceedance runs 2.53 to 2.87 sigma against F10.7's 2.93 to 3.49. §45
/// * The 273-day hole in 2017 is interpolated before the rotations are cut — fails when those days are counted as observations. Interpolated Ap is far smoother than real Ap, so the rotations covering them are easier to predict than any real rotation and the pooled spread is narrower than the record supports
pub const NODE_ID: &str = "sw_ap_mean_band_spread";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x45cad798f2700db8;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(3.593688, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "sigma_ap", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "sigma_ap", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "sigma_ap", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a value below zero is not a spread, and Ap itself floors at zero — a quiet day really is Ap 0" });
    }
    if answer.get() > 6.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "sigma_ap", value: answer.get(), bound: 6.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 6 the residual would exceed the standard deviation of the rotation means themselves, so the pattern would be worse than predicting the record's own mean. This row is identical in method to sw_mean_band_spread and takes that row's criterion rather than the vaguer one it carried before: the 381 Ap rotation means of bundles/solar-weather@2026.09.14 have a standard deviation of 4.96, and 6 is that rounded up. The bound WAS 20, which is four times what the record supports for a residual of this quantity" });
    }
    Ok(answer)
}
