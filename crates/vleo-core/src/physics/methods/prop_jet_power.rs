//! GENERATED from the method of `prop_jet_power` by `cargo xtask docs`, translated by
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

/// The method of `prop_jet_power`, source `romano2021`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-prop/nodes/prop_jet_power/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// if mi <= 0 [kg/s] then
///   refuse "no ion flow, so jet power is undefined rather than infinite"
/// end
/// return jet_power(t, mi)
/// ```
pub fn evaluate(t: f64, mi: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(t) || pmath::is_nan(t) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(mi) || pmath::is_nan(mi) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if (mi <= 0.0) {
        return Err(MethodError::Refused(
            "no ion flow, so jet power is undefined rather than infinite",
        ));
    }
    return Ok(rt::fin(
        (vleo_core::physics::prop::jet_power(
            vleo_units::Force::new(t),
            vleo_units::MassFlow::new(mi),
        )
        .get()),
        6,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
