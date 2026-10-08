//! Kernel functions a method may call by name.
//!
//! Some relations are not a formula a person writes in a line: the
//! thermosphere's composition is a five-species integral up the altitude, an
//! orbit's decay time is a quadrature over that density, the solar-cycle
//! analogue is an integral over a stacked cycle. They already live in
//! `vleo-core::physics`, reviewed once, and a method calls them by name — rule
//! 3: every formula lives in the kernel and nowhere else.
//!
//! Each entry says the unit of every argument and of the answer, so the
//! checker holds a call to them exactly as it holds `sqrt`, and the
//! interpreter runs the kernel itself, so a method and the kernel cannot
//! disagree. A function is added here only as a reviewed change:
//! it is the language growing.

use vleo_core::physics::{
    aero, comms, cost, env, gnc, mass, mission, orbit, payload, power, prop, thermal,
};
use vleo_units::*;

pub struct KernelFn {
    pub name: &'static str,
    /// Each argument's name and unit, as written in brackets.
    pub args: &'static [(&'static str, &'static str)],
    /// The answer's unit.
    pub out: &'static str,
    pub meaning: &'static str,
    /// Where it lives in the kernel, for the reader.
    pub kernel: &'static str,
    /// The interpreter's evaluation: every value in SI.
    pub eval: fn(&[f64]) -> f64,
    /// Why the kernel gives no answer, for a function that can refuse: its
    /// evaluation is then not a number, and the method refuses with these
    /// words. Empty for a function that always answers.
    pub refuses: &'static str,
}

fn o(a: &[f64]) -> f64 {
    env::composition(Length::new(a[0]), Temperature::new(a[1]))
        .o
        .get()
}
fn total(a: &[f64]) -> f64 {
    env::composition(Length::new(a[0]), Temperature::new(a[1]))
        .total()
        .get()
}
fn molar(a: &[f64]) -> f64 {
    env::composition(Length::new(a[0]), Temperature::new(a[1]))
        .mean_molar_mass()
        .get()
}
fn density(a: &[f64]) -> f64 {
    env::mass_density(Length::new(a[0]), Temperature::new(a[1])).get()
}
fn scale(a: &[f64]) -> f64 {
    env::scale_height(Length::new(a[0]), Temperature::new(a[1])).get()
}
fn decay(a: &[f64]) -> f64 {
    let t_inf = Temperature::new(a[3]);
    orbit::lifetime_estimate(Length::new(a[0]), Length::new(a[1]), a[2], 64, |z| {
        env::mass_density(z, t_inf)
    })
    .get()
}
const DAY: f64 = 86_400.0;
fn cycle_mean(a: &[f64]) -> f64 {
    env::solar_cycle_analogue_mean(a[0] / DAY, a[1] / DAY)
}
fn cycle_max(a: &[f64]) -> f64 {
    env::solar_cycle_analogue_max(a[0] / DAY, a[1] / DAY)
}

fn kp(a: &[f64]) -> f64 {
    env::kp_from_ap(a[0])
}
fn kp_mean_bias(a: &[f64]) -> f64 {
    env::kp_mean_slot_bias(a[0])
}
fn kp_peak_bias(a: &[f64]) -> f64 {
    env::kp_peak_slot_bias(a[0])
}

/// The unit an argument written `~` takes: any, so long as every `~`
/// argument of the call is in the same one. A closure's required and achieved
/// values are a time on one row and a mass on the next.
pub const ANY: &str = "~";

/// A quantity's unit as the method language writes it.
const fn unit_of(u: Unit) -> &'static str {
    match u {
        Unit::One => "1",
        _ => u.symbol(),
    }
}

const H_T: &[(&str, &str)] = &[("h", "m"), ("T_inf", "K")];
const AP: &[(&str, &str)] = &[("ap", "1")];

#[rustfmt::skip]
pub const KERNEL_FUNCTIONS: &[KernelFn] = &[
    KernelFn { name: "thermosphere_o", args: H_T, out: "1/m^3", eval: o, refuses: "",
        meaning: "atomic oxygen number density at altitude h, for exospheric temperature T_inf",
        kernel: "env::composition(h, T_inf).o" },
    KernelFn { name: "thermosphere_number_density", args: H_T, out: "1/m^3", eval: total, refuses: "",
        meaning: "every species' number density at altitude h, summed",
        kernel: "env::composition(h, T_inf).total()" },
    KernelFn { name: "thermosphere_molar_mass", args: H_T, out: "kg/mol", eval: molar, refuses: "",
        meaning: "the mean molar mass of the gas at altitude h",
        kernel: "env::composition(h, T_inf).mean_molar_mass()" },
    KernelFn { name: "thermosphere_density", args: H_T, out: "kg/m^3", eval: density, refuses: "",
        meaning: "the mass density of the gas at altitude h",
        kernel: "env::mass_density(h, T_inf)" },
    KernelFn { name: "thermosphere_scale_height", args: H_T, out: "m", eval: scale, refuses: "",
        meaning: "the density scale height at altitude h",
        kernel: "env::scale_height(h, T_inf)" },
    KernelFn { name: "orbit_decay_time", args: &[("h", "m"), ("h_end", "m"), ("bc", "kg/m^2"), ("T_inf", "K")], out: "s", eval: decay, refuses: "",
        meaning: "the time a circular orbit takes to decay from h to h_end, for ballistic coefficient bc, in the thermosphere of T_inf (64 steps)",
        kernel: "orbit::lifetime_estimate(h, h_end, bc, 64, |z| env::mass_density(z, T_inf))" },
    KernelFn { name: "solar_cycle_mean", args: &[("t0", "s"), ("t1", "s")], out: "1", eval: cycle_mean, refuses: "",
        meaning: "the solar-cycle analogue's mean F10.7 from mission time t0 to t1",
        kernel: "env::solar_cycle_analogue_mean(t0, t1), in days" },
    KernelFn { name: "solar_cycle_max", args: &[("t0", "s"), ("t1", "s")], out: "1", eval: cycle_max, refuses: "",
        meaning: "the solar-cycle analogue's highest F10.7 from mission time t0 to t1",
        kernel: "env::solar_cycle_analogue_max(t0, t1), in days" },
    KernelFn { name: "kp_from_ap", args: AP, out: "1", eval: kp, refuses: "",
        meaning: "Kp on the published three-hour scale for the planetary index ap, between its tabulated thirds",
        kernel: "env::kp_from_ap(ap)" },
    KernelFn { name: "kp_mean_slot_bias", args: AP, out: "1", eval: kp_mean_bias, refuses: "",
        meaning: "the measured offset of a day's mean three-hour Kp from the Kp of its daily Ap",
        kernel: "env::kp_mean_slot_bias(ap)" },
    KernelFn { name: "kp_peak_slot_bias", args: AP, out: "1", eval: kp_peak_bias, refuses: "",
        meaning: "the measured offset of a day's highest three-hour Kp from the Kp of its daily Ap",
        kernel: "env::kp_peak_slot_bias(ap)" },
    // The toolbox the built-in relations called (docs/PLAN_1_0.md, phase E):
    // opened so a node's method says which relation it uses, with which
    // inputs and constants, and the relation itself stays here.
    KernelFn { name: "aerodynamic_torque", args: &[("drag", unit_of(Force::UNIT)), ("cp_cm_offset", unit_of(Length::UNIT))], out: unit_of(Torque::UNIT), eval: t_aerodynamic_torque, refuses: "",
        meaning: "the torque drag makes about the centre of mass, from the centre-of-pressure offset",
        kernel: "aero::aerodynamic_torque(drag, cp_cm_offset)" },
    KernelFn { name: "drag_acceleration", args: &[("drag", unit_of(Force::UNIT)), ("mass", unit_of(Mass::UNIT))], out: unit_of(Acceleration::UNIT), eval: t_drag_acceleration, refuses: "",
        meaning: "drag deceleration, D/m",
        kernel: "aero::drag_acceleration(drag, mass)" },
    KernelFn { name: "dynamic_pressure", args: &[("density", unit_of(MassDensity::UNIT)), ("velocity", unit_of(Velocity::UNIT))], out: unit_of(Pressure::UNIT), eval: t_dynamic_pressure, refuses: "",
        meaning: "dynamic pressure, 0.5·rho·V^2",
        kernel: "aero::dynamic_pressure(density, velocity)" },
    KernelFn { name: "speed_ratio", args: &[("velocity", unit_of(Velocity::UNIT)), ("t", unit_of(Temperature::UNIT)), ("molar_mass", unit_of(MolarMass::UNIT))], out: unit_of(Ratio::UNIT), eval: t_speed_ratio, refuses: "",
        meaning: "molecular speed ratio, V over the most probable thermal speed",
        kernel: "aero::speed_ratio(velocity, t, molar_mass)" },
    KernelFn { name: "accommodation_coefficient", args: &[("n_atomic_oxygen", unit_of(NumberDensity::UNIT)), ("t_inf", unit_of(Temperature::UNIT))], out: "1", eval: t_accommodation_coefficient, refuses: "",
        meaning: "energy accommodation coefficient from Langmuir adsorption of atomic oxygen (Moe & Moe 2005)",
        kernel: "aero::accommodation_coefficient(n_atomic_oxygen, t_inf)" },
    KernelFn { name: "atomic_oxygen_fluence", args: &[("n_atomic_oxygen", unit_of(NumberDensity::UNIT)), ("velocity", unit_of(Velocity::UNIT)), ("duration", unit_of(Time::UNIT))], out: "1/m^2", eval: t_atomic_oxygen_fluence, refuses: "",
        meaning: "atomic oxygen fluence over a duration, atoms per square metre",
        kernel: "aero::atomic_oxygen_fluence(n_atomic_oxygen, velocity, duration)" },
    KernelFn { name: "ballistic_coefficient", args: &[("mass", unit_of(Mass::UNIT)), ("drag_coefficient", "1"), ("reference_area", unit_of(Area::UNIT))], out: "kg/m^2", eval: t_ballistic_coefficient, refuses: "",
        meaning: "ballistic coefficient, m/(Cd·A)",
        kernel: "aero::ballistic_coefficient(mass, drag_coefficient, reference_area)" },
    KernelFn { name: "cylinder_drag_coefficient", args: &[("s", "1"), ("length", unit_of(Length::UNIT)), ("diameter", unit_of(Length::UNIT)), ("alpha", "1"), ("t_wall", unit_of(Temperature::UNIT)), ("velocity", unit_of(Velocity::UNIT)), ("molar_mass", unit_of(MolarMass::UNIT))], out: "1", eval: t_cylinder_drag_coefficient, refuses: "",
        meaning: "drag coefficient of a right circular cylinder flying axially, Sentman free-molecular, for speed ratio s and accommodation alpha",
        kernel: "aero::cylinder_drag_coefficient(s, length, diameter, alpha, t_wall, velocity, molar_mass)" },
    KernelFn { name: "aperture_gain_db", args: &[("diameter", unit_of(Length::UNIT)), ("frequency", unit_of(Frequency::UNIT)), ("efficiency", unit_of(Ratio::UNIT))], out: "1", eval: t_aperture_gain_db, refuses: "",
        meaning: "gain of a circular aperture antenna, in dBi as a pure number",
        kernel: "comms::aperture_gain_db(diameter, frequency, efficiency)" },
    KernelFn { name: "carrier_to_noise_density_db", args: &[("eirp_dbw", "1"), ("path_loss_db", "1"), ("atmospheric_loss_db", "1"), ("g_over_t_db", "1")], out: "1", eval: t_carrier_to_noise_density_db, refuses: "",
        meaning: "carrier to noise density ratio, in dB-Hz as a pure number; every argument in dB as a pure number",
        kernel: "comms::carrier_to_noise_density_db(eirp_dbw, path_loss_db, atmospheric_loss_db, g_over_t_db)" },
    KernelFn { name: "max_contact_time", args: &[("earth_central_angle", unit_of(Angle::UNIT)), ("orbital_period", unit_of(Time::UNIT))], out: unit_of(Time::UNIT), eval: t_max_contact_time, refuses: "",
        meaning: "contact time for one overhead pass",
        kernel: "comms::max_contact_time(earth_central_angle, orbital_period)" },
    KernelFn { name: "daily_downlink_volume", args: &[("per_pass", unit_of(DataVolume::UNIT)), ("passes_per_day", "1"), ("availability", unit_of(Ratio::UNIT))], out: unit_of(DataVolume::UNIT), eval: t_daily_downlink_volume, refuses: "",
        meaning: "data downlinked in a day, from the volume per pass and the passes per day",
        kernel: "comms::daily_downlink_volume(per_pass, passes_per_day, availability)" },
    KernelFn { name: "achievable_data_rate", args: &[("cn0_db", "1"), ("required_eb_n0_db", "1"), ("implementation_loss_db", "1"), ("margin_db", "1")], out: unit_of(DataRate::UNIT), eval: t_achievable_data_rate, refuses: "",
        meaning: "the highest data rate that closes the link at the stated margin; every argument in dB as a pure number",
        kernel: "comms::achievable_data_rate(cn0_db, required_eb_n0_db, implementation_loss_db, margin_db)" },
    KernelFn { name: "doppler_shift", args: &[("frequency", unit_of(Frequency::UNIT)), ("relative_velocity", unit_of(Velocity::UNIT))], out: unit_of(Frequency::UNIT), eval: t_doppler_shift, refuses: "",
        meaning: "Doppler shift at closest approach, f·v/c",
        kernel: "comms::doppler_shift(frequency, relative_velocity)" },
    KernelFn { name: "eb_over_n0_db", args: &[("cn0_db", "1"), ("data_rate", unit_of(DataRate::UNIT))], out: "1", eval: t_eb_over_n0_db, refuses: "",
        meaning: "energy per bit over noise density, in dB as a pure number",
        kernel: "comms::eb_over_n0_db(cn0_db, data_rate)" },
    KernelFn { name: "eirp_dbw", args: &[("transmit_power", unit_of(Power::UNIT)), ("antenna_gain_db", "1"), ("line_loss_db", "1")], out: "1", eval: t_eirp_dbw, refuses: "",
        meaning: "effective isotropic radiated power, in dBW as a pure number",
        kernel: "comms::eirp_dbw(transmit_power, antenna_gain_db, line_loss_db)" },
    KernelFn { name: "g_over_t_db", args: &[("receive_gain_db", "1"), ("system_noise_temperature", unit_of(Temperature::UNIT))], out: "1", eval: t_g_over_t_db, refuses: "",
        meaning: "receiver figure of merit G/T, in dB/K as a pure number",
        kernel: "comms::g_over_t_db(receive_gain_db, system_noise_temperature)" },
    KernelFn { name: "pass_data_volume", args: &[("rate", unit_of(DataRate::UNIT)), ("contact", unit_of(Time::UNIT)), ("link_efficiency", unit_of(Ratio::UNIT))], out: unit_of(DataVolume::UNIT), eval: t_pass_data_volume, refuses: "",
        meaning: "data volume downlinked in one pass",
        kernel: "comms::pass_data_volume(rate, contact, link_efficiency)" },
    KernelFn { name: "free_space_path_loss_db", args: &[("range", unit_of(Length::UNIT)), ("frequency", unit_of(Frequency::UNIT))], out: "1", eval: t_free_space_path_loss_db, refuses: "",
        meaning: "free-space path loss, in dB as a pure number",
        kernel: "comms::free_space_path_loss_db(range, frequency)" },
    KernelFn { name: "system_noise_temperature", args: &[("antenna_temperature", unit_of(Temperature::UNIT)), ("line_loss_db", "1"), ("receiver_noise_figure_db", "1"), ("physical_temperature", unit_of(Temperature::UNIT))], out: unit_of(Temperature::UNIT), eval: t_system_noise_temperature, refuses: "",
        meaning: "system noise temperature from the antenna, line and receiver; losses in dB as pure numbers",
        kernel: "comms::system_noise_temperature(antenna_temperature, line_loss_db, receiver_noise_figure_db, physical_temperature)" },
    KernelFn { name: "link_margin_db", args: &[("eb_n0_db", "1"), ("required_eb_n0_db", "1"), ("implementation_loss_db", "1")], out: "1", eval: t_link_margin_db, refuses: "",
        meaning: "link margin against a required Eb/N0, with implementation loss; every value in dB as a pure number",
        kernel: "comms::link_margin_db(eb_n0_db, required_eb_n0_db, implementation_loss_db)" },
    KernelFn { name: "cost_per_service_unit", args: &[("programme_cost", unit_of(Money::UNIT)), ("service_units", "1")], out: unit_of(Money::UNIT), eval: t_cost_per_service_unit, refuses: "",
        meaning: "cost per unit of delivered service",
        kernel: "cost::cost_per_service_unit(programme_cost, service_units)" },
    KernelFn { name: "production_run_cost", args: &[("first_unit", unit_of(Money::UNIT)), ("units", "1"), ("slope", "1")], out: unit_of(Money::UNIT), eval: t_production_run_cost, refuses: "",
        meaning: "a production run's cost under Wright's learning curve, summed unit by unit; units a whole number, cut toward zero",
        kernel: "cost::production_run_cost(first_unit, units, slope)" },
    KernelFn { name: "programme_cost", args: &[("non_recurring", unit_of(Money::UNIT)), ("production", unit_of(Money::UNIT)), ("annual_operations", unit_of(Money::UNIT)), ("years", "1")], out: unit_of(Money::UNIT), eval: t_programme_cost, refuses: "",
        meaning: "non-recurring plus production plus operations for the years given",
        kernel: "cost::programme_cost(non_recurring, production, annual_operations, years)" },
    KernelFn { name: "replacement_avoided", args: &[("satellite_unit_cost", unit_of(Money::UNIT)), ("launch_cost", unit_of(Money::UNIT)), ("replacements_avoided", "1")], out: unit_of(Money::UNIT), eval: t_replacement_avoided, refuses: "",
        meaning: "the saving of satellites that need not be replaced and relaunched",
        kernel: "cost::replacement_avoided(satellite_unit_cost, launch_cost, replacements_avoided)" },
    KernelFn { name: "cer_inflated", args: &[("driver", ANY), ("a", "1"), ("b", "1"), ("base_year", "1"), ("target_year", "1"), ("annual_inflation", "1")], out: unit_of(Money::UNIT), eval: t_cer_inflated, refuses: "",
        meaning: "a cost estimating relationship a·driver^b in millions, in its base year's money, inflated to the target year at the annual rate; years whole numbers",
        kernel: "cost::Cer { a, b, base_year, .. }.evaluate_inflated(driver, target_year, annual_inflation)" },
    KernelFn { name: "density_uncertainty", args: &[("kp", "1")], out: unit_of(Ratio::UNIT), eval: t_density_uncertainty, refuses: "",
        meaning: "the one-sigma uncertainty the density model claims, as a fraction",
        kernel: "env::density_uncertainty(kp)" },
    KernelFn { name: "exospheric_temperature", args: &[("f107", "1"), ("f107a", "1"), ("kp", "1")], out: unit_of(Temperature::UNIT), eval: t_exospheric_temperature, refuses: "",
        meaning: "exospheric temperature from F10.7, its 81-day mean and Kp (Jacchia 1971)",
        kernel: "env::exospheric_temperature(f107, f107a, kp)" },
    KernelFn { name: "knudsen", args: &[("mean_free_path", unit_of(Length::UNIT)), ("characteristic_length", unit_of(Length::UNIT))], out: unit_of(Ratio::UNIT), eval: t_knudsen, refuses: "",
        meaning: "Knudsen number, lambda/L",
        kernel: "env::knudsen(mean_free_path, characteristic_length)" },
    KernelFn { name: "thermosphere_temperature", args: &[("altitude", unit_of(Length::UNIT)), ("t_inf", unit_of(Temperature::UNIT))], out: unit_of(Temperature::UNIT), eval: t_thermosphere_temperature, refuses: "",
        meaning: "the gas's kinetic temperature at altitude, Bates profile",
        kernel: "env::temperature(altitude, t_inf)" },
    KernelFn { name: "magnetic_field", args: &[("r", unit_of(Length::UNIT)), ("magnetic_latitude", unit_of(Angle::UNIT))], out: unit_of(MagneticFluxDensity::UNIT), eval: t_magnetic_field, refuses: "",
        meaning: "the geomagnetic field's magnitude, dipole approximation",
        kernel: "env::magnetic_field(r, magnetic_latitude)" },
    KernelFn { name: "mean_free_path", args: &[("n", unit_of(NumberDensity::UNIT))], out: unit_of(Length::UNIT), eval: t_mean_free_path, refuses: "",
        meaning: "mean free path for a number density",
        kernel: "env::mean_free_path(n)" },
    KernelFn { name: "along_track_error_from_drag", args: &[("drag_acceleration_error", unit_of(Acceleration::UNIT)), ("duration", unit_of(Time::UNIT))], out: unit_of(Length::UNIT), eval: t_along_track_error_from_drag, refuses: "",
        meaning: "along-track error grown from an unmodelled drag acceleration over a duration",
        kernel: "gnc::along_track_error_from_drag(drag_acceleration_error, duration)" },
    KernelFn { name: "gravity_gradient_torque", args: &[("radius", unit_of(Length::UNIT)), ("inertia_max", "kg.m^2"), ("inertia_min", "kg.m^2"), ("theta", unit_of(Angle::UNIT))], out: unit_of(Torque::UNIT), eval: t_gravity_gradient_torque, refuses: "",
        meaning: "gravity-gradient torque at pitch angle theta from local vertical",
        kernel: "gnc::gravity_gradient_torque(radius, inertia_max, inertia_min, theta)" },
    KernelFn { name: "pointing_to_ground_error", args: &[("pointing_error", unit_of(Angle::UNIT)), ("slant_range", unit_of(Length::UNIT))], out: unit_of(Length::UNIT), eval: t_pointing_to_ground_error, refuses: "",
        meaning: "ground position error from a pointing error at a slant range",
        kernel: "gnc::pointing_to_ground_error(pointing_error, slant_range)" },
    KernelFn { name: "magnetic_torque", args: &[("residual_dipole", unit_of(DipoleMoment::UNIT)), ("field", unit_of(MagneticFluxDensity::UNIT))], out: unit_of(Torque::UNIT), eval: t_magnetic_torque, refuses: "",
        meaning: "torque from a residual dipole in the geomagnetic field",
        kernel: "gnc::magnetic_torque(residual_dipole, field)" },
    KernelFn { name: "magnetorquer_dipole_required", args: &[("momentum", unit_of(AngularMomentum::UNIT)), ("field", unit_of(MagneticFluxDensity::UNIT)), ("dump_time", unit_of(Time::UNIT))], out: unit_of(DipoleMoment::UNIT), eval: t_magnetorquer_dipole_required, refuses: "",
        meaning: "the dipole a torque rod needs to dump a momentum in a time",
        kernel: "gnc::magnetorquer_dipole_required(momentum, field, dump_time)" },
    KernelFn { name: "momentum_storage_required", args: &[("secular_torque", unit_of(Torque::UNIT)), ("orbital_period", unit_of(Time::UNIT)), ("margin", unit_of(Ratio::UNIT))], out: unit_of(AngularMomentum::UNIT), eval: t_momentum_storage_required, refuses: "",
        meaning: "momentum a wheel stores against a secular torque for half an orbit, with a margin",
        kernel: "gnc::momentum_storage_required(secular_torque, orbital_period, margin)" },
    KernelFn { name: "navigation_position_error", args: &[("user_range_error", unit_of(Length::UNIT)), ("gdop", "1")], out: unit_of(Length::UNIT), eval: t_navigation_position_error, refuses: "",
        meaning: "GNSS position error from the receiver's range error (URE) and the geometry",
        kernel: "gnc::navigation_position_error(user_range_error, gdop)" },
    KernelFn { name: "pointing_error_rss", args: &[("a", unit_of(Angle::UNIT)), ("b", unit_of(Angle::UNIT)), ("c", unit_of(Angle::UNIT)), ("d", unit_of(Angle::UNIT))], out: unit_of(Angle::UNIT), eval: t_pointing_error_rss, refuses: "",
        meaning: "root-sum-square of four pointing error terms",
        kernel: "gnc::pointing_error_rss(&[a, b, c, d])" },
    KernelFn { name: "solar_pressure_torque", args: &[("area", unit_of(Area::UNIT)), ("reflectivity", unit_of(Ratio::UNIT)), ("cp_offset", unit_of(Length::UNIT)), ("incidence", unit_of(Angle::UNIT))], out: unit_of(Torque::UNIT), eval: t_solar_pressure_torque, refuses: "",
        meaning: "solar radiation pressure torque",
        kernel: "gnc::solar_pressure_torque(area, reflectivity, cp_offset, incidence)" },
    KernelFn { name: "total_disturbance_torque", args: &[("aerodynamic", unit_of(Torque::UNIT)), ("gravity_gradient", unit_of(Torque::UNIT)), ("solar", unit_of(Torque::UNIT)), ("magnetic", unit_of(Torque::UNIT))], out: unit_of(Torque::UNIT), eval: t_total_disturbance_torque, refuses: "",
        meaning: "the disturbance torques summed, worst case",
        kernel: "gnc::total_disturbance_torque(aerodynamic, gravity_gradient, solar, magnetic)" },
    KernelFn { name: "dry_mass", args: &[("structure", unit_of(Mass::UNIT)), ("propulsion", unit_of(Mass::UNIT)), ("power", unit_of(Mass::UNIT)), ("thermal", unit_of(Mass::UNIT)), ("avionics", unit_of(Mass::UNIT)), ("comms", unit_of(Mass::UNIT)), ("gnc", unit_of(Mass::UNIT)), ("harness", unit_of(Mass::UNIT)), ("payloads", unit_of(Mass::UNIT)), ("system_margin", unit_of(Ratio::UNIT))], out: unit_of(Mass::UNIT), eval: t_dry_mass, refuses: "",
        meaning: "dry mass: the subsystems summed, with the system margin",
        kernel: "mass::dry_mass(structure, propulsion, power, thermal, avionics, comms, gnc, harness, payloads, system_margin)" },
    KernelFn { name: "mass_margin", args: &[("limit", unit_of(Mass::UNIT)), ("actual", unit_of(Mass::UNIT))], out: unit_of(Ratio::UNIT), eval: t_mass_margin, refuses: "",
        meaning: "mass margin against a limit, as a fraction of the limit",
        kernel: "mass::mass_margin(limit, actual)" },
    KernelFn { name: "wet_mass", args: &[("dry", unit_of(Mass::UNIT)), ("propellant", unit_of(Mass::UNIT))], out: unit_of(Mass::UNIT), eval: t_wet_mass, refuses: "",
        meaning: "dry plus propellant",
        kernel: "mass::wet_mass(dry, propellant)" },
    KernelFn { name: "margin_at_least", args: &[("required", ANY), ("achieved", ANY)], out: "1", eval: t_margin_at_least, refuses: "",
        meaning: "signed margin of an achieved value that must be AT LEAST the required one, as a fraction of it: (achieved - required) / required, and 0 when nothing is required",
        kernel: "mission::closure(required, achieved, AtLeast).margin" },
    KernelFn { name: "margin_at_most", args: &[("required", ANY), ("achieved", ANY)], out: "1", eval: t_margin_at_most, refuses: "",
        meaning: "signed margin of an achieved value that must be AT MOST the required one, as a fraction of it: (required - achieved) / required, and 0 when nothing is required",
        kernel: "mission::closure(required, achieved, AtMost).margin" },
    KernelFn { name: "access_area", args: &[("earth_central_angle", unit_of(Angle::UNIT))], out: unit_of(Area::UNIT), eval: t_access_area, refuses: "",
        meaning: "instantaneous access area on the ground",
        kernel: "mission::access_area(earth_central_angle)" },
    KernelFn { name: "k_of_n_availability", args: &[("unit_availability", unit_of(Ratio::UNIT)), ("n", "1"), ("k", "1")], out: unit_of(Ratio::UNIT), eval: t_k_of_n_availability, refuses: "",
        meaning: "availability of k of n, binomial; n and k whole numbers, cut toward zero",
        kernel: "mission::k_of_n_availability(unit_availability, n, k)" },
    KernelFn { name: "instantaneous_coverage_fraction", args: &[("earth_central_angle", unit_of(Angle::UNIT))], out: unit_of(Ratio::UNIT), eval: t_instantaneous_coverage_fraction, refuses: "",
        meaning: "fraction of the Earth one satellite sees at an instant",
        kernel: "mission::instantaneous_coverage_fraction(earth_central_angle)" },
    KernelFn { name: "end_to_end_latency", args: &[("time_to_downlink", unit_of(Time::UNIT)), ("downlink_duration", unit_of(Time::UNIT)), ("ground_processing", unit_of(Time::UNIT)), ("delivery", unit_of(Time::UNIT))], out: unit_of(Time::UNIT), eval: t_end_to_end_latency, refuses: "",
        meaning: "collect, hold, downlink, process and deliver",
        kernel: "mission::end_to_end_latency(time_to_downlink, downlink_duration, ground_processing, delivery)" },
    KernelFn { name: "mean_revisit_time", args: &[("swath_width", unit_of(Length::UNIT)), ("ground_track_speed", unit_of(Velocity::UNIT)), ("satellites", "1")], out: unit_of(Time::UNIT), eval: t_mean_revisit_time, refuses: "",
        meaning: "mean revisit time by an area-rate argument",
        kernel: "mission::mean_revisit_time(swath_width, ground_track_speed, satellites)" },
    KernelFn { name: "satellites_for_revisit", args: &[("target_revisit", unit_of(Time::UNIT)), ("swath_width", unit_of(Length::UNIT)), ("ground_track_speed", unit_of(Velocity::UNIT))], out: "1", eval: t_satellites_for_revisit, refuses: "",
        meaning: "satellites needed for a revisit target",
        kernel: "mission::satellites_for_revisit(target_revisit, swath_width, ground_track_speed)" },
    KernelFn { name: "mean_time_to_downlink", args: &[("passes_per_day", "1")], out: unit_of(Time::UNIT), eval: t_mean_time_to_downlink, refuses: "",
        meaning: "mean time to the next ground contact",
        kernel: "mission::mean_time_to_downlink(passes_per_day)" },
    KernelFn { name: "circular_velocity", args: &[("radius", unit_of(Length::UNIT))], out: unit_of(Velocity::UNIT), eval: t_circular_velocity, refuses: "",
        meaning: "circular orbital speed, sqrt(mu/r)",
        kernel: "orbit::circular_velocity(radius)" },
    KernelFn { name: "decay_rate", args: &[("density", unit_of(MassDensity::UNIT)), ("ballistic_coefficient", "kg/m^2"), ("semi_major_axis", unit_of(Length::UNIT))], out: unit_of(Velocity::UNIT), eval: t_decay_rate, refuses: "",
        meaning: "rate of change of semi-major axis under drag",
        kernel: "orbit::decay_rate(density, ballistic_coefficient, semi_major_axis)" },
    KernelFn { name: "deorbit_delta_v", args: &[("from_altitude", unit_of(Length::UNIT)), ("target_perigee_altitude", unit_of(Length::UNIT))], out: unit_of(Velocity::UNIT), eval: t_deorbit_delta_v, refuses: "",
        meaning: "delta-v to lower perigee to a re-entry altitude",
        kernel: "orbit::deorbit_delta_v(from_altitude, target_perigee_altitude)" },
    KernelFn { name: "drag_makeup_delta_v", args: &[("drag_acceleration", unit_of(Acceleration::UNIT)), ("duration", unit_of(Time::UNIT))], out: unit_of(Velocity::UNIT), eval: t_drag_makeup_delta_v, refuses: "",
        meaning: "delta-v to hold altitude against drag for a duration",
        kernel: "orbit::drag_makeup_delta_v(drag_acceleration, duration)" },
    KernelFn { name: "earth_central_angle", args: &[("radius", unit_of(Length::UNIT)), ("min_elevation", unit_of(Angle::UNIT))], out: unit_of(Angle::UNIT), eval: t_earth_central_angle, refuses: "",
        meaning: "Earth-central angle to the horizon at a minimum elevation",
        kernel: "orbit::earth_central_angle(radius, min_elevation)" },
    KernelFn { name: "eclipse_fraction", args: &[("radius", unit_of(Length::UNIT)), ("beta", unit_of(Angle::UNIT))], out: unit_of(Ratio::UNIT), eval: t_eclipse_fraction, refuses: "",
        meaning: "fraction of an orbit in the Earth's shadow, cylindrical model",
        kernel: "orbit::eclipse_fraction(radius, beta)" },
    KernelFn { name: "ground_track_speed", args: &[("orbital_speed", unit_of(Velocity::UNIT)), ("radius", unit_of(Length::UNIT))], out: unit_of(Velocity::UNIT), eval: t_ground_track_speed, refuses: "",
        meaning: "speed of the sub-satellite point",
        kernel: "orbit::ground_track_speed(orbital_speed, radius)" },
    KernelFn { name: "nodal_regression", args: &[("semi_major_axis", unit_of(Length::UNIT)), ("eccentricity", "1"), ("inclination", unit_of(Angle::UNIT))], out: unit_of(AngularRate::UNIT), eval: t_nodal_regression, refuses: "",
        meaning: "secular regression of the node from J2",
        kernel: "orbit::nodal_regression(semi_major_axis, eccentricity, inclination)" },
    KernelFn { name: "orbital_period", args: &[("semi_major_axis", unit_of(Length::UNIT))], out: unit_of(Time::UNIT), eval: t_orbital_period, refuses: "",
        meaning: "orbital period, 2·pi·sqrt(a^3/mu)",
        kernel: "orbit::period(semi_major_axis)" },
    KernelFn { name: "orbit_radius", args: &[("altitude", unit_of(Length::UNIT))], out: unit_of(Length::UNIT), eval: t_orbit_radius, refuses: "",
        meaning: "geocentric radius from altitude above the WGS-84 equatorial radius",
        kernel: "orbit::radius(altitude)" },
    KernelFn { name: "slant_range", args: &[("radius", unit_of(Length::UNIT)), ("min_elevation", unit_of(Angle::UNIT))], out: unit_of(Length::UNIT), eval: t_slant_range, refuses: "",
        meaning: "slant range to the edge of the access circle",
        kernel: "orbit::slant_range(radius, min_elevation)" },
    KernelFn { name: "sun_synchronous_inclination", args: &[("semi_major_axis", unit_of(Length::UNIT)), ("eccentricity", "1")], out: unit_of(Angle::UNIT), eval: t_sun_synchronous_inclination, refuses: "no inclination gives Sun-synchronous regression at this radius",
        meaning: "the inclination at which the node regresses with the Sun; refused where none does",
        kernel: "orbit::sun_synchronous_inclination(semi_major_axis, eccentricity)" },
    KernelFn { name: "swath_width", args: &[("earth_central_angle", unit_of(Angle::UNIT))], out: unit_of(Length::UNIT), eval: t_swath_width, refuses: "",
        meaning: "ground swath width for an Earth-central half-angle",
        kernel: "orbit::swath_width(earth_central_angle)" },
    KernelFn { name: "propellant_mass", args: &[("dry_mass", unit_of(Mass::UNIT)), ("delta_v", unit_of(Velocity::UNIT)), ("exhaust_velocity", unit_of(Velocity::UNIT))], out: unit_of(Mass::UNIT), eval: t_propellant_mass, refuses: "",
        meaning: "the rocket equation, solved for propellant mass",
        kernel: "orbit::propellant_mass(dry_mass, delta_v, exhaust_velocity)" },
    KernelFn { name: "exhaust_velocity", args: &[("specific_impulse", unit_of(Time::UNIT))], out: unit_of(Velocity::UNIT), eval: t_exhaust_velocity, refuses: "",
        meaning: "exhaust velocity from specific impulse, Isp·g0",
        kernel: "orbit::exhaust_velocity(specific_impulse)" },
    KernelFn { name: "dwell_time", args: &[("gsd", unit_of(Length::UNIT)), ("ground_track_speed", unit_of(Velocity::UNIT))], out: unit_of(Time::UNIT), eval: t_dwell_time, refuses: "",
        meaning: "integration time per ground sample",
        kernel: "payload::dwell_time(gsd, ground_track_speed)" },
    KernelFn { name: "tdoa_geolocation_error", args: &[("timing_uncertainty", unit_of(Time::UNIT)), ("gdop", "1")], out: unit_of(Length::UNIT), eval: t_tdoa_geolocation_error, refuses: "",
        meaning: "geolocation error from time difference of arrival",
        kernel: "payload::tdoa_geolocation_error(timing_uncertainty, gdop)" },
    KernelFn { name: "achieved_gsd", args: &[("diffraction", unit_of(Length::UNIT)), ("detector", unit_of(Length::UNIT))], out: unit_of(Length::UNIT), eval: t_achieved_gsd, refuses: "",
        meaning: "the ground sample distance achieved: the worse of the two limits",
        kernel: "payload::achieved_gsd(diffraction, detector)" },
    KernelFn { name: "detector_limited_gsd", args: &[("altitude", unit_of(Length::UNIT)), ("pixel_pitch", unit_of(Length::UNIT)), ("focal_length", unit_of(Length::UNIT))], out: unit_of(Length::UNIT), eval: t_detector_limited_gsd, refuses: "",
        meaning: "detector-limited ground sample distance, h·p/f",
        kernel: "payload::detector_limited_gsd(altitude, pixel_pitch, focal_length)" },
    KernelFn { name: "diffraction_limited_gsd", args: &[("altitude", unit_of(Length::UNIT)), ("wavelength", unit_of(Length::UNIT)), ("aperture", unit_of(Length::UNIT))], out: unit_of(Length::UNIT), eval: t_diffraction_limited_gsd, refuses: "",
        meaning: "diffraction-limited ground resolution, Rayleigh",
        kernel: "payload::diffraction_limited_gsd(altitude, wavelength, aperture)" },
    KernelFn { name: "scene_data_volume", args: &[("pixels_across", "1"), ("pixels_along", "1"), ("bits_per_pixel", "1"), ("bands", "1"), ("compression_ratio", "1")], out: unit_of(DataVolume::UNIT), eval: t_scene_data_volume, refuses: "",
        meaning: "raw data volume of one optical scene",
        kernel: "payload::scene_data_volume(pixels_across, pixels_along, bits_per_pixel, bands, compression_ratio)" },
    KernelFn { name: "signal_electrons", args: &[("radiance", "1"), ("aperture", unit_of(Length::UNIT)), ("focal_length", unit_of(Length::UNIT)), ("pixel_pitch", unit_of(Length::UNIT)), ("transmission", unit_of(Ratio::UNIT)), ("quantum_efficiency", unit_of(Ratio::UNIT)), ("integration_time", unit_of(Time::UNIT)), ("wavelength", unit_of(Length::UNIT)), ("bandwidth", unit_of(Length::UNIT))], out: "1", eval: t_signal_electrons, refuses: "",
        meaning: "signal electrons per sample; radiance in W/m^2/sr/m as a pure number",
        kernel: "payload::signal_electrons(radiance, aperture, focal_length, pixel_pitch, transmission, quantum_efficiency, integration_time, wavelength, bandwidth)" },
    KernelFn { name: "optical_snr", args: &[("signal_electrons", "1"), ("dark_electrons", "1"), ("read_noise_electrons", "1")], out: "1", eval: t_optical_snr, refuses: "",
        meaning: "signal to noise ratio: shot, dark and read noise",
        kernel: "payload::optical_snr(signal_electrons, dark_electrons, read_noise_electrons)" },
    KernelFn { name: "optical_swath", args: &[("gsd", unit_of(Length::UNIT)), ("pixels_across_track", "1")], out: unit_of(Length::UNIT), eval: t_optical_swath, refuses: "",
        meaning: "swath from the pixels across track and the ground sample distance",
        kernel: "payload::optical_swath(gsd, pixels_across_track)" },
    KernelFn { name: "toa_timing_uncertainty", args: &[("bandwidth", unit_of(Frequency::UNIT)), ("snr_linear", "1"), ("integration", unit_of(Time::UNIT))], out: unit_of(Time::UNIT), eval: t_toa_timing_uncertainty, refuses: "",
        meaning: "Cramér-Rao bound on time-of-arrival",
        kernel: "payload::toa_timing_uncertainty(bandwidth, snr_linear, integration)" },
    KernelFn { name: "array_mass", args: &[("area", unit_of(Area::UNIT)), ("areal_density", "kg/m^2")], out: unit_of(Mass::UNIT), eval: t_array_mass, refuses: "",
        meaning: "solar array mass from area and areal density",
        kernel: "power::array_mass(area, areal_density)" },
    KernelFn { name: "array_power_bol", args: &[("area", unit_of(Area::UNIT)), ("cell_efficiency", unit_of(Ratio::UNIT)), ("packing_factor", unit_of(Ratio::UNIT)), ("incidence", unit_of(Angle::UNIT))], out: unit_of(Power::UNIT), eval: t_array_power_bol, refuses: "",
        meaning: "solar array output at beginning of life",
        kernel: "power::array_power_bol(area, cell_efficiency, packing_factor, incidence)" },
    KernelFn { name: "array_power_eol", args: &[("bol", unit_of(Power::UNIT)), ("degradation", unit_of(Ratio::UNIT))], out: unit_of(Power::UNIT), eval: t_array_power_eol, refuses: "",
        meaning: "array output at end of life",
        kernel: "power::array_power_eol(bol, degradation)" },
    KernelFn { name: "battery_cycles", args: &[("mission_duration", unit_of(Time::UNIT)), ("orbital_period", unit_of(Time::UNIT))], out: "1", eval: t_battery_cycles, refuses: "",
        meaning: "charge-discharge cycles over a mission, one per orbit",
        kernel: "power::battery_cycles(mission_duration, orbital_period)" },
    KernelFn { name: "battery_energy_required", args: &[("eclipse_load", unit_of(Power::UNIT)), ("eclipse_duration", unit_of(Time::UNIT)), ("depth_of_discharge", unit_of(Ratio::UNIT)), ("discharge_efficiency", unit_of(Ratio::UNIT))], out: unit_of(Energy::UNIT), eval: t_battery_energy_required, refuses: "",
        meaning: "battery energy to carry the eclipse load",
        kernel: "power::battery_energy_required(eclipse_load, eclipse_duration, depth_of_discharge, discharge_efficiency)" },
    KernelFn { name: "battery_mass", args: &[("energy", unit_of(Energy::UNIT)), ("specific_energy_wh_per_kg", "1")], out: unit_of(Mass::UNIT), eval: t_battery_mass, refuses: "",
        meaning: "battery mass from energy and specific energy; specific energy in Wh/kg as a pure number",
        kernel: "power::battery_mass(energy, specific_energy_wh_per_kg)" },
    KernelFn { name: "array_degradation", args: &[("annual_rate", unit_of(Ratio::UNIT)), ("years", "1")], out: unit_of(Ratio::UNIT), eval: t_array_degradation, refuses: "",
        meaning: "degradation factor after a number of years at an annual rate",
        kernel: "power::degradation(annual_rate, years)" },
    KernelFn { name: "power_demand", args: &[("propulsion", unit_of(Power::UNIT)), ("payload", unit_of(Power::UNIT)), ("avionics", unit_of(Power::UNIT)), ("comms", unit_of(Power::UNIT)), ("thermal", unit_of(Power::UNIT)), ("harness_loss", unit_of(Ratio::UNIT))], out: unit_of(Power::UNIT), eval: t_power_demand, refuses: "",
        meaning: "the named loads summed, with the harness loss",
        kernel: "power::power_demand(propulsion, payload, avionics, comms, thermal, harness_loss)" },
    KernelFn { name: "power_margin", args: &[("available", unit_of(Power::UNIT)), ("demand", unit_of(Power::UNIT))], out: unit_of(Ratio::UNIT), eval: t_power_margin, refuses: "",
        meaning: "power margin as a fraction of the demand",
        kernel: "power::power_margin(available, demand)" },
    KernelFn { name: "bus_power_demand", args: &[("thruster_input", unit_of(Power::UNIT)), ("ppu_efficiency", unit_of(Ratio::UNIT))], out: unit_of(Power::UNIT), eval: t_bus_power_demand, refuses: "",
        meaning: "power drawn from the bus, after the power processing unit",
        kernel: "prop::bus_power_demand(thruster_input, ppu_efficiency)" },
    KernelFn { name: "intake_collection_efficiency", args: &[("free_stream_density", unit_of(NumberDensity::UNIT)), ("velocity", unit_of(Velocity::UNIT)), ("intake_area", unit_of(Area::UNIT)), ("throat_area", unit_of(Area::UNIT)), ("eta_geo", unit_of(Ratio::UNIT)), ("beta_backflow", unit_of(Ratio::UNIT)), ("chamber_temperature", unit_of(Temperature::UNIT)), ("molar_mass", unit_of(MolarMass::UNIT))], out: unit_of(Ratio::UNIT), eval: t_intake_collection_efficiency, refuses: "",
        meaning: "the intake's collection efficiency, from its particle balance",
        kernel: "prop::intake_balance(free_stream_density, velocity, intake_area, throat_area, eta_geo, beta_backflow, chamber_temperature, molar_mass).collection_efficiency" },
    KernelFn { name: "intake_compression_ratio", args: &[("free_stream_density", unit_of(NumberDensity::UNIT)), ("velocity", unit_of(Velocity::UNIT)), ("intake_area", unit_of(Area::UNIT)), ("throat_area", unit_of(Area::UNIT)), ("eta_geo", unit_of(Ratio::UNIT)), ("beta_backflow", unit_of(Ratio::UNIT)), ("chamber_temperature", unit_of(Temperature::UNIT)), ("molar_mass", unit_of(MolarMass::UNIT))], out: unit_of(Ratio::UNIT), eval: t_intake_compression_ratio, refuses: "",
        meaning: "the intake's compression ratio, from its particle balance",
        kernel: "prop::intake_balance(free_stream_density, velocity, intake_area, throat_area, eta_geo, beta_backflow, chamber_temperature, molar_mass).compression_ratio" },
    KernelFn { name: "collected_mass_flow", args: &[("density", unit_of(MassDensity::UNIT)), ("velocity", unit_of(Velocity::UNIT)), ("intake_area", unit_of(Area::UNIT)), ("collection_efficiency", unit_of(Ratio::UNIT))], out: unit_of(MassFlow::UNIT), eval: t_collected_mass_flow, refuses: "",
        meaning: "mass flow reaching the thruster",
        kernel: "prop::collected_mass_flow(density, velocity, intake_area, collection_efficiency)" },
    KernelFn { name: "beam_exhaust_velocity", args: &[("beam_voltage", unit_of(Voltage::UNIT)), ("molar_mass", unit_of(MolarMass::UNIT))], out: unit_of(Velocity::UNIT), eval: t_beam_exhaust_velocity, refuses: "",
        meaning: "exhaust velocity of a beam accelerated through a potential, sqrt(2qV/m)",
        kernel: "prop::beam_exhaust_velocity(beam_voltage, molar_mass)" },
    KernelFn { name: "incident_mass_flux", args: &[("density", unit_of(MassDensity::UNIT)), ("velocity", unit_of(Velocity::UNIT))], out: unit_of(MassFlux::UNIT), eval: t_incident_mass_flux, refuses: "",
        meaning: "incident mass flux, rho·V",
        kernel: "prop::incident_mass_flux(density, velocity)" },
    KernelFn { name: "thruster_input_power", args: &[("jet", unit_of(Power::UNIT)), ("ionisation", unit_of(Power::UNIT)), ("other_losses", unit_of(Ratio::UNIT))], out: unit_of(Power::UNIT), eval: t_thruster_input_power, refuses: "",
        meaning: "electrical power the thruster draws",
        kernel: "prop::thruster_input_power(jet, ionisation, other_losses)" },
    KernelFn { name: "ion_mass_flow", args: &[("collected", unit_of(MassFlow::UNIT)), ("propellant_utilisation", unit_of(Ratio::UNIT))], out: unit_of(MassFlow::UNIT), eval: t_ion_mass_flow, refuses: "",
        meaning: "ion mass flow, from propellant utilisation",
        kernel: "prop::ion_mass_flow(collected, propellant_utilisation)" },
    KernelFn { name: "ionisation_power", args: &[("ion_flow", unit_of(MassFlow::UNIT)), ("molar_mass", unit_of(MolarMass::UNIT)), ("epsilon_ev", "1")], out: unit_of(Power::UNIT), eval: t_ionisation_power, refuses: "",
        meaning: "power spent making the ions; energy per ion in eV as a pure number",
        kernel: "prop::ionisation_power(ion_flow, molar_mass, epsilon_ev)" },
    KernelFn { name: "jet_power", args: &[("thrust", unit_of(Force::UNIT)), ("ion_flow", unit_of(MassFlow::UNIT))], out: unit_of(Power::UNIT), eval: t_jet_power, refuses: "",
        meaning: "jet power, F^2/(2·m_dot)",
        kernel: "prop::jet_power(thrust, ion_flow)" },
    KernelFn { name: "specific_impulse", args: &[("thrust", unit_of(Force::UNIT)), ("collected_flow", unit_of(MassFlow::UNIT))], out: unit_of(Time::UNIT), eval: t_specific_impulse, refuses: "",
        meaning: "specific impulse referred to the collected flow",
        kernel: "prop::specific_impulse(thrust, collected_flow)" },
    KernelFn { name: "beam_thrust", args: &[("ion_flow", unit_of(MassFlow::UNIT)), ("exhaust_velocity", unit_of(Velocity::UNIT)), ("alpha_div", unit_of(Ratio::UNIT)), ("alpha_double", unit_of(Ratio::UNIT))], out: unit_of(Force::UNIT), eval: t_beam_thrust, refuses: "",
        meaning: "thrust of an ion beam, with divergence and doubly-charged corrections",
        kernel: "prop::beam_thrust(ion_flow, exhaust_velocity, alpha_div, alpha_double)" },
    KernelFn { name: "total_efficiency", args: &[("jet", unit_of(Power::UNIT)), ("bus_power", unit_of(Power::UNIT))], out: unit_of(Ratio::UNIT), eval: t_total_efficiency, refuses: "",
        meaning: "jet power over bus power",
        kernel: "prop::total_efficiency(jet, bus_power)" },
    KernelFn { name: "thrust_to_drag", args: &[("thrust", unit_of(Force::UNIT)), ("drag", unit_of(Force::UNIT))], out: unit_of(Ratio::UNIT), eval: t_thrust_to_drag, refuses: "",
        meaning: "thrust over drag",
        kernel: "prop::thrust_to_drag(thrust, drag)" },
    KernelFn { name: "absorbed_albedo", args: &[("area", unit_of(Area::UNIT)), ("absorptivity", unit_of(Ratio::UNIT)), ("view_factor", unit_of(Ratio::UNIT))], out: unit_of(Power::UNIT), eval: t_absorbed_albedo, refuses: "",
        meaning: "albedo absorbed",
        kernel: "thermal::absorbed_albedo(area, absorptivity, view_factor)" },
    KernelFn { name: "absorbed_earth_ir", args: &[("area", unit_of(Area::UNIT)), ("emissivity", unit_of(Ratio::UNIT)), ("view_factor", unit_of(Ratio::UNIT))], out: unit_of(Power::UNIT), eval: t_absorbed_earth_ir, refuses: "",
        meaning: "Earth infrared absorbed",
        kernel: "thermal::absorbed_earth_ir(area, emissivity, view_factor)" },
    KernelFn { name: "absorbed_solar", args: &[("area", unit_of(Area::UNIT)), ("absorptivity", unit_of(Ratio::UNIT)), ("incidence", unit_of(Angle::UNIT))], out: unit_of(Power::UNIT), eval: t_absorbed_solar, refuses: "",
        meaning: "direct solar absorbed",
        kernel: "thermal::absorbed_solar(area, absorptivity, incidence)" },
    KernelFn { name: "free_molecular_heating", args: &[("density", unit_of(MassDensity::UNIT)), ("velocity", unit_of(Velocity::UNIT)), ("area", unit_of(Area::UNIT)), ("heat_transfer_coefficient", unit_of(Ratio::UNIT))], out: unit_of(Power::UNIT), eval: t_free_molecular_heating, refuses: "",
        meaning: "aerodynamic heating in free-molecular flow",
        kernel: "thermal::free_molecular_heating(density, velocity, area, heat_transfer_coefficient)" },
    KernelFn { name: "equilibrium_temperature", args: &[("total_absorbed", unit_of(Power::UNIT)), ("internal_dissipation", unit_of(Power::UNIT)), ("radiating_area", unit_of(Area::UNIT)), ("emissivity", unit_of(Ratio::UNIT))], out: unit_of(Temperature::UNIT), eval: t_equilibrium_temperature, refuses: "",
        meaning: "the temperature that balances what is absorbed and dissipated",
        kernel: "thermal::equilibrium_temperature(total_absorbed, internal_dissipation, radiating_area, emissivity)" },
    KernelFn { name: "required_radiator_area", args: &[("heat_to_reject", unit_of(Power::UNIT)), ("emissivity", unit_of(Ratio::UNIT)), ("radiator_temperature", unit_of(Temperature::UNIT)), ("sink_temperature", unit_of(Temperature::UNIT))], out: unit_of(Area::UNIT), eval: t_required_radiator_area, refuses: "",
        meaning: "radiator area to hold a temperature against a load",
        kernel: "thermal::required_radiator_area(heat_to_reject, emissivity, radiator_temperature, sink_temperature)" },
    KernelFn { name: "earth_view_factor", args: &[("radius", unit_of(Length::UNIT))], out: unit_of(Ratio::UNIT), eval: t_earth_view_factor, refuses: "",
        meaning: "view factor to the Earth from a nadir-facing plate",
        kernel: "thermal::earth_view_factor(radius)" },
    KernelFn { name: "thermal_margin", args: &[("predicted", unit_of(Temperature::UNIT)), ("limit", unit_of(Temperature::UNIT))], out: unit_of(Temperature::UNIT), eval: t_thermal_margin, refuses: "",
        meaning: "thermal margin, in kelvin, against a limit",
        kernel: "thermal::thermal_margin(predicted, limit)" },
];

fn t_aerodynamic_torque(a: &[f64]) -> f64 {
    aero::aerodynamic_torque(Force::new(a[0]), Length::new(a[1])).get()
}
fn t_drag_acceleration(a: &[f64]) -> f64 {
    aero::drag_acceleration(Force::new(a[0]), Mass::new(a[1])).get()
}
fn t_dynamic_pressure(a: &[f64]) -> f64 {
    aero::dynamic_pressure(MassDensity::new(a[0]), Velocity::new(a[1])).get()
}
fn t_speed_ratio(a: &[f64]) -> f64 {
    aero::speed_ratio(
        Velocity::new(a[0]),
        Temperature::new(a[1]),
        MolarMass::new(a[2]),
    )
    .get()
}
fn t_accommodation_coefficient(a: &[f64]) -> f64 {
    aero::accommodation_coefficient(NumberDensity::new(a[0]), Temperature::new(a[1]))
}
fn t_atomic_oxygen_fluence(a: &[f64]) -> f64 {
    aero::atomic_oxygen_fluence(
        NumberDensity::new(a[0]),
        Velocity::new(a[1]),
        Time::new(a[2]),
    )
}
fn t_ballistic_coefficient(a: &[f64]) -> f64 {
    aero::ballistic_coefficient(Mass::new(a[0]), a[1], Area::new(a[2]))
}
fn t_cylinder_drag_coefficient(a: &[f64]) -> f64 {
    aero::cylinder_drag_coefficient(
        a[0],
        Length::new(a[1]),
        Length::new(a[2]),
        a[3],
        Temperature::new(a[4]),
        Velocity::new(a[5]),
        MolarMass::new(a[6]),
    )
}
fn t_aperture_gain_db(a: &[f64]) -> f64 {
    comms::aperture_gain_db(Length::new(a[0]), Frequency::new(a[1]), Ratio::new(a[2]))
}
fn t_carrier_to_noise_density_db(a: &[f64]) -> f64 {
    comms::carrier_to_noise_density_db(a[0], a[1], a[2], a[3])
}
fn t_max_contact_time(a: &[f64]) -> f64 {
    comms::max_contact_time(Angle::new(a[0]), Time::new(a[1])).get()
}
fn t_daily_downlink_volume(a: &[f64]) -> f64 {
    comms::daily_downlink_volume(DataVolume::new(a[0]), a[1], Ratio::new(a[2])).get()
}
fn t_achievable_data_rate(a: &[f64]) -> f64 {
    comms::achievable_data_rate(a[0], a[1], a[2], a[3]).get()
}
fn t_doppler_shift(a: &[f64]) -> f64 {
    comms::doppler_shift(Frequency::new(a[0]), Velocity::new(a[1])).get()
}
fn t_eb_over_n0_db(a: &[f64]) -> f64 {
    comms::eb_over_n0_db(a[0], DataRate::new(a[1]))
}
fn t_eirp_dbw(a: &[f64]) -> f64 {
    comms::eirp_dbw(Power::new(a[0]), a[1], a[2])
}
fn t_g_over_t_db(a: &[f64]) -> f64 {
    comms::g_over_t_db(a[0], Temperature::new(a[1]))
}
fn t_pass_data_volume(a: &[f64]) -> f64 {
    comms::pass_data_volume(DataRate::new(a[0]), Time::new(a[1]), Ratio::new(a[2])).get()
}
fn t_free_space_path_loss_db(a: &[f64]) -> f64 {
    comms::free_space_path_loss_db(Length::new(a[0]), Frequency::new(a[1]))
}
fn t_system_noise_temperature(a: &[f64]) -> f64 {
    comms::system_noise_temperature(Temperature::new(a[0]), a[1], a[2], Temperature::new(a[3]))
        .get()
}
fn t_link_margin_db(a: &[f64]) -> f64 {
    comms::link_margin_db(a[0], a[1], a[2])
}
fn t_cost_per_service_unit(a: &[f64]) -> f64 {
    cost::cost_per_service_unit(Money::new(a[0]), a[1]).get()
}
fn t_production_run_cost(a: &[f64]) -> f64 {
    cost::production_run_cost(Money::new(a[0]), a[1] as u32, a[2]).get()
}
fn t_programme_cost(a: &[f64]) -> f64 {
    cost::programme_cost(Money::new(a[0]), Money::new(a[1]), Money::new(a[2]), a[3]).get()
}
fn t_replacement_avoided(a: &[f64]) -> f64 {
    cost::replacement_avoided(Money::new(a[0]), Money::new(a[1]), a[2]).get()
}
fn t_cer_inflated(a: &[f64]) -> f64 {
    cost::Cer {
        a: a[1],
        b: a[2],
        base_year: a[3] as u16,
        sigma: 0.0,
    }
    .evaluate_inflated(a[0], a[4] as u16, a[5])
    .get()
}
fn t_density_uncertainty(a: &[f64]) -> f64 {
    env::density_uncertainty(a[0]).get()
}
fn t_exospheric_temperature(a: &[f64]) -> f64 {
    env::exospheric_temperature(a[0], a[1], a[2]).get()
}
fn t_knudsen(a: &[f64]) -> f64 {
    env::knudsen(Length::new(a[0]), Length::new(a[1])).get()
}
fn t_thermosphere_temperature(a: &[f64]) -> f64 {
    env::temperature(Length::new(a[0]), Temperature::new(a[1])).get()
}
fn t_magnetic_field(a: &[f64]) -> f64 {
    env::magnetic_field(Length::new(a[0]), Angle::new(a[1])).get()
}
fn t_mean_free_path(a: &[f64]) -> f64 {
    env::mean_free_path(NumberDensity::new(a[0])).get()
}
fn t_along_track_error_from_drag(a: &[f64]) -> f64 {
    gnc::along_track_error_from_drag(Acceleration::new(a[0]), Time::new(a[1])).get()
}
fn t_gravity_gradient_torque(a: &[f64]) -> f64 {
    gnc::gravity_gradient_torque(Length::new(a[0]), a[1], a[2], Angle::new(a[3])).get()
}
fn t_pointing_to_ground_error(a: &[f64]) -> f64 {
    gnc::pointing_to_ground_error(Angle::new(a[0]), Length::new(a[1])).get()
}
fn t_magnetic_torque(a: &[f64]) -> f64 {
    gnc::magnetic_torque(DipoleMoment::new(a[0]), MagneticFluxDensity::new(a[1])).get()
}
fn t_magnetorquer_dipole_required(a: &[f64]) -> f64 {
    gnc::magnetorquer_dipole_required(
        AngularMomentum::new(a[0]),
        MagneticFluxDensity::new(a[1]),
        Time::new(a[2]),
    )
    .get()
}
fn t_momentum_storage_required(a: &[f64]) -> f64 {
    gnc::momentum_storage_required(Torque::new(a[0]), Time::new(a[1]), Ratio::new(a[2])).get()
}
fn t_navigation_position_error(a: &[f64]) -> f64 {
    gnc::navigation_position_error(Length::new(a[0]), a[1]).get()
}
fn t_pointing_error_rss(a: &[f64]) -> f64 {
    gnc::pointing_error_rss(&[
        Angle::new(a[0]),
        Angle::new(a[1]),
        Angle::new(a[2]),
        Angle::new(a[3]),
    ])
    .get()
}
fn t_solar_pressure_torque(a: &[f64]) -> f64 {
    gnc::solar_pressure_torque(
        Area::new(a[0]),
        Ratio::new(a[1]),
        Length::new(a[2]),
        Angle::new(a[3]),
    )
    .get()
}
fn t_total_disturbance_torque(a: &[f64]) -> f64 {
    gnc::total_disturbance_torque(
        Torque::new(a[0]),
        Torque::new(a[1]),
        Torque::new(a[2]),
        Torque::new(a[3]),
    )
    .get()
}
fn t_dry_mass(a: &[f64]) -> f64 {
    mass::dry_mass(
        Mass::new(a[0]),
        Mass::new(a[1]),
        Mass::new(a[2]),
        Mass::new(a[3]),
        Mass::new(a[4]),
        Mass::new(a[5]),
        Mass::new(a[6]),
        Mass::new(a[7]),
        Mass::new(a[8]),
        Ratio::new(a[9]),
    )
    .get()
}
fn t_mass_margin(a: &[f64]) -> f64 {
    mass::mass_margin(Mass::new(a[0]), Mass::new(a[1])).get()
}
fn t_wet_mass(a: &[f64]) -> f64 {
    mass::wet_mass(Mass::new(a[0]), Mass::new(a[1])).get()
}
fn t_margin_at_least(a: &[f64]) -> f64 {
    mission::closure(a[0], a[1], mission::Sense::AtLeast).margin
}
fn t_margin_at_most(a: &[f64]) -> f64 {
    mission::closure(a[0], a[1], mission::Sense::AtMost).margin
}
fn t_access_area(a: &[f64]) -> f64 {
    mission::access_area(Angle::new(a[0])).get()
}
fn t_k_of_n_availability(a: &[f64]) -> f64 {
    mission::k_of_n_availability(Ratio::new(a[0]), a[1] as u32, a[2] as u32).get()
}
fn t_instantaneous_coverage_fraction(a: &[f64]) -> f64 {
    mission::instantaneous_coverage_fraction(Angle::new(a[0])).get()
}
fn t_end_to_end_latency(a: &[f64]) -> f64 {
    mission::end_to_end_latency(
        Time::new(a[0]),
        Time::new(a[1]),
        Time::new(a[2]),
        Time::new(a[3]),
    )
    .get()
}
fn t_mean_revisit_time(a: &[f64]) -> f64 {
    mission::mean_revisit_time(Length::new(a[0]), Velocity::new(a[1]), a[2]).get()
}
fn t_satellites_for_revisit(a: &[f64]) -> f64 {
    mission::satellites_for_revisit(Time::new(a[0]), Length::new(a[1]), Velocity::new(a[2]))
}
fn t_mean_time_to_downlink(a: &[f64]) -> f64 {
    mission::mean_time_to_downlink(a[0]).get()
}
fn t_circular_velocity(a: &[f64]) -> f64 {
    orbit::circular_velocity(Length::new(a[0])).get()
}
fn t_decay_rate(a: &[f64]) -> f64 {
    orbit::decay_rate(MassDensity::new(a[0]), a[1], Length::new(a[2])).get()
}
fn t_deorbit_delta_v(a: &[f64]) -> f64 {
    orbit::deorbit_delta_v(Length::new(a[0]), Length::new(a[1])).get()
}
fn t_drag_makeup_delta_v(a: &[f64]) -> f64 {
    orbit::drag_makeup_delta_v(Acceleration::new(a[0]), Time::new(a[1])).get()
}
fn t_earth_central_angle(a: &[f64]) -> f64 {
    orbit::earth_central_angle(Length::new(a[0]), Angle::new(a[1])).get()
}
fn t_eclipse_fraction(a: &[f64]) -> f64 {
    orbit::eclipse_fraction(Length::new(a[0]), Angle::new(a[1])).get()
}
fn t_ground_track_speed(a: &[f64]) -> f64 {
    orbit::ground_track_speed(Velocity::new(a[0]), Length::new(a[1])).get()
}
fn t_nodal_regression(a: &[f64]) -> f64 {
    orbit::nodal_regression(Length::new(a[0]), a[1], Angle::new(a[2])).get()
}
fn t_orbital_period(a: &[f64]) -> f64 {
    orbit::period(Length::new(a[0])).get()
}
fn t_orbit_radius(a: &[f64]) -> f64 {
    orbit::radius(Length::new(a[0])).get()
}
fn t_slant_range(a: &[f64]) -> f64 {
    orbit::slant_range(Length::new(a[0]), Angle::new(a[1])).get()
}
fn t_sun_synchronous_inclination(a: &[f64]) -> f64 {
    match orbit::sun_synchronous_inclination(Length::new(a[0]), a[1]) {
        Some(v) => v.get(),
        None => f64::NAN,
    }
}
fn t_swath_width(a: &[f64]) -> f64 {
    orbit::swath_width(Angle::new(a[0])).get()
}
fn t_propellant_mass(a: &[f64]) -> f64 {
    orbit::propellant_mass(Mass::new(a[0]), Velocity::new(a[1]), Velocity::new(a[2])).get()
}
fn t_exhaust_velocity(a: &[f64]) -> f64 {
    orbit::exhaust_velocity(Time::new(a[0])).get()
}
fn t_dwell_time(a: &[f64]) -> f64 {
    payload::dwell_time(Length::new(a[0]), Velocity::new(a[1])).get()
}
fn t_tdoa_geolocation_error(a: &[f64]) -> f64 {
    payload::tdoa_geolocation_error(Time::new(a[0]), a[1]).get()
}
fn t_achieved_gsd(a: &[f64]) -> f64 {
    payload::achieved_gsd(Length::new(a[0]), Length::new(a[1])).get()
}
fn t_detector_limited_gsd(a: &[f64]) -> f64 {
    payload::detector_limited_gsd(Length::new(a[0]), Length::new(a[1]), Length::new(a[2])).get()
}
fn t_diffraction_limited_gsd(a: &[f64]) -> f64 {
    payload::diffraction_limited_gsd(Length::new(a[0]), Length::new(a[1]), Length::new(a[2])).get()
}
fn t_scene_data_volume(a: &[f64]) -> f64 {
    payload::scene_data_volume(a[0], a[1], a[2], a[3], a[4]).get()
}
fn t_signal_electrons(a: &[f64]) -> f64 {
    payload::signal_electrons(
        a[0],
        Length::new(a[1]),
        Length::new(a[2]),
        Length::new(a[3]),
        Ratio::new(a[4]),
        Ratio::new(a[5]),
        Time::new(a[6]),
        Length::new(a[7]),
        Length::new(a[8]),
    )
}
fn t_optical_snr(a: &[f64]) -> f64 {
    payload::optical_snr(a[0], a[1], a[2])
}
fn t_optical_swath(a: &[f64]) -> f64 {
    payload::optical_swath(Length::new(a[0]), a[1]).get()
}
fn t_toa_timing_uncertainty(a: &[f64]) -> f64 {
    payload::toa_timing_uncertainty(Frequency::new(a[0]), a[1], Time::new(a[2])).get()
}
fn t_array_mass(a: &[f64]) -> f64 {
    power::array_mass(Area::new(a[0]), a[1]).get()
}
fn t_array_power_bol(a: &[f64]) -> f64 {
    power::array_power_bol(
        Area::new(a[0]),
        Ratio::new(a[1]),
        Ratio::new(a[2]),
        Angle::new(a[3]),
    )
    .get()
}
fn t_array_power_eol(a: &[f64]) -> f64 {
    power::array_power_eol(Power::new(a[0]), Ratio::new(a[1])).get()
}
fn t_battery_cycles(a: &[f64]) -> f64 {
    power::battery_cycles(Time::new(a[0]), Time::new(a[1]))
}
fn t_battery_energy_required(a: &[f64]) -> f64 {
    power::battery_energy_required(
        Power::new(a[0]),
        Time::new(a[1]),
        Ratio::new(a[2]),
        Ratio::new(a[3]),
    )
    .get()
}
fn t_battery_mass(a: &[f64]) -> f64 {
    power::battery_mass(Energy::new(a[0]), a[1]).get()
}
fn t_array_degradation(a: &[f64]) -> f64 {
    power::degradation(Ratio::new(a[0]), a[1]).get()
}
fn t_power_demand(a: &[f64]) -> f64 {
    power::power_demand(
        Power::new(a[0]),
        Power::new(a[1]),
        Power::new(a[2]),
        Power::new(a[3]),
        Power::new(a[4]),
        Ratio::new(a[5]),
    )
    .get()
}
fn t_power_margin(a: &[f64]) -> f64 {
    power::power_margin(Power::new(a[0]), Power::new(a[1])).get()
}
fn t_bus_power_demand(a: &[f64]) -> f64 {
    prop::bus_power_demand(Power::new(a[0]), Ratio::new(a[1])).get()
}
fn t_intake_collection_efficiency(a: &[f64]) -> f64 {
    prop::intake_balance(
        NumberDensity::new(a[0]),
        Velocity::new(a[1]),
        Area::new(a[2]),
        Area::new(a[3]),
        Ratio::new(a[4]),
        Ratio::new(a[5]),
        Temperature::new(a[6]),
        MolarMass::new(a[7]),
    )
    .collection_efficiency
    .get()
}
fn t_intake_compression_ratio(a: &[f64]) -> f64 {
    prop::intake_balance(
        NumberDensity::new(a[0]),
        Velocity::new(a[1]),
        Area::new(a[2]),
        Area::new(a[3]),
        Ratio::new(a[4]),
        Ratio::new(a[5]),
        Temperature::new(a[6]),
        MolarMass::new(a[7]),
    )
    .compression_ratio
    .get()
}
fn t_collected_mass_flow(a: &[f64]) -> f64 {
    prop::collected_mass_flow(
        MassDensity::new(a[0]),
        Velocity::new(a[1]),
        Area::new(a[2]),
        Ratio::new(a[3]),
    )
    .get()
}
fn t_beam_exhaust_velocity(a: &[f64]) -> f64 {
    prop::beam_exhaust_velocity(Voltage::new(a[0]), MolarMass::new(a[1])).get()
}
fn t_incident_mass_flux(a: &[f64]) -> f64 {
    prop::incident_mass_flux(MassDensity::new(a[0]), Velocity::new(a[1])).get()
}
fn t_thruster_input_power(a: &[f64]) -> f64 {
    prop::thruster_input_power(Power::new(a[0]), Power::new(a[1]), Ratio::new(a[2])).get()
}
fn t_ion_mass_flow(a: &[f64]) -> f64 {
    prop::ion_mass_flow(MassFlow::new(a[0]), Ratio::new(a[1])).get()
}
fn t_ionisation_power(a: &[f64]) -> f64 {
    prop::ionisation_power(MassFlow::new(a[0]), MolarMass::new(a[1]), a[2]).get()
}
fn t_jet_power(a: &[f64]) -> f64 {
    prop::jet_power(Force::new(a[0]), MassFlow::new(a[1])).get()
}
fn t_specific_impulse(a: &[f64]) -> f64 {
    prop::specific_impulse(Force::new(a[0]), MassFlow::new(a[1])).get()
}
fn t_beam_thrust(a: &[f64]) -> f64 {
    prop::beam_thrust(
        MassFlow::new(a[0]),
        Velocity::new(a[1]),
        Ratio::new(a[2]),
        Ratio::new(a[3]),
    )
    .get()
}
fn t_total_efficiency(a: &[f64]) -> f64 {
    prop::total_efficiency(Power::new(a[0]), Power::new(a[1])).get()
}
fn t_thrust_to_drag(a: &[f64]) -> f64 {
    prop::thrust_to_drag(Force::new(a[0]), Force::new(a[1])).get()
}
fn t_absorbed_albedo(a: &[f64]) -> f64 {
    thermal::absorbed_albedo(Area::new(a[0]), Ratio::new(a[1]), Ratio::new(a[2])).get()
}
fn t_absorbed_earth_ir(a: &[f64]) -> f64 {
    thermal::absorbed_earth_ir(Area::new(a[0]), Ratio::new(a[1]), Ratio::new(a[2])).get()
}
fn t_absorbed_solar(a: &[f64]) -> f64 {
    thermal::absorbed_solar(Area::new(a[0]), Ratio::new(a[1]), Angle::new(a[2])).get()
}
fn t_free_molecular_heating(a: &[f64]) -> f64 {
    thermal::free_molecular_heating(
        MassDensity::new(a[0]),
        Velocity::new(a[1]),
        Area::new(a[2]),
        Ratio::new(a[3]),
    )
    .get()
}
fn t_equilibrium_temperature(a: &[f64]) -> f64 {
    thermal::equilibrium_temperature(
        Power::new(a[0]),
        Power::new(a[1]),
        Area::new(a[2]),
        Ratio::new(a[3]),
    )
    .get()
}
fn t_required_radiator_area(a: &[f64]) -> f64 {
    thermal::required_radiator_area(
        Power::new(a[0]),
        Ratio::new(a[1]),
        Temperature::new(a[2]),
        Temperature::new(a[3]),
    )
    .get()
}
fn t_earth_view_factor(a: &[f64]) -> f64 {
    thermal::earth_view_factor(Length::new(a[0])).get()
}
fn t_thermal_margin(a: &[f64]) -> f64 {
    thermal::thermal_margin(Temperature::new(a[0]), Temperature::new(a[1])).get()
}

pub fn kernel_function(name: &str) -> Option<&'static KernelFn> {
    KERNEL_FUNCTIONS.iter().find(|f| f.name == name)
}
