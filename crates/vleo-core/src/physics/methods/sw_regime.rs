//! GENERATED from the method of `sw_regime` by `cargo xtask docs`, translated by
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

/// The method of `sw_regime`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_regime/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # The mixture's own decision boundaries read off its published labels in bundles/solar-weather daily_regime.csv (noaa_swpc): quiet Ap 0 to 6, active 7 to 25, storm 26 and above.
/// let out = 3
/// if ap <= 6 then
///   set out = 1
/// else if ap <= 25 then
///   set out = 2
/// end
/// if out < 1 then
///   refuse "1 is quiet, the lowest regime the mixture defines; below it there is no label."
/// end
/// if out > 3 then
///   refuse "3 is storm, the highest; a fourth regime would be a different classifier."
/// end
/// return out
/// ```
pub fn evaluate(ap: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(ap) || pmath::is_nan(ap) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    let mut out: f64 = rt::fin(3.0, 3)?;
    if (ap <= 6.0) {
        out = rt::fin(1.0, 5)?;
    } else if (ap <= 25.0) {
        out = rt::fin(2.0, 7)?;
    }
    if (out < 1.0) {
        return Err(MethodError::Refused(
            "1 is quiet, the lowest regime the mixture defines; below it there is no label.",
        ));
    }
    if (out > 3.0) {
        return Err(MethodError::Refused(
            "3 is storm, the highest; a fourth regime would be a different classifier.",
        ));
    }
    return Ok(rt::fin(out, 15)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
