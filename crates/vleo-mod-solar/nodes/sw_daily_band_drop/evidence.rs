// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_daily_band_drop`.
//!
//! Every expected value below names a source outside this code. A number
//! produced by the thing being tested proves nothing, so the schema
//! refuses a fixture whose provenance is the implementation.

// No fixtures yet. The gap pass reports this node as unevidenced and
// its validation credibility factor is zero, which governs the whole
// vector — an unvalidated node cannot be quietly relied on.

use super::model;
use vleo_core::units::*;

fn relative_error(got: f64, expected: f64) -> f64 {
    if expected == 0.0 { pmath::abs(got) } else { pmath::abs((got - expected) / expected) }
}

/// The prior implementation's one number, against this row's one number.
///
/// Migrated from `prf_density.m:221 (01_kernel/sw_study/06_density, local function designWindow_)`. A second opinion and never an expected
/// value: an implementation cannot supply its own. A disagreement is
/// a finding about one of the two.
#[test]
fn parity_grid() {
    const GRID: &str = include_str!("parity.csv");
    const TOL: f64 = 0.01;
    let row = GRID
        .lines()
        .filter(|l| !l.trim_start().starts_with('#') && !l.trim().is_empty())
        .nth(1)
        .expect("parity.csv has a header and at least one data row");
    let expected: f64 = row
        .rsplit(',')
        .next()
        .expect("a last column")
        .trim()
        .parse()
        .expect("the last column of the first data row is a number");
    let got = model::evaluate().expect("the declared value").get();
    let err = relative_error(got, expected);
    assert!(
        err <= TOL,
        "sw_daily_band_drop: this row says {got} and the prior implementation `prf_density.m:221 (01_kernel/sw_study/06_density, local function designWindow_)` says {expected} — {err} apart, beyond {TOL}.\nThis is a finding about one of the two implementations, not a build failure and not proof this one is wrong. Take it to the node owner. Do not widen parity_tolerance and do not edit parity.csv to agree."
    );
}

