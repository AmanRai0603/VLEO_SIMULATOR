//! Every gate check, shown to catch the thing it exists to catch.
//!
//! The gate is what enforces the rules in AGENTS.md, and until these tests it
//! had none of its own: a check that stopped firing would have gone on reporting
//! green. Each test takes a real row (or the design itself, read from
//! `design/`), breaks exactly one thing in memory, and asserts that the named
//! check — and only the named check — fails. Nothing here writes to disk.

use std::path::Path;
use vleo_sheet::gate::{gate_node, validate_tree, Check};
use vleo_sheet::{Sheet, Tree};

fn tree() -> Tree {
    vleo_files::convert::open(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .unwrap()
        .0
}

fn failed(checks: &[Check]) -> Vec<&'static str> {
    checks
        .iter()
        .filter(|c| c.failed())
        .map(|c| c.name)
        .collect()
}

/// A published row matching `want`, cloned so it can be broken.
fn row(t: &Tree, want: impl Fn(&Sheet) -> bool) -> Sheet {
    t.sheets
        .values()
        .find(|s| s.state == "published" && want(s))
        .expect("the tree has a row of this shape")
        .clone()
}

/// `sh` breaks `check` and nothing else; the row as it was passes it.
fn breaks(t: &Tree, original: &Sheet, broken: &Sheet, check: &str) {
    assert!(
        !failed(&gate_node(original, t)).contains(&check),
        "{}: {check} fails before anything was broken",
        original.id
    );
    let f = failed(&gate_node(broken, t));
    assert_eq!(
        f,
        vec![check],
        "{}: breaking {check} should fail {check} alone",
        broken.id
    );
}

// ---------------------------------------------------------------------------
// the per-node checks

#[test]
fn schema_refuses_a_blank_required_field() {
    let t = tree();
    let ok = row(&t, |s| !s.is_declared() && !s.question.is_empty());
    let mut bad = ok.clone();
    bad.question.clear();
    breaks(&t, &ok, &bad, "schema");
}

#[test]
fn inputs_refuses_a_computed_row_from_nothing() {
    let t = tree();
    let ok = row(&t, |s| {
        !s.is_declared() && s.inputs.len() == 1 && s.fixtures.is_empty()
    });
    let mut bad = ok.clone();
    bad.inputs.clear();
    let f = failed(&gate_node(&bad, &t));
    assert!(f.contains(&"inputs"), "{f:?}");
}

#[test]
fn declared_value_refuses_a_number_nobody_confirmed() {
    let t = tree();
    let ok = row(&t, |s| s.is_declared() && s.value.is_some());
    let mut bad = ok.clone();
    bad.confirmed_by.clear();
    breaks(&t, &ok, &bad, "declared-value");
}

#[test]
fn publishes_refuses_a_member_with_no_room() {
    let t = tree();
    let ok = row(&t, |s| !s.publishes.is_empty());
    let mut bad = ok.clone();
    bad.publishes[0].lower = bad.publishes[0].upper;
    breaks(&t, &ok, &bad, "publishes");
}

#[test]
fn contract_refuses_an_input_of_the_wrong_quantity() {
    let t = tree();
    let ok = row(&t, |s| {
        !s.is_declared() && !s.inputs.is_empty() && !s.inputs[0].var.contains('.')
    });
    let mut bad = ok.clone();
    bad.inputs[0].ty = if bad.inputs[0].ty == "Mass" {
        "Length"
    } else {
        "Mass"
    }
    .into();
    let f = failed(&gate_node(&bad, &t));
    assert!(f.contains(&"contract"), "{f:?}");
}

#[test]
fn sources_refuses_a_citation_to_nothing() {
    let t = tree();
    let ok = row(&t, |s| !s.is_declared() && s.fixtures.is_empty());
    let mut bad = ok.clone();
    bad.source = "no_such_source_2099".into();
    breaks(&t, &ok, &bad, "sources");
}

#[test]
fn criticality_refuses_a_word_it_does_not_know() {
    let t = tree();
    let ok = row(&t, |_| true);
    let mut bad = ok.clone();
    bad.criticality = "urgent".into();
    breaks(&t, &ok, &bad, "criticality");
}

#[test]
fn sense_refuses_a_requirement_that_does_not_say_which_way() {
    let t = tree();
    let ok = row(&t, |s| s.kind == "required" && !s.sense.is_empty());
    for sense in ["", "<", "=>"] {
        let mut bad = ok.clone();
        bad.sense = sense.into();
        breaks(&t, &ok, &bad, "sense");
    }
}

#[test]
fn provenance_refuses_an_expected_value_from_the_code_itself() {
    let t = tree();
    let ok = row(&t, |s| !s.fixtures.is_empty());
    for (prov, tol) in [
        ("self-snapshot", 1e-6),
        ("agent-generated", 1e-6),
        ("published-source", 0.0),
    ] {
        let mut bad = ok.clone();
        bad.fixtures[0].provenance = prov.into();
        bad.fixtures[0].tolerance = tol;
        let f = failed(&gate_node(&bad, &t));
        assert!(f.contains(&"provenance"), "{prov}/{tol}: {f:?}");
    }
}

#[test]
fn domain_refuses_limits_out_of_order_and_a_value_outside_them() {
    let t = tree();
    let ok = row(&t, |s| {
        s.is_declared() && s.value.is_some() && s.lower < s.upper
    });
    let mut bad = ok.clone();
    bad.lower = bad.upper;
    let f = failed(&gate_node(&bad, &t));
    assert!(f.contains(&"domain"), "{f:?}");
    let mut bad = ok.clone();
    bad.value = Some(bad.upper + (bad.upper - bad.lower).abs() + 1.0);
    breaks(&t, &ok, &bad, "domain");
}

#[test]
fn seeded_rows_are_held_to_what_a_seed_owns() {
    let t = tree();
    let ok = t
        .sheets
        .values()
        .find(|s| s.is_seeded())
        .expect("a seeded row")
        .clone();
    let mut bad = ok.clone();
    bad.owner.clear();
    assert_eq!(failed(&gate_node(&bad, &t)), vec!["seeded"]);
    let mut bad = ok.clone();
    bad.parent = "no_such_group".into();
    assert_eq!(failed(&gate_node(&bad, &t)), vec!["parent"]);
}

// ---------------------------------------------------------------------------
// the checks that read a node's method

#[test]
fn sense_applied_reads_the_code_and_not_its_comments() {
    let t = tree();
    // A closure whose requirement declares a sense.
    let ok = row(&t, |s| {
        s.inputs.iter().any(|i| {
            i.binding == "req"
                && t.sheets
                    .get(i.var.split('.').next().unwrap())
                    .is_some_and(|r| matches!(r.sense.trim(), "<=" | ">="))
        })
    });
    let req = ok.inputs.iter().find(|i| i.binding == "req").unwrap();
    let sense = t.sheets[req.var.split('.').next().unwrap()]
        .sense
        .trim()
        .to_string();
    // A method says it as the margin it takes.
    let (want, other) = if sense == "<=" {
        ("margin_at_most", "margin_at_least")
    } else {
        ("margin_at_least", "margin_at_most")
    };
    assert!(
        ok.method.text.contains(want),
        "{} does not apply {want}",
        ok.id
    );
    let mut bad = ok.clone();
    // The method applies the opposite; a comment names the right one. The
    // check once read the comment and passed.
    bad.method.text = format!(
        "# the requirement is {want}\n{}",
        ok.method.text.replacen(want, other, 1)
    );
    let f = failed(&gate_node(&bad, &t));
    assert!(f.contains(&"sense-applied"), "{f:?}");
    // And the method as it is passes.
    assert!(!failed(&gate_node(&ok, &t)).contains(&"sense-applied"));
}

// ---------------------------------------------------------------------------
// the assembly checks, on the whole tree

fn tree_fails(t: &Tree) -> Vec<&'static str> {
    failed(&validate_tree(t))
}

#[test]
fn the_tree_as_it_is_passes_every_assembly_check() {
    assert_eq!(tree_fails(&tree()), Vec::<&str>::new());
}

fn assembly_breaks(check: &str, edit: impl FnOnce(&mut Tree)) {
    let mut t = tree();
    edit(&mut t);
    let f = tree_fails(&t);
    assert!(f.contains(&check), "breaking {check} failed {f:?}");
}

fn first_computed(t: &mut Tree) -> &mut Sheet {
    t.sheets
        .values_mut()
        .find(|s| s.state == "published" && !s.is_declared() && !s.inputs.is_empty())
        .unwrap()
}

#[test]
fn v1_an_edge_to_nothing() {
    assembly_breaks("V1 edge endpoints exist", |t| {
        first_computed(t).inputs[0].var = "no_such_row".into();
    });
}

#[test]
fn v2_an_edge_twice() {
    assembly_breaks("V2 no duplicate edges", |t| {
        let s = first_computed(t);
        let dup = s.inputs[0].clone();
        s.inputs.push(dup);
    });
}

#[test]
fn v3_a_parent_that_is_not_a_group() {
    assembly_breaks("V3 parents resolve", |t| {
        first_computed(t).parent = "no_such_group".into();
    });
}

#[test]
fn v4_an_undeclared_loop() {
    assembly_breaks("V4 every cycle is declared", |t| {
        // a reads b; make b read a as well.
        let (a, b) = {
            let s = t
                .sheets
                .values()
                .find(|s| {
                    s.state == "published"
                        && s.inputs.iter().any(|i| {
                            !i.var.contains('.')
                                && t.sheets.get(&i.var).is_some_and(|p| !p.is_declared())
                        })
                })
                .unwrap();
            let i = s
                .inputs
                .iter()
                .find(|i| !i.var.contains('.') && !t.sheets[&i.var].is_declared())
                .unwrap();
            (s.id.clone(), i.var.clone())
        };
        let mut back = t.sheets[&b].inputs[0].clone();
        back.var = a;
        back.binding = "loop_back".into();
        t.sheets.get_mut(&b).unwrap().inputs.push(back);
    });
}

#[test]
fn v5_a_computed_row_with_no_inputs() {
    assembly_breaks("V5 computed nodes have inputs", |t| {
        first_computed(t).inputs.clear();
    });
}

#[test]
fn v6_two_rows_in_one_folder() {
    assembly_breaks("V6 no folder collisions", |t| {
        let (crate_name, folder) = {
            let s = t.sheets.values().next().unwrap();
            (s.crate_name.clone(), s.folder.clone())
        };
        let other = t
            .sheets
            .values_mut()
            .filter(|s| s.crate_name == crate_name)
            .nth(1)
            .unwrap();
        other.folder = folder;
    });
}

#[test]
fn v7_a_group_with_no_owner() {
    assembly_breaks("V7 every layer has an owner", |t| {
        t.groups.values_mut().next().unwrap().owner.clear();
    });
}

#[test]
fn v8_a_citation_to_nothing() {
    assembly_breaks("V8 sources resolve", |t| {
        first_computed(t).source = "no_such_source_2099".into();
    });
}

#[test]
fn v10_an_unlabelled_relation() {
    assembly_breaks("V10 relations resolve and are labelled", |t| {
        t.relations.first_mut().expect("a relation").why.clear();
    });
}

#[test]
fn v11_a_cycle_that_converges_outside_itself() {
    assembly_breaks("V11 declared cycles are well formed", |t| {
        let c = t.cases.values_mut().find(|c| !c.cycles.is_empty()).unwrap();
        c.cycles[0].converge_on = "somewhere_else".into();
    });
    assembly_breaks("V11 declared cycles are well formed", |t| {
        let c = t.cases.values_mut().find(|c| !c.cycles.is_empty()).unwrap();
        c.cycles[0].max_iter = 0;
    });
}

#[test]
fn v12_one_group_split_across_crates() {
    assembly_breaks("V12 one crate per owner", |t| {
        first_computed(t).crate_name = "vleo-mod-elsewhere".into();
    });
}

#[test]
fn v14_two_rows_in_one_place() {
    assembly_breaks("V14 one row per place", |t| {
        let order = t.sheets.values().next().unwrap().order;
        t.sheets.values_mut().nth(1).unwrap().order = order;
    });
}

#[test]
fn v15_two_rows_with_one_label_in_a_group() {
    assembly_breaks("V15 one label per row in a group", |t| {
        let (parent, label) = {
            let s = t.sheets.values().next().unwrap();
            (s.parent.clone(), s.label.clone())
        };
        let twin = t
            .sheets
            .values_mut()
            .filter(|s| s.parent == parent)
            .nth(1)
            .expect("a group with two rows");
        twin.label = label;
    });
}

#[test]
fn v16_a_case_setting_a_row_a_run_overwrites() {
    assembly_breaks("V16 the case supplies only what a run can take", |t| {
        let computed = t
            .sheets
            .values()
            .find(|s| s.state == "published" && !s.is_declared())
            .unwrap()
            .id
            .clone();
        t.cases
            .values_mut()
            .next()
            .unwrap()
            .supply
            .push((computed, 1.0));
    });
}

#[test]
fn cases_ask_for_the_authors_code_only_when_they_came_from_it() {
    // A case worked by hand, in a spreadsheet or from a paper is evidence
    // without code beside it; one from the author's code, or that does not
    // say where it came from, is not until that code is here.
    let t = tree();
    let original = row(&t, |s| {
        s.cases.is_empty() && s.author.code.trim().is_empty() && !s.is_declared()
    });
    let case = |origin: &str| vleo_sheet::method::Case {
        label: format!("from {origin}"),
        inputs: Vec::new(),
        expect: Some(1.0),
        tolerance: 1e-6,
        also: Vec::new(),
        origin: origin.into(),
    };
    let with = |origins: &[&str]| {
        let mut s = original.clone();
        s.cases = origins.iter().map(|o| case(o)).collect();
        failed(&gate_node(&s, &t)).contains(&"cases")
    };
    assert!(
        !with(&["hand", "paper", "spreadsheet"]),
        "{}: no case came from code",
        original.id
    );
    assert!(
        with(&["hand", "code"]),
        "{}: a case from code, and no code",
        original.id
    );
    assert!(
        with(&["paper", ""]),
        "{}: a case that does not say is the author's code's",
        original.id
    );
}
