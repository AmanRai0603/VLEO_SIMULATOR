// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_ap_design_short`.
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

/// the sustained Ap level with the daily departure this row USED to be handed, before sw_ap_daily_band_spread was conditioned on level. Kept as an arithmetic case: the conditioned departure at Ap 26.70 is 63.85, not 15.00
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(26.69530964), Ratio::new(15.0018518519)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 41.6971614919);
    assert!(err <= 1e-12, "the sustained Ap level with the daily departure this row USED to be handed, before sw_ap_daily_band_spread was conditioned on level. Kept as an arithmetic case: the conditioned departure at Ap 26.70 is 63.85, not 15.00: got {} want 41.6971614919, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// round numbers, checkable without a calculator
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(25.0), Ratio::new(15.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 40.0);
    assert!(err <= 1e-12, "round numbers, checkable without a calculator: got {} want 40.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// a higher sustained level with a narrower daily departure — the terms are independent and the row must not assume otherwise
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(30.0), Ratio::new(12.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 42.0);
    assert!(err <= 1e-12, "a higher sustained level with a narrower daily departure — the terms are independent and the row must not assume otherwise: got {} want 42.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the author's case «the sustained Ap level with the daily departure this row USED to be handed, before sw_ap_daily_band_spread was conditioned on level. Kept as an arithmetic case: the conditioned departure at Ap 26.70 is 63.85, not 15.00.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Ratio::new(26.69530964), Ratio::new(15.0018518519)).expect("the sustained Ap level with the daily departure this row USED to be handed, before sw_ap_daily_band_spread was conditioned on level. Kept as an arithmetic case: the conditioned departure at Ap 26.70 is 63.85, not 15.00.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 41.6971614919);
    assert!(err <= 1e-12, "the sustained Ap level with the daily departure this row USED to be handed, before sw_ap_daily_band_spread was conditioned on level. Kept as an arithmetic case: the conditioned departure at Ap 26.70 is 63.85, not 15.00.: got {} and the author's code gave 41.6971614919; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «round numbers, checkable without a calculator.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Ratio::new(25.0), Ratio::new(15.0)).expect("round numbers, checkable without a calculator.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 40.0);
    assert!(err <= 1e-12, "round numbers, checkable without a calculator.: got {} and the author's code gave 40.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «a higher sustained level with a narrower daily departure — the terms are independent and the row must not assume otherwise.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Ratio::new(30.0), Ratio::new(12.0)).expect("a higher sustained level with a narrower daily departure — the terms are independent and the row must not assume otherwise.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 42.0);
    assert!(err <= 1e-12, "a higher sustained level with a narrower daily departure — the terms are independent and the row must not assume otherwise.: got {} and the author's code gave 42.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Ratio::new(-20.0), Ratio::new(5.0));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_5() {
    let got = model::evaluate(Ratio::new(300.0), Ratio::new(150.0));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 23 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_ap_design_short::evaluate;
    assert_eq!(evaluate(13.34765482, 7.50092592595).map(f64::to_bits), Ok(0x4034d93c9677dedc), "at (13.34765482, 7.50092592595)");
    assert_eq!(evaluate(24.025778676, 13.50166666671).map(f64::to_bits), Ok(0x4042c3835438aef9), "at (24.025778676, 13.50166666671)");
    assert_eq!(evaluate(26.4283565436, 14.851833333381).map(f64::to_bits), Ok(0x4044a3dd430b26de), "at (26.4283565436, 14.851833333381)");
    assert_eq!(evaluate(26.69530964, 15.0018518519).map(f64::to_bits), Ok(0x4044d93c9677dedc), "at (26.69530964, 15.0018518519)");
    assert_eq!(evaluate(26.962262736400003, 15.151870370419).map(f64::to_bits), Ok(0x40450e9be9e496d9), "at (26.962262736400003, 15.151870370419)");
    assert_eq!(evaluate(29.364840604000005, 16.502037037090002).map(f64::to_bits), Ok(0x4046eef5d8b70ebf), "at (29.364840604000005, 16.502037037090002)");
    assert_eq!(evaluate(53.39061928, 30.0037037038).map(f64::to_bits), Ok(0x4054d93c9677dedc), "at (53.39061928, 30.0037037038)");
    assert_eq!(evaluate(12.5, 7.5).map(f64::to_bits), Ok(0x4034000000000000), "at (12.5, 7.5)");
    assert_eq!(evaluate(22.5, 13.5).map(f64::to_bits), Ok(0x4042000000000000), "at (22.5, 13.5)");
    assert_eq!(evaluate(24.75, 14.85).map(f64::to_bits), Ok(0x4043cccccccccccd), "at (24.75, 14.85)");
    assert_eq!(evaluate(25.0, 15.0).map(f64::to_bits), Ok(0x4044000000000000), "at (25.0, 15.0)");
    assert_eq!(evaluate(25.25, 15.15).map(f64::to_bits), Ok(0x4044333333333333), "at (25.25, 15.15)");
    assert_eq!(evaluate(27.500000000000004, 16.5).map(f64::to_bits), Ok(0x4046000000000000), "at (27.500000000000004, 16.5)");
    assert_eq!(evaluate(50.0, 30.0).map(f64::to_bits), Ok(0x4054000000000000), "at (50.0, 30.0)");
    assert_eq!(evaluate(15.0, 6.0).map(f64::to_bits), Ok(0x4035000000000000), "at (15.0, 6.0)");
    assert_eq!(evaluate(27.0, 10.8).map(f64::to_bits), Ok(0x4042e66666666666), "at (27.0, 10.8)");
    assert_eq!(evaluate(29.7, 11.879999999999999).map(f64::to_bits), Ok(0x4044ca3d70a3d70a), "at (29.7, 11.879999999999999)");
    assert_eq!(evaluate(30.0, 12.0).map(f64::to_bits), Ok(0x4045000000000000), "at (30.0, 12.0)");
    assert_eq!(evaluate(30.3, 12.120000000000001).map(f64::to_bits), Ok(0x404535c28f5c28f6), "at (30.3, 12.120000000000001)");
    assert_eq!(evaluate(33.0, 13.200000000000001).map(f64::to_bits), Ok(0x404719999999999a), "at (33.0, 13.200000000000001)");
    assert_eq!(evaluate(60.0, 24.0).map(f64::to_bits), Ok(0x4055000000000000), "at (60.0, 24.0)");
    assert!(matches!(evaluate(-20.0, 5.0), Err(MethodError::Refused(_))), "at (-20.0, 5.0)");
    assert!(matches!(evaluate(300.0, 150.0), Err(MethodError::Refused(_))), "at (300.0, 150.0)");
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `the sustained Ap level with the daily departure this row USED to be handed, before sw_ap_daily_band_spread was conditioned on level. Kept as an arithmetic case: the conditioned departure at Ap 26.70 is 63.85, not 15.00` and the declared domain 0 … 400.
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
        if let Err(f) = model::evaluate(Ratio::new(26.69530964 * scale), Ratio::new(15.0018518519)) {
            refused.push(format!("sustained x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(26.69530964), Ratio::new(15.0018518519 * scale)) {
            refused.push(format!("daily x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_ap_design_short refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 0 … 400 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Ratio::new(26.69530964 * scale), Ratio::new(15.0018518519)) {
            assert!(v.get().is_finite(), "sw_ap_design_short produced a value that is not a number for Ap_short");
            assert!(v.get() >= 0.0 && v.get() <= 400.0, "sw_ap_design_short answered {} for Ap_short, outside its declared domain 0 … 400 — the guard did not stop it", v.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(26.69530964), Ratio::new(15.0018518519 * scale)) {
            assert!(v.get().is_finite(), "sw_ap_design_short produced a value that is not a number for Ap_short");
            assert!(v.get() >= 0.0 && v.get() <= 400.0, "sw_ap_design_short answered {} for Ap_short, outside its declared domain 0 … 400 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Ratio::new(26.69530964), Ratio::new(15.0018518519));
    let b = model::evaluate(Ratio::new(26.69530964), Ratio::new(15.0018518519));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_ap_design_short is not deterministic for Ap_short: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_ap_design_short refused on one call and answered on the other"),
    }
}

/// The prior implementation, over the grid it was exported on.
///
/// Migrated from `prf_density.m:227 (01_kernel/sw_study/06_density, designWindow_, hotday for ap)`. These numbers are a second opinion and never
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
        "sw_ap_design_short: {} of {rows} grid row(s) disagree with the prior implementation `prf_density.m:227 (01_kernel/sw_study/06_density, designWindow_, hotday for ap)` beyond {TOL} (worst {worst}).\n{}\nThis is a finding about one of the two implementations, not a build failure and not proof this one is wrong — both classes have been found before. Take it to the node owner. Do not widen parity_tolerance and do not edit parity.csv to agree.",
        findings.len(),
        findings.join("\n")
    );
}

