//! The tool opens the design converted to its files, and runs the engine on
//! the graph read from them (docs/PLAN_1_0.md, phase E): `VLEO_DESIGN` names
//! the folder the group, node and case files are in, laid out as the shared
//! drive holds them. A design other than the one this tool was released with
//! runs as its files state it: a changed method answers by its own method, and
//! a stated value outside its bounds is refused under its row's name.
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
fn the_engine_runs_the_design_from_its_files_as_they_state_it() {
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
    // Written by this application: one a later application wrote is refused.
    let app = format!("vleo {}", env!("CARGO_PKG_VERSION"));
    let files = convert::convert(&tree, &vleo_sheet::files::Disk, &app).unwrap();
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

    // The answer one row gives at its first case, on the graph that runs.
    let at_its_case = |id: &str| {
        let k = vleo_modules::Vleo::find(id).expect("the row is in the design");
        let inputs = vleo_modules::nodes()[k as usize].fixtures[0]
            .inputs
            .to_vec();
        vleo_modules::probe(k, &inputs).map(|o| o[0]).ok()
    };
    let before = at_its_case("sw_f107_design_long");

    // One row's method changed in its file: a design other than the one this
    // tool was released with. It runs, and the row answers by its own method.
    let mut changed = files.clone();
    let (_, f) = changed
        .iter_mut()
        .find(|(p, _)| p.ends_with("/sw_f107_design_long.vnode"))
        .unwrap();
    let m = f.texts.iter_mut().find(|t| t.kind == "method").unwrap();
    assert!(m.body.contains("const z = 1.28 [1]"));
    m.body = m.body.replace("const z = 1.28 [1]", "const z = 1.29 [1]");
    write_all(&changed, &dir);
    vleo_server::run_the_design(Some(root())).expect("a design with a method of its own");
    let after = at_its_case("sw_f107_design_long");
    assert!(
        before.is_some() && after.is_some() && before != after,
        "the changed method did not answer: {before:?} then {after:?}"
    );

    // One stated value changed in its file to below its bound: the design
    // opens, and the row refuses, under its own name; nothing is published.
    let mut low = files.clone();
    let (_, f) = low
        .iter_mut()
        .find(|(p, _)| p.ends_with("/com_frequency.vnode"))
        .unwrap();
    let port = f
        .ports
        .iter_mut()
        .find(|p| p.block_uid == "com_frequency" && p.direction == "out")
        .unwrap();
    assert_eq!(port.value, "8.2");
    port.value = "0.01".into();
    write_all(&low, &dir);
    vleo_server::run_the_design(Some(root())).expect("a design with a value of its own");
    let k = vleo_modules::Vleo::find("com_frequency").unwrap();
    let e = format!(
        "{:?}",
        vleo_modules::probe(k, &[]).expect_err("a stated value below its bound")
    );
    assert!(
        e.contains("OutOfDomain") && e.contains("com_frequency"),
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
