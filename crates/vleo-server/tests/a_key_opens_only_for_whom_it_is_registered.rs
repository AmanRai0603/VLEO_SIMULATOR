//! A key is made and opened in the program, and it opens only for the person
//! the drive's files register with it (docs/briefs/F1.md).
//!
//! A small drive is written here: a programme of invented people signed by its
//! manager, START HERE naming that key, the systems' file mounting solar, and
//! solar's group file registering its node engineer. Then, through the
//! program's routes: a key made and listed, a stranger's key opening nothing, a
//! wrong passphrase refused, the node engineer opened and closed, and a
//! changed anchor refused. One test, because the program's environment and its
//! open key are the process's own.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use vleo_files::chain::{self, FILE};
use vleo_files::keys::{SigningKey, MIN_ITERATIONS};
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::sqlite;

const APP: &str = "vleo 1.0.0 (test)";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn ask(port: u16, method: &str, path: &str, body: &str) -> String {
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

fn register(f: &mut File, name: &str, role: &str, key: &SigningKey, by: &str) {
    f.people.push(Person {
        name: s(name),
        role: s(role),
        deputy_for: s(""),
    });
    f.keys.push(PersonKey {
        person: s(name),
        public_key: key.public().to_hex(),
        registered_at: s("2026-01-01"),
        registered_by: s(by),
        revoked_from: s(""),
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
    f.meta.insert(s("based_on"), s(""));
    f
}

fn sign_whole(f: &mut File, who: &str, key: &SigningKey) {
    let row = chain::sign(f, FILE, who, key, "2026-02-01", "ok", "").unwrap();
    f.signatures.push(row);
}

fn write_key(dir: &Path, file: &str, person: &str, key: &SigningKey, passphrase: &str) {
    let locked = key
        .lock(person, passphrase, [7; 16], [7; 12], MIN_ITERATIONS)
        .unwrap();
    let mut f = File::new(Kind::Key, APP);
    f.meta.insert(s("person"), s(person));
    f.locked_keys.push(locked.to_row());
    sqlite::write(&f, &dir.join(file)).unwrap();
}

/// The drive, and the keys of the people it registers.
fn drive(at: &Path, keys: &Path) {
    let pm = SigningKey::from_seed([21; 32]);
    let se = SigningKey::from_seed([22; 32]);
    let sub = SigningKey::from_seed([23; 32]);
    let ne = SigningKey::from_seed([24; 32]);
    for g in ["programme", "systems", "l3_solar", "l3_thermal", "l3_copy"] {
        std::fs::create_dir_all(at.join("groups").join(g)).unwrap();
    }
    let mut programme = group("programme");
    register(
        &mut programme,
        "Test Pat",
        "programme manager",
        &pm,
        "Test Pat",
    );
    register(
        &mut programme,
        "Test Sam",
        "system engineer",
        &se,
        "Test Pat",
    );
    register(
        &mut programme,
        "Test Ola",
        "subsystem engineer",
        &sub,
        "Test Pat",
    );
    mount(&mut programme, "root", "systems", "Test Sam");
    sign_whole(&mut programme, "Test Pat", &pm);
    sqlite::write(&programme, &at.join("groups/programme/programme.vgroup")).unwrap();

    let mut systems = group("systems");
    mount(
        &mut systems,
        "sys_space_environment",
        "l3_solar",
        "Test Ola",
    );
    sign_whole(&mut systems, "Test Sam", &se);
    sqlite::write(&systems, &at.join("groups/systems/systems.vgroup")).unwrap();

    let mut solar = group("l3_solar");
    register(&mut solar, "Test Nia", "node engineer", &ne, "Test Ola");
    solar.assignments.push(Assignment {
        block_uid: s("b-ap"),
        person: s("Test Nia"),
        contract_version: 1,
    });
    // Solar's owner mounts thermal in her own file, and signs thermal herself:
    // only the programme's and the systems' files mount, so it is not hers.
    mount(&mut solar, "b-thermal", "l3_thermal", "Test Ola");
    sign_whole(&mut solar, "Test Ola", &sub);
    sqlite::write(&solar, &at.join("groups/l3_solar/l3_solar.vgroup")).unwrap();
    let mut thermal = group("l3_thermal");
    sign_whole(&mut thermal, "Test Ola", &sub);
    sqlite::write(&thermal, &at.join("groups/l3_thermal/l3_thermal.vgroup")).unwrap();
    // Solar's file again, in another group's folder.
    sqlite::write(&solar, &at.join("groups/l3_copy/l3_copy.vgroup")).unwrap();

    std::fs::write(
        at.join("START HERE.md"),
        format!(
            "# START HERE: an invented drive\n\nThe programme manager's key:\n\n    {}\n",
            pm.public().fingerprint()
        ),
    )
    .unwrap();
    std::fs::create_dir_all(keys).unwrap();
    write_key(
        keys,
        "test-nia.vkey",
        "Test Nia",
        &ne,
        "the node engineer's passphrase",
    );
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

#[test]
fn a_key_opens_only_for_whom_it_is_registered() {
    let scratch = std::env::temp_dir().join(format!("vleo-keys-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    let (drv, keys) = (scratch.join("drive"), scratch.join("keys"));
    drive(&drv, &keys);
    std::env::set_var("VLEO_KEYS", &keys);
    std::env::set_var("VLEO_DRIVE", &drv);
    std::env::set_var("VLEO_CASE", scratch.join("case.csv"));
    std::env::set_var("VLEO_RESULTS", scratch.join("results"));
    std::env::set_var("VLEO_LOG", scratch.join("log"));
    std::env::set_var("VLEO_DATA", scratch.join("data"));
    std::env::remove_var("VLEO_DESIGN");
    let port = vleo_server::serve(Some(root()), 18961, false, true).expect("no server");

    // Nobody yet.
    let me = ask(port, "GET", "/v1/me", "");
    assert_eq!(field(&me, "open"), Some("false"), "{me}");

    // A key made, and refused when it would be weak or written over.
    let short = ask(
        port,
        "POST",
        "/v1/key/make",
        "name=Test%20Stranger&passphrase=short",
    );
    assert_eq!(field(&short, "ok"), Some("false"), "{short}");
    let made = ask(
        port,
        "POST",
        "/v1/key/make",
        "name=Test%20Stranger&passphrase=a%20long%20enough%20passphrase",
    );
    assert_eq!(field(&made, "ok"), Some("true"), "{made}");
    assert!(keys.join("test-stranger.vkey").is_file());
    let again = ask(
        port,
        "POST",
        "/v1/key/make",
        "name=Test%20Stranger&passphrase=another%20long%20passphrase",
    );
    assert!(again.contains("never written over"), "{again}");
    let listed = ask(port, "GET", "/v1/keys", "");
    assert!(
        listed.contains("test-nia.vkey") && listed.contains("Test Stranger"),
        "{listed}"
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(keys.join("test-stranger.vkey"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600, "a key file is its person's alone");
    }
    let wide = ask(
        port,
        "POST",
        "/v1/key/make",
        "name=%E6%9D%8E%E6%98%8E&passphrase=a%20long%20enough%20passphrase",
    );
    let wide_file = field(&wide, "file").unwrap_or("").to_string();
    assert!(
        field(&wide, "ok") == Some("true") && !wide_file.ends_with("/.vkey"),
        "{wide}"
    );

    // The first time this computer opens the drive, the anchor is shown, and
    // nothing is remembered until it is accepted.
    let pm = SigningKey::from_seed([21; 32]).public().fingerprint();
    let first = ask(
        port,
        "POST",
        "/v1/key/open",
        "file=test-stranger.vkey&passphrase=a%20long%20enough%20passphrase",
    );
    assert_eq!(field(&first, "anchor"), Some(pm.as_str()), "{first}");
    assert!(!keys.join("anchors").exists());
    let other = SigningKey::from_seed([98; 32]).public().fingerprint();
    let wrong_anchor = ask(
        port,
        "POST",
        "/v1/key/open",
        &format!("file=test-stranger.vkey&passphrase=a%20long%20enough%20passphrase&accept_anchor={other}"),
    );
    assert_eq!(
        field(&wrong_anchor, "anchor"),
        Some(pm.as_str()),
        "{wrong_anchor}"
    );
    assert!(!keys.join("anchors").exists());

    // The stranger's key opens, but the drive registers nobody with it.
    let stranger = ask(
        port,
        "POST",
        "/v1/key/open",
        &format!(
            "file=test-stranger.vkey&passphrase=a%20long%20enough%20passphrase&accept_anchor={pm}"
        ),
    );
    assert!(keys.join("anchors").is_file());
    assert!(
        stranger.contains("opens nothing here") && stranger.contains("no file registers this key"),
        "{stranger}"
    );
    assert_eq!(
        field(&ask(port, "GET", "/v1/me", ""), "open"),
        Some("false")
    );

    // The node engineer: a wrong passphrase refused, the right one opened.
    let wrong = ask(
        port,
        "POST",
        "/v1/key/open",
        "file=test-nia.vkey&passphrase=not%20it%20at%20all",
    );
    assert_eq!(field(&wrong, "ok"), Some("false"), "{wrong}");
    assert!(
        !wrong.contains("not it at all"),
        "a passphrase is never echoed: {wrong}"
    );
    let nia = ask(
        port,
        "POST",
        "/v1/key/open",
        "file=test-nia.vkey&passphrase=the%20node%20engineer%27s%20passphrase",
    );
    assert_eq!(field(&nia, "name"), Some("Test Nia"), "{nia}");
    assert!(
        nia.contains("l3_thermal: no file mounts the group")
            && nia.contains("l3_copy: ")
            && nia.contains("says it is the group \\\"l3_solar\\\""),
        "{nia}"
    );
    assert!(
        nia.contains("\"role\":\"node engineer\"") && nia.contains("\"block\":\"b-ap\""),
        "{nia}"
    );
    let me = ask(port, "GET", "/v1/me", "");
    assert_eq!(
        (field(&me, "open"), field(&me, "name")),
        (Some("true"), Some("Test Nia")),
        "{me}"
    );

    // Closed, it is gone; and a failed open leaves nobody's key open.
    ask(port, "POST", "/v1/key/close", "");
    assert_eq!(
        field(&ask(port, "GET", "/v1/me", ""), "open"),
        Some("false")
    );
    ask(
        port,
        "POST",
        "/v1/key/open",
        "file=test-nia.vkey&passphrase=the%20node%20engineer%27s%20passphrase",
    );
    ask(
        port,
        "POST",
        "/v1/key/open",
        "file=test-nia.vkey&passphrase=not%20it%20at%20all",
    );
    assert_eq!(
        field(&ask(port, "GET", "/v1/me", ""), "open"),
        Some("false")
    );

    // Only a key the folder lists is reached for.
    std::fs::write(drv.join("x.vkey"), b"not a key").unwrap();
    for file in [
        "..%2Fdrive%2Fx.vkey",
        "..%5Cdrive%5Cx.vkey",
        "C%3Ax.vkey",
        ".vkey",
    ] {
        let out = ask(
            port,
            "POST",
            "/v1/key/open",
            &format!("file={file}&passphrase=whatever%20it%20is"),
        );
        assert!(out.contains("name the key by its file"), "{file}: {out}");
    }

    // START HERE names another key: this computer accepted the first one.
    let other = SigningKey::from_seed([99; 32]).public().fingerprint();
    std::fs::write(
        drv.join("START HERE.md"),
        format!("# START HERE\n\n    {other}\n"),
    )
    .unwrap();
    let moved = ask(
        port,
        "POST",
        "/v1/key/open",
        "file=test-nia.vkey&passphrase=the%20node%20engineer%27s%20passphrase",
    );
    assert!(
        moved.contains("accepted") && moved.contains("note signed by the old key"),
        "{moved}"
    );

    let _ = std::fs::remove_dir_all(&scratch);
}
