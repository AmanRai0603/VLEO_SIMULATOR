//! Orbital mechanics.
//!
//! Two-body with the J2 secular terms. Nothing here integrates a trajectory —
//! this is a design tool, and a design tool answers "what does this altitude
//! cost me" rather than "where will it be on Tuesday". The propagator lives
//! behind the campaign face, and it consumes these same relations.

use vleo_units::constants::*;
use vleo_units::pmath;
use vleo_units::*;

/// Geocentric radius from altitude above the WGS-84 equatorial radius.
pub fn radius(altitude: Length) -> Length {
    Length::new(R_EARTH.get() + altitude.get())
}

/// Altitude from geocentric radius — the inverse, written once so no node does
/// the subtraction with a different Earth radius.
pub fn altitude(radius: Length) -> Length {
    Length::new(radius.get() - R_EARTH.get())
}

/// Circular orbital speed, `sqrt(mu/r)`.
///
/// At 200 km this is 7.784 km/s and at 400 km it is 7.669 km/s. The difference
/// looks small and is not: drag goes as the square of it, and density between
/// those two altitudes changes by a factor of several hundred.
pub fn circular_velocity(radius: Length) -> Velocity {
    Velocity::new(pmath::sqrt(MU_EARTH / radius.get()))
}

/// Speed on an elliptical orbit at radius `r`, from the vis-viva equation.
pub fn vis_viva(radius: Length, semi_major_axis: Length) -> Velocity {
    Velocity::new(pmath::sqrt(
        MU_EARTH * (2.0 / radius.get() - 1.0 / semi_major_axis.get()),
    ))
}

/// Orbital period, `2·pi·sqrt(a^3/mu)`.
pub fn period(semi_major_axis: Length) -> Time {
    let a = semi_major_axis.get();
    Time::new(pmath::TAU * pmath::sqrt(a * a * a / MU_EARTH))
}

/// Mean motion, `sqrt(mu/a^3)`.
pub fn mean_motion(semi_major_axis: Length) -> AngularRate {
    let a = semi_major_axis.get();
    AngularRate::new(pmath::sqrt(MU_EARTH / (a * a * a)))
}

/// Apoapsis and periapsis radii from `a` and `e`.
pub fn apsides(semi_major_axis: Length, eccentricity: f64) -> (Length, Length) {
    let a = semi_major_axis.get();
    (
        Length::new(a * (1.0 + eccentricity)),
        Length::new(a * (1.0 - eccentricity)),
    )
}

/// Speed of the sub-satellite point over the ground.
///
/// Not the orbital speed: the ground track moves more slowly, in the ratio of
/// the Earth's radius to the orbital radius, and the difference is what sets
/// how long a target stays in a sensor's field of view.
pub fn ground_track_speed(orbital_speed: Velocity, radius: Length) -> Velocity {
    Velocity::new(orbital_speed.get() * R_EARTH.get() / radius.get())
}

/// Secular rate of change of right ascension of the ascending node, from the
/// second zonal harmonic.
///
/// ```text
/// dOmega/dt = -1.5 · J2 · sqrt(mu) · Re^2 · cos(i) / ((1-e^2)^2 · a^(7/2))
/// ```
///
/// Negative for a prograde orbit — the node regresses. This is the relation a
/// Sun-synchronous orbit is designed against, and the one that decides whether
/// a constellation's planes stay where they were put.
pub fn nodal_regression(semi_major_axis: Length, eccentricity: f64, inclination: Angle) -> AngularRate {
    let a = semi_major_axis.get();
    let p = (1.0 - eccentricity * eccentricity) * (1.0 - eccentricity * eccentricity);
    let re = R_EARTH.get();
    AngularRate::new(
        -1.5 * J2_EARTH * pmath::sqrt(MU_EARTH) * re * re * inclination.cos()
            / (p * pmath::powf(a, 3.5)),
    )
}

/// Mean rate at which the Sun's right ascension advances, one revolution per
/// tropical year. A Sun-synchronous orbit is one whose nodal regression matches
/// it.
pub const SUN_RA_RATE: AngularRate = AngularRate::new(1.991_063_853e-7);

/// The inclination at which nodal regression equals the Sun's apparent motion.
///
/// Returns `None` when no such inclination exists — which happens above roughly
/// 5 970 km, and is a real answer rather than a failure: the caller raises the
/// fault, because only the caller knows whether being asked was reasonable.
pub fn sun_synchronous_inclination(semi_major_axis: Length, eccentricity: f64) -> Option<Angle> {
    let a = semi_major_axis.get();
    let p = (1.0 - eccentricity * eccentricity) * (1.0 - eccentricity * eccentricity);
    let re = R_EARTH.get();
    let cos_i = -SUN_RA_RATE.get() * p * pmath::powf(a, 3.5)
        / (1.5 * J2_EARTH * pmath::sqrt(MU_EARTH) * re * re);
    if cos_i < -1.0 || cos_i > 1.0 {
        return None;
    }
    Some(Angle::new(pmath::acos(cos_i)))
}

/// Half-angle of the Earth as seen from orbit, `asin(Re/r)`.
pub fn earth_angular_radius(radius: Length) -> Angle {
    Angle::new(pmath::asin(R_EARTH.get() / radius.get()))
}

/// Earth-central angle to the horizon at a given minimum elevation.
///
/// The geometry every access, swath and contact-time calculation rests on:
///
/// ```text
/// lambda = acos( (Re/r)·cos(eps) ) - eps
/// ```
pub fn earth_central_angle(radius: Length, min_elevation: Angle) -> Angle {
    let ratio = R_EARTH.get() / radius.get();
    Angle::new(pmath::acos(ratio * min_elevation.cos()) - min_elevation.get())
}

/// Slant range to a point at the edge of the access circle.
pub fn slant_range(radius: Length, min_elevation: Angle) -> Length {
    let lambda = earth_central_angle(radius, min_elevation);
    let re = R_EARTH.get();
    let r = radius.get();
    Length::new(pmath::sqrt(
        re * re + r * r - 2.0 * re * r * lambda.cos(),
    ))
}

/// Ground swath width for a given Earth-central half-angle.
pub fn swath_width(earth_central_angle: Angle) -> Length {
    Length::new(2.0 * earth_central_angle.get() * R_EARTH_MEAN.get())
}

/// Fraction of an orbit spent in the Earth's shadow, cylindrical model.
///
/// `beta` is the angle between the orbit plane and the Sun direction. Above the
/// critical beta the orbit is fully sunlit and the fraction is zero — which is
/// the case a thermal design must not be sized on, and the case a power design
/// must not be sized on either, for opposite reasons.
pub fn eclipse_fraction(radius: Length, beta: Angle) -> Ratio {
    let re = R_EARTH.get();
    let r = radius.get();
    let cb = pmath::abs(beta.cos());
    // Below this beta the shadow cylinder is missed entirely.
    let beta_star = pmath::asin(re / r);
    if pmath::abs(beta.get()) >= pmath::FRAC_PI_2 - beta_star + 1.0e-12 && cb * r <= re {
        // Fully shadowed geometry does not arise for a circular LEO orbit.
    }
    let arg = pmath::sqrt(r * r - re * re) / (r * cb);
    if arg >= 1.0 {
        return Ratio::new(0.0); // full-sun orbit
    }
    Ratio::new(pmath::acos(arg) / pmath::PI)
}

/// Rate of change of semi-major axis under drag, for a near-circular orbit.
///
/// ```text
/// da/dt = -rho · (Cd·A/m) · sqrt(mu·a)
/// ```
///
/// Negative always. This is the number that decides whether a VLEO mission is a
/// mission or an experiment: at 200 km an uncompensated satellite of ordinary
/// ballistic coefficient loses its orbit in days.
pub fn decay_rate(
    density: MassDensity,
    ballistic_coefficient: f64,
    semi_major_axis: Length,
) -> Velocity {
    // Ballistic coefficient here is m/(Cd·A) in kg/m^2, so its reciprocal is
    // the area-to-mass term the relation wants.
    Velocity::new(
        -density.get() * pmath::sqrt(MU_EARTH * semi_major_axis.get()) / ballistic_coefficient,
    )
}

/// Orbital lifetime estimate, by integrating the decay rate at constant
/// atmospheric conditions.
///
/// Reported as an order of magnitude, never as a date. It assumes the density
/// at each altitude does not change while the satellite passes through it,
/// which over a solar cycle is wrong by a factor of several — and that is the
/// honest accuracy of any lifetime number that is not a Monte Carlo over solar
/// activity.
pub fn lifetime_estimate<F>(
    start_altitude: Length,
    end_altitude: Length,
    ballistic_coefficient: f64,
    steps: u32,
    density_at: F,
) -> Time
where
    F: Fn(Length) -> MassDensity,
{
    let h0 = start_altitude.get();
    let h1 = end_altitude.get();
    let dh = (h1 - h0) / steps as f64;
    let mut t = 0.0;
    for i in 0..steps {
        let h = h0 + dh * (i as f64 + 0.5);
        let a = radius(Length::new(h));
        let rho = density_at(Length::new(h)).get();
        let da_dt = -rho * pmath::sqrt(MU_EARTH * a.get()) / ballistic_coefficient;
        if da_dt >= 0.0 {
            continue;
        }
        t += dh / da_dt;
    }
    Time::new(pmath::abs(t))
}

/// Delta-v needed to hold an altitude against a drag acceleration for a
/// duration. Exact for a low-thrust, continuously-compensated orbit: the drag
/// impulse is the impulse that has to be replaced.
pub fn drag_makeup_delta_v(drag_acceleration: Acceleration, duration: Time) -> Velocity {
    drag_acceleration * duration
}

/// Delta-v for a Hohmann transfer between two circular orbits.
pub fn hohmann_delta_v(r1: Length, r2: Length) -> Velocity {
    let (r1, r2) = (r1.get(), r2.get());
    let a_t = 0.5 * (r1 + r2);
    let v1 = pmath::sqrt(MU_EARTH / r1);
    let v2 = pmath::sqrt(MU_EARTH / r2);
    let vp = pmath::sqrt(MU_EARTH * (2.0 / r1 - 1.0 / a_t));
    let va = pmath::sqrt(MU_EARTH * (2.0 / r2 - 1.0 / a_t));
    Velocity::new(pmath::abs(vp - v1) + pmath::abs(v2 - va))
}

/// Delta-v for a pure plane change at a given speed.
pub fn plane_change_delta_v(speed: Velocity, angle: Angle) -> Velocity {
    Velocity::new(2.0 * speed.get() * pmath::sin(0.5 * angle.get()))
}

/// Delta-v to lower periapsis to a target re-entry altitude — the disposal
/// manoeuvre ISO 24113 requires be budgeted at design time rather than found
/// at the end.
pub fn deorbit_delta_v(from_altitude: Length, target_perigee_altitude: Length) -> Velocity {
    let r1 = radius(from_altitude);
    let r2 = radius(target_perigee_altitude);
    let a_t = Length::new(0.5 * (r1.get() + r2.get()));
    let v_circ = circular_velocity(r1);
    let v_transfer = vis_viva(r1, a_t);
    Velocity::new(pmath::abs(v_circ.get() - v_transfer.get()))
}

/// Rocket equation, solved for propellant mass.
pub fn propellant_mass(dry_mass: Mass, delta_v: Velocity, exhaust_velocity: Velocity) -> Mass {
    Mass::new(dry_mass.get() * (pmath::exp(delta_v.get() / exhaust_velocity.get()) - 1.0))
}

/// Exhaust velocity from specific impulse, `v_e = Isp·g0`.
pub fn exhaust_velocity(specific_impulse: Time) -> Velocity {
    Velocity::new(specific_impulse.get() * G0.get())
}

/// Specific impulse from exhaust velocity.
pub fn specific_impulse(exhaust_velocity: Velocity) -> Time {
    Time::new(exhaust_velocity.get() / G0.get())
}
