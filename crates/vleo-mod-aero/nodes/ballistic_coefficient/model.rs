// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How well does this vehicle resist the atmosphere, per unit of its own mass?
///
/// `BC = m/(Cd*A)`
///
/// Source: `vallado2013`
///
/// Higher is better here. A cubesat is around 50; a slender VLEO platform
/// reaches 150, and that difference is worth tens of kilometres of altitude.
pub const NODE_ID: &str = "aero_ballistic_coefficient";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x7512f2f652dac3fa;

pub fn evaluate(m: Mass, cd: Ratio, a: Area) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : divide the wet mass by the product of drag coefficient and frontal area -> Ratio
    let bc: Ratio = Ratio::new(aero::ballistic_coefficient(m, cd.get(), a));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = bc;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "BC", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 5.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "BC", value: answer.get(), bound: 5.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 5 kg/m2 the vehicle is a sail, and nothing in this band closes" });
    }
    if answer.get() > 500.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "BC", value: answer.get(), bound: 500.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 500 kg/m2 the vehicle is denser than any spacecraft ever flown" });
    }
    Ok(answer)
}
