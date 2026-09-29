//! A lesson reads, or is refused with why; and its checks hold it to the tree.
//!
//! What must hold: the example lesson reads and passes against the real tree;
//! a misspelt key is refused, not ignored; and each way a lesson can mislead —
//! a sourced claim with no source, markup in the text, a widget naming a row
//! that is not there or asking a reader to move a computed one, a check whose
//! answer is not one of its options — is named.

use std::path::PathBuf;
use vleo_sheet::lesson::{json, problems, read};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn example() -> String {
    std::fs::read_to_string(root().join("docs/examples/orbit_velocity.lesson.toml")).unwrap()
}

#[test]
fn the_example_lesson_reads_and_passes() {
    let tree = vleo_sheet::load::load_all(&root()).unwrap();
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
    assert!(e.contains("`sourse`"), "{e}");
    let e = read(
        &format!("{}\n[extra]\nx = 1\n", example()),
        "orbit_velocity",
    )
    .unwrap_err();
    assert!(e.contains("`extra`"), "{e}");
}

#[test]
fn what_would_mislead_a_reader_is_named() {
    let tree = vleo_sheet::load::load_all(&root()).unwrap();
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
    let tree = vleo_sheet::load::load_all(&root()).unwrap();
    let mut sh = tree.sheets["orbit_velocity"].clone();
    let d = std::env::temp_dir().join(format!("vleo-lesson-gate-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    sh.dir = d.clone();
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
    std::fs::write(d.join("lesson.toml"), example()).unwrap();
    assert_eq!(verdict(&sh), Some(false), "the example lesson was refused");
    std::fs::write(
        d.join("lesson.toml"),
        example().replacen("answer = 3", "answer = 9", 1),
    )
    .unwrap();
    assert_eq!(verdict(&sh), Some(true), "a bad lesson passed the gate");
    let _ = std::fs::remove_dir_all(&d);
}
