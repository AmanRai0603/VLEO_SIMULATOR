// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How large is the semiannual variation in geomagnetic activity?
///
/// `Ap(doy) = 10.4936 + A*cos(4*pi*(doy-1)/365.25 - phi),  A = 1.2781`
///
/// Source: `noaa_swpc`
///
/// Geomagnetic activity is higher near the equinoxes than near the solstices
/// — the equinoctial or Russell-McPherron effect, from the changing angle
/// between the interplanetary field and the geomagnetic dipole. This row says
/// how big that seasonal swing is in Ap. It matters for a drag design because
/// Ap drives the density model, but the size of the effect is the point: it
/// moves the mean by about a tenth and predicts almost nothing about a given
/// day.
///
/// # Assumptions
///
/// * The seasonal signal is real and explains 0.63 per cent of the daily variance — fails when the amplitude is used to predict a day. Least squares over 10299 daily Ap values from 1997-01-09 to 2025-12-31 gives an amplitude of 1.278 on an offset of 10.494 — a swing of about 12 per cent about the mean — and the fit accounts for 0.006347 of the total variance. Daily Ap has a standard deviation of 11.34 against a mean of 10.50, so storm-to-storm variation dwarfs the season by more than an order of magnitude. The signal is a shift in the MEAN, visible only in aggregate: it belongs in a monthly or annual budget and is worthless as a daily correction
/// * The maxima land on the equinoxes, which is how the fit is known to be the physical effect and not a fitting artefact — fails when the phase is ignored. The fitted maxima are at day of year 96.8 and 279.4 — 6 April and 6 October — within a week of both equinoxes, and the fit was given no knowledge of them. The monthly means agree independently: September 11.88 and October 11.84 at the top, December 8.33 and January 8.61 at the bottom, a peak-to-trough ratio of 1.43. Nothing was tuned to make that happen, so the 1.278 is measuring the equinoctial effect rather than an arbitrary harmonic
/// * It is an additive amplitude in Ap, not a multiplicative one, and the distinction matters at high activity — fails when it is applied at an activity level far from the record's mean. The fit adds and subtracts 1.278 Ap regardless of the underlying level, so at the record's mean of 10.5 it is a 12 per cent modulation and at a storm level of 100 it would be 1.3 per cent. The physical mechanism is a modulation of coupling efficiency and so is closer to multiplicative, which means this additive form understates the seasonal effect during active periods and overstates it during quiet ones. It is carried additively because that is the form the record was fitted in, and a multiplicative version would need refitting on the log
/// * One harmonic, so the annual and the semiannual are not separated — fails when an annual asymmetry is present, and one is: the two fitted maxima should be equal by construction, and the monthly means are not — the autumn peak near September and October reaches 11.88 while the spring peak near May reaches 11.39. A single semiannual term cannot represent that difference and folds it into the residual. Separating an annual from a semiannual term would be a better fit and a different row
pub const NODE_ID: &str = "sw_semiannual_amplitude";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x620cf9684716a40f;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(1.278112, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "A_sa", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "A_sa", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.5 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "A_sa", value: answer.get(), bound: 0.5, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 0.5 Ap the seasonal swing would be under 5 per cent of the mean and indistinguishable from the scatter between the record's individual months, whose means range 8.33 to 11.88. A value there means the fit lost the signal" });
    }
    if answer.get() > 3.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "A_sa", value: answer.get(), bound: 3.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 3.0 Ap the modelled peak-to-trough swing would exceed 6 Ap, more than half the record's mean of 10.5 and larger than the observed spread of the monthly means, which is 3.55 from December to September. An answer that large means an annual or cycle term leaked into the semiannual one" });
    }
    Ok(answer)
}
