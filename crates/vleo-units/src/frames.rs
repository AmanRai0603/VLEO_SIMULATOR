//! Reference frames as types.
//!
//! A `Vec3<Eci>` and a `Vec3<Ecef>` are different types, so passing one where
//! the other is expected does not compile. The frame is a zero-sized marker: it
//! costs nothing and it removes the second most common class of silent error in
//! flight dynamics after units.
//!
//! Rotations are not defined here. Frame conversion is a node like any other —
//! it has a question, a source and evidence — and a conversion that lives in a
//! utility module is a conversion nobody reviewed.

use crate::pmath;
use core::marker::PhantomData;

/// Marker trait for a reference frame.
pub trait Frame: Copy + core::fmt::Debug {
    /// The name that appears on a page and in a ledger row.
    const NAME: &'static str;
}

/// Earth-centred inertial, J2000 equator and equinox.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Eci;
impl Frame for Eci {
    const NAME: &'static str = "ECI (J2000)";
}

/// Earth-centred, Earth-fixed. Rotates with the planet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ecef;
impl Frame for Ecef {
    const NAME: &'static str = "ECEF";
}

/// Local vertical, local horizontal — radial, along-track, cross-track.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lvlh;
impl Frame for Lvlh {
    const NAME: &'static str = "LVLH (RTN)";
}

/// Spacecraft body frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Body;
impl Frame for Body {
    const NAME: &'static str = "Body";
}

/// A three-component vector tagged with the frame it is expressed in.
///
/// The components are bare `f64` rather than typed quantities: a vector is used
/// for positions, velocities and torques alike, and giving it a quantity type
/// as well as a frame doubles the type surface for no defect it would catch
/// that the calling signature does not already catch.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Vec3<F: Frame> {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    _frame: PhantomData<F>,
}

impl<F: Frame> Vec3<F> {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 {
            x,
            y,
            z,
            _frame: PhantomData,
        }
    }
    pub const ZERO: Self = Vec3::new(0.0, 0.0, 0.0);

    #[inline]
    pub fn norm(self) -> f64 {
        pmath::sqrt(self.x * self.x + self.y * self.y + self.z * self.z)
    }
    #[inline]
    pub fn dot(self, o: Self) -> f64 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }
    #[inline]
    pub fn cross(self, o: Self) -> Self {
        Vec3::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }
    #[inline]
    pub fn scale(self, k: f64) -> Self {
        Vec3::new(self.x * k, self.y * k, self.z * k)
    }
    /// Named `plus` rather than `add`: a `Vec3<Eci>` and a `Vec3<Ecef>` must
    /// not be addable, so the standard `Add` trait is deliberately not
    /// implemented, and an inherent method that shadows it by name would be
    /// read as though it were.
    #[inline]
    pub fn plus(self, o: Self) -> Self {
        Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
    #[inline]
    pub fn minus(self, o: Self) -> Self {
        Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
    /// Unit vector, or zero when the input is zero — a caller that cares about
    /// the degenerate case checks the norm first and raises its own fault.
    #[inline]
    pub fn unit(self) -> Self {
        let n = self.norm();
        if n == 0.0 {
            Self::ZERO
        } else {
            self.scale(1.0 / n)
        }
    }
    /// The angle between two vectors in the same frame, in radians.
    #[inline]
    pub fn angle_to(self, o: Self) -> f64 {
        let d = self.unit().dot(o.unit());
        pmath::acos(pmath::max(-1.0, pmath::min(1.0, d)))
    }
    /// The frame name, for a page or a ledger row.
    pub const fn frame_name() -> &'static str {
        F::NAME
    }
}
