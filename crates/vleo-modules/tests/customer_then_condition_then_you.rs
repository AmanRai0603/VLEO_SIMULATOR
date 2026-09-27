//! Who a run is for, what it is asked to survive, and who has the last word.
//!
//! A run applies each row's declared value, then the customer's, then the
//! condition's, then whatever a person typed. These hold that order against the
//! real tables — the files in `cases/` as they are — so renaming a customer or
//! re-pointing a condition moves a value from one side of an assertion to the
//! other rather than quietly passing.
//!
//! And they hold the refusals. The engine used to run the bare declared design
//! for a case name it did not know, which is a number with somebody else's
//! name on it: a test here named "c1", a case that never existed, and passed.

use vleo_bus::{Case, RunMode};
use vleo_modules::{case_refusal, Scratch, Vleo, CASES};

fn value(case: &Case) -> Option<f64> {
    let mut scratch = Scratch::new();
    let r = vleo_modules::evaluate(case, &mut scratch).ok()?;
    r.values
        .iter()
        .find(|v| v.id == case.target)
        .map(|v| v.value)
}

fn case(base: &str, condition: &str, target: &str) -> Case {
    Case {
        base: base.into(),
        condition: condition.into(),
        target: target.into(),
        mode: RunMode::Branch,
        ..Default::default()
    }
}

/// A condition that supplies something runnable, with the row and the value.
fn a_condition() -> (&'static str, &'static str, f64) {
    let c = CASES
        .iter()
        .find(|c| c.kind == "condition" && c.unavailable.is_empty() && !c.supply.is_empty())
        .expect("cases/ has no condition that can be applied");
    let (v, val) = c.supply[0];
    (c.id, vleo_modules::VARS[v as usize].id, val)
}

#[test]
fn no_customer_named_is_the_first_customer() {
    let first = Vleo::default_customer().expect("cases/ has no customer");
    let (_, row, _) = a_condition();
    assert_eq!(
        value(&case("", "", row)),
        value(&case(first.id, "", row)),
        "a run naming no customer is not the run of {}",
        first.id
    );
}

#[test]
fn a_condition_is_laid_over_the_customer() {
    let (cond, row, val) = a_condition();
    for c in CASES.iter().filter(|c| c.kind == "customer") {
        assert_eq!(
            value(&case(c.id, cond, row)),
            Some(val),
            "{cond} did not reach {row} for {}",
            c.id
        );
    }
}

#[test]
fn what_a_person_typed_beats_the_condition() {
    let (cond, row, val) = a_condition();
    let v = vleo_modules::Vleo::find(row).unwrap();
    let lim = &vleo_modules::VARS[v as usize].limit;
    // Anything inside the domain that is not the condition's own value.
    let mine = if (lim.lower - val).abs() > f64::EPSILON {
        lim.lower
    } else {
        lim.upper
    };
    let mut c = case("", cond, row);
    c.supply.push((row.to_string(), mine));
    assert_eq!(
        value(&c),
        Some(mine),
        "the condition overwrote what was typed"
    );
}

#[test]
fn a_name_the_tables_do_not_hold_is_refused_not_ignored() {
    let c = case("c1", "", "orbit_altitude");
    assert!(
        case_refusal(&c).is_some(),
        "an unknown customer was accepted"
    );
    assert!(
        value(&c).is_none(),
        "an unknown customer ran the declared design instead of refusing"
    );
    let c = case("", "no_such_condition", "orbit_altitude");
    assert!(
        case_refusal(&c).is_some(),
        "an unknown condition was accepted"
    );
    assert!(value(&c).is_none(), "an unknown condition ran anyway");
}

#[test]
fn a_customer_and_a_condition_are_not_interchangeable() {
    let (cond, row, _) = a_condition();
    let cust = Vleo::default_customer().unwrap().id;
    let as_customer = case(cond, "", row);
    let why = case_refusal(&as_customer).expect("a condition was accepted as a customer");
    assert!(why.contains("not a customer"), "{why}");
    let as_condition = case("", cust, row);
    let why = case_refusal(&as_condition).expect("a customer was accepted as a condition");
    assert!(why.contains("not a condition"), "{why}");
}

#[test]
fn a_condition_that_cannot_be_applied_says_why_and_does_not_run() {
    for c in CASES.iter().filter(|c| !c.unavailable.is_empty()) {
        let k = case("", c.id, "orbit_altitude");
        let why = case_refusal(&k).unwrap_or_default();
        assert!(
            why.contains(c.unavailable),
            "{} is marked unavailable and was not refused with its reason: {why}",
            c.id
        );
        assert!(
            value(&k).is_none(),
            "{} ran although it cannot be applied",
            c.id
        );
    }
}
