//! GENERATED from the method of `sw_mean_cycle_level` by `cargo xtask docs`, translated by
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

/// The method of `sw_mean_cycle_level`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_mean_cycle_level/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # The mean F10.7 of cycles 23 and 24 stacked on phase, twenty bins (x is the bin centre), counted on bundles/solar-weather@2026.09.14 (noaa_swpc).
/// # The generated finite check on the answer: the table is finite and held at its ends, so the only
/// # non-finite answer comes from a phase that is not a number, and it is refused here, before the lookup.
/// if not (phase == phase) then
///   refuse "The computation produced a value that is not a number."
/// end
/// # Linear between bin centres, held at the ends (Table1::at).
/// let out = interp(phase, [0.025, 0.075, 0.125, 0.175, 0.225, 0.275, 0.325, 0.375, 0.425, 0.475, 0.525, 0.575, 0.625, 0.675, 0.725, 0.775, 0.825, 0.875, 0.925, 0.975] [1], [71.7632, 83.8947, 99.2177, 111.0502, 135.6555, 159.6603, 145.3702, 146.7273, 165.2799, 160.3301, 135.3182, 126.2321, 105.8445, 95.5084, 86.9916, 80.9171, 76.2094, 71.0478, 71.5383, 67.6538] [1])
/// if out < 60 then
///   refuse "Below 60 sfu has never been observed and no relation reading F10.7 has support there."
/// end
/// if out > 400 then
///   refuse "Above 400 sfu every consumer of F10.7 is extrapolating; unreachable by this table, it catches a broken one."
/// end
/// return out
/// ```
pub fn evaluate(phase: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(phase) || pmath::is_nan(phase) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if (!(phase == phase)) {
        return Err(MethodError::Refused(
            "The computation produced a value that is not a number.",
        ));
    }
    let out: f64 = rt::fin(
        pmath::interp(
            phase,
            &[
                0.025, 0.075, 0.125, 0.175, 0.225, 0.275, 0.325, 0.375, 0.425, 0.475, 0.525, 0.575,
                0.625, 0.675, 0.725, 0.775, 0.825, 0.875, 0.925, 0.975,
            ],
            &[
                71.7632, 83.8947, 99.2177, 111.0502, 135.6555, 159.6603, 145.3702, 146.7273,
                165.2799, 160.3301, 135.3182, 126.2321, 105.8445, 95.5084, 86.9916, 80.9171,
                76.2094, 71.0478, 71.5383, 67.6538,
            ],
        ),
        9,
    )?;
    if (out < 60.0) {
        return Err(MethodError::Refused(
            "Below 60 sfu has never been observed and no relation reading F10.7 has support there.",
        ));
    }
    if (out > 400.0) {
        return Err(MethodError::Refused("Above 400 sfu every consumer of F10.7 is extrapolating; unreachable by this table, it catches a broken one."));
    }
    return Ok(rt::fin(out, 16)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
