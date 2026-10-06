//! `vleo health` says where exactly the design breaks, and `--trace` walks a
//! closure down to the row that causes it — on the design's own files, and on
//! today's design built from a drive whose release breaks a closure.

use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn vleo(scratch: &Path, drive: Option<&Path>, args: &[&str]) -> (bool, String) {
    let mut c = Command::new(env!("CARGO_BIN_EXE_vleo"));
    c.args(args)
        .current_dir(root())
        .env("VLEO_DATA", scratch.join("data"))
        .env("VLEO_CASE", scratch.join("case.csv"))
        .env("VLEO_LOG", scratch.join("log"))
        .env("VLEO_RESULTS", scratch.join("results"))
        .env_remove("VLEO_DESIGN")
        .env_remove("VLEO_DRIVE");
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
    let s = std::env::temp_dir().join(format!("vleo-cli-health-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&s);
    std::fs::create_dir_all(&s).unwrap();
    s
}

fn sql(file: &Path, statement: &str) {
    let db = rusqlite::Connection::open(file).unwrap();
    assert!(db.execute(statement, []).unwrap() > 0, "{statement}");
}

#[test]
fn the_design_s_own_files_are_open_and_their_solar_closures_close() {
    let s = scratch("own");
    let (ok, said) = vleo(&s, None, &["health", "--defaults"]);
    assert!(ok, "{said}");
    assert!(said.contains("the spacecraft\x1b[0m \x1b[2mopen"), "{said}");
    for c in 1..=5 {
        assert!(
            said.lines()
                .any(|l| l.contains(&format!("l3_solar_ach_0{c}")) && l.contains("closes")),
            "{said}"
        );
    }
    assert!(said.contains("no margin policy"), "{said}");

    // A closure traced names what moves it, with the level it would close
    // from; a row that is not a closure is refused, by name.
    let (ok, said) = vleo(
        &s,
        None,
        &["health", "--defaults", "--trace", "l3_solar_ach_01"],
    );
    assert!(ok, "{said}");
    assert!(
        said.contains("no row it reads is refused, open or unproven"),
        "{said}"
    );
    assert!(
        said.lines()
            .any(|l| l.contains("l3_solar_req_01") && l.contains("closes from")),
        "{said}"
    );
    let (ok, said) = vleo(&s, None, &["health", "--trace", "sw_f107_design_long"]);
    assert!(!ok, "{said}");
    assert!(
        said.contains("sw_f107_design_long is not a closure"),
        "{said}"
    );
}

#[test]
fn a_release_that_breaks_a_closure_is_traced_by_name_from_it() {
    // Solar 1.2, sealed: the hot edge of the sustained F10.7 band with a
    // multiplier from another table below a centre of 95 sfu, where none of
    // the node's own cases look.
    let s = scratch("drive");
    let rel = s.join("drive/groups/l3_solar/releases");
    std::fs::create_dir_all(&rel).unwrap();
    let f = rel.join("l3_solar-1.2.vleo");
    std::fs::copy(
        root().join("crates/vleo-files/tests/fixtures/l3_solar-1.1.vleo"),
        &f,
    )
    .unwrap();
    sql(&f, "UPDATE meta SET value = '1.2' WHERE key = 'version'");
    sql(
        &f,
        "UPDATE tbl SET csv = csv || '1.2,2026-10-06,Aman Rai,\
         The band needed a wider margin when the cycle is low.,\
         The multiplier was read again.,\
         A wider band is safer.,\
         The multiplier of the hot edge.,,,\
         the one-sided 90th percentile,a multiplier from another table\n' \
         WHERE scope = 'group' AND path = 'versions.csv'",
    );
    sql(
        &f,
        "UPDATE doc SET body = replace(body, 'return sustained', \
         'if central < 95 then\n  return central + spread * 14\nend\nreturn sustained') \
         WHERE kind = 'pseudocode' AND scope = 'sw_f107_design_long'",
    );
    let r = vleo_files::intake::Release::open(&f).unwrap();
    let fp = vleo_files::seal::fingerprint_of(&r.folder, "group");
    sql(
        &f,
        &format!("UPDATE meta SET value = '{fp}' WHERE key = 'fingerprint'"),
    );
    let drive = s.join("drive");

    let (ok, said) = vleo(&s, Some(&drive), &["health", "--defaults"]);
    assert!(ok, "{said}");
    assert!(said.contains("l3_solar 1.2 — its latest"), "{said}");
    assert!(
        said.contains("the spacecraft\x1b[0m \x1b[31mfails"),
        "{said}"
    );
    assert!(
        said.lines()
            .any(|l| l.contains("l3_solar_ach_01") && l.contains("fails")),
        "{said}"
    );

    let (ok, said) = vleo(
        &s,
        Some(&drive),
        &["health", "--defaults", "--trace", "l3_solar_ach_01"],
    );
    assert!(ok, "{said}");
    let cause = said
        .lines()
        .skip_while(|l| !l.contains("caused by, nearest first"))
        .nth(1)
        .unwrap_or_default();
    assert!(
        cause.contains("sw_f107_design_long")
            && cause.contains("l3_solar · environment")
            && cause.contains("changed by l3_solar 1.2"),
        "{said}"
    );
}
