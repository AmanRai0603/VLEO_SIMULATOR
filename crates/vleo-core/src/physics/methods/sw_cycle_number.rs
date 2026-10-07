//! GENERATED from the method of `sw_cycle_number` by `cargo xtask docs`, translated by
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

/// The method of `sw_cycle_number`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_cycle_number/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// # Constants the code held, written out here as it held them: the three cycle starts; 23, the first cycle counted.
/// #
/// # What the code said beside it:
/// # The record's three cycle starts, as days since 2000-01-01, from
/// # solar_cycles.csv: cycle 23 at 1997-01-15, 24 at 2008-12-01, 25 at 2019-12-01.
/// # The comparison is >= so a boundary day belongs to the cycle it OPENS, which is
/// # what the boundary fixtures beside this pin.
/// #
/// # An epoch past the last start still returns 25, because nothing in the data
/// # says when cycle 26 begins and this row will not invent a boundary. The
/// # declared upper bound of 26 is what keeps that from being silent.
/// # The day each solar cycle began, counted from the mission epoch's day zero.
/// const STARTS = [-1081, 3257, 7274] [1]
/// let d = epoch / 86400 [s]
/// let n = 23 - 1
/// for start in STARTS
///   if d >= start then
///     set n = n + 1
///   end
/// end
/// return n
/// ```
pub fn evaluate(epoch: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(epoch) || pmath::is_nan(epoch) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    let STARTS: [f64; 3] = [(-1081.0), 3257.0, 7274.0];
    let d: f64 = rt::fin(rt::div(epoch, 86400.0, 16)?, 16)?;
    let mut n: f64 = rt::fin((23.0 - 1.0), 17)?;
    for entry_1 in STARTS.iter() {
        let start: f64 = *entry_1;
        if (d >= start) {
            n = rt::fin((n + 1.0), 20)?;
        }
    }
    return Ok(rt::fin(n, 23)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
