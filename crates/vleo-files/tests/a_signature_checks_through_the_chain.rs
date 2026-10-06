//! A signature checks through the chain of section 14, and nothing else does.
//!
//! docs/PLAN_1_0.md, phase C: "a release signed afresh checks through the
//! chain". A small programme is made here from nothing — its manager, system
//! engineer, one subsystem engineer and their deputy, a node engineer, and a
//! stranger, each with their own key — and a release and a design are signed
//! the way the application signs them. Then each link is broken in turn, and
//! each break is refused by name.

use std::path::Path;

use vleo_files::chain::{self, Registry, FILE};
use vleo_files::keys::SigningKey;
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::sqlite;
use vleo_files::upgrade::{self, Upgrade};
use vleo_files::ErrorKind;

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
        pm: SigningKey::from_seed([1; 32]),
        se: SigningKey::from_seed([2; 32]),
        sub: SigningKey::from_seed([3; 32]),
        deputy: SigningKey::from_seed([4; 32]),
        other_sub: SigningKey::from_seed([5; 32]),
        ne: SigningKey::from_seed([6; 32]),
        stranger: SigningKey::from_seed([7; 32]),
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

/// The programme's file, as its manager writes it, unsigned.
fn programme_unsigned(p: &People) -> File {
    let mut f = File::new(Kind::Group, APP);
    f.meta.insert(s("group_id"), s("programme"));
    f.meta.insert(s("writer"), s("Pat Morgan"));
    f.meta.insert(s("based_on"), s(""));
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
    for (uid, group, owner) in [
        ("m-solar", "l3_solar", "Ola Sun"),
        ("m-prop", "l3_prop", "Ivo Thrust"),
    ] {
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
    f
}

fn programme(p: &People) -> File {
    let mut f = programme_unsigned(p);
    let row = chain::sign(&f, FILE, "Pat Morgan", &p.pm, "2026-02-01", "ok", "").unwrap();
    f.signatures.push(row);
    f
}

fn open(p: &People) -> Registry {
    chain::open_programme(programme(p), &p.pm.public().fingerprint()).unwrap()
}

/// Solar's next release, its nodes written and signed by their node engineer,
/// not yet sealed.
fn release_unsealed(p: &People) -> File {
    let mut f = File::new(Kind::GroupRelease, APP);
    for (k, v) in [
        ("group_id", "l3_solar"),
        ("version", "1.2"),
        ("previous", "1.1"),
        ("based_on", ""),
        ("sealed_by", "Ola Sun"),
    ] {
        f.meta.insert(s(k), s(v));
    }
    register(&mut f, "Nia Node", "node engineer", "", &p.ne, "Ola Sun");
    for (uid, id) in [("b-ap", "sw_ap_design"), ("b-f107", "sw_f107_observed")] {
        f.blocks.push(Block {
            uid: s(uid),
            id: s(id),
            behaviour: s("method"),
            revision: 3,
            contract_version: 1,
            ..Block::default()
        });
        f.ports.push(Port {
            block_uid: s(uid),
            direction: s("out"),
            name: s("value"),
            port_type: s("number"),
            state: s("achieved"),
            ..Port::default()
        });
        f.texts.push(Text {
            scope: s(uid),
            kind: s("method"),
            body: s("result = 1"),
        });
        f.assignments.push(Assignment {
            block_uid: s(uid),
            person: s("Nia Node"),
            contract_version: 1,
        });
    }
    for uid in ["b-ap", "b-f107"] {
        let row = chain::sign(&f, uid, "Nia Node", &p.ne, "2026-03-01", "ok", "").unwrap();
        f.signatures.push(row);
    }
    f
}

fn seal(mut f: File, who: &str, key: &SigningKey) -> File {
    let row = chain::sign(&f, FILE, who, key, "2026-03-02", "ok", "sealed").unwrap();
    f.meta.insert(s("fingerprint"), row.digest.clone());
    f.signatures.push(row);
    f
}

fn release(p: &People) -> File {
    seal(release_unsealed(p), "Ola Sun", &p.sub)
}

fn refusals(f: &File, reg: &Registry) -> Vec<String> {
    chain::check_release(f, reg).unwrap().refused
}

#[test]
fn a_release_signed_afresh_checks_through_the_chain() {
    let p = people();
    let reg = open(&p);
    let checked = chain::check_release(&release(&p), &reg).unwrap();
    assert!(checked.holds(), "{:?}", checked.refused);
    assert_eq!(checked.signed, ["sw_ap_design", "sw_f107_observed"]);
    // And installed: written to disk, read back, it still checks.
    let dir = std::env::temp_dir().join(format!("vleo-chain-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("l3_solar-1.2.vleo");
    sqlite::write(&release(&p), &path).unwrap();
    let back = sqlite::read(Path::new(&path)).unwrap();
    assert!(chain::check_release(&back, &reg).unwrap().holds());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn the_programme_s_file_checks_only_by_the_anchored_key() {
    let p = people();
    // Another fingerprint in START HERE.
    let e = chain::open_programme(programme(&p), &p.stranger.public().fingerprint())
        .err()
        .unwrap();
    assert_eq!(e.kind(), ErrorKind::Signature);
    assert!(
        e.message().contains("not with the key START HERE anchors"),
        "{e}"
    );
    // Signed by somebody who is not its programme manager.
    let mut f = programme_unsigned(&p);
    let row = chain::sign(&f, FILE, "Sam Engel", &p.se, "2026-02-01", "ok", "").unwrap();
    f.signatures.push(row);
    let e = chain::open_programme(f, &p.pm.public().fingerprint())
        .err()
        .unwrap();
    assert!(e.message().contains("Sam Engel may not sign file"), "{e}");
    // Changed after it was signed: a key slipped in for the stranger.
    let mut f = programme(&p);
    register(
        &mut f,
        "Eve Stranger",
        "subsystem engineer",
        "",
        &p.stranger,
        "Pat Morgan",
    );
    let e = chain::open_programme(f, &p.pm.public().fingerprint())
        .err()
        .unwrap();
    assert!(e.message().contains("was given for revision"), "{e}");
    // Not signed at all.
    let e = chain::open_programme(programme_unsigned(&p), &p.pm.public().fingerprint())
        .err()
        .unwrap();
    assert!(e.message().contains("has no signature"), "{e}");
}

#[test]
fn a_seal_counts_only_from_the_group_s_own_subsystem_engineer_or_their_deputy() {
    let p = people();
    let reg = open(&p);
    // The deputy may seal for them.
    assert!(
        chain::check_release(&seal(release_unsealed(&p), "Dee Puty", &p.deputy), &reg)
            .unwrap()
            .holds()
    );
    // Another group's subsystem engineer may not.
    let r = refusals(
        &seal(release_unsealed(&p), "Ivo Thrust", &p.other_sub),
        &reg,
    );
    assert!(r[0].contains("Ivo Thrust may not sign file"), "{r:?}");
    // Nor somebody the programme's file does not register.
    let r = refusals(
        &seal(release_unsealed(&p), "Eve Stranger", &p.stranger),
        &reg,
    );
    assert!(
        r[0].contains("Eve Stranger is not registered in the programme's file"),
        "{r:?}"
    );
    // Nor the right person with a key that is not theirs.
    let r = refusals(&seal(release_unsealed(&p), "Ola Sun", &p.stranger), &reg);
    assert!(
        r[0].contains("not one the programme's file registers"),
        "{r:?}"
    );
    // And an unsealed release has no seal.
    let r = refusals(&release_unsealed(&p), &reg);
    assert!(r[0].contains("has no signature"), "{r:?}");
}

#[test]
fn a_release_changed_after_its_seal_does_not_check() {
    let p = people();
    let reg = open(&p);
    // A node's method changed: its node engineer's signature and the seal
    // were both given for something else.
    let mut f = release(&p);
    f.texts[0].body = s("result = 2");
    let r = refusals(&f, &reg);
    assert!(
        r[0].starts_with("the seal:") && r[0].contains("was given for revision"),
        "{r:?}"
    );
    // A node's signature taken out after sealing: the seal covers it.
    let mut f = release(&p);
    f.signatures.retain(|s| s.scope != "b-ap");
    let r = refusals(&f, &reg);
    assert!(r[0].starts_with("the seal:"), "{r:?}");
    // The seal's own note changed: the signature no longer checks.
    let mut f = release(&p);
    f.signatures.last_mut().unwrap().note = s("not what was signed");
    let r = refusals(&f, &reg);
    assert!(
        r[0].contains("does not check against the key it names"),
        "{r:?}"
    );
}

#[test]
fn a_node_counts_only_from_the_node_engineer_assigned_to_it() {
    let p = people();
    let reg = open(&p);
    // The node engineer's key is replaced by a stranger's signature on one
    // node, before sealing: the seal holds, the node does not.
    let mut f = release_unsealed(&p);
    f.signatures.retain(|s| s.scope != "b-ap");
    register(
        &mut f,
        "Eve Stranger",
        "node engineer",
        "",
        &p.stranger,
        "Ola Sun",
    );
    let row = chain::sign(
        &f,
        "b-ap",
        "Eve Stranger",
        &p.stranger,
        "2026-03-01",
        "ok",
        "",
    )
    .unwrap();
    f.signatures.push(row);
    let r = refusals(&seal(f, "Ola Sun", &p.sub), &reg);
    assert_eq!(r.len(), 1, "{r:?}");
    assert!(
        r[0].starts_with("sw_ap_design has no signature that counts")
            && r[0].contains("Eve Stranger may not sign b-ap"),
        "{r:?}"
    );

    // Even the group's own subsystem engineer does not sign for a node engineer.
    let mut f = release_unsealed(&p);
    f.signatures.retain(|s| s.scope != "b-ap");
    let row = chain::sign(&f, "b-ap", "Ola Sun", &p.sub, "2026-03-01", "ok", "").unwrap();
    f.signatures.push(row);
    let r = refusals(&seal(f, "Ola Sun", &p.sub), &reg);
    assert!(
        r[0].contains("Ola Sun is not registered in l3_solar's group file"),
        "{r:?}"
    );

    // A signature of a node at an earlier revision is stale.
    let mut f = release_unsealed(&p);
    f.blocks[1].revision = 4;
    let r = refusals(&seal(f, "Ola Sun", &p.sub), &reg);
    assert!(
        r[0].starts_with("sw_f107_observed") && r[0].contains("revision 3"),
        "{r:?}"
    );

    // A node signed `changes` is not signed off.
    let mut f = release_unsealed(&p);
    f.signatures.retain(|s| s.scope != "b-ap");
    let row = chain::sign(
        &f,
        "b-ap",
        "Nia Node",
        &p.ne,
        "2026-03-01",
        "changes",
        "not yet",
    )
    .unwrap();
    f.signatures.push(row);
    let r = refusals(&seal(f, "Ola Sun", &p.sub), &reg);
    assert!(r[0].contains("sw_ap_design has no signature"), "{r:?}");
}

#[test]
fn a_key_counts_from_its_registration_until_its_revocation() {
    let p = people();
    let reg = open(&p);
    let mut f = release_unsealed(&p);
    f.keys[0].revoked_from = s("2026-03-01");
    let r = refusals(&seal(f, "Ola Sun", &p.sub), &reg);
    assert_eq!(
        r.len(),
        2,
        "both nodes were signed on the day it was revoked: {r:?}"
    );
    assert!(r[0].contains("revoked from 2026-03-01"), "{r:?}");
    // A key counts only from the day it was registered.
    let mut f = release_unsealed(&p);
    f.keys[0].registered_at = s("2026-03-05");
    let r = refusals(&seal(f, "Ola Sun", &p.sub), &reg);
    assert!(r[0].contains("registered only from 2026-03-05"), "{r:?}");
    let mut f = release_unsealed(&p);
    f.keys[0].revoked_from = s("2026-03-02");
    assert!(
        refusals(&seal(f, "Ola Sun", &p.sub), &reg).is_empty(),
        "signed the day before"
    );
}

#[test]
fn a_released_design_counts_only_from_the_system_engineer() {
    let p = people();
    let reg = open(&p);
    let design = |who: &str, key: &SigningKey| {
        let mut f = File::new(Kind::Design, APP);
        for (k, v) in [
            ("version", "2026.11.1"),
            ("previous", ""),
            ("released_by", who),
            ("oldest_application", "1.0.0"),
        ] {
            f.meta.insert(s(k), s(v));
        }
        let row = chain::sign(&f, FILE, who, key, "2026-11-03", "ok", "").unwrap();
        f.signatures.push(row);
        f
    };
    chain::check_design(&design("Sam Engel", &p.se), &reg).unwrap();
    let e = chain::check_design(&design("Ola Sun", &p.sub), &reg)
        .err()
        .unwrap();
    assert!(e.message().contains("Ola Sun may not sign file"), "{e}");
    let mut changed = design("Sam Engel", &p.se);
    changed.meta.insert(s("oldest_application"), s("0.9.0"));
    let e = chain::check_design(&changed, &reg).err().unwrap();
    assert!(e.message().contains("was given for revision"), "{e}");
    // A group release is not a design, and the other way round.
    assert_eq!(
        chain::check_design(&release(&p), &reg)
            .err()
            .unwrap()
            .kind(),
        ErrorKind::WrongKind
    );
    assert_eq!(
        chain::check_release(&design("Sam Engel", &p.se), &reg)
            .err()
            .unwrap()
            .kind(),
        ErrorKind::WrongKind
    );
}

#[test]
fn a_release_from_before_1_0_checks_by_its_anchor() {
    let p = people();
    let reg = open(&p);
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/l3_solar-1.1.vleo");
    let how = Upgrade {
        app: APP,
        at: "2026-12-01T09:00:00Z",
    };
    let solar = sqlite::open(&fixture, &how).unwrap().file;
    let r = chain::check_release(&solar, &reg).unwrap();
    assert!(
        r.refused[0].contains("is from before 1.0 and is not anchored"),
        "{:?}",
        r.refused
    );
    // The programme manager anchors it in the programme's file, and signs the
    // file again: the anchor is covered by the chain like everything else.
    let anchored = |row: SignatureRow| {
        let mut f = programme_unsigned(&p);
        f.signatures.push(row);
        let sig = chain::sign(&f, FILE, "Pat Morgan", &p.pm, "2026-12-01", "ok", "").unwrap();
        f.signatures.push(sig);
        chain::open_programme(f, &p.pm.public().fingerprint()).unwrap()
    };
    let anchor = upgrade::anchor(&solar, "Pat Morgan", &p.pm, "2026-12-01").unwrap();
    let r = chain::check_release(&solar, &anchored(anchor.clone())).unwrap();
    assert!(r.holds(), "{:?}", r.refused);
    assert_eq!(r.signed.len(), solar.blocks.len());
    // Anchored by somebody who is not the programme manager, it is not.
    let wrong = upgrade::anchor(&solar, "Sam Engel", &p.se, "2026-12-01").unwrap();
    assert!(!chain::check_release(&solar, &anchored(wrong))
        .unwrap()
        .holds());
    // An anchor slipped into the programme's file after it was signed breaks
    // the programme's file itself.
    let mut f = programme(&p);
    f.signatures.push(anchor);
    assert!(chain::open_programme(f, &p.pm.public().fingerprint()).is_err());
    // And its names alone never count as keys.
    let e = chain::check_row(&solar, &solar.signatures[0])
        .err()
        .unwrap();
    assert!(
        e.message().contains("is a name, not a key's signature"),
        "{e}"
    );
}

#[test]
fn a_digest_is_the_content_not_the_order_it_was_stored_in() {
    let p = people();
    let f = release(&p);
    let mut reordered = f.clone();
    reordered.blocks.reverse();
    reordered.ports.reverse();
    reordered.texts.reverse();
    reordered.signatures.reverse();
    for scope in [FILE, "b-ap", "b-f107"] {
        assert_eq!(
            chain::digest(&f, scope).unwrap(),
            chain::digest(&reordered, scope).unwrap()
        );
    }
    // One block's content is not the other's.
    let mut other = f.clone();
    other.ports[1].unit = s("sfu");
    assert_eq!(
        chain::digest(&f, "b-ap").unwrap(),
        chain::digest(&other, "b-ap").unwrap()
    );
    assert_ne!(
        chain::digest(&f, "b-f107").unwrap(),
        chain::digest(&other, "b-f107").unwrap()
    );
    // What the file records about itself is not content.
    let mut commented = f.clone();
    commented.comments.push(Comment {
        scope: s("b-ap"),
        author: s("Nia Node"),
        at: s("2026-03-04"),
        body: s("checked again"),
        ..Comment::default()
    });
    commented.meta.insert(s("written_by_app"), s("vleo 1.0.1"));
    assert_eq!(
        chain::digest(&f, FILE).unwrap(),
        chain::digest(&commented, FILE).unwrap()
    );
    assert_eq!(
        chain::digest(&f, "nope").err().unwrap().kind(),
        ErrorKind::Malformed
    );
}
