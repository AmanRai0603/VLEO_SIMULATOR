//! A value of the wrong kind in a sheet stops the load, naming the file.
//!
//! The loader used to read `lower = "1e-6"` as a lower bound of 0.0 and a
//! table where a list of tables belonged as a panic. Each is tried on a one-node
//! tree in a temporary folder, not on the checkout.

use std::path::{Path, PathBuf};
use vleo_sheet::load::load_all;

fn real() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
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

/// A tree holding one real node and the real layer, case and source files.
fn one_node_tree(tag: &str, edit: impl Fn(&str) -> String) -> PathBuf {
    let root = std::env::temp_dir().join(format!("vleo-kind-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let tree = load_all(&real()).unwrap();
    let sh = tree
        .sheets
        .values()
        .find(|s| s.lower != 0.0 && s.inputs.is_empty())
        .expect("a row with a declared lower bound");
    let dir = root
        .join("crates")
        .join(&sh.crate_name)
        .join("nodes")
        .join(&sh.folder);
    std::fs::create_dir_all(&dir).unwrap();
    let text = std::fs::read_to_string(sh.dir.join("node.toml")).unwrap();
    std::fs::write(dir.join("node.toml"), edit(&text)).unwrap();
    for d in ["layers", "cases", "sources"] {
        copy(&real().join(d), &root.join(d));
    }
    root
}

#[test]
fn a_number_written_as_text_is_named_not_read_as_zero() {
    let root = one_node_tree("text", |t| {
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
    let Err(err) = load_all(&root) else {
        panic!("a bound written as text was accepted");
    };
    assert!(
        err.contains("\"1e-6\"") && err.contains("a number"),
        "{err}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_same_tree_untouched_loads() {
    let root = one_node_tree("clean", str::to_string);
    load_all(&root).expect("an unedited node did not load on its own");
    let _ = std::fs::remove_dir_all(&root);
}
