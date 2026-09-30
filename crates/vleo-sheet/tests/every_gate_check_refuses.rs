//! Every check the gate runs on a node refuses what it exists to refuse.
//!
//! A check that passes whatever it is given looks exactly like a check that
//! passes because the tree is right. So each one is handed a node the gate
//! passes, broken in the one way that check is for, and must name it. The
//! tree is read from the repository and changed only in memory; a check that
//! reads files beside the node reads a copy of its folder in a temporary one.

use std::path::{Path, PathBuf};
use vleo_sheet::gate::{gate_node, Verdict};
use vleo_sheet::model::{Flight, Sheet};
use vleo_sheet::{load_all, Tree};

fn tree() -> Tree {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    load_all(&root).expect("the tree loads")
}

/// The verdict of one named check on a sheet.
fn verdict(sh: &Sheet, tree: &Tree, name: &str) -> Verdict {
    gate_node(sh, tree)
        .into_iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("{}: the gate ran no check named {name}", sh.id))
        .verdict
}

fn refuses(sh: &Sheet, tree: &Tree, name: &str) {
    assert!(
        matches!(verdict(sh, tree, name), Verdict::Fail(_)),
        "{}: `{name}` passed a node broken for it",
        sh.id
    );
}

fn passes(sh: &Sheet, tree: &Tree, name: &str) {
    if let Verdict::Fail(w) = verdict(sh, tree, name) {
        panic!("{}: `{name}` refused the node as it is: {w}", sh.id);
    }
}

/// The first published node every check passes, that has what `want` asks.
fn a_good(tree: &Tree, want: impl Fn(&Sheet) -> bool) -> &Sheet {
    tree.ordered()
        .into_iter()
        .filter(|s| !s.is_seeded() && want(s))
        .find(|s| gate_node(s, tree).iter().all(|c| !c.failed()))
        .expect("no published node that passes the gate has what this check needs")
}

/// A copy of a node's folder in a temporary one, and the sheet pointed at it.
fn in_a_copy(sh: &Sheet, tag: &str) -> (Sheet, PathBuf) {
    let dir =
        std::env::temp_dir().join(format!("vleo-gate-{tag}-{}-{}", sh.id, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for e in std::fs::read_dir(&sh.dir).unwrap().flatten() {
        if e.path().is_file() {
            std::fs::copy(e.path(), dir.join(e.file_name())).unwrap();
        }
    }
    let mut copy = sh.clone();
    copy.dir = dir.clone();
    (copy, dir)
}

/// Put `line` at the top of the node's first hole, in its copied model.rs.
fn into_first_hole(dir: &Path, line: &str) {
    let p = dir.join("model.rs");
    let text = std::fs::read_to_string(&p).unwrap();
    let at = text.find("// ---- HOLE ").expect("the model has a hole");
    let eol = at + text[at..].find('\n').unwrap() + 1;
    std::fs::write(&p, format!("{}{line}\n{}", &text[..eol], &text[eol..])).unwrap();
}

#[test]
fn every_sheet_check_refuses_what_it_is_for() {
    let t = tree();
    let computed = a_good(&t, |s| {
        !s.is_declared() && !s.inputs.is_empty() && !s.fixtures.is_empty()
    });

    let mut s = computed.clone();
    s.question.clear();
    refuses(&s, &t, "schema");

    let mut s = computed.clone();
    s.inputs.clear();
    refuses(&s, &t, "inputs");

    let mut s = computed.clone();
    s.inputs[0].var = "no_such_row".into();
    refuses(&s, &t, "contract");

    let mut s = computed.clone();
    s.inputs[0].ty = "a type nothing publishes".into();
    refuses(&s, &t, "contract");

    let mut s = computed.clone();
    s.source = "no-such-source".into();
    refuses(&s, &t, "sources");

    let mut s = computed.clone();
    s.criticality = "major".into();
    refuses(&s, &t, "criticality");

    let mut s = computed.clone();
    s.fixtures[0].provenance = "the-code-under-test".into();
    refuses(&s, &t, "provenance");

    let mut s = computed.clone();
    s.fixtures[0].tolerance = 0.0;
    refuses(&s, &t, "provenance");

    let mut s = computed.clone();
    s.fixtures[0].variable = "not_published_here".into();
    refuses(&s, &t, "publishes");

    let mut s = computed.clone();
    s.upper = s.lower;
    refuses(&s, &t, "domain");

    let declared = a_good(&t, |s| s.is_declared() && s.value.is_some());
    let mut s = declared.clone();
    s.confirmed_by.clear();
    refuses(&s, &t, "declared-value");
    let mut s = declared.clone();
    s.value = Some(s.upper + (s.upper - s.lower).abs() + 1.0);
    refuses(&s, &t, "domain");

    let mut s = computed.clone();
    if let Some(v) = s.versions.first_mut() {
        v.n = 7;
    } else {
        s.versions.push(Default::default());
    }
    refuses(&s, &t, "versions");

    let mut s = computed.clone();
    s.flight.push(Flight::default());
    refuses(&s, &t, "flight");
}

#[test]
fn a_requirement_says_which_way_it_binds() {
    let t = tree();
    let req = a_good(&t, |s| s.kind == "required");
    passes(req, &t, "sense");
    for wrong in ["", "==", "< ="] {
        let mut s = req.clone();
        s.sense = wrong.into();
        refuses(&s, &t, "sense");
    }
}

/// Refused, and for the reason given — so a check that refuses a made-up
/// node for some other reason does not pass for this one.
fn refuses_because(sh: &Sheet, tree: &Tree, name: &str, why: &str) {
    match verdict(sh, tree, name) {
        Verdict::Fail(w) if w.contains(why) => {}
        other => panic!("{}: `{name}` did not refuse for «{why}»: {other:?}", sh.id),
    }
}

#[test]
fn a_method_by_an_assistant_and_cases_without_their_code_are_refused() {
    // No published row carries a method or cases yet, so a passing one is
    // given them, broken in the one way each check is for.
    let t = tree();
    let n = a_good(&t, |s| !s.is_declared() && !s.inputs.is_empty());
    let mut s = n.clone();
    s.method.text = "y = x".into();
    s.method.by = "Claude".into();
    refuses_because(&s, &t, "method", "an assistant");

    let mut s = n.clone();
    s.cases.push(Default::default());
    s.author.language = "python".into();
    s.author.name = "A. Author".into();
    s.author.test_code = "assert f(1) == 1".into();
    refuses_because(&s, &t, "cases", "the code is not here");
    s.author.code = "def f(x): return x".into();
    passes(&s, &t, "cases");
}

#[test]
fn a_hand_edit_outside_a_hole_is_refused() {
    let t = tree();
    let n = a_good(&t, |s| {
        !s.is_declared() && s.dir.join("contract.rs").is_file()
    });
    let (s, dir) = in_a_copy(n, "regenerate");
    passes(&s, &t, "regenerate");
    let p = dir.join("contract.rs");
    let text = std::fs::read_to_string(&p).unwrap();
    std::fs::write(&p, format!("{text}\npub const HAND_EDIT: u8 = 1;\n")).unwrap();
    refuses(&s, &t, "regenerate");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn platform_maths_in_a_hole_is_refused_and_in_a_comment_is_not() {
    let t = tree();
    let n = a_good(&t, |s| !s.is_declared() && s.dir.join("model.rs").is_file());
    for (tag, line, bad) in [
        ("comment", "// x.asin() is what pmath::asin replaces", false),
        ("method", "let _leak = 0.5_f64.asin();", true),
        ("path", "let _leak = f64::hypot(3.0, 4.0);", true),
    ] {
        let (s, dir) = in_a_copy(n, tag);
        into_first_hole(&dir, line);
        if bad {
            refuses(&s, &t, "portable-maths");
        } else {
            passes(&s, &t, "portable-maths");
        }
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn the_sense_the_code_applies_is_the_sense_the_requirement_declares() {
    let t = tree();
    // A row reading a requirement, whose code applies that requirement's sense.
    let n = a_good(&t, |s| {
        s.inputs.iter().any(|i| i.binding == "req")
            && std::fs::read_to_string(s.dir.join("model.rs"))
                .is_ok_and(|m| m.contains("Sense::AtMost") ^ m.contains("Sense::AtLeast"))
    });
    let model = std::fs::read_to_string(n.dir.join("model.rs")).unwrap();
    let (right, wrong) = if model.contains("Sense::AtMost") {
        ("Sense::AtMost", "Sense::AtLeast")
    } else {
        ("Sense::AtLeast", "Sense::AtMost")
    };

    // Named in a comment only: not applied.
    let (s, dir) = in_a_copy(n, "sense-comment");
    into_first_hole(
        &dir,
        &format!("// not {wrong}, which would read the bound backwards"),
    );
    passes(&s, &t, "sense-applied");
    std::fs::remove_dir_all(dir).unwrap();

    // Applied the wrong way round.
    let (s, dir) = in_a_copy(n, "sense-swapped");
    std::fs::write(dir.join("model.rs"), model.replace(right, wrong)).unwrap();
    refuses(&s, &t, "sense-applied");
    std::fs::remove_dir_all(dir).unwrap();

    // Applied both ways: one of them is wrong, whatever else is right.
    let (s, dir) = in_a_copy(n, "sense-both");
    into_first_hole(&dir, &format!("let _other = {wrong};"));
    refuses(&s, &t, "sense-applied");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn a_row_that_returns_its_one_input_unchanged_is_refused() {
    let t = tree();
    let n = a_good(&t, |s| {
        !s.is_declared()
            && s.inputs.len() == 1
            && s.crosses_to.trim().is_empty()
            && t.sheets
                .get(s.inputs[0].var.split('.').next().unwrap_or(""))
                .is_some_and(|p| p.layer == s.layer)
            && s.dir.join("model.rs").is_file()
    });
    let (s, dir) = in_a_copy(n, "identity");
    into_first_hole(&dir, &format!("let relayed = {};", n.inputs[0].binding));
    refuses(&s, &t, "no-identity");
    std::fs::remove_dir_all(dir).unwrap();
}
