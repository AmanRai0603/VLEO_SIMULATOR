//! A lesson reads, or is refused with why; and its checks hold it to the tree.
//!
//! What must hold: the example lesson reads and passes against the design, read
//! from `design/`;
//! a misspelt key is refused, not ignored; and each way a lesson can mislead —
//! a sourced claim with no source, markup in the text, a widget naming a row
//! that is not there or asking a reader to move a computed one, a check whose
//! answer is not one of its options — is named.

use std::path::PathBuf;
use vleo_sheet::lesson::{json, problems, read};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The design, read from `design/`.
fn design() -> vleo_sheet::Tree {
    vleo_files::convert::open(&root()).unwrap().0
}

fn example() -> String {
    std::fs::read_to_string(root().join("docs/examples/orbit_velocity.lesson.toml")).unwrap()
}

#[test]
fn the_example_lesson_reads_and_passes() {
    let tree = design();
    let l = read(&example(), "orbit_velocity").unwrap();
    assert_eq!(problems(&l, &tree), Vec::<String>::new());
    assert_eq!(l.stations.len(), 3);
    assert_eq!(l.widgets[0].outputs, vec!["orbit_velocity", "orbit_radius"]);
    assert_eq!(l.checks[0].answer, 3);
    let j = json(&l);
    assert!(
        j.starts_with("{\"node\":\"orbit_velocity\"") && j.contains("\"sweep\":\"orbit_altitude\"")
    );
}

#[test]
fn a_misspelt_key_is_refused_not_ignored() {
    let t = example().replacen("source = \"vallado2013\"", "sourse = \"vallado2013\"", 1);
    let e = read(&t, "orbit_velocity").unwrap_err();
    assert_eq!(e.kind(), vleo_sheet::ErrorKind::Malformed, "{e}");
    assert!(e.message().contains("`sourse`"), "{e}");
    let e = read(
        &format!("{}\n[extra]\nx = 1\n", example()),
        "orbit_velocity",
    )
    .unwrap_err();
    assert_eq!(e.kind(), vleo_sheet::ErrorKind::Malformed, "{e}");
    assert!(e.message().contains("`extra`"), "{e}");
}

#[test]
fn what_would_mislead_a_reader_is_named() {
    let tree = design();
    for (from, to, says) in [
        (
            "claim = \"sourced\"\nsource = \"vallado2013\"\n\n[[widget]]",
            "claim = \"sourced\"\n\n[[widget]]",
            "a sourced claim names its source",
        ),
        (
            "treating the orbit as a circle.",
            "treating the orbit as a <b>circle</b>.",
            "'<b>' is markup",
        ),
        (
            "inputs = [\"orbit_altitude\"]",
            "inputs = [\"orbit_altitud\"]",
            "input 'orbit_altitud' is not a row",
        ),
        (
            "inputs = [\"orbit_altitude\"]",
            "inputs = [\"orbit_radius\"]",
            "input 'orbit_radius' is computed",
        ),
        (
            "answer = 3",
            "answer = 4",
            "answer 4 is not one of its 3 options",
        ),
        (
            "kind = \"explanation\"",
            "kind = \"essay\"",
            "kind 'essay' is not one of",
        ),
        (
            "claim = \"derived\"",
            "claim = \"obvious\"",
            "claim 'obvious' is not one of",
        ),
        (
            "sweep = \"orbit_altitude\"",
            "sweep = \"orbit_eccentricity\"",
            "sweep 'orbit_eccentricity' is not one of its inputs",
        ),
    ] {
        let t = example();
        assert!(t.contains(from), "the example no longer contains {from:?}");
        let l = read(&t.replacen(from, to, 1), "orbit_velocity").unwrap();
        let p = problems(&l, &tree);
        assert!(
            p.iter().any(|x| x.contains(says)),
            "not named: «{says}» in {p:?}"
        );
    }
}

#[test]
fn the_gate_reads_a_rows_lesson_and_refuses_a_bad_one() {
    let tree = design();
    let mut sh = tree.sheets["orbit_velocity"].clone();
    let verdict = |sh: &vleo_sheet::model::Sheet| {
        vleo_sheet::gate::gate_node(sh, &tree)
            .into_iter()
            .find(|c| c.name == "lesson")
            .map(|c| c.failed())
    };
    assert_eq!(
        verdict(&sh),
        None,
        "a row with no lesson was checked for one"
    );
    sh.lesson = Some(example());
    assert_eq!(verdict(&sh), Some(false), "the example lesson was refused");
    sh.lesson = Some(example().replacen("answer = 3", "answer = 9", 1));
    assert_eq!(verdict(&sh), Some(true), "a bad lesson passed the gate");
}

/// The check a lesson form runs in the browser is the gate's own: fed the
/// form's plain text — the row table the form carries, then the TOML — it
/// names exactly the problems the gate names, for a good lesson and a bad one.
#[test]
fn the_forms_check_is_the_gates_check() {
    let tree = design();
    let rows = vleo_sheet::lesson::rows_block(&tree);
    for text in [
        example(),
        example()
            .replacen(
                "inputs = [\"orbit_altitude\"]",
                "inputs = [\"orbit_radius\"]",
                1,
            )
            .replacen("answer = 3", "answer = 5", 1),
    ] {
        let gate = problems(&read(&text, "orbit_velocity").unwrap(), &tree);
        let form = vleo_sheet::lesson::report(&format!("node orbit_velocity\n{rows}---\n{text}"));
        let want = format!(
            "{{\"ok\":{},\"read\":\"\",\"problems\":[{}]}}",
            gate.is_empty(),
            gate.iter()
                .map(|p| format!("\"{}\"", p.replace('\\', "\\\\").replace('"', "\\\"")))
                .collect::<Vec<_>>()
                .join(",")
        );
        assert_eq!(form, want);
    }
    let bad = vleo_sheet::lesson::report(&format!(
        "node orbit_velocity\n{rows}---\n[lesson]\nsourse = 1\n"
    ));
    assert!(
        bad.starts_with("{\"ok\":false,\"read\":\"") && bad.contains("sourse"),
        "{bad}"
    );
}
