//! A node is written in the working copy through the program
//! (docs/briefs/F3.md, F3-g2a): `POST /v1/node/<id>/edit`, and its dry run.
//!
//! The drive is the Node workspace test's: Test Nia assigned solar's
//! `sw_activity_band` as solar 1.1's release holds it, Test Ivo a node
//! engineer of solar assigned nothing. Through the routes:
//!
//! - an edit with no key open, or without the launch's word, is refused;
//! - to Nia the file says it can be written here, and what it says now: its
//!   question, the explanation's three texts, its method, its cases and its
//!   declaration, as the library reads them;
//! - a dry run writes nothing: a question with no record says why a save
//!   would be refused; a method that breaks the case at 130 sfu says so,
//!   case by case, and as what signing would find;
//! - saved, with its record, it is revision 2, written by Nia, the record a
//!   row numbered 2; signed, it holds; Dee writes it as Nia's deputy;
//! - a case whose origin is the code's own, a declaration naming an
//!   assistant, a file changed since it was opened, Ivo, and Ola (assigned it,
//!   with no key solar's file counts), are each refused, with nothing written;
//! - today's design is as it was: the node's page and its fragment are the
//!   same bytes before, during and after (rule 5).

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use vleo_files::chain::{self, FILE};
use vleo_files::keys::{SigningKey, MIN_ITERATIONS};
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::sqlite;

const APP: &str = "vleo 1.0.0 (test)";
const PASSPHRASE: &str = "a long enough passphrase";
const UID: &str = "sw_activity_band";
const NODE: &str = "groups/l3_solar/nodes/sw_activity_band.vnode";
const EDIT: &str = "/v1/node/sw_activity_band/edit";
const SIGN: &str = "/v1/node/sw_activity_band/sign";

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
    // Test Sue (77), Ola's deputy.
    register(
        &mut programme,
        "Test Sue",
        "subsystem engineer",
        &key(77),
        "Test Pat",
    );
    programme.people.last_mut().unwrap().deputy_for = s("Test Ola");
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
    // A second writer of the same block; an assignment that names nobody,
    // which makes nobody its writer; and Ola, whom only the programme's file
    // registers, so no key of hers counts here.
    register(
        &mut solar,
        "Test Pip",
        "node engineer",
        &key(78),
        "Test Ola",
    );
    for person in ["Test Pip", "", "Test Ola"] {
        solar.assignments.push(Assignment {
            block_uid: s(UID),
            person: s(person),
            contract_version: 1,
        });
    }
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
    write_key(keys, "test-ola.vkey", "Test Ola", &key(73));
    write_key(keys, "test-sue.vkey", "Test Sue", &key(77));
    write_key(keys, "test-pip.vkey", "Test Pip", &key(78));
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

#[test]
fn a_node_is_written_in_the_copy() {
    let scratch = std::env::temp_dir().join(format!("vleo-node-edit-{}", std::process::id()));
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
    let port = vleo_server::serve(Some(root()), 18964, false, true).expect("no server");
    let copied = post(port, "/v1/copy", &[("drive", &drv.display().to_string())]);
    assert!(copied.contains("\"ok\":true"), "{copied}");
    let file = || {
        let r = ask(port, "GET", "/v1/node/sw_activity_band/file", &[], false);
        assert!(r.contains("\"ok\":true"), "{r}");
        r
    };
    let bytes = || std::fs::read(copy.join(NODE)).unwrap();
    // Today's design, as its page and fragment: never moved by any of this.
    let today = || {
        (
            ask(port, "GET", "/v1/node/sw_activity_band", &[], false),
            ask(port, "GET", "/v1/fragment/sw_activity_band", &[], false),
        )
    };
    let was = today();
    let record: [(&str, &str); 6] = [
        (
            "why.believed",
            "The question named the band, not what it is for.",
        ),
        ("why.tested", "Read beside the drag nodes that use it."),
        ("why.learned", "They read it as the level of activity."),
        ("why.changed", "The question asks for the level."),
        ("why.rests_on", "NOAA's four F10.7 levels."),
        ("why.breaks_if", "A node reads it as a flux."),
    ];
    let question = "Which level of solar activity does this F10.7 fall in?";

    // No key, or no word: refused.
    let r = post(
        port,
        EDIT,
        &[("opened", field(&file(), "opened")), ("question", question)],
    );
    assert!(field(&r, "message").starts_with("no key is open"), "{r}");
    assert!(open_key(port, "test-nia.vkey").contains("\"open\":true"));
    let r = ask(
        port,
        "POST",
        EDIT,
        &[("opened", field(&file(), "opened")), ("question", question)],
        false,
    );
    assert!(
        r.contains("does not carry the word this launch gave its page"),
        "{r}"
    );

    // What the file says, as the library reads it.
    let f = file();
    assert!(
        f.contains("\"shape\":\"release\",\"editable\":true,\"edit_why_not\":\"\""),
        "{f}"
    );
    assert!(
        f.contains("\"question\":\"Which activity band does this F10.7 fall in?\""),
        "{f}"
    );
    assert!(
        f.contains("\"declared\":{\"author\":\"Aman Rai\",\"ai\":\"transcribed\""),
        "{f}"
    );
    assert!(field(&f, "simply").len() > 20, "{f}");
    let method = {
        let k = "\"method_text\":";
        let at = f.find(k).unwrap() + k.len();
        let s: String = f[at..].chars().take_while(|_| true).collect();
        let end = s[1..].find("\",\"case_csv\"").unwrap() + 1;
        s[1..end].replace("\\n", "\n").replace("\\\"", "\"")
    };
    assert!(method.contains("if f107 >= 130 then"), "{method}");

    // A dry run writes nothing: a question with no record, said why.
    let before = bytes();
    let r = post(
        port,
        EDIT,
        &[
            ("opened", field(&f, "opened")),
            ("dry_run", "1"),
            ("question", question),
        ],
    );
    assert!(
        r.contains("\"ok\":false,\"dry_run\":true,\"written\":false")
            && r.contains("does not say which belief broke"),
        "{r}"
    );
    assert!(r.contains("\"needs_record\":true"), "{r}");
    assert_eq!(bytes(), before);
    // A method that breaks the case at 130 sfu, case by case.
    let wrong = method.replace("if f107 >= 130 then", "if f107 >= 135 then");
    let mut fields: Vec<(&str, &str)> = vec![
        ("opened", field(&f, "opened")),
        ("dry_run", "1"),
        ("method_text", &wrong),
        ("declared.author", "Test Nia"),
        ("declared.ai", "none"),
    ];
    fields.extend(record);
    let r = post(port, EDIT, &fields);
    assert!(
        r.contains("\"ok\":true,\"dry_run\":true,\"written\":false"),
        "{r}"
    );
    assert!(r.contains("does not reproduce «130 sfu exactly"), "{r}");
    assert!(
        r.contains(
            "\"label\":\"130 sfu exactly — elevated begins.\",\"refuses\":false,\"agrees\":false"
        ),
        "{r}"
    );
    assert_eq!(r.matches("\"agrees\":true").count(), 9, "{r}");
    assert_eq!(bytes(), before);
    assert_eq!(today(), was, "a dry run moves nothing of today's design");

    // Refused, with nothing written: a case that is the code's own, an
    // assistant's declaration, and a file changed since it was opened.
    let cases = {
        let k = "\"case_csv\":\"";
        let at = f.find(k).unwrap() + k.len();
        let end = f[at..].find("\",\"declared\"").unwrap();
        f[at..at + end].replace("\\n", "\n").replace("\\\"", "\"")
    };
    let bad = cases.replacen(",paper,", ",self-snapshot,", 1);
    let r = post(
        port,
        EDIT,
        &[
            ("opened", field(&f, "opened")),
            ("case_csv", &bad),
            ("declared.author", "Test Nia"),
            ("declared.ai", "none"),
        ],
    );
    assert!(r.contains("never comes from the code under test"), "{r}");
    let r = post(
        port,
        EDIT,
        &[
            ("opened", field(&f, "opened")),
            ("case_csv", &cases.replacen(",paper,", ",hand,", 1)),
            ("declared.author", "Claude"),
            ("declared.ai", "none"),
        ],
    );
    assert!(
        r.contains("an assistant") && r.contains("declaration does not hold"),
        "{r}"
    );
    let r = post(
        port,
        EDIT,
        &[("opened", &"0".repeat(64)), ("question", question)],
    );
    assert!(r.contains("\"ok\":false,\"conflict\":true"), "{r}");
    assert_eq!(bytes(), before);

    // Saved, with its record: revision 2, written by Nia; then signed.
    let mut fields: Vec<(&str, &str)> =
        vec![("opened", field(&f, "opened")), ("question", question)];
    fields.extend(record);
    let r = post(port, EDIT, &fields);
    assert!(
        r.contains("\"ok\":true,\"dry_run\":false,\"written\":true")
            && r.contains("\"revision\":2"),
        "{r}"
    );
    let f = file();
    assert_eq!(field(&f, "revision"), "2");
    assert_eq!(field(&f, "writer"), "Test Nia");
    assert!(f.contains(&format!("\"question\":\"{question}\"")), "{f}");
    assert!(f.contains("\"versions\":\"version,date,by,"), "{f}");
    assert!(
        f.contains(",Test Nia,\\\"The question named the band"),
        "{f}"
    );
    let r = post(port, SIGN, &[("opened", field(&f, "opened"))]);
    assert!(r.contains("\"ok\":true"), "{r}");
    assert_eq!(today(), was, "a save moves nothing of today's design");
    post(port, "/v1/key/close", &[]);

    // Dee, Nia's deputy: theirs to write as the deputy.
    assert!(open_key(port, "test-dee.vkey").contains("\"open\":true"));
    let f = file();
    assert!(f.contains("\"editable\":true"), "{f}");
    let r = post(
        port,
        EDIT,
        &[
            ("opened", field(&f, "opened")),
            ("dry_run", "1"),
            ("explain_wrong", "Band 4 is not a storm."),
        ],
    );
    assert!(
        r.contains("\"ok\":true,\"dry_run\":true") && r.contains("\"checked\":true"),
        "{r}"
    );
    post(port, "/v1/key/close", &[]);

    // Ivo: not Ivo's to write, for a save and a dry run alike.
    assert!(open_key(port, "test-ivo.vkey").contains("\"open\":true"));
    let f = file();
    assert!(f.contains("\"editable\":false"), "{f}");
    let before = bytes();
    for dry in ["1", ""] {
        let r = post(
            port,
            EDIT,
            &[
                ("opened", field(&f, "opened")),
                ("dry_run", dry),
                ("question", "Another."),
            ],
        );
        assert!(r.contains("one writer per file"), "{r}");
    }
    assert_eq!(bytes(), before);
    post(port, "/v1/key/close", &[]);

    // Ola: assigned it, and registered only by the programme's file, so no
    // key of Ola's counts in solar's: refused, for a dry run too.
    assert!(open_key(port, "test-ola.vkey").contains("\"open\":true"));
    let f = file();
    for dry in ["1", ""] {
        let r = post(
            port,
            EDIT,
            &[
                ("opened", field(&f, "opened")),
                ("dry_run", dry),
                ("question", "Another."),
            ],
        );
        assert!(
            r.contains("\"ok\":false") && r.contains("Test Ola") && r.contains("registered"),
            "{r}"
        );
    }
    assert_eq!(bytes(), before);
    let _ = std::fs::remove_dir_all(&scratch);
}
