//! GENERATED from the method of `prop_ion_flow` by `cargo xtask docs`, translated by
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

/// The method of `prop_ion_flow`, source `romano2021`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-prop/nodes/prop_ion_flow/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return ion_mass_flow(md, eta_u)
/// ```
pub fn evaluate(md: f64, eta_u: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(md) || pmath::is_nan(md) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(eta_u) || pmath::is_nan(eta_u) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::prop::ion_mass_flow(
            vleo_units::MassFlow::new(md),
            vleo_units::Ratio::new(eta_u),
        )
        .get()),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
