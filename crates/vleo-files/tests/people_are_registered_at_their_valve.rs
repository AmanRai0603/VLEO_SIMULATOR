//! Whom a file registers is changed at the valve that owns it, and a change
//! that would leave the file refused is never given back.
//!
//! docs/briefs/F1.md, PR 2: the programme manager registers, replaces and
//! revokes the keys in the programme's file; a group's owner, or their deputy,
//! does the same for the node engineers in their group file. A small
//! programme of invented people is made here, then changed by each valve and
//! by people who do not hold it. Then `base_version`, which counts the
//! released design a file was based on.

use vleo_files::chain::{self, Registry, FILE};
use vleo_files::keys::SigningKey;
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::people::{self, Change, Valve};

const APP: &str = "vleo 1.0.0 (test)";

fn s(v: &str) -> String {
    v.to_string()
}

fn key(seed: u8) -> SigningKey {
    SigningKey::from_seed([seed; 32])
}

fn person(name: &str, role: &str, deputy_for: &str) -> Person {
    Person {
        name: s(name),
        role: s(role),
        deputy_for: s(deputy_for),
    }
}

fn register(f: &mut File, name: &str, role: &str, deputy_for: &str, k: &SigningKey) {
    f.people.push(person(name, role, deputy_for));
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

// The people: Pat the programme manager (11), Sam the system engineer (12),
// Ola who owns solar (13), Dee her deputy (14), Ivo who owns propulsion (15),
// Nia a node engineer in solar (16), and new keys from 30 on.
fn programme_file() -> File {
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
    register(&mut f, "Ivo Thrust", "subsystem engineer", "", &key(15));
    mount(&mut f, "root", "systems", "Sam Engel");
    signed(f, "Pat Morgan", &key(11))
}

fn anchor() -> String {
    key(11).public().fingerprint()
}

fn opened(f: File) -> Registry {
    chain::open_programme(f, &anchor()).unwrap()
}

fn systems(pr: &Registry) -> Registry {
    let mut f = group_file("systems");
    mount(&mut f, "sys_space_environment", "l3_solar", "Ola Sun");
    mount(&mut f, "sys_propulsion", "l3_prop", "Ivo Thrust");
    chain::open_systems(&signed(f, "Sam Engel", &key(12)), pr).unwrap()
}

fn solar_file() -> File {
    let mut f = group_file("l3_solar");
    f.people.push(person("Nia Node", "node engineer", ""));
    f.keys.push(PersonKey {
        person: s("Nia Node"),
        public_key: key(16).public().to_hex(),
        registered_at: s("2026-01-01"),
        registered_by: s("Ola Sun"),
        revoked_from: s(""),
        proof: vleo_files::people::proof(&key(16), "Nia Node"),
    });
    signed(f, "Ola Sun", &key(13))
}

fn message(e: vleo_files::Error) -> String {
    e.message().to_string()
}

fn register_change(name: &str, role: &str, deputy_for: &str, k: &SigningKey) -> Change {
    Change::Register {
        person: person(name, role, deputy_for),
        public_key: k.public().to_hex(),
        proof: people::proof(k, name),
    }
}

fn replace_change(name: &str, k: &SigningKey) -> Change {
    Change::Replace {
        person: s(name),
        public_key: k.public().to_hex(),
        proof: people::proof(k, name),
    }
}

#[test]
fn the_programme_manager_registers_in_the_programme() {
    let a = anchor();
    let valve = Valve::Programme { anchor: &a };
    let new = register_change("Tom Thermal", "subsystem engineer", "", &key(30));
    let f = people::change(
        &programme_file(),
        &valve,
        "Pat Morgan",
        &key(11),
        "2026-03-01",
        &new,
    )
    .unwrap();
    assert_eq!(
        f.meta.get("signed_rows").map(String::as_str),
        Some("issue change"),
        "its rows are signed with it (F6-d)"
    );
    let pr = opened(f.clone());
    let tom = chain::whoami(&key(30).public().to_hex(), "2026-03-02", &pr, None, &[]).unwrap();
    assert_eq!(tom.name, "Tom Thermal");
    assert_eq!(
        f.signatures
            .iter()
            .filter(|r| r.scope == FILE)
            .map(|r| r.signer.as_str())
            .collect::<Vec<_>>(),
        vec!["Pat Morgan"],
        "the change is signed afresh by whoever made it, and only by them"
    );

    // Nobody else holds the programme's valve.
    for (who, k) in [("Sam Engel", key(12)), ("Ola Sun", key(13))] {
        let e = message(
            people::change(&programme_file(), &valve, who, &k, "2026-03-01", &new).unwrap_err(),
        );
        assert!(e.contains("the programme manager does"), "{who}: {e}");
    }
    // A key the programme does not register for the signer.
    let e = message(
        people::change(
            &programme_file(),
            &valve,
            "Pat Morgan",
            &key(99),
            "2026-03-01",
            &new,
        )
        .unwrap_err(),
    );
    assert!(e.contains("not one the programme's file registers"), "{e}");

    // A node engineer is registered by their group, not here.
    let e = message(
        people::change(
            &programme_file(),
            &valve,
            "Pat Morgan",
            &key(11),
            "2026-03-01",
            &register_change("Nina", "node engineer", "", &key(31)),
        )
        .unwrap_err(),
    );
    assert!(e.contains("not a node engineer"), "{e}");
}

#[test]
fn one_name_one_key_and_a_deputy_holds_their_holders_role() {
    let a = anchor();
    let valve = Valve::Programme { anchor: &a };
    let change = |c: &Change| {
        message(
            people::change(
                &programme_file(),
                &valve,
                "Pat Morgan",
                &key(11),
                "2026-03-01",
                c,
            )
            .unwrap_err(),
        )
    };
    let e = change(&register_change(
        "Ola Sun",
        "subsystem engineer",
        "",
        &key(30),
    ));
    assert!(
        e.contains("registered in the programme's file already"),
        "{e}"
    );
    let e = change(&register_change(
        "Ola Again",
        "subsystem engineer",
        "",
        &key(13),
    ));
    assert!(
        e.contains("registered in the programme's file for Ola Sun already"),
        "{e}"
    );
    let e = change(&register_change(
        "Sid Standin",
        "subsystem engineer",
        "Sam Engel",
        &key(30),
    ));
    assert!(e.contains("a deputy holds the role"), "{e}");
}

#[test]
fn a_replaced_key_is_revoked_from_the_change_and_the_old_signatures_stand() {
    let a = anchor();
    let valve = Valve::Programme { anchor: &a };
    let f = people::change(
        &programme_file(),
        &valve,
        "Pat Morgan",
        &key(11),
        "2026-03-01",
        &replace_change("Ivo Thrust", &key(30)),
    )
    .unwrap();
    let pr = opened(f);
    let old = key(15).public().to_hex();
    let new = key(30).public().to_hex();
    assert_eq!(
        chain::whoami(&old, "2026-02-15", &pr, None, &[])
            .unwrap()
            .name,
        "Ivo Thrust",
        "what the old key signed before stands"
    );
    let e = message(chain::whoami(&old, "2026-03-02", &pr, None, &[]).unwrap_err());
    assert!(e.contains("revoked from 2026-03-01"), "{e}");
    assert_eq!(
        chain::whoami(&new, "2026-03-02", &pr, None, &[])
            .unwrap()
            .name,
        "Ivo Thrust"
    );
}

#[test]
fn a_key_is_revoked_once_and_never_by_its_own_person() {
    let a = anchor();
    let valve = Valve::Programme { anchor: &a };
    let revoke = |who: &str| Change::Revoke {
        person: s(who),
        public_key: key(15).public().to_hex(),
        from: s("2026-03-01"),
    };
    let f = people::change(
        &programme_file(),
        &valve,
        "Pat Morgan",
        &key(11),
        "2026-03-01",
        &revoke("Ivo Thrust"),
    )
    .unwrap();
    let e = message(
        people::change(
            &f,
            &valve,
            "Pat Morgan",
            &key(11),
            "2026-03-05",
            &revoke("Ivo Thrust"),
        )
        .unwrap_err(),
    );
    assert!(e.contains("revoked from 2026-03-01 already"), "{e}");

    // The programme manager's own key is the anchor.
    let e = message(
        people::change(
            &programme_file(),
            &valve,
            "Pat Morgan",
            &key(11),
            "2026-03-01",
            &Change::Revoke {
                person: s("Pat Morgan"),
                public_key: key(11).public().to_hex(),
                from: s("2026-03-01"),
            },
        )
        .unwrap_err(),
    );
    assert!(e.contains("may not change their own key"), "{e}");
}

#[test]
fn a_groups_owner_registers_its_node_engineers_and_nobody_else() {
    let pr = opened(programme_file());
    let sys = systems(&pr);
    let valve = Valve::Group {
        mounted_by: &sys,
        programme: &pr,
    };
    let nina = register_change("Nina Two", "node engineer", "", &key(30));
    for (who, k) in [("Ola Sun", key(13)), ("Dee Puty", key(14))] {
        let f = people::change(&solar_file(), &valve, who, &k, "2026-03-01", &nina).unwrap();
        let solar = chain::open_group_in(&f, &sys, &pr).unwrap();
        let me = chain::whoami(
            &key(30).public().to_hex(),
            "2026-03-02",
            &pr,
            Some(&sys),
            &[&solar],
        )
        .unwrap();
        assert_eq!(me.name, "Nina Two", "registered by {who}");
    }
    // Ivo owns propulsion, not solar; Sam owns the systems' branch, not solar.
    for (who, k) in [("Ivo Thrust", key(15)), ("Sam Engel", key(12))] {
        let e = message(
            people::change(&solar_file(), &valve, who, &k, "2026-03-01", &nina).unwrap_err(),
        );
        assert!(
            e.contains("its owner Ola Sun, or their deputy, does"),
            "{who}: {e}"
        );
    }
    let e = message(
        people::change(
            &solar_file(),
            &valve,
            "Ola Sun",
            &key(13),
            "2026-03-01",
            &register_change("Mal Lory", "system engineer", "", &key(31)),
        )
        .unwrap_err(),
    );
    assert!(e.contains("not a system engineer"), "{e}");
    let e = message(
        people::change(
            &solar_file(),
            &valve,
            "Ola Sun",
            &key(13),
            "2026-03-01",
            &register_change("Sam Engel", "node engineer", "", &key(31)),
        )
        .unwrap_err(),
    );
    assert!(e.contains("registered in the programme's file"), "{e}");
}

#[test]
fn a_change_that_would_not_open_is_not_made() {
    // A file that does not open now is not changed: solar signed by Ivo.
    let pr = opened(programme_file());
    let sys = systems(&pr);
    let valve = Valve::Group {
        mounted_by: &sys,
        programme: &pr,
    };
    let mut bad = solar_file();
    bad.signatures.clear();
    let bad = signed(bad, "Ivo Thrust", &key(15));
    let e = message(
        people::change(
            &bad,
            &valve,
            "Ola Sun",
            &key(13),
            "2026-03-01",
            &register_change("Nina Two", "node engineer", "", &key(30)),
        )
        .unwrap_err(),
    );
    assert!(e.contains("by its owner Ola Sun"), "{e}");

    // A deputy programme manager, whose key START HERE does not anchor: the
    // file they would sign is refused when next opened, so nothing is made.
    let mut f = programme_file();
    f.signatures.clear();
    register(
        &mut f,
        "Pia Deputy",
        "programme manager",
        "Pat Morgan",
        &key(32),
    );
    let f = signed(f, "Pat Morgan", &key(11));
    let a = anchor();
    let e = message(
        people::change(
            &f,
            &Valve::Programme { anchor: &a },
            "Pia Deputy",
            &key(32),
            "2026-03-01",
            &register_change("Tom Thermal", "subsystem engineer", "", &key(30)),
        )
        .unwrap_err(),
    );
    assert!(
        e.contains("is changed with the key START HERE anchors"),
        "{e}"
    );
}

#[test]
fn base_version_counts_the_released_design_a_file_is_based_on() {
    let mut f = group_file("l3_solar");
    assert_eq!(f.base_version(), None, "a file from before it was kept");
    f.meta.insert(s("base_version"), s("0"));
    assert_eq!(f.base_version(), Some(0));
    f.check_meta().expect("0, before the first release");

    f.meta.insert(s("based_on"), s("2026.11.2"));
    let e = message(f.check_meta().unwrap_err());
    assert!(e.contains("0 is before the first"), "{e}");
    f.meta.insert(s("base_version"), s("7"));
    f.check_meta()
        .expect("based on the seventh released design");
    assert_eq!(f.base_version(), Some(7));

    f.meta.insert(s("based_on"), s(""));
    f.meta.insert(s("base_version"), s("1"));
    let e = message(f.check_meta().unwrap_err());
    assert!(e.contains("base_version is 1"), "{e}");
    f.meta.insert(s("based_on"), s("2026.11.2"));
    f.meta.insert(s("base_version"), s("7"));

    f.meta.insert(s("based_on"), s(""));
    let e = message(f.check_meta().unwrap_err());
    assert!(e.contains("based_on names no released design"), "{e}");
    for v in ["07", "-1", "seven", ""] {
        f.meta.insert(s("base_version"), s(v));
        let e = message(f.check_meta().unwrap_err());
        assert!(e.contains("not a whole number from 0"), "{v:?}: {e}");
    }
}

#[test]
fn the_programme_has_one_programme_manager_and_one_system_engineer() {
    let a = anchor();
    let valve = Valve::Programme { anchor: &a };
    let made = |c: &Change| {
        people::change(
            &programme_file(),
            &valve,
            "Pat Morgan",
            &key(11),
            "2026-03-01",
            c,
        )
    };
    for role in ["system engineer", "programme manager"] {
        let e = message(made(&register_change("Mal Lory", role, "", &key(30))).unwrap_err());
        assert!(e.contains(&format!("the programme has one {role}")), "{e}");
    }
    made(&register_change(
        "Sid Standin",
        "system engineer",
        "Sam Engel",
        &key(30),
    ))
    .expect("the system engineer's deputy");
    // A deputy stands in for the holder, never for another deputy.
    let e = message(
        made(&register_change(
            "Dee Two",
            "subsystem engineer",
            "Dee Puty",
            &key(30),
        ))
        .unwrap_err(),
    );
    assert!(e.contains("or is a deputy"), "{e}");
}

#[test]
fn a_key_is_registered_only_with_its_holders_proof() {
    let a = anchor();
    let valve = Valve::Programme { anchor: &a };
    // Dee offers, as Ivo's "new key", Nia's public key, which anyone can read.
    let stolen = Change::Replace {
        person: s("Ivo Thrust"),
        public_key: key(16).public().to_hex(),
        proof: people::proof(&key(14), "Ivo Thrust"),
    };
    let e = message(
        people::change(
            &programme_file(),
            &valve,
            "Pat Morgan",
            &key(11),
            "2026-03-01",
            &stolen,
        )
        .unwrap_err(),
    );
    assert!(
        e.contains("nobody registers a key that is not theirs"),
        "{e}"
    );
    // A proof made for another name does not carry over.
    let renamed = Change::Register {
        person: person("Tom Thermal", "subsystem engineer", ""),
        public_key: key(30).public().to_hex(),
        proof: people::proof(&key(30), "Somebody Else"),
    };
    let e = message(
        people::change(
            &programme_file(),
            &valve,
            "Pat Morgan",
            &key(11),
            "2026-03-01",
            &renamed,
        )
        .unwrap_err(),
    );
    assert!(
        e.contains("nobody registers a key that is not theirs"),
        "{e}"
    );
}

#[test]
fn a_release_is_never_edited() {
    let pr = opened(programme_file());
    let sys = systems(&pr);
    let mut r = solar_file();
    r.meta.insert(s("file_kind"), s(Kind::GroupRelease.name()));
    let e = people::change(
        &r,
        &Valve::Group {
            mounted_by: &sys,
            programme: &pr,
        },
        "Ola Sun",
        &key(13),
        "2026-03-01",
        &register_change("Nina Two", "node engineer", "", &key(30)),
    )
    .unwrap_err();
    let e = message(e);
    assert!(e.contains("is never edited"), "{e}");
}

#[test]
fn a_revocation_counts_from_its_day_and_a_later_change_keeps_it() {
    let a = anchor();
    let valve = Valve::Programme { anchor: &a };
    let change =
        |f: &File, at: &str, c: &Change| people::change(f, &valve, "Pat Morgan", &key(11), at, c);
    // Found on 10 March that Ivo's key was copied on 1 March.
    let f = change(
        &programme_file(),
        "2026-03-10",
        &Change::Revoke {
            person: s("Ivo Thrust"),
            public_key: key(15).public().to_hex(),
            from: s("2026-03-01"),
        },
    )
    .unwrap();
    // A new key on 12 March leaves the old one revoked from 1 March.
    let f = change(&f, "2026-03-12", &replace_change("Ivo Thrust", &key(30))).unwrap();
    let pr = opened(f);
    let e = message(
        chain::whoami(&key(15).public().to_hex(), "2026-03-05", &pr, None, &[]).unwrap_err(),
    );
    assert!(e.contains("revoked from 2026-03-01"), "{e}");

    let e = message(
        change(
            &programme_file(),
            "2026-03-10",
            &Change::Revoke {
                person: s("Ivo Thrust"),
                public_key: key(15).public().to_hex(),
                from: s("2026-04-01"),
            },
        )
        .unwrap_err(),
    );
    assert!(e.contains("after the change"), "{e}");
}
