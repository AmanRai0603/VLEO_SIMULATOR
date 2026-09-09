//! Guidance, navigation and control.
//!
//! The VLEO difference, stated once: aerodynamic torque dominates. In a
//! 700 km orbit gravity gradient and solar pressure are the disturbances a
//! design sizes against; at 200 km the aerodynamic torque is one to three
//! orders of magnitude larger than either, it acts along-track continuously,
//! and it cannot be dumped magnetically as conveniently as a periodic torque
//! can. A control design carried over from a higher orbit is undersized here,
//! and the error is quiet until the wheels saturate.

use vleo_units::constants::*;
use vleo_units::pmath;
use vleo_units::*;

/// Gravity-gradient torque on a body with principal inertias, at pitch angle
/// `theta` from local vertical.
///
/// ```text
/// T = (3·mu/(2·r^3))·|Iz - Iy|·sin(2·theta)
/// ```
pub fn gravity_gradient_torque(
    radius: Length,
    inertia_max: f64,
    inertia_min: f64,
    theta: Angle,
) -> Torque {
    let r = radius.get();
    Torque::new(
        1.5 * MU_EARTH / (r * r * r)
            * pmath::abs(inertia_max - inertia_min)
            * pmath::sin(2.0 * theta.get()),
    )
}

/// Solar radiation pressure torque, from the illuminated area, its reflectivity
/// and the centre-of-pressure offset.
pub fn solar_pressure_torque(
    area: Area,
    reflectivity: Ratio,
    cp_offset: Length,
    incidence: Angle,
) -> Torque {
    let p = SOLAR_CONSTANT.get() / SPEED_OF_LIGHT.get();
    let cos = pmath::max(0.0, incidence.cos());
    Torque::new(p * area.get() * (1.0 + reflectivity.get()) * cos * cp_offset.get())
}

/// Torque from a residual magnetic dipole in the geomagnetic field.
pub fn magnetic_torque(residual_dipole: DipoleMoment, field: MagneticFluxDensity) -> Torque {
    field * residual_dipole
}

/// Worst-case sum of the disturbance torques.
///
/// Summed rather than root-sum-squared, deliberately. These are not
/// independent random errors — they are biases that can and do align, and a
/// wheel sized on the RSS of them saturates the first time they do.
pub fn total_disturbance_torque(
    aerodynamic: Torque,
    gravity_gradient: Torque,
    solar: Torque,
    magnetic: Torque,
) -> Torque {
    Torque::new(
        aerodynamic.get().abs()
            + gravity_gradient.get().abs()
            + solar.get().abs()
            + magnetic.get().abs(),
    )
}

/// Angular momentum a wheel must store if a secular torque acts for half an
/// orbit before it can be dumped.
///
/// Half an orbit rather than a whole one because a torque that reverses sign
/// with the orbit averages out; an along-track aerodynamic torque largely does
/// not, which is why this is the pessimistic and correct choice in VLEO.
pub fn momentum_storage_required(
    secular_torque: Torque,
    orbital_period: Time,
    margin: Ratio,
) -> AngularMomentum {
    secular_torque * Time::new(0.5 * orbital_period.get()) * (1.0 + margin.get())
}

/// Magnetic dipole a torque rod needs to dump a given momentum in a given time.
pub fn magnetorquer_dipole_required(
    momentum: AngularMomentum,
    field: MagneticFluxDensity,
    dump_time: Time,
) -> DipoleMoment {
    DipoleMoment::new(momentum.get() / (field.get() * dump_time.get()))
}

/// Slew time for a rest-to-rest manoeuvre, bang-bang at constant torque.
pub fn slew_time(angle: Angle, inertia: f64, control_torque: Torque) -> Time {
    Time::new(2.0 * pmath::sqrt(inertia * angle.get() / control_torque.get()))
}

/// Root-sum-square of a pointing error budget.
///
/// The opposite convention to the disturbance sum above, and for the opposite
/// reason: these terms are genuinely independent — sensor noise, thermal
/// distortion, alignment, control error — so summing them linearly would be a
/// budget nothing could ever meet.
pub fn pointing_error_rss(terms: &[Angle]) -> Angle {
    let mut s = 0.0;
    for t in terms {
        s += t.get() * t.get();
    }
    Angle::new(pmath::sqrt(s))
}

/// Ground position error from a pointing error at a given slant range.
///
/// The number a geolocation requirement actually lands on: a milliradian at
/// 300 km is 300 metres, and a customer's accuracy requirement is in metres.
pub fn pointing_to_ground_error(pointing_error: Angle, slant_range: Length) -> Length {
    Length::new(pointing_error.get() * slant_range.get())
}

/// Position accuracy of a GNSS receiver in orbit, from the user range error and
/// the geometry.
pub fn navigation_position_error(user_range_error: Length, gdop: f64) -> Length {
    Length::new(user_range_error.get() * gdop)
}

/// Velocity error from a carrier-phase or Doppler solution.
pub fn navigation_velocity_error(range_rate_error: Velocity, gdop: f64) -> Velocity {
    Velocity::new(range_rate_error.get() * gdop)
}

/// Along-track position error growth from an unmodelled drag acceleration
/// error, over a propagation interval.
///
/// Quadratic in time, and it is the term that dominates orbit prediction in
/// VLEO. A 15% density model error at 250 km is roughly a kilometre of
/// along-track error after one day — which is why the density uncertainty is a
/// declared output rather than a footnote.
pub fn along_track_error_from_drag(
    drag_acceleration_error: Acceleration,
    duration: Time,
) -> Length {
    let t = duration.get();
    Length::new(1.5 * drag_acceleration_error.get() * t * t)
}
