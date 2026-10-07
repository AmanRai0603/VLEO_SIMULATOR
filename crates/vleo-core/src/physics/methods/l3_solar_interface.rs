//! GENERATED from the method of `l3_solar_interface` by `cargo xtask docs`, translated by
//! the fixed rules in crates/vleo-sheet/src/method.rs. Do not edit: the method
//! is changed on the node's form, and this is written again from it.

#![allow(
    clippy::all,
    clippy::float_cmp,
    clippy::cast_precision_loss,
    unreachable_code,
    unused_assignments,
    unused_imports,
    unused_mut,
    unused_variables,
    unused_parens,
    non_snake_case
)]

use vleo_units::constants::*;
use vleo_units::method_rt::{self as rt, MethodError};
use vleo_units::pmath;

/// The method of `l3_solar_interface`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/l3_solar_interface/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// #
/// # What the code said beside it:
/// # A crossing carries; it does not compute. The one thing that can go wrong
/// # here is that the seam alters what it is handed — a stray factor, an
/// # unasked-for unit conversion, a clamp inherited from the wrong row — and
/// # both sides would still look plausible. So every member below is one input,
/// # named, with no arithmetic anywhere in the block.
/// #
/// # Written out member by member rather than looped, because the mapping from
/// # scenario to producing row IS the content of this row and a reader has to
/// # be able to check it against the study's own table. Fifteen assignments is
/// # the price of that being checkable.
/// # The three *mean scenarios: f107 == f107bar, because they ARE the
/// # window mean. The study's own table shows the same.
/// # The two *day scenarios: f107 is the day, f107bar is the SUSTAINED
/// # level it rides on. This is the one place the row publishes an input
/// # under a second name, and it is selection rather than arithmetic.
/// # Ap has no 81-day companion in the study's driver set, so five members
/// # rather than ten. The Kp columns are the ones missing from this set and
/// # the sheet's first assumption says why.
/// # The two Kp columns, relayed from the one row that evaluates the
/// # published scale and both measured slot offsets at all five Ap. The
/// # arithmetic is there and not here, because a crossing relays: putting
/// # kp_from_ap(ap) + bias in this block would be subsystem work done where
/// # no subsystem reviewer reads it.
/// publish F107bar_hotmean = f107_hot_long
/// publish F107_nominal = f107_centre
/// publish F107bar_nominal = f107_centre
/// publish F107_coldmean = f107_cold_long
/// publish F107bar_coldmean = f107_cold_long
/// publish F107_hotday = f107_hot_day
/// publish F107bar_hotday = f107_hot_long
/// publish F107_coldday = f107_cold_day
/// publish F107bar_coldday = f107_cold_long
/// publish Ap_nominal = ap_centre
/// publish Ap_hotmean = ap_hot_long
/// publish Ap_coldmean = ap_cold_long
/// publish Ap_hotday = ap_hot_day
/// publish Ap_coldday = ap_cold_day
/// publish Kp_mean_nominal = kp_mean_nominal
/// publish Kp_mean_hotmean = kp_mean_hotmean
/// publish Kp_mean_coldmean = kp_mean_coldmean
/// publish Kp_mean_hotday = kp_mean_hotday
/// publish Kp_mean_coldday = kp_mean_coldday
/// publish Kp_peak_nominal = kp_peak_nominal
/// publish Kp_peak_hotmean = kp_peak_hotmean
/// publish Kp_peak_coldmean = kp_peak_coldmean
/// publish Kp_peak_hotday = kp_peak_hotday
/// publish Kp_peak_coldday = kp_peak_coldday
/// return f107_hot_long
/// ```
/// Returns the answer, and the published members in this order: F107_nominal, F107_coldmean, F107_hotday, F107_coldday, F107bar_nominal, F107bar_hotmean, F107bar_coldmean, F107bar_hotday, F107bar_coldday, Ap_nominal, Ap_hotmean, Ap_coldmean, Ap_hotday, Ap_coldday, Kp_mean_nominal, Kp_mean_hotmean, Kp_mean_coldmean, Kp_mean_hotday, Kp_mean_coldday, Kp_peak_nominal, Kp_peak_hotmean, Kp_peak_coldmean, Kp_peak_hotday, Kp_peak_coldday.
pub fn evaluate(
    f107_centre: f64,
    f107_hot_long: f64,
    f107_cold_long: f64,
    f107_hot_day: f64,
    f107_cold_day: f64,
    ap_centre: f64,
    ap_hot_long: f64,
    ap_cold_long: f64,
    ap_hot_day: f64,
    ap_cold_day: f64,
    kp_mean_nominal: f64,
    kp_mean_hotmean: f64,
    kp_mean_coldmean: f64,
    kp_mean_hotday: f64,
    kp_mean_coldday: f64,
    kp_peak_nominal: f64,
    kp_peak_hotmean: f64,
    kp_peak_coldmean: f64,
    kp_peak_hotday: f64,
    kp_peak_coldday: f64,
) -> Result<(f64, [f64; 24]), MethodError> {
    let mut published = [0.0_f64; 24];
    if !pmath::is_finite(f107_centre) || pmath::is_nan(f107_centre) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(f107_hot_long) || pmath::is_nan(f107_hot_long) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(f107_cold_long) || pmath::is_nan(f107_cold_long) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(f107_hot_day) || pmath::is_nan(f107_hot_day) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(f107_cold_day) || pmath::is_nan(f107_cold_day) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(ap_centre) || pmath::is_nan(ap_centre) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(ap_hot_long) || pmath::is_nan(ap_hot_long) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(ap_cold_long) || pmath::is_nan(ap_cold_long) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(ap_hot_day) || pmath::is_nan(ap_hot_day) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(ap_cold_day) || pmath::is_nan(ap_cold_day) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(kp_mean_nominal) || pmath::is_nan(kp_mean_nominal) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(kp_mean_hotmean) || pmath::is_nan(kp_mean_hotmean) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(kp_mean_coldmean) || pmath::is_nan(kp_mean_coldmean) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(kp_mean_hotday) || pmath::is_nan(kp_mean_hotday) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(kp_mean_coldday) || pmath::is_nan(kp_mean_coldday) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(kp_peak_nominal) || pmath::is_nan(kp_peak_nominal) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(kp_peak_hotmean) || pmath::is_nan(kp_peak_hotmean) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(kp_peak_coldmean) || pmath::is_nan(kp_peak_coldmean) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(kp_peak_hotday) || pmath::is_nan(kp_peak_hotday) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(kp_peak_coldday) || pmath::is_nan(kp_peak_coldday) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    published[5] = rt::fin(f107_hot_long, 28)?; // F107bar_hotmean
    published[0] = rt::fin(f107_centre, 29)?; // F107_nominal
    published[4] = rt::fin(f107_centre, 30)?; // F107bar_nominal
    published[1] = rt::fin(f107_cold_long, 31)?; // F107_coldmean
    published[6] = rt::fin(f107_cold_long, 32)?; // F107bar_coldmean
    published[2] = rt::fin(f107_hot_day, 33)?; // F107_hotday
    published[7] = rt::fin(f107_hot_long, 34)?; // F107bar_hotday
    published[3] = rt::fin(f107_cold_day, 35)?; // F107_coldday
    published[8] = rt::fin(f107_cold_long, 36)?; // F107bar_coldday
    published[9] = rt::fin(ap_centre, 37)?; // Ap_nominal
    published[10] = rt::fin(ap_hot_long, 38)?; // Ap_hotmean
    published[11] = rt::fin(ap_cold_long, 39)?; // Ap_coldmean
    published[12] = rt::fin(ap_hot_day, 40)?; // Ap_hotday
    published[13] = rt::fin(ap_cold_day, 41)?; // Ap_coldday
    published[14] = rt::fin(kp_mean_nominal, 42)?; // Kp_mean_nominal
    published[15] = rt::fin(kp_mean_hotmean, 43)?; // Kp_mean_hotmean
    published[16] = rt::fin(kp_mean_coldmean, 44)?; // Kp_mean_coldmean
    published[17] = rt::fin(kp_mean_hotday, 45)?; // Kp_mean_hotday
    published[18] = rt::fin(kp_mean_coldday, 46)?; // Kp_mean_coldday
    published[19] = rt::fin(kp_peak_nominal, 47)?; // Kp_peak_nominal
    published[20] = rt::fin(kp_peak_hotmean, 48)?; // Kp_peak_hotmean
    published[21] = rt::fin(kp_peak_coldmean, 49)?; // Kp_peak_coldmean
    published[22] = rt::fin(kp_peak_hotday, 50)?; // Kp_peak_hotday
    published[23] = rt::fin(kp_peak_coldday, 51)?; // Kp_peak_coldday
    return Ok((rt::fin(f107_hot_long, 52)?, published));
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
