//! Service payloads — optical, radar, radio-frequency geolocation, and
//! positioning.
//!
//! One thing VLEO gives away for free, and it is the reason the whole regime is
//! interesting: every one of these payloads gets better as the altitude falls,
//! and most of them get better faster than linearly. An optical system's
//! ground sample distance is linear in altitude; its signal-to-noise ratio goes
//! as the inverse square; a radar's power-aperture requirement goes as the
//! fourth power of range. Halving the altitude is not a 50% improvement.

use vleo_units::constants::*;
use vleo_units::pmath;
use vleo_units::*;

// --- optical -----------------------------------------------------------------

/// Diffraction-limited ground resolution, Rayleigh criterion.
///
/// ```text
/// GSD_diff = 1.22·lambda·h/D
/// ```
///
/// The floor no detector can beat. At 300 km, 500 nm and a 0.3 m aperture it is
/// 0.61 m; the same telescope at 600 km gets 1.22 m.
pub fn diffraction_limited_gsd(altitude: Length, wavelength: Length, aperture: Length) -> Length {
    Length::new(1.22 * wavelength.get() * altitude.get() / aperture.get())
}

/// Detector-limited ground sample distance, `h·p/f`.
pub fn detector_limited_gsd(altitude: Length, pixel_pitch: Length, focal_length: Length) -> Length {
    Length::new(altitude.get() * pixel_pitch.get() / focal_length.get())
}

/// The ground sample distance actually achieved: the worse of the two limits.
///
/// Taking the better of them is a mistake with a specific shape — it produces a
/// resolution claim the optics cannot deliver and the detector cannot sample,
/// and it survives review because both inputs are individually correct.
pub fn achieved_gsd(diffraction: Length, detector: Length) -> Length {
    Length::new(pmath::max(diffraction.get(), detector.get()))
}

/// Swath width from the number of across-track pixels and the ground sample
/// distance.
pub fn optical_swath(gsd: Length, pixels_across_track: f64) -> Length {
    Length::new(gsd.get() * pixels_across_track)
}

/// Integration time available per ground sample, from the ground track speed.
///
/// The constraint VLEO imposes and higher orbits do not: the ground moves under
/// the sensor faster in angular terms, so there is less time to collect photons
/// from each sample, and that partly gives back the signal-to-noise the shorter
/// range won.
pub fn dwell_time(gsd: Length, ground_track_speed: Velocity) -> Time {
    Time::new(gsd.get() / ground_track_speed.get())
}

/// Photoelectrons collected per pixel per frame.
///
/// ```text
/// N_e = L·(pi/4)·(D/f)^2·A_px·tau·QE·t_int·lambda/(h·c)
/// ```
///
/// with `L` the at-aperture spectral radiance in W/m^2/sr/m.
#[allow(clippy::too_many_arguments)] // the relation has this many terms; naming them all is the point
pub fn signal_electrons(
    radiance: f64,
    aperture: Length,
    focal_length: Length,
    pixel_pitch: Length,
    transmission: Ratio,
    quantum_efficiency: Ratio,
    integration_time: Time,
    wavelength: Length,
    bandwidth: Length,
) -> f64 {
    let f_number = focal_length.get() / aperture.get();
    let solid_angle = pmath::PI / (4.0 * f_number * f_number);
    let a_px = pixel_pitch.get() * pixel_pitch.get();
    let photon_energy = PLANCK * SPEED_OF_LIGHT.get() / wavelength.get();
    radiance
        * solid_angle
        * a_px
        * bandwidth.get()
        * transmission.get()
        * quantum_efficiency.get()
        * integration_time.get()
        / photon_energy
}

/// Signal to noise ratio, shot noise plus dark current plus read noise.
pub fn optical_snr(signal_electrons: f64, dark_electrons: f64, read_noise_electrons: f64) -> f64 {
    let noise = pmath::sqrt(
        signal_electrons + dark_electrons + read_noise_electrons * read_noise_electrons,
    );
    if noise <= 0.0 {
        return 0.0;
    }
    signal_electrons / noise
}

/// Raw data volume of one optical scene.
pub fn scene_data_volume(
    pixels_across: f64,
    pixels_along: f64,
    bits_per_pixel: f64,
    bands: f64,
    compression_ratio: f64,
) -> DataVolume {
    DataVolume::new(pixels_across * pixels_along * bits_per_pixel * bands / compression_ratio)
}

// --- synthetic aperture radar -------------------------------------------------

/// Slant range resolution, `c/(2·B)`.
pub fn sar_range_resolution(bandwidth: Frequency) -> Length {
    Length::new(SPEED_OF_LIGHT.get() / (2.0 * bandwidth.get()))
}

/// Azimuth resolution of a stripmap SAR, `D_a/2` — independent of range, which
/// is the property that makes SAR different from every other sensor here.
pub fn sar_azimuth_resolution(antenna_length: Length) -> Length {
    Length::new(antenna_length.get() / 2.0)
}

/// Noise-equivalent sigma-zero — the radar's sensitivity floor.
///
/// Proportional to the cube of range for a stripmap SAR at fixed resolution.
/// Dropping from 500 km to 250 km is an eightfold improvement, or the same
/// image for one eighth of the transmit power.
#[allow(clippy::too_many_arguments)] // the relation has this many terms; naming them all is the point
pub fn sar_nesz(
    transmit_power: Power,
    antenna_gain_db: f64,
    range: Length,
    wavelength: Length,
    system_noise_temperature: Temperature,
    noise_bandwidth: Frequency,
    azimuth_resolution: Length,
    platform_velocity: Velocity,
    losses_db: f64,
) -> f64 {
    let g = crate::physics::comms::from_db(antenna_gain_db);
    let l = crate::physics::comms::from_db(losses_db);
    let r = range.get();
    let numerator = 4.0
        * pmath::powi(4.0 * pmath::PI, 2)
        * r
        * r
        * r
        * platform_velocity.get()
        * K_BOLTZMANN
        * system_noise_temperature.get()
        * noise_bandwidth.get()
        * l;
    let denominator = transmit_power.get()
        * g
        * g
        * wavelength.get()
        * wavelength.get()
        * wavelength.get()
        * azimuth_resolution.get();
    numerator / denominator
}

// --- radio-frequency geolocation ---------------------------------------------

/// Geolocation error from time-difference-of-arrival between two receivers.
///
/// ```text
/// sigma_pos = c·sigma_tau·GDOP
/// ```
///
/// The timing uncertainty is dominated by the signal's bandwidth and the
/// signal-to-noise ratio; the geometric dilution is dominated by the baseline
/// between the receivers relative to their range — which is exactly the term
/// VLEO improves, because the same formation baseline subtends a larger angle
/// from closer in.
pub fn tdoa_geolocation_error(timing_uncertainty: Time, gdop: f64) -> Length {
    Length::new(SPEED_OF_LIGHT.get() * timing_uncertainty.get() * gdop)
}

/// Cramér-Rao lower bound on time-of-arrival estimation.
///
/// ```text
/// sigma_tau = 1/(2·pi·B·sqrt(2·SNR·B·T))
/// ```
pub fn toa_timing_uncertainty(bandwidth: Frequency, snr_linear: f64, integration: Time) -> Time {
    let b = bandwidth.get();
    Time::new(1.0 / (2.0 * pmath::PI * b * pmath::sqrt(2.0 * snr_linear * b * integration.get())))
}

/// Geolocation error from frequency-difference-of-arrival.
///
/// A Doppler measurement locates an emitter along an isodoppler contour. The
/// position error for a frequency uncertainty `sigma_f` is
///
/// ```text
/// sigma_pos = (lambda·sigma_f/v_rel)·R·GDOP
/// ```
///
/// where `lambda·sigma_f` is the equivalent line-of-sight velocity error,
/// dividing by the relative velocity turns it into an angle, and multiplying by
/// the range turns the angle into metres on the ground.
pub fn fdoa_geolocation_error(
    frequency_uncertainty: Frequency,
    carrier: Frequency,
    relative_velocity: Velocity,
    range: Length,
    gdop: f64,
) -> Length {
    let lambda = SPEED_OF_LIGHT.get() / carrier.get();
    let velocity_error = lambda * frequency_uncertainty.get();
    let angular_error = velocity_error / relative_velocity.get();
    Length::new(angular_error * range.get() * gdop)
}

/// Detection range for a receiver against an emitter of known EIRP.
pub fn intercept_range(
    emitter_eirp_dbw: f64,
    receive_gain_db: f64,
    frequency: Frequency,
    sensitivity_dbw: f64,
) -> Length {
    let lambda = SPEED_OF_LIGHT.get() / frequency.get();
    let path_loss_allowed = emitter_eirp_dbw + receive_gain_db - sensitivity_dbw;
    let linear = crate::physics::comms::from_db(path_loss_allowed);
    Length::new(lambda * pmath::sqrt(linear) / (4.0 * pmath::PI))
}

// --- positioning, navigation and timing ---------------------------------------

/// User range error contributed by a signal at a given chip rate and
/// carrier-to-noise density.
pub fn pnt_ranging_error(
    chip_rate: Frequency,
    cn0_db_hz: f64,
    loop_bandwidth: Frequency,
) -> Length {
    let cn0 = crate::physics::comms::from_db(cn0_db_hz);
    let chip_length = SPEED_OF_LIGHT.get() / chip_rate.get();
    Length::new(chip_length * pmath::sqrt(loop_bandwidth.get() / (2.0 * cn0)))
}

/// Position error from ranging error and geometry.
pub fn pnt_position_error(user_range_error: Length, pdop: f64) -> Length {
    Length::new(user_range_error.get() * pdop)
}
