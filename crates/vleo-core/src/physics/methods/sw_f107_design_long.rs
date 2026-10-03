//! GENERATED from the method of `sw_f107_design_long` by `cargo xtask docs`, translated by
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

/// The method of `sw_f107_design_long`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_f107_design_long/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Numbers: NOAA SWPC (noaa_swpc) record via prf_density.m:217 (designWindow_); 1.28 is the one-sided 90th percentile of a normal, declared in the sheet.
/// # The hot edge of the sustained band: the central F10.7 plus 1.28 standard deviations of the rotation-forecast residual.
/// # The runtime's own finiteness check stands in for the generated "not a number" guard.
/// const z = 1.28 [1]   # one-sided 90th percentile of a normal
/// let sustained = central + spread * z
/// if sustained < 60 then
///   refuse "below 60 sfu has never been observed: the spread has been subtracted rather than added"
/// end
/// if sustained > 400 then
///   refuse "above 400 sfu the exospheric temperature relation is extrapolated past the largest recorded daily value"
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
    let sustained: f64 = rt::fin((central + (spread * z)), 6)?;
    if (sustained < 60.0) {
        return Err(MethodError::Refused("below 60 sfu has never been observed: the spread has been subtracted rather than added"));
    }
    if (sustained > 400.0) {
        return Err(MethodError::Refused("above 400 sfu the exospheric temperature relation is extrapolated past the largest recorded daily value"));
    }
    return Ok(rt::fin(sustained, 13)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
