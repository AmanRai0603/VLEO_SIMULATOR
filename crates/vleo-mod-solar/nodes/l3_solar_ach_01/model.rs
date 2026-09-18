// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much room does the sustained F10.7 requirement have left?
///
/// `M_f107_long = closure(F107_req_long, F107_long, AtMost).margin`
///
/// Source: `noaa_swpc`
///
/// The headline of the five. It is the same number the interface carries to
/// sys_space_environment as its primary member, restated on the achieved side
/// of the closure so that the comparison against what the spacecraft can
/// sustain happens on a row rather than in somebody's head.
///
/// # Assumptions
///
/// * It inherits every limitation of the row it restates, and a closure reading it sees none of them — fails when the same cost as the interface, and worth repeating on the row a closure actually binds. The number is sized on the cycle analogue at the mission's own epoch rather than on the record's unconditional mean, which this chain now reads the epoch to do; where it is a band it is 1.28 sigma, the 90th percentile, and not the 95 per cent the run is labelled; and where it is a return level its top end rests on two observations in 28.2 years. A margin computed from this row against a capability carries none of that, and will look like a clean number either way.
pub const NODE_ID: &str = "l3_solar_ach_01";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x906ef3769bef9709;

pub fn evaluate(ach: Ratio, req: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : compare achieved against required in the declared sense and return the signed fractional margin -> Ratio
    // The requirement declares sense "<=", so the achieved value must stay UNDER
    // the bound and the margin is (required - achieved) / required. Positive is
    // room; negative is a violation and its size. mission::closure is the twelve
    // KPI closures' own function rather than the arithmetic written out again,
    // because a second way of computing a margin is a second way of getting its
    // sign wrong -- and a sign error here still produces a plausible number.
    let m: Ratio = Ratio::new(mission::closure(req.get(), ach.get(), mission::Sense::AtMost).margin);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = m;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "M_f107_long", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < -10.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "M_f107_long", value: answer.get(), bound: -10.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a margin of -10 is a requirement exceeded by eleven times its own value. Below that the comparison has stopped being a design margin and become a sign that one side is in the wrong unit, and it should refuse rather than report a number nobody will read as a fraction" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "M_f107_long", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "1 is the whole of the requirement: an achieved value of zero against a positive bound. A margin above 1 under this sense would mean the achieved value is negative, which none of these drivers can be" });
    }
    Ok(answer)
}
