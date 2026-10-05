// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_f107_design_short`.
//!
//! Every expected value below names a source outside this code. A number
//! produced by the thing being tested proves nothing, so the schema
//! refuses a fixture whose provenance is the implementation.

#![allow(clippy::approx_constant, clippy::excessive_precision)]

use super::model;
use vleo_core::units::*;

fn relative_error(got: f64, expected: f64) -> f64 {
    if expected == 0.0 { pmath::abs(got) } else { pmath::abs((got - expected) / expected) }
}

/// the sustained level from sw_f107_design_long's first fixture plus the daily departure this row USED to be handed, before sw_daily_band_spread was conditioned on level. Kept as an arithmetic case: the pair no longer occurs in this tree, and what it still checks is that the relation is that addition
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(177.22160896), Ratio::new(34.2314814815)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 211.4530904415);
    assert!(err <= 1e-12, "the sustained level from sw_f107_design_long's first fixture plus the daily departure this row USED to be handed, before sw_daily_band_spread was conditioned on level. Kept as an arithmetic case: the pair no longer occurs in this tree, and what it still checks is that the relation is that addition: got {} want 211.4530904415, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// round numbers, so the arithmetic is checkable without a calculator
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(200.0), Ratio::new(30.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 230.0);
    assert!(err <= 1e-12, "round numbers, so the arithmetic is checkable without a calculator: got {} want 230.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// a quieter sustained level with a wider daily departure — the two terms are independent and the row must not assume otherwise
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(150.0), Ratio::new(40.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 190.0);
    assert!(err <= 1e-12, "a quieter sustained level with a wider daily departure — the two terms are independent and the row must not assume otherwise: got {} want 190.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the author's case «the sustained level from sw_f107_design_long's first fixture plus the daily departure this row USED to be handed, before sw_daily_band_spread was conditioned on level. Kept as an arithmetic case: the pair no longer occurs in this tree, and what it still checks is that the relation is that addition.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Ratio::new(177.22160896), Ratio::new(34.2314814815)).expect("the sustained level from sw_f107_design_long's first fixture plus the daily departure this row USED to be handed, before sw_daily_band_spread was conditioned on level. Kept as an arithmetic case: the pair no longer occurs in this tree, and what it still checks is that the relation is that addition.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 211.4530904415);
    assert!(err <= 1e-12, "the sustained level from sw_f107_design_long's first fixture plus the daily departure this row USED to be handed, before sw_daily_band_spread was conditioned on level. Kept as an arithmetic case: the pair no longer occurs in this tree, and what it still checks is that the relation is that addition.: got {} and the author's code gave 211.4530904415; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «round numbers, so the arithmetic is checkable without a calculator.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Ratio::new(200.0), Ratio::new(30.0)).expect("round numbers, so the arithmetic is checkable without a calculator.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 230.0);
    assert!(err <= 1e-12, "round numbers, so the arithmetic is checkable without a calculator.: got {} and the author's code gave 230.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «a quieter sustained level with a wider daily departure — the two terms are independent and the row must not assume otherwise.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Ratio::new(150.0), Ratio::new(40.0)).expect("a quieter sustained level with a wider daily departure — the two terms are independent and the row must not assume otherwise.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 190.0);
    assert!(err <= 1e-12, "a quieter sustained level with a wider daily departure — the two terms are independent and the row must not assume otherwise.: got {} and the author's code gave 190.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Ratio::new(40.0), Ratio::new(10.0));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_5() {
    let got = model::evaluate(Ratio::new(380.0), Ratio::new(40.0));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 23 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_f107_design_short::evaluate;
    assert_eq!(evaluate(88.61080448, 17.11574074075).map(f64::to_bits), Ok(0x405a6e7fb7868bed), "at (88.61080448, 17.11574074075)");
    assert_eq!(evaluate(159.499448064, 30.80833333335).map(f64::to_bits), Ok(0x4067c9d9585f7def), "at (159.499448064, 30.80833333335)");
    assert_eq!(evaluate(175.4493928704, 33.889166666685).map(f64::to_bits), Ok(0x406a2ad57acf70ee), "at (175.4493928704, 33.889166666685)");
    assert_eq!(evaluate(177.22160896, 34.2314814815).map(f64::to_bits), Ok(0x406a6e7fb7868bed), "at (177.22160896, 34.2314814815)");
    assert_eq!(evaluate(178.9938250496, 34.573796296314995).map(f64::to_bits), Ok(0x406ab229f43da6ec), "at (178.9938250496, 34.573796296314995)");
    assert_eq!(evaluate(194.94376985600002, 37.65462962965).map(f64::to_bits), Ok(0x406d132616ad99ec), "at (194.94376985600002, 37.65462962965)");
    assert!(matches!(evaluate(354.44321792, 68.462962963), Err(MethodError::Refused(_))), "at (354.44321792, 68.462962963)");
    assert_eq!(evaluate(100.0, 15.0).map(f64::to_bits), Ok(0x405cc00000000000), "at (100.0, 15.0)");
    assert_eq!(evaluate(180.0, 27.0).map(f64::to_bits), Ok(0x4069e00000000000), "at (180.0, 27.0)");
    assert_eq!(evaluate(198.0, 29.7).map(f64::to_bits), Ok(0x406c766666666666), "at (198.0, 29.7)");
    assert_eq!(evaluate(200.0, 30.0).map(f64::to_bits), Ok(0x406cc00000000000), "at (200.0, 30.0)");
    assert_eq!(evaluate(202.0, 30.3).map(f64::to_bits), Ok(0x406d09999999999a), "at (202.0, 30.3)");
    assert_eq!(evaluate(220.00000000000003, 33.0).map(f64::to_bits), Ok(0x406fa00000000001), "at (220.00000000000003, 33.0)");
    assert!(matches!(evaluate(400.0, 60.0), Err(MethodError::Refused(_))), "at (400.0, 60.0)");
    assert_eq!(evaluate(75.0, 20.0).map(f64::to_bits), Ok(0x4057c00000000000), "at (75.0, 20.0)");
    assert_eq!(evaluate(135.0, 36.0).map(f64::to_bits), Ok(0x4065600000000000), "at (135.0, 36.0)");
    assert_eq!(evaluate(148.5, 39.6).map(f64::to_bits), Ok(0x4067833333333333), "at (148.5, 39.6)");
    assert_eq!(evaluate(150.0, 40.0).map(f64::to_bits), Ok(0x4067c00000000000), "at (150.0, 40.0)");
    assert_eq!(evaluate(151.5, 40.4).map(f64::to_bits), Ok(0x4067fccccccccccd), "at (151.5, 40.4)");
    assert_eq!(evaluate(165.0, 44.0).map(f64::to_bits), Ok(0x406a200000000000), "at (165.0, 44.0)");
    assert_eq!(evaluate(300.0, 80.0).map(f64::to_bits), Ok(0x4077c00000000000), "at (300.0, 80.0)");
    assert!(matches!(evaluate(40.0, 10.0), Err(MethodError::Refused(_))), "at (40.0, 10.0)");
    assert!(matches!(evaluate(380.0, 40.0), Err(MethodError::Refused(_))), "at (380.0, 40.0)");
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `the sustained level from sw_f107_design_long's first fixture plus the daily departure this row USED to be handed, before sw_daily_band_spread was conditioned on level. Kept as an arithmetic case: the pair no longer occurs in this tree, and what it still checks is that the relation is that addition` and the declared domain 60 … 400.
///
/// One per cent, not a decade. These domains are design bands — an altitude
/// range somebody chose, not a range over which the mathematics holds — so a
/// decade leaves most of them legitimately, and a check that cries wolf is a
/// check people turn off. What is left is still worth asking: a relation that
/// refuses at the immediate neighbours of the one point somebody verified is
/// either discontinuous there, or has a domain declared tighter than the
/// physics. Both are sheet questions, and both are invisible from the fixture.
#[test]
fn answers_near_the_known_good_point() {
    let mut refused: Vec<String> = Vec::new();
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(177.22160896 * scale), Ratio::new(34.2314814815)) {
            refused.push(format!("sustained x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(177.22160896), Ratio::new(34.2314814815 * scale)) {
            refused.push(format!("daily x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_f107_design_short refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 60 … 400 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
        refused
    );
}

/// Every answer sits inside the declared domain, and no call panics.
///
/// Not a restatement of the generated guard: it proves the guard is reachable,
/// that nothing routes around it, and that a hole cannot return a value that
/// is not a number. A division by zero inside a hole is caught by no guard.
#[test]
fn every_answer_is_inside_the_declared_domain() {
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(177.22160896 * scale), Ratio::new(34.2314814815)) {
            assert!(v.get().is_finite(), "sw_f107_design_short produced a value that is not a number for F107_short");
            assert!(v.get() >= 60.0 && v.get() <= 400.0, "sw_f107_design_short answered {} for F107_short, outside its declared domain 60 … 400 — the guard did not stop it", v.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(177.22160896), Ratio::new(34.2314814815 * scale)) {
            assert!(v.get().is_finite(), "sw_f107_design_short produced a value that is not a number for F107_short");
            assert!(v.get() >= 60.0 && v.get() <= 400.0, "sw_f107_design_short answered {} for F107_short, outside its declared domain 60 … 400 — the guard did not stop it", v.get());
        }
    }
}

/// The same inputs give a bit-identical answer.
///
/// A relation that reaches a clock, a hash order or any hidden state fails
/// here and nowhere else, and it is the one defect that makes bit-for-bit
/// agreement across the faces impossible rather than merely hard.
#[test]
fn the_same_inputs_give_the_same_answer() {
    let a = model::evaluate(Ratio::new(177.22160896), Ratio::new(34.2314814815));
    let b = model::evaluate(Ratio::new(177.22160896), Ratio::new(34.2314814815));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_f107_design_short is not deterministic for F107_short: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_f107_design_short refused on one call and answered on the other"),
    }
}

/// The prior implementation, over the grid it was exported on.
///
/// Migrated from `prf_density.m:227 (01_kernel/sw_study/06_density, local function designWindow_)`. These numbers are a second opinion and never
/// an expected value: an implementation cannot supply its own, and the
/// prior tool is an implementation. A disagreement is a finding about
/// one of the two.
#[test]
fn agrees_with_the_prior_implementation() {
    const GRID: &str = include_str!("parity.csv");
    const TOL: f64 = 1e-9;

    let mut lines = GRID
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'));
    let header: Vec<&str> = lines
        .next()
        .expect("parity.csv is empty — a grid with no header is not a grid")
        .split(',')
        .map(str::trim)
        .collect();

    // Columns are keyed by binding, as fixtures are, so a grid exported
    // with the columns in another order still lines up — and one missing
    // a column this node reads fails by name rather than by position.
    let want = ["sustained", "daily"];
    let col: Vec<usize> = want
        .iter()
        .map(|w| {
            header.iter().position(|h| h == w).unwrap_or_else(|| {
                panic!("parity.csv has no column '{w}' — this node reads it, so the grid cannot be compared. Columns present: {header:?}")
            })
        })
        .collect();
    let out = header.len() - 1;
    assert!(
        header[out].starts_with("matlab_"),
        "the last column of parity.csv is '{}' — it must be the prior implementation's answer, named matlab_<symbol>, so that a column added on the end cannot silently become the thing being compared",
        header[out]
    );

    let mut rows = 0usize;
    let mut worst = 0.0f64;
    let mut findings = Vec::<String>::new();
    for (n, line) in lines.enumerate() {
        let line_no = n + 2;
        let row: Vec<f64> = line
            .split(',')
            .map(|c| {
                c.trim().parse::<f64>().unwrap_or_else(|_| {
                    panic!("parity.csv line {line_no}: '{}' is not a number", c.trim())
                })
            })
            .collect();
        assert_eq!(
            row.len(),
            header.len(),
            "parity.csv line {line_no}: {} value(s) against {} column(s)",
            row.len(),
            header.len()
        );
        rows += 1;

        // A refusal here is itself a finding: the prior implementation
        // answered this point, so either its inputs were outside a domain
        // this node declares too narrowly, or the guard is wrong.
        let got = match model::evaluate(Ratio::new(row[col[0]]), Ratio::new(row[col[1]])) {
            Ok(v) => v.get(),
            Err(e) => {
                findings.push(format!(
                    "line {line_no}: this engine refused a point the prior implementation answered ({e:?})"
                ));
                continue;
            }
        };
        let err = relative_error(got, row[out]);
        if err > worst {
            worst = err;
        }
        if err > TOL {
            findings.push(format!(
                "line {line_no}: this engine {got}, the prior implementation {}, relative difference {err}",
                row[out]
            ));
        }
    }

    assert!(
        rows > 0,
        "parity.csv has a header and no rows — a grid that compares nothing passes, which is worse than not having one"
    );
    assert!(
        findings.is_empty(),
        "sw_f107_design_short: {} of {rows} grid row(s) disagree with the prior implementation `prf_density.m:227 (01_kernel/sw_study/06_density, local function designWindow_)` beyond {TOL} (worst {worst}).\n{}\nThis is a finding about one of the two implementations, not a build failure and not proof this one is wrong — both classes have been found before. Take it to the node owner. Do not widen parity_tolerance and do not edit parity.csv to agree.",
        findings.len(),
        findings.join("\n")
    );
}

