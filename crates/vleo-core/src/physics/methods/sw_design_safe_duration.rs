//! GENERATED from the method of `sw_design_safe_duration` by `cargo xtask docs`, translated by
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

/// The method of `sw_design_safe_duration`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_design_safe_duration/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # sw_storm_return_level's own fit read backwards (bundles/solar-weather@2026.09.14, noaa_swpc): Ap(T) = A + B ln(T) inverts to T = exp((Ap - A) / B).
/// const a = 92.515531 [1]    # intercept of the storm return fit
/// const b = 40.926516 [1]    # slope per ln(year)
/// let years = exp((ap_design - a) / b)
/// # The answer is declared in years; a year is 365.25 days (Unit::Year).
/// let out : Time = years * 365.25 [d]
/// if out < 0.3 * 365.25 [d] then
///   refuse "Below 0.3 years means a design level under Ap 46, beneath anything the G scale calls a storm."
/// end
/// if out > 3 * 365.25 [d] then
///   refuse "Above 3 years is past G3's Ap 132, the top of the declared G range."
/// end
/// return out
/// ```
pub fn evaluate(ap_design: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(ap_design) || pmath::is_nan(ap_design) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    let a: f64 = rt::fin(92.515531, 3)?;
    let b: f64 = rt::fin(40.926516, 4)?;
    let years: f64 = rt::fin(pmath::exp(rt::div((ap_design - a), b, 5)?), 5)?;
    let out: f64 = rt::fin((years * 31557600.0), 7)?;
    if (out < (0.3 * 31557600.0)) {
        return Err(MethodError::Refused("Below 0.3 years means a design level under Ap 46, beneath anything the G scale calls a storm."));
    }
    if (out > (3.0 * 31557600.0)) {
        return Err(MethodError::Refused(
            "Above 3 years is past G3's Ap 132, the top of the declared G range.",
        ));
    }
    return Ok(rt::fin(out, 14)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
