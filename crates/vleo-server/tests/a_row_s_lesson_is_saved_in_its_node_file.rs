//! A row's lesson saved in its node file through the program's routes, as the
//! node's next revision, and signed with it (docs/briefs/F3.md, F3-e;
//! docs/LESSONS.md).
//!
//! The drive and the node are the signing test's (`a_node_is_signed_in_the_
//! copy.rs`): Test Nia assigned solar's `sw_activity_band`, Test Ivo assigned
//! nothing. The lesson is the example the documentation keeps. Through the
//! routes: Ivo is refused by name; a lesson whose widget names a row that is
//! not there, or that carries markup, is refused with the reason and nothing
//! written; Nia's is saved as revision 2, and a save from what she opened
//! before is stopped; she signs the node, lesson and all. Refused without the
//! launch's word and without a key.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use vleo_files::chain::{self, Registry, FILE};
use vleo_files::keys::{SigningKey, MIN_ITERATIONS};
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::sqlite;

const APP: &str = "vleo 1.0.0 (test)";
const PASSPHRASE: &str = "a long enough passphrase";
const UID: &str = "sw_activity_band";
const NODE: &str = "groups/l3_solar/nodes/sw_activity_band.vnode";
const SIGN: &str = "/v1/node/sw_activity_band/sign";
const LESSON: &str = "/v1/node/sw_activity_band/lesson";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn s(v: &str) -> String {
    v.to_string()
}

fn key(seed: u8) -> SigningKey {
    SigningKey::from_seed([seed; 32])
}

fn enc(v: &str) -> String {
    v.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

fn ask(port: u16, method: &str, path: &str, pairs: &[(&str, &str)], with_word: bool) -> String {
    let mut body: Vec<String> = pairs
        .iter()
        .map(|(k, v)| format!("{k}={}", enc(v)))
        .collect();
    if method == "POST" && with_word {
        body.insert(0, format!("session={}", vleo_server::session()));
    }
    let body = body.join("&");
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    write!(
        s,
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .unwrap();
    let mut raw = Vec::new();
    s.read_to_end(&mut raw).unwrap();
    let text = String::from_utf8_lossy(&raw).into_owned();
    text.split_once("\r\n\r\n")
        .map(|(_, b)| b.to_string())
        .unwrap_or(text)
}

fn post(port: u16, path: &str, pairs: &[(&str, &str)]) -> String {
    ask(port, "POST", path, pairs, true)
}

fn field<'a>(json: &'a str, key: &str) -> &'a str {
    let k = format!("\"{key}\":\"");
    let at = json
        .find(&k)
        .unwrap_or_else(|| panic!("no {key} in {json}"))
        + k.len();
    &json[at..at + json[at..].find('"').unwrap()]
}

/// Standard base64, as the program reads a file sent to it.
fn b64(bytes: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for c in bytes.chunks(3) {
        let n = (c[0] as u32) << 16
            | (*c.get(1).unwrap_or(&0) as u32) << 8
            | *c.get(2).unwrap_or(&0) as u32;
        for k in 0..4 {
            if k <= c.len() {
                out.push(A[(n >> (18 - 6 * k) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// `lesson` as a filled form carries it: escaped, in the form's textarea.
fn filled(form: &str, lesson: &str) -> String {
    let open = "id=\"vleo-lesson\">";
    let a = form.find(open).unwrap() + open.len();
    let b = form[a..].find("</textarea>").unwrap() + a;
    let escaped = lesson
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;");
    format!("{}{escaped}{}", &form[..a], &form[b..])
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

fn write_key(dir: &Path, file: &str, person: &str, k: &SigningKey) {
    let locked = k
        .lock(person, PASSPHRASE, [7; 16], [7; 12], MIN_ITERATIONS)
        .unwrap();
    let mut f = File::new(Kind::Key, APP);
    f.meta.insert(s("person"), s(person));
    f.locked_keys.push(locked.to_row());
    sqlite::write(&f, &dir.join(file)).unwrap();
}

/// The node as solar's 1.1 release holds it, in a node file of its own.
fn node() -> File {
    let how = vleo_files::upgrade::Upgrade {
        app: APP,
        at: "2026-10-01",
    };
    let rel = sqlite::open(
        &root().join("crates/vleo-files/tests/fixtures/l3_solar-1.1.vleo"),
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
        ("writer", "Test Nia"),
    ] {
        n.meta.insert(s(k), s(v));
    }
    n.blocks = vec![rel.blocks.iter().find(|b| b.uid == UID).unwrap().clone()];
    // One revision: the file's and its block's.
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

/// Solar's group file in the copy, as the chain reads it for this test.
fn solar(copy: &Path) -> Registry {
    Registry {
        from: s("l3_solar's group file"),
        file: sqlite::read(&copy.join("groups/l3_solar/l3_solar.vgroup")).unwrap(),
    }
}

// Test Pat (71), programme manager; Test Sam (72), system engineer; Test Ola
// (73), who owns solar; Test Nia (74), assigned sw_activity_band; Test Ivo
// (75), a node engineer of solar assigned nothing; Test Dee (76), Nia's
// deputy.
fn drive(at: &Path, keys: &Path) {
    let mut programme = group("programme", "Test Pat");
    register(
        &mut programme,
        "Test Pat",
        "programme manager",
        &key(71),
        "Test Pat",
    );
    register(
        &mut programme,
        "Test Sam",
        "system engineer",
        &key(72),
        "Test Pat",
    );
    register(
        &mut programme,
        "Test Ola",
        "subsystem engineer",
        &key(73),
        "Test Pat",
    );
    mount(&mut programme, "root", "systems", "Test Sam");
    std::fs::create_dir_all(at.join("groups/programme")).unwrap();
    sqlite::write(
        &signed(programme, "Test Pat", &key(71)),
        &at.join("groups/programme/programme.vgroup"),
    )
    .unwrap();
    let mut systems = group("systems", "Test Sam");
    mount(
        &mut systems,
        "sys_space_environment",
        "l3_solar",
        "Test Ola",
    );
    std::fs::create_dir_all(at.join("groups/systems")).unwrap();
    sqlite::write(
        &signed(systems, "Test Sam", &key(72)),
        &at.join("groups/systems/systems.vgroup"),
    )
    .unwrap();
    let mut solar = group("l3_solar", "Test Ola");
    register(
        &mut solar,
        "Test Nia",
        "node engineer",
        &key(74),
        "Test Ola",
    );
    register(
        &mut solar,
        "Test Ivo",
        "node engineer",
        &key(75),
        "Test Ola",
    );
    register(
        &mut solar,
        "Test Dee",
        "node engineer",
        &key(76),
        "Test Ola",
    );
    solar.people.last_mut().unwrap().deputy_for = s("Test Nia");
    solar.assignments.push(Assignment {
        block_uid: s(UID),
        person: s("Test Nia"),
        contract_version: 1,
    });
    std::fs::create_dir_all(at.join("groups/l3_solar/nodes")).unwrap();
    sqlite::write(
        &signed(solar, "Test Ola", &key(73)),
        &at.join("groups/l3_solar/l3_solar.vgroup"),
    )
    .unwrap();
    sqlite::write(&node(), &at.join(NODE)).unwrap();
    std::fs::write(
        at.join("START HERE.md"),
        format!(
            "# START HERE: an invented drive\n\nThe programme manager's key:\n\n    {}\n",
            key(71).public().fingerprint()
        ),
    )
    .unwrap();
    std::fs::create_dir_all(keys).unwrap();
    write_key(keys, "test-nia.vkey", "Test Nia", &key(74));
    write_key(keys, "test-ivo.vkey", "Test Ivo", &key(75));
    write_key(keys, "test-dee.vkey", "Test Dee", &key(76));
}

fn open_key(port: u16, file: &str) -> String {
    post(
        port,
        "/v1/key/open",
        &[
            ("file", file),
            ("passphrase", PASSPHRASE),
            ("accept_anchor", &key(71).public().fingerprint()),
        ],
    )
}

fn example() -> String {
    std::fs::read_to_string(root().join("docs/examples/orbit_velocity.lesson.toml")).unwrap()
}

#[test]
fn a_row_s_lesson_is_saved_in_its_node_file_and_signed_with_it() {
    let scratch = std::env::temp_dir().join(format!("vleo-node-lesson-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    let (drv, keys, copy) = (
        scratch.join("drive"),
        scratch.join("keys"),
        scratch.join("copy"),
    );
    drive(&drv, &keys);
    std::env::set_var("VLEO_KEYS", &keys);
    std::env::set_var("VLEO_COPY", &copy);
    std::env::set_var("VLEO_CASE", scratch.join("case.csv"));
    std::env::set_var("VLEO_RESULTS", scratch.join("results"));
    std::env::set_var("VLEO_LOG", scratch.join("log"));
    std::env::set_var("VLEO_DATA", scratch.join("data"));
    std::env::remove_var("VLEO_DRIVE");
    std::env::remove_var("VLEO_DESIGN");
    let port = vleo_server::serve(Some(root()), 18959, false, true).expect("no server");
    let copied = post(port, "/v1/copy", &[("drive", &drv.display().to_string())]);
    assert!(copied.contains("\"ok\":true"), "{copied}");
    let opened = |port: u16| {
        let r = ask(port, "GET", "/v1/node/sw_activity_band/file", &[], false);
        assert!(r.contains("\"ok\":true"), "{r}");
        field(&r, "opened").to_string()
    };
    let saving = |port: u16, opened: &str, lesson: &str| {
        post(port, LESSON, &[("opened", opened), ("lesson", lesson)])
    };

    let no_key = saving(port, &opened(port), &example());
    assert!(no_key.contains("no key is open"), "{no_key}");
    let bare = ask(
        port,
        "POST",
        LESSON,
        &[("opened", &opened(port)), ("lesson", &example())],
        false,
    );
    assert!(bare.contains("does not carry the word"), "{bare}");

    // Ivo is a node engineer of solar, and not assigned the node.
    assert!(open_key(port, "test-ivo.vkey").contains("\"open\":true"));
    let ivo = saving(port, &opened(port), &example());
    assert!(
        ivo.contains("Test Ivo is not assigned sw_activity_band"),
        "{ivo}"
    );
    post(port, "/v1/key/close", &[]);

    // Refused with the reason, and nothing written: a widget naming a row
    // that is not there, markup, and what is not a lesson at all.
    assert!(open_key(port, "test-nia.vkey").contains("\"open\":true"));
    let before = std::fs::read(copy.join(NODE)).unwrap();
    for (what, lesson, says) in [
        (
            "a row that is not there",
            example().replacen(
                "inputs = [\"orbit_altitude\"]",
                "inputs = [\"no_such_row\"]",
                1,
            ),
            "input 'no_such_row' is not a row",
        ),
        (
            "markup",
            example().replacen(
                "How fast the spacecraft",
                "<b>How</b> fast the spacecraft",
                1,
            ),
            "is markup",
        ),
        ("not TOML", s("[lesson\n"), "does not read"),
    ] {
        let r = saving(port, &opened(port), &lesson);
        assert!(
            r.contains("\"ok\":false") && r.to_lowercase().contains("nothing is saved"),
            "{what}: {r}"
        );
        assert!(r.contains(says), "{what}: {r}");
        assert_eq!(std::fs::read(copy.join(NODE)).unwrap(), before, "{what}");
    }

    // A lesson form made for another row is not this row's lesson.
    let form = ask(port, "GET", "/v1/lesson-form/orbit_velocity", &[], false);
    assert!(
        form.contains("id=\"vleo-lesson\""),
        "{}",
        &form[..form.len().min(300)]
    );
    let r = saving(port, &opened(port), &form);
    assert!(
        r.contains("the form is for 'orbit_velocity', not 'sw_activity_band'. Nothing is saved."),
        "{r}"
    );
    assert_eq!(std::fs::read(copy.join(NODE)).unwrap(), before);

    // Nor is one that names a row that is not there saved by the plain save,
    // or signed, written in the file by hand.
    let mut bad = sqlite::read(&copy.join(NODE)).unwrap();
    bad.texts.push(Text {
        scope: s(UID),
        kind: s(vleo_files::convert::LESSON),
        body: example().replacen(
            "inputs = [\"orbit_altitude\"]",
            "inputs = [\"no_such_row\"]",
            1,
        ),
    });
    let at = std::env::temp_dir().join(format!("vleo-lesson-bad-{}.vnode", std::process::id()));
    sqlite::write(&bad, &at).unwrap();
    let r = post(
        port,
        "/v1/node/sw_activity_band/save",
        &[
            ("opened", &opened(port)),
            ("file", &b64(&std::fs::read(&at).unwrap())),
        ],
    );
    assert!(
        r.contains("input 'no_such_row' is not a row") && r.contains("Nothing is saved."),
        "{r}"
    );
    assert_eq!(std::fs::read(copy.join(NODE)).unwrap(), before);
    sqlite::write(&bad, &copy.join(NODE)).unwrap();
    let r = post(port, SIGN, &[("opened", &opened(port))]);
    assert!(
        r.contains("input 'no_such_row' is not a row") && r.contains("Nothing is signed."),
        "{r}"
    );
    std::fs::write(copy.join(NODE), &before).unwrap();
    let _ = std::fs::remove_file(&at);

    // Nia's lesson, sent as a filled form, is saved word for word as the
    // node's next revision.
    let form = ask(port, "GET", "/v1/lesson-form/sw_activity_band", &[], false);
    let first = opened(port);
    let r = saving(port, &first, &filled(&form, &example()));
    assert!(r.contains("\"ok\":true"), "{r}");
    assert!(r.contains("\"revision\":2"), "{r}");
    assert_eq!(field(&r, "title"), "How fast a very low orbit goes");
    let f = sqlite::read(&copy.join(NODE)).unwrap();
    let held: Vec<&Text> = f
        .texts
        .iter()
        .filter(|t| t.scope == UID && t.kind == vleo_files::convert::LESSON)
        .collect();
    assert_eq!(held.len(), 1);
    assert_eq!(held[0].body, example());
    assert_eq!(f.meta.get("writer").map(String::as_str), Some("Test Nia"));
    // From what she opened before, it is a second writer.
    let stale = saving(port, &first, &example());
    assert!(stale.contains("\"conflict\":true"), "{stale}");
    // Saved again, it replaces the one there.
    let again = example().replacen("very low orbit", "very low orbit, again", 1);
    let r = saving(port, &opened(port), &again);
    assert!(r.contains("\"revision\":3"), "{r}");
    let f = sqlite::read(&copy.join(NODE)).unwrap();
    let held: Vec<&Text> = f
        .texts
        .iter()
        .filter(|t| t.scope == UID && t.kind == vleo_files::convert::LESSON)
        .collect();
    assert_eq!(held.len(), 1);
    assert_eq!(held[0].body, again);

    // She signs the node, lesson and all; a lesson changed after is not signed.
    let r = post(port, SIGN, &[("opened", &opened(port))]);
    assert!(r.contains("\"ok\":true"), "{r}");
    let signed = sqlite::read(&copy.join(NODE)).unwrap();
    chain::check_node(&signed, UID, &solar(&copy)).expect("signed, lesson and all");
    let mut edited = signed.clone();
    for t in edited
        .texts
        .iter_mut()
        .filter(|t| t.kind == vleo_files::convert::LESSON)
    {
        t.body.push('\n');
    }
    assert!(chain::check_node(&edited, UID, &solar(&copy)).is_err());
    post(port, "/v1/key/close", &[]);

    // Dee, her deputy, saves one for her.
    assert!(open_key(port, "test-dee.vkey").contains("\"open\":true"));
    let r = saving(port, &opened(port), &example());
    assert!(
        r.contains("\"ok\":true") && r.contains("written by Test Dee"),
        "{r}"
    );
    post(port, "/v1/key/close", &[]);
    let _ = std::fs::remove_dir_all(&scratch);
}
