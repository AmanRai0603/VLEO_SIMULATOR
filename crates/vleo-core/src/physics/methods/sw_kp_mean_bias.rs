//! GENERATED from the method of `sw_kp_mean_bias` by `cargo xtask docs`, translated by
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

/// The method of `sw_kp_mean_bias`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_kp_mean_bias/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// #
/// # What the code said beside it:
/// # The nine bin medians, measured on solar-weather@2026.09.14. The x values are
/// # prf_ap2kp's own bin CENTRES — the midpoints of its edges 0 5 10 15 20 30 45
/// # 70 110 400 — and the y values are the medians in each bin. They belong to
/// # the record, not to this code, and the sheet names the bundle version.
/// #
/// # THE TABLE LIVES IN vleo-core AS `env::kp_mean_slot_bias` RATHER THAN HERE, and
/// # this hole calls it. Two callers read it now: this row, at one Ap, and
/// # sw_kp_scenarios, at the five the driver set carries. A table copied into
/// # both would drift from itself without anything noticing — which is not
/// # hypothetical, it is what happened to the ap-to-Kp scale.
/// #
/// # It holds the end values instead of extrapolating, which is the same choice
/// # prf_ap2kp makes explicitly ("hold the end bins, never extrapolate").
/// return kp_mean_slot_bias(ap)
/// ```
pub fn evaluate(ap: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(ap) || pmath::is_nan(ap) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::env::kp_mean_slot_bias(ap)),
        18,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
