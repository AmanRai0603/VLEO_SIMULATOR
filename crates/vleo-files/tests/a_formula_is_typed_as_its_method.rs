//! A formula is typed as the node's method (docs/briefs/F8.md, F8-a item 4;
//! question 58 as proposed): `Edit::Formula` becomes the method text
//! `formula::read` makes of it against the node's own ports
//! (`checks::node_signature`), held to everything a method edit is held to,
//! the declaration restated and checked, the record of which belief broke,
//! the save; and its dry run writes nothing.
//!
//! The node is invented for the check, and says so (F4, question 5): solar
//! 1.1's `sw_activity_band` in a node file of its own, renamed
//! `sw_drag_invented` and given a drag force's ports, `rho` in kg/m^3, `v` in
//! m/s, `cd` (symbol `Cd`) and `a` (symbol `A`) in, `drag_force` in N out,
//! and two cases worked by hand (origin `hand`): the force is 6.5219e-4 N at
//! 1e-11 kg/m^3, 7700 m/s, a drag coefficient of 2.2 and 1 m^2, and
//! 5.625e-4 N at 2e-11 kg/m^3, 7500 m/s, 2 and 0.5 m^2. Its method before is
//! a wrong one, twice the force, so that the formula is seen to change what
//! the cases say. No expected value comes from the code under test.
//!
//! - typed as `D = 0.5 * rho * v^2 * Cd * A`, saved as revision 2: its
//!   method text says what was typed, reads `Cd` and `A` by their ports'
//!   symbols, and agrees with both cases; its declaration and its record are
//!   written as a method edit's are;
//! - refused by name, with nothing written: no declaration, a relation an
//!   assistant supplied, an input no port has (its name listed), a formula of
//!   the wrong dimension, and a method and a formula sent together;
//! - the dry run says what it would write and writes nothing;
//! - a node that holds design/'s `method` beside its `pseudocode` has both
//!   written with the one body, and a node whose two bodies differ is
//!   refused by its checks.

use std::path::{Path, PathBuf};
use vleo_files::chain::{self, Registry, FILE};
use vleo_files::checks;
use vleo_files::edit::{self, Declared, Edit, Edits, Formula, Tried};
use vleo_files::keys::SigningKey;
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::save::Saved;
use vleo_sheet::template::Derisk;

const APP: &str = "vleo 1.0.0 (test)";
const AT: &str = "2026-10-19T09:00:00Z";
const BAND: &str = "sw_activity_band";
const DRAG: &str = "sw_drag_invented";
const PATH: &str = "groups/l3_solar/nodes/sw_drag_invented.vnode";
const TYPED: &str = "D = 0.5 * rho * v^2 * Cd * A";

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

/// Solar's group file: its heading, Nia assigned the drag node, and Ivo.
fn solar() -> Registry {
    let mut f = group_file("l3_solar");
    f.blocks.push(Block {
        uid: s("l3_solar"),
        id: s("l3_solar"),
        behaviour: s("children"),
        ..Block::default()
    });
    for (name, seed) in [("Nia Node", 16), ("Ivo Other", 17)] {
        f.people.push(Person {
            name: s(name),
            role: s("node engineer"),
            deputy_for: s(""),
        });
        f.keys.push(PersonKey {
            person: s(name),
            public_key: key(seed).public().to_hex(),
            registered_at: s("2026-01-01"),
            registered_by: s("Ola Sun"),
            revoked_from: s(""),
            proof: vleo_files::people::proof(&key(seed), name),
        });
    }
    f.assignments.push(Assignment {
        block_uid: s(DRAG),
        person: s("Nia Node"),
        contract_version: 1,
    });
    let pr = programme();
    let sy = systems(&pr);
    chain::open_group_in(&signed(f, "Ola Sun", &key(13)), &sy, &pr).unwrap()
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
    n.blocks[0].revision = 1;
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

/// The invented drag node: the band's file, renamed, with a drag force's
/// ports, two cases worked by hand, and a wrong method.
fn drag() -> File {
    let mut n = from_release(BAND);
    n.meta.insert(s("block_uid"), s(DRAG));
    n.blocks[0].uid = s(DRAG);
    n.blocks[0].id = s(DRAG);
    n.blocks[0].question = s("What drag force does the spacecraft feel? (invented for the check)");
    let port = |dir: &str, name: &str, symbol: &str, unit: &str| Port {
        block_uid: s(DRAG),
        direction: s(dir),
        name: s(name),
        symbol: s(symbol),
        port_type: s("number"),
        unit: s(unit),
        state: s("decided"),
        ..Port::default()
    };
    n.ports = vec![
        port("in", "rho", "", "kg/m^3"),
        port("in", "v", "", "m/s"),
        port("in", "cd", "Cd", "1"),
        port("in", "a", "A", "m^2"),
        port("out", "drag_force", "", "N"),
    ];
    n.wires.clear();
    for t in n.texts.iter_mut() {
        t.scope = s(DRAG);
    }
    n.texts.retain(|t| t.kind != "pseudocode");
    n.texts.push(Text {
        scope: s(DRAG),
        kind: s("pseudocode"),
        body: s("return rho * v^2 * cd * a\n"),
    });
    n.tables.retain(|t| t.path == "declaration.csv");
    for t in n.tables.iter_mut() {
        t.scope = s(DRAG);
    }
    n.tables.push(Tbl {
        scope: s(DRAG),
        path: s("results/isolation.csv"),
        csv: s("rho [kg/m^3],v [m/s],cd [1],a [m^2],answer [N],tolerance,refuses,origin,says\n\
                1e-11,7700,2.2,1,6.5219e-4,1e-12,no,hand,\"0.5 * 1e-11 * 7700^2 * 2.2 * 1, worked by hand\"\n\
                2e-11,7500,2,0.5,5.625e-4,1e-12,no,hand,\"0.5 * 2e-11 * 7500^2 * 2 * 0.5, worked by hand\"\n"),
    });
    n
}

fn record() -> Derisk {
    Derisk {
        believed: s("The drag was twice what it is."),
        tested: s("Worked by hand at two densities."),
        learned: s("The half was missing."),
        changed: s("The method is the drag equation as typed."),
        rests_on: s("The drag equation with a constant drag coefficient."),
        breaks_if: s("The flow is not free molecular."),
        ..Derisk::default()
    }
}

fn nia() -> Declared {
    Declared {
        author: s("Nia Node"),
        ai: s("none"),
        source: s(""),
        checked_by: s(""),
    }
}

fn formula(typed: &str) -> Edits {
    Edits {
        edits: vec![Edit::Formula(Formula {
            typed: s(typed),
            fit: None,
        })],
        declared: Some(nia()),
        why: Some(record()),
    }
}

fn text<'a>(f: &'a File, kind: &str) -> &'a str {
    f.texts
        .iter()
        .find(|t| t.scope == DRAG && t.kind == kind)
        .map_or("", |t| t.body.as_str())
}

fn table<'a>(f: &'a File, path: &str) -> &'a str {
    f.tables
        .iter()
        .find(|t| t.scope == DRAG && t.path == path)
        .map_or("", |t| t.csv.as_str())
}

/// A copy holding `f` at the drag node's path, and the SHA-256 of its bytes.
fn copy(name: &str, f: &File) -> (PathBuf, String) {
    let c = std::env::temp_dir().join(format!("vleo-formula-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&c);
    let at = c.join(PATH);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    vleo_files::sqlite::write(f, &at).unwrap();
    let d = digest(&c);
    (c, d)
}

fn digest(c: &Path) -> String {
    vleo_files::copied::digest(&std::fs::read(c.join(PATH)).unwrap())
}

fn tried(c: &Path, opened: &str, e: &Edits) -> Tried {
    edit::try_edit(c, PATH, opened, e, "Nia Node", AT, APP, &solar()).unwrap()
}

#[test]
fn a_formula_typed_is_saved_as_the_method_and_agrees_with_its_cases() {
    // Before: the wrong method differs from both cases.
    let (c, opened) = copy("saved", &drag());
    let Tried::Would { report, .. } = tried(
        &c,
        &opened,
        &Edits {
            edits: vec![Edit::Method(s("return rho * v^2 * cd * a"))],
            declared: Some(nia()),
            why: Some(record()),
        },
    ) else {
        panic!("a second writer")
    };
    let (_, rep) = report.expect("the wrong method runs on its cases");
    assert!(
        rep.cases.iter().all(|(_, v)| !v.agrees()),
        "{:?}",
        rep.cases
    );

    // The dry run of the formula: it would write revision 2, agreeing with
    // both cases, and writes nothing.
    let e = formula(TYPED);
    match tried(&c, &opened, &e) {
        Tried::Would {
            revision,
            refused,
            report,
            ..
        } => {
            assert_eq!((revision, refused), (2, None));
            let (_, rep) = report.expect("it runs on its cases");
            assert_eq!(rep.cases.len(), 2);
            assert!(rep.cases.iter().all(|(_, v)| v.agrees()), "{:?}", rep.cases);
        }
        Tried::SecondWriter { .. } => panic!("a second writer"),
    }
    assert_eq!(digest(&c), opened, "a dry run writes nothing");

    let saved = edit::edit_in_place(&c, PATH, &opened, &e, "Nia Node", AT, APP, &solar()).unwrap();
    assert!(
        matches!(saved, Saved::Written { revision: 2, .. }),
        "{saved:?}"
    );
    let f = vleo_files::sqlite::open(
        &c.join(PATH),
        &vleo_files::upgrade::Upgrade { app: APP, at: AT },
    )
    .unwrap()
    .file;
    let m = text(&f, "pseudocode");
    assert!(m.starts_with(&format!("# Typed as: {TYPED}\n")), "{m}");
    assert!(m.contains("let Cd = cd") && m.contains("let A = a"), "{m}");
    assert_eq!(text(&f, "method"), "", "it holds no design/ method");
    assert!(
        table(&f, "declaration.csv").contains("Nia Node,none,2026-10-19"),
        "{}",
        table(&f, "declaration.csv")
    );
    assert!(
        table(&f, "versions.csv").contains("The half was missing."),
        "{}",
        table(&f, "versions.csv")
    );
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn a_formula_is_refused_by_name_with_nothing_written() {
    let (c, opened) = copy("refused", &drag());
    let refused = |e: &Edits| match tried(&c, &opened, e) {
        Tried::Would { refused, .. } => refused.expect("refused"),
        Tried::SecondWriter { .. } => panic!("a second writer"),
    };
    // No declaration: silence is not "none".
    let mut e = formula(TYPED);
    e.declared = None;
    assert!(refused(&e).contains("Silence is not"), "{}", refused(&e));
    // A relation an assistant supplied.
    let mut e = formula(TYPED);
    e.declared.as_mut().unwrap().ai = s("relation");
    let why = refused(&e);
    assert!(why.contains("declaration does not hold"), "{why}");
    // An input no port has, listed by name.
    let why = refused(&formula("D = 0.5 * rho * v^2 * Cd * A * q"));
    assert!(
        why.contains("formula is not written") && why.contains("«q»"),
        "{why}"
    );
    // Of the wrong dimension: a force is not a pressure.
    let why = refused(&formula("D = 0.5 * rho * v^2"));
    assert!(why.contains("formula is not written"), "{why}");
    // A method and a formula, both.
    let mut e = formula(TYPED);
    e.edits.push(Edit::Method(s("return rho * v^2 * cd * a")));
    assert!(
        refused(&e).contains("a method and a formula, both"),
        "{}",
        refused(&e)
    );
    assert_eq!(digest(&c), opened, "a refused formula writes nothing");
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn a_node_holding_design_s_method_has_both_bodies_written_as_one() {
    let mut f = drag();
    f.texts.push(Text {
        scope: s(DRAG),
        kind: s("method"),
        // design/'s copy, an older body than the release's.
        body: s("return 2 * rho * v^2 * cd * a\n"),
    });
    let a = edit::apply(&f, PATH, &formula(TYPED), "Nia Node", AT).unwrap();
    let (p, m) = (text(&a.file, "pseudocode"), text(&a.file, "method"));
    assert!(p.starts_with("# Typed as: "), "{p}");
    assert_eq!(p, m, "the checks and the engine read one method");
    // Two bodies that differ are refused by the node's checks.
    let found = checks::check_node(&f);
    assert!(
        found
            .errors()
            .any(|x| x.what.contains("holds two methods that differ")),
        "{found:?}"
    );
    let found = checks::check_node(&a.file);
    assert!(
        !found
            .errors()
            .any(|x| x.what.contains("holds two methods that differ")),
        "{found:?}"
    );
}
