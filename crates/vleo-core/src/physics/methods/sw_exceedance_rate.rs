//! GENERATED from the method of `sw_exceedance_rate` by `cargo xtask docs`, translated by
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

/// The method of `sw_exceedance_rate`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_exceedance_rate/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Numbers: days at or above each G-scale Ap bound per year of the 28.197 years of record, counted from bundles/solar-weather (NOAA SWPC record, noaa_swpc).
/// # Three anchors, Ap 48, 80 and 132 (G1, G2, G3) — the only values sw_ap_design can return. A Table1 lookup:
/// # linear between anchors (the sheet declares that interpolation wrong for this quantity and unreached), held at both ends.
/// let out = interp(ap_design, [48.0, 80.0, 132.0], [4.6458636761, 1.2412612875, 0.2837168657])
///
/// # Generated guards: the declared domain of R_exc.
/// if out < 0.1 then
///   refuse "below 0.1 is under the lowest rate the table holds (0.284 at Ap 132), so the table has been read at the wrong end"
/// end
/// if out > 6 then
///   refuse "above 6 days a year is over the highest rate the table holds (4.65 at Ap 48), below anything the G scale defines as a storm"
/// end
/// return out
/// ```
pub fn evaluate(ap_design: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(ap_design) || pmath::is_nan(ap_design) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    let out: f64 = rt::fin(
        pmath::interp(
            ap_design,
            &[48.0, 80.0, 132.0],
            &[4.6458636761, 1.2412612875, 0.2837168657],
        ),
        5,
    )?;
    if (out < 0.1) {
        return Err(MethodError::Refused("below 0.1 is under the lowest rate the table holds (0.284 at Ap 132), so the table has been read at the wrong end"));
    }
    if (out > 6.0) {
        return Err(MethodError::Refused("above 6 days a year is over the highest rate the table holds (4.65 at Ap 48), below anything the G scale defines as a storm"));
    }
    return Ok(rt::fin(out, 14)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
