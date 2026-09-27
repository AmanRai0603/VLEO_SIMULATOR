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

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
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
    let changed = edit(&form_for(ROW), DATA, |t| {
        set_field(t, "expression", "Ap_design(G) = a different relation")
    });
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
    let html = template::document(sh, &tree);
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
        .contains("vleo-node-form/0"));
    let retargeted = edit(&html, DATA, |t| {
        t.insert("node".into(), toml::Value::String("sw_f107_design".into()));
    });
    assert!(template::plan(&r, &retargeted)
        .unwrap_err()
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
        .contains("never adds a node"));
    let bogus = edit(&html, DATA, |t| {
        t.get_mut("filled_by")
            .and_then(|b| b.as_table_mut())
            .unwrap()
            .insert("ai".into(), toml::Value::String("a little".into()));
    });
    assert!(template::plan(&r, &bogus).unwrap_err().contains("a little"));
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
