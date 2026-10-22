//! A file's issue and change rows are signed with it once it says so
//! (docs/briefs/F6.md, F6-d; `chain::SIGNED_ROWS`).
//!
//! An issue file of one invented person, Sam, raising issue 4 on solar, with
//! a change row beside it. Nothing here is computed: every row is written in
//! the test, and every check is the chain's own.
//!
//! - marked `signed_rows = "issue change"` and signed, it checks; its issue
//!   row changed after it was signed, or a change row added or changed, it
//!   does not;
//! - a comment added, which others write, it still checks;
//! - the marker taken off after it was signed, it does not; a marker with
//!   any other value is refused by name;
//! - unmarked, as every file signed before F6-d, its rows are read as they
//!   were: changed after it was signed, it still checks, and its digest is
//!   the one the library gave before F6-d, recorded at 1a65459d;
//! - a release sealed whole, one of its change rows changed, does not check.

use vleo_files::chain::{self, FILE, ROWS_SIGNED, SIGNED_ROWS};
use vleo_files::keys::SigningKey;
use vleo_files::meta::Kind;
use vleo_files::model::*;

const APP: &str = "vleo 1.0.0 (test)";
const AT: &str = "2026-10-22T09:00:00Z";

fn s(v: &str) -> String {
    v.to_string()
}

fn key(seed: u8) -> SigningKey {
    SigningKey::from_seed([seed; 32])
}

/// Sam's issue 4 on solar, with one change row, marked or not.
fn issue_file(marked: bool) -> File {
    let mut f = File::new(Kind::Issue, APP);
    for (k, v) in [
        ("number", "4"),
        ("group_id", "l3_solar"),
        ("writer", "Sam Engel"),
    ] {
        f.meta.insert(s(k), s(v));
    }
    f.issues.push(Issue {
        number: 4,
        group_id: s("l3_solar"),
        raised_by: s("Sam Engel"),
        at: s(AT),
        place: s("closure:l3_solar_ach_01"),
        what: s("The margin is gone at solar maximum."),
        evidence: s(""),
        addressed_to: s("Ola Sun"),
        closed_by: s(""),
    });
    f.changes.push(Change {
        at: s(AT),
        who: s("Sam Engel"),
        scope: s(FILE),
        what: s("raised"),
        before: None,
        after: None,
    });
    if marked {
        chain::sign_rows(&mut f);
    }
    f
}

fn signed(mut f: File) -> File {
    let row = chain::sign(&f, FILE, "Sam Engel", &key(12), AT, "ok", "raised").unwrap();
    f.signatures.push(row);
    f
}

fn checks(f: &File) -> Result<(), String> {
    let row = f.signatures.iter().find(|r| r.scope == FILE).unwrap();
    chain::check_row(f, row).map_err(|e| e.message().to_string())
}

#[test]
fn a_marked_file_s_issue_and_change_rows_are_signed_with_it() {
    let f = signed(issue_file(true));
    assert_eq!(
        f.meta.get(SIGNED_ROWS).map(String::as_str),
        Some(ROWS_SIGNED)
    );
    checks(&f).expect("as signed, it checks");
    let mut what = f.clone();
    what.issues[0].addressed_to = s("Pat Morgan");
    let mut more = f.clone();
    more.changes.push(Change {
        at: s(AT),
        who: s("Ola Sun"),
        scope: s(FILE),
        what: s("closed"),
        before: None,
        after: None,
    });
    let mut changed = f.clone();
    changed.changes[0].what = s("withdrawn");
    let mut unmarked = f.clone();
    unmarked.meta.remove(SIGNED_ROWS);
    for (how, g) in [
        ("its issue row changed", what),
        ("a change row added", more),
        ("a change row changed", changed),
        ("its marker taken off", unmarked),
    ] {
        assert!(checks(&g).is_err(), "{how}");
    }
    // A comment is others' to write into it: it is not signed.
    let mut commented = f.clone();
    commented.comments.push(Comment {
        scope: s(FILE),
        section: s(""),
        author: s("Ola Sun"),
        at: s(AT),
        body: s("Seen."),
        resolved: 0,
    });
    checks(&commented).expect("a comment added, it still checks");
}

#[test]
fn a_marker_with_another_value_is_refused_by_name() {
    let mut f = issue_file(false);
    f.meta.insert(s(SIGNED_ROWS), s("issue"));
    let e = chain::digest(&f, FILE).unwrap_err();
    assert!(
        e.message().contains("signed_rows says \"issue\"")
            && e.message().contains("\"issue change\""),
        "{}",
        e.message()
    );
}

#[test]
fn an_unmarked_file_is_read_as_every_file_signed_before_it() {
    let f = signed(issue_file(false));
    checks(&f).expect("as signed, it checks");
    let mut changed = f.clone();
    changed.issues[0].addressed_to = s("Pat Morgan");
    changed.changes[0].what = s("withdrawn");
    checks(&changed).expect("its rows are not what it signed");
}

/// The digest of the unmarked issue file as the library gave it before F6-d
/// (1a65459d), not as this code gives it: a file signed before is hashed as
/// it was, and so is every fingerprint a released design names.
const BEFORE_F6D: &str = "b36844864047f71ea9ed3a043a9c4f616711c173cfca6a0157e7ae32ec1a1a3c";

#[test]
fn an_unmarked_file_hashes_as_it_did_before_f6_d() {
    assert_eq!(chain::digest(&issue_file(false), FILE).unwrap(), BEFORE_F6D);
    assert_ne!(chain::digest(&issue_file(true), FILE).unwrap(), BEFORE_F6D);
}

/// What `chain::check_issue` refuses before any signature is read: a file of
/// another kind, of no issue or two, whose row is not what its meta says, or
/// raised by another than its writer. With no registry to count a key, a
/// file as it should be is refused only for its signature.
#[test]
fn an_issue_file_is_refused_by_name_where_it_is_not_one() {
    let good = signed(issue_file(true));
    let e = chain::check_issue(&good, &[]).unwrap_err();
    assert!(
        e.message().contains("issue 4 on l3_solar, by Sam Engel,"),
        "{}",
        e.message()
    );
    let mut node = good.clone();
    node.meta.insert(s("file_kind"), s("node"));
    let mut none = good.clone();
    none.issues.clear();
    let mut two = good.clone();
    two.issues.push(good.issues[0].clone());
    let mut number = good.clone();
    number.issues[0].number = 5;
    let mut group = good.clone();
    group.issues[0].group_id = s("l3_power");
    let mut raiser = good.clone();
    raiser.issues[0].raised_by = s("Ola Sun");
    for (f, says) in [
        (node, "it is not an issue file"),
        (none, "it holds 0 issues, and an issue file holds one"),
        (two, "it holds 2 issues"),
        (
            number,
            "its row is issue 5 on l3_solar, and its meta says issue 4",
        ),
        (group, "its row is issue 4 on l3_power"),
        (
            raiser,
            "it is raised by \"Ola Sun\", and its meta says \"Sam Engel\" wrote it",
        ),
    ] {
        let e = chain::check_issue(&f, &[]).unwrap_err();
        assert!(e.message().contains(says), "{says}: {}", e.message());
    }
}
