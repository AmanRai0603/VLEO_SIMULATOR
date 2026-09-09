//! Newtype quantities.
//!
//! Every quantity is a one-field struct over `f64`, stored in the SI base unit
//! of its dimension. The newtype vanishes at `-O3`, so this costs nothing at
//! run time and buys the single property the whole programme is built on: a
//! millinewton and a newton are different types, and adding one to the other
//! does not compile.
//!
//! Cross-dimension arithmetic is declared, never inferred. `Force / Mass`
//! yields `Acceleration` because that impl is written below; anything not
//! written below is a combination no node in this system has needed yet, and
//! adding it is a reviewed change rather than something a generic system
//! permits silently.

use crate::unit::Unit;
use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

macro_rules! quantity {
    ($name:ident, $unit:expr, $doc:literal) => {
        #[doc = $doc]
        ///
        /// Stored in the SI base unit of its dimension. Construct with
        /// [`Self::new`] when the value is already SI, or with
        /// [`Self::from_unit`] when it is not — there is no third way in.
        #[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default)]
        #[repr(transparent)]
        pub struct $name(f64);

        impl $name {
            /// The SI unit this type stores.
            pub const UNIT: Unit = $unit;
            /// Zero of this quantity.
            pub const ZERO: Self = $name(0.0);

            /// Wrap a value already expressed in [`Self::UNIT`].
            #[inline]
            pub const fn new(v: f64) -> Self {
                $name(v)
            }
            /// The raw value, in [`Self::UNIT`]. Named `get` rather than
            /// `value` so that reading it out of the type is visible in a diff.
            #[inline]
            pub const fn get(self) -> f64 {
                self.0
            }
            /// Convert in from a declared unit. `None` when the dimension does
            /// not match, which the generators reject before code exists.
            #[inline]
            pub fn from_unit(v: f64, u: Unit) -> Option<Self> {
                u.convert(v, Self::UNIT).map(|x| $name(x))
            }
            /// Convert out to a declared unit, for a page, a wheel or a ledger row.
            #[inline]
            pub fn to_unit(self, u: Unit) -> Option<f64> {
                Self::UNIT.convert(self.0, u)
            }
            #[inline]
            pub fn abs(self) -> Self {
                $name(crate::pmath::abs(self.0))
            }
            #[inline]
            pub fn min(self, other: Self) -> Self {
                $name(crate::pmath::min(self.0, other.0))
            }
            #[inline]
            pub fn max(self, other: Self) -> Self {
                $name(crate::pmath::max(self.0, other.0))
            }
            #[inline]
            pub fn is_finite(self) -> bool {
                crate::pmath::is_finite(self.0) && !crate::pmath::is_nan(self.0)
            }
        }

        impl Add for $name {
            type Output = Self;
            #[inline]
            fn add(self, o: Self) -> Self {
                $name(self.0 + o.0)
            }
        }
        impl Sub for $name {
            type Output = Self;
            #[inline]
            fn sub(self, o: Self) -> Self {
                $name(self.0 - o.0)
            }
        }
        impl Neg for $name {
            type Output = Self;
            #[inline]
            fn neg(self) -> Self {
                $name(-self.0)
            }
        }
        impl AddAssign for $name {
            #[inline]
            fn add_assign(&mut self, o: Self) {
                self.0 += o.0;
            }
        }
        impl SubAssign for $name {
            #[inline]
            fn sub_assign(&mut self, o: Self) {
                self.0 -= o.0;
            }
        }
        impl Mul<f64> for $name {
            type Output = Self;
            #[inline]
            fn mul(self, k: f64) -> Self {
                $name(self.0 * k)
            }
        }
        impl Mul<$name> for f64 {
            type Output = $name;
            #[inline]
            fn mul(self, q: $name) -> $name {
                $name(self * q.0)
            }
        }
        impl Div<f64> for $name {
            type Output = Self;
            #[inline]
            fn div(self, k: f64) -> Self {
                $name(self.0 / k)
            }
        }
        /// Dividing a quantity by the same quantity is the only way to reach a
        /// bare `f64` by arithmetic, and the result is dimensionless by
        /// construction.
        impl Div for $name {
            type Output = f64;
            #[inline]
            fn div(self, o: Self) -> f64 {
                self.0 / o.0
            }
        }
        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{} {}", self.0, Self::UNIT.symbol())
            }
        }
    };
}

/// `a * b = c`, and the two divisions that follow from it.
macro_rules! product {
    ($a:ident, $b:ident, $c:ident) => {
        impl Mul<$b> for $a {
            type Output = $c;
            #[inline]
            fn mul(self, o: $b) -> $c {
                $c::new(self.get() * o.get())
            }
        }
        impl Mul<$a> for $b {
            type Output = $c;
            #[inline]
            fn mul(self, o: $a) -> $c {
                $c::new(self.get() * o.get())
            }
        }
        impl Div<$a> for $c {
            type Output = $b;
            #[inline]
            fn div(self, o: $a) -> $b {
                $b::new(self.get() / o.get())
            }
        }
        impl Div<$b> for $c {
            type Output = $a;
            #[inline]
            fn div(self, o: $b) -> $a {
                $a::new(self.get() / o.get())
            }
        }
    };
}

/// `a * a = c` — the squaring case, where the two divisions coincide.
macro_rules! square {
    ($a:ident, $c:ident) => {
        impl Mul<$a> for $a {
            type Output = $c;
            #[inline]
            fn mul(self, o: $a) -> $c {
                $c::new(self.get() * o.get())
            }
        }
        impl Div<$a> for $c {
            type Output = $a;
            #[inline]
            fn div(self, o: $a) -> $a {
                $a::new(self.get() / o.get())
            }
        }
    };
}

quantity!(Length, Unit::Metre, "A distance. Altitudes are quoted in kilometres and stored in metres.");
quantity!(Area, Unit::SquareMetre, "An area — intake mouth, frontal area, radiator, solar array.");
quantity!(Volume, Unit::CubicMetre, "A volume.");
quantity!(Mass, Unit::Kilogram, "A mass.");
quantity!(MassFlow, Unit::KgPerSecond, "A mass flow rate. ABEP intakes work at milligrams per second.");
quantity!(MassFlux, Unit::KgPerSecond, "A mass flow per unit area, kg/m^2/s — the incident flux before an intake.");
quantity!(MassDensity, Unit::KgPerCubicMetre, "A mass density. The VLEO regime spans roughly 1e-9 to 1e-13 kg/m^3.");
quantity!(NumberDensity, Unit::PerCubicMetre, "A number density, particles per cubic metre.");
quantity!(MolarMass, Unit::KgPerMol, "A molar mass — the mean molecular mass of the local atmosphere.");
quantity!(Velocity, Unit::MetrePerSecond, "A speed or a speed component.");
quantity!(Acceleration, Unit::MetrePerSecond2, "An acceleration.");
quantity!(Force, Unit::Newton, "A force. Thrust and drag are both this type, which is what lets the closure node subtract them.");
quantity!(Torque, Unit::NewtonMetre, "A torque.");
quantity!(Impulse, Unit::NewtonSecond, "An impulse — force integrated over time.");
quantity!(AngularMomentum, Unit::NewtonMetreSecond, "Stored angular momentum — reaction wheel sizing.");
quantity!(Pressure, Unit::Pascal, "A pressure.");
quantity!(Energy, Unit::Joule, "An energy. Battery capacity is stored here and displayed in watt-hours.");
quantity!(Power, Unit::Watt, "A power.");
quantity!(Irradiance, Unit::WattPerSquareMetre, "A power per unit area — the solar constant, albedo, Earth infrared.");
quantity!(Temperature, Unit::Kelvin, "An absolute temperature. There is no Celsius in the kernel.");
quantity!(Time, Unit::Second, "A duration.");
quantity!(Angle, Unit::Radian, "An angle, in radians. Degrees exist only at a face.");
quantity!(AngularRate, Unit::RadianPerSecond, "An angular rate.");
quantity!(Frequency, Unit::Hertz, "A frequency.");
quantity!(Voltage, Unit::Volt, "A potential difference.");
quantity!(Current, Unit::Ampere, "An electric current.");
quantity!(Charge, Unit::Coulomb, "An electric charge.");
quantity!(MagneticFluxDensity, Unit::Tesla, "A magnetic flux density.");
quantity!(DipoleMoment, Unit::AmpereSquareMetre, "A magnetic dipole moment.");
quantity!(DataRate, Unit::BitPerSecond, "A data rate.");
quantity!(DataVolume, Unit::Bit, "A quantity of data, in bits.");
quantity!(Money, Unit::UsDollar, "A cost, in the currency and year the cost model was fitted in.");

quantity!(Ratio, Unit::One, "A dimensionless ratio — an efficiency, a fraction, a margin factor. Kept as its own type so an efficiency cannot be silently used where a count or a bare scale factor was meant.");

impl Ratio {
    pub const ONE: Ratio = Ratio::new(1.0);
    /// True when the ratio lies in `[0, 1]` — the shape almost every
    /// efficiency in this system must have.
    pub fn is_fraction(self) -> bool {
        self.get() >= 0.0 && self.get() <= 1.0
    }
}

// --- declared products, each one a relation some node actually uses ---------
product!(Velocity, Time, Length); //           v · t = s
product!(Acceleration, Time, Velocity); //     a · t = v
product!(Mass, Acceleration, Force); //        m · a = F
product!(MassFlow, Velocity, Force); //        ṁ · v = F   thrust from a flow
product!(MassFlux, Area, MassFlow); //         (ρv) · A = ṁ
product!(MassDensity, Velocity, MassFlux); //  ρ · v = mass flux
product!(MassDensity, Volume, Mass); //        ρ · V = m
product!(NumberDensity, Volume, Ratio); //     n · V = count (dimensionless)
product!(Pressure, Area, Force); //            p · A = F
product!(Force, Length, Torque); //            F · r = M
product!(Force, Velocity, Power); //           F · v = P   jet power
product!(Force, Time, Impulse); //             F · t = I
product!(Torque, Time, AngularMomentum); //    M · t = h
product!(Power, Time, Energy); //              P · t = E
product!(Irradiance, Area, Power); //          S · A = P
product!(Voltage, Current, Power); //          V · I = P
product!(Current, Time, Charge); //            I · t = q
product!(DataRate, Time, DataVolume); //       R · t = bits
product!(AngularRate, Time, Angle); //         ω · t = θ
product!(MagneticFluxDensity, DipoleMoment, Torque); // B · m = M
product!(Length, Area, Volume); //             s · A = V
square!(Length, Area); //                      s · s = A

/// Speed from a specific energy, `v = sqrt(2 E / m)` — written once here so no
/// node re-derives the square root of a ratio of two typed quantities.
impl Velocity {
    #[inline]
    pub fn from_specific_energy(e: Energy, m: Mass) -> Velocity {
        Velocity::new(crate::pmath::sqrt(2.0 * e.get() / m.get()))
    }
}

impl Angle {
    pub const ZERO_ANGLE: Angle = Angle::new(0.0);
    #[inline]
    pub fn from_deg(d: f64) -> Angle {
        Angle::new(d * Unit::Degree.si_factor())
    }
    #[inline]
    pub fn deg(self) -> f64 {
        self.get() / Unit::Degree.si_factor()
    }
    #[inline]
    pub fn sin(self) -> f64 {
        crate::pmath::sin(self.get())
    }
    #[inline]
    pub fn cos(self) -> f64 {
        crate::pmath::cos(self.get())
    }
    #[inline]
    pub fn tan(self) -> f64 {
        crate::pmath::tan(self.get())
    }
    #[inline]
    pub fn sin_cos(self) -> (f64, f64) {
        crate::pmath::sin_cos(self.get())
    }
    /// Wrapped into `(-pi, pi]`.
    #[inline]
    pub fn wrapped(self) -> Angle {
        Angle::new(crate::pmath::wrap_pi(self.get()))
    }
}

impl Length {
    #[inline]
    pub fn from_km(km: f64) -> Length {
        Length::new(km * 1.0e3)
    }
    #[inline]
    pub fn km(self) -> f64 {
        self.get() * 1.0e-3
    }
}

impl Time {
    #[inline]
    pub fn from_minutes(m: f64) -> Time {
        Time::new(m * 60.0)
    }
    #[inline]
    pub fn from_days(d: f64) -> Time {
        Time::new(d * 86_400.0)
    }
    #[inline]
    pub fn minutes(self) -> f64 {
        self.get() / 60.0
    }
    #[inline]
    pub fn days(self) -> f64 {
        self.get() / 86_400.0
    }
}

impl Force {
    #[inline]
    pub fn from_mn(mn: f64) -> Force {
        Force::new(mn * 1.0e-3)
    }
    #[inline]
    pub fn mn(self) -> f64 {
        self.get() * 1.0e3
    }
}

impl MassFlow {
    #[inline]
    pub fn from_mg_s(mg: f64) -> MassFlow {
        MassFlow::new(mg * 1.0e-6)
    }
    #[inline]
    pub fn mg_s(self) -> f64 {
        self.get() * 1.0e6
    }
}
