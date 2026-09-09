//! Electrical power.
//!
//! In a VLEO air-breathing design the power system is not a supporting
//! subsystem: it is usually the thing that decides whether the design closes.
//! The thruster wants kilowatts, the array has to be small because a large
//! array is drag, and the two pull in opposite directions in a way that no
//! other orbit regime forces.

use vleo_units::constants::*;
use vleo_units::pmath;
use vleo_units::*;

/// Solar array output at beginning of life.
///
/// `incidence` is the angle between the array normal and the Sun.
pub fn array_power_bol(
    area: Area,
    cell_efficiency: Ratio,
    packing_factor: Ratio,
    incidence: Angle,
) -> Power {
    let cos = pmath::max(0.0, incidence.cos());
    SOLAR_CONSTANT * area * cell_efficiency.get() * packing_factor.get() * cos
}

/// Degradation factor after `years` at a stated annual rate.
///
/// VLEO is a benign radiation environment — the mission is below the inner
/// belt — so the dominant degradation is atomic-oxygen erosion of coverglass
/// adhesives and thermal cycling, not displacement damage. The rate is a
/// declared value with a source, not a constant, because it is the one place
/// where a VLEO design differs from the textbook figure.
pub fn degradation(annual_rate: Ratio, years: f64) -> Ratio {
    Ratio::new(pmath::powf(1.0 - annual_rate.get(), years))
}

/// Array output at end of life.
pub fn array_power_eol(bol: Power, degradation: Ratio) -> Power {
    bol * degradation.get()
}

/// Orbit-average power available to the loads.
///
/// The array only generates in sunlight, the battery only discharges in
/// eclipse, and both paths have their own efficiency. This is the number a
/// power budget must be closed against — instantaneous array output is not.
pub fn orbit_average_power(
    array_power: Power,
    sunlit_fraction: Ratio,
    path_efficiency_direct: Ratio,
    path_efficiency_battery: Ratio,
    eclipse_fraction: Ratio,
) -> Power {
    let direct = array_power.get() * sunlit_fraction.get() * path_efficiency_direct.get();
    let stored = array_power.get()
        * sunlit_fraction.get()
        * path_efficiency_battery.get()
        * eclipse_fraction.get()
        / pmath::max(sunlit_fraction.get(), 1.0e-9);
    Power::new(direct * (1.0 - eclipse_fraction.get()) + stored * eclipse_fraction.get())
}

/// Array area needed to supply a demand, solved from the same relation the
/// forward direction uses — so the two cannot disagree.
pub fn required_array_area(
    demand: Power,
    cell_efficiency: Ratio,
    packing_factor: Ratio,
    degradation: Ratio,
    sunlit_fraction: Ratio,
    path_efficiency: Ratio,
    incidence: Angle,
) -> Area {
    let cos = pmath::max(1.0e-6, incidence.cos());
    Area::new(
        demand.get()
            / (SOLAR_CONSTANT.get()
                * cell_efficiency.get()
                * packing_factor.get()
                * degradation.get()
                * sunlit_fraction.get()
                * path_efficiency.get()
                * cos),
    )
}

/// Battery energy needed to carry the eclipse load.
///
/// Depth of discharge is the design decision here, and it is a lifetime
/// decision rather than an energy one: a VLEO orbit is about 92 minutes, so a
/// five-year mission is roughly 29 000 cycles, and 29 000 cycles is deep in the
/// region where depth of discharge decides whether the cells survive.
pub fn battery_energy_required(
    eclipse_load: Power,
    eclipse_duration: Time,
    depth_of_discharge: Ratio,
    discharge_efficiency: Ratio,
) -> Energy {
    Energy::new(
        eclipse_load.get() * eclipse_duration.get()
            / (depth_of_discharge.get() * discharge_efficiency.get()),
    )
}

/// Number of charge-discharge cycles over a mission — one per orbit.
pub fn battery_cycles(mission_duration: Time, orbital_period: Time) -> f64 {
    mission_duration.get() / orbital_period.get()
}

/// Battery mass from energy and specific energy.
pub fn battery_mass(energy: Energy, specific_energy_wh_per_kg: f64) -> Mass {
    Mass::new(energy.get() / 3600.0 / specific_energy_wh_per_kg)
}

/// Solar array mass from area and areal density.
pub fn array_mass(area: Area, areal_density: f64) -> Mass {
    Mass::new(area.get() * areal_density)
}

/// Power margin, as a fraction of the demand. Negative means the design does
/// not close, and it is reported as a signed number rather than clamped — a
/// margin silently corrected to zero is a design that drifted without anyone
/// deciding to.
pub fn power_margin(available: Power, demand: Power) -> Ratio {
    Ratio::new((available.get() - demand.get()) / demand.get())
}

/// Total demand, summed from the named loads. Written as a function rather than
/// left to a caller's `+` chain so that the budget has one shape everywhere and
/// a forgotten term shows up as a missing argument.
pub fn power_demand(
    propulsion: Power,
    payload: Power,
    avionics: Power,
    comms: Power,
    thermal: Power,
    harness_loss: Ratio,
) -> Power {
    let sum = propulsion.get() + payload.get() + avionics.get() + comms.get() + thermal.get();
    Power::new(sum * (1.0 + harness_loss.get()))
}
