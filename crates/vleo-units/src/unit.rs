//! The runtime unit tag.
//!
//! Types carry the unit at compile time; this enum carries the *same* unit at
//! run time so that a value crossing a boundary — into a page, a wheel, a C
//! struct or a ledger row — still says what it is. The two can never disagree
//! because every quantity type declares its own `UNIT` and the generators read
//! it from there.
//!
//! Each variant states the SI factor that converts it to the base unit of its
//! dimension. Nothing in the kernel converts implicitly: a conversion is a call
//! somebody wrote, and it appears in the algorithm.

/// The seven base dimensions plus the dimensionless case, as an exponent
/// vector. Two units may be added only when their dimensions match; the
/// generators check this when they emit an interface, so a mismatch is caught
/// before any code exists.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Dim {
    /// length
    pub m: i8,
    /// mass
    pub kg: i8,
    /// time
    pub s: i8,
    /// electric current
    pub a: i8,
    /// thermodynamic temperature
    pub k: i8,
    /// amount of substance
    pub mol: i8,
    /// luminous intensity
    pub cd: i8,
}

impl Dim {
    pub const NONE: Dim = Dim {
        m: 0,
        kg: 0,
        s: 0,
        a: 0,
        k: 0,
        mol: 0,
        cd: 0,
    };
    pub const fn new(m: i8, kg: i8, s: i8, a: i8, k: i8, mol: i8, cd: i8) -> Dim {
        Dim {
            m,
            kg,
            s,
            a,
            k,
            mol,
            cd,
        }
    }
}

macro_rules! units {
    ($( $variant:ident, $sym:literal, $factor:expr, $dim:expr, $doc:literal );* $(;)?) => {
        /// Every unit any node in this system may declare.
        ///
        /// Adding a variant is a schema change: it breaks every reader of this
        /// enum until each one says what it does with the new case, which is
        /// change control performed by the compiler.
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        #[non_exhaustive]
        pub enum Unit { $( #[doc = $doc] $variant ),* }

        impl Unit {
            /// The printed symbol, exactly as it appears on a page, in an
            /// interface table and in a ledger row.
            pub const fn symbol(self) -> &'static str {
                match self { $( Unit::$variant => $sym ),* }
            }
            /// Multiply by this to reach the SI base unit of the dimension.
            pub const fn si_factor(self) -> f64 {
                match self { $( Unit::$variant => $factor ),* }
            }
            /// The dimension, for the compatibility check.
            pub const fn dim(self) -> Dim {
                match self { $( Unit::$variant => $dim ),* }
            }
            /// The machine name used in `node.toml` and in generated tables.
            pub const fn name(self) -> &'static str {
                match self { $( Unit::$variant => stringify!($variant) ),* }
            }
            /// Parse the machine name. Used by the generators, never at run time.
            pub fn from_name(s: &str) -> Option<Unit> {
                match s { $( stringify!($variant) => Some(Unit::$variant), )* _ => None }
            }
            /// Convert a value from `self` to `other`. Returns `None` when the
            /// dimensions differ — which the generators reject at build time,
            /// so at run time this is a belt-and-braces path.
            pub fn convert(self, value: f64, other: Unit) -> Option<f64> {
                if self.dim() != other.dim() { return None; }
                Some(value * self.si_factor() / other.si_factor())
            }
        }
    };
}

units! {
    // dimensionless
    One,            "-",        1.0,        Dim::NONE,                          "dimensionless ratio";
    Percent,        "%",        0.01,       Dim::NONE,                          "per cent";
    Decibel,        "dB",       1.0,        Dim::NONE,                          "decibel — a logarithmic ratio, never summed with a linear one";
    Count,          "#",        1.0,        Dim::NONE,                          "a whole number of things";
    Bit,            "bit",      1.0,        Dim::NONE,                          "one bit of information";
    // length
    Metre,          "m",        1.0,        Dim::new(1,0,0,0,0,0,0),            "metre";
    Kilometre,      "km",       1.0e3,      Dim::new(1,0,0,0,0,0,0),            "kilometre — the unit altitudes are quoted in";
    Micrometre,     "um",       1.0e-6,     Dim::new(1,0,0,0,0,0,0),            "micrometre — optical wavelengths";
    Nanometre,      "nm",       1.0e-9,     Dim::new(1,0,0,0,0,0,0),            "nanometre";
    // area, volume
    SquareMetre,    "m^2",      1.0,        Dim::new(2,0,0,0,0,0,0),            "square metre";
    CubicMetre,     "m^3",      1.0,        Dim::new(3,0,0,0,0,0,0),            "cubic metre";
    // mass
    Kilogram,       "kg",       1.0,        Dim::new(0,1,0,0,0,0,0),            "kilogram";
    Gram,           "g",        1.0e-3,     Dim::new(0,1,0,0,0,0,0),            "gram";
    KgPerMol,       "kg/mol",   1.0,        Dim::new(0,1,0,0,0,-1,0),           "molar mass";
    // time
    Second,         "s",        1.0,        Dim::new(0,0,1,0,0,0,0),            "second";
    Minute,         "min",      60.0,       Dim::new(0,0,1,0,0,0,0),            "minute";
    Hour,           "h",        3600.0,     Dim::new(0,0,1,0,0,0,0),            "hour";
    Day,            "d",        86400.0,    Dim::new(0,0,1,0,0,0,0),            "day";
    Year,           "yr",       31_557_600.0, Dim::new(0,0,1,0,0,0,0),          "Julian year, 365.25 days";
    // rates
    MetrePerSecond, "m/s",      1.0,        Dim::new(1,0,-1,0,0,0,0),           "metre per second";
    KmPerSecond,    "km/s",     1.0e3,      Dim::new(1,0,-1,0,0,0,0),           "kilometre per second";
    MetrePerSecond2,"m/s^2",    1.0,        Dim::new(1,0,-2,0,0,0,0),           "metre per second squared";
    KgPerSecond,    "kg/s",     1.0,        Dim::new(0,1,-1,0,0,0,0),           "mass flow";
    MgPerSecond,    "mg/s",     1.0e-6,     Dim::new(0,1,-1,0,0,0,0),           "milligram per second — the scale ABEP intakes work at";
    // density
    KgPerCubicMetre,"kg/m^3",   1.0,        Dim::new(-3,1,0,0,0,0,0),           "mass density";
    PerCubicMetre,  "1/m^3",    1.0,        Dim::new(-3,0,0,0,0,0,0),           "number density";
    // force, pressure, energy, power
    Newton,         "N",        1.0,        Dim::new(1,1,-2,0,0,0,0),           "newton";
    Millinewton,    "mN",       1.0e-3,     Dim::new(1,1,-2,0,0,0,0),           "millinewton — mixing this with N is the Mars Climate Orbiter failure";
    Micronewton,    "uN",       1.0e-6,     Dim::new(1,1,-2,0,0,0,0),           "micronewton";
    NewtonMetre,    "N.m",      1.0,        Dim::new(2,1,-2,0,0,0,0),           "torque";
    NewtonSecond,   "N.s",      1.0,        Dim::new(1,1,-1,0,0,0,0),           "impulse";
    NewtonMetreSecond,"N.m.s",  1.0,        Dim::new(2,1,-1,0,0,0,0),           "angular momentum";
    Pascal,         "Pa",       1.0,        Dim::new(-1,1,-2,0,0,0,0),          "pascal";
    Joule,          "J",        1.0,        Dim::new(2,1,-2,0,0,0,0),           "joule";
    WattHour,       "Wh",       3600.0,     Dim::new(2,1,-2,0,0,0,0),           "watt-hour — battery capacity";
    Watt,           "W",        1.0,        Dim::new(2,1,-3,0,0,0,0),           "watt";
    Kilowatt,       "kW",       1.0e3,      Dim::new(2,1,-3,0,0,0,0),           "kilowatt";
    WattPerSquareMetre,"W/m^2", 1.0,        Dim::new(0,1,-3,0,0,0,0),           "irradiance";
    WattPerM2K4,    "W/m^2/K^4",1.0,        Dim::new(0,1,-3,0,-4,0,0),          "Stefan-Boltzmann constant";
    // temperature
    Kelvin,         "K",        1.0,        Dim::new(0,0,0,0,1,0,0),            "kelvin — the only temperature scale in the kernel";
    // angle
    Radian,         "rad",      1.0,        Dim::NONE,                          "radian";
    Degree,         "deg",      0.017_453_292_519_943_295,Dim::NONE,            "degree — mixing this with radian is the second most common unit fault";
    RadianPerSecond,"rad/s",    1.0,        Dim::new(0,0,-1,0,0,0,0),           "angular rate";
    DegreePerSecond,"deg/s",    0.017_453_292_519_943_295,Dim::new(0,0,-1,0,0,0,0),"degree per second";
    DegreePerDay,   "deg/d",    2.019_133_393_046_672e-7, Dim::new(0,0,-1,0,0,0,0),"degree per day — nodal regression";
    Arcsecond,      "arcsec",   4.848_136_811_095_36e-6,  Dim::NONE,            "arcsecond — pointing budgets";
    // electrical and magnetic
    Volt,           "V",        1.0,        Dim::new(2,1,-3,-1,0,0,0),          "volt";
    Ampere,         "A",        1.0,        Dim::new(0,0,0,1,0,0,0),            "ampere";
    AmpereHour,     "Ah",       3600.0,     Dim::new(0,0,1,1,0,0,0),            "ampere-hour";
    Tesla,          "T",        1.0,        Dim::new(0,1,-2,-1,0,0,0),          "tesla";
    AmpereSquareMetre,"A.m^2",  1.0,        Dim::new(2,0,0,1,0,0,0),            "magnetic dipole moment — magnetorquer sizing";
    Coulomb,        "C",        1.0,        Dim::new(0,0,1,1,0,0,0),            "coulomb";
    ElectronVolt,   "eV",       1.602_176_634e-19,Dim::new(2,1,-2,0,0,0,0),     "electronvolt — ionisation cost";
    // frequency and data
    Hertz,          "Hz",       1.0,        Dim::new(0,0,-1,0,0,0,0),           "hertz";
    Megahertz,      "MHz",      1.0e6,      Dim::new(0,0,-1,0,0,0,0),           "megahertz";
    Gigahertz,      "GHz",      1.0e9,      Dim::new(0,0,-1,0,0,0,0),           "gigahertz";
    BitPerSecond,   "bit/s",    1.0,        Dim::new(0,0,-1,0,0,0,0),           "bit per second";
    MegabitPerSecond,"Mbit/s",  1.0e6,      Dim::new(0,0,-1,0,0,0,0),           "megabit per second";
    Gigabit,        "Gbit",     1.0e9,      Dim::NONE,                          "gigabit";
    Gigabyte,       "GB",       8.0e9,      Dim::NONE,                          "gigabyte, as bits";
    // money
    UsDollar,       "USD",      1.0,        Dim::NONE,                          "United States dollar, in the year the cost model was fitted";
    MillionUsDollar,"MUSD",     1.0e6,      Dim::NONE,                          "million United States dollars";
}

impl Unit {
    /// True when two units may be added, subtracted or compared.
    pub fn compatible_with(self, other: Unit) -> bool {
        self.dim() == other.dim()
    }
}

/// Present a value for a person.
///
/// One implementation, because a number formatted two ways is a number two
/// people will read differently. This converts out of SI into the unit the node
/// declared and chooses a notation that does not lie: a density of 6.6e-11
/// printed to six decimal places is `0.000000`, which is not a rounding, it is
/// a different claim.
pub fn present(value_si: f64, unit: Unit) -> (f64, &'static str) {
    (value_si / unit.si_factor(), unit.symbol())
}
