//! Communications and the link budget.
//!
//! Everything is in decibels except where it is not, and the two are never
//! added. The `Decibel` unit exists precisely so that a logarithmic ratio and a
//! linear one cannot be summed by accident — which is a real defect that
//! reaches flight, and it looks exactly like a working link budget with one
//! wrong number in it.

use vleo_units::constants::*;
use vleo_units::pmath;
use vleo_units::*;

/// Convert a linear ratio to decibels.
pub fn to_db(linear: f64) -> f64 {
    10.0 * pmath::log10(linear)
}

/// Convert decibels to a linear ratio.
pub fn from_db(db: f64) -> f64 {
    pmath::powf(10.0, db / 10.0)
}

/// Gain of a circular aperture antenna, in decibels relative to isotropic.
///
/// ```text
/// G = eta·(pi·D/lambda)^2
/// ```
pub fn aperture_gain_db(diameter: Length, frequency: Frequency, efficiency: Ratio) -> f64 {
    let lambda = SPEED_OF_LIGHT.get() / frequency.get();
    let g = efficiency.get() * pmath::powi(pmath::PI * diameter.get() / lambda, 2);
    to_db(g)
}

/// Half-power beamwidth of a circular aperture, in radians.
pub fn aperture_beamwidth(diameter: Length, frequency: Frequency) -> Angle {
    let lambda = SPEED_OF_LIGHT.get() / frequency.get();
    Angle::new(1.02 * lambda / diameter.get())
}

/// Free-space path loss in decibels, `(4·pi·d/lambda)^2`.
pub fn free_space_path_loss_db(range: Length, frequency: Frequency) -> f64 {
    let lambda = SPEED_OF_LIGHT.get() / frequency.get();
    to_db(pmath::powi(4.0 * pmath::PI * range.get() / lambda, 2))
}

/// Effective isotropic radiated power, in dBW.
pub fn eirp_dbw(transmit_power: Power, antenna_gain_db: f64, line_loss_db: f64) -> f64 {
    to_db(transmit_power.get()) + antenna_gain_db - line_loss_db
}

/// System noise temperature from antenna, line and receiver contributions.
pub fn system_noise_temperature(
    antenna_temperature: Temperature,
    line_loss_db: f64,
    receiver_noise_figure_db: f64,
    physical_temperature: Temperature,
) -> Temperature {
    let l = from_db(line_loss_db);
    let f = from_db(receiver_noise_figure_db);
    let t_line = physical_temperature.get() * (l - 1.0);
    let t_rx = physical_temperature.get() * (f - 1.0);
    Temperature::new(antenna_temperature.get() / l + t_line / l + t_rx)
}

/// Receiver figure of merit, `G/T` in dB/K.
pub fn g_over_t_db(receive_gain_db: f64, system_noise_temperature: Temperature) -> f64 {
    receive_gain_db - to_db(system_noise_temperature.get())
}

/// Carrier to noise density ratio, in dB-Hz.
///
/// ```text
/// C/N0 = EIRP - FSPL - L_atm + G/T - 10·log10(k)
/// ```
pub fn carrier_to_noise_density_db(
    eirp_dbw: f64,
    path_loss_db: f64,
    atmospheric_loss_db: f64,
    g_over_t_db: f64,
) -> f64 {
    eirp_dbw - path_loss_db - atmospheric_loss_db + g_over_t_db - to_db(K_BOLTZMANN)
}

/// Energy per bit over noise density, in dB.
pub fn eb_over_n0_db(cn0_db: f64, data_rate: DataRate) -> f64 {
    cn0_db - to_db(data_rate.get())
}

/// Link margin in dB against a required `Eb/N0`, including implementation loss.
///
/// A margin, not a verdict. Whether 3 dB is enough is a decision that belongs
/// to a person, and it is one of the nine.
pub fn link_margin_db(
    eb_n0_db: f64,
    required_eb_n0_db: f64,
    implementation_loss_db: f64,
) -> f64 {
    eb_n0_db - required_eb_n0_db - implementation_loss_db
}

/// The highest data rate that closes the link at a stated margin — the inverse
/// question, and the one a downlink design actually asks.
pub fn achievable_data_rate(
    cn0_db: f64,
    required_eb_n0_db: f64,
    implementation_loss_db: f64,
    margin_db: f64,
) -> DataRate {
    let allowed = cn0_db - required_eb_n0_db - implementation_loss_db - margin_db;
    DataRate::new(from_db(allowed))
}

/// Contact time for one pass, from the Earth-central half-angle and the
/// spacecraft's angular rate about the Earth.
///
/// This is the maximum — a pass directly overhead. A realistic average over
/// many passes is roughly 60% of it, and a downlink volume computed from the
/// maximum is a downlink volume that will not be achieved.
pub fn max_contact_time(earth_central_angle: Angle, orbital_period: Time) -> Time {
    Time::new(orbital_period.get() * earth_central_angle.get() / pmath::PI)
}

/// Data volume downlinked in a pass.
pub fn pass_data_volume(rate: DataRate, contact: Time, link_efficiency: Ratio) -> DataVolume {
    rate * contact * link_efficiency.get()
}

/// Doppler shift at closest approach, `f·v/c`.
pub fn doppler_shift(frequency: Frequency, relative_velocity: Velocity) -> Frequency {
    Frequency::new(frequency.get() * relative_velocity.get() / SPEED_OF_LIGHT.get())
}

/// Daily downlink capacity from a number of passes per day.
pub fn daily_downlink_volume(
    per_pass: DataVolume,
    passes_per_day: f64,
    availability: Ratio,
) -> DataVolume {
    DataVolume::new(per_pass.get() * passes_per_day * availability.get())
}
