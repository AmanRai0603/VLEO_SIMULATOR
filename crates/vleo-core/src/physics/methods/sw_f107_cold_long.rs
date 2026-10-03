//! GENERATED from the method of `sw_f107_cold_long` by `cargo xtask docs`, translated by
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

/// The method of `sw_f107_cold_long`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_f107_cold_long/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Numbers: 1.28 spreads, declared in the sheet (source noaa_swpc) and not chosen in the hole — the same 1.28 the hot edge adds.
/// # The generated guard refuses an answer that is not a finite number; x - x is zero only for a finite x.
/// if (central - spread * 1.28) - (central - spread * 1.28) != 0 then
///   refuse "the computation produced a value that is not a number"
/// end
/// let sustained = central - spread * 1.28
/// if sustained < 60 then
///   refuse "below 60 sfu has never been observed: the band is wider than the sky"
/// end
/// if sustained > 400 then
///   refuse "a cold level above 400 sfu is arithmetic, not sky: the spread was added rather than subtracted"
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
    if (((central - (spread * 1.28)) - (central - (spread * 1.28))) != 0.0) {
        return Err(MethodError::Refused(
            "the computation produced a value that is not a number",
        ));
    }
    let sustained: f64 = rt::fin((central - (spread * 1.28)), 7)?;
    if (sustained < 60.0) {
        return Err(MethodError::Refused(
            "below 60 sfu has never been observed: the band is wider than the sky",
        ));
    }
    if (sustained > 400.0) {
        return Err(MethodError::Refused("a cold level above 400 sfu is arithmetic, not sky: the spread was added rather than subtracted"));
    }
    return Ok(rt::fin(sustained, 14)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
