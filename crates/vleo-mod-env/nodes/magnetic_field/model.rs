// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How strong is the magnetic field at the flight altitude?
///
/// `B = B0*(Re/r)^3*sqrt(1 + 3*sin^2(lat_m))`
///
/// Source: `igrf2020`
///
/// # Assumptions
///
/// * Centred dipole, no higher-order terms — fails when the South Atlantic Anomaly departs from a dipole by up to 30%; a magnetorquer sized on this alone is undersized there
pub const NODE_ID: &str = "env_magnetic_field";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x803500f0a601e268;

pub fn evaluate(r: Length, lat_m: Angle) -> Result<MagneticFluxDensity, Fault> {
    // ---- HOLE 1 : evaluate the tilted dipole approximation at the orbital radius -> MagneticFluxDensity
    let b: MagneticFluxDensity = env::magnetic_field(r, lat_m);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: MagneticFluxDensity = b;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "B", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 1e-6 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "B", value: answer.get(), bound: 1e-6, edge: Edge::Lower, unit: MagneticFluxDensity::UNIT, reason: "the field never falls below a microtesla at these altitudes" });
    }
    if answer.get() > 0.0001 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "B", value: answer.get(), bound: 0.0001, edge: Edge::Upper, unit: MagneticFluxDensity::UNIT, reason: "the surface equatorial field is 3.1e-5 T; 1e-4 would be four times the polar surface value" });
    }
    Ok(answer)
}
