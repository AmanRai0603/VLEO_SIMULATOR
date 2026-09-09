//! Physical and astrodynamic constants.
//!
//! Every constant here is typed, and every one names where its value came from.
//! A constant with no source is a number somebody remembered, and this file is
//! read by every node in the system.

use crate::quantity::*;

/// Boltzmann constant. CODATA 2018 — exact by the 2019 SI redefinition.
pub const K_BOLTZMANN: f64 = 1.380_649e-23; // J/K

/// Universal gas constant, `R = N_A · k_B`. CODATA 2018, exact.
pub const R_UNIVERSAL: f64 = 8.314_462_618_153_24; // J/(mol·K)

/// Avogadro constant. CODATA 2018, exact.
pub const N_AVOGADRO: f64 = 6.022_140_76e23; // 1/mol

/// Stefan-Boltzmann constant, derived from the exact SI constants.
pub const SIGMA_SB: f64 = 5.670_374_419e-8; // W/(m^2·K^4)

/// Elementary charge. CODATA 2018, exact.
pub const ELEMENTARY_CHARGE: f64 = 1.602_176_634e-19; // C

/// Atomic mass constant. CODATA 2018.
pub const ATOMIC_MASS_UNIT: f64 = 1.660_539_066_60e-27; // kg

/// Speed of light in vacuum. Exact by definition.
pub const SPEED_OF_LIGHT: Velocity = Velocity::new(299_792_458.0);

/// Planck constant. CODATA 2018, exact.
pub const PLANCK: f64 = 6.626_070_15e-34; // J·s

/// Earth gravitational parameter, `GM`. EGM2008 / WGS-84.
pub const MU_EARTH: f64 = 3.986_004_418e14; // m^3/s^2

/// Earth equatorial radius. WGS-84.
pub const R_EARTH: Length = Length::new(6_378_137.0);

/// Earth mean radius. IUGG.
pub const R_EARTH_MEAN: Length = Length::new(6_371_008.8);

/// Earth flattening. WGS-84.
pub const F_EARTH: f64 = 1.0 / 298.257_223_563;

/// Second zonal harmonic of the Earth's gravity field. EGM2008.
pub const J2_EARTH: f64 = 1.082_626_68e-3;

/// Earth rotation rate, mean sidereal. IERS.
pub const OMEGA_EARTH: AngularRate = AngularRate::new(7.292_115_0e-5);

/// Sidereal day.
pub const SIDEREAL_DAY: Time = Time::new(86_164.090_53);

/// Solar constant at 1 au, mean. NASA/TSIS, 2019 mean total solar irradiance.
pub const SOLAR_CONSTANT: Irradiance = Irradiance::new(1_361.0);

/// Earth mean Bond albedo. Used by the thermal balance nodes.
pub const EARTH_ALBEDO: f64 = 0.306;

/// Earth mean outgoing longwave radiation, used as the infrared source term.
pub const EARTH_IR: Irradiance = Irradiance::new(237.0);

/// One astronomical unit. IAU 2012, exact.
pub const ASTRONOMICAL_UNIT: Length = Length::new(1.495_978_707e11);

/// Standard gravity, used only to convert specific impulse to exhaust
/// velocity. It is a definition, not a local gravity.
pub const G0: Acceleration = Acceleration::new(9.806_65);

/// Molar masses of the species that matter in the VLEO regime.
/// Values from the standard atomic weights, CIAAW 2021.
pub mod species {
    use crate::quantity::MolarMass;
    /// Atomic oxygen — the dominant species from roughly 200 to 600 km, and
    /// the one that makes VLEO materially different from lower altitudes.
    pub const O: MolarMass = MolarMass::new(0.015_999);
    /// Molecular nitrogen — dominant below roughly 180 km.
    pub const N2: MolarMass = MolarMass::new(0.028_014);
    /// Molecular oxygen.
    pub const O2: MolarMass = MolarMass::new(0.031_998);
    /// Helium — dominant above roughly 600 km, negligible here.
    pub const HE: MolarMass = MolarMass::new(0.004_002_602);
    /// Atomic hydrogen.
    pub const H: MolarMass = MolarMass::new(0.001_008);
    /// Argon.
    pub const AR: MolarMass = MolarMass::new(0.039_948);
    /// Atomic nitrogen.
    pub const N: MolarMass = MolarMass::new(0.014_007);
    /// Xenon — the reference stored propellant an ABEP system is measured against.
    pub const XE: MolarMass = MolarMass::new(0.131_293);
}

/// First ionisation energies, in electronvolts. NIST Atomic Spectra Database.
/// These set the floor on the energy an ABEP thruster must spend per ion.
pub mod ionisation_ev {
    pub const O: f64 = 13.618_055;
    pub const N2: f64 = 15.581;
    pub const O2: f64 = 12.070;
    pub const N: f64 = 14.534_13;
    pub const XE: f64 = 12.129_84;
}
