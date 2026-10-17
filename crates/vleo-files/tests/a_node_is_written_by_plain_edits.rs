//! A node is written by plain edits, in the working copy (docs/briefs/F3.md,
//! F3-g2a): its question, the explanation's three texts, its method and its
//! cases, each refused by name when it would break a rule, and saved as a save
//! is; and its dry run, checked as it is typed, writes nothing.
//!
//! Solar 1.1's `sw_activity_band` in a node file of its own, assigned to Nia,
//! a node engineer of solar; Ivo is solar's other node engineer, assigned
//! nothing. The people are invented, as People's test has them. The cases are
//! the release's own; no expected value here comes from the code under test.
//!
//! - a question is a decision: saved as revision 2 only with a record of
//!   which belief broke, a row of the node's own `versions.csv` numbered 2,
//!   and refused without one, naming what the record is missing; nothing but
//!   the question and the record changes;
//! - a section of the explanation is replaced alone, the others kept byte for
//!   byte, with no record; one that holds display maths is refused;
//! - a method or cases edit restates the declaration, refused for a relation
//!   an assistant supplied, for none said, for an assistant's name, for a
//!   transcription nobody checked; a case whose origin is the code's own, an
//!   assistant's or none is refused by name;
//! - the dry run of a method that breaks a case writes nothing and says which
//!   case it breaks; a dry run missing its record says so beside what was
//!   typed;
//! - a file changed since it was opened is a second writer, for a save and a
//!   dry run alike; Ivo may not write it; a node in design/'s shape, and an
//!   edit that changes nothing, are refused.

use std::path::{Path, PathBuf};
use vleo_files::chain::{self, Registry, FILE};
use vleo_files::edit::{self, Declared, Edit, Edits, Part, Tried};
use vleo_files::keys::SigningKey;
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::save::Saved;
use vleo_sheet::template::Derisk;

const APP: &str = "vleo 1.0.0 (test)";
const AT: &str = "2026-10-17T09:00:00Z";
const BAND: &str = "sw_activity_band";
const PATH: &str = "groups/l3_solar/nodes/sw_activity_band.vnode";

fn s(v: &str) -> String {
    v.to_string()
}

fn key(seed: u8) -> SigningKey {
    SigningKey::from_seed([seed; 32])
}

fn register(f: &mut File, name: &str, role: &str, deputy_for: &str, k: &SigningKey) {
    f.people.push(Person {
        name: s(name),
        role: s(role),
        deputy_for: s(deputy_for),
    });
    f.keys.push(PersonKey {
        person: s(name),
        public_key: k.public().to_hex(),
        registered_at: s("2026-01-01"),
        registered_by: s("Pat Morgan"),
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

fn group_file(id: &str) -> File {
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

// Pat the programme manager (11), Sam the system engineer (12), Ola who owns
// solar (13), Dee her deputy (14), Nia a node engineer of solar (16).
fn programme() -> Registry {
    let mut f = group_file("programme");
    register(&mut f, "Pat Morgan", "programme manager", "", &key(11));
    register(&mut f, "Sam Engel", "system engineer", "", &key(12));
    register(&mut f, "Ola Sun", "subsystem engineer", "", &key(13));
    register(
        &mut f,
        "Dee Puty",
        "subsystem engineer",
        "Ola Sun",
        &key(14),
    );
    mount(&mut f, "root", "systems", "Sam Engel");
    chain::open_programme(
        signed(f, "Pat Morgan", &key(11)),
        &key(11).public().fingerprint(),
    )
    .unwrap()
}

fn systems(pr: &Registry) -> Registry {
    let mut f = group_file("systems");
    mount(&mut f, "sys_space_environment", "l3_solar", "Ola Sun");
    chain::open_systems(&signed(f, "Sam Engel", &key(12)), pr).unwrap()
}

/// Solar's group file: its heading, Nia assigned the band, and Ivo.
fn solar() -> Registry {
    let mut f = group_file("l3_solar");
    f.blocks.push(Block {
        uid: s("l3_solar"),
        id: s("l3_solar"),
        behaviour: s("children"),
        ..Block::default()
    });
    for (name, seed) in [("Nia Node", 16), ("Ivo Other", 17)] {
        f.people.push(Person {
            name: s(name),
            role: s("node engineer"),
            deputy_for: s(""),
        });
        f.keys.push(PersonKey {
            person: s(name),
            public_key: key(seed).public().to_hex(),
            registered_at: s("2026-01-01"),
            registered_by: s("Ola Sun"),
            revoked_from: s(""),
            proof: vleo_files::people::proof(&key(seed), name),
        });
    }
    f.assignments.push(Assignment {
        block_uid: s(BAND),
        person: s("Nia Node"),
        contract_version: 1,
    });
    let pr = programme();
    let sy = systems(&pr);
    chain::open_group_in(&signed(f, "Ola Sun", &key(13)), &sy, &pr).unwrap()
}

/// `uid` from solar 1.1's release, in a node file of its own.
fn from_release(uid: &str) -> File {
    let how = vleo_files::upgrade::Upgrade { app: APP, at: AT };
    let rel = vleo_files::sqlite::open(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/l3_solar-1.1.vleo"),
        &how,
    )
    .unwrap()
    .file;
    let mut n = File::new(Kind::Node, APP);
    for (k, v) in [
        ("group_id", "l3_solar"),
        ("block_uid", uid),
        ("revision", "1"),
        ("contract_version", "1"),
        ("writer", "Nia Node"),
    ] {
        n.meta.insert(s(k), s(v));
    }
    n.blocks = vec![rel.blocks.iter().find(|b| b.uid == uid).unwrap().clone()];
    n.blocks[0].parent_uid = s("l3_solar");
    n.blocks[0].revision = 1;
    n.ports = rel
        .ports
        .iter()
        .filter(|p| p.block_uid == uid)
        .cloned()
        .collect();
    n.wires = rel
        .wires
        .iter()
        .filter(|w| w.to_block == uid)
        .cloned()
        .collect();
    n.texts = rel
        .texts
        .iter()
        .filter(|t| t.scope == uid)
        .cloned()
        .collect();
    n.tables = rel
        .tables
        .iter()
        .filter(|t| t.scope == uid)
        .cloned()
        .collect();
    n
}

fn record() -> Derisk {
    Derisk {
        believed: s("The question named the band, not what it is for."),
        tested: s("Read beside the drag nodes that use it."),
        learned: s("They read it as the level of activity."),
        changed: s("The question asks for the level."),
        rests_on: s("NOAA's four F10.7 levels."),
        breaks_if: s("A node reads it as a flux."),
        ..Derisk::default()
    }
}

fn nia() -> Declared {
    Declared {
        author: s("Nia Node"),
        ai: s("none"),
        source: s(""),
        checked_by: s(""),
    }
}

fn one(e: Edit) -> Edits {
    Edits {
        edits: vec![e],
        ..Edits::default()
    }
}

fn text<'a>(f: &'a File, kind: &str) -> &'a str {
    f.texts
        .iter()
        .find(|t| t.scope == BAND && t.kind == kind)
        .map_or("", |t| t.body.as_str())
}

fn table<'a>(f: &'a File, path: &str) -> &'a str {
    f.tables
        .iter()
        .find(|t| t.scope == BAND && t.path == path)
        .map_or("", |t| t.csv.as_str())
}

fn apply(e: &Edits) -> Result<edit::Applied, String> {
    edit::apply(&from_release(BAND), PATH, e, "Nia Node", AT).map_err(|e| e.message().to_string())
}

/// A copy holding the band, and the SHA-256 of its bytes.
fn copy(name: &str) -> (PathBuf, String) {
    let c = std::env::temp_dir().join(format!("vleo-edit-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&c);
    let at = c.join(PATH);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    vleo_files::sqlite::write(&from_release(BAND), &at).unwrap();
    let d = digest(&c);
    (c, d)
}

fn digest(c: &Path) -> String {
    vleo_files::copied::digest(&std::fs::read(c.join(PATH)).unwrap())
}

#[test]
fn a_question_is_saved_with_the_record_of_which_belief_broke() {
    let ask = s("Which level of solar activity does this F10.7 fall in?");
    let e = one(Edit::Question(ask.clone()));
    let refused = apply(&e).unwrap_err();
    assert!(
        refused.contains("does not say which belief broke")
            && refused.contains("believed, tested, learned, changed, rests_on, breaks_if"),
        "{refused}"
    );
    let (c, opened) = copy("question");
    let e = Edits {
        why: Some(record()),
        ..e
    };
    let saved = edit::edit_in_place(&c, PATH, &opened, &e, "Nia Node", AT, APP, &solar()).unwrap();
    let Saved::Written { revision, .. } = saved else {
        panic!("written: {saved:?}")
    };
    assert_eq!(revision, 2);
    let after = vleo_files::sqlite::read(&c.join(PATH)).unwrap();
    let was = from_release(BAND);
    assert_eq!(after.blocks[0].question, ask);
    let versions = table(&after, "versions.csv");
    assert!(
        versions.starts_with("version,date,by,believed,tested,learned,changed,risks,cost,rests_on,breaks_if\n2,2026-10-17T09:00:00Z,Nia Node,\"The question named the band, not what it is for.\""),
        "{versions}"
    );
    // Nothing but the question and the record changes.
    assert_eq!(after.ports, was.ports);
    assert_eq!(after.wires, was.wires);
    assert_eq!(after.texts, was.texts);
    assert_eq!(
        after
            .tables
            .iter()
            .filter(|t| t.path != "versions.csv")
            .collect::<Vec<_>>(),
        was.tables.iter().collect::<Vec<_>>()
    );
    assert_eq!(after.blocks[0].behaviour, was.blocks[0].behaviour);
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn a_section_of_the_explanation_is_replaced_alone() {
    let was = from_release(BAND);
    let sections = |t: &str| -> Vec<(String, String)> {
        let mut out: Vec<(String, String)> = Vec::new();
        for l in t.split('\n') {
            if let Some(h) = l.strip_prefix("## ") {
                out.push((h.to_string(), String::new()));
            } else if let Some(last) = out.last_mut() {
                last.1.push_str(l);
                last.1.push('\n');
            }
        }
        out
    };
    let said = s("A reading of 228 means little alone: the band says it is high.");
    let a =
        apply(&one(Edit::Explain(Part::Simply, said.clone()))).expect("wording needs no record");
    let (old, new) = (
        sections(text(&was, "explanation")),
        sections(text(&a.file, "explanation")),
    );
    assert_eq!(old.len(), new.len());
    for (o, n) in old.iter().zip(&new) {
        assert_eq!(o.0, n.0);
        if o.0 == "Said simply" {
            assert_eq!(n.1.trim(), said);
        } else {
            assert_eq!(o.1, n.1, "{} is kept byte for byte", o.0);
        }
    }
    assert!(a.about.is_empty());
    assert_eq!(table(&a.file, "versions.csv"), "");
    // Display maths belongs in the theory, as the folder's checks say.
    let e = apply(&one(Edit::Explain(
        Part::Breaks,
        s("Where $$F > 170$$ it saturates."),
    )))
    .unwrap_err();
    assert!(e.contains("display maths belongs in theory.md"), "{e}");
    let e = apply(&one(Edit::Explain(Part::Wrong, s("  ")))).unwrap_err();
    assert!(e.contains("would say nothing"), "{e}");
}

#[test]
fn a_method_or_its_cases_restate_the_declaration_and_their_origins() {
    let band = from_release(BAND);
    let method = text(&band, "pseudocode").to_string();
    let m = || Edit::Method(method.replace("# Transcribed from", "# Written again from"));
    let with = |d: Option<Declared>| Edits {
        edits: vec![m()],
        declared: d,
        why: Some(record()),
    };
    let e = apply(&with(None)).unwrap_err();
    assert!(e.contains("Silence is not \"none\""), "{e}");
    for (d, says) in [
        (
            Declared {
                ai: s("relation"),
                ..nia()
            },
            "an assistant supplied the relation",
        ),
        (Declared { ai: s(""), ..nia() }, "declaration"),
        (
            Declared {
                author: s("Claude"),
                ..nia()
            },
            "an assistant's name",
        ),
        (
            Declared {
                ai: s("transcribed"),
                source: s("model.rs"),
                ..nia()
            },
            "names nobody who checked the copy",
        ),
    ] {
        let e = apply(&with(Some(d.clone()))).unwrap_err();
        assert!(
            e.contains("declaration does not hold") && e.contains(says),
            "{d:?}: {e}"
        );
    }
    // Without its record, a method is refused like a question.
    let e = apply(&Edits {
        why: None,
        ..with(Some(nia()))
    })
    .unwrap_err();
    assert!(
        e.contains("math") && e.contains("does not say which belief broke"),
        "{e}"
    );
    let a = apply(&with(Some(nia()))).unwrap();
    assert!(table(&a.file, "declaration.csv")
        .starts_with("author,ai,date,source,checked_by\nNia Node,none,2026-10-17,,"));
    assert!(text(&a.file, "pseudocode").contains("# Written again from"));

    // The cases: each origin a reference, never the code's own.
    let cases = table(&band, "results/isolation.csv").to_string();
    for bad in ["self-snapshot", "agent-generated", ""] {
        let e = apply(&Edits {
            edits: vec![Edit::Cases(cases.replacen(
                ",paper,",
                &format!(",{bad},"),
                1,
            ))],
            declared: Some(nia()),
            why: None,
        })
        .unwrap_err();
        assert!(
            e.contains("never comes from the code under test")
                && e.contains(&format!("line 2: {bad:?}")),
            "{bad}: {e}"
        );
    }
    let a = apply(&Edits {
        edits: vec![Edit::Cases(cases.replacen(",paper,", ",hand,", 1))],
        declared: Some(nia()),
        why: None,
    })
    .expect("cases are not a decision: no record");
    assert!(table(&a.file, "results/isolation.csv").contains(",hand,"));
}

#[test]
fn the_dry_run_writes_nothing_and_says_what_breaks() {
    let (c, opened) = copy("dry");
    let band = from_release(BAND);
    let wrong = text(&band, "pseudocode").replace("if f107 >= 130 then", "if f107 >= 135 then");
    let e = Edits {
        edits: vec![Edit::Method(wrong)],
        declared: Some(nia()),
        why: Some(record()),
    };
    let t = edit::try_edit(&c, PATH, &opened, &e, "Nia Node", AT, APP, &solar()).unwrap();
    let Tried::Would {
        refused,
        findings,
        report,
        revision,
        ..
    } = t
    else {
        panic!("would: {t:?}")
    };
    assert_eq!(digest(&c), opened, "a dry run writes nothing");
    assert_eq!((refused, revision), (None, 2));
    assert!(
        findings
            .errors()
            .any(|f| f.what.contains("does not reproduce «130 sfu exactly")),
        "{:?}",
        findings.errors().collect::<Vec<_>>()
    );
    let (cases, rep) = report.expect("its method runs on its cases");
    assert_eq!(cases.len(), 10);
    assert_eq!(rep.cases.iter().filter(|(_, v)| !v.agrees()).count(), 1);
    // Missing its record: said, beside what was typed.
    let e = Edits { why: None, ..e };
    let Tried::Would { refused, file, .. } =
        edit::try_edit(&c, PATH, &opened, &e, "Nia Node", AT, APP, &solar()).unwrap()
    else {
        panic!()
    };
    assert!(refused.unwrap().contains("does not say which belief broke"));
    assert!(text(&file, "pseudocode").contains(">= 135"));
    assert_eq!(digest(&c), opened);
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn each_is_refused_where_a_save_is() {
    let (c, opened) = copy("refused");
    let q = Edits {
        edits: vec![Edit::Question(s("Which level of activity?"))],
        why: Some(record()),
        ..Edits::default()
    };
    // A second writer, for a save and a dry run alike.
    let stale = "0".repeat(64);
    let saved = edit::edit_in_place(&c, PATH, &stale, &q, "Nia Node", AT, APP, &solar()).unwrap();
    assert!(matches!(saved, Saved::SecondWriter { .. }), "{saved:?}");
    let tried = edit::try_edit(&c, PATH, &stale, &q, "Nia Node", AT, APP, &solar()).unwrap();
    assert!(matches!(tried, Tried::SecondWriter { .. }), "{tried:?}");
    // One writer per file.
    for e in [
        edit::edit_in_place(&c, PATH, &opened, &q, "Ivo Other", AT, APP, &solar())
            .map(|_| ())
            .unwrap_err(),
        edit::try_edit(&c, PATH, &opened, &q, "Ivo Other", AT, APP, &solar())
            .map(|_| ())
            .unwrap_err(),
    ] {
        assert!(
            e.message().contains("one writer per file"),
            "{}",
            e.message()
        );
    }
    assert_eq!(digest(&c), opened);
    // Nothing changed is not a revision, whatever record comes with it: a
    // record says something changed.
    let same = from_release(BAND).blocks[0].question.clone();
    let e = apply(&Edits {
        edits: vec![Edit::Question(same)],
        why: Some(record()),
        ..Edits::default()
    })
    .unwrap_err();
    assert!(e.contains("changes nothing"), "{e}");
    let simply = from_release(BAND);
    let body = text(&simply, "explanation");
    let said = body
        .split("## Said simply")
        .nth(1)
        .unwrap()
        .split("\n## ")
        .next()
        .unwrap()
        .trim()
        .to_string();
    let e = apply(&one(Edit::Explain(Part::Simply, said))).unwrap_err();
    assert!(e.contains("changes nothing"), "{e}");
    // A node in design/'s shape is not written here yet.
    let mut design = from_release(BAND);
    design.texts.retain(|t| t.kind != "pseudocode");
    design.tables.clear();
    design.texts.push(Text {
        scope: s(BAND),
        kind: s("sheet"),
        body: s("[node]\n"),
    });
    let e = edit::apply(&design, PATH, &q, "Nia Node", AT).unwrap_err();
    assert!(e.message().contains("F3-g0, question 9"), "{}", e.message());
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn a_section_never_begins_hides_or_moves_another() {
    let band = from_release(BAND);
    let explain = text(&band, "explanation").to_string();
    // A heading typed into a section, or a fence left open: refused.
    for (body, says) in [
        (
            "New.\n## Where it breaks\nInjected.",
            "would begin another section",
        ),
        ("Look:\n```\ncode", "leaves a code fence open"),
    ] {
        let e = apply(&one(Edit::Explain(Part::Simply, s(body)))).unwrap_err();
        assert!(e.contains(says), "{body:?}: {e}");
    }
    // A fence the file already leaves open runs across the sections after
    // it: an edit of a later one is refused, never written as a second.
    let mut open = band.clone();
    for t in open.texts.iter_mut().filter(|t| t.kind == "explanation") {
        t.body = t
            .body
            .replacen("## Said simply\n", "## Said simply\n\n```\n", 1);
    }
    let e = edit::apply(
        &open,
        PATH,
        &one(Edit::Explain(
            Part::Breaks,
            s("Above 300 sfu the bands say no more."),
        )),
        "Nia Node",
        AT,
    )
    .unwrap_err();
    assert!(
        e.message().contains("would not hold its other sections"),
        "{}",
        e.message()
    );
    // A section the text lacks goes where the pattern puts it.
    let mut without = band.clone();
    let start = explain.find("## Where it breaks").unwrap();
    let end = explain.find("## Common misreading").unwrap();
    for t in without.texts.iter_mut().filter(|t| t.kind == "explanation") {
        t.body = format!("{}{}", &explain[..start], &explain[end..]);
    }
    let a = edit::apply(
        &without,
        PATH,
        &one(Edit::Explain(
            Part::Breaks,
            s("Above 300 sfu the bands say no more."),
        )),
        "Nia Node",
        AT,
    )
    .unwrap();
    let body = text(&a.file, "explanation");
    let at = |h: &str| body.find(&format!("## {h}")).unwrap();
    assert!(
        at("Guess first") < at("Where it breaks")
            && at("Where it breaks") < at("Common misreading"),
        "{body}"
    );
    assert_eq!(
        edit::section(&a.file, BAND, Part::Breaks),
        "Above 300 sfu the bands say no more."
    );
    // A text with CRLF line endings keeps them, and its other sections.
    let mut crlf = band.clone();
    for t in crlf.texts.iter_mut().filter(|t| t.kind == "explanation") {
        t.body = t.body.replace('\n', "\r\n");
    }
    let a = edit::apply(
        &crlf,
        PATH,
        &one(Edit::Explain(Part::Wrong, s("Band 4 is not a storm."))),
        "Nia Node",
        AT,
    )
    .unwrap();
    let body = text(&a.file, "explanation");
    assert!(
        !body.replace("\r\n", "").contains('\n'),
        "one line ending throughout"
    );
    for part in [Part::Simply, Part::Breaks] {
        assert_eq!(
            edit::section(&a.file, BAND, part),
            edit::section(&crlf, BAND, part)
        );
    }
}

#[test]
fn cases_are_read_whole_every_row_held_to_its_origin() {
    let band = from_release(BAND);
    let cases = table(&band, "results/isolation.csv").to_string();
    // A later row, not only the first.
    let lines: Vec<&str> = cases.lines().collect();
    let late = lines[4].replacen(",paper,", ",agent-generated,", 1);
    let bad = cases.replacen(lines[4], &late, 1);
    let e = apply(&Edits {
        edits: vec![Edit::Cases(bad)],
        declared: Some(nia()),
        why: None,
    })
    .unwrap_err();
    assert!(e.contains("line 5: \"agent-generated\""), "{e}");
    // A quote left open is not read as rows that hide in one field.
    let open = cases.replacen(",paper,\"", ",paper,\"\"\"", 1);
    let e = apply(&Edits {
        edits: vec![Edit::Cases(format!("{open}\"unclosed"))],
        declared: Some(nia()),
        why: None,
    })
    .unwrap_err();
    assert!(e.contains("do not read as CSV"), "{e}");
}

#[test]
fn a_refused_part_is_never_checked_as_the_file_in_its_place() {
    let (c, opened) = copy("unchecked");
    let band = from_release(BAND);
    let cases = table(&band, "results/isolation.csv").replacen(",paper,", ",self-snapshot,", 1);
    let e = Edits {
        edits: vec![Edit::Cases(cases)],
        declared: Some(nia()),
        why: None,
    };
    let t = edit::try_edit(&c, PATH, &opened, &e, "Nia Node", AT, APP, &solar()).unwrap();
    let Tried::Would {
        refused,
        checked,
        findings,
        report,
        ..
    } = t
    else {
        panic!("{t:?}")
    };
    assert!(refused
        .unwrap()
        .contains("never comes from the code under test"));
    assert!(!checked && findings.0.is_empty() && report.is_none());
    let _ = std::fs::remove_dir_all(&c);
}
