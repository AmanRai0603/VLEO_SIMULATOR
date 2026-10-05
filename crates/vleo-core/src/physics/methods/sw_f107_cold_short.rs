//! GENERATED from the method of `sw_f107_cold_short` by `cargo xtask docs`, translated by
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

/// The method of `sw_f107_cold_short`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_f107_cold_short/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Both terms come from the daily F10.7 record (noaa_swpc) through sw_f107_cold_long and sw_daily_band_drop; no constant of its own.
/// # The daily term is a magnitude, so the sign lives here in the relation.
/// let single_day = sustained - daily
/// if single_day < 60 then
///   refuse "Below 60 sfu has never been observed and every relation reading F10.7 has no support there."
/// end
/// if single_day > 400 then
///   refuse "Above 400 sfu on the coldest scenario means a sign is wrong somewhere in the chain above it."
/// end
/// return single_day
/// ```
pub fn evaluate(sustained: f64, daily: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(sustained) || pmath::is_nan(sustained) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(daily) || pmath::is_nan(daily) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    let single_day: f64 = rt::fin((sustained - daily), 4)?;
    if (single_day < 60.0) {
        return Err(MethodError::Refused("Below 60 sfu has never been observed and every relation reading F10.7 has no support there."));
    }
    if (single_day > 400.0) {
        return Err(MethodError::Refused("Above 400 sfu on the coldest scenario means a sign is wrong somewhere in the chain above it."));
    }
    return Ok(rt::fin(single_day, 11)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
