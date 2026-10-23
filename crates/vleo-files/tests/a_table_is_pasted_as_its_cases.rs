//! A table pasted from a spreadsheet, or a CSV file's text, is read as the
//! node's cases (docs/briefs/F8.md, F8-b item 1; questions 72 and 73 as
//! proposed): `Edit::Table` becomes `results/isolation.csv` as
//! `vleo_files::table::cases` reads it against the node's own ports, each
//! number read in its column's unit and written in its port's, held to
//! everything a cases edit is held to; and its dry run writes nothing.
//!
//! The node is invented for the check, and says so (F4, question 5): solar
//! 1.1's `sw_activity_band` in a node file of its own, renamed
//! `sw_drag_invented` and given a drag force's ports, `rho` in kg/m^3, `v` in
//! m/s, `cd` (symbol `Cd`) and `a` (symbol `A`) in, `drag_force` in N out,
//! and the drag equation as its method. Its cases are worked by hand (origin
//! `hand`), each `0.5 * rho * v^2 * cd * a`: 6.5219e-4 N at 1e-11 kg/m^3,
//! 7700 m/s, 2.2 and 1 m^2; 5.625e-4 N at 2e-11, 7500, 2 and 0.5; 1.90608e-3
//! N at 1.5e-11, 7600, 2.2 and 2. The paste gives the speed in km/s and the
//! force in mN, as a spreadsheet might. No expected value comes from the
//! code under test.
//!
//! - pasted with tabs, its origin said once: read cell by cell, each speed
//!   and force written in its port's unit, said back; the dry run judges
//!   every case and writes nothing; saved, the node's cases are what was
//!   read, its declaration restated;
//! - a CSV file's text with its own origin column is read the same way;
//! - refused by name, with nothing written: no origin, an origin given beside
//!   the table's own, one not among the four, a column no case has, an input
//!   with no column, no answer, no tolerance, a cell with a comma, a cell of
//!   the wrong dimension, a range where a value goes, and cases and a table
//!   sent together.

use std::path::{Path, PathBuf};
use vleo_files::chain::{self, Registry, FILE};
use vleo_files::edit::{self, Declared, Edit, Edits, Pasted, Tried};
use vleo_files::keys::SigningKey;
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::save::Saved;

const APP: &str = "vleo 1.0.0 (test)";
const AT: &str = "2026-10-23T09:00:00Z";
const BAND: &str = "sw_activity_band";
const DRAG: &str = "sw_drag_invented";
const PATH: &str = "groups/l3_solar/nodes/sw_drag_invented.vnode";

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
/// ports, the drag equation as its method, and one case worked by hand.
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
        body: s("return 0.5 * rho * v^2 * cd * a\n"),
    });
    n.tables.retain(|t| t.path == "declaration.csv");
    for t in n.tables.iter_mut() {
        t.scope = s(DRAG);
    }
    n.tables.push(Tbl {
        scope: s(DRAG),
        path: s("results/isolation.csv"),
        csv: s("rho [kg/m^3],v [m/s],cd [1],a [m^2],answer [N],tolerance,refuses,origin,says\n\
                1e-11,7700,2.2,1,6.5219e-4,1e-12,no,hand,\"0.5 * 1e-11 * 7700^2 * 2.2 * 1, worked by hand\"\n"),
    });
    n
}

fn nia() -> Declared {
    Declared {
        author: s("Nia Node"),
        ai: s("none"),
        source: s(""),
        checked_by: s(""),
    }
}

fn pasted(text: &str, origin: &str) -> Edits {
    Edits {
        edits: vec![Edit::Table(Pasted {
            text: s(text),
            origin: s(origin),
        })],
        declared: Some(nia()),
        why: None,
    }
}

fn table<'a>(f: &'a File, path: &str) -> &'a str {
    f.tables
        .iter()
        .find(|t| t.scope == DRAG && t.path == path)
        .map_or("", |t| t.csv.as_str())
}

/// A copy holding `f` at the drag node's path, and the SHA-256 of its bytes.
fn copy(name: &str, f: &File) -> (PathBuf, String) {
    let c = std::env::temp_dir().join(format!("vleo-table-{name}-{}", std::process::id()));
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

/// The three cases as a spreadsheet copies them: tabs, the speed in km/s
/// and the force in mN, a pure number with no unit.
const PASTE: &str = "rho [kg/m^3]\tv [km/s]\tcd\ta [m^2]\tanswer [mN]\ttolerance\tsays\n\
    1e-11\t7.7\t2.2\t1\t0.65219\t1e-9\tworked by hand\n\
    2e-11\t7.5\t2\t0.5\t0.5625\t1e-9\tworked by hand\n\
    1.5e-11\t7.6\t2.2\t2\t1.90608\t1e-9\tworked by hand\n";

/// What the paste becomes: each speed and force in its port's unit, the
/// origin said once written on every row.
const CASES: &str = "rho [kg/m^3],v [m/s],cd [1],a [m^2],answer [N],tolerance,says,origin\n\
    1e-11,7700,2.2,1,0.00065219,1e-9,worked by hand,hand\n\
    2e-11,7500,2,0.5,0.0005625,1e-9,worked by hand,hand\n\
    1.5e-11,7600,2.2,2,0.00190608,1e-9,worked by hand,hand\n";

#[test]
fn a_pasted_table_is_read_cell_by_cell_and_saved_as_the_cases() {
    let (c, opened) = copy("saved", &drag());
    let f = vleo_files::sqlite::read(&c.join(PATH)).unwrap();
    let read = edit::table_of(
        &f,
        DRAG,
        &Pasted {
            text: s(PASTE),
            origin: s("hand"),
        },
    )
    .unwrap();
    assert_eq!(read.csv, CASES.replace("    ", ""));
    assert_eq!(read.cases, 3);
    let v = read
        .cells
        .iter()
        .find(|x| x.line == 2 && x.column == "v")
        .unwrap();
    assert!(
        v.typed == "7.7"
            && v.said.contains("7.7 km/s")
            && v.said.ends_with("; written as 7700 m/s"),
        "{v:?}"
    );
    let rho = read
        .cells
        .iter()
        .find(|x| x.line == 2 && x.column == "rho")
        .unwrap();
    assert!(
        !rho.said.contains("written as"),
        "in its port's unit already: {rho:?}"
    );

    // The dry run judges every case and writes nothing.
    let e = pasted(PASTE, "hand");
    match tried(&c, &opened, &e) {
        Tried::Would {
            revision,
            refused,
            report,
            ..
        } => {
            assert_eq!((revision, refused), (2, None));
            let (_, rep) = report.expect("the method runs on the pasted cases");
            assert_eq!(rep.cases.len(), 3);
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
    let f = vleo_files::sqlite::read(&c.join(PATH)).unwrap();
    assert_eq!(
        table(&f, "results/isolation.csv"),
        CASES.replace("    ", "")
    );
    assert!(
        table(&f, "declaration.csv").contains("Nia Node,none,2026-10-23"),
        "{}",
        table(&f, "declaration.csv")
    );
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn a_csv_file_with_its_own_origin_column_is_read_as_the_cases() {
    let (c, _) = copy("csv", &drag());
    let f = vleo_files::sqlite::read(&c.join(PATH)).unwrap();
    let text = "rho [kg/m^3],v [m/s],cd,a [m^2],answer [N],tolerance,origin\n\
                1e-11,7700,2.2,1,6.5219e-4,1e-9,paper\n\
                2e-11,7500,2,0.5,5.625e-4,1e-9,spreadsheet\n";
    let read = edit::table_of(
        &f,
        DRAG,
        &Pasted {
            text: s(text),
            origin: s(""),
        },
    )
    .unwrap();
    assert_eq!(
        read.csv,
        "rho [kg/m^3],v [m/s],cd [1],a [m^2],answer [N],tolerance,origin\n\
         1e-11,7700,2.2,1,0.00065219,1e-9,paper\n\
         2e-11,7500,2,0.5,0.0005625,1e-9,spreadsheet\n",
        "every number written as the amount reader says it, its own origins kept"
    );
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn a_table_is_refused_by_name_with_nothing_written() {
    let (c, opened) = copy("refused", &drag());
    let f = vleo_files::sqlite::read(&c.join(PATH)).unwrap();
    let read = |text: &str, origin: &str| {
        edit::table_of(
            &f,
            DRAG,
            &Pasted {
                text: s(text),
                origin: s(origin),
            },
        )
        .map(|r| r.csv)
        .map_err(|e| e.message().to_string())
    };
    let head = "rho [kg/m^3]\tv [m/s]\tcd\ta [m^2]\tanswer [N]\ttolerance";
    let row = "1e-11\t7700\t2.2\t1\t6.5219e-4\t1e-9";
    let one = format!("{head}\n{row}\n");
    for (what, text, origin, why) in [
        ("no origin", one.clone(), "", "say where these cases' expected values come from: code, hand, spreadsheet, paper"),
        ("an origin not among the four", one.clone(), "assistant", "\"assistant\" is not where an expected value comes from"),
        ("an origin beside the table's own", format!("{head}\torigin\n{row}\thand\n"), "hand", "the table has its own origin column"),
        ("a row's own origin not among the four", format!("{head}\torigin\n{row}\tguessed\n"), "", "line 2: the origin \"guessed\""),
        ("a column no case has", format!("{head}\tmass [kg]\n{row}\t1\n"), "hand", "a column \"mass\" no case has: its columns are the inputs rho, v, cd, a"),
        ("an input with no column", "rho [kg/m^3]\tv [m/s]\tcd\tanswer [N]\ttolerance\n1e-11\t7700\t2.2\t6.5219e-4\t1e-9\n".to_string(), "hand", "no column for the input a"),
        ("no answer", "rho [kg/m^3]\tv [m/s]\tcd\ta [m^2]\ttolerance\n1e-11\t7700\t2.2\t1\t1e-9\n".to_string(), "hand", "no answer column"),
        ("no tolerance", "rho [kg/m^3]\tv [m/s]\tcd\ta [m^2]\tanswer [N]\n1e-11\t7700\t2.2\t1\t6.5219e-4\n".to_string(), "hand", "no tolerance column"),
        ("a decimal comma", format!("{head}\n1e-11\t7700\t2,2\t1\t6.5219e-4\t1e-9\n"), "hand", "line 2, cd: «2,2» has a comma"),
        ("the wrong dimension", "rho [kg/m^3]\tv [kg]\tcd\ta [m^2]\tanswer [N]\ttolerance\n1e-11\t7700\t2.2\t1\t6.5219e-4\t1e-9\n".to_string(), "hand", "line 2, v: [kg] is a Mass"),
        ("a range", format!("{head}\n1e-11\t7000 to 8000\t2.2\t1\t6.5219e-4\t1e-9\n"), "hand", "line 2, v: «7000 to 8000» is a range or a spread: a case holds one value"),
        ("a header and no case", format!("{head}\n"), "hand", "a header and no case"),
    ] {
        let e = read(&text, origin).unwrap_err();
        assert!(e.contains(why), "{what}: {e}");
    }
    // Cases and a table sent together are refused, with nothing written.
    let both = Edits {
        edits: vec![
            Edit::Cases(s(CASES)),
            Edit::Table(Pasted {
                text: s(PASTE),
                origin: s("hand"),
            }),
        ],
        declared: Some(nia()),
        why: None,
    };
    match tried(&c, &opened, &both) {
        Tried::Would { refused, .. } => assert!(
            refused
                .as_deref()
                .is_some_and(|r| r.contains("given cases and a table, both")),
            "{refused:?}"
        ),
        Tried::SecondWriter { .. } => panic!("a second writer"),
    }
    assert!(
        edit::edit_in_place(&c, PATH, &opened, &both, "Nia Node", AT, APP, &solar()).is_err(),
        "nothing is written"
    );
    assert_eq!(digest(&c), opened);
    let _ = std::fs::remove_dir_all(&c);
}
