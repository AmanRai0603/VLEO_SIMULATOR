// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// Which activity band does this F10.7 fall in?
///
/// `band(F107) = 1 + count(edges <= F107),  edges = 90, 130, 170 sfu`
///
/// Source: `noaa_swpc`
///
/// The coarse label that puts a flux value in context — a reader who sees
/// 228 sfu and does not work with F10.7 daily has no idea whether that is
/// ordinary or extreme. The band says it is the top one, and that 12.8% of
/// the record sits there.
///
/// # Assumptions
///
/// * The bands are a published convention and this row is a lookup, not a measurement — fails when the four levels — low below 90, moderate 90 to 129, elevated 130 to 169, high 170 and above — are the standard NOAA F10.7 activity levels and prf_segment applies exactly these. Nothing here is fitted, so there is nothing in it to be wrong about this record, and equally nothing in it that adapts to this record: prf_segment ALSO offers data-driven terciles of the same quantity, which cut the archive into equal thirds and land in different places. Those are a different row and this is not it.
/// * A band is an ordinal label carried as a number, and arithmetic on it is meaningless — fails when the answer is 1, 2, 3 or 4 and the gaps between them are not equal in sfu — band 1 spans 26 sfu of observed record, band 4 spans 173. Averaging bands, interpolating between them, or treating band 4 as twice band 2 are all errors this row cannot prevent, because the tree carries one scalar per row and a label has to arrive as one. A consumer that wants a flux wants env_f107 or sw_f107_design.
/// * It bands a single day's flux, and a mission does not live on one day — fails when F10.7 moves through every band over any mission longer than a few months — the record spends 39.8% of its days in band 1 and 12.8% in band 4 — so banding the design value says which band the DESIGN POINT sits in and not which band the mission will experience. Reading it as the latter would be reading a design percentile as a forecast.
pub const NODE_ID: &str = "sw_activity_band";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x13655a54560fb263;

pub fn evaluate(f107: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : count how many of the three published edges this flux has reached, and add one -> Ratio
    // The three published edges. prf_segment applies the same three, and the
    // comparison is >= rather than > so that a flux sitting exactly on an edge
    // belongs to the band the edge opens — 90.0 sfu is moderate, not low. That
    // choice is what the boundary fixtures beside this exist to pin, because every
    // off-by-one here produces a plausible-looking label.
    const EDGES: [f64; 3] = [90.0, 130.0, 170.0];
    let mut n: f64 = 1.0;
    let mut i: usize = 0;
    while i < EDGES.len() {
        if f107.get() >= EDGES[i] {
            n += 1.0;
        }
        i += 1;
    }
    let b: Ratio = Ratio::new(n);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = b;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "band", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "band", value: answer.get(), bound: 1.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "there are four bands and the lowest is 1. A zero or negative band means the counting started in the wrong place, which would shift every label by one and still look like a valid answer" });
    }
    if answer.get() > 4.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "band", value: answer.get(), bound: 4.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "there are four bands and the highest is 4, unbounded above in flux — band 4 holds everything from 170 sfu upward, including the record's largest day at 343. A fifth band means an edge was added without the range being updated" });
    }
    Ok(answer)
}
