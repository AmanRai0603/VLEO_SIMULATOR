//! GENERATED from the method of `orbit_makeup_delta_v` by `cargo xtask docs`, translated by
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

/// The method of `orbit_makeup_delta_v`, source `vallado2013`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-envorbit/nodes/orbit_makeup_delta_v/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// # Constants the code held, written out here as it held them: 31557600 [s].
/// return drag_makeup_delta_v(a_drag, 31557600 [s])
/// ```
pub fn evaluate(a_drag: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(a_drag) || pmath::is_nan(a_drag) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::orbit::drag_makeup_delta_v(
            vleo_units::Acceleration::new(a_drag),
            vleo_units::Time::new(31557600.0),
        )
        .get()),
        4,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
