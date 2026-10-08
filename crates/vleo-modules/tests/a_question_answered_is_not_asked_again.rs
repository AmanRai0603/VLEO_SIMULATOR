//! A question already answered is shown from its saved result, not asked again.
//!
//! A result is kept as a folder — its values, its sweep when it is one, and the
//! report page to send — under a key worked out from everything that decides
//! the answer: the row, how much of the graph, the engine and tree, the
//! reference data, every input it ran at, and what a sweep moved. The same key
//! is the same answer, so a request that works it out finds the result that
//! holds it, on this laptop or from a shared folder, and a result is kept once.
//!
//! What must hold for that to be safe: the key a request works out equals the
//! key of the result it saved, after the result has been written and read
//! back; any input moved, or a sweep asked differently, is a different key; a
//! sweep and its report come back whole; and a folder that is not a result is
//! never removed.

#![cfg(feature = "std")]

mod design;

use vleo_bus::{Case, RunMode};
use vleo_modules::inputs::case_inputs;
use vleo_modules::results::{
    csv, from_run, html, question_for, read, read_sweep, store, sweep_csv, unwrap_report,
    unwrap_sweep, Saved, Sweep,
};
use vleo_modules::{evaluate, Scratch};

fn input() -> vleo_modules::inputs::Input {
    case_inputs()
        .into_iter()
        .find(|i| i.default > i.lo && i.default < i.hi)
        .expect("no input with room to move")
}

fn case_at(v: f64) -> Case {
    let i = input();
    Case {
        target: i.id.into(),
        mode: RunMode::Branch,
        supply: vec![(i.id.to_string(), v)],
        ..Default::default()
    }
}

fn saved(case: &Case) -> Saved {
    let r = evaluate(case, &mut Scratch::new()).expect("the input did not run");
    from_run(&r, &case.supply, "2026-09-29T12:00:00Z", "")
}

/// A sweep over the input itself, with one refused point kept.
fn a_sweep() -> Sweep {
    let i = input();
    Sweep {
        over: i.id.into(),
        over_name: i.label.into(),
        x_unit: i.unit.into(),
        x_factor: i.factor,
        y_unit: i.unit.into(),
        y_factor: i.factor,
        from: i.lo,
        to: i.hi,
        points: 3,
        x: vec![i.lo, i.default],
        y: vec![i.lo, i.default],
        refused: vec![(i.hi, "a reason, with a comma".into())],
    }
}

fn dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("vleo-reuse-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

#[test]
fn the_key_a_request_works_out_is_the_key_of_the_result_it_saved() {
    crate::design::open();
    let i = input();
    let v = (i.default + i.hi) / 3.0 + i.lo / 3.0;
    let case = case_at(v);
    let s = saved(&case);
    let asked = question_for(&case, None);
    assert_eq!(
        s.question(),
        asked,
        "the saved result does not answer its own question"
    );
    let back = read(&csv(&s)).unwrap();
    assert_eq!(
        back.question(),
        asked,
        "written and read back, it answers another question"
    );
    // A request that supplies the same value twice, or the defaults and then
    // the value, asks the same question: the last value supplied is the one
    // the run applies.
    let mut twice = case.clone();
    twice.supply.insert(0, (i.id.to_string(), i.default));
    assert_eq!(question_for(&twice, None), asked);
}

#[test]
fn anything_that_changes_the_answer_changes_the_question() {
    crate::design::open();
    let i = input();
    let base = case_at(i.default);
    let q = question_for(&base, None);
    let moved = case_at((i.default + i.hi) / 2.0);
    assert_ne!(
        question_for(&moved, None),
        q,
        "a moved input asked the same question"
    );
    let mut other_mode = base.clone();
    other_mode.mode = RunMode::All;
    assert_ne!(
        question_for(&other_mode, None),
        q,
        "another mode asked the same question"
    );
    let mut other_data = base.clone();
    other_data.data_versions = vec!["solar-drivers@2099.01.01#ffff".into()];
    assert_ne!(
        question_for(&other_data, None),
        q,
        "other data asked the same question"
    );
    let spec = (i.id, i.lo, i.hi, 20);
    let sq = question_for(&base, Some(spec));
    assert_ne!(
        sq, q,
        "a sweep asked the same question as the run at its case"
    );
    for other in [(i.id, i.lo, i.hi, 21), (i.id, i.lo, i.default, 20)] {
        assert_ne!(
            question_for(&base, Some(other)),
            sq,
            "{other:?} asked the same sweep"
        );
    }
}

#[test]
fn a_sweep_and_its_report_come_back_whole() {
    crate::design::open();
    let w = a_sweep();
    assert_eq!(
        read_sweep(&sweep_csv(&w)).unwrap(),
        w,
        "the sweep did not read back"
    );
    let mut s = saved(&case_at(input().default));
    s.sweep = Some(w.clone());
    let page = html(&s);
    assert!(page.contains("<svg"), "the report does not draw the sweep");
    let mut back = read(&unwrap_report(&page)).unwrap();
    back.sweep = Some(read_sweep(&unwrap_sweep(&page).expect("no sweep in the page")).unwrap());
    assert_eq!(back, s, "the report did not carry the result and its sweep");
    let e = read_sweep("x,y\n1,2\n").unwrap_err();
    assert_eq!(e.kind(), vleo_modules::ErrorKind::Malformed, "{e}");
    assert!(e.message().contains("not a saved sweep"), "{e}");
}

#[test]
fn a_question_is_kept_once_and_found_again() {
    crate::design::open();
    let d = dir("once");
    let case = case_at(input().default);
    let s = saved(&case);
    let (name, already) = store::save(&d, &s).unwrap();
    assert!(!already);
    for f in [store::RESULT, store::REPORT] {
        assert!(d.join(&name).join(f).is_file(), "{f} was not kept");
    }
    // Saved again, a second later: the same question, so the same result.
    let mut again = s.clone();
    again.saved = "2026-09-29T12:00:01Z".into();
    assert_eq!(store::save(&d, &again).unwrap(), (name.clone(), true));
    assert_eq!(store::list(&d).0.len(), 1, "one question was kept twice");
    let (found, got) = store::find(&d, &question_for(&case, None)).expect("not found");
    assert_eq!((found, got), (name.clone(), s.clone()));
    assert!(store::find(&d, &question_for(&case_at(input().hi), None)).is_none());

    // A sweep over the same case is another question, kept beside it.
    let mut sw = s.clone();
    sw.sweep = Some(a_sweep());
    let (sname, already) = store::save(&d, &sw).unwrap();
    assert!(!already && sname != name);
    assert!(d.join(&sname).join(store::SWEEP).is_file());
    assert_eq!(store::open(&d, &sname).unwrap(), sw);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn a_result_kept_as_one_file_is_still_read_and_what_is_not_a_result_is_left_alone() {
    crate::design::open();
    let d = dir("old");
    std::fs::create_dir_all(&d).unwrap();
    let s = saved(&case_at(input().default));
    // Before results were folders, each was one CSV.
    std::fs::write(d.join("2026-09-01_old_abc.csv"), csv(&s)).unwrap();
    let (good, _) = store::list(&d);
    assert_eq!(good.len(), 1);
    assert_eq!(
        store::find(&d, &s.question()).unwrap().0,
        "2026-09-01_old_abc.csv"
    );
    // Kept once holds across the two forms: the old file already answers it.
    assert_eq!(
        store::save(&d, &s).unwrap(),
        ("2026-09-01_old_abc.csv".into(), true)
    );
    // A folder the results folder happens to hold, that is not a result.
    std::fs::create_dir_all(d.join("photos")).unwrap();
    std::fs::write(d.join("photos").join("a.jpg"), "x").unwrap();
    assert!(store::remove(&d, "photos").is_err());
    assert!(
        d.join("photos").join("a.jpg").is_file(),
        "a folder that is not a result was removed"
    );
    // One being written is not listed.
    std::fs::create_dir_all(d.join(".x.part-1")).unwrap();
    assert!(store::list(&d).1.iter().all(|(n, _)| !n.starts_with('.')));
    store::remove(&d, "2026-09-01_old_abc.csv").unwrap();
    let _ = std::fs::remove_dir_all(&d);
}
