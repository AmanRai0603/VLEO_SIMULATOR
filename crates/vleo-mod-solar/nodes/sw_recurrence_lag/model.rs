// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// At what lag does the solar-rotation signal in F10.7 peak?
///
/// `L_rot = argmax over lag of corr(F107', F107' shifted by lag) = 26 d`
///
/// Source: `noaa_swpc`
///
/// The lag at which today's F10.7 best predicts a future day, once the
/// 11-year cycle is removed. It is the reason a 27-day outlook is 27 days
/// long, and it is the shortest horizon at which a design can expect an
/// active region to come back round.
///
/// # Assumptions
///
/// * 26 is where the first peak sits and 27 is the period, and the one-day gap is a known bias in the estimator rather than a disagreement — fails when the first peak is read as the rotation period. The autocorrelation of the detrended series decays steeply from +0.936 at lag 1 to -0.066 at lag 13, and the rotation bump rides on the tail of that decay, so its apparent peak is pulled toward zero lag. The harmonics settle it: the second peak is at lag 54 and the third at lag 81, both exactly 27.0 days per cycle, and they sit far enough out that the decay no longer tilts them. So the period is 27 days and the first-peak estimate is one day short. This row reports the measured peak, which is what the MATLAB field C.rot_peak_lag holds; a row that needs the period should use 27 and cite the harmonics
/// * The bump is broad, so the single lag overstates how sharp the recurrence is — fails when it is treated as a period a design can phase-lock to. Correlation exceeds +0.34 at every lag from 24 to 28 and exceeds +0.30 from 23 to 29 — a seven-day-wide shoulder. The Sun does not rotate as a solid body: the equator turns in about 25 days and mid-latitudes in about 28, and active regions emerge and decay within a rotation. So the recurrence is a tendency over a week-wide window, not a clock
/// * The 11-year cycle was removed with a 365-day centred mean, and that choice sets what is left — fails when the detrending window is comparable to the signal. At 365 days it is thirteen rotations long, so it removes the cycle and the annual terms while leaving the rotation untouched; the residual has a standard deviation of 19.85 sfu about a mean of -0.03. A window near 27 days would remove the rotation itself and this row would measure nothing. 3 of 10319 days in the record have no F10.7 and are excluded pairwise rather than interpolated
pub const NODE_ID: &str = "sw_recurrence_lag";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xe8db92a8b25cef88;

pub fn evaluate() -> Result<Time, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Time = match Time::from_unit(26.0, Unit::Day) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "L_rot", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Time = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "L_rot", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 1728000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "L_rot", value: answer.get(), bound: 1728000.0, edge: Edge::Lower, unit: Time::UNIT, reason: "below 20 days the autocorrelation is still on the steep descent from lag 1 and is falling, not peaking: it reads +0.186 at lag 20 against +0.376 at the peak. A value there would mean the detrending removed the rotation instead of the cycle" });
    }
    if answer.get() > 3024000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "L_rot", value: answer.get(), bound: 3024000.0, edge: Edge::Upper, unit: Time::UNIT, reason: "above 35 days the first bump has closed — the correlation is +0.031 at lag 35 and negative by 36 — and anything beyond is the second harmonic near lag 54, which is the same signal counted twice. A value there would be reporting a harmonic as the fundamental" });
    }
    Ok(answer)
}
