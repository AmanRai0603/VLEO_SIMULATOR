//! A design from another release runs, as its files state it.
//!
//! The tool holds no relation of the design in code: every row is a method,
//! a stated value, a table or its children, read from the design's own files,
//! and a method the tool was not built from runs in the interpreter. So a
//! design other than the one the tool was released with is not refused for
//! differing from it: it runs, and what it serves is its files'.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let p = e.unwrap().path();
        let dest = to.join(p.file_name().unwrap());
        if p.is_dir() {
            copy(&p, &dest);
        } else {
            std::fs::copy(&p, &dest).unwrap();
        }
    }
}

#[test]
fn a_design_changed_in_one_row_runs_and_serves_the_change() {
    let scratch = std::env::temp_dir().join(format!("vleo-design-engine-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    std::env::set_var("VLEO_CASE", scratch.join("case.csv"));
    std::env::set_var("VLEO_RESULTS", scratch.join("results"));
    std::env::set_var("VLEO_LOG", scratch.join("log"));
    std::env::set_var("VLEO_DATA", scratch.join("data"));
    std::env::remove_var("VLEO_DRIVE");
    let design = scratch.join("design");
    copy(&root().join("design"), &design);

    // The design as committed opens.
    std::env::set_var("VLEO_DESIGN", &design);
    vleo_server::serve(Some(root()), 18971, false, true)
        .expect("the design as committed was refused");

    // One row's question changed, as another release's design would have it.
    let node = design.join("groups/l3_solar/nodes/sw_activity_band.vnode");
    let db = rusqlite::Connection::open(&node).unwrap();
    let changed = db
        .execute(
            "UPDATE block SET question = ?1 WHERE id = 'sw_activity_band'",
            ["Which activity band does this F10.7 fall in, as another release asks it?"],
        )
        .unwrap();
    assert_eq!(changed, 1, "the row is not in its file");
    drop(db);

    vleo_server::serve(Some(root()), 18991, false, true)
        .expect("a design from another release was refused");
    std::env::remove_var("VLEO_DESIGN");
    let k = vleo_modules::Vleo::find("sw_activity_band").expect("the row runs");
    assert!(
        vleo_modules::nodes()[k as usize]
            .question
            .contains("as another release asks it"),
        "the row served is not the file's"
    );
    let _ = std::fs::remove_dir_all(&scratch);
}
