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
///
/// IT IS THE ONLY COPY, AND IT DID NOT USED TO BE. `sw_kp_from_ap` carried a
/// second hand-written copy of the same 28 pairs in its hole, tabulated in
/// exact thirds where this one used decimals, so the two disagreed by up to
/// 0.0033 Kp everywhere between the anchors. Nothing caught it: both were
/// evidenced against the published table at the anchor points, and the anchors
/// are where the two agree. That node now calls this function.
pub fn kp_from_ap(ap: f64) -> f64 {
    const AP: &[f64] = &[
        0.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 9.0, 12.0, 15.0, 18.0, 22.0, 27.0, 32.0, 39.0, 48.0,
        56.0, 67.0, 80.0, 94.0, 111.0, 132.0, 154.0, 179.0, 207.0, 236.0, 300.0, 400.0,
    ];
    // WRITTEN AS THIRDS, NOT AS DECIMALS. Kp is defined in thirds of a unit and
    // the scale is tabulated that way, so 0.33 and 0.67 are lossy
    // transcriptions of 1/3 and 2/3 rather than the published values. The
    // difference reaches 0.0033 Kp, which is small — and it was enough for this
    // copy and the one that used to sit in sw_kp_from_ap's hole to disagree at
    // every point between the anchors.
    const KP: &[f64] = &[
        0.0,
        1.0 / 3.0,
        2.0 / 3.0,
        1.0,
        4.0 / 3.0,
        5.0 / 3.0,
        2.0,
        7.0 / 3.0,
        8.0 / 3.0,
        3.0,
        10.0 / 3.0,
        11.0 / 3.0,
        4.0,
        13.0 / 3.0,
        14.0 / 3.0,
        5.0,
        16.0 / 3.0,
        17.0 / 3.0,
        6.0,
        19.0 / 3.0,
        20.0 / 3.0,
        7.0,
        22.0 / 3.0,
        23.0 / 3.0,
        8.0,
        25.0 / 3.0,
        26.0 / 3.0,
        9.0,
    ];
    pmath::interp(ap, AP, KP)
}

/// The bin centres both slot-bias tables are measured on.
///
/// `prf_ap2kp`'s own bins, as the midpoints of its edges 0 5 10 15 20 30 45 70
/// 110 400. Shared by the two functions below so the pair cannot drift apart in
/// their x values while agreeing in their y, which is a failure mode neither
/// one's tests would see.
const SLOT_BIN_CENTRES: &[f64] = &[2.5, 7.5, 12.5, 17.5, 25.0, 37.5, 57.5, 90.0, 255.0];

/// How far the daily peak `Kp` slot sits above what [`kp_from_ap`] returns.
///
/// `Kp(ap)` is concave, so a table run on a DAILY MEAN `Ap` returns a value
/// above the mean of the eight three-hourly `Kp` and well below the daily peak.
/// A design sized on the peak slot through the conversion alone is sized on a
/// quieter sky than the record's, and on exactly the days a drag design is
/// sized by. This is the measured correction.
///
/// MEASURED DATA, not a published relation. The nine values are the median of
/// `max_8(Kp) - kp_from_ap(Ap)` in each bin over
/// `bundles/solar-weather@2026.09.14`, and must be re-measured when that bundle
/// moves. It sits here rather than in a caller by the rule the whole module
/// follows: more than one caller reads it — `sw_kp_slot_bias` at one `Ap` and
/// `sw_kp_scenarios` at five — and a hand-copied table drifts from its original
/// without anything noticing.
///
/// The eighth and ninth entries are NOT monotone. That is the record's shape,
/// declared rather than smoothed.
///
/// Held at the end values rather than extrapolated, which is the choice
/// `prf_ap2kp` makes explicitly: "hold the end bins, never extrapolate".
pub fn kp_peak_slot_bias(ap: f64) -> f64 {
    const OFFSET: &[f64] = &[
        1.0,
        0.8333333333333333,
        1.1144067796610169,
        1.0,
        1.2,
        1.3333333333333333,
        1.5454545454545454,
        1.7469135802469136,
        1.3174603174603174,
    ];
    pmath::interp(ap, SLOT_BIN_CENTRES, OFFSET)
}

/// How far the daily MEAN `Kp` slot sits from what [`kp_from_ap`] returns.
///
/// The companion to [`kp_peak_slot_bias`], on the same day set and the same
/// bins, taking the mean over the eight slots where that one takes the maximum.
/// Almost every value is negative, which is what the concavity of `Kp(ap)`
/// predicts: the table run on a daily mean reads above the mean of the slots.
///
/// MEASURED DATA, not a published relation, from
/// `bundles/solar-weather@2026.09.14`, and here for the same reason its
/// companion is.
///
/// The FIRST entry is positive where every other is negative. That is the
/// record disagreeing with the concavity argument in the quietest bin, carried
/// rather than clipped to zero.
pub fn kp_mean_slot_bias(ap: f64) -> f64 {
    const OFFSET: &[f64] = &[
        0.041666666666666685,
        -0.125,
        -0.09763888888888905,
        -0.11111111111111116,
        -0.13333333333333286,
        -0.24099537037037067,
        -0.33333333333333304,
        -0.4487179487179489,
        -0.4868948412698413,
    ];
    pmath::interp(ap, SLOT_BIN_CENTRES, OFFSET)
}

/// The mean solar cycle, as a shape, an amplitude and a period.
///
/// `SOLAR_CYCLE_SHAPE[k]` is the expected F10.7 at `k` knots past a cycle
/// maximum, divided by that cycle's own peak: the mean over the record's
/// COMPLETED cycles of their centred 81-day mean normalised by their own
/// 81-day peak. Normalising each cycle by a peak measured the same way is what
/// keeps the shape inside `0..=1`; ten of these ninety-four knots rest on one
/// cycle rather than two, where the two cycles' differing lengths leave only
/// one covering that phase.
///
/// This is MEASURED DATA, not a published relation, and it is the one thing in
/// this kernel that is: it comes from `bundles/solar-weather@2026.09.14` and
/// must be re-measured when that bundle moves. It is here rather than in a
/// caller for the same reason [`kp_from_ap`] is: a hand-copied table drifts
/// from its original without anything noticing, which is not hypothetical —
/// `kp_from_ap` had exactly that happen and the two copies disagreed for as
/// long as both existed.
pub const SOLAR_CYCLE_SHAPE: &[f64] = &[
    1.000000, 0.903695, 0.807593, 0.773599, 0.747277, 0.750621, 0.702665, 0.699680, 0.671394,
    0.633116, 0.593519, 0.561394, 0.560421, 0.530158, 0.554559, 0.572413, 0.522981, 0.489549,
    0.476323, 0.450250, 0.465763, 0.491509, 0.455476, 0.469410, 0.460672, 0.437930, 0.421984,
    0.420851, 0.437148, 0.427348, 0.415508, 0.401208, 0.404088, 0.401337, 0.395792, 0.403236,
    0.402821, 0.384875, 0.380210, 0.383322, 0.393501, 0.400451, 0.340028, 0.324026, 0.324897,
    0.373319, 0.369609, 0.365922, 0.375961, 0.381349, 0.370032, 0.371935, 0.378041, 0.381774,
    0.397795, 0.405428, 0.393973, 0.389129, 0.426488, 0.453456, 0.463151, 0.470999, 0.479126,
    0.544836, 0.578380, 0.555890, 0.584878, 0.621035, 0.702772, 0.763337, 0.739668, 0.662755,
    0.650724, 0.730621, 0.757925, 0.724262, 0.745062, 0.767270, 0.758000, 0.773305, 0.764422,
    0.789755, 0.772538, 0.732759, 0.736271, 0.796305, 0.842015, 0.866375, 0.862554, 0.794588,
    0.753098, 0.812943, 0.899607, 0.963136,
];

/// One mean completed-cycle length: 11.88 years for cycle 23, 11.00 for cycle
/// 24. The shape wraps on this, which is what lets a date beyond any observed
/// cycle be answered at all.
pub const SOLAR_CYCLE_PERIOD_DAYS: f64 = 4178.0;

/// Cycle 25's own 81-day peak, and the day it fell on as days since 2000-01-01
/// (2024-09-04). Cycle 25 is NOT finished, so both move when the bundle does.
pub const SOLAR_CYCLE_PEAK_SFU: f64 = 225.135_802_469_135_8;
/// See [`SOLAR_CYCLE_PEAK_SFU`].
pub const SOLAR_CYCLE_PEAK_DAY: f64 = 9013.0;

/// The mean of the COMPLETED cycles' own peaks — 226.81 sfu for cycle 23 and
/// 160.90 for cycle 24.
///
/// This is the amplitude used for any date outside cycle 25, and the reason
/// the two differ is the honest part of the model: the shape of a cycle
/// repeats and its SIZE does not, so a date in a cycle that has not happened
/// gets the average of the ones that have rather than a repeat of this one.
/// The spread behind that average is a factor of 1.41, which is the error bar
/// nothing here publishes.
pub const SOLAR_CYCLE_MEAN_PEAK_SFU: f64 = 193.85802469135802;

/// The expected F10.7 at a date, in solar flux units.
///
/// `day` is days since 2000-01-01. The date's distance from cycle 25's maximum
/// is folded onto one cycle period, read off [`SOLAR_CYCLE_SHAPE`], and scaled
/// by the amplitude of the cycle it lands in: cycle 25's own peak for cycle 25,
/// and the completed-cycle mean for everything else.
///
/// The answer is DISCONTINUOUS at half a period either side of the maximum,
/// where the amplitude hands over. That is deliberate and it is not smoothed:
/// the step marks the boundary between a cycle whose size has been measured
/// and cycles whose size has not. It falls at knot 47 — half of ninety-four —
/// so a caller walking the knots meets it exactly rather than somewhere inside
/// a segment, and it sits at the cycle minimum, where the analogue is at its
/// lowest and the step is at its smallest.
pub fn solar_cycle_analogue(day: f64) -> f64 {
    let n = SOLAR_CYCLE_SHAPE.len();
    let p = SOLAR_CYCLE_PERIOD_DAYS;
    let d = day - SOLAR_CYCLE_PEAK_DAY;
    // Which cycle: 0 is cycle 25, anything else is a cycle whose size is a guess.
    let cycle = pmath::floor((d + 0.5 * p) / p);
    let amp = if cycle == 0.0 {
        SOLAR_CYCLE_PEAK_SFU
    } else {
        SOLAR_CYCLE_MEAN_PEAK_SFU
    };
    // Days past the maximum, folded onto one period. fmod keeps the dividend's
    // sign, so a date before the maximum needs the second fold.
    let u = pmath::fmod(pmath::fmod(d, p) + p, p);
    let step = p / n as f64;
    let i = u / step;
    let lo = (i as usize) % n;
    let hi = (lo + 1) % n;
    amp * (SOLAR_CYCLE_SHAPE[lo]
        + (SOLAR_CYCLE_SHAPE[hi] - SOLAR_CYCLE_SHAPE[lo]) * (i - pmath::floor(i)))
}

/// The mean of [`solar_cycle_analogue`] over `[t0, t1]`, both days since
/// 2000-01-01.
///
/// This is what a design that must last the window wants as a centre, rather
/// than the value at either end: the end of a five-year mission opening in
/// 2027 sits near a minimum, and sizing to it would miss the first three years
/// entirely. 512 panels holds the quadrature error below 0.005 sfu across every
/// window the tree can ask for.
pub fn solar_cycle_analogue_mean(t0: f64, t1: f64) -> f64 {
    crate::math::integrate::simpson(t0, t1, 512, solar_cycle_analogue) / (t1 - t0)
}

/// The highest [`solar_cycle_analogue`] reaches anywhere in `[t0, t1]`.
///
/// Exact, not sampled. The analogue is linear between knots, so its largest
/// value on an interval is at an end, at a knot, or — because the amplitude
/// handover is a step DOWN — at the instant before a handover. All three are
/// checked. Skipping the third looks safe and is not: over 6432 windows
/// spanning the declared domain it sets the answer in 18 of them, by as much as
/// 4.05 sfu.
pub fn solar_cycle_analogue_max(t0: f64, t1: f64) -> f64 {
    let n = SOLAR_CYCLE_SHAPE.len();
    let p = SOLAR_CYCLE_PERIOD_DAYS;
    let step = p / n as f64;
    let mut m = pmath::max(solar_cycle_analogue(t0), solar_cycle_analogue(t1));
    let k0 = pmath::ceil((t0 - SOLAR_CYCLE_PEAK_DAY) / step) as i64;
    let k1 = pmath::floor((t1 - SOLAR_CYCLE_PEAK_DAY) / step) as i64;
    for k in k0..=k1 {
        m = pmath::max(
            m,
            solar_cycle_analogue(SOLAR_CYCLE_PEAK_DAY + k as f64 * step),
        );
    }
    // The supremum just inside cycle 25 at each handover strictly within the
    // window. The shape there is knot n/2 and the amplitude is cycle 25's.
    let mut k = pmath::ceil((t0 - SOLAR_CYCLE_PEAK_DAY + 0.5 * p) / p);
    loop {
        let b = SOLAR_CYCLE_PEAK_DAY - 0.5 * p + k * p;
        if b >= t1 {
            break;
        }
        if b > t0 {
            m = pmath::max(m, SOLAR_CYCLE_SHAPE[n / 2] * SOLAR_CYCLE_PEAK_SFU);
        }
        k += 1.0;
    }
    m
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
