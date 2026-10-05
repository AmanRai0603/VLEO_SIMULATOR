// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_ap_cold_long`.
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

/// this repository's own chain — the Ap centre with its measured spread
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(22.095389), Ratio::new(3.593688)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 17.49546836);
    assert!(err <= 1e-12, "this repository's own chain — the Ap centre with its measured spread: got {} want 17.49546836, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// round numbers, checkable without a calculator — 20 - 1.28 × 4
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(20.0), Ratio::new(4.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 14.88);
    assert!(err <= 1e-12, "round numbers, checkable without a calculator — 20 - 1.28 × 4: got {} want 14.88, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// a quiet centre of 16, closer to the floor the sheet argues about — the guard fires below a centre of 4.6
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(16.0), Ratio::new(3.593688)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 11.40007936);
    assert!(err <= 1e-12, "a quiet centre of 16, closer to the floor the sheet argues about — the guard fires below a centre of 4.6: got {} want 11.40007936, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the author's case «this repository's own chain — the Ap centre with its measured spread.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Ratio::new(22.095389), Ratio::new(3.593688)).expect("this repository's own chain — the Ap centre with its measured spread.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 17.49546836);
    assert!(err <= 1e-12, "this repository's own chain — the Ap centre with its measured spread.: got {} and the author's code gave 17.49546836; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «round numbers, checkable without a calculator — 20 - 1.28 × 4.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Ratio::new(20.0), Ratio::new(4.0)).expect("round numbers, checkable without a calculator — 20 - 1.28 × 4.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 14.88);
    assert!(err <= 1e-12, "round numbers, checkable without a calculator — 20 - 1.28 × 4.: got {} and the author's code gave 14.88; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «a quiet centre of 16, closer to the floor the sheet argues about — the guard fires below a centre of 4.6.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Ratio::new(16.0), Ratio::new(3.593688)).expect("a quiet centre of 16, closer to the floor the sheet argues about — the guard fires below a centre of 4.6.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 11.40007936);
    assert!(err <= 1e-12, "a quiet centre of 16, closer to the floor the sheet argues about — the guard fires below a centre of 4.6.: got {} and the author's code gave 11.40007936; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Ratio::new(1.0), Ratio::new(1.0));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_5() {
    let got = model::evaluate(Ratio::new(400.0), Ratio::new(0.0));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 23 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_ap_cold_long::evaluate;
    assert_eq!(evaluate(11.0476945, 1.796844).map(f64::to_bits), Ok(0x40217ed703b2671b), "at (11.0476945, 1.796844)");
    assert_eq!(evaluate(19.885850100000003, 3.2343192000000003).map(f64::to_bits), Ok(0x402f7de96d0deccb), "at (19.885850100000003, 3.2343192000000003)");
    assert_eq!(evaluate(21.87443511, 3.5577511200000003).map(f64::to_bits), Ok(0x4031520d2f2e0f09), "at (21.87443511, 3.5577511200000003)");
    assert_eq!(evaluate(22.095389, 3.593688).map(f64::to_bits), Ok(0x40317ed703b2671b), "at (22.095389, 3.593688)");
    assert_eq!(evaluate(22.31634289, 3.62962488).map(f64::to_bits), Ok(0x4031aba0d836bf2e), "at (22.31634289, 3.62962488)");
    assert_eq!(evaluate(24.304927900000003, 3.9530568000000006).map(f64::to_bits), Ok(0x40333eb950ddd7d1), "at (24.304927900000003, 3.9530568000000006)");
    assert_eq!(evaluate(44.190778, 7.187376).map(f64::to_bits), Ok(0x40417ed703b2671b), "at (44.190778, 7.187376)");
    assert_eq!(evaluate(10.0, 2.0).map(f64::to_bits), Ok(0x401dc28f5c28f5c2), "at (10.0, 2.0)");
    assert_eq!(evaluate(18.0, 3.6).map(f64::to_bits), Ok(0x402ac8b439581062), "at (18.0, 3.6)");
    assert_eq!(evaluate(19.8, 3.96).map(f64::to_bits), Ok(0x402d765fd8adaba0), "at (19.8, 3.96)");
    assert_eq!(evaluate(20.0, 4.0).map(f64::to_bits), Ok(0x402dc28f5c28f5c2), "at (20.0, 4.0)");
    assert_eq!(evaluate(20.2, 4.04).map(f64::to_bits), Ok(0x402e0ebedfa43fe6), "at (20.2, 4.04)");
    assert_eq!(evaluate(22.0, 4.4).map(f64::to_bits), Ok(0x40305e353f7ced91), "at (22.0, 4.4)");
    assert_eq!(evaluate(40.0, 8.0).map(f64::to_bits), Ok(0x403dc28f5c28f5c2), "at (40.0, 8.0)");
    assert_eq!(evaluate(8.0, 1.796844).map(f64::to_bits), Ok(0x4016ccd733ae025c), "at (8.0, 1.796844)");
    assert_eq!(evaluate(14.4, 3.2343192000000003).map(f64::to_bits), Ok(0x4024852814e96886), "at (14.4, 3.2343192000000003)");
    assert_eq!(evaluate(15.84, 3.5577511200000003).map(f64::to_bits), Ok(0x40269278e3cd8c94), "at (15.84, 3.5577511200000003)");
    assert_eq!(evaluate(16.0, 3.593688).map(f64::to_bits), Ok(0x4026ccd733ae025c), "at (16.0, 3.593688)");
    assert_eq!(evaluate(16.16, 3.62962488).map(f64::to_bits), Ok(0x40270735838e7825), "at (16.16, 3.62962488)");
    assert_eq!(evaluate(17.6, 3.9530568000000006).map(f64::to_bits), Ok(0x4029148652729c32), "at (17.6, 3.9530568000000006)");
    assert_eq!(evaluate(32.0, 7.187376).map(f64::to_bits), Ok(0x4036ccd733ae025c), "at (32.0, 7.187376)");
    assert!(matches!(evaluate(1.0, 1.0), Err(MethodError::Refused(_))), "at (1.0, 1.0)");
    assert!(matches!(evaluate(400.0, 0.0), Err(MethodError::Refused(_))), "at (400.0, 0.0)");
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `this repository's own chain — the Ap centre with its measured spread` and the declared domain 0 … 300.
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
        if let Err(f) = model::evaluate(Ratio::new(22.095389 * scale), Ratio::new(3.593688)) {
            refused.push(format!("central x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(22.095389), Ratio::new(3.593688 * scale)) {
            refused.push(format!("spread x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_ap_cold_long refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 0 … 300 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Ratio::new(22.095389 * scale), Ratio::new(3.593688)) {
            assert!(v.get().is_finite(), "sw_ap_cold_long produced a value that is not a number for Ap_cold_long");
            assert!(v.get() >= 0.0 && v.get() <= 300.0, "sw_ap_cold_long answered {} for Ap_cold_long, outside its declared domain 0 … 300 — the guard did not stop it", v.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(22.095389), Ratio::new(3.593688 * scale)) {
            assert!(v.get().is_finite(), "sw_ap_cold_long produced a value that is not a number for Ap_cold_long");
            assert!(v.get() >= 0.0 && v.get() <= 300.0, "sw_ap_cold_long answered {} for Ap_cold_long, outside its declared domain 0 … 300 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Ratio::new(22.095389), Ratio::new(3.593688));
    let b = model::evaluate(Ratio::new(22.095389), Ratio::new(3.593688));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_ap_cold_long is not deterministic for Ap_cold_long: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_ap_cold_long refused on one call and answered on the other"),
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
        "sw_ap_cold_long: {} of {rows} grid row(s) disagree with the prior implementation `prf_density.m:217 (01_kernel/sw_study/06_density, local function designWindow_)` beyond {TOL} (worst {worst}).\n{}\nThis is a finding about one of the two implementations, not a build failure and not proof this one is wrong — both classes have been found before. Take it to the node owner. Do not widen parity_tolerance and do not edit parity.csv to agree.",
        findings.len(),
        findings.join("\n")
    );
}

