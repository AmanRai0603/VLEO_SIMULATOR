// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What is the drag coefficient of this body, at this speed ratio, with this surface?
///
/// `Cd = P/sqrt(pi) + gamma*Q*Z + (gamma/2)*vr*(gamma*sqrt(pi)*Z + P)`
///
/// Source: `doornbos2011`
///
/// Treating this as a constant 2.2 is the single largest avoidable error in a
/// VLEO drag estimate, and it is what most concept studies do.
///
/// # Assumptions
///
/// * Free-molecular flow — a molecule leaving the surface never returns and never strikes an incoming one — fails when below Kn = 10, which is roughly 150 km for this body; the Knudsen number is an input so that the guard on it is visible here
pub const NODE_ID: &str = "aero_drag_coefficient";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x05d4235d29c026dd;

pub fn evaluate(s: Ratio, l: Length, d: Length, alpha: Ratio, t_w: Temperature, v: Velocity, m: MolarMass, kn: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : evaluate Sentman's free-molecular coefficients for the front face and the side walls, referred back to the frontal area -> Ratio
    let cd: Ratio = Ratio::new(aero::cylinder_drag_coefficient(s.get(), l, d, alpha.get(), t_w, v, m));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = cd;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Cd", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 1.5 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Cd", value: answer.get(), bound: 1.5, edge: Edge::Lower, unit: Ratio::UNIT, reason: "no free-molecular body has a drag coefficient below 1.5; a lower value means the speed ratio or the accommodation is wrong" });
    }
    if answer.get() > 5.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Cd", value: answer.get(), bound: 5.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 5.0 the geometry is not the slender body this model assumes" });
    }
    Ok(answer)
}
