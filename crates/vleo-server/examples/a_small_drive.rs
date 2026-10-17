//! A small drive of invented people, and their keys, for the shell's check in
//! a browser (tools/shell_check.py).
//!
//!     cargo run -p vleo-server --example a_small_drive -- <folder>
//!
//! writes `<folder>/drive` and `<folder>/keys`. The people are invented and
//! their keys come from fixed seeds:
//!
//! - Test Pat, the programme manager, registered, whose key START HERE anchors;
//! - Test Ola, a subsystem engineer, registered, who owns solar;
//! - Test Sam, the system engineer, registered, who owns the systems' branch;
//! - Test Ivo, the subsystem engineer the systems' file assigns propulsion to,
//!   not registered yet: the programme manager registers him;
//! - Test Nia, solar's node engineer, with two nodes assigned: one unsigned
//!   and one signed;
//! - Test Pia, solar's other node engineer, assigned `sw_activity_band` as
//!   solar's 1.1 release holds it: a node whose checks hold, unsigned, for
//!   the put-back's check (tools/put_back_check.py);
//! - two children of that node, unassigned, as F5 will issue them: one with
//!   its own method, and one whose elevated edge is at 135 sfu, invented to
//!   be wrong, for the breakdown's check (tools/branch_check.py);
//! - a node engineer whose name is made of what a page must escape;
//! - solar's releases: 1.1, as the design already holds it; 1.2, whose hot
//!   edge of the sustained F10.7 band breaks the first solar closure; and
//!   1.3, which the programme manager has not anchored, so the chain does not
//!   admit it. Today's design takes 1.2, its last good one, and marks it;
//! - beside solar, Ola owns power, whose releases folder holds something that
//!   is not a release and nothing sealed, and thermal, whose file is signed by
//!   someone who does not own it, so it is refused;
//! - structure, mounted for Ola, whose folder holds no group file;
//! - and a folder under groups/ that is not a group.
//! - Test Stranger, whom no file registers.
//!
//! Every key is locked with the passphrase "a long enough passphrase".

use std::path::Path;
use vleo_files::chain::{self, FILE};
use vleo_files::keys::{SigningKey, MIN_ITERATIONS};
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::sqlite;

fn sql(file: &Path, statement: &str) {
    let db = rusqlite::Connection::open(file).unwrap();
    assert!(db.execute(statement, []).unwrap() > 0, "{statement}");
}

/// A copy of solar's 1.1 made into `version` as its group would: the version,
/// its record, any change, and sealed again.
fn solar_release(at: &Path, version: &str, change: Option<&str>) {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../vleo-files/tests/fixtures/l3_solar-1.1.vleo");
    std::fs::copy(&fixture, at).unwrap();
    if version == "1.1" {
        return;
    }
    sql(
        at,
        &format!("UPDATE meta SET value = '{version}' WHERE key = 'version'"),
    );
    sql(
        at,
        &format!(
            "UPDATE tbl SET csv = csv || '{version},2026-10-06,Test Ola,\
             The band needed a wider margin when the cycle is low.,\
             The multiplier was read again.,\
             A wider band is safer.,\
             The multiplier of the hot edge.,,,\
             the one-sided 90th percentile,a multiplier from another table\n' \
             WHERE scope = 'group' AND path = 'versions.csv'"
        ),
    );
    if let Some(change) = change {
        sql(at, change);
    }
    let r = vleo_files::intake::Release::open(at).unwrap();
    let fp = vleo_files::seal::fingerprint_of(&r.folder, "group");
    sql(
        at,
        &format!("UPDATE meta SET value = '{fp}' WHERE key = 'fingerprint'"),
    );
}

const APP: &str = "vleo 1.0.0 (example)";
const PASSPHRASE: &str = "a long enough passphrase";

fn s(v: &str) -> String {
    v.to_string()
}

fn key(seed: u8) -> SigningKey {
    SigningKey::from_seed([seed; 32])
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

fn write(f: &File, at: &Path) {
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    sqlite::write(f, at).unwrap();
}

/// Solar's `sw_activity_band` as its 1.1 release holds it: its block, ports,
/// wires, texts and tables, in a node file of its own, revision 1.
fn activity_band() -> File {
    const UID: &str = "sw_activity_band";
    let how = vleo_files::upgrade::Upgrade {
        app: APP,
        at: "2026-10-01",
    };
    let rel = sqlite::open(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../vleo-files/tests/fixtures/l3_solar-1.1.vleo"),
        &how,
    )
    .unwrap()
    .file;
    let mut n = File::new(Kind::Node, APP);
    for (k, v) in [
        ("group_id", "l3_solar"),
        ("block_uid", UID),
        ("revision", "1"),
        ("contract_version", "1"),
        ("writer", "Test Pia"),
    ] {
        n.meta.insert(s(k), s(v));
    }
    n.blocks = vec![rel.blocks.iter().find(|b| b.uid == UID).unwrap().clone()];
    n.blocks[0].revision = 1;
    n.ports = rel
        .ports
        .iter()
        .filter(|p| p.block_uid == UID)
        .cloned()
        .collect();
    n.wires = rel
        .wires
        .iter()
        .filter(|w| w.to_block == UID)
        .cloned()
        .collect();
    n.texts = rel
        .texts
        .iter()
        .filter(|t| t.scope == UID)
        .cloned()
        .collect();
    n.tables = rel
        .tables
        .iter()
        .filter(|t| t.scope == UID)
        .cloned()
        .collect();
    n
}

/// A child of `sw_activity_band`, as F5 will issue one, unassigned: the
/// band's own method under another id, reading the band's input, with
/// `edit` made to its method. For the breakdown's check (tools/
/// branch_check.py): one that reproduces the band, and one whose elevated
/// edge is at 135 sfu, invented to be wrong.
fn activity_child(id: &str, edit: &dyn Fn(&str) -> String) -> File {
    let mut c = activity_band();
    c.meta.insert(s("block_uid"), s(id));
    c.meta.insert(s("writer"), s(""));
    c.blocks[0].uid = s(id);
    c.blocks[0].id = s(id);
    c.blocks[0].parent_uid = s("sw_activity_band");
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
            from_ref: format!("sw_activity_band.{}", p.name),
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
/// test's own invented person, `who`, never transcribed by anyone from
/// anything (F4's question 5).
fn invented(mut c: File, who: &str) -> File {
    for t in c.texts.iter_mut().filter(|t| t.kind == "pseudocode") {
        let rest: Vec<&str> = t.body.lines().skip(1).collect();
        t.body = format!(
            "# INVENTED for the breakdown's check, never part of the design: sw_activity_band's method, its elevated edge moved from 130 to 135 sfu to be wrong.\n{}\n",
            rest.join("\n")
        );
    }
    for t in c.tables.iter_mut().filter(|t| t.path == "declaration.csv") {
        t.csv = format!(
            "author,ai,date,source,checked_by\n{who},transcribed,2026-10-17,\"sw_activity_band's method, its elevated edge moved to 135 sfu: invented to be wrong, for the breakdown's check\",{who}\n"
        );
    }
    c
}

fn main() {
    let out = std::env::args()
        .nth(1)
        .expect("usage: a_small_drive <folder>");
    let out = Path::new(&out);
    let (drive, keys) = (out.join("drive"), out.join("keys"));
    let (pat, sam, ola, nia, stranger, ivo) =
        (key(61), key(62), key(63), key(64), key(65), key(66));

    let mut programme = group("programme", "Test Pat");
    register(
        &mut programme,
        "Test Pat",
        "programme manager",
        &pat,
        "Test Pat",
    );
    register(
        &mut programme,
        "Test Sam",
        "system engineer",
        &sam,
        "Test Pat",
    );
    register(
        &mut programme,
        "Test Ola",
        "subsystem engineer",
        &ola,
        "Test Pat",
    );
    mount(&mut programme, "root", "systems", "Test Sam");
    // Solar's releases, from before 1.0: the programme manager anchors 1.1
    // and 1.2, and not 1.3. The anchors are part of what he signs.
    let releases = drive.join("groups/l3_solar/releases");
    std::fs::create_dir_all(&releases).unwrap();
    let breaks = "UPDATE doc SET body = replace(body, 'return sustained', \
         'if central < 95 then\n  return central + spread * 14\nend\nreturn sustained') \
         WHERE kind = 'pseudocode' AND scope = 'sw_f107_design_long'";
    for (v, change) in [("1.1", None), ("1.2", Some(breaks)), ("1.3", Some(breaks))] {
        solar_release(&releases.join(format!("l3_solar-{v}.vleo")), v, change);
    }
    let how = vleo_files::upgrade::Upgrade {
        app: APP,
        at: "2026-10-01",
    };
    for v in ["1.1", "1.2"] {
        let r = sqlite::open(&releases.join(format!("l3_solar-{v}.vleo")), &how)
            .unwrap()
            .file;
        programme
            .signatures
            .push(vleo_files::upgrade::anchor(&r, "Test Pat", &pat, "2026-10-07").unwrap());
    }
    write(
        &signed(programme, "Test Pat", &pat),
        &drive.join("groups/programme/programme.vgroup"),
    );

    let mut systems = group("systems", "Test Sam");
    mount(
        &mut systems,
        "sys_space_environment",
        "l3_solar",
        "Test Ola",
    );
    mount(&mut systems, "sys_power", "l3_power", "Test Ola");
    mount(&mut systems, "sys_thermal", "l3_thermal", "Test Ola");
    mount(&mut systems, "sys_propulsion", "l3_prop", "Test Ivo");
    // Mounted, but its folder holds no group file: refused, not passed over.
    mount(&mut systems, "sys_structure", "l3_struct", "Test Ola");
    std::fs::create_dir_all(drive.join("groups/l3_struct")).unwrap();
    write(
        &signed(systems, "Test Sam", &sam),
        &drive.join("groups/systems/systems.vgroup"),
    );

    let mut solar = group("l3_solar", "Test Ola");
    register(&mut solar, "Test Nia", "node engineer", &nia, "Test Ola");
    register(
        &mut solar,
        "Test <b>Nell</b> & \"Co\"",
        "node engineer",
        &key(67),
        "Test Ola",
    );
    let pia = key(68);
    register(&mut solar, "Test Pia", "node engineer", &pia, "Test Ola");
    for (block, who) in [
        ("b-ap", "Test Nia"),
        ("b-f107", "Test Nia"),
        ("sw_activity_band", "Test Pia"),
    ] {
        solar.assignments.push(Assignment {
            block_uid: s(block),
            person: s(who),
            contract_version: 1,
        });
    }
    write(
        &signed(solar, "Test Ola", &ola),
        &drive.join("groups/l3_solar/l3_solar.vgroup"),
    );
    let mut node = File::new(Kind::Node, APP);
    for (k, v) in [
        ("group_id", "l3_solar"),
        ("block_uid", "b-ap"),
        ("revision", "1"),
        ("contract_version", "1"),
        ("writer", "Test Nia"),
    ] {
        node.meta.insert(s(k), s(v));
    }
    node.blocks.push(Block {
        uid: s("b-ap"),
        id: s("sw_ap_design"),
        behaviour: s("method"),
        revision: 1,
        contract_version: 1,
        ..Block::default()
    });
    write(
        &node,
        &drive.join("groups/l3_solar/nodes/sw_ap_design.vnode"),
    );
    // Her second node, signed as it is.
    let mut signed_node = node.clone();
    signed_node.meta.insert(s("block_uid"), s("b-f107"));
    signed_node.blocks[0].uid = s("b-f107");
    signed_node.blocks[0].id = s("sw_f107_observed");
    let row = chain::sign(
        &signed_node,
        "b-f107",
        "Test Nia",
        &nia,
        "2026-03-01",
        "ok",
        "",
    )
    .unwrap();
    signed_node.signatures.push(row);
    write(
        &signed_node,
        &drive.join("groups/l3_solar/nodes/sw_f107_observed.vnode"),
    );
    // Pia's node, as solar's 1.1 release holds it, in a file of its own.
    write(
        &activity_band(),
        &drive.join("groups/l3_solar/nodes/sw_activity_band.vnode"),
    );
    // Two children of it, unassigned, for the breakdown's check.
    write(
        &activity_child("sw_activity_count", &|m| m.to_string()),
        &drive.join("groups/l3_solar/nodes/sw_activity_count.vnode"),
    );
    write(
        &invented(
            activity_child("sw_activity_count_late", &|m| {
                m.replace("if f107 >= 130 then", "if f107 >= 135 then")
            }),
            "Test Pia",
        ),
        &drive.join("groups/l3_solar/nodes/sw_activity_count_late.vnode"),
    );

    // Power, with nothing sealed: its releases folder holds something that is
    // not a release.
    write(
        &signed(group("l3_power", "Test Ola"), "Test Ola", &ola),
        &drive.join("groups/l3_power/l3_power.vgroup"),
    );
    std::fs::create_dir_all(drive.join("groups/l3_power/releases")).unwrap();
    std::fs::write(
        drive.join("groups/l3_power/releases/notes.txt"),
        "not a release",
    )
    .unwrap();

    // Thermal, signed by its node engineer rather than its owner: refused.
    write(
        &signed(group("l3_thermal", "Test Ola"), "Test Nia", &nia),
        &drive.join("groups/l3_thermal/l3_thermal.vgroup"),
    );
    // A folder under groups/ that is not a group.
    std::fs::create_dir_all(drive.join("groups/archive")).unwrap();
    std::fs::write(drive.join("groups/archive/old.txt"), "old").unwrap();

    std::fs::write(
        drive.join("START HERE.md"),
        format!(
            "# START HERE: an invented drive\n\nThe programme manager's key:\n\n    {}\n",
            pat.public().fingerprint()
        ),
    )
    .unwrap();

    std::fs::create_dir_all(&keys).unwrap();
    for (file, person, k) in [
        ("test-pat.vkey", "Test Pat", &pat),
        ("test-sam.vkey", "Test Sam", &sam),
        ("test-ola.vkey", "Test Ola", &ola),
        ("test-nia.vkey", "Test Nia", &nia),
        ("test-stranger.vkey", "Test Stranger", &stranger),
        ("test-ivo.vkey", "Test Ivo", &ivo),
        ("test-pia.vkey", "Test Pia", &pia),
    ] {
        let locked = k
            .lock(person, PASSPHRASE, [7; 16], [7; 12], MIN_ITERATIONS)
            .unwrap();
        let mut f = File::new(Kind::Key, APP);
        f.meta.insert(s("person"), s(person));
        f.locked_keys.push(locked.to_row());
        sqlite::write(&f, &keys.join(file)).unwrap();
    }
    println!("{}", pat.public().fingerprint());
}
