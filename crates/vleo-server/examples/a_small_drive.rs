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
//! - a node engineer whose name is made of what a page must escape;
//! - beside solar, Ola owns power, which has a release sealed, and thermal,
//!   whose file is signed by someone who does not own it, so it is refused;
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
    // Something in solar's releases that is not a release.
    std::fs::create_dir_all(drive.join("groups/l3_solar/releases")).unwrap();
    std::fs::write(
        drive.join("groups/l3_solar/releases/notes.txt"),
        "not a release",
    )
    .unwrap();

    // Power, with a release sealed (its bytes are not read here, only that one is there).
    write(
        &signed(group("l3_power", "Test Ola"), "Test Ola", &ola),
        &drive.join("groups/l3_power/l3_power.vgroup"),
    );
    std::fs::create_dir_all(drive.join("groups/l3_power/releases")).unwrap();
    std::fs::write(
        drive.join("groups/l3_power/releases/l3_power-1.0.vleo"),
        "sealed",
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
