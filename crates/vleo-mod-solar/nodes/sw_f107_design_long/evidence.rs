// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_f107_design_long`.
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

/// a mid-cycle centre of 160 with this repository's own measured spread — 160 + 1.28 × 13.454382
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(160.0), Ratio::new(13.454382)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 177.22160896);
    assert!(err <= 1e-12, "a mid-cycle centre of 160 with this repository's own measured spread — 160 + 1.28 × 13.454382: got {} want 177.22160896, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// round numbers, so the arithmetic is checkable without a calculator — 100 + 1.28 × 20
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(100.0), Ratio::new(20.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 125.6);
    assert!(err <= 1e-12, "round numbers, so the arithmetic is checkable without a calculator — 100 + 1.28 × 20: got {} want 125.6, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// a cycle-maximum centre of 240, which is where the upper guard starts to matter
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(240.0), Ratio::new(13.454382)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 257.22160896);
    assert!(err <= 1e-12, "a cycle-maximum centre of 240, which is where the upper guard starts to matter: got {} want 257.22160896, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the author's case «a mid-cycle centre of 160 with this repository's own measured spread — 160 + 1.28 × 13.454382.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Ratio::new(160.0), Ratio::new(13.454382)).expect("a mid-cycle centre of 160 with this repository's own measured spread — 160 + 1.28 × 13.454382.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 177.22160896);
    assert!(err <= 1e-12, "a mid-cycle centre of 160 with this repository's own measured spread — 160 + 1.28 × 13.454382.: got {} and the author's code gave 177.22160896; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «round numbers, so the arithmetic is checkable without a calculator — 100 + 1.28 × 20.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Ratio::new(100.0), Ratio::new(20.0)).expect("round numbers, so the arithmetic is checkable without a calculator — 100 + 1.28 × 20.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 125.6);
    assert!(err <= 1e-12, "round numbers, so the arithmetic is checkable without a calculator — 100 + 1.28 × 20.: got {} and the author's code gave 125.6; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «a cycle-maximum centre of 240, which is where the upper guard starts to matter.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Ratio::new(240.0), Ratio::new(13.454382)).expect("a cycle-maximum centre of 240, which is where the upper guard starts to matter.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 257.22160896);
    assert!(err <= 1e-12, "a cycle-maximum centre of 240, which is where the upper guard starts to matter.: got {} and the author's code gave 257.22160896; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Ratio::new(50.0), Ratio::new(0.0));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_5() {
    let got = model::evaluate(Ratio::new(390.0), Ratio::new(10.0));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 23 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_f107_design_long::evaluate;
    assert_eq!(evaluate(80.0, 6.727191).map(f64::to_bits), Ok(0x405627176bac766b), "at (80.0, 6.727191)");
    assert_eq!(evaluate(144.0, 12.1089438).map(f64::to_bits), Ok(0x4063effb7a819dc7), "at (144.0, 12.1089438)");
    assert_eq!(evaluate(158.4, 13.319838180000001).map(f64::to_bits), Ok(0x4065ee616d282d8e), "at (158.4, 13.319838180000001)");
    assert_eq!(evaluate(160.0, 13.454382).map(f64::to_bits), Ok(0x406627176bac766b), "at (160.0, 13.454382)");
    assert_eq!(evaluate(161.6, 13.58892582).map(f64::to_bits), Ok(0x40665fcd6a30bf48), "at (161.6, 13.58892582)");
    assert_eq!(evaluate(176.0, 14.799820200000003).map(f64::to_bits), Ok(0x40685e335cd74f10), "at (176.0, 14.799820200000003)");
    assert_eq!(evaluate(320.0, 26.908764).map(f64::to_bits), Ok(0x407627176bac766b), "at (320.0, 26.908764)");
    assert_eq!(evaluate(50.0, 10.0).map(f64::to_bits), Ok(0x404f666666666666), "at (50.0, 10.0)");
    assert_eq!(evaluate(90.0, 18.0).map(f64::to_bits), Ok(0x405c428f5c28f5c2), "at (90.0, 18.0)");
    assert_eq!(evaluate(99.0, 19.8).map(f64::to_bits), Ok(0x405f1604189374bc), "at (99.0, 19.8)");
    assert_eq!(evaluate(100.0, 20.0).map(f64::to_bits), Ok(0x405f666666666666), "at (100.0, 20.0)");
    assert_eq!(evaluate(101.0, 20.2).map(f64::to_bits), Ok(0x405fb6c8b4395810), "at (101.0, 20.2)");
    assert_eq!(evaluate(110.00000000000001, 22.0).map(f64::to_bits), Ok(0x4061451eb851eb86), "at (110.00000000000001, 22.0)");
    assert_eq!(evaluate(200.0, 40.0).map(f64::to_bits), Ok(0x406f666666666666), "at (200.0, 40.0)");
    assert_eq!(evaluate(120.0, 6.727191).map(f64::to_bits), Ok(0x4060138bb5d63b36), "at (120.0, 6.727191)");
    assert_eq!(evaluate(216.0, 12.1089438).map(f64::to_bits), Ok(0x406ceffb7a819dc7), "at (216.0, 12.1089438)");
    assert_eq!(evaluate(237.6, 13.319838180000001).map(f64::to_bits), Ok(0x406fd4c7d38e93f4), "at (237.6, 13.319838180000001)");
    assert_eq!(evaluate(240.0, 13.454382).map(f64::to_bits), Ok(0x4070138bb5d63b36), "at (240.0, 13.454382)");
    assert_eq!(evaluate(242.4, 13.58892582).map(f64::to_bits), Ok(0x40703cb381e52c71), "at (242.4, 13.58892582)");
    assert_eq!(evaluate(264.0, 14.799820200000003).map(f64::to_bits), Ok(0x4071af19ae6ba788), "at (264.0, 14.799820200000003)");
    assert!(matches!(evaluate(480.0, 26.908764), Err(MethodError::Refused(_))), "at (480.0, 26.908764)");
    assert!(matches!(evaluate(50.0, 0.0), Err(MethodError::Refused(_))), "at (50.0, 0.0)");
    assert!(matches!(evaluate(390.0, 10.0), Err(MethodError::Refused(_))), "at (390.0, 10.0)");
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `a mid-cycle centre of 160 with this repository's own measured spread — 160 + 1.28 × 13.454382` and the declared domain 60 … 400.
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
        "sw_f107_design_long refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 60 … 400 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
            assert!(v.get().is_finite(), "sw_f107_design_long produced a value that is not a number for F107_long");
            assert!(v.get() >= 60.0 && v.get() <= 400.0, "sw_f107_design_long answered {} for F107_long, outside its declared domain 60 … 400 — the guard did not stop it", v.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(160.0), Ratio::new(13.454382 * scale)) {
            assert!(v.get().is_finite(), "sw_f107_design_long produced a value that is not a number for F107_long");
            assert!(v.get() >= 60.0 && v.get() <= 400.0, "sw_f107_design_long answered {} for F107_long, outside its declared domain 60 … 400 — the guard did not stop it", v.get());
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
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_f107_design_long is not deterministic for F107_long: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_f107_design_long refused on one call and answered on the other"),
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
        "sw_f107_design_long: {} of {rows} grid row(s) disagree with the prior implementation `prf_density.m:217 (01_kernel/sw_study/06_density, local function designWindow_)` beyond {TOL} (worst {worst}).\n{}\nThis is a finding about one of the two implementations, not a build failure and not proof this one is wrong — both classes have been found before. Take it to the node owner. Do not widen parity_tolerance and do not edit parity.csv to agree.",
        findings.len(),
        findings.join("\n")
    );
}

