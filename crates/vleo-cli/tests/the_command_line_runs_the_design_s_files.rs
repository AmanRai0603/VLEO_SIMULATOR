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
    let missing = s.join("not-here");
    for args in [
        &["run", "sw_activity_band", "--defaults"][..],
        &["version"][..],
        // A figure is drawn from the engine's answers, so from the design too.
        &["figure", "closure", "pair=04"][..],
    ] {
        let (ok, said) = vleo(&s, Some(&missing), args);
        assert!(!ok, "{args:?} ran with no design: {said}");
        assert!(
            said.contains("is not a folder of the design's files"),
            "{said}"
        );
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
fn a_figure_is_drawn_from_the_design_s_files() {
    // The sustained Ap requirement raised from 48 to 60 in a copy of the
    // design: the closure's figure is drawn from that design, not from the
    // graph compiled into the program.
    let s = scratch("figure");
    let design = s.join("design");
    copy(&root().join("design"), &design);
    let node = design.join("groups/l3_solar/nodes/l3_solar_req_04.vnode");
    let db = rusqlite::Connection::open(&node).unwrap();
    let changed = db
        .execute(
            "UPDATE port SET value = '60.0' WHERE block_uid = 'l3_solar_req_04' AND direction = 'out'",
            [],
        )
        .unwrap();
    assert_eq!(changed, 1, "the requirement is not in its file");
    drop(db);
    let req = |design: Option<&Path>| {
        let (ok, said) = vleo(&s, design, &["figure", "closure", "pair=04"]);
        assert!(ok, "{said}");
        let at = said.find("\"req\":").unwrap_or_else(|| panic!("{said}")) + 6;
        let v: String = said[at..]
            .chars()
            .take_while(|c| !matches!(c, ',' | '}'))
            .collect();
        v.trim().parse::<f64>().unwrap()
    };
    assert_eq!(req(None), 48.0);
    assert_eq!(req(Some(&design)), 60.0);
    let _ = std::fs::remove_dir_all(&s);
}
