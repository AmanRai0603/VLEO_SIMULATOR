//! THE METHOD LANGUAGE — the pseudocode a node's author writes.
//!
//! A node's relation arrives in three independent forms: the author's own
//! code, which produced their test cases; this method, which says the same
//! thing in a small fixed language; and the Rust generated from the method,
//! which is what ships. Each checks the other two, so the language has to be
//! small enough that every construct has exactly one meaning and one
//! translation, and strict enough that a unit mistake is caught before any
//! code exists.
//!
//! ```text
//! # The speed of a circular orbit at altitude h.
//! let r = R_EARTH + h
//! if h < 0 [m] then
//!   refuse "the altitude is below the ground"
//! end
//! let v : Velocity = sqrt(MU_EARTH / r)
//! return v
//! ```
//!
//! Every value is a number in the SI base unit of its dimension, carrying that
//! dimension. A literal says its unit in brackets (`6371 [km]`) and is
//! converted on reading, so `h < 0 [m]` and `h < 0 [km]` are the same test.
//! The node's declared inputs arrive as variables of their quantity's
//! dimension; the kernel constants below are available by name. The checker
//! refuses a sum of unlike dimensions, a transcendental function of a
//! dimensioned value, a return of the wrong dimension, and a path that
//! neither returns nor refuses.
//!
//! One implementation: the gate, `xtask take`, the node form and the
//! translator all read this module, so the form and the checker cannot
//! disagree about what a method means. `docs/PSEUDOCODE.md` is written from
//! [`FUNCTIONS`], [`KERNEL_CONSTANTS`] and [`STATEMENTS`] by `xtask docs`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use vleo_units::method_rt as rt;
use vleo_units::unit::Dim;
use vleo_units::{constants as k, pmath, Unit};

/// The language version. A method is read by the version it was written for;
/// raising this is a reviewed change to every node that has one.
pub const LANGUAGE_VERSION: u32 = 1;

/// How many loop iterations a method may run in all. A method is a relation,
/// not a simulation: a loop exists for a short series or a fixed-point
/// refinement, and a method that needs more than this is a node that should be
/// split.
pub const MAX_STEPS: u64 = 100_000;

// ---------------------------------------------------------------------------
// diagnostics

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    /// Worth saying, and not a reason to refuse: an input the method never
    /// reads, for instance.
    Warning,
}

/// One finding, on one line of the method.
#[derive(Clone, Debug, PartialEq)]
pub struct Diag {
    /// 1-based. 0 means the method as a whole.
    pub line: usize,
    pub severity: Severity,
    pub msg: String,
}

impl Diag {
    fn err(line: usize, msg: impl Into<String>) -> Diag {
        Diag {
            line,
            severity: Severity::Error,
            msg: msg.into(),
        }
    }
    fn warn(line: usize, msg: impl Into<String>) -> Diag {
        Diag {
            line,
            severity: Severity::Warning,
            msg: msg.into(),
        }
    }
}

impl std::fmt::Display for Diag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self.severity {
            Severity::Error => "",
            Severity::Warning => "note: ",
        };
        if self.line == 0 {
            write!(f, "{kind}{}", self.msg)
        } else {
            write!(f, "line {}: {kind}{}", self.line, self.msg)
        }
    }
}

// ---------------------------------------------------------------------------
// dimensions

fn dim_arr(d: Dim) -> [i8; 7] {
    [d.m, d.kg, d.s, d.a, d.k, d.mol, d.cd]
}

fn dim_from(a: [i8; 7]) -> Dim {
    Dim::new(a[0], a[1], a[2], a[3], a[4], a[5], a[6])
}

fn dim_mul(a: Dim, b: Dim) -> Dim {
    let (x, y) = (dim_arr(a), dim_arr(b));
    dim_from(std::array::from_fn(|i| x[i] + y[i]))
}

fn dim_div(a: Dim, b: Dim) -> Dim {
    let (x, y) = (dim_arr(a), dim_arr(b));
    dim_from(std::array::from_fn(|i| x[i] - y[i]))
}

/// `a` raised to `num/den`, when every exponent stays whole.
fn dim_pow(a: Dim, num: i32, den: i32) -> Option<Dim> {
    let x = dim_arr(a);
    let mut out = [0i8; 7];
    for i in 0..7 {
        let p = x[i] as i32 * num;
        if p % den != 0 {
            return None;
        }
        out[i] = i8::try_from(p / den).ok()?;
    }
    Some(dim_from(out))
}

fn is_none(d: Dim) -> bool {
    d == Dim::NONE
}

/// A dimension as a person reads it: `m^3/s^2`, or `dimensionless`.
pub fn dim_text(d: Dim) -> String {
    if is_none(d) {
        return "dimensionless".into();
    }
    let names = ["m", "kg", "s", "A", "K", "mol", "cd"];
    let x = dim_arr(d);
    let part = |pos: bool| {
        let mut v = Vec::new();
        for i in 0..7 {
            let e = x[i];
            if (pos && e > 0) || (!pos && e < 0) {
                let e = e.abs();
                v.push(if e == 1 {
                    names[i].to_string()
                } else {
                    format!("{}^{e}", names[i])
                });
            }
        }
        v.join(".")
    };
    let (num, den) = (part(true), part(false));
    match (num.is_empty(), den.is_empty()) {
        (false, true) => num,
        (true, false) => format!("1/{den}"),
        _ => format!("{num}/{den}"),
    }
}

/// The dimension of a quantity type a sheet declares, such as `Velocity`.
pub fn quantity_dim(name: &str) -> Option<Dim> {
    vleo_units::quantity_unit(name).map(|u| u.dim())
}

/// Read a unit written in brackets: `km`, `m^3/s^2`, `W/m^2/K^4`, `1/m^3`.
///
/// Returns the factor to SI and the dimension. Any symbol a sheet may declare
/// is accepted whole, and the simple ones combine with `*` or `.`, `/` and a
/// whole-number `^`.
pub fn parse_unit(text: &str) -> Result<(f64, Dim), String> {
    let t = text.trim();
    if t.is_empty() {
        return Err("an empty unit — write [1] for a pure number".into());
    }
    if t == "1" {
        return Ok((1.0, Dim::NONE));
    }
    for name in Unit::NAMES {
        let u = Unit::from_name(name).expect("a name from the list");
        if u.symbol() == t {
            return Ok((u.si_factor(), u.dim()));
        }
    }
    let atom = |a: &str| -> Result<(f64, Dim), String> {
        if a == "1" {
            return Ok((1.0, Dim::NONE));
        }
        if a == "mol" {
            return Ok((1.0, Dim::new(0, 0, 0, 0, 0, 1, 0)));
        }
        for name in Unit::NAMES {
            let u = Unit::from_name(name).expect("a name from the list");
            let s = u.symbol();
            if s == a && !s.contains(['/', '.', '^']) {
                return Ok((u.si_factor(), u.dim()));
            }
        }
        Err(format!("«{a}» is not a unit this tool knows"))
    };
    let mut factor = 1.0;
    let mut dim = Dim::NONE;
    let mut divide = false;
    let mut rest = t;
    loop {
        let end = rest.find(['*', '.', '/']).unwrap_or(rest.len());
        let term = &rest[..end];
        let (base, exp) = match term.split_once('^') {
            Some((b, e)) => (
                b,
                e.parse::<i32>()
                    .map_err(|_| format!("«^{e}» in [{t}] is not a whole-number power"))?,
            ),
            None => (term, 1),
        };
        let (f, d) = atom(base.trim()).map_err(|e| format!("{e} (in [{t}])"))?;
        let d = dim_pow(d, exp, 1).ok_or_else(|| format!("[{t}] is too large a power"))?;
        let f = pmath::powi(f, exp);
        if divide {
            factor /= f;
            dim = dim_div(dim, d);
        } else {
            factor *= f;
            dim = dim_mul(dim, d);
        }
        if end == rest.len() {
            break;
        }
        divide = &rest[end..end + 1] == "/";
        rest = &rest[end + 1..];
    }
    Ok((factor, dim))
}

// ---------------------------------------------------------------------------
// the reference: what the language offers, as data

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
    FnSpec { name: "interp", arity: 3, rule: FnRule::Interp, meaning: "straight-line lookup in a table: interp(x, [x1, x2, …] [unit], [y1, y2, …] [unit]); held at the ends" },
];

/// How each function is computed — one table, read by the interpreter to run
/// a method and by the translator to write its Rust, so the two cannot take
/// different paths. The checked ones are `vleo_units::method_rt`, which the
/// translated code calls too.
pub enum Impl {
    Plain1(fn(f64) -> f64, &'static str),
    Plain2(fn(f64, f64) -> f64, &'static str),
    Checked1(fn(f64, u32) -> Result<f64, rt::MethodError>, &'static str),
    Checked2(
        fn(f64, f64, u32) -> Result<f64, rt::MethodError>,
        &'static str,
    ),
}

pub fn implementation(name: &str) -> Option<Impl> {
    use Impl::*;
    Some(match name {
        "sqrt" => Checked1(rt::sqrt, "rt::sqrt"),
        "cbrt" => Plain1(pmath::cbrt, "pmath::cbrt"),
        "abs" => Plain1(pmath::abs, "pmath::abs"),
        "min" => Plain2(pmath::min, "pmath::min"),
        "max" => Plain2(pmath::max, "pmath::max"),
        "hypot" => Plain2(pmath::hypot, "pmath::hypot"),
        "fmod" => Checked2(rt::fmod, "rt::fmod"),
        "pow" => Plain2(rt::pow, "rt::pow"),
        "exp" => Plain1(pmath::exp, "pmath::exp"),
        "ln" => Checked1(rt::ln, "rt::ln"),
        "log10" => Checked1(rt::log10, "rt::log10"),
        "log2" => Checked1(rt::log2, "rt::log2"),
        "sin" => Plain1(pmath::sin, "pmath::sin"),
        "cos" => Plain1(pmath::cos, "pmath::cos"),
        "tan" => Plain1(pmath::tan, "pmath::tan"),
        "asin" => Checked1(rt::asin, "rt::asin"),
        "acos" => Checked1(rt::acos, "rt::acos"),
        "atan" => Plain1(pmath::atan, "pmath::atan"),
        "atan2" => Plain2(pmath::atan2, "pmath::atan2"),
        "sinh" => Plain1(pmath::sinh, "pmath::sinh"),
        "cosh" => Plain1(pmath::cosh, "pmath::cosh"),
        "tanh" => Plain1(pmath::tanh, "pmath::tanh"),
        "erf" => Plain1(pmath::erf, "pmath::erf"),
        "erfc" => Plain1(pmath::erfc, "pmath::erfc"),
        "floor" => Plain1(pmath::floor, "pmath::floor"),
        "ceil" => Plain1(pmath::ceil, "pmath::ceil"),
        "round" => Plain1(pmath::round, "pmath::round"),
        "wrap_2pi" => Plain1(pmath::wrap_2pi, "pmath::wrap_2pi"),
        "wrap_pi" => Plain1(pmath::wrap_pi, "pmath::wrap_pi"),
        _ => return None,
    })
}

fn rt_diag(e: rt::MethodError) -> Diag {
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
    /// The Rust the translator writes for it.
    pub rust: &'static str,
}

#[rustfmt::skip]
pub const KERNEL_CONSTANTS: &[ConstSpec] = &[
    ConstSpec { name: "PI", value: core::f64::consts::PI, unit: "1", meaning: "π", rust: "core::f64::consts::PI" },
    ConstSpec { name: "MU_EARTH", value: k::MU_EARTH, unit: "m^3/s^2", meaning: "Earth's gravitational parameter", rust: "MU_EARTH" },
    ConstSpec { name: "R_EARTH", value: k::R_EARTH.get(), unit: "m", meaning: "Earth's equatorial radius (WGS-84)", rust: "R_EARTH.get()" },
    ConstSpec { name: "R_EARTH_MEAN", value: k::R_EARTH_MEAN.get(), unit: "m", meaning: "Earth's mean radius", rust: "R_EARTH_MEAN.get()" },
    ConstSpec { name: "F_EARTH", value: k::F_EARTH, unit: "1", meaning: "Earth's flattening (WGS-84)", rust: "F_EARTH" },
    ConstSpec { name: "J2_EARTH", value: k::J2_EARTH, unit: "1", meaning: "Earth's J2 zonal harmonic", rust: "J2_EARTH" },
    ConstSpec { name: "OMEGA_EARTH", value: k::OMEGA_EARTH.get(), unit: "rad/s", meaning: "Earth's rotation rate", rust: "OMEGA_EARTH.get()" },
    ConstSpec { name: "SIDEREAL_DAY", value: k::SIDEREAL_DAY.get(), unit: "s", meaning: "one sidereal day", rust: "SIDEREAL_DAY.get()" },
    ConstSpec { name: "G0", value: k::G0.get(), unit: "m/s^2", meaning: "standard gravity", rust: "G0.get()" },
    ConstSpec { name: "SOLAR_CONSTANT", value: k::SOLAR_CONSTANT.get(), unit: "W/m^2", meaning: "total solar irradiance at 1 AU", rust: "SOLAR_CONSTANT.get()" },
    ConstSpec { name: "EARTH_ALBEDO", value: k::EARTH_ALBEDO, unit: "1", meaning: "Earth's mean Bond albedo", rust: "EARTH_ALBEDO" },
    ConstSpec { name: "EARTH_IR", value: k::EARTH_IR.get(), unit: "W/m^2", meaning: "Earth's mean outgoing infrared", rust: "EARTH_IR.get()" },
    ConstSpec { name: "AU", value: k::ASTRONOMICAL_UNIT.get(), unit: "m", meaning: "the astronomical unit", rust: "ASTRONOMICAL_UNIT.get()" },
    ConstSpec { name: "SPEED_OF_LIGHT", value: k::SPEED_OF_LIGHT.get(), unit: "m/s", meaning: "the speed of light", rust: "SPEED_OF_LIGHT.get()" },
    ConstSpec { name: "K_BOLTZMANN", value: k::K_BOLTZMANN, unit: "J/K", meaning: "the Boltzmann constant", rust: "K_BOLTZMANN" },
    ConstSpec { name: "R_UNIVERSAL", value: k::R_UNIVERSAL, unit: "J/K/mol", meaning: "the molar gas constant", rust: "R_UNIVERSAL" },
    ConstSpec { name: "N_AVOGADRO", value: k::N_AVOGADRO, unit: "1/mol", meaning: "the Avogadro constant", rust: "N_AVOGADRO" },
    ConstSpec { name: "SIGMA_SB", value: k::SIGMA_SB, unit: "W/m^2/K^4", meaning: "the Stefan–Boltzmann constant", rust: "SIGMA_SB" },
    ConstSpec { name: "ELEMENTARY_CHARGE", value: k::ELEMENTARY_CHARGE, unit: "C", meaning: "the elementary charge", rust: "ELEMENTARY_CHARGE" },
    ConstSpec { name: "ATOMIC_MASS_UNIT", value: k::ATOMIC_MASS_UNIT, unit: "kg", meaning: "the atomic mass unit", rust: "ATOMIC_MASS_UNIT" },
    ConstSpec { name: "PLANCK", value: k::PLANCK, unit: "J.s", meaning: "the Planck constant", rust: "PLANCK" },
];

fn kernel_constant(name: &str) -> Option<(f64, Dim)> {
    let c = KERNEL_CONSTANTS.iter().find(|c| c.name == name)?;
    let (_, d) = parse_unit(c.unit).ok()?;
    Some((c.value, d))
}

fn function(name: &str) -> Option<&'static FnSpec> {
    FUNCTIONS.iter().find(|f| f.name == name)
}

const KEYWORDS: &[&str] = &[
    "let", "set", "const", "if", "then", "else", "end", "for", "to", "refuse", "return", "and",
    "or", "not", "true", "false",
];

// ---------------------------------------------------------------------------
// the syntax tree

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    And,
    Or,
}

impl BinOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Pow => "^",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::And => "and",
            BinOp::Or => "or",
        }
    }
    fn is_compare(self) -> bool {
        matches!(
            self,
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge | BinOp::Eq | BinOp::Ne
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    /// A literal, already in SI. `bare` is true when no unit was written,
    /// which lets a bare `0` stand for zero of any dimension.
    Num {
        si: f64,
        dim: Dim,
        bare: bool,
        line: usize,
    },
    Bool {
        value: bool,
        line: usize,
    },
    Var {
        name: String,
        line: usize,
    },
    Neg(Box<Expr>),
    Not(Box<Expr>),
    Bin {
        op: BinOp,
        l: Box<Expr>,
        r: Box<Expr>,
        line: usize,
    },
    Call {
        name: String,
        args: Vec<Expr>,
        line: usize,
    },
    /// A table for `interp`, already in SI.
    Table {
        si: Vec<f64>,
        dim: Dim,
        line: usize,
    },
}

impl Expr {
    pub fn line(&self) -> usize {
        match self {
            Expr::Num { line, .. }
            | Expr::Bool { line, .. }
            | Expr::Var { line, .. }
            | Expr::Bin { line, .. }
            | Expr::Call { line, .. }
            | Expr::Table { line, .. } => *line,
            Expr::Neg(e) | Expr::Not(e) => e.line(),
        }
    }
    /// The literal value, if this is one (a sign included).
    fn literal(&self) -> Option<f64> {
        match self {
            Expr::Num { si, .. } => Some(*si),
            Expr::Neg(e) => e.literal().map(|v| -v),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        quantity: Option<String>,
        expr: Expr,
        line: usize,
    },
    Const {
        name: String,
        expr: Expr,
        line: usize,
    },
    Set {
        name: String,
        expr: Expr,
        line: usize,
    },
    If {
        arms: Vec<(Expr, Vec<Stmt>)>,
        otherwise: Option<Vec<Stmt>>,
        line: usize,
    },
    For {
        var: String,
        first: i64,
        last: i64,
        body: Vec<Stmt>,
        line: usize,
    },
    Refuse {
        reason: String,
        line: usize,
    },
    Return {
        expr: Expr,
        line: usize,
    },
}

impl Stmt {
    pub fn line(&self) -> usize {
        match self {
            Stmt::Let { line, .. }
            | Stmt::Const { line, .. }
            | Stmt::Set { line, .. }
            | Stmt::If { line, .. }
            | Stmt::For { line, .. }
            | Stmt::Refuse { line, .. }
            | Stmt::Return { line, .. } => *line,
        }
    }
}

/// A parsed method.
#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub body: Vec<Stmt>,
}

// ---------------------------------------------------------------------------
// reading the text

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f64),
    Ident(String),
    Str(String),
    Unit(String),
    Sym(&'static str),
    Newline,
}

#[derive(Clone, Debug)]
struct Token {
    tok: Tok,
    line: usize,
}

const SYMBOLS: &[&str] = &[
    "<=", ">=", "==", "!=", "+", "-", "*", "/", "^", "(", ")", "[", "]", ",", "=", "<", ">", ":",
];

fn lex(src: &str) -> Result<Vec<Token>, Diag> {
    let mut out: Vec<Token> = Vec::new();
    let mut depth = 0i32;
    for (i, raw) in src.lines().enumerate() {
        let line = i + 1;
        let text = match raw.find('#') {
            Some(p) if !raw[..p].contains('"') || raw[..p].matches('"').count() % 2 == 0 => {
                &raw[..p]
            }
            _ => raw,
        };
        let b = text.as_bytes();
        let mut p = 0usize;
        while p < b.len() {
            let c = b[p] as char;
            if c.is_whitespace() {
                p += 1;
                continue;
            }
            if c == '"' {
                let rest = &text[p + 1..];
                let Some(q) = rest.find('"') else {
                    return Err(Diag::err(line, "a quotation is not closed on its line"));
                };
                out.push(Token {
                    tok: Tok::Str(rest[..q].to_string()),
                    line,
                });
                p += q + 2;
                continue;
            }
            if c.is_ascii_digit() || (c == '.' && p + 1 < b.len() && b[p + 1].is_ascii_digit()) {
                let start = p;
                while p < b.len() && ((b[p] as char).is_ascii_digit() || b[p] == b'.') {
                    p += 1;
                }
                if p < b.len() && (b[p] == b'e' || b[p] == b'E') {
                    let save = p;
                    p += 1;
                    if p < b.len() && (b[p] == b'+' || b[p] == b'-') {
                        p += 1;
                    }
                    if p < b.len() && (b[p] as char).is_ascii_digit() {
                        while p < b.len() && (b[p] as char).is_ascii_digit() {
                            p += 1;
                        }
                    } else {
                        p = save;
                    }
                }
                let s = &text[start..p];
                let v: f64 = s
                    .parse()
                    .map_err(|_| Diag::err(line, format!("«{s}» is not a number")))?;
                out.push(Token {
                    tok: Tok::Num(v),
                    line,
                });
                continue;
            }
            if c.is_ascii_alphabetic() || c == '_' {
                let start = p;
                while p < b.len() && ((b[p] as char).is_ascii_alphanumeric() || b[p] == b'_') {
                    p += 1;
                }
                out.push(Token {
                    tok: Tok::Ident(text[start..p].to_string()),
                    line,
                });
                continue;
            }
            if c == '[' {
                // A bracket straight after a number, or after a table's closing
                // bracket, is a unit; anywhere else it opens a table.
                let after_value = matches!(
                    out.last(),
                    Some(Token { tok: Tok::Num(_), line: l }) | Some(Token { tok: Tok::Sym("]"), line: l })
                        if *l == line
                );
                if after_value {
                    let rest = &text[p + 1..];
                    let Some(q) = rest.find(']') else {
                        return Err(Diag::err(line, "a unit's [ is not closed on its line"));
                    };
                    out.push(Token {
                        tok: Tok::Unit(rest[..q].trim().to_string()),
                        line,
                    });
                    p += q + 2;
                    continue;
                }
            }
            let Some(sym) = SYMBOLS.iter().find(|s| text[p..].starts_with(**s)) else {
                return Err(Diag::err(
                    line,
                    format!("«{c}» is not part of the method language"),
                ));
            };
            match *sym {
                "(" | "[" => depth += 1,
                ")" | "]" => depth -= 1,
                _ => {}
            }
            out.push(Token {
                tok: Tok::Sym(sym),
                line,
            });
            p += sym.len();
        }
        if depth <= 0 {
            depth = 0;
            if !matches!(
                out.last(),
                None | Some(Token {
                    tok: Tok::Newline,
                    ..
                })
            ) {
                out.push(Token {
                    tok: Tok::Newline,
                    line,
                });
            }
        }
    }
    if !matches!(
        out.last(),
        None | Some(Token {
            tok: Tok::Newline,
            ..
        })
    ) {
        let line = out.last().map(|t| t.line).unwrap_or(0);
        out.push(Token {
            tok: Tok::Newline,
            line,
        });
    }
    Ok(out)
}

struct Parser {
    toks: Vec<Token>,
    pos: usize,
}

type PResult<T> = Result<T, Diag>;

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos).map(|t| &t.tok)
    }
    fn line(&self) -> usize {
        self.toks
            .get(self.pos)
            .or_else(|| self.toks.last())
            .map(|t| t.line)
            .unwrap_or(0)
    }
    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).map(|t| t.tok.clone());
        self.pos += 1;
        t
    }
    fn is_sym(&self, s: &str) -> bool {
        matches!(self.peek(), Some(Tok::Sym(x)) if *x == s)
    }
    fn is_kw(&self, k: &str) -> bool {
        matches!(self.peek(), Some(Tok::Ident(x)) if x == k)
    }
    fn expect_sym(&mut self, s: &str) -> PResult<()> {
        if self.is_sym(s) {
            self.pos += 1;
            Ok(())
        } else {
            Err(Diag::err(self.line(), format!("expected «{s}» here")))
        }
    }
    fn expect_kw(&mut self, k: &str) -> PResult<()> {
        if self.is_kw(k) {
            self.pos += 1;
            Ok(())
        } else {
            Err(Diag::err(self.line(), format!("expected «{k}» here")))
        }
    }
    fn end_of_line(&mut self) -> PResult<()> {
        match self.peek() {
            Some(Tok::Newline) => {
                self.pos += 1;
                Ok(())
            }
            None => Ok(()),
            _ => Err(Diag::err(
                self.line(),
                "one statement per line — something follows the end of this one",
            )),
        }
    }
    fn name(&mut self, what: &str) -> PResult<String> {
        let line = self.line();
        match self.next() {
            Some(Tok::Ident(n)) if !KEYWORDS.contains(&n.as_str()) => Ok(n),
            Some(Tok::Ident(n)) => Err(Diag::err(
                line,
                format!("«{n}» is a word of the language and cannot be {what}"),
            )),
            _ => Err(Diag::err(line, format!("expected {what} here"))),
        }
    }

    fn block(&mut self, stops: &[&str]) -> PResult<Vec<Stmt>> {
        let mut body = Vec::new();
        loop {
            while matches!(self.peek(), Some(Tok::Newline)) {
                self.pos += 1;
            }
            match self.peek() {
                None => {
                    if stops.is_empty() {
                        return Ok(body);
                    }
                    return Err(Diag::err(
                        self.line(),
                        "a block is not closed — every if and for needs its «end»",
                    ));
                }
                Some(Tok::Ident(k)) if stops.contains(&k.as_str()) => return Ok(body),
                _ => body.push(self.stmt()?),
            }
        }
    }

    fn stmt(&mut self) -> PResult<Stmt> {
        let line = self.line();
        let Some(Tok::Ident(word)) = self.peek().cloned() else {
            return Err(Diag::err(
                line,
                "a statement starts with let, const, set, if, for, refuse or return",
            ));
        };
        match word.as_str() {
            "let" => {
                self.pos += 1;
                let name = self.name("a name")?;
                let quantity = if self.is_sym(":") {
                    self.pos += 1;
                    Some(self.name("a quantity type")?)
                } else {
                    None
                };
                self.expect_sym("=")?;
                let expr = self.expr()?;
                self.end_of_line()?;
                Ok(Stmt::Let {
                    name,
                    quantity,
                    expr,
                    line,
                })
            }
            "const" => {
                self.pos += 1;
                let name = self.name("a name")?;
                self.expect_sym("=")?;
                let expr = self.expr()?;
                if expr.literal().is_none() {
                    return Err(Diag::err(
                        line,
                        "a const is one number with its unit — use let for anything computed",
                    ));
                }
                self.end_of_line()?;
                Ok(Stmt::Const { name, expr, line })
            }
            "set" => {
                self.pos += 1;
                let name = self.name("a name")?;
                self.expect_sym("=")?;
                let expr = self.expr()?;
                self.end_of_line()?;
                Ok(Stmt::Set { name, expr, line })
            }
            "if" => {
                self.pos += 1;
                let mut arms = Vec::new();
                let cond = self.expr()?;
                self.expect_kw("then")?;
                self.end_of_line()?;
                let body = self.block(&["else", "end"])?;
                arms.push((cond, body));
                let mut otherwise = None;
                loop {
                    if self.is_kw("end") {
                        self.pos += 1;
                        self.end_of_line()?;
                        break;
                    }
                    self.expect_kw("else")?;
                    if self.is_kw("if") {
                        self.pos += 1;
                        let cond = self.expr()?;
                        self.expect_kw("then")?;
                        self.end_of_line()?;
                        let body = self.block(&["else", "end"])?;
                        arms.push((cond, body));
                    } else {
                        self.end_of_line()?;
                        otherwise = Some(self.block(&["end"])?);
                        self.expect_kw("end")?;
                        self.end_of_line()?;
                        break;
                    }
                }
                Ok(Stmt::If {
                    arms,
                    otherwise,
                    line,
                })
            }
            "for" => {
                self.pos += 1;
                let var = self.name("a loop name")?;
                self.expect_sym("=")?;
                let first = self.whole()?;
                self.expect_kw("to")?;
                let last = self.whole()?;
                self.end_of_line()?;
                let body = self.block(&["end"])?;
                self.expect_kw("end")?;
                self.end_of_line()?;
                Ok(Stmt::For {
                    var,
                    first,
                    last,
                    body,
                    line,
                })
            }
            "refuse" => {
                self.pos += 1;
                let reason = match self.next() {
                    Some(Tok::Str(s)) => s,
                    _ => {
                        return Err(Diag::err(
                            line,
                            "refuse is followed by its reason in quotation marks",
                        ))
                    }
                };
                self.end_of_line()?;
                Ok(Stmt::Refuse { reason, line })
            }
            "return" => {
                self.pos += 1;
                let expr = self.expr()?;
                self.end_of_line()?;
                Ok(Stmt::Return { expr, line })
            }
            "else" | "end" => Err(Diag::err(
                line,
                format!("«{word}» with no if or for open"),
            )),
            _ => Err(Diag::err(
                line,
                format!(
                    "a statement starts with let, const, set, if, for, refuse or return — not «{word}»"
                ),
            )),
        }
    }

    fn whole(&mut self) -> PResult<i64> {
        let line = self.line();
        let neg = if self.is_sym("-") {
            self.pos += 1;
            true
        } else {
            false
        };
        match self.next() {
            Some(Tok::Num(v)) if v.fract() == 0.0 && v.abs() < 1e9 => {
                Ok(if neg { -(v as i64) } else { v as i64 })
            }
            _ => Err(Diag::err(line, "a loop runs between two whole numbers")),
        }
    }

    fn expr(&mut self) -> PResult<Expr> {
        let mut l = self.and_expr()?;
        while self.is_kw("or") {
            let line = self.line();
            self.pos += 1;
            let r = self.and_expr()?;
            l = Expr::Bin {
                op: BinOp::Or,
                l: Box::new(l),
                r: Box::new(r),
                line,
            };
        }
        Ok(l)
    }
    fn and_expr(&mut self) -> PResult<Expr> {
        let mut l = self.not_expr()?;
        while self.is_kw("and") {
            let line = self.line();
            self.pos += 1;
            let r = self.not_expr()?;
            l = Expr::Bin {
                op: BinOp::And,
                l: Box::new(l),
                r: Box::new(r),
                line,
            };
        }
        Ok(l)
    }
    fn not_expr(&mut self) -> PResult<Expr> {
        if self.is_kw("not") {
            self.pos += 1;
            return Ok(Expr::Not(Box::new(self.not_expr()?)));
        }
        self.compare()
    }
    fn compare(&mut self) -> PResult<Expr> {
        let l = self.sum()?;
        let op = match self.peek() {
            Some(Tok::Sym("<")) => BinOp::Lt,
            Some(Tok::Sym("<=")) => BinOp::Le,
            Some(Tok::Sym(">")) => BinOp::Gt,
            Some(Tok::Sym(">=")) => BinOp::Ge,
            Some(Tok::Sym("==")) => BinOp::Eq,
            Some(Tok::Sym("!=")) => BinOp::Ne,
            _ => return Ok(l),
        };
        let line = self.line();
        self.pos += 1;
        let r = self.sum()?;
        if matches!(
            self.peek(),
            Some(Tok::Sym("<" | "<=" | ">" | ">=" | "==" | "!="))
        ) {
            return Err(Diag::err(
                line,
                "comparisons do not chain — write a < b and b < c",
            ));
        }
        Ok(Expr::Bin {
            op,
            l: Box::new(l),
            r: Box::new(r),
            line,
        })
    }
    fn sum(&mut self) -> PResult<Expr> {
        let mut l = self.product()?;
        loop {
            let op = match self.peek() {
                Some(Tok::Sym("+")) => BinOp::Add,
                Some(Tok::Sym("-")) => BinOp::Sub,
                _ => return Ok(l),
            };
            let line = self.line();
            self.pos += 1;
            let r = self.product()?;
            l = Expr::Bin {
                op,
                l: Box::new(l),
                r: Box::new(r),
                line,
            };
        }
    }
    fn product(&mut self) -> PResult<Expr> {
        let mut l = self.unary()?;
        loop {
            let op = match self.peek() {
                Some(Tok::Sym("*")) => BinOp::Mul,
                Some(Tok::Sym("/")) => BinOp::Div,
                _ => return Ok(l),
            };
            let line = self.line();
            self.pos += 1;
            let r = self.unary()?;
            l = Expr::Bin {
                op,
                l: Box::new(l),
                r: Box::new(r),
                line,
            };
        }
    }
    fn unary(&mut self) -> PResult<Expr> {
        if self.is_sym("-") {
            self.pos += 1;
            return Ok(Expr::Neg(Box::new(self.unary()?)));
        }
        if self.is_sym("+") {
            self.pos += 1;
            return self.unary();
        }
        self.power()
    }
    fn power(&mut self) -> PResult<Expr> {
        let base = self.atom()?;
        if self.is_sym("^") {
            let line = self.line();
            self.pos += 1;
            let exp = self.unary()?;
            return Ok(Expr::Bin {
                op: BinOp::Pow,
                l: Box::new(base),
                r: Box::new(exp),
                line,
            });
        }
        Ok(base)
    }
    fn unit_after(&mut self) -> PResult<Option<(f64, Dim)>> {
        if let Some(Tok::Unit(u)) = self.peek().cloned() {
            let line = self.line();
            self.pos += 1;
            return parse_unit(&u).map(Some).map_err(|e| Diag::err(line, e));
        }
        Ok(None)
    }
    fn atom(&mut self) -> PResult<Expr> {
        let line = self.line();
        match self.next() {
            Some(Tok::Num(v)) => {
                let unit = self.unit_after()?;
                let (f, dim) = unit.unwrap_or((1.0, Dim::NONE));
                Ok(Expr::Num {
                    si: v * f,
                    dim,
                    bare: unit.is_none(),
                    line,
                })
            }
            Some(Tok::Ident(n)) if n == "true" || n == "false" => Ok(Expr::Bool {
                value: n == "true",
                line,
            }),
            Some(Tok::Ident(n)) if KEYWORDS.contains(&n.as_str()) => Err(Diag::err(
                line,
                format!("«{n}» cannot stand in an expression"),
            )),
            Some(Tok::Ident(n)) => {
                if self.is_sym("(") {
                    self.pos += 1;
                    let mut args = Vec::new();
                    if !self.is_sym(")") {
                        loop {
                            args.push(self.expr()?);
                            if self.is_sym(",") {
                                self.pos += 1;
                                continue;
                            }
                            break;
                        }
                    }
                    self.expect_sym(")")?;
                    Ok(Expr::Call {
                        name: n,
                        args,
                        line,
                    })
                } else {
                    Ok(Expr::Var { name: n, line })
                }
            }
            Some(Tok::Sym("(")) => {
                let e = self.expr()?;
                self.expect_sym(")")?;
                Ok(e)
            }
            Some(Tok::Sym("[")) => {
                let mut vals = Vec::new();
                loop {
                    let neg = if self.is_sym("-") {
                        self.pos += 1;
                        true
                    } else {
                        false
                    };
                    match self.next() {
                        Some(Tok::Num(v)) => vals.push(if neg { -v } else { v }),
                        _ => return Err(Diag::err(
                            line,
                            "a table holds plain numbers, with its unit after the closing bracket",
                        )),
                    }
                    if self.is_sym(",") {
                        self.pos += 1;
                        continue;
                    }
                    break;
                }
                self.expect_sym("]")?;
                let (f, dim) = self.unit_after()?.unwrap_or((1.0, Dim::NONE));
                Ok(Expr::Table {
                    si: vals.into_iter().map(|v| v * f).collect(),
                    dim,
                    line,
                })
            }
            Some(Tok::Unit(u)) => Err(Diag::err(
                line,
                format!("a unit [{u}] belongs straight after a number"),
            )),
            _ => Err(Diag::err(line, "expected a value here")),
        }
    }
}

/// Read a method. Only the grammar is checked here; [`check`] does the rest.
pub fn parse(src: &str) -> Result<Program, Diag> {
    let toks = lex(src)?;
    let mut p = Parser { toks, pos: 0 };
    let body = p.block(&[])?;
    if body.is_empty() {
        return Err(Diag::err(0, "the method is empty"));
    }
    Ok(Program { body })
}

// ---------------------------------------------------------------------------
// checking: names, dimensions, and that every path ends

/// What a method is checked against: the node's inputs, by binding name, with
/// their dimensions, and the dimension of its answer.
#[derive(Clone, Debug)]
pub struct Signature {
    pub inputs: Vec<(String, Dim)>,
    pub output: Dim,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Ty {
    Num(Dim),
    /// A bare `0` or a bare literal, which adopts the other side's dimension
    /// in a sum or comparison. Only zero does: `h < 0` is fine, `h < 5` is
    /// not, because 5 of what is exactly the question.
    Zero,
    Pure(f64),
    Bool,
}

impl Ty {
    fn dim(self) -> Option<Dim> {
        match self {
            Ty::Num(d) => Some(d),
            Ty::Zero | Ty::Pure(_) => Some(Dim::NONE),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Input,
    Kernel,
    Const,
    Let,
    Loop,
}

struct Checker<'a> {
    scopes: Vec<BTreeMap<String, (Kind, Dim)>>,
    diags: Vec<Diag>,
    read: BTreeSet<String>,
    sig: &'a Signature,
    iterations: u64,
}

impl Checker<'_> {
    fn lookup(&self, name: &str) -> Option<(Kind, Dim)> {
        for s in self.scopes.iter().rev() {
            if let Some(v) = s.get(name) {
                return Some(*v);
            }
        }
        if let Some((_, d)) = kernel_constant(name) {
            return Some((Kind::Kernel, d));
        }
        None
    }
    fn err(&mut self, line: usize, msg: impl Into<String>) {
        self.diags.push(Diag::err(line, msg));
    }
    fn define(&mut self, name: &str, kind: Kind, dim: Dim, line: usize) {
        if RUST_UNRAW.contains(&name) || name.starts_with("__") || name == "_" {
            self.err(
                line,
                format!("«{name}» cannot be a name in a method — choose another"),
            );
            return;
        }
        if let Some((k, _)) = self.lookup(name) {
            let what = match k {
                Kind::Input => "an input of this node",
                Kind::Kernel => "a kernel constant",
                _ => "already defined",
            };
            self.err(line, format!("«{name}» is {what} — choose another name"));
            return;
        }
        self.scopes
            .last_mut()
            .expect("a scope")
            .insert(name.to_string(), (kind, dim));
    }

    /// Both sides of a sum or comparison, as one dimension.
    fn like(&mut self, a: Ty, b: Ty, line: usize, what: &str) -> Option<Dim> {
        match (a, b) {
            (Ty::Zero, Ty::Zero) => Some(Dim::NONE),
            (Ty::Zero, t) | (t, Ty::Zero) => t.dim(),
            _ => {
                let (Some(x), Some(y)) = (a.dim(), b.dim()) else {
                    self.err(line, format!("{what} needs two numbers"));
                    return None;
                };
                if x != y {
                    self.err(
                        line,
                        format!(
                            "{what} of unlike quantities: {} and {}",
                            dim_text(x),
                            dim_text(y)
                        ),
                    );
                    return None;
                }
                Some(x)
            }
        }
    }

    fn expr(&mut self, e: &Expr) -> Option<Ty> {
        match e {
            Expr::Num { si, dim, bare, .. } => Some(if *bare && *si == 0.0 {
                Ty::Zero
            } else if *bare {
                Ty::Pure(*si)
            } else {
                Ty::Num(*dim)
            }),
            Expr::Bool { .. } => Some(Ty::Bool),
            Expr::Var { name, line } => {
                match self.lookup(name) {
                    Some((kind, d)) => {
                        if kind == Kind::Input {
                            self.read.insert(name.clone());
                        }
                        Some(Ty::Num(d))
                    }
                    None => {
                        let hint = if function(name).is_some() {
                            " — it is a function; call it with brackets".to_string()
                        } else {
                            String::new()
                        };
                        self.err(
                        *line,
                        format!("«{name}» is not an input, a constant or a value made with let{hint}"),
                    );
                        None
                    }
                }
            }
            Expr::Neg(x) => match self.expr(x)? {
                Ty::Bool => {
                    self.err(e.line(), "a condition cannot be negated with - (use not)");
                    None
                }
                Ty::Pure(v) => Some(Ty::Pure(-v)),
                t => Some(t),
            },
            Expr::Not(x) => {
                if self.expr(x)? != Ty::Bool {
                    self.err(e.line(), "not applies to a condition");
                    return None;
                }
                Some(Ty::Bool)
            }
            Expr::Table { line, .. } => {
                self.err(*line, "a table stands only inside interp(…)");
                None
            }
            Expr::Bin { op, l, r, line } => self.binary(*op, l, r, *line),
            Expr::Call { name, args, line } => self.call(name, args, *line),
        }
    }

    fn binary(&mut self, op: BinOp, l: &Expr, r: &Expr, line: usize) -> Option<Ty> {
        if op == BinOp::Pow {
            let a = self.expr(l)?;
            return self.power(a, r, line);
        }
        let (a, b) = (self.expr(l), self.expr(r));
        let (a, b) = (a?, b?);
        match op {
            BinOp::And | BinOp::Or => {
                if a != Ty::Bool || b != Ty::Bool {
                    self.err(line, format!("{} joins two conditions", op.symbol()));
                    return None;
                }
                Some(Ty::Bool)
            }
            _ if op.is_compare() => {
                self.like(a, b, line, "a comparison")?;
                Some(Ty::Bool)
            }
            BinOp::Add | BinOp::Sub => {
                if let (Ty::Pure(x), Ty::Pure(y)) = (a, b) {
                    return Some(Ty::Pure(if op == BinOp::Add { x + y } else { x - y }));
                }
                let what = if op == BinOp::Add {
                    "a sum"
                } else {
                    "a difference"
                };
                self.like(a, b, line, what).map(Ty::Num)
            }
            BinOp::Mul | BinOp::Div => {
                if let (Ty::Pure(x), Ty::Pure(y)) = (a, b) {
                    return Some(Ty::Pure(if op == BinOp::Mul { x * y } else { x / y }));
                }
                let (Some(x), Some(y)) = (a.dim(), b.dim()) else {
                    self.err(line, format!("{} needs two numbers", op.symbol()));
                    return None;
                };
                Some(Ty::Num(if op == BinOp::Mul {
                    dim_mul(x, y)
                } else {
                    dim_div(x, y)
                }))
            }
            _ => unreachable!("every operator is handled above"),
        }
    }

    fn power(&mut self, base: Ty, exp: &Expr, line: usize) -> Option<Ty> {
        let Some(d) = base.dim() else {
            self.err(line, "^ raises a number");
            return None;
        };
        if let Some(p) = exp.literal() {
            self.expr(exp)?;
            if let Ty::Pure(b) = base {
                return Some(Ty::Pure(pmath::powf(b, p)));
            }
            if is_none(d) {
                return Some(Ty::Num(d));
            }
            // A dimensioned base needs a power that keeps every exponent whole:
            // m^2 ^ 0.5 is fine, m ^ 0.5 is not a quantity.
            for den in [1, 2, 3, 4, 6] {
                let num = p * den as f64;
                if num.fract() == 0.0 && num.abs() < 100.0 {
                    if let Some(r) = dim_pow(d, num as i32, den) {
                        return Some(Ty::Num(r));
                    }
                }
            }
            self.err(
                line,
                format!("{} to the power {p} is not a quantity", dim_text(d)),
            );
            return None;
        }
        let t = self.expr(exp)?;
        if !is_none(d) || t.dim() != Some(Dim::NONE) {
            self.err(
                line,
                "a computed power needs a pure number on both sides — a dimensioned value takes a written power, like r ^ 3",
            );
            return None;
        }
        Some(Ty::Num(Dim::NONE))
    }

    fn call(&mut self, name: &str, args: &[Expr], line: usize) -> Option<Ty> {
        let Some(f) = function(name) else {
            let hint = if self.lookup(name).is_some() {
                format!(" — «{name}» is a value, not a function")
            } else {
                " — the functions are listed in docs/PSEUDOCODE.md".to_string()
            };
            self.err(
                line,
                format!("«{name}» is not a function of the language{hint}"),
            );
            return None;
        };
        if args.len() != f.arity {
            self.err(
                line,
                format!(
                    "{name} takes {} argument{}, not {}",
                    f.arity,
                    if f.arity == 1 { "" } else { "s" },
                    args.len()
                ),
            );
            return None;
        }
        if f.rule == FnRule::Interp {
            return self.interp(args, line);
        }
        if f.rule == FnRule::Pow {
            let b = self.expr(&args[0])?;
            return self.power(b, &args[1], line);
        }
        let mut tys = Vec::new();
        for a in args {
            tys.push(self.expr(a)?);
        }
        let dims: Vec<Dim> = tys.iter().map(|t| t.dim().unwrap_or(Dim::NONE)).collect();
        if tys.iter().any(|t| t.dim().is_none()) {
            self.err(line, format!("{name} takes numbers, not conditions"));
            return None;
        }
        match f.rule {
            FnRule::Pure => {
                if !is_none(dims[0]) {
                    self.err(
                        line,
                        format!(
                            "{name} of a {} value — it needs a pure number (divide by a reference first)",
                            dim_text(dims[0])
                        ),
                    );
                    return None;
                }
                Some(Ty::Num(Dim::NONE))
            }
            FnRule::Same => Some(Ty::Num(dims[0])),
            FnRule::SameTwo | FnRule::Ratio => {
                let d = self.like(tys[0], tys[1], line, name)?;
                Some(Ty::Num(if f.rule == FnRule::Ratio {
                    Dim::NONE
                } else {
                    d
                }))
            }
            FnRule::Sqrt | FnRule::Cbrt => {
                let den = if f.rule == FnRule::Sqrt { 2 } else { 3 };
                match dim_pow(dims[0], 1, den) {
                    Some(d) => Some(Ty::Num(d)),
                    None => {
                        self.err(
                            line,
                            format!("{name} of {} is not a quantity", dim_text(dims[0])),
                        );
                        None
                    }
                }
            }
            FnRule::Interp | FnRule::Pow => unreachable!("handled above"),
        }
    }

    fn interp(&mut self, args: &[Expr], line: usize) -> Option<Ty> {
        let x = self.expr(&args[0])?;
        let (
            Expr::Table {
                si: xs, dim: dx, ..
            },
            Expr::Table {
                si: ys, dim: dy, ..
            },
        ) = (&args[1], &args[2])
        else {
            self.err(
                line,
                "interp(x, [x1, x2, …] [unit], [y1, y2, …] [unit]) — the table is written out",
            );
            return None;
        };
        if xs.len() != ys.len() || xs.len() < 2 {
            self.err(
                line,
                format!(
                    "the table's two rows must be the same length, at least 2 ({} and {})",
                    xs.len(),
                    ys.len()
                ),
            );
            return None;
        }
        if xs.windows(2).any(|w| w[1] <= w[0]) {
            self.err(line, "the table's x row must rise strictly");
            return None;
        }
        self.like(x, Ty::Num(*dx), line, "interp's x and its table")?;
        Some(Ty::Num(*dy))
    }

    /// Check a block, and say whether every path through it ends in return
    /// or refuse.
    fn block(&mut self, body: &[Stmt]) -> bool {
        let mut ends = false;
        for s in body {
            if ends {
                self.err(
                    s.line(),
                    "this can never run — the line before it always ends the method",
                );
                break;
            }
            ends = self.stmt(s);
        }
        ends
    }

    fn stmt(&mut self, s: &Stmt) -> bool {
        match s {
            Stmt::Let {
                name,
                quantity,
                expr,
                line,
            } => {
                let t = self.expr(expr);
                let d = match t {
                    Some(Ty::Bool) => {
                        self.err(*line, "a value made with let is a number, not a condition");
                        None
                    }
                    Some(t) => t.dim(),
                    None => None,
                };
                if let (Some(q), Some(d)) = (quantity, d) {
                    match quantity_dim(q) {
                        None => self.err(*line, format!("«{q}» is not a quantity type")),
                        Some(qd) if qd != d => self.err(
                            *line,
                            format!(
                                "{name} is declared {q} ({}) but is {}",
                                dim_text(qd),
                                dim_text(d)
                            ),
                        ),
                        _ => {}
                    }
                }
                self.define(name, Kind::Let, d.unwrap_or(Dim::NONE), *line);
                false
            }
            Stmt::Const { name, expr, line } => {
                let d = self.expr(expr).and_then(|t| t.dim()).unwrap_or(Dim::NONE);
                self.define(name, Kind::Const, d, *line);
                false
            }
            Stmt::Set { name, expr, line } => {
                let t = self.expr(expr);
                match self.lookup(name) {
                    None => self.err(*line, format!("«{name}» has not been made with let yet")),
                    Some((Kind::Let, d)) => {
                        if let Some(t) = t {
                            self.like(Ty::Num(d), t, *line, "set")
                                .map(|_| ())
                                .unwrap_or(());
                        }
                    }
                    Some((k, _)) => {
                        let what = match k {
                            Kind::Input => "an input",
                            Kind::Loop => "the loop's counter",
                            _ => "a constant",
                        };
                        self.err(*line, format!("«{name}» is {what} and cannot be changed"));
                    }
                }
                false
            }
            Stmt::If {
                arms, otherwise, ..
            } => {
                let mut all = true;
                for (cond, body) in arms {
                    if let Some(t) = self.expr(cond) {
                        if t != Ty::Bool {
                            self.err(cond.line(), "if needs a condition, such as h < 0 [m]");
                        }
                    }
                    self.scopes.push(BTreeMap::new());
                    all &= self.block(body);
                    self.scopes.pop();
                }
                match otherwise {
                    Some(body) => {
                        self.scopes.push(BTreeMap::new());
                        all &= self.block(body);
                        self.scopes.pop();
                    }
                    None => all = false,
                }
                all
            }
            Stmt::For {
                var,
                first,
                last,
                body,
                line,
            } => {
                if last < first {
                    self.err(
                        *line,
                        format!("the loop runs from {first} to {last}, which is never"),
                    );
                }
                self.iterations = self
                    .iterations
                    .saturating_mul(1)
                    .saturating_add((last - first + 1).max(0) as u64);
                if self.iterations > MAX_STEPS {
                    self.err(
                        *line,
                        format!("more than {MAX_STEPS} loop steps — a method this long is a node to split"),
                    );
                }
                self.scopes.push(BTreeMap::new());
                self.define(var, Kind::Loop, Dim::NONE, *line);
                self.block(body);
                self.scopes.pop();
                false
            }
            Stmt::Refuse { reason, line } => {
                if reason.trim().is_empty() {
                    self.err(*line, "a refusal says why");
                }
                true
            }
            Stmt::Return { expr, line } => {
                match self.expr(expr) {
                    Some(Ty::Bool) => self.err(*line, "the answer is a number, not a condition"),
                    Some(t) => {
                        let want = self.sig.output;
                        match t {
                            Ty::Zero => {}
                            Ty::Pure(_) if is_none(want) => {}
                            _ => {
                                let d = t.dim().unwrap_or(Dim::NONE);
                                if d != want {
                                    self.err(
                                        *line,
                                        format!(
                                            "the answer must be {} and this is {}",
                                            dim_text(want),
                                            dim_text(d)
                                        ),
                                    );
                                }
                            }
                        }
                    }
                    None => {}
                }
                true
            }
        }
    }
}

/// Check a parsed method against the node it belongs to. Errors and notes,
/// in line order; the method is sound when none is an error.
pub fn check(p: &Program, sig: &Signature) -> Vec<Diag> {
    let mut c = Checker {
        scopes: vec![BTreeMap::new()],
        diags: Vec::new(),
        read: BTreeSet::new(),
        sig,
        iterations: 0,
    };
    for (name, d) in &sig.inputs {
        if kernel_constant(name).is_some() {
            c.err(
                0,
                format!("the input «{name}» has the name of a kernel constant"),
            );
        }
        c.scopes[0].insert(name.clone(), (Kind::Input, *d));
    }
    let ends = c.block(&p.body);
    if !ends {
        let line = p.body.last().map(|s| s.line()).unwrap_or(0);
        c.err(
            line,
            "some path reaches the end without return or refuse — every path must end in one",
        );
    }
    for (name, _) in &sig.inputs {
        if !c.read.contains(name) {
            c.diags.push(Diag::warn(
                0,
                format!("the input «{name}» is declared but the method never reads it"),
            ));
        }
    }
    c.diags.sort_by_key(|d| d.line);
    c.diags
}

/// Parse and check in one step: the program, or every error.
pub fn compile(src: &str, sig: &Signature) -> Result<Program, Vec<Diag>> {
    let p = parse(src).map_err(|d| vec![d])?;
    let diags = check(&p, sig);
    if diags.iter().any(|d| d.severity == Severity::Error) {
        return Err(diags);
    }
    Ok(p)
}

// ---------------------------------------------------------------------------
// running

/// What a method gave for one set of inputs.
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    /// The answer, in the SI unit of the node's quantity.
    Answer(f64),
    Refused {
        line: usize,
        reason: String,
    },
}

enum Flow {
    Next,
    Done(Outcome),
}

enum Val {
    Num(f64),
    Bool(bool),
}

struct Machine<'a> {
    scopes: Vec<BTreeMap<String, f64>>,
    inputs: &'a [(String, f64)],
    steps: u64,
}

impl Machine<'_> {
    fn get(&self, name: &str, line: usize) -> Result<f64, Diag> {
        for s in self.scopes.iter().rev() {
            if let Some(v) = s.get(name) {
                return Ok(*v);
            }
        }
        if let Some((_, v)) = self.inputs.iter().find(|(n, _)| n == name) {
            return Ok(*v);
        }
        if let Some((v, _)) = kernel_constant(name) {
            return Ok(v);
        }
        Err(Diag::err(line, format!("«{name}» has no value")))
    }
    fn set(&mut self, name: &str, v: f64) {
        for s in self.scopes.iter_mut().rev() {
            if let Some(slot) = s.get_mut(name) {
                *slot = v;
                return;
            }
        }
    }
    fn num(&mut self, e: &Expr) -> Result<f64, Diag> {
        match self.eval(e)? {
            Val::Num(v) => Ok(v),
            Val::Bool(_) => Err(Diag::err(e.line(), "a condition where a number belongs")),
        }
    }
    fn cond(&mut self, e: &Expr) -> Result<bool, Diag> {
        match self.eval(e)? {
            Val::Bool(b) => Ok(b),
            Val::Num(_) => Err(Diag::err(e.line(), "a number where a condition belongs")),
        }
    }
    fn eval(&mut self, e: &Expr) -> Result<Val, Diag> {
        let v = match e {
            Expr::Num { si, .. } => *si,
            Expr::Bool { value, .. } => return Ok(Val::Bool(*value)),
            Expr::Var { name, line } => self.get(name, *line)?,
            Expr::Neg(x) => -self.num(x)?,
            Expr::Not(x) => return Ok(Val::Bool(!self.cond(x)?)),
            Expr::Table { line, .. } => {
                return Err(Diag::err(*line, "a table stands only inside interp(…)"))
            }
            Expr::Bin { op, l, r, line } => {
                let line = *line;
                match op {
                    BinOp::And => {
                        return Ok(Val::Bool(self.cond(l)? && self.cond(r)?));
                    }
                    BinOp::Or => {
                        return Ok(Val::Bool(self.cond(l)? || self.cond(r)?));
                    }
                    _ => {}
                }
                let (a, b) = (self.num(l)?, self.num(r)?);
                match op {
                    BinOp::Add => a + b,
                    BinOp::Sub => a - b,
                    BinOp::Mul => a * b,
                    BinOp::Div => rt::div(a, b, line as u32).map_err(rt_diag)?,
                    BinOp::Pow => rt::pow(a, b),
                    BinOp::Lt => return Ok(Val::Bool(a < b)),
                    BinOp::Le => return Ok(Val::Bool(a <= b)),
                    BinOp::Gt => return Ok(Val::Bool(a > b)),
                    BinOp::Ge => return Ok(Val::Bool(a >= b)),
                    BinOp::Eq => return Ok(Val::Bool(a == b)),
                    BinOp::Ne => return Ok(Val::Bool(a != b)),
                    BinOp::And | BinOp::Or => unreachable!("handled above"),
                }
            }
            Expr::Call { name, args, line } => self.call(name, args, *line)?,
        };
        // Finiteness is checked where a value is named or returned — see
        // `rt::fin` — exactly as the translated code checks it.
        Ok(Val::Num(v))
    }
    fn call(&mut self, name: &str, args: &[Expr], line: usize) -> Result<f64, Diag> {
        if name == "interp" {
            let x = self.num(&args[0])?;
            let (Expr::Table { si: xs, .. }, Expr::Table { si: ys, .. }) = (&args[1], &args[2])
            else {
                return Err(Diag::err(line, "interp needs its table written out"));
            };
            return Ok(pmath::interp(x, xs, ys));
        }
        let mut a = Vec::with_capacity(args.len());
        for e in args {
            a.push(self.num(e)?);
        }
        let l = line as u32;
        match implementation(name) {
            Some(Impl::Plain1(f, _)) => Ok(f(a[0])),
            Some(Impl::Plain2(f, _)) => Ok(f(a[0], a[1])),
            Some(Impl::Checked1(f, _)) => f(a[0], l).map_err(rt_diag),
            Some(Impl::Checked2(f, _)) => f(a[0], a[1], l).map_err(rt_diag),
            None => Err(Diag::err(line, format!("«{name}» is not a function"))),
        }
    }
    fn block(&mut self, body: &[Stmt]) -> Result<Flow, Diag> {
        for s in body {
            if let Flow::Done(o) = self.stmt(s)? {
                return Ok(Flow::Done(o));
            }
        }
        Ok(Flow::Next)
    }
    fn stmt(&mut self, s: &Stmt) -> Result<Flow, Diag> {
        match s {
            Stmt::Let {
                name, expr, line, ..
            }
            | Stmt::Const { name, expr, line } => {
                let v = rt::fin(self.num(expr)?, *line as u32).map_err(rt_diag)?;
                self.scopes
                    .last_mut()
                    .expect("a scope")
                    .insert(name.clone(), v);
            }
            Stmt::Set { name, expr, line } => {
                let v = rt::fin(self.num(expr)?, *line as u32).map_err(rt_diag)?;
                self.set(name, v);
            }
            Stmt::If {
                arms, otherwise, ..
            } => {
                for (cond, body) in arms {
                    if self.cond(cond)? {
                        self.scopes.push(BTreeMap::new());
                        let f = self.block(body);
                        self.scopes.pop();
                        return f;
                    }
                }
                if let Some(body) = otherwise {
                    self.scopes.push(BTreeMap::new());
                    let f = self.block(body);
                    self.scopes.pop();
                    return f;
                }
            }
            Stmt::For {
                var,
                first,
                last,
                body,
                line,
            } => {
                for i in *first..=*last {
                    self.steps += 1;
                    if self.steps > MAX_STEPS {
                        return Err(Diag::err(*line, "too many loop steps"));
                    }
                    let mut scope = BTreeMap::new();
                    scope.insert(var.clone(), i as f64);
                    self.scopes.push(scope);
                    let f = self.block(body);
                    self.scopes.pop();
                    if let Flow::Done(o) = f? {
                        return Ok(Flow::Done(o));
                    }
                }
            }
            Stmt::Refuse { reason, line } => {
                return Ok(Flow::Done(Outcome::Refused {
                    line: *line,
                    reason: reason.clone(),
                }))
            }
            Stmt::Return { expr, line } => {
                let v = rt::fin(self.num(expr)?, *line as u32).map_err(rt_diag)?;
                return Ok(Flow::Done(Outcome::Answer(v)));
            }
        }
        Ok(Flow::Next)
    }
}

/// Run a checked method on one set of inputs, each in the SI unit of its
/// quantity. An error is a fault in the method or its inputs — a square root
/// of a negative number, say — and never a refusal: only the method's own
/// `refuse` refuses.
pub fn run(p: &Program, inputs: &[(String, f64)]) -> Result<Outcome, Diag> {
    let mut m = Machine {
        scopes: vec![BTreeMap::new()],
        inputs,
        steps: 0,
    };
    match m.block(&p.body)? {
        Flow::Done(o) => Ok(o),
        Flow::Next => Err(Diag::err(0, "the method ended without return or refuse")),
    }
}

// ---------------------------------------------------------------------------
// the author's cases, against the method

/// One of the author's test cases, in SI: the inputs by binding name, and
/// either the answer their own code gave or the fact that it must refuse.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Case {
    pub label: String,
    pub inputs: Vec<(String, f64)>,
    /// `None` for a case that must be refused.
    pub expect: Option<f64>,
    /// Relative, exactly as a fixture's: |got − expect| / |expect|, or |got|
    /// when the expected value is zero.
    pub tolerance: f64,
}

impl Case {
    pub fn refuses(&self) -> bool {
        self.expect.is_none()
    }
}

/// How many normal cases, and how many refusals, a node's author supplies at
/// least. Three answers, so one wrong case cannot be the only evidence; one
/// refusal, so the method is shown to say no where the author's code says no
/// — a method that answers everything is a method nobody has tried to break.
pub const MIN_CASES: usize = 3;
pub const MIN_REFUSALS: usize = 1;

#[derive(Clone, Debug, PartialEq)]
pub enum Verdict {
    /// Within tolerance, or refused as the author said it must be.
    Agrees,
    /// Answered, and further from the author's value than the tolerance.
    Differs { got: f64, relative: f64 },
    /// Answered where the author's code refused.
    AnsweredRefusal { got: f64 },
    /// Refused where the author's code answered.
    RefusedAnswer { reason: String },
    /// The method faulted: a square root of a negative number, say.
    Faulted(String),
    /// The case names an input the node does not have, or leaves one out.
    Malformed(String),
}

impl Verdict {
    pub fn agrees(&self) -> bool {
        *self == Verdict::Agrees
    }
    /// One line, for a terminal, the gate and the form.
    pub fn text(&self, c: &Case) -> String {
        match self {
            Verdict::Agrees if c.refuses() => "refused, as your code does".into(),
            Verdict::Agrees => "agrees with your code".into(),
            Verdict::Differs { got, relative } => format!(
                "the method gives {}, your code gave {} — {:.2e} apart (relative), more than your tolerance {:e}",
                show(*got),
                show(c.expect.unwrap_or(0.0)),
                relative,
                c.tolerance
            ),
            Verdict::AnsweredRefusal { got } => format!(
                "your code refuses this case, but the method answers {}",
                show(*got)
            ),
            Verdict::RefusedAnswer { reason } => format!(
                "your code answers this case, but the method refuses it: {reason}"
            ),
            Verdict::Faulted(m) => format!("the method faults here: {m}"),
            Verdict::Malformed(m) => m.clone(),
        }
    }
}

/// A number as a person reads it: plainly where that is short, in powers of
/// ten where it is not.
pub fn show(v: f64) -> String {
    let a = v.abs();
    if a == 0.0 || (1e-4..1e9).contains(&a) {
        format!("{v}")
    } else {
        format!("{v:e}")
    }
}

fn relative_error(got: f64, expected: f64) -> f64 {
    if expected == 0.0 {
        pmath::abs(got)
    } else {
        pmath::abs((got - expected) / expected)
    }
}

/// What the method gave for one case, when it gave a number.
pub fn value_of(p: &Program, c: &Case) -> Option<f64> {
    match run(p, &c.inputs) {
        Ok(Outcome::Answer(v)) => Some(v),
        _ => None,
    }
}

/// Run one case and say whether the method agrees with the author's code.
pub fn judge(p: &Program, sig: &Signature, c: &Case) -> Verdict {
    for (name, _) in &sig.inputs {
        if !c.inputs.iter().any(|(n, _)| n == name) {
            return Verdict::Malformed(format!("this case gives no value for the input «{name}»"));
        }
    }
    for (name, _) in &c.inputs {
        if !sig.inputs.iter().any(|(n, _)| n == name) {
            return Verdict::Malformed(format!("«{name}» is not an input of this node"));
        }
    }
    match (run(p, &c.inputs), c.expect) {
        (Err(d), _) => Verdict::Faulted(d.to_string()),
        (Ok(Outcome::Refused { .. }), None) => Verdict::Agrees,
        (Ok(Outcome::Refused { reason, .. }), Some(_)) => Verdict::RefusedAnswer { reason },
        (Ok(Outcome::Answer(got)), None) => Verdict::AnsweredRefusal { got },
        (Ok(Outcome::Answer(got)), Some(want)) => {
            let relative = relative_error(got, want);
            if relative <= c.tolerance {
                Verdict::Agrees
            } else {
                Verdict::Differs { got, relative }
            }
        }
    }
}

/// Everything the checker has to say about one node's method and cases.
#[derive(Clone, Debug, Default)]
pub struct Report {
    /// The method's own findings: grammar, names, units, paths.
    pub diags: Vec<Diag>,
    /// Each case with its verdict, in the order given. Empty when the method
    /// has errors, since nothing can be run.
    pub cases: Vec<(Case, Verdict)>,
    /// What the method gave for each case, when it gave a number, so a face
    /// can show it beside the author's value.
    pub got: Vec<Option<f64>>,
    /// What is missing from the set of cases: too few answers, no refusal.
    pub shortfall: Vec<String>,
}

impl Report {
    /// Sound: the method checks, every case agrees, and the set is complete.
    pub fn sound(&self) -> bool {
        !self.diags.iter().any(|d| d.severity == Severity::Error)
            && self.cases.iter().all(|(_, v)| v.agrees())
            && self.shortfall.is_empty()
    }

    /// As JSON, for the form and the web face.
    pub fn json(&self) -> String {
        let q = jstr;
        let mut o = String::from("{");
        let _ = write!(
            o,
            "\"sound\": {}, \"version\": {LANGUAGE_VERSION}, \"diags\": [",
            self.sound()
        );
        for (i, d) in self.diags.iter().enumerate() {
            let _ = write!(
                o,
                "{}{{\"line\": {}, \"severity\": {}, \"msg\": {}}}",
                if i > 0 { ", " } else { "" },
                d.line,
                q(match d.severity {
                    Severity::Error => "error",
                    Severity::Warning => "note",
                }),
                q(&d.msg)
            );
        }
        o.push_str("], \"cases\": [");
        for (i, (c, v)) in self.cases.iter().enumerate() {
            let got = match v {
                Verdict::Differs { got, .. } | Verdict::AnsweredRefusal { got } => {
                    format!("{got:e}")
                }
                _ => "null".into(),
            };
            let _ = write!(
                o,
                "{}{{\"label\": {}, \"agrees\": {}, \"refuse\": {}, \"got\": {got}, \"text\": {}}}",
                if i > 0 { ", " } else { "" },
                q(&c.label),
                v.agrees(),
                c.refuses(),
                q(&v.text(c))
            );
        }
        o.push_str("], \"shortfall\": [");
        for (i, s) in self.shortfall.iter().enumerate() {
            let _ = write!(o, "{}{}", if i > 0 { ", " } else { "" }, q(s));
        }
        o.push_str("]}");
        o
    }
}

/// A JSON string. Its own copy rather than the form module's, so the checker
/// the node form carries is built from this file and the units alone — see
/// [`CHECKER_SOURCES`].
fn jstr(v: &str) -> String {
    let mut o = String::with_capacity(v.len() + 2);
    o.push('"');
    for c in v.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(o, "\\u{:04x}", c as u32);
            }
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// The files the node form's checker is compiled from. `web/method.wasm.gz` is
/// built from these by `cargo run -p xtask -- method-wasm`, which records
/// their fingerprint beside it; a test refuses a checkout whose sources have
/// moved on from the checker every form carries.
pub const CHECKER_SOURCES: &[&str] = &[
    "crates/vleo-sheet/src/method.rs",
    "crates/vleo-sheet/src/lesson.rs",
    "crates/vleo-units/src/pmath.rs",
    "crates/vleo-units/src/method_rt.rs",
    "crates/vleo-units/src/unit.rs",
    "crates/vleo-units/src/quantity.rs",
    "crates/vleo-units/src/constants.rs",
    "crates/vleo-method-wasm/src/lib.rs",
    "crates/vleo-method-wasm/Cargo.toml",
];

/// The fingerprint of [`CHECKER_SOURCES`] in a checkout: line endings made
/// one kind first, so a Windows checkout agrees with Linux.
pub fn checker_fingerprint(root: &std::path::Path) -> Result<String, String> {
    let mut all = String::new();
    for f in CHECKER_SOURCES {
        let t = std::fs::read_to_string(root.join(f)).map_err(|e| format!("{f}: {e}"))?;
        all.push_str(f);
        all.push('\n');
        all.push_str(&t.replace("\r\n", "\n"));
    }
    Ok(format!("{:016x}", crate::fnv1a(&all)))
}

/// A report that could not be made, as JSON of the same shape.
pub fn error_json(msg: &str) -> String {
    format!(
        "{{\"sound\": false, \"version\": {LANGUAGE_VERSION}, \"error\": {}, \"diags\": [], \"cases\": [], \"shortfall\": []}}",
        jstr(msg)
    )
}

/// Check a method and run the author's cases through it.
pub fn report(src: &str, sig: &Signature, cases: &[Case]) -> Report {
    let mut r = Report::default();
    let normal = cases.iter().filter(|c| !c.refuses()).count();
    let refusals = cases.len() - normal;
    if normal < MIN_CASES {
        r.shortfall.push(format!(
            "{normal} case(s) with an answer — at least {MIN_CASES}, so no single case is the only evidence"
        ));
    }
    if refusals < MIN_REFUSALS {
        r.shortfall.push(format!(
            "no case that must be refused — at least {MIN_REFUSALS}, to show the method says no where your code does"
        ));
    }
    for c in cases {
        let tolerance_ok = c.tolerance > 0.0 && c.tolerance.is_finite();
        if !c.refuses() && !tolerance_ok {
            r.shortfall.push(format!(
                "«{}» has no tolerance above zero — say how close is close enough",
                c.label
            ));
        }
    }
    let p = match parse(src) {
        Ok(p) => p,
        Err(d) => {
            r.diags.push(d);
            return r;
        }
    };
    r.diags = check(&p, sig);
    if r.diags.iter().any(|d| d.severity == Severity::Error) {
        return r;
    }
    for c in cases {
        r.cases.push((c.clone(), judge(&p, sig, c)));
        r.got.push(value_of(&p, c));
    }
    r
}

/// The same, from a sheet's own text: `[method] text`, `[output] type`, every
/// `[[input]]` and every `[[case]]`. This is what the node form sends from the
/// browser, through WebAssembly, and what intake and the gate read from the
/// file — one function, so the three cannot disagree.
pub fn report_toml(text: &str) -> Result<Report, String> {
    let v: toml::Value = text.parse().map_err(|e| format!("not TOML: {e}"))?;
    let src = v
        .get("method")
        .and_then(|m| m.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("");
    let out = v
        .get("output")
        .and_then(|o| o.get("type"))
        .and_then(|t| t.as_str())
        .unwrap_or("");
    let output = quantity_dim(out)
        .ok_or_else(|| format!("the answer's quantity «{out}» is not one this tool has"))?;
    let mut inputs = Vec::new();
    for i in v
        .get("input")
        .and_then(|a| a.as_array())
        .into_iter()
        .flatten()
    {
        let b = i.get("binding").and_then(|x| x.as_str()).unwrap_or("");
        let t = i.get("type").and_then(|x| x.as_str()).unwrap_or("");
        let d = quantity_dim(t)
            .ok_or_else(|| format!("the input «{b}» has no known quantity «{t}»"))?;
        inputs.push((b.to_string(), d));
    }
    let cases = cases_of(&v)?;
    Ok(report(src, &Signature { inputs, output }, &cases))
}

/// The same report from the plain form the node form's checker is handed —
/// no TOML parser, so the WebAssembly every form carries does not need one.
///
/// ```text
/// output Velocity
/// input r Length
/// case 0|1 <expect> <tolerance> <name=value;name=value> <label …>
/// method
/// <the method, to the end>
/// ```
///
/// `case 1` must be refused; its expect and tolerance are `-`.
pub fn report_plain(text: &str) -> Result<Report, String> {
    let mut output = None;
    let mut inputs = Vec::new();
    let mut cases = Vec::new();
    let mut lines = text.lines();
    let mut src = String::new();
    while let Some(l) = lines.next() {
        let mut w = l.splitn(2, ' ');
        match (w.next().unwrap_or(""), w.next().unwrap_or("").trim()) {
            ("output", q) => {
                output = Some(quantity_dim(q).ok_or_else(|| {
                    format!("the answer's quantity «{q}» is not one this tool has")
                })?)
            }
            ("input", rest) => {
                let (b, q) = rest.split_once(' ').unwrap_or((rest, ""));
                let d = quantity_dim(q.trim())
                    .ok_or_else(|| format!("the input «{b}» has no known quantity «{q}»"))?;
                inputs.push((b.to_string(), d));
            }
            ("case", rest) => {
                let f: Vec<&str> = rest.splitn(5, ' ').collect();
                if f.len() < 4 {
                    return Err(format!("a case line is short: «{l}»"));
                }
                let num = |x: &str| {
                    x.parse::<f64>()
                        .map_err(|_| format!("«{x}» is not a number"))
                };
                let refuse = f[0] == "1";
                let mut ins = Vec::new();
                for kv in f[3].split(';').filter(|x| !x.is_empty()) {
                    let (k, v) = kv
                        .split_once('=')
                        .ok_or_else(|| format!("«{kv}» is not name=value"))?;
                    ins.push((k.to_string(), num(v)?));
                }
                cases.push(Case {
                    label: f.get(4).unwrap_or(&"").to_string(),
                    inputs: ins,
                    expect: if refuse { None } else { Some(num(f[1])?) },
                    tolerance: if refuse { 0.0 } else { num(f[2])? },
                });
            }
            ("method", _) => {
                src = lines.collect::<Vec<_>>().join("\n");
                break;
            }
            ("", _) => {}
            (other, _) => return Err(format!("«{other}» is not a line the checker reads")),
        }
    }
    let output = output.ok_or("the answer's quantity is not given")?;
    Ok(report(&src, &Signature { inputs, output }, &cases))
}

/// The `[[case]]` blocks of a parsed sheet.
pub fn cases_of(v: &toml::Value) -> Result<Vec<Case>, String> {
    let mut out = Vec::new();
    for (i, c) in v
        .get("case")
        .and_then(|a| a.as_array())
        .into_iter()
        .flatten()
        .enumerate()
    {
        let num = |x: &toml::Value| x.as_float().or_else(|| x.as_integer().map(|n| n as f64));
        let label = c
            .get("label")
            .and_then(|x| x.as_str())
            .map(String::from)
            .unwrap_or_else(|| format!("case {}", i + 1));
        let refuse = c.get("refuse").and_then(|x| x.as_str()) == Some("yes");
        let expect = if refuse {
            None
        } else {
            Some(c.get("expect").and_then(num).ok_or_else(|| {
                format!("«{label}» has no expected value (or say refuse = \"yes\")")
            })?)
        };
        let mut inputs = Vec::new();
        if let Some(t) = c.get("inputs").and_then(|x| x.as_table()) {
            for (k, x) in t {
                let n =
                    num(x).ok_or_else(|| format!("«{label}»: the input «{k}» is not a number"))?;
                inputs.push((k.clone(), n));
            }
        }
        out.push(Case {
            label,
            inputs,
            expect,
            tolerance: c.get("tolerance").and_then(num).unwrap_or(0.0),
        });
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// a node's method

/// What a node's method is checked against: its inputs in declared order, and
/// its answer. `None` when a quantity is not one the tool has.
pub fn node_signature(sh: &crate::model::Sheet) -> Option<Signature> {
    let mut inputs = Vec::new();
    for i in &sh.inputs {
        inputs.push((i.binding.clone(), quantity_dim(&i.ty)?));
    }
    Some(Signature {
        inputs,
        output: quantity_dim(&sh.ty)?,
    })
}

/// A node's method, when it has one the tool can generate code from: written,
/// checking without error, on a row with one computed answer. Anything else
/// keeps its hand-written holes — and the gate says why.
pub fn node_program(sh: &crate::model::Sheet) -> Option<Program> {
    if sh.method.text.trim().is_empty() || sh.is_declared() || !sh.publishes.is_empty() {
        return None;
    }
    compile(&sh.method.text, &node_signature(sh)?).ok()
}

/// The node's kernel function, `vleo_core::physics::methods::<module>`, when
/// it has a method to translate.
pub fn node_rust(sh: &crate::model::Sheet) -> Option<String> {
    let p = node_program(sh)?;
    let inputs: Vec<String> = sh.inputs.iter().map(|i| i.binding.clone()).collect();
    Some(to_rust(&p, &sh.id, &sh.source, &sh.method.text, &inputs))
}

// ---------------------------------------------------------------------------
// the translation into Rust

/// Rust's reserved words. A method name that is one is written as a raw
/// identifier; the four that cannot be are refused by the checker.
const RUST_WORDS: &[&str] = &[
    "as", "async", "await", "break", "continue", "dyn", "enum", "extern", "fn", "impl", "in",
    "loop", "match", "mod", "move", "mut", "pub", "ref", "static", "struct", "trait", "type",
    "unsafe", "use", "where", "while", "abstract", "become", "box", "do", "final", "gen", "macro",
    "override", "priv", "try", "typeof", "unsized", "virtual", "yield",
];
const RUST_UNRAW: &[&str] = &["self", "Self", "super", "crate"];

fn ident(name: &str) -> String {
    if RUST_WORDS.contains(&name) {
        format!("r#{name}")
    } else {
        name.to_string()
    }
}

fn lit(v: f64) -> String {
    let t = format!("{v:?}");
    if v < 0.0 {
        format!("({t})")
    } else {
        t
    }
}

fn rstr(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

struct Rust {
    mutable: BTreeSet<String>,
    depth: usize,
}

impl Rust {
    fn expr(&self, e: &Expr) -> String {
        match e {
            Expr::Num { si, .. } => lit(*si),
            Expr::Bool { value, .. } => value.to_string(),
            Expr::Var { name, .. } => match KERNEL_CONSTANTS.iter().find(|c| c.name == name) {
                Some(c) => c.rust.to_string(),
                None => ident(name),
            },
            Expr::Neg(x) => format!("(-{})", self.expr(x)),
            Expr::Not(x) => format!("(!{})", self.expr(x)),
            Expr::Table { si, .. } => format!(
                "&[{}]",
                si.iter().map(|v| lit(*v)).collect::<Vec<_>>().join(", ")
            ),
            Expr::Bin { op, l, r, line } => {
                let (a, b) = (self.expr(l), self.expr(r));
                match op {
                    BinOp::Div => format!("rt::div({a}, {b}, {line})?"),
                    BinOp::Pow => format!("rt::pow({a}, {b})"),
                    BinOp::And => format!("({a} && {b})"),
                    BinOp::Or => format!("({a} || {b})"),
                    _ => format!("({a} {} {b})", op.symbol()),
                }
            }
            Expr::Call { name, args, line } => {
                let a: Vec<String> = args.iter().map(|x| self.expr(x)).collect();
                if name == "interp" {
                    return format!("pmath::interp({}, {}, {})", a[0], a[1], a[2]);
                }
                match implementation(name) {
                    Some(Impl::Plain1(_, p)) => format!("{p}({})", a[0]),
                    Some(Impl::Plain2(_, p)) => format!("{p}({}, {})", a[0], a[1]),
                    Some(Impl::Checked1(_, p)) => format!("{p}({}, {line})?", a[0]),
                    Some(Impl::Checked2(_, p)) => format!("{p}({}, {}, {line})?", a[0], a[1]),
                    None => format!("/* no function {name} */ f64::NAN"),
                }
            }
        }
    }

    fn block(&mut self, body: &[Stmt], o: &mut String) {
        for s in body {
            self.stmt(s, o);
        }
    }

    fn stmt(&mut self, s: &Stmt, o: &mut String) {
        match s {
            Stmt::Let {
                name, expr, line, ..
            }
            | Stmt::Const { name, expr, line } => {
                let m = if self.mutable.contains(name) {
                    "mut "
                } else {
                    ""
                };
                let _ = writeln!(
                    o,
                    "let {m}{}: f64 = rt::fin({}, {line})?;",
                    ident(name),
                    self.expr(expr)
                );
            }
            Stmt::Set { name, expr, line } => {
                let _ = writeln!(
                    o,
                    "{} = rt::fin({}, {line})?;",
                    ident(name),
                    self.expr(expr)
                );
            }
            Stmt::If {
                arms, otherwise, ..
            } => {
                for (i, (c, body)) in arms.iter().enumerate() {
                    let _ = writeln!(
                        o,
                        "{}if {} {{",
                        if i == 0 { "" } else { "} else " },
                        self.expr(c)
                    );
                    self.block(body, o);
                }
                if let Some(body) = otherwise {
                    o.push_str("} else {\n");
                    self.block(body, o);
                }
                o.push_str("}\n");
            }
            Stmt::For {
                var,
                first,
                last,
                body,
                ..
            } => {
                self.depth += 1;
                let i = format!("step_{}", self.depth);
                let _ = writeln!(o, "for {i} in ({first}_i64)..=({last}_i64) {{");
                let _ = writeln!(o, "let {}: f64 = {i} as f64;", ident(var));
                self.block(body, o);
                o.push_str("}\n");
                self.depth -= 1;
            }
            Stmt::Refuse { reason, .. } => {
                let _ = writeln!(o, "return Err(MethodError::Refused({}));", rstr(reason));
            }
            Stmt::Return { expr, line } => {
                let _ = writeln!(o, "return Ok(rt::fin({}, {line})?);", self.expr(expr));
            }
        }
    }
}

fn set_targets(body: &[Stmt], out: &mut BTreeSet<String>) {
    for s in body {
        match s {
            Stmt::Set { name, .. } => {
                out.insert(name.clone());
            }
            Stmt::If {
                arms, otherwise, ..
            } => {
                for (_, b) in arms {
                    set_targets(b, out);
                }
                if let Some(b) = otherwise {
                    set_targets(b, out);
                }
            }
            Stmt::For { body, .. } => set_targets(body, out),
            _ => {}
        }
    }
}

/// The method as a kernel function: `vleo_core::physics::methods::<node>`.
///
/// BY RULE, NOT BY JUDGEMENT. Each construct has exactly one translation, the
/// checked operations are the same `vleo_units::method_rt` functions the
/// interpreter calls, and a value is checked for finiteness at the same places
/// — so the function gives the interpreter's answer to the last bit, which is
/// what the node's generated translation test asserts. `inputs` is the
/// parameter order, the node's declared input order; every value is SI.
pub fn to_rust(p: &Program, node: &str, source: &str, src: &str, inputs: &[String]) -> String {
    let mut mutable = BTreeSet::new();
    set_targets(&p.body, &mut mutable);
    let mut r = Rust { mutable, depth: 0 };
    let mut o = String::new();
    let _ = writeln!(
        o,
        "//! GENERATED from the method of `{node}` by `cargo xtask docs`, translated by\n\
         //! the fixed rules in crates/vleo-sheet/src/method.rs. Do not edit: the method\n\
         //! is changed on the node's form, and this is written again from it.\n"
    );
    o.push_str(
        "#![allow(clippy::all, clippy::float_cmp, clippy::cast_precision_loss, unreachable_code, \
         unused_imports, unused_mut, unused_variables, unused_parens)]\n\n",
    );
    o.push_str("use vleo_units::constants::*;\nuse vleo_units::method_rt::{self as rt, MethodError};\nuse vleo_units::pmath;\n\n");
    let _ = writeln!(
        o,
        "/// The method of `{node}`, source `{source}`:\n///\n/// ```text"
    );
    for l in src.lines() {
        let _ = writeln!(o, "/// {l}");
    }
    o.push_str("/// ```\n");
    let params: Vec<String> = inputs
        .iter()
        .map(|n| format!("{}: f64", ident(n)))
        .collect();
    let _ = writeln!(
        o,
        "pub fn evaluate({}) -> Result<f64, MethodError> {{",
        params.join(", ")
    );
    r.block(&p.body, &mut o);
    o.push_str(
        "Err(MethodError::Degenerate { line: 0, what: \"the method ended without an answer\" })\n}\n",
    );
    o
}

// ---------------------------------------------------------------------------
// the reference page

/// `docs/PSEUDOCODE.md`, from the tables above, so the page and the checker
/// cannot describe two different languages — with the worked example from
/// `crate::example`, the one every node form shows.
pub fn reference_md() -> String {
    let example_node = crate::example::TITLE;
    let example_method = crate::example::METHOD;
    let mut o = String::new();
    o.push_str("<!-- GENERATED from crates/vleo-sheet/src/method.rs by `cargo run -p xtask -- docs`. Do not edit. -->\n");
    let _ = writeln!(o, "# The method language, version {LANGUAGE_VERSION}\n");
    o.push_str(
        "> **Answer first.** Every node's relation is written once more as a *method*: a few lines in a\n\
         > small fixed language that the tool can check, run and translate. The checker refuses a\n\
         > sum of unlike units, a logarithm of a length, an answer of the wrong quantity, and a path\n\
         > that ends without an answer or a refusal — before any code exists.\n\
         >\n\
         > **Kind:** reference · **For:** node authors and developers\n\n",
    );
    o.push_str("## Said simply\n\n");
    o.push_str(
        "A method is the recipe for the node's answer, written so a machine can follow it exactly. It\n\
         reads the node's inputs by their names, may use the constants below, works out named values\n\
         with `let`, and ends every path with `return` (the answer) or `refuse` (the reason it will not\n\
         answer). Every number carries its unit: write `6371 [km]`, and the tool works in SI from there.\n\n",
    );
    o.push_str("## Why a method, when my code already works\n\n");
    o.push_str(
        "Your code produced your test cases. The method is a second, independent statement of the same\n\
         relation, and the Rust the tool ships is generated from the method. The three are compared on\n\
         your cases: if any one of them is wrong, a case disagrees, and it is caught before the change\n\
         reaches anyone.\n\n",
    );
    o.push_str("## Statements\n\n| Form | Meaning | Example |\n|---|---|---|\n");
    for s in STATEMENTS {
        let _ = writeln!(
            o,
            "| `{}` | {} | `{}` |",
            s.form.replace('|', "\\|"),
            s.meaning.replace('|', "\\|"),
            s.example.replace('\n', " ⏎ ").replace('|', "\\|")
        );
    }
    o.push_str(
        "\nOperators: `+ - * / ^`, comparisons `< <= > >= == !=`, and `and`, `or`, `not`. A power of a\n\
         dimensioned value is written as a number (`r ^ 3`, `a ^ 0.5` when every exponent stays whole).\n\
         A bare `0` is zero of any unit; any other number that is not a pure ratio needs its unit.\n\n",
    );
    o.push_str("## Functions\n\n| Function | Units | Meaning |\n|---|---|---|\n");
    for f in FUNCTIONS {
        let rule = match f.rule {
            FnRule::Pure => "a pure number in, a pure number out",
            FnRule::Same => "keeps the unit",
            FnRule::SameTwo => "two like values, that unit out",
            FnRule::Ratio => "two like values, a pure number out",
            FnRule::Sqrt => "halves every exponent (m^2 → m)",
            FnRule::Cbrt => "thirds every exponent",
            FnRule::Interp => "x like the table's x row; the y row's unit out",
            FnRule::Pow => "as ^",
        };
        let _ = writeln!(o, "| `{}` | {} | {} |", f.name, rule, f.meaning);
    }
    o.push_str("\n## Constants every method may use\n\n| Name | Value | Unit | Meaning |\n|---|---|---|---|\n");
    for c in KERNEL_CONSTANTS {
        let _ = writeln!(
            o,
            "| `{}` | {:e} | `{}` | {} |",
            c.name, c.value, c.unit, c.meaning
        );
    }
    o.push_str("\n## Units\n\nWrite a unit in brackets straight after a number: `7.8 [km/s]`, `3.986e14 [m^3/s^2]`,\n`30 [deg]`, `1 [1]` for a pure number. Every symbol a sheet may declare is accepted:\n\n");
    let syms: Vec<String> = Unit::NAMES
        .iter()
        .filter_map(|n| Unit::from_name(n))
        .map(|u| format!("`{}`", u.symbol()))
        .collect();
    o.push_str(&syms.join(" · "));
    o.push_str(
        "\n\nand the simple ones combine with `*` or `.`, `/` and whole-number powers `^`.\n\n",
    );
    let _ = writeln!(o, "## The worked example: {example_node}\n");
    let _ = writeln!(
        o,
        "*{}.* The same example sits behind **Show the example** on every node form.\n",
        crate::example::TAG
    );
    o.push_str("**The method.**\n\n```text\n");
    o.push_str(example_method.trim_end());
    o.push_str("\n```\n\n");
    let f = |k: &str| crate::example::field(k).unwrap_or("");
    let _ = writeln!(
        o,
        "**The author's own code** ({}, `{}`), which produced the cases below.\n\n```text\n{}\n```\n",
        f("author_language"),
        f("author_entry"),
        f("author_code")
    );
    let _ = writeln!(
        o,
        "**The test code** that ran it on each case.\n\n```text\n{}\n```\n",
        f("author_test_code")
    );
    o.push_str("**The cases**, in SI, and what the method gives for each.\n\n| Case | Inputs | The author's code | The method |\n|---|---|---|---|\n");
    if let Ok(r) = report_toml(&crate::example::sheet_text()) {
        for (i, (c, v)) in r.cases.iter().enumerate() {
            let inputs = c
                .inputs
                .iter()
                .map(|(k, x)| format!("{k} = {}", show(*x)))
                .collect::<Vec<_>>()
                .join(", ");
            let got = match r.got.get(i).copied().flatten() {
                Some(g) if !c.refuses() => format!("{} — {}", show(g), v.text(c)),
                _ => v.text(c),
            };
            let _ = writeln!(
                o,
                "| {} | `{inputs}` | {} | {got} |",
                c.label,
                c.expect.map(show).unwrap_or_else(|| "refuses".into())
            );
        }
    }
    o.push('\n');
    o.push_str("## Where the simple version breaks\n\n");
    o.push_str(
        "- **One answer per method.** A node that publishes a set of values (a few rows do) keeps its\n\
           \x20 hand-written hole for now; the method language answers one quantity.\n\
         - **No iteration to convergence.** A loop runs a fixed count. A solver that stops when it\n\
           \x20 converges is marked for a developer, who writes it, and your cases still decide.\n\
         - **Tables are written out.** A lookup into a large data file is a reference-data bundle,\n\
           \x20 not a method.\n",
    );
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig(inputs: &[(&str, &str)], out: &str) -> Signature {
        Signature {
            inputs: inputs
                .iter()
                .map(|(n, q)| (n.to_string(), quantity_dim(q).unwrap()))
                .collect(),
            output: quantity_dim(out).unwrap(),
        }
    }

    const ORBIT: &str = "\
# The speed of a circular orbit at altitude h.
let r = R_EARTH + h
if h < 0 [m] then
  refuse \"the altitude is below the ground\"
end
let v : Velocity = sqrt(MU_EARTH / r)
return v
";

    #[test]
    fn a_sound_method_checks_and_runs() {
        let s = sig(&[("h", "Length")], "Velocity");
        let p = compile(ORBIT, &s).unwrap();
        let o = run(&p, &[("h".into(), 400e3)]).unwrap();
        let Outcome::Answer(v) = o else {
            panic!("{o:?}")
        };
        assert!((v - 7668.6).abs() < 1.0, "{v}");
        let o = run(&p, &[("h".into(), -1.0)]).unwrap();
        assert!(matches!(o, Outcome::Refused { line: 4, .. }), "{o:?}");
    }

    #[test]
    fn units_are_read_and_converted() {
        assert_eq!(parse_unit("km").unwrap().0, 1e3);
        let (f, d) = parse_unit("m^3/s^2").unwrap();
        assert_eq!(f, 1.0);
        assert_eq!(d, Dim::new(3, 0, -2, 0, 0, 0, 0));
        assert_eq!(
            parse_unit("1/m^3").unwrap().1,
            Dim::new(-3, 0, 0, 0, 0, 0, 0)
        );
        assert!(parse_unit("kN").unwrap_err().contains("kN"));
        assert!((parse_unit("deg").unwrap().0 - core::f64::consts::PI / 180.0).abs() < 1e-15);
        assert_eq!(dim_text(Dim::new(3, 0, -2, 0, 0, 0, 0)), "m^3/s^2");
        assert_eq!(dim_text(Dim::new(-3, 0, 0, 0, 0, 0, 0)), "1/m^3");
    }

    fn errors(src: &str, s: &Signature) -> Vec<String> {
        match parse(src) {
            Err(d) => vec![d.to_string()],
            Ok(p) => check(&p, s)
                .into_iter()
                .filter(|d| d.severity == Severity::Error)
                .map(|d| d.to_string())
                .collect(),
        }
    }

    #[test]
    fn every_unit_mistake_is_refused_by_line() {
        let s = sig(&[("h", "Length"), ("t", "Time")], "Velocity");
        let e = errors("let x = h + t\nreturn h / t", &s);
        assert!(
            e[0].starts_with("line 1: a sum of unlike quantities: m and s"),
            "{e:?}"
        );
        let e = errors("return exp(h) * h / t", &s);
        assert!(e[0].contains("exp of a m value"), "{e:?}");
        let e = errors("return h", &s);
        assert!(
            e[0].contains("the answer must be m/s and this is m"),
            "{e:?}"
        );
        let e = errors("return sqrt(h) / t", &s);
        assert!(e[0].contains("sqrt of m is not a quantity"), "{e:?}");
        let e = errors("if h < 5 then\n return h / t\nend\nreturn h / t", &s);
        assert!(e[0].contains("a comparison of unlike quantities"), "{e:?}");
        // A bare zero is zero of anything.
        assert!(errors("if h < 0 then\n refuse \"low\"\nend\nreturn h / t", &s).is_empty());
    }

    #[test]
    fn every_path_must_end() {
        let s = sig(&[("h", "Length"), ("t", "Time")], "Velocity");
        let e = errors("if h > 0 [m] then\n return h / t\nend", &s);
        assert!(
            e.iter().any(|m| m.contains("without return or refuse")),
            "{e:?}"
        );
        let ok = "if h > 0 [m] then\n return h / t\nelse\n refuse \"no\"\nend";
        assert!(errors(ok, &s).is_empty());
        let e = errors("return h / t\nlet x = h", &s);
        assert!(e[0].contains("can never run"), "{e:?}");
    }

    #[test]
    fn names_are_checked() {
        let s = sig(&[("h", "Length")], "Length");
        assert!(errors("return q", &s)[0].contains("«q» is not an input"));
        assert!(errors("let h = 3 [m]\nreturn h", &s)[0].contains("an input of this node"));
        assert!(errors("set h = 3 [m]\nreturn h", &s)[0].contains("cannot be changed"));
        assert!(errors("let R_EARTH = h\nreturn h", &s)[0].contains("kernel constant"));
        assert!(errors("return foo(h)", &s)[0].contains("not a function"));
        let p = parse("return 2 [m]").unwrap();
        let notes = check(&p, &s);
        assert!(notes
            .iter()
            .any(|d| d.severity == Severity::Warning && d.msg.contains("never reads")));
    }

    #[test]
    fn loops_tables_and_powers() {
        let s = sig(&[("x", "Ratio")], "Ratio");
        let series =
            "let total = 0\nfor n = 1 to 20\n  set total = total + x ^ n / n\nend\nreturn total";
        let p = compile(series, &s).unwrap();
        let Outcome::Answer(v) = run(&p, &[("x".into(), 0.5)]).unwrap() else {
            panic!()
        };
        assert!((v - core::f64::consts::LN_2).abs() < 1e-6, "{v}");

        let s = sig(&[("h", "Length")], "MassDensity");
        let table = "return interp(h, [200, 300, 400] [km], [2.5e-10, 1.9e-11, 2.8e-12] [kg/m^3])";
        let p = compile(table, &s).unwrap();
        let Outcome::Answer(v) = run(&p, &[("h".into(), 250e3)]).unwrap() else {
            panic!()
        };
        assert!((v - (2.5e-10 + 1.9e-11) / 2.0).abs() < 1e-18, "{v}");

        let s = sig(&[("a", "Area")], "Length");
        assert!(errors("return a ^ 0.5", &s).is_empty());
        let s = sig(&[("r", "Length")], "Volume");
        assert!(errors("return 4 / 3 * PI * r ^ 3", &s).is_empty());
        assert!(errors("return r ^ 1.5", &s)[0].contains("not a quantity"));
    }

    #[test]
    fn runtime_faults_are_errors_not_refusals() {
        let s = sig(&[("x", "Ratio")], "Ratio");
        let p = compile("return sqrt(x)", &s).unwrap();
        let e = run(&p, &[("x".into(), -1.0)]).unwrap_err();
        assert!(e.msg.contains("no square root"), "{e}");
        let p = compile("return 1 / x", &s).unwrap();
        assert!(run(&p, &[("x".into(), 0.0)])
            .unwrap_err()
            .msg
            .contains("division by zero"));
    }

    #[test]
    fn the_grammar_says_where_it_went_wrong() {
        let s = sig(&[("x", "Ratio")], "Ratio");
        assert!(errors("let = 3", &s)[0].starts_with("line 1"));
        assert!(errors("if x > 0 then\n return x", &s)[0].contains("needs its «end»"));
        assert!(errors("return x x", &s)[0].contains("one statement per line"));
        assert!(errors("return x $ 2", &s)[0].contains("not part of the method language"));
        assert!(errors("print x", &s)[0].contains("not «print»"));
        assert!(errors("const c = x\nreturn x", &s)[0].contains("one number with its unit"));
        // A table may run across lines inside its brackets.
        let s = sig(&[("h", "Length")], "Ratio");
        let multi = "return interp(h, [1, 2,\n 3] [m], [0, 1,\n 2] [1])";
        assert!(errors(multi, &s).is_empty(), "{:?}", errors(multi, &s));
    }

    #[test]
    fn cases_are_judged_against_the_method() {
        let sheet = format!(
            "[method]\ntext = \"\"\"\n{ORBIT}\"\"\"\n[output]\ntype = \"Velocity\"\n\n[[input]]\nbinding = \"h\"\ntype = \"Length\"\n\n\
             [[case]]\nlabel = \"250 km\"\nexpect = 7754.84549737\ntolerance = 1e-6\ninputs = {{ h = 250000.0 }}\n\n\
             [[case]]\nlabel = \"400 km\"\nexpect = 7668.55817541\ntolerance = 1e-6\ninputs = {{ h = 400000.0 }}\n\n\
             [[case]]\nlabel = \"wrong\"\nexpect = 7000.0\ntolerance = 1e-6\ninputs = {{ h = 400000.0 }}\n\n\
             [[case]]\nlabel = \"underground\"\nrefuse = \"yes\"\ninputs = {{ h = -5.0 }}\n"
        );
        let r = report_toml(&sheet).unwrap();
        assert!(r.shortfall.is_empty(), "{:?}", r.shortfall);
        let v: Vec<bool> = r.cases.iter().map(|(_, v)| v.agrees()).collect();
        assert_eq!(v, [true, true, false, true]);
        assert!(!r.sound());
        assert!(r.cases[2]
            .1
            .text(&r.cases[2].0)
            .contains("more than your tolerance"));
        let j = r.json();
        assert!(j.starts_with("{\"sound\": false"), "{j}");
        // Too few, and no refusal: said, not guessed.
        let r = report("return 1 [m/s]", &sig(&[], "Velocity"), &[]);
        assert_eq!(r.shortfall.len(), 2);
    }

    #[test]
    fn every_function_has_one_implementation_for_both_runner_and_translator() {
        for f in FUNCTIONS {
            if f.name == "interp" {
                continue;
            }
            assert!(
                implementation(f.name).is_some(),
                "{} has no implementation",
                f.name
            );
        }
    }

    #[test]
    fn the_plain_form_reads_as_the_sheet_does() {
        let plain = format!(
            "output Velocity\ninput r Length\ncase 0 7754.84549737 1e-6 r=6628137 250 km\n\
             case 0 7000 1e-6 r=6778137 wrong\ncase 1 - - r=6000000 inside\nmethod\n{}",
            crate::example::METHOD
        );
        let r = report_plain(&plain).unwrap();
        let v: Vec<bool> = r.cases.iter().map(|(_, v)| v.agrees()).collect();
        assert_eq!(v, [true, false, true]);
        assert_eq!(r.cases[0].0.label, "250 km");
        assert!(report_plain("output Nonsense\nmethod\nreturn 1").is_err());
    }

    #[test]
    fn the_reference_covers_every_function_and_constant() {
        let md = reference_md();
        for f in FUNCTIONS {
            assert!(md.contains(&format!("`{}`", f.name)));
        }
        for c in KERNEL_CONSTANTS {
            assert!(md.contains(&format!("`{}`", c.name)));
            assert!(
                parse_unit(c.unit).is_ok(),
                "{} has an unreadable unit",
                c.name
            );
        }
    }
}
