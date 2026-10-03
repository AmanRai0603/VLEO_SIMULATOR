//! GENERATED from the method of `sw_ap_cold_long` by `cargo xtask docs`, translated by
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

/// The method of `sw_ap_cold_long`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_ap_cold_long/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Numbers: NOAA SWPC (noaa_swpc) record via prf_density.m:217 (designWindow_); 1.28 is the one-sided 90th percentile of a normal, declared in the sheet.
/// # The cold edge of the sustained band: the central Ap less 1.28 standard deviations of the rotation-forecast residual.
/// # The runtime's own finiteness check stands in for the generated "not a number" guard.
/// const z = 1.28 [1]   # one-sided 90th percentile of a normal, the same 1.28 the hot edge adds
/// let sustained = central - spread * z
/// if sustained < 0 then
///   refuse "Ap floors at zero: a symmetric band subtracted from a small centre has reached below it"
/// end
/// if sustained > 300 then
///   refuse "above 300 exceeds the largest daily Ap in the record, 273: the spread has been added rather than subtracted"
/// end
/// return sustained
/// ```
pub fn evaluate(central: f64, spread: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(central) || pmath::is_nan(central) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(spread) || pmath::is_nan(spread) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    let z: f64 = rt::fin(1.28, 5)?;
    let sustained: f64 = rt::fin((central - (spread * z)), 6)?;
    if (sustained < 0.0) {
        return Err(MethodError::Refused("Ap floors at zero: a symmetric band subtracted from a small centre has reached below it"));
    }
    if (sustained > 300.0) {
        return Err(MethodError::Refused("above 300 exceeds the largest daily Ap in the record, 273: the spread has been added rather than subtracted"));
    }
    return Ok(rt::fin(sustained, 13)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
