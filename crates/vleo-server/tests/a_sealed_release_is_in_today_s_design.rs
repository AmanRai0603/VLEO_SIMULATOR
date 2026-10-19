//! A group's release, sealed by its owner and put back on the drive, is in
//! today's design for everyone who opens it (docs/briefs/F5.md, W5).
//!
//! A drive of invented people: Test Pat (91), programme manager; Test Sam
//! (92), system engineer; Test Ola (93), who owns solar through the systems'
//! mount; Test Nia (94), assigned solar's `sw_activity_band`. Solar's group
//! file and the node's file are the design's own, with its people added. Nia
//! says the node again in her copy and signs it; Ola seals solar 1.0 there
//! and puts it back. Today's design, built from the drive by anyone, holds
//! the node as Nia said it, and says the release and who sealed it.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use vleo_files::chain::{self, Registry, FILE};
use vleo_files::copied::{self, Copied};
use vleo_files::keys::SigningKey;
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::put_back::{digests, put_back, Chain};
use vleo_files::save::{sign_in_place, Saved};
use vleo_files::sqlite;
use vleo_server::today::{build, Today};

const APP: &str = "vleo 1.0.0 (test)";
const AT: &str = "2026-10-15T12:00:00Z";
const UID: &str = "sw_activity_band";
const NODE: &str = "groups/l3_solar/nodes/sw_activity_band.vnode";
const SOLAR: &str = "groups/l3_solar/l3_solar.vgroup";
const SAID: &str = " Said again by its node engineer for solar 1.0.";

fn s(v: &str) -> String {
    v.to_string()
}

fn key(seed: u8) -> SigningKey {
    SigningKey::from_seed([seed; 32])
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn register(f: &mut File, name: &str, role: &str, k: &SigningKey, by: &str) {
    f.people.push(Person {
        name: s(name),
        role: s(role),
        deputy_for: s(""),
    });
    f.keys.push(PersonKey {
        person: s(name),
        public_key: k.public().to_hex(),
        registered_at: s("2026-01-01"),
        registered_by: s(by),
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

fn group(id: &str, writer: &str) -> File {
    let mut f = File::new(Kind::Group, APP);
    f.meta.insert(s("group_id"), s(id));
    f.meta.insert(s("writer"), s(writer));
    f.meta.insert(s("based_on"), s(""));
    f
}

fn signed(mut f: File, who: &str, k: &SigningKey) -> File {
    let row = chain::sign(&f, FILE, who, k, "2026-02-01", "ok", "").unwrap();
    f.signatures.push(row);
    f
}

fn design(path: &str) -> File {
    sqlite::read(&root().join("design").join(path)).unwrap()
}

/// The design's node as its node engineer brings it to 1.0's checks, from
/// what solar's 1.1 release holds of it: its method in the method language,
/// its declaration and its cases, and its input's default, and its units as the method language writes them.
fn brought_up(mut n: File) -> File {
    let how = vleo_files::upgrade::Upgrade {
        app: APP,
        at: "2026-10-01",
    };
    let rel = sqlite::open(
        &root().join("crates/vleo-files/tests/fixtures/l3_solar-1.1.vleo"),
        &how,
    )
    .unwrap()
    .file;
    n.texts.extend(
        rel.texts
            .iter()
            .filter(|t| t.scope == UID && t.kind == "pseudocode")
            .cloned(),
    );
    n.tables
        .extend(rel.tables.iter().filter(|t| t.scope == UID).cloned());
    for p in n.ports.iter_mut() {
        let theirs = rel
            .ports
            .iter()
            .find(|q| {
                q.block_uid == UID
                    && q.direction == p.direction
                    && (q.name == p.name || p.direction == "out")
            })
            .unwrap();
        p.unit = theirs.unit.clone();
        if p.direction == "in" {
            p.value = theirs.value.clone();
            p.lower = theirs.lower.clone();
            p.upper = theirs.upper.clone();
        }
    }
    n
}

/// The drive, and a copy taken of it, under a scratch folder of `name`.
fn world(name: &str) -> (PathBuf, PathBuf) {
    let scratch =
        std::env::temp_dir().join(format!("vleo-today-sealed-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    let (drive, copy) = (scratch.join("drive"), scratch.join("copy"));
    let mut programme = group("programme", "Test Pat");
    register(
        &mut programme,
        "Test Pat",
        "programme manager",
        &key(91),
        "Test Pat",
    );
    register(
        &mut programme,
        "Test Sam",
        "system engineer",
        &key(92),
        "Test Pat",
    );
    register(
        &mut programme,
        "Test Ola",
        "subsystem engineer",
        &key(93),
        "Test Pat",
    );
    mount(&mut programme, "root", "systems", "Test Sam");
    let mut systems = group("systems", "Test Sam");
    mount(
        &mut systems,
        "sys_space_environment",
        "l3_solar",
        "Test Ola",
    );
    // Solar's group file and the node's, as the design holds them, with the
    // group's node engineer and her assignment.
    let mut solar = design(SOLAR);
    solar.meta.insert(s("writer"), s("Test Ola"));
    register(
        &mut solar,
        "Test Nia",
        "node engineer",
        &key(94),
        "Test Ola",
    );
    // Ola's record of which belief broke: solar's first release.
    solar.tables.push(Tbl {
        scope: s(FILE),
        path: s("versions.csv"),
        csv: s("version,date,by,believed,tested,learned,changed,risks,cost,rests_on,breaks_if\n\
             1.0,2026-10-15,Test Ola,,,,,,,the four NOAA SWPC activity levels,a correction to those levels\n"),
    });
    solar.assignments.push(Assignment {
        block_uid: s(UID),
        person: s("Test Nia"),
        contract_version: 1,
    });
    let mut node = brought_up(design(NODE));
    node.meta.insert(s("writer"), s("Test Nia"));
    for d in [
        "groups/programme",
        "groups/systems",
        "groups/l3_solar/nodes",
    ] {
        std::fs::create_dir_all(drive.join(d)).unwrap();
    }
    for (f, who, k, at) in [
        (
            programme,
            "Test Pat",
            key(91),
            "groups/programme/programme.vgroup",
        ),
        (
            systems,
            "Test Sam",
            key(92),
            "groups/systems/systems.vgroup",
        ),
        (solar, "Test Ola", key(93), SOLAR),
    ] {
        sqlite::write(&signed(f, who, &k), &drive.join(at)).unwrap();
    }
    sqlite::write(&node, &drive.join(NODE)).unwrap();
    std::fs::write(
        drive.join("START HERE.md"),
        format!("# START HERE\n\n    {}\n", key(91).public().fingerprint()),
    )
    .unwrap();
    let files = digests(&drive).unwrap();
    for path in files.keys() {
        let to = copy.join(path);
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(drive.join(path), &to).unwrap();
    }
    let r = Copied {
        drive: std::fs::canonicalize(&drive).unwrap().display().to_string(),
        copied_at: s("2026-10-15T09:00:00Z"),
        files,
    };
    std::fs::write(copy.join(copied::FILE_NAME), r.to_toml()).unwrap();
    (drive, copy)
}

struct Opened {
    anchor: String,
    programme: Registry,
    systems: Registry,
    solar: Registry,
}

fn opened(at: &Path) -> Opened {
    let read = |p: &str| sqlite::read(&at.join(p)).unwrap();
    let anchor = key(91).public().fingerprint();
    let programme =
        chain::open_programme(read("groups/programme/programme.vgroup"), &anchor).unwrap();
    let systems = chain::open_systems(&read("groups/systems/systems.vgroup"), &programme).unwrap();
    let solar = chain::open_group_in(&read(SOLAR), &systems, &programme).unwrap();
    Opened {
        anchor,
        programme,
        systems,
        solar,
    }
}

/// Nia says the node again in the copy, as its second revision, and signs
/// it; Ola seals solar 1.0 there and puts it back on the drive.
fn sealed_and_put_back(drive: &Path, copy: &Path) -> String {
    sealed_with(drive, copy, &|_| {})
}

fn today(drive: &Path) -> Today {
    let (design, _) = vleo_files::convert::serve(&root()).unwrap();
    build(Arc::new(design), &root(), drive).expect("today's design builds")
}

fn sheet(t: &Today) -> String {
    t.files
        .read_to_string(&root().join("crates/vleo-mod-solar/nodes/sw_activity_band/node.toml"))
        .unwrap()
}

#[test]
fn a_release_its_owner_sealed_and_put_back_is_in_today_s_design_for_anyone() {
    let (drive, copy) = world("taken");
    // Before: solar has no release, and today's design is the design's own.
    let before = today(&drive);
    assert!(before.groups.is_empty(), "{:?}", before.groups);
    assert!(!sheet(&before).contains(SAID.trim()));
    sealed_and_put_back(&drive, &copy);
    // Anyone who opens the drive now: solar 1.0, sealed by Ola, is taken, and
    // the node is as Nia said it.
    let t = today(&drive);
    let g = t.groups.iter().find(|g| g.id == "l3_solar").unwrap();
    let k = g
        .taken
        .as_ref()
        .unwrap_or_else(|| panic!("not taken: {:?}", g.refused));
    assert_eq!(
        (k.version.as_str(), k.sealed_by.as_str()),
        ("1.0", "Test Ola")
    );
    assert_eq!(k.changes, [UID]);
    assert!(k.chain.contains("sealed by Test Ola"), "{}", k.chain);
    assert!(g.said().contains("l3_solar 1.0"), "{}", g.said());
    assert!(sheet(&t).contains(SAID.trim()), "{}", sheet(&t));
    assert!(!t.written.is_empty());
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_release_of_the_one_schema_whose_seal_does_not_check_is_refused_by_name() {
    let (drive, copy) = world("refused");
    let rel = sealed_and_put_back(&drive, &copy);
    // The release changed on the drive after it was sealed.
    let mut f = sqlite::read(&drive.join(&rel)).unwrap();
    let t = f
        .texts
        .iter_mut()
        .find(|t| t.scope == UID && t.kind == "explain.simply")
        .unwrap();
    t.body.push_str(" And changed after the seal.");
    // Its fingerprint made to agree with what it now holds: only the seal
    // tells.
    let fingerprint = vleo_files::chain::digest(&f, FILE).unwrap();
    f.meta.insert(s("fingerprint"), fingerprint);
    sqlite::write(&f, &drive.join(&rel)).unwrap();
    let t = today(&drive);
    let g = t.groups.iter().find(|g| g.id == "l3_solar").unwrap();
    assert!(g.taken.is_none());
    let (v, why) = &g.refused[0];
    assert_eq!(v, "1.0");
    assert!(
        why.contains("its seal does not check through the chain"),
        "{why}"
    );
    assert!(!sheet(&t).contains(SAID.trim()));
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

/// Design 2026.10.1, released by Test Sam from today's design on `drive`,
/// as `design::release` writes it.
fn released(drive: &Path) -> File {
    use vleo_files::design::{self, Asked, Held, Part, Taken as Took, Whole};
    let o = opened(drive);
    let rel = drive.join("groups/l3_solar/releases/l3_solar-1.0.vleo");
    let (rb, pb, sb) = (
        std::fs::read(&rel).unwrap(),
        std::fs::read(drive.join("groups/programme/programme.vgroup")).unwrap(),
        std::fs::read(drive.join("groups/systems/systems.vgroup")).unwrap(),
    );
    let file = sqlite::read(&rel).unwrap();
    let checks = design::integration(
        &[Part {
            group: "l3_solar",
            taken: Ok(Took {
                version: "1.0",
                chain: "sealed by Test Ola",
                file: &file,
            }),
        }],
        "",
    );
    let fingerprint = file.meta.get("fingerprint").cloned().unwrap_or_default();
    let held = Held {
        releases: vec![(
            "l3_solar",
            "1.0",
            &fingerprint,
            Whole {
                path: "groups/l3_solar/releases/l3_solar-1.0.vleo",
                bytes: &rb,
            },
        )],
        programme: Whole {
            path: "groups/programme/programme.vgroup",
            bytes: &pb,
        },
        systems: Whole {
            path: "groups/systems/systems.vgroup",
            bytes: &sb,
        },
    };
    let asked = Asked {
        version: "2026.10.1",
        previous: None,
        changed: "Solar 1.0.",
        marks: &[],
        understood: &[],
    };
    let d = design::release(
        &checks,
        &held,
        &asked,
        "Test Sam",
        &key(92),
        AT,
        APP,
        &o.programme,
    )
    .unwrap_or_else(|e| panic!("not released: {}", e.message()));
    design::open_held(&d, &o.programme, APP).expect("it reads back");
    d
}

#[test]
fn a_released_design_is_built_from_the_releases_it_holds() {
    let (drive, copy) = world("held");
    sealed_and_put_back(&drive, &copy);
    let held = released(&drive);
    let (design, _) = vleo_files::convert::serve(&root()).unwrap();
    let t = vleo_server::today::build_held(Arc::new(design), &root(), &drive, &held)
        .unwrap_or_else(|e| panic!("not built: {e}"));
    // As today's design takes solar 1.0, so does the design built from it.
    let today = today(&drive);
    assert_eq!(t.written, today.written);
    assert!(sheet(&t).contains(SAID.trim()));
    let g = &t.groups[0];
    let k = g.taken.as_ref().unwrap();
    assert_eq!((g.id.as_str(), k.version.as_str()), ("l3_solar", "1.0"));
    assert!(k.chain.contains("sealed by Test Ola"), "{}", k.chain);
    assert!(
        t.base.said.contains("the released design 2026.10.1"),
        "{}",
        t.base.said
    );

    // A release held whose content changed after its seal, the design
    // signed again over it: refused by name, and nothing taken in its place.
    let mut bad = held.clone();
    let path = "groups/l3_solar/releases/l3_solar-1.0.vleo";
    let mut f = sqlite::read(&drive.join(path)).unwrap();
    f.texts
        .iter_mut()
        .find(|t| t.scope == UID && t.kind == "explain.simply")
        .unwrap()
        .body
        .push_str(" And changed after the seal.");
    let at = drive.parent().unwrap().join("changed.vleo");
    let m = bad.media.iter_mut().find(|m| m.path == path).unwrap();
    m.bytes = sqlite::written(&f, &at).unwrap();
    m.sha256 = vleo_files::keys::hex(&vleo_files::keys::sha256(&m.bytes));
    let sha = m.sha256.clone();
    let old = held
        .media
        .iter()
        .find(|m| m.path == path)
        .unwrap()
        .sha256
        .clone();
    for t in bad.tables.iter_mut().filter(|t| t.path == "released.csv") {
        t.csv = t.csv.replace(&old, &sha);
    }
    bad.signatures.retain(|r| r.scope != FILE);
    let row = chain::sign(&bad, FILE, "Test Sam", &key(92), AT, "ok", "").unwrap();
    bad.signatures.push(row);
    let built = |d: &File| {
        let (design, _) = vleo_files::convert::serve(&root()).unwrap();
        vleo_server::today::build_held(Arc::new(design), &root(), &drive, d)
    };
    let e = built(&bad).err().expect("refused");
    assert!(
        e.contains(&format!(
            "holds {path}, whose fingerprint is not its content's"
        )),
        "{e}"
    );
    // A drive whose chain does not open builds no released design.
    let pf = drive.join("groups/programme/programme.vgroup");
    std::fs::rename(&pf, drive.join("programme.aside")).unwrap();
    let e = built(&held).err().expect("refused");
    assert!(
        e.contains("a released design is built only through the chain"),
        "{e}"
    );
    std::fs::rename(drive.join("programme.aside"), &pf).unwrap();
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

/// As [`sealed_and_put_back`], with `edit` made to Nia's node before she
/// signs it.
fn sealed_with(drive: &Path, copy: &Path, edit: &dyn Fn(&mut File)) -> String {
    let mut n = sqlite::read(&copy.join(NODE)).unwrap();
    n.meta.insert(s("revision"), s("2"));
    n.blocks[0].revision = 2;
    let t = n
        .texts
        .iter_mut()
        .find(|t| t.scope == UID && t.kind == "explain.simply")
        .unwrap();
    t.body.push_str(SAID);
    edit(&mut n);
    sqlite::write(&n, &copy.join(NODE)).unwrap();
    let seen = copied::digest(&std::fs::read(copy.join(NODE)).unwrap());
    let o = opened(copy);
    match sign_in_place(copy, NODE, &seen, "Test Nia", &key(94), AT, APP, &o.solar).unwrap() {
        Saved::Written { .. } => {}
        other => panic!("Nia did not sign: {other:?}"),
    }
    let rel = vleo_files::release::seal_in_place(
        copy,
        "l3_solar",
        "1.0",
        "Test Ola",
        &key(93),
        AT,
        APP,
        &o.systems,
        &o.programme,
    )
    .unwrap();
    let open = || {
        let o = opened(drive);
        Ok(Chain {
            anchor: o.anchor,
            programme: o.programme,
            systems: Some(o.systems),
            groups: vec![o.solar],
            left_out: Vec::new(),
        })
    };
    let digest = copied::digest(&std::fs::read(copy.join(&rel)).unwrap());
    let r = put_back(
        copy,
        drive,
        "Test Ola",
        "2026-10-16T09:00:00Z",
        &open,
        Some(&[(rel.clone(), digest)]),
    )
    .unwrap();
    assert!(r.written, "{:?}", r.lines);
    rel
}

fn refused_why(t: &Today) -> String {
    let g = t.groups.iter().find(|g| g.id == "l3_solar").unwrap();
    assert!(g.taken.is_none(), "taken: {:?}", g.taken);
    g.refused[0].1.clone()
}

#[test]
fn who_sealed_it_and_when_are_its_seal_s_never_its_unsigned_meta() {
    let (drive, copy) = world("seal-of");
    let rel = sealed_and_put_back(&drive, &copy);
    let mut f = sqlite::read(&drive.join(&rel)).unwrap();
    let fingerprint = vleo_files::chain::digest(&f, FILE).unwrap();
    for (k, v) in [
        ("sealed_by", "Test Pat"),
        ("sealed_at", "2000-01-01T00:00:00Z"),
    ] {
        f.meta.insert(s(k), s(v));
    }
    sqlite::write(&f, &drive.join(&rel)).unwrap();
    let t = today(&drive);
    let k = t.groups[0].taken.as_ref().expect("its seal still holds");
    assert_eq!(
        (
            k.sealed_by.as_str(),
            k.sealed.as_str(),
            k.fingerprint.as_str()
        ),
        ("Test Ola", AT, fingerprint.as_str())
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_release_is_known_only_by_the_fingerprint_of_what_it_holds() {
    let (drive, copy) = world("relabelled");
    let rel = sealed_and_put_back(&drive, &copy);
    let held = sqlite::read(&drive.join(&rel)).unwrap();
    // Its seal does not cover its fingerprint: relabelled, or with none, the
    // seal still holds, and it is refused all the same.
    for (label, says) in [
        (
            "0000000000000000000000000000000000000000000000000000000000000000",
            "its fingerprint is 0000",
        ),
        ("", "says no fingerprint"),
    ] {
        let mut f = held.clone();
        f.meta.insert(s("fingerprint"), s(label));
        sqlite::write(&f, &drive.join(&rel)).unwrap();
        let why = refused_why(&today(&drive));
        assert!(why.contains(says), "{label:?}: {why}");
    }
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_release_of_the_one_schema_upgraded_from_before_1_0_is_refused() {
    let (drive, copy) = world("upgraded");
    let rel = sealed_and_put_back(&drive, &copy);
    let mut f = sqlite::read(&drive.join(&rel)).unwrap();
    f.meta.insert(s("upgraded_from"), s("format 1"));
    sqlite::write(&f, &drive.join(&rel)).unwrap();
    let why = refused_why(&today(&drive));
    assert!(
        why.contains("a release from before 1.0 written in 1.0's one schema"),
        "{why}"
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_release_that_renames_a_node_is_refused() {
    // The copy's own signing refuses a node renamed in its file, so the
    // release is made as the library would make it from such a file: Nia
    // signs the renamed node, Ola seals it, and it is put on the drive.
    let (drive, copy) = world("renamed");
    let o = opened(&copy);
    let mut n = sqlite::read(&copy.join(NODE)).unwrap();
    n.blocks[0].id = s("sw_activity_level");
    let (n, _) = chain::sign_node(&n, UID, "Test Nia", &key(94), AT, &o.solar).unwrap();
    let group = sqlite::read(&copy.join(SOLAR)).unwrap();
    let r = vleo_files::release::seal(
        &group,
        &[n],
        "1.0",
        None,
        "Test Ola",
        &key(93),
        AT,
        APP,
        &o.systems,
        &o.programme,
    )
    .unwrap();
    let dir = drive.join("groups/l3_solar/releases");
    std::fs::create_dir_all(&dir).unwrap();
    sqlite::write(&r, &dir.join("l3_solar-1.0.vleo")).unwrap();
    let why = refused_why(&today(&drive));
    assert!(
        why.contains("a node is renamed through its group's breakdown"),
        "{why}"
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn what_a_release_s_node_no_longer_holds_is_not_served_from_the_design() {
    let (drive, copy) = world("dropped");
    sealed_with(&drive, &copy, &|n| {
        n.cases.clear();
        n.texts
            .retain(|t| t.kind != "fixtures" && t.kind != "as converted: fixtures.toml");
    });
    let fixtures = root().join("crates/vleo-mod-solar/nodes/sw_activity_band/fixtures.toml");
    let t = today(&drive);
    let g = t.groups.iter().find(|g| g.id == "l3_solar").unwrap();
    match &g.taken {
        Some(_) => assert!(
            !t.files.is_file(&fixtures),
            "the design's cases are still served"
        ),
        None => panic!("refused: {:?}", g.refused),
    }
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}
