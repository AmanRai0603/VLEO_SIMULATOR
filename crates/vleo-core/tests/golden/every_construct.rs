//! GENERATED from the method of `every_construct` by `cargo xtask docs`, translated by
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
    unused_parens
)]

use vleo_units::constants::*;
use vleo_units::method_rt::{self as rt, MethodError};
use vleo_units::pmath;

/// The method of `every_construct`, source `none — a translator test`:
///
/// ```text
/// # Not physics: a method that uses every construct, for the translator.
/// if x < 0 then
///   refuse "x must not be negative"
/// end
/// let total = 0
/// for n = 1 to 12
///   set total = total + x ^ n / n
/// end
/// let t : Ratio = interp(x, [0, 0.5, 1, 4] [1], [1, 2, 0.5, 3] [1])
/// let a = sqrt(x + 1) + cbrt(x) + ln(x + 2) + exp(-x) + log10(x + 1) + log2(x + 1)
/// let b = sin(x) * cos(x) + tan(x / 10) + atan(x) + atan2(x, 1) + sinh(x / 5) + cosh(x / 5) + tanh(x)
/// let c = min(x, 2) + max(x, 0.5) + abs(0 - x) + hypot(x, 1) + fmod(x + 7, 3) + erf(x) + erfc(x)
/// let d = floor(x * 3) + ceil(x) + round(x * 2) + wrap_2pi(x * 7) + wrap_pi(x * 7) + pow(x + 1, 2.5) + (x + 1) ^ (x / 4)
/// let e = asin(min(x, 1)) + acos(min(x, 1)) + PI
/// if x > 3 and not (x > 3.5) then
///   return total + t
/// else if x > 2 or x == 1.25 then
///   return a + b
/// else
///   return (c + d + e) / (x + 1)
/// end
/// ```
pub fn evaluate(x: f64) -> Result<f64, MethodError> {
    if (x < 0.0) {
        return Err(MethodError::Refused("x must not be negative"));
    }
    let mut total: f64 = rt::fin(0.0, 5)?;
    for step_1 in (1_i64)..=(12_i64) {
        let n: f64 = step_1 as f64;
        total = rt::fin((total + rt::div(rt::pow(x, n), n, 7)?), 7)?;
    }
    let t: f64 = rt::fin(
        pmath::interp(x, &[0.0, 0.5, 1.0, 4.0], &[1.0, 2.0, 0.5, 3.0]),
        9,
    )?;
    let a: f64 = rt::fin(
        (((((rt::sqrt((x + 1.0), 10)? + pmath::cbrt(x)) + rt::ln((x + 2.0), 10)?)
            + pmath::exp((-x)))
            + rt::log10((x + 1.0), 10)?)
            + rt::log2((x + 1.0), 10)?),
        10,
    )?;
    let b: f64 = rt::fin(
        (((((((pmath::sin(x) * pmath::cos(x)) + pmath::tan(rt::div(x, 10.0, 11)?))
            + pmath::atan(x))
            + pmath::atan2(x, 1.0))
            + pmath::sinh(rt::div(x, 5.0, 11)?))
            + pmath::cosh(rt::div(x, 5.0, 11)?))
            + pmath::tanh(x)),
        11,
    )?;
    let c: f64 = rt::fin(
        ((((((pmath::min(x, 2.0) + pmath::max(x, 0.5)) + pmath::abs((0.0 - x)))
            + pmath::hypot(x, 1.0))
            + rt::fmod((x + 7.0), 3.0, 12)?)
            + pmath::erf(x))
            + pmath::erfc(x)),
        12,
    )?;
    let d: f64 = rt::fin(
        ((((((pmath::floor((x * 3.0)) + pmath::ceil(x)) + pmath::round((x * 2.0)))
            + pmath::wrap_2pi((x * 7.0)))
            + pmath::wrap_pi((x * 7.0)))
            + rt::pow((x + 1.0), 2.5))
            + rt::pow((x + 1.0), rt::div(x, 4.0, 13)?)),
        13,
    )?;
    let e: f64 = rt::fin(
        ((rt::asin(pmath::min(x, 1.0), 14)? + rt::acos(pmath::min(x, 1.0), 14)?)
            + core::f64::consts::PI),
        14,
    )?;
    if ((x > 3.0) && (!(x > 3.5))) {
        return Ok(rt::fin((total + t), 16)?);
    } else if ((x > 2.0) || (x == 1.25)) {
        return Ok(rt::fin((a + b), 18)?);
    } else {
        return Ok(rt::fin(rt::div(((c + d) + e), (x + 1.0), 20)?, 20)?);
    }
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
