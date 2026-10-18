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

/// The system engineer, whom the programme's file registers and mounts the
/// systems' branch for.
fn sam() -> SigningKey {
    SigningKey::from_seed([83; 32])
}

fn sql(file: &Path, statement: &str) {
    let db = rusqlite::Connection::open(file).unwrap();
    assert!(db.execute(statement, []).unwrap() > 0, "{statement}");
}

/// 1.1 made into its 1.2 and sealed again, as its group would: one node's
/// method said again with a line of its own, so 1.2 writes into the design
/// where 1.1, already taken into it, writes nothing.
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
    sql(
        file,
        "UPDATE doc SET body = '# 1.2: the same four bands, said again.\n' || body \
         WHERE kind = 'pseudocode' AND scope = 'sw_activity_band'",
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
        proof: vleo_files::people::proof(&pat(), "Test Pat"),
    });
    programme.people.push(Person {
        name: s("Test Sam"),
        role: s("system engineer"),
        deputy_for: s(""),
    });
    programme.keys.push(PersonKey {
        person: s("Test Sam"),
        public_key: sam().public().to_hex(),
        registered_at: s("2026-01-01"),
        registered_by: s("Test Pat"),
        revoked_from: s(""),
        proof: vleo_files::people::proof(&sam(), "Test Sam"),
    });
    programme.blocks.push(Block {
        uid: s("root"),
        id: s("root"),
        behaviour: s("children"),
        ..Block::default()
    });
    programme.mounts.push(Mount {
        block_uid: s("root"),
        group_id: s("systems"),
        release: s(""),
    });
    programme.assignments.push(Assignment {
        block_uid: s("root"),
        person: s("Test Sam"),
        contract_version: 1,
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
        k.chain.contains("anchored by the programme manager"),
        "{}",
        k.chain
    );
    // Whether this computer has accepted the anchor is read when it is said:
    // not yet, and then, once accepted as opening a key accepts it, so.
    let said = k.chain_said(&d);
    assert!(said.contains("has not accepted that anchor yet"), "{said}");
    let anchors = Path::new(&std::env::var("VLEO_KEYS").unwrap()).join("anchors");
    let line = format!(
        "{} {}\n",
        pat().public().fingerprint(),
        std::fs::canonicalize(&d).unwrap().display()
    );
    let before = std::fs::read_to_string(&anchors).unwrap_or_default();
    std::fs::write(&anchors, format!("{before}{line}")).unwrap();
    let said = k.chain_said(&d);
    assert!(!said.contains("not accepted"), "{said}");
    assert!(said.contains("anchored by the programme manager"), "{said}");
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

/// Today's design at `root`, and the fingerprint of the design's own files.
fn today_at(root: &Path, d: &Path) -> (Today, String) {
    let (design, fp) = vleo_files::convert::serve(root).expect("design/ is served");
    let t = build(Arc::new(design), root, d).expect("today's design builds");
    (t, fp)
}

#[test]
fn two_instances_on_one_drive_build_the_same_design_by_fingerprint() {
    keys();
    let d = drive("twice", &["1.1", "1.2"], &pat().public().fingerprint());
    // A second copy of the tool, in another folder, on the same design.
    let other = std::env::temp_dir().join(format!("vleo-today-chain-other-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&other);
    std::fs::create_dir_all(&other).unwrap();
    std::os::unix::fs::symlink(root().join("design"), other.join("design")).unwrap();
    let (one, fp) = today_at(&root(), &d);
    let (two, fp_two) = today_at(&other, &d);
    assert_eq!(fp, fp_two, "the same design's own files");
    assert_eq!(one.groups[0].taken.as_ref().unwrap().version, "1.2");
    assert!(!one.written.is_empty(), "1.2 writes into the design");
    assert_eq!(one.fingerprint(&fp), two.fingerprint(&fp_two));
    assert_ne!(
        one.fingerprint(&fp),
        fp,
        "today's design is not the design's own files"
    );

    // Another release taken is another design; 1.1, already in the design's
    // own files, is that design.
    let older = drive("twice-older", &["1.1"], &pat().public().fingerprint());
    let (then, _) = today_at(&root(), &older);
    assert_eq!(then.groups[0].taken.as_ref().unwrap().version, "1.1");
    assert_ne!(then.fingerprint(&fp), one.fingerprint(&fp));
    assert_eq!(then.fingerprint(&fp), fp);
    for x in [&d, &older, &other] {
        let _ = std::fs::remove_dir_all(x);
    }
}

/// The systems' file, naming its base, and signed by the system engineer
/// when `signed`.
fn systems(d: &Path, based_on: &str, base_version: &str, signed: bool) {
    let mut f = File::new(Kind::Group, APP);
    for (k, v) in [
        ("group_id", "systems"),
        ("writer", "Test Sam"),
        ("based_on", based_on),
        ("base_version", base_version),
    ] {
        f.meta.insert(s(k), s(v));
    }
    if signed {
        let row = chain::sign(&f, FILE, "Test Sam", &sam(), "2026-10-08", "ok", "").unwrap();
        f.signatures.push(row);
    }
    std::fs::create_dir_all(d.join("groups/systems")).unwrap();
    vleo_files::sqlite::write(&f, &d.join("groups/systems/systems.vgroup")).unwrap();
}

#[test]
fn the_design_s_base_is_the_one_the_systems_file_names() {
    keys();
    let d = drive("base", &["1.1"], &pat().public().fingerprint());
    let (t, _) = today_at(&root(), &d);
    assert!(t.base.said.contains("no systems' file"), "{}", t.base.said);
    assert_eq!(t.base.base_version, None);

    // A systems' file the chain does not admit names no base.
    systems(&d, "design-2026-09", "2", false);
    let (t, _) = today_at(&root(), &d);
    assert!(
        t.base.said.contains("does not check through the chain") && t.base.base_version.is_none(),
        "{:?}",
        t.base
    );

    systems(&d, "design-2026-09", "2", true);
    let (t, _) = today_at(&root(), &d);
    assert_eq!(t.base.based_on, "design-2026-09");
    assert_eq!(t.base.base_version, Some(2));
    assert!(
        t.base.said.contains("design-2026-09, the 2nd"),
        "{}",
        t.base.said
    );
    let k = t.groups[0].taken.as_ref().unwrap();
    assert_eq!(
        (k.based_on.as_str(), k.base_version),
        ("", None),
        "1.1 is from before they were kept"
    );

    // A base that does not say one thing is said as such, never as a base.
    for (on, n, says) in [
        ("design-2026-09", "two", "not a whole number"),
        ("", "3", "names none"),
        ("design-2026-09", "0", "counts none before it"),
    ] {
        systems(&d, on, n, true);
        let (t, _) = today_at(&root(), &d);
        assert!(t.base.said.contains(says), "{on:?} {n:?}: {}", t.base.said);
    }
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn a_drive_with_an_anchor_and_no_programme_s_file_admits_nothing() {
    keys();
    let d = drive(
        "no-programme",
        &["1.1", "1.2"],
        &pat().public().fingerprint(),
    );
    std::fs::remove_file(d.join("groups/programme/programme.vgroup")).unwrap();
    let t = today(&d);
    let g = &t.groups[0];
    assert!(g.taken.is_none(), "{:?}", g.taken);
    assert!(
        g.refused
            .iter()
            .all(|(_, why)| why.contains("has an anchor and no programme's file")),
        "{:?}",
        g.refused
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn accepting_another_anchor_later_does_not_vouch_for_seals_checked_from_this_one() {
    keys();
    let d = drive("other-anchor", &["1.1"], &pat().public().fingerprint());
    let t = today(&d);
    let k = t.groups[0].taken.as_ref().unwrap();
    let anchors = Path::new(&std::env::var("VLEO_KEYS").unwrap()).join("anchors");
    let before = std::fs::read_to_string(&anchors).unwrap_or_default();
    let other = SigningKey::from_seed([82; 32]).public().fingerprint();
    std::fs::write(
        &anchors,
        format!(
            "{before}{other} {}\n",
            std::fs::canonicalize(&d).unwrap().display()
        ),
    )
    .unwrap();
    let said = k.chain_said(&d);
    assert!(said.contains("accepted another anchor"), "{said}");
    std::fs::write(&anchors, before).unwrap();
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn a_release_in_the_one_schema_whose_seal_cannot_be_checked_is_refused_by_name() {
    keys();
    let d = drive("schema", &["1.1"], &pat().public().fingerprint());
    let mut f = File::new(Kind::GroupRelease, APP);
    for (k, v) in [("group_id", "l3_solar"), ("version", "2.0")] {
        f.meta.insert(s(k), s(v));
    }
    vleo_files::sqlite::write(&f, &d.join("groups/l3_solar/releases/l3_solar-2.0.vleo")).unwrap();
    let t = today(&d);
    let g = &t.groups[0];
    assert_eq!(g.taken.as_ref().map(|k| k.version.as_str()), Some("1.1"));
    assert!(
        g.refused
            .iter()
            .any(|(v, why)| v == "2.0" && why.contains("the drive has no systems' file")),
        "{:?}",
        g.refused
    );
    let _ = std::fs::remove_dir_all(&d);
}
