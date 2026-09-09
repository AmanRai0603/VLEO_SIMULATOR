// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much data is one imaged scene?
///
/// `V = N_x*N_y*b*bands/CR`
///
/// Source: `ccsds122`
pub const NODE_ID: &str = "pay_scene_volume";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x04360bd70a781701;

pub fn evaluate(n: Ratio, b: Ratio, cr: Ratio) -> Result<DataVolume, Fault> {
    // ---- HOLE 1 : assume a square scene of the across-track width, in four spectral bands -> DataVolume
    let v: DataVolume = payload::scene_data_volume(n.get(), n.get(), b.get(), 4.0, cr.get());
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: DataVolume = v;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "V_scene", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "V_scene", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: DataVolume::UNIT, reason: "a data volume cannot be negative" });
    }
    if answer.get() > 1000000000000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "V_scene", value: answer.get(), bound: 1000000000000000.0, edge: Edge::Upper, unit: DataVolume::UNIT, reason: "above a petabit no single scene in this design is produced" });
    }
    Ok(answer)
}
