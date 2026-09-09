// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much air is there at the flight altitude, in kilograms per cubic metre?
///
/// `rho = SUM_i n_i(z)*m_i,  n_i(z) = n_i(z0)*(T0/T)^(1+alpha_i)*exp(-INT M_i*g/(R*T) dz)`
///
/// Source: `jacchia1971`
///
/// # Assumptions
///
/// * Diffusive equilibrium above 120 km, five species — fails when below about 100 km turbulent mixing dominates and species do not separate; the model is not valid there
/// * No diurnal, seasonal or longitudinal variation — fails when the real density at a fixed altitude varies by a factor of two around the orbit; this returns the daily mean
pub const NODE_ID: &str = "env_mass_density";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x6ee594b58f539680;

pub fn evaluate(h: Length, t_inf: Temperature) -> Result<MassDensity, Fault> {
    // ---- HOLE 1 : integrate diffusive equilibrium from the 120 km base for every species and sum the masses -> MassDensity
    let rho: MassDensity = env::mass_density(h, t_inf);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: MassDensity = rho;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "rho", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 1e-15 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "rho", value: answer.get(), bound: 1e-15, edge: Edge::Lower, unit: MassDensity::UNIT, reason: "below 1e-15 kg/m3 the free-molecular relations downstream stop producing a meaningful drag" });
    }
    if answer.get() > 1e-6 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "rho", value: answer.get(), bound: 1e-6, edge: Edge::Upper, unit: MassDensity::UNIT, reason: "above 1e-6 kg/m3 the flow is no longer free-molecular and every aerodynamic node in this tree is outside its envelope" });
    }
    Ok(answer)
}
