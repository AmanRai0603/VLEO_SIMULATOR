//! The node form: made from a sheet, filled somewhere else, read back.
//!
//! The form travels outside the repository and comes back through somebody
//! else's hands, so the tests here are about what can go wrong on the way: a
//! form that does not read back as the node it was made from, a change in the
//! repository overwritten by a form made before it, a relation an assistant
//! supplied, a numbered step removed from the middle, a file that is not a form
//! at all. Each is tried against the real tree, and every edit is made to the
//! TOML block directly — the way a person with a text editor, or an assistant,
//! fills it without the page.

use std::path::{Path, PathBuf};
use vleo_sheet::form::{self, Saved};
use vleo_sheet::load::load_all;
use vleo_sheet::template::{self, Verdict};

/// A copy of the tree in a temporary folder, made once for this test binary:
/// applying a form writes sheets and regenerates folders, and that is done to
/// the copy, never to the checkout a developer may be editing.
fn root() -> PathBuf {
    static COPY: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    COPY.get_or_init(|| {
        let real = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let to = std::env::temp_dir().join(format!("vleo-form-tree-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&to);
        for e in std::fs::read_dir(real.join("crates")).unwrap().flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("vleo-mod-") {
                copy(
                    &e.path().join("nodes"),
                    &to.join("crates").join(&name).join("nodes"),
                );
            }
        }
        copy(
            &real.join("crates/vleo-wasm/src"),
            &to.join("crates/vleo-wasm/src"),
        );
        for d in ["layers", "cases", "sources", "web"] {
            copy(&real.join(d), &to.join(d));
        }
        to
    })
    .clone()
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            copy(&p, &to.join(e.file_name()));
        } else {
            std::fs::copy(&p, to.join(e.file_name())).unwrap();
        }
    }
}

/// A published computed row with a note, four assumptions and a relation.
const ROW: &str = "sw_ap_design";

fn form_for(id: &str) -> String {
    let tree = load_all(&root()).unwrap();
    template::document(tree.sheets.get(id).unwrap(), &tree)
}

/// Rewrite one of the form's TOML blocks, as a text editor would.
fn edit(html: &str, id: &str, f: impl FnOnce(&mut toml::Table)) -> String {
    let open = format!("id=\"{id}\">\n");
    let a = html.find(&open).unwrap() + open.len();
    let b = a + html[a..].find("</script>").unwrap();
    let mut t: toml::Table = html[a..b].parse().unwrap();
    f(&mut t);
    format!(
        "{}{}{}",
        &html[..a],
        toml::to_string(&t).unwrap(),
        &html[b..]
    )
}
const DATA: &str = "vleo-node-form";
const ORIGINAL: &str = "vleo-node-original";

fn set_field(t: &mut toml::Table, field: &str, v: &str) {
    t.get_mut("fields")
        .and_then(|f| f.as_table_mut())
        .unwrap()
        .insert(field.into(), toml::Value::String(v.into()));
}

/// Say why the node is changing, as a filler would.
fn with_record(html: &str) -> String {
    edit(html, DATA, |t| {
        let mut d = toml::Table::new();
        for (k, v) in [
            (
                "believed",
                "that the old relation held across the storm scale",
            ),
            (
                "tested",
                "compared against the NOAA scale table, docs/MATLAB_PORT_PLAN.md",
            ),
            ("learned", "it did not hold above G3"),
            ("cost", "half a day"),
            ("changed", "the relation, which now holds across the scale"),
            ("risks", ""),
            ("rests_on", "the NOAA G-scale thresholds"),
            ("breaks_if", "NOAA revises the scale"),
        ] {
            d.insert(k.into(), toml::Value::String(v.into()));
        }
        t.insert("derisk".into(), toml::Value::Table(d));
    })
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
fn every_node_form_reads_back_as_the_node_it_was_made_from() {
    let tree = load_all(&root()).unwrap();
    let mut n = 0;
    for sh in tree.sheets.values() {
        let html = template::document(sh, &tree);
        let f = template::read(&html).unwrap_or_else(|e| panic!("{}: {e}", sh.id));
        let now = template::content(sh);
        assert_eq!(f.node, sh.id);
        assert_eq!(
            f.original, now,
            "{}: the original block is not the node",
            sh.id
        );
        assert_eq!(f.filled, now, "{}: the fresh form is not the node", sh.id);
        n += 1;
    }
    assert_eq!(n, tree.sheets.len());
}

#[test]
fn an_untouched_form_changes_nothing() {
    let p = template::plan(&root(), &form_for(ROW)).unwrap();
    assert!(p.base_current);
    assert!(p.items.is_empty(), "{:?}", p.items);
    assert!(p.text.is_none());
}

#[test]
fn a_change_is_applied_unless_the_repository_changed_it_too() {
    let html = edit(&form_for(ROW), DATA, |t| {
        set_field(t, "note", "A note from the payload team.")
    });
    let p = template::plan(&root(), &html).unwrap();
    assert_eq!(verdict_of(&p, "note"), Verdict::Apply);
    assert!(p
        .text
        .as_deref()
        .unwrap()
        .contains("A note from the payload team."));

    // The form was made when the note said something else, and the repository
    // has moved it since: the form must not overwrite that.
    let stale = edit(&html, ORIGINAL, |t| set_field(t, "note", "An older note."));
    let p = template::plan(&root(), &stale).unwrap();
    assert!(
        matches!(verdict_of(&p, "note"), Verdict::Conflict(_)),
        "{:?}",
        p.items
    );
    assert!(p.text.is_none(), "a conflict was written anyway");

    // And a form asking for what the node already says has nothing to do.
    let tree = load_all(&root()).unwrap();
    let now = form::value(tree.sheets.get(ROW).unwrap(), "note");
    let same = edit(&stale, DATA, |t| set_field(t, "note", &now));
    assert_eq!(
        verdict_of(&template::plan(&root(), &same).unwrap(), "note"),
        Verdict::Already
    );
}

#[test]
fn a_relation_an_assistant_supplied_is_refused_and_a_persons_is_applied() {
    let changed = with_record(&edit(&form_for(ROW), DATA, |t| {
        set_field(t, "expression", "Ap_design(G) = a different relation")
    }));
    let p = template::plan(&root(), &changed).unwrap();
    assert_eq!(verdict_of(&p, "expression"), Verdict::Apply);
    assert!(
        p.relation,
        "a changed relation must carry a name when applied"
    );

    let assisted = edit(&changed, DATA, |t| {
        t.get_mut("filled_by")
            .and_then(|b| b.as_table_mut())
            .unwrap()
            .insert("ai".into(), toml::Value::String("relation".into()));
    });
    let p = template::plan(&root(), &assisted).unwrap();
    assert!(
        matches!(verdict_of(&p, "expression"), Verdict::Refused(ref w) if w.contains("mathematics")),
        "{:?}",
        p.items
    );
    assert!(p.text.is_none());
}

#[test]
fn a_numbered_step_removed_from_the_middle_is_refused_and_the_last_is_not() {
    let tree = load_all(&root()).unwrap();
    let sh = tree
        .ordered()
        .into_iter()
        .find(|s| s.steps.len() >= 2 && s.state == "published")
        .expect("no published row with two algorithm steps");
    let html = with_record(&template::document(sh, &tree));
    let drop = |at: usize| {
        edit(&html, DATA, |t| {
            t.get_mut("algorithm")
                .and_then(|a| a.as_array_mut())
                .unwrap()
                .remove(at);
        })
    };
    let middle = template::plan(&root(), &drop(0)).unwrap();
    assert!(
        matches!(verdict_of(&middle, "algorithm"), Verdict::Refused(ref w) if w.contains("middle")),
        "{}: {:?}",
        sh.id,
        middle.items
    );
    let last = template::plan(&root(), &drop(sh.steps.len() - 1)).unwrap();
    let removed = format!("algorithm {} · removed", sh.steps.len());
    assert_eq!(
        verdict_of(&last, &removed),
        Verdict::Apply,
        "{:?}",
        last.items
    );
}

#[test]
fn known_values_are_a_request_and_never_touch_the_node() {
    let html = edit(&form_for(ROW), DATA, |t| {
        let mut k = toml::Table::new();
        for (key, v) in [
            ("label", "G2"),
            ("inputs", "sw_storm_design_level = 2"),
            ("expected", "80"),
            ("tolerance", ""),
            ("provenance", "published-source"),
            ("source", "NOAA G scale"),
        ] {
            k.insert(key.into(), toml::Value::String(v.into()));
        }
        t.insert("known_value".into(), toml::Value::Array(vec![k.into()]));
    });
    let p = template::plan(&root(), &html).unwrap();
    assert!(p.items.is_empty() && p.text.is_none(), "{:?}", p.items);
    let req = template::fixture_request(&p.form);
    assert!(
        req.contains("[[fixture]]") && req.contains("expect = 80"),
        "{req}"
    );
    let parsed: toml::Value = req.parse().expect("the request is not TOML");
    assert!(parsed.get("fixture").is_some());
}

#[test]
fn what_is_not_a_form_for_a_node_here_is_refused_by_name() {
    let r = root();
    assert!(template::plan(&r, "<html>hello</html>")
        .unwrap_err()
        .message()
        .contains("not a node form"));
    let html = form_for(ROW);
    let other_format = edit(&html, DATA, |t| {
        t.insert(
            "format".into(),
            toml::Value::String("vleo-node-form/0".into()),
        );
    });
    assert!(template::plan(&r, &other_format)
        .unwrap_err()
        .message()
        .contains("vleo-node-form/0"));
    let retargeted = edit(&html, DATA, |t| {
        t.insert("node".into(), toml::Value::String("sw_f107_design".into()));
    });
    assert!(template::plan(&r, &retargeted)
        .unwrap_err()
        .message()
        .contains("cannot be trusted"));
    let unknown = edit(
        &edit(&html, DATA, |t| {
            t.insert("node".into(), toml::Value::String("no_such_row".into()));
        }),
        ORIGINAL,
        |t| {
            t.insert("node".into(), toml::Value::String("no_such_row".into()));
        },
    );
    assert!(template::plan(&r, &unknown)
        .unwrap_err()
        .message()
        .contains("never adds a node"));
    let bogus = edit(&html, DATA, |t| {
        t.get_mut("filled_by")
            .and_then(|b| b.as_table_mut())
            .unwrap()
            .insert("ai".into(), toml::Value::String("a little".into()));
    });
    assert!(template::plan(&r, &bogus)
        .unwrap_err()
        .message()
        .contains("a little"));
}

// ---------------------------------------------------------------------------
// applying — against the real tree, one test at a time, and put back

fn serially() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|p| p.into_inner())
}

/// Puts a node's folder back exactly as it was, whatever the test did.
struct Restore(Vec<(PathBuf, Vec<u8>)>, PathBuf);

impl Restore {
    fn take(id: &str) -> Restore {
        let tree = load_all(&root()).unwrap();
        let dir = tree.sheets.get(id).unwrap().dir.clone();
        let files = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .map(|p| {
                let b = std::fs::read(&p).unwrap();
                (p, b)
            })
            .collect();
        Restore(files, dir)
    }
}

impl Drop for Restore {
    fn drop(&mut self) {
        for e in std::fs::read_dir(&self.1).unwrap().flatten() {
            if e.path().is_file() && !self.0.iter().any(|(p, _)| *p == e.path()) {
                let _ = std::fs::remove_file(e.path());
            }
        }
        for (p, b) in &self.0 {
            let _ = std::fs::write(p, b);
        }
    }
}

fn sheet_text(id: &str) -> String {
    let tree = load_all(&root()).unwrap();
    std::fs::read_to_string(tree.sheets.get(id).unwrap().dir.join("node.toml")).unwrap()
}

#[test]
fn applying_writes_regenerates_and_gates_or_puts_everything_back() {
    let _serial = serially();
    let _put_back = Restore::take(ROW);
    let before = sheet_text(ROW);

    // A bound the gate refuses — above the upper one — is not left behind.
    let bad = edit(&form_for(ROW), DATA, |t| set_field(t, "lower", "1000000.0"));
    let p = template::plan(&root(), &bad).unwrap();
    if p.text.is_some() {
        match template::apply(&root(), &p) {
            Saved::Refused(_) => {}
            Saved::Ok { .. } => panic!("a lower bound above the upper was applied"),
            Saved::Stale { .. } => panic!("stale on a fresh plan"),
        }
    }
    assert_eq!(
        sheet_text(ROW),
        before,
        "a refused form left the sheet changed"
    );

    // A good one is written, and the gate passes on it.
    let good = edit(&form_for(ROW), DATA, |t| {
        set_field(t, "note", "A note applied from a form.")
    });
    let p = template::plan(&root(), &good).unwrap();
    match template::apply(&root(), &p) {
        Saved::Ok { .. } => {}
        Saved::Stale { .. } => panic!("stale on a fresh plan"),
        Saved::Refused(e) => panic!("a good form was refused: {e}"),
    }
    assert!(sheet_text(ROW).contains("A note applied from a form."));

    // And a plan made before that write is stale now, not applied over it.
    let late = edit(&form_for(ROW), DATA, |t| set_field(t, "note", "Too late."));
    let mut p2 = template::plan(&root(), &late).unwrap();
    p2.current_hash = "000000".into();
    assert!(matches!(template::apply(&root(), &p2), Saved::Stale { .. }));
}

// ---------------------------------------------------------------------------
// a node the design does not have yet, and the interfaces a form declares

fn new_form(id: &str, parent: &str, kind: &str, input: Option<(&str, &str)>) -> String {
    let tree = load_all(&root()).unwrap();
    let html = template::document_new(&tree);
    edit(&html, DATA, |t| {
        let mut n = toml::Table::new();
        for (k, v) in [("id", id), ("parent", parent), ("kind", kind)] {
            n.insert(k.into(), toml::Value::String(v.into()));
        }
        t.insert("new".into(), toml::Value::Table(n));
        set_field(t, "label", "A new row");
        set_field(t, "question", "What does the new row answer?");
        if let Some((var, ty)) = input {
            let mut r = toml::Table::new();
            for (k, v) in [("binding", "x"), ("var", var), ("type", ty)] {
                r.insert(k.into(), toml::Value::String(v.into()));
            }
            t.insert("input".into(), toml::Value::Array(vec![r.into()]));
        }
    })
}

#[test]
fn a_new_nodes_form_is_checked_for_its_place_and_its_interfaces_before_anything_is_built() {
    let tree = load_all(&root()).unwrap();
    let producer = tree.sheets.get(ROW).unwrap();
    let parent = producer.parent.clone();

    let good = template::plan(
        &root(),
        &new_form(
            "a_brand_new_row",
            &parent,
            "computed",
            Some((ROW, &producer.ty)),
        ),
    )
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
    let taken = template::plan(&root(), &new_form(ROW, &parent, "computed", None)).unwrap();
    assert!(
        matches!(verdict_of(&taken, "new · id"), Verdict::Refused(ref w) if w.contains("already"))
    );
    let bad_id =
        template::plan(&root(), &new_form("Not An Id", &parent, "computed", None)).unwrap();
    assert!(matches!(
        verdict_of(&bad_id, "new · id"),
        Verdict::Refused(_)
    ));
    let nowhere = template::plan(
        &root(),
        &new_form("x_row", "no_such_group", "computed", None),
    )
    .unwrap();
    assert!(matches!(
        verdict_of(&nowhere, "new · parent"),
        Verdict::Refused(_)
    ));
    let odd = template::plan(&root(), &new_form("x_row", &parent, "banana", None)).unwrap();
    assert!(matches!(
        verdict_of(&odd, "new · kind"),
        Verdict::Refused(_)
    ));

    // An input that names no row, or a row of another quantity, does not connect.
    let ghost = template::plan(
        &root(),
        &new_form("x_row", &parent, "computed", Some(("no_such_row", "Ratio"))),
    )
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
    let mismatch = template::plan(
        &root(),
        &new_form("x_row", &parent, "computed", Some((ROW, wrong))),
    )
    .unwrap();
    assert!(
        mismatch.interfaces[0].why.contains(wrong),
        "{:?}",
        mismatch.interfaces
    );
}

#[test]
fn an_input_changed_to_one_that_does_not_connect_is_refused_and_the_rest_still_applies() {
    let html = edit(&form_for(ROW), DATA, |t| {
        set_field(t, "note", "Still applies.");
        let ins = t.get_mut("input").and_then(|a| a.as_array_mut()).unwrap();
        ins[0]
            .as_table_mut()
            .unwrap()
            .insert("var".into(), toml::Value::String("no_such_row".into()));
    });
    let p = template::plan(&root(), &html).unwrap();
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
    let had = load_all(&root()).unwrap().sheets[ROW].versions.len();
    // A new bound and a new note, with nothing said about why.
    let html = edit(&form_for(ROW), DATA, |t| {
        set_field(t, "upper", "450.0");
        set_field(t, "note", "A reworded note.");
    });
    let p = template::plan(&root(), &html).unwrap();
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
    let p = template::plan(&root(), &with_record(&html)).unwrap();
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
    let html = with_record(&edit(&form_for(ROW), DATA, |t| {
        set_field(t, "upper", "450.0")
    }));
    let html = edit(&html, DATA, |t| {
        t.get_mut("derisk")
            .and_then(|d| d.as_table_mut())
            .unwrap()
            .insert("risks".into(), toml::Value::String("R-9999 closed".into()));
    });
    let p = template::plan(&root(), &html).unwrap();
    assert!(
        matches!(verdict_of(&p, "de-risking"), Verdict::Refused(ref w) if w.contains("not registered")),
        "{:?}",
        p.items
    );
    assert!(matches!(verdict_of(&p, "upper"), Verdict::Refused(_)));
}

#[test]
fn a_row_with_no_plain_words_yet_takes_them_from_its_form() {
    // A sheet with no [explain] table yet: the form's first explanatory answer
    // makes one, beside the question. Every published row has one now, so the
    // table is taken off a real sheet's text to see it made again.
    let tree = load_all(&root()).unwrap();
    let sh = tree.sheets.get(ROW).unwrap();
    let full = std::fs::read_to_string(sh.dir.join("node.toml")).unwrap();
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
    assert!(
        text.find("[explain]").unwrap() < text.find("[maths]").unwrap(),
        "the plain words are not beside the question"
    );

    // Through a form: the change applies, records no version, and names whose
    // words these are — taken from the form's filler.
    let html = edit(&form_for(ROW), DATA, |t| {
        set_field(
            t,
            "explain_simply",
            "What the row works out, in plain words.",
        );
        set_field(t, "explain_breaks", "Where that stops being true.");
        let who = t
            .get_mut("filled_by")
            .and_then(|b| b.as_table_mut())
            .unwrap();
        who.insert("name".into(), toml::Value::String("R. Kumar".into()));
        who.insert("team".into(), toml::Value::String("Solar".into()));
    });
    let p = template::plan(&root(), &html).unwrap();
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
    let tree = load_all(&root()).unwrap();
    let listed = tree.sources.keys().next().unwrap().clone();
    for (cited, open) in [
        ("the_book_nobody_listed_2031", true),
        (listed.as_str(), false),
    ] {
        let html = with_record(&edit(&form_for(ROW), DATA, |t| {
            set_field(t, "source", cited)
        }));
        let p = template::plan(&root(), &html).unwrap();
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
    // The form asks for the answer in the node's unit and the inputs as a
    // person writes them; fixtures.toml holds SI, by binding. Pasted as typed,
    // 70.2 degrees was 70.2 radians.
    let html = edit(
        &new_form(
            "orbit_new_angle",
            "l3_x_envorbit",
            "computed",
            Some(("orbit_radius", "Length")),
        ),
        DATA,
        |t| {
            set_field(t, "unit", "Degree");
            let mut k = toml::Table::new();
            for (key, v) in [
                ("label", "400 km"),
                ("inputs", "orbit_radius = 6778.137 km"),
                ("expected", "70.2179"),
                ("tolerance", "1e-4"),
                ("provenance", "independent-derivation"),
                ("source", "larson_wertz"),
            ] {
                k.insert(key.into(), toml::Value::String(v.into()));
            }
            t.insert("known_value".into(), toml::Value::Array(vec![k.into()]));
        },
    );
    let p = template::plan(&root(), &html).unwrap();
    let req = template::fixture_request(&p.form);
    let v: toml::Value = req.parse().expect("the request is not TOML");
    let fx = &v["fixture"][0];
    let expect = fx["expect"].as_float().unwrap();
    assert!((expect - 70.2179f64.to_radians()).abs() < 1e-9, "{req}");
    assert_eq!(fx["inputs"]["x"].as_float(), Some(6_778_137.0), "{req}");
    // Unreadable inputs are left for the person, never half-converted.
    let html = edit(&html, DATA, |t| {
        let k = t["known_value"].as_array_mut().unwrap()[0]
            .as_table_mut()
            .unwrap();
        k.insert(
            "inputs".into(),
            toml::Value::String("orbit_radius = 6778".into()),
        );
    });
    let req = template::fixture_request(&template::plan(&root(), &html).unwrap().form);
    let v: toml::Value = req.parse().unwrap();
    assert!(v["fixture"][0].get("inputs").is_none(), "{req}");
}

#[test]
fn a_method_of_the_forms_own_is_no_longer_a_transcription() {
    // A transcribed method names what it was copied from and who read the
    // copy against it. When a person's own method replaces it, neither
    // describes the row any more: intake clears both, and names the person.
    let id = "orbit_velocity";
    let tree = load_all(&root()).unwrap();
    assert!(
        !tree
            .sheets
            .get(id)
            .unwrap()
            .method
            .transcribed_from
            .is_empty(),
        "{id} is no longer a transcription; pick another"
    );
    let html = with_record(&edit(&form_for(id), DATA, |t| {
        set_field(
            t,
            "method_text",
            "# Vallado (2013), eq. 1-18: the circular two-body speed.\nreturn circular_velocity(r)",
        );
        t.get_mut("filled_by")
            .and_then(|b| b.as_table_mut())
            .unwrap()
            .insert("name".into(), toml::Value::String("R. Kumar".into()));
    }));
    let p = template::plan(&root(), &html).unwrap();
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
