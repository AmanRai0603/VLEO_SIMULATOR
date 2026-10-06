//! `vleo version` names the design it runs and what it answers, each by its
//! fingerprint — the two lines CI compares across Linux, Windows and macOS,
//! today's design built on each from the same drive.

use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The `design` and `answers` lines `vleo version` prints, on `drive`.
fn fingerprints(scratch: &Path, drive: Option<&Path>) -> (String, String) {
    let mut c = Command::new(env!("CARGO_BIN_EXE_vleo"));
    c.arg("version")
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
    let said = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        out.status.success(),
        "{said}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let line = |k: &str| {
        said.lines()
            .find(|l| l.trim_start().starts_with(k))
            .unwrap_or_else(|| panic!("no {k} line in: {said}"))
            .trim()
            .to_string()
    };
    (line("design "), line("answers "))
}

fn sql(file: &Path, statement: &str) {
    let db = rusqlite::Connection::open(file).unwrap();
    assert!(db.execute(statement, []).unwrap() > 0, "{statement}");
}

#[test]
fn today_s_design_is_named_by_its_fingerprint() {
    let s = std::env::temp_dir().join(format!("vleo-cli-fingerprint-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&s);
    let rel = s.join("drive/groups/l3_solar/releases");
    std::fs::create_dir_all(&rel).unwrap();
    let fixture = root().join("crates/vleo-files/tests/fixtures/l3_solar-1.1.vleo");
    std::fs::copy(&fixture, rel.join("l3_solar-1.1.vleo")).unwrap();

    let own = fingerprints(&s, None);
    assert!(
        own.0.len() > "design ".len() && own.1.contains("solar-weather"),
        "{own:?}"
    );
    // Solar 1.1 is the design's own: built from the drive, the same design,
    // answering alike.
    assert_eq!(fingerprints(&s, Some(&s.join("drive"))), own);

    // Solar 1.2, sealed, with a method changed where its cases do not look:
    // another design, answering otherwise.
    let f = rel.join("l3_solar-1.2.vleo");
    std::fs::copy(&fixture, &f).unwrap();
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
    let changed = fingerprints(&s, Some(&s.join("drive")));
    assert_ne!(changed.0, own.0);
    assert_ne!(changed.1, own.1);
    let _ = std::fs::remove_dir_all(&s);
}
