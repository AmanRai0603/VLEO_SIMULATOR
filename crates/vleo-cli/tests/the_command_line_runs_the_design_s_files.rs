//! The command line runs the graph read from the design's files, says so, and
//! refuses — rather than runs the graph compiled into it — when the design
//! does not open.

use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn vleo(scratch: &Path, design: Option<&Path>, args: &[&str]) -> (bool, String) {
    vleo_on(scratch, design, None, args)
}

fn vleo_on(
    scratch: &Path,
    design: Option<&Path>,
    drive: Option<&Path>,
    args: &[&str],
) -> (bool, String) {
    let mut c = Command::new(env!("CARGO_BIN_EXE_vleo"));
    c.args(args)
        .current_dir(root())
        .env("VLEO_DATA", scratch.join("data"))
        .env("VLEO_CASE", scratch.join("case.csv"))
        .env("VLEO_LOG", scratch.join("log"))
        .env("VLEO_RESULTS", scratch.join("results"))
        .env_remove("VLEO_DESIGN")
        .env_remove("VLEO_DRIVE");
    if let Some(d) = design {
        c.env("VLEO_DESIGN", d);
    }
    if let Some(d) = drive {
        c.env("VLEO_DRIVE", d);
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

#[test]
fn the_command_line_says_which_releases_today_s_design_is_built_from() {
    let s = scratch("drive");
    let rel = s.join("drive/groups/l3_solar/releases");
    std::fs::create_dir_all(&rel).unwrap();
    for v in ["1.0", "1.1"] {
        std::fs::copy(
            root().join(format!(
                "crates/vleo-files/tests/fixtures/l3_solar-{v}.vleo"
            )),
            rel.join(format!("l3_solar-{v}.vleo")),
        )
        .unwrap();
    }
    let (ok, said) = vleo_on(&s, None, Some(&s.join("drive")), &["version"]);
    assert!(ok, "{said}");
    assert!(
        said.contains("today's design, built from the drive"),
        "{said}"
    );
    assert!(said.contains("l3_solar 1.1 — its latest"), "{said}");
    // A folder that is not a drive is refused, never shown as today's design.
    let (ok, said) = vleo_on(
        &s,
        None,
        Some(&s),
        &["run", "sw_activity_band", "--defaults"],
    );
    assert!(!ok, "{said}");
    assert!(
        said.contains("is not a drive: it has no groups/ folder"),
        "{said}"
    );
}
