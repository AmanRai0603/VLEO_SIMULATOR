//! Root finding.
//!
//! Every routine here reports non-convergence rather than returning its last
//! guess. A sizing loop that quietly hands back an unconverged number is the
//! same failure as a stale cache: a plausible value that nothing marks.

use vleo_units::pmath;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RootError {
    /// `f(a)` and `f(b)` have the same sign, so no root is bracketed.
    NotBracketed { fa: f64, fb: f64 },
    /// The iteration limit was reached. Carries the best estimate and the
    /// residual, so the caller can report both.
    NotConverged {
        best: f64,
        residual: f64,
        iterations: u32,
    },
}

/// Bisection. Slow and unconditionally convergent once a root is bracketed,
/// which is the right trade for a sizing loop that runs a few hundred times and
/// must never fail to terminate.
pub fn bisect<F>(mut a: f64, mut b: f64, tol: f64, max_iter: u32, f: F) -> Result<f64, RootError>
where
    F: Fn(f64) -> f64,
{
    let (mut fa, fb) = (f(a), f(b));
    if fa * fb > 0.0 {
        return Err(RootError::NotBracketed { fa, fb });
    }
    for i in 0..max_iter {
        let m = 0.5 * (a + b);
        let fm = f(m);
        if pmath::abs(b - a) <= tol || fm == 0.0 {
            return Ok(m);
        }
        if fa * fm <= 0.0 {
            b = m;
        } else {
            a = m;
            fa = fm;
        }
        let _ = i;
    }
    Err(RootError::NotConverged {
        best: 0.5 * (a + b),
        residual: pmath::abs(b - a),
        iterations: max_iter,
    })
}

/// Secant iteration, for a smooth function with a good starting pair.
pub fn secant<F>(x0: f64, x1: f64, tol: f64, max_iter: u32, f: F) -> Result<f64, RootError>
where
    F: Fn(f64) -> f64,
{
    let (mut a, mut b) = (x0, x1);
    let (mut fa, mut fb) = (f(a), f(b));
    for i in 0..max_iter {
        // Exact equality is the right test here and not a tolerance: the next
        // line divides by this difference, and the only value that must be
        // caught is the one that would divide by zero.
        #[allow(clippy::float_cmp)]
        if fb == fa {
            return Err(RootError::NotConverged {
                best: b,
                residual: pmath::abs(fb),
                iterations: i,
            });
        }
        let c = b - fb * (b - a) / (fb - fa);
        if pmath::abs(c - b) <= tol {
            return Ok(c);
        }
        a = b;
        fa = fb;
        b = c;
        fb = f(c);
    }
    Err(RootError::NotConverged {
        best: b,
        residual: pmath::abs(fb),
        iterations: max_iter,
    })
}
