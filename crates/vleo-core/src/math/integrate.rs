//! Integration.

/// Fixed-step classical Runge-Kutta over a scalar state.
///
/// Fixed step rather than adaptive on purpose: the atmosphere integration this
/// serves is over a known, bounded altitude range with a smooth integrand, and
/// a fixed step makes the result a deterministic function of its arguments.
/// An adaptive step would make the answer depend on a tolerance, which is one
/// more thing that can differ between two runs that ought to agree.
pub fn rk4_scalar<F>(y0: f64, x0: f64, x1: f64, steps: u32, f: F) -> f64
where
    F: Fn(f64, f64) -> f64,
{
    if steps == 0 {
        return y0;
    }
    let h = (x1 - x0) / steps as f64;
    let mut y = y0;
    let mut x = x0;
    for _ in 0..steps {
        let k1 = f(x, y);
        let k2 = f(x + 0.5 * h, y + 0.5 * h * k1);
        let k3 = f(x + 0.5 * h, y + 0.5 * h * k2);
        let k4 = f(x + h, y + h * k3);
        y += h * (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0;
        x += h;
    }
    y
}

/// Composite Simpson's rule over `[a, b]`. `panels` is rounded up to even.
pub fn simpson<F>(a: f64, b: f64, panels: u32, f: F) -> f64
where
    F: Fn(f64) -> f64,
{
    let n = if panels % 2 == 0 { panels.max(2) } else { panels + 1 };
    let h = (b - a) / n as f64;
    let mut s = f(a) + f(b);
    for i in 1..n {
        let x = a + h * i as f64;
        s += if i % 2 == 1 { 4.0 * f(x) } else { 2.0 * f(x) };
    }
    s * h / 3.0
}
