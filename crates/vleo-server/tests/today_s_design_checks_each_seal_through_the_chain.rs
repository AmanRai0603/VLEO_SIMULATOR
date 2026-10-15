//! Today's design checks each release's seal through the chain, when the drive
//! has a programme's file (docs/briefs/F2.md, item 1; W16).
//!
//! The releases are the solar group's, kept as test fixtures: 1.1 passes every
//! check, and a 1.2 is made from it as its group would and sealed again. Both
//! are from before 1.0, so each checks through the chain by the programme
//! manager's anchor of its fingerprint, kept in the programme's file. The
//! programme manager here, an invented person, anchors 1.1 and not 1.2: 1.2 is
//! a release the chain does not admit, refused, and 1.1 taken in its place and
//! marked. A programme's file that does not open under START HERE's anchor
//! admits nothing.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use vleo_files::chain::{self, FILE};
use vleo_files::keys::SigningKey;
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_server::today::{build, Today};

const APP: &str = "vleo 1.0.0 (test)";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn s(v: &str) -> String {
    v.to_string()
}

fn pat() -> SigningKey {
    SigningKey::from_seed([81; 32])
}

fn sql(file: &Path, statement: &str) {
    let db = rusqlite::Connection::open(file).unwrap();
    assert!(db.execute(statement, []).unwrap() > 0, "{statement}");
}

/// 1.1 made into its 1.2 and sealed again, as its group would.
fn as_1_2(file: &Path) {
    sql(file, "UPDATE meta SET value = '1.2' WHERE key = 'version'");
    sql(
        file,
        "UPDATE tbl SET csv = csv || '1.2,2026-10-06,Test Ola,\
         The bands were right and their method said them plainly.,\
         The method was read again against its cases.,\
         A line saying what the method is helps the next reader.,\
         A first line saying what the method does.,,,\
         the four NOAA SWPC activity levels,a correction to those levels\\n' \
         WHERE scope = 'group' AND path = 'versions.csv'",
    );
    let r = vleo_files::intake::Release::open(file).unwrap();
    let fp = vleo_files::seal::fingerprint_of(&r.folder, "group");
    sql(
        file,
        &format!("UPDATE meta SET value = '{fp}' WHERE key = 'fingerprint'"),
    );
}

/// A release as the one schema holds it, as the chain reads it.
fn as_file(path: &Path) -> File {
    let how = vleo_files::upgrade::Upgrade {
        app: APP,
        at: "2026-10-01",
    };
    vleo_files::sqlite::open(path, &how).unwrap().file
}

/// A drive with solar's 1.1 and 1.2, and a programme's file whose manager
/// anchors the releases named in `anchored`. START HERE names `start_here`.
fn drive(name: &str, anchored: &[&str], start_here: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("vleo-today-chain-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let rel = d.join("groups/l3_solar/releases");
    std::fs::create_dir_all(&rel).unwrap();
    let fixture = root().join("crates/vleo-files/tests/fixtures/l3_solar-1.1.vleo");
    std::fs::copy(&fixture, rel.join("l3_solar-1.1.vleo")).unwrap();
    std::fs::copy(&fixture, rel.join("l3_solar-1.2.vleo")).unwrap();
    as_1_2(&rel.join("l3_solar-1.2.vleo"));

    let mut programme = File::new(Kind::Group, APP);
    for (k, v) in [
        ("group_id", "programme"),
        ("writer", "Test Pat"),
        ("based_on", ""),
    ] {
        programme.meta.insert(s(k), s(v));
    }
    programme.people.push(Person {
        name: s("Test Pat"),
        role: s("programme manager"),
        deputy_for: s(""),
    });
    programme.keys.push(PersonKey {
        person: s("Test Pat"),
        public_key: pat().public().to_hex(),
        registered_at: s("2026-01-01"),
        registered_by: s("Test Pat"),
        revoked_from: s(""),
    });
    // The anchors are part of what the programme manager signs: they go in
    // first, then the file is signed as a whole.
    for v in anchored {
        let release = as_file(&rel.join(format!("l3_solar-{v}.vleo")));
        programme
            .signatures
            .push(vleo_files::upgrade::anchor(&release, "Test Pat", &pat(), "2026-10-07").unwrap());
    }
    let row = chain::sign(&programme, FILE, "Test Pat", &pat(), "2026-10-08", "ok", "").unwrap();
    programme.signatures.push(row);

    std::fs::create_dir_all(d.join("groups/programme")).unwrap();
    vleo_files::sqlite::write(&programme, &d.join("groups/programme/programme.vgroup")).unwrap();
    std::fs::write(
        d.join("START HERE.md"),
        format!("# START HERE: an invented drive\n\n    {start_here}\n"),
    )
    .unwrap();
    d
}

fn today(d: &Path) -> Today {
    let (design, _) = vleo_files::convert::serve(&root()).expect("design/ is served");
    build(Arc::new(design), &root(), d).expect("today's design builds")
}

/// The keys folder this computer reads its accepted anchors from: one of the
/// test's own, which has accepted none.
fn keys() {
    let k = std::env::temp_dir().join(format!("vleo-today-chain-keys-{}", std::process::id()));
    std::fs::create_dir_all(&k).unwrap();
    std::env::set_var("VLEO_KEYS", &k);
}

#[test]
fn a_release_the_chain_does_not_admit_is_refused_and_the_last_good_one_taken() {
    keys();
    let d = drive("one", &["1.1"], &pat().public().fingerprint());
    let t = today(&d);
    let g = &t.groups[0];
    let k = g.taken.as_ref().expect("a release is taken");
    assert_eq!(k.version, "1.1", "{:?}", g.refused);
    assert_eq!(g.refused.len(), 1, "{:?}", g.refused);
    assert_eq!(g.refused[0].0, "1.2");
    assert!(
        g.refused[0].1.contains("does not check through the chain")
            && g.refused[0].1.contains("anchor"),
        "{}",
        g.refused[0].1
    );
    assert!(
        k.chain.contains("anchored by the programme manager")
            && k.chain.contains("has not accepted that anchor yet"),
        "{}",
        k.chain
    );
    assert!(g.said().contains("1.2, is refused"), "{}", g.said());
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn a_release_the_chain_admits_is_taken() {
    keys();
    let d = drive("both", &["1.1", "1.2"], &pat().public().fingerprint());
    let t = today(&d);
    let g = &t.groups[0];
    assert_eq!(
        g.taken.as_ref().map(|k| k.version.as_str()),
        Some("1.2"),
        "{:?}",
        g.refused
    );
    assert!(g.refused.is_empty(), "{:?}", g.refused);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn a_programme_s_file_that_does_not_open_admits_nothing() {
    keys();
    let other = SigningKey::from_seed([82; 32]).public().fingerprint();
    let d = drive("broken", &["1.1", "1.2"], &other);
    let t = today(&d);
    let g = &t.groups[0];
    assert!(g.taken.is_none(), "{:?}", g.taken);
    assert_eq!(g.refused.len(), 2, "{:?}", g.refused);
    assert!(
        g.refused
            .iter()
            .all(|(_, why)| why.contains("cannot be checked through the chain")),
        "{:?}",
        g.refused
    );
    let _ = std::fs::remove_dir_all(&d);
}
