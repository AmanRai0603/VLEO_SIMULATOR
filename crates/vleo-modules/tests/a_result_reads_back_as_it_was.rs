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
    // The run's own rows keep the versions they ran at; the row under test and
    // one more are added to the same line.
    let (s, _, _) = a_result();
    let text = csv(&s);
    let line = text
        .lines()
        .find(|l| l.starts_with("#! versions "))
        .unwrap();
    let own = line.trim_start_matches("#! versions ").replace("none", "");
    let with = |n: u32| {
        text.replace(
            &format!("{line}\n"),
            &format!("#! versions {own} {id}=v{n}@{rel} other_row=v1@0.1.0\n"),
        )
    };
    let current = read(&with(now)).unwrap();
    assert!(current
        .versions
        .contains(&(id.to_string(), now, rel.to_string())));
    assert!(current
        .versions
        .contains(&("other_row".into(), 1, "0.1.0".into())));
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
    let (mut s, _, _) = a_result();
    // A run whose rows had no recorded version when it was saved.
    s.versions.clear();
    s.outputs.clear();
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

#[cfg(feature = "std")]
#[test]
fn a_listing_reads_summaries_and_rebuilds_one_lost_or_stale() {
    use vleo_modules::results::store;
    let dir = std::env::temp_dir().join(format!("vleo-results-index-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let (s, _, _) = a_result();
    let name = store::save(&dir, &s).unwrap();
    let head = dir.join(store::INDEX).join(format!("{name}.head"));
    assert!(head.is_file(), "a save wrote no summary");
    let listed = &store::list(&dir).0[0].1;
    assert_eq!(listed, &s.summary(), "the listing is not the summary");
    assert_eq!(listed.answer(), s.answer());
    assert_eq!(listed.changed(), s.changed());
    assert!(listed.outputs.len() < s.outputs.len() || s.outputs.len() == 1);
    // What the listing shows comes from the summary, not from the result.
    let marked = std::fs::read_to_string(&head)
        .unwrap()
        .replace("#! name ", "#! name from the summary ");
    std::fs::write(&head, marked).unwrap();
    assert!(store::list(&dir).0[0]
        .1
        .name
        .starts_with("from the summary"));
    // Lost: rebuilt from the result.
    std::fs::remove_file(&head).unwrap();
    assert_eq!(store::list(&dir).0[0].1, s.summary());
    assert!(head.is_file(), "a lost summary was not rebuilt");
    // Stale — the result replaced after its summary was written: read whole.
    std::fs::write(
        &head,
        csv(&Saved {
            name: "stale".into(),
            ..s.summary()
        }),
    )
    .unwrap();
    let old = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
    std::fs::File::options()
        .write(true)
        .open(&head)
        .unwrap()
        .set_modified(old)
        .unwrap();
    assert_eq!(
        store::list(&dir).0[0].1,
        s.summary(),
        "a stale summary was believed"
    );
    store::remove(&dir, &name).unwrap();
    assert!(!head.exists(), "a removed result left its summary");
    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(feature = "std")]
#[test]
fn an_old_result_is_thinned_unless_pinned_and_still_runs_again() {
    use vleo_modules::results::store;
    let dir = std::env::temp_dir().join(format!("vleo-results-thin-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let (s, _, _) = a_result();
    let kept = store::save(&dir, &s).unwrap();
    let other = store::save(
        &dir,
        &Saved {
            name: "other".into(),
            chain: "0000".into(),
            ..s.clone()
        },
    )
    .unwrap();
    assert!(store::pin(&dir, "../x.csv", true).is_err());
    store::pin(&dir, &kept, true).unwrap();
    assert!(store::pinned(&dir, &kept));
    // Not old enough: nothing thinned.
    assert!(store::thin(&dir, "2026-09-27", "2026-10-30").is_empty());
    assert_eq!(
        store::thin(&dir, "2026-09-28", "2026-10-30"),
        vec![other.clone()]
    );
    assert_eq!(
        store::open(&dir, &kept).unwrap(),
        s,
        "a pinned result was thinned"
    );
    let thin = store::open(&dir, &other).unwrap();
    assert_eq!(thin.thinned, "2026-10-30");
    assert_eq!(thin.answer(), s.answer(), "thinning lost the answer");
    assert_eq!(
        thin.case_values(),
        s.case_values(),
        "thinning lost what it ran on"
    );
    assert!(html(&thin).contains("Thinned on 2026-10-30"));
    // Pinned stays whole at any age, and thinned once is thinned: the date it
    // happened stays.
    assert!(store::thin(&dir, "2026-12-01", "2026-12-01").is_empty());
    assert_eq!(store::open(&dir, &other).unwrap().thinned, "2026-10-30");
    store::pin(&dir, &kept, false).unwrap();
    assert!(!store::pinned(&dir, &kept));
    assert_eq!(
        store::thin(&dir, "2026-12-01", "2026-12-01"),
        vec![kept.clone()]
    );
    // Its inputs are its case: run again, it is the same run.
    let case = Case {
        target: thin.target.clone(),
        mode: RunMode::Branch,
        supply: thin.case_values(),
        ..Default::default()
    };
    let again = evaluate(&case, &mut Scratch::new()).unwrap();
    assert_eq!(
        again.manifest.chain, s.chain,
        "a thinned result did not run again the same"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(feature = "std")]
#[test]
fn a_share_file_carries_the_result_and_its_report() {
    use vleo_modules::results::share;
    let (s, _, _) = a_result();
    let file = share::pack(&s);
    assert_eq!(share::unpack(&file).unwrap(), s);
    let files = vleo_modules::files::unzip(&file).unwrap();
    let names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["manifest.toml", "result.csv", "report.html"]);
    assert_eq!(
        read(&unwrap_report(&String::from_utf8_lossy(&files[2].1))).unwrap(),
        s
    );
    assert!(share::unpack(b"PK\x03\x04 nope").is_err());
    let foreign = vleo_modules::files::zip(&[("manifest.toml", b"format = \"other\"")]);
    assert!(share::unpack(&foreign)
        .unwrap_err()
        .contains("vleo-share/1"));
}
