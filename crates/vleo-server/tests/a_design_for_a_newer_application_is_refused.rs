//! Each file of a design names the application that wrote it, and an
//! application older than that refuses the design, by name, before anything
//! is read from it.
//!
//! One test, alone in its binary, because which design opens is read from the
//! environment the whole process shares.

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

fn written_by(file: &Path, app: &str) {
    let db = rusqlite::Connection::open(file).unwrap();
    let n = db
        .execute(
            "UPDATE meta SET value = ?1 WHERE key = 'written_by_app'",
            [app],
        )
        .unwrap();
    assert_eq!(n, 1, "{} names no application", file.display());
}

#[test]
fn a_design_written_by_a_newer_application_is_refused_by_name() {
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
    let design = scratch.join("design");
    copy(&root().join("design"), &design);
    std::env::set_var("VLEO_DESIGN", &design);
    let one = design.join("groups/l3_solar/nodes/sw_activity_band.vnode");

    // Written by this application, it opens.
    written_by(&one, &format!("vleo {ours}"));
    vleo_server::run_the_design(Some(root())).expect("this application's own design");

    // One file written by a later application, and the design is refused,
    // saying which file and which application.
    written_by(&one, "vleo 99.0.0");
    let e =
        vleo_server::run_the_design(Some(root())).expect_err("a design from a later application");
    assert!(
        e.contains("sw_activity_band.vnode was written by vleo 99.0.0, and this is vleo")
            && e.contains(ours),
        "{e}"
    );
    // An older one is no bar; a version that is not one is refused by what it
    // says.
    written_by(&one, "vleo 0.0.1");
    vleo_server::run_the_design(Some(root())).expect("a design an older application wrote");
    written_by(&one, "soon");
    let e = vleo_server::run_the_design(Some(root())).expect_err("a version that is not one");
    assert!(e.contains("'soon' is not an application's version"), "{e}");

    // A design named that is not a folder of its files is refused, never read
    // past to the design beside the tool.
    std::env::set_var("VLEO_DESIGN", scratch.join("design.vleo"));
    let e = vleo_server::run_the_design(Some(root())).expect_err("a design that is not a folder");
    assert!(e.contains("is not a folder of the design's files"), "{e}");
    std::env::remove_var("VLEO_DESIGN");

    // Compared as numbers, part by part: 0.10 is later than 0.9.
    assert_eq!(vleo_server::runs_on("0.10.0", "0.9.9"), Ok(false));
    assert_eq!(vleo_server::runs_on("0.9.9", "0.10.0"), Ok(true));
    assert_eq!(vleo_server::runs_on("1.0.0", "1.0.0"), Ok(true));
    let _ = std::fs::remove_dir_all(&scratch);
}
