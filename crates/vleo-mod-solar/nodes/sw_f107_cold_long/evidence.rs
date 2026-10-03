// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_f107_cold_long`.
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

/// a mid-cycle centre of 160 with this repository's own measured spread — 160 - 1.28 × 13.454382
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(160.0), Ratio::new(13.454382)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 142.77839104);
    assert!(err <= 1e-12, "a mid-cycle centre of 160 with this repository's own measured spread — 160 - 1.28 × 13.454382: got {} want 142.77839104, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// round numbers, so the arithmetic is checkable without a calculator — 100 - 1.28 × 20
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(100.0), Ratio::new(20.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 74.4);
    assert!(err <= 1e-12, "round numbers, so the arithmetic is checkable without a calculator — 100 - 1.28 × 20: got {} want 74.4, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the centre sw_central_expectation actually gives at the declared epoch, 86.8497 — the case that matters, nine and a half sfu above the guard
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(86.8497), Ratio::new(13.454382)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 69.62809104);
    assert!(err <= 1e-10, "the centre sw_central_expectation actually gives at the declared epoch, 86.8497 — the case that matters, nine and a half sfu above the guard: got {} want 69.62809104, relative error {} exceeds the declared tolerance 1e-10. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the author's case «a mid-cycle centre of 160 with this repository's own measured spread — 160 - 1.28 × 13.454382.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Ratio::new(160.0), Ratio::new(13.454382)).expect("a mid-cycle centre of 160 with this repository's own measured spread — 160 - 1.28 × 13.454382.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 142.77839104);
    assert!(err <= 1e-12, "a mid-cycle centre of 160 with this repository's own measured spread — 160 - 1.28 × 13.454382.: got {} and the author's code gave 142.77839104; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «round numbers, so the arithmetic is checkable without a calculator — 100 - 1.28 × 20.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Ratio::new(100.0), Ratio::new(20.0)).expect("round numbers, so the arithmetic is checkable without a calculator — 100 - 1.28 × 20.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 74.4);
    assert!(err <= 1e-12, "round numbers, so the arithmetic is checkable without a calculator — 100 - 1.28 × 20.: got {} and the author's code gave 74.4; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «the centre sw_central_expectation actually gives at the declared epoch, 86.8497 — the case that matters, nine and a half sfu above the guard.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Ratio::new(86.8497), Ratio::new(13.454382)).expect("the centre sw_central_expectation actually gives at the declared epoch, 86.8497 — the case that matters, nine and a half sfu above the guard.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 69.62809104);
    assert!(err <= 1e-10, "the centre sw_central_expectation actually gives at the declared epoch, 86.8497 — the case that matters, nine and a half sfu above the guard.: got {} and the author's code gave 69.62809104; relative error {} is more than their tolerance 1e-10. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Ratio::new(100.0), Ratio::new(40.0));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_5() {
    let got = model::evaluate(Ratio::new(500.0), Ratio::new(10.0));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_6() {
    let got = model::evaluate(Ratio::new(f64::INFINITY), Ratio::new(10.0));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 24 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_f107_cold_long::evaluate;
    assert_eq!(evaluate(80.0, 6.727191).map(f64::to_bits), Ok(0x4051d8e894538995), "at (80.0, 6.727191)");
    assert_eq!(evaluate(144.0, 12.1089438).map(f64::to_bits), Ok(0x40601004857e6239), "at (144.0, 12.1089438)");
    assert_eq!(evaluate(158.4, 13.319838180000001).map(f64::to_bits), Ok(0x4061ab382c716c0c), "at (158.4, 13.319838180000001)");
    assert_eq!(evaluate(160.0, 13.454382).map(f64::to_bits), Ok(0x4061d8e894538995), "at (160.0, 13.454382)");
    assert_eq!(evaluate(161.6, 13.58892582).map(f64::to_bits), Ok(0x40620698fc35a71e), "at (161.6, 13.58892582)");
    assert_eq!(evaluate(176.0, 14.799820200000003).map(f64::to_bits), Ok(0x4063a1cca328b0f0), "at (176.0, 14.799820200000003)");
    assert_eq!(evaluate(320.0, 26.908764).map(f64::to_bits), Ok(0x4071d8e894538995), "at (320.0, 26.908764)");
    assert!(matches!(evaluate(50.0, 10.0), Err(MethodError::Refused(_))), "at (50.0, 10.0)");
    assert_eq!(evaluate(90.0, 18.0).map(f64::to_bits), Ok(0x4050bd70a3d70a3e), "at (90.0, 18.0)");
    assert_eq!(evaluate(99.0, 19.8).map(f64::to_bits), Ok(0x405269fbe76c8b44), "at (99.0, 19.8)");
    assert_eq!(evaluate(100.0, 20.0).map(f64::to_bits), Ok(0x405299999999999a), "at (100.0, 20.0)");
    assert_eq!(evaluate(101.0, 20.2).map(f64::to_bits), Ok(0x4052c9374bc6a7f0), "at (101.0, 20.2)");
    assert_eq!(evaluate(110.00000000000001, 22.0).map(f64::to_bits), Ok(0x405475c28f5c28f7), "at (110.00000000000001, 22.0)");
    assert_eq!(evaluate(200.0, 40.0).map(f64::to_bits), Ok(0x406299999999999a), "at (200.0, 40.0)");
    assert!(matches!(evaluate(43.42485, 6.727191), Err(MethodError::Refused(_))), "at (43.42485, 6.727191)");
    assert_eq!(evaluate(78.16473, 12.1089438).map(f64::to_bits), Ok(0x404f5527f55edeab), "at (78.16473, 12.1089438)");
    assert_eq!(evaluate(85.981203, 13.319838180000001).map(f64::to_bits), Ok(0x40513ba2c6f42daa), "at (85.981203, 13.319838180000001)");
    assert_eq!(evaluate(86.8497, 13.454382).map(f64::to_bits), Ok(0x40516832a4c2ed7b), "at (86.8497, 13.454382)");
    assert_eq!(evaluate(87.718197, 13.58892582).map(f64::to_bits), Ok(0x405194c28291ad4c), "at (87.718197, 13.58892582)");
    assert_eq!(evaluate(95.53467, 14.799820200000003).map(f64::to_bits), Ok(0x405325d14ed66ba1), "at (95.53467, 14.799820200000003)");
    assert_eq!(evaluate(173.6994, 26.908764).map(f64::to_bits), Ok(0x40616832a4c2ed7b), "at (173.6994, 26.908764)");
    assert!(matches!(evaluate(100.0, 40.0), Err(MethodError::Refused(_))), "at (100.0, 40.0)");
    assert!(matches!(evaluate(500.0, 10.0), Err(MethodError::Refused(_))), "at (500.0, 10.0)");
    assert!(matches!(evaluate(f64::INFINITY, 10.0), Err(MethodError::Refused(_))), "at (f64::INFINITY, 10.0)");
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `a mid-cycle centre of 160 with this repository's own measured spread — 160 - 1.28 × 13.454382` and the declared domain 60 … 400.
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
        if let Err(f) = model::evaluate(Ratio::new(160.0 * scale), Ratio::new(13.454382)) {
            refused.push(format!("central x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(160.0), Ratio::new(13.454382 * scale)) {
            refused.push(format!("spread x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_f107_cold_long refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 60 … 400 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Ratio::new(160.0 * scale), Ratio::new(13.454382)) {
            assert!(v.get().is_finite(), "sw_f107_cold_long produced a value that is not a number for F107_cold_long");
            assert!(v.get() >= 60.0 && v.get() <= 400.0, "sw_f107_cold_long answered {} for F107_cold_long, outside its declared domain 60 … 400 — the guard did not stop it", v.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(160.0), Ratio::new(13.454382 * scale)) {
            assert!(v.get().is_finite(), "sw_f107_cold_long produced a value that is not a number for F107_cold_long");
            assert!(v.get() >= 60.0 && v.get() <= 400.0, "sw_f107_cold_long answered {} for F107_cold_long, outside its declared domain 60 … 400 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Ratio::new(160.0), Ratio::new(13.454382));
    let b = model::evaluate(Ratio::new(160.0), Ratio::new(13.454382));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_f107_cold_long is not deterministic for F107_cold_long: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_f107_cold_long refused on one call and answered on the other"),
    }
}

/// The prior implementation, over the grid it was exported on.
///
/// Migrated from `prf_density.m:217 (01_kernel/sw_study/06_density, local function designWindow_)`. These numbers are a second opinion and never
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
    let want = ["central", "spread"];
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
        "sw_f107_cold_long: {} of {rows} grid row(s) disagree with the prior implementation `prf_density.m:217 (01_kernel/sw_study/06_density, local function designWindow_)` beyond {TOL} (worst {worst}).\n{}\nThis is a finding about one of the two implementations, not a build failure and not proof this one is wrong — both classes have been found before. Take it to the node owner. Do not widen parity_tolerance and do not edit parity.csv to agree.",
        findings.len(),
        findings.join("\n")
    );
}

