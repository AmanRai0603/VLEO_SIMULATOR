//! A key says who it is, and a group's owner is read from the file that
//! mounts it.
//!
//! docs/briefs/F1.md: the design mounts the systems' branch in the programme's
//! file and each subsystem's group in the systems' file, so the chain runs
//! programme → systems → group. A small programme is made here in that shape:
//! its manager, the system engineer, two subsystem engineers and a deputy, a
//! node engineer, and a stranger, each with their own key. Each link is opened,
//! then broken in turn and refused by name; and each key is asked who it is.

use vleo_files::chain::{self, Registry, FILE};
use vleo_files::keys::SigningKey;
use vleo_files::meta::Kind;
use vleo_files::model::*;

const APP: &str = "vleo 1.0.0 (test)";

struct People {
    pm: SigningKey,
    se: SigningKey,
    sub: SigningKey,
    deputy: SigningKey,
    other_sub: SigningKey,
    ne: SigningKey,
    stranger: SigningKey,
}

fn people() -> People {
    People {
        pm: SigningKey::from_seed([11; 32]),
        se: SigningKey::from_seed([12; 32]),
        sub: SigningKey::from_seed([13; 32]),
        deputy: SigningKey::from_seed([14; 32]),
        other_sub: SigningKey::from_seed([15; 32]),
        ne: SigningKey::from_seed([16; 32]),
        stranger: SigningKey::from_seed([17; 32]),
    }
}

fn s(v: &str) -> String {
    v.to_string()
}

fn register(f: &mut File, name: &str, role: &str, deputy_for: &str, key: &SigningKey, by: &str) {
    f.people.push(Person {
        name: s(name),
        role: s(role),
        deputy_for: s(deputy_for),
    });
    f.keys.push(PersonKey {
        person: s(name),
        public_key: key.public().to_hex(),
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

fn group_file(id: &str) -> File {
    let mut f = File::new(Kind::Group, APP);
    f.meta.insert(s("group_id"), s(id));
    f.meta.insert(s("based_on"), s(""));
    f
}

fn signed(mut f: File, who: &str, key: &SigningKey) -> File {
    let row = chain::sign(&f, FILE, who, key, "2026-02-01", "ok", "").unwrap();
    f.signatures.push(row);
    f
}

/// The programme's file as its manager writes it, unsigned: it registers the
/// people and mounts the systems' branch on its root, assigned to `systems_owner`.
fn programme_unsigned(p: &People, systems_owner: &str) -> File {
    let mut f = group_file("programme");
    f.meta.insert(s("writer"), s("Pat Morgan"));
    register(
        &mut f,
        "Pat Morgan",
        "programme manager",
        "",
        &p.pm,
        "Pat Morgan",
    );
    register(
        &mut f,
        "Sam Engel",
        "system engineer",
        "",
        &p.se,
        "Pat Morgan",
    );
    register(
        &mut f,
        "Ola Sun",
        "subsystem engineer",
        "",
        &p.sub,
        "Pat Morgan",
    );
    register(
        &mut f,
        "Dee Puty",
        "subsystem engineer",
        "Ola Sun",
        &p.deputy,
        "Pat Morgan",
    );
    register(
        &mut f,
        "Ivo Thrust",
        "subsystem engineer",
        "",
        &p.other_sub,
        "Pat Morgan",
    );
    mount(&mut f, "root", "systems", systems_owner);
    f
}

fn programme(p: &People) -> Registry {
    let f = signed(programme_unsigned(p, "Sam Engel"), "Pat Morgan", &p.pm);
    chain::open_programme(f, &p.pm.public().fingerprint()).unwrap()
}

/// The systems' file, unsigned: it mounts solar and propulsion on their blocks.
fn systems_unsigned() -> File {
    let mut f = group_file("systems");
    mount(&mut f, "sys_space_environment", "l3_solar", "Ola Sun");
    mount(&mut f, "sys_propulsion", "l3_prop", "Ivo Thrust");
    f
}

fn systems(p: &People) -> File {
    signed(systems_unsigned(), "Sam Engel", &p.se)
}

/// Solar's group file, unsigned: it registers its node engineer and assigns
/// them two nodes.
fn solar_unsigned(p: &People) -> File {
    let mut f = group_file("l3_solar");
    register(&mut f, "Nia Node", "node engineer", "", &p.ne, "Ola Sun");
    for uid in ["b-ap", "b-f107"] {
        f.assignments.push(Assignment {
            block_uid: s(uid),
            person: s("Nia Node"),
            contract_version: 1,
        });
    }
    f
}

fn message(e: vleo_files::Error) -> String {
    e.message().to_string()
}

#[test]
fn the_systems_file_opens_through_the_programme_by_its_system_engineer() {
    let p = people();
    let pr = programme(&p);
    chain::open_systems(&systems(&p), &pr).expect("signed by the system engineer");

    let by_sub = signed(systems_unsigned(), "Ola Sun", &p.sub);
    let e = message(chain::open_systems(&by_sub, &pr).unwrap_err());
    assert!(e.contains("by its owner Sam Engel"), "{e}");

    // The programme assigns its root to a subsystem engineer: refused, whoever signs.
    let f = signed(programme_unsigned(&p, "Ola Sun"), "Pat Morgan", &p.pm);
    let odd = chain::open_programme(f, &p.pm.public().fingerprint()).unwrap();
    let e = message(chain::open_systems(&by_sub, &odd).unwrap_err());
    assert!(e.contains("not registered as its system engineer"), "{e}");

    // The old opener admits only a subsystem engineer: the systems' file cannot
    // open through it, which is why open_systems exists.
    assert!(chain::open_group(&systems(&p), &pr).is_err());
}

#[test]
fn a_group_opens_through_the_file_that_mounts_it() {
    let p = people();
    let pr = programme(&p);
    let sys = chain::open_systems(&systems(&p), &pr).unwrap();

    chain::open_group_in(&signed(solar_unsigned(&p), "Ola Sun", &p.sub), &sys, &pr)
        .expect("signed by its owner");
    chain::open_group_in(
        &signed(solar_unsigned(&p), "Dee Puty", &p.deputy),
        &sys,
        &pr,
    )
    .expect("signed by its owner's deputy");

    let e = message(
        chain::open_group_in(
            &signed(solar_unsigned(&p), "Ivo Thrust", &p.other_sub),
            &sys,
            &pr,
        )
        .unwrap_err(),
    );
    assert!(
        e.contains("by its owner Ola Sun") && e.contains("Ivo Thrust may not sign"),
        "{e}"
    );

    let e = message(
        chain::open_group_in(
            &signed(solar_unsigned(&p), "Ola Sun", &p.stranger),
            &sys,
            &pr,
        )
        .unwrap_err(),
    );
    assert!(e.contains("not one the programme's file registers"), "{e}");

    // Read through the programme instead, solar has no mount: refused by name.
    let e = message(
        chain::open_group(&signed(solar_unsigned(&p), "Ola Sun", &p.sub), &pr).unwrap_err(),
    );
    assert!(e.contains("mounts no group \"l3_solar\""), "{e}");
}

#[test]
fn a_group_has_one_parent_and_an_assigned_mount() {
    let p = people();
    let pr = programme(&p);
    let sys = chain::open_systems(&systems(&p), &pr).unwrap();

    let (from, owner) = chain::mounted_by("l3_solar", &[&pr, &sys]).unwrap();
    assert_eq!(
        (from.from.as_str(), owner),
        ("systems's group file", "Ola Sun")
    );

    let e = message(chain::mounted_by("l3_thermal", &[&pr, &sys]).unwrap_err());
    assert!(e.contains("no file mounts the group \"l3_thermal\""), "{e}");

    // Mounted again in the programme's file, as a workaround would: two parents.
    let mut twice = programme_unsigned(&p, "Sam Engel");
    mount(&mut twice, "m-solar", "l3_solar", "Ola Sun");
    let twice = chain::open_programme(
        signed(twice, "Pat Morgan", &p.pm),
        &p.pm.public().fingerprint(),
    )
    .unwrap();
    let e = message(chain::mounted_by("l3_solar", &[&twice, &sys]).unwrap_err());
    assert!(e.contains("mounted twice"), "{e}");

    // A mount on a block nobody is assigned.
    let mut bare = systems_unsigned();
    bare.assignments.retain(|a| a.block_uid != "sys_propulsion");
    let bare = chain::open_systems(&signed(bare, "Sam Engel", &p.se), &pr).unwrap();
    let e = message(chain::mounted_by("l3_prop", &[&pr, &bare]).unwrap_err());
    assert!(e.contains("assigned to nobody"), "{e}");
}

#[test]
fn a_release_is_sealed_through_the_file_that_mounts_its_group() {
    let p = people();
    let pr = programme(&p);
    let sys = chain::open_systems(&systems(&p), &pr).unwrap();
    let mut r = File::new(Kind::GroupRelease, APP);
    for (k, v) in [
        ("group_id", "l3_solar"),
        ("version", "1.2"),
        ("previous", "1.1"),
        ("based_on", ""),
    ] {
        r.meta.insert(s(k), s(v));
    }
    register(&mut r, "Nia Node", "node engineer", "", &p.ne, "Ola Sun");
    r.blocks.push(Block {
        uid: s("b-ap"),
        id: s("sw_ap_design"),
        behaviour: s("method"),
        revision: 3,
        contract_version: 1,
        ..Block::default()
    });
    r.assignments.push(Assignment {
        block_uid: s("b-ap"),
        person: s("Nia Node"),
        contract_version: 1,
    });
    let row = chain::sign(&r, "b-ap", "Nia Node", &p.ne, "2026-03-01", "ok", "").unwrap();
    r.signatures.push(row);
    let r = signed(r, "Ola Sun", &p.sub);

    let c = chain::check_release_in(&r, &sys, &pr).unwrap();
    assert!(c.holds(), "{:?}", c.refused);
    assert_eq!(c.signed, vec![s("sw_ap_design")]);

    // Checked through the programme, as before, the seal finds no mount.
    let c = chain::check_release(&r, &pr).unwrap();
    assert!(
        !c.holds() && c.refused[0].starts_with("the seal:"),
        "{:?}",
        c.refused
    );
}

#[test]
fn every_key_says_who_it_is() {
    let p = people();
    let pr = programme(&p);
    let sys = chain::open_systems(&systems(&p), &pr).unwrap();
    let solar =
        chain::open_group_in(&signed(solar_unsigned(&p), "Ola Sun", &p.sub), &sys, &pr).unwrap();
    let groups = [&sys, &solar];
    let who = |k: &SigningKey| chain::whoami(&k.public().to_hex(), "2026-04-01", &pr, &groups);

    let pm = who(&p.pm).unwrap();
    assert_eq!(pm.name, "Pat Morgan");
    assert_eq!(
        pm.roles,
        vec![(s("programme manager"), s("the programme's file"))]
    );
    assert!(pm.owns.is_empty() && pm.nodes.is_empty());

    let se = who(&p.se).unwrap();
    assert_eq!(
        (se.name.as_str(), se.owns.clone()),
        ("Sam Engel", vec![s("systems")])
    );

    let sub = who(&p.sub).unwrap();
    assert_eq!(sub.owns, vec![s("l3_solar")]);
    assert!(sub.deputy_for.is_empty());

    let dep = who(&p.deputy).unwrap();
    assert_eq!(dep.deputy_for, vec![s("Ola Sun")]);
    assert!(
        dep.owns.is_empty(),
        "a deputy acts for the owner, but owns nothing"
    );

    let ne = who(&p.ne).unwrap();
    assert_eq!(
        ne.roles,
        vec![(s("node engineer"), s("l3_solar's group file"))]
    );
    assert_eq!(
        ne.nodes,
        vec![(s("l3_solar"), s("b-ap")), (s("l3_solar"), s("b-f107"))]
    );

    let e = message(who(&p.stranger).unwrap_err());
    assert!(e.contains("no file registers this key"), "{e}");
}

#[test]
fn a_revoked_or_shared_key_says_nobody() {
    let p = people();
    let mut f = programme_unsigned(&p, "Sam Engel");
    for k in f.keys.iter_mut().filter(|k| k.person == "Ivo Thrust") {
        k.revoked_from = s("2026-03-01");
    }
    // The same key registered again under a second name.
    f.people.push(Person {
        name: s("Ola Again"),
        role: s("subsystem engineer"),
        deputy_for: s(""),
    });
    f.keys.push(PersonKey {
        person: s("Ola Again"),
        public_key: p.sub.public().to_hex(),
        registered_at: s("2026-01-01"),
        registered_by: s("Pat Morgan"),
        revoked_from: s(""),
    });
    let pr = chain::open_programme(signed(f, "Pat Morgan", &p.pm), &p.pm.public().fingerprint())
        .unwrap();

    let before = chain::whoami(&p.other_sub.public().to_hex(), "2026-02-15", &pr, &[]).unwrap();
    assert_eq!(before.name, "Ivo Thrust");
    let e =
        message(chain::whoami(&p.other_sub.public().to_hex(), "2026-04-01", &pr, &[]).unwrap_err());
    assert!(e.contains("revoked from 2026-03-01"), "{e}");

    let e = message(chain::whoami(&p.sub.public().to_hex(), "2026-04-01", &pr, &[]).unwrap_err());
    assert!(
        e.contains("more than one person") && e.contains("Ola Again"),
        "{e}"
    );
}
