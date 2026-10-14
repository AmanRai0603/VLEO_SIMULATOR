//! `test-drive [--out <dir>]` — the shared drive every browser scenario starts
//! from (docs/PLAN_F.md, decision 5).
//!
//! An empty database folder laid out as docs/OPERATING_1_0.md, section 15, with
//! a programme of invented people, each with a key made from a fixed seed, and
//! three groups taken from design/: their group files, their node files and an
//! empty releases/. Everything is the same bytes every time it is written, so
//! a scenario that passes once passes on any computer, and every scenario starts
//! from a fresh copy of it.
//!
//! **The people are invented, and say so.** Their names start with "Test", the
//! START HERE file says what they are, and no real person's name or key is in
//! it. Each private key is locked by a passphrase written beside it, in keys/
//! and people.toml next to the drive, never on it: a scenario opens a person's
//! key as that person would, and the drive holds only what a real drive holds.
//!
//! **Two sealed releases sit beside the drive, not on it**: Solar 1.1 as the
//! group sealed it, and a 1.2 sealed over the same relations, in
//! `releases/`. A scenario puts on the drive the ones it needs, as a group
//! would, and may change one after sealing to see it refused.
//!
//! **Who owns a group is read the way the library reads it today**
//! (`vleo_files::chain::owner_of`): the programme's file mounts each group on a
//! block and assigns that block to the group's subsystem engineer. The
//! programme's and the systems' files are otherwise design/'s own.

use std::path::{Path, PathBuf};

use vleo_files::chain::{self, FILE};
use vleo_files::keys::{SigningKey, MIN_ITERATIONS};
use vleo_files::meta::Kind;
use vleo_files::model::{Assignment, Block, File, Mount, Person, PersonKey};
use vleo_files::sqlite;
use vleo_files::upgrade::Upgrade;

/// What marks a folder as a test drive this command wrote, so it may write it
/// again; any other folder is refused rather than emptied.
const MARK: &str = "TEST DRIVE";

/// The one moment everything on the drive was registered and signed.
const AT: &str = "2026-10-14T20:00:00Z";
const ON: &str = "2026-10-14";

/// The groups taken from design/, each with the subsystem engineer who owns it
/// and the node engineer assigned its first nodes.
const GROUPS: &[(&str, &str, &str)] = &[
    ("l3_solar", "Test Solar Lead", "Test Solar Node Engineer"),
    (
        "l3_prop",
        "Test Propulsion Lead",
        "Test Propulsion Node Engineer",
    ),
    ("l3_power", "Test Power Lead", "Test Power Node Engineer"),
];

/// How many of each group's nodes its node engineer is assigned.
const ASSIGNED: usize = 3;

/// One invented person: their name, role, whose deputy they are, the group
/// they belong to, and the seed their key is made from.
struct Who {
    name: &'static str,
    role: &'static str,
    deputy_for: &'static str,
    group: &'static str,
    seed: u8,
}

fn people() -> Vec<Who> {
    let mut out = vec![
        Who {
            name: "Test Programme Manager",
            role: "programme manager",
            deputy_for: "",
            group: "programme",
            seed: 1,
        },
        Who {
            name: "Test System Engineer",
            role: "system engineer",
            deputy_for: "",
            group: "systems",
            seed: 2,
        },
        Who {
            name: "Test Solar Deputy",
            role: "subsystem engineer",
            deputy_for: "Test Solar Lead",
            group: "l3_solar",
            seed: 3,
        },
    ];
    for (i, (group, lead, engineer)) in GROUPS.iter().enumerate() {
        out.push(Who {
            name: lead,
            role: "subsystem engineer",
            deputy_for: "",
            group,
            seed: 10 + 2 * i as u8,
        });
        out.push(Who {
            name: engineer,
            role: "node engineer",
            deputy_for: "",
            group,
            seed: 11 + 2 * i as u8,
        });
    }
    out
}

fn key_of(w: &Who) -> SigningKey {
    SigningKey::from_seed([w.seed; 32])
}

/// A person's name as a file name: lower case, words joined by hyphens.
fn slug(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}

fn passphrase(w: &Who) -> String {
    format!("{} passphrase", slug(w.name))
}

fn register(f: &mut File, w: &Who, by: &str) {
    f.people.push(Person {
        name: w.name.into(),
        role: w.role.into(),
        deputy_for: w.deputy_for.into(),
    });
    f.keys.push(PersonKey {
        person: w.name.into(),
        public_key: key_of(w).public().to_hex(),
        registered_at: ON.into(),
        registered_by: by.into(),
        revoked_from: String::new(),
    });
}

fn opened(path: &Path) -> Result<File, String> {
    let how = Upgrade {
        app: concat!("vleo ", env!("CARGO_PKG_VERSION")),
        at: AT,
    };
    sqlite::open(path, &how)
        .map(|o| o.file)
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn write(f: &File, path: &Path) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    sqlite::write(f, path).map_err(|e| format!("{}: {e}", path.display()))
}

fn sign_whole(f: &mut File, w: &Who) -> Result<(), String> {
    let row = chain::sign(f, FILE, w.name, &key_of(w), AT, "ok", "")
        .map_err(|e| format!("{} could not sign: {e}", w.name))?;
    f.signatures.push(row);
    Ok(())
}

/// Solar's release `from`, as the next version `version` over the same
/// relations: its version and its de-risking record say so, and its seal is
/// made again over what it now holds, as the group application seals.
fn sealed_again(from: &Path, to: &Path, version: &str) -> Result<(), String> {
    std::fs::copy(from, to).map_err(|e| format!("{}: {e}", from.display()))?;
    let sql = |statement: &str, params: &[&str]| -> Result<(), String> {
        let db = rusqlite::Connection::open(to).map_err(|e| format!("{}: {e}", to.display()))?;
        let n = db
            .execute(statement, rusqlite::params_from_iter(params))
            .map_err(|e| format!("{}: {e}", to.display()))?;
        if n == 0 {
            return Err(format!(
                "{}: nothing to change for {statement}",
                to.display()
            ));
        }
        Ok(())
    };
    sql(
        "UPDATE meta SET value = ?1 WHERE key = 'version'",
        &[version],
    )?;
    sql(
        "UPDATE tbl SET csv = csv || ?1 WHERE scope = 'group' AND path = 'versions.csv'",
        &[&format!(
            "{version},{ON},Test Solar Lead,The release is sealed again for the tests.,\
             Nothing about the relations was tested.,Nothing new is known.,\
             Nothing changed but the version.,,,the relations of 1.1,a change to them\n"
        )],
    )?;
    let r = vleo_files::intake::Release::open(to)?;
    let fingerprint = vleo_files::seal::fingerprint_of(&r.folder, "group");
    sql(
        "UPDATE meta SET value = ?1 WHERE key = 'fingerprint'",
        &[&fingerprint],
    )?;
    vleo_files::intake::Release::open(to)?
        .check_seal()
        .map_err(|e| format!("{}: sealed again, it does not check: {e}", to.display()))?;
    Ok(())
}

/// `test-drive [--out <dir>]`.
pub fn cmd_test_drive(root: &Path, args: &[&str]) -> Result<(), String> {
    let out = args
        .iter()
        .position(|a| *a == "--out")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target/test-drive"));
    let written = write_test_drive(root, &out)?;
    println!(
        "test-drive: {} — {} people, {} groups from design/, anchor {}",
        out.display(),
        written.people,
        GROUPS.len(),
        written.anchor
    );
    Ok(())
}

/// What a test drive was written with.
pub struct Written {
    pub people: usize,
    pub anchor: String,
}

/// Write the test drive into `out`: `out/drive/` the shared drive, `out/keys/`
/// each person's locked key, `out/people.toml` who they are and how to open
/// each key. A folder that is not a test drive is refused, never emptied.
pub fn write_test_drive(root: &Path, out: &Path) -> Result<Written, String> {
    if out.exists() {
        if !out.join(MARK).is_file() {
            return Err(format!(
                "{} is there and is not a test drive: name an empty or new folder",
                out.display()
            ));
        }
        std::fs::remove_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    }
    let drive = out.join("drive");
    for dir in [
        "apps",
        "guides",
        "design",
        "daily",
        "integration",
        "issues",
        "readable",
        "cases",
    ] {
        std::fs::create_dir_all(drive.join(dir)).map_err(|e| format!("{dir}: {e}"))?;
    }
    let people = people();
    let pm = &people[0];
    let anchor = key_of(pm).public().fingerprint();
    let design = root.join("design/groups");

    // The programme's file: design's own, with the people it registers and
    // the group each subsystem engineer owns, signed by its programme manager.
    let mut programme = opened(&design.join("programme/programme.vgroup"))?;
    for w in people.iter().filter(|w| w.role != "node engineer") {
        register(&mut programme, w, pm.name);
    }
    for (group, lead, _) in GROUPS {
        let uid = format!("test-owner-{group}");
        programme.blocks.push(Block {
            uid: uid.clone(),
            id: uid.clone(),
            behaviour: "children".into(),
            question: format!("Which subsystem engineer owns {group}? (test drive)"),
            ..Block::default()
        });
        programme.mounts.push(Mount {
            block_uid: uid.clone(),
            group_id: (*group).into(),
            release: String::new(),
        });
        programme.assignments.push(Assignment {
            block_uid: uid,
            person: (*lead).into(),
            contract_version: 1,
        });
    }
    sign_whole(&mut programme, pm)?;
    write(&programme, &drive.join("groups/programme/programme.vgroup"))?;

    // The systems' file, as design/ holds it, signed by its system engineer.
    let se = &people[1];
    let mut systems = opened(&design.join("systems/systems.vgroup"))?;
    sign_whole(&mut systems, se)?;
    write(&systems, &drive.join("groups/systems/systems.vgroup"))?;
    for g in ["programme", "systems"] {
        std::fs::create_dir_all(drive.join("groups").join(g).join("releases"))
            .map_err(|e| format!("{g}: {e}"))?;
    }

    // Each group: its file, registering its node engineer and assigning them
    // its first nodes, signed by its subsystem engineer; its node files as
    // design/ holds them, signed by nobody yet.
    for (group, lead, engineer) in GROUPS {
        let lead = people.iter().find(|w| w.name == *lead).expect("a lead");
        let ne = people
            .iter()
            .find(|w| w.name == *engineer)
            .expect("a node engineer");
        let from = design.join(group);
        let mut f = opened(&from.join(format!("{group}.vgroup")))?;
        register(&mut f, ne, lead.name);
        let mut nodes: Vec<PathBuf> = std::fs::read_dir(from.join("nodes"))
            .map_err(|e| format!("{group}: {e}"))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "vnode"))
            .collect();
        nodes.sort();
        let to = drive.join("groups").join(group);
        for (i, path) in nodes.iter().enumerate() {
            let node = opened(path)?;
            if i < ASSIGNED {
                let uid = node.meta.get("block_uid").cloned().unwrap_or_default();
                f.assignments.push(Assignment {
                    block_uid: uid,
                    person: ne.name.into(),
                    contract_version: 1,
                });
            }
            let name = path.file_name().expect("a node file has a name");
            write(&node, &to.join("nodes").join(name))?;
        }
        sign_whole(&mut f, lead)?;
        write(&f, &to.join(format!("{group}.vgroup")))?;
        std::fs::create_dir_all(to.join("releases")).map_err(|e| format!("{group}: {e}"))?;
    }

    // Each person's key, locked by their passphrase, beside the drive.
    let mut listed = String::from(
        "# The invented people of the test drive, written by `cargo run -p xtask -- test-drive`.\n\
         # No real person's name or key is here. Each key is made from the seed\n\
         # given, and locked in keys/<file> by the passphrase given.\n",
    );
    for w in &people {
        let file = format!("{}.vkey", slug(w.name));
        let locked = key_of(w)
            .lock(
                w.name,
                &passphrase(w),
                [w.seed; 16],
                [w.seed; 12],
                MIN_ITERATIONS,
            )
            .map_err(|e| format!("{}: {e}", w.name))?;
        let mut k = File::new(Kind::Key, concat!("vleo ", env!("CARGO_PKG_VERSION")));
        k.meta.insert("person".into(), w.name.into());
        k.locked_keys.push(locked.to_row());
        write(&k, &out.join("keys").join(&file))?;
        listed.push_str(&format!(
            "\n[[person]]\nname = \"{}\"\nrole = \"{}\"\ndeputy_for = \"{}\"\ngroup = \"{}\"\nseed = {}\nkey = \"keys/{file}\"\npassphrase = \"{}\"\npublic_key = \"{}\"\n",
            w.name,
            w.role,
            w.deputy_for,
            w.group,
            w.seed,
            passphrase(w),
            key_of(w).public().to_hex()
        ));
    }
    std::fs::write(out.join("people.toml"), listed).map_err(|e| format!("people.toml: {e}"))?;

    // Solar's releases, sealed, for a scenario to put on the drive.
    let fixture = root.join("crates/vleo-files/tests/fixtures/l3_solar-1.1.vleo");
    let releases = out.join("releases");
    std::fs::create_dir_all(&releases).map_err(|e| format!("releases: {e}"))?;
    std::fs::copy(&fixture, releases.join("l3_solar-1.1.vleo"))
        .map_err(|e| format!("{}: {e}", fixture.display()))?;
    sealed_again(&fixture, &releases.join("l3_solar-1.2.vleo"), "1.2")?;

    let start = format!(
        "# START HERE — a test drive\n\n\
         This drive is for the tool's own tests. Everyone on it is invented, and\n\
         every key is made from a fixed seed: nothing here belongs to a person.\n\n\
         The programme manager's key fingerprint, the anchor every signature\n\
         checks through:\n\n    {anchor}\n\n\
         Written by `cargo run -p xtask -- test-drive`. The people and how to\n\
         open each one's key are in ../people.toml.\n"
    );
    std::fs::write(drive.join("START HERE.md"), start).map_err(|e| format!("START HERE: {e}"))?;
    std::fs::write(
        out.join(MARK),
        "Written by `cargo run -p xtask -- test-drive`; written again whole each time.\n",
    )
    .map_err(|e| format!("{MARK}: {e}"))?;
    Ok(Written {
        people: people.len(),
        anchor,
    })
}
