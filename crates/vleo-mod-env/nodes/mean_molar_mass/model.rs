// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What is the mean molar mass of the air the vehicle is flying through?
///
/// `M = rho*N_A/n`
///
/// Source: `jacchia1971`
///
/// Falls from 26 g/mol at 120 km to near 16 — pure atomic oxygen — by 350
/// km. Every free-molecular relation downstream reads it.
pub const NODE_ID: &str = "env_mean_molar_mass";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x1f6c235a719a7055;

pub fn evaluate(h: Length, t_inf: Temperature) -> Result<MolarMass, Fault> {
    // ---- HOLE 1 : take the mass-weighted mean over the five species -> MolarMass
    let m: MolarMass = env::composition(h, t_inf).mean_molar_mass();
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: MolarMass = m;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "M", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.002 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "M", value: answer.get(), bound: 0.002, edge: Edge::Lower, unit: MolarMass::UNIT, reason: "below 2 g/mol the mixture would be pure hydrogen, which does not occur in this band" });
    }
    if answer.get() > 0.03 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "M", value: answer.get(), bound: 0.03, edge: Edge::Upper, unit: MolarMass::UNIT, reason: "above 30 g/mol the mixture would be heavier than molecular nitrogen, which does not occur above the base" });
    }
    Ok(answer)
}
