//! The one case, its inputs, and the CSV they travel in.
//!
//! Held against the real tables, so a row that is added, retired or re-ranged
//! moves from one side of an assertion to the other rather than quietly
//! passing. The CSV is the only way a person's values reach a run outside the
//! browser's own fields, so every way a file can be wrong is tried here, and
//! each must be refused by name — never corrected, never dropped.

use vleo_bus::{Case, RunMode};
use vleo_modules::inputs::{
    case_inputs, check_values, csv, csv_with, read_csv, template, template_of, Group, SetAside,
    Upgrade,
};
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

// ---------------------------------------------------------------------------
// a file from an older version of the tool

/// A template no tree has: a file naming it was written for another set of inputs.
const OLD: &str = "#! template 000000000000\n";

#[test]
fn every_file_the_tool_writes_names_its_template_and_reads_back_current() {
    let t = template();
    assert_eq!(t.len(), 12, "the template id is not 12 hex digits: {t}");
    let text = csv(&[]);
    assert!(text.contains(&format!("#! template {t}")), "{text}");
    let r = read_csv(&text);
    assert_eq!(r.template.as_deref(), Some(t.as_str()));
    assert!(!r.outdated && r.upgrade.is_none());
}

#[test]
fn the_template_moves_with_what_can_stop_a_value_applying_and_nothing_else() {
    let all = case_inputs();
    let t = template_of(&all);
    let mut ranged = all.clone();
    ranged[0].hi *= 2.0;
    assert_ne!(
        template_of(&ranged),
        t,
        "a re-ranged input left the template alone"
    );
    let mut unit = all.clone();
    unit[0].unit = "furlong";
    assert_ne!(
        template_of(&unit),
        t,
        "a changed unit left the template alone"
    );
    assert_ne!(
        template_of(&all[1..]),
        t,
        "a removed input left the template alone"
    );
    let mut default = all.clone();
    default[0].default = default[0].lo;
    default[0].group = Group::Condition;
    assert_eq!(
        template_of(&default),
        t,
        "a moved default or group changed the template, so every saved case would upgrade for nothing"
    );
}

#[test]
fn an_old_file_is_carried_over_and_what_cannot_be_carried_is_set_aside_by_name() {
    let i = an_input();
    let v = (i.default + i.hi) / 2.0;
    let computed = NODES
        .iter()
        .find(|n| n.kind == vleo_core::graph::Kind::Computed)
        .unwrap()
        .id;
    let text = format!(
        "{OLD}id,value,unit\n{},{},{}\ngone_since,4,-\n{computed},1,-\nblank_and_gone,,-\n",
        i.id,
        i.shown(v),
        i.unit
    );
    let r = read_csv(&text);
    assert!(r.ok(), "an old file was refused: {:?}", r.refused);
    assert!(r.outdated);
    assert_eq!(r.set, vec![(i.id.to_string(), i.shown(v) * i.factor)]);
    let u = r.upgrade.expect("no record of the upgrade");
    assert_eq!(u.from, "000000000000");
    let aside: Vec<&str> = u.set_aside.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        aside,
        vec!["gone_since", computed],
        "set aside the wrong rows; a blank value asked for nothing and loses nothing"
    );
    assert_eq!(
        u.set_aside[0].value, "4",
        "the value set aside was not kept to type back"
    );
    assert!(u.set_aside[1].why.contains("computed"));
    // Every input the old file did not name is new, and runs at its default.
    assert_eq!(u.new.len(), case_inputs().len() - 1);
    assert!(!u.new.iter().any(|n| n == i.id));
}

#[test]
fn an_old_value_outside_a_changed_range_is_set_aside_and_a_mistake_is_still_refused() {
    let i = an_input();
    let far = i.shown(i.hi) * 10.0 + 1.0;
    let r = read_csv(&format!("{OLD}id,value\n{},{far}\n", i.id));
    assert!(r.ok(), "{:?}", r.refused);
    let u = r.upgrade.unwrap();
    assert!(
        u.set_aside[0].why.contains("above") && u.set_aside[0].why.contains("range has changed"),
        "{:?}",
        u.set_aside
    );
    // Not a number, and twice, are mistakes in the file whatever tool wrote it.
    let r = read_csv(&format!("{OLD}id,value\n{},twelve\n", i.id));
    assert!(
        !r.ok(),
        "a value that is not a number was set aside instead of refused"
    );
    // And without an older template line, an unknown name is refused, not set
    // aside: a hand-written file with a typo must not lose the typo quietly.
    assert!(!read_csv("id,value\ngone_since,4\n").ok());
}

#[test]
fn an_old_file_in_another_unit_of_the_same_quantity_is_converted_never_between_ratios() {
    let km = case_inputs()
        .into_iter()
        .find(|i| i.unit == "km")
        .expect("no input in km");
    let si = (km.default + km.hi) / 2.0;
    // The tool once kept this row in metres: carried over, converted.
    let r = read_csv(&format!("{OLD}id,value,unit\n{},{si},m\n", km.id));
    assert_eq!(r.set, vec![(km.id.to_string(), si)], "{:?}", r.upgrade);
    // The same file read strictly is refused: the unit is not the row's.
    assert!(!read_csv(&format!("id,value,unit\n{},{si},m\n", km.id)).ok());
    // A ratio and a decibel share a dimension and are not interchangeable.
    let ratio = case_inputs().into_iter().find(|i| i.unit == "-").unwrap();
    let r = read_csv(&format!(
        "{OLD}id,value,unit\n{},{},dB\n",
        ratio.id,
        ratio.shown(ratio.default)
    ));
    assert!(r.set.is_empty(), "a decibel was read as a ratio");
    assert!(r.upgrade.unwrap().set_aside[0].why.contains("dB"));
}

#[test]
fn the_record_of_an_upgrade_survives_being_written_and_read() {
    let i = an_input();
    let note = Upgrade {
        from: "0123456789ab".into(),
        new: vec![i.id.to_string()],
        set_aside: vec![SetAside {
            id: "gone_since".into(),
            value: "4.5".into(),
            unit: "km".into(),
            why: "is not a row in this tree, \"retired\", since".into(),
        }],
        backup: Some("/tmp/inputs.before-0123456789ab.csv".into()),
    };
    let r = read_csv(&csv_with(&[(i.id.to_string(), i.default)], Some(&note)));
    assert!(r.ok() && !r.outdated, "{:?}", r.refused);
    assert_eq!(r.upgrade, Some(note));
}

#[cfg(feature = "std")]
mod on_disk {
    use super::*;
    use vleo_modules::inputs::saved::load;

    fn scratch(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!(
            "vleo-case-{}-{}-{name}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&d).unwrap();
        d.join("inputs.csv")
    }

    #[test]
    fn no_saved_case_is_every_default() {
        let s = load(&scratch("none"));
        assert!(!s.stored && s.reading.set.is_empty() && s.reading.upgrade.is_none());
    }

    #[test]
    fn a_saved_case_from_an_older_tool_is_kept_aside_and_carried_over_once() {
        let i = an_input();
        let v = (i.default + i.hi) / 2.0;
        let path = scratch("old");
        let old = format!("{OLD}id,value\n{},{}\ngone_since,4\n", i.id, i.shown(v));
        std::fs::write(&path, &old).unwrap();

        let s = load(&path);
        assert!(s.stored && s.error.is_none(), "{:?}", s.error);
        assert_eq!(
            s.reading.set,
            vec![(i.id.to_string(), i.shown(v) * i.factor)]
        );
        let u = s.reading.upgrade.clone().expect("no record of the upgrade");
        assert_eq!(u.set_aside[0].id, "gone_since");
        let backup = std::path::PathBuf::from(u.backup.expect("no copy kept"));
        assert_eq!(
            std::fs::read_to_string(&backup).unwrap(),
            old,
            "the copy is not the file as it was"
        );
        let now = std::fs::read_to_string(&path).unwrap();
        assert!(
            now.contains(&format!("#! template {}", template())),
            "not rewritten: {now}"
        );
        assert!(
            now.contains("#! set-aside gone_since,4"),
            "the set-aside value is not in the file"
        );

        // Read again: already current, so no second copy — and the record
        // stays until the case is saved again.
        let again = load(&path);
        assert_eq!(again.reading.set, s.reading.set);
        assert!(again.reading.upgrade.is_some());
        let copies = std::fs::read_dir(path.parent().unwrap()).unwrap().count();
        assert_eq!(copies, 2, "a current case was copied aside again");

        std::fs::write(&path, csv(&again.reading.set)).unwrap();
        assert!(
            load(&path).reading.upgrade.is_none(),
            "saving did not clear the record"
        );
    }

    #[test]
    fn a_saved_case_with_a_row_that_no_longer_applies_still_runs_whole() {
        // Current template, one row edited by hand into nonsense: the rest
        // applies, the bad row is set aside by name, and the file as it was is
        // kept. A run on the case never leaves a value out without a record.
        let i = an_input();
        let path = scratch("bad");
        let text = format!("#! template {}\nid,value\n{},twelve\n", template(), i.id);
        std::fs::write(&path, &text).unwrap();
        let s = load(&path);
        assert!(s.reading.ok(), "{:?}", s.reading.refused);
        let u = s.reading.upgrade.expect("the bad row was not recorded");
        assert!(u.set_aside[0].why.contains("not a number"));
        assert!(std::path::Path::new(u.backup.as_ref().unwrap()).exists());
    }
}
