//! The method language: the reference: what the language offers, as data.

use super::*;

/// One statement form, for the reference page and the form's help.
pub struct StatementSpec {
    pub form: &'static str,
    pub meaning: &'static str,
    pub example: &'static str,
}

pub const STATEMENTS: &[StatementSpec] = &[
    StatementSpec {
        form: "let NAME = EXPR",
        meaning: "A new named value. Its dimension is whatever the expression's is.",
        example: "let r = R_EARTH + h",
    },
    StatementSpec {
        form: "let NAME : Quantity = EXPR",
        meaning: "The same, and the checker confirms the expression has that quantity's dimension.",
        example: "let v : Velocity = sqrt(MU_EARTH / r)",
    },
    StatementSpec {
        form: "const NAME = NUMBER [unit]",
        meaning: "A constant from your source, with its unit. Say where it comes from in a comment.",
        example: "const cd = 2.2 [1]   # drag coefficient, Sentman flat plate",
    },
    StatementSpec {
        form: "const NAME = [A, B, …] [unit]",
        meaning: "A list from your source, written out, with one unit for every entry. It is written once, at the method's top level, and never changes. Read one entry as NAME[i], counted from 1; its length as len(NAME); every entry with for … in; or use it as a row of interp's table.",
        example: "const EDGES = [90, 130, 170] [1]   # the published band edges, sfu",
    },
    StatementSpec {
        form: "set NAME = EXPR",
        meaning: "Change a value made with let. Same dimension; inputs and constants cannot be changed.",
        example: "set total = total + term",
    },
    StatementSpec {
        form: "if COND then … else if COND then … else … end",
        meaning: "Choose. Conditions compare like with like: h < 0 [m], not h < 0 [s].",
        example: "if h < 150 [km] then\n  refuse \"below the lowest altitude the model covers\"\nend",
    },
    StatementSpec {
        form: "for NAME = FIRST to LAST … end",
        meaning: "Repeat for whole numbers FIRST..LAST. The count is fixed when written; the loop variable is a pure number.",
        example: "for n = 1 to 10\n  set total = total + x ^ n / n\nend",
    },
    StatementSpec {
        form: "for NAME in LIST … end",
        meaning: "Repeat once for each entry of a list, in order; NAME holds the entry, in the list's unit.",
        example: "for edge in EDGES\n  if f107 >= edge then\n    set band = band + 1\n  end\nend",
    },
    StatementSpec {
        form: "while COND at most N times … end",
        meaning: "Repeat while COND holds — an iteration that settles, or a count the inputs decide. N is the most passes it may take, fixed when written; if COND still holds after N passes the node refuses, saying the loop did not settle, rather than answer with wherever it had got to.",
        example: "while abs(r * r - a) > 1e-12 * a at most 40 times\n  set r = (r + a / r) / 2\nend",
    },
    StatementSpec {
        form: "refuse \"reason\"",
        meaning: "The node will not answer here, and says why. A refusal is never a substitute value.",
        example: "refuse \"the orbit is inside the Earth\"",
    },
    StatementSpec {
        form: "return EXPR",
        meaning: "The node's answer, in its declared quantity. Every path ends in return or refuse.",
        example: "return v",
    },
    StatementSpec {
        form: "publish SYMBOL = EXPR",
        meaning: "For a node that publishes several values: one of them, by the member's symbol on the sheet, in that member's quantity. Each member is published once, at the method's top level, before the method returns — so every answer carries every member. A refusal may still come anywhere.",
        example: "publish Kp_mean_nominal = kp_from_ap(ap_nominal) + kp_mean_slot_bias(ap_nominal)",
    },
    StatementSpec {
        form: "# comment",
        meaning: "Anything after # on a line is for the reader.",
        example: "# Vallado (2013), eq. 1-18",
    },
];

/// The rule a function applies to dimensions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FnRule {
    /// Takes a pure number, gives a pure number: `exp`, `sin` and the like.
    Pure,
    /// Keeps its argument's dimension: `abs`.
    Same,
    /// Two arguments of one dimension, result that dimension: `min`, `hypot`.
    SameTwo,
    /// Two arguments of one dimension, result a pure number: `atan2`.
    Ratio,
    /// The square root: every exponent halves, so each must be even.
    Sqrt,
    /// The cube root: every exponent divides by three.
    Cbrt,
    /// `interp(x, [xs] [unit], [ys] [unit])`: a straight-line table lookup.
    Interp,
    /// `pow(x, p)`: the same as `x ^ p`.
    Pow,
    /// `len(LIST)`: how many entries a list has, a pure number.
    Len,
}

pub struct FnSpec {
    pub name: &'static str,
    pub arity: usize,
    pub rule: FnRule,
    pub meaning: &'static str,
}

#[rustfmt::skip]
pub const FUNCTIONS: &[FnSpec] = &[
    FnSpec { name: "sqrt", arity: 1, rule: FnRule::Sqrt, meaning: "square root; refused below zero" },
    FnSpec { name: "cbrt", arity: 1, rule: FnRule::Cbrt, meaning: "cube root" },
    FnSpec { name: "abs", arity: 1, rule: FnRule::Same, meaning: "size without sign" },
    FnSpec { name: "min", arity: 2, rule: FnRule::SameTwo, meaning: "the smaller of two like values" },
    FnSpec { name: "max", arity: 2, rule: FnRule::SameTwo, meaning: "the larger of two like values" },
    FnSpec { name: "hypot", arity: 2, rule: FnRule::SameTwo, meaning: "sqrt(a^2 + b^2), without overflow" },
    FnSpec { name: "fmod", arity: 2, rule: FnRule::SameTwo, meaning: "remainder of a / b, with a's sign" },
    FnSpec { name: "pow", arity: 2, rule: FnRule::Pow, meaning: "a to the power p — the same as a ^ p" },
    FnSpec { name: "exp", arity: 1, rule: FnRule::Pure, meaning: "e to the power x" },
    FnSpec { name: "ln", arity: 1, rule: FnRule::Pure, meaning: "natural logarithm; refused at or below zero" },
    FnSpec { name: "log10", arity: 1, rule: FnRule::Pure, meaning: "base-10 logarithm" },
    FnSpec { name: "log2", arity: 1, rule: FnRule::Pure, meaning: "base-2 logarithm" },
    FnSpec { name: "sin", arity: 1, rule: FnRule::Pure, meaning: "sine of an angle (radians; write 30 [deg] for degrees)" },
    FnSpec { name: "cos", arity: 1, rule: FnRule::Pure, meaning: "cosine" },
    FnSpec { name: "tan", arity: 1, rule: FnRule::Pure, meaning: "tangent" },
    FnSpec { name: "asin", arity: 1, rule: FnRule::Pure, meaning: "inverse sine, in radians; refused outside -1..1" },
    FnSpec { name: "acos", arity: 1, rule: FnRule::Pure, meaning: "inverse cosine, in radians; refused outside -1..1" },
    FnSpec { name: "atan", arity: 1, rule: FnRule::Pure, meaning: "inverse tangent, in radians" },
    FnSpec { name: "atan2", arity: 2, rule: FnRule::Ratio, meaning: "the angle of (x, y), from y and x of one dimension" },
    FnSpec { name: "sinh", arity: 1, rule: FnRule::Pure, meaning: "hyperbolic sine" },
    FnSpec { name: "cosh", arity: 1, rule: FnRule::Pure, meaning: "hyperbolic cosine" },
    FnSpec { name: "tanh", arity: 1, rule: FnRule::Pure, meaning: "hyperbolic tangent" },
    FnSpec { name: "erf", arity: 1, rule: FnRule::Pure, meaning: "the error function" },
    FnSpec { name: "erfc", arity: 1, rule: FnRule::Pure, meaning: "1 - erf(x)" },
    FnSpec { name: "floor", arity: 1, rule: FnRule::Pure, meaning: "round down — pure numbers only, since the answer would depend on the unit" },
    FnSpec { name: "ceil", arity: 1, rule: FnRule::Pure, meaning: "round up — pure numbers only" },
    FnSpec { name: "round", arity: 1, rule: FnRule::Pure, meaning: "round to nearest, halves away from zero — pure numbers only" },
    FnSpec { name: "wrap_2pi", arity: 1, rule: FnRule::Pure, meaning: "an angle brought into 0..2π" },
    FnSpec { name: "wrap_pi", arity: 1, rule: FnRule::Pure, meaning: "an angle brought into -π..π" },
    FnSpec { name: "interp", arity: 3, rule: FnRule::Interp, meaning: "straight-line lookup in a table: interp(x, XS, YS), each row a list by name or written out as [x1, x2, …] [unit]; held at the ends" },
    FnSpec { name: "len", arity: 1, rule: FnRule::Len, meaning: "how many entries a list has: len(EDGES)" },
];

/// How each function is computed — one table, read by the interpreter to run
/// a method. The checked ones are `vleo_units::method_rt`, which say why a
/// value has no answer in the method's own words.
pub enum Impl {
    Plain1(fn(f64) -> f64),
    Plain2(fn(f64, f64) -> f64),
    Checked1(fn(f64, u32) -> Result<f64, rt::MethodError>),
    Checked2(fn(f64, f64, u32) -> Result<f64, rt::MethodError>),
}

pub fn implementation(name: &str) -> Option<Impl> {
    use Impl::*;
    Some(match name {
        "sqrt" => Checked1(rt::sqrt),
        "cbrt" => Plain1(pmath::cbrt),
        "abs" => Plain1(pmath::abs),
        "min" => Plain2(pmath::min),
        "max" => Plain2(pmath::max),
        "hypot" => Plain2(pmath::hypot),
        "fmod" => Checked2(rt::fmod),
        "pow" => Plain2(rt::pow),
        "exp" => Plain1(pmath::exp),
        "ln" => Checked1(rt::ln),
        "log10" => Checked1(rt::log10),
        "log2" => Checked1(rt::log2),
        "sin" => Plain1(pmath::sin),
        "cos" => Plain1(pmath::cos),
        "tan" => Plain1(pmath::tan),
        "asin" => Checked1(rt::asin),
        "acos" => Checked1(rt::acos),
        "atan" => Plain1(pmath::atan),
        "atan2" => Plain2(pmath::atan2),
        "sinh" => Plain1(pmath::sinh),
        "cosh" => Plain1(pmath::cosh),
        "tanh" => Plain1(pmath::tanh),
        "erf" => Plain1(pmath::erf),
        "erfc" => Plain1(pmath::erfc),
        "floor" => Plain1(pmath::floor),
        "ceil" => Plain1(pmath::ceil),
        "round" => Plain1(pmath::round),
        "wrap_2pi" => Plain1(pmath::wrap_2pi),
        "wrap_pi" => Plain1(pmath::wrap_pi),
        _ => return None,
    })
}

pub(super) fn rt_diag(e: rt::MethodError) -> Diag {
    match e {
        rt::MethodError::Refused(r) => Diag::err(0, r),
        rt::MethodError::Degenerate { line, what } => Diag::err(line as usize, what),
    }
}

/// A constant the kernel already holds, available to every method by name.
pub struct ConstSpec {
    pub name: &'static str,
    pub value: f64,
    /// Its unit, as written in brackets.
    pub unit: &'static str,
    pub meaning: &'static str,
}

#[rustfmt::skip]
pub const KERNEL_CONSTANTS: &[ConstSpec] = &[
    ConstSpec { name: "PI", value: core::f64::consts::PI, unit: "1", meaning: "π" },
    ConstSpec { name: "MU_EARTH", value: k::MU_EARTH, unit: "m^3/s^2", meaning: "Earth's gravitational parameter" },
    ConstSpec { name: "R_EARTH", value: k::R_EARTH.get(), unit: "m", meaning: "Earth's equatorial radius (WGS-84)" },
    ConstSpec { name: "R_EARTH_MEAN", value: k::R_EARTH_MEAN.get(), unit: "m", meaning: "Earth's mean radius" },
    ConstSpec { name: "F_EARTH", value: k::F_EARTH, unit: "1", meaning: "Earth's flattening (WGS-84)" },
    ConstSpec { name: "J2_EARTH", value: k::J2_EARTH, unit: "1", meaning: "Earth's J2 zonal harmonic" },
    ConstSpec { name: "OMEGA_EARTH", value: k::OMEGA_EARTH.get(), unit: "rad/s", meaning: "Earth's rotation rate" },
    ConstSpec { name: "SIDEREAL_DAY", value: k::SIDEREAL_DAY.get(), unit: "s", meaning: "one sidereal day" },
    ConstSpec { name: "G0", value: k::G0.get(), unit: "m/s^2", meaning: "standard gravity" },
    ConstSpec { name: "SOLAR_CONSTANT", value: k::SOLAR_CONSTANT.get(), unit: "W/m^2", meaning: "total solar irradiance at 1 AU" },
    ConstSpec { name: "EARTH_ALBEDO", value: k::EARTH_ALBEDO, unit: "1", meaning: "Earth's mean Bond albedo" },
    ConstSpec { name: "EARTH_IR", value: k::EARTH_IR.get(), unit: "W/m^2", meaning: "Earth's mean outgoing infrared" },
    ConstSpec { name: "AU", value: k::ASTRONOMICAL_UNIT.get(), unit: "m", meaning: "the astronomical unit" },
    ConstSpec { name: "SPEED_OF_LIGHT", value: k::SPEED_OF_LIGHT.get(), unit: "m/s", meaning: "the speed of light" },
    ConstSpec { name: "K_BOLTZMANN", value: k::K_BOLTZMANN, unit: "J/K", meaning: "the Boltzmann constant" },
    ConstSpec { name: "R_UNIVERSAL", value: k::R_UNIVERSAL, unit: "J/K/mol", meaning: "the molar gas constant" },
    ConstSpec { name: "N_AVOGADRO", value: k::N_AVOGADRO, unit: "1/mol", meaning: "the Avogadro constant" },
    ConstSpec { name: "SIGMA_SB", value: k::SIGMA_SB, unit: "W/m^2/K^4", meaning: "the Stefan–Boltzmann constant" },
    ConstSpec { name: "ELEMENTARY_CHARGE", value: k::ELEMENTARY_CHARGE, unit: "C", meaning: "the elementary charge" },
    ConstSpec { name: "ATOMIC_MASS_UNIT", value: k::ATOMIC_MASS_UNIT, unit: "kg", meaning: "the atomic mass unit" },
    ConstSpec { name: "PLANCK", value: k::PLANCK, unit: "J.s", meaning: "the Planck constant" },
];

pub(super) fn kernel_constant(name: &str) -> Option<(f64, Dim)> {
    let c = KERNEL_CONSTANTS.iter().find(|c| c.name == name)?;
    let (_, d) = parse_unit(c.unit).ok()?;
    Some((c.value, d))
}

pub(super) fn function(name: &str) -> Option<&'static FnSpec> {
    FUNCTIONS.iter().find(|f| f.name == name)
}

pub(super) const KEYWORDS: &[&str] = &[
    "let", "set", "const", "if", "then", "else", "end", "for", "in", "to", "while", "refuse",
    "return", "publish", "and", "or", "not", "true", "false",
];
