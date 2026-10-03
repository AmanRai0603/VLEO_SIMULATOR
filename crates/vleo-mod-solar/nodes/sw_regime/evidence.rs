// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_regime`.
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

/// Ap 158.38 — the five-year return level — storm, as the table labels every day near it
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(158.38)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 3.0);
    assert!(err <= 1e-12, "Ap 158.38 — the five-year return level — storm, as the table labels every day near it: got {} want 3.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Ap 20 — the bottom of the producer's range — active; the table labels all 20s active
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(20.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 2.0);
    assert!(err <= 1e-12, "Ap 20 — the bottom of the producer's range — active; the table labels all 20s active: got {} want 2.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Ap 25 — the last active value in the published table; 26 is the first storm
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(25.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 2.0);
    assert!(err <= 1e-12, "Ap 25 — the last active value in the published table; 26 is the first storm: got {} want 2.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Ap 26 — the first storm value in the published table, and the boundary that matters
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Ratio::new(26.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 3.0);
    assert!(err <= 1e-12, "Ap 26 — the first storm value in the published table, and the boundary that matters: got {} want 3.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Ap 132 — the G3 design value — storm
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Ratio::new(132.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 3.0);
    assert!(err <= 1e-12, "Ap 132 — the G3 design value — storm: got {} want 3.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Ap 230 — the top of the producer's range — storm
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Ratio::new(230.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 3.0);
    assert!(err <= 1e-12, "Ap 230 — the top of the producer's range — storm: got {} want 3.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the author's case «Ap 158.38 — the five-year return level — storm, as the table labels every day near it.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Ratio::new(158.38)).expect("Ap 158.38 — the five-year return level — storm, as the table labels every day near it.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 3.0);
    assert!(err <= 1e-12, "Ap 158.38 — the five-year return level — storm, as the table labels every day near it.: got {} and the author's code gave 3.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «Ap 20 — the bottom of the producer's range — active; the table labels all 20s active.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Ratio::new(20.0)).expect("Ap 20 — the bottom of the producer's range — active; the table labels all 20s active.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 2.0);
    assert!(err <= 1e-12, "Ap 20 — the bottom of the producer's range — active; the table labels all 20s active.: got {} and the author's code gave 2.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «Ap 25 — the last active value in the published table; 26 is the first storm.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Ratio::new(25.0)).expect("Ap 25 — the last active value in the published table; 26 is the first storm.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 2.0);
    assert!(err <= 1e-12, "Ap 25 — the last active value in the published table; 26 is the first storm.: got {} and the author's code gave 2.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «Ap 26 — the first storm value in the published table, and the boundary that matters.», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Ratio::new(26.0)).expect("Ap 26 — the first storm value in the published table, and the boundary that matters.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 3.0);
    assert!(err <= 1e-12, "Ap 26 — the first storm value in the published table, and the boundary that matters.: got {} and the author's code gave 3.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «Ap 132 — the G3 design value — storm.», from their own  code.
#[test]
fn case_5() {
    let got = model::evaluate(Ratio::new(132.0)).expect("Ap 132 — the G3 design value — storm.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 3.0);
    assert!(err <= 1e-12, "Ap 132 — the G3 design value — storm.: got {} and the author's code gave 3.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «Ap 230 — the top of the producer's range — storm.», from their own  code.
#[test]
fn case_6() {
    let got = model::evaluate(Ratio::new(230.0)).expect("Ap 230 — the top of the producer's range — storm.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 3.0);
    assert!(err <= 1e-12, "Ap 230 — the top of the producer's range — storm.: got {} and the author's code gave 3.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «a value that is not a number: the node must refuse it, as every method does at its door. The code before 1.1 answered regime 3 here, which no value is», from their own  code.
#[test]
fn case_7() {
    let got = model::evaluate(Ratio::new(f64::NAN));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "a value that is not a number: the node must refuse it, as every method does at its door. The code before 1.1 answered regime 3 here, which no value is: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 43 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_regime::evaluate;
    assert_eq!(evaluate(79.19).map(f64::to_bits), Ok(0x4008000000000000), "at (79.19)");
    assert_eq!(evaluate(142.542).map(f64::to_bits), Ok(0x4008000000000000), "at (142.542)");
    assert_eq!(evaluate(156.7962).map(f64::to_bits), Ok(0x4008000000000000), "at (156.7962)");
    assert_eq!(evaluate(158.38).map(f64::to_bits), Ok(0x4008000000000000), "at (158.38)");
    assert_eq!(evaluate(159.9638).map(f64::to_bits), Ok(0x4008000000000000), "at (159.9638)");
    assert_eq!(evaluate(174.21800000000002).map(f64::to_bits), Ok(0x4008000000000000), "at (174.21800000000002)");
    assert_eq!(evaluate(316.76).map(f64::to_bits), Ok(0x4008000000000000), "at (316.76)");
    assert_eq!(evaluate(10.0).map(f64::to_bits), Ok(0x4000000000000000), "at (10.0)");
    assert_eq!(evaluate(18.0).map(f64::to_bits), Ok(0x4000000000000000), "at (18.0)");
    assert_eq!(evaluate(19.8).map(f64::to_bits), Ok(0x4000000000000000), "at (19.8)");
    assert_eq!(evaluate(20.0).map(f64::to_bits), Ok(0x4000000000000000), "at (20.0)");
    assert_eq!(evaluate(20.2).map(f64::to_bits), Ok(0x4000000000000000), "at (20.2)");
    assert_eq!(evaluate(22.0).map(f64::to_bits), Ok(0x4000000000000000), "at (22.0)");
    assert_eq!(evaluate(40.0).map(f64::to_bits), Ok(0x4008000000000000), "at (40.0)");
    assert_eq!(evaluate(12.5).map(f64::to_bits), Ok(0x4000000000000000), "at (12.5)");
    assert_eq!(evaluate(22.5).map(f64::to_bits), Ok(0x4000000000000000), "at (22.5)");
    assert_eq!(evaluate(24.75).map(f64::to_bits), Ok(0x4000000000000000), "at (24.75)");
    assert_eq!(evaluate(25.0).map(f64::to_bits), Ok(0x4000000000000000), "at (25.0)");
    assert_eq!(evaluate(25.25).map(f64::to_bits), Ok(0x4008000000000000), "at (25.25)");
    assert_eq!(evaluate(27.500000000000004).map(f64::to_bits), Ok(0x4008000000000000), "at (27.500000000000004)");
    assert_eq!(evaluate(50.0).map(f64::to_bits), Ok(0x4008000000000000), "at (50.0)");
    assert_eq!(evaluate(13.0).map(f64::to_bits), Ok(0x4000000000000000), "at (13.0)");
    assert_eq!(evaluate(23.400000000000002).map(f64::to_bits), Ok(0x4000000000000000), "at (23.400000000000002)");
    assert_eq!(evaluate(25.74).map(f64::to_bits), Ok(0x4008000000000000), "at (25.74)");
    assert_eq!(evaluate(26.0).map(f64::to_bits), Ok(0x4008000000000000), "at (26.0)");
    assert_eq!(evaluate(26.26).map(f64::to_bits), Ok(0x4008000000000000), "at (26.26)");
    assert_eq!(evaluate(28.6).map(f64::to_bits), Ok(0x4008000000000000), "at (28.6)");
    assert_eq!(evaluate(52.0).map(f64::to_bits), Ok(0x4008000000000000), "at (52.0)");
    assert_eq!(evaluate(66.0).map(f64::to_bits), Ok(0x4008000000000000), "at (66.0)");
    assert_eq!(evaluate(118.8).map(f64::to_bits), Ok(0x4008000000000000), "at (118.8)");
    assert_eq!(evaluate(130.68).map(f64::to_bits), Ok(0x4008000000000000), "at (130.68)");
    assert_eq!(evaluate(132.0).map(f64::to_bits), Ok(0x4008000000000000), "at (132.0)");
    assert_eq!(evaluate(133.32).map(f64::to_bits), Ok(0x4008000000000000), "at (133.32)");
    assert_eq!(evaluate(145.20000000000002).map(f64::to_bits), Ok(0x4008000000000000), "at (145.20000000000002)");
    assert_eq!(evaluate(264.0).map(f64::to_bits), Ok(0x4008000000000000), "at (264.0)");
    assert_eq!(evaluate(115.0).map(f64::to_bits), Ok(0x4008000000000000), "at (115.0)");
    assert_eq!(evaluate(207.0).map(f64::to_bits), Ok(0x4008000000000000), "at (207.0)");
    assert_eq!(evaluate(227.7).map(f64::to_bits), Ok(0x4008000000000000), "at (227.7)");
    assert_eq!(evaluate(230.0).map(f64::to_bits), Ok(0x4008000000000000), "at (230.0)");
    assert_eq!(evaluate(232.3).map(f64::to_bits), Ok(0x4008000000000000), "at (232.3)");
    assert_eq!(evaluate(253.00000000000003).map(f64::to_bits), Ok(0x4008000000000000), "at (253.00000000000003)");
    assert_eq!(evaluate(460.0).map(f64::to_bits), Ok(0x4008000000000000), "at (460.0)");
    assert!(matches!(evaluate(f64::NAN), Err(MethodError::Refused(_))), "at (f64::NAN)");
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `Ap 158.38 — the five-year return level — storm, as the table labels every day near it` and the declared domain 1 … 3.
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
        if let Err(f) = model::evaluate(Ratio::new(158.38 * scale)) {
            refused.push(format!("ap x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_regime refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 1 … 3 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Ratio::new(158.38 * scale)) {
            assert!(v.get().is_finite(), "sw_regime produced a value that is not a number for regime");
            assert!(v.get() >= 1.0 && v.get() <= 3.0, "sw_regime answered {} for regime, outside its declared domain 1 … 3 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Ratio::new(158.38));
    let b = model::evaluate(Ratio::new(158.38));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_regime is not deterministic for regime: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_regime refused on one call and answered on the other"),
    }
}

/// The prior implementation, over the grid it was exported on.
///
/// Migrated from `prf_cluster.m (01_kernel/sw_study, three-component Gaussian mixture on daily Ap)`. These numbers are a second opinion and never
/// an expected value: an implementation cannot supply its own, and the
/// prior tool is an implementation. A disagreement is a finding about
/// one of the two.
#[test]
fn agrees_with_the_prior_implementation() {
    const GRID: &str = include_str!("parity.csv");
    const TOL: f64 = 1e-12;

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
    let want = ["ap"];
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
        let got = match model::evaluate(Ratio::new(row[col[0]])) {
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
        "sw_regime: {} of {rows} grid row(s) disagree with the prior implementation `prf_cluster.m (01_kernel/sw_study, three-component Gaussian mixture on daily Ap)` beyond {TOL} (worst {worst}).\n{}\nThis is a finding about one of the two implementations, not a build failure and not proof this one is wrong — both classes have been found before. Take it to the node owner. Do not widen parity_tolerance and do not edit parity.csv to agree.",
        findings.len(),
        findings.join("\n")
    );
}

