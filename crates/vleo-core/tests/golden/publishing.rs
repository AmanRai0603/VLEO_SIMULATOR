//! GENERATED from the method of `publishing` by `cargo xtask docs`, translated by
//! the fixed rules in crates/vleo-sheet/src/method.rs. Do not edit: the method
//! is changed on the node's form, and this is written again from it.

#![allow(
    clippy::all,
    clippy::float_cmp,
    clippy::cast_precision_loss,
    unreachable_code,
    unused_imports,
    unused_mut,
    unused_variables,
    unused_parens,
    non_snake_case
)]

use vleo_units::constants::*;
use vleo_units::method_rt::{self as rt, MethodError};
use vleo_units::pmath;

/// The method of `publishing`, source `none — a translator test`:
///
/// ```text
/// if x < 0 then
///   refuse "below zero"
/// end
/// let s = x * x
/// publish Root = sqrt(x)
/// publish Square = s
/// return s + 1
/// ```
/// Returns the answer, and the published members in this order: Square, Root.
pub fn evaluate(x: f64) -> Result<(f64, [f64; 2]), MethodError> {
    let mut published = [0.0_f64; 2];
    if (x < 0.0) {
        return Err(MethodError::Refused("below zero"));
    }
    let s: f64 = rt::fin((x * x), 4)?;
    published[1] = rt::fin(rt::sqrt(x, 5)?, 5)?; // Root
    published[0] = rt::fin(s, 6)?; // Square
    return Ok((rt::fin((s + 1.0), 7)?, published));
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
