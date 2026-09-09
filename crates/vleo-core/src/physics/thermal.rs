//! Thermal balance.
//!
//! A single-node balance: everything absorbed equals everything radiated, and
//! the spacecraft is one temperature. That is a coarse model and it is stated
//! as such — it sizes a radiator and bounds a temperature, and it does not
//! predict a gradient across a panel. A design that needs the gradient needs a
//! finite-element model, which is a different node with a different tier.

use vleo_units::constants::*;
use vleo_units::pmath;
use vleo_units::*;

/// Direct solar absorbed by a surface of area `a` at incidence `theta`.
pub fn absorbed_solar(area: Area, absorptivity: Ratio, incidence: Angle) -> Power {
    let cos = pmath::max(0.0, incidence.cos());
    SOLAR_CONSTANT * area * absorptivity.get() * cos
}

/// Albedo — sunlight reflected off the Earth. Strongly dependent on where the
/// spacecraft is over, and treated here as the mean with a view factor.
pub fn absorbed_albedo(area: Area, absorptivity: Ratio, view_factor: Ratio) -> Power {
    SOLAR_CONSTANT * area * absorptivity.get() * EARTH_ALBEDO * view_factor.get()
}

/// Earth infrared. Unlike albedo it does not switch off in eclipse, which is
/// what stops a VLEO spacecraft getting as cold as an intuition built on higher
/// orbits expects.
pub fn absorbed_earth_ir(area: Area, emissivity: Ratio, view_factor: Ratio) -> Power {
    EARTH_IR * area * emissivity.get() * view_factor.get()
}

/// View factor to the Earth from a flat plate facing nadir, at radius `r`.
pub fn earth_view_factor(radius: Length) -> Ratio {
    let s = R_EARTH.get() / radius.get();
    Ratio::new(s * s)
}

/// Aerodynamic heating in free-molecular flow.
///
/// Usually neglected, and at 200 km it should not be: the incident kinetic
/// energy flux is `0.5·rho·V^3`, and with a heat transfer coefficient near one
/// that is a real term in the balance of a small, fast, low-flying vehicle.
pub fn free_molecular_heating(
    density: MassDensity,
    velocity: Velocity,
    area: Area,
    heat_transfer_coefficient: Ratio,
) -> Power {
    let v = velocity.get();
    Power::new(0.5 * density.get() * v * v * v * area.get() * heat_transfer_coefficient.get())
}

/// Radiated power from a grey surface, `epsilon·sigma·A·T^4`.
pub fn radiated(area: Area, emissivity: Ratio, temperature: Temperature) -> Power {
    let t = temperature.get();
    Power::new(emissivity.get() * SIGMA_SB * area.get() * t * t * t * t)
}

/// Equilibrium temperature that balances a total absorbed-plus-dissipated load.
pub fn equilibrium_temperature(
    total_absorbed: Power,
    internal_dissipation: Power,
    radiating_area: Area,
    emissivity: Ratio,
) -> Temperature {
    let q = total_absorbed.get() + internal_dissipation.get();
    let denom = emissivity.get() * SIGMA_SB * radiating_area.get();
    Temperature::new(pmath::powf(q / denom, 0.25))
}

/// Radiator area needed to hold a temperature against a load.
pub fn required_radiator_area(
    heat_to_reject: Power,
    emissivity: Ratio,
    radiator_temperature: Temperature,
    sink_temperature: Temperature,
) -> Area {
    let t = radiator_temperature.get();
    let ts = sink_temperature.get();
    let net = emissivity.get() * SIGMA_SB * (t * t * t * t - ts * ts * ts * ts);
    if net <= 0.0 {
        return Area::new(f64::INFINITY);
    }
    Area::new(heat_to_reject.get() / net)
}

/// Thermal margin, in kelvin, against a declared limit.
pub fn thermal_margin(predicted: Temperature, limit: Temperature) -> Temperature {
    Temperature::new(limit.get() - predicted.get())
}
