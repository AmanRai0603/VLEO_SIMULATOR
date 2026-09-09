//! Interpolation over a fixed table.
//!
//! Tables are `&'static` slices compiled in, never files read at run time —
//! R3: the engine makes no network calls and the kernel opens nothing. A table
//! that would be too large to compile in is reference data, which arrives as a
//! verified bundle and is passed to the kernel as a resolved handle.

use vleo_units::pmath;

/// A one-dimensional table with linear interpolation and clamped ends.
#[derive(Clone, Copy, Debug)]
pub struct Table1 {
    pub x: &'static [f64],
    pub y: &'static [f64],
}

impl Table1 {
    /// Linear interpolation. Clamps outside the table rather than
    /// extrapolating — extrapolation past a fitted range is the most common
    /// silent error in this class of model, and a node that means to
    /// extrapolate declares a wider domain instead.
    pub fn at(&self, x: f64) -> f64 {
        pmath::interp(x, self.x, self.y)
    }

    /// Interpolation in the logarithm of `y`, for quantities that vary over
    /// decades. Atmospheric density falls by four orders of magnitude across
    /// the VLEO band; interpolating it linearly between table points is wrong
    /// by tens of per cent in the middle of every interval.
    pub fn at_log(&self, x: f64) -> f64 {
        if x <= self.x[0] {
            return self.y[0];
        }
        let n = self.x.len();
        if x >= self.x[n - 1] {
            return self.y[n - 1];
        }
        let mut i = 0usize;
        while i + 1 < n && self.x[i + 1] < x {
            i += 1;
        }
        let t = (x - self.x[i]) / (self.x[i + 1] - self.x[i]);
        let ly = pmath::ln(self.y[i]) + t * (pmath::ln(self.y[i + 1]) - pmath::ln(self.y[i]));
        pmath::exp(ly)
    }

    /// True when `x` lies inside the table. A node that cares about
    /// extrapolation checks this and raises its own fault, rather than
    /// receiving a clamped value it cannot tell from a real one.
    pub fn covers(&self, x: f64) -> bool {
        x >= self.x[0] && x <= self.x[self.x.len() - 1]
    }
}
