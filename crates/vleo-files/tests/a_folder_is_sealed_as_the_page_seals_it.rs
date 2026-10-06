//! The seal of a group's folder, said by the library as the page says it.
//!
//! The oracle is what the page did: Solar 1.0 and 1.1 were each sealed in the
//! group application, which seals only when its rules (`web/js/gseal.js`) see
//! nothing in the way, and which wrote the fingerprint it sealed into the
//! file. So on both, the library must find the sealed fingerprint, every
//! sign-off current, and nothing in the way. `tools/files_check.mjs` holds the
//! library to the page's own functions on the same files and on broken copies,
//! in CI. Here a handful of breaks are refused in the page's words.

use std::path::Path;

use vleo_files::folder::{Finding, Level};
use vleo_files::format_1::{Folder, Old};
use vleo_files::seal::{self, Member};
use vleo_files::sqlite;

fn solar(version: &str) -> Old {
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/fixtures/l3_solar-{version}.vleo"));
    sqlite::read_format_1(&p).unwrap()
}

fn carried() -> vleo_files::folder::Spec {
    vleo_files::folder::Spec::carried().unwrap()
}

#[test]
fn solar_as_the_page_sealed_it() {
    for v in ["1.0", "1.1"] {
        let old = solar(v);
        let f = old.folder();
        let sealed = &old.meta["fingerprint"];
        assert_eq!(
            seal::check_sealed(&f, sealed).as_deref(),
            Ok(sealed.as_str()),
            "Solar {v}"
        );
        let s = seal::state(&f, &carried());
        assert!(s.blockers.is_empty(), "Solar {v}: {:#?}", s.blockers);
        assert!(
            !s.reviews.is_empty() && s.reviews.iter().all(|r| r.current),
            "Solar {v}"
        );
        assert_eq!(
            s.scopes[0],
            ("group".to_string(), sealed.clone()),
            "Solar {v}"
        );
    }
}

fn text(f: &mut Folder, path: &str, how: impl Fn(&str) -> String) {
    let t = String::from_utf8(f[path].clone()).unwrap();
    f.insert(path.to_string(), how(&t).into_bytes());
}

#[test]
fn one_byte_changed_after_signing_and_its_sign_off_is_stale() {
    let old = solar("1.1");
    let mut f = old.folder();
    text(&mut f, "nodes/sw_regime/pseudocode.txt", |t| {
        format!("{t} ")
    });
    assert!(seal::check_sealed(&f, &old.meta["fingerprint"])
        .unwrap_err()
        .starts_with("the files are not the ones that were sealed"));
    let s = seal::state(&f, &carried());
    assert_eq!(
        s.blockers,
        [
            "sw_regime has no current sign-off from its node engineer",
            "the group has no current sign-off from its owner",
        ]
    );
    let stale: Vec<&str> = s
        .reviews
        .iter()
        .filter(|r| !r.current)
        .map(|r| r.scope.as_str())
        .collect();
    assert!(
        stale.contains(&"sw_regime") && stale.contains(&"group"),
        "{stale:?}"
    );
    // The folder's record of itself is not what is signed: a new sign-off,
    // an issue, a package leave every fingerprint as it was.
    let mut g = old.folder();
    text(&mut g, "reviews.csv", |t| {
        format!("{t}somebody,group,1.1,x,2026-10-06,ok,\n")
    });
    g.insert("issues/a.md".into(), b"# a\n".to_vec());
    g.insert("packages/x.zip".into(), vec![0]);
    assert!(seal::check_sealed(&g, &old.meta["fingerprint"]).is_ok());
}

#[test]
fn who_may_sign_what() {
    let people = [
        Member {
            name: "O".into(),
            role: "owner".into(),
            nodes: String::new(),
        },
        Member {
            name: "E".into(),
            role: "engineer".into(),
            nodes: "a  b".into(),
        },
        Member {
            name: "S".into(),
            role: "engineer".into(),
            nodes: "*".into(),
        },
    ];
    assert!(seal::may_sign(&people, "O", "group"));
    assert!(seal::may_sign(&people, "O", "a"));
    assert!(!seal::may_sign(&people, "E", "group"));
    assert!(seal::may_sign(&people, "E", "b"));
    assert!(!seal::may_sign(&people, "E", "c"));
    assert!(seal::may_sign(&people, "S", "c"));
    assert!(!seal::may_sign(&people, "S", "group"));
    assert!(!seal::may_sign(&people, "nobody", "a"));
}

#[test]
fn a_seal_waits_for_the_errors_and_for_the_owner() {
    let mut f = solar("1.1").folder();
    // Nobody is the owner any more: the group's sign-off no longer counts,
    // and neither does any node that only the owner signed.
    text(&mut f, "members.csv", |t| {
        t.replace(",owner,", ",engineer,")
    });
    let error = Finding {
        level: Level::Error,
        place: "nodes.csv".into(),
        msg: "x".into(),
        line: 0,
    };
    let b = seal::blockers(&f, &[error.clone(), error]);
    assert_eq!(
        b.first().map(String::as_str),
        Some("2 error(s) in the checks")
    );
    assert_eq!(
        b.last().map(String::as_str),
        Some("the group has no current sign-off from its owner")
    );
}
