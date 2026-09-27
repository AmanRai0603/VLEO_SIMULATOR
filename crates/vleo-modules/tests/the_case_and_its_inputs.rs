//! The one case, its inputs, and the CSV they travel in.
//!
//! Held against the real tables, so a row that is added, retired or re-ranged
//! moves from one side of an assertion to the other rather than quietly
//! passing. The CSV is the only way a person's values reach a run outside the
//! browser's own fields, so every way a file can be wrong is tried here, and
//! each must be refused by name — never corrected, never dropped.

use vleo_bus::{Case, RunMode};
use vleo_modules::inputs::{case_inputs, check_values, csv, read_csv, Group};
use vleo_modules::{case_refusal, Scratch, Vleo, NODES, VARS};

fn value(case: &Case) -> Option<f64> {
    let mut scratch = Scratch::new();
    let r = vleo_modules::evaluate(case, &mut scratch).ok()?;
    r.values
        .iter()
        .find(|v| v.id == case.target)
        .map(|v| v.value)
}

fn run(target: &str, supply: Vec<(String, f64)>) -> Case {
    Case {
        target: target.into(),
        mode: RunMode::Branch,
        supply,
        ..Default::default()
    }
}

/// An input with a unit and room either side of its default, from the tree.
fn an_input() -> vleo_modules::inputs::Input {
    case_inputs()
        .into_iter()
        .find(|i| !i.unit.is_empty() && i.default > i.lo && i.default < i.hi)
        .expect("no input with a unit and room to move")
}

#[test]
fn there_is_a_case_and_an_unknown_one_is_refused() {
    assert!(Vleo::default_case().is_some(), "cases/ holds no case");
    let bad = Case {
        base: "c1".into(),
        target: "orbit_altitude".into(),
        ..Default::default()
    };
    assert!(case_refusal(&bad).is_some(), "an unknown case was accepted");
    assert!(
        value(&bad).is_none(),
        "an unknown case ran the declared design instead"
    );
}

#[test]
fn every_input_is_a_declared_published_row_in_its_group() {
    let case = Vleo::default_case().unwrap();
    let all = case_inputs();
    assert!(!all.is_empty());
    let mut seen_condition = false;
    for i in &all {
        let def = &NODES[VARS[i.var as usize].producer as usize];
        assert_eq!(
            def.kind,
            vleo_core::graph::Kind::Declared,
            "{} is not declared",
            i.id
        );
        assert_eq!(
            def.state,
            vleo_core::graph::State::Published,
            "{} is not published",
            i.id
        );
        assert!(i.hi > i.lo, "{} has no room to move", i.id);
        assert!(
            i.default >= i.lo && i.default <= i.hi,
            "{}'s default is outside its range",
            i.id
        );
        let listed = case.conditions.contains(&i.var);
        assert_eq!(
            i.group == Group::Condition,
            listed,
            "{} is in the wrong group",
            i.id
        );
        // Customers first, then conditions: the order a person reads the file in.
        if i.group == Group::Condition {
            seen_condition = true;
        } else {
            assert!(
                !seen_condition,
                "{} (customer) comes after a condition",
                i.id
            );
        }
    }
    assert!(seen_condition, "no input is in the Condition group");
}

#[test]
fn the_template_reads_back_as_every_default() {
    let r = read_csv(&csv(&[]));
    assert!(r.ok(), "the template refuses itself: {:?}", r.refused);
    assert!(r.set.is_empty());
    assert_eq!(r.changed, 0);
    assert_eq!(r.defaulted, case_inputs().len());
}

#[test]
fn a_value_survives_the_round_trip_exactly() {
    let i = an_input();
    let v = (i.default + i.hi) / 2.0;
    let r = read_csv(&csv(&[(i.id.to_string(), v)]));
    assert!(r.ok(), "{:?}", r.refused);
    assert_eq!(r.changed, 1);
    let (_, back) = r.set.iter().find(|(k, _)| k == i.id).unwrap();
    // Written in the row's own unit and read back to SI: the same f64, or a
    // download uploaded unchanged would move the design.
    assert!(
        (back - v).abs() <= v.abs() * 1e-12,
        "{} came back as {back}, was {v}",
        i.id
    );
}

#[test]
fn columns_in_any_order_comments_quotes_and_a_blank_value_are_read() {
    let i = an_input();
    let text = format!(
        "\u{feff}# a note\nvalue,unit,name,id\n{},{},\"a label, with a comma\",{}\n,,,{}\n",
        i.shown(i.hi),
        i.unit,
        i.id,
        case_inputs().last().unwrap().id
    );
    let r = read_csv(&text);
    assert!(r.ok(), "{:?}", r.refused);
    assert_eq!(r.set.len(), 1, "a blank value is the default, not a value");
}

#[test]
fn every_kind_of_bad_row_is_refused_by_name_and_nothing_is_kept() {
    let i = an_input();
    let computed = NODES
        .iter()
        .find(|n| n.kind == vleo_core::graph::Kind::Computed)
        .unwrap()
        .id;
    let rows = [
        ("no_such_input", "1".to_string(), String::new(), "not a row"),
        (computed, "1".to_string(), String::new(), "computed"),
        (
            i.id,
            format!("{}", i.shown(i.hi) * 10.0 + 1.0),
            String::new(),
            "above",
        ),
        (i.id, "twelve".to_string(), String::new(), "not a number"),
        (
            i.id,
            format!("{}", i.shown(i.default)),
            "furlong".to_string(),
            "furlong",
        ),
    ];
    for (id, v, unit, want) in rows {
        let text = format!("id,value,unit\n{id},{v},{unit}\n");
        let r = read_csv(&text);
        assert!(!r.ok(), "{id} = {v} {unit} was accepted");
        let (_, rid, why) = &r.refused[0];
        assert_eq!(rid, id);
        assert!(why.contains(want), "{id}: '{why}' does not say '{want}'");
    }
    // Twice is refused, and one good row beside a bad one does not survive:
    // the reading says it is not ok, and the caller saves nothing.
    let text = format!("id,value\n{0},{1}\n{0},{1}\n", i.id, i.shown(i.default));
    let r = read_csv(&text);
    assert!(
        !r.ok() && r.refused[0].2.contains("twice"),
        "{:?}",
        r.refused
    );
    assert!(
        !read_csv("no header here\n1,2\n").ok(),
        "a file without the columns was accepted"
    );
    assert!(
        !read_csv("# only comments\n").ok(),
        "an empty file was accepted"
    );
}

#[test]
fn stored_values_are_rechecked_against_the_tree_as_it_is() {
    let i = an_input();
    let r = check_values(&[
        (i.id.into(), i.hi * 2.0 + 1.0),
        ("retired_or_renamed".into(), 1.0),
    ]);
    assert_eq!(r.refused.len(), 2, "{:?}", r.refused);
}

#[test]
fn an_uploaded_value_is_what_the_run_uses() {
    // Any input with room to move, run as its own target: what the file says
    // is what comes back. Chosen from the tables rather than named, and with
    // no reference data needed, so it holds as rows are added and retired.
    let i = an_input();
    let v = (i.default + i.hi) / 2.0;
    let text = format!("id,value\n{},{}\n", i.id, i.shown(v));
    let r = read_csv(&text);
    assert!(r.ok(), "{:?}", r.refused);
    let got = value(&run(i.id, r.set)).expect("the input did not run");
    assert!(
        (got - v).abs() <= v.abs() * 1e-12,
        "{} ran at {got}, the file said {v}",
        i.id
    );
    assert_eq!(
        value(&run(i.id, Vec::new())),
        Some(i.default),
        "no file is not the default"
    );
}
