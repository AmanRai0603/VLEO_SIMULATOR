//! The space environment — and the part of it this programme lives or dies on.
//!
//! VLEO means roughly 200 to 450 km. The air there is thick enough to drag a
//! satellite down and thin enough to fly through, and the density spans four
//! orders of magnitude across that band and another order with solar activity.
//! No published model is better than about 15% in this regime, which is why the
//! uncertainty is carried explicitly rather than assumed away.
//!
//! # The model
//!
//! Diffusive equilibrium above a 120 km base, with a Bates temperature profile
//! — the structure Jacchia's models and every descendant use. For each species
//! independently:
//!
//! ```text
//! n_i(z) = n_i(z0) · (T(z0)/T(z))^(1+alpha_i) · exp( -INT_{z0}^{z} m_i·g(z)/(k·T(z)) dz )
//! T(z)   = T_inf - (T_inf - T_120)·exp(-s·(z - z0))
//! ```
//!
//! The exospheric temperature `T_inf` carries solar and geomagnetic activity.
//! The integral is evaluated by Runge-Kutta over altitude rather than by a
//! closed form, because the closed form only exists if `g` is held constant,
//! and holding `g` constant is a 4% error at 400 km.
//!
//! # What this is not
//!
//! It is not NRLMSISE-00 and does not claim to be. It reproduces the structure
//! and the activity dependence, and it is the node's stated model — every
//! result computed from it carries that in its provenance. Fitted corrections
//! against measured drag data arrive as a reference-data bundle, not as a
//! change to this file.

use crate::math::integrate::rk4_scalar;
use vleo_units::constants::*;
use vleo_units::pmath;
use vleo_units::*;

/// Base altitude of the diffusive-equilibrium integration, in metres.
pub const Z_BASE: f64 = 120_000.0;
/// Temperature at the base. Jacchia 1971 uses 355 K; 360 K is the value used
/// throughout the models this chain is compared against.
pub const T_BASE: Temperature = Temperature::new(360.0);
/// Bates shape parameter, per metre. Controls how quickly the profile
/// approaches the exospheric temperature.
pub const BATES_S: f64 = 2.0e-5;

/// Number densities at the 120 km base, per cubic metre.
///
/// Climatological means. Total is 5.61e17 /m^3, which reproduces the US
/// Standard Atmosphere 1976 mass density of 2.44e-8 kg/m^3 at that altitude.
pub mod base_density {
    /// Molecular nitrogen — 63.4% of the base, and the species that dominates
    /// below about 180 km.
    pub const N2: f64 = 3.556e17;
    /// Molecular oxygen — 15.7%.
    pub const O2: f64 = 8.808e16;
    /// Atomic oxygen — 20.3% at the base, and dominant above roughly 200 km.
    /// This is the species that makes VLEO a different problem: it erodes
    /// polymers, it is what an air-breathing intake actually collects, and its
    /// scale height is what sets the shape of the whole band.
    pub const O: f64 = 1.139e17;
    /// Argon — 0.6%.
    pub const AR: f64 = 3.366e15;
    /// Helium — a trace here, dominant far above.
    pub const HE: f64 = 3.366e13;
}

/// Thermal diffusion coefficient per species. Only helium is materially
/// non-zero at these altitudes.
const ALPHA_N2: f64 = 0.0;
const ALPHA_O2: f64 = 0.0;
const ALPHA_O: f64 = 0.0;
const ALPHA_AR: f64 = 0.0;
const ALPHA_HE: f64 = -0.38;

/// Convert the planetary amplitude `Ap` to the planetary index `Kp`.
///
/// The two indices measure the same thing on different scales — `Ap` is
/// quasi-linear in field disturbance, `Kp` is quasi-logarithmic — and the
/// conversion is a published lookup, not a formula. Solar-driver bundles are
/// distributed in `Ap`; Jacchia's temperature correction is stated in `Kp`.
/// Mixing them is not a small error: `0.03·exp(Ap)` at a storm-time `Ap` of 100
/// is `10^42`, and the model returns an exospheric temperature of `10^18` K
/// without complaining. That is exactly the class of silent, plausible-looking
/// failure a guard exists to prevent, so the conversion is a named function
/// with a table rather than an assumption inside a caller.
pub fn kp_from_ap(ap: f64) -> f64 {
    const AP: &[f64] = &[
        0.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 9.0, 12.0, 15.0, 18.0, 22.0, 27.0, 32.0, 39.0, 48.0,
        56.0, 67.0, 80.0, 94.0, 111.0, 132.0, 154.0, 179.0, 207.0, 236.0, 300.0, 400.0,
    ];
    const KP: &[f64] = &[
        0.0, 0.33, 0.67, 1.0, 1.33, 1.67, 2.0, 2.33, 2.67, 3.0, 3.33, 3.67, 4.0, 4.33, 4.67, 5.0,
        5.33, 5.67, 6.0, 6.33, 6.67, 7.0, 7.33, 7.67, 8.0, 8.33, 8.67, 9.0,
    ];
    pmath::interp(ap, AP, KP)
}

/// Exospheric temperature from solar and geomagnetic activity.
///
/// Jacchia 1971, night-time minimum, with the geomagnetic correction:
///
/// ```text
/// T_c   = 379 + 3.24·F10.7A + 1.3·(F10.7 - F10.7A)
/// dT_kp = 28·Kp + 0.03·exp(Kp)
/// ```
///
/// `f107a` is the 81-day centred mean and `f107` the daily value, both in solar
/// flux units. `kp` is the planetary index, `0..=9` — **not** `Ap`; use
/// [`kp_from_ap`] if a bundle gave you the amplitude.
///
/// The two solar terms are separated because they do different work: the 81-day
/// mean sets where the atmosphere sits, and the daily value sets how far it is
/// from there today. A model given only one of them cannot tell a quiet day in
/// an active year from an active day in a quiet one, and those have very
/// different densities.
pub fn exospheric_temperature(f107: f64, f107a: f64, kp: f64) -> Temperature {
    let t_c = 379.0 + 3.24 * f107a + 1.3 * (f107 - f107a);
    let d_kp = 28.0 * kp + 0.03 * pmath::exp(kp);
    Temperature::new(t_c + d_kp)
}

/// Bates temperature profile. Approaches `t_inf` asymptotically from `T_BASE`.
pub fn temperature(altitude: Length, t_inf: Temperature) -> Temperature {
    let dz = altitude.get() - Z_BASE;
    if dz <= 0.0 {
        return T_BASE;
    }
    let t = t_inf.get() - (t_inf.get() - T_BASE.get()) * pmath::exp(-BATES_S * dz);
    Temperature::new(t)
}

/// Local gravitational acceleration at altitude, inverse-square. Held as a
/// function rather than a constant because treating `g` as constant to 400 km
/// is a 4% error in the exponent of the barometric integral, which is a 20%
/// error in density at the top of the band.
pub fn gravity(altitude: Length) -> Acceleration {
    let r = R_EARTH.get() + altitude.get();
    Acceleration::new(MU_EARTH / (r * r))
}

/// Number density of one species at altitude, by diffusive equilibrium.
///
/// `molar_mass` selects the species; `n_base` and `alpha` are its base density
/// and thermal diffusion coefficient.
pub fn species_number_density(
    altitude: Length,
    t_inf: Temperature,
    n_base: f64,
    molar_mass: MolarMass,
    alpha: f64,
) -> NumberDensity {
    let z = altitude.get();
    if z <= Z_BASE {
        return NumberDensity::new(n_base);
    }
    // d(integral)/dz = M·g(z) / (R·T(z))   [1/m], integrated by RK4 in altitude.
    // Molar mass over the universal gas constant, rather than particle mass
    // over Boltzmann: the two are the same ratio, and the molar form is the one
    // the source states.
    let integral = rk4_scalar(0.0, Z_BASE, z, integration_steps(z), |zz, _| {
        let g = gravity(Length::new(zz)).get();
        let t = temperature(Length::new(zz), t_inf).get();
        molar_mass.get() * g / (R_UNIVERSAL * t)
    });
    let t_z = temperature(altitude, t_inf).get();
    let ratio = T_BASE.get() / t_z;
    NumberDensity::new(n_base * pmath::powf(ratio, 1.0 + alpha) * pmath::exp(-integral))
}

/// One integration step per kilometre, so the answer does not depend on where
/// the caller happens to be asking. Bounded so a nonsense altitude cannot spin.
fn integration_steps(z: f64) -> u32 {
    let km = (z - Z_BASE) / 1000.0;
    let s = km as u32;
    s.clamp(8, 2000)
}

/// Every species at once. Returned as a struct rather than a tuple because a
/// five-tuple of densities is exactly the shape a caller mixes up.
#[derive(Clone, Copy, Debug)]
pub struct Composition {
    pub n2: NumberDensity,
    pub o2: NumberDensity,
    pub o: NumberDensity,
    pub ar: NumberDensity,
    pub he: NumberDensity,
}

impl Composition {
    /// Total number density.
    pub fn total(&self) -> NumberDensity {
        NumberDensity::new(
            self.n2.get() + self.o2.get() + self.o.get() + self.ar.get() + self.he.get(),
        )
    }
    /// Mass density — the quantity the drag chain actually consumes.
    pub fn mass_density(&self) -> MassDensity {
        let per_particle = |m: MolarMass| m.get() / N_AVOGADRO;
        MassDensity::new(
            self.n2.get() * per_particle(species::N2)
                + self.o2.get() * per_particle(species::O2)
                + self.o.get() * per_particle(species::O)
                + self.ar.get() * per_particle(species::AR)
                + self.he.get() * per_particle(species::HE),
        )
    }
    /// Mean molar mass of the mixture. Falls from about 26 g/mol at 120 km to
    /// near 16 g/mol — pure atomic oxygen — by 400 km.
    pub fn mean_molar_mass(&self) -> MolarMass {
        let n = self.total().get();
        if n <= 0.0 {
            return MolarMass::new(0.0);
        }
        MolarMass::new(self.mass_density().get() * N_AVOGADRO / n)
    }
    /// The atomic oxygen fraction by number. Drives material erosion, and it is
    /// what an air-breathing intake is actually collecting.
    pub fn atomic_oxygen_fraction(&self) -> Ratio {
        Ratio::new(self.o.get() / self.total().get())
    }
}

/// The full composition at an altitude, for a given activity level.
pub fn composition(altitude: Length, t_inf: Temperature) -> Composition {
    Composition {
        n2: species_number_density(altitude, t_inf, base_density::N2, species::N2, ALPHA_N2),
        o2: species_number_density(altitude, t_inf, base_density::O2, species::O2, ALPHA_O2),
        o: species_number_density(altitude, t_inf, base_density::O, species::O, ALPHA_O),
        ar: species_number_density(altitude, t_inf, base_density::AR, species::AR, ALPHA_AR),
        he: species_number_density(altitude, t_inf, base_density::HE, species::HE, ALPHA_HE),
    }
}

/// Mass density at altitude. The single most-consumed number in the whole
/// design: 54 nodes read it directly or transitively.
pub fn mass_density(altitude: Length, t_inf: Temperature) -> MassDensity {
    composition(altitude, t_inf).mass_density()
}

/// Local density scale height, `H = R·T/(M·g)`.
pub fn scale_height(altitude: Length, t_inf: Temperature) -> Length {
    let t = temperature(altitude, t_inf);
    let m = composition(altitude, t_inf).mean_molar_mass();
    let g = gravity(altitude);
    Length::new(R_UNIVERSAL * t.get() / (m.get() * g.get()))
}

/// The one-sigma uncertainty this model claims, as a fraction of the density.
///
/// Stated rather than assumed. Published comparisons of empirical thermosphere
/// models against accelerometer-derived densities put the residual at 10–15% at
/// solar minimum and worse during storms; this reports 15% quiet and grows it
/// with geomagnetic activity. A node that consumes density and does not carry
/// this forward is a node whose margin is fictional.
///
/// `kp` is the planetary index, on the same scale as
/// [`exospheric_temperature`].
pub fn density_uncertainty(kp: f64) -> Ratio {
    Ratio::new(0.15 + 0.030 * kp)
}

/// Mean thermal speed of a species at a temperature, `sqrt(8·k·T/(pi·m))`.
///
/// This is what an air-breathing intake is competing against: the collected gas
/// wants to leave the chamber at this speed in every direction, while it only
/// arrives from one.
pub fn mean_thermal_speed(t: Temperature, molar_mass: MolarMass) -> Velocity {
    Velocity::new(pmath::sqrt(
        8.0 * R_UNIVERSAL * t.get() / (pmath::PI * molar_mass.get()),
    ))
}

/// Most probable speed, `sqrt(2·k·T/m)` — the denominator of the speed ratio in
/// every free-molecular aerodynamics relation.
pub fn most_probable_speed(t: Temperature, molar_mass: MolarMass) -> Velocity {
    Velocity::new(pmath::sqrt(2.0 * R_UNIVERSAL * t.get() / molar_mass.get()))
}

/// Mean free path, `1/(sqrt(2)·n·sigma)`, with an effective collision
/// cross-section of 1e-19 m^2 for the thermospheric mixture.
pub fn mean_free_path(n: NumberDensity) -> Length {
    const SIGMA_COLLISION: f64 = 1.0e-19;
    Length::new(1.0 / (pmath::SQRT_2 * n.get() * SIGMA_COLLISION))
}

/// Knudsen number, `lambda/L`. Above about 10 the flow is free-molecular, which
/// is what licenses every aerodynamic relation in this system — and checking it
/// is how a node notices it has been asked to work outside its regime.
pub fn knudsen(mean_free_path: Length, characteristic_length: Length) -> Ratio {
    Ratio::new(mean_free_path.get() / characteristic_length.get())
}

/// Solar radiation pressure at one astronomical unit, `S/c`.
pub fn solar_radiation_pressure() -> Pressure {
    Pressure::new(SOLAR_CONSTANT.get() / SPEED_OF_LIGHT.get())
}

/// Dipole approximation to the geomagnetic field magnitude at radius `r` and
/// magnetic latitude `lat`. Sizes magnetorquers and bounds the residual-dipole
/// disturbance torque.
pub fn magnetic_field(r: Length, magnetic_latitude: Angle) -> MagneticFluxDensity {
    /// Earth dipole moment, IGRF epoch 2020.
    const B0: f64 = 3.12e-5; // T at the equatorial surface
    let ratio = R_EARTH.get() / r.get();
    let s = magnetic_latitude.sin();
    MagneticFluxDensity::new(B0 * ratio * ratio * ratio * pmath::sqrt(1.0 + 3.0 * s * s))
}
