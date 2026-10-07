//! The tool opens the design converted to its files, and runs the engine on
//! the graph read from them (docs/PLAN_1_0.md, phase E): `VLEO_DESIGN` names
//! the folder the group, node and case files are in, laid out as the shared
//! drive holds them. A file that states a row other than this engine answers
//! it is refused, by name, as a design file is.
//!
//! The faces name what the engine answers from the graph that runs, so a
//! design with rows this build has not, the open blocks the conversion adds
//! where a breakdown holds none, runs, each of them under its own name.
//!
//! One test, alone in its binary, because which design opens is read from the
//! environment the whole process shares.

use std::path::{Path, PathBuf};

use vleo_files::convert;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn write_all(files: &[(String, vleo_files::model::File)], dir: &Path) {
    let _ = std::fs::remove_dir_all(dir);
    for (path, f) in files {
        let p = dir.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        vleo_files::sqlite::write(f, &p).unwrap();
    }
}

#[test]
fn the_engine_runs_the_design_from_its_files_and_refuses_one_it_does_not_answer() {
    let scratch = std::env::temp_dir().join(format!("vleo-converted-run-{}", std::process::id()));
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
    let tree = vleo_sheet::load::load_all(&root()).unwrap();
    let files = convert::convert(&tree, &vleo_sheet::files::Disk, "vleo test").unwrap();
    let added: Vec<String> = convert::PROPOSED
        .iter()
        .flat_map(|(g, n)| n.iter().map(move |(name, _)| convert::proposed_id(g, name)))
        .collect();
    let dir = scratch.join("design");

    // With the blocks the conversion adds: run, and every one of them named.
    write_all(&files, &dir);
    std::env::set_var("VLEO_DESIGN", &dir);
    let said = vleo_server::run_the_design(Some(root())).expect("the converted design runs");
    assert!(
        said.contains(&format!("{} rows", tree.sheets.len() + added.len())),
        "{said}"
    );
    for id in &added {
        assert!(vleo_modules::Vleo::find(id).is_some(), "{id} is not named");
    }

    // The rows that were there: run from their files.
    let files: Vec<_> = files
        .into_iter()
        .filter(|(p, _)| !added.iter().any(|id| p.ends_with(&format!("/{id}.vnode"))))
        .collect();
    write_all(&files, &dir);
    let said = vleo_server::run_the_design(Some(root())).expect("the converted design runs");
    assert!(
        said.contains(&format!("{} rows", tree.sheets.len())),
        "{said}"
    );

    // One row's method changed in its file: not the design this engine
    // answers, and refused, naming the row.
    let mut changed = files.clone();
    let (_, f) = changed
        .iter_mut()
        .find(|(p, _)| p.ends_with("/sw_f107_design_long.vnode"))
        .unwrap();
    let m = f.texts.iter_mut().find(|t| t.kind == "method").unwrap();
    m.body = m.body.replace("const z = 1.28 [1]", "const z = 1.29 [1]");
    write_all(&changed, &dir);
    let e =
        vleo_server::run_the_design(Some(root())).expect_err("a row this engine does not answer");
    assert!(
        e.contains("was made for a different engine")
            && e.contains("sw_f107_design_long: a different sheet"),
        "{e}"
    );

    // A folder with nothing of the design in it is refused, saying so.
    let empty = scratch.join("empty");
    std::fs::create_dir_all(&empty).unwrap();
    std::env::set_var("VLEO_DESIGN", &empty);
    let e = vleo_server::run_the_design(Some(root())).expect_err("an empty folder");
    assert!(e.contains("holds no group, node or case file"), "{e}");
    let _ = std::fs::remove_dir_all(&scratch);
}
