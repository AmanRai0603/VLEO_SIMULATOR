//! GENERATED from the method of `sw_storm_return_level` by `cargo xtask docs`, translated by
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

/// The method of `sw_storm_return_level`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_storm_return_level/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # The fit: bundles/solar-weather@2026.09.14 (noaa_swpc), ranks 2 to 56 of the daily Ap record, return periods 0.5035 to 14.0986 years.
/// const a = 92.515531 [1]    # intercept of the log-linear tail fit
/// const b = 40.926516 [1]    # slope per ln(year)
/// # 365.25 days, the Julian year prf_design divides by.
/// let years = life / 365.25 [d]
/// # ln needs a positive argument; a zero or negative mission length lands far below the lower bound.
/// let safe = max(years, 1.0e-9)
/// let level = a + b * ln(safe)
/// if level < 20 then
///   refuse "Below Ap 20 the answer is not a storm at all, so the input or the fit reached somewhere neither was meant to go."
/// end
/// if level > 230 then
///   refuse "A return period beyond the 28.2-year record is past the fit's own reach."
/// end
/// return level
/// ```
pub fn evaluate(life: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(life) || pmath::is_nan(life) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    let a: f64 = rt::fin(92.515531, 3)?;
    let b: f64 = rt::fin(40.926516, 4)?;
    let years: f64 = rt::fin(rt::div(life, 31557600.0, 6)?, 6)?;
    let safe: f64 = rt::fin(pmath::max(years, 1e-9), 8)?;
    let level: f64 = rt::fin((a + (b * rt::ln(safe, 9)?)), 9)?;
    if (level < 20.0) {
        return Err(MethodError::Refused("Below Ap 20 the answer is not a storm at all, so the input or the fit reached somewhere neither was meant to go."));
    }
    if (level > 230.0) {
        return Err(MethodError::Refused(
            "A return period beyond the 28.2-year record is past the fit's own reach.",
        ));
    }
    return Ok(rt::fin(level, 16)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
