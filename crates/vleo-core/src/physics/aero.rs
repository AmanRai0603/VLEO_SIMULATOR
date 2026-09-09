//! Free-molecular aerodynamics and gas-surface interaction.
//!
//! Everything here assumes free-molecular flow: the mean free path is much
//! larger than the vehicle, so a molecule that leaves the surface never comes
//! back and never collides with an incoming one. That holds above roughly
//! 150 km and it is why `Kn` is checked by a guard rather than assumed — the
//! Ariane 501 failure was a component used outside the envelope it was
//! specified for, and nothing was re-derived.
//!
//! The drag coefficient of a satellite in VLEO is **not** a constant. It
//! depends on the speed ratio, the surface temperature, how much the incoming
//! atomic oxygen accommodates to the surface, and the shape. Treating it as a
//! constant 2.2 is the single largest avoidable error in a VLEO drag estimate,
//! and it is what most concept studies do.

use crate::physics::env;
use vleo_units::constants::*;
use vleo_units::pmath;
use vleo_units::*;

/// Molecular speed ratio, `s = V / sqrt(2·R·T/M)` — the ratio of the bulk speed
/// to the most probable thermal speed. In VLEO it is around 7 to 9, which is
/// what "hyperthermal" means: the gas arrives as a beam, not as a cloud.
pub fn speed_ratio(velocity: Velocity, t: Temperature, molar_mass: MolarMass) -> Ratio {
    Ratio::new(velocity.get() / env::most_probable_speed(t, molar_mass).get())
}

/// Sentman's free-molecular drag and lift coefficients for a flat surface
/// element, in the form given by Doornbos (2011) §3.2.
///
/// * `s` — molecular speed ratio
/// * `theta` — angle between the surface normal and the flow
/// * `alpha` — energy accommodation coefficient, `0` specular to `1` full
///   thermal accommodation
/// * `t_wall` — surface temperature
/// * `velocity` — bulk flow speed
///
/// ```text
/// gamma = cos(theta),  ell = sin(theta)
/// P = exp(-(gamma·s)^2)/s
/// G = 1/(2 s^2),  Q = 1 + G,  Z = 1 + erf(gamma·s)
/// vr = sqrt( 0.5·(1 + alpha·(4·R·T_w/(M·V^2) - 1)) )
/// Cd = P/sqrt(pi) + gamma·Q·Z + (gamma/2)·vr·(gamma·sqrt(pi)·Z + P)
/// Cl = ell·G·Z    + (ell/2)  ·vr·(gamma·sqrt(pi)·Z + P)
/// ```
///
/// Coefficients are referred to the *element's own* area, so a body is the
/// area-weighted sum of its panels projected onto the flow — which is what
/// [`plate_drag_coefficient`] does.
pub fn sentman_panel(
    s: f64,
    theta: Angle,
    alpha: f64,
    t_wall: Temperature,
    velocity: Velocity,
    molar_mass: MolarMass,
) -> (f64, f64) {
    let (ell, gamma) = theta.sin_cos();
    let p = pmath::exp(-(gamma * s) * (gamma * s)) / s;
    let g = 1.0 / (2.0 * s * s);
    let q = 1.0 + g;
    let z = 1.0 + pmath::erf(gamma * s);
    let v2 = velocity.get() * velocity.get();
    let vr = pmath::sqrt(
        0.5 * (1.0 + alpha * (4.0 * R_UNIVERSAL * t_wall.get() / (molar_mass.get() * v2) - 1.0)),
    );
    let common = gamma * pmath::sqrt(pmath::PI) * z + p;
    let cd = p / pmath::sqrt(pmath::PI) + gamma * q * z + 0.5 * gamma * vr * common;
    let cl = ell * g * z + 0.5 * ell * vr * common;
    (cd, cl)
}

/// Drag coefficient of a flat plate normal to the flow — the reference shape,
/// and the one an intake mouth actually is.
pub fn plate_drag_coefficient(
    s: f64,
    alpha: f64,
    t_wall: Temperature,
    velocity: Velocity,
    molar_mass: MolarMass,
) -> f64 {
    sentman_panel(s, Angle::new(0.0), alpha, t_wall, velocity, molar_mass).0
}

/// Drag coefficient of a right circular cylinder of length `l` and diameter `d`
/// flying axially — a serviceable model for a slender VLEO bus.
///
/// Front face at incidence zero, side walls at grazing incidence. The side
/// walls contribute little drag and most of the *lift* sensitivity, which is
/// why they are carried rather than dropped.
pub fn cylinder_drag_coefficient(
    s: f64,
    length: Length,
    diameter: Length,
    alpha: f64,
    t_wall: Temperature,
    velocity: Velocity,
    molar_mass: MolarMass,
) -> f64 {
    let a_front = 0.25 * pmath::PI * diameter.get() * diameter.get();
    let a_side = pmath::PI * diameter.get() * length.get();
    let (cd_front, _) = sentman_panel(s, Angle::new(0.0), alpha, t_wall, velocity, molar_mass);
    let (cd_side, _) = sentman_panel(
        s,
        Angle::new(pmath::FRAC_PI_2 * 0.98),
        alpha,
        t_wall,
        velocity,
        molar_mass,
    );
    // Referred back to the frontal area, which is what the drag relation uses.
    (cd_front * a_front + cd_side * a_side * 0.5) / a_front
}

/// Energy accommodation coefficient from Langmuir adsorption of atomic oxygen.
///
/// Moe & Moe (2005): a surface in VLEO is covered in adsorbed atomic oxygen,
/// and how completely it is covered decides how completely an incoming molecule
/// accommodates. The coverage follows the local atomic-oxygen number density:
///
/// ```text
/// alpha = 7.5e-17·n_O·T_inf / (1 + 7.5e-17·n_O·T_inf)
/// ```
///
/// The consequence is that accommodation, and therefore drag, changes with
/// altitude and with solar activity even for an unchanged spacecraft. A design
/// that assumes a fixed `alpha` has a margin that moves when the Sun does.
pub fn accommodation_coefficient(n_atomic_oxygen: NumberDensity, t_inf: Temperature) -> f64 {
    let k = 7.5e-17 * n_atomic_oxygen.get() * t_inf.get();
    k / (1.0 + k)
}

/// Dynamic pressure, `0.5·rho·V^2`.
pub fn dynamic_pressure(density: MassDensity, velocity: Velocity) -> Pressure {
    Pressure::new(0.5 * density.get() * velocity.get() * velocity.get())
}

/// Drag force, `0.5·rho·V^2·Cd·A`.
pub fn drag_force(
    density: MassDensity,
    velocity: Velocity,
    drag_coefficient: f64,
    reference_area: Area,
) -> Force {
    Force::new(
        dynamic_pressure(density, velocity).get() * drag_coefficient * reference_area.get(),
    )
}

/// Lift force, same reference area and dynamic pressure.
pub fn lift_force(
    density: MassDensity,
    velocity: Velocity,
    lift_coefficient: f64,
    reference_area: Area,
) -> Force {
    Force::new(
        dynamic_pressure(density, velocity).get() * lift_coefficient * reference_area.get(),
    )
}

/// Ballistic coefficient, `m/(Cd·A)`, in kg/m^2.
///
/// Higher is better in VLEO: it is the inverse of how hard the atmosphere pulls
/// on a given mass. A cubesat is around 50; a slender, drag-optimised VLEO
/// platform reaches 150 or more, and that difference is worth tens of
/// kilometres of altitude.
pub fn ballistic_coefficient(mass: Mass, drag_coefficient: f64, reference_area: Area) -> f64 {
    mass.get() / (drag_coefficient * reference_area.get())
}

/// Drag deceleration, `D/m`.
pub fn drag_acceleration(drag: Force, mass: Mass) -> Acceleration {
    drag / mass
}

/// Aerodynamic torque from the offset between the centre of pressure and the
/// centre of mass. In VLEO this is the dominant disturbance torque by an order
/// of magnitude, and it is the reason a VLEO attitude design starts from
/// aerodynamics rather than from gravity gradient.
pub fn aerodynamic_torque(drag: Force, cp_cm_offset: Length) -> Torque {
    drag * cp_cm_offset
}

/// Atomic oxygen fluence over a mission, in atoms per square metre.
///
/// The number a materials selection is made against: polyimide erodes at about
/// 3e-24 cubic centimetres per incident atom, so a fluence of 1e22 removes
/// roughly 30 micrometres of it.
pub fn atomic_oxygen_fluence(
    n_atomic_oxygen: NumberDensity,
    velocity: Velocity,
    duration: Time,
) -> f64 {
    n_atomic_oxygen.get() * velocity.get() * duration.get()
}
