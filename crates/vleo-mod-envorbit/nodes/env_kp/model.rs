// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What planetary geomagnetic index is the design sized to?
///
/// `Kp = Kp_sys`
///
/// Source: `orbitt_case_c1`
///
/// The geomagnetic driver of the thermosphere relation, computed by the
/// solar- weather subsystem and carried here through layer 2. WHICH SLOT this
/// is remains open. The subsystem publishes a daily MEAN slot and a daily
/// PEAK slot per scenario and they are different numbers; sw_kp_driving_slot
/// is the seeded row that will say which one a design is driven by, and it
/// needs a person. Until it carries a value this row carries what layer 2
/// publishes, and env_exospheric_temperature's second assumption says so at
/// length.
pub const NODE_ID: &str = "env_kp";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x2181dd88d8825954;

pub fn evaluate(from_system: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : carry the system layer's geomagnetic index through unchanged -> Ratio
    let index: Ratio = from_system;
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = index;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kp", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp is defined on 0..9; a negative index is not a quiet day, it is a unit error" });
    }
    if answer.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp", value: answer.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Kp is defined on 0..9. This is the guard that catches an Ap value passed in by mistake, which would otherwise return an exospheric temperature of 1e18 K without complaint" });
    }
    Ok(answer)
}
