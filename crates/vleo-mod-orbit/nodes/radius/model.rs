// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How far is the spacecraft from the centre of the Earth?
///
/// `r = R_earth + h`
///
/// Source: `wgs84`
pub const NODE_ID: &str = "orbit_radius";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x23a3a87429910134;

pub fn evaluate(h: Length) -> Result<Length, Fault> {
    // ---- HOLE 1 : add the WGS-84 equatorial radius to the altitude -> Length
    let r: Length = orbit::radius(h);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Length = r;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "r", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 6400000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "r", value: answer.get(), bound: 6400000.0, edge: Edge::Lower, unit: Length::UNIT, reason: "below the Earth's mean radius is not an orbit" });
    }
    if answer.get() > 7000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "r", value: answer.get(), bound: 7000000.0, edge: Edge::Upper, unit: Length::UNIT, reason: "above 7000 km is outside the band this tool is scoped to" });
    }
    Ok(answer)
}
