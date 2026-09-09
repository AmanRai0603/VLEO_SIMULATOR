//! RING 0 — the leaf crate. Units, frames, constants and portable maths.
//!
//! Three decisions live here because none of them can be retrofitted across
//! hundreds of nodes later:
//!
//! 1. **Units are types.** Adding a [`Newton`] to a [`Kilogram`] does not
//!    compile. This is the Mars Climate Orbiter failure made impossible rather
//!    than documented.
//! 2. **Frames are types.** A position expressed in ECI cannot be passed where
//!    an ECEF position is expected.
//! 3. **Portable maths.** `sin`, `cos`, `exp`, `ln` and `powf` come from
//!    [`pmath`], never from the platform maths library. A native build and a
//!    WebAssembly build otherwise differ in the last bit, and the nightly
//!    "golden vectors agree bit for bit" gate fails on night one for a reason
//!    that is not a defect — after which the team learns to ignore a red build.
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

pub mod constants;
pub mod frames;
pub mod pmath;
pub mod quantity;
pub mod unit;

pub use constants::*;
pub use frames::{Body, Ecef, Eci, Frame, Lvlh, Vec3};
pub use quantity::*;
pub use unit::Unit;
