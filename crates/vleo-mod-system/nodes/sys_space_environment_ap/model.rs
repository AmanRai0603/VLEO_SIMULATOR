// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What geomagnetic index does the system design to, as a level sustained over the mission?
///
/// `Ap_sys = l3_solar_interface.ap_hotmean`
///
/// Source: `noaa_swpc`
///
/// A day at this level may cost a safe mode, a slewed array and some
/// propellant; what it may not cost is the vehicle. That is a different
/// design case from the sustained level its sibling carries, and the two are
/// separate rows here because they are separate rows in the subsystem below.
///
/// # Assumptions
///
/// * It receives and does not compute, and a reader here sees none of the subsystem's limitations — fails when a margin is taken against this number. It stacks a 1.28-sigma band edge on the rotation level with a within-rotation percentile on top, which is nearer a one-in-a-hundred day than a one-in-twenty one, and the two terms are not independent because a disturbed rotation is made of disturbed days. None of that crosses the seam
/// * It names one member of a fifteen-variable set, and five of them look alike — fails when the wrong member is named. The crossing's f107 column alone holds five values in the same range with the same unit and the same declared domain, and the ap column another five. Assembly checks that the variable EXISTS and that its type matches; nothing checks that it is the one this row meant
/// * It is the single day and not the sustained level — fails when somebody integrates it over a mission. The sustained level is what sys_space_environment_solar_flux carries, and a drag budget or an array sizing built on a single-day value is designing for a sky the mission does not sit in
pub const NODE_ID: &str = "sys_space_environment_ap";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x643016096e92c010;

pub fn evaluate(crossing: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : receive the subsystem's single-day conclusion across the seam -> Ratio
    // A layer-2 row receives; it does not compute. The one thing that can go
    // wrong here is that the seam alters what it carries — a stray factor, an
    // unasked-for unit conversion, a clamp inherited from the wrong row — and
    // both sides would still look plausible. So this is the identity, and the
    // fixtures beside it pin the identity at real values.
    //
    // The declared range is the crossing's own, restated so a system reader sees
    // the limit without opening the subsystem. It therefore guards nothing this
    // line can break, and that is correct.
    let received: Ratio = crossing;
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = received;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Ap_sys", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_sys", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the crossing's own floor, restated. A row that narrowed the range it received would be changing the answer while appearing to relay it" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_sys", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the crossing's own ceiling, restated. This is a single-day value, so it is the one most likely of the pair to approach it" });
    }
    Ok(answer)
}
