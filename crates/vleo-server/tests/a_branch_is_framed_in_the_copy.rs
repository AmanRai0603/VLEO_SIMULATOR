//! A branch is framed in the working copy through the program, by its owner
//! alone (docs/briefs/F4.md, F4-b; `vleo_files::breakdown`).
//!
//! Solar's branch holds solar 1.1's `sw_activity_band`, issued to Test Nia
//! and signed by Nia, and two children of it placed as F5 will issue them:
//! one carrying the band's own method, and one whose elevated edge is at 135
//! sfu, invented here to be wrong, and saying so. Test Ola owns solar; Test
//! Dee is Ola's deputy; Test Sam owns the systems' branch. Through
//! `GET /v1/branch` and `POST /v1/branch/<id>`:
//!
//! - with no copy, the branch is refused, saying to copy the drive; an edit
//!   with no key open, or without the launch's word, is refused;
//! - to Nia it is not Nia's to frame, said whose it is, and Nia's edit is
//!   refused by name, with nothing written;
//! - to Ola it is Ola's: the band and its children, each in its node file,
//!   open parts counted; broken down into the child whose edge is wrong, it
//!   is refused naming the case at 130 sfu and nothing is written; into the
//!   child that reproduces it, accepted, read as its children with all ten
//!   cases agreeing, a change row saying who and why, and no node file
//!   touched;
//! - an edit from what was opened before is a second writer: refused, said,
//!   and nothing written;
//! - the band changed since it was signed: its breakdown is not judged, and
//!   said so, never as children that disagree; a node file that cannot be
//!   read refuses the branch by its name;
//! - add, fill (a question, then an output and an input), rename, move (an
//!   order that is not a whole number refused) and split, each through its
//!   fields; an answer that is not `<output>=<child>.<port>` refused;
//! - the systems' branch is Sam's, and not Ola's; solar's is not Sam's;
//! - Dee frames solar as Ola's deputy, and folds it back;
//! - an edit that is not one, a group that is not an id, a group no file
//!   mounts and the programme's branch are each refused by name;
//! - the drive holds what it held when it was copied.

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
const GOOD: &str = "sw_activity_count";
const LATE: &str = "sw_activity_count_late";
const GROUP: &str = "groups/l3_solar/l3_solar.vgroup";
const WHY: &str = "The band is counted from its edges; the count is a node of its own.";

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

/// A child of the band, as F5 will issue it: the band's own file, renamed,
/// under it, reading the band's inputs, its method edited by `edit`.
fn child(id: &str, edit: &dyn Fn(&str) -> String) -> File {
    let mut c = node();
    c.meta.insert(s("block_uid"), s(id));
    c.meta.insert(s("writer"), s(""));
    c.blocks[0].uid = s(id);
    c.blocks[0].id = s(id);
    c.blocks[0].parent_uid = s(UID);
    for p in c.ports.iter_mut() {
        p.block_uid = s(id);
    }
    c.wires = c
        .ports
        .iter()
        .filter(|p| p.direction == "in")
        .map(|p| Wire {
            to_block: s(id),
            to_port: p.name.clone(),
            from_ref: format!("{UID}.{}", p.name),
        })
        .collect();
    for t in c.texts.iter_mut() {
        t.scope = s(id);
        if t.kind == "pseudocode" {
            t.body = edit(&t.body);
        }
    }
    for t in c.tables.iter_mut() {
        t.scope = s(id);
    }
    c
}

/// The child at 135 sfu says what it is: invented for this check by the
/// test's own invented person, `who`, never transcribed by anyone from
/// anything (F4's question 5).
fn invented(mut c: File, who: &str) -> File {
    for t in c.texts.iter_mut().filter(|t| t.kind == "pseudocode") {
        let rest: Vec<&str> = t.body.lines().skip(1).collect();
        t.body = format!(
            "# INVENTED for the breakdown's check, never part of the design: sw_activity_band's method, its elevated edge moved from 130 to 135 sfu to be wrong.\n{}\n",
            rest.join("\n")
        );
    }
    for t in c.tables.iter_mut().filter(|t| t.path == "declaration.csv") {
        t.csv = format!(
            "author,ai,date,source,checked_by\n{who},transcribed,2026-10-17,\"sw_activity_band's method, its elevated edge moved to 135 sfu: invented to be wrong, for the breakdown's check\",{who}\n"
        );
    }
    c
}

// Test Pat (81), programme manager; Test Sam (82), system engineer; Test Ola
// (83), who owns solar; Test Dee (86), Ola's deputy; Test Nia (84), a node
// engineer of solar, assigned the band.
fn drive(at: &Path, keys: &Path) {
    let mut programme = group("programme", "Test Pat");
    register(
        &mut programme,
        "Test Pat",
        "programme manager",
        &key(81),
        "Test Pat",
    );
    register(
        &mut programme,
        "Test Sam",
        "system engineer",
        &key(82),
        "Test Pat",
    );
    register(
        &mut programme,
        "Test Ola",
        "subsystem engineer",
        &key(83),
        "Test Pat",
    );
    register(
        &mut programme,
        "Test Dee",
        "subsystem engineer",
        &key(86),
        "Test Pat",
    );
    programme.people.last_mut().unwrap().deputy_for = s("Test Ola");
    mount(&mut programme, "root", "systems", "Test Sam");
    std::fs::create_dir_all(at.join("groups/programme")).unwrap();
    sqlite::write(
        &signed(programme, "Test Pat", &key(81)),
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
        &signed(systems, "Test Sam", &key(82)),
        &at.join("groups/systems/systems.vgroup"),
    )
    .unwrap();
    let mut solar = group("l3_solar", "Test Ola");
    register(
        &mut solar,
        "Test Nia",
        "node engineer",
        &key(84),
        "Test Ola",
    );
    solar.assignments.push(Assignment {
        block_uid: s(UID),
        person: s("Test Nia"),
        contract_version: 1,
    });
    std::fs::create_dir_all(at.join("groups/l3_solar/nodes")).unwrap();
    sqlite::write(&signed(solar, "Test Ola", &key(83)), &at.join(GROUP)).unwrap();
    // Signed by Nia as it is: its cases are evidence for its breakdown.
    let mut band = node();
    let row = chain::sign(&band, UID, "Test Nia", &key(84), "2026-10-01", "ok", "").unwrap();
    band.signatures.push(row);
    sqlite::write(&band, &at.join(nodes(UID))).unwrap();
    sqlite::write(&child(GOOD, &|m| m.to_string()), &at.join(nodes(GOOD))).unwrap();
    sqlite::write(
        &invented(
            child(LATE, &|m| {
                m.replace("if f107 >= 130 then", "if f107 >= 135 then")
            }),
            "Test Nia",
        ),
        &at.join(nodes(LATE)),
    )
    .unwrap();
    std::fs::write(
        at.join("START HERE.md"),
        format!(
            "# START HERE: an invented drive\n\nThe programme manager's key:\n\n    {}\n",
            key(81).public().fingerprint()
        ),
    )
    .unwrap();
    std::fs::create_dir_all(keys).unwrap();
    write_key(keys, "test-ola.vkey", "Test Ola", &key(83));
    write_key(keys, "test-dee.vkey", "Test Dee", &key(86));
    write_key(keys, "test-nia.vkey", "Test Nia", &key(84));
    write_key(keys, "test-sam.vkey", "Test Sam", &key(82));
}

fn nodes(uid: &str) -> String {
    format!("groups/l3_solar/nodes/{uid}.vnode")
}

fn open_key(port: u16, file: &str) -> String {
    post(
        port,
        "/v1/key/open",
        &[
            ("file", file),
            ("passphrase", PASSPHRASE),
            ("accept_anchor", &key(81).public().fingerprint()),
        ],
    )
}

/// Every node file of the branch, as bytes.
fn node_bytes(copy: &Path) -> Vec<Vec<u8>> {
    [UID, GOOD, LATE]
        .iter()
        .map(|u| std::fs::read(copy.join(nodes(u))).unwrap())
        .collect()
}

#[test]
fn a_branch_is_framed_in_the_copy() {
    let scratch = std::env::temp_dir().join(format!("vleo-branch-{}", std::process::id()));
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
    let port = vleo_server::serve(Some(root()), 18963, false, true).expect("no server");
    let tree = || ask(port, "GET", "/v1/branch?group=l3_solar", &[], false);
    let edit = |name: &str, pairs: &[(&str, &str)]| {
        let mut all = vec![("group", "l3_solar"), ("why", WHY)];
        all.extend_from_slice(pairs);
        post(port, &format!("/v1/branch/{name}"), &all)
    };
    let group_bytes = || std::fs::read(copy.join(GROUP)).unwrap();

    // No copy yet: refused, saying to copy the drive.
    let r = tree();
    assert!(r.contains("\"ok\":false"), "{r}");
    assert!(
        field(&r, "message").contains("copy it from the drive first"),
        "{r}"
    );
    let copied = post(port, "/v1/copy", &[("drive", &drv.display().to_string())]);
    assert!(copied.contains("\"ok\":true"), "{copied}");
    // What the drive holds, before anything is framed: never written here.
    let on_drive = |d: &Path| -> Vec<Vec<u8>> {
        [GROUP.to_string(), nodes(UID), nodes(GOOD), nodes(LATE)]
            .iter()
            .map(|p| std::fs::read(d.join(p)).unwrap())
            .collect()
    };
    let drive_before = on_drive(&drv);

    // An edit with no key open is refused.
    let r = edit("add", &[("parent", UID), ("id", "sw_activity_note")]);
    assert!(field(&r, "message").starts_with("no key is open"), "{r}");

    // Nia: read, not Nia's to frame, said whose; Nia's edit refused by name.
    assert!(open_key(port, "test-nia.vkey").contains("\"open\":true"));
    let r = tree();
    assert!(
        r.contains("\"ok\":true") && r.contains("\"yours\":false"),
        "{r}"
    );
    assert_eq!(field(&r, "owner"), "Test Ola");
    assert!(
        field(&r, "why_not").starts_with(
            "Not yours to frame: l3_solar is framed by its owner Test Ola, or their deputy."
        ),
        "{r}"
    );
    assert!(
        field(&r, "next").starts_with("Nothing: l3_solar is framed by its owner Test Ola"),
        "{r}"
    );
    let before = group_bytes();
    let r = edit(
        "add",
        &[
            ("opened", field(&tree(), "opened")),
            ("parent", UID),
            ("id", "sw_activity_note"),
        ],
    );
    assert!(
        r.contains("\"ok\":false") && field(&r, "message").contains("Test Nia"),
        "{r}"
    );
    assert_eq!(group_bytes(), before);
    post(port, "/v1/key/close", &[]);

    // Ola: Ola's to frame. The band and its children, each in its node file.
    assert!(open_key(port, "test-ola.vkey").contains("\"open\":true"));
    let r = tree();
    assert!(
        r.contains("\"yours\":true") && r.contains("\"why_not\":\"\""),
        "{r}"
    );
    assert!(r.contains("\"signed\":{\"ok\":true"), "{r}");
    assert!(field(&r, "shows").starts_with("your copy"), "{r}");
    for uid in [UID, GOOD, LATE] {
        assert!(
            r.contains(&format!(
                "{{\"uid\":\"{uid}\",\"id\":\"{uid}\",\"parent\":\"{}\"",
                if uid == UID { "" } else { UID }
            )),
            "{uid} in {r}"
        );
    }
    assert!(
        r.contains("\"read_as\":\"method\",\"in\":\"its node file\""),
        "{r}"
    );
    assert!(!r.contains("\"reproduces\""), "{r}");
    let nodes_before = node_bytes(&copy);

    // Without the launch's word, Ola's edit is refused, and nothing written.
    let before = group_bytes();
    let opened = field(&r, "opened").to_string();
    let r = ask(
        port,
        "POST",
        "/v1/branch/add",
        &[
            ("group", "l3_solar"),
            ("why", WHY),
            ("opened", &opened),
            ("parent", UID),
            ("id", "sw_activity_note"),
        ],
        false,
    );
    assert!(
        r.contains("does not carry the word this launch gave its page"),
        "{r}"
    );
    assert_eq!(group_bytes(), before);

    // Into the child whose edge is wrong: refused, naming the case.
    let before = group_bytes();
    let bd = |opened: &str, child: &str| {
        edit(
            "break-down",
            &[
                ("opened", opened),
                ("node", UID),
                ("answers", &format!("band={child}.band")),
            ],
        )
    };
    let r = bd(field(&tree(), "opened"), LATE);
    assert!(r.contains("\"ok\":false"), "{r}");
    assert!(r.contains("130 sfu exactly") && r.contains(LATE), "{r}");
    assert_eq!(group_bytes(), before);

    // Into the child that reproduces it: accepted, and read as its children.
    let stale = tree();
    let r = bd(field(&stale, "opened"), GOOD);
    assert!(
        r.contains("\"ok\":true") && r.contains("\"edit\":\"break down\""),
        "{r}"
    );
    assert!(r.contains("reproduce all 10 of its cases"), "{r}");
    assert_eq!(field(&r, "opened"), field(&tree(), "opened"));
    let r = tree();
    assert!(
        r.contains(&format!(
            "\"answers\":[{{\"output\":\"band\",\"child\":\"{GOOD}\",\"port\":\"band\"}}]"
        )),
        "{r}"
    );
    assert!(r.contains("\"read_as\":\"children\""), "{r}");
    assert!(r.contains("\"reproduces\":{\"state\":\"agrees\""), "{r}");
    assert_eq!(r.matches("\"agrees\":true").count(), 10, "{r}");
    assert!(
        r.contains("\"label\":\"130 sfu exactly — elevated begins.\""),
        "{r}"
    );
    assert!(r.contains("\"who\":\"Test Ola\"") && r.contains(WHY), "{r}");
    assert_eq!(node_bytes(&copy), nodes_before);

    // An edit from what was opened before: a second writer, nothing written.
    let before = group_bytes();
    let r = edit(
        "add",
        &[
            ("opened", field(&stale, "opened")),
            ("parent", UID),
            ("id", "sw_activity_note"),
        ],
    );
    assert!(r.contains("\"ok\":false,\"conflict\":true"), "{r}");
    assert!(
        field(&r, "message").contains("changed in your copy since you opened it"),
        "{r}"
    );
    assert_eq!(group_bytes(), before);

    // The band changed since Nia signed it: its breakdown cannot be judged,
    // said as that, never as children that disagree.
    let band_bytes = std::fs::read(copy.join(nodes(UID))).unwrap();
    let mut changed = sqlite::read(&copy.join(nodes(UID))).unwrap();
    for t in changed
        .tables
        .iter_mut()
        .filter(|t| t.path == "results/isolation.csv")
    {
        t.csv = t.csv.replacen(",paper,", ",hand,", 1);
    }
    sqlite::write(&changed, &copy.join(nodes(UID))).unwrap();
    let r = tree();
    assert!(r.contains("\"reproduces\":{\"state\":\"not run\""), "{r}");
    assert!(
        field(&r, "next").starts_with("A breakdown cannot be judged now"),
        "{r}"
    );
    std::fs::write(copy.join(nodes(UID)), &band_bytes).unwrap();
    // A node file that cannot be read refuses the branch by its name.
    std::fs::write(copy.join(nodes("zz_unread")), b"not a design file").unwrap();
    let r = tree();
    assert!(
        r.contains("\"ok\":false") && r.contains("zz_unread.vnode cannot be read"),
        "{r}"
    );
    std::fs::remove_file(copy.join(nodes("zz_unread"))).unwrap();

    // Each edit's own fields, through the route.
    let opened = || field(&tree(), "opened").to_string();
    let ok = |r: String| {
        assert!(r.contains("\"ok\":true"), "{r}");
        r
    };
    ok(edit(
        "add",
        &[
            ("opened", &opened()),
            ("parent", UID),
            ("id", "sw_activity_note"),
        ],
    ));
    ok(edit(
        "fill",
        &[
            ("opened", &opened()),
            ("block", "sw_activity_note"),
            ("question", "What does the band leave out?"),
        ],
    ));
    for (dir, name) in [("out", "note"), ("in", "seen")] {
        ok(edit(
            "fill",
            &[
                ("opened", &opened()),
                ("block", "sw_activity_note"),
                ("direction", dir),
                ("name", name),
                ("type", "number"),
                ("unit", "1"),
                ("state", "open"),
            ],
        ));
    }
    ok(edit(
        "rename",
        &[
            ("opened", &opened()),
            ("block", "sw_activity_note"),
            ("id", "sw_activity_remark"),
        ],
    ));
    let r = edit(
        "move",
        &[
            ("opened", &opened()),
            ("block", "sw_activity_note"),
            ("parent", UID),
            ("ord", "first"),
        ],
    );
    assert!(r.contains("ord is a whole number"), "{r}");
    ok(edit(
        "move",
        &[
            ("opened", &opened()),
            ("block", "sw_activity_note"),
            ("parent", UID),
            ("ord", "4"),
        ],
    ));
    ok(edit(
        "split",
        &[
            ("opened", &opened()),
            ("block", "sw_activity_note"),
            ("id", "sw_activity_aside"),
            ("ports", " note , "),
        ],
    ));
    let r = tree();
    assert!(
        r.contains("\"uid\":\"sw_activity_note\",\"id\":\"sw_activity_remark\""),
        "{r}"
    );
    assert!(
        r.contains("\"uid\":\"sw_activity_aside\",\"id\":\"sw_activity_aside\""),
        "{r}"
    );
    let aside = &r[r.find("\"uid\":\"sw_activity_aside\"").unwrap()..];
    assert!(
        aside[..aside.find("\"answers\"").unwrap()].contains("\"name\":\"note\""),
        "the split moved its output: {aside}"
    );
    assert!(r.contains("\"ord\":4"), "{r}");
    let r = edit(
        "break-down",
        &[("opened", &opened()), ("node", UID), ("answers", "band")],
    );
    assert!(r.contains("is not <output>=<child>.<port>"), "{r}");
    post(port, "/v1/key/close", &[]);

    // The systems' branch is the system engineer's: framed by Sam, read by Ola.
    assert!(open_key(port, "test-sam.vkey").contains("\"open\":true"));
    let sys = || ask(port, "GET", "/v1/branch?group=systems", &[], false);
    let r = sys();
    assert!(
        r.contains("\"owner\":\"Test Sam\"") && r.contains("\"yours\":true"),
        "{r}"
    );
    let r = tree();
    assert!(r.contains("\"yours\":false"), "{r}");
    post(port, "/v1/key/close", &[]);
    assert!(open_key(port, "test-ola.vkey").contains("\"open\":true"));
    let r = sys();
    assert!(r.contains("\"yours\":false"), "{r}");
    post(port, "/v1/key/close", &[]);

    // Dee, Ola's deputy, frames it, and folds it back.
    assert!(open_key(port, "test-dee.vkey").contains("\"open\":true"));
    let r = tree();
    assert!(r.contains("\"yours\":true"), "{r}");
    let r = edit(
        "fold-back",
        &[("opened", field(&tree(), "opened")), ("node", UID)],
    );
    assert!(r.contains("\"ok\":true"), "{r}");
    let r = tree();
    assert!(
        !r.contains("\"reproduces\"") && r.contains("\"who\":\"Test Dee\""),
        "{r}"
    );
    assert_eq!(node_bytes(&copy), nodes_before);

    // Dee assigns a child to Nia (F5): the assignment is in the group file.
    let r = edit(
        "assign",
        &[
            ("opened", field(&tree(), "opened")),
            ("block", GOOD),
            ("person", "Test Nia"),
        ],
    );
    assert!(r.contains("\"ok\":true"), "{r}");
    let g = vleo_files::sqlite::read(&copy.join(GROUP)).unwrap();
    assert!(
        g.assignments
            .iter()
            .any(|a| a.block_uid == GOOD && a.person == "Test Nia"),
        "{:?}",
        g.assignments
    );
    let r = edit(
        "wire",
        &[
            ("opened", field(&tree(), "opened")),
            ("block", GOOD),
            ("port", "f107"),
        ],
    );
    assert!(r.contains("wire needs from"), "{r}");
    // The band's contract changes (W4): it rises in the group file, and the
    // band's file is issued again at it.
    let r = edit(
        "contract",
        &[("opened", field(&tree(), "opened")), ("node", UID)],
    );
    assert!(r.contains("\"ok\":true"), "{r}");
    let g = vleo_files::sqlite::read(&copy.join(GROUP)).unwrap();
    assert_eq!(
        g.assignments
            .iter()
            .find(|a| a.block_uid == UID)
            .map(|a| a.contract_version),
        Some(2)
    );
    let band = vleo_files::sqlite::read(&copy.join(nodes(UID))).unwrap();
    assert_eq!(band.blocks[0].contract_version, 2);
    // Issued: a block framed in the group file, refused by name while it is
    // assigned to nobody, and nothing written.
    let before_issue = group_bytes();
    let r = edit(
        "issue",
        &[
            ("opened", field(&tree(), "opened")),
            ("block", "sw_activity_aside"),
            ("behaviour", "method"),
        ],
    );
    assert!(r.contains("is assigned to nobody"), "{r}");
    assert_eq!(group_bytes(), before_issue);
    assert!(!copy
        .join("groups/l3_solar/nodes/sw_activity_aside.vnode")
        .exists());
    // A sheet: two blocks set up at once, through the route.
    let r = edit(
        "sheet",
        &[
            ("opened", field(&tree(), "opened")),
            (
                "sheet",
                "id,parent,question,output,type,unit\n\
                 sw_storm_onset,sw_activity_band,When does a storm begin?,sw_storm_onset,number,1\n\
                 sw_storm_peak,sw_activity_band,How high does it go?,sw_storm_peak,number,1\n",
            ),
        ],
    );
    assert!(r.contains("\"ok\":true"), "{r}");
    let r = tree();
    assert!(
        r.contains("\"uid\":\"sw_storm_onset\"") && r.contains("\"uid\":\"sw_storm_peak\""),
        "{r}"
    );

    // Refused by name: an edit that is not one, a group that is not an id, a
    // group no file mounts.
    let r = edit("graft", &[("opened", field(&tree(), "opened"))]);
    assert!(
        r.contains("\\\"graft\\\" is not an edit of a branch"),
        "{r}"
    );
    let r = ask(port, "GET", "/v1/branch?group=..%2Fsystems", &[], false);
    assert!(r.contains("is not a group's id"), "{r}");
    let r = ask(port, "GET", "/v1/branch?group=l3_power", &[], false);
    assert!(r.contains("no file mounts the group"), "{r}");
    let r = ask(port, "GET", "/v1/branch?group=programme", &[], false);
    assert!(r.contains("Programme workspace (F7)"), "{r}");
    // The drive is as it was copied: everything above is in the copy alone.
    assert_eq!(on_drive(&drv), drive_before);
    let _ = std::fs::remove_dir_all(&scratch);
}
