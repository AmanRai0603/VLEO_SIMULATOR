//! A lesson form carries what its check needs and gives back what was written.
//!
//! What must hold: the form carries the row it is for, the tree's rows, the
//! checker, and the lesson as it stands; what a filled form holds is read back
//! as the TOML the gate reads; a form for one row is refused for another, and
//! a bare lesson.toml needs its row named; nothing a lesson says can end the
//! script element it travels in.

use std::path::PathBuf;
use vleo_sheet::lesson_form::{document, from_file};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn a_form_carries_its_row_the_rows_the_checker_and_the_lesson() {
    let tree = vleo_sheet::load::load_all(&root()).unwrap();
    let html = document(&tree.sheets["orbit_velocity"], &tree).unwrap();
    for id in [
        "vleo-lesson-json",
        "vleo-lesson",
        "vleo-lesson-rows",
        "vleo-method-wasm",
    ] {
        assert!(
            html.contains(&format!("id=\"{id}\"")),
            "the form has no {id} block"
        );
    }
    assert!(html.contains("node orbit_velocity\n"));
    assert!(
        html.contains("row orbit_altitude d p\n"),
        "the row table does not mark a declared row"
    );
    assert!(
        html.contains("row orbit_radius c p\n"),
        "the row table does not mark a computed row"
    );
    let (node, toml_text) = from_file(&html, None).unwrap();
    assert_eq!(node, "orbit_velocity");
    assert!(
        toml_text.trim().is_empty(),
        "a row with no lesson carried one"
    );
}

#[test]
fn what_a_filled_form_holds_is_read_back_as_the_gate_reads_it() {
    let tree = vleo_sheet::load::load_all(&root()).unwrap();
    let html = document(&tree.sheets["orbit_velocity"], &tree).unwrap();
    let example =
        std::fs::read_to_string(root().join("docs/examples/orbit_velocity.lesson.toml")).unwrap();
    // As the page saves it: the lesson block rewritten in place.
    let a = html.find("id=\"vleo-lesson\">").unwrap() + "id=\"vleo-lesson\">".len();
    let b = a + html[a..].find("</textarea>").unwrap();
    let filled = format!("{}{}{}", &html[..a], example, &html[b..]);
    let (node, t) = from_file(&filled, Some("orbit_velocity")).unwrap();
    let l = vleo_sheet::lesson::read(&t, &node).unwrap();
    assert!(vleo_sheet::lesson::problems(&l, &tree).is_empty());
    assert!(from_file(&filled, Some("orbit_radius"))
        .unwrap_err()
        .message()
        .contains("is for 'orbit_velocity'"));
    assert!(from_file(&example, None)
        .unwrap_err()
        .message()
        .contains("--for"));
    assert_eq!(
        from_file(&example, Some("orbit_velocity")).unwrap().1,
        example
    );
}

#[test]
fn nothing_a_lesson_says_ends_the_script_it_travels_in() {
    let tree = vleo_sheet::load::load_all(&root()).unwrap();
    let sh = tree.sheets["orbit_velocity"].clone();
    let d = std::env::temp_dir().join(format!("vleo-lesson-form-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    let mut sh = sh;
    sh.dir = d.clone();
    std::fs::write(
        d.join("lesson.toml"),
        "[lesson]\ntitle = \"a </script><script>alert(1)</script>\"\nby = \"x\"\nanswer = \"y\"\nkind = \"explanation\"\n",
    )
    .unwrap();
    let html = document(&sh, &tree).unwrap();
    let scripts_opened = html.matches("<script").count();
    assert!(!html.contains("<script><script>"));
    let scripts_closed = html.matches("</script>").count();
    assert_eq!(
        scripts_opened, scripts_closed,
        "a lesson's text closed or opened a script"
    );
    let (_, back) = from_file(&html, None).unwrap();
    assert!(
        back.contains("a </script><script>alert(1)</script>"),
        "the text did not come back as written"
    );
    let _ = std::fs::remove_dir_all(&d);
}
