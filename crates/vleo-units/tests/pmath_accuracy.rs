//! Accuracy of the portable maths library against reference values.
//!
//! These are not determinism tests — determinism follows from the
//! implementation using only exactly-specified operations, and is checked
//! across targets by the golden-vector gate. These check that the portable
//! implementation is *right*, against values computed independently at higher
//! precision. Reference values come from an independent implementation — the C standard
//! library reached through CPython's `math` module — which is exactly the
//! provenance rule every fixture in this system obeys: an expected value may
//! never be produced by the code under test.
// These literals are reference values from an independent implementation, not
// approximations standing in for a named constant, so the lints that would
// rewrite them into `core::f64::consts` are switched off here: substituting the
// constant would make the test compare the implementation against itself.
#![allow(clippy::approx_constant, clippy::excessive_precision)]

use vleo_units::pmath::*;

/// Relative error, or absolute error when the reference is zero.
fn rel(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        (a - b).abs()
    } else {
        ((a - b) / b).abs()
    }
}

const TOL: f64 = 4.0e-15;

#[test]
fn sqrt_matches_reference() {
    for &(x, r) in &[
        (2.0f64, 1.414_213_562_373_095_1f64),
        (1.0e-13, 3.162_277_660_168_379_4e-7),
        (3.986_004_418e14, 19_964_980.385_665_298),
        (1.0e300, 1.0e150),
        (0.25, 0.5),
    ] {
        assert!(rel(sqrt(x), r) < TOL, "sqrt({x}) = {} want {r}", sqrt(x));
    }
    assert_eq!(sqrt(0.0), 0.0);
    assert!(sqrt(-1.0).is_nan());
}

#[test]
fn exp_and_ln_match_reference() {
    for &(x, r) in &[
        (1.0f64, 2.718_281_828_459_045_1f64),
        (-30.0, 9.357_622_968_840_175e-14),
        (0.5, 1.648_721_270_700_128_2),
        (700.0, 1.014_232_054_735_004_4e304),
        (-0.001, 0.999_000_499_833_374_9),
    ] {
        assert!(rel(exp(x), r) < TOL, "exp({x}) = {} want {r}", exp(x));
    }
    for &(x, r) in &[
        (2.0f64, 0.693_147_180_559_945_29f64),
        (1.0e-13, -29.933_606_208_922_594),
        (1361.0, 7.215_975_002_651_466),
        (0.5, -0.693_147_180_559_945_29),
    ] {
        assert!(rel(ln(x), r) < TOL, "ln({x}) = {} want {r}", ln(x));
    }
}

#[test]
fn exp_ln_round_trip() {
    let mut x = -300.0;
    while x < 300.0 {
        assert!(rel(ln(exp(x)), x) < 1.0e-13 || x.abs() < 1.0e-9);
        x += 7.3;
    }
}

#[test]
fn trig_matches_reference() {
    for &(x, s, c) in &[
        (0.0f64, 0.0f64, 1.0f64),
        (0.5, 0.479_425_538_604_203_0, 0.877_582_561_890_372_7),
        (1.570_796_326_794_896_6, 1.0, 6.123_233_995_736_766e-17),
        (3.0, 0.141_120_008_059_867_2, -0.989_992_496_600_445_5),
        (-2.5, -0.598_472_144_103_957_3, -0.801_143_615_546_933_7),
        (100.0, -0.506_365_641_109_758_8, 0.862_318_872_287_684_3),
    ] {
        assert!(rel(sin(x), s) < 1.0e-14, "sin({x}) = {} want {s}", sin(x));
        assert!(rel(cos(x), c) < 1.0e-14, "cos({x}) = {} want {c}", cos(x));
        let (ss, cc) = sin_cos(x);
        assert_eq!(ss, sin(x));
        assert_eq!(cc, cos(x));
    }
}

#[test]
fn pythagorean_identity_holds_everywhere() {
    let mut x = -50.0;
    while x < 50.0 {
        let (s, c) = sin_cos(x);
        assert!((s * s + c * c - 1.0).abs() < 4.0e-16, "at {x}");
        x += 0.017;
    }
}

#[test]
fn atan_and_atan2_match_reference() {
    for &(x, r) in &[
        (1.0f64, 0.785_398_163_397_448_31f64),
        (0.1, 0.099_668_652_491_162_04),
        (10.0, 1.471_127_674_303_734_6),
        (-3.5, -1.292_496_667_789_785_3),
        (1.0e8, 1.570_796_316_794_896_6),
    ] {
        assert!(rel(atan(x), r) < TOL, "atan({x}) = {} want {r}", atan(x));
    }
    assert!(rel(atan2(1.0, -1.0), 2.356_194_490_192_344_9) < TOL);
    assert!(rel(atan2(-1.0, -1.0), -2.356_194_490_192_344_9) < TOL);
    assert!(rel(atan2(1.0, 0.0), FRAC_PI_2) < TOL);
}

#[test]
fn asin_acos_are_inverses() {
    // Absolute, not relative. `acos` near zero returns an angle near pi/2, and
    // taking its cosine back cancels almost every significant digit — the round
    // trip is ill-conditioned there whatever implementation is underneath, so a
    // relative tolerance would be measuring the conditioning rather than the
    // function.
    let mut x = -0.999;
    while x < 0.999 {
        assert!((sin(asin(x)) - x).abs() < 1.0e-15, "asin at {x}");
        assert!((cos(acos(x)) - x).abs() < 1.0e-15, "acos at {x}");
        x += 0.031;
    }
    assert!(rel(asin(0.5), 0.523_598_775_598_298_9) < TOL);
    assert!(rel(acos(0.5), 1.047_197_551_196_597_7) < TOL);
}

#[test]
fn powf_matches_reference() {
    assert!(rel(powf(2.0, 10.0), 1024.0) < TOL);
    assert!(rel(powf(1.5, 2.5), 2.755_675_960_631_075_2) < TOL);
    assert!(rel(powf(10.0, -13.0), 1.0e-13) < 1.0e-14);
    // The exponent the free-molecular drag and CER nodes both use.
    assert!(rel(powf(2.7, 0.75), 2.106_312_587_388_644_4) < TOL);
    assert!(powf(-2.0, 0.5).is_nan());
    assert_eq!(powf(0.0, 2.0), 0.0);
}

#[test]
fn rounding_and_wrapping_are_exact() {
    assert_eq!(floor(-2.5), -3.0);
    assert_eq!(ceil(-2.5), -2.0);
    assert_eq!(trunc(-2.5), -2.0);
    assert_eq!(round(-2.5), -3.0);
    assert_eq!(abs(-0.0), 0.0);
    assert!((wrap_2pi(-0.5) - (TAU - 0.5)).abs() < 1.0e-15);
    assert!((wrap_pi(TAU + 0.5) - 0.5).abs() < 1.0e-15);
}

#[test]
fn interp_clamps_at_both_ends() {
    let xs = [0.0, 1.0, 2.0];
    let ys = [10.0, 20.0, 40.0];
    assert_eq!(interp(-1.0, &xs, &ys), 10.0);
    assert_eq!(interp(3.0, &xs, &ys), 40.0);
    assert_eq!(interp(0.5, &xs, &ys), 15.0);
    assert_eq!(interp(1.5, &xs, &ys), 30.0);
}
