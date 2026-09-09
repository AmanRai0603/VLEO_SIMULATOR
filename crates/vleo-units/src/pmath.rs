//! Portable maths — the one implementation of every transcendental function
//! in the system.
//!
//! # Why this file exists
//!
//! Addition, subtraction, multiplication, division and square root are exactly
//! specified by IEEE-754 and agree on every target. `sin`, `cos`, `exp`, `ln`
//! and `powf` are **not** specified: a native build calls the operating
//! system's maths library, a WebAssembly build calls the one compiled into the
//! module, and the two differ in the last bit. Any node with a trigonometric or
//! exponential term — which is most of the orbital ones — would then fail the
//! nightly *golden vectors agree bit for bit* gate on the first night, for a
//! reason that is not a defect. Within a fortnight the team learns to ignore a
//! red nightly build, which is worse than having no gate at all.
//!
//! So the kernel crates are forbidden from calling the standard library's
//! transcendental functions (`xtask gate` step `portable-maths` enforces it by
//! lint) and route everything through this module. Every function below uses
//! only `+ - * /`, comparisons and bit manipulation, all of which are exactly
//! specified — so the same input produces the same bits on a laptop, in the
//! browser, on the bench and on a flight target.
//!
//! # What is promised, and what is not
//!
//! *Promised*: determinism. The identical bit pattern on every target.
//! *Not promised*: correct rounding to 0.5 ulp. These are accurate to roughly
//! 1–2 ulp, which is far inside every tolerance any node in this system
//! declares. Accuracy is verified against reference values in `tests/`.
#![allow(clippy::excessive_precision)]

// Re-exported from `core` rather than re-typed. These are exactly specified
// double values, identical on every target, and a second copy of them is a
// second thing that can be wrong by a digit.
pub use core::f64::consts::{FRAC_PI_2, FRAC_PI_4, LN_10, LN_2, LOG2_E, PI, SQRT_2, TAU};

const SIGN_MASK: u64 = 0x8000_0000_0000_0000;
const EXP_MASK: u64 = 0x7FF0_0000_0000_0000;

// Three-part split of pi/2, so that `k * PIO2` is representable exactly enough
// for Cody-Waite argument reduction to keep ~60 significant bits. The high part
// *is* the nearest double to pi/2 by construction — that is what makes the
// remaining two parts the residual — so it is taken from `core` rather than
// re-typed.
const PIO2_HI: f64 = FRAC_PI_2;
const PIO2_MID: f64 = 6.123_233_995_736_765_886_130_33e-17;
const PIO2_LO: f64 = 3.749_399_456_246_780_698_378_31e-33;

/// `|x|`, by clearing the sign bit. Exact.
#[inline]
pub fn abs(x: f64) -> f64 {
    f64::from_bits(x.to_bits() & !SIGN_MASK)
}

/// Sign-carrying copy: magnitude of `x`, sign of `y`. Exact.
#[inline]
pub fn copysign(x: f64, y: f64) -> f64 {
    f64::from_bits((x.to_bits() & !SIGN_MASK) | (y.to_bits() & SIGN_MASK))
}

#[inline]
pub fn is_nan(x: f64) -> bool {
    f64::is_nan(x)
}

#[inline]
pub fn is_finite(x: f64) -> bool {
    (x.to_bits() & EXP_MASK) != EXP_MASK
}

#[inline]
pub fn min(a: f64, b: f64) -> f64 {
    if a < b {
        a
    } else {
        b
    }
}

#[inline]
pub fn max(a: f64, b: f64) -> f64 {
    if a > b {
        a
    } else {
        b
    }
}

/// Round toward zero. Exact.
pub fn trunc(x: f64) -> f64 {
    let bits = x.to_bits();
    let exp = ((bits >> 52) & 0x7FF) as i32 - 1023;
    if exp < 0 {
        return copysign(0.0, x);
    }
    if exp >= 52 {
        return x;
    }
    let mask = (1u64 << (52 - exp)) - 1;
    f64::from_bits(bits & !mask)
}

/// Round toward negative infinity. Exact.
pub fn floor(x: f64) -> f64 {
    let t = trunc(x);
    if x < 0.0 && t != x {
        t - 1.0
    } else {
        t
    }
}

/// Round toward positive infinity. Exact.
pub fn ceil(x: f64) -> f64 {
    let t = trunc(x);
    if x > 0.0 && t != x {
        t + 1.0
    } else {
        t
    }
}

/// Round half away from zero. Exact.
pub fn round(x: f64) -> f64 {
    let t = trunc(x);
    let f = x - t;
    if abs(f) >= 0.5 {
        t + copysign(1.0, x)
    } else {
        t
    }
}

/// Round half to even — the tie rule used by argument reduction.
pub fn round_ties_even(x: f64) -> f64 {
    let r = round(x);
    if abs(x - trunc(x)) == 0.5 {
        let half = r * 0.5;
        if half != trunc(half) {
            return r - copysign(1.0, x);
        }
    }
    r
}

/// Floating remainder `x - n*y`, `n` truncated. Uses only exact operations
/// while the quotient stays representable.
pub fn fmod(x: f64, y: f64) -> f64 {
    if y == 0.0 || !is_finite(x) {
        return f64::NAN;
    }
    let r = x - trunc(x / y) * y;
    // One correction pass: the division above may round the quotient.
    if abs(r) >= abs(y) {
        r - trunc(r / y) * y
    } else {
        r
    }
}

/// Square root by Newton–Raphson from a bit-shift seed.
///
/// Not the IEEE `sqrt` instruction — that is exactly rounded and therefore
/// portable, but reaching it from `core` requires the platform library this
/// module exists to avoid. Five iterations from a seed good to ~5% converge to
/// well under 1 ulp, using only multiply, divide and add.
pub fn sqrt(x: f64) -> f64 {
    if is_nan(x) || x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 || !is_finite(x) {
        return x;
    }
    // Seed: halve the exponent in the bit pattern.
    let bits = x.to_bits();
    let mut r = f64::from_bits((bits >> 1) + (0x1FF8_0000_0000_0000));
    // Newton–Raphson on r^2 = x.
    r = 0.5 * (r + x / r);
    r = 0.5 * (r + x / r);
    r = 0.5 * (r + x / r);
    r = 0.5 * (r + x / r);
    r = 0.5 * (r + x / r);
    r
}

/// Cube root, by Newton–Raphson on `r^3 = x`. Sign-preserving.
pub fn cbrt(x: f64) -> f64 {
    if x == 0.0 || !is_finite(x) || is_nan(x) {
        return x;
    }
    let s = copysign(1.0, x);
    let a = abs(x);
    let bits = a.to_bits();
    let mut r = f64::from_bits(bits / 3 + 0x2A9F_8000_0000_0000);
    for _ in 0..6 {
        r = r - (r - a / (r * r)) / 3.0;
    }
    s * r
}

/// `sqrt(x^2 + y^2)` without intermediate overflow.
pub fn hypot(x: f64, y: f64) -> f64 {
    let (a, b) = (abs(x), abs(y));
    let (hi, lo) = if a > b { (a, b) } else { (b, a) };
    if hi == 0.0 {
        return 0.0;
    }
    let t = lo / hi;
    hi * sqrt(1.0 + t * t)
}

/// `2^n` by exponent construction. Exact for `-1022 <= n <= 1023`.
fn exp2i(n: i32) -> f64 {
    if n > 1023 {
        return f64::INFINITY;
    }
    if n < -1074 {
        return 0.0;
    }
    if n < -1022 {
        // Subnormal: build 2^-1022 then scale down by exact halving.
        let mut v = f64::from_bits(1u64 << 52);
        for _ in 0..(-1022 - n) {
            v *= 0.5;
        }
        return v;
    }
    f64::from_bits(((n + 1023) as u64) << 52)
}

/// `e^x`. Cody-Waite reduction `x = k·ln2 + r`, then a Taylor series in `r`
/// truncated at `r^13/13!` — the next term is below 2e-16 relative on
/// `|r| <= ln2/2`.
pub fn exp(x: f64) -> f64 {
    if is_nan(x) {
        return x;
    }
    if x > 709.782_712_893_384 {
        return f64::INFINITY;
    }
    if x < -745.133_219_101_941_2 {
        return 0.0;
    }
    let k = round_ties_even(x * LOG2_E);
    // ln2 split so that k*LN2_HI is exact.
    const LN2_HI: f64 = 6.931_471_803_691_238_164_901_733_398_44e-1;
    const LN2_LO: f64 = 1.908_214_929_270_587_700_284_836_984_47e-10;
    let r = (x - k * LN2_HI) - k * LN2_LO;
    let mut s = 1.0 / 6_227_020_800.0; // 1/13!
    s = s * r + 1.0 / 479_001_600.0; // 1/12!
    s = s * r + 1.0 / 39_916_800.0; // 1/11!
    s = s * r + 1.0 / 3_628_800.0; // 1/10!
    s = s * r + 1.0 / 362_880.0; // 1/9!
    s = s * r + 1.0 / 40_320.0; // 1/8!
    s = s * r + 1.0 / 5_040.0; // 1/7!
    s = s * r + 1.0 / 720.0; // 1/6!
    s = s * r + 1.0 / 120.0; // 1/5!
    s = s * r + 1.0 / 24.0; // 1/4!
    s = s * r + 1.0 / 6.0; // 1/3!
    s = s * r + 0.5; // 1/2!
    s = s * r + 1.0;
    s = s * r + 1.0;
    s * exp2i(k as i32)
}

/// `e^x - 1`, accurate for small `x` where `exp(x) - 1` cancels.
pub fn expm1(x: f64) -> f64 {
    if abs(x) > 0.25 {
        return exp(x) - 1.0;
    }
    let mut s = 1.0 / 3_628_800.0;
    s = s * x + 1.0 / 362_880.0;
    s = s * x + 1.0 / 40_320.0;
    s = s * x + 1.0 / 5_040.0;
    s = s * x + 1.0 / 720.0;
    s = s * x + 1.0 / 120.0;
    s = s * x + 1.0 / 24.0;
    s = s * x + 1.0 / 6.0;
    s = s * x + 0.5;
    s = s * x + 1.0;
    s * x
}

/// Natural logarithm. `x = m·2^e` with `m ∈ [√½, √2)`, then the atanh series
/// `ln m = 2·(s + s³/3 + … + s²¹/21)` with `s = (m-1)/(m+1)`, `|s| ≤ 0.172`.
pub fn ln(x: f64) -> f64 {
    if is_nan(x) || x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 {
        return f64::NEG_INFINITY;
    }
    if !is_finite(x) {
        return x;
    }
    let mut bits = x.to_bits();
    let mut e = (((bits >> 52) & 0x7FF) as i32) - 1023;
    if e == -1023 {
        // Subnormal: scale into the normal range by an exact power of two.
        let scaled = x * exp2i(64);
        bits = scaled.to_bits();
        e = (((bits >> 52) & 0x7FF) as i32) - 1023 - 64;
    }
    let mut m = f64::from_bits((bits & 0x000F_FFFF_FFFF_FFFF) | 0x3FF0_0000_0000_0000);
    if m > SQRT_2 {
        m *= 0.5;
        e += 1;
    }
    let s = (m - 1.0) / (m + 1.0);
    let s2 = s * s;
    let mut p = 1.0 / 21.0;
    p = p * s2 + 1.0 / 19.0;
    p = p * s2 + 1.0 / 17.0;
    p = p * s2 + 1.0 / 15.0;
    p = p * s2 + 1.0 / 13.0;
    p = p * s2 + 1.0 / 11.0;
    p = p * s2 + 1.0 / 9.0;
    p = p * s2 + 1.0 / 7.0;
    p = p * s2 + 1.0 / 5.0;
    p = p * s2 + 1.0 / 3.0;
    p = p * s2 + 1.0;
    2.0 * s * p + (e as f64) * LN_2
}

/// `ln(1 + x)`, accurate for small `x`.
pub fn ln_1p(x: f64) -> f64 {
    if abs(x) > 0.25 {
        return ln(1.0 + x);
    }
    let s = x / (2.0 + x);
    let s2 = s * s;
    let mut p = 1.0 / 15.0;
    p = p * s2 + 1.0 / 13.0;
    p = p * s2 + 1.0 / 11.0;
    p = p * s2 + 1.0 / 9.0;
    p = p * s2 + 1.0 / 7.0;
    p = p * s2 + 1.0 / 5.0;
    p = p * s2 + 1.0 / 3.0;
    p = p * s2 + 1.0;
    2.0 * s * p
}

pub fn log10(x: f64) -> f64 {
    ln(x) / LN_10
}

pub fn log2(x: f64) -> f64 {
    ln(x) * LOG2_E
}

/// Reduce `x` into `[-pi/4, pi/4]` and report the quadrant `k mod 4`.
fn reduce_pio2(x: f64) -> (f64, i64) {
    // Coarse reduction first, so Cody-Waite stays accurate for large angles.
    let x = if abs(x) > 1.0e8 { fmod(x, TAU) } else { x };
    let kf = round_ties_even(x * (2.0 / PI));
    let r = ((x - kf * PIO2_HI) - kf * PIO2_MID) - kf * PIO2_LO;
    (r, kf as i64)
}

/// `sin r` on `|r| <= pi/4`. Taylor in `z = r^2` to `r^17`; the first dropped
/// term is below 5e-17 relative at the interval edge.
fn sin_kernel(r: f64) -> f64 {
    let z = r * r;
    let mut p = 1.0 / 355_687_428_096_000.0; // z^8  ->  r^17 / 17!
    p = p * z - 1.0 / 1_307_674_368_000.0; //  z^7  -> -r^15 / 15!
    p = p * z + 1.0 / 6_227_020_800.0; //      z^6  ->  r^13 / 13!
    p = p * z - 1.0 / 39_916_800.0; //         z^5  -> -r^11 / 11!
    p = p * z + 1.0 / 362_880.0; //            z^4  ->  r^9  / 9!
    p = p * z - 1.0 / 5_040.0; //              z^3  -> -r^7  / 7!
    p = p * z + 1.0 / 120.0; //                z^2  ->  r^5  / 5!
    p = p * z - 1.0 / 6.0; //                  z^1  -> -r^3  / 3!
    p = p * z + 1.0; //                        z^0  ->  r
    r * p
}

/// `cos r` on `|r| <= pi/4`. Taylor in `z = r^2` to `r^18`.
fn cos_kernel(r: f64) -> f64 {
    let z = r * r;
    let mut p = -1.0 / 6_402_373_705_728_000.0; // z^9  -> -r^18 / 18!
    p = p * z + 1.0 / 20_922_789_888_000.0; //     z^8  ->  r^16 / 16!
    p = p * z - 1.0 / 87_178_291_200.0; //         z^7  -> -r^14 / 14!
    p = p * z + 1.0 / 479_001_600.0; //            z^6  ->  r^12 / 12!
    p = p * z - 1.0 / 3_628_800.0; //              z^5  -> -r^10 / 10!
    p = p * z + 1.0 / 40_320.0; //                 z^4  ->  r^8  / 8!
    p = p * z - 1.0 / 720.0; //                    z^3  -> -r^6  / 6!
    p = p * z + 1.0 / 24.0; //                     z^2  ->  r^4  / 4!
    p = p * z - 0.5; //                            z^1  -> -r^2  / 2!
    p = p * z + 1.0; //                            z^0  ->  1
    p
}

pub fn sin(x: f64) -> f64 {
    if !is_finite(x) {
        return f64::NAN;
    }
    let (r, k) = reduce_pio2(x);
    match k.rem_euclid(4) {
        0 => sin_kernel(r),
        1 => cos_kernel(r),
        2 => -sin_kernel(r),
        _ => -cos_kernel(r),
    }
}

pub fn cos(x: f64) -> f64 {
    if !is_finite(x) {
        return f64::NAN;
    }
    let (r, k) = reduce_pio2(x);
    match k.rem_euclid(4) {
        0 => cos_kernel(r),
        1 => -sin_kernel(r),
        2 => -cos_kernel(r),
        _ => sin_kernel(r),
    }
}

/// Both at once — the orbital nodes almost always want the pair, and one
/// reduction is both cheaper and guaranteed consistent.
pub fn sin_cos(x: f64) -> (f64, f64) {
    if !is_finite(x) {
        return (f64::NAN, f64::NAN);
    }
    let (r, k) = reduce_pio2(x);
    let (s, c) = (sin_kernel(r), cos_kernel(r));
    match k.rem_euclid(4) {
        0 => (s, c),
        1 => (c, -s),
        2 => (-s, -c),
        _ => (-c, s),
    }
}

pub fn tan(x: f64) -> f64 {
    let (s, c) = sin_cos(x);
    s / c
}

/// `atan t` on `|t| <= tan(pi/8) = 0.4142`, by the alternating odd series.
/// Twenty-four terms put the first dropped term below 3e-18 relative.
fn atan_core(t: f64) -> f64 {
    let z = t * t;
    let n: i32 = 23;
    let coeff = |k: i32| -> f64 {
        let c = 1.0 / ((2 * k + 1) as f64);
        if k % 2 == 0 {
            c
        } else {
            -c
        }
    };
    let mut p = coeff(n);
    let mut k = n - 1;
    loop {
        p = p * z + coeff(k);
        if k == 0 {
            break;
        }
        k -= 1;
    }
    t * p
}

/// `atan a` for `a` in `[0, 1]`, folding the upper half onto the lower with
/// `atan a = pi/4 + atan((a-1)/(a+1))`.
fn atan_unit(a: f64) -> f64 {
    if a > 0.414_213_562_373_095_1 {
        FRAC_PI_4 + atan_core((a - 1.0) / (a + 1.0))
    } else {
        atan_core(a)
    }
}

/// `atan x` over the whole real line. `|x| > 1` folds with
/// `atan x = pi/2 - atan(1/x)`, so every argument reaches `atan_core` inside
/// its interval of validity.
pub fn atan(x: f64) -> f64 {
    if is_nan(x) {
        return x;
    }
    let s = copysign(1.0, x);
    let a = abs(x);
    if !is_finite(a) {
        return s * FRAC_PI_2;
    }
    let r = if a > 1.0 {
        FRAC_PI_2 - atan_unit(1.0 / a)
    } else {
        atan_unit(a)
    };
    s * r
}

/// Four-quadrant arctangent. `y` is the ordinate, `x` the abscissa.
pub fn atan2(y: f64, x: f64) -> f64 {
    if is_nan(x) || is_nan(y) {
        return f64::NAN;
    }
    if x == 0.0 && y == 0.0 {
        return 0.0;
    }
    if x > 0.0 {
        atan(y / x)
    } else if x < 0.0 {
        if y >= 0.0 {
            atan(y / x) + PI
        } else {
            atan(y / x) - PI
        }
    } else {
        copysign(FRAC_PI_2, y)
    }
}

pub fn asin(x: f64) -> f64 {
    if !(-1.0..=1.0).contains(&x) {
        return f64::NAN;
    }
    if x == 1.0 {
        return FRAC_PI_2;
    }
    if x == -1.0 {
        return -FRAC_PI_2;
    }
    atan2(x, sqrt((1.0 - x) * (1.0 + x)))
}

pub fn acos(x: f64) -> f64 {
    if !(-1.0..=1.0).contains(&x) {
        return f64::NAN;
    }
    atan2(sqrt((1.0 - x) * (1.0 + x)), x)
}

pub fn sinh(x: f64) -> f64 {
    let e = expm1(abs(x));
    let r = 0.5 * (e + e / (e + 1.0));
    copysign(r, x)
}

pub fn cosh(x: f64) -> f64 {
    let e = exp(abs(x));
    0.5 * (e + 1.0 / e)
}

pub fn tanh(x: f64) -> f64 {
    if abs(x) > 20.0 {
        return copysign(1.0, x);
    }
    let e = expm1(2.0 * abs(x));
    copysign(e / (e + 2.0), x)
}

/// `x^y`. Small integer exponents go through exact repeated squaring, which is
/// both more accurate and faster than `exp(y·ln x)`.
pub fn powf(x: f64, y: f64) -> f64 {
    if y == 0.0 {
        return 1.0;
    }
    if y == 1.0 {
        return x;
    }
    if y == 2.0 {
        return x * x;
    }
    if y == 0.5 {
        return sqrt(x);
    }
    if y == -1.0 {
        return 1.0 / x;
    }
    if x == 0.0 {
        return if y > 0.0 { 0.0 } else { f64::INFINITY };
    }
    if y == trunc(y) && abs(y) <= 64.0 {
        return powi(x, y as i32);
    }
    if x < 0.0 {
        // A negative base with a non-integer exponent has no real value. The
        // caller is asking something the model does not mean; NaN propagates
        // into the fault the node raises rather than a plausible number.
        return f64::NAN;
    }
    exp(y * ln(x))
}

/// Integer power by repeated squaring. Exact until the mantissa runs out.
pub fn powi(x: f64, n: i32) -> f64 {
    let neg = n < 0;
    let mut n = if neg { -(n as i64) } else { n as i64 };
    let mut base = x;
    let mut acc = 1.0;
    while n > 0 {
        if n & 1 == 1 {
            acc *= base;
        }
        base *= base;
        n >>= 1;
    }
    if neg {
        1.0 / acc
    } else {
        acc
    }
}

/// Wrap an angle in radians into `[0, 2pi)`.
pub fn wrap_2pi(a: f64) -> f64 {
    let r = fmod(a, TAU);
    if r < 0.0 {
        r + TAU
    } else {
        r
    }
}

/// Wrap an angle in radians into `(-pi, pi]`.
pub fn wrap_pi(a: f64) -> f64 {
    let r = wrap_2pi(a);
    if r > PI {
        r - TAU
    } else {
        r
    }
}

/// Linear interpolation, clamped at both ends of the table.
pub fn interp(x: f64, xs: &[f64], ys: &[f64]) -> f64 {
    if xs.is_empty() {
        return f64::NAN;
    }
    if x <= xs[0] {
        return ys[0];
    }
    let n = xs.len();
    if x >= xs[n - 1] {
        return ys[n - 1];
    }
    let mut i = 0usize;
    while i + 1 < n && xs[i + 1] < x {
        i += 1;
    }
    let t = (x - xs[i]) / (xs[i + 1] - xs[i]);
    ys[i] + t * (ys[i + 1] - ys[i])
}

/// The error function.
///
/// Needed by the Sentman gas-surface interaction model, which is the single
/// most-used relation in the aerodynamics chain. Maclaurin series for
/// `|x| <= 3`, where it converges quickly and unconditionally; the asymptotic
/// tail beyond that, where `erf` is within 3e-8 of one anyway.
pub fn erf(x: f64) -> f64 {
    use core::f64::consts::FRAC_2_SQRT_PI as TWO_OVER_SQRT_PI;
    let a = abs(x);
    if a > 6.0 {
        return copysign(1.0, x);
    }
    if a <= 3.0 {
        // erf(x) = 2/sqrt(pi) * sum_{n>=0} (-1)^n x^(2n+1) / (n! (2n+1))
        let z = x * x;
        let mut term = x; // n = 0 term, x^(2n+1)/n!
        let mut sum = x;
        let mut n = 1.0f64;
        while n < 80.0 {
            term *= -z / n;
            let add = term / (2.0 * n + 1.0);
            sum += add;
            if abs(add) < 1.0e-18 * abs(sum) {
                break;
            }
            n += 1.0;
        }
        return TWO_OVER_SQRT_PI * sum;
    }
    // Asymptotic complementary error function for the tail.
    let z = a * a;
    let mut s = 1.0;
    let mut t = 1.0;
    let mut k = 1.0f64;
    while k < 12.0 {
        t *= -(2.0 * k - 1.0) / (2.0 * z);
        s += t;
        k += 1.0;
    }
    let erfc = exp(-z) / (a * 1.772_453_850_905_516_027_298_167_483_341_1) * s;
    copysign(1.0 - erfc, x)
}

/// The complementary error function, `1 - erf(x)`.
pub fn erfc(x: f64) -> f64 {
    1.0 - erf(x)
}
