//! GENERATED from the method of `pay_signal_electrons` by `cargo xtask docs`, translated by
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

/// The method of `pay_signal_electrons`, source `larson_wertz`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-payload/nodes/pay_signal_electrons/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// # Constants the code held, written out here as it held them: 1.0e-7 [m].
/// return signal_electrons(l * 1.0e6, d, f, p, tau, qe, t, lam, 1.0e-7 [m])
/// ```
pub fn evaluate(
    l: f64,
    d: f64,
    f: f64,
    p: f64,
    tau: f64,
    qe: f64,
    t: f64,
    lam: f64,
) -> Result<f64, MethodError> {
    if !pmath::is_finite(l) || pmath::is_nan(l) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(d) || pmath::is_nan(d) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(f) || pmath::is_nan(f) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(p) || pmath::is_nan(p) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(tau) || pmath::is_nan(tau) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(qe) || pmath::is_nan(qe) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(t) || pmath::is_nan(t) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(lam) || pmath::is_nan(lam) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::payload::signal_electrons(
            (l * 1000000.0),
            vleo_units::Length::new(d),
            vleo_units::Length::new(f),
            vleo_units::Length::new(p),
            vleo_units::Ratio::new(tau),
            vleo_units::Ratio::new(qe),
            vleo_units::Time::new(t),
            vleo_units::Length::new(lam),
            vleo_units::Length::new(1e-7),
        )),
        4,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
