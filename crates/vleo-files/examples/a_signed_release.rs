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
            unit: s("1"),
            state: s("achieved"),
            ..Port::default()
        });
        // One input, set by the case, and the node's whole content: its
        // method, who made it, and the cases its code must reproduce.
        release.ports.push(Port {
            block_uid: s(uid),
            direction: s("in"),
            name: s("x"),
            port_type: s("number"),
            unit: s("1"),
            lower: s("0"),
            upper: s("10"),
            state: s("decided"),
            value: s("1"),
            ..Port::default()
        });
        release.wires.push(Wire {
            to_block: s(uid),
            to_port: s("x"),
            from_ref: s("case"),
        });
        release.texts.push(Text {
            scope: s(uid),
            kind: s("pseudocode"),
            body: s("if x < 0 then\n  refuse \"x is never below zero\"\nend\nreturn x"),
        });
        release.tables.push(Tbl {
            scope: s(uid),
            path: s("declaration.csv"),
            csv: s("author,ai,date,source,checked_by\nNia Node,none,2026-03-01,,\n"),
        });
        release.tables.push(Tbl {
            scope: s(uid),
            path: s("results/isolation.csv"),
            csv: s("x [1],answer [1],tolerance,refuses,origin\n\
                    1,1,1e-9,no,hand\n0,0,1e-9,no,hand\n10,10,1e-9,no,hand\n-1,,,yes,hand\n"),
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
    release.tables.push(Tbl {
        scope: s(FILE),
        path: s("versions.csv"),
        csv: s(
            "version,date,by,believed,tested,learned,changed,risks,cost,rests_on,breaks_if\n\
                1.1,2026-02-01,Ola Sun,b,t,l,c,,,r,k\n\
                1.2,2026-03-02,Ola Sun,the two nodes answer their input,every case,nothing broke,\
                the cases at both ends,,,the cases,a case either node does not reproduce\n",
        ),
    });
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
    // And what the installed library finds in the two real solar releases: 1.0,
    // which intake refused, and 1.1, which it took.
    for v in ["1.0", "1.1"] {
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("tests/fixtures/l3_solar-{v}.vleo"));
        std::fs::copy(&fixture, dir.join(format!("l3_solar-{v}.vleo")))?;
        let rows = vleo_files::rows::encode(&raw_tables(&fixture)?);
        std::fs::write(
            dir.join(format!("expected-{v}.json")),
            page::check_content(&rows),
        )?;
    }
    // And the two compared, block by block.
    let rows = |v: &str| -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("tests/fixtures/l3_solar-{v}.vleo"));
        Ok(vleo_files::rows::encode(&raw_tables(&p)?))
    };
    std::fs::write(
        dir.join("expected-compare.json"),
        page::compare(&page::compare_request(&rows("1.0")?, &rows("1.1")?)),
    )?;
    println!("wrote {} — {answer}", dir.display());
    Ok(())
}

/// A file's tables as SQLite holds them, whatever its format: what the page's
/// own SQLite reads.
fn raw_tables(path: &std::path::Path) -> Result<Vec<Table>, Box<dyn std::error::Error>> {
    let db = rusqlite::Connection::open(path)?;
    let names: Vec<String> = db
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")?
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    let mut out = Vec::new();
    for name in names {
        let mut st = db.prepare(&format!("SELECT * FROM \"{name}\" ORDER BY rowid"))?;
        let columns: Vec<String> = st.column_names().iter().map(|c| c.to_string()).collect();
        let n = columns.len();
        let rows = st
            .query_map([], |r| {
                (0..n)
                    .map(|i| {
                        Ok(match r.get_ref(i)? {
                            rusqlite::types::ValueRef::Null => Cell::Null,
                            rusqlite::types::ValueRef::Integer(v) => Cell::Int(v),
                            rusqlite::types::ValueRef::Real(v) => Cell::Text(v.to_string()),
                            rusqlite::types::ValueRef::Text(t) => {
                                Cell::Text(String::from_utf8_lossy(t).into_owned())
                            }
                            rusqlite::types::ValueRef::Blob(b) => Cell::Blob(b.to_vec()),
                        })
                    })
                    .collect::<rusqlite::Result<Vec<Cell>>>()
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        out.push(Table {
            name,
            columns,
            rows,
        });
    }
    Ok(out)
}
