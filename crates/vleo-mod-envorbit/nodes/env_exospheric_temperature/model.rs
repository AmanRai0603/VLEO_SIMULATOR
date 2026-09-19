// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How hot is the upper thermosphere today, given the Sun and the geomagnetic field?
///
/// `T_inf = 379 + 3.24*F10.7A + 1.3*(F10.7 - F10.7A) + 28*Kp + 0.03*exp(Kp)`
///
/// Source: `jacchia1971`
///
/// # Assumptions
///
/// * Night-time minimum, with no diurnal or seasonal term — fails when the diurnal bulge adds up to 30% at 14:00 local solar time; a design sized on this value alone is sized on the quiet side
/// * Kp is a THREE-HOURLY index and what reaches this row is a daily statistic — fails when the two are treated as the same quantity. Jacchia's geomagnetic correction takes the Kp of an interval; the solar subsystem publishes a daily MEAN slot and a daily PEAK slot per scenario and they are different numbers — 3.832 against 5.198 at the hot sustained scenario. Fed through this relation that is 824.9 K against 867.2 K, and on the worst-day scenario it is 914.7 K against 1055.3 K, a 15 per cent difference in the quantity every density in this tree is built from. Nothing in the tree yet says which slot the design is driven by; §30 B1 is the row that would
/// * The geomagnetic correction is applied with no lag — fails when the timing of a storm matters rather than only its size. Jacchia applies the correction to Kp lagged by about a quarter of a day, because the energy deposited in the auroral zone takes hours to reach the altitudes and latitudes this is a temperature for. Applied instantaneously, a storm's heating arrives too early and leaves too early, and the peak is placed about six hours before it happens
/// * F10.7 is the PREVIOUS day's flux in the source, and this reads the same day's — fails when the fast term is read as a same-day response. The EUV that heated the thermosphere is yesterday's, which is why the source lags it; taking today's makes the departure term lead the temperature it is meant to explain by one day. Small on a monthly mean and not small on a single design day, which is exactly where the fast term is doing the work
/// * No semiannual term, on a record that measures one — fails when the answer is read as the same in March as in June. The thermosphere has a well-known semiannual density variation with maxima near the equinoxes, and this subsystem MEASURES it — sw_semiannual_amplitude, off the record's own day-of-year means, and the climate panel draws it with both equinoxes marked. This relation has no term for it, so the measurement exists in the tree and does not reach the temperature
/// * These are the global night-time coefficients, and the source has variants — fails when this is compared against another implementation of 'Jacchia 1971' and they disagree. The 1971 model is a family — different coefficient sets for the static diffusion tables, for the exospheric temperature, and for the geomagnetic correction above and below 350 km. Nothing here records which set these four coefficients are, so a disagreement cannot be traced to a variant rather than to a defect
pub const NODE_ID: &str = "env_exospheric_temperature";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x2e87228e844416ab;

pub fn evaluate(f107: Ratio, f107a: Ratio, kp: Ratio) -> Result<Temperature, Fault> {
    // ---- HOLE 1 : apply the Jacchia 1971 night-time minimum relation with its geomagnetic correction -> Temperature
    let t_inf: Temperature = env::exospheric_temperature(f107.get(), f107a.get(), kp.get());
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Temperature = t_inf;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "T_inf", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_inf", value: answer.get(), bound: 400.0, edge: Edge::Lower, unit: Temperature::UNIT, reason: "no observed thermosphere is colder than 400 K; below that the Bates profile inverts" });
    }
    if answer.get() > 2500.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_inf", value: answer.get(), bound: 2500.0, edge: Edge::Upper, unit: Temperature::UNIT, reason: "above 2500 K is beyond any recorded storm and beyond the fit" });
    }
    Ok(answer)
}
