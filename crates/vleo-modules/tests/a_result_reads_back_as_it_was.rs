//! A saved result: what one run returned and the inputs it ran on, kept as a
//! CSV and read back exactly — without running anything again.
//!
//! A result travels: it is saved, sent, opened in a spreadsheet, uploaded
//! somewhere else. What matters is that what comes back is the same numbers bit
//! for bit, that its inputs are the case it ran on, and that a file which is not
//! a result is refused by name rather than half read.

use vleo_bus::{Case, RunMode};
use vleo_modules::inputs::case_inputs;
use vleo_modules::results::{csv, from_run, html, read, unwrap_report, Saved};
use vleo_modules::{evaluate, Scratch};

/// A run of an input row as its own target, at a value off its default.
fn a_result() -> (Saved, f64, &'static str) {
    let i = case_inputs()
        .into_iter()
        .find(|i| i.default > i.lo && i.default < i.hi)
        .expect("no input with room to move");
    let v = (i.default + i.hi) / 3.0 + i.lo / 3.0;
    let case = Case {
        target: i.id.into(),
        mode: RunMode::Branch,
        supply: vec![(i.id.to_string(), v)],
        ..Default::default()
    };
    let r = evaluate(&case, &mut Scratch::new()).expect("the input did not run");
    (
        from_run(
            &r,
            &case.supply,
            "2026-09-27T12:00:00Z",
            "a, \"quoted\" name",
        ),
        v,
        i.id,
    )
}

#[test]
fn a_result_reads_back_exactly() {
    let (s, v, id) = a_result();
    let back = read(&csv(&s)).expect("a result did not read back");
    assert_eq!(back, s, "what was saved is not what came back");
    let answer = back.answer().expect("no answer");
    assert_eq!(answer.si, Some(v), "the answer is not the same f64");
    assert_eq!(back.changed(), 1);
    assert_eq!(back.case_values(), vec![(id.to_string(), v)]);
    assert_eq!(
        back.inputs.len(),
        case_inputs().len(),
        "not every input is recorded"
    );
}

#[test]
fn the_report_carries_the_result_and_reads_back_as_it() {
    let (s, _, _) = a_result();
    let page = html(&s);
    assert!(page.contains("<script type=\"text/csv\" id=\"vleo-result\">"));
    assert!(!page[page.find("id=\"vleo-result\"").unwrap()..]
        .trim_end()
        .trim_end_matches("</html>")
        .trim_end()
        .trim_end_matches("</body>")
        .trim_end()
        .trim_end_matches("</script>")
        .contains("</script>"));
    assert_eq!(read(&unwrap_report(&page)).unwrap(), s);
}

#[test]
fn what_is_not_a_result_is_refused_by_name() {
    assert!(read("id,value\nx,1\n")
        .unwrap_err()
        .contains("not a saved result"));
    let (s, _, _) = a_result();
    let text = csv(&s);
    assert!(read(&text.replace("vleo-result/1", "vleo-result/0"))
        .unwrap_err()
        .contains("vleo-result/0"));
    let sideways = text.replacen("\noutput,", "\nsideways,", 1);
    assert!(read(&sideways).unwrap_err().contains("sideways"));
}

#[cfg(feature = "std")]
#[test]
fn results_are_kept_listed_and_removed_and_a_path_is_never_followed() {
    use vleo_modules::results::store;
    let dir = std::env::temp_dir().join(format!("vleo-results-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let (s, _, _) = a_result();
    let name = store::save(&dir, &s).unwrap();
    std::fs::write(dir.join("broken.csv"), "not a result").unwrap();
    let (good, bad) = store::list(&dir);
    assert_eq!(good.len(), 1);
    assert_eq!(
        bad.len(),
        1,
        "an unreadable file must be named, not dropped"
    );
    assert_eq!(store::open(&dir, &name).unwrap(), s);
    for evil in ["../etc/passwd", "/etc/passwd", "a/../../b.csv", "x.toml"] {
        assert!(store::open(&dir, evil).is_err(), "{evil} was opened");
        assert!(store::remove(&dir, evil).is_err(), "{evil} was removed");
    }
    store::remove(&dir, &name).unwrap();
    assert!(store::list(&dir).0.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_result_keeps_the_beliefs_it_rests_on_and_says_when_one_breaks() {
    // The only rows with a recorded history read a reference-data bundle, and
    // whether a bundle is on this machine is not what is being tested. So the
    // record is written into a result the way `from_run` writes it — one
    // `#! versions` line — and read back.
    let (id, now, rel) = vleo_modules::tables::NODE_VERSIONS
        .iter()
        .copied()
        .find(|(_, n, _)| *n >= 2)
        .expect("no node with two recorded versions");
    assert_eq!(vleo_modules::results::node_version(id), Some((now, rel)));
    let (s, _, _) = a_result();
    let with = |n: u32| {
        csv(&s).replace(
            "#! versions none\n",
            &format!("#! versions {id}=v{n}@{rel} other_row=v1@0.1.0\n"),
        )
    };
    let current = read(&with(now)).unwrap();
    assert_eq!(
        current.versions,
        vec![
            (id.to_string(), now, rel.to_string()),
            ("other_row".into(), 1, "0.1.0".into())
        ]
    );
    assert!(vleo_modules::results::moved_since(&current).is_empty());
    assert_eq!(
        read(&csv(&current)).unwrap(),
        current,
        "the versions did not survive a round trip"
    );

    // Saved when the row was one version behind: the belief has broken since,
    // and the report says so in its first lines.
    let old = read(&with(now - 1)).unwrap();
    assert_eq!(
        vleo_modules::results::moved_since(&old),
        vec![(id.to_string(), now - 1, now)]
    );
    let page = html(&old);
    assert!(
        page.contains("since broken"),
        "the report does not say a belief broke"
    );
    assert!(
        page.find("Answer first").unwrap() < page.find("Every value the run returned").unwrap()
    );
}

#[test]
fn a_row_whose_first_belief_came_after_the_result_is_a_belief_that_moved() {
    // A run through rows with no recorded version says so — `#! versions none`
    // — so a row's first version after it is a belief the result rested on
    // without anyone having written it down, and that has moved since. A
    // result saved before the tool recorded beliefs says nothing either way.
    let (id, now, _) = vleo_modules::tables::NODE_VERSIONS[0];
    let (s, _, _) = a_result();
    let text = csv(&s);
    assert!(text.contains("#! versions none\n"), "{text}");
    let with_row = text.replace(
        "section,id,name,value,unit,si,credibility,governing,note\n",
        &format!(
            "section,id,name,value,unit,si,credibility,governing,note\noutput,{id},x,1,-,1,1,x,\n"
        ),
    );
    let then = read(&with_row).unwrap();
    assert_eq!(
        vleo_modules::results::moved_since(&then),
        vec![(id.to_string(), 0, now)]
    );
    assert!(html(&then).contains("had no recorded belief when this ran"));
    let before_beliefs = read(&with_row.replace("#! versions none\n", "")).unwrap();
    assert!(vleo_modules::results::moved_since(&before_beliefs).is_empty());
}
