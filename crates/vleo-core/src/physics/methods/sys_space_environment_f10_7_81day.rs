//! GENERATED from the method of `sys_space_environment_f10_7_81day` by `cargo xtask docs`, translated by
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

/// The method of `sys_space_environment_f10_7_81day`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-system/nodes/sys_space_environment_f10_7_81day/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// #
/// # What the code said beside it:
/// # A layer-2 row receives; it does not compute. The identity is the point, and
/// # the fixtures beside it pin that the number arriving is the number leaving.
/// #
/// # The member received is f107bar_hotday, which is the hot MEAN and not the
/// # 81-day mean of the hot day — a *day scenario rides on the sustained level
/// # beneath it. The sheet argues about why that name is easy to misread.
/// return crossing
/// ```
pub fn evaluate(crossing: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(crossing) || pmath::is_nan(crossing) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(crossing, 11)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
