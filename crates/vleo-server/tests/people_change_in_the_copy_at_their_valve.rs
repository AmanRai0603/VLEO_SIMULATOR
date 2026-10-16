//! The database copied from the drive, and whom its files register changed in
//! that copy at the valve that owns each (docs/briefs/F1.md, PR 2; decision 3
//! of docs/PLAN_F.md).
//!
//! A small drive of invented people is written here. Then, through the
//! program's routes: the copy taken and its record read back; the programme
//! manager registering a subsystem engineer in the programme's file; a
//! subsystem engineer registering a node engineer in their group, and
//! refused without the key's proof; the programme manager refused in a group
//! they do not own; a second copy refused over work not yet put back; a change
//! on the drive seen; and the pause after wrong passphrases. One test, because
//! the program's environment and its open key are the process's own.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use vleo_files::chain::{self, FILE};
use vleo_files::keys::{SigningKey, MIN_ITERATIONS};
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::{people, sqlite};

const APP: &str = "vleo 1.0.0 (test)";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn enc(v: &str) -> String {
    v.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// A request as the page makes it: a POST carries this launch's word.
fn ask(port: u16, method: &str, path: &str, pairs: &[(&str, &str)]) -> String {
    ask_as(port, method, path, pairs, true)
}

/// A request as another program on the same computer makes it: no word.
fn ask_bare(port: u16, method: &str, path: &str, pairs: &[(&str, &str)]) -> String {
    ask_as(port, method, path, pairs, false)
}

fn ask_as(port: u16, method: &str, path: &str, pairs: &[(&str, &str)], with_word: bool) -> String {
    let mut body: Vec<String> = pairs
        .iter()
        .map(|(k, v)| format!("{k}={}", enc(v)))
        .collect();
    if method == "POST" && with_word {
        body.insert(0, format!("session={}", vleo_server::session()));
    }
    let body = body.join("&");
    let path = if method == "GET" && !body.is_empty() {
        format!("{path}?{body}")
    } else {
        path.to_string()
    };
    let body = if method == "GET" { String::new() } else { body };
    let mut s = TcpStream::connect(("127.0.0.1", port)).expect("the server did not accept");
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

fn s(v: &str) -> String {
    v.to_string()
}

fn key(seed: u8) -> SigningKey {
    SigningKey::from_seed([seed; 32])
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

fn group(id: &str) -> File {
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

fn write_key(dir: &Path, file: &str, person: &str, k: &SigningKey, passphrase: &str) {
    let locked = k
        .lock(person, passphrase, [7; 16], [7; 12], MIN_ITERATIONS)
        .unwrap();
    let mut f = File::new(Kind::Key, APP);
    f.meta.insert(s("person"), s(person));
    f.locked_keys.push(locked.to_row());
    sqlite::write(&f, &dir.join(file)).unwrap();
}

// Test Pat, the programme manager (41); Test Sam, the system engineer (42);
// Test Ola, who owns solar (43); Test Nia, solar's node engineer (44).
fn drive(at: &Path, keys: &Path) {
    for g in ["programme", "systems", "l3_solar"] {
        std::fs::create_dir_all(at.join("groups").join(g)).unwrap();
    }
    let mut programme = group("programme");
    register(
        &mut programme,
        "Test Pat",
        "programme manager",
        &key(41),
        "Test Pat",
    );
    register(
        &mut programme,
        "Test Sam",
        "system engineer",
        &key(42),
        "Test Pat",
    );
    register(
        &mut programme,
        "Test Ola",
        "subsystem engineer",
        &key(43),
        "Test Pat",
    );
    mount(&mut programme, "root", "systems", "Test Sam");
    let programme = signed(programme, "Test Pat", &key(41));
    sqlite::write(&programme, &at.join("groups/programme/programme.vgroup")).unwrap();

    let mut systems = group("systems");
    mount(
        &mut systems,
        "sys_space_environment",
        "l3_solar",
        "Test Ola",
    );
    let systems = signed(systems, "Test Sam", &key(42));
    sqlite::write(&systems, &at.join("groups/systems/systems.vgroup")).unwrap();

    let mut solar = group("l3_solar");
    register(
        &mut solar,
        "Test Nia",
        "node engineer",
        &key(44),
        "Test Ola",
    );
    let solar = signed(solar, "Test Ola", &key(43));
    sqlite::write(&solar, &at.join("groups/l3_solar/l3_solar.vgroup")).unwrap();

    std::fs::write(
        at.join("START HERE.md"),
        format!(
            "# START HERE: an invented drive\n\nThe programme manager's key:\n\n    {}\n",
            key(41).public().fingerprint()
        ),
    )
    .unwrap();
    std::fs::create_dir_all(keys).unwrap();
    for (file, person, seed) in [
        ("test-pat.vkey", "Test Pat", 41),
        ("test-ola.vkey", "Test Ola", 43),
        ("test-nia.vkey", "Test Nia", 44),
    ] {
        write_key(keys, file, person, &key(seed), "a long enough passphrase");
    }
}

/// Owned pairs as the borrowed pairs `ask` takes.
fn as_pairs(v: &[(String, String)]) -> Vec<(&str, &str)> {
    v.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect()
}

/// A database with one thing in it that is not an ordinary file or folder.
fn bad_drive(at: &Path, what: &str) {
    std::fs::create_dir_all(at.join("groups/l3_solar")).unwrap();
    std::fs::write(at.join("START HERE.md"), "# START HERE\n").unwrap();
    let thing = at.join("groups/l3_solar");
    match what {
        "link" => {
            #[cfg(unix)]
            std::os::unix::fs::symlink(std::env::temp_dir(), thing.join("elsewhere")).unwrap();
        }
        "key" => std::fs::write(thing.join("someone.vkey"), b"a key").unwrap(),
        _ => {
            let made = std::process::Command::new("mkfifo")
                .arg(thing.join("a-pipe"))
                .status();
            assert!(
                made.is_ok_and(|s| s.success()),
                "mkfifo is there to make a pipe"
            );
        }
    }
}

fn field<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let at = json.find(&format!("\"{key}\":"))? + key.len() + 3;
    let rest = &json[at..];
    if let Some(r) = rest.strip_prefix('"') {
        r.find('"').map(|e| &r[..e])
    } else {
        rest.split([',', '}']).next()
    }
}

fn open(port: u16, file: &str) -> String {
    ask(
        port,
        "POST",
        "/v1/key/open",
        &[
            ("file", file),
            ("passphrase", "a long enough passphrase"),
            ("accept_anchor", &key(41).public().fingerprint()),
        ],
    )
}

#[test]
fn people_change_in_the_copy_at_their_valve() {
    let scratch = std::env::temp_dir().join(format!("vleo-people-{}", std::process::id()));
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
    let port = vleo_server::serve(Some(root()), 18971, false, true).expect("no server");

    // No copy yet; a copy is taken only from a database, and only from the page.
    let state = ask(port, "GET", "/v1/copy", &[]);
    assert_eq!(field(&state, "copied"), Some("false"), "{state}");
    let early = ask(
        port,
        "POST",
        "/v1/people/register",
        &[("file", "programme"), ("name", "Test Tom")],
    );
    assert!(early.contains("copy it from the drive first"), "{early}");
    let drive_text = drv.display().to_string();
    for (path, pairs) in [
        ("/v1/copy", vec![("drive", drive_text.as_str())]),
        (
            "/v1/people/register",
            vec![("file", "programme"), ("name", "Test Tom")],
        ),
    ] {
        let bare = ask_bare(port, "POST", path, &pairs);
        assert!(
            bare.contains("does not carry the word this launch gave its page"),
            "{path}: {bare}"
        );
    }
    // Only folders and ordinary files are copied, and never a key.
    for (what, says) in [
        ("link", "is a link"),
        ("key", "is a key"),
        ("pipe", "neither a folder nor a file"),
    ] {
        let bad = scratch.join(format!("bad-{what}"));
        bad_drive(&bad, what);
        let e = ask(
            port,
            "POST",
            "/v1/copy",
            &[("drive", &bad.display().to_string())],
        );
        assert!(e.contains(says), "{what}: {e}");
        assert!(!copy.exists(), "{what}: nothing is copied");
    }
    // A folder the program did not fill is never copied over.
    let foreign = scratch.join("foreign");
    std::fs::create_dir_all(&foreign).unwrap();
    std::fs::write(foreign.join("mine.txt"), "someone's own").unwrap();
    std::env::set_var("VLEO_COPY", &foreign);
    let e = ask(port, "POST", "/v1/copy", &[("drive", &drive_text)]);
    assert!(e.contains("holds files the program did not copy"), "{e}");
    std::env::set_var("VLEO_COPY", &copy);
    // Nor while another program holds the copy's lock.
    let lock = scratch.join(".copy.lock");
    std::fs::write(&lock, "1").unwrap();
    let e = ask(port, "POST", "/v1/copy", &[("drive", &drive_text)]);
    assert!(e.contains("another program is copying"), "{e}");
    std::fs::remove_file(&lock).unwrap();
    let nowhere = ask(port, "POST", "/v1/copy", &[("drive", "/nowhere/at/all")]);
    assert!(nowhere.contains("is not a database"), "{nowhere}");

    // The copy, and its record.
    let took = ask(port, "POST", "/v1/copy", &[("drive", &drive_text)]);
    assert_eq!(
        (field(&took, "ok"), field(&took, "files")),
        (Some("true"), Some("4")),
        "{took}"
    );
    let state = ask(port, "GET", "/v1/copy", &[]);
    assert!(
        state.contains("\"copied\":true")
            && state.contains("\"changed_here\":[]")
            && state.contains("\"changed_on_drive\":[]"),
        "{state}"
    );

    // The page the program opens carries the word, and only this person reads it.
    let page = std::fs::read_to_string(keys.join("open.html")).unwrap();
    assert!(page.contains(&vleo_server::session()), "{page}");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(keys.join("open.html"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    // The first open names the drive the anchor will be kept for.
    let first = ask(
        port,
        "POST",
        "/v1/key/open",
        &[
            ("file", "test-pat.vkey"),
            ("passphrase", "a long enough passphrase"),
        ],
    );
    let real_drive = std::fs::canonicalize(&drv).unwrap().display().to_string();
    assert_eq!(field(&first, "drive"), Some(real_drive.as_str()), "{first}");

    // The programme manager registers a subsystem engineer, with the key's proof.
    let pat = open(port, "test-pat.vkey");
    assert_eq!(field(&pat, "name"), Some("Test Pat"), "{pat}");
    let tom = key(45);
    let registered = ask(
        port,
        "POST",
        "/v1/people/register",
        &[
            ("file", "programme"),
            ("name", "Test Tom"),
            ("role", "subsystem engineer"),
            ("public_key", &tom.public().to_hex()),
            ("proof", &people::proof(&tom, "Test Tom")),
        ],
    );
    assert_eq!(field(&registered, "ok"), Some("true"), "{registered}");
    let listed = ask(port, "GET", "/v1/people", &[("file", "programme")]);
    assert!(
        listed.contains("\"name\":\"Test Tom\"") && listed.contains(&tom.public().fingerprint()),
        "{listed}"
    );
    // Written in the copy, signed by Pat; the drive is not touched.
    let in_copy = sqlite::read(&copy.join("groups/programme/programme.vgroup")).unwrap();
    let pr = chain::open_programme(in_copy, &key(41).public().fingerprint()).unwrap();
    assert_eq!(
        chain::whoami(&tom.public().to_hex(), "2027-01-01", &pr, None, &[])
            .unwrap()
            .name,
        "Test Tom"
    );
    let on_drive = sqlite::read(&drv.join("groups/programme/programme.vgroup")).unwrap();
    assert!(!on_drive.people.iter().any(|p| p.name == "Test Tom"));
    let state = ask(port, "GET", "/v1/copy", &[]);
    assert!(
        state.contains("\"changed_here\":[\"groups/programme/programme.vgroup\"]"),
        "{state}"
    );

    // Pat does not own solar.
    let nina = key(46);
    let nina_pairs = |proof: &str| -> Vec<(String, String)> {
        vec![
            (s("file"), s("l3_solar")),
            (s("name"), s("Test Nina")),
            (s("role"), s("node engineer")),
            (s("public_key"), nina.public().to_hex()),
            (s("proof"), s(proof)),
        ]
    };

    let good = nina_pairs(&people::proof(&nina, "Test Nina"));
    let e = ask(port, "POST", "/v1/people/register", &as_pairs(&good));
    assert!(
        e.contains("its owner Test Ola, or their deputy, does"),
        "{e}"
    );

    // Ola, who owns solar, registers its node engineer; without the key's own
    // proof, nobody is registered.
    let ola = open(port, "test-ola.vkey");
    assert_eq!(field(&ola, "name"), Some("Test Ola"), "{ola}");
    let forged = nina_pairs(&people::proof(&key(43), "Test Nina"));
    let e = ask(port, "POST", "/v1/people/register", &as_pairs(&forged));
    assert!(
        e.contains("nobody registers a key that is not theirs"),
        "{e}"
    );
    let ok = ask(port, "POST", "/v1/people/register", &as_pairs(&good));
    assert_eq!(field(&ok, "ok"), Some("true"), "{ok}");
    let revoked = ask(
        port,
        "POST",
        "/v1/people/revoke",
        &[
            ("file", "l3_solar"),
            ("name", "Test Nina"),
            ("public_key", &nina.public().to_hex()),
            ("from", "2026-10-01"),
        ],
    );
    assert_eq!(field(&revoked, "ok"), Some("true"), "{revoked}");

    // A second copy is never taken over work that has not gone back.
    let again = ask(port, "POST", "/v1/copy", &[("drive", &drive_text)]);
    assert!(
        again.contains("never copied over") && again.contains("groups/programme/programme.vgroup"),
        "{again}"
    );
    // Someone else's work on the drive is seen.
    std::fs::write(drv.join("groups/l3_solar/notes.txt"), "new on the drive").unwrap();
    let state = ask(port, "GET", "/v1/copy", &[]);
    assert!(
        state.contains("\"changed_on_drive\":[\"groups/l3_solar/notes.txt\"]"),
        "{state}"
    );

    // Closed, nothing is signed.
    ask(port, "POST", "/v1/key/close", &[]);
    let e = ask(
        port,
        "POST",
        "/v1/people/revoke",
        &[
            ("file", "l3_solar"),
            ("name", "Test Nia"),
            ("public_key", &key(44).public().to_hex()),
            ("from", "2026-10-01"),
        ],
    );
    assert!(e.contains("no key is open"), "{e}");

    // A second copy, somewhere else: it is named by the drive it came from, so
    // the anchor this computer accepted for that drive holds for it too.
    let copy2 = scratch.join("copy2");
    std::env::set_var("VLEO_COPY", &copy2);
    let took = ask(port, "POST", "/v1/copy", &[("drive", &drive_text)]);
    assert_eq!(field(&took, "ok"), Some("true"), "{took}");
    let ola = ask(
        port,
        "POST",
        "/v1/key/open",
        &[
            ("file", "test-ola.vkey"),
            ("passphrase", "a long enough passphrase"),
        ],
    );
    assert_eq!(field(&ola, "name"), Some("Test Ola"), "{ola}");
    assert!(!ola.contains("drive_newer"), "{ola}");
    ask(port, "POST", "/v1/key/close", &[]);

    // Someone signs a newer solar on the drive: opening on the older copy
    // says so, for the copy may not hold a revocation made since.
    let mut solar = group("l3_solar");
    register(
        &mut solar,
        "Test Nia",
        "node engineer",
        &key(44),
        "Test Ola",
    );
    register(
        &mut solar,
        "Test Nell",
        "node engineer",
        &key(47),
        "Test Ola",
    );
    let solar = signed(solar, "Test Ola", &key(43));
    sqlite::write(&solar, &drv.join("groups/l3_solar/l3_solar.vgroup")).unwrap();
    let ola = ask(
        port,
        "POST",
        "/v1/key/open",
        &[
            ("file", "test-ola.vkey"),
            ("passphrase", "a long enough passphrase"),
        ],
    );
    assert!(
        ola.contains("\"drive_newer\":[\"groups/l3_solar/l3_solar.vgroup\"]"),
        "{ola}"
    );
    ask(port, "POST", "/v1/key/close", &[]);

    // Copied again, cleanly: the copy before is kept beside, never deleted.
    for _ in 0..2 {
        let took = ask(port, "POST", "/v1/copy", &[("drive", &drive_text)]);
        assert_eq!(field(&took, "ok"), Some("true"), "{took}");
    }
    let kept: Vec<_> = std::fs::read_dir(&scratch)
        .unwrap()
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with(".copy2.before-")
        })
        .collect();
    assert_eq!(kept.len(), 2, "each copy before is kept, under its date");
    assert!(kept[0].path().join("copied.toml").is_file());
    // A copy kept under the same moment is never written over: the next gets
    // a number.
    let record = vleo_files::copied::Copied::from_toml(
        &std::fs::read_to_string(copy2.join("copied.toml")).unwrap(),
    )
    .unwrap();
    let stamp = record.copied_at.replace([':', '-'], "");
    let clash = scratch.join(format!(".copy2.before-{stamp}"));
    std::fs::create_dir_all(&clash).unwrap();
    std::fs::write(clash.join("someone's"), "kept").unwrap();
    let took = ask(port, "POST", "/v1/copy", &[("drive", &drive_text)]);
    assert_eq!(field(&took, "ok"), Some("true"), "{took}");
    assert!(clash.join("someone's").is_file());
    assert!(scratch
        .join(format!(".copy2.before-{stamp}-2"))
        .join("copied.toml")
        .is_file());

    // A record that does not read is refused, never taken for a drive of its own.
    std::fs::write(copy2.join("copied.toml"), "not a record").unwrap();
    let e = open(port, "test-ola.vkey");
    assert!(e.contains("copied.toml"), "{e}");

    // Five wrong passphrases, and the key waits, even for the right one.
    for _ in 0..5 {
        let wrong = ask(
            port,
            "POST",
            "/v1/key/open",
            &[
                ("file", "test-nia.vkey"),
                ("passphrase", "not the passphrase"),
            ],
        );
        assert_eq!(field(&wrong, "ok"), Some("false"), "{wrong}");
    }
    let paused = open(port, "test-nia.vkey");
    assert!(paused.contains("try again in"), "{paused}");
    // That key, not every key.
    std::fs::write(copy2.join("copied.toml"), "").unwrap();
    std::env::set_var("VLEO_COPY", &copy);
    let ola = open(port, "test-ola.vkey");
    assert!(!ola.contains("try again in"), "{ola}");

    // A drive out of reach is said, not passed over in silence.
    std::fs::rename(&drv, scratch.join("drive-moved")).unwrap();
    let ola = open(port, "test-ola.vkey");
    assert!(ola.contains("could not be checked"), "{ola}");

    let _ = std::fs::remove_dir_all(&scratch);
}
