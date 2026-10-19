//! A branch's breakdown is grown by its owner, in its group file, and a node
//! is broken down only into children that reproduce its own cases
//! (docs/briefs/F4.md, F4-a; W13; docs/SYSTEM_MODEL.md, section 2).
//!
//! A small programme of invented people, as People's test has it: Ola owns
//! solar, Dee is Ola's deputy, Nia a node engineer of solar, Sam the system
//! engineer. Solar's branch holds solar 1.1's `sw_activity_band` as an issued
//! node file, and two children of it placed as F5 will issue them: one
//! carrying the node's own transcribed method, and one whose elevated edge
//! is at 135 sfu, invented here to be wrong, and saying so in its method and
//! its declaration. The expected values are the
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
//! - each edit's refusals: an id taken or not one, here or in another group's
//!   file, a cycle, a decided block filled, a port the schema does not allow,
//!   an issued node moved, a node broken down twice, a block another group
//!   reads renamed, an output split away from its reader, the programme's
//!   branch; and a move, a split and a port filled again, made;
//! - folded back, the estimate answers again, and the children stay;
//! - the node's cases are evidence only from a file its checks pass and its
//!   node engineer signed as it is: unsigned, or with a case whose expected
//!   value is the code's own, nothing is run; a group file changed since it
//!   was signed is refused before anything is framed on it;
//! - a child is held to its declaration before it runs; a child answering
//!   with a member it publishes is refused; a child reading from where its
//!   node's input comes reads that input;
//! - a breakdown makes no loop in the design's N2, and a child's file reads
//!   its node's input as it will in a release.

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
    f.assignments.push(Assignment {
        block_uid: s(BAND),
        person: s("Nia Node"),
        contract_version: 1,
    });
    signed(f, "Ola Sun", &key(13))
}

/// Solar's registry, as the chain opens its group file.
fn solar_registry() -> Registry {
    let pr = programme();
    let sy = systems(&pr);
    chain::open_group_in(&solar(), &sy, &pr).unwrap()
}

/// The band's node file, signed by Nia, its node engineer, as it is.
fn band() -> File {
    let (f, _) = chain::sign_node(
        &from_release(BAND),
        BAND,
        "Nia Node",
        &key(16),
        AT,
        &solar_registry(),
    )
    .expect("Nia signs the band");
    f
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

/// The child at 135 sfu says what it is: invented for this check by the
/// test's own invented person, never transcribed by anyone from anything
/// (F4's question 5).
fn invented(mut c: File) -> File {
    for t in c.texts.iter_mut().filter(|t| t.kind == "pseudocode") {
        let rest: Vec<&str> = t.body.lines().skip(1).collect();
        t.body = format!(
            "# INVENTED for the breakdown's check, never part of the design: sw_activity_band's method, its elevated edge moved from 130 to 135 sfu to be wrong.\n{}\n",
            rest.join("\n")
        );
    }
    for t in c.tables.iter_mut().filter(|t| t.path == "declaration.csv") {
        t.csv = "author,ai,date,source,checked_by\nNia Node,transcribed,2026-10-17,\"sw_activity_band's method, its elevated edge moved to 135 sfu: invented to be wrong, for the breakdown's check\",Nia Node\n".into();
    }
    c
}

fn nodes() -> Vec<File> {
    vec![
        band(),
        child(GOOD, &|m| m.to_string()),
        invented(child(LATE, &|m| {
            m.replace("if f107 >= 130 then", "if f107 >= 135 then")
        })),
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

/// An edit by Ola, with the branch's node files `ns` and the design's other
/// files `elsewhere`.
fn ola_with(
    group: &File,
    ns: &[File],
    elsewhere: &[File],
    e: &Edit,
) -> Result<(File, Option<breakdown::Reproduced>), String> {
    let pr = programme();
    let sy = systems(&pr);
    breakdown::edit(
        group,
        ns,
        elsewhere,
        e,
        WHY,
        "Ola Sun",
        &key(13),
        AT,
        &sy,
        &pr,
    )
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
    // What it was judged on: the verdict, and the child that ran by its rows.
    let judged = c.after.as_deref().unwrap_or("");
    assert!(
        judged.contains("reproduces: its children reproduce all 10 of its cases"),
        "{judged}"
    );
    assert_eq!(v.children.len(), 1, "{v:?}");
    assert_eq!(v.children[0].0, GOOD);
    assert!(
        judged.contains(&format!("child \"{GOOD}\" rows {}", v.children[0].1))
            && v.children[0].1.len() == 64,
        "{judged}"
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
    assert_eq!(ns[0], band());

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
        (
            "sw_storm_onset",
            "is in l3_solar's branch or another file of the design already",
        ),
        (
            BAND,
            "is in l3_solar's branch or another file of the design already",
        ),
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
    // A node is broken down once.
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
    // A file of the copy that cannot be read is never skipped: whether the
    // edit leaves a reader without what it reads cannot be said.
    let other = copy.join("groups/l3_power/nodes");
    std::fs::create_dir_all(&other).unwrap();
    std::fs::write(other.join("pw_array.vnode"), b"not a design file").unwrap();
    let e = breakdown::edit_in_place(
        &copy,
        "l3_solar",
        &now,
        &Edit::FoldBack { node: s(BAND) },
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
        e.message()
            .contains("groups/l3_power/nodes/pw_array.vnode cannot be read"),
        "{}",
        e.message()
    );
    assert_eq!(digest(&group), now);
    let _ = std::fs::remove_dir_all(&copy);
}

#[test]
fn the_cases_are_evidence_only_from_a_file_its_checks_pass_and_its_engineer_signed() {
    // Unsigned: its cases are not taken.
    let mut ns = nodes();
    ns[0] = from_release(BAND);
    let e = ola_with(&solar(), &ns, &[], &break_down_into(GOOD)).unwrap_err();
    assert!(
        e.contains("are not taken until they sign it as it is"),
        "{e}"
    );
    // Signed, then a case changed: the signature no longer holds.
    let mut ns = nodes();
    for t in ns[0]
        .tables
        .iter_mut()
        .filter(|t| t.path == "results/isolation.csv")
    {
        t.csv = t.csv.replacen(",paper,", ",code,", 1);
    }
    let e = ola_with(&solar(), &ns, &[], &break_down_into(GOOD)).unwrap_err();
    assert!(
        e.contains("are not taken until they sign it as it is"),
        "{e}"
    );
    // A case whose expected value is the code's own, signed as it is: its
    // own checks refuse it, and nothing is run.
    let mut agent = from_release(BAND);
    for t in agent
        .tables
        .iter_mut()
        .filter(|t| t.path == "results/isolation.csv")
    {
        t.csv = t.csv.replacen(",paper,", ",agent-generated,", 1);
    }
    let row = chain::sign(&agent, BAND, "Nia Node", &key(16), AT, "ok", "").unwrap();
    agent.signatures.push(row);
    chain::check_node(&agent, BAND, &solar_registry()).expect("signed as it is");
    let mut ns = nodes();
    ns[0] = agent;
    let e = ola_with(&solar(), &ns, &[], &break_down_into(GOOD)).unwrap_err();
    assert!(
        e.contains("own checks refuse it, so its cases are not evidence")
            && e.contains("never from the code under test"),
        "{e}"
    );
    // Through the server's reading too: no registry, nothing is run.
    let (after, _) = ola(&solar(), &break_down_into(GOOD)).unwrap();
    let ns = nodes();
    let r = breakdown::reproduces(&breakdown::Branch::new(&after, &ns), BAND, None);
    assert!(
        matches!(&r.state, State::NotRun(why) if why.contains("does not open through the chain")),
        "{r:?}"
    );
}

#[test]
fn a_group_file_changed_since_it_was_signed_is_refused_before_anything_is_framed() {
    // A node engineer's key written into the copy's group file by hand: the
    // owner's next edit would sign it in, so the file is refused first.
    let mut forged = solar();
    forged.assignments.push(Assignment {
        block_uid: s(GOOD),
        person: s("Nia Node"),
        contract_version: 1,
    });
    let e = ola(
        &forged,
        &Edit::Add {
            parent: s("l3_solar"),
            id: s("sw_storm_onset"),
            question: s(""),
        },
    )
    .unwrap_err();
    assert!(
        e.contains("does not open through the chain as it is, so nothing is framed on it"),
        "{e}"
    );
}

#[test]
fn a_child_is_held_to_its_declaration_and_answers_with_its_answer() {
    // A child whose declaration says an assistant supplied its relation is
    // not run (rule 6).
    let mut ns = nodes();
    for t in ns[1]
        .tables
        .iter_mut()
        .filter(|t| t.path == "declaration.csv")
    {
        t.csv = t.csv.replace(",transcribed,", ",relation,");
    }
    let e = ola_with(&solar(), &ns, &[], &break_down_into(GOOD)).unwrap_err();
    assert!(
        e.contains(&format!(
            "{GOOD} is not run: its declaration says an assistant supplied the relation"
        )),
        "{e}"
    );
    // A child's member, not its answer, answering the node: refused.
    let mut ns = nodes();
    let mut member = ns[1]
        .ports
        .iter()
        .find(|p| p.direction == "out")
        .unwrap()
        .clone();
    member.name = s("band_alone");
    ns[1].ports.push(member);
    let out = out_port(&from_release(BAND));
    let e = ola_with(
        &solar(),
        &ns,
        &[],
        &Edit::BreakDown {
            node: s(BAND),
            answers: vec![Answer {
                output: out.clone(),
                child: s(GOOD),
                port: s("band_alone"),
            }],
        },
    )
    .unwrap_err();
    assert!(
        e.contains(&format!(
            "{GOOD}.band_alone is a member {GOOD} publishes, not its answer"
        )),
        "{e}"
    );
    // A child answering in another unit: its method is checked against its
    // own ports before it runs, and refused there.
    let mut ns = nodes();
    for p in ns[1].ports.iter_mut().filter(|p| p.direction == "out") {
        p.unit = s("d");
    }
    let e = ola_with(&solar(), &ns, &[], &break_down_into(GOOD)).unwrap_err();
    assert!(
        e.contains(&format!(
            "{GOOD} is not run: its method, line 21: the answer must be s and this is dimensionless"
        )),
        "{e}"
    );
    // A child wired from where the band's own input comes reads that input.
    let mut ns = nodes();
    let source = ns[0]
        .wires
        .iter()
        .find(|w| w.to_block == BAND)
        .unwrap()
        .from_ref
        .clone();
    for w in ns[1].wires.iter_mut() {
        w.from_ref = source.clone();
    }
    let (_, v) = ola_with(&solar(), &ns, &[], &break_down_into(GOOD)).unwrap();
    assert_eq!(v.unwrap().state, State::Agrees);
}

#[test]
fn a_breakdown_makes_no_loop_and_a_child_reads_its_node_s_input_as_a_release_will() {
    let (after, _) = ola(&solar(), &break_down_into(GOOD)).unwrap();
    // The band's input comes from the observed level, a node of its own.
    let mut files = vec![(s("l3_solar.vgroup"), after.clone())];
    let mut observed = from_release("sw_f107_observed");
    observed.meta.insert(s("block_uid"), s("sw_f107_observed"));
    files.push((s("sw_f107_observed.vnode"), observed));
    for n in nodes() {
        files.push((format!("{}.vnode", n.blocks[0].uid), n));
    }
    let d = vleo_files::n2::Design::of(&files).expect("the design reads");
    assert!(d.loops().is_empty(), "{:?}", d.loops());
    // Node and child in one file, as F5 will issue them in a release: the
    // child's input reads its node's.
    let mut both = band();
    let good = child(GOOD, &|m| m.to_string());
    both.blocks.extend(good.blocks.clone());
    both.ports.extend(good.ports.clone());
    both.wires.extend(good.wires.clone());
    let found = vleo_files::checks::check_node(&both);
    assert!(
        !found
            .errors()
            .any(|e| e.what.contains("connects to nothing")),
        "{:?}",
        found.errors().collect::<Vec<_>>()
    );
}

#[test]
fn each_edit_keeps_the_design_s_readers_and_ids() {
    let add = |parent: &str, id: &str| Edit::Add {
        parent: s(parent),
        id: s(id),
        question: s(""),
    };
    // Another group's file holds a block, and reads one of solar's.
    let mut thermal = group_file("l3_thermal");
    thermal.blocks.push(Block {
        uid: s("th_panel"),
        id: s("th_panel"),
        behaviour: s("open"),
        ..Block::default()
    });
    thermal.wires.push(Wire {
        to_block: s("th_panel"),
        to_port: s("load"),
        from_ref: s("l3_solar.sw_storm_onset.onset"),
    });
    let elsewhere = vec![thermal];
    let ns = nodes();
    let e = ola_with(&solar(), &ns, &elsewhere, &add("l3_solar", "th_panel")).unwrap_err();
    assert!(e.contains("another file of the design already"), "{e}");
    let (f, _) = ola_with(
        &solar(),
        &ns,
        &elsewhere,
        &add("l3_solar", "sw_storm_onset"),
    )
    .unwrap();
    let e = ola_with(
        &f,
        &ns,
        &elsewhere,
        &Edit::Rename {
            block: s("sw_storm_onset"),
            id: s("sw_storm_start"),
        },
    )
    .unwrap_err();
    assert!(e.contains("is read by th_panel's input load"), "{e}");

    // The broken-down band reads the observed level by its input: framed
    // here, the level is not renamed under it.
    let (f, _) = ola_with(&solar(), &ns, &[], &add("l3_solar", "sw_f107_observed")).unwrap();
    let (f, _) = ola_with(&f, &ns, &[], &break_down_into(GOOD)).unwrap();
    let e = ola_with(
        &f,
        &ns,
        &[],
        &Edit::Rename {
            block: s("sw_f107_observed"),
            id: s("sw_f107_seen"),
        },
    )
    .unwrap_err();
    assert!(
        e.contains("is read by sw_activity_band's input f107"),
        "{e}"
    );

    // Filled, a port again in its place; moved; split, an output read by
    // another block staying where it is read from.
    let port = |dir: &str, name: &str, unit: &str| Port {
        direction: s(dir),
        name: s(name),
        port_type: s("number"),
        unit: s(unit),
        state: s("open"),
        ..Port::default()
    };
    let fill = |f: &File, block: &str, p: Port| {
        ola_with(
            f,
            &ns,
            &[],
            &Edit::Fill {
                block: s(block),
                part: Part::Port(Box::new(p)),
            },
        )
        .unwrap()
        .0
    };
    let f = fill(&f, "sw_f107_observed", port("in", "seen", "1"));
    let f = fill(&f, "sw_f107_observed", port("out", "level", "1"));
    let f = fill(&f, "sw_f107_observed", port("out", "level", "d"));
    let ps: Vec<(&str, &str, i64)> = f
        .ports
        .iter()
        .filter(|p| p.block_uid == "sw_f107_observed")
        .map(|p| (p.name.as_str(), p.unit.as_str(), p.ord))
        .collect();
    assert_eq!(ps, vec![("seen", "1", 0), ("level", "d", 1)]);
    let (f, _) = ola_with(&f, &ns, &[], &add("l3_solar", "sw_storm_onset")).unwrap();
    let (f, _) = ola_with(
        &f,
        &ns,
        &[],
        &Edit::Move {
            block: s("sw_storm_onset"),
            parent: s("sw_f107_observed"),
            ord: 0,
        },
    )
    .unwrap();
    assert!(f
        .blocks
        .iter()
        .any(|b| b.uid == "sw_storm_onset" && b.parent_uid == "sw_f107_observed"));
    // A block after it, so a split cannot take its place in the order.
    let (f, _) = ola_with(&f, &ns, &[], &add("l3_solar", "sw_f107_trend")).unwrap();
    // Read by another group: its output stays where it is read from.
    let mut power = group_file("l3_power");
    power.wires.push(Wire {
        to_block: s("pw_array"),
        to_port: s("level"),
        from_ref: s("l3_solar.sw_f107_observed.level"),
    });
    let e = ola_with(
        &f,
        &ns,
        &[power],
        &Edit::Split {
            block: s("sw_f107_observed"),
            id: s("sw_f107_level"),
            ports: vec![s("level")],
        },
    )
    .unwrap_err();
    assert!(
        e.contains("sw_f107_observed.level is read by pw_array's input level"),
        "{e}"
    );
    let (f, _) = ola_with(
        &f,
        &ns,
        &[],
        &Edit::Split {
            block: s("sw_f107_observed"),
            id: s("sw_f107_level"),
            ports: vec![s("level")],
        },
    )
    .unwrap();
    let sib = f.blocks.iter().find(|b| b.uid == "sw_f107_level").unwrap();
    let orders: Vec<i64> = f
        .blocks
        .iter()
        .filter(|b| b.parent_uid == sib.parent_uid && b.uid != sib.uid)
        .map(|b| b.ord)
        .collect();
    assert!(!orders.contains(&sib.ord), "{orders:?} {}", sib.ord);
    assert!(f
        .ports
        .iter()
        .any(|p| p.block_uid == "sw_f107_level" && p.name == "level"));
    let c = f.changes.last().unwrap();
    assert!(
        c.after.as_deref().unwrap_or("").contains("sw_f107_level"),
        "the change says what it made: {c:?}"
    );
}

// ── Assigned and wired by the branch's owner (docs/briefs/F5.md, F5-c) ──

/// Solar with an open block `sw_storm_onset` under its heading, an input
/// `f107` and an output of its own.
fn with_onset() -> File {
    let (f, _) = ola(
        &solar(),
        &Edit::Add {
            parent: s("l3_solar"),
            id: s("sw_storm_onset"),
            question: s("When does a storm begin?"),
        },
    )
    .unwrap();
    let mut f = f;
    for (dir, name) in [("in", "f107"), ("out", "sw_storm_onset")] {
        let p = Port {
            direction: s(dir),
            name: s(name),
            port_type: s("number"),
            unit: s("1"),
            state: s("open"),
            ..Port::default()
        };
        f = ola(
            &f,
            &Edit::Fill {
                block: s("sw_storm_onset"),
                part: Part::Port(Box::new(p)),
            },
        )
        .unwrap()
        .0;
    }
    f
}

#[test]
fn a_block_is_assigned_by_its_owner_to_a_node_engineer_its_group_registers() {
    let f = with_onset();
    let (a, _) = ola(
        &f,
        &Edit::Assign {
            block: s("sw_storm_onset"),
            person: s("Nia Node"),
        },
    )
    .unwrap();
    let row = a
        .assignments
        .iter()
        .find(|x| x.block_uid == "sw_storm_onset")
        .unwrap();
    assert_eq!((row.person.as_str(), row.contract_version), ("Nia Node", 1));
    let c = a.changes.last().unwrap();
    assert_eq!(c.what, format!("assign: {WHY}"));
    assert!(c.after.as_deref().unwrap().contains("Nia Node"), "{c:?}");
    // An issued node assigned again keeps its contract: the work moves, the
    // contract does not.
    let mut moved = solar_registry().file;
    for x in moved.assignments.iter_mut() {
        x.contract_version = 3;
    }
    let moved = signed(moved, "Ola Sun", &key(13));
    let mut nia_too = moved.clone();
    nia_too.people.push(Person {
        name: s("Ivo Node"),
        role: s("node engineer"),
        deputy_for: s(""),
    });
    nia_too.keys.push(PersonKey {
        person: s("Ivo Node"),
        public_key: key(17).public().to_hex(),
        registered_at: s("2026-01-01"),
        registered_by: s("Ola Sun"),
        revoked_from: s(""),
        proof: vleo_files::people::proof(&key(17), "Ivo Node"),
    });
    nia_too.signatures.retain(|x| x.scope != FILE);
    let nia_too = signed(nia_too, "Ola Sun", &key(13));
    let (b, _) = ola(
        &nia_too,
        &Edit::Assign {
            block: s(BAND),
            person: s("Ivo Node"),
        },
    )
    .unwrap();
    let rows: Vec<_> = b
        .assignments
        .iter()
        .filter(|x| x.block_uid == BAND)
        .collect();
    assert_eq!(rows.len(), 1, "one assignment a block");
    assert_eq!(
        (rows[0].person.as_str(), rows[0].contract_version),
        ("Ivo Node", 3)
    );

    for (block, person, says) in [
        (
            "sw_storm_onset",
            "Pat Morgan",
            "registers no node engineer Pat Morgan",
        ),
        (
            "sw_storm_onset",
            "Ola Sun",
            "registers no node engineer Ola Sun",
        ),
        ("sw_nowhere", "Nia Node", "has no block \"sw_nowhere\""),
        ("l3_solar", "Nia Node", "is a heading"),
    ] {
        let e = ola(
            &f,
            &Edit::Assign {
                block: s(block),
                person: s(person),
            },
        )
        .unwrap_err();
        assert!(e.contains(says), "expected «{says}» in: {e}");
    }
    let e = ola(
        &a,
        &Edit::Assign {
            block: s("sw_storm_onset"),
            person: s("Nia Node"),
        },
    )
    .unwrap_err();
    assert!(e.contains("is assigned to Nia Node already"), "{e}");
    // Only the owner assigns: Sam, whom the programme registers, is not
    // solar's owner.
    let e = frame(
        &f,
        &Edit::Assign {
            block: s("sw_storm_onset"),
            person: s("Nia Node"),
        },
        WHY,
        "Sam Engel",
        &key(12),
    )
    .unwrap_err();
    assert!(
        e.contains("its owner Ola Sun, or their deputy, does"),
        "{e}"
    );
}

/// `f` with `name` registered as a node engineer, deputy for `deputy_for`
/// when it is not empty, and signed again by Ola.
fn with_node_engineer(f: &File, name: &str, deputy_for: &str, k: &SigningKey) -> File {
    let mut f = f.clone();
    f.people.push(Person {
        name: s(name),
        role: s("node engineer"),
        deputy_for: s(deputy_for),
    });
    f.keys.push(PersonKey {
        person: s(name),
        public_key: k.public().to_hex(),
        registered_at: s("2026-01-01"),
        registered_by: s("Ola Sun"),
        revoked_from: s(""),
        proof: vleo_files::people::proof(k, name),
    });
    f.signatures.retain(|x| x.scope != FILE);
    signed(f, "Ola Sun", &key(13))
}

#[test]
fn a_node_is_assigned_to_its_node_engineer_never_their_deputy_or_an_assistant() {
    let f = with_node_engineer(&with_onset(), "Ivo Node", "Nia Node", &key(17));
    let assign = |g: &File, person: &str| {
        ola(
            g,
            &Edit::Assign {
                block: s("sw_storm_onset"),
                person: s(person),
            },
        )
    };
    let e = assign(&f, "Ivo Node").unwrap_err();
    assert!(e.contains("Ivo Node is Nia Node's deputy"), "{e}");
    let f = with_node_engineer(&with_onset(), "Claude", "", &key(18));
    let e = assign(&f, "Claude").unwrap_err();
    assert!(e.contains("Claude is an assistant's name"), "{e}");
}

#[test]
fn an_assignment_is_taken_away_by_the_owner_and_nobody_writes_it_until_it_is_given_again() {
    let f = with_onset();
    let unassign = |g: &File, block: &str| ola(g, &Edit::Unassign { block: s(block) });
    let e = unassign(&f, "sw_storm_onset").unwrap_err();
    assert!(e.contains("sw_storm_onset is assigned to nobody"), "{e}");
    let (a, _) = ola(
        &f,
        &Edit::Assign {
            block: s("sw_storm_onset"),
            person: s("Nia Node"),
        },
    )
    .unwrap();
    let (b, _) = unassign(&a, "sw_storm_onset").unwrap();
    assert!(
        !b.assignments
            .iter()
            .any(|x| x.block_uid == "sw_storm_onset"),
        "{:?}",
        b.assignments
    );
    let c = b.changes.last().unwrap();
    assert_eq!(c.what, format!("unassign: {WHY}"));
    assert!(c.before.as_deref().unwrap().contains("Nia Node"), "{c:?}");
    // Unassigned, the block is removed as any open block is.
    ola(
        &b,
        &Edit::Remove {
            block: s("sw_storm_onset"),
        },
    )
    .expect("an unassigned open block is removed");
    // An issued node unassigned: Nia's signature no longer counts for it.
    let (u, _) = unassign(&solar(), BAND).unwrap();
    let pr = programme();
    let sy = systems(&pr);
    let g = chain::open_group_in(&u, &sy, &pr).unwrap();
    assert!(chain::check_node(&band(), BAND, &g).is_err());
}

#[test]
fn an_issued_node_assigned_again_counts_only_its_new_node_engineer_s_signature() {
    let f = with_node_engineer(&solar(), "Ivo Node", "", &key(17));
    let pr = programme();
    let sy = systems(&pr);
    let before = chain::open_group_in(&f, &sy, &pr).unwrap();
    chain::check_node(&band(), BAND, &before).expect("Nia's signature counts while she has it");
    let (moved, _) = ola(
        &f,
        &Edit::Assign {
            block: s(BAND),
            person: s("Ivo Node"),
        },
    )
    .unwrap();
    let after = chain::open_group_in(&moved, &sy, &pr).unwrap();
    let e = chain::check_node(&band(), BAND, &after).unwrap_err();
    assert!(e.message().contains(BAND), "{}", e.message());
}

#[test]
fn an_input_of_a_framed_block_is_wired_to_where_it_comes_from() {
    let f = with_onset();
    let band_out = out_port(&from_release(BAND));
    let wire = |g: &File, from: &str, elsewhere: &[File]| {
        ola_with(
            g,
            &nodes(),
            elsewhere,
            &Edit::Wire {
                block: s("sw_storm_onset"),
                port: s("f107"),
                from: s(from),
            },
        )
    };
    // From an output of a block of the branch.
    let (a, _) = wire(&f, &format!("{BAND}.{band_out}"), &[]).unwrap();
    let into: Vec<_> = a
        .wires
        .iter()
        .filter(|w| w.to_block == "sw_storm_onset")
        .collect();
    assert_eq!(into.len(), 1);
    assert_eq!(into[0].from_ref, format!("{BAND}.{band_out}"));
    assert_eq!(a.changes.last().unwrap().what, format!("wire: {WHY}"));
    // Wired again, from a case: the one wire, replaced.
    let (b, _) = wire(&a, "case", &[]).unwrap();
    let into: Vec<_> = b
        .wires
        .iter()
        .filter(|w| w.to_block == "sw_storm_onset")
        .collect();
    assert_eq!((into.len(), into[0].from_ref.as_str()), (1, "case"));
    // From another group's node, which that group's file holds.
    let mut power = group_file("l3_power");
    power.blocks.push(Block {
        uid: s("pw_bus"),
        id: s("pw_bus"),
        behaviour: s("method"),
        ..Block::default()
    });
    power.ports.push(Port {
        block_uid: s("pw_bus"),
        direction: s("out"),
        name: s("pw_bus"),
        port_type: s("number"),
        ..Port::default()
    });
    wire(&f, "l3_power.pw_bus.pw_bus", std::slice::from_ref(&power)).unwrap();

    for (from, says) in [
        ("sw_nowhere.x", "another group's node is read as"),
        ("l3_power.pw_bus", "another group's node is read as"),
        ("l3_power.pw_bus.pw_volts", "comes from nothing"),
        (&format!("{BAND}.f107") as &str, "comes from nothing"),
        ("sw_storm_onset.sw_storm_onset", "reads itself"),
        ("", "comes from nothing"),
    ] {
        let e = wire(&f, from, std::slice::from_ref(&power)).unwrap_err();
        assert!(e.contains(says), "{from:?}: expected «{says}» in: {e}");
    }
    let refused = |block: &str, port: &str, says: &str| {
        let e = ola(
            &f,
            &Edit::Wire {
                block: s(block),
                port: s(port),
                from: s("case"),
            },
        )
        .unwrap_err();
        assert!(e.contains(says), "expected «{says}» in: {e}");
    };
    refused(
        "sw_storm_onset",
        "sw_storm_onset",
        "has no input sw_storm_onset",
    );
    refused("sw_storm_onset", "f10", "has no input f10");
    refused(BAND, "f107", "is issued");
}

// ── Issued as a node file of its own (docs/briefs/F5.md, F5-c) ──

/// Solar's onset, its input from a case and assigned to Nia: ready to issue.
fn onset_ready() -> File {
    let f = with_onset();
    let (f, _) = ola(
        &f,
        &Edit::Wire {
            block: s("sw_storm_onset"),
            port: s("f107"),
            from: s("case"),
        },
    )
    .unwrap();
    ola(
        &f,
        &Edit::Assign {
            block: s("sw_storm_onset"),
            person: s("Nia Node"),
        },
    )
    .unwrap()
    .0
}

fn issue(block: &str, behaviour: &str) -> Edit {
    Edit::Issue {
        block: s(block),
        behaviour: s(behaviour),
    }
}

#[test]
fn a_framed_block_is_issued_as_a_node_file_its_node_engineer_writes() {
    let before = onset_ready();
    let (after, _) = ola(&before, &issue("sw_storm_onset", "method")).unwrap();
    // The group file holds none of its rows now, and keeps its assignment.
    assert!(!after.blocks.iter().any(|b| b.uid == "sw_storm_onset"));
    assert!(!after.ports.iter().any(|p| p.block_uid == "sw_storm_onset"));
    assert!(!after.wires.iter().any(|w| w.to_block == "sw_storm_onset"));
    assert!(after
        .assignments
        .iter()
        .any(|a| a.block_uid == "sw_storm_onset" && a.person == "Nia Node"));
    let c = after.changes.last().unwrap();
    assert_eq!(c.what, format!("issue: {WHY}"));
    assert!(
        c.after
            .as_deref()
            .unwrap()
            .contains("nodes/sw_storm_onset.vnode"),
        "{c:?}"
    );
    // Its node file: the block, its ports and its wire, Nia its writer, at
    // her contract, revision 1, signed by nobody yet.
    let n = breakdown::issued(&before, "sw_storm_onset", "method").unwrap();
    assert_eq!(n.kind().unwrap(), Kind::Node);
    for (k, v) in [
        ("group_id", "l3_solar"),
        ("block_uid", "sw_storm_onset"),
        ("writer", "Nia Node"),
        ("revision", "1"),
        ("contract_version", "1"),
    ] {
        assert_eq!(n.meta.get(k).map(String::as_str), Some(v), "{k}");
    }
    assert_eq!(n.blocks.len(), 1);
    assert_eq!(
        (n.blocks[0].behaviour.as_str(), n.blocks[0].revision),
        ("method", 1)
    );
    assert_eq!(n.ports.len(), 2);
    assert_eq!(n.wires.len(), 1);
    assert!(n.signatures.is_empty());
    n.check_meta().unwrap();
    // The branch holds it once, in its node file.
    let ns = [n];
    let branch = breakdown::Branch::new(&after, &ns);
    assert_eq!(
        branch
            .blocks()
            .iter()
            .filter(|(b, _)| b.uid == "sw_storm_onset")
            .count(),
        1
    );

    let refused = |g: &File, e: Edit, says: &str| {
        let got = ola(g, &e).unwrap_err();
        assert!(got.contains(says), "expected «{says}» in: {got}");
    };
    let unassigned = with_onset();
    let (unassigned, _) = ola(
        &unassigned,
        &Edit::Wire {
            block: s("sw_storm_onset"),
            port: s("f107"),
            from: s("case"),
        },
    )
    .unwrap();
    refused(
        &unassigned,
        issue("sw_storm_onset", "method"),
        "is assigned to nobody",
    );
    let unwired = ola(
        &with_onset(),
        &Edit::Assign {
            block: s("sw_storm_onset"),
            person: s("Nia Node"),
        },
    )
    .unwrap()
    .0;
    refused(
        &unwired,
        issue("sw_storm_onset", "method"),
        "its input f107 does not say where it comes from",
    );
    refused(
        &before,
        issue("sw_storm_onset", "open"),
        "is issued as a method or a stated node",
    );
    refused(&before, issue("l3_solar", "method"), "is a heading");
    refused(&before, issue(BAND, "method"), "is issued already");
    // A block with blocks framed under it.
    let (parent, _) = ola(
        &before,
        &Edit::Add {
            parent: s("sw_storm_onset"),
            id: s("sw_storm_part"),
            question: s("A part of it"),
        },
    )
    .unwrap();
    refused(
        &parent,
        issue("sw_storm_onset", "method"),
        "has blocks under it",
    );
    // A block with no output, and no question.
    let (bare, _) = ola(
        &solar(),
        &Edit::Add {
            parent: s("l3_solar"),
            id: s("sw_bare"),
            question: s(""),
        },
    )
    .unwrap();
    let (bare, _) = ola(
        &bare,
        &Edit::Assign {
            block: s("sw_bare"),
            person: s("Nia Node"),
        },
    )
    .unwrap();
    refused(&bare, issue("sw_bare", "method"), "has no question");
}

#[test]
fn an_issue_in_the_copy_writes_the_node_file_beside_the_group_file() {
    let copy = std::env::temp_dir().join(format!("vleo-issue-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&copy);
    let dir = copy.join("groups/l3_solar");
    std::fs::create_dir_all(dir.join("nodes")).unwrap();
    let group = dir.join("l3_solar.vgroup");
    vleo_files::sqlite::write(&onset_ready(), &group).unwrap();
    for n in nodes() {
        let id = n.blocks[0].id.clone();
        vleo_files::sqlite::write(&n, &dir.join(format!("nodes/{id}.vnode"))).unwrap();
    }
    let digest = |p: &std::path::Path| vleo_files::copied::digest(&std::fs::read(p).unwrap());
    let pr = programme();
    let sy = systems(&pr);
    let made = breakdown::edit_in_place(
        &copy,
        "l3_solar",
        &digest(&group),
        &issue("sw_storm_onset", "method"),
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
        matches!(made, breakdown::Framed::Written { .. }),
        "{made:?}"
    );
    let node = dir.join("nodes/sw_storm_onset.vnode");
    let n = vleo_files::sqlite::read(&node).unwrap();
    assert_eq!(n.meta.get("writer").map(String::as_str), Some("Nia Node"));
    let g = vleo_files::sqlite::read(&group).unwrap();
    assert!(!g.blocks.iter().any(|b| b.uid == "sw_storm_onset"));
    // A node file already at that path is never written over: the group
    // file as it was before the issue, given again, writes nothing.
    vleo_files::sqlite::write(&onset_ready(), &group).unwrap();
    let held = std::fs::read(&node).unwrap();
    let e = breakdown::edit_in_place(
        &copy,
        "l3_solar",
        &digest(&group),
        &issue("sw_storm_onset", "method"),
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
        e.message().contains("sw_storm_onset is in") || e.message().contains("never written over"),
        "{}",
        e.message()
    );
    assert_eq!(std::fs::read(&node).unwrap(), held);
    let _ = std::fs::remove_dir_all(&copy);
}

// ── Many nodes set up at once, from a sheet (docs/briefs/F5.md, F5-c) ──

const SHEET: &str = "id,parent,question,output,type,unit,lower,upper,inputs,assign
sw_storm_onset,l3_solar,When does a storm begin?,sw_storm_onset,number,1,0,10,f107:1=case,Nia Node
sw_storm_peak,l3_solar,How high does it go?,sw_storm_peak,number,1,,,onset:1=sw_storm_onset.sw_storm_onset; f107:1=case,Nia Node
sw_storm_note,sw_storm_onset,What is noted of it?,sw_storm_note,text,,,,,
";

fn sheet(csv: &str) -> Edit {
    Edit::Sheet { csv: s(csv) }
}

#[test]
fn a_sheet_sets_up_many_blocks_at_once_each_edit_saying_why() {
    let (f, _) = ola(&solar(), &sheet(SHEET)).unwrap();
    let ns = nodes();
    let branch = breakdown::Branch::new(&f, &ns);
    for id in ["sw_storm_onset", "sw_storm_peak", "sw_storm_note"] {
        assert!(
            branch.blocks().iter().any(|(b, _)| b.uid == id),
            "{id} is set up"
        );
    }
    assert_eq!(
        branch.children("sw_storm_onset")[0].uid,
        "sw_storm_note",
        "a row hangs from the block its parent names"
    );
    let ports = |uid: &str| f.ports.iter().filter(|p| p.block_uid == uid).count();
    assert_eq!(
        (
            ports("sw_storm_onset"),
            ports("sw_storm_peak"),
            ports("sw_storm_note")
        ),
        (2, 3, 1)
    );
    let peak = f
        .ports
        .iter()
        .find(|p| p.block_uid == "sw_storm_onset" && p.direction == "out")
        .unwrap();
    assert_eq!(
        (peak.unit.as_str(), peak.lower.as_str(), peak.upper.as_str()),
        ("1", "0", "10")
    );
    // A row reads another's output, set up in the same sheet.
    assert!(f.wires.iter().any(|w| w.to_block == "sw_storm_peak"
        && w.to_port == "onset"
        && w.from_ref == "sw_storm_onset.sw_storm_onset"));
    assert_eq!(f.wires.len(), 3);
    let assigned: Vec<&str> = f
        .assignments
        .iter()
        .filter(|a| a.person == "Nia Node" && a.block_uid != BAND)
        .map(|a| a.block_uid.as_str())
        .collect();
    assert_eq!(assigned, ["sw_storm_onset", "sw_storm_peak"]);
    // Each edit says why: 3 added, 6 ports, 3 wires, 2 assigned.
    let made: Vec<&str> = f.changes.iter().map(|c| c.what.as_str()).collect();
    assert_eq!(made.len(), 14, "{made:?}");
    // Each says it came from the sheet, by the row that made it.
    assert!(
        made.iter()
            .all(|w| w.contains(WHY) && w.contains("(the sheet's row ")),
        "{made:?}"
    );
    assert!(made.contains(&format!("add: {WHY} (the sheet's row 3)").as_str()));
    // A row reads an output set up in a later row as well as an earlier one.
    let lines: Vec<&str> = SHEET.lines().collect();
    let swapped = format!("{}\n{}\n{}\n{}\n", lines[0], lines[2], lines[1], lines[3]);
    let (g, _) = ola(&solar(), &sheet(&swapped)).unwrap();
    assert_eq!(g.wires.len(), 3);
    // The sheet's blocks are ready to issue, one by one.
    let (issued, _) = ola(&f, &issue("sw_storm_peak", "method")).unwrap();
    assert!(!issued.blocks.iter().any(|b| b.uid == "sw_storm_peak"));
}

#[test]
fn a_sheet_with_one_row_refused_sets_up_nothing_and_names_the_row() {
    let refused = |csv: &str, says: &[&str]| {
        let e = ola(&solar(), &sheet(csv)).unwrap_err();
        for s in says {
            assert!(e.contains(s), "expected «{s}» in: {e}");
        }
    };
    let bad_person = SHEET.replace(
        "onset; f107:1=case,Nia Node",
        "onset; f107:1=case,Zed Nobody",
    );
    refused(
        &bad_person,
        &[
            "row 3",
            "sw_storm_peak",
            "registers no node engineer Zed Nobody",
        ],
    );
    let bad_wire = SHEET.replace(
        "onset:1=sw_storm_onset.sw_storm_onset",
        "onset:1=l3_nowhere.sw_nowhere.x",
    );
    refused(&bad_wire, &["row 3", "sw_storm_peak", "comes from nothing"]);
    let bad_input = SHEET.replace(
        "f107:1=case,Nia Node\nsw_storm_peak",
        "f107 from case,Nia Node\nsw_storm_peak",
    );
    refused(&bad_input, &["row 2", "sw_storm_onset", "name:unit=from"]);
    refused(
        "id,parent,colour\nsw_x,l3_solar,red\n",
        &["column", "colour"],
    );
    refused(
        "id,parent,question,output,type,unit,lower,upper,inputs,assign\n",
        &["no row"],
    );
    let taken = format!("{SHEET}sw_storm_onset,l3_solar,Again?,sw_storm_onset,number,1,,,,\n");
    refused(&taken, &["row 5", "sw_storm_onset", "never used twice"]);
}

#[test]
fn a_sheet_is_read_as_a_person_reads_it_or_refused_by_its_row() {
    let refused = |csv: &str, says: &str| {
        let e = ola(&solar(), &sheet(csv)).unwrap_err();
        assert!(e.contains(says), "expected «{says}» in: {e}");
    };
    // A quote never closed would make every row after it one cell.
    let open_quote = SHEET.replace("How high does it go?", "Ring 5\" wide?");
    refused(
        &open_quote,
        "the sheet's row 3 opens a quote that is never closed",
    );
    // A comma outside quotes moves every column after it.
    let ragged = SHEET.replace("How high does it go?", "How high, really?");
    refused(&ragged, "the sheet's row 3 has 11 cells and its header 10");
    // An input named twice, and a column named twice.
    let twice = SHEET.replace(
        "onset:1=sw_storm_onset.sw_storm_onset; f107:1=case",
        "f107:1=case; f107:1=sw_storm_onset.sw_storm_onset",
    );
    refused(&twice, "the sheet's row 3 (sw_storm_peak) names its input f107 twice");
    refused(
        "id,parent,unit,unit\nsw_x,l3_solar,1,m\n",
        "names its column unit twice",
    );
    // What a spreadsheet writes is read: a byte-order mark, CRLF, a quoted
    // cell with a comma and a doubled quote, and a blank line, which is
    // counted.
    let excel = format!("\u{feff}{}", SHEET.replace('\n', "\r\n")).replace(
        "How high does it go?",
        "\"How high, \"\"really\"\"?\"",
    );
    let (f, _) = ola(&solar(), &sheet(&excel)).unwrap();
    let peak = f.blocks.iter().find(|b| b.uid == "sw_storm_peak").unwrap();
    assert_eq!(peak.question, "How high, \"really\"?");
    let blank = SHEET.replacen('\n', "\n\n", 1).replace("Nia Node\nsw_storm_note", "Zed Nobody\nsw_storm_note");
    refused(&blank, "the sheet's row 4 (sw_storm_peak)");
}

/// `f` with an open block `id` under `parent`, an input `x` and an output
/// named as the block.
fn with_block(f: &File, parent: &str, id: &str) -> File {
    let (mut f, _) = ola(
        f,
        &Edit::Add {
            parent: s(parent),
            id: s(id),
            question: s("What does it answer?"),
        },
    )
    .unwrap();
    for (dir, name) in [("in", "x"), ("out", id)] {
        let p = Port {
            direction: s(dir),
            name: s(name),
            port_type: s("number"),
            unit: s("1"),
            state: s("open"),
            ..Port::default()
        };
        f = ola(
            &f,
            &Edit::Fill {
                block: s(id),
                part: Part::Port(Box::new(p)),
            },
        )
        .unwrap()
        .0;
    }
    f
}

fn wired(
    g: &File,
    block: &str,
    port: &str,
    from: &str,
    elsewhere: &[File],
) -> Result<File, String> {
    ola_with(
        g,
        &nodes(),
        elsewhere,
        &Edit::Wire {
            block: s(block),
            port: s(port),
            from: s(from),
        },
    )
    .map(|(f, _)| f)
}

#[test]
fn a_wire_makes_no_loop_through_the_branch_or_its_breakdown() {
    let f = with_block(&with_onset(), "l3_solar", "sw_storm_peak");
    // The peak reads the onset; the onset reading the peak would close a loop.
    let f = wired(
        &f,
        "sw_storm_peak",
        "x",
        "sw_storm_onset.sw_storm_onset",
        &[],
    )
    .unwrap();
    let e = wired(
        &f,
        "sw_storm_onset",
        "f107",
        "sw_storm_peak.sw_storm_peak",
        &[],
    )
    .unwrap_err();
    assert!(
        e.contains("which already reads sw_storm_onset through the branch"),
        "{e}"
    );
    // A block under the onset reading the onset's answer reads what it feeds.
    let f = with_block(&f, "sw_storm_onset", "sw_storm_rise");
    let e = wired(
        &f,
        "sw_storm_rise",
        "x",
        "sw_storm_onset.sw_storm_onset",
        &[],
    )
    .unwrap_err();
    assert!(
        e.contains("reads the output of sw_storm_onset, which it hangs under"),
        "{e}"
    );
}

#[test]
fn another_group_s_node_is_read_by_its_uid_from_what_that_group_frames_and_holds() {
    let f = with_onset();
    // Power's bus, renamed: its uid is not its id.
    let power = |archived: i64, kind: Kind| {
        let mut p = File::new(kind, APP);
        p.meta.insert(s("group_id"), s("l3_power"));
        p.blocks.push(Block {
            uid: s("pw_bus_7"),
            id: s("pw_bus"),
            behaviour: s("method"),
            archived,
            ..Block::default()
        });
        p.ports.push(Port {
            block_uid: s("pw_bus_7"),
            direction: s("out"),
            name: s("pw_volts"),
            port_type: s("number"),
            ..Port::default()
        });
        p
    };
    let live = power(0, Kind::Group);
    wired(
        &f,
        "sw_storm_onset",
        "f107",
        "l3_power.pw_bus_7.pw_volts",
        std::slice::from_ref(&live),
    )
    .expect("read by its uid");
    for (from, there) in [
        ("l3_power.pw_bus.pw_volts", power(0, Kind::Group)),
        ("l3_power.pw_bus_7.pw_volts", power(1, Kind::Group)),
        ("l3_power.pw_bus_7.pw_volts", power(0, Kind::GroupRelease)),
    ] {
        let e = wired(
            &f,
            "sw_storm_onset",
            "f107",
            from,
            std::slice::from_ref(&there),
        )
        .unwrap_err();
        assert!(e.contains("comes from nothing"), "{from:?}: {e}");
    }
}
