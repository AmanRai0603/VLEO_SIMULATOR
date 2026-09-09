//! Numerical methods.
//!
//! Only the things the physics actually needs: integrators, root finding and
//! table interpolation. Linear algebra is not here — no node in this system has
//! yet needed a decomposition, and a linear algebra module written before a
//! node needs it is a module nobody reviews.
//!
//! Everything is `no_std`, allocation-free, and generic over closures rather
//! than trait objects: an integrator that is monomorphised compiles to
//! straight-line code, and a dynamic call in an inner loop does not.

pub mod integrate;
pub mod rootfind;
pub mod table;

pub use integrate::{rk4_scalar, simpson};
pub use rootfind::{bisect, secant, RootError};
pub use table::Table1;
