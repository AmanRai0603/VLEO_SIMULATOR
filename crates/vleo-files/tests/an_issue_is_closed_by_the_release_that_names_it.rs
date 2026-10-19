//! An issue is resolved by a release whose notes name it, and closes when
//! that release is in today's design and its mark is no longer red
//! (docs/briefs/F5.md, item 6; W11).
//!
//! A release's notes are its de-risking record, the group's `versions.csv`
//! row for its version; the issues it resolves are in its `resolves`
//! column. Every value here is invented for the check.

use vleo_files::chain::FILE;
use vleo_files::issues::{self, State};
use vleo_files::meta::Kind;
use vleo_files::model::*;

fn s(v: &str) -> String {
    v.to_string()
}

/// A release of solar `version` whose record for it resolves `resolves`.
fn release(version: &str, resolves: &str) -> File {
    let mut f = File::new(Kind::GroupRelease, "vleo 1.0.0 (test)");
    f.meta.insert(s("group_id"), s("l3_solar"));
    f.meta.insert(s("version"), s(version));
    f.tables.push(Tbl {
        scope: s(FILE),
        path: s("versions.csv"),
        csv: format!(
            "version,date,by,believed,tested,learned,changed,risks,cost,rests_on,breaks_if,resolves\n\
             1.0,2026-10-01,Ola Sun,,,,,,,the levels,a correction,\n\
             {version},2026-10-10,Ola Sun,b,t,l,c,,,the levels,a correction,\"{resolves}\"\n"
        ),
    });
    f
}

/// Issue `number`, raised against solar.
fn issue(number: i64) -> Issue {
    Issue {
        number,
        group_id: s("l3_solar"),
        raised_by: s("Pia Power"),
        ..Issue::default()
    }
}

/// The numbers alone.
fn numbers(r: &File) -> Vec<u64> {
    issues::resolves(r)
        .unwrap()
        .into_iter()
        .map(|(n, _)| n)
        .collect()
}

#[test]
fn a_release_resolves_the_issues_its_notes_name() {
    assert_eq!(numbers(&release("1.2", "3, 7")), [3, 7]);
    assert_eq!(numbers(&release("1.2", "3;7 12")), [3, 7, 12]);
    assert!(numbers(&release("1.2", "")).is_empty());
    // A later release still resolves what an earlier one did: the record is
    // carried whole, and every row up to its own version is read.
    let mut later = release("1.2", "3");
    later.meta.insert(s("version"), s("1.3"));
    assert_eq!(
        issues::resolves(&later).unwrap(),
        [(3, s("1.2"))],
        "1.3 carries 1.2's row"
    );
    // An earlier release does not resolve what a later row names.
    let mut r = release("1.2", "3");
    r.meta.insert(s("version"), s("1.0"));
    assert!(numbers(&r).is_empty());
    // A release with no record resolves nothing.
    let mut bare = release("1.2", "3");
    bare.tables.clear();
    assert!(numbers(&bare).is_empty());
    // An entry that is not an issue's number is refused by its row.
    let e = issues::resolves(&release("1.2", "3a, 7")).unwrap_err();
    assert!(
        e.contains("versions.csv's row 1.2 resolves \"3a\", which is not an issue's number"),
        "{e}"
    );
}

#[test]
fn an_issue_closes_only_once_its_release_is_taken_and_its_mark_is_not_red() {
    let r = release("1.2", "3");
    let state = |n: i64, who: Option<&str>, taken: Option<&File>, red: bool| {
        issues::state(&issue(n), who, taken, red).unwrap()
    };
    assert_eq!(
        state(3, None, Some(&r), false),
        State::Closed { by: s("1.2") }
    );
    assert_eq!(
        state(3, Some("Nia Node"), Some(&r), true),
        State::Resolved { by: s("1.2") }
    );
    assert_eq!(
        state(4, Some("Nia Node"), Some(&r), true),
        State::Assigned(s("Nia Node"))
    );
    assert_eq!(state(4, None, Some(&r), false), State::Open);
    assert_eq!(state(3, None, None, false), State::Open);
    // Closed by 1.2, it stays closed once 1.3 is taken.
    let mut next = release("1.2", "3");
    next.meta.insert(s("version"), s("1.3"));
    assert_eq!(
        state(3, None, Some(&next), false),
        State::Closed { by: s("1.2") }
    );
    assert_eq!(
        issues::State::Closed { by: s("1.2") }.said("l3_solar"),
        "closed by l3_solar 1.2"
    );
    // Another group's release never resolves solar's issue 3.
    let mut power = release("1.2", "3");
    power.meta.insert(s("group_id"), s("l3_power"));
    let e = issues::state(&issue(3), None, Some(&power), false).unwrap_err();
    assert!(
        e.contains("the release given is l3_power's, and issue 3 is addressed to l3_solar"),
        "{e}"
    );
    // Closed before 1.0, as its upgraded file says: read as it was said.
    let mut old = issue(5);
    old.closed_by = s("Ola Sun");
    assert_eq!(
        issues::state(&old, None, None, true).unwrap(),
        State::ClosedBefore1_0 { by: s("Ola Sun") }
    );
}

#[test]
fn an_issue_s_assignment_is_read_from_its_group_file() {
    let mut g = File::new(Kind::Group, "vleo 1.0.0 (test)");
    g.texts.push(Text {
        scope: s(FILE),
        kind: s("issue 3"),
        body: s("Nia Node"),
    });
    assert_eq!(issues::assigned(&g, 3).as_deref(), Some("Nia Node"));
    assert_eq!(issues::assigned(&g, 4), None);
}
