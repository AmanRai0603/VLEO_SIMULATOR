//! GENERATED from the method of `sw_exceedance_phase` by `cargo xtask docs`, translated by
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

/// The method of `sw_exceedance_phase`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_exceedance_phase/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # The median cycle phase of the days at or above each design Ap (48, 80, 132), counted outside this crate from bundles/solar-weather (noaa_swpc).
/// # The generated finite check on the answer: the table is finite and held at its ends, so the only
/// # non-finite answer comes from an Ap that is not a number, and it is refused here, before the lookup.
/// if not (ap_design == ap_design) then
///   refuse "The computation produced a value that is not a number."
/// end
/// # Three anchors; linear between them and held at both ends (Table1::at), as the Rust does.
/// let out = interp(ap_design, [48.0, 80.0, 132.0] [1], [0.5716920240, 0.5763024435, 0.6026970954] [1])
/// if out < 0 then
///   refuse "Cycle phase runs from 0 at minimum, so below 0 is not a phase."
/// end
/// if out > 1 then
///   refuse "Cycle phase ends at 1 at the next minimum, so above 1 is not a phase."
/// end
/// return out
/// ```
pub fn evaluate(ap_design: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(ap_design) || pmath::is_nan(ap_design) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if (!(ap_design == ap_design)) {
        return Err(MethodError::Refused(
            "The computation produced a value that is not a number.",
        ));
    }
    let out: f64 = rt::fin(
        pmath::interp(
            ap_design,
            &[48.0, 80.0, 132.0],
            &[0.571692024, 0.5763024435, 0.6026970954],
        ),
        9,
    )?;
    if (out < 0.0) {
        return Err(MethodError::Refused(
            "Cycle phase runs from 0 at minimum, so below 0 is not a phase.",
        ));
    }
    if (out > 1.0) {
        return Err(MethodError::Refused(
            "Cycle phase ends at 1 at the next minimum, so above 1 is not a phase.",
        ));
    }
    return Ok(rt::fin(out, 16)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
