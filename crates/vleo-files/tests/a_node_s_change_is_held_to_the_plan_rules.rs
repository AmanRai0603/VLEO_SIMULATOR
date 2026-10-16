//! A change to a node, held to the plan rules before anything is written
//! (`vleo_sheet::template`): what today's design holds each node of a group's
//! release from before 1.0 to (`vleo_files::intake`).
//!
//! A change arrives as a `Form`: the node as it was when the change began,
//! what the person made of it, who they are, how an assistant helped, and why
//! the node is changing. Here each is built from the node as the design holds
//! it and edited directly, as intake builds one from a release. What can go
//! wrong is tried against the design, read from `design/`: a change to what
//! the design changed too, a relation an assistant supplied, a numbered step
//! removed from the middle, a change to what a node computes with no reason,
//! a risk moved that nobody registered, a new node in the wrong place.
//!
//! These were the node form's tests, before the form was retired (F3-f).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use vleo_sheet::files::Files;
use vleo_sheet::form;
use vleo_sheet::template::{self, Content, Derisk, Form, Known, NewNode, Verdict};
use vleo_sheet::Tree;

/// The repository, whose `design/` the tree is read from. Planning a change
/// writes nothing, so every test reads the design itself.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The design, read from `design/` once for this test binary: nothing here
/// changes it.
fn design() -> &'static Tree {
    static DESIGN: std::sync::OnceLock<Tree> = std::sync::OnceLock::new();
    DESIGN.get_or_init(|| vleo_files::convert::open(&root()).unwrap().0)
}

/// What a change would do, planned against the design.
fn plan(f: Form) -> Result<template::Plan, vleo_sheet::Error> {
    template::plan_form_against(design(), f)
}

/// A published computed row with a note, four assumptions and a relation.
const ROW: &str = "sw_ap_design";

/// A change to `id` that changes nothing yet: the node as it is, on both
/// sides, made from the design as it is now.
fn form_for(id: &str) -> Form {
    let sh = &design().sheets[id];
    let now = template::content(sh);
    Form {
        node: id.to_string(),
        new: None,
        base: form::file_hash(&sh.text),
        ai: "none".into(),
        original: now.clone(),
        filled: now,
        ..Form::default()
    }
}

fn set(c: &mut Content, field: &str, v: &str) {
    c.fields.insert(field.into(), v.into());
}

/// Say why the node is changing, as a person would.
fn with_record(mut f: Form) -> Form {
    f.derisk = Derisk {
        believed: "that the old relation held across the storm scale".into(),
        tested: "compared against the NOAA scale table, docs/MATLAB_PORT_PLAN.md".into(),
        learned: "it did not hold above G3".into(),
        cost: "half a day".into(),
        changed: "the relation, which now holds across the scale".into(),
        risks: String::new(),
        rests_on: "the NOAA G-scale thresholds".into(),
        breaks_if: "NOAA revises the scale".into(),
    };
    f
}

fn verdict_of(p: &template::Plan, what: &str) -> Verdict {
    p.items
        .iter()
        .find(|i| i.what == what)
        .unwrap_or_else(|| panic!("no item '{what}' in {:?}", p.items))
        .verdict
        .clone()
}

#[test]
fn a_change_that_changes_nothing_changes_nothing_for_any_node() {
    for sh in design().sheets.values() {
        let p = plan(form_for(&sh.id)).unwrap_or_else(|e| panic!("{}: {e}", sh.id));
        assert!(p.base_current, "{}", sh.id);
        assert!(p.items.is_empty(), "{}: {:?}", sh.id, p.items);
        assert!(p.text.is_none(), "{}", sh.id);
    }
}

#[test]
fn a_change_is_applied_unless_the_design_changed_it_too() {
    let mut f = form_for(ROW);
    set(&mut f.filled, "note", "A note from the payload team.");
    let p = plan(f.clone()).unwrap();
    assert_eq!(verdict_of(&p, "note"), Verdict::Apply);
    assert!(p
        .text
        .as_deref()
        .unwrap()
        .contains("A note from the payload team."));

    // The change began when the note said something else, and the design has
    // moved it since: the change must not overwrite that.
    let mut stale = f.clone();
    set(&mut stale.original, "note", "An older note.");
    let p = plan(stale.clone()).unwrap();
    assert!(
        matches!(verdict_of(&p, "note"), Verdict::Conflict(_)),
        "{:?}",
        p.items
    );
    assert!(p.text.is_none(), "a conflict was written anyway");

    // And a change asking for what the node already says has nothing to do.
    let now = form::value(&design().sheets[ROW], "note");
    set(&mut stale.filled, "note", &now);
    assert_eq!(verdict_of(&plan(stale).unwrap(), "note"), Verdict::Already);
}

#[test]
fn a_relation_an_assistant_supplied_is_refused_and_a_persons_is_applied() {
    let mut changed = with_record(form_for(ROW));
    set(
        &mut changed.filled,
        "expression",
        "Ap_design(G) = a different relation",
    );
    let p = plan(changed.clone()).unwrap();
    assert_eq!(verdict_of(&p, "expression"), Verdict::Apply);
    assert!(
        p.relation,
        "a changed relation must carry a name when applied"
    );

    let mut assisted = changed;
    assisted.ai = "relation".into();
    let p = plan(assisted).unwrap();
    assert!(
        matches!(verdict_of(&p, "expression"), Verdict::Refused(ref w) if w.contains("mathematics")),
        "{:?}",
        p.items
    );
    assert!(p.text.is_none());
}

#[test]
fn how_an_assistant_helped_is_said_or_nothing_is_planned() {
    let mut f = form_for(ROW);
    f.ai = "a little".into();
    let e = plan(f.clone()).unwrap_err();
    assert!(e.message().contains("\"a little\" is not one of"), "{e}");
    // Each answer intake gives is taken.
    for ai in template::AI_HELP {
        f.ai = ai.to_string();
        plan(f.clone()).unwrap_or_else(|e| panic!("{ai}: {e}"));
    }
}

#[test]
fn a_change_never_adds_a_node() {
    let mut f = form_for(ROW);
    f.node = "no_such_row".into();
    assert!(plan(f).unwrap_err().message().contains("never adds a node"));
}

#[test]
fn a_numbered_step_removed_from_the_middle_is_refused_and_the_last_is_not() {
    let tree = design();
    let sh = tree
        .ordered()
        .into_iter()
        .find(|s| s.steps.len() >= 2 && s.state == "published")
        .expect("no published row with two algorithm steps");
    let drop = |at: usize| {
        let mut f = with_record(form_for(&sh.id));
        f.filled.arrays.get_mut("algorithm").unwrap().remove(at);
        f
    };
    let middle = plan(drop(0)).unwrap();
    assert!(
        matches!(verdict_of(&middle, "algorithm"), Verdict::Refused(ref w) if w.contains("middle")),
        "{}: {:?}",
        sh.id,
        middle.items
    );
    let last = plan(drop(sh.steps.len() - 1)).unwrap();
    let removed = format!("algorithm {} · removed", sh.steps.len());
    assert_eq!(
        verdict_of(&last, &removed),
        Verdict::Apply,
        "{:?}",
        last.items
    );
}

fn known(
    label: &str,
    inputs: &str,
    expected: &str,
    tolerance: &str,
    provenance: &str,
    source: &str,
) -> Known {
    Known {
        label: label.into(),
        inputs: inputs.into(),
        expected: expected.into(),
        tolerance: tolerance.into(),
        provenance: provenance.into(),
        source: source.into(),
    }
}

#[test]
fn known_values_are_a_request_and_never_touch_the_node() {
    let mut f = form_for(ROW);
    f.known = vec![known(
        "G2",
        "sw_storm_design_level = 2",
        "80",
        "",
        "published-source",
        "NOAA G scale",
    )];
    let p = plan(f).unwrap();
    assert!(p.items.is_empty() && p.text.is_none(), "{:?}", p.items);
    let req = template::fixture_request(&p.form);
    assert!(
        req.contains("[[fixture]]") && req.contains("expect = 80"),
        "{req}"
    );
    let parsed: toml::Value = req.parse().expect("the request is not TOML");
    assert!(parsed.get("fixture").is_some());
}

// ---------------------------------------------------------------------------
// a node the design does not have yet, and the interfaces a change declares

/// Nothing yet: every field asked and empty, every block empty.
fn blank() -> Content {
    let mut c = Content::default();
    for f in form::FIELDS.iter().filter(|f| f.asked) {
        c.fields.insert(f.field.to_string(), String::new());
    }
    for a in form::ARRAYS {
        c.arrays.insert(a.name.to_string(), Vec::new());
    }
    c.view = Some(("number".into(), String::new(), "0".into()));
    c
}

fn new_form(id: &str, parent: &str, kind: &str, input: Option<(&str, &str)>) -> Form {
    let mut filled = blank();
    set(&mut filled, "label", "A new row");
    set(&mut filled, "question", "What does the new row answer?");
    if let Some((var, ty)) = input {
        filled.arrays.insert(
            "input".into(),
            vec![BTreeMap::from([
                ("binding".to_string(), "x".to_string()),
                ("var".to_string(), var.to_string()),
                ("type".to_string(), ty.to_string()),
            ])],
        );
    }
    Form {
        new: Some(NewNode {
            id: id.into(),
            parent: parent.into(),
            kind: kind.into(),
        }),
        ai: "none".into(),
        original: blank(),
        filled,
        ..Form::default()
    }
}

#[test]
fn a_new_node_is_checked_for_its_place_and_its_interfaces_before_anything_is_built() {
    let tree = design();
    let producer = tree.sheets.get(ROW).unwrap();
    let parent = producer.parent.clone();

    let good = plan(new_form(
        "a_brand_new_row",
        &parent,
        "computed",
        Some((ROW, &producer.ty)),
    ))
    .unwrap();
    assert_eq!(good.new.as_ref().unwrap().id, "a_brand_new_row");
    for w in [
        "new · id",
        "new · parent",
        "new · kind",
        "label",
        "input 1 · added",
    ] {
        assert_eq!(
            verdict_of(&good, w),
            Verdict::Apply,
            "{w}: {:?}",
            good.items
        );
    }
    assert!(
        good.interfaces.len() == 1 && good.interfaces[0].ok(),
        "{:?}",
        good.interfaces
    );
    assert!(
        good.text.is_none(),
        "a new node's plan must not claim a sheet text"
    );
    assert!(
        good.open.iter().any(|o| o.contains("expression")),
        "{:?}",
        good.open
    );

    // Each way the placement can be wrong is refused by name.
    let taken = plan(new_form(ROW, &parent, "computed", None)).unwrap();
    assert!(
        matches!(verdict_of(&taken, "new · id"), Verdict::Refused(ref w) if w.contains("already"))
    );
    let bad_id = plan(new_form("Not An Id", &parent, "computed", None)).unwrap();
    assert!(matches!(
        verdict_of(&bad_id, "new · id"),
        Verdict::Refused(_)
    ));
    let nowhere = plan(new_form("x_row", "no_such_group", "computed", None)).unwrap();
    assert!(matches!(
        verdict_of(&nowhere, "new · parent"),
        Verdict::Refused(_)
    ));
    let odd = plan(new_form("x_row", &parent, "banana", None)).unwrap();
    assert!(matches!(
        verdict_of(&odd, "new · kind"),
        Verdict::Refused(_)
    ));

    // An input that names no row, or a row of another quantity, does not connect.
    let ghost = plan(new_form(
        "x_row",
        &parent,
        "computed",
        Some(("no_such_row", "Ratio")),
    ))
    .unwrap();
    assert!(!ghost.interfaces[0].ok() && ghost.interfaces[0].why.contains("no row"));
    assert!(matches!(
        verdict_of(&ghost, "input 1 · added"),
        Verdict::Refused(_)
    ));
    let wrong = if producer.ty == "Length" {
        "Time"
    } else {
        "Length"
    };
    let mismatch = plan(new_form("x_row", &parent, "computed", Some((ROW, wrong)))).unwrap();
    assert!(
        mismatch.interfaces[0].why.contains(wrong),
        "{:?}",
        mismatch.interfaces
    );
}

#[test]
fn an_input_changed_to_one_that_does_not_connect_is_refused_and_the_rest_still_applies() {
    let mut f = form_for(ROW);
    set(&mut f.filled, "note", "Still applies.");
    f.filled.arrays.get_mut("input").unwrap()[0].insert("var".into(), "no_such_row".into());
    let p = plan(f).unwrap();
    assert!(
        matches!(verdict_of(&p, "input"), Verdict::Refused(_)),
        "{:?}",
        p.items
    );
    assert_eq!(verdict_of(&p, "note"), Verdict::Apply);
    assert!(p.interfaces.iter().any(|i| !i.ok()));
    let text = p.text.expect("the note should still apply");
    assert!(text.contains("Still applies.") && !text.contains("no_such_row"));
}

#[test]
fn a_change_to_what_a_node_computes_says_why_or_only_its_wording_goes_in() {
    // Whatever versions the row already records; the next one follows them.
    let had = design().sheets[ROW].versions.len();
    // A new bound and a new note, with nothing said about why.
    let mut f = form_for(ROW);
    set(&mut f.filled, "upper", "450.0");
    set(&mut f.filled, "note", "A reworded note.");
    let p = plan(f.clone()).unwrap();
    assert_eq!(
        verdict_of(&p, "note"),
        Verdict::Apply,
        "wording is never withheld"
    );
    assert!(
        matches!(verdict_of(&p, "upper"), Verdict::Refused(ref w) if w.contains("withheld")),
        "{:?}",
        p.items
    );
    assert!(
        matches!(verdict_of(&p, "de-risking"), Verdict::Refused(ref w) if w.contains("output") && w.contains("believed")),
        "{:?}",
        p.items
    );
    let text = p.text.clone().unwrap();
    assert!(text.contains("A reworded note.") && !text.contains("upper = 450.0"));
    assert_eq!(
        text.matches("[[version]]").count(),
        had,
        "a version was recorded without its reason"
    );
    assert_eq!(p.about, vec!["output"]);

    // The same changes with the reason: both go in, and the next version is
    // recorded with what moved, the relation as it now stands, and `next` for a
    // release.
    let p = plan(with_record(f)).unwrap();
    assert_eq!(verdict_of(&p, "upper"), Verdict::Apply, "{:?}", p.items);
    assert_eq!(p.version, Some(had as u32 + 1));
    let text = p.text.unwrap();
    let sheet: toml::Value = text
        .parse()
        .expect("the sheet with its version still reads");
    let v = &sheet["version"][had];
    assert_eq!(v["n"].as_integer(), Some(had as i64 + 1));
    assert_eq!(v["release"].as_str(), Some("next"));
    assert_eq!(v["about"][0].as_str(), Some("output"));
    assert_eq!(v["learned"].as_str(), Some("it did not hold above G3"));
    assert!(v["relation"].as_str().unwrap().contains("Ap_design"));
}

#[test]
fn a_risk_move_must_name_a_registered_risk() {
    let mut f = with_record(form_for(ROW));
    set(&mut f.filled, "upper", "450.0");
    f.derisk.risks = "R-9999 closed".into();
    let p = plan(f).unwrap();
    assert!(
        matches!(verdict_of(&p, "de-risking"), Verdict::Refused(ref w) if w.contains("not registered")),
        "{:?}",
        p.items
    );
    assert!(matches!(verdict_of(&p, "upper"), Verdict::Refused(_)));
}

#[test]
fn a_row_with_no_plain_words_yet_takes_them_from_its_change() {
    // A sheet with no [explain] table yet: the first explanatory answer makes
    // one, beside the question. Every published row has one now, so the table
    // is taken off a real sheet's text to see it made again.
    let tree = design();
    let sh = tree.sheets.get(ROW).unwrap();
    let (served, _) = vleo_files::convert::serve(&root()).unwrap();
    let full = served.read_to_string(&sh.dir.join("node.toml")).unwrap();
    let mut bare = String::new();
    let mut skip = false;
    for line in full.lines() {
        let l = line.trim_start();
        if l.starts_with('[') {
            skip = l == "[explain]";
        }
        if !skip && !l.starts_with("# SAID SIMPLY") && !l.starts_with("# sheet hash.") {
            bare.push_str(line);
            bare.push('\n');
        }
    }
    assert!(!bare.contains("[explain]"));
    let text = vleo_sheet::form::set(&bare, "explain_simply", "What the row works out.").unwrap();
    let v: toml::Value = text.parse().expect("the sheet no longer reads");
    assert_eq!(
        v["explain"]["simply"].as_str(),
        Some("What the row works out.")
    );
    // The design's files lay a sheet's tables out in their own order, so
    // beside the question is the next table after it, whichever that was.
    let question = text.find("[question]").unwrap();
    assert!(
        text[question..].find("\n[") == text[question..].find("\n[explain]"),
        "the plain words are not beside the question"
    );

    // Through a change: it applies, records no version, and names whose words
    // these are, taken from the change's author.
    let mut f = form_for(ROW);
    set(
        &mut f.filled,
        "explain_simply",
        "What the row works out, in plain words.",
    );
    set(
        &mut f.filled,
        "explain_breaks",
        "Where that stops being true.",
    );
    f.name = "R. Kumar".into();
    f.team = "Solar".into();
    let p = plan(f).unwrap();
    assert_eq!(
        verdict_of(&p, "explain_simply"),
        Verdict::Apply,
        "{:?}",
        p.items
    );
    assert_eq!(verdict_of(&p, "explain_breaks"), Verdict::Apply);
    assert!(p.version.is_none(), "wording recorded a version");
    let v: toml::Value = p.text.unwrap().parse().unwrap();
    assert_eq!(
        v["explain"]["simply"].as_str(),
        Some("What the row works out, in plain words.")
    );
    assert_eq!(v["explain"]["by"].as_str(), Some("R. Kumar (Solar)"));
}

#[test]
fn a_source_not_listed_yet_is_said_at_the_check_and_does_not_stop_it() {
    let tree = design();
    let listed = tree.sources.keys().next().unwrap().clone();
    for (cited, open) in [
        ("the_book_nobody_listed_2031", true),
        (listed.as_str(), false),
    ] {
        let mut f = with_record(form_for(ROW));
        set(&mut f.filled, "source", cited);
        let p = plan(f).unwrap();
        assert_eq!(verdict_of(&p, "source"), Verdict::Apply, "{:?}", p.items);
        assert_eq!(
            p.open
                .iter()
                .any(|o| o.contains(cited) && o.contains("sources/sources.toml")),
            open,
            "{cited}: {:?}",
            p.open
        );
    }
}

#[test]
fn a_known_value_is_requested_in_si_by_binding_as_fixtures_hold_it() {
    // A change gives the answer in the node's unit and the inputs as a person
    // writes them; fixtures.toml holds SI, by binding. Pasted as typed, 70.2
    // degrees was 70.2 radians.
    let mut f = new_form(
        "orbit_new_angle",
        "l3_x_envorbit",
        "computed",
        Some(("orbit_radius", "Length")),
    );
    set(&mut f.filled, "unit", "Degree");
    f.known = vec![known(
        "400 km",
        "orbit_radius = 6778.137 km",
        "70.2179",
        "1e-4",
        "independent-derivation",
        "larson_wertz",
    )];
    let p = plan(f.clone()).unwrap();
    let req = template::fixture_request(&p.form);
    let v: toml::Value = req.parse().expect("the request is not TOML");
    let fx = &v["fixture"][0];
    let expect = fx["expect"].as_float().unwrap();
    assert!((expect - 70.2179f64.to_radians()).abs() < 1e-9, "{req}");
    assert_eq!(fx["inputs"]["x"].as_float(), Some(6_778_137.0), "{req}");
    // Unreadable inputs are left for the person, never half-converted.
    f.known[0].inputs = "orbit_radius = 6778".into();
    let req = template::fixture_request(&plan(f).unwrap().form);
    let v: toml::Value = req.parse().unwrap();
    assert!(v["fixture"][0].get("inputs").is_none(), "{req}");
}

#[test]
fn a_method_of_the_changes_own_is_no_longer_a_transcription() {
    // A transcribed method names what it was copied from and who read the
    // copy against it. When a person's own method replaces it, neither
    // describes the row any more: the plan clears both, and names the person.
    // Any row still a transcription will do: which ones are changes as
    // groups send their own methods.
    let tree = design();
    let (id, copied) = tree
        .sheets
        .values()
        .find(|s| !s.method.transcribed_from.is_empty())
        .map(|s| (s.id.clone(), s.method.text.clone()))
        .expect("a row whose method is a transcription");
    // The person's own method: the same relation, in their words.
    let own = format!(
        "# The relation as I read it in its source.\n{}",
        copied.trim_end()
    );
    let mut f = with_record(form_for(&id));
    set(&mut f.filled, "method_text", &own);
    f.name = "R. Kumar".into();
    let p = plan(f).unwrap();
    assert_eq!(
        verdict_of(&p, "method_text"),
        Verdict::Apply,
        "{:?}",
        p.items
    );
    let v: toml::Value = p.text.unwrap().parse().unwrap();
    let m = &v["method"];
    assert!(m["by"].as_str().unwrap().starts_with("R. Kumar"), "{m}");
    assert_eq!(m["transcribed_from"].as_str(), Some(""), "{m}");
    assert_eq!(m["checked_by"].as_str(), Some(""), "{m}");
}
