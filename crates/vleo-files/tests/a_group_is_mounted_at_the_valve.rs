//! A group is mounted, and given what it is allocated, in the systems' file
//! by the system engineer, and started by the owner the mount names
//! (docs/briefs/F6.md, F6-a; W1, W13).
//!
//! A small programme of invented people: Pat the programme manager, Sam the
//! system engineer and Sid their deputy, Ola who owns solar and Dee their
//! deputy, Rui a subsystem engineer who owns nothing yet and Uma their
//! deputy. The systems' file
//! holds solar's mount and a leaf, `sys_radiation`, framed with the output
//! the new group answers; solar's group file holds a leaf of its own,
//! `sw_activity_count`. Nothing here is computed: a mount, an allocation and
//! a new file are rows, and every expected value is written in the test.
//!
//! - W1: Sam mounts `l3_radiation` on `sys_radiation` for Rui, with a change
//!   row saying why, and the systems' file opens through the chain with Rui
//!   its owner; Sid, Sam's deputy, may mount too; Rui and Ola may not;
//! - each mount's refusal: an id that is not one, `programme` or `systems`,
//!   a group mounted already or with a group file already, a block no branch
//!   holds, one with children, broken down, assigned, archived or in the
//!   programme's branch, and an owner who is not registered as a subsystem
//!   engineer or the system engineer, or is someone's deputy;
//! - an allocation on the mount, with its closure and its sense, and its
//!   refusals: no sense, a sense that is not one, an output the block does
//!   not have, a block with no mount, no value and no range, one end only;
//!   and on any group file but the systems';
//! - W13: Sam mounts `l3_activity` on solar's `sw_activity_count` for Rui;
//!   the valve above it is Ola, and solar's file is not touched; Ola's
//!   edits on that leaf are refused, naming the mount, and a port filled as
//!   allocated is refused anywhere, for it says no sense;
//! - in the copy: Rui starts `l3_radiation` once it is mounted and
//!   allocated, its file one heading whose question is the mount's, signed
//!   by Rui, opening through the chain; Uma, Rui's deputy, starts it for
//!   Rui; Sam, Sid and Ola may not; a group nobody mounts, one started
//!   already, one the systems' file allocates nothing (F6's question 43), a
//!   folder holding something, and a systems' file that does not open are
//!   refused, with nothing written; a group on solar's leaf starts with no
//!   allocation, since none can be given there yet; a refused mount or
//!   allocation leaves the systems' file's bytes as they were.

use vleo_files::breakdown::{self, Edit, Part};
use vleo_files::chain::{self, Registry, FILE};
use vleo_files::frame::{self, Valve};
use vleo_files::keys::SigningKey;
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::start;

const APP: &str = "vleo 1.0.0 (test)";
const AT: &str = "2026-10-17T09:00:00Z";
const WHY: &str = "Radiation is a subsystem's question now: its dose is shielded by mass.";

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

fn block(uid: &str, parent: &str, behaviour: &str, question: &str) -> Block {
    Block {
        uid: s(uid),
        id: s(uid),
        parent_uid: s(parent),
        behaviour: s(behaviour),
        question: s(question),
        contract_version: 1,
        ..Block::default()
    }
}

fn mount(f: &mut File, uid: &str, group: &str, owner: &str) {
    f.blocks.push(block(uid, "", "children", ""));
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

fn out_port(uid: &str, name: &str, unit: &str) -> Port {
    Port {
        block_uid: s(uid),
        direction: s("out"),
        name: s(name),
        port_type: s("number"),
        unit: s(unit),
        state: s("achieved"),
        ..Port::default()
    }
}

fn group_file(id: &str, writer: &str) -> File {
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

// Pat (11), Sam (12), Ola (13), Dee (14, Ola's deputy), Rui (15), Sid (17,
// Sam's deputy), Uma (18, Rui's deputy).
fn programme() -> Registry {
    let mut f = group_file("programme", "Pat Morgan");
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
    register(&mut f, "Rui Ray", "subsystem engineer", "", &key(15));
    register(&mut f, "Sid Side", "system engineer", "Sam Engel", &key(17));
    register(
        &mut f,
        "Uma Under",
        "subsystem engineer",
        "Rui Ray",
        &key(18),
    );
    mount(&mut f, "root", "systems", "Sam Engel");
    chain::open_programme(
        signed(f, "Pat Morgan", &key(11)),
        &key(11).public().fingerprint(),
    )
    .unwrap()
}

/// The systems' file: solar's mount, the leaf radiation is mounted on, and
/// the blocks each refusal is about.
fn systems_file() -> File {
    let mut f = group_file("systems", "Sam Engel");
    mount(&mut f, "sys_space_environment", "l3_solar", "Ola Sun");
    f.blocks.push(block(
        "sys_radiation",
        "",
        "open",
        "What dose does the payload take over the mission?",
    ));
    f.ports.push(out_port("sys_radiation", "shield_mass", "kg"));
    // A block with a child.
    f.blocks
        .push(block("sys_power", "", "children", "What power?"));
    f.blocks
        .push(block("sys_power_array", "sys_power", "open", "The array?"));
    // A block broken down whose child is archived: its wire still answers it.
    f.blocks
        .push(block("sys_thermal", "", "children", "What heat?"));
    f.ports.push(out_port("sys_thermal", "rejected", "W"));
    let mut gone = block("sys_thermal_old", "sys_thermal", "open", "Old.");
    gone.archived = 1;
    f.blocks.push(gone);
    f.wires.push(Wire {
        to_block: s("sys_thermal"),
        to_port: s("rejected"),
        from_ref: s("sys_thermal_old.rejected"),
    });
    // A block someone writes, and one removed.
    f.blocks.push(block("sys_comms", "", "open", "What link?"));
    f.assignments.push(Assignment {
        block_uid: s("sys_comms"),
        person: s("Sam Engel"),
        contract_version: 1,
    });
    let mut removed = block("sys_old", "", "open", "Gone.");
    removed.archived = 1;
    f.blocks.push(removed);
    signed(f, "Sam Engel", &key(12))
}

fn systems(pr: &Registry) -> Registry {
    chain::open_systems(&systems_file(), pr).unwrap()
}

/// Solar's group file: its heading and a leaf nobody writes.
fn solar() -> File {
    let mut f = group_file("l3_solar", "Ola Sun");
    f.blocks.push(block(
        "l3_solar",
        "",
        "children",
        "What solar weather does the design face?",
    ));
    f.blocks.push(block(
        "sw_activity_count",
        "l3_solar",
        "open",
        "How many active days a year?",
    ));
    f.ports
        .push(out_port("sw_activity_count", "active_days", ""));
    signed(f, "Ola Sun", &key(13))
}

/// The systems' file with `e` made by `who`, the design's other files beside it.
fn frame_it(systems: &File, e: &Edit, who: &str, k: &SigningKey) -> Result<File, String> {
    let pr = programme();
    let elsewhere = vec![pr.file.clone(), solar()];
    breakdown::edit(systems, &[], &elsewhere, e, WHY, who, k, AT, &pr, &pr)
        .map(|(f, _)| f)
        .map_err(|e| e.message().to_string())
}

fn mount_edit(block: &str, group: &str, owner: &str) -> Edit {
    Edit::Mount {
        block: s(block),
        group: s(group),
        owner: s(owner),
    }
}

fn radiation_mounted() -> File {
    let f = frame_it(
        &systems_file(),
        &mount_edit("sys_radiation", "l3_radiation", "Rui Ray"),
        "Sam Engel",
        &key(12),
    )
    .expect("Sam mounts radiation for Rui");
    assert_eq!(
        f.meta.get("signed_rows").map(String::as_str),
        Some("issue change"),
        "its rows are signed with it (F6-d)"
    );
    f
}

#[test]
fn the_system_engineer_mounts_a_group_on_a_leaf_for_its_owner() {
    let f = radiation_mounted();
    let m = f
        .mounts
        .iter()
        .find(|m| m.group_id == "l3_radiation")
        .expect("the mount is written");
    assert_eq!(m.block_uid, "sys_radiation");
    assert!(f
        .assignments
        .iter()
        .any(|a| a.block_uid == "sys_radiation" && a.person == "Rui Ray"));
    let b = f.blocks.iter().find(|b| b.uid == "sys_radiation").unwrap();
    assert_eq!(
        b.behaviour, "children",
        "a mounted leaf is answered by its group"
    );
    let c = f.changes.last().unwrap();
    assert_eq!(
        (c.who.as_str(), c.scope.as_str()),
        ("Sam Engel", "sys_radiation")
    );
    assert!(
        c.what.starts_with("mount: ") && c.what.contains(WHY),
        "{}",
        c.what
    );
    assert!(
        c.after.as_deref().is_some_and(|a| a
            .contains("mount \"l3_radiation\" on \"sys_radiation\"")
            && a.contains("assignment \"sys_radiation\" to \"Rui Ray\"")),
        "{:?}",
        c.after
    );
    let sig = f.signatures.iter().find(|r| r.scope == FILE).unwrap();
    assert_eq!(sig.signer, "Sam Engel");
    // Opened again through the chain, Rui owns it.
    let pr = programme();
    let sy = chain::open_systems(&f, &pr).expect("the systems' file still opens");
    let (_, owner) = chain::mounted_by("l3_radiation", &[&pr, &sy]).unwrap();
    assert_eq!(owner, "Rui Ray");
    // Sid, Sam's deputy, mounts as Sam may.
    frame_it(
        &systems_file(),
        &mount_edit("sys_radiation", "l3_radiation", "Rui Ray"),
        "Sid Side",
        &key(17),
    )
    .expect("Sam's deputy mounts");
}

#[test]
fn only_the_system_engineer_or_their_deputy_mounts() {
    for (who, k) in [("Rui Ray", 15), ("Ola Sun", 13)] {
        let e = frame_it(
            &systems_file(),
            &mount_edit("sys_radiation", "l3_radiation", "Rui Ray"),
            who,
            &key(k),
        )
        .unwrap_err();
        assert!(
            e.contains("may not change systems's group file"),
            "{who}: {e}"
        );
    }
    // In solar's own file, Ola mounts nothing: a group is mounted in the
    // systems' file.
    let pr = programme();
    let sy = systems(&pr);
    let e = breakdown::edit(
        &solar(),
        &[],
        &[systems_file()],
        &mount_edit("sw_activity_count", "l3_activity", "Rui Ray"),
        WHY,
        "Ola Sun",
        &key(13),
        AT,
        &sy,
        &pr,
    )
    .unwrap_err();
    assert!(
        e.message().contains(
            "a group is mounted, and given what it is allocated, in the systems' file by the system engineer"
        ),
        "{}",
        e.message()
    );
}

#[test]
fn a_mount_is_refused_by_name_with_nothing_written() {
    let mut with_file = vec![programme().file.clone(), solar()];
    with_file.push(group_file("l3_radiation", "Rui Ray"));
    let pr = programme();
    let e = breakdown::edit(
        &systems_file(),
        &[],
        &with_file,
        &mount_edit("sys_radiation", "l3_radiation", "Rui Ray"),
        WHY,
        "Sam Engel",
        &key(12),
        AT,
        &pr,
        &pr,
    )
    .unwrap_err();
    assert!(
        e.message().contains("holds a group file already"),
        "{}",
        e.message()
    );
    for (block, group, owner, says) in [
        (
            "sys_radiation",
            "L3 Radiation",
            "Rui Ray",
            "is not a group's id",
        ),
        (
            "sys_radiation",
            "systems",
            "Rui Ray",
            "is the systems's own branch",
        ),
        (
            "sys_radiation",
            "programme",
            "Rui Ray",
            "is the programme's own branch",
        ),
        (
            "sys_radiation",
            "l3_solar",
            "Rui Ray",
            "l3_solar is mounted already",
        ),
        (
            "sys_nothing",
            "l3_radiation",
            "Rui Ray",
            "no branch holds a block \"sys_nothing\"",
        ),
        (
            "sys_power",
            "l3_radiation",
            "Rui Ray",
            "sys_power holds sys_power_array",
        ),
        (
            "sys_thermal",
            "l3_radiation",
            "Rui Ray",
            "sys_thermal is broken down",
        ),
        (
            "sys_comms",
            "l3_radiation",
            "Rui Ray",
            "sys_comms is assigned to Sam Engel",
        ),
        ("sys_old", "l3_radiation", "Rui Ray", "sys_old is removed"),
        (
            "root",
            "l3_radiation",
            "Rui Ray",
            "root is in the programme's branch",
        ),
        (
            "sys_space_environment",
            "l3_radiation",
            "Rui Ray",
            "carries a mount already",
        ),
        (
            "sys_radiation",
            "l3_radiation",
            "Zed Nobody",
            "Zed Nobody is not registered",
        ),
        (
            "sys_radiation",
            "l3_radiation",
            "Pat Morgan",
            "Pat Morgan is not registered",
        ),
        (
            "sys_radiation",
            "l3_radiation",
            "Dee Puty",
            "Dee Puty is Ola Sun's deputy",
        ),
        (
            "sys_radiation",
            "l3_radiation",
            "Sid Side",
            "Sid Side is Sam Engel's deputy",
        ),
        (
            "l3_solar",
            "l3_radiation",
            "Rui Ray",
            "l3_solar is the heading of l3_solar's branch",
        ),
        (
            "sys_radiation",
            "sys_power",
            "Rui Ray",
            "sys_power is a block's uid in the design already",
        ),
        (
            "sys_radiation",
            "l3_radiation",
            "Claude",
            "Claude is an assistant's name",
        ),
    ] {
        let e = frame_it(
            &systems_file(),
            &mount_edit(block, group, owner),
            "Sam Engel",
            &key(12),
        )
        .unwrap_err();
        assert!(e.contains(says), "{block} {group} {owner}: {e}");
    }
    // A leaf of the systems' own branch, mounted for the system engineer.
    frame_it(
        &systems_file(),
        &mount_edit("sys_radiation", "l3_radiation", "Sam Engel"),
        "Sam Engel",
        &key(12),
    )
    .expect("the system engineer may own a group of their own");
}

fn allocation(name: &str, value: &str, lower: &str, upper: &str) -> Box<Port> {
    Box::new(Port {
        name: s(name),
        port_type: s("number"),
        unit: s("kg"),
        value: s(value),
        lower: s(lower),
        upper: s(upper),
        ..Port::default()
    })
}

fn allocate(port: Box<Port>, bounds: &str, sense: &str) -> Edit {
    Edit::Allocate {
        block: s("sys_radiation"),
        port,
        bounds: s(bounds),
        sense: s(sense),
        says: s("the shield stays within the mass it is given"),
    }
}

#[test]
fn a_mount_is_allocated_with_the_sense_it_binds() {
    let f = frame_it(
        &radiation_mounted(),
        &allocate(
            allocation("shield_mass_allocated", "12", "", ""),
            "shield_mass",
            "<=",
        ),
        "Sam Engel",
        &key(12),
    )
    .expect("Sam allocates");
    let p = f
        .ports
        .iter()
        .find(|p| p.name == "shield_mass_allocated")
        .unwrap();
    assert_eq!(
        (
            p.block_uid.as_str(),
            p.direction.as_str(),
            p.state.as_str(),
            p.parameter.as_str(),
            p.value.as_str()
        ),
        ("sys_radiation", "in", "allocated", "system", "12")
    );
    let c = f
        .closures
        .iter()
        .find(|c| c.uid == "sys_radiation.shield_mass_allocated")
        .unwrap();
    assert_eq!(
        (
            c.required_ref.as_str(),
            c.achieved_ref.as_str(),
            c.sense.as_str()
        ),
        (
            "sys_radiation.shield_mass_allocated",
            "sys_radiation.shield_mass",
            "<="
        )
    );
    assert!(f.changes.last().unwrap().what.starts_with("allocate: "));
    chain::open_systems(&f, &programme()).expect("it opens");
    // A range, both ends, reaching it.
    let f = frame_it(
        &radiation_mounted(),
        &allocate(
            allocation("shield_mass_band", "", "8", "12"),
            "shield_mass",
            ">=",
        ),
        "Sam Engel",
        &key(12),
    )
    .expect("a range is allocated");
    assert_eq!(f.closures.last().unwrap().sense, ">=");
}

#[test]
fn an_allocation_is_refused_by_name() {
    let mounted = radiation_mounted();
    for (port, bounds, sense, on, says) in [
        (
            allocation("a", "", "8", ""),
            "shield_mass",
            "<=",
            &mounted,
            "gives one end of a range",
        ),
        (
            allocation("a", "12", "8", "14"),
            "shield_mass",
            "<=",
            &mounted,
            "gives a value and a range: it is one or the other",
        ),
        (
            allocation("a", "twelve", "", ""),
            "shield_mass",
            "<=",
            &mounted,
            "has the value \"twelve\", which is not a number",
        ),
        (
            allocation("a", "", "abc", "14"),
            "shield_mass",
            "<=",
            &mounted,
            "has the lower end \"abc\", which is not a number",
        ),
        (
            allocation("a", "", "14", "8"),
            "shield_mass",
            "<=",
            &mounted,
            "its lower end is above its upper",
        ),
        (
            allocation("a", "12", "", ""),
            "shield_mass",
            "",
            &mounted,
            "say which way a binds shield_mass",
        ),
        (
            allocation("a", "12", "", ""),
            "shield_mass",
            "=",
            &mounted,
            "\"=\" is not a sense",
        ),
        (
            allocation("a", "12", "", ""),
            "shield_area",
            "<=",
            &mounted,
            "has no output \"shield_area\"",
        ),
        (
            allocation("a", "", "", ""),
            "shield_mass",
            "<=",
            &mounted,
            "gives no value and no range",
        ),
        (
            allocation("a", "12", "8", ""),
            "shield_mass",
            "<=",
            &mounted,
            "gives one end of a range",
        ),
        (
            allocation("shield_mass", "12", "", ""),
            "shield_mass",
            "<=",
            &mounted,
            "has a port shield_mass already",
        ),
        (
            allocation("a", "12", "", ""),
            "shield_mass",
            "<=",
            &systems_file(),
            "no group is mounted on sys_radiation",
        ),
    ] {
        let e = frame_it(on, &allocate(port, bounds, sense), "Sam Engel", &key(12)).unwrap_err();
        assert!(e.contains(says), "{e}");
    }
}

#[test]
fn a_group_on_a_subsystem_s_leaf_has_its_owner_above_it() {
    let f = frame_it(
        &systems_file(),
        &mount_edit("sw_activity_count", "l3_activity", "Rui Ray"),
        "Sam Engel",
        &key(12),
    )
    .expect("Sam mounts on solar's leaf");
    let pr = programme();
    let elsewhere = vec![pr.file.clone(), solar()];
    assert_eq!(
        frame::valve_above("l3_activity", &f, &[], &elsewhere, &pr),
        Some(Valve {
            branch: s("l3_solar"),
            owner: s("Ola Sun"),
        })
    );
    // A group on the systems' own leaf has the system engineer above it.
    let f = frame_it(
        &f,
        &mount_edit("sys_radiation", "l3_radiation", "Rui Ray"),
        "Sam Engel",
        &key(12),
    )
    .unwrap();
    assert_eq!(
        frame::valve_above("l3_radiation", &f, &[], &elsewhere, &pr),
        Some(Valve {
            branch: s("systems"),
            owner: s("Sam Engel"),
        })
    );
    // Solar's file is not written: the mount is the systems' file's row.
    assert!(f.blocks.iter().all(|b| b.uid != "sw_activity_count"));
}
/// A copy holding the programme's, the systems' and solar's files, as the
/// application lays them out.
fn a_copy(name: &str, systems: &File) -> std::path::PathBuf {
    let copy = std::env::temp_dir().join(format!("vleo-valve-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&copy);
    for (id, f) in [
        ("programme", programme().file.clone()),
        ("systems", systems.clone()),
        ("l3_solar", solar()),
    ] {
        let dir = copy.join("groups").join(id);
        std::fs::create_dir_all(&dir).unwrap();
        vleo_files::sqlite::write(&f, &dir.join(format!("{id}.vgroup"))).unwrap();
    }
    copy
}

fn digest(p: &std::path::Path) -> String {
    vleo_files::copied::digest(&std::fs::read(p).unwrap())
}

/// `e` made on the systems' file in `copy` by `who`.
fn frame_in(copy: &std::path::Path, e: &Edit, who: &str, k: &SigningKey) -> Result<String, String> {
    let pr = programme();
    let at = copy.join("groups/systems/systems.vgroup");
    match breakdown::edit_in_place(
        copy,
        "systems",
        &digest(&at),
        e,
        WHY,
        who,
        k,
        AT,
        APP,
        &pr,
        &pr,
    )
    .map_err(|e| e.message().to_string())?
    {
        breakdown::Framed::Written { digest, .. } => Ok(digest),
        other => Err(format!("{other:?}")),
    }
}

fn start_in(
    copy: &std::path::Path,
    group: &str,
    who: &str,
    k: &SigningKey,
) -> Result<String, String> {
    start::start_in_place(copy, group, who, k, AT, APP, &programme())
        .map_err(|e| e.message().to_string())
}

#[test]
fn the_owner_a_mount_names_starts_its_group_and_it_opens_through_the_chain() {
    let copy = a_copy("w1", &systems_file());
    frame_in(
        &copy,
        &mount_edit("sys_radiation", "l3_radiation", "Rui Ray"),
        "Sam Engel",
        &key(12),
    )
    .expect("Sam mounts radiation for Rui");
    frame_in(
        &copy,
        &allocate(
            allocation("shield_mass_allocated", "12", "", ""),
            "shield_mass",
            "<=",
        ),
        "Sam Engel",
        &key(12),
    )
    .expect("and allocates it its shield's mass");
    let made = start_in(&copy, "l3_radiation", "Rui Ray", &key(15)).expect("Rui starts it");
    let at = copy.join("groups/l3_radiation/l3_radiation.vgroup");
    assert_eq!(made, digest(&at));
    let f = vleo_files::sqlite::read(&at).unwrap();
    assert_eq!(f.kind().unwrap(), Kind::Group);
    assert_eq!(
        f.meta.get("signed_rows").map(String::as_str),
        Some("issue change"),
        "its rows are signed with it (F6-d)"
    );
    assert_eq!(
        f.meta.get("group_id").map(String::as_str),
        Some("l3_radiation")
    );
    assert_eq!(f.meta.get("writer").map(String::as_str), Some("Rui Ray"));
    assert_eq!(f.blocks.len(), 1, "one heading");
    let h = &f.blocks[0];
    assert_eq!(
        (h.uid.as_str(), h.id.as_str(), h.question.as_str()),
        (
            "l3_radiation",
            "l3_radiation",
            "What dose does the payload take over the mission?"
        )
    );
    assert!(f.ports.is_empty(), "the mount's ports are not copied");
    let c = f.changes.last().unwrap();
    assert!(
        c.what.starts_with("start: ") && c.who == "Rui Ray",
        "{}",
        c.what
    );
    let sig = f.signatures.iter().find(|r| r.scope == FILE).unwrap();
    assert_eq!(sig.signer, "Rui Ray");
    // It opens through the chain, through the systems' file that mounts it.
    let pr = programme();
    let sy = chain::open_systems(
        &vleo_files::sqlite::read(&copy.join("groups/systems/systems.vgroup")).unwrap(),
        &pr,
    )
    .unwrap();
    chain::open_group_in(&f, &sy, &pr).expect("the new group opens through the chain");
    // Started once: a second start is refused, and nothing is written.
    let e = start_in(&copy, "l3_radiation", "Rui Ray", &key(15)).unwrap_err();
    assert!(e.contains("l3_radiation is started already"), "{e}");
    assert_eq!(digest(&at), made);
    let _ = std::fs::remove_dir_all(&copy);
}

/// Radiation mounted for Rui in `copy`, and allocated its shield's mass.
fn radiation_in(copy: &std::path::Path) {
    frame_in(
        copy,
        &mount_edit("sys_radiation", "l3_radiation", "Rui Ray"),
        "Sam Engel",
        &key(12),
    )
    .unwrap();
    frame_in(
        copy,
        &allocate(
            allocation("shield_mass_allocated", "12", "", ""),
            "shield_mass",
            "<=",
        ),
        "Sam Engel",
        &key(12),
    )
    .unwrap();
}

#[test]
fn a_deputy_starts_for_the_owner_and_nobody_else_does() {
    let copy = a_copy("deputy", &systems_file());
    radiation_in(&copy);
    for (who, k) in [("Sam Engel", 12), ("Ola Sun", 13), ("Sid Side", 17)] {
        let e = start_in(&copy, "l3_radiation", who, &key(k)).unwrap_err();
        assert!(
            e.contains("may not change l3_radiation's group file")
                && e.contains("its owner Rui Ray"),
            "{who}: {e}"
        );
    }
    assert!(
        !copy.join("groups/l3_radiation").exists(),
        "a refused start writes nothing"
    );
    start_in(&copy, "l3_radiation", "Uma Under", &key(18)).expect("Rui's deputy starts it");
    let f =
        vleo_files::sqlite::read(&copy.join("groups/l3_radiation/l3_radiation.vgroup")).unwrap();
    assert_eq!(f.meta.get("writer").map(String::as_str), Some("Rui Ray"));
    assert_eq!(f.signatures[0].signer, "Uma Under");
    let _ = std::fs::remove_dir_all(&copy);
}

#[test]
fn a_start_is_refused_by_name_with_nothing_written() {
    let copy = a_copy("refused", &systems_file());
    let e = start_in(&copy, "l3_radiation", "Rui Ray", &key(15)).unwrap_err();
    assert!(
        e.contains("the systems' file mounts no group l3_radiation"),
        "{e}"
    );
    for (bad, says) in [
        ("systems", "is not a subsystem's group"),
        ("programme", "is not a subsystem's group"),
        ("L3", "is not a group's id"),
    ] {
        let e = start_in(&copy, bad, "Rui Ray", &key(15)).unwrap_err();
        assert!(e.contains(says), "{bad}: {e}");
    }
    frame_in(
        &copy,
        &mount_edit("sys_radiation", "l3_radiation", "Rui Ray"),
        "Sam Engel",
        &key(12),
    )
    .unwrap();
    // Mounted on the systems' own leaf and allocated nothing: the system
    // engineer gives it what it must reach or may use first (F6's question
    // 43).
    let e = start_in(&copy, "l3_radiation", "Rui Ray", &key(15)).unwrap_err();
    assert!(
        e.contains("the systems' file allocates l3_radiation nothing yet")
            && e.contains("on sys_radiation"),
        "{e}"
    );
    assert!(!copy.join("groups/l3_radiation").exists());
    frame_in(
        &copy,
        &allocate(
            allocation("shield_mass_allocated", "12", "", ""),
            "shield_mass",
            "<=",
        ),
        "Sam Engel",
        &key(12),
    )
    .unwrap();
    // A folder that holds something already.
    std::fs::create_dir_all(copy.join("groups/l3_radiation")).unwrap();
    std::fs::write(copy.join("groups/l3_radiation/notes.txt"), "kept").unwrap();
    let e = start_in(&copy, "l3_radiation", "Rui Ray", &key(15)).unwrap_err();
    assert!(
        e.contains("groups/l3_radiation/ holds notes.txt already"),
        "{e}"
    );
    assert!(!copy
        .join("groups/l3_radiation/l3_radiation.vgroup")
        .exists());
    std::fs::remove_file(copy.join("groups/l3_radiation/notes.txt")).unwrap();
    // A systems' file that does not open: changed since Sam signed it.
    let at = copy.join("groups/systems/systems.vgroup");
    let mut sf = vleo_files::sqlite::read(&at).unwrap();
    sf.blocks[0].question = s("changed by hand");
    vleo_files::sqlite::write(&sf, &at).unwrap();
    let e = start_in(&copy, "l3_radiation", "Rui Ray", &key(15)).unwrap_err();
    assert!(
        e.contains("the systems' file does not open through the chain"),
        "{e}"
    );
    assert!(!copy
        .join("groups/l3_radiation/l3_radiation.vgroup")
        .exists());
    let _ = std::fs::remove_dir_all(&copy);
}

#[test]
fn an_allocation_counts_only_as_allocate_gives_it_on_the_mount_s_own_block() {
    // Radiation, mounted on sys_radiation and given its shield's mass.
    let f = radiation_allocated();
    assert_eq!(start::allocation_wanted(&f, "l3_radiation"), None);
    let refused = |f: &File, g: &str| {
        start::allocation_wanted(f, g)
            .is_some_and(|why| why.contains(&format!("allocates {g} nothing yet")))
    };
    // Another group mounted on another of the systems' leaves: radiation's
    // allocation is not its own.
    let mut other = f.clone();
    other
        .blocks
        .push(block("sys_optics", "", "open", "What do the optics weigh?"));
    other.mounts.push(Mount {
        block_uid: s("sys_optics"),
        group_id: s("l3_optics"),
        release: s(""),
    });
    assert!(refused(&other, "l3_optics"));
    assert_eq!(start::allocation_wanted(&other, "l3_radiation"), None);
    // Not as allocate gives it: its closure gone, its closure saying no
    // sense, or the port an output.
    let mut no_closure = f.clone();
    no_closure
        .closures
        .retain(|c| c.required_ref != "sys_radiation.shield_mass_allocated");
    let mut no_sense = f.clone();
    for c in no_sense.closures.iter_mut() {
        c.sense = s("");
    }
    let mut output = f.clone();
    for p in output.ports.iter_mut().filter(|p| p.state == "allocated") {
        p.direction = s("out");
    }
    for (what, g) in [
        ("its closure gone", &no_closure),
        ("its closure with no sense", &no_sense),
        ("an output", &output),
    ] {
        assert!(refused(g, "l3_radiation"), "{what}");
    }
    // A block removed is still the systems' own: a group on it allocated
    // nothing is not started.
    let mut removed = other.clone();
    removed.blocks.last_mut().unwrap().archived = 1;
    assert!(refused(&removed, "l3_optics"));
}

#[test]
fn a_group_on_a_subsystem_s_leaf_is_not_held_to_an_allocation_yet() {
    // Its allocation is its valve owner's, in solar's file, and no edit
    // gives one there: it starts as it did.
    let copy = a_copy("w13-start", &systems_file());
    frame_in(
        &copy,
        &mount_edit("sw_activity_count", "l3_activity", "Rui Ray"),
        "Sam Engel",
        &key(12),
    )
    .unwrap();
    let sf = vleo_files::sqlite::read(&copy.join("groups/systems/systems.vgroup")).unwrap();
    assert_eq!(start::allocation_wanted(&sf, "l3_activity"), None);
    start_in(&copy, "l3_activity", "Rui Ray", &key(15)).expect("Rui starts it");
    let _ = std::fs::remove_dir_all(&copy);
}

#[test]
fn a_refused_mount_leaves_the_systems_file_as_it_was() {
    let copy = a_copy("unchanged", &systems_file());
    let at = copy.join("groups/systems/systems.vgroup");
    let was = digest(&at);
    for e in [
        mount_edit("sys_power", "l3_radiation", "Rui Ray"),
        mount_edit("sys_radiation", "l3_radiation", "Dee Puty"),
        allocate(allocation("a", "12", "", ""), "shield_mass", ""),
    ] {
        frame_in(&copy, &e, "Sam Engel", &key(12)).unwrap_err();
        assert_eq!(digest(&at), was, "{e:?}");
    }
    frame_in(
        &copy,
        &mount_edit("sys_radiation", "l3_radiation", "Rui Ray"),
        "Rui Ray",
        &key(15),
    )
    .unwrap_err();
    assert_eq!(digest(&at), was);
    let _ = std::fs::remove_dir_all(&copy);
}

#[test]
fn the_branch_that_holds_a_mounted_leaf_does_not_change_it() {
    let mounted = frame_it(
        &systems_file(),
        &mount_edit("sw_activity_count", "l3_activity", "Rui Ray"),
        "Sam Engel",
        &key(12),
    )
    .unwrap();
    let pr = programme();
    let sy = chain::open_systems(&mounted, &pr).unwrap();
    let ola = |e: &Edit| {
        breakdown::edit(
            &solar(),
            &[],
            std::slice::from_ref(&mounted),
            e,
            WHY,
            "Ola Sun",
            &key(13),
            AT,
            &sy,
            &pr,
        )
        .map(|_| ())
        .map_err(|e| e.message().to_string())
    };
    for e in [
        Edit::Fill {
            block: s("sw_activity_count"),
            part: Part::Question(s("How many quiet days?")),
        },
        Edit::Remove {
            block: s("sw_activity_count"),
        },
        Edit::Move {
            block: s("sw_activity_count"),
            parent: s("l3_solar"),
            ord: 3,
        },
        Edit::Rename {
            block: s("sw_activity_count"),
            id: s("sw_active"),
        },
        Edit::Add {
            parent: s("sw_activity_count"),
            id: s("sw_activity_child"),
            question: s("Under it?"),
        },
    ] {
        let why = ola(&e).unwrap_err();
        assert!(
            why.contains("sw_activity_count carries l3_activity, mounted on it in systems's file"),
            "{e:?}: {why}"
        );
    }
    // Before the mount, Ola fills it as they like.
    let pr2 = programme();
    let sy2 = systems(&pr2);
    breakdown::edit(
        &solar(),
        &[],
        &[systems_file()],
        &Edit::Fill {
            block: s("sw_activity_count"),
            part: Part::Question(s("How many quiet days?")),
        },
        WHY,
        "Ola Sun",
        &key(13),
        AT,
        &sy2,
        &pr2,
    )
    .expect("an unmounted leaf is Ola's to fill");
}

#[test]
fn a_port_filled_as_allocated_is_refused_for_it_says_no_sense() {
    let pr = programme();
    let sy = systems(&pr);
    let mut p = (*allocation("days_allocated", "40", "", "")).clone();
    p.direction = s("in");
    p.state = s("allocated");
    p.unit = s("");
    let e = breakdown::edit(
        &solar(),
        &[],
        &[systems_file()],
        &Edit::Fill {
            block: s("sw_activity_count"),
            part: Part::Port(Box::new(p)),
        },
        WHY,
        "Ola Sun",
        &key(13),
        AT,
        &sy,
        &pr,
    )
    .unwrap_err();
    assert!(
        e.message().contains("is filled as allocated"),
        "{}",
        e.message()
    );
}

#[test]
fn a_mount_where_its_owner_could_sign_or_start_nothing_is_refused() {
    let pr = programme();
    // An issued node of the systems' branch: its file is its node
    // engineer's, and a mount's assignment would let the owner sign it.
    let mut node = File::new(Kind::Node, APP);
    for (k, v) in [
        ("group_id", "systems"),
        ("block_uid", "sys_node"),
        ("revision", "1"),
        ("contract_version", "1"),
        ("writer", "Sam Engel"),
    ] {
        node.meta.insert(s(k), s(v));
    }
    node.blocks
        .push(block("sys_node", "", "method", "A node of its own?"));
    let e = breakdown::edit(
        &systems_file(),
        std::slice::from_ref(&node),
        &[pr.file.clone(), solar()],
        &mount_edit("sys_node", "l3_radiation", "Rui Ray"),
        WHY,
        "Sam Engel",
        &key(12),
        AT,
        &pr,
        &pr,
    )
    .unwrap_err();
    assert!(
        e.message()
            .contains("sys_node is issued, a node file of the systems' branch"),
        "{}",
        e.message()
    );
    // A leaf of a subsystem's branch that is a node file of its own is
    // mounted on, as W13 has it.
    let mut solar_node = node.clone();
    solar_node.meta.insert(s("group_id"), s("l3_solar"));
    solar_node.meta.insert(s("block_uid"), s("sw_issued"));
    solar_node.blocks = vec![block("sw_issued", "l3_solar", "method", "Issued?")];
    breakdown::edit(
        &systems_file(),
        &[],
        &[pr.file.clone(), solar(), solar_node],
        &mount_edit("sw_issued", "l3_issued", "Rui Ray"),
        WHY,
        "Sam Engel",
        &key(12),
        AT,
        &pr,
        &pr,
    )
    .expect("an issued leaf of a subsystem's branch is mounted on");
    // A group the copy holds a release of already: its owner could not start
    // it in an empty folder.
    let mut release = File::new(Kind::GroupRelease, APP);
    release.meta.insert(s("group_id"), s("l3_radiation"));
    let e = breakdown::edit(
        &systems_file(),
        &[],
        &[pr.file.clone(), solar(), release],
        &mount_edit("sys_radiation", "l3_radiation", "Rui Ray"),
        WHY,
        "Sam Engel",
        &key(12),
        AT,
        &pr,
        &pr,
    )
    .unwrap_err();
    assert!(
        e.message().contains("the copy holds a") && e.message().contains("of l3_radiation already"),
        "{}",
        e.message()
    );
}

#[test]
fn an_allocation_on_an_issued_node_is_refused_for_its_ports_are_its_contract() {
    let pr = programme();
    // Written by hand: a mount on an issued node of the systems' branch,
    // which mount itself refuses.
    let mut sf = systems_file();
    sf.signatures.clear();
    sf.mounts.push(Mount {
        block_uid: s("sys_node"),
        group_id: s("l3_node"),
        release: s(""),
    });
    sf.assignments.push(Assignment {
        block_uid: s("sys_node"),
        person: s("Rui Ray"),
        contract_version: 1,
    });
    let sf = signed(sf, "Sam Engel", &key(12));
    let mut node = File::new(Kind::Node, APP);
    for (k, v) in [
        ("group_id", "systems"),
        ("block_uid", "sys_node"),
        ("revision", "1"),
        ("contract_version", "1"),
        ("writer", "Sam Engel"),
    ] {
        node.meta.insert(s(k), s(v));
    }
    node.blocks
        .push(block("sys_node", "", "method", "A node of its own?"));
    node.ports.push(out_port("sys_node", "mass", "kg"));
    let e = breakdown::edit(
        &sf,
        std::slice::from_ref(&node),
        &[pr.file.clone(), solar()],
        &Edit::Allocate {
            block: s("sys_node"),
            port: allocation("mass_allocated", "3", "", ""),
            bounds: s("mass"),
            sense: s("<="),
            says: s(""),
        },
        WHY,
        "Sam Engel",
        &key(12),
        AT,
        &pr,
        &pr,
    )
    .unwrap_err();
    assert!(
        e.message().contains("its ports are its contract (W4)"),
        "{}",
        e.message()
    );
}

// ── An allocation changed (F6-c, question 48) ──

fn reallocate(name: &str, value: &str, lower: &str, upper: &str, sense: &str) -> Edit {
    Edit::Reallocate {
        block: s("sys_radiation"),
        port: s(name),
        value: s(value),
        lower: s(lower),
        upper: s(upper),
        sense: s(sense),
    }
}

/// Radiation's mount, given 12 kg of shield, kept under it.
fn radiation_allocated() -> File {
    frame_it(
        &radiation_mounted(),
        &allocate(
            allocation("shield_mass_allocated", "12", "", ""),
            "shield_mass",
            "<=",
        ),
        "Sam Engel",
        &key(12),
    )
    .expect("Sam allocates")
}

fn allocated(f: &File) -> (&Port, &Closure) {
    (
        f.ports
            .iter()
            .find(|p| p.name == "shield_mass_allocated")
            .unwrap(),
        f.closures
            .iter()
            .find(|c| c.uid == "sys_radiation.shield_mass_allocated")
            .unwrap(),
    )
}

#[test]
fn an_allocation_is_changed_and_its_sense_only_when_given() {
    let before = radiation_allocated();
    // Another value; its sense kept.
    let f = frame_it(
        &before,
        &reallocate("shield_mass_allocated", "14", "", "", ""),
        "Sam Engel",
        &key(12),
    )
    .expect("Sam gives it another value");
    let (p, c) = allocated(&f);
    assert_eq!((p.value.as_str(), c.sense.as_str()), ("14", "<="));
    let change = f.changes.last().unwrap();
    assert!(change.what.starts_with("reallocate: "), "{change:?}");
    assert!(
        change.before.as_deref().unwrap().contains("value=\"12\"")
            && change.after.as_deref().unwrap().contains("value=\"14\""),
        "{change:?}"
    );
    chain::open_systems(&f, &programme()).expect("it opens");
    // A range in place of the value, reaching it: the sense given changes.
    let f = frame_it(
        &f,
        &reallocate("shield_mass_allocated", "", "10", "16", ">="),
        "Sid Side",
        &key(17),
    )
    .expect("Sam's deputy gives it a range and the other sense");
    let (p, c) = allocated(&f);
    assert_eq!(
        (
            p.value.as_str(),
            p.lower.as_str(),
            p.upper.as_str(),
            c.sense.as_str()
        ),
        ("", "10", "16", ">=")
    );
    // Its unit, what it bounds and its state are not the change's.
    assert_eq!(
        (p.unit.as_str(), p.state.as_str(), c.achieved_ref.as_str()),
        ("kg", "allocated", "sys_radiation.shield_mass")
    );
}

#[test]
fn an_allocation_changed_is_refused_by_name() {
    let before = radiation_allocated();
    for (e, who, k, why) in [
        (
            reallocate("shield_mass_allocated", "14", "", "", ""),
            "Ola Sun",
            13,
            "may not",
        ),
        (
            reallocate("shield_mass", "14", "", "", ""),
            "Sam Engel",
            12,
            "sys_radiation has no allocation \"shield_mass\" to change",
        ),
        (
            reallocate("shield_mass_allocated", "", "", "", ""),
            "Sam Engel",
            12,
            "is given no value and no range",
        ),
        (
            reallocate("shield_mass_allocated", "14", "10", "16", ""),
            "Sam Engel",
            12,
            "is given a value and a range",
        ),
        (
            reallocate("shield_mass_allocated", "", "10", "", ""),
            "Sam Engel",
            12,
            "is given one end of a range",
        ),
        (
            reallocate("shield_mass_allocated", "heavy", "", "", ""),
            "Sam Engel",
            12,
            "is given the value \"heavy\", which is not a number",
        ),
        (
            reallocate("shield_mass_allocated", "14", "", "", "<"),
            "Sam Engel",
            12,
            "\"<\" is not a sense",
        ),
        (
            reallocate("shield_mass_allocated", "12", "", "", ""),
            "Sam Engel",
            12,
            "is given what it holds already: nothing changes",
        ),
        (
            reallocate("shield_mass_allocated", "12", "", "", "<="),
            "Sam Engel",
            12,
            "is given what it holds already: nothing changes",
        ),
        (
            reallocate("shield_mass_allocated", "", "16", "10", ""),
            "Sam Engel",
            12,
            "runs from 16 to 10: its lower end is above its upper",
        ),
    ] {
        let e = frame_it(&before, &e, who, &key(k)).unwrap_err();
        assert!(e.contains(why), "{why}: {e}");
    }
    // On a mount removed, never.
    let mut gone = before.clone();
    gone.blocks
        .iter_mut()
        .find(|b| b.uid == "sys_radiation")
        .unwrap()
        .archived = 1;
    gone.signatures.clear();
    let gone = signed(gone, "Sam Engel", &key(12));
    let e = frame_it(
        &gone,
        &reallocate("shield_mass_allocated", "14", "", "", ""),
        "Sam Engel",
        &key(12),
    )
    .unwrap_err();
    assert!(
        e.contains("sys_radiation is removed: it is archived"),
        "{e}"
    );
    // In a subsystem's own file, never.
    let e = breakdown::edit(
        &solar(),
        &[],
        &[programme().file.clone(), before.clone()],
        &reallocate("shield_mass_allocated", "14", "", "", ""),
        WHY,
        "Ola Sun",
        &key(13),
        AT,
        &systems(&programme()),
        &programme(),
    )
    .unwrap_err();
    assert!(e.message().contains("mounts nothing"), "{}", e.message());
}
