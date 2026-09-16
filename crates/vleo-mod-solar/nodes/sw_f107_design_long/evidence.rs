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
            assert!(v.get().is_finite(), "sw_f107_design_long produced a value that is not a number");
            assert!(v.get() >= 60.0 && v.get() <= 400.0, "sw_f107_design_long answered {}, outside its declared domain 60 … 400 — the guard did not stop it", v.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(160.0), Ratio::new(13.454382 * scale)) {
            assert!(v.get().is_finite(), "sw_f107_design_long produced a value that is not a number");
            assert!(v.get() >= 60.0 && v.get() <= 400.0, "sw_f107_design_long answered {}, outside its declared domain 60 … 400 — the guard did not stop it", v.get());
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
        (Ok(x), Ok(y)) => assert!(x.get().to_bits() == y.get().to_bits(), "sw_f107_design_long is not deterministic: {} then {}", x.get(), y.get()),
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

