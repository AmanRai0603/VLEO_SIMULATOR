//! The group folder's checks, said by the library as the page says them.
//!
//! The page's own checker, `web/js/gcheck.js`, is the oracle: on Solar 1.0 it
//! gives 80 findings and on Solar 1.1 it gives 74 (its maths warnings aside),
//! none of them an error — `tools/files_check.mjs` holds the library to it
//! finding for finding, in CI. Here the same counts are held in `cargo test`,
//! and a handful of folders broken on purpose are refused in the page's words.

use std::path::Path;

use vleo_files::folder::{self, Level, Spec};
use vleo_files::format_1::Folder;
use vleo_files::sqlite;

fn solar(version: &str) -> Folder {
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/fixtures/l3_solar-{version}.vleo"));
    sqlite::read_format_1(&p).unwrap().folder()
}

fn findings(f: &Folder) -> Vec<folder::Finding> {
    folder::check(f, &Spec::carried().unwrap())
}

#[test]
fn solar_as_the_page_checked_it() {
    for (v, count) in [("1.0", 80), ("1.1", 74)] {
        let found = findings(&solar(v));
        assert_eq!(found.len(), count, "Solar {v}: {found:#?}");
        assert!(found.iter().all(|f| f.level != Level::Error), "Solar {v}");
    }
}

fn has(found: &[folder::Finding], level: Level, place: &str, msg: &str) -> bool {
    found
        .iter()
        .any(|f| f.level == level && f.place == place && f.msg.contains(msg))
}

fn text(f: &mut Folder, path: &str, how: impl Fn(&str) -> String) {
    let t = String::from_utf8(f[path].clone()).unwrap();
    f.insert(path.to_string(), how(&t).into_bytes());
}

#[test]
fn a_broken_folder_is_refused_in_the_page_s_words() {
    let mut f = solar("1.1");
    text(&mut f, "nodes/sw_regime/explanation.md", |t| {
        format!("{t}\n{{{{eq NOPE}}}} {{{{bogus x}}}} {{{{node nope}}}} {{{{guess q}}}}\n")
    });
    text(&mut f, "nodes/sw_regime/theory.md", |t| {
        t.replace("## Validity", "## Later")
    });
    f.remove("nodes/sw_ap_design/theory.md");
    f.insert("notes.xlsx".into(), vec![0]);
    text(&mut f, "nodes.csv", |t| {
        t.replacen("sw_band_confidence,", "Sw Band,", 1)
    });
    let found = findings(&f);
    for (level, place, msg) in [
        (
            Level::Error,
            "nodes/sw_regime/explanation.md",
            "{{eq NOPE}} names no equation in equations.csv",
        ),
        (
            Level::Error,
            "nodes/sw_regime/explanation.md",
            "{{bogus}} is not one of eq, fig, node, guess",
        ),
        (
            Level::Error,
            "nodes/sw_regime/explanation.md",
            "{{node nope}} names no node",
        ),
        (
            Level::Error,
            "nodes/sw_regime/explanation.md",
            "{{guess …}} needs the question, then ||, then the answer",
        ),
        (
            Level::Error,
            "nodes/sw_regime/theory.md",
            "has no \"## Validity\" section",
        ),
        (
            Level::Error,
            "nodes/sw_ap_design/theory.md",
            "is missing — ",
        ),
        (
            Level::Error,
            "notes.xlsx",
            "is a .xlsx — save a spreadsheet as CSV",
        ),
        (
            Level::Error,
            "nodes.csv",
            "the id \"Sw Band\" is not lower case letters",
        ),
        (
            Level::Warning,
            "nodes/sw_band_confidence/",
            "is a folder nodes.csv does not list",
        ),
    ] {
        assert!(has(&found, level, place, msg), "{place}: {msg}\n{found:#?}");
    }
}

#[test]
fn a_table_that_does_not_parse_says_where() {
    let mut f = solar("1.1");
    text(&mut f, "requirements.csv", |t| format!("{t}one,two\n"));
    let found = findings(&f);
    assert!(
        found.iter().any(|x| x.level == Level::Error
            && x.place == "requirements.csv"
            && x.msg == "has 2 fields where the header has 5"
            && x.line == 3),
        "{found:#?}"
    );
}
