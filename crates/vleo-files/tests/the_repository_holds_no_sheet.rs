//! The design is `design/`, and nothing beside it is (docs/PLAN_1_0.md,
//! phase E). The sheets it was converted from are gone: no node folder, no
//! `layers/`, no `node.toml` anywhere in the repository, and no case or list
//! of sources beside the ones `design/` holds. One that came back
//! would be read by nothing and believed by whoever opened it — a second
//! design beside the first, and the one that looks editable.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every path under `dir` with one of `names` as its file or folder name,
/// skipping build output and version control.
fn found(dir: &Path, names: &[&str], out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().to_string();
        if matches!(name.as_str(), "target" | ".git" | "node_modules") {
            continue;
        }
        if names.contains(&name.as_str()) {
            out.push(p.clone());
        }
        if p.is_dir() {
            found(&p, names, out);
        }
    }
}

#[test]
fn no_sheet_and_no_folder_of_them_is_left_beside_the_design() {
    let root = root();
    let mut left = Vec::new();
    found(&root, &["node.toml", "layers"], &mut left);
    if let Ok(crates) = std::fs::read_dir(root.join("crates")) {
        for c in crates.flatten() {
            if c.file_name().to_string_lossy().starts_with("vleo-mod-") {
                left.push(c.path());
            }
        }
    }
    for kept in ["sources/sources.toml", "cases/multipayload.toml"] {
        if root.join(kept).exists() {
            left.push(root.join(kept));
        }
    }
    assert!(
        left.is_empty(),
        "the design is design/, and these are sheets or their folders beside it: {left:?}"
    );
    // And the design is there: what this guards is a repository with one.
    let (tree, _) = vleo_files::convert::open(&root).expect("design/ is the design");
    assert!(!tree.sheets.is_empty());
}
