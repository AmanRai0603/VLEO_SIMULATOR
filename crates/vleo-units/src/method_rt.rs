//! What a node's method needs at run time, written once.
//!
//! A method is run two ways: by the interpreter in `vleo-sheet`, on the
//! author's cases, and as the Rust it is translated into, which is what ships.
//! The two must agree to the last bit on every input, so the operations whose
//! meaning is more than one line of arithmetic — a division that can meet
//! zero, a square root that can meet a negative, a power whose exponent may be
//! whole — are these functions, called by both. A second copy in either place
//! would be two definitions of what a method means.
//!
//! Every value is SI and `f64`; the dimensions were checked before any of this
//! runs.

use crate::pmath;

/// Why a method gave no answer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MethodError {
    /// The method's own `refuse`, with its sentence.
    Refused(&'static str),
    /// The mathematics is undefined here: a square root of a negative number,
    /// a division by zero, a value that is not finite. `line` is the method's.
    Degenerate { line: u32, what: &'static str },
}

impl MethodError {
    pub fn text(&self) -> &'static str {
        match self {
            MethodError::Refused(r) => r,
            MethodError::Degenerate { what, .. } => what,
        }
    }
}

fn bad(line: u32, what: &'static str) -> MethodError {
    MethodError::Degenerate { line, what }
}

/// A named value, checked: a method never carries a value that is not a
/// finite number from one line to the next.
pub fn fin(v: f64, line: u32) -> Result<f64, MethodError> {
    if pmath::is_finite(v) && !pmath::is_nan(v) {
        Ok(v)
    } else {
        Err(bad(line, "the value here is not a finite number"))
    }
}

pub fn div(a: f64, b: f64, line: u32) -> Result<f64, MethodError> {
    if b == 0.0 {
        Err(bad(line, "division by zero"))
    } else {
        Ok(a / b)
    }
}

pub fn sqrt(x: f64, line: u32) -> Result<f64, MethodError> {
    if x < 0.0 {
        Err(bad(line, "a negative number has no square root"))
    } else {
        Ok(pmath::sqrt(x))
    }
}

pub fn ln(x: f64, line: u32) -> Result<f64, MethodError> {
    if x > 0.0 {
        Ok(pmath::ln(x))
    } else {
        Err(bad(line, "the logarithm needs a number above zero"))
    }
}

pub fn log10(x: f64, line: u32) -> Result<f64, MethodError> {
    if x > 0.0 {
        Ok(pmath::log10(x))
    } else {
        Err(bad(line, "the logarithm needs a number above zero"))
    }
}

pub fn log2(x: f64, line: u32) -> Result<f64, MethodError> {
    if x > 0.0 {
        Ok(pmath::log2(x))
    } else {
        Err(bad(line, "the logarithm needs a number above zero"))
    }
}

pub fn asin(x: f64, line: u32) -> Result<f64, MethodError> {
    if (-1.0..=1.0).contains(&x) {
        Ok(pmath::asin(x))
    } else {
        Err(bad(line, "asin of a number outside -1..1"))
    }
}

pub fn acos(x: f64, line: u32) -> Result<f64, MethodError> {
    if (-1.0..=1.0).contains(&x) {
        Ok(pmath::acos(x))
    } else {
        Err(bad(line, "acos of a number outside -1..1"))
    }
}

pub fn fmod(a: f64, b: f64, line: u32) -> Result<f64, MethodError> {
    if b == 0.0 {
        Err(bad(line, "fmod by zero"))
    } else {
        Ok(pmath::fmod(a, b))
    }
}

/// `a ^ b`: a whole power below 64 by repeated multiplication, anything else
/// through the logarithm — the same choice every time, so the interpreter and
/// the translated code take the same path.
pub fn pow(a: f64, b: f64) -> f64 {
    if pmath::trunc(b) == b && pmath::abs(b) < 64.0 {
        pmath::powi(a, b as i32)
    } else {
        pmath::powf(a, b)
    }
}
