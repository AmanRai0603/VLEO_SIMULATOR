//! The command line runs the graph read from the design's files, says so, and
//! refuses — rather than runs the graph compiled into it — when the design
//! does not open.

use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn vleo(scratch: &Path, design: Option<&Path>, args: &[&str]) -> (bool, String) {
    let mut c = Command::new(env!("CARGO_BIN_EXE_vleo"));
    c.args(args)
        .current_dir(root())
        .env("VLEO_DATA", scratch.join("data"))
        .env("VLEO_CASE", scratch.join("case.csv"))
        .env("VLEO_LOG", scratch.join("log"))
        .env("VLEO_RESULTS", scratch.join("results"))
        .env_remove("VLEO_DESIGN");
    if let Some(d) = design {
        c.env("VLEO_DESIGN", d);
    }
    let out = c.output().expect("vleo did not start");
    (
        out.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

fn scratch(name: &str) -> PathBuf {
    let s = std::env::temp_dir().join(format!("vleo-cli-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&s);
    std::fs::create_dir_all(&s).unwrap();
    s
}

#[test]
fn the_command_line_says_it_runs_the_graph_read_from_the_design_s_files() {
    let s = scratch("engine");
    let (ok, said) = vleo(&s, None, &["version"]);
    assert!(ok, "{said}");
    assert!(
        said.contains("engine the graph read from the design's files"),
        "{said}"
    );
    let (ok, said) = vleo(&s, None, &["run", "sw_activity_band", "--defaults"]);
    assert!(ok, "{said}");
}

#[test]
fn a_design_that_does_not_open_is_refused_not_run_as_the_compiled_one() {
    let s = scratch("refused");
    let missing = s.join("not-here.vleo");
    for args in [
        &["run", "sw_activity_band", "--defaults"][..],
        &["version"][..],
    ] {
        let (ok, said) = vleo(&s, Some(&missing), args);
        assert!(!ok, "{args:?} ran with no design: {said}");
        assert!(said.contains("the design file does not open"), "{said}");
    }
}
