//! The design file is the tree: written from it, it reads back as it.
//!
//! The kit carries the design as one file and the daemon reads that in place
//! of the folders, so anything the file loses or changes is something a team
//! sees and a developer does not. These hold the two to each other: the same
//! files, byte for byte; the same tree, loaded by the same loader; a file
//! changed after it was written caught; and a file that is not a design file
//! refused by name.

use std::path::{Path, PathBuf};
use vleo_design::{Design, ErrorKind, Stamp};
use vleo_sheet::files::Files;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn scratch(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("vleo-design-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn written(dir: &Path) -> PathBuf {
    let out = dir.join("design.vleo");
    let stamp = Stamp {
        tool: "test".into(),
        commit: String::new(),
        built: "2026-10-03".into(),
    };
    let w = vleo_design::write(&root(), &out, &stamp).expect("the design file was not written");
    assert!(w.rows > 1000, "only {} rows written", w.rows);
    assert!(w.files > w.rows, "only {} files written", w.files);
    out
}

#[test]
fn it_holds_every_file_and_loads_as_the_same_tree() {
    let dir = scratch("same");
    let out = written(&dir);
    let d = Design::open(&out, &root()).expect("the design file does not open");
    d.verify().expect("the file is not what it says");
    let diff = d.compare(&root()).expect("the comparison did not run");
    assert!(
        diff.is_empty(),
        "the file is not the tree:\n{}",
        diff.join("\n")
    );

    let folders = vleo_sheet::load_all(&root()).expect("the folders do not load");
    let file = vleo_sheet::load::load_all_from(&d, &root()).expect("the file does not load");
    assert_eq!(
        folders.sheets.keys().collect::<Vec<_>>(),
        file.sheets.keys().collect::<Vec<_>>(),
        "a different set of rows"
    );
    for (id, a) in &folders.sheets {
        let b = &file.sheets[id];
        assert_eq!(a.sheet_hash, b.sheet_hash, "{id}: a different sheet");
        assert_eq!(a.impl_hash, b.impl_hash, "{id}: different hole bodies");
        assert_eq!(
            a.fixtures.len(),
            b.fixtures.len(),
            "{id}: different fixtures"
        );
        // The page a reader opens is rendered from the sheet; from the file
        // it must be the page the folders render, byte for byte.
        assert_eq!(
            vleo_sheet::page::fragment(a, &vleo_sheet::load::read_holes(&a.dir), &folders),
            vleo_sheet::page::fragment(b, &vleo_sheet::load::read_holes_in(&d, &b.dir), &file),
            "{id}: a different page"
        );
    }
    assert_eq!(folders.groups.len(), file.groups.len(), "different layers");
    assert_eq!(
        folders.cases.keys().collect::<Vec<_>>(),
        file.cases.keys().collect::<Vec<_>>(),
        "different cases"
    );
    assert_eq!(
        folders.sources.len(),
        file.sources.len(),
        "different sources"
    );
    assert_eq!(d.meta("rows"), folders.sheets.len().to_string());

    // The catalogue it carries is the tree's: one line per published row and
    // reader, a crossing row read by nobody once.
    let db = rusqlite::Connection::open(&out).unwrap();
    let mut held: Vec<(String, String, String)> = db
        .prepare("SELECT node, read_by_grp, read_by FROM published")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    held.sort();
    let mut want: Vec<(String, String, String)> = Vec::new();
    for p in vleo_sheet::catalogue::catalogue(&folders) {
        if p.read_by.is_empty() {
            want.push((p.node.clone(), String::new(), String::new()));
        }
        for (g, n) in &p.read_by {
            want.push((p.node.clone(), g.clone(), n.clone()));
        }
    }
    want.sort();
    assert!(!want.is_empty());
    assert_eq!(held, want, "the published table is not the catalogue");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn it_answers_for_its_own_paths_and_nothing_else() {
    let dir = scratch("paths");
    let out = written(&dir);
    let d = Design::open(&out, &root()).unwrap();
    let r = root();
    assert!(d.is_dir(&r.join("layers")));
    assert!(d.is_dir(&r.join("crates/vleo-mod-solar/nodes")));
    assert!(d.is_file(&r.join("sources/sources.toml")));
    // The web face, the kernel's source and the reference data stay files of
    // their own: the design file never answers for them.
    assert!(!d.is_file(&r.join("web/index.html")));
    assert!(!d.is_dir(&r.join("crates/vleo-core")));
    assert!(d.read(&r.join("Cargo.toml")).is_err());
    let mut layers = d.entries(&r.join("layers")).unwrap();
    layers.sort();
    let mut on_disk: Vec<PathBuf> = std::fs::read_dir(r.join("layers"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    on_disk.sort();
    assert_eq!(layers, on_disk);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_file_changed_after_it_was_written_is_caught() {
    let dir = scratch("changed");
    let out = written(&dir);
    {
        let db = rusqlite::Connection::open(&out).unwrap();
        db.execute(
            "UPDATE file SET bytes = CAST('changed' AS BLOB) WHERE path = 'sources/sources.toml'",
            [],
        )
        .unwrap();
    }
    let d = Design::open(&out, &root()).unwrap();
    let e = d.verify().expect_err("a changed file passed");
    assert!(
        e.message().contains("sources/sources.toml"),
        "{}",
        e.message()
    );
    let diff = d.compare(&root()).unwrap();
    assert_eq!(diff, vec!["sources/sources.toml: differs".to_string()]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn what_is_not_a_design_file_is_refused_by_name() {
    let dir = scratch("refused");
    let text = dir.join("notes.vleo");
    std::fs::write(&text, "not a database").unwrap();
    let e = Design::open(&text, &root()).err().expect("text opened");
    assert_eq!(e.kind(), ErrorKind::WrongFile);

    // A group's release is a VLEO database, and not a design file.
    let release = dir.join("example-1.0.vleo");
    {
        let db = rusqlite::Connection::open(&release).unwrap();
        db.execute_batch(
            "PRAGMA application_id = 1447838031; PRAGMA user_version = 1;
             CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO meta VALUES ('file_kind', 'release');",
        )
        .unwrap();
    }
    let e = Design::open(&release, &root())
        .err()
        .expect("a release opened");
    assert_eq!(e.kind(), ErrorKind::WrongFile);
    assert!(e.message().contains("release"), "{}", e.message());

    // One from a newer tool is refused, not misread.
    let newer = dir.join("newer.vleo");
    {
        let db = rusqlite::Connection::open(&newer).unwrap();
        db.execute_batch("PRAGMA application_id = 1447838031; PRAGMA user_version = 99;")
            .unwrap();
    }
    let e = Design::open(&newer, &root())
        .err()
        .expect("a newer file opened");
    assert_eq!(e.kind(), ErrorKind::Newer);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_design_and_results_files_are_the_kinds_the_one_table_lists() {
    use vleo_kinds::{Reader, KINDS};
    let listed = |name: &str, format: u32| {
        KINDS
            .iter()
            .any(|k| k.name == name && k.format() == Some(format) && k.reader == Reader::Design)
    };
    assert!(listed("design", vleo_design::FORMAT));
    assert!(listed(
        vleo_design::results::KIND,
        vleo_design::results::FORMAT
    ));
    for (sql, format) in [
        (include_str!("../design.sql"), vleo_design::FORMAT),
        (include_str!("../results.sql"), vleo_design::results::FORMAT),
    ] {
        assert!(sql.contains(&format!(
            "PRAGMA application_id = {};",
            vleo_kinds::APPLICATION_ID
        )));
        assert!(sql.contains(&format!("PRAGMA user_version = {format};")));
    }
}
