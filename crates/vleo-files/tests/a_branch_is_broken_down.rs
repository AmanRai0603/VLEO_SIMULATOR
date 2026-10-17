//! A branch's breakdown is grown by its owner, in its group file, and a node
//! is broken down only into children that reproduce its own cases
//! (docs/briefs/F4.md, F4-a; W13; docs/SYSTEM_MODEL.md, section 2).
//!
//! A small programme of invented people, as People's test has it: Ola owns
//! solar, Dee is her deputy, Nia a node engineer of solar, Sam the system
//! engineer. Solar's branch holds solar 1.1's `sw_activity_band` as an issued
//! node file, and two children of it placed as F5 will issue them: one
//! carrying the node's own transcribed method, and one whose elevated edge
//! is at 135 sfu, invented here to be wrong. The expected values are the
//! release's own, from its results; nothing here is computed by the code
//! under test and then expected of it.
//!
//! - broken down into the child that reproduces its cases, it is accepted,
//!   with every case agreeing beside its estimate, and a change row saying
//!   why; the node's own file is not touched;
//! - into the child whose edge is wrong, it is refused, naming the case at
//!   130 sfu; into an open child, refused by that child's name;
//! - only Ola or her deputy frames solar; Nia and Sam are refused, and an
//!   edit with no why;
//! - each edit's refusals: an id taken or not one, a cycle, a decided block
//!   filled, a port the schema does not allow, an issued node moved, a block
//!   that answers a node removed, the programme's branch;
//! - folded back, the estimate answers again, and the children stay.

use vleo_files::breakdown::{self, Answer, Edit, Part, State};
use vleo_files::chain::{self, Registry, FILE};
use vleo_files::keys::SigningKey;
use vleo_files::meta::Kind;
use vleo_files::model::*;

const APP: &str = "vleo 1.0.0 (test)";
const AT: &str = "2026-10-17T09:00:00Z";
const BAND: &str = "sw_activity_band";
const GOOD: &str = "sw_activity_count";
const LATE: &str = "sw_activity_count_late";
const WHY: &str = "The band is counted from its edges; the count is a node of its own.";

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

/// Solar's group file: its heading, and Nia.
fn solar() -> File {
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
    signed(f, "Ola Sun", &key(13))
}

/// `uid` from solar 1.1's release, in a node file of its own.
fn from_release(uid: &str) -> File {
    let how = vleo_files::upgrade::Upgrade { app: APP, at: AT };
    let rel = vleo_files::sqlite::open(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/l3_solar-1.1.vleo"),
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

fn out_port(f: &File) -> String {
    f.ports
        .iter()
        .find(|p| p.direction == "out")
        .unwrap()
        .name
        .clone()
}

/// A child of the band, issued: the band's own method under another id,
/// reading the band's input, with `edit` made to its method.
fn child(id: &str, edit: &dyn Fn(&str) -> String) -> File {
    let band = from_release(BAND);
    let mut c = band.clone();
    c.meta.insert(s("block_uid"), s(id));
    c.blocks[0].uid = s(id);
    c.blocks[0].id = s(id);
    c.blocks[0].parent_uid = s(BAND);
    for p in c.ports.iter_mut() {
        p.block_uid = s(id);
    }
    c.wires = c
        .ports
        .iter()
        .filter(|p| p.direction == "in")
        .map(|p| Wire {
            to_block: s(id),
            to_port: p.name.clone(),
            from_ref: format!("{BAND}.{}", p.name),
        })
        .collect();
    for t in c.texts.iter_mut() {
        t.scope = s(id);
        if t.kind == "pseudocode" {
            t.body = edit(&t.body);
        }
    }
    for t in c.tables.iter_mut() {
        t.scope = s(id);
    }
    c
}

fn nodes() -> Vec<File> {
    vec![
        from_release(BAND),
        child(GOOD, &|m| m.to_string()),
        child(LATE, &|m| {
            m.replace("if f107 >= 130 then", "if f107 >= 135 then")
        }),
    ]
}

fn break_down_into(child_id: &str) -> Edit {
    let band = from_release(BAND);
    let out = out_port(&band);
    Edit::BreakDown {
        node: s(BAND),
        answers: vec![Answer {
            output: out.clone(),
            child: s(child_id),
            port: out,
        }],
    }
}

fn frame(
    group: &File,
    e: &Edit,
    why: &str,
    who: &str,
    k: &SigningKey,
) -> Result<(File, Option<breakdown::Reproduced>), String> {
    let pr = programme();
    let sy = systems(&pr);
    breakdown::edit(group, &nodes(), &[], e, why, who, k, AT, &sy, &pr)
        .map_err(|e| e.message().to_string())
}

fn ola(group: &File, e: &Edit) -> Result<(File, Option<breakdown::Reproduced>), String> {
    frame(group, e, WHY, "Ola Sun", &key(13))
}

#[test]
fn a_node_is_broken_down_only_into_children_that_reproduce_its_cases() {
    let (after, verdict) =
        ola(&solar(), &break_down_into(GOOD)).expect("the good child reproduces the band");
    let v = verdict.expect("a breakdown says how its children did");
    assert_eq!(v.state, State::Agrees, "{v:?}");
    // Solar 1.1's ten cases for the band, nine answers and one refusal.
    assert_eq!(v.cases.len(), 10, "{v:?}");
    let edge = v
        .cases
        .iter()
        .find(|c| c.label == "130 sfu exactly — elevated begins.")
        .expect("the case at the edge");
    assert_eq!(edge.expected, Some(3.0));
    assert_eq!(edge.children, "3");
    assert!(edge.estimate.contains("agrees"), "{edge:?}");
    let refusal = v.cases.iter().find(|c| c.expected.is_none()).unwrap();
    assert!(
        refusal.agrees && refusal.children.starts_with("refused by sw_activity_count"),
        "{refusal:?}"
    );
    // Written in the group file: one wire answering the band's output.
    let out = out_port(&from_release(BAND));
    assert_eq!(
        after.wires,
        vec![Wire {
            to_block: s(BAND),
            to_port: out.clone(),
            from_ref: format!("{GOOD}.{out}"),
        }]
    );
    let c = after.changes.last().unwrap();
    assert_eq!((c.who.as_str(), c.scope.as_str()), ("Ola Sun", BAND));
    assert_eq!(c.what, format!("break down: {WHY}"));
    assert!(
        c.after
            .as_deref()
            .unwrap_or("")
            .contains(&format!("from \"{GOOD}.{out}\"")),
        "{c:?}"
    );
    // Signed afresh by her, and it opens as it will be read.
    let pr = programme();
    let sy = systems(&pr);
    chain::open_group_in(&after, &sy, &pr).expect("the framed file opens through the chain");
    assert_eq!(
        after
            .signatures
            .iter()
            .filter(|r| r.scope == FILE)
            .map(|r| r.signer.as_str())
            .collect::<Vec<_>>(),
        vec!["Ola Sun"]
    );
    // The branch reads the band as broken down; the band's file is as it was.
    let ns = nodes();
    let branch = breakdown::Branch::new(&after, &ns);
    assert_eq!(branch.answers(BAND).len(), 1);
    assert_eq!(ns[0], from_release(BAND));

    // Into the child whose edge is at 135, refused at the case it breaks.
    let e = ola(&solar(), &break_down_into(LATE)).unwrap_err();
    assert!(
        e.contains("do not reproduce its cases")
            && e.contains("«130 sfu exactly — elevated begins.»")
            && e.contains("answered by sw_activity_count_late"),
        "{e}"
    );
    assert!(
        !e.contains("«129.9 sfu"),
        "only the cases that differ are named: {e}"
    );

    // Into an open child: refused by its name.
    let (with_open, _) = ola(
        &solar(),
        &Edit::Add {
            parent: s(BAND),
            id: s("sw_activity_draft"),
            question: s("Which band, counted another way?"),
        },
    )
    .unwrap();
    let port = Port {
        direction: s("out"),
        name: s(&out),
        port_type: s("number"),
        unit: s("1"),
        state: s("achieved"),
        ..Port::default()
    };
    let (with_open, _) = ola(
        &with_open,
        &Edit::Fill {
            block: s("sw_activity_draft"),
            part: Part::Port(Box::new(port)),
        },
    )
    .unwrap();
    let e = ola(&with_open, &break_down_into("sw_activity_draft")).unwrap_err();
    assert!(
        e.contains("sw_activity_draft cannot run: it is open"),
        "{e}"
    );

    // Folded back: the estimate answers again, and the children stay.
    let (back, _) = ola(&after, &Edit::FoldBack { node: s(BAND) }).unwrap();
    assert!(back.wires.is_empty());
    assert_eq!(breakdown::Branch::new(&back, &ns).children(BAND).len(), 2);
    assert_eq!(
        back.changes.last().unwrap().what,
        format!("fold back: {WHY}")
    );
    let e = ola(&back, &Edit::FoldBack { node: s(BAND) }).unwrap_err();
    assert!(e.contains("is not broken down"), "{e}");
}

#[test]
fn only_the_branch_s_owner_or_their_deputy_frames_it_and_each_edit_says_why() {
    let add = Edit::Add {
        parent: s("l3_solar"),
        id: s("sw_storm_onset"),
        question: s("When does a storm begin?"),
    };
    for (who, k, says) in [
        (
            "Nia Node",
            key(16),
            "Nia Node is not registered in the programme's file",
        ),
        (
            "Sam Engel",
            key(12),
            "its owner Ola Sun, or their deputy, does",
        ),
        (
            "Ola Sun",
            key(99),
            "is not one the programme's file registers",
        ),
    ] {
        let e = frame(&solar(), &add, WHY, who, &k).unwrap_err();
        assert!(e.contains(says), "{who}: {e}");
    }
    let (f, _) = frame(&solar(), &add, WHY, "Dee Puty", &key(14)).expect("her deputy frames it");
    assert_eq!(f.changes.last().unwrap().who, "Dee Puty");
    for why in ["", "   "] {
        let e = frame(&solar(), &add, why, "Ola Sun", &key(13)).unwrap_err();
        assert!(e.contains("say why"), "{e}");
    }
    // The programme's branch is not framed here.
    let pr = programme();
    let sy = systems(&pr);
    let e = breakdown::edit(
        &pr.file,
        &[],
        &[],
        &add,
        WHY,
        "Pat Morgan",
        &key(11),
        AT,
        &sy,
        &pr,
    )
    .unwrap_err();
    assert!(
        e.message().contains("Programme workspace (F7)"),
        "{}",
        e.message()
    );
}

#[test]
fn each_edit_is_refused_by_name_with_nothing_written() {
    let add = |parent: &str, id: &str| Edit::Add {
        parent: s(parent),
        id: s(id),
        question: s(""),
    };
    let (f, _) = ola(&solar(), &add("l3_solar", "sw_storm_onset")).unwrap();
    // An open block: what is still open in it.
    let ns = nodes();
    let open = breakdown::Branch::new(&f, &ns).open_parts("sw_storm_onset");
    assert_eq!(open, vec!["its question", "its behaviour", "an output"]);
    for (id, says) in [
        ("sw_storm_onset", "is in l3_solar's branch already"),
        (BAND, "is in l3_solar's branch already"),
        ("Not an id", "is not an id"),
    ] {
        let e = ola(&f, &add("l3_solar", id)).unwrap_err();
        assert!(e.contains(says), "{id}: {e}");
    }
    let e = ola(&f, &add("no_such_block", "sw_x")).unwrap_err();
    assert!(e.contains("has no block \"no_such_block\""), "{e}");

    // Filled a part at a time; held to the schema and the method language.
    let (f, _) = ola(
        &f,
        &Edit::Fill {
            block: s("sw_storm_onset"),
            part: Part::Question(s("When does a storm begin?")),
        },
    )
    .unwrap();
    let port = |t: &str, unit: &str, lo: &str, hi: &str| Port {
        direction: s("out"),
        name: s("sw_storm_onset"),
        port_type: s(t),
        unit: s(unit),
        lower: s(lo),
        upper: s(hi),
        state: s("open"),
        ..Port::default()
    };
    for (p, says) in [
        (
            port("banana", "1", "", ""),
            "port_type \"banana\", which the schema does not allow",
        ),
        (
            port("number", "furlong", "", ""),
            "which the method language does not read",
        ),
        (
            port("number", "1", "5", "1"),
            "its lower end is above its upper",
        ),
    ] {
        let e = ola(
            &f,
            &Edit::Fill {
                block: s("sw_storm_onset"),
                part: Part::Port(Box::new(p)),
            },
        )
        .unwrap_err();
        assert!(e.contains(says), "{e}");
    }
    let (f, _) = ola(
        &f,
        &Edit::Fill {
            block: s("sw_storm_onset"),
            part: Part::Port(Box::new(port("number", "d", "0", "30"))),
        },
    )
    .unwrap();
    assert_eq!(
        breakdown::Branch::new(&f, &ns).open_parts("sw_storm_onset"),
        vec!["its behaviour", "the state of sw_storm_onset"]
    );
    // A decided block is not filled here.
    let e = ola(
        &f,
        &Edit::Fill {
            block: s("l3_solar"),
            part: Part::Question(s("Another question.")),
        },
    )
    .unwrap_err();
    assert!(e.contains("is decided, not open"), "{e}");

    // Moved, never under itself.
    let (f, _) = ola(&f, &add("sw_storm_onset", "sw_storm_peak")).unwrap();
    let e = ola(
        &f,
        &Edit::Move {
            block: s("sw_storm_onset"),
            parent: s("sw_storm_peak"),
            ord: 0,
        },
    )
    .unwrap_err();
    assert!(e.contains("cannot hang under itself"), "{e}");
    // Removed only once nothing hangs from it.
    let e = ola(
        &f,
        &Edit::Remove {
            block: s("sw_storm_onset"),
        },
    )
    .unwrap_err();
    assert!(e.contains("holds sw_storm_peak"), "{e}");
    let (f, _) = ola(
        &f,
        &Edit::Remove {
            block: s("sw_storm_peak"),
        },
    )
    .unwrap();
    assert!(
        f.blocks
            .iter()
            .any(|b| b.uid == "sw_storm_peak" && b.archived == 1),
        "archived, not deleted"
    );
    let e = ola(&f, &add("sw_storm_onset", "sw_storm_peak")).unwrap_err();
    assert!(
        e.contains("live or removed"),
        "an id is never used twice: {e}"
    );
    // Renamed, then split.
    let (f, _) = ola(
        &f,
        &Edit::Rename {
            block: s("sw_storm_onset"),
            id: s("sw_storm_start"),
        },
    )
    .unwrap();
    assert!(f
        .blocks
        .iter()
        .any(|b| b.uid == "sw_storm_onset" && b.id == "sw_storm_start"));
    let e = ola(
        &f,
        &Edit::Split {
            block: s("sw_storm_onset"),
            id: s("sw_storm_end"),
            ports: vec![s("no_such_port")],
        },
    )
    .unwrap_err();
    assert!(e.contains("has no port no_such_port"), "{e}");

    // An issued node is moved by issuing it again, not here.
    let e = ola(
        &f,
        &Edit::Move {
            block: s(BAND),
            parent: s("sw_storm_onset"),
            ord: 0,
        },
    )
    .unwrap_err();
    assert!(e.contains("is issued, a node file of its own"), "{e}");
    // A block that answers a broken-down node stays while it answers.
    let (f, _) = ola(&f, &break_down_into(GOOD)).unwrap();
    let e = ola(
        &f,
        &Edit::BreakDown {
            node: s(BAND),
            answers: vec![],
        },
    )
    .unwrap_err();
    assert!(e.contains("is broken down already"), "{e}");
}

#[test]
fn an_edit_in_the_copy_writes_the_group_file_alone_and_stops_a_second_writer() {
    let copy = std::env::temp_dir().join(format!("vleo-branch-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&copy);
    let dir = copy.join("groups/l3_solar");
    std::fs::create_dir_all(dir.join("nodes")).unwrap();
    let group = dir.join("l3_solar.vgroup");
    vleo_files::sqlite::write(&solar(), &group).unwrap();
    for n in nodes() {
        let id = n.blocks[0].id.clone();
        vleo_files::sqlite::write(&n, &dir.join(format!("nodes/{id}.vnode"))).unwrap();
    }
    let node_bytes = |id: &str| std::fs::read(dir.join(format!("nodes/{id}.vnode"))).unwrap();
    let before: Vec<Vec<u8>> = [BAND, GOOD, LATE].iter().map(|id| node_bytes(id)).collect();
    let digest = |p: &std::path::Path| vleo_files::copied::digest(&std::fs::read(p).unwrap());
    let opened = digest(&group);
    let pr = programme();
    let sy = systems(&pr);
    let made = breakdown::edit_in_place(
        &copy,
        "l3_solar",
        &opened,
        &break_down_into(GOOD),
        WHY,
        "Ola Sun",
        &key(13),
        AT,
        APP,
        &sy,
        &pr,
    )
    .unwrap();
    let breakdown::Framed::Written {
        digest: now,
        reproduced,
    } = made
    else {
        panic!("written: {made:?}")
    };
    assert_eq!(now, digest(&group));
    assert_eq!(reproduced.unwrap().state, State::Agrees);
    let read = vleo_files::sqlite::read(&group).unwrap();
    assert_eq!(read.wires.len(), 1);
    assert_eq!(
        read.meta.get("written_by_app").map(String::as_str),
        Some(APP)
    );
    // From what she opened before: a second writer, and nothing written.
    let stale = breakdown::edit_in_place(
        &copy,
        "l3_solar",
        &opened,
        &Edit::FoldBack { node: s(BAND) },
        WHY,
        "Ola Sun",
        &key(13),
        AT,
        APP,
        &sy,
        &pr,
    )
    .unwrap();
    assert!(
        matches!(stale, breakdown::Framed::SecondWriter { ref digest } if *digest == now),
        "{stale:?}"
    );
    assert_eq!(digest(&group), now, "nothing is written over");
    // A refused edit writes nothing either.
    let e = breakdown::edit_in_place(
        &copy,
        "l3_solar",
        &now,
        &break_down_into(LATE),
        WHY,
        "Ola Sun",
        &key(13),
        AT,
        APP,
        &sy,
        &pr,
    )
    .unwrap_err();
    assert!(
        e.message().contains("is broken down already"),
        "{}",
        e.message()
    );
    assert_eq!(digest(&group), now);
    // No node file is touched by any of it.
    let after: Vec<Vec<u8>> = [BAND, GOOD, LATE].iter().map(|id| node_bytes(id)).collect();
    assert_eq!(after, before);
    let _ = std::fs::remove_dir_all(&copy);
}
