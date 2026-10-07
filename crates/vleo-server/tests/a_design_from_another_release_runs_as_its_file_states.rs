//! A design file from another release runs, as its file states it.
//!
//! The tool holds no relation of the design in code: every row is a method,
//! a stated value, a table or its children, read from the design's own files,
//! and a method the tool was not built from runs in the interpreter. So a
//! design file other than the one the tool was released with is not refused
//! for differing from it: it runs, and what it serves is the file's.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn a_design_file_changed_in_one_row_runs_and_serves_the_change() {
    let scratch = std::env::temp_dir().join(format!("vleo-design-engine-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    std::env::set_var("VLEO_CASE", scratch.join("case.csv"));
    std::env::set_var("VLEO_RESULTS", scratch.join("results"));
    std::env::set_var("VLEO_LOG", scratch.join("log"));
    std::env::set_var("VLEO_DATA", scratch.join("data"));
    let file = scratch.join("design.vleo");
    vleo_design::write(&root(), &file, &vleo_design::Stamp::default())
        .expect("the design file was not written");

    // The file as written opens.
    std::env::set_var("VLEO_DESIGN", &file);
    vleo_server::serve(Some(root()), 18971, false, true)
        .expect("the design file as written was refused");

    // One row's question changed, as another release's design would have it.
    let node = "crates/vleo-mod-solar/nodes/sw_activity_band/node.toml";
    let db = rusqlite::Connection::open(&file).unwrap();
    let bytes: Vec<u8> = db
        .query_row("SELECT bytes FROM file WHERE path = ?1", [node], |r| {
            r.get(0)
        })
        .expect("the row is not in the design file");
    let text = String::from_utf8(bytes).unwrap();
    let changed = text.replacen(
        "Which activity band does this F10.7 fall in?",
        "Which activity band does this F10.7 fall in, as another release asks it?",
        1,
    );
    assert_ne!(text, changed, "the question was not found to change");
    db.execute(
        "UPDATE file SET bytes = ?1 WHERE path = ?2",
        rusqlite::params![changed.into_bytes(), node],
    )
    .unwrap();
    drop(db);

    vleo_server::serve(Some(root()), 18991, false, true)
        .expect("a design file from another release was refused");
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
