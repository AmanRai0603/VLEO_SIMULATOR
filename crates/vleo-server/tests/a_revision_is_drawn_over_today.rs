//! A node's page as the copy's revision of it would make it, over today's
//! design (docs/briefs/F3.md, F3-g2b): `GET /v1/node/<id>/live`.
//!
//! The drive is the Node workspace test's: Test Nia assigned solar's
//! `sw_activity_band` as solar 1.1's release holds it, which today's design
//! already holds. Through the routes:
//!
//! - with no copy, it is refused;
//! - copied, the revision changes nothing on the page, and the page is
//!   today's fragment, byte for byte;
//! - a new question, saved as revision 2, still changes nothing on the page,
//!   and the answer says the question is not taken: a release does not bring
//!   a node's question into today's design;
//! - a method with a line of Nia's, saved with its declaration and record as
//!   revision 3, is drawn: the page carries the line and its version record,
//!   and today's fragment does not;
//! - a method withheld (a risk moved that nobody registered) beside a cases
//!   change that would apply alone, saved as revision 4: nothing is drawn,
//!   since today's design refuses such a release whole;
//! - a method that no longer reproduces its cases, saved as revision 5: its
//!   checks refuse it, by the case, and nothing is drawn; nor is a
//!   transcription nobody checked, written in the file by hand;
//! - a file that says it is another group's than its folder, a node of
//!   today's design with no file in the copy, and a node file of a node
//!   today's design does not hold, are each refused, by name;
//! - today's design is as it was: the node's page and its fragment are the
//!   same bytes before, during and after, and it runs as it did (rule 5).

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
const LIVE: &str = "/v1/node/sw_activity_band/live";

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

/// `s` as a JSON string's body, as the program writes it.
fn json_text(s: &str) -> String {
    let mut o = String::new();
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o
}

/// The page the answer draws, as the program sent it.
fn html(r: &str) -> &str {
    let k = "\"html\":\"";
    let at = r.find(k).unwrap_or_else(|| panic!("no html in {r}")) + k.len();
    &r[at..r.len() - 2]
}

/// The answer without its page, to read what it says.
fn said(r: &str) -> &str {
    &r[..r.find("\"html\"").unwrap_or(r.len())]
}

#[test]
fn a_revision_is_drawn_over_today() {
    let scratch = std::env::temp_dir().join(format!("vleo-node-live-{}", std::process::id()));
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
    let port = vleo_server::serve(Some(root()), 18966, false, true).expect("no server");
    let live = || ask(port, "GET", LIVE, &[], false);
    let file = || {
        let r = ask(port, "GET", "/v1/node/sw_activity_band/file", &[], false);
        assert!(r.contains("\"ok\":true"), "{r}");
        r
    };
    // Today's design, as its page and fragment: never moved by any of this.
    let today = || {
        (
            ask(port, "GET", "/v1/node/sw_activity_band", &[], false),
            ask(port, "GET", "/v1/fragment/sw_activity_band", &[], false),
        )
    };
    let was = today();
    let fragment = json_text(&was.1);
    let run = || ask(port, "GET", "/v1/run?node=sw_activity_band", &[], false);
    let ran = run();
    assert!(ran.contains("\"ok\":true"), "{ran}");

    // With no copy: refused.
    let r = live();
    assert!(
        r.contains("\"ok\":false") && r.contains("copy it from the drive first"),
        "{r}"
    );
    let copied = post(port, "/v1/copy", &[("drive", &drv.display().to_string())]);
    assert!(copied.contains("\"ok\":true"), "{copied}");

    // As copied: it changes nothing, and the page is today's, byte for byte.
    let r = live();
    assert!(
        said(&r).contains("\"revision\":\"1\",\"writer\":\"Test Nia\",\"not_taken\":[],\"checks\":[],\"changes\":[],\"withheld\":[],\"drawn\":true,\"changes_nothing\":true"),
        "{}",
        said(&r)
    );
    assert!(
        said(&r).contains("changes nothing on today's page"),
        "{}",
        said(&r)
    );
    assert_eq!(html(&r), fragment);

    // A new question, saved as revision 2: not taken, and said so.
    assert!(open_key(port, "test-nia.vkey").contains("\"open\":true"));
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
    let f = file();
    let mut fields: Vec<(&str, &str)> =
        vec![("opened", field(&f, "opened")), ("question", question)];
    fields.extend(record);
    let r = post(port, EDIT, &fields);
    assert!(
        r.contains("\"written\":true") && r.contains("\"revision\":2"),
        "{r}"
    );
    let r = live();
    assert!(
        said(&r).contains(&format!(
            "\"revision\":\"2\",\"writer\":\"Test Nia\",\"not_taken\":[{{\"what\":\"its question\",\"yours\":\"{question}\",\"page\":\"Which activity band does this F10.7 fall in?\"}}],\"checks\":[],\"changes\":[],"
        )),
        "{}",
        said(&r)
    );
    assert!(
        said(&r).contains("\"changes_nothing\":true"),
        "{}",
        said(&r)
    );
    assert_eq!(html(&r), fragment);

    // A method with a line of Nia's, saved as revision 3: drawn.
    let f = file();
    let method = {
        let k = "\"method_text\":\"";
        let at = f.find(k).unwrap() + k.len();
        let end = f[at..].find("\",\"case_csv\"").unwrap();
        f[at..at + end].replace("\\n", "\n").replace("\\\"", "\"")
    };
    let line = "# Nia: the four levels are NOAA's, read at 130, 150 and 200 sfu.";
    let mine = format!("{line}\n{method}");
    let mut fields: Vec<(&str, &str)> = vec![
        ("opened", field(&f, "opened")),
        ("method_text", &mine),
        ("declared.author", "Test Nia"),
        ("declared.ai", "none"),
    ];
    let why: [(&str, &str); 6] = [
        (
            "why.believed",
            "The method said nothing of where its edges come from.",
        ),
        ("why.tested", "Read beside NOAA's table of the four levels."),
        ("why.learned", "Its edges are NOAA's, and nothing said so."),
        ("why.changed", "A line says where the edges come from."),
        ("why.rests_on", "NOAA's four F10.7 levels."),
        ("why.breaks_if", "NOAA moves an edge."),
    ];
    fields.extend(why);
    let r = post(port, EDIT, &fields);
    assert!(
        r.contains("\"written\":true") && r.contains("\"revision\":3"),
        "{r}"
    );
    let r = live();
    assert!(
        said(&r).contains("\"changes\":[{\"what\":\"method_text\""),
        "{}",
        said(&r)
    );
    assert!(said(&r).contains("de-risking · version"), "{}", said(&r));
    assert!(
        said(&r).contains(
            "\"withheld\":[],\"record\":\"Its record of which belief broke is the node's own"
        ) && said(&r).contains("\"drawn\":true,\"changes_nothing\":false"),
        "{}",
        said(&r)
    );
    assert!(
        said(&r).contains("Your revision 3, from your copy, over today's design: not sealed, not in today's design, and not run."),
        "{}",
        said(&r)
    );
    let page = html(&r);
    assert_ne!(page, fragment);
    assert!(
        page.contains("Nia: the four levels are NOAA"),
        "the line is on the page"
    );
    assert!(page.contains("Its edges are NOAA"), "and its record");
    assert!(!fragment.contains("Nia: the four levels are NOAA"));
    assert_eq!(
        today(),
        was,
        "drawing a revision moves nothing of today's design"
    );

    // Saved through the edit route, a change that would apply beside one
    // that is withheld: a release of it is refused whole, so nothing of it
    // is drawn, not even what would apply alone. A risk moved that nobody
    // registered withholds the method; the cases' new label would apply.
    let f = file();
    let cases = {
        let k = "\"case_csv\":\"";
        let at = f.find(k).unwrap() + k.len();
        let end = f[at..].find("\",\"declared\"").unwrap();
        f[at..at + end].replace("\\n", "\n").replace("\\\"", "\"")
    };
    assert!(cases.contains("the record's quietest day"), "{cases}");
    let relabelled = cases.replacen("the record's quietest day", "the quietest day on record", 1);
    let twice = format!("# Nia: and read again.\n{mine}");
    let mut fields: Vec<(&str, &str)> = vec![
        ("opened", field(&f, "opened")),
        ("method_text", &twice),
        ("case_csv", &relabelled),
        ("declared.author", "Test Nia"),
        ("declared.ai", "none"),
        ("why.risks", "R-99 L5->L4"),
    ];
    fields.extend(why);
    let r = post(port, EDIT, &fields);
    assert!(
        r.contains("\"written\":true") && r.contains("\"revision\":4"),
        "{r}"
    );
    let r = live();
    assert!(
        said(&r).contains("\"withheld\":[{\"what\":\"method_text\""),
        "{}",
        said(&r)
    );
    assert!(
        said(&r).contains("\"drawn\":false")
            && said(&r).contains("today's design would refuse a release of it whole"),
        "{}",
        said(&r)
    );
    assert_eq!(html(&r), fragment, "nothing of it is drawn");

    // A method that no longer reproduces its cases, saved: its checks
    // refuse it, as today's design would refuse a release holding it.
    let f = file();
    let late = method.replace("if f107 >= 130 then", "if f107 >= 135 then");
    let mut fields: Vec<(&str, &str)> = vec![
        ("opened", field(&f, "opened")),
        ("method_text", &late),
        ("case_csv", &cases),
        ("declared.author", "Test Nia"),
        ("declared.ai", "none"),
        ("why.risks", ""),
    ];
    fields.extend(why);
    let r = post(port, EDIT, &fields);
    assert!(
        r.contains("\"written\":true") && r.contains("\"revision\":5"),
        "{r}"
    );
    let r = live();
    assert!(
        said(&r).contains("\"checks\":[{\"place\":\"sw_activity_band\"")
            && said(&r).contains("130 sfu exactly"),
        "{}",
        said(&r)
    );
    assert!(
        said(&r).contains("\"drawn\":false")
            && said(&r).contains("its checks find what today's design would refuse"),
        "{}",
        said(&r)
    );
    assert_eq!(html(&r), fragment);

    // Changed in the copy's file itself: the method put back, and a
    // transcription nobody checked. Its checks refuse it, by its declaration.
    let at = copy.join(NODE);
    let mut n = sqlite::read(&at).unwrap();
    for t in n.texts.iter_mut().filter(|t| t.kind == "pseudocode") {
        t.body = format!("{mine}\n");
    }
    for t in n.tables.iter_mut().filter(|t| t.path == "declaration.csv") {
        t.csv = "author,ai,date,source,checked_by\nTest Nia,transcribed,2026-10-17,NOAA's table,\n"
            .into();
    }
    sqlite::write(&n, &at).unwrap();
    let r = live();
    assert!(
        said(&r).contains("\"checks\":[{\"place\":\"sw_activity_band\"")
            && said(&r).contains("\"drawn\":false"),
        "{}",
        said(&r)
    );
    assert!(
        said(&r).contains("checked"),
        "names who did not check it: {}",
        said(&r)
    );
    assert_eq!(html(&r), fragment);

    // A file that says it is another group's than the folder it is in.
    let mut moved = n.clone();
    moved.meta.insert(s("group_id"), s("l3_power"));
    sqlite::write(&moved, &at).unwrap();
    let r = live();
    assert!(
        r.contains("\"ok\":false") && r.contains("is not in that group's folder"),
        "{r}"
    );
    sqlite::write(&n, &at).unwrap();

    // A node of today's design with no file in the copy: refused, by name.
    let r = ask(port, "GET", "/v1/node/sw_ap_design/live", &[], false);
    assert!(
        r.contains("\"ok\":false") && r.contains("no node file sw_ap_design.vnode"),
        "{r}"
    );

    // A node file of a node today's design does not hold: refused, by name.
    let mut stray = File::new(Kind::Node, APP);
    stray.meta = n.meta.clone();
    stray.meta.insert(s("block_uid"), s("sw_not_in_today"));
    stray.blocks = vec![Block {
        uid: s("sw_not_in_today"),
        id: s("sw_not_in_today"),
        behaviour: s("method"),
        revision: 1,
        contract_version: 1,
        ..Block::default()
    }];
    sqlite::write(
        &stray,
        &copy.join("groups/l3_solar/nodes/sw_not_in_today.vnode"),
    )
    .unwrap();
    let r = ask(port, "GET", "/v1/node/sw_not_in_today/live", &[], false);
    assert!(
        r.contains("\"ok\":false") && r.contains("today's design holds no node sw_not_in_today"),
        "{r}"
    );
    let r = ask(port, "GET", "/v1/node/sw%20x/live", &[], false);
    assert!(r.contains("is not a node's id"), "{r}");

    assert_eq!(today(), was, "today's design is as it was");
    assert_eq!(run(), ran, "and it runs as it did");
    let _ = std::fs::remove_dir_all(&scratch);
}
