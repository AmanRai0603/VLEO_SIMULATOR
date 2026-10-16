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
        proof: vleo_files::people::proof(key, name),
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
    let who = |k: &SigningKey| {
        chain::whoami(
            &k.public().to_hex(),
            "2026-04-01",
            &pr,
            Some(&sys),
            &[&solar],
        )
    };

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
        proof: vleo_files::people::proof(&p.sub, "Ola Again"),
    });
    let pr = chain::open_programme(signed(f, "Pat Morgan", &p.pm), &p.pm.public().fingerprint())
        .unwrap();

    let before =
        chain::whoami(&p.other_sub.public().to_hex(), "2026-02-15", &pr, None, &[]).unwrap();
    assert_eq!(before.name, "Ivo Thrust");
    let e = message(
        chain::whoami(&p.other_sub.public().to_hex(), "2026-04-01", &pr, None, &[]).unwrap_err(),
    );
    assert!(e.contains("revoked from 2026-03-01"), "{e}");

    let e =
        message(chain::whoami(&p.sub.public().to_hex(), "2026-04-01", &pr, None, &[]).unwrap_err());
    assert!(
        e.contains("more than one person") && e.contains("Ola Again"),
        "{e}"
    );
}

#[test]
fn a_group_file_registers_only_its_own_node_engineers() {
    let p = people();
    let pr = programme(&p);
    let sys = chain::open_systems(&systems(&p), &pr).unwrap();
    let opens =
        |f: File| chain::open_group_in(&signed(f, "Ola Sun", &p.sub), &sys, &pr).map(|_| ());
    // Ola registers, in her own signed group file, the system engineer's name
    // with a key she made; a "system engineer" of her own; and the system
    // engineer's key, revoked, under another name, to lock him out.
    let mut f = solar_unsigned(&p);
    register(
        &mut f,
        "Sam Engel",
        "node engineer",
        "",
        &p.stranger,
        "Ola Sun",
    );
    let e = message(opens(f).unwrap_err());
    assert!(e.contains("whom the programme's file registers"), "{e}");

    let mut f = solar_unsigned(&p);
    register(
        &mut f,
        "Mal Lory",
        "system engineer",
        "",
        &p.stranger,
        "Ola Sun",
    );
    let e = message(opens(f).unwrap_err());
    assert!(e.contains("registers only its node engineers"), "{e}");

    let mut f = solar_unsigned(&p);
    register(&mut f, "Not Sam", "node engineer", "", &p.se, "Ola Sun");
    f.keys.last_mut().unwrap().revoked_from = s("2026-01-02");
    let e = message(opens(f).unwrap_err());
    assert!(e.contains("a key the programme's file registers"), "{e}");
    // The system engineer is still himself.
    let se = chain::whoami(&p.se.public().to_hex(), "2026-04-01", &pr, Some(&sys), &[]).unwrap();
    assert_eq!(se.name, "Sam Engel");
}

#[test]
fn a_revocation_counts_in_the_file_that_made_it() {
    let p = people();
    let pr = programme(&p);
    let mut sys = systems_unsigned();
    mount(&mut sys, "sys_thermal", "l3_thermal", "Ola Sun");
    let sys = chain::open_systems(&signed(sys, "Sam Engel", &p.se), &pr).unwrap();
    // Nia works in two groups; solar revokes her key, thermal does not.
    let mut solar = solar_unsigned(&p);
    solar.keys[0].revoked_from = s("2026-03-01");
    let solar = chain::open_group_in(&signed(solar, "Ola Sun", &p.sub), &sys, &pr).unwrap();
    let mut thermal = group_file("l3_thermal");
    register(
        &mut thermal,
        "Nia Node",
        "node engineer",
        "",
        &p.ne,
        "Ola Sun",
    );
    thermal.assignments.push(Assignment {
        block_uid: s("b-radiator"),
        person: s("Nia Node"),
        contract_version: 1,
    });
    let thermal = chain::open_group_in(&signed(thermal, "Ola Sun", &p.sub), &sys, &pr).unwrap();

    let ne = p.ne.public().to_hex();
    let before = chain::whoami(&ne, "2026-02-15", &pr, Some(&sys), &[&solar, &thermal]).unwrap();
    assert_eq!(before.roles.len(), 2, "{before:?}");
    assert_eq!(before.nodes.len(), 3, "{before:?}");
    for order in [[&solar, &thermal], [&thermal, &solar]] {
        let after = chain::whoami(&ne, "2026-04-01", &pr, Some(&sys), &order).unwrap();
        assert_eq!(
            (after.roles.clone(), after.nodes.clone()),
            (
                vec![(s("node engineer"), s("l3_thermal's group file"))],
                vec![(s("l3_thermal"), s("b-radiator"))]
            ),
            "solar's revocation counts in solar"
        );
    }
    let e = message(chain::whoami(&ne, "2026-04-01", &pr, Some(&sys), &[&solar]).unwrap_err());
    assert!(e.contains("revoked from 2026-03-01"), "{e}");
}

#[test]
fn an_owner_is_an_engineer_and_a_deputy_stands_in_for_one_person() {
    let p = people();
    // The stranger, registered as Ivo's deputy: they may act for Ivo, not Ola.
    let mut f = programme_unsigned(&p, "Sam Engel");
    register(
        &mut f,
        "Ivy Standin",
        "subsystem engineer",
        "Ivo Thrust",
        &p.stranger,
        "Pat Morgan",
    );
    let pr = chain::open_programme(signed(f, "Pat Morgan", &p.pm), &p.pm.public().fingerprint())
        .unwrap();
    let sys = chain::open_systems(&systems(&p), &pr).unwrap();
    let e = message(
        chain::open_group_in(
            &signed(solar_unsigned(&p), "Ivy Standin", &p.stranger),
            &sys,
            &pr,
        )
        .unwrap_err(),
    );
    assert!(e.contains("Ivy Standin may not sign"), "{e}");

    // A group mounted on a block assigned to the programme manager.
    let mut odd = systems_unsigned();
    mount(&mut odd, "sys_thermal", "l3_thermal", "Pat Morgan");
    let odd = chain::open_systems(&signed(odd, "Sam Engel", &p.se), &pr).unwrap();
    let thermal = signed(group_file("l3_thermal"), "Pat Morgan", &p.pm);
    let e = message(chain::open_group_in(&thermal, &odd, &pr).unwrap_err());
    assert!(
        e.contains("neither a system engineer nor a subsystem engineer"),
        "{e}"
    );
}

#[test]
fn a_stamp_is_a_day_or_a_moment() {
    let p = people();
    let mut f = programme_unsigned(&p, "Sam Engel");
    for k in f.keys.iter_mut().filter(|k| k.person == "Ivo Thrust") {
        // Read as text, "2026-3-1" sorts after "2026-10-15" and the
        // revocation would never come.
        k.revoked_from = s("2026-3-1");
    }
    let pr = chain::open_programme(signed(f, "Pat Morgan", &p.pm), &p.pm.public().fingerprint())
        .unwrap();
    let ivo = p.other_sub.public().to_hex();
    let e = message(chain::whoami(&ivo, "2026-10-15T09:00:00Z", &pr, None, &[]).unwrap_err());
    assert!(e.contains("revoked from 2026-3-1"), "{e}");

    let mut f = programme_unsigned(&p, "Sam Engel");
    for k in f.keys.iter_mut().filter(|k| k.person == "Ivo Thrust") {
        k.revoked_from = s("first of March");
    }
    let pr = chain::open_programme(signed(f, "Pat Morgan", &p.pm), &p.pm.public().fingerprint())
        .unwrap();
    let e = message(chain::whoami(&ivo, "2026-04-01", &pr, None, &[]).unwrap_err());
    assert!(e.contains("neither a day"), "{e}");
}

/// The page checks a release through the programme's file, by the old opener;
/// it holds the group file to the same rule as the program does.
#[test]
fn the_page_holds_a_released_group_file_to_the_same_rule() {
    let p = people();
    let mut f = programme_unsigned(&p, "Sam Engel");
    mount(&mut f, "m-solar", "l3_solar", "Ola Sun");
    let programme = signed(f, "Pat Morgan", &p.pm);
    let mut r = File::new(Kind::GroupRelease, APP);
    for (k, v) in [
        ("group_id", "l3_solar"),
        ("version", "1.2"),
        ("previous", "1.1"),
        ("based_on", ""),
    ] {
        r.meta.insert(s(k), s(v));
    }
    // Solar's owner registers herself as a node engineer, with another key.
    register(
        &mut r,
        "Ola Sun",
        "node engineer",
        "",
        &p.stranger,
        "Ola Sun",
    );
    let r = signed(r, "Ola Sun", &p.sub);
    let mut request = Vec::new();
    let mut put = |b: &[u8]| {
        request.extend_from_slice(&(b.len() as u32).to_le_bytes());
        request.extend_from_slice(b);
    };
    put(p.pm.public().fingerprint().as_bytes());
    put(&vleo_files::rows::encode(&programme.to_tables()));
    put(&vleo_files::rows::encode(&r.to_tables()));
    let answer = vleo_files::page::check_release(&request);
    assert!(
        answer.contains("\"holds\":false")
            && answer.contains("whom the programme's file registers"),
        "{answer}"
    );
}
