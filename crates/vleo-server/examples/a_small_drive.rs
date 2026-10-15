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
//! - Test Nia, solar's node engineer, with one node assigned and unsigned;
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
    mount(&mut systems, "sys_propulsion", "l3_prop", "Test Ivo");
    write(
        &signed(systems, "Test Sam", &sam),
        &drive.join("groups/systems/systems.vgroup"),
    );

    let mut solar = group("l3_solar", "Test Ola");
    register(&mut solar, "Test Nia", "node engineer", &nia, "Test Ola");
    solar.assignments.push(Assignment {
        block_uid: s("b-ap"),
        person: s("Test Nia"),
        contract_version: 1,
    });
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
