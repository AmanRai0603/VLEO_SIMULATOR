//! Propulsion — and the part of it nobody else has: air-breathing electric
//! propulsion.
//!
//! An ABEP system collects the atmosphere it is flying through, ionises it, and
//! throws it out of the back faster than it came in. If the thrust it produces
//! exceeds the drag the vehicle suffers, the orbit holds indefinitely with no
//! stored propellant. That single inequality — thrust over drag, at the design
//! altitude, at the worst solar activity the mission must survive — is what the
//! whole design closes against.
//!
//! The chain is: intake collects, thruster ionises and accelerates, power
//! processing pays for it, and the platform has to carry the power system that
//! results. Every link has an efficiency below one, and their product is small.

use crate::physics::env;
use vleo_units::constants::*;
use vleo_units::pmath;
use vleo_units::*;

/// Incident mass flux on a surface facing the flow, `rho·V` in kg/m^2/s.
///
/// At 200 km and 7.8 km/s this is about 1.3e-6 kg/m^2/s — a square metre of
/// intake sees roughly 1.3 milligrams per second arrive, and that number is the
/// ceiling on everything downstream.
pub fn incident_mass_flux(density: MassDensity, velocity: Velocity) -> MassFlux {
    density * velocity
}

/// The result of the intake balance: what reaches the thruster, and how much
/// the gas was compressed to get there.
#[derive(Clone, Copy, Debug)]
pub struct IntakeResult {
    /// Fraction of the flux entering the mouth that reaches the thruster.
    pub collection_efficiency: Ratio,
    /// Chamber number density over free-stream number density.
    pub compression_ratio: Ratio,
    /// Number density in the collection chamber.
    pub chamber_density: NumberDensity,
}

/// Steady-state balance for a free-molecular intake.
///
/// The gas arrives hyperthermally — as a beam, at 7.8 km/s — and leaves
/// thermally, at the mean speed of a gas at chamber temperature, in every
/// direction. That asymmetry is the entire physics of the device:
///
/// ```text
/// in:   n_inf · V · A_in · eta_geo
/// out:  n_c · (v_bar/4) · (A_out + beta·A_in)
/// ```
///
/// with `A_out` the throat to the thruster and `beta` the probability that a
/// thermal molecule in the chamber finds its way back out of the mouth. Solving
/// for the chamber density gives both numbers a design cares about:
///
/// ```text
/// CR    = n_c/n_inf = 4·V·A_in·eta_geo / (v_bar·(A_out + beta·A_in))
/// eta_c = A_out·eta_geo / (A_out + beta·A_in)
/// ```
///
/// Reading the second of those is worth doing slowly. **The collection
/// efficiency does not depend on the flight speed at all** — only on the
/// geometry. Speed buys compression, not capture. A design that tries to raise
/// capture by flying faster is optimising the wrong term, and the relation says
/// so on its face.
///
/// * `eta_geo` — the fraction of the mouth that is open duct rather than
///   structure, times the transmission of the duct to a beam.
/// * `beta` — back-flow transmission of the duct to a thermal molecule. A long,
///   narrow, honeycombed duct makes this small, which is what an intake *is*.
pub fn intake_balance(
    free_stream_density: NumberDensity,
    velocity: Velocity,
    intake_area: Area,
    throat_area: Area,
    eta_geo: Ratio,
    beta_backflow: Ratio,
    chamber_temperature: Temperature,
    molar_mass: MolarMass,
) -> IntakeResult {
    let v_bar = env::mean_thermal_speed(chamber_temperature, molar_mass).get();
    let a_in = intake_area.get();
    let a_out = throat_area.get();
    let denom = a_out + beta_backflow.get() * a_in;
    let cr = 4.0 * velocity.get() * a_in * eta_geo.get() / (v_bar * denom);
    let eta_c = a_out * eta_geo.get() / denom;
    IntakeResult {
        collection_efficiency: Ratio::new(eta_c),
        compression_ratio: Ratio::new(cr),
        chamber_density: NumberDensity::new(free_stream_density.get() * cr),
    }
}

/// Mass flow reaching the thruster.
pub fn collected_mass_flow(
    density: MassDensity,
    velocity: Velocity,
    intake_area: Area,
    collection_efficiency: Ratio,
) -> MassFlow {
    incident_mass_flux(density, velocity) * intake_area * collection_efficiency.get()
}

/// Ion mass flow, from the propellant utilisation efficiency.
///
/// Propellant utilisation is the fraction of the collected mass that is ionised
/// and therefore accelerable. Everything else leaves the thruster at thermal
/// speed and contributes nothing but a small drag penalty.
pub fn ion_mass_flow(collected: MassFlow, propellant_utilisation: Ratio) -> MassFlow {
    collected * propellant_utilisation.get()
}

/// Exhaust velocity of a beam accelerated through a potential, `sqrt(2qV/m)`.
///
/// The molar mass matters more than it looks: atomic oxygen at 16 g/mol reaches
/// a far higher exhaust velocity for the same accelerating voltage than xenon
/// at 131, which is the one respect in which air-breathing propulsion is
/// *easier* than stored-propellant propulsion rather than harder.
pub fn beam_exhaust_velocity(beam_voltage: Voltage, molar_mass: MolarMass) -> Velocity {
    let ion_mass = molar_mass.get() / N_AVOGADRO;
    Velocity::new(pmath::sqrt(
        2.0 * ELEMENTARY_CHARGE * beam_voltage.get() / ion_mass,
    ))
}

/// Thrust from an ion beam, with divergence and doubly-charged corrections.
///
/// * `alpha_div` — cosine loss from beam divergence, typically 0.95–0.99
/// * `alpha_double` — correction for doubly-charged ions, typically 0.95–1.0
pub fn beam_thrust(
    ion_flow: MassFlow,
    exhaust_velocity: Velocity,
    alpha_div: Ratio,
    alpha_double: Ratio,
) -> Force {
    ion_flow * exhaust_velocity * alpha_div.get() * alpha_double.get()
}

/// Jet power, `F^2/(2·m_dot)` — the power that actually ends up in the beam.
pub fn jet_power(thrust: Force, ion_flow: MassFlow) -> Power {
    if ion_flow.get() <= 0.0 {
        return Power::new(f64::INFINITY);
    }
    Power::new(thrust.get() * thrust.get() / (2.0 * ion_flow.get()))
}

/// Power spent creating the ions, from the energy cost per ion.
///
/// `epsilon_i` is in electronvolts per ion and is *not* the ionisation
/// potential: a real ioniser spends 2 to 10 times the theoretical minimum on
/// excitation, radiation and wall losses. Atomic oxygen ionises at 13.6 eV; an
/// inductively coupled plasma source spends 100 to 400 eV per ion to do it.
pub fn ionisation_power(ion_flow: MassFlow, molar_mass: MolarMass, epsilon_ev: f64) -> Power {
    let ion_mass = molar_mass.get() / N_AVOGADRO;
    let ions_per_second = ion_flow.get() / ion_mass;
    Power::new(ions_per_second * epsilon_ev * ELEMENTARY_CHARGE)
}

/// Total electrical power the thruster draws, before power processing.
pub fn thruster_input_power(jet: Power, ionisation: Power, other_losses: Ratio) -> Power {
    Power::new((jet.get() + ionisation.get()) / (1.0 - other_losses.get()))
}

/// Power drawn from the bus, after the power processing unit's own efficiency.
pub fn bus_power_demand(thruster_input: Power, ppu_efficiency: Ratio) -> Power {
    Power::new(thruster_input.get() / ppu_efficiency.get())
}

/// Total system efficiency, jet power over bus power. For an ABEP system this
/// is typically a few per cent, and stating it plainly is the difference
/// between a feasibility study and a brochure.
pub fn total_efficiency(jet: Power, bus_power: Power) -> Ratio {
    Ratio::new(jet.get() / bus_power.get())
}

/// Specific impulse, referred to the **collected** flow rather than the ionised
/// flow.
///
/// This is the honest denominator. Referring `Isp` to the ion flow alone flatters
/// the number by the reciprocal of the propellant utilisation, and an ABEP
/// system's utilisation is not close to one.
pub fn specific_impulse(thrust: Force, collected_flow: MassFlow) -> Time {
    Time::new(thrust.get() / (collected_flow.get() * G0.get()))
}

/// Thrust-to-drag ratio — the closure quantity for the whole design.
///
/// At or above one the orbit holds without stored propellant. Below one the
/// mission has a lifetime rather than an altitude, and every other number in
/// the design is a detail.
pub fn thrust_to_drag(thrust: Force, drag: Force) -> Ratio {
    Ratio::new(thrust.get() / drag.get())
}

/// The intake area that would be needed for thrust to equal drag.
///
/// The inverse question, and the one a concept trade actually asks: not "does
/// this close" but "how big would the mouth have to be". Returns `None` when no
/// area closes it — which happens whenever the thrust per unit collected flow
/// is below the drag per unit intake area, and no amount of mouth fixes that
/// because a bigger mouth is also more drag.
pub fn closing_intake_area(
    thrust_per_intake_area: f64,
    drag_excluding_intake: Force,
    drag_coefficient_intake: f64,
    dynamic_pressure: Pressure,
) -> Option<Area> {
    let drag_per_area = drag_coefficient_intake * dynamic_pressure.get();
    if thrust_per_intake_area <= drag_per_area {
        return None;
    }
    Some(Area::new(
        drag_excluding_intake.get() / (thrust_per_intake_area - drag_per_area),
    ))
}

/// Equivalent stored-propellant mass an air-breathing system saves over a
/// mission — the commercial argument, quantified.
pub fn equivalent_stored_propellant(
    thrust: Force,
    duration: Time,
    reference_isp: Time,
) -> Mass {
    Mass::new(thrust.get() * duration.get() / (reference_isp.get() * G0.get()))
}
