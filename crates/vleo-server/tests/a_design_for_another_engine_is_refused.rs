//! A design file is only run by the engine built from it.
//!
//! The kit carries `design.vleo` beside a tool whose relations are compiled
//! in. A design file from another release still opens and still answers —
//! with relations its sheets do not state. So the tool compares every row of
//! the file with the engine before it serves anything, and refuses a file that
//! differs, naming the rows.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn a_design_file_changed_in_one_row_is_refused_by_name() {
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

    // The file as written is this engine's design: it opens.
    std::env::set_var("VLEO_DESIGN", &file);
    vleo_server::serve(Some(root()), 18971, false, true)
        .expect("a design file of this engine was refused");

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

    let refused = vleo_server::serve(Some(root()), 18991, false, true)
        .expect_err("a design file for another engine was served");
    std::env::remove_var("VLEO_DESIGN");
    assert!(refused.contains("different engine"), "{refused}");
    assert!(
        refused.contains("sw_activity_band: a different sheet"),
        "{refused}"
    );
    let _ = std::fs::remove_dir_all(&scratch);
}
