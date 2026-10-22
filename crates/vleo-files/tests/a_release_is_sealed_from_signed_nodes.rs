//! A group's release is sealed only from nodes signed as they are, by its
//! subsystem engineer, and is what today's design will check it to be
//! (docs/briefs/F5.md, the seal; W5).
//!
//! A small programme of invented people, as the breakdown's test has it: Ola
//! owns solar, Dee is Ola's deputy, Nia a node engineer of solar, Sam the
//! system engineer. Solar's branch holds solar 1.1's `sw_activity_band`, in a
//! node file of its own, assigned to Nia.
//!
//! - signed by Nia as it is, the branch is sealed by Ola, or Dee: the release
//!   names its version, the release before it, what it was based on and who
//!   sealed it, carries Nia's signature, and checks through the chain;
//! - not by Nia, nor Sam;
//! - refused, each by name: a node unsigned, changed since it was signed,
//!   behind its contract, assigned with no file, held twice, or held by the
//!   group file too; a version that is not one, or not after the last;
//! - in the copy, written into releases/, never over a release already there,
//!   and the next one records it as the release before.

use std::path::{Path, PathBuf};

use vleo_files::chain::{self, Registry, FILE};
use vleo_files::keys::SigningKey;
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::release;

const APP: &str = "vleo 1.0.0 (test)";
const AT: &str = "2026-10-17T09:00:00Z";
const BAND: &str = "sw_activity_band";

fn s(v: &str) -> String {
    v.to_string()
}

fn key(seed: u8) -> SigningKey {
    SigningKey::from_seed([seed; 32])
}

fn register(f: &mut File, name: &str, role: &str, deputy_for: &str, k: &SigningKey) {
    f.people.push(Person {
        name: s(name),
        role: s(role),
        deputy_for: s(deputy_for),
    });
    f.keys.push(PersonKey {
        person: s(name),
        public_key: k.public().to_hex(),
        registered_at: s("2026-01-01"),
        registered_by: s("Pat Morgan"),
        revoked_from: s(""),
        proof: vleo_files::people::proof(k, name),
    });
}

fn mount(f: &mut File, uid: &str, group: &str, owner: &str) {
    f.blocks.push(Block {
        uid: s(uid),
        id: s(uid),
        behaviour: s("children"),
        ..Block::default()
    });
    f.mounts.push(Mount {
        block_uid: s(uid),
        group_id: s(group),
        release: s(""),
    });
    f.assignments.push(Assignment {
        block_uid: s(uid),
        person: s(owner),
        contract_version: 1,
    });
}

fn group_file(id: &str) -> File {
    let mut f = File::new(Kind::Group, APP);
    f.meta.insert(s("group_id"), s(id));
    f.meta.insert(s("writer"), s("whoever"));
    f.meta.insert(s("based_on"), s(""));
    f
}

fn signed(mut f: File, who: &str, k: &SigningKey) -> File {
    f.signatures.retain(|r| r.scope != FILE);
    let row = chain::sign(&f, FILE, who, k, "2026-02-01", "ok", "").unwrap();
    f.signatures.push(row);
    f
}

// Pat the programme manager (11), Sam the system engineer (12), Ola who owns
// solar (13), Dee her deputy (14), Nia a node engineer of solar (16).
fn programme() -> Registry {
    let mut f = group_file("programme");
    register(&mut f, "Pat Morgan", "programme manager", "", &key(11));
    register(&mut f, "Sam Engel", "system engineer", "", &key(12));
    register(&mut f, "Ola Sun", "subsystem engineer", "", &key(13));
    register(
        &mut f,
        "Dee Puty",
        "subsystem engineer",
        "Ola Sun",
        &key(14),
    );
    mount(&mut f, "root", "systems", "Sam Engel");
    chain::open_programme(
        signed(f, "Pat Morgan", &key(11)),
        &key(11).public().fingerprint(),
    )
    .unwrap()
}

fn systems(pr: &Registry) -> Registry {
    let mut f = group_file("systems");
    mount(&mut f, "sys_space_environment", "l3_solar", "Ola Sun");
    chain::open_systems(&signed(f, "Sam Engel", &key(12)), pr).unwrap()
}

/// Solar's group file, unsigned: its heading, Nia, and the band assigned to
/// her at `contract`.
fn solar_unsigned(contract: i64) -> File {
    let mut f = group_file("l3_solar");
    f.blocks.push(Block {
        uid: s("l3_solar"),
        id: s("l3_solar"),
        behaviour: s("children"),
        question: s("What solar weather does the design face?"),
        ..Block::default()
    });
    f.people.push(Person {
        name: s("Nia Node"),
        role: s("node engineer"),
        deputy_for: s(""),
    });
    f.keys.push(PersonKey {
        person: s("Nia Node"),
        public_key: key(16).public().to_hex(),
        registered_at: s("2026-01-01"),
        registered_by: s("Ola Sun"),
        revoked_from: s(""),
        proof: vleo_files::people::proof(&key(16), "Nia Node"),
    });
    f.assignments.push(Assignment {
        block_uid: s(BAND),
        person: s("Nia Node"),
        contract_version: contract,
    });
    // The owner's record of which belief broke, a row for each version.
    f.tables.push(Tbl {
        scope: s(FILE),
        path: s("versions.csv"),
        csv: s("version,date,by,believed,tested,learned,changed,risks,cost,rests_on,breaks_if\n\
             1.0,2026-10-01,Ola Sun,,,,,,,the four NOAA SWPC activity levels,a correction to those levels\n\
             1.1,2026-10-05,Ola Sun,The bands held.,Read against their cases.,They held.,Nothing but words.,,,the four NOAA SWPC activity levels,a correction to those levels\n\
             1.2,2026-10-10,Ola Sun,The bands held.,Read again.,They held.,A method said plainly.,,,the four NOAA SWPC activity levels,a correction to those levels\n"),
    });
    f
}

fn solar() -> File {
    signed(solar_unsigned(1), "Ola Sun", &key(13))
}

struct World {
    pr: Registry,
    sy: Registry,
}

fn world() -> World {
    let pr = programme();
    let sy = systems(&pr);
    World { pr, sy }
}

/// `uid` from solar 1.1's release, in a node file of its own, unsigned.
fn from_release(uid: &str) -> File {
    let how = vleo_files::upgrade::Upgrade { app: APP, at: AT };
    let rel = vleo_files::sqlite::open(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/l3_solar-1.1.vleo"),
        &how,
    )
    .unwrap()
    .file;
    let mut n = File::new(Kind::Node, APP);
    for (k, v) in [
        ("group_id", "l3_solar"),
        ("block_uid", uid),
        ("revision", "1"),
        ("contract_version", "1"),
        ("writer", "Nia Node"),
    ] {
        n.meta.insert(s(k), s(v));
    }
    n.blocks = vec![rel.blocks.iter().find(|b| b.uid == uid).unwrap().clone()];
    n.blocks[0].parent_uid = s("l3_solar");
    n.ports = rel
        .ports
        .iter()
        .filter(|p| p.block_uid == uid)
        .cloned()
        .collect();
    n.wires = rel
        .wires
        .iter()
        .filter(|w| w.to_block == uid)
        .cloned()
        .collect();
    n.texts = rel
        .texts
        .iter()
        .filter(|t| t.scope == uid)
        .cloned()
        .collect();
    n.tables = rel
        .tables
        .iter()
        .filter(|t| t.scope == uid)
        .cloned()
        .collect();
    n
}

/// The band's node file, signed by Nia as it is.
fn band(w: &World) -> File {
    let reg = chain::open_group_in(&solar(), &w.sy, &w.pr).unwrap();
    chain::sign_node(&from_release(BAND), BAND, "Nia Node", &key(16), AT, &reg)
        .expect("Nia signs the band")
        .0
}

fn seal(
    w: &World,
    group: &File,
    nodes: &[File],
    version: &str,
    previous: Option<&str>,
    by: &str,
    k: u8,
) -> Result<File, String> {
    release::seal(
        group,
        nodes,
        version,
        previous,
        by,
        &key(k),
        AT,
        APP,
        &w.sy,
        &w.pr,
    )
    .map_err(|e| e.message().to_string())
}

#[test]
fn a_branch_of_signed_nodes_is_sealed_by_its_owner_and_checks_through_the_chain() {
    let w = world();
    let r = seal(&w, &solar(), &[band(&w)], "1.2", Some("1.1"), "Ola Sun", 13).unwrap();
    assert_eq!(r.kind().unwrap(), Kind::GroupRelease);
    assert_eq!(
        r.meta.get("signed_rows").map(String::as_str),
        Some("issue change"),
        "its rows are signed with it (F6-d)"
    );
    for (k, v) in [
        ("group_id", "l3_solar"),
        ("version", "1.2"),
        ("previous", "1.1"),
        ("based_on", ""),
        ("sealed_by", "Ola Sun"),
    ] {
        assert_eq!(r.meta.get(k).map(String::as_str), Some(v), "{k}");
    }
    assert_eq!(
        r.meta.get("fingerprint").unwrap(),
        &chain::digest(&r, FILE).unwrap(),
        "the fingerprint is not the digest of what the release holds"
    );
    assert!(
        r.signatures
            .iter()
            .any(|g| g.scope == BAND && g.signer == "Nia Node"),
        "the release does not carry Nia's signature of the band"
    );
    let checked = chain::check_release_in(&r, &w.sy, &w.pr).unwrap();
    assert!(checked.holds(), "{:?}", checked.refused);
    assert!(
        checked.signed.iter().any(|b| b == BAND),
        "{:?}",
        checked.signed
    );

    // Dee, Ola's deputy, seals it too. Nia, whom the programme's file does
    // not register, does not; Sam, whom it does, is refused by the owner rule.
    seal(&w, &solar(), &[band(&w)], "1.2", None, "Dee Puty", 14).unwrap();
    let e = seal(&w, &solar(), &[band(&w)], "1.2", None, "Nia Node", 16).unwrap_err();
    assert!(e.contains("Nia Node is not registered"), "{e}");
    let e = seal(&w, &solar(), &[band(&w)], "1.2", None, "Sam Engel", 12).unwrap_err();
    assert!(e.contains("may not change l3_solar"), "{e}");
}

#[test]
fn a_seal_is_refused_by_name_while_anything_stands_in_its_way() {
    let w = world();
    let refused = |group: &File, nodes: &[File], says: &str| {
        let e = seal(&w, group, nodes, "1.2", None, "Ola Sun", 13).unwrap_err();
        assert!(e.contains(says), "expected «{says}» in: {e}");
        assert!(e.contains("is not sealed"), "{e}");
    };

    // Unsigned.
    refused(&solar(), &[from_release(BAND)], "not signed as it is");

    // Changed after Nia signed it.
    let mut changed = band(&w);
    changed.blocks[0].question.push_str(" (and one more word)");
    refused(&solar(), &[changed], "not signed as it is");

    // Behind its contract: the group moved the band's contract to 2.
    let moved = signed(solar_unsigned(2), "Ola Sun", &key(13));
    refused(&moved, &[band(&w)], "behind its contract");

    // Assigned, with no node file.
    refused(&solar(), &[], "there is no node file of it");

    // Held twice.
    refused(&solar(), &[band(&w), band(&w)], "two node files hold it");

    // Held by the group file too.
    let mut both = solar_unsigned(1);
    both.texts.push(Text {
        scope: s(BAND),
        kind: s("note"),
        body: s("a note the group file keeps of the band"),
    });
    refused(
        &signed(both, "Ola Sun", &key(13)),
        &[band(&w)],
        "the group file holds rows of it too",
    );

    // A version the group's record says nothing of: no belief said broken.
    let e = seal(&w, &solar(), &[band(&w)], "1.3", None, "Ola Sun", 13).unwrap_err();
    assert!(e.contains("versions.csv has no row for version 1.3"), "{e}");
    // A version that is not one, or does not come after the last.
    let e = seal(&w, &solar(), &[band(&w)], "two", None, "Ola Sun", 13).unwrap_err();
    assert!(e.contains("not a release's version"), "{e}");
    let e = seal(&w, &solar(), &[band(&w)], "1.1", Some("1.1"), "Ola Sun", 13).unwrap_err();
    assert!(e.contains("does not come after 1.1"), "{e}");
}

fn copy_with(name: &str, w: &World) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vleo-seal-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let g = dir.join("groups/l3_solar");
    std::fs::create_dir_all(g.join("nodes")).unwrap();
    vleo_files::sqlite::write(&solar(), &g.join("l3_solar.vgroup")).unwrap();
    vleo_files::sqlite::write(&band(w), &g.join(format!("nodes/{BAND}.vnode"))).unwrap();
    dir
}

#[test]
fn a_release_is_written_into_the_copy_and_never_written_over() {
    let w = world();
    let copy = copy_with("copy", &w);
    let seal_in = |v: &str| {
        release::seal_in_place(
            &copy,
            "l3_solar",
            v,
            "Ola Sun",
            &key(13),
            AT,
            APP,
            &w.sy,
            &w.pr,
        )
        .map_err(|e| e.message().to_string())
    };
    let rel = seal_in("1.0").unwrap();
    assert_eq!(rel, "groups/l3_solar/releases/l3_solar-1.0.vleo");
    let how = vleo_files::upgrade::Upgrade { app: APP, at: AT };
    let written = vleo_files::sqlite::open(&copy.join(&rel), &how)
        .unwrap()
        .file;
    assert!(chain::check_release_in(&written, &w.sy, &w.pr)
        .unwrap()
        .holds());
    assert_eq!(written.meta.get("previous").map(String::as_str), Some(""));

    // Never written over; the next records the one before it.
    let before = std::fs::read(copy.join(&rel)).unwrap();
    let e = seal_in("1.0").unwrap_err();
    assert!(
        e.contains("l3_solar-1.0.vleo is already there: a release is never written over"),
        "{e}"
    );
    assert_eq!(
        std::fs::read(copy.join(&rel)).unwrap(),
        before,
        "a release was written over"
    );
    let next = seal_in("1.1").unwrap();
    let next = vleo_files::sqlite::open(&copy.join(&next), &how)
        .unwrap()
        .file;
    assert_eq!(next.meta.get("previous").map(String::as_str), Some("1.0"));
    let _ = std::fs::remove_dir_all(&copy);
}

#[test]
fn a_release_s_node_in_the_design_s_file_keeps_only_the_sheet_s_own_words() {
    let text = |kind: &str, body: &str| Text {
        scope: s(BAND),
        kind: s(kind),
        body: s(body),
    };
    let mut base = from_release(BAND);
    base.texts = vec![
        text("sheet", "the sheet's own words"),
        text("method", "return 1"),
        text("pseudocode", "return 1"),
        text("explain.simply", "the design's old words"),
        text(
            "fixtures",
            "the design's cases, which the release no longer holds",
        ),
    ];
    base.meta.insert(s("sheet.crate"), s("vleo-mod-solar"));
    let kinds = |rel: &File| -> Vec<(String, String)> {
        vleo_files::release::node_in(rel, BAND, &base)
            .texts
            .into_iter()
            .map(|t| (t.kind, t.body))
            .collect()
    };
    // Its method and pseudocode are the release's; the design's sheet is
    // kept, and nothing else the release dropped.
    let mut rel = from_release(BAND);
    rel.texts = vec![text("method", "return 2"), text("pseudocode", "return 2")];
    let mut got = kinds(&rel);
    got.sort();
    assert_eq!(
        got,
        [
            (s("method"), s("return 2")),
            (s("pseudocode"), s("return 2")),
            (s("sheet"), s("the sheet's own words")),
        ]
    );
    // A release whose node says its pseudocode and no method runs no method
    // of the design's: none is kept, and the node is refused as it is read.
    rel.texts = vec![text("pseudocode", "return 2")];
    let got = kinds(&rel);
    assert!(!got.iter().any(|(k, _)| k == "method"), "{got:?}");
    let n = vleo_files::release::node_in(&rel, BAND, &base);
    assert_eq!(
        n.meta.get("sheet.crate").map(String::as_str),
        Some("vleo-mod-solar")
    );
}

#[test]
fn only_a_heading_is_the_seal_s_own_and_a_node_file_holds_its_block_s_rows_alone() {
    let w = world();
    let refused = |group: &File, nodes: &[File], says: &str| {
        let e = seal(&w, group, nodes, "1.2", None, "Ola Sun", 13).unwrap_err();
        assert!(e.contains(says), "expected «{says}» in: {e}");
    };
    // A computing block framed in the group file, assigned to nobody.
    let mut framed = solar_unsigned(1);
    framed.blocks.push(Block {
        uid: s("sw_unsigned_rule"),
        id: s("sw_unsigned_rule"),
        parent_uid: s("l3_solar"),
        behaviour: s("method"),
        ..Block::default()
    });
    refused(
        &signed(framed, "Ola Sun", &key(13)),
        &[band(&w)],
        "sw_unsigned_rule: framed in the group file as a method block and not issued",
    );
    // A release built by hand with the band's assignment taken away: the
    // band is no heading, and nobody's signature counts for it.
    let r = seal(&w, &solar(), &[band(&w)], "1.2", None, "Ola Sun", 13).unwrap();
    let mut bare = r.clone();
    bare.assignments.retain(|a| a.block_uid != BAND);
    let bare = signed(bare, "Ola Sun", &key(13));
    let checked = chain::check_release_in(&bare, &w.sy, &w.pr).unwrap();
    assert!(
        checked
            .refused
            .iter()
            .any(|e| e.contains("sw_activity_band is a method block that no assignment names")),
        "{:?}",
        checked.refused
    );
    // A node file carrying a requirement of the file's own, which its node
    // engineer's signature does not cover.
    let mut carried = band(&w);
    carried.tables.push(Tbl {
        scope: s(FILE),
        path: s("requirements.csv"),
        csv: s("id,sense\nr1,>=\n"),
    });
    chain::check_node(
        &carried,
        BAND,
        &chain::open_group_in(&solar(), &w.sy, &w.pr).unwrap(),
    )
    .expect("the signature does not cover it");
    refused(
        &solar(),
        &[carried],
        "its file holds rows that are not its block's (tbl)",
    );
    // Ahead of its contract.
    let mut ahead = band(&w);
    ahead.blocks[0].contract_version = 2;
    refused(&solar(), &[ahead], "ahead of its contract");
    // A node file of two blocks; a node assigned to nobody; a group file
    // that does not open; a file that is not a group file.
    let mut two = band(&w);
    let mut extra = two.blocks[0].clone();
    extra.uid = s("sw_second");
    two.blocks.push(extra);
    refused(&solar(), &[two], "holds its one block, and this holds 2");
    refused(
        &solar(),
        &[band(&w), from_release("sw_f107_observed")],
        "the group file assigns it to nobody",
    );
    refused(&solar_unsigned(1), &[band(&w)], "l3_solar's group file:");
    refused(
        &band(&w),
        &[band(&w)],
        "sealed from a group file, not a node",
    );
}

#[test]
fn the_releases_already_there_are_each_one_the_chain_admits_or_the_seal_is_refused() {
    let w = world();
    let copy = copy_with("releases-there", &w);
    let rel = copy.join("groups/l3_solar/releases");
    let seal_in = |v: &str| {
        release::seal_in_place(
            &copy,
            "l3_solar",
            v,
            "Ola Sun",
            &key(13),
            AT,
            APP,
            &w.sy,
            &w.pr,
        )
        .map_err(|e| e.message().to_string())
    };
    let first = copy.join(seal_in("1.0").unwrap());
    let refused = |name: &str, bytes: Option<&[u8]>, says: &str| {
        let at = rel.join(name);
        match bytes {
            Some(b) => std::fs::write(&at, b).unwrap(),
            None => {
                std::fs::copy(&first, &at).unwrap();
            }
        }
        let e = seal_in("1.1").unwrap_err();
        assert!(e.contains(says), "{name}: expected «{says}» in: {e}");
        std::fs::remove_file(&at).unwrap();
    };
    refused(
        "l3_solar-1.0 (1).vleo",
        None,
        "two files where there should be one",
    );
    refused(
        "l3_solar-1.0-again.vleo",
        None,
        "both say they are l3_solar 1.0",
    );
    refused("l3_solar-1.1.VLEO", None, "named in another case");
    refused(
        "l3_solar-9.0.vleo",
        Some(b"not a release"),
        "cannot be read",
    );
    // One changed after its seal: the chain does not admit it.
    let how = vleo_files::upgrade::Upgrade { app: APP, at: AT };
    let mut f = vleo_files::sqlite::open(&first, &how).unwrap().file;
    f.meta.insert(s("version"), s("9.0"));
    vleo_files::sqlite::write(&f, &rel.join("l3_solar-9.0.vleo")).unwrap();
    let e = seal_in("9.1").unwrap_err();
    assert!(
        e.contains("releases/l3_solar-9.0.vleo does not check through the chain"),
        "{e}"
    );
    std::fs::remove_file(rel.join("l3_solar-9.0.vleo")).unwrap();
    // With only what the chain admits there, the next is sealed after it.
    seal_in("1.1").unwrap();
    let _ = std::fs::remove_dir_all(&copy);
}
