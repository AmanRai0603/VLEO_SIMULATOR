//! A preview of a release that moves other groups' values, and the answers
//! to it (docs/briefs/F6.md, F6-c items 3 and 4; W7).
//!
//! The released design's own programme of invented people, with Uma, who
//! owns the orbit's environment and the drag: Pat the programme manager,
//! Sam the system engineer and Sid their deputy, Ola who owns solar and
//! radiation, Nia a node engineer of solar. Design 2026.10.1 holds solar
//! 1.2; solar 1.3 is sealed after it. What moved is written in the test by
//! hand, as the engine would give it: nothing here is computed, and no
//! number is the code's own.
//!
//! - Sam previews solar 1.3 on 2026.10.1: the release held whole, every
//!   value that moved, and the orbit's environment and the drag named to
//!   answer; the systems' branch, radiation, which solar's owner owns too,
//!   thermal, mounted for the system engineer, and a group nothing mounts
//!   are shown and not named. It opens through
//!   the chain, every number read back as the same bits. Sid writes one too;
//! - refused by name: by Ola, under an assistant's name, with a key the
//!   programme does not register, for a release the design holds already,
//!   for one that moves nothing, and for one not where the drive keeps it;
//! - changed after it was written, or not as a preview writes it, it does
//!   not open, by name;
//! - each group named says fine, object and why, unanswered, or refused and
//!   why: an answer to another release, changed after it was signed, or
//!   written as another group's.

use std::path::Path;

use vleo_files::answer::{self, Said};
use vleo_files::chain::{self, Registry, FILE};
use vleo_files::design::{self, Answered, Held, Move, Part, Taken, Whole};
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

// Pat the programme manager (11), Sam the system engineer (12), Sid their
// deputy (17), Ola who owns solar and radiation (13), Dee their deputy (14),
// Nia a node engineer of solar (16), Uma who owns the orbit's environment
// and the drag (18).
fn programme() -> Registry {
    let mut f = group_file("programme");
    register(&mut f, "Pat Morgan", "programme manager", "", &key(11));
    register(&mut f, "Sam Engel", "system engineer", "", &key(12));
    register(&mut f, "Sid Side", "system engineer", "Sam Engel", &key(17));
    register(&mut f, "Ola Sun", "subsystem engineer", "", &key(13));
    register(
        &mut f,
        "Dee Puty",
        "subsystem engineer",
        "Ola Sun",
        &key(14),
    );
    register(&mut f, "Uma Orbit", "subsystem engineer", "", &key(18));
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
    mount(&mut f, "sys_orbit", "l3_x_envorbit", "Uma Orbit");
    mount(&mut f, "sys_drag", "l3_drag", "Uma Orbit");
    mount(&mut f, "sys_radiation", "l3_radiation", "Ola Sun");
    mount(&mut f, "sys_thermal", "l3_thermal", "Sam Engel");
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
             1.2,2026-10-10,Ola Sun,The bands held.,Read again.,They held.,A method said plainly.,,,the four NOAA SWPC activity levels,a correction to those levels\n\
             1.3,2026-10-18,Ola Sun,The bands held.,Read once more.,They held.,The question said plainly.,,,the four NOAA SWPC activity levels,a correction to those levels\n"),
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

/// `f` as the drive holds it: its bytes as a file of the one schema.
fn bytes(f: &File) -> Vec<u8> {
    let dir = std::env::temp_dir().join(format!(
        "vleo-design-bytes-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let at = dir.join("f.vleo");
    let _ = std::fs::remove_file(&at);
    let b = vleo_files::sqlite::written(f, &at).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    b
}

/// Solar 1.2, sealed by Ola from the band Nia signed.
fn solar_12(w: &World) -> File {
    seal(w, &solar(), &[band(w)], "1.2", Some("1.1"), "Ola Sun", 13).unwrap()
}

/// Solar 1.3, sealed after 1.2 from the band again, its question said
/// plainly.
fn solar_13(w: &World) -> File {
    let mut b = from_release(BAND);
    b.blocks[0].question = s("Which of the four activity levels is the sun at?");
    let reg = chain::open_group_in(&solar(), &w.sy, &w.pr).unwrap();
    let b = chain::sign_node(&b, BAND, "Nia Node", &key(16), AT, &reg)
        .expect("Nia signs the band")
        .0;
    seal(w, &solar(), &[b], "1.3", Some("1.2"), "Ola Sun", 13).unwrap()
}

/// Design 2026.10.1, holding solar 1.2, released by Sam and read back.
fn released(w: &World) -> design::Opened {
    let r = solar_12(w);
    let i = design::integration(
        &[Part {
            group: "l3_solar",
            taken: Ok(Taken {
                version: "1.2",
                chain: "sealed by Ola Sun, its owner as the systems' file mounts it",
                file: &r,
            }),
        }],
        "",
    );
    let (rb, pb, sb) = (bytes(&r), bytes(&w.pr.file), bytes(&w.sy.file));
    let held = Held {
        releases: vec![(
            "l3_solar",
            "1.2",
            r.meta.get("fingerprint").map_or("", String::as_str),
            Whole {
                path: "groups/l3_solar/releases/l3_solar-1.2.vleo",
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
    let asked = design::Asked {
        version: "2026.10.1",
        previous: None,
        changed: "Solar's bands.",
        marks: &[],
        understood: &[],
        margins: &[],
    };
    let d = design::release(&i, &held, &asked, "Sam Engel", &key(12), AT, APP, &w.pr).unwrap();
    design::open_held(&d, &w.pr, APP).unwrap()
}

fn moved(value: &str, group: &str, before: Option<f64>, after: Option<f64>) -> Move {
    Move {
        value: s(value),
        node: s(value.split('.').next().unwrap()),
        group: s(group),
        before,
        after,
        before_why: before
            .is_none()
            .then(|| s("it needs the solar-drivers data")),
        after_why: after
            .is_none()
            .then(|| s("it needs the solar-drivers data")),
    }
}

/// What taking solar 1.3 into the design moves, as the engine would say it:
/// two of the orbit's environment's values, one first answered with it; the
/// drag's; the systems' own; radiation's; thermal's, mounted for the system
/// engineer; and one of a group nothing mounts.
fn what_moved() -> Vec<Move> {
    vec![
        moved("env_f107", "l3_x_envorbit", Some(150.0), Some(151.5)),
        moved(
            "env_exospheric_temperature.t_inf",
            "l3_x_envorbit",
            None,
            Some(0.1 + 0.2),
        ),
        moved("drag_density", "l3_drag", Some(2.5e-11), Some(2.6e-11)),
        moved(
            "sys_space_environment_f10_7",
            "sys_space_environment",
            Some(150.0),
            Some(151.5),
        ),
        moved("rad_dose", "l3_radiation", Some(1.0), Some(1.25)),
        moved("thermal_margin", "l3_thermal", Some(4.0), Some(3.5)),
        moved("ghost_value", "l3_ghost", Some(1.0), Some(2.0)),
    ]
}

fn preview_of(
    w: &World,
    d: &design::Opened,
    r: &File,
    moves: &[Move],
    by: &str,
    k: &SigningKey,
) -> Result<File, String> {
    let b = bytes(r);
    let path = format!(
        "groups/l3_solar/releases/l3_solar-{}.vleo",
        r.meta["version"]
    );
    design::preview(
        &Whole {
            path: &path,
            bytes: &b,
        },
        d,
        moves,
        &[&w.pr, &w.sy],
        by,
        k,
        AT,
        APP,
        &w.pr,
    )
    .map_err(|e| e.message().to_string())
}

fn table<'a>(f: &'a File, path: &str) -> &'a str {
    &f.tables.iter().find(|t| t.path == path).unwrap().csv
}

#[test]
fn sam_previews_a_release_that_moves_other_groups_and_names_their_owners() {
    let w = world();
    let d = released(&w);
    let r = solar_13(&w);
    let p = preview_of(&w, &d, &r, &what_moved(), "Sam Engel", &key(12))
        .expect("Sam writes the preview");
    assert_eq!(p.kind().unwrap(), Kind::Preview);
    for (k, v) in [
        ("group_id", "l3_solar"),
        ("version", "1.3"),
        ("based_on", "2026.10.1"),
        ("writer", "Sam Engel"),
    ] {
        assert_eq!(p.meta.get(k).map(String::as_str), Some(v), "{k}");
    }
    assert_eq!(
        chain::preview_path("l3_solar", "1.3", "2026.10.1"),
        "integration/l3_solar-1.3/preview-l3_solar-1.3-on-2026.10.1.vleo"
    );
    // The release whole, by its bytes and their SHA-256, and nothing beside.
    assert_eq!(p.media.len(), 1);
    assert_eq!(
        p.media[0].path,
        "groups/l3_solar/releases/l3_solar-1.3.vleo"
    );
    assert_eq!(p.media[0].bytes, bytes(&r));
    // Every value that moved; the subsystems' groups whose owners answer.
    assert_eq!(
        table(&p, "moved.csv"),
        "value,node,group,before,after,before_why,after_why\n\
         env_f107,env_f107,l3_x_envorbit,150.0,151.5,,\n\
         env_exospheric_temperature.t_inf,env_exospheric_temperature,l3_x_envorbit,,0.30000000000000004,it needs the solar-drivers data,\n\
         drag_density,drag_density,l3_drag,2.5e-11,2.6e-11,,\n\
         sys_space_environment_f10_7,sys_space_environment_f10_7,sys_space_environment,150.0,151.5,,\n\
         rad_dose,rad_dose,l3_radiation,1.0,1.25,,\n\
         thermal_margin,thermal_margin,l3_thermal,4.0,3.5,,\n\
         ghost_value,ghost_value,l3_ghost,1.0,2.0,,\n"
    );
    assert_eq!(table(&p, "named.csv"), "group\nl3_drag\nl3_x_envorbit\n");
    chain::check_preview(&p, &w.pr).expect("the preview opens through the chain");
    let o = design::open_preview(&p, &[&w.pr, &w.sy], &w.pr, APP).expect("it reads back");
    assert_eq!(
        (o.group.as_str(), o.version.as_str(), o.based_on.as_str()),
        ("l3_solar", "1.3", "2026.10.1")
    );
    assert_eq!(o.fingerprint, r.meta["fingerprint"]);
    assert_eq!(o.named, ["l3_drag", "l3_x_envorbit"]);
    // Every number read back is the engine's, bit for bit.
    for (got, was) in o.moved.iter().zip(what_moved()) {
        assert_eq!(got.before.map(f64::to_bits), was.before.map(f64::to_bits));
        assert_eq!(got.after.map(f64::to_bits), was.after.map(f64::to_bits));
        assert_eq!((&got.value, &got.group), (&was.value, &was.group));
        assert_eq!(got.before_why, was.before_why);
    }
    // Sid, Sam's deputy, writes one too.
    let p = preview_of(&w, &d, &r, &what_moved(), "Sid Side", &key(17))
        .expect("Sam's deputy writes the preview");
    chain::check_preview(&p, &w.pr).unwrap();
    // Moving only what nobody answers for, it names nobody.
    let p = preview_of(&w, &d, &r, &what_moved()[3..], "Sam Engel", &key(12)).unwrap();
    assert_eq!(table(&p, "named.csv"), "group\n");
}

#[test]
fn a_preview_is_refused_by_name() {
    let w = world();
    let d = released(&w);
    let r = solar_13(&w);
    let all = what_moved();
    for (by, k, why) in [
        (
            "Ola Sun",
            key(13),
            "Ola Sun may not write a preview: the system engineer, or their deputy, writes one",
        ),
        ("Claude", key(12), "is an assistant's name"),
        (
            "Sam Engel",
            key(99),
            "the key Sam Engel signed with is not one the programme's file registers",
        ),
    ] {
        let e = preview_of(&w, &d, &r, &all, by, &k).unwrap_err();
        assert!(e.contains(why), "{by}: {e}");
    }
    let e = preview_of(&w, &d, &solar_12(&w), &all, "Sam Engel", &key(12)).unwrap_err();
    assert!(
        e.contains("design 2026.10.1 holds l3_solar 1.2 already: there is nothing to preview"),
        "{e}"
    );
    let mut unsealed = r.clone();
    unsealed.signatures.retain(|row| row.scope != FILE);
    let e = preview_of(&w, &d, &unsealed, &all, "Sam Engel", &key(12)).unwrap_err();
    assert!(e.contains("has no signature"), "{e}");
    let e = preview_of(&w, &d, &r, &[], "Sam Engel", &key(12)).unwrap_err();
    assert!(
        e.contains(
            "l3_solar 1.3 moves nothing outside l3_solar on design 2026.10.1: it needs no preview"
        ),
        "{e}"
    );
    let e = preview_of(
        &w,
        &d,
        &r,
        &[moved("sw_activity_band", "l3_solar", Some(1.0), Some(2.0))],
        "Sam Engel",
        &key(12),
    )
    .unwrap_err();
    assert!(e.contains("of l3_solar itself, moved"), "{e}");
    let b = bytes(&r);
    let e = design::preview(
        &Whole {
            path: "groups/l3_solar/releases/latest.vleo",
            bytes: &b,
        },
        &d,
        &all,
        &[&w.pr, &w.sy],
        "Sam Engel",
        &key(12),
        AT,
        APP,
        &w.pr,
    )
    .unwrap_err();
    assert!(
        e.message()
            .contains("which the drive keeps as groups/l3_solar/releases/l3_solar-1.3.vleo"),
        "{e}"
    );
}

/// `p` changed by `change` and signed again by Sam, so only what it holds
/// is wrong: why it does not open.
fn resigned(w: &World, p: &File, change: impl Fn(&mut File)) -> String {
    let mut f = p.clone();
    change(&mut f);
    f.signatures.retain(|r| r.scope != FILE);
    let row = chain::sign(&f, FILE, "Sam Engel", &key(12), AT, "ok", "").unwrap();
    f.signatures.push(row);
    let e = design::open_preview(&f, &[&w.pr, &w.sy], &w.pr, APP).unwrap_err();
    assert_eq!(
        chain::check_preview(&f, &w.pr).is_err(),
        !e.message().contains("holds groups/"),
        "{e}"
    );
    e.message().to_string()
}

#[test]
fn a_preview_changed_or_not_as_a_preview_writes_it_does_not_open() {
    let w = world();
    let d = released(&w);
    let r = solar_13(&w);
    let p = preview_of(&w, &d, &r, &what_moved(), "Sam Engel", &key(12)).unwrap();
    let mut bare = r.clone();
    bare.signatures.retain(|row| row.scope != FILE);
    let unsealed = bytes(&bare);
    // Changed after Sam signed it.
    let mut f = p.clone();
    f.tables[0].csv = f.tables[0].csv.replace("151.5", "152.5");
    let e = chain::check_preview(&f, &w.pr).unwrap_err();
    assert!(e.message().contains("has no signature that counts"), "{e}");
    // Its release's bytes, not the ones held with it.
    let mut f = p.clone();
    f.media[0].bytes.push(0);
    let e = chain::check_preview(&f, &w.pr).unwrap_err();
    assert!(e.message().contains("their SHA-256 differs"), "{e}");
    // Not as a preview writes it, signed again by Sam.
    for (change, why) in [
        (
            Box::new(|f: &mut File| f.tables[1].csv.push_str("l3_power\n"))
                as Box<dyn Fn(&mut File)>,
            "names l3_power, and no value of l3_power moved",
        ),
        (
            Box::new(|f: &mut File| f.tables[1].csv.push_str("l3_drag\n")),
            "names l3_drag twice",
        ),
        (
            Box::new(|f: &mut File| f.tables[1].csv.push_str("systems\n")),
            "names \"systems\" to answer",
        ),
        (
            Box::new(|f: &mut File| {
                f.tables[0]
                    .csv
                    .push_str("sw_activity_band,sw_activity_band,l3_solar,1.0,2.0,,\n")
            }),
            "of l3_solar itself, moved",
        ),
        (
            Box::new(|f: &mut File| f.tables[0].csv.push_str("x,x,l3_drag,1.0,1.0,,\n")),
            "says x moved, and its two answers are one",
        ),
        (
            Box::new(|f: &mut File| f.tables[0].csv.push_str("x,x,l3_drag,1.0,heavy,,\n")),
            "says \"heavy\" is an answer, and it is not a number",
        ),
        (
            Box::new(|f: &mut File| f.tables[0].csv.push_str("x.y,z,l3_drag,1.0,2.0,,\n")),
            "says \"x.y\" moved in \"z\", which is not a node's value",
        ),
        (
            Box::new(|f: &mut File| f.tables[0].csv = s("value,node,group,before,after\n")),
            "holds a moved.csv that is not as a preview writes it",
        ),
        (
            Box::new(|f: &mut File| {
                f.tables[0].csv = s("value,node,group,before,after,before_why,after_why\n");
                f.tables[1].csv = s("group\n");
            }),
            "says nothing moved",
        ),
        (
            Box::new(|f: &mut File| {
                f.tables.push(Tbl {
                    scope: s(FILE),
                    path: s("extra.csv"),
                    csv: s("a\n1\n"),
                })
            }),
            "holds tables beside moved.csv and named.csv",
        ),
        (
            Box::new(|f: &mut File| {
                let mut m = f.media[0].clone();
                m.path = s("groups/l3_solar/releases/l3_solar-1.2.vleo");
                f.media.push(m);
            }),
            "holds 2 files",
        ),
        (
            Box::new(|f: &mut File| f.blocks.push(Block::default())),
            "holds block rows",
        ),
        (
            Box::new(|f: &mut File| f.media[0].path = s("groups/l3_solar/releases/latest.vleo")),
            "holds 1 file: a preview holds the one release it previews, whole, as groups/l3_solar/releases/l3_solar-1.3.vleo",
        ),
        (
            Box::new(move |f: &mut File| {
                f.media[0].bytes = unsealed.clone();
                f.media[0].sha256 = vleo_files::keys::hex(&vleo_files::keys::sha256(&unsealed));
            }),
            "holds groups/l3_solar/releases/l3_solar-1.3.vleo: l3_solar's release, by its owner Ola Sun, has no signature",
        ),
        (
            Box::new(|f: &mut File| {
                f.meta.insert(s("writer"), s("Sid Side"));
            }),
            "written by Sid Side as the system engineer, has no signature that counts",
        ),
        (
            Box::new(|f: &mut File| {
                f.meta.insert(s("version"), s("1.2"));
                f.media[0].path = s("groups/l3_solar/releases/l3_solar-1.2.vleo");
            }),
            "which is not l3_solar's release 1.2",
        ),
    ] {
        let e = resigned(&w, &p, change);
        assert!(e.contains(why), "{why}: {e}");
    }
}

#[test]
fn each_group_named_says_fine_object_unanswered_or_refused() {
    let w = world();
    let d = released(&w);
    let r = solar_13(&w);
    let p = preview_of(&w, &d, &r, &what_moved(), "Sam Engel", &key(12)).unwrap();
    let o = design::open_preview(&p, &[&w.pr, &w.sy], &w.pr, APP).unwrap();
    let parents = [&w.pr, &w.sy];
    let answering = |release: &File, g: &str, said: &Said| {
        answer::answer(
            release,
            g,
            said,
            "Uma Orbit",
            &key(18),
            AT,
            APP,
            &w.sy,
            &w.pr,
        )
        .unwrap()
    };
    let read_from = |held: Vec<(String, File)>| {
        move |path: &str| {
            held.iter()
                .find(|(p, _)| p == path)
                .map(|(_, f)| Ok(f.clone()))
        }
    };
    let at = |g: &str| answer::path_of("l3_solar", "1.3", g);
    // None yet: both unanswered.
    let none = design::answers(&o, &read_from(Vec::new()), &parents, &w.pr);
    assert_eq!(
        none,
        [
            (s("l3_drag"), Answered::Unanswered),
            (s("l3_x_envorbit"), Answered::Unanswered)
        ]
    );
    // Fine, and an objection with why.
    let because = s("The density at solar maximum leaves no margin on the drag.");
    let read = read_from(vec![
        (
            at("l3_x_envorbit"),
            answering(&r, "l3_x_envorbit", &Said::Fine),
        ),
        (
            at("l3_drag"),
            answering(&r, "l3_drag", &Said::Object(because.clone())),
        ),
    ]);
    assert_eq!(
        design::answers(&o, &read, &parents, &w.pr),
        [
            (s("l3_drag"), Answered::Object(because)),
            (s("l3_x_envorbit"), Answered::Fine)
        ]
    );
    // An answer to solar 1.2, put where 1.3's is: refused, never fine.
    let mut to_12 = answering(&solar_12(&w), "l3_x_envorbit", &Said::Fine);
    to_12.meta.insert(s("version"), s("1.3"));
    // Changed after Uma signed it.
    let mut changed = answering(&r, "l3_drag", &Said::Fine);
    changed.texts[0].body = s("object");
    changed.texts.push(Text {
        scope: s(FILE),
        kind: s("because"),
        body: s("said by nobody who signed it"),
    });
    let read = read_from(vec![(at("l3_x_envorbit"), to_12), (at("l3_drag"), changed)]);
    let got = design::answers(&o, &read, &parents, &w.pr);
    match &got[1] {
        (g, Answered::Refused(why)) => {
            assert_eq!(g, "l3_x_envorbit");
            assert!(
                why.contains("it answers a release whose fingerprint is")
                    && why.contains("and the preview holds l3_solar 1.3 as"),
                "{why}"
            );
        }
        other => panic!("{other:?}"),
    }
    match &got[0] {
        (g, Answered::Refused(why)) => {
            assert_eq!(g, "l3_drag");
            assert!(why.contains("has no signature that counts"), "{why}");
        }
        other => panic!("{other:?}"),
    }
    // The drag's answer, put where the orbit's should be.
    let read = read_from(vec![(
        at("l3_x_envorbit"),
        answering(&r, "l3_drag", &Said::Fine),
    )]);
    let got = design::answers(&o, &read, &parents, &w.pr);
    assert!(
        matches!(&got[1], (_, Answered::Refused(why)) if why.contains("it says it is l3_drag's answer to l3_solar 1.3, not l3_x_envorbit's")),
        "{got:?}"
    );
    // A file there that does not read is refused, said so.
    let bad = |_: &str| Some(Err(s("not a database")));
    let got = design::answers(&o, &bad, &parents, &w.pr);
    assert!(
        matches!(&got[0], (_, Answered::Refused(why)) if why.contains("does not read: not a database")),
        "{got:?}"
    );
}
