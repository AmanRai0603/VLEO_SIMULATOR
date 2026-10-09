//! A value of the wrong kind in a sheet stops the load, naming the file.
//!
//! The loader used to read `lower = "1e-6"` as a lower bound of 0.0 and a
//! table where a list of tables belonged as a panic. Each is tried on the
//! design, read from `design/`, with one real node's sheet edited in memory as
//! the files serve it; nothing on disk is touched.

use std::path::{Path, PathBuf};
use vleo_files::convert::{serve, Served};
use vleo_sheet::files::Files;
use vleo_sheet::load::load_all_from;

fn real() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The design's files, with the sheet of one real node edited.
fn one_node_edited(edit: impl Fn(&str) -> String) -> Served {
    let (mut served, _) = serve(&real()).unwrap();
    let tree = load_all_from(&served, &real()).unwrap();
    let sh = tree
        .sheets
        .values()
        .find(|s| s.lower != 0.0 && s.inputs.is_empty())
        .expect("a row with a declared lower bound");
    let sheet = sh.dir.join("node.toml");
    let text = served.read_to_string(&sheet).unwrap();
    served.replace(&sheet, Some(edit(&text).into_bytes()));
    served
}

#[test]
fn a_number_written_as_text_is_named_not_read_as_zero() {
    let served = one_node_edited(|t| {
        let mut out = String::new();
        let mut done = false;
        for line in t.lines() {
            if !done && line.trim_start().starts_with("lower =") {
                out.push_str("lower = \"1e-6\"\n");
                done = true;
            } else {
                out.push_str(line);
                out.push('\n');
            }
        }
        assert!(done, "the sheet has no `lower =` line");
        out
    });
    let Err(err) = load_all_from(&served, &real()) else {
        panic!("a bound written as text was accepted");
    };
    assert_eq!(err.kind(), vleo_sheet::ErrorKind::Malformed, "{err}");
    assert!(
        err.message().contains("\"1e-6\"") && err.message().contains("a number"),
        "{err}"
    );
}

#[test]
fn the_same_tree_untouched_loads() {
    let served = one_node_edited(str::to_string);
    load_all_from(&served, &real()).expect("an unedited node did not load");
}

#[test]
fn a_misspelt_key_is_named_with_the_one_it_meant() {
    let served = one_node_edited(|t| t.replacen("\nlower =", "\nlowr =", 1));
    let Err(err) = load_all_from(&served, &real()) else {
        panic!("a misspelt key was passed over");
    };
    assert_eq!(err.kind(), vleo_sheet::ErrorKind::Malformed, "{err}");
    assert!(
        err.message().contains("output.lowr") && err.message().contains("did you mean `lower`"),
        "{err}"
    );
}

#[test]
fn every_key_the_form_writes_is_one_the_loader_takes() {
    use vleo_sheet::form::{ARRAYS, FIELDS};
    use vleo_sheet::load::SCHEMA;
    let allowed = |table: &str, key: &str| {
        SCHEMA
            .iter()
            .any(|(t, keys)| *t == table && keys.contains(&key))
    };
    for f in FIELDS {
        // A field of a table is also that table's key at the top level.
        let (parent, own) = match f.table.rsplit_once('.') {
            Some((p, o)) => (p, o),
            None => ("", f.table),
        };
        if !f.table.is_empty() {
            assert!(allowed(parent, own), "the form writes table `{}`", f.table);
        }
        assert!(
            allowed(f.table, f.key),
            "the form writes `{}.{}`",
            f.table,
            f.key
        );
    }
    for a in ARRAYS {
        assert!(
            allowed("", a.path.split('.').next().unwrap()),
            "the form writes [[{}]]",
            a.path
        );
        for c in a.columns {
            assert!(
                allowed(a.path, c.key),
                "the form writes `{}.{}`",
                a.path,
                c.key
            );
        }
    }
}
