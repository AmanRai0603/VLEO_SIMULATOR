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

#[test]
fn a_release_resolves_the_issues_its_notes_name() {
    assert_eq!(issues::resolves(&release("1.2", "3, 7")), [3, 7]);
    assert_eq!(issues::resolves(&release("1.2", "3;7 12")), [3, 7, 12]);
    assert!(issues::resolves(&release("1.2", "")).is_empty());
    // Another version's row says nothing of this one.
    let mut r = release("1.2", "3");
    r.meta.insert(s("version"), s("1.0"));
    assert!(issues::resolves(&r).is_empty());
    // A release with no record resolves nothing.
    let mut bare = release("1.2", "3");
    bare.tables.clear();
    assert!(issues::resolves(&bare).is_empty());
}

#[test]
fn an_issue_closes_only_once_its_release_is_taken_and_its_mark_is_not_red() {
    let r = release("1.2", "3");
    let taken = Some(("1.2", &r));
    assert_eq!(
        issues::state(3, None, taken, false),
        State::Closed { by: s("1.2") }
    );
    assert_eq!(
        issues::state(3, Some("Nia Node"), taken, true),
        State::Resolved { by: s("1.2") }
    );
    assert_eq!(
        issues::state(4, Some("Nia Node"), taken, true),
        State::Assigned(s("Nia Node"))
    );
    assert_eq!(issues::state(4, None, taken, false), State::Open);
    assert_eq!(issues::state(3, None, None, false), State::Open);
    assert_eq!(
        issues::State::Closed { by: s("1.2") }.said("l3_solar"),
        "closed by l3_solar 1.2"
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
