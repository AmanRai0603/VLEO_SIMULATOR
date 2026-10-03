//! GENERATED from the method of `sw_exceedance_duration` by `cargo xtask docs`, translated by
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

/// The method of `sw_exceedance_duration`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_exceedance_duration/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Numbers: the mean length of a consecutive run at or above Ap 48, 80 and 132, counted outside this crate from bundles/solar-weather.
/// # The table clamps at both ends, so the only input the code refuses is one that is not a number:
/// # it makes the answer not a number, which the generated guard refuses.
/// if not (ap_design == ap_design) then
///   refuse "the computation produced a value that is not a number"
/// end
/// let out = interp(ap_design, [48.0, 80.0, 132.0] [1], [1.2718446601941749, 1.206896551724138, 1.1428571428571428] [d])
/// if out < 1 [d] then
///   refuse "a run is at least one day by construction, so the run-finding is broken"
/// end
/// if out > 5 [d] then
///   refuse "a mean run above 5 days would exceed every single event the record contains"
/// end
/// return out
/// ```
pub fn evaluate(ap_design: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(ap_design) || pmath::is_nan(ap_design) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if (!(ap_design == ap_design)) {
        return Err(MethodError::Refused(
            "the computation produced a value that is not a number",
        ));
    }
    let out: f64 = rt::fin(
        pmath::interp(
            ap_design,
            &[48.0, 80.0, 132.0],
            &[109887.3786407767, 104275.86206896552, 98742.85714285713],
        ),
        8,
    )?;
    if (out < 86400.0) {
        return Err(MethodError::Refused(
            "a run is at least one day by construction, so the run-finding is broken",
        ));
    }
    if (out > 432000.0) {
        return Err(MethodError::Refused(
            "a mean run above 5 days would exceed every single event the record contains",
        ));
    }
    return Ok(rt::fin(out, 15)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
