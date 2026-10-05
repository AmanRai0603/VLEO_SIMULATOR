//! GENERATED from the method of `sw_activity_band` by `cargo xtask docs`, translated by
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

/// The method of `sw_activity_band`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_activity_band/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Numbers: the standard NOAA SWPC (noaa_swpc) F10.7 activity levels, as prf_segment applies them — edges 90, 130, 170 sfu.
/// # One plus the count of edges reached; >= so a flux exactly on an edge opens the higher band.
/// # The runtime's own finiteness check stands in for the generated "not a number" guard.
/// let n = 1
/// if f107 >= 90 then
///   set n = n + 1
/// end
/// if f107 >= 130 then
///   set n = n + 1
/// end
/// if f107 >= 170 then
///   set n = n + 1
/// end
/// if n < 1 then
///   refuse "there are four bands and the lowest is 1: the counting started in the wrong place"
/// end
/// if n > 4 then
///   refuse "there are four bands and the highest is 4: an edge was added without the range being updated"
/// end
/// return n
/// ```
pub fn evaluate(f107: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(f107) || pmath::is_nan(f107) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    let mut n: f64 = rt::fin(1.0, 5)?;
    if (f107 >= 90.0) {
        n = rt::fin((n + 1.0), 7)?;
    }
    if (f107 >= 130.0) {
        n = rt::fin((n + 1.0), 10)?;
    }
    if (f107 >= 170.0) {
        n = rt::fin((n + 1.0), 13)?;
    }
    if (n < 1.0) {
        return Err(MethodError::Refused(
            "there are four bands and the lowest is 1: the counting started in the wrong place",
        ));
    }
    if (n > 4.0) {
        return Err(MethodError::Refused("there are four bands and the highest is 4: an edge was added without the range being updated"));
    }
    return Ok(rt::fin(n, 21)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
