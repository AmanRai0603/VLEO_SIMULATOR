//! A programme's file and a release, signed through the chain, written where
//! the page's check can read them.
//!
//!     cargo run -p vleo-files --example a_signed_release -- <dir>
//!
//! writes `programme.vleo`, `release.vleo`, `anchor.txt` (the fingerprint
//! START HERE would hold) and `expected.json` (what the installed library
//! answers when asked to check the release). `tools/files_check.mjs` reads the
//! files with the page's own SQLite, asks the library the page carries, and
//! holds its answer to this one.

use std::path::PathBuf;

use vleo_files::chain::{self, FILE};
use vleo_files::keys::SigningKey;
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::{page, sqlite};

const APP: &str = "vleo 1.0.0 (example)";

fn s(v: &str) -> String {
    v.to_string()
}

fn register(f: &mut File, name: &str, role: &str, key: &SigningKey) {
    f.people.push(Person {
        name: s(name),
        role: s(role),
        deputy_for: s(""),
    });
    f.keys.push(PersonKey {
        person: s(name),
        public_key: key.public().to_hex(),
        registered_at: s("2026-01-01"),
        registered_by: s("Pat Morgan"),
        revoked_from: s(""),
    });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("usage: a_signed_release <dir>")?,
    );
    std::fs::create_dir_all(&dir)?;
    let pm = SigningKey::from_seed([1; 32]);
    let sub = SigningKey::from_seed([3; 32]);
    let ne = SigningKey::from_seed([6; 32]);

    let mut programme = File::new(Kind::Group, APP);
    for (k, v) in [
        ("group_id", "programme"),
        ("writer", "Pat Morgan"),
        ("based_on", ""),
    ] {
        programme.meta.insert(s(k), s(v));
    }
    register(&mut programme, "Pat Morgan", "programme manager", &pm);
    register(&mut programme, "Ola Sun", "subsystem engineer", &sub);
    programme.blocks.push(Block {
        uid: s("m-solar"),
        id: s("m-solar"),
        behaviour: s("children"),
        ..Block::default()
    });
    programme.mounts.push(Mount {
        block_uid: s("m-solar"),
        group_id: s("l3_solar"),
        release: s(""),
    });
    programme.assignments.push(Assignment {
        block_uid: s("m-solar"),
        person: s("Ola Sun"),
        contract_version: 1,
    });
    let row = chain::sign(&programme, FILE, "Pat Morgan", &pm, "2026-02-01", "ok", "")?;
    programme.signatures.push(row);

    let mut release = File::new(Kind::GroupRelease, APP);
    for (k, v) in [
        ("group_id", "l3_solar"),
        ("version", "1.2"),
        ("previous", "1.1"),
        ("based_on", ""),
        ("sealed_by", "Ola Sun"),
    ] {
        release.meta.insert(s(k), s(v));
    }
    register(&mut release, "Nia Node", "node engineer", &ne);
    for (uid, id) in [("b-ap", "sw_ap_design"), ("b-f107", "sw_f107_observed")] {
        release.blocks.push(Block {
            uid: s(uid),
            id: s(id),
            behaviour: s("method"),
            revision: 3,
            contract_version: 1,
            ..Block::default()
        });
        release.ports.push(Port {
            block_uid: s(uid),
            direction: s("out"),
            name: s("value"),
            port_type: s("number"),
            state: s("achieved"),
            ..Port::default()
        });
        release.texts.push(Text {
            scope: s(uid),
            kind: s("method"),
            body: s("result = 1"),
        });
        release.media.push(Media {
            scope: s(uid),
            path: s("figure.png"),
            media_type: s("image/png"),
            sha256: s(""),
            bytes: vec![0x89, b'P', b'N', b'G', 0, 255],
        });
        release.assignments.push(Assignment {
            block_uid: s(uid),
            person: s("Nia Node"),
            contract_version: 1,
        });
    }
    for uid in ["b-ap", "b-f107"] {
        let row = chain::sign(&release, uid, "Nia Node", &ne, "2026-03-01", "ok", "")?;
        release.signatures.push(row);
    }
    let seal = chain::sign(
        &release,
        FILE,
        "Ola Sun",
        &sub,
        "2026-03-02",
        "ok",
        "sealed",
    )?;
    release.meta.insert(s("fingerprint"), seal.digest.clone());
    release.signatures.push(seal);

    let anchor = pm.public().fingerprint();
    for name in ["programme.vleo", "release.vleo"] {
        let _ = std::fs::remove_file(dir.join(name));
    }
    sqlite::write(&programme, &dir.join("programme.vleo"))?;
    sqlite::write(&release, &dir.join("release.vleo"))?;
    std::fs::write(dir.join("anchor.txt"), &anchor)?;
    // What the installed library answers, from the files as written.
    let back_p = sqlite::read(&dir.join("programme.vleo"))?;
    let back_r = sqlite::read(&dir.join("release.vleo"))?;
    let answer = page::check_release(&page::check_release_request(&anchor, &back_p, &back_r));
    std::fs::write(dir.join("expected.json"), &answer)?;
    println!("wrote {} — {answer}", dir.display());
    Ok(())
}
