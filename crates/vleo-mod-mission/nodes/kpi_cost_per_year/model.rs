// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// Does a year of service cost what the business case allows?
///
/// `M = (achieved - required)/required, in the sense the requirement is stated`
///
/// Source: `orbitt_case_c1`
pub const NODE_ID: &str = "kpi_cost_per_year";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xddf36607236bc15e;

pub fn evaluate(req: Money, ach: Money) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : compare achieved against required in the declared sense and return the signed fractional margin -> Ratio
    let m: Ratio = Ratio::new(mission::closure(req.get(), ach.get(), mission::Sense::AtMost).margin);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = m;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "M", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < -100.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "M", value: answer.get(), bound: -100.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a margin below -10000% means the inputs are not the ones the requirement was written against" });
    }
    if answer.get() > 1000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "M", value: answer.get(), bound: 1000.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "a margin above 100000% means the requirement is not the one that binds" });
    }
    Ok(answer)
}
