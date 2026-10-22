//! An objection is decided, and the design is released only once every
//! answer is in (docs/briefs/F6.md, F6-c items 5 and 6; W7; W8, its step 2).
//!
//! The preview's own programme of invented people: Pat the programme
//! manager, Sam the system engineer and Sid their deputy, Ola who owns solar
//! and radiation, Dee their deputy, Nia a node engineer of solar, Uma who
//! owns the orbit's environment and the drag. Design 2026.10.1 holds solar
//! 1.2; today's design takes solar 1.3, based on it, and its preview names
//! the drag and the orbit's environment. What each group said is written in
//! the test by hand, as their answers would say it: nothing is computed.
//!
//! - the design is refused while a group named has not answered, or answered
//!   by a file that does not count; while a preview that may be one does not
//!   open; while an objection has no decision; and, decided *next version*,
//!   while today's design still holds solar 1.3. An objection is never
//!   marked understood;
//! - released anyway, the objection is a red mark raised by the programme
//!   manager's issue, and the released design keeps it in `objections.csv`;
//! - a preview on another design, or of a release today's design does not
//!   hold, asks nothing;
//! - Sam, or Sid, decides by an issue: on solar to Ola for its next version,
//!   or on the programme to Pat to release anyway; refused for Ola and under
//!   an assistant's name;
//! - a decision is read only from an issue the system engineer raised and
//!   signed, at the objection's place, naming its answer by its digest, its
//!   signed text saying what its row says;
//! - or from a reallocation (F6-e): a `reallocate` change row of the systems'
//!   file as it opens through the chain, its rows signed with it, made by Sam
//!   or Sid, naming the objection and its answer. Released, it is no red
//!   mark, and `objections.csv` keeps it.

use std::path::Path;

use vleo_files::answer::{self, Said};
use vleo_files::chain::{self, Registry, FILE};
use vleo_files::design::{
    self, Answered, Decide, Decided, Decision, Held, Objection, Part, Standing, Taken, Whole,
};
use vleo_files::keys::SigningKey;
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::release;

const APP: &str = "vleo 1.0.0 (test)";
const AT: &str = "2026-10-17T09:00:00Z";
const BAND: &str = "sw_activity_band";
const RELEASED: &str = "2026.10.1";
const NEXT: &str = "2026.10.2";
const DRAG: &str = "objection:l3_solar-1.3.l3_drag";

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

/// Solar 1.3, based on design 2026.10.1, sealed after 1.2 from the band
/// again, its question said plainly.
fn solar_13(w: &World) -> File {
    let mut b = from_release(BAND);
    b.blocks[0].question = s("Which of the four activity levels is the sun at?");
    let reg = chain::open_group_in(&solar(), &w.sy, &w.pr).unwrap();
    let b = chain::sign_node(&b, BAND, "Nia Node", &key(16), AT, &reg)
        .expect("Nia signs the band")
        .0;
    let mut g = solar_unsigned(1);
    g.meta.insert(s("based_on"), s(RELEASED));
    let g = signed(g, "Ola Sun", &key(13));
    seal(w, &g, &[b], "1.3", Some("1.2"), "Ola Sun", 13).unwrap()
}

/// What each group named said, as a preview stands for the release.
fn standing(based_on: &str, version: &str, drag: Answered, orbit: Answered) -> Standing {
    Standing {
        group: s("l3_solar"),
        version: s(version),
        based_on: s(based_on),
        answers: vec![(s("l3_drag"), drag), (s("l3_x_envorbit"), orbit)],
    }
}

const BECAUSE: &str = "The density at solar maximum leaves the drag no margin.";

fn objection() -> Objection {
    Objection {
        group: s("l3_solar"),
        version: s("1.3"),
        answering: s("l3_drag"),
        because: s(BECAUSE),
        answer: s("a1".repeat(32).as_str()),
    }
}

/// What was decided of `objection()`, read by `design::decisions` from the
/// issue `number` that `by`, with key `k`, raised deciding `d`, or from none.
fn decided(w: &World, d: Option<(Decide, i64, &str, u8)>) -> Vec<Decided> {
    let issues: Vec<File> = d
        .into_iter()
        .map(|(d, number, by, k)| {
            design::decision_issue(
                &objection(),
                d,
                number,
                by,
                &key(k),
                AT,
                APP,
                &[&w.pr, &w.sy],
                &w.pr,
            )
            .unwrap()
        })
        .collect();
    design::decisions(&[objection()], &issues, None, &[&w.pr, &w.sy], &w.pr)
}

/// Design 2026.10.2, after 2026.10.1, from today's design taking solar 1.3,
/// as the previews `previews` stand and with `decided`, released by Sam.
fn release_it(
    w: &World,
    r: &File,
    previews: &[Standing],
    decided: &[Decided],
    understood: &[String],
) -> Result<File, String> {
    release_unread(w, r, previews, &[], decided, understood)
}

/// As `release_it`, while `unread` says what previews do not open.
fn release_unread(
    w: &World,
    r: &File,
    previews: &[Standing],
    unread: &[String],
    decided: &[Decided],
    understood: &[String],
) -> Result<File, String> {
    let i = design::integration(
        &[Part {
            group: "l3_solar",
            taken: Ok(Taken {
                version: "1.3",
                chain: "sealed by Ola Sun, its owner as the systems' file mounts it",
                file: r,
            }),
        }],
        RELEASED,
    );
    assert!(i.holds(), "{:?}", i.failing());
    let (rb, pb, sb) = (bytes(r), bytes(&w.pr.file), bytes(&w.sy.file));
    let held = Held {
        releases: vec![(
            "l3_solar",
            "1.3",
            r.meta.get("fingerprint").map_or("", String::as_str),
            Whole {
                path: "groups/l3_solar/releases/l3_solar-1.3.vleo",
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
        version: NEXT,
        previous: Some(RELEASED),
        changed: "Solar's question said plainly.",
        marks: &[],
        understood,
        margins: &[],
        previews,
        unread,
        decided,
    };
    design::release(&i, &held, &asked, "Sam Engel", &key(12), AT, APP, &w.pr)
        .map_err(|e| e.message().to_string())
}

fn table<'a>(f: &'a File, path: &str) -> &'a str {
    &f.tables.iter().find(|t| t.path == path).unwrap().csv
}

#[test]
fn the_design_is_released_only_once_every_answer_is_in_and_every_objection_decided() {
    let w = world();
    let r = solar_13(&w);
    let object = || Answered::Object(s(BECAUSE));
    // Unanswered, and answered by a file that does not count.
    let e = release_it(
        &w,
        &r,
        &[standing(
            RELEASED,
            "1.3",
            Answered::Unanswered,
            Answered::Fine,
        )],
        &[],
        &[],
    )
    .unwrap_err();
    assert!(
        e.contains("every answer is in before the design is released (W8)")
            && e.contains("l3_drag has not answered the preview of l3_solar 1.3"),
        "{e}"
    );
    let e = release_it(
        &w,
        &r,
        &[standing(
            RELEASED,
            "1.3",
            Answered::Fine,
            Answered::Refused(s("its signature does not count")),
        )],
        &[],
        &[],
    )
    .unwrap_err();
    assert!(
        e.contains("l3_x_envorbit's answer to the preview of l3_solar 1.3 does not count: its signature does not count"),
        "{e}"
    );
    // A preview that may be one does not open, every answer given.
    let fine = [standing(RELEASED, "1.3", Answered::Fine, Answered::Fine)];
    let e = release_unread(
        &w,
        &r,
        &fine,
        &[s("its signature does not check")],
        &[],
        &[],
    )
    .unwrap_err();
    assert!(
        e.contains("every answer is in before the design is released (W8)")
            && e.contains("a preview that may be one does not open: its signature does not check"),
        "{e}"
    );
    // An objection with no decision.
    let previews = [standing(RELEASED, "1.3", object(), Answered::Fine)];
    let e = release_it(&w, &r, &previews, &decided(&w, None), &[]).unwrap_err();
    assert!(
        e.contains("every objection is decided before the design is released (W7)")
            && e.contains(&format!("l3_drag objects to l3_solar 1.3: {BECAUSE}")),
        "{e}"
    );
    // Its next version asked for: not while today's design holds 1.3.
    let next = decided(&w, Some((Decide::NextVersion, 7, "Sam Engel", 12)));
    let e = release_it(&w, &r, &previews, &next, &[]).unwrap_err();
    assert!(
        e.contains("l3_solar 1.3 is to be released again, as issue 7 asks for l3_drag's objection"),
        "{e}"
    );
    // Never marked understood.
    let anyway = decided(&w, Some((Decide::ReleaseAnyway, 8, "Sid Side", 17)));
    let e = release_it(&w, &r, &previews, &anyway, &[s(DRAG)]).unwrap_err();
    assert!(
        e.contains(&format!(
            "{DRAG} is an objection, and an objection is never marked understood"
        )),
        "{e}"
    );
    // Released anyway: a red mark the programme manager's issue raises.
    let d = release_it(&w, &r, &previews, &anyway, &[]).expect("released anyway");
    assert!(
        table(&d, "red_marks.csv").contains(&format!("{DRAG},\"l3_drag objects to l3_solar 1.3: {BECAUSE}; released anyway, for the programme manager\",raised as issue 8")),
        "{}",
        table(&d, "red_marks.csv")
    );
    assert_eq!(
        table(&d, "objections.csv"),
        format!(
            "objection,group,version,answering,because,answer,decision\n\
             {DRAG},l3_solar,1.3,l3_drag,{BECAUSE},{},\"released anyway, put by Sid Side to the programme manager in issue 8\"\n",
            "a1".repeat(32)
        )
    );
    // Fine from both: released, nothing objected.
    let d = release_it(
        &w,
        &r,
        &[standing(RELEASED, "1.3", Answered::Fine, Answered::Fine)],
        &[],
        &[],
    )
    .expect("released");
    assert_eq!(
        table(&d, "objections.csv"),
        "objection,group,version,answering,because,answer,decision\n"
    );
}

#[test]
fn a_stale_preview_asks_nothing_of_the_release() {
    let w = world();
    let r = solar_13(&w);
    // On another design, and of a release today's design does not hold.
    for p in [
        standing(
            "2026.9.1",
            "1.3",
            Answered::Unanswered,
            Answered::Unanswered,
        ),
        standing(RELEASED, "1.2", Answered::Unanswered, Answered::Unanswered),
    ] {
        release_it(&w, &r, &[p], &[], &[]).expect("a stale preview asks nothing");
    }
}

/// Uma's objection to solar 1.3 for the drag, as an answer file, and the
/// objection it is, named by its digest.
fn uma_objects(w: &World, r: &File) -> (File, Objection) {
    let a = answer::answer(
        r,
        "l3_drag",
        &Said::Object(s(BECAUSE)),
        "Uma Orbit",
        &key(18),
        AT,
        APP,
        &w.sy,
        &w.pr,
    )
    .unwrap();
    let o = Objection {
        group: s("l3_solar"),
        version: s("1.3"),
        answering: s("l3_drag"),
        because: s(BECAUSE),
        answer: chain::digest(&a, FILE).unwrap(),
    };
    (a, o)
}

fn decide(w: &World, o: &Objection, d: Decide, by: &str, k: u8) -> Result<File, String> {
    design::decision_issue(o, d, 7, by, &key(k), AT, APP, &[&w.pr, &w.sy], &w.pr)
        .map_err(|e| e.message().to_string())
}

fn decisions(w: &World, o: &Objection, issues: &[File]) -> Option<Decision> {
    design::decisions(
        std::slice::from_ref(o),
        issues,
        None,
        &[&w.pr, &w.sy],
        &w.pr,
    )
    .remove(0)
    .decision()
    .cloned()
}

#[test]
fn the_system_engineer_decides_an_objection_by_an_issue() {
    let w = world();
    let r = solar_13(&w);
    let (_, o) = uma_objects(&w, &r);
    // Its next version: on solar, to its owner.
    let f = decide(&w, &o, Decide::NextVersion, "Sam Engel", 12).expect("Sam decides");
    assert_eq!(f.kind().unwrap(), Kind::Issue);
    assert_eq!(
        f.meta.get("signed_rows").map(String::as_str),
        Some("issue change"),
        "its rows are signed with it (F6-d)"
    );
    let i = &f.issues[0];
    assert_eq!(
        (
            i.group_id.as_str(),
            i.addressed_to.as_str(),
            i.place.as_str(),
            i.raised_by.as_str()
        ),
        ("l3_solar", "Ola Sun", DRAG, "Sam Engel")
    );
    assert_eq!(i.evidence, format!("answer {}", o.answer));
    assert!(
        i.what.contains(BECAUSE) && i.what.contains("ask for its next version"),
        "{}",
        i.what
    );
    assert_eq!(
        decisions(&w, &o, &[f]),
        Some(Decision::NextVersion {
            issue: 7,
            by: s("Sam Engel")
        })
    );
    // Release anyway: on the programme, to its programme manager.
    let f = decide(&w, &o, Decide::ReleaseAnyway, "Sid Side", 17).expect("Sam's deputy decides");
    let i = &f.issues[0];
    assert_eq!(
        (i.group_id.as_str(), i.addressed_to.as_str()),
        ("programme", "Pat Morgan")
    );
    assert_eq!(
        decisions(&w, &o, &[f]),
        Some(Decision::ReleaseAnyway {
            issue: 7,
            by: s("Sid Side")
        })
    );
    // Refused for anyone else.
    for (by, k, why) in [
        ("Ola Sun", 13, "Ola Sun may not decide an objection"),
        ("Claude", 12, "is an assistant's name"),
    ] {
        let e = decide(&w, &o, Decide::NextVersion, by, k).unwrap_err();
        assert!(e.contains(why), "{by}: {e}");
    }
}

#[test]
fn a_decision_is_read_only_from_what_the_system_engineer_signed() {
    let w = world();
    let r = solar_13(&w);
    let (_, o) = uma_objects(&w, &r);
    let good = decide(&w, &o, Decide::ReleaseAnyway, "Sam Engel", 12).unwrap();
    assert!(decisions(&w, &o, &[]).is_none(), "nothing decided");
    // Raised by Ola, signed by Ola: not a decision.
    let mut ola = good.clone();
    ola.issues[0].raised_by = s("Ola Sun");
    ola.signatures.clear();
    ola.signatures
        .push(chain::sign(&ola, FILE, "Ola Sun", &key(13), AT, "ok", "raised").unwrap());
    // Raised by Sid, signed by Sam: not who raised it.
    let mut sid = good.clone();
    sid.issues[0].raised_by = s("Sid Side");
    // Naming another answer.
    let mut other = good.clone();
    other.issues[0].evidence = format!("answer {}", "b2".repeat(32));
    other.signatures.clear();
    other
        .signatures
        .push(chain::sign(&other, FILE, "Sam Engel", &key(12), AT, "ok", "raised").unwrap());
    // Changed after Sam signed it: its row, which his signature does not
    // cover, turned from Ola's next version into release anyway; and its
    // signed text changed to agree.
    let mut changed = decide(&w, &o, Decide::NextVersion, "Sam Engel", 12).unwrap();
    changed.issues[0].group_id = s("programme");
    changed.issues[0].addressed_to = s("Pat Morgan");
    let mut both = changed.clone();
    for t in both.texts.iter_mut().filter(|t| t.kind == "decision") {
        t.body = t.body.replace("l3_solar\nOla Sun", "programme\nPat Morgan");
    }
    // To someone who is neither the owner nor the programme manager, its
    // text agreeing and signed again by Sam: on the programme, and on solar.
    let to_uma = |mut f: File| {
        let was = f.issues[0].addressed_to.clone();
        f.issues[0].addressed_to = s("Uma Orbit");
        for t in f.texts.iter_mut().filter(|t| t.kind == "decision") {
            t.body = t.body.replace(&was, "Uma Orbit");
        }
        f.signatures.clear();
        let row = chain::sign(&f, FILE, "Sam Engel", &key(12), AT, "ok", "raised").unwrap();
        f.signatures.push(row);
        f
    };
    // Its number, or its group, not what its signed meta says, the rest
    // agreeing and signed again by Sam.
    let again = |mut f: File| {
        f.signatures.clear();
        let row = chain::sign(&f, FILE, "Sam Engel", &key(12), AT, "ok", "raised").unwrap();
        f.signatures.push(row);
        f
    };
    let mut number = good.clone();
    number.issues[0].number = 9;
    let number = again(number);
    let mut moved = decide(&w, &o, Decide::NextVersion, "Sam Engel", 12).unwrap();
    moved.issues[0].group_id = s("programme");
    moved.issues[0].addressed_to = s("Pat Morgan");
    for t in moved.texts.iter_mut().filter(|t| t.kind == "decision") {
        t.body = t.body.replace("l3_solar\nOla Sun", "programme\nPat Morgan");
    }
    let moved = again(moved);
    // Its row alone moved to another objection, of the same release: the
    // signature still counts, and its signed text says the drag's.
    let o2 = Objection {
        answering: s("l3_x_envorbit"),
        answer: "c3".repeat(32),
        ..o.clone()
    };
    let mut shifted = good.clone();
    shifted.issues[0].place = o2.id();
    shifted.issues[0].evidence = format!("answer {}", o2.answer);
    assert!(
        decisions(&w, &o2, &[shifted]).is_none(),
        "moved to another objection"
    );
    // Its meta naming another writer than its raiser, signed again by Sam:
    // not an issue file as every reader holds one (`chain::check_issue`).
    let mut writer = good.clone();
    writer.meta.insert(s("writer"), s("Sid Side"));
    let writer = again(writer);
    let wrong = to_uma(good.clone());
    let wrong_owner = to_uma(decide(&w, &o, Decide::NextVersion, "Sam Engel", 12).unwrap());
    for (what, f) in [
        ("raised by Ola", ola),
        ("raised by Sid, signed by Sam", sid),
        ("naming another answer", other),
        ("its row changed after it was signed", changed),
        ("its row and its text changed after it was signed", both),
        ("its number not its meta's", number),
        ("its writer not its raiser", writer),
        ("its group not its meta's", moved),
        ("on the programme, to someone else", wrong),
        ("on solar, to someone else", wrong_owner),
    ] {
        assert!(decisions(&w, &o, &[f]).is_none(), "{what}");
    }
    // The systems' file is no issue file: its reallocation decides nothing
    // read as one.
    let mut sy = w.sy.file.clone();
    sy.changes
        .push(reallocated("Sam Engel", &design::reallocation_names(&o)));
    chain::sign_rows(&mut sy);
    let issues = [signed(sy, "Sam Engel", &key(12))];
    assert!(decisions(&w, &o, &issues).is_none(), "a reallocation");
}

/// A `reallocate` change row by `who`, its why saying `names`.
fn reallocated(who: &str, names: &str) -> Change {
    Change {
        at: s(AT),
        who: s(who),
        scope: s("sys_drag"),
        what: format!("reallocate: the drag's mass widened, deciding {names}"),
        before: None,
        after: None,
    }
}

/// A reallocation decides an objection (F6-e), read from the systems' file
/// as it opens through the chain, marked so its change rows are signed with
/// it: a `reallocate` row by Sam, or Sid, naming the objection and its
/// answer, by 12 characters of its digest or more. Nothing else decides: no
/// systems' file, one unmarked, one that does not open, a row added after it
/// was signed, a row by Ola, another edit's, one naming another answer, too
/// little of it, or the place alone. Released, it is no red mark, and
/// `objections.csv` keeps it.
#[test]
fn a_reallocation_in_the_systems_file_decides_an_objection() {
    let w = world();
    let o = objection();
    let names = design::reallocation_names(&o);
    assert_eq!(names, format!("{DRAG} answer {}", "a1".repeat(6)));
    let with = |rows: Vec<Change>, marked: bool, who: &str, k: u8| {
        let mut f = w.sy.file.clone();
        f.changes.extend(rows);
        if marked {
            chain::sign_rows(&mut f);
        }
        signed(f, who, &key(k))
    };
    let decided = |f: Option<&File>| {
        design::decisions(std::slice::from_ref(&o), &[], f, &[&w.pr, &w.sy], &w.pr)
    };
    let decision = |f: Option<&File>| decided(f).remove(0).decision().cloned();
    let sam = with(
        vec![reallocated("Sam Engel", &names)],
        true,
        "Sam Engel",
        12,
    );
    let by = |who: &str| {
        Some(Decision::Reallocated {
            by: s(who),
            at: s(AT),
        })
    };
    assert_eq!(decision(Some(&sam)), by("Sam Engel"));
    let whole = format!("{DRAG} answer {}.", o.answer);
    let sid = with(vec![reallocated("Sid Side", &whole)], true, "Sam Engel", 12);
    assert_eq!(
        decision(Some(&sid)),
        by("Sid Side"),
        "Sid, the whole digest named"
    );
    let mut later = with(vec![], true, "Sam Engel", 12);
    later.changes.push(reallocated("Sam Engel", &names));
    let mut mount = reallocated("Sam Engel", &names);
    mount.what = format!("mount: {}", mount.what);
    let other = format!("{DRAG} answer {}", "b2".repeat(6));
    let little = format!("{DRAG} answer {}", "a1".repeat(4));
    for (what, f) in [
        (
            "unmarked",
            with(
                vec![reallocated("Sam Engel", &names)],
                false,
                "Sam Engel",
                12,
            ),
        ),
        (
            "signed by Ola",
            with(vec![reallocated("Sam Engel", &names)], true, "Ola Sun", 13),
        ),
        ("a row added after it was signed", later),
        (
            "a row by Ola",
            with(vec![reallocated("Ola Sun", &names)], true, "Sam Engel", 12),
        ),
        ("another edit's", with(vec![mount], true, "Sam Engel", 12)),
        (
            "another answer",
            with(
                vec![reallocated("Sam Engel", &other)],
                true,
                "Sam Engel",
                12,
            ),
        ),
        (
            "too little of it",
            with(
                vec![reallocated("Sam Engel", &little)],
                true,
                "Sam Engel",
                12,
            ),
        ),
        (
            "the place alone",
            with(vec![reallocated("Sam Engel", DRAG)], true, "Sam Engel", 12),
        ),
    ] {
        assert_eq!(decision(Some(&f)), None, "{what}");
    }
    assert_eq!(decision(None), None, "no systems' file");
    // Released: no red mark of it, and its row in objections.csv.
    let r = solar_13(&w);
    let previews = [standing(
        RELEASED,
        "1.3",
        Answered::Object(s(BECAUSE)),
        Answered::Fine,
    )];
    let d = release_it(&w, &r, &previews, &decided(Some(&sam)), &[]).expect("reallocated");
    assert!(
        !table(&d, "red_marks.csv").contains(DRAG),
        "{}",
        table(&d, "red_marks.csv")
    );
    assert_eq!(
        table(&d, "objections.csv"),
        format!(
            "objection,group,version,answering,because,answer,decision\n\
             {DRAG},l3_solar,1.3,l3_drag,{BECAUSE},{},\"an allocation changed, by Sam Engel at {AT}, in the systems' file\"\n",
            "a1".repeat(32)
        )
    );
}

/// `chain::check_issue`, as every reader of an issue file holds it, on the
/// files the system engineer raises: one as Sam signed it checks; its row
/// changed after, it does not; one signed by Sid as Sam's does not; one from
/// before F6-d, its row unsigned, checks with its row changed, as every file
/// signed before is read.
#[test]
fn an_issue_file_checks_as_its_raiser_signed_it() {
    let w = world();
    let r = solar_13(&w);
    let (_, o) = uma_objects(&w, &r);
    let f = decide(&w, &o, Decide::NextVersion, "Sam Engel", 12).unwrap();
    chain::check_issue(&f, &[&w.pr]).expect("as Sam signed it");
    let mut changed = f.clone();
    changed.issues[0].what = s("Nothing to decide.");
    let e = chain::check_issue(&changed, &[&w.pr]).unwrap_err();
    assert!(
        e.message().contains("issue 7 on l3_solar, by Sam Engel,"),
        "{}",
        e.message()
    );
    let mut sid = f.clone();
    sid.signatures.clear();
    sid.signatures
        .push(chain::sign(&sid, FILE, "Sid Side", &key(17), AT, "ok", "raised").unwrap());
    assert!(
        chain::check_issue(&sid, &[&w.pr]).is_err(),
        "signed by Sid as Sam's"
    );
    let mut before = f.clone();
    before.meta.remove(chain::SIGNED_ROWS);
    before.signatures.clear();
    before
        .signatures
        .push(chain::sign(&before, FILE, "Sam Engel", &key(12), AT, "ok", "raised").unwrap());
    before.issues[0].what = s("Changed, and unsigned before F6-d.");
    chain::check_issue(&before, &[&w.pr]).expect("read as it was before F6-d");
}
