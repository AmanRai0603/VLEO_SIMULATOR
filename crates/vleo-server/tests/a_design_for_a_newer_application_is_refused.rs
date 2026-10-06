//! A design names the oldest application that can run it, and an application
//! older than that refuses it, by name, before anything is read from it.
//!
//! One test, alone in its binary, because which design opens is read from the
//! environment the whole process shares.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn needs(file: &Path, version: &str) {
    let db = rusqlite::Connection::open(file).unwrap();
    db.execute(
        "UPDATE meta SET value = ?1 WHERE key = 'oldest_application'",
        [version],
    )
    .unwrap();
}

#[test]
fn a_design_names_the_oldest_application_that_can_run_it() {
    let ours = env!("CARGO_PKG_VERSION");
    let scratch = std::env::temp_dir().join(format!("vleo-design-app-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    for (k, v) in [
        ("VLEO_CASE", "case.csv"),
        ("VLEO_RESULTS", "results"),
        ("VLEO_LOG", "log"),
        ("VLEO_DATA", "data"),
    ] {
        std::env::set_var(k, scratch.join(v));
    }
    std::env::remove_var("VLEO_DRIVE");
    let file = scratch.join("design.vleo");
    let stamp = vleo_design::Stamp {
        tool: ours.into(),
        ..Default::default()
    };
    vleo_design::write(&root(), &file, &stamp).unwrap();
    std::env::set_var("VLEO_DESIGN", &file);

    // Written by this application, it names this application, and opens.
    let d = vleo_design::Design::open(&file, &root()).unwrap();
    assert_eq!(d.meta("oldest_application"), ours);
    drop(d);
    vleo_server::run_the_design(Some(root())).expect("this application's own design");

    // One that needs a later application is refused, saying which.
    needs(&file, "99.0.0");
    let e =
        vleo_server::run_the_design(Some(root())).expect_err("a design for a later application");
    assert!(
        e.contains(&format!(
            "needs vleo 99.0.0 or later, and this is vleo {ours}"
        )),
        "{e}"
    );
    // An older one is no bar; a version that is not one is refused by what it
    // says.
    needs(&file, "0.0.1");
    vleo_server::run_the_design(Some(root())).expect("a design an older application could run");
    needs(&file, "soon");
    let e = vleo_server::run_the_design(Some(root())).expect_err("a version that is not one");
    assert!(e.contains("'soon' is not an application's version"), "{e}");
    std::env::remove_var("VLEO_DESIGN");

    // Compared as numbers, part by part: 0.10 is later than 0.9.
    assert_eq!(vleo_design::runs_on("0.10.0", "0.9.9"), Ok(false));
    assert_eq!(vleo_design::runs_on("0.9.9", "0.10.0"), Ok(true));
    assert_eq!(vleo_design::runs_on("1.0.0", "1.0.0"), Ok(true));
    let _ = std::fs::remove_dir_all(&scratch);
}
