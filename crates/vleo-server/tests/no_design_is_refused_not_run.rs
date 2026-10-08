//! Where the tool finds no design at all — no design named, no design's files
//! and no folders — it refuses, saying where it looked. It holds no design of
//! its own to run instead: the engine before a design opens has no rows.
//!
//! One test, alone in its binary, because which design opens is read from the
//! environment the whole process shares.

use vleo_modules::{engine, EMPTY};

#[test]
fn a_root_with_no_design_is_refused_and_the_engine_holds_nothing() {
    let empty = std::env::temp_dir().join(format!("vleo-no-design-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&empty);
    std::fs::create_dir_all(&empty).unwrap();
    std::env::remove_var("VLEO_DESIGN");
    std::env::remove_var("VLEO_DRIVE");

    let said = vleo_server::run_the_design(Some(empty.clone()))
        .expect_err("a root with no design was run");
    assert!(said.contains("no design where the tool looked"), "{said}");
    assert!(said.contains(&empty.display().to_string()), "{said}");

    // Nothing was installed in its place.
    assert!(std::ptr::eq(engine(), &EMPTY));
    assert!(vleo_modules::nodes().is_empty());
    let _ = std::fs::remove_dir_all(&empty);
}
